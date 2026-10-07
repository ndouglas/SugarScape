//! Shared-worker excavation and food return with private dynamic topology.
//!
//! Workers hold at most one tagged food or spoil token. Excavation opens one
//! eligible cell, exposes hidden food and creates carried spoil without moving
//! or picking up food. Spoil goes directly to the separate waste outlet; disposal
//! resumes the paused food intent without food arrival draws or publication.
//! Workers explore private open frontiers before remembered diggable faces.
//! Old diggable walls remain valid private beliefs until locally observed again.
//!
//! Hidden, exposed, physically accessible and delivered food are distinct.
//! Researcher access ignores occupancy and private knowledge and uses one nest
//! BFS at construction and each successful Dig. Views clone its cached records;
//! they never observe, learn, run BFS, expire advice or consume random draws.
//! Physical opportunities and worker/researcher computation are reported separately.
//!
//! Dimensions are 3–125; workers 1–256; food 0–256. Runs request 1–7,200
//! ticks and at most 1,000,000 opportunities, continuing through exhaustion.
//! Sampling intervals are positive even when disabled. Initial/final frames
//! occur once; disabled recording yields zero frames/bytes. A bounded counting
//! writer caps summed compact Snapshot JSON at 64 MiB, including all repeated
//! fields and excluding Episode/setup/options wrappers and frame delimiters.
//! Errors yield no successful partial episode. Maps/scratch are grid-bounded,
//! with at most 4,000,000 worker classifications and 15,625 spoil records.
//!
//! Replay requires the same normalized setup, seed, options, implementation and
//! supported platform. No restoration or cross-platform identity is promised.
//! Missing pickup, access, delivery or disposal outcomes at cutoff are censored.
//! Parameters are explicit engineering inputs, not biological calibration.
//! Dedicated roles, relay transport and scientific comparisons remain deferred;
//! existing passage and Burrow APIs/behavior remain separate baselines.
//!
//! ```
//! use sugarscape_core::foraging::construction::{
//!     run, Parameters, Pos, Resource, RunOptions, Setup, World,
//! };
//! let pos = |x, y| Pos { x, y };
//! let setup = Setup {
//!     width: 5, height: 3,
//!     open: vec![pos(0,0), pos(1,0), pos(2,0), pos(0,1)],
//!     diggable: vec![pos(3,0)], nest: vec![pos(0,0), pos(1,0)],
//!     waste: pos(0,1), workers: vec![pos(0,0)],
//!     food: vec![Resource { id: u64::MAX, pos: pos(3,0) }],
//!     parameters: Parameters {
//!         p_search: 1.0, p_return: 0.0, lambda_fidelity: 0.0,
//!         lambda_publish: 0.0, lambda_waypoint: 0.0,
//!     },
//! };
//! let options = RunOptions { ticks: 40, sample_every: 7, snapshots: true };
//! let episode = run(setup.clone(), 12, options).unwrap();
//! let mut world = World::new(setup, 12).unwrap();
//! for _ in 0..40 { world.step().unwrap(); }
//! assert_eq!(episode.summary, world.summary().unwrap());
//! assert_eq!(episode.summary.work.opportunities, 40);
//! let food = episode.summary.food;
//! assert_eq!(food.initial, food.hidden + food.available + food.carried + food.delivered);
//! let spoil = episode.summary.spoil;
//! assert_eq!(spoil.excavated, spoil.carried + spoil.disposed);
//! assert_eq!(episode.snapshots.first().unwrap().summary.completed_ticks, 0);
//! assert_eq!(episode.snapshots.last().unwrap().summary.completed_ticks, 40);
//! assert_eq!(world.knowledge(0).unwrap().agent, 0);
//! assert_eq!(run(episode.setup.clone(), episode.seed, episode.options.clone()).unwrap(), episode);
//! ```
mod draws;
mod knowledge;
mod metrics;
mod navigation;
mod observation;
mod setup;
mod terrain;
pub use crate::foraging::passage::{Parameters, Pos, Resource};
pub use knowledge::{CellKnowledge, KnownCell};
pub use setup::Setup;
pub use terrain::TerrainInventory;
type Checked<T> = Result<T, Vec<crate::config::FieldError>>;
#[cfg(test)]
mod tests;

mod access;
mod food;
mod server;
mod spoil;
mod state;
pub use access::{EventContext, EventMilestone, Milestones};
pub use food::{FoodInventory, FoodState, FoodView};
pub use spoil::{SpoilInventory, SpoilState, SpoilView};
pub use state::{Cargo, FoodPhase, WorkCounts};

mod actions;
mod controller;
mod decision;
mod world;
pub use state::Mode;
pub use world::World;

mod runner;
mod view;
pub use access::{AccessSummary, FoodAccessRecord};
pub use metrics::{AccessCompute, ComputeCounts};
pub use runner::{run, Episode, RunOptions};
pub use view::{AgentView, FindView, KnowledgeView, Snapshot, Summary, WaypointView};
