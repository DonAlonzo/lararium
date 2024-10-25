use clap::Parser;
use lararium_beehive::prelude::*;
use serialport::{DataBits, SerialPortInfo, SerialPortType, StopBits};
use std::time::Duration;

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
        info!("{}", port_name);
        info!("  VID: 0x{:04x}", port_info.vid);
        info!("  PID: 0x{:04x}", port_info.pid);
        info!("  Serial Number: {:?}", port_info.serial_number);
        info!("  Manufacturer: {:?}", port_info.manufacturer);
        info!("  Product: {:?}", port_info.product);

        let port = serialport::new(port_name, 115_200)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight)
            .timeout(Duration::from_millis(50))
            .open()?;

        let mut beehive = Beehive::new(port);

        let listen_task = tokio::task::spawn({
            let mut beehive = beehive.clone();
            async move {
                beehive.listen().await;
            }
        });

        beehive.startup().await;

        tokio::select! {
            result = listen_task => result?,
            _ = tokio::signal::ctrl_c() => (),
        };
    }

    Ok(())
}
