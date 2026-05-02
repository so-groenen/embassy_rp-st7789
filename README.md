
# Async ST7789 driver for Pi Pico (rp2040/rp235x) 
https://github.com/user-attachments/assets/5233a544-aacb-490f-b410-c79cc7486b45

### ADAPTED FROM [ri-char/rp2040-st7789](https://github.com/ri-char/rp2040-st7789)

This is a lightweight async Rust driver for the Raspberry Pi Pico chips (Pico, Pico W, Pico 2, Pico 2W), adapted using [Rust embassy](https://github.com/embassy-rs/embassy/)'s async SPI implementation.<br>

The async methods allow to do extra work while data is being transferred to the display: <br>
On the Pi Pico, async driver methods like "draw_text", can be "awaited", meaning we can do some work while data is being processed. This is done without the need of the CPU thanks to the DMA (Direct Memory Access) controller. When the DMA controller has finished sending data via SPI to the display, an interrupt gets triggered (DMA_IRQ_0 for instance). The corresponding driver method is then ready to be polled, and marked as finished by the embassy executor, and we are ready to start sending data again.

The driver support for:
- Async operations (write) 
- 320x240, 240x240 and 135x240 pixel displays
- Display rotation
- Hardware based scrolling
- Drawing text using 8 and 16 bit wide bitmap fonts with heights that are multiples of 8. Included are 12 bitmap fonts derived from classic pc BIOS text mode fonts.
- Drawing text using converted TrueType fonts.
- Drawing converted bitmaps


 
## Get Started

Add this to `Cargo.toml`, using the feature `rp235xa` or `rp2040` depending if you use the Raspberry Pi Pico 2(W) or Pico (W)
```toml
embassy-rp-st7789 = { version = "0.1.0", features = ["rp235xa"], git = "https://github.com/so-groenen/embassy_rp-st7789"}
```

## Font

The project contains two fonts and 6 different sizes each type.
You can add your own font by implementing `trait Font`.


## Example

The example is at [./example](./example).

## Reference

1. [ri-char/rp2040-st7789](https://github.com/ri-char/rp2040-st7789)
2. [russhughes/st7789py_mpy](https://github.com/russhughes/st7789py_mpy)
