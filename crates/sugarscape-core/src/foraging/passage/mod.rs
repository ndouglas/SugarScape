//! Fixed passage foraging with immediate observations and private topology.
mod setup;
// Task 4 supplies navigation decision consumers.
#[allow(dead_code)]
mod navigation;
// Task 4 supplies the production consumers of draws and observations.
#[allow(dead_code)]
mod draws;
#[allow(dead_code)]
mod observation;
// Task 4 initializes, updates and exposes the private maps.
#[allow(dead_code)]
mod knowledge;
// Task 4 accumulates measured work from observations and navigation.
#[allow(dead_code)]
mod metrics;
pub use knowledge::{CellKnowledge, KnownCell};
pub use setup::{Parameters, Pos, Resource, Setup};
type Checked<T> = Result<T, Vec<crate::config::FieldError>>;
#[cfg(test)]
mod tests;
