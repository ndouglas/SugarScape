//! Propositional STRIPS (Fikes & Nilsson 1971, §3.2 "Operator descriptions
//! and applications"): a state is a set of facts, an action has
//! preconditions, a delete list and an add list, and applying it removes the
//! delete list and then adds the add list. Facts are bits of a `u128`.
//!
//! The generators build the task families Helmert & Mattmüller (2007) use
//! to bound heuristic accuracy, whose optimal plan lengths they prove; the
//! planner is verified against those lengths (tests/goap.rs).

use std::collections::HashMap;

use super::Domain;

/// The most facts a [`Strips`] task can have (the bits of its state).
pub const MAX_FACTS: usize = 128;

/// A ground STRIPS action: applicable when every `pre` fact holds; applying
/// it deletes `del`, then adds `add`.
#[derive(Clone, Debug, PartialEq)]
pub struct StripsAction {
    pub name: String,
    pub pre: u128,
    pub add: u128,
    pub del: u128,
    pub cost: f64,
}

/// A ground STRIPS task: `facts` facts, a start state, the actions and the
/// goal (the facts every goal state holds).
#[derive(Clone, Debug)]
pub struct Strips {
    pub facts: usize,
    pub init: u128,
    pub actions: Vec<StripsAction>,
    pub goal: u128,
    /// The heuristic's cost per unsatisfied goal fact: the cheapest action's
    /// cost when no action adds more than one goal fact, else 0.
    per_goal: f64,
}

impl Strips {
    /// A task. Panics on more than [`MAX_FACTS`] facts, a fact outside
    /// them, or a cost that isn't positive.
    pub fn new(facts: usize, init: u128, actions: Vec<StripsAction>, goal: u128) -> Self {
        assert!(facts <= MAX_FACTS, "{facts} facts > {MAX_FACTS}");
        let all = if facts == MAX_FACTS {
            u128::MAX
        } else {
            (1u128 << facts) - 1
        };
        assert_eq!(init & !all, 0, "init names a fact past {facts}");
        assert_eq!(goal & !all, 0, "goal names a fact past {facts}");
        for a in &actions {
            assert_eq!(
                (a.pre | a.add | a.del) & !all,
                0,
                "{}: fact past {facts}",
                a.name
            );
            assert!(a.cost > 0.0, "{}: cost {} is not positive", a.name, a.cost);
        }
        // Counting unsatisfied goal facts is admissible and consistent only
        // when one action can satisfy at most one of them.
        let informed = actions.iter().all(|a| (a.add & goal).count_ones() <= 1);
        let per_goal = if informed {
            actions.iter().map(|a| a.cost).fold(f64::INFINITY, f64::min)
        } else {
            0.0
        };
        let per_goal = if per_goal.is_finite() { per_goal } else { 0.0 };
        Strips {
            facts,
            init,
            actions,
            goal,
            per_goal,
        }
    }

    /// Whether the heuristic counts unsatisfied goal facts (else it is 0).
    pub fn informed(&self) -> bool {
        self.per_goal > 0.0
    }
}

impl Domain for Strips {
    type State = u128;
    /// The action's index in [`Strips::actions`].
    type Action = usize;

    fn actions(&self, s: &u128, out: &mut Vec<(usize, f64, u128)>) {
        for (i, a) in self.actions.iter().enumerate() {
            if s & a.pre == a.pre {
                out.push((i, a.cost, (s & !a.del) | a.add));
            }
        }
    }

    fn is_goal(&self, s: &u128) -> bool {
        s & self.goal == self.goal
    }

    fn heuristic(&self, s: &u128) -> f64 {
        (self.goal & !s).count_ones() as f64 * self.per_goal
    }
}

/// Names facts as they're first used and builds ground actions over them.
#[derive(Default)]
struct Builder {
    facts: HashMap<String, usize>,
    actions: Vec<StripsAction>,
}

impl Builder {
    fn fact(&mut self, name: &str) -> u128 {
        let next = self.facts.len();
        let i = *self.facts.entry(name.to_string()).or_insert(next);
        assert!(i < MAX_FACTS, "more than {MAX_FACTS} facts");
        1u128 << i
    }

    fn facts(&mut self, names: &[String]) -> u128 {
        names.iter().fold(0, |m, n| m | self.fact(n))
    }

