use crate::{
    color::Hsv,
    pattern::{Index, Pattern},
};

pub trait Layout {
    fn spines(&self) -> u8;
    fn arcs(&self) -> u8;
    fn leds(&self) -> u16;

    fn spine_len_at(&self, index: u8) -> u8;
    fn tip_len_at(&self, index: u8) -> u8;
    fn arc_len_at(&self, index: u8) -> u8;
}

#[derive(Clone, Copy, Debug)]
pub struct FixedLayout {
    pub spines: u8,
    pub arcs: u8,
    pub leds: u16,
    pub spine_len: u8,
    pub tip_len: u8,
    pub arc_len: u8,
}

impl FixedLayout {
    pub fn update_led_count(&mut self) {
        let spines = self.spines as u16 * self.spine_len as u16 * 2;
        let arcs = self.arcs as u16 * self.arc_len as u16;
        let tip_len = self.spines as u16 * self.tip_len as u16;
        self.leds = spines + arcs + tip_len;
    }
}

impl Layout for FixedLayout {
    #[inline(always)]
    fn spines(&self) -> u8 {
        self.spines
    }

    #[inline(always)]
    fn arcs(&self) -> u8 {
        self.arcs
    }

    #[inline(always)]
    fn leds(&self) -> u16 {
        self.leds
    }

    #[inline(always)]
    fn spine_len_at(&self, _index: u8) -> u8 {
        self.spine_len
    }

    #[inline(always)]
    fn tip_len_at(&self, _index: u8) -> u8 {
        self.tip_len
    }

    #[inline(always)]
    fn arc_len_at(&self, _index: u8) -> u8 {
        self.arc_len
    }
}

/// 3D star configuration with compile-time spine count and arc LED count
///
/// The star consists of:
/// - N spines radiating from center, each with LEDs going out and mirroring back
/// - Arc LEDs on inner sphere connecting adjacent spines
///
/// Physical LED order: Spine 0 out, Spine 0 back, Arc 0→1, Spine 1 out, Spine 1 back, Arc 1→2, ...
pub struct Star<L, P>
where
    L: Layout,
    P: Pattern,
{
    pub layout: L,
    pub pattern: P,
}

impl<L, P> Star<L, P>
where
    L: Layout,
    P: Pattern,
{
    #[inline(always)]
    pub fn new(layout: L, pattern: P) -> Self {
        Self { layout, pattern }
    }

    pub fn tick(&mut self) {
        self.pattern.tick();
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = Hsv> + ExactSizeIterator + '_ {
        StarIter {
            star: self,
            current_spine: 0,
            current_position: Position::SpineOut,
            position_offset: 0,
        }
    }
}

struct StarIter<'a, L, P>
where
    L: Layout,
    P: Pattern,
{
    star: &'a Star<L, P>,
    current_spine: u8,
    current_position: Position,
    position_offset: u8,
}

#[derive(Debug, Clone, Copy)]
enum Position {
    SpineOut,
    // Used for odd spine lengths
    SpineTip,
    SpineBack,
    Arc,
}

impl<'a, L, P> StarIter<'a, L, P>
where
    L: Layout,
    P: Pattern,
{
    #[inline(always)]
    fn advance(&mut self) -> bool {
        self.position_offset += 1;

        match self.current_position {
            Position::SpineOut => {
                let spine_len = self.star.layout.spine_len_at(self.current_spine);
                if self.position_offset >= spine_len {
                    // If we have an even number of leds on this spine then we move straight to the back.
                    // Otherwise we need to calculate the odd "tip" of the spine.
                    let tip_len = self.star.layout.tip_len_at(self.current_spine);
                    self.current_position = if tip_len == 0 {
                        Position::SpineBack
                    } else {
                        Position::SpineTip
                    };
                    self.position_offset = 0;
                }
            }
            Position::SpineTip => {
                let tip_len = self.star.layout.tip_len_at(self.current_spine);
                if self.position_offset >= tip_len {
                    // Move to spine back
                    self.current_position = Position::SpineBack;
                    self.position_offset = 0;
                }
            }
            Position::SpineBack => {
                let spine_len = self.star.layout.spine_len_at(self.current_spine);
                if self.position_offset >= spine_len {
                    // Check if arc has any LEDs
                    let arc_len = self.star.layout.arc_len_at(self.current_spine);
                    if arc_len == 0 {
                        // Skip arc, move to next spine
                        self.current_spine += 1;
                        if self.current_spine >= self.star.layout.spines() {
                            return false; // Done iterating
                        }
                        self.current_position = Position::SpineOut;
                        self.position_offset = 0;
                    } else {
                        // Move to arc
                        self.current_position = Position::Arc;
                        self.position_offset = 0;
                    }
                }
            }
            Position::Arc => {
                let arc_len = self.star.layout.arc_len_at(self.current_spine);
                if self.position_offset >= arc_len {
                    // Move to next spine
                    self.current_spine += 1;
                    if self.current_spine >= self.star.layout.spines() {
                        return false; // Done iterating
                    }
                    self.current_position = Position::SpineOut;
                    self.position_offset = 0;
                }
            }
        }

        true
    }
}

