//! The utility mind (Minds 1; Mark 2009, Dill and Mark 2010, Lewis 2017):
//! each candidate site's score is the product of its considerations —
//! rule M's welfare W, travel T(d) = 1 / (1 + k·d) and crowding
//! C = (1 + n)^(−m), n the Flumps on the site's von Neumann neighbors other
//! than the mover. With k = m = 0 the score is W exactly, and the choice is
//! rule M's (same candidates, tie rule and draw).
//!
//! Stated choices (the spec): W is not normalized (dividing by a constant
//! shared by every candidate doesn't change the order); no compensation
//! factor (every candidate has the same considerations); crowding is local.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::config::{Decision, Idle};
use crate::geometry::Pos;
use crate::portable::{exp_neg, ln};
use crate::rules::movement::{arrive, candidates_with_memory, choose, record_choice};
use crate::rules::Harvest;
use crate::world::World;

/// W · T(d) · C(n). A consideration at its neutral value (k = 0, m = 0) is
/// not computed, so the score is then `welfare` bit for bit.
pub fn score(welfare: f64, distance: u32, crowd: u32, d: &Decision) -> f64 {
    let mut s = welfare;
    if d.travel > 0.0 {
        s /= 1.0 + d.travel * f64::from(distance);
    }
    if d.crowding > 0.0 {
        s *= exp_neg(-d.crowding * ln(1.0 + f64::from(crowd)));
    }
    s
}

/// Flumps on `site`'s four von Neumann neighbors, not counting `mover`.
pub fn crowd(world: &World, site: Pos, mover: AgentId) -> u32 {
    world
        .torus
        .neighbors(site)
        .into_iter()
        .filter(|&q| world.occupant(q).is_some_and(|o| o != mover))
        .count() as u32
}

