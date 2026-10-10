//! One unchanged native P4 run, then bounded allowlisted boundary projections.
use super::projection;
use crate::{
    browser_experiments::{
        error, record, wire, Checkpoint, EpisodeKind, EpisodeRecord, FieldError, Input, Semantics,
        EPISODE_VERSION, MAX_EPISODE_BYTES,
    },
    minds::deception::{
        records::{EpisodeFailure, FrameRecord},
        runner,
    },
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(crate) fn checkpoint(
    frame: &FrameRecord,
    initial: bool,
    terminal: bool,
) -> Result<Checkpoint, Vec<FieldError>> {
    if frame.roles.len() > 2
        || frame.actions.len() > 2
        || frame.deaths.len() > 2
        || frame.stocks.len() > 81
        || frame.observations.len() > 2
        || frame.tick > 64
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
        let own_actions = frame.actions.iter().filter(|a| a.actor == role.id).map(|a| json!({
            "phase":a.phase,"action":a.action,"target":a.target,"harvest":a.harvest,"dug":a.dug,"buried":a.buried,"effort":a.effort,"metabolic_demand":a.metabolic_demand,"metabolic_consumed":a.metabolic_consumed,"pos":a.pos,"walk_outcome":a.walk_outcome
        })).collect::<Vec<_>>();
        let received = frame
            .observations
            .iter()
            .filter(|o| o.receiver == role.id)
            .map(|o| {
                if o.public.tick >= frame.tick || !(1..=2).contains(&o.public.actor) {
                    return Err(error(
                        "episode.observations",
                        "observation clock or actor exceeds completed boundary",
                    ));
                }
                projection::site_position(o.public.site)?;
                wire::lossless_value(&o.public)
            })
            .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
        let sender = role.sender.as_ref().map_or(Value::Null, |s| json!({"stage":s.stage,"attempted":s.attempted,"pending_departure":s.pending_departure}));
        local.insert(role.id.to_string(),json!({"agent":agent,"seen":seen,"own_actions":own_actions,"sender":sender,"received_observations":received,"availability":{
            "agent":"captured_living_role","sender":if role.sender.is_some(){"captured"}else{"not_applicable"},"sensory_field":"not_recorded","research_diagnostics":"withheld","decision_evidence":"committed_actions_and_retained_memory_only","seen":"retained_memory_not_current_stock","received_observations":"native_receiver_public_channel"
        }}));
    }
    let stocks = frame
        .stocks
        .iter()
        .map(|s| {
            Ok(json!({"site":s.site,"pos":projection::site_position(s.site)?,"amount":s.amount}))
        })
        .collect::<Result<Vec<_>, Vec<FieldError>>>()?;
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
        public: projection::public("deception", terminal),
        local,
        researcher: Some(
            json!({"frame":wire::lossless_value(frame)?,"display":{"kind":"derived_captured_role_state","agents":agents,"stocks":stocks}}),
        ),
    })
}
fn failure(failure: EpisodeFailure) -> Vec<FieldError> {
    let context = failure.partial.as_ref().map_or_else(|| "before initial boundary".into(), |partial| format!("after {} completed ticks, {} partial frames; fixture errors: {}; ledger errors: {}",partial.completed_ticks,partial.frames.len(),partial.fixture_errors.join("; "),partial.ledger_errors.join("; ")));
    error(
        "episode.deception",
        format!("native episode failed {context}: {}", failure.message),
    )
}
pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    let Input::DeceptionGestures { lab, seed } = input else {
        return Err(error(
            "study",
            "deception adapter requires deception_gestures",
        ));
    };
    super::validate_input(input)?;
    let native = runner::run_episode(lab.to_core()?, seed.value(), true).map_err(failure)?;
    if !native.fixture_errors.is_empty() {
        return Err(error(
            "episode.deception.fixture",
            format!(
                "native fixture failed: {}",
                native.fixture_errors.join("; ")
            ),
        ));
    }
    if !native.ledger_errors.is_empty() {
        return Err(error(
            "episode.deception.ledger",
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
        let at = checkpoint(frame, index == 0, index + 1 == native.frames.len())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caching_deception_failure_context_never_returns_partial_success() {
        let native = runner::run_episode(Default::default(), 7, true).unwrap();
        let errors = failure(EpisodeFailure {
            message: "sink refused capture".into(),
            partial: Some(native),
        });
        assert_eq!(errors[0].field, "episode.deception");
        assert!(errors[0]
            .message
            .contains("64 completed ticks, 65 partial frames"));
        assert!(errors[0].message.contains("sink refused capture"));
        let errors = failure(EpisodeFailure {
            message: "invalid rig".into(),
            partial: None,
        });
        assert_eq!(
            errors[0].message,
            "native episode failed before initial boundary: invalid rig"
        );
    }
}
