#![no_std]
#![no_main]


use core::iter::Take;

use embassy_rp::bind_interrupts;
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


bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner){
    let peripherals = embassy_rp::init(Default::default());

    
    let sda = peripherals.PIN_20;
    let scl = peripherals.PIN_21;

    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());

    let mut TARGET_ADDR: u16 = 0x00;

    let tx_buf = [20,30];
    let result = i2c.write(TARGET_ADDR, &tx_buf).await;

    match result {
        Ok(_) => {
            info!("{}",TARGET_ADDR);
        }
        Err(_) => {
            info!("Error!");
        }
    }

} 