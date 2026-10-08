//! Bounded public-history cases for unchanged reporting and inference engines.
use super::{
    catalog, record, recorded::retained, wire::lossless_value, Checkpoint, EpisodeKind,
    EpisodeRecord, FieldError, Input, Semantics, StudyDescriptor, StudyFamily, StudyId,
    EPISODE_VERSION, MAX_INPUT_BYTES,
};
use crate::deduction::{
    adversarial_audit as audit, strategic_reporting as reporting, strategy_inference as inference,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::collections::BTreeMap;
fn decode<T: DeserializeOwned>(value: &Value, field: &str) -> Result<T, Vec<FieldError>> {
    serde_json::from_value(value.clone()).map_err(|e| super::error(field, e.to_string()))
}
fn select<'a>(
    value: &'a Value,
    array: &str,
    field: &str,
    name: &str,
) -> Result<&'a Value, Vec<FieldError>> {
    value[array]
        .as_array()
        .and_then(|values| values.iter().find(|v| v[field] == name))
        .ok_or_else(|| super::error(field, format!("unknown original {array} selection: {name}")))
}
fn listener(environment: &Value, name: &str) -> Result<reporting::FrozenListener, Vec<FieldError>> {
    let value = environment["listeners"]
        .as_array()
        .and_then(|rows| rows.iter().find(|r| r["algorithm"]["kind"] == name))
        .ok_or_else(|| {
            super::error(
                "listener",
                "listener is not in this original environment's panel",
            )
        })?;
    decode(value, "listener")
}
fn audit_policy(
    value: &Value,
    environment: &str,
    witness: &str,
) -> Result<(reporting::Policy, Option<Value>), Vec<FieldError>> {
    if let Some(target) = witness.strip_prefix("witness_") {
        let row = value["targeted_audits"]
            .as_array()
            .and_then(|rows| {
                rows.iter()
                    .find(|r| r["environment_id"] == environment && r["controller"] == target)
            })
            .ok_or_else(|| super::error("witness", "unknown retained target witness"))?;
        return Ok((
            decode(&row["witness"]["policy"], "witness")?,
            Some(row["witness"].clone()),
        ));
    }
    let provenance = select(value, "policy_provenance", "id", witness)?;
    Ok((
        decode(&json!({"bits":provenance["raw_bits"]}), "witness")?,
        None,
    ))
}
pub(super) fn validate_input(input: &Input) -> Result<(), Vec<FieldError>> {
    let history = match input {
        Input::StrategicReporting {
            environment,
            history,
            policy,
            listener: name,
        } => {
            let value = retained("strategic_reporting")?;
            let env = select(value, "environments", "name", environment)?;
            listener(env, name)?;
            select(value, "policies", "id", policy)?;
            *history
        }
        Input::StrategyInference {
            environment,
            history,
            catalog,
        } => {
            let value = retained("strategy_inference")?;
            select(value, "environments", "id", environment)?;
            select(value, "catalogs", "id", catalog)?;
            *history
        }
        Input::AdversarialAudit {
            environment,
            history,
            controller,
            witness,
        } => {
            let value = retained("adversarial_audit")?;
            select(value, "environments", "id", environment)?;
            let _: audit::ControllerKind = decode(&json!(controller), "controller")?;
            audit_policy(value, environment, witness)?;
            *history
        }
        _ => {
            return Err(super::error(
                "study",
                "expected a reporting, inference, or audit case",
            ))
        }
    };
    if history >= 32 {
        return Err(super::error(
            "history",
            "history must be 0..31 in original observation order",
        ));
    }
    Ok(())
}
pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    record::serialized_size(input, MAX_INPUT_BYTES, "input")?;
    validate_input(input)?;
    match input {
        Input::StrategicReporting {
            environment,
            history,
            policy,
            listener: name,
        } => {
            let value = retained("strategic_reporting")?;
            let environment = select(value, "environments", "name", environment)?;
            let rules: reporting::Config = decode(&environment["rules"], "rules")?;
            let listener = listener(environment, name)?;
            let policy_source = if policy == "canonical_optimum" {
                "Informed reference optimum for selected environment"
            } else if policy.starts_with("genetic_") || policy.starts_with("random_") {
                "Retained training champion transferred to selected environment"
            } else {
                "Original named reporter control"
            };
            let policy: reporting::Policy = decode(
                if policy == "canonical_optimum" {
                    &environment["optimum"]["policy"]
                } else {
                    &select(value, "policies", "id", policy)?["policy"]
                },
                "policy",
            )?;
            let distribution = reporting::enumerate(&rules).map_err(reporting_error)?;
            let masses = reporting::histories(&distribution, &policy).map_err(reporting_error)?;
            let evaluation =
                reporting::evaluate(&distribution, &policy, &listener).map_err(reporting_error)?;
            let row = &evaluation.histories[usize::from(*history)];
            if row.total_mass != masses[usize::from(*history)].total_mass {
                return Err(super::error("history", "original evaluation mass differs"));
            }
            let action = listener.decide(&row.observation).map_err(reporting_error)?;
            if row.total_mass > 0 && Some(action) != row.listener_action {
                return Err(super::error(
                    "decision",
                    "original evaluation action differs",
                ));
            }
            let decision = json!({"action":action,"posterior_true":null,"posterior_availability":"not_exposed_by_frozen_listener"});
            conditional_record(
                input,
                *history,
                row,
                evaluation.denominator,
                &decision,
                json!({"label":"Declared frozen listener assumptions","listener":lossless_value(&listener)?}),
                json!({"actual_policy":lossless_value(&policy)?,"actual_policy_source":policy_source,"retained_witness":null}),
            )
        }
        Input::StrategyInference {
            environment,
            history,
            catalog: name,
        } => {
            let value = retained("strategy_inference")?;
            let rules = decode(
                &select(value, "environments", "id", environment)?["rules"],
                "rules",
            )?;
            let catalog: inference::Catalog = decode(
                &select(value, "catalogs", "id", name)?["catalog"],
                "catalog",
            )?;
            inference_record(input, *history, &rules, &catalog)
        }
        Input::AdversarialAudit {
            environment,
            history,
            controller,
            witness,
        } => {
            let value = retained("adversarial_audit")?;
            let rules = decode(
                &select(value, "environments", "id", environment)?["rules"],
                "rules",
            )?;
            let controller = decode(&json!(controller), "controller")?;
            let (policy, witness) = audit_policy(value, environment, witness)?;
            let frozen = audit::FrozenActions::freeze(&rules, controller).map_err(audit_error)?;
            let snapshot = value["controller_snapshots"]
                .as_array()
                .and_then(|rows| {
                    rows.iter().find(|r| {
                        r["environment_id"] == environment.as_str()
                            && r["snapshot"]["controller"] == json!(controller)
                    })
                })
                .ok_or_else(|| {
                    super::error("controller", "missing retained controller snapshot")
                })?;
            let original: audit::ActionSnapshot =
                decode(&snapshot["snapshot"], "controller_snapshot")?;
            if frozen.snapshot() != original {
                return Err(super::error(
                    "controller",
                    "frozen public actions differ from original report",
                ));
            }
            let evaluation = audit::evaluate_fixed(&frozen, &policy).map_err(audit_error)?;
            let row = &evaluation.histories[usize::from(*history)];
            let decision = lossless_value(&frozen.rows()[usize::from(*history)].decision)?;
            let assumptions = match controller {
                audit::ControllerKind::StrategyUniform => {
                    json!({"controller":controller,"catalog":lossless_value(&inference::Catalog::uniform())?})
                }
                audit::ControllerKind::StrategyOptimizationInformed => {
                    json!({"controller":controller,"catalog":lossless_value(&inference::Catalog::optimization_informed())?})
                }
                audit::ControllerKind::FixedOnly => {
                    json!({"controller":controller,"evidence":"fixed reporter only"})
                }
                audit::ControllerKind::Passive => {
                    json!({"controller":controller,"posterior_true":null})
                }
            };
            conditional_record(
                input,
                *history,
                row,
                evaluation.denominator,
                &decision,
                assumptions,
                json!({"actual_policy":lossless_value(&policy)?,"retained_witness":lossless_value(&witness)?,"retained_audit_summary":audit_summary(value,environment,controller)?}),
            )
        }
        _ => Err(super::error(
            "study",
            "expected a reporting, inference, or audit case",
        )),
    }
}

