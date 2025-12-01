use crate::{
    color::Hsv,
    osc::*,
    pattern::{Compound, Oscillator, Pattern, PerSpine},
    streak::StreakSpawner,
};

const SPINE_LEN: u8 = 70 / 2;
const TIP_LEN: u8 = 0;
const ARC_LEN: u8 = 5;

const SPINE_COUNT: u8 = 12;
const ARC_COUNT: u8 = 12;

const LED_COUNT: u16 = {
    let spines = SPINE_COUNT as u16 * SPINE_LEN as u16 * 2;
    let arcs = ARC_COUNT as u16 * ARC_LEN as u16;
    let tip_len = SPINE_COUNT as u16 * TIP_LEN as u16;
    spines + arcs + tip_len
};

pub struct Layout;

impl crate::star::Layout for Layout {
    #[inline(always)]
    fn spines(&self) -> u8 {
        SPINE_COUNT
    }

    #[inline(always)]
    fn arcs(&self) -> u8 {
        ARC_COUNT
    }

    #[inline(always)]
    fn leds(&self) -> u16 {
        LED_COUNT
    }

    #[inline(always)]
    fn spine_len_at(&self, _index: u8) -> u8 {
        SPINE_LEN
    }

    #[inline(always)]
    fn tip_len_at(&self, _index: u8) -> u8 {
        TIP_LEN
    }

    #[inline(always)]
    fn arc_len_at(&self, _index: u8) -> u8 {
        ARC_LEN
    }
}

pub fn layout() -> impl crate::star::Layout {
    Layout
}

pub fn pattern() -> impl Pattern {
    Compound {
        spine: spines::<{ SPINE_COUNT as usize }>(),
        // TODO
        tip: Hsv::new(0, 0, 0),
        arc: arc_pattern(),
    }
}

pub fn arc_pattern() -> impl Pattern {
    let osc = Oscillator {
        h: sawtooth(),
        s: Constant::<127>,
        v: Constant::<127>,
    };

    crate::streak::ArcStreak::<_, _, _, ARC_LEN, ARC_COUNT>::new(
        Constant::<64>, // Length oscillator (~20 LEDs)
        Constant::<0>,  // Velocity oscillator (~1x speed)
        osc,
    )
}

pub fn spine_pattern(spine: u8) -> impl Pattern {
    let values_per_spine = 255 / SPINE_COUNT;
    let phase = (spine * values_per_spine) as i8;

    let osc = Oscillator {
        // rotate the hue around the color wheel
        h: sawtooth().add(phase),
        // oscillate the saturation
        s: triangle(),
        // max value by default
        v: 127i8,
    };

    StreakSpawner::new(
        random_pulse(Constant::<5>, Constant::<{ i8::MIN }>), // randomly spawn streaks
        rng().max(Constant::<2>),                             // randomize lengths
        rng(),                                                // randomize velocities
        Constant::<{ SPINE_LEN as i8 }>,                      // Total LEDs in spine
        osc,
        [crate::streak::StreakState::default(); 8],
    )
}

pub fn spines<const LEN: usize>() -> impl Pattern {
    let storage: [_; LEN] = core::array::from_fn(|v| {
        let v = (v + SPINE_COUNT as usize / 2 - 1) % SPINE_COUNT as usize;
        spine_pattern(v as _)
    });
    PerSpine::new(storage)
}
