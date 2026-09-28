//! The book's named rule systems. Parameters come from the text of
//! Chapters II–III; `source` cites the animation each reproduces.

use serde::Serialize;

use crate::config::{
    three_tribes, Config, CultureKind, DecisionRule, Good, Idle, Map, MoveMode, Outbreak, Peak,
    Placement, Pollutant, Pollution, ScheduledChange, Transform, URange, Wall, SPICE_COLOR,
};
use crate::model::ModelConfig;

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
    c.goods[0].endowment = URange::new(50, 100);
}

/// Schedule a single dotted-path change at `tick`.
fn schedule(c: &mut Config, tick: u64, path: &str, value: serde_json::Value) {
    c.schedule.push(ScheduledChange {
        tick,
        set: [(path.to_string(), value)].into_iter().collect(),
    });
}

/// Chapter IV's second good: spice on the two-peak map mirrored left↔right.
fn spice(c: &mut Config, metabolism: URange, endowment: URange) {
    c.add_good(Good {
        metabolism,
        endowment,
        ..Good::spice()
    });
}

/// Chapter IV's neoclassical market: 200 immortal agents, symmetric goods.
fn market(c: &mut Config) {
    c.population = 200;
    c.vision = URange::new(1, 5);
    c.goods[0].metabolism = URange::new(1, 5);
    c.goods[0].endowment = URange::new(25, 50);
    spice(c, URange::new(1, 5), URange::new(25, 50));
    c.trade.enabled = true;
}

/// Animation V-1's disease setup: 10 diseases of length 1–10, 4 per agent,
/// 50-bit immune strings (the `DiseaseRule` defaults).
fn disease(c: &mut Config) {
    c.disease.enabled = true;
}

/// Chapter VI's indecomposability society (animations VI-2 and VI-3): 500
/// agents with Chapter IV's traits on its sugar and spice landscape under M
/// and S with Chapter III's demography (lifetimes 60-100); `trade` switches
/// rule T, the only difference.
fn indecomposability(c: &mut Config, trade: bool) {
    c.population = 500;
    demography(c);
    // No calibration: Chapter IV's traits, as the spec fixes them. Measured
    // (`measure_indecomposability`, release), population every 50 ticks
    // from t = 0 to 1000, seeds 1-5:
    //   vi-2-no-trade seed 1: [500, 250, 192, 383, 724, 933, 857, 814, 840, 845, 838, 866, 809, 744, 854, 910, 866, 872, 857, 827, 851]
    //   vi-2-no-trade seed 2: [500, 253, 191, 340, 623, 850, 782, 742, 745, 806, 856, 828, 828, 821, 814, 816, 842, 848, 870, 842, 815]
    //   vi-2-no-trade seed 3: [500, 261, 151, 253, 555, 809, 843, 791, 762, 752, 784, 867, 838, 809, 733, 769, 873, 886, 871, 893, 856]
    //   vi-2-no-trade seed 4: [500, 276, 244, 462, 812, 883, 796, 790, 756, 776, 890, 907, 865, 880, 885, 845, 810, 844, 880, 845, 781]
    //   vi-2-no-trade seed 5: [500, 270, 176, 300, 532, 767, 880, 808, 782, 796, 791, 745, 787, 866, 849, 784, 833, 877, 858, 864, 880]
    //   vi-3-trade seed 1: [500, 246, 130, 330, 746, 989, 890, 844, 832, 834, 829, 868, 879, 873, 844, 770, 785, 874, 838, 770, 857]
    //   vi-3-trade seed 2: [500, 291, 178, 385, 755, 926, 842, 807, 813, 811, 843, 856, 854, 824, 821, 829, 785, 760, 816, 768, 793]
    //   vi-3-trade seed 3: [500, 275, 121, 162, 346, 653, 821, 726, 706, 775, 767, 747, 742, 823, 829, 806, 787, 779, 735, 727, 747]
    //   vi-3-trade seed 4: [500, 280, 118, 145, 403, 836, 965, 901, 871, 888, 859, 881, 847, 850, 883, 912, 905, 854, 851, 856, 860]
    //   vi-3-trade seed 5: [500, 284, 124, 209, 485, 821, 880, 808, 785, 786, 791, 776, 838, 914, 890, 884, 876, 883, 900, 907, 898]
    // VI-3 follows the book's curve (a dip by t ~ 100, recovery to 1.7-2.0x
    // the initial 500, minima near 700; the book's ~115-year period between
    // peaks isn't measured here); VI-2 does the same instead of
    // crashing. Every stated rule (M, S, T, death, the landscape) matches the
    // book and Appendix B, and no setting of 216 tried separated the two on
    // all of seeds 1-5 except on a knife edge: under these rules trade moves
    // holdings toward each agent's metabolism ratio but does not raise
    // fertility. The crash most likely depended on unreported details of
    // the original software.
    c.vision = URange::new(1, 10);
    c.goods[0].metabolism = URange::new(1, 5);
    c.goods[0].endowment = URange::new(25, 50);
    spice(c, URange::new(1, 5), URange::new(25, 50));
    c.trade.enabled = trade;
}

/// A further good with good 0's trait ranges, named `name` in `color` on `map`.
fn another(c: &mut Config, name: &str, color: &str, map: Map) {
    let like = c.goods[0].clone();
    c.add_good(Good {
        name: name.into(),
        color: color.into(),
        map,
        ..like
    });
}

/// Gives every good the same metabolism and endowment ranges.
fn traits(c: &mut Config, metabolism: URange, endowment: URange) {
    for g in &mut c.goods {
        g.metabolism = metabolism;
        g.endowment = endowment;
    }
}

/// Animation II-4/II-5's finite lifetimes with replacement.
fn wealth(c: &mut Config) {
    c.lifespan.enabled = true;
    c.replacement.enabled = true;
}

/// Animation II-7's seasons.
fn enable_seasons(c: &mut Config) {
    c.seasons.enabled = true;
}

