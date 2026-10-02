//! Stateless Q choices, using explicit draws and a frozen bootstrap maximum.
use super::config::*;
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Learner {
    pub q: Vec<f64>,
    pub greedy: usize,
    pub chosen: Vec<u64>,
    pub updated: Vec<u64>,
}
impl Learner {
    pub fn new(c: &AuctionsConfig, grid: &[f64], draw: f64) -> Self {
        let initial = match c.q_init {
            QInit::Optimistic => {
                c.q_scale
                    / if c.optimism == Optimism::Discounted {
                        1.0 - c.discount
                    } else {
                        1.0
                    }
            }
            QInit::Constant => c.q_level,
            QInit::Biased => c.bias_rest,
        };
        let mut out = Self {
            q: vec![initial; grid.len()],
            greedy: 0,
            chosen: vec![0; grid.len()],
            updated: vec![0; grid.len()],
        };
        if c.q_init == QInit::Biased {
            let index = grid
                .iter()
                .position(|b| (*b - c.bias_bid).abs() <= 1e-12)
                .unwrap();
            out.q[index] = c.bias_q;
        }
        out.refresh(c, draw);
        out
    }
    pub fn maximum(&self) -> f64 {
        self.q.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }
    pub fn refresh(&mut self, c: &AuctionsConfig, draw: f64) {
        let max = self.maximum();
        let ties: Vec<usize> = self
            .q
            .iter()
            .enumerate()
            .filter_map(|(i, q)| ((max - q) <= c.q_tolerance).then_some(i))
            .collect();
        self.greedy = match c.greedy_ties {
            GreedyTies::Lowest => ties[0],
            GreedyTies::Highest => *ties.last().unwrap(),
            GreedyTies::Random => ties[(draw * ties.len() as f64).floor() as usize],
            GreedyTies::Incumbent => {
                if ties.contains(&self.greedy) {
                    self.greedy
                } else {
                    ties[0]
                }
            }
        };
    }
    pub fn explore(&self, c: &AuctionsConfig, u: f64) -> usize {
        let m = self.q.len();
        match c.exploration_set {
            ExplorationSet::All => (u * m as f64).floor() as usize,
            ExplorationSet::Other => {
                let k = (u * (m - 1) as f64).floor() as usize;
                k + usize::from(k >= self.greedy)
            }
            ExplorationSet::Neighbors => {
                if c.neighbor_boundary == NeighborBoundary::Clamp {
                    if u < 0.5 {
                        self.greedy.saturating_sub(1)
                    } else {
                        (self.greedy + 1).min(m - 1)
                    }
                } else {
                    let mut actions = vec![];
                    if self.greedy > 0 {
                        actions.push(self.greedy - 1);
                    }
                    if self.greedy + 1 < m {
                        actions.push(self.greedy + 1);
                    }
                    actions[(u * actions.len() as f64).floor() as usize]
                }
            }
        }
    }
    pub fn update(&mut self, c: &AuctionsConfig, action: usize, rewards: &[f64], draw: f64) {
        let max = self.maximum();
        self.chosen[action] += 1;
        match c.update {
            Update::Chosen => {
                self.q[action] = (1.0 - c.learning_rate) * self.q[action]
                    + c.learning_rate * (rewards[action] + c.discount * max);
                self.updated[action] += 1;
            }
            Update::All => {
                for (i, q) in self.q.iter_mut().enumerate() {
                    *q = (1.0 - c.learning_rate) * *q
                        + c.learning_rate * (rewards[i] + c.discount * max);
                    self.updated[i] += 1;
                }
            }
        }
        self.refresh(c, draw);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn greedy_tolerance_and_incumbent_select_only_maximizers() {
        let mut f = Learner {
            q: vec![1.0, 1.0 - 1e-9, 1.0],
            greedy: 1,
            chosen: vec![0; 3],
            updated: vec![0; 3],
        };
        let mut c = AuctionsConfig {
            q_tolerance: 1e-8,
            greedy_ties: GreedyTies::Incumbent,
            ..Default::default()
        };
        f.refresh(&c, 0.8);
        assert_eq!(f.greedy, 1);
        c.q_tolerance = 0.0;
        f.refresh(&c, 0.8);
        assert_eq!(f.greedy, 0);
        c.greedy_ties = GreedyTies::Highest;
        f.refresh(&c, 0.0);
        assert_eq!(f.greedy, 2);
        c.greedy_ties = GreedyTies::Random;
        f.refresh(&c, 0.75);
        assert_eq!(f.greedy, 2);
    }
    #[test]
    fn neighbor_boundary_and_other_exploration_keep_distinct_choices() {
        let f = Learner {
            q: vec![1.0; 3],
            greedy: 0,
            chosen: vec![0; 3],
            updated: vec![0; 3],
        };
        let mut c = AuctionsConfig {
            exploration_set: ExplorationSet::Neighbors,
            ..Default::default()
        };
        assert_eq!(f.explore(&c, 0.1), 1);
        assert_eq!(f.explore(&c, 0.9), 1);
        c.neighbor_boundary = NeighborBoundary::Clamp;
        assert_eq!(f.explore(&c, 0.1), 0);
        assert_eq!(f.explore(&c, 0.9), 1);
        c.exploration_set = ExplorationSet::Other;
        assert_eq!(f.explore(&c, 0.1), 1);
        assert_eq!(f.explore(&c, 0.9), 2);
    }
    #[test]
    fn stage_and_discounted_optimism_and_bias_initialize_distinct_values() {
        let grid = vec![0.2, 0.4, 0.6];
        let c = AuctionsConfig {
            discount: 0.5,
            ..Default::default()
        };
        assert_eq!(Learner::new(&c, &grid, 0.0).q, vec![2.0; 3]);
        assert_eq!(
            Learner::new(
                &AuctionsConfig {
                    optimism: Optimism::Stage,
                    ..c.clone()
                },
                &grid,
                0.0
            )
            .q,
            vec![1.0; 3]
        );
        let f = Learner::new(
            &AuctionsConfig {
                q_init: QInit::Biased,
                bias_rest: 2.0,
                ..c
            },
            &grid,
            0.0,
        );
        assert_eq!(f.q, vec![2.0, 30.0, 2.0]);
        assert_eq!(f.greedy, 1);
    }
    #[test]
    fn all_updates_freeze_old_maximum() {
        let c = AuctionsConfig {
            learning_rate: 0.5,
            discount: 0.5,
            feedback: Feedback::RivalBids,
            update: Update::All,
            ..Default::default()
        };
        let mut f = Learner {
            q: vec![4.0, 2.0],
            greedy: 0,
            chosen: vec![0; 2],
            updated: vec![0; 2],
        };
        f.update(&c, 0, &[1.0, 0.0], 0.0);
        assert_eq!(f.q, vec![3.5, 2.0]);
    }
    #[test]
    fn chosen_update_leaves_other_actions_untouched() {
        let c = AuctionsConfig {
            learning_rate: 0.5,
            discount: 0.5,
            ..Default::default()
        };
        let mut f = Learner {
            q: vec![2.0, 4.0],
            greedy: 1,
            chosen: vec![0; 2],
            updated: vec![0; 2],
        };
        f.update(&c, 0, &[1.0, 0.0], 0.0);
        assert_eq!(f.q, vec![2.5, 4.0]);
    }
}
