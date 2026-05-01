#![no_std]
#![no_main]
 

use embassy_executor::Spawner;
use embassy_rp::spi::{self, Spi};
use embassy_rp::{bind_interrupts};
use embassy_rp::peripherals::{DMA_CH0}; 
use embassy_rp::gpio::{Level, Output};
use embassy_rp_st7789::st7789::{ST7789Display, Rotation, NoPin};
use embassy_time::Timer;

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

    let spi = Spi::new_txonly(p.SPI0, p.PIN_18, p.PIN_19, p.DMA_CH0, Irqs, config);
 
    let cat_gif = include_bytes!("../../../gifs_to_bin/nyan_220x220_frames=17.bin"); 
    let reset_pin = NoPin::new();
    let dc_pin    = Output::new(p.PIN_20, Level::High);
    let cs_pin    = Output::new(p.PIN_17, Level::High);
    let bl_pin    = Output::new(p.PIN_22, Level::High);
 
    let mut display = ST7789Display::new(reset_pin, dc_pin, cs_pin, bl_pin, spi, 240, 240, Rotation::Landscape).await.unwrap();
 
    display.fill(0).await.unwrap();
 
    let frames          = 17;
    let bytes_per_frame = cat_gif.len() / frames;
    let mut n           = 0;
    
    loop
    {
        let start = n*bytes_per_frame;
        let stop  = (n+1)*bytes_per_frame;
        display.draw_color_buf_raw(&cat_gif[start..stop], 0, 0, NYAN_WIDTH, NYAN_HEIGTH).await.unwrap();

        n = (n + 1)%frames;
        Timer::after_millis(50).await;  
    }
}

