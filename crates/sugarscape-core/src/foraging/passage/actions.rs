//! Physical transactions and nest-only advice run against the candidate tick.
use super::{
    decision::{Action, Decision, WaitReason},
    draws::{checked_uniform, DrawSource},
    observation::Observation,
    world::State,
    Checked, Phase, Setup, WorkCounts,
};
use crate::{
    config::FieldError,
    foraging::{Departure, FindRecord},
};
fn error(message: &str) -> Vec<FieldError> {
    vec![FieldError::new("action", message)]
}
pub(super) fn apply(
    setup: &Setup,
    state: &mut State,
    index: usize,
    decision: Decision,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<()> {
    let previous = state
        .agents
        .get(index)
        .ok_or_else(|| error("unknown worker"))?;
    if previous.id != decision.agent.id
        || previous.pos != decision.agent.pos
        || observation.origin != previous.pos
    {
        return Err(error("decision must belong to current worker"));
    }
    let moving_phase = previous.phase;
    state.agents[index] = decision.agent;
    let mut work = WorkCounts {
        opportunities: 1,
        ..Default::default()
    };
    match decision.action {
        Action::Move(to) => {
            let agent = &state.agents[index];
            if !agent.pos.neighbors(setup.width, setup.height).contains(&to)
                || !setup.open.contains(&to)
                || state.agents.iter().filter(|a| a.pos == to).count() >= 2
            {
                return Err(error("move must reach an unfilled open cardinal neighbor"));
            }
            work.moves = 1;
            match moving_phase {
                Phase::Departing => work.departure_moves = 1,
                Phase::Searching => work.search_moves = 1,
                Phase::Returning => {
                    if agent.cargo.is_some() {
                        work.loaded_return_moves = 1
                    } else {
                        work.empty_return_moves = 1
                    }
                }
            }
            state.agents[index].pos = to;
        }
        Action::Pickup => {
            let agent = &mut state.agents[index];
            if moving_phase != Phase::Searching
                || agent.cargo.is_some()
                || !observation
                    .cells
                    .iter()
                    .any(|c| c.pos == agent.pos && c.open && c.food)
            {
                return Err(error(
                    "pickup requires current food and empty searching hands",
                ));
            }
            let count = observation
                .cells
                .iter()
                .filter(|c| c.open && c.food)
                .count() as u32;
            let find = FindRecord {
                site: setup.site(agent.pos)?,
                count,
            };
            let cargo = state
                .ledger
                .claim(agent.pos, agent.id)?
                .ok_or_else(|| error("observed current food must remain available"))?;
            agent.cargo = Some(cargo);
            agent.find = Some(find);
            agent.phase = Phase::Returning;
            agent.site = None;
            agent.frontier = None;
            work.pickups = 1;
            state.first_pickup_tick.get_or_insert(state.tick);
        }
        Action::Deposit => {
            let agent = &state.agents[index];
            if moving_phase != Phase::Returning
                || !setup.nest.contains(&agent.pos)
                || agent.find.is_none()
            {
                return Err(error(
                    "deposit requires returning in nest with successful find",
                ));
            }
            let cargo = agent.cargo.ok_or_else(|| error("deposit requires cargo"))?;
            state.ledger.deposit(cargo, agent.id)?;
            work.deposits = 1;
            state.first_delivery_tick.get_or_insert(state.tick);
            let inventory = state.ledger.inventory();
            if inventory.initial > 0 && inventory.delivered == inventory.initial {
                state.all_delivered_tick.get_or_insert(state.tick);
            }
        }
        Action::Wait(reason) => {
            work.waits = 1;
            match reason {
                WaitReason::Transition => work.transition_waits = 1,
                WaitReason::Congestion => work.congestion_waits = 1,
                WaitReason::NoNeighbor => work.no_neighbor_waits = 1,
                WaitReason::EmptyArrival => {
                    let agent = &state.agents[index];
                    if moving_phase != Phase::Returning
                        || agent.cargo.is_some()
                        || !setup.nest.contains(&agent.pos)
                    {
                        return Err(error(
                            "empty arrival requires returning in nest with empty hands",
                        ));
                    }
                    work.empty_arrival_waits = 1;
                    work.empty_returns = 1;
                }
            }
        }
    }
    if matches!(
        decision.action,
        Action::Deposit | Action::Wait(WaitReason::EmptyArrival)
    ) {
        let find = if decision.action == Action::Deposit {
            state.agents[index].find
        } else {
            None
        };
        let arrival = [
            checked_uniform(draws)?,
            checked_uniform(draws)?,
            checked_uniform(draws)?,
        ];
        let advice = state.server.arrive(
            &setup.parameters,
            state.tick,
            state.ledger.inventory().initial,
            find,
            arrival,
        )?;
        work.publications = u64::from(advice.published);
        let site = match advice.departure {
            Departure::SiteFidelity { site } => {
                work.fidelity_departures = 1;
                Some(setup.position(site)?)
            }
            Departure::Recruitment { site, .. } => {
                work.recruited_departures = 1;
                Some(setup.position(site)?)
            }
            Departure::Uninformed => {
                work.uninformed_departures = 1;
                None
            }
        };
        let agent = &mut state.agents[index];
        agent.site = site;
        agent.frontier = None;
        agent.find = None;
        agent.cargo = None;
        agent.phase = Phase::Departing;
    }
    state.agents[index].work.checked_include(&work)
}
