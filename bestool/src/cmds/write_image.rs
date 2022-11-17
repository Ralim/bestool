use crate::beslink;
use crate::beslink::{
    burn_image_to_flash, load_programmer_runtime_binary_blob, query_memory_info,
    start_programmer_runtime_binary_blob, BESLinkError, BES_PROGRAMMING_BAUDRATE,
};
use serialport::SerialPort;
use std::fs;
use std::time::Duration;
use tracing::error;
use tracing::info;

pub fn cmd_write_image(input_file: String, serial_port: String) {
    //First gain sync to the device
    println!(
        "Opening serial monitor to {} @ {}",
        serial_port, BES_PROGRAMMING_BAUDRATE
    );
    let mut serial_port = serialport::new(serial_port, BES_PROGRAMMING_BAUDRATE);
    serial_port = serial_port.timeout(Duration::from_millis(5000));

    match serial_port.open() {
        Ok(mut port) => {
            info!("Starting loader and checking communications");
            match run_through_to_flash_info(&mut port) {
                Ok(_) => {
                    info!("Done...");
                }
                Err(e) => {
                    error!("Failed {:?}", e);
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
        Err(e) => println!("Failed to open serial port - {:?}", e),
    }
}
fn do_burn_image_to_flash(
    input_file: String,
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<(), BESLinkError> {
    // Open file, read file, call burn_image_to_flash
    let file_contents = fs::read(input_file)?;

    return burn_image_to_flash(serial_port, file_contents, 0x3C000000);
}
fn run_through_to_flash_info(serial_port: &mut Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    sync_into_bootloader(serial_port)?;
    info!("In bootloader");
    load_programmer_runtime_binary_blob(serial_port)?;
    info!("Loaded programmer blob");
    start_programmer_runtime_binary_blob(serial_port)?;
    info!("Started programmer blob");
    query_memory_info(serial_port)?;
    info!("Got Memory info");
    return Ok(());
}
fn sync_into_bootloader(serial_port: &mut Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    // Gain sync
    info!("Syncing into bootloader");
    let _ = beslink::sync(serial_port, beslink::MessageTypes::Sync)?;
    info!("Saw boot sync, sending ack");
    // Send message to stay in bootloader
    let msg = beslink::BesMessage {
        sync: beslink::BES_SYNC,
        type1: beslink::MessageTypes::Sync,
        payload: vec![0x00, 0x01, 0x01],
        checksum: 0xEF,
    };
    return match beslink::send_packet(serial_port, msg) {
        Ok(_) => Ok(()),
        Err(e) => Err(BESLinkError::from(e)),
    };
}
