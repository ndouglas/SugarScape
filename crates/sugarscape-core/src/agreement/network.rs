//! Who can meet whom: a torus lattice (DNAW §3, AD §3.1), a Watts–Strogatz
//! small world grown from a ring or the torus (AD §3.2), or a Barabási–Albert
//! network (Weisbuch). Built once, from the seed, before any opinion is drawn.

use super::config::{AgreementConfig, Network, Substrate};
use crate::graph::{barabasi_albert, rewire, ring, Graph};
use crate::opinions::Neighborhood;
use crate::rng::SimRng;

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
