use crate::beslink::errors::BESLinkError;
use crate::beslink::message::MessageTypes;
use crate::beslink::packet::read_packet;
use serialport::SerialPort;

pub fn sync(mut serial_port: Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    println!("Finding Sync on the port");

    match read_packet(serial_port) {
        Ok(packet) => {
            if matches!(packet.type1, MessageTypes::Sync) {
                return Ok(());
            }
        }
        Err(e) => return Err(e),
    }
    return Err(BESLinkError::Timeout);
}
