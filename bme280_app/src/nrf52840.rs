#![no_std]
#![no_main]

use bme280_driver::{Bme280Builder, FilterMode, OsrsH, OsrsP, OsrsT, TsbMode};
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, embassy_nrf as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("System initialized.");

    let config = twim::Config::default();
    static RAM_BUFFER: StaticCell<[u8; 64]> = StaticCell::new();

    // nRF52840 DK 默认 I2C 引脚：P0.26 为 SDA，P0.27 为 SCL
    let i2c = Twim::new(
        p.TWISPI0,
        Irqs,
        p.P0_06, // SDA
        p.P0_08, // SCL
        config,
        RAM_BUFFER.init([0; 64]),
    );

    const BME280_ADDR: u8 = 0x77;

    let sensor = Bme280Builder::new()
        .oversampling_temp(OsrsT::X2)
        .oversampling_pressure(OsrsP::X16)
        .oversampling_humidity(OsrsH::X1)
        .filter(FilterMode::F4)
        .standby(TsbMode::SB125)
        .timeout(Duration::from_millis(100))
        .build_i2c(i2c, BME280_ADDR);

    let mut sensor = sensor;

    info!("Initializing BME280...");
    if let Err(e) = sensor.init().await {
        error!("BME280 Init failed: {:?}", e);
        return;
    }

    info!("Entering Normal Mode...");
    let mut normal_sensor = match sensor.into_normal_mode().await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to enter Normal Mode: {:?}", e);
            return;
        }
    };

    loop {
        match normal_sensor.read_all().await {
            Ok((temp, press, hum)) => {
                info!("-----------------------------------------");
                info!("Temperature : {} °C", temp);
                info!("Pressure    : {} hPa", press / 100.0);
                info!("Humidity    : {} %", hum);
            }
            Err(e) => {
                error!("Read error: {:?}", e);
            }
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
