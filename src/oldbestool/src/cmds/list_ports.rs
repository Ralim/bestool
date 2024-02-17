use serialport::SerialPortType;

pub fn cmd_list_serial_ports() {
    println!("Detected serial ports and their type:");
    fn port_type_name(t: SerialPortType) -> String {
        match t {
            SerialPortType::UsbPort(info) => format!("USB 0x{:04X}:0x{:04X}", info.vid, info.pid),
            SerialPortType::PciPort => "PCI".to_owned(),
            SerialPortType::BluetoothPort => "Bluetooth".to_owned(),
            SerialPortType::Unknown => "Unknown".to_owned(),
        }
    }
    match serialport::available_ports() {
        Ok(ports) => {
            for port in ports {
                println!("{}\t[{}]", port.port_name, port_type_name(port.port_type))
            }
        }
        Err(e) => println!("Could not list ports due to {e:?}"),
    }
}
