//! The records a run keeps for Axtell's §3.4 measurements, in bounded
//! memory (histograms and running sums), and the fits made from them: the
//! firm-size exponent µ by the paper's OLS and by maximum likelihood, the
//! output exponent, productivity, the growth-rate distribution (Laplace
//! against Gaussian) and σ_r's dependence on size (γ), and lifetimes.

use crate::portable::ln;

/// Output bins: width 0.05 in ln(output), from output 1.
const OUT_BIN: f64 = 0.05;
/// Growth-rate bins over [−5, 5].
const R_MAX: f64 = 5.0;
const R_BINS: usize = 2000;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Records {
    /// Firms of each size, pooled over the sampled periods (index = size).
    pub sizes: Vec<u64>,
    /// Firms by ln(output) bin, and per size: firms and their summed output.
    pub outputs: Vec<u64>,
    pub size_output: Vec<(u64, f64)>,
    /// Growth rates r = ln(s_t / s_{t−1}): a histogram, and per prior size
    /// the count, Σr and Σr².
    pub growth: Vec<u64>,
    pub growth_by_size: Vec<(u64, f64, f64)>,
    /// Lifetimes (periods) of firms that died, by lifetime; and the same for
    /// firms that ever had a second member.
    pub lifetimes: Vec<u64>,
    pub lifetimes_team: Vec<u64>,
}

fn bump<T: Clone + Default>(v: &mut Vec<T>, i: usize) -> &mut T {
    if v.len() <= i {
        v.resize(i + 1, T::default());
    }
    &mut v[i]
}

impl Records {
    /// One sampled period's firms: (size, output).
    pub fn sample(&mut self, firms: &[(u32, f64)]) {
        for &(s, o) in firms {
            *bump(&mut self.sizes, s as usize) += 1;
            if o >= 1.0 {
                *bump(&mut self.outputs, (ln(o) / OUT_BIN) as usize) += 1;
            }
            let cell = bump(&mut self.size_output, s as usize);
            cell.0 += 1;
            cell.1 += o;
        }
    }

    /// A surviving firm's size last period and now.
    pub fn grow(&mut self, before: u32, after: u32) {
        if before == 0 || after == 0 {
            return;
        }
        let r = ln(f64::from(after) / f64::from(before));
        let bin = (((r + R_MAX) / (2.0 * R_MAX)) * R_BINS as f64).clamp(0.0, (R_BINS - 1) as f64)
            as usize;
        if self.growth.len() < R_BINS {
            self.growth.resize(R_BINS, 0);
        }
        self.growth[bin] += 1;
        let cell = bump(&mut self.growth_by_size, before as usize);
        cell.0 += 1;
        cell.1 += r;
        cell.2 += r * r;
    }

    pub fn died(&mut self, lifetime: u64, team: bool) {
        *bump(&mut self.lifetimes, lifetime as usize) += 1;
        if team {
            *bump(&mut self.lifetimes_team, lifetime as usize) += 1;
        }
    }
}

/// e^x, portable.
fn exp(x: f64) -> f64 {
    if x <= 0.0 {
        crate::portable::exp_neg(x)
    } else {
        1.0 / crate::portable::exp_neg(-x)
    }
}

/// Ordinary least squares slope and intercept of y on x.
pub fn ols(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let n = points.len() as f64;
    if points.len() < 2 {
        return None;
    }
    let mx = points.iter().map(|p| p.0).sum::<f64>() / n;
    let my = points.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|p| (p.0 - mx) * (p.0 - mx)).sum();
    if sxx == 0.0 {
        return None;
    }
    let slope = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / sxx;
    Some((slope, my - slope * mx))
}

/// A99 §3.4's µ: OLS of ln p(s) on ln s, dropping size 1 and frequencies
/// below 10⁻⁵; p(s) ∝ s^−(1+µ). NaN without enough points.
pub fn mu_ols(sizes: &[u64]) -> f64 {
    let total: u64 = sizes.iter().sum();
    if total == 0 {
        return f64::NAN;
    }
    let points: Vec<(f64, f64)> = sizes
        .iter()
        .enumerate()
        .skip(2)
        .filter(|&(_, &c)| c > 0 && c as f64 / total as f64 >= 1e-5)
        .map(|(s, &c)| (ln(s as f64), ln(c as f64 / total as f64)))
        .collect();
    ols(&points).map_or(f64::NAN, |(slope, _)| -slope - 1.0)
}

