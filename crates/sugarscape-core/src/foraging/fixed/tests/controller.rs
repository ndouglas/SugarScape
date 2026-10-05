use super::super::{
    controller::advance, draws::DrawSource, world::State, Phase, Pos, Resource, World,
};
use super::{setup, Scripted};
use crate::config::FieldError;
fn searcher(agents: u32) -> World {
    let mut s = setup();
    s.agents = agents;
    s.resources.push(Resource {
        id: 17,
        pos: Pos { x: 3, y: 2 },
    });
    let mut w = World::new(s, 0).unwrap();
    for a in &mut w.state.agents {
        a.phase = Phase::Searching;
        a.pos = Pos { x: 1, y: 2 };
        a.heading = 0.0;
        a.delay = 0;
    }
    w
}
fn tick(w: &mut World, values: &[f64]) {
    advance(&w.setup, &mut w.state, &mut Scripted::new(values)).unwrap();
    w.state.ledger.check(&w.state.agents).unwrap();
}
// Break caught: returning during the survey wait would score delivery early.
#[test]
fn survey_waits_do_not_deliver_a_token_early() {
    let mut w = searcher(1);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    assert_eq!(w.state.agents[0].delay, 9);
    assert_eq!(w.state.agents[0].find.unwrap().count, 1);
    for _ in 0..9 {
        tick(&mut w, &[]);
    }
    assert_eq!(w.state.ledger.inventory().assigned, 1);
    tick(&mut w, &[0.0; 5]);
    assert_eq!(w.state.ledger.inventory().delivered, 1);
    assert_eq!(w.state.first_delivery_tick, Some(10));
    assert_eq!(w.state.all_delivered_tick, Some(10));
    assert_eq!(w.state.agents[0].cargo, None);
    assert_eq!(w.state.agents[0].find, None);
    assert_eq!(w.state.agents[0].work.deliveries, 1);
    assert_eq!(w.state.agents[0].work.fidelity_departures, 1);
}
// Break caught: movement/detection before give-up would claim the resource.
#[test]
fn give_up_ends_opportunity_before_pickup() {
    let mut w = searcher(1);
    w.setup.parameters.p_return = 1.0;
    tick(&mut w, &[0.5]);
    assert_eq!(w.state.agents[0].phase, Phase::Returning);
    assert_eq!(w.state.ledger.inventory().available, 1);
    assert_eq!(w.state.agents[0].pos, Pos { x: 1, y: 2 });
}
// Break caught: detecting on departure or skipping switch draw on equality.
#[test]
fn departure_turns_without_detecting_and_draws_switch_on_target_equality() {
    let mut w = searcher(1);
    w.state.agents[0].phase = Phase::Departing;
    w.state.agents[0].target = w.state.agents[0].pos;
    let mut d = Scripted::new(&[0.5, 0.0, 0.0]);
    advance(&w.setup, &mut w.state, &mut d).unwrap();
    assert_eq!(d.next, 3);
    assert_eq!(w.state.agents[0].phase, Phase::Searching);
    assert_eq!(w.state.ledger.inventory().available, 1);
    assert_eq!(w.state.agents[0].delay, 1);
    assert_eq!(w.state.agents[0].work.search_switches, 1);
}
// Break caught: double ownership or nonstable processing would pick the later agent.
#[test]
fn ascending_agents_contend_for_one_token() {
    let mut w = searcher(2);
    w.state.agents.reverse();
    tick(&mut w, &[0.5, 0.0, 0.0, 0.5, 0.0, 0.0]);
    assert_eq!(
        w.state.agents.iter().find(|a| a.id == 0).unwrap().cargo,
        Some(17)
    );
    assert_eq!(
        w.state.agents.iter().find(|a| a.id == 1).unwrap().cargo,
        None
    );
    assert_eq!(w.state.ledger.inventory().assigned, 1);
}
fn transactional(
    setup: &super::super::Setup,
    state: &mut State,
    draws: &mut (impl DrawSource + Clone),
) -> Result<(), Vec<FieldError>> {
    let mut next = state.clone();
    let mut next_draws = draws.clone();
    advance(setup, &mut next, &mut next_draws)?;
    *state = next;
    *draws = next_draws;
    Ok(())
}
// Break caught: committing an earlier pickup despite a later draw failure.
#[test]
fn later_failure_rolls_back_earlier_pickup_and_draw_cursor() {
    for bad in [None, Some(f64::NAN)] {
        let mut w = searcher(2);
        let before = w.state.clone();
        let mut values = vec![0.5, 0.0, 0.0];
        if let Some(bad) = bad {
            values.push(bad);
        }
        let mut d = Scripted::new(&values);
        assert!(transactional(&w.setup, &mut w.state, &mut d).is_err());
        assert_eq!(w.state, before);
        assert_eq!(d.next, 0);
    }
}
// Break caught: failing limits after committing work or advancing RNG.
#[test]
fn production_limit_failure_preserves_world_and_rng() {
    use rand::Rng;
    let mut w = World::new(setup(), 77).unwrap();
    w.state.tick = 7200;
    let mut before = w.clone();
    assert!(w.step().is_err());
    assert_eq!(w.state, before.state);
    assert_eq!(w.snapshot().unwrap(), before.snapshot().unwrap());
    assert_eq!(w.rng.gen::<u64>(), before.rng.gen::<u64>());
}
// Break caught: aging during waits/travel or omitting the initial search turn.
#[test]
fn informed_age_counts_only_turns() {
    let mut w = World::new(setup(), 0).unwrap();
    let a = &mut w.state.agents[0];
    a.informed = true;
    a.target = a.pos;
    a.heading = 0.0;
    tick(&mut w, &[0.0, 0.0]);
    assert_eq!(w.state.agents[0].informed_turns, 1);
    tick(&mut w, &[]);
    assert_eq!(w.state.agents[0].informed_turns, 1);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    assert_eq!(w.state.agents[0].informed_turns, 2);
    w.state.agents[0].delay = 0;
    w.state.agents[0].phase = Phase::Departing;
    w.state.agents[0].target = Pos { x: 4, y: 2 };
    tick(&mut w, &[0.5]);
    assert_eq!(w.state.agents[0].informed_turns, 2);
}
// Break caught: attempting a directed move when an edge nest equals target.
#[test]
fn edge_nest_target_equality_enters_search() {
    let mut s = setup();
    s.nest = Pos { x: 0, y: 0 };
    s.parameters.p_search = 0.0;
    let mut w = World::new(s, 0).unwrap();
    w.state.agents[0].target = w.setup.nest;
    tick(&mut w, &[0.5, 0.0, 0.0]);
    assert_eq!(w.state.agents[0].phase, Phase::Searching);
    assert_eq!(w.state.agents[0].pos, w.setup.nest);
    assert_eq!(w.state.agents[0].work.directed_moves, 0);
}
// Break caught: empty arrivals publishing stale memory or omitting three uniforms.
#[test]
fn empty_return_clears_find_and_draws_arrival_and_edge() {
    let mut w = World::new(setup(), 0).unwrap();
    w.state.agents[0].phase = Phase::Returning;
    w.state.agents[0].find = Some(crate::foraging::FindRecord { site: 6, count: 1 });
    w.state.agents[0].informed_turns = 5;
    let mut d = Scripted::new(&[0.0; 5]);
    advance(&w.setup, &mut w.state, &mut d).unwrap();
    assert_eq!(d.next, 5);
    assert_eq!(w.state.agents[0].find, None);
    assert_eq!(w.state.agents[0].work.empty_returns, 1);
    assert_eq!(w.state.agents[0].work.publications, 0);
    assert_eq!(w.state.agents[0].work.uninformed_departures, 1);
    assert_eq!(w.state.agents[0].informed_turns, 0);
}