/// The book's first frame for Animation II-6: a 20×20 block in the
/// bottom-left corner, otherwise as Animation II-2 (400 agents, so full).
/// Surveyed (docs/survey/2026-09-24-model-survey.md): the ring of the book's
/// second frame appears by t ≈ 8, but no northeasterly waves follow on any
/// seed.
fn waves(c: &mut Config) {
    c.placement = Placement::Block {
        x: 0,
        y: 30,
        width: 20,
        height: 20,
    };
    c.vision = URange::new(1, 10);
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
            wealth,
        ),
        preset(
            "ii-6-waves",
            "Diagonal waves",
            "Animation II-6",
            "A full 20×20 block of agents with vision up to 10 starts in the southwest corner, as in the book, and bursts outward as a ring. The book's waves then travel northeast; here they don't: the survivors settle on the southwest mountain.",
            waves,
        ),
        preset(
            "ii-7-seasons",
            "({S₁,₈,₅₀}, {M})",
            "Animation II-7",
            "Seasons flip every 50 ticks: high-vision agents migrate, low-vision low-metabolism agents hibernate.",
            enable_seasons,
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
            "Cultural transmission: tag-flipping makes neighbors alike, so each sugar mountain becomes one tribe. On about half of runs the two mountains settle on different tribes.",
            |c| c.culture.enabled = true,
        ),
        preset(
            "iii-6-three-tribes",
            "({G₁}, {M, K}) with three tribes",
            "Chapter III, note 20",
            "Cultural transmission with the book's three-group tag scheme: Blue 0–3 zeros, Green 4–7, Red 8–11. Watch the Group shares chart.",
            |c| {
                c.culture.enabled = true;
                c.culture.groups = three_tribes(c.tag_length);
            },
        ),
        preset(
            "iii-9-combat",
            "({G₁}, {C∞})",
            "Animation III-9",
            "Unlimited combat between two tribes: the winner takes its victim's whole wealth. An agent may attack only a poorer enemy, and not where a richer one would see it, so equals never fight and the fighting starts slowly. In most runs one agent grows rich enough that no enemy may attack it, makes nearly all the kills, and its tribe wipes out the other, or nearly, by t = 2000: the book's blitzkrieg.",
            |c| {
                tribes(c);
                c.combat.enabled = true;
            },
        ),
        preset(
            "iii-11-combat-fixed",
            "({G₁}, {C₂, R[60,100]})",
            "Animation III-11",
            "Fixed reward of 2 per kill, population held at 400 by replacement. The book reports coherent battle fronts; under its stated rule, which puts each replacement at a random site, the fighting never stops but no front forms: kills spread over the whole board, and most of the dead are newcomers killed within a few ticks of landing among enemies.",
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
            "Opposed blocks of Blues and Reds with vision up to 10 start in the southwest and northeast corners, as in the book (combat off). In the book they travel toward the center in waves that collide; here each tribe settles on its own mountain and they never meet.",
            |c| {
                tribes(c);
                c.vision = URange::new(1, 10);
            },
        ),
        preset(
            "iii-14-combat-culture",
            "({G₁}, {C∞, K})",
            "Animation III-14",
            "Combat with cultural transmission. The book shows invaders converted before they can conquer; with its random tags many agents start one flip from the other tribe, so converts appear at once and fight their own side: a civil war leaves fewer than 20 agents by t = 100, and combat leaves far fewer conversions than culture alone.",
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
            "Two goods on opposite mountains: to stay alive, about half the agents shuttle back and forth between sugar and spice.",
            |c| {
                c.vision = URange::new(1, 10);
                c.goods[0].metabolism = URange::new(1, 5);
                c.goods[0].endowment = URange::new(25, 50);
                spice(c, URange::new(1, 5), URange::new(25, 50));
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
            "Figure IV-14",
            "Finite lives and evolving preferences keep prices from settling: their dispersion grows over time. With the book's fertility ages (childbearing ending at 35–45 for women, 45–55 for men), most runs die out: 15 of 20 by t = 1000.",
            |c| {
                market(c);
                c.sex.enabled = true;
                c.lifespan.enabled = true;
                // The book's Figure IV-14: childbearing ends at 35–45 for
                // women and 45–55 for men (not Chapter III's 40–50 / 50–60).
                c.sex.female_end = URange::new(35, 45);
                c.sex.male_end = URange::new(45, 55);
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
                schedule(c, 150, "pollution.pollutants.0.production.0", serde_json::json!(0.0));
                schedule(c, 150, "pollution.pollutants.0.consumption.0", serde_json::json!(0.0));
                schedule(c, 150, "diffusion.enabled", serde_json::json!(true));
            },
        ),
        preset(
            "iv-18-foresight",
            "({G₁}, {M, S}) with foresight",
            "Figure IV-18",
            "Agents plan φ periods ahead. Mean foresight starts near 5 and drifts down only slightly (to about 4.4 by t = 1000); it is not reliably selected down.",
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
                c.goods[0].endowment = URange::new(25, 50);
                spice(c, URange::new(1, 4), URange::new(25, 50));
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
            "Immune systems learn the diseases their agents carry, one immune bit per tick (the book's note 16): society rids itself of every disease, as the book says (disease-free by t = 1000 in 20 of 20 runs). With a flip for each carried disease instead (learning: per_disease), learning one disease can undo another's flips and a small residue persists.",
            disease,
        ),
        preset(
            "v-2-endemic",
            "({G₁}, {M, E}) with 25 diseases",
            "Animation V-2",
            "Animation V-2: 25 diseases, 10 per agent. The book finds an endemic level of infection; under its one-flip-per-tick rule (note 16) disease clears here too (disease-free by t = 1000 in 20 of 20 runs). Only a flip for each carried disease (learning: per_disease) keeps it endemic, from agents stuck between diseases whose flips undo each other.",
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
            "A reproducing society that has learned away its familiar diseases by t = 300 meets a novel one, given to 5 agents (each already immune with probability about 4%).",
            |c| {
                demography(c);
                disease(c);
                // A 10-bit length (the top of disease.length's own 1-10
                // range, but forced rather than left to chance) keeps the
                // outbreak's disease genuinely novel: with the default 1-10
                // draw, a short length (1-3 bits) is very likely already a
                // substring of most agents' 50-bit immune strings by pure
                // chance, so the outbreak sometimes "infects" 5 agents who
                // are immediately immune and nothing spreads. A 10-bit
                // string is unlikely to already be present, so the outbreak
                // reliably takes hold and is then transmitted onward.
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
            "Every rule at once: spice, sex, finite lives, inheritance, culture, trade, credit and disease, with new diseases arriving by outbreak at t = 150, 400 and 650; disease flares after the outbreaks (after all three on about three runs in four) and tends to die out again before the next one. The book's eighteen views, in its order: (1) Agents → Disease colors; (2) the Neighbor network overlay; (3) Charts → Wealth distribution (sugar); (4) Charts → Goods → Wealth distribution · spice; (5) Charts → Goods → Lorenz curve and Gini coefficient (total wealth); (6) Charts → Population; (7) Charts → Age histogram; (8) the Family network overlay (with Agents → Lineage for the book's colors); (9) Charts → Cultural tags; (10) the Friends network overlay; (11) and (12) Charts → Economy → Trade price (its mean and ± SD band); (13) Charts → Economy → Trade volume; (14) the Trade network overlay; (15) the Credit network overlay; (16) the Credit tab's hierarchy; (17) Charts → Disease; (18) the Disease network overlay.",
            |c| {
                demography(c);
                c.inheritance.enabled = true;
                c.culture.enabled = true;
                spice(c, URange::new(1, 4), URange::new(25, 50));
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
                c.goods[0].endowment = URange::new(25, 50);
                // With the endemic (25/10) disease load alone, this preset's
                // own early population crash (sex + lifespan + spice + trade
                // + credit together: t=0 400 -> a trough within t<=200 of
                // 78-189, measured seeds 1-5 release as 115, 189, 78, 82,
                // 167 -- re-measured after per-good credit -- before
                // recovering to ~1750+ by t=1000) wipes out every carried
                // disease by t~60-100, and with no remaining carriers
                // disease can never return - infected_fraction stays 0.000
                // from then on. Scheduled outbreaks reseed a novel disease
                // at t=150, 400 and 650, each offered to 20 agents (as in
                // the book's McNeill discussion of new diseases meeting a
                // settled society) - well after the crash and spaced
                // through the growth/plateau phase.
                //
                // Each `Outbreak` pins `length: Some(10, 10)` rather than
                // drawing from the default 1-10-bit `disease.length` range:
                // a short string (1-3 bits) is very likely already a
                // substring of most agents' 50-bit immune strings by pure
                // chance in a large, rapidly-adapting population, so the
                // outbreak's take would be small and short-lived. A 10-bit
                // string is unlikely to already be present, so each
                // outbreak reliably takes hold.
                //
                // Measured (seeds 1-5, t=1000 population; re-measured when
                // credit became per-good): 1832, 1767, 1800, 1790, 1752 -
                // comfortably above the 50-agent bar.
                // infected_fraction is 0.000 at every one of the
                // t=200/500/800/1000 sampling points for every seed, but a
                // finer-grained trace shows each outbreak does take hold
                // substantially before clearing again. Re-measured (seeds
                // 1-5, release, max infected_fraction within 50 ticks after
                // each outbreak): t=150 -> 18.8%, 8.0%, 15.7%, 9.7%, 7.0%;
                // t=400 -> 7.3%, 7.1%, 6.7%, 3.8%, 8.3%; t=650 -> 7.1%,
                // 10.4%, 22.4%, 8.5%, 14.7% (seeds 1-5 respectively) --
                // peaks range 3.8%-22.4% across every seed and outbreak,
                // and it is fully cleared again by the next 50-150-tick-
                // later sampling point. Disease vanishing between outbreaks
                // (rather than persisting endemically, as in v-2-endemic)
                // is an accepted property of this preset; only the
                // outbreaks reseed it.
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
        preset(
            "vi-2-no-trade",
            "({G₁}, {M, S}) with spice, no trade",
            "Animation VI-2",
            "500 agents with Chapter IV's traits move and reproduce on the sugar and spice landscape but never trade. The book's population crashes; here it does not: it dips to about 150–235 by t = 100–150, recovers to about 1.8 times its start and fluctuates around 800, like VI-3. Every stated rule matches the book, so the crash most likely depended on unreported details of the original software. Compare it with VI-3 from the presets menu.",
            |c| indecomposability(c, false),
        ),
        preset(
            "vi-3-trade",
            "({G₁}, {M, S, T}) with spice",
            "Animation VI-3",
            "Everything as in VI-2, with trade on. This follows the book's curve: the population dips to about 100–175 by t = 100–150, recovers to 1.7–2.0 times its initial 500, then fluctuates with minima near 750. (VI-2 without trade does the same here, unlike the book.)",
            |c| indecomposability(c, true),
        ),
        preset(
            "n-3-trade",
            "({G₁}, {M, S, T}) with three goods",
            "Chapter IV, footnote 7",
            "Sugar, spice and salt on three turned copies of the two-peak map: neighbors barter over whichever pair they value most differently, and prices form for all three pairs.",
            |c| {
                market(c);
                demography(c);
                another(c, "salt", "#7fb3d5", Map::TwoPeaks { transform: Transform::Rotate90 });
                // Measured (`measure_n_goods_presets`, release, seeds 1-5,
                // t=1000 population, total trades over t=1..1000): tried in
                // the brief's order (metabolism outer, endowment inner).
                //   (1,5)/(25,50): [0, 0, 0, 0, 0], trades 61570
                //   (1,5)/(50,100): [0, 0, 0, 0, 0], trades 109786
                //   (1,5)/(15,40): [0, 209, 864, 0, 0], trades 709097
                //   (1,4)/(25,50): [665, 0, 0, 0, 0], trades 606163
                //   (1,4)/(50,100): [0, 0, 0, 0, 0], trades 139432
                //   (1,4)/(15,40): [898, 0, 875, 888, 904], trades 1915884
                //   (1,3)/(25,50): [0, 0, 635, 0, 0], trades 530979
                //   (1,3)/(50,100): [0, 0, 0, 0, 0], trades 155457
                //   (1,3)/(15,40): [858, 820, 838, 791, 760], trades 2722255
                // Three goods, each with its own per-tick metabolism, split
                // the market()-scale 200-agent population three ways over
                // turned copies of the two-peak map; every combination with
                // a (25,50) or (50,100) endowment starves out on at least
                // one seed, and even (15,40) only survives once metabolism
                // is down to its minimum range (1,3). (1,3)/(15,40) is the
                // first combination in the required order whose five t=1000
                // populations are all >=50 with trades > 0, so it is used.
                traits(c, URange::new(1, 3), URange::new(15, 40));
            },
        ),
        preset(
            "n-4-peaks",
            "({G₁}, {M, T}) with four goods",
            "Chapter IV, footnote 7",
            "Four goods, each on one peak near a different corner of the torus. The peaks overlap near the wrapped corner, where agents can hold all four; elsewhere they must travel or trade.",
            |c| {
                let corner = |x, y| Map::Peaks {
                    peaks: vec![Peak { x, y, radius: 20.0, height: 4.0 }],
                };
                c.vision = URange::new(1, 10);
                c.goods[0].map = corner(10, 10);
                another(c, "spice", SPICE_COLOR, corner(39, 10));
                another(c, "salt", "#7fb3d5", corner(10, 39));
                another(c, "silk", "#8fcf6b", corner(39, 39));
                c.trade.enabled = true;
                // Measured (`measure_n_goods_presets`, release, seeds 1-5,
                // t=1000 population, total trades over t=1..1000): tried in
                // the brief's order (metabolism outer, endowment inner).
                //   (1,3)/(25,50): [36, 41, 44, 38, 40], trades 237200
                //   (1,3)/(50,100): [52, 50, 49, 50, 47], trades 531513
                //   (1,2)/(25,50): [83, 95, 92, 94, 88], trades 685932
                // Four goods, each on a single small peak near a different
                // corner of the default 50x50 torus with default population
                // 400, leave each corner's capacity scarce relative to
                // demand; (1,3) metabolism starves the population below 50
                // on at least one seed at both endowments tried (including
                // one seed at only 47 with (50,100)). (1,2)/(25,50) is the
                // first combination in the required order whose five t=1000
                // populations are all >=50 with trades > 0, so it is used.
                traits(c, URange::new(1, 2), URange::new(25, 50));
            },
        ),
        preset(
            "n-2-pollutants",
            "({G₁, D₁}, {M, P}) with two pollutants",
            "Appendix B, rule P",
            "Sugar gives off smoke, which makes sugar sites less attractive; spice gives off runoff, which spoils spice sites. Both diffuse.",
            |c| {
                c.vision = URange::new(1, 10);
                c.goods[0].metabolism = URange::new(1, 5);
                c.goods[0].endowment = URange::new(25, 50);
                spice(c, URange::new(1, 5), URange::new(25, 50));
                // Measured (`measure_n_goods_presets`, release, seeds 1-5,
                // t=1000 population; seed-1 mean pollution [smoke, runoff]
                // at t=1000): tried in the brief's order.
                //   k=1.0: [82, 86, 85, 86, 85], pollution [154.96, 161.48]
                //   k=0.5: [80, 89, 86, 90, 82], pollution [77.03, 79.43]
                //   k=0.25: [80, 91, 89, 88, 85], pollution [39.11, 40.73]
                // k=1.0 is the first coefficient in the required order whose
                // five t=1000 populations are all >=50 with nonzero mean
                // pollution for both pollutants, so it is used.
                let k = 1.0;
                let pollutant = |name: &str, good: usize| Pollutant {
                    name: name.into(),
                    production: (0..2).map(|i| if i == good { k } else { 0.0 }).collect(),
                    consumption: (0..2).map(|i| if i == good { k } else { 0.0 }).collect(),
                    devalues: (0..2).map(|i| i == good).collect(),
                };
                c.pollution = Pollution {
                    enabled: true,
                    pollutants: vec![pollutant("smoke", 0), pollutant("runoff", 1)],
                };
                c.diffusion.enabled = true;
            },
        ),
        preset(
            "dock-mobility-15",
            "Docking: Axelrod's culture on the move, 15 traits",
            "Axtell, Axelrod, Epstein & Cohen 1996, §4.3.1",
            "Axtell, Axelrod, Epstein and Cohen's mobility experiment (1996): 100 agents with vision 5–10 on one sugar mountain in the middle of the 50 × 50 torus move to the richest site they see, eat, then run Axelrod's culture rule (5 features of 15 traits) with one random neighbor; nobody starves. The run stops once every two agents' cultures are identical or share nothing. They report 1.1 ± 0.3 cultures over 10 runs, all of which stopped; their mountain's shape is not given (here: one cone of radius 35, height 4). Measured (20 seeds, 20 000 ticks): 4.4 ± 1.4 cultures, and 17 of 20 runs never stop — one culture takes almost everyone, but a few stragglers that rarely meet anyone keep second and third cultures alive. Mobility still collapses diversity (the fixed 10 × 10 lattice keeps about 20), as they say; their numbers do not reproduce.",
            |c| docking(c, 15),
        ),
        preset(
            "dock-mobility-30",
            "Docking: Axelrod's culture on the move, 30 traits",
            "Axtell, Axelrod, Epstein & Cohen 1996, §4.3.1",
            "The mobility experiment with 30 traits per feature. Axtell et al. 1996: 2.2 ± 1.2 cultures, more than with 15 traits. Measured (20 seeds, 20 000 ticks): 5.7 ± 1.6, and 13 of 20 runs never stop — more than with 15 traits, as they found, but well above their count.",
            |c| docking(c, 30),
        ),
        preset(
            "ifd-even",
            "Ideal free distribution: equal patches",
            "Fretwell & Lucas 1969; Minds 1",
            "Two cone-shaped sugar patches of the same size (305 sites each, input 0.25 a tick per site) on a 60 × 40 torus, and 100 Flumps of metabolism 1 and vision 1–6. The ideal free distribution predicts an even split. Measured (20 seeds, tick 1000): 0.98 as many Flumps on the first patch as the second, with 45 of 100 alive. Measured: the on-patch counts are the same when nobody starves, so the dead are Flumps who never found sugar.",
            |c| two_patches(c, 10.0),
        ),
        preset(
            "ifd-two-to-one",
            "Ideal free distribution: 2.1 : 1",
            "Parker 1978; Milinski 1979; Minds 1",
            "The second patch has radius 7 (145 sites against 305: input ratio 2.10, near Milinski's 2 : 1). Input matching predicts 2.10 times as many Flumps on the richer patch. Measured (20 seeds, tick 1000): 1.71 times. Fitted per seed across input ratios 1, 1.36, 2.10, 2.80 and 4.42, the matching exponent s has median 0.72 (s = 1 is matching). Parker's input matching fails (4 of 20 seeds within 0.9–1.1); undermatching, which Kennedy & Gray 1993 report for most animal experiments, holds. The median is close to the catchment prediction of 0.71, which counts the sites from which each patch is in sight. But the seeds scatter widely (IQR 0.59–0.92), and only 6 of 20 land within 0.1 of it, so that claim fails as judged.",
            |c| two_patches(c, 7.0),
        ),
        preset(
            "ifd-four-to-one",
            "Ideal free distribution: 4.4 : 1",
            "Parker 1978; Minds 1",
            "The second patch has radius 5 (69 sites: input ratio 4.42). Input matching predicts 4.42 times as many Flumps on the richer patch. Measured (20 seeds, tick 1000): 2.96 times.",
            |c| two_patches(c, 5.0),
        ),
        preset(
            "ifd-far-sighted",
            "Ideal free distribution: vision 10–20",
            "Kennedy & Gray 1993; Minds 1",
            "The 2.10 : 1 patches with vision 10–20, far enough to see across the 7-site gap between the patches. Measured (20 seeds, tick 1000): 1.85 times as many Flumps on the richer patch; s has median 0.90 across the five input ratios, close to matching but still under it. Every seed undermatches, and only 8 of 20 are within 0.9–1.1, so Parker's input matching narrowly fails here too.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
            },
        ),
        preset(
            "ifd-no-starving",
            "Ideal free distribution: nobody starves",
            "Fretwell & Lucas 1969; Minds 1",
            "The 2.10 : 1 patches with an endowment of 100 000, so nobody starves within the run. Rule M keeps a Flump in place when nothing it sees is better, so one that starts out of sight of sugar never moves. Measured (20 seeds, tick 1000): a median 66 of 100 off both patches (over half in all 20 seeds), so the 'free' of the ideal free distribution fails. The on-patch counts are the same as with starvation, and s is identical seed by seed in all 20 seeds, so the survival claim holds: the Flumps who die are the ones who never find sugar.",
            |c| {
                two_patches(c, 7.0);
                c.goods[0].endowment = URange::new(100_000, 100_000);
            },
        ),
        preset(
            "ifd-wander",
            "Utility mind: wander when nothing scores",
            "Minds 1",
            "The no-starving world under the utility mind with idle wander: a Flump that sees no sugar moves to a random free site in sight instead of staying put. It tests the 'free' of the ideal free distribution apart from the 'ideal'. Measured (20 seeds, tick 1000): under 1 of 100 off both patches, so wandering does make the Flumps free. But only 1.43 times as many are on the richer patch (1.71 under stay), and s falls from 0.72 to 0.40, away from matching, so the claim that wandering moves s toward 1 fails. Since nobody starves, the wanderers overfill the poorer patch (41 Flumps on its 36 sugar a tick), so the split likely follows where they arrive, not the inputs.",
            |c| {
                two_patches(c, 7.0);
                c.goods[0].endowment = URange::new(100_000, 100_000);
                c.decision.rule = DecisionRule::Utility;
                c.decision.idle = Idle::Wander;
            },
        ),
        preset(
            "ifd-crowding",
            "Utility mind: crowding m = 1",
            "Sutherland 1983; Minds 1",
            "The 2.10 : 1 patches with vision 10–20 (far enough to see both patches, as in ifd-far-sighted) under the utility mind with crowding m = 1: a site's welfare is divided by (1 + n), n the Flumps next to it. Sutherland's interference model predicts input matching at m = 1; here the interference is local. We expected local crowding only to push Flumps apart and lower s. Measured (20 seeds, tick 1000): 1.93 times as many Flumps on the richer patch (1.85 without crowding). Across the five input ratios s has median 0.95 against 0.90 at m = 0, with 18 of 20 seeds within 0.9–1.1. So Sutherland's matching holds, and crowding raises s (one-sided Mann–Whitney p = 0.0003).",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.decision.rule = DecisionRule::Utility;
                c.decision.crowding = 1.0;
            },
        ),
        preset(
            "ifd-travel",
            "Utility mind: travel k = 0.5",
            "Baum & Kraft 1998; Minds 1",
            "The 2.10 : 1 patches with vision 10–20 (far enough to see both patches, as in ifd-far-sighted) under the utility mind with travel k = 0.5: a site's welfare is divided by (1 + 0.5·d), d its distance. Baum & Kraft found that requiring travel to switch patches slightly reduced undermatching; here travel is a preference for nearby sugar under rule M's one-tick jump, not a cost of switching. Measured (20 seeds, tick 1000): 1.52 times as many Flumps on the richer patch (1.85 without travel). Across the five input ratios s has median 0.73 against 0.90 at k = 0, so undermatching grows, the opposite of Baum & Kraft's direction, and their claim fails here.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.decision.rule = DecisionRule::Utility;
                c.decision.travel = 0.5;
            },
        ),
        preset(
            "walk-capacity",
            "Walking: carrying capacity",
            "Epstein & Axtell II-2; Minds 2",
            "Rule M's Flumps jump to the best site in sight; these walk there one step a tick along an A* path. Measured (20 seeds, mean population over ticks 300–500): 181 against 228 under the jump, so the book's carrying capacity of about 224 fails under walking (no seed within 214–234), and walking lowers it in every seed, by a median 47 Flumps. In the walk-speed and walk-vision sweeps (an observation, not a judged claim), capacity tracks how far a Flump gets in a tick, roughly min(speed, vision): walking at vision 1–6 (181.5) is about jumping at vision 1 (182.7), and walking at speed 3 (212.8) and 6 (225.2) is near jumping at vision 1–3 (205.5) and 1–6 (228.5). Walking faster brings the population back (see walk-fast and the walk-speed sweep).",
            |c| c.movement.mode = MoveMode::Walk,
        ),
        preset(
            "walk-wealth",
            "Walking: wealth distribution",
            "Epstein & Axtell II-5; Minds 2",
            "Finite lifetimes with replacement, as in ii-5-wealth, but rule M's Flumps walk to the best site they see instead of jumping there in one tick. Measured (20 seeds, tick 500): wealth is still right-skewed in every seed, so the book's skewed distribution (Animation II-5) doesn't need the jump. The skewness has median 1.26 against 1.27 under the jump, and the Gini 0.46 against 0.48.",
            |c| {
                wealth(c);
                c.movement.mode = MoveMode::Walk;
            },
        ),
        preset(
            "walk-seasons",
            "Walking: seasons",
            "Epstein & Axtell II-7; Minds 2",
            "Seasons flipping every 50 ticks, as in ii-7-seasons, but rule M's Flumps walk to the best site they see instead of jumping there in one tick. Measured (20 seeds, Flumps alive over ticks 100–300): a median 55 % change hemisphere at least twice, against 83 % under the jump, so migration with the seasons (Animation II-7) survives walking, but fewer Flumps migrate. Likely cause: crossing to the other hemisphere takes a walker many ticks, so fewer finish the crossing before the season or their target changes.",
            |c| {
                enable_seasons(c);
                c.movement.mode = MoveMode::Walk;
            },
        ),
        preset(
            "walk-waves",
            "Walking: diagonal waves",
            "Epstein & Axtell II-6; Minds 2",
            "The book's waves travel northeast from the southwest block; under rule M's jump they don't. Measured (20 seeds, tick 100): a median 0.6 % of Flumps are farther than 25 sites from the block's center, against 0.8 % under the jump; no seed comes near a quarter (in planning, the block settled on the near mountain under both). So walking isn't the missing mechanism behind the waves (Animation II-6).",
            |c| {
                waves(c);
                c.movement.mode = MoveMode::Walk;
            },
        ),
        preset(
            "walk-fast",
            "Walking: speed 3",
            "Minds 2",
            "ii-2-unit's unit growback and carrying capacity, but Flumps walk three steps a tick instead of one along their A* path. Measured (20 seeds, mean population over ticks 300–500): 214, between walking at one step a tick (181) and the jump (228). The capacity under walk rises toward the jump's as speed rises: at ten steps a tick it is 227, higher than at one step in every seed and within about one Flump of the jump's 228.",
            |c| {
                c.movement.mode = MoveMode::Walk;
                c.movement.speed = 3;
            },
        ),
        preset(
            "ifd-fence",
            "Travel between patches: a fence with a central gap",
            "Baum & Kraft 1998; Minds 2",
            "The 2.10 : 1 patches with vision 10–20 (as in ifd-far-sighted), fenced apart except for a two-site gap at the midline of the 60 × 40 torus (and a matching gap in a second fence at x = 2, closing the route the other way around the torus); rule M's Flumps walk to the best site they see instead of jumping there, so switching patches costs a walk through the gap. Measured (20 seeds, ticks 500–1000): 1.99 times as many Flumps on the richer patch, against 1.97 walking with no fence and 1.88 jumping (ifd-far-sighted). Across the five input ratios s has median 0.88, against 0.90 walking with no fence, so the fence doesn't reduce undermatching. A confound, unmeasured: the fences leave 25 columns on the richer patch's side and 33 on the poorer's, so about 57 % of Flumps start on the poorer side.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.movement.mode = MoveMode::Walk;
                fence(c, 0, false);
            },
        ),
        preset(
            "ifd-fence-far",
            "Travel between patches: the gap moves to the far end",
            "Baum & Kraft 1998; Minds 2",
            "The same fence as ifd-fence, but its gap sits at rows 35–36 instead of 20–21, below both patches (which span rows 10–30), so a Flump switching patches faces a much longer walk to reach the gap. Baum & Kraft found that requiring travel to switch patches slightly reduced undermatching. Measured (20 seeds, ticks 500–1000, across the five input ratios): s has median 0.85 against 0.90 walking with no fence, and is lower in 18 of 20 seeds, so undermatching grows, the opposite of Baum & Kraft's direction, and their claim fails here. At 2.10 : 1 alone there are 1.96 times as many Flumps on the richer patch, against 1.97 walking with no fence and 1.88 jumping; that points the same way, but the walking arms' ratios (1.96–1.99) are untested and too close to tell apart.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.movement.mode = MoveMode::Walk;
                fence(c, 15, false);
            },
        ),
        preset(
            "ifd-wall",
            "Travel between patches: an opaque wall with a central gap",
            "Baum & Kraft 1998; Minds 2",
            "ifd-fence with an opaque wall instead of a fence. An opaque wall: the other patch is visible only through the gap. Baum & Kraft found that a visual barrier had no effect. Measured (20 seeds, ticks 500–1000, across the five input ratios): s has median 0.91 against 0.88 with the fence, and only 10 of 20 seeds are within 0.05 of their fence's s, so no effect is not shown seed by seed (a weak result); the wall, if anything, raises s. At 2.10 : 1 alone there are 1.99 times as many Flumps on the richer patch, as with the fence.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.movement.mode = MoveMode::Walk;
                fence(c, 0, true);
            },
        ),
    ]
}

