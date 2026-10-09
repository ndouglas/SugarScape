//! Standalone task adapters. Metabolism belongs to the caller.
use super::policy::{allowed, expire_failed, note_failure, select_target};
use super::runtime::{self, Host, Node, Physical, Status};
use super::state::*;
use super::telemetry;
use crate::minds::goap::{forage::Forage, plan, Domain};
use crate::rules::movement::{arrive, candidates_with_memory, choose, record_choice};
use crate::rules::Harvest;
use crate::world::World;

pub(crate) fn observe(w: &World, id: u64, s: &TaskState) -> Result<Observation, PolicyError> {
    let a = w.agent(id).ok_or_else(|| error("task actor is not live"))?;
    if w.config.goods.len() != 1 {
        return Err(error("food task needs exactly one good"));
    }
    let action_tick = w
        .tick
        .checked_add(1)
        .ok_or_else(|| error("action tick overflow"))?;
    let (candidates, start) = candidates_with_memory(w, id);
    Ok(Observation {
        action_tick,
        origin: a.pos,
        quota: s.quota,
        gross: s.gross,
        candidates: candidates
            .into_iter()
            .enumerate()
            .map(|(i, (pos, distance, value))| Candidate {
                site: w.torus.index(pos) as u32,
                pos,
                distance,
                value,
                remembered: i >= start,
            })
            .collect(),
    })
}

fn error(message: &str) -> PolicyError {
    PolicyError {
        message: message.into(),
    }
}

pub(crate) fn preflight(w: &World, id: u64, s: &TaskState) -> Result<Observation, PolicyError> {
    let o = observe(w, id, s)?;
    // Any physical failure can be settled without overflowing its retry clock.
    o.action_tick
        .checked_add(3)
        .ok_or_else(|| error("failed-target expiry overflow"))?;
    if s.target.is_some_and(|site| site as usize >= w.torus.len()) {
        return Err(error("task target is out of range"));
    }
    Ok(o)
}

pub(crate) fn completed(s: &TaskState) -> bool {
    s.gross >= f64::from(s.quota)
}

pub(crate) fn hold(w: &World, id: u64, o: &Observation) -> Turn {
    let pos = w.agent(id).expect("validated live actor").pos;
    Turn {
        harvest: Harvest::default(),
        receipt: Some(PhysicalReceipt {
            action_tick: o.action_tick,
            actor: id,
            origin: pos,
            target: pos,
            destination: pos,
            gathered: 0.0,
            route_failed: false,
        }),
        status: Status::Success,
        visits: 0,
        exhausted: false,
    }
}

pub(crate) fn select(w: &mut World, o: &Observation, s: &mut TaskState) {
    telemetry::note_selection(w);
    telemetry::note_candidates(
        w,
        o.candidates
            .iter()
            .filter(|c| c.value > 0.0 && !s.failed_until.contains_key(&c.site))
            .count() as u64,
    );
    s.target = select_target(o, s, &mut w.rng);
}

/// Only this primitive touches the actual destination stock. Incidental
/// underfoot food is gathered even when every selected target is excluded.
pub(crate) fn physical(
    w: &mut World,
    id: u64,
    s: &TaskState,
    o: &Observation,
) -> (Harvest, PhysicalReceipt, Status) {
    let target = s.target.map_or(o.origin, |site| w.torus.pos(site as usize));
    let candidates: Vec<_> = o
        .candidates
        .iter()
        .map(|c| (c.pos, c.distance, c.value))
        .collect();
    let start = o
        .candidates
        .iter()
        .position(|c| c.remembered)
        .unwrap_or(candidates.len());
    record_choice(w, id, &candidates, start, target);
    let harvest = arrive(w, id, target);
    let a = w.agent(id).expect("validated live actor");
    let route_failed = a.pos != target && a.plan.path.is_empty();
    let empty_arrival = s.target.is_some() && a.pos == target && harvest.gathered[0] <= 0.0;
    let status = if route_failed || empty_arrival {
        Status::Failure
    } else if a.pos == target {
        Status::Success
    } else {
        Status::Running
    };
    let receipt = PhysicalReceipt {
        action_tick: o.action_tick,
        actor: id,
        origin: o.origin,
        target,
        destination: a.pos,
        gathered: harvest.gathered[0],
        route_failed,
    };
    (harvest, receipt, status)
}

pub(crate) fn settle(s: &mut TaskState, receipt: &PhysicalReceipt) -> Result<(), PolicyError> {
    s.gross += receipt.gathered;
    if let Some(site) = s.target {
        if receipt.route_failed {
            note_failure(s, site, receipt.action_tick)?;
        } else if receipt.destination == receipt.target {
            s.target = None;
        }
    }
    if completed(s) {
        s.first_completion.get_or_insert(receipt.action_tick);
        s.target = None;
        s.task_plan = None;
    }
    Ok(())
}

