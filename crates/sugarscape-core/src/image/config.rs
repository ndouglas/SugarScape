//! Image scoring's parameters: NS98's Fig. 1 by default (with LH01's
//! statement of the payoff offset), LH01's island model, errors, standing
//! and q strategies, and each unstated choice as a named switch.

use serde::{Deserialize, Serialize};

use super::strategy::{Class, Strategy};
use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// How many rounds a group plays in a generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundsKind {
    /// Exactly `rounds` (NS98, LH01 §2).
    #[default]
    Fixed,
    /// Each round is the last with probability 1/`rounds` (LH01 §3's
    /// stability analysis): at least one, `rounds` on average.
    Random,
}

/// NS98's "to avoid negative payoffs we add 0.1 in each interaction".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Offset {
    /// c to both donor and recipient every round (LH01: "as did Nowak &
    /// Sigmund"; FAIR23 does the same).
    #[default]
    Both,
    /// Nothing added.
    None,
}

/// What a donor knows of a recipient's score.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Information {
    /// Everyone sees every interaction (NS98 Figs. 1–2, LH01).
    #[default]
    Perfect,
    /// Each interaction is seen by the recipient and on average `observers`
    /// others; each member keeps its own record, 0 when unknown (NS98 Fig. 3).
    Observers,
}

/// What an observer writes into its record of the donor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Records {
    /// Its own record of the donor, one up or down by the action seen: what
    /// it has itself seen (FAIR23's `imagescoreothers`; NS98's Fig. 3
    /// reproduces with it).
    #[default]
    Tally,
    /// The donor's new score: its true score before the round, one up or
    /// down by the action seen (one sighting reveals the whole score).
    Score,
}

/// The first generation's strategies.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Initial {
    /// `"uniform"`: each agent's strategy uniform over the allowed set.
    #[default]
    #[serde(with = "uniform")]
    Uniform,
    /// `{"only": s}`: everyone plays `s`, but in each group the first
    /// round(`share` × n) members play `invader`.
    Seeded(Seeded),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seeded {
    pub only: Strategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invader: Option<Strategy>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub share: f64,
}

fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

