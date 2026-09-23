//! Every rule parameter and toggle. Rule names follow the book's notation:
//! G_α growback, S_{α,β,γ} seasons, P/D pollution, R_[a,b] replacement,
//! S sex, I inheritance, K culture, C_α combat.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::agent::Sex;

/// Inclusive integer range sampled uniformly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct URange {
    pub min: u32,
    pub max: u32,
}

impl URange {
    pub const fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> u32 {
        rng.gen_range(self.min..=self.max)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LandscapeKind {
    /// The book's 50×50 map with sugar mountains in the northeast and southwest.
    TwoPeaks,
    Flat {
        capacity: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Placement {
    /// Uniformly random distinct sites.
    Random,
    /// Random sites inside a rectangle (Animation II-6's block of agents).
    Block {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Blues in a `size`×`size` southwest block, Reds in the northeast block.
    Tribes { size: u32 },
}

/// G_α: grow back `rate` per tick up to capacity; `instant` is G_∞.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Growback {
    pub rate: f64,
    pub instant: bool,
}

/// S_{α,β,γ}: summer in the north first; flip every `period` (γ) ticks;
/// winter grows at α / `winter_divisor` (β) per tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seasons {
    pub enabled: bool,
    pub winter_divisor: u32,
    pub period: u32,
}

/// P_{α,β}: pollution += α·gathered + β·metabolized, on the agent's site.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pollution {
    pub enabled: bool,
    pub production: f64,
    pub consumption: f64,
    /// Chapter IV makes sugar the only "dirty" good; set to pollute spice too.
    #[serde(default)]
    pub spice_pollutes: bool,
}

/// D_α: every `every` ticks each site's pollution becomes its neighbors' mean.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diffusion {
    pub enabled: bool,
    pub every: u32,
}

/// Death from old age; `max_age` is also R_[a,b]'s [a, b].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lifespan {
    pub enabled: bool,
    pub max_age: URange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Toggle {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SexRule {
    pub enabled: bool,
    pub fertility_onset: URange,
    pub female_end: URange,
    pub male_end: URange,
}

impl SexRule {
    pub fn end_for(&self, sex: Sex) -> URange {
        match sex {
            Sex::Female => self.female_end,
            Sex::Male => self.male_end,
        }
    }
}

/// C_α: reward is min(α, victim's sugar); `unlimited` is C_∞.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatRule {
    pub enabled: bool,
    pub unlimited: bool,
    pub reward: f64,
}

/// Chapter IV's second commodity. When off, agents never draw spice traits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpiceRule {
    pub enabled: bool,
    pub metabolism: URange,
    pub endowment: URange,
}

/// L_{d,r}: sugar loans of `duration` (d) ticks at `rate` (r) percent simple
/// interest per tick.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreditRule {
    pub enabled: bool,
    pub duration: u32,
    pub rate: f64,
}

/// Book eq. 6: agents value holdings as if `φ` periods of metabolism were
/// already spent; `range` is φ's initial distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Foresight {
    pub enabled: bool,
    pub range: URange,
}

/// Fields a schedule may not change (they shape the world's storage or setup).
pub const STRUCTURAL_FIELDS: [&str; 6] = [
    "width",
    "height",
    "tag_length",
    "landscape",
    "population",
    "placement",
];

