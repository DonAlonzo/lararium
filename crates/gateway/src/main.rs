use clap::Parser;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use sqlx::postgres::PgPoolOptions;
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
    #[arg(env, long)]
    tls_private_key_path: PathBuf,
    #[arg(env, long)]
    tls_certificate_path: PathBuf,
    #[arg(env, long)]
    ca_path: PathBuf,
    #[arg(env, long, default_value = "localhost")]
    postgres_host: String,
    #[arg(env, long, default_value_t = 5432)]
    postgres_port: u16,
    #[arg(env, long, default_value = "lararium")]
    postgres_database: String,
    #[arg(env, long, default_value = "postgres")]
    postgres_username: String,
    #[arg(env, long, default_value = "password")]
    postgres_password: String,
    #[arg(env, long, default_value_t = 500)]
    postgres_max_connections: u32,
    #[arg(env, long, default_value = "localhost")]
    mqtt_host: String,
    #[arg(env, long, default_value_t = 1883)]
    mqtt_port: u16,
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
        ("lararium_discovery", "info"),
    ]);

    let private_key = tokio::fs::read(&args.private_key_path).await?;
    let private_key = PrivateSignatureKey::from_pem(&private_key)?;
    let certificate = tokio::fs::read(&args.certificate_path).await?;
    let certificate = Certificate::from_pem(&certificate)?;
    let identity = private_key.clone().into_identity(certificate.clone())?;

    let tls_private_key = tokio::fs::read_to_string(&args.tls_private_key_path).await?;
    let tls_certificate = tokio::fs::read_to_string(&args.tls_certificate_path).await?;

    let ca_certificate = tokio::fs::read(&args.ca_path).await?;
    let ca_certificate = Certificate::from_pem(&ca_certificate)?;

    let pg_pool = PgPoolOptions::new()
        .max_connections(args.postgres_max_connections)
        .connect_lazy(&format!(
            "postgresql://{username}:{password}@{host}:{port}/{database}",
            host = &args.postgres_host,
            port = &args.postgres_port,
            database = &args.postgres_database,
            username = &args.postgres_username,
            password = &args.postgres_password,
        ))?;

    let engine = lararium_gateway_engine::Engine::new(
        pg_pool,
        identity,
        String::from_utf8(ca_certificate.to_pem()?)?,
    );

    let admittance_engine = engine.clone();
    let admittance_server = tokio::spawn(async move {
        let admittance_server = lararium_gateway_tonic::Admittance::new(admittance_engine);
        let admittance_server = lararium::AdmittanceServer::new(admittance_server);

        tracing::info!(
            "🚀 Listening to admittance requests: {}",
            args.admittance_listen_address
        );

        Server::builder()
            .add_service(admittance_server)
            .serve(args.admittance_listen_address)
            .await?;

        Ok::<(), color_eyre::Report>(())
    });

    let gateway_server = tokio::spawn(async move {
        let gateway_server = lararium_gateway_tonic::Gateway::new(engine.clone());
        let gateway_server = lararium::GatewayServer::new(gateway_server);
        let gateway_layer = tower::ServiceBuilder::new()
            .layer(lararium_gateway_tower::ServerLayer::new(engine))
            .into_inner();

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(lararium::DESCRIPTOR_SET)
            .build_v1()?;

        let tls_config = ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(
                tls_certificate,
                tls_private_key,
            ))
            .client_ca_root(tonic::transport::Certificate::from_pem(
                ca_certificate.to_pem()?,
            ));

        tracing::info!("🚀 Listening to requests: {}", args.listen_address);

        Server::builder()
            .tls_config(tls_config)?
            .layer(gateway_layer)
            .add_service(gateway_server)
            .add_service(reflection_service)
            .serve(args.listen_address)
            .await?;

        Ok::<(), color_eyre::Report>(())
    });

    tokio::select! {
        _ = admittance_server => (),
        _ = gateway_server => (),
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
