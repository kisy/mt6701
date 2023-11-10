use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

use core::f32::consts::PI;
use libm::fabsf;
use nb;

const _2PI: f32 = PI * 2.0;

#[derive(Debug, Default)]
pub struct MT6701SSI<SPI, CS>
 {
    spi: SPI,
    cs: CS,
    turns: i64,
    angle_single_prev: f32,
    angle_single: f32,
}

impl<SPI, CS> MT6701SSI<SPI, CS>
where
    SPI: Transfer<u16>,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        MT6701SSI {
            spi,
            cs,
            turns: 0,
            angle_single_prev: 0.0,
            angle_single: 0.0,
        }
    }

    pub fn init(&mut self) -> nb::Result<(), CS::Error> {
        let _ = self.cs.set_high()?;
        Ok(())
    }

    pub fn read_raw_angle(&mut self) -> nb::Result<u16, SPI::Error> {
        let mut buffer: [u16; 1] = [0];

        let _ = self.cs.set_low();

        self.spi.transfer(&mut buffer)?;

        let _ = self.cs.set_high();

        Ok((buffer[0] >> 1) & 0x3FFF)
    }

    pub fn update(&mut self) -> nb::Result<(), SPI::Error> {
        let raw_angle = self.read_raw_angle()?;

        self.angle_single = (raw_angle as f32 / 16384_f32) * _2PI;
        let move_angle = self.angle_single - self.angle_single_prev;

        if fabsf(move_angle) > (0.8 * _2PI) {
            self.turns += if move_angle > 0.0 { -1 } else { 1 };
        }
        self.angle_single_prev = self.angle_single;

        Ok(())
    }

    pub fn get_angle_single(&mut self) -> f32 {
        self.angle_single
    }

    pub fn get_turns(&mut self) -> i64 {
        self.turns
    }

    pub fn get_angle(&mut self) -> f64 {
        self.turns as f64 * _2PI as f64 + self.angle_single as f64
    }
}
