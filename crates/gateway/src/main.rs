use clap::Parser;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use std::net::{Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use tonic::transport::{Server, ServerTlsConfig};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8443).into())]
    listen_address: SocketAddr,
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8080).into())]
    admittance_listen_address: SocketAddr,
    #[arg(env, long)]
    private_key_path: PathBuf,
    #[arg(env, long)]
    certificate_path: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    init_tracing(&[
        ("lararium_gateway", "info"),
        ("lararium_gateway_tonic", "info"),
        ("lararium_gateway_tower", "info"),
        ("lararium_gateway", "info"),
        ("lararium_mqtt", "info"),
    ]);

    let private_key = tokio::fs::read(&args.private_key_path).await?;
    let private_key = PrivateSignatureKey::from_pem(&private_key)?;
    let certificate = tokio::fs::read(&args.certificate_path).await?;
    let certificate = Certificate::from_pem(&certificate)?;
    let identity = private_key.clone().into_identity(certificate.clone())?;

    let tls_private_key = PrivateSignatureKey::new()?;
    let csr = tls_private_key.generate_csr()?;
    let tls_certificate = identity.sign_csr(&csr, "gateway.lararium")?;

    let engine =
        lararium_gateway_engine::Engine::new(identity, String::from_utf8(certificate.to_pem()?)?);

    let admittance_engine = engine.clone();
    let admittance_server = tokio::spawn(async move {
        let admittance_server = lararium_gateway_tonic::Admittance::new(admittance_engine);
        let admittance_server = lararium::AdmittanceServer::new(admittance_server);

        tracing::info!(
            "🎟️ Listening for CSR requests: {}",
            args.admittance_listen_address
        );

        Server::builder()
            .add_service(admittance_server)
            .serve(args.admittance_listen_address)
            .await?;

        tracing::info!("🛑 Admittance server stopped");

        Ok::<(), color_eyre::Report>(())
    });

    let gateway_server = tokio::spawn(async move {
        let gateway_server = lararium_gateway_tonic::Gateway::new(engine);
        let gateway_server = lararium::GatewayServer::new(gateway_server);
        let gateway_layer = tower::ServiceBuilder::new()
            .layer(lararium_gateway_tower::ServerLayer::new())
            .into_inner();

        let tls_config = ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(
                tls_certificate.to_pem()?,
                tls_private_key.to_pem()?,
            ))
            .client_ca_root(tonic::transport::Certificate::from_pem(
                certificate.to_pem()?,
            ));

        tracing::info!("🚀 Listening for gRPCs requests: {}", args.listen_address);

        Server::builder()
            .tls_config(tls_config)?
            .layer(gateway_layer)
            .add_service(gateway_server)
            .serve(args.listen_address)
            .await?;

        tracing::info!("🛑 Gateway server stopped");

        Ok::<(), color_eyre::Report>(())
    });

    let mqtt_server = tokio::spawn(async move {
        let server = lararium_mqtt::Server::new();

        tracing::info!("📫 Listening for MQTT requests");

        server.listen((Ipv6Addr::UNSPECIFIED, 1883).into()).await?;

        tracing::info!("🛑 MQTT server stopped");

        Ok::<(), color_eyre::Report>(())
    });

    tokio::select! {
        result = admittance_server => result??,
        result = gateway_server => result??,
        result = mqtt_server => result??,
        _ = tokio::signal::ctrl_c() => (),
    }

    tracing::info!("🛑 Stopping");

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
