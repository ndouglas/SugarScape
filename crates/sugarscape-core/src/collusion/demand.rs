//! The stage game: logit demand (CCDP eq. 5), the one-shot Bertrand–Nash and
//! joint-monopoly prices, the price grid and the payoff table.

use super::config::{CollusionConfig, Grid, BELOW_BOTTOM};
use crate::portable::exp_neg;

/// The market's demand parameters, per firm.
#[derive(Clone, Debug, PartialEq)]
pub struct Market {
    pub quality: Vec<f64>,
    pub cost: Vec<f64>,
    pub outside: f64,
    pub mu: f64,
}

impl Market {
    pub fn of(c: &CollusionConfig) -> Self {
        let n = c.firms as usize;
        Market {
            quality: vec![c.quality; n],
            cost: (0..n).map(|i| c.cost_of(i)).collect(),
            outside: c.outside,
            mu: c.mu,
        }
    }

    pub fn firms(&self) -> usize {
        self.cost.len()
    }

    /// Market shares at prices `p`: exp((aᵢ − pᵢ)/μ) / (Σ exp((aⱼ − pⱼ)/μ) +
    /// exp(a₀/μ)), every exponent shifted by the largest so the portable
    /// exp sees only non-positive arguments.
    pub fn shares(&self, p: &[f64]) -> Vec<f64> {
        let z: Vec<f64> = (0..self.firms())
            .map(|i| (self.quality[i] - p[i]) / self.mu)
            .collect();
        let z0 = self.outside / self.mu;
        let top = z.iter().copied().fold(z0, f64::max);
        let e: Vec<f64> = z.iter().map(|&x| exp_neg(x - top)).collect();
        let total = e.iter().sum::<f64>() + exp_neg(z0 - top);
        e.iter().map(|x| x / total).collect()
    }

    pub fn profits(&self, p: &[f64]) -> Vec<f64> {
        self.shares(p)
            .iter()
            .enumerate()
            .map(|(i, q)| (p[i] - self.cost[i]) * q)
            .collect()
    }

