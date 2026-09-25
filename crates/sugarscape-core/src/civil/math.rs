//! An exponential that is bit-for-bit the same on every platform. `f64::exp`
//! calls the platform's math library, which may differ in the last bit
//! between a native build and wasm32; the arrest probability decides who
//! rebels, so it uses only IEEE-exact arithmetic (as the anasazi's `ln`).

use std::f64::consts::LN_2;

/// ln 2 split in two (fdlibm's): the high part's low 21 bits are zero, so
/// `n · LN2_HI` is exact for every `n` used here.
const LN2_HI: f64 = f64::from_bits(0x3fe6_2e42_fee0_0000);
const LN2_LO: f64 = f64::from_bits(0x3dea_39ef_3579_3c76);

/// `e^x` for `x ≤ 0` from `+ − × ÷` only: `x = n·ln 2 + r` with `n` the
/// nearest whole number to `x / ln 2` (so `|r| ≤ ½ ln 2`, reduced with the
/// split ln 2 so large `n` lose nothing), `e^r` summed by Horner's rule over
/// 20 Taylor terms (far below one ulp for `|r| ≤ 0.35`), then scaled by
/// `2^n` through the exponent bits. Below −708 the result is 0 (the smallest
/// normal double is about e^−708).
pub fn exp_neg(x: f64) -> f64 {
    debug_assert!(x <= 0.0 && !x.is_nan(), "exp_neg of {x}");
    if x < -708.0 {
        return 0.0;
    }
    let n = (x / LN_2).round();
    let r = (x - n * LN2_HI) - n * LN2_LO;
    let mut series = 1.0;
    for k in (1..=20).rev() {
        series = 1.0 + series * r / f64::from(k);
    }
    // 2^n for n in −1022..=0, built from its exponent bits.
    let scale = f64::from_bits(((n as i64 + 1023) as u64) << 52);
    series * scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn exp_neg_matches_the_library_to_a_few_ulps() {
        let mut r = crate::rng::seeded(5);
        let mut xs: Vec<f64> = (0..20_000).map(|_| -r.gen::<f64>() * 50.0).collect();
        xs.extend([0.0, -1e-12, -0.5, -1.0, -2.3, -2.3 / 3.0, -100.0, -700.0]);
        for x in xs {
            let (ours, std) = (exp_neg(x), x.exp());
            assert!(
                (ours - std).abs() <= 8.0 * f64::EPSILON * std,
                "{x}: {ours} vs {std}"
            );
        }
        assert_eq!(exp_neg(0.0), 1.0);
        assert_eq!(exp_neg(-800.0), 0.0);
    }
}
