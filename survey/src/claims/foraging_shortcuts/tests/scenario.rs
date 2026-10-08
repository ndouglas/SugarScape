use super::super::{manifest::*, scenario::*, *};
use std::collections::BTreeSet;
use sugarscape_core::foraging::construction as core;

fn pos(x: u32, y: u32) -> core::Pos {
    core::Pos { x, y }
}

#[test]
fn literal_routes_match_independent_hand_counts_and_distances() {
    for (geometry, count, distance, capacity) in [
        (Geometry::Straight, 40, 15, 489),
        (Geometry::Detour, 73, 48, 456),
        (Geometry::Twisting, 85, 60, 444),
    ] {
        let paid = build(Panel::Route, geometry, Regime::Paid).unwrap();
        let protected = build(Panel::Route, geometry, Regime::Protected).unwrap();
        assert_eq!(paid.open.len(), count);
        assert_eq!(paid.open, protected.open);
        assert_eq!(
            patch_distances(&paid)
                .unwrap()
                .iter()
                .filter_map(|(_, d)| *d)
                .min(),
            Some(distance)
        );
        let expected_distances: Vec<_> = (2..=5)
            .flat_map(|y| {
                (2..=5).map(move |x: u32| {
                    let approach = if geometry == Geometry::Straight {
                        x.abs_diff(3)
                    } else {
                        5 - x
                    };
                    distance + 5 - y + approach
                })
            })
            .enumerate()
            .map(|(id, d)| (id as u64, Some(d)))
            .collect();
        assert_eq!(patch_distances(&paid).unwrap(), expected_distances);
        assert_eq!(paid.diggable.len(), 529);
        assert_eq!(
            paid.diggable
                .iter()
                .filter(|p| !paid.open.contains(p))
                .count(),
            capacity
        );
        assert!(protected.diggable.is_empty());
        assert!(paid
            .diggable
            .iter()
            .all(|p| (1..=23).contains(&p.x) && (1..=23).contains(&p.y)));
        assert!(paid
            .open
            .iter()
            .all(|p| (1..=23).contains(&p.x) && (1..=23).contains(&p.y)));
        let reference = build(Panel::Route, geometry, Regime::AlreadyOpen).unwrap();
        assert_eq!(
            patch_distances(&reference)
                .unwrap()
                .iter()
                .filter_map(|(_, d)| *d)
                .min(),
            Some(15)
        );
        assert!(reference.diggable.is_empty());
        let sealed = build(Panel::Access, geometry, Regime::Paid).unwrap();
        assert_eq!(sealed.open.len(), count - 1);
        assert_eq!(
            sealed
                .diggable
                .iter()
                .filter(|p| !sealed.open.contains(p))
                .count(),
            capacity + 1
        );
        assert!(patch_distances(&sealed)
            .unwrap()
            .iter()
            .all(|(_, d)| d.is_none()));
        let gate = if geometry == Geometry::Straight {
            pos(3, 6)
        } else {
            pos(6, 5)
        };
        let mut expected_open = paid.open.clone();
        expected_open.retain(|p| *p != gate);
        assert_eq!(sealed.open, expected_open);
        let protected_sealed = build(Panel::Access, geometry, Regime::Protected).unwrap();
        assert_eq!(protected_sealed.open, sealed.open);
        assert!(protected_sealed.diggable.is_empty());
        assert!(patch_distances(&protected_sealed)
            .unwrap()
            .iter()
            .all(|(_, d)| d.is_none()));
    }
}

