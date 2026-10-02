//! Algorithmic Collusion's parameters: Calvano, Calzolari, Denicolò and
//! Pastorello's (2020) baseline, every place where their paper and their
//! code differ, and the critics' tests (Asker, Fershtman & Pakes; Lambin;
//! Epivent & Lambin; den Boer, Meylahn & Schinkel) as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// The price grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grid {
    /// m prices evenly from p^N − ξ(p^M − p^N) to p^M + ξ(p^M − p^N) (CCDP).
    Calvano,
    /// m prices evenly from p^N − (1 + ξ)ζ to p^N + (1 + ξ)ζ, ζ = p^M − p^N:
    /// centered on the Nash price (den Boer, Meylahn & Schinkel's Ã).
    Symmetric,
    /// m prices evenly from 1.25 to `below_top` (Epivent & Lambin, App. C).
    BelowNash,
}

/// How a firm explores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exploration {
    /// ε = e^(−βt): with probability ε a price drawn uniformly (CCDP eq. 7).
    Decaying,
    /// ε fixed at `epsilon`.
    Constant,
    /// Choice probabilities ∝ exp((Q − max Q)/T), T starting at `temperature`
    /// and multiplied by (1 − `cooling`) each period (the code's type 2).
    Boltzmann,
    /// ε = 1 for `explore_for` periods, then 0 (Lambin 2024).
    TwoPhase,
}

/// Which Q-values a firm updates each period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Only the price it charged (CCDP eq. 4).
    Asynchronous,
    /// Every price, toward the profit it would have earned against the
    /// rivals' actual prices and the state that would have followed (Asker,
    /// Fershtman & Pakes).
    Synchronous,
}

/// The starting Q-table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QInit {
    /// The discounted profit against a uniformly random rival, the same in
    /// every state (CCDP eq. 8).
    Calvano,
    Zero,
    /// Each cell uniform on [`q_low`, `q_high`] (Asker, Fershtman & Pakes's
    /// optimistic start with their 10–20; Lambin's theory).
    Random,
}

/// How the greedy price is chosen among equal Q-values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ties {
    /// The lowest price (CCDP p. 3275).
    Lowest,
    /// At random, and only when the greedy price's own value falls (the code).
    Random,
}

/// The random numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RngKind {
    /// The project's generator.
    Ours,
    /// Numerical Recipes' RAN2, seeded as the authors' code seeds session
    /// number `seed`: sessions match the code's period for period.
    Calvano,
}

/// What counts as an equilibrium after convergence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquilibriumCheck {
    /// Each firm's price is a best response to the rivals' strategies, by
    /// value iteration on the true Q (the paper's description).
    BestResponse,
    /// No one-period deviation pays when everyone, the deviator included,
    /// returns to the learned strategies (the code).
    OneShot,
}

/// The deviation the impulse response applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Impulse {
    /// One period at the static best response (CCDP Fig. 4).
    BestResponseDown,
    /// One period at every other grid price (CCDP-A; Epivent & Lambin's Table 1).
    EveryPrice,
    /// One grid step up, the rival forced to match the next period on, the
    /// deviator regaining control after `invitation_hold` periods (Epivent
    /// & Lambin's Fig. 2).
    Invitation,
    /// One period one grid step up.
    Up,
}

