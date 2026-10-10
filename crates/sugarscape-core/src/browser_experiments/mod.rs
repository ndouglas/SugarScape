//! Bounded, reproducible display adapters for retained experiment engines.
pub mod caching;
mod catalog;
mod equivalence;
mod input;
mod record;
mod recorded;
pub use recorded::recorded_results_json;
pub mod reporting;
pub mod spatial;
pub mod surfaces;
pub mod testimony;
pub mod wink;
pub mod wire;

pub use crate::config::FieldError;
pub use catalog::{catalog, StudyDescriptor, StudyFamily};
pub use input::{normalize_input, Input, SeedText, WinkMode};
pub use record::{
    episode_json, validate_episode, Checkpoint, EpisodeKind, EpisodeRecord, Semantics,
    EPISODE_VERSION,
};
use serde::{Deserialize, Serialize};

pub const MAX_INPUT_BYTES: usize = 64 * 1024;
pub const MAX_CHECKPOINTS: usize = 4096;
pub const MAX_EPISODE_BYTES: usize = 16 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyId {
    ProtectionRecaching,
    DeceptionGestures,
    BurrowExcavation,
    BurrowAccess,
    ForagingFixed,
    ForagingPassage,
    ForagingConstruction,
    Wink,
    Testimony,
    TestimonyGame,
    StrategicReporting,
    StrategyInference,
    AdversarialAudit,
    SharedSurface,
    ActiveSurface,
}

pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    // Revalidate typed inputs too as other adapters acquire bounded selectors.
    let input = normalize_input(&record::bounded_json(input, MAX_INPUT_BYTES, "input")?)?;
    match input {
        Input::ProtectionRecaching { .. } | Input::DeceptionGestures { .. } => caching::run(&input),
        Input::BurrowExcavation { .. }
        | Input::BurrowAccess { .. }
        | Input::ForagingFixed { .. }
        | Input::ForagingPassage { .. }
        | Input::ForagingConstruction { .. } => spatial::run(&input),
        Input::Wink { .. } => wink::run(&input),
        Input::SharedSurface { .. } | Input::ActiveSurface { .. } => surfaces::run(&input),
        Input::Testimony { .. } | Input::TestimonyGame { .. } => testimony::run(&input),
        _ => reporting::run(&input),
    }
}
pub(crate) fn error(field: &str, message: impl Into<String>) -> Vec<FieldError> {
    vec![FieldError::new(field, message)]
}
#[cfg(test)]
mod tests;
