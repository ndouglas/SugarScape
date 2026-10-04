use super::super::{
    controller::{decide, decide_measured, select_ticket, target_weight},
    observation::{exit_distances, exit_distances_measured, is_recent, observe, observe_measured},
    *,
};
use crate::rng;
use rand::{Rng, RngCore};

fn p(x: u32, y: u32) -> Pos {
    Pos { x, y }
}

fn choice(side: Side, pile: Pile) -> World {
    World::new(
        LabConfig {
            fixture: Fixture::Choice { side, pile },
            ..Default::default()
        },
        7,
    )
    .unwrap()
}

fn view(w: &World, id: u32) -> WorkerView {
    WorkerView::from(&w.workers[id as usize])
}

fn decision(w: &World, id: u32, outward: &[ExitNeighbor]) -> Decision {
    decide(
        &observe(w, id),
        &view(w, id),
        &w.config,
        w.workers[id as usize].pos == w.setup.exit,
        outward,
        &mut rng::seeded(7),
    )
}

fn corridor() -> World {
    World::new(
        LabConfig {
            fixture: Fixture::Corridor {
                length: 7,
                workers: 1,
            },
            transport: Transport::Relay,
            ..Default::default()
        },
        7,
    )
    .unwrap()
}

#[test]
fn freshness_excludes_the_boundary_and_blind_weights_ignore_counts() {
    assert!(is_recent(31, 0, 32));
    assert!(!is_recent(32, 0, 32));
    assert!(!is_recent(0, 1, 32));
    let mut c = LabConfig {
        cue: Cue::Blind,
        ..Default::default()
    };
    assert_eq!(target_weight(&c, 4), 1);
    c.cue = Cue::Responsive;
    assert_eq!(target_weight(&c, 1), 1);
    assert_eq!(target_weight(&c, 2), 3);
}

#[test]
fn integer_tickets_map_exactly_and_mirror() {
    assert_eq!(
        (0..4)
            .map(|t| select_ticket(&[3, 1], t))
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1]
    );
    assert_eq!(
        (0..4)
            .map(|t| select_ticket(&[1, 3], t))
            .collect::<Vec<_>>(),
        vec![0, 1, 1, 1]
    );
    assert_eq!(select_ticket(&[u32::MAX, u32::MAX], u64::from(u32::MAX)), 1);
}

#[test]
fn observes_two_open_hops_and_exposes_only_adjoining_faces() {
    let w = choice(Side::Left, Pile::FreshAccumulation);
    let o = observe(&w, 0);
    assert_eq!(
        o.open.iter().map(|c| c.pos).collect::<Vec<_>>(),
        (2..=6).map(|x| p(x, 3)).collect::<Vec<_>>()
    );
    assert_eq!(o.frontier, vec![p(1, 3), p(7, 3)]);
    assert_eq!(o.open[0].recent, 4);
    assert_eq!(o.open[0].loose, 4);
    assert_eq!(o.open[2].occupants, 1);
}

#[test]
fn solid_cells_hide_nearby_piles_and_deeper_targets() {
    // The pile is Manhattan-distance two but four traversable hops away.
    let s = Setup {
        width: 5,
        height: 4,
        exit: p(0, 1),
        start_tick: 64,
        open: vec![p(0, 1), p(0, 2), p(1, 2), p(2, 2), p(2, 1)],
        diggable: vec![p(1, 1), p(3, 1)],
        workers: vec![p(0, 1)],
        spoil: vec![InitialSpoil {
            pos: p(2, 1),
            born: 64,
        }],
    };
    let w = World::from_setup(LabConfig::default(), s, 7).unwrap();
    let o = observe(&w, 0);
    assert_eq!(o.frontier, vec![p(1, 1)]);
    assert!(!o.open.iter().any(|c| c.pos == p(2, 1)));
    assert!(o.open.iter().all(|c| c.loose == 0 && c.recent == 0));
}

