//! The paper's runs and the docking paper's variants.

use super::config::{Activation, Changes, CultureConfig, Edges, Neighborhood};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut CultureConfig),
) -> ModelPreset {
    let mut c = CultureConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Culture(c),
    }
}

/// Five features of fifteen traits on a `side` × `side` lattice: the
/// paper's setting for the most regions (Fig. 2).
fn many(c: &mut CultureConfig, side: u32) {
    c.traits = 15;
    c.width = side;
    c.height = side;
}

const PAPER: &str = "Axelrod 1997, J. Conflict Resolution 41";
const AAEC: &str = "Axtell, Axelrod, Epstein & Cohen 1996";

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ac-sample-run",
            "Sample run: 10 × 10, 5 features of 10 traits",
            PAPER,
            "Axelrod's sample run: 100 sites on a 10 × 10 lattice (edges bounded, four neighbors), each with 5 features of 10 traits drawn at random. An event picks a random site and one of its neighbors; with probability equal to the share of features they already have in common, the site copies one feature on which they differ. The run stops when every two neighbors are identical or share nothing. The paper: 3.2 stable regions on average (10 runs), and over 100 runs a median of 3, 14% with one region and 10% with more than six. Measured (1000 seeds, recorded 2026-09-25): a mean of 4.3, a median of 4, 10% with one and 18% with more than six — a little more diverse than the paper reports.",
            |_| {},
        ),
        preset(
            "ac-many-regions",
            "Many regions: 12 × 12, 15 traits",
            PAPER,
            "Five features of 15 traits on 12 × 12 — the setting of the paper's Fig. 2, near its peak of about 23 stable regions. Measured (20 seeds): 21.3 on average. Few features and many traits leave neighbors with nothing in common, so many regions freeze apart.",
            |c| many(c, 12),
        ),
        preset(
            "ac-large-territory",
            "Large territory: 100 × 100",
            PAPER,
            "Five features of 15 traits on 100 × 100, the paper's largest territory: about 2 stable regions after 'a little more than 100,000 events' per site (a billion events). Measured (8 seeds): 2.25 regions. A long run even at Max speed; the paper's Fig. 3 story — zones settle long before regions — plays out in the Regions, zones and cultures chart.",
            |c| many(c, 100),
        ),
        preset(
            "ac-torus",
            "Many regions on a torus",
            PAPER,
            "The many-regions setup on a torus. The paper: with the edges wrapped 'the peak occurs earlier … and is not as high'. Measured (20 seeds): the torus peaks at about 10 regions on 8 × 8, against about 23 bounded at 10–12 a side.",
            |c| {
                many(c, 12);
                c.boundary = Edges::Torus;
            },
        ),
        preset(
            "ac-random-activation-20",
            "20 × 20, a random site each event",
            AAEC,
            "Five features of 15 traits on 20 × 20 with Axelrod's activation: each event a random site, with replacement. Axtell et al. 1996 found Axelrod's 16.25 regions here, where the Sugarscape's shuffled sweeps gave 9.23. Measured (20 seeds): a median of 16.5 regions. Open 'Literal vs Sugarscape activation' in the presets menu to run both.",
            |c| many(c, 20),
        ),
        preset(
            "ac-sweep-activation",
            "20 × 20, shuffled sweeps (Sugarscape)",
            AAEC,
            "The same 20 × 20 lattice activated as the Sugarscape does it: each tick one shuffled pass over every site. Axtell et al. 1996: 9.23 regions, against Axelrod's 16.25 — the discrepancy their docking traced to this one detail. Measured (20 seeds): a median of 11, fewer than with random activation (one-sided p = 0.01).",
            |c| {
                many(c, 20);
                c.activation = Activation::Sweep;
            },
        ),
        preset(
            "ac-neighbor-changes",
            "Sample run, the neighbor changes",
            AAEC,
            "The sample run with the original Sugarscape's transmission, which Axtell et al. 1996 caught two months into the docking: the chosen neighbor copies the active site, instead of the other way round. Measured (20 seeds): 4.9 regions against 5.15 — at this size it makes no measurable difference.",
            |c| {
                c.changes = Changes::Neighbor;
            },
        ),
        preset(
            "ac-soup",
            "Soup: random pairing, 30 traits",
            AAEC,
            "Axtell et al. 1996's 'soup': any two sites can meet, as if every agent were everyone's neighbor; 5 features of 30 traits. They found 1.4 cultures (15 traits: never more than one). Measured (20 seeds): 1.65 at 30 traits, one culture in 19 of 20 runs at 15. Mixing erases the diversity that locality preserves.",
            |c| {
                c.traits = 30;
                c.neighborhood = Neighborhood::Soup;
            },
        ),
        preset(
            "ac-drift",
            "Many regions with cultural drift",
            "Klemm, Eguíluz, Toral & San Miguel 2003",
            "The many-regions setup with cultural drift (Klemm et al. 2003): on 1 event in 10 000 a random trait of a random feature changes at random. The lattice never becomes stable; the drift keeps reopening frozen borders, so one culture spreads. Measured (20 seeds, 20 000 ticks): a median of 5 cultures, against 17 without drift. Axelrod's polarization needs exactly zero noise.",
            |c| {
                many(c, 12);
                c.drift = 1e-4;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_the_papers_setups_plus_one_named_change() {
        let d = CultureConfig::default();
        let many = |side| CultureConfig {
            width: side,
            height: side,
            traits: 15,
            ..d.clone()
        };
        let expected = [
            ("ac-sample-run", d.clone()),
            ("ac-many-regions", many(12)),
            ("ac-large-territory", many(100)),
            (
                "ac-torus",
                CultureConfig {
                    boundary: Edges::Torus,
                    ..many(12)
                },
            ),
            ("ac-random-activation-20", many(20)),
            (
                "ac-sweep-activation",
                CultureConfig {
                    activation: Activation::Sweep,
                    ..many(20)
                },
            ),
            (
                "ac-neighbor-changes",
                CultureConfig {
                    changes: Changes::Neighbor,
                    ..d.clone()
                },
            ),
            (
                "ac-soup",
                CultureConfig {
                    traits: 30,
                    neighborhood: Neighborhood::Soup,
                    ..d.clone()
                },
            ),
            (
                "ac-drift",
                CultureConfig {
                    drift: 1e-4,
                    ..many(12)
                },
            ),
        ];
        let got: Vec<(&str, CultureConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Culture(c) => (p.id, c),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(got.len(), expected.len());
        for ((id, c), (eid, e)) in got.iter().zip(&expected) {
            assert_eq!((id, c), (eid, e));
            assert!(c.validate().is_ok(), "{id}");
        }
    }
}
