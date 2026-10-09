use serde::{Deserialize, Serialize};
use sugarscape_core::minds::behavior_tree::{lab, state::LabConfig};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub id: String,
    pub lab: LabConfig,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub conditions: Vec<Condition>,
    pub seeds: Vec<u64>,
}
pub fn id(lab: &LabConfig) -> String {
    let controller = serde_json::to_string(&lab.controller)
        .unwrap()
        .trim_matches('"')
        .replace('_', "-");
    let scenario = serde_json::to_string(&lab.scenario)
        .unwrap()
        .trim_matches('"')
        .replace('_', "-");
    format!(
        "{controller}-{scenario}-quota{}-m{}",
        lab.quota,
        u8::from(lab.mirrored)
    )
}
pub fn manifest() -> Manifest {
    let mut conditions: Vec<_> = lab::conditions()
        .into_iter()
        .map(|lab| Condition { id: id(&lab), lab })
        .collect();
    conditions.sort_by(|a, b| a.id.cmp(&b.id));
    Manifest {
        schema: "minds-behavior-tree-manifest-v1".into(),
        conditions,
        seeds: (30001..=30040).collect(),
    }
}
