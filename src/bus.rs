use crate::{address_from_pins, error::MultiplexerError, error::Result as CrateResult};
use embedded_hal::blocking::i2c::{Read, SevenBitAddress, Write, WriteRead};

pub struct MultiplexerBus {
    address: u8,
}

impl MultiplexerBus {
    pub fn new() -> Self {
        Self { address: 0x70 }
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

    pub fn new_port<I2C: 'static + Send + Sync>(&self, i2c: I2C, port: u8) -> BusPort<I2C> {
        let id = match port {
            0 => 0b000_0001,
            1 => 0b000_0010,
            2 => 0b000_0100,
            _ => 0b000_1000,
        };

        BusPort {
            bus: i2c,
            address: self.address,
            port: id,
        }
    }
}

pub struct BusPort<I2C: 'static + Send + Sync> {
    bus: I2C,
    address: u8,
    port: u8,
}

impl<I2C> BusPort<I2C>
where
    I2C: Write + Send + Sync,
{
    fn open_port(&mut self) -> CrateResult<()> {
        match self.bus.write(self.address, &[self.port]) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::WriteI2CError),
        }
    }
}

impl<I2C> Write for BusPort<I2C>
where
    I2C: Write + Send + Sync,
{
    type Error = MultiplexerError;

    fn write(&mut self, address: SevenBitAddress, bytes: &[u8]) -> Result<(), Self::Error> {
        self.open_port()?;
        match self.bus.write(address, bytes) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::WriteI2CError),
        }
    }
}

impl<I2C> Read for BusPort<I2C>
where
    I2C: Read + Write + Send + Sync,
{
    type Error = MultiplexerError;

    fn read(&mut self, address: SevenBitAddress, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.open_port()?;
        match self.bus.read(address, bytes) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::ReadI2CError),
        }
    }
}

