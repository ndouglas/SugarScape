//! Only runnable adapters appear in this catalog.
use super::{FieldError, StudyId};
use crate::deduction;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyFamily {
    Game,
    Testimony,
    Surface,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyDescriptor {
    pub id: StudyId,
    pub family: StudyFamily,
    pub title: String,
    pub supplied: String,
    pub question: String,
    pub default_input: Value,
    pub controls: Value,
}
pub fn catalog() -> Vec<StudyDescriptor> {
    let mut studies = vec![StudyDescriptor {
        id: StudyId::Wink, family: StudyFamily::Game, title: "Wink deduction".into(),
        supplied: "Six Agents, bounded attention and memory, delayed effects, and request-only controllers.".into(),
        question: "When does local evidence support identifying the capability holder?".into(),
        default_input: json!({"study":"wink","seed":"7","policy":"evidence","mode":"ordinary"}),
        controls: json!({"seed":{"type":"decimal_u64"},"policy":{"values":["evidence","random","reckless","passive"]},"mode":{"values":["ordinary","diagnostic"]}}),
    }];
    studies.extend(super::testimony::descriptors());
    studies
}
pub(crate) fn rules_identity(study: StudyId) -> Result<String, Vec<FieldError>> {
    match study {
        StudyId::Wink => {
            let receipts: Value =
                serde_json::from_str(include_str!("fixtures/engine-identities.json"))
                    .map_err(|e| super::error("rules_identity", e.to_string()))?;
            let digest = receipts["wink"]["source_sha256"]
                .as_str()
                .ok_or_else(|| super::error("rules_identity", "missing retained source receipt"))?;
            Ok(format!(
                "wink:protocol-{}:rules-{}:policy-{}:source-sha256:{digest}",
                deduction::PROTOCOL_VERSION,
                deduction::RULES_VERSION,
                deduction::POLICY_VERSION
            ))
        }
        StudyId::Testimony | StudyId::TestimonyGame => super::testimony::rules_identity(study),
        _ => Err(super::error(
            "study",
            "study adapter is not available in this viewer version",
        )),
    }
}
