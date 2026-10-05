use super::super::{
    departure, poisson_cdf, publication, CpfaParameters, Departure, FindRecord, Waypoint,
    WaypointSelection as Selection, WaypointThreshold as Threshold,
};

fn valid_parameters() -> CpfaParameters {
    CpfaParameters {
        p_search: 0.5,
        p_return: 0.5,
        omega: 1.0,
        lambda_informed: 1.0,
        lambda_fidelity: 1.0,
        lambda_publish: 1.0,
        lambda_waypoint: 1.0,
    }
}
fn largest_draw() -> f64 {
    f64::from_bits(1.0_f64.to_bits() - 1)
}
fn records() -> [Waypoint; 2] {
    [
        Waypoint {
            id: 11,
            site: 101,
            strength: 0.25,
        },
        Waypoint {
            id: 22,
            site: 202,
            strength: 0.75,
        },
    ]
}
fn choose(
    memory: Option<FindRecord>,
    records: &[Waypoint],
    selection: Selection,
    threshold: Threshold,
    fidelity: f64,
    recruitment: f64,
) -> Departure {
    departure(
        &valid_parameters(),
        memory,
        records,
        selection,
        threshold,
        fidelity,
        recruitment,
    )
    .unwrap()
}
#[test]
fn independent_publication_and_fidelity_draws() {
    let find = Some(FindRecord { site: 0, count: 1 });
    for (publish, fidelity, expected_publish, expected_departure) in [
        (
            0.7,
            0.8,
            Some(0),
            Departure::Recruitment {
                waypoint: 11,
                site: 101,
            },
        ),
        (0.8, 0.7, None, Departure::SiteFidelity { site: 0 }),
    ] {
        assert_eq!(
            publication(&valid_parameters(), find, publish).unwrap(),
            expected_publish
        );
        assert_eq!(
            choose(
                find,
                &records(),
                Selection::UniformComparison,
                Threshold::PaperBelow,
                fidelity,
                0.0
            ),
            expected_departure
        );
    }
}
#[test]
fn strict_cdf_equality_fails_both_decisions() {
    let probability = poisson_cdf(1, 1.0).unwrap();
    let find = Some(FindRecord {
        site: 101,
        count: 1,
    });
    assert_eq!(
        publication(&valid_parameters(), find, probability).unwrap(),
        None
    );
    assert_eq!(
        choose(
            find,
            &[],
            Selection::UniformComparison,
            Threshold::PaperBelow,
            probability,
            0.0
        ),
        Departure::Uninformed
    );
}
#[test]
fn zero_rate_probability_one_accepts_largest_draw() {
    let mut p = valid_parameters();
    p.lambda_publish = 0.0;
    p.lambda_fidelity = 0.0;
    let find = Some(FindRecord {
        site: u64::MAX,
        count: 0,
    });
    assert_eq!(
        publication(&p, find, largest_draw()).unwrap(),
        Some(u64::MAX)
    );
    assert_eq!(
        departure(
            &p,
            find,
            &records(),
            Selection::UniformComparison,
            Threshold::PaperBelow,
            largest_draw(),
            largest_draw()
        )
        .unwrap(),
        Departure::SiteFidelity { site: u64::MAX }
    );
}
#[test]
fn no_find_and_empty_or_inactive_snapshot_fall_back() {
    assert_eq!(publication(&valid_parameters(), None, 0.0).unwrap(), None);
    for snapshot in [
        &[][..],
        &[Waypoint {
            id: 0,
            site: u64::MAX,
            strength: 0.0005,
        }][..],
    ] {
        for selection in [
            Selection::UniformComparison,
            Selection::LaterArgosStrengthWeighted,
        ] {
            assert_eq!(
                choose(
                    None,
                    snapshot,
                    selection,
                    Threshold::PaperBelow,
                    0.0,
                    largest_draw()
                ),
                Departure::Uninformed
            );
        }
    }
}
#[test]
fn weighted_boundary_belongs_to_the_next_record() {
    for (draw, id, site) in [(0.0, 11, 101), (0.25, 22, 202), (largest_draw(), 22, 202)] {
        assert_eq!(
            choose(
                None,
                &records(),
                Selection::LaterArgosStrengthWeighted,
                Threshold::LaterArgosStrict,
                0.0,
                draw
            ),
            Departure::Recruitment { waypoint: id, site }
        );
    }
}
#[test]
fn uniform_intervals_preserve_active_snapshot_order() {
    let snapshot = [
        Waypoint {
            id: 99,
            site: 99,
            strength: 0.0,
        },
        Waypoint {
            id: 3,
            site: 30,
            strength: 0.1,
        },
        Waypoint {
            id: 1,
            site: 10,
            strength: 0.9,
        },
        Waypoint {
            id: 2,
            site: 20,
            strength: 0.2,
        },
    ];
    for (draw, id, site) in [(0.0, 3, 30), (0.5, 1, 10), (largest_draw(), 2, 20)] {
        assert_eq!(
            choose(
                None,
                &snapshot,
                Selection::UniformComparison,
                Threshold::PaperBelow,
                0.0,
                draw
            ),
            Departure::Recruitment { waypoint: id, site }
        );
    }
}
#[test]
fn threshold_equality_and_weighted_largest_draw_are_explicit() {
    let snapshot = [
        Waypoint {
            id: 0,
            site: 100,
            strength: 0.001,
        },
        Waypoint {
            id: u64::MAX,
            site: 200,
            strength: 0.2,
        },
    ];
    for selection in [
        Selection::UniformComparison,
        Selection::LaterArgosStrengthWeighted,
    ] {
        assert_eq!(
            choose(None, &snapshot, selection, Threshold::PaperBelow, 0.0, 0.0),
            Departure::Recruitment {
                waypoint: 0,
                site: 100
            }
        );
        assert_eq!(
            choose(
                None,
                &snapshot,
                selection,
                Threshold::LaterArgosStrict,
                0.0,
                0.0
            ),
            Departure::Recruitment {
                waypoint: u64::MAX,
                site: 200
            }
        );
    }
    assert_eq!(
        choose(
            None,
            &snapshot,
            Selection::LaterArgosStrengthWeighted,
            Threshold::PaperBelow,
            0.0,
            largest_draw()
        ),
        Departure::Recruitment {
            waypoint: u64::MAX,
            site: 200
        }
    );
    assert_eq!(
        choose(
            None,
            &snapshot[..1],
            Selection::UniformComparison,
            Threshold::LaterArgosStrict,
            0.0,
            0.0
        ),
        Departure::Uninformed
    );
}
#[test]
fn duplicate_sites_with_distinct_ids_are_legal() {
    let snapshot = [
        Waypoint {
            id: 11,
            site: 101,
            strength: 0.25,
        },
        Waypoint {
            id: 22,
            site: 101,
            strength: 0.75,
        },
    ];
    assert_eq!(
        choose(
            None,
            &snapshot,
            Selection::LaterArgosStrengthWeighted,
            Threshold::PaperBelow,
            0.0,
            0.25
        ),
        Departure::Recruitment {
            waypoint: 22,
            site: 101
        }
    );
}
#[test]
fn validates_every_draw_even_when_unused() {
    for draw in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.0] {
        assert_eq!(
            publication(&valid_parameters(), None, draw).unwrap_err()[0].field,
            "draw"
        );
        for (fidelity, recruitment, field) in [
            (draw, 0.0, "fidelity_draw"),
            (0.0, draw, "recruitment_draw"),
        ] {
            assert_eq!(
                departure(
                    &valid_parameters(),
                    Some(FindRecord { site: 1, count: 1 }),
                    &[],
                    Selection::UniformComparison,
                    Threshold::PaperBelow,
                    fidelity,
                    recruitment
                )
                .unwrap_err()[0]
                    .field,
                field
            );
        }
    }
    assert_eq!(
        publication(&valid_parameters(), None, largest_draw()).unwrap(),
        None
    );
    assert_eq!(
        choose(
            None,
            &[],
            Selection::UniformComparison,
            Threshold::PaperBelow,
            largest_draw(),
            largest_draw()
        ),
        Departure::Uninformed
    );
}
#[test]
fn validates_inactive_strengths_before_fidelity() {
    for strength in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.1] {
        let snapshot = [
            records()[0],
            Waypoint {
                id: 22,
                site: 202,
                strength,
            },
        ];
        assert_eq!(
            departure(
                &valid_parameters(),
                Some(FindRecord { site: 1, count: 1 }),
                &snapshot,
                Selection::UniformComparison,
                Threshold::PaperBelow,
                0.0,
                0.0
            )
            .unwrap_err()[0]
                .field,
            "waypoints[1].strength"
        );
    }
}
#[test]
fn duplicate_inactive_ids_reject_before_fidelity() {
    let snapshot = [
        Waypoint {
            id: 11,
            site: 101,
            strength: 0.0,
        },
        Waypoint {
            id: 11,
            site: 202,
            strength: 0.0,
        },
    ];
    assert_eq!(
        departure(
            &valid_parameters(),
            Some(FindRecord { site: 1, count: 1 }),
            &snapshot,
            Selection::UniformComparison,
            Threshold::PaperBelow,
            0.0,
            0.0
        )
        .unwrap_err()[0]
            .field,
        "waypoints[1].id"
    );
}
#[test]
fn accumulates_parameter_observation_and_snapshot_errors_in_order() {
    let mut p = valid_parameters();
    p.p_search = f64::NAN;
    p.lambda_publish = -1.0;
    let find = Some(FindRecord {
        site: 1,
        count: 257,
    });
    let publication_fields: Vec<_> = publication(&p, find, 1.0)
        .unwrap_err()
        .into_iter()
        .map(|e| e.field)
        .collect();
    assert_eq!(
        publication_fields,
        ["p_search", "lambda_publish", "find.count", "draw"]
    );
    let snapshot = [
        Waypoint {
            id: 1,
            site: 1,
            strength: -1.0,
        },
        Waypoint {
            id: 1,
            site: 2,
            strength: f64::NAN,
        },
    ];
    let fields: Vec<_> = departure(
        &p,
        find,
        &snapshot,
        Selection::UniformComparison,
        Threshold::PaperBelow,
        1.0,
        1.0,
    )
    .unwrap_err()
    .into_iter()
    .map(|e| e.field)
    .collect();
    assert_eq!(
        fields,
        [
            "p_search",
            "lambda_publish",
            "memory.count",
            "fidelity_draw",
            "recruitment_draw",
            "waypoints[0].strength",
            "waypoints[1].id",
            "waypoints[1].strength"
        ]
    );
}
#[test]
fn all_parameter_fields_are_checked_in_irrelevant_branches() {
    for field in 0..7 {
        let mut p = valid_parameters();
        let fields = [
            &mut p.p_search,
            &mut p.p_return,
            &mut p.omega,
            &mut p.lambda_informed,
            &mut p.lambda_fidelity,
            &mut p.lambda_publish,
            &mut p.lambda_waypoint,
        ];
        *fields.into_iter().nth(field).unwrap() = f64::NAN;
        assert!(publication(&p, None, 0.0).is_err());
        assert!(departure(
            &p,
            None,
            &[],
            Selection::UniformComparison,
            Threshold::PaperBelow,
            0.0,
            0.0
        )
        .is_err());
    }
}
