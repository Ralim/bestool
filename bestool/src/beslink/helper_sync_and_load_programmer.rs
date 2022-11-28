use crate::beslink::{
    load_programmer_runtime_binary_blob, query_memory_info, send_message,
    start_programmer_runtime_binary_blob, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC,
};
use serialport::SerialPort;
use tracing::info;

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
    return Ok(());
}
fn sync_with_bootloader(serial_port: &mut Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    // Gain sync
    info!("Syncing into bootloader");
    let _ = sync(serial_port, MessageTypes::Sync)?;
    info!("Saw boot sync, sending ack");
    // Send message to stay in bootloader
    let msg = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::Sync,
        payload: vec![0x00, 0x01, 0x01],
        checksum: 0xEF,
    };
    return match send_message(serial_port, msg) {
        Ok(_) => Ok(()),
        Err(e) => Err(BESLinkError::from(e)),
    };
}
