use bme280_driver::{Bme280Builder, FilterMode, OsrsH, OsrsP, OsrsT, TsbMode};
use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use futures::executor::block_on;

fn main() {
    println!("Starting BME280 I2C Async Example...");

    const ADDR: u8 = 0x76;

    // Define mock expectations for a complete sensor lifecycle:
    // 1. init(): Reset -> Read calibration1 -> Read calibration2 -> Apply Config
    // 2. into_normal_mode(): Apply config (mode = Normal)
    // 3. read_all(): Read data registers
    // 4. stop(): Transition back to Sleep mode
    let expectations = [
        // 1. init()
        // Write REG_RESET (0xE0) = 0xB6
        I2cTrans::write(ADDR, vec![0xE0, 0xB6]),
        // Read CALIB1 (0x88) - 26 bytes (we fill with realistic mock values)
        I2cTrans::write_read(ADDR, vec![0x88], vec![
            0xAC, 0x6B, // dig_T1 (27500)
            0x03, 0x67, // dig_T2 (26371)
            0x32, 0x00, // dig_T3 (50)
            0x88, 0x8E, // dig_P1 (36488)
            0x56, 0xD6, // dig_P2 (-10666)
            0xD0, 0x0B, // dig_P3 (3024)
            0x7A, 0x1E, // dig_P4 (7802)
            0x6D, 0x00, // dig_P5 (109)
            0xF9, 0xFF, // dig_P6 (-7)
            0x8D, 0x3C, // dig_P7 (15501)
            0xF9, 0xC6, // dig_P8 (-14600)
            0x60, 0x0B, // dig_P9 (2912)
            0x00, 0x00, // padding & dig_H1
        ]),
        // Read CALIB2 (0xE1) - 16 bytes
        I2cTrans::write_read(ADDR, vec![0xE1], vec![
            0x63, 0x01, // dig_H2 (355)
            0x00,       // dig_H3 (0)
            0x14, 0x00, // dig_H4, dig_H5
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]),
        // Apply Config: CTRL_HUM, CONFIG, CTRL_MEAS
        I2cTrans::write(ADDR, vec![0xF2, 0x01]), // CTRL_HUM (osrs_h = X1)
        I2cTrans::write(ADDR, vec![0xF5, 0x40]), // CONFIG (filter = Off, standby = 125ms)
        I2cTrans::write(ADDR, vec![0xF4, 0x24]), // CTRL_MEAS (osrs_t = X1, osrs_p = X1, mode = Sleep)

        // 2. into_normal_mode()
        I2cTrans::write(ADDR, vec![0xF2, 0x01]),
        I2cTrans::write(ADDR, vec![0xF5, 0x40]),
        I2cTrans::write(ADDR, vec![0xF4, 0x27]), // mode = Normal (0x24 | 0x03)

        // 3. read_all()
        // Read data starting from REG_ALLDATA_START (0xF7) - 8 bytes
        I2cTrans::write_read(ADDR, vec![0xF7], vec![
            0x51, 0x93, 0x00, // Pressure raw (0x51930)
            0x83, 0x9C, 0x00, // Temperature raw (0x839C0)
            0x68, 0x6C,       // Humidity raw (0x686C)
        ]),

        // 4. stop()
        I2cTrans::write(ADDR, vec![0xF2, 0x01]),
        I2cTrans::write(ADDR, vec![0xF5, 0x40]),
        I2cTrans::write(ADDR, vec![0xF4, 0x24]), // mode = Sleep (0x24)
    ];

    let i2c = I2cMock::new(&expectations);

    // Build the BME280 sensor using the Builder API
    let sensor = Bme280Builder::new()
        .oversampling_temp(OsrsT::X1)
        .oversampling_pressure(OsrsP::X1)
        .oversampling_humidity(OsrsH::X1)
        .filter(FilterMode::Off)
        .standby(TsbMode::SB125)
        .build_i2c(i2c, ADDR);

    block_on(async {
        let mut sensor = sensor;

        // Initialize the sensor (reset, read calibration, write initial configuration)
        println!("Initializing BME280 sensor...");
        sensor.init().await.expect("Failed to initialize BME280");

        // Transition the sensor to Normal mode (continuous measurement)
        println!("Transitioning sensor to Normal Mode...");
        let mut normal_sensor = sensor.into_normal_mode().await.expect("Failed to transition to Normal Mode");

        // Read all measurements (temperature, pressure, humidity)
        println!("Reading measurement data...");
        let (temp, press, hum) = normal_sensor.read_all().await.expect("Failed to read measurements");

        println!("--------------------------------------");
        println!("BME280 Measurement Results:");
        println!("Temperature : {:.2} °C", temp);
        println!("Pressure    : {:.2} hPa", press / 100.0);
        println!("Humidity    : {:.2} %", hum);
        println!("--------------------------------------");

        // Stop the sensor (returns ownership as Sleep mode sensor)
        println!("Stopping sensor (returning to Sleep Mode)...");
        let sleep_sensor = normal_sensor.stop().await.expect("Failed to stop sensor");

        // Verify all I2C transactions matched perfectly
        sleep_sensor.into_inner().done();
        println!("Success! All I2C expectations met.");
    });
}
