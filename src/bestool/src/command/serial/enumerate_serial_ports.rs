use serialport::SerialPortType;
use std::io::Result;

fn get_port_type(port: SerialPortType) -> Option<String> {
    match port {
        #[cfg(windows)]
        SerialPortType::UsbPort(info) => Some(format!("USB {:04X}:{:04X}", info.vid, info.pid)),
        #[cfg(unix)]
        SerialPortType::UsbPort(info) => Some(format!("USB {:04x}:{:04x}", info.vid, info.pid)),
        SerialPortType::PciPort => Some(String::from("PCI")),
        SerialPortType::BluetoothPort => Some(String::from("Bluetooth")),
        SerialPortType::Unknown => None,
    }
}

pub fn run() -> Result<()> {
    match serialport::available_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                eprintln!("No serial ports found");
                return Ok(());
            }
            println!("Detected serial ports and their type:");
            for port in ports {
                if let Some(port_type) = get_port_type(port.port_type) {
                    println!("{}\t[{}]", port.port_name, port_type);
                } else {
                    continue;
                }
            }
        }
        Err(e) => eprintln!("Could not list ports due to {e:?}"),
    }

    Ok(())
}
