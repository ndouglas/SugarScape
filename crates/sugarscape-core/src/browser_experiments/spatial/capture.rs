//! Shared checkpoint wire schema and charging, independent of native Worlds.
use super::budget::CaptureBudget;
use crate::browser_experiments::{wire::lossless_value, Checkpoint, FieldError};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

type Result<T> = std::result::Result<T, Vec<FieldError>>;

/// Charge the envelope/public/researcher occurrence before inserting local data.
pub(super) fn begin_checkpoint<T: Serialize>(
    snapshot: &T,
    completed_ticks: u32,
    kind: &str,
    index: u32,
    public: &Value,
    budget: &mut CaptureBudget,
) -> Result<Checkpoint> {
    let checkpoint = Checkpoint {
        index,
        clock: json!({"completed_ticks":completed_ticks.to_string()}),
        kind: kind.into(),
        public: public.clone(),
        local: BTreeMap::new(),
        researcher: Some(json!({"snapshot":lossless_value(snapshot)?})),
    };
    // Local entries are charged separately, once each. Charging the complete
    // checkpoint again would count every retained sparse map twice.
    budget.charge(&checkpoint)?;
    Ok(checkpoint)
}

/// Pair only the supplied original own AgentView and captured KnowledgeView.
/// Charge the actual keyed entry before moving it into checkpoint.local.
pub(super) fn insert_local<A: Serialize, K: Serialize>(
    checkpoint: &mut Checkpoint,
    id: u32,
    agent: &A,
    knowledge: &K,
    budget: &mut CaptureBudget,
) -> Result<()> {
    let local = lossless_value(&json!({
        "agent":agent,"knowledge":knowledge,"capture_at":checkpoint.clock,
        "cell_observed_at":null,
        "knowledge_availability":"captured private topology beliefs; per-cell observation times were not recorded"
    }))?;
    let entry = BTreeMap::from([(id.to_string(), local)]);
    // Includes the actual ID key/colon and object punctuation, not just value.
    budget.charge(&entry)?;
    checkpoint.local.extend(entry);
    Ok(())
}
