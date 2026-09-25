//! The anasazi model's parameters (§2) and the replication's departures
//! from the written description as named switches (§7).

use serde::{Deserialize, Serialize};

use crate::anasazi::valley::{FIRST_YEAR, LAST_YEAR};
use crate::config::FieldError;
use crate::model::ModelConfig;
use crate::presets::ModelPreset;
use crate::schema::{Apply, Param};

/// A range of kilograms, `[min, max]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CornRange {
    pub min: f64,
    pub max: f64,
}

/// The published replication's departures from the ODD and JASSS text
/// (§7). Each is named for what it does; all on reproduces the replication
/// (the "published" presets), all off is the documented model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Quirks {
    /// A-2: households age at the start of the year, before the death
    /// check, rather than at its end (ODD step 9).
    pub age_before_death_check: bool,
    /// A-3: farm plots need not lie within a mile of water.
    pub no_farm_water_check: bool,
    /// A-6: Arable Uplands keep a PDSI of 0 (always the (−1, 1) class)
    /// instead of following the uplands series.
    pub uplands_single_class: bool,
    /// A-9: distances wrap around the map's edges (a torus).
    pub wrap_edges: bool,
    /// A-11: each initial household fills every storage slot with an
    /// initial-corn draw, rather than holding one draw in total.
    pub initial_corn_per_slot: bool,
    /// A-12: a new household's corn is drawn fresh (fcs / (1 − fcs) × an
    /// initial-corn draw per slot) rather than taken from its parent, so
    /// corn is not conserved.
    pub fission_fresh_endowment: bool,
    /// A-12: fission is tried only while at least one farm plot is free.
    pub fission_needs_free_farm: bool,
    /// A-13: the initial households choose farms by `y · q`, without the
    /// harvest adjustment.
    pub initial_eligibility_ignores_adjustment: bool,
    /// A-19: a household that moves never frees its old residence, and its
    /// new one counts it twice; only a removal frees one count, so a cell
    /// once lived on stays closed to farming.
    pub occupancy_leak: bool,
    /// A-1: one harvest s.d. (the annual one) also sets soil quality; the
    /// spatial s.d. is ignored.
    pub single_harvest_variance: bool,
}

impl Quirks {
    /// Every departure on: the published replication.
    pub const ALL: Quirks = Quirks {
        age_before_death_check: true,
        no_farm_water_check: true,
        uplands_single_class: true,
        wrap_edges: true,
        initial_corn_per_slot: true,
        fission_fresh_endowment: true,
        fission_needs_free_farm: true,
        initial_eligibility_ignores_adjustment: true,
        occupancy_leak: true,
        single_harvest_variance: true,
    };
    /// None: the documented model.
    pub const NONE: Quirks = Quirks {
        age_before_death_check: false,
        no_farm_water_check: false,
        uplands_single_class: false,
        wrap_edges: false,
        initial_corn_per_slot: false,
        fission_fresh_endowment: false,
        fission_needs_free_farm: false,
        initial_eligibility_ignores_adjustment: false,
        occupancy_leak: false,
        single_harvest_variance: false,
    };
    /// The switches' names, in field order (also their config paths under
    /// `quirks.`).
    pub const NAMES: [&'static str; 10] = [
        "age_before_death_check",
        "no_farm_water_check",
        "uplands_single_class",
        "wrap_edges",
        "initial_corn_per_slot",
        "fission_fresh_endowment",
        "fission_needs_free_farm",
        "initial_eligibility_ignores_adjustment",
        "occupancy_leak",
        "single_harvest_variance",
    ];
}

