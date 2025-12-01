use led_star::{
    config,
    pattern::Pattern,
    star::{Layout, Star},
};
use wasm_bindgen::prelude::*;

// Set up panic hook for better error messages in browser console
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

struct State<L: Layout, P: Pattern> {
    star: Star<L, P>,
}

trait StateI {
    fn spines(&self) -> u8;

    fn spine_len_at(&self, index: u8) -> u8;
    fn tip_len_at(&self, index: u8) -> u8;
    fn arc_len_at(&self, index: u8) -> u8;

    fn leds(&self) -> u16;

    fn tick(&mut self);
    fn fill(&self, buf: &mut [u8]) -> Result<(), &'static str>;
}

impl<L: Layout, P: Pattern> StateI for State<L, P> {
    fn spines(&self) -> u8 {
        self.star.layout.spines()
    }

    fn spine_len_at(&self, index: u8) -> u8 {
        self.star.layout.spine_len_at(index)
    }

    fn tip_len_at(&self, index: u8) -> u8 {
        self.star.layout.tip_len_at(index)
    }

    fn arc_len_at(&self, index: u8) -> u8 {
        self.star.layout.arc_len_at(index)
    }

    fn leds(&self) -> u16 {
        self.star.layout.leds()
    }

    fn tick(&mut self) {
        self.star.tick();
    }

    fn fill(&self, buffer: &mut [u8]) -> Result<(), &'static str> {
        if buffer.len() < self.star.layout.leds() as usize * 3 {
            return Err("buffer is too small");
        }
        let mut i = 0;
        for hsv in self.star.iter() {
            if i + 2 >= buffer.len() {
                return Err("Buffer overflow - iterator produced too many LEDs");
            }
            buffer[i] = hsv.h;
            buffer[i + 1] = hsv.s;
            buffer[i + 2] = hsv.v;
            i += 3;
        }
        Ok(())
    }
}

/// Visualizer wrapping a Star with a specific pattern
#[wasm_bindgen]
pub struct Visualizer {
    state: Box<dyn StateI>,
}

impl Default for Visualizer {
    fn default() -> Self {
        let layout = config::layout();
        let pattern = config::pattern();
        let star = Star::new(layout, pattern);
        let state = State { star };
        let state = Box::new(state);

        Self { state }
    }
}

#[wasm_bindgen]
impl Visualizer {
    /// Create a new visualizer with the default layout (6 spines, 10 LEDs per spine, 3 arc LEDs)
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Visualizer, JsValue> {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        Ok(Self::default())
    }

    /// Advance the animation by one tick
    pub fn tick(&mut self) {
        self.state.tick();
    }

    pub fn set_pattern(&mut self, _pattern: &str) -> Result<(), JsValue> {
        // match pattern.parse() {
        //     Ok(pattern) => {
        //         self.star.pattern = pattern;
        //         Ok(())
        //     }
        //     Err(_) => Err(JsValue::from_str("Invalid pattern")),
        // }
        Ok(())
    }

    /// Write LED colors directly into the provided buffer (h, s, v, h, s, v, ...)
    /// The buffer must be at least total_leds() * 3 bytes
    pub fn read_leds_into(&self, buffer: &mut [u8]) -> Result<(), JsValue> {
        self.state.fill(buffer).map_err(|e| JsValue::from(e))
    }

    /// Get the number of spines
    pub fn spines(&self) -> u8 {
        self.state.spines()
    }

    /// Get the number of LEDs per spine (going out)
    pub fn spine_len(&self, spine_idx: u8) -> u8 {
        self.state.spine_len_at(spine_idx)
    }

    /// Get the number of tip LEDs
    pub fn tip_len(&self, spine_idx: u8) -> u8 {
        self.state.tip_len_at(spine_idx)
    }

    /// Get the number of arc LEDs
    pub fn arc_len(&self, index: u8) -> u8 {
        self.state.arc_len_at(index)
    }

    /// Get the total number of LEDs
    pub fn total_leds(&self) -> u16 {
        self.state.leds()
    }
}

/// Get all available patterns
#[wasm_bindgen]
pub fn get_available_patterns() -> Vec<JsValue> {
    // DynamicPattern::NAMES
    //     .into_iter()
    //     .map(|p| JsValue::from(*p))
    //     .collect()
    vec![JsValue::from("Classic")]
}
