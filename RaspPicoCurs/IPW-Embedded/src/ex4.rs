#![no_std]
#![no_main]

use cyw43::new;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use export::display;
use ipw_embedded::display::{self, SPIDeviceInterface};

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Text, Alignment},
};
use core::{cell::RefCell, iter::Take};
use embassy_rp::{bind_interrupts, spi};
use embassy_rp::spi::Spi;
use rand::RngCore;
use embassy_rp::clocks::RoscRng;
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use embedded_hal_async::i2c::{Error, I2c as _};
use embassy_rp::peripherals::I2C0;
use cortex_m::peripheral;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{self, Either};
use embassy_rp::{config, gpio::{Input, Level, Output, Pull}, pwm::Pwm};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::pwm::Config as ConfigPwm;
use embassy_futures::select::select;
use st7789::{Orientation, ST7789};

#[embassy_executor::main]
async fn main(spawner: Spawner){
    let peripherals = embassy_rp::init(Default::default());

    
    let miso = peripherals.PIN_4;
    let mosi = peripherals.PIN_19;
    let clk = peripherals.PIN_18;
    let display_cs = peripherals.PIN_17;
    let dc = peripherals.PIN_16;
    let rst = peripherals.PIN_0;

    let mut config = embassy_rp::spi::Config::default();
    config.frequency = 64_000_000;
    config.phase = embassy_rp::spi::Phase::CaptureOnFirstTransition;
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;

    let spi: embassy_rp::spi::Spi<'_, _, embassy_rp::spi::Blocking> =
        embassy_rp::spi::Spi::new_blocking(peripherals.SPI0, clk, mosi, miso, config.clone());

    let spi_bus: embassy_sync::blocking_mutex::Mutex<
        embassy_sync::blocking_mutex::raw::NoopRawMutex,
        _,
    > = embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(spi));


    let display_spi = embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig::new(
        &spi_bus, embassy_rp::gpio::Output::new(display_cs,embassy_rp::gpio::Level::High),config,);
    
    let dc = embassy_rp::gpio::Output::new(dc,embassy_rp::gpio::Level::Low);
    let rst = embassy_rp::gpio::Output::new(rst,embassy_rp::gpio::Level::Low);
    // dcx: 0 = command, 1 = data

    // Enable LCD backlight
    //let _bl = Output::new(bl, Level::High);

    // display interface abstraction from SPI and DC
    let di = display::SPIDeviceInterface::new(display_spi, dc);
    
    // create driver
    let mut display = st7789::ST7789::new(di, rst, 240, 240);

    // initialize
    display.init(&mut embassy_time::Delay).unwrap();

    // set default orientation
    display.set_orientation(st7789::Orientation::Portrait).unwrap();

    display.clear(embedded_graphics::pixelcolor::Rgb565::BLACK).unwrap();

    //PT STIL TEXT ETC

    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

    Text::with_alignment(
        "Hello,IPW!",
        Point::new(120, 120),
        style,
        Alignment::Center,
    )
    .draw(&mut display).unwrap();
    loop{
    Timer::after_secs(1).await;
    };
}