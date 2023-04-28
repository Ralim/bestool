use crate::serial_monitor::run_serial_monitor;

pub fn cmd_serial_port_monitor(port_name: String, baud_rate: u32) {
    // Span a basic serial port monitor attached to the serial port
    // Eventually we will hook in extra utility commands
    println!("Opening serial monitor to {port_name} @ {baud_rate}");
    let serial_port = serialport::new(port_name, baud_rate);
    match serial_port.open() {
        Ok(port) => {
            let _ = run_serial_monitor(port);
        }
        Err(e) => println!("Failed to open serial port - {e:?}"),
    }
}
