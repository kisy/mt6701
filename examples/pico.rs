#![no_std]
#![no_main]

use mt6701::AngleSensorTrait;
use panic_halt as _;
use rp2040_hal as hal;

use fugit::RateExtU32;
use hal::clocks::Clock;

use hal::spi::Spi;

use hal::pac;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let spi_sclk = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio16.into_function::<hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();

    let spi_cs = pins.gpio17.into_push_pull_output();

    let spi = Spi::<_, _, _, 16>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1.MHz(),
        &embedded_hal::spi::MODE_0,
    );

    let mut sensor = mt6701::MT6701Spi::new(spi, spi_cs);
    sensor.init().unwrap();

    loop {
        sensor.update(timer.get_counter().ticks()).unwrap();
        let angle = sensor.get_angle();
        let turns = sensor.get_turns();
        let position = sensor.get_position();
        let velocity = sensor.get_velocity();
    }
}
