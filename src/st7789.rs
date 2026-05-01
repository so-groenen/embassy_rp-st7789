// ADAPTED FROM https://github.com/ri-char/rp2040-st7789
#![allow(dead_code)]




use core::mem;
use core::ops::BitOr;
use embassy_rp::gpio::{Output};
use embassy_rp::spi::{self, Async, Spi};

use embassy_time::{Timer};
use crate::font::Font;
 
 
#[repr(u8)]
#[allow(dead_code)]
pub enum Rotation {
    Portrait = 0,
    Landscape = 0x60,
    InvertedPortrait = 0xC0,
    InvertedLandscape = 0xA0,
}

#[repr(u8)]
#[allow(dead_code)]
pub enum ColorMode {
    ColorMode65k = 0x50,
    ColorMode262k = 0x60,
    ColorMode12bit = 0x03,
    ColorMode16bit = 0x05,
    ColorMode18bit = 0x06,
    ColorMode16m = 0x07,
}

impl BitOr for ColorMode {
    type Output = u8;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}


#[repr(u8)]
pub enum Command {
    Nop = 0x00,
    Swreset = 0x01,
    Rddid = 0x04,
    Rddst = 0x09,
    Slpin = 0x10,
    Slpout = 0x11,
    Ptlon = 0x12,
    Noron = 0x13,
    Invoff = 0x20,
    Invon = 0x21,
    Dispoff = 0x28,
    Dispon = 0x29,
    Caset = 0x2A,
    Raset = 0x2B,
    Ramwr = 0x2C,
    Ramrd = 0x2E,
    Ptlar = 0x30,
    Vscrdef = 0x33,
    Colmod = 0x3A,
    Madctl = 0x36,
    Vscsad = 0x37,
}

pub trait OptionalOutput
{
    /// Set the output pin to the specified value.
    fn set(&mut self, value: bool);
    /// Return whether the output pin is none.
    fn is_none(&self) -> bool;
}
#[derive(Clone,Copy)]
pub struct NoPin{}
impl NoPin
{
    pub fn new() -> Self 
    {
        Self{}
    }
}

 

impl OptionalOutput for NoPin
{
    fn set(&mut self, _value: bool) 
    {
    }

    fn is_none(&self) -> bool {
        true
    }
}
 

impl<'a> OptionalOutput for Output<'a>
{
    fn set(&mut self, value: bool) 
    {
        if value 
        {
            self.set_high();
        }
        else 
        {
            self.set_low();
        }
    }

    fn is_none(&self) -> bool {
        false
    }
}


pub struct ST7789Display<'a,
    K: OptionalOutput,
    // L: PinId,
    M: OptionalOutput,
    N: OptionalOutput,
    T: spi::Instance
> {
    /// Reset
    reset_pin: K,
    /// Data/Command
    dc_pin: Output<'a>,
    /// Chip select
    cs_pin: M,
    /// Backlight
    bl_pin: N,
    /// SPI
    spi: Spi<'a, T, Async>,
    /// the width of the display in pixels
    width: u16,
    /// the height of the display in pixels
    height: u16,
}
const BUFFER_SIZE: u16 = 4096;

impl<'a, K: OptionalOutput, M: OptionalOutput, N: OptionalOutput, T: spi::Instance>  ST7789Display<'a,K,M,N,T>
{
    pub async fn new(
        // Reset
        reset_pin: K,
        // Data/Command
        dc_pin: Output<'a>,
        // Chip select
        cs_pin: M,
        // Backlight
        bl_pin: N,
        // SPI
        spi: Spi<'a, T, Async>,
        width: u16,
        height: u16,
        rotation: Rotation
    ) -> Result<Self,spi::Error>
    {
        let mut i = Self {
            reset_pin,
            dc_pin,
            cs_pin,
            bl_pin,
            spi,
            width,
            height,
        };

        i.hard_reset().await;
        i.soft_reset().await?;
        i.set_sleep_mode(false).await?;
        i.set_color_mode(ColorMode::ColorMode65k | ColorMode::ColorMode16bit).await?;
        Timer::after_millis(50).await;  
        i.set_rotation(rotation).await?;
        i.set_inversion_mode(true).await?;
        Timer::after_millis(10).await;  
        i.send_command(Command::Noron).await?;
        Timer::after_millis(10).await;  
        i.fill(0).await?;
        i.send_command(Command::Dispon).await?;
        i.set_backlight(true);
        Timer::after_millis(500).await;  
        Ok(i)
    }
    pub fn set_backlight(&mut self, val: bool)
    {
        self.bl_pin.set(val);
    }
    pub async fn hard_reset(&mut self)
    {
        if self.reset_pin.is_none()
        {
            return;
        }
        self.cs_pin.set(false);
        self.reset_pin.set(true);
        Timer::after_millis(50).await;
        self.reset_pin.set(false);
        Timer::after_millis(50).await;  
        self.reset_pin.set(true);
        Timer::after_millis(150).await;
        self.cs_pin.set(true);
    }
    /// Write Spi command to the display.
    pub async fn send_command(&mut self, command: Command) -> Result<(),spi::Error>
    {
        self.cs_pin.set(false);
        self.dc_pin.set_low();
        self.spi.write(&[command as u8]).await?;
        self.cs_pin.set(true);
        Ok(())
    }
    #[inline(always)]
    pub async fn send_command_no_cs(&mut self, command: Command) -> Result<(),spi::Error>
    {
        self.dc_pin.set_low();
        self.spi.write(&[command as u8]).await?;
        Ok(())
    }
    /// Write Spi data to the display.
    pub async fn send_data(&mut self, data: &[u8]) -> Result<(),spi::Error>
    {
        self.cs_pin.set(false);
        self.dc_pin.set_high();
        self.spi.write(data).await?;
        self.cs_pin.set(true);
        Ok(())
    }
    
