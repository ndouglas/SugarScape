//! Normal draws that are bit-for-bit the same on every platform. `f64::ln`
//! calls the platform's maths library, which may differ in the last bit
//! between a native build and wasm32; the harvest noise feeds every
//! fingerprint, so the logarithm here uses only IEEE-exact arithmetic.

use rand::Rng;

use crate::rng::SimRng;

/// The natural logarithm of a positive, finite, normal `x` from `+ − × ÷`
/// only: `x = m · 2^e` with `m` in `[√½, √2)`, and
/// `ln m = 2 atanh t`, `t = (m − 1)/(m + 1)`, summed as a series (`|t| ≤
/// 0.1716`, so 20 terms are far below one ulp).
pub fn ln(x: f64) -> f64 {
    debug_assert!(x.is_normal() && x > 0.0, "ln of {x}");
    let bits = x.to_bits();
    let mut e = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mut m = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000);
    if m > std::f64::consts::SQRT_2 {
        m *= 0.5;
        e += 1;
    }
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    // Horner's rule over 1 + t²/3 + t⁴/5 + … + t³⁸/39.
    let mut series = 0.0;
    for k in (0..20).rev() {
        series = series * t2 + 1.0 / f64::from(2 * k + 1);
    }
    e as f64 * std::f64::consts::LN_2 + 2.0 * t * series
}

/// A standard normal draw (Marsaglia's polar method): pairs `(u, v)`
/// uniform in `(−1, 1)²` until `0 < s = u² + v² < 1`, then
/// `u · √(−2 ln s / s)`. Each call draws its own pairs (the second normal
/// is not kept). This is a standard textbook method, chosen (over
/// Box–Muller's trigonometric form) because it needs no `sin`/`cos` — like
/// the `ln` above, every step is portable, ordinary `+ − × ÷` and `sqrt`
/// arithmetic that agrees bit-for-bit between native and wasm32.
pub fn normal(rng: &mut SimRng) -> f64 {
    loop {
        let u = rng.gen::<f64>() * 2.0 - 1.0;
        let v = rng.gen::<f64>() * 2.0 - 1.0;
        let s = u * u + v * v;
        if s > 0.0 && s < 1.0 {
            return u * (-2.0 * ln(s) / s).sqrt();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    #[test]
    fn ln_matches_the_library_to_a_few_ulps() {
        let mut r = rng::seeded(7);
        let mut xs: Vec<f64> = (0..20_000)
            .map(|_| r.gen::<f64>())
            .filter(|&x| x > 0.0)
            .collect();
        xs.extend([
            1.0,
            0.5,
            2.0,
            1e-300,
            1e300,
            std::f64::consts::E,
            1.0 + 1e-12,
            0.999_999,
        ]);
        for x in xs {
            let (ours, std) = (ln(x), x.ln());
            assert!(
                (ours - std).abs() <= 4.0 * f64::EPSILON * std.abs().max(1e-300),
                "{x}: {ours} vs {std}"
            );
        }
        assert_eq!(ln(1.0), 0.0);
    }

    #[test]
    fn normal_draws_have_mean_0_and_sd_1() {
        let mut r = rng::seeded(11);
        let n = 200_000;
        let draws: Vec<f64> = (0..n).map(|_| normal(&mut r)).collect();
        let mean = draws.iter().sum::<f64>() / n as f64;
        let sd = (draws.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
        assert!(mean.abs() < 0.01, "mean {mean}");
        assert!((sd - 1.0).abs() < 0.01, "sd {sd}");
        let below = draws.iter().filter(|&&d| d < -1.0).count() as f64 / n as f64;
        assert!((below - 0.1587).abs() < 0.005, "P(z < −1) = {below}");
    }

    #[test]
    fn draws_are_a_function_of_the_seed() {
        let a: Vec<f64> = {
            let mut r = rng::seeded(3);
            (0..5).map(|_| normal(&mut r)).collect()
        };
        let mut r = rng::seeded(3);
        assert_eq!(a, (0..5).map(|_| normal(&mut r)).collect::<Vec<_>>());
    }
}