/// `Initial::Uniform` as the string "uniform".
mod uniform {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str("uniform")
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<(), D::Error> {
        let s = String::deserialize(d)?;
        if s == "uniform" {
            Ok(())
        } else {
            Err(serde::de::Error::custom(format!(
                "expected \"uniform\", not {s:?}"
            )))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ImageConfig {
    /// g groups of n.
    pub groups: u32,
    pub group_size: u32,
    /// p: the chance an offspring's parent is drawn from its own group
    /// rather than the whole population.
    pub local: f64,
    /// m: rounds per group per generation (fixed, or the mean).
    pub rounds: u32,
    pub rounds_kind: RoundsKind,
    /// Benefit to the recipient, cost to the donor, payoff at the start.
    pub b: f64,
    pub c: f64,
    pub u0: f64,
    pub offset: Offset,
    /// Scores stay in −clamp … +clamp (0: unbounded). Binary scorers
    /// always live in {−1, 0}.
    pub clamp: u32,
    pub information: Information,
    /// With `observers`: the mean number of members besides the pair who
    /// see an interaction (each with probability observers / (n − 2)).
    pub observers: f64,
    pub records: Records,
    /// e: a donor does the other action.
    pub execution_error: f64,
    /// ε: an observer sees the other action.
    pub perception_error: f64,
    /// ν: an offspring's strategy is redrawn uniformly from the allowed set.
    pub mutation: f64,
    /// The classes allowed (mutation and a uniform start draw from them).
    pub strategies: Vec<Class>,
    pub initial: Initial,
    /// The last generation; the run stops there (0: never).
    pub end: u32,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for ImageConfig {
    /// NS98 Fig. 1: one group of 100, k −5 … +6 uniform, m = 125, b = 1,
    /// c = 0.1 added to both players each round, scores clamped at ±5,
    /// perfect information, no errors, no mutation.
    fn default() -> Self {
        ImageConfig {
            groups: 1,
            group_size: 100,
            local: 1.0,
            rounds: 125,
            rounds_kind: RoundsKind::Fixed,
            b: 1.0,
            c: 0.1,
            u0: 0.0,
            offset: Offset::Both,
            clamp: 5,
            information: Information::Perfect,
            observers: 10.0,
            records: Records::Tally,
            execution_error: 0.0,
            perception_error: 0.0,
            mutation: 0.0,
            strategies: vec![Class::K],
            initial: Initial::Uniform,
            end: 0,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world (from the next generation);
/// every other field rebuilds it.
pub const LIVE: [&str; 14] = [
    "local",
    "rounds",
    "rounds_kind",
    "b",
    "c",
    "u0",
    "offset",
    "clamp",
    "observers",
    "records",
    "execution_error",
    "perception_error",
    "mutation",
    "end",
];

/// The most agents in a world, and the most in a group.
pub const MAX_AGENTS: u32 = 40_000;
pub const MAX_GROUP: u32 = 500;
pub const MAX_ROUNDS: u32 = 20_000;
/// A generation's work budget: the rounds all groups play, and (with
/// private records, where each round may be watched by every member) the
/// rounds times the group size. Each keeps a step well under a second.
pub const MAX_ROUNDS_PLAYED: u64 = 2_000_000;
pub const MAX_SIGHTINGS: u64 = 50_000_000;

impl ImageConfig {
    /// Every strategy the allowed classes contain, in `strategies` order.
    pub fn allowed(&self) -> Vec<Strategy> {
        self.strategies.iter().flat_map(|c| c.members()).collect()
    }

    /// The score range: {−1, 0} with binary scorers, else ±clamp (a
    /// million when unbounded, beyond any run's reach).
    pub fn score_range(&self) -> (i32, i32) {
        if self.strategies.contains(&Class::Binary) {
            (-1, 0)
        } else if self.clamp == 0 {
            (-1_000_000, 1_000_000)
        } else {
            (-(self.clamp as i32), self.clamp as i32)
        }
    }

    /// Whether members keep private records (observers or perception errors).
    pub fn private(&self) -> bool {
        self.information == Information::Observers || self.perception_error > 0.0
    }

    /// The chance each member besides the pair sees an interaction.
    pub fn watch_probability(&self) -> f64 {
        match self.information {
            Information::Perfect => 1.0,
            Information::Observers if self.group_size > 2 => {
                (self.observers / f64::from(self.group_size - 2)).min(1.0)
            }
            Information::Observers => 0.0,
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
        let size_ok = (2..=MAX_GROUP).contains(&self.group_size);
        check(size_ok, "group_size", "must be between 2 and 500");
        let groups_ok = (1..=400).contains(&self.groups);
        check(groups_ok, "groups", "must be between 1 and 400");
        check(
            !(size_ok && groups_ok) || self.groups * self.group_size <= MAX_AGENTS,
            "groups",
            "groups × group size must be at most 40,000",
        );
        check(unit(self.local), "local", "must be between 0 and 1");
        let rounds_ok = (1..=MAX_ROUNDS).contains(&self.rounds);
        check(rounds_ok, "rounds", "must be between 1 and 20,000");
        if size_ok && groups_ok && rounds_ok {
            let played = u64::from(self.groups) * u64::from(self.rounds);
            check(
                played <= MAX_ROUNDS_PLAYED,
                "rounds",
                "groups × rounds must be at most 2,000,000 (the rounds a generation plays)",
            );
            check(
                played > MAX_ROUNDS_PLAYED
                    || !self.private()
                    || played * u64::from(self.group_size) <= MAX_SIGHTINGS,
                "rounds",
                "with private records (observers or perception errors), groups × rounds × group size must be at most 50,000,000",
            );
        }
        for (field, v) in [("b", self.b), ("c", self.c), ("u0", self.u0)] {
            check(
                v.is_finite() && (0.0..=1000.0).contains(&v),
                field,
                "must be a number between 0 and 1,000",
            );
        }
        check(self.clamp <= 100, "clamp", "must be between 0 and 100");
        check(
            self.observers.is_finite() && (0.0..=f64::from(MAX_GROUP)).contains(&self.observers),
            "observers",
            "must be a number between 0 and 500",
        );
        check(
            unit(self.execution_error),
            "execution_error",
            "must be between 0 and 1",
        );
        check(
            unit(self.perception_error),
            "perception_error",
            "must be between 0 and 1",
        );
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        e.extend(self.validate_strategies());
        e
    }

    fn validate_strategies(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        let s = &self.strategies;
        let mut sorted = s.clone();
        sorted.sort();
        sorted.dedup();
        if s.is_empty() {
            e.push(FieldError::new(
                "strategies",
                "must list at least one class",
            ));
        } else if sorted.len() != s.len() {
            e.push(FieldError::new("strategies", "must not repeat a class"));
        } else if s.contains(&Class::Binary)
            && s.iter()
                .any(|c| !matches!(c, Class::Binary | Class::Standing))
        {
            e.push(FieldError::new(
                "strategies",
                "binary scorers (scores 0 and −1) combine only with standing",
            ));
        } else if s.contains(&Class::Q) && self.clamp == 0 {
            e.push(FieldError::new(
                "strategies",
                "q strategies need bounded scores (clamp at least 1)",
            ));
        }
        if let Initial::Seeded(seed) = &self.initial {
            let allowed = |x: &Strategy| x.is_valid() && s.contains(&x.class());
            if !allowed(&seed.only) || !seed.invader.as_ref().is_none_or(allowed) {
                e.push(FieldError::new(
                    "initial",
                    "the starting strategies must be of the allowed classes",
                ));
            } else if !(0.0..=1.0).contains(&seed.share) {
                e.push(FieldError::new("initial", "share must be between 0 and 1"));
            }
        }
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
                match ModelConfig::Image(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Image(next)) => {
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
    pub fn changes(&self, next: &ImageConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !LIVE.contains(&k.as_str()))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields (`strategies` and `initial` come from presets,
/// files and links).
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::number("Game", "b", "Benefit b", (0.0, 10.0, 0.05), Live)
            .with_help("What the recipient gains when helped"),
        Param::number("Game", "c", "Cost c", (0.0, 10.0, 0.05), Live)
            .with_help("What helping costs the donor"),
        Param::number("Game", "u0", "Initial payoff u₀", (0.0, 50.0, 0.5), Live),
        Param::choice(
            "Game",
            "offset",
            "Payoff offset",
            &[
                ("both", "c to donor and recipient each round (LH01)"),
                ("none", "None"),
            ],
            Live,
        ),
        Param::integer("Population", "groups", "Groups", (1, 400), Reset),
        Param::integer("Population", "group_size", "Group size", (2, 500), Reset),
        Param::number(
            "Population",
            "local",
            "Local parents",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("The chance an offspring's parent comes from its own group (LH01's p)"),
        Param::integer(
            "Rounds",
            "rounds",
            "Rounds per generation",
            (1, 20_000),
            Live,
        ),
        Param::choice(
            "Rounds",
            "rounds_kind",
            "Number of rounds",
            &[
                ("fixed", "Fixed"),
                ("random", "Random (each the last with chance 1/m)"),
            ],
            Live,
        ),
        Param::choice(
            "Information",
            "information",
            "Information",
            &[
                ("perfect", "Perfect: everyone sees everything"),
                ("observers", "Observers: each keeps its own record"),
            ],
            Reset,
        ),
        Param::number(
            "Information",
            "observers",
            "Observers per interaction",
            (0.0, 100.0, 1.0),
            Live,
        )
        .with_help("Besides the recipient, who always sees (NS98: 10)")
        .shown_if("information", "observers"),
        Param::choice(
            "Information",
            "records",
            "An observer records",
            &[
                ("tally", "Its own record ± 1 (FAIR23)"),
                ("score", "The donor's new score"),
            ],
            Live,
        ),
        Param::integer(
            "Information",
            "clamp",
            "Score limit (0: none)",
            (0, 100),
            Live,
        ),
        Param::number(
            "Errors",
            "execution_error",
            "Execution error",
            (0.0, 1.0, 0.005),
            Live,
        )
        .with_help("The chance a donor does the other action"),
        Param::number(
            "Errors",
            "perception_error",
            "Perception error",
            (0.0, 1.0, 0.005),
            Live,
        )
        .with_help("The chance an observer sees the other action"),
        Param::number(
            "Evolution",
            "mutation",
            "Mutation rate",
            (0.0, 0.1, 0.0001),
            Live,
        )
        .with_help("The chance an offspring's strategy is redrawn from all allowed"),
        Param::integer(
            "Run",
            "end",
            "Last generation (0: never)",
            (0, 100_000),
            Live,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &ImageConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_ns98_fig_1_and_validates() {
        let c = ImageConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!(
            (c.groups, c.group_size, c.rounds, c.clamp),
            (1, 100, 125, 5)
        );
        assert_eq!((c.b, c.c, c.u0, c.mutation), (1.0, 0.1, 0.0, 0.0));
        assert_eq!((c.local, c.observers), (1.0, 10.0));
        assert_eq!(c.strategies, [Class::K]);
        assert_eq!(c.allowed().len(), 12);
        assert_eq!(c.score_range(), (-5, 5));
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["offset"], "both");
        assert_eq!(v["rounds_kind"], "fixed");
        assert_eq!(v["information"], "perfect");
        assert_eq!(v["records"], "tally");
        assert_eq!(v["initial"], "uniform");
        assert_eq!(v["strategies"], json!(["k"]));
    }

    #[test]
    fn initial_reads_uniform_or_a_seeded_start() {
        let c: ImageConfig = serde_json::from_value(json!({
            "strategies": ["k", "h"],
            "initial": {"only": {"k": 0}, "invader": {"h": 1}, "share": 0.01}
        }))
        .unwrap();
        let Initial::Seeded(s) = &c.initial else {
            panic!("{:?}", c.initial)
        };
        assert_eq!(
            (s.only, s.invader, s.share),
            (Strategy::K(0), Some(Strategy::H(1)), 0.01)
        );
        assert!(c.validate().is_ok());
        let back = serde_json::to_value(&c).unwrap();
        assert_eq!(
            back["initial"],
            json!({"only": {"k": 0}, "invader": {"h": 1}, "share": 0.01})
        );
        let c: ImageConfig = serde_json::from_value(json!({"initial": "uniform"})).unwrap();
        assert_eq!(c.initial, Initial::Uniform);
        assert!(serde_json::from_value::<ImageConfig>(json!({"initial": "random"})).is_err());
        assert!(serde_json::from_value::<ImageConfig>(
            json!({"initial": {"only": {"k": 0}, "extra": 1}})
        )
        .is_err());
    }

    #[test]
    fn validation_names_the_field() {
        let bad = |edit: &dyn Fn(&mut ImageConfig)| {
            let mut c = ImageConfig::default();
            edit(&mut c);
            fields(&c)
        };
        assert_eq!(bad(&|c| c.group_size = 1), ["group_size"]);
        assert_eq!(bad(&|c| c.group_size = 501), ["group_size"]);
        assert_eq!(bad(&|c| c.groups = 0), ["groups"]);
        assert_eq!(bad(&|c| c.groups = 401), ["groups"]);
        assert_eq!(
            bad(&|c| {
                c.groups = 100;
                c.group_size = 401;
            }),
            ["groups"]
        );
        assert!(bad(&|c| c.groups = 400).is_empty());
        assert_eq!(bad(&|c| c.local = 1.5), ["local"]);
        assert_eq!(bad(&|c| c.rounds = 0), ["rounds"]);
        assert_eq!(bad(&|c| c.rounds = 20_001), ["rounds"]);
        assert!(bad(&|c| {
            c.groups = 100;
            c.rounds = 20_000;
        })
        .is_empty());
        assert_eq!(
            bad(&|c| {
                c.groups = 101;
                c.rounds = 20_000;
            }),
            ["rounds"]
        );
        assert!(bad(&|c| {
            c.groups = 25;
            c.rounds = 20_000;
            c.information = Information::Observers;
        })
        .is_empty());
        assert_eq!(
            bad(&|c| {
                c.groups = 26;
                c.rounds = 20_000;
                c.information = Information::Observers;
            }),
            ["rounds"]
        );
        assert_eq!(
            bad(&|c| {
                c.groups = 80;
                c.group_size = 500;
                c.rounds = 20_000;
                c.perception_error = 0.02;
            }),
            ["rounds"],
            "one error for the rounds, not two"
        );
        assert_eq!(
            bad(&|c| {
                c.groups = 80;
                c.group_size = 500;
                c.rounds = 1_251;
                c.perception_error = 0.02;
            }),
            ["rounds"]
        );
        let e = ImageConfig {
            groups: 101,
            rounds: 20_000,
            ..Default::default()
        }
        .validate()
        .unwrap_err();
        assert!(e[0].message.contains("2,000,000"), "{}", e[0].message);
        let e = ImageConfig {
            groups: 26,
            rounds: 20_000,
            information: Information::Observers,
            ..Default::default()
        }
        .validate()
        .unwrap_err();
        assert!(e[0].message.contains("50,000,000"), "{}", e[0].message);
        assert_eq!(bad(&|c| c.b = f64::NAN), ["b"]);
        assert_eq!(bad(&|c| c.c = -0.1), ["c"]);
        assert_eq!(bad(&|c| c.u0 = f64::INFINITY), ["u0"]);
        assert_eq!(bad(&|c| c.clamp = 101), ["clamp"]);
        assert_eq!(bad(&|c| c.observers = -1.0), ["observers"]);
        assert_eq!(bad(&|c| c.execution_error = 2.0), ["execution_error"]);
        assert_eq!(bad(&|c| c.perception_error = -0.1), ["perception_error"]);
        assert_eq!(bad(&|c| c.mutation = 1.1), ["mutation"]);
        assert_eq!(bad(&|c| c.strategies = vec![]), ["strategies"]);
        assert_eq!(
            bad(&|c| c.strategies = vec![Class::K, Class::K]),
            ["strategies"]
        );
        assert_eq!(
            bad(&|c| c.strategies = vec![Class::Binary, Class::K]),
            ["strategies"]
        );
        assert!(bad(&|c| c.strategies = vec![Class::Binary, Class::Standing]).is_empty());
        assert_eq!(
            bad(&|c| {
                c.strategies = vec![Class::Q];
                c.clamp = 0;
            }),
            ["strategies"]
        );
        let seeded = |only, invader, share| {
            Initial::Seeded(Seeded {
                only,
                invader,
                share,
            })
        };
        assert_eq!(
            bad(&|c| c.initial = seeded(Strategy::H(1), None, 0.0)),
            ["initial"]
        );
        assert_eq!(
            bad(&|c| c.initial = seeded(Strategy::K(9), None, 0.0)),
            ["initial"]
        );
        assert_eq!(
            bad(&|c| c.initial = seeded(Strategy::K(0), Some(Strategy::K(1)), 1.5)),
            ["initial"]
        );
        assert!(bad(&|c| c.initial = seeded(Strategy::K(0), Some(Strategy::K(1)), 0.5)).is_empty());
    }

    #[test]
    fn binary_scores_are_zero_and_minus_one_and_the_watch_chance_follows_n() {
        let mut c = ImageConfig {
            strategies: vec![Class::Binary, Class::Standing],
            ..Default::default()
        };
        assert_eq!(c.score_range(), (-1, 0));
        assert_eq!(c.allowed().len(), 4);
        c.strategies = vec![Class::K];
        c.clamp = 0;
        assert_eq!(c.score_range(), (-1_000_000, 1_000_000));
        assert_eq!(c.watch_probability(), 1.0);
        c.information = Information::Observers;
        c.group_size = 20;
        assert_eq!(c.watch_probability(), 10.0 / 18.0);
        c.group_size = 10;
        assert_eq!(c.watch_probability(), 1.0);
        c.group_size = 2;
        assert_eq!(c.watch_probability(), 0.0);
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = ImageConfig {
            schedule: vec![entry("c", json!(0.25))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("group_size", json!(40))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("mutation", json!(2))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = ImageConfig::default();
        let mut b = a.clone();
        b.c = 0.25;
        b.records = Records::Score;
        b.execution_error = 0.02;
        assert!(a.changes(&b).is_empty());
        b.groups = 10;
        b.strategies = vec![Class::And];
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["groups", "strategies"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: ImageConfig = serde_json::from_str(r#"{"rounds": 300, "offset": "none"}"#).unwrap();
        assert_eq!((c.rounds, c.offset, c.group_size), (300, Offset::None, 100));
        assert!(serde_json::from_str::<ImageConfig>(r#"{"visibility": 0.1}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Image(ImageConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