struct RoutineHost<'a> {
    world: &'a mut World,
    actor: u64,
    state: &'a mut TaskState,
    observation: &'a Observation,
    guarded: bool,
    harvest: Harvest,
    settlement_error: Option<PolicyError>,
}
impl Host for RoutineHost<'_> {
    type Receipt = PhysicalReceipt;
    fn supports(&self, node: &Node) -> bool {
        matches!(
            node,
            Node::Condition(0 | 1) | Node::Logical(0) | Node::Physical(0)
        )
    }
    fn condition(&self, id: u8) -> bool {
        match id {
            0 => self.observation.gross < f64::from(self.observation.quota),
            1 => self
                .state
                .target
                .is_none_or(|site| allowed(self.observation, self.state, site, self.guarded)),
            _ => unreachable!("validated condition"),
        }
    }
    fn logical(&mut self, _: u8) -> Status {
        // Restored logical continuations may already own a target. Selection
        // is made once, including the terminal None choice cached by runtime.
        if self.state.target.is_none() {
            select(self.world, self.observation, self.state);
        }
        Status::Success
    }
    fn physical(&mut self, _: u8) -> Physical<PhysicalReceipt> {
        let (harvest, receipt, status) =
            physical(self.world, self.actor, self.state, self.observation);
        self.harvest = harvest;
        Physical { receipt, status }
    }
    fn settle(&mut self, receipt: &PhysicalReceipt) {
        self.settlement_error = settle(self.state, receipt).err();
    }
    fn complete(&self) -> bool {
        completed(self.state)
    }
    fn halt(&mut self, _: u8) {
        self.state.target = None;
    }
}

// Standalone adapter fixtures retain the original self-expiring interface.
#[cfg(test)]
pub(crate) fn act_routine(
    w: &mut World,
    id: u64,
    s: &mut TaskState,
    guarded: bool,
    visits: u16,
) -> Result<Turn, PolicyError> {
    act_routine_inner(w, id, s, guarded, visits, true)
}
pub(crate) fn act_routine_after_expiry(
    w: &mut World,
    id: u64,
    s: &mut TaskState,
    guarded: bool,
    visits: u16,
) -> Result<Turn, PolicyError> {
    act_routine_inner(w, id, s, guarded, visits, false)
}
fn act_routine_inner(
    w: &mut World,
    id: u64,
    s: &mut TaskState,
    guarded: bool,
    visits: u16,
    expire: bool,
) -> Result<Turn, PolicyError> {
    let tree = super::routine_tree();
    s.tree
        .validate_for_tree(&tree)
        .map_err(|e| error(&e.message))?;
    if !(1..=64).contains(&visits) {
        return Err(error("visit budget must be in 1..=64"));
    }
    let o = preflight(w, id, s)?;
    telemetry::reset(w);
    telemetry::note_candidates(w, o.candidates.len() as u64);
    let mut traversal = std::mem::take(&mut s.tree);
    let mut host = RoutineHost {
        world: w,
        actor: id,
        state: s,
        observation: &o,
        guarded,
        harvest: Harvest::default(),
        settlement_error: None,
    };
    if expire {
        expire_failed(host.state, o.action_tick);
    }
    let normalize = host
        .state
        .target
        .is_some_and(|site| !allowed(&o, host.state, site, guarded));
    let halted = if normalize || host.complete() {
        runtime::halt(&tree, &mut traversal, &mut host)
    } else {
        Ok(())
    };
    if normalize {
        host.state.target = None;
    }
    let result = halted.and_then(|()| runtime::tick(&tree, &mut traversal, &mut host, visits));
    // Restore traversal on success AND every error before leaving this adapter.
    host.state.tree = traversal;
    let evaluated = result.map_err(|e| error(&e.message))?;
    if let Some(e) = host.settlement_error {
        return Err(e);
    }
    if let Some(c) = host.world.bt_work.as_mut() {
        c.node_visits += u64::from(evaluated.visits);
    }
    if host.complete() && evaluated.receipt.is_none() {
        return Ok(hold(host.world, id, &o));
    }
    Ok(Turn {
        harvest: host.harvest,
        receipt: evaluated.receipt,
        status: evaluated.status,
        visits: evaluated.visits,
        exhausted: evaluated.exhausted,
    })
}

