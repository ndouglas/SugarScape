use super::super::{access::*, food::*, server::Server, spoil::*, state::*};
use super::*;
use crate::foraging::{Departure, FindRecord};

pub(in super::super) fn agent(id: u32, cargo: Option<Cargo>) -> Agent {
    Agent {
        id,
        pos: pos(3, 0),
        phase: FoodPhase::Searching,
        map: Knowledge::new(5, 3).unwrap(),
        cargo,
        find: None,
        site: None,
        frontier: None,
        face: None,
        work: WorkCounts::default(),
        compute: ComputeCounts::default(),
    }
}
fn context(opportunity: u64, excavated: u32) -> EventContext {
    EventContext {
        tick: (opportunity - 1) as u32,
        opportunity,
        worker: 0,
        excavated,
        spoil_disposed: excavated.saturating_sub(1),
        food_delivered: 0,
    }
}
fn find(s: &Setup) -> Option<FindRecord> {
    Some(FindRecord {
        site: s.site(pos(3, 0)).unwrap(),
        count: 1,
    })
}

#[test]
fn exposure_is_not_collection_or_delivery() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    let mut sp = SpoilLedger::new(t.capacity()).unwrap();
    assert_eq!(f.claim(pos(3, 0), 0).unwrap(), None);
    t.dig(pos(3, 0)).unwrap();
    let sid = sp.spawn(pos(3, 0), 0, 0).unwrap();
    assert_eq!(sid, 0);
    assert_eq!(f.expose(pos(3, 0)).unwrap(), Some(u64::MAX));
    assert_eq!(
        f.inventory(),
        FoodInventory {
            initial: 1,
            hidden: 0,
            available: 1,
            carried: 0,
            delivered: 0
        }
    );
    f.check(&t, &[agent(0, Some(Cargo::Spoil(sid)))]).unwrap();
    sp.check(&t, &[agent(0, Some(Cargo::Spoil(sid)))]).unwrap();
    let before = f.clone();
    assert!(f.expose(pos(3, 0)).is_err());
    assert_eq!(f, before);
    assert_eq!(f.expose(pos(4, 0)).unwrap(), None);
    assert_eq!(f.claim(pos(3, 0), 0).unwrap(), Some(u64::MAX));
    f.deposit(u64::MAX, 0).unwrap();
    assert_eq!(f.inventory().delivered, 1);
}

#[test]
fn equal_numeric_food_and_spoil_ids_are_separate_namespaces() {
    let mut s = setup();
    s.food[0].id = 0;
    s.food[0].pos = pos(2, 0);
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    let mut sp = SpoilLedger::new(t.capacity()).unwrap();
    t.dig(pos(3, 0)).unwrap();
    sp.spawn(pos(3, 0), 1, 0).unwrap();
    f.claim(pos(2, 0), 0).unwrap();
    let agents = [
        agent(0, Some(Cargo::Food(0))),
        agent(1, Some(Cargo::Spoil(0))),
    ];
    f.check(&t, &agents).unwrap();
    sp.check(&t, &agents).unwrap();
    let wrong = [
        agent(0, Some(Cargo::Spoil(0))),
        agent(1, Some(Cargo::Food(0))),
    ];
    assert!(f.check(&t, &wrong).is_err());
    assert!(sp.check(&t, &wrong).is_err());
}

#[test]
fn material_owner_checks_are_bijective_and_tagged() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    t.dig(pos(3, 0)).unwrap();
    f.expose(pos(3, 0)).unwrap();
    f.claim(pos(3, 0), 7).unwrap();
    f.check(&t, &[agent(7, Some(Cargo::Food(u64::MAX)))])
        .unwrap();
    for agents in [
        vec![],
        vec![agent(7, None)],
        vec![agent(7, Some(Cargo::Spoil(u64::MAX)))],
        vec![agent(8, Some(Cargo::Food(u64::MAX)))],
        vec![
            agent(7, Some(Cargo::Food(u64::MAX))),
            agent(7, Some(Cargo::Food(u64::MAX))),
        ],
        vec![
            agent(7, Some(Cargo::Food(u64::MAX))),
            agent(8, Some(Cargo::Food(9))),
        ],
    ] {
        assert!(f.check(&t, &agents).is_err());
    }
}

