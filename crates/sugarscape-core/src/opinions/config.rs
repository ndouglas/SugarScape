//! The Bounded Confidence model's parameters: Hegselmann and Krause's (2002)
//! symmetric, opinion-independent and opinion-dependent confidence, with
//! their two unfigured claims (random serial updating, local neighborhoods)
//! as switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How opinions start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// Uniform on [0, 1] (HK's random start profiles).
    Random,
    /// Evenly spaced, i/(n − 1): both ends included (HK's regular profiles).
    Regular,
}

/// The shape of every agent's confidence interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// εl = εr = ε (§4.1).
    Symmetric,
    /// Fixed εl and εr (§4.2.1).
    Asymmetric,
    /// A total ε split by the agent's own opinion (§4.2.2):
    /// εr = (m·x + (1 − m)/2)·ε, εl = ε − εr.
    OpinionDependent,
}

/// Who revises when.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Updating {
    /// Everyone at once from the last period's opinions (HK).
    Simultaneous,
    /// Each agent once per period, in a fresh random order, seeing opinions
    /// as they change.
    SerialShuffled,
    /// n agents drawn uniformly with replacement per period.
    SerialRandom,
}

/// Whose opinions an agent can take into account.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    /// Everyone (HK).
    All,
    /// Itself and its torus neighbors (HK §4.3's "first simulations").
    Lattice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    Moore,
    VonNeumann,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatticeConfig {
    pub width: u32,
    pub height: u32,
    pub neighborhood: Neighborhood,
}

