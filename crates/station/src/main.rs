mod media;
use media::MediaSink;

use clap::Parser;
use lararium::prelude::*;
use lararium_api::JoinRequest;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_mqtt::QoS;
use lararium_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value = "./data")]
    persistence_dir: Store,
    #[arg(env, long, default_value = "gateway.lararium")]
    gateway_host: String,
    #[arg(env, long, default_value_t = 443)]
    gateway_api_port: u16,
    #[arg(env, long, default_value_t = 1883)]
    gateway_mqtt_port: u16,
    #[arg(env, long, default_value_t = true)]
    use_wayland: bool,
}

#[derive(Serialize, Deserialize)]
struct Bundle {
    private_key: PrivateSignatureKey,
    certificate: Certificate,
    ca: Certificate,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let store = args.persistence_dir;
    init_tracing(&[("lararium_station", "info")]);
    gstreamer::init()?;

    let api_client =
        lararium_api::Client::connect(args.gateway_host.clone(), args.gateway_api_port);

    let bundle = match store.load("bundle") {
        Ok(bundle) => serde_json::from_slice(&bundle)?,
        Err(lararium_store::Error::NotFound) => {
            let private_key = PrivateSignatureKey::new()?;
            let csr = private_key.generate_csr()?;
            let response = api_client.join(JoinRequest { csr }).await?;
            let bundle = Bundle {
                private_key,
                certificate: response.certificate,
                ca: response.ca,
            };
            store.save("bundle", serde_json::to_string(&bundle)?)?;
            bundle
        }
        Err(error) => return Err(error.into()),
    };

    let mqtt_client = lararium_mqtt::Client::connect(&format!(
        "{}:{}",
        &args.gateway_host, args.gateway_mqtt_port
    ))
    .await?;

    mqtt_client
        .subscribe("0000/video/source", QoS::AtLeastOnce)
        .await?;
    mqtt_client
        .subscribe("0000/audio/source", QoS::AtLeastOnce)
        .await?;
    mqtt_client
        .subscribe("0000/status", QoS::AtLeastOnce)
        .await?;

    let (video_src_tx, mut video_src_rx) = mpsc::channel::<String>(1);
    let (audio_src_tx, mut audio_src_rx) = mpsc::channel::<String>(1);

    tokio::spawn({
        let mqtt_client = mqtt_client.clone();
        async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let _ = mqtt_client
                .publish("0000/command/power", &[], QoS::AtMostOnce)
                .await;
        }
    });

    tokio::spawn({
        let mqtt_client = mqtt_client.clone();
        async move {
            loop {
                let message = mqtt_client.poll_message().await.unwrap();
                match message.topic_name.as_str() {
                    "0000/status" => {
                        tracing::info!("Received power command");
                        break;
                    }
                    "0000/video/source" => {
                        let Ok(ciborium::Value::Text(source)) =
                            ciborium::de::from_reader(&message.payload[..])
                        else {
                            tracing::error!("Failed to decode video source");
                            continue;
                        };
                        tracing::info!("Received video source: {source}");
                        let _ = video_src_tx.send(source.to_string()).await;
                    }
                    "0000/audio/source" => {
                        let Ok(ciborium::Value::Text(source)) =
                            ciborium::de::from_reader(&message.payload[..])
                        else {
                            tracing::error!("Failed to decode audio source");
                            continue;
                        };
                        tracing::info!("Received audio source: {source}");
                        let _ = audio_src_tx.send(source.to_string()).await;
                    }
                    _ => tracing::warn!("Unknown topic: {}", message.topic_name),
                }
            }
        }
    });

    if let Ok(status) = api_client.get(&Topic::from_str("0000/status")).await {
        println!("{:?}", status);
    }

    let media_sink = Arc::new(MediaSink::new(args.use_wayland));
    media_sink.play();

    let video_server_task = tokio::spawn({
        let media_sink = media_sink.clone();
        async move {
            while let Some(src) = video_src_rx.recv().await {
                tracing::info!("Connecting video to {src}");
                let stream = match TcpStream::connect(src).await {
                    Ok(stream) => stream,
                    Err(error) => {
                        tracing::error!("Failed to connect: {error}");
                        continue;
                    }
                };
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
                    let sample = ciborium::de::from_reader(&packet_data[..]).unwrap();
                    media_sink.push_video_sample(sample);
                }
            }
        }
    });

    let audio_server_task = tokio::spawn({
        let media_sink = media_sink.clone();
        async move {
            while let Some(src) = audio_src_rx.recv().await {
                tracing::info!("Connecting audio to {src}");
                let stream = match TcpStream::connect(src).await {
                    Ok(stream) => stream,
                    Err(error) => {
                        tracing::error!("Failed to connect: {error}");
                        continue;
                    }
                };
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
                    let sample = ciborium::de::from_reader(&packet_data[..]).unwrap();
                    media_sink.push_audio_sample(sample);
                }
            }
        }
    });

    tokio::select! {
        result = video_server_task => result?,
        result = audio_server_task => result?,
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