#[test]
fn food_transactions_reject_wrong_owner_and_repeats_atomically() {
    let mut s = setup();
    s.food = vec![
        Resource {
            id: 2,
            pos: pos(2, 0),
        },
        Resource {
            id: 9,
            pos: pos(3, 0),
        },
    ];
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    t.dig(pos(3, 0)).unwrap();
    f.expose(pos(3, 0)).unwrap();
    assert_eq!(f.claim(pos(4, 0), 7).unwrap(), None);
    f.claim(pos(2, 0), 7).unwrap();
    let before = f.clone();
    assert!(f.claim(pos(3, 0), 7).is_err());
    assert_eq!(f, before);
    for (id, owner) in [(2, 8), (12, 7), (9, 7)] {
        assert!(f.deposit(id, owner).is_err());
        assert_eq!(f, before);
    }
    f.deposit(2, 7).unwrap();
    let before = f.clone();
    assert!(f.deposit(2, 7).is_err());
    assert_eq!(f, before);
    assert_eq!(f.available(), BTreeSet::from([pos(3, 0)]));
    assert_eq!(
        f.views().iter().map(|v| v.resource.id).collect::<Vec<_>>(),
        vec![2, 9]
    );
}

#[test]
fn food_state_matches_original_cell_topology() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    f.check(&t, &[]).unwrap();
    f.expose(pos(3, 0)).unwrap();
    assert!(f.check(&t, &[]).is_err());
    let mut f = FoodLedger::new(&s, &t).unwrap();
    t.dig(pos(3, 0)).unwrap();
    assert!(f.check(&t, &[]).is_err());
    f.expose(pos(3, 0)).unwrap();
    f.check(&t, &[]).unwrap();
    f.claim(pos(3, 0), 7).unwrap();
    f.deposit(u64::MAX, 7).unwrap();
    f.check(&t, &[]).unwrap();
}

#[test]
fn spoil_capacity_counts_only_initially_solid_mask_cells() {
    let mut s = setup();
    s.diggable.push(pos(2, 0));
    let t = Terrain::new(&s).unwrap();
    assert_eq!(t.capacity(), 1);
    let mut sp = SpoilLedger::new(t.capacity()).unwrap();
    sp.spawn(pos(3, 0), 0, 0).unwrap();
    let before = sp.clone();
    assert!(sp.spawn(pos(4, 0), 1, 0).is_err());
    assert_eq!(sp, before);
    assert!(SpoilLedger::new(15_626).is_err());
}

#[test]
fn spoil_transactions_preserve_creator_origin_and_time() {
    let mut s = setup();
    s.diggable.push(pos(4, 0));
    let mut t = Terrain::new(&s).unwrap();
    let mut sp = SpoilLedger::new(t.capacity()).unwrap();
    let id = sp.spawn(pos(3, 0), 7, 10).unwrap();
    assert!(sp.check(&t, &[agent(7, Some(Cargo::Spoil(id)))]).is_err());
    t.dig(pos(3, 0)).unwrap();
    sp.check(&t, &[agent(7, Some(Cargo::Spoil(id)))]).unwrap();
    let before = sp.clone();
    for (origin, owner) in [(pos(3, 0), 8), (pos(4, 0), 7)] {
        assert!(sp.spawn(origin, owner, 11).is_err());
        assert_eq!(sp, before);
    }
    for (id, owner, tick) in [(id, 8, 11), (id, 7, 9), (12, 7, 11)] {
        assert!(sp.dispose(id, owner, tick).is_err());
        assert_eq!(sp, before);
    }
    sp.dispose(id, 7, 11).unwrap();
    let before = sp.clone();
    assert!(sp.dispose(id, 7, 11).is_err());
    assert_eq!(sp, before);
    sp.check(&t, &[]).unwrap();
    assert_eq!(
        sp.inventory(),
        SpoilInventory {
            excavated: 1,
            carried: 0,
            disposed: 1
        }
    );
    assert_eq!(
        sp.views()[0],
        SpoilView {
            id,
            origin: pos(3, 0),
            creator: 7,
            born_tick: 10,
            state: SpoilState::Disposed { tick: 11 }
        }
    );
}

