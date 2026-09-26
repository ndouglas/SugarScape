//! The fixed social structures, drawn at reset: the torus (2DK), fixed
//! random neighbors (FRN) and the symmetric random regular graph (FRNE),
//! and the fan-out CRA measure in Table A1.

use std::collections::VecDeque;

use rand::seq::SliceRandom;
use rand::Rng;

use super::config::{square_side, Structure, StructureConfig};
use crate::rng::SimRng;

/// Each agent's fixed chosen partners (empty under RWR), and on the torus
/// where each agent sits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub chosen: Vec<Vec<u32>>,
    /// Torus only: `site[agent]` and `agent_at[site]`, row-major.
    pub site: Vec<u32>,
    pub agent_at: Vec<u32>,
}

impl Graph {
    pub fn new(c: &StructureConfig, rng: &mut SimRng) -> Self {
        let n = c.agents as usize;
        match c.structure {
            Structure::Rwr => Graph::default(),
            Structure::Frn => Graph {
                chosen: (0..n)
                    .map(|a| (0..c.partners).map(|_| other(rng, n, a)).collect())
                    .collect(),
                ..Graph::default()
            },
            Structure::Torus => torus(n, rng),
            Structure::Frne => Graph {
                chosen: regular(n, c.partners as usize, rng),
                ..Graph::default()
            },
        }
    }
}

/// A uniform agent other than `a`.
pub fn other(rng: &mut SimRng, n: usize, a: usize) -> u32 {
    let b = rng.gen_range(0..n as u32 - 1) as usize;
    (if b >= a { b + 1 } else { b }) as u32
}

/// Agents at random on the √n × √n torus; each chooses its NEWS neighbors
/// (north, east, west, south), so every pair plays twice.
fn torus(n: usize, rng: &mut SimRng) -> Graph {
    let s = square_side(n as u32).expect("validated: a square") as i32;
    let mut agent_at: Vec<u32> = (0..n as u32).collect();
    agent_at.shuffle(rng);
    let mut site = vec![0; n];
    for (k, &a) in agent_at.iter().enumerate() {
        site[a as usize] = k as u32;
    }
    let chosen = (0..n)
        .map(|a| {
            let k = site[a] as i32;
            let (x, y) = (k % s, k / s);
            [(0, -1), (1, 0), (-1, 0), (0, 1)]
                .iter()
                .map(|&(dx, dy)| {
                    agent_at[((y + dy).rem_euclid(s) * s + (x + dx).rem_euclid(s)) as usize]
                })
                .collect()
        })
        .collect();
    Graph {
        chosen,
        site,
        agent_at,
    }
}

/// A random `k`-regular simple symmetric graph (k even): a ring lattice
/// (each agent linked to the k/2 nearest on each side) mixed by CRA's
/// procedure — each agent, n times, swaps one of its neighbors with a random
/// other agent's neighbor (a double-edge swap), kept only if no agent gains
/// itself or a repeated neighbor.
fn regular(n: usize, k: usize, rng: &mut SimRng) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = (0..n)
        .map(|a| {
            (1..=k / 2)
                .flat_map(|d| [(a + d) % n, (a + n - d) % n])
                .map(|b| b as u32)
                .collect()
        })
        .collect();
    for a in 0..n {
        for _ in 0..n {
            let c = other(rng, n, a) as usize;
            let bi = rng.gen_range(0..k as u32) as usize;
            let di = rng.gen_range(0..k as u32) as usize;
            let (b, d) = (adj[a][bi] as usize, adj[c][di] as usize);
            // a–b, c–d → a–d, c–b
            if d == a || b == c || b == d {
                continue;
            }
            if adj[a].contains(&(d as u32)) || adj[c].contains(&(b as u32)) {
                continue;
            }
            let replace = |adj: &mut Vec<Vec<u32>>, x: usize, from: usize, to: usize| {
                let i = adj[x].iter().position(|&v| v as usize == from).unwrap();
                adj[x][i] = to as u32;
            };
            replace(&mut adj, a, b, d);
            replace(&mut adj, b, a, c);
            replace(&mut adj, c, d, b);
            replace(&mut adj, d, c, a);
        }
    }
    adj
}

