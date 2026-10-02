//! State-machine fixtures with hand-derived food balances and real turns.

use super::state::{Delivery, FounderTraits, SpatialState};
use crate::config::{CachingRule, DigBelow, MoveMode};
use crate::geometry::Pos;
use crate::testkit::*;
use crate::world::World;

fn fixture() -> (World, crate::agent::AgentId) {
    let mut w = blank_world(15, 15);
    w.config.spatial_hoarding.enabled = true;
    w.config.spatial_hoarding.guard = false;
    w.config.caching.rule = CachingRule::Even;
    w.config.caching.capacity = 50;
    w.config.movement.mode = MoveMode::Walk;
    let id = spawn(&mut w, 7, 7);
    let a = w.agent_mut(id).unwrap();
    a.metabolism[0] = 1;
    a.holdings[0] = 30.0;
    a.spatial = Some(SpatialState::new(
        a.pos,
        FounderTraits {
            larder: 1.0,
            defense: 0.5,
            cheater: false,
            watches: false,
        },
    ));
    (w, id)
}

fn turn(w: &mut World, id: crate::agent::AgentId) {
    w.events = crate::world::TickEvents::default();
    crate::rules::agent_turn(w, id);
}

fn state(w: &World, id: crate::agent::AgentId) -> &SpatialState {
    w.agent(id).unwrap().spatial.as_ref().unwrap()
}

#[test]
fn spatial_hoarding_away_allocation_keeps_food_in_holdings() {
    let (mut w, id) = fixture();
    w.move_agent(id, Pos::new(3, 7));
    turn(&mut w, id);
    assert_eq!(state(&w, id).delivery, Some(Delivery { amount: 10.0 }));
    assert_eq!(w.agent(id).unwrap().holdings[0], 29.0);
    assert_eq!(w.events.buried, 0.0);
}

#[test]
fn spatial_hoarding_delivery_walks_successive_steps_and_completes_once() {
    let (mut w, id) = fixture();
    w.move_agent(id, Pos::new(3, 7));
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().delivery = Some(Delivery { amount: 8.0 });
    for x in [4, 5, 6] {
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(x, 7));
    }
    assert_eq!(state(&w, id).larder, 8.0);
    assert_eq!(state(&w, id).delivery, None);
    assert!(w.agent(id).unwrap().caches.is_empty());
    assert_eq!(w.agent(id).unwrap().holdings[0], 19.0);
}

#[test]
fn spatial_hoarding_metabolism_cancels_depleted_delivery() {
    let (mut w, id) = fixture();
    w.move_agent(id, Pos::new(1, 7));
    let a = w.agent_mut(id).unwrap();
    a.holdings[0] = 11.0;
    a.spatial.as_mut().unwrap().delivery = Some(Delivery { amount: 1.0 });
    turn(&mut w, id);
    assert_eq!(state(&w, id).delivery, None);
    assert_eq!(w.agent(id).unwrap().holdings[0], 10.0);
}

#[test]
fn spatial_hoarding_later_intake_does_not_enlarge_delivery_or_allocate_again() {
    let (mut w, id) = fixture();
    w.move_agent(id, Pos::new(3, 7));
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().delivery = Some(Delivery { amount: 3.0 });
    set_sugar(&mut w, 4, 7, 30.0);
    turn(&mut w, id);
    assert_eq!(state(&w, id).delivery, Some(Delivery { amount: 3.0 }));
    assert_eq!(w.agent(id).unwrap().holdings[0], 49.0);
    assert_eq!(w.site(Pos::new(4, 7)).resource[0], 10.0);
    assert_eq!(w.events.buried, 0.0);
}

#[test]
fn spatial_hoarding_allocation_at_contact_deposits_immediately_with_one_cost() {
    let (mut w, id) = fixture();
    w.config.caching.bury_cost = 0.25;
    turn(&mut w, id);
    assert_eq!(state(&w, id).larder, 10.0);
    assert_eq!(state(&w, id).delivery, None);
    assert_eq!(w.agent(id).unwrap().holdings[0], 16.5);
    assert_eq!((w.events.buried, w.events.bury_cost), (10.0, 2.5));
}

#[test]
fn spatial_hoarding_contact_delivery_caps_surplus_and_cost_affordability() {
    let (mut w, id) = fixture();
    w.config.caching.bury_cost = 4.0;
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().delivery = Some(Delivery { amount: 40.0 });
    turn(&mut w, id);
    assert_eq!(w.events.buried, 6.0);
    assert_eq!(w.events.spatial_stores.unwrap().larder.buried, 6.0);
    assert_eq!(w.events.bury_cost, 24.0);
    assert!(
        w.agent(id).is_none(),
        "ordinary mortality after paying and metabolizing"
    );
}

#[test]
fn spatial_hoarding_zero_surplus_allocates_nothing() {
    let (mut w, id) = fixture();
    w.agent_mut(id).unwrap().holdings[0] = 10.0;
    turn(&mut w, id);
    assert_eq!((state(&w, id).delivery, state(&w, id).larder), (None, 0.0));
}

