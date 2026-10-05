//! El Farol and the Minority Game (milestone 23): Arthur, "Inductive
//! Reasoning and Bounded Rationality" (AER 1994), and Challet and Zhang,
//! "Emergence of Cooperation and Organization in an Evolutionary Game"
//! (Physica A 1997), with Savit, Manuca and Riolo's (1999) memory transition
//! and Challet, Marsili and Ottino's (2004) critique as presets and switches.
//! See docs/superpowers/specs/2026-09-27-el-farol-design.md.

mod config;
mod predictors;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, AtCapacity, Behavior, Evolution, FarolConfig, Game, Information, MixedMemory, Payoff,
    Rounding, Scoring, BIT_BUDGET, LIBRARY, MAX_MEMORY, MAX_STRATEGIES,
};
pub use predictors::{library, Family, Predictor, LOOKBACK};
pub use presets::presets;
pub use stats::{random_fluctuation, FarolSnapshot, SERIES, WINDOW};
pub use view::{grid, row, GRID_X, HIST_X, SHOWN, TALL, TIME_W};
pub use world::{
    payoff, Agent, FarolAgent, FarolCell, FarolDecision, FarolDecisionAgent, FarolInspection,
    FarolMode, FarolWorld, StrategyView,
};
