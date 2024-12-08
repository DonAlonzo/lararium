mod container;
mod prelude;

use clap::Parser;
use container::ContainerBlueprint;
use lararium::prelude::*;
use lararium_api::JoinRequest;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_mqtt::QoS;
use lararium_store::Store;
use serde::{Deserialize, Serialize};
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
        .subscribe(Topic::from_str("0000/status"), QoS::AtLeastOnce)
        .await?;

    let container = ContainerBlueprint {
        rootfs_path: std::path::PathBuf::from("/tmp/rootfs"),
        work_dir: std::path::PathBuf::from("/"),
        command: String::from("/usr/bin/kodi"),
        args: vec![String::from("kodi")],
        env: vec![
            (String::from("DISPLAY"), String::from(":0")),
            (String::from("HOME"), String::from("/home/donalonzo")),
            (String::from("PATH"), String::from("/bin")),
            (String::from("WAYLAND_DISPLAY"), String::from("wayland-1")),
            (
                String::from("XDG_RUNTIME_DIR"),
                String::from("/run/user/1001"),
            ),
        ],
        // $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY
        // /tmp/.X11-unix
        // /etc/group
        // /etc/passwd
        // /home/donalonzo
        hostname: String::from("busy-container"),
        gid: 1001,
        uid: 1001,
    };

    let _container_handle = container.run();

    tokio::spawn({
        let mqtt_client = mqtt_client.clone();
        async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let _ = mqtt_client
                .publish(
                    Topic::from_str("0000/command/power"),
                    Value::Null,
                    QoS::AtMostOnce,
                )
                .await;
        }
    });

    tokio::spawn({
        let mqtt_client = mqtt_client.clone();
        async move {
            loop {
                let message = mqtt_client.poll_message().await.unwrap();
                match message.topic.to_string().as_str() {
                    "0000/status" => {
                        tracing::info!("Received power command");
                        break;
                    }
                    _ => tracing::warn!("Unknown topic: {}", message.topic),
                }
            }
        }
    });

    if let Ok(status) = api_client.get(&Topic::from_str("0000/status")).await {
        println!("{:?}", status);
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => (),
    };
    tracing::info!("Shutting down...");
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
