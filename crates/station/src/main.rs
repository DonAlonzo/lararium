use clap::Parser;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use lararium::*;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_mqtt::QoS;
use lararium_store::Store;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8081).into())]
    listen_address: SocketAddr,
    #[arg(env, long, default_value = "./data")]
    persistence_dir: Store,
    #[arg(env, long, default_value = "gateway.lararium")]
    gateway_host: String,
    #[arg(env, long, default_value_t = 1883)]
    gateway_port: u16,
    #[arg(env, long, default_value_t = 8080)]
    gateway_admittance_port: u16,
    #[arg(env, long, default_value_t = true)]
    use_wayland: bool,
}

#[derive(Serialize, Deserialize)]
struct Bundle {
    private_key: Vec<u8>,
    certificate: Vec<u8>,
    ca: Vec<u8>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let store = args.persistence_dir;
    init_tracing(&[("lararium_station", "info")]);
    gst::init()?;

    let bundle = match store.load("bundle") {
        Ok(bundle) => serde_json::from_slice(&bundle)?,
        Err(lararium_store::Error::NotFound) => {
            let private_key = PrivateSignatureKey::new()?;
            let mut admittance = AdmittanceClient::connect(format!(
                "grpc://{}:{}",
                args.gateway_host, args.gateway_admittance_port
            ))
            .await?;
            let csr = private_key.generate_csr()?.to_pem()?;
            let csr = String::from_utf8(csr)?;
            let JoinResponse { certificate, ca } =
                admittance.join(JoinRequest { csr }).await?.into_inner();
            let certificate = Certificate::from_pem(certificate.as_bytes())?;
            let ca = Certificate::from_pem(ca.as_bytes())?;
            let bundle = Bundle {
                private_key: private_key.to_pem()?,
                certificate: certificate.to_pem()?,
                ca: ca.to_pem()?,
            };
            store.save("bundle", serde_json::to_string(&bundle)?)?;
            bundle
        }
        Err(error) => return Err(error.into()),
    };

    let mut mqtt_client =
        lararium_mqtt::Client::connect(&format!("{}:{}", &args.gateway_host, args.gateway_port))
            .await?;
    let _ = mqtt_client
        .publish(
            "lararium/station",
            b"Hello, world! Greetings from outer space \xF0\x9F\x9A\x80",
            QoS::AtMostOnce,
        )
        .await?;

    let pipeline = gst::Pipeline::new();

    let video_src = gst::ElementFactory::make("appsrc")
        .name("video_src")
        .build()?;
    let video_decode = gst::ElementFactory::make("decodebin")
        .name("video_decode")
        .build()?;
    let video_queue = gst::ElementFactory::make("queue")
        .name("video_queue")
        .property("max-size-time", 3_000_000_000u64)
        .build()?;
    let video_convert = gst::ElementFactory::make("videoconvert").build()?;
    let video_scale = gst::ElementFactory::make("videoscale").build()?;
    let video_sink = if args.use_wayland {
        gst::ElementFactory::make("waylandsink").build()?
    } else {
        gst::ElementFactory::make("kmssink")
            //.property("bus-id", "PCI:0000:01:00.0")
            .build()?
    };
    pipeline.add_many(&[
        &video_src,
        &video_decode,
        &video_queue,
        &video_convert,
        &video_scale,
        &video_sink,
    ])?;
    let video_src = video_src.dynamic_cast::<gst_app::AppSrc>().unwrap();
    video_src.set_stream_type(gst_app::AppStreamType::Stream);
    video_src.set_format(gst::Format::Time);
    video_src.set_property("is-live", &true);
    video_src.set_property("do-timestamp", &true);
    video_src.link(&video_decode)?;
    video_decode.connect_pad_added({
        let video_queue = video_queue.clone();
        move |_, src_pad| {
            let sink_pad = video_queue.static_pad("sink").unwrap();
            if sink_pad.is_linked() {
                return;
            }
            src_pad.link(&sink_pad).unwrap();
        }
    });
    video_queue.sync_state_with_parent()?;
    video_queue.link(&video_convert)?;
    video_convert.link(&video_scale)?;
    video_scale.link(&video_sink)?;
    video_sink.set_property("sync", &true);

