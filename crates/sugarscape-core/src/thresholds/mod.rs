//! Threshold Models (milestone 25): Granovetter, "Threshold Models of
//! Collective Behavior" (AJS 1978), with the extensions he sketches (friends,
//! crowds sampled from a city, clusters with movement, ceilings) and Watts's
//! "A Simple Model of Global Cascades on Random Networks" (PNAS 2002) as
//! presets and switches. See docs/superpowers/specs/2026-09-27-thresholds-design.md.

mod config;
mod crowd;
mod presets;
mod stats;
mod theory;
mod view;
mod world;

pub use config::{
    schema, Ceilings, Clusters, Crowd, Distribution, Friends, Network, Population, Rounding,
    ThresholdsConfig, Trigger, Update, Zero, MAX_ACTORS, POWER_LAW_CAP,
};
pub use crowd::{city, degrees, draw, inverse_normal, power_law, Th, SCALE};
pub use presets::presets;
pub use stats::{ThresholdsSnapshot, RECENT, SERIES};
pub use theory::{
    cascade_ratio, granovetter, normal_cdf, poisson, poisson_ratio, poisson_window,
    power_law_ratio, vulnerable,
};
pub use view::{grid, row, GRID_X, MID_X, SHOWN, TALL, TIME_W};
pub use world::{ActorView, ThresholdsCell, ThresholdsInspection, ThresholdsMode, ThresholdsWorld};