#[test]
fn every_scene_preserves_resource_identity_spawns_nest_and_outlet() {
    let manifest = candidate().unwrap();
    let nest: BTreeSet<_> = (20..=22)
        .flat_map(|y| (2..=4).map(move |x| pos(x, y)))
        .collect();
    let food: Vec<_> = (2..=5)
        .flat_map(|y| (2..=5).map(move |x| pos(x, y)))
        .enumerate()
        .map(|(id, p)| core::Resource {
            id: id as u64,
            pos: p,
        })
        .collect();
    for c in &manifest.conditions {
        let s = &c.setup;
        assert_eq!((s.width, s.height), (25, 25));
        assert_eq!(s.nest.iter().copied().collect::<BTreeSet<_>>(), nest);
        assert_eq!(s.waste, pos(1, 21));
        assert!(s.open.contains(&s.waste));
        assert!(s.nest.iter().all(|p| s.open.contains(p)));
        assert_eq!(
            s.workers,
            vec![
                pos(2, 21),
                pos(2, 21),
                pos(3, 21),
                pos(3, 21),
                pos(4, 21),
                pos(4, 21),
                pos(3, 20),
                pos(3, 20)
            ]
        );
        assert_eq!(s.food, food);
        assert!(s.food.iter().all(|r| s.open.contains(&r.pos)));
        assert_eq!(
            patch_distances(s)
                .unwrap()
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            (0..16).collect::<Vec<_>>()
        );
        assert_eq!(s.clone().normalized().unwrap(), *s);
        assert_eq!(
            s.parameters,
            core::Parameters {
                p_search: 0.05,
                p_return: 0.01,
                lambda_fidelity: 1.0,
                lambda_publish: 1.0,
                lambda_waypoint: 0.01
            }
        );
        assert_eq!(
            c.options,
            core::RunOptions {
                ticks: 512,
                sample_every: 128,
                snapshots: true
            }
        );
    }
    assert_eq!(
        build(Panel::Route, Geometry::Straight, Regime::Protected).unwrap(),
        build(Panel::Route, Geometry::Straight, Regime::AlreadyOpen).unwrap()
    );
}

#[test]
fn geometry_uses_exact_literal_inclusive_vertices() {
    let straight = build(Panel::Route, Geometry::Straight, Regime::Protected).unwrap();
    for (geometry, vertices) in [
        (Geometry::Straight, vec![(3, 20), (3, 5)]),
        (Geometry::Detour, vec![(4, 20), (21, 20), (21, 5), (5, 5)]),
        (
            Geometry::Twisting,
            vec![
                (4, 20),
                (21, 20),
                (21, 17),
                (18, 17),
                (18, 14),
                (21, 14),
                (21, 11),
                (18, 11),
                (18, 8),
                (21, 8),
                (21, 5),
                (5, 5),
            ],
        ),
    ] {
        let mut expected: BTreeSet<_> = straight
            .nest
            .iter()
            .copied()
            .chain(straight.food.iter().map(|r| r.pos))
            .chain([straight.waste])
            .collect();
        // Independent test expansion steps one cell toward each endpoint.
        for endpoints in vertices.windows(2) {
            let (mut x, mut y) = endpoints[0];
            let (end_x, end_y) = endpoints[1];
            expected.insert(pos(x, y));
            while (x, y) != (end_x, end_y) {
                if x < end_x {
                    x += 1;
                } else if x > end_x {
                    x -= 1;
                } else if y < end_y {
                    y += 1;
                } else {
                    y -= 1;
                }
                expected.insert(pos(x, y));
            }
        }
        let baseline = build(Panel::Route, geometry, Regime::Protected).unwrap();
        assert_eq!(
            baseline.open.iter().copied().collect::<BTreeSet<_>>(),
            expected
        );
        expected.extend(straight.open.iter().copied());
        assert_eq!(
            build(Panel::Route, geometry, Regime::AlreadyOpen)
                .unwrap()
                .open
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
            expected
        );
    }
}

#[test]
fn unsupported_access_reference_and_diagonal_segments_fail() {
    for geometry in [Geometry::Straight, Geometry::Detour, Geometry::Twisting] {
        assert!(build(Panel::Access, geometry, Regime::AlreadyOpen)
            .unwrap_err()
            .contains("already_open"));
    }
    let mut cells = BTreeSet::new();
    assert!(segment(pos(1, 1), pos(2, 2), &mut cells)
        .unwrap_err()
        .contains("cardinal"));
    assert!(cells.is_empty());
    segment(pos(3, 5), pos(3, 5), &mut cells).unwrap();
    assert_eq!(cells, BTreeSet::from([pos(3, 5)]));
}

#[test]
fn independent_bfs_handles_multisource_boundaries_and_invalid_setups() {
    let s = core::Setup {
        width: 3,
        height: 3,
        open: vec![pos(0, 0), pos(1, 0), pos(0, 1), pos(2, 0)],
        diggable: vec![],
        nest: vec![pos(0, 0), pos(1, 0)],
        waste: pos(0, 1),
        workers: vec![pos(0, 0)],
        food: vec![
            core::Resource {
                id: 9,
                pos: pos(2, 0),
            },
            core::Resource {
                id: 10,
                pos: pos(2, 2),
            },
        ],
        parameters: core::Parameters {
            p_search: 0.05,
            p_return: 0.01,
            lambda_fidelity: 1.0,
            lambda_publish: 1.0,
            lambda_waypoint: 0.01,
        },
    };
    assert_eq!(patch_distances(&s).unwrap(), vec![(9, Some(1)), (10, None)]);
    let mut invalid = s;
    invalid.open.push(pos(3, 0));
    assert!(patch_distances(&invalid).is_err());
}

