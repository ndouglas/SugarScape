//! The hoarding model's parameters: Vander Wall and Jenkins's (2003) genetic
//! algorithm as they print it, with every gap the paper leaves open filled by
//! a stated choice (the spec's amendments) and exposed as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How a larder's apparency weighs into the food available to a searcher.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LarderWeight {
    /// Pages 662 and 664 (the default): each non-empty larder, a burrow
    /// entrance, counts `app_lard` once.
    PerBurrow,
    /// One reading of the Appendix: each larder item counts `app_lard`.
    PerItem,
}

/// What becomes of a dead agent's stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadStores {
    /// They stay in the world and stay pilferable.
    Remain,
    /// They are removed at death.
    Remove,
}

/// Whether a defended larder is part of the food a searcher sees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefendedInPool {
    /// It stays in the pool and the draw, and yields nothing.
    Counted,
    /// It is left out of both.
    Excluded,
}

/// What a cheater's fitness is when parents are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheaterFitness {
    /// Leftover stores, as for everyone (the literal default). A cheater
    /// stores nothing, so it is never a parent unless every survivor holds
    /// nothing.
    Stores,
    /// Survival: a surviving cheater weighs the mean leftover stores of the
    /// surviving hoarders.
    Survival,
}

/// The most agents a population can hold: the frame draws one column each.
pub const MAX_AGENTS: u32 = 20;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HoardConfig {
    /// Agents in the population.
    pub n: u32,
    /// Days in the season, and foraging bouts in a day.
    pub days: u32,
    pub bouts: u32,
    /// Public food: day d (1 to `food_days`) gets
    /// floor(`food_first` − `food_step` × (d − 1) + 0.5) items.
    pub food_days: u32,
    pub food_first: f64,
    pub food_step: f64,
    /// Days at the start of the season on which agents are fed by
    /// nonstorable food.
    pub nonstorable_days: u32,
    /// The search's calibration: with this many items available, the chance
    /// of finding none in all `bouts` bouts is `search_miss`.
    pub search_items: u32,
    pub search_miss: f64,
    /// The standard deviation of foraging efficiency (mean 1).
    pub forage_sd: f64,
    /// Relative detectability of others' scattered items and larder items.
    pub app_scat: f64,
    pub app_lard: f64,
    /// Chance an agent is preyed upon in a bout.
    pub predation: f64,
    /// h² for both heritable traits, and the segregation variance on the
    /// logit scale.
    pub heritability: f64,
    pub v_seg: f64,
    /// The founders' centers for the probability of larder hoarding and the
    /// propensity to defend.
    pub l_mean: f64,
    pub d_mean: f64,
    /// Generations to run.
    pub generations: u32,
    /// Share of founders that never cache and eat what they find. Founders
    /// are assigned by id, with no draw (`founder_cheats`).
    pub cheaters: f64,
    /// A cheater's fitness when parents are drawn.
    pub cheater_fitness: CheaterFitness,
    /// Chance an owner finds its scatter items when it goes to eat from them.
    pub owner_recovery: f64,
    /// The defense logistic's slope.
    pub defense_slope: f64,
    pub larder_weight: LarderWeight,
    pub dead_stores: DeadStores,
    pub defended_in_pool: DefendedInPool,
    /// On days 2 to `nonstorable_days`, bout 1 applies the bout-1 rule as
    /// printed.
    pub early_bout1_eats: bool,
}

impl Default for HoardConfig {
    /// Vander Wall and Jenkins's population of 20 over 100 days of 20 bouts,
    /// with the amendments' gap choices.
    fn default() -> Self {
        HoardConfig {
            n: 20,
            days: 100,
            bouts: 20,
            food_days: 50,
            food_first: 82.3,
            food_step: 1.645,
            nonstorable_days: 5,
            search_items: 82,
            search_miss: 0.01,
            forage_sd: 0.1,
            app_scat: 0.44,
            app_lard: 2.0,
            predation: 0.0001,
            heritability: 0.8,
            v_seg: 0.5,
            l_mean: 0.15,
            d_mean: 0.5,
            generations: 60,
            cheaters: 0.0,
            cheater_fitness: CheaterFitness::Stores,
            owner_recovery: 1.0,
            defense_slope: 10.0,
            larder_weight: LarderWeight::PerBurrow,
            dead_stores: DeadStores::Remain,
            defended_in_pool: DefendedInPool::Counted,
            early_bout1_eats: false,
        }
    }
}