/// At the start of the tick when `World::tick == tick`, set each dotted config
/// path in `set` to its value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledChange {
    pub tick: u64,
    pub set: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub landscape: LandscapeKind,
    pub population: u32,
    pub placement: Placement,
    pub vision: URange,
    pub metabolism: URange,
    pub endowment: URange,
    pub tag_length: u32,
    pub growback: Growback,
    pub seasons: Seasons,
    pub pollution: Pollution,
    pub diffusion: Diffusion,
    pub lifespan: Lifespan,
    pub replacement: Toggle,
    pub sex: SexRule,
    pub inheritance: Toggle,
    pub culture: Toggle,
    pub combat: CombatRule,
    pub spice: SpiceRule,
    pub trade: Toggle,
    pub credit: CreditRule,
    pub foresight: Foresight,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for Config {
    /// ({G₁}, {M}) on the two-peak map with Chapter II's agent distributions;
    /// every other rule's parameters preset to the book's values but off.
    fn default() -> Self {
        Self {
            width: 50,
            height: 50,
            landscape: LandscapeKind::TwoPeaks,
            population: 400,
            placement: Placement::Random,
            vision: URange::new(1, 6),
            metabolism: URange::new(1, 4),
            endowment: URange::new(5, 25),
            tag_length: 11,
            growback: Growback {
                rate: 1.0,
                instant: false,
            },
            seasons: Seasons {
                enabled: false,
                winter_divisor: 8,
                period: 50,
            },
            pollution: Pollution {
                enabled: false,
                production: 1.0,
                consumption: 1.0,
                spice_pollutes: false,
            },
            diffusion: Diffusion {
                enabled: false,
                every: 1,
            },
            lifespan: Lifespan {
                enabled: false,
                max_age: URange::new(60, 100),
            },
            replacement: Toggle { enabled: false },
            sex: SexRule {
                enabled: false,
                fertility_onset: URange::new(12, 15),
                female_end: URange::new(40, 50),
                male_end: URange::new(50, 60),
            },
            inheritance: Toggle { enabled: false },
            culture: Toggle { enabled: false },
            combat: CombatRule {
                enabled: false,
                unlimited: true,
                reward: 2.0,
            },
            spice: SpiceRule {
                enabled: false,
                metabolism: URange::new(1, 4),
                endowment: URange::new(5, 25),
            },
            trade: Toggle { enabled: false },
            credit: CreditRule {
                enabled: false,
                duration: 10,
                rate: 10.0,
            },
            foresight: Foresight {
                enabled: false,
                range: URange::new(0, 10),
            },
            schedule: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Default)]
struct Errors(Vec<FieldError>);

impl Errors {
    fn check(&mut self, ok: bool, field: &str, message: impl Into<String>) {
        if !ok {
            self.0.push(FieldError::new(field, message));
        }
    }

    fn range(&mut self, r: URange, field: &str) {
        self.check(r.min <= r.max, field, "min must be ≤ max");
    }

    fn non_negative(&mut self, v: f64, field: &str) {
        self.check(v.is_finite() && v >= 0.0, field, "must be a number ≥ 0");
    }

    fn finish(self) -> Result<(), Vec<FieldError>> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.0)
        }
    }
}

impl Config {
    pub fn from_json(json: &str) -> Result<Self, Vec<FieldError>> {
        let config: Config = serde_json::from_str(json)
            .map_err(|e| vec![FieldError::new("config", e.to_string())])?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        self.validate_fields()?;
        self.validate_schedule()
    }

