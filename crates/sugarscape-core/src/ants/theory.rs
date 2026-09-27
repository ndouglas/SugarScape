//! What the colony should do in the long run. Under Kirman's rule on the
//! complete graph with two sources the state k (ants at the first source) is a
//! birth-and-death chain, so its stationary distribution follows exactly from
//! detailed balance (Kirman's eqs. 2–5; with no pull it is the beta-binomial
//! with α = ε(N − 1)/(1 − δ)). Under Alfarano and Milaković's rule the mean
//! field gives the same kind of chain with rates (N − k)(a + λDk/N) and
//! k(a + λD(N − k)/N), whose limit is their Beta(α, α) with α = aN/λD.

use super::config::{AntsConfig, Conversion};

/// The chance that an ant at a source holding share `xi` joins a met ant's
/// source holding share `xj` (given it did not self-convert and the met ant is
/// elsewhere), under `c`'s conversion rule and pull.
pub fn join(c: &AntsConfig, xi: f64, xj: f64) -> f64 {
    let f = 1.0 + c.pull * (xj - xi);
    match c.conversion {
        Conversion::Kirman => {
            let r = ((1.0 - c.delta) * f).clamp(0.0, 1.0);
            (c.epsilon + r).min(1.0) - c.epsilon
        }
        Conversion::Footnote => (1.0 - c.epsilon) * (c.gamma() * f).clamp(0.0, 1.0),
    }
}

/// Kirman's P(k, k + 1) and P(k, k − 1) per meeting, two sources, the
/// complete graph, every ant herding.
pub fn kirman_rates(c: &AntsConfig, k: u32) -> (f64, f64) {
    let n = f64::from(c.ants);
    let kf = f64::from(k);
    let (xb, xw) = (kf / n, 1.0 - kf / n);
    let up = xw * (c.epsilon + kf / (n - 1.0) * join(c, xw, xb));
    let down = xb * (c.epsilon + (n - kf) / (n - 1.0) * join(c, xb, xw));
    (up, down)
}

/// The mean-field chain's rates for Alfarano and Milaković, with mean degree `d`.
pub fn alfarano_rates(c: &AntsConfig, d: f64, k: u32) -> (f64, f64) {
    let n = f64::from(c.ants);
    let kf = f64::from(k);
    let up = (n - kf) * (c.a + c.lambda * d * kf / n);
    let down = kf * (c.a + c.lambda * d * (n - kf) / n);
    (up, down)
}

/// The stationary distribution of a birth-and-death chain on 0..=n by
/// detailed balance, or `None` when a rate is zero (the chain is absorbed or
/// splits, and has no single long-run distribution).
pub fn stationary(n: u32, rates: impl Fn(u32) -> (f64, f64)) -> Option<Vec<f64>> {
    let mut log = vec![0.0f64; n as usize + 1];
    for k in 0..n {
        let (up, _) = rates(k);
        let (_, down) = rates(k + 1);
        if up <= 0.0 || down <= 0.0 {
            return None;
        }
        log[k as usize + 1] = log[k as usize] + (up / down).ln();
    }
    let top = log.iter().copied().fold(f64::MIN, f64::max);
    let mut p: Vec<f64> = log.iter().map(|l| (l - top).exp()).collect();
    let sum: f64 = p.iter().sum();
    for v in &mut p {
        *v /= sum;
    }
    Some(p)
}

/// Kirman's exact distribution of k, or `None` (ε = 0).
pub fn kirman(c: &AntsConfig) -> Option<Vec<f64>> {
    stationary(c.ants, |k| kirman_rates(c, k))
}

/// Alfarano and Milaković's mean-field distribution of k, or `None` (a = 0).
pub fn alfarano(c: &AntsConfig, d: f64) -> Option<Vec<f64>> {
    stationary(c.ants, |k| alfarano_rates(c, d, k))
}

/// Kirman's shape parameter, ε(N − 1)/(1 − δ): below 1 the colony piles up
/// at the extremes, at 1 every split is equally likely, above 1 it centers.
pub fn kirman_alpha(c: &AntsConfig) -> f64 {
    c.epsilon * f64::from(c.ants - 1) / (1.0 - c.delta)
}

