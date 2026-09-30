#![allow(unused)]

pub(crate) const ADDR_58X: u8 = 0x50;
pub(crate) const ADDR_585: u8 = 0x51;

pub(crate) const CHIP_ID: u8 = 0x01;
pub(crate) const REV_ID: u8 = 0x02;

pub(crate) const CHIP_STATUS: u8 = 0x11;

pub(crate) const DRIVE_CONFIG: u8 = 0x13;
pub(crate) const INT_CONFIG: u8 = 0x14;
pub(crate) const INT_SOURCE: u8 = 0x15;
pub(crate) const FIFO_CONFIG: u8 = 0x16;
pub(crate) const FIFO_COUNT: u8 = 0x17;
pub(crate) const FIFO_SEL: u8 = 0x18;

pub(crate) const RESERVED_REG_0: u8 = 0x1C;
pub(crate) const TEMP_DATA_XLSB: u8 = 0x1D;
pub(crate) const TEMP_DATA_LSB: u8 = 0x1E;
pub(crate) const TEMP_DATA_MSB: u8 = 0x1F;
pub(crate) const PRESS_DATA_XLSB: u8 = 0x20;
pub(crate) const PRESS_DATA_LSB: u8 = 0x21;
pub(crate) const PRESS_DATA_MSB: u8 = 0x22;
pub(crate) const RESERVED_REG1: u8 = 0x23;
pub(crate) const RESERVED_REG2: u8 = 0x24;
pub(crate) const RESERVED_REG3: u8 = 0x25;
pub(crate) const RESERVED_REG4: u8 = 0x26;
pub(crate) const INT_STATUS: u8 = 0x27;
pub(crate) const STATUS: u8 = 0x28;
pub(crate) const FIFO_DATA: u8 = 0x29;

pub(crate) const NVM_ADDR: u8 = 0x2B;
pub(crate) const NVM_DATA_LSB: u8 = 0x2C;
pub(crate) const NVM_DATA_MSB: u8 = 0x2D;

pub(crate) const DSP_CONFIG: u8 = 0x30;
pub(crate) const DSP_IIR: u8 = 0x31;
pub(crate) const OOR_THR_P_LSB: u8 = 0x32;
pub(crate) const OOR_THR_P_MSB: u8 = 0x33;
pub(crate) const OOR_RANGE: u8 = 0x34;
pub(crate) const OOR_CONFIG: u8 = 0x35;
pub(crate) const OSR_CONFIG: u8 = 0x36;
pub(crate) const ODR_CONFIG: u8 = 0x37;
pub(crate) const OSR_EFF: u8 = 0x38;

pub(crate) const CMD: u8 = 0x7E;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OSR {
    X1 = 0x00,
    X2 = 0x01,
    X4 = 0x02,
    X8 = 0x03,
    X16 = 0x04,
    X32 = 0x05,
    X64 = 0x06,
    X128 = 0x07
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ODR {
    Hz240 = 0x00,
    Hz218p5 = 0x01,
    Hz199p1 = 0x02,
    Hz179p2 = 0x03,
    Hz160 = 0x04,
    Hz149p3 = 0x05,
    Hz140 = 0x06,
    Hz129p8 = 0x07,
    Hz120 = 0x08,
    Hz110p1 = 0x09,
    Hz100p2 = 0x0A,
    Hz89p6 = 0x0B,
    Hz80 = 0x0C,
    Hz70 = 0x0D,
    Hz60 = 0x0E,
    Hz50 = 0x0F,
    Hz45 = 0x10,
    Hz40 = 0x11,
    Hz35 = 0x12,
    Hz30 = 0x13,
    Hz25 = 0x14,
    Hz20 = 0x15,
    Hz15 = 0x16,
    Hz10 = 0x17,
    Hz05 = 0x18,
    Hz04 = 0x19,
    Hz03 = 0x1A,
    Hz02 = 0x1B,
    Hz01 = 0x1C,
    Hz0p5 = 0x1D,
    Hz0p250 = 0x1E,
    Hz0p125 = 0x1F,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerMode {
    /// Stops sensor measurements to enter power-saving.
    Standby = 0b00,
    /// Sample sensors at the rate set by OSR/ODR config.
    Normal = 0b01,
    /// Sample sensors only when the user requests to take a measurement.
    Forced = 0b10,
    /// Sample sensors as fast as possible.
    Continuous = 0b11,
}