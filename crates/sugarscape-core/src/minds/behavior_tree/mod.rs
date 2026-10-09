//! Bounded behavior trees independent of the simulation world.
pub mod runtime;

#[cfg(test)]
mod reference_tests;
#[cfg(test)]
mod runtime_tests;

#[cfg(test)]
mod policy_tests;

pub(crate) mod forage;
pub(crate) mod fsm;
pub(crate) mod policy;
pub mod state;
pub(crate) mod telemetry;
pub use telemetry::WorkCounters;
/// The closed task graph, for strict controller-specific saved-state validation.
pub fn tree_for_controller(controller: state::Controller) -> Option<runtime::Tree> {
    matches!(
        controller,
        state::Controller::GuardedTree | state::Controller::UnguardedTree
    )
    .then(routine_tree)
}
pub(crate) fn routine_tree() -> runtime::Tree {
    use runtime::{Node, Tree};
    Tree::new(vec![
        Node::ReactiveSequence(vec![1, 2, 3]),
        Node::Condition(0),
        Node::Condition(1),
        Node::MemorySequence(vec![4, 5]),
        Node::Logical(0),
        Node::Physical(0),
    ])
    .expect("closed valid graph")
}

pub mod lab;
pub mod records;
pub mod runner;
pub use records::{EpisodeFailure, EpisodeRecord, RunOptions};
pub const EPISODE_SCHEMA: &str = "minds-behavior-tree-episode-v1";
#[cfg(test)]
mod lab_tests;
#[cfg(test)]
mod runner_tests;
