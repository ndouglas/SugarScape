//! Decisions inspect one worker's map and current observation, never physical truth.
use super::{
    draws::{checked_uniform, DrawSource},
    knowledge::CellKnowledge,
    metrics::ComputeCounts,
    navigation::{frontiers, route_step, select_frontier, wander, Navigation},
    observation::Observation,
    state::Agent,
    Checked, Parameters, Phase, Pos,
};
use crate::config::FieldError;

pub(super) struct Policy {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) nest: Vec<Pos>,
    pub(super) parameters: Parameters,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WaitReason {
    Transition,
    Congestion,
    NoNeighbor,
    EmptyArrival,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Move(Pos),
    Pickup,
    Deposit,
    Wait(WaitReason),
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Decision {
    pub(super) agent: Agent,
    pub(super) action: Action,
}

fn search(agent: &mut Agent, abandoned: bool) -> Checked<Action> {
    agent.phase = Phase::Searching;
    agent.site = None;
    agent.frontier = None;
    agent.work.checked_include(&super::WorkCounts {
        search_entries: 1,
        abandoned_targets: u64::from(abandoned),
        ..Default::default()
    })?;
    Ok(Action::Wait(WaitReason::Transition))
}
fn route(
    agent: &mut Agent,
    goals: &[Pos],
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Navigation> {
    let (navigation, counts) = route_step(&agent.map, agent.pos, goals, observation, draws)?;
    agent.compute.checked_include(&counts)?;
    Ok(navigation)
}
fn movement(navigation: Navigation) -> Checked<Action> {
    match navigation {
        Navigation::Move(p) => Ok(Action::Move(p)),
        Navigation::Blocked => Ok(Action::Wait(WaitReason::Congestion)),
        _ => Err(vec![FieldError::new(
            "navigation",
            "expected a reachable travel destination",
        )]),
    }
}
fn frontier(
    agent: &mut Agent,
    informed: Option<Pos>,
    draws: &mut impl DrawSource,
) -> Checked<Option<Pos>> {
    if let Some(target) = agent.frontier {
        let (candidates, counts) = frontiers(&agent.map, agent.pos)?;
        agent.compute.checked_include(&counts)?;
        if candidates.iter().any(|f| f.pos == target) {
            return Ok(Some(target));
        }
    }
    let (target, counts) = select_frontier(&agent.map, agent.pos, informed, draws)?;
    agent.compute.checked_include(&counts)?;
    agent.frontier = target;
    Ok(target)
}

pub(super) fn decide(
    policy: &Policy,
    agent: &Agent,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Decision> {
    observation.validate(policy.width, policy.height)?;
    if observation.origin != agent.pos
        || observation.cells.len() != agent.pos.neighbors(policy.width, policy.height).len() + 1
        || agent.map.dimensions() != (policy.width, policy.height)
    {
        return Err(vec![FieldError::new(
            "observation",
            "must be complete and fresh for the worker and policy dimensions",
        )]);
    }
    let mut agent = agent.clone();
    let learned = agent.map.learn(observation)?;
    agent.compute.checked_include(&ComputeCounts {
        observations: 1,
        cells_inspected: observation.cells.len() as u64,
        cells_learned: learned,
        ..Default::default()
    })?;
    let action = match agent.phase {
        Phase::Departing => {
            if let Some(site) = agent.site {
                if site == agent.pos {
                    search(&mut agent, false)?
                } else {
                    match agent.map.kind(site)? {
                        CellKnowledge::KnownSolid => search(&mut agent, true)?,
                        CellKnowledge::KnownOpen => {
                            let navigation = route(&mut agent, &[site], observation, draws)?;
                            if navigation == Navigation::Unreachable {
                                search(&mut agent, true)?
                            } else {
                                movement(navigation)?
                            }
                        }
                        CellKnowledge::Unknown => {
                            if let Some(target) = frontier(&mut agent, Some(site), draws)? {
                                let navigation = route(&mut agent, &[target], observation, draws)?;
                                movement(navigation)?
                            } else {
                                search(&mut agent, true)?
                            }
                        }
                    }
                }
            } else if checked_uniform(draws)? < policy.parameters.p_search
                || agent.frontier == Some(agent.pos)
            {
                search(&mut agent, false)?
            } else if let Some(target) = frontier(&mut agent, None, draws)? {
                let navigation = route(&mut agent, &[target], observation, draws)?;
                movement(navigation)?
            } else {
                search(&mut agent, false)?
            }
        }
        Phase::Searching => {
            if checked_uniform(draws)? < policy.parameters.p_return {
                agent.phase = Phase::Returning;
                agent.find = None;
                agent.site = None;
                agent.frontier = None;
                Action::Wait(WaitReason::Transition)
            } else if observation
                .cells
                .iter()
                .any(|c| c.pos == agent.pos && c.food)
            {
                Action::Pickup
            } else if let Some(target) = frontier(&mut agent, None, draws)? {
                let navigation = route(&mut agent, &[target], observation, draws)?;
                movement(navigation)?
            } else {
                match wander(agent.pos, observation, draws)? {
                    Navigation::Move(p) => Action::Move(p),
                    Navigation::Blocked => Action::Wait(WaitReason::NoNeighbor),
                    _ => {
                        return Err(vec![FieldError::new(
                            "navigation",
                            "unexpected wandering result",
                        )])
                    }
                }
            }
        }
        Phase::Returning => {
            if policy.nest.contains(&agent.pos) {
                if agent.cargo.is_some() {
                    Action::Deposit
                } else {
                    Action::Wait(WaitReason::EmptyArrival)
                }
            } else {
                let navigation = route(&mut agent, &policy.nest, observation, draws)?;
                movement(navigation)?
            }
        }
    };
    Ok(Decision { agent, action })
}
