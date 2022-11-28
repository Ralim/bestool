use crate::beslink::{send_message, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC};
use serialport::SerialPort;
use std::io::Write;
use tracing::error;
use tracing::info;
//Embed the bin file for future
const PROGRAMMER_BINARY: &'static [u8; 78564] = include_bytes!("../../../programmer.bin");

pub fn load_programmer_runtime_binary_blob(
    mut serial_port: &mut Box<dyn SerialPort>,
) -> Result<(), BESLinkError> {
    let preload_setup_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::StartProgrammer,
        payload: vec![
            0x00, 0x0C, //??
            0xDC, 0x05, 0x01, 0x20, // Possibly load address in ram?
            0xDC, 0x32, 0x01, 0x00, // Suspect length
            0xC0, 0xA7, 0xE8, 0x0C,
        ],
        checksum: 0x76,
    };
    let _ = send_message(&mut serial_port, preload_setup_message)?;
    let _ = sync(serial_port, MessageTypes::StartProgrammer)?;
    match serial_port.write_all(PROGRAMMER_BINARY) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to write the programmer binary {:?}", e);
            return Err(BESLinkError::from(e));
        }
    }

    return Ok(());
}
pub fn start_programmer_runtime_binary_blob(
    mut serial_port: &mut Box<dyn SerialPort>,
) -> Result<BesMessage, BESLinkError> {
    let preload_setup_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::ProgrammerStart,
        payload: vec![0x01, 0x00],
        checksum: 0xEB,
    };
    send_message(&mut serial_port, preload_setup_message)?;
    info!("Sent start programmer message");
    return sync(serial_port, MessageTypes::ProgrammerInit);
}
