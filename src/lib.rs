#[cfg(feature = "bus")]
pub mod bus;
pub mod error;

use embedded_hal::blocking::i2c;
use error::{MultiplexerError, Result};

pub mod prelude {
    #[cfg(feature = "bus")]
    pub use crate::bus::{BusPort, MultiplexerBus};
    pub use crate::{error::MultiplexerError, Multiplexer, PortState};
}

#[derive(Copy, Clone, Debug)]
pub enum PortState {
    Enabled,
    Disabled,
}

impl From<bool> for PortState {
    fn from(value: bool) -> Self {
        match value {
            true => PortState::Enabled,
            false => PortState::Disabled,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Multiplexer<I2C: 'static + Send + Sync> {
    i2c: I2C,
    address: u8,
    state: [bool; 4],
}

pub(crate) fn address_from_pins(a0: bool, a1: bool, a2: bool) -> u8 {
    let mut address = 0b1110_0000;
    if a0 {
        address |= 0b0000_0001;
    }
    if a1 {
        address |= 0b0000_0010;
    }
    if a2 {
        address |= 0b0000_0100;
    }
    address
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
        self.address = address_from_pins(a0, a1, a2);
        self
    }

    /// Sets the address
    pub fn with_address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }

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
}

impl<I2C> Multiplexer<I2C>
where
    I2C: i2c::WriteRead + i2c::Write + Send + Sync,
{
    /// Disables all ports
    pub fn with_ports_disabled(self) -> Result<Self> {
        self.with_ports([false; 4])
    }

    /// Disables all ports
    pub fn set_ports_disabled(mut self) -> Result<()> {
        self.set_ports([false; 4])
    }

    /// Enables all ports
    pub fn with_ports_enabled(self) -> Result<Self> {
        self.with_ports([true; 4])
    }

    /// Enables all ports
    pub fn set_ports_enabled(mut self) -> Result<()> {
        self.set_ports([true; 4])
    }

    /// Enables / Disables the selected port
    pub fn set_port(&mut self, port: u8, state: impl Into<bool>) -> Result<()> {
        if port >= 4 {
            return Err(MultiplexerError::PortError);
        }

        self.state[port as usize] = state.into();

        let code = Self::port_code(self.state);

        self.i2c_write(&[code])
    }

    /// Sets the selected port
    pub fn with_port(mut self, port: u8, state: impl Into<bool>) -> Result<Self> {
        self.set_port(port, state.into())?;
        Ok(self)
    }

    /// Enables / Disables the selected ports
    pub fn set_ports(&mut self, ports: [bool; 4]) -> Result<()> {
        let code = Self::port_code(ports);
        self.i2c_write(&[code])
    }

    /// Enables / Disables the selected ports
    pub fn with_ports(mut self, ports: [bool; 4]) -> Result<Self> {
        self.set_ports(ports)?;
        Ok(self)
    }

    fn i2c_write(&mut self, bytes: &[u8]) -> Result<()> {
        match self.i2c.write(self.address, bytes) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::WriteI2CError),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use embedded_hal_mock::i2c::Mock;
    use rstest::*;

    #[rstest]
    #[case([true;4], 0b0000_1111)]
    #[case([false;4], 0b0000_0000)]
    #[case([true, false, true, false], 0b0000_0101)]
    fn setup_ports(#[case] ports: [bool; 4], #[case] result: u8) {
        assert_eq!(Multiplexer::<Mock>::port_code(ports), result)
    }

    #[rstest]
    #[case([true;3], 0b1110_0111)]
    #[case([false;3], 0b1110_0000)]
    #[case([true, false, false], 0b1110_0001)]
    #[case([false, true, false], 0b1110_0010)]
    #[case([true, false, true], 0b1110_0101)]
    fn setup_address(#[case] addr: [bool; 3], #[case] result: u8) {
        assert_eq!(
            Multiplexer::new(Mock::new([]))
                .with_address_pins(addr[0], addr[1], addr[2])
                .address,
            result
        )
    }
}
