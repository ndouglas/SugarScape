//! What theory expects. Granovetter's continuous equilibrium: the forward
//! recursion r ← N·F(r/N) from r = 0, F the normal c.d.f. of thresholds with
//! the mass below 0 at 0 (his Fig. 1–2). Watts's cascade condition (Eq. 5):
//! global cascades are possible where Σ k(k − 1)ρ_k p_k exceeds z, ρ_k = F(1/k)
//! the chance a degree-k node is vulnerable. For display and the survey
//! only; nothing here feeds a run.

use super::crowd::power_law;

/// The standard normal c.d.f. (Φ), from the complementary error function
/// (Numerical Recipes' Chebyshev fit, fractional error under 1.2·10⁻⁷).
pub fn normal_cdf(x: f64) -> f64 {
    let z = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.5 * z);
    let erfc = t
        * (-z * z - 1.265_512_23
            + t * (1.000_023_68
                + t * (0.374_091_96
                    + t * (0.096_784_18
                        + t * (-0.186_288_06
                            + t * (0.278_868_07
                                + t * (-1.135_203_98
                                    + t * (1.488_515_87
                                        + t * (-0.822_152_23 + t * 0.170_872_77)))))))))
            .exp();
    if x >= 0.0 {
        1.0 - 0.5 * erfc
    } else {
        0.5 * erfc
    }
}

/// Granovetter's continuous equilibrium, in people, for N people with normal
/// thresholds (mean, sd as fractions): r ← N·Φ((r/N − mean)/sd).
pub fn granovetter(n: u32, mean: f64, sd: f64) -> f64 {
    let nf = f64::from(n);
    let f = |r: f64| {
        let x = r / nf;
        if sd == 0.0 {
            if x >= mean {
                nf
            } else {
                0.0
            }
        } else {
            nf * normal_cdf((x - mean) / sd)
        }
    };
    let mut r = 0.0;
    for _ in 0..1_000_000 {
        let next = f(r);
        if (next - r).abs() < 1e-9 {
            return next;
        }
        r = next;
    }
    r
}

/// Poisson degrees with mean z: p_k for k = 0..=`kmax`.
pub fn poisson(z: f64, kmax: u32) -> Vec<f64> {
    let mut p = vec![(-z).exp()];
    for k in 1..=kmax {
        let prev = p[k as usize - 1];
        p.push(prev * z / f64::from(k));
    }
    p
}

/// ρ_k: the chance a node of degree k is vulnerable (θ ≤ 1/k), for
/// thresholds normal around `phi` with `sd` (0: all at `phi`).
pub fn vulnerable(k: u32, phi: f64, sd: f64) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let edge = 1.0 / f64::from(k);
    if sd == 0.0 {
        if phi <= edge + 1e-12 {
            1.0
        } else {
            0.0
        }
    } else {
        normal_cdf((edge - phi) / sd)
    }
}

/// Watts's G₀″(1)/z = Σ k(k − 1)ρ_k p_k / z for degrees `p` (p[i] is degree
/// i + `first`): above 1, a vulnerable cluster percolates.
pub fn cascade_ratio(p: &[f64], first: u32, phi: f64, sd: f64) -> f64 {
    let (mut top, mut z) = (0.0, 0.0);
    for (i, &q) in p.iter().enumerate() {
        let k = i as u32 + first;
        let kf = f64::from(k);
        z += kf * q;
        top += kf * (kf - 1.0) * vulnerable(k, phi, sd) * q;
    }
    top / z
}

/// The ratio on a uniform random graph (Poisson) of mean degree z.
pub fn poisson_ratio(z: f64, phi: f64, sd: f64) -> f64 {
    cascade_ratio(&poisson(z, 200), 0, phi, sd)
}

/// The ratio on Watts's power-law graph (τ 2.5) of mean degree z (< 1.95).
pub fn power_law_ratio(z: f64, phi: f64, sd: f64) -> f64 {
    cascade_ratio(&power_law(z, 5000), 1, phi, sd)
}

/// Where the Poisson ratio crosses 1, scanning z from `from` to `to` in
/// steps of 0.001: the analytic cascade window's lower and upper edges.
pub fn poisson_window(phi: f64, sd: f64, from: f64, to: f64) -> Option<(f64, f64)> {
    let mut edges = None;
    let mut z = from;
    while z <= to {
        if poisson_ratio(z, phi, sd) > 1.0 {
            edges = Some(match edges {
                None => (z, z),
                Some((lo, _)) => (lo, z),
            });
        }
        z += 0.001;
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_normal_cdf_is_accurate() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7);
        assert!((normal_cdf(1.959_964) - 0.975).abs() < 1e-6);
        assert!((normal_cdf(-1.0) - 0.158_655_25).abs() < 1e-6);
    }

    #[test]
    fn figure_2_jumps_between_sd_12_2_and_12_3() {
        let below = granovetter(100, 0.25, 0.122);
        let above = granovetter(100, 0.25, 0.123);
        assert!(below > 5.0 && below < 6.0, "{below}");
        assert!(above > 99.9, "{above}");
        // As σ grows without bound the equilibrium falls toward 50.
        assert!((granovetter(100, 0.25, 10.0) - 50.0).abs() < 1.5);
    }

    #[test]
    fn watts_s_poisson_window_at_phi_0_18() {
        // zQ(K* − 1, z) = 1 with K* = 1/0.18: nodes of degree ≤ 5 are vulnerable.
        let (lo, hi) = poisson_window(0.18, 0.0, 0.5, 10.0).unwrap();
        assert!(
            (lo - 1.02).abs() < 0.02 && (hi - 5.76).abs() < 0.03,
            "{lo} {hi}"
        );
        assert!(poisson_ratio(3.0, 0.18, 0.0) > 1.0);
        assert!(poisson_ratio(7.0, 0.18, 0.0) < 1.0);
        // Varied thresholds widen the window's upper side.
        let (_, wide) = poisson_window(0.18, 0.1, 0.5, 20.0).unwrap();
        assert!(wide > 10.0, "{wide}");
    }

    #[test]
    fn the_power_law_window_is_empty_at_phi_0_18() {
        for z in [1.2, 1.5, 1.9] {
            assert!(power_law_ratio(z, 0.18, 0.0) < 0.7, "{z}");
        }
        assert!(power_law_ratio(1.78, 0.05, 0.0) > 1.5);
    }
}
