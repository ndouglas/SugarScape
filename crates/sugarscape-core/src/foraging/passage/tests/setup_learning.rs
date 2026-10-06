use super::*;
fn rejects(s: Setup, field: &str) {
    assert!(
        s.validate().unwrap_err().iter().any(|e| e.field == field),
        "{field}"
    );
}
#[test]
fn chamber_capacity_applies_per_cell() {
    let mut s = setup();
    s.workers = vec![pos(0, 0), pos(0, 0), pos(1, 0), pos(1, 0)];
    assert!(s.validate().is_ok());
    s.workers.push(pos(0, 0));
    rejects(s, "workers[4]");
}
#[test]
fn dimensions_and_population_bounds() {
    for value in [2, 126, u32::MAX] {
        let mut s = setup();
        s.width = value;
        rejects(s, "width");
        let mut s = setup();
        s.height = value;
        rejects(s, "height");
    }
    for n in [0, 257] {
        let mut s = setup();
        s.workers = vec![pos(0, 0); n];
        rejects(s, "workers");
    }
    let mut s = setup();
    s.resources = vec![
        Resource {
            id: 0,
            pos: pos(2, 0)
        };
        257
    ];
    rejects(s, "resources");
    let mut s = setup();
    s.width = 125;
    s.height = 125;
    assert!(s.validate().is_ok());
}
#[test]
fn original_indexed_geometry_errors_are_aggregated() {
    let mut s = setup();
    s.open.push(pos(0, 0));
    s.open.push(pos(5, 0));
    s.nest.push(pos(0, 0));
    s.nest.push(pos(4, 4));
    s.workers.push(pos(2, 0));
    let errors = s.validate().unwrap_err();
    for field in ["open[5]", "open[6]", "nest[2]", "nest[3]", "workers[1]"] {
        assert!(errors.iter().any(|e| e.field == field), "{field}");
    }
    let mut s = setup();
    s.open.clear();
    rejects(s, "open");
    let mut s = setup();
    s.nest = vec![pos(0, 0)];
    rejects(s, "nest");
    let mut s = setup();
    s.nest = vec![pos(0, 0), pos(3, 0)];
    rejects(s, "nest");
}
#[test]
fn resource_errors_are_indexed() {
    let mut s = setup();
    s.resources = vec![
        Resource {
            id: 7,
            pos: pos(2, 0),
        },
        Resource {
            id: 7,
            pos: pos(2, 0),
        },
    ];
    rejects(s.clone(), "resources[1].id");
    rejects(s, "resources[1].pos");
    for p in [pos(4, 4), pos(0, 0), pos(5, 0)] {
        let mut s = setup();
        s.resources = vec![Resource { id: 0, pos: p }];
        rejects(s, "resources[0].pos");
    }
}
#[test]
fn disconnected_food_and_max_identity_are_valid() {
    let mut s = setup();
    s.open.push(pos(4, 4));
    s.resources.push(Resource {
        id: u64::MAX,
        pos: pos(4, 4),
    });
    assert!(s.validate().is_ok());
    assert!(setup().validate().is_ok());
}
#[test]
fn parameters_keep_f1_domains() {
    for v in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        let mut s = setup();
        s.parameters.p_search = v;
        rejects(s, "parameters.p_search");
        let mut s = setup();
        s.parameters.p_return = v;
        rejects(s, "parameters.p_return");
    }
    for v in [f64::NAN, f64::INFINITY, -0.1, 257.0] {
        let mut s = setup();
        s.parameters.lambda_fidelity = v;
        rejects(s, "parameters.lambda_fidelity");
        let mut s = setup();
        s.parameters.lambda_publish = v;
        rejects(s, "parameters.lambda_publish");
    }
    for v in [f64::NAN, f64::INFINITY, -0.1] {
        let mut s = setup();
        s.parameters.lambda_waypoint = v;
        rejects(s, "parameters.lambda_waypoint");
    }
    let mut s = setup();
    s.parameters.lambda_fidelity = 256.0;
    s.parameters.lambda_publish = 256.0;
    s.parameters.lambda_waypoint = f64::MAX;
    assert!(s.validate().is_ok());
    assert_eq!(s.parameters.information().omega, 0.0);
    assert_eq!(s.parameters.information().lambda_informed, 0.0);
}
#[test]
fn normalization_preserves_worker_identity() {
    let mut s = setup();
    s.open.reverse();
    s.nest.reverse();
    s.workers = vec![pos(1, 0), pos(0, 0)];
    s.resources = vec![
        Resource {
            id: 9,
            pos: pos(3, 0),
        },
        Resource {
            id: 2,
            pos: pos(2, 0),
        },
    ];
    let n = s.normalized().unwrap();
    assert_eq!(n.workers, vec![pos(1, 0), pos(0, 0)]);
    assert_eq!(n.nest, vec![pos(0, 0), pos(1, 0)]);
    assert!(n.open.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(n.resources[0].id, 2);
}
#[test]
fn checked_geometry_and_neighbor_order() {
    assert_eq!(
        pos(1, 1).neighbors(5, 5),
        vec![pos(1, 0), pos(1, 2), pos(2, 1), pos(0, 1)]
    );
    assert_eq!(pos(0, 0).neighbors(5, 5), vec![pos(0, 1), pos(1, 0)]);
    assert!(pos(u32::MAX, 0).neighbors(5, 5).is_empty());
    assert_eq!(setup().site(pos(4, 4)).unwrap(), 24);
    assert_eq!(setup().position(24).unwrap(), pos(4, 4));
    assert!(setup().site(pos(5, 0)).is_err());
    assert!(setup().position(25).is_err());
    let mut s = setup();
    s.width = 0;
    assert!(s.position(0).is_err());
    assert!(s.site(pos(0, 0)).is_err());
    assert!(Knowledge::new(u32::MAX, u32::MAX).is_err());
}
#[test]
fn learning_does_not_reveal_second_hop_food_or_topology() {
    let s = setup();
    let o = observe(
        &s,
        pos(0, 0),
        &BTreeMap::from([(pos(0, 0), 1)]),
        &BTreeSet::from([pos(3, 0)]),
    )
    .unwrap();
    let mut k = Knowledge::new(5, 5).unwrap();
    assert_eq!(k.learn(&o).unwrap(), 3);
    assert_eq!(k.kind(pos(2, 0)).unwrap(), CellKnowledge::Unknown);
    assert!(o.cells.iter().all(|c| c.pos != pos(3, 0)));
    assert_eq!(
        o.cells.iter().map(|c| c.pos).collect::<Vec<_>>(),
        vec![pos(0, 0), pos(0, 1), pos(1, 0)]
    );
    assert_eq!(k.counts(), (2, 1));
    assert_eq!(k.known().len(), 3);
    assert!(k.known().windows(2).all(|w| w[0].pos < w[1].pos));
    assert!(k.kind(pos(5, 0)).is_err());
}
#[test]
fn changing_ephemeral_data_does_not_change_knowledge() {
    let mut k = Knowledge::new(5, 5).unwrap();
    let first = observe(&setup(), pos(0, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    k.learn(&first).unwrap();
    let before = k.clone();
    let o = observe(
        &setup(),
        pos(0, 0),
        &BTreeMap::from([(pos(1, 0), 2)]),
        &BTreeSet::from([pos(1, 0)]),
    )
    .unwrap();
    assert_eq!(k.learn(&o).unwrap(), 0);
    assert_eq!(k, before);
}
#[test]
fn malformed_observation_and_late_conflict_roll_back() {
    let good = observe(&setup(), pos(0, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    for mode in 0..8 {
        let mut o = good.clone();
        match mode {
            0 => o.origin = pos(4, 4),
            1 => o.cells.push(o.cells[0].clone()),
            2 => o.cells[1].pos = pos(2, 0),
            3 => o.cells[1].pos = pos(5, 0),
            4 => o.cells[0].open = false,
            5 => o.cells[1].occupants = 1,
            6 => o.cells[1].food = true,
            _ => o.cells[0].occupants = 3,
        }
        let mut k = Knowledge::new(5, 5).unwrap();
        let before = k.clone();
        assert!(k.learn(&o).is_err(), "mode {mode}");
        assert_eq!(k, before);
    }
    let mut k = Knowledge::new(5, 5).unwrap();
    k.learn(&good).unwrap();
    let before = k.clone();
    let o = Observation {
        origin: pos(1, 0),
        cells: vec![
            ObservedCell {
                pos: pos(2, 0),
                open: true,
                occupants: 0,
                food: false,
            },
            ObservedCell {
                pos: pos(1, 0),
                open: true,
                occupants: 0,
                food: false,
            },
            ObservedCell {
                pos: pos(0, 0),
                open: false,
                occupants: 0,
                food: false,
            },
        ],
    };
    assert!(k.learn(&o).is_err());
    assert_eq!(k, before);
}
#[test]
fn observation_rejects_invalid_local_state() {
    assert!(observe(&setup(), pos(4, 4), &BTreeMap::new(), &BTreeSet::new()).is_err());
    assert!(observe(
        &setup(),
        pos(0, 0),
        &BTreeMap::from([(pos(0, 0), 3)]),
        &BTreeSet::new()
    )
    .is_err());
    assert!(observe(
        &setup(),
        pos(0, 0),
        &BTreeMap::new(),
        &BTreeSet::from([pos(0, 1)])
    )
    .is_err());
}
#[test]
fn one_candidate_still_draws() {
    let mut d = Scripted::new(&[0.9]);
    assert_eq!(choose(&[pos(1, 0)], &mut d).unwrap(), Some(pos(1, 0)));
    assert_eq!(d.next, 1);
}
#[test]
fn choices_use_half_open_intervals_and_empty_choices_do_not_draw() {
    let mut d = Scripted::new(&[0.0, f64::from_bits(1.0f64.to_bits() - 1)]);
    assert_eq!(draw_index(&mut d, 4).unwrap(), 0);
    assert_eq!(draw_index(&mut d, 4).unwrap(), 3);
    assert_eq!(choose(&[], &mut d).unwrap(), None);
    assert!(draw_index(&mut d, 0).is_err());
    assert_eq!(d.next, 2);
    for v in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
        assert!(draw_index(&mut Scripted::new(&[v]), 1).is_err());
    }
    let mut rng = crate::rng::seeded(7);
    assert!((0.0..1.0).contains(&checked_uniform(&mut PcgDraws(&mut rng)).unwrap()));
}
#[test]
fn compute_totals_are_atomic_and_queue_peak_is_maximum() {
    let mut c = ComputeCounts {
        observations: 1,
        cells_inspected: 2,
        cells_learned: 3,
        route_calls: 4,
        route_visits: 5,
        peak_queue: 6,
        frontier_scans: 7,
    };
    let d = c;
    c.checked_include(&d).unwrap();
    assert_eq!(
        c,
        ComputeCounts {
            observations: 2,
            cells_inspected: 4,
            cells_learned: 6,
            route_calls: 8,
            route_visits: 10,
            peak_queue: 6,
            frontier_scans: 14
        }
    );
    for field in 0..6 {
        let mut d = ComputeCounts::default();
        match field {
            0 => d.observations = u64::MAX,
            1 => d.cells_inspected = u64::MAX,
            2 => d.cells_learned = u64::MAX,
            3 => d.route_calls = u64::MAX,
            4 => d.route_visits = u64::MAX,
            _ => d.frontier_scans = u64::MAX,
        };
        let before = c;
        assert!(c.checked_include(&d).is_err());
        assert_eq!(c, before);
    }
    c.checked_include(&ComputeCounts {
        peak_queue: 20,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(c.peak_queue, 20);
}
