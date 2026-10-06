//! Fixed passage foraging with immediate observations and private topology.
mod setup;
// Task 4 supplies the production consumers of draws and observations.
#[allow(dead_code)]
mod draws;
#[allow(dead_code)]
mod observation;
// Task 2 supplies routing consumers; Task 4 initializes and updates maps.
#[allow(dead_code)]
mod knowledge;
// Tasks 2 and 4 accumulate computational work.
#[allow(dead_code)]
mod metrics;
pub use knowledge::{CellKnowledge, KnownCell};
pub use setup::{Parameters, Pos, Resource, Setup};
type Checked<T> = Result<T, Vec<crate::config::FieldError>>;
#[cfg(test)]
mod tests;
