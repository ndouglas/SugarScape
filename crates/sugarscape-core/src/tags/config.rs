//! The tags model's parameters: the paper's (Riolo, Cohen & Axelrod 2001)
//! and the replications' switches, each defaulting to the paper's literal
//! reading.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How tolerances start: the paper's U[0, 1], or every agent at `x` (its
/// other experiments' 0.5 and 0.005; E&H's 0).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialTolerance {
    Uniform,
    Fixed(f64),
}

/// Who reproduces when a tournament's two scores are equal (E&H Table 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TieRule {
    /// A coin flip ("no bias"): the literal reading.
    Random,
    /// The agent whose turn it is ("selected bias"): what reproduces RCA's tables.
    Current,
    /// The randomly drawn opponent ("random bias").
    Other,
}

/// When a donor gives: |τ_A − τ_B| ≤ T_A (the paper) or < T_A (E&H).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DonationTest {
    AtMost,
    Below,
}

/// How the next generation is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Selection {
    /// Each agent against a random other; the higher score gives the offspring.
    Tournament,
    /// The paper's learning variant: adopt a better agent's traits with
    /// probability proportional to how much better it scored.
    Adopt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TagsConfig {
    pub agents: u32,
    /// P: donation chances per agent per generation.
    pub pairings: u32,
    /// c, paid by a donor.
    pub cost: f64,
    /// b, gained by a recipient.
    pub benefit: f64,
    pub initial_tolerance: InitialTolerance,
    /// Probability an offspring gets a fresh U[0, 1) tag.
    pub tag_mutation: f64,
    /// Probability an offspring's tolerance gets Gaussian noise.
    pub tolerance_mutation: f64,
    /// That noise's standard deviation.
    pub tolerance_sd: f64,
    /// The last generation; the run stops there (0: never).
    pub end: u32,
    pub tie_rule: TieRule,
    pub donation_test: DonationTest,
    /// A mutated tolerance is raised to at least this (the paper: 0; R&S: −10⁻⁶).
    pub tolerance_floor: f64,
    /// S.d. of Gaussian noise on every offspring's tag (E&H: 10⁻⁶), clamped to [0, 1].
    pub tag_noise: f64,
    pub selection: Selection,
}

impl Default for TagsConfig {
    /// The paper's defaults (P = 3, c = 0.1, b = 1, 100 agents, 30 000
    /// generations) with its literal rules: coin-flip ties, ≤, a floor of 0,
    /// no tag noise, tournament selection.
    fn default() -> Self {
        TagsConfig {
            agents: 100,
            pairings: 3,
            cost: 0.1,
            benefit: 1.0,
            initial_tolerance: InitialTolerance::Uniform,
            tag_mutation: 0.1,
            tolerance_mutation: 0.1,
            tolerance_sd: 0.01,
            end: 30_000,
            tie_rule: TieRule::Random,
            donation_test: DonationTest::AtMost,
            tolerance_floor: 0.0,
            tag_noise: 0.0,
            selection: Selection::Tournament,
        }
    }
}

impl TagsConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |x: f64| (0.0..=1.0).contains(&x);
        let nonneg = |x: f64| x.is_finite() && x >= 0.0;
        check(
            (2..=10_000).contains(&self.agents),
            "agents",
            "must be between 2 and 10000",
        );
        check(self.pairings <= 100, "pairings", "must be at most 100");
        check(nonneg(self.cost), "cost", "must be a number ≥ 0");
        check(nonneg(self.benefit), "benefit", "must be a number ≥ 0");
        if let InitialTolerance::Fixed(x) = self.initial_tolerance {
            check(nonneg(x), "initial_tolerance", "must be a number ≥ 0");
        }
        check(
            unit(self.tag_mutation),
            "tag_mutation",
            "must be between 0 and 1",
        );
        check(
            unit(self.tolerance_mutation),
            "tolerance_mutation",
            "must be between 0 and 1",
        );
        check(
            nonneg(self.tolerance_sd) && self.tolerance_sd <= 1.0,
            "tolerance_sd",
            "must be between 0 and 1",
        );
        check(
            (-1.0..=0.0).contains(&self.tolerance_floor),
            "tolerance_floor",
            "must be between −1 and 0",
        );
        check(
            nonneg(self.tag_noise) && self.tag_noise <= 1.0,
            "tag_noise",
            "must be between 0 and 1",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &TagsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        if self.agents != next.agents {
            out.push(FieldError::new("agents", "changes only on reset"));
        }
        if self.initial_tolerance != next.initial_tolerance {
            out.push(FieldError::new(
                "initial_tolerance",
                "changes only on reset",
            ));
        }
        out
    }
}

