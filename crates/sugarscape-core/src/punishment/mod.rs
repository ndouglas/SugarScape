//! Altruistic Punishment (milestone 27): Boyd, Gintis, Bowles and Richerson,
//! "The evolution of altruistic punishment" (PNAS 100: 3531–3535, 2003),
//! with Cooney's PDE model (arXiv:2405.18419, 2024) as the critique. See
//! docs/superpowers/specs/2026-09-28-punishment-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, Refill, Start,
    Structure, Traits, Victory, MOST_AGENTS,
};
pub use presets::presets;
pub use stats::{PunishmentSnapshot, SERIES};
pub use view::{Mosaic, STRIP, STRIP_GAP};
pub use world::{
    payoff_range, AgentView, GroupView, Kind, PunishmentCell, PunishmentInspection, PunishmentMode,
    PunishmentWorld,
};