/// Rule M's step under the utility mind. A rememberer's remembered sites
/// (Minds 3) are scored with crowding 0: nobody out of sight can be counted.
/// Wandering happens only when nothing in sight scores and nothing
/// remembered scores above 0, and then only among the sites in sight.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let d = world.config.decision;
    let (welfare, start) = candidates_with_memory(world, id);
    // Scored only when a consideration is on; otherwise the score is the
    // welfare itself, and no copy is made.
    let rescored = (d.crowding > 0.0 || d.travel > 0.0).then(|| {
        let mut scored = welfare.clone();
        for (k, c) in scored.iter_mut().enumerate() {
            let n = if d.crowding > 0.0 && k < start {
                crowd(world, c.0, id)
            } else {
                0
            };
            c.2 = score(c.2, c.1, n, &d);
        }
        scored
    });
    let scored = rescored.as_deref().unwrap_or(&welfare);
    let idle = d.idle == Idle::Wander
        && scored[..start].iter().all(|c| c.2 == 0.0)
        && scored[start..].iter().all(|c| c.2 <= 0.0);
    let target = if idle {
        // The current site is scored[0]; wander among the others in sight,
        // if any.
        match scored[1..start].choose(&mut world.rng) {
            Some(c) => c.0,
            None => choose(scored, &mut world.rng),
        }
    } else {
        choose(scored, &mut world.rng)
    };
    record_choice(world, id, &welfare, start, target);
    arrive(world, id, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Decision, DecisionRule, Idle};
    use crate::geometry::Pos;
    use crate::testkit::*;

    fn utility(travel: f64, crowding: f64, idle: Idle) -> Decision {
        Decision {
            rule: DecisionRule::Utility,
            travel,
            crowding,
            idle,
        }
    }

    #[test]
    fn neutral_considerations_leave_welfare_exactly() {
        let d = utility(0.0, 0.0, Idle::Stay);
        for w in [0.0, 0.25, 3.0, 1e-300, 12345.678] {
            for (dist, n) in [(0, 0), (6, 4), (20, 3)] {
                assert_eq!(score(w, dist, n, &d).to_bits(), w.to_bits());
            }
        }
    }

    #[test]
    fn travel_discounts_hyperbolically_and_crowding_as_a_power() {
        let t = utility(0.5, 0.0, Idle::Stay);
        assert_eq!(score(4.0, 2, 0, &t), 4.0 / 2.0);
        assert_eq!(score(4.0, 0, 0, &t), 4.0);
        let c = utility(0.0, 1.0, Idle::Stay);
        assert!((score(4.0, 0, 1, &c) - 2.0).abs() < 1e-12); // 4 · 2^−1
        assert!((score(4.0, 0, 3, &c) - 1.0).abs() < 1e-12); // 4 · 4^−1
        assert_eq!(score(4.0, 0, 0, &c), 4.0);
        let c2 = utility(0.0, 2.0, Idle::Stay);
        assert!((score(9.0, 0, 2, &c2) - 1.0).abs() < 1e-12); // 9 · 3^−2
    }

    #[test]
    fn crowd_counts_neighbors_but_never_the_mover() {
        let mut w = blank_world(11, 11);
        let me = spawn(&mut w, 5, 5);
        spawn(&mut w, 5, 4);
        spawn(&mut w, 4, 5);
        assert_eq!(crowd(&w, Pos::new(5, 5), me), 2);
        // (6, 5)'s neighbors are (6, 4), (6, 6), (7, 5) and (5, 5): only the mover.
        assert_eq!(crowd(&w, Pos::new(6, 5), me), 0);
        // (4, 4)'s neighbors hold (5, 4) and (4, 5).
        assert_eq!(crowd(&w, Pos::new(4, 4), me), 2);
    }

    #[test]
    fn crowding_turns_a_flump_away_from_a_crowded_site() {
        let mut w = blank_world(11, 11);
        w.config.decision = utility(0.0, 1.0, Idle::Stay);
        let me = spawn(&mut w, 5, 5);
        w.agent_mut(me).unwrap().vision = 3;
        set_sugar(&mut w, 5, 8, 3.0); // neighbors (4, 8), (6, 8): crowd 2, score 1
        spawn(&mut w, 4, 8);
        spawn(&mut w, 6, 8);
        set_sugar(&mut w, 8, 5, 2.0); // no crowd: score 2
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(8, 5));
    }

    #[test]
    fn wander_moves_only_when_nothing_in_sight_scores_and_stay_does_not() {
        let run = |idle: Idle| {
            let mut w = blank_world(11, 11);
            w.config.decision = utility(0.0, 0.0, idle);
            let me = spawn(&mut w, 5, 5);
            w.agent_mut(me).unwrap().vision = 2;
            act(&mut w, me);
            w.agent(me).unwrap().pos
        };
        assert_eq!(run(Idle::Stay), Pos::new(5, 5));
        let p = run(Idle::Wander);
        assert_ne!(p, Pos::new(5, 5));
        assert!((p.x == 5) != (p.y == 5), "a site in sight along one axis");
    }

    #[test]
    fn a_wanderer_still_takes_sugar_in_sight() {
        let mut w = blank_world(11, 11);
        w.config.decision = utility(0.0, 0.0, Idle::Wander);
        let me = spawn(&mut w, 5, 5);
        w.agent_mut(me).unwrap().vision = 2;
        set_sugar(&mut w, 7, 5, 1.0);
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(7, 5));
    }

    #[test]
    fn a_boxed_in_wanderer_stays_and_draws_as_rule_m() {
        let mut w = blank_world(11, 11);
        w.config.decision = utility(0.0, 0.0, Idle::Wander);
        let me = spawn(&mut w, 5, 5);
        for (x, y) in [(5, 4), (5, 6), (4, 5), (6, 5)] {
            spawn(&mut w, x, y);
        }
        let before = w.rng.clone();
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(5, 5));
        // Staying draws exactly as rule M's choose over one candidate draws.
        let mut expected = before;
        crate::rules::movement::choose(&[(Pos::new(5, 5), 0, 0.0)], &mut expected);
        assert_eq!(w.rng, expected);
    }

    #[test]
    fn crowding_is_zero_for_a_remembered_site() {
        // Remembered (5, 9) worth 4 has two Flumps beside it, out of sight:
        // crowded it would score 4 / 3 and lose to (6, 5)'s 3, but the
        // walker can't know who's there, so it scores 4.
        let mut c = blank_config(21, 21);
        c.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        c.memory.span = 100;
        c.memory.belief = crate::config::Belief::Recall;
        c.decision = utility(0.0, 1.0, Idle::Stay);
        let mut w = World::new(c, 7).unwrap();
        let me = spawn(&mut w, 5, 5);
        w.agent_mut(me).unwrap().remembers = true;
        let idx = w.torus.index(Pos::new(5, 9)) as u32;
        let seen = crate::minds::memory::Seen::new(&[4.0], &[4.0], 0);
        w.agent_mut(me).unwrap().memory.sites.insert(idx, seen);
        spawn(&mut w, 4, 9);
        spawn(&mut w, 6, 9);
        set_sugar(&mut w, 6, 5, 3.0);
        w.tick = 1;
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().plan.target, Some(Pos::new(5, 9)));
        assert_eq!(w.events().remembered_moves, 1);
    }

    /// A rememberer at (5, 5) with vision 1 in a walking utility world with
    /// wander, remembering sugar `level` at (5, 9).
    fn wandering_rememberer(level: f64) -> (World, AgentId) {
        let mut c = blank_config(21, 21);
        c.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        c.memory.span = 100;
        c.memory.belief = crate::config::Belief::Recall;
        c.decision = utility(0.0, 0.0, Idle::Wander);
        let mut w = World::new(c, 7).unwrap();
        let me = spawn(&mut w, 5, 5);
        w.agent_mut(me).unwrap().remembers = true;
        let idx = w.torus.index(Pos::new(5, 9)) as u32;
        let seen = crate::minds::memory::Seen::new(&[level], &[level], 0);
        w.agent_mut(me).unwrap().memory.sites.insert(idx, seen);
        w.tick = 1;
        (w, me)
    }

    #[test]
    fn a_rememberer_with_nothing_in_sight_walks_to_a_remembered_site_rather_than_wander() {
        let (mut w, me) = wandering_rememberer(4.0);
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().plan.target, Some(Pos::new(5, 9)));
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(5, 6));
    }

    #[test]
    fn a_rememberer_with_nothing_positive_anywhere_wanders_only_in_sight() {
        for seed in 0..40 {
            let (mut w, me) = wandering_rememberer(0.0);
            w.rng = crate::rng::seeded(seed);
            act(&mut w, me);
            let t = w.agent(me).unwrap().plan.target.unwrap();
            assert_ne!(t, Pos::new(5, 9), "never a remembered site");
            assert_ne!(t, Pos::new(5, 5), "it wanders");
            assert_eq!(t.x.abs_diff(5) + t.y.abs_diff(5), 1, "in sight: {t:?}");
            assert_eq!(w.events().remembered_moves, 0);
        }
    }
}
