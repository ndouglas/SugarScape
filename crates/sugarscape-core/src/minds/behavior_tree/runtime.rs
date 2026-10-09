//! A bounded executor with one physical attempt per actor turn.
//!
//! Traversal state describes a logical activation across turns. Terminal action
//! results remain cached until that activation ends; reactive ancestors reenter
//! earlier guards, while memory sequences resume their first unfinished child.
//! Settlement and ancestor unwind never enter a new node. Graph depth is at most
//! eight, so after a leaf there are at most seven ancestor returns, even when its
//! visit exhausted the budget. No simulation world or random source is needed.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Success,
    Failure,
    Running,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Node {
    Condition(u8),
    Logical(u8),
    Physical(u8),
    ReactiveSequence(Vec<u8>),
    ReactiveFallback(Vec<u8>),
    MemorySequence(Vec<u8>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tree {
    pub nodes: Vec<Node>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeState {
    pub cursors: BTreeMap<u8, u8>,
    pub statuses: BTreeMap<u8, Status>,
    pub running_leaves: BTreeSet<u8>,
    pub deferred: Option<u8>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Physical<R> {
    pub status: Status,
    pub receipt: R,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Tick<R> {
    pub status: Status,
    pub receipt: Option<R>,
    pub visits: u16,
    pub exhausted: bool,
    pub deferred_physical: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}
pub trait Host {
    type Receipt: Clone;
    fn supports(&self, node: &Node) -> bool;
    fn condition(&self, id: u8) -> bool;
    fn logical(&mut self, id: u8) -> Status;
    fn physical(&mut self, id: u8) -> Physical<Self::Receipt>;
    fn settle(&mut self, receipt: &Self::Receipt);
    fn complete(&self) -> bool;
    fn halt(&mut self, id: u8);
}
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for RuntimeError {}
fn invalid(message: impl Into<String>) -> RuntimeError {
    RuntimeError {
        message: message.into(),
    }
}
impl Node {
    fn children(&self) -> Option<&[u8]> {
        match self {
            Self::ReactiveSequence(children)
            | Self::ReactiveFallback(children)
            | Self::MemorySequence(children) => Some(children),
            _ => None,
        }
    }
    fn action_id(&self) -> Option<u8> {
        match self {
            Self::Logical(id) | Self::Physical(id) => Some(*id),
            _ => None,
        }
    }
}
impl Tree {
    pub fn new(nodes: Vec<Node>) -> Result<Self, RuntimeError> {
        let tree = Self { nodes };
        tree.validate()?;
        Ok(tree)
    }
    pub fn validate(&self) -> Result<(), RuntimeError> {
        if self.nodes.is_empty() || self.nodes.len() > 32 {
            return Err(invalid("tree must contain 1..=32 nodes"));
        }
        let mut seen = BTreeSet::new();
        self.validate_branch(0, 1, &mut seen)?;
        if seen.len() != self.nodes.len() {
            return Err(invalid("tree contains unreachable nodes"));
        }
        Ok(())
    }
    fn validate_branch(
        &self,
        id: u8,
        depth: u8,
        seen: &mut BTreeSet<u8>,
    ) -> Result<(), RuntimeError> {
        let node = self
            .nodes
            .get(usize::from(id))
            .ok_or_else(|| invalid(format!("child node {id} is out of range")))?;
        if depth > 8 {
            return Err(invalid(format!("node {id} exceeds root-to-leaf depth 8")));
        }
        if !seen.insert(id) {
            return Err(invalid(format!(
                "node {id} has a cycle or shared child ownership"
            )));
        }
        if let Some(children) = node.children() {
            for &child in children {
                self.validate_branch(child, depth + 1, seen)?;
            }
        }
        Ok(())
    }
}
impl TreeState {
    /// Validate structural saved-state references, without demanding diagnostic
    /// history. In particular, a valid active-ID set alone can be halted.
    pub fn validate_for_tree(&self, tree: &Tree) -> Result<(), RuntimeError> {
        tree.validate()?;
        let node_at = |id: u8| {
            tree.nodes
                .get(usize::from(id))
                .ok_or_else(|| invalid(format!("saved node {id} is out of range")))
        };
        for (&id, &cursor) in &self.cursors {
            let children = node_at(id)?
                .children()
                .ok_or_else(|| invalid(format!("saved cursor at non-composite node {id}")))?;
            if usize::from(cursor) >= children.len() {
                return Err(invalid(format!(
                    "saved cursor {cursor} at node {id} is out of range"
                )));
            }
        }
        for (&id, &status) in &self.statuses {
            if matches!(node_at(id)?, Node::Condition(_)) && status == Status::Running {
                return Err(invalid(format!("condition node {id} cannot be running")));
            }
        }
        for &id in &self.running_leaves {
            if node_at(id)?.action_id().is_none() {
                return Err(invalid(format!("active node {id} is not an action leaf")));
            }
            if self
                .statuses
                .get(&id)
                .is_some_and(|status| *status != Status::Running)
            {
                return Err(invalid(format!(
                    "active node {id} has a terminal saved status"
                )));
            }
        }
        if let Some(id) = self.deferred {
            node_at(id)?;
        }
        Ok(())
    }
}
fn validate_host<H: Host>(tree: &Tree, host: &H) -> Result<(), RuntimeError> {
    for (id, node) in tree.nodes.iter().enumerate() {
        if node.children().is_none() && !host.supports(node) {
            return Err(invalid(format!(
                "host does not support leaf node {id}: {node:?}"
            )));
        }
    }
    Ok(())
}
fn reset<H: Host>(tree: &Tree, state: &mut TreeState, host: &mut H) {
    for &id in &state.running_leaves {
        // Public entry points validated every active graph ID before effects.
        host.halt(
            tree.nodes[usize::from(id)]
                .action_id()
                .expect("validated action leaf"),
        );
    }
    *state = TreeState::default();
}
/// Halt each active graph leaf once, passing its registered payload ID to the
/// host. Validation finishes before any halt callback or mutation of state.
pub fn halt<H: Host>(tree: &Tree, state: &mut TreeState, host: &mut H) -> Result<(), RuntimeError> {
    state.validate_for_tree(tree)?;
    validate_host(tree, host)?;
    reset(tree, state, host);
    Ok(())
}
/// Execute a turn with a fresh physical token and a bounded node-entry budget.
pub fn tick<H: Host>(
    tree: &Tree,
    state: &mut TreeState,
    host: &mut H,
    visits: u16,
) -> Result<Tick<H::Receipt>, RuntimeError> {
    if !(1..=64).contains(&visits) {
        return Err(invalid("visit budget must be in 1..=64"));
    }
    state.validate_for_tree(tree)?;
    validate_host(tree, host)?;
    let mut evaluator = Evaluator {
        tree,
        state,
        host,
        limit: visits,
        physical_used: false,
        completed: false,
        result: Tick {
            status: Status::Running,
            receipt: None,
            visits: 0,
            exhausted: false,
            deferred_physical: false,
        },
    };
    evaluator.state.deferred = None;
    evaluator.result.status = if evaluator.host.complete() {
        Status::Success
    } else {
        evaluator.enter(0, false)
    };
    if evaluator.result.status != Status::Running {
        reset(tree, evaluator.state, evaluator.host);
    }
    Ok(evaluator.result)
}
struct Evaluator<'a, H: Host> {
    tree: &'a Tree,
    state: &'a mut TreeState,
    host: &'a mut H,
    limit: u16,
    physical_used: bool,
    completed: bool,
    result: Tick<H::Receipt>,
}
impl<H: Host> Evaluator<'_, H> {
    fn enter(&mut self, id: u8, rechecking: bool) -> Status {
        let node = self.tree.nodes[usize::from(id)].clone();
        // Reconstructing a reactive path must not reissue a settled physical
        // action or logical selection. A Running action, however, can step.
        if node.action_id().is_some() {
            if let Some(&status) = self.state.statuses.get(&id) {
                if status != Status::Running {
                    return status;
                }
            }
        }
        if self.result.visits == self.limit {
            self.result.exhausted = true;
            self.state.deferred = Some(id);
            return Status::Running;
        }
        self.result.visits += 1;
        let status = match node {
            Node::Condition(payload) => {
                if self.host.condition(payload) {
                    Status::Success
                } else {
                    Status::Failure
                }
            }
            Node::Logical(payload) => {
                let status = self.host.logical(payload);
                self.leaf_status(id, status);
                status
            }
            Node::Physical(payload) => {
                if self.physical_used {
                    self.result.deferred_physical = true;
                    self.state.deferred = Some(id);
                    return Status::Running;
                }
                self.physical_used = true;
                let attempted = self.host.physical(payload);
                self.host.settle(&attempted.receipt);
                self.result.receipt = Some(attempted.receipt);
                self.leaf_status(id, attempted.status);
                // Completion wins even on Failure/Running and on the last
                // budgeted entry. Unwind cannot enter another child.
                self.completed = self.host.complete();
                if self.completed {
                    return Status::Success;
                }
                attempted.status
            }
            Node::ReactiveSequence(children) => {
                self.composite(id, &children, false, true, rechecking)
            }
            Node::ReactiveFallback(children) => {
                self.composite(id, &children, true, true, rechecking)
            }
            Node::MemorySequence(children) => {
                self.composite(id, &children, false, false, rechecking)
            }
        };
        self.state.statuses.insert(id, status);
        status
    }
    fn leaf_status(&mut self, id: u8, status: Status) {
        self.state.statuses.insert(id, status);
        if status == Status::Running {
            self.state.running_leaves.insert(id);
        } else {
            self.state.running_leaves.remove(&id);
        }
    }
    fn composite(
        &mut self,
        id: u8,
        children: &[u8],
        fallback: bool,
        reactive: bool,
        rechecking: bool,
    ) -> Status {
        let resume = usize::from(self.state.cursors.get(&id).copied().unwrap_or(0));
        let start = if reactive || rechecking { 0 } else { resume };
        let advance = if fallback {
            Status::Failure
        } else {
            Status::Success
        };
        for (index, &child) in children.iter().enumerate().skip(start) {
            // Store the first unfinished child *before* any unavailable entry.
            // Guard reconstruction must not move an existing cursor backwards.
            self.state.cursors.insert(id, resume.max(index) as u8);
            let status = self.enter(child, rechecking || index < resume);
            if self.completed {
                return Status::Success;
            }
            if status == Status::Running {
                if index < resume && self.state.deferred.is_none() {
                    // A genuinely running earlier branch displaced the former
                    // branch; budget deferral while checking guards did not.
                    for &later in &children[index + 1..] {
                        self.clear_subtree(later, true);
                    }
                    self.state.cursors.insert(id, index as u8);
                }
                return Status::Running;
            }
            if status != advance {
                self.clear_subtree(id, false);
                return status;
            }
            if index >= resume && index + 1 < children.len() {
                self.state.cursors.insert(id, (index + 1) as u8);
            }
        }
        self.clear_subtree(id, false);
        if fallback {
            Status::Failure
        } else {
            Status::Success
        }
    }
    fn clear_subtree(&mut self, id: u8, forget_results: bool) {
        if self.state.running_leaves.remove(&id) {
            self.host.halt(
                self.tree.nodes[usize::from(id)]
                    .action_id()
                    .expect("validated action leaf"),
            );
            self.state.statuses.remove(&id);
        }
        self.state.cursors.remove(&id);
        if forget_results {
            self.state.statuses.remove(&id);
        }
        if let Some(children) = self.tree.nodes[usize::from(id)].children() {
            for &child in children {
                self.clear_subtree(child, forget_results);
            }
        }
    }
}
