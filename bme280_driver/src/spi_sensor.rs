use crate::error::{Error, Result, SpiError};
use crate::fmt::*;
use crate::regs::*;
use crate::states::*;
use core::marker::PhantomData;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};

#[cfg(not(feature = "async"))]
pub use embedded_hal::spi::SpiDevice as SpiDeviceTrait;
#[cfg(feature = "async")]
pub use embedded_hal_async::spi::SpiDevice as SpiDeviceTrait;

#[derive(Debug)]
pub struct Bme280Spi<SPI, MODE = Sleep> {
    pub(crate) spi: SPI,
    pub config: Config,
    pub(crate) cal1: Option<CalibrationData1>,
    pub(crate) cal2: Option<CalibrationData2>,
    _mode: PhantomData<MODE>,
}

impl<SPI, MODE> Bme280Spi<SPI, MODE>
where
    SPI: SpiDeviceTrait,
    MODE: SensorState,
{
    pub async fn read_reg(&mut self, reg: u8, buf: &mut [u8]) -> Result<()> {
        let addr = reg | 0x80;
        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(&[addr]),
                embedded_hal::spi::Operation::Read(buf),
            ])
            .await
            .map_err(|_| Error::Spi(SpiError::Read))
    }

    pub async fn write_reg(&mut self, reg: u8, val: u8) -> Result<()> {
        trace!("BME280 SPI: write reg 0x{:02x} = 0x{:02x}", reg, val);
        let addr = reg & 0x7F;
        self.spi
            .write(&[addr, val])
            .await
            .map_err(|_| Error::Spi(SpiError::Write))
    }

    pub async fn reset(&mut self) -> Result<()> {
        self.write_reg(REG_RESET, 0xB6).await
    }

    pub async fn device_id(&mut self) -> Result<u8> {
        let mut id = [0u8; 1];
        self.read_reg(REG_CHIP_ID, &mut id).await?;
        Ok(id[0])
    }

    pub async fn init(&mut self) -> Result<()> {
        self.reset().await?;
        self.cal1 = Some(self.get_calibration1_data().await?);
        self.cal2 = Some(self.get_calibration2_data().await?);
        self.apply_config().await?;
        Ok(())
    }

    pub async fn apply_config(&mut self) -> Result<()> {
        let mut hum_reg = CtrlHumReg(0);
        hum_reg.set_osrs_h(self.config.osrs_h);
        self.write_reg(REG_CTRL_HUM, hum_reg.0).await?;

        let mut reg_val = ConfigReg(0);
        reg_val.set_t_sb(self.config.standby);
        reg_val.set_filter(self.config.filter);
        reg_val.set_spi3w_enable(self.config.spi3w as u8);
        self.write_reg(REG_CONFIG, reg_val.0).await?;

        let mut meas_reg = CtrlMeasReg(0);
        meas_reg.set_osrs_t(self.config.osrs_t);
        meas_reg.set_osrs_p(self.config.osrs_p);
        meas_reg.set_mode(self.config.mode);
        self.write_reg(REG_CTRL_MEAS, meas_reg.0).await?;

        Ok(())
    }

    pub async fn get_calibration1_data(&mut self) -> Result<CalibrationData1> {
        let mut data = [0u8; CALIB1_TEMP_LEN as usize];
        self.read_reg(CALIB1_TEMP_START, &mut data).await?;
        Ok(CalibrationData1(data))
    }

    pub async fn get_calibration2_data(&mut self) -> Result<CalibrationData2> {
        let mut data = [0u8; 16];
        self.read_reg(CALIB2_HUM_START, &mut data).await?;
        Ok(CalibrationData2(data))
    }

    pub async fn get_cal_refs(&mut self) -> Result<(&CalibrationData1, &CalibrationData2)> {
        if self.cal1.is_none() || self.cal2.is_none() {
            self.init().await?;
        }
        let c1 = self.cal1.as_ref().ok_or(Error::CalibDataNotAvailable)?;
        let c2 = self.cal2.as_ref().ok_or(Error::CalibDataNotAvailable)?;
        Ok((c1, c2))
    }

    pub async fn status(&mut self) -> Result<StatusReg> {
        let mut buf = [0u8; 1];
        self.read_reg(REG_STATUS, &mut buf).await?;
        Ok(StatusReg(buf[0]))
    }

    fn calculate_compensation(
        meas: MeasurementData,
        cal1: &CalibrationData1,
        cal2: &CalibrationData2,
    ) -> Result<(f32, f32, f32)> {
        let adc_t = meas.temperature() as f32;
        let var1_t = (adc_t / 16384.0 - (cal1.dig_t1() as f32) / 1024.0) * (cal1.dig_t2() as f32);
        let var2_t = ((adc_t / 131072.0 - (cal1.dig_t1() as f32) / 8192.0)
            * (adc_t / 131072.0 - (cal1.dig_t1() as f32) / 8192.0))
            * (cal1.dig_t3() as f32);
        let t_fine = var1_t + var2_t;
        let temperature = t_fine / 5120.0;

        let adc_p = meas.pressure() as f32;
        let mut var1_p = (t_fine / 2.0) - 64000.0;
        let mut var2_p = var1_p * var1_p * (cal1.dig_p6() as f32) / 32768.0;
        var2_p = var2_p + var1_p * (cal1.dig_p5() as f32) * 2.0;
        var2_p = (var2_p / 4.0) + ((cal1.dig_p4() as f32) * 65536.0);
        var1_p = ((cal1.dig_p3() as f32) * var1_p * var1_p / 524288.0
            + (cal1.dig_p2() as f32) * var1_p)
            / 524288.0;
        var1_p = (1.0 + var1_p / 32768.0) * (cal1.dig_p1() as f32);

        let mut pressure = 0.0;
        if var1_p != 0.0 {
            pressure = 1048576.0 - adc_p;
            pressure = (pressure - (var2_p / 4096.0)) * 6250.0 / var1_p;
            var1_p = (cal1.dig_p9() as f32) * pressure * pressure / 2147483648.0;
            var2_p = pressure * (cal1.dig_p8() as f32) / 32768.0;
            pressure = pressure + (var1_p + var2_p + (cal1.dig_p7() as f32)) / 16.0;
        }

        let adc_h = meas.humidity() as f32;
        let mut h = t_fine - 76800.0;
        h = (adc_h - (cal2.dig_h4() as f32 * 64.0 + cal2.dig_h5() as f32 / 16384.0 * h))
            * (cal2.dig_h2() as f32 / 65536.0
                * (1.0
                    + cal2.dig_h6() as f32 / 67108864.0
                        * h
                        * (1.0 + cal2.dig_h3() as f32 / 67108864.0 * h)));
        h = h * (1.0 - cal1.dig_h1() as f32 * h / 524288.0);
        let humidity = h.clamp(0.0, 100.0);

        Ok((temperature, pressure, humidity))
    }
}