#[test]
fn spoil_requires_excavated_origins_and_all_excavations() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    t.dig(pos(3, 0)).unwrap();
    let mut sp = SpoilLedger::new(t.capacity()).unwrap();
    assert!(sp.check(&t, &[]).is_err());
    sp.spawn(pos(2, 0), 0, 0).unwrap();
    assert!(sp.check(&t, &[agent(0, Some(Cargo::Spoil(0)))]).is_err());
}

#[test]
fn spoil_owner_checks_reject_mismatched_or_duplicate_cargo() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    t.dig(pos(3, 0)).unwrap();
    let mut sp = SpoilLedger::new(t.capacity()).unwrap();
    sp.spawn(pos(3, 0), 7, 0).unwrap();
    for agents in [
        vec![],
        vec![agent(7, None)],
        vec![agent(8, Some(Cargo::Spoil(0)))],
        vec![agent(7, Some(Cargo::Food(0)))],
        vec![
            agent(7, Some(Cargo::Spoil(0))),
            agent(7, Some(Cargo::Spoil(0))),
        ],
        vec![
            agent(7, Some(Cargo::Spoil(0))),
            agent(8, Some(Cargo::Spoil(9))),
        ],
    ] {
        assert!(sp.check(&t, &agents).is_err());
    }
}

#[test]
fn food_server_has_self_visible_publication_and_fidelity_priority() {
    let mut s = setup();
    let mut server = Server::default();
    assert_eq!(
        server
            .arrive(&s.parameters, 0, 2, find(&s), [0.0; 3])
            .unwrap()
            .departure,
        Departure::SiteFidelity { site: 3 }
    );
    s.parameters.lambda_fidelity = 256.0;
    let arrival = server
        .arrive(&s.parameters, 1, 2, find(&s), [0.0, 0.99, 0.9])
        .unwrap();
    assert!(arrival.published);
    assert_eq!(
        arrival.departure,
        Departure::Recruitment {
            waypoint: 1,
            site: 3
        }
    );
    assert_eq!(
        server
            .views(&s.parameters, 2)
            .unwrap()
            .iter()
            .map(|v| (v.id, v.site))
            .collect::<Vec<_>>(),
        vec![(0, 3), (1, 3)]
    );
    assert_eq!(
        server
            .arrive(&s.parameters, 2, 2, None, [0.0; 3])
            .unwrap()
            .departure,
        Departure::Recruitment {
            waypoint: 0,
            site: 3
        }
    );
}

#[test]
fn server_expiry_is_lazy_and_full_store_replacement_is_atomic() {
    let mut s = setup();
    s.parameters.lambda_waypoint = 10.0_f64.ln();
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0; 3])
        .unwrap();
    let before = server.clone();
    assert!(server.views(&s.parameters, 4).unwrap()[0].strength < 0.001);
    assert_eq!(server, before);
    let arrival = server
        .arrive(&s.parameters, 4, 1, find(&s), [0.0; 3])
        .unwrap();
    assert!(arrival.published);
    assert_eq!(server.expired(), 1);
    assert_eq!(server.views(&s.parameters, 4).unwrap()[0].id, 1);
    assert_eq!(
        server
            .arrive(&s.parameters, 8, 1, None, [0.0; 3])
            .unwrap()
            .departure,
        Departure::Uninformed
    );
    assert_eq!(server.expired(), 2);
}

