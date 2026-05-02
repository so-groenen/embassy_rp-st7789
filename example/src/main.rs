#![no_std]
#![no_main]
 

use embassy_executor::Spawner;
use embassy_rp::spi::{self, Spi};
use embassy_rp::{bind_interrupts};
use embassy_rp::peripherals::{DMA_CH0}; 
use embassy_rp::gpio::{Level, Output};
use embassy_rp_st7789::st7789::{ST7789Display, Rotation, NoPin};
use embassy_rp_st7789::colors;
use embassy_time::Timer;
use defmt::*;
use {defmt_rtt as _, panic_probe as _};


bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

const NYAN_WIDTH: u16  = 220;
const NYAN_HEIGTH: u16 = 220;


#[embassy_executor::main]
async fn main(_spawner: Spawner) 
{
    let p = embassy_rp::init(Default::default());

    let mut config   = spi::Config::default();
    config.frequency = 50_000_000; 

    let spi = Spi::new_txonly(p.SPI0, p.PIN_6, p.PIN_7, p.DMA_CH0, Irqs, config);

    let cat_gif = include_bytes!("../gifs_to_bin/nyan_220x220_frames=17.bin"); 
    let reset_pin = NoPin::new();
    let dc_pin = Output::new(p.PIN_9, Level::High);
    let cs_pin = Output::new(p.PIN_5, Level::High);
    let bl_pin = Output::new(p.PIN_4, Level::High);

    let mut display = ST7789Display::new(reset_pin, dc_pin, cs_pin, bl_pin, spi, 240, 240, Rotation::Landscape).await
                        .expect("Critical: Could not init display!");
 
    display.fill(colors::BLACK).await.unwrap();
 
    let frames          = 17;
    let bytes_per_frame = cat_gif.len() / frames;
    let mut n           = 0;
    
    loop
    {
        let start = n*bytes_per_frame;
        let stop  = (n+1)*bytes_per_frame;
        if let Err(e) = display.draw_color_buf_raw(&cat_gif[start..stop], 10, 10, NYAN_WIDTH, NYAN_HEIGTH).await
        {
            error!("Could not send data to display: {:?}", e);
            return;
        }

        n = (n + 1)%frames;
        Timer::after_millis(50).await;  
    }
}

