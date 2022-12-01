use crate::beslink::{
    helper_sync_and_load_programmer, read_flash_data, send_device_reboot, BESLinkError,
    BES_PROGRAMMING_BAUDRATE,
};
use serialport::SerialPort;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use tracing::error;
use tracing::info;

pub fn cmd_read_image(input_file: String, serial_port: String, start: usize, length: usize) {
    //First gain sync to the device
    println!(
        "Reading binary data from {} @ {}",
        serial_port, BES_PROGRAMMING_BAUDRATE
    );
    let mut serial_port = serialport::new(serial_port, BES_PROGRAMMING_BAUDRATE);
    serial_port = serial_port.timeout(Duration::from_millis(5000));

    match serial_port.open() {
        Ok(mut port) => match do_read_flash_data(input_file, &mut port, start, length) {
            Ok(_) => {
                info!("Done...");
            }
            Err(e) => {
                error!("Failed {:?}", e);
            }
        },
        Err(e) => println!("Failed to open serial port - {:?}", e),
    }
}
fn do_read_flash_data(
    output_file_path: String,
    serial_port: &mut Box<dyn SerialPort>,
    start: usize,
    length: usize,
) -> Result<(), BESLinkError> {
    let mut flash_content: Vec<u8> = vec![];
    const MAX_READ_BEFORE_RESET: usize = 1024 * 1024; //1MB chunks
    while flash_content.len() < length {
        let chunk_length = {
            if (length - flash_content.len()) < MAX_READ_BEFORE_RESET {
                length - flash_content.len()
            } else {
                MAX_READ_BEFORE_RESET
            }
        };
        let chunk = do_reset_sync_read(
            serial_port,
            0x3C000000 + start + flash_content.len(),
            chunk_length,
        )?;
        flash_content.extend(chunk);
    }

    let mut file = File::create(output_file_path)?;
    // Write a slice of bytes to the file
    file.write_all(flash_content.as_slice())?;

    return Ok(());
}

//The main bootloader wasn't super designed to allow reading the flash;
// but they shipped a debugging memory read that will try and dump memory content out to the uart basically.
// But as this is a "debug" message; it doesnt seem to reset the watchdog
// This means that reads larger than about 2MB will fail randomly when the watchdog trips
// To work around this, we read 1MB chunks with a device reset between the reads

fn do_reset_sync_read(
    serial_port: &mut Box<dyn SerialPort>,
    start: usize,
    length: usize,
) -> Result<Vec<u8>, BESLinkError> {
    info!("Starting loader and checking communications");
    match helper_sync_and_load_programmer(serial_port) {
        Ok(_) => {
            info!("Done...Bootloader start");
        }
        Err(e) => {
            error!("Failed {:?}", e);
        }
    }
    info!("Now doing flash read");
    //Dump device flash from 0x3C000000 to local file
    let flash_content = read_flash_data(serial_port, start, length)?;
    //Send reset
    send_device_reboot(serial_port)?;
    return Ok(flash_content);
}
