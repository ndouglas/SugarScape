//! The statistics the survey's judges need: Mann–Whitney U, Welch TOST,
//! quantiles. Self-contained so the survey has no statistics dependency.

pub fn finite(v: &[f64]) -> Vec<f64> {
    v.iter().copied().filter(|x| x.is_finite()).collect()
}

pub fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn sorted(v: &[f64]) -> Vec<f64> {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).expect("finite values"));
    s
}

pub fn quantile(v: &[f64], q: f64) -> f64 {
    let s = sorted(v);
    let at = q * (s.len() - 1) as f64;
    let (i, frac) = (at.floor() as usize, at.fract());
    if i + 1 < s.len() {
        s[i] + frac * (s[i + 1] - s[i])
    } else {
        s[i]
    }
}

pub fn median(v: &[f64]) -> f64 {
    quantile(v, 0.5)
}

pub fn frac_in(v: &[f64], lo: f64, hi: f64) -> f64 {
    v.iter().filter(|x| (lo..=hi).contains(*x)).count() as f64 / v.len() as f64
}

fn variance(v: &[f64]) -> f64 {
    let m = mean(v);
    v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() as f64 - 1.0)
}

pub fn skewness(v: &[f64]) -> f64 {
    let m = mean(v);
    let n = v.len() as f64;
    let m2 = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
    if m2 == 0.0 {
        return 0.0;
    }
    let m3 = v.iter().map(|x| (x - m).powi(3)).sum::<f64>() / n;
    m3 / m2.powf(1.5)
}

/// Abramowitz & Stegun 7.1.26 (absolute error < 1.5e-7).
fn erf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let y = 1.0
        - t * (0.254829592
            + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))))
            * (-x * x).exp();
    y.copysign(x)
}

pub fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

/// Lanczos approximation (g = 7, n = 9).
fn ln_gamma(x: f64) -> f64 {
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let t = x + 7.5;
    let sum = C[1..]
        .iter()
        .enumerate()
        .fold(C[0], |s, (i, c)| s + c / (x + i as f64 + 1.0));
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
}

/// Continued fraction for the incomplete beta (Numerical Recipes `betacf`).
fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    let (tiny, eps) = (1e-300, 1e-14);
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..300 {
        let m = f64::from(m);
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + aa / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + aa / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < eps {
            break;
        }
    }
    h
}

/// The regularized incomplete beta I_x(a, b).
fn beta_inc(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let front =
        (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_cf(a, b, x) / a
    } else {
        1.0 - front * beta_cf(b, a, 1.0 - x) / b
    }
}

pub fn student_t_cdf(t: f64, df: f64) -> f64 {
    let tail = 0.5 * beta_inc(df / 2.0, 0.5, df / (df + t * t));
    if t > 0.0 {
        1.0 - tail
    } else {
        tail
    }
}

/// U for `a`: pairs where a > b, plus half the ties.
fn u_stat(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .map(|x| {
            b.iter()
                .map(|y| if x > y { 1.0 } else if x == y { 0.5 } else { 0.0 })
                .sum::<f64>()
        })
        .sum()
}

fn has_ties(a: &[f64], b: &[f64]) -> bool {
    let mut all = sorted(&[a, b].concat());
    let n = all.len();
    all.dedup();
    all.len() != n
}

/// Exact P(U ≥ u) under H0 with no ties, by the recurrence
/// f(u; m, n) = f(u − n; m − 1, n) + f(u; m, n − 1).
fn exact_upper(m: usize, n: usize, u: f64) -> f64 {
    let max = m * n;
    // counts[j][k][w] = arrangements of j a's and k b's with U = w.
    let mut counts = vec![vec![vec![0f64; max + 1]; n + 1]; m + 1];
    for j in 0..=m {
        for k in 0..=n {
            if j == 0 || k == 0 {
                counts[j][k][0] = 1.0;
                continue;
            }
            for w in 0..=j * k {
                let with_a_last = if w >= k { counts[j - 1][k][w - k] } else { 0.0 };
                counts[j][k][w] = with_a_last + counts[j][k - 1][w];
            }
        }
    }
    let total: f64 = counts[m][n].iter().sum();
    let from = u.ceil() as usize;
    counts[m][n][from.min(max + 1)..].iter().sum::<f64>() / total
}

/// Normal approximation with tie correction and continuity correction.
fn normal_upper(a: &[f64], b: &[f64], u: f64) -> f64 {
    let (m, n) = (a.len() as f64, b.len() as f64);
    let all = sorted(&[a, b].concat());
    let big_n = m + n;
    let mut ties = 0.0;
    let mut i = 0;
    while i < all.len() {
        let j = all[i..].iter().take_while(|x| **x == all[i]).count();
        let t = j as f64;
        ties += t * t * t - t;
        i += j;
    }
    let var = m * n / 12.0 * ((big_n + 1.0) - ties / (big_n * (big_n - 1.0)));
    if var <= 0.0 {
        return 1.0;
    }
    let z = (u - m * n / 2.0 - 0.5) / var.sqrt();
    1.0 - normal_cdf(z)
}