#[test]
fn old_and_single_fresh_piles_have_baseline_response() {
    for pile in [Pile::OldAccumulation, Pile::SingleFresh] {
        let mut w = choice(Side::Left, pile);
        w.config.cue = Cue::Responsive;
        let o = observe(&w, 0);
        assert_eq!(target_weight(&w.config, o.open[0].recent), 1);
        assert_eq!(
            o.open[0].loose,
            if pile == Pile::SingleFresh { 1 } else { 4 }
        );
    }
}

#[test]
fn shared_approach_material_counts_once_for_each_target() {
    let s = Setup {
        width: 5,
        height: 4,
        exit: p(2, 2),
        start_tick: 64,
        open: vec![p(2, 2), p(2, 1)],
        diggable: vec![p(1, 1), p(3, 1)],
        workers: vec![p(2, 2)],
        spoil: vec![
            InitialSpoil {
                pos: p(2, 1),
                born: 64
            };
            2
        ],
    };
    let mut w = World::from_setup(LabConfig::default(), s, 7).unwrap();
    w.config.cue = Cue::Responsive;
    for (threshold, total, left_weight) in [(2, 6u64, 3), (3, 2, 1)] {
        w.config.minimum_recent_units = threshold;
        for seed in 0..16 {
            let mut expected = rng::seeded(seed);
            let ticket = expected.gen_range(0..total);
            let d = decide(
                &observe(&w, 0),
                &view(&w, 0),
                &w.config,
                true,
                &[],
                &mut rng::seeded(seed),
            );
            assert_eq!(
                d.selected,
                Some(if ticket < left_weight {
                    p(1, 1)
                } else {
                    p(3, 1)
                })
            );
            assert_eq!(d.action, Action::Move(p(2, 1)));
        }
    }
}

#[test]
fn responsive_selection_uses_observed_approach_counts_only() {
    for (side, weights) in [(Side::Left, [3u32, 1]), (Side::Right, [1, 3])] {
        let mut w = choice(side, Pile::FreshAccumulation);
        w.config.cue = Cue::Responsive;
        for seed in 0..16 {
            let mut expected = rng::seeded(seed);
            let ticket = expected.gen_range(0..4u64);
            let want = if ticket < u64::from(weights[0]) {
                p(1, 3)
            } else {
                p(7, 3)
            };
            assert_eq!(
                decide(
                    &observe(&w, 0),
                    &view(&w, 0),
                    &w.config,
                    true,
                    &[],
                    &mut rng::seeded(seed)
                )
                .selected,
                Some(want)
            );
        }
    }
}

#[test]
fn response_weight_one_reduces_to_blind_decisions() {
    let w = choice(Side::Left, Pile::FreshAccumulation);
    let mut c = w.config.clone();
    c.cue = Cue::Responsive;
    c.response_weight = 1;
    for seed in 0..16 {
        assert_eq!(
            decide(
                &observe(&w, 0),
                &view(&w, 0),
                &c,
                true,
                &[],
                &mut rng::seeded(seed)
            ),
            decide(
                &observe(&w, 0),
                &view(&w, 0),
                &w.config,
                true,
                &[],
                &mut rng::seeded(seed)
            )
        );
    }
}

#[test]
fn existing_valid_target_is_kept_without_selection_draw() {
    let mut w = choice(Side::Left, Pile::FreshAccumulation);
    w.workers[0].target = Some(p(1, 3));
    let mut actual = rng::seeded(7);
    let d = decide(
        &observe(&w, 0),
        &view(&w, 0),
        &w.config,
        true,
        &[],
        &mut actual,
    );
    assert_eq!(d.action, Action::Move(p(3, 3)));
    assert_eq!(d.target, Some(p(1, 3)));
    assert_eq!(d.selected, None);
    assert_eq!(actual.next_u64(), rng::seeded(7).next_u64());
}

#[test]
fn target_removed_by_other_worker_is_reselected() {
    let mut s = fixtures::setup(&choice(Side::Left, Pile::OldAccumulation).config).unwrap();
    s.workers.push(p(2, 3));
    let mut w = World::from_setup(LabConfig::default(), s, 7).unwrap();
    w.workers[0].target = Some(p(1, 3));
    assert_eq!(w.apply(1, Action::Dig(p(1, 3))).outcome, Outcome::Success);
    let d = decision(&w, 0, &[]);
    assert_eq!(d.selected, Some(p(7, 3)));
    assert_eq!(d.action, Action::Move(p(5, 3)));
}

