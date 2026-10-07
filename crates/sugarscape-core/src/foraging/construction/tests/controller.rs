use super::super::{
    actions::apply,
    controller::{advance, check, occupancy},
    decision::{decide, Action, Decision, Policy, WaitReason},
    world::World,
};
use super::*;
use rand::Rng;

fn view(w: &World, i: usize) -> Observation {
    observe(
        &w.state.terrain,
        w.state.agents[i].pos,
        &occupancy(&w.state),
        &w.state.food.available(),
    )
    .unwrap()
}
fn advance_scripted(w: &mut World, d: &mut Scripted) -> Checked<()> {
    let mut candidate = w.state.clone();
    let mut draws = d.clone();
    advance(&w.setup, &mut candidate, &mut draws)?;
    w.state = candidate;
    *d = draws;
    Ok(())
}
fn tick(w: &mut World, values: &[f64]) {
    let mut d = Scripted::new(values);
    advance_scripted(w, &mut d).unwrap();
    assert_eq!(d.next, values.len());
}
// Private fixture actions still learn complete ordinary observations, and use
// the same physical apply transaction; no teleporting or through-wall learning.
fn action(w: &mut World, i: usize, a: Action, values: &[f64]) -> Checked<()> {
    let o = view(w, i);
    let mut candidate = w.state.clone();
    let mut d = Scripted::new(values);
    let mut agent = candidate.agents[i].clone();
    let delta = agent.map.learn(&o)?;
    agent.compute.checked_include(&ComputeCounts {
        observations: 1,
        cells_inspected: o.cells.len() as u64,
        cells_learned: delta.first,
        observed_revisions: delta.observed_revisions,
        ..Default::default()
    })?;
    apply(
        &w.setup,
        &mut candidate,
        i,
        Decision { agent, action: a },
        &o,
        &mut d,
    )?;
    w.state = candidate;
    Ok(())
}
fn searching_at_face() -> World {
    let mut w = World::new(setup(), 12).unwrap();
    tick(&mut w, &[0.0]);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    tick(&mut w, &[0.5, 0.0]);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    w
}
#[test]
fn construction_observes_once_starts_one_trip_and_consumes_zero_draws() {
    let mut w = World::new(setup(), 12).unwrap();
    let mut r = crate::rng::seeded(12);
    assert_eq!(w.rng.gen::<u64>(), r.gen::<u64>());
    let a = &w.state.agents[0];
    assert_eq!(a.work.uninformed_departures, 1);
    assert_eq!(a.work.opportunities, 0);
    assert_eq!(a.compute.observations, 1);
    assert_eq!(a.compute.cells_learned, 3);
    assert_eq!(a.mode(), Mode::Departing);
    assert_eq!(w.state.food.inventory().hidden, 1);
}
#[test]
fn hidden_food_cannot_change_decisions_or_draws() {
    let w = searching_at_face();
    let mut s = setup();
    s.food.clear();
    let mut empty = World::new(s, 12).unwrap();
    tick(&mut empty, &[0.0]);
    tick(&mut empty, &[0.5, 0.0, 0.0]);
    tick(&mut empty, &[0.5, 0.0, 0.0]);
    tick(&mut empty, &[0.5, 0.0]);
    tick(&mut empty, &[0.5, 0.0, 0.0]);
    let mut d = Scripted::new(&[0.5, 0.0]);
    let mut e = d.clone();
    let a = decide(
        &Policy::from(&w.setup),
        &w.state.agents[0],
        &view(&w, 0),
        &mut d,
    )
    .unwrap();
    let b = decide(
        &Policy::from(&empty.setup),
        &empty.state.agents[0],
        &view(&empty, 0),
        &mut e,
    )
    .unwrap();
    assert_eq!(a, b);
    assert_eq!(d.next, e.next);
    assert_eq!(a.action, Action::Dig(pos(3, 0)));
}
#[test]
fn buried_food_cycle_pays_separate_dig_disposal_pickup_and_deposit() {
    let mut w = searching_at_face();
    assert_eq!(w.state.agents[0].pos, pos(2, 0));
    tick(&mut w, &[0.5, 0.0]); // give-up, new face; adjacent Dig, no step draw
    assert_eq!(w.state.agents[0].cargo, Some(Cargo::Spoil(0)));
    assert_eq!(w.state.agents[0].pos, pos(2, 0));
    assert_eq!(w.state.food.inventory().available, 1);
    assert_eq!(w.state.agents[0].work.pickups, 0);
    assert_eq!(w.state.agents[0].compute.dig_confirmations, 1);
    assert_eq!(
        w.state
            .milestones
            .first_access
            .as_ref()
            .unwrap()
            .context
            .opportunity,
        6
    );
    tick(&mut w, &[0.0]); // known outlet route step to nest
    tick(&mut w, &[0.0]);
    tick(&mut w, &[0.0]); // reach outlet; movement alone
    assert_eq!(w.state.agents[0].pos, pos(0, 1));
    assert_eq!(w.state.spoil.inventory().disposed, 0);
    let entries = w.state.agents[0].work.search_entries;
    tick(&mut w, &[]); // disposal no advice or food-trip draws
    assert_eq!(w.state.agents[0].phase, FoodPhase::Searching);
    assert_eq!(w.state.agents[0].work.search_entries, entries);
    assert_eq!(w.state.agents[0].work.empty_returns, 0);
    assert_eq!(w.state.agents[0].work.uninformed_departures, 1);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    tick(&mut w, &[0.5, 0.0]);
    tick(&mut w, &[0.5, 0.0]);
    tick(&mut w, &[0.5, 0.0]);
    assert_eq!(w.state.agents[0].pos, pos(3, 0));
    tick(&mut w, &[0.5]); // pickup no movement
    assert_eq!(w.state.agents[0].cargo, Some(Cargo::Food(u64::MAX)));
    assert_eq!(w.state.food.inventory().delivered, 0);
    tick(&mut w, &[0.0]);
    tick(&mut w, &[0.0]);
    assert_eq!(w.state.agents[0].pos, pos(1, 0));
    tick(&mut w, &[0.0, 0.0, 0.0]); // food deposit, three arrival draws
    assert_eq!(w.state.food.inventory().delivered, 1);
    assert_eq!(w.state.spoil.inventory().disposed, 1);
    check(&w.setup, &w.state).unwrap();
}
#[test]
fn late_failure_preserves_candidate_terrain_and_rng() {
    let mut w = World::new(setup(), 12).unwrap();
    let before = w.state.clone();
    let mut d = Scripted::new(&[]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn late_worker_failure_rolls_back_dig_exposure_confirmation_observer_and_draws() {
    let mut s = setup();
    s.workers.push(pos(1, 0));
    let mut w = World::new(s, 12).unwrap();
    locate(&mut w, 0, &[pos(0, 1), pos(0, 0), pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    let before = w.state.clone();
    let mut d = Scripted::new(&[0.5, 0.0]);
    let errors = advance_scripted(&mut w, &mut d).unwrap_err();
    assert!(errors[0].field.contains("agents[1]"));
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn malformed_observation_is_rejected_before_learning_or_rng() {
    let w = World::new(setup(), 12).unwrap();
    let mut o = view(&w, 0);
    o.cells.pop();
    let mut d = Scripted::new(&[0.0]);
    assert!(decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).is_err());
    assert_eq!(d.next, 0);
}
#[test]
fn cargo_priority_and_spoil_resumption_preserve_food_site() {
    let mut w = searching_at_face();
    w.state.agents[0].phase = FoodPhase::Departing;
    w.state.agents[0].site = Some(pos(3, 0));
    tick(&mut w, &[0.0]);
    assert_eq!(w.state.agents[0].cargo, Some(Cargo::Spoil(0)));
    assert_eq!(w.state.agents[0].site, Some(pos(3, 0)));
    tick(&mut w, &[0.0]);
    tick(&mut w, &[0.0]);
    tick(&mut w, &[0.0]);
    tick(&mut w, &[]);
    assert_eq!(w.state.agents[0].phase, FoodPhase::Departing);
    assert_eq!(w.state.agents[0].site, Some(pos(3, 0)));
    assert_eq!(w.state.agents[0].work.search_entries, 1);
}

fn locate(w: &mut World, i: usize, path: &[Pos]) {
    // Each fixture relocation is a real cardinal Move, plus one paid wait for
    // every other worker, preserving complete-tick event numbering/capacity.
    for &p in path {
        for j in 0..w.state.agents.len() {
            let a = if j == i {
                Action::Move(p)
            } else {
                Action::Wait(WaitReason::NoNeighbor)
            };
            action(w, j, a, &[]).unwrap();
        }
        w.state.tick += 1;
    }
    let o = view(w, i);
    let a = &mut w.state.agents[i];
    let delta = a.map.learn(&o).unwrap();
    a.compute
        .checked_include(&ComputeCounts {
            observations: 1,
            cells_inspected: o.cells.len() as u64,
            cells_learned: delta.first,
            observed_revisions: delta.observed_revisions,
            ..Default::default()
        })
        .unwrap();
    check(&w.setup, &w.state).unwrap();
}
fn choice(w: &World, i: usize, values: &[f64]) -> Decision {
    let mut d = Scripted::new(values);
    let result = decide(
        &Policy::from(&w.setup),
        &w.state.agents[i],
        &view(w, i),
        &mut d,
    )
    .unwrap();
    assert_eq!(d.next, values.len());
    result
}
fn paired_face() -> World {
    let mut s = setup();
    s.workers.push(pos(1, 0));
    let mut w = World::new(s, 12).unwrap();
    locate(&mut w, 0, &[pos(0, 1), pos(0, 0), pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    w
}
#[test]
fn uninformed_switch_endpoint_and_equality_always_draw_before_transition() {
    let mut w = World::new(setup(), 1).unwrap();
    for (probability, value) in [(1.0, 0.0), (0.0, 0.0), (0.5, 0.5)] {
        w.setup.parameters.p_search = probability;
        w.state.agents[0].frontier = Some(pos(0, 0));
        let d = choice(&w, 0, &[value]);
        assert_eq!(d.action, Action::Wait(WaitReason::Transition));
        assert_eq!(d.agent.work.search_entries, 1);
    }
}
#[test]
fn departing_frontier_selection_retention_and_blocking_have_exact_draws() {
    let mut w = World::new(setup(), 1).unwrap();
    w.setup.parameters.p_search = 0.0;
    let d = choice(&w, 0, &[0.0, 0.999, 0.0]);
    assert_eq!(d.action, Action::Move(pos(1, 0)));
    assert_eq!(d.agent.frontier, Some(pos(1, 0)));
    w.state.agents[0].frontier = Some(pos(1, 0));
    let d = choice(&w, 0, &[0.0, 0.0]);
    assert_eq!(d.action, Action::Move(pos(1, 0)));
    let mut o = view(&w, 0);
    o.cells
        .iter_mut()
        .find(|c| c.pos == pos(1, 0))
        .unwrap()
        .occupants = 2;
    let mut draws = Scripted::new(&[0.0]);
    let d = decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut draws).unwrap();
    assert_eq!(draws.next, 1);
    assert_eq!(d.action, Action::Wait(WaitReason::Congestion));
    assert_eq!(d.agent.frontier, Some(pos(1, 0)));
}
#[test]
fn search_give_up_precedes_current_food_pickup_and_clears_all_targets() {
    let mut s = setup();
    s.food = vec![Resource {
        id: 0,
        pos: pos(2, 0),
    }];
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    w.state.agents[0].face = Some(pos(3, 0));
    w.setup.parameters.p_return = 1.0;
    let d = choice(&w, 0, &[0.0]);
    assert_eq!(d.action, Action::Wait(WaitReason::Transition));
    assert_eq!(
        (d.agent.find, d.agent.site, d.agent.frontier, d.agent.face),
        (None, None, None, None)
    );
    w.setup.parameters.p_return = 0.0;
    assert_eq!(choice(&w, 0, &[0.0]).action, Action::PickupFood);
}
#[test]
fn protected_informed_site_abandons_but_stale_diggable_site_keeps_pursuit() {
    let mut w = searching_at_face();
    w.state.agents[0].phase = FoodPhase::Departing;
    w.state.agents[0].site = Some(pos(2, 1));
    let d = choice(&w, 0, &[]);
    assert_eq!(d.action, Action::Wait(WaitReason::Transition));
    assert_eq!(d.agent.work.abandoned_targets, 1);
    w.state.agents[0].site = Some(pos(3, 0));
    let d = choice(&w, 0, &[0.0]);
    assert_eq!(d.action, Action::Dig(pos(3, 0)));
    assert_eq!(d.agent.site, Some(pos(3, 0)));
    w.state.agents[0].site = Some(pos(2, 0));
    assert_eq!(
        choice(&w, 0, &[]).action,
        Action::Wait(WaitReason::Transition)
    );
}
#[test]
fn new_open_frontier_clears_face_before_any_excavation() {
    let mut w = World::new(setup(), 1).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    w.state.agents[0].face = Some(pos(3, 0));
    let d = choice(&w, 0, &[0.5, 0.0, 0.0]);
    assert_eq!(d.action, Action::Move(pos(1, 0)));
    assert_eq!(d.agent.face, None);
    assert_eq!(d.agent.frontier, Some(pos(0, 1)));
}
#[test]
fn retained_remote_face_blocked_route_consumes_only_give_up_draw() {
    let mut w = searching_at_face();
    locate(&mut w, 0, &[pos(1, 0), pos(0, 0)]);
    w.state.agents[0].face = Some(pos(3, 0));
    let mut o = view(&w, 0);
    o.cells
        .iter_mut()
        .find(|c| c.pos == pos(1, 0))
        .unwrap()
        .occupants = 2;
    let mut d = Scripted::new(&[0.5]);
    let result = decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).unwrap();
    assert_eq!(result.action, Action::Wait(WaitReason::Congestion));
    assert_eq!(result.agent.face, Some(pos(3, 0)));
    assert_eq!(d.next, 1);
}
#[test]
fn stale_wall_survives_remote_dig_until_local_observation_and_face_is_dropped() {
    let mut w = paired_face();
    locate(&mut w, 1, &[pos(2, 0), pos(1, 0), pos(0, 0)]);
    w.state.agents[1].phase = FoodPhase::Searching;
    w.state.agents[1].face = Some(pos(3, 0));
    tick(&mut w, &[0.5, 0.0, 0.5, 0.0, 0.0]); // digger Dig; remote chooses waste frontier
    assert_eq!(
        w.state.agents[1].map.kind(pos(3, 0)).unwrap(),
        CellKnowledge::KnownSolid { diggable: true }
    );
    assert_eq!(w.state.agents[1].compute.observed_revisions, 0);
    check(&w.setup, &w.state).unwrap();
    locate(&mut w, 1, &[pos(0, 0), pos(1, 0), pos(2, 0)]);
    assert_eq!(w.state.agents[1].compute.observed_revisions, 1);
    w.state.agents[1].face = Some(pos(3, 0)); // tests policy reconciliation after fresh sensing
    let d = choice(&w, 1, &[0.5, 0.0, 0.0]);
    assert_eq!(d.action, Action::Move(pos(3, 0)));
    assert_eq!(d.agent.face, None);
}
#[test]
fn later_local_worker_sees_earlier_dig_food_but_does_not_collect_in_move() {
    let mut w = paired_face();
    locate(&mut w, 1, &[pos(2, 0)]);
    w.state.agents[1].phase = FoodPhase::Searching;
    tick(&mut w, &[0.5, 0.0, 0.5, 0.999, 0.0]);
    assert_eq!(w.state.agents[1].compute.observed_revisions, 1);
    assert_eq!(w.state.agents[1].pos, pos(3, 0));
    assert_eq!(w.state.agents[1].cargo, None);
    let d = choice(&w, 1, &[0.5]);
    assert_eq!(d.action, Action::PickupFood);
}
#[test]
fn spoil_unknown_outlet_explores_by_label_without_phase_draw_or_dig() {
    let mut s = setup();
    s.open.push(pos(0, 2));
    s.waste = pos(0, 2);
    let mut w = World::new(s, 12).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    action(&mut w, 0, Action::Dig(pos(3, 0)), &[]).unwrap();
    w.state.tick += 1;
    assert_eq!(
        w.state.agents[0].map.kind(pos(0, 2)).unwrap(),
        CellKnowledge::Unknown
    );
    let d = choice(&w, 0, &[0.0, 0.0]);
    assert_eq!(d.action, Action::Move(pos(1, 0)));
    assert_eq!(d.agent.frontier, Some(pos(0, 1)));
    assert_eq!(d.agent.face, None);
}
#[test]
fn wrong_material_destination_and_full_hands_transactions_roll_back() {
    // Keep another fresh, physically eligible face beside full hands. This
    // failure must exercise the cargo guard rather than an already-open face.
    let mut substrate = setup();
    substrate.diggable.push(pos(2, 1));
    let mut loaded = World::new(substrate, 12).unwrap();
    locate(
        &mut loaded,
        0,
        &[pos(0, 1), pos(0, 0), pos(1, 0), pos(2, 0)],
    );
    loaded.state.agents[0].phase = FoodPhase::Searching;
    action(&mut loaded, 0, Action::Dig(pos(3, 0)), &[]).unwrap();
    loaded.state.tick += 1;
    assert!(loaded.state.terrain.is_diggable(pos(2, 1)).unwrap());
    let before = loaded.state.clone();
    assert!(action(&mut loaded, 0, Action::Dig(pos(2, 1)), &[]).is_err());
    assert_eq!(loaded.state, before);
    let mut w = searching_at_face();
    tick(&mut w, &[0.5, 0.0]);
    for a in [
        Action::Dig(pos(3, 0)),
        Action::PickupFood,
        Action::DepositFood,
        Action::DisposeSpoil,
    ] {
        let before = w.state.clone();
        assert!(action(&mut w, 0, a, &[]).is_err());
        assert_eq!(w.state, before);
    }
    locate(&mut w, 0, &[pos(1, 0)]);
    let before = w.state.clone();
    assert!(action(&mut w, 0, Action::DepositFood, &[0.0; 3]).is_err());
    assert_eq!(w.state, before);
}
#[test]
fn late_failure_restores_earlier_move_learning_and_script_position() {
    let mut s = setup();
    s.workers.push(pos(1, 0));
    let mut w = World::new(s, 12).unwrap();
    w.state.agents[0].phase = FoodPhase::Searching;
    let before = w.state.clone();
    let mut d = Scripted::new(&[0.5, 0.0, 0.0]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn late_failure_restores_spoil_disposal_milestones_and_food_intent() {
    let mut w = paired_face();
    tick(&mut w, &[0.5, 0.0, 0.0]);
    locate(&mut w, 0, &[pos(1, 0), pos(0, 0), pos(0, 1)]);
    let before = w.state.clone();
    let mut d = Scripted::new(&[]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
fn paired_food_at_nest() -> World {
    let mut s = setup();
    s.food = vec![Resource {
        id: 0,
        pos: pos(2, 0),
    }];
    s.workers.push(pos(1, 0));
    s.parameters.lambda_publish = 10.0;
    s.parameters.lambda_fidelity = 10.0;
    let mut w = World::new(s, 12).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    action(&mut w, 0, Action::PickupFood, &[]).unwrap();
    action(&mut w, 1, Action::Wait(WaitReason::NoNeighbor), &[]).unwrap();
    w.state.tick += 1;
    locate(&mut w, 0, &[pos(1, 0)]);
    w
}
#[test]
fn late_failure_restores_deposit_publication_milestones_and_draws() {
    let mut w = paired_food_at_nest();
    let before = w.state.clone();
    let mut d = Scripted::new(&[0.0; 3]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn production_late_checked_counter_failure_restores_full_state_and_pcg_replay() {
    for kind in 0..3 {
        let mut w = match kind {
            0 => paired_face(),
            1 => {
                let mut w = paired_face();
                tick(&mut w, &[0.5, 0.0, 0.0]);
                locate(&mut w, 0, &[pos(1, 0), pos(0, 0), pos(0, 1)]);
                w
            }
            _ => paired_food_at_nest(),
        };
        w.setup.parameters.p_return = 0.0;
        let mut replay = w.clone();
        w.state.agents[1].compute.cells_inspected = u64::MAX;
        let before = w.state.clone();
        let mut original_rng = w.rng.clone();
        // The identical production draw adapter reaches an earlier physical
        // transaction before worker 1's private checked-counter fault.
        let mut scratch = before.clone();
        let mut candidate_rng = w.rng.clone();
        assert!(
            advance(&w.setup, &mut scratch, &mut PcgDraws(&mut candidate_rng)).unwrap_err()[0]
                .field
                .contains("agents[1]")
        );
        match kind {
            0 => assert_eq!(scratch.terrain.counts().excavated, 1),
            1 => assert_eq!(scratch.spoil.inventory().disposed, 1),
            _ => assert_eq!(scratch.food.inventory().delivered, 1),
        }
        assert!(w.step().unwrap_err()[0].field.contains("agents[1]"));
        assert_eq!(w.state, before);
        let mut actual = w.rng.clone();
        assert_eq!(actual.gen::<u64>(), original_rng.gen::<u64>());
        w.state.agents[1].compute.cells_inspected = replay.state.agents[1].compute.cells_inspected;
        w.step().unwrap();
        replay.step().unwrap();
        assert_eq!(w.state, replay.state);
        assert_eq!(w.rng.gen::<u64>(), replay.rng.gen::<u64>());
    }
}
#[test]
fn loaded_food_ignores_faces_and_spoil_ignores_visible_current_food() {
    let w = paired_food_at_nest();
    assert_eq!(choice(&w, 0, &[]).action, Action::DepositFood);
    let mut w = searching_at_face();
    tick(&mut w, &[0.5, 0.0]);
    locate(&mut w, 0, &[pos(3, 0)]);
    assert!(view(&w, 0)
        .cells
        .iter()
        .any(|c| c.pos == pos(3, 0) && c.food));
    assert_eq!(choice(&w, 0, &[0.0]).action, Action::Move(pos(2, 0)));
}
#[test]
fn informed_known_site_and_empty_arrival_draws_match_table() {
    let mut w = World::new(setup(), 1).unwrap();
    w.state.agents[0].site = Some(pos(1, 0));
    assert_eq!(choice(&w, 0, &[0.0]).action, Action::Move(pos(1, 0)));
    w.state.agents[0].site = None;
    w.state.agents[0].phase = FoodPhase::Returning;
    tick(&mut w, &[0.0; 3]);
    assert_eq!(w.state.agents[0].work.empty_returns, 1);
    assert_eq!(w.state.agents[0].work.uninformed_departures, 2);
}
#[test]
fn invalid_arrival_draw_preserves_carried_food_deposit_and_advice() {
    let mut w = paired_food_at_nest();
    let before = w.state.clone();
    let mut d = Scripted::new(&[0.0, f64::NAN, 0.0]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn informed_opened_face_becomes_site_route_and_clears_face_commitment() {
    let mut w = paired_face();
    locate(&mut w, 1, &[pos(2, 0)]);
    w.state.agents[1].phase = FoodPhase::Departing;
    w.state.agents[1].site = Some(pos(3, 0));
    w.state.agents[1].face = Some(pos(3, 0));
    tick(&mut w, &[0.5, 0.0, 0.0]);
    assert_eq!(w.state.agents[1].pos, pos(3, 0));
    assert_eq!(w.state.agents[1].face, None);
    assert_eq!(w.state.agents[1].site, Some(pos(3, 0)));
}
#[test]
fn full_outlet_approach_retains_spoil_and_informed_food_intent_without_step_draw() {
    let mut w = searching_at_face();
    w.state.agents[0].phase = FoodPhase::Departing;
    w.state.agents[0].site = Some(pos(3, 0));
    tick(&mut w, &[0.0]);
    locate(&mut w, 0, &[pos(1, 0), pos(0, 0)]);
    let mut o = view(&w, 0);
    o.cells
        .iter_mut()
        .find(|c| c.pos == w.setup.waste)
        .unwrap()
        .occupants = 2;
    let mut d = Scripted::new(&[]);
    let result = decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).unwrap();
    assert_eq!(result.action, Action::Wait(WaitReason::Congestion));
    assert_eq!(result.agent.cargo, Some(Cargo::Spoil(0)));
    assert_eq!(result.agent.site, Some(pos(3, 0)));
    assert_eq!(d.next, 0);
}
#[test]
fn full_nearest_nest_step_retains_food_and_cannot_dig() {
    let mut w = paired_food_at_nest();
    locate(&mut w, 0, &[pos(2, 0)]);
    let mut o = view(&w, 0);
    o.cells
        .iter_mut()
        .find(|c| c.pos == pos(1, 0))
        .unwrap()
        .occupants = 2;
    let mut d = Scripted::new(&[]);
    let result = decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d).unwrap();
    assert_eq!(result.action, Action::Wait(WaitReason::Congestion));
    assert_eq!(result.agent.cargo, Some(Cargo::Food(0)));
    assert_eq!(d.next, 0);
}
#[test]
fn exhausted_search_wanders_and_uninformed_departure_transitions_without_face_draw() {
    let mut s = setup();
    s.diggable.clear();
    s.food.clear();
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(0, 1), pos(0, 0), pos(1, 0), pos(2, 0)]);
    w.setup.parameters.p_search = 0.0;
    assert_eq!(
        choice(&w, 0, &[0.0]).action,
        Action::Wait(WaitReason::Transition)
    );
    w.state.agents[0].phase = FoodPhase::Searching;
    assert_eq!(choice(&w, 0, &[0.5, 0.0]).action, Action::Move(pos(1, 0)));
    let mut o = view(&w, 0);
    o.cells
        .iter_mut()
        .find(|c| c.pos == pos(1, 0))
        .unwrap()
        .occupants = 2;
    let mut d = Scripted::new(&[0.5]);
    assert_eq!(
        decide(&Policy::from(&w.setup), &w.state.agents[0], &o, &mut d)
            .unwrap()
            .action,
        Action::Wait(WaitReason::NoNeighbor)
    );
    assert_eq!(d.next, 1);
}
#[test]
fn impossible_informed_site_abandons_with_no_draw_and_unknown_site_explores() {
    let mut s = setup();
    s.diggable.clear();
    let mut w = World::new(s, 1).unwrap();
    w.state.agents[0].site = Some(pos(4, 2));
    let result = choice(&w, 0, &[0.0, 0.0]);
    assert_eq!(result.action, Action::Move(pos(0, 1)));
    locate(&mut w, 0, &[pos(0, 1), pos(0, 0), pos(1, 0), pos(2, 0)]);
    assert_eq!(
        choice(&w, 0, &[]).action,
        Action::Wait(WaitReason::Transition)
    );
}
#[test]
fn new_remote_face_selection_draws_once_then_step_retained_face_only_step() {
    let mut w = searching_at_face();
    locate(&mut w, 0, &[pos(1, 0), pos(0, 0)]);
    let result = choice(&w, 0, &[0.5, 0.0, 0.0]);
    assert_eq!(result.action, Action::Move(pos(1, 0)));
    assert_eq!(result.agent.face, Some(pos(3, 0)));
    w.state.agents[0].face = result.agent.face;
    assert_eq!(choice(&w, 0, &[0.5, 0.0]).action, Action::Move(pos(1, 0)));
    w.state.agents[0].phase = FoodPhase::Departing;
    w.state.agents[0].site = Some(pos(3, 0));
    assert_eq!(choice(&w, 0, &[0.0]).action, Action::Move(pos(1, 0)));
}
#[test]
fn equal_numeric_food_and_spoil_ids_cannot_cross_handle() {
    let mut s = setup();
    s.food[0].id = 0;
    let mut w = World::new(s, 1).unwrap();
    locate(&mut w, 0, &[pos(0, 1), pos(0, 0), pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    tick(&mut w, &[0.5, 0.0]);
    locate(&mut w, 0, &[pos(1, 0), pos(0, 0), pos(0, 1)]);
    tick(&mut w, &[]);
    locate(&mut w, 0, &[pos(0, 0), pos(1, 0), pos(2, 0), pos(3, 0)]);
    tick(&mut w, &[0.5]);
    assert_eq!(w.state.agents[0].cargo, Some(Cargo::Food(0)));
    locate(&mut w, 0, &[pos(2, 0), pos(1, 0), pos(0, 0), pos(0, 1)]);
    let before = w.state.clone();
    assert!(action(&mut w, 0, Action::DisposeSpoil, &[]).is_err());
    assert_eq!(w.state, before);
}
#[test]
fn budget_failure_happens_before_any_candidate_learning_or_draws() {
    let mut w = World::new(setup(), 1).unwrap();
    w.state.tick = 7200;
    let before = w.state.clone();
    let mut d = Scripted::new(&[0.0]);
    assert!(advance_scripted(&mut w, &mut d).is_err());
    assert_eq!(w.state, before);
    assert_eq!(d.next, 0);
}
#[test]
fn check_rejects_identity_topology_material_find_targets_and_accounting_corruption() {
    let w = searching_at_face();
    for kind in 0..9 {
        let mut bad = w.state.clone();
        match kind {
            0 => bad.agents[0].id = 4,
            1 => bad.agents[0].pos = pos(4, 2),
            2 => bad.agents[0].cargo = Some(Cargo::Food(u64::MAX)),
            3 => bad.agents[0].cargo = Some(Cargo::Spoil(0)),
            4 => bad.agents[0].find = Some(crate::foraging::FindRecord { site: 3, count: 0 }),
            5 => bad.agents[0].frontier = Some(pos(3, 0)),
            6 => bad.agents[0].face = Some(pos(2, 1)),
            7 => bad.agents[0].compute.cells_learned += 1,
            _ => bad.agents[0].work.digs = 1,
        }
        assert!(check(&w.setup, &bad).is_err(), "kind {kind}");
    }
}
#[test]
fn scratch_failures_reach_earlier_material_and_advice_mutations() {
    let mut digging = paired_face();
    let mut state = digging.state.clone();
    let mut d = Scripted::new(&[0.5, 0.0]);
    assert!(advance(&digging.setup, &mut state, &mut d).is_err());
    assert_eq!(state.terrain.counts().excavated, 1);
    assert_eq!(state.food.inventory().available, 1);
    assert_eq!(state.agents[0].compute.dig_confirmations, 1);
    assert!(state.milestones.first_access.is_some());
    assert_eq!(state.access.summary().unwrap().compute.calls, 2);
    tick(&mut digging, &[0.5, 0.0, 0.0]);
    locate(&mut digging, 0, &[pos(1, 0), pos(0, 0), pos(0, 1)]);
    let mut state = digging.state.clone();
    let mut d = Scripted::new(&[]);
    assert!(advance(&digging.setup, &mut state, &mut d).is_err());
    assert_eq!(state.spoil.inventory().disposed, 1);
    assert!(state.milestones.first_disposal.is_some());
    let w = paired_food_at_nest();
    let mut state = w.state.clone();
    let mut d = Scripted::new(&[0.0; 3]);
    assert!(advance(&w.setup, &mut state, &mut d).is_err());
    assert_eq!(state.food.inventory().delivered, 1);
    assert_eq!(
        state
            .server
            .views(&w.setup.parameters, state.tick)
            .unwrap()
            .len(),
        1
    );
    assert!(state.milestones.first_delivery_tick.is_some());
}
#[test]
fn density_freezes_only_available_current_and_cardinal_food() {
    let mut s = setup();
    s.open.extend([pos(2, 1), pos(3, 1)]);
    s.food = vec![
        Resource {
            id: 0,
            pos: pos(2, 0),
        },
        Resource {
            id: 1,
            pos: pos(2, 1),
        },
        Resource {
            id: 2,
            pos: pos(3, 0),
        },
        Resource {
            id: 3,
            pos: pos(3, 1),
        },
    ];
    let mut w = World::new(s, 12).unwrap();
    locate(&mut w, 0, &[pos(1, 0), pos(2, 0)]);
    w.state.agents[0].phase = FoodPhase::Searching;
    tick(&mut w, &[0.5]);
    assert_eq!(w.state.agents[0].find.unwrap().count, 2);
    assert_eq!(w.state.food.inventory().hidden, 1);
    locate(&mut w, 0, &[pos(1, 0)]);
    assert_eq!(w.state.agents[0].find.unwrap().count, 2);
}
#[test]
fn malformed_dimensions_origin_and_duplicate_cells_fail_without_draws() {
    let w = World::new(setup(), 1).unwrap();
    for kind in 0..3 {
        let mut p = Policy::from(&w.setup);
        let mut o = view(&w, 0);
        match kind {
            0 => p.width = 4,
            1 => o.origin = pos(1, 0),
            _ => o.cells.push(o.cells[0].clone()),
        }
        let mut d = Scripted::new(&[0.0]);
        assert!(decide(&p, &w.state.agents[0], &o, &mut d).is_err());
        assert_eq!(d.next, 0);
    }
}
#[test]
fn physical_moves_reject_solid_nonlocal_and_full_cells_atomically() {
    let mut s = setup();
    s.workers = vec![pos(0, 0), pos(0, 0), pos(1, 0)];
    let mut w = World::new(s, 1).unwrap();
    for target in [pos(3, 0), pos(0, 2), pos(1, 1)] {
        let before = w.state.clone();
        assert!(action(&mut w, 0, Action::Move(target), &[]).is_err());
        assert_eq!(w.state, before);
    }
    action(&mut w, 0, Action::Move(pos(1, 0)), &[]).unwrap();
    let before = w.state.clone();
    assert!(action(&mut w, 1, Action::Move(pos(1, 0)), &[]).is_err());
    assert_eq!(w.state, before);
}
#[test]
fn known_site_retains_f3_frontier_cadence_and_clears_only_a_resolved_face() {
    // Frontier and face commitments are mutually exclusive. Exercise both
    // valid alternatives after another worker opens the informed site.
    for remembered_face in [false, true] {
        let mut w = paired_face();
        locate(&mut w, 1, &[pos(2, 0)]);
        let a = &mut w.state.agents[1];
        a.phase = FoodPhase::Departing;
        a.site = Some(pos(3, 0));
        if remembered_face {
            a.face = Some(pos(3, 0));
        } else {
            a.frontier = Some(pos(0, 0));
        }
        check(&w.setup, &w.state).unwrap();
        action(&mut w, 0, Action::Dig(pos(3, 0)), &[]).unwrap();
        let d = choice(&w, 1, &[0.0]); // exactly the single route-step draw in F3
        assert_eq!(d.action, Action::Move(pos(3, 0)));
        assert_eq!(d.agent.face, None);
        assert_eq!(
            d.agent.frontier,
            if remembered_face {
                None
            } else {
                Some(pos(0, 0))
            }
        );
        assert_eq!(d.agent.compute.observed_revisions, 1);
    }
}
