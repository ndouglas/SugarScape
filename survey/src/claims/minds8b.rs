//! Minds 8, second round: watching
//! (docs/superpowers/specs/2026-10-01-minds-8-second-round-design.md,
//! binding word for word).
//!
//! Pairing as in Minds 3–8: every configuration compared runs the same
//! seeds and is judged on per-seed values or per-seed differences. Every run
//! lasts 200 ticks: a summer (ticks 0–99) and the first winter (100–199). A
//! tick stands for a day.
//!
//! **Fitness** (the spec's pre-mortem, "Floors, ceilings and caps checked"),
//! of a group of founders (hoarders against cheaters by `Agent.cheater`,
//! watchers against the rest by `Agent.watches`, or everyone):
//! - **Field worlds:** agent-ticks alive per founder over ticks 1–200,
//!   divided by 200. After each of the 200 steps (the world at ticks 1, 2,
//!   …, 200) every living founder of the group adds 1; the sum is divided by
//!   the group's founders and by 200. A founder alive at 200 scores 1, one
//!   dead in the first step 0, so the measure is continuous where survival
//!   at 200 floors near 0.46.
//! - **Arena worlds:** wealth per founder at tick 200, Minds 6's arena rule:
//!   holdings + caches + stomach of the group's living founders at 200 ÷ its
//!   founders (the dead as 0). In an arena nearly every agent survives, so
//!   ticks alive can't discriminate there.
//!
//! Only founders count: an agent born during the run (none is, in these
//! worlds) is in no group.
//!
//! **p_s and p_o** are Minds 6's (`minds6::p_s`, `minds6::p_o`):
//! amount-weighted from the tick events, p_s = dug ÷ (dug + pilfered) and
//! p_o = pilfered ÷ (dug + pilfered + lost with a dead owner). Sugar still
//! buried at tick 200 is excluded.
//!
//! **Claim 2's precondition: the calibration** (`watch-ak.calibration`, and
//! `survey --calibration`, which writes `survey/out/minds8b-calibration.md`).
//! Its rule was fixed in the spec and committed here before it ran:
//! - Take the first world from this list in which p_s > p_o holds in ≥ 16 of
//!   20 seeds (seeds 1–20) without watching:
//!   1. `theft-arena-2`;
//!   2. `theft-winter-half` at `find` 0.02;
//!   3. `theft-winter-half` at `find` 0.05;
//!   4. `theft-winter-half` at `find` 0.02, with owners digging below their
//!      whole reserve (the survey probe `World::probe_dig_at_reserve`,
//!      since made the setting `caching.dig_below: reserve`, which runs
//!      identically and is what these items set);
//!   5. the same at `find` 0.05.
//! - A seed whose p_s or p_o is undefined (no sugar ended) does not have
//!   p_s > p_o.
//! - Every item is run and reported, not only up to the first that
//!   qualifies; the rule picks the first.
//! - That world becomes `watch-ak` (with watching on for everyone). If none
//!   qualifies, claims 2a–2d are Untestable, not failed.
//!
//! The calibration always runs seeds 1–20, whatever `--seeds` says. Its
//! claim reads Holds when a world qualifies (naming it) and Untestable when
//! none does.

use std::collections::HashSet;
use std::fmt::Write as _;

use sugarscape_core::agent::{Agent, AgentId};
use sugarscape_core::config::{Config, DigBelow};
use sugarscape_core::world::World;

use crate::claim::{untestable, Claim, Outcome, Source, Verdict};
use crate::claims::minds6::{nan_div, p_o, p_s, wealth};
use crate::runner::{each_seed, preset};

const SPEC: &str = "docs/superpowers/specs/2026-10-01-minds-8-second-round-design.md";

/// Every run: a summer and the first winter.
const TICKS: u64 = 200;
/// Claims 1 and 2 (and the calibration) run seeds 1–20.
const SEEDS_1_2: u64 = 20;
/// The thresholds: ≥ 16 of 20 seeds.
const QUALIFY: usize = 16;

/// Group indices: by caching (hoarders, cheaters) and by watching
/// (watchers, others).
pub(crate) const H: usize = 0;
pub(crate) const C: usize = 1;
// The watching groups and the advantages are for claims 2 and 3's judges,
// which follow this commit; until then only the tests read them.
#[allow(dead_code)]
pub(crate) const W: usize = 0;
#[allow(dead_code)]
pub(crate) const O: usize = 1;