/// The discrete power law's maximum-likelihood exponent for sizes ≥ `smin`
/// (Clauset, Shalizi and Newman 2009, eq. 3.5: maximizing −n ln ζ(α, smin) −
/// α Σ ln s, the Hurwitz zeta summed to the largest size seen and the tail
/// by its integral), as µ = α − 1.
pub fn mu_mle(sizes: &[u64], smin: usize) -> f64 {
    let (mut n, mut sum_ln) = (0.0, 0.0);
    for (s, &c) in sizes.iter().enumerate().skip(smin) {
        if c > 0 {
            n += c as f64;
            sum_ln += c as f64 * ln(s as f64);
        }
    }
    if n == 0.0 || sizes.len() <= smin + 1 {
        return f64::NAN;
    }
    let top = sizes.len() as f64;
    let zeta = |alpha: f64| {
        let mut z = 0.0;
        for s in smin..sizes.len() {
            z += 1.0 / super::effort::powf(s as f64, alpha);
        }
        // The rest, from `top`: ∫ x^−α dx from top − ½.
        z + super::effort::powf(top - 0.5, 1.0 - alpha) / (alpha - 1.0)
    };
    let loglik = |alpha: f64| -n * ln(zeta(alpha)) - alpha * sum_ln;
    let (mut l, mut h) = (1.000_1, 6.0);
    let phi = 0.5 * (5.0_f64.sqrt() - 1.0);
    for _ in 0..60 {
        let (x1, x2) = (h - phi * (h - l), l + phi * (h - l));
        if loglik(x1) >= loglik(x2) {
            h = x2;
        } else {
            l = x1;
        }
    }
    0.5 * (l + h) - 1.0
}

/// The output-distribution exponent on the µ convention (density per unit
/// output over the ln bins; dropping densities below 10⁻⁵ of the total).
pub fn output_exponent(outputs: &[u64]) -> f64 {
    let total: u64 = outputs.iter().sum();
    if total == 0 {
        return f64::NAN;
    }
    let points: Vec<(f64, f64)> = outputs
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c > 0 && c as f64 / total as f64 >= 1e-5)
        .map(|(k, &c)| {
            let lo = k as f64 * OUT_BIN;
            let (x0, x1) = (exp(lo), exp(lo + OUT_BIN));
            let mid = 0.5 * (x0 + x1);
            (ln(mid), ln(c as f64 / total as f64 / (x1 - x0)))
        })
        .collect();
    ols(&points).map_or(f64::NAN, |(slope, _)| -slope - 1.0)
}

/// Productivity: mean output against size, fitted as c·s^k (A99 fig. 19).
pub fn productivity(size_output: &[(u64, f64)]) -> (f64, f64) {
    let points: Vec<(f64, f64)> = size_output
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, c)| c.0 > 0 && c.1 > 0.0)
        .map(|(s, c)| (ln(s as f64), ln(c.1 / c.0 as f64)))
        .collect();
    ols(&points).map_or((f64::NAN, f64::NAN), |(slope, icept)| (exp(icept), slope))
}

/// The growth-rate distribution: mean, standard deviation, and the mean
/// log-likelihood per observation of the fitted Laplace and Gaussian.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrowthFit {
    pub n: u64,
    pub mean: f64,
    pub sd: f64,
    pub laplace_ll: f64,
    pub gauss_ll: f64,
}

