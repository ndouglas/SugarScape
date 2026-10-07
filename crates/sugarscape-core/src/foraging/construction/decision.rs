//! Worker decisions receive only private knowledge, destination labels and local sensing.
use super::{
    draws::{checked_uniform, DrawSource},
    knowledge::CellKnowledge,
    metrics::ComputeCounts,
    navigation::{
        face_approaches, faces, frontiers, route_step, select_face, select_frontier, wander,
        Navigation,
    },
    observation::Observation,
    state::Agent,
    Checked, FoodPhase, Mode, Parameters, Pos, Setup, WorkCounts,
};
use crate::config::FieldError;
pub(super) struct Policy {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) nest: Vec<Pos>,
    pub(super) waste: Pos,
    pub(super) parameters: Parameters,
}
impl From<&Setup> for Policy {
    fn from(setup: &Setup) -> Self {
        Self {
            width: setup.width,
            height: setup.height,
            nest: setup.nest.clone(),
            waste: setup.waste,
            parameters: setup.parameters.clone(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WaitReason {
    Transition,
    EmptyArrival,
    NoNeighbor,
    Congestion,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Move(Pos),
    Dig(Pos),
    PickupFood,
    DepositFood,
    DisposeSpoil,
    Wait(WaitReason),
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Decision {
    pub(super) agent: Agent,
    pub(super) action: Action,
}
fn error(message: &str) -> Vec<FieldError> {
    vec![FieldError::new("navigation", message)]
}
fn search(agent: &mut Agent, abandoned: bool) -> Checked<Action> {
    agent.phase = FoodPhase::Searching;
    agent.site = None;
    agent.frontier = None;
    agent.face = None;
    agent.work.checked_include(&WorkCounts {
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
        _ => Err(error("expected a reachable travel destination")),
    }
}
fn frontier(
    agent: &mut Agent,
    site: Option<Pos>,
    draws: &mut impl DrawSource,
) -> Checked<Option<Pos>> {
    if let Some(target) = agent.frontier {
        let (candidates, counts) = frontiers(&agent.map, agent.pos)?;
        agent.compute.checked_include(&counts)?;
        if candidates.iter().any(|f| f.pos == target) {
            agent.face = None;
            return Ok(Some(target));
        }
    }
    let (target, counts) = select_frontier(&agent.map, agent.pos, site, draws)?;
    agent.compute.checked_include(&counts)?;
    agent.frontier = target;
    if target.is_some() {
        agent.face = None;
    }
    Ok(target)
}
fn excavation(
    agent: &mut Agent,
    site: Option<Pos>,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Option<Action>> {
    if let Some(target) = agent.face {
        let (candidates, counts) = faces(&agent.map, agent.pos)?;
        agent.compute.checked_include(&counts)?;
        if !candidates.iter().any(|f| f.pos == target) {
            agent.face = None;
        }
    }
    if agent.face.is_none() {
        let (target, counts) = select_face(&agent.map, agent.pos, site, draws)?;
        agent.compute.checked_include(&counts)?;
        agent.face = target;
    }
    let Some(target) = agent.face else {
        return Ok(None);
    };
    if observation
        .cells
        .iter()
        .any(|c| c.pos == target && !c.open && c.diggable)
    {
        return Ok(Some(Action::Dig(target)));
    }
    let (approaches, counts) = face_approaches(&agent.map, agent.pos, target)?;
    agent.compute.checked_include(&counts)?;
    let navigation = route(agent, &approaches, observation, draws)?;
    Ok(Some(movement(navigation)?))
}
fn pursue(
    agent: &mut Agent,
    site: Pos,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Action> {
    if site == agent.pos {
        return search(agent, false);
    }
    match agent.map.kind(site)? {
        CellKnowledge::KnownSolid { diggable: false } => search(agent, true),
        CellKnowledge::KnownOpen => {
            // Preserve F3's retained frontier field until its normal transition;
            // fresh sensing may invalidate an excavation commitment here.
            agent.face = None;
            let navigation = route(agent, &[site], observation, draws)?;
            if navigation == Navigation::Unreachable {
                search(agent, true)
            } else {
                movement(navigation)
            }
        }
        CellKnowledge::Unknown | CellKnowledge::KnownSolid { diggable: true } => {
            if let Some(target) = frontier(agent, Some(site), draws)? {
                let navigation = route(agent, &[target], observation, draws)?;
                movement(navigation)
            } else if let Some(action) = excavation(agent, Some(site), observation, draws)? {
                Ok(action)
            } else {
                search(agent, true)
            }
        }
    }
}
pub(super) fn decide(
    policy: &Policy,
    agent: &Agent,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Decision> {
    observation.validate(policy.width, policy.height)?;
    if observation.origin != agent.pos || agent.map.dimensions() != (policy.width, policy.height) {
        return Err(vec![FieldError::new(
            "observation",
            "must be fresh for the worker and actual policy dimensions",
        )]);
    }
    let mut agent = agent.clone();
    let delta = agent.map.learn(observation)?;
    agent.compute.checked_include(&ComputeCounts {
        observations: 1,
        cells_inspected: observation.cells.len() as u64,
        cells_learned: delta.first,
        observed_revisions: delta.observed_revisions,
        dig_confirmations: delta.dig_confirmations,
        ..Default::default()
    })?;
    let action = match agent.mode() {
        Mode::FoodReturning | Mode::EmptyReturning => {
            if policy.nest.contains(&agent.pos) {
                if agent.cargo.is_some() {
                    Action::DepositFood
                } else {
                    Action::Wait(WaitReason::EmptyArrival)
                }
            } else {
                let navigation = route(&mut agent, &policy.nest, observation, draws)?;
                movement(navigation)?
            }
        }
        Mode::SpoilHauling => {
            if agent.pos == policy.waste {
                Action::DisposeSpoil
            } else {
                match agent.map.kind(policy.waste)? {
                    CellKnowledge::KnownOpen => {
                        let navigation = route(&mut agent, &[policy.waste], observation, draws)?;
                        movement(navigation)?
                    }
                    CellKnowledge::Unknown => {
                        let target =
                            frontier(&mut agent, Some(policy.waste), draws)?.ok_or_else(|| {
                                error("unknown outlet after exhausting private open frontiers")
                            })?;
                        let navigation = route(&mut agent, &[target], observation, draws)?;
                        movement(navigation)?
                    }
                    CellKnowledge::KnownSolid { .. } => {
                        return Err(error("validated outlet cannot be known solid"))
                    }
                }
            }
        }
        Mode::Departing => {
            if let Some(site) = agent.site {
                pursue(&mut agent, site, observation, draws)?
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
        Mode::Searching => {
            if checked_uniform(draws)? < policy.parameters.p_return {
                agent.phase = FoodPhase::Returning;
                agent.find = None;
                agent.site = None;
                agent.frontier = None;
                agent.face = None;
                Action::Wait(WaitReason::Transition)
            } else if observation
                .cells
                .iter()
                .any(|c| c.pos == agent.pos && c.food)
            {
                Action::PickupFood
            } else if let Some(target) = frontier(&mut agent, None, draws)? {
                let navigation = route(&mut agent, &[target], observation, draws)?;
                movement(navigation)?
            } else if let Some(action) = excavation(&mut agent, None, observation, draws)? {
                action
            } else {
                match wander(agent.pos, observation, draws)? {
                    Navigation::Move(p) => Action::Move(p),
                    Navigation::Blocked => Action::Wait(WaitReason::NoNeighbor),
                    _ => return Err(error("unexpected wandering result")),
                }
            }
        }
    };
    Ok(Decision { agent, action })
}
