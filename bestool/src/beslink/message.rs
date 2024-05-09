use crate::beslink::{BESLinkError, BES_SYNC, FLASH_BUFFER_SIZE};
use serialport::SerialPort;
use std::convert::TryFrom;
use std::io::ErrorKind::TimedOut;
use std::io::{Read, Write};

use tracing::{debug, error, info, warn};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MessageTypes {
    DeviceCommand = 0x00, // General commands to the device
    FlashRead = 0x03,     // Debugging message that lets you dump from address space
    Sync = 0x50,          // Seems to be used at boot for locking with ROM
    StartProgrammer = 0x53,
    ProgrammerRunning = 0x54,
    ProgrammerStart = 0x55,
    ProgrammerInit = 0x60,
    EraseBurnStart = 0x61,
    FlashBurnData = 0x62,
    FlashCommand = 0x65, // Suspect used to push extra commands to flash controller/chip/die
    UnknownORInfo = 0x66, // Unknown at this point in time, but references "OR Info"; suspect NOR flash info
}
impl TryFrom<u8> for MessageTypes {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == MessageTypes::Sync as u8 => Ok(MessageTypes::Sync),
            x if x == MessageTypes::StartProgrammer as u8 => Ok(MessageTypes::StartProgrammer),
            x if x == MessageTypes::ProgrammerRunning as u8 => Ok(MessageTypes::ProgrammerRunning),
            x if x == MessageTypes::ProgrammerInit as u8 => Ok(MessageTypes::ProgrammerInit),
            x if x == MessageTypes::ProgrammerStart as u8 => Ok(MessageTypes::ProgrammerStart),
            x if x == MessageTypes::FlashCommand as u8 => Ok(MessageTypes::FlashCommand),
            x if x == MessageTypes::EraseBurnStart as u8 => Ok(MessageTypes::EraseBurnStart),
            x if x == MessageTypes::FlashBurnData as u8 => Ok(MessageTypes::FlashBurnData),
            x if x == MessageTypes::FlashRead as u8 => Ok(MessageTypes::FlashRead),
            x if x == MessageTypes::UnknownORInfo as u8 => Ok(MessageTypes::UnknownORInfo),
            x if x == MessageTypes::DeviceCommand as u8 => Ok(MessageTypes::DeviceCommand),
            _ => Err(()),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct BesMessage {
    pub sync: u8,
    pub type1: MessageTypes,
    pub payload: Vec<u8>,
    pub checksum: u8,
}

impl BesMessage {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];
        result.push(self.sync);
        result.push(self.type1 as u8);
        result.append(&mut self.payload.clone());
        result.push(self.checksum);
        result
    }
    pub fn set_checksum(&mut self) {
        let mut v = self.to_vec();
        v.pop();
        self.checksum = calculate_message_checksum(&v);
    }
}

impl From<Vec<u8>> for BesMessage {
    fn from(d: Vec<u8>) -> Self {
        let mut msg = BesMessage {
            sync: d[0],
            type1: MessageTypes::Sync,
            payload: vec![],
            checksum: d[d.len() - 1],
        };

        match d[1].try_into() {
            Ok(type1) => msg.type1 = type1,
            Err(_) => {
                println!("Unknown packet type 0x{:02X}", d[1]);
            }
        };

        msg.payload = d[2..d.len() - 1].to_vec();

        msg
    }
}

pub fn send_message(serial_port: &mut Box<dyn SerialPort>, msg: BesMessage) -> std::io::Result<()> {
    let packet = msg.to_vec();
    return match serial_port.write_all(packet.as_slice()) {
        Ok(_) => {
            debug!("Wrote {} bytes", packet.len());
            info!("Sent message type {:?} {:X?}", msg.type1, msg.to_vec());
            let _ = serial_port.flush();
            Ok(())
        }
        Err(e) => {
            error!("Writing to port raised {:?}", e);
            Err(e)
        }
    };
}
pub fn read_message_with_trailing_data(
    serial_port: &mut Box<dyn SerialPort>,
    expected_data_len: usize,
) -> Result<(BesMessage, Vec<u8>), BESLinkError> {
    //First read the packet; then read the expected_raw_bytes from the uart
    //TODO for now assuming the 0x03 code for response

    let response = read_message(serial_port)?;
    if response.type1 != MessageTypes::FlashRead {
        error!("Bad packet type: {:?}", response.type1);
        return Err(BESLinkError::InvalidArgs);
    }
    let mut packet: Vec<u8> = vec![];
    let mut buffer: [u8; FLASH_BUFFER_SIZE] = [0; FLASH_BUFFER_SIZE];

    while packet.len() < expected_data_len {
        match serial_port.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    packet.extend(&buffer[0..n]);
                } else {
                    warn!("Stalled packet");
                }
            }
            Err(e) => {
                if e.kind() != TimedOut {
                    println!("Error reading packet header {e:?}");
                    return Err(BESLinkError::from(e));
                }
            }
        }
    }
    Ok((response, packet))
}
pub fn read_message(serial_port: &mut Box<dyn SerialPort>) -> Result<BesMessage, BESLinkError> {
    //
    let mut packet: Vec<u8> = vec![];
    let mut packet_len: usize = 4; //Start expectations at the minimum
    let mut buffer: [u8; 1] = [0; 1];

    while packet.len() < packet_len {
        match serial_port.read(&mut buffer) {
            Ok(n) => {
                if n == 1 {
                    // Only grab if actual data
                    if !(packet.is_empty() && buffer[0] != BES_SYNC) {
                        packet.push(buffer[0]);
                    }
                }
            }
            Err(e) => {
                if e.kind() != TimedOut {
                    println!("Error reading packet header {e:?}");
                    return Err(BESLinkError::from(e));
                }
            }
        }
        if packet.len() == 4 && packet_len == 4 {
            packet_len = (0x05 + packet[3]) as usize;
        }
        //TODO timeout
    }
    match validate_packet_checksum(&packet) {
        Ok(_) => Ok(BesMessage::from(packet)),
        Err(e) => Err(e),
    }
}
pub fn validate_packet_checksum(packet: &[u8]) -> Result<(), BESLinkError> {
    let checksum = calculate_message_checksum(&packet[0..packet.len() - 1]);
    if checksum == packet[packet.len() - 1] {
        return Ok(());
    }
    let e = BESLinkError::BadChecksumError {
        failed_packet: packet.to_vec(),
        got: packet[packet.len() - 1],
        wanted: checksum,
    };
    warn!("Bad Checksum!! {:?}", e);
    Err(e)
}
pub fn calculate_message_checksum(packet: &[u8]) -> u8 {
    let mut sum: u32 = 0;
    for b in packet {
        sum += u32::from(*b);
        sum &= 0xFF;
    }
    (0xFF - sum) as u8
}

