use crate::{color::Hsv, osc, pattern::*, slotmap::SlotMap, storage::Storage};
use core::fmt;

/// A bitpacked streak state stored in 2 bytes
/// Byte 0: [pppppppf] - position (7 bits int, 1 bit frac)
/// Byte 1: [lllllvvv] - length (5 bits) | velocity (3 bits)
#[derive(Clone, Copy, Default)]
pub struct StreakState {
    data: [u8; 2],
}

impl fmt::Debug for StreakState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreakState")
            .field("position", &self.position())
            .field("length", &self.length())
            .field("velocity", &self.velocity_multiplier())
            .finish()
    }
}

impl StreakState {
    /// Create a new streak state with the given length and velocity
    /// - length: 0-31 (5 bits)
    /// - velocity: 0-7 (3 bits, maps to 0.5x-2x multiplier)
    #[inline(always)]
    pub fn new(length: u8, velocity: u8) -> Self {
        debug_assert!(length <= 31, "length must be 0-31");
        debug_assert!(velocity <= 7, "velocity must be 0-7");

        Self {
            data: [
                0, // position starts at 0
                (length << 3) | (velocity & 0x07),
            ],
        }
    }

    /// Get the position as a 7.1 fixed-point value
    #[inline(always)]
    pub fn position_fixed(&self) -> u8 {
        self.data[0]
    }

    /// Set the position as a 7.1 fixed-point value
    #[inline(always)]
    pub fn set_position(&mut self, pos: u8) {
        self.data[0] = pos;
    }

    /// Get the integer part of the position (0-127)
    #[inline(always)]
    pub fn position(&self) -> u8 {
        self.data[0] >> 1
    }

    /// Get the length (0-31)
    #[inline(always)]
    pub fn length(&self) -> u8 {
        self.data[1] >> 3
    }

    /// Get the velocity multiplier value (0-7)
    #[inline(always)]
    pub fn velocity_multiplier(&self) -> u8 {
        self.data[1] & 0x07
    }

    #[inline(always)]
    pub fn tick(&mut self) {
        let vel_mult = velocity_to_multiplier(self.velocity_multiplier());
        let new_pos = self.position_fixed().saturating_add(vel_mult);
        self.set_position(new_pos);
    }
}

/// Convert oscillator value (-128..127) to 5-bit value (0..31)
#[inline(always)]
fn map_i8_to_5bit(value: i8) -> u8 {
    // Add 128 to make 0..255, then divide by 8 (256/32)
    ((value as u8).wrapping_add(128) >> 3).min(31)
}

/// Convert oscillator value (-128..127) to 3-bit value (0..7)
#[inline(always)]
fn map_i8_to_3bit(value: i8) -> u8 {
    // Add 128 to make 0..255, then divide by 32 (256/8)
    ((value as u8).wrapping_add(128) >> 5).min(7)
}

/// Convert 3-bit velocity value to fixed-point multiplier for 7.1 format
/// Maps 0..7 to 1..4 (representing 0.5x to 2.0x in 7.1 fixed-point)
#[inline(always)]
fn velocity_to_multiplier(velocity_3bit: u8) -> u8 {
    debug_assert!(velocity_3bit <= 7);
    // Linear mapping: 0→1 (0.5x), 7→4 (2.0x) in 7.1 fixed-point
    // mult = 1 + velocity * 3/7
    1 + (velocity_3bit * 3) / 7
}

pub struct StreakSpawner<Spawner, Length, Velocity, TotalLeds, Inner, Streaks>
where
    Spawner: osc::Oscillator,
    Length: osc::Oscillator,
    Velocity: osc::Oscillator,
    TotalLeds: osc::Oscillator,
    Inner: Pattern,
    Streaks: Storage<Value = StreakState>,
{
    pub spawner: Spawner,
    pub length: Length,
    pub velocity: Velocity,
    pub total_leds: TotalLeds,
    pub inner: Inner,
    pub streaks: SlotMap<StreakState, Streaks, 8>,
}

impl<Spawner, Length, Velocity, TotalLeds, Inner, Streaks> fmt::Debug
    for StreakSpawner<Spawner, Length, Velocity, TotalLeds, Inner, Streaks>
