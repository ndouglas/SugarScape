use super::super::*;
use crate::deduction::{self, Controller};
use serde_json::{json, Value};

fn episode(seed: &str, policy: &str, mode: &str) -> EpisodeRecord {
    run(&normalize_input(
        &json!({"study":"wink","seed":seed,"policy":policy,"mode":mode}).to_string(),
    )
    .unwrap())
    .unwrap()
}

#[test]
fn ordinary_evidence_matches_retained_cli_fingerprint() {
    let record = episode("7", "evidence", "ordinary");
    assert_eq!(
        record.payload["fingerprint"],
        format!("{:016x}", 2685439217561942331u64)
    );
    assert_eq!(record, episode("7", "evidence", "ordinary"));
}

#[test]
fn diagnostic_matches_retained_game_outcomes_and_differs_from_ordinary() {
    for (seed, policy, winner, rounds) in [
        ("0", "reckless", "threat", "1"),
        ("0", "passive", "threat", "5"),
        ("0", "evidence", "accuser", "3"),
        ("7", "reckless", "threat", "1"),
        ("7", "passive", "threat", "6"),
        ("7", "evidence", "accuser", "1"),
    ] {
        let record = episode(seed, policy, "diagnostic");
        assert_eq!(record.payload["outcome"]["winner"], winner);
        assert_eq!(record.payload["outcome"]["completed_rounds"], rounds);
    }
    assert_ne!(
        episode("7", "passive", "ordinary").payload["fingerprint"],
        episode("7", "passive", "diagnostic").payload["fingerprint"]
    );
}

#[test]
fn checkpoint_requests_and_pending_boundaries_follow_engine_exactly() {
    let record = episode("7", "evidence", "ordinary");
    let mut engine = deduction::Engine::new(deduction::wink_config(6), 7).unwrap();
    let mut controllers: Vec<_> = (0..6)
        .map(|id| {
            deduction::BuiltinController::new(
                deduction::PolicyKind::Evidence,
                deduction::policy_seed(7, deduction::PolicyKind::Evidence, id),
            )
        })
        .collect();
    let mut delivered = std::collections::BTreeMap::new();
    let mut checkpoints = record.checkpoints.iter();
    while let Some(request) = engine.request() {
        let before = checkpoints.next().unwrap();
        let id = request.actor.to_string();
        delivered.insert(id.clone(), wire::lossless_value(&request).unwrap());
        assert_eq!(before.local.len(), delivered.len());
        for (id, request) in &delivered {
            assert_eq!(&before.local[id]["request"], request);
        }
        assert_eq!(before.local[&id]["delivered_clock"], before.clock);
        assert_eq!(
            before.public["roster"],
            wire::lossless_value(&request.observation.roster).unwrap()
        );
        let response = controllers[usize::from(request.actor)].respond(&request);
        engine.submit(response).unwrap();
        let next = engine.request();
        let committed = next
            .as_ref()
            .is_none_or(|r| r.round != request.round || r.phase != request.phase);
        let after = checkpoints.next().unwrap();
        assert_eq!(
            after.public["submission_status"],
            if committed { "committed" } else { "pending" }
        );
        // A request for the next actor is not yet delivered in the after checkpoint.
        for (id, request) in &delivered {
            assert_eq!(&after.local[id]["request"], request);
        }
        if next.is_none() {
            assert_eq!(after.public["roster"], Value::Null);
        }
    }
    assert!(checkpoints.next().is_none());
    assert_eq!(
        record.payload["archive"]["responses"],
        wire::lossless_value(&engine.archive().responses).unwrap()
    );
    assert_eq!(
        deduction::replay(&engine.archive()).unwrap().fingerprint(),
        engine.fingerprint()
    );
}

#[test]
fn public_projection_excludes_private_evidence_and_model_beliefs() {
    let record = episode("7", "evidence", "ordinary");
    assert_eq!(record.checkpoints[0].local.len(), 1);
    for checkpoint in &record.checkpoints {
        let public = checkpoint.public.to_string();
        assert!(
            !public.contains("grants")
                && !public.contains("objective\"")
                && !public.contains("recipient_notice")
        );
        assert!(!serde_json::to_string(checkpoint)
            .unwrap()
            .contains("posterior"));
        for event in checkpoint.public["events"].as_array().unwrap() {
            assert!(matches!(
                event["content"]["kind"].as_str(),
                Some("status_change" | "claim")
            ));
        }
    }
}
