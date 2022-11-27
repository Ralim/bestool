use crate::beslink::packet::read_packet_with_trailing_data;
use crate::beslink::{send_packet, BESLinkError, BesMessage, MessageTypes, BES_SYNC};
use serialport::SerialPort;

pub fn read_flash_data(
    serial_port: &mut Box<dyn SerialPort>,
    address: u32,
    length: usize,
) -> Result<Vec<u8>, BESLinkError> {
    let mut cfg_data_1 = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::SysConfig,
        payload: vec![0x05, 0x08],
        checksum: 0xF6,
    };

    cfg_data_1.payload.extend((address as u32).to_le_bytes());
    cfg_data_1.payload.extend((length as u32).to_le_bytes());
    cfg_data_1.set_checksum();

    send_packet(serial_port, cfg_data_1)?;
    //response is 4102 bytes total = 4096 (0x1000)
    let (resp_message, payload) = read_packet_with_trailing_data(serial_port, length)?;
    return Ok(payload);
}
