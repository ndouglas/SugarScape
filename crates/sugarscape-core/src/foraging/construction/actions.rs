//! Paid actions mutate only agent candidate tick; cargo transactions remain separate.
use super::{
    access::EventContext,
    decision::{Action, Decision, WaitReason},
    draws::{checked_uniform, DrawSource},
    metrics::ComputeCounts,
    observation::Observation,
    setup::neighbors,
    world::State,
    Cargo, Checked, FoodPhase, Mode, Setup, WorkCounts,
};
use crate::{
    config::FieldError,
    foraging::{Departure, FindRecord},
};
fn error(message: &str) -> Vec<FieldError> {
    vec![FieldError::new("action", message)]
}
fn event(state: &State, index: usize) -> Checked<EventContext> {
    let opportunity = state.agents.iter().try_fold(0u64, |n, agent| {
        n.checked_add(agent.work.opportunities)
            .ok_or_else(|| error("opportunity sum overflow"))
    })?;
    Ok(EventContext {
        tick: state.tick,
        opportunity,
        worker: state.agents[index].id,
        excavated: state.terrain.counts().excavated,
        spoil_disposed: state.spoil.inventory().disposed,
        food_delivered: state.food.inventory().delivered,
    })
}
pub(super) fn apply(
    setup: &Setup,
    state: &mut State,
    index: usize,
    decision: Decision,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<()> {
    let label = match decision.action {
        Action::Dig(_) => "dig.spoil.food",
        Action::PickupFood | Action::DepositFood => "food",
        Action::DisposeSpoil => "spoil",
        Action::Move(_) => "move",
        Action::Wait(_) => "wait",
    };
    transaction(setup, state, index, decision, observation, draws).map_err(|errors| {
        errors
            .into_iter()
            .map(|e| FieldError::new(format!("{label}.{}", e.field), e.message))
            .collect()
    })
}
fn transaction(
    setup: &Setup,
    state: &mut State,
    index: usize,
    decision: Decision,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<()> {
    observation.validate(setup.width, setup.height)?;
    let previous = state
        .agents
        .get(index)
        .ok_or_else(|| error("unknown worker"))?;
    if previous.id != decision.agent.id
        || previous.pos != decision.agent.pos
        || observation.origin != previous.pos
        || previous.cargo != decision.agent.cargo
    {
        return Err(error(
            "decision must belong to the current worker with unchanged hands",
        ));
    }
    let mode = previous.mode();
    state.agents[index] = decision.agent;
    let mut work = WorkCounts {
        opportunities: 1,
        ..Default::default()
    };
    match decision.action {
        Action::Move(to) => {
            let agent = &state.agents[index];
            if !neighbors(agent.pos, setup.width, setup.height).contains(&to)
                || !state.terrain.is_open(to)?
                || state.agents.iter().filter(|agent| agent.pos == to).count() >= 2
            {
                return Err(error("move must reach an unfilled open cardinal neighbor"));
            }
            work.moves = 1;
            match mode {
                Mode::Departing => work.departure_moves = 1,
                Mode::Searching => work.search_moves = 1,
                Mode::EmptyReturning => work.empty_return_moves = 1,
                Mode::FoodReturning => work.food_moves = 1,
                Mode::SpoilHauling => work.spoil_moves = 1,
            }
            state.agents[index].pos = to;
        }
        Action::Dig(target) => {
            let agent = &state.agents[index];
            if !matches!(mode, Mode::Departing | Mode::Searching)
                || agent.cargo.is_some()
                || agent.find.is_some()
                || !neighbors(agent.pos, setup.width, setup.height).contains(&target)
                || !state.terrain.is_diggable(target)?
                || !observation
                    .cells
                    .iter()
                    .any(|c| c.pos == target && !c.open && c.diggable)
            {
                return Err(error(
                    "Dig requires empty hands and a fresh adjacent eligible solid face",
                ));
            }
            state.terrain.dig(target)?;
            let id = state
                .spoil
                .spawn(target, state.agents[index].id, state.tick)?;
            state.food.expose(target)?;
            let agent = &mut state.agents[index];
            let delta = agent.map.confirm_dig(agent.pos, target)?;
            agent.compute.checked_include(&ComputeCounts {
                dig_confirmations: delta.dig_confirmations,
                ..Default::default()
            })?;
            agent.cargo = Some(Cargo::Spoil(id));
            agent.frontier = None;
            agent.face = None;
            work.digs = 1;
            work.spoil_hauls = 1;
        }
        Action::PickupFood => {
            let agent = &mut state.agents[index];
            if mode != Mode::Searching
                || agent.cargo.is_some()
                || !observation
                    .cells
                    .iter()
                    .any(|c| c.pos == agent.pos && c.open && c.food)
            {
                return Err(error(
                    "pickup requires current Available food and empty searching hands",
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
            let id = state
                .food
                .claim(agent.pos, agent.id)?
                .ok_or_else(|| error("observed food must remain Available"))?;
            agent.cargo = Some(Cargo::Food(id));
            agent.find = Some(find);
            agent.phase = FoodPhase::Returning;
            agent.site = None;
            agent.frontier = None;
            agent.face = None;
            work.pickups = 1;
            state.milestones.first_pickup_tick.get_or_insert(state.tick);
        }
        Action::DepositFood => {
            let agent = &state.agents[index];
            let Some(Cargo::Food(id)) = agent.cargo else {
                return Err(error("deposit requires tagged Food cargo"));
            };
            if mode != Mode::FoodReturning
                || !setup.nest.contains(&agent.pos)
                || agent.find.is_none()
            {
                return Err(error(
                    "deposit requires nest arrival and successful food find",
                ));
            }
            state.food.deposit(id, agent.id)?;
            work.deposits = 1;
            state
                .milestones
                .first_delivery_tick
                .get_or_insert(state.tick);
            let inv = state.food.inventory();
            if inv.initial > 0 && inv.delivered == inv.initial {
                state
                    .milestones
                    .all_food_delivered_tick
                    .get_or_insert(state.tick);
            }
        }
        Action::DisposeSpoil => {
            let agent = &state.agents[index];
            let Some(Cargo::Spoil(id)) = agent.cargo else {
                return Err(error("disposal requires tagged Spoil cargo"));
            };
            if mode != Mode::SpoilHauling || agent.pos != setup.waste {
                return Err(error("spoil disposal requires its outlet"));
            }
            state.spoil.dispose(id, agent.id, state.tick)?;
            let agent = &mut state.agents[index];
            agent.cargo = None;
            agent.frontier = None;
            agent.face = None;
            work.disposals = 1;
        }
        Action::Wait(reason) => {
            work.waits = 1;
            match reason {
                WaitReason::Transition => {
                    if !matches!(mode, Mode::Departing | Mode::Searching) {
                        return Err(error("transition requires empty food intent"));
                    }
                    work.transition_waits = 1;
                }
                WaitReason::NoNeighbor => work.no_neighbor_waits = 1,
                WaitReason::Congestion => match mode {
                    Mode::FoodReturning => work.food_congestion_waits = 1,
                    Mode::SpoilHauling => work.spoil_congestion_waits = 1,
                    _ => work.empty_congestion_waits = 1,
                },
                WaitReason::EmptyArrival => {
                    if mode != Mode::EmptyReturning
                        || !setup.nest.contains(&state.agents[index].pos)
                    {
                        return Err(error("empty arrival requires empty Returning in nest"));
                    }
                    work.empty_arrival_waits = 1;
                    work.empty_returns = 1;
                }
            }
        }
    }
    state.agents[index].work.checked_include(&work)?;
    match decision.action {
        Action::Dig(target) => {
            let context = event(state, index)?;
            let delta = state
                .access
                .after_dig(setup, &state.terrain, &state.food, &context)?;
            let milestone = state.access.milestone(&context, target)?;
            state.milestones.first_excavation.get_or_insert(milestone);
            if state.milestones.first_exposure.is_none() {
                state.milestones.first_exposure = delta.exposure;
            }
            if state.milestones.first_access.is_none() {
                state.milestones.first_access = delta.access;
            }
        }
        Action::DisposeSpoil => {
            let milestone = state.access.milestone(&event(state, index)?, setup.waste)?;
            state.milestones.first_disposal.get_or_insert(milestone);
        }
        Action::DepositFood | Action::Wait(WaitReason::EmptyArrival) => {
            let find = if decision.action == Action::DepositFood {
                state.agents[index].find
            } else {
                None
            };
            let draws = [
                checked_uniform(draws)?,
                checked_uniform(draws)?,
                checked_uniform(draws)?,
            ];
            let advice = state.server.arrive(
                &setup.parameters,
                state.tick,
                state.food.inventory().initial,
                find,
                draws,
            )?;
            let mut trip = WorkCounts {
                publications: u64::from(advice.published),
                ..Default::default()
            };
            let site = match advice.departure {
                Departure::SiteFidelity { site } => {
                    trip.fidelity_departures = 1;
                    Some(setup.position(site)?)
                }
                Departure::Recruitment { site, .. } => {
                    trip.recruited_departures = 1;
                    Some(setup.position(site)?)
                }
                Departure::Uninformed => {
                    trip.uninformed_departures = 1;
                    None
                }
            };
            let agent = &mut state.agents[index];
            agent.work.checked_include(&trip)?;
            agent.site = site;
            agent.frontier = None;
            agent.face = None;
            agent.find = None;
            agent.cargo = None;
            agent.phase = FoodPhase::Departing;
        }
        _ => {}
    }
    Ok(())
}