/// Which fitness a world is judged by (see the module's list).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fitness {
    /// Agent-ticks alive per founder over ticks 1–200, ÷ 200.
    Field,
    /// Wealth per founder at tick 200.
    Arena,
}

// ------------------------------------------------------------------- the run

/// One run's measures. Arrays of two are by group: `_c` by caching (H, C),
/// `_w` by watching (W, O).
#[derive(Clone, Debug, Default)]
pub(crate) struct Run {
    pub(crate) founders_c: [f64; 2],
    pub(crate) founders_w: [f64; 2],
    /// Σ over ticks 1–200 of the group's living founders.
    pub(crate) ticks_alive_c: [f64; 2],
    pub(crate) ticks_alive_w: [f64; 2],
    /// Σ holdings + caches + stomach of the group's living founders at 200.
    pub(crate) wealth200_c: [f64; 2],
    pub(crate) wealth200_w: [f64; 2],
    /// Sugar dug by its owner, pilfered, and lost with a dead owner, over
    /// the run (tick events).
    pub(crate) dug: f64,
    pub(crate) pilfered: f64,
    pub(crate) lost: f64,
}

/// Field fitness from a group's agent-ticks alive and founders.
fn field(ticks_alive: f64, founders: f64) -> f64 {
    nan_div(ticks_alive, founders) / TICKS as f64
}

/// Arena fitness from a group's wealth at 200 and founders.
fn arena(wealth200: f64, founders: f64) -> f64 {
    nan_div(wealth200, founders)
}

impl Run {
    fn of(kind: Fitness, ticks: [f64; 2], wealth: [f64; 2], founders: [f64; 2], g: usize) -> f64 {
        match kind {
            Fitness::Field => field(ticks[g], founders[g]),
            Fitness::Arena => arena(wealth[g], founders[g]),
        }
    }
    /// Fitness of a caching group (H or C).
    pub(crate) fn fit_c(&self, kind: Fitness, g: usize) -> f64 {
        Self::of(
            kind,
            self.ticks_alive_c,
            self.wealth200_c,
            self.founders_c,
            g,
        )
    }
    /// Fitness of a watching group (W or O).
    #[allow(dead_code)]
    pub(crate) fn fit_w(&self, kind: Fitness, g: usize) -> f64 {
        Self::of(
            kind,
            self.ticks_alive_w,
            self.wealth200_w,
            self.founders_w,
            g,
        )
    }
    /// Fitness of everyone (the world's).
    #[allow(dead_code)]
    pub(crate) fn fit_all(&self, kind: Fitness) -> f64 {
        let sum = |x: [f64; 2]| x[0] + x[1];
        match kind {
            Fitness::Field => field(sum(self.ticks_alive_c), sum(self.founders_c)),
            Fitness::Arena => arena(sum(self.wealth200_c), sum(self.founders_c)),
        }
    }
    /// The hoarders' advantage: hoarder − cheater fitness.
    #[allow(dead_code)]
    pub(crate) fn hoarder_adv(&self, kind: Fitness) -> f64 {
        self.fit_c(kind, H) - self.fit_c(kind, C)
    }
    /// The watchers' advantage: watcher − other fitness.
    #[allow(dead_code)]
    pub(crate) fn watcher_adv(&self, kind: Fitness) -> f64 {
        self.fit_w(kind, W) - self.fit_w(kind, O)
    }
    pub(crate) fn p_s(&self) -> f64 {
        p_s(self.dug, self.pilfered)
    }
    pub(crate) fn p_o(&self) -> f64 {
        p_o(self.dug, self.pilfered, self.lost)
    }
}

pub(crate) fn run(mut w: World) -> Run {
    let gc = |a: &Agent| usize::from(a.cheater);
    let gw = |a: &Agent| usize::from(!a.watches);
    let mut r = Run::default();
    let founders: HashSet<AgentId> = w.agents().map(|a| a.id).collect();
    for a in w.agents() {
        r.founders_c[gc(a)] += 1.0;
        r.founders_w[gw(a)] += 1.0;
    }
    while w.tick < TICKS {
        w.step();
        let e = w.events();
        r.dug += e.dug;
        r.pilfered += e.pilfered;
        r.lost += e.cache_lost;
        for a in w.agents().filter(|a| founders.contains(&a.id)) {
            r.ticks_alive_c[gc(a)] += 1.0;
            r.ticks_alive_w[gw(a)] += 1.0;
            if w.tick == TICKS {
                r.wealth200_c[gc(a)] += wealth(a);
                r.wealth200_w[gw(a)] += wealth(a);
            }
        }
    }
    r
}

