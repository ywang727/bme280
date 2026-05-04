use bitfield::bitfield;

pub const REG_RESET: u8 = 0xE0;
pub const REG_CHIP_ID: u8 = 0xD0;
pub const REG_CTRL_HUM: u8 = 0xF2;
pub const REG_STATUS: u8 = 0xF3;
pub const REG_CTRL_MEAS: u8 = 0xF4;
pub const REG_CONFIG: u8 = 0xF5;

pub const REG_ALLDATA_START: u8 = 0xF7;

pub const REG_PRESS_MSB: u8 = 0xF7;
pub const REG_PRESS_LSB: u8 = 0xF8;
pub const REG_PRESS_XLSB: u8 = 0xF9;

pub const REG_TEMP_MSB: u8 = 0xFA;
pub const REG_TEMP_LSB: u8 = 0xFB;
pub const REG_TEMP_XLSB: u8 = 0xFC;

pub const REG_HUMI_MSB: u8 = 0xFD;
pub const REG_HUMI_LSB: u8 = 0xFE;

pub const CALIB1_TEMP_START: u8 = 0x88;
pub const CALIB1_TEMP_LEN: u8 = 26;

pub const CALIB2_HUM_START: u8 = 0xE1;
pub const CALIB_HUM_LEN: u8 = 7;

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct Config {
    pub mode: Mode,
    pub osrs_t: OsrsT,
    pub osrs_p: OsrsP,
    pub osrs_h: OsrsH,
    pub standby: TsbMode,
    pub filter: FilterMode,
    pub spi3w: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Sleep,
            osrs_t: OsrsT::X1,
            osrs_p: OsrsP::X1,
            osrs_h: OsrsH::X1,
            standby: TsbMode::SB125,
            filter: FilterMode::Off,
            spi3w: false,
        }
    }
}

impl Config {
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_osrs_t(mut self, osrs_t: OsrsT) -> Self {
        self.osrs_t = osrs_t;
        self
    }

    pub fn with_osrs_p(mut self, osrs_p: OsrsP) -> Self {
        self.osrs_p = osrs_p;
        self
    }

    pub fn with_osrs_h(mut self, osrs_h: OsrsH) -> Self {
        self.osrs_h = osrs_h;
        self
    }

    pub fn with_standby(mut self, standby: TsbMode) -> Self {
        self.standby = standby;
        self
    }

    pub fn with_filter(mut self, filter: FilterMode) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_spi3w(mut self, spi3w: bool) -> Self {
        self.spi3w = spi3w;
        self
    }
}

bitfield! {
    #[derive(Copy, Clone, PartialEq )]

    pub struct MeasurementData(u64);
    impl Debug;


    pub u8, p_msb, _: 7, 0;
    pub u8, p_lsb, _: 15, 8;
    pub u8, p_xlsb, _: 23, 20;
    pub u8, t_msb, _: 31, 24;
    pub u8, t_lsb, _: 39, 32;
    pub u8, t_xlsb, _: 47, 44;
    pub u8, h_msb, _: 55, 48;
    pub u8, h_lsb, _: 63, 56;
}

impl MeasurementData {
    pub fn pressure(&self) -> u32 {
        ((self.p_msb() as u32) << 12) | ((self.p_lsb() as u32) << 4) | (self.p_xlsb() as u32)
    }

    pub fn temperature(&self) -> u32 {
        ((self.t_msb() as u32) << 12) | ((self.t_lsb() as u32) << 4) | (self.t_xlsb() as u32)
    }

