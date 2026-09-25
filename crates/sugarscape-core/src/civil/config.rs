//! The civil violence model's parameters (Epstein 2002, Table 2), NetLogo
//! *Rebellion*'s departures as named switches, and the model's own schedule:
//! steps (`schedule`) and linear ramps (`ramps`) of its live fields.

use serde::{Deserialize, Serialize};

use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// Model I (rebellion against a central authority) or Model II (violence
/// between two ethnic groups).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    #[default]
    Rebellion,
    Ethnic,
}

/// Agent and cop vision: Euclidean radii in cells.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Vision {
    pub agent: f64,
    pub cop: f64,
}

impl Default for Vision {
    fn default() -> Self {
        Vision {
            agent: 7.0,
            cop: 7.0,
        }
    }
}

/// The maximum jail term J_max, in ticks, or terms that never end.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Jail {
    pub max: u32,
    pub infinite: bool,
}

impl Default for Jail {
    fn default() -> Self {
        Jail {
            max: 30,
            infinite: false,
        }
    }
}

/// NetLogo *Rebellion*'s departures from the paper (all off: the paper).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Quirks {
    /// P = 1 − exp(−k·⌊C/A⌋): the ratio rounded down.
    pub floor_ratio: bool,
    /// A counts an already-active agent twice (NetLogo's `1 + count …
    /// with [active?]` includes the agent itself).
    pub active_counts_twice: bool,
    /// The arresting cop steps onto the arrested agent's site.
    pub cop_moves_to_arrest: bool,
    /// Jailed agents keep their site, which counts as empty; a released
    /// agent is free where it stands, perhaps sharing the site.
    pub jailed_stay: bool,
    /// Jail terms are uniform whole numbers in 0..J_max − 1.
    pub netlogo_jail_term: bool,
}

impl Quirks {
    pub const ALL: Quirks = Quirks {
        floor_ratio: true,
        active_counts_twice: true,
        cop_moves_to_arrest: true,
        jailed_stay: true,
        netlogo_jail_term: true,
    };
}

/// A numeric live field moving linearly from its value when tick `start`
/// begins to `to` when tick `end` begins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ramp {
    pub path: String,
    pub start: u64,
    pub end: u64,
    pub to: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CivilConfig {
    pub variant: Variant,
    pub width: u32,
    pub height: u32,
    pub agent_density: f64,
    pub cop_density: f64,
    pub legitimacy: f64,
    pub threshold: f64,
    pub k: f64,
    pub vision: Vision,
    pub movement: bool,
    pub jail: Jail,
    pub clone_probability: f64,
    pub max_age: u32,
    pub stop_at_extinction: bool,
    pub outburst_threshold: u32,
    pub quirks: Quirks,
    pub schedule: Vec<ScheduledChange>,
    pub ramps: Vec<Ramp>,
}