    pub async fn send_command_data(&mut self, command: Command, data: &[u8]) -> Result<(),spi::Error>
    {
        self.cs_pin.set(false);
        self.dc_pin.set_low();
        self.spi.write(&[command as u8]).await?;
        self.dc_pin.set_high();
        self.spi.write(data).await?;
        self.cs_pin.set(true);
        Ok(())
    }
    pub async fn send_command_data_no_cs(&mut self, command: Command, data: &[u8]) -> Result<(),spi::Error>
    {
        self.dc_pin.set_low();
        self.spi.write(&[command as u8]).await?;
        self.dc_pin.set_high();
        self.spi.write(data).await?;
        Ok(())
    }

    /// Reset by sending a software reset command.
    /// It will be called automatically when created.
    /// It is usually called after `hard_reset`.
    pub async fn soft_reset(&mut self) -> Result<(),spi::Error>
    {
        self.send_command(Command::Swreset).await?;
        Timer::after_millis(150).await;
        Ok(())
    }
    /// Set the display to sleep mode.
    pub async fn set_sleep_mode(&mut self, value: bool) -> Result<(),spi::Error>
    {
        if value 
        {
            self.send_command(Command::Slpin).await?;
        }
        else 
        {
            self.send_command(Command::Slpout).await?;
        }
        Ok(())
    }

    /// Set the display to inversion mode.
    pub async fn set_inversion_mode(&mut self, value: bool)  -> Result<(),spi::Error>
    {
        if value 
        {
            self.send_command(Command::Invon).await?;
        } 
        else
        {
            self.send_command(Command::Invoff).await?;
        }
        Ok(())
    }

        pub async fn set_color_mode(&mut self, mode: u8) -> Result<(),spi::Error>  {
        self.send_command_data(
            Command::Colmod,
            &[mode]).await?;
        // self.send_data(&[mode]).await?;
        Ok(())
    }

    /// Set the display to rotation mode.
    pub async fn set_rotation(&mut self, rotation: Rotation) -> Result<(),spi::Error> {
        self.send_command_data(
            Command::Madctl,
            &[rotation as u8]).await?;
        // self.send_data(&[rotation as u8]).await?;
        Ok(())
    }

    //  TODO: THis function is only called once in set_window; ==> remove self.cs_pin.set(false)/(true) and keep it outside

    /// Select columns.
    async fn set_columns(&mut self, start: u16, end: u16) -> Result<(),spi::Error> {
        assert!(start <= end && end <= self.width);
        self.send_command_data( 
            Command::Caset,
            &[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]
        ).await?;
        // self.send_data(&[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]).await?;
        Ok(())
    }

    /// Select rows.
    async fn set_rows(&mut self, start: u16, end: u16) -> Result<(),spi::Error> {
        assert!(start <= end && end <= self.height);
        self.send_command_data(
            Command::Raset,
            &[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]).await?;
        // self.send_data(&[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]).await?;
        Ok(())
    }
    #[inline(always)]
    async fn set_columns_no_cs(&mut self, start: u16, end: u16) -> Result<(),spi::Error> {
        assert!(start <= end && end <= self.width);
        self.send_command_data_no_cs( 
            Command::Caset,
            &[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]
        ).await?;
        // self.send_data(&[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]).await?;
        Ok(())
    }

    #[inline(always)]
    async fn set_rows_no_cs(&mut self, start: u16, end: u16) -> Result<(),spi::Error> {
        assert!(start <= end && end <= self.height);
        self.send_command_data_no_cs(
            Command::Raset,
            &[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]).await?;
        // self.send_data(&[(start >> 8) as u8, (start & 0xff) as u8, (end >> 8) as u8, (end & 0xff) as u8]).await?;
        Ok(())
    }


