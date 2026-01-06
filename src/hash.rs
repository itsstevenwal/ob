use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// FxHasher - a fast, non-cryptographic hasher
/// Extracted from rustc-hash (https://github.com/rust-lang/rustc-hash)
/// Licensed under MIT/Apache-2.0
#[derive(Clone)]
pub struct FxHasher {
    hash: usize,
}

#[cfg(target_pointer_width = "64")]
const K: usize = 0xf1357aea2e62a9c5;
#[cfg(target_pointer_width = "32")]
const K: usize = 0x93d765dd;

impl FxHasher {
    #[inline]
    fn add_to_hash(&mut self, i: usize) {
        self.hash = self.hash.wrapping_add(i).wrapping_mul(K);
    }
}

impl Default for FxHasher {
    #[inline]
    fn default() -> FxHasher {
        FxHasher { hash: 0 }
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.write_u64(hash_bytes(bytes));
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(i as usize);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
        #[cfg(target_pointer_width = "32")]
        self.add_to_hash((i >> 32) as usize);
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.add_to_hash(i as usize);
        #[cfg(target_pointer_width = "32")]
        self.add_to_hash((i >> 32) as usize);
        self.add_to_hash((i >> 64) as usize);
        #[cfg(target_pointer_width = "32")]
        self.add_to_hash((i >> 96) as usize);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(i);
    }

    #[inline]
    fn finish(&self) -> u64 {
        #[cfg(target_pointer_width = "64")]
        const ROTATE: u32 = 26;
        #[cfg(target_pointer_width = "32")]
        const ROTATE: u32 = 15;

        self.hash.rotate_left(ROTATE) as u64
    }
}

// Nothing special, digits of pi.
const SEED1: u64 = 0x243f6a8885a308d3;
const SEED2: u64 = 0x13198a2e03707344;
const PREVENT_TRIVIAL_ZERO_COLLAPSE: u64 = 0xa4093822299f31d0;

#[inline]
fn multiply_mix(x: u64, y: u64) -> u64 {
    #[cfg(target_pointer_width = "64")]
    {
        let full = (x as u128).wrapping_mul(y as u128);
        let lo = full as u64;
        let hi = (full >> 64) as u64;
        lo ^ hi
    }

    #[cfg(target_pointer_width = "32")]
    {
        let lx = x as u32;
        let ly = y as u32;
        let hx = (x >> 32) as u32;
        let hy = (y >> 32) as u32;

        let afull = (lx as u64).wrapping_mul(hy as u64);
        let bfull = (hx as u64).wrapping_mul(ly as u64);

        afull ^ bfull.rotate_right(32)
    }
}

/// A wyhash-inspired non-collision-resistant hash for strings/slices designed
/// by Orson Peters, with a focus on small strings and small codesize.
#[inline]
fn hash_bytes(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut s0 = SEED1;
    let mut s1 = SEED2;

    if len <= 16 {
        if len >= 8 {
            s0 ^= u64::from_le_bytes(bytes[0..8].try_into().unwrap());
            s1 ^= u64::from_le_bytes(bytes[len - 8..].try_into().unwrap());
        } else if len >= 4 {
            s0 ^= u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as u64;
            s1 ^= u32::from_le_bytes(bytes[len - 4..].try_into().unwrap()) as u64;
        } else if len > 0 {
            let lo = bytes[0];
            let mid = bytes[len / 2];
            let hi = bytes[len - 1];
            s0 ^= lo as u64;
            s1 ^= ((hi as u64) << 8) | mid as u64;
        }
    } else {
        let mut off = 0;
        while off < len - 16 {
            let x = u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap());
            let y = u64::from_le_bytes(bytes[off + 8..off + 16].try_into().unwrap());

            let t = multiply_mix(s0 ^ x, PREVENT_TRIVIAL_ZERO_COLLAPSE ^ y);
            s0 = s1;
            s1 = t;
            off += 16;
        }

        let suffix = &bytes[len - 16..];
        s0 ^= u64::from_le_bytes(suffix[0..8].try_into().unwrap());
        s1 ^= u64::from_le_bytes(suffix[8..16].try_into().unwrap());
    }

    multiply_mix(s0, s1) ^ (len as u64)
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