impl<'a, L, P> Iterator for StarIter<'a, L, P>
where
    L: Layout,
    P: Pattern,
{
    type Item = Hsv;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_spine >= self.star.layout.spines() {
            return None;
        }

        let spine_index = Index {
            index: self.current_spine as u8,
            total: self.star.layout.spines(),
        };

        let color = match self.current_position {
            Position::SpineOut => {
                let spine_len = self.star.layout.spine_len_at(self.current_spine);
                unsafe {
                    assume!(spine_len > 0);
                }
                // LED index is just the offset
                let led_index = Index {
                    index: self.position_offset as u8,
                    total: spine_len as u8,
                };
                self.star.pattern.spine_color_at(spine_index, led_index)
            }
            Position::SpineTip => {
                let tip_len = self.star.layout.tip_len_at(self.current_spine);
                unsafe {
                    assume!(tip_len > 0);
                }
                // Tip LED at the end of the spine
                let led_index = Index {
                    index: self.position_offset as u8,
                    total: tip_len as u8,
                };
                self.star.pattern.spine_tip_color_at(spine_index, led_index)
            }
            Position::SpineBack => {
                let spine_len = self.star.layout.spine_len_at(self.current_spine);
                unsafe {
                    assume!(spine_len > 0);
                }
                // Mirror: if spine has N LEDs, offset 0 back = LED N-1
                let led_index = (spine_len - 1 - self.position_offset) as u8;
                let led_index = Index {
                    index: led_index,
                    total: spine_len as u8,
                };
                self.star.pattern.spine_color_at(spine_index, led_index)
            }
            Position::Arc => {
                let arc_len = self.star.layout.arc_len_at(self.current_spine);
                unsafe {
                    assume!(arc_len > 0);
                }
                let led_index = Index {
                    index: self.position_offset as u8,
                    total: arc_len as u8,
                };
                self.star.pattern.arc_color_at(spine_index, led_index)
            }
        };

        self.advance();
        Some(color)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let leds = self.star.layout.leds() as usize;
        (leds, Some(leds))
    }
}