    /// Select a window.
    async fn set_window(&mut self, start_x: u16, start_y: u16, end_x: u16, end_y: u16)-> Result<(),spi::Error> {
        // self.cs_pin.set(false);
        // self.set_columns_no_cs(start_x, end_x).await?;
        // self.set_rows_no_cs(start_y, end_y).await?;
        // self.send_command_no_cs(Command::Ramwr).await?;
        // self.cs_pin.set(true);
        self.set_columns(start_x, end_x).await?;
        self.set_rows(start_y, end_y).await?;
        self.send_command(Command::Ramwr).await?;
        Ok(())
    }

    /// Draw a vertical line.
    pub async fn draw_vertical_line(&mut self, x: u16, y: u16, length: u16, color: u16) -> Result<(),spi::Error> {
        self.draw_solid_rect(x, y, 1, length, color).await?;
        Ok(())
    }

    /// Draw a horizontal line.
    pub async fn draw_horizontal_line(&mut self, x: u16, y: u16, length: u16, color: u16)-> Result<(),spi::Error> {
        self.draw_solid_rect(x, y, length, 1, color).await?;
        Ok(())
    }

    /// Draw a single pixel.**Not recommended**.
    pub async fn pixel(&mut self, x: u16, y: u16, color: u16)-> Result<(),spi::Error> {
        self.set_window(x, y, x, y).await?;
        self.send_data(&[(color >> 8) as u8, (color & 0xff) as u8]).await?;
        Ok(())
    }
    /// Draw the color buffer into an area.
    ///
    /// The `bitmap` is a color array of `u16`.
    pub async fn draw_color_buf(&mut self, bitmap: &[u16], x: u16, y: u16, width: u16, height: u16) -> Result<(),spi::Error>  {
        assert_eq!(bitmap.len(), width as usize * height as usize);
        self.set_window(x, y, x + width - 1, y + height - 1).await?;
        let chunks = (width * height) / BUFFER_SIZE;
        let rest = (width * height) % BUFFER_SIZE;

        let buf: &mut [u8] = &mut [0u8; BUFFER_SIZE as usize * 2];

        let mut index = 0;
        for _ in 0..chunks {
            for i in 0..BUFFER_SIZE {
                buf[i as usize * 2]     = (bitmap[index] >> 8) as u8;
                buf[i as usize * 2 + 1] = (bitmap[index] & 0xff) as u8;
                index += 1;
            }
            self.send_data(buf).await?;
        }
        if rest > 0 {
            for i in 0..rest {
                buf[i as usize * 2]     = (bitmap[index] >> 8) as u8;
                buf[i as usize * 2 + 1] = (bitmap[index] & 0xff) as u8;
                index += 1;
            }
            self.send_data(&buf[0..2 * rest as usize]).await?;
        }
        Ok(())
    }


    /// Draw the raw color buffer into an area.
    ///
    /// The `buf` is a color array of `u8` which encoded with big-endian.
    pub async fn draw_color_buf_raw(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<(),spi::Error>   {
        assert_eq!(buffer.len(), width as usize * height as usize * 2);
        self.set_window(x, y, x + width - 1, y + height - 1).await?;
        self.send_data(buffer).await?;
        Ok(())
    }

    /// Draw a solid rectangle.
    pub async fn draw_solid_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) -> Result<(),spi::Error>   {
        self.set_window(x, y, x + width - 1, y + height - 1).await?;
        let pixel: [u8; 2] = [(color >> 8) as u8, (color & 0xff) as u8];
        let chunks = (width * height) / BUFFER_SIZE;
        let rest   = (width * height) % BUFFER_SIZE;

        let buf: &mut [u8] = &mut [0u8; BUFFER_SIZE as usize * 2];
        for i in 0..BUFFER_SIZE {
            buf[i as usize * 2]     = pixel[0];
            buf[i as usize * 2 + 1] = pixel[1];
        }

        for _ in 0..chunks {
            self.send_data(buf).await?;
        }
        if rest > 0 {
            self.send_data(&buf[0..2 * rest as usize]).await?;
        }
        Ok(())
    }

    /// Fill the screen with a color.
    pub async fn fill(&mut self, color: u16)-> Result<(),spi::Error> {
        self.draw_solid_rect(0, 0, self.width, self.height, color).await?;
        Ok(())
    }

