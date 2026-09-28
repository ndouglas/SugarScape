//! Drawing a crowd's thresholds, and power-law degrees, bit-for-bit the same
//! on every platform (the portable logarithm and exponential only). A
//! threshold is kept as an exact fraction `num/den`, so "the others acting
//! reach θ of the group" is compared in integers: floating division stopped
//! the planning prototype's uniform crowd at 29 (29/100 × 100 < 29).

use rand::Rng;

use super::config::{Crowd, Distribution, Rounding, ThresholdsConfig};
use crate::anasazi::random::normal;
use crate::portable::{exp_neg, ln};
use crate::rng::SimRng;

/// Real thresholds are kept to six decimals.
pub const SCALE: u64 = 1_000_000;

/// A threshold θ = num/den; above 1 (num > den) the actor never acts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Th {
    pub num: u64,
    pub den: u64,
}

impl Th {
    pub const ZERO: Th = Th { num: 0, den: 1 };
    pub const NEVER: Th = Th { num: 2, den: 1 };

    /// A real fraction: 0 or less is 0, above 1 is never, else six decimals.
    pub fn from_fraction(x: f64) -> Th {
        if x <= 0.0 {
            Th::ZERO
        } else if x > 1.0 {
            Th::NEVER
        } else {
            Th {
                num: (x * SCALE as f64).round() as u64,
                den: SCALE,
            }
        }
    }

    /// `k` people out of `n` (more than `n`: never).
    pub fn people(k: i64, n: u32) -> Th {
        if k <= 0 {
            Th::ZERO
        } else if k > i64::from(n) {
            Th::NEVER
        } else {
            Th {
                num: k as u64,
                den: u64::from(n),
            }
        }
    }

