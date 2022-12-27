use thiserror::Error;

#[derive(Error, Debug)]
pub enum BESLinkError {
    #[error("IOError")]
    IOError(#[from] std::io::Error),
    #[error("SerialPortError")]
    SerialPortError(#[from] serialport::Error),
    #[error("BadChecksumError Bad checksum; got {got:?} wanted {wanted:?} : {failed_packet:X?}")]
    BadChecksumError {
        failed_packet: Vec<u8>,
        got: u8,
        wanted: u8,
    },
    #[error("BadResponseCode Bad result; got {got:?} wanted {wanted:?} : {failed_packet:X?}")]
    BadResponseCode {
        failed_packet: Vec<u8>,
        got: u8,
        wanted: u8,
    },
    #[error("Invalid Argument")]
    InvalidArgs,
    // #[error("Communications timed out")]
    // Timeout,
}