    pub fn humidity(&self) -> u16 {
        ((self.h_msb() as u16) << 8) | (self.h_lsb() as u16)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum TsbMode {
    SB0p5,
    SB62p5,
    SB125,
    SB250,
    SB500,
    SB1000,
    SB2000,
    SB4000,
}

impl From<u8> for TsbMode {
    fn from(value: u8) -> Self {
        match value {
            0b000 => TsbMode::SB0p5,
            0b001 => TsbMode::SB62p5,
            0b010 => TsbMode::SB125,
            0b011 => TsbMode::SB250,
            0b100 => TsbMode::SB500,
            0b101 => TsbMode::SB1000,
            0b110 => TsbMode::SB2000,
            0b111 => TsbMode::SB4000,
            _ => TsbMode::SB4000,
        }
    }
}

impl From<TsbMode> for u8 {
    fn from(value: TsbMode) -> Self {
        match value {
            TsbMode::SB0p5 => 0b000,
            TsbMode::SB62p5 => 0b001,
            TsbMode::SB125 => 0b010,
            TsbMode::SB250 => 0b011,
            TsbMode::SB500 => 0b100,
            TsbMode::SB1000 => 0b101,
            TsbMode::SB2000 => 0b110,
            TsbMode::SB4000 => 0b111,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum FilterMode {
    Off,
    F2,
    F4,
    F8,
    F16,
}

impl From<u8> for FilterMode {
    fn from(value: u8) -> Self {
        match value {
            0b000 => FilterMode::Off,
            0b001 => FilterMode::F2,
            0b010 => FilterMode::F4,
            0b011 => FilterMode::F8,
            _ => FilterMode::F16,
        }
    }
}

impl From<FilterMode> for u8 {
    fn from(value: FilterMode) -> Self {
        match value {
            FilterMode::Off => 0b000,
            FilterMode::F2 => 0b001,
            FilterMode::F4 => 0b010,
            FilterMode::F8 => 0b011,
            FilterMode::F16 => 0b100,
        }
    }
}

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct ConfigReg(u8);
    impl Debug;

    pub u8, from into TsbMode, t_sb, set_t_sb: 7, 5;
    pub u8, from into FilterMode, filter, set_filter: 4, 2;
    pub u8, spi3w_enable, set_spi3w_enable: 0, 0;
}

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct CtrlMeasReg(u8);
    impl Debug;
    pub u8, from into OsrsT, osrs_t, set_osrs_t: 7, 5;
    pub u8, from into OsrsP, osrs_p, set_osrs_p: 4, 2;
    pub u8, from into Mode, mode, set_mode: 1, 0;
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum OsrsT {
    Skipped,
    X1,
    X2,
    X4,
    X8,
    X16,
}

impl From<OsrsT> for u8 {
    fn from(value: OsrsT) -> Self {
        match value {
            OsrsT::Skipped => 0b000,
            OsrsT::X1 => 0b001,
            OsrsT::X2 => 0b010,
            OsrsT::X4 => 0b011,
            OsrsT::X8 => 0b100,
            OsrsT::X16 => 0b101,
        }
    }
}

impl From<u8> for OsrsT {
    fn from(value: u8) -> Self {
        match value {
            0b000 => OsrsT::Skipped,
            0b001 => OsrsT::X1,
            0b010 => OsrsT::X2,
            0b011 => OsrsT::X4,
            0b100 => OsrsT::X8,
            _ => OsrsT::X16,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum OsrsP {
    Skipped,
    X1,
    X2,
    X4,
    X8,
    X16,
}

impl From<OsrsP> for u8 {
    fn from(value: OsrsP) -> Self {
        match value {
            OsrsP::Skipped => 0b000,
            OsrsP::X1 => 0b001,
            OsrsP::X2 => 0b010,
            OsrsP::X4 => 0b011,
            OsrsP::X8 => 0b100,
            OsrsP::X16 => 0b101,
        }
    }
}

impl From<u8> for OsrsP {
    fn from(value: u8) -> Self {
        match value {
            0b000 => OsrsP::Skipped,
            0b001 => OsrsP::X1,
            0b010 => OsrsP::X2,
            0b011 => OsrsP::X4,
            0b100 => OsrsP::X8,
            _ => OsrsP::X16,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum Mode {
    Sleep,
    Forced,
    Normal,
}

impl From<Mode> for u8 {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Sleep => 0b00,
            Mode::Forced => 0b01,
            Mode::Normal => 0b11,
        }
    }
}

impl From<u8> for Mode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Mode::Sleep,
            0b01 | 0b10 => Mode::Forced,
            0b11 => Mode::Normal,
            _ => Mode::Sleep,
        }
    }
}

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct StatusReg(u8);
    impl Debug;
    pub u8, measuring, _: 3, 3;
    pub u8, im_update, _: 0, 0;
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum OsrsH {
    Skipped,
    X1,
    X2,
    X4,
    X8,
    X16,
}

impl From<OsrsH> for u8 {
    fn from(value: OsrsH) -> Self {
        match value {
            OsrsH::Skipped => 0b000,
            OsrsH::X1 => 0b001,
            OsrsH::X2 => 0b010,
            OsrsH::X4 => 0b011,
            OsrsH::X8 => 0b100,
            OsrsH::X16 => 0b101,
        }
    }
}

impl From<u8> for OsrsH {
    fn from(value: u8) -> Self {
        match value {
            0b000 => OsrsH::Skipped,
            0b001 => OsrsH::X1,
            0b010 => OsrsH::X2,
            0b011 => OsrsH::X4,
            0b100 => OsrsH::X8,
            _ => OsrsH::X16,
        }
    }
}

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct CtrlHumReg(u8);
    impl Debug;
    pub u8, from into OsrsH, osrs_h, set_osrs_h: 2, 0;
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct Reset(u8);
    impl Debug;
    pub u8, _,set_reset: 7, 0;

}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct ID(u8);
    impl Debug;
    pub u8, id, _: 7, 0;
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct CalibrationData1(pub [u8; 26]); //   0x88 to 0xA1

impl CalibrationData1 {
    pub fn dig_t1(&self) -> u16 {
        u16::from_le_bytes([self.0[0], self.0[1]])
    }
    pub fn dig_t2(&self) -> i16 {
        i16::from_le_bytes([self.0[2], self.0[3]])
    }
    pub fn dig_t3(&self) -> i16 {
        i16::from_le_bytes([self.0[4], self.0[5]])
    }
    pub fn dig_p1(&self) -> u16 {
        u16::from_le_bytes([self.0[6], self.0[7]])
    }
    pub fn dig_p2(&self) -> i16 {
        i16::from_le_bytes([self.0[8], self.0[9]])
    }
    pub fn dig_p3(&self) -> i16 {
        i16::from_le_bytes([self.0[10], self.0[11]])
    }
    pub fn dig_p4(&self) -> i16 {
        i16::from_le_bytes([self.0[12], self.0[13]])
    }
    pub fn dig_p5(&self) -> i16 {
        i16::from_le_bytes([self.0[14], self.0[15]])
    }
    pub fn dig_p6(&self) -> i16 {
        i16::from_le_bytes([self.0[16], self.0[17]])
    }
    pub fn dig_p7(&self) -> i16 {
        i16::from_le_bytes([self.0[18], self.0[19]])
    }
    pub fn dig_p8(&self) -> i16 {
        i16::from_le_bytes([self.0[20], self.0[21]])
    }
    pub fn dig_p9(&self) -> i16 {
        i16::from_le_bytes([self.0[22], self.0[23]])
    }
    pub fn dig_h1(&self) -> u8 {
        self.0[25]
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct CalibrationData2(pub [u8; 16]); //   0xE1 to 0xF0

impl CalibrationData2 {
    pub fn dig_h2(&self) -> i16 {
        i16::from_le_bytes([self.0[0], self.0[1]])
    }
    pub fn dig_h3(&self) -> u8 {
        self.0[2]
    }

    pub fn dig_h4(&self) -> i16 {
        let val = ((self.0[3] as i16) << 4) | (self.0[4] as i16 & 0x0F);
        if val > 2047 { val - 4096 } else { val }
    }

    pub fn dig_h5(&self) -> i16 {
        let val = ((self.0[5] as i16) << 4) | (self.0[4] as i16 >> 4);
        if val > 2047 { val - 4096 } else { val }
    }

    pub fn dig_h6(&self) -> i8 {
        self.0[6] as i8
    }
}
