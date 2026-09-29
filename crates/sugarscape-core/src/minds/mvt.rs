//! Minds 4's marginal-value rule (`decision.rule: mvt`; Constantino & Daw
//! 2015): a Flump stays on a patch while its immediate surroundings still
//! out-yield a running estimate ρ of the habitat's average intake rate, and
//! leaves for the best site it knows of once they don't.
//!
//! - **ρ.** Updated every tick, after the move, from good 0's harvest that
//!   tick (0 on a travel tick that gathers nothing): ρ += α·(gain − ρ), the
//!   exponential moving average Constantino & Daw fit foragers to. Clamped
//!   at ≥ 0; `f64::max` also turns a NaN result into 0 rather than letting
//!   it escape (Review Focus 5).
//! - **Deciding.** A Flump already committed to leaving (`leaving`) walks
//!   there without reconsidering, so a better option passed along the way
//!   doesn't distract it. The commitment is dropped, and decided again the
//!   same tick, once it's reached or once it's no longer one of the
//!   Flump's reachable candidates (occupied out from under it, walled off,
//!   or where the last walk found no path — `reachable_candidates`, Task
//!   4's filter over `candidates_with_memory`). With nothing committed, the
//!   Flump looks at its own site and its lattice neighbors only (distance ≤
//!   1): the best of those by believed value, ties broken as rule M breaks
//!   them (`choose`, a draw among the nearest of the best). If that value
//!   is at least ρ, it heads there — its own site included, so staying is
//!   just heading to distance 0. Otherwise the patch has run dry: it
//!   commits to the single best site it knows of at all, in sight or
//!   remembered, ties broken without a draw (nearer, then the lower site
//!   index), and starts walking.

use crate::agent::AgentId;
use crate::geometry::{Pos, Torus};
use crate::rules::movement::{arrive, choose, record_choice};
use crate::rules::Harvest;
use crate::world::World;

use super::goap::forage::reachable_candidates;

/// ρ's exponential update from `gain` (this tick's good-0 harvest, 0 on a
/// travel tick), clamped at ≥ 0. With `gain ≥ 0` and `alpha ∈ (0, 1]` the
/// result can't go negative on its own, but the clamp guards the arithmetic
/// regardless; `f64::max` returns its non-NaN argument when the other is
/// NaN, so this can't produce NaN either.
fn updated_rate(rate: f64, alpha: f64, gain: f64) -> f64 {
    (rate + alpha * (gain - rate)).max(0.0)
}

/// Whether `t` still holds as where the Flump is heading: it's arrived, or
/// `t` is still one of its reachable candidates (so, if it was in sight,
/// it's still unoccupied; it isn't walled off; and it isn't where the last
/// walk found no path).
fn still_reachable(pos: Pos, candidates: &[(Pos, u32, f64)], t: Pos) -> bool {
    t == pos || candidates.iter().any(|c| c.0 == t)
}

/// The single best-known candidate (in sight or remembered): highest
/// believed value, then nearer, then the lower site index. Deterministic;
/// draws nothing (rule M's tie rule is only for the local choice).
fn best_known(torus: Torus, candidates: &[(Pos, u32, f64)]) -> Pos {
    candidates
        .iter()
        .min_by(|a, b| {
            b.2.total_cmp(&a.2)
                .then(a.1.cmp(&b.1))
                .then(torus.index(a.0).cmp(&torus.index(b.0)))
        })
        .expect("the Flump's own site is always a candidate")
        .0
}

/// Heads for `target` (counted as Minds 3 counts a choice).
fn go(
    world: &mut World,
    id: AgentId,
    candidates: &[(Pos, u32, f64)],
    start: usize,
    target: Pos,
) -> Harvest {
    record_choice(world, id, candidates, start, target);
    arrive(world, id, target)
}

