use crate::beslink::packet::calculate_packet_checksum;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MessageTypes {
    Sync = 0x50,
    FlashRead = 0x03,
    StartProgrammer = 0x53,
    ProgrammerRunning = 0x54,
    ProgrammerStart = 0x55,
    ProgrammerInit = 0x60,
    FlashCommand = 0x65,
    EraseBurnStart = 0x61,
    FlashBurnData = 0x62,
}
impl TryFrom<u8> for MessageTypes {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == MessageTypes::Sync as u8 => Ok(MessageTypes::Sync),
            x if x == MessageTypes::StartProgrammer as u8 => Ok(MessageTypes::StartProgrammer),
            x if x == MessageTypes::ProgrammerRunning as u8 => Ok(MessageTypes::ProgrammerRunning),
            x if x == MessageTypes::ProgrammerInit as u8 => Ok(MessageTypes::ProgrammerInit),
            x if x == MessageTypes::FlashCommand as u8 => Ok(MessageTypes::FlashCommand),
            x if x == MessageTypes::EraseBurnStart as u8 => Ok(MessageTypes::EraseBurnStart),
            x if x == MessageTypes::FlashBurnData as u8 => Ok(MessageTypes::FlashBurnData),
            x if x == MessageTypes::FlashRead as u8 => Ok(MessageTypes::FlashRead),
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
        return result;
    }
    pub fn set_checksum(&mut self) {
        let mut v = self.to_vec();
        v.pop();
        self.checksum = calculate_packet_checksum(&v);
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

        msg.payload = d[1..d.len() - 1].to_vec();

        return msg;
    }
}
