//! Minds 4: GOAP and the marginal-value rule
//! (docs/superpowers/specs/2026-09-28-minds-4-goap-design.md).
//!
//! Pairing as in Minds 3: the within-world comparisons (rememberers against
//! others) are paired per seed by construction, and every configuration
//! compared (GOAP, the marginal-value rule, rule M, the value shortlist)
//! runs the same seeds. The judges and thresholds are Minds 3's: a
//! per-seed slope above 0 in at least 80 % of seeds; overstaying at over
//! half of departures in at least 80 % of seeds; memory paying when the
//! paired difference is above 0 in at least 80 % of seeds.
//!
//! The spacing worlds are `goap-mvt`'s at spacing s (nine peaks at
//! (s/2 + s·i, s/2 + s·j) on a 3s × 3s torus), keeping Task 7's balance:
//! the patches, growback 0.02 and 3 agents don't change with s, so a patch
//! still takes in 0.5 a tick and the nine 1.5 times the need. Vision 1–6
//! and the prior map at every spacing, so s = 20 is the preset.

use std::collections::HashMap;
use sugarscape_core::config::{Config, DecisionRule, Map, Peak, Shortlist};

use sugarscape_core::agent::{Agent, AgentId};
use sugarscape_core::geometry::Pos;
use sugarscape_core::landscape::patch_of;
use sugarscape_core::world::World;

use crate::claim::{range, Claim, Outcome, Source};
use crate::claims::minds1::slope;
use crate::claims::minds2::paired_greater;
use crate::claims::minds3::{
    col, describe, measure, med_or_nan, q_or_nan, residence_medians, residence_slopes, visits,
    Groups, Visits, MVT_TICKS, SPACINGS,
};
use crate::runner::{each_seed, preset, series, window_mean};
use crate::stats::{self, median};

const SPEC: &str = "docs/superpowers/specs/2026-09-28-minds-4-goap-design.md";

/// Index of s = 20 (the presets) in `SPACINGS`.
const PRESET_SPACING: usize = 2;

/// `goap-mvt`'s world at spacing `s` under `rule`.
fn tori(s: u32, rule: DecisionRule) -> Config {
    let mut c = preset("goap-mvt");
    let Map::Peaks { peaks } = &c.goods[0].map else {
        unreachable!("goap-mvt is a peaks map")
    };
    let (radius, height) = (peaks[0].radius, peaks[0].height);
    c.width = 3 * s;
    c.height = 3 * s;
    c.goods[0].map = Map::Peaks {
        peaks: (0..3u32)
            .flat_map(|i| {
                (0..3u32).map(move |j| Peak {
                    x: s / 2 + s * i,
                    y: s / 2 + s * j,
                    radius,
                    height,
                })
            })
            .collect(),
    };
    c.decision.rule = rule;
    c
}

/// GOAP with the spec's original shortlist: the K best known sites by value.
fn value_shortlist(s: u32) -> Config {
    let mut c = tori(s, DecisionRule::Goap);
    c.goap.shortlist = Shortlist::Value;
    c
}

fn rule_name(rule: DecisionRule) -> &'static str {
    match rule {
        DecisionRule::Goap => "GOAP",
        DecisionRule::Mvt => "the marginal-value rule",
        _ => "rule M",
    }
}

pub(crate) fn list(v: &[f64], digits: usize) -> String {
    v.iter()
        .map(|x| format!("{x:.digits$}"))
        .collect::<Vec<_>>()
        .join(" ")
}

// --------------------------------------------------------------------- usage

/// How often GOAP agents planned or fell back, summed over a run's ticks
/// (`World::events` after each step; agent-ticks are the living counts after
/// each tick, as the stats series count them).
#[derive(Default, Clone, Copy)]
struct Usage {
    agent_ticks: u64,
    plans: u64,
    plans_by_rememberers: u64,
    plans_with_remembered: u64,
    fallback_short: u64,
    fallback_limit: u64,
}

