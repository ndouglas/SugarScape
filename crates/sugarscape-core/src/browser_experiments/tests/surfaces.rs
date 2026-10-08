use super::super::*;
use crate::{active_surface as active, shared_surface as shared};
use serde_json::json;

fn active_input(
    role: shared::Role,
    policy: active::PolicyKind,
    environment: shared::Environment,
    sequence: u16,
) -> Input {
    normalize_input(&json!({"study":"active_surface","protocol":active::Protocol::new(role,policy),"environment":environment,"sequence":sequence}).to_string()).unwrap()
}
fn shared_input(model: shared::Mechanism, sequence: u16) -> Input {
    normalize_input(&json!({"study":"shared_surface","protocol":{"calibration_rounds":3,"pair":{"roles":["Unknown","Unknown"]},"prior_mode":"Treatment","ids":{"surface":"status-field","agents":["Agent-A","Agent-B"]}},"environment":{"InFamily":model},"sequence":sequence}).to_string()).unwrap()
}
#[test]
fn failed_dataflip_record_has_costs_but_no_terminal_net() {
    let record = run(&active_input(
        shared::Role::B,
        active::PolicyKind::InspectOnly,
        shared::Environment::DataFlip,
        0,
    ))
    .unwrap();
    assert_eq!(record.payload["metrics"][0]["spent"], "5");
    assert_eq!(record.payload["metrics"][1]["spent"], "7");
    assert!(record.payload["metrics"][0]["net"].is_null());
    assert_eq!(
        record.checkpoints.last().unwrap().clock,
        json!({"AfterSlot":{"phase":{"Live":{"trial":"0"}},"round":"3","slot":"1"}})
    );
}
#[test]
fn zero_probe_stop_is_free_and_precedes_bits() {
    let r = run(&active_input(
        shared::Role::A,
        active::PolicyKind::NoProbe,
        shared::Environment::InFamily(shared::Mechanism::SharedPersistent),
        255,
    ))
    .unwrap();
    assert_eq!(r.checkpoints[1].kind, "after_choice");
    assert_eq!(r.checkpoints[1].local["Agent-A"]["credits"], "48");
    assert!(r.checkpoints[1].local["Agent-A"]["private_bit"].is_null());
    assert_eq!(r.checkpoints[1].public["probe_stop"], "0");
}
#[test]
fn opaque_initial_agent_views_do_not_disclose_actual_mechanism() {
    let a = run(&shared_input(shared::Mechanism::PrivatePersistent, 0)).unwrap();
    let b = run(&shared_input(shared::Mechanism::Inert, 255)).unwrap();
    assert_eq!(a.checkpoints[0].local, b.checkpoints[0].local);
    assert_eq!(a.checkpoints[0].public, b.checkpoints[0].public);
    assert_eq!(
        a.checkpoints[0].local["Agent-A"]["surface"]["kind"],
        "opaque_surface"
    );
}
#[test]
fn private_routine_is_excluded_from_helper_view() {
    let r = run(&active_input(
        shared::Role::B,
        active::PolicyKind::InspectOnly,
        shared::Environment::InFamily(shared::Mechanism::SharedPersistent),
        0,
    ))
    .unwrap();
    let choice = r
        .checkpoints
        .iter()
        .position(|c| c.kind == "before_choice" && c.clock.get("BeforeSlot").is_some())
        .unwrap();
    assert_eq!(
        r.checkpoints[choice].local["Agent-A"],
        r.checkpoints[choice + 1].local["Agent-A"]
    );
    assert_eq!(
        r.checkpoints[choice].public,
        r.checkpoints[choice + 1].public
    );
    assert!(r.checkpoints[choice + 1].local["Agent-B"]["decision"].is_object());
}
#[test]
fn eight_catalog_studies_have_working_surface_entries() {
    assert_eq!(catalog().len(), 8);
}

