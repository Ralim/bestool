use crate::beslink;
use crate::beslink::{load_programmer_runtime_binary_blob, BESLinkError, BES_PROGRAMMING_BAUDRATE};
use serialport::SerialPort;

pub fn cmd_write_image(_input_file: String, serial_port: String) {
    //First gain sync to the device
    println!(
        "Opening serial monitor to {} @ {}",
        serial_port, BES_PROGRAMMING_BAUDRATE
    );
    let serial_port = serialport::new(serial_port, BES_PROGRAMMING_BAUDRATE);
    match serial_port.open() {
        Ok(mut port) => {
            let _ = sync_into_bootloader(&mut port);
            let _ = load_programmer_runtime_binary_blob(&mut port);
        }
        Err(e) => println!("Failed to open serial port - {:?}", e),
    }
}

fn sync_into_bootloader(serial_port: &mut Box<dyn SerialPort>) -> Result<(), BESLinkError> {
    // Gain sync

    let _ = beslink::sync(serial_port, beslink::MessageTypes::Sync)?;
    // Send message to stay in bootloader
    let msg = beslink::BesMessage {
        sync: beslink::BES_SYNC,
        type1: beslink::MessageTypes::Sync,
        payload: vec![0x00, 0x01, 0x01],
        checksum: 0xEF,
    };
    return match beslink::send_packet(serial_port, msg) {
        Ok(_) => Ok(()),
        Err(e) => Err(BESLinkError::from(e)),
    };
}
