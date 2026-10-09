//! Pure selection from an actor's estimated food, with independent retry clocks.
use super::state::{Observation, PolicyError, TaskState};
use crate::config::{Decision, DecisionRule, Idle};
use crate::rng::SimRng;
use crate::rules::movement::choose;

pub(crate) fn expire_failed(s: &mut TaskState, action_tick: u64) {
    s.failed_until.retain(|_, until| action_tick < *until);
}

pub(crate) fn note_failure(
    s: &mut TaskState,
    site: u32,
    action_tick: u64,
) -> Result<(), PolicyError> {
    let until = action_tick.checked_add(3).ok_or_else(|| PolicyError {
        message: "failed-target expiry overflow".into(),
    })?;
    s.failed_until.insert(site, until);
    s.target = None;
    Ok(())
}

pub(crate) fn allowed(o: &Observation, s: &TaskState, site: u32, guarded: bool) -> bool {
    !s.failed_until.contains_key(&site)
        && o.candidates
            .iter()
            .any(|c| c.site == site && (!guarded || c.value > 0.0))
}

pub(crate) fn select_target(o: &Observation, s: &TaskState, rng: &mut SimRng) -> Option<u32> {
    let decision = Decision {
        rule: DecisionRule::Utility,
        travel: 1.0,
        crowding: 0.0,
        idle: Idle::Stay,
    };
    let scored: Vec<_> = o
        .candidates
        .iter()
        .filter(|c| c.value > 0.0 && !s.failed_until.contains_key(&c.site))
        .map(|c| {
            (
                c.pos,
                c.distance,
                crate::minds::utility::score(c.value, c.distance, 0, &decision),
            )
        })
        .collect();
    if scored.is_empty() {
        return None;
    }
    let target = choose(&scored, rng);
    o.candidates
        .iter()
        .find(|c| c.pos == target)
        .map(|c| c.site)
}
