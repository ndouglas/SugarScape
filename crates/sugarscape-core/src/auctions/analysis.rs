//! Expected terminal policies, with exact affine integration of a uniform fringe.
use super::{config::*, mechanism};
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Evaluation {
    pub revenue: f64,
    pub profits: Vec<f64>,
    pub deviation_gain: Vec<f64>,
}
fn payoff(c: &AuctionsConfig, bids: &[f64]) -> (f64, Vec<f64>) {
    if c.fringe == Fringe::None {
        let o = mechanism::clear(c, bids, &vec![0.0f64; bids.len()], true);
        return (o.revenue, o.rewards);
    }
    let mut cuts = vec![0.0, 1.0, c.reserve];
    cuts.extend(bids.iter().copied().filter(|b| *b > 0.0 && *b < 1.0));
    cuts.sort_by(f64::total_cmp);
    cuts.dedup();
    let mut revenue = 0.0;
    let mut profits = vec![0.0f64; bids.len()];
    let mut profile = bids.to_vec();
    profile.push(0.0);
    let priorities = vec![0.0; profile.len()];
    for edge in cuts.windows(2) {
        let width = edge[1] - edge[0];
        for fraction in [0.25, 0.75] {
            *profile.last_mut().unwrap() = edge[0] + width * fraction;
            let o = mechanism::clear(c, &profile, &priorities, true);
            revenue += width * 0.5 * o.revenue;
            for (p, r) in profits.iter_mut().zip(o.rewards) {
                *p += width * 0.5 * r;
            }
        }
    }
    (revenue, profits)
}
pub fn evaluate(c: &AuctionsConfig, bids: &[f64]) -> Evaluation {
    let (revenue, profits) = payoff(c, bids);
    let mut deviation_gain = vec![0.0f64; bids.len()];
    let actions = mechanism::grid(c);
    for i in 0..bids.len() {
        let mut profile = bids.to_vec();
        for b in &actions {
            profile[i] = *b;
            deviation_gain[i] = deviation_gain[i].max(payoff(c, &profile).1[i] - profits[i]);
        }
    }
    Evaluation {
        revenue,
        profits,
        deviation_gain,
    }
}
pub fn equilibria(c: &AuctionsConfig) -> Vec<Vec<f64>> {
    if c.fringe != Fringe::None {
        return vec![];
    }
    mechanism::grid(c)
        .into_iter()
        .filter(|b| *b > 0.0 && *b >= c.reserve)
        .map(|b| vec![b; c.bidders as usize])
        .filter(|b| evaluate(c, b).deviation_gain.iter().all(|g| *g <= 1e-12))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weak_first_price_equilibrium_is_distinct_from_below_top_non_nash() {
        assert_eq!(
            equilibria(&AuctionsConfig::default()),
            vec![vec![0.9, 0.9], vec![0.95, 0.95]]
        );
        let c = AuctionsConfig {
            auction: Auction::SecondPrice,
            ..Default::default()
        };
        assert_eq!(equilibria(&c), vec![vec![0.95, 0.95]]);
        assert!((evaluate(&c, &[0.9, 0.9]).deviation_gain[0] - 0.05).abs() < 1e-12);
    }
    #[test]
    fn uniform_fringe_integrates_winning_and_losing_regions() {
        let c = AuctionsConfig {
            fringe: Fringe::Uniform,
            ..Default::default()
        };
        let o = evaluate(&c, &[0.5, 0.5]);
        assert!((o.revenue - 0.625).abs() < 1e-12);
        assert_eq!(o.profits, vec![0.125, 0.125]);
    }
}
