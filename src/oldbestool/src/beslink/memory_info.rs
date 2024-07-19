use crate::beslink::{send_message, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC};
use serialport::SerialPort;

pub fn query_memory_info(serial_port: &mut Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    let get_flash_id_cmd = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::FlashCommand,
        payload: vec![0x02, 0x01, 0x11],
        checksum: 0xC8,
    };
    let get_flash_unique_id_cmd = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::FlashCommand,
        payload: vec![0x03, 0x01, 0x12],
        checksum: 0xC6,
    };

    send_message(serial_port, get_flash_id_cmd)?;
    let flash_id = sync(serial_port, MessageTypes::FlashCommand)?;
    send_message(serial_port, get_flash_unique_id_cmd)?;
    let flash_unique_id = sync(serial_port, MessageTypes::FlashCommand)?;
    println!("Flash General ID {:?}", flash_id.payload);
    println!("Flash Unique ID {:?}", flash_unique_id.payload);
    Ok(())
}