#[test]
fn server_rejects_original_and_retained_capacity_without_mutation() {
    let s = setup();
    let mut server = Server::default();
    let before = server.clone();
    assert!(server
        .arrive(&s.parameters, 0, 257, None, [0.0; 3])
        .is_err());
    assert_eq!(server, before);
    assert!(server
        .arrive(&s.parameters, 0, 0, find(&s), [0.0; 3])
        .is_err());
    assert_eq!(server, before);
    server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0; 3])
        .unwrap();
    let before = server.clone();
    assert!(server
        .arrive(&s.parameters, 1, 1, find(&s), [0.0; 3])
        .is_err());
    assert_eq!(server, before);
    assert!(server.arrive(&s.parameters, 1, 0, None, [0.0; 3]).is_err());
    assert_eq!(server, before);
}

#[test]
fn server_validates_all_independent_draws_and_future_ticks() {
    let s = setup();
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 2, 1, find(&s), [0.0; 3])
        .unwrap();
    let before = server.clone();
    assert!(server.views(&s.parameters, 1).is_err());
    assert!(server.arrive(&s.parameters, 1, 1, None, [0.0; 3]).is_err());
    assert_eq!(server, before);
    for index in 0..3 {
        for value in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
            let mut draws = [0.0; 3];
            draws[index] = value;
            for f in [None, find(&s)] {
                assert!(server.arrive(&s.parameters, 20, 1, f, draws).is_err());
                assert_eq!(server, before);
            }
        }
    }
}

#[test]
fn server_uses_literal_threshold_without_rounding_decay() {
    let mut s = setup();
    s.parameters.lambda_waypoint = -(0.001_f64).ln();
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0; 3])
        .unwrap();
    assert!(server.views(&s.parameters, 1).unwrap()[0].strength >= 0.001);
    assert!(matches!(
        server
            .arrive(&s.parameters, 1, 1, None, [0.0; 3])
            .unwrap()
            .departure,
        Departure::Recruitment { .. }
    ));
    s.parameters.lambda_waypoint = f64::from_bits(s.parameters.lambda_waypoint.to_bits() + 1);
    assert!(server.views(&s.parameters, 1).unwrap()[0].strength < 0.001);
    assert_eq!(
        server
            .arrive(&s.parameters, 1, 1, None, [0.0; 3])
            .unwrap()
            .departure,
        Departure::Uninformed
    );
}

#[test]
fn work_counts_check_material_actions_movement_and_wait_equations() {
    let valid = WorkCounts {
        opportunities: 16,
        moves: 5,
        digs: 1,
        pickups: 1,
        deposits: 1,
        disposals: 1,
        waits: 7,
        departure_moves: 1,
        search_moves: 1,
        empty_return_moves: 1,
        food_moves: 1,
        spoil_moves: 1,
        transition_waits: 1,
        empty_arrival_waits: 1,
        no_neighbor_waits: 1,
        empty_congestion_waits: 2,
        food_congestion_waits: 1,
        spoil_congestion_waits: 1,
        ..WorkCounts::default()
    };
    valid.check().unwrap();
    for field in 0..3 {
        let mut bad = valid.clone();
        match field {
            0 => bad.digs += 1,
            1 => bad.spoil_moves += 1,
            _ => bad.spoil_congestion_waits += 1,
        };
        assert!(bad.check().is_err());
    }
    let overflow = WorkCounts {
        moves: u64::MAX,
        digs: 1,
        ..WorkCounts::default()
    };
    assert!(overflow.check().is_err());
}

