//! Frozen explanatory inputs; preset deltas are declared before viewer evaluation.
use crate::browser_experiments::{
    error, normalize_input, FieldError, Input, StudyDescriptor, StudyFamily, StudyId,
};
use serde_json::Value;
pub fn input_for(id: &str) -> Result<Input, Vec<FieldError>> {
    let definitions: Value = serde_json::from_str(include_str!("../fixtures/caching-scenes.json"))
        .map_err(|e| error("scene", e.to_string()))?;
    let row = definitions["scenes"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["id"] == id))
        .ok_or_else(|| error("scene", format!("unknown caching engineering scene {id:?}")))?;
    normalize_input(&row["input"].to_string())
}
pub(crate) fn descriptor(
    id: StudyId,
    key: &str,
    title: &str,
    supplied: &str,
    question: &str,
) -> StudyDescriptor {
    let definitions: Value = serde_json::from_str(include_str!("../fixtures/caching-scenes.json"))
        .expect("retained caching scenes");
    let study = &definitions["studies"][key];
    let input = input_for(study["default_scene"].as_str().expect("default scene ID"))
        .expect("validated default scene");
    StudyDescriptor {
        id,
        family: StudyFamily::Spatial,
        title: title.into(),
        supplied: supplied.into(),
        question: question.into(),
        default_input: serde_json::to_value(input).expect("normalized input"),
        controls: study["controls"].clone(),
    }
}