/// Axtell et al.'s mobility experiment: 100 agents with vision 5–10 on one
/// sugar mountain in the middle of the 50 × 50 torus, moving, eating and
/// then running Axelrod's culture rule (5 features of `traits` traits) with
/// one neighbor, until every two cultures are identical or share nothing.
/// Nobody starves (metabolism 0): Axtell et al.'s agents never die.
fn docking(c: &mut Config, traits: u32) {
    c.population = 100;
    c.vision = URange::new(5, 10);
    c.goods[0].map = Map::Peaks {
        peaks: vec![Peak {
            x: 25,
            y: 25,
            radius: 35.0,
            height: 4.0,
        }],
    };
    c.goods[0].metabolism = URange::new(0, 0);
    c.culture.enabled = true;
    c.culture.rule = CultureKind::Axelrod;
    c.culture.features = 5;
    c.culture.traits = traits;
    c.culture.stop_when_settled = true;
}

/// Minds 1's world (the spec's probe): a 60 × 40 torus with two cone patches
/// of height 4 at (15, 20), radius 10, and (42, 20), radius `second_radius`;
/// sugar grows back 0.25 a tick; 100 Flumps with metabolism 1, endowment 50
/// and vision 1–6. Nominal inputs (sites with capacity ≥ 1, × 0.25): 305
/// against 305, 225, 145, 109 and 69 sites at radius 10, 8.5, 7, 6 and 5
/// (R 1.00, 1.36, 2.10, 2.80, 4.42); about 112 Flumps can be fed at R 2.10.
pub(crate) fn two_patches(c: &mut Config, second_radius: f64) {
    c.width = 60;
    c.height = 40;
    c.population = 100;
    c.goods[0].map = Map::Peaks {
        peaks: vec![
            Peak {
                x: 15,
                y: 20,
                radius: 10.0,
                height: 4.0,
            },
            Peak {
                x: 42,
                y: 20,
                radius: second_radius,
                height: 4.0,
            },
        ],
    };
    c.goods[0].metabolism = URange::new(1, 1);
    c.goods[0].endowment = URange::new(50, 50);
    c.growback.rate = 0.25;
}

