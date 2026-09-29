//! Balinese Water Temples (milestone 29): Lansing and Kremer, "Emergent
//! Properties of Balinese Water Temples" (American Anthropologist 95: 97–114,
//! 1993), and Janssen, "Coordination in Irrigation Systems: An Analysis of the
//! Lansing–Kremer Model of Bali" (Agricultural Systems 93: 170–190, 2007), on
//! Janssen's data for the Oos and Petanu. See
//! docs/superpowers/specs/2026-09-28-bali-water-temples-design.md.

mod config;
mod data;
mod engine;
mod presets;
mod search;
mod stats;
mod two_node;
mod view;
mod world;

pub use config::{
    schema, BaliConfig, DamColumns, Decision, Perturb, PestForm, Plans, Rain, Routing, Watershed,
    LEVELS,
};
pub use data::{watershed, DamData, SubakData};
pub use engine::{area_mean, steady_year, Network, Params};
pub use presets::presets;
pub use search::{groups, search};
pub use stats::{adjusted_rand, BaliSnapshot, SERIES};
pub use two_node::{best, simulate, NodePlan, Nodes};
pub use view::{MAP_H, TALL, WIDE};
pub use world::{BaliCell, BaliInspection, BaliMode, BaliWorld, DamView, SubakView};
