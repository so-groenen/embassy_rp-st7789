#![no_std]
#![no_main]
 

use embassy_executor::Spawner;
use embassy_rp::spi::{self, Spi};
use embassy_rp::{bind_interrupts, clocks};
use embassy_rp::peripherals::{DMA_CH0}; 
use embassy_rp::gpio::{Level, Output};
use embassy_rp_st7789::st7789::{ST7789Display, Rotation, NoPin};
use embassy_rp_st7789::colors;
use embassy_time::{Duration, Ticker};
use defmt::*;
use {defmt_rtt as _, panic_probe as _};


bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

const NYAN_WIDTH: u16  = 220;
const NYAN_HEIGTH: u16 = 220;

const DISPLAY_WIDTH: u16  = 240;
const DISPLAY_HEIGTH: u16 = 240;

const X: u16 = (DISPLAY_WIDTH - NYAN_WIDTH)/2;
const Y: u16 = (DISPLAY_HEIGTH - NYAN_HEIGTH)/2;


#[embassy_executor::main]
async fn main(_spawner: Spawner) 
{
    let p = embassy_rp::init(Default::default());

    let mut config   = spi::Config::default();
    config.frequency = clocks::clk_sys_freq()/4; // 155/4 = 37.5Mhz; 
    config.polarity  = spi::Polarity::IdleHigh;
    config.phase     = spi::Phase::CaptureOnSecondTransition;
    
    // SPI Display wire color coding suggestion: 
    // vcc: red
    // gnd: black
    // mosi: green
    // sclk: orange
    // dc: violent
    // cs: yellow
    // bl: blue

    let spi = Spi::new_txonly(p.SPI0, p.PIN_2, p.PIN_3, p.DMA_CH0, Irqs, config);

    let cat_gif = include_bytes!("../gifs_to_bin/nyan_220x220_frames=17.bin"); 
    let reset_pin = NoPin::new();
    let dc_pin = Output::new(p.PIN_4, Level::High); 
    let cs_pin = Output::new(p.PIN_5, Level::High); 
    let bl_pin = Output::new(p.PIN_6, Level::High); 

    let mut display = ST7789Display::new(reset_pin, dc_pin, cs_pin, bl_pin, spi, DISPLAY_WIDTH, DISPLAY_HEIGTH, Rotation::Landscape).await
                        .expect("Critical: Could not init display!");
    
    display.fill(colors::BLACK).await.expect("Could not fill display");
 
    let frames          = 17;
    let bytes_per_frame = cat_gif.len() / frames;
    let mut n           = 0;
    let mut ticker      = Ticker::every(Duration::from_millis(33));

    loop
    {
        let start = n*bytes_per_frame;
        let stop  = (n+1)*bytes_per_frame;
        if let Err(e) = display.draw_color_buf_raw(&cat_gif[start..stop], X, Y, NYAN_WIDTH, NYAN_HEIGTH).await
        {
            error!("Could not send data to display: {:?}", e);
            return;
        }

        n = (n + 1)%frames;
        ticker.next().await;
    }
}

