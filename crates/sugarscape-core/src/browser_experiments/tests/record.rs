use super::super::*;
use serde_json::json;

fn input() -> Input {
    normalize_input(r#"{"study":"wink","seed":"7","policy":"evidence","mode":"ordinary"}"#).unwrap()
}

#[test]
fn strict_decimal_seeds_and_fields() {
    for seed in [
        json!(7),
        json!(""),
        json!("+7"),
        json!("-7"),
        json!(" 7"),
        json!("7.0"),
        json!("1e2"),
        json!("１８"),
        json!("18446744073709551616"),
    ] {
        let value = json!({"study":"wink","seed":seed,"policy":"evidence","mode":"ordinary"});
        assert!(normalize_input(&value.to_string()).is_err(), "{value}");
    }
    let mut value = serde_json::to_value(input()).unwrap();
    value["extra"] = json!(true);
    assert!(normalize_input(&value.to_string()).is_err());
}

#[test]
fn max_seed_stays_decimal_text() {
    let input = normalize_input(
        r#"{"study":"wink","seed":"18446744073709551615","policy":"passive","mode":"ordinary"}"#,
    )
    .unwrap();
    assert_eq!(run(&input).unwrap().input["seed"], "18446744073709551615");
}

#[test]
fn leading_zero_seed_normalizes_before_reconstruction() {
    let input =
        normalize_input(r#"{"study":"wink","seed":"0007","policy":"evidence","mode":"ordinary"}"#)
            .unwrap();
    assert_eq!(serde_json::to_value(input).unwrap()["seed"], "7");
}

#[test]
fn raw_input_byte_limit_includes_whitespace() {
    let input = serde_json::to_string(&input()).unwrap();
    assert!(normalize_input(&format!("{}{input}", " ".repeat(65_536))).is_err());
    assert!(normalize_input(&format!("{}{input}", " ".repeat(65_536 - input.len()))).is_ok());
}

#[test]
fn forged_checkpoint_cannot_validate() {
    let mut record = run(&input()).unwrap();
    record.checkpoints[0].public = json!({"forged":true});
    assert!(validate_episode(&serde_json::to_string(&record).unwrap()).is_err());
}

#[test]
fn all_saved_fields_are_reconstructed() {
    let record = run(&input()).unwrap();
    assert_eq!(
        validate_episode(&episode_json(&record).unwrap()).unwrap(),
        record
    );
    let original = serde_json::to_value(&record).unwrap();
    for pointer in [
        "/payload",
        "/input/seed",
        "/checkpoints/0/local",
        "/checkpoints/0/researcher",
        "/rules_identity",
        "/version",
        "/semantics",
        "/kind",
    ] {
        let mut forged = original.clone();
        *forged.pointer_mut(pointer).unwrap() = json!("forged");
        assert!(validate_episode(&forged.to_string()).is_err(), "{pointer}");
    }
    let mut forged = original;
    forged["passed"] = json!(true);
    assert!(validate_episode(&forged.to_string()).is_err());
}

#[test]
fn checkpoint_and_compact_output_limits_are_enforced() {
    let mut record = run(&input()).unwrap();
    record.checkpoints = vec![record.checkpoints[0].clone(); 4097];
    assert!(episode_json(&record).is_err());
    assert!(validate_episode(&serde_json::to_string(&record).unwrap()).is_err());
    record.checkpoints.truncate(1);
    record.payload = json!("x".repeat(16 * 1024 * 1024));
    assert!(episode_json(&record).is_err());
    assert!(validate_episode(&" ".repeat(16 * 1024 * 1024 + 1)).is_err());
}

#[test]
fn integer_payload_atoms_are_lossless_and_probabilities_stay_float() {
    let value = wire::lossless_value(&json!({"seed":u64::MAX,"negative":i64::MIN,"fraction":{"numerator":1,"denominator":3},"probability":0.5,"unit_probability":1.0})).unwrap();
    assert_eq!(
        value,
        json!({"seed":"18446744073709551615","negative":"-9223372036854775808","fraction":{"numerator":"1","denominator":"3"},"probability":0.5,"unit_probability":1.0})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&value.to_string()).unwrap(),
        value
    );
}

#[test]
fn only_working_catalog_entries_are_published() {
    for descriptor in catalog() {
        assert!(run(&normalize_input(&descriptor.default_input.to_string()).unwrap()).is_ok());
    }
    assert!(normalize_input(r#"{"study":"testimony","fixture":"unknown"}"#).is_err());
}
