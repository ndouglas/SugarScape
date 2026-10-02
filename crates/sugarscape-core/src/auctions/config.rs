//! Every experimental convention is serialized and applies at reset.
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Auction {
    FirstPrice,
    SecondPrice,
    Mixture,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservePayment {
    Floor,
    EligibilityOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fringe {
    None,
    Uniform,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feedback {
    Outcome,
    RivalBids,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    Chosen,
    All,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuctionTies {
    Sampled,
    Expected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HindsightTies {
    Expected,
    Realized,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GreedyTies {
    Lowest,
    Highest,
    Random,
    Incumbent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QInit {
    Optimistic,
    Constant,
    Biased,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Optimism {
    Discounted,
    Stage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exploration {
    Decaying,
    Constant,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExplorationSet {
    All,
    Other,
    Neighbors,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeighborBoundary {
    Available,
    Clamp,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodOrigin {
    Zero,
    One,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvergencePhase {
    PostUpdate,
    PreUpdate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownwardTrigger {
    Off,
    Stable,
    Period,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownwardClock {
    Activation,
    Global,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AuctionsConfig {
    pub bidders: u32,
    pub bids: u32,
    pub auction: Auction,
    pub auction_alpha: f64,
    pub reserve: f64,
    pub reserve_payment: ReservePayment,
    pub out_bids: u32,
    pub fringe: Fringe,
    pub learning_rate: f64,
    pub discount: f64,
    pub feedback: Feedback,
    pub update: Update,
    pub auction_ties: AuctionTies,
    pub hindsight_ties: HindsightTies,
    pub greedy_ties: GreedyTies,
    pub q_tolerance: f64,
    pub q_init: QInit,
    pub optimism: Optimism,
    pub q_scale: f64,
    pub q_level: f64,
    pub bias_bid: f64,
    pub bias_q: f64,
    pub bias_rest: f64,
    pub exploration: Exploration,
    pub epsilon: f64,
    pub beta: f64,
    pub exploration_set: ExplorationSet,
    pub neighbor_boundary: NeighborBoundary,
    pub period_origin: PeriodOrigin,
    pub convergence_phase: ConvergencePhase,
    pub downward_trigger: DownwardTrigger,
    pub downward_at: u32,
    pub downward_clock: DownwardClock,
    pub downward_chi: f64,
    pub downward_beta: f64,
    pub downward_gap: f64,
    pub horizon: u32,
    pub window: u32,
    pub periods_per_tick: u32,
}
impl Default for AuctionsConfig {
    fn default() -> Self {
        Self {
            bidders: 2,
            bids: 19,
            auction: Auction::FirstPrice,
            auction_alpha: 1.0,
            reserve: 0.0,
            reserve_payment: ReservePayment::Floor,
            out_bids: 0,
            fringe: Fringe::None,
            learning_rate: 0.05,
            discount: 0.99,
            feedback: Feedback::Outcome,
            update: Update::Chosen,
            auction_ties: AuctionTies::Sampled,
            hindsight_ties: HindsightTies::Expected,
            greedy_ties: GreedyTies::Lowest,
            q_tolerance: 0.0,
            q_init: QInit::Optimistic,
            optimism: Optimism::Discounted,
            q_scale: 1.0,
            q_level: 100.0,
            bias_bid: 0.4,
            bias_q: 30.0,
            bias_rest: 0.0,
            exploration: Exploration::Decaying,
            epsilon: 0.025,
            beta: 0.0002,
            exploration_set: ExplorationSet::All,
            neighbor_boundary: NeighborBoundary::Available,
            period_origin: PeriodOrigin::Zero,
            convergence_phase: ConvergencePhase::PostUpdate,
            downward_trigger: DownwardTrigger::Off,
            downward_at: 100_000,
            downward_clock: DownwardClock::Activation,
            downward_chi: 0.62,
            downward_beta: 0.002,
            downward_gap: 0.3,
            horizon: 1_000_000,
            window: 1000,
            periods_per_tick: 1000,
        }
    }
}
impl AuctionsConfig {
    pub fn alpha(&self) -> f64 {
        match self.auction {
            Auction::FirstPrice => 1.0,
            Auction::SecondPrice => 2.0,
            Auction::Mixture => self.auction_alpha,
        }
    }
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = vec![];
        let mut check = |ok: bool, field: &str, msg: &str| {
            if !ok {
                errors.push(FieldError::new(field, msg));
            }
        };
        check(
            (2..=3).contains(&self.bidders),
            "bidders",
            "must be finite and between 2 and 3",
        );
        check(
            (2..=99).contains(&self.bids),
            "bids",
            "must be finite and between 2 and 99",
        );
        check(
            self.auction_alpha.is_finite() && (1.0..=2.0).contains(&self.auction_alpha),
            "auction_alpha",
            "must be finite and between 1 and 2",
        );
        check(
            self.reserve.is_finite() && (0.0..=0.99).contains(&self.reserve),
            "reserve",
            "must be finite and between 0 and 0.99",
        );
        check(
            (0..=16).contains(&self.out_bids),
            "out_bids",
            "must be finite and between 0 and 16",
        );
        check(
            self.learning_rate.is_finite() && (0.0..=1.0).contains(&self.learning_rate),
            "learning_rate",
            "must be finite and between 0 and 1",
        );
        check(
            self.discount.is_finite() && (0.0..=0.9999).contains(&self.discount),
            "discount",
            "must be finite and between 0 and 0.9999",
        );
        check(
            self.q_tolerance.is_finite() && (0.0..=1e-08).contains(&self.q_tolerance),
            "q_tolerance",
            "must be finite and between 0 and 1e-08",
        );
        check(
            self.q_scale.is_finite() && (1.0..=10.0).contains(&self.q_scale),
            "q_scale",
            "must be finite and between 1 and 10",
        );
        check(
            self.q_level.is_finite() && (0.0..=1000000.0).contains(&self.q_level),
            "q_level",
            "must be finite and between 0 and 1000000.0",
        );
        check(
            self.bias_bid.is_finite() && (0.0..=1.0).contains(&self.bias_bid),
            "bias_bid",
            "must be finite and between 0 and 1",
        );
        check(
            self.bias_q.is_finite() && (0.0..=1000000.0).contains(&self.bias_q),
            "bias_q",
            "must be finite and between 0 and 1000000.0",
        );
        check(
            self.bias_rest.is_finite() && (0.0..=1000000.0).contains(&self.bias_rest),
            "bias_rest",
            "must be finite and between 0 and 1000000.0",
        );
        check(
            self.epsilon.is_finite() && (0.0..=1.0).contains(&self.epsilon),
            "epsilon",
            "must be finite and between 0 and 1",
        );
        check(
            self.beta.is_finite() && (0.0..=1.0).contains(&self.beta),
            "beta",
            "must be finite and between 0 and 1",
        );
        check(
            (1..=u32::MAX).contains(&self.downward_at),
            "downward_at",
            "must be positive",
        );
        check(
            self.downward_chi.is_finite() && (0.0..=1.0).contains(&self.downward_chi),
            "downward_chi",
            "must be finite and between 0 and 1",
        );
        check(
            self.downward_beta.is_finite() && (0.0..=1.0).contains(&self.downward_beta),
            "downward_beta",
            "must be finite and between 0 and 1",
        );
        check(
            self.downward_gap.is_finite() && (0.0..=1000000.0).contains(&self.downward_gap),
            "downward_gap",
            "must be finite and between 0 and 1000000.0",
        );
        check(
            (1..=100000000).contains(&self.horizon),
            "horizon",
            "must be finite and between 1 and 100000000",
        );
        check(
            (1..=100000000).contains(&self.window),
            "window",
            "must be finite and between 1 and 100000000",
        );
        check(
            (1..=1000000).contains(&self.periods_per_tick),
            "periods_per_tick",
            "must be finite and between 1 and 1000000",
        );
        check(
            self.learning_rate > 0.0,
            "learning_rate",
            "must be positive",
        );
        check(
            self.reserve <= f64::from(self.bids) / (f64::from(self.bids) + 1.0),
            "reserve",
            "must not exceed highest positive bid",
        );
        check(
            self.window <= self.horizon,
            "window",
            "must not exceed horizon",
        );
        check(
            self.update != Update::All || self.feedback == Feedback::RivalBids,
            "feedback",
            "all-action updates require rival_bids",
        );
        if self.q_init == QInit::Biased && (2..=99).contains(&self.bids) {
            check(
                (1..=self.bids).any(|k| {
                    (f64::from(k) / (f64::from(self.bids) + 1.0) - self.bias_bid).abs() <= 1e-12
                }),
                "bias_bid",
                "must be a positive grid action",
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
pub fn schema() -> Vec<Param> {
    use Apply::Reset;
    vec![
        Param::integer("Auction", "bidders", "Bidders", (2, 3), Reset),
        Param::integer("Bids", "bids", "Bids", (2, 99), Reset),
        Param::choice(
            "Auction",
            "auction",
            "Auction",
            &[
                ("first_price", "FirstPrice"),
                ("second_price", "SecondPrice"),
                ("mixture", "Mixture"),
            ],
            Reset,
        ),
        Param::number(
            "Auction",
            "auction_alpha",
            "Auction alpha",
            (1.0, 2.0, 0.001),
            Reset,
        )
        .shown_if("auction", "mixture")
        .with_help("Active when auction is mixture; applies at reset."),
        Param::number("Auction", "reserve", "Reserve", (0.0, 0.99, 0.001), Reset),
        Param::choice(
            "Auction",
            "reserve_payment",
            "Reserve payment",
            &[("floor", "Floor"), ("eligibility_only", "EligibilityOnly")],
            Reset,
        ),
        Param::integer("Bids", "out_bids", "Out bids", (0, 16), Reset),
        Param::choice(
            "Auction",
            "fringe",
            "Fringe",
            &[("none", "None"), ("uniform", "Uniform")],
            Reset,
        ),
        Param::number(
            "Learning",
            "learning_rate",
            "Learning rate",
            (0.0, 1.0, 0.001),
            Reset,
        ),
        Param::number(
            "Learning",
            "discount",
            "Discount",
            (0.0, 0.9999, 0.001),
            Reset,
        ),
        Param::choice(
            "Information",
            "feedback",
            "Feedback",
            &[("outcome", "Outcome"), ("rival_bids", "RivalBids")],
            Reset,
        ),
        Param::choice(
            "Information",
            "update",
            "Update",
            &[("chosen", "Chosen"), ("all", "All")],
            Reset,
        ),
        Param::choice(
            "Auction",
            "auction_ties",
            "Auction ties",
            &[("sampled", "Sampled"), ("expected", "Expected")],
            Reset,
        ),
        Param::choice(
            "Information",
            "hindsight_ties",
            "Hindsight ties",
            &[("expected", "Expected"), ("realized", "Realized")],
            Reset,
        )
        .shown_if("update", "all")
        .with_help("Active when update is all; applies at reset."),
        Param::choice(
            "Learning",
            "greedy_ties",
            "Greedy ties",
            &[
                ("lowest", "Lowest"),
                ("highest", "Highest"),
                ("random", "Random"),
                ("incumbent", "Incumbent"),
            ],
            Reset,
        ),
        Param::number(
            "Learning",
            "q_tolerance",
            "Q tolerance",
            (0.0, 1e-08, 1e-09),
            Reset,
        ),
        Param::choice(
            "Learning",
            "q_init",
            "Q init",
            &[
                ("optimistic", "Optimistic"),
                ("constant", "Constant"),
                ("biased", "Biased"),
            ],
            Reset,
        ),
        Param::choice(
            "Learning",
            "optimism",
            "Optimism",
            &[("discounted", "Discounted"), ("stage", "Stage")],
            Reset,
        )
        .shown_if("q_init", "optimistic")
        .with_help("Active when q_init is optimistic; applies at reset."),
        Param::number("Learning", "q_scale", "Q scale", (1.0, 10.0, 0.001), Reset)
            .shown_if("q_init", "optimistic")
            .with_help("Active when q_init is optimistic; applies at reset."),
        Param::number(
            "Learning",
            "q_level",
            "Q level",
            (0.0, 1000000.0, 0.001),
            Reset,
        )
        .shown_if("q_init", "constant")
        .with_help("Active when q_init is constant; applies at reset."),
        Param::number("Learning", "bias_bid", "Bias bid", (0.0, 1.0, 0.001), Reset)
            .shown_if("q_init", "biased")
            .with_help("Active when q_init is biased; applies at reset."),
        Param::number(
            "Learning",
            "bias_q",
            "Bias q",
            (0.0, 1000000.0, 0.001),
            Reset,
        )
        .shown_if("q_init", "biased")
        .with_help("Active when q_init is biased; applies at reset."),
        Param::number(
            "Learning",
            "bias_rest",
            "Bias rest",
            (0.0, 1000000.0, 0.001),
            Reset,
        )
        .shown_if("q_init", "biased")
        .with_help("Active when q_init is biased; applies at reset."),
        Param::choice(
            "Exploration",
            "exploration",
            "Exploration",
            &[("decaying", "Decaying"), ("constant", "Constant")],
            Reset,
        ),
        Param::number(
            "Exploration",
            "epsilon",
            "Epsilon",
            (0.0, 1.0, 0.001),
            Reset,
        ),
        Param::number("Exploration", "beta", "Beta", (0.0, 1.0, 0.001), Reset)
            .shown_if("exploration", "decaying")
            .with_help("Active when exploration is decaying; applies at reset."),
        Param::choice(
            "Exploration",
            "exploration_set",
            "Exploration set",
            &[
                ("all", "All"),
                ("other", "Other"),
                ("neighbors", "Neighbors"),
            ],
            Reset,
        ),
        Param::choice(
            "Exploration",
            "neighbor_boundary",
            "Neighbor boundary",
            &[("available", "Available"), ("clamp", "Clamp")],
            Reset,
        )
        .shown_if("exploration_set", "neighbors")
        .with_help("Active when exploration_set is neighbors; applies at reset."),
        Param::choice(
            "Session",
            "period_origin",
            "Period origin",
            &[("zero", "Zero"), ("one", "One")],
            Reset,
        ),
        Param::choice(
            "Session",
            "convergence_phase",
            "Convergence phase",
            &[("post_update", "PostUpdate"), ("pre_update", "PreUpdate")],
            Reset,
        ),
        Param::choice(
            "Exploration",
            "downward_trigger",
            "Downward trigger",
            &[("off", "Off"), ("stable", "Stable"), ("period", "Period")],
            Reset,
        ),
        Param::integer(
            "Exploration",
            "downward_at",
            "Downward at",
            (1, u32::MAX),
            Reset,
        )
        .shown_if("downward_trigger", "period")
        .with_help("Active when downward_trigger is period; applies at reset."),
        Param::choice(
            "Exploration",
            "downward_clock",
            "Downward clock",
            &[("activation", "Activation"), ("global", "Global")],
            Reset,
        ),
        Param::number(
            "Exploration",
            "downward_chi",
            "Downward chi",
            (0.0, 1.0, 0.001),
            Reset,
        ),
        Param::number(
            "Exploration",
            "downward_beta",
            "Downward beta",
            (0.0, 1.0, 0.001),
            Reset,
        ),
        Param::number(
            "Exploration",
            "downward_gap",
            "Downward gap",
            (0.0, 1000000.0, 0.001),
            Reset,
        ),
        Param::integer("Session", "horizon", "Horizon", (1, 100000000), Reset),
        Param::integer("Session", "window", "Window", (1, 100000000), Reset),
        Param::integer(
            "Session",
            "periods_per_tick",
            "Periods per tick",
            (1, 1000000),
            Reset,
        ),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};
    #[test]
    fn oversized_grid_returns_field_error_without_overflow_or_membership_search() {
        let c = AuctionsConfig {
            bids: u32::MAX,
            q_init: QInit::Biased,
            ..Default::default()
        };
        assert!(c.validate().unwrap_err().iter().any(|e| e.field == "bids"));
    }
    #[test]
    fn all_action_learning_requires_observable_rival_bids() {
        let c = AuctionsConfig {
            update: Update::All,
            ..Default::default()
        };
        assert!(c
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "feedback"));
    }
    #[test]
    fn finite_ranges_validate_unused_fields_and_biased_requires_canonical_action() {
        let c = AuctionsConfig {
            q_scale: f64::NAN,
            q_level: f64::INFINITY,
            window: 1_000_001,
            ..Default::default()
        };
        let fields: Vec<_> = c
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert!(fields.contains(&"q_scale".into()));
        assert!(fields.contains(&"q_level".into()));
        assert!(fields.contains(&"window".into()));
        let c = AuctionsConfig {
            q_init: QInit::Biased,
            bias_bid: 0.41,
            ..Default::default()
        };
        assert_eq!(c.validate().unwrap_err()[0].field, "bias_bid");
    }
    #[test]
    fn schema_round_trips_and_every_field_applies_at_reset() {
        let config = ModelConfig::Auctions(AuctionsConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
