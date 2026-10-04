use sugarscape_core::foraging::{
    departure, publication, CpfaParameters, Departure, FindRecord, Waypoint, WaypointSelection,
    WaypointThreshold,
};

fn parameters() -> CpfaParameters {
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

#[test]
fn publication_and_private_fidelity_use_independent_variates() {
    let p = parameters();
    let find = Some(FindRecord { site: 77, count: 1 });
    let records = [Waypoint {
        id: 9,
        site: 88,
        strength: 0.8,
    }];
    assert_eq!(publication(&p, find, 0.8).unwrap(), None);
    assert_eq!(
        departure(
            &p,
            find,
            &records,
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::LaterArgosStrict,
            0.7,
            0.0
        )
        .unwrap(),
        Departure::SiteFidelity { site: 77 }
    );
    assert_eq!(publication(&p, find, 0.7).unwrap(), Some(77));
    assert_eq!(
        departure(
            &p,
            find,
            &records,
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::LaterArgosStrict,
            0.8,
            0.0
        )
        .unwrap(),
        Departure::Recruitment {
            waypoint: 9,
            site: 88
        }
    );
}

#[test]
fn empty_return_and_retained_private_memory_are_separate_inputs() {
    let p = parameters();
    let memory = Some(FindRecord { site: 77, count: 1 });
    assert_eq!(publication(&p, None, 0.0).unwrap(), None);
    assert_eq!(
        departure(
            &p,
            memory,
            &[],
            WaypointSelection::UniformComparison,
            WaypointThreshold::PaperBelow,
            0.7,
            0.0
        )
        .unwrap(),
        Departure::SiteFidelity { site: 77 }
    );
}

#[test]
fn recruitment_accepts_an_active_record_without_resource_truth() {
    let p = parameters();
    // The caller may know site 77 is depleted; F1 receives no such truth.
    let records = [Waypoint {
        id: 9,
        site: 77,
        strength: 0.8,
    }];
    let before = records;
    assert_eq!(
        departure(
            &p,
            None,
            &records,
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::LaterArgosStrict,
            0.0,
            0.0
        )
        .unwrap(),
        Departure::Recruitment {
            waypoint: 9,
            site: 77
        }
    );
    assert_eq!(records, before);
}