#[test]
fn work_count_aggregation_is_atomic_under_late_field_overflow() {
    let mut counts = WorkCounts {
        spoil_hauls: u64::MAX,
        ..WorkCounts::default()
    };
    let before = counts.clone();
    assert!(counts
        .checked_include(&WorkCounts {
            opportunities: 1,
            spoil_hauls: 1,
            ..WorkCounts::default()
        })
        .is_err());
    assert_eq!(counts, before);
    let mut counts = WorkCounts::default();
    let increment = WorkCounts {
        opportunities: 1,
        digs: 1,
        spoil_hauls: 1,
        ..WorkCounts::default()
    };
    counts.checked_include(&increment).unwrap();
    assert_eq!(counts, increment);
}

#[test]
fn connected_exposed_pocket_gets_access_without_exposure_event() {
    let mut s = setup();
    s.open = vec![pos(0, 0), pos(1, 0), pos(0, 1), pos(3, 0)];
    s.diggable = vec![pos(2, 0)];
    let mut t = Terrain::new(&s).unwrap();
    let f = FoodLedger::new(&s, &t).unwrap();
    let mut a = AccessObserver::new(&s, &t, &f).unwrap();
    let initial = a.summary().unwrap();
    assert_eq!(
        (
            initial.initially_exposed,
            initial.initially_accessible,
            initial.accessible
        ),
        (1, 0, 0)
    );
    assert_eq!(initial.records[0].first_exposure, None);
    assert_eq!(initial.records[0].first_access, None);
    t.dig(pos(2, 0)).unwrap();
    let delta = a.after_dig(&s, &t, &f, &context(1, 1)).unwrap();
    assert!(delta.exposure.is_none());
    assert_eq!(delta.access.unwrap().nest_distance, 2);
    a.check(&s, &t, &f).unwrap();
    assert_eq!(a.distance(pos(3, 0)).unwrap(), Some(2));
}

#[test]
fn initial_access_is_a_flag_and_protected_food_stays_censored() {
    let mut s = setup();
    s.food = vec![
        Resource {
            id: 9,
            pos: pos(2, 0),
        },
        Resource {
            id: 2,
            pos: pos(4, 2),
        },
    ];
    let t = Terrain::new(&s).unwrap();
    let f = FoodLedger::new(&s, &t).unwrap();
    let a = AccessObserver::new(&s, &t, &f).unwrap();
    a.check(&s, &t, &f).unwrap();
    let v = a.summary().unwrap();
    assert_eq!(
        (v.initially_exposed, v.initially_accessible, v.accessible),
        (1, 1, 1)
    );
    assert_eq!(
        v.records.iter().map(|v| v.id).collect::<Vec<_>>(),
        vec![2, 9]
    );
    for r in &v.records {
        assert!(r.first_exposure.is_none());
        assert!(r.first_access.is_none());
    }
    assert_eq!(v.records[0].distance, None);
    assert_eq!(v.records[1].distance, Some(1));
    s.food.clear();
    let t = Terrain::new(&s).unwrap();
    let f = FoodLedger::new(&s, &t).unwrap();
    assert!(AccessObserver::new(&s, &t, &f)
        .unwrap()
        .summary()
        .unwrap()
        .records
        .is_empty());
}

#[test]
fn hidden_food_exposure_and_access_share_committed_candidate_context() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    let mut a = AccessObserver::new(&s, &t, &f).unwrap();
    t.dig(pos(3, 0)).unwrap();
    f.expose(pos(3, 0)).unwrap();
    let ctx = context(11, 1);
    let delta = a.after_dig(&s, &t, &f, &ctx).unwrap();
    assert_eq!(delta.exposure, delta.access);
    assert_eq!(
        delta.access.unwrap(),
        EventMilestone {
            context: ctx.clone(),
            pos: pos(3, 0),
            nest_distance: 2
        }
    );
    let r = &a.summary().unwrap().records[0];
    assert!(!r.initially_exposed);
    assert!(!r.initially_accessible);
    assert_eq!(r.first_exposure.as_ref().unwrap().context, ctx);
}

