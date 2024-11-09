mod media;
use media::MediaSink;

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
use std::sync::Arc;
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

    let media_sink = Arc::new(MediaSink::new(args.use_wayland));
    media_sink.play();

    let video_server_task = tokio::spawn({
        let media_sink = media_sink.clone();
        async move {
            let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42000);
            let listener = TcpListener::bind(listen_address).await.unwrap();
            loop {
                let (stream, _address) = listener.accept().await.unwrap();
                let (mut reader, mut _writer) = stream.into_split();
                loop {
                    let Ok(packet_length) = reader.read_u32().await else {
                        break;
                    };
                    let mut packet_data = vec![0; packet_length as usize];
                    let Ok(bytes_read) = reader.read_exact(&mut packet_data).await else {
                        break;
                    };
                    if bytes_read == 0 {
                        break;
                    }
                    let sample = bincode::deserialize(&packet_data).unwrap();
                    media_sink.push_video_sample(sample);
                }
            }
        }
    });

    let audio_server_task = tokio::spawn({
        let media_sink = media_sink.clone();
        async move {
            let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42001);
            let listener = TcpListener::bind(listen_address).await.unwrap();
            loop {
                let (stream, _address) = listener.accept().await.unwrap();
                let (mut reader, mut _writer) = stream.into_split();
                loop {
                    let Ok(packet_length) = reader.read_u32().await else {
                        break;
                    };
                    let mut packet_data = vec![0; packet_length as usize];
                    let Ok(bytes_read) = reader.read_exact(&mut packet_data).await else {
                        break;
                    };
                    if bytes_read == 0 {
                        break;
                    }
                    let sample = bincode::deserialize(&packet_data).unwrap();
                    media_sink.push_audio_sample(sample);
                }
            }
        }
    });

    tokio::select! {
        _ = video_server_task => (),
        _ = audio_server_task => (),
        _ = tokio::signal::ctrl_c() => (),
    };
    tracing::info!("Shutting down...");
    mqtt_client.disconnect().await?;
    media_sink.stop();

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
