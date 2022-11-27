use std::io::Write;
use std::time::Duration;
use crate::beslink::{send_packet, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC};
use crc::{Crc, CRC_32_ISO_HDLC};
use serialport::SerialPort;
use tracing::error;
use tracing::info;
const FLASH_WRITE_SIZE: usize = 0x8000;
const MAX_UNACKED_PACKETS: usize = 2;

pub fn burn_image_to_flash(
    serial_port: &mut Box<dyn SerialPort>,
    payload_in: Vec<u8>,
    address: usize,
) -> Result<(), BESLinkError> {
    let mut payload = payload_in.clone();
    //Pad image to FLASH_WRITE_SIZE
    while payload.len() % FLASH_WRITE_SIZE != 0 {
        payload.push(0xFF);
    }
    let file_length = payload.len();
    match send_flash_erase(serial_port, file_length, address) {
        Ok(m) => {
            info!("Flash Erase confirmed, {:?}", m)
        }
        Err(e) => {
            error!("Flash erase message failed {:?}", e);
            return Err(e);
        }
    }

    //Now loop, send a flash chunk and handle an ack
    let mut chunk_num = 0;
    let mut outstanding_chunks = 0;
    let file_chunks = payload.chunks(FLASH_WRITE_SIZE);
    for chunk in file_chunks {
        loop {
            if outstanding_chunks < MAX_UNACKED_PACKETS {
                info!("Sending flash chunk {}", chunk_num);
                send_flash_chunk_msg(serial_port, chunk.to_vec(), chunk_num)?;
                chunk_num += 1;
                outstanding_chunks += 1;
                break; // Step to next chunk
            }
            //Wait for an ack
            match sync(serial_port, MessageTypes::FlashBurnData) {
                Ok(m) => {
                    outstanding_chunks -= 1;
                    info!("Confirmation for message {}", m.payload[3]);
                }
                Err(e) => {
                    error!("Waiting for flash confirmation, {:?}", e);
                }
            }
        }
    }
    //Wait for rest of chunk confirmations
    while outstanding_chunks > 0 {
        match sync(serial_port, MessageTypes::FlashBurnData) {
            Ok(m) => {
                outstanding_chunks -= 1;
                info!("Confirmation for message {}", m.payload[3]);
            }
            Err(e) => {
                error!("Waiting for flash confirmation, {:?}", e);
            }
        }
    }
    info!("Sending flash finalise");
    return send_flash_commit_message(serial_port, address);
}
fn send_flash_commit_message(
    serial_port: &mut Box<dyn SerialPort>,
    address: usize,
) -> Result<(), BESLinkError> {
    let mut burn_prepare_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::FlashCommand,
        payload: vec![0x08, 0x09, 0x22],
        checksum: 0xEB,
    };
    burn_prepare_message
        .payload
        .extend((address as u32).to_le_bytes());

    burn_prepare_message
        .payload
        .extend(vec![0x1C, 0xEC, 0x57, 0xBE]);
    burn_prepare_message.set_checksum();
    send_packet(serial_port, burn_prepare_message)?;
    info!("Sent flash finalise message");
    sync(serial_port, MessageTypes::FlashCommand)?;
    return Ok(());
}
fn get_flash_chunk_msg(payload: Vec<u8>, chunk: usize) -> BesMessage {
    let mut data_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::FlashBurnData,
        payload: vec![0xC1 + (chunk as u8), 0x0B],
        checksum: 0xEB,
    };
    data_message
        .payload
        .extend((FLASH_WRITE_SIZE as u16).to_le_bytes());
    data_message.payload.extend(vec![0x00, 0x00]);

    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut digest = crc.digest();
    digest.update(&payload);
    let crc_value = digest.finalize();
    data_message
        .payload
        .extend((crc_value as u32).to_le_bytes());
    data_message.payload.extend(vec![chunk as u8, 0x00, 0x00]);
    data_message.set_checksum();
    return data_message;
}

fn send_flash_chunk_msg(
    serial_port: &mut Box<dyn SerialPort>,
    payload: Vec<u8>,
    chunk: usize,
) -> Result<(), BESLinkError> {
    if payload.len() != FLASH_WRITE_SIZE {
        return Err(BESLinkError::InvalidArgs {});
    }
    let data_message = get_flash_chunk_msg(payload.clone(), chunk);
    info!("Flash message {:x?}", data_message.to_vec());
    let mut message_vec = data_message.to_vec();
    message_vec.extend(payload);

    return match serial_port.write_all(message_vec.as_slice()) {
        Ok(_) => {
            info!("Wrote flash buffer of len {} ", message_vec.len());
            let _ =serial_port.flush();
            std::thread::sleep(Duration::from_millis(10));
            Ok(())
        }
        Err(e) => {
            error!("Writing to flash buffer to port raised {:?}", e);
            Err(BESLinkError::from(e))
        }
    };
}

fn send_flash_erase(
    serial_port: &mut Box<dyn SerialPort>,
    payload_len: usize,
    address: usize,
) -> Result<BesMessage, BESLinkError> {
    let mut burn_prepare_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::EraseBurnStart,
        payload: vec![0x07, 0x0C],
        checksum: 0xEB,
    };
    burn_prepare_message
        .payload
        .extend((address as u32).to_le_bytes());
    burn_prepare_message
        .payload
        .extend((payload_len as u32).to_le_bytes());
    burn_prepare_message
        .payload
        .extend(vec![0x00, 0x80, 0x00, 0x00]);
    burn_prepare_message.set_checksum();
    info!(
        "Sent erase start message, {:?}",
        burn_prepare_message.to_vec()
    );
    send_packet(serial_port, burn_prepare_message)?;
    return sync(serial_port, MessageTypes::EraseBurnStart);
}

#[cfg(test)]
mod tests {
    use crate::beslink::write_flash::get_flash_chunk_msg;

    //Embed the bin file for future
    const CHUNK1_TEST: &'static [u8; 32768] = include_bytes!("../../../chunk1.bin");
    const CHUNK2_TEST: &'static [u8; 32768] = include_bytes!("../../../chunk2.bin");

    #[test]
    fn test_get_flash_chunk_msg() {
        //make fake port it can write to
        let expected_header_data: Vec<u8> = vec![
            0xBE, 0x62, 0xC1, 0x0B, 0x00, 0x80, 0x00, 0x00, 0xAB, 0x77, 0x7F, 0xF4, 0x00, 0x00,
            0x00, 0xFE,
        ];
        let message = get_flash_chunk_msg(CHUNK1_TEST.to_vec(), 0);
        let message_flat = message.to_vec();
        assert_eq!(message_flat, expected_header_data);
    }
    #[test]
    fn test_get_flash_chunk_msg2() {
        //make fake port it can write to
        let expected_header_data: Vec<u8> = vec![
            0xBE, 0x62, 0xC2, 0x0B, 0x00, 0x80, 0x00, 0x00, 0x34, 0x90, 0x61, 0xF9, 0x01, 0x00,
            0x00, 0x73,
        ];
        let message = get_flash_chunk_msg(CHUNK2_TEST.to_vec(), 1);
        let message_flat = message.to_vec();
        assert_eq!(message_flat, expected_header_data);
    }
}