// Break caught: missing snapshot conversion or pruning/sampling as a read side effect.
#[test]
fn snapshot_decodes_find_and_is_observational() {
    use rand::Rng;
    let mut w = searcher(1);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    let mut before = w.clone();
    let snapshot = w.snapshot().unwrap();
    assert_eq!(
        snapshot.agents[0].find,
        Some(super::super::FindView {
            site: Pos { x: 3, y: 2 },
            count: 1
        })
    );
    assert_eq!(snapshot.work.pickups, 1);
    assert_eq!(snapshot.completed_ticks, 1);
    assert_eq!(w.state, before.state);
    assert_eq!(w.snapshot().unwrap(), before.snapshot().unwrap());
    assert_eq!(w.rng.gen::<u64>(), before.rng.gen::<u64>());
    assert_eq!(snapshot, w.snapshot().unwrap());
}
// Break caught: interpreting no resources as a completed delivery event.
#[test]
fn empty_initial_snapshot_has_no_delivery_event() {
    let w = World::new(setup(), 0).unwrap();
    assert_eq!(w.snapshot().unwrap().all_delivered_tick, None);
}
// Break caught: a snapshot accepting an invalid encoded find site.
#[test]
fn snapshot_reports_invalid_find_geometry() {
    let mut w = World::new(setup(), 0).unwrap();
    w.state.agents[0].find = Some(crate::foraging::FindRecord { site: 25, count: 1 });
    let errors = w.snapshot().unwrap_err();
    assert!(errors.iter().any(|e| e.field.contains("site")));
}
// Break caught: processing publication after departure/later agents, or refreshing frozen density.
#[test]
fn publication_is_visible_to_publisher_and_later_empty_arrival() {
    let mut w = searcher(2);
    tick(&mut w, &[0.5, 0.0, 0.0, 0.5, 0.0, 0.0]);
    w.setup.parameters.lambda_fidelity = 256.0;
    w.state.agents[0].delay = 0;
    w.state.agents[1].delay = 0;
    w.state.agents[1].phase = Phase::Returning;
    w.state.agents[1].pos = w.setup.nest;
    tick(&mut w, &[0.0, 0.5, 0.0, 0.0, 0.0, 0.0]);
    assert_eq!(w.state.agents[0].target, Pos { x: 3, y: 2 });
    assert_eq!(w.state.agents[0].work.recruited_departures, 1);
    assert_eq!(w.state.agents[1].target, Pos { x: 3, y: 2 });
    assert_eq!(w.state.agents[1].work.recruited_departures, 1);
    assert_eq!(w.state.agents[0].work.publications, 1);
    assert_eq!(w.state.agents[0].find, None);
    assert_eq!(w.state.agents[1].work.empty_returns, 1);
}
// Break caught: ignoring the weighted return draw or treating partial approach as arrival.
#[test]
fn return_moves_weighted_neighbor_before_processing_arrival() {
    let mut w = World::new(setup(), 0).unwrap();
    w.state.agents[0].phase = Phase::Returning;
    w.state.agents[0].pos = Pos { x: 0, y: 0 };
    tick(&mut w, &[0.0]);
    assert_eq!(w.state.agents[0].pos, Pos { x: 0, y: 1 });
    assert_eq!(w.state.agents[0].work.directed_moves, 1);
    assert_eq!(w.state.agents[0].work.empty_returns, 0);
    tick(&mut w, &[0.999]);
    assert_eq!(w.state.agents[0].pos, Pos { x: 1, y: 2 });
    tick(&mut w, &[0.0; 5]);
    assert_eq!(w.state.agents[0].pos, Pos { x: 2, y: 2 });
    assert_eq!(w.state.agents[0].work.empty_returns, 1);
}
// Break caught: sensing the old forward heading instead of the completed turn.
#[test]
fn detection_uses_heading_after_turn() {
    let mut s = setup();
    s.parameters.omega = std::f64::consts::FRAC_PI_2;
    s.resources.push(Resource {
        id: 17,
        pos: Pos { x: 2, y: 3 },
    });
    let mut w = World::new(s, 0).unwrap();
    let a = &mut w.state.agents[0];
    a.phase = Phase::Searching;
    a.pos = Pos { x: 1, y: 2 };
    a.heading = 0.0;
    tick(&mut w, &[0.5, 1.0 - (-0.5_f64).exp(), 0.0]);
    assert_eq!(w.state.agents[0].cargo, Some(17));
    assert_eq!(w.state.agents[0].pos, Pos { x: 2, y: 2 });
}
// Break caught: refreshing density during waits, or publishing a successful find twice.
#[test]
fn frozen_density_survives_other_pickups_and_empty_return() {
    let mut s = setup();
    s.agents = 2;
    s.resources = vec![
        Resource {
            id: 17,
            pos: Pos { x: 3, y: 2 },
        },
        Resource {
            id: 18,
            pos: Pos { x: 3, y: 3 },
        },
    ];
    let mut w = World::new(s, 0).unwrap();
    for a in &mut w.state.agents {
        a.phase = Phase::Searching;
        a.pos = Pos { x: 1, y: 2 + a.id };
        a.heading = 0.0;
    }
    tick(&mut w, &[0.5, 0.0, 0.0, 0.5, 0.0, 0.0]);
    for _ in 0..9 {
        tick(&mut w, &[]);
    }
    assert_eq!(w.state.agents[0].find.unwrap().count, 2);
    assert_eq!(w.state.agents[1].find.unwrap().count, 1);
    tick(&mut w, &[0.0; 6]);
    assert_eq!(w.state.all_delivered_tick, Some(10));
    for a in &mut w.state.agents {
        a.phase = Phase::Returning;
    }
    tick(&mut w, &[0.0; 6]);
    assert_eq!(w.state.agents[0].work.publications, 1);
    assert_eq!(w.state.agents[0].work.deliveries, 1);
    assert_eq!(w.state.agents[0].work.empty_returns, 1);
}
// Break caught: committing an earlier deposit/publication when the later agent fails.
#[test]
fn later_failure_rolls_back_publication_and_deposit() {
    let mut w = searcher(2);
    tick(&mut w, &[0.5, 0.0, 0.0, 0.5, 0.0, 0.0]);
    w.state.agents[0].delay = 0;
    w.state.agents[1].delay = 0;
    let before = w.state.clone();
    let mut draws = Scripted::new(&[0.0; 3]);
    assert!(transactional(&w.setup, &mut w.state, &mut draws).is_err());
    assert_eq!(w.state, before);
    assert_eq!(draws.next, 0);
}
// Break caught: allowing the next tick beyond the million-opportunity boundary.
#[test]
fn opportunity_budget_rejects_next_complete_tick() {
    let mut s = setup();
    s.agents = 256;
    let mut w = World::new(s, 0).unwrap();
    w.state.tick = 3906;
    let before = w.state.clone();
    assert!(w.step().is_err());
    assert_eq!(w.state, before);
}

