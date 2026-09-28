//! A* (Hart, Nilsson & Raphael 1968) over any graph with an admissible,
//! consistent heuristic (Minds 2). Deterministic: ties go to the lowest f,
//! then the lowest h, then the earliest pushed; it draws no random numbers.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// A graph A* can search. Nodes are indices.
pub trait Graph {
    /// Appends `n`'s neighbors and step costs to `out`, in the graph's
    /// fixed order (the tie rule's push order).
    fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>);
    /// A lower bound on the cost from `n` to `goal` (admissible) that never
    /// drops by more than a step's cost (consistent).
    fn heuristic(&self, n: usize, goal: usize) -> f64;
}

/// A found path: `start ..= goal`, its cost, and how many nodes were
/// expanded to find it.
#[derive(Clone, Debug, PartialEq)]
pub struct Search {
    pub path: Vec<usize>,
    pub cost: f64,
    pub expanded: usize,
}

struct Open {
    f: f64,
    h: f64,
    seq: u64,
    node: usize,
    g: f64,
}

impl PartialEq for Open {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for Open {}
impl PartialOrd for Open {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Open {
    /// Reversed, so `BinaryHeap` pops the lowest f, then h, then the
    /// earliest pushed (lowest `seq`).
    fn cmp(&self, o: &Self) -> Ordering {
        o.f.total_cmp(&self.f)
            .then(o.h.total_cmp(&self.h))
            .then(o.seq.cmp(&self.seq))
    }
}

/// The minimum-cost path from `start` to `goal`, or `None` when there is
/// none or finding it would expand more than `limit` nodes.
pub fn astar<G: Graph>(g: &G, start: usize, goal: usize, limit: usize) -> Option<Search> {
    if start == goal {
        return Some(Search {
            path: vec![start],
            cost: 0.0,
            expanded: 0,
        });
    }
    let mut best: HashMap<usize, (f64, usize)> = HashMap::new(); // g, parent
    let mut closed: HashMap<usize, ()> = HashMap::new();
    let mut open = BinaryHeap::new();
    let mut seq = 0u64;
    let h0 = g.heuristic(start, goal);
    best.insert(start, (0.0, start));
    open.push(Open {
        f: h0,
        h: h0,
        seq,
        node: start,
        g: 0.0,
    });
    let mut out = Vec::new();
    let mut expanded = 0usize;
    while let Some(Open { node, g: gn, .. }) = open.pop() {
        if closed.contains_key(&node) || gn > best[&node].0 {
            continue;
        }
        if node == goal {
            let mut path = vec![goal];
            let mut at = goal;
            while at != start {
                at = best[&at].1;
                path.push(at);
            }
            path.reverse();
            return Some(Search {
                path,
                cost: gn,
                expanded,
            });
        }
        if expanded == limit {
            return None;
        }
        expanded += 1;
        closed.insert(node, ());
        out.clear();
        g.neighbors(node, &mut out);
        for &(m, c) in &out {
            let gm = gn + c;
            if closed.contains_key(&m) || best.get(&m).is_some_and(|&(b, _)| b <= gm) {
                continue;
            }
            best.insert(m, (gm, node));
            let h = g.heuristic(m, goal);
            seq += 1;
            open.push(Open {
                f: gm + h,
                h,
                seq,
                node: m,
                g: gm,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Pos, Torus};
    use crate::minds::grid::TorusGrid;

    #[test]
    fn start_is_goal() {
        let t = Torus::new(5, 5);
        let g = TorusGrid::new(t, |_| true);
        let s = astar(&g, 7, 7, 10).unwrap();
        assert_eq!((s.path, s.cost, s.expanded), (vec![7], 0.0, 0));
    }

    #[test]
    fn unreachable_and_over_the_limit_give_no_path() {
        let t = Torus::new(5, 5);
        // The goal (2, 2) is walled in on all four sides.
        let walls = [
            Pos::new(2, 1),
            Pos::new(2, 3),
            Pos::new(1, 2),
            Pos::new(3, 2),
        ];
        let g = TorusGrid::new(t, |p| !walls.contains(&p));
        assert!(astar(
            &g,
            t.index(Pos::new(0, 0)),
            t.index(Pos::new(2, 2)),
            usize::MAX
        )
        .is_none());
        let open = TorusGrid::new(t, |_| true);
        assert!(astar(&open, t.index(Pos::new(0, 0)), t.index(Pos::new(2, 2)), 1).is_none());
    }

    #[test]
    fn ties_follow_the_stated_order_and_nothing_is_expanded_twice() {
        // On an open 9x9 torus from (4, 4) to (6, 6), every shortest path
        // has 4 steps; the stated order (f, then h, then push order N, S, E,
        // W) picks the path that goes south first, then east.
        let t = Torus::new(9, 9);
        let g = TorusGrid::new(t, |_| true);
        let s = astar(
            &g,
            t.index(Pos::new(4, 4)),
            t.index(Pos::new(6, 6)),
            usize::MAX,
        )
        .unwrap();
        let path: Vec<Pos> = s.path.iter().map(|&i| t.pos(i)).collect();
        assert_eq!(path.len(), 5);
        assert_eq!(
            path[1],
            Pos::new(4, 5),
            "south first (pushed before east at equal f and h)"
        );
        assert!(s.expanded <= 81, "each site at most once");
    }
}
