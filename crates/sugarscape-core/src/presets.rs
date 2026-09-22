//! The book's named rule systems. Parameters come from the text of
//! Chapters II–III; `source` cites the animation each reproduces.

use serde::Serialize;

use crate::config::{Config, Placement, URange};

#[derive(Clone, Debug, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    pub description: &'static str,
    pub config: Config,
}

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut Config),
) -> Preset {
    let mut config = Config::default();
    edit(&mut config);
    Preset {
        id,
        name,
        source,
        description,
        config,
    }
}

/// The two-tribe setup used by the combat runs: Blues southwest, Reds northeast.
fn tribes(c: &mut Config) {
    c.placement = Placement::Tribes { size: 20 };
}

/// Chapter III's demographic parameters (sex with finite lifetimes).
fn demography(c: &mut Config) {
    c.sex.enabled = true;
    c.lifespan.enabled = true;
    c.endowment = URange::new(50, 100);
}

pub fn all() -> Vec<Preset> {
    vec![
        preset(
            "ii-1-instant",
            "({G∞}, {M})",
            "Animation II-1",
            "Instant growback: agents climb to the best ridge they can see and settle; the poorly endowed starve.",
            |c| c.growback.instant = true,
        ),
        preset(
            "ii-2-unit",
            "({G₁}, {M})",
            "Animation II-2",
            "Unit growback: continuous hiving on the two sugar mountains; population falls to a carrying capacity near 224.",
            |_| {},
        ),
        preset(
            "ii-5-wealth",
            "({G₁}, {M, R[60,100]})",
            "Animations II-4/II-5",
            "Finite lifetimes with replacement: a skewed wealth distribution emerges; watch the Lorenz curve and Gini coefficient.",
            |c| {
                c.lifespan.enabled = true;
                c.replacement.enabled = true;
            },
        ),
        preset(
            "ii-6-waves",
            "Diagonal waves",
            "Animation II-6",
            "A block of high-vision agents in the southwest propagates northeast in collective waves no individual can move in.",
            |c| {
                c.placement = Placement::Block {
                    x: 0,
                    y: 25,
                    width: 25,
                    height: 25,
                };
                c.vision = URange::new(1, 10);
            },
        ),
        preset(
            "ii-7-seasons",
            "({S₁,₈,₅₀}, {M})",
            "Animation II-7",
            "Seasons flip every 50 ticks: high-vision agents migrate, low-vision low-metabolism agents hibernate.",
            |c| c.seasons.enabled = true,
        ),
        preset(
            "ii-8-pollution",
            "({G₁, D₁}, {M, P₁₁})",
            "Animation II-8",
            "Gathering and eating pollute; diffusion spreads it. (The book switches pollution on at t = 50 and diffusion at t = 100; toggle them yourself to replay that.)",
            |c| {
                c.pollution.enabled = true;
                c.diffusion.enabled = true;
            },
        ),
        preset(
            "iii-2-sex",
            "({G₁}, {M, S})",
            "Animations III-1/III-2",
            "Sexual reproduction with finite lifetimes: a roughly stable population made of many generations.",
            demography,
        ),
        preset(
            "iii-4-inheritance",
            "({G₁}, {M, S, I})",
            "Animation III-4",
            "Inheritance passes wealth to children; compare the Gini coefficient with and without it.",
            |c| {
                demography(c);
                c.inheritance.enabled = true;
            },
        ),
        preset(
            "iii-6-culture",
            "({G₁}, {M, K})",
            "Animations III-6/III-7",
            "Cultural transmission: tag-flipping converts spatially separated groups toward uniform tribes.",
            |c| c.culture.enabled = true,
        ),
        preset(
            "iii-9-combat",
            "({G₁}, {C∞})",
            "Animation III-9",
            "Unlimited combat between two tribes: the winner accumulates its victims' whole wealth.",
            |c| {
                tribes(c);
                c.combat.enabled = true;
            },
        ),
        preset(
            "iii-11-combat-fixed",
            "({G₁}, {C₂, R[60,100]})",
            "Animation III-11",
            "Fixed reward of 2 per kill, population held at 400 by replacement: prolonged battle fronts.",
            |c| {
                tribes(c);
                c.combat.enabled = true;
                c.combat.unlimited = false;
                c.combat.reward = 2.0;
                c.lifespan.enabled = true;
                c.replacement.enabled = true;
            },
        ),
        preset(
            "iii-12-collision",
            "Colliding waves",
            "Animation III-12",
            "Opposed blocks of high-vision Blues and Reds propagate toward the center and interpenetrate (combat off).",
            |c| {
                tribes(c);
                c.vision = URange::new(1, 10);
            },
        ),
        preset(
            "iii-14-combat-culture",
            "({G₁}, {C∞, K})",
            "Animation III-14",
            "Combat with cultural transmission: conquest and conversion together.",
            |c| {
                tribes(c);
                c.combat.enabled = true;
                c.culture.enabled = true;
            },
        ),
    ]
}

pub fn by_id(id: &str) -> Option<Preset> {
    all().into_iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[test]
    fn every_preset_is_valid_and_runs() {
        let presets = all();
        assert_eq!(presets.len(), 13);
        for p in presets {
            p.config
                .validate()
                .unwrap_or_else(|e| panic!("{}: {e:?}", p.id));
            let mut w = World::new(p.config.clone(), 1).unwrap();
            w.run(20);
        }
    }

    #[test]
    fn ids_are_unique_and_findable() {
        let presets = all();
        let mut ids: Vec<&str> = presets.iter().map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), presets.len());
        assert_eq!(by_id("ii-2-unit").unwrap().config, Config::default());
        assert!(by_id("nope").is_none());
    }
}