/// Minds 2's fences: one-site-wide fences at x = 28 (between the patches)
/// and x = 2 (closing the route around the torus), from top to bottom
/// except a two-site gap at rows 20 + `offset` and 21 + `offset`.
///
/// A confound, unmeasured: the fences split the torus into 25 columns on
/// the richer patch's side (x = 3–27) and 33 on the poorer's, so random
/// placement starts about 57 % of Flumps on the poorer side.
fn fence(c: &mut Config, offset: u32, opaque: bool) {
    let rect = |x, y, height| Wall {
        x,
        y,
        width: 1,
        height,
        opaque,
    };
    c.walls = vec![
        rect(28, 0, 20 + offset),
        rect(28, 22 + offset, 18 - offset),
        rect(2, 0, 20 + offset),
        rect(2, 22 + offset, 18 - offset),
    ];
}

pub fn by_id(id: &str) -> Option<Preset> {
    all().into_iter().find(|p| p.id == id)
}

/// A preset of any model: what the page's presets menu, sweeps and the CLI
/// list. A sugarscape preset's config serializes exactly as its `Preset`'s;
/// the JSON also carries its `title` (see `crate::titles`).
#[derive(Clone, Debug)]
pub struct ModelPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    pub description: &'static str,
    pub config: ModelConfig,
}