impl Usage {
    fn add(&mut self, w: &World) {
        let e = w.events();
        self.agent_ticks += w.population() as u64;
        self.plans += u64::from(e.plans);
        self.plans_by_rememberers += u64::from(e.plans_by_rememberers);
        self.plans_with_remembered += u64::from(e.plans_with_remembered);
        self.fallback_short += u64::from(e.fallback_short);
        self.fallback_limit += u64::from(e.fallback_limit);
    }

    fn sum(all: &[Usage]) -> Usage {
        all.iter().fold(Usage::default(), |a, u| Usage {
            agent_ticks: a.agent_ticks + u.agent_ticks,
            plans: a.plans + u.plans,
            plans_by_rememberers: a.plans_by_rememberers + u.plans_by_rememberers,
            plans_with_remembered: a.plans_with_remembered + u.plans_with_remembered,
            fallback_short: a.fallback_short + u.fallback_short,
            fallback_limit: a.fallback_limit + u.fallback_limit,
        })
    }

    fn describe(&self, ticks: u32) -> String {
        let per = |n: u64| 100.0 * n as f64 / self.agent_ticks as f64;
        let used = if self.plans_by_rememberers == 0 {
            "no rememberer planned".to_string()
        } else {
            format!(
                "{} of the rememberers' {} plans ({:.1} %) include a remembered site out of sight",
                self.plans_with_remembered,
                self.plans_by_rememberers,
                100.0 * self.plans_with_remembered as f64 / self.plans_by_rememberers as f64
            )
        };
        format!(
            "Usage over ticks 1–{ticks}, all seeds ({} agent-ticks): a new plan on {:.1} %, the fallback because known sugar falls short of G on {:.1} %, because the search passed its limit on {:.2} %; the rest follow a plan or stay. {used}.",
            self.agent_ticks,
            per(self.plans),
            per(self.fallback_short),
            per(self.fallback_limit),
        )
    }
}

/// Steps `c` for `ticks` ticks per seed, summing its usage.
fn usage(c: &Config, seeds: &[u64], ticks: u32) -> Usage {
    Usage::sum(&each_seed(c, seeds, |mut w| {
        let mut u = Usage::default();
        for _ in 0..ticks {
            w.step();
            u.add(&w);
        }
        u
    }))
}

// ---------------------------------------------------------------- travel time

/// Per spacing: each seed's alive count and completed visits, for the seeds
/// whose slope is NaN (fewer than 3 spacings with a completed visit).
fn no_fit(seeds: &[u64], slopes: &[f64], runs: &[Vec<Visits>]) -> String {
    let rows: Vec<String> = slopes
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.is_finite())
        .map(|(i, _)| {
            let cells: Vec<String> = SPACINGS
                .iter()
                .zip(runs)
                .map(|(s, r)| format!("s {s}: {:.0} alive, {} visits", r[i].alive, r[i].visits))
                .collect();
            format!("seed {} ({})", seeds[i], cells.join("; "))
        })
        .collect();
    if rows.is_empty() {
        "Every seed has a fit.".into()
    } else {
        format!(
            "Seeds with no fit, with their alive counts at tick {MVT_TICKS} of 3: {}.",
            rows.join(", ")
        )
    }
}

/// A rule's residence-on-spacing figures, for the detail.
fn slope_report(name: &str, seeds: &[u64], slopes: &[f64], runs: &[Vec<Visits>]) -> String {
    format!(
        "{name}: slope finite in {} of {} seeds, positive in {}; median {:.4} (IQR {:.4}–{:.4}); per seed {}. Median residence by spacing: {}. {}",
        stats::finite(slopes).len(),
        slopes.len(),
        slopes.iter().filter(|&&x| x > 0.0).count(),
        med_or_nan(slopes),
        q_or_nan(slopes, 0.25),
        q_or_nan(slopes, 0.75),
        list(slopes, 3),
        residence_medians(runs, preset("goap-mvt").population),
        no_fit(seeds, slopes, runs),
    )
}

