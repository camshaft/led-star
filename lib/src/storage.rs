pub type Index = u8;

pub trait Storage {
    type Value;

    fn len(&self) -> Index;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, index: Index) -> &Self::Value;
    fn get_mut(&mut self, index: Index) -> &mut Self::Value;
    fn set(&mut self, index: Index, value: Self::Value);
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Self::Value> + 'a
    where
        Self::Value: 'a;
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Self::Value> + 'a
    where
        Self::Value: 'a;
}

impl<V> Storage for [V] {
    type Value = V;

    #[inline(always)]
    fn len(&self) -> u8 {
        u8::try_from(self.len()).unwrap()
    }

    #[inline(always)]
    fn get(&self, index: u8) -> &V {
        &self[index as usize]
    }

    #[inline(always)]
    fn get_mut(&mut self, index: u8) -> &mut V {
        &mut self[index as usize]
    }

    #[inline(always)]
    fn set(&mut self, index: u8, value: V) {
        self[index as usize] = value;
    }

    #[inline(always)]
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a
    where
        V: 'a,
    {
        self[..].iter()
    }

    #[inline(always)]
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a
    where
        V: 'a,
    {
        self[..].iter_mut()
    }
}

impl<V, const N: usize> Storage for [V; N] {
    type Value = V;

    #[inline(always)]
    fn len(&self) -> u8 {
        const {
            if N == 0 {
                panic!("Cannot create a storage of length 0");
            }
            if N > u8::MAX as usize {
                panic!("Cannot create a storage of length greater than u8::MAX");
            }
        }
        N as u8
    }

    #[inline(always)]
    fn get(&self, index: u8) -> &V {
        &self[index as usize]
    }

    #[inline(always)]
    fn get_mut(&mut self, index: u8) -> &mut V {
        &mut self[index as usize]
    }

    #[inline(always)]
    fn set(&mut self, index: u8, value: V) {
        self[index as usize] = value;
    }

    #[inline(always)]
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a
    where
        V: 'a,
    {
        self[..].iter()
    }

    #[inline(always)]
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a
    where
        V: 'a,
    {
        self[..].iter_mut()
    }
}

#[cfg(any(test, feature = "std"))]
impl<V> Storage for Vec<V> {
    type Value = V;

    #[inline(always)]
    fn len(&self) -> u8 {
        Vec::len(self).try_into().unwrap()
    }

    #[inline(always)]
    fn get(&self, index: u8) -> &V {
        &self[index as usize]
    }

    #[inline(always)]
    fn get_mut(&mut self, index: u8) -> &mut V {
        &mut self[index as usize]
    }

    #[inline(always)]
    fn set(&mut self, index: u8, value: V) {
        self[index as usize] = value;
    }

    #[inline(always)]
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a
    where
        V: 'a,
    {
        self[..].iter()
    }

    #[inline(always)]
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a
    where
        V: 'a,
    {
        self[..].iter_mut()
    }
}

pub struct Cell<V>(pub V);

impl<V> Cell<V> {
    pub fn new(value: V) -> Self {
        Self(value)
    }
}

impl<V> Storage for Cell<V> {
    type Value = V;

    #[inline(always)]
    fn len(&self) -> u8 {
        1
    }

    #[inline(always)]
    fn get(&self, _index: u8) -> &V {
        &self.0
    }

    #[inline(always)]
    fn get_mut(&mut self, _index: u8) -> &mut V {
        &mut self.0
    }

    #[inline(always)]
    fn set(&mut self, _index: u8, value: V) {
        self.0 = value;
    }

    #[inline(always)]
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a
    where
        V: 'a,
    {
        core::iter::once(&self.0)
    }

    #[inline(always)]
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a
    where
        V: 'a,
    {
        core::iter::once(&mut self.0)
    }
}
