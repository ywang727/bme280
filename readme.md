# BME280 Rust Driver (Async & Typestate)

High-performance `no_std` Rust driver for BME280 sensors. It leverages the Typestate pattern for compile-time safety, supporting both blocking and async (`embedded-hal` 1.0) operations. Features I2C/SPI compatibility, a fluent Builder API, and safety timeouts for robust, reliable embedded applications.

## Features

- **Async First**: Native support for `embedded-hal-async` 1.0.
- **Typestate Safety**: Compile-time enforcement of sensor states (`Sleep`, `Normal`, `Forced`).
- **Dual Interface**: Seamless support for both I2C and SPI (with automatic CS management).
- **Robustness**: Built-in 150ms safety timeouts for measurements.
- **Fluent API**: Modern `Bme280Builder` for elegant configuration.
- **no_std**: Zero-cost abstractions suitable for bare-metal development.

## Quick Start

### I2C Example (Async)

```rust
use bme280_driver::{Bme280Builder, OsrsT, FilterMode, TsbMode};

// Initialize via Builder
let mut bme = Bme280Builder::new()
    .osrs_t(OsrsT::X16)
    .filter(FilterMode::X4)
    .standby(TsbMode::Ms10)
    .build_i2c(i2c_bus, 0x76);

// Initialize hardware
bme.init().await?;

// Switch to Normal Mode for continuous sampling
let mut bme_normal = bme.into_normal_mode().await?;
let (temp, press, hum) = bme_normal.read_all().await?;
```

### Forced Mode (Single Measurement)

```rust
// Transition to Forced Mode from Sleep
let mut bme_forced = bme.into_forced_mode().await?;

// Trigger, wait (with timeout), and read
let (temp, press, hum) = bme_forced.read_once().await?;
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bme280_driver = { git = "git@github.com:ywang727/bme280.git" }
```

## License
- MIT license

