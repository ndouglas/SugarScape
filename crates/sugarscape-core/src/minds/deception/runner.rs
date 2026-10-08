//! Fixed prospective schedule and researcher-only episode execution.
pub(crate) use super::lab::{observer_route, owner_route};
use super::{
    controller::{BoutKind, BoutRecord, BoutResult},
    lab::{self, transform},
    records::*,
    state::*,
    SCHEMA,
};
use crate::{
    geometry::Pos,
    rules::{movement, Harvest},
    world::World,
};

pub(crate) fn action(w: &mut World, actor: u64, f: impl FnOnce(&mut ActionRecord)) {
    if w.config.deception_lab.is_none() {
        return;
    }
    if let Some(a) = w
        .deception
        .as_mut()
        .and_then(|r| r.actions.iter_mut().rev().find(|a| a.actor == actor))
    {
        f(a);
    }
}
pub(crate) fn start_action(w: &mut World, actor: u64) {
    if w.config.deception_lab.is_none() {
        return;
    }
    let phase = w
        .agent(actor)
        .and_then(|a| a.deception.as_ref())
        .map_or_else(
            || {
                if w.tick < 32 {
                    "scripted_observer".into()
                } else {
                    "ordinary".into()
                }
            },
            |s| format!("{:?}", s.stage).to_lowercase(),
        );
    if let Some(r) = w.deception.as_mut() {
        r.actions.push(ActionRecord {
            actor,
            phase,
            action: "ordinary".into(),
            ..Default::default()
        });
    }
}
pub(crate) fn finish_action(w: &mut World, actor: u64, h: Harvest) {
    if w.config.deception_lab.is_none() {
        return;
    }
    let pos = w.agent(actor).map(|a| a.pos);
    if actor == 1
        && h.dug > 0.0
        && w.deception
            .as_ref()
            .is_some_and(|r| pos == Some(w.torus.pos(r.source as usize)))
    {
        if let Some(s) = w.agent_mut(actor).and_then(|a| a.deception.as_mut()) {
            s.pending_departure = true;
        }
        if let Some(r) = w.deception.as_mut() {
            r.source_recovered = true;
        }
    }
    action(w, actor, |a| {
        a.harvest = h.gathered[0];
        a.dug = h.dug;
        a.pos = pos;
    });
    let restricted = actor == 2 && w.tick < 32
        || actor == 1
            && (w.tick < 8
                || w.tick < 20
                    && w.config
                        .deception_lab
                        .as_ref()
                        .is_some_and(|c| c.sender != SenderPolicy::Ordinary));
    if restricted {
        if let Some(r) = w.deception.as_mut() {
            *r.restrictions.entry(actor).or_default() += 1;
        }
    }
}
pub(crate) fn cancel(w: &mut World, actor: u64, result: BoutResult) {
    let Some(lab) = w.config.deception_lab.clone() else {
        return;
    };
    if lab.sender == SenderPolicy::Ordinary {
        return;
    }
    let Some(s) = w.agent_mut(actor).and_then(|a| a.deception.as_mut()) else {
        return;
    };
    if s.attempted {
        return;
    }
    s.attempted = true;
    s.stage = Stage::Cancelled;
    let pos = w.agent(actor).map(|a| a.pos);
    if w.deception
        .as_ref()
        .is_some_and(|r| !r.actions.iter().any(|a| a.actor == actor))
    {
        start_action(w, actor);
    }
    if let Some(r) = w.deception.as_mut() {
        r.bouts.push(BoutRecord {
            actor,
            tick: w.tick,
            kind: if lab.sender == SenderPolicy::Sham {
                BoutKind::Sham
            } else {
                BoutKind::Neutral
            },
            effort: lab.effort_cost,
            result,
        });
    }
    action(w, actor, |a| {
        a.cancellation = Some(result);
        a.action = "cancelled".into();
        a.pos = pos;
    });
}
pub(crate) fn removed(w: &mut World, actor: u64, cause: &str) {
    if w.config.deception_lab.is_none() {
        return;
    }
    if let Some(pos) = w.agent(actor).map(|a| a.pos) {
        cancel(w, actor, BoutResult::OwnerDied);
        if let Some(r) = w.deception.as_mut() {
            r.deaths.push(DeathRecord {
                actor,
                pos,
                cause: cause.into(),
            });
        }
    }
}
/// Reset only per-tick research buffers; schedules read supplied time/state.
pub(crate) fn begin_tick(w: &mut World) {
    let Some(lab) = w.config.deception_lab.clone() else {
        return;
    };
    if let Some(r) = w.deception.as_mut() {
        r.actions.clear();
        r.observations.clear();
        r.choices.clear();
        r.deaths.clear();
        r.bouts.clear();
    }
    let tick = w.tick;
    if let Some(s) = w.agent_mut(1).and_then(|a| a.deception.as_mut()) {
        if s.stage != Stage::Cancelled {
            s.stage = if s.pending_departure {
                Stage::Departure
            } else if tick < 8 {
                Stage::Preparation
            } else if lab.sender == SenderPolicy::Ordinary || tick >= 20 {
                Stage::Ordinary
            } else if tick < 13 {
                Stage::ToDisplay
            } else if tick == 13 {
                Stage::Display
            } else {
                Stage::Return
            };
        }
    }
    if w.tick == 13 && lab.sender != SenderPolicy::Ordinary {
        let desired = w
            .deception
            .as_ref()
            .map(|r| w.torus.pos(r.display as usize));
        if w.agent(1).is_some_and(|a| Some(a.pos) != desired) {
            cancel(w, 1, BoutResult::Unreachable);
        }
    }
}
fn hold(w: &mut World, actor: u64) -> Harvest {
    action(w, actor, |a| a.action = "hold".into());
    Harvest::default()
}
fn walk(w: &mut World, actor: u64, target: Pos, zero_gather: bool, departure: bool) -> Harvest {
    let occupied = w.occupant(target).filter(|id| *id != actor);
    let recovered = w.deception.as_ref().is_some_and(|r| r.source_recovered);
    let before = w.agent(actor).expect("live scripted role").pos;
    let (h, outcome) = if zero_gather {
        (
            Harvest::default(),
            movement::walk_without_gather(w, actor, target),
        )
    } else {
        let h = movement::arrive(w, actor, target);
        let pos = w.agent(actor).expect("live role").pos;
        let outcome = if pos == target {
            movement::WalkOutcome::Arrived
        } else if pos != before {
            movement::WalkOutcome::Advanced
        } else if occupied.is_some() {
            movement::WalkOutcome::Blocked
        } else {
            movement::WalkOutcome::Unreachable
        };
        (h, outcome)
    };
    action(w, actor, |a| {
        a.action = if departure {
            "departure"
        } else if zero_gather {
            "walk_without_gather"
        } else {
            "walk_and_gather"
        }
        .into();
        a.target = Some(target);
        a.walk_outcome = Some(format!("{outcome:?}").to_lowercase());
        a.target_occupant = occupied;
        a.source_recovered = recovered;
    });
    if !departure && outcome != movement::WalkOutcome::Arrived {
        let result = if outcome == movement::WalkOutcome::Blocked {
            BoutResult::Occupied
        } else {
            BoutResult::Unreachable
        };
        if actor == 1 {
            cancel(w, actor, result);
        }
        if let Some(r) = w.deception.as_mut() {
            r.fixture_errors.push(format!(
                "tick {} role {actor} waypoint {target:?}: {outcome:?}",
                w.tick
            ));
        }
    }
    h
}
pub(crate) fn scripted_action(w: &mut World, actor: u64) -> Option<Harvest> {
    let lab = w.config.deception_lab.clone()?;
    if actor == 1 {
        if w.agent(actor)?.deception.as_ref()?.pending_departure {
            let s = w.agent_mut(actor)?.deception.as_mut()?;
            s.pending_departure = false;
            s.stage = Stage::Departure;
            return Some(walk(w, actor, transform(&lab, Pos::new(3, 2)), false, true));
        }
        if w.tick == 0 {
            let q = crate::minds::caching::bury(w, actor, NOMINAL_AMOUNT);
            action(w, actor, |a| {
                a.action = "prepare".into();
                a.buried = q;
            });
            return Some(Harvest::default());
        }
        if w.tick < 8 {
            return Some(hold(w, actor));
        }
        if lab.sender == SenderPolicy::Ordinary {
            return None;
        }
        if (8..13).contains(&w.tick) {
            let route = owner_route(lab.layout, true);
            return Some(if let Some(&p) = route.get((w.tick - 8) as usize) {
                walk(w, actor, transform(&lab, p), false, false)
            } else {
                hold(w, actor)
            });
        }
        if w.tick == 13 {
            return Some(hold(w, actor));
        } // Display seam acts before this when valid.
        if (14..20).contains(&w.tick) {
            let route = owner_route(lab.layout, false);
            return Some(if let Some(&p) = route.get((w.tick - 14) as usize) {
                walk(w, actor, transform(&lab, p), false, false)
            } else {
                hold(w, actor)
            });
        }
    } else if actor == 2 && w.tick < 32 {
        if !lab.display_seen {
            let index = if (8..13).contains(&w.tick) {
                Some((true, (w.tick - 8) as usize))
            } else if (14..19).contains(&w.tick) {
                Some((false, (w.tick - 14) as usize))
            } else {
                None
            };
            if let Some((out, i)) = index {
                return Some(walk(
                    w,
                    actor,
                    transform(&lab, observer_route(out)[i]),
                    true,
                    false,
                ));
            }
        }
        return Some(hold(w, actor));
    }
    None
}
/// Record the selected candidate at the decision seam, before movement.
pub(crate) fn choice(
    w: &mut World,
    actor: u64,
    candidates: &[(Pos, u32, f64)],
    target: Pos,
    actual_value: f64,
) {
    if w.config.deception_lab.is_none() {
        return;
    }
    let value = candidates
        .iter()
        .find(|c| c.0 == target)
        .map_or(0.0, |c| c.2);
    let occupied = w.occupant(target).filter(|id| *id != actor);
    let recovered = w.deception.as_ref().is_some_and(|r| r.source_recovered);
    if let Some(r) = w.deception.as_mut() {
        r.choices.push(ChoiceRecord {
            actor,
            target,
            remembered_value: value,
            actual_value,
            arrived: false,
            raid_amount: 0.0,
            wasted: false,
            inspection: None,
            inspected_stock: None,
            target_occupant: occupied,
            source_recovered: recovered,
        });
    }
}
/// Actual inspection and raid result; target was already captured at choice.
pub(crate) fn inspection(
    w: &mut World,
    actor: u64,
    pos: Pos,
    stock: f64,
    h: Harvest,
    wasted: bool,
) {
    if w.config.deception_lab.is_none() {
        return;
    }
    if let Some(c) = w
        .deception
        .as_mut()
        .and_then(|r| r.choices.iter_mut().rev().find(|c| c.actor == actor))
    {
        c.arrived = pos == c.target;
        c.inspection = Some(pos);
        c.inspected_stock = Some(stock);
        c.raid_amount = h.pilfered;
        c.wasted = wasted;
    }
}
fn frame(w: &World) -> FrameRecord {
    let r = w.deception.as_ref().expect("constructed runtime");
    FrameRecord {
        tick: w.tick,
        fingerprint: format!("{:016x}", w.fingerprint()),
        roles: w
            .agents()
            .map(|a| RoleRecord {
                id: a.id,
                pos: a.pos,
                holdings: a.holdings[0],
                caches: a.caches.clone(),
                sender: a.deception.clone(),
                seen: a
                    .seen
                    .iter()
                    .map(|(&(site, owner), e)| SeenRecord {
                        site,
                        owner,
                        amount: e.amount,
                        tick: e.tick,
                    })
                    .collect(),
            })
            .collect(),
        actions: r.actions.clone(),
        observations: r.observations.clone(),
        choices: r.choices.clone(),
        deaths: r.deaths.clone(),
        restrictions: r.restrictions.clone(),
        stocks: w
            .sites
            .iter()
            .enumerate()
            .map(|(i, s)| StockRecord {
                site: i as u32,
                amount: s.resource[0],
            })
            .collect(),
    }
}
fn episode(
    w: &World,
    lab: &LabConfig,
    seed: u64,
    frames: &[FrameRecord],
    owner_ticks_alive: u64,
    requested_diagnostics: bool,
) -> EpisodeRecord {
    let r = w.deception.as_ref().expect("constructed runtime");
    let cohorts = r.ledger.clone();
    let thief_transferred = cohorts
        .as_ref()
        .map(|l| l.cohorts.values().map(|c| c.transferred).sum());
    let reason = if cohorts.is_some() {
        None
    } else {
        Some(if !requested_diagnostics {
            "lineage diagnostics disabled".into()
        } else {
            format!("lineage unavailable: {}", r.ledger_errors.join("; "))
        })
    };
    EpisodeRecord {
        schema: SCHEMA.into(),
        lab: lab.clone(),
        seed,
        requested_ticks: TICKS,
        completed_ticks: w.tick,
        owner_ticks_alive,
        owner_alive: w.agent(1).is_some(),
        frames: frames.to_vec(),
        fixture_errors: r.fixture_errors.clone(),
        ledger_errors: r.ledger_errors.clone(),
        cohorts,
        thief_transferred,
        diagnostics_enabled: r.diagnostics,
        lineage_unavailable_reason: reason,
    }
}
// The approved wire/API contract retains Option<EpisodeRecord> directly in
// EpisodeFailure so consumers can recover every known partial frame.
#[allow(clippy::result_large_err)]
pub fn run_episode(
    lab: LabConfig,
    seed: u64,
    diagnostics: bool,
) -> Result<EpisodeRecord, EpisodeFailure> {
    run_episode_with_sink(lab, seed, diagnostics, |_| Ok(()))
}
// Keep the same mandated partial-record error type as run_episode.
#[allow(clippy::result_large_err)]
pub fn run_episode_with_sink<F: FnMut(&FrameRecord) -> Result<(), String>>(
    lab: LabConfig,
    seed: u64,
    diagnostics: bool,
    mut sink: F,
) -> Result<EpisodeRecord, EpisodeFailure> {
    let mut w = World::new(lab::rig_config(lab.clone()), seed).map_err(|e| EpisodeFailure {
        message: format!("{e:?}"),
        partial: None,
    })?;
    if !diagnostics {
        let r = w.deception.as_mut().expect("runtime");
        r.diagnostics = false;
        r.ledger = None;
    }
    let mut frames = vec![frame(&w)];
    let mut owner_ticks_alive = 0;
    if let Err(message) = sink(&frames[0]) {
        return Err(EpisodeFailure {
            message,
            partial: Some(episode(
                &w,
                &lab,
                seed,
                &frames,
                owner_ticks_alive,
                diagnostics,
            )),
        });
    }
    for _ in 0..TICKS {
        if w.population() == 0 {
            break;
        }
        owner_ticks_alive += u64::from(w.agent(1).is_some());
        w.step();
        frames.push(frame(&w));
        if let Err(message) = sink(frames.last().expect("completed frame")) {
            return Err(EpisodeFailure {
                message,
                partial: Some(episode(
                    &w,
                    &lab,
                    seed,
                    &frames,
                    owner_ticks_alive,
                    diagnostics,
                )),
            });
        }
    }
    Ok(episode(
        &w,
        &lab,
        seed,
        &frames,
        owner_ticks_alive,
        diagnostics,
    ))
}
