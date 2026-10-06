use super::super::navigation::*;
use super::*;

fn learned(s: &Setup, visited: &[Pos]) -> Knowledge {
    s.validate().unwrap();
    let mut k = Knowledge::new(s.width, s.height).unwrap();
    for &p in visited {
        k.learn(&observe(s, p, &BTreeMap::new(), &BTreeSet::new()).unwrap())
            .unwrap();
    }
    k
}
fn fresh(s: &Setup, p: Pos, occupied: &[(Pos, u32)]) -> Observation {
    observe(s, p, &occupied.iter().copied().collect(), &BTreeSet::new()).unwrap()
}
fn corridor() -> Setup {
    let mut s = setup();
    s.open = (0..5).map(|x| pos(x, 0)).collect();
    s
}

#[test]
fn known_route_accepts_a_step_away_from_the_destination() {
    let mut s = setup();
    s.open = vec![pos(1, 1), pos(1, 2), pos(2, 2), pos(3, 2), pos(3, 1)];
    s.nest = vec![pos(1, 1), pos(1, 2)];
    s.workers = vec![pos(1, 1)];
    let k = learned(&s, &s.open);
    let o = fresh(&s, pos(1, 1), &[]);
    let mut d = Scripted::new(&[0.5]);
    let (step, _) = route_step(&k, pos(1, 1), &[pos(3, 1)], &o, &mut d).unwrap();
    assert_eq!(step, Navigation::Move(pos(1, 2)));
    assert_eq!(d.next, 1);
}

#[test]
fn route_choices_follow_cardinal_order_at_half_open_boundary() {
    let mut s = setup();
    s.open = vec![
        pos(1, 1),
        pos(1, 0),
        pos(1, 2),
        pos(2, 0),
        pos(2, 2),
        pos(3, 0),
        pos(3, 2),
        pos(3, 1),
    ];
    s.nest = vec![pos(1, 0), pos(1, 1)];
    s.workers = vec![pos(1, 1)];
    let k = learned(&s, &s.open);
    let o = fresh(&s, pos(1, 1), &[]);
    for (draw, expected) in [
        (0.0, pos(1, 0)),
        (0.5 - f64::EPSILON, pos(1, 0)),
        (0.5, pos(1, 2)),
        (0.99, pos(1, 2)),
    ] {
        let mut d = Scripted::new(&[draw]);
        assert_eq!(
            route_step(&k, pos(1, 1), &[pos(3, 1)], &o, &mut d)
                .unwrap()
                .0,
            Navigation::Move(expected)
        );
        assert_eq!(d.next, 1);
    }
}

#[test]
fn current_goal_returns_without_bfs_or_draw() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let mut d = Scripted::new(&[]);
    let (step, c) = route_step(
        &k,
        pos(0, 0),
        &[pos(0, 0), pos(3, 1)],
        &fresh(&s, pos(0, 0), &[]),
        &mut d,
    )
    .unwrap();
    assert_eq!(step, Navigation::AtGoal);
    assert_eq!(c, ComputeCounts::default());
    assert_eq!(d.next, 0);
}

#[test]
fn unknown_solid_and_disconnected_goals_are_unreachable_without_draw() {
    let mut s = setup();
    s.open.push(pos(4, 4));
    let k = learned(&s, &[pos(0, 0), pos(4, 4)]);
    for goals in [vec![], vec![pos(3, 1)], vec![pos(0, 1)], vec![pos(4, 4)]] {
        let mut d = Scripted::new(&[]);
        assert_eq!(
            route_step(&k, pos(0, 0), &goals, &fresh(&s, pos(0, 0), &[]), &mut d)
                .unwrap()
                .0,
            Navigation::Unreachable
        );
        assert_eq!(d.next, 0);
    }
}

#[test]
fn all_equal_nearest_home_destinations_contribute_steps() {
    let s = corridor();
    let k = learned(&s, &s.open);
    let o = fresh(&s, pos(2, 0), &[]);
    // Cardinal order E,W: both home goals are equally near.
    for (draw, next) in [(0.0, pos(3, 0)), (0.5, pos(1, 0))] {
        let mut d = Scripted::new(&[draw]);
        assert_eq!(
            route_step(
                &k,
                pos(2, 0),
                &[pos(0, 0), pos(4, 0), pos(0, 0)],
                &o,
                &mut d
            )
            .unwrap()
            .0,
            Navigation::Move(next)
        );
    }
}

#[test]
fn full_nearest_route_blocks_without_hidden_longer_home_detour() {
    let s = corridor();
    let k = learned(&s, &s.open);
    let mut d = Scripted::new(&[]);
    let goals = [pos(1, 0), pos(4, 0)];
    assert_eq!(
        route_step(
            &k,
            pos(2, 0),
            &goals,
            &fresh(&s, pos(2, 0), &[(pos(1, 0), 2)]),
            &mut d
        )
        .unwrap()
        .0,
        Navigation::Blocked
    );
    assert_eq!(d.next, 0);
    // The same retained destination resumes when the immediate cell has capacity.
    let mut d = Scripted::new(&[0.8]);
    assert_eq!(
        route_step(
            &k,
            pos(2, 0),
            &goals,
            &fresh(&s, pos(2, 0), &[(pos(1, 0), 1)]),
            &mut d
        )
        .unwrap()
        .0,
        Navigation::Move(pos(1, 0))
    );
}

