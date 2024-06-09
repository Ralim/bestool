use serialport::SerialPort;
use std::cmp::min;
use std::error::Error;
use std::io::{stdout, Read, Write};
use std::time::Duration;

pub fn run_serial_monitor(mut port: Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    // Until exit, read from the port and display; and send back anything the user types to the uart
    // Except we catch an exit combo
    port.set_timeout(Duration::from_millis(1000))
        .expect("Setting port read timeout failed");
    const BUFFER_SIZE: usize = 128;
    let mut read_buffer = [0; BUFFER_SIZE];
    loop {
        match port.read(&mut read_buffer) {
            Ok(bytes_read) => {
                let mut out = stdout();
                let _ = out.write(&read_buffer[0..min(bytes_read, BUFFER_SIZE)])?;
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut => { /*No-op for timeouts */ }
                    std::io::ErrorKind::BrokenPipe => {
                        println!("USB Port disconnected");
                        return Ok(());
                    }
                    _ => {
                        println!("Error reading from port {e:?} / {}", e.kind())
                    }
                }
            }
        }
    }
}
