//! Where the spatial games' players sit and whom each plays: square and
//! cubic lattices (Moore or von Neumann, fixed or periodic edges) and NBM94's
//! random arrays (a fraction of a grid's cells, neighbors within a radius).

use rand::seq::SliceRandom;

use super::config::{Boundary, Lattice, Neighborhood, SpatialConfig};
use crate::rng::SimRng;

/// No player on a cell (random arrays).
pub const EMPTY: u32 = u32::MAX;

/// Players, their cells and their neighbor lists.
#[derive(Clone, Debug)]
pub struct Geometry {
    /// The grid's sides (x, y, z); z = 1 except in a cube.
    pub dims: (u32, u32, u32),
    /// Each player's cell, `x + y·w + z·w·h`, in player order (ascending).
    pub cell: Vec<u32>,
    /// Each cell's player, or `EMPTY`.
    pub player: Vec<u32>,
    /// Player i's neighbors are `neighbors[start[i]..start[i + 1]]`, in
    /// ascending order of offset (z, then y, then x).
    start: Vec<u32>,
    neighbors: Vec<u32>,
}

impl Geometry {
    /// Builds the lattice of `c`; a random array draws its cells from `rng`.
    pub fn new(c: &SpatialConfig, rng: &mut SimRng) -> Self {
        let dims = c.dims();
        let cells = (dims.0 * dims.1 * dims.2) as usize;
        let cell: Vec<u32> = match c.lattice {
            Lattice::Square | Lattice::Cube => (0..cells as u32).collect(),
            Lattice::Random => {
                let k = (c.occupancy * cells as f64).round() as usize;
                let mut all: Vec<u32> = (0..cells as u32).collect();
                all.shuffle(rng);
                let mut chosen = all[..k.max(1)].to_vec();
                chosen.sort_unstable();
                chosen
            }
        };
        let mut player = vec![EMPTY; cells];
        for (i, &x) in cell.iter().enumerate() {
            player[x as usize] = i as u32;
        }
        let offsets = offsets(c);
        let periodic = c.boundary == Boundary::Periodic;
        let (w, h, d) = (dims.0 as i64, dims.1 as i64, dims.2 as i64);
        let mut start = Vec::with_capacity(cell.len() + 1);
        let mut neighbors = Vec::new();
        for &x in &cell {
            start.push(neighbors.len() as u32);
            let (px, py, pz) = unpack(x, dims);
            for &(dx, dy, dz) in &offsets {
                let (mut nx, mut ny, mut nz) = (px as i64 + dx, py as i64 + dy, pz as i64 + dz);
                if periodic {
                    nx = nx.rem_euclid(w);
                    ny = ny.rem_euclid(h);
                    nz = nz.rem_euclid(d);
                } else if nx < 0 || ny < 0 || nz < 0 || nx >= w || ny >= h || nz >= d {
                    continue;
                }
                let j = player[(nx + ny * w + nz * w * h) as usize];
                if j != EMPTY {
                    neighbors.push(j);
                }
            }
        }
        start.push(neighbors.len() as u32);
        Geometry {
            dims,
            cell,
            player,
            start,
            neighbors,
        }
    }

    pub fn len(&self) -> usize {
        self.cell.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cell.is_empty()
    }

    /// Player `i`'s neighbors (not itself).
    pub fn neighbors(&self, i: usize) -> &[u32] {
        &self.neighbors[self.start[i] as usize..self.start[i + 1] as usize]
    }

    /// Player `i`'s cell as (x, y, z).
    pub fn xyz(&self, i: usize) -> (u32, u32, u32) {
        unpack(self.cell[i], self.dims)
    }

    /// The player on cell (x, y, z), if any.
    pub fn at(&self, x: u32, y: u32, z: u32) -> Option<usize> {
        let (w, h, d) = self.dims;
        if x >= w || y >= h || z >= d {
            return None;
        }
        let p = self.player[(x + y * w + z * w * h) as usize];
        (p != EMPTY).then_some(p as usize)
    }

    /// The player nearest the grid's center ((n − 1)/2 rounded down on each
    /// axis): the lowest-numbered at the smallest squared distance.
    pub fn central(&self) -> usize {
        let (w, h, d) = self.dims;
        let c = ((w - 1) / 2, (h - 1) / 2, (d - 1) / 2);
        let dist = |i: usize| {
            let (x, y, z) = self.xyz(i);
            let (dx, dy, dz) = (
                x as i64 - c.0 as i64,
                y as i64 - c.1 as i64,
                z as i64 - c.2 as i64,
            );
            dx * dx + dy * dy + dz * dz
        };
        (0..self.len()).min_by_key(|&i| (dist(i), i)).unwrap_or(0)
    }
}

fn unpack(cell: u32, (w, h, _): (u32, u32, u32)) -> (u32, u32, u32) {
    (cell % w, (cell / w) % h, cell / (w * h))
}

