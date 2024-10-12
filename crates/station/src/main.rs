use clap::Parser;
use lararium::*;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_library_tonic::Library;
use lararium_store::Store;
use serde::{Deserialize, Serialize};
use std::net::{Ipv6Addr, SocketAddr};
use tonic::transport::{Channel, ClientTlsConfig, Server, ServerTlsConfig};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8081).into())]
    listen_address: SocketAddr,
    #[arg(env, long, default_value = "./data")]
    persistence_dir: Store,
    #[arg(env, long, default_value = "gateway.lararium")]
    gateway_host: String,
    #[arg(env, long, default_value_t = 8443)]
    gateway_port: u16,
    #[arg(env, long, default_value_t = 8080)]
    gateway_admittance_port: u16,
}

#[derive(Serialize, Deserialize)]
struct Bundle {
    private_key: Vec<u8>,
    certificate: Vec<u8>,
    ca: Vec<u8>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let store = args.persistence_dir;
    init_tracing(&[("lararium_discovery", "info"), ("lararium_station", "info")]);

    let bundle = match store.load("bundle") {
        Ok(bundle) => serde_json::from_slice(&bundle)?,
        Err(lararium_store::Error::NotFound) => {
            let private_key = PrivateSignatureKey::new()?;
            let mut admittance = AdmittanceClient::connect(format!(
                "grpc://{}:{}",
                args.gateway_host, args.gateway_admittance_port
            ))
            .await?;
            let csr = private_key
                .generate_certificate_signing_request()?
                .to_pem()?;
            let csr = String::from_utf8(csr)?;
            let JoinResponse { certificate, ca } =
                admittance.join(JoinRequest { csr }).await?.into_inner();
            let certificate = Certificate::from_pem(certificate.as_bytes())?;
            let ca = Certificate::from_pem(ca.as_bytes())?;
            let bundle = Bundle {
                private_key: private_key.to_pem()?,
                certificate: certificate.to_pem()?,
                ca: ca.to_pem()?,
            };
            store.save("bundle", serde_json::to_string(&bundle)?)?;
            bundle
        }
        Err(error) => return Err(error.into()),
    };
    let identity = tonic::transport::Identity::from_pem(bundle.certificate, bundle.private_key);
    let ca = tonic::transport::Certificate::from_pem(bundle.ca);
    let server_task_identity = identity.clone();
    let server_task_ca = ca.clone();
    let server_task = tokio::spawn(async move {
        let library = LibraryServer::new(Library::new());

        let tls_config = ServerTlsConfig::new()
            .identity(server_task_identity)
            .client_ca_root(server_task_ca);

        tracing::info!("🚀 Listening to requests: {}", args.listen_address);

        Server::builder()
            .tls_config(tls_config)?
            .add_service(library)
            .serve(args.listen_address)
            .await?;

        Ok::<(), color_eyre::Report>(())
    });

    let tls = ClientTlsConfig::new()
        .identity(identity)
        .ca_certificate(ca)
        .domain_name(&args.gateway_host);
    println!("Connecting to gateway at {}:{}", args.gateway_host, args.gateway_port);
    let channel = Channel::from_shared(format!(
        "https://{}:{}",
        args.gateway_host, args.gateway_port
    ))?
    .tls_config(tls)?
    .connect()
    .await?;
    let mut gateway = GatewayClient::new(channel);

    gateway.check_in(CheckInRequest {}).await?;

    tokio::select! {
        _ = server_task => (),
        _ = tokio::signal::ctrl_c() => (),
    }

    gateway.check_out(CheckOutRequest {}).await?;

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