#[test]
fn shorter_routes_update_cache_but_preserve_first_access_distance() {
    let mut s = setup();
    s.open = vec![
        pos(0, 0),
        pos(1, 0),
        pos(0, 1),
        pos(1, 1),
        pos(1, 2),
        pos(2, 2),
        pos(3, 2),
        pos(3, 0),
    ];
    s.diggable = vec![pos(3, 1), pos(2, 0)];
    let mut t = Terrain::new(&s).unwrap();
    let f = FoodLedger::new(&s, &t).unwrap();
    let mut a = AccessObserver::new(&s, &t, &f).unwrap();
    t.dig(pos(3, 1)).unwrap();
    let first = a
        .after_dig(&s, &t, &f, &context(11, 1))
        .unwrap()
        .access
        .unwrap();
    assert_eq!(first.nest_distance, 6);
    t.dig(pos(2, 0)).unwrap();
    let delta = a.after_dig(&s, &t, &f, &context(21, 2)).unwrap();
    assert!(delta.access.is_none());
    let summary = a.summary().unwrap();
    assert_eq!(summary.records[0].first_access, Some(first));
    assert_eq!(summary.records[0].distance, Some(2));
    assert_eq!(summary.compute.calls, 3);
    a.check(&s, &t, &f).unwrap();
}

#[test]
fn simultaneous_food_access_delta_uses_lowest_food_id() {
    let mut s = setup();
    s.open = vec![pos(0, 0), pos(1, 0), pos(0, 1), pos(3, 0), pos(4, 0)];
    s.diggable = vec![pos(2, 0)];
    s.food = vec![
        Resource {
            id: 9,
            pos: pos(3, 0),
        },
        Resource {
            id: 2,
            pos: pos(4, 0),
        },
    ];
    let mut t = Terrain::new(&s).unwrap();
    let f = FoodLedger::new(&s, &t).unwrap();
    let mut a = AccessObserver::new(&s, &t, &f).unwrap();
    t.dig(pos(2, 0)).unwrap();
    let delta = a.after_dig(&s, &t, &f, &context(1, 1)).unwrap();
    assert_eq!(delta.access.unwrap().pos, pos(4, 0));
    let r = a.summary().unwrap().records;
    assert_eq!(r[0].first_access.as_ref().unwrap().nest_distance, 3);
    assert_eq!(r[1].first_access.as_ref().unwrap().nest_distance, 2);
}

#[test]
fn observer_calls_visits_and_queue_peaks_are_separate_and_views_are_pure() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    let mut a = AccessObserver::new(&s, &t, &f).unwrap();
    let initial = a.summary().unwrap().compute;
    assert_eq!((initial.calls, initial.visits), (1, 4));
    assert!(initial.peak_queue >= 2);
    t.dig(pos(3, 0)).unwrap();
    f.expose(pos(3, 0)).unwrap();
    a.after_dig(&s, &t, &f, &context(1, 1)).unwrap();
    let before = a.clone();
    for _ in 0..3 {
        a.check(&s, &t, &f).unwrap();
        a.summary().unwrap();
        a.distance(pos(3, 0)).unwrap();
        a.milestone(&context(2, 1), pos(3, 0)).unwrap();
    }
    assert_eq!(a, before);
    assert_eq!(a.summary().unwrap().compute.visits, 9);
}

#[test]
fn observer_rejects_reinitialization_after_digs_or_food_claims() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    t.dig(pos(3, 0)).unwrap();
    f.expose(pos(3, 0)).unwrap();
    assert!(AccessObserver::new(&s, &t, &f).is_err());
    let mut s = s;
    s.food[0].pos = pos(2, 0);
    let t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    f.claim(pos(2, 0), 0).unwrap();
    assert!(AccessObserver::new(&s, &t, &f).is_err());
}

