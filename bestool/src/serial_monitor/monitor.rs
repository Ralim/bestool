use serialport::SerialPort;
use std::cmp::min;
use std::io::{stdout, Read, Write};
use std::thread::sleep;
use std::{error::Error, time::Duration};

pub fn run_serial_monitor(mut port: Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    // Until exit, read from the port and display; and send back anything the user types to the uart
    // Except we catch an exit combo
    const BUFFER_SIZE: usize = 128;
    let mut read_buffer = [0; BUFFER_SIZE];
    loop {
        if port.bytes_to_read()? > 0 {
            match port.read(&mut read_buffer) {
                Ok(bytes_read) => {
                    let mut out = stdout();
                    out.write(&read_buffer[0..min(bytes_read, BUFFER_SIZE)])?;
                }
                Err(e) => println!("Error reading from port {:?}", e),
            }
        } else {
            sleep(Duration::from_millis(50));
        }
    }
}
