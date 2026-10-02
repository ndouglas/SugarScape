//! The learners: the state space (the last k periods' prices), a firm's
//! Q-table and greedy strategy, kept incrementally as the authors' code
//! keeps them, and the random numbers.

use rand::Rng;

use super::config::{CollusionConfig, Ties};
use super::ran2::Ran2;
use crate::rng::{self, SimRng};

/// Two Q-values this close are equal (the code's `AreEqualReals`:
/// `|a − b| ≤ EPSILON(1d0)`).
pub const TIE: f64 = f64::EPSILON;

/// The states: everyone's prices in the last k periods, the latest first,
/// firm 0 most significant (the code's `computeStateNumber`, 0-based).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Space {
    pub firms: usize,
    pub prices: usize,
    pub memory: usize,
    pub states: usize,
}

impl Space {
    pub fn of(c: &CollusionConfig) -> Self {
        Space {
            firms: c.firms as usize,
            prices: c.prices as usize,
            memory: c.memory as usize,
            states: c.states() as usize,
        }
    }

    /// The state from prices `p[d][i]` (d periods ago, firm i).
    pub fn encode(&self, p: &[Vec<u8>]) -> usize {
        let mut s = 0;
        for depth in p.iter().take(self.memory) {
            for &a in depth {
                s = s * self.prices + usize::from(a);
            }
        }
        s
    }

    /// The state after everyone plays `actions` in state `s`.
    pub fn next(&self, s: usize, actions: &[u8]) -> usize {
        if self.memory == 0 {
            return 0;
        }
        let block = self.prices.pow(self.firms as u32);
        let code = actions
            .iter()
            .fold(0, |acc, &a| acc * self.prices + usize::from(a));
        code * block.pow(self.memory as u32 - 1) + s / block
    }

    /// The prices the state records for the last period (none without memory).
    pub fn last(&self, s: usize) -> Option<Vec<u8>> {
        if self.memory == 0 {
            return None;
        }
        let block = self.prices.pow(self.firms as u32);
        let mut code = s / block.pow(self.memory as u32 - 1);
        let mut out = vec![0u8; self.firms];
        for i in (0..self.firms).rev() {
            out[i] = (code % self.prices) as u8;
            code /= self.prices;
        }
        Some(out)
    }

    /// `s` with firm `i`'s latest price replaced by `a` (the state a
    /// counterfactual price would have led to).
    pub fn with_own(&self, s: usize, i: usize, a: u8) -> usize {
        let Some(last) = self.last(s) else {
            return 0;
        };
        let place = self.prices.pow((self.firms - 1 - i) as u32)
            * self.prices.pow((self.firms * (self.memory - 1)) as u32);
        s - usize::from(last[i]) * place + usize::from(a) * place
    }
}

/// The random numbers a session draws.
#[derive(Clone, Debug)]
pub enum Draws {
    Ours(SimRng),
    /// The code's streams: exploration, and ties and Q initialization, both
    /// seeded −session.
    Calvano(Box<[Ran2; 2]>),
}

impl Draws {
    pub fn new(c: &CollusionConfig, seed: u64) -> Self {
        match c.rng {
            super::config::RngKind::Ours => Draws::Ours(rng::seeded(seed)),
            super::config::RngKind::Calvano => {
                let session = -(session(seed) as i32);
                Draws::Calvano(Box::new([Ran2::new(session), Ran2::new(session)]))
            }
        }
    }

    /// One period's exploration uniforms, in the code's order: u(1, firm 1),
    /// u(1, firm 2), …, then u(2, firm 1), ….
    pub fn explore(&mut self, out: &mut [[f64; 2]]) {
        for k in 0..2 {
            for u in out.iter_mut() {
                u[k] = match self {
                    Draws::Ours(r) => r.gen::<f64>(),
                    Draws::Calvano(streams) => streams[0].next_f64(),
                };
            }
        }
    }

    /// A uniform for breaking a tie or drawing a starting Q-value.
    pub fn tie(&mut self) -> f64 {
        match self {
            Draws::Ours(r) => r.gen::<f64>(),
            Draws::Calvano(streams) => streams[1].next_f64(),
        }
    }