pub fn growth_fit(growth: &[u64]) -> GrowthFit {
    let n: u64 = growth.iter().sum();
    let center = |k: usize| -R_MAX + (k as f64 + 0.5) * 2.0 * R_MAX / R_BINS as f64;
    if n < 2 {
        return GrowthFit {
            n,
            mean: f64::NAN,
            sd: f64::NAN,
            laplace_ll: f64::NAN,
            gauss_ll: f64::NAN,
        };
    }
    let nf = n as f64;
    let mean = growth
        .iter()
        .enumerate()
        .map(|(k, &c)| c as f64 * center(k))
        .sum::<f64>()
        / nf;
    let var = growth
        .iter()
        .enumerate()
        .map(|(k, &c)| c as f64 * (center(k) - mean).powi(2))
        .sum::<f64>()
        / nf;
    // The Laplace's MLE: location the median, scale the mean absolute deviation.
    let mut seen = 0;
    let mut median = 0.0;
    for (k, &c) in growth.iter().enumerate() {
        seen += c;
        if 2 * seen >= n {
            median = center(k);
            break;
        }
    }
    let scale = growth
        .iter()
        .enumerate()
        .map(|(k, &c)| c as f64 * (center(k) - median).abs())
        .sum::<f64>()
        / nf;
    let two_pi_ln = ln(2.0 * std::f64::consts::PI);
    let laplace_ll = if scale > 0.0 {
        -ln(2.0 * scale) - 1.0
    } else {
        f64::INFINITY
    };
    let gauss_ll = if var > 0.0 {
        -0.5 * (two_pi_ln + ln(var)) - 0.5
    } else {
        f64::INFINITY
    };
    GrowthFit {
        n,
        mean,
        sd: var.sqrt(),
        laplace_ll,
        gauss_ll,
    }
}

/// σ_r by prior size, and γ in σ_r ∝ s^−γ by OLS over sizes from `from` to
/// `to` with at least `min_n` observations (A99 drops the first two sizes and
/// the noisy large ones).
pub fn gamma(growth_by_size: &[(u64, f64, f64)], from: usize, to: usize, min_n: u64) -> f64 {
    let points: Vec<(f64, f64)> = growth_by_size
        .iter()
        .enumerate()
        .filter(|&(s, c)| s >= from && s <= to && c.0 >= min_n)
        .filter_map(|(s, c)| {
            let n = c.0 as f64;
            let var = c.2 / n - (c.1 / n) * (c.1 / n);
            (var > 0.0).then(|| (ln(s as f64), 0.5 * ln(var)))
        })
        .collect();
    ols(&points).map_or(f64::NAN, |(slope, _)| -slope)
}