#[cfg(test)]
pub(crate) fn act_task_goap(
    w: &mut World,
    id: u64,
    s: &mut TaskState,
) -> Result<Turn, PolicyError> {
    act_task_goap_inner(w, id, s, true)
}
pub(crate) fn act_task_goap_after_expiry(
    w: &mut World,
    id: u64,
    s: &mut TaskState,
) -> Result<Turn, PolicyError> {
    act_task_goap_inner(w, id, s, false)
}
fn act_task_goap_inner(
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
        s.task_plan = None;
        return Ok(hold(w, id, &o));
    }
    // The legacy reachable/visible-half-value contract, not tree guarding.
    let (candidates, start) = crate::minds::goap::forage::reachable_candidates(w, id);
    telemetry::note_candidates(w, o.candidates.len() as u64);
    let retained = s
        .task_plan
        .as_ref()
        .and_then(|p| p.steps.first())
        .copied()
        .filter(|(target, planned)| {
            candidates
                .iter()
                .position(|c| c.0 == *target)
                .is_some_and(|i| i >= start || candidates[i].2 >= planned / 2.0)
        });
    let target = if let Some((target, _)) = retained {
        target
    } else {
        s.task_plan = None;
        telemetry::note_selection(w);
        let mut others: Vec<_> = (1..candidates.len()).collect();
        let evaluations = w.bt_work.as_ref().map(|_| std::cell::Cell::new(0u64));
        let rate = |i: usize| {
            if let Some(n) = evaluations.as_ref() {
                n.set(n.get() + 1);
            }
            candidates[i].2 / (f64::from(candidates[i].1) + 1.0)
        };
        others.sort_by(|&i, &j| {
            rate(j)
                .total_cmp(&rate(i))
                .then(candidates[i].1.cmp(&candidates[j].1))
                .then(
                    w.torus
                        .index(candidates[i].0)
                        .cmp(&w.torus.index(candidates[j].0)),
                )
        });
        if let Some(n) = evaluations.as_ref() {
            telemetry::note_candidates(w, n.get());
        }
        others.truncate(8);
        let slots: Vec<_> = std::iter::once(0).chain(others).collect();
        let sites: Vec<_> = slots
            .iter()
            .map(|&i| (candidates[i].0, candidates[i].2))
            .collect();
        let goal = (f64::from(s.quota) - s.gross).max(0.0);
        let mut domain = Forage::new(w.torus, &sites, goal);
        if w.bt_work.is_some() {
            domain.observe_candidates();
        }
        let short = !domain.is_goal(&(0, (1u16 << sites.len()) - 1));
        let found = if short {
            None
        } else {
            plan(&domain, (0, 0), crate::minds::goap::forage::PLAN_LIMIT)
        };
        if let Some(n) = domain.candidate_evaluations() {
            telemetry::note_candidates(w, n);
        }
        if let Some(found) = found {
            telemetry::note_search(w, Some(found.expanded as u64), None);
            let steps: Vec<_> = found
                .actions
                .iter()
                .map(|&i| sites[usize::from(i)])
                .collect();
            let target = steps.first().map_or(o.origin, |s| s.0);
            s.task_plan = Some(TaskPlan { goal, steps });
            target
        } else {
            if short {
                if let Some(c) = w.bt_work.as_mut() {
                    c.fallback_short += 1;
                }
            } else {
                telemetry::note_search(
                    w,
                    None,
                    Some("failed search does not expose exact expansions"),
                );
                if let Some(c) = w.bt_work.as_mut() {
                    c.fallback_limit += 1;
                }
            }
            // The fallback evaluates each candidate in its maximum scan and filter.
            telemetry::note_candidates(w, 2 * candidates.len() as u64);
            let best = candidates
                .iter()
                .map(|c| c.2 / (f64::from(c.1) + 1.0))
                .fold(f64::NEG_INFINITY, f64::max);
            let top: Vec<_> = candidates
                .iter()
                .filter(|c| c.2 / (f64::from(c.1) + 1.0) == best)
                .copied()
                .collect();
            choose(&top, &mut w.rng)
        }
    };
    s.target = Some(w.torus.index(target) as u32);
    let (harvest, receipt, status) = physical(w, id, s, &o);
    if receipt.destination == target {
        if let Some(p) = s.task_plan.as_mut() {
            if p.steps.first().is_some_and(|step| step.0 == target) {
                p.steps.remove(0);
            }
        }
    }
    settle(s, &receipt)?;
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

pub(crate) fn act(w: &mut World, id: u64) -> Harvest {
    telemetry::reset(w);
    match w.config.behavior_tree.profile {
        Profile::BookLeaf => crate::rules::movement::act(w, id),
        Profile::UtilityLeaf => crate::minds::utility::act(w, id),
        Profile::GuardedRate | Profile::UnguardedRate => unreachable!("checked lab dispatch"),
    }
}
