use super::resources::*;
#[test]
fn loss_reduces_next_capacity_and_artifact_sign_increases_it() {
    assert!((capacity(10.0, 8.0, 2.0, 0.25, false) - 9.0).abs() < 1e-12);
    assert!((capacity(10.0, 8.0, 2.0, 0.25, true) - 10.0).abs() < 1e-12);
}
#[test]
fn distance_curves_follow_source_contradiction_and_center_override() {
    assert_eq!(distance_curve(0.0, 0.1, 2.0, 3.0, true), 1.0);
    assert!((distance_curve(4.0, 0.1, 2.0, 3.0, false) - 0.2).abs() < 1e-12);
    assert!((distance_curve(4.0, 0.1, 2.0, 3.0, true) - 0.9).abs() < 1e-12);
}
#[test]
fn passive_plans_are_conditional_not_an_additive_budget() {
    assert_eq!(commitment(100.0, 0.5, 5, 15.0, 40.0, true), 28.75);
    assert!((commitment(100.0, 0.5, 5, 20.0, 40.0, false) - 26.666666666666668).abs() < 1e-12);
    assert_eq!(commitment(100.0, 0.5, 5, 0.0, 0.0, false), 60.0);
    assert_eq!(commitment(100.0, 0.5, 0, 0.0, 0.0, false), 0.0);
}
#[test]
fn probabilistic_superiority_is_increasing_and_half_at_threshold() {
    assert_eq!(probability(3.0, 3.0, 20), 0.5);
    assert!(probability(6.0, 3.0, 20) > 0.999);
    assert!(probability(1.5, 3.0, 20) < 0.001);
}
use super::*;
use std::collections::BTreeMap;
pub(super) fn prescribed(owners: &[usize], width: u32) -> GeosimWorld {
    let c = GeosimConfig {
        width,
        height: owners.len() as u32 / width,
        initial_states: 1,
        initialization_periods: 0,
        observation_periods: 30,
        attack_probability: 0.0,
        shock_probability: 0.0,
        ..Default::default()
    };
    let mut w = GeosimWorld::new(c, 7).unwrap();
    w.states = BTreeMap::new();
    for (i, &owner) in owners.iter().enumerate() {
        let id = StateId {
            capital_cell: owner,
            sovereignty_generation: 0,
        };
        w.cells[i].owner = id;
        w.states.entry(id).or_insert(State {
            id,
            capacity: Some(20.0),
            threshold: 2.0,
            alert: false,
            campaign: None,
            previous_damage: 0.0,
            newly_independent: false,
            extracted_yield: 0.0,
            recurrence_residual: 0.0,
        });
    }
    w.rebuild();
    w
}

#[test]
fn proven_distance_projection_uses_common_software_power_bits() {
    let distance = f64::from_bits(0x4001e3779b97f4a8);
    assert_eq!(
        distance_curve(distance, 0.1, 2.0, 3.0, false).to_bits(),
        0x3fde6cb29b754c8a
    );
}
#[test]
fn proven_contest_input_uses_common_software_log_exp_bits() {
    assert_eq!(
        probability(0.058823529411764705, 3.0, 20).to_bits(),
        0x38d7731ada15eef0
    );
}
