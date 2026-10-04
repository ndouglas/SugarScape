use super::*;
use serde::{Deserialize, Serialize};
/// Privileged reproducibility transcript; never a controller observation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayArchive {
    pub protocol_version: u16,
    pub rules_version: u16,
    pub config: ScenarioConfig,
    pub seed: u64,
    pub responses: Vec<TurnResponse>,
    pub fingerprint: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    VersionMismatch,
    InvalidConfig(Vec<FieldError>),
    InvalidAction { index: usize },
    FingerprintMismatch,
}
impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionMismatch => f.write_str("archive version mismatch"),
            Self::InvalidConfig(_) => f.write_str("invalid archive configuration"),
            Self::InvalidAction { index } => {
                write!(f, "invalid archived response at index {index}")
            }
            Self::FingerprintMismatch => f.write_str("archive fingerprint mismatch"),
        }
    }
}
impl std::error::Error for ReplayError {}
pub fn replay(archive: &ReplayArchive) -> Result<Engine, ReplayError> {
    if archive.protocol_version != PROTOCOL_VERSION || archive.rules_version != RULES_VERSION {
        return Err(ReplayError::VersionMismatch);
    }
    let mut engine =
        Engine::new(archive.config.clone(), archive.seed).map_err(ReplayError::InvalidConfig)?;
    for (index, response) in archive.responses.iter().enumerate() {
        engine
            .submit(response.clone())
            .map_err(|_| ReplayError::InvalidAction { index })?;
    }
    if engine.fingerprint() != archive.fingerprint {
        return Err(ReplayError::FingerprintMismatch);
    }
    Ok(engine)
}
