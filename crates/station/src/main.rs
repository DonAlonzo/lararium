mod prelude;

use clap::Parser;
use derive_more::From;
use lararium_station::{RunArgs, Station};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value = "./modules")]
    modules_dir: PathBuf,
    #[arg(env, long, default_value = "gateway.lararium")]
    gateway_host: String,
    #[arg(env, long, default_value_t = 443)]
    gateway_api_port: u16,
    #[arg(env, long, default_value_t = 1883)]
    gateway_mqtt_port: u16,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    init_tracing(&[("lararium_station", "debug")]);

    let api = api::Client::connect(&args.gateway_host, args.gateway_api_port);
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