where
    Spawner: osc::Oscillator + fmt::Debug,
    Length: osc::Oscillator + fmt::Debug,
    Velocity: osc::Oscillator + fmt::Debug,
    TotalLeds: osc::Oscillator + fmt::Debug,
    Inner: Pattern + fmt::Debug,
    Streaks: Storage<Value = StreakState>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return self.streaks.fmt(f);
        }

        f.debug_struct("StreakSpawner")
            .field("spawner", &self.spawner)
            .field("length", &self.length)
            .field("velocity", &self.velocity)
            .field("total_leds", &self.total_leds)
            .field("inner", &self.inner)
            .field("streaks", &self.streaks)
            .finish()
    }
}

impl<Spawner, Length, Velocity, TotalLeds, Inner, Streaks>
    StreakSpawner<Spawner, Length, Velocity, TotalLeds, Inner, Streaks>
where
    Spawner: osc::Oscillator,
    Length: osc::Oscillator,
    Velocity: osc::Oscillator,
    TotalLeds: osc::Oscillator,
    Inner: Pattern,
    Streaks: Storage<Value = StreakState>,
{
    pub fn new(
        spawner: Spawner,
        length: Length,
        velocity: Velocity,
        total_leds: TotalLeds,
        inner: Inner,
        streaks: Streaks,
    ) -> Self {
        Self {
            spawner,
            length,
            velocity,
            total_leds,
            inner,
            streaks: SlotMap::new(streaks),
        }
    }
}

impl<Spawner, Length, Velocity, TotalLeds, Inner, Streaks> Pattern
    for StreakSpawner<Spawner, Length, Velocity, TotalLeds, Inner, Streaks>
where
    Spawner: osc::Oscillator,
    Length: osc::Oscillator,
    Velocity: osc::Oscillator,
    TotalLeds: osc::Oscillator,
    Inner: Pattern,
    Streaks: Storage<Value = StreakState>,
{
    #[inline(always)]
    fn tick(&mut self) {
        // Tick all oscillators
        self.spawner.tick();
        self.length.tick();
        self.velocity.tick();
        self.total_leds.tick();
        self.inner.tick();

        // Spawn new streak if conditions met
        if self.spawner.get() > 0 && !self.streaks.is_full() {
            // Sample current oscillator values
            let length = map_i8_to_5bit(self.length.get());
            // 0-length spawns are not valid
            if length > 0 {
                let velocity = map_i8_to_3bit(self.velocity.get());

                let streak = StreakState::new(length, velocity);
                self.streaks.insert(streak);
            }
        }

        // Update all active streaks
        let total_leds = self.total_leds.get() as u8;
        self.streaks.retain(|streak| {
            streak.tick();

            // Keep alive while tail is visible
            let head_pos = streak.position();
            let tail_pos = head_pos.saturating_sub(streak.length());
            tail_pos < total_leds
        });
    }

    #[inline(always)]
    fn spine_color_at(&self, spine: Index, led: Index) -> Hsv {
        self.calculate_streak_color(self.inner.spine_color_at(spine, led), led)
    }

    #[inline(always)]
    fn spine_tip_color_at(&self, spine: Index, led: Index) -> Hsv {
        self.calculate_streak_color(self.inner.spine_tip_color_at(spine, led), led)
    }

    #[inline(always)]
    fn arc_color_at(&self, arc: Index, led: Index) -> Hsv {
        self.calculate_streak_color(self.inner.arc_color_at(arc, led), led)
    }
}

impl<Spawner, Length, Velocity, TotalLeds, Inner, Streaks>
    StreakSpawner<Spawner, Length, Velocity, TotalLeds, Inner, Streaks>
where
    Spawner: osc::Oscillator,
    Length: osc::Oscillator,
    Velocity: osc::Oscillator,
    TotalLeds: osc::Oscillator,
    Inner: Pattern,
    Streaks: Storage<Value = StreakState>,
{
    fn calculate_streak_color(&self, mut color: Hsv, led: Index) -> Hsv {
        let brightness = self.calculate_streak_brightness(led);
        color.v = brightness;
        color
    }

    #[inline(always)]
    fn calculate_streak_brightness(&self, led: Index) -> u8 {
        let mut max_brightness = 0u8;

        // Iterate over all active streaks
        for streak in self.streaks.iter() {
            let head_pos = streak.position();
            let length = streak.length();
            unsafe {
                assume!(length > 0);
            }

            if head_pos == led.index {
                // LED is at head, full brightness
                return 255;
            }

            // Calculate distance from head to this LED
            // Distance is positive when LED is behind the head
            let Some(distance) = head_pos.checked_sub(led.index) else {
                // LED is ahead of head, not illuminated
                continue;
            };

            // Check if LED is within the streak length
            if distance > length {
                continue;
            }

            // Calculate brightness based on distance within the streak
            // Brightness is highest at head (distance=0) and decreases linearly
            let brightness_factor = (((length - distance) as u16 * 255) / length as u16) as u8;

            max_brightness = max_brightness.max(brightness_factor);
        }

        max_brightness
    }
}

