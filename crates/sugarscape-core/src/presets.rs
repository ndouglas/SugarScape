//! The book's named rule systems. Parameters come from the text of
//! Chapters II–III; `source` cites the animation each reproduces.

use serde::Serialize;

use crate::config::{
    three_tribes, Config, Good, Map, Outbreak, Peak, Placement, Pollutant, Pollution,
    ScheduledChange, Transform, URange, SPICE_COLOR,
};

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
    // VI-3 matches the book's curve (a dip by t ~ 100, recovery to 1.7-2.0x
    // the initial 500, minima near 700); VI-2 does the same instead of
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
                schedule(c, 150, "pollution.pollutants.0.production.0", serde_json::json!(0.0));
                schedule(c, 150, "pollution.pollutants.0.consumption.0", serde_json::json!(0.0));
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
            "Every rule at once: spice, sex, finite lives, inheritance, culture, trade, credit and disease, with new diseases arriving by outbreak at t = 150, 400 and 650; disease flares after each outbreak and tends to die out again before the next one. The book's eighteen views, in its order: (1) Agents → Disease colors; (2) the Neighbor network overlay; (3) Charts → Wealth distribution (sugar); (4) Charts → Goods → Wealth distribution · spice; (5) Charts → Goods → Lorenz curve and Gini coefficient (total wealth); (6) Charts → Population; (7) Charts → Age histogram; (8) the Family network overlay (with Agents → Lineage for the book's colors); (9) Charts → Cultural tags; (10) the Friends network overlay; (11) and (12) Charts → Economy → Trade price (its mean and ± SD band); (13) Charts → Economy → Trade volume; (14) the Trade network overlay; (15) the Credit network overlay; (16) the Credit tab's hierarchy; (17) Charts → Disease; (18) the Disease network overlay.",
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
            "Everything as in VI-2, with trade on. This reproduces the book's curve: the population dips to about 100–175 by t = 100–150, recovers to 1.7–2.0 times its initial 500, then fluctuates with minima near 700. (VI-2 without trade does the same here, unlike the book.)",
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
            "Four goods, each on one peak near a different corner of the torus: agents must travel or trade to hold all four.",
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
        assert_eq!(presets.len(), 29);
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
}
