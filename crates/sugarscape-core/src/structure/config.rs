//! The Social Structure model's parameters: Cohen, Riolo and Axelrod's
//! (2001) population, structures and adaptation, with the points the paper
//! leaves unstated or states twice as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Who plays whom (CRA's Appendix).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Structure {
    /// Random With Replacement: fresh random partners every period.
    Rwr,
    /// 2DK: a torus, each agent playing its four NEWS neighbors.
    Torus,
    /// Fixed Random Neighbors, Equal: a fixed random regular symmetric graph.
    Frne,
    /// Fixed Random Neighbors: partners drawn once, with replacement, one-way.
    Frn,
}

/// Who gets the copying noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseOn {
    /// Every agent every period, "regardless of which of the two strategies
    /// … is adopted" (the Appendix).
    Always,
    /// Only an agent that copied ("errors in the actual copying process", §2).
    Copy,
}

/// How strategies start (y = p either way).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// "Evenly distributing the agents throughout the strategy space" (the
    /// Appendix): p and q on an evenly spaced grid.
    Grid,
    /// "Strategies that were initialized randomly" (§3.1): p, q uniform.
    Random,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StructureConfig {
    /// n; a perfect square on the torus.
    pub agents: u32,
    pub structure: Structure,
    /// FFR-x: each fixed link replaced for the period by a random partner
    /// with this probability (ignored under RWR).
    pub substitution: f64,
    /// Partners each agent chooses (4 on the torus).
    pub partners: u32,
    /// Moves per game.
    pub moves: u32,
    /// The chance an agent misjudges whether its best partner did better.
    pub judge_error: f64,
    /// The chance per strategy parameter of Gaussian noise, and its s.d.
    pub mutation: f64,
    pub mutation_sd: f64,
    pub noise_on: NoiseOn,
    pub start: Start,
    /// "High cooperation": a mean payoff per move of at least this (the
    /// paper does not state its threshold).
    pub high: f64,
    /// Stop at this period (0: never).
    pub stop_at: u32,
}

impl Default for StructureConfig {
    /// CRA's Table 2, row 1: 256 agents, random partners each period.
    fn default() -> Self {
        StructureConfig {
            agents: 256,
            structure: Structure::Rwr,
            substitution: 0.0,
            partners: 4,
            moves: 4,
            judge_error: 0.1,
            mutation: 0.1,
            mutation_sd: 0.4,
            noise_on: NoiseOn::Always,
            start: Start::Grid,
            high: 2.3,
            stop_at: 0,
        }
    }
}

/// √n when n is a perfect square.
pub fn square_side(n: u32) -> Option<u32> {
    let s = (f64::from(n)).sqrt().round() as u32;
    (s * s == n).then_some(s)
}

