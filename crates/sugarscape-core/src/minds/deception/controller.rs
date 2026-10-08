//! A paid, owner-local gesture primitive. Episode routing lives in the schedule.
use super::{
    accounting, observation,
    state::{SenderPolicy, Stage},
};
use crate::{minds::protection::ledger::Outflow, rules::Harvest, world::World};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoutKind {
    Neutral,
    Sham,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoutResult {
    Completed,
    Unaffordable,
    Occupied,
    Unreachable,
    OwnerDied,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoutRecord {
    pub actor: u64,
    pub tick: u64,
    pub kind: BoutKind,
    pub effort: f64,
    pub result: BoutResult,
}
fn record(w: &mut World, actor: u64, kind: BoutKind, effort: f64, result: BoutResult) {
    if let Some(r) = w.deception.as_mut().filter(|r| r.diagnostics) {
        r.bouts.push(BoutRecord {
            actor,
            tick: w.tick,
            kind,
            effort,
            result,
        });
    }
}
pub(crate) fn perform_bout(
    w: &mut World,
    actor: u64,
    kind: BoutKind,
    effort: f64,
) -> Result<BoutResult, String> {
    if w.config.deception_lab.is_none() || w.deception.is_none() {
        return Err("paid bout requires the bounded deception lab".into());
    }
    if !effort.is_finite() || effort < 0.0 {
        return Err("bout effort must be finite and nonnegative".into());
    }
    let a = w
        .agent(actor)
        .ok_or_else(|| format!("bout actor {actor} is not alive"))?;
    let state = a
        .deception
        .as_ref()
        .ok_or("bout actor has no sender state")?;
    if state.attempted {
        return Err("sender already attempted its episode bout".into());
    }
    let held = a.holdings[0];
    if !held.is_finite() {
        return Err("bout holdings must be finite".into());
    }
    let site = w.torus.index(a.pos) as u32;
    let a = w.agent_mut(actor).expect("checked alive actor");
    a.deception.as_mut().expect("checked sender").attempted = true;
    if held < effort {
        record(w, actor, kind, effort, BoutResult::Unaffordable);
        return Ok(BoutResult::Unaffordable);
    }
    a.holdings[0] -= effort;
    accounting::update(w, actor, |l| l.outflow(effort, Outflow::ActionCost));
    accounting::reconcile_world(w);
    // The sender never reads receiver memory, view, visibility or stock.
    if kind == BoutKind::Sham {
        observation::dispatch(w, actor, site, 0.0, false)?;
    }
    record(w, actor, kind, effort, BoutResult::Completed);
    Ok(BoutResult::Completed)
}
/// Minimal scheduled-turn seam. The scheduler owns stage transitions and routing.
pub(crate) fn display_turn(w: &mut World, actor: u64) -> Option<Harvest> {
    let lab = w.config.deception_lab.as_ref()?;
    let a = w.agent(actor)?;
    let state = a.deception.as_ref()?;
    if state.stage != Stage::Display || state.attempted {
        return None;
    }
    let kind = match lab.sender {
        SenderPolicy::Ordinary => return None,
        SenderPolicy::MatchedNeutral => BoutKind::Neutral,
        SenderPolicy::Sham => BoutKind::Sham,
    };
    let effort = lab.effort_cost;
    perform_bout(w, actor, kind, effort).expect("checked lab display bout");
    Some(Harvest::default())
}