#[test]
fn remote_occupancy_and_food_cannot_affect_route_or_draws() {
    let s = corridor();
    let k = learned(&s, &s.open);
    for remote in [0, 2] {
        let mut d = Scripted::new(&[0.7]);
        let o = observe(
            &s,
            pos(0, 0),
            &BTreeMap::from([(pos(3, 0), remote)]),
            &BTreeSet::from([pos(4, 0)]),
        )
        .unwrap();
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
fn literal_corridor_reports_executed_bfs_and_frontier_scans() {
    let s = corridor();
    let k = learned(&s, &s.open);
    let mut d = Scripted::new(&[0.2]);
    let (_, c) = route_step(
        &k,
        pos(0, 0),
        &[pos(4, 0)],
        &fresh(&s, pos(0, 0), &[]),
        &mut d,
    )
    .unwrap();
    assert_eq!(
        (
            c.route_calls,
            c.route_visits,
            c.peak_queue,
            c.frontier_scans
        ),
        (1, 5, 1, 0)
    );
    let (f, c) = frontiers(&k, pos(0, 0)).unwrap();
    assert!(f.is_empty());
    assert_eq!(
        (
            c.route_calls,
            c.route_visits,
            c.peak_queue,
            c.frontier_scans
        ),
        (1, 5, 1, 5)
    );
    let mut d = Scripted::new(&[]);
    assert_eq!(
        select_frontier(&k, pos(0, 0), None, &mut d).unwrap().0,
        None
    );
    assert_eq!(d.next, 0);
}

#[test]
fn observed_nonvisited_open_cell_can_be_a_frontier() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let (f, c) = frontiers(&k, pos(0, 0)).unwrap();
    assert_eq!(
        f,
        vec![Frontier {
            pos: pos(1, 0),
            unknown_neighbors: vec![pos(1, 1), pos(2, 0)],
            distance: 1
        }]
    );
    assert_eq!(c.frontier_scans, 2);
    let mut d = Scripted::new(&[0.9]);
    assert_eq!(
        select_frontier(&k, pos(0, 0), None, &mut d).unwrap().0,
        Some(pos(1, 0))
    );
    assert_eq!(d.next, 1);
}

#[test]
fn frontiers_exclude_privately_disconnected_open_cells() {
    let mut s = setup();
    s.open.push(pos(4, 4));
    let k = learned(&s, &[pos(0, 0), pos(4, 4)]);
    assert_eq!(
        frontiers(&k, pos(0, 0))
            .unwrap()
            .0
            .iter()
            .map(|f| f.pos)
            .collect::<Vec<_>>(),
        vec![pos(1, 0)]
    );
}

#[test]
fn uninformed_frontier_ties_use_sorted_positions_and_exact_boundary() {
    let mut s = setup();
    s.open = vec![pos(2, 2), pos(2, 1), pos(2, 3), pos(3, 2), pos(1, 2)];
    s.nest = vec![pos(2, 2), pos(2, 1)];
    s.workers = vec![pos(2, 2)];
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
    }
}

#[test]
fn informed_frontier_uses_unknown_neighbors_then_route_distance() {
    let s = corridor();
    let k = learned(&s, &[pos(0, 0), pos(1, 0), pos(2, 0)]);
    // Only (3,0) borders unknown cells; (4,0) is the closest unknown neighbor.
    let mut d = Scripted::new(&[0.1]);
    assert_eq!(
        select_frontier(&k, pos(0, 0), Some(pos(4, 0)), &mut d)
            .unwrap()
            .0,
        Some(pos(3, 0))
    );
    // Cross branches tie on unknown-neighbor distance to center; nearer frontier wins.
    let mut s = setup();
    s.open = vec![pos(2, 2), pos(2, 1), pos(2, 3), pos(3, 2), pos(1, 2)];
    s.nest = vec![pos(2, 2), pos(2, 1)];
    s.workers = vec![pos(2, 2)];
    let k = learned(&s, &[pos(2, 2)]);
    // Unknown-neighbor proximity wins even though the selected frontier is
    // farther from this origin than the origin frontier itself.
    let mut d = Scripted::new(&[0.1]);
    assert_eq!(
        select_frontier(&k, pos(2, 1), Some(pos(4, 2)), &mut d)
            .unwrap()
            .0,
        Some(pos(3, 2))
    );
    let mut d = Scripted::new(&[0.9]);
    assert_eq!(
        select_frontier(&k, pos(2, 1), Some(pos(2, 2)), &mut d)
            .unwrap()
            .0,
        Some(pos(2, 1))
    );
    let mut d = Scripted::new(&[0.5]);
    assert_eq!(
        select_frontier(&k, pos(2, 2), Some(pos(2, 2)), &mut d)
            .unwrap()
            .0,
        Some(pos(2, 3))
    );
}