#[test]
fn observer_updates_reject_wrong_provenance_and_are_atomic() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    let mut a = AccessObserver::new(&s, &t, &f).unwrap();
    let before = a.clone();
    assert!(a.after_dig(&s, &t, &f, &context(1, 1)).is_err());
    assert_eq!(a, before);
    t.dig(pos(3, 0)).unwrap();
    assert!(a.after_dig(&s, &t, &f, &context(1, 1)).is_err());
    assert_eq!(a, before);
    f.expose(pos(3, 0)).unwrap();
    let mut bad_contexts = vec![
        context(1, 0),
        context(1, 2),
        context(7201, 1),
        context(1_000_001, 1),
    ];
    let mut bad = context(1, 1);
    bad.opportunity = 0;
    bad_contexts.push(bad);
    let mut bad = context(1, 1);
    bad.worker = 1;
    bad_contexts.push(bad);
    let mut bad = context(1, 1);
    bad.tick = 1;
    bad_contexts.push(bad);
    let mut bad = context(1, 1);
    bad.food_delivered = 1;
    bad_contexts.push(bad);
    let mut bad = context(1, 1);
    bad.spoil_disposed = 2;
    bad_contexts.push(bad);
    for bad in bad_contexts {
        assert!(a.after_dig(&s, &t, &f, &bad).is_err());
        assert_eq!(a, before);
    }
    let mut wrong = s.clone();
    wrong.width = 6;
    assert!(a.after_dig(&wrong, &t, &f, &context(1, 1)).is_err());
    assert_eq!(a, before);
    a.after_dig(&s, &t, &f, &context(1, 1)).unwrap();
    let before = a.clone();
    assert!(a.after_dig(&s, &t, &f, &context(1, 1)).is_err());
    assert_eq!(a, before);
    assert!(a.milestone(&context(2, 0), pos(3, 0)).is_err());
    assert!(a.milestone(&context(2, 1), pos(4, 2)).is_err());
    assert!(a.distance(pos(5, 0)).is_err());
    assert_eq!(a, before);
}

#[test]
fn observer_accepts_ascending_worker_context_and_rejects_partial_tick_budget() {
    let mut s = setup();
    s.workers = vec![pos(0, 0), pos(1, 0)];
    let mut t = Terrain::new(&s).unwrap();
    let mut f = FoodLedger::new(&s, &t).unwrap();
    let mut a = AccessObserver::new(&s, &t, &f).unwrap();
    t.dig(pos(3, 0)).unwrap();
    f.expose(pos(3, 0)).unwrap();
    let ctx = EventContext {
        tick: 0,
        opportunity: 2,
        worker: 1,
        excavated: 1,
        spoil_disposed: 0,
        food_delivered: 0,
    };
    a.after_dig(&s, &t, &f, &ctx).unwrap();
    assert_eq!(
        a.summary().unwrap().records[0]
            .first_access
            .as_ref()
            .unwrap()
            .context,
        ctx
    );
    let mut s = setup();
    s.workers = (0..139).map(|i| pos(i % 2, 0)).collect();
    // Complete-tick limit with large valid populations is covered via a legal expanded nest.
    s.width = 125;
    s.height = 3;
    s.open = (0..125).map(|x| pos(x, 0)).chain([pos(0, 1)]).collect();
    s.nest = (0..70).map(|x| pos(x, 0)).collect();
    s.workers = (0..139).map(|i| pos(i / 2, 0)).collect();
    s.food.clear();
    s.diggable.clear();
    let t = Terrain::new(&s).unwrap();
    let f = FoodLedger::new(&s, &t).unwrap();
    let a = AccessObserver::new(&s, &t, &f).unwrap();
    // 139*7194=999966: the next processing tick cannot fit the complete-tick cap.
    let ctx = EventContext {
        tick: 7194,
        opportunity: 999967,
        worker: 0,
        excavated: 0,
        spoil_disposed: 0,
        food_delivered: 0,
    };
    assert!(a.milestone(&ctx, pos(0, 0)).is_err());
}
