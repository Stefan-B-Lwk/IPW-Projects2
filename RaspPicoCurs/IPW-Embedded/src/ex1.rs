#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(__spawner: Spawner){
    let peripherals = embassy_rp::init(Default::default());

    let mut red = Output::new(peripherals.PIN_5, Level::High);
    let mut green = Output::new(peripherals.PIN_4, Level::Low);

    loop {
        info!("red!");
        red.set_high();
        green.set_low();
        Timer::after_secs(1).await;

        info!("green!");
        red.set_low();
        green.set_high();
        Timer::after_secs(1).await;
    }

}