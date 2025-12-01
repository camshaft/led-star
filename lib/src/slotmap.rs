use crate::storage::Storage;
use core::fmt;

/// A slot-based storage structure with occupied tracking via bitset
/// Supports up to 8 slots (limited by u8 bitset)
pub struct SlotMap<V, S, const MAX_SLOTS: usize>
where
    V: Copy,
    S: Storage<Value = V>,
{
    storage: S,
    occupied: u8, // Bitset tracking which slots are occupied
}

impl<V, S, const MAX_SLOTS: usize> fmt::Debug for SlotMap<V, S, MAX_SLOTS>
where
    V: Copy + fmt::Debug,
    S: Storage<Value = V>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<V, S, const MAX_SLOTS: usize> SlotMap<V, S, MAX_SLOTS>
where
    V: Copy,
    S: Storage<Value = V>,
{
    /// Create a new SlotMap with the given storage
    pub fn new(storage: S) -> Self {
        const {
            if MAX_SLOTS > 8 {
                panic!("SlotMap supports at most 8 slots");
            }
        }
        Self {
            storage,
            occupied: 0,
        }
    }

    /// Insert a value into the first available slot
    /// Returns the slot index if successful, None if full
    pub fn insert(&mut self, value: V) -> Option<u8> {
        if self.is_full() {
            return None;
        }

        // Find first empty slot by finding the first 0 bit
        // Invert the occupied bitset and find trailing zeros
        let inverted = !self.occupied;
        let index = inverted.trailing_zeros() as u8;

        debug_assert!(index < MAX_SLOTS as u8, "index out of bounds");

        self.occupied |= 1 << index;
        self.storage.set(index, value);
        Some(index)
    }

    /// Remove a value from the specified slot
    #[inline(always)]
    pub fn remove(&mut self, index: u8) {
        debug_assert!((index as usize) < MAX_SLOTS);
        self.occupied &= !(1 << index);
    }

    /// Check if the slot map is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.occupied == 0
    }

    /// Check if the slot map is full
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        let mask = const {
            if MAX_SLOTS == 8 {
                0xff
            } else {
                (1u8 << MAX_SLOTS) - 1
            }
        };
        self.occupied == mask
    }

    #[inline(always)]
    pub fn len(&self) -> u8 {
        self.occupied.count_ones() as u8
    }

    /// Iterate over occupied slots, yielding (index, value) pairs
    pub fn iter(&self) -> impl Iterator<Item = &V> + '_ {
        let occupied = self.occupied;
        let len = self.len() as usize;
        self.storage
            .iter()
            .enumerate()
            .filter_map(move |(i, v)| {
                if (occupied & (1 << i)) != 0 {
                    Some(v)
                } else {
                    None
                }
            })
            .take(len)
    }

    /// Iterate mutably over occupied slots, yielding (index, value) pairs
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> + '_ {
        let occupied = self.occupied;
        let len = self.len() as usize;
        self.storage
            .iter_mut()
            .enumerate()
            .filter_map(move |(i, v)| {
                if (occupied & (1 << i)) != 0 {
                    Some(v)
                } else {
                    None
                }
            })
            .take(len)
    }

    /// Retain only the slots for which the predicate returns true
    /// The predicate receives (index, value) and returns whether to keep the slot
    pub fn retain(&mut self, mut f: impl FnMut(&mut V) -> bool) {
        let mut remaining = self.len();
        for i in 0..MAX_SLOTS as u8 {
            if remaining == 0 {
                break;
            }
            if (self.occupied & (1 << i)) != 0 {
                if !f(self.storage.get_mut(i)) {
                    self.remove(i);
                }
                remaining -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slotmap_basic() {
        let mut map = SlotMap::<u8, [u8; 4], 4>::new([0; 4]);

        assert!(map.is_empty());
        assert!(!map.is_full());

        let idx0 = map.insert(10).unwrap();
        assert_eq!(idx0, 0);
        assert!(!map.is_empty());
        assert!(!map.is_full());

        let idx1 = map.insert(20).unwrap();
        assert_eq!(idx1, 1);

        let idx2 = map.insert(30).unwrap();
        assert_eq!(idx2, 2);

        let idx3 = map.insert(40).unwrap();
        assert_eq!(idx3, 3);

        assert!(map.is_full());
        assert!(map.insert(50).is_none());
    }

    #[test]
    fn test_slotmap_remove() {
        let mut map = SlotMap::<u8, [u8; 4], 4>::new([0; 4]);

        map.insert(10);
        map.insert(20);
        map.insert(30);

        map.remove(1);
        assert!(!map.is_full());

        // Should reuse slot 1
        let idx = map.insert(40).unwrap();
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_slotmap_iter() {
        let mut map = SlotMap::<u8, [u8; 4], 4>::new([0; 4]);

        map.insert(10);
        map.insert(20);
        map.insert(30);

        let values: Vec<_> = map.iter().copied().collect();
        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn test_slotmap_retain() {
        let mut map = SlotMap::<u8, [u8; 4], 4>::new([0; 4]);

        map.insert(10);
        map.insert(20);
        map.insert(30);
        map.insert(40);

        // Remove values > 25
        map.retain(|v| *v <= 25);

        let values: Vec<_> = map.iter().copied().collect();
        assert_eq!(values, vec![10, 20]);
    }
}
