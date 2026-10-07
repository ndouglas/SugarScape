//! Fixtures learn only complete current/cardinal observations of real terrain.
use super::super::navigation::*;
use super::*;

fn fresh(s: &Setup, origin: Pos, occupants: &[(Pos, u32)]) -> Observation {
    observe(
        &Terrain::new(s).unwrap(),
        origin,
        &occupants.iter().copied().collect(),
        &BTreeSet::new(),
    )
    .unwrap()
}
fn learned(s: &Setup, observations: &[Pos]) -> Knowledge {
    let mut k = Knowledge::new(s.width, s.height).unwrap();
    for &origin in observations {
        k.learn(&fresh(s, origin, &[])).unwrap();
    }
    k
}
fn corridor() -> Setup {
    let mut s = setup();
    s.open = (0..5).map(|x| pos(x, 0)).collect();
    s.waste = pos(4, 0);
    s.diggable.clear();
    s.food.clear();
    s
}
fn cross() -> Setup {
    let mut s = setup();
    s.height = 5;
    s.open = vec![pos(2, 2), pos(2, 1), pos(2, 3), pos(3, 2), pos(1, 2)];
    s.nest = vec![pos(2, 2), pos(2, 1)];
    s.waste = pos(2, 3);
    s.workers = vec![pos(2, 2)];
    s.diggable = vec![pos(0, 2), pos(2, 0), pos(2, 4), pos(4, 2)];
    s.food.clear();
    s
}

#[test]
fn private_u_route_moves_away_from_goal_around_protected_wall() {
    let mut s = setup();
    s.open = vec![pos(1, 1), pos(1, 2), pos(2, 2), pos(3, 2), pos(3, 1)];
    s.nest = vec![pos(1, 1), pos(1, 2)];
    s.waste = pos(2, 2);
    s.workers = vec![pos(1, 1)];
    s.diggable.clear();
    s.food.clear();
    let k = learned(&s, &s.open);
    let mut d = Scripted::new(&[0.4]);
    let (step, c) = route_step(
        &k,
        pos(1, 1),
        &[pos(3, 1)],
        &fresh(&s, pos(1, 1), &[]),
        &mut d,
    )
    .unwrap();
    assert_eq!(step, Navigation::Move(pos(1, 2)));
    assert_eq!(
        (c.route_calls, c.route_visits, c.peak_queue, d.next),
        (1, 5, 1, 1)
    );
}

#[test]
fn nearest_home_ties_use_cardinal_order_and_boundary_draws() {
    let s = corridor();
    let k = learned(&s, &s.open);
    for (draw, next) in [
        (0.0, pos(3, 0)),
        (0.499, pos(3, 0)),
        (0.5, pos(1, 0)),
        (0.99, pos(1, 0)),
    ] {
        let mut d = Scripted::new(&[draw]);
        let (step, c) = route_step(
            &k,
            pos(2, 0),
            &[pos(0, 0), pos(4, 0), pos(0, 0)],
            &fresh(&s, pos(2, 0), &[]),
            &mut d,
        )
        .unwrap();
        assert_eq!(step, Navigation::Move(next));
        assert_eq!((c.route_visits, c.peak_queue, d.next), (5, 2, 1));
    }
}

#[test]
fn full_nearest_steps_wait_instead_of_longer_escape() {
    let s = corridor();
    let k = learned(&s, &s.open);
    let mut d = Scripted::new(&[]);
    assert_eq!(
        route_step(
            &k,
            pos(2, 0),
            &[pos(1, 0), pos(4, 0)],
            &fresh(&s, pos(2, 0), &[(pos(1, 0), 2)]),
            &mut d
        )
        .unwrap()
        .0,
        Navigation::Blocked
    );
    assert_eq!(d.next, 0);
    let mut d = Scripted::new(&[0.8]);
    assert_eq!(
        route_step(
            &k,
            pos(2, 0),
            &[pos(1, 0), pos(4, 0)],
            &fresh(&s, pos(2, 0), &[(pos(1, 0), 1)]),
            &mut d
        )
        .unwrap()
        .0,
        Navigation::Move(pos(1, 0))
    );
}

