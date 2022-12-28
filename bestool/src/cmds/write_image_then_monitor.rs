use crate::beslink::{
    burn_image_to_flash, helper_sync_and_load_programmer, send_device_reboot, BESLinkError,
    BES_PROGRAMMING_BAUDRATE,
};
use crate::serial_monitor::run_serial_monitor;
use serialport::{ClearBuffer, SerialPort};
use std::error::Error;
use std::fs;
use std::io::Write;
use std::time::Duration;
use tracing::error;
use tracing::info;

pub fn cmd_write_image_then_monitor(
    input_file: String,
    serial_port: String,
    monitor_baud_rate: u32,
) {
    //First gain sync to the device
    println!(
        "Writing binary data to {} @ {}; then monitoring at {}",
        serial_port, BES_PROGRAMMING_BAUDRATE, monitor_baud_rate
    );
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
                    return;
                }
            }
            info!("Starting monitoring");
            match port.set_baud_rate(monitor_baud_rate) {
                Ok(_) => {
                    info!("Done...");
                }
                Err(e) => {
                    error!("Failed {:?}", e);
                    return;
                }
            }
            match run_serial_monitor(port) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed monitoring: {:?}", e);
                    return;
                }
            }
        }
        Err(e) => println!("Failed to open serial port - {:?}", e),
    }
}
fn do_burn_image_to_flash(
    input_file: String,
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<(), BESLinkError> {
    // Open file, read file, call burn_image_to_flash
    let file_contents = fs::read(input_file)?;

    burn_image_to_flash(serial_port, file_contents, 0x3C000000)?;
    //Send reset
    send_device_reboot(serial_port)?;
    Ok(())
}
