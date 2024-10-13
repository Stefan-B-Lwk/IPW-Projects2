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

    let mut TARGET_ADDR: u16 = 0x50;
    

    let mut rng = RoscRng;

    let mut i = 0;
    let mut config1: ConfigPwm = Default::default();
    let mut config2: ConfigPwm = Default::default();
    let mut config3: ConfigPwm = Default::default();

    config1.top = 255;
    config1.compare_a = 0;
  

    config2.top = 255;
    config2.compare_a = 0;

    config3.top = 255;
    config3.compare_a = 0;

    let mut pwm_red = Pwm::new_output_a(
        peripherals.PWM_SLICE0,
        peripherals.PIN_0,
        config1.clone()
    );

    let mut pwm_green = Pwm::new_output_a(
        peripherals.PWM_SLICE1,
        peripherals.PIN_2,
        config1.clone()
    );

    let mut pwm_blue = Pwm::new_output_a(
        peripherals.PWM_SLICE2,
        peripherals.PIN_4,
        config2.clone()
    );
    
    loop {
        let mut my_u32 = rng.next_u32();
        let mut my_u8 = my_u32 as u8;
        let mut tx_buf = [0,i, my_u8];

        let result = i2c.write(TARGET_ADDR, &tx_buf).await;
    
        match result {
            Ok(_) => {
                info!("write:{} si {} cu nr {} si {}",TARGET_ADDR,tx_buf[0],tx_buf[1],tx_buf[2]);
                info!("merge");
            }
            Err(_) => {
                info!("{} si {}",TARGET_ADDR,tx_buf[0]);
                info!("Nu a mers");
            }
        }
        Timer::after_millis(10).await;

        i += 1;
        if i == 30 {
            break;
        }
    }
    TARGET_ADDR = 0x50;
    i = 0x00;
    while i < 30 {
        let mut rx_buf = [0,0, 0];
        let mut tx_buf = [0,i];
        let read_result = i2c.write_read(TARGET_ADDR,&mut tx_buf, &mut rx_buf).await;
        match read_result {
            Ok(_) => {
                info!("read:{} si {} si {} si {}",i ,rx_buf[0],rx_buf[1],rx_buf[2]);
            }
            Err(_) => {
                info!("err: write:{} si {} si {} si {}",TARGET_ADDR,rx_buf[0],rx_buf[1],rx_buf[2]);
            }
        }
        config1.compare_a = config1.top - rx_buf[0] as u16;
        config2.compare_a = config2.top - rx_buf[1] as u16;
        config3.compare_a = config3.top - rx_buf[2] as u16;
        pwm_red.set_config(&config1);
        pwm_green.set_config(&config2);
        pwm_blue.set_config(&config3);
        i += 3;
        Timer::after_secs(1).await;
        if i == 30 {
            i = 0;
        }
    }
}