impl Default for LatticeConfig {
    /// 25 × 25: HK's 625 agents on a torus, eight neighbors each.
    fn default() -> Self {
        LatticeConfig {
            width: 25,
            height: 25,
            neighborhood: Neighborhood::Moore,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OpinionsConfig {
    /// n; with a lattice, width × height.
    pub agents: u32,
    pub start: Start,
    pub confidence: Confidence,
    /// ε: the reach each way (symmetric), or the total (opinion-dependent).
    pub epsilon: f64,
    /// εl and εr under `asymmetric`.
    pub epsilon_left: f64,
    pub epsilon_right: f64,
    /// m, the slope of the opinion-dependent bias.
    pub bias: f64,
    pub updating: Updating,
    pub interaction: Interaction,
    pub lattice: LatticeConfig,
    /// Stop at the first stable period.
    pub stop_when_stable: bool,
}

impl Default for OpinionsConfig {
    /// HK's Fig. 2b: 625 random opinions, ε = 0.15, simultaneous updating.
    fn default() -> Self {
        OpinionsConfig {
            agents: 625,
            start: Start::Random,
            confidence: Confidence::Symmetric,
            epsilon: 0.15,
            epsilon_left: 0.1,
            epsilon_right: 0.2,
            bias: 0.5,
            updating: Updating::Simultaneous,
            interaction: Interaction::All,
            lattice: LatticeConfig::default(),
            stop_when_stable: true,
        }
    }
}

impl OpinionsConfig {
    /// How far an agent at opinion `x` reaches left and right.
    pub fn reach(&self, x: f64) -> (f64, f64) {
        match self.confidence {
            Confidence::Symmetric => (self.epsilon, self.epsilon),
            Confidence::Asymmetric => (self.epsilon_left, self.epsilon_right),
            Confidence::OpinionDependent => {
                let right = (self.bias * x + (1.0 - self.bias) / 2.0) * self.epsilon;
                (self.epsilon - right, right)
            }
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (2..=2000).contains(&self.agents),
            "agents",
            "must be between 2 and 2000",
        );
        check(unit(self.epsilon), "epsilon", "must be between 0 and 1");
        check(
            unit(self.epsilon_left),
            "epsilon_left",
            "must be between 0 and 1",
        );
        check(
            unit(self.epsilon_right),
            "epsilon_right",
            "must be between 0 and 1",
        );
        check(unit(self.bias), "bias", "must be between 0 and 1");
        if self.interaction == Interaction::Lattice {
            let l = &self.lattice;
            check(
                (3..=44).contains(&l.width) && (3..=44).contains(&l.height),
                "lattice",
                "sides must be between 3 and 44",
            );
            check(
                u64::from(self.agents) == u64::from(l.width) * u64::from(l.height),
                "agents",
                "must be the lattice's width × height",
            );
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &OpinionsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("start", self.start == next.start),
            ("interaction", self.interaction == next.interaction),
            ("lattice", self.lattice == next.lattice),
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
        Param::integer("Population", "agents", "Agents (n)", (2, 2000), Reset)
            .with_help("HK: 625 random opinions, or 50 and 100 evenly spaced. With a lattice, width × height."),
        Param::choice(
            "Population",
            "start",
            "Opinions start",
            &[
                ("random", "At random (uniform)"),
                ("regular", "Evenly spaced"),
            ],
            Reset,
        ),
        Param::bool("Population", "stop_when_stable", "Stop when stable", Live)
            .with_help("Stable: no opinion moved more than 10⁻¹⁰ this period. Opinions within 10⁻⁶ count as one."),
        Param::choice(
            "Confidence",
            "confidence",
            "Confidence",
            &[
                ("symmetric", "Symmetric (§4.1)"),
                ("asymmetric", "Asymmetric, the same for all (§4.2.1)"),
                ("opinion_dependent", "Leaning with one's opinion (§4.2.2)"),
            ],
            Live,
        ),
        Param::number("Confidence", "epsilon", "Confidence (ε)", (0.0, 1.0, 0.01), Live)
            .with_help("Symmetric: the reach each way. Leaning: the total, split by the bias."),
        Param::number(
            "Confidence",
            "epsilon_left",
            "Left reach (εl)",
            (0.0, 1.0, 0.01),
            Live,
        )
        .shown_if("confidence", "asymmetric"),
        Param::number(
            "Confidence",
            "epsilon_right",
            "Right reach (εr)",
            (0.0, 1.0, 0.01),
            Live,
        )
        .shown_if("confidence", "asymmetric"),
        Param::number("Confidence", "bias", "Bias (m)", (0.0, 1.0, 0.01), Live)
            .shown_if("confidence", "opinion_dependent")
            .with_help("HK §4.2.2: 0 is symmetric; at 1 an agent at 0 or 1 listens only to its own side."),
        Param::choice(
            "Updating",
            "updating",
            "Updating",
            &[
                ("simultaneous", "Simultaneous (HK)"),
                ("serial_shuffled", "Serial, each once in random order"),
                ("serial_random", "Serial, n random draws"),
            ],
            Live,
        )
        .with_help("HK §4.3: 'none of the results … depends crucially on simultaneous updating' — which serial order is not said."),
        Param::choice(
            "Lattice",
            "interaction",
            "Who listens to whom",
            &[
                ("all", "Everyone (HK)"),
                ("lattice", "Lattice neighbors (HK §4.3)"),
            ],
            Reset,
        )
        .with_help("HK §4.3: with small overlapping neighborhoods 'polarization disappears' — reported, not shown."),
        Param::integer("Lattice", "lattice.width", "Width", (3, 44), Reset)
            .shown_if("interaction", "lattice"),
        Param::integer("Lattice", "lattice.height", "Height", (3, 44), Reset)
            .shown_if("interaction", "lattice"),
        Param::choice(
            "Lattice",
            "lattice.neighborhood",
            "Neighbors",
            &[("moore", "8 (Moore)"), ("von_neumann", "4 (von Neumann)")],
            Reset,
        )
        .shown_if("interaction", "lattice"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_hks_figure_2b() {
        let c = OpinionsConfig::default();
        assert_eq!((c.agents, c.epsilon), (625, 0.15));
        assert_eq!(
            (c.start, c.confidence, c.updating, c.interaction),
            (
                Start::Random,
                Confidence::Symmetric,
                Updating::Simultaneous,
                Interaction::All
            )
        );
        assert!(c.stop_when_stable && c.validate().is_ok());
    }

    #[test]
    fn the_reach_follows_each_confidence() {
        let mut c = OpinionsConfig::default();
        assert_eq!(c.reach(0.9), (0.15, 0.15));
        c.confidence = Confidence::Asymmetric;
        assert_eq!(c.reach(0.9), (0.1, 0.2));
        // HK's worked example: x 0.6, ε 0.4, m 0.5 → εl 0.18, εr 0.22.
        c.confidence = Confidence::OpinionDependent;
        c.epsilon = 0.4;
        let (l, r) = c.reach(0.6);
        assert!(
            (l - 0.18).abs() < 1e-12 && (r - 0.22).abs() < 1e-12,
            "{l} {r}"
        );
        let (l, r) = c.reach(0.5);
        assert!((l - r).abs() < 1e-12, "the center is symmetric");
        c.bias = 1.0;
        assert_eq!(
            c.reach(0.0),
            (0.4, 0.0),
            "at m = 1 the left edge looks only left"
        );
    }

    #[test]
    fn validation_names_fields() {
        let bad = OpinionsConfig {
            agents: 1,
            epsilon: 1.5,
            epsilon_left: -0.1,
            epsilon_right: 2.0,
            bias: 1.1,
            ..OpinionsConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["agents", "epsilon", "epsilon_left", "epsilon_right", "bias"]
        );
        let lattice = OpinionsConfig {
            agents: 600,
            interaction: Interaction::Lattice,
            ..OpinionsConfig::default()
        };
        assert_eq!(lattice.validate().unwrap_err()[0].field, "agents");
        let tiny = OpinionsConfig {
            agents: 4,
            interaction: Interaction::Lattice,
            lattice: LatticeConfig {
                width: 2,
                height: 2,
                neighborhood: Neighborhood::Moore,
            },
            ..OpinionsConfig::default()
        };
        assert_eq!(tiny.validate().unwrap_err()[0].field, "lattice");
    }

    #[test]
    fn huge_pasted_lattice_sides_fail_validation_without_overflowing() {
        // width * height as u32 would overflow (70_000² > u32::MAX); this
        // must report an error, not panic in a debug build.
        let huge = OpinionsConfig {
            agents: 4,
            interaction: Interaction::Lattice,
            lattice: LatticeConfig {
                width: 70_000,
                height: 70_000,
                neighborhood: Neighborhood::Moore,
            },
            ..OpinionsConfig::default()
        };
        let fields: Vec<String> = huge
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert!(fields.contains(&"lattice".to_string()), "{fields:?}");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Opinions(OpinionsConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