impl<I2C> WriteRead for BusPort<I2C>
where
    I2C: WriteRead + Write + Send + Sync,
{
    type Error = MultiplexerError;

    fn write_read(
        &mut self,
        address: SevenBitAddress,
        buffer_in: &[u8],
        buffer_out: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.open_port()?;
        match self.bus.write_read(address, buffer_in, buffer_out) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::WriteReadI2CError),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use embedded_hal::prelude::{
        _embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_Write,
        _embedded_hal_blocking_i2c_WriteRead,
    };
    use embedded_hal_mock::common::Generic;
    use embedded_hal_mock::i2c::{Mock, Transaction};

    #[test]
    fn multi_port_write() {
        let multiplexer_addr = 0x01;
        let component_addr = 0x02;

        // Use port 1, 3, 2, 4 in that order
        let ports = vec![
            (0, 0b000_0001),
            (2, 0b000_0100),
            (1, 0b000_0010),
            (3, 0b000_1000),
        ];

        let expectations = [
            Transaction::write(multiplexer_addr, vec![ports[0].1]),
            Transaction::write(component_addr, vec![0x05, 0x43]),
            Transaction::write(multiplexer_addr, vec![ports[1].1]),
            Transaction::write(component_addr, vec![0x55]),
            Transaction::write(multiplexer_addr, vec![ports[2].1]),
            Transaction::write(component_addr, vec![0x07, 0x39, 0x87]),
            Transaction::write(multiplexer_addr, vec![ports[3].1]),
            Transaction::write(component_addr, vec![0x45, 0x48]),
        ];

        let i2c = Mock::new(&expectations);
        let bus = shared_bus::new_std!(Generic<Transaction> = i2c).unwrap();
        let multiplexer = MultiplexerBus::new().with_address(multiplexer_addr);

        let mut multiplexed_i2c_a = multiplexer.new_port(bus.acquire_i2c(), ports[0].0);
        let mut multiplexed_i2c_b = multiplexer.new_port(bus.acquire_i2c(), ports[1].0);
        let mut multiplexed_i2c_c = multiplexer.new_port(bus.acquire_i2c(), ports[2].0);
        let mut multiplexed_i2c_d = multiplexer.new_port(bus.acquire_i2c(), ports[3].0);

        assert!(multiplexed_i2c_a
            .write(component_addr, &[0x05, 0x43])
            .is_ok());
        assert!(multiplexed_i2c_b.write(component_addr, &[0x55]).is_ok());
        assert!(multiplexed_i2c_c
            .write(component_addr, &[0x07, 0x39, 0x87])
            .is_ok());
        assert!(multiplexed_i2c_d
            .write(component_addr, &[0x45, 0x48])
            .is_ok());
    }

    #[test]
    fn multi_port_read() {
        let multiplexer_addr = 0x01;
        let component_addr = 0x02;

        // Use port 1, 3, 2, 4 in that order
        let ports = vec![
            (0, 0b000_0001),
            (2, 0b000_0100),
            (1, 0b000_0010),
            (3, 0b000_1000),
        ];

        let expectations = [
            Transaction::write(multiplexer_addr, vec![ports[0].1]),
            Transaction::read(component_addr, vec![0x05, 0x43]),
            Transaction::write(multiplexer_addr, vec![ports[1].1]),
            Transaction::read(component_addr, vec![0x55]),
            Transaction::write(multiplexer_addr, vec![ports[2].1]),
            Transaction::read(component_addr, vec![0x07, 0x39, 0x87]),
            Transaction::write(multiplexer_addr, vec![ports[3].1]),
            Transaction::read(component_addr, vec![0x45, 0x48]),
        ];

        let i2c = Mock::new(&expectations);
        let bus = shared_bus::new_std!(Generic<Transaction> = i2c).unwrap();
        let multiplexer = MultiplexerBus::new().with_address(multiplexer_addr);

        let mut multiplexed_i2c_a = multiplexer.new_port(bus.acquire_i2c(), ports[0].0);
        let mut multiplexed_i2c_b = multiplexer.new_port(bus.acquire_i2c(), ports[1].0);
        let mut multiplexed_i2c_c = multiplexer.new_port(bus.acquire_i2c(), ports[2].0);
        let mut multiplexed_i2c_d = multiplexer.new_port(bus.acquire_i2c(), ports[3].0);

        let mut ma = [0; 2];
        assert!(multiplexed_i2c_a.read(component_addr, &mut ma).is_ok());
        assert_eq!(ma, [0x05, 0x43]);

        let mut mb = [0; 1];
        assert!(multiplexed_i2c_b.read(component_addr, &mut mb).is_ok());
        assert_eq!(mb, [0x55]);

        let mut mc = [0; 3];
        assert!(multiplexed_i2c_c.read(component_addr, &mut mc).is_ok());
        assert_eq!(mc, [0x07, 0x39, 0x87]);

        let mut md = [0; 2];
        assert!(multiplexed_i2c_d.read(component_addr, &mut md).is_ok());
        assert_eq!(md, [0x45, 0x48]);
    }

    #[test]
    fn multi_port_read_write() {
        let multiplexer_addr = 0x01;
        let component_addr = 0x02;

        // Use port 1, 3, 2, 4 in that order
        let ports = vec![
            (0, 0b000_0001),
            (2, 0b000_0100),
            (1, 0b000_0010),
            (3, 0b000_1000),
        ];

        let expectations = [
            Transaction::write(multiplexer_addr, vec![ports[0].1]),
            Transaction::write_read(component_addr, vec![0x05, 0x43], vec![0x33, 0x43]),
            Transaction::write(multiplexer_addr, vec![ports[1].1]),
            Transaction::write_read(component_addr, vec![0x55], vec![0x33, 0x43]),
            Transaction::write(multiplexer_addr, vec![ports[2].1]),
            Transaction::write_read(component_addr, vec![0x07, 0x39, 0x87], vec![0x33, 0x43]),
            Transaction::write(multiplexer_addr, vec![ports[3].1]),
            Transaction::write_read(component_addr, vec![0x45, 0x48], vec![0x33, 0x43]),
        ];

        let i2c = Mock::new(&expectations);
        let bus = shared_bus::new_std!(Generic<Transaction> = i2c).unwrap();
        let multiplexer = MultiplexerBus::new().with_address(multiplexer_addr);

        let mut multiplexed_i2c_a = multiplexer.new_port(bus.acquire_i2c(), ports[0].0);
        let mut multiplexed_i2c_b = multiplexer.new_port(bus.acquire_i2c(), ports[1].0);
        let mut multiplexed_i2c_c = multiplexer.new_port(bus.acquire_i2c(), ports[2].0);
        let mut multiplexed_i2c_d = multiplexer.new_port(bus.acquire_i2c(), ports[3].0);

        let mut ma = [0x33, 0x43];
        assert!(multiplexed_i2c_a
            .write_read(component_addr, &[0x05, 0x43], &mut ma)
            .is_ok());
        assert_eq!(ma, [0x33, 0x43]);

        let mut mb = [0x33, 0x43];
        assert!(multiplexed_i2c_b
            .write_read(component_addr, &[0x55], &mut mb)
            .is_ok());
        assert_eq!(mb, [0x33, 0x43]);

        let mut mc = [0x33, 0x43];
        assert!(multiplexed_i2c_c
            .write_read(component_addr, &[0x07, 0x39, 0x87], &mut mc)
            .is_ok());
        assert_eq!(mc, [0x33, 0x43]);

        let mut md = [0x33, 0x43];
        assert!(multiplexed_i2c_d
            .write_read(component_addr, &[0x45, 0x48], &mut md)
            .is_ok());
        assert_eq!(md, [0x33, 0x43]);
    }
}
