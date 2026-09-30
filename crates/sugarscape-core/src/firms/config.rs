//! The Emergence of Firms' parameters: Axtell's (1999) base case, every
//! variation of his §4, the 2013 paper's parameterization, and every detail
//! the paper leaves open as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How preferences for income against leisure are distributed (A99 §4.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preferences {
    /// Cobb–Douglas, θ ~ U[0, 1] (the base case).
    Uniform,
    /// θ ~ U[0.25, 0.75].
    Middle,
    /// θ triangular on [0, 1] with mode 0.5.
    Triangular,
    /// θ triangular on [0, 1] with mode 0.75.
    TriangularHigh,
    /// θ normal (mean 0.5, variance ½) truncated to [0, 1]; the mean is our
    /// reading.
    Normal,
    /// θ ~ Beta(1, 2) (mean ⅓; the parameters' order is our reading).
    Beta,
    /// Every θ = `theta`.
    Fixed,
    /// CES: δ ~ U[0, 1], ρ ~ U[`rho`, `rho_max`].
    Ces,
}

/// The CES exponent's sign convention (A99 p. 73's printed formula against
/// its text's limits).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CesSign {
    /// (δ y^−ρ + (1−δ) l^−ρ)^(−1/ρ): linear at ρ = −1, Cobb–Douglas as
    /// ρ → 0, Leontief as ρ grows — the limits the text states.
    Text,
    /// (δ y^ρ + (1−δ) l^ρ)^(1/ρ), as printed.
    Printed,
}

/// Where an agent looks for other firms (A99 §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// ν fixed random agents, assigned at the start: their current firms.
    Friends,
    /// ν firms drawn uniformly at each activation, not the agent's own.
    RandomFirms,
}

/// Who is activated in a period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activation {
    /// Drawn uniformly with replacement (A99: "random activation").
    Random,
    /// Each agent at most once, in random order.
    Uniform,
}

/// What an agent takes as the other members' effort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OthersEffort {
    /// Inferred from last period's output (A99 pp. 9, 26; A13 p. 8).
    LastPeriod,
    /// Their current efforts.
    Live,
}

/// How an agent finds its best effort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortSearch {
    /// The optimum (the limit of A99's line search).
    Exact,
    /// The best of `grid_steps` + 1 evenly spaced efforts.
    Grid,
}

/// How a firm divides its output (A99 §4.7–4.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pay {
    /// Equal shares (the base case).
    Equal,
    /// Shares ∝ `seniority_base`^−rank, rank 1 the longest-serving member.
    Seniority,
    /// Base pay Φ plus an equal share of what output exceeds the base pay.
    Base,
}

/// Where sticky effort's window and groping apply (A99 §4.6 does not say).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustScope {
    /// To effort in any firm: a start-up or a firm joined too.
    Everywhere,
    /// Only to effort in the agent's own firm; in a new one it chooses freely.
    OwnFirm,
}

/// Which way seniority shares run (A99 §4.7: "i = 1 referring to the firm
/// founder").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeniorityOrder {
    /// The longest-serving member has the largest share (the text).
    SeniorFirst,
    /// The newest member has the largest share.
    JuniorFirst,
}

/// Whose singleton income sets base pay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasePay {
    /// Each agent's own.
    Own,
    /// The median agent's (θ = ½).
    Median,
    /// The mean over the population.
    Mean,
}

/// A99 §4.1's random behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandomBehavior {
    None,
    /// Stay, move or start up at random (a random option), then the best
    /// effort.
    Choices,
    /// The best option at a random effort.
    Effort,
}

/// How agents start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Initial {
    /// Every agent alone (the base case).
    Alone,
    /// Groups of geometrically distributed size, mean 4 (our reading).
    RandomGroups,
    /// One firm of everyone.
    OneFirm,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FirmsConfig {
    pub agents: u32,
    /// Output a·E + b·E^β; each drawn per firm at founding between the value
    /// and its maximum when the maximum is above it (otherwise fixed).
    pub a: f64,
    pub a_max: f64,
    pub b: f64,
    pub b_max: f64,
    pub beta: f64,
    pub beta_max: f64,
    pub preferences: Preferences,
    pub theta: f64,
    pub rho: f64,
    pub rho_max: f64,
    pub ces_sign: CesSign,
    pub network: Network,
    /// ν, or drawn per agent between the two.
    pub neighbors: u32,
    pub neighbors_max: u32,
    pub activation: Activation,
    /// Activations a period ÷ agents.
    pub activation_rate: f64,
    pub others_effort: OthersEffort,
    pub effort_search: EffortSearch,
    pub grid_steps: u32,
    /// Sticky effort: a new effort within ±window/2 of the current one.
    pub effort_window: f64,
    pub groping: bool,
    pub adjust_scope: AdjustScope,
    /// λ: moves only after wanting to more than λ times.
    pub loyalty: u32,
    pub loyalty_max: u32,
    pub pay: Pay,
    pub seniority_base: f64,
    pub seniority_order: SeniorityOrder,
    pub base_pay: BasePay,
    pub base_share: f64,
    /// φ: a firm admits only θ ≥ φ·θ of its longest-serving member.
    pub hiring: f64,
    pub hiring_max: f64,
    pub random_behavior: RandomBehavior,
    pub initial: Initial,
    /// The size, growth and lifetime records start after `burn_in` periods;
    /// sizes are sampled every `sample_every` periods.
    pub burn_in: u32,
    pub sample_every: u32,
    pub stop_at: u32,
}

