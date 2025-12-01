//! WS2812 LED driver for AVR microcontrollers
//!
//! This module implements bit-banged WS2812 protocol for controlling addressable RGB LEDs.
//! The protocol requires precise timing:
//! - 0 bit: 400ns high, 850ns low
//! - 1 bit: 800ns high, 450ns low
//! - Reset: >50μs low

use arduino_hal::port::{Pin, PinOps, mode::Output};
use core::iter::{IntoIterator, Iterator};
use led_star::color::Hsv;

/// WS2812 LED strip controller
pub struct Ws2812<P: PinOps> {
    pin: Pin<Output, P>,
    brightness: u8,
}

impl<P: PinOps> Ws2812<P> {
    /// Create a new WS2812 controller on the given pin
    pub fn new(pin: Pin<Output, P>) -> Self {
        Self {
            pin,
            brightness: 255,
        }
    }

    /// Set global brightness (0-255)
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
    }

    /// Send color data to the LED strip
    ///
    /// # Arguments
    /// * `colors` - Iterator of HSV colors to send to the strip
    pub fn write<I>(&mut self, colors: I)
    where
        I: IntoIterator<Item = Hsv>,
    {
        // Disable interrupts for precise timing
        avr_device::interrupt::free(|_| {
            for hsv in colors {
                let rgb = hsv.to_rgb_with_brightness(self.brightness);
                // WS2812 expects GRB order
                self.write_byte(rgb.g);
                self.write_byte(rgb.r);
                self.write_byte(rgb.b);
            }
        });

        // Reset pulse (>50μs low)
        self.pin.set_low();
        arduino_hal::delay_us(60);
    }

    /// Write a single byte using WS2812 timing
    #[inline(always)]
    fn write_byte(&mut self, byte: u8) {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            if bit == 1 {
                self.write_one();
            } else {
                self.write_zero();
            }
        }
    }

    /// Write a '1' bit: 800ns high, 450ns low (approx)
    /// At 16MHz, each cycle is 62.5ns
    /// 800ns ≈ 13 cycles, 450ns ≈ 7 cycles
    #[inline(always)]
    fn write_one(&mut self) {
        self.pin.set_high();
        // ~800ns high (assembly ensures precise timing)
        unsafe {
            core::arch::asm!(
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                options(nomem, nostack)
            );
        }
        self.pin.set_low();
        // ~450ns low
        unsafe {
            core::arch::asm!("nop", "nop", "nop", options(nomem, nostack));
        }
    }

    /// Write a '0' bit: 400ns high, 850ns low (approx)
    /// At 16MHz: 400ns ≈ 6 cycles, 850ns ≈ 14 cycles
    #[inline(always)]
    fn write_zero(&mut self) {
        self.pin.set_high();
        // ~400ns high
        unsafe {
            core::arch::asm!("nop", "nop", "nop", options(nomem, nostack));
        }
        self.pin.set_low();
        // ~850ns low
        unsafe {
            core::arch::asm!(
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                options(nomem, nostack)
            );
        }
    }
}