impl StructureConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (4..=4096).contains(&self.agents),
            "agents",
            "must be between 4 and 4096",
        );
        check(
            (1..=16).contains(&self.partners) && self.partners < self.agents,
            "partners",
            "must be between 1 and 16, and fewer than the agents",
        );
        check(
            (1..=100).contains(&self.moves),
            "moves",
            "must be between 1 and 100",
        );
        check(
            unit(self.substitution),
            "substitution",
            "must be between 0 and 1",
        );
        check(
            unit(self.judge_error),
            "judge_error",
            "must be between 0 and 1",
        );
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        check(
            (0.0..=2.0).contains(&self.mutation_sd),
            "mutation_sd",
            "must be between 0 and 2",
        );
        check(
            (0.0..=5.0).contains(&self.high),
            "high",
            "must be between 0 and 5",
        );
        match self.structure {
            Structure::Torus => {
                check(
                    square_side(self.agents).is_some_and(|s| s >= 3),
                    "agents",
                    "must be a square of at least 3 × 3 on the torus",
                );
                check(self.partners == 4, "partners", "must be 4 on the torus");
            }
            Structure::Frne => check(
                self.partners.is_multiple_of(2) && self.partners + 1 < self.agents,
                "partners",
                "must be even and at most the agents − 2 for FRNE",
            ),
            Structure::Rwr | Structure::Frn => {}
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &StructureConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("structure", self.structure == next.structure),
            ("partners", self.partners == next.partners),
            ("start", self.start == next.start),
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
        Param::integer("Population", "agents", "Agents (n)", (4, 4096), Reset)
            .with_help("CRA: 256 (note 1: up to 4096 'very similar'). A square on the torus."),
        Param::choice(
            "Population",
            "start",
            "Strategies start",
            &[
                ("grid", "Evenly spread (Appendix)"),
                ("random", "At random (§3.1)"),
            ],
            Reset,
        )
        .with_help("The paper says both: 'evenly distributing … throughout the strategy space' and 'initialized randomly'."),
        Param::choice(
            "Structure",
            "structure",
            "Who plays whom",
            &[
                ("rwr", "Random each period (RWR)"),
                ("torus", "Torus neighbors (2DK)"),
                ("frne", "Fixed random, symmetric (FRNE)"),
                ("frn", "Fixed random, one-way (FRN)"),
            ],
            Reset,
        ),
        Param::number(
            "Structure",
            "substitution",
            "Random substitution (FFR)",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("CRA: with FRN, each fixed partner is replaced for the period with this probability; 0.3 is where 'the dynamics shift'. Ignored under RWR."),
        Param::integer("Structure", "partners", "Partners chosen", (1, 16), Reset)
            .with_help("CRA: 4. The torus always has 4; FRNE needs an even number."),
        Param::integer("Adaptation", "moves", "Moves per game", (1, 100), Live)
            .with_help("CRA: 4, 'short enough to make cooperation difficult … but still possible'."),
        Param::number(
            "Adaptation",
            "judge_error",
            "Misjudging the best",
            (0.0, 1.0, 0.01),
            Live,
        ),
        Param::number("Adaptation", "mutation", "Noise chance", (0.0, 1.0, 0.01), Live),
        Param::number("Adaptation", "mutation_sd", "Noise s.d.", (0.0, 2.0, 0.05), Live),
        Param::choice(
            "Adaptation",
            "noise_on",
            "Noise on",
            &[
                ("always", "Every agent (Appendix)"),
                ("copy", "Only on copying (§2)"),
            ],
            Live,
        )
        .with_help("The Appendix adds noise 'regardless of which … is adopted'; §2 describes 'errors in the actual copying process'."),
        Param::number(
            "Measures",
            "high",
            "High cooperation at",
            (0.0, 5.0, 0.05),
            Live,
        )
        .with_help("Mean payoff per move. CRA do not state their threshold; see the cra-threshold sweep."),
        Param::integer("Measures", "stop_at", "Stop at period", (0, 1_000_000), Live)
            .with_help("CRA ran 2500 periods. 0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_table_2_row_1() {
        let c = StructureConfig::default();
        assert_eq!(
            (c.agents, c.structure, c.partners, c.moves),
            (256, Structure::Rwr, 4, 4)
        );
        assert_eq!((c.judge_error, c.mutation, c.mutation_sd), (0.1, 0.1, 0.4));
        assert_eq!((c.noise_on, c.start), (NoiseOn::Always, Start::Grid));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = StructureConfig {
            agents: 2,
            moves: 0,
            substitution: 1.5,
            judge_error: -0.1,
            mutation: 2.0,
            mutation_sd: 3.0,
            high: 6.0,
            ..StructureConfig::default()
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
                "partners",
                "moves",
                "substitution",
                "judge_error",
                "mutation",
                "mutation_sd",
                "high"
            ]
        );
        let torus = StructureConfig {
            agents: 250,
            structure: Structure::Torus,
            ..StructureConfig::default()
        };
        assert_eq!(torus.validate().unwrap_err()[0].field, "agents");
        let frne = StructureConfig {
            partners: 3,
            structure: Structure::Frne,
            ..StructureConfig::default()
        };
        assert_eq!(frne.validate().unwrap_err()[0].field, "partners");
        assert_eq!((square_side(256), square_side(250)), (Some(16), None));
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Structure(StructureConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
