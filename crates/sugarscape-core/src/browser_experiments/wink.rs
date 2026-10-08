//! Request-driven Wink. No privileged engine getters or predicted hidden effects.
use super::{
    catalog, record, wire::lossless_value, Checkpoint, EpisodeKind, EpisodeRecord, FieldError,
    Input, Semantics, StudyId, WinkMode, EPISODE_VERSION,
};
use crate::deduction::{
    self, ActionError, BuiltinController, Controller, Engine, EventContent,
    ExperimentThreatController, ObjectiveTeam, Outcome, TurnRequest, TurnResponse, Visibility,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    let Input::Wink { seed, policy, mode } = input;
    let seed = seed.value();
    let mut engine = Engine::new(deduction::wink_config(6), seed)?;
    let mut controllers: Vec<Option<Box<dyn Controller>>> = (0..6).map(|_| None).collect();
    let mut checkpoints = Vec::new();
    while let Some(request) = engine.request() {
        push_before(&mut checkpoints, &request)?;
        let controller = controllers[usize::from(request.actor)].get_or_insert_with(|| {
            if *mode == WinkMode::Diagnostic
                && request.observation.objective == ObjectiveTeam::Threat
            {
                Box::new(ExperimentThreatController::new(seed, request.actor))
            } else {
                Box::new(BuiltinController::new(
                    *policy,
                    deduction::policy_seed(seed, *policy, request.actor),
                ))
            }
        });
        let response = controller.respond(&request);
        engine
            .submit(response.clone())
            .map_err(action_field_error)?;
        push_after(&mut checkpoints, &request, &response, &engine)?;
    }
    let archive = engine.archive();
    let replayed =
        deduction::replay(&archive).map_err(|e| super::error("replay", e.to_string()))?;
    let fingerprint = format!("{:016x}", replayed.fingerprint());
    let mut archived = lossless_value(&archive)?;
    archived["fingerprint"] = json!(fingerprint);
    let record = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: StudyId::Wink,
        rules_identity: catalog::rules_identity(StudyId::Wink)?,
        input: serde_json::to_value(input).map_err(|e| super::error("input", e.to_string()))?,
        semantics: Semantics::Trajectory,
        checkpoints,
        payload: json!({"archive":archived,"fingerprint":fingerprint,"outcome":lossless_value(&engine.outcome())?}),
    };
    record::check_bounds(&record)?;
    Ok(record)
}
fn clock(request: &TurnRequest) -> Result<Value, Vec<FieldError>> {
    lossless_value(
        &json!({"round":request.round,"phase":request.phase,"request_id":request.request_id}),
    )
}
fn public(
    request: Option<&TurnRequest>,
    outcome: Option<&Outcome>,
    submission_status: Option<&str>,
) -> Result<Value, Vec<FieldError>> {
    let events: Vec<_> = request
        .into_iter()
        .flat_map(|r| {
            r.observation.events.iter().filter_map(|event| {
                let public = match &event.content {
                    EventContent::StatusChange { .. } | EventContent::Claim { .. } => true,
                    EventContent::Use { capability, .. } => r
                        .observation
                        .capabilities
                        .iter()
                        .any(|c| c.id == *capability && c.visibility == Visibility::Public),
                    EventContent::RecipientNotice { .. } => false,
                };
                // Per-observer event sequence numbers can count private evidence.
                public.then(|| json!({"round":event.round,"content":event.content}))
            })
        })
        .collect();
    lossless_value(
        &json!({"roster":request.map(|r| &r.observation.roster),"events":events,"outcome":outcome,"submission_status":submission_status,"availability":if request.is_some() {"current_request"} else {"terminal_outcome_only"}}),
    )
}
fn retained_local(checkpoints: &[Checkpoint]) -> BTreeMap<String, Value> {
    checkpoints
        .last()
        .map(|c| c.local.clone())
        .unwrap_or_default()
}
fn push_before(
    checkpoints: &mut Vec<Checkpoint>,
    request: &TurnRequest,
) -> Result<(), Vec<FieldError>> {
    let clock = clock(request)?;
    let mut local = retained_local(checkpoints);
    local.insert(
        request.actor.to_string(),
        json!({"delivered_clock":clock,"request":lossless_value(request)?}),
    );
    record::push_checkpoint(
        checkpoints,
        Checkpoint {
            index: 0,
            clock,
            kind: "before_request".into(),
            public: public(Some(request), None, None)?,
            local,
            researcher: Some(
                json!({"label":"Researcher","current_request":lossless_value(request)?}),
            ),
        },
    )
}
fn push_after(
    checkpoints: &mut Vec<Checkpoint>,
    request: &TurnRequest,
    response: &TurnResponse,
    engine: &Engine,
) -> Result<(), Vec<FieldError>> {
    let next = engine.request();
    let committed = next
        .as_ref()
        .is_none_or(|next| next.round != request.round || next.phase != request.phase);
    let status = if committed { "committed" } else { "pending" };
    let mut local = retained_local(checkpoints);
    if let Some(actor) = local.get_mut(&request.actor.to_string()) {
        actor["submission"] = json!({"response":lossless_value(response)?,"status":status});
    }
    if committed {
        for actor in local.values_mut() {
            if let Some(submission) = actor.get_mut("submission") {
                submission["status"] = json!("committed");
            }
        }
    }
    record::push_checkpoint(
        checkpoints,
        Checkpoint {
            index: 0,
            clock: clock(next.as_ref().unwrap_or(request))?,
            kind: "after_submission".into(),
            public: public(next.as_ref(), engine.outcome(), Some(status))?,
            local,
            researcher: Some(
                json!({"label":"Researcher","current_request":lossless_value(&next)?,"submitted_response":lossless_value(response)?,"submission_status":status}),
            ),
        },
    )
}
fn action_field_error(error: ActionError) -> Vec<FieldError> {
    super::error("action", error.to_string())
}
