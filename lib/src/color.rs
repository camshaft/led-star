#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// HSV color representation for LED patterns
/// - h: Hue (0-255, wrapping around the color wheel)
/// - s: Saturation (0=gray, 255=fully saturated)
/// - v: Value/Brightness (0=off, 255=full brightness)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Hsv {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

impl Hsv {
    /// Create a new HSV color
    #[inline(always)]
    pub const fn new(h: u8, s: u8, v: u8) -> Self {
        Self { h, s, v }
    }
}
