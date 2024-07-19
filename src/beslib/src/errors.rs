use std::fmt;
use std::error;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum BesLinkError {
    IoError(IoError),
}

impl error::Error for BesLinkError {}

impl fmt::Display for BesLinkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BesLinkError::IoError(ref e) => write!(f, "IO error: {}", e)
                    
        }
    }
}

impl From<IoError> for BesLinkError {
    fn from(e: IoError) -> Self {
        BesLinkError::IoError(e)
    }
}