impl Default for FirmsConfig {
    /// Axtell's base case (A99 Table 2).
    fn default() -> Self {
        FirmsConfig {
            agents: 1000,
            a: 1.0,
            a_max: 0.0,
            b: 1.0,
            b_max: 0.0,
            beta: 2.0,
            beta_max: 0.0,
            preferences: Preferences::Uniform,
            theta: 0.75,
            rho: -1.0,
            rho_max: 0.0,
            ces_sign: CesSign::Text,
            network: Network::Friends,
            neighbors: 2,
            neighbors_max: 0,
            activation: Activation::Random,
            activation_rate: 1.0,
            others_effort: OthersEffort::LastPeriod,
            effort_search: EffortSearch::Exact,
            grid_steps: 100,
            effort_window: 1.0,
            groping: false,
            adjust_scope: AdjustScope::Everywhere,
            loyalty: 0,
            loyalty_max: 0,
            pay: Pay::Equal,
            seniority_base: 2.0,
            seniority_order: SeniorityOrder::SeniorFirst,
            base_pay: BasePay::Own,
            base_share: 0.5,
            hiring: 0.0,
            hiring_max: 0.0,
            random_behavior: RandomBehavior::None,
            initial: Initial::Alone,
            burn_in: 500,
            sample_every: 1,
            stop_at: 5000,
        }
    }
}

