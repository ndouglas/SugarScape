//! Minds 5: Raby's and Amodio's protocols, with each hypothesis as an oracle.
//!
//! Every expected pattern below is derived by hand (the derivations are
//! written beside each test) and asserted exactly: a rule that misses its
//! oracle is a bug, not a finding. Caches are [K1, K2, K3] in whole units
//! of the test evening's F = 30; whole-unit remainders go to K1 < K2 < K3.

use sugarscape_core::config::{CachingRule, Lab, LabProtocol};
use sugarscape_core::minds::caching::lab::{run_lab, run_population, LabParams};

fn raby(breakfast_first: bool) -> Lab {
    Lab {
        protocol: LabProtocol::Raby,
        food_first: breakfast_first,
    }
}

fn amodio(food_first: bool) -> Lab {
    Lab {
        protocol: LabProtocol::Amodio,
        food_first,
    }
}

fn params(lookahead: u32) -> LabParams {
    LabParams {
        lookahead,
        ..LabParams::default()
    }
}

fn caches(lab: Lab, rule: CachingRule, lookahead: u32) -> [u32; 3] {
    let c = run_lab(lab, rule, params(lookahead), 1).caches;
    // The schedule draws nothing: any seed gives the same caches.
    assert_eq!(run_lab(lab, rule, params(lookahead), 99).caches, c);
    c
}

// Raby: K1 on days 1, 3, 5 and K3 on days 2, 4, 6. Breakfast first puts
// breakfast in K1; otherwise in K3. Caching trays in K1 and K3 only.

/// `even`: 30 / 2 = 15 in each tray, whichever compartment had breakfast.
#[test]
fn raby_even_ties() {
    for first in [true, false] {
        assert_eq!(caches(raby(first), CachingRule::Even, 1), [15, 0, 15]);
    }
}

/// `compensate`, λ = 0.5: the breakfast compartment found food three times,
/// w = 0.5³ = 0.125; the other never, w = 1. Shares: 30 × 1 / 1.125 =
/// 26.67 and 30 × 0.125 / 1.125 = 3.33, floored 26 and 3 (29); the unit
/// left goes to K1.
/// - Breakfast in K3 (not first): K1 = 26 + 1 = 27, K3 = 3.
/// - Breakfast in K1 (first): K1 = 3 + 1 = 4, K3 = 26. The remainder rule
///   gives the breakfast compartment the extra unit here, so the split is
///   27/3 in one counterbalancing and 26/4 in the other.
#[test]
fn raby_compensate_caches_most_where_there_was_no_breakfast() {
    assert_eq!(caches(raby(false), CachingRule::Compensate, 1), [27, 0, 3]);
    assert_eq!(caches(raby(true), CachingRule::Compensate, 1), [4, 0, 26]);
}

/// `plan`, lookahead 1: places K1, K3, K1, K3, K1, K3 have period 2, so
/// day 7 is K1. Food alternates with period 2, so day 7's food is day 1's.
/// - Breakfast first: food T, F, T, F, T, F → day 7 has food; nothing is
///   needed, nothing cached.
/// - Breakfast second: F, T, F, T, F, T → day 7 in K1 without food: all 30
///   in K1, the no-breakfast compartment.
#[test]
fn raby_plan_provisions_only_when_tomorrows_compartment_lacks_breakfast() {
    assert_eq!(caches(raby(true), CachingRule::Plan, 1), [0, 0, 0]);
    assert_eq!(caches(raby(false), CachingRule::Plan, 1), [30, 0, 0]);
}

/// `plan`, lookahead 3 (days 7, 8, 9: K1, K3, K1). Breakfast first: food
/// T, F, T → only K3 lacks it: all 30 in K3. Breakfast second: F, T, F →
/// only K1: all 30 in K1. Either way, all in the no-breakfast compartment.
#[test]
fn raby_plan_looking_three_days_ahead_caches_in_the_no_breakfast_compartment() {
    assert_eq!(caches(raby(true), CachingRule::Plan, 3), [0, 0, 30]);
    assert_eq!(caches(raby(false), CachingRule::Plan, 3), [30, 0, 0]);
}

// Amodio Experiment 2: K1 on days 1, 4, 7; K2 on 2, 5, 8; K3 on 3, 6, 9.
// Food-First has food on odd days: K1 on 1 and 7, K2 on 5, K3 on 3 and 9.
// Empty-First on even days: K2 on 2 and 8, K1 on 4, K3 on 6.

/// `even`: 30 / 3 = 10 each, in both groups.
#[test]
fn amodio_even_ties() {
    for first in [true, false] {
        assert_eq!(caches(amodio(first), CachingRule::Even, 1), [10, 10, 10]);
    }
}

