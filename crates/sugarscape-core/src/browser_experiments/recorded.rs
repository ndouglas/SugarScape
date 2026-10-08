//! Source-bound retained measurements. This module never runs training or diagnostics.
use super::{record, wire::lossless_value, FieldError, MAX_EPISODE_BYTES};
use serde_json::{json, Value};
use std::sync::OnceLock;
const MAX_ASSET_BYTES: usize = 5 * 1024 * 1024;
static STRATEGIC: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static STRATEGY: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static AUDIT: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static GAME: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
/// Embedded immutable assets are parsed once; no simulation state is cached.
pub(super) fn retained(study: &str) -> Result<&'static Value, Vec<FieldError>> {
    let (slot, text) = match study {
        "strategic_reporting" => (&STRATEGIC, include_str!("fixtures/strategic-recorded.json")),
        "strategy_inference" => (&STRATEGY, include_str!("fixtures/strategy-recorded.json")),
        "adversarial_audit" => (&AUDIT, include_str!("fixtures/audit-recorded.json")),
        "testimony_game" => (
            &GAME,
            include_str!("fixtures/testimony-search-recorded.json"),
        ),
        _ => return Err(super::error("study", "unknown retained study")),
    };
    slot.get_or_init(|| {
        if text.len() > MAX_ASSET_BYTES {
            return Err(super::error("fixture", "retained asset exceeds 5 MiB"));
        }
        let value: Value =
            serde_json::from_str(text).map_err(|e| super::error("fixture", e.to_string()))?;
        if value["transformation_version"] != "browser-retained-v1"
            || value["provenance"]["sha256"]
                .as_str()
                .is_none_or(|s| s.len() != 64)
            || value["identity"]["source_sha256"]
                .as_str()
                .is_none_or(|s| s.len() != 64)
        {
            return Err(super::error("fixture", "missing retained source identity"));
        }
        Ok(value)
    })
    .as_ref()
    .map_err(Clone::clone)
}
/// Complete original GA/random campaigns, separately identified by study/source.
/// Curves use the declared columns and champion dictionary without dropping points.
pub fn recorded_results_json() -> Result<String, Vec<FieldError>> {
    let mut studies = serde_json::Map::new();
    for study in ["testimony_game", "strategic_reporting"] {
        let mut value = retained(study)?.clone();
        if value["runs"].as_array().is_none_or(|r| r.len() != 40) {
            return Err(super::error(
                "recorded",
                "retained search campaign must contain all 40 runs",
            ));
        }
        // Selected-case references serve the case adapter, not the search panel.
        let object = value
            .as_object_mut()
            .ok_or_else(|| super::error("fixture", "expected retained object"))?;
        object.remove("case_references");
        object.remove("policies");
        studies.insert(study.into(), lossless_value(&value)?);
    }
    record::bounded_json(
        &json!({"kind":"recorded_results","version":1,"studies":studies}),
        MAX_EPISODE_BYTES,
        "recorded",
    )
}
