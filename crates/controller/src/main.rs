use clap::Parser;
use futures::StreamExt;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_discovery::{Capability, Discovery, Event, Service};
use sqlx::postgres::PgPoolOptions;
use std::net::{Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use tonic::transport::{Server, ServerTlsConfig};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8080).into())]
    listen_address: SocketAddr,
    #[arg(env, long)]
    private_key_path: PathBuf,
    #[arg(env, long)]
    certificate_path: PathBuf,
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
        ("lararium_controller", "info"),
        ("lararium_controller_tonic", "info"),
        ("lararium_controller_tower", "info"),
        ("lararium_controller", "info"),
        ("lararium_discovery", "info"),
    ]);

    let private_key = tokio::fs::read(&args.private_key_path).await?;
    let private_key = PrivateSignatureKey::from_pem(&private_key)?;

    let certificate = tokio::fs::read(&args.certificate_path).await?;
    let certificate = Certificate::from_pem(&certificate)?;

    let ca_certificate = tokio::fs::read(&args.ca_path).await?;
    let ca_certificate = Certificate::from_pem(&ca_certificate)?;

    let uid = base32::encode(
        base32::Alphabet::Rfc4648Lower { padding: false },
        &private_key.public_key()?.to_raw()?,
    );

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

    let controller_engine = lararium_controller_engine::Engine::new(pg_pool);
    let controller_server = lararium_controller_tonic::Server::new(controller_engine.clone());
    let controller_server = lararium::ControllerServer::new(controller_server);
    let controller_layer = tower::ServiceBuilder::new()
        .layer(lararium_controller_tower::ServerLayer::new(controller_engine))
        .into_inner();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(lararium::DESCRIPTOR_SET)
        .build_v1()?;

    let server_task = tokio::spawn(async move {
        tracing::info!("🚀 Listening to {}", args.listen_address);

        let tls_config = ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(
                certificate.to_pem()?,
                private_key.to_pem()?,
            ))
            .client_ca_root(tonic::transport::Certificate::from_pem(
                ca_certificate.to_pem()?,
            ));

        Server::builder()
            .tls_config(tls_config)?
            .layer(controller_layer)
            .add_service(controller_server)
            .add_service(reflection_service)
            .serve(args.listen_address)
            .await?;

        Ok::<(), color_eyre::Report>(())
    });

    let discovery = Discovery::new()?;
    let _registration = discovery.register(Service {
        uid: &uid,
        port: args.listen_address.port(),
        capability: Capability::Controller,
    })?;
    let discovery_task = tokio::spawn(async move {
        tracing::info!("🔭 Discovering other devices.");
        let mut events = discovery.listen()?;
        while let Some(Ok(event)) = events.next().await {
            match event {
                Event::ServiceFound { name, .. } => {
                    tracing::info!("[{name}] found");
                }
                Event::ServiceResolved { name, .. } => {
                    tracing::info!("[{name}] resolved");
                }
                Event::ServiceLost { name, .. } => {
                    tracing::info!("[{name}] lost");
                }
            }
        }
        Ok::<(), color_eyre::Report>(())
    });

    tokio::select! {
        _ = discovery_task => (),
        _ = server_task => (),
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