/// The Rules panel's fields. Everything but the population applies to the
/// running world. `initial_tolerance` is not on the panel (its `fixed`
/// form is not a panel kind); presets, files and links set it.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Population", "agents", "Agents", (2, 1000), Reset),
        Param::integer("Population", "end", "Last generation (0: never)", (0, 100_000), Live),
        Param::integer("Donation", "pairings", "Pairings per agent (P)", (0, 20), Live),
        Param::number("Donation", "cost", "Cost to donor (c)", (0.0, 2.0, 0.05), Live),
        Param::number("Donation", "benefit", "Benefit to recipient (b)", (0.0, 5.0, 0.1), Live),
        Param::number("Mutation", "tag_mutation", "New tag probability", (0.0, 1.0, 0.01), Live),
        Param::number(
            "Mutation",
            "tolerance_mutation",
            "Tolerance mutation probability",
            (0.0, 1.0, 0.01),
            Live,
        ),
        Param::number("Mutation", "tolerance_sd", "Tolerance mutation s.d.", (0.0, 0.1, 0.001), Live),
        Param::choice(
            "Replications",
            "tie_rule",
            "Equal scores",
            &[
                ("random", "Coin flip (literal)"),
                ("current", "Current agent wins (reproduces the paper)"),
                ("other", "Opponent wins"),
            ],
            Live,
        )
        .with_help("The paper does not say; Edmonds & Hales 2003 found only “current agent wins” matches its tables."),
        Param::choice(
            "Replications",
            "donation_test",
            "Donate when",
            &[("at_most", "|Δtag| ≤ tolerance (paper)"), ("below", "|Δtag| < tolerance")],
            Live,
        )
        .with_help("With <, identical tags need not donate (Edmonds & Hales 2003)."),
        Param::number(
            "Replications",
            "tolerance_floor",
            "Tolerance floor",
            (-0.001, 0.0, 0.000001),
            Live,
        )
        .with_help("The paper: 0. Roberts & Sherratt 2002: −10⁻⁶, so identical tags need not donate."),
        Param::number("Replications", "tag_noise", "Tag noise s.d.", (0.0, 0.001, 0.000001), Live)
            .with_help("Edmonds & Hales 2003: 10⁻⁶ on every offspring, so no two tags are ever equal."),
        Param::choice(
            "Replications",
            "selection",
            "Selection",
            &[
                ("tournament", "Tournament (paper)"),
                ("adopt", "Adopt a better agent’s traits, in proportion"),
            ],
            Live,
        )
        .with_help("The paper’s learning variant; “in proportion” is normalized by this generation’s score range (our reading)."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_the_papers_with_the_literal_tie_rule() {
        let c = TagsConfig::default();
        assert_eq!(
            (c.agents, c.pairings, c.cost, c.benefit, c.end),
            (100, 3, 0.1, 1.0, 30_000)
        );
        assert_eq!(
            (c.tag_mutation, c.tolerance_mutation, c.tolerance_sd),
            (0.1, 0.1, 0.01)
        );
        assert_eq!(c.tie_rule, TieRule::Random);
        assert_eq!(c.donation_test, DonationTest::AtMost);
        assert_eq!((c.tolerance_floor, c.tag_noise), (0.0, 0.0));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn initial_tolerance_is_a_word_or_a_fixed_value() {
        let c = ModelConfig::from_json(r#"{"model": "tags", "initial_tolerance": {"fixed": 0}}"#)
            .unwrap();
        let ModelConfig::Tags(t) = &c else {
            unreachable!()
        };
        assert_eq!(t.initial_tolerance, InitialTolerance::Fixed(0.0));
        assert_eq!(
            serde_json::to_value(TagsConfig::default()).unwrap()["initial_tolerance"],
            "uniform"
        );
    }

    #[test]
    fn validation_names_fields() {
        let bad = TagsConfig {
            agents: 1,
            pairings: 101,
            cost: -1.0,
            benefit: f64::NAN,
            initial_tolerance: InitialTolerance::Fixed(-0.1),
            tag_mutation: 2.0,
            tolerance_mutation: -0.1,
            tolerance_sd: 2.0,
            tolerance_floor: 0.1,
            tag_noise: -1.0,
            ..TagsConfig::default()
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
                "pairings",
                "cost",
                "benefit",
                "initial_tolerance",
                "tag_mutation",
                "tolerance_mutation",
                "tolerance_sd",
                "tolerance_floor",
                "tag_noise"
            ]
        );
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Tags(TagsConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
