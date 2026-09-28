//! Who can meet whom: a graph as compressed neighbor lists, and the builders
//! the network models share (a ring, Watts–Strogatz rewiring, a ring with
//! added shortcuts, a random graph and Barabási–Albert growth). Every
//! builder draws from the simulation's stream in a fixed order.

use rand::Rng;

use crate::rng::SimRng;

/// Every agent's neighbors (in the order links were made) and the links
/// themselves (lower index first). Empty when anyone meets anyone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    start: Vec<u32>,
    list: Vec<u32>,
    edges: Vec<(u32, u32)>,
}

impl Graph {
    pub fn from_lists(adj: Vec<Vec<u32>>) -> Self {
        let mut start = Vec::with_capacity(adj.len() + 1);
        let mut list = Vec::new();
        let mut edges = Vec::new();
        for (a, ns) in adj.iter().enumerate() {
            start.push(list.len() as u32);
            list.extend_from_slice(ns);
            edges.extend(
                ns.iter()
                    .filter(|&&b| a < b as usize)
                    .map(|&b| (a as u32, b)),
            );
        }
        start.push(list.len() as u32);
        Graph { start, list, edges }
    }

    /// Whether anyone meets anyone (no graph).
    pub fn is_complete(&self) -> bool {
        self.start.is_empty()
    }

    pub fn of(&self, i: usize) -> &[u32] {
        &self.list[self.start[i] as usize..self.start[i + 1] as usize]
    }

    pub fn edges(&self) -> &[(u32, u32)] {
        &self.edges
    }
}

/// A ring: each agent linked to the k/2 nearest on each side.
pub fn ring(n: usize, k: usize) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); n];
    for i in 0..n {
        for j in 1..=k / 2 {
            let b = (i + j) % n;
            adj[i].push(b as u32);
            adj[b].push(i as u32);
        }
    }
    adj
}

/// Watts–Strogatz: each link (a, b), a < b, in order of a then of a's list,
/// is with probability p replaced by (a, c) for a uniform c that is not a
/// and not already a's neighbor.
pub fn rewire(adj: &mut [Vec<u32>], p: f64, rng: &mut SimRng) {
    let n = adj.len() as u32;
    let links: Vec<(u32, u32)> = adj
        .iter()
        .enumerate()
        .flat_map(|(a, ns)| {
            ns.iter()
                .filter(move |&&b| (a as u32) < b)
                .map(move |&b| (a as u32, b))
        })
        .collect();
    for (a, b) in links {
        if rng.gen::<f64>() >= p || adj[a as usize].len() as u32 >= n - 1 {
            continue;
        }
        let c = loop {
            let c = rng.gen_range(0..n);
            if c != a && !adj[a as usize].contains(&c) {
                break c;
            }
        };
        adj[a as usize].retain(|&x| x != b);
        adj[b as usize].retain(|&x| x != a);
        adj[a as usize].push(c);
        adj[c as usize].push(a);
    }
}

/// Barabási–Albert: a complete graph on max(3, m + 1) agents, then each new
/// agent linked to m distinct earlier ones chosen in proportion to degree.
pub fn barabasi_albert(n: usize, m: usize, rng: &mut SimRng) -> Vec<Vec<u32>> {
    let seed = 3.max(m + 1).min(n);
    let mut adj = vec![Vec::new(); n];
    // Every link's two ends: a uniform pick is a pick by degree.
    let mut ends: Vec<u32> = Vec::new();
    for a in 0..seed {
        for b in a + 1..seed {
            adj[a].push(b as u32);
            adj[b].push(a as u32);
            ends.extend([a as u32, b as u32]);
        }
    }
    for v in seed..n {
        let mut targets: Vec<u32> = Vec::with_capacity(m);
        while targets.len() < m {
            let t = ends[rng.gen_range(0..ends.len() as u32) as usize];
            if !targets.contains(&t) {
                targets.push(t);
            }
        }
        for t in targets {
            adj[v].push(t);
            adj[t as usize].push(v as u32);
            ends.extend([v as u32, t]);
        }
    }
    adj
}

/// A ring plus `count` distinct random links between agents not yet linked
/// (Alfarano and Milaković's small world: shortcuts added, none rewired).
pub fn shortcuts(adj: &mut [Vec<u32>], count: usize, rng: &mut SimRng) {
    let n = adj.len() as u32;
    let room = adj.len() * (adj.len() - 1) / 2 - adj.iter().map(Vec::len).sum::<usize>() / 2;
    let mut added = 0;
    while added < count.min(room) {
        let a = rng.gen_range(0..n);
        let b = rng.gen_range(0..n);
        if a == b || adj[a as usize].contains(&b) {
            continue;
        }
        adj[a as usize].push(b);
        adj[b as usize].push(a);
        added += 1;
    }
}

/// A random graph: each of the n(n − 1)/2 pairs, a then b > a in order,
/// linked with probability `p`.
pub fn gnp(n: usize, p: f64, rng: &mut SimRng) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); n];
    for a in 0..n {
        for b in a + 1..n {
            if rng.gen::<f64>() < p {
                adj[a].push(b as u32);
                adj[b].push(a as u32);
            }
        }
    }
    adj
}

