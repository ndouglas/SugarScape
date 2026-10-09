//! Bounded browser inputs and adapters around unchanged spatial lab engines.
pub mod budget;
pub mod burrow;
mod capture;
pub mod construction;
pub mod fixed;
pub mod input;
pub mod passage;
pub mod scenes;
use super::{error, EpisodeRecord, FieldError, Input, StudyDescriptor, StudyId};
pub use input::IdText;

pub fn validate_input(input: &Input) -> Result<(), Vec<FieldError>> {
    budget::preflight(input)
}
/// Each adapter adds its descriptor only after its original runner is wired.
pub(crate) fn descriptors() -> Vec<StudyDescriptor> {
    [
        (
            StudyId::BurrowExcavation,
            "burrow_excavation",
            "Burrow excavation engineering demonstration",
            "Supplied excavation fixtures and original preference/transport controllers.",
            "What do the recorded action and material histories show at each sampled boundary?",
        ),
        (
            StudyId::BurrowAccess,
            "burrow_access",
            "Burrow access engineering demonstration",
            "Supplied structural-access task with Explore or KnownGoal information.",
            "When does structural access occur, and which Agents have locally observed completion?",
        ),
        (
            StudyId::ForagingFixed,
            "foraging_fixed",
            "Fixed-world foraging engineering demonstration",
            "Supplied arena and original fixed-world CPFA controller.",
            "What do original completed-tick snapshots and each Agent's own captured state show?",
        ),
        (
            StudyId::ForagingPassage,
            "foraging_passage",
            "Passage foraging engineering demonstration",
            "Supplied passage geometry and original private-memory CPFA controller.",
            "What do physical snapshots and each Agent's captured topology beliefs show?",
        ),
        (
            StudyId::ForagingConstruction,
            "foraging_construction",
            "Construction foraging engineering demonstration",
            "Supplied construction task and original private-memory CPFA controller.",
            "How do recorded physical state and private topology beliefs differ at each boundary?",
        ),
    ]
    .into_iter()
    .map(|(id, key, title, supplied, question)| {
        scenes::descriptor(id, key, title, supplied, question)
    })
    .collect()
}
pub(crate) fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    if matches!(
        input,
        Input::BurrowExcavation { .. } | Input::BurrowAccess { .. }
    ) {
        return burrow::run(input);
    }
    if matches!(input, Input::ForagingFixed { .. }) {
        return fixed::run(input);
    }
    if matches!(input, Input::ForagingPassage { .. }) {
        return passage::run(input);
    }
    if matches!(input, Input::ForagingConstruction { .. }) {
        return construction::run(input);
    }
    Err(error(
        "study",
        format!("spatial adapter {:?} is not implemented", input.study()),
    ))
}
pub(crate) fn rules_identity(study: StudyId) -> Result<String, Vec<FieldError>> {
    let key = match study {
        StudyId::BurrowExcavation => "burrow_excavation",
        StudyId::BurrowAccess => "burrow_access",
        StudyId::ForagingFixed => "foraging_fixed",
        StudyId::ForagingPassage => "foraging_passage",
        StudyId::ForagingConstruction => "foraging_construction",
        _ => {
            return Err(error(
                "rules_identity",
                format!("spatial adapter {study:?} is not implemented"),
            ))
        }
    };
    let identities: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/spatial-identities.json"))
            .map_err(|e| error("rules_identity", e.to_string()))?;
    let digest = identities[key]["source_sha256"]
        .as_str()
        .ok_or_else(|| error("rules_identity", "missing original source digest"))?;
    let schema = identities[key]["adapter_schema_version"]
        .as_u64()
        .ok_or_else(|| error("rules_identity", "missing adapter schema version"))?;
    let source = format!("{key}:spatial-adapter-{schema}:source-sha256:{digest}");
    if matches!(
        study,
        StudyId::ForagingFixed | StudyId::ForagingPassage | StudyId::ForagingConstruction
    ) {
        Ok(format!("{source}:target:{}", target_tag()))
    } else {
        Ok(source)
    }
}

/// Actual compilation target, shared by the CPFA source identity and capture.
pub(crate) fn target_tag() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}
