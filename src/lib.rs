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

#[derive(Clone, Copy, Debug)]
pub struct BmpConfig {
    pub osr_t: OSR,
    pub osr_p: OSR,
    pub odr: ODR,
}

impl Default for BmpConfig {
    fn default() -> Self {
        Self {
            osr_t: OSR::X2,
            osr_p: OSR::X16,
            odr: ODR::Hz50,
        }
    }
}

pub struct Bmp5xx<SPI: SpiDevice, D: DelayNs> {
    spi: SPI,
    delay: D,
    config: BmpConfig,
}

impl<SPI: SpiDevice, D: DelayNs> Bmp5xx<SPI, D> {
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

    async fn enter_standby(&mut self) -> Result<(), SPI::Error> {
        let odr = self.read_reg(ODR_CONFIG).await?;
        self.write_reg(ODR_CONFIG, (odr & !0x03) | 0x80).await?;
        self.delay.delay_us(2500).await;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), SPI::Error> {
        self.write_reg(CMD, 0xB6).await?;
        self.delay.delay_ms(6).await;
        let _ = self.read_reg(CHIP_ID).await?;

        match self.read_reg(CHIP_ID).await? {
            ADDR_58X | ADDR_585 => Ok(()),
            id => Err(Error::ChipId(id)),
        }
    }

    pub async fn init(&mut self) -> Result<(), SPI::Error> {
        self.reset().await?;
        let status = self.read_reg(STATUS).await?;
        if status & 0x02 == 0 { return Err(Error::NotReady); }
        if status & 0x04 != 0 { return Err(Error::NotReady); }
        let _ = self.read_reg(INT_STATUS).await?;

        self.configure().await?;
        
        Ok(())
    }

    pub async fn configure(&mut self) -> Result<(), SPI::Error> {
        self.enter_standby().await?;

        let mut regs = [0u8; 2];
        self.read_regs(OSR_CONFIG, &mut regs).await?;

        let osr = (regs[0] & 0x80) | (1 << 6) | ((self.config.osr_p as u8) << 3) | (self.config.osr_t as u8);
        let odr = (regs[1] & 0x83) | ((self.config.odr as u8) << 2);

        // burst write: address auto-increments 0x36 -> 0x37
        self.spi.write(&[OSR_CONFIG & 0x7F, osr, odr]).await?;

        // OSR_EFF bit 7 = odr_is_valid
        if self.read_reg(OSR_EFF).await? & 0x80 == 0 {
            return Err(Error::BadConfig);
        }

        Ok(())
    }

    pub async fn set_power_mode(&mut self, mode: PowerMode) -> Result<(), SPI::Error> {
        let odr = self.read_reg(ODR_CONFIG).await?;
        self.write_reg(ODR_CONFIG, (odr & !0x03) | mode as u8).await
    }
}