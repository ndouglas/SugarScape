//! Altruistic Punishment's parameters: Boyd, Gintis, Bowles and Richerson's
//! (2003) groups of contributors, defectors and punishers, with payoff-biased
//! imitation, mixing, intergroup conflict and mutation, the variants their
//! text describes, and every detail it leaves open as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// What a punisher pays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Punishing {
    /// k/n for each defector it punishes (the text).
    Variable,
    /// A flat cost each period, whatever the group (Fig. 4).
    Fixed,
}

/// How groups meet in conflict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pairing {
    /// "groups are paired at random, and with probability ε" each pair fights.
    Paired,
    /// Groups paired at random, and either group of a pair can start the
    /// conflict, each with probability ε: a pair fights with probability
    /// 2ε − ε² (the reading the figures fit).
    Either,
    /// Each group, in random order, challenges a random group not yet
    /// fighting with probability ε (Janssen's reading; about twice the rate).
    Challenge,
}

/// Who wins a conflict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Victory {
    /// ½(1 + dⱼ − dᵢ): the group with fewer defectors (the text).
    Defectors,
    /// ½ + ½(Ḡᵢ − Ḡⱼ)/(G_max − G_min): average payoffs normalized by the
    /// widest possible difference (Cooney's eq. 3.18).
    Payoff,
    /// ½ + ½ tanh(s(Ḡᵢ − Ḡⱼ)): Cooney's group-level Fermi rule.
    Tanh,
}

/// What d, a group's share of defectors, counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Counted {
    /// Defector types (1 − the mean cooperation trait).
    Types,
    /// Those who defected this period (Janssen).
    Acts,
}

/// What a punisher who errs and defects does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Erring {
    /// Still punishes the other defectors.
    Others,
    /// Punishes nobody.
    None,
    /// Punishes the other defectors and itself (Janssen).
    #[serde(rename = "self")]
    Itself,
}

/// When imitation takes effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Imitation {
    /// Everyone imitates from the period's starting traits.
    Together,
    /// In turn: later agents see earlier agents' changes (Janssen).
    InTurn,
}

/// What replaces a defeated group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refill {
    /// A copy of the winners, member for member.
    Copy,
    /// Both groups refilled by drawing the winners' members with replacement.
    Split,
}

/// What an agent's traits can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Traits {
    /// Contributors, defectors and punishers.
    Discrete,
    /// Cooperation and punishment in [0, 1]; mutants uniform.
    Continuous,
}

/// How groups are arranged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Structure {
    /// Migrants from any group; conflict between random pairs.
    Groups,
    /// A ring: migrants only from the two neighbors, and no conflict.
    Ring,
}

/// The first period's population.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// "one group consisted of all altruistic punishers and the other 127
    /// groups were all defectors".
    OnePunisherGroup,
    AllDefectors,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PunishmentConfig {
    /// N.
    pub groups: u32,
    /// n.
    pub size: u32,
    /// c: the cost of cooperating.
    pub cost: f64,
    /// k: a punisher pays k/n per defector.
    pub punish_cost: f64,
    /// p: a defector pays p/n per punisher.
    pub fine: f64,
    pub punishing: Punishing,
    /// The flat cost under `Punishing::Fixed`.
    pub fixed_cost: f64,
    /// b: each cooperative act gives b/n to every other member.
    pub benefit: f64,
    /// The payoff the game's costs and benefits are added to.
    pub baseline: f64,
    /// e.
    pub error: f64,
    /// m.
    pub mixing: f64,
    /// μ.
    pub mutation: f64,
    /// ε.
    pub conflict: f64,
    pub pairing: Pairing,
    pub victory: Victory,
    /// s, for `Victory::Tanh`.
    pub sensitivity: f64,
    pub counted: Counted,
    pub erring: Erring,
    pub imitation: Imitation,
    pub refill: Refill,
    pub traits: Traits,
    pub structure: Structure,
    pub start: Start,
    /// `long_run` averages over the last `window` periods before `stop_at`.
    pub window: u32,
    /// Stop at this period (0: never).
    pub stop_at: u32,
}

impl Default for PunishmentConfig {
    /// The base case: 128 groups of 32, c = k = 0.2, p = 0.8, e = 0.02,
    /// m = 0.01, μ = 0.01, ε = 0.015, 2 000 periods.
    fn default() -> Self {
        PunishmentConfig {
            groups: 128,
            size: 32,
            cost: 0.2,
            punish_cost: 0.2,
            fine: 0.8,
            punishing: Punishing::Variable,
            fixed_cost: 0.2,
            benefit: 0.0,
            baseline: 1.0,
            error: 0.02,
            mixing: 0.01,
            mutation: 0.01,
            conflict: 0.015,
            pairing: Pairing::Paired,
            victory: Victory::Defectors,
            sensitivity: 10.0,
            counted: Counted::Types,
            erring: Erring::Others,
            imitation: Imitation::Together,
            refill: Refill::Copy,
            traits: Traits::Discrete,
            structure: Structure::Groups,
            start: Start::OnePunisherGroup,
            window: 1000,
            stop_at: 2000,
        }
    }
}