impl<SPI> Bme280Spi<SPI, Sleep>
where
    SPI: SpiDeviceTrait,
{
    pub fn new(spi: SPI) -> Self {
        Self::new_with_config(spi, Config::default())
    }

    pub(crate) fn new_with_config(spi: SPI, config: Config) -> Self {
        Self {
            spi,
            config,
            cal1: None,
            cal2: None,
            _mode: PhantomData,
        }
    }

    pub async fn into_normal_mode(mut self) -> Result<Bme280Spi<SPI, Normal>> {
        self.config.mode = Mode::Normal;
        self.apply_config().await?;
        Ok(Bme280Spi {
            spi: self.spi,
            config: self.config,
            cal1: self.cal1,
            cal2: self.cal2,
            _mode: PhantomData,
        })
    }

    pub async fn into_forced_mode(mut self) -> Result<Bme280Spi<SPI, Forced>> {
        self.config.mode = Mode::Sleep;
        self.apply_config().await?;

        Ok(Bme280Spi {
            spi: self.spi,
            config: self.config,
            cal1: self.cal1,
            cal2: self.cal2,
            _mode: PhantomData,
        })
    }
}

impl<SPI> Bme280Spi<SPI, Normal>
where
    SPI: SpiDeviceTrait,
{
    pub async fn read_all(&mut self) -> Result<(f32, f32, f32)> {
        let mut data = [0u8; 8];
        self.read_reg(REG_ALLDATA_START, &mut data).await?;
        let meas = MeasurementData(u64::from_le_bytes(data));
        let (cal1, cal2) = self.get_cal_refs().await?;
        Self::calculate_compensation(meas, cal1, cal2)
    }

    pub async fn stop(mut self) -> Result<Bme280Spi<SPI, Sleep>> {
        self.config.mode = Mode::Sleep;
        self.apply_config().await?;
        Ok(Bme280Spi {
            spi: self.spi,
            config: self.config,
            cal1: self.cal1,
            cal2: self.cal2,
            _mode: PhantomData,
        })
    }
}

impl<SPI> Bme280Spi<SPI, Forced>
where
    SPI: SpiDeviceTrait,
{
    pub async fn read_once(&mut self) -> Result<(f32, f32, f32)> {
        let mut meas_reg = CtrlMeasReg(0);
        meas_reg.set_osrs_t(self.config.osrs_t);
        meas_reg.set_osrs_p(self.config.osrs_p);
        meas_reg.set_mode(Mode::Forced);
        self.write_reg(REG_CTRL_MEAS, meas_reg.0).await?;

        let timeout = Timer::after(Duration::from_millis(150));
        let poller = async {
            loop {
                let s = self.status().await?;
                if s.measuring() == 0 {
                    break Ok(());
                }
                embassy_futures::yield_now().await;
            }
        };

        match select(timeout, poller).await {
            Either::First(_) => return Err(Error::Timeout),
            Either::Second(res) => res?,
        }

        let mut data = [0u8; 8];
        self.read_reg(REG_ALLDATA_START, &mut data).await?;
        let meas = MeasurementData(u64::from_le_bytes(data));
        let (cal1, cal2) = self.get_cal_refs().await?;
        Self::calculate_compensation(meas, cal1, cal2)
    }
}
