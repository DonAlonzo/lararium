use clap::Parser;
use lararium_beehive::*;
use serialport::{DataBits, SerialPortInfo, SerialPortType, StopBits};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(version)]
struct Args {}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _args = Args::parse();
    tracing_subscriber::fmt().init();

    let ports = serialport::available_ports()?;
    for SerialPortInfo {
        port_name,
        port_type,
    } in ports
    {
        let SerialPortType::UsbPort(port_info) = port_type else {
            continue;
        };
        match (port_info.vid, port_info.pid) {
            (0x1a86, 0x55d4) => (), // SONOFF Zigbee 3.0 USB Dongle Plus V2
            (0x1cf1, 0x0030) => (), // Dresden Elektronik ConBee II
            _ => continue,
        }
        tracing::info!("{}", port_name);
        tracing::info!("  VID: 0x{:04x}", port_info.vid);
        tracing::info!("  PID: 0x{:04x}", port_info.pid);
        tracing::info!("  Serial Number: {:?}", port_info.serial_number);
        tracing::info!("  Manufacturer: {:?}", port_info.manufacturer);
        tracing::info!("  Product: {:?}", port_info.product);
    }

    let port = serialport::new("/dev/ttyACM0", 115_200)
        .stop_bits(StopBits::One)
        .data_bits(DataBits::Eight)
        .timeout(Duration::from_millis(50))
        .open()?;

    let beehive = Arc::new(Mutex::new(Beehive::new(port)));

    beehive.lock().await.reset().await;

    let listen_task = tokio::task::spawn({
        let beehive = beehive.clone();
        async move {
            beehive.lock().await.listen().await;
        }
    });

    tracing::info!("Waiting for device to be ready...");
    beehive.lock().await.wait_until_ready().await;
    tracing::info!("Device is ready");

    beehive.lock().await.send_query_version().await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    beehive.lock().await.send_query_version().await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    beehive.lock().await.send_query_version().await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    beehive.lock().await.send_query_version().await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    beehive.lock().await.send_init_network().await;

    tokio::select! {
        result = listen_task => result?,
        _ = tokio::signal::ctrl_c() => (),
    };

    Ok(())
}
