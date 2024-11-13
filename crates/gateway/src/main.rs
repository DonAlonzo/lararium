use clap::Parser;
use lararium_crypto::{Certificate, PrivateSignatureKey};
use lararium_gateway::Gateway;
use std::net::{Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 443).into())]
    api_listen_address: SocketAddr,
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 67).into())]
    dhcp_listen_address: SocketAddr,
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 53).into())]
    dns_listen_address: SocketAddr,
    #[arg(env, long, default_value_t = (Ipv6Addr::UNSPECIFIED, 1883).into())]
    mqtt_listen_address: SocketAddr,
    #[arg(env, long)]
    ca_path: PathBuf,
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
        ("lararium_api", "info"),
        ("lararium_dhcp", "info"),
        ("lararium_dns", "info"),
        ("lararium_gateway", "debug"),
        ("lararium_mqtt", "info"),
        ("lararium_registry", "debug"),
    ]);

    let ca = tokio::fs::read(&args.ca_path).await?;
    let ca = Certificate::from_pem(&ca)?;
    let private_key = tokio::fs::read(&args.private_key_path).await?;
    let private_key = PrivateSignatureKey::from_pem(&private_key)?;
    let certificate = tokio::fs::read(&args.certificate_path).await?;
    let certificate = Certificate::from_pem(&certificate)?;
    let identity = private_key.clone().into_identity(certificate.clone())?;
    let tls_private_key = PrivateSignatureKey::new()?;
    let tls_csr = tls_private_key.generate_csr()?;
    let tls_certificate = identity.sign_csr(&tls_csr, "gateway.lararium")?;

    let api_server =
        lararium_api::Server::bind(args.api_listen_address, tls_private_key, tls_certificate)
            .await?;
    let mqtt_server = lararium_mqtt::Server::bind(args.mqtt_listen_address).await?;
    let dns_server = lararium_dns::Server::bind(args.dns_listen_address).await?;
    let dhcp_server = lararium_dhcp::Server::bind(args.dhcp_listen_address).await?;

    let gateway = Gateway::new(ca, identity.clone()).await;

    let api_server = tokio::spawn({
        let gateway = gateway.clone();
        async move {
            tracing::info!("🛎️ Listening for API requests: {}", args.api_listen_address);
            api_server.listen(gateway).await?;
            tracing::info!("🛑 API server stopped");
            Ok::<(), color_eyre::Report>(())
        }
    });

    let mqtt_server = tokio::spawn({
        let gateway = gateway.clone();
        async move {
            tracing::info!(
                "📫 Listening for MQTT requests: {}",
                args.mqtt_listen_address
            );
            mqtt_server.listen(gateway).await?;
            tracing::info!("🛑 MQTT server stopped");
            Ok::<(), color_eyre::Report>(())
        }
    });

    let dns_server = tokio::spawn({
        let gateway = gateway.clone();
        async move {
            tracing::info!("🪪 Listening for DNS requests: {}", args.dns_listen_address);
            dns_server.listen(gateway).await?;
            tracing::info!("🛑 DNS server stopped");
            Ok::<(), color_eyre::Report>(())
        }
    });

    let dhcp_server = tokio::spawn(async move {
        tracing::info!(
            "📍 Listening for DHCP requests: {}",
            args.dhcp_listen_address
        );
        dhcp_server.listen(gateway).await?;
        tracing::info!("🛑 DHCP server stopped");
        Ok::<(), color_eyre::Report>(())
    });

    tokio::select! {
        result = api_server => result??,
        result = mqtt_server => result??,
        result = dns_server => result??,
        result = dhcp_server => result??,
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
