use crate::{serial_monitor::run_serial_monitor, serial_port_opener::open_serial_port_with_wait};

pub fn cmd_serial_port_monitor(port_name: &str, baud_rate: u32, wait_for_port: bool) {
    // Span a basic serial port monitor attached to the serial port
    // Eventually we will hook in extra utility commands
    let port = open_serial_port_with_wait(port_name, baud_rate, wait_for_port);

    run_serial_monitor(port).unwrap();
}
