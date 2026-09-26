//! The demographic Prisoner's Dilemma's parameters: the published text (GSS
//! Table 9.1, the CD's settings where the prose is silent) by default, with
//! the working paper's rule, Radax and Rengs' timing choices, soup and
//! metabolism as named switches.

use serde::{Deserialize, Serialize};

use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// When `metabolism` is charged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetabolismPer {
    /// Once per cycle, on the agent's own turn (WP note 29).
    #[default]
    Cycle,
    /// To both players after every game (the text's "after every interaction").
    Interaction,
}

/// Whom a mover plays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Play {
    /// One game with each occupied von Neumann neighbour (GSS).
    #[default]
    EachNeighbor,
    /// One game with one random occupied neighbour (the working paper).
    RandomNeighbor,
}

/// Where agents meet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pairing {
    /// On the lattice: neighbours play, offspring go next door.
    #[default]
    Space,
    /// "Equiprobable random agent pairings": a random partner, random
    /// placement anywhere.
    Soup,
}

/// When an agent whose wealth goes negative dies (RR's "die immediately").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathTiming {
    /// At once, even on another agent's turn.
    #[default]
    Immediate,
    /// At the end of its own next turn, if still negative.
    OwnTurn,
}

/// When a dead agent leaves its site (RR's "remove dead agents immediately").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Removal {
    /// At once.
    #[default]
    Immediate,
    /// At the cycle's end; until then it blocks its site and takes no part.
    EndOfCycle,
}

/// Where an offspring's endowment comes from (RR's "initial endowment inherited").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndowmentFrom {
    /// Subtracted from the parent's wealth (the text).
    #[default]
    Parent,
    /// Granted without cost to the parent.
    Granted,
}

/// An offspring's starting age (RR's "random birth age").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewbornAge {
    /// Uniform in 1 … `max_age`, like the initial agents (0 with no maximum).
    #[default]
    Random,
    /// 0.
    Zero,
}

/// How the cycle is scheduled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Updating {
    /// Each agent moves, plays, reproduces and ages in its turn.
    #[default]
    Asynchronous,
    /// All move, then all play, then all reproduce, then all age (RR's control).
    Synchronous,
}

/// How the agent list is reordered after each cycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shuffle {
    /// N/2 swaps of two random agents (GSS p. 206).
    #[default]
    Swaps,
    /// A full Fisher–Yates shuffle (RR's Repast list shuffle).
    Full,
}