fn travel(rule: DecisionRule, seeds: &[u64]) -> Outcome {
    let (x, xr) = residence_slopes(seeds, |s| tori(s, rule));
    let (m, mr) = residence_slopes(seeds, |s| tori(s, DecisionRule::Book));
    let mut out = range(&x, f64::MIN_POSITIVE, f64::INFINITY, false)
        .with(&format!(
            "goap-mvt's world at spacing s (a 3s × 3s torus, the same nine patches, growback 0.02, 3 agents, vision 1–6, the prior map); completed visits over ticks 1–{MVT_TICKS} (visits under way at tick 0 or at the end, or cut short by death, are dropped); a slope needs at least 3 spacings with a completed visit; residences are medians over seeds of per-seed means."
        ))
        .with(&slope_report(rule_name(rule), seeds, &x, &xr))
        .with(&format!(
            "Reported, the same knowledge under {}: {}",
            rule_name(DecisionRule::Book),
            slope_report(rule_name(DecisionRule::Book), seeds, &m, &mr)
        ));
    if rule == DecisionRule::Goap {
        let (v, vr) = residence_slopes(seeds, value_shortlist);
        let alive = |r: &[Vec<Visits>]| {
            r[PRESET_SPACING]
                .iter()
                .map(|x| x.alive)
                .collect::<Vec<_>>()
        };
        let (rate_alive, value_alive) = (alive(&xr), alive(&vr));
        out = out
            .with(&format!(
                "Reported, GOAP with the spec's value shortlist (the K best known sites by value, not by value ÷ (distance + 1)): {}",
                slope_report("value shortlist", seeds, &v, &vr)
            ))
            .with(&format!(
                "Alive at tick {MVT_TICKS} of 3 at s = 20 (the preset), per seed: rate shortlist {} (total {:.0}); value shortlist {} (total {:.0}).",
                list(&rate_alive, 0),
                rate_alive.iter().sum::<f64>(),
                list(&value_alive, 0),
                value_alive.iter().sum::<f64>(),
            ));
        let per_spacing: Vec<String> = SPACINGS
            .iter()
            .map(|&s| {
                format!(
                    "s = {s}: {}",
                    usage(&tori(s, rule), seeds, MVT_TICKS).describe(MVT_TICKS)
                )
            })
            .collect();
        out = out.with(&per_spacing.join(" "));
    }
    out
}

// --------------------------------------------------------------- overstaying

fn shares(r: &[Visits]) -> Vec<f64> {
    r.iter().map(Visits::overstay_share).collect()
}

