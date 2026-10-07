use super::super::{
    controller::{advance, check},
    decision::{decide, Action, Policy, WaitReason},
};
use super::*;
use crate::{foraging::FindRecord, rng};
use rand::RngCore;

fn advance_scripted(world: &mut World, draws: &mut Scripted) -> Checked<()> {
    let mut state = world.state.clone();
    let mut pending = draws.clone();
    advance(&world.setup, &mut state, &mut pending)?;
    world.state = state;
    *draws = pending;
    Ok(())
}
fn view(w: &World, index: usize) -> Observation {
    let mut occupancy = BTreeMap::new();
    for a in &w.state.agents {
        *occupancy.entry(a.pos).or_insert(0) += 1;
    }
    observe(
        &w.setup,
        w.state.agents[index].pos,
        &occupancy,
        &w.state.ledger.available(),
    )
    .unwrap()
}
fn locate(w: &mut World, index: usize, positions: &[Pos], phase: Phase) {
    for &p in positions {
        w.state.agents[index].pos = p;
        let o = view(w, index);
        w.state.agents[index].map.learn(&o).unwrap();
    }
    w.state.agents[index].phase = phase;
}
fn decision(w: &World, index: usize, values: &[f64]) -> (super::super::decision::Decision, usize) {
    let mut d = Scripted::new(values);
    let result = decide(
        &Policy::from(&w.setup),
        &w.state.agents[index],
        &view(w, index),
        &mut d,
    )
    .unwrap();
    (result, d.next)
}
fn food_world(workers: Vec<Pos>) -> World {
    let mut s = setup();
    s.workers = workers;
    s.resources = vec![Resource {
        id: 9,
        pos: pos(3, 0),
    }];
    World::new(s, 12).unwrap()
}

