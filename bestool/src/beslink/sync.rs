use crate::beslink::errors::BESLinkError;
use crate::beslink::message::MessageTypes;
use crate::beslink::packet::read_packet;
use serialport::SerialPort;

pub fn sync(
    serial_port: &mut Box<dyn SerialPort>,
    sync_type: MessageTypes,
) -> Result<(), BESLinkError> {
    println!("Finding Sync on the port");

    match read_packet(serial_port) {
        Ok(packet) => {
            if packet.type1 == sync_type {
                return Ok(());
            }
        }
        Err(e) => return Err(e),
    }
    return Err(BESLinkError::Timeout);
}