/// Median overstay share and departures at each spacing.
fn overstay_by_spacing(runs: &[Vec<Visits>]) -> String {
    SPACINGS
        .iter()
        .zip(runs)
        .map(|(s, r)| {
            format!(
                "{s}: {:.3} ({} departures)",
                med_or_nan(&shares(r)),
                r.iter().map(|v| v.departures).sum::<usize>()
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Per seed: the slope of the overstay share on spacing.
fn overstay_slopes(runs: &[Vec<Visits>], n: usize) -> Vec<f64> {
    let x: Vec<f64> = SPACINGS.iter().map(|&s| f64::from(s)).collect();
    (0..n)
        .map(|i| {
            slope(
                &x,
                &runs
                    .iter()
                    .map(|r| r[i].overstay_share())
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn overstay(rule: DecisionRule, seeds: &[u64]) -> Outcome {
    let runs: Vec<Vec<Visits>> = SPACINGS
        .iter()
        .map(|&s| visits(&tori(s, rule), seeds))
        .collect();
    let share = shares(&runs[PRESET_SPACING]);
    let slopes = overstay_slopes(&runs, seeds.len());
    let other = |r: DecisionRule| {
        let runs: Vec<Vec<Visits>> = SPACINGS
            .iter()
            .map(|&s| visits(&tori(s, r), seeds))
            .collect();
        format!(
            "{}: median share by spacing {}",
            rule_name(r),
            overstay_by_spacing(&runs)
        )
    };
    let others = [DecisionRule::Goap, DecisionRule::Mvt, DecisionRule::Book]
        .into_iter()
        .filter(|&r| r != rule)
        .map(other)
        .collect::<Vec<_>>()
        .join("; ");
    range(&share, 0.5 + f64::EPSILON, 1.0, false).with(&format!(
        "Per-seed share of departures whose last tick in the patch gathered less than the agent's mean gathered per tick so far, goap-mvt's world at s = 20 (the preset), ticks 1–{MVT_TICKS}; per seed {}. With travel (spacing): median share by spacing {}; per-seed slope of the share on spacing finite in {} of {} seeds, positive in {}, median {:.4} (IQR {:.4}–{:.4}). Compared on the same worlds: {others}.",
        list(&share, 2),
        overstay_by_spacing(&runs),
        stats::finite(&slopes).len(),
        slopes.len(),
        slopes.iter().filter(|&&x| x > 0.0).count(),
        med_or_nan(&slopes),
        q_or_nan(&slopes, 0.25),
        q_or_nan(&slopes, 0.75),
    ))
    .with(&settled_report(rule, seeds))
}

/// The overstay measure without transit (reported, not judged): each
/// departure's gain on its last in-patch tick on which the agent wasn't in
/// transit, against the same realized mean gain rate as `visits`. Visits
/// whose every in-patch tick was in transit are dropped.
#[derive(Default)]
struct Settled {
    departures: usize,
    dropped: usize,
    overstays: usize,
}

impl Settled {
    fn share(&self) -> f64 {
        let kept = self.departures - self.dropped;
        if kept == 0 {
            f64::NAN
        } else {
            self.overstays as f64 / kept as f64
        }
    }
}

/// What the survey's trace keeps of an agent before a step.
struct Before {
    leaving: Option<Pos>,
    targets: Vec<Pos>,
}

fn before(a: &Agent) -> Before {
    Before {
        leaving: a.leaving,
        targets: a
            .goap_plan
            .as_ref()
            .map_or(Vec::new(), |g| g.steps.iter().map(|s| s.0).collect()),
    }
}

/// Whether the agent was in transit this tick. The marginal-value rule:
/// `leaving` was set at the start of the tick or is set after it. GOAP: its
/// landing site wasn't a plan target (a step of its plan before or after the
/// tick, or the target it reached this tick under a plan). A GOAP tick on
/// the fallback (no plan) counts as transit.
fn in_transit(rule: DecisionRule, b: &Before, a: &Agent) -> bool {
    match rule {
        DecisionRule::Mvt => b.leaving.is_some() || a.leaving.is_some(),
        _ => {
            let reached = a.goap_plan.is_some() && a.plan.target == Some(a.pos);
            let listed = |t: &Vec<Pos>| t.contains(&a.pos);
            let after: Vec<Pos> = a
                .goap_plan
                .as_ref()
                .map_or(Vec::new(), |g| g.steps.iter().map(|s| s.0).collect());
            !(reached || listed(&b.targets) || listed(&after))
        }
    }
}

#[derive(Clone, Copy)]
struct SettledTrack {
    patch: Option<usize>,
    held: f64,
    gathered: f64,
    /// The gain on the visit's last tick not in transit, if any.
    settled_gain: Option<f64>,
}

/// `visits`'s trace, with the gain on the last tick not in transit.
fn settled(c: &Config, seeds: &[u64]) -> Vec<Settled> {
    let Map::Peaks { peaks } = &c.goods[0].map else {
        unreachable!("a patch world is a peaks map")
    };
    let peaks = peaks.clone();
    let (gw, gh, rule) = (c.width, c.height, c.decision.rule);
    each_seed(c, seeds, move |mut w| {
        let on = |x: u32, y: u32| patch_of(&peaks, x, y, gw, gh);
        let mut tracks: HashMap<AgentId, SettledTrack> = w
            .agents()
            .map(|a| {
                let t = SettledTrack {
                    patch: on(a.pos.x, a.pos.y),
                    held: a.holdings[0],
                    gathered: 0.0,
                    settled_gain: None,
                };
                (a.id, t)
            })
            .collect();
        let mut out = Settled::default();
        for _ in 0..MVT_TICKS {
            let pre: HashMap<AgentId, Before> = w.agents().map(|a| (a.id, before(a))).collect();
            w.step();
            let t = w.tick;
            for a in w.agents() {
                let tr = tracks.get_mut(&a.id).expect("no births in a patch world");
                let gain = a.holdings[0] - tr.held + f64::from(a.metabolism[0]);
                let patch = on(a.pos.x, a.pos.y);
                if patch != tr.patch {
                    if tr.patch.is_some() && t >= 2 {
                        out.departures += 1;
                        match tr.settled_gain {
                            None => out.dropped += 1,
                            Some(g) if g < tr.gathered / (t - 1) as f64 => out.overstays += 1,
                            Some(_) => {}
                        }
                    }
                    tr.patch = patch;
                    tr.settled_gain = None;
                }
                if patch.is_some() && !in_transit(rule, &pre[&a.id], a) {
                    tr.settled_gain = Some(gain);
                }
                tr.gathered += gain;
                tr.held = a.holdings[0];
            }
        }
        out
    })
}

/// The robustness row: per spacing, the median settled share, departures
/// kept and the share of departures dropped (all in transit).
fn settled_report(rule: DecisionRule, seeds: &[u64]) -> String {
    let cells: Vec<String> = SPACINGS
        .iter()
        .map(|&s| {
            let v = settled(&tori(s, rule), seeds);
            let deps: usize = v.iter().map(|x| x.departures).sum();
            let dropped: usize = v.iter().map(|x| x.dropped).sum();
            let shares: Vec<f64> = v.iter().map(Settled::share).collect();
            format!(
                "{s}: {:.3} ({} departures kept, {:.1} % dropped as all transit; above 0.5 in {} of {} seeds)",
                med_or_nan(&shares),
                deps - dropped,
                100.0 * dropped as f64 / deps.max(1) as f64,
                shares.iter().filter(|&&x| x > 0.5).count(),
                stats::finite(&shares).len(),
            )
        })
        .collect();
    format!(
        "Reported, not judged: the same measure without transit (the gain on the last in-patch tick not in transit; {}), median share by spacing: {}.",
        match rule {
            DecisionRule::Mvt => "in transit: `leaving` set at the start of the tick or after it",
            _ => "in transit: the landing site wasn't a plan target, or no plan (the fallback)",
        },
        cells.join(", ")
    )
}

// -------------------------------------------------------- memory for a planner

/// One 500-tick run of a memory world.
struct MemRun {
    g: Groups,
    usage: Usage,
    /// Mean population over ticks 200–500.
    pop: f64,
    /// Sugar held per founding rememberer and per founding other (the dead
    /// count as 0), each a mean over ticks 200–500 like the headline: the
    /// advantage without survivorship.
    per_founder: (f64, f64),
}

fn mem_runs(c: &Config, seeds: &[u64]) -> Vec<MemRun> {
    each_seed(c, seeds, |mut w| {
        let founders = |w: &World, r: bool| w.agents().filter(|a| a.remembers == r).count() as f64;
        let (r0, o0) = (founders(&w, true), founders(&w, false));
        let held = |w: &World, r: bool| -> f64 {
            w.agents()
                .filter(|a| a.remembers == r)
                .map(|a| a.holdings[0])
                .sum()
        };
        let mut usage = Usage::default();
        let (mut fr, mut fo, mut n) = (0.0, 0.0, 0.0);
        for _ in 0..500 {
            w.step();
            usage.add(&w);
            if w.tick >= 200 {
                fr += held(&w, true) / r0;
                fo += held(&w, false) / o0;
                n += 1.0;
            }
        }
        MemRun {
            g: measure(&w),
            usage,
            pop: window_mean(&series(&w, "population"), 200, 500),
            per_founder: (fr / n, fo / n),
        }
    })
}

/// Rememberers against others under GOAP in `goap_id`, paired per seed;
/// beside rule M's advantage in Minds 3's `mem_id` on the same seeds, and
/// the usage check.
fn planner_memory(goap_id: &str, mem_id: &str, seeds: &[u64]) -> Outcome {
    let (gr, mr) = (
        mem_runs(&preset(goap_id), seeds),
        mem_runs(&preset(mem_id), seeds),
    );
    let g: Vec<Groups> = gr.iter().map(|r| r.g.clone()).collect();
    let m: Vec<Groups> = mr.iter().map(|r| r.g.clone()).collect();
    let (ga, ma) = (col(&g, |x| x.adv), col(&m, |x| x.adv));
    let f = |r: &[MemRun], k: fn(&MemRun) -> f64| r.iter().map(k).collect::<Vec<f64>>();
    let founder = |r: &[MemRun]| {
        let d = f(r, |x| x.per_founder.0 - x.per_founder.1);
        format!(
            "rememberers {:.1}, others {:.1}, difference {:.1}, positive in {} of {} seeds",
            median(&f(r, |x| x.per_founder.0)),
            median(&f(r, |x| x.per_founder.1)),
            median(&d),
            d.iter().filter(|&&x| x > 0.0).count(),
            d.len()
        )
    };
    let (gp, mp) = (f(&gr, |x| x.pop), f(&mr, |x| x.pop));
    let usage: Vec<Usage> = gr.iter().map(|r| r.usage).collect();
    paired_greater(
        &col(&g, |x| x.rem),
        &col(&g, |x| x.oth),
        "rememberers",
        "others",
    )
    .with(&format!("GOAP: {}", describe(&g)))
    .with(&format!(
        "Per-seed advantage under GOAP: {}. Rule M ({mem_id}, same seeds): median advantage {:.2}; GOAP's advantage higher than rule M's in {} of {} seeds (reported, not judged). Rule M's {}",
        list(&ga, 1),
        median(&ma),
        ga.iter().zip(&ma).filter(|(a, b)| a > b).count(),
        seeds.len(),
        describe(&m),
    ))
    .with(&format!(
        "Reported, without survivorship (sugar held per founding member, the dead counting 0, mean over ticks 200–500): GOAP {}; rule M {}. Median population over ticks 200–500: GOAP {:.1}, rule M {:.1} (GOAP higher in {} of {} seeds).",
        founder(&gr),
        founder(&mr),
        median(&gp),
        median(&mp),
        gp.iter().zip(&mp).filter(|(a, b)| a > b).count(),
        seeds.len(),
    ))
    .with(&Usage::sum(&usage).describe(500))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "goap-mvt.travel",
            item: "goap-mvt",
            source: Source::Book,
            citation: "Charnov 1976; Stephens & Krebs 1986; Hayden, Pearson & Platt 2011",
            text: "Longer travel, longer stays: under GOAP, mean patch residence rises with the patches' spacing (the per-seed slope over spacings 12, 16, 20 and 24 is above 0 in at least 80 % of seeds); rule M with the same knowledge and the value shortlist are reported",
            check: |seeds| travel(DecisionRule::Goap, seeds),
        },
        Claim {
            id: "mvt-rule.travel",
            item: "mvt-rule",
            source: Source::Book,
            citation: "Charnov 1976; Stephens & Krebs 1986; Hayden, Pearson & Platt 2011",
            text: "Longer travel, longer stays: under the marginal-value rule, mean patch residence rises with the patches' spacing (the per-seed slope over spacings 12, 16, 20 and 24 is above 0 in at least 80 % of seeds); rule M with the same knowledge is reported",
            check: |seeds| travel(DecisionRule::Mvt, seeds),
        },
        Claim {
            id: "goap-mvt.overstay",
            item: "goap-mvt",
            source: Source::Book,
            citation: "Nonacs 2001; Constantino & Daw 2015",
            text: "Planners overstay: at over half of departures, the sugar gathered on the last tick in the patch is below the agent's mean gathered per tick so far, in at least 80 % of seeds (s = 20); the share against spacing is reported",
            check: |seeds| overstay(DecisionRule::Goap, seeds),
        },
        Claim {
            id: "mvt-rule.overstay",
            item: "mvt-rule",
            source: Source::Book,
            citation: "Nonacs 2001; Constantino & Daw 2015",
            text: "Marginal-value foragers overstay: at over half of departures, the sugar gathered on the last tick in the patch is below the agent's mean gathered per tick so far, in at least 80 % of seeds (s = 20); the share against spacing is reported",
            check: |seeds| overstay(DecisionRule::Mvt, seeds),
        },
        Claim {
            id: "goap-open.advantage",
            item: "goap-open",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays a planner on the open scape: under GOAP, rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| planner_memory("goap-open", "mem-open", seeds),
        },
        Claim {
            id: "goap-truffles.advantage",
            item: "goap-truffles",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays a planner with hidden truffles: under GOAP, rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| planner_memory("goap-truffles", "mem-truffles", seeds),
        },
        Claim {
            id: "goap-walled.advantage",
            item: "goap-walled",
            source: Source::Comment,
            citation: SPEC,
            text: "Memory pays a planner behind a wall: under GOAP, rememberers are wealthier than others (ticks 200–500), seed by seed",
            check: |seeds| planner_memory("goap-walled", "mem-walled", seeds),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_20_is_the_preset() {
        assert_eq!(tori(20, DecisionRule::Goap), preset("goap-mvt"));
        assert_eq!(tori(20, DecisionRule::Mvt), preset("mvt-rule"));
        let c = tori(12, DecisionRule::Book);
        assert_eq!((c.width, c.height), (36, 36));
        let (a, b) = (preset("goap-mvt"), c.clone());
        assert_eq!(
            (a.growback.rate, a.population, a.vision),
            (b.growback.rate, b.population, b.vision),
            "the balance doesn't change with s"
        );
        c.validate().expect("rule M with the prior map is valid");
        assert_eq!(value_shortlist(20).goap.shortlist, Shortlist::Value);
    }

    #[test]
    fn spacing_12_places_the_nine_peaks_at_s_over_2_plus_s_i() {
        let c = tori(12, DecisionRule::Goap);
        let Map::Peaks { peaks } = &c.goods[0].map else {
            panic!("a peaks map")
        };
        let mut at: Vec<(u32, u32)> = peaks.iter().map(|p| (p.x, p.y)).collect();
        at.sort_unstable();
        let mut want: Vec<(u32, u32)> = [6, 18, 30]
            .iter()
            .flat_map(|&x| [6, 18, 30].map(|y| (x, y)))
            .collect();
        want.sort_unstable();
        assert_eq!(at, want);
        assert!(peaks.iter().all(|p| p.radius == 3.0 && p.height == 4.0));
    }

    #[test]
    fn usage_counts_plans_and_agent_ticks() {
        let u = usage(&preset("goap-mvt"), &[1], 20);
        assert!(
            u.agent_ticks > 0 && u.agent_ticks <= 60,
            "{}",
            u.agent_ticks
        );
        assert!(u.plans > 0);
        assert!(u.plans_with_remembered <= u.plans_by_rememberers);
        assert!(u.plans_by_rememberers <= u.plans);
    }
}