/// Which state the static best response answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BestResponseTo {
    /// The rivals' prices at the pre-deviation state (the paper's words).
    Path,
    /// The rivals' prices at the state numbered by the cycle position (1, 2,
    /// …): the code passes the position where it means the state.
    Code,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CollusionConfig {
    /// n.
    pub firms: u32,
    /// m.
    pub prices: u32,
    pub grid: Grid,
    /// ξ.
    pub xi: f64,
    /// The top of the `below_nash` grid (Epivent & Lambin's text: 1.47).
    pub below_top: f64,
    /// cᵢ.
    pub cost: f64,
    /// Firm 2's cost, if different (CCDP-A's asymmetric firms).
    pub cost2: Option<f64>,
    /// aᵢ.
    pub quality: f64,
    /// a₀.
    pub outside: f64,
    /// μ.
    pub mu: f64,
    /// k: 0, 1 or 2 periods of everyone's prices.
    pub memory: u32,
    /// α.
    pub alpha: f64,
    /// β, per period.
    pub beta: f64,
    /// δ.
    pub delta: f64,
    pub exploration: Exploration,
    pub epsilon: f64,
    pub temperature: f64,
    pub cooling: f64,
    pub explore_for: u32,
    pub update: Update,
    pub q_init: QInit,
    pub q_low: f64,
    pub q_high: f64,
    pub ties: Ties,
    pub rng: RngKind,
    /// The stop when strategies never settle.
    pub cap: u32,
    /// Periods of unchanged strategies that count as convergence.
    pub window: u32,
    pub equilibrium_check: EquilibriumCheck,
    pub impulse: Impulse,
    pub best_response_to: BestResponseTo,
    pub invitation_hold: u32,
    /// Periods a tick runs: a session (about 2 × 10⁶ periods) takes a few
    /// thousand ticks, and each charted point summarizes one tick.
    pub periods_per_tick: u32,
}

impl Default for CollusionConfig {
    /// CCDP's baseline (p. 3274): two firms, 15 prices, one period of memory.
    fn default() -> Self {
        CollusionConfig {
            firms: 2,
            prices: 15,
            grid: Grid::Calvano,
            xi: 0.1,
            below_top: 1.47,
            cost: 1.0,
            cost2: None,
            quality: 2.0,
            outside: 0.0,
            mu: 0.25,
            memory: 1,
            alpha: 0.15,
            beta: 4e-6,
            delta: 0.95,
            exploration: Exploration::Decaying,
            epsilon: 0.05,
            temperature: 1000.0,
            cooling: 1e-5,
            explore_for: 1000,
            update: Update::Asynchronous,
            q_init: QInit::Calvano,
            q_low: 10.0,
            q_high: 20.0,
            ties: Ties::Lowest,
            rng: RngKind::Ours,
            cap: 1_000_000_000,
            window: 100_000,
            equilibrium_check: EquilibriumCheck::BestResponse,
            impulse: Impulse::BestResponseDown,
            best_response_to: BestResponseTo::Path,
            invitation_hold: 5,
            periods_per_tick: 1000,
        }
    }
}

/// The bottom of the `below_nash` grid (Epivent & Lambin, App. C).
pub const BELOW_BOTTOM: f64 = 1.25;
/// The most Q-values a world may hold (states × prices × firms).
pub const Q_BUDGET: u64 = 1 << 24;

impl CollusionConfig {
    /// The number of states, m^(n·k).
    pub fn states(&self) -> u64 {
        u64::from(self.prices).saturating_pow(self.firms * self.memory)
    }

