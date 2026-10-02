//! The evolution of larder hoarding (Minds 7): Vander Wall and Jenkins,
//! "Reciprocal pilferage and the evolution of food-hoarding behavior"
//! (Behavioral Ecology 14(5): 656–667, 2003), a genetic algorithm over a
//! population of 20 agents storing food through a 100-day season, bred over
//! 60 generations. See
//! docs/superpowers/specs/2026-09-30-minds-7-hoarding-evolution-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{schema, CheaterFitness, DeadStores, DefendedInPool, HoardConfig, LarderWeight};
pub use presets::presets;
pub use stats::{by_generation, HoardSnapshot, GENERATION_SERIES, SERIES};
pub use world::{
    Agent, AgentView, Cause, Death, Fate, HoardInspection, HoardWorld, Outcome, Record, Season,
    SeasonSummary, EPSILON, EXPOSURE_FLOOR, LOW_L, TAKEOVER_L, TALL, WIDE, WINDOW,
};

pub(crate) use world::{clamped_logit, inverse_logit};
