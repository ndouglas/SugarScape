//! Goal-oriented action planning (Minds 4; Orkin 2004): the cheapest
//! sequence of actions from a state to any goal state, found by Minds 2's A*
//! over the states the search reaches. Deterministic: it draws no random
//! numbers, and among equal-cost plans A*'s tie rule (lowest f, then lowest
//! h, then earliest pushed, with actions pushed in the domain's order)
//! decides.
//!
//! States are interned to indices as A* reaches them. Every goal state has
//! one extra edge, of cost 0, to a sentinel node, and the sentinel is A*'s
//! goal; the sentinel's heuristic is 0 and so is a goal state's, so a
//! consistent domain heuristic stays consistent.

pub mod strips;

use std::cell::RefCell;
use std::collections::HashMap;

use super::astar::{astar, Graph};

/// A planning domain (Minds 4): states, actions with costs, a goal and an
/// admissible heuristic.
pub trait Domain {
    type State: Clone + Eq + std::hash::Hash;
    type Action: Clone;
    /// Applicable actions from `s`, each with its cost (> 0) and the state it
    /// leads to, in a fixed order (the tie rule's push order).
    fn actions(&self, s: &Self::State, out: &mut Vec<(Self::Action, f64, Self::State)>);
    fn is_goal(&self, s: &Self::State) -> bool;
    /// A lower bound on the cost from `s` to any goal state (0 at goal
    /// states). It must also be consistent: it never drops by more than an
    /// action's cost.
    fn heuristic(&self, s: &Self::State) -> f64;
}

/// A found plan: its actions in order and its cost, and how many states A*
/// expanded to find it.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan<A> {
    pub actions: Vec<A>,
    pub cost: f64,
    pub expanded: usize,
}

/// A*'s goal: the node every goal state steps to at cost 0.
const SENTINEL: usize = usize::MAX;

/// An interned state: the state, its heuristic and whether it is a goal.
struct Node<S> {
    state: S,
    h: f64,
    goal: bool,
}

/// A state's applicable actions: each action, its cost and where it leads.
type Successors<D> = Vec<(<D as Domain>::Action, f64, <D as Domain>::State)>;

/// The domain as a graph over interned states. `neighbors` takes `&self`,
/// so the interner sits in `RefCell`s. Its map is only looked up, never
/// iterated.
struct Interned<'d, D: Domain> {
    domain: &'d D,
    index: RefCell<HashMap<D::State, usize>>,
    nodes: RefCell<Vec<Node<D::State>>>,
    scratch: RefCell<Successors<D>>,
}

impl<'d, D: Domain> Interned<'d, D> {
    fn new(domain: &'d D) -> Self {
        Interned {
            domain,
            index: RefCell::new(HashMap::new()),
            nodes: RefCell::new(Vec::new()),
            scratch: RefCell::new(Vec::new()),
        }
    }

    fn intern(&self, s: &D::State) -> usize {
        if let Some(&i) = self.index.borrow().get(s) {
            return i;
        }
        let mut nodes = self.nodes.borrow_mut();
        let i = nodes.len();
        let goal = self.domain.is_goal(s);
        let h = self.domain.heuristic(s);
        debug_assert!(!goal || h == 0.0, "a goal state's heuristic must be 0");
        nodes.push(Node {
            state: s.clone(),
            h,
            goal,
        });
        self.index.borrow_mut().insert(s.clone(), i);
        i
    }
}

impl<D: Domain> Graph for Interned<'_, D> {
    fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>) {
        if n == SENTINEL {
            return;
        }
        let mut scratch = self.scratch.borrow_mut();
        scratch.clear();
        {
            let nodes = self.nodes.borrow();
            let node = &nodes[n];
            if node.goal {
                // A goal state only steps to the sentinel: any action from it
                // costs more than stopping.
                out.push((SENTINEL, 0.0));
                return;
            }
            self.domain.actions(&node.state, &mut scratch);
        }
        for (_, c, t) in scratch.iter() {
            out.push((self.intern(t), *c));
        }
    }

    fn heuristic(&self, n: usize, _goal: usize) -> f64 {
        if n == SENTINEL {
            0.0
        } else {
            self.nodes.borrow()[n].h
        }
    }
}