impl<'a, L, P> ExactSizeIterator for StarIter<'a, L, P>
where
    L: Layout,
    P: Pattern,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock layout for testing
    struct TestLayout {
        spine_lens: Vec<u8>,
        tip_lens: Vec<u8>,
        arc_lens: Vec<u8>,
    }

    impl Layout for TestLayout {
        fn spines(&self) -> u8 {
            self.spine_lens.len() as u8
        }

        fn arcs(&self) -> u8 {
            self.arc_lens.len() as u8
        }

        fn leds(&self) -> u16 {
            let mut total = 0u16;
            for i in 0..self.spines() {
                // Spine out + tip + spine back
                total += self.spine_len_at(i) as u16 * 2;
                total += self.tip_len_at(i) as u16;
                // Arc
                total += self.arc_len_at(i) as u16;
            }
            total
        }

        fn spine_len_at(&self, index: u8) -> u8 {
            self.spine_lens[index as usize]
        }

        fn tip_len_at(&self, index: u8) -> u8 {
            self.tip_lens[index as usize]
        }

        fn arc_len_at(&self, index: u8) -> u8 {
            self.arc_lens[index as usize]
        }
    }

    // Mock pattern that encodes position info in the color
    struct TestPattern;

    impl Pattern for TestPattern {
        fn tick(&mut self) {
            // No-op for test pattern
        }

        fn spine_color_at(&self, spine: Index, led: Index) -> Hsv {
            // Encode spine and LED index in hue and saturation
            Hsv::new(spine.index, led.index, 255)
        }

        fn spine_tip_color_at(&self, spine: Index, _led: Index) -> Hsv {
            // Use max saturation to mark tip
            Hsv::new(spine.index, 255, 128)
        }

        fn arc_color_at(&self, spine: Index, led: Index) -> Hsv {
            // Use v=64 to mark arcs
            Hsv::new(spine.index, led.index, 64)
        }
    }

    #[test]
    fn test_even_spine_no_tip() {
        // Single spine with even number of LEDs (no tip)
        let layout = TestLayout {
            spine_lens: vec![3],
            tip_lens: vec![0],
            arc_lens: vec![1],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        // Should have: 3 out + 0 tip + 3 back + 1 arc = 7 LEDs
        assert_eq!(star.layout.leds(), 7);

        let colors: Vec<Hsv> = star.iter().collect();
        assert_eq!(colors.len(), 7, "Should have exactly 7 LEDs");

        // Check spine out (0, 1, 2)
        assert_eq!(colors[0], Hsv::new(0, 0, 255)); // Spine 0, LED 0
        assert_eq!(colors[1], Hsv::new(0, 1, 255)); // Spine 0, LED 1
        assert_eq!(colors[2], Hsv::new(0, 2, 255)); // Spine 0, LED 2

        // Check spine back (2, 1, 0) - mirrored
        assert_eq!(colors[3], Hsv::new(0, 2, 255)); // Spine 0, LED 2 (mirrored)
        assert_eq!(colors[4], Hsv::new(0, 1, 255)); // Spine 0, LED 1 (mirrored)
        assert_eq!(colors[5], Hsv::new(0, 0, 255)); // Spine 0, LED 0 (mirrored)

        // Check arc
        assert_eq!(colors[6], Hsv::new(0, 0, 64)); // Arc LED
    }

    #[test]
    fn test_odd_spine_with_tip() {
        // Single spine with odd config (has tip)
        let layout = TestLayout {
            spine_lens: vec![2],
            tip_lens: vec![1],
            arc_lens: vec![2],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        // Should have: 2 out + 1 tip + 2 back + 2 arc = 7 LEDs
        assert_eq!(star.layout.leds(), 7);

        let colors: Vec<Hsv> = star.iter().collect();
        assert_eq!(colors.len(), 7, "Should have exactly 7 LEDs");

        // Check spine out
        assert_eq!(colors[0], Hsv::new(0, 0, 255)); // LED 0
        assert_eq!(colors[1], Hsv::new(0, 1, 255)); // LED 1

        // Check tip (special marker: v=128)
        assert_eq!(colors[2], Hsv::new(0, 255, 128)); // Tip LED

        // Check spine back (mirrored)
        assert_eq!(colors[3], Hsv::new(0, 1, 255)); // LED 1
        assert_eq!(colors[4], Hsv::new(0, 0, 255)); // LED 0

        // Check arc
        assert_eq!(colors[5], Hsv::new(0, 0, 64)); // Arc LED 0
        assert_eq!(colors[6], Hsv::new(0, 1, 64)); // Arc LED 1
    }

    #[test]
    fn test_multiple_spines() {
        // Multiple spines with different lengths
        let layout = TestLayout {
            spine_lens: vec![2, 3],
            tip_lens: vec![0, 0],
            arc_lens: vec![1, 1],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        // Spine 0: 2 out + 2 back + 1 arc = 5
        // Spine 1: 3 out + 3 back + 1 arc = 7
        // Total: 12
        assert_eq!(star.layout.leds(), 12);

        let colors: Vec<Hsv> = star.iter().collect();
        assert_eq!(colors.len(), 12);

        // Spine 0 out
        assert_eq!(colors[0], Hsv::new(0, 0, 255));
        assert_eq!(colors[1], Hsv::new(0, 1, 255));

        // Spine 0 back
        assert_eq!(colors[2], Hsv::new(0, 1, 255));
        assert_eq!(colors[3], Hsv::new(0, 0, 255));

        // Arc 0
        assert_eq!(colors[4], Hsv::new(0, 0, 64));

        // Spine 1 out
        assert_eq!(colors[5], Hsv::new(1, 0, 255));
        assert_eq!(colors[6], Hsv::new(1, 1, 255));
        assert_eq!(colors[7], Hsv::new(1, 2, 255));

        // Spine 1 back
        assert_eq!(colors[8], Hsv::new(1, 2, 255));
        assert_eq!(colors[9], Hsv::new(1, 1, 255));
        assert_eq!(colors[10], Hsv::new(1, 0, 255));

        // Arc 1
        assert_eq!(colors[11], Hsv::new(1, 0, 64));
    }

    #[test]
    fn test_varied_spine_configurations() {
        // Mix of even, odd, and different arc lengths
        let layout = TestLayout {
            spine_lens: vec![1, 2, 3],
            tip_lens: vec![1, 0, 1], // spine 0 and 2 have tips
            arc_lens: vec![2, 1, 3],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        // Spine 0: 1 out + 1 tip + 1 back + 2 arc = 5
        // Spine 1: 2 out + 0 tip + 2 back + 1 arc = 5
        // Spine 2: 3 out + 1 tip + 3 back + 3 arc = 10
        // Total: 20
        assert_eq!(star.layout.leds(), 20);

        let colors: Vec<Hsv> = star.iter().collect();
        assert_eq!(colors.len(), 20, "Should have exactly 20 LEDs");

        // Just verify we don't panic and get the right count
        // Detailed order checking would be very verbose
    }

    #[test]
    fn test_iterator_exact_size() {
        let layout = TestLayout {
            spine_lens: vec![5, 5, 5],
            tip_lens: vec![0, 0, 0],
            arc_lens: vec![2, 2, 2],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        let iter = star.iter();

        // ExactSizeIterator should give accurate size
        assert_eq!(iter.len(), 36); // (5*2 + 2) * 3 = 36

        let colors: Vec<Hsv> = iter.collect();
        assert_eq!(colors.len(), 36);
    }

    #[test]
    fn test_no_leds_consumed_twice() {
        // Ensure iterator doesn't produce more LEDs than declared
        let layout = TestLayout {
            spine_lens: vec![10],
            tip_lens: vec![1],
            arc_lens: vec![3],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        let expected = star.layout.leds() as usize;
        let mut iter = star.iter();

        let mut count = 0;
        while iter.next().is_some() {
            count += 1;
            assert!(
                count <= expected,
                "Iterator produced more LEDs than expected!"
            );
        }

        assert_eq!(
            count, expected,
            "Iterator produced exactly the expected number of LEDs"
        );
    }

    #[test]
    fn test_mirroring_symmetry() {
        // Verify that spine out and back have symmetric LED indices
        let layout = TestLayout {
            spine_lens: vec![5],
            tip_lens: vec![0],
            arc_lens: vec![0],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        let colors: Vec<Hsv> = star.iter().collect();

        // Spine out: 0, 1, 2, 3, 4
        // Spine back: 4, 3, 2, 1, 0 (should mirror)
        for i in 0..5 {
            assert_eq!(
                colors[i].s,     // LED index going out
                colors[9 - i].s, // LED index coming back (mirrored)
                "LED at position {} out should match position {} back",
                i,
                9 - i
            );
        }
    }

    #[test]
    fn test_empty_arc() {
        // Test with no arc LEDs
        let layout = TestLayout {
            spine_lens: vec![3, 3],
            tip_lens: vec![0, 0],
            arc_lens: vec![0, 0],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        // Should have: (3*2) * 2 = 12 LEDs total
        assert_eq!(star.layout.leds(), 12);

        let colors: Vec<Hsv> = star.iter().collect();
        assert_eq!(colors.len(), 12);

        // All should be spine LEDs (v=255), no arc LEDs (v=64)
        assert!(colors.iter().all(|c| c.v == 255));
    }

    #[test]
    fn test_single_led_spine() {
        // Edge case: spine with just 1 LED
        let layout = TestLayout {
            spine_lens: vec![1],
            tip_lens: vec![0],
            arc_lens: vec![1],
        };
        let pattern = TestPattern;
        let star = Star::new(layout, pattern);

        // 1 out + 1 back + 1 arc = 3
        assert_eq!(star.layout.leds(), 3);

        let colors: Vec<Hsv> = star.iter().collect();
        assert_eq!(colors.len(), 3);

        // Both out and back should reference LED 0
        assert_eq!(colors[0], Hsv::new(0, 0, 255)); // Out
        assert_eq!(colors[1], Hsv::new(0, 0, 255)); // Back (same LED)
        assert_eq!(colors[2], Hsv::new(0, 0, 64)); // Arc
    }
}
