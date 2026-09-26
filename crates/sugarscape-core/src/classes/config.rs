//! The Emergence of Classes model's parameters: Axtell, Epstein & Young's
//! (2000) and Poza et al.'s (2011) departures, each defaulting to the
//! better-supported reading of the paper.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Where tagged agents keep what they remember.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagMemory {
    /// A memory of length m for each tag (Poza et al.'s reading of AEY).
    PerTag,
    /// One memory of the last m opponents, each with its tag.
    Shared,
}

/// How an agent picks its demand when it does not err.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Maximize expected payoff against the remembered frequencies (AEY).
    Expected,
    /// Best reply to the most frequent remembered demand (Poza et al. §3.2).
    Mode,
}

/// How memories start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// Every slot uniform over L, M and H (AEY's "random" initial state).
    Random,
    /// Every memory ⌈m/2⌉ H and ⌊m/2⌋ L, shuffled (the fractious regime).
    Fractious,
    /// Empty memories that grow to m (Poza et al. §3.4).
    Progressive,
    /// With tags: M within types; darks remember lights as L, lights
    /// remember darks as H (a class system from the start).
    Classes,
}

/// Who meets whom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    /// Random pairs from the whole population (AEY).
    Random,
    /// A random agent and one of its lattice neighbors (Poza et al. §5).
    Lattice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    Moore,
    VonNeumann,
}

/// Where the lattice's two tags sit (Poza et al.'s Fig. 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    Random,
    FourZones,
    TwoZones,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatticeConfig {
    pub width: u32,
    pub height: u32,
    pub neighborhood: Neighborhood,
    pub layout: Layout,
}

