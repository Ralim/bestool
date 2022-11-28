use crate::beslink::{
    helper_sync_and_load_programmer, read_flash_data, BESLinkError, BES_PROGRAMMING_BAUDRATE,
};
use serialport::SerialPort;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use tracing::error;
use tracing::info;

pub fn cmd_read_image(input_file: String, serial_port: String, start: u32, length: u32) {
    //First gain sync to the device
    println!(
        "Reading binary data from {} @ {}",
        serial_port, BES_PROGRAMMING_BAUDRATE
    );
    let mut serial_port = serialport::new(serial_port, BES_PROGRAMMING_BAUDRATE);
    serial_port = serial_port.timeout(Duration::from_millis(5000));

    match serial_port.open() {
        Ok(mut port) => {
            info!("Starting loader and checking communications");
            match helper_sync_and_load_programmer(&mut port) {
                Ok(_) => {
                    info!("Done...Bootloader start");
                }
                Err(e) => {
                    error!("Failed {:?}", e);
                }
            }
            info!("Now doing flash read");
            match do_read_flash_data(input_file, &mut port, start, length) {
                Ok(_) => {
                    info!("Done...");
                }
                Err(e) => {
                    error!("Failed {:?}", e);
                }
            }
        }
        Err(e) => println!("Failed to open serial port - {:?}", e),
    }
}
fn do_read_flash_data(
    output_file_path: String,
    serial_port: &mut Box<dyn SerialPort>,
    start: u32,
    length: u32,
) -> Result<(), BESLinkError> {
    //Dump device flash from 0x3C000000 to local file
    let flash_content =
        read_flash_data(serial_port, (0x3C000000 + start) as usize, length as usize)?;

    let mut file = File::create(output_file_path)?;
    // Write a slice of bytes to the file
    file.write_all(flash_content.as_slice())?;

    return Ok(());
}