impl Default for CivilConfig {
    /// Table 2's Run 2 on the "All models" line: a 40 × 40 torus, density
    /// 0.7, cops 0.04, vision 7, L = 0.82, J_max = 30, k = 2.3, T = 0.1.
    fn default() -> Self {
        CivilConfig {
            variant: Variant::Rebellion,
            width: 40,
            height: 40,
            agent_density: 0.7,
            cop_density: 0.04,
            legitimacy: 0.82,
            threshold: 0.1,
            k: 2.3,
            vision: Vision::default(),
            movement: true,
            jail: Jail::default(),
            clone_probability: 0.05,
            max_age: 200,
            stop_at_extinction: false,
            outburst_threshold: 50,
            quirks: Quirks::default(),
            schedule: Vec::new(),
            ramps: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 12] = [
    "cop_density",
    "legitimacy",
    "threshold",
    "k",
    "vision",
    "movement",
    "jail",
    "clone_probability",
    "max_age",
    "stop_at_extinction",
    "outburst_threshold",
    "quirks",
];

/// The numeric fields a ramp may move.
pub const RAMPABLE: [&str; 7] = [
    "legitimacy",
    "cop_density",
    "threshold",
    "k",
    "vision.agent",
    "vision.cop",
    "clone_probability",
];

/// Whether a dotted path names a live field or lies inside one.
pub fn is_live(path: &str) -> bool {
    LIVE.contains(&path.split('.').next().unwrap_or(path))
}

impl CivilConfig {
    pub fn sites(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    /// round(density × sites).
    pub fn count(&self, density: f64) -> u32 {
        (density * self.sites() as f64).round() as u32
    }

    pub fn agents(&self) -> u32 {
        self.count(self.agent_density)
    }

    pub fn cops(&self) -> u32 {
        self.count(self.cop_density)
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = self.validate_fields();
        e.extend(self.validate_changes());
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// The checks on the fields themselves (not the schedule or ramps).
    fn validate_fields(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (5..=400).contains(&self.width),
            "width",
            "must be between 5 and 400",
        );
        check(
            (5..=400).contains(&self.height),
            "height",
            "must be between 5 and 400",
        );
        check(unit(self.agent_density), "agent_density", "must be 0–1");
        check(unit(self.cop_density), "cop_density", "must be 0–1");
        check(
            u64::from(self.agents()) + u64::from(self.cops()) <= self.sites(),
            "cop_density",
            "agents and cops must fit on the lattice (densities sum to at most 1)",
        );
        check(unit(self.legitimacy), "legitimacy", "must be 0–1");
        check(
            (-1.0..=1.0).contains(&self.threshold),
            "threshold",
            "must be between −1 and 1",
        );
        check(
            self.k.is_finite() && self.k > 0.0 && self.k <= 100.0,
            "k",
            "must be above 0 and at most 100",
        );
        let side = f64::from(self.width.min(self.height));
        for (field, v) in [
            ("vision.agent", self.vision.agent),
            ("vision.cop", self.vision.cop),
        ] {
            check(
                v.is_finite() && v >= 1.0 && 2.0 * v.floor() < side,
                field,
                "must be at least 1 and less than half the lattice's smaller side",
            );
        }
        check(self.jail.max >= 1, "jail.max", "must be ≥ 1");
        check(
            unit(self.clone_probability),
            "clone_probability",
            "must be 0–1",
        );
        check(self.max_age >= 1, "max_age", "must be ≥ 1");
        check(
            self.outburst_threshold >= 1,
            "outburst_threshold",
            "must be ≥ 1",
        );
        e
    }

    /// Schedule entries and ramps: live paths only, values that validate,
    /// ramps on numeric fields with `start < end`, and no two ramps of one
    /// path overlapping.
    fn validate_changes(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        for change in &self.schedule {
            for (path, value) in &change.set {
                if !is_live(path) {
                    e.push(FieldError::new(
                        "schedule",
                        format!("{path} changes only on reset and cannot be scheduled"),
                    ));
                    continue;
                }
                match ModelConfig::Civil(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Civil(next)) => {
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
        for (i, r) in self.ramps.iter().enumerate() {
            if !RAMPABLE.contains(&r.path.as_str()) {
                e.push(FieldError::new(
                    "ramps",
                    format!("{} cannot be ramped (ramp a live number field)", r.path),
                ));
                continue;
            }
            if r.start >= r.end {
                e.push(FieldError::new(
                    "ramps",
                    format!("{}: start must be before end", r.path),
                ));
            }
            let mut probe = self.clone();
            probe.set_number(&r.path, r.to);
            for f in probe.validate_fields() {
                e.push(FieldError::new(
                    "ramps",
                    format!("{} to {}: {}", r.path, r.to, f.message),
                ));
            }
            for other in &self.ramps[i + 1..] {
                if other.path == r.path && other.start < r.end && r.start < other.end {
                    e.push(FieldError::new(
                        "ramps",
                        format!("two ramps of {} overlap", r.path),
                    ));
                }
            }
        }
        e
    }

    /// The value of a rampable field.
    pub fn number(&self, path: &str) -> f64 {
        match path {
            "legitimacy" => self.legitimacy,
            "cop_density" => self.cop_density,
            "threshold" => self.threshold,
            "k" => self.k,
            "vision.agent" => self.vision.agent,
            "vision.cop" => self.vision.cop,
            "clone_probability" => self.clone_probability,
            _ => unreachable!("{path} is not rampable"),
        }
    }

    /// Sets a rampable field.
    pub fn set_number(&mut self, path: &str, v: f64) {
        let slot = match path {
            "legitimacy" => &mut self.legitimacy,
            "cop_density" => &mut self.cop_density,
            "threshold" => &mut self.threshold,
            "k" => &mut self.k,
            "vision.agent" => &mut self.vision.agent,
            "vision.cop" => &mut self.vision.cop,
            "clone_probability" => &mut self.clone_probability,
            _ => unreachable!("{path} is not rampable"),
        };
        *slot = v;
    }

    /// The reset-only fields that differ from `next`.
    pub fn changes(&self, next: &CivilConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !is_live(k))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::choice(
            "World",
            "variant",
            "Model",
            &[
                ("rebellion", "I — rebellion against a central authority"),
                ("ethnic", "II — violence between two groups"),
            ],
            Reset,
        ),
        Param::integer("World", "width", "Width", (5, 200), Reset),
        Param::integer("World", "height", "Height", (5, 200), Reset),
        Param::number(
            "World",
            "legitimacy",
            "Legitimacy (L)",
            (0.0, 1.0, 0.01),
            Live,
        )
        .with_help("G = H(1 − L): the regime's legitimacy, or in Model II each group's view of the other's right to exist"),
        Param::number(
            "Agents",
            "agent_density",
            "Agent density",
            (0.0, 1.0, 0.01),
            Reset,
        ),
        Param::number(
            "Agents",
            "vision.agent",
            "Agent vision (cells)",
            (1.0, 20.0, 0.1),
            Live,
        )
        .with_help("A Euclidean radius: 1.7 sees the eight neighbors"),
        Param::number(
            "Agents",
            "threshold",
            "Threshold (T)",
            (-1.0, 1.0, 0.01),
            Live,
        )
        .with_help("Active when G − R·P > T"),
        Param::number("Agents", "k", "Arrest constant (k)", (0.1, 10.0, 0.1), Live)
            .with_help("P = 1 − exp(−k·C/A); 2.3 gives P ≈ 0.9 for one cop and one active"),
        Param::bool("Agents", "movement", "Agents move", Live),
        Param::number(
            "Cops",
            "cop_density",
            "Cop density",
            (0.0, 0.3, 0.001),
            Live,
        )
        .with_help("Cops are added on random empty sites or removed at random"),
        Param::number(
            "Cops",
            "vision.cop",
            "Cop vision (cells)",
            (1.0, 20.0, 0.1),
            Live,
        ),
        Param::integer("Jail", "jail.max", "Longest term (ticks)", (1, 1000), Live),
        Param::bool("Jail", "jail.infinite", "Terms never end", Live),
        Param::number(
            "Population",
            "clone_probability",
            "Cloning probability",
            (0.0, 1.0, 0.01),
            Live,
        )
        .with_help("Each tick a free agent clones onto an empty neighboring site")
        .shown_if("variant", "ethnic"),
        Param::integer("Population", "max_age", "Longest life (ticks)", (1, 1000), Live)
            .with_help("Each agent's death age is uniform in 1…this")
            .shown_if("variant", "ethnic"),
        Param::bool(
            "Population",
            "stop_at_extinction",
            "Stop when a group dies out",
            Live,
        )
        .shown_if("variant", "ethnic"),
        Param::integer(
            "Outbursts",
            "outburst_threshold",
            "Outburst above (actives)",
            (1, 1000),
            Live,
        )
        .with_help("An outburst starts above this many actives and ends below it (the paper's 50)"),
        Param::bool(
            "NetLogo quirks",
            "quirks.floor_ratio",
            "Round C/A down",
            Live,
        )
        .with_help("NetLogo's floor(C/A): no risk once actives outnumber the cops in view. The paper's rule gives Model I no outbursts without it"),
        Param::bool(
            "NetLogo quirks",
            "quirks.active_counts_twice",
            "Count an active agent twice",
            Live,
        )
        .with_help("NetLogo's A = 1 + actives in view includes the agent itself once it is active"),
        Param::bool(
            "NetLogo quirks",
            "quirks.cop_moves_to_arrest",
            "Cop moves to the arrest",
            Live,
        )
        .with_help("The arresting cop steps onto the arrested agent's site"),
        Param::bool(
            "NetLogo quirks",
            "quirks.jailed_stay",
            "Jailed agents stay put",
            Live,
        )
        .with_help("A jailed agent's site counts as empty; released agents may share a site"),
        Param::bool(
            "NetLogo quirks",
            "quirks.netlogo_jail_term",
            "Terms 0 to J − 1",
            Live,
        )
        .with_help("NetLogo's random J_max: a term may be 0 ticks"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn change(tick: u64, path: &str, value: serde_json::Value) -> ScheduledChange {
        ScheduledChange {
            tick,
            set: [(path.to_string(), value)].into_iter().collect(),
        }
    }

    fn ramp(path: &str, start: u64, end: u64, to: f64) -> Ramp {
        Ramp {
            path: path.into(),
            start,
            end,
            to,
        }
    }

    fn fields(c: &CivilConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_run_2_and_validates() {
        let c = CivilConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!((c.agents(), c.cops()), (1120, 64));
        assert_eq!((c.legitimacy, c.jail.max, c.vision.agent), (0.82, 30, 7.0));
        assert_eq!(c.quirks, Quirks::default(), "the paper's rules");
    }

    #[test]
    fn agents_and_cops_must_fit_and_vision_must_stay_under_half_the_side() {
        let mut c = CivilConfig {
            agent_density: 0.97,
            ..Default::default()
        };
        assert_eq!(fields(&c), ["cop_density"]);
        c.agent_density = 0.7;
        c.vision.agent = 20.0;
        assert_eq!(fields(&c), ["vision.agent"]);
        c.vision.agent = 19.9;
        assert!(c.validate().is_ok(), "2·⌊19.9⌋ = 38 < 40");
        c.vision.cop = 0.5;
        assert_eq!(fields(&c), ["vision.cop"]);
    }

    #[test]
    fn schedules_take_only_live_fields_with_valid_values() {
        let mut c = CivilConfig {
            schedule: vec![change(10, "legitimacy", json!(0.5))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![change(10, "width", json!(50))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![change(10, "legitimacy", json!(1.5))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![change(10, "quirks.floor_ratio", json!(true))];
        assert!(c.validate().is_ok(), "leaves of a live object are live");
    }

    #[test]
    fn ramps_take_live_numbers_forward_in_time_without_overlapping() {
        let mut c = CivilConfig {
            ramps: vec![ramp("legitimacy", 77, 147, 0.2)],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.ramps = vec![ramp("movement", 1, 2, 1.0)];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![ramp("legitimacy", 5, 5, 0.2)];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![ramp("legitimacy", 0, 10, 2.0)];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![
            ramp("legitimacy", 0, 10, 0.5),
            ramp("legitimacy", 5, 20, 0.1),
        ];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![
            ramp("legitimacy", 0, 10, 0.5),
            ramp("legitimacy", 10, 20, 0.1),
        ];
        assert!(c.validate().is_ok(), "one may start where another ends");
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = CivilConfig::default();
        let mut b = a.clone();
        b.legitimacy = 0.5;
        b.cop_density = 0.1;
        b.quirks.jailed_stay = true;
        b.vision.cop = 3.0;
        assert!(a.changes(&b).is_empty());
        b.width = 50;
        b.variant = Variant::Ethnic;
        b.ramps = vec![ramp("k", 1, 2, 3.0)];
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["ramps", "variant", "width"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: CivilConfig =
            serde_json::from_str(r#"{"variant": "ethnic", "quirks": {"floor_ratio": true}}"#)
                .unwrap();
        assert_eq!(c.variant, Variant::Ethnic);
        assert!(c.quirks.floor_ratio && !c.quirks.jailed_stay);
        assert_eq!(c.width, 40);
        assert!(serde_json::from_str::<CivilConfig>(r#"{"cops": 3}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Civil(CivilConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
