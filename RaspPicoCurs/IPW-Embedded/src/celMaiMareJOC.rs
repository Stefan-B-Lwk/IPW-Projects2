#![no_std]
#![no_main]

use cyw43::new;
use cyw43_pio::PioSpi;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use export::display;
use heapless::{String, Vec};
use ipw_embedded::display::{self, SPIDeviceInterface};
use embassy_net::{Config, Stack, StackResources};
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};

use reqwless::client::{HttpClient, TlsConfig, TlsVerify};
use reqwless::request::{Method, RequestBody, RequestBuilder};
use serde::{Deserialize, Serialize};
use static_cell::StaticCell;
use serde_json_core;


use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Text, Alignment},
};

#[derive(Serialize,Deserialize,Debug)]
struct UserScore {
    username: String<128>,
    score: usize
}

use core::str::{from_utf8, FromStr};
use core::{cell::RefCell, iter::Take};
use embassy_rp::{bind_interrupts, peripherals::{DMA_CH0, PIO0}, pio::{InterruptHandler, Pio}, spi};
use embassy_rp::spi::Spi;
use rand::RngCore;
use embassy_rp::clocks::RoscRng;
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use embedded_hal_async::i2c::{Error, I2c as _};
use embassy_rp::peripherals::I2C0;
use cortex_m::peripheral;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{self, select4, Either, Either4};
use embassy_rp::{config, gpio::{Input, Level, Output, Pull}, pwm::Pwm};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::pwm::Config as ConfigPwm;
use embassy_futures::select::select;
use st7789::{Orientation, ST7789};


bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});

