mod prelude;

use clap::Parser;
use derive_more::From;
use lararium_api::JoinRequest;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_station::{RunArgs, Station};
use lararium_store::Store;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
    init_tracing(&[("lararium_station", "debug")]);

    let api = lararium_api::Client::connect(&args.gateway_host, args.gateway_api_port);
    let bundle = match store.load("bundle") {
        Ok(bundle) => serde_json::from_slice(&bundle)?,
        Err(lararium_store::Error::NotFound) => {
            let private_key = PrivateSignatureKey::new()?;
            let csr = private_key.generate_csr()?;
            let response = api.join(JoinRequest { csr }).await?;
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
    let station = Station::new()?;

    let kodi_handle = tokio::spawn({
        let wasm = std::fs::read("target/wasm32-wasip2/release/kodi.wasm")?;
        let station = station.clone();
        async move {
            station
                .run(RunArgs {
                    root_dir: PathBuf::from("/tmp/rootfs"),
                    wasm: &wasm,
                    name: "kodi",
                    node_name: "rpi5",
                    gateway: &args.gateway_host,
                    mqtt_port: args.gateway_mqtt_port,
                })
                .await?;
            Ok::<(), color_eyre::Report>(())
        }
    });

    tokio::select! {
        result = kodi_handle => result??,
        _ = tokio::signal::ctrl_c() => (),
    };
    tracing::info!("Shutting down...");

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
