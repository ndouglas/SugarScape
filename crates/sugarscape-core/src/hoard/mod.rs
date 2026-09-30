//! The evolution of larder hoarding (Minds 7): Vander Wall and Jenkins,
//! "Reciprocal pilferage and the evolution of food-hoarding behavior"
//! (Behavioral Ecology 14(5): 656–667, 2003), a genetic algorithm over a
//! population of 20 agents storing food through a 100-day season. See
//! docs/superpowers/specs/2026-09-30-minds-7-hoarding-evolution-design.md.

mod config;
mod stats;
mod world;

pub use config::{schema, DeadStores, DefendedInPool, HoardConfig, LarderWeight};
pub use stats::{HoardSnapshot, SERIES};
pub use world::{HoardWorld, TALL, WIDE};
