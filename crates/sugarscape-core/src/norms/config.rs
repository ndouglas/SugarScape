//! The Norms and Metanorms model's parameters: Axelrod's (1986) values, his
//! dominance variant, and Galán & Izquierdo's (2005) departures and readings
//! where his text is ambiguous, as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How the next generation's parents are chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Selection {
    /// Axelrod: two offspring at one standard deviation above the mean,
    /// none at one below, one otherwise.
    Axelrod,
    /// G&I: n times, the better of two random agents.
    Tournament,
    /// G&I: n times, a pick proportional to payoff minus the minimum.
    Roulette,
    /// G&I: two offspring at or above the mean, none below.
    Average,
}

/// How the offspring are brought back to the population size ("For
/// convenience, the number of offspring is adjusted").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refill {
    /// G&I: remove, or duplicate, random offspring.
    Random,
    /// Remove the worst parents' copies first; duplicate the best's.
    Ranked,
}

/// Axelrod's rule when every payoff is equal (no standard deviation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllEqual {
    /// G&I's note 4: everyone twice, then a random half removed.
    Drift,
    /// Everyone once.
    Keep,
}

/// Axelrod's dominance variant: a strong group less hurt by punishment and
/// more numerous than a weak one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GroupsConfig {
    pub enabled: bool,
    pub strong: u32,
    pub weak: u32,
    /// The strong group's cost of being punished (the weak group's is `punishment`).
    pub strong_punishment: f64,
}

