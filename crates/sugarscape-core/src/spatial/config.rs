//! The spatial games' parameters: NM92's lattice game with HG93's
//! asynchronous updating and NBM94's probabilistic winning, irregular
//! arrays and cubes, and a schedule of live changes.

use serde::{Deserialize, Serialize};

use super::geometry::offsets;
use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// Where the players sit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lattice {
    /// Every cell of a width × height grid holds a player.
    #[default]
    Square,
    /// Every cell of an n × n × n cube (n = width).
    Cube,
    /// NBM94's irregular arrays: a fraction of a width × height grid's
    /// cells, chosen at random, hold players.
    Random,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    /// 8 neighbors in a square, 26 in a cube.
    #[default]
    Moore,
    /// 4 neighbors in a square, 6 in a cube.
    VonNeumann,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    /// Players at the edges have fewer neighbors (NM92's figures).
    #[default]
    Fixed,
    Periodic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Discrete time: every site at once (NM92).
    #[default]
    Synchronous,
    /// Continuous time: one random site at a time (HG93, NBM94 Fig. 2).
    Asynchronous,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Winning {
    /// The highest scorer takes the site (m = ∞).
    #[default]
    Deterministic,
    /// NBM94's Eq. 1 with exponent `m`.
    Probabilistic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// A fraction of the players, chosen at random, start as defectors.
    #[default]
    Random,
    /// One defector at the center, every other player a cooperator.
    SingleDefector,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpatialConfig {
    pub lattice: Lattice,
    pub width: u32,
    pub height: u32,
    pub neighborhood: Neighborhood,
    pub boundary: Boundary,
    /// Random arrays: the fraction of cells holding a player.
    pub occupancy: f64,
    /// Random arrays: the interaction radius, in cells.
    pub radius: f64,
    /// The temptation T = b (R = 1, S = 0).
    pub b: f64,
    /// P = ε.
    pub epsilon: f64,
    /// a: the weight of the payoff a player earns against itself.
    pub self_weight: f64,
    pub update: Update,
    pub winning: Winning,
    /// Eq. 1's exponent when winning is probabilistic.
    pub m: f64,
    pub start: Start,
    /// A random start's fraction of defectors.
    pub defectors: f64,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for SpatialConfig {
    /// NM92's Fig. 1b: 200 × 200, fixed edges, eight neighbors and self,
    /// 10% defectors, b = 1.9 (spatial chaos).
    fn default() -> Self {
        SpatialConfig {
            lattice: Lattice::Square,
            width: 200,
            height: 200,
            neighborhood: Neighborhood::Moore,
            boundary: Boundary::Fixed,
            occupancy: 0.05,
            radius: 5.0,
            b: 1.9,
            epsilon: 0.0,
            self_weight: 1.0,
            update: Update::Synchronous,
            winning: Winning::Deterministic,
            m: 1.0,
            start: Start::Random,
            defectors: 0.1,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 6] = ["b", "epsilon", "self_weight", "update", "winning", "m"];

impl SpatialConfig {
    /// The side lengths of the lattice or base grid: (x, y, z).
    pub fn dims(&self) -> (u32, u32, u32) {
        match self.lattice {
            Lattice::Cube => (self.width, self.width, self.width),
            Lattice::Square | Lattice::Random => (self.width, self.height, 1),
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
        // Periodic edges need 3 cells a side, or a neighbor would be the
        // player itself or counted twice.
        let min = if self.boundary == Boundary::Periodic {
            3
        } else {
            1
        };
        match self.lattice {
            Lattice::Cube => check(
                (min..=64).contains(&self.width),
                "width",
                &format!("a cube's side must be between {min} and 64"),
            ),
            Lattice::Square | Lattice::Random => {
                let message = format!("must be between {min} and 1000");
                check((min..=1000).contains(&self.width), "width", &message);
                check((min..=1000).contains(&self.height), "height", &message);
            }
        }
        if self.lattice == Lattice::Random {
            check(
                self.occupancy > 0.0 && self.occupancy <= 1.0,
                "occupancy",
                "must be above 0 and at most 1",
            );
            let side = f64::from(self.width.min(self.height));
            check(
                self.radius >= 1.0 && 2.0 * self.radius.floor() < side,
                "radius",
                "must be at least 1 and less than half the grid's smaller side",
            );
            // The schema's own maximum: kept here too so a raw JSON or CLI
            // config cannot skip it (a huge radius also feeds the budget
            // check below, which clamps it before calling `offsets`).
            check(self.radius <= 20.0, "radius", "must be at most 20");
        }
        // Neighbor storage: a world holds one CSR neighbor list entry per
        // (player, neighbor) pair. Bound players × neighbors-a-player so a
        // config that passes every other check cannot still exhaust memory
        // (e.g. a 1000 × 1000 random array at full occupancy and radius 20
        // needs ~1.26e9 entries). The radius fed to `offsets` here is
        // clamped to the schema's maximum so this estimate itself cannot
        // blow up on a wild out-of-range value.
        let cells = match self.lattice {
            Lattice::Cube => f64::from(self.width).powi(3),
            Lattice::Square | Lattice::Random => f64::from(self.width) * f64::from(self.height),
        };
        let players = if self.lattice == Lattice::Random {
            (self.occupancy * cells).round().max(1.0)
        } else {
            cells
        };
        let neighbors_per_player = if self.lattice == Lattice::Random {
            let probe = SpatialConfig {
                radius: self.radius.clamp(0.0, 20.0),
                ..self.clone()
            };
            offsets(&probe).len() as f64
        } else {
            offsets(self).len() as f64
        };
        check(
            players * neighbors_per_player <= 20_000_000.0,
            if self.lattice == Lattice::Random {
                "radius"
            } else {
                "width"
            },
            if self.lattice == Lattice::Random {
                "too many neighbor pairs: players × neighbors must stay under 20 million — lower the radius or the occupancy"
            } else {
                "the lattice is too large"
            },
        );
        check(
            self.b.is_finite() && self.b > 0.0 && self.b <= 10.0,
            "b",
            "must be above 0 and at most 10",
        );
        check(
            self.epsilon == 0.0 || (1e-6..1.0).contains(&self.epsilon),
            "epsilon",
            "must be 0, or at least 0.000001 and below 1",
        );
        check(
            (0.0..=10.0).contains(&self.self_weight),
            "self_weight",
            "must be between 0 and 10",
        );
        check(
            self.m.is_finite() && (0.0..=1000.0).contains(&self.m),
            "m",
            "must be between 0 and 1000",
        );
        check(
            (0.0..=1.0).contains(&self.defectors),
            "defectors",
            "must be 0–1",
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
                match ModelConfig::Spatial(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Spatial(next)) => {
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
    pub fn changes(&self, next: &SpatialConfig) -> Vec<FieldError> {
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
    vec![
        Param::number("Game", "b", "Temptation (b)", (1.0, 3.0, 0.01), Live)
            .with_help("A defector's payoff against a cooperator (R = 1, S = 0)"),
        Param::number(
            "Game",
            "epsilon",
            "Mutual defection (ε)",
            (0.0, 0.5, 0.01),
            Live,
        )
        .with_help("P = ε: the papers use 0"),
        Param::number(
            "Game",
            "self_weight",
            "Self-interaction (a)",
            (0.0, 2.0, 0.05),
            Live,
        )
        .with_help("The weight of the game a player plays against itself (0: none)"),
        Param::choice(
            "Lattice",
            "lattice",
            "Lattice",
            &[
                ("square", "Square"),
                ("cube", "Cube (n × n × n)"),
                ("random", "Random array"),
            ],
            Reset,
        ),
        Param::integer("Lattice", "width", "Width (a cube's side)", (3, 400), Reset),
        Param::integer("Lattice", "height", "Height", (3, 400), Reset),
        Param::choice(
            "Lattice",
            "neighborhood",
            "Neighbors",
            &[
                ("moore", "Moore (8, or 26 in a cube)"),
                ("von_neumann", "von Neumann (4, or 6)"),
            ],
            Reset,
        ),
        Param::choice(
            "Lattice",
            "boundary",
            "Edges",
            &[("fixed", "Fixed"), ("periodic", "Periodic (wrap)")],
            Reset,
        ),
        Param::number(
            "Lattice",
            "occupancy",
            "Cells occupied",
            (0.01, 1.0, 0.01),
            Reset,
        )
        .shown_if("lattice", "random"),
        Param::number(
            "Lattice",
            "radius",
            "Interaction radius",
            (1.0, 20.0, 0.5),
            Reset,
        )
        .shown_if("lattice", "random"),
        Param::choice(
            "Update",
            "update",
            "Update",
            &[
                ("synchronous", "Synchronous (discrete time)"),
                ("asynchronous", "Asynchronous (continuous time)"),
            ],
            Live,
        )
        .with_help("Asynchronous: one random site at a time, as Huberman and Glance proposed"),
        Param::choice(
            "Update",
            "winning",
            "Winning",
            &[
                ("deterministic", "Highest score wins"),
                ("probabilistic", "Probabilistic (Eq. 1)"),
            ],
            Live,
        ),
        Param::number("Update", "m", "Exponent (m)", (0.0, 100.0, 0.5), Live)
            .with_help("P(C) = Σ A^m (C) / Σ A^m: 0 is random drift, 1 proportional")
            .shown_if("winning", "probabilistic"),
        Param::choice(
            "Start",
            "start",
            "Start",
            &[
                ("random", "Random defectors"),
                ("single_defector", "One defector at the center"),
            ],
            Reset,
        ),
        Param::number("Start", "defectors", "Defectors", (0.0, 1.0, 0.01), Reset)
            .shown_if("start", "random"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &SpatialConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_nm92s_fig_1b_and_validates() {
        let c = SpatialConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!(
            (c.width, c.b, c.defectors, c.self_weight),
            (200, 1.9, 0.1, 1.0)
        );
        assert_eq!(c.dims(), (200, 200, 1));
    }

    #[test]
    fn sides_radius_and_ranges_are_checked() {
        let mut c = SpatialConfig {
            width: 2,
            boundary: Boundary::Periodic,
            ..Default::default()
        };
        assert_eq!(fields(&c), ["width"], "periodic edges need 3 a side");
        c.boundary = Boundary::Fixed;
        assert!(c.validate().is_ok(), "fixed edges allow 1");
        c.lattice = Lattice::Cube;
        c.width = 65;
        assert_eq!(fields(&c), ["width"]);
        c.width = 20;
        assert_eq!(c.dims(), (20, 20, 20));
        c.lattice = Lattice::Random;
        c.radius = 10.0;
        assert_eq!(fields(&c), ["radius"], "2·10 is not below the side 20");
        c.radius = 9.5;
        assert!(c.validate().is_ok());
        for (field, value) in [("b", json!(0)), ("epsilon", json!(1)), ("m", json!(-1))] {
            let bad = ModelConfig::Spatial(SpatialConfig::default())
                .with_path(field, &value)
                .unwrap();
            assert_eq!(bad.validate().unwrap_err()[0].field, field);
        }
    }

    #[test]
    fn random_arrays_are_bounded_by_a_neighbor_pair_budget() {
        let full = SpatialConfig {
            lattice: Lattice::Random,
            width: 1000,
            height: 1000,
            occupancy: 1.0,
            radius: 20.0,
            ..Default::default()
        };
        assert_eq!(
            fields(&full),
            ["radius"],
            "1e6 players × ~1256 neighbors is far past the budget"
        );
        let mut past_the_cap = full.clone();
        past_the_cap.radius = 21.0;
        assert!(fields(&past_the_cap).contains(&"radius".to_string()));
        let default_random = SpatialConfig {
            lattice: Lattice::Random,
            ..Default::default()
        };
        assert!(
            default_random.validate().is_ok(),
            "the default random array (200², 5%, r = 5) stays well under budget"
        );
        let big_square = SpatialConfig {
            width: 1000,
            height: 1000,
            ..Default::default()
        };
        assert!(
            big_square.validate().is_ok(),
            "a 1000 × 1000 Moore square is 8e6 pairs, under budget"
        );
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = SpatialConfig {
            schedule: vec![entry("update", json!("asynchronous"))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("width", json!(50))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("b", json!(20))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = SpatialConfig::default();
        let mut b = a.clone();
        b.b = 1.5;
        b.update = Update::Asynchronous;
        b.winning = Winning::Probabilistic;
        assert!(a.changes(&b).is_empty());
        b.lattice = Lattice::Cube;
        b.defectors = 0.5;
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["defectors", "lattice"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: SpatialConfig =
            serde_json::from_str(r#"{"lattice": "random", "update": "asynchronous"}"#).unwrap();
        assert_eq!(
            (c.lattice, c.update, c.width),
            (Lattice::Random, Update::Asynchronous, 200)
        );
        assert!(serde_json::from_str::<SpatialConfig>(r#"{"cells": 3}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Spatial(SpatialConfig {
            width: 40,
            height: 40,
            ..Default::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
