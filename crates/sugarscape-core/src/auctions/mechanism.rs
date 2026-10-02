//! Allocation and payment, independent of learning and random generation.
use super::config::*;
#[derive(Clone, Debug, PartialEq)]
pub struct Clearing {
    pub revenue: f64,
    pub rewards: Vec<f64>,
    pub shares: Vec<f64>,
}
pub fn grid(c: &AuctionsConfig) -> Vec<f64> {
    (1 - i32::try_from(c.out_bids).unwrap()..=i32::try_from(c.bids).unwrap())
        .map(|j| f64::from(j) / f64::from(c.bids + 1))
        .collect()
}
pub fn clear(c: &AuctionsConfig, bids: &[f64], priorities: &[f64], expected: bool) -> Clearing {
    let n = bids.len();
    let mut out = Clearing {
        revenue: 0.0,
        rewards: vec![0.0; n],
        shares: vec![0.0; n],
    };
    let top = bids
        .iter()
        .copied()
        .filter(|b| *b > 0.0 && *b >= c.reserve)
        .fold(0.0, f64::max);
    if top == 0.0 {
        return out;
    }
    let tied: Vec<usize> = bids
        .iter()
        .enumerate()
        .filter_map(|(i, b)| (*b == top).then_some(i))
        .collect();
    let winner = *tied
        .iter()
        .min_by(|a, b| priorities[**a].total_cmp(&priorities[**b]).then(a.cmp(b)))
        .unwrap();
    let competing = bids
        .iter()
        .enumerate()
        .filter(|(i, b)| *i != winner && **b > 0.0)
        .map(|(_, b)| *b)
        .fold(0.0, f64::max);
    let second = if c.reserve_payment == ReservePayment::Floor {
        competing.max(c.reserve)
    } else {
        competing
    };
    out.revenue = (2.0 - c.alpha()) * top + (c.alpha() - 1.0) * second;
    if expected {
        for i in tied.iter().copied() {
            out.shares[i] = 1.0 / tied.len() as f64;
            out.rewards[i] = out.shares[i] * (1.0 - out.revenue);
        }
    } else {
        out.shares[winner] = 1.0;
        out.rewards[winner] = 1.0 - out.revenue;
    }
    out
}
pub fn hypothetical(
    c: &AuctionsConfig,
    bids: &[f64],
    priorities: &[f64],
    bidder: usize,
    candidate: f64,
) -> f64 {
    let mut profile = bids.to_vec();
    profile[bidder] = candidate;
    let expected =
        c.hindsight_ties == HindsightTies::Expected || c.auction_ties == AuctionTies::Expected;
    clear(c, &profile, priorities, expected).rewards[bidder]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixture_and_three_way_expected_ties_conserve_reward() {
        let c = AuctionsConfig {
            auction: Auction::Mixture,
            auction_alpha: 1.5,
            bidders: 3,
            ..Default::default()
        };
        let o = clear(&c, &[0.8, 0.4, 0.3], &[0.2, 0.1, 0.9], false);
        assert!((o.revenue - 0.6).abs() < 1e-12);
        let o = clear(&c, &[0.6, 0.6, 0.6], &[0.2, 0.1, 0.9], true);
        assert!((o.rewards.iter().sum::<f64>() + o.revenue - 1.0).abs() < 1e-12);
        assert_eq!(o.shares, vec![1.0 / 3.0; 3]);
    }
    #[test]
    fn realized_hindsight_reuses_priorities_and_expected_hindsight_splits() {
        let mut c = AuctionsConfig {
            hindsight_ties: HindsightTies::Realized,
            ..Default::default()
        };
        assert!((hypothetical(&c, &[0.4, 0.6], &[0.1, 0.9], 0, 0.6) - 0.4).abs() < 1e-12);
        assert_eq!(hypothetical(&c, &[0.4, 0.6], &[0.9, 0.1], 0, 0.6), 0.0);
        c.hindsight_ties = HindsightTies::Expected;
        assert!((hypothetical(&c, &[0.4, 0.6], &[0.9, 0.1], 0, 0.6) - 0.2).abs() < 1e-12);
        c.hindsight_ties = HindsightTies::Realized;
        c.auction_ties = AuctionTies::Expected;
        assert!((hypothetical(&c, &[0.4, 0.6], &[0.9, 0.1], 0, 0.6) - 0.2).abs() < 1e-12);
    }
    #[test]
    fn out_actions_are_distinct_canonical_divisions_and_reserve_equality_is_eligible() {
        let c = AuctionsConfig {
            out_bids: 7,
            reserve: 0.2,
            ..Default::default()
        };
        let g = grid(&c);
        assert_eq!(&g[..7], &[-0.3, -0.25, -0.2, -0.15, -0.1, -0.05, 0.0]);
        assert_eq!(clear(&c, &[0.2, 0.0], &[0.9, 0.1], false).revenue, 0.2);
    }
    #[test]
    fn first_price_winner_pays_own_bid() {
        let o = clear(&AuctionsConfig::default(), &[0.8, 0.4], &[0.1, 0.9], false);
        assert_eq!(o.revenue, 0.8);
        assert!((o.rewards[0] - 0.2).abs() < 1e-12);
    }
    #[test]
    fn second_price_and_tie_rewards() {
        let c = AuctionsConfig {
            auction: Auction::SecondPrice,
            ..Default::default()
        };
        let o = clear(&c, &[0.8, 0.4], &[0.1, 0.9], true);
        assert_eq!(o.revenue, 0.4);
        assert_eq!(o.rewards, vec![0.6, 0.0]);
        let o = clear(&c, &[0.8, 0.8], &[0.1, 0.9], true);
        assert!((o.rewards[0] - 0.1).abs() < 1e-12);
    }
    #[test]
    fn nonparticipation_never_wins_and_reserve_changes_second_payment() {
        let c = AuctionsConfig {
            auction: Auction::SecondPrice,
            reserve: 0.2,
            ..Default::default()
        };
        assert_eq!(clear(&c, &[0.0, -0.1], &[0.0, 0.0], false).revenue, 0.0);
        assert_eq!(clear(&c, &[0.3, 0.1], &[0.0, 0.0], false).revenue, 0.2);
        let c = AuctionsConfig {
            reserve_payment: ReservePayment::EligibilityOnly,
            ..c
        };
        assert_eq!(clear(&c, &[0.3, 0.1], &[0.0, 0.0], false).revenue, 0.1);
    }
}
