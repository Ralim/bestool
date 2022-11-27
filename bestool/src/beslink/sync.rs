use crate::beslink::errors::BESLinkError;
use crate::beslink::message::MessageTypes;
use crate::beslink::packet::read_packet;
use crate::beslink::BesMessage;
use serialport::SerialPort;
use tracing::warn;
pub fn sync(
    serial_port: &mut Box<dyn SerialPort>,
    sync_type: MessageTypes,
) -> Result<BesMessage, BESLinkError> {
    println!("Finding Sync on the port for type {:?}", sync_type);
    loop {
        match read_packet(serial_port) {
            Ok(packet) => {
                if packet.type1 == sync_type {
                    return Ok(packet);
                } else {
                    warn!(
                        "Ignored packet type {:?} waiting for {:?}",
                        packet.type1, sync_type
                    );
                }
            }
            Err(e) => return Err(e),
        }
    }
    // return Err(BESLinkError::Timeout);
}