    /// The root of an increasing `f` on [lo, hi], by bisection to the last bit.
    fn bisect(lo: f64, hi: f64, mut f: impl FnMut(f64) -> f64) -> f64 {
        let (mut lo, mut hi) = (lo, hi);
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if f(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// The one-shot Bertrand–Nash prices: each firm's best response solves
    /// pᵢ − cᵢ = μ/(1 − qᵢ) given the others' prices (increasing in pᵢ, so
    /// bisection), repeated until no price moves.
    pub fn nash(&self) -> Vec<f64> {
        let mut p: Vec<f64> = self.cost.iter().map(|c| c + self.mu).collect();
        for _ in 0..10_000 {
            let mut moved: f64 = 0.0;
            for i in 0..self.firms() {
                let mut trial = p.clone();
                let best = Self::bisect(self.cost[i], self.cost[i] + 1000.0 * self.mu, |x| {
                    trial[i] = x;
                    x - self.cost[i] - self.mu / (1.0 - self.shares(&trial)[i])
                });
                moved = moved.max((best - p[i]).abs());
                p[i] = best;
            }
            if moved == 0.0 {
                break;
            }
        }
        p
    }

    /// The joint-profit maximum: equal markups m = μ/(1 − Σq), the root of
    /// an increasing function of m.
    pub fn monopoly(&self) -> Vec<f64> {
        let at = |m: f64| -> Vec<f64> { self.cost.iter().map(|c| c + m).collect() };
        let m = Self::bisect(0.0, 1000.0 * self.mu, |m| {
            m - self.mu / (1.0 - self.shares(&at(m)).iter().sum::<f64>())
        });
        at(m)
    }
}

/// Rounds to 5 decimals, as the authors' inputs are.
pub fn five(x: f64) -> f64 {
    (x * 1e5).round() / 1e5
}

/// The stage game a session is played on.
#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub firms: usize,
    pub prices: usize,
    /// Each firm's grid, `grid[i][a]`.
    pub grid: Vec<Vec<f64>>,
    /// Profits: `payoff[profile * firms + i]`, profile = Σ aᵢ·m^(n−1−i).
    pub payoff: Vec<f64>,
    pub nash: Vec<f64>,
    pub monopoly: Vec<f64>,
    /// π^N and π^M per firm, at the (rounded) benchmark prices.
    pub nash_profit: Vec<f64>,
    pub monopoly_profit: Vec<f64>,
}

impl Game {
    pub fn new(c: &CollusionConfig) -> Self {
        let market = Market::of(c);
        let n = c.firms as usize;
        let m = c.prices as usize;
        let nash: Vec<f64> = market.nash().into_iter().map(five).collect();
        let monopoly: Vec<f64> = market.monopoly().into_iter().map(five).collect();
        let grid: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                let zeta = monopoly[i] - nash[i];
                let (lo, hi) = match c.grid {
                    Grid::Calvano => (nash[i] - c.xi * zeta, monopoly[i] + c.xi * zeta),
                    Grid::Symmetric => {
                        (nash[i] - (1.0 + c.xi) * zeta, nash[i] + (1.0 + c.xi) * zeta)
                    }
                    Grid::BelowNash => (BELOW_BOTTOM, c.below_top),
                };
                // The code's construction: the ends set, the inside by
                // cumulative addition.
                let step = (hi - lo) / (m - 1) as f64;
                let mut g = vec![0.0; m];
                g[0] = lo;
                for a in 1..m - 1 {
                    g[a] = g[a - 1] + step;
                }
                g[m - 1] = hi;
                g
            })
            .collect();
        let profiles = m.pow(n as u32);
        let mut payoff = vec![0.0; profiles * n];
        let mut prices = vec![0.0; n];
        for profile in 0..profiles {
            let mut rest = profile;
            for i in (0..n).rev() {
                prices[i] = grid[i][rest % m];
                rest /= m;
            }
            let pi = market.profits(&prices);
            payoff[profile * n..(profile + 1) * n].copy_from_slice(&pi);
        }
        Game {
            firms: n,
            prices: m,
            nash_profit: market.profits(&nash),
            monopoly_profit: market.profits(&monopoly),
            grid,
            payoff,
            nash,
            monopoly,
        }
    }

    /// The profile index of `actions` (firm 0 most significant).
    pub fn profile(&self, actions: &[u8]) -> usize {
        actions
            .iter()
            .fold(0, |acc, &a| acc * self.prices + usize::from(a))
    }

    pub fn profit(&self, actions: &[u8], i: usize) -> f64 {
        self.payoff[self.profile(actions) * self.firms + i]
    }

    /// Firm `i`'s profit gain (π − π^N)/(π^M − π^N).
    pub fn gain(&self, i: usize, profit: f64) -> f64 {
        (profit - self.nash_profit[i]) / (self.monopoly_profit[i] - self.nash_profit[i])
    }

    /// The grid index nearest `price` for firm `i`.
    pub fn nearest(&self, i: usize, price: f64) -> u8 {
        let mut best = 0;
        for a in 1..self.prices {
            if (self.grid[i][a] - price).abs() < (self.grid[i][best] - price).abs() {
                best = a;
            }
        }
        best as u8
    }

    /// The best one-period price for firm `i` against `actions` (lowest on ties).
    pub fn static_best_response(&self, actions: &[u8], i: usize) -> u8 {
        let mut a = actions.to_vec();
        let mut best = (0u8, f64::NEG_INFINITY);
        for p in 0..self.prices as u8 {
            a[i] = p;
            let v = self.profit(&a, i);
            if v > best.1 {
                best = (p, v);
            }
        }
        best.0
    }

    /// den Boer, Meylahn & Schinkel's effective horizon T_δ =
    /// ⌈ln(π_min/(1000 π_max))/ln δ⌉ over the positive profits in the table
    /// (1 when δ = 0).
    pub fn horizon(&self, delta: f64) -> u32 {
        if delta <= 0.0 {
            return 1;
        }
        let positive = self.payoff.iter().copied().filter(|&p| p > 0.0);
        let (lo, hi) = positive.fold((f64::INFINITY, 0.0_f64), |(lo, hi), p| {
            (lo.min(p), hi.max(p))
        });
        if !lo.is_finite() {
            return 1;
        }
        let t = crate::portable::ln(lo / (1000.0 * hi)) / crate::portable::ln(delta);
        t.ceil().max(1.0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(edit: impl FnOnce(&mut CollusionConfig)) -> Game {
        let mut c = CollusionConfig::default();
        edit(&mut c);
        Game::new(&c)
    }

    #[test]
    fn the_benchmarks_are_the_codes_inputs() {
        let g = game(|_| {});
        assert_eq!((g.nash[0], g.monopoly[0]), (1.47293, 1.92498));
        let unrounded = Market::of(&CollusionConfig::default());
        assert!((unrounded.nash()[0] - 1.472927).abs() < 1e-6);
        assert!((unrounded.monopoly()[0] - 1.924981).abs() < 1e-6);
        assert!((g.nash_profit[0] - 0.22293).abs() < 1e-5);
        assert!((g.monopoly_profit[0] - 0.33749).abs() < 1e-5);
        let three = game(|c| c.firms = 3);
        assert_eq!((three.nash[0], three.monopoly[0]), (1.37016, 2.0));
        let wide = game(|c| c.mu = 0.5);
        assert_eq!((wide.nash[0], wide.monopoly[0]), (1.79947, 2.18741));
    }

    #[test]
    // 1.4142 is the code's input for firm 1's Nash price at c₂ = 0.75, not √2.
    #[allow(clippy::approx_constant)]
    fn asymmetric_costs_match_the_codes_inputs() {
        for (c2, n1, n2, m1, m2) in [
            (0.875, 1.44135, 1.39588, 1.97674, 1.85174),
            (0.75, 1.4142, 1.32539, 2.04051, 1.79051),
            (0.5, 1.37233, 1.20377, 2.1984, 1.6984),
            (0.25, 1.34369, 1.10519, 2.38411, 1.63411),
        ] {
            let g = game(|c| c.cost2 = Some(c2));
            assert_eq!(
                (g.nash[0], g.nash[1], g.monopoly[0], g.monopoly[1]),
                (n1, n2, m1, m2),
                "c2 {c2}"
            );
        }
    }

    #[test]
    fn the_grid_is_the_codes() {
        let g = game(|_| {});
        // The authors' run prints its grid to 7 decimals (A_res.txt).
        let want = [
            1.4277250, 1.4664721, 1.5052193, 1.5439664, 1.5827136, 1.6214607, 1.6602079, 1.6989550,
            1.7377021, 1.7764493, 1.8151964, 1.8539436, 1.8926907, 1.9314379, 1.9701850,
        ];
        for (a, w) in want.iter().enumerate() {
            assert!(
                (g.grid[0][a] - w).abs() < 6e-8,
                "price {a}: {}",
                g.grid[0][a]
            );
        }
        assert_eq!(g.grid[0][14], 1.92498 + 0.1 * (1.92498 - 1.47293));
        let below = game(|c| c.grid = Grid::BelowNash);
        assert_eq!((below.grid[0][0], below.grid[0][14]), (1.25, 1.47));
        let sym = game(|c| {
            c.grid = Grid::Symmetric;
            c.xi = 0.0;
        });
        assert!((sym.grid[0][7] - 1.47293).abs() < 1e-12, "centered on Nash");
        assert_eq!(sym.grid[0][14], 1.92498);
    }

    #[test]
    fn payoffs_follow_the_profile_order() {
        let g = game(|_| {});
        let m = Market::of(&CollusionConfig::default());
        let p = m.profits(&[g.grid[0][3], g.grid[1][9]]);
        assert_eq!(g.profit(&[3, 9], 0), p[0]);
        assert_eq!(g.profit(&[3, 9], 1), p[1]);
        assert_eq!(g.profile(&[3, 9]), 3 * 15 + 9);
        // Symmetric prices: Δ at index 9 (1.7377) is 0.794 (Lambin's I at δ = 0.95).
        let pi = g.profit(&[8, 8], 0);
        assert!((g.gain(0, pi) - 0.794).abs() < 0.001, "{}", g.gain(0, pi));
    }

    #[test]
    fn uniform_random_pricing_earns_den_boer_s_0_497() {
        let g = game(|_| {});
        let mean: f64 = (0..225).map(|k| g.payoff[k * 2]).sum::<f64>() / 225.0;
        assert!(
            (g.gain(0, mean) - 0.497).abs() < 0.0005,
            "{}",
            g.gain(0, mean)
        );
        let sym = game(|c| {
            c.grid = Grid::Symmetric;
            c.xi = 0.0;
        });
        let mean: f64 = (0..225).map(|k| sym.payoff[k * 2]).sum::<f64>() / 225.0;
        assert!(
            (sym.gain(0, mean) + 0.510).abs() < 0.0005,
            "{}",
            sym.gain(0, mean)
        );
        assert_eq!(g.horizon(0.95), 165);
        assert_eq!(g.horizon(0.0), 1);
    }

    #[test]
    fn the_static_best_response_undercuts_a_high_rival() {
        let g = game(|_| {});
        let br = g.static_best_response(&[14, 14], 0);
        assert!(br < 14 && br > 0, "{br}");
        assert_eq!(g.nearest(0, 1.47293), 1);
    }
}
