use super::super::*;
use crate::deduction::{self, testimony_game};
use serde_json::{json, Value};

fn retained() -> Value {
    serde_json::from_str(include_str!("../fixtures/testimony-game-recorded.json")).unwrap()
}
fn testimony_input(fixture: &str) -> Input {
    normalize_input(&json!({"study":"testimony","fixture":fixture}).to_string()).unwrap()
}
fn game_input(environment: &str, history: u8, listener: &str) -> Input {
    normalize_input(&json!({"study":"testimony_game","environment":environment,"history":history,"listener":listener}).to_string()).unwrap()
}
#[test]
fn ordered_evidence_never_reveals_future_verification() {
    let record = run(&testimony_input("transfer")).unwrap();
    assert_eq!(record.semantics, Semantics::EvidenceSequence);
    assert_eq!(record.checkpoints.len(), 4);
    assert!(record.checkpoints[0].public["evidence"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(
        (record.checkpoints[1].local["listener"]["snapshot"]["propositions"][0]
            ["probability_true"]
            .as_f64()
            .unwrap()
            - 0.65)
            .abs()
            <= 1e-12
    );
    assert!(!record.checkpoints[1]
        .public
        .to_string()
        .contains("verified"));
    assert!(
        (record.checkpoints[2].local["listener"]["snapshot"]["propositions"][0]
            ["probability_true"]
            .as_f64()
            .unwrap()
            - 1.0)
            .abs()
            <= 1e-12
    );
    assert_eq!(
        record.checkpoints[3].public["evidence"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}
#[test]
fn duplicate_evidence_id_is_idempotent() {
    let record = run(&testimony_input("duplicate_record")).unwrap();
    assert_eq!(record.checkpoints.len(), 3);
    assert_eq!(
        record.checkpoints[1].local["listener"]["snapshot"],
        record.checkpoints[2].local["listener"]["snapshot"]
    );
    assert_eq!(
        record.checkpoints[2].local["listener"]["snapshot"]["evidence_count"],
        "1"
    );
}
#[test]
fn contradictions_preserve_last_supported_snapshot() {
    let record = run(&testimony_input("model_contradiction")).unwrap();
    let terminal = record.checkpoints.last().unwrap();
    assert_eq!(terminal.kind, "evidence_error");
    assert_eq!(
        terminal.local["listener"]["snapshot"],
        record.checkpoints[1].local["listener"]["snapshot"]
    );
    assert!(terminal.public["error"][0]["message"]
        .as_str()
        .unwrap()
        .contains("zero probability"));
    assert_eq!(terminal.public["error"][0]["field"], "records[1]");
    assert_eq!(terminal.public["evidence"].as_array().unwrap().len(), 1);
    assert_eq!(record.payload["status"], "evidence_error");
}
#[test]
fn copy_invert_ambiguity_stays_in_the_joint_belief() {
    let record = run(&testimony_input("uncertain_speaker")).unwrap();
    let snapshot = &record.checkpoints.last().unwrap().local["listener"]["snapshot"];
    assert_eq!(snapshot["hypotheses"].as_array().unwrap().len(), 4);
    assert!(
        (snapshot["speakers"][0]["profiles"][0]["probability"]
            .as_f64()
            .unwrap()
            - 0.75)
            .abs()
            <= 1e-12
    );
    assert!(
        (snapshot["propositions"][0]["probability_true"]
            .as_f64()
            .unwrap()
            - 0.65)
            .abs()
            <= 1e-12
    );
}
#[test]
fn all_original_testimony_references_match() {
    for fixture in retained()["fixtures"].as_array().unwrap() {
        let record = run(&testimony_input(fixture["name"].as_str().unwrap())).unwrap();
        let snapshot = &record.checkpoints.last().unwrap().local["listener"]["snapshot"];
        for reference in fixture["references"].as_array().unwrap() {
            let quantity = reference["quantity"].as_str().unwrap();
            let actual = if quantity == "speaker_0_copy" {
                snapshot["speakers"][0]["profiles"][0]["probability"]
                    .as_f64()
                    .unwrap()
            } else {
                let id: usize = quantity
                    .trim_start_matches("proposition_")
                    .trim_end_matches("_true")
                    .parse()
                    .unwrap();
                snapshot["propositions"][id]["probability_true"]
                    .as_f64()
                    .unwrap()
            };
            assert!(
                (actual - reference["expected"].as_f64().unwrap()).abs() <= 1e-12,
                "{} {quantity}",
                fixture["name"]
            );
        }
        assert_eq!(
            validate_episode(&episode_json(&record).unwrap()).unwrap(),
            record
        );
    }
}
#[test]
fn passive_case_has_no_invented_posterior() {
    let record = run(&game_input("training", 31, "passive")).unwrap();
    assert_eq!(record.semantics, Semantics::ConditionalCase);
    assert!(record.payload["decision"]["posterior_true"].is_null());
}
#[test]
fn history31_is_actual_public_observation_and_no_realization() {
    let record = run(&game_input("training", 31, "bayesian")).unwrap();
    let config: testimony_game::Config =
        serde_json::from_value(retained()["environments"][0]["config"].clone()).unwrap();
    let distribution = testimony_game::enumerate(&config).unwrap();
    assert_eq!(
        record.payload["observation"],
        wire::lossless_value(&distribution.histories[31].observation).unwrap()
    );
    assert_eq!(record.checkpoints.len(), 2);
    assert!(record.checkpoints[0].local["2"].get("decision").is_none());
    for checkpoint in &record.checkpoints {
        assert!(!checkpoint.local.contains_key("0"));
        assert!(!checkpoint.local.contains_key("1"));
        assert!(checkpoint.local["2"].get("reference_posterior").is_none());
        assert!(checkpoint.public.get("true_mass").is_none());
    }
    for field in [
        "live_truth",
        "private_signals",
        "reporter_profiles",
        "realized_payoff",
    ] {
        assert!(record.payload.get(field).unwrap().is_null(), "{field}");
    }
    assert_eq!(record.payload["conditional_mass"]["total_mass"], "2657");
    assert_eq!(record.payload["conditional_mass"]["true_mass"], "2401");
    assert_eq!(record.payload["expected_payoff"]["denominator"], "2657");
    assert_eq!(record.payload["expected_payoff"]["numerator"], "2145");
    assert_eq!(
        validate_episode(&episode_json(&record).unwrap()).unwrap(),
        record
    );
}
#[test]
fn all32_histories_match_retained_decisions_for_all_documented_listeners() {
    let retained = retained();
    for reference in retained["game_references"].as_array().unwrap() {
        let environment = reference["environment"].as_str().unwrap();
        let listener = reference["listener"].as_str().unwrap();
        let mask =
            u32::from_str_radix(reference["intervention_mask"].as_str().unwrap(), 16).unwrap();
        for history in 0..32 {
            let record = run(&game_input(environment, history, listener)).unwrap();
            let action = if mask & (1 << history) != 0 {
                "intervene"
            } else {
                "abstain"
            };
            assert_eq!(
                record.payload["decision"]["action"], action,
                "{environment}/{listener}/{history}"
            );
            match reference.get("posteriors") {
                None => assert!(record.payload["decision"]["posterior_true"].is_null()),
                Some(posteriors) => assert!(
                    (record.payload["decision"]["posterior_true"]
                        .as_f64()
                        .unwrap()
                        - posteriors[history as usize].as_f64().unwrap())
                    .abs()
                        <= 1e-12
                ),
            }
            let env = retained["environments"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["name"] == environment)
                .unwrap();
            assert_eq!(
                record.payload["conditional_mass"]["total_mass"],
                env["masses"][history as usize][0]
                    .as_u64()
                    .unwrap()
                    .to_string()
            );
            assert_eq!(
                record.payload["conditional_mass"]["true_mass"],
                env["masses"][history as usize][1]
                    .as_u64()
                    .unwrap()
                    .to_string()
            );
        }
    }
}
#[test]
fn selection_and_unknown_fields_are_rejected_contextually() {
    for (value, field) in [
        (json!({"study":"testimony","fixture":"invented"}), "fixture"),
        (
            json!({"study":"testimony_game","environment":"invented","history":0,"listener":"passive"}),
            "environment",
        ),
        (
            json!({"study":"testimony_game","environment":"training","history":32,"listener":"passive"}),
            "history",
        ),
        (
            json!({"study":"testimony_game","environment":"training","history":0,"listener":"genetic_seed_20"}),
            "listener",
        ),
    ] {
        let errors = normalize_input(&value.to_string()).unwrap_err();
        assert_eq!(errors[0].field, field);
    }
    assert!(normalize_input(r#"{"study":"testimony","fixture":"transfer","records":[]}"#).is_err());
    assert!(normalize_input(r#"{"study":"testimony_game","environment":"training","history":31,"listener":"passive","live_truth":true}"#).is_err());
}
#[test]
fn wink_rejects_direct_wrong_study_calls() {
    assert!(wink::run(&testimony_input("empty_evidence")).is_err());
}
#[test]
fn all_original_models_are_unchanged_from_the_diagnostic_contract() {
    let diagnostic = deduction::diagnose_testimony().unwrap();
    let retained = retained();
    for (original, fixture) in diagnostic
        .fixtures
        .iter()
        .zip(retained["fixtures"].as_array().unwrap())
    {
        assert_eq!(
            serde_json::to_value(&original.model).unwrap(),
            fixture["model"]
        );
        assert_eq!(
            serde_json::to_value(&original.records).unwrap(),
            fixture["records"]
        );
    }
}
#[test]
fn conflicting_duplicate_id_terminates_without_mutation() {
    let value = retained()["fixtures"][1].clone();
    let model: deduction::TestimonyModel = serde_json::from_value(value["model"].clone()).unwrap();
    let mut records: Vec<deduction::EvidenceRecord> =
        serde_json::from_value(value["records"].clone()).unwrap();
    let mut conflicting = records[0].clone();
    if let deduction::TestimonyEvidence::Report {
        ref mut positive, ..
    } = conflicting.content
    {
        *positive = false;
    }
    records.push(conflicting);
    let input = testimony_input("uncertain_speaker");
    let record = super::super::testimony::run_evidence(&input, model, &records).unwrap();
    assert_eq!(
        record.checkpoints.last().unwrap().local["listener"]["snapshot"],
        record.checkpoints[1].local["listener"]["snapshot"]
    );
    assert!(
        record.checkpoints.last().unwrap().public["error"][0]["message"]
            .as_str()
            .unwrap()
            .contains("conflicting evidence ID 1")
    );
}
#[test]
fn declared_assumptions_are_checkpoint_local_without_future_evidence() {
    let record = run(&testimony_input("transfer")).unwrap();
    let fixture = retained()["fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "transfer")
        .unwrap()
        .clone();
    let expected = json!({"label":"Declared inference assumptions","model":wire::lossless_value(&fixture["model"]).unwrap()});
    for checkpoint in &record.checkpoints {
        assert_eq!(checkpoint.public["inference_assumptions"], expected);
    }
    assert!(record.checkpoints[0].public["evidence"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        record.checkpoints[1].public["evidence"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(!record.checkpoints[1].public["evidence"]
        .to_string()
        .contains("verified"));
}
#[test]
fn typed_invalid_inputs_and_evidence_capacity_fail_before_output() {
    let input = Input::Testimony {
        fixture: "x".repeat(MAX_INPUT_BYTES),
    };
    assert_eq!(
        super::super::testimony::run(&input).unwrap_err()[0].field,
        "input"
    );
    let fixture = retained()["fixtures"][1].clone();
    let model: deduction::TestimonyModel =
        serde_json::from_value(fixture["model"].clone()).unwrap();
    let records: Vec<deduction::EvidenceRecord> =
        serde_json::from_value(fixture["records"].clone()).unwrap();
    assert_eq!(
        super::super::testimony::run_evidence(
            &testimony_input("uncertain_speaker"),
            model,
            &vec![records[0].clone(); 513]
        )
        .unwrap_err()[0]
            .field,
        "records"
    );
}
#[test]
fn a_terminal_error_prevents_later_verification_delivery() {
    let fixture = retained()["fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "model_contradiction")
        .unwrap()
        .clone();
    let model: deduction::TestimonyModel =
        serde_json::from_value(fixture["model"].clone()).unwrap();
    let mut records: Vec<deduction::EvidenceRecord> =
        serde_json::from_value(fixture["records"].clone()).unwrap();
    records.push(deduction::EvidenceRecord {
        id: 3,
        content: deduction::TestimonyEvidence::Verified {
            proposition: 0,
            value: true,
        },
    });
    let record = super::super::testimony::run_evidence(
        &testimony_input("model_contradiction"),
        model,
        &records,
    )
    .unwrap();
    assert_eq!(record.checkpoints.len(), 3);
    assert!(!record
        .checkpoints
        .last()
        .unwrap()
        .public
        .to_string()
        .contains("verified"));
}
#[test]
fn testimony_rejects_direct_wrong_study_calls() {
    let input =
        normalize_input(r#"{"study":"wink","seed":"7","policy":"evidence","mode":"ordinary"}"#)
            .unwrap();
    assert_eq!(
        super::super::testimony::run(&input).unwrap_err()[0].field,
        "study"
    );
}