#[test]
fn free_equal_shortest_step_is_selected_when_other_is_full() {
    let s = corridor();
    let k = learned(&s, &s.open);
    let mut d = Scripted::new(&[0.9]);
    assert_eq!(
        route_step(
            &k,
            pos(2, 0),
            &[pos(0, 0), pos(4, 0)],
            &fresh(&s, pos(2, 0), &[(pos(3, 0), 2)]),
            &mut d
        )
        .unwrap()
        .0,
        Navigation::Move(pos(1, 0))
    );
    assert_eq!(d.next, 1);
}

#[test]
fn remote_occupancy_and_food_do_not_change_routes_or_draws() {
    let s = corridor();
    let t = Terrain::new(&s).unwrap();
    let k = learned(&s, &s.open);
    for occupants in [0, 2] {
        let o = observe(
            &t,
            pos(0, 0),
            &BTreeMap::from([(pos(3, 0), occupants)]),
            &BTreeSet::from([pos(4, 0)]),
        )
        .unwrap();
        let mut d = Scripted::new(&[0.7]);
        assert_eq!(
            route_step(&k, pos(0, 0), &[pos(4, 0)], &o, &mut d)
                .unwrap()
                .0,
            Navigation::Move(pos(1, 0))
        );
        assert_eq!(d.next, 1);
    }
}

#[test]
fn goal_equality_and_absent_open_goals_use_no_draws() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0), pos(1, 0)]);
    let o = fresh(&s, pos(0, 0), &[]);
    let mut d = Scripted::new(&[]);
    assert_eq!(
        route_step(&k, pos(0, 0), &[pos(0, 0)], &o, &mut d).unwrap(),
        (Navigation::AtGoal, ComputeCounts::default())
    );
    for goals in [
        vec![],
        vec![pos(4, 2)],
        vec![pos(1, 1)],
        vec![pos(4, 2), pos(1, 1)],
    ] {
        let (step, c) = route_step(&k, pos(0, 0), &goals, &o, &mut d).unwrap();
        assert_eq!(step, Navigation::Unreachable);
        assert_eq!((c.route_calls, c.route_visits, c.peak_queue), (1, 0, 0));
    }
    assert_eq!(d.next, 0);
}

#[test]
fn privately_disconnected_open_goal_remains_unreachable() {
    let mut s = setup();
    s.open.push(pos(4, 2));
    let k = learned(&s, &[pos(0, 0), pos(4, 2)]);
    let mut d = Scripted::new(&[]);
    let (step, c) = route_step(
        &k,
        pos(0, 0),
        &[pos(4, 2)],
        &fresh(&s, pos(0, 0), &[]),
        &mut d,
    )
    .unwrap();
    assert_eq!(
        (step, c.route_visits, c.peak_queue, d.next),
        (Navigation::Unreachable, 1, 1, 0)
    );
}

#[test]
fn frontiers_include_observed_unvisited_open_cells_with_sorted_unknowns() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let (f, c) = frontiers(&k, pos(0, 0)).unwrap();
    assert_eq!(
        f,
        vec![
            Frontier {
                pos: pos(0, 1),
                unknown_neighbors: vec![pos(0, 2), pos(1, 1)],
                distance: 1
            },
            Frontier {
                pos: pos(1, 0),
                unknown_neighbors: vec![pos(1, 1), pos(2, 0)],
                distance: 1
            }
        ]
    );
    assert_eq!(
        (
            c.route_calls,
            c.route_visits,
            c.peak_queue,
            c.frontier_scans
        ),
        (1, 3, 2, 3)
    );
}

#[test]
fn frontiers_exclude_privately_disconnected_open_cells() {
    let mut s = setup();
    s.open.push(pos(4, 2));
    let k = learned(&s, &[pos(0, 0), pos(4, 2)]);
    assert_eq!(
        frontiers(&k, pos(0, 0))
            .unwrap()
            .0
            .iter()
            .map(|f| f.pos)
            .collect::<Vec<_>>(),
        vec![pos(0, 1), pos(1, 0)]
    );
}