/// The number of agents exactly d links away (d = 1, 2, …), averaged over
/// all agents, treating links as undirected (Table A1's distribution
/// sequence).
pub fn fanout(chosen: &[Vec<u32>], max_d: usize) -> Vec<f64> {
    let n = chosen.len();
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (a, list) in chosen.iter().enumerate() {
        for &b in list {
            for (x, y) in [(a, b as usize), (b as usize, a)] {
                if !adj[x].contains(&(y as u32)) {
                    adj[x].push(y as u32);
                }
            }
        }
    }
    let mut totals = vec![0usize; max_d];
    for s in 0..n {
        let mut dist = vec![usize::MAX; n];
        dist[s] = 0;
        let mut queue = VecDeque::from([s]);
        while let Some(u) = queue.pop_front() {
            if dist[u] >= max_d {
                continue;
            }
            for &v in &adj[u] {
                let v = v as usize;
                if dist[v] == usize::MAX {
                    dist[v] = dist[u] + 1;
                    totals[dist[v] - 1] += 1;
                    queue.push_back(v);
                }
            }
        }
    }
    totals.iter().map(|&t| t as f64 / n as f64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    fn config(structure: Structure) -> StructureConfig {
        StructureConfig {
            structure,
            ..StructureConfig::default()
        }
    }

    #[test]
    fn the_torus_wraps_and_neighbors_are_mutual() {
        let g = Graph::new(&config(Structure::Torus), &mut rng::seeded(1));
        let at = |x: i32, y: i32| g.agent_at[(y.rem_euclid(16) * 16 + x.rem_euclid(16)) as usize];
        let corner = at(0, 0) as usize;
        assert_eq!(g.chosen[corner], [at(0, -1), at(1, 0), at(-1, 0), at(0, 1)]);
        for (a, list) in g.chosen.iter().enumerate() {
            for &b in list {
                assert!(g.chosen[b as usize].contains(&(a as u32)));
            }
        }
    }

    #[test]
    fn frn_draws_others_with_replacement() {
        let g = Graph::new(&config(Structure::Frn), &mut rng::seeded(2));
        assert!(g.chosen.iter().all(|l| l.len() == 4));
        assert!(g
            .chosen
            .iter()
            .enumerate()
            .all(|(a, l)| !l.contains(&(a as u32))));
        let repeats = g
            .chosen
            .iter()
            .filter(|l| (1..4).any(|i| l[..i].contains(&l[i])))
            .count();
        assert!(
            repeats > 0,
            "with replacement, some agent picks a partner twice"
        );
    }

    #[test]
    fn frne_is_regular_simple_and_symmetric() {
        let g = Graph::new(&config(Structure::Frne), &mut rng::seeded(3));
        for (a, list) in g.chosen.iter().enumerate() {
            assert_eq!(list.len(), 4);
            assert!(!list.contains(&(a as u32)));
            for (i, &b) in list.iter().enumerate() {
                assert!(!list[..i].contains(&b), "no repeated neighbor");
                assert!(g.chosen[b as usize].contains(&(a as u32)), "symmetric");
            }
        }
        // Mixed: few links are left from the ring lattice.
        let ring = g
            .chosen
            .iter()
            .enumerate()
            .flat_map(|(a, l)| l.iter().map(move |&b| (a, b as usize)))
            .filter(|&(a, b)| (a + 256 - b) % 256 <= 2 || (b + 256 - a) % 256 <= 2)
            .count();
        assert!(ring < 20, "{ring} ring links survive");
    }

    #[test]
    fn fanout_matches_the_papers_table_a1_shape() {
        let torus = Graph::new(&config(Structure::Torus), &mut rng::seeded(4));
        assert_eq!(
            fanout(&torus.chosen, 3),
            [4.0, 8.0, 12.0],
            "N(d) = 4d on the torus"
        );
        let frne = Graph::new(&config(Structure::Frne), &mut rng::seeded(4));
        let f = fanout(&frne.chosen, 6);
        assert_eq!(f[0], 4.0);
        assert!(
            (f[1] - 11.74).abs() < 0.5 && (f[2] - 32.4).abs() < 2.0,
            "{f:?}"
        );
    }
}