#[test]
fn after_slot_beliefs_precede_reset_and_next_private_bits() {
    let r = run(&shared_input(shared::Mechanism::SharedResetting, 255)).unwrap();
    let i = r
        .checkpoints
        .iter()
        .position(|c| {
            c.clock == json!({"AfterSlot":{"phase":"Calibration","round":"1","slot":"4"}})
        })
        .unwrap();
    assert_ne!(
        r.checkpoints[i].researcher.as_ref().unwrap()["fields"][0]["symbol"],
        "Blank"
    );
    assert_eq!(
        r.checkpoints[i + 1].researcher.as_ref().unwrap()["fields"][0]["symbol"],
        "Blank"
    );
    assert!(r.checkpoints[i].local["Agent-A"]["private_bit"].is_null());
    assert!(r.checkpoints[i].local["Agent-A"]["prefix"]["entries"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .get("Action")
        .is_some());
}
#[test]
fn researcher_fields_follow_private_inert_and_dataflip_physics() {
    for model in [
        shared::Mechanism::PrivatePersistent,
        shared::Mechanism::Inert,
    ] {
        let r = run(&shared_input(model, 0)).unwrap();
        let first = r
            .checkpoints
            .iter()
            .find(|c| c.kind == "after_slot")
            .unwrap()
            .researcher
            .as_ref()
            .unwrap();
        assert_eq!(
            first["fields"][0]["symbol"],
            if model == shared::Mechanism::Inert {
                "Blank"
            } else {
                "Probe0"
            }
        );
        assert_eq!(first["fields"][1]["symbol"], "Blank");
    }
    let r = run(&active_input(
        shared::Role::B,
        active::PolicyKind::NoProbe,
        shared::Environment::DataFlip,
        0,
    ))
    .unwrap();
    let first = r
        .checkpoints
        .iter()
        .find(|c| c.kind == "after_slot")
        .unwrap()
        .researcher
        .as_ref()
        .unwrap();
    assert_eq!(first["fields"][0]["symbol"], "Data1");
    assert_eq!(first["fields"][0]["lineage"]["symbol"], "Data0");
}
#[test]
fn known_certainty_is_supplied_and_decisions_name_actual_continuation() {
    let r = run(&active_input(
        shared::Role::A,
        active::PolicyKind::Known,
        shared::Environment::InFamily(shared::Mechanism::SharedPersistent),
        0,
    ))
    .unwrap();
    assert_eq!(
        r.checkpoints[0].local["Agent-A"]["catalog_discovery"]["source"],
        "Supplied"
    );
    assert_eq!(
        r.checkpoints[1].local["Agent-A"]["decision"]["continuation_policy"],
        "Known"
    );
    let r = run(&shared_input(shared::Mechanism::SharedPersistent, 0)).unwrap();
    assert_eq!(
        r.checkpoints
            .iter()
            .find_map(|c| c.local["Agent-A"]["catalog_discovery"]["source"].as_str()),
        Some("Observed")
    );
}
#[test]
fn last_read_remains_an_observation_with_its_original_clock() {
    let r = run(&shared_input(shared::Mechanism::SharedPersistent, 255)).unwrap();
    let i = r
        .checkpoints
        .iter()
        .position(|c| !c.local["Agent-B"]["surface"]["last_observation"].is_null())
        .unwrap();
    assert_eq!(
        r.checkpoints[i].local["Agent-B"]["surface"],
        r.checkpoints[i + 1].local["Agent-B"]["surface"]
    );
    assert_eq!(
        r.checkpoints[i].local["Agent-B"]["surface"]["current_state"],
        "unobserved"
    );
}
#[test]
fn b_first_trial_choice_follows_a_unobserved_paid_write() {
    let r = run(&active_input(
        shared::Role::B,
        active::PolicyKind::NoProbe,
        shared::Environment::InFamily(shared::Mechanism::SharedPersistent),
        255,
    ))
    .unwrap();
    let c = r
        .checkpoints
        .iter()
        .find(|c| c.kind == "before_choice" && c.clock.get("BeforeSlot").is_some())
        .unwrap();
    assert_eq!(
        c.researcher.as_ref().unwrap()["credits"],
        json!(["47", "48"])
    );
    assert_eq!(c.local["Agent-B"]["credits"], "48");
    assert!(c.local["Agent-B"]["surface"]["last_observation"].is_null());
    assert_eq!(
        c.local["Agent-B"]["prefix"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["Physical"].get("Action").is_some())
            .count(),
        0
    );
}
#[test]
fn undeclared_active_controls_and_out_of_range_sequences_are_rejected() {
    for policy in [active::PolicyKind::Known, active::PolicyKind::FixedThree] {
        assert!(normalize_input(&json!({"study":"active_surface","protocol":active::Protocol::new(shared::Role::A,policy),"environment":"DataFlip","sequence":0}).to_string()).is_err());
    }
    let mut input =
        serde_json::to_value(shared_input(shared::Mechanism::SharedPersistent, 0)).unwrap();
    input["sequence"] = json!(256);
    assert!(normalize_input(&input.to_string()).is_err());
}

// The fixture stores bounded canonical full-payload reference digests extracted
// from the original immutable reports, alongside their SHA256 provenance.
fn fnv1a(bytes: &[u8]) -> String {
    let mut h = 0xcbf29ce484222325u64;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}
#[test]
fn all_active_controls_and_bounded_shared_cases_match_complete_first_report_episodes() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/surface-examples.json")).unwrap();
    for study in ["active_surface", "shared_surface"] {
        for example in fixture["studies"][study]["examples"].as_array().unwrap() {
            let input = normalize_input(&example["input"].to_string()).unwrap();
            let result = run(&input).unwrap();
            assert_eq!(
                fnv1a(result.payload.to_string().as_bytes()),
                example["transport_fnv1a64"].as_str().unwrap(),
                "{}",
                example["input"]
            );
            assert!(result.checkpoints.len() <= MAX_CHECKPOINTS);
        }
    }
}
#[test]
fn consistent_id_renaming_changes_no_behavior() {
    let input = active_input(
        shared::Role::B,
        active::PolicyKind::InspectOnly,
        shared::Environment::DataFlip,
        0,
    );
    let original = run(&input).unwrap();
    let renamed = serde_json::to_string(&input)
        .unwrap()
        .replace("Agent-A", "opaque-17")
        .replace("Agent-B", "opaque-93")
        .replace("status-field", "opaque-204");
    let result = run(&normalize_input(&renamed).unwrap()).unwrap();
    let restored = serde_json::to_string(&result.payload)
        .unwrap()
        .replace("opaque-17", "Agent-A")
        .replace("opaque-93", "Agent-B")
        .replace("opaque-204", "status-field");
    assert_eq!(
        original.payload,
        serde_json::from_str::<serde_json::Value>(&restored).unwrap()
    );
}