#[test]
fn spatial_hoarding_unavailable_return_endpoint_never_transfers() {
    let (mut w, id) = fixture();
    w.move_agent(id, Pos::new(3, 7));
    for (x, y) in [(7, 7), (6, 7), (8, 7), (7, 6), (7, 8)] {
        spawn(&mut w, x, y);
    }
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().delivery = Some(Delivery { amount: 8.0 });
    turn(&mut w, id);
    assert_eq!(w.agent(id).unwrap().pos, Pos::new(3, 7));
    assert_eq!(state(&w, id).larder, 0.0);
    assert_eq!(state(&w, id).delivery, Some(Delivery { amount: 8.0 }));
    assert_eq!(w.events.buried, 0.0);
}

#[test]
fn spatial_hoarding_guard_displaces_harvest_and_keeps_metabolism() {
    let (mut w, id) = fixture();
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().guarding = true;
    set_sugar(&mut w, 7, 7, 8.0);
    set_sugar(&mut w, 8, 7, 12.0);
    turn(&mut w, id);
    assert_eq!(w.agent(id).unwrap().pos, Pos::new(7, 7));
    assert_eq!(w.agent(id).unwrap().holdings[0], 29.0);
    assert_eq!(w.site(Pos::new(7, 7)).resource[0], 8.0);
    assert_eq!(state(&w, id).larder, 0.0);
}

#[test]
fn spatial_hoarding_hungry_guard_recovers_larder_without_scatter_or_harvest() {
    let (mut w, id) = fixture();
    let a = w.agent_mut(id).unwrap();
    a.holdings[0] = 2.0;
    let s = a.spatial.as_mut().unwrap();
    s.larder = 7.0;
    s.guarding = true;
    set_sugar(&mut w, 7, 7, 20.0);
    turn(&mut w, id);
    assert_eq!(w.events.dug, 7.0);
    assert_eq!(w.agent(id).unwrap().holdings[0], 8.0);
    assert_eq!(w.site(Pos::new(7, 7)).resource[0], 20.0);
}

#[test]
fn spatial_hoarding_ordinary_hungry_owner_recovers_larder_before_harvest() {
    let (mut w, id) = fixture();
    w.agent_mut(id).unwrap().holdings[0] = 6.0;
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().larder = 2.0;
    w.config.caching.dig_below = DigBelow::Reserve;
    set_sugar(&mut w, 7, 7, 5.0);
    turn(&mut w, id);
    assert_eq!(w.events.dug, 2.0);
    assert_eq!(w.agent(id).unwrap().holdings[0], 7.0);
    assert_eq!(w.site(Pos::new(7, 7)).resource[0], 5.0);
}

#[test]
fn spatial_hoarding_guards_are_prepared_by_step_and_reset_when_off() {
    let (mut w, id) = fixture();
    w.config.spatial_hoarding.guard = true;
    w.config.spatial_hoarding.defense_slope = 100.0;
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().larder = 1000.0;
    set_sugar(&mut w, 7, 7, 5.0);
    w.step();
    assert!(state(&w, id).guarding);
    assert_eq!(w.agent(id).unwrap().holdings[0], 29.0);
    w.config.spatial_hoarding.guard = false;
    w.step();
    assert!(!state(&w, id).guarding);
}

#[test]
fn spatial_hoarding_depleted_delivery_is_cancelled() {
    assert_eq!(
        super::delivery::clamped_delivery(Some(Delivery { amount: 8.0 }), 0.0),
        None
    );
}

#[test]
fn spatial_hoarding_delivery_cannot_expand_after_new_harvest() {
    assert_eq!(
        super::delivery::clamped_delivery(Some(Delivery { amount: 3.0 }), 9.0),
        Some(Delivery { amount: 3.0 })
    );
}

#[test]
fn spatial_hoarding_guard_intentions_use_id_order_and_are_frozen() {
    use rand::Rng;
    let (mut w, first) = fixture();
    let second = spawn(&mut w, 2, 2);
    w.config.spatial_hoarding.guard = true;
    // R=10, D=0.5, C=50 gives T=35; stock=17.5 gives p=0.5.
    for id in [first, second] {
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.spatial = Some(SpatialState::new(
            a.pos,
            FounderTraits {
                larder: 1.0,
                defense: 0.5,
                cheater: false,
                watches: false,
            },
        ));
        a.spatial.as_mut().unwrap().larder = 17.5;
    }
    let mut expected_rng = w.rng.clone();
    let expected = [
        expected_rng.gen::<f64>() < 0.5,
        expected_rng.gen::<f64>() < 0.5,
    ];
    super::guard::prepare_guards(&mut w);
    assert_eq!(
        [state(&w, first).guarding, state(&w, second).guarding],
        expected
    );
    assert_eq!(
        w.events.spatial_stores.unwrap().guard.intended,
        expected.into_iter().filter(|x| *x).count() as u32
    );
    // Subsequent sugar/holdings change does not recalculate intentions.
    w.agent_mut(first).unwrap().spatial.as_mut().unwrap().larder = 0.0;
    assert_eq!(state(&w, first).guarding, expected[0]);
    assert_eq!(w.rng.gen::<u64>(), expected_rng.gen::<u64>());
}