    fn action(&mut self, name: String, pre: &[String], add: &[String], del: &[String]) {
        let (pre, add, del) = (self.facts(pre), self.facts(add), self.facts(del));
        self.actions.push(StripsAction {
            name,
            pre,
            add,
            del,
            cost: 1.0,
        });
    }

    fn build(mut self, init: &[String], goal: &[String]) -> Strips {
        let (init, goal) = (self.facts(init), self.facts(goal));
        Strips::new(self.facts.len(), init, self.actions, goal)
    }
}

/// Gripper (IPC 1998; Helmert & Mattmüller 2007, "Results: Performance
/// ratio of h+", GRIPPER, and "Performance ratio of additive pattern
/// database heuristics", GRIPPER): a robot with two grippers moves between
/// rooms A and B; `n` balls start in A with the robot and both grippers
/// free, and the goal is every ball in B. Unit costs. The optimal cost is
/// 3n − 1 for even n and 3n for odd n.
pub fn gripper(n: usize) -> Strips {
    let rooms = ["A", "B"];
    let grippers = ["left", "right"];
    let mut b = Builder::default();
    for from in rooms {
        for to in rooms {
            if from != to {
                b.action(
                    format!("move({from},{to})"),
                    &[format!("at-robby({from})")],
                    &[format!("at-robby({to})")],
                    &[format!("at-robby({from})")],
                );
            }
        }
    }
    for ball in 1..=n {
        for room in rooms {
            for g in grippers {
                b.action(
                    format!("pick(ball{ball},{room},{g})"),
                    &[
                        format!("at(ball{ball},{room})"),
                        format!("at-robby({room})"),
                        format!("free({g})"),
                    ],
                    &[format!("carry(ball{ball},{g})")],
                    &[format!("at(ball{ball},{room})"), format!("free({g})")],
                );
                b.action(
                    format!("drop(ball{ball},{room},{g})"),
                    &[
                        format!("carry(ball{ball},{g})"),
                        format!("at-robby({room})"),
                    ],
                    &[format!("at(ball{ball},{room})"), format!("free({g})")],
                    &[format!("carry(ball{ball},{g})")],
                );
            }
        }
    }
    let mut init = vec![
        "at-robby(A)".to_string(),
        "free(left)".to_string(),
        "free(right)".to_string(),
    ];
    init.extend((1..=n).map(|i| format!("at(ball{i},A)")));
    let goal: Vec<String> = (1..=n).map(|i| format!("at(ball{i},B)")).collect();
    b.build(&init, &goal)
}

/// Logistics trucks' actions (IPC 1998, STRIPS, no airplanes): drive between
/// two locations of a city, load a package at the truck's location, unload
/// it there.
fn truck_actions(b: &mut Builder, truck: &str, locations: &[String], packages: &[String]) {
    for from in locations {
        for to in locations {
            if from != to {
                b.action(
                    format!("drive({truck},{from},{to})"),
                    &[format!("at({truck},{from})")],
                    &[format!("at({truck},{to})")],
                    &[format!("at({truck},{from})")],
                );
            }
        }
    }
    for p in packages {
        for l in locations {
            b.action(
                format!("load({p},{truck},{l})"),
                &[format!("at({truck},{l})"), format!("at({p},{l})")],
                &[format!("in({p},{truck})")],
                &[format!("at({p},{l})")],
            );
            b.action(
                format!("unload({p},{truck},{l})"),
                &[format!("at({truck},{l})"), format!("in({p},{truck})")],
                &[format!("at({p},{l})")],
                &[format!("in({p},{truck})")],
            );
        }
    }
}

/// Logistics with a truck per city (Helmert & Mattmüller 2007, "Results:
/// Performance ratio of h+", LOGISTICS): `n` cities, no airports; city i has
/// two locations ℓi1 and ℓi2, a truck ti at ℓi1 and a package with origin ℓi2
/// and destination ℓi1. "An optimal plan consists of 4n actions, namely two
/// drive, one pick-up and one drop action in each city."
///
/// Grounded over reachable facts only, as a planner's relaxed-reachability
/// grounding would: with no airports nothing leaves its city, so truck ti and
/// package pi are only ever at city i's locations (or pi in ti).
pub fn logistics_trucks(n: usize) -> Strips {
    let mut b = Builder::default();
    let (mut init, mut goal) = (Vec::new(), Vec::new());
    for i in 1..=n {
        let (truck, package) = (format!("t{i}"), format!("p{i}"));
        let locations = [format!("l{i}-1"), format!("l{i}-2")];
        truck_actions(&mut b, &truck, &locations, std::slice::from_ref(&package));
        init.push(format!("at({truck},{})", locations[0]));
        init.push(format!("at({package},{})", locations[1]));
        goal.push(format!("at({package},{})", locations[0]));
    }
    b.build(&init, &goal)
}

