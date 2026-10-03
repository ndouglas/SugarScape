//! Fixed grid cells and generation-aware sovereign states.
use super::{GeosimConfig, StateId};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub id: usize,
    pub owner: StateId,
    pub last_threshold: f64,
    pub next_generation: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub id: StateId,
    pub capacity: Option<f64>,
    pub threshold: f64,
    pub alert: bool,
    pub campaign: Option<StateId>,
    pub previous_damage: f64,
    pub newly_independent: bool,
    pub extracted_yield: f64,
    pub recurrence_residual: f64,
}
pub fn adjacent(c: &GeosimConfig, id: usize) -> Vec<usize> {
    let w = c.width as usize;
    let h = c.height as usize;
    let x = id % w;
    let y = id / w;
    let torus = c.topology == super::Topology::Torus;
    let mut v = Vec::new();
    if x > 0 {
        v.push(id - 1);
    } else if torus {
        v.push(y * w + w - 1);
    }
    if x + 1 < w {
        v.push(id + 1);
    } else if torus {
        v.push(y * w);
    }
    if y > 0 {
        v.push(id - w);
    } else if torus {
        v.push((h - 1) * w + x);
    }
    if y + 1 < h {
        v.push(id + w);
    } else if torus {
        v.push(x);
    }
    v.sort_unstable();
    v.dedup();
    v.retain(|&n| n != id);
    v
}
pub fn distance(c: &GeosimConfig, a: usize, b: usize) -> f64 {
    let w = c.width as usize;
    let h = c.height as usize;
    let mut dx = (a % w).abs_diff(b % w);
    let mut dy = (a / w).abs_diff(b / w);
    if c.topology == super::Topology::Torus {
        dx = dx.min(w - dx);
        dy = dy.min(h - dy);
    }
    if c.distance_metric == super::DistanceMetric::Manhattan {
        (dx + dy) as f64
    } else {
        ((dx * dx + dy * dy) as f64).sqrt()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_and_torus_neighbors_have_no_duplicates() {
        let mut c = GeosimConfig {
            width: 3,
            height: 2,
            initial_states: 1,
            ..Default::default()
        };
        assert_eq!(adjacent(&c, 0), vec![1, 3]);
        c.topology = super::super::Topology::Torus;
        assert_eq!(adjacent(&c, 0), vec![1, 2, 3]);
    }
    #[test]
    fn euclidean_and_manhattan_are_explicit() {
        let mut c = GeosimConfig {
            width: 4,
            height: 3,
            initial_states: 1,
            ..Default::default()
        };
        assert!((distance(&c, 0, 6) - 5f64.sqrt()).abs() < 1e-12);
        c.distance_metric = super::super::DistanceMetric::Manhattan;
        assert_eq!(distance(&c, 0, 6), 3.0);
    }
}
