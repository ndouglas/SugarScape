//! Fixed-value memoryless Q-learning auctions (Banchio and Skrzypacz, 2022).
//! Optimistic initialization and tie conventions are explicit reconstruction choices.
pub mod analysis;
mod config;
pub mod learner;
pub mod mechanism;
mod presets;
mod stats;
mod view;
mod world;
pub use config::*;
pub use presets::presets;
pub use stats::{series_names, AuctionsSnapshot, SERIES};
pub use world::{AuctionsWorld, Outcome};