impl ModelPreset {
    /// A plain headline of what happens in this preset (`crate::titles`).
    pub fn title(&self) -> &'static str {
        crate::titles::title(self.id)
    }
}

impl Serialize for ModelPreset {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut out = s.serialize_struct("ModelPreset", 6)?;
        out.serialize_field("id", self.id)?;
        out.serialize_field("title", self.title())?;
        out.serialize_field("name", self.name)?;
        out.serialize_field("source", self.source)?;
        out.serialize_field("description", self.description)?;
        out.serialize_field("config", &self.config)?;
        out.end()
    }
}

impl From<Preset> for ModelPreset {
    fn from(p: Preset) -> Self {
        ModelPreset {
            id: p.id,
            name: p.name,
            source: p.source,
            description: p.description,
            config: p.config.into(),
        }
    }
}

/// Every model's presets: the sugarscape's (`all`), then Schelling's, Ring
/// World's, the anasazi's, civil violence's, the tags model's, the spatial
/// games', the ethnocentrism model's, the demographic PD's, the norms
/// model's, relative agreement's and image scoring's.
pub fn catalog() -> Vec<ModelPreset> {
    let mut out: Vec<ModelPreset> = all().into_iter().map(ModelPreset::from).collect();
    out.extend(crate::schelling::presets());
    out.extend(crate::ring::presets());
    out.extend(crate::anasazi::presets());
    out.extend(crate::civil::presets());
    out.extend(crate::tags::presets());
    out.extend(crate::culture::presets());
    out.extend(crate::classes::presets());
    out.extend(crate::opinions::presets());
    out.extend(crate::structure::presets());
    out.extend(crate::spatial::presets());
    out.extend(crate::ethno::presets());
    out.extend(crate::dpd::presets());
    out.extend(crate::norms::presets());
    out.extend(crate::agreement::presets());
    out.extend(crate::image::presets());
    out.extend(crate::farol::presets());
    out.extend(crate::ants::presets());
    out.extend(crate::thresholds::presets());
    out.extend(crate::retirement::presets());
    out.extend(crate::punishment::presets());
    out
}

