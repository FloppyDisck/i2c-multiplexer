use embedded_hal::i2c::{Error, ErrorKind};
use thiserror::Error;

pub type Result<T, I2cError> = core::result::Result<T, MultiplexerError<I2cError>>;

#[derive(Error, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum MultiplexerError<I2cError> where I2cError: Error {
    #[error("Write Read I2C Error")]
    WriteReadI2CError,
    #[error("Write I2C Error")]
    WriteI2CError,
    #[error("Read I2C Error")]
    ReadI2CError,
    #[error("Incorrect port supplied")]
    PortError,
    #[error("I2C Error")]
    I2CError(I2cError)
}

impl<I2cError> Error for MultiplexerError<I2cError> where I2cError: Error {
    fn kind(&self) -> ErrorKind {
        match self { 
            Self::I2CError(e) => e.kind(),
            _ => ErrorKind::Other
        }
    }
}