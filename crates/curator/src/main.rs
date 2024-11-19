mod media;
use media::MediaSource;

use clap::Parser;
use lararium_mqtt::QoS;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value = "gateway.lararium")]
    gateway_host: String,
    #[arg(env, long, default_value_t = 1883)]
    gateway_port: u16,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    init_tracing(&[("lararium_curator", "info")]);
    gstreamer::init()?;

    let mqtt_client =
        lararium_mqtt::Client::connect(&format!("{}:{}", &args.gateway_host, args.gateway_port))
            .await?;
    let _ = mqtt_client
        .publish("lararium/curator", b"Hello, world!", QoS::AtMostOnce)
        .await?;

    let file_path = "century.mp4";
    let media_source = Arc::new(MediaSource::new(file_path));

    let video_stream_task = tokio::spawn({
        let media_source = media_source.clone();
        async move {
            let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42000);
            let listener = TcpListener::bind(listen_address).await.unwrap();
            loop {
                let (stream, _address) = listener.accept().await.unwrap();
                let (mut _reader, mut writer) = stream.into_split();
                media_source.play();
                loop {
                    let sample = media_source.pull_video_sample().await;
                    let sample = bincode::serialize(&sample).unwrap();
                    if let Err(_) = writer.write_all(&(sample.len() as u32).to_be_bytes()).await {
                        break;
                    };
                    if let Err(_) = writer.write_all(&sample).await {
                        break;
                    };
                }
                media_source.pause();
            }
        }
    });

    let audio_stream_task = tokio::spawn({
        let media_source = media_source.clone();
        async move {
            let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42001);
            let listener = TcpListener::bind(listen_address).await.unwrap();
            media_source.play();
            loop {
                let (stream, _address) = listener.accept().await.unwrap();
                let (mut _reader, mut writer) = stream.into_split();
                media_source.play();
                loop {
                    let sample = media_source.pull_audio_sample().await;
                    let sample = bincode::serialize(&sample).unwrap();
                    if let Err(_) = writer.write_all(&(sample.len() as u32).to_be_bytes()).await {
                        break;
                    };
                    if let Err(_) = writer.write_all(&sample).await {
                        break;
                    };
                }
                media_source.pause();
            }
        }
    });

    tokio::select! {
        _ = video_stream_task => (),
        _ = audio_stream_task => (),
        _ = tokio::signal::ctrl_c() => (),
    };
    tracing::info!("Shutting down...");
    media_source.stop();
    mqtt_client.disconnect().await?;

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