#[cfg(test)]
mod tests {
    use crate::beslink::message::calculate_message_checksum;
    use crate::beslink::message::validate_packet_checksum;
    use crate::beslink::{BesMessage, MessageTypes, BES_SYNC};

    #[test]
    fn test_from() {
        let expected_vec: Vec<u8> = vec![BES_SYNC, 0x55, 0x3A, 0x00, 0xB2];
        let msg = BesMessage::from(expected_vec.clone());
        assert_eq!(expected_vec, msg.to_vec())
    }
    #[test]
    fn test_to_vec() {
        let test = BesMessage {
            sync: BES_SYNC,
            type1: MessageTypes::ProgrammerStart,
            payload: vec![0x3A, 0x00],
            checksum: 0xB2,
        };
        let expected_vec: Vec<u8> = vec![BES_SYNC, 0x55, 0x3A, 0x00, 0xB2];
        assert_eq!(expected_vec, test.to_vec())
    }
    #[test]
    fn test_calculate_packet_checksum() {
        //make fake port it can write to
        let test_messages: Vec<Vec<u8>> = vec![
            vec![0xBE, 0x50, 0x00, 0x03, 0x00, 0x00, 0x01, 0xED],
            vec![0xBE, 0x50, 0x00, 0x01, 0x01, 0xEF],
            vec![0xBE, 0x53, 0x00, 0x01, 0x00, 0xED],
            vec![0xBE, 0x65, 0x02, 0x01, 0x11, 0xC8],
            vec![0xBE, 0x65, 0x03, 0x01, 0x12, 0xC6],
            vec![
                0xBE, 0x62, 0xC1, 0x0B, 0x00, 0x80, 0x00, 0x00, 0xAB, 0x77, 0x7F, 0xF4, 0x00, 0x00,
                0x00, 0xFE,
            ],
            vec![
                0xBE, 0x62, 0xC2, 0x0B, 0x00, 0x80, 0x00, 0x00, 0x34, 0x90, 0x61, 0xF9, 0x01, 0x00,
                0x00, 0x73,
            ],
            vec![
                0xBE, 0x61, 0x07, 0x0C, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x80,
                0x00, 0x00, 0x04,
            ],
            vec![
                0xBE, 0x03, 0x06, 0x08, 0x00, 0xF0, 0x0F, 0x3C, 0x00, 0x10, 0x00, 0x00, 0xE5,
            ],
            vec![
                0xBE, 0x03, 0x05, 0x08, 0x00, 0xE0, 0x0F, 0x3C, 0x00, 0x10, 0x00, 0x00, 0xF6,
            ],
        ];
        for mut v in test_messages {
            let old_checksum = v.pop().unwrap();
            let new_checksum = calculate_message_checksum(&v);
            assert_eq!(old_checksum, new_checksum);
        }
    }
    #[test]
    fn test_validate_packet_checksum() {
        let test_messages: Vec<Vec<u8>> = vec![
            vec![0xBE, 0x50, 0x00, 0x03, 0x00, 0x00, 0x01, 0xED],
            vec![0xBE, 0x50, 0x00, 0x01, 0x01, 0xEF],
            vec![0xBE, 0x53, 0x00, 0x01, 0x00, 0xED],
            vec![0xBE, 0x65, 0x02, 0x01, 0x11, 0xC8],
            vec![0xBE, 0x65, 0x03, 0x01, 0x12, 0xC6],
            vec![
                0xBE, 0x62, 0xC1, 0x0B, 0x00, 0x80, 0x00, 0x00, 0xAB, 0x77, 0x7F, 0xF4, 0x00, 0x00,
                0x00, 0xFE,
            ],
            vec![
                0xBE, 0x62, 0xC2, 0x0B, 0x00, 0x80, 0x00, 0x00, 0x34, 0x90, 0x61, 0xF9, 0x01, 0x00,
                0x00, 0x73,
            ],
            vec![
                0xBE, 0x61, 0x07, 0x0C, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x80,
                0x00, 0x00, 0x04,
            ],
            vec![
                0xBE, 0x03, 0x06, 0x08, 0x00, 0xF0, 0x0F, 0x3C, 0x00, 0x10, 0x00, 0x00, 0xE5,
            ],
            vec![
                0xBE, 0x03, 0x05, 0x08, 0x00, 0xE0, 0x0F, 0x3C, 0x00, 0x10, 0x00, 0x00, 0xF6,
            ],
        ];
        for v in test_messages {
            assert!(validate_packet_checksum(&v).is_ok())
        }
    }
}
