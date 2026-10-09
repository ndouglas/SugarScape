//! Bounded browser inputs and adapters around unchanged spatial lab engines.
pub mod budget;
pub mod input;
pub mod scenes;
use super::{error, EpisodeRecord, FieldError, Input, StudyDescriptor, StudyId};
pub use input::IdText;

pub fn validate_input(input: &Input) -> Result<(), Vec<FieldError>> {
    budget::preflight(input)
}
/// Each adapter adds its descriptor only after its original runner is wired.
pub(crate) fn descriptors() -> Vec<StudyDescriptor> {
    Vec::new()
}
pub(crate) fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>> {
    Err(error(
        "study",
        format!("spatial adapter {:?} is not implemented", input.study()),
    ))
}
pub(crate) fn rules_identity(study: StudyId) -> Result<String, Vec<FieldError>> {
    Err(error(
        "rules_identity",
        format!("spatial adapter {study:?} is not implemented"),
    ))
}