    /// A price index uniform on 0..m.
    pub fn price(&mut self, m: usize) -> u8 {
        match self {
            Draws::Ours(r) => r.gen_range(0..m as u32) as u8,
            Draws::Calvano(streams) => (m as f64 * streams[1].next_f64()) as u8,
        }
    }
}

/// Sessions the code's RNG numbers: seed s is session ((s − 1) mod 10⁶) + 1,
/// so any 32-bit seed starts at once (session s skips (s − 1)·k·n
/// starting-price draws).
pub const SESSIONS: u64 = 1_000_000;

/// The session number a seed runs under `rng = calvano`.
pub fn session(seed: u64) -> u64 {
    (seed.max(1) - 1) % SESSIONS + 1
}

/// The starting prices: under `calvano`, the code's shared stream seeded −1,
/// session s taking draws after the (s − 1)·k·n before it; otherwise drawn.
pub fn initial_prices(c: &CollusionConfig, seed: u64, draws: &mut Draws) -> Vec<Vec<u8>> {
    let (n, m) = (c.firms as usize, c.prices as usize);
    let depth = (c.memory as usize).max(1);
    match draws {
        Draws::Calvano(_) => {
            let mut stream = Ran2::new(-1);
            let skip = (session(seed) - 1) * (depth * n) as u64;
            for _ in 0..skip {
                stream.next_f64();
            }
            (0..depth)
                .map(|_| {
                    (0..n)
                        .map(|_| (m as f64 * stream.next_f64()) as u8)
                        .collect()
                })
                .collect()
        }
        Draws::Ours(_) => (0..depth)
            .map(|_| (0..n).map(|_| draws.price(m)).collect())
            .collect(),
    }
}

/// One firm's learner.
#[derive(Clone, Debug, PartialEq)]
pub struct Firm {
    /// Q(s, a) at `q[s * m + a]`.
    pub q: Vec<f64>,
    /// max_a Q(s, a).
    pub best: Vec<f64>,
    /// The greedy price in each state.
    pub greedy: Vec<u8>,
    /// The exploration rate (or Boltzmann temperature) now.
    pub eps: f64,
}

impl Firm {
    pub fn new(q: Vec<f64>, m: usize, ties: Ties, draws: &mut Draws, eps: f64) -> Self {
        let states = q.len() / m;
        let mut f = Firm {
            q,
            best: vec![0.0; states],
            greedy: vec![0; states],
            eps,
        };
        for s in 0..states {
            f.rescan(s, m, ties, draws);
        }
        f
    }

    pub fn row(&self, s: usize, m: usize) -> &[f64] {
        &self.q[s * m..(s + 1) * m]
    }

    /// Recomputes state `s`'s maximum and greedy price: among values within
    /// `TIE` of the maximum, the lowest, or one drawn (the code's
    /// `MaxLocBreakTies`, which draws only when there is more than one).
    pub fn rescan(&mut self, s: usize, m: usize, ties: Ties, draws: &mut Draws) {
        let row = &self.q[s * m..(s + 1) * m];
        let top = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let tied: Vec<u8> = (0..m as u8)
            .filter(|&a| (row[a as usize] - top).abs() <= TIE)
            .collect();
        self.best[s] = top;
        self.greedy[s] = match ties {
            Ties::Lowest => tied[0],
            Ties::Random if tied.len() > 1 => tied[(tied.len() as f64 * draws.tie()) as usize],
            Ties::Random => tied[0],
        };
    }

