//! Tag-based cooperation (milestone 12): Riolo, Cohen & Axelrod, "Evolution
//! of cooperation without reciprocity", Nature 414 (2001), with Edmonds &
//! Hales' (JASSS 2003) and Roberts & Sherratt's (Nature 2002) departures as
//! named switches. See docs/superpowers/specs/2026-09-25-tags-design.md.

mod config;
mod presets;
mod stats;
mod world;

pub use config::{schema, DonationTest, InitialTolerance, Selection, TagsConfig, TieRule};
pub use presets::presets;
pub use stats::{cluster, Cluster, TagsSnapshot, Takeovers, CLUSTER_RADIUS, SERIES};
pub use world::{
    bin, Bin, Tagger, TaggerView, TagsCell, TagsInspection, TagsMode, TagsWorld, ToleranceView,
    BINS, HISTORY,
};
