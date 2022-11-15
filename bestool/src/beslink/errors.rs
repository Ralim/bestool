use thiserror::Error;

#[derive(Error, Debug)]
pub enum BESLinkError {
    #[error("IOError")]
    IOError(#[from] std::io::Error),
    #[error("BadChecksumError Bad checksum; got {got:?} wanted {wanted:?} : {failed_packet:?}")]
    BadChecksumError {
        failed_packet: Vec<u8>,
        got: u8,
        wanted: u8,
    },

    #[error("Communications timed out")]
    Timeout,
    #[error("unknown data store error")]
    Unknown,
}