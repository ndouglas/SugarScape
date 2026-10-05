//! Fixed-world central-place foraging after Hecker and Moses (2015).
//! Angular eight-neighbor geometry follows the historical iAnt reconstruction;
//! supplied engineering fixtures do not establish scientific efficacy.
//!
//! Ticks process agents in ascending ID order, biasing competitive access.
//! Event times start at processing tick zero; snapshots count completed ticks.
//! Informed age counts turns; waypoint age counts ticks. Controllers only know
//! local successful observations and nest advice, which can remain stale.
//! Resources are conserved tokens; assigned cargo at the horizon is censored.
//! Replay requires identical setup, seed, version and supported platform.
//! Runs execute the complete horizon even after every resource is delivered.
//!
//! ```
//! use sugarscape_core::foraging::{CpfaParameters, fixed::{run, Pos, Resource, Setup, RunOptions}};
//! let setup = Setup {
//!     width: 5, height: 5, nest: Pos { x: 2, y: 2 }, agents: 1,
//!     resources: vec![Resource { id: u64::MAX, pos: Pos { x: 3, y: 2 } }],
//!     parameters: CpfaParameters {
//!         p_search: 1.0, p_return: 0.0, omega: 0.0,
//!         lambda_informed: 0.0, lambda_fidelity: 0.0,
//!         lambda_publish: 0.0, lambda_waypoint: 0.0,
//!     },
//! };
//! let episode = run(setup, 12, RunOptions { ticks: 20, sample_every: 7, snapshots: true }).unwrap();
//! assert_eq!(episode.summary.completed_ticks, 20);
//! assert_eq!(episode.summary.work.opportunities, 20);
//! assert_eq!(episode.snapshots.first().unwrap().completed_ticks, 0);
//! assert_eq!(episode.snapshots.last().unwrap().completed_ticks, 20);
//! ```

mod draws;
mod movement;
mod setup;

pub use setup::{Pos, Resource, Setup};

#[cfg(test)]
mod tests;

mod ledger;
mod state;

pub use ledger::{Inventory, ResourceState, ResourceView};
pub use state::{Phase, WorkCounts};

mod server;
pub use server::WaypointView;

mod controller;
mod world;
pub use world::World;

pub use state::{AgentView, FindView, Snapshot};

mod runner;
pub use runner::{run, Episode, RunOptions, Summary};
