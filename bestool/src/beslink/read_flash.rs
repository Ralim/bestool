use crate::beslink::packet::read_packet_with_trailing_data;
use crate::beslink::{
    send_packet, BESLinkError, BesMessage, MessageTypes, BES_SYNC, FLASH_BUFFER_SIZE,
};
use serialport::SerialPort;
use tracing::info;

pub fn read_flash_data(
    serial_port: &mut Box<dyn SerialPort>,
    address: usize,
    length: usize,
) -> Result<Vec<u8>, BESLinkError> {
    let mut result = vec![];
    while result.len() < length {
        let chunk = read_flash_chunk(serial_port, address + result.len())?;
        result.extend_from_slice(&chunk);
        info!(
            "Read {} bytes out of {}  ({}%) from flash",
            result.len(),
            length,
            result.len() * 100 / length
        );
    }
    result.resize(length, 0xFF);
    return Ok(result);
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

    cfg_data_1.payload.extend((address as u32).to_le_bytes());
    cfg_data_1
        .payload
        .extend((FLASH_BUFFER_SIZE as u32).to_le_bytes());
    cfg_data_1.set_checksum();

    send_packet(serial_port, cfg_data_1)?;
    //response is 4102 bytes total = 4096 (0x1000)
    let (_, payload) = read_packet_with_trailing_data(serial_port, FLASH_BUFFER_SIZE as usize)?;
    return Ok(payload);
}