/// Arc-specific streak that visits all spines in circular order
/// Uses fixed-point position that wraps at total arc LEDs
pub struct ArcStreak<Length, Velocity, Inner, const ARC_LEN: u8, const TOTAL_ARCS: u8>
where
    Length: osc::Oscillator,
    Velocity: osc::Oscillator,
    Inner: Pattern,
{
    position: u8, // 7.1 fixed-point position
    length: Length,
    velocity: Velocity,
    inner: Inner,
}

impl<Length, Velocity, Inner, const ARC_LEN: u8, const TOTAL_ARCS: u8>
    ArcStreak<Length, Velocity, Inner, ARC_LEN, TOTAL_ARCS>
where
    Length: osc::Oscillator,
    Velocity: osc::Oscillator,
    Inner: Pattern,
{
    pub fn new(length: Length, velocity: Velocity, inner: Inner) -> Self {
        Self {
            position: 0,
            length,
            velocity,
            inner,
        }
    }

    /// Get the current head position in arc coordinates
    /// Returns (spine_index, led_index) where the streak head is located
    #[inline(always)]
    fn head_position(&self) -> (u8, u8) {
        let pos_int = self.position >> 1; // Convert from 7.1 to integer
        let spine = pos_int / ARC_LEN;
        let led = pos_int % ARC_LEN;
        (spine, led)
    }

    /// Calculate the global LED index for a given spine/led coordinate
    #[inline(always)]
    fn global_led_index(spine: u8, led: u8) -> u8 {
        spine * ARC_LEN + led
    }

    const fn total_leds() -> u8 {
        ARC_LEN * TOTAL_ARCS
    }

    // Gets the total number of LEDs in 7.1 fixed-point
    const fn total_led_fixed() -> u8 {
        (ARC_LEN * TOTAL_ARCS) << 1
    }
}

impl<Length, Velocity, Inner, const ARC_LEN: u8, const TOTAL_ARCS: u8> Pattern
    for ArcStreak<Length, Velocity, Inner, ARC_LEN, TOTAL_ARCS>
