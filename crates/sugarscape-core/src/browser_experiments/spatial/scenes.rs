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

/// Only called for an adapter whose implementation has passed its task gate.
pub(crate) fn descriptor(
    id: super::StudyId,
    key: &str,
    title: &str,
    supplied: &str,
    question: &str,
) -> super::StudyDescriptor {
    let definitions: Value = serde_json::from_str(include_str!("../fixtures/spatial-scenes.json"))
        .expect("retained engineering scenes");
    let study = &definitions["studies"][key];
    let input = input_for(study["default_scene"].as_str().expect("default scene ID"))
        .expect("validated default scene");
    super::StudyDescriptor {
        id,
        family: crate::browser_experiments::StudyFamily::Spatial,
        title: title.into(),
        supplied: supplied.into(),
        question: question.into(),
        default_input: serde_json::to_value(input).expect("serializable normalized input"),
        controls: study["controls"].clone(),
    }
}