#[test]
fn target_leaving_observation_is_cleared() {
    let mut w = corridor();
    w.workers[0].pos = p(0, 1);
    w.workers[0].target = Some(p(7, 1));
    let d = decision(&w, 0, &[]);
    assert_eq!(d.target, None);
    assert_eq!(d.action, Action::Move(p(1, 1)));
}

#[test]
fn blind_pickup_uses_one_half_draw_and_smallest_material() {
    let mut s = fixtures::corridor_setup(7, 1).unwrap();
    s.spoil = vec![
        InitialSpoil {
            pos: p(6, 1),
            born: 0
        };
        2
    ];
    let w = World::from_setup(LabConfig::default(), s, 7).unwrap();
    let mut pickups = 0;
    for seed in 0..16 {
        let mut reference = rng::seeded(seed);
        let pickup = reference.gen_bool(0.5);
        let mut actual = rng::seeded(seed);
        let d = decide(
            &observe(&w, 0),
            &view(&w, 0),
            &w.config,
            false,
            &[],
            &mut actual,
        );
        if pickup {
            pickups += 1;
            assert_eq!(d.action, Action::Pickup);
            assert_eq!(actual.next_u64(), reference.next_u64());
            let mut copy = w.clone();
            assert_eq!(copy.apply(0, d.action).material, Some(0));
        } else {
            assert_eq!(d.action, Action::Dig(p(7, 1)));
        }
    }
    assert!(pickups > 0 && pickups < 16);
}

#[test]
fn adjacent_selected_frontier_is_dug_and_clears_target() {
    let w = corridor();
    let d = decision(&w, 0, &[]);
    assert_eq!(d.action, Action::Dig(p(7, 1)));
    assert_eq!(d.target, None);
    assert_eq!(d.selected, Some(p(7, 1)));
}

#[test]
fn full_route_waits_with_valid_target_even_when_other_wander_move_exists() {
    let mut s = fixtures::setup(&choice(Side::Left, Pile::OldAccumulation).config).unwrap();
    s.workers.extend([p(3, 3), p(3, 3)]);
    let mut w = World::from_setup(LabConfig::default(), s, 7).unwrap();
    w.workers[0].target = Some(p(1, 3));
    let d = decision(&w, 0, &[]);
    assert_eq!(d.action, Action::Wait);
    assert_eq!(d.target, Some(p(1, 3)));
}

#[test]
fn wandering_uses_only_legal_observed_neighbors() {
    let mut w = corridor();
    w.workers[0].pos = p(2, 1);
    w.workers[0].target = None;
    for seed in 0..16 {
        let d = decide(
            &observe(&w, 0),
            &view(&w, 0),
            &w.config,
            false,
            &[],
            &mut rng::seeded(seed),
        );
        assert!(matches!(d.action, Action::Move(to) if to == p(1, 1) || to == p(3, 1)));
    }
}

#[test]
fn no_target_or_legal_neighbor_waits() {
    let s = Setup {
        width: 1,
        height: 1,
        exit: p(0, 0),
        start_tick: 0,
        open: vec![p(0, 0)],
        diggable: vec![],
        workers: vec![p(0, 0)],
        spoil: vec![],
    };
    let w = World::from_setup(LabConfig::default(), s, 7).unwrap();
    assert_eq!(decision(&w, 0, &[]).action, Action::Wait);
}

#[test]
fn exit_disposal_has_priority_over_relay_drop() {
    let mut w = corridor();
    w.apply(0, Action::Dig(p(7, 1)));
    for x in (0..6).rev() {
        w.apply(0, Action::Move(p(x, 1)));
    }
    assert_eq!(decision(&w, 0, &[]).action, Action::Dispose);
}

#[test]
fn relay_drops_only_after_three_successful_loaded_moves() {
    let mut w = corridor();
    w.apply(0, Action::Dig(p(7, 1)));
    for x in (3..6).rev() {
        let next = ExitNeighbor {
            pos: p(x, 1),
            distance: x,
            occupants: 0,
        };
        assert_eq!(decision(&w, 0, &[next]).action, Action::Move(p(x, 1)));
        w.apply(0, Action::Move(p(x, 1)));
    }
    assert_eq!(decision(&w, 0, &[]).action, Action::Drop);
}