#[test]
fn spatial_hoarding_guards_count_paid_turn_and_recovery_separately() {
    let (mut w, id) = fixture();
    w.agent_mut(id).unwrap().holdings[0] = 2.0;
    let s = w.agent_mut(id).unwrap().spatial.as_mut().unwrap();
    s.larder = 7.0;
    s.guarding = true;
    turn(&mut w, id);
    let e = w.events.spatial_stores.unwrap().guard;
    assert_eq!((e.intended, e.executed, e.recovered), (0, 1, 7.0));
}

#[test]
fn spatial_hoarding_delivery_events_count_batches_and_actual_return_actions() {
    let (mut w, id) = fixture();
    w.config.caching.bury_cost = 0.25;
    turn(&mut w, id);
    let immediate = w.events.spatial_stores.unwrap().delivery;
    assert_eq!(
        (
            immediate.starts,
            immediate.completions,
            immediate.return_turns
        ),
        (1, 1, 0)
    );
    assert_eq!((immediate.delivered, immediate.bury_cost), (10.0, 2.5));

    w.move_agent(id, Pos::new(3, 7));
    w.agent_mut(id).unwrap().spatial.as_mut().unwrap().delivery = Some(Delivery { amount: 3.0 });
    turn(&mut w, id);
    let returning = w.events.spatial_stores.unwrap().delivery;
    assert_eq!(
        (
            returning.starts,
            returning.completions,
            returning.return_turns
        ),
        (0, 0, 1)
    );

    w.agent_mut(id).unwrap().holdings[0] = 11.0;
    turn(&mut w, id);
    let cancelled = w.events.spatial_stores.unwrap().delivery;
    assert_eq!((cancelled.cancellations, cancelled.return_turns), (1, 1));
}

#[test]
fn spatial_hoarding_guard_off_occupied_home_allows_theft_and_dead_owner_has_no_guard() {
    let (mut w, owner) = fixture();
    let taker = spawn(&mut w, 7, 6);
    let t = w.agent_mut(taker).unwrap();
    t.holdings[0] = 0.0;
    t.spatial = Some(SpatialState::new(
        t.pos,
        FounderTraits {
            larder: 0.0,
            defense: 0.0,
            cheater: true,
            watches: false,
        },
    ));
    w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 8.0;
    assert_eq!(super::stores::take(&mut w, owner, taker, 3.0), 3.0);
    w.agent_mut(owner)
        .unwrap()
        .spatial
        .as_mut()
        .unwrap()
        .guarding = true;
    assert_eq!(super::stores::take(&mut w, owner, taker, 3.0), 0.0);
    w.agent_mut(owner).unwrap().holdings[0] = 0.5;
    w.agent_mut(owner).unwrap().metabolism[0] = 10;
    turn(&mut w, owner);
    assert!(w.agent(owner).is_none());
    assert_eq!(super::stores::take(&mut w, owner, taker, 3.0), 0.0);
}

#[test]
fn spatial_hoarding_l_zero_guard_off_matches_scatter_physics_and_rng() {
    use rand::Rng;
    let mut scatter = blank_world(11, 11);
    scatter.config.caching.rule = CachingRule::Even;
    scatter.config.caching.capacity = 50;
    scatter.config.movement.mode = MoveMode::Walk;
    for y in 0..11 {
        for x in 0..11 {
            set_sugar(&mut scatter, x, y, f64::from((x + 2 * y) % 5 + 1));
        }
    }
    for (x, y) in [(1, 1), (5, 5), (9, 9)] {
        let id = spawn(&mut scatter, x, y);
        scatter.agent_mut(id).unwrap().metabolism[0] = 1;
        scatter.agent_mut(id).unwrap().holdings[0] = 30.0;
    }
    let mut spatial = scatter.clone();
    spatial.config.spatial_hoarding.enabled = true;
    spatial.config.spatial_hoarding.guard = false;
    for id in spatial.agent_ids() {
        let a = spatial.agent_mut(id).unwrap();
        a.spatial = Some(SpatialState::new(
            a.pos,
            FounderTraits {
                larder: 0.0,
                defense: 0.5,
                cheater: false,
                watches: false,
            },
        ));
    }
    for tick in 0..15 {
        scatter.step();
        spatial.step();
        assert_eq!(
            scatter.fingerprint(),
            spatial.fingerprint(),
            "physical tick {tick}"
        );
        let mut a = scatter.rng.clone();
        let mut b = spatial.rng.clone();
        assert_eq!(a.gen::<u64>(), b.gen::<u64>(), "rng tick {tick}");
    }
}