/// Var[z] of a distribution of k over 0..=n.
pub fn variance(p: &[f64]) -> f64 {
    let n = (p.len() - 1) as f64;
    let mean: f64 = p.iter().enumerate().map(|(k, q)| q * k as f64 / n).sum();
    p.iter()
        .enumerate()
        .map(|(k, q)| q * (k as f64 / n - mean).powi(2))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kirman_config(n: u32, epsilon: f64, delta: f64) -> AntsConfig {
        AntsConfig {
            ants: n,
            epsilon,
            delta,
            ..AntsConfig::default()
        }
    }

    fn beta_binomial(n: u32, alpha: f64) -> Vec<f64> {
        // P(k) ∝ C(n, k) B(k + α, n − k + α), by its ratio recurrence.
        let mut p = vec![1.0f64; n as usize + 1];
        for k in 0..n {
            let (kf, nf) = (f64::from(k), f64::from(n));
            p[k as usize + 1] =
                p[k as usize] * (nf - kf) / (kf + 1.0) * (kf + alpha) / (nf - kf - 1.0 + alpha);
        }
        let s: f64 = p.iter().sum();
        p.iter().map(|v| v / s).collect()
    }

    #[test]
    fn eq_one_s_rates_match_the_paper() {
        let c = kirman_config(100, 0.005, 0.01);
        let (up, down) = kirman_rates(&c, 30);
        assert!((up - 0.7 * (0.005 + 0.99 * 30.0 / 99.0)).abs() < 1e-15);
        assert!((down - 0.3 * (0.005 + 0.99 * 70.0 / 99.0)).abs() < 1e-15);
    }

    #[test]
    fn kirman_s_chain_is_the_beta_binomial() {
        for (e, d) in [(0.005, 0.01), (0.01, 0.02), (0.15, 0.3)] {
            let c = kirman_config(100, e, d);
            let p = kirman(&c).unwrap();
            let q = beta_binomial(100, kirman_alpha(&c));
            let gap = p
                .iter()
                .zip(&q)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
            assert!(gap < 1e-12, "ε {e}, δ {d}: {gap}");
            assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        }
        // Figure Ia is U-shaped, Ic centered.
        let a = kirman(&kirman_config(100, 0.005, 0.01)).unwrap();
        assert!(a[0] > a[50] * 5.0);
        let c = kirman(&kirman_config(100, 0.15, 0.3)).unwrap();
        assert!(c[50] > c[0] * 1e6);
    }

    #[test]
    fn the_uniform_threshold_is_exact() {
        let d = 0.02;
        let c = kirman_config(100, (1.0 - d) / 99.0, d);
        let p = kirman(&c).unwrap();
        assert!(p.iter().all(|v| (v - 1.0 / 101.0).abs() < 1e-12));
    }

    #[test]
    fn ehrenfest_and_the_martingale_are_the_special_cases() {
        // ε ½, δ 1: no interaction, a binomial.
        let p = kirman(&kirman_config(10, 0.5, 1.0)).unwrap();
        let binom =
            |k: u32| (0..k).fold(1.0, |acc, i| acc * f64::from(10 - i) / f64::from(i + 1)) / 1024.0;
        assert!((0..=10).all(|k| (p[k as usize] - binom(k)).abs() < 1e-12));
        // ε 0: absorbed, no stationary distribution.
        assert!(kirman(&kirman_config(10, 0.0, 0.0)).is_none());
    }

    #[test]
    fn the_footnote_gives_the_same_chain_with_two_sources() {
        let k = kirman_config(100, 0.005, 0.01);
        let f = AntsConfig {
            conversion: Conversion::Footnote,
            ..k.clone()
        };
        for j in [1, 30, 70, 99] {
            let (a, b) = (kirman_rates(&k, j), kirman_rates(&f, j));
            assert!((a.0 - b.0).abs() < 1e-15 && (a.1 - b.1).abs() < 1e-15);
        }
    }

    #[test]
    fn pull_favors_the_majority() {
        let base = kirman_config(100, 0.15, 0.3);
        let pulled = AntsConfig {
            pull: 1.0,
            ..base.clone()
        };
        assert_eq!(join(&base, 0.2, 0.8), 0.7);
        assert!(join(&pulled, 0.2, 0.8) > join(&pulled, 0.8, 0.2));
        assert!(
            (join(&pulled, 0.2, 0.8) - 0.85).abs() < 1e-12,
            "clamped at 1 − ε"
        );
        // With pull the chain can peak off the extremes and off the middle.
        let p = kirman(&pulled).unwrap();
        let modes: Vec<usize> = (1..100)
            .filter(|&k| p[k] > p[k - 1] && p[k] > p[k + 1])
            .collect();
        assert_eq!(modes, [18, 82]);
    }

    #[test]
    fn the_mean_field_matches_alfarano_and_milakovic_s_variance() {
        // α = aN/λD = 2: Var[z] → 1/(4(2α + 1)) = 0.05 for large N.
        let c = AntsConfig {
            ants: 2000,
            a: 0.01,
            lambda: 1.0,
            ..AntsConfig::default()
        };
        let v = variance(&alfarano(&c, 10.0).unwrap());
        assert!((v - 0.05).abs() < 0.001, "{v}");
    }
}
