#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;
use defmt::*;
use dotenv_hex::fetch_hex_env;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals::{PIN_25, SPI0};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_rp::Peripheral;
use embassy_sync::blocking_mutex::{CriticalSectionMutex, Mutex};
use embassy_time::{block_for, Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{epd2in66b::*, prelude::*};
use graphics::DisplayContent;
use lora_phy::iv::GenericSx126xInterfaceVariant;
use lorawan_device::{AppEui, AppKey, DevEui, JoinMode};
use {defmt_rtt as _, panic_probe as _};

mod display;
mod graphics;
mod lora;

use lora::config_lora;

#[embassy_executor::task]
async fn blinky_task(led_pin: embassy_rp::PeripheralRef<'static, PIN_25>) {
    let mut led = Output::new(led_pin, Level::Low);
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    unwrap!(spawner.spawn(blinky_task(p.PIN_25.into_ref())));

    let spi_mutex: CriticalSectionMutex<RefCell<Spi<SPI0, Async>>>;
    let mut spi_device;
    let mut e_paper = {
        let spi0 = {
            let clk = p.PIN_6;
            let mosi = p.PIN_7;
            let miso = p.PIN_4;
            let mut spi = Spi::new(
                p.SPI0,
                clk,
                mosi,
                miso,
                p.DMA_CH0,
                p.DMA_CH1,
                Config::default(),
            );
            spi.set_frequency(20_000_000u32); // The SSD1675B docs say 20MHz max
            spi
        };
        let chip_select = Output::new(p.PIN_5, Level::Low);
        let is_busy = Input::new(p.PIN_0, Pull::None);
        let data_or_command = Output::new(p.PIN_8, Level::Low);
        let reset = Output::new(p.PIN_1, Level::Low);

        spi_mutex = Mutex::new(RefCell::new(spi0));
        spi_device = SpiDevice::new(&spi_mutex, chip_select);

        let e_paper = Epd2in66b::new(
            &mut spi_device,
            is_busy,
            data_or_command,
            reset,
            &mut Delay,
            Some(100),
        )
        .unwrap();
        e_paper
    };

    let mut lora_device = {
        let nss = Output::new(p.PIN_3.degrade(), Level::High);
        let reset = Output::new(p.PIN_15.degrade(), Level::High);
        let dio1 = Input::new(p.PIN_20.degrade(), Pull::None);
        let busy = Input::new(p.PIN_2.degrade(), Pull::None);

        let spi1 = {
            let miso = p.PIN_12;
            let mosi = p.PIN_11;
            let clk = p.PIN_10;
            Spi::new(
                p.SPI1,
                clk,
                mosi,
                miso,
                p.DMA_CH2,
                p.DMA_CH3,
                Config::default(),
            )
        };
        let spi = ExclusiveDevice::new(spi1, nss, Delay);

        let iv = GenericSx126xInterfaceVariant::new(reset, dio1, busy, None, None).unwrap();
        match config_lora(spi, iv).await {
            Ok(device) => device,
            Err(err) => {
                defmt::error!("Failed to configure LoRA device: {}", err);
                return;
            }
        }
    };

    display::display_frame(
        &DisplayContent::from("", "Connecting...").unwrap(),
        &mut e_paper,
        &mut spi_device,
        &mut Delay,
    );

    let mut recv_buffer: [u8; 32] = [0; 32];
    loop {
        defmt::info!("Joining LoRaWAN network");
        match lora_device
            .join(&JoinMode::OTAA {
                deveui: DevEui::from(fetch_hex_env!("DEV_EUI")),
                appeui: AppEui::from(fetch_hex_env!("APP_EUI")),
                appkey: AppKey::from(fetch_hex_env!("APP_KEY")),
            })
            .await
        {
            Ok(response) => {
                match response {
                    lorawan_device::async_device::JoinResponse::JoinSuccess => {
                        defmt::info!("LoRaWAN network joined");
                        loop {
                            match lora_device.send(&[], 1, true).await {
                                Ok(resp) => {
                                    match resp {
                                    lorawan_device::async_device::SendResponse::DownlinkReceived(
                                        sz,
                                    ) => {
                                        defmt::info!(
                                            "Sent message successfully; received {} bytes",
                                            sz
                                        );
                                        match &sz {
                                            1 => {
                                                let text = match &recv_buffer[0] {
                                                    0x00 => "POTATO TOMATO",
                                                    0x01 => "example text 9000",
                                                    _ => "this is a PLACEHOLDER",
                                                };
                                                defmt::info!("Displaying text: {}", text);
                                                display::display_frame(
                                                    &DisplayContent::from(text, "").unwrap(),
                                                    &mut e_paper,
                                                    &mut spi_device,
                                                    &mut Delay,
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                    lorawan_device::async_device::SendResponse::SessionExpired => {}
                                    lorawan_device::async_device::SendResponse::NoAck => {}
                                    lorawan_device::async_device::SendResponse::RxComplete => {}
                                }

                                    // Wait politely before resend
                                    block_for(Duration::from_secs(30));
                                }
                                Err(err) => {
                                    info!("Send error = {}", err);
                                }
                            }
                        }
                    }
                    lorawan_device::async_device::JoinResponse::NoJoinAccept => {
                        defmt::error!("LoRaWAN join failed");
                    }
                }
            }
            Err(err) => {
                info!("Radio error = {}", err);
            }
        };
    }
}