/// `compensate`, λ = 0.5.
/// - Food-First: w = 0.25, 0.5, 0.25 (Σ 1): 7.5, 15, 7.5 → 7, 15, 7 (29),
///   the unit left to K1: 8, 15, 7. Most in K2 (CCH).
/// - Empty-First: w = 0.5, 0.25, 0.5 (Σ 1.25): 12, 6, 12 exactly.
#[test]
fn amodio_compensate_caches_where_food_was_rarest() {
    assert_eq!(caches(amodio(true), CachingRule::Compensate, 1), [8, 15, 7]);
    assert_eq!(
        caches(amodio(false), CachingRule::Compensate, 1),
        [12, 6, 12]
    );
}

/// `plan`, lookahead 1: places have period 3, so day 10 is K1; food has
/// period 2, so day 10's food is day 8's.
/// - Food-First: day 8 had none → K1 lacks food on day 10: all 30 in K1
///   (FPH 1).
/// - Empty-First: day 8 had food → day 10 in K1 has food: nothing cached.
///   (The paper's FPH 1 for this group provisions for "the nearest relevant
///   future event", day 11 in K2; a lookahead of 1 sees only day 10.)
#[test]
fn amodio_plan_provisions_for_tomorrow() {
    assert_eq!(caches(amodio(true), CachingRule::Plan, 1), [30, 0, 0]);
    assert_eq!(caches(amodio(false), CachingRule::Plan, 1), [0, 0, 0]);
}

/// `plan`, lookahead 3: days 10, 11, 12 are K1, K2, K3.
/// - Food-First: food F, T, F (days 8, 9, 8's parity) → K1 and K3 lack it:
///   15 in each. The paper states FPH 2 for this group as "distribute the
///   caches across compartments K1 and K2"; the cycle finder puts them in
///   K1 and K3 (day 11, in K2, has food on the 2-day cycle). Reported,
///   not resolved.
/// - Empty-First: food T, F, T → only K2 (day 11) lacks it: all 30 in K2,
///   the paper's FPH 2 (and FPH 1) for this group.
#[test]
fn amodio_plan_looking_three_days_ahead() {
    assert_eq!(caches(amodio(true), CachingRule::Plan, 3), [15, 0, 15]);
    assert_eq!(caches(amodio(false), CachingRule::Plan, 3), [0, 30, 0]);
}

/// A lab population: per-agent (share, λ) drawn from 0.4–0.6 and 0.3–0.7.
/// Only λ matters in the lab (it caches all of F), so `even` and `plan`
/// give every agent the same caches, while `compensate` spreads: with λ in
/// [0.3, 0.7) the breakfast compartment's w = (1 − λ)³ lies in (0.027,
/// 0.343]. With two places and whole units, K3 (breakfast) = ⌊30w /
/// (1 + w)⌋ and K1 = 30 − K3 (the remainder goes to K1). At w = 0.343,
/// 30w / (1 + w) = 7.66, so K3 ≤ 7 and K1 ∈ 23..=30; K3 = 0 (all 30 in
/// K1) whenever 30w < 1 + w, i.e. w < 1/29, i.e. λ > 0.6745. Every agent
/// caches more in the no-breakfast compartment.
#[test]
fn a_raby_population_differs_only_under_compensate() {
    let p = LabParams::default();
    let even = run_population(raby(false), CachingRule::Even, p, 8, 5);
    assert_eq!(even.len(), 8);
    assert!(even.iter().all(|r| r.caches == [15, 0, 15]));
    let plan = run_population(raby(false), CachingRule::Plan, p, 8, 5);
    assert!(plan.iter().all(|r| r.caches == [30, 0, 0]));
    let comp = run_population(raby(false), CachingRule::Compensate, p, 8, 5);
    assert_eq!(comp.len(), 8);
    for r in &comp {
        let [k1, k2, k3] = r.caches;
        assert_eq!((k1 + k3, k2), (30, 0));
        assert!(k1 > k3, "{r:?}");
        assert!((23..=30).contains(&k1), "{r:?}");
    }
    let spread: std::collections::BTreeSet<u32> = comp.iter().map(|r| r.caches[0]).collect();
    assert!(spread.len() > 1, "individual differences: {spread:?}");
    // Deterministic by seed.
    assert_eq!(
        run_population(raby(false), CachingRule::Compensate, p, 8, 5),
        comp
    );
}

#[test]
fn an_amodio_population_of_six_caches_every_unit() {
    let p = params(3);
    for rule in [CachingRule::Even, CachingRule::Compensate] {
        for first in [true, false] {
            let rs = run_population(amodio(first), rule, p, 6, 2);
            assert_eq!(rs.len(), 6);
            assert!(rs.iter().all(|r| r.caches.iter().sum::<u32>() == 30));
        }
    }
    let rs = run_population(amodio(true), CachingRule::Plan, p, 6, 2);
    assert!(rs.iter().all(|r| r.caches == [15, 0, 15]));
}
