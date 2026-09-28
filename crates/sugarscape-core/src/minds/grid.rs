//! The grids Minds 2's A* searches: the Sugarscape's 4-way torus, and the
//! flat 8-way octile maps of Sturtevant's benchmarks.

use crate::geometry::{Pos, Torus};
use crate::minds::astar::Graph;

/// The 4-way torus; `passable` decides which sites a path may enter.
/// Neighbors come in `geometry::DIRECTIONS` order (N, S, E, W); each step
/// costs 1; the heuristic is torus Manhattan distance.
pub struct TorusGrid<F: Fn(Pos) -> bool> {
    torus: Torus,
    passable: F,
}

impl<F: Fn(Pos) -> bool> TorusGrid<F> {
    pub fn new(torus: Torus, passable: F) -> Self {
        Self { torus, passable }
    }
}

impl<F: Fn(Pos) -> bool> Graph for TorusGrid<F> {
    fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>) {
        for q in self.torus.neighbors(self.torus.pos(n)) {
            if (self.passable)(q) {
                out.push((self.torus.index(q), 1.0));
            }
        }
    }

    fn heuristic(&self, n: usize, goal: usize) -> f64 {
        let (a, b) = (self.torus.pos(n), self.torus.pos(goal));
        let d = |p: u32, q: u32, len: u32| {
            let d = p.abs_diff(q);
            d.min(len - d)
        };
        f64::from(d(a.x, b.x, self.torus.width) + d(a.y, b.y, self.torus.height))
    }
}

/// A Moving AI octile map: 8-way, flat (not a torus), diagonals cost sqrt(2)
/// and may not cut a corner (both orthogonal neighbors must be passable).
/// `.`, `G` and `S` are passable.
pub struct OctileMap {
    width: usize,
    height: usize,
    open: Vec<bool>,
}

impl OctileMap {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        let mut header = |key: &str| -> Result<String, String> {
            let line = lines.next().ok_or("truncated header")?;
            line.strip_prefix(key)
                .map(|v| v.trim().to_string())
                .ok_or(format!("expected {key}"))
        };
        header("type")?;
        let height: usize = header("height")?.parse().map_err(|_| "bad height")?;
        let width: usize = header("width")?.parse().map_err(|_| "bad width")?;
        header("map")?;
        let mut open = Vec::with_capacity(width * height);
        for row in lines.by_ref().take(height) {
            let row: Vec<char> = row.chars().collect();
            if row.len() < width {
                return Err("short row".into());
            }
            open.extend(row[..width].iter().map(|c| matches!(c, '.' | 'G' | 'S')));
        }
        if open.len() != width * height {
            return Err("too few rows".into());
        }
        Ok(Self {
            width,
            height,
            open,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn passable(&self, x: usize, y: usize) -> bool {
        self.open[self.index(x, y)]
    }
}

impl Graph for OctileMap {
    fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>) {
        let (x, y) = ((n % self.width) as i64, (n / self.width) as i64);
        let ok = |x: i64, y: i64| {
            x >= 0
                && y >= 0
                && (x as usize) < self.width
                && (y as usize) < self.height
                && self.passable(x as usize, y as usize)
        };
        for (dx, dy) in [
            (0, -1),
            (0, 1),
            (1, 0),
            (-1, 0),
            (1, -1),
            (1, 1),
            (-1, 1),
            (-1, -1),
        ] {
            let (nx, ny) = (x + dx, y + dy);
            if !ok(nx, ny) {
                continue;
            }
            let diagonal = dx != 0 && dy != 0;
            if diagonal && !(ok(x + dx, y) && ok(x, y + dy)) {
                continue; // no corner cutting
            }
            let cost = if diagonal {
                std::f64::consts::SQRT_2
            } else {
                1.0
            };
            out.push((self.index(nx as usize, ny as usize), cost));
        }
    }

    fn heuristic(&self, n: usize, goal: usize) -> f64 {
        let (dx, dy) = (
            (n % self.width).abs_diff(goal % self.width),
            (n / self.width).abs_diff(goal / self.width),
        );
        let (lo, hi) = (dx.min(dy) as f64, dx.max(dy) as f64);
        hi + (std::f64::consts::SQRT_2 - 1.0) * lo
    }
}
