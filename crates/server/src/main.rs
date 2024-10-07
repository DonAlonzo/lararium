use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::net::{Ipv6Addr, SocketAddr};

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 8080).into())]
    listen_address: SocketAddr,
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
    tracing_subscriber::fmt::init();
    let args = Args::parse();

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

    let auth_engine = lararium_auth_engine::Engine::new(pg_pool);
    let auth_server = lararium_auth_tonic::Server::new(auth_engine.clone());
    let auth_server = lararium::AuthServer::new(auth_server);
    let auth_layer = tower::ServiceBuilder::new()
        .layer(lararium_auth_tower::ServerLayer::new(auth_engine))
        .into_inner();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(lararium::DESCRIPTOR_SET)
        .build_v1()?;

    tracing::info!("🚀 Listening to {}", args.listen_address);

    tonic::transport::Server::builder()
        .layer(auth_layer)
        .add_service(auth_server)
        .add_service(reflection_service)
        .serve(args.listen_address)
        .await?;

    Ok(())
}
