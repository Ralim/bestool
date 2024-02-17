use crate::beslink::{
    burn_image_to_flash, helper_sync_and_load_programmer, send_device_reboot, BESLinkError,
    BES_PROGRAMMING_BAUDRATE,
};
use serialport::{ClearBuffer, SerialPort};
use std::fs;

use std::time::Duration;
use tracing::error;
use tracing::info;

pub fn cmd_write_image(input_file: String, serial_port: String) {
    //First gain sync to the device
    println!("Writing binary data to {serial_port} @ {BES_PROGRAMMING_BAUDRATE}");
    let mut serial_port = serialport::new(serial_port, BES_PROGRAMMING_BAUDRATE);
    serial_port = serial_port.timeout(Duration::from_millis(5000));

    match serial_port.open() {
        Ok(mut port) => {
            let _ = port.clear(ClearBuffer::All);
            info!("Starting loader and checking communications");
            match helper_sync_and_load_programmer(&mut port) {
                Ok(_) => {
                    info!("Done...");
                }
                Err(e) => {
                    error!("Failed {:?}", e);
                    return;
                }
            }
            info!("Now doing firmware load");
            match do_burn_image_to_flash(input_file, &mut port) {
                Ok(_) => {
                    info!("Done...");
                }
                Err(e) => {
                    error!("Failed {:?}", e);
                }
            }
        }
        Err(e) => println!("Failed to open serial port - {e:?}"),
    }
}
fn do_burn_image_to_flash(
    input_file: String,
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<(), BESLinkError> {
    // Open file, read file, call burn_image_to_flash
    let file_contents = fs::read(input_file)?;
    burn_image_to_flash(serial_port, file_contents, 0x3C00_0000)?;
    //Send reset
    send_device_reboot(serial_port)?;
    Ok(())
}