/// The preset `id` of any model.
pub fn find(id: &str) -> Option<ModelPreset> {
    catalog().into_iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DecisionRule, Idle, Movement};

    #[test]
    fn the_ifd_presets_share_the_two_patch_world() {
        let ids = [
            "ifd-even",
            "ifd-two-to-one",
            "ifd-four-to-one",
            "ifd-far-sighted",
            "ifd-no-starving",
            "ifd-wander",
            "ifd-crowding",
            "ifd-travel",
        ];
        for id in ids {
            let c = by_id(id).unwrap_or_else(|| panic!("{id}")).config;
            assert_eq!((c.width, c.height, c.population), (60, 40, 100), "{id}");
            assert_eq!(c.growback.rate, 0.25, "{id}");
            let Map::Peaks { peaks } = &c.goods[0].map else {
                panic!("{id}: peaks")
            };
            assert_eq!(
                (peaks[0].x, peaks[0].y, peaks[0].radius),
                (15, 20, 10.0),
                "{id}"
            );
            assert_eq!((peaks[1].x, peaks[1].y), (42, 20), "{id}");
        }
        let radius = |id: &str| match &by_id(id).unwrap().config.goods[0].map {
            Map::Peaks { peaks } => peaks[1].radius,
            _ => unreachable!(),
        };
        assert_eq!(
            [
                radius("ifd-even"),
                radius("ifd-two-to-one"),
                radius("ifd-four-to-one")
            ],
            [10.0, 7.0, 5.0]
        );
        let d = |id: &str| by_id(id).unwrap().config.decision;
        assert_eq!(d("ifd-two-to-one").rule, DecisionRule::Book);
        assert_eq!(
            (d("ifd-wander").rule, d("ifd-wander").idle),
            (DecisionRule::Utility, Idle::Wander)
        );
        assert_eq!(d("ifd-crowding").crowding, 1.0);
        assert_eq!(d("ifd-travel").travel, 0.5);
        for id in ["ifd-far-sighted", "ifd-crowding", "ifd-travel"] {
            assert_eq!(
                by_id(id).unwrap().config.vision,
                URange::new(10, 20),
                "{id}"
            );
        }
        assert_eq!(
            by_id("ifd-no-starving").unwrap().config.goods[0].endowment,
            URange::new(100_000, 100_000)
        );
    }

    #[test]
    fn the_walking_presets_walk_and_the_fenced_ones_have_a_gap() {
        for id in [
            "walk-capacity",
            "walk-wealth",
            "walk-seasons",
            "walk-waves",
            "walk-fast",
            "ifd-fence",
            "ifd-fence-far",
            "ifd-wall",
        ] {
            let c = by_id(id).unwrap_or_else(|| panic!("{id}")).config;
            assert_eq!(c.movement.mode, MoveMode::Walk, "{id}");
        }
        assert_eq!(by_id("walk-fast").unwrap().config.movement.speed, 3);
        let base = |id: &str| by_id(id).unwrap().config;
        let mut jumped = base("walk-capacity");
        jumped.movement = Movement::default();
        assert_eq!(jumped, base("ii-2-unit"));
        let gap_rows = |id: &str| {
            let c = base(id);
            let covered = |y: u32| {
                c.walls
                    .iter()
                    .any(|w| w.x == 28 && (w.y..w.y + w.height).contains(&y))
            };
            (0..40).filter(|&y| !covered(y)).collect::<Vec<_>>()
        };
        assert_eq!(gap_rows("ifd-fence"), [20, 21]);
        assert_eq!(gap_rows("ifd-fence-far"), [35, 36]);
        assert!(base("ifd-fence").walls.iter().all(|w| !w.opaque));
        assert!(base("ifd-wall").walls.iter().all(|w| w.opaque));
        assert_eq!(
            base("ifd-fence").walls.len(),
            4,
            "x = 28 and x = 2, above and below the gap"
        );
        assert_eq!(base("ifd-fence").vision, URange::new(10, 20));
    }

    #[test]
    fn every_preset_has_a_plain_title_and_serializes_it() {
        let catalog = catalog();
        assert_eq!(
            crate::titles::TITLES.len(),
            catalog.len(),
            "one title per preset"
        );
        for p in &catalog {
            let t = p.title();
            assert!(!t.is_empty(), "{} has no title", p.id);
            // A title says what happens, not where it is from or which rules it runs.
            for jargon in ["Animation", "Fig.", "Figure", "Table "] {
                assert!(!t.starts_with(jargon), "{}: {t}", p.id);
            }
            assert!(!t.contains("({"), "{}: rule notation in {t}", p.id);
            let numbered_run = t
                .strip_prefix("Run ")
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()));
            assert!(!numbered_run, "{}: {t}", p.id);
            let json = serde_json::to_value(p).unwrap();
            assert_eq!(json["title"], t, "{}", p.id);
            assert_eq!(
                (json["id"].as_str(), json["name"].as_str()),
                (Some(p.id), Some(p.name))
            );
        }
        for (id, _) in crate::titles::TITLES {
            assert!(
                catalog.iter().any(|p| p.id == id),
                "a title for no preset: {id}"
            );
        }
    }
    use crate::world::World;

    #[test]
    fn indecomposability_presets_differ_only_in_trade() {
        let no_trade = by_id("vi-2-no-trade").unwrap().config;
        let mut trade = by_id("vi-3-trade").unwrap().config;
        assert!(!no_trade.trade.enabled && trade.trade.enabled);
        assert_eq!(no_trade.population, 500);
        assert_eq!(no_trade.vision, URange::new(1, 10), "Chapter IV's traits");
        assert_eq!(no_trade.goods.len(), 2, "sugar and spice");
        for g in &no_trade.goods {
            assert_eq!(
                (g.metabolism, g.endowment),
                (URange::new(1, 5), URange::new(25, 50))
            );
        }
        assert_eq!(
            no_trade.goods[1].map,
            by_id("iv-1-spice").unwrap().config.goods[1].map
        );
        assert!(no_trade.sex.enabled && no_trade.lifespan.enabled);
        assert_eq!(no_trade.lifespan.max_age, URange::new(60, 100));
        assert!(!no_trade.culture.enabled && !no_trade.credit.enabled && !no_trade.disease.enabled);
        trade.trade.enabled = false;
        assert_eq!(trade, no_trade);
    }

    #[test]
    fn every_preset_is_valid_and_runs() {
        let presets = all();
        assert_eq!(presets.len(), 47);
        for p in presets {
            p.config
                .validate()
                .unwrap_or_else(|e| panic!("{}: {e:?}", p.id));
            let mut w = World::new(p.config.clone(), 1).unwrap();
            w.run(20);
        }
    }

    #[test]
    fn the_catalog_lists_every_sugarscape_preset_first_unchanged() {
        let catalog = catalog();
        let sugarscape = all();
        for (p, q) in sugarscape.iter().zip(&catalog) {
            assert_eq!(p.id, q.id);
            assert_eq!(q.config.sugarscape(), Some(&p.config));
            // The catalog adds the menu's title; everything else is unchanged.
            let mut listed = serde_json::to_value(q).unwrap();
            let title = listed.as_object_mut().unwrap().remove("title").unwrap();
            assert_eq!(title, q.title());
            assert_eq!(serde_json::to_value(p).unwrap(), listed);
        }
        assert_eq!(
            find("iv-3-trade").unwrap().config,
            by_id("iv-3-trade").unwrap().config.into()
        );
        assert!(find("nope").is_none());
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
            e.goods.len() == 2
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
            (
                c.pollution.pollutants[0].production[0],
                c.pollution.pollutants[0].consumption[0]
            ),
            (0.0, 0.0)
        );
    }

    #[test]
    fn n_goods_presets_have_their_goods_and_pollutants() {
        let t = by_id("n-3-trade").unwrap().config;
        let names: Vec<&str> = t.goods.iter().map(|g| g.name.as_str()).collect();
        assert_eq!(names, ["sugar", "spice", "salt"]);
        assert_eq!(
            t.goods[2].map,
            Map::TwoPeaks {
                transform: Transform::Rotate90
            }
        );
        assert!(t.trade.enabled && t.sex.enabled && t.lifespan.enabled);
        let p = by_id("n-4-peaks").unwrap().config;
        assert_eq!(p.goods.len(), 4);
        assert!(p
            .goods
            .iter()
            .all(|g| matches!(&g.map, Map::Peaks { peaks } if peaks.len() == 1)));
        assert!(p.trade.enabled);
        let q = by_id("n-2-pollutants").unwrap().config;
        let names: Vec<&str> = q
            .pollution
            .pollutants
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names, ["smoke", "runoff"]);
        assert_eq!(q.pollution.pollutants[0].devalues, vec![true, false]);
        assert_eq!(q.pollution.pollutants[1].devalues, vec![false, true]);
        assert!(q.pollution.enabled && q.diffusion.enabled);
    }

    #[test]
    fn the_docking_presets_run_axelrods_rule_on_one_mountain_and_stop_when_settled() {
        use crate::config::CultureKind;
        for (id, q) in [("dock-mobility-15", 15), ("dock-mobility-30", 30)] {
            let c = by_id(id).unwrap().config;
            assert_eq!((c.population, c.vision), (100, URange::new(5, 10)), "{id}");
            assert!(
                c.culture.enabled
                    && c.culture.rule == CultureKind::Axelrod
                    && c.culture.stop_when_settled
            );
            assert_eq!((c.culture.features, c.culture.traits), (5, q));
            assert_eq!(c.goods[0].metabolism, URange::new(0, 0));
            assert!(matches!(&c.goods[0].map, Map::Peaks { peaks } if peaks.len() == 1));
            let w = crate::world::World::new(c, 1).unwrap();
            assert!(w
                .agents()
                .all(|a| a.culture.len() == 5 && a.culture.iter().all(|&t| u32::from(t) < q)));
        }
    }

    #[test]
    fn a_settled_sugarscape_stops() {
        let mut c = by_id("dock-mobility-15").unwrap().config;
        c.population = 2;
        let mut w = crate::world::World::new(c, 3).unwrap();
        let ids: Vec<_> = w.agents().map(|a| a.id).collect();
        w.agent_mut(ids[0]).unwrap().culture = vec![1, 1, 1, 1, 1];
        w.agent_mut(ids[1]).unwrap().culture = vec![2, 2, 2, 2, 2];
        w.run(1);
        let s = w.stats.latest().unwrap().axelrod.unwrap();
        assert_eq!((s.distinct_cultures, s.settled), (2, true));
        assert!(w.is_finished());
        let tick = w.tick;
        w.run(10);
        assert_eq!(w.tick, tick, "a finished world does not step");
        let flip = crate::world::World::new(Config::default(), 1).unwrap();
        assert!(flip.stats.latest().unwrap().axelrod.is_none() && !flip.is_finished());
    }

    #[test]
    fn only_axelrods_rule_at_work_among_two_or_more_agents_can_settle() {
        // Rule K off: nothing is settling, so nothing stops.
        let mut off = by_id("dock-mobility-15").unwrap().config;
        off.population = 2;
        off.culture.enabled = false;
        let mut w = crate::world::World::new(off, 3).unwrap();
        let ids: Vec<_> = w.agents().map(|a| a.id).collect();
        w.agent_mut(ids[0]).unwrap().culture = vec![1, 1, 1, 1, 1];
        w.agent_mut(ids[1]).unwrap().culture = vec![2, 2, 2, 2, 2];
        w.run(3);
        assert_eq!(w.tick, 3);
        assert!(!w.is_finished());
        // An empty world has no cultures to settle.
        let mut empty = by_id("dock-mobility-15").unwrap().config;
        empty.population = 0;
        let mut e = crate::world::World::new(empty, 1).unwrap();
        e.run(3);
        assert_eq!((e.tick, e.is_finished()), (3, false));
    }

    #[test]
    fn three_tribes_is_the_culture_preset_with_the_books_groups() {
        let three = by_id("iii-6-three-tribes").unwrap().config;
        let mut culture = by_id("iii-6-culture").unwrap().config;
        culture.culture.groups = three_tribes(11);
        assert_eq!(three, culture);
        let spans: Vec<(&str, u32, u32)> = three
            .culture
            .groups
            .iter()
            .map(|g| (g.name.as_str(), g.zeros.min, g.zeros.max))
            .collect();
        assert_eq!(spans, [("Blue", 0, 3), ("Green", 4, 7), ("Red", 8, 11)]);
    }

    #[test]
    fn waves_start_from_the_books_full_southwest_block() {
        // Animation II-6's first frame: a 20×20 block in the bottom-left
        // corner, "in all other respects … exactly as in animation II-2"
        // (400 agents, so the block is full), with vision up to 10.
        let c = by_id("ii-6-waves").unwrap().config;
        assert_eq!(
            c.placement,
            Placement::Block {
                x: 0,
                y: 30,
                width: 20,
                height: 20
            }
        );
        assert_eq!((c.population, c.vision), (400, URange::new(1, 10)));
        let w = World::new(c, 1).unwrap();
        assert!(w.agents().all(|a| a.pos.x < 20 && a.pos.y >= 30));
        assert_eq!(w.population(), 400);
    }
}
