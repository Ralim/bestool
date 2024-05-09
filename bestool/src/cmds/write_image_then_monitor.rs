use crate::beslink::{
    burn_image_to_flash, helper_sync_and_load_programmer, send_device_reboot, BESLinkError,
    BES_PROGRAMMING_BAUDRATE,
};
use crate::serial_monitor::run_serial_monitor;
use crate::serial_port_opener::open_serial_port_with_wait;
use serialport::{ClearBuffer, SerialPort};

use std::fs;

use std::path::PathBuf;
use std::time::Duration;
use tracing::error;
use tracing::info;

pub fn cmd_write_image_then_monitor(
    input_file_path: &PathBuf,
    serial_port: &str,
    monitor_baud_rate: u32,
    wait_for_port: bool,
) {
    //First gain sync to the device
    println!(
        "Writing binary data to {serial_port} @ {BES_PROGRAMMING_BAUDRATE}; then monitoring at {monitor_baud_rate}"
    );
    let mut port = open_serial_port_with_wait(serial_port, BES_PROGRAMMING_BAUDRATE, wait_for_port);
    port.set_timeout(Duration::from_millis(5000))
        .expect("Cant set port timeout");

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
    match do_burn_image_to_flash(input_file_path, &mut port) {
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
        }
    }
}
fn do_burn_image_to_flash(
    input_file: &PathBuf,
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<(), BESLinkError> {
    // Open file, read file, call burn_image_to_flash
    let file_contents = fs::read(input_file)?;

    burn_image_to_flash(serial_port, file_contents, 0x3C00_0000)?;
    //Send reset
    send_device_reboot(serial_port)?;
    Ok(())
}