impl HoardConfig {
    /// Whether the founder with `id` (ids count from 1, so agent index i has
    /// id i + 1) is a cheater: ⌊i·s⌋ > ⌊(i − 1)·s⌋ for s = `cheaters`. Over
    /// ids 1..=n that's ⌊n·s⌋ cheaters, with no draw (Minds 6's convention).
    pub fn founder_cheats(&self, id: u64) -> bool {
        let s = self.cheaters;
        s > 0.0 && (id as f64 * s).floor() > (id.saturating_sub(1) as f64 * s).floor()
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        // The frame gives each agent a column of `WIDE / n` cells; at most 20
        // keeps every column wide enough to draw and click (V&J run 20).
        check(
            (2..=MAX_AGENTS).contains(&self.n),
            "n",
            "must be between 2 and 20",
        );
        check(
            (1..=1000).contains(&self.days),
            "days",
            "must be between 1 and 1000",
        );
        check(
            (1..=1000).contains(&self.bouts),
            "bouts",
            "must be between 1 and 1000",
        );
        check(
            self.food_days <= self.days,
            "food_days",
            "must be at most the days in the season",
        );
        check(
            self.food_first.is_finite() && (0.0..=10_000.0).contains(&self.food_first),
            "food_first",
            "must be between 0 and 10000",
        );
        check(
            self.food_step.is_finite() && (0.0..=1000.0).contains(&self.food_step),
            "food_step",
            "must be between 0 and 1000",
        );
        check(
            self.food_first - self.food_step * f64::from(self.food_days.saturating_sub(1)) + 0.5
                >= 0.0,
            "food_step",
            "must not take the food below 0 by the last day of production",
        );
        check(
            self.nonstorable_days <= self.days,
            "nonstorable_days",
            "must be at most the days in the season",
        );
        check(
            (1..=1_000_000).contains(&self.search_items),
            "search_items",
            "must be between 1 and 1000000",
        );
        check(
            self.search_miss > 0.0 && self.search_miss < 1.0,
            "search_miss",
            "must be between 0 and 1, exclusive",
        );
        check(
            self.forage_sd.is_finite() && (0.0..=1.0).contains(&self.forage_sd),
            "forage_sd",
            "must be between 0 and 1",
        );
        check(
            self.app_scat.is_finite() && self.app_scat > 0.0 && self.app_scat <= 100.0,
            "app_scat",
            "must be above 0 and at most 100",
        );
        check(
            self.app_lard.is_finite() && self.app_lard > 0.0 && self.app_lard <= 100.0,
            "app_lard",
            "must be above 0 and at most 100",
        );
        check(
            (0.0..=1.0).contains(&self.predation),
            "predation",
            "must be between 0 and 1",
        );
        check(
            (0.0..=1.0).contains(&self.heritability),
            "heritability",
            "must be between 0 and 1",
        );
        check(
            self.v_seg.is_finite() && (0.0..=10.0).contains(&self.v_seg),
            "v_seg",
            "must be between 0 and 10",
        );
        check(
            self.l_mean > 0.0 && self.l_mean < 1.0,
            "l_mean",
            "must be between 0 and 1, exclusive",
        );
        check(
            self.d_mean > 0.0 && self.d_mean < 1.0,
            "d_mean",
            "must be between 0 and 1, exclusive",
        );
        check(
            (1..=10_000).contains(&self.generations),
            "generations",
            "must be between 1 and 10000",
        );
        check(
            (0.0..=1.0).contains(&self.cheaters),
            "cheaters",
            "must be between 0 and 1",
        );
        check(
            (0.0..=1.0).contains(&self.owner_recovery),
            "owner_recovery",
            "must be between 0 and 1",
        );
        check(
            self.defense_slope.is_finite()
                && self.defense_slope > 0.0
                && self.defense_slope <= 1000.0,
            "defense_slope",
            "must be above 0 and at most 1000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &HoardConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("n", self.n == next.n),
            ("days", self.days == next.days),
            ("bouts", self.bouts == next.bouts),
            ("food_days", self.food_days == next.food_days),
            ("food_first", self.food_first == next.food_first),
            ("food_step", self.food_step == next.food_step),
            (
                "nonstorable_days",
                self.nonstorable_days == next.nonstorable_days,
            ),
            ("search_items", self.search_items == next.search_items),
            ("search_miss", self.search_miss == next.search_miss),
            ("forage_sd", self.forage_sd == next.forage_sd),
            ("l_mean", self.l_mean == next.l_mean),
            ("d_mean", self.d_mean == next.d_mean),
            ("cheaters", self.cheaters == next.cheaters),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields, grouped so that every group applies one way:
/// live (to the running world) or on reset (rebuilding it).
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Season", "n", "Agents", (2, 20), Reset),
        Param::integer("Season", "days", "Days in the season", (1, 1000), Reset),
        Param::integer("Season", "bouts", "Foraging bouts a day", (1, 1000), Reset),
        Param::integer("Run", "generations", "Generations", (1, 10_000), Live),
        Param::integer(
            "Food",
            "food_days",
            "Days food is produced",
            (0, 1000),
            Reset,
        ),
        Param::number(
            "Food",
            "food_first",
            "Items on day 1",
            (0.0, 10_000.0, 0.1),
            Reset,
        ),
        Param::number(
            "Food",
            "food_step",
            "Items lost a day",
            (0.0, 1000.0, 0.005),
            Reset,
        ),
        Param::integer(
            "Food",
            "nonstorable_days",
            "Days fed by nonstorable food",
            (0, 1000),
            Reset,
        ),
        Param::integer(
            "Search",
            "search_items",
            "Calibration items",
            (1, 1_000_000),
            Reset,
        ),
        Param::number(
            "Search",
            "search_miss",
            "Chance of finding nothing in a day",
            (0.001, 0.999, 0.001),
            Reset,
        ),
        Param::number(
            "Search",
            "forage_sd",
            "Foraging efficiency spread",
            (0.0, 1.0, 0.01),
            Reset,
        ),
        Param::number(
            "Apparency",
            "app_scat",
            "Apparency of scattered items",
            (0.01, 100.0, 0.01),
            Live,
        ),
        Param::number(
            "Apparency",
            "app_lard",
            "Apparency of larders",
            (0.01, 100.0, 0.01),
            Live,
        ),
        Param::choice(
            "Apparency",
            "larder_weight",
            "A larder weighs",
            &[
                ("per_burrow", "Once per burrow (pages 662 and 664)"),
                ("per_item", "Per item (one reading of the Appendix)"),
            ],
            Live,
        ),
        Param::number(
            "Risk",
            "predation",
            "Predation a bout",
            (0.0, 1.0, 0.0001),
            Live,
        ),
        Param::number(
            "Inheritance",
            "heritability",
            "Heritability",
            (0.0, 1.0, 0.05),
            Live,
        ),
        Param::number(
            "Inheritance",
            "v_seg",
            "Segregation variance",
            (0.0, 10.0, 0.05),
            Live,
        ),
        Param::number(
            "Founders",
            "l_mean",
            "Founders' larder probability",
            (0.01, 0.99, 0.01),
            Reset,
        ),
        Param::number(
            "Founders",
            "d_mean",
            "Founders' defense propensity",
            (0.01, 0.99, 0.01),
            Reset,
        ),
        Param::number(
            "Founders",
            "cheaters",
            "Share of founders that never cache",
            (0.0, 1.0, 0.05),
            Reset,
        ),
        Param::number(
            "Defense",
            "defense_slope",
            "Defense logistic's slope",
            (0.1, 1000.0, 0.5),
            Live,
        ),
        Param::choice(
            "Defense",
            "defended_in_pool",
            "A defended larder is",
            &[
                ("counted", "Still in the food available"),
                ("excluded", "Left out of the food available"),
            ],
            Live,
        ),
        Param::bool(
            "Switches",
            "early_bout1_eats",
            "Eat from stores in bout 1 on days 2 to 5",
            Live,
        ),
        Param::choice(
            "Switches",
            "dead_stores",
            "The dead's stores",
            &[
                ("remain", "Stay and can be pilfered"),
                ("remove", "Are removed"),
            ],
            Live,
        ),
        Param::number(
            "Switches",
            "owner_recovery",
            "Owner finds its scattered items",
            (0.0, 1.0, 0.05),
            Live,
        ),
        Param::choice(
            "Switches",
            "cheater_fitness",
            "A cheater's fitness",
            &[
                ("stores", "Leftover stores, as for everyone"),
                (
                    "survival",
                    "Survival, weighed as the average surviving hoarder",
                ),
            ],
            Live,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_vander_wall_and_jenkins() {
        let c = HoardConfig::default();
        assert_eq!((c.n, c.days, c.bouts, c.generations), (20, 100, 20, 60));
        assert_eq!((c.heritability, c.predation), (0.8, 0.0001));
        assert_eq!((c.app_scat, c.app_lard), (0.44, 2.0));
        assert_eq!((c.l_mean, c.d_mean, c.v_seg), (0.15, 0.5, 0.5));
        assert_eq!((c.owner_recovery, c.cheaters), (1.0, 0.0));
        assert_eq!(c.cheater_fitness, CheaterFitness::Stores);
        assert_eq!(c.defense_slope, 10.0);
        assert_eq!(
            (c.larder_weight, c.dead_stores, c.defended_in_pool),
            (
                LarderWeight::PerBurrow,
                DeadStores::Remain,
                DefendedInPool::Counted
            )
        );
        assert!(!c.early_bout1_eats);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn the_food_schedule_matches_the_printed_values() {
        let c = HoardConfig::default();
        let day = |d: u32| (c.food_first - c.food_step * f64::from(d - 1) + 0.5).floor() as i64;
        assert_eq!((day(1), day(2), day(3), day(50)), (82, 81, 79, 2));
        assert_eq!((1..=c.food_days).map(day).sum::<i64>(), 2100);
    }

    #[test]
    fn bad_values_name_their_config_fields() {
        for (edit, field) in [
            ((|c: &mut HoardConfig| c.n = 1) as fn(&mut HoardConfig), "n"),
            (|c| c.n = 21, "n"),
            (|c| c.bouts = 0, "bouts"),
            (|c| c.food_days = 101, "food_days"),
            (|c| c.search_miss = 1.0, "search_miss"),
            (|c| c.app_lard = 0.0, "app_lard"),
            (|c| c.heritability = 1.5, "heritability"),
            (|c| c.l_mean = 0.0, "l_mean"),
            (|c| c.generations = 0, "generations"),
            (|c| c.cheaters = -0.1, "cheaters"),
            (|c| c.owner_recovery = 2.0, "owner_recovery"),
            (|c| c.defense_slope = 0.0, "defense_slope"),
            (|c| c.food_step = f64::NAN, "food_step"),
        ] {
            let mut c = HoardConfig::default();
            edit(&mut c);
            let e = c.validate().unwrap_err();
            assert!(e.iter().any(|f| f.field == field), "{field}: {e:?}");
        }
    }

    #[test]
    fn older_configs_load_with_the_defaults() {
        let c: HoardConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(c, HoardConfig::default());
        let c: HoardConfig = serde_json::from_str(r#"{"n": 30, "app_lard": 3.0}"#).unwrap();
        assert_eq!((c.n, c.app_lard, c.app_scat), (30, 3.0, 0.44));
        assert_eq!(c.owner_recovery, 1.0);
        let c = ModelConfig::from_json(r#"{"model": "hoard", "cheaters": 0.25}"#).unwrap();
        let h = match &c {
            ModelConfig::Hoard(h) => h,
            _ => panic!("not a hoard config"),
        };
        assert_eq!(
            (h.cheaters, h.generations, h.defense_slope),
            (0.25, 60, 10.0)
        );
        assert!(serde_json::from_str::<HoardConfig>(r#"{"nope": 1}"#).is_err());
        let c: HoardConfig =
            serde_json::from_str(r#"{"larder_weight": "per_item", "dead_stores": "remove"}"#)
                .unwrap();
        assert_eq!(
            (c.larder_weight, c.dead_stores),
            (LarderWeight::PerItem, DeadStores::Remove)
        );
        let c: HoardConfig = serde_json::from_str(r#"{"cheater_fitness": "survival"}"#).unwrap();
        assert_eq!(c.cheater_fitness, CheaterFitness::Survival);
    }

    /// Founders are cheaters by id (1-based), an exact share with no draw.
    #[test]
    fn founders_cheat_by_id() {
        let at = |s: f64, n: u64| {
            let c = HoardConfig {
                cheaters: s,
                ..HoardConfig::default()
            };
            (1..=n).filter(|&i| c.founder_cheats(i)).collect::<Vec<_>>()
        };
        assert_eq!(at(0.25, 20), vec![4, 8, 12, 16, 20]);
        assert_eq!(at(0.5, 6), vec![2, 4, 6]);
        assert!(at(0.0, 20).is_empty());
        assert_eq!(at(1.0, 20).len(), 20);
        for s in [0.1, 0.3, 0.33, 0.7] {
            assert_eq!(at(s, 100).len(), (100.0 * s).floor() as usize, "{s}");
        }
    }

    /// Every group applies one way, so its note (live, or rebuilds the
    /// world) holds for each field in it.
    #[test]
    fn each_schema_group_applies_one_way() {
        let params = schema();
        for p in &params {
            let same = params
                .iter()
                .filter(|q| q.group == p.group)
                .all(|q| q.apply == p.apply);
            assert!(same, "{} mixes live and reset fields", p.group);
        }
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Hoard(HoardConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