impl Default for LatticeConfig {
    /// Poza et al.'s 10 × 10 torus with Moore neighbors, tags at random.
    fn default() -> Self {
        LatticeConfig {
            width: 10,
            height: 10,
            neighborhood: Neighborhood::Moore,
            layout: Layout::Random,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClassesConfig {
    /// N; with a lattice, width × height.
    pub agents: u32,
    /// m: opponents remembered (per tag with `per_tag`).
    pub memory: u32,
    /// ε: the chance of a random demand.
    pub noise: f64,
    /// Two types, half each, told apart by a meaningless tag.
    pub tags: bool,
    pub tag_memory: TagMemory,
    pub decision: Decision,
    /// The L demand (percent); H = 100 − L, M = 50.
    pub low: u32,
    pub start: Start,
    pub interaction: Interaction,
    pub lattice: LatticeConfig,
    /// Stop at the first period in equity.
    pub stop_at_equity: bool,
}

impl Default for ClassesConfig {
    /// AEY's Fig. 2: 100 agents remembering 10 opponents, ε = 0.2, random
    /// memories, random pairs, no tags.
    fn default() -> Self {
        ClassesConfig {
            agents: 100,
            memory: 10,
            noise: 0.2,
            tags: false,
            tag_memory: TagMemory::PerTag,
            decision: Decision::Expected,
            low: 30,
            start: Start::Random,
            interaction: Interaction::Random,
            lattice: LatticeConfig::default(),
            stop_at_equity: false,
        }
    }
}

impl ClassesConfig {
    /// The H demand: what L leaves of the pie.
    pub fn high(&self) -> u32 {
        100 - self.low
    }

    /// The population: N, or the lattice's cells.
    pub fn population(&self) -> u32 {
        match self.interaction {
            Interaction::Random => self.agents,
            Interaction::Lattice => self.lattice.width * self.lattice.height,
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
            (2..=1000).contains(&self.agents) && self.agents.is_multiple_of(2),
            "agents",
            "must be an even number between 2 and 1000",
        );
        check(
            (1..=100).contains(&self.memory),
            "memory",
            "must be between 1 and 100",
        );
        check(
            (0.0..=1.0).contains(&self.noise),
            "noise",
            "must be between 0 and 1",
        );
        check(
            (5..=45).contains(&self.low),
            "low",
            "must be between 5 and 45",
        );
        check(
            self.start != Start::Classes || self.tags,
            "start",
            "a classes start needs tags",
        );
        if self.interaction == Interaction::Lattice {
            let l = &self.lattice;
            check(
                (3..=40).contains(&l.width) && (3..=40).contains(&l.height),
                "lattice",
                "sides must be between 3 and 40",
            );
            check(
                self.agents == l.width * l.height,
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
    pub(crate) fn structural_changes(&self, next: &ClassesConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("memory", self.memory == next.memory),
            ("tags", self.tags == next.tags),
            ("tag_memory", self.tag_memory == next.tag_memory),
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
        Param::integer("Population", "agents", "Agents (N, even)", (2, 1000), Reset),
        Param::integer("Population", "memory", "Memory (m)", (1, 100), Reset),
        Param::bool("Population", "tags", "Two tags", Reset),
        Param::number("Bargaining", "noise", "Noise (ε)", (0.0, 1.0, 0.01), Live),
        Param::integer(
            "Bargaining",
            "low",
            "Low demand (L; H = 100 − L)",
            (5, 45),
            Live,
        )
        .with_help("AEY: 30. Poza et al. 2011 tried 5 to 45: a higher L slows the way to equity."),
        Param::bool("Bargaining", "stop_at_equity", "Stop at equity", Live),
        Param::choice(
            "Start",
            "start",
            "Memories start",
            &[
                ("random", "Random (AEY)"),
                ("fractious", "Fractious: half H, half L"),
                ("progressive", "Empty, growing (Poza et al.)"),
                ("classes", "A class system (tags)"),
            ],
            Reset,
        ),
        Param::choice(
            "Lattice",
            "interaction",
            "Who meets whom",
            &[
                ("random", "Random pairs (AEY)"),
                ("lattice", "Lattice neighbors (Poza et al.)"),
            ],
            Reset,
        ),
        Param::choice(
            "Lattice",
            "lattice.neighborhood",
            "Neighbors",
            &[("moore", "8 (Moore)"), ("von_neumann", "4 (von Neumann)")],
            Reset,
        ),
        Param::choice(
            "Lattice",
            "lattice.layout",
            "Tags laid out",
            &[
                ("random", "At random"),
                ("four_zones", "In four zones"),
                ("two_zones", "In two zones"),
            ],
            Reset,
        ),
        Param::choice(
            "Departures",
            "decision",
            "Decision",
            &[
                ("expected", "Maximize expected payoff (AEY)"),
                ("mode", "Best reply to the most frequent demand"),
            ],
            Live,
        )
        .with_help("Poza et al. 2011: with the mode rule segregation emerges far more often."),
        Param::choice(
            "Departures",
            "tag_memory",
            "Tagged memory",
            &[
                ("per_tag", "One memory per tag"),
                ("shared", "One memory for both tags"),
            ],
            Reset,
        )
        .with_help("AEY do not say; Poza et al. 2011 describe one memory set per tag."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_aeys_figure_2() {
        let c = ClassesConfig::default();
        assert_eq!(
            (c.agents, c.memory, c.noise, c.low, c.high()),
            (100, 10, 0.2, 30, 70)
        );
        assert!(!c.tags && c.start == Start::Random && c.interaction == Interaction::Random);
        assert_eq!(
            (c.tag_memory, c.decision),
            (TagMemory::PerTag, Decision::Expected)
        );
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = ClassesConfig {
            agents: 3,
            memory: 0,
            noise: 1.5,
            low: 50,
            start: Start::Classes,
            ..ClassesConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(fields, ["agents", "memory", "noise", "low", "start"]);
        let lattice = ClassesConfig {
            agents: 50,
            interaction: Interaction::Lattice,
            ..ClassesConfig::default()
        };
        assert_eq!(lattice.validate().unwrap_err()[0].field, "agents");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Classes(ClassesConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