/// Logistics with one truck (Helmert & Mattmüller 2007, "Performance ratio
/// of additive pattern database heuristics", LOGISTICS): "a single truck in a
/// city with 2n + 1 locations. There are n packages, all to be moved within
/// this city. All initial and goal locations of the packages are different
/// from each other and from the initial truck location." The truck starts at
/// ℓ0 and package i goes from ℓ(2i − 1) to ℓ(2i). The optimal cost is 4n.
pub fn logistics_one_truck(n: usize) -> Strips {
    let mut b = Builder::default();
    let locations: Vec<String> = (0..=2 * n).map(|l| format!("l{l}")).collect();
    let packages: Vec<String> = (1..=n).map(|i| format!("p{i}")).collect();
    truck_actions(&mut b, "t", &locations, &packages);
    let mut init = vec!["at(t,l0)".to_string()];
    init.extend((1..=n).map(|i| format!("at(p{i},l{})", 2 * i - 1)));
    let goal: Vec<String> = (1..=n).map(|i| format!("at(p{i},l{})", 2 * i)).collect();
    b.build(&init, &goal)
}

/// The blocks family (Helmert & Mattmüller 2007, "Results: Performance
/// ratio of h+", BLOCKSWORLD), in the 4-operator encoding (pick-up,
/// put-down, stack, unstack; IPC 2000): one stack B1, …, B(n+1) from top to
/// bottom and a singleton stack B(n+2), the hand empty; the goal is the
/// stack B1, …, Bn, B(n+2), "with the position of B(n+1) not being
/// important" (the goal is the n `on` facts). The optimal cost is 4n − 2.
pub fn blocks_family(n: usize) -> Strips {
    let blocks: Vec<String> = (1..=n + 2).map(|i| format!("B{i}")).collect();
    let mut b = Builder::default();
    for x in &blocks {
        b.action(
            format!("pick-up({x})"),
            &[
                format!("clear({x})"),
                format!("ontable({x})"),
                "handempty".to_string(),
            ],
            &[format!("holding({x})")],
            &[
                format!("ontable({x})"),
                format!("clear({x})"),
                "handempty".to_string(),
            ],
        );
        b.action(
            format!("put-down({x})"),
            &[format!("holding({x})")],
            &[
                format!("clear({x})"),
                "handempty".to_string(),
                format!("ontable({x})"),
            ],
            &[format!("holding({x})")],
        );
        for y in &blocks {
            if x == y {
                continue;
            }
            b.action(
                format!("stack({x},{y})"),
                &[format!("holding({x})"), format!("clear({y})")],
                &[
                    format!("on({x},{y})"),
                    format!("clear({x})"),
                    "handempty".to_string(),
                ],
                &[format!("holding({x})"), format!("clear({y})")],
            );
            b.action(
                format!("unstack({x},{y})"),
                &[
                    format!("on({x},{y})"),
                    format!("clear({x})"),
                    "handempty".to_string(),
                ],
                &[format!("holding({x})"), format!("clear({y})")],
                &[
                    format!("on({x},{y})"),
                    format!("clear({x})"),
                    "handempty".to_string(),
                ],
            );
        }
    }
    let mut init = vec![
        "handempty".to_string(),
        "clear(B1)".to_string(),
        format!("clear(B{})", n + 2),
    ];
    init.extend((1..=n).map(|i| format!("on(B{i},B{})", i + 1)));
    init.push(format!("ontable(B{})", n + 1));
    init.push(format!("ontable(B{})", n + 2));
    let mut goal: Vec<String> = (1..n).map(|i| format!("on(B{i},B{})", i + 1)).collect();
    goal.push(format!("on(B{n},B{})", n + 2));
    b.build(&init, &goal)
}
