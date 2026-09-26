//! The ethnocentrism model's parameters: HA06's text by default, with its
//! appendix, its authors' code and its critics' variants as named switches.

use serde::{Deserialize, Serialize};

use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// How the lattice starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// Empty; immigration fills it (HA06).
    #[default]
    Empty,
    /// Every site holds an agent with random traits (the archived Java).
    Random,
    /// Every site holds an agent with a random tag that helps no one (HA06's
    /// "full lattice of egoists").
    Selfish,
}

/// How often each agent decides whether to help each neighbor in a period.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairPlay {
    /// Once per neighbor (HA-Java, NetLogo).
    #[default]
    Once,
    /// Twice: HA06's appendix loop read literally.
    Twice,
}

/// What an agent's help decision can depend on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Discrimination {
    /// Two bits: help the same colour, help other colours.
    #[default]
    SameOther,
    /// One bit: help everyone or no one (HA06's "unable to distinguish").
    None,
    /// One bit per colour (HA06's "distinguish all four colors").
    EachColor,
}

/// Where an offspring goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Offspring {
    /// A random empty neighbor, or nowhere (HA06).
    #[default]
    Adjacent,
    /// A random empty site anywhere (J13 §3.6).
    Anywhere,
}

/// How an offspring inherits the kin-strategies basis bit (tag or kin
/// marker). J13 does not say.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KinBasis {
    /// Like the strategy bits: it flips with probability `mutation`.
    #[default]
    Mutates,
    /// Drawn at immigration and never mutates.
    Fixed,
}

/// A strategy as the statistics and Inspect name it. `allowed` lists only
/// the first four.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Strategy {
    /// Helps its own colour (or kin) only.
    #[serde(rename = "E")]
    Ethnocentric,
    /// Helps everyone.
    #[serde(rename = "H")]
    Humanitarian,
    /// Helps no one.
    #[serde(rename = "S")]
    Selfish,
    /// Helps other colours only.
    #[serde(rename = "T")]
    Traitorous,
    /// Helps its kin (same kin marker) only (J13 §5.2).
    #[serde(rename = "kin")]
    Kin,
    /// Helps non-kin only.
    #[serde(rename = "nonkin")]
    Nonkin,
    /// Each colour: helps some other pattern of colours.
    #[serde(rename = "mixed")]
    Mixed,
}

impl Strategy {
    /// The four HA06 strategies, as `allowed` lists them by default.
    pub const FOUR: [Strategy; 4] = [
        Strategy::Ethnocentric,
        Strategy::Humanitarian,
        Strategy::Selfish,
        Strategy::Traitorous,
    ];

    /// The strategy of the two bits (help same, help other).
    pub fn of_bits(same: bool, other: bool) -> Strategy {
        match (same, other) {
            (true, false) => Strategy::Ethnocentric,
            (true, true) => Strategy::Humanitarian,
            (false, false) => Strategy::Selfish,
            (false, true) => Strategy::Traitorous,
        }
    }

