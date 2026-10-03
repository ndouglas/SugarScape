//! The book's named rule systems. Parameters come from the text of
//! Chapters II–III; `source` cites the animation each reproduces.

use serde::Serialize;

use crate::config::{
    three_tribes, CachingRule, Config, CultureKind, DecisionRule, DigBelow, Good, Idle, Lab,
    LabProtocol, Loot, Map, MemoryPrior, MoveMode, Outbreak, Peak, Placement, Pollutant, Pollution,
    ScheduledChange, Scrounge, SeasonMode, Transform, URange, Wall, SPICE_COLOR,
};
use crate::minds::caching::lab::{rig_config, LabParams};
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

/// Minds 3: `span` ticks of memory for a `share` of newborns, under
/// `project` belief (the engine's default).
fn memory(c: &mut Config, span: u32, share: f64) {
    c.memory.span = span;
    c.memory.share = share;
}

/// Minds 4: the marginal value theorem's world. Nine peaks (radius 3,
/// height 4: 25 sites of capacity ≥ 1 each) at (10 + 20i, 10 + 20j) on a
/// 60 × 60 torus; 3 agents of metabolism 1, endowment 50 and vision 1–6,
/// walking, every one remembering (span 1 000, `project`) and starting with
/// the whole map (`memory.prior: map`); growback 0.02. The balance was
/// measured before this was recorded (Minds 4, Task 7): the spec's
/// growback 0.05 gives a patch 25 × 0.05 = 1.25 sugar a tick, more than one
/// forager eats, so a patch never runs out; the plan's mechanical rule set
/// growback so a patch takes in 0.5 a tick, and the population so all nine
/// patches' 4.5 a tick is 1.5 times the need (3 × 1).
fn mvt_world(c: &mut Config, rule: DecisionRule) {
    c.width = 60;
    c.height = 60;
    c.population = 3;
    c.goods[0].map = Map::Peaks {
        peaks: (0..3u32)
            .flat_map(|i| {
                (0..3u32).map(move |j| Peak {
                    x: 10 + 20 * i,
                    y: 10 + 20 * j,
                    radius: 3.0,
                    height: 4.0,
                })
            })
            .collect(),
    };
    c.goods[0].metabolism = URange::new(1, 1);
    c.goods[0].endowment = URange::new(50, 50);
    c.vision = URange::new(1, 6);
    c.growback.rate = 0.02;
    c.movement.mode = MoveMode::Walk;
    c.decision.rule = rule;
    memory(c, 1000, 1.0);
    c.memory.prior = MemoryPrior::Map;
}

/// Minds 5: the winter world. walk-capacity's landscape (the default
/// sugarscape, rule M's agents walking, mem-open's memory: half of them
/// remember for 100 ticks) with a winter everywhere at once
/// (`seasons.mode: global`, γ = 100, β = 32), 175 agents of metabolism 1,
/// a carrying limit of 50 (half a winter's need: without a limit, holdings
/// are an unlimited cache), `goap.horizon` 20 (R = 20 for every rule) and
/// caching `rule` (`mixed` deals the four rules round-robin instead).
///
/// The balance was measured before this was recorded (Task 9; 5 seeds):
/// (a) the first summer's surplus, measured with no caching and no carrying
/// limit, is 151.0 per agent against a winter need of 100 (1.51 ≥ 1.5),
/// and (b) winter regrowth (sites with
/// capacity × α γ / β = 6 466) is 0.39 of the population's winter need
/// (165.8 alive × 100). walk-capacity's own traits (metabolism 1–4) can't
/// meet (a) at any population, β or γ tried (a walker harvests at most
/// about 3.5 a tick), hence metabolism 1. Survival through the first winter
/// (alive at 200 ÷ alive at 100): none 48.5 %, even 74.9 %, compensate
/// 73.8 %, plan 88.2 %. At the default horizon (R = 10) even did worse than
/// none (45.2 %, compensate 44.6 %, plan 71.5 %): under rule M a hungry
/// agent heads for its biggest cache, not its nearest, and with a thin
/// reserve starves on the way (it goes hungry only below R / 2).
///
/// Minds 6 turns theft on in this world (`theft_winter`) at `theft.find`
/// 0.25, chosen by measurement before any preset was recorded (Task 5;
/// `even` hoarders only, 5 seeds; the daily pilferage rate is the mean of
/// `pilferage_rate` over ticks 101–200). The rate is about 0.09 × `find`:
/// 0.12 %, 0.24 %, 0.55 %, 1.02 % and 1.83 % at 0.01, 0.02, 0.05, 0.1 and
/// 0.2, none of them inside Vander Wall and Jenkins's 2–30 % a day, so the
/// list was extended mechanically (the smallest of 0.25, 0.3, 0.4 and 0.5
/// at ≥ 2 %): 0.25 gives 2.21 %. Even at `find` 1.0 the rate is only
/// 7.7 %, below the literature's median of 9 %: thieves who only stumble on
/// caches where they happen to stand can't reach it (active search, by
/// watching others cache, comes in P2). Theft pools stores: first-winter
/// survival rises from 74.9 % with no theft to 91.7 % at 0.25 (and 98.3 %
/// at 1.0), since a thief is likely a hungry agent near someone else's
/// surplus.
///
/// Two windows, two measurements. The figures above are Task 5's probe (5
/// seeds, the winter only: ticks 101–200). The survey (20 seeds, ticks
/// 0–200, a summer and the first winter) gives a pilferage rate of 2.31 %
/// at 0.25 and 6.96 % at 1 (10.9 % of the cached sugar), with first-winter
/// survival 90.4 % and 98.5 %. The probe's summer rate at 1 was 5.77 %
/// against 7.67 % in the winter, so the whole-run mean sits between them.
/// Both are below the median of 9 %. `find` is a free parameter; 0.25 is an
/// anchor, not a fit to the field.
fn winter_world(c: &mut Config, rule: CachingRule, mixed: bool) {
    c.movement.mode = MoveMode::Walk;
    memory(c, 100, 0.5);
    c.population = 175;
    c.goods[0].metabolism = URange::new(1, 1);
    c.seasons.enabled = true;
    c.seasons.mode = SeasonMode::Global;
    c.seasons.period = 100;
    c.seasons.winter_divisor = 32;
    c.caching.capacity = 50;
    c.caching.rule = rule;
    c.caching.mixed = mixed;
    c.goap.horizon = 20;
}

/// Minds 6: the pilfering rate every theft preset uses (see `winter_world`).
const THEFT_FIND: f64 = 0.25;

/// Minds 6: the winter world (`winter_world`, `even` hoarders) with theft on
/// at `THEFT_FIND` and a `cheaters` share of agents (by id, no draw) who
/// never cache and pilfer whatever they find. Loot is kept, owners remember
/// their caches and burying is free (the defaults).
fn theft_winter(c: &mut Config, cheaters: f64) {
    winter_world(c, CachingRule::Even, false);
    c.theft.find = THEFT_FIND;
    c.theft.cheaters = cheaters;
}

/// Minds 8: theft-winter's world (`theft_winter`) with a `cheaters` share,
/// stumbling at `find` and watching on, a `watchers` share of founders
/// (dealt by id, as cheaters are) watching. Span 7 and `raid_when: always`
/// (the defaults).
fn watch_winter(c: &mut Config, cheaters: f64, find: f64, watchers: f64) {
    theft_winter(c, cheaters);
    c.theft.find = find;
    c.watching.on = true;
    c.watching.watchers = watchers;
}

/// Minds 6: the arena. `n` agents (2, 4 or 8) shut in a k × k room, with
/// k = 4, 6 and 8: a (k + 2) × (k + 2) torus with opaque walls along rows 0
/// and k + 1 and columns 0 and k + 1, so the room is the centered k × k
/// block, closed on all four sides, the group shares one area and Andersson
/// and Krebs's n is exact. (Until 2026-09-30 it was a (k + 1)-torus walled
/// along row 0 and column 0 only: closed on the torus, but drawn with the
/// room against the bottom and right edges. The numbers below were
/// re-measured for the four walls.) The
/// sugar is flat: capacity 4 and growback 0.3 a site, 8 sites an agent at
/// n = 2 (16 sites) and n = 8 (64). No square gives n = 4 exactly 8 sites
/// an agent (6 × 6 is 9), so there capacity and growback are × 8/9 (32/9
/// and 0.2667): standing sugar (32) and regrowth (2.4 a tick in summer,
/// 0.075 in winter) per agent are the same at every n. Everything else is
/// the winter world's (walking, one good, metabolism 1, a winter everywhere
/// at once with γ 100 and β 32, carrying limit 50, horizon 20, half the
/// agents remembering for 100 ticks, `even` hoarders), except vision: 1 to
/// half the room's side (1–2, 1–3 and 1–4 at n = 2, 4 and 8), so every
/// room is seen alike relative to its size and sight isn't tied to n (half
/// the room's side is within each torus's cap of half the grid). Theft is
/// on at `THEFT_FIND` with half the agents cheaters.
///
/// The balance was measured before this was recorded (Task 5), and again
/// for the four walls (2026-09-30; 100 seeds per n, first-winter survival =
/// alive at 200 ÷ alive at 100; nobody died before 100). Without theft,
/// with nobody caching 0.0 %, 0.2 % and 2.0 % survive at n = 2, 4 and 8,
/// against 99.5 %, 96.0 % and 92.8 % of `even` hoarders. At `THEFT_FIND`
/// with half cheaters, the daily pilferage rate over ticks 101–200 (the
/// mean of `pilferage_rate`) is 1.54 %, 1.92 % and 2.34 % at n = 2, 4 and 8
/// (1.34 %, 1.74 % and 2.16 % with no cheaters), and 100 %, 99.8 % and
/// 97.8 % of the agents alive at 100 survive the winter.
fn theft_arena(c: &mut Config, n: u32) {
    winter_world(c, CachingRule::Even, false);
    let (side, scale) = match n {
        2 => (4, 1.0),
        4 => (6, 8.0 / 9.0),
        8 => (8, 1.0),
        _ => panic!("the arena holds 2, 4 or 8 agents, not {n}"),
    };
    c.width = side + 2;
    c.height = side + 2;
    c.population = n;
    c.vision = URange::new(1, side / 2);
    c.goods[0].map = Map::Flat {
        capacity: 4.0 * scale,
    };
    c.growback.rate = 0.3 * scale;
    let wall = |x, y, width, height| Wall {
        x,
        y,
        width,
        height,
        opaque: true,
    };
    c.walls = vec![
        wall(0, 0, side + 2, 1),
        wall(0, side + 1, side + 2, 1),
        wall(0, 1, 1, side),
        wall(side + 1, 1, 1, side),
    ];
    c.theft.find = THEFT_FIND;
    c.theft.cheaters = 0.5;
}

/// Minds 5: the central-place world. A 60 × 30 torus with a column of
/// five patches (peaks of radius 3 and height 4 at x = 45, y = 3, 9, 15,
/// 21, 27) growing back 0.25 a tick (`instant` for linear loading: every
/// site refills at once, so a load grows in step with the ticks spent
/// gathering it). 5 agents of metabolism 1, endowment 60 and vision 1–6
/// have their homes at random rows of the column x = `home_x`, so the
/// patches are 45 − `home_x` columns east of home (and at most 3 rows off a
/// patch center). They walk, know the whole map (memory span 1 000, `prior:
/// map`), carry loads of at most 320 (the limit caps a trip's load, not
/// load plus provisions) and forage in round trips under the marginal-value
/// rule (`mvt.alpha` 0.05). Near is `home_x` 37 (8 columns), far is 25 (20
/// columns).
///
/// The limit was raised (80, 120, 160, 240, 320) until ρ or an empty site,
/// not a full load, ended at least half the trips at both distances (seeds
/// 1–3, ticks 1–1000): at 320, full ends 35 % near, 49 % far and 33 %
/// linear; the guard for food to get home ends under 1 %. Mean gross load
/// (gathered on the trip) and delivered load (brought home, after what was
/// eaten of it on the way): near 127.7 and 84.0, far 178.0 and 109.2,
/// linear 169.8 and 131.5; every agent alive at tick 1000. Far loads are
/// larger than near ones, the direction central-place theory predicts. A
/// first try with one patch and the homes in a 5-row block starved: the
/// agents all made for the one best site, and an endowment above the
/// carrying limit left no room to gather.
fn central_world(c: &mut Config, home_x: u32, instant: bool) {
    c.width = 60;
    c.height = 30;
    c.population = 5;
    c.placement = Placement::Block {
        x: home_x,
        y: 0,
        width: 1,
        height: 30,
    };
    c.goods[0].map = Map::Peaks {
        peaks: (0..5u32)
            .map(|j| Peak {
                x: 45,
                y: 3 + 6 * j,
                radius: 3.0,
                height: 4.0,
            })
            .collect(),
    };
    c.goods[0].metabolism = URange::new(1, 1);
    c.goods[0].endowment = URange::new(60, 60);
    c.vision = URange::new(1, 6);
    c.growback.rate = 0.25;
    c.growback.instant = instant;
    c.movement.mode = MoveMode::Walk;
    c.decision.rule = DecisionRule::Mvt;
    memory(c, 1000, 1.0);
    c.memory.prior = MemoryPrior::Map;
    c.caching.capacity = 320;
    c.central.enabled = true;
}

