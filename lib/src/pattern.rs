use crate::{color::Hsv, osc, storage::Storage};

#[derive(Clone, Copy)]
pub struct Index {
    pub index: u8,
    pub total: u8,
}

pub trait Pattern {
    fn tick(&mut self);
    fn spine_color_at(&self, spine: Index, led: Index) -> Hsv;
    fn spine_tip_color_at(&self, spine: Index, led: Index) -> Hsv;
    fn arc_color_at(&self, arc: Index, led: Index) -> Hsv;
}

impl Pattern for Hsv {
    #[inline(always)]
    fn tick(&mut self) {}

    #[inline(always)]
    fn spine_color_at(&self, _spine: Index, _led: Index) -> Hsv {
        *self
    }

    #[inline(always)]
    fn spine_tip_color_at(&self, _spine: Index, _led: Index) -> Hsv {
        *self
    }

    #[inline(always)]
    fn arc_color_at(&self, _arc: Index, _led: Index) -> Hsv {
        *self
    }
}

#[cfg(any(test, feature = "std"))]
impl<T: ?Sized + Pattern> Pattern for Box<T> {
    #[inline(always)]
    fn tick(&mut self) {
        (**self).tick();
    }

    #[inline(always)]
    fn spine_color_at(&self, spine: Index, led: Index) -> Hsv {
        (**self).spine_color_at(spine, led)
    }

    #[inline(always)]
    fn spine_tip_color_at(&self, spine: Index, led: Index) -> Hsv {
        (**self).spine_tip_color_at(spine, led)
    }

    #[inline(always)]
    fn arc_color_at(&self, arc: Index, led: Index) -> Hsv {
        (**self).arc_color_at(arc, led)
    }
}

pub struct Compound<Spine, Tip, Arc>
where
    Spine: Pattern,
    Tip: Pattern,
    Arc: Pattern,
{
    pub spine: Spine,
    pub tip: Tip,
    pub arc: Arc,
}

impl<Spine, Tip, Arc> Pattern for Compound<Spine, Tip, Arc>
where
    Spine: Pattern,
    Tip: Pattern,
    Arc: Pattern,
{
    #[inline(always)]
    fn tick(&mut self) {
        self.spine.tick();
        self.tip.tick();
        self.arc.tick();
    }

    #[inline(always)]
    fn spine_color_at(&self, spine: Index, led: Index) -> Hsv {
        self.spine.spine_color_at(spine, led)
    }

    #[inline(always)]
    fn spine_tip_color_at(&self, spine: Index, led: Index) -> Hsv {
        self.tip.spine_tip_color_at(spine, led)
    }

    #[inline(always)]
    fn arc_color_at(&self, arc: Index, led: Index) -> Hsv {
        self.arc.arc_color_at(arc, led)
    }
}

pub struct PerSpine<V>
where
    V: Storage,
    V::Value: Pattern,
{
    pub values: V,
}

impl<V> PerSpine<V>
where
    V: Storage,
    V::Value: Pattern,
{
    pub fn new(values: V) -> Self {
        PerSpine { values }
    }
}

impl<V> Pattern for PerSpine<V>
where
    V: Storage,
    V::Value: Pattern,
{
    #[inline(always)]
    fn tick(&mut self) {
        for v in self.values.iter_mut() {
            v.tick();
        }
    }

    #[inline(always)]
    fn spine_color_at(&self, spine: Index, led: Index) -> Hsv {
        self.values.get(spine.index).spine_color_at(spine, led)
    }

    #[inline(always)]
    fn spine_tip_color_at(&self, spine: Index, led: Index) -> Hsv {
        self.values.get(spine.index).spine_tip_color_at(spine, led)
    }

    #[inline(always)]
    fn arc_color_at(&self, arc: Index, led: Index) -> Hsv {
        self.values.get(arc.index).arc_color_at(arc, led)
    }
}

pub struct Oscillator<H, S, V> {
    pub h: H,
    pub s: S,
    pub v: V,
}

impl<H, S, V> Oscillator<H, S, V>
where
    H: osc::Oscillator,
    S: osc::Oscillator,
    V: osc::Oscillator,
{
    #[inline(always)]
    fn get(&self) -> Hsv {
        let scale = |v: i8| -> u8 {
            // Scale the oscillator signed to unsigned
            let v = (v as u8) + 128;
            v
        };
        let h = scale(self.h.get());
        let s = scale(self.s.get());
        let v = scale(self.v.get());
        Hsv::new(h, s, v)
    }
}

impl<H, S, V> Pattern for Oscillator<H, S, V>
where
    H: osc::Oscillator,
    S: osc::Oscillator,
    V: osc::Oscillator,
{
    #[inline(always)]
    fn tick(&mut self) {
        self.h.tick();
        self.s.tick();
        self.v.tick();
    }

    #[inline(always)]
    fn spine_color_at(&self, _spine: Index, _led: Index) -> Hsv {
        self.get()
    }

    #[inline(always)]
    fn spine_tip_color_at(&self, _spine: Index, _led: Index) -> Hsv {
        self.get()
    }

    #[inline(always)]
    fn arc_color_at(&self, _arc: Index, _led: Index) -> Hsv {
        self.get()
    }
}