    /// Firm `i`'s marginal cost.
    pub fn cost_of(&self, i: usize) -> f64 {
        match (i, self.cost2) {
            (1, Some(c)) => c,
            _ => self.cost,
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (2..=4).contains(&self.firms),
            "firms",
            "must be between 2 and 4",
        );
        check(
            (2..=100).contains(&self.prices),
            "prices",
            "must be between 2 and 100",
        );
        check(
            (0.0..=1.0).contains(&self.xi),
            "xi",
            "must be between 0 and 1",
        );
        check(
            self.below_top > BELOW_BOTTOM && self.below_top <= 10.0,
            "below_top",
            "must be above 1.25 and at most 10",
        );
        check(
            (0.0..=10.0).contains(&self.cost),
            "cost",
            "must be between 0 and 10",
        );
        check(
            self.cost2.is_none_or(|c| (0.0..=10.0).contains(&c)),
            "cost2",
            "must be between 0 and 10",
        );
        check(
            self.quality > self.cost && self.quality <= 20.0,
            "quality",
            "must be above the cost and at most 20",
        );
        check(
            (-10.0..=10.0).contains(&self.outside),
            "outside",
            "must be between −10 and 10",
        );
        check(
            (0.01..=10.0).contains(&self.mu),
            "mu",
            "must be between 0.01 and 10",
        );
        check(self.memory <= 2, "memory", "must be 0, 1 or 2");
        check(
            self.states() * u64::from(self.prices) * u64::from(self.firms) <= Q_BUDGET,
            "memory",
            "too many states for this many firms and prices (states × prices × firms must be at most 2^24)",
        );
        check(
            u64::from(self.prices).saturating_pow(self.firms) * u64::from(self.firms) <= Q_BUDGET,
            "prices",
            "too many price profiles for this many firms (prices^firms × firms must be at most 2^24)",
        );
        check(
            self.alpha > 0.0 && self.alpha <= 1.0,
            "alpha",
            "must be above 0 and at most 1",
        );
        check(
            (0.0..=1.0).contains(&self.beta),
            "beta",
            "must be between 0 and 1",
        );
        check(
            (0.0..1.0).contains(&self.delta),
            "delta",
            "must be at least 0 and below 1",
        );
        check(
            (0.0..=1.0).contains(&self.epsilon),
            "epsilon",
            "must be between 0 and 1",
        );
        check(
            self.temperature > 0.0 && self.temperature <= 1e6,
            "temperature",
            "must be above 0 and at most 1000000",
        );
        check(
            (0.0..1.0).contains(&self.cooling),
            "cooling",
            "must be at least 0 and below 1",
        );
        check(
            self.q_low <= self.q_high && self.q_low.abs() <= 1e6 && self.q_high.abs() <= 1e6,
            "q_high",
            "must be at least q_low (both within ±1000000)",
        );
        check(self.cap >= 1, "cap", "must be at least 1");
        check(
            self.window >= 1 && self.window <= self.cap,
            "window",
            "must be at least 1 and at most the cap",
        );
        check(
            (1..=100).contains(&self.invitation_hold),
            "invitation_hold",
            "must be between 1 and 100",
        );
        check(
            (1..=1_000_000).contains(&self.periods_per_tick),
            "periods_per_tick",
            "must be between 1 and 1000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &CollusionConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("firms", self.firms == next.firms),
            ("prices", self.prices == next.prices),
            ("grid", self.grid == next.grid),
            ("xi", self.xi == next.xi),
            ("below_top", self.below_top == next.below_top),
            ("cost", self.cost == next.cost),
            ("cost2", self.cost2 == next.cost2),
            ("quality", self.quality == next.quality),
            ("outside", self.outside == next.outside),
            ("mu", self.mu == next.mu),
            ("memory", self.memory == next.memory),
            ("beta", self.beta == next.beta),
            ("exploration", self.exploration == next.exploration),
            ("epsilon", self.epsilon == next.epsilon),
            ("temperature", self.temperature == next.temperature),
            ("cooling", self.cooling == next.cooling),
            ("explore_for", self.explore_for == next.explore_for),
            ("update", self.update == next.update),
            ("q_init", self.q_init == next.q_init),
            ("q_low", self.q_low == next.q_low),
            ("q_high", self.q_high == next.q_high),
            ("ties", self.ties == next.ties),
            ("rng", self.rng == next.rng),
            ("cap", self.cap == next.cap),
            ("window", self.window == next.window),
            (
                "periods_per_tick",
                self.periods_per_tick == next.periods_per_tick,
            ),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Market", "firms", "Firms (n)", (2, 4), Reset),
        Param::number("Market", "cost", "Cost (c)", (0.0, 10.0, 0.05), Reset),
        Param::number("Market", "cost2", "Firm 2's cost", (0.0, 10.0, 0.05), Reset)
            .nullable()
            .with_help("Empty: the same as the others (CCDP-A's asymmetric firms set it lower)."),
        Param::number("Market", "quality", "Quality (a)", (0.0, 20.0, 0.05), Reset),
        Param::number("Market", "outside", "Outside good (a₀)", (-10.0, 10.0, 0.05), Reset),
        Param::number("Market", "mu", "Differentiation (μ)", (0.01, 10.0, 0.01), Reset)
            .with_help("Logit demand's horizontal differentiation: 0.25 in the baseline."),
        Param::integer("Prices", "prices", "Prices (m)", (2, 100), Reset),
        Param::choice(
            "Prices",
            "grid",
            "Grid",
            &[
                ("calvano", "Around Nash and monopoly (CCDP)"),
                ("symmetric", "Centered on Nash (den Boer et al.)"),
                ("below_nash", "From 1.25 up to Nash (Epivent & Lambin)"),
            ],
            Reset,
        ),
        Param::number("Prices", "xi", "Extension (ξ)", (0.0, 1.0, 0.05), Reset),
        Param::number("Prices", "below_top", "Top price", (1.26, 10.0, 0.01), Reset)
            .shown_if("grid", "below_nash")
            .with_help("Epivent & Lambin's text: 1.47; the Nash price is 1.47293."),
        Param::integer("Learning", "memory", "Memory (k)", (0, 2), Reset)
            .with_help("Periods of everyone's prices a firm conditions on. 0: one state, so no punishment is possible (Lambin's test keeps δ)."),
        Param::number("Learning", "alpha", "Learning rate (α)", (0.0025, 1.0, 0.0025), Live),
        Param::number("Learning", "delta", "Discount factor (δ)", (0.0, 0.99, 0.01), Live),
        Param::choice(
            "Learning",
            "update",
            "Update",
            &[
                ("asynchronous", "The price charged (CCDP)"),
                ("synchronous", "Every price (Asker, Fershtman & Pakes)"),
            ],
            Reset,
        ),
        Param::choice(
            "Learning",
            "q_init",
            "Starting Q",
            &[
                ("calvano", "Profit against a random rival (CCDP)"),
                ("zero", "Zero"),
                ("random", "Uniform between two values"),
            ],
            Reset,
        ),
        Param::number("Learning", "q_low", "Lowest", (-1000.0, 1000.0, 0.5), Reset)
            .shown_if("q_init", "random"),
        Param::number("Learning", "q_high", "Highest", (-1000.0, 1000.0, 0.5), Reset)
            .shown_if("q_init", "random"),
        Param::choice(
            "Learning",
            "ties",
            "Equal Q-values",
            &[
                ("lowest", "The lowest price (the paper)"),
                ("random", "At random (the code)"),
            ],
            Reset,
        ),
        Param::choice(
            "Exploration",
            "exploration",
            "Exploration",
            &[
                ("decaying", "ε = e^(−βt) (CCDP)"),
                ("constant", "A constant ε"),
                ("boltzmann", "Boltzmann (the code's type 2)"),
                ("two_phase", "All at first, then none (Lambin)"),
            ],
            Reset,
        ),
        Param::number("Exploration", "beta", "Decay (β)", (0.0, 1.0, 1e-6), Reset)
            .shown_if("exploration", "decaying"),
        Param::number("Exploration", "epsilon", "ε", (0.0, 1.0, 0.005), Reset)
            .shown_if("exploration", "constant"),
        Param::number("Exploration", "temperature", "Starting temperature", (0.001, 1e6, 1.0), Reset)
            .shown_if("exploration", "boltzmann"),
        Param::number("Exploration", "cooling", "Cooling", (0.0, 0.999, 1e-6), Reset)
            .shown_if("exploration", "boltzmann"),
        Param::integer("Exploration", "explore_for", "Periods of exploring", (0, 10_000_000), Reset)
            .shown_if("exploration", "two_phase"),
        Param::choice(
            "Session",
            "rng",
            "Random numbers",
            &[
                ("ours", "The playground's"),
                ("calvano", "The authors' (session number = seed)"),
            ],
            Reset,
        )
        .with_help("The authors' generator and seeding: a session runs period for period as in their code. Seed s is their session s, wrapping every 10⁶ sessions."),
        Param::integer("Session", "cap", "Stop at most at period", (1, 4_000_000_000), Reset)
            .with_help("The paper: 10⁹. The code: 1.25 × 10⁹."),
        Param::integer("Session", "window", "Converged after", (1, 10_000_000), Reset)
            .with_help("Periods of unchanged strategies (CCDP: 100 000)."),
        Param::choice(
            "Analysis",
            "equilibrium_check",
            "Equilibrium",
            &[
                ("best_response", "A best response (the paper)"),
                ("one_shot", "No one-period deviation pays (the code)"),
            ],
            Live,
        ),
        Param::choice(
            "Analysis",
            "impulse",
            "Deviation",
            &[
                ("best_response_down", "To the static best response (CCDP Fig. 4)"),
                ("every_price", "To every price (Epivent & Lambin's Table 1)"),
                ("invitation", "An invitation: up a step, the rival made to follow"),
                ("up", "Up one step"),
            ],
            Live,
        ),
        Param::choice(
            "Analysis",
            "best_response_to",
            "Best response to",
            &[
                ("path", "The rival's price before the deviation (the paper)"),
                ("code", "The state numbered by the cycle position (the code)"),
            ],
            Live,
        )
        .shown_if("impulse", "best_response_down"),
        Param::integer("Analysis", "invitation_hold", "Deviator held for", (1, 100), Live)
            .shown_if("impulse", "invitation"),
        Param::integer("Session", "periods_per_tick", "Periods a tick", (1, 1_000_000), Reset)
            .with_help("A tick runs this many periods; the charts count ticks, each point summarizing one tick. 1 000: a session takes a few thousand ticks."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_calvano_s_baseline() {
        let c = CollusionConfig::default();
        assert_eq!((c.firms, c.prices, c.memory, c.states()), (2, 15, 1, 225));
        assert_eq!((c.alpha, c.beta, c.delta), (0.15, 4e-6, 0.95));
        assert_eq!(
            (c.ties, c.cap, c.window),
            (Ties::Lowest, 1_000_000_000, 100_000)
        );
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = CollusionConfig {
            firms: 5,
            prices: 1,
            xi: 2.0,
            mu: 0.0,
            alpha: 0.0,
            delta: 1.0,
            q_low: 5.0,
            q_high: 1.0,
            window: 0,
            ..CollusionConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["firms", "prices", "xi", "mu", "alpha", "delta", "q_high", "window"]
        );
    }

    #[test]
    fn big_tables_are_refused() {
        let c = CollusionConfig {
            firms: 4,
            memory: 2,
            ..CollusionConfig::default()
        };
        assert_eq!(c.validate().unwrap_err()[0].field, "memory");
        let ok = CollusionConfig {
            firms: 2,
            memory: 2,
            ..CollusionConfig::default()
        };
        assert_eq!(ok.states(), 50_625);
        assert!(ok.validate().is_ok());
        assert_eq!(
            CollusionConfig {
                memory: 0,
                ..CollusionConfig::default()
            }
            .states(),
            1
        );
    }

    #[test]
    fn payoff_tables_too_big_to_build_are_refused() {
        // Without memory the states are few, but the payoff table holds
        // prices^firms profiles: 4 firms of 100 prices would be 10⁸.
        let c = CollusionConfig {
            firms: 4,
            prices: 100,
            memory: 0,
            ..CollusionConfig::default()
        };
        assert_eq!(c.validate().unwrap_err()[0].field, "prices");
    }

    #[test]
    fn firm_two_may_have_its_own_cost() {
        let c = CollusionConfig {
            cost2: Some(0.75),
            ..CollusionConfig::default()
        };
        assert_eq!((c.cost_of(0), c.cost_of(1)), (1.0, 0.75));
        let json = serde_json::to_value(CollusionConfig::default()).unwrap();
        assert!(json["cost2"].is_null());
    }

    #[test]
    fn learning_rules_change_only_on_reset() {
        let next = CollusionConfig {
            update: Update::Synchronous,
            alpha: 0.1,
            ..CollusionConfig::default()
        };
        let changes = CollusionConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "update");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Collusion(CollusionConfig {
            cost2: Some(1.0),
            ..CollusionConfig::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
