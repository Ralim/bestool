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

use std::fmt;

impl fmt::Display for BESLinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BESLinkError::IOError { e } => write!(f, "IO error: {e}"),
            BESLinkError::BadChecksumError {
                failed_packet,
                got,
                wanted,
            } => {
                write!(
                    f,
                    "Bad checksum error: failed_packet={failed_packet:?}, got={got}, wanted={wanted}"
                )
            }
            BESLinkError::BadResponseCode {
                failed_packet,
                got,
                wanted,
            } => {
                write!(
                    f,
                    "Bad response code: failed_packet={failed_packet:?}, got={got}, wanted={wanted}"
                )
            }
            BESLinkError::InvalidArgs => write!(f, "Invalid arguments"),
        }
    }
}