/// The neighbor offsets of `c`'s lattice (excluding (0, 0, 0)), ascending by
/// (dz, dy, dx).
pub fn offsets(c: &SpatialConfig) -> Vec<(i64, i64, i64)> {
    let mut out = Vec::new();
    match c.lattice {
        Lattice::Random => {
            let r = c.radius.floor() as i64;
            let r2 = c.radius * c.radius;
            for dy in -r..=r {
                for dx in -r..=r {
                    if (dx, dy) != (0, 0) && ((dx * dx + dy * dy) as f64) <= r2 {
                        out.push((dx, dy, 0));
                    }
                }
            }
        }
        Lattice::Square | Lattice::Cube => {
            let zs: &[i64] = if c.lattice == Lattice::Cube {
                &[-1, 0, 1]
            } else {
                &[0]
            };
            for &dz in zs {
                for dy in -1..=1i64 {
                    for dx in -1..=1i64 {
                        let manhattan = dx.abs() + dy.abs() + dz.abs();
                        let keep = match c.neighborhood {
                            Neighborhood::Moore => manhattan > 0,
                            Neighborhood::VonNeumann => manhattan == 1,
                        };
                        if keep {
                            out.push((dx, dy, dz));
                        }
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    fn geo(edit: impl FnOnce(&mut SpatialConfig)) -> Geometry {
        let mut c = SpatialConfig {
            width: 5,
            height: 4,
            ..Default::default()
        };
        edit(&mut c);
        Geometry::new(&c, &mut rng::seeded(1))
    }

    #[test]
    fn fixed_edges_give_boundary_players_fewer_neighbors() {
        let g = geo(|_| {});
        assert_eq!(g.len(), 20);
        assert_eq!(g.neighbors(g.at(0, 0, 0).unwrap()).len(), 3, "a corner");
        assert_eq!(g.neighbors(g.at(2, 0, 0).unwrap()).len(), 5, "an edge");
        assert_eq!(g.neighbors(g.at(2, 2, 0).unwrap()).len(), 8, "inside");
        let vn = geo(|c| c.neighborhood = Neighborhood::VonNeumann);
        assert_eq!(vn.neighbors(0).len(), 2);
        assert_eq!(vn.neighbors(vn.at(2, 2, 0).unwrap()).len(), 4);
    }

    #[test]
    fn periodic_edges_wrap() {
        let g = geo(|c| c.boundary = Boundary::Periodic);
        let corner = g.neighbors(0);
        assert_eq!(corner.len(), 8);
        assert!(corner.contains(&(g.at(4, 3, 0).unwrap() as u32)));
        assert!(corner.contains(&(g.at(4, 0, 0).unwrap() as u32)));
    }

    #[test]
    fn cubes_have_26_or_6_neighbors() {
        let g = geo(|c| {
            c.lattice = Lattice::Cube;
            c.width = 4;
        });
        assert_eq!(g.len(), 64);
        assert_eq!(g.neighbors(g.at(1, 1, 1).unwrap()).len(), 26);
        assert_eq!(g.neighbors(0).len(), 7, "a corner of the cube");
        let p = geo(|c| {
            c.lattice = Lattice::Cube;
            c.width = 4;
            c.boundary = Boundary::Periodic;
            c.neighborhood = Neighborhood::VonNeumann;
        });
        assert_eq!(p.neighbors(0).len(), 6);
        assert!(p.neighbors(0).contains(&(p.at(0, 0, 3).unwrap() as u32)));
    }

    #[test]
    fn random_arrays_hold_a_fraction_and_meet_within_the_radius() {
        let mut c = SpatialConfig {
            lattice: Lattice::Random,
            width: 40,
            height: 40,
            occupancy: 0.25,
            radius: 3.0,
            ..Default::default()
        };
        let g = Geometry::new(&c, &mut rng::seeded(4));
        assert_eq!(g.len(), 400);
        assert!(g.cell.windows(2).all(|w| w[0] < w[1]), "ascending");
        for i in 0..g.len() {
            let (x, y, _) = g.xyz(i);
            for &j in g.neighbors(i) {
                let (a, b, _) = g.xyz(j as usize);
                let (dx, dy) = (a as f64 - x as f64, b as f64 - y as f64);
                assert!(dx * dx + dy * dy <= 9.0);
            }
        }
        c.occupancy = 1.0;
        let full = Geometry::new(&c, &mut rng::seeded(4));
        assert_eq!(full.neighbors(full.at(20, 20, 0).unwrap()).len(), 28);
    }

    #[test]
    fn the_central_player_is_at_the_middle() {
        let g = geo(|c| {
            c.width = 99;
            c.height = 99;
        });
        assert_eq!(g.xyz(g.central()), (49, 49, 0));
        let even = geo(|c| {
            c.width = 4;
            c.height = 4;
        });
        assert_eq!(even.xyz(even.central()), (1, 1, 0));
    }
}
