#[cfg(feature = "i2c")]
use crate::i2c_sensor::{Bme280I2C, I2cTrait};
use crate::regs::*;
#[cfg(feature = "spi")]
use crate::spi_sensor::{Bme280Spi, SpiDeviceTrait};
use crate::states::Sleep;

pub struct Bme280Builder {
    config: Config,
    timeout: Option<embassy_time::Duration>,
}

impl Bme280Builder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            timeout: None,
        }
    }

    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            timeout: None,
        }
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn oversampling_temp(mut self, osrs_t: OsrsT) -> Self {
        self.config.osrs_t = osrs_t;
        self
    }

    pub fn oversampling_pressure(mut self, osrs_p: OsrsP) -> Self {
        self.config.osrs_p = osrs_p;
        self
    }

    pub fn oversampling_humidity(mut self, osrs_h: OsrsH) -> Self {
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

    pub fn timeout(mut self, timeout: embassy_time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[cfg(feature = "i2c")]
    pub fn build_i2c<I2C>(self, i2c: I2C, addr: u8) -> Bme280I2C<I2C, Sleep>
    where
        I2C: I2cTrait,
    {
        let mut sensor = Bme280I2C::new_with_config(i2c, addr, self.config);
        if let Some(timeout) = self.timeout {
            sensor.timeout = timeout;
        }
        sensor
    }

    #[cfg(feature = "spi")]
    pub fn build_spi<SPI>(self, spi: SPI) -> Bme280Spi<SPI, Sleep>
    where
        SPI: SpiDeviceTrait,
    {
        let mut sensor = Bme280Spi::new_with_config(spi, self.config);
        if let Some(timeout) = self.timeout {
            sensor.timeout = timeout;
        }
        sensor
    }
}

impl Default for Bme280Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_time::Duration;

    #[cfg(feature = "i2c")]
    #[test]
    fn test_builder_i2c() {
        use embedded_hal_mock::eh1::i2c::Mock as I2cMock;
        let i2c = I2cMock::new(&[]);
        let sensor = Bme280Builder::new()
            .oversampling_temp(OsrsT::X2)
            .oversampling_pressure(OsrsP::X4)
            .oversampling_humidity(OsrsH::X8)
            .filter(FilterMode::F4)
            .standby(TsbMode::SB250)
            .spi3w(true)
            .timeout(Duration::from_millis(200))
            .build_i2c(i2c, 0x76);

        assert_eq!(sensor.config.osrs_t, OsrsT::X2);
        assert_eq!(sensor.config.osrs_p, OsrsP::X4);
        assert_eq!(sensor.config.osrs_h, OsrsH::X8);
        assert_eq!(sensor.config.filter, FilterMode::F4);
        assert_eq!(sensor.config.standby, TsbMode::SB250);
        assert_eq!(sensor.config.spi3w, true);
        assert_eq!(sensor.timeout, Duration::from_millis(200));
    }

    #[cfg(feature = "spi")]
    #[test]
    fn test_builder_spi() {
        use embedded_hal_mock::eh1::spi::Mock as SpiMock;
        let spi = SpiMock::new(&[]);
        let sensor = Bme280Builder::new()
            .oversampling_temp(OsrsT::X16)
            .oversampling_pressure(OsrsP::Skipped)
            .oversampling_humidity(OsrsH::X1)
            .filter(FilterMode::Off)
            .standby(TsbMode::SB1000)
            .spi3w(false)
            .timeout(Duration::from_millis(50))
            .build_spi(spi);

        assert_eq!(sensor.config.osrs_t, OsrsT::X16);
        assert_eq!(sensor.config.osrs_p, OsrsP::Skipped);
        assert_eq!(sensor.config.osrs_h, OsrsH::X1);
        assert_eq!(sensor.config.filter, FilterMode::Off);
        assert_eq!(sensor.config.standby, TsbMode::SB1000);
        assert_eq!(sensor.config.spi3w, false);
        assert_eq!(sensor.timeout, Duration::from_millis(50));
    }
}
