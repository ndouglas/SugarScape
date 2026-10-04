use super::burrow::*;
use std::collections::BTreeSet;
#[test]
fn burrow_manifest_has_all_replicates_and_unique_ids() {
    let m = manifest().unwrap();
    assert_eq!(expected_keys(&m, Panel::Scientific).len(), 1280);
    assert_eq!(expected_keys(&m, Panel::Construction).len(), 36);
    let ids: BTreeSet<_> = m
        .conditions
        .iter()
        .chain(&m.construction_conditions)
        .map(|c| &c.id)
        .collect();
    assert_eq!(ids.len(), 50);
    assert!(!m.execution_authorized);
    assert!(m
        .scientific_seeds
        .iter()
        .all(|s| !m.construction_seeds.contains(s)));
}
#[test]
fn burrow_manifest_rejects_identity_changes() {
    let original = manifest().unwrap();
    for change in 0..6 {
        let mut m = original.clone();
        match change {
            0 => m.conditions[1].id = m.conditions[0].id.clone(),
            1 => m.conditions[0].config.response_weight = 4,
            2 => m.scientific_seeds[0] = "010001".into(),
            3 => m.primary_contrasts[0].plus = "wrong".into(),
            4 => m.expected_conditions = 31,
            _ => m.negative_control.comparison = "wrong".into(),
        }
        assert!(validate_manifest(&m).is_err());
    }
}
#[test]
fn burrow_manifest_requires_explicit_strict_fields() {
    let original: serde_json::Value = serde_json::from_slice(manifest_bytes()).unwrap();
    for field in [
        "fixture",
        "transport",
        "cue",
        "freshness_window",
        "relay_distance",
        "response_weight",
        "minimum_recent_units",
    ] {
        let mut v = original.clone();
        v["conditions"][0]["config"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(serde_json::from_value::<Manifest>(v).is_err(), "{field}");
    }
    for path in [0, 1, 2, 3] {
        let mut v = original.clone();
        let object = match path {
            0 => &mut v,
            1 => &mut v["conditions"][0],
            2 => &mut v["conditions"][0]["config"],
            _ => &mut v["conditions"][0]["options"],
        };
        object["unknown"] = true.into();
        assert!(serde_json::from_value::<Manifest>(v).is_err());
    }
}
#[test]
fn burrow_manifest_routes_are_explicit_and_non_running() {
    assert!(cli(&[]).is_ok());
    assert!(cli(&["--manifest".into()]).is_ok());
    for args in [
        vec!["--run"],
        vec!["--analyze"],
        vec!["--unknown"],
        vec!["--manifest", "--manifest"],
        vec!["--help", "--manifest"],
        vec!["--burrow"],
    ] {
        assert!(cli(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>()).is_err());
    }
}
#[test]
fn burrow_manifest_exact_hash_and_canonical_order() {
    assert_eq!(
        manifest_sha256(),
        "3ef020036fa96ebc5a440480dccfcab64fd8991dfc2ff2dc4129b05466acb890"
    );
    let m = manifest().unwrap();
    let keys = expected_keys(&m, Panel::Scientific);
    assert_eq!(keys[0].condition, m.conditions[0].id);
    assert_eq!(keys[0].seed, "10001");
    assert_eq!(keys[39].seed, "10040");
}

#[test]
fn burrow_manifest_preserves_panel_opportunity_budgets() {
    use sugarscape_core::burrow::Fixture;
    let m = manifest().unwrap();
    for c in m.conditions.iter().chain(&m.construction_conditions) {
        match c.config.fixture {
            Fixture::Growing { workers, .. } => {
                assert_eq!(workers * c.options.ticks, 4096, "{}", c.id)
            }
            Fixture::Corridor { workers, .. } => {
                assert_eq!(workers * c.options.ticks, 512, "{}", c.id)
            }
            Fixture::Choice { .. } => {
                assert_eq!(c.options.ticks, 1);
                assert_eq!(c.options.sample_every, 1);
            }
        }
    }
}

#[test]
fn burrow_manifest_analysis_with_index_explains_unavailable_route() {
    let error = cli(&["--analyze".into(), "index.json".into()]).unwrap_err();
    assert!(error.contains("unavailable"), "{error}");
}
