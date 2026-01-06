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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hasher_deterministic() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write(b"hello");
        h2.write(b"hello");
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_hasher_different_inputs() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write(b"hello");
        h2.write(b"world");
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_hasher_produces_nonzero() {
        let mut h = FxHasher::default();
        h.write(b"test");
        assert_ne!(h.finish(), 0);
    }

    #[test]
    fn test_write_u8() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write_u8(42);
        h2.write_u8(42);
        assert_eq!(h1.finish(), h2.finish());

        let mut h3 = FxHasher::default();
        h3.write_u8(43);
        assert_ne!(h1.finish(), h3.finish());
    }

    #[test]
    fn test_write_u16() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write_u16(1000);
        h2.write_u16(1000);
        assert_eq!(h1.finish(), h2.finish());

        let mut h3 = FxHasher::default();
        h3.write_u16(1001);
        assert_ne!(h1.finish(), h3.finish());
    }

    #[test]
    fn test_write_u32() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write_u32(100_000);
        h2.write_u32(100_000);
        assert_eq!(h1.finish(), h2.finish());

        let mut h3 = FxHasher::default();
        h3.write_u32(100_001);
        assert_ne!(h1.finish(), h3.finish());
    }

    #[test]
    fn test_write_u64() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write_u64(10_000_000_000);
        h2.write_u64(10_000_000_000);
        assert_eq!(h1.finish(), h2.finish());

        let mut h3 = FxHasher::default();
        h3.write_u64(10_000_000_001);
        assert_ne!(h1.finish(), h3.finish());
    }

    #[test]
    fn test_write_usize() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write_usize(999_999);
        h2.write_usize(999_999);
        assert_eq!(h1.finish(), h2.finish());

        let mut h3 = FxHasher::default();
        h3.write_usize(999_998);
        assert_ne!(h1.finish(), h3.finish());
    }

    #[test]
    fn test_sequential_writes() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write_u8(1);
        h1.write_u16(2);
        h1.write_u32(3);
        h2.write_u8(1);
        h2.write_u16(2);
        h2.write_u32(3);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_order_matters() {
        let mut h1 = FxHasher::default();
        let mut h2 = FxHasher::default();
        h1.write_u8(1);
        h1.write_u8(2);
        h2.write_u8(2);
        h2.write_u8(1);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_fxhashmap_basic() {
        let mut map: FxHashMap<u64, &str> = FxHashMap::default();
        map.insert(1, "one");
        map.insert(2, "two");
        map.insert(3, "three");

        assert_eq!(map.get(&1), Some(&"one"));
        assert_eq!(map.get(&2), Some(&"two"));
        assert_eq!(map.get(&3), Some(&"three"));
        assert_eq!(map.get(&4), None);
    }

    #[test]
    fn test_fxhashmap_overwrite() {
        let mut map: FxHashMap<u64, i32> = FxHashMap::default();
        map.insert(1, 100);
        assert_eq!(map.get(&1), Some(&100));
        map.insert(1, 200);
        assert_eq!(map.get(&1), Some(&200));
    }
}
