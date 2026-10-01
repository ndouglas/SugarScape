//! Vander Wall and Jenkins's (2003) genetic algorithm at its threshold and on
//! either side of it, and the two switches that are new ground: owners who
//! must search for their own scattered caches, and founders who never cache.

use super::config::HoardConfig;
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

/// Every preset's source names the paper and the campaign, so the page lists
/// them under Minds 7.
const VJ: &str = "Vander Wall & Jenkins 2003; Minds 7";

fn preset(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut HoardConfig),
) -> ModelPreset {
    let mut c = HoardConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source: VJ,
        description,
        config: ModelConfig::Hoard(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "hoard-threshold",
            "The threshold: app_scat 0.44, app_lard 2",
            "Vander Wall and Jenkins's genetic algorithm: 20 agents through a 100-day season of 20 foraging bouts a day, bred for 60 generations. Public food falls off over the first 50 days; on the first 5 days agents are fed by food that can't be stored, and from then on an agent that eats nothing all day starves. Each find beyond what an agent eats is stored, in its larder (a burrow it can defend) with its heritable probability L, otherwise scattered in caches; its heritable propensity D sets how often it stays home to guard the larder. Searching agents find others' stores in proportion to their apparency: each scattered item counts app_scat 0.44 and each non-empty larder app_lard 2, a ratio of 0.22, the 50 % point of the paper's Fig. 2B fit. Parents are drawn in proportion to the survivors' leftover stores; L and D are inherited on the logit scale with heritability 0.8. Founders start near L 0.15. Measured (seeds 1–50, 60 generations each, the survey): larders take over (the hoarders' mean L above 0.95 over generations 51–60) in 21 of 50 runs, and L stays below 0.2 in the other 29. Takeovers rise late, a median of 20 generations in, not within 10 as the paper reports. Over generations 1–10 a larder loses a median 125 % of its items a day against 29 % for scattered caches (losses ÷ items held, so a larder emptied and refilled within a day loses more than 100 %), and larder loss is above scatter loss in all 50 runs. Where larders take over, about half the agents die each season (mostly by starving; the cause is not isolated), against about one in five where they don't.",
            |_| {},
        ),
        preset(
            "hoard-scatter",
            "Scattered caches hard to find: ratio 0.1",
            "hoard-threshold's world with scattered caches harder to find: each scattered item counts app_scat 0.2 against a larder's app_lard 2, a ratio of 0.1, below the paper's Fig. 2B threshold. Everything else is Vander Wall and Jenkins's defaults. Measured (seeds 1–50, 60 generations each): larders never take over; L stays below 0.2 in 47 runs and ends between 0.2 and 0.95 in 3. Over generations 1–10 scattered caches lose a median 18 % of their items a day and larders 180 %, close to the paper's 18 % (without takeover) and 186 %, though the paper doesn't define its rate. About four agents in five survive each season.",
            |c| c.app_scat = 0.2,
        ),
        preset(
            "hoard-larder",
            "Scattered caches easy to find: ratio 0.4",
            "hoard-threshold's world with scattered caches easier to find: each scattered item counts app_scat 0.8 against a larder's app_lard 2, a ratio of 0.4, above the paper's Fig. 2B threshold. Everything else is Vander Wall and Jenkins's defaults. Measured (seeds 1–50, 60 generations each): larders take over in 49 of 50 runs, rising above 0.95 a median 13 generations in (mean L 0.90 by generation 10). Larders lose little more than scattered caches over generations 1–10 (a median 47 % of their items a day against 42 %), and no more in 12 of the 50 runs, where the paper has larders losing more in every run. About half the agents die each season, most of them starving, against one in five in hoard-scatter; the cause is not isolated.",
            |c| c.app_scat = 0.8,
        ),
        preset(
            "hoard-no-free-recovery",
            "Owners find their own caches half the time",
            "hoard-threshold's world (ratio 0.22) where an owner no longer recovers its scattered caches for free: each time it tries to eat from them in the day's first bout, with no larder item at home, it finds one with chance 0.5. On a miss it keeps its caches, goes hungry, forages and eats its first find, and starves at the day's end if it finds nothing. The larder is at home and stays free. New ground: Vander Wall and Jenkins give owners perfect recall. Measured (seeds 1–50, 60 generations each): larders take over in 28 of 50 runs, against 21 of 50 with free recovery, a difference within what 50 runs can tell apart. At a ratio of 0.1 they never take over at any recovery from 0.25 to 1, and survival falls as owners miss more (82 % of agents a season with free recovery, 74 % at 0.25), likely because a missed meal on a day with no public food left is a starved agent.",
            |c| c.owner_recovery = 0.5,
        ),
        preset(
            "hoard-cheaters",
            "A quarter of the founders never cache",
            "hoard-threshold's world (ratio 0.22) where a quarter of the founders, dealt by id (ids 4, 8, 12, 16 and 20), are cheaters: a cheater never stores, forages each day until its first find and eats it, and then idles, pilfering only what it can eat. A child is a cheater if its mother is. Fitness is leftover stores, as for everyone, and a cheater's are always 0. New ground: Vander Wall and Jenkins describe the cheater but don't run it. Measured (seeds 1–50, 60 generations each): in the first generation cheaters survive about as well as hoarders (a median 80 % against 87 %), but with no stores they are never parents, so none is born in generation 2 in any run. The hoarders then take over in 22 of 50 runs, as without cheaters. If a surviving cheater instead weighs as much as an average surviving hoarder's stores (the survival switch), cheaters persist at about 6 % of births by generation 60, and larders take over in 33 of 50 runs.",
            |c| c.cheaters = 0.25,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_presets_set_what_they_name_and_nothing_else() {
        let p = presets();
        let config = |id: &str| match &p.iter().find(|p| p.id == id).unwrap().config {
            ModelConfig::Hoard(c) => c.clone(),
            _ => unreachable!(),
        };
        let d = HoardConfig::default();
        assert_eq!(config("hoard-threshold"), d);
        let ratio = |c: &HoardConfig| c.app_scat / c.app_lard;
        assert!((ratio(&config("hoard-threshold")) - 0.22).abs() < 1e-12);
        assert!((ratio(&config("hoard-scatter")) - 0.1).abs() < 1e-12);
        assert!((ratio(&config("hoard-larder")) - 0.4).abs() < 1e-12);
        let only = |id: &str, edit: fn(&mut HoardConfig)| {
            let mut e = d.clone();
            edit(&mut e);
            assert_eq!(config(id), e, "{id}");
        };
        only("hoard-scatter", |c| c.app_scat = 0.2);
        only("hoard-larder", |c| c.app_scat = 0.8);
        only("hoard-no-free-recovery", |c| c.owner_recovery = 0.5);
        only("hoard-cheaters", |c| c.cheaters = 0.25);
        for p in &p {
            assert!(p.source.contains("Minds 7"), "{}", p.id);
            assert!(p.source.contains("Vander Wall & Jenkins 2003"), "{}", p.id);
            match &p.config {
                ModelConfig::Hoard(c) => c.validate().unwrap(),
                _ => unreachable!(),
            }
        }
    }
}
