use crate::beslink::{send_message, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC};
use serialport::SerialPort;
use std::io::Write;
use tracing::error;
use tracing::info;
//Embed the bin file for future
const PROGRAMMER_BINARY: &[u8; 75928] = include_bytes!("../../../programmer.bin");

pub fn load_programmer_runtime_binary_blob(
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<(), BESLinkError> {
    let preload_setup_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::StartProgrammer,
        payload: vec![
            0x00, 0x0C, //??
            0x1C, 0x06, 0x01, 0x20, // Possibly load address in ram?
            0x78, 0x24, 0x01, 0x00, // Suspect length
            0x8A, 0xD7, 0xB9, 0x9E,
        ],
        checksum: 0x4A,
    };
    info!("Start Message {:X?}", preload_setup_message.to_vec());
    send_message(serial_port, preload_setup_message)?;
    let response = sync(serial_port, MessageTypes::StartProgrammer)?;
    if response.payload[0] != 0x00 {
        return Err(BESLinkError::BadResponseCode {
            failed_packet: response.to_vec(),
            got: response.payload[0],
            wanted: 0,
        });
    }
    let programmer_leader = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::ProgrammerRunning,
        payload: vec![
            0xA2, 0x03, 0x00, 0x00, 0x00, 0x48, 0x05, 0x45, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
        checksum: 0x00,
    };
    send_message(serial_port, programmer_leader)?;
    match serial_port.write_all(&PROGRAMMER_BINARY[0x428..PROGRAMMER_BINARY.len() - 4]) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to write the programmer binary {:?}", e);
            return Err(BESLinkError::from(e));
        }
    }
    let response = sync(serial_port, MessageTypes::ProgrammerRunning)?;
    if response.payload != vec![0xA2, 0x01, 0x20] {
        return Err(BESLinkError::BadResponseCode {
            failed_packet: response.to_vec(),
            got: response.payload[2],
            wanted: 0x20,
        });
    }

    Ok(())
}
pub fn start_programmer_runtime_binary_blob(
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<BesMessage, BESLinkError> {
    let preload_setup_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::ProgrammerStart,
        payload: vec![0x01, 0x00],
        checksum: 0xEB,
    };
    send_message(serial_port, preload_setup_message)?;
    info!("Sent start programmer message");
    let resp = sync(serial_port, MessageTypes::ProgrammerInit)?;
    if resp.payload != vec![0x00, 0x06, 0x03, 0x01, 0x00, 0x90, 0x00, 0x00] {
        return Err(BESLinkError::BadResponseCode {
            failed_packet: resp.to_vec(),
            got: resp.payload[0],
            wanted: 0x0,
        });
    }
    Ok(resp)
}