    /// Everything except the schedule (renamed from the old `validate` body).
    fn validate_fields(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Errors::default();
        e.check(
            (5..=500).contains(&self.width),
            "width",
            "must be between 5 and 500",
        );
        e.check(
            (5..=500).contains(&self.height),
            "height",
            "must be between 5 and 500",
        );
        match self.landscape {
            LandscapeKind::TwoPeaks => e.check(
                self.width == 50 && self.height == 50,
                "landscape",
                "the two-peak map is 50×50; set width and height to 50",
            ),
            LandscapeKind::Flat { capacity } => e.non_negative(capacity, "landscape.capacity"),
        }
        let pop = u64::from(self.population);
        match self.placement {
            Placement::Random => e.check(
                pop <= u64::from(self.width) * u64::from(self.height),
                "population",
                "cannot exceed the number of sites",
            ),
            Placement::Block {
                x,
                y,
                width,
                height,
            } => {
                e.check(
                    width > 0
                        && height > 0
                        && x.saturating_add(width) <= self.width
                        && y.saturating_add(height) <= self.height,
                    "placement",
                    "block must fit inside the grid",
                );
                e.check(
                    pop <= u64::from(width) * u64::from(height),
                    "population",
                    "cannot exceed the block's area",
                );
            }
            Placement::Tribes { size } => {
                e.check(
                    size > 0 && size.saturating_mul(2) <= self.width.min(self.height),
                    "placement.size",
                    "the two corner blocks must fit without overlapping",
                );
                e.check(
                    pop <= 2 * u64::from(size) * u64::from(size),
                    "population",
                    "cannot exceed the two blocks' area",
                );
            }
        }
        let max_vision = self.width.min(self.height) / 2;
        e.range(self.vision, "vision");
        e.check(self.vision.min >= 1, "vision.min", "must be ≥ 1");
        e.check(
            self.vision.max <= max_vision,
            "vision.max",
            format!("must be ≤ {max_vision} (half the grid)"),
        );
        e.range(self.metabolism, "metabolism");
        e.range(self.endowment, "endowment");
        e.check(
            (1..=64).contains(&self.tag_length),
            "tag_length",
            "must be between 1 and 64",
        );
        e.check(
            self.growback.rate.is_finite() && self.growback.rate > 0.0,
            "growback.rate",
            "must be a number > 0",
        );
        e.check(
            self.seasons.winter_divisor >= 1,
            "seasons.winter_divisor",
            "must be ≥ 1",
        );
        e.check(self.seasons.period >= 1, "seasons.period", "must be ≥ 1");
        e.non_negative(self.pollution.production, "pollution.production");
        e.non_negative(self.pollution.consumption, "pollution.consumption");
        e.check(self.diffusion.every >= 1, "diffusion.every", "must be ≥ 1");
        e.range(self.lifespan.max_age, "lifespan.max_age");
        e.range(self.sex.fertility_onset, "sex.fertility_onset");
        e.range(self.sex.female_end, "sex.female_end");
        e.range(self.sex.male_end, "sex.male_end");
        e.check(
            !(self.replacement.enabled && self.sex.enabled),
            "replacement.enabled",
            "replacement (R) and sex (S) are mutually exclusive",
        );
        e.check(
            !self.replacement.enabled || self.lifespan.enabled,
            "replacement.enabled",
            "replacement R[a,b] needs lifespan on (it supplies [a,b])",
        );
        e.non_negative(self.combat.reward, "combat.reward");
        e.range(self.spice.metabolism, "spice.metabolism");
        e.range(self.spice.endowment, "spice.endowment");
        e.range(self.foresight.range, "foresight.range");
        e.check(
            !self.trade.enabled || self.spice.enabled,
            "trade.enabled",
            "trade (T) needs spice on",
        );
        e.check(
            !self.foresight.enabled || self.spice.enabled,
            "foresight.enabled",
            "foresight needs spice on",
        );
        e.check(
            !self.credit.enabled || self.sex.enabled,
            "credit.enabled",
            "credit (L) needs sex (S) on",
        );
        e.check(
            !(self.combat.enabled && self.spice.enabled),
            "combat.enabled",
            "combat (C) and spice are mutually exclusive",
        );
        e.check(self.credit.duration >= 1, "credit.duration", "must be ≥ 1");
        e.non_negative(self.credit.rate, "credit.rate");
        e.finish()
    }

    fn validate_schedule(&self) -> Result<(), Vec<FieldError>> {
        let mut entries: Vec<&ScheduledChange> = self.schedule.iter().collect();
        entries.sort_by_key(|c| c.tick);
        let mut patched = self.clone();
        for change in entries {
            if change.tick == 0 {
                return Err(vec![FieldError::new(
                    "schedule",
                    "scheduled ticks start at 1",
                )]);
            }
            patched = patched.apply_change(change).map_err(|e| vec![e])?;
        }
        Ok(())
    }

    /// A copy with one dotted `path` set to `value`.
    pub fn with_path(&self, path: &str, value: &serde_json::Value) -> Result<Config, FieldError> {
        let mut json = serde_json::to_value(self).expect("config serializes");
        let mut slot = &mut json;
        for key in path.split('.') {
            slot = slot
                .get_mut(key)
                .ok_or_else(|| FieldError::new("schedule", format!("unknown field {path}")))?;
        }
        *slot = value.clone();
        serde_json::from_value(json)
            .map_err(|e| FieldError::new("schedule", format!("{path}: {e}")))
    }