#[test]
fn hidden_peer_cost_and_field_do_not_change_same_helper_observation() {
    let env = shared::Environment::InFamily(shared::Mechanism::SharedPersistent);
    let inspect = run(&active_input(
        shared::Role::B,
        active::PolicyKind::InspectOnly,
        env,
        0,
    ))
    .unwrap();
    let communicate = run(&active_input(
        shared::Role::B,
        active::PolicyKind::NoProbe,
        env,
        0,
    ))
    .unwrap();
    let clock = json!({"AfterSlot":{"phase":{"Live":{"trial":"0"}},"round":"2","slot":"3"}});
    let a = inspect
        .checkpoints
        .iter()
        .find(|c| c.clock == clock)
        .unwrap();
    let b = communicate
        .checkpoints
        .iter()
        .find(|c| c.clock == clock)
        .unwrap();
    assert_ne!(
        a.researcher.as_ref().unwrap()["credits"],
        b.researcher.as_ref().unwrap()["credits"]
    );
    assert_ne!(
        a.researcher.as_ref().unwrap()["fields"],
        b.researcher.as_ref().unwrap()["fields"]
    );
    assert_eq!(a.local["Agent-A"], b.local["Agent-A"]);
    assert_eq!(a.public, b.public);
}
#[test]
fn supported_wrong_dataflip_keeps_nominal_certainty_separate_from_truth() {
    let r = run(&active_input(
        shared::Role::A,
        active::PolicyKind::Adaptive,
        shared::Environment::DataFlip,
        0,
    ))
    .unwrap();
    assert!(r.payload["failure"].is_null());
    assert_eq!(r.payload["metrics"][0]["correct"], "0");
    assert_eq!(r.payload["metrics"][0]["net"], "-26");
    assert!(r.payload["model_projections"][0]["true_model_mass"].is_null());
    assert_eq!(
        r.checkpoints.last().unwrap().local["Agent-A"]["catalog_discovery"]["model"],
        "SharedPersistent"
    );
}
