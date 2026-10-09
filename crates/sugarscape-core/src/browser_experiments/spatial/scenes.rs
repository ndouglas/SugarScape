//! Source-bound supplied engineering demonstrations, without efficacy claims.
use crate::browser_experiments::{error, normalize_input, FieldError, Input};
use serde_json::Value;

pub fn input_for(id: &str) -> Result<Input, Vec<FieldError>> {
    let scenes: Value = serde_json::from_str(include_str!("../fixtures/spatial-scenes.json"))
        .map_err(|e| error("scene", e.to_string()))?;
    let row = scenes["scenes"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["id"] == id))
        .ok_or_else(|| error("scene", format!("unknown spatial engineering scene {id:?}")))?;
    normalize_input(&row["input"].to_string())
}
