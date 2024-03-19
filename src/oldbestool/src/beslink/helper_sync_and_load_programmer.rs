use crate::beslink::{
    load_programmer_runtime_binary_blob, query_memory_info, send_message,
    start_programmer_runtime_binary_blob, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC,
};
use serialport::SerialPort;
use std::time::Duration;
use tracing::{info, warn};

pub fn helper_sync_and_load_programmer(
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<(), BESLinkError> {
    sync_with_bootloader(serial_port)?;
    info!("In bootloader");
    load_programmer_runtime_binary_blob(serial_port)?;
    info!("Loaded programmer blob");
    start_programmer_runtime_binary_blob(serial_port)?;
    info!("Started programmer blob");
    query_memory_info(serial_port)?;
    info!("Got Memory info Done; so programmer blob is working");
    Ok(())
}
fn get_stay_in_programmer_message() -> BesMessage {
    BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::Sync,
        payload: vec![0x00, 0x01, 0x01],
        checksum: 0xEF,
    }
}
fn sync_with_bootloader(serial_port: &mut Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    // Gain sync
    info!("Syncing into bootloader");

    match send_message(serial_port, get_stay_in_programmer_message()) {
        Ok(_) => {}
        Err(e) => return Err(BESLinkError::from(e)),
    };
    let sync_message = sync(serial_port, MessageTypes::Sync)?;
    info!("Received sync advertisement {:X?}", sync_message.to_vec());
    loop {
        std::thread::sleep(Duration::from_millis(2));

        info!("Saw boot sync, sending ack");
        // Send message to stay in bootloader

        match send_message(serial_port, get_stay_in_programmer_message()) {
            Ok(_) => {}
            Err(e) => return Err(BESLinkError::from(e)),
        };
        info!("Sent sync message");
        let response = sync(serial_port, MessageTypes::Sync)?;
        info!("Sync response: {:X?}", response.to_vec());
        if response.payload[2] == 0x02 && response.payload[3] == 0x00 {
            return Ok(());
        } else {
            warn!("Received bad sync response {:X?}", response.to_vec());
        }
    }
}
