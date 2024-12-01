use thiserror::Error;

pub type Result<T> = core::result::Result<T, MultiplexerError>;

#[derive(Error, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum MultiplexerError {
    #[error("Write Read I2C Error")]
    WriteReadI2CError,
    #[error("Write I2C Error")]
    WriteI2CError,
    #[error("Read I2C Error")]
    ReadI2CError,
    #[error("Incorrect port supplied")]
    PortError,
}
