use clap::Parser;
use lararium_discovery::{Capability, Discovery, Service};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _args = Args::parse();
    init_tracing(&[("lararium_discovery", "info"), ("lararium_station", "info")]);

    let discovery = Discovery::new()?;
    let _registration = discovery.register(Service {
        name: "station",
        port: 10101,
        capability: Capability::Station,
    })?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => (),
    }

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
