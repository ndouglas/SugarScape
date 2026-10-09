//! Bounded behavior trees independent of the simulation world.
pub mod runtime;

#[cfg(test)]
mod reference_tests;
#[cfg(test)]
mod runtime_tests;

#[cfg(test)]
mod policy_tests;

// Task 3 supplies the production lab callers for this internal adapter graph.
#[allow(dead_code)]
pub(crate) mod forage;
#[allow(dead_code)]
pub(crate) mod fsm;
#[allow(dead_code)]
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
