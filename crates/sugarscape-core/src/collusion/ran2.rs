//! Numerical Recipes' RAN2 (L'Ecuyer's combined generator with a
//! Bays–Durham shuffle), exactly as the authors' `generic_routines.f90`
//! writes it: 32-bit integer arithmetic by Schrage's method, so every
//! platform gives the same draws as their Fortran.

const IM1: i32 = 2_147_483_563;
const IM2: i32 = 2_147_483_399;
const IMM1: i32 = IM1 - 1;
const IA1: i32 = 40_014;
const IA2: i32 = 40_692;
const IQ1: i32 = 53_668;
const IQ2: i32 = 52_774;
const IR1: i32 = 12_211;
const IR2: i32 = 3_791;
const NDIV: i32 = 1 + IMM1 / 32;
const AM: f64 = 1.0 / IM1 as f64;
const RNMX: f64 = 1.0 - 1.2e-7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran2 {
    idum: i32,
    idum2: i32,
    iy: i32,
    iv: [i32; 32],
}

impl Ran2 {
    /// A stream seeded as the code seeds it: `idum` negative (the code uses
    /// −session, or −1 for initial prices), `idum2` = 123456789, `iv` and
    /// `iy` zero; the first draw initializes.
    pub fn new(idum: i32) -> Self {
        Ran2 {
            idum,
            idum2: 123_456_789,
            iy: 0,
            iv: [0; 32],
        }
    }

    /// A uniform deviate in (0, 1).
    pub fn next_f64(&mut self) -> f64 {
        if self.idum <= 0 {
            self.idum = (-self.idum).max(1);
            self.idum2 = self.idum;
            for j in (1..=40).rev() {
                let k = self.idum / IQ1;
                self.idum = IA1 * (self.idum - k * IQ1) - k * IR1;
                if self.idum < 0 {
                    self.idum += IM1;
                }
                if j <= 32 {
                    self.iv[j - 1] = self.idum;
                }
            }
            self.iy = self.iv[0];
        }
        let k = self.idum / IQ1;
        self.idum = IA1 * (self.idum - k * IQ1) - k * IR1;
        if self.idum < 0 {
            self.idum += IM1;
        }
        let k = self.idum2 / IQ2;
        self.idum2 = IA2 * (self.idum2 - k * IQ2) - k * IR2;
        if self.idum2 < 0 {
            self.idum2 += IM2;
        }
        let j = (self.iy / NDIV) as usize;
        self.iy = self.iv[j] - self.idum2;
        self.iv[j] = self.idum;
        if self.iy < 1 {
            self.iy += IMM1;
        }
        (AM * f64::from(self.iy)).min(RNMX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_draws_are_the_fortran_s() {
        // The authors' ran2, compiled with gfortran 14, printed to 17 digits.
        let mut one = Ran2::new(-1);
        let mut seven = Ran2::new(-7);
        for want in [0.2853808990946861, 0.2533581892659171, 0.09346853100919404] {
            assert_eq!(one.next_f64(), want);
        }
        for want in [0.45206034994923033, 0.8885129427181557, 0.3178740604870464] {
            assert_eq!(seven.next_f64(), want);
        }
    }

    #[test]
    fn draws_are_uniform_and_reproducible() {
        let mut a = Ran2::new(-7);
        let mut b = Ran2::new(-7);
        let xs: Vec<f64> = (0..10_000).map(|_| a.next_f64()).collect();
        assert!(xs.iter().all(|&x| x > 0.0 && x < 1.0));
        assert_eq!(xs, (0..10_000).map(|_| b.next_f64()).collect::<Vec<_>>());
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        assert!((mean - 0.5).abs() < 0.01, "{mean}");
        assert_ne!(Ran2::new(-1).next_f64(), Ran2::new(-2).next_f64());
    }
}
