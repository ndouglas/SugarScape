use super::runtime::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

struct Script {
    actions: BTreeMap<u8, VecDeque<(Status, u32)>>,
    attempts: usize,
    settled: Vec<u32>,
    halts: Vec<u8>,
    gathered: u32,
    quota: u32,
}
impl Script {
    fn two_actions(a: u32, b: u32, quota: u32) -> Self {
        Self {
            actions: BTreeMap::from([
                (0, VecDeque::from([(Status::Success, a)])),
                (1, VecDeque::from([(Status::Success, b)])),
            ]),
            attempts: 0,
            settled: vec![],
            halts: vec![],
            gathered: 0,
            quota,
        }
    }
}
impl Host for Script {
    type Receipt = u32;
    fn supports(&self, n: &Node) -> bool {
        match n {
            Node::Condition(id) | Node::Logical(id) => *id == 0,
            Node::Physical(id) => self.actions.contains_key(id),
            _ => true,
        }
    }
    fn condition(&self, _: u8) -> bool {
        true
    }
    fn logical(&mut self, _: u8) -> Status {
        Status::Success
    }
    fn physical(&mut self, id: u8) -> Physical<u32> {
        self.attempts += 1;
        let (status, receipt) = self.actions.get_mut(&id).unwrap().pop_front().unwrap();
        Physical { status, receipt }
    }
    fn settle(&mut self, r: &u32) {
        self.settled.push(*r);
        self.gathered += r;
    }
    fn complete(&self) -> bool {
        self.gathered >= self.quota
    }
    fn halt(&mut self, id: u8) {
        self.halts.push(id);
    }
}
fn pair_tree() -> Tree {
    Tree::new(vec![
        Node::MemorySequence(vec![1, 2]),
        Node::Physical(0),
        Node::Physical(1),
    ])
    .unwrap()
}
#[test]
fn behavior_tree_one_turn_cannot_execute_two_physical_leaves() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(12, 8, 100);
    let r = tick(&pair_tree(), &mut s, &mut h, 64).unwrap();
    assert_eq!(
        (h.attempts, r.receipt, r.status),
        (1, Some(12), Status::Running)
    );
}
#[test]
fn behavior_tree_last_visit_settles_harvest_before_deferral() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(12, 8, 100);
    let r = tick(&pair_tree(), &mut s, &mut h, 2).unwrap();
    assert_eq!(
        (h.settled, r.receipt, r.exhausted),
        (vec![12], Some(12), true)
    );
}
#[test]
fn behavior_tree_quota_completion_overrides_unentered_second_action() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(20, 8, 20);
    let r = tick(&pair_tree(), &mut s, &mut h, 2).unwrap();
    assert_eq!((h.attempts, h.gathered, r.status), (1, 20, Status::Success));
    assert_eq!(s, TreeState::default());
}
#[test]
fn behavior_tree_budget_one_preserves_first_unticked_child() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(12, 8, 100);
    let r = tick(&pair_tree(), &mut s, &mut h, 1).unwrap();
    assert_eq!(
        (r.visits, r.exhausted, r.receipt, h.attempts),
        (1, true, None, 0)
    );
    assert_eq!((s.cursors[&0], s.deferred), (0, Some(1)));
}
#[test]
fn behavior_tree_memory_continuation_does_not_repeat_settlement() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(12, 8, 100);
    tick(&pair_tree(), &mut s, &mut h, 2).unwrap();
    assert_eq!((s.cursors[&0], s.deferred), (1, Some(2)));
    let saved = serde_json::to_string(&s).unwrap();
    let mut restored: TreeState = serde_json::from_str(&saved).unwrap();
    let r = tick(&pair_tree(), &mut restored, &mut h, 2).unwrap();
    assert_eq!(
        (h.settled, h.attempts, r.status, r.receipt),
        (vec![12, 8], 2, Status::Success, Some(8))
    );
    assert_eq!(restored, TreeState::default());
}
#[test]
fn behavior_tree_consumed_failure_defers_fallback_without_retrying_failure() {
    let tree = Tree::new(vec![
        Node::ReactiveFallback(vec![1, 2]),
        Node::Physical(0),
        Node::Physical(1),
    ])
    .unwrap();
    let mut s = TreeState::default();
    let mut h = Script::two_actions(12, 8, 100);
    h.actions.get_mut(&0).unwrap()[0].0 = Status::Failure;
    let first = tick(&tree, &mut s, &mut h, 64).unwrap();
    assert_eq!(
        (
            first.status,
            first.exhausted,
            first.deferred_physical,
            h.attempts
        ),
        (Status::Running, false, true, 1)
    );
    assert_eq!(
        (s.cursors[&0], s.statuses[&1], s.deferred),
        (1, Status::Failure, Some(2))
    );
    let second = tick(&tree, &mut s, &mut h, 64).unwrap();
    assert_eq!((second.status, h.settled), (Status::Success, vec![12, 8]));
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Event {
    Condition(u8),
    Logical(u8),
    Physical(u8),
    Settle(u32),
    Halt(u8),
}
pub(super) struct Environment {
    pub queue: VecDeque<(Status, u32)>,
    pub guard: bool,
    pub logical_status: Status,
    pub events: RefCell<Vec<Event>>,
    pub gathered: u32,
    pub quota: u32,
}
impl Environment {
    pub fn new(script: impl IntoIterator<Item = (Status, u32)>) -> Self {
        Self {
            queue: script.into_iter().collect(),
            guard: true,
            logical_status: Status::Success,
            events: RefCell::new(vec![]),
            gathered: 0,
            quota: u32::MAX,
        }
    }
}
impl Host for Environment {
    type Receipt = u32;
    fn supports(&self, node: &Node) -> bool {
        match node {
            Node::Condition(id) | Node::Logical(id) | Node::Physical(id) => *id < 10,
            _ => true,
        }
    }
    fn condition(&self, id: u8) -> bool {
        self.events.borrow_mut().push(Event::Condition(id));
        self.guard
    }
    fn logical(&mut self, id: u8) -> Status {
        self.events.borrow_mut().push(Event::Logical(id));
        self.logical_status
    }
    fn physical(&mut self, id: u8) -> Physical<u32> {
        self.events.borrow_mut().push(Event::Physical(id));
        let (status, receipt) = self
            .queue
            .pop_front()
            .expect("script has one return per possible physical turn");
        Physical { status, receipt }
    }
    fn settle(&mut self, r: &u32) {
        self.events.borrow_mut().push(Event::Settle(*r));
        self.gathered += r;
    }
    fn complete(&self) -> bool {
        self.gathered >= self.quota
    }
    fn halt(&mut self, id: u8) {
        self.events.borrow_mut().push(Event::Halt(id));
    }
}
#[test]
fn behavior_tree_running_physical_steps_next_turn_and_settles_each_step() {
    let tree = Tree::new(vec![Node::Physical(7)]).unwrap();
    let mut h = Environment::new([(Status::Running, 2), (Status::Success, 3)]);
    let mut s = TreeState::default();
    assert_eq!(
        tick(&tree, &mut s, &mut h, 1).unwrap().status,
        Status::Running
    );
    assert_eq!(s.running_leaves, BTreeSet::from([0]));
    assert_eq!(
        tick(&tree, &mut s, &mut h, 1).unwrap().status,
        Status::Success
    );
    assert_eq!(
        *h.events.borrow(),
        vec![
            Event::Physical(7),
            Event::Settle(2),
            Event::Physical(7),
            Event::Settle(3)
        ]
    );
}
#[test]
fn behavior_tree_quota_halts_running_leaf_after_last_visit_settlement() {
    let tree = Tree::new(vec![
        Node::MemorySequence(vec![1, 2]),
        Node::Physical(7),
        Node::Physical(8),
    ])
    .unwrap();
    let mut h = Environment::new([(Status::Running, 20)]);
    h.quota = 20;
    let mut s = TreeState::default();
    let r = tick(&tree, &mut s, &mut h, 2).unwrap();
    assert_eq!(
        (r.status, r.receipt, r.visits),
        (Status::Success, Some(20), 2)
    );
    assert_eq!(
        *h.events.borrow(),
        vec![Event::Physical(7), Event::Settle(20), Event::Halt(7)]
    );
    assert_eq!(s, TreeState::default());
}
#[test]
fn behavior_tree_failed_physical_receipt_can_complete_quota() {
    let tree = Tree::new(vec![Node::Physical(7)]).unwrap();
    let mut h = Environment::new([(Status::Failure, 20)]);
    h.quota = 20;
    let r = tick(&tree, &mut TreeState::default(), &mut h, 1).unwrap();
    assert_eq!(
        (r.status, r.receipt, h.gathered),
        (Status::Success, Some(20), 20)
    );
}
fn guarded_tree() -> Tree {
    Tree::new(vec![
        Node::ReactiveSequence(vec![1, 2]),
        Node::Condition(3),
        Node::MemorySequence(vec![3, 4]),
        Node::Logical(5),
        Node::Physical(7),
    ])
    .unwrap()
}
#[test]
fn behavior_tree_reactive_guard_change_halts_displaced_running_leaf_once() {
    let mut h = Environment::new([(Status::Running, 1)]);
    let mut s = TreeState::default();
    tick(&guarded_tree(), &mut s, &mut h, 64).unwrap();
    h.guard = false;
    assert_eq!(
        tick(&guarded_tree(), &mut s, &mut h, 64).unwrap().status,
        Status::Failure
    );
    halt(&guarded_tree(), &mut s, &mut h).unwrap();
    assert_eq!(
        *h.events.borrow(),
        vec![
            Event::Condition(3),
            Event::Logical(5),
            Event::Physical(7),
            Event::Settle(1),
            Event::Condition(3),
            Event::Halt(7)
        ]
    );
    assert_eq!(s, TreeState::default());
}
#[test]
fn behavior_tree_guard_budget_deferral_retains_active_leaf_without_halt() {
    let mut h = Environment::new([(Status::Running, 1), (Status::Success, 2)]);
    let mut s = TreeState::default();
    tick(&guarded_tree(), &mut s, &mut h, 64).unwrap();
    assert!(tick(&guarded_tree(), &mut s, &mut h, 1).unwrap().exhausted);
    assert_eq!(s.running_leaves, BTreeSet::from([4]));
    tick(&guarded_tree(), &mut s, &mut h, 64).unwrap();
    assert_eq!(
        *h.events.borrow(),
        vec![
            Event::Condition(3),
            Event::Logical(5),
            Event::Physical(7),
            Event::Settle(1),
            Event::Condition(3),
            Event::Physical(7),
            Event::Settle(2)
        ]
    );
}
#[test]
fn behavior_tree_nested_completed_guard_rechecks_without_reissuing_settled_action() {
    let tree = Tree::new(vec![
        Node::ReactiveSequence(vec![1, 4]),
        Node::MemorySequence(vec![2, 3]),
        Node::Condition(3),
        Node::Physical(7),
        Node::Physical(8),
    ])
    .unwrap();
    let mut h = Environment::new([(Status::Success, 1)]);
    let mut s = TreeState::default();
    tick(&tree, &mut s, &mut h, 64).unwrap();
    h.guard = false;
    assert_eq!(
        tick(&tree, &mut s, &mut h, 64).unwrap().status,
        Status::Failure
    );
    assert_eq!(
        *h.events.borrow(),
        vec![
            Event::Condition(3),
            Event::Physical(7),
            Event::Settle(1),
            Event::Condition(3)
        ]
    );
}
#[test]
fn behavior_tree_memory_skips_prior_guard_but_reactive_fallback_preempts() {
    for reactive in [false, true] {
        let node = if reactive {
            Node::ReactiveFallback(vec![1, 2])
        } else {
            Node::MemorySequence(vec![1, 2])
        };
        let tree = Tree::new(vec![node, Node::Condition(3), Node::Physical(7)]).unwrap();
        let mut h = Environment::new([(Status::Running, 1), (Status::Success, 2)]);
        h.guard = !reactive;
        let mut s = TreeState::default();
        tick(&tree, &mut s, &mut h, 64).unwrap();
        h.guard = reactive;
        assert_eq!(
            tick(&tree, &mut s, &mut h, 64).unwrap().status,
            Status::Success
        );
        let want = if reactive {
            vec![
                Event::Condition(3),
                Event::Physical(7),
                Event::Settle(1),
                Event::Condition(3),
                Event::Halt(7),
            ]
        } else {
            vec![
                Event::Condition(3),
                Event::Physical(7),
                Event::Settle(1),
                Event::Physical(7),
                Event::Settle(2),
            ]
        };
        assert_eq!(*h.events.borrow(), want);
    }
}
#[test]
fn behavior_tree_logical_selection_is_not_reissued_after_budget_deferral() {
    let tree = Tree::new(vec![
        Node::ReactiveSequence(vec![1, 2]),
        Node::Logical(5),
        Node::Physical(7),
    ])
    .unwrap();
    let mut h = Environment::new([(Status::Success, 2)]);
    let mut s = TreeState::default();
    tick(&tree, &mut s, &mut h, 2).unwrap();
    tick(&tree, &mut s, &mut h, 2).unwrap();
    assert_eq!(
        *h.events.borrow(),
        vec![Event::Logical(5), Event::Physical(7), Event::Settle(2)]
    );
}
#[test]
fn behavior_tree_halt_accepts_active_ids_without_status_history_and_clears_state() {
    let tree = Tree::new(vec![
        Node::MemorySequence(vec![1, 2]),
        Node::Logical(5),
        Node::Physical(7),
    ])
    .unwrap();
    let mut h = Environment::new([]);
    let mut s = TreeState {
        running_leaves: BTreeSet::from([1, 2]),
        ..TreeState::default()
    };
    halt(&tree, &mut s, &mut h).unwrap();
    halt(&tree, &mut s, &mut h).unwrap();
    assert_eq!(*h.events.borrow(), vec![Event::Halt(5), Event::Halt(7)]);
    assert_eq!(s, TreeState::default());
}
#[test]
fn behavior_tree_root_completion_starts_a_new_activation() {
    let tree = Tree::new(vec![Node::Logical(5)]).unwrap();
    let mut h = Environment::new([]);
    let mut s = TreeState::default();
    tick(&tree, &mut s, &mut h, 1).unwrap();
    tick(&tree, &mut s, &mut h, 1).unwrap();
    assert_eq!(
        *h.events.borrow(),
        vec![Event::Logical(5), Event::Logical(5)]
    );
}
#[test]
fn behavior_tree_rejects_invalid_graphs_before_effects() {
    let mut deep: Vec<Node> = (1..9).map(|i| Node::MemorySequence(vec![i])).collect();
    deep.push(Node::Condition(0));
    let invalid = vec![
        vec![],
        vec![Node::Condition(0); 33],
        vec![Node::MemorySequence(vec![1])],
        vec![Node::MemorySequence(vec![0])],
        vec![Node::MemorySequence(vec![1, 1]), Node::Condition(0)],
        vec![Node::Condition(0), Node::Condition(1)],
        deep,
    ];
    for nodes in invalid {
        assert!(Tree::new(nodes.clone()).is_err());
        let tree = Tree { nodes };
        let mut h = Environment::new([]);
        let mut s = TreeState::default();
        assert!(tick(&tree, &mut s, &mut h, 64).is_err());
        assert!(halt(&tree, &mut s, &mut h).is_err());
        assert!(h.events.borrow().is_empty());
    }
}
#[test]
fn behavior_tree_accepts_boundary_shapes_and_unwinds_without_extra_visits() {
    let mut deep: Vec<Node> = (1..8).map(|i| Node::MemorySequence(vec![i])).collect();
    deep.push(Node::Physical(7));
    let tree = Tree::new(deep).unwrap();
    let mut h = Environment::new([(Status::Success, 2)]);
    let r = tick(&tree, &mut TreeState::default(), &mut h, 8).unwrap();
    assert_eq!(
        (r.status, r.visits, r.exhausted),
        (Status::Success, 8, false)
    );
    let mut wide = vec![Node::ReactiveSequence((1..32).collect())];
    wide.extend(vec![Node::Condition(0); 31]);
    let r = tick(
        &Tree::new(wide).unwrap(),
        &mut TreeState::default(),
        &mut h,
        32,
    )
    .unwrap();
    assert_eq!((r.status, r.visits), (Status::Success, 32));
}
#[test]
fn behavior_tree_empty_composites_have_identity_results() {
    for (node, expected) in [
        (Node::MemorySequence(vec![]), Status::Success),
        (Node::ReactiveSequence(vec![]), Status::Success),
        (Node::ReactiveFallback(vec![]), Status::Failure),
    ] {
        let r = tick(
            &Tree::new(vec![node]).unwrap(),
            &mut TreeState::default(),
            &mut Environment::new([]),
            1,
        )
        .unwrap();
        assert_eq!((r.status, r.exhausted), (expected, false));
    }
}
#[test]
fn behavior_tree_rejects_bad_saved_ids_types_and_explicit_status_conflicts_atomically() {
    let tree = guarded_tree();
    let invalid = vec![
        TreeState {
            cursors: BTreeMap::from([(0, 2)]),
            ..TreeState::default()
        },
        TreeState {
            cursors: BTreeMap::from([(1, 0)]),
            ..TreeState::default()
        },
        TreeState {
            cursors: BTreeMap::from([(9, 0)]),
            ..TreeState::default()
        },
        TreeState {
            statuses: BTreeMap::from([(9, Status::Running)]),
            ..TreeState::default()
        },
        TreeState {
            statuses: BTreeMap::from([(1, Status::Running)]),
            ..TreeState::default()
        },
        TreeState {
            running_leaves: BTreeSet::from([9]),
            ..TreeState::default()
        },
        TreeState {
            running_leaves: BTreeSet::from([1]),
            ..TreeState::default()
        },
        TreeState {
            running_leaves: BTreeSet::from([0]),
            ..TreeState::default()
        },
        TreeState {
            running_leaves: BTreeSet::from([4]),
            statuses: BTreeMap::from([(4, Status::Success)]),
            ..TreeState::default()
        },
        TreeState {
            deferred: Some(9),
            ..TreeState::default()
        },
    ];
    for mut s in invalid {
        let before = s.clone();
        let mut h = Environment::new([]);
        assert!(s.validate_for_tree(&tree).is_err());
        assert!(tick(&tree, &mut s, &mut h, 64).is_err());
        assert!(halt(&tree, &mut s, &mut h).is_err());
        assert_eq!(s, before);
        assert!(h.events.borrow().is_empty());
    }
}
#[test]
fn behavior_tree_rejects_unsupported_future_leaf_and_invalid_budgets_before_callbacks() {
    let unsupported = Tree::new(vec![
        Node::ReactiveSequence(vec![1, 2]),
        Node::Condition(0),
        Node::Physical(99),
    ])
    .unwrap();
    for (tree, budget) in [(unsupported, 64), (guarded_tree(), 0), (guarded_tree(), 65)] {
        let mut h = Environment::new([]);
        assert!(tick(&tree, &mut TreeState::default(), &mut h, budget).is_err());
        assert!(h.events.borrow().is_empty());
    }
}
#[test]
fn behavior_tree_wire_rejects_unknown_tree_and_state_fields() {
    assert!(serde_json::from_str::<Tree>(r#"{"nodes":[{"condition":0}],"extra":true}"#).is_err());
    assert!(serde_json::from_str::<TreeState>(
        r#"{"cursors":{},"statuses":{},"running_leaves":[],"deferred":null,"extra":true}"#
    )
    .is_err());
}
