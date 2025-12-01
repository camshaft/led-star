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

    /// Convert HSV to RGB
    ///
    /// This uses a fast approximation suitable for embedded systems,
    /// based on the FastLED HSV-to-RGB algorithm.
    #[inline]
    pub fn to_rgb(self) -> Rgb {
        self.to_rgb_with_brightness(255)
    }

    /// Convert HSV to RGB with brightness scaling
    ///
    /// # Arguments
    /// * `brightness` - Global brightness multiplier (0-255)
    #[inline]
    pub fn to_rgb_with_brightness(self, brightness: u8) -> Rgb {
        let h = self.h;
        let s = self.s;
        let v = scale8(self.v, brightness);

        // If saturation is 0, it's grayscale
        if s == 0 {
            return Rgb { r: v, g: v, b: v };
        }

        // Divide hue into 6 regions (0-5)
        let region = h / 43; // 256 / 6 ≈ 43
        let remainder = (h - (region * 43)) * 6;

        let p = scale8(v, 255 - s);
        let q = scale8(v, 255 - scale8(s, remainder));
        let t = scale8(v, 255 - scale8(s, 255 - remainder));

        match region {
            0 => Rgb { r: v, g: t, b: p },
            1 => Rgb { r: q, g: v, b: p },
            2 => Rgb { r: p, g: v, b: t },
            3 => Rgb { r: p, g: q, b: v },
            4 => Rgb { r: t, g: p, b: v },
            _ => Rgb { r: v, g: p, b: q }, // region 5
        }
    }
}

/// RGB color with 8-bit channels
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Create a new RGB color
    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Scale a value by a factor (0-255)
/// Returns (value * scale + 1) / 256, which provides better rounding
#[inline(always)]
fn scale8(value: u8, scale: u8) -> u8 {
    let product = value as u16 * scale as u16;
    ((product + 1) >> 8) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsv_to_rgb_grayscale() {
        // Zero saturation should produce grayscale
        // Note: scale8(200, 255) = 199 due to fast approximation
        let hsv = Hsv::new(128, 0, 200);
        let rgb = hsv.to_rgb();
        assert_eq!(rgb.r, 199);
        assert_eq!(rgb.g, 199);
        assert_eq!(rgb.b, 199);
    }

    #[test]
    fn test_hsv_to_rgb_pure_red() {
        // Pure red: h=0, full saturation and value
        // Note: scale8 fast approximation produces 254 for 255*255
        let hsv = Hsv::new(0, 255, 255);
        let rgb = hsv.to_rgb();
        assert!(rgb.r >= 254); // Fast approximation
        assert!(rgb.g < 10); // Should be very close to 0
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_hsv_to_rgb_pure_green() {
        // Pure green: h≈85 (256/6 * 2), full saturation and value
        let hsv = Hsv::new(85, 255, 255);
        let rgb = hsv.to_rgb();
        assert!(rgb.r < 10);
        assert!(rgb.g >= 254); // Fast approximation
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_hsv_to_rgb_pure_blue() {
        // Pure blue: h≈170 (256/6 * 4), full saturation and value
        let hsv = Hsv::new(170, 255, 255);
        let rgb = hsv.to_rgb();
        assert_eq!(rgb.r, 0);
        assert!(rgb.g < 10);
        assert!(rgb.b >= 254); // Fast approximation
    }

    #[test]
    fn test_hsv_to_rgb_with_brightness() {
        // Test brightness scaling
        let hsv = Hsv::new(0, 255, 255);
        let rgb = hsv.to_rgb_with_brightness(128);
        // Should be roughly half brightness
        assert!(rgb.r >= 127 && rgb.r <= 128);
        assert!(rgb.g < 5);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_hsv_to_rgb_black() {
        // Zero value should produce black
        let hsv = Hsv::new(128, 255, 0);
        let rgb = hsv.to_rgb();
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_scale8() {
        // Test the scale8 function - uses fast approximation (>> 8 instead of / 255)
        assert_eq!(scale8(255, 255), 254); // (255*255+1)/256 = 254
        assert_eq!(scale8(255, 128), 127);
        assert_eq!(scale8(128, 128), 64); // (128*128+1)/256 = 64
        assert_eq!(scale8(255, 0), 0);
        assert_eq!(scale8(0, 255), 0);
    }
}
