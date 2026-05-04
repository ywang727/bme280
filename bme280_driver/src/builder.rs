use crate::regs::*;
use crate::states::Sleep;
#[cfg(feature = "i2c")]
use crate::i2c_sensor::{Bme280I2C, I2cTrait};
#[cfg(feature = "spi")]
use crate::spi_sensor::{Bme280Spi, SpiDeviceTrait};

pub struct Bme280Builder {
    config: Config,
}

impl Bme280Builder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn osrs_t(mut self, osrs_t: OsrsT) -> Self {
        self.config.osrs_t = osrs_t;
        self
    }

    pub fn osrs_p(mut self, osrs_p: OsrsP) -> Self {
        self.config.osrs_p = osrs_p;
        self
    }

    pub fn osrs_h(mut self, osrs_h: OsrsH) -> Self {
        self.config.osrs_h = osrs_h;
        self
    }

    pub fn filter(mut self, filter: FilterMode) -> Self {
        self.config.filter = filter;
        self
    }

    pub fn standby(mut self, standby: TsbMode) -> Self {
        self.config.standby = standby;
        self
    }

    pub fn spi3w(mut self, spi3w: bool) -> Self {
        self.config.spi3w = spi3w;
        self
    }

    #[cfg(feature = "i2c")]
    pub fn build_i2c<I2C>(self, i2c: I2C, addr: u8) -> Bme280I2C<I2C, Sleep>
    where
        I2C: I2cTrait,
    {
        Bme280I2C::new_with_config(i2c, addr, self.config)
    }

    #[cfg(feature = "spi")]
    pub fn build_spi<SPI>(self, spi: SPI) -> Bme280Spi<SPI, Sleep>
    where
        SPI: SpiDeviceTrait,
    {
        Bme280Spi::new_with_config(spi, self.config)
    }
}

impl Default for Bme280Builder {
    fn default() -> Self {
        Self::new()
    }
}
