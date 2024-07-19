use std::time::Duration;

use serialport::SerialPort;
use tracing::info;

pub fn open_serial_port_with_wait(
    port_path: &str,
    baud_rate: u32,
    wait_for_port: bool,
) -> Box<dyn SerialPort> {
    // If wait for port is true, we handle it not being openable by retrying while waiting for it
    info!("Opening {port_path} @ {baud_rate}");
    loop {
        let serial_port = serialport::new(port_path, baud_rate);
        match serial_port.open() {
            Ok(port) => return port,
            Err(_) => {
                //Port didnt open
                if !wait_for_port {
                    panic!("Unable to open requested Serial Port");
                }
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}