impl Default for GroupsConfig {
    /// Axelrod's 20 whites (P = −3) and 10 blacks (P = −9).
    fn default() -> Self {
        GroupsConfig {
            enabled: false,
            strong: 20,
            weak: 10,
            strong_punishment: -3.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NormsConfig {
    /// The population (ignored under `groups`).
    pub agents: u32,
    pub metanorms: bool,
    /// Opportunities to defect per agent per generation.
    pub rounds: u32,
    pub temptation: f64,
    pub hurt: f64,
    pub punishment: f64,
    pub enforcement: f64,
    pub meta_punishment: f64,
    pub meta_enforcement: f64,
    /// The chance each bit of each offspring flips.
    pub mutation: f64,
    pub selection: Selection,
    pub refill: Refill,
    pub all_equal: AllEqual,
    pub groups: GroupsConfig,
    /// Stop at this generation (0: never).
    pub stop_at: u32,
}

impl Default for NormsConfig {
    /// Axelrod's norms game: 20 players, four rounds, T 3, H −1, P −9, E −2,
    /// 1 % mutation per bit.
    fn default() -> Self {
        NormsConfig {
            agents: 20,
            metanorms: false,
            rounds: 4,
            temptation: 3.0,
            hurt: -1.0,
            punishment: -9.0,
            enforcement: -2.0,
            meta_punishment: -9.0,
            meta_enforcement: -2.0,
            mutation: 0.01,
            selection: Selection::Axelrod,
            refill: Refill::Random,
            all_equal: AllEqual::Drift,
            groups: GroupsConfig::default(),
            stop_at: 0,
        }
    }
}

impl NormsConfig {
    /// The number of agents: `agents`, or the two groups together.
    pub fn population(&self) -> u32 {
        if self.groups.enabled {
            self.groups.strong + self.groups.weak
        } else {
            self.agents
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
            (2..=200).contains(&self.agents),
            "agents",
            "must be between 2 and 200",
        );
        check(
            (1..=20).contains(&self.rounds),
            "rounds",
            "must be between 1 and 20",
        );
        for (field, v) in [
            ("temptation", self.temptation),
            ("hurt", self.hurt),
            ("punishment", self.punishment),
            ("enforcement", self.enforcement),
            ("meta_punishment", self.meta_punishment),
            ("meta_enforcement", self.meta_enforcement),
            ("groups", self.groups.strong_punishment),
        ] {
            check(
                (-100.0..=100.0).contains(&v),
                field,
                "must be between −100 and 100",
            );
        }
        check(
            (0.0..=1.0).contains(&self.mutation),
            "mutation",
            "must be between 0 and 1",
        );
        if self.groups.enabled {
            check(
                (2..=200).contains(&self.groups.strong) && (2..=200).contains(&self.groups.weak),
                "groups",
                "each group must have between 2 and 200 agents",
            );
            check(
                self.groups.strong + self.groups.weak <= 200,
                "groups",
                "the two groups together must have at most 200 agents",
            );
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &NormsConfig) -> Vec<FieldError> {
        let g = (&self.groups, &next.groups);
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            (
                "groups",
                g.0.enabled == g.1.enabled && g.0.strong == g.1.strong && g.0.weak == g.1.weak,
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
        Param::integer("Population", "agents", "Agents", (2, 200), Reset)
            .with_help("Axelrod: 20."),
        Param::bool("Population", "groups.enabled", "Two groups (dominance)", Reset)
            .with_help("Axelrod: 20 whites less hurt by punishment (P = −3) and 10 blacks (P = −9); selection within each group."),
        Param::integer("Population", "groups.strong", "Strong group", (2, 198), Reset)
            .shown_if("groups.enabled", "true"),
        Param::integer("Population", "groups.weak", "Weak group", (2, 198), Reset)
            .shown_if("groups.enabled", "true"),
        Param::integer("Population", "stop_at", "Stop at generation", (0, 10_000_000), Live)
            .with_help("Axelrod ran 100 generations; Galán & Izquierdo up to 10⁶. 0: never."),
        Param::bool("Payoffs", "metanorms", "Metanorms", Live)
            .with_help("Punish those seen not punishing, with the same vengefulness (Axelrod's 'critical assumption')."),
        Param::number("Payoffs", "temptation", "Temptation (T)", (0.0, 20.0, 0.5), Live)
            .with_help("Axelrod: 3. Galán & Izquierdo: at 10 the metanorm holds."),
        Param::number("Payoffs", "hurt", "Hurt (H)", (-10.0, 0.0, 0.5), Live),
        Param::number("Payoffs", "punishment", "Punishment (P)", (-20.0, 0.0, 0.5), Live),
        Param::number("Payoffs", "enforcement", "Enforcement (E)", (-10.0, 0.0, 0.5), Live),
        Param::number(
            "Payoffs",
            "meta_punishment",
            "Metapunishment (MP)",
            (-20.0, 0.0, 0.1),
            Live,
        )
        .with_help("Axelrod: −9. Galán & Izquierdo's milder −0.9 (with ME −0.2) reverses his result."),
        Param::number(
            "Payoffs",
            "meta_enforcement",
            "Metaenforcement (ME)",
            (-10.0, 0.0, 0.1),
            Live,
        ),
        Param::number(
            "Payoffs",
            "groups.strong_punishment",
            "Strong group's punishment",
            (-20.0, 0.0, 0.5),
            Live,
        )
        .shown_if("groups.enabled", "true"),
        Param::integer("Evolution", "rounds", "Rounds per generation", (1, 20), Live)
            .with_help("Axelrod: four opportunities to defect each."),
        Param::number("Evolution", "mutation", "Mutation per bit", (0.0, 1.0, 0.001), Live)
            .with_help("Axelrod: 0.01. Galán & Izquierdo: at 0.001 the metanorm collapses faster."),
        Param::choice(
            "Evolution",
            "selection",
            "Selection",
            &[
                ("axelrod", "Axelrod (one s.d. above: two; below: none)"),
                ("tournament", "Random tournament"),
                ("roulette", "Roulette wheel"),
                ("average", "Above the mean: two; below: none"),
            ],
            Live,
        )
        .with_help("Galán & Izquierdo: three equally valid rules; all lose the norm sooner than Axelrod's."),
        Param::choice(
            "Readings",
            "refill",
            "Back to the population size",
            &[
                ("random", "Random removal or copies (G&I)"),
                ("ranked", "The worst removed, the best copied"),
            ],
            Live,
        )
        .with_help("Axelrod: 'the number of offspring is adjusted' — how is not said."),
        Param::choice(
            "Readings",
            "all_equal",
            "When every payoff ties",
            &[
                ("drift", "Everyone twice, half removed (G&I)"),
                ("keep", "Everyone once"),
            ],
            Live,
        )
        .with_help("Galán & Izquierdo's note 4: this choice 'can alter the long-term results significantly'."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_axelrods() {
        let c = NormsConfig::default();
        assert_eq!((c.agents, c.rounds, c.mutation), (20, 4, 0.01));
        assert_eq!(
            (c.temptation, c.hurt, c.punishment, c.enforcement),
            (3.0, -1.0, -9.0, -2.0)
        );
        assert_eq!((c.meta_punishment, c.meta_enforcement), (-9.0, -2.0));
        assert!(!c.metanorms && !c.groups.enabled);
        assert_eq!(
            (c.selection, c.refill, c.all_equal),
            (Selection::Axelrod, Refill::Random, AllEqual::Drift)
        );
        assert_eq!(
            (c.groups.strong, c.groups.weak, c.groups.strong_punishment),
            (20, 10, -3.0)
        );
        assert!(c.validate().is_ok());
        assert_eq!(c.population(), 20);
        let g = NormsConfig {
            groups: GroupsConfig {
                enabled: true,
                ..GroupsConfig::default()
            },
            ..c
        };
        assert_eq!(g.population(), 30);
    }

    #[test]
    fn validation_names_fields() {
        let bad = NormsConfig {
            agents: 1,
            rounds: 0,
            temptation: 200.0,
            mutation: 1.5,
            groups: GroupsConfig {
                enabled: true,
                strong: 1,
                ..GroupsConfig::default()
            },
            ..NormsConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["agents", "rounds", "temptation", "mutation", "groups"]
        );
    }

    #[test]
    fn validation_caps_the_metanorms_stall_paths() {
        // 400 agents x 100 rounds is O(rounds*n^3) per generation, about 3.9s
        // natively; both must stay small enough for the UI not to stall.
        let bad = NormsConfig {
            rounds: 21,
            groups: GroupsConfig {
                enabled: true,
                strong: 150,
                weak: 150,
                ..GroupsConfig::default()
            },
            ..NormsConfig::default()
        };
        let errors = bad.validate().unwrap_err();
        let fields: Vec<String> = errors.iter().map(|e| e.field.clone()).collect();
        assert_eq!(fields, ["rounds", "groups"]);
        assert_eq!(errors[0].message, "must be between 1 and 20");
        assert_eq!(
            errors[1].message,
            "the two groups together must have at most 200 agents"
        );
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Norms(NormsConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