/// Minds 5: a lab (`lab::rig_config`) with `n` agents under
/// `caching.mixed`: agents 1, 2, 3, 4, 5, … follow none, even, compensate,
/// plan, none, … (the config's own rule, `even`, is ignored).
fn mixed_lab(c: &mut Config, lab: Lab, n: u32) {
    *c = rig_config(lab, CachingRule::Even, LabParams::default(), n);
    c.caching.mixed = true;
}

/// Minds 3: truffle spots covering `share` of non-wall sites, worth `value`
/// sugar and regrowing after `regrow` ticks (`seed` stays the default, 1).
fn truffles(c: &mut Config, share: f64, value: f64, regrow: u32) {
    c.truffles.share = share;
    c.truffles.value = value;
    c.truffles.regrow = regrow;
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
            "Axtell, Axelrod, Epstein and Cohen's mobility experiment (1996): 100 agents with vision 5–10 on a \"single (Gaussian) sugar mountain\" in the middle of the 50 × 50 torus move to the richest site they see, eat, then run Axelrod's culture rule (5 features of 15 traits) with one random neighbor; nobody starves (our reading: they say only \"a standard version of the Sugarscape\"; with metabolism 1–4, 1.33 cultures over 12 seeds). The run stops once every two agents' cultures are identical or share nothing. They report 1.1 ± 0.3 cultures over 10 runs. The mountain's width and height are not given; here it is as wide as the board (σ = 25) at the Sugarscape's usual height of 4. Measured (20 seeds): 1.15 ± 0.37 cultures (17 runs one, 3 two), every run stopping (median at tick 1,800). The landscape decides it: mountains broad or tall enough reproduce their number; narrow, low ones leave many cultures (sweep dock-mountain).",
            |c| docking(c, 15),
        ),
        preset(
            "dock-mobility-30",
            "Docking: Axelrod's culture on the move, 30 traits",
            "Axtell, Axelrod, Epstein & Cohen 1996, §4.3.1",
            "The mobility experiment with 30 traits per feature. Axtell et al. 1996: 2.2 ± 1.2 cultures, more than with 15 traits. Measured (20 seeds): 2.25 ± 0.85, every run stopping (median at tick 2,300).",
            |c| docking(c, 30),
        ),
        preset(
            "ifd-even",
            "Ideal free distribution: equal patches",
            "Fretwell & Lucas 1969; Minds 1",
            "Two cone-shaped sugar patches of the same size (305 sites each, input 0.25 a tick per site) on a 60 × 40 torus, and 100 agents of metabolism 1 and vision 1–6. The ideal free distribution predicts an even split. Measured (20 seeds, tick 1000): 0.98 as many agents on the first patch as the second, with 45 of 100 alive. Measured: the on-patch counts are the same when nobody starves, so the dead are agents who never found sugar.",
            |c| two_patches(c, 10.0),
        ),
        preset(
            "ifd-two-to-one",
            "Ideal free distribution: 2.1 : 1",
            "Parker 1978; Milinski 1979; Minds 1",
            "The second patch has radius 7 (145 sites against 305: input ratio 2.10, near Milinski's 2 : 1). Input matching predicts 2.10 times as many agents on the richer patch. Measured (20 seeds, tick 1000): 1.71 times. Fitted per seed across input ratios 1, 1.36, 2.10, 2.80 and 4.42, the matching exponent s has median 0.72 (s = 1 is matching). Parker's input matching fails (4 of 20 seeds within 0.9–1.1); undermatching, which Kennedy & Gray 1993 report for most animal experiments, holds. The median is close to the catchment prediction of 0.71, which counts the sites from which each patch is in sight. But the seeds scatter widely (IQR 0.59–0.92), and only 6 of 20 land within 0.1 of it, so that claim fails as judged.",
            |c| two_patches(c, 7.0),
        ),
        preset(
            "ifd-four-to-one",
            "Ideal free distribution: 4.4 : 1",
            "Parker 1978; Minds 1",
            "The second patch has radius 5 (69 sites: input ratio 4.42). Input matching predicts 4.42 times as many agents on the richer patch. Measured (20 seeds, tick 1000): 2.96 times.",
            |c| two_patches(c, 5.0),
        ),
        preset(
            "ifd-far-sighted",
            "Ideal free distribution: vision 10–20",
            "Kennedy & Gray 1993; Minds 1",
            "The 2.10 : 1 patches with vision 10–20, far enough to see across the 7-site gap between the patches. Measured (20 seeds, tick 1000): 1.85 times as many agents on the richer patch; s has median 0.90 across the five input ratios, close to matching but still under it. Every seed undermatches, and only 8 of 20 are within 0.9–1.1, so Parker's input matching narrowly fails here too.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
            },
        ),
        preset(
            "ifd-no-starving",
            "Ideal free distribution: nobody starves",
            "Fretwell & Lucas 1969; Minds 1",
            "The 2.10 : 1 patches with an endowment of 100 000, so nobody starves within the run. Rule M keeps an agent in place when nothing it sees is better, so one that starts out of sight of sugar never moves. Measured (20 seeds, tick 1000): a median 66 of 100 off both patches (over half in all 20 seeds), so the 'free' of the ideal free distribution fails. The on-patch counts are the same as with starvation, and s is identical seed by seed in all 20 seeds, so the survival claim holds: the agents who die are the ones who never find sugar.",
            |c| {
                two_patches(c, 7.0);
                c.goods[0].endowment = URange::new(100_000, 100_000);
            },
        ),
        preset(
            "ifd-wander",
            "Utility mind: wander when nothing scores",
            "Minds 1",
            "The no-starving world under the utility mind with idle wander: an agent that sees no sugar moves to a random free site in sight instead of staying put. It tests the 'free' of the ideal free distribution apart from the 'ideal'. Measured (20 seeds, tick 1000): under 1 of 100 off both patches, so wandering does make the agents free. But only 1.43 times as many are on the richer patch (1.71 under stay), and s falls from 0.72 to 0.40, away from matching, so the claim that wandering moves s toward 1 fails. Since nobody starves, the wanderers overfill the poorer patch (41 agents on its 36 sugar a tick), so the split likely follows where they arrive, not the inputs.",
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
            "The 2.10 : 1 patches with vision 10–20 (far enough to see both patches, as in ifd-far-sighted) under the utility mind with crowding m = 1: a site's welfare is divided by (1 + n), n the agents next to it. Sutherland's interference model predicts input matching at m = 1; here the interference is local. We expected local crowding only to push agents apart and lower s. Measured (20 seeds, tick 1000): 1.93 times as many agents on the richer patch (1.85 without crowding). Across the five input ratios s has median 0.95 against 0.90 at m = 0, with 18 of 20 seeds within 0.9–1.1. So Sutherland's matching holds, and crowding raises s (one-sided Mann–Whitney p = 0.0003).",
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
            "The 2.10 : 1 patches with vision 10–20 (far enough to see both patches, as in ifd-far-sighted) under the utility mind with travel k = 0.5: a site's welfare is divided by (1 + 0.5·d), d its distance. Baum & Kraft found that requiring travel to switch patches slightly reduced undermatching; here travel is a preference for nearby sugar under rule M's one-tick jump, not a cost of switching. Measured (20 seeds, tick 1000): 1.52 times as many agents on the richer patch (1.85 without travel). Across the five input ratios s has median 0.73 against 0.90 at k = 0, so undermatching grows, the opposite of Baum & Kraft's direction, and their claim fails here.",
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
            "Rule M's agents jump to the best site in sight; these walk there one step a tick along an A* path. Measured (20 seeds, mean population over ticks 300–500): 181 against 228 under the jump, so the book's carrying capacity of about 224 fails under walking (no seed within 214–234), and walking lowers it in every seed, by a median 47 agents. In the walk-speed and walk-vision sweeps (an observation, not a judged claim), capacity tracks how far an agent gets in a tick, roughly min(speed, vision): walking at vision 1–6 (181.5) is about jumping at vision 1 (182.7), and walking at speed 3 (212.8) and 6 (225.2) is near jumping at vision 1–3 (205.5) and 1–6 (228.5). Walking faster brings the population back (see walk-fast and the walk-speed sweep).",
            |c| c.movement.mode = MoveMode::Walk,
        ),
        preset(
            "walk-wealth",
            "Walking: wealth distribution",
            "Epstein & Axtell II-5; Minds 2",
            "Finite lifetimes with replacement, as in ii-5-wealth, but rule M's agents walk to the best site they see instead of jumping there in one tick. Measured (20 seeds, tick 500): wealth is still right-skewed in every seed, so the book's skewed distribution (Animation II-5) doesn't need the jump. The skewness has median 1.26 against 1.27 under the jump, and the Gini 0.46 against 0.48.",
            |c| {
                wealth(c);
                c.movement.mode = MoveMode::Walk;
            },
        ),
        preset(
            "walk-seasons",
            "Walking: seasons",
            "Epstein & Axtell II-7; Minds 2",
            "Seasons flipping every 50 ticks, as in ii-7-seasons, but rule M's agents walk to the best site they see instead of jumping there in one tick. Measured (20 seeds, agents alive over ticks 100–300): a median 55 % change hemisphere at least twice, against 83 % under the jump, so migration with the seasons (Animation II-7) survives walking, but fewer agents migrate. Likely cause: crossing to the other hemisphere takes a walker many ticks, so fewer finish the crossing before the season or their target changes.",
            |c| {
                enable_seasons(c);
                c.movement.mode = MoveMode::Walk;
            },
        ),
        preset(
            "walk-waves",
            "Walking: diagonal waves",
            "Epstein & Axtell II-6; Minds 2",
            "The book's waves travel northeast from the southwest block; under rule M's jump they don't. Measured (20 seeds, tick 100): a median 0.6 % of agents are farther than 25 sites from the block's center, against 0.8 % under the jump; no seed comes near a quarter (in planning, the block settled on the near mountain under both). So walking isn't the missing mechanism behind the waves (Animation II-6).",
            |c| {
                waves(c);
                c.movement.mode = MoveMode::Walk;
            },
        ),
        preset(
            "walk-fast",
            "Walking: speed 3",
            "Minds 2",
            "ii-2-unit's unit growback and carrying capacity, but agents walk three steps a tick instead of one along their A* path. Measured (20 seeds, mean population over ticks 300–500): 214, between walking at one step a tick (181) and the jump (228). The capacity under walk rises toward the jump's as speed rises: at ten steps a tick it is 227, higher than at one step in every seed and within about one agent of the jump's 228.",
            |c| {
                c.movement.mode = MoveMode::Walk;
                c.movement.speed = 3;
            },
        ),
        preset(
            "ifd-fence",
            "Travel between patches: a fence with a central gap",
            "Baum & Kraft 1998; Minds 2",
            "The 2.10 : 1 patches with vision 10–20 (as in ifd-far-sighted), fenced apart except for a two-site gap at the midline of the 60 × 40 torus (and a matching gap in a second fence at x = 2, closing the route the other way around the torus); rule M's agents walk to the best site they see instead of jumping there, so switching patches costs a walk through the gap. Measured (20 seeds, ticks 500–1000): 1.99 times as many agents on the richer patch, against 1.97 walking with no fence and 1.88 jumping (ifd-far-sighted). Across the five input ratios s has median 0.88, against 0.90 walking with no fence, so the fence doesn't reduce undermatching. A confound, unmeasured: the fences leave 25 columns on the richer patch's side and 33 on the poorer's, so about 57 % of agents start on the poorer side.",
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
            "The same fence as ifd-fence, but its gap sits at rows 35–36 instead of 20–21, below both patches (which span rows 10–30), so an agent switching patches faces a much longer walk to reach the gap. Baum & Kraft found that requiring travel to switch patches slightly reduced undermatching. Measured (20 seeds, ticks 500–1000, across the five input ratios): s has median 0.85 against 0.90 walking with no fence, and is lower in 18 of 20 seeds, so undermatching grows, the opposite of Baum & Kraft's direction, and their claim fails here. At 2.10 : 1 alone there are 1.96 times as many agents on the richer patch, against 1.97 walking with no fence and 1.88 jumping; that points the same way, but the walking arms' ratios (1.96–1.99) are untested and too close to tell apart.",
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
            "ifd-fence with an opaque wall instead of a fence. An opaque wall: the other patch is visible only through the gap. Baum & Kraft found that a visual barrier had no effect. Measured (20 seeds, ticks 500–1000, across the five input ratios): s has median 0.91 against 0.88 with the fence, and only 10 of 20 seeds are within 0.05 of their fence's s, so no effect is not shown seed by seed (a weak result); the wall, if anything, raises s. At 2.10 : 1 alone there are 1.99 times as many agents on the richer patch, as with the fence.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.movement.mode = MoveMode::Walk;
                fence(c, 0, true);
            },
        ),
        preset(
            "mem-open",
            "Memory: open sugarscape",
            "Minds 3",
            "walk-capacity's world (rule M's agents walking to the best site in sight) with memory: a rememberer that leaves a mountain can walk back to it for 100 ticks after it drops out of sight. The mountains stay in sight of most sites and refill fast, so a rememberer should gain little. Measured (20 seeds, ticks 200–500): rememberers lose instead. They hold a median 324 sugar against the others' 438 (an advantage of −113), within 10 % of the others in only 2 of 20 seeds. Two likely causes. First, rule M prices no travel: it values a site by its sugar alone, so a far remembered site believed full beats a near one. Under the utility mind with travel 0.5 (memory as here), the advantage rises by a median 119, higher in every seed, to about neutral: +7.4 (IQR −18 to +37), rememberers wealthier in only 10 of 20 seeds. But the share of rememberers' choices aimed at a remembered site falls from 0.68 to 0.10, and across the arms the loss tracks how often memory is used (projection uses it about twice as often as recall, 0.68 against 0.34, and loses more). So pricing travel removes the loss, largely by using memory less; no gain was shown. Second, projection ignores competitors who harvest a site first: under recall (believing a site holds what it held when seen) the advantage is −34, and projection is worse than recall in every seed, with a larger belief error (2.67 against 2.46 sugar; measured only on the targets chosen, the ones believed best, so biased by that selection). Most choices of a remembered site out of sight find less there than believed under either belief (93 % under project, 95 % under recall). Memory for everyone (share 1) lowers the population to 154 from walking's 181 (mean over ticks 300–500), in every seed, so memory doesn't restore the capacity walking lost. Across spans (the mem-span sweeps) the advantage is negative at every span and growback rate, so the \"best\" span is only the least-harmful one; under recall it's 10 ticks at growback 1, the shortest span tested (IQR 10–10), against 25 at 0.25 and 0.5. Pinned at the floor of the grid, it can't show Bracis et al.'s forgetting that tracks regrowth.",
            |c| {
                c.movement.mode = MoveMode::Walk;
                memory(c, 100, 0.5);
            },
        ),
        preset(
            "mem-catchment",
            "Memory: patches out of sight",
            "Minds 3",
            "ifd-no-starving's world (the 2.10 : 1 patches with an endowment so large nobody starves), walking under the utility mind with idle wander, and memory: a rememberer that once saw a patch can walk back to it once it's out of sight, instead of wandering blind like everyone else. Measured (20 seeds): rememberers first reach a patch sooner (median tick 27.5 against 33) and spend slightly more of ticks 200–500 on a patch (90 % against 89 %), yet they end slightly poorer in every seed, by a median 90 sugar over ticks 200–500, on holdings near 100 000 (under 0.1 %). Likely causes, not isolated here: travel that costs a walker time goes unpriced in the choice, so far remembered sites beat near ones; and projection counts on regrowth that others harvest first (see mem-open).",
            |c| {
                two_patches(c, 7.0);
                c.goods[0].endowment = URange::new(100_000, 100_000);
                c.movement.mode = MoveMode::Walk;
                c.decision.rule = DecisionRule::Utility;
                c.decision.idle = Idle::Wander;
                memory(c, 200, 0.5);
            },
        ),
        preset(
            "mem-walled",
            "Memory: beyond the wall",
            "Minds 3",
            "ifd-wall's world (the opaque wall with a central gap, hiding the far patch from view) with memory: only a rememberer that once passed through the gap and saw the far patch can walk back to it once it's hidden again. Measured (20 seeds, ticks 200–500): memory is ruinous here. Rememberers hold a median 12 sugar against the others' 127, poorer in every seed, and only 7 % of them are alive at tick 500 against 74 % of the others. On ticks when any rememberer chose a remembered site out of sight, such choices were 96 % of their moves, and 98 % of them found less than believed. Recall is strongly negative too: an advantage of −40, negative in every seed, with 49 % of rememberers alive at tick 500 against 73 % of the others; so projection isn't the only driver; likely causes are both unpriced travel and projection. Rule M prices no travel, so with vision 10–20 a rememberer walks to far remembered sites believed full and starves on the way. Under the utility mind with travel 0.5 the advantage improves from −114 to −55, higher in every seed, but stays negative, while the share of rememberers' choices aimed at a remembered site falls from 0.96 to 0.65: pricing travel shrinks the loss, largely by using memory less; no gain was shown. The two arms aren't additive and the travel arm also changes the mind, so they don't rank the causes.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.movement.mode = MoveMode::Walk;
                fence(c, 0, true);
                memory(c, 200, 0.5);
            },
        ),
        preset(
            "mem-seasons",
            "Memory: the other hemisphere",
            "Minds 3",
            "walk-seasons's world (seasons flipping every 50 ticks, agents walking) with memory: a rememberer can walk back toward the hemisphere it last saw thriving, instead of relying on what's in sight when the season turns. Measured (20 seeds, ticks 200–500): rememberers end poorer in every seed (a median 295 sugar against 342), and 23 % of them are alive at tick 500 against 36 % of the others; 96 % of their choices of a remembered site find less than believed. Likely causes, not isolated here: rule M prices no travel, so far remembered sites beat near ones, and projection counts on regrowth that others harvest first (see mem-open).",
            |c| {
                enable_seasons(c);
                c.movement.mode = MoveMode::Walk;
                memory(c, 100, 0.5);
            },
        ),
        preset(
            "mem-truffles",
            "Memory: hidden truffle spots",
            "Minds 3",
            "walk-capacity's world with truffles: hash-placed spots that are invisible until walked onto, worth 5 sugar, and take 30 ticks to regrow after being picked (5 % of sites). Only a rememberer can head back to a spot it has already found. Measured (20 seeds): rememberers do find more truffles, 0.0147 an agent-tick against 0.0065 (ticks 1–500), more in every seed, but they end poorer in 18 of 20 seeds (a median 347 sugar against 430 over ticks 200–500). A likely contributor: chasing remembered sugar far away costs more than truffles bring, since rule M prices no travel. Under the utility mind with travel 0.5 the advantage rises from −82 to about neutral, +1.5, higher in 18 of 20 seeds, while the share of rememberers' choices aimed at a remembered site falls from 0.73 to 0.25: pricing travel removes the loss, largely by using memory less; no gain was shown. Projection adds to the loss: under recall the advantage is −25 (still a loss) against −82, and projection is worse than recall in every seed, with twice the belief error (4.99 against 2.46).",
            |c| {
                c.movement.mode = MoveMode::Walk;
                truffles(c, 0.05, 5.0, 30);
                memory(c, 200, 0.5);
            },
        ),
        preset(
            "mem-trapline",
            "Memory: a trapline of truffle spots",
            "Thomson, Slatkin & Thomson 1997; Ohashi & Thomson 2005; Gill 1988; Minds 3",
            "A sparse 50 × 50 torus (flat capacity 1, growback 0.1) where truffle spots (2 % of sites, worth 10 sugar, regrowing after 40 ticks) are the main food; 20 agents of metabolism 1, endowment 30 and vision 1–6 walk, and every agent remembers for 400 ticks. Traplining — a fixed round of revisits timed to regrowth — should appear among rememberers. Measured (20 seeds, ticks 1–1000): it does. The index of return variability (Thomson, Slatkin & Thomson 1997: 0 for a perfect trapliner, 1 for random revisits) has a per-seed median of 0.15, below 0.8 in every seed. With half the agents remembering, rememberers gather 0.050 truffles an agent-tick against 0.011 and are wealthier in every seed (144 sugar against 76 over ticks 200–500), as Ohashi & Thomson's \"more competitive\" predicts. The non-rememberers' index is 0.36 there, which also passes the 0.8 threshold, so the index alone doesn't show memory making the trapline; likely the sparse map channels anyone's wanderings through the same spots. The +69 advantage (144 against 76) is at share 0.5, not the preset's share 1. Gill's competition effect doesn't appear: the median interval between visits to the same spot is 50 ticks with 5 agents and with 20 (regrowth takes 40), and only about 12 % of revisits come sooner than 40 ticks either way.",
            |c| {
                c.width = 50;
                c.height = 50;
                c.population = 20;
                c.goods[0].map = Map::Flat { capacity: 1.0 };
                c.growback.rate = 0.1;
                c.goods[0].metabolism = URange::new(1, 1);
                c.goods[0].endowment = URange::new(30, 30);
                c.movement.mode = MoveMode::Walk;
                truffles(c, 0.02, 10.0, 40);
                memory(c, 400, 1.0);
            },
        ),
        preset(
            "mem-mvt",
            "Memory: the marginal value theorem",
            "Charnov 1976; Minds 3",
            "10 agents (metabolism 1, endowment 50, vision 1–20) on a 60 × 60 torus with nine equal patches (peaks of radius 4, height 4) on a square lattice of spacing 20, growback 0.25; walking under the utility mind with travel k = 0.5, and memory (span 400, share 1) so every agent can walk back to a remembered patch. Few foragers, patches in sight and a travel cost is the marginal value theorem's setting: when to leave a patch that's still yielding. Measured (20 seeds, ticks 1–1000): under this mind, foragers who find a patch never leave it. A patch here takes in far more than one agent eats, so no agent ever departs (0 departures over all 20 seeds) and a median 5 of the 10 are alive at tick 1000, each settled on a patch; the rest likely start with no sugar in sight and stand still until they starve. Depleting patches don't help: with radius 2 and growback 0.05 (0.45 sugar a tick per patch), nobody is alive after tick 200 with 10 agents; with 3, nobody is alive at tick 1000 and only 3 departures happen across 20 seeds. So the theorem's decision, leaving when a patch's intake falls to the habitat's average, is untestable here. Likely reason: rule M and the utility mind compare the values of sites, not rates of intake, and hold no estimate of the habitat's average rate; and with sight only along rows and columns, a forager that has emptied a patch often has no other patch in sight. The marginal value theorem is left to Minds 4 (planning).",
            |c| {
                c.width = 60;
                c.height = 60;
                c.population = 10;
                c.goods[0].map = Map::Peaks {
                    peaks: (0..3u32)
                        .flat_map(|i| {
                            (0..3u32).map(move |j| Peak {
                                x: 10 + 20 * i,
                                y: 10 + 20 * j,
                                radius: 4.0,
                                height: 4.0,
                            })
                        })
                        .collect(),
                };
                c.goods[0].metabolism = URange::new(1, 1);
                c.goods[0].endowment = URange::new(50, 50);
                c.vision = URange::new(1, 20);
                c.growback.rate = 0.25;
                c.movement.mode = MoveMode::Walk;
                c.decision.rule = DecisionRule::Utility;
                c.decision.travel = 0.5;
                memory(c, 400, 1.0);
            },
        ),
        preset(
            "goap-mvt",
            "Planning: when to leave a patch",
            "Charnov 1976; Orkin 2006; Minds 4",
            "The marginal value theorem's world, for a planner: 3 agents (metabolism 1, endowment 50, vision 1–6) on a 60 × 60 torus with nine equal patches (peaks of radius 3, height 4, 25 sites each) on a square lattice of spacing 20, growback 0.02. They walk, and plan with GOAP: each tick an agent without a plan searches for the fastest sequence of harvests, among the 8 sites it knows with the most sugar per tick of walking (value ÷ (distance + 1)), that gathers 10 ticks of food, pricing each walk by its length. Every agent remembers (span 1 000, share 1) and starts knowing the whole map, as the theorem's ideal forager does. The balance is set so a patch runs out under one forager (it takes in 0.5 sugar a tick against a forager's 1) while all nine patches together take in 1.5 times what the population eats (4.5 against 3). Measured (20 seeds, ticks 1–1000, the same world at spacings 12, 16, 20 and 24): longer travel, longer stays, as Charnov's theorem predicts. The fitted slope of mean patch residence on spacing is positive in every seed (a median 0.41 ticks per unit of spacing), though the median residence (18.6, 22.6, 25.7 and 22.8 ticks) dips between spacings 20 and 24, where a median 1 of the 3 agents is alive at tick 1000 at both. Survival falls with spacing (a median 3, 2, 1 and 1 alive), so the longest spacings rest on fewer foragers. At spacing 20, 28 of the 60 agents are alive at tick 1000. By the literal measure, agents overstay at a median 57 % of departures (above half in 18 of 20 seeds): their last tick in the patch gathers less than their average so far. That last tick is likely often a step walking out: counting only the last in-patch tick on which an agent landed on a plan target, the share is 30 % at spacing 20, above half in no seed (1 % of departures dropped as all transit). Unlike Constantino and Daw's people, they overstay less as travel grows (61 % at spacing 12, 46 % at 24 by the literal measure; 33 % and 20 % without transit). Rule M with the same knowledge (the map, memory) leaves after about 5 ticks at every spacing, its stays shorten as spacing grows in every seed, and at spacings 16 to 24 a median 0 of its 3 agents is alive at tick 1000. The planner plans afresh on 12 % of agent-ticks and falls back on under 1 %. With the value shortlist (the spec's first design: the 8 best known sites by value alone), stays shorten with spacing in every seed and only 4 of 60 are alive at tick 1000 at spacing 20; likely because ranking by value alone leaves only far patch centers to plan over.",
            |c| mvt_world(c, DecisionRule::Goap),
        ),
        preset(
            "mvt-rule",
            "The marginal-value rule",
            "Charnov 1976; Constantino & Daw 2015; Minds 4",
            "goap-mvt's world under the marginal-value rule instead of planning: each agent keeps ρ, a running mean of its intake per tick (travel ticks count as 0; smoothing α 0.05; starting at its metabolism). It stays while the best site within one step, its own included, is believed to hold at least ρ, and harvests there; otherwise it leaves for the best site it knows, committed until it arrives. Every agent knows the whole map from the start and remembers what it sees (span 1 000). Measured (20 seeds, ticks 1–1000, the same world at spacings 12, 16, 20 and 24): stays lengthen with travel in 19 of 20 seeds, but only slightly (median residence 11.8, 12.4, 12.7 and 12.9 ticks); at spacing 20 that's half the planner's 25.7. A median 1 of the 3 agents is alive at tick 1000 at spacing 20. It doesn't overstay: at spacing 20 a median 39 % of departures follow a last tick in the patch below the agent's average so far, under half in every seed, and the share falls as travel grows (56 % at spacing 12, 35 % at 24), against the planner's 57 %. Counting only the last in-patch tick on which it wasn't leaving, the share is 2 % at spacing 20 (9 % of departures dropped as all transit), so the literal figure is likely mostly ticks spent walking out. Likely reason: it leaves as soon as the best site within one step is believed below ρ, a test made before the harvest, so it rarely stays on for a poor last tick.",
            |c| mvt_world(c, DecisionRule::Mvt),
        ),
        preset(
            "goap-open",
            "Planning with memory: open sugarscape",
            "Orkin 2006; Minds 4",
            "mem-open's world (walk-capacity's agents walking to their targets; half of them remember for 100 ticks) with the planner instead of rule M: each agent without a plan searches for the fastest sequence of harvests, among the 8 sites it sees or remembers with the most sugar per tick of walking (value ÷ (distance + 1)), that gathers 10 ticks of food, pricing each walk by its length. Measured (20 seeds, ticks 200–500): memory pays a planner. Rememberers hold a median 319 sugar against the others' 240, an advantage of +69, positive in 19 of 20 seeds, where under rule M it was −113 on the same seeds; the planner's advantage is higher in every seed. Counting the dead as 0 (sugar held per founding member, mean over ticks 200–500, no survivorship), the advantage is still +35, positive in 18 of 20 seeds. Memory is used, not avoided: 99 % of rememberers' plans include a remembered site out of sight. The planner's others are much poorer than rule M's (240 against 438). Two likely causes, neither isolated: the planner holds more agents (a median 200 against 177 over ticks 200–500), so more share the same sugar; and an agent takes the fallback (too little known sugar for 10 ticks of food) on 40 % of agent-ticks, likely mostly the non-rememberers, who know only what's in sight. A new plan is made on 22 % of agent-ticks. In the goap-k sweep, K from 2 to 12 changes the population little (197–208).",
            |c| {
                c.movement.mode = MoveMode::Walk;
                memory(c, 100, 0.5);
                c.decision.rule = DecisionRule::Goap;
            },
        ),
        preset(
            "goap-truffles",
            "Planning with memory: hidden truffle spots",
            "Orkin 2006; Minds 4",
            "mem-truffles's world (walk-capacity's world with truffle spots, invisible until walked onto, worth 5 sugar and regrowing 30 ticks after being picked, on 5 % of sites; half the agents remember for 200 ticks) with the planner instead of rule M: each agent without a plan searches for the fastest sequence of harvests, among the 8 sites it sees or remembers with the most sugar per tick of walking (value ÷ (distance + 1)), that gathers 10 ticks of food, pricing each walk by its length. Measured (20 seeds, ticks 200–500): memory pays a planner. Rememberers hold a median 363 sugar against the others' 254, an advantage of +99, positive in every seed, where under rule M it was −82 on the same seeds. Fewer rememberers survive (41 % alive at tick 500 against 53 %), so part of the gap is survivorship: counting the dead as 0 (sugar per founding member, mean over ticks 200–500), the advantage is +26 but positive in only 14 of 20 seeds, short of the 80 % the survey asks for; so rememberers are richer while they live, not clearly per head of those born. 98 % of rememberers' plans include a remembered site out of sight. In the goap-memory sweep the mean advantage is +90 to +113 at every share remembering from 0.1 to 0.9, against −73 to −98 under rule M (mem-share).",
            |c| {
                c.movement.mode = MoveMode::Walk;
                truffles(c, 0.05, 5.0, 30);
                memory(c, 200, 0.5);
                c.decision.rule = DecisionRule::Goap;
            },
        ),
        preset(
            "goap-walled",
            "Planning with memory: beyond the wall",
            "Orkin 2006; Minds 4",
            "mem-walled's world (the 2.10 : 1 patches with vision 10–20, split by an opaque wall with a central gap; half the agents remember for 200 ticks) with the planner instead of rule M: each agent without a plan searches for the fastest sequence of harvests, among the 8 sites it sees or remembers with the most sugar per tick of walking (value ÷ (distance + 1)), that gathers 10 ticks of food, pricing each walk by its length. Measured (20 seeds, ticks 200–500): in mem-walled, rememberers starved walking to far remembered sites (7 % alive at tick 500 against 74 %, an advantage of −114). Under the planner they don't: rememberers hold a median 106 sugar against the others' 75, an advantage of +33, positive in every seed, and 60 % of them are alive at tick 500 against 74 % of the others. Counting the dead as 0 (sugar per founding member, mean over ticks 200–500), the advantage is +10, positive in only 15 of 20 seeds, short of the 80 % the survey asks for. 99 % of rememberers' plans include a remembered site out of sight. The population is larger than rule M's (a median 67 against 46 over ticks 200–500). Likely cause: the planner counts the walk, so a far remembered site enters a plan only when its sugar repays the ticks of walking.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
                c.movement.mode = MoveMode::Walk;
                fence(c, 0, true);
                memory(c, 200, 0.5);
                c.decision.rule = DecisionRule::Goap;
            },
        ),
        preset(
            "cache-winter-none",
            "Caching: winter, nothing put away",
            "Minds 5",
            "The winter world: walk-capacity's landscape with 175 agents of metabolism 1 (half of them remembering for 100 ticks) walking under rule M, and a winter everywhere at once: every site grows back 1 a tick for 100 ticks, then 1/32 a tick for 100. An agent carries at most 50 sugar, half a winter's need; every winter world sets the caching reserve to 20 ticks' food. Nobody caches: whatever an agent can't carry, it leaves. Measured (20 seeds, ticks 0–1000): half of those alive when winter comes die in it (49 % survive the first winter, from tick 100 to 200), and later winters barely thin the rest (49 % of those alive at tick 100 are alive at 1000), likely because the survivors are those who can find winter's scarce sugar. Of the 175 founders, 47 % are alive at tick 200 and 46 % at tick 1000.",
            |c| winter_world(c, CachingRule::None, false),
        ),
        preset(
            "cache-winter-even",
            "Caching: winter, an even share",
            "Minds 5",
            "The winter world: walk-capacity's landscape with 175 agents of metabolism 1 (half of them remembering for 100 ticks) walking under rule M, and a winter everywhere at once: every site grows back 1 a tick for 100 ticks, then 1/32 a tick for 100. An agent carries at most 50 sugar, half a winter's need; every winter world sets the caching reserve to 20 ticks' food. Each agent buries half its surplus (what it holds above its reserve) where it stands, every tick it has any, and digs its caches when it holds less than half its reserve. Measured (20 seeds, ticks 0–1000): 74 % survive the first winter against 49 % with no caching in the same world (higher in all 20 seeds), and 57 % of those alive at tick 100 are alive after five winters, against 49 %. Of the 175 founders (the dead counted), 70 % are alive at tick 200 and 54 % at 1000. Agents bury often but dig back only a quarter of it: by tick 1000, the end of the fifth winter, 25 % of the sugar buried has been dug (a mean 220 ticks after the cache was begun); of the 75 % not dug by then, 16 % of all buried died with its owner and 59 % is still buried, so sugar held and cached per founder climbs to 387. A hungry agent heads for its biggest cache (70 % of its choices, and 70 % of those where no biggest cache is also a nearest one), a mean 6.5 steps away when its nearest is 1.4. 89 % of the agents who die holding caches die heading for one, and 41 % of those, when they last chose a cache, chose one beyond the reach of their food while a nearer one was within it: likely one cause of the deaths, not isolated.",
            |c| winter_world(c, CachingRule::Even, false),
        ),
        preset(
            "cache-winter-compensate",
            "Caching: winter, compensating",
            "Amodio et al. 2021; Minds 5",
            "The winter world: walk-capacity's landscape with 175 agents of metabolism 1 (half of them remembering for 100 ticks) walking under rule M, and a winter everywhere at once: every site grows back 1 a tick for 100 ticks, then 1/32 a tick for 100. An agent carries at most 50 sugar, half a winter's need; every winter world sets the caching reserve to 20 ticks' food. Each agent buries a share of its surplus where it stands, weighted by where food has been scarce: a site's weight halves each time it finds food there, and it buries half its surplus × the site's weight ÷ the mean weight of the sites it knows. It digs its caches when it holds less than half its reserve. Measured (20 seeds, ticks 0–1000): 74 % survive the first winter against 49 % with no caching (higher in all 20 seeds), about as many as with an even share, and 55 % of those alive at tick 100 are alive after five winters, against 49 %. Of the 175 founders, 70 % are alive at tick 200 and 53 % at 1000. By tick 1000, the end of the fifth winter, 25 % of the sugar buried has been dug; of the 75 % not dug by then, 18 % of all buried died with its owner and 57 % is still buried. A hungry agent heads for its biggest cache (70 % of its choices, and 70 % of those where no biggest cache is also a nearest one), a mean 6.6 steps away when its nearest is 1.4. 91 % of the agents who die holding caches die heading for one, and 44 % of those, when they last chose a cache, chose one beyond the reach of their food while a nearer one was within it.",
            |c| winter_world(c, CachingRule::Compensate, false),
        ),
        preset(
            "cache-winter-plan",
            "Caching: winter, planning",
            "Raby et al. 2007; Minds 5",
            "The winter world: walk-capacity's landscape with 175 agents of metabolism 1 (half of them remembering for 100 ticks) walking under rule M, and a winter everywhere at once: every site grows back 1 a tick for 100 ticks, then 1/32 a tick for 100. An agent carries at most 50 sugar, half a winter's need; every winter world sets the caching reserve to 20 ticks' food. Each agent knows the calendar and plans: through the summer it buries toward its forecast shortfall (100 ticks' food, less the winter intake it recorded last winter, less what it has cached), at sites it foraged last winter (anywhere before its first winter, when it forecasts no winter intake). It digs its caches when it holds less than half its reserve. Measured (20 seeds, ticks 0–1000): 88 % survive the first winter, against 49 % with no caching, 74 % with an even share and 74 % compensating (higher than each in all 20 seeds), and 70 % of those alive at tick 100 are alive after five winters, against 49 %, 57 % and 55 %. Of the 175 founders, 84 % are alive at tick 200 and 67 % at 1000. Planners bury a sixth as much sugar as even-share agents, and by tick 1000, the end of the fifth winter, have dug back 44 % of it (a mean 312 ticks after the cache was begun); of the 56 % not dug by then, 12 % of all buried died with its owner and 44 % is still buried. They bury a median 82 per agent in the first summer, forecasting no winter intake, and 5.4 in the second: the survivors of the first winter forecast a mean 54 of winter intake from the sugar they gathered at sites (likely the sugar standing when winter came), and the rule counts what they still have cached against the shortfall. A hungry planner heads for its biggest cache (76 % of its choices, and 76 % of those where no biggest cache is also a nearest one), a mean 5.0 steps away when its nearest is 2.1. 94 % of the planners who die holding caches die heading for one, and 41 % of those, when they last chose a cache, chose one beyond the reach of their food while a nearer one was within it.",
            |c| winter_world(c, CachingRule::Plan, false),
        ),
        preset(
            "cache-winter-mixed",
            "Caching: winter, four rules side by side",
            "Minds 5",
            "The winter world: walk-capacity's landscape with 175 agents of metabolism 1 (half of them remembering for 100 ticks) walking under rule M, and a winter everywhere at once: every site grows back 1 a tick for 100 ticks, then 1/32 a tick for 100. An agent carries at most 50 sugar, half a winter's need; every winter world sets the caching reserve to 20 ticks' food. A quarter of the agents on each caching rule, dealt round-robin by id: none, an even share, compensating and planning. Measured (20 seeds, ticks 0–1000): planners are a quarter of the founders but 32 % of the survivors at tick 200 and 31 % at tick 1000; agents who don't cache are 18 % and 19 %. Of each rule's founders, alive at tick 200 and 1000: planning 84 % and 70 %, compensating 68 % and 52 %, an even share 65 % and 56 %, none 47 % and 41 %. Each rule fares about as it does in a world of its own, likely because nobody sees or takes another's caches.",
            |c| winter_world(c, CachingRule::None, true),
        ),
        preset(
            "central-near",
            "Central-place foraging: a near patch",
            "Orians & Pearson 1979; Stephens & Krebs 1986; Minds 5",
            "5 agents with homes 8 columns west of a column of five sugar patches (a 60 × 30 torus; the patches grow back 0.25 a tick) forage in round trips and carry their loads home to a larder. Each keeps ρ, its delivered sugar per tick over its trips; in a patch it stays while the best site within one step yields at least ρ (and it isn't full, or low on food for the walk home), then walks home, delivers and leaves again, keeping enough in hand for the walk out and back. They know the whole map, carry loads of at most 320 and burn 1 a tick. central-far is the same world with homes 20 columns away. Measured (20 seeds, ticks 1–1000): every agent lives; a trip gathers a mean 128 sugar and brings 84 home, having eaten the rest on the way. 64 % of trips end when no site nearby yields ρ, and 36 % with a full load. On 40 % of trips the agent turns home on its first tick deciding in the patch: a delivery lifts ρ by about 0.05 × the load (4.1 here), as much as the best site yields, and ρ hasn't decayed much by the time it gets back (a median 2.4 on arrival). Loads rise with the patches' distance from home, as central-place theory predicts: a mean 68 per trip at 4 columns, 84 at 8, 100 at 12, 107 at 16, 108 at 20 and 110 at 24, rising in all 20 seeds. ρ on arrival is lower after a longer walk (0.73 at 20 columns), likely a contributor, not isolated; and far out the carrying limit sets the loads (53 % of trips end full at 20 columns). Lima's prediction, that a near and a far patch in one habitat give the same load, fails as it was set out: sorting trips by the patch that gave most of the load, far trips bring home 120 and near ones 54, and the carrying limit ends 75 % of those far trips. But the near patch lay on the way to the far one, and agents harvest where they step. Sorted by where the agent was headed, the loads are 75 far and 71 near; with ρ held fixed, 82 and 85, though fewer agents live (88 of 100 at tick 1000, against 97 with ρ learned). With the far patch on the other side of home, so no trip crosses the other patch, they are 83 and 73, and with ρ held fixed there, 100 and 74 (80 of 100 alive, against 99). There the far patch holds more sugar when an agent arrives (37.5 against 33.6): an agent heads for the far patch only when a site there is strictly richer than every near one (ties go to the nearer), likely part of why; but that 12 % difference in standing sugar only partly explains a 35 % difference in loads, and the rest isn't isolated. In no arrangement are near and far within 10 % of each other in most seeds.",
            |c| central_world(c, 37, false),
        ),
        preset(
            "central-far",
            "Central-place foraging: a far patch",
            "Orians & Pearson 1979; Stephens & Krebs 1986; Minds 5",
            "central-near's world with the homes 20 columns west of the patches instead of 8: 5 agents forage in round trips under the marginal-value rule and carry their loads home to a larder, loads of at most 320. Measured (20 seeds, ticks 1–1000): every agent lives; a trip gathers a mean 176 sugar and brings 108 home (central-near: 128 and 84). The carrying limit sets these loads: 53 % of trips end with a full load, and 47 % when no site nearby yields ρ. The agent's ρ has decayed to a median 0.73 by the time it reaches the patch (2.4 in central-near), and it stays a median 117 ticks, harvesting what the sites regrow, likely because the long walk has eroded ρ, not because the patch is richer.",
            |c| central_world(c, 25, false),
        ),
        preset(
            "central-linear",
            "Central-place foraging: linear loading",
            "Orians & Pearson 1979; Stephens & Krebs 1986; Minds 5",
            "central-near's world (homes 8 columns from the patches) with instant growback: every site refills at once, so a patch never runs down and a load grows in step with the time spent gathering it. Measured (20 seeds, ticks 1–1000): every agent lives; a trip brings home a mean 121 sugar. Trips come in two kinds: 54 % end almost at once (36 % on the agent's first tick deciding in the patch, and 17 % before it decides there at all, likely because its target was taken on the way), and 46 % fill to the limit. A delivery lifts ρ by about 0.05 × the load (5.9 here), above the most any site yields (4), so the next trip turns back at once; with little delivered, ρ likely decays and the trip after fills up. The marginal-value theorem predicts no effect of distance with linear loading, but here, as with Kacelnik and Cuthill's starlings, loads grow with distance: 151 per trip with the homes 20 columns away, likely because ρ decays over the longer walk.",
            |c| central_world(c, 37, true),
        ),
        preset(
            "cache-raby",
            "Caching lab: Raby's breakfast test",
            "Raby et al. 2007; Minds 5",
            "Raby et al.'s \"planning for breakfast\" protocol on a 13 × 8 rig of three compartments and a hall. For six days, 8 agents spend each morning shut in K1 or K3 in turn (K1 first), with breakfast only in K3; each evening they're back in the hall. On the test evening the doorways of K1 and K3 open and each agent, given 30 sugar, walks out alone and caches it. The agents take the caching rules round-robin: none, an even split, compensating (weights halved where food was found) and planning (the cycle finder's forecast of the next morning), two agents each. In the other order (breakfast first), a planner looking one day ahead caches nothing: tomorrow has breakfast. Measured (20 seeds, 8 agents per seed in each order, 320 agents per rule): compensating agents cache 26.4 ± 0.1 in the compartment without breakfast against 3.6 ± 0.1 in the other (mean ± s.e.m.; Raby et al.'s birds: 16.3 ± 1.8 against 5.4 ± 1.8), every one of them more where breakfast was missing. Planners cache all 30 there in this order and nothing at all in the other, so half of all planners cache nothing. Even splitters cache 15 in each, as their rule predicts.",
            |c| mixed_lab(c, Lab { protocol: LabProtocol::Raby, food_first: false }, 8),
        ),
        preset(
            "cache-amodio",
            "Caching lab: Amodio's rotating compartments",
            "Amodio et al. 2021; Minds 5",
            "Amodio et al.'s Experiment 2 (Food-First) on a 13 × 8 rig of three compartments and a hall. For nine days, 6 agents spend each morning shut in K1, K2, K3, K1, … in turn, with food on the first day and every other day after; each evening they're back in the hall. On the test evening all three doorways open and each agent, given 30 sugar, walks out alone and caches it. The agents take the caching rules round-robin: none, an even split, compensating (weights halved where food was found) and planning (the cycle finder's forecast of the next morning). Measured (20 seeds, 6 agents per group per seed, both groups): each rule leaves its predicted pattern in every seed. Even splitters cache 10/10/10 in K1/K2/K3; compensating agents most in K2 in the Food-First group (a mean 7.9/15.2/6.9) and least in K2 in the Empty-First group (12.5/6.1/11.5); planners 30 in K1, and nothing in the Empty-First group, where tomorrow has food; planners looking three days ahead 15/0/15 and 0/30/0. Amodio et al.'s Bayesian model comparison, written from their Methods and checked on their own Table 2 (it gives their 0.72, 0.16 and 0.002), picks each rule's hypothesis on its data: the compartment-independent model at 0.999 for even splitters, CCH at 0.90 for compensating agents (at the paper's size, three agents a group, CCH wins in 15 of 20 seeds and the compartment-independent model in the other 5), FPH 1 at 0.55 and FPH 2 at 0.62 for planners. Those two are the most the comparison can give planners: every planner caches alike, so it treats them as one bird, and the hypotheses share the evidence (FPH 2's constraint also fits caching only in K1, FPH 1's fits 15/0/15 at half weight, K1 and K3 tied, and the two agree on the Empty-First group). The jays' 0.72 for the compartment-independent model looks like the even splitters' signature.",
            |c| mixed_lab(c, Lab { protocol: LabProtocol::Amodio, food_first: true }, 6),
        ),
        preset(
            "theft-winter",
            "Theft: winter, hoarders who can be robbed",
            "Vander Wall & Jenkins 2003; Minds 6",
            "cache-winter-even's world (walk-capacity's landscape, 175 agents of metabolism 1 walking under rule M, half of them remembering for 100 ticks; every site grows back 1 a tick for 100 ticks, then 1/32 a tick for 100; a carrying limit of 50 and a caching reserve of 20 ticks' food), where every agent buries half its surplus where it stands, with theft: an agent arriving on a site finds each cache of someone else's there with chance 0.25, and takes what it can carry of it. Owners remember their own caches, and burying is free. Measured (20 seeds, ticks 1–200): 90 % of the agents alive at tick 100 survive the first winter (86 % of the founders), against 74 % (70 %) in the same world without theft: theft likely pools the stores, since a thief is likely a hungry agent that walks onto someone else's surplus. 2.3 % of the caches are pilfered a tick, at the low end of field studies' 2–30 % a day and below their median of 9 %: a stranger stands on a given cache only 0.12 times a tick, an arrival takes at most one of the caches on a site, and even finding every cache stood on gives 7.0 % of the caches (10.9 % of the sugar). Sugar changes hands again and again (66 000 pilfered per seed against 74 000 buried, likely much of it buried again by its thief): by tick 200, 89 % of the sugar ever buried has been pilfered, 0.3 % dug by its owner and 11 % is still buried. Owners dig only when they hold under half their reserve, which in practice means winter (99 % of digging), while thieves take caches all year (47 % in summer); the sugar owners dig is a mean 81 ticks old, the sugar pilfered 24 (lowered because loot buried again counts as new): likely why an owner's memory of its caches is so seldom used.",
            |c| theft_winter(c, 0.0),
        ),
        preset(
            "theft-winter-quarter",
            "Theft: winter, a quarter cheaters",
            "Vander Wall & Jenkins 2003; Andersson & Krebs 1978; Minds 6",
            "theft-winter's world (cache-winter-even's winter, with each cache found by a stranger arriving on it with chance 0.25) where a quarter of the agents, dealt by id, are cheaters: they never cache, and take what they can carry of any cache they find. The rest bury half their surplus where they stand. Measured (20 seeds, ticks 1–200): of those alive at tick 100, 97.5 % of the cheaters survive the first winter against 88 % of the hoarders (91 % and 84 % of the founders). Cheating pays by transfer, not on its own: in the same world with nobody finding caches, cheaters survive 49 %, as agents do where nobody caches; and here everyone steals, hoarders five to seven times as much per founder as cheaters (likely much of it buried again, and stolen again). Hoarders end the winter richer, 53 against 36 sugar held and cached per founder, mostly in caches still buried. 2.4 % of the caches are pilfered a tick, and 89 % of the sugar ever buried has been pilfered by tick 200, 0.5 % dug by its owner.",
            |c| theft_winter(c, 0.25),
        ),
        preset(
            "theft-winter-half",
            "Theft: winter, half cheaters",
            "Vander Wall & Jenkins 2003; Andersson & Krebs 1978; Minds 6",
            "theft-winter's world (cache-winter-even's winter, with each cache found by a stranger arriving on it with chance 0.25) where half the agents, dealt by id, are cheaters: they never cache, and take what they can carry of any cache they find. The rest bury half their surplus where they stand. Measured (20 seeds, ticks 1–200): of those alive at tick 100, 95 % of the cheaters survive the first winter against 82 % of the hoarders (90 % and 78 % of the founders); with nobody finding caches the cheaters survive 50 %. The owner's advantage is almost never used: owners dig back 1.5 % of their ended caches' sugar and thieves take 98 %. Owners dig only below half their reserve, in practice in winter (99.7 % of digging), while thieves take caches all year (47 % in summer), so the sugar owners dig is a mean 80 ticks old and the sugar pilfered 30 (lowered because loot buried again counts as new). Likely much of the cheaters' lead: with owners digging below their whole reserve instead (`caching.dig_below: reserve`, which removes the band between half the reserve and all of it, so the sugar dug is a mean 27 ticks old), they dig 26 % of what they bury and survive 93 % against the cheaters' 96 %.",
            |c| theft_winter(c, 0.5),
        ),
        preset(
            "theft-arena-2",
            "Theft arena: two agents",
            "Andersson & Krebs 1978; Minds 6",
            "2 agents shut in a 4 × 4 room walled on all four sides, with flat sugar (capacity 4, growing back 0.3 a tick; 8 sites an agent) through a winter everywhere at once: the room grows back its full rate for 100 ticks, then 1/32 of it for 100. They walk with vision 1–2, burn 1 a tick and carry at most 50. One is a hoarder, burying half its surplus where it stands; the other, a cheater, never caches. Arriving on a site, an agent finds each cache of the other's there with chance 0.25, and takes what it can carry of it. Every agent remembers where its own caches are, burying is free and loot is kept; half the agents, as in the winter world, remember what they've seen for 100 ticks. Measured (20 seeds, ticks 1–200): both agents survive the winter in every seed. The cheater takes 51 % of the hoarder's ended caches' sugar and the hoarder digs 49 % (the hoarder steals nothing: the cheater has no caches), so Andersson and Krebs's condition for hoarding to pay (the ratio above 1 with free burying) holds in only 7 seeds. Who ends richer depends on how still-buried caches are valued: counting them in full, the hoarder is richer in all 20 seeds (80 against 41 per founder, 65 of its 80 still in the ground); counting them as nothing, the cheater is richer in all 20 (41 against 14 held). 1.45 % of the caches are pilfered a tick, four fifths of the sugar in winter.",
            |c| theft_arena(c, 2),
        ),
        preset(
            "theft-arena-4",
            "Theft arena: four agents",
            "Andersson & Krebs 1978; Minds 6",
            "4 agents shut in a 6 × 6 room walled on all four sides, with flat sugar, through a winter everywhere at once, with the same sugar per agent as theft-arena-2's room: capacity 32/9 and growback 0.27 a tick at 9 sites an agent (the full rate for 100 ticks, then 1/32 of it for 100). They walk with vision 1–3, burn 1 a tick and carry at most 50. Half are hoarders, burying half their surplus where they stand; half are cheaters and never cache. Arriving on a site, an agent finds each cache of someone else's there with chance 0.25, and takes what it can carry of it. Every agent remembers where its own caches are, burying is free and loot is kept; half the agents, as in the winter world, remember what they've seen for 100 ticks. Measured (20 seeds, ticks 1–200): every agent survives the winter in nearly every seed; thieves take 85 % of the ended caches' sugar and owners dig 15 %, so Andersson and Krebs's condition fails in every seed. Hoarders pilfer 1.2 times as much per founder as cheaters. Counting still-buried caches in full, the hoarders end richer in 12 of 20 seeds (33 against 32 per founder); counting them as nothing, the cheaters are richer in 19 (32 against 11 held). 1.8 % of the caches are pilfered a tick, more than with two agents, likely because more strangers cross each cache (0.075 visits a cache a tick against 0.059).",
            |c| theft_arena(c, 4),
        ),
        preset(
            "theft-arena-8",
            "Theft arena: eight agents",
            "Andersson & Krebs 1978; Minds 6",
            "8 agents shut in an 8 × 8 room walled on all four sides, with flat sugar (capacity 4, growing back 0.3 a tick; 8 sites an agent, as in theft-arena-2) through a winter everywhere at once: the full rate for 100 ticks, then 1/32 of it for 100. They walk with vision 1–4, burn 1 a tick and carry at most 50. Half are hoarders, burying half their surplus where they stand; half are cheaters and never cache. Arriving on a site, an agent finds each cache of someone else's there with chance 0.25, and takes what it can carry of it. Every agent remembers where its own caches are, burying is free and loot is kept; half the agents, as in the winter world, remember what they've seen for 100 ticks. Measured (20 seeds, ticks 1–200): every agent survives the winter in nearly every seed; thieves take 91 % of the ended caches' sugar and owners dig 9 %, so Andersson and Krebs's condition fails in every seed. Hoarders pilfer 1.8 times as much per founder as cheaters. Counting still-buried caches in full, the hoarders end richer in 17 of 20 seeds (37 against 29 per founder); counting them as nothing, the cheaters are richer in all 20 (29 against 12 held). 2.2 % of the caches are pilfered a tick, the most of the three rooms, likely because more strangers cross each cache (0.094 visits a cache a tick), as Andersson and Krebs's reason for their (n − 1) has it.",
            |c| theft_arena(c, 8),
        ),
        preset(
            "watch-winter",
            "Watching: winter, every agent a watcher",
            "Bugnyar & Kotrschal 2002; Heinrich & Pepper 1998; Minds 8",
            "theft-winter's world (cache-winter-even's winter: walk-capacity's landscape, 175 agents of metabolism 1 walking under rule M, half of them remembering for 100 ticks; every site grows back 1 a tick for 100 ticks, then 1/32 a tick for 100; a carrying limit of 50 and a caching reserve of 20 ticks' food; every agent burying half its surplus where it stands), with no stumbling on caches (find 0) and every agent a watcher. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; while it remembers one it may walk there, and on arriving it raids only if it remembers at least as much sugar there as the site would give it, taking what it can carry of the first remembered cache still there (or finding them all gone); otherwise it harvests as usual and keeps the memory. An owner digging its own cache comes first. Owners remember their own caches, burying is free and loot is kept. Measured (20 seeds, ticks 1–200): of the caches buried in ticks 1–90, 6.6 % are taken within a day and 36 % within a week, a hazard of 6.1 % a day, against 0.8 %, 19 % and 3.0 % with Minds 6's stumbling alone (theft-winter); watching's hazard is higher in all 20 seeds. That holds for a memory of 3 ticks or more: with a memory of 1 or 2 ticks, watching's hazard is lower than stumbling's in every seed (0.85 % and 2.49 % a day). Watchers see 88 % of burials and raid 54 % of the caches they see, so what limits theft here is acting on what they see, not seeing it. Watching costs lives: 59 % of the founders survive the first winter, against 70 % in the same world without watching (fewer in all 20 seeds; fitness, the share of ticks 1–200 a founder is alive, 0.891 against 0.907). About two thirds of the cost is the harvest a raid replaces: with a raid also harvesting its site (a survey probe, not a setting), 66 % survive. Raiding whatever the site would give (`raid_if: always`, the first round's rule) costs more: 53 % survive. Capping a cache's value at the room under the carrying limit (`value: room`) changes survival by no detectable amount (−0.1 points, 95 % interval −1.8 to 1.5).",
            |c| watch_winter(c, 0.0, 0.0, 1.0),
        ),
        preset(
            "watch-winter-stumble",
            "Watching: winter, watching and stumbling",
            "Bugnyar & Kotrschal 2002; Heinrich & Pepper 1998; Minds 8",
            "watch-winter's world (theft-winter's winter, every agent a watcher) with Minds 6's stumbling added back: an agent arriving on a site also finds each cache of someone else's there with chance 0.25, and takes what it can carry of it. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; while it remembers one it may walk there, and on arriving it raids only if it remembers at least as much sugar there as the site would give it, taking what it can carry of the first remembered cache still there (or finding them all gone); otherwise it harvests as usual and keeps the memory. An owner digging its own cache comes first. Measured (20 seeds, ticks 1–200): of the caches buried in ticks 1–90, 7.8 % are taken within a day and 50 % within a week, a hazard of 9.6 % a day, three times stumbling's alone (3.0 %, theft-winter). 3.5 % of the caches present are pilfered a tick, against 2.3 % with stumbling alone. Raids take 44 700 sugar a seed and stumbling 20 900, where stumbling alone takes 66 100. 64 % of the founders survive the first winter, against 86 % with stumbling alone (fewer in all 20 seeds; fitness, the share of ticks 1–200 a founder is alive, 0.921 against 0.942). The harvest a raid replaces is the whole cost: with a raid also harvesting its site (a survey probe, not a setting), 89 % survive. Raiding whatever the site would give (`raid_if: always`) leaves 61 %.",
            |c| watch_winter(c, 0.0, THEFT_FIND, 1.0),
        ),
        preset(
            "watch-half",
            "Watching: winter, half cheaters, everyone watching",
            "Bugnyar & Kotrschal 2002; Heinrich & Pepper 1998; Andersson & Krebs 1978; Minds 8",
            "theft-winter-half's world (theft-winter's winter, with each cache found by a stranger arriving on it with chance 0.25, where half the agents, dealt by id, are cheaters who never cache) with every agent a watcher. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; while it remembers one it may walk there, and on arriving it raids only if it remembers at least as much sugar there as the site would give it, taking what it can carry of the first remembered cache still there (or finding them all gone); otherwise it harvests as usual and keeps the memory. An owner digging its own cache comes first. The rest bury half their surplus where they stand. Measured (20 seeds, ticks 1–200): watching hits the hoarders hardest. 36 % of the hoarders survive the first winter against 84 % of the cheaters (per founder), where without watching it is 78 % against 90 %. The hoarders' fitness, the share of ticks 1–200 a founder is alive, falls from 0.929 to 0.876 while the cheaters' stays near 0.95 (0.948 to 0.949), so the hoarders' shortfall grows, in all 20 seeds. Andersson and Krebs's ratio p_s ÷ p_o stays near 0 either way: owners dig 1.8 % of their ended caches' sugar and thieves take 97 % (1.5 % and 98 % without watching), and p_s > p_o in no seed. 4.0 % of the caches are pilfered a tick (2.6 % without watching); raids take 10 800 sugar a seed and stumbling 10 300. Valuing a remembered cache beyond what an agent can carry costs lives: capped at the room under the carrying limit (`value: room`), 65 % of everyone survive instead of 59 % (more in 19 of 20 seeds). With a raid also harvesting its site (a survey probe, not a setting), 46 % of the hoarders survive.",
            |c| watch_winter(c, 0.5, THEFT_FIND, 1.0),
        ),
        preset(
            "watch-scroungers",
            "Watching: winter, half the agents watchers",
            "Bugnyar & Kotrschal 2002; Heinrich & Pepper 1998; Barnard & Sibly 1981; Minds 8",
            "watch-winter's world (theft-winter's winter, no stumbling on caches, every agent burying half its surplus where it stands) where half the agents, dealt by id, are watchers and the rest never watch. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; while it remembers one it may walk there, and on arriving it raids only if it remembers at least as much sugar there as the site would give it, taking what it can carry of the first remembered cache still there (or finding them all gone); otherwise it harvests as usual and keeps the memory. An owner digging its own cache comes first. Measured (ticks 1–200; watcher shares 0.1 to 0.9 over seeds 1–60, the rest seeds 1–20): watchers come out slightly ahead. Pooled over the shares, their fitness, the share of ticks 1–200 a founder is alive, leads the others' by 0.009 (95 % interval 0.003 to 0.016). No fall in the lead was detected as watching becomes common: the per-seed slope on the share is 0.011 (90 % interval −0.004 to 0.027), within the ±0.05 the survey reads as flat. Meanwhile everyone's fitness falls as more watch (slope −0.008, 95 % interval −0.010 to −0.006): a mild social dilemma. The lead depends on the memory: it holds with a memory of 7 or 13 ticks, but with 1, 2 or 3 ticks no lead is detected (pooled 0.001, −0.002 and −0.003). At half, 68 % of the watchers survive the first winter against 67 % of the others (per founder), and 67 % of everyone against 70 % without watching (fewer in 16 of 20 seeds). 0.26 % of the caches are raided a tick.",
            |c| watch_winter(c, 0.0, 0.0, 0.5),
        ),
        preset(
            "watch-scroungers-only",
            "Watching: winter, half the agents only watch and steal",
            "Bugnyar & Kotrschal 2002; Heinrich & Pepper 1998; Barnard & Sibly 1981; Minds 8",
            "watch-scroungers' world (theft-winter's winter, no stumbling on caches, half the agents watchers) where the watchers are also cheaters: watchers and cheaters are dealt by the same id rule, so at half each they are the same agents, who never cache and live by watching. The rest bury half their surplus where they stand and never watch. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; while it remembers one it may walk there, and on arriving it raids only if it remembers at least as much sugar there as the site would give it, taking what it can carry of the first remembered cache still there (or finding them all gone); otherwise it harvests as usual and keeps the memory. An owner digging its own cache comes first. Measured (20 seeds, ticks 1–200): the scroungers trail the hoarders, but the shortfall is there without watching. With watching off, 48 % of the same agents survive the first winter against 70 % of the hoarders (per founder); with watching, 49 % against 65 %, so watching narrows the gap (the hoarders' lead in fitness, the share of ticks 1–200 a founder is alive, falls by 0.009, 95 % interval 0.002 to 0.016). The scroungers live about as long on average as the hoarders (fitness 0.890 against 0.894). A raid takes 1.1 sugar on average, against 5.8 for watchers who also bury. Valuing caches they have no room to carry costs the scroungers: with a cache's value capped at the room under the carrying limit (`value: room`) they raid a fifth as often and their fitness rises by 0.018 (in all 20 seeds), to 54 % surviving. Across scrounger shares from 0.1 to 0.9 they survive 16 to 23 points less than the hoarders (medians), with no trend detected (the 95 % interval of the mean per-seed slope, −0.03 to 0.09, includes 0). 0.24 % of the caches are raided a tick.",
            |c| watch_winter(c, 0.5, 0.0, 0.5),
        ),
        preset(
            "watch-scroungers-forgo",
            "Watching: winter, scroungers who only watch, steal and eat",
            "Bugnyar & Kotrschal 2002; Barnard & Sibly 1981; Vickery et al. 1991; Minds 8",
            "watch-scroungers-only's world (theft-winter's winter, no stumbling on caches, half the agents watchers who are also cheaters, dealt by the same id rule so at half each they are the same agents) with two changes. A scrounger holding a fresh memory of a cache it saw buried considers only those caches and staying put, and harvests no site on any tick it holds one and doesn't raid, as a pure scrounger gives up foraging for its own food. And loot is eaten on the spot, not carried, so a scrounger's raid never fills its carrying limit. The rest bury half their surplus where they stand and never watch. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; a scrounger raids a remembered cache on arriving whatever the site would give, since it would not harvest the site anyway, and eats what it takes of the first remembered cache still there (or finds them all gone). An owner digging its own cache comes first. Measured (ticks 1–200; scrounger shares 0.1 to 0.9 over seeds 1–60, the rest seeds 1–20): forgoing doesn't pay. At half, the scroungers' fitness, the share of ticks 1–200 a founder is alive, is 0.784 against the hoarders' 0.893; 53 % of them survive the first winter against 55 % (per founder), so those that die, die sooner. Their shortfall grows as they become common, as Barnard and Sibly's producer–scrounger game has it (per-seed slope on the share −0.20, 95 % interval −0.23 to −0.17, below 0 in 59 of 60 seeds), from 0.04 at a share of 0.1 up to 0.23 at 0.8 (0.17 at 0.9; medians). But no stable mix forms: the scroungers are ahead at a share of 0.1 in only 13 of 60 seeds, and the median scrounger trails at every share. Never caching costs them a little on its own: in the same worlds with watching off they trail by about 0.02 at every share (a median 0.015 at a share of 0.1; at half, fitness 0.888 against 0.911 and 48 % against 70 % surviving the first winter). Watching adds the rest: per seed, the shortfall with watching less that without is a median 0.034 at a share of 0.1 (in 47 of 60 seeds) and 0.20 at 0.8, so the shortfall when rare is mostly scrounging's, and its growth with share entirely so. Within scrounging, forgoing is the cost: letting scroungers harvest too (`scrounge: harvest`) raises their fitness by 0.13 (in all 20 seeds), above the hoarders'. With a memory of 1 or 2 ticks instead of 7, their shortfall shrinks as they become common instead.",
            |c| {
                watch_winter(c, 0.5, 0.0, 0.5);
                c.watching.scrounge = Scrounge::Forgo;
                c.theft.loot = Loot::Eat;
            },
        ),
        preset(
            "watch-ak",
            "Watching: winter, half cheaters, owners who dig early, everyone watching",
            "Andersson & Krebs 1978; Bugnyar & Kotrschal 2002; Minds 8",
            "theft-winter-half's world (theft-winter's winter, where half the agents, dealt by id, are cheaters who never cache, and the rest bury half their surplus where they stand) with two changes, and every agent a watcher. A stranger arriving on a site finds each cache there with chance 0.02, not 0.25. And an owner digs up one of its caches whenever it holds less than its whole reserve of 20 ticks' food, not half of it. This is item 4 of the calibration list for Andersson and Krebs's claim under watching: the first world on the list where, without watching, owners dig back more of their caches' ended sugar than thieves take (p_s > p_o) in at least 16 of 20 seeds. It did so in all 20, and the items before it in 7, 0 and 0. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; while it remembers one it may walk there, and on arriving it raids only if it remembers at least as much sugar there as the site would give it, taking what it can carry of the first remembered cache still there (or finding them all gone); otherwise it harvests as usual and keeps the memory. An owner digging its own cache comes first. Measured (20 seeds, ticks 1–200): watching flips Andersson and Krebs's condition. Without watching, owners dig back 78 % of their ended caches' sugar and thieves take 22 % (p_s ÷ p_o 3.6); with everyone watching, 39 % and 59 % (0.67), below 1 in all 20 seeds. It flips with a memory of 3, 7 or 13 ticks, but not of 1 or 2 (1.72 and 1.10). Fitness hardly follows. The hoarders' fitness, the share of ticks 1–200 a founder is alive, falls from 0.933 to 0.916 and the cheaters' rises from 0.938 to 0.943, so the hoarders' shortfall grows by 0.023 (in all 20 seeds); but across watching off and memories of 1, 3, 7 and 13 ticks the sign of their advantage matches the condition's in only 62 of 100 runs. Survival moves more: 67 % of the hoarders survive the first winter against 76 % of the cheaters (per founder), where without watching it is 87 % against 75 %. Watching only by the cheaters costs the hoarders 0.012 of advantage (95 % interval 0.008 to 0.017), well under the 0.05 the claim asked for; watching only by the hoarders changes their own fitness by no detectable amount (0.003 lower, interval −0.002 to 0.007).",
            |c| {
                watch_winter(c, 0.5, 0.02, 1.0);
                c.caching.dig_below = DigBelow::Reserve;
            },
        ),
        preset(
            "spatial-scatter",
            "Spatial scatter",
            "Minds 9 spatial hoarding",
            "A fixed spatial episode in theft-winter's world: scattered stores and carried deliveries to a home larder, with probability L=0.0 and Defense target D=0.5. A larger D requires more stored food for the same guard probability. Pending delivery food remains carried; guarding costs a foraging turn. No between-season breeding runs in this episode.",
            |c| {
                theft_winter(c, 0.0);
                c.watching.span = 2;
                c.spatial_hoarding.enabled = true;
                c.spatial_hoarding.larder = 0.0;
                c.spatial_hoarding.guard = false;
            },
        ),
        preset(
            "spatial-larder",
            "Spatial larder",
            "Minds 9 spatial hoarding",
            "A fixed spatial episode in theft-winter's world: scattered stores and carried deliveries to a home larder, with probability L=1.0 and Defense target D=0.5. A larger D requires more stored food for the same guard probability. Pending delivery food remains carried; guarding costs a foraging turn. No between-season breeding runs in this episode.",
            |c| {
                theft_winter(c, 0.0);
                c.watching.span = 2;
                c.spatial_hoarding.enabled = true;
                c.spatial_hoarding.larder = 1.0;
                c.spatial_hoarding.guard = false;
            },
        ),
        preset(
            "spatial-larder-guard",
            "Spatial larder with guard",
            "Minds 9 spatial hoarding",
            "A fixed spatial episode in theft-winter's world: scattered stores and carried deliveries to a home larder, with probability L=1.0 and Defense target D=0.5. A larger D requires more stored food for the same guard probability. Pending delivery food remains carried; guarding costs a foraging turn. No between-season breeding runs in this episode.",
            |c| {
                theft_winter(c, 0.0);
                c.watching.span = 2;
                c.spatial_hoarding.enabled = true;
                c.spatial_hoarding.larder = 1.0;
                c.spatial_hoarding.guard = true;
            },
        ),
        preset(
            "watch-arena",
            "Watching arena: four agents, half watchers",
            "Bugnyar & Kotrschal 2002; Heinrich & Pepper 1998; Minds 8",
            "theft-arena-4's room (4 agents shut in a 6 × 6 room walled on all four sides, with flat sugar through a winter everywhere at once; vision 1–3, a carrying limit of 50) with no cheaters: every agent buries half its surplus where it stands. Arriving on a site, an agent finds each cache of someone else's there with chance 0.25, and takes what it can carry of it. Half the agents, dealt by id, are watchers. An agent who watches and sees another bury (the site on one of the four lattice lines from it, within its vision and not behind an opaque wall) remembers the cache for 7 ticks; while it remembers one it may walk there, and on arriving it raids only if it remembers at least as much sugar there as the site would give it, taking what it can carry of the first remembered cache still there (or finding them all gone); otherwise it harvests as usual and keeps the memory. An owner digging its own cache comes first. Measured (20 seeds, ticks 1–200): every agent survives the winter in every seed. Raids are few: 44 a seed, taking 198 sugar, against 750 found by stumbling; 1.9 % of the caches are pilfered a tick (1.6 % without watching). The room ends poorer: wealth per founder at tick 200 (holdings, caches and stomach) is 46 against 53 without watching (95 % interval of the difference 2.9 to 9.9). The watchers end with 47 per founder and the others with 38, but no change in the same agents' gap was detected against watching off (1.1 larger with watching, 95 % interval −13 to 15).",
            |c| {
                theft_arena(c, 4);
                c.theft.cheaters = 0.0;
                c.watching.on = true;
                c.watching.watchers = 0.5;
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
    // Their "single (Gaussian) sugar mountain", its width and height unstated:
    // as wide as the board (σ = 25), the Sugarscape's usual height of 4.
    c.goods[0].map = Map::Gaussian {
        x: 25,
        y: 25,
        sigma: 25.0,
        height: 4.0,
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
/// sugar grows back 0.25 a tick; 100 agents with metabolism 1, endowment 50
/// and vision 1–6. Nominal inputs (sites with capacity ≥ 1, × 0.25): 305
/// against 305, 225, 145, 109 and 69 sites at radius 10, 8.5, 7, 6 and 5
/// (R 1.00, 1.36, 2.10, 2.80, 4.42); about 112 agents can be fed at R 2.10.
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
/// placement starts about 57 % of agents on the poorer side.
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
    out.extend(crate::zi::presets());
    out.extend(crate::bali::presets());
    out.extend(crate::firms::presets());
    out.extend(crate::line::presets());
    out.extend(crate::tipping::presets());
    out.extend(crate::hoard::presets());
    out.extend(crate::collusion::presets());
    out.extend(crate::auctions::presets());
    out.extend(crate::polarity::presets());
    out.extend(crate::geosim::presets());
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
    fn spatial_presets_extend_theft_winter_with_fixed_episode_traits() {
        for (id, larder, guard) in [
            ("spatial-scatter", 0.0, false),
            ("spatial-larder", 1.0, false),
            ("spatial-larder-guard", 1.0, true),
        ] {
            let c = by_id(id).expect("spatial preset").config;
            assert_eq!(
                (c.population, c.caching.capacity, c.goap.horizon),
                (175, 50, 20)
            );
            assert_eq!(
                (c.seasons.period, c.seasons.winter_divisor, c.caching.share),
                (100, 32, 0.5)
            );
            assert_eq!(
                (
                    c.spatial_hoarding.enabled,
                    c.spatial_hoarding.larder,
                    c.spatial_hoarding.defense,
                    c.spatial_hoarding.guard
                ),
                (true, larder, 0.5, guard)
            );
            assert_eq!(
                (
                    c.theft.find,
                    c.spatial_hoarding.find_larder,
                    c.watching.span
                ),
                (0.25, 0.25, 2)
            );
            let mut reduced = c;
            reduced.spatial_hoarding = Default::default();
            reduced.watching.span = 7;
            assert_eq!(reduced, by_id("theft-winter").unwrap().config);
        }
    }

    #[test]
    fn every_preset_is_valid_and_runs() {
        let presets = all();
        assert_eq!(
            presets
                .iter()
                .filter(|p| !p.config.spatial_hoarding.enabled)
                .count(),
            83
        );
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
            // Their "single (Gaussian) sugar mountain", as wide as the board.
            assert!(matches!(
                c.goods[0].map,
                Map::Gaussian { x: 25, y: 25, sigma, height } if sigma == 25.0 && height == 4.0
            ));
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
    fn the_memory_presets_walk_and_remember() {
        let ids = [
            "mem-open",
            "mem-catchment",
            "mem-walled",
            "mem-seasons",
            "mem-truffles",
            "mem-trapline",
            "mem-mvt",
        ];
        for id in ids {
            let c = by_id(id).unwrap_or_else(|| panic!("{id}")).config;
            c.validate().unwrap_or_else(|e| panic!("{id}: {e:?}"));
            assert_eq!(c.movement.mode, MoveMode::Walk, "{id}");
            assert!(c.memory.span > 0, "{id}: no memory");
        }
        for id in ["mem-truffles", "mem-trapline"] {
            assert!(
                by_id(id).unwrap().config.truffles.share > 0.0,
                "{id}: no truffles"
            );
        }
        let mvt = by_id("mem-mvt").unwrap().config;
        let Map::Peaks { peaks } = &mvt.goods[0].map else {
            panic!("mem-mvt: not a peaks map")
        };
        assert_eq!(peaks.len(), 9, "mem-mvt: nine peaks");
        let mut centers: Vec<(u32, u32)> = peaks.iter().map(|p| (p.x, p.y)).collect();
        centers.sort();
        let mut expected: Vec<(u32, u32)> = (0..3u32)
            .flat_map(|i| (0..3u32).map(move |j| (10 + 20 * i, 10 + 20 * j)))
            .collect();
        expected.sort();
        assert_eq!(centers, expected, "mem-mvt: peak centers");
        assert!(
            peaks.iter().all(|p| p.radius == 4.0 && p.height == 4.0),
            "mem-mvt: peak radius/height"
        );
    }

    #[test]
    fn the_planning_presets_are_minds_3_worlds_under_goap_and_the_mvt_world() {
        // goap-open, goap-truffles and goap-walled are Minds 3's worlds with
        // only the decision rule changed.
        for (goap, mem) in [
            ("goap-open", "mem-open"),
            ("goap-truffles", "mem-truffles"),
            ("goap-walled", "mem-walled"),
        ] {
            let g = by_id(goap).unwrap().config;
            g.validate().unwrap_or_else(|e| panic!("{goap}: {e:?}"));
            let mut m = by_id(mem).unwrap().config;
            m.decision.rule = DecisionRule::Goap;
            assert_eq!(g, m, "{goap}");
        }
        // goap-mvt and mvt-rule share one world, balanced per Task 7: a
        // patch takes in 25 × 0.02 = 0.5 a tick (< one forager's 1), and
        // nine patches take in 1.5 times the population's need.
        let goap = by_id("goap-mvt").unwrap().config;
        let mut mvt = by_id("mvt-rule").unwrap().config;
        assert_eq!(goap.decision.rule, DecisionRule::Goap);
        assert_eq!(mvt.decision.rule, DecisionRule::Mvt);
        mvt.decision.rule = DecisionRule::Goap;
        assert_eq!(goap, mvt);
        goap.validate().unwrap();
        assert_eq!(goap.memory.prior, MemoryPrior::Map);
        assert_eq!((goap.memory.span, goap.memory.share), (1000, 1.0));
        assert_eq!(goap.vision, URange::new(1, 6));
        let w = crate::world::World::new(goap.clone(), 1).unwrap();
        let sites = w.sites.iter().filter(|s| s.capacity[0] >= 1.0).count();
        assert_eq!(sites, 9 * 25);
        let per_patch = 25.0 * goap.growback.rate;
        assert!((per_patch - 0.5).abs() < 1e-12, "{per_patch}");
        let need = f64::from(goap.population);
        assert!((9.0 * per_patch / need - 1.5).abs() < 1e-12);
    }

    #[test]
    fn the_minds_5_presets_share_their_worlds_and_name_minds_5() {
        use crate::config::{Caching, CachingRule, SeasonMode};
        // The winter presets differ only in the caching rule (or mixed).
        let base = by_id("cache-winter-none").unwrap().config;
        assert_eq!(base.population, 175);
        assert_eq!(base.goods[0].metabolism, URange::new(1, 1));
        assert!(base.seasons.enabled && base.seasons.mode == SeasonMode::Global);
        assert_eq!(
            (base.seasons.period, base.seasons.winter_divisor),
            (100, 32)
        );
        assert_eq!((base.caching.capacity, base.goap.horizon), (50, 20));
        assert_eq!(base.movement.mode, MoveMode::Walk);
        assert_eq!((base.memory.span, base.memory.share), (100, 0.5));
        let walk = by_id("walk-capacity").unwrap().config;
        assert_eq!(
            walk.goods[0].map, base.goods[0].map,
            "walk-capacity's landscape"
        );
        for (id, rule, mixed) in [
            ("cache-winter-even", CachingRule::Even, false),
            ("cache-winter-compensate", CachingRule::Compensate, false),
            ("cache-winter-plan", CachingRule::Plan, false),
            ("cache-winter-mixed", CachingRule::None, true),
        ] {
            let mut c = by_id(id).unwrap().config;
            assert_eq!((c.caching.rule, c.caching.mixed), (rule, mixed), "{id}");
            c.caching.rule = CachingRule::None;
            c.caching.mixed = false;
            assert_eq!(c, base, "{id}");
        }
        // Mixed deals a quarter of the founders to each rule.
        let w = World::new(by_id("cache-winter-mixed").unwrap().config, 1).unwrap();
        for rule in Caching::MIXED {
            let n = w.agents().filter(|a| a.caching_rule == rule).count();
            assert!((43..=44).contains(&n), "{rule:?}: {n}");
        }
        // Central: near and far differ only in where the homes are; linear
        // is near with instant growback.
        let near = by_id("central-near").unwrap().config;
        let mut far = by_id("central-far").unwrap().config;
        let mut linear = by_id("central-linear").unwrap().config;
        assert!(near.central.enabled && near.decision.rule == DecisionRule::Mvt);
        assert_eq!(
            (near.placement, far.placement),
            (
                Placement::Block {
                    x: 37,
                    y: 0,
                    width: 1,
                    height: 30
                },
                Placement::Block {
                    x: 25,
                    y: 0,
                    width: 1,
                    height: 30
                }
            )
        );
        far.placement = near.placement;
        assert_eq!(far, near);
        assert!(linear.growback.instant && !near.growback.instant);
        linear.growback.instant = false;
        assert_eq!(linear, near);
        // The labs are the rig with mixed rules.
        for (id, n) in [("cache-raby", 8), ("cache-amodio", 6)] {
            let c = by_id(id).unwrap().config;
            assert!(c.caching.mixed && c.lab.is_some(), "{id}");
            assert_eq!(c.population, n, "{id}");
        }
        for id in [
            "cache-winter-none",
            "cache-winter-even",
            "cache-winter-compensate",
            "cache-winter-plan",
            "cache-winter-mixed",
            "central-near",
            "central-far",
            "central-linear",
            "cache-raby",
            "cache-amodio",
        ] {
            let p = by_id(id).unwrap();
            assert!(p.source.contains("Minds 5"), "{id}: {}", p.source);
            p.config
                .validate()
                .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        }
    }

    #[test]
    fn the_minds_6_presets_share_their_worlds_and_name_minds_6() {
        use crate::config::{CachingRule, Loot};
        // The winter theft presets are cache-winter-even with theft on at
        // 0.25 and a share of cheaters; nothing else changes.
        let even = by_id("cache-winter-even").unwrap().config;
        for (id, cheaters) in [
            ("theft-winter", 0.0),
            ("theft-winter-quarter", 0.25),
            ("theft-winter-half", 0.5),
        ] {
            let mut c = by_id(id).unwrap().config;
            assert_eq!((c.theft.find, c.theft.cheaters), (0.25, cheaters), "{id}");
            assert!(c.theft.owner_memory && c.theft.loot == Loot::Keep, "{id}");
            assert_eq!(c.caching.bury_cost, 0.0, "{id}");
            c.theft = Default::default();
            assert_eq!(c, even, "{id}");
        }
        // The arena: n agents in a k × k room closed by walls, with the same
        // standing sugar and regrowth per agent at every n.
        for (id, n, side) in [
            ("theft-arena-2", 2u32, 4u32),
            ("theft-arena-4", 4, 6),
            ("theft-arena-8", 8, 8),
        ] {
            let c = by_id(id).unwrap().config;
            assert_eq!(c.population, n, "{id}");
            assert_eq!((c.width, c.height), (side + 2, side + 2), "{id}");
            assert_eq!((c.theft.find, c.theft.cheaters), (0.25, 0.5), "{id}");
            assert_eq!(c.caching.rule, CachingRule::Even, "{id}");
            assert_eq!(c.movement.mode, MoveMode::Walk, "{id}");
            assert_eq!(c.goods.len(), 1, "{id}");
            let Map::Flat { capacity } = c.goods[0].map else {
                panic!("{id}: flat sugar");
            };
            let w = World::new(c.clone(), 1).unwrap();
            let open: Vec<_> = w.sites.iter().filter(|s| s.capacity[0] > 0.0).collect();
            assert_eq!(open.len() as u32, side * side, "{id}: the room's sites");
            let per_agent = |x: f64| x * f64::from(side * side) / f64::from(n);
            assert!((per_agent(capacity) - 32.0).abs() < 1e-9, "{id}");
            assert!((per_agent(c.growback.rate) - 2.4).abs() < 1e-9, "{id}");
            assert_eq!(w.agents().filter(|a| a.cheater).count() as u32, n / 2);
            // Vision is 1 to half the room's side.
            assert_eq!(c.vision, URange::new(1, side / 2), "{id}");
            // The walls: rows 0 and k + 1 and columns 0 and k + 1, the
            // (k + 2)-square's border, 4k + 4 sites in all; the room is the
            // centered k × k block, every site of it open.
            let walls = (0..w.torus.len())
                .filter(|&i| w.is_wall(w.torus.pos(i)))
                .count() as u32;
            assert_eq!(walls, 4 * side + 4, "{id}");
            for i in 0..w.torus.len() {
                let p = w.torus.pos(i);
                let border = p.x == 0 || p.y == 0 || p.x == side + 1 || p.y == side + 1;
                assert_eq!(w.is_wall(p), border, "{id}: {p:?}");
                assert_eq!(w.sites[i].capacity[0] > 0.0, !border, "{id}: {p:?}");
            }
            // Every agent starts inside the room, off the walls.
            for a in w.agents() {
                assert!((1..=side).contains(&a.pos.x), "{id}: {:?}", a.pos);
                assert!((1..=side).contains(&a.pos.y), "{id}: {:?}", a.pos);
                assert!(!w.is_wall(a.pos), "{id}");
            }
        }
        for id in [
            "theft-winter",
            "theft-winter-quarter",
            "theft-winter-half",
            "theft-arena-2",
            "theft-arena-4",
            "theft-arena-8",
        ] {
            let p = by_id(id).unwrap();
            assert!(p.source.contains("Minds 6"), "{id}: {}", p.source);
            p.config
                .validate()
                .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        }
    }

    #[test]
    fn the_minds_8_presets_add_watching_to_the_minds_6_worlds_and_name_minds_8() {
        use crate::config::{RaidWhen, Watching};
        // (id, base, cheaters, find, watchers): each is a Minds 6 world with
        // watching on (span 7, raid_when always) and nothing else changed.
        for (id, base, cheaters, find, watchers) in [
            ("watch-winter", "theft-winter", 0.0, 0.0, 1.0),
            ("watch-winter-stumble", "theft-winter", 0.0, 0.25, 1.0),
            ("watch-half", "theft-winter-half", 0.5, 0.25, 1.0),
            ("watch-scroungers", "theft-winter", 0.0, 0.0, 0.5),
            ("watch-scroungers-only", "theft-winter-half", 0.5, 0.0, 0.5),
            ("watch-arena", "theft-arena-4", 0.0, 0.25, 0.5),
        ] {
            let p = by_id(id).unwrap();
            let mut c = p.config.clone();
            assert_eq!((c.theft.cheaters, c.theft.find), (cheaters, find), "{id}");
            assert_eq!(
                c.watching,
                Watching {
                    on: true,
                    span: 7,
                    watchers,
                    raid_when: RaidWhen::Always,
                    ..Watching::default()
                },
                "{id}"
            );
            let mut b = by_id(base).unwrap().config;
            c.watching = Default::default();
            c.theft.find = 0.0;
            c.theft.cheaters = 0.0;
            b.theft.find = 0.0;
            b.theft.cheaters = 0.0;
            assert_eq!(c, b, "{id} is {base} with watching");
            assert!(p.source.contains("Minds 8"), "{id}: {}", p.source);
            assert!(p.source.contains("Bugnyar & Kotrschal 2002"), "{id}");
            assert!(p.source.contains("Heinrich & Pepper 1998"), "{id}");
            assert_eq!(
                p.source.contains("Andersson & Krebs 1978"),
                id == "watch-half",
                "{id}"
            );
            assert_eq!(
                p.source.contains("Barnard & Sibly 1981"),
                id.starts_with("watch-scroungers"),
                "{id}"
            );
            p.config
                .validate()
                .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        }
        // Pure scroungers: at equal shares the watchers are the cheaters.
        let w = World::new(by_id("watch-scroungers-only").unwrap().config, 1).unwrap();
        assert!(w.agents().all(|a| a.watches == a.cheater));
        assert_eq!(w.agents().filter(|a| a.watches).count(), 87);
        let w = World::new(by_id("watch-arena").unwrap().config, 1).unwrap();
        assert_eq!(w.agents().filter(|a| a.watches).count(), 2);
        assert!(w.agents().all(|a| !a.cheater));
    }

    #[test]
    fn watch_scroungers_forgo_is_the_only_scrounger_with_forgo_and_eaten_loot() {
        use crate::config::{Loot, Scrounge};
        let p = by_id("watch-scroungers-forgo").unwrap();
        let c = &p.config;
        assert_eq!((c.theft.cheaters, c.theft.find), (0.5, 0.0));
        assert_eq!(c.theft.loot, Loot::Eat);
        assert!(c.watching.on && c.watching.watchers == 0.5);
        assert_eq!(c.watching.scrounge, Scrounge::Forgo);
        assert_eq!(c.watching.who, crate::config::Who::Share);
        for id in ["watch-scroungers-only", "watch-winter", "watch-arena"] {
            let o = by_id(id).unwrap().config;
            assert_eq!(o.watching.scrounge, Scrounge::Harvest, "{id}");
            assert_eq!(o.theft.loot, Loot::Keep, "{id}");
        }
        // Otherwise watch-scroungers-only's world.
        let mut b = by_id("watch-scroungers-only").unwrap().config;
        b.watching.scrounge = Scrounge::Forgo;
        b.theft.loot = Loot::Eat;
        assert_eq!(*c, b);
        for s in [
            "Bugnyar & Kotrschal 2002",
            "Barnard & Sibly 1981",
            "Vickery et al. 1991",
            "Minds 8",
        ] {
            assert!(p.source.contains(s), "{}", p.source);
        }
    }

    #[test]
    fn watch_ak_is_the_calibrated_world_with_everyone_watching() {
        use crate::config::{DigBelow, Who};
        let p = by_id("watch-ak").unwrap();
        let c = &p.config;
        assert_eq!((c.theft.cheaters, c.theft.find), (0.5, 0.02));
        assert_eq!(c.caching.dig_below, DigBelow::Reserve);
        assert!(c.watching.on);
        assert_eq!((c.watching.who, c.watching.watchers), (Who::Share, 1.0));
        let mut b = by_id("theft-winter-half").unwrap().config;
        b.theft.find = 0.02;
        b.caching.dig_below = DigBelow::Reserve;
        b.watching = crate::config::Watching {
            on: true,
            ..Default::default()
        };
        assert_eq!(
            *c, b,
            "theft-winter-half at find 0.02, dig below R, watching on"
        );
        for other in all().iter().filter(|q| q.id != "watch-ak") {
            assert_eq!(
                other.config.caching.dig_below,
                DigBelow::Half,
                "{}",
                other.id
            );
        }
        for s in [
            "Andersson & Krebs 1978",
            "Bugnyar & Kotrschal 2002",
            "Minds 8",
        ] {
            assert!(p.source.contains(s), "{}", p.source);
        }
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
