//! Axelrod's culture model's parameters: the paper's (1997) and the
//! docking paper's and later literature's departures, each defaulting to
//! the paper's literal reading.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Whom a site can interact with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    /// North, east, south and west (the paper's four).
    VonNeumann,
    /// The four plus the diagonals (the paper's eight).
    Moore,
    /// The eight plus the four sites two away in the cardinal directions
    /// (the paper's twelve, "a diamond-shaped neighborhood").
    Diamond,
    /// Every other site (AAEC's "soup": random pairing).
    Soup,
}

/// The lattice's edges.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edges {
    /// Edge sites have fewer neighbors (the paper).
    Bounded,
    /// Wrapping north–south and east–west (the paper's aside).
    Torus,
}

/// How events pick their site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activation {
    /// Each event picks a site uniformly, with replacement (the paper).
    Random,
    /// Each tick is one shuffled pass over every site (AAEC's Sugarscape).
    Sweep,
}

/// Which site of an interacting pair changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Changes {
    /// The active site copies its neighbor (the paper).
    Active,
    /// The neighbor copies the active site (the original Sugarscape, per AAEC).
    Neighbor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CultureConfig {
    pub width: u32,
    pub height: u32,
    /// F: cultural features per site.
    pub features: u32,
    /// q: traits per feature.
    pub traits: u32,
    pub neighborhood: Neighborhood,
    pub boundary: Edges,
    pub activation: Activation,
    pub changes: Changes,
    /// Per event, the probability that the active site instead gets a random
    /// trait on a random feature (cultural drift; Klemm et al. 2003).
    pub drift: f64,
    /// Stop once no neighboring pair can interact (never with drift).
    pub stop_when_stable: bool,
}

impl Default for CultureConfig {
    /// The paper's sample run: 10 × 10 bounded, four neighbors, five features
    /// of ten traits, one random event at a time, the active site changing.
    fn default() -> Self {
        CultureConfig {
            width: 10,
            height: 10,
            features: 5,
            traits: 10,
            neighborhood: Neighborhood::VonNeumann,
            boundary: Edges::Bounded,
            activation: Activation::Random,
            changes: Changes::Active,
            drift: 0.0,
            stop_when_stable: true,
        }
    }
}

impl CultureConfig {
    pub fn sites(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (1..=200).contains(&self.width),
            "width",
            "must be between 1 and 200",
        );
        check(
            (1..=200).contains(&self.height),
            "height",
            "must be between 1 and 200",
        );
        check(self.sites() >= 2, "width", "there must be at least 2 sites");
        check(
            (1..=32).contains(&self.features),
            "features",
            "must be between 1 and 32",
        );
        check(
            (2..=255).contains(&self.traits),
            "traits",
            "must be between 2 and 255",
        );
        check(
            (0.0..=1.0).contains(&self.drift),
            "drift",
            "must be between 0 and 1",
        );
        // On a torus a side must be long enough that no offset reaches a
        // site twice or wraps onto the site itself.
        let reach = match self.neighborhood {
            Neighborhood::Diamond => 5,
            _ => 3,
        };
        check(
            self.boundary == Edges::Bounded
                || self.neighborhood == Neighborhood::Soup
                || self.width.min(self.height) >= reach,
            "boundary",
            if reach == 5 {
                "a torus with twelve neighbors needs sides of at least 5"
            } else {
                "a torus needs sides of at least 3"
            },
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &CultureConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("width", self.width == next.width),
            ("height", self.height == next.height),
            ("features", self.features == next.features),
            ("traits", self.traits == next.traits),
            ("neighborhood", self.neighborhood == next.neighborhood),
            ("boundary", self.boundary == next.boundary),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields: the lattice and culture rebuild the world; the
/// departures apply as it runs.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("World", "width", "Width", (1, 200), Reset),
        Param::integer("World", "height", "Height", (1, 200), Reset),
        Param::choice(
            "World",
            "boundary",
            "Edges",
            &[("bounded", "Bounded (paper)"), ("torus", "Torus")],
            Reset,
        ),
        Param::integer("Culture", "features", "Features (F)", (1, 32), Reset),
        Param::integer("Culture", "traits", "Traits per feature (q)", (2, 255), Reset),
        Param::choice(
            "Interaction",
            "neighborhood",
            "Neighbors",
            &[
                ("von_neumann", "4 (paper)"),
                ("moore", "8"),
                ("diamond", "12"),
                ("soup", "Anyone (soup)"),
            ],
            Reset,
        )
        .with_help("The paper's 4, 8 and 12; Axtell et al. 1996's random pairing."),
        Param::bool("Interaction", "stop_when_stable", "Stop when stable", Live),
        Param::choice(
            "Departures",
            "activation",
            "Activation",
            &[
                ("random", "A random site each event (paper)"),
                ("sweep", "Shuffled sweeps (Sugarscape)"),
            ],
            Live,
        )
        .with_help("Axtell et al. 1996: this difference alone took 20 × 20 from 16.25 regions to 9.23."),
        Param::choice(
            "Departures",
            "changes",
            "Who changes",
            &[
                ("active", "The active site (paper)"),
                ("neighbor", "The neighbor (original Sugarscape)"),
            ],
            Live,
        )
        .with_help("Axtell et al. 1996 found the Sugarscape changed the neighbor, two months into the docking."),
        Param::number("Departures", "drift", "Cultural drift", (0.0, 0.01, 0.0001), Live)
            .with_help("Per event, a random trait on a random feature (Klemm et al. 2003): any drift ends in one culture."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_the_papers_sample_run() {
        let c = CultureConfig::default();
        assert_eq!((c.width, c.height, c.features, c.traits), (10, 10, 5, 10));
        assert_eq!(c.neighborhood, Neighborhood::VonNeumann);
        assert_eq!(
            (c.boundary, c.activation, c.changes),
            (Edges::Bounded, Activation::Random, Changes::Active)
        );
        assert_eq!((c.drift, c.stop_when_stable), (0.0, true));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = CultureConfig {
            width: 0,
            height: 201,
            features: 0,
            traits: 256,
            drift: 2.0,
            ..CultureConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["width", "height", "width", "features", "traits", "drift"]
        );
        let small_torus = CultureConfig {
            width: 4,
            height: 4,
            boundary: Edges::Torus,
            neighborhood: Neighborhood::Diamond,
            ..CultureConfig::default()
        };
        assert_eq!(small_torus.validate().unwrap_err()[0].field, "boundary");
        let ok = CultureConfig {
            neighborhood: Neighborhood::Moore,
            ..small_torus
        };
        assert!(ok.validate().is_ok());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Culture(CultureConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
