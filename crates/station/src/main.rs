use clap::Parser;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_discovery::{Capability, Discovery, Service};
use lararium_store::Store;
use std::net::{Ipv6Addr, SocketAddr};
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8081).into())]
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
    let certificate = match store.load("station.crt") {
        Ok(certificate) => Certificate::from_pem(&certificate)?,
        Err(lararium_store::Error::NotFound) => {
            let _registration = discovery.register(Service {
                uid: &uid,
                port: args.listen_address.port(),
                capability: Capability::Station,
            })?;

            let adoption_server = lararium_adoption_tonic::Server::new();
            let adoption_server_clone = adoption_server.clone();
            let certificate = tokio::spawn(async move {
                let certificate = adoption_server_clone.wait_for_adoption().await?;

                Ok::<_, color_eyre::Report>(certificate)
            });

            let adoption_server = lararium::AdoptionServer::new(adoption_server);
            let server_task = tokio::spawn(async move {
                tracing::info!(
                    "🙋 Listening for adoption requests on {}",
                    args.listen_address
                );

                Server::builder()
                    .add_service(adoption_server)
                    .serve(args.listen_address)
                    .await?;

                Ok::<(), color_eyre::Report>(())
            });

            let certificate = tokio::select! {
                certificate = certificate => certificate??,
                _ = server_task => return Ok(()),
                _ = tokio::signal::ctrl_c() => return Ok(()),
            };

            store.save("station.crt", certificate.to_pem()?)?;
            certificate
        }
        Err(error) => return Err(error.into()),
    };

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
