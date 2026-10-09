use super::super::*;
use crate::deduction::{
    adversarial_audit as audit, strategic_reporting as reporting, strategy_inference as inference,
};
use serde_json::{json, Value};
fn case(study: &str, fields: Value) -> EpisodeRecord {
    let mut input = fields;
    input["study"] = json!(study);
    run(&normalize_input(&input.to_string()).unwrap()).unwrap()
}
#[test]
fn catalog_exposes_three_original_reporting_studies() {
    for id in [
        StudyId::StrategicReporting,
        StudyId::StrategyInference,
        StudyId::AdversarialAudit,
    ] {
        assert!(catalog().iter().any(|s| s.id == id));
    }
}
#[test]
fn identical_calibration_likelihood_does_not_reveal_actual_policy() {
    let result = case(
        "strategy_inference",
        json!({"environment":"q4_5","history":0,"catalog":"uniform"}),
    );
    assert!(result.payload["calibration_belief"]
        .get("actual_policy")
        .is_none());
    assert!(result.checkpoints[0].public.get("live_reports").is_none());
    assert!(result.checkpoints[0].local["2"]["calibration_view"]
        .get("live_reports")
        .is_none());
}
#[test]
fn honest_and_live_invert_have_equal_calibration_likelihood_and_disclosed_prior_odds() {
    let value: Value =
        serde_json::from_str(include_str!("../fixtures/strategy-recorded.json")).unwrap();
    let rules: reporting::Config =
        serde_json::from_value(value["environments"][0]["rules"].clone()).unwrap();
    let copy = inference::canonical_bits(&reporting::Policy::copy())
        .unwrap()
        .to_string();
    let invert = inference::canonical_bits(&reporting::Policy::calibration_copy_live_invert())
        .unwrap()
        .to_string();
    for history in 0..32 {
        for (catalog, odds) in [("uniform", 1u64), ("optimization_informed", 16)] {
            let result = case(
                "strategy_inference",
                json!({"environment":"q4_5","history":history,"catalog":catalog}),
            );
            let policies = result.payload["calibration_belief"]["policies"]
                .as_array()
                .unwrap();
            let mass = |bits: &str| {
                policies
                    .iter()
                    .find(|p| p["canonical_bits"] == bits)
                    .unwrap()["probability"]["numerator"]
                    .as_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            };
            assert_eq!(mass(&invert), odds * mass(&copy));
            let model = inference::Model::new(
                &rules,
                &if catalog == "uniform" {
                    inference::Catalog::uniform()
                } else {
                    inference::Catalog::optimization_informed()
                },
            )
            .unwrap();
            let view = audit::history_view(&rules, history).unwrap();
            let original = model
                .calibration(&inference::CalibrationView {
                    rules: rules.clone(),
                    calibration_reports: view.calibration_reports,
                    calibration_truth: view.calibration_truth,
                })
                .unwrap();
            assert_eq!(
                result.payload["calibration_belief"],
                wire::lossless_value(&original).unwrap()
            );
        }
    }
}
#[test]
fn strategy_all_original_inference_rows_match_without_actual_policy_input() {
    let value: Value =
        serde_json::from_str(include_str!("../fixtures/strategy-recorded.json")).unwrap();
    for row in value["inference_models"].as_array().unwrap() {
        let original: inference::InferenceModelRow = serde_json::from_value(row.clone()).unwrap();
        for (history, reference) in original.histories.iter().enumerate() {
            let result = case(
                "strategy_inference",
                json!({"environment":original.environment_id,"catalog":original.catalog_id,"history":history}),
            );
            assert_eq!(
                result.payload["decision"],
                wire::lossless_value(&reference.decision).unwrap()
            );
        }
    }
}
#[test]
fn strategic_all_selected_named_controls_match_original_exact_rows() {
    let value: Value =
        serde_json::from_str(include_str!("../fixtures/strategic-recorded.json")).unwrap();
    for reference in value["case_references"].as_array().unwrap() {
        if reference["policy"].as_str().unwrap().contains('_')
            && reference["policy"].as_str().unwrap().starts_with("random_")
        {
            continue;
        }
        if reference["policy"]
            .as_str()
            .unwrap()
            .starts_with("genetic_")
        {
            continue;
        }
        let environment = value["environments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["name"] == reference["environment"])
            .unwrap();
        for (index, evaluation) in reference["evaluations"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let listener = environment["listeners"][index]["algorithm"]["kind"].clone();
            for (history, row) in evaluation["histories"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let result = case(
                    "strategic_reporting",
                    json!({"environment":reference["environment"],"policy":reference["policy"],"listener":listener,"history":history}),
                );
                assert_eq!(
                    result.payload["reference_posterior"],
                    wire::lossless_value(&row["reference_posterior"]).unwrap()
                );
                assert_eq!(result.payload["decision"]["action"], row["listener_action"]);
                assert!(result
                    .checkpoints
                    .iter()
                    .all(|c| c.local["2"].get("reference_posterior").is_none()));
            }
        }
    }
}
#[test]
fn zero_mass_case_has_no_successful_decision_or_realization() {
    let result = case(
        "strategic_reporting",
        json!({"environment":"training","policy":"always_positive","listener":"bayesian","history":0}),
    );
    assert_eq!(result.payload["availability"], "zero_mass");
    for field in [
        "decision",
        "reference_posterior",
        "expected_payoff",
        "live_truth",
        "private_signals",
        "realized_payoff",
    ] {
        assert!(result.payload[field].is_null(), "{field}");
    }
}
#[test]
fn audit_all_six_reporter_controls_and_four_receivers_match_original_certified_rows() {
    let value: Value =
        serde_json::from_str(include_str!("../fixtures/audit-recorded.json")).unwrap();
    for control in value["control_evaluations"].as_array().unwrap() {
        let snapshot = value["controller_snapshots"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| {
                s["environment_id"] == control["environment_id"]
                    && s["snapshot"]["controller"] == control["controller"]
            })
            .unwrap();
        let snapshot: audit::ActionSnapshot =
            serde_json::from_value(snapshot["snapshot"].clone()).unwrap();
        let frozen = audit::FrozenActions::from_snapshot(&snapshot).unwrap();
        for history in 0..32 {
            let result = case(
                "adversarial_audit",
                json!({"environment":control["environment_id"],"witness":control["policy_id"],"controller":control["controller"],"history":history}),
            );
            let row = &control["evaluation"]["histories"][history];
            assert_eq!(
                result.payload["reference_posterior"],
                wire::lossless_value(&row["reference_posterior"]).unwrap()
            );
            if row["total_mass"].as_u64().unwrap() > 0 {
                assert_eq!(
                    result.payload["decision"],
                    wire::lossless_value(&frozen.rows()[history].decision).unwrap()
                );
            }
        }
    }
}
#[test]
fn passive_has_no_posterior_and_actual_reference_stays_researcher_only() {
    let result = case(
        "adversarial_audit",
        json!({"environment":"q4_5","witness":"copy","controller":"passive","history":0}),
    );
    assert!(result.payload["decision"]["posterior_true"].is_null());
    assert!(!result.payload["reference_posterior"].is_null());
    assert!(result.checkpoints[1].local["2"]
        .get("reference_posterior")
        .is_none());
}
#[test]
fn targeted_witnesses_resolve_original_audits_without_running_search() {
    let value: Value =
        serde_json::from_str(include_str!("../fixtures/audit-recorded.json")).unwrap();
    for target in value["targeted_audits"].as_array().unwrap() {
        let id = format!("witness_{}", target["controller"].as_str().unwrap());
        let result = case(
            "adversarial_audit",
            json!({"environment":target["environment_id"],"witness":id,"controller":target["controller"],"history":0}),
        );
        assert_eq!(
            result.payload["retained_witness"],
            wire::lossless_value(&target["witness"]).unwrap()
        );
    }
}
#[test]
fn original_names_and_index_bounds_are_strict() {
    for fields in [
        json!({"environment":"training","catalog":"uniform","history":0}),
        json!({"environment":"q4_5","catalog":"uniform","history":32}),
        json!({"environment":"q4_5","catalog":"uniform","history":0,"actual_policy":"copy"}),
    ] {
        let mut fields = fields;
        fields["study"] = json!("strategy_inference");
        assert!(normalize_input(&fields.to_string()).is_err());
    }
    let result = case(
        "strategy_inference",
        json!({"environment":"q4_5","catalog":"uniform","history":31}),
    );
    assert_eq!(
        validate_episode(&episode_json(&result).unwrap()).unwrap(),
        result
    );
}
#[test]
fn audit_population_and_guarantee_summaries_are_retained_payload_only() {
    let result = case(
        "adversarial_audit",
        json!({"environment":"q4_5","witness":"copy","controller":"fixed_only","history":0}),
    );
    let summary = &result.payload["retained_audit_summary"];
    assert_eq!(
        summary["label"],
        "Retained population and guarantee measurements"
    );
    assert_eq!(summary["nominal_evaluations"].as_array().unwrap().len(), 2);
    assert_eq!(
        summary["cross_target_evaluations"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(summary["control_evaluations"].as_array().unwrap().len(), 6);
    assert_eq!(summary["targeted_audits"].as_array().unwrap().len(), 1);
    assert_eq!(
        summary["bound_check"]["fixed_guarantee"],
        json!({"numerator":"9","denominator":"50"})
    );
    assert!(summary["nominal_evaluations"][0]["evaluation"]
        .get("histories")
        .is_none());
    assert!(result
        .checkpoints
        .iter()
        .all(|c| c.public.get("retained_audit_summary").is_none()
            && c.local["2"].get("retained_audit_summary").is_none()));
}
#[test]
fn typed_reporting_inputs_cannot_bypass_history_or_input_limits() {
    let invalid = Input::AdversarialAudit {
        environment: "q4_5".into(),
        history: 255,
        controller: "fixed_only".into(),
        witness: "copy".into(),
    };
    assert!(super::super::reporting::run(&invalid).is_err());
    assert!(run(&invalid).is_err());
    let oversized = Input::StrategyInference {
        environment: "x".repeat(MAX_INPUT_BYTES),
        history: 0,
        catalog: "uniform".into(),
    };
    assert_eq!(
        super::super::reporting::run(&oversized).unwrap_err()[0].field,
        "input"
    );
}
#[test]
fn receiver_public_and_local_responses_do_not_reveal_actual_policy_support() {
    for (study, environment, selector, receiver_field, receiver) in [
        (
            "strategic_reporting",
            "training",
            "policy",
            "listener",
            "bayesian",
        ),
        (
            "adversarial_audit",
            "q4_5",
            "witness",
            "controller",
            "strategy_uniform",
        ),
    ] {
        let input = |policy: &str| {
            let mut fields = json!({"environment":environment,"history":0});
            fields[selector] = json!(policy);
            fields[receiver_field] = json!(receiver);
            case(study, fields)
        };
        let unsupported = input("always_positive");
        let supported = input("always_negative");
        assert_eq!(unsupported.payload["availability"], "zero_mass");
        assert_eq!(supported.payload["availability"], "conditional_only");
        for (left, right) in unsupported.checkpoints.iter().zip(&supported.checkpoints) {
            assert_eq!(left.public, right.public, "{study} public projection");
            assert_eq!(left.local, right.local, "{study} receiver projection");
        }
        assert_eq!(
            unsupported.checkpoints[1].local["2"]["response_label"],
            "Hypothetical public-view response"
        );
        assert_eq!(
            supported.checkpoints[1].local["2"]["response_label"],
            "Hypothetical public-view response"
        );
        assert!(unsupported.payload["decision"].is_null());
    }
}
#[test]
fn entire_calibration_checkpoint_is_invariant_to_future_live_reports() {
    let agent_safe = |checkpoint: &Checkpoint| {
        json!({
            "index":checkpoint.index,
            "kind":checkpoint.kind,
            "clock":checkpoint.clock,
            "public":checkpoint.public,
            "local":checkpoint.local,
        })
    };
    for environment in ["q4_5", "q3_5"] {
        for catalog in ["uniform", "optimization_informed"] {
            for calibration in 0..8 {
                let first_history = calibration * 4;
                let baseline = case(
                    "strategy_inference",
                    json!({"environment":environment,"catalog":catalog,"history":first_history}),
                );
                let expected = agent_safe(&baseline.checkpoints[0]);
                for live_suffix in 1..4 {
                    let history = first_history + live_suffix;
                    let result = case(
                        "strategy_inference",
                        json!({"environment":environment,"catalog":catalog,"history":history}),
                    );
                    assert_eq!(
                        agent_safe(&result.checkpoints[0]),
                        expected,
                        "{environment}/{catalog}/calibration{calibration}/live{live_suffix}"
                    );
                    let history_text = history.to_string();
                    assert_eq!(
                        result.checkpoints[1].clock["history"].as_str(),
                        Some(history_text.as_str())
                    );
                }
            }
        }
    }
}