#[test]
fn waits_and_blocked_moves_do_not_advance_relay_leg() {
    let mut w = corridor();
    w.apply(0, Action::Dig(p(7, 1)));
    for _ in 0..4 {
        assert_eq!(decision(&w, 0, &[]).action, Action::Wait);
        w.apply(0, Action::Wait);
        w.apply(0, Action::Move(p(0, 1)));
    }
    assert_eq!(w.workers[0].loaded_moves, 0);
    assert_eq!(decision(&w, 0, &[]).action, Action::Wait);
}

#[test]
fn direct_transport_never_drops_internally() {
    let mut w = corridor();
    w.config.transport = Transport::Direct;
    w.apply(0, Action::Dig(p(7, 1)));
    for x in (1..6).rev() {
        w.apply(0, Action::Move(p(x, 1)));
    }
    assert_eq!(decision(&w, 0, &[]).action, Action::Wait);
    let next = ExitNeighbor {
        pos: p(0, 1),
        distance: 0,
        occupants: 0,
    };
    assert_eq!(decision(&w, 0, &[next]).action, Action::Move(p(0, 1)));
}

#[test]
fn full_descending_neighbors_are_excluded() {
    let mut w = corridor();
    w.apply(0, Action::Dig(p(7, 1)));
    let full = ExitNeighbor {
        pos: p(5, 1),
        distance: 5,
        occupants: 2,
    };
    assert_eq!(decision(&w, 0, &[full]).action, Action::Wait);
}

#[test]
fn exit_bfs_uses_open_cells_and_bounded_distances() {
    let w = corridor();
    let distances = exit_distances(&w);
    assert_eq!(distances.len(), 27);
    assert_eq!(
        &distances[9..18],
        &[
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            Some(6),
            None,
            None
        ]
    );
    assert!(distances[..9].iter().all(Option::is_none));
}

#[test]
fn cue_and_observation_changes_cannot_override_action_legality() {
    for cue in [Cue::Blind, Cue::Responsive] {
        let mut w = corridor();
        w.config.cue = cue;
        w.apply(0, Action::Dig(p(7, 1)));
        let o = Observation {
            origin: p(6, 1),
            open: vec![ObservedCell {
                pos: p(6, 1),
                occupants: 1,
                loose: 0,
                recent: 0,
            }],
            frontier: vec![p(6, 0)],
        };
        let mut fake = view(&w, 0);
        fake.carrying = false;
        let d = decide(&o, &fake, &w.config, false, &[], &mut rng::seeded(7));
        assert!(matches!(
            w.apply(0, d.action).outcome,
            Outcome::Blocked { .. }
        ));
        w.check_invariants().unwrap();
    }
}

#[test]
fn measured_observation_and_exit_bfs_report_exact_corridor_work() {
    let w = corridor();
    let (o, local) = observe_measured(&w, 0);
    assert_eq!(o, observe(&w, 0));
    assert_eq!((local.calls, local.visits, local.peak_queue), (1, 3, 1));
    let (field, global) = exit_distances_measured(&w);
    assert_eq!(field, exit_distances(&w));
    assert_eq!((global.calls, global.visits, global.peak_queue), (1, 7, 1));
}

#[test]
fn measured_routing_matches_plain_decisions_without_extra_draws() {
    let mut w = choice(Side::Left, Pile::FreshAccumulation);
    w.workers[0].target = Some(p(1, 3));
    let o = observe(&w, 0);
    let mut plain = rng::seeded(7);
    let mut measured = rng::seeded(7);
    let d = decide(&o, &view(&w, 0), &w.config, true, &[], &mut plain);
    let (md, stats) = decide_measured(&o, &view(&w, 0), &w.config, true, &[], &mut measured);
    assert_eq!(d, md);
    assert_eq!(plain.next_u64(), measured.next_u64());
    assert_eq!((stats.calls, stats.visits, stats.peak_queue), (2, 10, 2));
}
