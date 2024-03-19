use crate::beslink::errors::BESLinkError;
use crate::beslink::message::read_message;
use crate::beslink::message::MessageTypes;
use crate::beslink::BesMessage;
use serialport::SerialPort;
use tracing::{debug, warn};
pub fn sync(
    serial_port: &mut Box<dyn SerialPort>,
    sync_type: MessageTypes,
) -> Result<BesMessage, BESLinkError> {
    debug!("Finding Sync on the port for type {:?}", sync_type);
    loop {
        match read_message(serial_port) {
            Ok(packet) => {
                if packet.type1 == sync_type {
                    return Ok(packet);
                } else {
                    warn!(
                        "Ignored packet type {:?} waiting for {:?} => {:X?}",
                        packet.type1,
                        sync_type,
                        packet.to_vec()
                    );
                }
            }
            Err(e) => match e {
                BESLinkError::BadChecksumError { .. } => {
                    warn!("Ignoring bad checksum; you might not be in programmer mode.")
                }
                _ => return Err(e),
            },
        }
    }
    // return Err(BESLinkError::Timeout);
}
