//! Retained evidence sequences and finite public testimony decision cases.
use super::{
    catalog, record, wire::lossless_value, Checkpoint, EpisodeKind, EpisodeRecord, FieldError,
    Input, Semantics, StudyDescriptor, StudyFamily, StudyId, EPISODE_VERSION, MAX_CHECKPOINTS,
    MAX_INPUT_BYTES,
};
use crate::deduction::{
    testimony_game, Belief, BeliefSnapshot, EvidenceRecord, TestimonyError, TestimonyModel,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const RETAINED: &str = include_str!("fixtures/testimony-game-recorded.json");
const MAX_RETAINED_BYTES: usize = 128 * 1024;
const MAX_EVIDENCE: usize = 512;
#[derive(Deserialize)]
struct Retained {
    fixtures: Vec<Fixture>,
    environments: Vec<Environment>,
    listeners: Vec<NamedListener>,
    identities: Value,
    provenance: Value,
    versions: Value,
}
#[derive(Deserialize)]
struct Fixture {
    name: String,
    model: TestimonyModel,
    records: Vec<EvidenceRecord>,
}
#[derive(Deserialize)]
struct Environment {
    name: String,
    config: testimony_game::Config,
}
#[derive(Deserialize)]
struct NamedListener {
    id: String,
    listener: testimony_game::Listener,
}
fn retained() -> Result<Retained, Vec<FieldError>> {
    if RETAINED.len() > MAX_RETAINED_BYTES {
        return Err(super::error(
            "fixture",
            "retained definitions exceed 128 KiB",
        ));
    }
    let retained: Retained =
        serde_json::from_str(RETAINED).map_err(|e| super::error("fixture", e.to_string()))?;
    if retained.fixtures.len() != 13
        || retained.environments.len() != 4
        || retained.listeners.len() != 45
    {
        return Err(super::error(
            "fixture",
            "retained definition counts do not match the original studies",
        ));
    }
    Ok(retained)
}
pub(super) fn validate_input(input: &Input) -> Result<(), Vec<FieldError>> {
    let retained = retained()?;
    match input {
        Input::Testimony { fixture } => {
            select_fixture(&retained, fixture)?;
        }
        Input::TestimonyGame {
            environment,
            history,
            listener,
        } => {
            select_environment(&retained, environment)?;
            select_listener(&retained, listener)?;
            if *history >= 32 {
                return Err(super::error(
                    "history",
                    "history must be 0..31 in original observation order",
                ));
            }
        }
        _ => {
            return Err(super::error(
                "study",
                "expected testimony or testimony_game",
            ))
        }
    }
    Ok(())
}
fn select_fixture<'a>(retained: &'a Retained, name: &str) -> Result<&'a Fixture, Vec<FieldError>> {
    retained
        .fixtures
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| super::error("fixture", "unknown original testimony fixture"))
}
fn select_environment<'a>(
    retained: &'a Retained,
    name: &str,
) -> Result<&'a Environment, Vec<FieldError>> {
    // The browser name `training` already equals the original report's name.
    retained
        .environments
        .iter()
        .find(|e| e.name == name)
        .ok_or_else(|| super::error("environment", "unknown original testimony game environment"))
}
fn select_listener<'a>(
    retained: &'a Retained,
    name: &str,
) -> Result<&'a NamedListener, Vec<FieldError>> {
    retained
        .listeners
        .iter()
        .find(|l| l.id == name)
        .ok_or_else(|| super::error("listener", "unknown fixed or retained method/seed listener"))
}
pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    record::serialized_size(input, MAX_INPUT_BYTES, "input")?;
    validate_input(input)?;
    let retained = retained()?;
    match input {
        Input::Testimony { fixture } => {
            let fixture = select_fixture(&retained, fixture)?;
            run_evidence(input, fixture.model.clone(), &fixture.records)
        }
        Input::TestimonyGame {
            environment,
            history,
            listener,
        } => {
            let config = &select_environment(&retained, environment)?.config;
            let listener = &select_listener(&retained, listener)?.listener;
            run_case(input, config, *history, listener)
        }
        _ => Err(super::error(
            "study",
            "expected testimony or testimony_game",
        )),
    }
}
/// Sequentially condition one attributed record at a time; a failed observation is atomic.
pub(super) fn run_evidence(
    input: &Input,
    model: TestimonyModel,
    records: &[EvidenceRecord],
) -> Result<EpisodeRecord, Vec<FieldError>> {
    if records.len() > MAX_EVIDENCE || records.len() + 1 > MAX_CHECKPOINTS {
        return Err(super::error(
            "records",
            "retained evidence exceeds 512 records",
        ));
    }
    let mut belief = Belief::new(model.clone()).map_err(|e| testimony_error("model", e))?;
    let assumptions =
        json!({"label":"Declared inference assumptions","model":lossless_value(&model)?});
    let mut checkpoints = Vec::with_capacity(records.len() + 1);
    let mut accepted = Vec::with_capacity(records.len());
    push_evidence_checkpoint(
        &mut checkpoints,
        &assumptions,
        &accepted,
        &belief.snapshot(),
        None,
        None,
    )?;
    let mut failure = None;
    for (index, evidence) in records.iter().enumerate() {
        match belief.observe(evidence.clone()) {
            Ok(()) => {
                accepted.push(evidence.clone());
                push_evidence_checkpoint(
                    &mut checkpoints,
                    &assumptions,
                    &accepted,
                    &belief.snapshot(),
                    None,
                    None,
                )?;
            }
            Err(error) => {
                let errors = testimony_error(&format!("records[{index}]"), error);
                push_evidence_checkpoint(
                    &mut checkpoints,
                    &assumptions,
                    &accepted,
                    &belief.snapshot(),
                    Some(evidence),
                    Some(errors.as_slice()),
                )?;
                failure = Some(errors);
                break;
            }
        }
    }
    let result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: StudyId::Testimony,
        rules_identity: catalog::rules_identity(StudyId::Testimony)?,
        input: serde_json::to_value(input).map_err(|e| super::error("input", e.to_string()))?,
        semantics: Semantics::EvidenceSequence,
        checkpoints,
        payload: json!({"model":lossless_value(&model)?,"records":lossless_value(&records)?,"snapshot":lossless_value(&belief.snapshot())?,"status":if failure.is_some(){"evidence_error"}else{"complete"},"error":failure}),
    };
    record::check_bounds(&result)?;
    Ok(result)
}
fn testimony_error(field: &str, error: TestimonyError) -> Vec<FieldError> {
    super::error(field, error.to_string())
}
fn push_evidence_checkpoint(
    checkpoints: &mut Vec<Checkpoint>,
    assumptions: &Value,
    evidence: &[EvidenceRecord],
    snapshot: &BeliefSnapshot,
    attempted: Option<&EvidenceRecord>,
    errors: Option<&[FieldError]>,
) -> Result<(), Vec<FieldError>> {
    let clock = json!({"evidence_step":checkpoints.len().to_string()});
    let evidence = lossless_value(&evidence)?;
    let snapshot = lossless_value(snapshot)?;
    let attempted = lossless_value(&attempted)?;
    record::push_checkpoint(
        checkpoints,
        Checkpoint {
            index: 0,
            clock,
            kind: if errors.is_some() {
                "evidence_error"
            } else if evidence.as_array().is_some_and(Vec::is_empty) {
                "prior"
            } else {
                "after_evidence"
            }
            .into(),
            public: json!({"inference_assumptions":assumptions,"evidence":evidence,"attempted_record":attempted,"error":errors}),
            local: BTreeMap::from([(
                "listener".into(),
                json!({"evidence":evidence,"snapshot":snapshot,"attempted_record":attempted,"error":errors}),
            )]),
            researcher: Some(json!({"label":"Researcher","snapshot":snapshot})),
        },
    )
}
fn run_case(
    input: &Input,
    config: &testimony_game::Config,
    history: u8,
    listener: &testimony_game::Listener,
) -> Result<EpisodeRecord, Vec<FieldError>> {
    let distribution = testimony_game::enumerate(config).map_err(game_error)?;
    let evaluation = testimony_game::evaluate(&distribution, listener).map_err(game_error)?;
    let row = evaluation
        .histories
        .get(usize::from(history))
        .ok_or_else(|| super::error("history", "selected conditional history is unavailable"))?;
    let decision = listener.decide(&row.observation).map_err(game_error)?;
    if decision != row.decision {
        return Err(super::error(
            "decision",
            "listener decision differs from unchanged evaluation",
        ));
    }
    let observation = lossless_value(&row.observation)?;
    let decision = lossless_value(&decision)?;
    let mass = lossless_value(
        &json!({"total_mass":row.total_mass,"true_mass":row.true_mass,"distribution_denominator":distribution.denominator}),
    )?;
    let payoff = if row.decision.action == testimony_game::DecisionAction::Intervene {
        2 * row.true_mass as i64 - row.total_mass as i64
    } else {
        0
    };
    let expected_payoff =
        lossless_value(&json!({"numerator":payoff,"denominator":row.total_mass}))?;
    let reference =
        json!({"numerator":row.true_mass.to_string(),"denominator":row.total_mass.to_string()});
    let mut checkpoints = Vec::with_capacity(2);
    for phase in ["before_decision", "after_decision"] {
        let mut local = json!({"observation":observation});
        let mut public = json!({"observation":observation});
        if phase == "after_decision" {
            local["decision"] = decision.clone();
            public["action"] = decision["action"].clone();
        }
        record::push_checkpoint(
            &mut checkpoints,
            Checkpoint {
                index: 0,
                clock: json!({"history":history.to_string(),"phase":phase}),
                kind: phase.into(),
                public,
                local: BTreeMap::from([(config.decider.to_string(), local)]),
                researcher: Some(
                    json!({"label":"Researcher","conditional_mass":mass,"reference_posterior":reference,"availability":"conditional_only"}),
                ),
            },
        )?;
    }
    let result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: StudyId::TestimonyGame,
        rules_identity: catalog::rules_identity(StudyId::TestimonyGame)?,
        input: serde_json::to_value(input).map_err(|e| super::error("input", e.to_string()))?,
        semantics: Semantics::ConditionalCase,
        checkpoints,
        payload: json!({"observation":observation,"listener":lossless_value(listener)?,"decision":decision,"conditional_mass":mass,"reference_posterior":reference,"conditional_regret":row.conditional_regret,"expected_payoff":expected_payoff,"live_truth":null,"private_signals":null,"reporter_profiles":null,"realized_payoff":null,"availability":"conditional_only"}),
    };
    record::check_bounds(&result)?;
    Ok(result)
}
fn game_error(error: testimony_game::Error) -> Vec<FieldError> {
    super::error("testimony_game", error.to_string())
}
pub(super) fn rules_identity(study: StudyId) -> Result<String, Vec<FieldError>> {
    let retained = retained()?;
    let name = match study {
        StudyId::Testimony => "testimony",
        StudyId::TestimonyGame => "testimony_game",
        _ => {
            return Err(super::error(
                "study",
                "expected testimony or testimony_game",
            ))
        }
    };
    let source = retained.identities[name]["source_sha256"]
        .as_str()
        .ok_or_else(|| super::error("rules_identity", "missing retained engine source digest"))?;
    let report = retained.provenance[name]["sha256"]
        .as_str()
        .ok_or_else(|| super::error("rules_identity", "missing first-report digest"))?;
    let version = if study == StudyId::Testimony {
        retained.versions["testimony"].as_str()
    } else {
        retained.versions["version"].as_str()
    }
    .ok_or_else(|| super::error("rules_identity", "missing retained version"))?;
    if study == StudyId::Testimony {
        Ok(format!(
            "{name}:{version}:source-sha256:{source}:first-report-sha256:{report}"
        ))
    } else {
        let listener_version = retained.versions["listener_version"]
            .as_str()
            .ok_or_else(|| super::error("rules_identity", "missing retained listener version"))?;
        Ok(format!("{name}:{version}:protocol-{}:game-{}:listener-{listener_version}:source-sha256:{source}:first-report-sha256:{report}",testimony_game::GAME_PROTOCOL_VERSION,testimony_game::GAME_VERSION))
    }
}
pub(super) fn descriptors() -> Vec<StudyDescriptor> {
    let retained = retained().expect("embedded original testimony definitions must be valid");
    vec![
        StudyDescriptor {
            id:StudyId::Testimony,family:StudyFamily::Testimony,title:"Noisy testimony".into(),
            supplied:"Named retained evidence sequences with attributed reports, persistent reporting profiles, and explicit verification.".into(),
            question:"How do repeated, correlated, or verified reports change supported beliefs?".into(),
            default_input:json!({"study":"testimony","fixture":"transfer"}),
            controls:json!({"fixture":{"values":retained.fixtures.iter().map(|f|&f.name).collect::<Vec<_>>()}}),
        },
        StudyDescriptor {
            id:StudyId::TestimonyGame,family:StudyFamily::Testimony,title:"Testimony decision game".into(),
            supplied:"Original named environments, all 32 public histories, five fixed listeners, and all 40 retained evolved listeners; conditional mass and expected payoff.".into(),
            question:"Which intervention is supported by this public calibration and live-report history?".into(),
            default_input:json!({"study":"testimony_game","environment":"training","history":31,"listener":"bayesian"}),
            controls:json!({"environment":{"values":retained.environments.iter().map(|e|&e.name).collect::<Vec<_>>()},"history":{"type":"integer","min":0,"max":31},"listener":{"values":retained.listeners.iter().map(|l|&l.id).collect::<Vec<_>>()}}),
        },
    ]
}
