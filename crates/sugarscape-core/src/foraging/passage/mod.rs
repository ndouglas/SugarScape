//! Fixed four-neighbor passage foraging with immediate observations and private topology.
//!
//! Workers learn only their own current/cardinal cell classifications. Nest advice
//! supplies coordinates without sharing maps or remembered food/occupancy truth.
//! Researcher views consume no draws and do not expire advice. Physical work and
//! computation have separate checked counters; queue peaks aggregate by maximum.
//!
//! Runs request 1–7,200 ticks and at most 1,000,000 worker opportunities. Sampling
//! intervals must be positive even when recording is disabled. Initial and final
//! frames occur once. Compact snapshot JSON, including duplicated geometry, is
//! limited to 64 MiB; episode setup, enclosing fields and delimiters are excluded.
//! Failures yield no successful partial episode. Each private map is grid-bounded.
//!
//! The replay setup is normalized without reordering workers or changing original
//! resource coordinates. Replay requires the same implementation and supported
//! platform; there is no saved-state restoration or cross-platform identity claim.
//! These are supplied engineering adaptations, not evolved or calibrated defaults.
//!
//! ```
//! use sugarscape_core::foraging::passage::{run, Parameters, Pos, Resource, RunOptions, Setup, World};
//! let pos = |x, y| Pos { x, y };
//! let setup = Setup {
//!     width: 5, height: 5,
//!     open: vec![pos(0,0), pos(1,0), pos(2,0), pos(3,0), pos(3,1)],
//!     nest: vec![pos(0,0), pos(1,0)], workers: vec![pos(0,0)],
//!     resources: vec![Resource { id: u64::MAX, pos: pos(3,0) }],
//!     parameters: Parameters {
//!         p_search: 1.0, p_return: 0.0, lambda_fidelity: 0.0,
//!         lambda_publish: 0.0, lambda_waypoint: 0.0,
//!     },
//! };
//! let episode = run(setup.clone(), 12,
//!     RunOptions { ticks: 40, sample_every: 7, snapshots: true }).unwrap();
//! let mut world = World::new(setup, 12).unwrap();
//! for _ in 0..40 { world.step().unwrap(); }
//! assert_eq!(episode.summary, world.summary().unwrap());
//! assert_eq!(episode.summary.work.opportunities, 40);
//! let inventory = episode.summary.inventory;
//! assert_eq!(inventory.initial, inventory.available + inventory.carried + inventory.delivered);
//! assert_eq!(episode.snapshots.first().unwrap().summary.completed_ticks, 0);
//! assert_eq!(episode.snapshots.last().unwrap().summary.completed_ticks, 40);
//! let knowledge = world.knowledge(0).unwrap();
//! assert_eq!(knowledge.agent, 0);
//! ```
mod draws;
mod knowledge;
mod metrics;
mod navigation;
mod observation;
mod setup;
pub use knowledge::{CellKnowledge, KnownCell};
pub use setup::{Parameters, Pos, Resource, Setup};
type Checked<T> = Result<T, Vec<crate::config::FieldError>>;
mod ledger;
mod server;
mod state;
#[cfg(test)]
mod tests;
pub use ledger::{Inventory, ResourceState, ResourceView};
pub use server::ServerRecordView;
pub use state::{Phase, WorkCounts};
mod actions;
mod controller;
mod decision;
mod world;
pub use world::World;

mod runner;
mod view;
pub use metrics::ComputeCounts;
pub use runner::{run, Episode, RunOptions};
pub use view::{AgentView, FindView, KnowledgeView, Snapshot, Summary, WaypointView};

/// Read-only cloned RNG continuation for the construction no-dig unit regression.
#[cfg(test)]
pub(crate) fn construction_rng_probe(world: &World) -> [u64; 4] {
    use rand::RngCore;
    let mut rng = world.rng.clone();
    std::array::from_fn(|_| rng.next_u64())
}