/// One-sided p-value that `a` tends to exceed `b`.
pub fn mw_greater(a: &[f64], b: &[f64]) -> f64 {
    let u = u_stat(a, b);
    let p = if !has_ties(a, b) && a.len() <= 20 && b.len() <= 20 {
        exact_upper(a.len(), b.len(), u)
    } else {
        normal_upper(a, b, u)
    };
    p.clamp(0.0, 1.0)
}

pub fn mw_two_sided(a: &[f64], b: &[f64]) -> f64 {
    (2.0 * mw_greater(a, b).min(mw_greater(b, a))).min(1.0)
}

/// Welch two one-sided tests: p for H1 "−margin < mean(a) − mean(b) < margin".
pub fn tost(a: &[f64], b: &[f64], margin: f64) -> f64 {
    let d = mean(a) - mean(b);
    let (va, vb) = (variance(a) / a.len() as f64, variance(b) / b.len() as f64);
    let se = (va + vb).sqrt();
    if se == 0.0 {
        return if d.abs() < margin { 0.0 } else { 1.0 };
    }
    let df = (va + vb).powi(2)
        / (va * va / (a.len() as f64 - 1.0) + vb * vb / (b.len() as f64 - 1.0));
    let p_low = 1.0 - student_t_cdf((d + margin) / se, df);
    let p_high = student_t_cdf((d - margin) / se, df);
    p_low.max(p_high)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn summaries() {
        let v = [1.0, 2.0, 3.0, 4.0, 10.0];
        assert_eq!(mean(&v), 4.0);
        assert_eq!(median(&v), 3.0);
        assert_eq!(quantile(&v, 0.25), 2.0);
        assert_eq!(quantile(&v, 0.75), 4.0);
        assert_eq!(frac_in(&v, 2.0, 4.0), 0.6);
        assert!(skewness(&v) > 1.0);
        assert_eq!(skewness(&[3.0, 3.0]), 0.0);
        assert_eq!(finite(&[1.0, f64::NAN, f64::INFINITY, 2.0]), vec![1.0, 2.0]);
    }

    #[test]
    fn distributions_match_tables() {
        assert!(close(normal_cdf(1.96), 0.975, 1e-4));
        assert!(close(normal_cdf(0.0), 0.5, 1e-9));
        assert!(close(student_t_cdf(1.0, 1.0), 0.75, 1e-6));
        assert!(close(student_t_cdf(2.0, 10.0), 0.963306, 1e-5));
        assert!(close(student_t_cdf(-2.0, 10.0), 1.0 - 0.963306, 1e-5));
    }

    #[test]
    fn mann_whitney_exact_without_ties() {
        // Complete separation of 3 vs 3: p = 1 / C(6, 3) = 0.05.
        let (lo, hi) = ([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert!(close(mw_greater(&hi, &lo), 0.05, 1e-12));
        assert!(close(mw_greater(&lo, &hi), 1.0, 1e-12));
        assert!(close(mw_two_sided(&hi, &lo), 0.10, 1e-12));
    }

    #[test]
    fn mann_whitney_with_ties_and_identical_samples() {
        let z = [0.0; 20];
        assert_eq!(mw_greater(&z, &z), 1.0);
        assert_eq!(mw_two_sided(&z, &z), 1.0);
        let a: Vec<f64> = (0..20).map(|i| f64::from(i / 2) + 5.0).collect();
        let b: Vec<f64> = (0..20).map(|i| f64::from(i / 2)).collect();
        assert!(mw_greater(&a, &b) < 0.01);
    }

    #[test]
    fn tost_separates_equivalent_from_different() {
        let a: Vec<f64> = (0..20).map(|i| 100.0 + f64::from(i % 5)).collect();
        let b: Vec<f64> = (0..20).map(|i| 100.5 + f64::from(i % 5)).collect();
        assert!(tost(&a, &b, 5.0) < 0.05);
        let c: Vec<f64> = a.iter().map(|x| x + 20.0).collect();
        assert!(tost(&a, &c, 5.0) > 0.5);
    }

    #[test]
    fn tost_with_zero_variance() {
        assert_eq!(tost(&[3.0; 5], &[4.0; 5], 2.0), 0.0);
        assert_eq!(tost(&[3.0; 5], &[6.0; 5], 2.0), 1.0);
    }
}
