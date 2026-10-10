//! Display adapters around the original fixed P3/P4 lab episodes.
pub mod input;
pub mod projection;
pub mod protection;
pub mod scenes;
use super::{error, EpisodeRecord, FieldError, Input, StudyDescriptor, StudyId, MAX_EPISODE_BYTES};
pub use input::{DeceptionInput, ProtectionInput};

/// Fixed rigs have 81 sites and two initial Agents, no births/replacement,
/// at most 65 frames, 81 cache keys and 162 (site,owner) seen keys per role.
/// Native frames reserve 64 KiB and projected frames 128 KiB (including
/// the repeated researcher frame); 256 KiB covers the native
/// two-cohort ledger/summary and 128 KiB covers the input/envelope. This includes
/// escaped enum/phase labels and longest u64 text, without changing inputs.
pub fn validate_input(input: &Input) -> Result<(), Vec<FieldError>> {
    match input {
        Input::ProtectionRecaching { lab, .. } => {
            lab.to_core()?;
        }
        Input::DeceptionGestures { lab, .. } => {
            lab.to_core()?;
        }
        _ => return Err(error("study", "caching adapter requires a caching study")),
    }
    let estimated = 65usize * (64 + 128) * 1024 + 384 * 1024;
    if estimated > MAX_EPISODE_BYTES {
        return Err(error(
            "episode",
            "fixed caching retention profile exceeds 16 MiB",
        ));
    }
    Ok(())
}
pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    validate_input(input)?;
    match input {
        Input::ProtectionRecaching { .. } => protection::run(input),
        _ => Err(error("study", "deception adapter is not implemented")),
    }
}
pub(crate) fn descriptors() -> Vec<StudyDescriptor> {
    vec![scenes::descriptor(
        StudyId::ProtectionRecaching,
        "protection_recaching",
        "Protection re-caching engineering demonstration",
        "Supplied P3 protection policies and original exposure-memory controller.",
        "What do recorded re-caching actions and perceived exposure show at each native boundary?",
    )]
}
pub fn rules_identity(study: StudyId) -> Result<String, Vec<FieldError>> {
    let key = match study {
        StudyId::ProtectionRecaching => "protection_recaching",
        StudyId::DeceptionGestures => "deception_gestures",
        _ => return Err(error("rules_identity", "not a caching study")),
    };
    let identities: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/caching-identities.json"))
            .map_err(|e| error("rules_identity", e.to_string()))?;
    let digest = identities[key]["source_sha256"]
        .as_str()
        .ok_or_else(|| error("rules_identity", "missing original source digest"))?;
    let schema = identities[key]["adapter_schema_version"]
        .as_u64()
        .ok_or_else(|| error("rules_identity", "missing caching adapter schema"))?;
    Ok(format!(
        "{key}:caching-adapter-{schema}:source-sha256:{digest}"
    ))
}