// Break caught: drawing an extra variate on successful informed nest departure.
#[test]
fn successful_arrival_consumes_exactly_three_decision_draws() {
    let mut w = searcher(1);
    tick(&mut w, &[0.5, 0.0, 0.0]);
    w.state.agents[0].delay = 0;
    w.state.agents[0].pos = w.setup.nest;
    let mut draws = Scripted::new(&[0.0, 0.0, 0.0]);
    advance(&w.setup, &mut w.state, &mut draws).unwrap();
    assert_eq!(draws.next, 3);
    assert_eq!(w.state.agents[0].work.deliveries, 1);
}
// Break caught: rejecting the last legal tick at either cumulative limit.
#[test]
fn last_legal_tick_is_accepted_at_both_limits() {
    for (agents, before, after) in [(1, 7199, 7200), (250, 3999, 4000)] {
        let mut s = setup();
        s.agents = agents;
        let mut w = World::new(s, 0).unwrap();
        w.state.tick = before;
        for a in &mut w.state.agents {
            a.work.opportunities = u64::from(before);
            a.delay = 1;
        }
        w.step().unwrap();
        assert_eq!(w.summary().unwrap().completed_ticks, after);
        assert_eq!(
            w.summary().unwrap().work.opportunities,
            u64::from(agents) * u64::from(after)
        );
        assert!(w.step().is_err());
    }
}