    let audio_src = gst::ElementFactory::make("appsrc")
        .name("audio_src")
        .build()?;
    let audio_decode = gst::ElementFactory::make("decodebin")
        .name("audio_decode")
        .build()?;
    let audio_queue = gst::ElementFactory::make("queue")
        .name("audio_queue")
        .property("max-size-time", 3_000_000_000u64)
        .build()?;
    let audio_convert = gst::ElementFactory::make("audioconvert").build()?;
    let audio_resample = gst::ElementFactory::make("audioresample").build()?;
    let audio_sink = gst::ElementFactory::make("alsasink").build()?;
    pipeline.add_many(&[
        &audio_src,
        &audio_decode,
        &audio_queue,
        &audio_convert,
        &audio_resample,
        &audio_sink,
    ])?;
    let audio_src = audio_src.dynamic_cast::<gst_app::AppSrc>().unwrap();
    audio_src.set_stream_type(gst_app::AppStreamType::Stream);
    audio_src.set_format(gst::Format::Time);
    audio_src.set_property("is-live", &true);
    audio_src.set_property("do-timestamp", &true);
    audio_src.link(&audio_decode)?;
    audio_decode.connect_pad_added({
        let audio_queue = audio_queue.clone();
        move |_, src_pad| {
            let sink_pad = audio_queue.static_pad("sink").unwrap();
            if sink_pad.is_linked() {
                return;
            }
            src_pad.link(&sink_pad).unwrap();
        }
    });
    audio_queue.sync_state_with_parent()?;
    audio_queue.link(&audio_convert)?;
    audio_convert.link(&audio_resample)?;
    audio_resample.link(&audio_sink)?;
    audio_sink.set_property("sync", &true);

    pipeline.set_state(gst::State::Playing)?;

    let video_server_task = tokio::spawn({
        async move {
            let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42000);
            let listener = TcpListener::bind(listen_address).await.unwrap();
            loop {
                let (stream, _address) = listener.accept().await.unwrap();
                let (mut reader, mut _writer) = stream.into_split();
                loop {
                    let Ok(caps_length) = reader.read_u32().await else {
                        break;
                    };
                    let mut caps = vec![0; caps_length as usize];
                    let Ok(bytes_read) = reader.read_exact(&mut caps).await else {
                        break;
                    };
                    if bytes_read == 0 {
                        break;
                    }
                    let Ok(caps) = String::from_utf8(caps) else {
                        break;
                    };
                    let Ok(caps) = gst::Caps::from_str(&caps) else {
                        break;
                    };
                    video_src.set_caps(Some(&caps));
                    let Ok(frame_length) = reader.read_u32().await else {
                        break;
                    };
                    let mut frame = vec![0; frame_length as usize];
                    let Ok(bytes_read) = reader.read_exact(&mut frame).await else {
                        break;
                    };
                    if bytes_read == 0 {
                        break;
                    }
                    let buffer = gst::Buffer::from_mut_slice(frame);
                    if let Err(error) = video_src.push_buffer(buffer) {
                        eprintln!("Failed to push buffer: {error}");
                        break;
                    }
                }
            }
        }
    });

    let audio_server_task = tokio::spawn({
        async move {
            let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42001);
            let listener = TcpListener::bind(listen_address).await.unwrap();
            loop {
                let (stream, _address) = listener.accept().await.unwrap();
                let (mut reader, mut _writer) = stream.into_split();
                loop {
                    let Ok(caps_length) = reader.read_u32().await else {
                        break;
                    };
                    let mut caps = vec![0; caps_length as usize];
                    let Ok(bytes_read) = reader.read_exact(&mut caps).await else {
                        break;
                    };
                    if bytes_read == 0 {
                        break;
                    }
                    let Ok(caps) = String::from_utf8(caps) else {
                        break;
                    };
                    let Ok(caps) = gst::Caps::from_str(&caps) else {
                        break;
                    };
                    audio_src.set_caps(Some(&caps));
                    let Ok(buffer_length) = reader.read_u32().await else {
                        break;
                    };
                    let mut buffer = vec![0; buffer_length as usize];
                    let Ok(bytes_read) = reader.read_exact(&mut buffer).await else {
                        break;
                    };
                    if bytes_read == 0 {
                        break;
                    }
                    let buffer = gst::Buffer::from_mut_slice(buffer);
                    if let Err(error) = audio_src.push_buffer(buffer) {
                        eprintln!("Failed to push buffer: {error}");
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = video_server_task => (),
        _ = audio_server_task => (),
        _ = tokio::signal::ctrl_c() => (),
    };
    println!("Shutting down...");
    //mqtt_client.disconnect().await?;
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

fn init_tracing(filter: &[(&str, &str)]) {
    let filter = filter
        .iter()
        .map(|(name, level)| format!("{}={}", name, level))
        .collect::<Vec<_>>()
        .join(",");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();
}
