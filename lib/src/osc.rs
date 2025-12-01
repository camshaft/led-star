mod lut;

pub type Value = i8;

pub trait Oscillator {
    fn tick(&mut self);
    fn get(&self) -> Value;
}

macro_rules! impl_binary_ext {
    ($fun:ident, $name:ident) => {
        #[inline(always)]
        fn $fun<R: Oscillator>(self, rhs: R) -> $name<Self, R>
        where
            Self: Sized,
        {
            $name::new(self, rhs)
        }
    };
}

macro_rules! impl_unary_ext {
    ($fun:ident, $name:ident) => {
        #[inline(always)]
        fn $fun(self) -> $name<Self>
        where
            Self: Sized,
        {
            $name::new(self)
        }
    };
}

pub trait OscillatorExt: Oscillator {
    impl_binary_ext!(add, Add);
    impl_binary_ext!(saturating_add, SaturatingAdd);
    impl_binary_ext!(sub, Sub);
    impl_binary_ext!(saturating_sub, SaturatingSub);
    impl_binary_ext!(mul, Mul);
    impl_binary_ext!(saturating_mul, SaturatingMul);
    impl_binary_ext!(div, Div);
    impl_binary_ext!(saturating_div, SaturatingDiv);
    impl_binary_ext!(rem, Rem);
    impl_binary_ext!(max, Max);
    impl_binary_ext!(min, Min);

    impl_unary_ext!(neg, Neg);

    impl_binary_ext!(freq, WithFrequency);
}

impl<T: Oscillator> OscillatorExt for T {}

#[derive(Debug)]
pub struct Constant<const V: Value>;

impl<const V: Value> Oscillator for Constant<V> {
    #[inline(always)]
    fn tick(&mut self) {}

    #[inline(always)]
    fn get(&self) -> Value {
        V
    }
}

impl Oscillator for Value {
    #[inline(always)]
    fn tick(&mut self) {}

    #[inline(always)]
    fn get(&self) -> Value {
        *self
    }
}

pub fn triangle() -> Triangle {
    Triangle::new()
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    counter: Value,
    direction: bool,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            counter: 0,
            direction: true,
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new()
    }
}

impl Oscillator for Triangle {
    #[inline(always)]
    fn tick(&mut self) {
        if self.direction {
            self.counter = self.counter.wrapping_add(1);
            if self.counter == Value::MAX {
                self.direction = false;
            }
        } else {
            self.counter = self.counter.wrapping_sub(1);
            if self.counter == Value::MIN {
                self.direction = true;
            }
        }
    }

    #[inline(always)]
    fn get(&self) -> Value {
        self.counter
    }
}

