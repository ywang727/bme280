use thiserror::Error;

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature = "i2c")]
pub enum I2cError {
    #[error("I2C read failure")]
    Read,
    #[error("I2C write failure")]
    Write,
    #[error("Interrupt pin error")]
    Interrupt,
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature = "spi")]
pub enum SpiError {
    #[error("SPI read failure")]
    Read,
    #[error("SPI write failure")]
    Write,
    #[error("Interrupt pin error")]
    Interrupt,
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    #[cfg(feature = "i2c")]
    #[error("I2C error: {0}")]
    I2c(I2cError),

    #[cfg(feature = "spi")]
    #[error("SPI error: {0}")]
    Spi(SpiError),

    #[error("Calibration data is not available")]
    CalibDataNotAvailable,
}

pub type Result<T> = core::result::Result<T, Error>;
