//! Two independently weighted parents per slot, then portable L and D normals.
//! Zero variance and fixed-trait contests draw no normals. Both flags follow parent one.

use super::runner::{FounderRecord, Lineage};
use super::state::FounderTraits;
use crate::anasazi::random::normal;
use crate::hoard::{clamped_logit, inverse_logit};
use crate::rng::SimRng;
use rand::Rng;

pub(crate) struct Child {
    pub traits: FounderTraits,
    pub parents: [Lineage; 2],
}

pub(crate) fn generation_mean(founders: &[FounderRecord]) -> [f64; 2] {
    let n = founders.len() as f64;
    [
        founders
            .iter()
            .map(|f| clamped_logit(f.traits.larder))
            .sum::<f64>()
            / n,
        founders
            .iter()
            .map(|f| clamped_logit(f.traits.defense))
            .sum::<f64>()
            / n,
    ]
}

/// Logit regression is based on every archived founder, including the dead.
fn regressed_logit(midparent: f64, mean: f64, h2: f64, noise: f64) -> f64 {
    h2 * midparent + (1.0 - h2) * mean + noise
}

fn inherit(
    first: FounderTraits,
    second: FounderTraits,
    mean: [f64; 2],
    h2: f64,
    variance: f64,
    fixed: bool,
    rng: &mut SimRng,
) -> FounderTraits {
    if fixed {
        return first;
    }
    let mut trait_value = |a, b, reference| {
        let noise = if variance > 0.0 {
            variance.sqrt() * normal(rng)
        } else {
            0.0
        };
        inverse_logit(regressed_logit(
            (clamped_logit(a) + clamped_logit(b)) / 2.0,
            reference,
            h2,
            noise,
        ))
    };
    FounderTraits {
        larder: trait_value(first.larder, second.larder, mean[0]),
        defense: trait_value(first.defense, second.defense, mean[1]),
        cheater: first.cheater,
        watches: first.watches,
    }
}

/// Positive weights only; callers have already rejected extinct/zero-fitness cohorts.
fn pick(weights: &[f64], total: f64, u: f64) -> usize {
    let mut remainder = u * total;
    let mut last = 0;
    for (slot, &weight) in weights.iter().enumerate() {
        if weight <= 0.0 {
            continue;
        }
        last = slot;
        if remainder < weight {
            return slot;
        }
        remainder -= weight;
    }
    last
}

pub(crate) fn breed(
    founders: &[FounderRecord],
    weights: &[f64],
    count: usize,
    h2: f64,
    variance: f64,
    fixed: bool,
    rng: &mut SimRng,
) -> Vec<Child> {
    let total = weights.iter().sum();
    let mean = generation_mean(founders);
    (0..count)
        .map(|_| {
            let first = &founders[pick(weights, total, rng.gen())];
            let second = &founders[pick(weights, total, rng.gen())];
            Child {
                traits: inherit(first.traits, second.traits, mean, h2, variance, fixed, rng),
                parents: [first.lineage, second.lineage],
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::runner::{FounderRecord, Lineage};
    use super::*;
    use crate::rng;
    fn founder(slot: usize, l: f64, alive: bool) -> FounderRecord {
        FounderRecord {
            slot,
            traits: FounderTraits {
                larder: l,
                defense: 0.5,
                cheater: slot == 0,
                watches: slot == 0,
            },
            alive,
            ticks_alive: 1,
            holdings: 0.0,
            scatter: 0.0,
            larder: 0.0,
            parent_weight: 0.0,
            lineage: Lineage {
                generation: 4,
                slot,
            },
            parents: None,
        }
    }
    #[test]
    fn midpoint_and_dead_inclusive_regression_are_exact() {
        let f = [
            founder(0, 0.2, true),
            founder(1, 0.8, true),
            founder(2, 0.8, false),
        ];
        let mean = generation_mean(&f);
        let a = inherit(
            f[0].traits,
            f[1].traits,
            mean,
            1.0,
            0.0,
            false,
            &mut rng::seeded(3),
        );
        assert_eq!(a.larder, 0.5);
        let b = inherit(
            f[0].traits,
            f[0].traits,
            mean,
            0.0,
            0.0,
            false,
            &mut rng::seeded(3),
        );
        assert!((b.larder - 0.6135117904356906).abs() < 1e-12);
        assert!(b.cheater && b.watches);
    }
    #[test]
    fn finite_endpoints_and_fixed_first_parent_traits() {
        let mut a = founder(0, 0.0, true).traits;
        a.defense = 1.0;
        let b = founder(1, 1.0, true).traits;
        let child = inherit(a, b, [0.0, 0.0], 1.0, 0.0, false, &mut rng::seeded(3));
        assert!(child.larder.is_finite() && child.defense.is_finite());
        assert_eq!(
            inherit(a, b, [9.0, 9.0], 0.0, 10.0, true, &mut rng::seeded(3)),
            a
        );
    }
    #[test]
    fn draws_follow_parent_parent_l_normal_d_normal_in_slot_order() {
        let f = [founder(0, 0.2, true), founder(1, 0.8, true)];
        let mut actual = rng::seeded(9);
        let children = breed(&f, &[1.0, 3.0], 2, 1.0, 0.5, false, &mut actual);
        let mut expected = rng::seeded(9);
        for child in children {
            let p1 = usize::from(expected.gen::<f64>() >= 0.25);
            let p2 = usize::from(expected.gen::<f64>() >= 0.25);
            let l_noise = normal(&mut expected) * 0.5_f64.sqrt();
            let d_noise = normal(&mut expected) * 0.5_f64.sqrt();
            assert_eq!(child.parents, [f[p1].lineage, f[p2].lineage]);
            assert_eq!(
                child.traits.larder,
                inverse_logit(
                    (clamped_logit(f[p1].traits.larder) + clamped_logit(f[p2].traits.larder)) / 2.0
                        + l_noise
                )
            );
            assert_eq!(child.traits.defense, inverse_logit(d_noise));
            assert_eq!(
                (child.traits.cheater, child.traits.watches),
                (f[p1].traits.cheater, f[p1].traits.watches)
            );
        }
        assert_eq!(actual.gen::<u64>(), expected.gen::<u64>());
    }
    #[test]
    fn zero_variance_and_fixed_traits_consume_only_parent_draws() {
        let f = [founder(0, 0.2, true), founder(1, 0.8, true)];
        for (variance, fixed) in [(0.0, false), (10.0, true)] {
            let mut actual = rng::seeded(9);
            breed(&f, &[1.0, 1.0], 2, 0.8, variance, fixed, &mut actual);
            let mut expected = rng::seeded(9);
            for _ in 0..4 {
                expected.gen::<f64>();
            }
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>());
        }
    }
}