impl FirmsConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (2..=200_000).contains(&self.agents),
            "agents",
            "must be between 2 and 200000",
        );
        check(
            (0.0..=10.0).contains(&self.a) && (0.0..=10.0).contains(&self.a_max),
            "a",
            "a and its maximum must be between 0 and 10",
        );
        check(
            (0.0..=10.0).contains(&self.b) && (0.0..=10.0).contains(&self.b_max),
            "b",
            "b and its maximum must be between 0 and 10",
        );
        check(
            (1.0..=3.0).contains(&self.beta) && (0.0..=3.0).contains(&self.beta_max),
            "beta",
            "β must be between 1 and 3 (its maximum at most 3)",
        );
        check(self.a + self.b > 0.0, "b", "a and b cannot both be 0");
        check(unit(self.theta), "theta", "must be between 0 and 1");
        check(
            (-1.0..=10.0).contains(&self.rho) && (-1.0..=10.0).contains(&self.rho_max),
            "rho",
            "ρ and its maximum must be between −1 and 10",
        );
        check(
            (1..=100).contains(&self.neighbors) && self.neighbors_max <= 100,
            "neighbors",
            "ν must be between 1 and 100 (its maximum at most 100)",
        );
        check(
            self.activation_rate > 0.0 && self.activation_rate <= 10.0,
            "activation_rate",
            "must be above 0 and at most 10",
        );
        check(
            (1..=10_000).contains(&self.grid_steps),
            "grid_steps",
            "must be between 1 and 10000",
        );
        check(
            self.effort_window > 0.0 && self.effort_window <= 1.0,
            "effort_window",
            "must be above 0 and at most 1",
        );
        check(
            self.loyalty <= 1000 && self.loyalty_max <= 1000,
            "loyalty",
            "λ and its maximum must be at most 1000",
        );
        check(
            (1.0..=100.0).contains(&self.seniority_base),
            "seniority_base",
            "must be between 1 and 100",
        );
        check(
            unit(self.base_share),
            "base_share",
            "must be between 0 and 1",
        );
        check(
            unit(self.hiring) && unit(self.hiring_max),
            "hiring",
            "φ and its maximum must be between 0 and 1",
        );
        check(
            (1..=100_000).contains(&self.sample_every),
            "sample_every",
            "must be between 1 and 100000",
        );
        check(self.burn_in <= 100_000, "burn_in", "must be at most 100000");
        check(self.stop_at <= 100_000, "stop_at", "must be at most 100000");
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &FirmsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("preferences", self.preferences == next.preferences),
            ("theta", self.theta == next.theta),
            ("rho", (self.rho, self.rho_max) == (next.rho, next.rho_max)),
            ("ces_sign", self.ces_sign == next.ces_sign),
            ("network", self.network == next.network),
            (
                "neighbors",
                (self.neighbors, self.neighbors_max) == (next.neighbors, next.neighbors_max),
            ),
            (
                "loyalty",
                (self.loyalty, self.loyalty_max) == (next.loyalty, next.loyalty_max),
            ),
            ("initial", self.initial == next.initial),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }

    /// Whether a firm's output parameters are drawn per firm.
    pub fn drawn(lo: f64, hi: f64) -> bool {
        hi > lo
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Agents", "agents", "Agents", (2, 200_000), Reset),
        Param::choice(
            "Agents",
            "initial",
            "Start",
            &[
                ("alone", "Everyone alone (Axtell)"),
                ("random_groups", "In random groups (mean size 4)"),
                ("one_firm", "All in one firm"),
            ],
            Reset,
        ),
        Param::number(
            "Production",
            "a",
            "Constant returns a",
            (0.0, 10.0, 0.05),
            Live,
        ),
        Param::number(
            "Production",
            "a_max",
            "a up to (0: fixed)",
            (0.0, 10.0, 0.05),
            Live,
        )
        .with_help("Above a: each firm draws its a at founding."),
        Param::number(
            "Production",
            "b",
            "Increasing returns b",
            (0.0, 10.0, 0.05),
            Live,
        ),
        Param::number(
            "Production",
            "b_max",
            "b up to (0: fixed)",
            (0.0, 10.0, 0.05),
            Live,
        )
        .with_help("Above b: each firm draws its b at founding (Axtell §4.2)."),
        Param::number("Production", "beta", "Exponent β", (1.0, 3.0, 0.05), Live),
        Param::number(
            "Production",
            "beta_max",
            "β up to (0: fixed)",
            (0.0, 3.0, 0.05),
            Live,
        )
        .with_help("Above β: each firm draws its β at founding (our reading of §4.2)."),
        Param::choice(
            "Preferences",
            "preferences",
            "Preferences",
            &[
                ("uniform", "Cobb–Douglas, θ uniform on [0, 1] (Axtell)"),
                ("middle", "θ uniform on [0.25, 0.75]"),
                ("triangular", "θ triangular, mode 0.5"),
                ("triangular_high", "θ triangular, mode 0.75"),
                ("normal", "θ normal (0.5, variance ½), truncated"),
                ("beta", "θ Beta(1, 2)"),
                ("fixed", "Every θ the same"),
                ("ces", "CES"),
            ],
            Reset,
        ),
        Param::number("Preferences", "theta", "θ", (0.0, 1.0, 0.01), Reset)
            .shown_if("preferences", "fixed"),
        Param::number("Preferences", "rho", "ρ from", (-1.0, 10.0, 0.1), Reset)
            .shown_if("preferences", "ces"),
        Param::number(
            "Preferences",
            "rho_max",
            "ρ up to (0: fixed)",
            (0.0, 10.0, 0.1),
            Reset,
        )
        .shown_if("preferences", "ces"),
        Param::choice(
            "Preferences",
            "ces_sign",
            "CES exponent",
            &[
                ("text", "As the text's limits read (−ρ)"),
                ("printed", "As printed (+ρ)"),
            ],
            Reset,
        )
        .shown_if("preferences", "ces"),
        Param::choice(
            "Network",
            "network",
            "Agents look at",
            &[
                ("friends", "Their friends' firms (Axtell)"),
                ("random_firms", "Random firms, each time"),
            ],
            Reset,
        ),
        Param::integer(
            "Network",
            "neighbors",
            "Friends or firms ν",
            (1, 100),
            Reset,
        ),
        Param::integer(
            "Network",
            "neighbors_max",
            "ν up to (0: fixed)",
            (0, 100),
            Reset,
        ),
        Param::choice(
            "Decisions",
            "activation",
            "Activation",
            &[
                ("random", "Random, with replacement (Axtell)"),
                ("uniform", "Each agent once, in random order"),
            ],
            Live,
        ),
        Param::number(
            "Decisions",
            "activation_rate",
            "Activations a period ÷ agents",
            (0.01, 10.0, 0.01),
            Live,
        )
        .with_help("Axtell 1999: 1 (a period is 1 000 activations of 1 000 agents); 2013: 0.04."),
        Param::choice(
            "Decisions",
            "others_effort",
            "Others' effort",
            &[
                ("last_period", "From last period's output (the text)"),
                ("live", "Their current efforts"),
            ],
            Live,
        ),
        Param::choice(
            "Decisions",
            "effort_search",
            "Best effort",
            &[
                ("exact", "The optimum"),
                ("grid", "The best of a grid (a coarse line search)"),
            ],
            Live,
        ),
        Param::integer("Decisions", "grid_steps", "Grid steps", (1, 10_000), Live)
            .shown_if("effort_search", "grid"),
        Param::number(
            "Decisions",
            "effort_window",
            "Effort changes by at most (window)",
            (0.01, 1.0, 0.01),
            Live,
        )
        .with_help("1: free. Axtell's sticky effort: 0.10 (±0.05)."),
        Param::bool(
            "Decisions",
            "groping",
            "Grope for effort (one random try)",
            Live,
        ),
        Param::choice(
            "Decisions",
            "adjust_scope",
            "Sticky effort and groping apply",
            &[
                ("everywhere", "In any firm, joined or founded too"),
                ("own_firm", "Only in the agent's own firm"),
            ],
            Live,
        ),
        Param::integer("Decisions", "loyalty", "Loyalty λ", (0, 1000), Reset),
        Param::integer(
            "Decisions",
            "loyalty_max",
            "λ up to (0: fixed)",
            (0, 1000),
            Reset,
        ),
        Param::choice(
            "Decisions",
            "random_behavior",
            "Random behavior (§4.1)",
            &[
                ("none", "None (Axtell)"),
                ("choices", "Random choices"),
                ("effort", "Random effort"),
            ],
            Live,
        ),
        Param::choice(
            "Pay",
            "pay",
            "Output is shared",
            &[
                ("equal", "Equally (Axtell)"),
                ("seniority", "By seniority"),
                ("base", "Base pay plus a bonus"),
            ],
            Live,
        ),
        Param::number(
            "Pay",
            "seniority_base",
            "Seniority p (shares ∝ p^−rank)",
            (1.0, 100.0, 0.5),
            Live,
        )
        .shown_if("pay", "seniority"),
        Param::choice(
            "Pay",
            "seniority_order",
            "The largest share goes to",
            &[
                ("senior_first", "The longest-serving (the text)"),
                ("junior_first", "The newest"),
            ],
            Live,
        )
        .shown_if("pay", "seniority"),
        Param::choice(
            "Pay",
            "base_pay",
            "Base pay from",
            &[
                ("own", "Each agent's own singleton income"),
                ("median", "The median agent's"),
                ("mean", "The mean agent's"),
            ],
            Live,
        )
        .shown_if("pay", "base"),
        Param::number(
            "Pay",
            "base_share",
            "Base pay share",
            (0.0, 1.0, 0.05),
            Live,
        )
        .shown_if("pay", "base"),
        Param::number("Pay", "hiring", "Hiring standard φ", (0.0, 1.0, 0.05), Live),
        Param::number(
            "Pay",
            "hiring_max",
            "φ up to (0: fixed)",
            (0.0, 1.0, 0.05),
            Live,
        ),
        Param::integer(
            "Measurement",
            "burn_in",
            "Records start after period",
            (0, 100_000),
            Live,
        ),
        Param::integer(
            "Measurement",
            "sample_every",
            "Sample sizes every",
            (1, 100_000),
            Live,
        )
        .with_help("Axtell sampled less often than the longest firm lifetime."),
        Param::integer(
            "Measurement",
            "stop_at",
            "Stop after period",
            (0, 100_000),
            Live,
        )
        .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_axtells_base_case() {
        let c = FirmsConfig::default();
        assert_eq!(
            (c.agents, c.a, c.b, c.beta, c.neighbors),
            (1000, 1.0, 1.0, 2.0, 2)
        );
        assert_eq!(
            (
                c.preferences,
                c.network,
                c.activation,
                c.others_effort,
                c.pay
            ),
            (
                Preferences::Uniform,
                Network::Friends,
                Activation::Random,
                OthersEffort::LastPeriod,
                Pay::Equal
            )
        );
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = FirmsConfig {
            agents: 1,
            beta: 0.5,
            theta: 2.0,
            neighbors: 0,
            activation_rate: 0.0,
            effort_window: 0.0,
            seniority_base: 0.5,
            hiring: 1.5,
            sample_every: 0,
            stop_at: 200_000,
            ..FirmsConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            [
                "agents",
                "beta",
                "theta",
                "neighbors",
                "activation_rate",
                "effort_window",
                "seniority_base",
                "hiring",
                "sample_every",
                "stop_at"
            ]
        );
        // A maximum at or below its value means fixed.
        assert!(FirmsConfig {
            b: 1.0,
            b_max: 0.5,
            ..FirmsConfig::default()
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn the_population_and_network_change_only_on_reset() {
        let next = FirmsConfig {
            neighbors: 4,
            beta: 1.8,
            ..FirmsConfig::default()
        };
        let changes = FirmsConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "neighbors");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Firms(FirmsConfig {
            agents: 50,
            ..FirmsConfig::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