/// Lifetimes: count, mean, standard deviation, and the slope of lifetime on
/// log₁₀ rank (longest first; the 3 longest dropped, as A99 fig. 22).
pub fn lifetimes(hist: &[u64]) -> (u64, f64, f64, f64) {
    let n: u64 = hist.iter().sum();
    if n == 0 {
        return (0, f64::NAN, f64::NAN, f64::NAN);
    }
    let nf = n as f64;
    let mean = hist
        .iter()
        .enumerate()
        .map(|(l, &c)| c as f64 * l as f64)
        .sum::<f64>()
        / nf;
    let var = hist
        .iter()
        .enumerate()
        .map(|(l, &c)| c as f64 * (l as f64 - mean).powi(2))
        .sum::<f64>()
        / nf;
    // Rank plot, streamed: ranks 4.. in descending order of lifetime.
    let (mut sx, mut sy, mut sxx, mut sxy, mut m) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut rank = 0u64;
    let log10 = ln(10.0);
    for (l, &c) in hist.iter().enumerate().rev() {
        for _ in 0..c {
            rank += 1;
            if rank <= 3 {
                continue;
            }
            let x = ln(rank as f64) / log10;
            let y = l as f64;
            sx += x;
            sy += y;
            sxx += x * x;
            sxy += x * y;
            m += 1.0;
        }
    }
    let slope = if m >= 2.0 {
        (m * sxy - sx * sy) / (m * sxx - sx * sx)
    } else {
        f64::NAN
    };
    (n, mean, var.sqrt(), slope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;
    use rand::Rng;

    /// A synthetic discrete power law with exponent µ (p(s) ∝ s^−(1+µ)),
    /// sizes 1..=cap, `n` draws by inversion.
    fn power_law(mu: f64, n: usize, cap: usize) -> Vec<u64> {
        let weights: Vec<f64> = (1..=cap).map(|s| (s as f64).powf(-(1.0 + mu))).collect();
        let total: f64 = weights.iter().sum();
        let mut cdf = Vec::with_capacity(cap);
        let mut acc = 0.0;
        for w in &weights {
            acc += w / total;
            cdf.push(acc);
        }
        let mut r = rng::seeded(3);
        let mut hist = vec![0u64; cap + 1];
        for _ in 0..n {
            let u: f64 = r.gen();
            let s = cdf.partition_point(|&c| c < u) + 1;
            hist[s.min(cap)] += 1;
        }
        hist
    }

    #[test]
    fn both_estimators_recover_a_known_exponent() {
        let hist = power_law(1.3, 1_000_000, 5000);
        let (o, m) = (mu_ols(&hist), mu_mle(&hist, 2));
        assert!((o - 1.3).abs() < 0.1, "OLS {o}");
        assert!((m - 1.3).abs() < 0.02, "MLE {m}");
    }

    #[test]
    fn the_ols_drops_size_one_and_rare_sizes() {
        let mut hist = vec![0u64; 11];
        for (s, c) in hist.iter_mut().enumerate().skip(2) {
            *c = (1_000_000.0 / (s as f64).powi(3)) as u64;
        }
        let base = mu_ols(&hist);
        hist[1] = 10_000_000; // size 1 is dropped
        hist.push(1); // frequency below 10⁻⁵ is dropped
        assert!((mu_ols(&hist) - base).abs() < 1e-9);
        assert!((base - 2.0).abs() < 0.01, "{base}");
    }

    #[test]
    fn laplace_wins_on_laplace_data_and_loses_on_gaussian() {
        let mut r = rng::seeded(5);
        let (mut lap, mut gau) = (Records::default(), Records::default());
        for _ in 0..200_000 {
            let u: f64 = r.gen::<f64>() - 0.5;
            let x = -0.3 * u.signum() * (1.0 - 2.0 * u.abs()).ln();
            lap.grow(100, (100.0 * x.exp()).round().max(1.0) as u32);
            let (u1, u2): (f64, f64) = (r.gen(), r.gen());
            let z = (-2.0 * (1.0 - u1).ln()).sqrt() * (std::f64::consts::TAU * u2).cos() * 0.3;
            gau.grow(100, (100.0 * z.exp()).round().max(1.0) as u32);
        }
        let (fl, fg) = (growth_fit(&lap.growth), growth_fit(&gau.growth));
        assert!(fl.laplace_ll > fl.gauss_ll, "{fl:?}");
        assert!(fg.gauss_ll > fg.laplace_ll, "{fg:?}");
    }

    #[test]
    fn gamma_recovers_a_known_scaling() {
        let mut by = vec![(0u64, 0.0, 0.0); 200];
        for (s, cell) in by.iter_mut().enumerate().skip(1) {
            let sd = 0.5 * (s as f64).powf(-0.2);
            // A two-point distribution ±sd has mean 0 and variance sd².
            *cell = (1000, 0.0, 1000.0 * sd * sd);
        }
        assert!((gamma(&by, 3, 150, 10) - 0.2).abs() < 1e-9);
    }

    #[test]
    fn lifetimes_report_mean_sd_and_rank_slope() {
        let mut hist = vec![0u64; 11];
        hist[2] = 3;
        hist[10] = 1;
        let (n, mean, sd, _) = lifetimes(&hist);
        assert_eq!(n, 4);
        assert!((mean - 4.0).abs() < 1e-12 && (sd - 12f64.sqrt()).abs() < 1e-12);
        // Exponential lifetimes give a straight line in log rank.
        let mut exp = vec![0u64; 200];
        for (l, c) in exp.iter_mut().enumerate() {
            *c = (10_000.0 * (-(l as f64) / 20.0).exp()) as u64;
        }
        let slope = lifetimes(&exp).3;
        assert!((slope + 20.0 * 10f64.ln()).abs() < 3.0, "{slope}");
    }

    #[test]
    fn productivity_recovers_constant_returns() {
        let so: Vec<(u64, f64)> = (0..100)
            .map(|s| {
                if s == 0 {
                    (0, 0.0)
                } else {
                    (10, 10.0 * 2.0 * s as f64)
                }
            })
            .collect();
        let (c, k) = productivity(&so);
        assert!((k - 1.0).abs() < 1e-9 && (c - 2.0).abs() < 1e-9, "{c} {k}");
    }
}