pub(crate) fn runs(c: &Config, seeds: &[u64]) -> Vec<Run> {
    each_seed(c, seeds, run)
}

// ----------------------------------------------------------- the calibration

/// An item of claim 2's calibration list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Candidate {
    pub(crate) preset: &'static str,
    /// `theft.find` set, or the preset's.
    pub(crate) find: Option<f64>,
    /// Owners dig below their whole reserve (`caching.dig_below:
    /// reserve`), the spec's "survey probe" as a setting.
    pub(crate) dig_at_reserve: bool,
    pub(crate) fitness: Fitness,
}

impl Candidate {
    pub(crate) fn config(&self) -> Config {
        let mut c = preset(self.preset);
        if let Some(f) = self.find {
            c.theft.find = f;
        }
        if self.dig_at_reserve {
            c.caching.dig_below = DigBelow::Reserve;
        }
        c
    }
    pub(crate) fn label(&self) -> String {
        let mut s = format!("`{}`", self.preset);
        if let Some(f) = self.find {
            write!(s, " at `find` {f}").unwrap();
        }
        if self.dig_at_reserve {
            s.push_str(", owners digging below their whole reserve (`dig_below: reserve`)");
        }
        s
    }
}

const fn field_item(find: f64, dig_at_reserve: bool) -> Candidate {
    Candidate {
        preset: "theft-winter-half",
        find: Some(find),
        dig_at_reserve,
        fitness: Fitness::Field,
    }
}

/// The spec's list, in order.
pub(crate) const CALIBRATION: [Candidate; 5] = [
    Candidate {
        preset: "theft-arena-2",
        find: None,
        dig_at_reserve: false,
        fitness: Fitness::Arena,
    },
    field_item(0.02, false),
    field_item(0.05, false),
    field_item(0.02, true),
    field_item(0.05, true),
];

/// A seed has the condition when p_s > p_o (an undefined value doesn't).
fn has_condition(ps: f64, po: f64) -> bool {
    ps > po
}

/// The seeds (of each item's runs) with p_s > p_o.
fn qualifying(per_seed: &[(f64, f64)]) -> usize {
    per_seed
        .iter()
        .filter(|&&(ps, po)| has_condition(ps, po))
        .count()
}

/// The rule: the first item whose count is at least 16 (of 20).
fn chosen(counts: &[usize]) -> Option<usize> {
    counts.iter().position(|&n| n >= QUALIFY)
}

/// Each item's runs, seeds 1–20, in the list's order.
type Calibration = Vec<(Candidate, Vec<Run>)>;

/// Per-seed (p_s, p_o).
fn pairs(runs: &[Run]) -> Vec<(f64, f64)> {
    runs.iter().map(|r| (r.p_s(), r.p_o())).collect()
}

fn calibrate() -> Calibration {
    let seeds: Vec<u64> = (1..=SEEDS_1_2).collect();
    CALIBRATION
        .iter()
        .map(|item| {
            let c = item.config();
            assert!(!c.watching.on, "the calibration runs without watching");
            (*item, runs(&c, &seeds))
        })
        .collect()
}

