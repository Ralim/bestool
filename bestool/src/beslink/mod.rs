mod errors;
mod message;
mod packet;
mod sync;

pub const BES_PROGRAMMING_BAUDRATE: u32 = 921600;
pub use errors::BESLinkError;
pub use sync::sync;
