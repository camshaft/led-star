// Simple LCG (Linear Congruential Generator) RNG
// Using 16-bit state for Arduino compatibility
const RNG_A: u16 = 25173;
const RNG_C: u16 = 13849;

#[cfg(test)]
thread_local! {
    static RNG_STATE: core::cell::Cell<u16> = const { core::cell::Cell::new(1) };
}

#[cfg(not(test))]
static mut RNG_STATE: u16 = 1;

/// Access RNG state with a closure
#[inline(always)]
fn with_state<R>(f: impl FnOnce(&mut u16) -> R) -> R {
    #[cfg(test)]
    {
        RNG_STATE.with(|state| {
            let mut val = state.get();
            let result = f(&mut val);
            state.set(val);
            result
        })
    }
    #[cfg(not(test))]
    unsafe {
        let mut v = RNG_STATE;
        let result = f(&mut v);
        RNG_STATE = v;
        result
    }
}

/// Seed the RNG with a value
pub fn seed(seed: u16) {
    with_state(|state| *state = seed);
}

/// Generate next random u16
fn next() -> u16 {
    with_state(|state| {
        *state = RNG_A.wrapping_mul(*state).wrapping_add(RNG_C);
        *state
    })
}

/// Generate random i8
pub fn i8() -> i8 {
    let val = next();
    // Use top 8 bits for better distribution
    (val >> 8) as i8
}

/// Generate random u8 in range [min, max] inclusive
pub fn range_u8(min: u8, max: u8) -> u8 {
    if min >= max {
        return min;
    }
    let range = (max - min) + 1;
    let val = (next() >> 8) as u8;
    min + (val % range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_deterministic() {
        seed(12345);
        let a = i8();
        let b = i8();

        seed(12345);
        let c = i8();
        let d = i8();

        assert_eq!(a, c);
        assert_eq!(b, d);
    }

    #[test]
    fn test_range_u8() {
        seed(42);
        for _ in 0..100 {
            let val = range_u8(10, 20);
            assert!((10..=20).contains(&val));
        }
    }

    #[test]
    fn test_range_u8_single() {
        let val = range_u8(5, 5);
        assert_eq!(val, 5);
    }
}
