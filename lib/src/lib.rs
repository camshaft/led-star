#![cfg_attr(not(any(feature = "std", test)), no_std)]

macro_rules! assume {
    ($cond:expr) => {
        debug_assert!($cond);
        if cfg!(not(debug_assertions)) && !$cond {
            core::hint::unreachable_unchecked();
        }
    };
}

pub mod color;
pub mod config;
pub mod osc;
pub mod pattern;
pub mod rand;
pub mod slotmap;
pub mod star;
pub mod storage;
pub mod streak;

pub use pattern::*;
