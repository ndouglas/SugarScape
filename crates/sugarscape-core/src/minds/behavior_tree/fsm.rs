//! Independent transitions with the routine's physical/task/RNG projection.
use super::forage::{completed, hold, physical, preflight, select, settle};
use super::policy::{allowed, expire_failed};
use super::runtime::Status;
use super::state::{FsmPhase, PolicyError, TaskState, Turn};
use super::telemetry;
use crate::world::World;

#[cfg(test)]
pub(crate) fn act_fsm(w: &mut World, id: u64, s: &mut TaskState) -> Result<Turn, PolicyError> {
    act_fsm_inner(w, id, s, true)
}
pub(crate) fn act_fsm_after_expiry(
    w: &mut World,
    id: u64,
    s: &mut TaskState,
) -> Result<Turn, PolicyError> {
    act_fsm_inner(w, id, s, false)
}
fn act_fsm_inner(
    w: &mut World,
    id: u64,
    s: &mut TaskState,
    expire: bool,
) -> Result<Turn, PolicyError> {
    let o = preflight(w, id, s)?;
    telemetry::reset(w);
    telemetry::note_candidates(w, o.candidates.len() as u64);
    if expire {
        expire_failed(s, o.action_tick);
    }
    if completed(s) {
        s.target = None;
        s.fsm.phase = FsmPhase::Finished;
        return Ok(hold(w, id, &o));
    }
    if s.target.is_some_and(|site| !allowed(&o, s, site, true)) {
        s.target = None;
        s.fsm.phase = FsmPhase::Selecting;
    }
    if s.target.is_none() {
        s.fsm.phase = FsmPhase::Selecting;
        select(w, &o, s);
    }
    let (harvest, receipt, status) = physical(w, id, s, &o);
    settle(s, &receipt)?;
    s.fsm.phase = if completed(s) {
        FsmPhase::Finished
    } else if status == Status::Running {
        FsmPhase::Moving
    } else if status == Status::Failure || s.target.is_none() && receipt.target == o.origin {
        FsmPhase::Deferred
    } else {
        FsmPhase::Selecting
    };
    Ok(Turn {
        harvest,
        receipt: Some(receipt),
        status: if completed(s) {
            Status::Success
        } else {
            status
        },
        visits: 0,
        exhausted: false,
    })
}
