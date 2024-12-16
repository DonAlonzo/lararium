mod container;
mod prelude;

use clap::Parser;
use container::{ContainerBlueprint, ContainerHandle};
use lararium::prelude::*;
use lararium_api::JoinRequest;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_mqtt::QoS;
use lararium_store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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

#[derive(Clone)]
struct Station {
    blueprints: HashMap<String, ContainerBlueprint>,
    handles: Arc<RwLock<HashMap<String, ContainerHandle>>>,
}

impl Station {
    fn new() -> Self {
        Self {
            blueprints: HashMap::new(),
            handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn add(
        &mut self,
        name: impl Into<String>,
        blueprint: ContainerBlueprint,
    ) {
        let _ = self.blueprints.insert(name.into(), blueprint);
    }

    async fn run(
        &self,
        name: &str,
    ) {
        tracing::debug!("Starting container {name}");
        let blueprint = self.blueprints.get(name).unwrap();
        let handle = blueprint.run().unwrap();
        self.handles.write().await.insert(name.to_string(), handle);
    }

    async fn kill(
        &self,
        name: &str,
    ) {
        tracing::debug!("Killing container {name}");
        self.handles.write().await.remove(name);
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let store = args.persistence_dir;
    init_tracing(&[("lararium_station", "debug")]);

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
        .subscribe(
            Topic::from_str("tv/containers/kodi/power"),
            QoS::AtLeastOnce,
        )
        .await?;

    mqtt_client
        .subscribe(
            Topic::from_str("tv/containers/kodi/status"),
            QoS::AtLeastOnce,
        )
        .await?;

    let mut station = Station::new();
    station.add(
        "kodi",
        ContainerBlueprint {
            rootfs_path: std::path::PathBuf::from("/tmp/rootfs"),
            work_dir: std::path::PathBuf::from("/"),
            command: String::from("/usr/bin/kodi"),
            args: vec![String::from("kodi")],
            env: vec![(String::from("PATH"), String::from("/bin"))],
            hostname: String::from("kodi"),
            username: String::from("lararium"),
            gid: 1000,
            uid: 1000,
        },
    );

    if let Ok(Entry::Record {
        value: Value::Boolean(mut status),
        ..
    }) = api_client
        .get(&Topic::from_str("tv/containers/kodi/status"))
        .await
    {
        if status {
            station.run("kodi").await;
        }

        tokio::spawn({
            let mqtt_client = mqtt_client.clone();
            async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                    status = !status;
                    let _ = mqtt_client
                        .publish(
                            Topic::from_str("tv/containers/kodi/status"),
                            Value::Boolean(status),
                            QoS::AtMostOnce,
                        )
                        .await;
                }
            }
        });
    };

    tokio::spawn({
        let station = station.clone();
        let mqtt_client = mqtt_client.clone();
        async move {
            loop {
                let Ok(message) = mqtt_client.poll_message().await else {
                    break;
                };
                match message.topic.to_string().as_str() {
                    "tv/containers/kodi/status" => {
                        let Value::Boolean(status) = message.payload else {
                            continue;
                        };
                        if status {
                            station.run("kodi").await;
                        } else {
                            station.kill("kodi").await;
                        }
                    }
                    _ => tracing::warn!("Unknown topic: {}", message.topic),
                }
            }
        }
    });

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