#[test]
fn informed_exploration_keeps_nonimproving_frontier_eligible() {
    let mut s = setup();
    s.open = vec![pos(1, 1), pos(1, 2), pos(2, 2)];
    s.nest = vec![pos(1, 1), pos(1, 2)];
    s.workers = vec![pos(1, 1)];
    let k = learned(&s, &[pos(1, 1)]);
    let mut d = Scripted::new(&[0.8]);
    let target = select_frontier(&k, pos(1, 1), Some(pos(3, 1)), &mut d)
        .unwrap()
        .0
        .unwrap();
    assert_eq!(target, pos(1, 2));
    let mut d = Scripted::new(&[0.2]);
    assert_eq!(
        route_step(&k, pos(1, 1), &[target], &fresh(&s, pos(1, 1), &[]), &mut d)
            .unwrap()
            .0,
        Navigation::Move(pos(1, 2))
    );
}

#[test]
fn wander_uses_fresh_capacity_in_cardinal_order_and_empty_uses_no_draw() {
    let s = corridor();
    let mut d = Scripted::new(&[0.5]);
    assert_eq!(
        wander(pos(2, 0), &fresh(&s, pos(2, 0), &[]), &mut d).unwrap(),
        Navigation::Move(pos(1, 0))
    );
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
}

#[test]
fn malformed_navigation_inputs_fail_before_draw() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let o = fresh(&s, pos(0, 0), &[]);
    let mut d = Scripted::new(&[]);
    assert!(frontiers(&k, pos(5, 0)).is_err());
    assert!(frontiers(&k, pos(0, 1)).is_err());
    assert!(select_frontier(&k, pos(0, 0), Some(pos(5, 0)), &mut d).is_err());
    assert!(route_step(&k, pos(0, 0), &[pos(5, 0)], &o, &mut d).is_err());
    assert!(route_step(&k, pos(1, 0), &[pos(1, 0)], &o, &mut d).is_err());
    let mut bad = o.clone();
    bad.cells.push(bad.cells[0].clone());
    assert!(route_step(&k, pos(0, 0), &[pos(0, 0)], &bad, &mut d).is_err());
    assert!(wander(pos(0, 0), &bad, &mut d).is_err());
    assert!(wander(pos(1, 0), &o, &mut d).is_err());
    let mut bad = o.clone();
    bad.cells[1].occupants = 3;
    assert!(wander(pos(0, 0), &bad, &mut d).is_err());
    let mut bad = o.clone();
    bad.cells.pop();
    assert!(route_step(&k, pos(0, 0), &[pos(1, 0)], &bad, &mut d).is_err());
    assert_eq!(d.next, 0);
}

#[test]
fn malicious_selection_draws_are_checked() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let o = fresh(&s, pos(0, 0), &[]);
    for value in [f64::NAN, -0.1, 1.0, f64::INFINITY] {
        assert!(select_frontier(&k, pos(0, 0), None, &mut Scripted::new(&[value])).is_err());
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
fn bfs_scratch_queue_stays_within_literal_grid_bound() {
    let mut s = setup();
    s.open = (0..5)
        .flat_map(|x| (0..5).map(move |y| pos(x, y)))
        .collect();
    let k = learned(&s, &s.open);
    let (_, c) = frontiers(&k, pos(2, 2)).unwrap();
    assert_eq!(c.route_visits, 25);
    assert!(c.peak_queue <= 25);
    assert_eq!(c.frontier_scans, 25);
}

#[test]
fn an_unfilled_equal_shortest_step_remains_eligible() {
    let s = corridor();
    let k = learned(&s, &s.open);
    let o = fresh(&s, pos(2, 0), &[(pos(3, 0), 2)]);
    let mut d = Scripted::new(&[0.9]);
    assert_eq!(
        route_step(&k, pos(2, 0), &[pos(0, 0), pos(4, 0)], &o, &mut d)
            .unwrap()
            .0,
        Navigation::Move(pos(1, 0))
    );
    assert_eq!(d.next, 1);
}

#[test]
fn observations_reject_nonlocal_and_conflicting_cells_before_route_draws() {
    let s = setup();
    let k = learned(&s, &[pos(0, 0)]);
    let o = fresh(&s, pos(0, 0), &[]);
    let mut d = Scripted::new(&[]);
    let mut diagonal = o.clone();
    diagonal.cells[1].pos = pos(1, 1);
    assert!(wander(pos(0, 0), &diagonal, &mut d).is_err());
    assert!(route_step(&k, pos(0, 0), &[pos(1, 0)], &diagonal, &mut d).is_err());
    let mut conflicting = o;
    let east = conflicting
        .cells
        .iter_mut()
        .find(|c| c.pos == pos(1, 0))
        .unwrap();
    east.open = false;
    assert!(route_step(&k, pos(0, 0), &[pos(1, 0)], &conflicting, &mut d).is_err());
    assert_eq!(d.next, 0);
}
