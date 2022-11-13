pub fn cmd_serial_port_monitor(port_name:String,baud_rate:u32) {
    // Span a basic serial port monitor attached to the serial port
    // Eventually we will hook in extra utility commands
    println!("Opening serial monitor to {} @ {}", port_name,baud_rate);
}