#[test]
fn uninformed_frontier_choices_use_sorted_positions_and_exact_boundaries() {
    let s = cross();
    let k = learned(&s, &[pos(2, 2)]);
    for (draw, next) in [
        (0.0, pos(1, 2)),
        (0.25, pos(2, 1)),
        (0.5, pos(2, 3)),
        (0.75, pos(3, 2)),
    ] {
        let mut d = Scripted::new(&[draw]);
        assert_eq!(
            select_frontier(&k, pos(2, 2), None, &mut d).unwrap().0,
            Some(next)
        );
        assert_eq!(d.next, 1);
    }
}

#[test]
fn informed_frontiers_rank_unknown_neighbors_before_known_distance() {
    let s = cross();
    let k = learned(&s, &[pos(2, 2)]);
    for (origin, site, draw, next) in [
        (pos(2, 1), pos(4, 2), 0.1, pos(3, 2)),
        (pos(2, 1), pos(2, 2), 0.9, pos(2, 1)),
        (pos(2, 2), pos(2, 2), 0.5, pos(2, 3)),
    ] {
        let mut d = Scripted::new(&[draw]);
        assert_eq!(
            select_frontier(&k, origin, Some(site), &mut d).unwrap().0,
            Some(next)
        );
    }
}

#[test]
fn unknown_outlet_uses_private_open_frontier_then_known_route() {
    let s = corridor();
    let mut k = learned(&s, &[pos(0, 0), pos(1, 0)]);
    let mut d = Scripted::new(&[0.9]);
    assert_eq!(k.kind(s.waste).unwrap(), CellKnowledge::Unknown);
    assert_eq!(
        select_frontier(&k, pos(0, 0), Some(s.waste), &mut d)
            .unwrap()
            .0,
        Some(pos(2, 0))
    );
    assert_eq!(d.next, 1);
    k.learn(&fresh(&s, pos(2, 0), &[])).unwrap();
    k.learn(&fresh(&s, pos(3, 0), &[])).unwrap();
    let mut d = Scripted::new(&[0.3]);
    assert_eq!(
        route_step(
            &k,
            pos(3, 0),
            &[s.waste],
            &fresh(&s, pos(3, 0), &[]),
            &mut d
        )
        .unwrap()
        .0,
        Navigation::Move(pos(4, 0))
    );
}

#[test]
fn informed_selection_keeps_nonimproving_frontiers() {
    let mut s = setup();
    s.open = vec![pos(1, 1), pos(1, 2), pos(2, 2)];
    s.nest = vec![pos(1, 1), pos(1, 2)];
    s.waste = pos(2, 2);
    s.workers = vec![pos(1, 1)];
    s.food.clear();
    s.diggable.clear();
    let k = learned(&s, &[pos(1, 1)]);
    let mut d = Scripted::new(&[0.8]);
    assert_eq!(
        select_frontier(&k, pos(1, 1), Some(pos(3, 1)), &mut d)
            .unwrap()
            .0,
        Some(pos(1, 2))
    );
}

#[test]
fn complete_corridor_has_no_frontier_and_no_selection_draw() {
    let s = corridor();
    let k = learned(&s, &s.open);
    let (f, c) = frontiers(&k, pos(0, 0)).unwrap();
    assert!(f.is_empty());
    assert_eq!((c.route_visits, c.peak_queue, c.frontier_scans), (5, 1, 5));
    let mut d = Scripted::new(&[]);
    assert_eq!(
        select_frontier(&k, pos(0, 0), Some(s.waste), &mut d)
            .unwrap()
            .0,
        None
    );
    assert_eq!(d.next, 0);
}

#[test]
fn only_locally_known_diggable_faces_are_candidates() {
    let s = setup();
    let k = learned(&s, &[pos(2, 0)]);
    let (f, c) = faces(&k, pos(2, 0)).unwrap();
    assert_eq!(
        f,
        vec![Face {
            pos: pos(3, 0),
            approaches: vec![pos(2, 0)],
            distance: 0
        }]
    );
    assert_eq!(
        (c.route_calls, c.route_visits, c.peak_queue, c.face_scans),
        (1, 2, 1, 2)
    );
}

