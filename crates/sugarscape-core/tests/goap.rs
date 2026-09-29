//! Minds 4's GOAP planner: verified on STRIPS families with known optimal
//! plan lengths (Helmert & Mattmüller 2007) and against a Dijkstra over the
//! fact-set graph on random STRIPS instances.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use rand::{Rng, SeedableRng};
use sugarscape_core::minds::goap::plan;
use sugarscape_core::minds::goap::strips::{
    blocks_family, gripper, logistics_one_truck, logistics_trucks, Strips, StripsAction,
};

/// Replays `actions` from the start (Fikes & Nilsson's semantics: the
/// preconditions hold, the delete list is removed, then the add list is
/// added), checks it ends at a goal, and returns its cost.
fn replay(p: &Strips, actions: &[usize]) -> f64 {
    let mut s = p.init;
    let mut cost = 0.0;
    for &i in actions {
        let a = &p.actions[i];
        assert_eq!(
            s & a.pre,
            a.pre,
            "{} applied without its preconditions",
            a.name
        );
        s = (s & !a.del) | a.add;
        cost += a.cost;
    }
    assert_eq!(s & p.goal, p.goal, "the plan does not reach the goal");
    cost
}

/// Plans `p`, checks the plan replays to a goal at the reported cost, and
/// returns that cost.
fn optimal(p: &Strips) -> f64 {
    let found = plan(p, p.init, usize::MAX).expect("the family is solvable");
    assert_eq!(replay(p, &found.actions), found.cost);
    found.cost
}

#[test]
fn gripper_costs_3n_minus_1_for_even_n_and_3n_for_odd_n() {
    for n in 1..=8usize {
        let want = if n % 2 == 0 { 3 * n - 1 } else { 3 * n };
        assert_eq!(optimal(&gripper(n)), want as f64, "gripper({n})");
    }
}

#[test]
fn logistics_with_a_truck_per_city_costs_4n() {
    for n in 1..=6usize {
        assert_eq!(
            optimal(&logistics_trucks(n)),
            (4 * n) as f64,
            "logistics_trucks({n})"
        );
    }
}

#[test]
fn logistics_with_one_truck_costs_4n() {
    // n = 6 (97 facts, 13 locations) expands about 19 million states under
    // the goal-counting heuristic, about a minute in release; it runs below
    // with `--ignored`.
    for n in 1..=5usize {
        assert_eq!(
            optimal(&logistics_one_truck(n)),
            (4 * n) as f64,
            "logistics_one_truck({n})"
        );
    }
}

#[test]
#[ignore = "about a minute in release: 19 million expansions"]
fn logistics_with_one_truck_costs_24_for_6_packages() {
    assert_eq!(optimal(&logistics_one_truck(6)), 24.0);
}

#[test]
fn the_blocks_family_costs_4n_minus_2() {
    for n in 1..=6usize {
        assert_eq!(
            optimal(&blocks_family(n)),
            (4 * n - 2) as f64,
            "blocks_family({n})"
        );
    }
}

/// Dijkstra's optimal cost over the fact-set graph, the oracle: no
/// heuristic, no interning, its own successor function.
fn dijkstra(p: &Strips) -> Option<f64> {
    #[derive(PartialEq)]
    struct Item(f64, u128);
    impl Eq for Item {}
    impl PartialOrd for Item {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Item {
        fn cmp(&self, o: &Self) -> Ordering {
            o.0.total_cmp(&self.0).then(o.1.cmp(&self.1))
        }
    }
    let mut dist: HashMap<u128, f64> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(p.init, 0.0);
    heap.push(Item(0.0, p.init));
    while let Some(Item(d, s)) = heap.pop() {
        if d > dist[&s] {
            continue;
        }
        if s & p.goal == p.goal {
            return Some(d);
        }
        for a in &p.actions {
            if s & a.pre != a.pre {
                continue;
            }
            let t = (s & !a.del) | a.add;
            let dt = d + a.cost;
            if dist.get(&t).is_none_or(|&b| dt < b) {
                dist.insert(t, dt);
                heap.push(Item(dt, t));
            }
        }
    }
    None
}

#[test]
fn random_strips_instances_match_dijkstra() {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(20260928);
    let (mut stepped, mut unsolvable, mut informed) = (0, 0, 0);
    for case in 0..500 {
        let facts = rng.gen_range(6..=12usize);
        let all = (1u128 << facts) - 1;
        // A sparse random subset of the facts: each fact with probability 1/k.
        let subset = |rng: &mut rand_pcg::Pcg64Mcg, k: u32| {
            (0..facts).fold(
                0u128,
                |m, f| {
                    if rng.gen_ratio(1, k) {
                        m | 1 << f
                    } else {
                        m
                    }
                },
            )
        };
        let n = rng.gen_range(4..=12usize);
        let actions: Vec<StripsAction> = (0..n)
            .map(|i| StripsAction {
                name: format!("a{i}"),
                pre: subset(&mut rng, 4),
                add: subset(&mut rng, 4),
                del: subset(&mut rng, 3),
                cost: rng.gen_range(1..=3u32) as f64,
            })
            .collect();
        let init = subset(&mut rng, 2);
        let goal = subset(&mut rng, 3) & all;
        let p = Strips::new(facts, init, actions, goal);
        if p.informed() {
            informed += 1;
        }
        let want = dijkstra(&p);
        let got = plan(&p, p.init, usize::MAX);
        match (want, got) {
            (None, None) => unsolvable += 1,
            (Some(w), Some(g)) => {
                assert_eq!(g.cost, w, "case {case}: cost");
                assert_eq!(replay(&p, &g.actions), g.cost, "case {case}: replay");
                if !g.actions.is_empty() {
                    stepped += 1;
                }
            }
            (w, g) => panic!("case {case}: Dijkstra {w:?}, plan {:?}", g.map(|g| g.cost)),
        }
    }
    // The sample must exercise both outcomes and both heuristics (seeded:
    // 130 non-empty plans, 107 starts already at a goal, 263 unsolvable,
    // 219 informed).
    assert!(
        stepped >= 100 && unsolvable >= 50,
        "{stepped} non-empty, {unsolvable} unsolvable"
    );
    assert!((50..=450).contains(&informed), "{informed} informed");
}
