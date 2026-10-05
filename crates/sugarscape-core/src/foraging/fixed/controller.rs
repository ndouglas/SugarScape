use super::{
    draws::{checked_uniform, DrawSource},
    movement::{ahead, directed_step, edge_target, search_step, turn},
    state::Agent,
    world::State,
    Phase, Setup,
};
use crate::{
    config::FieldError,
    foraging::{informed_variation, uninformed_variation, Departure},
};
type Checked<T> = Result<T, Vec<FieldError>>;
fn increment(value: &mut u64) -> Checked<()> {
    *value = value
        .checked_add(1)
        .ok_or_else(|| vec![FieldError::new("work", "counter overflow")])?;
    Ok(())
}
fn perform_turn(setup: &Setup, a: &mut Agent, draws: &mut impl DrawSource) -> Checked<()> {
    let variation = if a.informed {
        informed_variation(
            setup.parameters.omega,
            setup.parameters.lambda_informed,
            f64::from(a.informed_turns),
        )?
    } else {
        uninformed_variation(setup.parameters.omega)?
    };
    let (heading, delay) = turn(a.heading, variation, draws)?;
    a.heading = heading;
    a.delay = a
        .delay
        .checked_add(delay)
        .ok_or_else(|| vec![FieldError::new("delay", "counter overflow")])?;
    if a.informed {
        a.informed_turns = a
            .informed_turns
            .checked_add(1)
            .ok_or_else(|| vec![FieldError::new("informed_turns", "counter overflow")])?;
    }
    Ok(())
}
pub(super) fn check(setup: &Setup, state: &State) -> Checked<()> {
    state.ledger.check(&state.agents)?;
    if state.agents.len() != setup.agents as usize {
        return Err(vec![FieldError::new("agents", "count differs from setup")]);
    }
    for a in &state.agents {
        if !setup.contains(a.pos)
            || !setup.contains(a.target)
            || !a.heading.is_finite()
            || !(0.0..std::f64::consts::TAU).contains(&a.heading)
        {
            return Err(vec![FieldError::new(
                format!("agents[{}].geometry", a.id),
                "position/target must be in bounds and heading finite in [0,2*pi)",
            )]);
        }
        if a.id >= setup.agents {
            return Err(vec![FieldError::new(
                format!("agents[{}].id", a.id),
                "identity outside setup",
            )]);
        }
        if a.cargo.is_some() && (a.phase != Phase::Returning || a.find.is_none()) {
            return Err(vec![FieldError::new(
                format!("agents[{}].cargo", a.id),
                "cargo requires returning with a successful find",
            )]);
        }
        if let Some(find) = a.find {
            setup.position(find.site)?;
            if find.count == 0 || find.count > 256 {
                return Err(vec![FieldError::new(
                    format!("agents[{}].find", a.id),
                    "count must be in [1,256]",
                )]);
            }
        }
    }
    Ok(())
}
fn opportunity(
    setup: &Setup,
    state: &mut State,
    index: usize,
    draws: &mut impl DrawSource,
) -> Checked<()> {
    let a = &mut state.agents[index];
    increment(&mut a.work.opportunities)?;
    if a.delay > 0 {
        a.delay -= 1;
        increment(&mut a.work.waits)?;
        return Ok(());
    }
    match a.phase {
        Phase::Departing => {
            let switch = if !a.informed {
                checked_uniform(draws)? < setup.parameters.p_search
            } else {
                false
            };
            if switch || a.pos == a.target {
                increment(&mut a.work.search_switches)?;
                a.phase = Phase::Searching;
                perform_turn(setup, a, draws)?;
            } else {
                a.pos = directed_step(setup, a.pos, a.target, draws)?;
                increment(&mut a.work.directed_moves)?;
            }
        }
        Phase::Searching => {
            if checked_uniform(draws)? < setup.parameters.p_return {
                a.find = None;
                a.target = setup.nest;
                a.phase = Phase::Returning;
                return Ok(());
            }
            a.pos = search_step(setup, a.pos, &mut a.heading, draws)?;
            increment(&mut a.work.search_moves)?;
            perform_turn(setup, a, draws)?;
            if let Some(cell) = ahead(setup, a.pos, a.heading) {
                if let Some((cargo, find)) = state.ledger.claim(setup, cell, a.id)? {
                    a.cargo = Some(cargo);
                    a.find = Some(find);
                    a.delay = 9;
                    a.target = setup.nest;
                    a.phase = Phase::Returning;
                    increment(&mut a.work.pickups)?;
                    state.first_pickup_tick.get_or_insert(state.tick);
                }
            }
        }
        Phase::Returning => {
            if a.pos != setup.nest {
                a.pos = directed_step(setup, a.pos, setup.nest, draws)?;
                increment(&mut a.work.directed_moves)?;
            }
            if a.pos != setup.nest {
                return Ok(());
            }
            let arrival = [
                checked_uniform(draws)?,
                checked_uniform(draws)?,
                checked_uniform(draws)?,
            ];
            let find = if let Some(cargo) = a.cargo {
                state.ledger.deposit(cargo, a.id)?;
                increment(&mut a.work.deliveries)?;
                state.first_delivery_tick.get_or_insert(state.tick);
                let inventory = state.ledger.inventory();
                if inventory.delivered == inventory.initial {
                    state.all_delivered_tick.get_or_insert(state.tick);
                }
                a.find
            } else {
                increment(&mut a.work.empty_returns)?;
                None
            };
            let (choice, published) = state.server.arrive(setup, state.tick, find, arrival)?;
            if published {
                increment(&mut a.work.publications)?;
            }
            a.informed = true;
            a.target = match choice {
                Departure::SiteFidelity { site } => {
                    increment(&mut a.work.fidelity_departures)?;
                    setup.position(site)?
                }
                Departure::Recruitment { site, .. } => {
                    increment(&mut a.work.recruited_departures)?;
                    setup.position(site)?
                }
                Departure::Uninformed => {
                    a.informed = false;
                    increment(&mut a.work.uninformed_departures)?;
                    edge_target(setup, draws)?
                }
            };
            a.cargo = None;
            a.find = None;
            a.informed_turns = 0;
            a.phase = Phase::Departing;
        }
    }
    Ok(())
}
/// Advances scratch state only. World commits a cloned candidate after success.
pub(super) fn advance(
    setup: &Setup,
    state: &mut State,
    draws: &mut impl DrawSource,
) -> Checked<()> {
    let tick = state
        .tick
        .checked_add(1)
        .ok_or_else(|| vec![FieldError::new("tick", "counter overflow")])?;
    if tick > 7200 || u64::from(tick) * u64::from(setup.agents) > 1_000_000 {
        return Err(vec![FieldError::new(
            format!("tick[{}].budget", state.tick),
            "maximum cumulative ticks/opportunities exceeded",
        )]);
    }
    check(setup, state).map_err(|errors| {
        errors
            .into_iter()
            .map(|e| FieldError::new(format!("tick[{}].{}", state.tick, e.field), e.message))
            .collect::<Vec<_>>()
    })?;
    state.agents.sort_by_key(|a| a.id);
    for index in 0..state.agents.len() {
        let id = state.agents[index].id;
        opportunity(setup, state, index, draws).map_err(|errors| {
            errors
                .into_iter()
                .map(|e| {
                    FieldError::new(
                        format!("tick[{}].agents[{id}].{}", state.tick, e.field),
                        e.message,
                    )
                })
                .collect::<Vec<_>>()
        })?;
    }
    check(setup, state).map_err(|errors| {
        errors
            .into_iter()
            .map(|e| FieldError::new(format!("tick[{}].{}", state.tick, e.field), e.message))
            .collect::<Vec<_>>()
    })?;
    state.tick = tick;
    Ok(())
}