#[test]
fn faces_with_only_disconnected_approaches_are_excluded() {
    let mut s = setup();
    s.open.push(pos(4, 2));
    s.diggable.push(pos(3, 2));
    let k = learned(&s, &[pos(0, 0), pos(4, 2)]);
    let (f, c) = faces(&k, pos(0, 0)).unwrap();
    assert!(f.is_empty());
    assert_eq!(c.face_scans, 2);
    assert!(face_approaches(&k, pos(0, 0), pos(3, 2))
        .unwrap()
        .0
        .is_empty());
}

#[test]
fn face_approaches_are_sorted_reachable_and_nearest_distance_is_reported() {
    let mut s = setup();
    s.open.extend([pos(2, 1), pos(3, 1)]);
    s.diggable = vec![pos(3, 0)];
    let k = learned(&s, &s.open);
    let (f, _) = faces(&k, pos(0, 0)).unwrap();
    assert_eq!(
        f,
        vec![Face {
            pos: pos(3, 0),
            approaches: vec![pos(2, 0), pos(3, 1)],
            distance: 2
        }]
    );
    assert_eq!(
        face_approaches(&k, pos(0, 0), pos(3, 0)).unwrap().0,
        vec![pos(2, 0), pos(3, 1)]
    );
}

#[test]
fn reaching_a_face_approach_is_at_goal_without_draw_or_dig() {
    let s = setup();
    let k = learned(&s, &[pos(2, 0)]);
    let (approaches, _) = face_approaches(&k, pos(2, 0), pos(3, 0)).unwrap();
    let mut d = Scripted::new(&[]);
    assert_eq!(
        route_step(
            &k,
            pos(2, 0),
            &approaches,
            &fresh(&s, pos(2, 0), &[]),
            &mut d
        )
        .unwrap()
        .0,
        Navigation::AtGoal
    );
    assert_eq!(d.next, 0);
}

#[test]
fn unknown_open_and_protected_cells_are_not_face_approaches() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0), pos(1, 0)]);
    for face in [pos(4, 2), pos(1, 0), pos(1, 1)] {
        assert!(face_approaches(&k, pos(0, 0), face).unwrap().0.is_empty());
    }
    let mut d = Scripted::new(&[]);
    assert_eq!(select_face(&k, pos(0, 0), None, &mut d).unwrap().0, None);
    assert_eq!(d.next, 0);
}

#[test]
fn uninformed_faces_use_stable_coordinate_order_and_boundary_draws() {
    let s = cross();
    let k = learned(&s, &s.open);
    for (draw, next) in [
        (0.0, pos(0, 2)),
        (0.25, pos(2, 0)),
        (0.5, pos(2, 4)),
        (0.75, pos(4, 2)),
    ] {
        let mut d = Scripted::new(&[draw]);
        assert_eq!(
            select_face(&k, pos(2, 2), None, &mut d).unwrap().0,
            Some(next)
        );
        assert_eq!(d.next, 1);
    }
}

#[test]
fn informed_faces_rank_site_proximity_before_nearest_approach_distance() {
    let s = cross();
    let k = learned(&s, &s.open);
    for (site, next) in [(pos(4, 2), pos(4, 2)), (pos(2, 2), pos(2, 0))] {
        let mut d = Scripted::new(&[0.9]);
        assert_eq!(
            select_face(&k, pos(2, 1), Some(site), &mut d).unwrap().0,
            Some(next)
        );
    }
    let mut d = Scripted::new(&[0.5]);
    assert_eq!(
        select_face(&k, pos(2, 2), Some(pos(2, 2)), &mut d)
            .unwrap()
            .0,
        Some(pos(2, 4))
    );
}

#[test]
fn informed_faces_keep_nonimproving_candidates_and_singletons_draw_once() {
    let s = setup();
    let k = learned(&s, &[pos(2, 0)]);
    let mut d = Scripted::new(&[0.8]);
    assert_eq!(
        select_face(&k, pos(2, 0), Some(pos(1, 0)), &mut d)
            .unwrap()
            .0,
        Some(pos(3, 0))
    );
    assert_eq!(d.next, 1);
}

