use crate::beslink::{send_packet, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC};
use serialport::{ClearBuffer, SerialPort};
use std::time::Duration;

pub fn send_cfg_data(serial_port: &mut Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    // Have not reverse engineered these two yet
    let cfg_data_1 = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::SysConfig,
        payload: vec![0x05, 0x08, 0x00, 0xE0, 0x0F, 0x3C, 0x00, 0x10, 0x00, 0x00],
        checksum: 0xF6,
    };

    send_packet(serial_port, cfg_data_1)?;
    std::thread::sleep(Duration::from_millis(100));
    let cfg_data_2 = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::SysConfig,
        payload: vec![0x06, 0x08, 0x00, 0xF0, 0x0F, 0x3C, 0x00, 0x10, 0x00, 0x00],
        checksum: 0xE5,
    };

    send_packet(serial_port, cfg_data_2)?;
    sync(serial_port, MessageTypes::SysConfig)?;
    std::thread::sleep(Duration::from_millis(200));
    serial_port.clear(ClearBuffer::Input)?;
    return Ok(());
}
