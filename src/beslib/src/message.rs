use std::io::Write;

use tracing::{debug, error, info, warn};

#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MessageKind {
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
    #[default]
    UnknownOrInfo = 0x66, // Unknown at this point in time, but references "OR Info"; suspect NOR flash info
}

impl From<MessageKind> for u8 {
    fn from(v: MessageKind) -> Self {
        v as Self
    }
}

impl From<u8> for MessageKind {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Self::DeviceCommand,
            0x03 => Self::FlashRead,
            0x50 => Self::Sync,
            0x53 => Self::StartProgrammer,
            0x54 => Self::ProgrammerRunning,
            0x55 => Self::ProgrammerStart,
            0x60 => Self::ProgrammerInit,
            0x61 => Self::EraseBurnStart,
            0x62 => Self::FlashBurnData,
            0x65 => Self::FlashCommand,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BesMessage {
    pub sync: u8,
    pub msg_type: MessageKind,
    pub payload: Vec<u8>,
    pub checksum: u8,
}

impl From<BesMessage> for Vec<u8> {
    fn from(mut val: BesMessage) -> Self {
        debug!("Converting BesMessage to Vec<u8>");
        let mut packet: Vec<u8> = vec![];
        packet.push(val.sync);
        packet.push(val.msg_type.into());
        packet.append(&mut val.payload);
        packet.push(val.checksum);

        packet
    }
}

impl From<&mut BesMessage> for Vec<u8> {
    fn from(val: &mut BesMessage) -> Self {
        debug!("Converting &mut BesMessage to Vec<u8>");
        let mut packet: Vec<u8> = vec![];
        packet.push(val.sync);
        packet.push(val.msg_type.into());
        packet.append(&mut val.payload);
        packet.push(val.checksum);

        packet
    }
}

impl From<&BesMessage> for Vec<u8> {
    fn from(val: &BesMessage) -> Self {
        debug!("Converting &BesMessage to Vec<u8>");
        let mut val = val.clone();
        let mut packet: Vec<u8> = vec![];
        packet.push(val.sync);
        packet.push(val.msg_type.into());
        packet.append(&mut val.payload);
        packet.push(val.checksum);

        packet
    }
}

impl From<Vec<u8>> for BesMessage {
    fn from(v: Vec<u8>) -> Self {
        let mut msg = BesMessage {
            sync: v[0],
            msg_type: MessageKind::Sync,
            payload: vec![],
            checksum: v[v.len() - 1],
        };

        msg.msg_type = v[1].into();
        msg.payload = v[2..v.len() - 1].to_vec();

        msg
    }
}

impl BesMessage {
    pub fn set_checksum(&mut self) {
        let mut v: Vec<u8> = self.into();
        v.pop();
        self.checksum = crate::utils::calculate_message_checksum(&v);
    }

    pub fn send_packet(&self, writer: &mut dyn Write) -> std::io::Result<()> {
        let packet: Vec<u8>;

        if let Ok(p) = self.try_into() {
            packet = p;
        } else {
            panic!("Failed to convert BesMessage to Vec<u8>");
        }// Return Error.

        return match writer.write_all(packet.as_slice()) {
            Ok(_) => {
                debug!("Sent message to chip: {packet:X?}", packet = packet);
                debug!("Wrote {len} bytes.", len = packet.len());

                // Serial port should be flushed?

                info!("Sent message type: {:?}", self.msg_type);
                Ok(())
            }
            Err(e) => Err(e),
        };
    }
}