#[test]
fn remote_opening_preserves_stale_face_until_local_revision() {
    let s = setup();
    let mut t = Terrain::new(&s).unwrap();
    let mut k = learned(&s, &[pos(2, 0)]);
    let old = k.clone();
    t.dig(pos(3, 0)).unwrap();
    let (f, _) = faces(&k, pos(2, 0)).unwrap();
    assert_eq!(f.iter().map(|f| f.pos).collect::<Vec<_>>(), vec![pos(3, 0)]);
    let o = observe(&t, pos(2, 0), &BTreeMap::new(), &BTreeSet::new()).unwrap();
    let mut d = Scripted::new(&[]);
    // Current physical openness cannot become a route until the private map learns it.
    assert_eq!(
        route_step(&k, pos(2, 0), &[pos(3, 0)], &o, &mut d)
            .unwrap()
            .0,
        Navigation::Unreachable
    );
    assert_eq!(k, old);
    assert_eq!(k.learn(&o).unwrap().observed_revisions, 1);
    assert!(faces(&k, pos(2, 0)).unwrap().0.is_empty());
    assert_eq!(
        frontiers(&k, pos(2, 0))
            .unwrap()
            .0
            .iter()
            .map(|f| f.pos)
            .collect::<Vec<_>>(),
        vec![pos(1, 0), pos(3, 0)]
    );
    let mut d = Scripted::new(&[0.2]);
    assert_eq!(
        route_step(&k, pos(2, 0), &[pos(3, 0)], &o, &mut d)
            .unwrap()
            .0,
        Navigation::Move(pos(3, 0))
    );
}

#[test]
fn wander_uses_fresh_cardinal_capacity_and_draws_only_for_nonempty_steps() {
    let s = corridor();
    let mut d = Scripted::new(&[0.5]);
    assert_eq!(
        wander(pos(2, 0), &fresh(&s, pos(2, 0), &[]), &mut d).unwrap(),
        Navigation::Move(pos(1, 0))
    );
    assert_eq!(d.next, 1);
    let mut d = Scripted::new(&[]);
    assert_eq!(
        wander(
            pos(2, 0),
            &fresh(&s, pos(2, 0), &[(pos(1, 0), 2), (pos(3, 0), 2)]),
            &mut d
        )
        .unwrap(),
        Navigation::Blocked
    );
    assert_eq!(d.next, 0);
    // A complete actual-width boundary view must not be rejected as incomplete at width 125.
    let mut d = Scripted::new(&[0.3]);
    assert_eq!(
        wander(pos(4, 0), &fresh(&s, pos(4, 0), &[]), &mut d).unwrap(),
        Navigation::Move(pos(3, 0))
    );
}

#[test]
fn malformed_route_observations_are_rejected_before_goal_equality_or_draws() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let o = fresh(&s, pos(0, 0), &[]);
    let mut d = Scripted::new(&[]);
    let mut duplicate = o.clone();
    duplicate.cells.push(duplicate.cells[0].clone());
    let mut missing = o.clone();
    missing.cells.pop();
    let mut diagonal = o.clone();
    diagonal.cells[1].pos = pos(1, 1);
    let mut overflow = o.clone();
    overflow.cells[1].occupants = 3;
    let mut bad_open = o.clone();
    bad_open.cells[1].diggable = true;
    let mut closed_origin = o.clone();
    closed_origin.cells[0].open = false;
    let mut remote = o.clone();
    remote.cells[1].pos = pos(4, 2);
    for bad in [
        duplicate,
        missing,
        diagonal,
        overflow,
        bad_open,
        closed_origin,
        remote,
    ] {
        assert!(route_step(&k, pos(0, 0), &[pos(0, 0)], &bad, &mut d).is_err());
    }
    assert!(route_step(&k, pos(1, 0), &[pos(1, 0)], &o, &mut d).is_err());
    assert_eq!(d.next, 0);
}

#[test]
fn locally_impossible_classifications_are_rejected_before_route_draws() {
    let s = setup();
    let k = learned(&s, &[pos(2, 0)]);
    let o = fresh(&s, pos(2, 0), &[]);
    let mut d = Scripted::new(&[]);
    for target in [pos(1, 0), pos(2, 1), pos(3, 0)] {
        let mut bad = o.clone();
        let cell = bad.cells.iter_mut().find(|c| c.pos == target).unwrap();
        match target {
            p if p == pos(1, 0) => cell.open = false,
            p if p == pos(2, 1) => cell.open = true,
            _ => cell.diggable = false,
        }
        assert!(route_step(&k, pos(2, 0), &[pos(2, 0)], &bad, &mut d).is_err());
    }
    assert_eq!(d.next, 0);
}

