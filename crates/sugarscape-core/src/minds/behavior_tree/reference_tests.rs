//! A finite-state oracle for four small graphs. It has no evaluator stack,
//! TreeState, or calls into production traversal. The transition table and
//! explicit phases model the one-action adaptation, including cached failure.
use super::runtime::*;
use super::runtime_tests::{Environment, Event};
use std::collections::{BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug)]
enum Shape {
    Memory,
    Reactive,
    Fallback,
    Nested,
}
impl Shape {
    fn tree(self) -> Tree {
        Tree::new(match self {
            Self::Memory => vec![
                Node::MemorySequence(vec![1, 2]),
                Node::Physical(7),
                Node::Physical(8),
            ],
            Self::Reactive => vec![
                Node::ReactiveSequence(vec![1, 2]),
                Node::Physical(7),
                Node::Physical(8),
            ],
            Self::Fallback => vec![
                Node::ReactiveFallback(vec![1, 2]),
                Node::Physical(7),
                Node::Physical(8),
            ],
            Self::Nested => vec![
                Node::ReactiveSequence(vec![1, 4]),
                Node::ReactiveSequence(vec![2, 3]),
                Node::Condition(3),
                Node::Physical(7),
                Node::Physical(8),
            ],
        })
        .unwrap()
    }
    fn node(self, phase: usize) -> u8 {
        match self {
            Self::Nested => [3, 4][phase],
            _ => [1, 2][phase],
        }
    }
}
#[derive(Clone, Copy)]
enum Transition {
    Advance,
    Finish(Status),
    Wait,
}
// Columns are Success, Failure, Running; rows are sequence and fallback.
const TRANSITIONS: [[Transition; 3]; 2] = [
    [
        Transition::Advance,
        Transition::Finish(Status::Failure),
        Transition::Wait,
    ],
    [
        Transition::Finish(Status::Success),
        Transition::Advance,
        Transition::Wait,
    ],
];
struct Reference {
    shape: Shape,
    phase: usize,
    active: Option<usize>,
    deferred: Option<u8>,
    queue: VecDeque<(Status, u32)>,
    log: Vec<Event>,
    gathered: u32,
    quota: u32,
}
impl Reference {
    fn stop(&mut self) {
        if let Some(phase) = self.active.take() {
            self.log.push(Event::Halt([7, 8][phase]));
        }
        self.phase = 0;
        self.deferred = None;
    }
    fn enter(&mut self, node: u8, limit: u16, result: &mut Tick<u32>) -> bool {
        if result.visits == limit {
            result.exhausted = true;
            self.deferred = Some(node);
            false
        } else {
            result.visits += 1;
            true
        }
    }
    fn turn(&mut self, limit: u16, guard: bool) -> Tick<u32> {
        self.deferred = None;
        let mut result = Tick {
            status: Status::Running,
            receipt: None,
            visits: 0,
            exhausted: false,
            deferred_physical: false,
        };
        if self.gathered >= self.quota {
            self.stop();
            result.status = Status::Success;
            return result;
        }
        // Every fixture enters its root. Nested then checks its earlier guard
        // subtree on every turn, even after action7 has finished successfully.
        result.visits = 1;
        if matches!(self.shape, Shape::Nested) {
            if !self.enter(1, limit, &mut result) || !self.enter(2, limit, &mut result) {
                return result;
            }
            self.log.push(Event::Condition(3));
            if !guard {
                self.stop();
                result.status = Status::Failure;
                return result;
            }
        }
        loop {
            let node = self.shape.node(self.phase);
            if !self.enter(node, limit, &mut result) {
                return result;
            }
            if result.receipt.is_some() {
                result.deferred_physical = true;
                self.deferred = Some(node);
                return result;
            }
            let (status, amount) = self
                .queue
                .pop_front()
                .expect("at most one action per supplied turn");
            self.log.push(Event::Physical([7, 8][self.phase]));
            self.log.push(Event::Settle(amount));
            self.gathered += amount;
            result.receipt = Some(amount);
            self.active = (status == Status::Running).then_some(self.phase);
            if self.gathered >= self.quota {
                self.stop();
                result.status = Status::Success;
                return result;
            }
            let row = usize::from(matches!(self.shape, Shape::Fallback));
            let column = match status {
                Status::Success => 0,
                Status::Failure => 1,
                Status::Running => 2,
            };
            match TRANSITIONS[row][column] {
                Transition::Wait => return result,
                Transition::Finish(status) => {
                    self.stop();
                    result.status = status;
                    return result;
                }
                Transition::Advance => {
                    self.phase += 1;
                    if self.phase == 2 {
                        self.stop();
                        result.status = if row == 0 {
                            Status::Success
                        } else {
                            Status::Failure
                        };
                        return result;
                    }
                }
            }
        }
    }
}
fn scripts(len: u32) -> Vec<Vec<(Status, u32)>> {
    (0..3_u32.pow(len))
        .map(|mut code| {
            (0..len)
                .map(|index| {
                    let status =
                        [Status::Success, Status::Failure, Status::Running][(code % 3) as usize];
                    code /= 3;
                    (status, index + 1)
                })
                .collect()
        })
        .collect()
}
#[test]
fn behavior_tree_reference_exhaustive_short_scripts() {
    let mut comparisons = 0;
    // Length zero is a condition-only graph; no unavailable action queue can
    // accidentally serve as an oracle result or expected runtime failure.
    for budget in [1, 2, 3, 64] {
        for guard in [false, true] {
            let mut host = Environment::new([]);
            host.guard = guard;
            let tree = Tree::new(vec![Node::Condition(3)]).unwrap();
            let actual = tick(&tree, &mut TreeState::default(), &mut host, budget).unwrap();
            assert_eq!(
                actual,
                Tick {
                    status: if guard {
                        Status::Success
                    } else {
                        Status::Failure
                    },
                    receipt: None,
                    visits: 1,
                    exhausted: false,
                    deferred_physical: false
                }
            );
            assert_eq!(*host.events.borrow(), vec![Event::Condition(3)]);
            comparisons += 1;
        }
    }
    for len in 1..=4 {
        for script in scripts(len) {
            for shape in [
                Shape::Memory,
                Shape::Reactive,
                Shape::Fallback,
                Shape::Nested,
            ] {
                for budget in [1, 2, 3, 64] {
                    for quota in [1, 3, u32::MAX] {
                        for interrupt in [false, true] {
                            let tree = shape.tree();
                            let mut host = Environment::new(script.clone());
                            host.quota = quota;
                            let mut state = TreeState::default();
                            let mut reference = Reference {
                                shape,
                                phase: 0,
                                active: None,
                                deferred: None,
                                queue: script.clone().into(),
                                log: vec![],
                                gathered: 0,
                                quota,
                            };
                            for turn in 0..len {
                                host.guard = !interrupt || turn % 3 != 1;
                                let actual = tick(&tree, &mut state, &mut host, budget).unwrap();
                                let expected = reference.turn(budget, host.guard);
                                let context = format!("shape={shape:?} script={script:?} budget={budget} quota={quota} interrupt={interrupt} turn={turn}");
                                assert_eq!(actual, expected, "{context}");
                                assert_eq!(*host.events.borrow(), reference.log, "{context}");
                                assert_eq!(host.gathered, reference.gathered, "{context}");
                                assert_eq!(state.deferred, reference.deferred, "{context}");
                                let active: BTreeSet<u8> = reference
                                    .active
                                    .map(|phase| shape.node(phase))
                                    .into_iter()
                                    .collect();
                                assert_eq!(state.running_leaves, active, "{context}");
                                state.validate_for_tree(&tree).unwrap();
                                // Persistence is part of continuation, with diagnostic logs kept
                                // outside the serialized production traversal state.
                                state =
                                    serde_json::from_str(&serde_json::to_string(&state).unwrap())
                                        .unwrap();
                                if interrupt && turn == 2 {
                                    halt(&tree, &mut state, &mut host).unwrap();
                                    reference.stop();
                                    assert_eq!(*host.events.borrow(), reference.log, "{context}");
                                    assert_eq!(state, TreeState::default());
                                }
                                comparisons += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(comparisons, 40904);
}

/// Separate flat machine for Fallback[Sequence[guard, action7, action8], action9].
/// A successful or running higher-priority action claims the continuation. A
/// failed candidate returns control to action9; an unfinished guard probe leaves
/// the owner intact. These transitions are independent of evaluator stack state.
struct PreemptionReference {
    actions: [Option<Status>; 3],
    active: BTreeSet<u8>,
    cursor: u8,
    deferred: Option<u8>,
    queue: VecDeque<(Status, u32)>,
    log: Vec<Event>,
}
impl PreemptionReference {
    fn retire(&mut self, action: usize) {
        if self.active.remove(&[3, 4, 5][action]) {
            self.log.push(Event::Halt([7, 8, 9][action]));
            self.actions[action] = None;
        }
    }
    fn reset(&mut self) {
        for action in 0..3 {
            self.retire(action);
        }
        self.actions = [None; 3];
        self.cursor = 0;
        self.deferred = None;
    }
    fn enter(&mut self, node: u8, budget: u16, tick: &mut Tick<u32>) -> bool {
        if tick.visits == budget {
            self.deferred = Some(node);
            tick.exhausted = true;
            false
        } else {
            tick.visits += 1;
            true
        }
    }
    fn action(&mut self, action: usize, budget: u16, tick: &mut Tick<u32>) -> Option<Status> {
        if let Some(status @ (Status::Success | Status::Failure)) = self.actions[action] {
            return Some(status);
        }
        let node = [3, 4, 5][action];
        if !self.enter(node, budget, tick) {
            return None;
        }
        if tick.receipt.is_some() {
            self.deferred = Some(node);
            tick.deferred_physical = true;
            return None;
        }
        let (status, receipt) = self
            .queue
            .pop_front()
            .expect("one supplied result per possible physical turn");
        self.log.push(Event::Physical([7, 8, 9][action]));
        self.log.push(Event::Settle(receipt));
        tick.receipt = Some(receipt);
        self.actions[action] = Some(status);
        if status == Status::Running {
            self.active.insert(node);
        } else {
            self.active.remove(&node);
        }
        Some(status)
    }
    fn turn(&mut self, budget: u16, guard: bool) -> Tick<u32> {
        self.deferred = None;
        let mut tick = Tick {
            status: Status::Running,
            receipt: None,
            visits: 1,
            exhausted: false,
            deferred_physical: false,
        };
        if !self.enter(1, budget, &mut tick) || !self.enter(2, budget, &mut tick) {
            return tick;
        }
        self.log.push(Event::Condition(0));
        if guard {
            let mut high_succeeded = true;
            for action in 0..2 {
                let cached = matches!(
                    self.actions[action],
                    Some(Status::Success | Status::Failure)
                );
                let Some(status) = self.action(action, budget, &mut tick) else {
                    return tick;
                };
                match TRANSITIONS[0][match status {
                    Status::Success => 0,
                    Status::Failure => 1,
                    Status::Running => 2,
                }] {
                    Transition::Advance => {
                        // Replaying an old success is not new replacement work.
                        if !cached {
                            self.retire(2);
                            self.cursor = 0;
                        }
                    }
                    Transition::Wait => {
                        self.retire(2);
                        self.cursor = 0;
                        return tick;
                    }
                    Transition::Finish(_) => {
                        high_succeeded = false;
                        break;
                    }
                }
            }
            if high_succeeded {
                self.reset();
                tick.status = Status::Success;
                return tick;
            }
        }
        // A false guard/failing higher branch retires only its own activity.
        self.retire(0);
        self.retire(1);
        self.cursor = 1;
        if let Some(status) = self.action(2, budget, &mut tick) {
            tick.status = status;
            if status != Status::Running {
                self.reset();
            }
        }
        tick
    }
}
#[test]
fn behavior_tree_reference_exhaustive_replacement_deferral() {
    let tree = Tree::new(vec![
        Node::ReactiveFallback(vec![1, 5]),
        Node::ReactiveSequence(vec![2, 3, 4]),
        Node::Condition(0),
        Node::Physical(7),
        Node::Physical(8),
        Node::Physical(9),
    ])
    .unwrap();
    let mut comparisons = 0;
    for len in 1..=4 {
        for script in scripts(len) {
            for budget in [1, 2, 3, 4, 5, 64] {
                for guards in [
                    [true, true, false, true],
                    [true, false, true, false],
                    [false, true, true, true],
                ] {
                    let mut host = Environment::new(
                        std::iter::once((Status::Running, 9)).chain(script.clone()),
                    );
                    host.guard = false;
                    let mut state = TreeState::default();
                    let initial = tick(&tree, &mut state, &mut host, 64).unwrap();
                    assert_eq!(
                        initial,
                        Tick {
                            status: Status::Running,
                            receipt: Some(9),
                            visits: 4,
                            exhausted: false,
                            deferred_physical: false
                        }
                    );
                    let mut reference = PreemptionReference {
                        actions: [None, None, Some(Status::Running)],
                        active: BTreeSet::from([5]),
                        cursor: 1,
                        deferred: None,
                        queue: script.clone().into(),
                        log: vec![Event::Condition(0), Event::Physical(9), Event::Settle(9)],
                    };
                    for (turn, &guard) in guards.iter().enumerate().take(len as usize) {
                        host.guard = guard;
                        let actual = tick(&tree, &mut state, &mut host, budget).unwrap();
                        let expected = reference.turn(budget, host.guard);
                        let context = format!(
                            "script={script:?} budget={budget} guards={guards:?} turn={turn}"
                        );
                        assert_eq!(actual, expected, "{context}");
                        assert_eq!(*host.events.borrow(), reference.log, "{context}");
                        assert_eq!(state.running_leaves, reference.active, "{context}");
                        assert_eq!(state.deferred, reference.deferred, "{context}");
                        assert_eq!(
                            state.cursors.get(&0).copied(),
                            (actual.status == Status::Running).then_some(reference.cursor),
                            "{context}"
                        );
                        state.validate_for_tree(&tree).unwrap();
                        state =
                            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
                        comparisons += 1;
                    }
                }
            }
        }
    }
    assert_eq!(comparisons, 7668);
}
