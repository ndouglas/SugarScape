//! The Timing of Retirement's parameters: Axtell and Epstein's (1999) cohorts,
//! rationals, randoms and imitators in transient social networks, the
//! policy switch from 65 to 62, and two loosely coupled sub-populations, with
//! every detail the texts leave open as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Whom an imitator counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Counts {
    /// "some fraction f of eligibles who have actually retired" (the text).
    Eligible,
    /// Every member of its network (footnote 5's alternative).
    All,
}

/// What becomes of a dead member's place in someone's network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Renewal {
    /// It passes to the 20-year-old reborn in the dead agent's slot (the
    /// pseudo-code's reused agent objects).
    Slot,
    /// The holder picks someone new within its own extent.
    Replace,
}

/// The order agents act in each period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// Cohorts oldest first, randomly within each (the footnote: "randomized
    /// within cohorts"; the cohort order is not given).
    ByCohort,
    /// One random order over everyone (the pseudo-code).
    Shuffled,
}

/// The initial population's death ages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialDeaths {
    /// U[60, 100] for everyone: those already past it die in period 1.
    Literal,
    /// U[max(age, 60), 100]: only ages still ahead.
    Survivors,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Size {
    pub min: u32,
    pub max: u32,
}

impl Default for Size {
    /// Table 6-1: U[10, 25].
    fn default() -> Self {
        Size { min: 10, max: 25 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    pub enabled: bool,
    /// The new eligibility age, once the norm is reached.
    pub to: u32,
}

impl Default for Policy {
    /// Congress's 1961 change: 65 to 62.
    fn default() -> Self {
        Policy {
            enabled: false,
            to: 62,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Groups {
    pub enabled: bool,
    /// The chance each network member is drawn from the other group.
    pub coupling: f64,
}

impl Default for Groups {
    /// AE's animation 6-4: "10% of each agent's network belongs to the other sub-population".
    fn default() -> Self {
        Groups {
            enabled: false,
            coupling: 0.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RetirementConfig {
    /// C: agents in each of the 81 initial cohorts.
    pub per_cohort: u32,
    pub rational: f64,
    pub random: f64,
    /// A random agent's chance of retiring each eligible period.
    pub p: f64,
    /// τ: the imitators' mean threshold.
    pub threshold: f64,
    /// τ's standard deviation, spread uniformly.
    pub spread: f64,
    pub size: Size,
    /// E ~ U[0, extent].
    pub extent: u32,
    pub counts: Counts,
    pub renewal: Renewal,
    pub order: Order,
    pub initial_deaths: InitialDeaths,
    pub eligibility: u32,
    /// Everyone still working retires at this age (0: none).
    pub mandatory: u32,
    pub policy: Policy,
    pub groups: Groups,
    /// The share of eligible agents retired that marks the norm.
    pub norm: f64,
    /// Stop once the norm (with the policy, the new norm) is reached.
    pub stop_at_norm: bool,
    /// Stop at this period (0: never).
    pub stop_at: u32,
}

impl Default for RetirementConfig {
    /// Table 6-1's base case.
    fn default() -> Self {
        RetirementConfig {
            per_cohort: 100,
            rational: 0.10,
            random: 0.05,
            p: 0.5,
            threshold: 0.5,
            spread: 0.0,
            size: Size::default(),
            extent: 5,
            counts: Counts::Eligible,
            renewal: Renewal::Slot,
            order: Order::ByCohort,
            initial_deaths: InitialDeaths::Literal,
            eligibility: 65,
            mandatory: 0,
            policy: Policy::default(),
            groups: Groups::default(),
            norm: 0.95,
            stop_at_norm: false,
            stop_at: 0,
        }
    }
}

/// The youngest and oldest ages, and the cohorts between them.
pub const YOUNGEST: u32 = 20;
pub const OLDEST: u32 = 100;
pub const COHORTS: u32 = OLDEST - YOUNGEST + 1;

impl RetirementConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (1..=500).contains(&self.per_cohort),
            "per_cohort",
            "must be between 1 and 500",
        );
        check(
            unit(self.rational) && unit(self.random) && self.rational + self.random <= 1.0,
            "rational",
            "the rational and random shares must be between 0 and 1 and sum to at most 1",
        );
        check(unit(self.p), "p", "must be between 0 and 1");
        check(unit(self.threshold), "threshold", "must be between 0 and 1");
        check(
            (0.0..=0.5).contains(&self.spread),
            "spread",
            "must be between 0 and 0.5",
        );
        check(
            self.size.min <= self.size.max && self.size.max <= 200,
            "size",
            "needs min ≤ max ≤ 200",
        );
        check(self.extent <= 40, "extent", "must be at most 40");
        let age = |a: u32| (YOUNGEST..=OLDEST).contains(&a);
        check(
            age(self.eligibility),
            "eligibility",
            "must be between 20 and 100",
        );
        check(
            self.mandatory <= OLDEST,
            "mandatory",
            "must be at most 100 (0: none)",
        );
        check(
            !self.policy.enabled || age(self.policy.to),
            "policy",
            "the new eligibility age must be between 20 and 100",
        );
        check(
            unit(self.groups.coupling),
            "groups",
            "the coupling must be between 0 and 1",
        );
        check(
            self.norm > 0.0 && self.norm <= 1.0,
            "norm",
            "must be above 0 and at most 1",
        );
        check(
            self.stop_at <= 1_000_000,
            "stop_at",
            "must be at most 1000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &RetirementConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("per_cohort", self.per_cohort == next.per_cohort),
            ("rational", self.rational == next.rational),
            ("random", self.random == next.random),
            ("threshold", self.threshold == next.threshold),
            ("spread", self.spread == next.spread),
            ("size", self.size == next.size),
            ("extent", self.extent == next.extent),
            ("renewal", self.renewal == next.renewal),
            ("initial_deaths", self.initial_deaths == next.initial_deaths),
            ("groups", self.groups == next.groups),
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
        Param::integer(
            "Population",
            "per_cohort",
            "Agents per cohort (C)",
            (1, 500),
            Reset,
        )
        .with_help("81 cohorts, ages 20 to 100. Axtell and Epstein: 100."),
        Param::choice(
            "Population",
            "initial_deaths",
            "The first agents' death ages",
            &[
                ("literal", "U[60, 100] (those past it die at once)"),
                ("survivors", "Only ages still ahead"),
            ],
            Reset,
        ),
        Param::choice(
            "Population",
            "order",
            "Each period, agents act",
            &[
                ("by_cohort", "Cohort by cohort, oldest first (the footnote)"),
                ("shuffled", "In one random order (the pseudo-code)"),
            ],
            Live,
        ),
        Param::number(
            "Agents",
            "rational",
            "Rational share",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .with_help("Retire as soon as they may."),
        Param::number("Agents", "random", "Random share", (0.0, 1.0, 0.01), Reset)
            .with_help("Retire with probability p each eligible period. The rest imitate."),
        Param::number(
            "Agents",
            "p",
            "Random agents' chance (p)",
            (0.0, 1.0, 0.01),
            Live,
        ),
        Param::number(
            "Agents",
            "threshold",
            "Imitation threshold (τ)",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .with_help("An imitator retires once this share of its network has."),
        Param::number(
            "Agents",
            "spread",
            "Threshold spread (sd)",
            (0.0, 0.5, 0.01),
            Reset,
        )
        .with_help("Uniform, as every random variable in the paper."),
        Param::choice(
            "Agents",
            "counts",
            "Imitators count",
            &[
                ("eligible", "Eligible members (the text)"),
                ("all", "Every member (footnote 5)"),
            ],
            Live,
        ),
        Param::range(
            "Networks",
            "size",
            "Network size (S)",
            (0.0, 200.0, 1.0),
            Reset,
        ),
        Param::integer(
            "Networks",
            "extent",
            "Extent (E up to, cohorts)",
            (0, 40),
            Reset,
        ),
        Param::choice(
            "Networks",
            "renewal",
            "When a member dies",
            &[
                ("slot", "The newborn in its slot takes its place"),
                ("replace", "Replaced within the holder's extent"),
            ],
            Reset,
        ),
        Param::integer("Policy", "eligibility", "Eligibility age", (20, 100), Live),
        Param::integer("Policy", "mandatory", "Mandatory age", (0, 100), Live)
            .with_help("0: none. Axtell and Epstein's policy runs: 70."),
        Param::bool(
            "Policy",
            "policy.enabled",
            "Lower the age once the norm is reached",
            Live,
        ),
        Param::integer("Policy", "policy.to", "To", (20, 100), Live)
            .shown_if("policy.enabled", "true"),
        Param::number(
            "Policy",
            "norm",
            "The norm is reached at",
            (0.01, 1.0, 0.01),
            Live,
        )
        .with_help(
            "The share of eligible agents retired. The paper never defines its transition time.",
        ),
        Param::bool("Groups", "groups.enabled", "Two sub-populations", Reset)
            .with_help("Every cohort halved; rationals only in the second half."),
        Param::number(
            "Groups",
            "groups.coupling",
            "Coupling",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .shown_if("groups.enabled", "true"),
        Param::bool("Stopping", "stop_at_norm", "Stop at the norm", Live),
        Param::integer(
            "Stopping",
            "stop_at",
            "Stop at period",
            (0, 1_000_000),
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
    fn defaults_are_table_6_1() {
        let c = RetirementConfig::default();
        assert_eq!(
            (c.per_cohort, c.rational, c.random, c.p),
            (100, 0.10, 0.05, 0.5)
        );
        assert_eq!(
            (c.threshold, c.size.min, c.size.max, c.extent),
            (0.5, 10, 25, 5)
        );
        assert_eq!((c.eligibility, c.mandatory), (65, 0));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = RetirementConfig {
            per_cohort: 0,
            rational: 0.7,
            random: 0.5,
            p: 2.0,
            threshold: -0.1,
            spread: 0.6,
            size: Size { min: 30, max: 20 },
            extent: 50,
            eligibility: 10,
            mandatory: 150,
            policy: Policy {
                enabled: true,
                to: 120,
            },
            groups: Groups {
                enabled: true,
                coupling: 1.5,
            },
            norm: 0.0,
            stop_at: 2_000_000,
            ..RetirementConfig::default()
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
                "per_cohort",
                "rational",
                "p",
                "threshold",
                "spread",
                "size",
                "extent",
                "eligibility",
                "mandatory",
                "policy",
                "groups",
                "norm",
                "stop_at"
            ]
        );
    }

    #[test]
    fn the_population_changes_only_on_reset() {
        let next = RetirementConfig {
            renewal: Renewal::Replace,
            counts: Counts::All,
            ..RetirementConfig::default()
        };
        let changes = RetirementConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "renewal");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Retirement(RetirementConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
