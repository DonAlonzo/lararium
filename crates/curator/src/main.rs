mod media;
use media::MediaSource;

use clap::Parser;
use lararium_mqtt::QoS;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
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

    let mut mqtt_client =
        lararium_mqtt::Client::connect(&format!("{}:{}", &args.gateway_host, args.gateway_port))
            .await?;
    let _ = mqtt_client
        .publish("lararium/curator", b"Hello, world!", QoS::AtMostOnce)
        .await?;

    let file_path = "century.mp4";
    let media_source = Arc::new(MediaSource::new(file_path));
    media_source.play();

    let mut video_stream = loop {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 42000);
        match TcpStream::connect(address).await {
            Ok(stream) => break stream,
            Err(error) => {
                println!("Failed to connect: {error}. Retrying...");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    let mut audio_stream = loop {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 42001);
        match TcpStream::connect(address).await {
            Ok(stream) => break stream,
            Err(error) => {
                println!("Failed to connect: {error}. Retrying...");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    let video_stream_task = tokio::spawn({
        let media_source = media_source.clone();
        async move {
            loop {
                let sample = media_source.pull_video_sample().await;
                let sample = bincode::serialize(&sample).unwrap();
                video_stream
                    .write_all(&(sample.len() as u32).to_be_bytes())
                    .await
                    .unwrap();
                video_stream.write_all(&sample).await.unwrap();
            }
        }
    });

    let audio_stream_task = tokio::spawn({
        let media_source = media_source.clone();
        async move {
            loop {
                let sample = media_source.pull_audio_sample().await;
                let sample = bincode::serialize(&sample).unwrap();
                audio_stream
                    .write_all(&(sample.len() as u32).to_be_bytes())
                    .await
                    .unwrap();
                audio_stream.write_all(&sample).await.unwrap();
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
