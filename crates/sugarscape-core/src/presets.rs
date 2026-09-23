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
            "Immune systems learn the diseases their agents carry: near-eradication (a residue of ~1-3% persists because learning one disease can overwrite the window that cured another).",
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
                // A 10-bit length (the top of disease.length's own 1-10
                // range, but forced rather than left to chance) keeps the
                // outbreak's disease genuinely novel: task-14's book test
                // found that with the default 1-10 draw, a short length
                // (1-3 bits) is very likely already a substring of most
                // agents' 50-bit immune strings by pure chance, so the
                // outbreak sometimes "infects" 5 agents who are immediately
                // immune and nothing spreads. A 10-bit string is unlikely to
                // already be present, so the outbreak reliably takes hold
                // and is then transmitted onward.
                c.disease.outbreaks = vec![Outbreak {
                    tick: 300,
                    agents: 5,
                    length: Some(URange::new(10, 10)),
                }];
            },
        ),
        preset(
            "vi-1-everything",
            "({G₁}, {M, S, I, K, T, L, E})",
            "Chapter VI",
            "Every rule at once: spice, sex, finite lives, inheritance, culture, trade, credit and disease, with new diseases arriving by outbreak at t = 150, 400 and 650 (as in the book's McNeill discussion).",
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
                c.endowment = URange::new(25, 50);
                c.spice.endowment = URange::new(25, 50);
                // Fix round 1 found that with the endemic (25/10) disease
                // load alone, this preset's own early population crash (sex
                // + lifespan + spice + trade + credit together: t=0 400 ->
                // t~100 ~130-230, before recovering to ~1750+ by t=1000)
                // wipes out every carried disease by t~60-100, and with no
                // remaining carriers disease can never return -
                // infected_fraction is 0.000 from then on. Fix round 2
                // (this): scheduled outbreaks reseed a novel disease at
                // t=150, 400 and 650, each offered to 20 agents (as in the
                // book's McNeill discussion of new diseases meeting a
                // settled society) - well after the crash and spaced through
                // the growth/plateau phase.
                //
                // Measured (seeds 1-5, t=1000 population and infected_fraction
                // at t=200/500/800/1000): population stays healthy (1810,
                // 1770, 1760, 1776, 1737), well above the 50-agent bar.
                // infected_fraction is 0.000 at all four sampled ticks for
                // every seed: each outbreak's disease is a fresh random
                // string (length 1-10, same as disease.length), and a large,
                // rapidly-adapting population (per round 1) makes most
                // agents already immune to short strings by chance (a 50-bit
                // immune string is very likely to already contain any
                // length-1 or length-2 pattern as a substring), so the takes
                // are small and short-lived. A finer-grained diagnostic
                // confirmed the outbreaks do fire and occasionally infect a
                // few agents (e.g. one seed saw 9 new infections in the 10
                // ticks after the t=150 outbreak, and a brief 0.5% blip
                // shortly after the t=400 outbreak in another seed) but each
                // spike is gone again within single-digit ticks, so it
                // never survives to the 200/500/800/1000 sampling points.
                // Per the controller's round-2 ruling this (infection
                // vanishing between outbreaks) is an acceptable outcome.
                //
                // Fix round 3 (task-14 review): each `Outbreak` below now
                // pins `length: Some(10, 10)` instead of drawing from the
                // default 1-10-bit `disease.length` range, so its disease is
                // reliably novel rather than sometimes already a substring
                // of most agents' immune strings by chance. Re-measured
                // (seeds 1-5, t=1000 population): 1741, 1787, 1746, 1764,
                // 1773 - still comfortably above the 50-agent bar (the
                // change only affects the outbreak's own disease length, not
                // endowment or the endemic 25/10 load). infected_fraction at
                // t=200/500/800/1000 is still 0.000 for every seed - the
                // sampling points are unchanged and still land well after
                // each outbreak clears - but a finer-grained trace now shows
                // each outbreak taking hold far more substantially than
                // round 2's short-string draws: peak infected_fraction in
                // the 20 ticks after each outbreak (seeds 1-3) ranges
                // 4.8%-16.8%, with 51-1155 new infections summed over that
                // window (vs. round 2's single-digit counts), and it is
                // still fully cleared again by the next 50-150-tick-later
                // sampling point. The controller's round-2 ruling (infection
                // vanishing between outbreaks is an acceptable outcome for
                // this preset) still applies; only the outbreaks' own
                // severity changed, not the sampled headline numbers.
                c.disease.outbreaks = vec![
                    Outbreak {
                        tick: 150,
                        agents: 20,
                        length: Some(URange::new(10, 10)),
                    },
                    Outbreak {
                        tick: 400,
                        agents: 20,
                        length: Some(URange::new(10, 10)),
                    },
                    Outbreak {
                        tick: 650,
                        agents: 20,
                        length: Some(URange::new(10, 10)),
                    },
                ];
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
                agents: 5,
                length: Some(URange::new(10, 10)),
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