    /// Draw a hollow rectangle.
    pub async fn draw_hollow_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) -> Result<(),spi::Error> {
        self.draw_horizontal_line(x, y, width, color).await?;
        self.draw_horizontal_line(x, y + height - 1, width, color).await?;
        self.draw_vertical_line(x, y, height, color).await?;
        self.draw_vertical_line(x + width - 1, y, height, color).await?;
        Ok(())
    }

    
    /// Draw a line from (x0, y0) to (x1, y1).
    pub async fn line(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(),spi::Error> {
        let mut x0 = x0;
        let mut y0 = y0;
        let mut x1 = x1;
        let mut y1 = y1;
        let steep = (y1 as i16 - y0 as i16).abs() > (x1 as i16 - x0 as i16).abs();
        if steep {
            mem::swap(&mut x0, &mut y0);
            mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
            mem::swap(&mut y0, &mut y1);
        }
        let dx: i16 = x1 as i16 - x0 as i16;
        let dy: i16 = y1 as i16 - y0 as i16;
        let mut derror: i16 = (dx / 2) as i16;
        let ystep: i16 = if y0 < y1 { 1 } else { -1 };
        let mut y: i16 = y0 as i16;
        for x in x0..=x1 {
            if steep {
                self.pixel(y as u16, x as u16, 0xffff).await?;
            } else {
                self.pixel(x as u16, y as u16, 0xffff).await?;
            }
            derror -= dy;
            if derror < 0 {
                y += ystep;
                derror += dx;
            }
        }
        Ok(())
    }


    /// Set Vertical Scrolling Definition.
    ///
    /// To scroll a 135x240 display these values should be 40, 240, 40.
    /// There are 40 lines above the display that are not shown followed by
    /// 240 lines that are shown followed by 40 more lines that are not shown.
    /// You could write to these areas off display and scroll them into view by
    /// changing the TFA, VSA and BFA values.
    ///
    /// Args:
    ///
    /// tfa (u16): Top Fixed Area
    ///
    /// vsa (u16): Vertical Scrolling Area
    ///
    /// bfa (u16): Bottom Fixed Area
    pub async fn vscrdef(&mut self, tfa: u16, vsa: u16, bfa: u16) -> Result<(),spi::Error>  {
        self.send_command(Command::Vscrdef).await?;
        self.send_data(&tfa.to_be_bytes()).await?;
        self.send_data(&vsa.to_be_bytes()).await?;
        self.send_data(&bfa.to_be_bytes()).await?;
        Ok(())
    }

    /// Set Vertical Scroll Start Address of RAM.
    ///
    /// Defines which line in the Frame Memory will be written as the first
    /// line after the last line of the Top Fixed Area on the display.
    ///
    /// Args:
    ///
    /// vssa (u16): Vertical Scrolling Start Address
    pub async fn vscsad(&mut self, vssa: u16) -> Result<(),spi::Error>  {
        self.send_command(Command::Vscsad).await?;
        self.send_data(&vssa.to_be_bytes()).await?;
        Ok(())
    }


    /// Draw text with a specific font.
    /// `x` and `y` are the top left corner of the text.
    /// Returns the bottom right corner of the text box.
    ///
    /// When meeting a newline character or reach the end of screen, the next line will be drawn.
    ///
    /// When the distance to the bottom of the screen is less than the font height, it will stop drawing.
    pub async fn draw_text(&mut self, x: u16, y: u16, text: &str, font: &dyn Font, font_color: u16, background_color: u16) -> Result<(u16,u16),spi::Error>
    {
        let start_x = x;
        let mut end_x = x;
        let height = font.get_height() as u16;
        let mut x = x;
        let mut y = y;

        let render_buffer = &mut [0u8; BUFFER_SIZE as usize * 2];

        for c in text.chars() {
            if c == '\n' {
                if x > end_x {
                    end_x = x;
                }
                x = start_x;
                y += height;
                if y + height > self.height as u16 {
                    return Ok((end_x, y));
                } else {
                    continue;
                }
            }
            if let Some((buf, w)) = font.get_char(c) {
                if x + w as u16 > self.width {
                    if x > end_x {
                        end_x = x;
                    }
                    x = start_x;
                    y += height;
                    if y + height > self.height as u16 {
                        return Ok((end_x, y));
                    }
                }
                self.set_window(x, y, x + w as u16 - 1, y + height - 1).await?;

                let mut buf_index: usize = 0;
                for i in (0 as usize)..(w as usize * height as usize) {
                    if buf_index == (BUFFER_SIZE * 2) as usize 
                    {
                        buf_index = 0;
                        self.send_data(render_buffer).await?;
                    }
                    if buf[i >> 3] & (0x80 >> (i & 7)) != 0 {
                        render_buffer[buf_index] = (font_color >> 8) as u8;
                        render_buffer[buf_index + 1] = (font_color & 0xff) as u8;
                    } else {
                        render_buffer[buf_index] = (background_color >> 8) as u8;
                        render_buffer[buf_index + 1] = (background_color & 0xff) as u8;
                    }
                    buf_index += 2;
                }
                if buf_index != 0 {
                    self.send_data(&render_buffer[0..buf_index]).await?;
                }
                x += w as u16;
            }
        }
        Ok((end_x, y + height as u16))
    }

}
 