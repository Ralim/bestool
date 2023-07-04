use crate::beslink::message::read_message_with_trailing_data;
use crate::beslink::{
    send_message, BESLinkError, BesMessage, MessageTypes, BES_SYNC, FLASH_BUFFER_SIZE,
};
use serialport::SerialPort;
use std::time::Duration;
use tracing::{info, warn};

pub fn read_flash_data(
    serial_port: &mut Box<dyn SerialPort>,
    address: usize,
    length: usize,
) -> Result<Vec<u8>, BESLinkError> {
    let mut result = vec![];
    let mut tries = 0;
    while result.len() < length {
        match read_flash_chunk(serial_port, address + result.len()) {
            Ok(chunk) => {
                result.extend_from_slice(&chunk);
                std::thread::sleep(Duration::from_millis(10)); // Try to yield to let watch dog reset
                info!(
                    "Read {} bytes out of {}  ({}%) from flash",
                    result.len(),
                    length,
                    result.len() * 100 / length
                );
            }
            Err(e) => {
                warn!("Error {:?}", e);
                tries += 1;
                if tries > 10 {
                    return Err(e);
                }
            }
        }
    }
    result.resize(length, 0xFF);
    Ok(result)
}

//

fn read_flash_chunk(
    serial_port: &mut Box<dyn SerialPort>,
    address: usize,
) -> Result<Vec<u8>, BESLinkError> {
    let mut cfg_data_1 = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::FlashRead,
        payload: vec![0x05, 0x08], // No idea what these two mean yet
        checksum: 0xF6,
    };
    let chunk_size = FLASH_BUFFER_SIZE / 2;
    cfg_data_1.payload.extend((address as u32).to_le_bytes());
    cfg_data_1.payload.extend((chunk_size as u32).to_le_bytes());
    cfg_data_1.set_checksum();

    send_message(serial_port, cfg_data_1)?;
    //response is 4102 bytes total = 4096 (0x1000)
    let (_, payload) = read_message_with_trailing_data(serial_port, chunk_size)?;
    Ok(payload)
}