fn verdict_line(cal: &Calibration) -> (Option<usize>, String) {
    let counts: Vec<usize> = cal.iter().map(|(_, v)| qualifying(&pairs(v))).collect();
    let pick = chosen(&counts);
    let line = match pick {
        Some(i) => format!(
            "item {} qualifies ({} of {} seeds): {}",
            i + 1,
            counts[i],
            cal[i].1.len(),
            cal[i].0.label()
        ),
        None => format!(
            "no item qualifies (counts {}; need {QUALIFY} of {SEEDS_1_2}): claims 2a–2d are Untestable",
            counts
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    (pick, line)
}

fn fmt(x: f64) -> String {
    if x.is_nan() {
        "undefined".into()
    } else {
        format!("{x:.4}")
    }
}

/// The calibration's full output, as Markdown: every item's per-seed p_s,
/// p_o and qualifying count, and the rule's pick.
fn markdown(cal: &Calibration) -> String {
    let mut s = String::new();
    writeln!(s, "# Minds 8b: claim 2's calibration\n").unwrap();
    writeln!(
        s,
        "Generated by `cargo run --release -- --calibration` in `survey/` \
         (`survey/src/claims/minds8b.rs`). The rule, from {SPEC} (\"The precondition\"), \
         was committed before this ran: take the first world from the list in which \
         p_s > p_o holds in ≥ {QUALIFY} of {SEEDS_1_2} seeds (seeds 1–{SEEDS_1_2}) without \
         watching. p_s and p_o are Minds 6's, amount-weighted from the tick events, \
         with sugar still buried at tick 200 excluded; an undefined value (no sugar \
         ended) doesn't count as p_s > p_o. Fitness (the hoarders' and the \
         cheaters', reported per seed, not part of the rule) is the world's kind: wealth \
         per founder at tick 200 in the arena, agent-ticks alive per founder over ticks \
         1–200 ÷ 200 in the field.\n"
    )
    .unwrap();
    let (_, line) = verdict_line(cal);
    writeln!(s, "**Result:** {line}.\n").unwrap();
    writeln!(
        s,
        "| # | world | seeds with p_s > p_o | qualifies |\n|---|---|---|---|"
    )
    .unwrap();
    for (i, (item, v)) in cal.iter().enumerate() {
        let n = qualifying(&pairs(v));
        writeln!(
            s,
            "| {} | {} | {n} of {} | {} |",
            i + 1,
            item.label(),
            v.len(),
            if n >= QUALIFY { "yes" } else { "no" }
        )
        .unwrap();
    }
    for (i, (item, v)) in cal.iter().enumerate() {
        writeln!(s, "\n## {}. {}\n", i + 1, item.label()).unwrap();
        writeln!(
            s,
            "| seed | p_s | p_o | p_s > p_o | hoarder fitness | cheater fitness |\n\
             |---|---|---|---|---|---|"
        )
        .unwrap();
        for (k, r) in v.iter().enumerate() {
            let (ps, po) = (r.p_s(), r.p_o());
            writeln!(
                s,
                "| {} | {} | {} | {} | {} | {} |",
                k + 1,
                fmt(ps),
                fmt(po),
                if has_condition(ps, po) { "yes" } else { "no" },
                fmt(r.fit_c(item.fitness, H)),
                fmt(r.fit_c(item.fitness, C)),
            )
            .unwrap();
        }
    }
    s
}

/// `survey --calibration`: runs the calibration, writes
/// `survey/out/minds8b-calibration.md` and returns its text.
pub(crate) fn calibration_report() -> String {
    let text = markdown(&calibrate());
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/out");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(format!("{dir}/minds8b-calibration.md"), &text).unwrap();
    text
}

fn calibration_claim(_seeds: &[u64]) -> Outcome {
    let cal = calibrate();
    let (pick, line) = verdict_line(&cal);
    let counts = cal
        .iter()
        .enumerate()
        .map(|(i, (_, v))| format!("{}: {}", i + 1, qualifying(&pairs(v))))
        .collect::<Vec<_>>()
        .join(", ");
    match pick {
        Some(_) => Outcome {
            verdict: Verdict::Holds,
            measured: line,
            detail: format!("seeds with p_s > p_o by item: {counts}"),
        },
        None => untestable(&line).with(&format!("seeds with p_s > p_o by item: {counts}")),
    }
}

pub fn claims() -> Vec<Claim> {
    vec![Claim {
        id: "watch-ak.calibration",
        item: "watch-ak",
        source: Source::Comment,
        citation: SPEC,
        text: "Claim 2's precondition: some world on the calibration list (theft-arena-2; theft-winter-half at find 0.02, 0.05; the same with owners digging below their whole reserve) has p_s > p_o in at least 16 of 20 seeds without watching; the first such world is watch-ak's",
        check: calibration_claim,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hand_built() -> Run {
        Run {
            founders_c: [10.0, 5.0],
            founders_w: [4.0, 11.0],
            // Hoarders: 10 founders, 1000 agent-ticks (half the 2000
            // possible); cheaters: 5 founders, all alive throughout.
            ticks_alive_c: [1000.0, 1000.0],
            ticks_alive_w: [600.0, 1400.0],
            wealth200_c: [300.0, 50.0],
            wealth200_w: [100.0, 250.0],
            dug: 30.0,
            pilfered: 10.0,
            lost: 10.0,
        }
    }

    #[test]
    fn field_fitness_is_agent_ticks_alive_per_founder_over_200() {
        let r = hand_built();
        assert_eq!(r.fit_c(Fitness::Field, H), 0.5);
        assert_eq!(r.fit_c(Fitness::Field, C), 1.0);
        assert_eq!(r.hoarder_adv(Fitness::Field), -0.5);
        assert_eq!(r.fit_w(Fitness::Field, W), 600.0 / 4.0 / 200.0);
        assert_eq!(r.fit_w(Fitness::Field, O), 1400.0 / 11.0 / 200.0);
        assert_eq!(r.fit_all(Fitness::Field), 2000.0 / 15.0 / 200.0);
    }

    #[test]
    fn arena_fitness_is_wealth_per_founder_at_200() {
        let r = hand_built();
        assert_eq!(r.fit_c(Fitness::Arena, H), 30.0);
        assert_eq!(r.fit_c(Fitness::Arena, C), 10.0);
        assert_eq!(r.hoarder_adv(Fitness::Arena), 20.0);
        assert_eq!(r.watcher_adv(Fitness::Arena), 25.0 - 250.0 / 11.0);
        assert_eq!(r.fit_all(Fitness::Arena), 350.0 / 15.0);
    }

    #[test]
    fn an_empty_group_has_no_fitness() {
        let r = Run::default();
        assert!(r.fit_c(Fitness::Field, H).is_nan());
        assert!(r.fit_w(Fitness::Arena, W).is_nan());
        assert!(r.fit_all(Fitness::Field).is_nan());
    }

    #[test]
    fn p_s_and_p_o_are_minds_6s() {
        let r = hand_built();
        assert_eq!(r.p_s(), 0.75, "dug ÷ (dug + pilfered)");
        assert_eq!(r.p_o(), 0.2, "pilfered ÷ everything ended");
        assert!(Run::default().p_s().is_nan());
    }

    #[test]
    fn the_rule_takes_the_first_item_with_16_of_20() {
        assert_eq!(chosen(&[7, 15, 16, 20, 20]), Some(2));
        assert_eq!(chosen(&[16, 20, 0, 0, 0]), Some(0));
        assert_eq!(chosen(&[15, 15, 15, 15, 15]), None);
        assert_eq!(chosen(&[0, 0, 0, 0, 16]), Some(4));
    }

    #[test]
    fn a_seed_has_the_condition_only_when_p_s_exceeds_p_o() {
        let v = [
            (0.6, 0.4),
            (0.5, 0.5),
            (0.1, 0.9),
            (f64::NAN, f64::NAN),
            (f64::NAN, 0.0),
            (1.0, 0.0),
        ];
        assert_eq!(qualifying(&v), 2, "ties and undefined values don't count");
    }

    #[test]
    fn the_list_is_the_specs() {
        let labels: Vec<String> = CALIBRATION.iter().map(Candidate::label).collect();
        assert_eq!(
            labels,
            [
                "`theft-arena-2`",
                "`theft-winter-half` at `find` 0.02",
                "`theft-winter-half` at `find` 0.05",
                "`theft-winter-half` at `find` 0.02, owners digging below their whole reserve (`dig_below: reserve`)",
                "`theft-winter-half` at `find` 0.05, owners digging below their whole reserve (`dig_below: reserve`)",
            ]
        );
        assert_eq!(CALIBRATION[0].config(), preset("theft-arena-2"));
        assert_eq!(CALIBRATION[0].fitness, Fitness::Arena);
        for item in &CALIBRATION[1..] {
            let c = item.config();
            c.validate().expect("valid");
            assert!(!c.watching.on);
            let mut back = c.clone();
            back.theft.find = preset("theft-winter-half").theft.find;
            back.caching.dig_below = DigBelow::Half;
            assert_eq!(back, preset("theft-winter-half"));
            assert_eq!(item.fitness, Fitness::Field);
        }
    }

    #[test]
    fn watch_ak_is_the_chosen_item_with_everyone_watching() {
        let mut c = preset("watch-ak");
        assert!(c.watching.on);
        c.watching = Default::default();
        assert_eq!(c, CALIBRATION[3].config());
    }

    #[test]
    fn a_world_where_nobody_dies_has_field_fitness_1() {
        // The two-agent arena: both survive the winter in every seed.
        let r = &runs(&preset("theft-arena-2"), &[1])[0];
        assert_eq!(r.founders_c, [1.0, 1.0]);
        assert_eq!(r.fit_c(Fitness::Field, H), 1.0);
        assert_eq!(r.fit_c(Fitness::Field, C), 1.0);
        assert!(r.fit_all(Fitness::Arena) > 0.0);
        assert_eq!(
            r.wealth200_c[H] + r.wealth200_c[C],
            r.wealth200_w[W] + r.wealth200_w[O]
        );
    }
}
