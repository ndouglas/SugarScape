//! One unchanged native P3 run, then bounded allowlisted boundary projections.
use super::projection;
use crate::{
    browser_experiments::{
        error, record, wire, Checkpoint, EpisodeKind, EpisodeRecord, FieldError, Input, Semantics,
        EPISODE_VERSION, MAX_EPISODE_BYTES,
    },
    minds::protection::{runner, state::ProtectionState},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn protection(
    state: &Option<ProtectionState>,
    tick: u64,
    exposure_span: u64,
) -> Result<Value, Vec<FieldError>> {
    let Some(state) = state else {
        return Ok(Value::Null);
    };
    if state.sources.len() > 2 || state.exposure.entries.len() > 81 {
        return Err(error(
            "episode.protection",
            "native memory exceeds fixed caching profile",
        ));
    }
    let sources = state
        .sources
        .iter()
        .map(|(&site, source)| {
            if source.tick > tick {
                return Err(error(
                    "episode.protection",
                    "source evidence is ahead of boundary",
                ));
            }
            Ok(json!({
                "site": site,
                "pos": projection::site_position(site)?,
                "tick": source.tick.to_string(),
                "age": tick.saturating_sub(source.tick).to_string(),
                "expired": tick.saturating_sub(source.tick)>exposure_span,
                "expiry_semantics": "retained_memory_eligibility_at_current_boundary",
                "initial_amount": source.initial_amount,
                "attempted": source.attempted,
            }))
        })
        .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
    let exposure = state
        .exposure
        .entries
        .iter()
        .map(|(&site, exposure)| {
            if exposure.tick > tick {
                return Err(error(
                    "episode.protection",
                    "exposure evidence is ahead of boundary",
                ));
            }
            Ok(json!({
                "site": site,
                "pos": projection::site_position(site)?,
                "tick": exposure.tick.to_string(),
                "age": tick.saturating_sub(exposure.tick).to_string(),
                "expired": tick.saturating_sub(exposure.tick)>exposure_span,
                "expiry_semantics": "retained_memory_eligibility_at_current_boundary",
                "exposed": exposure.exposed,
                "kind": "perceived_exposure_memory",
            }))
        })
        .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
    let intent = match &state.intent {
        None => Value::Null,
        Some(i) => {
            json!({"source":i.source,"source_pos":projection::site_position(i.source)?,"destination":i.destination,"amount":i.amount,"stage":i.stage})
        }
    };
    Ok(
        json!({"exposure_span":exposure_span.to_string(),"sources":sources,"exposure":{"entries":exposure},"intent":intent}),
    )
}
fn checkpoint(
    frame: &runner::FrameRecord,
    initial: bool,
    terminal: bool,
    exposure_span: u64,
) -> Result<Checkpoint, Vec<FieldError>> {
    if frame.roles.len() > 2 || frame.actions.len() > 2 || frame.deaths.len() > 2 || frame.tick > 64
    {
        return Err(error(
            "episode.frames",
            "native frame exceeds fixed caching profile",
        ));
    }
    let mut local = BTreeMap::new();
    let mut agents = Vec::new();
    for role in &frame.roles {
        if role.seen.len() > 162 {
            return Err(error(
                "episode.seen",
                "native memory exceeds fixed caching profile",
            ));
        }
        let agent = projection::agent(role.id, role.pos, role.holdings, &role.caches)?;
        agents.push(agent.clone());
        let seen = role
            .seen
            .iter()
            .map(|s| projection::seen(s.site, s.owner, s.amount, s.tick, frame.tick))
            .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
        let own_actions = frame
            .actions
            .iter()
            .filter(|action| action.id == role.id)
            .map(|action| {
                json!({
                    "phase": action.phase,
                    "action": action.action,
                    "harvest": action.harvest,
                    "metabolic_demand": action.metabolic_demand,
                    "metabolic_consumed": action.metabolic_consumed,
                    "gross_dug": action.gross_dug,
                    "gross_buried": action.gross_buried,
                    "burial_cost": action.burial_cost,
                    "source": action.source,
                    "target": action.target,
                    "perceived_exposure": action.perceived_exposure,
                })
            })
            .collect::<Vec<_>>();
        local.insert(
            role.id.to_string(),
            json!({
                "agent": agent,
                "seen": seen,
                "own_actions": own_actions,
                "protection": protection(&role.protection,frame.tick,exposure_span)?,
                "availability": {
                    "agent": "captured_living_role",
                    "protection": if role.protection.is_some() {"captured"} else {"not_applicable"},
                    "sensory_field": "not_recorded",
                    "research_diagnostics": "withheld",
                    "seen": "retained_memory_not_current_stock",
                },
            }),
        );
    }
    Ok(Checkpoint {
        index: 0,
        clock: projection::clock(frame.tick),
        kind: if initial {
            "caching_initial"
        } else if terminal {
            "caching_terminal"
        } else {
            "caching_step"
        }
        .into(),
        public: projection::public("protection", terminal),
        local,
        researcher: Some(
            json!({"frame":wire::lossless_value(frame)?,"display":{"kind":"derived_captured_role_state","agents":agents,"physical_stock":"not_recorded"}}),
        ),
    })
}
pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    let Input::ProtectionRecaching { lab, seed } = input else {
        return Err(error(
            "study",
            "protection adapter requires protection_recaching",
        ));
    };
    super::validate_input(input)?;
    let native = runner::run_episode(lab.to_core()?, seed.value(), true)
        .map_err(|message| error("episode.protection", message))?;
    if !native.fixture_errors.is_empty() {
        return Err(error(
            "episode.protection.fixture",
            format!(
                "native fixture failed: {}",
                native.fixture_errors.join("; ")
            ),
        ));
    }
    if !native.ledger_errors.is_empty() {
        return Err(error(
            "episode.protection.ledger",
            format!("native ledger failed: {}", native.ledger_errors.join("; ")),
        ));
    }
    if native.frames.is_empty() || native.frames.len() > 65 || native.completed_ticks > 64 {
        return Err(error(
            "episode.frames",
            "native record exceeds fixed caching profile",
        ));
    }
    // Check native size before allocating its lossless wire copy. Exact charging
    // then includes each retained payload and researcher/local checkpoint occurrence.
    record::serialized_size(&native, MAX_EPISODE_BYTES, "episode")?;
    let payload = json!({"native":wire::lossless_value(&native)?,"capture":{"clock_semantics":"native_completed_step_boundary","requested_ticks":"64","action_interval_semantics":"actions in boundary tick t belong to [t-1,t); initial has no actions; memory retains original observation tick"}});
    let mut budget = crate::browser_experiments::spatial::budget::CaptureBudget::new();
    budget.charge(&payload)?;
    let mut checkpoints = Vec::new();
    for (index, frame) in native.frames.iter().enumerate() {
        let at = checkpoint(
            frame,
            index == 0,
            index + 1 == native.frames.len(),
            native.lab.exposure_span,
        )?;
        budget.charge(&at)?;
        record::push_checkpoint(&mut checkpoints, at)?;
    }
    let record = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: input.study(),
        rules_identity: super::rules_identity(input.study())?,
        input: serde_json::to_value(input).map_err(|e| error("input", e.to_string()))?,
        semantics: Semantics::Trajectory,
        checkpoints,
        payload,
    };
    record::check_bounds(&record)?;
    Ok(record)
}