/// A random graph drawn in expected time proportional to its links
/// (Batagelj and Brandes's geometric skipping): each pair (a, b), b < a, in
/// order, is linked with probability `p`, the gaps between links drawn as
/// geometric variates with the portable logarithm.
pub fn gnp_sparse(n: usize, p: f64, rng: &mut SimRng) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); n];
    if p <= 0.0 || n < 2 {
        return adj;
    }
    if p >= 1.0 {
        for a in 0..n {
            for b in 0..a {
                adj[a].push(b as u32);
                adj[b].push(a as u32);
            }
        }
        return adj;
    }
    let lp = crate::portable::ln(1.0 - p);
    if lp >= 0.0 {
        // p so small that 1 − p rounds to 1: no links.
        return adj;
    }
    let (mut v, mut w) = (1usize, -1i64);
    while v < n {
        let r: f64 = rng.gen();
        let skip = (crate::portable::ln(1.0 - r) / lp).floor();
        w += 1 + skip as i64;
        while w >= v as i64 && v < n {
            w -= v as i64;
            v += 1;
        }
        if v < n {
            adj[v].push(w as u32);
            adj[w as usize].push(v as u32);
        }
    }
    adj
}

/// The configuration model: each agent gets `degrees[i]` link ends, the
/// ends are shuffled and paired in order, and self-links and repeated links
/// are dropped.
pub fn configuration(degrees: &[u32], rng: &mut SimRng) -> Vec<Vec<u32>> {
    let n = degrees.len();
    let mut ends: Vec<u32> = Vec::new();
    for (i, &d) in degrees.iter().enumerate() {
        ends.extend(std::iter::repeat_n(i as u32, d as usize));
    }
    for i in (1..ends.len()).rev() {
        let j = rng.gen_range(0..=i as u32) as usize;
        ends.swap(i, j);
    }
    let mut adj = vec![Vec::new(); n];
    for pair in ends.as_chunks::<2>().0 {
        let (a, b) = (pair[0], pair[1]);
        if a != b && !adj[a as usize].contains(&b) {
            adj[a as usize].push(b);
            adj[b as usize].push(a);
        }
    }
    adj
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    fn simple(g: &Graph, n: usize) {
        for i in 0..n {
            let ns = g.of(i);
            assert!(!ns.contains(&(i as u32)), "no loop at {i}");
            let mut sorted = ns.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), ns.len(), "no double link at {i}");
            for &j in ns {
                assert!(g.of(j as usize).contains(&(i as u32)), "{i}–{j} both ways");
            }
        }
    }

    #[test]
    fn shortcuts_add_distinct_links_to_a_ring() {
        let mut adj = ring(100, 10);
        shortcuts(&mut adj, 10, &mut rng::seeded(1));
        let g = Graph::from_lists(adj);
        assert_eq!(g.edges().len(), 500 + 10);
        simple(&g, 100);
        // A nearly full graph takes only what room is left.
        let mut full = ring(5, 2);
        shortcuts(&mut full, 99, &mut rng::seeded(1));
        assert_eq!(Graph::from_lists(full).edges().len(), 10);
    }

    #[test]
    fn random_graphs_link_about_p_of_all_pairs() {
        let g = Graph::from_lists(gnp(400, 0.1, &mut rng::seeded(2)));
        simple(&g, 400);
        let links = g.edges().len() as f64;
        let want = 0.1 * 400.0 * 399.0 / 2.0;
        assert!((links - want).abs() < 4.0 * want.sqrt(), "{links}");
        assert!(Graph::from_lists(gnp(10, 0.0, &mut rng::seeded(2)))
            .edges()
            .is_empty());
        assert_eq!(
            Graph::from_lists(gnp(10, 1.0, &mut rng::seeded(2)))
                .edges()
                .len(),
            45
        );
    }

    #[test]
    fn sparse_random_graphs_link_about_p_of_all_pairs() {
        let g = Graph::from_lists(gnp_sparse(4000, 0.001, &mut rng::seeded(3)));
        simple(&g, 4000);
        let links = g.edges().len() as f64;
        let want = 0.001 * 4000.0 * 3999.0 / 2.0;
        assert!((links - want).abs() < 4.0 * want.sqrt(), "{links}");
        assert!(gnp_sparse(10, 0.0, &mut rng::seeded(3))
            .iter()
            .all(Vec::is_empty));
        let full = Graph::from_lists(gnp_sparse(10, 1.0, &mut rng::seeded(3)));
        assert_eq!(full.edges().len(), 45);
        simple(&full, 10);
    }

    #[test]
    fn a_vanishing_link_probability_gives_no_links_rather_than_a_panic() {
        // 1 − 1e-17 rounds to 1, so ln(1 − p) is 0 and the skip would be −∞.
        assert!(gnp_sparse(20_000, 1e-17, &mut rng::seeded(5))
            .iter()
            .all(Vec::is_empty));
    }

    #[test]
    fn the_configuration_model_keeps_simple_links_near_the_degrees() {
        let degrees: Vec<u32> = (0..1000).map(|i| 1 + i % 5).collect();
        let g = Graph::from_lists(configuration(&degrees, &mut rng::seeded(4)));
        simple(&g, 1000);
        let ends: u32 = degrees.iter().sum();
        let kept = 2 * g.edges().len() as u32;
        assert!(kept <= ends && kept + 20 >= ends, "{kept} of {ends}");
    }
}
