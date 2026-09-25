//! Normal draws that are bit-for-bit the same on every platform: the
//! harvest noise feeds every fingerprint, so its logarithm is
//! `crate::portable::ln`, not the platform's `f64::ln`.

use rand::Rng;

use crate::rng::SimRng;

/// The portable logarithm (moved to `crate::portable` in milestone 12).
pub use crate::portable::ln;

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
