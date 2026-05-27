# BME280 Rust Driver (Async & Typestate)

High-performance `no_std` Rust driver for BME280 sensors. It leverages the Typestate pattern for compile-time safety, supporting both blocking and async (`embedded-hal` 1.0) operations. Features I2C/SPI compatibility and a fluent Builder API for robust, reliable embedded applications.

## Features

- **Async First**: Native support for `embedded-hal-async` 1.0.
- **Typestate Safety**: Compile-time enforcement of sensor states (`Sleep`, `Normal`, `Forced`).
- **Dual Interface**: Seamless support for both I2C and SPI (with automatic CS management).
- **Robustness**: A 10ms post-reset OTP reload guard.
- **Fluent API**: Modern `Bme280Builder` for elegant configuration.
- **no_std**: Zero-cost abstractions suitable for bare-metal development.

## Quick Start

### I2C Example (Async)

```rust
use bme280_driver::{Bme280Builder, OsrsT, OsrsP, OsrsH, FilterMode, TsbMode};
use embassy_time::Duration;

// Initialize via Builder
let mut bme = Bme280Builder::new()
    .oversampling_temp(OsrsT::X2)
    .oversampling_pressure(OsrsP::X16)
    .oversampling_humidity(OsrsH::X1)
    .filter(FilterMode::F4)
    .standby(TsbMode::SB125)
    .build_i2c(i2c_bus, 0x77);

// Initialize hardware (includes reset and loading OTP calibration registers)
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

## Running Examples & Tests

The workspace contains two run targets:

### 1. Host Simulation & Unit Tests
You can compile and run a simulated I2C sensor target on your host PC:
```bash
# Run host mock example
cargo host-run

# Run comprehensive unit tests
cargo test --no-default-features --features "i2c spi async embassy-time/std embassy-time/generic-queue-64"
```

### 2. Embedded Target (nRF52840 DK)
To build and run the driver on an actual nRF52840 DK board (using embassy-nrf's TWIM peripheral):
```bash
cargo run -p bme280_app --bin nrf52840
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bme280_driver = { git = "git@github.com:ywang727/bme280.git" }
```

## License
- MIT license