#[test]
fn initial_observations_count_learning_without_draws_or_opportunities() {
    let mut s = setup();
    s.workers = vec![pos(0, 0), pos(1, 0)];
    let mut w = World::new(s, 12).unwrap();
    let a = &w.state.agents[0];
    assert_eq!(
        (
            a.compute.observations,
            a.compute.cells_inspected,
            a.compute.cells_learned
        ),
        (1, 3, 3)
    );
    assert_eq!((a.work.opportunities, a.work.uninformed_departures), (0, 1));
    assert_eq!(a.map.kind(pos(2, 0)).unwrap(), CellKnowledge::Unknown);
    assert_ne!(w.state.agents[0].map, w.state.agents[1].map);
    assert_eq!(w.rng.next_u64(), rng::seeded(12).next_u64());
    assert_eq!(w.state.all_delivered_tick, None);
}
#[test]
fn a_failed_tick_preserves_knowledge_physics_and_draw_position() {
    let mut w = World::new(setup(), 12).unwrap();
    let before = w.state.clone();
    let mut d = Scripted::new(&[]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn uninformed_switch_endpoints_and_target_equality_cost_one_draw() {
    for (p, target) in [(1.0, None), (0.0, Some(pos(0, 0)))] {
        let mut s = setup();
        s.parameters.p_search = p;
        let mut w = World::new(s, 1).unwrap();
        w.state.agents[0].frontier = target;
        let (d, n) = decision(&w, 0, &[0.8]);
        assert_eq!((d.action, n), (Action::Wait(WaitReason::Transition), 1));
        assert_eq!(d.agent.phase, Phase::Searching);
        assert_eq!(d.agent.work.search_entries, 1);
        assert_eq!(d.agent.frontier, None);
    }
}
#[test]
fn uninformed_frontier_selection_and_retention_have_exact_draw_order() {
    let mut s = setup();
    s.parameters.p_search = 0.0;
    let mut w = World::new(s, 1).unwrap();
    let (d, n) = decision(&w, 0, &[0.9, 0.0, 0.0]);
    assert_eq!(
        (d.action, d.agent.frontier, n),
        (Action::Move(pos(1, 0)), Some(pos(1, 0)), 3)
    );
    w.state.agents[0].frontier = Some(pos(1, 0));
    assert_eq!(decision(&w, 0, &[0.9, 0.0]).1, 2);
}
#[test]
fn search_give_up_precedes_current_food_and_clears_targets() {
    let mut w = food_world(vec![pos(0, 0)]);
    locate(
        &mut w,
        0,
        &[pos(1, 0), pos(2, 0), pos(3, 0)],
        Phase::Searching,
    );
    w.setup.parameters.p_return = 1.0;
    w.state.agents[0].site = Some(pos(3, 0));
    w.state.agents[0].frontier = Some(pos(2, 0));
    let (d, n) = decision(&w, 0, &[0.9]);
    assert_eq!(
        (d.action, d.agent.phase, d.agent.cargo, n),
        (
            Action::Wait(WaitReason::Transition),
            Phase::Returning,
            None,
            1
        )
    );
    assert_eq!((d.agent.site, d.agent.frontier), (None, None));
}
#[test]
fn movement_pickup_return_movement_and_deposit_are_distinct_opportunities() {
    let mut w = food_world(vec![pos(0, 0)]);
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)], Phase::Departing);
    w.state.agents[0].site = Some(pos(3, 0));
    let mut d = Scripted::new(&[0.0]);
    advance_scripted(&mut w, &mut d).unwrap();
    assert_eq!(
        (w.state.agents[0].pos, w.state.agents[0].cargo),
        (pos(3, 0), None)
    );
    advance_scripted(&mut w, &mut Scripted::new(&[])).unwrap();
    assert_eq!(w.state.agents[0].phase, Phase::Searching);
    advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    assert_eq!(w.state.agents[0].cargo, Some(9));
    assert_eq!(w.state.first_pickup_tick, Some(2));
    advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    assert_eq!(
        (w.state.agents[0].pos, w.state.ledger.inventory().delivered),
        (pos(1, 0), 0)
    );
    let known = w.state.agents[0].map.clone();
    let mut draws = Scripted::new(&[0.0, 0.9, 0.0]);
    advance_scripted(&mut w, &mut draws).unwrap();
    assert_eq!(draws.next, 3);
    assert_eq!(
        (w.state.first_delivery_tick, w.state.all_delivered_tick),
        (Some(5), Some(5))
    );
    assert_eq!(
        (w.state.agents[0].find, w.state.agents[0].cargo),
        (None, None)
    );
    assert_eq!(w.state.agents[0].work.publications, 1);
    assert_eq!(w.state.agents[0].map, known);
    assert_eq!(w.state.agents[0].work.opportunities, 6);
    assert_eq!(w.state.agents[0].work.moves, 3);
    assert_eq!(w.state.agents[0].work.pickups, 1);
    assert_eq!(w.state.agents[0].work.deposits, 1);
    check(&w.setup, &w.state).unwrap();
}
#[test]
fn local_density_uses_center_and_cardinals_and_freezes_before_claim() {
    let mut s = setup();
    s.open.extend([pos(2, 1), pos(4, 0), pos(4, 1)]);
    s.resources = vec![
        Resource {
            id: 1,
            pos: pos(3, 0),
        },
        Resource {
            id: 2,
            pos: pos(3, 1),
        },
        Resource {
            id: 3,
            pos: pos(4, 1),
        },
    ];
    let mut w = World::new(s, 1).unwrap();
    locate(
        &mut w,
        0,
        &[pos(1, 0), pos(2, 0), pos(3, 0)],
        Phase::Searching,
    );
    advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    assert_eq!(
        w.state.agents[0].find,
        Some(FindRecord { site: 3, count: 2 })
    );
}
#[test]
fn later_worker_sees_earlier_pickup_and_cannot_claim_same_token() {
    let mut w = food_world(vec![pos(0, 0), pos(1, 0)]);
    for i in 0..2 {
        locate(
            &mut w,
            i,
            &[pos(1, 0), pos(2, 0), pos(3, 0)],
            Phase::Searching,
        );
    }
    advance_scripted(&mut w, &mut Scripted::new(&[0.0, 0.0, 0.0, 0.0])).unwrap();
    assert_eq!(
        (w.state.agents[0].cargo, w.state.agents[1].cargo),
        (Some(9), None)
    );
    assert_eq!(w.state.agents[1].work.pickups, 0);
}
#[test]
fn outside_view_geometry_and_food_cannot_change_decision_or_draws() {
    let mut s = setup();
    s.parameters.p_search = 0.0;
    let a = World::new(s.clone(), 1).unwrap();
    s.open.push(pos(4, 4));
    s.resources.push(Resource {
        id: 99,
        pos: pos(4, 4),
    });
    let b = World::new(s, 1).unwrap();
    let (da, na) = decision(&a, 0, &[0.0, 0.0, 0.0]);
    let (db, nb) = decision(&b, 0, &[0.0, 0.0, 0.0]);
    assert_eq!(da, db);
    assert_eq!(na, nb);
}
#[test]
fn different_branch_visits_remain_private() {
    let mut s = setup();
    s.open.extend([pos(0, 1), pos(0, 2)]);
    s.workers = vec![pos(0, 0), pos(1, 0)];
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(0, 1)], Phase::Searching);
    locate(&mut w, 1, &[pos(2, 0)], Phase::Searching);
    assert_eq!(
        w.state.agents[0].map.kind(pos(3, 0)).unwrap(),
        CellKnowledge::Unknown
    );
    assert_eq!(
        w.state.agents[1].map.kind(pos(0, 2)).unwrap(),
        CellKnowledge::Unknown
    );
}
#[test]
fn invalid_observation_dimensions_origin_and_completeness_fail_before_draw() {
    let w = World::new(setup(), 1).unwrap();
    for case in 0..3 {
        let mut o = view(&w, 0);
        match case {
            0 => o.cells.push(ObservedCell {
                pos: pos(5, 0),
                open: true,
                occupants: 0,
                food: false,
            }),
            1 => o.origin = pos(1, 0),
            _ => {
                o.cells.pop();
            }
        }
        let mut d = Scripted::new(&[0.0]);
        assert!(decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).is_err());
        assert_eq!(d.next, 0);
    }
}
#[test]
fn invalid_injected_phase_or_arrival_draw_rolls_back() {
    for value in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
        let mut w = World::new(setup(), 1).unwrap();
        let before = w.state.clone();
        let mut d = Scripted::new(&[value]);
        assert!(advance_scripted(&mut w, &mut d).is_err());
        assert_eq!(w.state, before);
        assert_eq!(d.next, 0);
        w.state.agents[0].phase = Phase::Returning;
        for slot in 0..3 {
            let mut values = [0.0; 3];
            values[slot] = value;
            let mut d = Scripted::new(&values);
            let before = w.state.clone();
            assert!(advance_scripted(&mut w, &mut d).is_err());
            assert_eq!(w.state, before);
            assert_eq!(d.next, 0);
        }
    }
}
#[test]
fn known_solid_and_exhausted_unknown_advice_abandon_without_truth_queries() {
    let mut s = setup();
    s.open = vec![pos(0, 0), pos(1, 0)];
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(1, 0)], Phase::Departing);
    for site in [pos(2, 0), pos(4, 4)] {
        w.state.agents[0].site = Some(site);
        let (d, n) = decision(&w, 0, &[]);
        assert_eq!(d.action, Action::Wait(WaitReason::Transition));
        assert_eq!(d.agent.work.abandoned_targets, 1);
        assert_eq!(n, 0);
    }
}
#[test]
fn exhausted_search_wanders_or_waits_without_frontier_draw() {
    let mut s = setup();
    s.open = vec![pos(0, 0), pos(1, 0)];
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(0, 0)], Phase::Searching);
    assert_eq!(
        decision(&w, 0, &[0.0, 0.0]).0.action,
        Action::Move(pos(1, 0))
    );
    let mut o = view(&w, 0);
    for c in &mut o.cells {
        if c.pos != o.origin && c.open {
            c.occupants = 2;
        }
    }
    let mut d = Scripted::new(&[0.0]);
    let result = decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).unwrap();
    assert_eq!(result.action, Action::Wait(WaitReason::NoNeighbor));
    assert_eq!(d.next, 1);
}
#[test]
fn blocked_departure_retains_site_and_frontier_with_no_route_draw() {
    let mut s = setup();
    s.parameters.p_search = 0.0;
    let mut w = World::new(s, 1).unwrap();
    for site in [None, Some(pos(4, 4)), Some(pos(1, 0))] {
        w.state.agents[0].site = site;
        w.state.agents[0].frontier = Some(pos(1, 0));
        let mut o = view(&w, 0);
        o.cells
            .iter_mut()
            .find(|c| c.pos == pos(1, 0))
            .unwrap()
            .occupants = 2;
        let values = if site.is_none() { vec![0.5] } else { vec![] };
        let mut d = Scripted::new(&values);
        let result = decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).unwrap();
        assert_eq!(result.action, Action::Wait(WaitReason::Congestion));
        assert_eq!(
            (result.agent.site, result.agent.frontier),
            (site, Some(pos(1, 0)))
        );
        assert_eq!(d.next, values.len());
    }
}
#[test]
fn nearest_home_capacity_blocks_without_longer_detour_or_swaps() {
    let mut s = setup();
    s.nest = vec![pos(0, 0), pos(1, 0), pos(2, 0)];
    s.workers = vec![pos(0, 0), pos(2, 0), pos(2, 0)];
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(3, 0)], Phase::Returning);
    let (d, n) = decision(&w, 0, &[]);
    assert_eq!(d.action, Action::Wait(WaitReason::Congestion));
    assert_eq!(n, 0);
    assert_eq!(d.agent.pos, pos(3, 0));
}
#[test]
fn later_worker_observes_capacity_freed_by_earlier_move() {
    let mut s = setup();
    s.workers = vec![pos(1, 0), pos(1, 0), pos(0, 0)];
    s.parameters.p_search = 0.0;
    let mut w = World::new(s, 1).unwrap();
    w.state.agents[0].site = Some(pos(2, 0));
    w.state.agents[1].site = Some(pos(1, 0));
    w.state.agents[2].frontier = Some(pos(1, 0));
    advance_scripted(&mut w, &mut Scripted::new(&[0.0, 0.5, 0.0])).unwrap();
    assert_eq!(w.state.agents[2].pos, pos(1, 0));
}
#[test]
fn empty_arrival_cannot_republish_cleared_find_and_recruitment_imports_no_map() {
    let mut w = food_world(vec![pos(0, 0)]);
    locate(
        &mut w,
        0,
        &[pos(1, 0), pos(2, 0), pos(3, 0)],
        Phase::Searching,
    );
    advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    locate(&mut w, 0, &[pos(2, 0), pos(1, 0)], Phase::Returning);
    advance_scripted(&mut w, &mut Scripted::new(&[0.0, 0.9, 0.0])).unwrap();
    let publications = w.state.agents[0].work.publications;
    w.state.agents[0].map = Knowledge::new(5, 5).unwrap();
    let o = view(&w, 0);
    w.state.agents[0].map.learn(&o).unwrap();
    w.state.agents[0].phase = Phase::Returning;
    w.state.agents[0].site = None;
    w.state.agents[0].frontier = None;
    advance_scripted(&mut w, &mut Scripted::new(&[0.0, 0.0, 0.0])).unwrap();
    assert_eq!(w.state.agents[0].work.publications, publications);
    assert_eq!(w.state.agents[0].site, Some(pos(3, 0)));
    assert_eq!(
        w.state.agents[0].map.kind(pos(3, 0)).unwrap(),
        CellKnowledge::Unknown
    );
    assert_eq!(w.state.agents[0].work.empty_returns, 1);
}
#[test]
fn later_failure_rolls_back_earlier_move_and_learning() {
    let mut s = setup();
    s.workers = vec![pos(1, 0), pos(0, 0)];
    s.parameters.p_search = 0.0;
    let mut w = World::new(s, 1).unwrap();
    w.state.agents[0].pos = pos(2, 0);
    w.state.agents[0].site = Some(pos(3, 0));
    assert_eq!(
        w.state.agents[0].map.kind(pos(3, 0)).unwrap(),
        CellKnowledge::Unknown
    );
    let before = w.state.clone();
    let mut d = Scripted::new(&[0.0]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn later_failure_rolls_back_earlier_pickup() {
    let mut w = food_world(vec![pos(0, 0), pos(1, 0)]);
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)], Phase::Searching);
    w.state.agents[0].pos = pos(3, 0);
    assert_eq!(
        w.state.agents[0].map.kind(pos(3, 1)).unwrap(),
        CellKnowledge::Unknown
    );
    let before = w.state.clone();
    let mut d = Scripted::new(&[0.0]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn later_failure_rolls_back_deposit_publication_and_milestones() {
    let mut w = food_world(vec![pos(0, 0), pos(1, 0)]);
    locate(
        &mut w,
        0,
        &[pos(1, 0), pos(2, 0), pos(3, 0)],
        Phase::Searching,
    );
    let o = view(&w, 0);
    let d = decide(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &o,
        &mut Scripted::new(&[0.0]),
    )
    .unwrap();
    super::super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut Scripted::new(&[]))
        .unwrap();
    locate(&mut w, 0, &[pos(2, 0), pos(1, 0)], Phase::Returning);
    let before = w.state.clone();
    let mut draws = Scripted::new(&[0.0, 0.9, 0.0]);
    let errors = advance_scripted(&mut w, &mut draws).unwrap_err();
    assert_eq!(w.state, before);
    assert_eq!(draws.next, 0);
    assert!(errors[0].field.contains("tick[0].agents[1]"));
}
#[test]
fn production_checked_counter_failure_preserves_pcg_and_subsequent_replay() {
    let mut w = World::new(setup(), 12).unwrap();
    w.state.agents[0].compute.route_calls = u64::MAX;
    w.setup.parameters.p_search = 0.0;
    let before = w.state.clone();
    let mut reference = w.clone();
    assert!(w.step().is_err());
    assert_eq!(w.state, before);
    assert_eq!(w.rng.next_u64(), reference.rng.next_u64());
    w.state.agents[0].compute.route_calls = 0;
    reference.state.agents[0].compute.route_calls = 0;
    w.step().unwrap();
    reference.step().unwrap();
    assert_eq!(w.state, reference.state);
}
#[test]
fn cumulative_tick_and_opportunity_limits_fail_atomically() {
    for (workers, tick) in [(vec![pos(0, 0)], 7200), (vec![pos(0, 0); 2], 7200)] {
        let mut s = setup();
        s.workers = workers;
        let mut w = World::new(s, 1).unwrap();
        w.state.tick = tick;
        let before = w.state.clone();
        assert!(w.step().is_err());
        assert_eq!(w.state, before);
    }
    let mut s = setup();
    s.width = 125;
    s.height = 125;
    s.open = (0..128).map(|i| pos(i % 125, i / 125)).collect();
    s.nest = s.open.clone();
    s.workers = s.open.iter().flat_map(|&p| [p, p]).collect();
    let mut w = World::new(s, 1).unwrap();
    w.state.tick = 3906;
    let before = w.state.clone();
    assert!(w.step().is_err());
    assert_eq!(w.state, before);
}
#[test]
fn researcher_check_rejects_bad_identity_topology_cargo_find_and_targets() {
    let w = World::new(setup(), 1).unwrap();
    for case in 0..6 {
        let mut s = w.state.clone();
        match case {
            0 => s.agents[0].id = 1,
            1 => s.agents[0].pos = pos(4, 4),
            2 => s.agents[0].cargo = Some(9),
            3 => s.agents[0].find = Some(FindRecord { site: 3, count: 0 }),
            4 => s.agents[0].site = Some(pos(5, 0)),
            _ => s.agents[0].map = Knowledge::new(3, 3).unwrap(),
        };
        assert!(check(&w.setup, &s).is_err());
    }
}

