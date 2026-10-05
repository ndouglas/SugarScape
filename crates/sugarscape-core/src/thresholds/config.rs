//! The Threshold Models' parameters: Granovetter's (1978) crowds with the
//! four extensions he sketches (friends, crowds sampled from a city, clusters
//! with movement, ceilings), and Watts's (2002) cascades on random networks,
//! as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How the thresholds are distributed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Distribution {
    /// Granovetter: one each at 0, 1/N, …, (N − 1)/N.
    Uniform,
    /// The uniform crowd with the person at 1/N moved to 2/N.
    Perturbed,
    /// Normal with `mean` and `sd` (below 0 is 0; above 1 never acts).
    Normal,
    /// Everyone at `mean` (Watts's φ*).
    Fixed,
}

/// How a normal crowd is realized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Crowd {
    /// The normal's (i + ½)/N quantiles: a finite realization of its CDF.
    Quantiles,
    /// N independent draws.
    Sampled,
}

/// Whether thresholds stay fractions or become whole people.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rounding {
    Exact,
    Floor,
    Nearest,
}

/// Where each episode's crowd comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Population {
    /// The crowd as drawn from `distribution`.
    Fixed,
    /// N drawn from a city with uniform thresholds 0–99 % (Granovetter).
    City,
}

/// Who sees whom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// Granovetter: everyone sees the whole crowd.
    Everyone,
    /// Watts: Poisson degrees with mean `degree`.
    Random,
    /// Watts's Fig. 4b: p_k ∝ k^−2.5 e^−k/κ (k ≥ 1), κ set for `degree`.
    PowerLaw,
}

/// What starts an episode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    /// Granovetter: whoever has threshold 0 acts at once.
    Instigators,
    /// Watts: one random actor switched on.
    Random,
    /// The highest-degree actor (the lower index on a tie).
    Hub,
}

/// What an actor with threshold 0 or less does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zero {
    /// Acts spontaneously (the rule read literally: 0 ≥ 0).
    Acts,
    /// Acts once at least one neighbor does.
    WhenReached,
}

/// How actors update within a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Everyone from the last step's states: r(t + 1) = F[r(t)].
    Synchronous,
    /// A random order, each seeing the latest states (Watts).
    Asynchronous,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Friends {
    pub enabled: bool,
    /// The chance two actors are friends (Granovetter's "acquaintance volume").
    pub acquaintance: f64,
    /// How many strangers a friend counts for.
    pub weight: u32,
    pub symmetric: bool,
}