    /// This config with every path in `change` set, checked for validity
    /// (excluding the schedule itself).
    pub fn apply_change(&self, change: &ScheduledChange) -> Result<Config, FieldError> {
        let mut next = self.clone();
        for (path, value) in &change.set {
            let root = path.split('.').next().unwrap_or_default();
            if STRUCTURAL_FIELDS.contains(&root) {
                return Err(FieldError::new(
                    "schedule",
                    format!("{path} changes only on reset"),
                ));
            }
            next = next.with_path(path, value)?;
        }
        next.validate_fields().map_err(|errs| {
            let e = &errs[0];
            FieldError::new(
                "schedule",
                format!("at t={}: {}: {}", change.tick, e.field, e.message),
            )
        })?;
        Ok(next)
    }

    /// Fields that cannot change on a running world (they shape its storage).
    pub fn structural_changes(&self, next: &Config) -> Vec<FieldError> {
        let mut out = Vec::new();
        let msg = "changes only on reset";
        if self.width != next.width {
            out.push(FieldError::new("width", msg));
        }
        if self.height != next.height {
            out.push(FieldError::new("height", msg));
        }
        if self.tag_length != next.tag_length {
            out.push(FieldError::new("tag_length", msg));
        }
        if self.landscape != next.landscape {
            out.push(FieldError::new("landscape", msg));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(r: Result<(), Vec<FieldError>>) -> Vec<String> {
        r.err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn default_is_the_books_chapter_two_setup_and_valid() {
        let c = Config::default();
        assert_eq!((c.width, c.height, c.population), (50, 50, 400));
        assert_eq!(c.vision, URange::new(1, 6));
        assert_eq!(c.metabolism, URange::new(1, 4));
        assert_eq!(c.endowment, URange::new(5, 25));
        assert_eq!(c.tag_length, 11);
        assert_eq!(c.growback.rate, 1.0);
        c.validate().unwrap();
    }

    #[test]
    fn rejects_inverted_ranges_with_field_names() {
        let c = Config {
            metabolism: URange::new(4, 1),
            ..Default::default()
        };
        assert_eq!(fields(c.validate()), vec!["metabolism"]);
    }

    #[test]
    fn two_peak_map_requires_50_by_50() {
        let c = Config {
            width: 40,
            ..Default::default()
        };
        assert!(fields(c.validate()).contains(&"landscape".to_string()));
    }

    #[test]
    fn width_height_bounds_are_5_to_500() {
        let c_too_small = Config {
            landscape: LandscapeKind::Flat { capacity: 1.0 },
            width: 4,
            ..Default::default()
        };
        assert!(fields(c_too_small.validate()).contains(&"width".to_string()));
        let c_valid = Config {
            landscape: LandscapeKind::Flat { capacity: 1.0 },
            width: 5,
            height: 5,
            population: 10,
            vision: URange::new(1, 2),
            ..Default::default()
        };
        c_valid.validate().unwrap();
    }

    #[test]
    fn vision_cannot_exceed_half_the_grid() {
        let mut c = Config::default();
        c.vision.max = 26;
        assert!(fields(c.validate()).contains(&"vision.max".to_string()));
    }

    #[test]
    fn replacement_excludes_sex_and_needs_lifespan() {
        let mut c = Config::default();
        c.replacement.enabled = true;
        assert!(fields(c.validate()).contains(&"replacement.enabled".to_string()));
        c.lifespan.enabled = true;
        c.validate().unwrap();
        c.sex.enabled = true;
        assert!(fields(c.validate()).contains(&"replacement.enabled".to_string()));
    }

    #[test]
    fn tribes_placement_must_fit() {
        let c = Config {
            placement: Placement::Tribes { size: 30 },
            ..Default::default()
        };
        assert!(fields(c.validate()).contains(&"placement.size".to_string()));
    }

    #[test]
    fn json_round_trips_and_missing_fields_default() {
        let c = Config::default();
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(Config::from_json(&json).unwrap(), c);
        let partial = Config::from_json(r#"{"population": 100}"#).unwrap();
        assert_eq!(partial.population, 100);
        assert_eq!(partial.width, 50);
        assert!(json.contains(r#""landscape":{"kind":"two_peaks"}"#));
    }

    #[test]
    fn bad_json_is_a_config_field_error() {
        let errs = Config::from_json("{not json").unwrap_err();
        assert_eq!(errs[0].field, "config");
    }

    #[test]
    fn structural_changes_are_reported() {
        let a = Config::default();
        let mut b = a.clone();
        b.sex.enabled = true;
        assert!(a.structural_changes(&b).is_empty());
        b.tag_length = 5;
        assert_eq!(a.structural_changes(&b)[0].field, "tag_length");
    }

    #[test]
    fn chapter_four_defaults_are_off_and_old_json_still_loads() {
        let c = Config::default();
        assert!(!c.spice.enabled && !c.trade.enabled && !c.credit.enabled && !c.foresight.enabled);
        assert_eq!(c.spice.metabolism, URange::new(1, 4));
        assert_eq!(c.spice.endowment, URange::new(5, 25));
        assert_eq!((c.credit.duration, c.credit.rate), (10, 10.0));
        assert_eq!(c.foresight.range, URange::new(0, 10));
        assert!(!c.pollution.spice_pollutes);
        let old = r#"{"pollution":{"enabled":true,"production":1.0,"consumption":1.0}}"#;
        let loaded = Config::from_json(old).unwrap();
        assert!(loaded.pollution.enabled && !loaded.pollution.spice_pollutes);
    }

    #[test]
    fn chapter_four_rule_dependencies_are_validated() {
        let with = |f: fn(&mut Config)| {
            let mut c = Config::default();
            f(&mut c);
            fields(c.validate())
        };
        assert!(with(|c| c.trade.enabled = true).contains(&"trade.enabled".to_string()));
        assert!(with(|c| c.foresight.enabled = true).contains(&"foresight.enabled".to_string()));
        assert!(with(|c| c.credit.enabled = true).contains(&"credit.enabled".to_string()));
        assert!(with(|c| {
            c.spice.enabled = true;
            c.combat.enabled = true;
        })
        .contains(&"combat.enabled".to_string()));
        assert!(with(|c| c.credit.duration = 0).contains(&"credit.duration".to_string()));
        assert!(with(|c| c.credit.rate = -1.0).contains(&"credit.rate".to_string()));
        assert!(with(|c| c.spice.metabolism = URange::new(3, 1))
            .contains(&"spice.metabolism".to_string()));
        assert!(with(|c| {
            c.spice.enabled = true;
            c.trade.enabled = true;
            c.foresight.enabled = true;
        })
        .is_empty());
    }

    fn change(tick: u64, path: &str, value: serde_json::Value) -> ScheduledChange {
        ScheduledChange {
            tick,
            set: [(path.to_string(), value)].into_iter().collect(),
        }
    }

    #[test]
    fn with_path_sets_nested_fields_and_rejects_unknown_ones() {
        let c = Config::default();
        let on = c
            .with_path("pollution.enabled", &serde_json::json!(true))
            .unwrap();
        assert!(on.pollution.enabled);
        assert_eq!(
            c.with_path("pollution.nope", &serde_json::json!(1))
                .unwrap_err()
                .field,
            "schedule"
        );
        assert_eq!(
            c.with_path("pollution.enabled", &serde_json::json!("yes"))
                .unwrap_err()
                .field,
            "schedule"
        );
    }

    #[test]
    fn schedule_entries_are_validated() {
        let mut c = Config {
            schedule: vec![change(50, "pollution.enabled", serde_json::json!(true))],
            ..Default::default()
        };
        c.validate().unwrap();
        c.schedule = vec![change(0, "pollution.enabled", serde_json::json!(true))];
        assert_eq!(fields(c.validate()), vec!["schedule"]);
        c.schedule = vec![change(5, "width", serde_json::json!(60))];
        assert_eq!(fields(c.validate()), vec!["schedule"]);
        c.schedule = vec![change(5, "trade.enabled", serde_json::json!(true))];
        assert_eq!(
            fields(c.validate()),
            vec!["schedule"],
            "trade without spice is invalid"
        );
    }
}