#[test]
fn informed_unknown_frontier_draws_are_rank_then_step_and_retention_step_only() {
    let mut w = World::new(setup(), 1).unwrap();
    w.state.agents[0].site = Some(pos(3, 0));
    let (d, n) = decision(&w, 0, &[0.0, 0.0]);
    assert_eq!(
        (d.action, d.agent.frontier, n),
        (Action::Move(pos(1, 0)), Some(pos(1, 0)), 2)
    );
    w.state.agents[0].frontier = Some(pos(1, 0));
    assert_eq!(decision(&w, 0, &[0.0]).1, 1);
}
#[test]
fn uninformed_exhaustion_enters_search_with_only_phase_draw() {
    let mut s = setup();
    s.open = vec![pos(0, 0), pos(1, 0)];
    s.parameters.p_search = 0.0;
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(0, 0)], Phase::Departing);
    let (d, n) = decision(&w, 0, &[0.0]);
    assert_eq!(d.action, Action::Wait(WaitReason::Transition));
    assert_eq!(n, 1);
}
#[test]
fn search_retains_frontier_and_reselects_resolved_frontier_in_same_opportunity() {
    let mut w = World::new(setup(), 1).unwrap();
    w.state.agents[0].phase = Phase::Searching;
    assert_eq!(decision(&w, 0, &[0.0, 0.0, 0.0]).1, 3);
    w.state.agents[0].frontier = Some(pos(1, 0));
    assert_eq!(decision(&w, 0, &[0.0, 0.0]).1, 2);
    locate(&mut w, 0, &[pos(1, 0)], Phase::Searching);
    let (d, n) = decision(&w, 0, &[0.0, 0.0, 0.0]);
    assert_eq!(d.action, Action::Move(pos(2, 0)));
    assert_eq!(n, 3);
    assert_eq!(d.agent.frontier, Some(pos(2, 0)));
}
#[test]
fn full_outside_next_cell_retains_loaded_food_and_target_without_displacement() {
    let mut w = food_world(vec![pos(0, 0), pos(1, 0), pos(1, 0)]);
    locate(&mut w, 0, &[pos(3, 0)], Phase::Searching);
    let o = view(&w, 0);
    let d = decide(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &o,
        &mut Scripted::new(&[0.0]),
    )
    .unwrap();
    super::super::actions::apply(&w.setup, &mut w.state, 0, d, &o, &mut Scripted::new(&[]))
        .unwrap();
    locate(&mut w, 1, &[pos(2, 0)], Phase::Departing);
    locate(&mut w, 2, &[pos(2, 0)], Phase::Departing);
    let (d, n) = decision(&w, 0, &[]);
    assert_eq!(d.action, Action::Wait(WaitReason::Congestion));
    assert_eq!(n, 0);
    assert_eq!(d.agent.cargo, Some(9));
    assert_eq!(w.state.ledger.inventory().delivered, 0);
    assert_eq!(w.state.agents[1].pos, pos(2, 0));
}
#[test]
fn missing_private_route_home_is_an_invariant_error_without_draws() {
    let mut w = World::new(setup(), 1).unwrap();
    locate(&mut w, 0, &[pos(3, 0)], Phase::Returning);
    w.state.agents[0].map = Knowledge::new(5, 5).unwrap();
    let o = view(&w, 0);
    w.state.agents[0].map.learn(&o).unwrap();
    let mut d = Scripted::new(&[]);
    assert!(decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).is_err());
    assert_eq!(d.next, 0);
}
#[test]
fn final_legal_tick_succeeds_and_empty_world_does_not_finish() {
    let mut w = World::new(setup(), 1).unwrap();
    w.state.tick = 7199;
    w.step().unwrap();
    assert_eq!(w.state.tick, 7200);
    assert_eq!(w.state.all_delivered_tick, None);
    assert!(w.step().is_err());
}
#[test]
fn physical_application_rejects_nonlocal_solid_and_full_moves() {
    let w = World::new(setup(), 1).unwrap();
    for target in [pos(4, 4), pos(0, 1)] {
        let mut state = w.state.clone();
        let (mut d, _) = decision(&w, 0, &[0.0]);
        d.action = Action::Move(target);
        assert!(super::super::actions::apply(
            &w.setup,
            &mut state,
            0,
            d,
            &view(&w, 0),
            &mut Scripted::new(&[])
        )
        .is_err());
    }
}

