#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{config, gpio::{Level, Output}, pwm::Pwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::pwm::Config as ConfigPwm;



#[embassy_executor::main]
async fn main(__spawner: Spawner){
    let peripherals= embassy_rp::init(Default::default());

    let mut config1: ConfigPwm = Default::default();
    let mut config2: ConfigPwm = Default::default();

    config1.top = 255;
    config1.compare_a = 250;
    config1.compare_b = 250;

    config2.top = 255;
    config2.compare_a = 250;

    let mut pwm_red_green = Pwm::new_output_ab(
        peripherals.PWM_SLICE0,
        peripherals.PIN_0,
        peripherals.PIN_1,
        config1.clone()
    );
    let mut pwm_blue = Pwm::new_output_a(
        peripherals.PWM_SLICE1,
        peripherals.PIN_2,
        config2.clone()
    );

    loop {
        config1.compare_a -= 25;
        config1.compare_b -= 25;
        config2.compare_a -= 25;
        pwm_red_green.set_config(&config1);
        pwm_blue.set_config(&config2);
        Timer::after_secs(1).await;
        if config1.compare_a <= 0 {
            config1.compare_a = 250;
            config1.compare_b = 250;
            config2.compare_a = 250;
        }
    }
}