    /// Writes `new` into Q(s, a) and keeps the greedy price as `ties` says:
    /// under `random` exactly the code's rule (a new maximum takes over; the
    /// greedy price's own value falling triggers a rescan); under `lowest`
    /// a rescan whenever the maximum may have moved.
    pub fn set(&mut self, s: usize, a: u8, new: f64, m: usize, ties: Ties, draws: &mut Draws) {
        let k = s * m + usize::from(a);
        self.q[k] = new;
        match ties {
            Ties::Random => {
                if new > self.best[s] {
                    self.best[s] = new;
                    self.greedy[s] = a;
                }
                if new < self.best[s] && self.greedy[s] == a {
                    self.rescan(s, m, ties, draws);
                }
            }
            Ties::Lowest => {
                if self.greedy[s] == a || new >= self.best[s] - TIE {
                    self.rescan(s, m, ties, draws);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(firms: u32, memory: u32) -> Space {
        Space::of(&CollusionConfig {
            firms,
            memory,
            ..CollusionConfig::default()
        })
    }

    #[test]
    fn states_record_the_last_prices_firm_one_first() {
        let s = space(2, 1);
        assert_eq!(s.states, 225);
        assert_eq!(s.encode(&[vec![3, 9]]), 3 * 15 + 9);
        assert_eq!(s.next(0, &[3, 9]), 54);
        assert_eq!(s.last(54), Some(vec![3, 9]));
        assert_eq!(s.with_own(54, 1, 2), s.encode(&[vec![3, 2]]));
        assert_eq!(s.with_own(54, 0, 14), s.encode(&[vec![14, 9]]));
    }

    #[test]
    fn two_periods_of_memory_shift_the_older_prices_down() {
        let s = space(2, 2);
        let start = s.encode(&[vec![1, 2], vec![3, 4]]);
        let after = s.next(start, &[5, 6]);
        assert_eq!(after, s.encode(&[vec![5, 6], vec![1, 2]]));
        assert_eq!(s.last(after), Some(vec![5, 6]));
        assert_eq!(s.with_own(after, 0, 7), s.encode(&[vec![7, 6], vec![1, 2]]));
        let none = space(2, 0);
        assert_eq!(
            (none.states, none.next(0, &[3, 4]), none.last(0)),
            (1, 0, None)
        );
    }

    #[test]
    fn the_greedy_price_follows_the_tie_rule() {
        let mut draws = Draws::new(&CollusionConfig::default(), 1);
        let q = vec![1.0, 3.0, 3.0, 2.0];
        let low = Firm::new(q.clone(), 4, Ties::Lowest, &mut draws, 1.0);
        assert_eq!((low.greedy[0], low.best[0]), (1, 3.0));
        // Lowering the greedy price's value hands the lead to the tied one.
        let mut f = low.clone();
        f.set(0, 1, 0.5, 4, Ties::Lowest, &mut draws);
        assert_eq!(f.greedy[0], 2);
        // Under `lowest` a new value equal to the maximum takes over if lower.
        let mut g = Firm::new(vec![1.0, 2.0, 3.0, 0.0], 4, Ties::Lowest, &mut draws, 1.0);
        g.set(0, 0, 3.0, 4, Ties::Lowest, &mut draws);
        assert_eq!(g.greedy[0], 0);
        // The code's rule leaves an exact tie alone.
        let mut r = Firm::new(vec![1.0, 2.0, 3.0, 0.0], 4, Ties::Random, &mut draws, 1.0);
        r.set(0, 0, 3.0, 4, Ties::Random, &mut draws);
        assert_eq!(r.greedy[0], 2);
        r.set(0, 3, 4.0, 4, Ties::Random, &mut draws);
        assert_eq!((r.greedy[0], r.best[0]), (3, 4.0));
    }

    #[test]
    fn incremental_greedy_prices_match_a_full_rescan() {
        let mut draws = Draws::new(&CollusionConfig::default(), 3);
        let mut f = Firm::new(vec![0.0; 6 * 5], 5, Ties::Lowest, &mut draws, 1.0);
        let mut r = rng::seeded(9);
        for _ in 0..20_000 {
            let (s, a) = (r.gen_range(0..6u32) as usize, r.gen_range(0..5u32) as u8);
            let v = f64::from(r.gen_range(0..8u32)) * 0.5;
            f.set(s, a, v, 5, Ties::Lowest, &mut draws);
            let mut check = f.clone();
            check.rescan(s, 5, Ties::Lowest, &mut draws);
            assert_eq!((f.greedy[s], f.best[s]), (check.greedy[s], check.best[s]));
        }
    }
}
