//! Fixed-length bit strings for Chapter V's immune systems and diseases.
//! Bit `i` is string position `i`; strings print position 0 first. Culture
//! tags keep their own `Tags` type.

use rand::Rng;

/// A string of `len` bits, 0 ≤ len ≤ 64. Length 0 is the empty string agents
/// hold while disease is off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bits {
    bits: u64,
    len: u32,
}

fn mask(len: u32) -> u64 {
    if len == 64 {
        u64::MAX
    } else {
        (1u64 << len) - 1
    }
}

impl Bits {
    pub fn new(bits: u64, len: u32) -> Self {
        assert!(len <= 64, "bit string length must be ≤ 64");
        Self {
            bits: bits & mask(len),
            len,
        }
    }

    pub fn random(len: u32, rng: &mut impl Rng) -> Self {
        Self::new(rng.gen(), len)
    }

    /// Parses `0`/`1` characters, position 0 first; `None` for any other
    /// character or more than 64 of them.
    pub fn parse(s: &str) -> Option<Self> {
        let mut bits = 0u64;
        let mut len = 0u32;
        for c in s.chars() {
            if len == 64 {
                return None;
            }
            match c {
                '0' => {}
                '1' => bits |= 1 << len,
                _ => return None,
            }
            len += 1;
        }
        Some(Self::new(bits, len))
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn bits(&self) -> u64 {
        self.bits
    }

    fn check(&self, i: u32) {
        assert!(i < self.len, "bit index {i} out of range [0, {})", self.len);
    }

    pub fn get(&self, i: u32) -> bool {
        self.check(i);
        (self.bits >> i) & 1 == 1
    }

    pub fn set(&mut self, i: u32, value: bool) {
        self.check(i);
        if value {
            self.bits |= 1 << i;
        } else {
            self.bits &= !(1 << i);
        }
    }

    pub fn flip(&mut self, i: u32) {
        self.check(i);
        self.bits ^= 1 << i;
    }

    /// The `len` bits starting at position `start`.
    pub fn window(&self, start: u32, len: u32) -> Bits {
        assert!(
            start + len <= self.len,
            "window [{start}, {}) out of range [0, {})",
            start + len,
            self.len
        );
        Bits::new(self.bits.checked_shr(start).unwrap_or(0), len)
    }

    /// Number of positions at which two equal-length strings differ.
    pub fn hamming(&self, other: &Bits) -> u32 {
        assert_eq!(self.len, other.len, "Hamming distance needs equal lengths");
        (self.bits ^ other.bits).count_ones()
    }

    /// The window of this string closest to `d` in Hamming distance (leftmost
    /// on ties) as `(start, distance)`; `None` if `d` is empty or longer.
    pub fn closest_window(&self, d: &Bits) -> Option<(u32, u32)> {
        if d.is_empty() || d.len > self.len {
            return None;
        }
        let mut best: Option<(u32, u32)> = None;
        for start in 0..=self.len - d.len {
            let distance = self.window(start, d.len).hamming(d);
            if best.is_none_or(|(_, b)| distance < b) {
                best = Some((start, distance));
                if distance == 0 {
                    break;
                }
            }
        }
        best
    }

    /// Whether `d` occurs in this string (the book's "substring").
    pub fn contains(&self, d: &Bits) -> bool {
        matches!(self.closest_window(d), Some((_, 0)))
    }

    /// One step of Appendix B's immune response: unless `d` already occurs,
    /// flip the first bit (lowest position) where the closest window differs
    /// from `d`. Returns whether a bit was flipped.
    pub fn learn(&mut self, d: &Bits) -> bool {
        match self.closest_window(d) {
            Some((start, distance)) if distance > 0 => {
                let first = (self.window(start, d.len).bits ^ d.bits).trailing_zeros();
                self.flip(start + first);
                true
            }
            _ => false,
        }
    }

    /// Vaccination: overwrites the closest window with `d`, so `d` now occurs.
    /// Returns `false` (and changes nothing) if `d` does not fit.
    pub fn imprint(&mut self, d: &Bits) -> bool {
        let Some((start, _)) = self.closest_window(d) else {
            return false;
        };
        for i in 0..d.len {
            self.set(start + i, d.get(i));
        }
        true
    }

    /// Position 0 first.
    pub fn to_bit_string(&self) -> String {
        (0..self.len)
            .map(|i| if self.get(i) { '1' } else { '0' })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(s: &str) -> Bits {
        Bits::parse(s).unwrap()
    }

    #[test]
    fn parses_and_prints_position_zero_first() {
        let x = b("0110");
        assert_eq!(
            (x.len(), x.get(0), x.get(1), x.get(3)),
            (4, false, true, false)
        );
        assert_eq!(x.to_bit_string(), "0110");
        assert_eq!(Bits::parse("01a"), None);
        assert_eq!(Bits::parse(&"1".repeat(65)), None);
        assert!(Bits::default().is_empty());
        assert_eq!(
            Bits::new(u64::MAX, 3).bits(),
            0b111,
            "bits beyond the length are masked"
        );
    }

    #[test]
    fn set_and_flip() {
        let mut x = b("000");
        x.set(0, true);
        x.flip(2);
        assert_eq!(x.to_bit_string(), "101");
        x.flip(2);
        assert_eq!(x.to_bit_string(), "100");
    }

    #[test]
    fn windows_and_hamming_distance() {
        assert_eq!(b("1011101001").window(3, 4).to_bit_string(), "1101");
        assert_eq!(b("10111").hamming(&b("10011")), 1);
        let full = Bits::new(u64::MAX, 64);
        assert_eq!(full.window(60, 4).to_bit_string(), "1111");
        assert!(full.window(64, 0).is_empty());
    }

    #[test]
    fn book_example_learns_the_disease_in_one_flip() {
        // Appendix B: immune 1011101001, disease 10011. The closest window is
        // at position 0 (10111, distance 1); flipping its first differing bit
        // (position 2) gives 1001101001, which contains the disease.
        let mut immune = b("1011101001");
        let disease = b("10011");
        assert_eq!(immune.closest_window(&disease), Some((0, 1)));
        assert!(!immune.contains(&disease));
        assert!(immune.learn(&disease));
        assert_eq!(immune.to_bit_string(), "1001101001");
        assert!(immune.contains(&disease));
        assert!(!immune.learn(&disease), "nothing left to learn");
    }

    #[test]
    fn ties_go_to_the_leftmost_window() {
        // Windows 01, 10, 01 are all at distance 1 from 11.
        let mut immune = b("0101");
        assert_eq!(immune.closest_window(&b("11")), Some((0, 1)));
        assert!(immune.learn(&b("11")));
        assert_eq!(immune.to_bit_string(), "1101");
    }

    #[test]
    fn diseases_longer_than_the_immune_string_are_never_learned() {
        let mut immune = b("01");
        assert_eq!(immune.closest_window(&b("010")), None);
        assert!(!immune.contains(&b("010")));
        assert!(!immune.learn(&b("010")));
        assert!(!Bits::default().contains(&b("1")));
    }

    #[test]
    fn imprint_writes_the_disease_over_the_closest_window() {
        // Windows of 000110 against 111: 000 (3), 001 (2), 011 (1), 110 (1).
        let mut immune = b("000110");
        assert!(immune.imprint(&b("111")));
        assert_eq!(immune.to_bit_string(), "001110");
        assert!(immune.contains(&b("111")));
        let mut short = b("01");
        assert!(!short.imprint(&b("010")));
    }
}
