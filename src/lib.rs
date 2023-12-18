#![no_std]

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

use core::f32::consts::PI;
use core::fmt::Debug;
use libm::fabsf;
use nb;

const _2PI: f32 = PI * 2.0;

#[derive(Debug)]
pub enum MT6701Error {
    SpiError,
    CsnError,
}

pub trait AngleSensorTrait {
    type Error: Debug;

    fn init(&mut self) -> nb::Result<(), Self::Error>;
    fn read_raw_angle(&mut self) -> nb::Result<u16, Self::Error>;
    fn update(&mut self, now_us: u64) -> nb::Result<(), Self::Error>;
    fn get_angle(&mut self) -> f32;
    fn get_turns(&mut self) -> i64;
    fn get_position(&mut self) -> f64;
    fn get_velocity(&mut self) -> f32;
}

#[derive(Debug)]
pub struct MT6701Spi<SPI, CS> {
    spi: SPI,
    cs: CS,
    turns: i64,
    angle: f32,
    angle_prev: f32,
    velocity: f32,
    prev_ns: u64,
}

impl<SPI, CS> MT6701Spi<SPI, CS>
where
    SPI: Transfer<u16>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        MT6701Spi {
            spi,
            cs,
            turns: 0,
            angle_prev: 0.0,
            angle: 0.0,
            velocity: 0.0,
            prev_ns: 0,
        }
    }

    fn cal_velocity(&mut self, now_ns: u64) {
        let mut ts = (now_ns - self.prev_ns) as f32 * 1e-6;
        if ts < 0.0 {
            ts = 1e-3;
        }

        self.velocity = (self.angle - self.angle_prev) / ts;
    }
}

impl<SPI, CS> AngleSensorTrait for MT6701Spi<SPI, CS>
where
    SPI: Transfer<u16>,
    CS: OutputPin,
{
    type Error = MT6701Error;

    fn init(&mut self) -> nb::Result<(), MT6701Error> {
        self.cs.set_high().map_err(|_| MT6701Error::CsnError)?;
        Ok(())
    }

    fn read_raw_angle(&mut self) -> nb::Result<u16, MT6701Error> {
        let mut buffer: [u16; 1] = [0];

        self.cs.set_low().map_err(|_| MT6701Error::CsnError)?;

        self.spi
            .transfer(&mut buffer)
            .map_err(|_| MT6701Error::SpiError)?;

        self.cs.set_high().map_err(|_| MT6701Error::CsnError)?;

        Ok((buffer[0] >> 1) & 0x3FFF)
    }

    fn update(&mut self, ts_ns: u64) -> nb::Result<(), MT6701Error> {
        let raw_angle = self.read_raw_angle()?;

        self.angle = (raw_angle as f32 / 16384_f32) * _2PI;
        let move_angle = self.angle - self.angle_prev;

        if fabsf(move_angle) > (0.8 * _2PI) {
            self.turns += if move_angle > 0.0 { -1 } else { 1 };
        }

        if ts_ns > 0 {
            self.cal_velocity(ts_ns);
        }

        self.angle_prev = self.angle;
        self.prev_ns = ts_ns;

        Ok(())
    }

    fn get_angle(&mut self) -> f32 {
        self.angle
    }

    fn get_turns(&mut self) -> i64 {
        self.turns
    }

    fn get_position(&mut self) -> f64 {
        self.turns as f64 * _2PI as f64 + self.angle as f64
    }

    fn get_velocity(&mut self) -> f32 {
        self.velocity
    }
}
