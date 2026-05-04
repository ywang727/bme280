#![cfg_attr(not(test), no_std)]
#![recursion_limit = "256"]

pub mod builder;
pub mod error;
pub mod fmt;
pub mod regs;
pub mod states;

#[cfg(feature = "i2c")]
pub mod i2c_sensor;

#[cfg(feature = "spi")]
pub mod spi_sensor;

pub use crate::builder::Bme280Builder;
pub use crate::error::*;
pub use crate::states::*;
pub use crate::regs::{Config, FilterMode, Mode, OsrsH, OsrsP, OsrsT, StatusReg, TsbMode};

#[cfg(feature = "i2c")]
pub use crate::i2c_sensor::Bme280I2C;
#[cfg(feature = "spi")]
pub use crate::spi_sensor::Bme280Spi;