/// When an offspring first takes a turn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewbornsAct {
    /// From the next cycle.
    #[default]
    NextCycle,
    /// In this cycle, after the agents already in the list.
    ThisCycle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DpdConfig {
    /// The torus is `width` × `width`.
    pub width: u32,
    /// Initial agents, on random empty sites.
    pub agents: u32,
    /// The chance an initial agent cooperates.
    pub initial_cooperators: f64,
    /// The initial agents' wealth (the CD's `Initial Wealth`).
    pub initial_wealth: f64,
    /// Payoffs: T to a defector against a cooperator, R to mutual
    /// cooperators, P to mutual defectors, S to a cooperator against a
    /// defector.
    pub t: f64,
    pub r: f64,
    pub p: f64,
    pub s: f64,
    /// An agent with at least this wealth may clone (the CD's `Fission Wealth`).
    pub fission_wealth: f64,
    /// An offspring's starting wealth.
    pub endowment: f64,
    /// The maximum age (0: none).
    pub max_age: u32,
    /// Wealth charged per cycle or per game.
    pub metabolism: f64,
    pub metabolism_per: MetabolismPer,
    /// The chance an offspring's strategy is the other one.
    pub mutation: f64,
    /// The movement radius (von Neumann distance).
    pub vision: u32,
    pub play: Play,
    pub pairing: Pairing,
    pub death_timing: DeathTiming,
    pub removal: Removal,
    pub endowment_from: EndowmentFrom,
    pub newborn_age: NewbornAge,
    pub updating: Updating,
    pub shuffle: Shuffle,
    pub newborns_act: NewbornsAct,
    /// The last cycle; the run stops there (0: never).
    pub end: u32,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for DpdConfig {
    /// GSS Table 9.1 (Run 1) with the CD's initial wealth 6 and fission
    /// wealth 11: a 30 × 30 torus, 100 agents, payoffs 6, 5, −5, −6, no
    /// maximum age, no mutation, no metabolism, vision 1.
    fn default() -> Self {
        DpdConfig {
            width: 30,
            agents: 100,
            initial_cooperators: 0.5,
            initial_wealth: 6.0,
            t: 6.0,
            r: 5.0,
            p: -5.0,
            s: -6.0,
            fission_wealth: 11.0,
            endowment: 6.0,
            max_age: 0,
            metabolism: 0.0,
            metabolism_per: MetabolismPer::Cycle,
            mutation: 0.0,
            vision: 1,
            play: Play::EachNeighbor,
            pairing: Pairing::Space,
            death_timing: DeathTiming::Immediate,
            removal: Removal::Immediate,
            endowment_from: EndowmentFrom::Parent,
            newborn_age: NewbornAge::Random,
            updating: Updating::Asynchronous,
            shuffle: Shuffle::Swaps,
            newborns_act: NewbornsAct::NextCycle,
            end: 0,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 20] = [
    "t",
    "r",
    "p",
    "s",
    "fission_wealth",
    "endowment",
    "max_age",
    "metabolism",
    "metabolism_per",
    "mutation",
    "play",
    "pairing",
    "death_timing",
    "removal",
    "endowment_from",
    "newborn_age",
    "updating",
    "shuffle",
    "newborns_act",
    "end",
];

impl DpdConfig {
    /// The payoff to a player with strategy `me` against `other` (true:
    /// cooperate).
    pub fn payoff(&self, me: bool, other: bool) -> f64 {
        match (me, other) {
            (true, true) => self.r,
            (true, false) => self.s,
            (false, true) => self.t,
            (false, false) => self.p,
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = self.validate_fields();
        e.extend(self.validate_schedule());
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    fn validate_fields(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |x: f64| (0.0..=1.0).contains(&x);
        let nonneg = |x: f64| x.is_finite() && x >= 0.0;
        let width_ok = (3..=200).contains(&self.width);
        check(width_ok, "width", "must be between 3 and 200");
        check(
            !width_ok || self.agents <= self.width * self.width,
            "agents",
            "must be at most width × width",
        );
        check(
            unit(self.initial_cooperators),
            "initial_cooperators",
            "must be between 0 and 1",
        );
        check(
            nonneg(self.initial_wealth),
            "initial_wealth",
            "must be a number ≥ 0",
        );
        for (field, v) in [("t", self.t), ("r", self.r), ("p", self.p), ("s", self.s)] {
            check(
                v.is_finite() && v.abs() <= 1e6,
                field,
                "must be a number with magnitude at most 1,000,000",
            );
        }
        check(
            nonneg(self.fission_wealth),
            "fission_wealth",
            "must be a number ≥ 0",
        );
        check(nonneg(self.endowment), "endowment", "must be a number ≥ 0");
        check(
            nonneg(self.metabolism),
            "metabolism",
            "must be a number ≥ 0",
        );
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        check(
            (1..=10).contains(&self.vision),
            "vision",
            "must be between 1 and 10",
        );
        e
    }

    /// Schedule entries: live paths only, values that validate.
    fn validate_schedule(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        for change in &self.schedule {
            for (path, value) in &change.set {
                if !LIVE.contains(&path.as_str()) {
                    e.push(FieldError::new(
                        "schedule",
                        format!("{path} changes only on reset and cannot be scheduled"),
                    ));
                    continue;
                }
                match ModelConfig::Dpd(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Dpd(next)) => {
                        for f in next.validate_fields() {
                            e.push(FieldError::new(
                                "schedule",
                                format!("tick {}: {}: {}", change.tick, f.field, f.message),
                            ));
                        }
                    }
                    Ok(_) => unreachable!("with_path keeps the model"),
                    Err(f) => e.push(f),
                }
            }
        }
        e
    }

    /// The reset-only fields that differ from `next`.
    pub fn changes(&self, next: &DpdConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !LIVE.contains(&k.as_str()))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    let payoff = (-20.0, 20.0, 1.0);
    vec![
        Param::number("Game", "t", "T (defect against a cooperator)", payoff, Live)
            .with_help("The temptation: a defector's payoff against a cooperator"),
        Param::number("Game", "r", "R (both cooperate)", payoff, Live)
            .with_help("The reward to each of two cooperators"),
        Param::number("Game", "p", "P (both defect)", payoff, Live)
            .with_help("The punishment to each of two defectors"),
        Param::number(
            "Game",
            "s",
            "S (cooperate against a defector)",
            payoff,
            Live,
        )
        .with_help("The sucker's payoff: a cooperator's against a defector"),
        Param::integer(
            "Population",
            "width",
            "Width (a square torus)",
            (3, 200),
            Reset,
        ),
        Param::integer("Population", "agents", "Initial agents", (0, 40_000), Reset),
        Param::number(
            "Population",
            "initial_cooperators",
            "Initial cooperators",
            (0.0, 1.0, 0.05),
            Reset,
        )
        .with_help("The chance an initial agent cooperates"),
        Param::number(
            "Population",
            "initial_wealth",
            "Initial wealth",
            (0.0, 50.0, 1.0),
            Reset,
        ),
        Param::number(
            "Population",
            "fission_wealth",
            "Wealth to clone",
            (0.0, 100.0, 1.0),
            Live,
        )
        .with_help("An agent with at least this much has an offspring (the CD's 11)"),
        Param::number(
            "Population",
            "endowment",
            "Offspring's endowment",
            (0.0, 50.0, 1.0),
            Live,
        ),
        Param::integer(
            "Population",
            "max_age",
            "Maximum age (0: none)",
            (0, 100_000),
            Live,
        ),
        Param::number(
            "Population",
            "metabolism",
            "Metabolism",
            (0.0, 20.0, 1.0),
            Live,
        )
        .with_help("Wealth charged every cycle (or every game)"),
        Param::choice(
            "Population",
            "metabolism_per",
            "Metabolism charged",
            &[
                ("cycle", "Per cycle (the text's note)"),
                ("interaction", "Per game"),
            ],
            Live,
        ),
        Param::integer("Population", "vision", "Vision", (1, 10), Reset)
            .with_help("How far an agent can move (von Neumann)"),
        Param::number(
            "Evolution",
            "mutation",
            "Mutation rate",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("The chance an offspring's strategy is not its parent's"),
        Param::choice(
            "Timing",
            "death_timing",
            "Death",
            &[
                ("immediate", "As wealth goes negative"),
                ("own_turn", "At the end of its own turn"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "removal",
            "The dead leave",
            &[
                ("immediate", "At once"),
                ("end_of_cycle", "At the cycle's end"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "endowment_from",
            "Endowment",
            &[("parent", "Taken from the parent"), ("granted", "Granted")],
            Live,
        ),
        Param::choice(
            "Timing",
            "newborn_age",
            "Newborns' age",
            &[("random", "Random, 1 to the maximum age"), ("zero", "Zero")],
            Live,
        ),
        Param::choice(
            "Timing",
            "newborns_act",
            "Newborns act",
            &[
                ("next_cycle", "From the next cycle"),
                ("this_cycle", "In this cycle"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "updating",
            "Updating",
            &[
                ("asynchronous", "Asynchronous (each agent in turn)"),
                ("synchronous", "Synchronous (all move, all play, …)"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "shuffle",
            "Call order",
            &[
                ("swaps", "N/2 random swaps (Epstein)"),
                ("full", "A full shuffle"),
            ],
            Live,
        ),
        Param::choice(
            "Interaction",
            "pairing",
            "Pairing",
            &[
                ("space", "Space: neighbours"),
                ("soup", "Soup: random partners"),
            ],
            Live,
        ),
        Param::choice(
            "Interaction",
            "play",
            "Play",
            &[
                ("each_neighbor", "Each neighbour (the published text)"),
                (
                    "random_neighbor",
                    "One random neighbour (the working paper)",
                ),
            ],
            Live,
        )
        .shown_if("pairing", "space"),
        Param::integer("Run", "end", "Last cycle (0: never)", (0, 100_000), Live),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &DpdConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_table_9_1_with_the_cds_wealths_and_validates() {
        let c = DpdConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!((c.width, c.agents, c.vision, c.max_age), (30, 100, 1, 0));
        assert_eq!((c.t, c.r, c.p, c.s), (6.0, 5.0, -5.0, -6.0));
        assert_eq!(
            (c.fission_wealth, c.endowment, c.initial_wealth),
            (11.0, 6.0, 6.0)
        );
        assert_eq!((c.mutation, c.metabolism, c.end), (0.0, 0.0, 0));
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["play"], "each_neighbor");
        assert_eq!(v["death_timing"], "immediate");
        assert_eq!(v["removal"], "immediate");
        assert_eq!(v["endowment_from"], "parent");
        assert_eq!(v["newborn_age"], "random");
        assert_eq!(v["updating"], "asynchronous");
        assert_eq!(v["shuffle"], "swaps");
        assert_eq!(v["newborns_act"], "next_cycle");
        assert_eq!(v["metabolism_per"], "cycle");
        assert_eq!(v["pairing"], "space");
    }

    #[test]
    fn payoffs_follow_the_matrix() {
        let c = DpdConfig::default();
        assert_eq!(c.payoff(true, true), 5.0);
        assert_eq!(c.payoff(true, false), -6.0);
        assert_eq!(c.payoff(false, true), 6.0);
        assert_eq!(c.payoff(false, false), -5.0);
    }

    #[test]
    fn validation_names_the_field() {
        let bad = |edit: &dyn Fn(&mut DpdConfig)| {
            let mut c = DpdConfig::default();
            edit(&mut c);
            fields(&c)
        };
        assert_eq!(bad(&|c| c.width = 2), ["width"]);
        assert_eq!(bad(&|c| c.width = 201), ["width"]);
        assert_eq!(bad(&|c| c.agents = 901), ["agents"]);
        assert!(bad(&|c| c.agents = 900).is_empty());
        assert_eq!(
            bad(&|c| c.initial_cooperators = 1.5),
            ["initial_cooperators"]
        );
        assert_eq!(bad(&|c| c.initial_wealth = -1.0), ["initial_wealth"]);
        assert_eq!(bad(&|c| c.t = f64::NAN), ["t"]);
        assert_eq!(bad(&|c| c.r = f64::INFINITY), ["r"]);
        assert_eq!(bad(&|c| c.p = f64::NAN), ["p"]);
        assert_eq!(bad(&|c| c.s = f64::NEG_INFINITY), ["s"]);
        assert_eq!(bad(&|c| c.t = 1.7e308), ["t"]);
        assert_eq!(bad(&|c| c.t = 1_000_001.0), ["t"]);
        assert!(bad(&|c| c.t = 1_000_000.0).is_empty());
        assert_eq!(bad(&|c| c.fission_wealth = -1.0), ["fission_wealth"]);
        assert_eq!(bad(&|c| c.endowment = -0.5), ["endowment"]);
        assert_eq!(bad(&|c| c.metabolism = -1.0), ["metabolism"]);
        assert_eq!(bad(&|c| c.mutation = 1.1), ["mutation"]);
        assert_eq!(bad(&|c| c.vision = 0), ["vision"]);
        assert_eq!(bad(&|c| c.vision = 11), ["vision"]);
        // No ordering or sign is imposed on the payoffs.
        assert!(bad(&|c| {
            c.t = -3.0;
            c.r = 1.0;
            c.p = 1.0;
            c.s = -3.0;
        })
        .is_empty());
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = DpdConfig {
            schedule: vec![entry("r", json!(1.0))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("width", json!(40))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("mutation", json!(2))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = DpdConfig::default();
        let mut b = a.clone();
        b.r = 1.0;
        b.max_age = 100;
        b.updating = Updating::Synchronous;
        assert!(a.changes(&b).is_empty());
        b.width = 40;
        b.vision = 2;
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["vision", "width"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: DpdConfig =
            serde_json::from_str(r#"{"max_age": 100, "removal": "end_of_cycle"}"#).unwrap();
        assert_eq!(
            (c.max_age, c.removal, c.width),
            (100, Removal::EndOfCycle, 30)
        );
        assert!(serde_json::from_str::<DpdConfig>(r#"{"threshold": 10}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Dpd(DpdConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