impl Default for Friends {
    /// Granovetter's example: friends count twice; people know a quarter of the crowd.
    fn default() -> Self {
        Friends {
            enabled: false,
            acquaintance: 0.25,
            weight: 2,
            symmetric: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Ceilings {
    /// The share of actors (chosen at random) who also leave.
    pub share: f64,
    /// They stop once the proportion of others acting exceeds this.
    pub at: f64,
}

impl Default for Ceilings {
    /// Granovetter: "leave when the total passed 90%".
    fn default() -> Self {
        Ceilings {
            share: 0.0,
            at: 0.9,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Clusters {
    pub enabled: bool,
    pub count: u32,
    /// Each actor's chance per step of moving to a random other crowd.
    pub movement: f64,
}

impl Default for Clusters {
    fn default() -> Self {
        Clusters {
            enabled: false,
            count: 10,
            movement: 0.05,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThresholdsConfig {
    /// N, per crowd.
    pub actors: u32,
    pub distribution: Distribution,
    /// The normal's mean (and `fixed`'s threshold), as a fraction.
    pub mean: f64,
    pub sd: f64,
    pub crowd: Crowd,
    pub rounding: Rounding,
    pub population: Population,
    pub network: Network,
    /// Watts's z.
    pub degree: f64,
    /// The actor counts as a nonacting member of the group he divides by.
    pub counts_self: bool,
    pub friends: Friends,
    pub trigger: Trigger,
    pub zero: Zero,
    pub update: Update,
    pub ceilings: Ceilings,
    pub clusters: Clusters,
    /// Start a new episode at each equilibrium.
    pub repeat: bool,
    /// The share of actors that makes a cascade global.
    pub global: f64,
    /// An episode ends here if it has not settled.
    pub max_steps: u32,
    /// Stop at this step (0: never).
    pub stop_at: u32,
}

impl Default for ThresholdsConfig {
    /// Granovetter's uniform crowd: 100 people, thresholds 0 to 99.
    fn default() -> Self {
        ThresholdsConfig {
            actors: 100,
            distribution: Distribution::Uniform,
            mean: 0.25,
            sd: 0.122,
            crowd: Crowd::Quantiles,
            rounding: Rounding::Exact,
            population: Population::Fixed,
            network: Network::Everyone,
            degree: 3.0,
            counts_self: true,
            friends: Friends::default(),
            trigger: Trigger::Instigators,
            zero: Zero::Acts,
            update: Update::Synchronous,
            ceilings: Ceilings::default(),
            clusters: Clusters::default(),
            repeat: false,
            global: 0.1,
            max_steps: 1000,
            stop_at: 0,
        }
    }
}

/// The most actors in a world.
pub const MAX_ACTORS: u32 = 20_000;
/// The power-law family's largest mean degree (k ≥ 1, τ 2.5): ζ(1.5)/ζ(2.5).
pub const POWER_LAW_CAP: f64 = 1.95;

impl ThresholdsConfig {
    /// Actors in the world: N, or K crowds of N.
    pub fn population_size(&self) -> u32 {
        if self.clusters.enabled {
            self.actors * self.clusters.count
        } else {
            self.actors
        }
    }

    /// Whether acting can stop (ceilings or clusters): Granovetter's
    /// "removal", which makes oscillation possible.
    pub fn reversible(&self) -> bool {
        self.ceilings.share > 0.0 || self.clusters.enabled
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
            (2..=MAX_ACTORS).contains(&self.actors),
            "actors",
            "must be between 2 and 20000",
        );
        check(unit(self.mean), "mean", "must be between 0 and 1");
        check(unit(self.sd), "sd", "must be between 0 and 1");
        check(
            match self.network {
                Network::Everyone => true,
                Network::Random => {
                    self.degree >= 0.0
                        && self.degree <= 100.0
                        && self.degree < f64::from(self.actors - 1)
                }
                Network::PowerLaw => self.degree > 1.0 && self.degree < POWER_LAW_CAP,
            },
            "degree",
            match self.network {
                Network::PowerLaw => {
                    "must be between 1 and 1.95: with k ≥ 1 and τ 2.5 the power law's mean degree cannot exceed ζ(1.5)/ζ(2.5)"
                }
                _ => "must be between 0 and 100 and less than the number of actors",
            },
        );
        let f = &self.friends;
        check(
            !f.enabled
                || (self.network == Network::Everyone
                    && !self.clusters.enabled
                    && unit(f.acquaintance)
                    && (1..=20).contains(&f.weight)
                    && self.actors <= 2000),
            "friends",
            "need everyone to see everyone, no clusters, an acquaintance between 0 and 1, a weight of 1 to 20 and at most 2000 actors",
        );
        check(
            self.trigger != Trigger::Hub || self.network != Network::Everyone,
            "trigger",
            "the hub needs a network",
        );
        let c = &self.ceilings;
        check(
            unit(c.share) && unit(c.at),
            "ceilings",
            "the share and the level must be between 0 and 1",
        );
        let k = &self.clusters;
        check(
            !k.enabled
                || ((2..=50).contains(&k.count)
                    && unit(k.movement)
                    && self.population == Population::City
                    && self.network == Network::Everyone
                    && self.actors * k.count <= MAX_ACTORS),
            "clusters",
            "need 2 to 50 crowds drawn from the city, everyone seeing their own crowd, a movement between 0 and 1 and at most 20000 actors in all",
        );
        check(
            self.global > 0.0 && self.global <= 1.0,
            "global",
            "must be above 0 and at most 1",
        );
        check(
            (1..=100_000).contains(&self.max_steps),
            "max_steps",
            "must be between 1 and 100000",
        );
        check(
            self.stop_at <= 10_000_000,
            "stop_at",
            "must be at most 10000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &ThresholdsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("actors", self.actors == next.actors),
            ("distribution", self.distribution == next.distribution),
            ("mean", self.mean == next.mean),
            ("sd", self.sd == next.sd),
            ("crowd", self.crowd == next.crowd),
            ("rounding", self.rounding == next.rounding),
            ("population", self.population == next.population),
            ("network", self.network == next.network),
            ("degree", self.degree == next.degree),
            ("counts_self", self.counts_self == next.counts_self),
            ("friends", self.friends == next.friends),
            ("trigger", self.trigger == next.trigger),
            ("zero", self.zero == next.zero),
            ("ceilings", self.ceilings == next.ceilings),
            ("clusters", self.clusters == next.clusters),
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
        Param::integer("Crowd", "actors", "Actors (N)", (2, MAX_ACTORS), Reset)
            .with_help("Granovetter's examples: 100. Watts: 10 000."),
        Param::choice(
            "Crowd",
            "population",
            "Each crowd",
            &[
                ("fixed", "As drawn"),
                ("city", "Sampled from a city (uniform 0–99 %)"),
            ],
            Reset,
        ),
        Param::choice(
            "Crowd",
            "trigger",
            "Started by",
            &[
                ("instigators", "Those with threshold 0 (Granovetter)"),
                ("random", "One random actor (Watts)"),
                ("hub", "The best-connected actor"),
            ],
            Reset,
        ),
        Param::choice(
            "Crowd",
            "update",
            "Actors decide",
            &[
                ("synchronous", "Together, from the last step"),
                ("asynchronous", "One at a time (Watts)"),
            ],
            Live,
        ),
        Param::bool("Crowd", "counts_self", "Count oneself in the group", Reset)
            .shown_if("network", "everyone")
            .with_help("Granovetter's example divides by the whole crowd, himself included: 63/120."),
        Param::choice(
            "Thresholds",
            "distribution",
            "Thresholds",
            &[
                ("uniform", "Uniform: 0, 1, …, N − 1"),
                ("perturbed", "Uniform, the 1 moved to 2"),
                ("normal", "Normal"),
                ("fixed", "Everyone the same"),
            ],
            Reset,
        ),
        Param::number("Thresholds", "mean", "Mean (or the threshold)", (0.0, 1.0, 0.01), Reset)
            .with_help("A fraction of the group: Granovetter's 25 % is 0.25; Watts's φ* 0.18."),
        Param::number("Thresholds", "sd", "Spread (sd)", (0.0, 1.0, 0.001), Reset)
            .shown_if("distribution", "normal"),
        Param::choice(
            "Thresholds",
            "crowd",
            "A normal crowd is",
            &[
                ("quantiles", "The normal's quantiles (Granovetter)"),
                ("sampled", "Drawn at random"),
            ],
            Reset,
        )
        .shown_if("distribution", "normal"),
        Param::choice(
            "Thresholds",
            "rounding",
            "Thresholds are",
            &[
                ("exact", "Fractions"),
                ("floor", "Whole people, rounded down"),
                ("nearest", "Whole people, rounded"),
            ],
            Reset,
        )
        .shown_if("distribution", "normal"),
        Param::choice(
            "Thresholds",
            "zero",
            "A threshold of 0",
            &[
                ("acts", "Acts at once"),
                ("when_reached", "Acts once a neighbor does"),
            ],
            Reset,
        ),
        Param::bool("Friends", "friends.enabled", "Friends count more", Reset)
            .shown_if("network", "everyone"),
        Param::number("Friends", "friends.acquaintance", "Acquaintance", (0.0, 1.0, 0.01), Reset)
            .shown_if("friends.enabled", "true")
            .with_help("The chance two actors are friends. Granovetter: the largest effect at about a quarter."),
        Param::integer("Friends", "friends.weight", "A friend counts as", (1, 20), Reset)
            .shown_if("friends.enabled", "true"),
        Param::bool("Friends", "friends.symmetric", "Friendship is mutual", Reset)
            .shown_if("friends.enabled", "true"),
        Param::choice(
            "Network",
            "network",
            "Who sees whom",
            &[
                ("everyone", "The whole crowd (Granovetter)"),
                ("random", "A random network (Watts)"),
                ("power_law", "A network with hubs (Watts, Fig. 4b)"),
            ],
            Reset,
        ),
        Param::number("Network", "degree", "Mean degree (z)", (0.0, 100.0, 0.01), Reset)
            .with_help("For the networks. With hubs it must be below 1.95."),
        Param::number("Ceilings", "ceilings.share", "Share who also leave", (0.0, 1.0, 0.01), Reset)
            .with_help("Granovetter's Fig. 3: join at a crowd, leave at a mob."),
        Param::number("Ceilings", "ceilings.at", "They leave above", (0.0, 1.0, 0.01), Reset),
        Param::bool("Clusters", "clusters.enabled", "Several crowds", Reset)
            .with_help("Crowds from the city, with people moving between them."),
        Param::integer("Clusters", "clusters.count", "Crowds", (2, 50), Reset)
            .shown_if("clusters.enabled", "true"),
        Param::number("Clusters", "clusters.movement", "Movement per step", (0.0, 1.0, 0.001), Reset)
            .shown_if("clusters.enabled", "true"),
        Param::bool("Episodes", "repeat", "Start again at each equilibrium", Live),
        Param::number("Episodes", "global", "A cascade is global above", (0.001, 1.0, 0.001), Live),
        Param::integer("Episodes", "max_steps", "An episode ends after", (1, 100_000), Live),
        Param::integer("Stopping", "stop_at", "Stop at step", (0, 10_000_000), Live)
            .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_granovetter_s_uniform_crowd() {
        let c = ThresholdsConfig::default();
        assert_eq!((c.actors, c.distribution), (100, Distribution::Uniform));
        assert_eq!(
            (c.network, c.trigger),
            (Network::Everyone, Trigger::Instigators)
        );
        assert!(c.counts_self && !c.reversible());
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = ThresholdsConfig {
            actors: 1,
            mean: 1.5,
            sd: -0.1,
            network: Network::PowerLaw,
            degree: 3.0,
            trigger: Trigger::Instigators,
            ceilings: Ceilings {
                share: 2.0,
                at: 0.9,
            },
            global: 0.0,
            max_steps: 0,
            stop_at: 20_000_000,
            ..ThresholdsConfig::default()
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
                "actors",
                "mean",
                "sd",
                "degree",
                "ceilings",
                "global",
                "max_steps",
                "stop_at"
            ]
        );
    }

    #[test]
    fn combinations_that_cannot_run_are_refused() {
        let friends_on_a_network = ThresholdsConfig {
            network: Network::Random,
            friends: Friends {
                enabled: true,
                ..Friends::default()
            },
            ..ThresholdsConfig::default()
        };
        assert_eq!(
            friends_on_a_network.validate().unwrap_err()[0].field,
            "friends"
        );
        let hub_without_network = ThresholdsConfig {
            trigger: Trigger::Hub,
            ..ThresholdsConfig::default()
        };
        assert_eq!(
            hub_without_network.validate().unwrap_err()[0].field,
            "trigger"
        );
        let clusters_of_a_fixed_crowd = ThresholdsConfig {
            clusters: Clusters {
                enabled: true,
                ..Clusters::default()
            },
            ..ThresholdsConfig::default()
        };
        assert_eq!(
            clusters_of_a_fixed_crowd.validate().unwrap_err()[0].field,
            "clusters"
        );
        let clusters = ThresholdsConfig {
            population: Population::City,
            ..clusters_of_a_fixed_crowd
        };
        assert!(clusters.validate().is_ok());
        assert_eq!(clusters.population_size(), 1000);
        assert!(clusters.reversible());
        let hubs = ThresholdsConfig {
            network: Network::PowerLaw,
            degree: 1.5,
            ..ThresholdsConfig::default()
        };
        assert!(hubs.validate().is_ok());
    }

    #[test]
    fn the_crowd_changes_only_on_reset() {
        let next = ThresholdsConfig {
            distribution: Distribution::Perturbed,
            repeat: true,
            ..ThresholdsConfig::default()
        };
        let changes = ThresholdsConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "distribution");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Thresholds(ThresholdsConfig {
            distribution: Distribution::Normal,
            ..ThresholdsConfig::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