/// Rule M's step under the marginal value rule (see the module doc for the
/// stay/leave decision). Updates ρ from this tick's good-0 gain before
/// returning the harvest.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let (candidates, start) = reachable_candidates(world, id);
    let a = world.agent(id).expect("live agent");
    let (pos, committed, rate) = (a.pos, a.leaving, a.rate);

    let target = match committed {
        Some(t) if still_reachable(pos, &candidates, t) => t,
        _ => {
            if committed.is_some() {
                world.agent_mut(id).expect("live agent").leaving = None;
            }
            let local: Vec<(Pos, u32, f64)> =
                candidates.iter().copied().filter(|c| c.1 <= 1).collect();
            let best_local = local.iter().map(|c| c.2).fold(f64::NEG_INFINITY, f64::max);
            if best_local >= rate {
                choose(&local, &mut world.rng)
            } else {
                let t = best_known(world.torus, &candidates);
                world.agent_mut(id).expect("live agent").leaving = Some(t);
                world.events.leaves += 1;
                t
            }
        }
    };

    let harvest = go(world, id, &candidates, start, target);
    let alpha = world.config.mvt.alpha;
    let a = world.agent_mut(id).expect("live agent");
    if a.leaving == Some(a.pos) {
        a.leaving = None;
    }
    a.rate = updated_rate(a.rate, alpha, harvest.gathered[0]);
    harvest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DecisionRule, MoveMode, Movement, Mvt};
    use crate::testkit::*;

    /// A walking MVT world (speed 1) with smoothing `alpha`.
    fn mvt_world(size: u32, alpha: f64) -> World {
        let mut c = blank_config(size, size);
        c.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        c.decision.rule = DecisionRule::Mvt;
        c.mvt = Mvt { alpha };
        World::new(c, 7).unwrap()
    }

    /// A Flump at (x, y) with `vision` and rate ρ = `rate`.
    fn forager(w: &mut World, x: u32, y: u32, vision: u32, rate: f64) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.vision = vision;
        a.rate = rate;
        id
    }

    #[test]
    fn rho_updates_by_the_smoothed_gain_and_clamps_at_zero() {
        assert_eq!(updated_rate(2.0, 0.5, 10.0), 6.0);
        // A gain of 0 (a travel tick) decays ρ toward 0, never past it.
        assert_eq!(updated_rate(1.0, 0.25, 0.0), 0.75);
        // Never negative even against a contrived negative input.
        assert_eq!(updated_rate(1.0, 1.0, -5.0), 0.0);
        // Never NaN even from a NaN input: the arithmetic propagates it, but
        // the clamp's `f64::max` turns it into 0 rather than letting it out.
        assert_eq!(updated_rate(f64::NAN, 0.5, 1.0), 0.0);
        assert_eq!(updated_rate(1.0, 0.5, f64::NAN), 0.0);
    }

    #[test]
    fn staying_while_the_local_value_is_at_least_rho() {
        let mut w = mvt_world(21, 0.1);
        let id = forager(&mut w, 5, 5, 6, 2.0);
        set_sugar(&mut w, 5, 5, 2.0); // own site: local best, ties ρ
        set_sugar(&mut w, 5, 9, 100.0); // far and rich, but out of reach (dist 4)
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5), "stays");
        assert_eq!(w.agent(id).unwrap().leaving, None);
        assert_eq!(w.events.leaves, 0);
        // ρ is smoothed toward the 2.0 it just gathered: stays at 2.0.
        assert_eq!(w.agent(id).unwrap().rate, 2.0);
    }

    #[test]
    fn a_neighbor_at_least_as_good_as_rho_is_taken_over_staying() {
        let mut w = mvt_world(21, 0.1);
        let id = forager(&mut w, 5, 5, 6, 1.0);
        set_sugar(&mut w, 5, 4, 3.0); // one neighbor, north
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 4));
        assert_eq!(w.agent(id).unwrap().leaving, None);
    }

    #[test]
    fn leaving_commits_past_a_closer_distraction() {
        // Own site and every neighbor are worth 0, well below ρ = 1: the
        // Flump commits to the only known sugar, three sites away.
        let mut w = mvt_world(21, 0.1);
        let id = forager(&mut w, 5, 5, 6, 1.0);
        set_sugar(&mut w, 5, 8, 5.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().leaving, Some(Pos::new(5, 8)));
        assert_eq!(w.events.leaves, 1);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6), "one step closer");
        // A closer, richer site now appears beside its path; a fresh choice
        // would take it, but the commitment holds instead.
        set_sugar(&mut w, 5, 7, 50.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7), "on the way");
        assert_eq!(
            w.agent(id).unwrap().leaving,
            Some(Pos::new(5, 8)),
            "still committed to the original target"
        );
        assert_eq!(w.events.leaves, 1, "no second commitment");
    }

    #[test]
    fn arrival_clears_the_commitment() {
        let mut w = mvt_world(21, 0.1);
        let id = forager(&mut w, 5, 5, 6, 1.0);
        set_sugar(&mut w, 5, 7, 5.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().leaving, Some(Pos::new(5, 7)));
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6));
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7), "arrived");
        assert_eq!(w.agent(id).unwrap().leaving, None, "cleared on arrival");
    }

    #[test]
    fn a_commitment_taken_by_another_flump_is_dropped_and_redecided() {
        let mut w = mvt_world(21, 0.1);
        let id = forager(&mut w, 5, 5, 6, 1.0);
        set_sugar(&mut w, 5, 8, 5.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().leaving, Some(Pos::new(5, 8)));
        spawn(&mut w, 5, 8); // taken
        act(&mut w, id);
        assert_ne!(
            w.agent(id).unwrap().leaving,
            Some(Pos::new(5, 8)),
            "dropped, not walked into"
        );
    }

    #[test]
    fn metabolism_zero_holds_rho_at_zero_over_many_gainless_ticks() {
        // Metabolism 0 starts ρ at 0 (no draw); with no sugar anywhere to
        // gain, it must stay exactly 0, never dipping negative.
        let mut w = mvt_world(21, 0.2);
        let id = spawn(&mut w, 5, 5);
        assert_eq!(w.agent(id).unwrap().metabolism[0], 0);
        assert_eq!(
            w.agent(id).unwrap().rate,
            0.0,
            "starts at metabolism, no draw"
        );
        for _ in 0..40 {
            act(&mut w, id);
            let rate = w.agent(id).unwrap().rate;
            assert!(rate >= 0.0, "never negative: {rate}");
            assert_eq!(rate, 0.0, "no gain ever: ρ decays to (and stays at) 0");
        }
    }

    #[test]
    fn a_nonzero_rho_decays_toward_zero_without_gain_and_never_overshoots() {
        let mut w = mvt_world(21, 0.2);
        let id = forager(&mut w, 5, 5, 6, 5.0);
        let mut prev = 5.0;
        for _ in 0..40 {
            act(&mut w, id);
            let rate = w.agent(id).unwrap().rate;
            assert!(rate >= 0.0, "never negative: {rate}");
            assert!(rate <= prev, "monotonically decaying: {rate} > {prev}");
            prev = rate;
        }
        assert!(prev < 0.01, "decayed close to zero: {prev}");
    }

    #[test]
    fn rate_and_leaving_are_not_hashed() {
        let mut w = mvt_world(9, 0.1);
        let id = spawn(&mut w, 1, 1);
        let before = w.fingerprint();
        let a = w.agent_mut(id).unwrap();
        a.rate = 99.0;
        a.leaving = Some(Pos::new(2, 2));
        assert_eq!(w.fingerprint(), before);
    }
}
