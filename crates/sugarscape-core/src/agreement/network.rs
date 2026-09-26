//! Who can meet whom: a torus lattice (DNAW §3, AD §3.1), a Watts–Strogatz
//! small world grown from a ring or the torus (AD §3.2), or a Barabási–Albert
//! network (Weisbuch). Built once, from the seed, before any opinion is drawn.

use rand::Rng;

use super::config::{AgreementConfig, Network, Substrate};
use crate::opinions::Neighborhood;
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
    pub fn new(c: &AgreementConfig, rng: &mut SimRng) -> Self {
        let n = c.population();
        let adj = match c.network {
            Network::All => return Graph::default(),
            Network::Lattice => {
                let r = 1;
                torus(c, r, c.lattice.neighborhood == Neighborhood::VonNeumann)
            }
            Network::SmallWorld => {
                let mut adj = match c.small_world.substrate {
                    Substrate::Ring => ring(n, c.small_world.degree as usize),
                    Substrate::Grid => torus(c, c.grid_radius().expect("validated"), false),
                };
                rewire(&mut adj, c.small_world.rewire, rng);
                adj
            }
            Network::ScaleFree => barabasi_albert(n, c.scale_free.links as usize, rng),
        };
        Graph::from_lists(adj)
    }

    fn from_lists(adj: Vec<Vec<u32>>) -> Self {
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

/// The torus's neighbors within Moore radius `r` (only the four nearest
/// with `von_neumann`), in row-major order of offset.
fn torus(c: &AgreementConfig, r: u32, von_neumann: bool) -> Vec<Vec<u32>> {
    let (w, h) = (c.lattice.width as i64, c.lattice.height as i64);
    let r = r as i64;
    let mut adj = vec![Vec::new(); (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let a = (y * w + x) as usize;
            for dy in -r..=r {
                for dx in -r..=r {
                    if (dx, dy) == (0, 0) || (von_neumann && dx.abs() + dy.abs() != 1) {
                        continue;
                    }
                    let b = (y + dy).rem_euclid(h) * w + (x + dx).rem_euclid(w);
                    adj[a].push(b as u32);
                }
            }
        }
    }
    adj
}

/// A ring: each agent linked to the k/2 nearest on each side.
fn ring(n: usize, k: usize) -> Vec<Vec<u32>> {
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
fn rewire(adj: &mut [Vec<u32>], p: f64, rng: &mut SimRng) {
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
fn barabasi_albert(n: usize, m: usize, rng: &mut SimRng) -> Vec<Vec<u32>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agreement::config::{LatticeConfig, ScaleFreeConfig, SmallWorldConfig};
    use crate::rng;

    fn graph(edit: impl FnOnce(&mut AgreementConfig)) -> (AgreementConfig, Graph) {
        let mut c = AgreementConfig::default();
        edit(&mut c);
        c.validate().unwrap();
        let g = Graph::new(&c, &mut rng::seeded(1));
        (c, g)
    }

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
    fn anyone_meets_anyone_without_a_graph() {
        let (_, g) = graph(|_| {});
        assert!(g.is_complete() && g.edges().is_empty());
    }

    #[test]
    fn lattices_wrap_with_four_or_eight_neighbors() {
        let (c, g) = graph(|c| c.network = Network::Lattice);
        assert_eq!(c.population(), 841);
        assert_eq!(g.of(0), [812, 28, 1, 29], "von Neumann, wrapped");
        assert_eq!(g.edges().len(), 841 * 2);
        simple(&g, 841);
        let (_, m) = graph(|c| {
            c.network = Network::Lattice;
            c.lattice.neighborhood = Neighborhood::Moore;
        });
        assert!((0..841).all(|i| m.of(i).len() == 8));
        simple(&m, 841);
    }

    #[test]
    fn rings_rewire_into_small_worlds() {
        let ring = |p| {
            graph(|c| {
                c.agents = 100;
                c.network = Network::SmallWorld;
                c.small_world = SmallWorldConfig {
                    degree: 6,
                    rewire: p,
                    ..SmallWorldConfig::default()
                };
            })
            .1
        };
        let regular = ring(0.0);
        assert_eq!(regular.of(0), [1, 2, 3, 97, 98, 99]);
        assert_eq!(regular.edges().len(), 300);
        let small = ring(0.3);
        assert_eq!(
            small.edges().len(),
            300,
            "rewiring keeps the number of links"
        );
        simple(&small, 100);
        assert_ne!(small, regular);
        let random = ring(1.0);
        simple(&random, 100);
    }

    #[test]
    fn a_grid_substrate_uses_a_wider_moore_neighborhood() {
        let (c, g) = graph(|c| {
            c.network = Network::SmallWorld;
            c.lattice = LatticeConfig {
                width: 10,
                height: 10,
                ..LatticeConfig::default()
            };
            c.small_world = SmallWorldConfig {
                substrate: Substrate::Grid,
                degree: 24,
                rewire: 0.0,
            };
        });
        assert_eq!(c.population(), 100);
        assert!((0..100).all(|i| g.of(i).len() == 24));
        simple(&g, 100);
    }

    #[test]
    fn scale_free_networks_grow_by_preference() {
        let (_, g) = graph(|c| {
            c.agents = 900;
            c.network = Network::ScaleFree;
        });
        simple(&g, 900);
        assert_eq!(
            g.edges().len(),
            3 + 897 * 2,
            "a triangle, then two links each"
        );
        let max = (0..900).map(|i| g.of(i).len()).max().unwrap();
        assert!(max > 30, "hubs: the largest degree is {max}");
        assert!((3..900).all(|i| g.of(i).len() >= 2));
        let (_, four) = graph(|c| {
            c.agents = 100;
            c.network = Network::ScaleFree;
            c.scale_free = ScaleFreeConfig { links: 4 };
        });
        assert_eq!(four.edges().len(), 10 + 95 * 4, "K5, then four links each");
        simple(&four, 100);
    }

    #[test]
    fn graphs_are_deterministic_from_the_seed() {
        let c = AgreementConfig {
            network: Network::ScaleFree,
            ..AgreementConfig::default()
        };
        let a = Graph::new(&c, &mut rng::seeded(4));
        let b = Graph::new(&c, &mut rng::seeded(4));
        let d = Graph::new(&c, &mut rng::seeded(5));
        assert_eq!(a, b);
        assert_ne!(a, d);
    }
}
