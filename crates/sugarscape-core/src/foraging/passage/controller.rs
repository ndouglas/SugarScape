//! Ordered observations are rebuilt after each physical candidate action.
use super::{
    actions::apply,
    decision::{decide, Policy},
    draws::DrawSource,
    knowledge::CellKnowledge,
    observation::observe,
    world::State,
    Checked, Phase, Pos, Setup, WorkCounts,
};
use crate::config::FieldError;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn occupancy(state: &State) -> BTreeMap<Pos, u32> {
    let mut cells = BTreeMap::new();
    for agent in &state.agents {
        *cells.entry(agent.pos).or_insert(0) += 1;
    }
    cells
}
pub(super) fn check(setup: &Setup, state: &State) -> Checked<()> {
    let invalid = |field: &str, message: &str| vec![FieldError::new(field, message)];
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
    state.ledger.check(&state.agents)?;
    // Researcher invariant index only; workers never receive authoritative topology.
    let open_cells: BTreeSet<_> = setup.open.iter().copied().collect();
    let mut work = WorkCounts::default();
    for (index, agent) in state.agents.iter().enumerate() {
        let field = format!("agents[{index}]");
        if agent.id as usize != index
            || !open_cells.contains(&agent.pos)
            || agent.map.dimensions() != (setup.width, setup.height)
        {
            return Err(invalid(
                &field,
                "identity, physical position or map dimensions disagree with setup",
            ));
        }
        if agent
            .map
            .kind(agent.pos)
            .map_err(|e| context(e, field.clone()))?
            != CellKnowledge::KnownOpen
        {
            return Err(invalid(
                &field,
                "current position must be privately known open",
            ));
        }
        let (open, solid) = agent.map.counts();
        if open + solid > setup.width * setup.height {
            return Err(invalid(&field, "private classifications exceed grid"));
        }
        for cell in agent.map.known() {
            if (cell.kind == CellKnowledge::KnownOpen) != open_cells.contains(&cell.pos) {
                return Err(invalid(
                    &field,
                    "known classification conflicts with authoritative fixed topology",
                ));
            }
        }
        if agent.cargo.is_some() != agent.find.is_some()
            || (agent.cargo.is_some() && agent.phase != Phase::Returning)
        {
            return Err(invalid(
                &field,
                "cargo and successful find require loaded Returning",
            ));
        }
        if let Some(find) = agent.find {
            let pos = setup
                .position(find.site)
                .map_err(|e| context(e, format!("{field}.find")))?;
            if find.count == 0
                || find.count > 5
                || !state
                    .ledger
                    .views()
                    .iter()
                    .any(|v| Some(v.resource.id) == agent.cargo && v.resource.pos == pos)
            {
                return Err(invalid(
                    &field,
                    "find requires its cargo pickup site and a local count in [1,5]",
                ));
            }
        }
        for target in [agent.site, agent.frontier].into_iter().flatten() {
            setup
                .site(target)
                .map_err(|e| context(e, format!("{field}.target")))?;
        }
        if agent.phase == Phase::Returning && (agent.site.is_some() || agent.frontier.is_some()) {
            return Err(invalid(&field, "Returning must clear departure targets"));
        }
        if agent
            .frontier
            .is_some_and(|p| agent.map.kind(p).ok() != Some(CellKnowledge::KnownOpen))
        {
            return Err(invalid(
                &field,
                "frontier travel target must be privately known open",
            ));
        }
        agent.work.check().map_err(|e| context(e, field.clone()))?;
        work.checked_include(&agent.work)
            .map_err(|e| context(e, field.clone()))?;
    }
    work.check()?;
    if work.opportunities > 1_000_000
        || work.publications > u64::from(state.ledger.inventory().delivered)
    {
        return Err(invalid(
            "work",
            "opportunity or sole-deposit publication budget exceeded",
        ));
    }
    let records = state.server.views(&setup.parameters, state.tick)?;
    if records.len() as u64 + state.server.expired() != work.publications {
        return Err(invalid(
            "server",
            "retained plus expired records must equal publications",
        ));
    }
    for record in records {
        setup.position(record.site)?;
    }
    Ok(())
}
fn context(errors: Vec<FieldError>, prefix: String) -> Vec<FieldError> {
    errors
        .into_iter()
        .map(|e| FieldError::new(format!("{prefix}.{}", e.field), e.message))
        .collect()
}
/// Mutates scratch state only; callers commit state and draws together after success.
pub(super) fn advance(
    setup: &Setup,
    state: &mut State,
    draws: &mut impl DrawSource,
) -> Checked<()> {
    let next = state
        .tick
        .checked_add(1)
        .ok_or_else(|| vec![FieldError::new("tick", "counter overflow")])?;
    if next > 7200 || u64::from(next) * state.agents.len() as u64 > 1_000_000 {
        return Err(vec![FieldError::new(
            format!("tick[{}].budget", state.tick),
            "maximum cumulative ticks/opportunities exceeded",
        )]);
    }
    check(setup, state).map_err(|e| context(e, format!("tick[{}]", state.tick)))?;
    let policy = Policy::from(setup);
    for index in 0..state.agents.len() {
        let mut opportunity = || -> Checked<()> {
            let observation = observe(
                setup,
                state.agents[index].pos,
                &occupancy(state),
                &state.ledger.available(),
            )?;
            let decision = decide(&policy, &state.agents[index], &observation, draws)?;
            apply(setup, state, index, decision, &observation, draws)
        };
        opportunity().map_err(|e| context(e, format!("tick[{}].agents[{index}]", state.tick)))?;
    }
    check(setup, state).map_err(|e| context(e, format!("tick[{}]", state.tick)))?;
    state.tick = next;
    Ok(())
}