    pub fn letter(self) -> &'static str {
        match self {
            Strategy::Ethnocentric => "E",
            Strategy::Humanitarian => "H",
            Strategy::Selfish => "S",
            Strategy::Traitorous => "T",
            Strategy::Kin => "kin",
            Strategy::Nonkin => "nonkin",
            Strategy::Mixed => "mixed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EthnoConfig {
    /// The torus is `width` × `width`.
    pub width: u32,
    /// The number of tags.
    pub colors: u32,
    pub start: Start,
    /// Immigrants per period: ⌊r⌋, plus one more with probability r − ⌊r⌋.
    pub immigration: f64,
    /// PTR at the start of each interaction step.
    pub base_ptr: f64,
    /// PTR a helper loses.
    pub cost: f64,
    /// PTR the helped gains.
    pub benefit: f64,
    /// Per-period death probability.
    pub death: f64,
    /// Per strategy trait (and per tag unless `tag_mutation` is set).
    pub mutation: f64,
    /// The tag's own mutation rate (J13 §4.4); `None` = `mutation`.
    pub tag_mutation: Option<f64>,
    pub pair_play: PairPlay,
    pub discrimination: Discrimination,
    /// Probability a decision misjudges same/other (`same_other` only).
    pub misperception: f64,
    pub offspring: Offspring,
    /// The strategies immigrants and mutations may produce (HKS13 Study 2).
    pub allowed: Vec<Strategy>,
    /// Adds a basis bit: discriminate on the tag or on the kin marker (J13 §5.2).
    pub kin_strategies: bool,
    /// Whether the basis bit mutates (`kin_strategies` only).
    pub kin_basis: KinBasis,
    /// The kin marker's mutation rate: an offspring founds a new family.
    pub kin_mutation: f64,
    /// The last period; the run stops there (0: never).
    pub end: u32,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for EthnoConfig {
    /// HA06's text: an empty 50 × 50 torus, four colours, one immigrant a
    /// period, PTR 0.12, cost 0.01, benefit 0.03, death 0.10, mutation
    /// 0.005, 2,000 periods.
    fn default() -> Self {
        EthnoConfig {
            width: 50,
            colors: 4,
            start: Start::Empty,
            immigration: 1.0,
            base_ptr: 0.12,
            cost: 0.01,
            benefit: 0.03,
            death: 0.10,
            mutation: 0.005,
            tag_mutation: None,
            pair_play: PairPlay::Once,
            discrimination: Discrimination::SameOther,
            misperception: 0.0,
            offspring: Offspring::Adjacent,
            allowed: Strategy::FOUR.to_vec(),
            kin_strategies: false,
            kin_basis: KinBasis::Mutates,
            kin_mutation: 0.005,
            end: 2000,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 12] = [
    "immigration",
    "base_ptr",
    "cost",
    "benefit",
    "death",
    "mutation",
    "tag_mutation",
    "pair_play",
    "misperception",
    "offspring",
    "kin_mutation",
    "end",
];

impl EthnoConfig {
    /// The tag's mutation rate.
    pub fn tag_rate(&self) -> f64 {
        self.tag_mutation.unwrap_or(self.mutation)
    }

    /// Whether `allowed` holds all four strategies.
    pub fn allows_all(&self) -> bool {
        Strategy::FOUR.iter().all(|s| self.allowed.contains(s))
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
        check(
            (3..=200).contains(&self.width),
            "width",
            "must be between 3 and 200",
        );
        check(
            (1..=40).contains(&self.colors),
            "colors",
            "must be between 1 and 40",
        );
        check(
            nonneg(self.immigration) && self.immigration <= 100.0,
            "immigration",
            "must be between 0 and 100",
        );
        check(nonneg(self.base_ptr), "base_ptr", "must be a number ≥ 0");
        check(nonneg(self.cost), "cost", "must be a number ≥ 0");
        check(nonneg(self.benefit), "benefit", "must be a number ≥ 0");
        check(unit(self.death), "death", "must be between 0 and 1");
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        if let Some(t) = self.tag_mutation {
            check(unit(t), "tag_mutation", "must be between 0 and 1");
        }
        check(
            unit(self.misperception),
            "misperception",
            "must be between 0 and 1",
        );
        check(
            unit(self.kin_mutation),
            "kin_mutation",
            "must be between 0 and 1",
        );
        let each = self.discrimination == Discrimination::EachColor;
        check(
            !(each && self.kin_strategies),
            "kin_strategies",
            "each-colour strategies have no kin variant",
        );
        check(
            !(each && self.misperception > 0.0),
            "misperception",
            "applies only to same/other discrimination",
        );
        check(
            !self.allowed.is_empty(),
            "allowed",
            "must allow at least one strategy",
        );
        check(
            self.allowed.iter().all(|s| Strategy::FOUR.contains(s)),
            "allowed",
            "may list only E, H, S and T",
        );
        check(
            self.allows_all()
                || (self.discrimination == Discrimination::SameOther && !self.kin_strategies),
            "allowed",
            "restricting strategies needs same/other discrimination without kin strategies",
        );
        check(
            self.start != Start::Selfish || self.allowed.contains(&Strategy::Selfish),
            "start",
            "a selfish start needs S among the allowed strategies",
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
                match ModelConfig::Ethno(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Ethno(next)) => {
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
    pub fn changes(&self, next: &EthnoConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !LIVE.contains(&k.as_str()))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields. `allowed` is not on the panel (a list is not a
/// panel kind); presets, files and links set it.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::number("Game", "cost", "Cost of helping", (0.0, 0.1, 0.001), Live)
            .with_help("PTR the helper loses"),
        Param::number(
            "Game",
            "benefit",
            "Benefit of help",
            (0.0, 0.1, 0.001),
            Live,
        )
        .with_help("PTR the helped agent gains"),
        Param::number("Game", "base_ptr", "Base PTR", (0.0, 0.5, 0.01), Live)
            .with_help("Each period's potential to reproduce before any help"),
        Param::choice(
            "Game",
            "pair_play",
            "Decisions per neighbor",
            &[
                ("once", "Once (the code)"),
                ("twice", "Twice (the appendix, literally)"),
            ],
            Live,
        ),
        Param::integer(
            "Population",
            "width",
            "Width (a square torus)",
            (3, 200),
            Reset,
        ),
        Param::choice(
            "Population",
            "start",
            "Start",
            &[
                ("empty", "Empty (HA06)"),
                ("random", "Full, random traits (the archived code)"),
                ("selfish", "Full of egoists"),
            ],
            Reset,
        ),
        Param::number(
            "Population",
            "immigration",
            "Immigrants per period",
            (0.0, 5.0, 0.5),
            Live,
        )
        .with_help("A fraction is the chance of one more"),
        Param::number("Population", "death", "Death rate", (0.0, 0.5, 0.01), Live),
        Param::choice(
            "Population",
            "offspring",
            "Offspring",
            &[
                ("adjacent", "Next to the parent (HA06)"),
                ("anywhere", "Anywhere (Jansson)"),
            ],
            Live,
        ),
        Param::integer("Traits", "colors", "Colours (tags)", (1, 40), Reset),
        Param::choice(
            "Traits",
            "discrimination",
            "Agents see",
            &[
                ("same_other", "Same or other colour"),
                ("none", "Nothing (help all or none)"),
                ("each_color", "Each colour"),
            ],
            Reset,
        ),
        Param::number(
            "Traits",
            "misperception",
            "Misperception",
            (0.0, 0.5, 0.01),
            Live,
        )
        .with_help("The chance a decision mistakes same for other colour")
        .shown_if("discrimination", "same_other"),
        Param::bool(
            "Traits",
            "kin_strategies",
            "Kin strategies (Jansson)",
            Reset,
        )
        .with_help("Agents discriminate on the tag or on a kin marker"),
        Param::choice(
            "Traits",
            "kin_basis",
            "Tag or kin basis",
            &[
                ("mutates", "Mutates like a strategy bit"),
                ("fixed", "Fixed at immigration"),
            ],
            Reset,
        )
        .shown_if("kin_strategies", "true"),
        Param::number(
            "Traits",
            "kin_mutation",
            "Kin marker mutation",
            (0.0, 0.1, 0.001),
            Live,
        )
        .shown_if("kin_strategies", "true"),
        Param::number(
            "Mutation",
            "mutation",
            "Mutation rate",
            (0.0, 0.1, 0.0005),
            Live,
        )
        .with_help("Per trait and offspring"),
        Param::number(
            "Mutation",
            "tag_mutation",
            "Tag mutation rate",
            (0.0, 1.0, 0.005),
            Live,
        )
        .with_help("Empty: the mutation rate"),
        Param::integer("Run", "end", "Last period (0: never)", (0, 100_000), Live),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &EthnoConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_ha06s_text_and_validates() {
        let c = EthnoConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!(
            (c.width, c.colors, c.immigration, c.mutation, c.end),
            (50, 4, 1.0, 0.005, 2000)
        );
        assert_eq!(
            (c.base_ptr, c.cost, c.benefit, c.death),
            (0.12, 0.01, 0.03, 0.1)
        );
        assert_eq!(c.tag_rate(), 0.005);
        assert_eq!(c.kin_basis, KinBasis::Mutates);
        assert_eq!(serde_json::to_value(&c).unwrap()["kin_basis"], "mutates");
        assert!(c.allows_all());
        assert_eq!(
            serde_json::to_value(&c).unwrap()["allowed"],
            json!(["E", "H", "S", "T"])
        );
    }

    #[test]
    fn validation_names_the_field() {
        let bad = |edit: &dyn Fn(&mut EthnoConfig)| {
            let mut c = EthnoConfig::default();
            edit(&mut c);
            fields(&c)
        };
        assert_eq!(bad(&|c| c.width = 2), ["width"]);
        assert_eq!(bad(&|c| c.width = 201), ["width"]);
        assert_eq!(bad(&|c| c.colors = 0), ["colors"]);
        assert_eq!(bad(&|c| c.colors = 41), ["colors"]);
        assert_eq!(bad(&|c| c.cost = -0.01), ["cost"]);
        assert_eq!(bad(&|c| c.benefit = f64::NAN), ["benefit"]);
        assert_eq!(bad(&|c| c.base_ptr = -1.0), ["base_ptr"]);
        assert_eq!(bad(&|c| c.death = 1.5), ["death"]);
        assert_eq!(bad(&|c| c.mutation = -0.1), ["mutation"]);
        assert_eq!(bad(&|c| c.tag_mutation = Some(2.0)), ["tag_mutation"]);
        assert_eq!(bad(&|c| c.kin_mutation = 2.0), ["kin_mutation"]);
        assert_eq!(bad(&|c| c.immigration = -1.0), ["immigration"]);
        assert_eq!(
            bad(&|c| {
                c.discrimination = Discrimination::EachColor;
                c.kin_strategies = true;
            }),
            ["kin_strategies"]
        );
        assert_eq!(
            bad(&|c| {
                c.discrimination = Discrimination::EachColor;
                c.misperception = 0.1;
            }),
            ["misperception"]
        );
        assert_eq!(bad(&|c| c.allowed.clear()), ["allowed"]);
        assert_eq!(bad(&|c| c.allowed = vec![Strategy::Kin]), ["allowed"]);
        assert_eq!(
            bad(&|c| {
                c.allowed = vec![Strategy::Humanitarian];
                c.discrimination = Discrimination::None;
            }),
            ["allowed"]
        );
        assert_eq!(
            bad(&|c| {
                c.allowed = vec![Strategy::Humanitarian];
                c.kin_strategies = true;
            }),
            ["allowed"]
        );
        assert_eq!(
            bad(&|c| {
                c.allowed = vec![Strategy::Humanitarian];
                c.start = Start::Selfish;
            }),
            ["start"]
        );
        assert!(bad(&|c| c.allowed = vec![Strategy::Humanitarian, Strategy::Selfish]).is_empty());
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = EthnoConfig {
            schedule: vec![entry("cost", json!(0.02))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("colors", json!(5))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("death", json!(2))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = EthnoConfig::default();
        let mut b = a.clone();
        b.cost = 0.02;
        b.tag_mutation = Some(0.3);
        b.offspring = Offspring::Anywhere;
        assert!(a.changes(&b).is_empty());
        b.colors = 5;
        b.start = Start::Random;
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["colors", "start"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: EthnoConfig =
            serde_json::from_str(r#"{"start": "random", "allowed": ["H", "S"]}"#).unwrap();
        assert_eq!(
            (c.start, c.allowed.clone(), c.width),
            (
                Start::Random,
                vec![Strategy::Humanitarian, Strategy::Selfish],
                50
            )
        );
        assert!(serde_json::from_str::<EthnoConfig>(r#"{"tags": 3}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        // The panel's tag mutation is a number; null (the default) is
        // "the mutation rate", so the check starts from a set value.
        let config = ModelConfig::Ethno(EthnoConfig {
            width: 10,
            tag_mutation: Some(0.005),
            ..Default::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