where
    Length: osc::Oscillator,
    Velocity: osc::Oscillator,
    Inner: Pattern,
{
    fn tick(&mut self) {
        self.length.tick();
        self.velocity.tick();
        self.inner.tick();

        // Update position with velocity
        let velocity = map_i8_to_3bit(self.velocity.get());
        let vel_mult = velocity_to_multiplier(velocity);
        let new_pos = self.position.wrapping_add(vel_mult);

        // Wrap at total arc LEDs (in 7.1 fixed-point)
        self.position = new_pos % const { Self::total_led_fixed() };
    }

    fn spine_color_at(&self, _spine: Index, _led: Index) -> Hsv {
        // Arc streaks don't affect spines
        Hsv::new(0, 0, 0)
    }

    fn spine_tip_color_at(&self, _spine: Index, _led: Index) -> Hsv {
        // Arc streaks don't affect spine tips
        Hsv::new(0, 0, 0)
    }

    fn arc_color_at(&self, arc: Index, led: Index) -> Hsv {
        let mut color = self.inner.arc_color_at(arc, led);

        // Get current length
        let length = map_i8_to_5bit(self.length.get());
        if length == 0 {
            color.v = 0;
            return color;
        }

        // Calculate head position
        let (head_spine, head_led) = self.head_position();
        let head_global = Self::global_led_index(head_spine, head_led);
        let led_global = Self::global_led_index(arc.index, led.index);

        // Calculate distance from head (with wraparound)
        let distance = if head_global >= led_global {
            head_global - led_global
        } else {
            // Wrap around
            (const { Self::total_leds() } - led_global) + head_global
        };

        // Check if within streak length
        if distance <= length {
            // Calculate brightness based on distance
            let remaining = length - distance;
            let brightness = ((remaining as u16 * 255) / length as u16) as u8;
            color.v = brightness;
        } else {
            color.v = 0;
        }

        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc::{Constant, OscillatorExt as _, sawtooth};
    use core::fmt;
    use fmt::Write;
    use insta::assert_snapshot;

    fn run(rows: usize, leds: u8, mut pattern: impl Pattern + fmt::Debug) -> String {
        let mut out = String::new();
        let mut prev_row = (0, String::new());
        let mut pushed_rows = 0;

        loop {
            if pushed_rows >= rows {
                break;
            }
            let mut row = String::new();
            for led in 0..leds {
                if led > 0 {
                    write!(row, " ").unwrap();
                }
                let color = pattern.spine_color_at(
                    Index {
                        index: 0,
                        total: leds,
                    },
                    Index {
                        index: led,
                        total: leds,
                    },
                );
                write!(row, "{:03}", color.v).unwrap();
            }

            // log the positions of the streaks
            write!(row, " | {pattern:?}").unwrap();

            row.push('\n');
            pattern.tick();

            if prev_row.1 == row {
                prev_row.0 += 1;
                continue;
            }

            if prev_row.0 > 0 {
                write!(out, "[{:03}]: {}", prev_row.0, prev_row.1).unwrap();
                pushed_rows += 1;
            }
            prev_row = (1, row);
        }

        if prev_row.0 > 0 {
            write!(out, "[{:03}]: {}", prev_row.0, prev_row.1).unwrap();
        }

        out
    }

    #[test]
    fn test_streak_state_new() {
        let state = StreakState::new(10, 5);
        assert_eq!(state.position_fixed(), 0);
        assert_eq!(state.position(), 0);
        assert_eq!(state.length(), 10);
        assert_eq!(state.velocity_multiplier(), 5);
    }

    #[test]
    fn test_streak_state_position() {
        let mut state = StreakState::new(0, 0);

        // Set position to 8 in fixed-point (4.0 in 7.1 format)
        state.set_position(8);
        assert_eq!(state.position(), 4);
        assert_eq!(state.position_fixed(), 8);

        // Set position to 255 (maximum)
        state.set_position(255);
        assert_eq!(state.position(), 127); // Top 7 bits
        assert_eq!(state.position_fixed(), 255);
    }

    #[test]
    fn test_map_i8_to_5bit() {
        assert_eq!(map_i8_to_5bit(-128), 0);
        assert_eq!(map_i8_to_5bit(0), 16); // (0+128) >> 3 = 128 >> 3 = 16
        assert_eq!(map_i8_to_5bit(127), 31);
    }

    #[test]
    fn test_map_i8_to_3bit() {
        assert_eq!(map_i8_to_3bit(-128), 0);
        assert_eq!(map_i8_to_3bit(0), 4); // (0+128) >> 5 = 128 >> 5 = 4
        assert_eq!(map_i8_to_3bit(127), 7);
    }

    #[test]
    fn test_velocity_to_multiplier() {
        assert_eq!(velocity_to_multiplier(0), 1); // 0.5x in 7.1
        assert_eq!(velocity_to_multiplier(7), 4); // 2.0x in 7.1
    }

    #[test]
    fn test_streak_spawner_basic() {
        let pattern = StreakSpawner::new(
            sawtooth().saturating_sub(126), // spawn once per peak
            Constant::<64>,                 // Mid-range length (~7)
            Constant::<64>,                 // Mid-range velocity (~2x)
            Constant::<8>,                  // 8 LEDs total
            Hsv::new(0, 0, 255),
            [StreakState::default(); 8],
        );

        assert_snapshot!(run(32, 8, pattern));
    }

    #[test]
    fn test_streak_spawner_slow() {
        let pattern = StreakSpawner::new(
            sawtooth().saturating_sub(126), // spawn once per peak
            Constant::<64>,                 // Mid-range length (~7)
            Constant::<-64>,                // Slow velocity (~0.5x)
            Constant::<16>,                 // 16 LEDs total
            Hsv::new(0, 0, 255),
            [StreakState::default(); 8],
        );

        assert_snapshot!(run(64, 16, pattern));
    }
}