bind_interrupts!(struct Irqs2 {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn wifi_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

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

    let mut rng = RoscRng;

    //PT WIFI

    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    let pwr = Output::new(peripherals.PIN_23, Level::Low);
    let cs = Output::new(peripherals.PIN_25, Level::High);
    let mut pio = Pio::new(peripherals.PIO0, Irqs2);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, peripherals.PIN_24, peripherals.PIN_29, peripherals.DMA_CH0);


    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(wifi_task(runner)).unwrap();

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;    


    let config = Config::dhcpv4(Default::default());    

    let seed = 0x0123_4567_89ab_112f; // chosen by fair dice roll. guaranteed to be random.

// Init network stack
    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<5>::new()),
        seed,
    ));

    let _ = spawner.spawn(net_task(stack));


    loop {
        match control.join_wpa2("IPW-Users-2024", "ipwusers").await  {
            Ok(_) => {
                info!("Connected to el networuk");
                break;
            }
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }
    
    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
        info!("waiting for DHCP...");

    }
    info!("DHCP is now up!");

    loop {
        let mut rx_buffer = [0; 8192];

        let client_state = TcpClientState::<1, 1024, 1024>::new();
        let tcp_client = TcpClient::new(stack, &client_state);
        let dns_client = DnsSocket::new(stack);

        let mut http_client = HttpClient::new(&tcp_client, &dns_client);
        let url = "https://172.20.20.121:2024/new_score";
        info!("connecting to {}", &url);

        let mock_user = UserScore {
            score: 59,
            username: String::from_str("mgl").unwrap()
        };

        info!("bef ser");

        let serialized = serde_json_core::to_string::<_, 256>(&mock_user).unwrap();

        info!("after ser");

        let hdr = [("Content-Type","application/json")];
        let mut request = match http_client.request(Method::POST, &url).await {
            Ok(req) => req.body(serialized.as_bytes()).headers(&hdr),
            Err(e) => {
                error!("Failed to make HTTP request");
                return; // handle the error
            }
        };

        info!("Jello!");

        let response = match request.send(&mut rx_buffer).await {
            Ok(resp) => resp,
            Err(_e) => {
                error!("Failed to send HTTP request");
                return; // handle the error;
            }
        };

        info!("Jbye!");

        let body = match from_utf8(response.body().read_to_end().await.unwrap()) {
            Ok(b) => b,
            Err(_e) => {
                error!("Failed to read response body");
                return; // handle the error
            }
        };
        info!("Response body: {:?}", &body);

        let bytes = body.as_bytes();
        match serde_json_core::de::from_slice::<Vec<UserScore, 5>>(bytes) {
            Ok((output, _used)) => {
                info!("Scores:");
            }
            Err(_e) => {
                error!("Failed to parse response body");
                return; // handle the error
            }
        }

        loop {
            Timer::after(Duration::from_secs(5)).await;  
            info!("Merge in gol"); 
        }
    }

    
    

    //PT EEPROM

    let sda = peripherals.PIN_20;
    let scl = peripherals.PIN_21;

    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());

    let mut TARGET_ADDR: u16 = 0x50;

    //BUTOANE + LED

    let mut green = Output::new(peripherals.PIN_2, Level::Low);
    let mut red = Output::new(peripherals.PIN_6, Level::Low);

    let mut button1 = Input::new(peripherals.PIN_12, Pull::Up);
    let mut button2 = Input::new(peripherals.PIN_13, Pull::Up);
    let mut button3 = Input::new(peripherals.PIN_14, Pull::Up); //de verificat PIN la butoane
    let mut button4 = Input::new(peripherals.PIN_15, Pull::Up);


    let mut score = 0;

    let mut tx_buf = [0, 0, 0];

    let result = i2c.write(TARGET_ADDR, &tx_buf).await;

    match result {
        Ok(_) => {

        }
        Err(_) => {

        }
    }
    loop {
        Text::with_alignment(
            "Get ready!",
            Point::new(120, 120),
            style,
            Alignment::Center,
        )
        .draw(&mut display).unwrap();
        Timer::after_secs(2).await;
        display.clear(embedded_graphics::pixelcolor::Rgb565::BLACK).unwrap();
        Timer::after_secs(1).await;
        let letter1 = rng.next_u32() as u8 % 4;
        let letter2 = rng.next_u32() as u8 % 4;
        let letter3 = rng.next_u32() as u8 % 4;
        let letter4 = rng.next_u32() as u8 % 4;
        let mut chosen_letter: u8;
        if letter1 == 0 {
            Text::with_alignment(
                "A",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter1 == 1 {
            Text::with_alignment(
                "B",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter1 == 2 {
            Text::with_alignment(
                "X",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else {
            Text::with_alignment(
                "Y",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        }

        Timer::after_millis(500).await;
        display.clear(embedded_graphics::pixelcolor::Rgb565::BLACK).unwrap();
        Timer::after_millis(500).await;

        if letter2 == 0 {
            Text::with_alignment(
                "A",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter2 == 1 {
            Text::with_alignment(
                "B",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter2 == 2 {
            Text::with_alignment(
                "X",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else {
            Text::with_alignment(
                "Y",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        }

        Timer::after_millis(500).await;
        display.clear(embedded_graphics::pixelcolor::Rgb565::BLACK).unwrap();
        Timer::after_millis(500).await;

        if letter3 == 0 {
            Text::with_alignment(
                "A",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter3 == 1 {
            Text::with_alignment(
                "B",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter3 == 2 {
            Text::with_alignment(
                "X",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else {
            Text::with_alignment(
                "Y",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        }
        
        Timer::after_millis(500).await;
        display.clear(embedded_graphics::pixelcolor::Rgb565::BLACK).unwrap();
        Timer::after_millis(500).await;

        if letter4 == 0 {
            Text::with_alignment(
                "A",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter4 == 1 {
            Text::with_alignment(
                "B",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else if letter4 == 2 {
            Text::with_alignment(
                "X",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        } else {
            Text::with_alignment(
                "Y",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
        }
        
        Timer::after_millis(500).await;
        display.clear(embedded_graphics::pixelcolor::Rgb565::BLACK).unwrap();
        
        //let select = select4(button1.wait_for_falling_edge(), button2.wait_for_falling_edge(), button3.wait_for_falling_edge(), button4.wait_for_falling_edge());
        match select4(button1.wait_for_falling_edge(), button2.wait_for_falling_edge(), button3.wait_for_falling_edge(), button4.wait_for_falling_edge()).await{
            Either4::First(_) => {
                chosen_letter = 0;
            },
            Either4::Second(_) => {
                chosen_letter = 1;
            },
            Either4::Third(_) => {
                chosen_letter = 2;
            },
            Either4::Fourth(_) => {
                chosen_letter = 3;
            }
        }

        if chosen_letter != letter1 {
            red.set_high();
            Text::with_alignment(
                "Game over!",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();

            let mut rx_buf = [0];
            let mut tx_buf = [0,0];
            let read_result = i2c.write_read(TARGET_ADDR,&mut tx_buf, &mut rx_buf).await;
            match read_result {
                Ok(_) => {
                    info!("{}", rx_buf[0]);
                }
                Err(_) => {
                    
                }
            }
            if rx_buf[0] < score {
                let mut tx_buf = [0, 0, score];

                let result = i2c.write(TARGET_ADDR, &tx_buf).await;
                match result {
                    Ok(_) => {
                        info!("{}", tx_buf[2]);
                    }
                    Err(_) => {
                        
                    }
                }
            }
            Timer::after_secs(1).await;
            score = 0;
            red.set_low();
            continue;
        }

        match select4(button1.wait_for_falling_edge(), button2.wait_for_falling_edge(), button3.wait_for_falling_edge(), button4.wait_for_falling_edge()).await{
            Either4::First(_) => {
                chosen_letter = 0;
            },
            Either4::Second(_) => {
                chosen_letter = 1;
            },
            Either4::Third(_) => {
                chosen_letter = 2;
            },
            Either4::Fourth(_) => {
                chosen_letter = 3;
            }
        }

        if chosen_letter != letter2 {
            red.set_high();
            Text::with_alignment(
                "Game over!",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
            
            let mut rx_buf = [0];
            let mut tx_buf = [0,0];
            let read_result = i2c.write_read(TARGET_ADDR,&mut tx_buf, &mut rx_buf).await;
            match read_result {
                Ok(_) => {
                    info!("{}", rx_buf[0]);
                }
                Err(_) => {
                    
                }
            }
            if rx_buf[0] < score {
                let mut tx_buf = [0, 0, score];

                let result = i2c.write(TARGET_ADDR, &tx_buf).await;
                match result {
                    Ok(_) => {
                        info!("{}", tx_buf[2]);
                    }
                    Err(_) => {
                        
                    }
                }
            }
            Timer::after_secs(1).await;
            score = 0;
            red.set_low();
            continue;
        }

        match select4(button1.wait_for_falling_edge(), button2.wait_for_falling_edge(), button3.wait_for_falling_edge(), button4.wait_for_falling_edge()).await{
            Either4::First(_) => {
                chosen_letter = 0;
            },
            Either4::Second(_) => {
                chosen_letter = 1;
            },
            Either4::Third(_) => {
                chosen_letter = 2;
            },
            Either4::Fourth(_) => {
                chosen_letter = 3;
            }
        }

        if chosen_letter != letter3 {
            red.set_high();
            Text::with_alignment(
                "Game over!",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();
            
            let mut rx_buf = [0];
            let mut tx_buf = [0,0];
            let read_result = i2c.write_read(TARGET_ADDR,&mut tx_buf, &mut rx_buf).await;
            match read_result {
                Ok(_) => {
                    info!("{}", rx_buf[0]);
                }
                Err(_) => {
                    
                }
            }
            if rx_buf[0] < score {
                let mut tx_buf = [0, 0, score];

                let result = i2c.write(TARGET_ADDR, &tx_buf).await;
                match result {
                    Ok(_) => {
                        info!("{}", tx_buf[2]);
                    }
                    Err(_) => {
                        
                    }
                }
            }
            Timer::after_secs(1).await;
            score = 0;
            red.set_low();
            continue;
        }

        match select4(button1.wait_for_falling_edge(), button2.wait_for_falling_edge(), button3.wait_for_falling_edge(), button4.wait_for_falling_edge()).await{
            Either4::First(_) => {
                chosen_letter = 0;
            },
            Either4::Second(_) => {
                chosen_letter = 1;
            },
            Either4::Third(_) => {
                chosen_letter = 2;
            },
            Either4::Fourth(_) => {
                chosen_letter = 3;
            }
        }

        if chosen_letter != letter4 {
            red.set_high();
            Text::with_alignment(
                "Game over!",
                Point::new(120, 120),
                style,
                Alignment::Center,
            )
            .draw(&mut display).unwrap();

            let mut rx_buf = [0];
            let mut tx_buf = [0,0];
            let read_result = i2c.write_read(TARGET_ADDR,&mut tx_buf, &mut rx_buf).await;
            match read_result {
                Ok(_) => {
                    info!("{}", rx_buf[0]);
                }
                Err(_) => {
                    
                }
            }
            if rx_buf[0] < score {
                let mut tx_buf = [0, 0, score];

                let result = i2c.write(TARGET_ADDR, &tx_buf).await;
                match result {
                    Ok(_) => {
                        info!("{}", tx_buf[2]);
                    }
                    Err(_) => {
                        
                    }
                }
            }
            Timer::after_secs(1).await;
            score = 0;
            red.set_low();
            continue;
        }

        green.set_high();
        score += 1;
        Text::with_alignment(
            "Well done!",
            Point::new(120, 120),
            style,
            Alignment::Center,
        )
        .draw(&mut display).unwrap();
        Timer::after_secs(3).await;
        green.set_low();
        display.clear(embedded_graphics::pixelcolor::Rgb565::BLACK).unwrap();
    };

}