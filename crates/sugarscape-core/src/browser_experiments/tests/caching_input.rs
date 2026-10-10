use crate::browser_experiments as browser;
pub(super) const P3_JSON: &str = r#"{"study":"protection_recaching","seed":"7","lab":{"policy":"selective","fixture":{"kind":"single","initial_observed":true,"redeposit_observed":false},"mirrored":false,"reburial_cost":0.25,"discovery":0.0,"exposure_span":"64","observer_span":64}}"#;
pub(super) const P4_JSON: &str = r#"{"study":"deception_gestures","seed":"7","lab":{"sender":"sham","view":"ambiguous","display_seen":true,"layout":"off_route","effort_cost":3.0,"mirrored":false}}"#;
#[test]
fn caching_exact_examples_normalize() {
    for raw in [P3_JSON, P4_JSON] {
        assert_eq!(
            serde_json::to_value(browser::normalize_input(raw).unwrap()).unwrap(),
            serde_json::from_str::<serde_json::Value>(raw).unwrap()
        );
    }
}
#[test]
fn caching_nested_fields_are_required_and_strict() {
    for raw in [P3_JSON, P4_JSON] {
        let base: serde_json::Value = serde_json::from_str(raw).unwrap();
        for field in base["lab"].as_object().unwrap().keys() {
            let mut input = base.clone();
            input["lab"].as_object_mut().unwrap().remove(field);
            assert!(
                browser::normalize_input(&input.to_string()).is_err(),
                "missing {field}"
            );
        }
        let mut input = base.clone();
        input["lab"]["extra"] = true.into();
        assert!(browser::normalize_input(&input.to_string()).is_err());
    }
    let mut input: serde_json::Value = serde_json::from_str(P3_JSON).unwrap();
    input["lab"]["fixture"]["extra"] = true.into();
    assert!(browser::normalize_input(&input.to_string()).is_err());
    input["lab"]["fixture"]
        .as_object_mut()
        .unwrap()
        .remove("extra");
    input["lab"]["fixture"]
        .as_object_mut()
        .unwrap()
        .remove("initial_observed");
    assert!(browser::normalize_input(&input.to_string()).is_err());
}
#[test]
fn caching_decimal_domains_are_lossless() {
    for raw in [P3_JSON, P4_JSON] {
        for invalid in ["", "07", "+7", "7.0", "18446744073709551616"] {
            let mut input: serde_json::Value = serde_json::from_str(raw).unwrap();
            input["seed"] = invalid.into();
            assert!(browser::normalize_input(&input.to_string()).is_err());
        }
        let input = raw.replace("\"seed\":\"7\"", "\"seed\":\"18446744073709551615\"");
        assert!(browser::normalize_input(&input).is_ok());
    }
    for invalid in ["0", "01", "-1", "18446744073709551616"] {
        let input = P3_JSON.replace(
            "\"exposure_span\":\"64\"",
            &format!("\"exposure_span\":\"{invalid}\""),
        );
        assert!(browser::normalize_input(&input).is_err());
    }
    assert!(
        browser::normalize_input(&P3_JSON.replace("\"64\"", "\"18446744073709551615\"")).is_ok()
    );
}
#[test]
fn caching_native_domain_validation_is_used() {
    for (raw, field, value) in [
        (P3_JSON, "reburial_cost", -1.0),
        (P3_JSON, "discovery", 1.1),
        (P3_JSON, "observer_span", 0.0),
        (P4_JSON, "effort_cost", 1.0),
    ] {
        let mut input: serde_json::Value = serde_json::from_str(raw).unwrap();
        input["lab"][field] = serde_json::json!(value);
        assert!(browser::normalize_input(&input.to_string()).is_err());
    }
}
#[test]
fn caching_every_fixture_variant_rejects_unknown_or_missing_fields() {
    for fixture in [
        serde_json::json!({"kind":"single","initial_observed":true,"redeposit_observed":false}),
        serde_json::json!({"kind":"mixed","observed_first":true}),
        serde_json::json!({"kind":"stumble","initial_observed":false}),
        serde_json::json!({"kind":"cue_visible_nonwatcher"}),
        serde_json::json!({"kind":"cue_unseen_watcher"}),
    ] {
        let mut input: serde_json::Value = serde_json::from_str(P3_JSON).unwrap();
        input["lab"]["fixture"] = fixture.clone();
        assert!(browser::normalize_input(&input.to_string()).is_ok());
        input["lab"]["fixture"]["extra"] = true.into();
        assert!(browser::normalize_input(&input.to_string()).is_err());
        for field in fixture.as_object().unwrap().keys() {
            input["lab"]["fixture"] = fixture.clone();
            input["lab"]["fixture"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                browser::normalize_input(&input.to_string()).is_err(),
                "missing {field}"
            );
        }
    }
}
#[test]
fn caching_numeric_seeds_spans_and_out_of_domain_enums_reject() {
    let mut input: serde_json::Value = serde_json::from_str(P3_JSON).unwrap();
    input["seed"] = serde_json::json!(7);
    assert!(browser::normalize_input(&input.to_string()).is_err());
    input = serde_json::from_str(P3_JSON).unwrap();
    input["lab"]["exposure_span"] = serde_json::json!(64);
    assert!(browser::normalize_input(&input.to_string()).is_err());
    for (raw, field) in [
        (P3_JSON, "policy"),
        (P4_JSON, "sender"),
        (P4_JSON, "view"),
        (P4_JSON, "layout"),
    ] {
        let mut input: serde_json::Value = serde_json::from_str(raw).unwrap();
        input["lab"][field] = "unsupported".into();
        assert!(browser::normalize_input(&input.to_string()).is_err());
    }
}
#[test]
fn caching_fixed_schedule_and_required_top_level_fields_reject_edits() {
    for raw in [P3_JSON, P4_JSON] {
        let base: serde_json::Value = serde_json::from_str(raw).unwrap();
        for field in ["study", "seed", "lab"] {
            let mut input = base.clone();
            input.as_object_mut().unwrap().remove(field);
            assert!(browser::normalize_input(&input.to_string()).is_err());
        }
        for field in ["ticks", "sample_every"] {
            let mut input = base.clone();
            input[field] = serde_json::json!(64);
            assert!(browser::normalize_input(&input.to_string()).is_err());
        }
    }
}
