use clap::Parser;
use lararium_crypto::PrivateSignatureKey;
use lararium_discovery::{Capability, Discovery, Service};
use lararium_store::Store;
use std::net::{Ipv6Addr, SocketAddr};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8080).into())]
    listen_address: SocketAddr,
    #[arg(env, long, default_value = "./data")]
    persistence_dir: Store,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let store = args.persistence_dir;
    init_tracing(&[("lararium_discovery", "info"), ("lararium_station", "info")]);

    let private_key = match store.load("station.key") {
        Ok(private_key) => PrivateSignatureKey::from_pem(&private_key)?,
        Err(lararium_store::Error::NotFound) => {
            let private_key = PrivateSignatureKey::new()?;
            store.save("station.key", private_key.to_pem()?)?;
            private_key
        }
        Err(error) => return Err(error.into()),
    };
    let public_key = private_key.public_key()?.to_raw()?;
    let uid = base32::encode(
        base32::Alphabet::Rfc4648Lower { padding: false },
        &public_key,
    );

    let discovery = Discovery::new()?;
    let _registration = discovery.register(Service {
        uid: &uid,
        port: args.listen_address.port(),
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