pub fn sawtooth() -> Sawtooth {
    Sawtooth::new()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sawtooth {
    counter: Value,
}

impl Sawtooth {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl Oscillator for Sawtooth {
    #[inline(always)]
    fn tick(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    #[inline(always)]
    fn get(&self) -> Value {
        self.counter
    }
}

pub fn square<D: Oscillator>(duty_cycle: D) -> Square<D> {
    Square::new(duty_cycle)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Square<D> {
    counter: Value,
    duty_cycle: D,
}

impl<D: Oscillator> Square<D> {
    pub fn new(duty_cycle: D) -> Self {
        Self {
            counter: 0,
            duty_cycle,
        }
    }
}

impl<D: Oscillator> Oscillator for Square<D> {
    #[inline(always)]
    fn tick(&mut self) {
        self.counter = self.counter.wrapping_add(1);
        self.duty_cycle.tick();
    }

    #[inline(always)]
    fn get(&self) -> Value {
        if self.counter < self.duty_cycle.get() {
            Value::MIN
        } else {
            Value::MAX
        }
    }
}

pub fn sine() -> Sine {
    Sine::new()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sine {
    counter: u8,
}

impl Sine {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn phase(counter: u8) -> Self {
        Self { counter }
    }
}

impl Oscillator for Sine {
    #[inline(always)]
    fn tick(&mut self) {
        self.counter = self.counter.wrapping_add(1);
    }

    #[inline(always)]
    fn get(&self) -> Value {
        match self.counter {
            0..64 => lut::SINE[self.counter as usize],
            64..128 => lut::SINE[127 - self.counter as usize],
            128..192 => -lut::SINE[self.counter as usize - 128],
            _ => -lut::SINE[255 - self.counter as usize],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Neg<O> {
    inner: O,
}

impl<O> Neg<O> {
    pub fn new(inner: O) -> Self {
        Self { inner }
    }
}

impl<O: Oscillator> Oscillator for Neg<O> {
    #[inline(always)]
    fn tick(&mut self) {
        self.inner.tick();
    }

    #[inline(always)]
    fn get(&self) -> Value {
        let value = self.inner.get();
        if value == Value::MIN {
            Value::MAX
        } else {
            -value
        }
    }
}

mod math {
    use super::{Oscillator, Value};

    macro_rules! impl_math {
        ($name:ident, $op:ident) => {
            #[derive(Clone, Copy, Debug, Default)]
            pub struct $name<A, B>(pub A, pub B);

            impl<A, B> $name<A, B> {
                pub fn new(a: A, b: B) -> Self {
                    Self(a, b)
                }
            }

            impl<A, B> Oscillator for $name<A, B>
            where
                A: Oscillator,
                B: Oscillator,
            {
                #[inline(always)]
                fn tick(&mut self) {
                    self.0.tick();
                    self.1.tick();
                }

                #[inline(always)]
                fn get(&self) -> Value {
                    self.0.get().$op(self.1.get())
                }
            }
        };
    }

    impl_math!(Add, wrapping_add);
    impl_math!(SaturatingAdd, saturating_add);
    impl_math!(Sub, wrapping_sub);
    impl_math!(SaturatingSub, saturating_sub);
    impl_math!(Mul, wrapping_mul);
    impl_math!(SaturatingMul, saturating_mul);
    impl_math!(Div, wrapping_div);
    impl_math!(SaturatingDiv, saturating_div);
    impl_math!(Rem, wrapping_rem);
    impl_math!(Max, max);
    impl_math!(Min, min);
}

pub use math::*;

pub const fn rng() -> Rng {
    Rng
}

/// Random number generator oscillator
/// Returns a random i8 value on each get()
#[derive(Clone, Copy, Debug, Default)]
pub struct Rng;

impl Oscillator for Rng {
    #[inline(always)]
    fn tick(&mut self) {
        // No-op, randomness happens on get()
    }

    #[inline(always)]
    fn get(&self) -> Value {
        crate::rand::i8()
    }
}

pub fn random_pulse<Min: Oscillator, Max: Oscillator>(
    min_count: Min,
    max_count: Max,
) -> RandomPulse<Min, Max> {
    RandomPulse::new(min_count, max_count)
}

/// Random pulse oscillator
/// Counts down randomly and emits 127 when reaching 0
#[derive(Clone, Copy, Debug)]
pub struct RandomPulse<Min, Max> {
    counter: u8,
    min_count: Min,
    max_count: Max,
}

impl<Min, Max> RandomPulse<Min, Max>
where
    Min: Oscillator,
    Max: Oscillator,
{
    pub fn new(min_count: Min, max_count: Max) -> Self {
        let mut osc = Self {
            counter: 0,
            min_count,
            max_count,
        };
        osc.tick();
        osc
    }
}

impl<Min, Max> Oscillator for RandomPulse<Min, Max>
where
    Min: Oscillator,
    Max: Oscillator,
{
    fn tick(&mut self) {
        self.min_count.tick();
        self.max_count.tick();

        if self.counter > 0 {
            self.counter -= 1;
        } else {
            // Reset with new random count
            let min_count = self.min_count.get() as u8;
            let max_count = self.max_count.get() as u8;
            self.counter = crate::rand::range_u8(min_count, max_count);
        }
    }

    fn get(&self) -> Value {
        if self.counter == 0 { 127 } else { 0 }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WithFrequency<O, V> {
    inner: O,
    frequency: V,
    clock: FrequencyClock,
}

impl<O, V> WithFrequency<O, V> {
    pub fn new(inner: O, frequency: V) -> Self {
        Self {
            inner,
            frequency,
            clock: FrequencyClock::default(),
        }
    }
}

impl<O, V> Oscillator for WithFrequency<O, V>
where
    O: Oscillator,
    V: Oscillator,
{
    fn tick(&mut self) {
        self.frequency.tick();
        let ticks = self.clock.tick(self.frequency.get());
        for _ in 0..ticks {
            self.inner.tick();
        }
    }

    fn get(&self) -> Value {
        self.inner.get()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct FrequencyClock {
    frac: u8,
}

impl FrequencyClock {
    fn tick(&mut self, freq: Value) -> u8 {
        // If the frequency is set to 0 then it's the regular rate
        if freq == 0 {
            return 1;
        }

        // If the frequency is positive then its multiplied by the regular rate.
        // Otherwise it is divided.
        let is_positive = freq > 0;
        // Use wrapping_abs to handle i8::MIN (-128) safely
        // For i8::MIN, wrapping_abs returns -128 (0x80), which as u8 is 128
        let freq_abs = freq.wrapping_abs() as u8;

        // Use a 4.4 fixed-point format for better precision
        // Base speed: 16 (represents 1.0x, since 16 >> 4 = 1)
        // Max speed: 64 (represents 4.0x, since 64 >> 4 = 4)
        // Min speed: 4 (represents 0.25x, since 4 >> 4 = 0.25)
        // This gives us 1/16 = 0.0625x precision

        let increment: u8 = if is_positive {
            // Linear scale from 16 (1x) to 64 (4x)
            // increment = 16 + (freq_abs * 48) / 127
            // where 48 = 16 * (DIVISIONS - 1)
            // freq_abs * 48 fits in u16 (max 6096)
            let scale = (freq_abs as u16 * 48 / i8::MAX as u16) as u8;
            16 + scale
        } else {
            // Linear scale from 16 (1x) to 4 (0.25x)
            // increment = 16 - (freq_abs * 12) / 128
            // where 12 = 16 * (DIVISIONS - 1) / DIVISIONS
            // freq_abs * 12 fits in u16 (max 1536)
            let scale = (freq_abs as u16 * 12 / 128) as u8;
            16 - scale
        };

        // Phase accumulation: add increment to fractional accumulator
        // With 4.4 format, frac holds 0-15, increment is 4-64
        let accumulated = self.frac + increment;

        // Extract integer ticks (top 4 bits via >>4)
        let ticks = accumulated >> 4;

        // Keep fractional part (bottom 4 bits)
        self.frac = accumulated & 0xF;

        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        let mut osc = Constant::<42>;
        assert_eq!(osc.get(), 42);
        osc.tick();
        assert_eq!(osc.get(), 42);
    }

    #[test]
    fn test_value_oscillator() {
        let mut osc: Value = 10;
        assert_eq!(osc.get(), 10);
        osc.tick();
        assert_eq!(osc.get(), 10);
    }

    #[test]
    fn test_triangle() {
        let mut tri = Triangle::default();

        // Start at 0
        assert_eq!(tri.get(), 0);

        // Should count up
        tri.tick();
        assert_eq!(tri.get(), 1);
        tri.tick();
        assert_eq!(tri.get(), 2);

        // Continue to max
        for _ in 0..125 {
            tri.tick();
        }
        assert_eq!(tri.get(), 127);

        // At max, direction reverses - should count back down
        tri.tick();
        assert_eq!(tri.get(), 126);
        tri.tick();
        assert_eq!(tri.get(), 125);

        // Continue down to min
        for _ in 0..253 {
            tri.tick();
        }
        assert_eq!(tri.get(), -128);

        // At min, direction reverses - should count back up
        tri.tick();
        assert_eq!(tri.get(), -127);
        tri.tick();
        assert_eq!(tri.get(), -126);
    }

    #[test]
    fn test_sawtooth() {
        let mut saw = Sawtooth { counter: 0 };

        assert_eq!(saw.get(), 0);
        saw.tick();
        assert_eq!(saw.get(), 1);

        // Should count up continuously
        for i in 2..=127 {
            saw.tick();
            assert_eq!(saw.get(), i);
        }

        // Wrap around
        saw.tick();
        assert_eq!(saw.get(), -128);
    }

    #[test]
    fn test_square() {
        let mut sq = Square {
            counter: 0,
            duty_cycle: Constant::<0>,
        };

        // With duty cycle at 0, should always be MAX
        assert_eq!(sq.get(), Value::MAX);
        sq.tick();
        assert_eq!(sq.get(), Value::MAX);

        let mut sq = Square {
            counter: 0,
            duty_cycle: Constant::<64>,
        };

        // First half should be MIN
        assert_eq!(sq.get(), Value::MIN);
        for _ in 0..63 {
            sq.tick();
            assert_eq!(sq.get(), Value::MIN);
        }

        // Second half should be MAX
        sq.tick();
        assert_eq!(sq.get(), Value::MAX);
    }

    #[test]
    fn test_sine() {
        let mut sine = Sine { counter: 0 };

        // At 0 degrees, sine should be 0
        assert_eq!(sine.get(), 0);

        // At 64 (90 degrees), should be at max
        for _ in 0..64 {
            sine.tick();
        }
        assert_eq!(sine.get(), 127);

        // At 128 (180 degrees), should be back to 0
        for _ in 0..64 {
            sine.tick();
        }
        assert_eq!(sine.get(), 0);

        // At 192 (270 degrees), should be at min
        for _ in 0..64 {
            sine.tick();
        }
        assert_eq!(sine.get(), -127);
    }

    #[test]
    fn test_sine_multiple_cycles_calculated() {
        let mut sine = Sine::new();

        // Test 3 complete cycles
        for cycle in 0..3 {
            for step in 0..256 {
                let actual = sine.get();

                // Calculate expected value using f32
                // counter ranges from 0-255, representing 0-360 degrees
                let angle = (step as f32 / 256.0) * 2.0 * std::f32::consts::PI;
                let sine_value = angle.sin();
                let expected = (sine_value * 127.0).round() as i8;

                // Allow for quantization error from the 64-entry LUT (±2)
                let diff = (actual - expected).abs();
                assert!(
                    diff <= 4,
                    "Cycle {}, step {}: expected {} (angle: {:.2}°), got {} (diff: {})",
                    cycle,
                    step,
                    expected,
                    step as f32 * 360.0 / 256.0,
                    actual,
                    diff
                );

                sine.tick();
            }
        }

        // Verify the oscillator wrapped correctly back to start
        assert_eq!(
            sine.get(),
            0,
            "After 3 complete cycles, should be back at 0"
        );
    }

    #[test]
    fn test_neg() {
        let mut inv = Neg {
            inner: Constant::<42>,
        };

        assert_eq!(inv.get(), -42);
        inv.tick();
        assert_eq!(inv.get(), -42);

        let mut inv = Neg {
            inner: Sawtooth { counter: 10 },
        };

        assert_eq!(inv.get(), -10);
        inv.tick();
        assert_eq!(inv.get(), -11);
    }

    #[test]
    fn test_add() {
        let mut add = Add(Constant::<10>, Constant::<20>);

        assert_eq!(add.get(), 30);
        add.tick();
        assert_eq!(add.get(), 30);

        // Test wrapping
        let add = Add(Constant::<100>, Constant::<100>);
        assert_eq!(add.get(), -56); // 200 wraps to -56
    }

    #[test]
    fn test_sub() {
        let sub = Sub(Constant::<50>, Constant::<20>);

        assert_eq!(sub.get(), 30);
    }

    #[test]
    fn test_mul() {
        let mul = Mul(Constant::<10>, Constant::<5>);

        assert_eq!(mul.get(), 50);
    }

    #[test]
    fn test_frequency_clock_zero() {
        let mut clock = FrequencyClock::default();

        // At frequency 0, should tick once per call
        assert_eq!(clock.tick(0), 1);
        assert_eq!(clock.tick(0), 1);
        assert_eq!(clock.tick(0), 1);
    }

    #[test]
    fn test_frequency_clock_positive() {
        let mut clock = FrequencyClock::default();

        // At max positive frequency (127), should tick ~4x per call
        let mut total_ticks = 0;
        for _ in 0..10 {
            total_ticks += clock.tick(i8::MAX) as u32;
        }

        // Should be close to 40 ticks (4x speed over 10 calls)
        // Allow some tolerance due to fixed-point rounding
        assert!(
            total_ticks >= 38 && total_ticks <= 42,
            "Got {} ticks",
            total_ticks
        );
    }

    #[test]
    fn test_frequency_clock_negative() {
        let mut clock = FrequencyClock::default();

        // At max negative frequency (-128), should tick ~0.25x per call
        let mut total_ticks = 0;
        for _ in 0..100 {
            total_ticks += clock.tick(i8::MIN) as u32;
        }

        // Should be close to 25 ticks (0.25x speed over 100 calls)
        // Allow some tolerance due to fixed-point rounding
        assert!(
            total_ticks >= 23 && total_ticks <= 27,
            "Got {} ticks",
            total_ticks
        );
    }

    #[test]
    fn test_frequency_clock_mid_positive() {
        let mut clock = FrequencyClock::default();

        // At mid-range positive frequency (64), should tick ~2.5x per call
        let mut total_ticks = 0;
        for _ in 0..10 {
            total_ticks += clock.tick(64) as u32;
        }

        // Should be close to 25 ticks (2.5x speed over 10 calls)
        assert!(
            total_ticks >= 23 && total_ticks <= 27,
            "Got {} ticks",
            total_ticks
        );
    }

    #[test]
    fn test_frequency_clock_mid_negative() {
        let mut clock = FrequencyClock::default();

        // At mid-range negative frequency (-64), should tick ~0.625x per call
        let mut total_ticks = 0;
        for _ in 0..100 {
            total_ticks += clock.tick(-64) as u32;
        }

        // Should be close to 62-63 ticks (0.625x speed over 100 calls)
        assert!(
            total_ticks >= 60 && total_ticks <= 65,
            "Got {} ticks",
            total_ticks
        );
    }

    #[test]
    fn test_with_frequency() {
        let mut osc = WithFrequency {
            inner: Sawtooth { counter: 0 },
            frequency: Constant::<0>,
            clock: FrequencyClock::default(),
        };

        // At frequency 0, should advance normally
        assert_eq!(osc.get(), 0);
        osc.tick();
        assert_eq!(osc.get(), 1);
        osc.tick();
        assert_eq!(osc.get(), 2);
    }

    #[test]
    fn test_with_frequency_fast() {
        let mut osc = WithFrequency {
            inner: Sawtooth { counter: 0 },
            frequency: Constant::<127>,
            clock: FrequencyClock::default(),
        };

        // At max frequency, should advance 4x faster
        for _ in 0..256 {
            let before = osc.get();
            osc.tick();
            let after = osc.get();
            assert_eq!(after, before.wrapping_add(4));
        }
    }

    #[test]
    fn test_with_frequency_slow() {
        let mut osc = WithFrequency {
            inner: Sawtooth { counter: 0 },
            frequency: Constant::<-128>,
            clock: FrequencyClock::default(),
        };

        // At min frequency, should advance at 0.25x speed
        for i in 0..(256 * 5) {
            let before = osc.get();
            osc.tick();
            let after = osc.get();
            if i % 4 == 3 {
                assert_eq!(after, before.wrapping_add(1));
            } else {
                assert_eq!(after, before);
            }
        }
    }

    #[test]
    fn test_rng() {
        crate::rand::seed(42);
        let mut rng = Rng;

        // Should produce different values
        let a = rng.get();
        let b = rng.get();
        let c = rng.get();

        // Very unlikely to get three identical values
        assert!(a != b || b != c);

        // tick() should be a no-op
        rng.tick();
        let d = rng.get();
        assert!(d != c); // Still producing different values
    }

    #[test]
    fn test_random_pulse() {
        crate::rand::seed(123);
        let mut pulse = RandomPulse::new(5, 10);

        let mut pulse_count = 0;
        let mut zero_count = 0;

        // Run for several cycles
        for _ in 0..100 {
            let val = pulse.get();
            if val == 127 {
                pulse_count += 1;
            } else if val == 0 {
                zero_count += 1;
            }
            pulse.tick();
        }

        // Should have emitted some pulses
        assert!(pulse_count > 0);
        assert!(zero_count > 0);
        // Most ticks should be zero (counting down)
        assert!(zero_count > pulse_count);
    }
}