/// The cheapest plan from `start` to a goal state, or `None` (unreachable,
/// or more than `limit` expansions). A start that is already a goal gives
/// the empty plan with nothing expanded.
pub fn plan<D: Domain>(domain: &D, start: D::State, limit: usize) -> Option<Plan<D::Action>> {
    if domain.is_goal(&start) {
        return Some(Plan {
            actions: Vec::new(),
            cost: 0.0,
            expanded: 0,
        });
    }
    let g = Interned::new(domain);
    let s0 = g.intern(&start);
    let found = astar(&g, s0, SENTINEL, limit)?;
    // Rebuild the actions from the path's edges: A* keeps, for each state,
    // the first cheapest edge from its parent (a later edge replaces it only
    // when strictly cheaper), so regenerating the parent's actions and taking
    // the first cheapest one into the child recovers the edge A* used.
    let nodes = g.nodes.into_inner();
    let states = &found.path[..found.path.len() - 1]; // drop the sentinel
    let mut actions = Vec::with_capacity(states.len().saturating_sub(1));
    let mut out = Vec::new();
    for w in states.windows(2) {
        let (from, to) = (&nodes[w[0]].state, &nodes[w[1]].state);
        out.clear();
        domain.actions(from, &mut out);
        let mut best: Option<(f64, &D::Action)> = None;
        for (a, c, t) in &out {
            if t == to && best.is_none_or(|(b, _)| *c < b) {
                best = Some((*c, a));
            }
        }
        let (_, a) = best.expect("A*'s path follows the domain's actions");
        actions.push(a.clone());
    }
    Some(Plan {
        actions,
        cost: found.cost,
        expanded: found.expanded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line 0 → 1 → … → `len` - 1, one step right at cost 1 (and a
    /// two-step jump at cost 3); the goal is `goal`.
    struct Line {
        len: u32,
        goal: u32,
    }

    impl Domain for Line {
        type State = u32;
        type Action = &'static str;
        fn actions(&self, s: &u32, out: &mut Vec<(&'static str, f64, u32)>) {
            if s + 1 < self.len {
                out.push(("step", 1.0, s + 1));
            }
            if s + 2 < self.len {
                out.push(("jump", 3.0, s + 2));
            }
        }
        fn is_goal(&self, s: &u32) -> bool {
            *s == self.goal
        }
        fn heuristic(&self, s: &u32) -> f64 {
            self.goal.saturating_sub(*s) as f64
        }
    }

    #[test]
    fn a_start_that_is_a_goal_gives_the_empty_plan() {
        let p = plan(&Line { len: 5, goal: 2 }, 2, 0).unwrap();
        assert_eq!(
            p,
            Plan {
                actions: vec![],
                cost: 0.0,
                expanded: 0
            }
        );
    }

    #[test]
    fn the_cheapest_plan_steps_rather_than_jumps() {
        let p = plan(&Line { len: 5, goal: 4 }, 0, usize::MAX).unwrap();
        assert_eq!(p.actions, vec!["step"; 4]);
        assert_eq!(p.cost, 4.0);
    }

    #[test]
    fn an_unreachable_goal_gives_none() {
        // Behind the start on a one-way line.
        assert!(plan(&Line { len: 5, goal: 1 }, 3, usize::MAX).is_none());
        // Past the line's end.
        assert!(plan(&Line { len: 5, goal: 9 }, 0, usize::MAX).is_none());
    }

    #[test]
    fn the_limit_bounds_expansions() {
        let d = Line { len: 8, goal: 5 };
        let p = plan(&d, 0, usize::MAX).unwrap();
        // 0..=4 on the way, then the goal 5 steps to the sentinel.
        assert_eq!(p.expanded, 6);
        assert_eq!(plan(&d, 0, p.expanded), Some(p.clone()));
        assert!(plan(&d, 0, p.expanded - 1).is_none());
    }
}
