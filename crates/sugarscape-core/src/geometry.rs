//! Lattice positions and wraparound (toroidal) geometry.

use serde::{Deserialize, Serialize};

/// A lattice position. `y = 0` is the northern (top) row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

impl Pos {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// The four principal lattice directions, in order: north, south, east, west.
pub const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (0, 1), (1, 0), (-1, 0)];

/// Wraparound lattice geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Torus {
    pub width: u32,
    pub height: u32,
}

impl Torus {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "torus must be non-empty");
        Self { width, height }
    }

    pub fn len(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn index(&self, p: Pos) -> usize {
        (p.y * self.width + p.x) as usize
    }

    pub fn pos(&self, i: usize) -> Pos {
        Pos::new(i as u32 % self.width, i as u32 / self.width)
    }

    pub fn offset(&self, p: Pos, dx: i32, dy: i32) -> Pos {
        let x = (i64::from(p.x) + i64::from(dx)).rem_euclid(i64::from(self.width));
        let y = (i64::from(p.y) + i64::from(dy)).rem_euclid(i64::from(self.height));
        Pos::new(x as u32, y as u32)
    }

    /// Von Neumann neighbors in `DIRECTIONS` order.
    pub fn neighbors(&self, p: Pos) -> [Pos; 4] {
        DIRECTIONS.map(|(dx, dy)| self.offset(p, dx, dy))
    }

    /// Every site visible from `p` with `vision`, with its distance, nearest
    /// first. Excludes `p`; on small tori where lines of sight wrap onto the
    /// same site, each site appears once at its shortest distance.
    pub fn sight(&self, p: Pos, vision: u32) -> Vec<(Pos, u32)> {
        let mut seen = Vec::with_capacity(4 * vision as usize);
        for (dx, dy) in DIRECTIONS {
            for d in 1..=vision as i32 {
                seen.push((self.offset(p, dx * d, dy * d), d as u32));
            }
        }
        seen.sort_by_key(|&(_, d)| d);
        let mut out: Vec<(Pos, u32)> = Vec::with_capacity(seen.len());
        for (q, d) in seen {
            if q != p && !out.iter().any(|&(r, _)| r == q) {
                out.push((q, d));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_and_pos_round_trip() {
        let t = Torus::new(7, 5);
        for i in 0..t.len() {
            assert_eq!(t.index(t.pos(i)), i);
        }
        assert_eq!(t.pos(8), Pos::new(1, 1));
    }

    #[test]
    fn offset_wraps_in_both_axes() {
        let t = Torus::new(10, 8);
        assert_eq!(t.offset(Pos::new(0, 0), -1, -1), Pos::new(9, 7));
        assert_eq!(t.offset(Pos::new(9, 7), 1, 1), Pos::new(0, 0));
        assert_eq!(t.offset(Pos::new(3, 3), 25, -17), Pos::new(8, 2));
    }

    #[test]
    fn neighbors_are_north_south_east_west() {
        let t = Torus::new(10, 10);
        assert_eq!(
            t.neighbors(Pos::new(5, 5)),
            [
                Pos::new(5, 4),
                Pos::new(5, 6),
                Pos::new(6, 5),
                Pos::new(4, 5)
            ]
        );
    }

    #[test]
    fn sight_covers_four_directions_without_diagonals() {
        let t = Torus::new(11, 11);
        let seen = t.sight(Pos::new(5, 5), 2);
        assert_eq!(seen.len(), 8);
        assert!(seen.contains(&(Pos::new(5, 3), 2)));
        assert!(seen.contains(&(Pos::new(7, 5), 2)));
        assert!(seen.contains(&(Pos::new(4, 5), 1)));
        assert!(!seen.iter().any(|&(p, _)| p == Pos::new(6, 6)));
        assert!(!seen.iter().any(|&(p, _)| p == Pos::new(5, 5)));
    }

    #[test]
    fn sight_is_sorted_by_distance_and_deduplicated_on_small_tori() {
        let t = Torus::new(4, 4);
        let seen = t.sight(Pos::new(0, 0), 3);
        let mut positions: Vec<Pos> = seen.iter().map(|&(p, _)| p).collect();
        positions.sort();
        positions.dedup();
        assert_eq!(positions.len(), seen.len(), "no duplicates");
        assert!(seen.windows(2).all(|w| w[0].1 <= w[1].1), "nearest first");
        // (0,3) is one step north (wrapping), not three steps south.
        assert!(seen.contains(&(Pos::new(0, 3), 1)));
    }
}
