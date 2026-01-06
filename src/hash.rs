use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::ops::BitXor;

/// FxHasher - a fast, non-cryptographic hasher (same algorithm as rustc-hash)
#[derive(Default)]
pub struct FxHasher {
    hash: usize,
}

const K: usize = 0x517cc1b727220a95;

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash = self
                .hash
                .rotate_left(5)
                .bitxor(byte as usize)
                .wrapping_mul(K);
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.hash = self.hash.rotate_left(5).bitxor(i as usize).wrapping_mul(K);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.hash = self.hash.rotate_left(5).bitxor(i as usize).wrapping_mul(K);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.hash = self.hash.rotate_left(5).bitxor(i as usize).wrapping_mul(K);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.hash = self.hash.rotate_left(5).bitxor(i as usize).wrapping_mul(K);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash as u64
    }
}

pub type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

