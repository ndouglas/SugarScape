//! Authoritative fixed-horizon runner. Research projections never feed a controller.
use super::{
    forage, fsm, lab, policy, records::*, runtime::Status, state::*, WorkCounters, EPISODE_SCHEMA,
};
use crate::{rules::Harvest, world::World};

pub(crate) fn turn(w: &mut World, id: u64) -> Option<Harvest> {
    let controller = w.config.behavior_tree_lab.as_ref()?.controller;
    let mut r = w.behavior_tree_lab.take().expect("constructed runtime");
    r.living_ticks += 1;
    w.bt_work = r.diagnostics.then(WorkCounters::default);
    let origin = w.agent(id).expect("live lab actor").pos;
    // Separately identified researcher projection, not reused as a free input.
    // Each controller still performs and counts its own actual candidate scans.
    r.observation = (r.diagnostics && !forage::completed(&r.task))
        .then(|| forage::observe(w, id, &r.task))
        .transpose()
        .expect("checked live actor in one-good lab");
    #[cfg(not(target_arch = "wasm32"))]
    let start = r.controller_timing.then(std::time::Instant::now);
    policy::expire_failed(&mut r.task, w.tick + 1);
    if forage::completed(&r.task) {
        // Runtime settlement already halted the Running child. The common hold
        // preserves the physical motion plan from the actual completing action.
        r.observation = None;
        r.controller_seconds = None;
        r.receipt = Some(PhysicalReceipt {
            action_tick: w.tick + 1,
            actor: id,
            origin,
            target: origin,
            destination: origin,
            gathered: 0.0,
            route_failed: false,
        });
        w.behavior_tree_lab = Some(r);
        return Some(Harvest::default());
    }
    let result = match controller {
        Controller::GuardedTree | Controller::UnguardedTree => forage::act_routine_after_expiry(
            w,
            id,
            &mut r.task,
            controller == Controller::GuardedTree,
            w.config.behavior_tree.visits,
        ),
        Controller::MatchedFsm => fsm::act_fsm_after_expiry(w, id, &mut r.task),
        Controller::TaskGoap => forage::act_task_goap_after_expiry(w, id, &mut r.task),
        Controller::ReactiveUtility | Controller::LegacyGoap => {
            let harvest = if controller == Controller::ReactiveUtility {
                crate::minds::utility::act(w, id)
            } else {
                crate::minds::goap::forage::act(w, id)
            };
            let a = w.agent(id).expect("live lab actor");
            let target = a.plan.target.unwrap_or(origin);
            let receipt = PhysicalReceipt {
                action_tick: w.tick + 1,
                actor: id,
                origin,
                target,
                destination: a.pos,
                gathered: harvest.gathered[0],
                route_failed: a.pos != target && a.plan.path.is_empty(),
            };
            forage::settle(&mut r.task, &receipt).map(|()| Turn {
                harvest,
                receipt: Some(receipt),
                status: Status::Success,
                visits: 0,
                exhausted: false,
            })
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        r.controller_seconds = start.map(|s| s.elapsed().as_secs_f64());
    }
    // The WASM platform has no native Instant; unavailable is None, never zero.
    #[cfg(target_arch = "wasm32")]
    {
        r.controller_seconds = None;
    }
    let harvest = match result {
        Ok(turn) => {
            r.receipt = turn.receipt;
            turn.harvest
        }
        Err(error) => {
            r.fatal_error = Some(error.message.clone());
            r.errors.push(error.message);
            Harvest::default()
        }
    };
    if forage::completed(&r.task) {
        r.task.target = None;
        r.task.task_plan = None;
        w.agent_mut(id).expect("live lab actor").goap_plan = None;
    }
    w.behavior_tree_lab = Some(r);
    Some(harvest)
}

pub(crate) fn note_metabolism(w: &mut World, consumed: f64) {
    if let Some(r) = w.behavior_tree_lab.as_mut() {
        r.consumed += consumed;
    }
}
pub(crate) fn note_removal(w: &mut World, id: u64, loss: f64) {
    if id == 1 {
        if let Some(r) = w.behavior_tree_lab.as_mut() {
            r.death_loss += loss;
        }
    }
}
pub(crate) fn reconcile(w: &mut World) {
    if w.behavior_tree_lab.is_none() {
        return;
    }
    let remaining = w.sites.iter().map(|s| s.resource[0]).sum::<f64>()
        + w.agents().map(|a| a.holdings[0].max(0.0)).sum::<f64>();
    let r = w.behavior_tree_lab.as_mut().unwrap();
    let supplied = 52.0 + 16.0 + r.external_added - r.external_removed;
    let residual = supplied - remaining - r.consumed - r.death_loss;
    if !residual.is_finite() || residual.abs() > 1e-9 {
        r.errors.push(format!("food balance residual {residual}"));
    }
}

fn actor(w: &World) -> Option<Actor> {
    w.agent(1).map(|a| Actor {
        id: a.id,
        pos: a.pos,
        holdings: a.holdings[0],
        metabolism: a.metabolism[0],
        vision: a.vision,
        remembers: a.remembers,
        age: a.age,
        max_age: a.max_age,
        memory: a
            .memory
            .sites
            .iter()
            .map(|(&site, s)| MemoryRecord {
                site,
                levels: s.levels().to_vec(),
                most: s.most().to_vec(),
                tick: s.tick,
            })
            .collect(),
        motion_plan: MotionPlan {
            target: a.plan.target,
            path: a.plan.path.clone(),
            walked: a.plan.walked,
        },
    })
}
fn legacy_plan(w: &World) -> Option<LegacyPlan> {
    w.agent(1)
        .and_then(|a| a.goap_plan.as_ref())
        .map(|p| LegacyPlan {
            steps: p.steps.clone(),
            goal: p.goal,
            gathers: p.gathers,
        })
}
/// Enabled-only semantic additions. Diagnostic options, observations, receipts,
/// counters, timing, and error history cannot affect this continuation hash.
pub(crate) fn semantic_bytes(w: &World) -> Vec<u8> {
    let r = w.behavior_tree_lab.as_ref().expect("enabled lab");
    serde_json::to_vec(&(
        &w.config,
        &r.task,
        r.living_ticks,
        r.external_added,
        r.external_removed,
        r.consumed,
        r.death_loss,
        &r.fatal_error,
        actor(w),
        legacy_plan(w),
        &w.walls,
        crate::rng::state_json(&w.rng),
    ))
    .expect("serializable semantic lab state")
}
/// Read-only research projection of an enabled food-task laboratory.
/// This uses the runner's canonical frame and cannot affect policy or RNG state.
pub fn snapshot(w: &World) -> Result<Frame, String> {
    if w.behavior_tree_lab.is_none() {
        return Err("behavior-tree snapshot requires an enabled lab".into());
    }
    Ok(frame(w))
}

pub(crate) fn frame(w: &World) -> Frame {
    let r = w.behavior_tree_lab.as_ref().expect("enabled lab");
    Frame {
        tick: w.tick,
        fingerprint: format!("{:016x}", w.fingerprint()),
        rng_state_json: crate::rng::state_json(&w.rng),
        actor: actor(w),
        cells: w
            .sites
            .iter()
            .enumerate()
            .map(|(i, s)| Cell {
                site: i as u32,
                food: s.resource[0],
                capacity: s.capacity[0],
                wall: w.walls[i],
            })
            .collect(),
        task: r.task.clone(),
        legacy_plan: legacy_plan(w),
        observation: r.observation.clone(),
        receipt: r.receipt.clone(),
        work: w.bt_work.clone(),
        controller_seconds: r.controller_seconds,
        external_added: r.external_added,
        external_removed: r.external_removed,
        consumed: r.consumed,
        death_loss: r.death_loss,
        living_ticks: r.living_ticks,
        errors: r.errors.clone(),
    }
}
fn episode(lab: &LabConfig, seed: u64, frames: Vec<Frame>) -> EpisodeRecord {
    let last = frames.last().expect("initial frame");
    let first_completion = last.task.first_completion;
    let quota_attained = first_completion.is_some();
    let complete = last.tick == 64;
    EpisodeRecord {
        schema: EPISODE_SCHEMA.into(),
        lab: lab.clone(),
        seed,
        completed_ticks: last.tick,
        quota_attained,
        first_completion,
        restricted_completion_ticks: first_completion.unwrap_or(64),
        right_censored: complete && !quota_attained,
        unattained_reason: if complete && !quota_attained {
            Some(if last.actor.is_none() {
                UnattainedReason::DiedBeforeQuota
            } else {
                UnattainedReason::HorizonWithoutQuota
            })
        } else {
            None
        },
        gross_gathered: last.task.gross,
        living_ticks: last.living_ticks,
        alive_at_horizon: complete && last.actor.is_some(),
        errors: last.errors.clone(),
        frames,
    }
}

pub fn run_episode(
    lab: LabConfig,
    seed: u64,
    options: RunOptions,
) -> Result<EpisodeRecord, EpisodeFailure> {
    run_episode_to(lab, seed, options, |_| Ok(()))
}
pub fn run_episode_to(
    lab: LabConfig,
    seed: u64,
    options: RunOptions,
    mut sink: impl FnMut(&Frame) -> Result<(), String>,
) -> Result<EpisodeRecord, EpisodeFailure> {
    let mut w =
        World::new(lab::rig_config(lab.clone()), seed).map_err(|errors| EpisodeFailure {
            message: format!("invalid lab: {errors:?}"),
            partial: None,
        })?;
    let r = w.behavior_tree_lab.as_mut().expect("constructed runtime");
    r.diagnostics = options.diagnostics;
    r.controller_timing = options.controller_timing;
    let mut frames = vec![frame(&w)];
    if let Err(message) = sink(frames.last().unwrap()) {
        return Err(EpisodeFailure {
            message,
            partial: Some(Box::new(episode(&lab, seed, frames))),
        });
    }
    while w.tick < 64 {
        let before = w.tick;
        w.step();
        if w.tick == before {
            let message = w
                .behavior_tree_lab
                .as_ref()
                .unwrap()
                .fatal_error
                .clone()
                .unwrap_or_else(|| "lab clock made no progress".into());
            return Err(EpisodeFailure {
                message,
                partial: Some(Box::new(episode(&lab, seed, frames))),
            });
        }
        frames.push(frame(&w)); // Known completed prefix precedes sink I/O.
        if let Err(message) = sink(frames.last().unwrap()) {
            return Err(EpisodeFailure {
                message,
                partial: Some(Box::new(episode(&lab, seed, frames))),
            });
        }
        let r = w.behavior_tree_lab.as_ref().unwrap();
        if let Some(message) = r.fatal_error.clone().or_else(|| r.errors.last().cloned()) {
            return Err(EpisodeFailure {
                message,
                partial: Some(Box::new(episode(&lab, seed, frames))),
            });
        }
    }
    Ok(episode(&lab, seed, frames))
}

pub fn physical_projection(f: &Frame) -> serde_json::Value {
    // Common commitment/deadline/motion state is comparable. Representation-specific
    // tree/FSM/task plans, instrumentation, history and RNG are compared separately.
    serde_json::json!({"tick":f.tick,"actor":f.actor,"cells":f.cells,"receipt":f.receipt,
        "quota":f.task.quota,"gross":f.task.gross,"first_completion":f.task.first_completion,
        "target":f.task.target,"failed_until":f.task.failed_until,
        "living_ticks":f.living_ticks,"external_added":f.external_added,"external_removed":f.external_removed,"consumed":f.consumed,"death_loss":f.death_loss})
}
