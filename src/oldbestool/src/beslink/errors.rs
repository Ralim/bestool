#[derive(Debug)]
pub enum BESLinkError {
    IOError {
        e: std::io::Error,
    },
    BadChecksumError {
        failed_packet: Vec<u8>,
        got: u8,
        wanted: u8,
    },
    BadResponseCode {
        failed_packet: Vec<u8>,
        got: u8,
        wanted: u8,
    },
    InvalidArgs,
    // #[error("Communications timed out")]
    // Timeout,
}

impl From<std::io::Error> for BESLinkError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError { e: value }
    }
}
