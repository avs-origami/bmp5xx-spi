#![no_std]
use byteorder::{ByteOrder, LittleEndian};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::{Operation, SpiDevice};

mod addrs;
use addrs::*;
pub use addrs::{OSR, ODR, PowerMode};

#[derive(Debug)]
pub enum Error<E> {
    Spi(E),
    ChipId(u8),
    NotReady,
    BadConfig,
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::Spi(e)
    }
}

pub type Result<T, E> = core::result::Result<T, Error<E>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Val {
    pub t: f32,
    pub p: f32
}

impl From<(f32, f32)> for Val {
    fn from(value: (f32, f32)) -> Self {
        Val { t: value.0, p: value.1 }
    }
}

impl From<Val> for (f32, f32) {
    fn from(value: Val) -> Self {
        (value.t, value.p)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IntConfig {
    pub mode: IntMode,
    pub pol: IntPol,
    pub drive: IntDrive,
    pub sources: IntSources,
    pub enable: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IntSources {
    pub data_ready: bool,
    pub fifo_full: bool,
    pub fifo_threshold: bool,
    pub pressure_oor: bool,
}

impl IntSources {
    fn bits(&self) -> u8 {
        (self.data_ready as u8) | ((self.fifo_full as u8) << 1) | ((self.fifo_threshold as u8) << 2) | ((self.pressure_oor as u8) << 3)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BmpConfig {
    pub osr_t: OSR,
    pub osr_p: OSR,
    pub odr: ODR,
    pub pwr: PowerMode,
    pub p_en: bool,
}

impl Default for BmpConfig {
    fn default() -> Self {
        Self {
            osr_t: OSR::X2,
            osr_p: OSR::X16,
            odr: ODR::Hz50,
            pwr: PowerMode::Normal,
            p_en: true,
        }
    }
}

pub struct Bmp5xx<SPI: SpiDevice, D: DelayNs> {
    spi: SPI,
    delay: D,
    config: BmpConfig,
}

impl<SPI: SpiDevice, D: DelayNs> Bmp5xx<SPI, D> {
    /// Creates the Bmp5xx object. Must also initialize with sensor.init().
    pub fn new(spi: SPI, delay: D, config: BmpConfig) -> Self {
        return Self {
            spi,
            delay,
            config,
        };
    }

    async fn read_regs(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), SPI::Error> {
        self.spi.transaction(&mut [Operation::Write(&[reg | 0x80]), Operation::Read(buf)]).await?;
        Ok(())
    }

    async fn read_reg(&mut self, reg: u8) -> Result<u8, SPI::Error> {
        let mut b = [0u8; 1];
        self.read_regs(reg, &mut b).await?;
        Ok(b[0])
    }

    async fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), SPI::Error> {
        self.spi.write(&[reg & 0x7F, val]).await?;
        Ok(())
    }

    async fn standby(&mut self) -> Result<(), SPI::Error> {
        let odr = self.read_reg(ODR_CONFIG).await?;
        self.write_reg(ODR_CONFIG, (odr & !0x03) | 0x80).await?;
        self.delay.delay_us(2500).await;
        Ok(())
    }

    /// Reset the sensor and set SPI mode.
    pub async fn reset(&mut self) -> Result<(), SPI::Error> {
        self.write_reg(CMD, 0xB6).await?;
        self.delay.delay_ms(6).await;
        let _ = self.read_reg(CHIP_ID).await?;

        match self.read_reg(CHIP_ID).await? {
            ADDR_58X | ADDR_585 => Ok(()),
            id => Err(Error::ChipId(id)),
        }
    }

    /// Initialize the sensor, waking it up from standby and configuring
    /// the data output settings.
    pub async fn init(&mut self) -> Result<(), SPI::Error> {
        self.reset().await?;
        let status = self.read_reg(STATUS).await?;
        if status & 0x02 == 0 { return Err(Error::NotReady); }
        if status & 0x04 != 0 { return Err(Error::NotReady); }
        let _ = self.read_reg(INT_STATUS).await?;

        self.data_config().await?;
        
        Ok(())
    }

    /// Configure the sensor data output settings according to the config
    /// parameters passed at sensor creation. Runs automatically at init.
    pub async fn data_config(&mut self) -> Result<(), SPI::Error> {
        self.standby().await?;

        let mut regs = [0u8; 2];
        self.read_regs(OSR_CONFIG, &mut regs).await?;

        let osr = (regs[0] & 0x80) | ((self.config.p_en as u8) << 6) | ((self.config.osr_p as u8) << 3) | (self.config.osr_t as u8);
        let odr = (regs[1] & 0x83) | ((self.config.odr as u8) << 2);

        // burst write: address auto-increments 0x36 -> 0x37
        self.spi.write(&[OSR_CONFIG & 0x7F, osr, odr]).await?;

        // OSR_EFF bit 7 = odr_is_valid
        if self.read_reg(OSR_EFF).await? & 0x80 == 0 {
            return Err(Error::BadConfig);
        }

        self.set_power_mode(self.config.pwr).await?;

        Ok(())
    }

    /// Configure the data-ready interrupt.
    pub async fn int_config(&mut self, cfg: IntConfig) -> Result<(), SPI::Error> {
        let int_conf = self.read_reg(INT_CONFIG).await?;
        self.write_reg(INT_CONFIG, 0x00).await?;
        self.read_reg(INT_STATUS).await?;

        let int_conf = (int_conf & !0xF0) | cfg.mode as u8 | ((cfg.pol as u8) << 1) | ((cfg.drive as u8) << 2) | ((cfg.enable as u8) << 3);
        self.write_reg(INT_CONFIG, int_conf).await?;
        self.write_reg(INT_SOURCE, cfg.sources.bits()).await?;
        
        Ok(())
    }

    /// Set the sensor power mode; see `PowerMode` for explanation of each.
    pub async fn set_power_mode(&mut self, mode: PowerMode) -> Result<(), SPI::Error> {
        // From Bosch BMP5 implementations -- must set standby before switching
        // to a different power mode.
        self.standby().await?;
        
        if mode != PowerMode::Standby {
            let odr = self.read_reg(ODR_CONFIG).await?;
            self.write_reg(ODR_CONFIG, (odr & !0x03) | mode as u8).await?;
        }

        Ok(())
    }

    /// Read the temperature (in Celsius) and pressure (in Pa).
    pub async fn read(&mut self) -> Result<Val, SPI::Error> {
        let mut buf = [0u8; 6];
        self.read_regs(TEMP_DATA_XLSB, &mut buf).await?;

        Ok(Val {
            t: LittleEndian::read_i24(&buf[0..3]) as f32 / 65536.0,
            p: LittleEndian::read_u24(&buf[3..6]) as f32 / 64.0
        })
    }
}