#[test]
fn wander_checks_locally_enforceable_payloads_before_draw() {
    let s = setup();
    let o = fresh(&s, pos(0, 0), &[]);
    let mut d = Scripted::new(&[]);
    let mut duplicate = o.clone();
    duplicate.cells.push(duplicate.cells[0].clone());
    let mut diagonal = o.clone();
    diagonal.cells[1].pos = pos(1, 1);
    let mut overflow = o.clone();
    overflow.cells[1].occupants = 3;
    let mut closed_origin = o.clone();
    closed_origin.cells[0].open = false;
    let mut bad_solid = fresh(&s, pos(2, 0), &[]);
    bad_solid
        .cells
        .iter_mut()
        .find(|c| c.pos == pos(2, 1))
        .unwrap()
        .food = true;
    let mut bad_diggable = o.clone();
    bad_diggable.cells[1].diggable = true;
    for bad in [duplicate, diagonal, overflow, closed_origin, bad_diggable] {
        assert!(wander(pos(0, 0), &bad, &mut d).is_err());
    }
    assert!(wander(pos(2, 0), &bad_solid, &mut d).is_err());
    assert!(wander(pos(1, 0), &o, &mut d).is_err());
    assert_eq!(d.next, 0);
}

#[test]
fn invalid_origins_sites_faces_and_goals_fail_before_draws() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let o = fresh(&s, pos(0, 0), &[]);
    let mut d = Scripted::new(&[]);
    for origin in [pos(5, 0), pos(1, 1), pos(4, 2)] {
        assert!(frontiers(&k, origin).is_err());
        assert!(faces(&k, origin).is_err());
        assert!(face_approaches(&k, origin, pos(3, 0)).is_err());
    }
    assert!(select_frontier(&k, pos(0, 0), Some(pos(5, 0)), &mut d).is_err());
    assert!(select_face(&k, pos(0, 0), Some(pos(5, 0)), &mut d).is_err());
    assert!(face_approaches(&k, pos(0, 0), pos(5, 0)).is_err());
    assert!(route_step(&k, pos(0, 0), &[pos(0, 0), pos(5, 0)], &o, &mut d).is_err());
    assert_eq!(d.next, 0);
}

#[test]
fn malicious_draws_are_rejected_for_all_nonempty_selections() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0), pos(2, 0)]);
    let o = fresh(&s, pos(0, 0), &[]);
    for value in [f64::NAN, -0.1, 1.0, f64::INFINITY] {
        assert!(select_frontier(&k, pos(0, 0), None, &mut Scripted::new(&[value])).is_err());
        assert!(select_face(&k, pos(0, 0), None, &mut Scripted::new(&[value])).is_err());
        assert!(route_step(
            &k,
            pos(0, 0),
            &[pos(1, 0)],
            &o,
            &mut Scripted::new(&[value])
        )
        .is_err());
        assert!(wander(pos(0, 0), &o, &mut Scripted::new(&[value])).is_err());
    }
}

#[test]
fn bounded_grid_counts_dequeues_and_unique_queue_peak() {
    let mut s = setup();
    s.height = 5;
    s.open = (0..5)
        .flat_map(|x| (0..5).map(move |y| pos(x, y)))
        .collect();
    s.diggable.clear();
    s.food.clear();
    let k = learned(&s, &s.open);
    let (_, c) = frontiers(&k, pos(2, 2)).unwrap();
    assert_eq!(
        (
            c.route_calls,
            c.route_visits,
            c.peak_queue,
            c.frontier_scans
        ),
        (1, 25, 10, 25)
    );
    let (f, c) = faces(&k, pos(2, 2)).unwrap();
    assert!(f.is_empty());
    assert_eq!(
        (c.route_calls, c.route_visits, c.peak_queue, c.face_scans),
        (1, 25, 10, 0)
    );
}