/// The most agents a world holds.
pub const MOST_AGENTS: u32 = 131_072;

impl PunishmentConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        let amount = |v: f64| (0.0..=10.0).contains(&v);
        check(
            (2..=512).contains(&self.groups),
            "groups",
            "must be between 2 and 512",
        );
        check(
            (2..=512).contains(&self.size) && self.groups * self.size <= MOST_AGENTS,
            "size",
            "must be between 2 and 512, with at most 131072 agents in all",
        );
        check(amount(self.cost), "cost", "must be between 0 and 10");
        check(
            amount(self.punish_cost),
            "punish_cost",
            "must be between 0 and 10",
        );
        check(amount(self.fine), "fine", "must be between 0 and 10");
        check(
            amount(self.fixed_cost),
            "fixed_cost",
            "must be between 0 and 10",
        );
        check(amount(self.benefit), "benefit", "must be between 0 and 10");
        check(
            (0.0..=100.0).contains(&self.baseline),
            "baseline",
            "must be between 0 and 100",
        );
        check(unit(self.error), "error", "must be between 0 and 1");
        check(unit(self.mixing), "mixing", "must be between 0 and 1");
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        check(unit(self.conflict), "conflict", "must be between 0 and 1");
        check(
            (0.0..=1000.0).contains(&self.sensitivity),
            "sensitivity",
            "must be between 0 and 1000",
        );
        check(
            (1..=1_000_000).contains(&self.window),
            "window",
            "must be between 1 and 1000000",
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
    pub(crate) fn structural_changes(&self, next: &PunishmentConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("groups", self.groups == next.groups),
            ("size", self.size == next.size),
            ("traits", self.traits == next.traits),
            ("structure", self.structure == next.structure),
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
        Param::integer("Groups", "groups", "Groups (N)", (2, 512), Reset)
            .with_help("Boyd and coauthors: 128."),
        Param::integer("Groups", "size", "Group size (n)", (2, 512), Reset)
            .with_help("Their figures run from 4 to 256."),
        Param::choice(
            "Groups",
            "start",
            "At the start",
            &[
                ("one_punisher_group", "One group of punishers, the rest defectors"),
                ("all_defectors", "Everyone defects"),
            ],
            Reset,
        ),
        Param::number("Game", "cost", "Cost of cooperating (c)", (0.0, 2.0, 0.01), Live),
        Param::number("Game", "fine", "Cost of being punished (p)", (0.0, 4.0, 0.01), Live)
            .with_help("A defector pays p/n for each punisher."),
        Param::choice(
            "Game",
            "punishing",
            "Punishers pay",
            &[
                ("variable", "k/n for each defector (the text)"),
                ("fixed", "A fixed cost every period (Fig. 4)"),
            ],
            Live,
        ),
        Param::number(
            "Game",
            "punish_cost",
            "Cost of punishing (k)",
            (0.0, 2.0, 0.01),
            Live,
        )
        .shown_if("punishing", "variable"),
        Param::number("Game", "fixed_cost", "Fixed cost", (0.0, 2.0, 0.01), Live)
            .shown_if("punishing", "fixed"),
        Param::number("Game", "error", "Errors (e)", (0.0, 1.0, 0.01), Live)
            .with_help("Contributors and punishers defect by mistake."),
        Param::choice(
            "Game",
            "erring",
            "A punisher who errs",
            &[
                ("others", "Still punishes the other defectors"),
                ("none", "Punishes nobody"),
                ("self", "Punishes itself too (Janssen)"),
            ],
            Live,
        ),
        Param::number("Game", "benefit", "Benefit to others (b)", (0.0, 4.0, 0.01), Live)
            .with_help("Each cooperative act gives b/n to every other member. The base model: 0."),
        Param::number("Game", "baseline", "Baseline payoff", (0.0, 10.0, 0.1), Live)
            .with_help(
                "What costs and fines are subtracted from. Never stated; 1 fits their 50-period calibration.",
            ),
        Param::number("Imitation", "mixing", "Mixing (m)", (0.0, 1.0, 0.001), Live)
            .with_help("The chance the one imitated comes from another group."),
        Param::choice(
            "Imitation",
            "imitation",
            "Imitation happens",
            &[
                ("together", "All at once"),
                ("in_turn", "In turn (Janssen)"),
            ],
            Live,
        ),
        Param::number("Imitation", "mutation", "Mutation (μ)", (0.0, 1.0, 0.001), Live),
        Param::number("Conflict", "conflict", "Conflict (ε)", (0.0, 1.0, 0.001), Live),
        Param::choice(
            "Conflict",
            "pairing",
            "Groups meet",
            &[
                ("paired", "In random pairs (the text)"),
                ("either", "In random pairs; either can start it (the figures)"),
                ("challenge", "Each challenges one (Janssen)"),
            ],
            Live,
        ),
        Param::choice(
            "Conflict",
            "victory",
            "Groups fight over",
            &[
                ("defectors", "Their share of defectors (the text)"),
                ("payoff", "Their payoffs, normalized (Cooney)"),
                ("tanh", "Their payoffs, through tanh (Cooney)"),
            ],
            Live,
        ),
        Param::number(
            "Conflict",
            "sensitivity",
            "Sensitivity (s)",
            (0.0, 100.0, 0.1),
            Live,
        )
        .shown_if("victory", "tanh"),
        Param::choice(
            "Conflict",
            "counted",
            "Defectors are counted by",
            &[
                ("types", "Type"),
                ("acts", "What they did this period (Janssen)"),
            ],
            Live,
        ),
        Param::choice(
            "Conflict",
            "refill",
            "A defeated group",
            &[
                ("copy", "Becomes a copy of the winners"),
                ("split", "Is refilled, with the winners, from the winners"),
            ],
            Live,
        ),
        Param::choice(
            "Variants",
            "traits",
            "Traits",
            &[
                ("discrete", "Contributors, defectors, punishers"),
                ("continuous", "Cooperate and punish by degrees"),
            ],
            Reset,
        ),
        Param::choice(
            "Variants",
            "structure",
            "Groups are",
            &[
                ("groups", "Anywhere, with conflict"),
                ("ring", "On a ring, without conflict"),
            ],
            Reset,
        ),
        Param::integer(
            "Stopping",
            "window",
            "Long-run window (periods)",
            (1, 1_000_000),
            Live,
        )
        .with_help("The long-run average covers this many periods before the stop."),
        Param::integer(
            "Stopping",
            "stop_at",
            "Stop at period",
            (0, 1_000_000),
            Live,
        )
        .with_help("0: never. Boyd and coauthors: 2000."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_the_base_case() {
        let c = PunishmentConfig::default();
        assert_eq!(
            (c.groups, c.size, c.stop_at, c.window),
            (128, 32, 2000, 1000)
        );
        assert_eq!((c.cost, c.punish_cost, c.fine), (0.2, 0.2, 0.8));
        assert_eq!(
            (c.error, c.mixing, c.mutation, c.conflict),
            (0.02, 0.01, 0.01, 0.015)
        );
        assert_eq!((c.benefit, c.baseline), (0.0, 1.0));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = PunishmentConfig {
            groups: 1,
            size: 600,
            cost: -1.0,
            punish_cost: 11.0,
            fine: f64::NAN,
            fixed_cost: 20.0,
            benefit: -0.5,
            baseline: 101.0,
            error: 1.5,
            mixing: -0.1,
            mutation: 2.0,
            conflict: 1.1,
            sensitivity: -1.0,
            window: 0,
            stop_at: 2_000_000,
            ..PunishmentConfig::default()
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
                "groups",
                "size",
                "cost",
                "punish_cost",
                "fine",
                "fixed_cost",
                "benefit",
                "baseline",
                "error",
                "mixing",
                "mutation",
                "conflict",
                "sensitivity",
                "window",
                "stop_at"
            ]
        );
        let crowded = PunishmentConfig {
            groups: 512,
            size: 512,
            ..PunishmentConfig::default()
        };
        assert_eq!(crowded.validate().unwrap_err()[0].field, "size");
    }

    #[test]
    fn the_population_changes_only_on_reset() {
        let next = PunishmentConfig {
            traits: Traits::Continuous,
            fine: 0.4,
            victory: Victory::Payoff,
            ..PunishmentConfig::default()
        };
        let changes = PunishmentConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "traits");
    }

    #[test]
    fn erring_self_reads_and_writes_as_self() {
        let c: PunishmentConfig = serde_json::from_str(r#"{"erring": "self"}"#).unwrap();
        assert_eq!(c.erring, Erring::Itself);
        assert!(serde_json::to_string(&c)
            .unwrap()
            .contains(r#""erring":"self""#));
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Punishment(PunishmentConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
