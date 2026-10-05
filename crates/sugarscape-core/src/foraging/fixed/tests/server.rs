use super::super::{server::Server, Pos, Resource};
use super::setup;
use crate::foraging::{Departure, FindRecord};
fn fixture() -> super::super::Setup {
    let mut s = setup();
    s.resources = vec![
        Resource {
            id: 9,
            pos: Pos { x: 1, y: 1 },
        },
        Resource {
            id: 10,
            pos: Pos { x: 3, y: 3 },
        },
    ];
    s
}
fn find() -> FindRecord {
    FindRecord { site: 6, count: 1 }
}
#[test]
fn publication_is_visible_to_its_own_departure() {
    let mut s = fixture();
    s.parameters.lambda_fidelity = 256.0;
    assert_eq!(
        Server::default()
            .arrive(&s, 0, Some(find()), [0.0, 0.5, 0.0])
            .unwrap(),
        (
            Departure::Recruitment {
                waypoint: 0,
                site: 6
            },
            true
        )
    );
}
#[test]
fn fidelity_has_priority_over_own_publication() {
    assert_eq!(
        Server::default()
            .arrive(&fixture(), 0, Some(find()), [0.0, 0.0, 0.0])
            .unwrap(),
        (Departure::SiteFidelity { site: 6 }, true)
    );
}
#[test]
fn publication_and_fidelity_use_independent_draws() {
    let mut s = fixture();
    s.parameters.lambda_publish = 1.0;
    s.parameters.lambda_fidelity = 1.0;
    for (draws, want) in [
        (
            [0.9, 0.0, 0.0],
            (Departure::SiteFidelity { site: 6 }, false),
        ),
        (
            [0.0, 0.9, 0.0],
            (
                Departure::Recruitment {
                    waypoint: 0,
                    site: 6,
                },
                true,
            ),
        ),
    ] {
        assert_eq!(
            Server::default()
                .arrive(&s, 0, Some(find()), draws)
                .unwrap(),
            want
        );
    }
}
#[test]
fn empty_return_recruits_stale_site_even_with_zero_fidelity_draw() {
    let s = fixture();
    let mut server = Server::default();
    server.arrive(&s, 0, Some(find()), [0.0; 3]).unwrap();
    assert_eq!(
        server.arrive(&s, 1, None, [0.0; 3]).unwrap(),
        (
            Departure::Recruitment {
                waypoint: 0,
                site: 6
            },
            false
        )
    );
}
#[test]
fn duplicate_sites_keep_distinct_monotonic_ids() {
    let s = fixture();
    let mut server = Server::default();
    for tick in 0..2 {
        server.arrive(&s, tick, Some(find()), [0.0; 3]).unwrap();
    }
    assert_eq!(
        server
            .views(&s, 2)
            .unwrap()
            .iter()
            .map(|v| (v.id, v.site, v.strength))
            .collect::<Vec<_>>(),
        vec![(0, Pos { x: 1, y: 1 }, 1.0), (1, Pos { x: 1, y: 1 }, 1.0)]
    );
}
#[test]
fn views_do_not_expire_weak_records_but_arrival_does() {
    let mut s = fixture();
    s.parameters.lambda_waypoint = 10.0_f64.ln();
    let mut server = Server::default();
    server.arrive(&s, 0, Some(find()), [0.0; 3]).unwrap();
    let before = server.clone();
    assert!((server.views(&s, 4).unwrap()[0].strength - 0.0001).abs() < 1e-12);
    assert_eq!(server, before);
    assert_eq!(server.expired(), 0);
    assert_eq!(
        server.arrive(&s, 4, None, [0.0; 3]).unwrap(),
        (Departure::Uninformed, false)
    );
    assert!(server.views(&s, 4).unwrap().is_empty());
    assert_eq!(server.expired(), 1);
    server.arrive(&s, 4, Some(find()), [0.0; 3]).unwrap();
    assert_eq!(server.views(&s, 4).unwrap()[0].id, 1);
}
#[test]
fn weighted_intervals_use_separate_creation_ticks() {
    let mut s = fixture();
    s.parameters.lambda_waypoint = 2.0_f64.ln();
    let mut server = Server::default();
    server.arrive(&s, 0, Some(find()), [0.0; 3]).unwrap();
    server
        .arrive(&s, 1, Some(FindRecord { site: 18, count: 1 }), [0.0; 3])
        .unwrap();
    // At tick two weights are 1/4 and 1/2: boundary is 1/3.
    for (draw, id, site) in [(0.3, 0, 6), (0.34, 1, 18)] {
        assert_eq!(
            server.arrive(&s, 2, None, [0.0, 0.0, draw]).unwrap().0,
            Departure::Recruitment { waypoint: id, site }
        );
    }
}
#[test]
fn malformed_inputs_never_publish_or_expire() {
    let s = fixture();
    let mut server = Server::default();
    server.arrive(&s, 2, Some(find()), [0.0; 3]).unwrap();
    let before = server.clone();
    for (tick, f, draws) in [
        (1, None, [0.0; 3]),
        (2, Some(FindRecord { site: 25, count: 1 }), [0.0; 3]),
        (
            2,
            Some(FindRecord {
                site: 6,
                count: 257,
            }),
            [0.0; 3],
        ),
        (2, None, [f64::NAN, 0.0, 0.0]),
        (2, None, [0.0, 1.0, 0.0]),
        (2, None, [0.0, 0.0, -0.1]),
    ] {
        assert!(server.arrive(&s, tick, f, draws).is_err());
        assert_eq!(server, before);
    }
    assert!(server.views(&s, 1).is_err());
}
#[test]
fn publication_capacity_is_checked_before_mutation() {
    let mut s = fixture();
    s.resources.truncate(1);
    let mut server = Server::default();
    server.arrive(&s, 0, Some(find()), [0.0; 3]).unwrap();
    let before = server.clone();
    assert!(server.arrive(&s, 0, Some(find()), [0.0; 3]).is_err());
    assert_eq!(server, before);
}

#[test]
fn an_expired_record_frees_capacity_for_a_new_publication() {
    let mut s = fixture();
    s.resources.truncate(1);
    s.parameters.lambda_waypoint = 10.0_f64.ln();
    let mut server = Server::default();
    server.arrive(&s, 0, Some(find()), [0.0; 3]).unwrap();
    assert!(server.arrive(&s, 4, Some(find()), [0.0; 3]).unwrap().1);
    assert_eq!(server.views(&s, 4).unwrap()[0].id, 1);
    assert_eq!(server.expired(), 1);
}

#[test]
fn no_find_and_no_messages_produce_no_publication_or_recruitment() {
    let mut server = Server::default();
    assert_eq!(
        server.arrive(&setup(), 0, None, [0.0; 3]).unwrap(),
        (Departure::Uninformed, false)
    );
    assert!(server.views(&setup(), 0).unwrap().is_empty());
}
