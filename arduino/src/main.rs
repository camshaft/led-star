#![no_std]
#![no_main]
#![cfg_attr(target_arch = "avr", feature(asm_experimental_arch))]

use led_star::{config, star::Star};
use panic_halt as _;

mod ws2812;
use ws2812::Ws2812;

const TIME_DELAY: u32 = 25; // milliseconds between frames

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Configure LED data pin (D3)
    let data_pin = pins.d3.into_output();

    // Initialize WS2812 controller with brightness
    let mut ws2812 = Ws2812::new(data_pin);
    ws2812.set_brightness(84); // ~33% brightness

    // Create star with layout and pattern from config
    let layout = config::layout();
    let pattern = config::pattern();
    let mut star = Star::new(layout, pattern);

    loop {
        // Tick the pattern
        star.tick();

        // Write colors to LED strip
        ws2812.write(star.iter());

        // Delay between frames
        arduino_hal::delay_ms(TIME_DELAY);
    }
}
