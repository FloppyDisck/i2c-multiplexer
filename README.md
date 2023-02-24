# I2C-Multiplexer &emsp; [![Build Status]][actions] [![Latest Version]][crates.io]
[Build Status]: https://img.shields.io/github/actions/workflow/status/FloppyDisck/i2c-multiplexer/rust.yml?branch=main
[actions]: https://github.com/FloppyDisck/i2c-multiplexer/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/i2c-multiplexer.svg
[crates.io]: https://crates.io/crates/i2c-multiplexer
An I2C Multiplexer library that supports the PCA9546 and TCA9546A chips

---

## Usage
The sensor is initialized 
```rust
use i2c_multiplexer::prelude::*;

fn main() -> Result<()> {
    // Disable all ports and only enable port 0
    Multiplexer::new(i2c).with_ports_disabled()?.set_port(0, true)?;
}
```

## Changing Address
```rust
use i2c_multiplexer::prelude::*;

fn main() -> Result<()> {
    // Manually set the address
    Multiplexer::new(i2c).with_address(0x72);
    
    // Or set it according to the selected hardware pins
    // This uses A0 which means the address is 0x71
    Multiplexer::new(i2c).with_address_pins(true, false, false);
}
```

## Setting multiple ports
```rust
use i2c_multiplexer::prelude::*;

fn main() -> Result<()> {
    // Manually set the ports 0,2 to enabled and 1,3 to disabled
    Multiplexer::new(i2c).with_ports([true, false, true, false])?;
}
```