impl Default for Quirks {
    /// The published replication (every departure on).
    fn default() -> Self {
        Quirks::ALL
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AnasaziConfig {
    /// Tick 0 is this year; each tick is one year.
    pub start_year: u32,
    /// The last year simulated: the world is finished once it gets there.
    pub end_year: u32,
    pub initial_households: u32,
    /// Each initial household's corn (kg), uniform in the range.
    pub initial_corn: CornRange,
    /// Food a household needs a year (kg): 5 persons × 160 kg.
    pub need: f64,
    /// Fission is possible for households older than this…
    pub fertility_start: u32,
    /// …and at most this old.
    pub fertility_end: u32,
    /// Households older than this are removed.
    pub death_age: u32,
    /// Each year's chance that an eligible household splits (pf).
    pub fission_probability: f64,
    /// The share of its parent's corn a new household gets (fcs).
    pub child_endowment: f64,
    /// Years corn keeps beyond the year it is harvested.
    pub storage_years: u32,
    /// Ha: the share of the zone yield a plot gives.
    pub harvest_adjustment: f64,
    /// σshv: the s.d. of soil quality around 1.
    pub spatial_sd: f64,
    /// σahv: the s.d. of each year's harvest around the base yield.
    pub annual_sd: f64,
    pub quirks: Quirks,
}

impl Default for AnasaziConfig {
    /// `lhv-published`: JASSS Table 4's calibrated values with every
    /// replication departure on.
    fn default() -> Self {
        AnasaziConfig::calibrated(Quirks::ALL)
    }
}

impl AnasaziConfig {
    /// The ODD's defaults (Table 2; the "Dean et al. 2000" set, §2.1).
    pub fn table_2(quirks: Quirks) -> Self {
        AnasaziConfig {
            start_year: 800,
            end_year: 1350,
            initial_households: 14,
            initial_corn: CornRange {
                min: 2000.0,
                max: 2400.0,
            },
            need: 800.0,
            fertility_start: 16,
            fertility_end: 30,
            death_age: 30,
            fission_probability: 0.125,
            child_endowment: 0.33,
            storage_years: 2,
            harvest_adjustment: 1.0,
            spatial_sd: 0.1,
            annual_sd: 0.1,
            quirks,
        }
    }

    /// JASSS Table 4's best fit for the replicated model (population L1 and
    /// L2): death age 38, end of fertility 34, fission 0.155, harvest
    /// adjustment 0.56 (not the NetLogo slider's 0.54, A-15), harvest s.d.
    /// 0.4 (spatial and annual).
    pub fn calibrated(quirks: Quirks) -> Self {
        AnasaziConfig {
            death_age: 38,
            fertility_end: 34,
            fission_probability: 0.155,
            harvest_adjustment: 0.56,
            spatial_sd: 0.4,
            annual_sd: 0.4,
            ..AnasaziConfig::table_2(quirks)
        }
    }

    /// The s.d. soil quality is drawn with (A-1).
    pub fn soil_sd(&self) -> f64 {
        if self.quirks.single_harvest_variance {
            self.annual_sd
        } else {
            self.spatial_sd
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: String| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let years = FIRST_YEAR..=LAST_YEAR;
        check(
            years.contains(&self.start_year),
            "start_year",
            format!("must be between {FIRST_YEAR} and {LAST_YEAR} (the data's years)"),
        );
        check(
            years.contains(&self.end_year) && self.end_year > self.start_year,
            "end_year",
            format!("must be after the start year and at most {LAST_YEAR}"),
        );
        check(
            self.initial_households <= 1000,
            "initial_households",
            "must be at most 1000".into(),
        );
        let corn = self.initial_corn;
        check(
            corn.min.is_finite() && corn.max.is_finite() && 0.0 <= corn.min && corn.min <= corn.max,
            "initial_corn",
            "must be a range of kilograms with 0 ≤ min ≤ max".into(),
        );
        check(
            self.need.is_finite() && self.need > 0.0,
            "need",
            "must be a number > 0".into(),
        );
        check(
            self.fertility_start <= self.fertility_end,
            "fertility_end",
            "must be at least the fertility start".into(),
        );
        check(
            (1..=200).contains(&self.death_age),
            "death_age",
            "must be between 1 and 200".into(),
        );
        check(
            (0.0..=1.0).contains(&self.fission_probability),
            "fission_probability",
            "must be between 0 and 1".into(),
        );
        check(
            (0.0..1.0).contains(&self.child_endowment),
            "child_endowment",
            "must be at least 0 and below 1".into(),
        );
        check(
            self.storage_years <= 10,
            "storage_years",
            "must be at most 10".into(),
        );
        for (field, v) in [
            ("harvest_adjustment", self.harvest_adjustment),
            ("spatial_sd", self.spatial_sd),
            ("annual_sd", self.annual_sd),
        ] {
            check(
                v.is_finite() && (0.0..=10.0).contains(&v),
                field,
                "must be between 0 and 10".into(),
            );
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// The reset-only fields that differ in `next` (the harvest adjustment
    /// and the annual s.d. apply to the running world).
    pub fn changes(&self, next: &AnasaziConfig) -> Vec<FieldError> {
        let live = AnasaziConfig {
            harvest_adjustment: self.harvest_adjustment,
            annual_sd: self.annual_sd,
            ..next.clone()
        };
        if &live == self {
            return Vec::new();
        }
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(&live).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k])
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields: the harvest applies to the running world;
/// households, the valley's years and soil, and the quirks rebuild it.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    let mut out = vec![
        Param::integer(
            "Households",
            "initial_households",
            "Households at the start",
            (0, 200),
            Reset,
        ),
        Param::range(
            "Households",
            "initial_corn",
            "Their corn (kg)",
            (0.0, 10_000.0, 100.0),
            Reset,
        ),
        Param::number(
            "Households",
            "need",
            "Food need (kg a year)",
            (100.0, 2000.0, 10.0),
            Reset,
        ),
        Param::integer(
            "Households",
            "fertility_start",
            "Fission when older than (years)",
            (0, 60),
            Reset,
        ),
        Param::integer(
            "Households",
            "fertility_end",
            "Fission until age (years)",
            (0, 60),
            Reset,
        ),
        Param::integer(
            "Households",
            "death_age",
            "Death age (years)",
            (1, 100),
            Reset,
        ),
        Param::number(
            "Households",
            "fission_probability",
            "Fission probability (a year)",
            (0.0, 1.0, 0.005),
            Reset,
        ),
        Param::number(
            "Households",
            "child_endowment",
            "Child’s share of the corn",
            (0.0, 0.95, 0.01),
            Reset,
        ),
        Param::integer(
            "Households",
            "storage_years",
            "Corn keeps (years)",
            (0, 10),
            Reset,
        ),
        Param::number(
            "Harvest",
            "harvest_adjustment",
            "Harvest adjustment",
            (0.0, 2.0, 0.01),
            Live,
        ),
        Param::number(
            "Harvest",
            "annual_sd",
            "Harvest s.d. (each year)",
            (0.0, 1.0, 0.01),
            Live,
        ),
        Param::integer(
            "Valley",
            "start_year",
            "Start year (AD)",
            (FIRST_YEAR, LAST_YEAR - 1),
            Reset,
        ),
        Param::integer(
            "Valley",
            "end_year",
            "End year (AD)",
            (FIRST_YEAR + 1, LAST_YEAR),
            Reset,
        ),
        Param::number(
            "Valley",
            "spatial_sd",
            "Soil quality s.d.",
            (0.0, 1.0, 0.01),
            Reset,
        ),
    ];
    for (path, label, help) in QUIRKS {
        out.push(Param::bool("Replication quirks", path, label, Reset).with_help(help));
    }
    out
}

/// Each quirk's path, label and one-line explanation (citing the
/// extraction), in `Quirks::NAMES` order.
const QUIRKS: [(&str, &str, &str); 10] = [
    (
        "quirks.age_before_death_check",
        "Age before the death check",
        "Households age at the start of the year, before the death check, not at its end (A-2).",
    ),
    (
        "quirks.no_farm_water_check",
        "No water check for farms",
        "Farm plots need not lie within a mile of water, which the ODD requires (A-3).",
    ),
    (
        "quirks.uplands_single_class",
        "Arable Uplands in one PDSI class",
        "Arable Uplands always yield their (−1, 1) class instead of following the uplands PDSI (A-6).",
    ),
    (
        "quirks.wrap_edges",
        "Wrap the map’s edges",
        "Distances wrap around the map’s edges, as on a torus (A-9).",
    ),
    (
        "quirks.initial_corn_per_slot",
        "Initial corn in every slot",
        "The first households hold an initial-corn draw in each storage slot, not one in total (A-11).",
    ),
    (
        "quirks.fission_fresh_endowment",
        "Fresh corn for new households",
        "A new household’s corn is drawn fresh instead of taken from its parent, so corn is not conserved (A-12).",
    ),
    (
        "quirks.fission_needs_free_farm",
        "Fission needs a free farm",
        "Fission is tried only while some farm plot is free (A-12).",
    ),
    (
        "quirks.initial_eligibility_ignores_adjustment",
        "First farms ignore the harvest adjustment",
        "The first households choose farms by yield before the harvest adjustment (A-13).",
    ),
    (
        "quirks.occupancy_leak",
        "Vacated homes stay occupied",
        "A household that moves never frees its old home, so a cell once lived on is never farmed (A-19).",
    ),
    (
        "quirks.single_harvest_variance",
        "One harvest s.d.",
        "The yearly harvest s.d. also sets soil quality; the soil quality s.d. is ignored (A-1).",
    ),
];

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    config: AnasaziConfig,
) -> ModelPreset {
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Anasazi(config),
    }
}

/// The published replication (calibrated and with the ODD's defaults) and
/// the documented model.
pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "lhv-published",
            "Long House Valley: the published replication",
            "Janssen (2009), Table 4 and Figure 10",
            "Janssen's 2009 NetLogo replication of Artificial Anasazi (itself an approximation of Axtell et al. 2002) with JASSS Table 4's best fit — death age 38, fission until 34 with probability 0.155, harvest adjustment 0.56 (Table 4's, not the NetLogo slider's 0.54) and harvest s.d. 0.4 — and every replication quirk on. Households climb to the valley's carrying capacity (about 190 plots), dip with it in the 1140s–1160s, climb again and fall after 1270, but, as in all of Janssen's runs, do not vanish in 1300. Measured over seeds 1–15: 172 households in 1050–1130 (the record: 156), 94 in 1140–1170 (133), 180 in 1180–1265 (172), 59 in 1300 and 22 in 1350 (0); mean fit (the sum of squared differences from the record) 922 516.",
            AnasaziConfig::calibrated(Quirks::ALL),
        ),
        preset(
            "lhv-published-defaults",
            "Long House Valley: the replication with the original defaults",
            "Janssen (2009), Figure 2",
            "The replication with the ODD's defaults (harvest adjustment 1, harvest s.d. 0.1, death and fertility end 30, fission 0.125) and every quirk on: with the full harvest about 1 050 plots can feed a household, and households fill them — five times the archaeological record, as in JASSS Figure 2. Measured over seeds 1–15: 1 046 households in 1050–1130, 864 in 1140–1170, 1 045 in 1180–1265, 482 in 1300 and 385 in 1350.",
            AnasaziConfig::table_2(Quirks::ALL),
        ),
        preset(
            "lhv-documented",
            "Long House Valley: the documented model",
            "Janssen's ODD (2013) with Table 4's values",
            "The model as the ODD and JASSS describe it — every replication quirk off, with Table 4's calibrated values and the rules the text leaves undefined taken from the replication — does not reproduce the published curve: a new household gets a third of its parent's corn rather than a fresh store, so few new households survive. Measured over seeds 1–15: 41 households in 1050–1130, 80 in 1180–1265, 7 of 15 runs with none left by 1350; mean fit 4 296 013, 4.7 times the replication's. Turning the fresh endowment back on alone restores the fit (see the lhv-quirks experiment).",
            AnasaziConfig::calibrated(Quirks::NONE),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_values_are_the_papers() {
        let t2 = AnasaziConfig::table_2(Quirks::ALL);
        assert_eq!(
            (t2.need, t2.death_age, t2.fertility_start, t2.fertility_end),
            (800.0, 30, 16, 30)
        );
        assert_eq!(
            (
                t2.fission_probability,
                t2.child_endowment,
                t2.harvest_adjustment
            ),
            (0.125, 0.33, 1.0)
        );
        assert_eq!(
            (t2.spatial_sd, t2.annual_sd, t2.storage_years),
            (0.1, 0.1, 2)
        );
        assert_eq!(
            (t2.start_year, t2.end_year, t2.initial_households),
            (800, 1350, 14)
        );
        let cal = AnasaziConfig::default();
        assert_eq!((cal.death_age, cal.fertility_end), (38, 34));
        assert_eq!(
            (
                cal.fission_probability,
                cal.harvest_adjustment,
                cal.annual_sd
            ),
            (0.155, 0.56, 0.4)
        );
        assert_eq!(cal.quirks, Quirks::ALL);
        assert_eq!(AnasaziConfig::calibrated(Quirks::NONE).quirks, Quirks::NONE);
    }

    #[test]
    fn the_presets_are_the_published_and_documented_models() {
        let ps = presets();
        let ids: Vec<&str> = ps.iter().map(|p| p.id).collect();
        assert_eq!(
            ids,
            ["lhv-published", "lhv-published-defaults", "lhv-documented"]
        );
        let config = |k: usize| match &ps[k].config {
            ModelConfig::Anasazi(c) => c.clone(),
            _ => unreachable!(),
        };
        assert_eq!(config(0), AnasaziConfig::default());
        assert_eq!(config(1), AnasaziConfig::table_2(Quirks::ALL));
        assert_eq!(
            config(2),
            AnasaziConfig {
                quirks: Quirks::NONE,
                ..config(0)
            }
        );
        for p in &ps {
            // Each description records what it was measured to do (Decision 21).
            assert!(p.description.contains("seeds 1–15"), "{}", p.id);
        }
    }

    #[test]
    fn quirks_default_to_on_and_are_named_in_order() {
        let q: Quirks = serde_json::from_str(r#"{"wrap_edges": false}"#).unwrap();
        assert!(!q.wrap_edges && q.occupancy_leak);
        let json = serde_json::to_value(Quirks::NONE).unwrap();
        let keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();
        let mut names = Quirks::NAMES.to_vec();
        names.sort_unstable();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, names);
        assert!(serde_json::from_str::<Quirks>(r#"{"nope": true}"#).is_err());
    }

    #[test]
    fn the_soil_sd_is_the_spatial_one_unless_one_sd_is_used() {
        let mut c = AnasaziConfig::table_2(Quirks::NONE);
        c.spatial_sd = 0.2;
        c.annual_sd = 0.3;
        assert_eq!(c.soil_sd(), 0.2);
        c.quirks.single_harvest_variance = true;
        assert_eq!(c.soil_sd(), 0.3);
    }

    #[test]
    fn validation_names_fields() {
        let bad = AnasaziConfig {
            start_year: 300,
            end_year: 1600,
            initial_households: 5000,
            initial_corn: CornRange {
                min: 10.0,
                max: 5.0,
            },
            need: 0.0,
            fertility_start: 40,
            fertility_end: 30,
            death_age: 0,
            fission_probability: 1.5,
            child_endowment: 1.0,
            storage_years: 11,
            harvest_adjustment: -1.0,
            spatial_sd: f64::NAN,
            annual_sd: 11.0,
            quirks: Quirks::NONE,
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
                "start_year",
                "end_year",
                "initial_households",
                "initial_corn",
                "need",
                "fertility_end",
                "death_age",
                "fission_probability",
                "child_endowment",
                "storage_years",
                "harvest_adjustment",
                "spatial_sd",
                "annual_sd"
            ]
        );
        assert!(AnasaziConfig::default().validate().is_ok());
        let e = AnasaziConfig {
            end_year: 800,
            ..AnasaziConfig::default()
        };
        assert_eq!(e.validate().unwrap_err()[0].field, "end_year");
    }

    #[test]
    fn only_the_harvest_changes_live() {
        let c = AnasaziConfig::default();
        let live = AnasaziConfig {
            harvest_adjustment: 0.6,
            annual_sd: 0.2,
            ..c.clone()
        };
        assert!(c.changes(&live).is_empty());
        let mut reset = live.clone();
        reset.death_age = 30;
        reset.quirks.wrap_edges = false;
        let fields: Vec<String> = c.changes(&reset).into_iter().map(|e| e.field).collect();
        assert_eq!(fields, ["death_age", "quirks"]);
    }

    #[test]
    fn every_quirk_has_a_path_label_and_help() {
        let quirks: Vec<Param> = schema()
            .into_iter()
            .filter(|p| p.group == "Replication quirks")
            .collect();
        let paths: Vec<&str> = quirks.iter().map(|p| p.path).collect();
        let expected: Vec<String> = Quirks::NAMES
            .iter()
            .map(|n| format!("quirks.{n}"))
            .collect();
        assert_eq!(paths, expected);
        for p in &quirks {
            let help = p.help.unwrap();
            assert!(
                help.ends_with(").") && help.contains("(A-"),
                "{}: {help}",
                p.path
            );
        }
    }
}
