//! Configs saved before N goods — share links, exported configs — used a
//! top-level `landscape`, `metabolism`, `endowment` and `spice` and a
//! single-pollutant `pollution` block. `Config::from_value` routes any JSON
//! object without a `goods` key here (Decision 2); the result is the new
//! shape, and export always writes the new shape.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::{
    default_groups, CombatRule, Config, CreditRule, CultureRule, Diffusion, DiseaseRule,
    FieldError, Foresight, Good, Growback, Lifespan, Map, Placement, Pollutant, Pollution,
    ScheduledChange, Seasons, SexRule, Toggle, Transform, URange, SPICE_COLOR, SUGAR_COLOR,
};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum LandscapeKind {
    TwoPeaks,
    Flat { capacity: f64 },
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct LegacyPollution {
    enabled: bool,
    production: f64,
    consumption: f64,
    #[serde(default)]
    spice_pollutes: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct SpiceRule {
    enabled: bool,
    metabolism: URange,
    endowment: URange,
}

/// The pre-N-goods `Config`, field for field, with the same defaults.
#[derive(Deserialize)]
#[serde(default)]
struct LegacyConfig {
    width: u32,
    height: u32,
    landscape: LandscapeKind,
    population: u32,
    placement: Placement,
    vision: URange,
    metabolism: URange,
    endowment: URange,
    tag_length: u32,
    growback: Growback,
    seasons: Seasons,
    pollution: LegacyPollution,
    diffusion: Diffusion,
    lifespan: Lifespan,
    replacement: Toggle,
    sex: SexRule,
    inheritance: Toggle,
    culture: Toggle,
    combat: CombatRule,
    spice: SpiceRule,
    trade: Toggle,
    credit: CreditRule,
    foresight: Foresight,
    disease: DiseaseRule,
    schedule: Vec<ScheduledChange>,
}

impl Default for LegacyConfig {
    fn default() -> Self {
        let c = Config::default();
        Self {
            width: c.width,
            height: c.height,
            landscape: LandscapeKind::TwoPeaks,
            population: c.population,
            placement: c.placement,
            vision: c.vision,
            metabolism: URange::new(1, 4),
            endowment: URange::new(5, 25),
            tag_length: c.tag_length,
            growback: c.growback,
            seasons: c.seasons,
            pollution: LegacyPollution {
                enabled: false,
                production: 1.0,
                consumption: 1.0,
                spice_pollutes: false,
            },
            diffusion: c.diffusion,
            lifespan: c.lifespan,
            replacement: c.replacement,
            sex: c.sex,
            inheritance: c.inheritance,
            culture: Toggle {
                enabled: c.culture.enabled,
            },
            combat: c.combat,
            spice: SpiceRule {
                enabled: false,
                metabolism: URange::new(1, 4),
                endowment: URange::new(5, 25),
            },
            trade: c.trade,
            credit: c.credit,
            foresight: c.foresight,
            disease: c.disease,
            schedule: Vec::new(),
        }
    }
}

/// A pre-N-goods config in the new shape (the spec's "Legacy configs").
pub(crate) fn convert(value: serde_json::Value) -> Result<Config, FieldError> {
    let old: LegacyConfig =
        serde_json::from_value(value).map_err(|e| FieldError::new("config", e.to_string()))?;
    let sugar_map = match old.landscape {
        LandscapeKind::TwoPeaks => Map::TwoPeaks {
            transform: Transform::Identity,
        },
        LandscapeKind::Flat { capacity } => Map::Flat { capacity },
    };
    let mut goods = vec![Good {
        name: "sugar".into(),
        color: SUGAR_COLOR.into(),
        map: sugar_map.clone(),
        metabolism: old.metabolism,
        endowment: old.endowment,
    }];
    if old.spice.enabled {
        let map = match sugar_map {
            Map::TwoPeaks { .. } => Map::TwoPeaks {
                transform: Transform::MirrorX,
            },
            flat => flat,
        };
        goods.push(Good {
            name: "spice".into(),
            color: SPICE_COLOR.into(),
            map,
            metabolism: old.spice.metabolism,
            endowment: old.spice.endowment,
        });
    }
    let n = goods.len();
    let p = old.pollution;
    let dirty = |i: usize| i == 0 || p.spice_pollutes;
    let coefficients = |v: f64| (0..n).map(|i| if dirty(i) { v } else { 0.0 }).collect();
    let pollutant = Pollutant {
        name: "pollution".into(),
        production: coefficients(p.production),
        consumption: coefficients(p.consumption),
        devalues: (0..n).map(dirty).collect(),
    };
    let mut schedule = Vec::new();
    for change in old.schedule {
        let mut set = BTreeMap::new();
        for (path, value) in change.set {
            for new_path in convert_path(&path, n, p.spice_pollutes)? {
                set.insert(new_path, value.clone());
            }
        }
        if !set.is_empty() {
            schedule.push(ScheduledChange {
                tick: change.tick,
                set,
            });
        }
    }
    Ok(Config {
        width: old.width,
        height: old.height,
        population: old.population,
        placement: old.placement,
        vision: old.vision,
        tag_length: old.tag_length,
        goods,
        growback: old.growback,
        seasons: old.seasons,
        pollution: Pollution {
            enabled: p.enabled,
            pollutants: vec![pollutant],
        },
        diffusion: old.diffusion,
        lifespan: old.lifespan,
        replacement: old.replacement,
        sex: old.sex,
        inheritance: old.inheritance,
        culture: CultureRule {
            enabled: old.culture.enabled,
            groups: default_groups(old.tag_length),
        },
        combat: old.combat,
        trade: old.trade,
        credit: old.credit,
        foresight: old.foresight,
        disease: old.disease,
        schedule,
    })
}

fn is_under(path: &str, root: &str) -> bool {
    path == root
        || path
            .strip_prefix(root)
            .is_some_and(|rest| rest.starts_with('.'))
}

/// The new paths a legacy schedule path sets (Decision 3); empty when the
/// entry could have had no effect.
fn convert_path(path: &str, n: usize, spice_pollutes: bool) -> Result<Vec<String>, FieldError> {
    if is_under(path, "metabolism") || is_under(path, "endowment") {
        return Ok(vec![format!("goods.0.{path}")]);
    }
    if let Some(rest) = path.strip_prefix("spice.") {
        if is_under(rest, "metabolism") || is_under(rest, "endowment") {
            return Ok(if n >= 2 {
                vec![format!("goods.1.{rest}")]
            } else {
                Vec::new()
            });
        }
    }
    for field in ["production", "consumption"] {
        if path == format!("pollution.{field}") {
            let polluting = if spice_pollutes { n } else { 1 };
            return Ok((0..polluting)
                .map(|i| format!("pollution.pollutants.0.{field}.{i}"))
                .collect());
        }
    }
    if path == "pollution" || path == "pollution.spice_pollutes" {
        return Err(FieldError::new(
            "schedule",
            format!("{path} can no longer be scheduled; schedule pollution.enabled or pollution.pollutants.0.… instead"),
        ));
    }
    Ok(vec![path.to_string()])
}
