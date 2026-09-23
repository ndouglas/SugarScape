//! The book's named rule systems. Parameters come from the text of
//! Chapters II–III; `source` cites the animation each reproduces.

use serde::Serialize;

use crate::config::{Config, Outbreak, Placement, ScheduledChange, URange};

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

/// Schedule a single dotted-path change at `tick`.
fn schedule(c: &mut Config, tick: u64, path: &str, value: serde_json::Value) {
    c.schedule.push(ScheduledChange {
        tick,
        set: [(path.to_string(), value)].into_iter().collect(),
    });
}

/// Chapter IV's neoclassical market: 200 immortal agents, symmetric goods.
fn market(c: &mut Config) {
    c.population = 200;
    c.vision = URange::new(1, 5);
    c.metabolism = URange::new(1, 5);
    c.endowment = URange::new(25, 50);
    c.spice.enabled = true;
    c.spice.metabolism = URange::new(1, 5);
    c.spice.endowment = URange::new(25, 50);
    c.trade.enabled = true;
}

/// Animation V-1's disease setup: 10 diseases of length 1–10, 4 per agent,
/// 50-bit immune strings (the `DiseaseRule` defaults).
fn disease(c: &mut Config) {
    c.disease.enabled = true;
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
            "Gathering and eating pollute from t = 50; diffusion spreads it from t = 100 (scheduled, as in the book).",
            |c| {
                schedule(c, 50, "pollution.enabled", serde_json::json!(true));
                schedule(c, 100, "diffusion.enabled", serde_json::json!(true));
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
        preset(
            "iv-1-spice",
            "({G₁}, {M}) with spice",
            "Animation IV-1",
            "Two goods on opposite mountains: agents shuttle between sugar and spice to stay alive.",
            |c| {
                c.vision = URange::new(1, 10);
                c.metabolism = URange::new(1, 5);
                c.endowment = URange::new(25, 50);
                c.spice.enabled = true;
                c.spice.metabolism = URange::new(1, 5);
                c.spice.endowment = URange::new(25, 50);
            },
        ),
        preset(
            "iv-3-trade",
            "({G₁}, {M, T})",
            "Figures IV-3 to IV-5",
            "Bilateral barter between neighbors: prices converge toward the market-clearing level of 1.",
            market,
        ),
        preset(
            "iv-15-trade-sex",
            "({G₁}, {M, S, T})",
            "Figure IV-15",
            "Finite lives and evolving preferences keep prices from settling.",
            |c| {
                market(c);
                c.sex.enabled = true;
                c.lifespan.enabled = true;
            },
        ),
        preset(
            "iv-3-pollution",
            "({G₁, D₁}, {M, T, P})",
            "Animation IV-3",
            "Sugar becomes a dirty good at t = 100, driving its price up; at t = 150 agents stop polluting and the pollution already made diffuses away.",
            |c| {
                market(c);
                schedule(c, 100, "pollution.enabled", serde_json::json!(true));
                // Keep pollution on so what remains still repels agents while
                // diffusion spreads it out (the book's Animation IV-3).
                schedule(c, 150, "pollution.production", serde_json::json!(0.0));
                schedule(c, 150, "pollution.consumption", serde_json::json!(0.0));
                schedule(c, 150, "diffusion.enabled", serde_json::json!(true));
            },
        ),
        preset(
            "iv-18-foresight",
            "({G₁}, {M, S}) with foresight",
            "Figure IV-18",
            "Agents plan φ periods ahead; evolution keeps a modest, non-zero foresight.",
            |c| {
                demography(c);
                // Demography's Chapter III endowment (50-100) is tuned for a
                // single-good economy, where reaching "wealth >= initial
                // endowment" (fertility) just needs sugar. With spice too,
                // fertility needs BOTH sugar and spice simultaneously at that
                // high a bar; the two resources are anti-correlated on the
                // map, so agents almost never clear both at once. Measured:
                // at (50,100)/(50,100) with no trade, seeds 1-3 all crashed
                // to population 0 by t~200 (only ~1 birth in the first 55
                // ticks). Matching the market()-scale endowment (25-50) used
                // by the other Chapter IV presets keeps fertility reachable:
                // population grows from 400 to ~600-700 by t=1000 and mean
                // foresight still declines from a ~5 start (seeds 1-5:
                // 4.87->4.74, 5.01->4.40, 4.86->3.23, 5.09->4.37, 4.82->1.29).
                c.endowment = URange::new(25, 50);
                c.spice.enabled = true;
                c.spice.endowment = URange::new(25, 50);
                c.foresight.enabled = true;
            },
        ),
        preset(
            "iv-5-credit",
            "({G₁}, {M, S, L₁₀,₁₀})",
            "Animation IV-5",
            "Old agents lend to young ones for childbearing; lender–borrower hierarchies emerge.",
            |c| {
                demography(c);
                c.credit.enabled = true;
            },
        ),
        preset(
            "v-1-rid",
            "({G₁}, {M, E})",
            "Animation V-1",
            "Immune systems learn the diseases their agents carry: the society rids itself of disease.",
            disease,
        ),
        preset(
            "v-2-endemic",
            "({G₁}, {M, E}) with 25 diseases",
            "Animation V-2",
            "Too many diseases for one immune string: learning one immunity can overwrite another, and disease stays endemic.",
            |c| {
                disease(c);
                c.disease.count = 25;
                c.disease.initial = 10;
            },
        ),
        preset(
            "v-mcneill",
            "({G₁}, {M, S, E}) + outbreak",
            "Chapter V (after McNeill)",
            "A reproducing society carrying its familiar diseases meets a novel one at t = 300, brought in by 5 agents.",
            |c| {
                demography(c);
                disease(c);
                c.disease.outbreaks = vec![Outbreak {
                    tick: 300,
                    agents: 5,
                }];
            },
        ),
        preset(
            "vi-1-everything",
            "({G₁}, {M, S, I, K, T, L, E})",
            "Chapter VI",
            "Every rule at once: spice, sex, finite lives, inheritance, culture, trade, credit and disease.",
            |c| {
                demography(c);
                c.inheritance.enabled = true;
                c.culture.enabled = true;
                c.spice.enabled = true;
                c.trade.enabled = true;
                c.credit.enabled = true;
                disease(c);
                // Use V-2's endemic disease load (25 diseases, 10 per agent)
                // rather than V-1's defaults: this preset showcases every
                // rule together, so disease should persist, not be learned
                // away the way V-1's "society rids itself" setup is designed
                // to do.
                c.disease.count = 25;
                c.disease.initial = 10;
                // demography()'s Chapter III endowment (50-100) is tuned for
                // a single-good economy; iv-18-foresight found that with a
                // second good (spice), fertility's "wealth >= initial
                // endowment" bar becomes unreachable on both goods at once
                // without trade. Here trade and credit are also on, but with
                // the endemic (25/10) disease load above, measurement at
                // t=1000 (seeds 1-5) shows: 50-100 still collapses population
                // to 0 for every seed; 25-50 keeps population healthy (1823,
                // 1782, 1757, 1767, 1751); 15-40 also survives easily (1848,
                // 1808, 1728, 1836, 1845). 25-50, the first range in the
                // required trial order that clears the >=50-agent bar, is
                // used.
                //
                // infected_fraction at t=250/500/1000 is 0.000 for every seed
                // at all three endowment ranges (population is not the
                // limiting factor here). Diagnostic run (seeds 1-2, 25-50):
                // infected_fraction falls from ~1.0 at t=0 to 0 by t~60-70,
                // and diseases_in_circulation reaches 0 by t=100 - i.e. every
                // disease actually goes extinct, not merely diluted by
                // population growth. This happens during this preset's own
                // early population crash (t=0 400 -> t=50 ~200-223 ->
                // t=100 ~129-226, before the later recovery to ~1750+ by
                // t=1000): the population most exposed to disease is also the
                // population dying fastest (starvation, disease fee, and
                // finite lifespan together), so every disease is cured or
                // dies with its carriers well before the population regrows,
                // and with no remaining carriers disease can never return.
                // v-2-endemic alone (no sex/lifespan, no crash) stays endemic
                // at ~4-6% at the same tick range, so the extinction here is
                // this preset's population dynamics, not the disease
                // parameters. Switching from V-1's defaults (10 diseases, 4
                // per agent) to V-2's (25/10) did not change this outcome:
                // disease still reaches full extinction, if anything slightly
                // earlier. Endowment and diseases-per-agent were the only
                // levers specified for this fix; population stays >=50 at
                // every range tried, so the "fall back to 6 diseases per
                // agent" contingency does not apply here. Left as 25-50/25/10
                // pending further guidance, since no prescribed lever fixes
                // the extinction; see the task-13 fix report for the full
                // measurement tables.
                c.endowment = URange::new(25, 50);
                c.spice.endowment = URange::new(25, 50);
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
        assert_eq!(presets.len(), 23);
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

    #[test]
    fn chapter_v_presets_use_the_books_disease_setups() {
        let v1 = by_id("v-1-rid").unwrap().config;
        let d = &v1.disease;
        assert!(d.enabled);
        assert_eq!(
            (d.count, d.length, d.initial, d.immune_length),
            (10, URange::new(1, 10), 4, 50)
        );
        assert!(!v1.sex.enabled, "Chapter II agents");
        let v2 = by_id("v-2-endemic").unwrap().config.disease;
        assert_eq!((v2.count, v2.initial), (25, 10));
        let m = by_id("v-mcneill").unwrap().config;
        assert!(m.sex.enabled && m.lifespan.enabled && m.disease.enabled);
        assert_eq!(
            m.disease.outbreaks,
            vec![Outbreak {
                tick: 300,
                agents: 5
            }]
        );
        let e = by_id("vi-1-everything").unwrap().config;
        assert!(
            e.spice.enabled
                && e.sex.enabled
                && e.lifespan.enabled
                && e.inheritance.enabled
                && e.culture.enabled
                && e.trade.enabled
                && e.credit.enabled
                && e.disease.enabled
        );
        assert!(!e.combat.enabled && !e.replacement.enabled);
    }

    #[test]
    fn iv_3_pollution_stops_producing_but_keeps_its_pollution() {
        let config = by_id("iv-3-pollution").unwrap().config;
        let mut c = config.clone();
        for change in &config.schedule {
            c = c.apply_change(change).unwrap();
        }
        assert!(
            c.pollution.enabled,
            "existing pollution still repels agents"
        );
        assert!(c.diffusion.enabled);
        assert_eq!(
            (c.pollution.production, c.pollution.consumption),
            (0.0, 0.0)
        );
    }
}