    pub fn value(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// Whether `a` of `g` reaches θ: a·den ≥ num·g, exactly.
    pub fn reached(self, a: u64, g: u64) -> bool {
        u128::from(a) * u128::from(self.den) >= u128::from(self.num) * u128::from(g)
    }

    /// Whether `a` of `g` exceeds θ: a·den > num·g.
    pub fn exceeded(self, a: u64, g: u64) -> bool {
        u128::from(a) * u128::from(self.den) > u128::from(self.num) * u128::from(g)
    }
}

/// The standard normal's quantile (Acklam's rational approximation, relative
/// error under 1.2·10⁻⁹), from the portable logarithm and `sqrt` only.
pub fn inverse_normal(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    const LOW: f64 = 0.02425;
    if p < LOW {
        tail((-2.0 * ln(p)).sqrt())
    } else if p > 1.0 - LOW {
        -tail((-2.0 * ln(1.0 - p)).sqrt())
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    }
}

/// A real threshold `x` made into a `Th` under `rounding` for `n` people.
fn rounded(x: f64, rounding: Rounding, n: u32) -> Th {
    let people = x * f64::from(n);
    match rounding {
        Rounding::Exact => Th::from_fraction(x),
        Rounding::Floor => Th::people(people.floor() as i64, n),
        Rounding::Nearest => Th::people(people.round() as i64, n),
    }
}

/// The crowd's thresholds (N of them), drawn from `c`'s distribution.
pub fn draw(c: &ThresholdsConfig, rng: &mut SimRng) -> Vec<Th> {
    let n = c.actors;
    match c.distribution {
        Distribution::Uniform => (0..n).map(|i| Th::people(i64::from(i), n)).collect(),
        Distribution::Perturbed => (0..n)
            .map(|i| Th::people(i64::from(if i == 1 { 2 } else { i }), n))
            .collect(),
        Distribution::Fixed => vec![Th::from_fraction(c.mean); n as usize],
        Distribution::Normal => (0..n)
            .map(|i| {
                let z = match c.crowd {
                    Crowd::Quantiles => inverse_normal((f64::from(i) + 0.5) / f64::from(n)),
                    Crowd::Sampled => normal(rng),
                };
                rounded(c.mean + c.sd * z, c.rounding, n)
            })
            .collect(),
    }
}

/// Thresholds for `n` actors drawn from Granovetter's city: uniform 0–99 %.
pub fn city(n: u32, rng: &mut SimRng) -> Vec<Th> {
    (0..n)
        .map(|_| Th::people(i64::from(rng.gen_range(0..100u32)), 100))
        .collect()
}

/// p_k ∝ k^−2.5 e^−k/κ for k = 1..=`kmax`, normalized, with κ set so the
/// mean is `z` (1 < z < 1.95), portably: k^−2.5 = 1/(k² √k).
pub fn power_law(z: f64, kmax: u32) -> Vec<f64> {
    let dist = |kappa: f64| {
        let mut p: Vec<f64> = (1..=kmax)
            .map(|k| {
                let k = f64::from(k);
                exp_neg(-k / kappa) / (k * k * k.sqrt())
            })
            .collect();
        let s: f64 = p.iter().sum();
        for v in &mut p {
            *v /= s;
        }
        p
    };
    let mean = |p: &[f64]| {
        p.iter()
            .enumerate()
            .map(|(i, v)| (i + 1) as f64 * v)
            .sum::<f64>()
    };
    let (mut lo, mut hi) = (0.01f64, 1.0e9f64);
    for _ in 0..100 {
        let mid = (lo * hi).sqrt();
        if mean(&dist(mid)) < z {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    dist((lo * hi).sqrt())
}

/// `n` degrees drawn from `p` (p[0] is k = 1) by inverse cumulative lookup.
pub fn degrees(p: &[f64], n: u32, rng: &mut SimRng) -> Vec<u32> {
    let mut cum = Vec::with_capacity(p.len());
    let mut s = 0.0;
    for v in p {
        s += v;
        cum.push(s);
    }
    (0..n)
        .map(|_| {
            let u: f64 = rng.gen::<f64>() * s;
            cum.partition_point(|&c| c <= u).min(p.len() - 1) as u32 + 1
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    #[test]
    fn exact_thresholds_compare_in_integers() {
        // 29/100 of 100 is reached by 29, which floating division misses.
        let t = Th::people(29, 100);
        assert!(t.reached(29, 100) && !t.reached(28, 100));
        assert!(Th::ZERO.reached(0, 5) && Th::ZERO.reached(0, 0));
        assert!(!Th::NEVER.reached(10, 10));
        assert!(
            Th::from_fraction(0.9).exceeded(91, 100) && !Th::from_fraction(0.9).exceeded(90, 100)
        );
        assert_eq!(Th::from_fraction(-0.2), Th::ZERO);
        assert_eq!(Th::from_fraction(1.2), Th::NEVER);
    }

    #[test]
    fn the_uniform_and_perturbed_crowds_are_granovetter_s() {
        let c = ThresholdsConfig::default();
        let t = draw(&c, &mut rng::seeded(1));
        assert_eq!(t.len(), 100);
        assert_eq!(
            (t[0], t[1], t[99]),
            (Th::ZERO, Th::people(1, 100), Th::people(99, 100))
        );
        let p = draw(
            &ThresholdsConfig {
                distribution: Distribution::Perturbed,
                ..c
            },
            &mut rng::seeded(1),
        );
        assert_eq!((p[1], p[2]), (Th::people(2, 100), Th::people(2, 100)));
    }

    #[test]
    fn the_inverse_normal_matches_known_quantiles() {
        for (p, z) in [
            (0.5, 0.0),
            (0.975, 1.959_963_985),
            (0.1, -1.281_551_566),
            (0.001, -3.090_232_306),
        ] {
            assert!((inverse_normal(p) - z).abs() < 1e-8, "{p}");
        }
    }

    #[test]
    fn normal_crowds_follow_quantiles_or_samples_and_round() {
        let c = ThresholdsConfig {
            distribution: Distribution::Normal,
            sd: 0.12,
            ..ThresholdsConfig::default()
        };
        let q = draw(&c, &mut rng::seeded(1));
        // The median actors sit at the mean; the lowest below 0 is 0.
        assert!((q[50].value() - (0.25 + 0.12 * inverse_normal(0.505))).abs() < 1e-6);
        assert_eq!(q[0], Th::ZERO);
        let near = draw(
            &ThresholdsConfig {
                rounding: Rounding::Nearest,
                ..c.clone()
            },
            &mut rng::seeded(1),
        );
        assert!(near.iter().all(|t| t.den == 1 || t.den == 100));
        let floor = draw(
            &ThresholdsConfig {
                rounding: Rounding::Floor,
                ..c.clone()
            },
            &mut rng::seeded(1),
        );
        assert!(floor.iter().zip(&near).all(|(f, n)| f.value() <= n.value()));
        let sampled = |seed| {
            draw(
                &ThresholdsConfig {
                    crowd: Crowd::Sampled,
                    ..c.clone()
                },
                &mut rng::seeded(seed),
            )
        };
        assert_ne!(sampled(1), sampled(2));
        assert_eq!(sampled(3), sampled(3));
    }

    #[test]
    fn the_city_is_uniform_on_whole_percents() {
        let t = city(10_000, &mut rng::seeded(2));
        assert!(t.iter().all(|t| t.den == 100 || *t == Th::ZERO));
        let zeros = t.iter().filter(|t| **t == Th::ZERO).count();
        assert!((60..=140).contains(&zeros), "{zeros}");
    }

    #[test]
    fn power_laws_hit_their_mean_degree() {
        for z in [1.2, 1.5, 1.9] {
            let p = power_law(z, 5000);
            let m: f64 = p.iter().enumerate().map(|(i, v)| (i + 1) as f64 * v).sum();
            assert!((m - z).abs() < 1e-6, "{z}: {m}");
        }
        let d = degrees(&power_law(1.5, 5000), 20_000, &mut rng::seeded(3));
        let mean = d.iter().map(|&k| f64::from(k)).sum::<f64>() / 20_000.0;
        assert!((mean - 1.5).abs() < 0.1, "{mean}");
        assert!(d.iter().all(|&k| k >= 1));
    }
}