#[test]
fn manifest_declares_exact_canonical_conditions_and_workload() {
    let m = candidate().unwrap();
    let ids = [
        "route.straight.paid",
        "route.straight.protected",
        "route.straight.already_open",
        "route.detour.paid",
        "route.detour.protected",
        "route.detour.already_open",
        "route.twisting.paid",
        "route.twisting.protected",
        "route.twisting.already_open",
        "access.straight.paid",
        "access.straight.protected",
        "access.detour.paid",
        "access.detour.protected",
        "access.twisting.paid",
        "access.twisting.protected",
    ];
    assert_eq!(
        m.conditions
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        m.conditions
            .iter()
            .map(|c| &c.id)
            .collect::<BTreeSet<_>>()
            .len(),
        15
    );
    for c in &m.conditions {
        assert_eq!(c.setup, build(c.panel, c.geometry, c.regime).unwrap());
        assert_eq!(condition(&m, &c.id).unwrap(), c);
    }
    assert!(condition(&m, "missing").unwrap_err().contains("missing"));
    assert_eq!(m.schema, "foraging-shortcut-manifest-v1");
    assert_eq!(m.version, 1);
    assert_eq!(m.status, "draft");
    assert!(!m.execution_authorized);
    assert_eq!(
        m.protocol,
        "docs/superpowers/specs/2026-10-07-foraging-5-shortcut-comparison-design.md"
    );
    assert_eq!(m.construction_seeds, vec![7, 8]);
    assert_eq!(m.scientific_seeds, (10001..=10040).collect::<Vec<_>>());
    assert_eq!(
        (m.raw_record_limit, m.raw_total_limit, m.metadata_limit),
        (4 * 1024 * 1024, 1024 * 1024 * 1024, 4 * 1024 * 1024)
    );
    for (mode, seeds) in [
        (CollectionMode::Construction, vec![7, 8]),
        (CollectionMode::Scientific, (10001..=10040).collect()),
    ] {
        let expected: Vec<_> = ids
            .iter()
            .flat_map(|id| {
                seeds.iter().map(move |seed| RunKey {
                    condition: (*id).into(),
                    seed: *seed,
                })
            })
            .collect();
        assert_eq!(expected_keys(&m, mode), expected);
    }
    assert!(authorize(&m, CollectionMode::Scientific).is_err());
    assert!(authorize(&m, CollectionMode::Construction).is_ok());
}

#[test]
fn manifest_bytes_and_sha256_are_stable_and_canonical() {
    let bytes = manifest_bytes().unwrap();
    let mut expected = serde_json::to_vec_pretty(&candidate().unwrap()).unwrap();
    expected.push(b'\n');
    assert_eq!(bytes, expected);
    assert_eq!(manifest_bytes().unwrap(), bytes);
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256(&bytes),
        "8e532eaf779b970d4bd9a2550fc1634f89abe63a00b7d05d3134f04a4a2deb62"
    );
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["conditions"][5]["regime"], "already_open");
    assert_eq!(json["conditions"][9]["panel"], "access");
}

#[test]
fn scientific_authorization_requires_both_registration_fields() {
    let mut m = candidate().unwrap();
    m.execution_authorized = true;
    assert!(authorize(&m, CollectionMode::Scientific).is_err());
    m.status = "registered".into();
    m.execution_authorized = false;
    assert!(authorize(&m, CollectionMode::Scientific).is_err());
    m.execution_authorized = true;
    assert!(authorize(&m, CollectionMode::Scientific).is_ok());
}

#[test]
fn decoded_shared_identity_records_reject_unknown_fields() {
    assert!(serde_json::from_str::<RunKey>(
        r#"{"condition":"route.straight.paid","seed":7,"extra":0}"#
    )
    .is_err());
    assert!(serde_json::from_str::<Provenance>(r#"{"code_revision":"a","protocol_revision":"b","manifest_sha256":"c","collector_sha256":"d","extra":0}"#).is_err());
    assert!(serde_json::from_str::<Regime>(r#""AlreadyOpen""#).is_err());
}