fn audit_summary(
    value: &Value,
    environment: &str,
    controller: audit::ControllerKind,
) -> Result<Value, Vec<FieldError>> {
    let mut summary = json!({"label":"Retained population and guarantee measurements","environment":environment,"controller":controller,"provenance":value["provenance"]});
    for field in [
        "control_evaluations",
        "targeted_audits",
        "nominal_evaluations",
        "cross_target_evaluations",
    ] {
        let rows = value[field]
            .as_array()
            .ok_or_else(|| super::error("fixture", "missing retained audit evaluations"))?;
        summary[field] = Value::Array(
            rows.iter()
                .filter(|r| {
                    r["environment_id"] == environment && r["controller"] == json!(controller)
                })
                .map(|r| {
                    let mut row = r.clone();
                    if let Some(evaluation) = row["evaluation"].as_object_mut() {
                        evaluation.remove("histories");
                    }
                    row
                })
                .collect(),
        );
    }
    summary["bound_check"] = select(value, "bound_checks", "environment_id", environment)?.clone();
    lossless_value(&summary)
}

fn conditional_record(
    input: &Input,
    history: u8,
    row: &reporting::HistoryEvaluation,
    distribution_denominator: u64,
    decision: &Value,
    assumptions: Value,
    extra: Value,
) -> Result<EpisodeRecord, Vec<FieldError>> {
    let observation = lossless_value(&row.observation)?;
    let reference = lossless_value(&row.reference_posterior)?;
    let mass = json!({"total_mass":row.total_mass.to_string(),"true_mass":row.true_mass.to_string(),"distribution_denominator":distribution_denominator.to_string()});
    let availability = if row.total_mass == 0 {
        "zero_mass"
    } else {
        "conditional_only"
    };
    let payoff = if let Some(action) = row.listener_action {
        let numerator = if action == reporting::DecisionAction::Intervene {
            2 * row.true_mass as i64 - row.total_mass as i64
        } else {
            0
        };
        json!({"numerator":numerator.to_string(),"denominator":row.total_mass.to_string()})
    } else {
        Value::Null
    };
    let researcher = json!({"label":"Researcher","conditional_mass":mass,"reference_posterior":reference,"availability":availability,"actual_policy":extra["actual_policy"]});
    let mut checkpoints = Vec::with_capacity(2);
    for phase in ["before_decision", "after_decision"] {
        let mut local = json!({"observation":observation,"inference_assumptions":assumptions});
        let mut public = json!({"observation":observation,"inference_assumptions":assumptions});
        if phase == "after_decision" {
            local["decision"] = decision.clone();
            public["action"] = decision["action"].clone();
            local["response_label"] = json!("Hypothetical public-view response");
            public["response_label"] = json!("Hypothetical public-view response");
        }
        // Receiver response depends only on public view, even if actual support is zero.
        // Actual-policy support is Researcher-only; it never enters the receiver view.
        record::push_checkpoint(
            &mut checkpoints,
            Checkpoint {
                index: 0,
                clock: json!({"history":history.to_string(),"phase":phase}),
                kind: phase.into(),
                public,
                local: BTreeMap::from([(row.observation.rules.decider.to_string(), local)]),
                researcher: Some(researcher.clone()),
            },
        )?;
    }
    let conditional_decision = if row.total_mass > 0 {
        decision.clone()
    } else {
        Value::Null
    };
    let mut payload = json!({"observation":observation,"decision":conditional_decision,"public_view_response":{"label":"Hypothetical public-view response","decision":decision},"inference_assumptions":assumptions,"conditional_mass":mass,"reference_posterior":reference,"expected_payoff":payoff,"conditional_regret":if row.total_mass>0 {json!({"numerator":row.regret_numerator.to_string(),"denominator":row.total_mass.to_string()})}else{Value::Null},"availability":availability,"live_truth":null,"private_signals":null,"reporter_profiles":null,"realized_payoff":null});
    for (key, value) in extra
        .as_object()
        .ok_or_else(|| super::error("payload", "expected case metadata"))?
    {
        payload[key] = value.clone();
    }
    finish(input, checkpoints, payload)
}
fn inference_record(
    input: &Input,
    history: u8,
    rules: &reporting::Config,
    catalog: &inference::Catalog,
) -> Result<EpisodeRecord, Vec<FieldError>> {
    let view = audit::history_view(rules, history).map_err(audit_error)?;
    let calibration_view = inference::CalibrationView {
        rules: rules.clone(),
        calibration_reports: view.calibration_reports,
        calibration_truth: view.calibration_truth,
    };
    let model = inference::Model::new(rules, catalog).map_err(inference_error)?;
    let calibration = model
        .calibration(&calibration_view)
        .map_err(inference_error)?;
    let decision = model.decide(&view).map_err(inference_error)?;
    let calibration = lossless_value(&calibration)?;
    let decision = lossless_value(&decision)?;
    let assumptions =
        json!({"label":"Disclosed policy prior assumptions","catalog":lossless_value(catalog)?});
    let mut checkpoints = Vec::with_capacity(2);
    let calibration_view = lossless_value(&calibration_view)?;
    record::push_checkpoint(
        &mut checkpoints,
        Checkpoint {
            index: 0,
            clock: json!({"phase":"after_calibration"}),
            kind: "after_calibration".into(),
            public: json!({"calibration_view":calibration_view,"inference_assumptions":assumptions}),
            local: BTreeMap::from([(
                rules.decider.to_string(),
                json!({"calibration_view":calibration_view,"calibration_belief":calibration,"inference_assumptions":assumptions}),
            )]),
            researcher: Some(json!({"label":"Researcher","availability":"prior_predictive_only"})),
        },
    )?;
    let observation = lossless_value(&view)?;
    record::push_checkpoint(
        &mut checkpoints,
        Checkpoint {
            index: 0,
            clock: json!({"history":history.to_string(),"phase":"after_live_decision"}),
            kind: "after_live_decision".into(),
            public: json!({"observation":observation,"action":decision["action"],"inference_assumptions":assumptions}),
            local: BTreeMap::from([(
                rules.decider.to_string(),
                json!({"observation":observation,"decision":decision,"inference_assumptions":assumptions}),
            )]),
            researcher: Some(json!({"label":"Researcher","availability":"inference_only"})),
        },
    )?;
    finish(
        input,
        checkpoints,
        json!({"observation":observation,"calibration_belief":calibration,"decision":decision,"inference_assumptions":assumptions,"availability":"inference_only","actual_policy":null,"live_truth":null,"private_signals":null,"realized_payoff":null}),
    )
}
fn finish(
    input: &Input,
    checkpoints: Vec<Checkpoint>,
    payload: Value,
) -> Result<EpisodeRecord, Vec<FieldError>> {
    let result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: input.study(),
        rules_identity: catalog::rules_identity(input.study())?,
        input: serde_json::to_value(input).map_err(|e| super::error("input", e.to_string()))?,
        semantics: Semantics::ConditionalCase,
        checkpoints,
        payload,
    };
    record::check_bounds(&result)?;
    Ok(result)
}
fn reporting_error(error: reporting::Error) -> Vec<FieldError> {
    super::error("strategic_reporting", error.to_string())
}
fn inference_error(error: inference::Error) -> Vec<FieldError> {
    super::error("strategy_inference", error.to_string())
}
fn audit_error(error: audit::Error) -> Vec<FieldError> {
    super::error("adversarial_audit", error.to_string())
}
pub(super) fn rules_identity(study: StudyId) -> Result<String, Vec<FieldError>> {
    let name = study_name(study)?;
    let value = retained(name)?;
    let version = value["versions"]["version"]
        .as_str()
        .or_else(|| value["version"].as_str())
        .ok_or_else(|| super::error("rules_identity", "missing retained version"))?;
    Ok(format!(
        "{name}:{version}:source-sha256:{}:first-report-sha256:{}:transform-browser-retained-v1",
        value["identity"]["source_sha256"]
            .as_str()
            .ok_or_else(|| super::error("rules_identity", "missing engine identity"))?,
        value["provenance"]["sha256"]
            .as_str()
            .ok_or_else(|| super::error("rules_identity", "missing first-report identity"))?
    ))
}
fn study_name(study: StudyId) -> Result<&'static str, Vec<FieldError>> {
    match study {
        StudyId::StrategicReporting => Ok("strategic_reporting"),
        StudyId::StrategyInference => Ok("strategy_inference"),
        StudyId::AdversarialAudit => Ok("adversarial_audit"),
        _ => Err(super::error("study", "expected reporting study")),
    }
}
pub(super) fn descriptors() -> Vec<StudyDescriptor> {
    let s = retained("strategic_reporting").expect("embedded reporting definitions must be valid");
    let i = retained("strategy_inference").expect("embedded inference definitions must be valid");
    let a = retained("adversarial_audit").expect("embedded audit definitions must be valid");
    let values = |v: &Value, key: &str, field: &str| {
        v[key]
            .as_array()
            .expect("embedded list")
            .iter()
            .map(|e| e[field].clone())
            .collect::<Vec<_>>()
    };
    let mut witnesses = values(a, "policy_provenance", "id");
    let controllers = vec![
        "fixed_only",
        "passive",
        "strategy_optimization_informed",
        "strategy_uniform",
    ];
    witnesses.extend(controllers.iter().map(|c| json!(format!("witness_{c}"))));
    vec![
        StudyDescriptor{id:StudyId::StrategicReporting,family:StudyFamily::Testimony,title:"Strategic reporting".into(),supplied:"Original seven environments, named policies, all retained champions, and frozen listener assumptions; actual-policy references are Researcher knowledge.".into(),question:"How does strategic reporting affect a frozen receiver on this conditional public history?".into(),default_input:json!({"study":"strategic_reporting","environment":"training","history":0,"policy":"copy_calibration_invert_live","listener":"bayesian"}),controls:json!({"environment":{"values":values(s,"environments","name")},"history":{"type":"integer","min":0,"max":31},"policy":{"values":values(s,"policies","id"),"labels":{"canonical_optimum":"Informed reference optimum for selected environment"}},"listener":{"values":["bayesian","credulous","skeptical","evolved","passive"],"by_environment":s["environments"].as_array().expect("environments").iter().map(|e|json!({"environment":e["name"],"values":e["listeners"].as_array().expect("listeners").iter().map(|l|l["algorithm"]["kind"].clone()).collect::<Vec<_>>()})).collect::<Vec<_>>()}})},
        StudyDescriptor{id:StudyId::StrategyInference,family:StudyFamily::Testimony,title:"Strategy-aware listeners".into(),supplied:"Exact inference over two disclosed policy priors and original public calibration/live histories.".into(),question:"Which policy hypotheses and actions does this disclosed prior support?".into(),default_input:json!({"study":"strategy_inference","environment":"q4_5","history":0,"catalog":"uniform"}),controls:json!({"environment":{"values":values(i,"environments","id")},"history":{"type":"integer","min":0,"max":31},"catalog":{"values":values(i,"catalogs","id")}})},
        StudyDescriptor{id:StudyId::AdversarialAudit,family:StudyFamily::Testimony,title:"Adversarial reporting audit".into(),supplied:"Four certified public receivers, six reporter controls, retained champions, and original targeted witnesses. Bounds concern expected payoff.".into(),question:"How does this receiver perform against an original reporting control or targeted witness?".into(),default_input:json!({"study":"adversarial_audit","environment":"q4_5","history":0,"controller":"fixed_only","witness":"copy"}),controls:json!({"environment":{"values":values(a,"environments","id")},"history":{"type":"integer","min":0,"max":31},"controller":{"values":controllers},"witness":{"values":witnesses,"reporter_controls":a["policy_provenance"].as_array().expect("policies").iter().filter(|p|p["seed"].is_null()).map(|p|p["id"].clone()).collect::<Vec<_>>(),"targeted_prefix":"witness_","label":"Original reporter policies or retained targeted witnesses"}})}
    ]
}
