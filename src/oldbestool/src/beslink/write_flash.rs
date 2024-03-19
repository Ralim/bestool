use crate::beslink::{
    send_message, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC, FLASH_BUFFER_SIZE,
};
use crc::{Crc, CRC_32_ISO_HDLC};
use serialport::SerialPort;
use std::io::Write;
use std::time::Duration;
use tracing::error;
use tracing::info;
const MAX_UNACKED_PACKETS: usize = 2;

pub fn burn_image_to_flash(
    serial_port: &mut Box<dyn SerialPort>,
    payload_in: Vec<u8>,
    address: usize,
) -> Result<(), BESLinkError> {
    let mut payload = payload_in;
    //Pad image to FLASH_BUFFER_SIZE
    while payload.len() % FLASH_BUFFER_SIZE != 0 {
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
    let file_chunks = payload.chunks(FLASH_BUFFER_SIZE);
    let file_chunk_count = file_chunks.len();
    for chunk in file_chunks {
        loop {
            if outstanding_chunks < MAX_UNACKED_PACKETS {
                info!(
                    "Sending flash chunk {} out of {}",
                    chunk_num, file_chunk_count
                );
                send_flash_chunk_msg(serial_port, chunk.to_vec(), chunk_num)?;
                if chunk_num == 0x00 {
                    std::thread::sleep(Duration::from_millis(411));
                }
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
    send_flash_commit_message(serial_port, address)
}
fn send_flash_commit_message(
    serial_port: &mut Box<dyn SerialPort>,
    address: usize,
) -> Result<(), BESLinkError> {
    let mut burn_prepare_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::FlashCommand,
        payload: vec![0x06, 0x09, 0x22],
        checksum: 0xEB,
    };
    burn_prepare_message
        .payload
        .extend((address as u32).to_le_bytes());

    burn_prepare_message
        .payload
        .extend(vec![0x1C, 0xEC, 0x57, 0xBE]);
    burn_prepare_message.set_checksum();
    send_message(serial_port, burn_prepare_message)?;
    info!("Sent flash finalise message");
    let resp = sync(serial_port, MessageTypes::FlashCommand)?;
    if resp.payload != vec![6, 1, 0] {
        return Err(BESLinkError::BadResponseCode {
            failed_packet: resp.to_vec(),
            got: resp.payload[0],
            wanted: 0x06,
        });
    }
    Ok(())
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
        .extend((FLASH_BUFFER_SIZE as u16).to_le_bytes());
    data_message.payload.extend(vec![0x00, 0x00]);

    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut digest = crc.digest();
    digest.update(&payload);
    let crc_value = digest.finalize();
    data_message.payload.extend(crc_value.to_le_bytes());
    data_message.payload.extend(vec![chunk as u8, 0x00, 0x00]);
    data_message.set_checksum();
    data_message
}

fn send_flash_chunk_msg(
    serial_port: &mut Box<dyn SerialPort>,
    payload: Vec<u8>,
    chunk: usize,
) -> Result<(), BESLinkError> {
    if payload.len() != FLASH_BUFFER_SIZE {
        return Err(BESLinkError::InvalidArgs {});
    }
    let data_message = get_flash_chunk_msg(payload.clone(), chunk);
    let mut message_vec = data_message.to_vec();
    message_vec.extend(payload);

    return match serial_port.write_all(message_vec.as_slice()) {
        Ok(_) => {
            info!("Wrote flash buffer of len 0x{:X} ", message_vec.len());
            std::thread::sleep(Duration::from_millis(10)); // This is just a small rate limiter
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
        payload: vec![0x05, 0x0C],
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
        "Sent erase start message, {:X?}",
        burn_prepare_message.to_vec()
    );
    send_message(serial_port, burn_prepare_message)?;
    let resp = sync(serial_port, MessageTypes::EraseBurnStart)?;
    if resp.payload != vec![0x05, 0x01, 0x00] {
        return Err(BESLinkError::BadResponseCode {
            failed_packet: resp.to_vec(),
            got: resp.payload[0],
            wanted: 0x05,
        });
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use crate::beslink::write_flash::get_flash_chunk_msg;

    //Embed the bin file for future
    const CHUNK1_TEST: &[u8; 32768] = include_bytes!("../../../chunk1.bin");
    const CHUNK2_TEST: &[u8; 32768] = include_bytes!("../../../chunk2.bin");

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