#[test]
fn invariant_counter_errors_identify_processing_tick_and_worker() {
    let mut w = World::new(setup(), 1).unwrap();
    w.state.agents[0].work.opportunities = 1;
    let errors = w.step().unwrap_err();
    assert!(errors[0]
        .field
        .contains("tick[0].agents[0].work.opportunities"));
}

#[test]
fn searching_move_into_food_cannot_pick_up_until_next_opportunity() {
    let mut w = food_world(vec![pos(0, 0)]);
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)], Phase::Searching);
    advance_scripted(&mut w, &mut Scripted::new(&[0.0, 0.0, 0.0])).unwrap();
    assert_eq!(
        (w.state.agents[0].pos, w.state.agents[0].cargo),
        (pos(3, 0), None)
    );
    advance_scripted(&mut w, &mut Scripted::new(&[0.0])).unwrap();
    assert_eq!(w.state.agents[0].cargo, Some(9));
}

#[test]
fn returning_equal_distance_home_steps_use_all_known_nest_sources() {
    let mut s = setup();
    s.open
        .extend([pos(0, 1), pos(0, 2), pos(1, 1), pos(1, 2), pos(2, 2)]);
    s.nest = vec![
        pos(0, 0),
        pos(1, 0),
        pos(2, 0),
        pos(0, 1),
        pos(0, 2),
        pos(1, 2),
        pos(2, 2),
    ];
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(1, 1)], Phase::Returning);
    for (draw, want) in [(0.0, pos(1, 0)), (0.5, pos(1, 2)), (0.99, pos(0, 1))] {
        let (d, n) = decision(&w, 0, &[draw]);
        assert_eq!((d.action, n), (Action::Move(want), 1));
    }
}
