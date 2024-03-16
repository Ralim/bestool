use std::io::Error as BESLinkError;

pub fn validate_packet_checksum(packet: &[u8]) -> Result<(), BESLinkError> {
    let checksum = calculate_message_checksum(&packet[1..packet.len()]);
    if checksum == packet[packet.len() - 1] {
        return Ok(());
    }
//    let e = BESLinkError::BadChecksumError {
//       failed_packet: packet.to_vec(),
//       got: packet[packet.len() - 1],
//       wanted: checksum,
//   };
//    warn!("Bad Checksum!! {:?}", e);

    Err(BESLinkError::other("Error."))
}

pub fn calculate_message_checksum(packet: &[u8]) -> u8 {
    let mut sum: u32 = 0;
    for b in packet {
        sum += u32::from(*b);
        sum &= 0xFF;
    }
    (0xFF - sum) as u8
}
