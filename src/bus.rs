use crate::address_from_pins;
use embedded_hal::i2c::{ErrorType, I2c, Operation, SevenBitAddress};
use crate::prelude::MultiplexerError;

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

    pub fn new_port<I2C>(&self, i2c: I2C, port: u8) -> BusPort<I2C> {
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

pub struct BusPort<I2C> {
    bus: I2C,
    address: u8,
    port: u8,
}

impl<I2C> BusPort<I2C>
where
    I2C: I2c,
{
    fn open_port(&mut self) -> Result<(), MultiplexerError<I2C::Error>> {
        match self.bus.write(self.address, &[self.port]) {
            Ok(res) => Ok(res),
            Err(_) => Err(MultiplexerError::PortError),
        }
    }
}

impl<I2C> ErrorType for BusPort<I2C> where I2C: I2c {
    type Error = MultiplexerError<I2C::Error>;
}

impl<I2C> I2c for BusPort<I2C>
where I2C: I2c
{
    fn read(&mut self, address: SevenBitAddress, read: &mut [u8]) -> Result<(), Self::Error> {
        self.open_port()?;
        self.bus.read(address, read).map_err(|err| MultiplexerError::I2CError(err))
    }

    fn write(&mut self, address: SevenBitAddress, write: &[u8]) -> Result<(), Self::Error> {
        self.open_port()?;
        self.bus.write(address, write).map_err(|err| MultiplexerError::I2CError(err))
    }

    fn write_read(&mut self, address: SevenBitAddress, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.open_port()?;
        self.bus.write_read(address, write, read).map_err(|err| MultiplexerError::I2CError(err))
    }

    fn transaction(&mut self, address: SevenBitAddress, operations: &mut [Operation<'_>]) -> Result<(), Self::Error> {
        self.open_port()?;
        self.bus.transaction(address, operations).map_err(|err| MultiplexerError::I2CError(err))
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;
    use alloc::vec;
    use core::cell::RefCell;
    use embedded_hal::i2c::I2c;
    use embedded_hal_bus::i2c::RefCellDevice;
    use crate::prelude::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

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

        let i2c = RefCell::new(Mock::new(&expectations));
        let multiplexer = MultiplexerBus::new().with_address(multiplexer_addr);

        {
            let mut multiplexed_i2c_a = multiplexer.new_port(RefCellDevice::new(&i2c), ports[0].0);
            let mut multiplexed_i2c_b = multiplexer.new_port(RefCellDevice::new(&i2c), ports[1].0);
            let mut multiplexed_i2c_c = multiplexer.new_port(RefCellDevice::new(&i2c), ports[2].0);
            let mut multiplexed_i2c_d = multiplexer.new_port(RefCellDevice::new(&i2c), ports[3].0);

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

        i2c.into_inner().done();
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

        let i2c = RefCell::new(Mock::new(&expectations));
        let multiplexer = MultiplexerBus::new().with_address(multiplexer_addr);

        {
            let mut multiplexed_i2c_a = multiplexer.new_port(RefCellDevice::new(&i2c), ports[0].0);
            let mut multiplexed_i2c_b = multiplexer.new_port(RefCellDevice::new(&i2c), ports[1].0);
            let mut multiplexed_i2c_c = multiplexer.new_port(RefCellDevice::new(&i2c), ports[2].0);
            let mut multiplexed_i2c_d = multiplexer.new_port(RefCellDevice::new(&i2c), ports[3].0);

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
        
        i2c.into_inner().done();
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

        let i2c = RefCell::new(Mock::new(&expectations));
        let multiplexer = MultiplexerBus::new().with_address(multiplexer_addr);

        {
            let mut multiplexed_i2c_a = multiplexer.new_port(RefCellDevice::new(&i2c), ports[0].0);
            let mut multiplexed_i2c_b = multiplexer.new_port(RefCellDevice::new(&i2c), ports[1].0);
            let mut multiplexed_i2c_c = multiplexer.new_port(RefCellDevice::new(&i2c), ports[2].0);
            let mut multiplexed_i2c_d = multiplexer.new_port(RefCellDevice::new(&i2c), ports[3].0);

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

        i2c.into_inner().done();
    }
}
