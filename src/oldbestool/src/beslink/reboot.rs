//Reboot message:
//0 - SYNC
//1 - 0
//2 - ?? 0/1/2
//3 - 0x05 -- Message length+5??
//4 - 0xF1

use crate::beslink::{send_message, sync, BESLinkError, BesMessage, MessageTypes, BES_SYNC};
use serialport::SerialPort;
use tracing::info;

pub fn send_device_reboot(
    serial_port: &mut Box<dyn SerialPort>,
) -> Result<BesMessage, BESLinkError> {
    let mut device_reboot_message = BesMessage {
        sync: BES_SYNC,
        type1: MessageTypes::DeviceCommand,
        payload: vec![0x00, 0x01, 0xF1],
        checksum: 0xEB,
    };
    device_reboot_message.set_checksum();

    info!(
        "Sent device reboot message, {:?}",
        device_reboot_message.to_vec()
    );
    send_message(serial_port, device_reboot_message)?;
    sync(serial_port, MessageTypes::DeviceCommand)
}
