use super::*;
#[test]
fn legal_mask_food_and_identity_variants() {
    let mut s = setup();
    assert!(!s.diggable.contains(&s.waste));
    s.validate().unwrap();
    s.diggable.push(pos(0, 0));
    s.food.push(Resource {
        id: 0,
        pos: pos(4, 2),
    });
    s.open.push(pos(4, 1));
    s.food.push(Resource {
        id: 1,
        pos: pos(4, 1),
    });
    s.validate().unwrap();
    let t = Terrain::new(&s).unwrap();
    assert_eq!(t.capacity(), 1);
    assert!(!t.is_diggable(pos(0, 0)).unwrap());
}
#[test]
fn rejects_bad_dimensions_and_lengths() {
    for value in [2, 126] {
        let mut s = setup();
        s.width = value;
        assert!(s.validate().is_err());
        s = setup();
        s.height = value;
        assert!(s.validate().is_err());
    }
    for n in [0, 257] {
        let mut s = setup();
        s.workers = vec![pos(0, 0); n];
        assert!(s.validate().is_err());
    }
    let mut s = setup();
    s.food = vec![s.food[0].clone(); 257];
    assert!(s.validate().is_err());
}
#[test]
fn rejects_solid_diggable_waste_outlet() {
    let mut s = setup();
    s.waste = pos(2, 1);
    s.diggable.push(s.waste);
    let errors = s.validate().unwrap_err();
    assert!(errors.iter().any(|error| error.field == "waste"));
}
#[test]
fn rejects_original_indexed_geometry_food_and_parameter_errors() {
    let mutations: Vec<fn(&mut Setup)> = vec![
        |s| s.open.push(s.open[0]),
        |s| s.open.push(pos(5, 0)),
        |s| s.diggable.push(s.diggable[0]),
        |s| s.diggable.push(pos(5, 0)),
        |s| s.nest.push(s.nest[0]),
        |s| s.nest.push(pos(5, 0)),
        |s| s.nest = vec![pos(0, 0)],
        |s| s.nest = vec![pos(0, 1), pos(2, 0)],
        |s| s.waste = pos(4, 2),
        |s| s.waste = pos(0, 0),
        |s| {
            s.waste = pos(4, 2);
            s.open.push(s.waste)
        },
        |s| s.workers = vec![pos(0, 0); 3],
        |s| s.workers = vec![pos(2, 0)],
        |s| s.food.push(s.food[0].clone()),
        |s| {
            s.food.push(Resource {
                id: u64::MAX,
                pos: pos(4, 2),
            })
        },
        |s| {
            s.food.push(Resource {
                id: 0,
                pos: s.food[0].pos,
            })
        },
        |s| s.food[0].pos = pos(5, 0),
        |s| s.food[0].pos = s.waste,
        |s| s.food[0].pos = s.nest[0],
        |s| s.parameters.p_search = f64::NAN,
    ];
    for mutate in mutations {
        let mut s = setup();
        mutate(&mut s);
        assert!(s.validate().is_err(), "{s:?}");
    }
    let mut s = setup();
    s.open = vec![pos(2, 0), pos(0, 0), pos(2, 0), pos(1, 0), pos(0, 1)];
    let e = s.normalized().unwrap_err();
    assert!(e.iter().any(|e| e.field == "open[2]"));
}
#[test]
fn normalization_preserves_spawn_order_and_site_roundtrip() {
    let mut s = setup();
    s.workers = vec![pos(1, 0), pos(0, 0)];
    s.open.reverse();
    s.food.push(Resource {
        id: 0,
        pos: pos(4, 2),
    });
    let s = s.normalized().unwrap();
    assert_eq!(s.workers, vec![pos(1, 0), pos(0, 0)]);
    assert_eq!(s.food[0].id, 0);
    assert_eq!(s.position(14).unwrap(), pos(4, 2));
    assert_eq!(s.site(pos(4, 2)).unwrap(), 14);
    assert!(s.position(15).is_err());
    assert!(s.site(pos(5, 0)).is_err());
}
#[test]
fn terrain_dig_is_checked_and_atomic() {
    let mut t = Terrain::new(&setup()).unwrap();
    assert_eq!(t.dimensions(), (5, 3));
    assert_eq!(
        t.counts(),
        TerrainInventory {
            initial_open: 4,
            open: 4,
            excavated: 0
        }
    );
    for p in [pos(0, 0), pos(4, 0), pos(5, 0)] {
        let before = t.clone();
        assert!(t.dig(p).is_err());
        assert_eq!(t, before);
    }
    t.dig(pos(3, 0)).unwrap();
    assert!(t.was_excavated(pos(3, 0)).unwrap());
    assert!(!t.was_excavated(pos(0, 0)).unwrap());
    assert_eq!(t.counts().open, 5);
    t.check().unwrap();
    let before = t.clone();
    assert!(t.dig(pos(3, 0)).is_err());
    assert_eq!(t, before);
    assert_eq!(
        t.open_positions(),
        vec![pos(0, 0), pos(0, 1), pos(1, 0), pos(2, 0), pos(3, 0)]
    );
}
#[test]
fn remote_wall_memory_waits_for_local_revision() {
    let mut t = Terrain::new(&setup()).unwrap();
    let mut a = Knowledge::new(5, 3).unwrap();
    let mut b = a.clone();
    let o = observe(&t, pos(2, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    assert_eq!(a.learn(&o).unwrap().first, 4);
    b.learn(&o).unwrap();
    t.dig(pos(3, 0)).unwrap();
    assert_eq!(
        a.confirm_dig(pos(2, 0), pos(3, 0))
            .unwrap()
            .dig_confirmations,
        1
    );
    assert!(a.confirm_dig(pos(2, 0), pos(3, 0)).is_err());
    assert_eq!(
        b.kind(pos(3, 0)).unwrap(),
        CellKnowledge::KnownSolid { diggable: true }
    );
    let d = b
        .learn(&observe(&t, pos(2, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap())
        .unwrap();
    assert_eq!(
        (d.first, d.observed_revisions, d.dig_confirmations),
        (0, 1, 0)
    );
    assert_eq!(
        a.learn(&observe(&t, pos(2, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap())
            .unwrap(),
        LearnDelta::default()
    );
    assert_eq!(b.counts(), (3, 1, 0));
    assert_eq!(b.dimensions(), (5, 3));
    assert_eq!(b.known().len(), 4);
    assert!(b.known().windows(2).all(|w| w[0].pos < w[1].pos));
}
#[test]
fn observations_are_complete_local_ordered_and_occlude_food() {
    let t = Terrain::new(&setup()).unwrap();
    let o = observe(
        &t,
        pos(2, 0),
        &BTreeMap::new(),
        &BTreeSet::from([pos(3, 0)]),
    )
    .unwrap();
    assert_eq!(
        o.cells.iter().map(|c| c.pos).collect::<Vec<_>>(),
        vec![pos(2, 0), pos(2, 1), pos(3, 0), pos(1, 0)]
    );
    assert!(!o.cells[2].food);
    assert!(o.cells[2].diggable);
    for mutate in [
        |o: &mut Observation| {
            o.cells.pop();
        },
        |o: &mut Observation| o.cells[1].pos = pos(4, 2),
        |o: &mut Observation| o.cells[1].pos = o.origin,
        |o: &mut Observation| o.cells[0].diggable = true,
        |o: &mut Observation| o.cells[1].food = true,
        |o: &mut Observation| o.cells[0].occupants = 3,
        |o: &mut Observation| o.origin = pos(5, 0),
    ] {
        let mut bad = o.clone();
        mutate(&mut bad);
        assert!(bad.validate(5, 3).is_err());
    }
    assert!(observe(&t, pos(3, 0), &BTreeMap::new(), &BTreeSet::new()).is_err());
    assert!(observe(
        &t,
        pos(2, 0),
        &BTreeMap::from([(pos(2, 0), 3)]),
        &BTreeSet::new()
    )
    .is_err());
    assert_eq!(
        setup::neighbors(pos(2, 1), 5, 3),
        vec![pos(2, 0), pos(2, 2), pos(3, 1), pos(1, 1)]
    );
}
#[test]
fn late_contradictions_rollback_learning_and_dig_preconditions() {
    let t = Terrain::new(&setup()).unwrap();
    let mut k = Knowledge::new(5, 3).unwrap();
    let mut o = observe(&t, pos(2, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    k.learn(&o).unwrap();
    let before = k.clone();
    o.cells[2].open = true;
    o.cells[2].diggable = false;
    o.cells[3].open = false;
    assert!(k.learn(&o).is_err());
    assert_eq!(k, before);
    let mut late = observe(&t, pos(1, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    late.cells[2].open = false;
    assert!(k.learn(&late).is_err());
    assert_eq!(k, before);
    for target in [pos(4, 0), pos(2, 1), pos(1, 0)] {
        assert!(k.confirm_dig(pos(2, 0), target).is_err());
        assert_eq!(k, before);
    }
    assert!(k.confirm_dig(pos(3, 1), pos(3, 0)).is_err());
    o = observe(&t, pos(2, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    o.cells[2].diggable = false;
    assert!(k.learn(&o).is_err());
    o = observe(&t, pos(2, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    o.cells[1].open = true;
    assert!(k.learn(&o).is_err());
    assert!(Knowledge::new(2, 3).is_err());
    assert!(k.kind(pos(5, 0)).is_err());
}
#[test]
fn draws_enforce_half_open_domain_and_selection_cadence() {
    for value in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
        assert!(checked_uniform(&mut Scripted::new(&[value])).is_err());
        assert!(draw_index(&mut Scripted::new(&[value]), 2).is_err());
    }
    let mut d = Scripted::new(&[0.0, 0.999999]);
    assert_eq!(choose(&[], &mut d).unwrap(), None);
    assert_eq!(d.next, 0);
    assert_eq!(choose(&[pos(1, 0)], &mut d).unwrap(), Some(pos(1, 0)));
    assert_eq!(draw_index(&mut d, 2).unwrap(), 1);
    assert_eq!(d.next, 2);
    assert!(draw_index(&mut d, 0).is_err());
}
#[test]
fn compute_sums_are_atomic_and_queue_peaks_use_max() {
    let mut c = ComputeCounts {
        observations: 2,
        peak_queue: 7,
        ..Default::default()
    };
    c.checked_include(&ComputeCounts {
        observations: 3,
        peak_queue: 5,
        ..Default::default()
    })
    .unwrap();
    assert_eq!((c.observations, c.peak_queue), (5, 7));
    let before = c;
    c.face_scans = u64::MAX;
    let overflow = c;
    assert!(c
        .checked_include(&ComputeCounts {
            observations: 1,
            face_scans: 1,
            ..Default::default()
        })
        .is_err());
    assert_eq!(c, overflow);
    assert_eq!(before.observations, 5);
    let mut a = AccessCompute {
        calls: 2,
        visits: u64::MAX,
        peak_queue: 4,
    };
    let before = a;
    assert!(a
        .checked_include(&AccessCompute {
            calls: 1,
            visits: 1,
            peak_queue: 5
        })
        .is_err());
    assert_eq!(a, before);
    a = AccessCompute::default();
    a.checked_include(&AccessCompute {
        calls: 2,
        visits: 3,
        peak_queue: 4,
    })
    .unwrap();
    a.checked_include(&AccessCompute {
        calls: 2,
        visits: 3,
        peak_queue: 2,
    })
    .unwrap();
    assert_eq!(
        a,
        AccessCompute {
            calls: 4,
            visits: 6,
            peak_queue: 4
        }
    );
}

#[test]
fn dig_exposes_local_food_only_on_open_observations() {
    let mut t = Terrain::new(&setup()).unwrap();
    let food = BTreeSet::from([pos(3, 0), pos(4, 2)]);
    let occupancy = BTreeMap::from([(pos(2, 0), 2)]);
    let before = observe(&t, pos(2, 0), &occupancy, &food).unwrap();
    assert_eq!(before.cells[0].occupants, 2);
    assert!(!before.cells[2].food);
    t.dig(pos(3, 0)).unwrap();
    let after = observe(&t, pos(2, 0), &occupancy, &food).unwrap();
    assert!(after.cells[2].food);
    assert!(!after.cells[2].diggable);
    assert_eq!(after.cells.len(), 4);
}
#[test]
fn legal_grid_worker_food_limits_are_accepted() {
    let mut s = setup();
    s.width = 125;
    s.height = 125;
    s.nest = (0..128).map(|i| pos(i % 125, i / 125)).collect();
    s.open = s.nest.clone();
    s.waste = pos(3, 1);
    s.open.push(s.waste);
    s.workers = s.nest.iter().flat_map(|&p| [p, p]).collect();
    s.food = (0..256)
        .map(|i| Resource {
            id: u64::from(i),
            pos: pos(i % 125, 2 + i / 125),
        })
        .collect();
    s.validate().unwrap();
    let t = Terrain::new(&s).unwrap();
    assert_eq!(t.dimensions(), (125, 125));
    s = setup();
    s.width = 3;
    s.food.clear();
    s.diggable.clear();
    s.validate().unwrap();
}
