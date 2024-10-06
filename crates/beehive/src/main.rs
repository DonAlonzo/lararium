use serialport::{DataBits, SerialPortInfo, SerialPortType, StopBits};
use std::io::{self, Write};
use std::time::Duration;

fn main() {
    let ports = serialport::available_ports().expect("No ports found!");
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
        println!("{}", port_name);
        println!("  VID: 0x{:04x}", port_info.vid);
        println!("  PID: 0x{:04x}", port_info.pid);
        println!("  Serial Number: {:?}", port_info.serial_number);
        println!("  Manufacturer: {:?}", port_info.manufacturer);
        println!("  Product: {:?}", port_info.product);
    }

    let mut port = serialport::new("/dev/ttyACM0", 115_200)
        .stop_bits(StopBits::One)
        .data_bits(DataBits::Eight)
        .timeout(Duration::from_millis(50))
        .open()
        .expect("Failed to open port");

    let rst: [u8; 5] = [0x1a, 0xc0, 0x38, 0xbc, 0x7e];
    port.write_all(&rst)
        .expect("Failed to send network discovery");
    port.flush().expect("Failed to flush port");

    //let rst: [u8; 4] = [0x00, 0x00, 0x00, 0x02];
    //port.write_all(&rst).expect("Failed to send network discovery");
    //port.flush().expect("Failed to flush port");

    //a0 54 7d 3a 7e

    let mut buffer = vec![0; 1000];
    loop {
        match port.read(buffer.as_mut_slice()) {
            Ok(length) => {
                for byte in buffer.iter().take(length) {
                    print!("{:02x} ", byte);
                }
                println!("");
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => panic!("broken pipe"),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
