use super::super::{ledger::Ledger, server::Server, state::Agent};
use super::*;
use crate::foraging::{Departure, FindRecord};

fn find(s: &Setup) -> Option<FindRecord> {
    Some(FindRecord {
        site: s.site(pos(3, 0)).unwrap(),
        count: 1,
    })
}

#[test]
fn server_rejects_capacity_above_food_limit_without_mutation() {
    let s = setup();
    let mut server = Server::default();
    let before = server.clone();
    assert!(server
        .arrive(&s.parameters, 0, 257, None, [0.0; 3])
        .is_err());
    assert_eq!(server, before);
}

#[test]
fn publication_replaces_expired_record_in_full_retained_store() {
    let mut s = setup();
    s.parameters.lambda_waypoint = 10.0_f64.ln();
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0; 3])
        .unwrap();
    let arrival = server
        .arrive(&s.parameters, 4, 1, find(&s), [0.0; 3])
        .unwrap();
    assert!(arrival.published);
    assert_eq!(server.expired(), 1);
    let records = server.views(&s.parameters, 4).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(
        (records[0].id, records[0].created_tick, records[0].strength),
        (1, 4, 1.0)
    );
}
fn agent(id: u32, cargo: Option<u64>) -> Agent {
    Agent {
        id,
        pos: pos(3, 0),
        phase: Phase::Returning,
        map: Knowledge::new(5, 5).unwrap(),
        cargo,
        find: None,
        site: None,
        frontier: None,
        work: WorkCounts::default(),
        compute: ComputeCounts::default(),
    }
}
#[test]
fn maximal_resource_identity_has_no_sentinel_meaning() {
    let mut s = setup();
    s.resources = vec![Resource {
        id: u64::MAX,
        pos: pos(3, 0),
    }];
    let mut l = Ledger::new(&s);
    assert_eq!(l.claim(pos(3, 0), 7).unwrap(), Some(u64::MAX));
    l.deposit(u64::MAX, 7).unwrap();
    assert_eq!(
        l.inventory(),
        Inventory {
            initial: 1,
            available: 0,
            carried: 0,
            delivered: 1
        }
    );
}
#[test]
fn claim_requires_the_supplied_resource_cell_and_empty_hands() {
    let mut s = setup();
    s.resources = vec![
        Resource {
            id: 9,
            pos: pos(3, 0),
        },
        Resource {
            id: 2,
            pos: pos(3, 1),
        },
    ];
    let mut l = Ledger::new(&s);
    assert_eq!(l.claim(pos(2, 0), 7).unwrap(), None);
    assert_eq!(l.claim(pos(3, 0), 7).unwrap(), Some(9));
    let before = l.clone();
    assert!(l.claim(pos(3, 1), 7).is_err());
    assert_eq!(l, before);
    assert_eq!(l.available(), BTreeSet::from([pos(3, 1)]));
    assert_eq!(
        l.views().iter().map(|v| v.resource.id).collect::<Vec<_>>(),
        vec![2, 9]
    );
    assert_eq!(
        l.inventory(),
        Inventory {
            initial: 2,
            available: 1,
            carried: 1,
            delivered: 0
        }
    );
}
#[test]
fn invalid_deposits_preserve_assignment() {
    let mut s = setup();
    s.resources.push(Resource {
        id: 9,
        pos: pos(3, 0),
    });
    let mut l = Ledger::new(&s);
    l.claim(pos(3, 0), 7).unwrap();
    for (id, owner) in [(9, 8), (12, 7)] {
        let before = l.clone();
        assert!(l.deposit(id, owner).is_err());
        assert_eq!(l, before);
    }
    l.deposit(9, 7).unwrap();
    let before = l.clone();
    assert!(l.deposit(9, 7).is_err());
    assert_eq!(l, before);
}
#[test]
fn cargo_ownership_is_bijective() {
    let mut s = setup();
    s.resources.push(Resource {
        id: 9,
        pos: pos(3, 0),
    });
    let mut l = Ledger::new(&s);
    l.claim(pos(3, 0), 7).unwrap();
    assert!(l.check(&[agent(7, Some(9))]).is_ok());
    for agents in [
        vec![],
        vec![agent(7, None)],
        vec![agent(8, Some(9))],
        vec![agent(7, Some(9)), agent(7, Some(9))],
    ] {
        assert!(l.check(&agents).is_err());
    }
    l.deposit(9, 7).unwrap();
    assert!(l.check(&[agent(7, Some(9))]).is_err());
}
#[test]
fn publication_is_visible_to_the_same_arrival_departure() {
    let mut s = setup();
    s.parameters.lambda_fidelity = 256.0;
    let mut server = Server::default();
    let result = server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0, 0.99, 0.0])
        .unwrap();
    assert!(result.published);
    assert!(matches!(result.departure, Departure::Recruitment { .. }));
}
#[test]
fn private_fidelity_precedes_recruitment() {
    let s = setup();
    let mut server = Server::default();
    assert_eq!(
        server
            .arrive(&s.parameters, 0, 1, find(&s), [0.0; 3])
            .unwrap()
            .departure,
        Departure::SiteFidelity {
            site: s.site(pos(3, 0)).unwrap()
        }
    );
}
#[test]
fn empty_arrival_recruits_stale_sites_without_ledger_truth() {
    let mut s = setup();
    s.parameters.lambda_fidelity = 256.0;
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0, 0.99, 0.0])
        .unwrap();
    let result = server.arrive(&s.parameters, 20, 1, None, [0.0; 3]).unwrap();
    assert!(!result.published);
    assert_eq!(
        result.departure,
        Departure::Recruitment {
            waypoint: 0,
            site: 3
        }
    );
}
#[test]
fn duplicate_sites_have_distinct_monotonic_publication_ids() {
    let s = setup();
    let mut server = Server::default();
    for tick in 0..2 {
        server
            .arrive(&s.parameters, tick, 2, find(&s), [0.0; 3])
            .unwrap();
    }
    assert_eq!(
        server
            .views(&s.parameters, 2)
            .unwrap()
            .iter()
            .map(|v| (v.id, v.site, v.created_tick))
            .collect::<Vec<_>>(),
        vec![(0, 3, 0), (1, 3, 1)]
    );
    let before = server.clone();
    assert!(server
        .arrive(&s.parameters, 2, 2, find(&s), [0.0; 3])
        .is_err());
    assert_eq!(server, before);
}
#[test]
fn views_do_not_expire_weak_records_but_arrivals_do() {
    let mut s = setup();
    s.parameters.lambda_waypoint = 10.0_f64.ln();
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0; 3])
        .unwrap();
    let before = server.clone();
    assert!(server.views(&s.parameters, 4).unwrap()[0].strength < 0.001);
    assert_eq!(server, before);
    assert_eq!(
        server
            .arrive(&s.parameters, 4, 1, None, [0.0; 3])
            .unwrap()
            .departure,
        Departure::Uninformed
    );
    assert_eq!(server.expired(), 1);
    assert!(server.views(&s.parameters, 4).unwrap().is_empty());
}
#[test]
fn decay_at_threshold_is_retained() {
    let mut s = setup();
    s.parameters.lambda_waypoint = -(0.001_f64).ln();
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 0, 1, find(&s), [0.0; 3])
        .unwrap();
    assert!((server.views(&s.parameters, 1).unwrap()[0].strength - 0.001).abs() < 1e-15);
    assert!(matches!(
        server
            .arrive(&s.parameters, 1, 1, None, [0.0; 3])
            .unwrap()
            .departure,
        Departure::Recruitment { .. }
    ));
    assert_eq!(server.expired(), 0);
    // The next representable rate crosses the threshold without changing F1 decay.
    s.parameters.lambda_waypoint = f64::from_bits(s.parameters.lambda_waypoint.to_bits() + 1);
    assert!(server.views(&s.parameters, 1).unwrap()[0].strength < 0.001);
    assert_eq!(
        server
            .arrive(&s.parameters, 1, 1, None, [0.0; 3])
            .unwrap()
            .departure,
        Departure::Uninformed
    );
    assert_eq!(server.expired(), 1);
}
#[test]
fn every_unused_draw_is_independently_validated_without_mutation() {
    let s = setup();
    for index in 0..3 {
        for value in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
            let mut server = Server::default();
            let before = server.clone();
            let mut draws = [0.0; 3];
            draws[index] = value;
            assert!(server.arrive(&s.parameters, 0, 1, None, draws).is_err());
            assert_eq!(server, before);
            assert!(server.arrive(&s.parameters, 0, 1, find(&s), draws).is_err());
            assert_eq!(server, before);
        }
    }
}
#[test]
fn future_creation_ticks_are_rejected_without_mutation() {
    let s = setup();
    let mut server = Server::default();
    server
        .arrive(&s.parameters, 2, 1, find(&s), [0.0; 3])
        .unwrap();
    let before = server.clone();
    assert!(server.views(&s.parameters, 1).is_err());
    assert!(server.arrive(&s.parameters, 1, 1, None, [0.0; 3]).is_err());
    assert_eq!(server, before);
}
#[test]
fn work_counts_enforce_actions_and_subcategories() {
    let valid = WorkCounts {
        opportunities: 10,
        moves: 4,
        departure_moves: 1,
        search_moves: 1,
        empty_return_moves: 1,
        loaded_return_moves: 1,
        pickups: 1,
        deposits: 1,
        waits: 4,
        transition_waits: 1,
        congestion_waits: 1,
        no_neighbor_waits: 1,
        empty_arrival_waits: 1,
        ..WorkCounts::default()
    };
    assert!(valid.check().is_ok());
    for field in 0..3 {
        let mut bad = valid.clone();
        match field {
            0 => bad.opportunities += 1,
            1 => bad.departure_moves += 1,
            _ => bad.transition_waits += 1,
        };
        assert!(bad.check().is_err());
    }
    let overflow = WorkCounts {
        moves: u64::MAX,
        pickups: 1,
        ..WorkCounts::default()
    };
    assert!(overflow.check().is_err());
}
#[test]
fn checked_work_sum_prepares_every_field_before_assignment() {
    let mut counts = WorkCounts {
        abandoned_targets: u64::MAX,
        ..WorkCounts::default()
    };
    let before = counts.clone();
    assert!(counts
        .checked_include(&WorkCounts {
            opportunities: 1,
            abandoned_targets: 1,
            ..WorkCounts::default()
        })
        .is_err());
    assert_eq!(counts, before);
    let mut counts = WorkCounts::default();
    let increment = WorkCounts {
        opportunities: 1,
        waits: 1,
        no_neighbor_waits: 1,
        ..WorkCounts::default()
    };
    counts.checked_include(&increment).unwrap();
    assert_eq!(counts, increment);
}
