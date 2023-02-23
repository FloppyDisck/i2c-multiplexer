pub mod error;

use embedded_hal::blocking::i2c;
use error::{Result, MultiplexerError};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// TODO: work with builder idea
// Multiplexer::new(i2c).with_address(device_address).route(impl Into<u8>, impl Into<u8>).route(port, address) ; done
// Include A0, A1, A2 translation 1 1 1 0 A2 A1 A0 R/W - R/W determines if read or write

// Follow up with x x x x 3 2 1 0 (channels), bit determines if enabled / disabled

#[derive(Copy, Clone, Debug)]
pub enum PortState {
    Enabled,
    Disabled
}

impl Into<bool> for PortState {
    fn into(self) -> bool {
        match self {
            PortState::Enabled => true,
            PortState::Disabled => false
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Multiplexer<I2C: 'static + Send + Sync> {
    i2c: I2C,
    address: u8,
    state: [bool; 4]
}

impl<I2C> Multiplexer<I2C>
    where
        I2C: i2c::WriteRead + i2c::Write + Send + Sync,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: 0x70,
            state: [false; 4],
        }
    }

    /// Sets the address according to the enabled hardware settings
    pub fn with_address_pins(mut self, a0: bool, a1: bool, a2: bool) -> Self {
        // TODO: might have a bit issue in the last bit
        self.address = 0x70;
        if a0 {
            self.address |= 0b00000001;
        }
        if a1 {
            self.address |= 0b00000010;
        }
        if a2 {
            self.address |= 0b00000100;
        }
        self
    }

    /// Sets the address
    pub fn with_address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }
}

impl<I2C> Multiplexer<I2C>
    where
        I2C: i2c::WriteRead + i2c::Write + Send + Sync,
{
    fn port_code(states: [bool; 4]) -> u8 {
        let mut code = 0;
        if states[0] {
            code |= 0b000_0001;
        }
        if states[1] {
            code |= 0b000_0010;
        }
        if states[2] {
            code |= 0b000_0100;
        }
        if states[3] {
            code |= 0b000_1000;
        }

        code
    }

    /// Enables / Disables the selected port
    pub fn set_port(&mut self, port: u8, state: impl Into<bool>) -> Result<()> {
        if port <= 4 {
            return Err(MultiplexerError::PortError);
        }

        self.state[port] = state.into();

        let code = Self::port_code(self.state);

        self.i2c_write(&[code])
    }

    /// Enables the selected port
    pub fn with_port(mut self, port: u8) -> Result<Self> {
        self.set_port(port, PortState::Enabled)?;
        Ok(self)
    }

    /// Enables / Disables the selected ports
    pub fn set_ports(&mut self, ports: [impl Into<u8>; 4]) -> Result<()> {
        let code = Self::port_code(ports.iter().map(|x| x.into()).collect());
        self.i2c_write(&[code])
    }

    /// Enables / Disables the selected ports
    pub fn with_ports(mut self, ports: [impl Into<u8>; 4]) -> Result<Self> {
        self.set_ports(ports)?;
        Ok(self)
    }

    fn i2c_write(&mut self, bytes: &[u8]) -> Result<()> {
        match self.i2c.write(self.address, bytes) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::WriteI2CError),
        }
    }

    fn i2c_read(&mut self, bytes: &[u8], buffer: &mut [u8]) -> Result<()> {
        match self.i2c.write_read(self.address, bytes, buffer) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::WriteReadI2CError),
        }
    }
}
