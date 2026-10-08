use super::super::{
    manifest::{candidate, condition},
    validate::validate_episode,
    wire::*,
    CollectionMode, RunKey,
};
use super::support::*;
#[test]
fn original_envelope_round_trips_without_float_token_loss() {
    let bytes = encoded_fixture("route.straight.protected", 7).unwrap();
    let e = decode_fixture("route.straight.protected", 7).unwrap();
    assert_eq!(serde_json::to_vec(&e).unwrap(), bytes);
}
#[test]
fn total_opportunities_must_match_completed_clock() {
    let m = candidate().unwrap();
    let c = condition(&m, "route.straight.protected").unwrap();
    let mut e = decode_fixture(&c.id, 7).unwrap();
    e.episode
        .snapshots
        .last_mut()
        .unwrap()
        .summary
        .work
        .opportunities = 4097;
    e.episode.summary = e.episode.snapshots.last().unwrap().summary.clone();
    assert!(validate_episode(c, &e.key, &e.episode)
        .unwrap_err()
        .contains("work.opportunities"));
}
#[test]
fn finite_float_tokens_preserve_lexical_form() {
    for token in ["0.010000000000000002", "1e-12", "-0.0", "1"] {
        let f: WireFloat = serde_json::from_str(token).unwrap();
        assert_eq!(serde_json::to_string(&f).unwrap(), token);
    }
}
#[test]
fn finite_float_tokens_reject_non_numbers_nonfinite_and_long_tokens() {
    for token in ["null", "\"1\"", "true", "NaN", "1e999", "{}"] {
        assert!(serde_json::from_str::<WireFloat>(token).is_err(), "{token}");
    }
    assert!(serde_json::from_str::<WireFloat>(&format!("0.{}", "1".repeat(65))).is_err());
}
#[test]
fn encoder_caps_are_exact() {
    let e = cached_candidate_episode("route.straight.protected", 7).unwrap();
    let key = RunKey {
        condition: "route.straight.protected".into(),
        seed: 7,
    };
    let p = test_provenance();
    let b = encode_envelope(&key, CollectionMode::Construction, &p, &e, u64::MAX).unwrap();
    assert_eq!(
        encode_envelope(&key, CollectionMode::Construction, &p, &e, b.len() as u64).unwrap(),
        b
    );
    assert!(encode_envelope(
        &key,
        CollectionMode::Construction,
        &p,
        &e,
        b.len() as u64 - 1
    )
    .is_err());
}
#[test]
fn fixture_refuses_scientific_seed_before_core_run() {
    assert!(cached_candidate_episode("route.straight.protected", 10001).is_err());
}

fn decode_bytes(bytes: &[u8]) -> Result<WireEnvelope, String> {
    decode_envelope(
        bytes,
        &RunKey {
            condition: "route.straight.protected".into(),
            seed: 7,
        },
        CollectionMode::Construction,
        &test_provenance(),
        4 * 1024 * 1024,
    )
}
#[test]
fn strict_decode_rejects_unknown_duplicate_trailing_and_utf8_fields() {
    let bytes = encoded_fixture("route.straight.protected", 7).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    for bad in [
        text.replacen("{", "{\"intruder\":0,", 1),
        text.replacen("\"seed\":7", "\"seed\":7,\"seed\":7", 1),
        format!("{text}null"),
        text.replace("\"opportunities\":0", "\"opportunities\":-1"),
        text.replace("\"completed_ticks\":0", "\"completed_ticks\":1.0"),
        text.replace("\"Departing\"", "\"Flying\""),
        text.replace("\"p_search\":0.05", "\"p_search\":null"),
    ] {
        assert!(decode_bytes(bad.as_bytes()).is_err());
    }
    let mut bad = bytes.clone();
    bad[0] = 255;
    assert!(decode_bytes(&bad).is_err());
    assert!(decode_envelope(
        &bytes,
        &RunKey {
            condition: "route.straight.protected".into(),
            seed: 7
        },
        CollectionMode::Construction,
        &test_provenance(),
        bytes.len() as u64 - 1
    )
    .is_err());
}
#[test]
fn identity_fields_bind_before_episode_validation() {
    let e = decode_fixture("route.straight.protected", 7).unwrap();
    for index in 0..5 {
        let mut bad = e.clone();
        match index {
            0 => bad.schema = "alien".into(),
            1 => bad.key.seed = 8,
            2 => bad.mode = CollectionMode::Scientific,
            3 => bad.provenance.collector_sha256 = "d".repeat(64),
            _ => bad.episode.seed = 8,
        }
        assert!(decode_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}
#[test]
fn normalized_input_frames_summary_and_snapshot_bytes_are_bound() {
    let m = candidate().unwrap();
    let c = condition(&m, "route.straight.protected").unwrap();
    let valid = decode_fixture(&c.id, 7).unwrap();
    validate_episode(c, &valid.key, &valid.episode).unwrap();
    for index in 0..10 {
        let mut bad = valid.clone();
        match index {
            0 => bad.episode.setup.parameters.p_search = WireFloat("0.050".into()),
            1 => bad.episode.options.ticks = 511,
            2 => {
                bad.episode.snapshots.remove(0);
            }
            3 => {
                bad.episode.snapshots.pop();
            }
            4 => bad.episode.snapshots[1] = bad.episode.snapshots[0].clone(),
            5 => bad.episode.summary.expired_records += 1,
            6 => bad.episode.snapshot_bytes += 1,
            7 => bad.episode.setup.workers.swap(0, 7),
            8 => bad.key.condition = "route.detour.protected".into(),
            _ => bad.episode.seed = 8,
        }
        assert!(
            validate_episode(c, &bad.key, &bad.episode).is_err(),
            "mutation {index}"
        );
    }
}
#[test]
fn hash_consistent_malicious_body_still_fails_observed_validation() {
    let m = candidate().unwrap();
    let c = condition(&m, "route.straight.protected").unwrap();
    let mut e = decode_fixture(&c.id, 7).unwrap();
    e.episode.snapshots[1].summary.compute.peak_queue = u64::MAX;
    let bytes = serde_json::to_vec(&e).unwrap();
    assert_eq!(super::super::sha256(&bytes).len(), 64);
    let decoded = decode_bytes(&bytes).unwrap();
    assert!(validate_episode(c, &decoded.key, &decoded.episode).is_err());
}
#[test]
fn missing_nullable_fields_are_schema_errors() {
    let bytes = encoded_fixture("route.straight.protected", 7).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let fields = [
        (
            "/episode/snapshots/0/agents/0",
            &["cargo", "find", "site", "frontier", "face"][..],
        ),
        (
            "/episode/snapshots/0/summary/access/records/0",
            &["first_exposure", "first_access", "distance"][..],
        ),
        (
            "/episode/snapshots/0/summary/milestones",
            &[
                "first_excavation",
                "first_exposure",
                "first_access",
                "first_disposal",
                "first_pickup_tick",
                "first_delivery_tick",
                "all_food_delivered_tick",
            ][..],
        ),
    ];
    for (path, fields) in fields {
        for field in fields {
            let mut bad = value.clone();
            assert!(
                bad.pointer_mut(path)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(*field)
                    .is_some(),
                "fixture field {path}/{field}"
            );
            assert!(
                decode_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
                "missing {path}/{field}"
            );
        }
    }
}
#[test]
fn every_fixed_construction_key_validates_without_scientific_runs() {
    let m = candidate().unwrap();
    for c in &m.conditions {
        for seed in [7, 8] {
            let e = decode_fixture(&c.id, seed).unwrap();
            validate_episode(c, &e.key, &e.episode).unwrap();
        }
    }
}
