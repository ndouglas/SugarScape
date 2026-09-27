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
use crate::rules::movement::{candidates, choose, go_and_gather};
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

/// Rule M's step under the utility mind.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let d = world.config.decision;
    let mut scored = candidates(world, id);
    if d.crowding > 0.0 || d.travel > 0.0 {
        for c in &mut scored {
            let n = if d.crowding > 0.0 {
                crowd(world, c.0, id)
            } else {
                0
            };
            c.2 = score(c.2, c.1, n, &d);
        }
    }
    let target = if d.idle == Idle::Wander && scored.iter().all(|c| c.2 == 0.0) {
        // The current site is scored[0]; wander among the others, if any.
        match scored[1..].choose(&mut world.rng) {
            Some(c) => c.0,
            None => {
                let target = choose(&scored, &mut world.rng);
                return go_and_gather(world, id, target);
            }
        }
    } else {
        choose(&scored, &mut world.rng)
    };
    go_and_gather(world, id, target)
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
    fn a_boxed_in_wanderer_stays_and_draws_nothing() {
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
}
