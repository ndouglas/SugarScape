use super::{
    actions::apply,
    decision::{decide, Policy},
    knowledge::CellKnowledge,
    observation::observe,
    Cargo, FoodPhase, WorkCounts,
};
use super::{draws::DrawSource, world::State, Checked, Pos, Setup};
use crate::config::FieldError;
use std::collections::BTreeMap;
pub(super) fn occupancy(setup: &State) -> BTreeMap<Pos, u32> {
    let mut cells = BTreeMap::new();
    for agent in &setup.agents {
        *cells.entry(agent.pos).or_insert(0) += 1;
    }
    cells
}
fn invalid(field: impl Into<String>, message: &str) -> Vec<FieldError> {
    vec![FieldError::new(field, message)]
}
pub(super) fn check(setup: &Setup, state: &State) -> Checked<()> {
    if state.agents.len() != setup.workers.len() {
        return Err(invalid("agents", "count differs from setup"));
    }
    if state.tick > 7200 || u64::from(state.tick) * state.agents.len() as u64 > 1_000_000 {
        return Err(invalid(
            "budget",
            "maximum cumulative ticks/opportunities exceeded",
        ));
    }
    if occupancy(state).values().any(|&n| n > 2) {
        return Err(invalid("occupancy", "each cell admits at most two workers"));
    }
    state.terrain.check()?;
    state.food.check(&state.terrain, &state.agents)?;
    state.spoil.check(&state.terrain, &state.agents)?;
    state.access.check(setup, &state.terrain, &state.food)?;
    for record in state.spoil.views() {
        if record.creator as usize >= state.agents.len()
            || record.born_tick > state.tick
            || matches!(record.state,super::SpoilState::Disposed {tick} if tick>state.tick)
        {
            return Err(invalid(
                format!("spoil[{}]", record.id),
                "creator and birth/disposal ticks must belong to this world",
            ));
        }
    }
    // Researcher-only indexed membership, never passed to decisions/navigation.
    let size = (setup.width * setup.height) as usize;
    let index = |p: Pos| (p.y * setup.width + p.x) as usize;
    let mut open = vec![false; size];
    let mut mask = vec![false; size];
    for p in state.terrain.open_positions() {
        open[index(p)] = true;
    }
    for &p in &setup.diggable {
        setup.site(p)?;
        mask[index(p)] = true;
    }
    let mut work = WorkCounts::default();
    for (i, agent) in state.agents.iter().enumerate() {
        let field = format!("agents[{i}]");
        if agent.id as usize != i
            || !setup.contains(agent.pos)
            || !open[index(agent.pos)]
            || agent.map.dimensions() != (setup.width, setup.height)
        {
            return Err(invalid(
                &field,
                "identity, position or map dimensions disagree",
            ));
        }
        if agent.map.kind(agent.pos)? != CellKnowledge::KnownOpen {
            return Err(invalid(
                &field,
                "current position must be privately known open",
            ));
        }
        let (known_open, known_solid, _) = agent.map.counts();
        if u64::from(known_open) + u64::from(known_solid) != agent.compute.cells_learned {
            return Err(invalid(
                &field,
                "first classifications must equal known cells",
            ));
        }
        if agent.compute.dig_confirmations != agent.work.digs
            || agent
                .compute
                .observed_revisions
                .checked_add(agent.compute.dig_confirmations)
                .is_none_or(|n| n > agent.compute.cells_learned)
            || agent.work.spoil_hauls != agent.work.digs
        {
            return Err(invalid(
                &field,
                "revision/confirmation and excavation accounting disagree",
            ));
        }
        for cell in agent.map.known() {
            let j = index(cell.pos);
            let valid = match cell.kind {
                CellKnowledge::Unknown => false,
                CellKnowledge::KnownOpen => open[j],
                CellKnowledge::KnownSolid { diggable } => {
                    diggable == mask[j] && (diggable || !open[j])
                }
            };
            if !valid {
                return Err(invalid(
                    &field,
                    "private classification conflicts with immutable substrate or monotone terrain",
                ));
            }
        }
        match agent.cargo {
            Some(Cargo::Food(id)) => {
                if agent.phase != FoodPhase::Returning || agent.find.is_none() {
                    return Err(invalid(
                        &field,
                        "Food cargo requires Returning and frozen find",
                    ));
                }
                let find = agent.find.expect("checked");
                let pos = setup.position(find.site)?;
                if find.count == 0
                    || find.count > 5
                    || !state
                        .food
                        .views()
                        .iter()
                        .any(|v| v.resource.id == id && v.resource.pos == pos)
                {
                    return Err(invalid(
                        &field,
                        "find must match carried food origin and local count [1,5]",
                    ));
                }
            }
            Some(Cargo::Spoil(_)) => {
                if agent.find.is_some() || agent.phase == FoodPhase::Returning {
                    return Err(invalid(
                        &field,
                        "Spoil pauses empty Departing/Searching without a food find",
                    ));
                }
            }
            None => {
                if agent.find.is_some() {
                    return Err(invalid(
                        &field,
                        "empty hands cannot retain a successful find",
                    ));
                }
            }
        }
        for target in [agent.site, agent.frontier, agent.face]
            .into_iter()
            .flatten()
        {
            setup.site(target)?;
        }
        if agent.phase == FoodPhase::Returning
            && (agent.site.is_some() || agent.frontier.is_some() || agent.face.is_some())
        {
            return Err(invalid(&field, "food Returning must clear targets"));
        }
        if agent.phase == FoodPhase::Searching && agent.site.is_some() {
            return Err(invalid(&field, "Searching must clear informed site"));
        }
        if agent.frontier.is_some() && agent.face.is_some() {
            return Err(invalid(
                &field,
                "open frontier and dig face cannot be committed together",
            ));
        }
        if agent
            .frontier
            .is_some_and(|p| agent.map.kind(p).ok() != Some(CellKnowledge::KnownOpen))
        {
            return Err(invalid(&field, "frontier must be privately known open"));
        }
        if agent.face.is_some_and(|p| {
            agent.map.kind(p).ok() != Some(CellKnowledge::KnownSolid { diggable: true })
        }) || agent.cargo.is_some() && agent.face.is_some()
        {
            return Err(invalid(
                &field,
                "face requires empty hands and remembered diggable solid",
            ));
        }
        agent.work.check().map_err(|e| context(e, field.clone()))?;
        work.checked_include(&agent.work)
            .map_err(|e| context(e, field.clone()))?;
    }
    work.check()?;
    let food = state.food.inventory();
    let spoil = state.spoil.inventory();
    if work.opportunities > 1_000_000
        || work.publications > u64::from(food.delivered)
        || work.digs != u64::from(spoil.excavated)
        || work.disposals != u64::from(spoil.disposed)
        || work.pickups != u64::from(food.carried + food.delivered)
        || work.deposits != u64::from(food.delivered)
    {
        return Err(invalid(
            "work",
            "physical counters must agree with material inventories and budgets",
        ));
    }
    let records = state.server.views(&setup.parameters, state.tick)?;
    if (records.len() as u64).checked_add(state.server.expired()) != Some(work.publications) {
        return Err(invalid(
            "server",
            "retained plus expired must equal sole-food-deposit publications",
        ));
    }
    for r in records {
        setup.position(r.site)?;
    }
    let access = state.access.summary()?;
    if access.accessible > food.initial {
        return Err(invalid("access", "accessible inventory exceeds food"));
    }
    Ok(())
}
fn context(errors: Vec<FieldError>, prefix: String) -> Vec<FieldError> {
    errors
        .into_iter()
        .map(|e| FieldError::new(format!("{prefix}.{}", e.field), e.message))
        .collect()
}
/// Scratch-only; caller commits the whole state and RNG together.
pub(super) fn advance(
    setup: &Setup,
    state: &mut State,
    draws: &mut impl DrawSource,
) -> Checked<()> {
    let next = state
        .tick
        .checked_add(1)
        .ok_or_else(|| invalid("tick", "counter overflow"))?;
    if next > 7200 || u64::from(next) * state.agents.len() as u64 > 1_000_000 {
        return Err(invalid(
            format!("tick[{}].budget", state.tick),
            "maximum cumulative ticks/opportunities exceeded",
        ));
    }
    check(setup, state).map_err(|e| context(e, format!("tick[{}]", state.tick)))?;
    let p = Policy::from(setup);
    for i in 0..state.agents.len() {
        let material = match state.agents[i].cargo {
            Some(Cargo::Food(_)) => "food",
            Some(Cargo::Spoil(_)) => "spoil",
            None => "empty",
        };
        let mut opportunity = || -> Checked<()> {
            let observation = observe(
                &state.terrain,
                state.agents[i].pos,
                &occupancy(state),
                &state.food.available(),
            )?;
            let decision = decide(&p, &state.agents[i], &observation, draws)?;
            apply(setup, state, i, decision, &observation, draws)
        };
        opportunity()
            .map_err(|e| context(e, format!("tick[{}].agents[{i}].{material}", state.tick)))?;
    }
    check(setup, state).map_err(|e| context(e, format!("tick[{}]", state.tick)))?;
    state.tick = next;
    Ok(())
}
