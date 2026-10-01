//! Minds 8: watching
//! (docs/superpowers/specs/2026-10-01-minds-8-watching-design.md, section
//! "Questions and judges", binding word for word).
//!
//! Pairing as in Minds 3–6: every configuration compared runs the same
//! seeds (1–20) and is judged on per-seed values or per-seed differences.
//! Every run lasts 200 ticks: a summer (ticks 0–99) and the first winter
//! (100–199). A tick stands for a day. The judges and thresholds below were
//! fixed in the spec (commit 57ec71c) and committed here before any survey
//! run. The Task 5 and 5b sanity runs (seeds 1–5) later showed watch-winter
//! near 0.9 % pilferage; that changed no threshold, and the survey reports
//! it as context.
//!
//! **`span` is a free parameter.** The presets' 7 is a gap choice (H&P show
//! recovery the next day and none at 14 days). Every claim is judged at
//! span 7 and reported at span 1, 3, 7 and 13.
//!
//! **Measures** (per run; Minds 6's unless said):
//! - **The pilferage rate** is Σ `caches_pilfered` ÷ Σ `pilfer_candidates`
//!   over ticks 1–200.
//! - **p_s and p_o** are amount-weighted from the tick events: p_s = dug ÷
//!   (dug + pilfered), p_o = pilfered ÷ (dug + pilfered + lost with a dead
//!   owner). Sugar still buried at 200 is excluded.
//! - **Survival per founder** of a group is alive at 200 ÷ the group's
//!   founders, the dead counting as 0. Groups: watchers (`Agent.watches`)
//!   against everyone else, and hoarders against cheaters (`Agent.cheater`),
//!   both dealt to founders by id; nobody is born in these worlds. Minds 6's
//!   survival (alive at 200 ÷ alive at 100) is reported beside it.
//! - **The decomposition** (claim 1): burials seen per burial (Σ
//!   `burials_seen` ÷ Σ burials), raids per sighting (Σ `raids` ÷ Σ
//!   `sightings`) and wasted raids per sighting (Σ `raids_wasted` ÷ Σ
//!   `sightings`). A burial is an agent whose cache at the site it ends the
//!   tick on grew during the tick (an agent buries once a turn, where it
//!   stands, and nobody else can stand there); the detected amounts are
//!   checked against `buried`.
//! - **Pilfers by source:** seen (`raids`, `raided`) against stumbled
//!   (`pilfers − raids`, `pilfered − raided`).
//! - **Raid freshness** (reported): the raided sugar by the age of the
//!   watcher's memory when it raided, now − the tick it last saw a burial
//!   there (0 to `span`). A take by an agent is attributed to a raid when,
//!   at the tick's start, it held a fresh entry at the site it ended on for
//!   an owner whose cache there shrank during the tick (the first such
//!   owner in id order gives the age). The attributed sugar is checked
//!   against `raided`.
//! - **Moves targeting a seen cache** (reported): of the choices made by
//!   agents alive after the tick (`Agent.plan.target`, set every turn), the
//!   share whose target is a site where the agent held a fresh entry at the
//!   tick's start. Entries made earlier in the same tick are missed, so the
//!   share is a slight undercount.
//! - **Caches known** (reported): at each tick's start, the share of the
//!   caches in the world that some living agent holds a fresh entry for (the
//!   share of existing caches seen buried within `span`), pooled over the
//!   summer's and the winter's ticks.
//! - **Sugar gathered from sites** (reported) is not an event, so it comes
//!   from the agents' ledger. Pilfers cancel over everyone, so per tick
//!   gathered = Σ holdings + caches + stomach of the living after the tick +
//!   the caches and stomachs lost with the dead − the same Σ over everyone
//!   at the tick's start + Σ metabolism burned + bury cost. A starved
//!   agent's own holdings at death (in (−metabolism, 0]) aren't seen, so
//!   the figure overstates by less than the metabolism of the tick's dead,
//!   a bound reported beside it.
//! - **Wealth at tick 100**: holdings + caches + stomach per living agent.
//!
//! **Judges** (seeds 1–20, span 7):
//! 1. **The field band** (`watch-winter.pilferage`): in `watch-winter`, the
//!    pilferage rate is at least 9 % in at least 80 % of seeds (`range` on
//!    flags); fails otherwise. Reported beside it: the decomposition, the
//!    same world with stumbling added back (`watch-winter-stumble`, find
//!    0.25), and Minds 6's stumbling-only rate (`theft-winter`).
//! 2. **Andersson and Krebs under watching** (`watch-half.threshold`):
//!    `theft-winter-half` against `watch-half`, paired on seeds. Holds when
//!    both hold in at least 80 % of seeds (`all_of` of two `range`s on
//!    flags): p_s ÷ p_o is lower under watching (a seed with p_o = 0 under
//!    both has no value; p_o = 0 under one only makes that world's ratio
//!    infinite, and a seed whose p_s is undefined in either world has no
//!    value), and the hoarders' advantage (hoarder survival per founder −
//!    cheater survival per founder) is lower under watching. Reported:
//!    whether p_s > p_o holds at all in either world.
//! 3. **Producers and scroungers** (`watch-scroungers.frequency` for
//!    watchers who also bury, `watch-scroungers-only.frequency` for pure
//!    scroungers: each variant is its own claim, judged and reported
//!    separately). The winter field with find 0, the watcher share s = 0.1,
//!    0.2, …, 0.9: `watch-scroungers` with no cheaters, and
//!    `watch-scroungers-only` with cheaters at the same share s (the same
//!    agents, who never bury). The watcher advantage is watcher survival
//!    per founder − non-watcher survival per founder. Per seed, the OLS
//!    slope of the advantage on s. Holds when the 95 % t confidence interval
//!    of the mean per-seed slope lies wholly below 0; fails when it includes
//!    0 or lies above it (fewer than 5 finite slopes: untestable). The
//!    per-seed crossing (the first sign change from above 0 to at most 0,
//!    interpolated) is reported where one exists. Our expectation, recorded
//!    in the spec before any run: pure scroungers show the prediction, and
//!    watchers who also bury don't.
//! 4. **Usage** (`watch-winter.usage`): in each watching preset except
//!    `watch-arena` (where the check is a Rust test), some sugar is taken by
//!    a raid in at least 80 % of seeds (`range` on flags, `all_of` over
//!    presets).
//!
//! `range` reads a share of seeds from 50 % to under 80 % as Weak; the spec's
//! "holds when at least 80 %" makes Weak a failure to hold.
//!
//! **Reported, not judged:** every claim's measures at span 1, 3, 7 and 13;
//! raid freshness; pilfers by source in every world; the share of moves
//! that targeted a seen cache; the mechanism the controller's investigation
//! found (sugar gathered in summer and winter, wealth per living agent at
//! tick 100, the moves share, raids by season, the share of caches known),
//! for watch-winter, cache-winter-even and theft-winter; and two switches
//! that isolate causes:
//! - **The probe** `World::probe_raid_harvests` (survey-only, not a
//!   setting): a raid that took something also harvests its site that tick,
//!   instead of replacing the harvest. Under it a tick can both pilfer and
//!   gather, and Compensate's weight update is skipped on such ticks (it
//!   skips every tick that pilfered). Reported beside claims 1 and 2 for
//!   watch-winter, watch-winter-stumble and watch-half: survival per founder
//!   and the pilferage rate.
//! - **Winter stumbling**: `find` set live at tick 100 (watch-winter with
//!   find 0.25 from then; theft-winter with find 0 from then).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Mutex;

use sugarscape_core::agent::{Agent, AgentId};
use sugarscape_core::config::Config;
use sugarscape_core::world::World;

use crate::claim::{all_of, range, untestable, Claim, Outcome, Source, Verdict};
use crate::claims::minds4::list;
use crate::claims::minds5::med;
use crate::claims::minds6::{ci95, col, crossing, flag, m, medp, nan_div, seed_slopes};
use crate::runner::{each_seed, preset};
use crate::stats;

const SPEC: &str = "docs/superpowers/specs/2026-10-01-minds-8-watching-design.md";

/// Every run: a summer and the first winter.
const TICKS: u64 = 200;
/// The span sweep, and the presets' anchor.
const SPANS: [u32; 4] = [1, 3, 7, 13];
const ANCHOR: u32 = 7;
/// Watcher shares for claim 3.
const SHARES: [f64; 9] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
/// Claim 1: the field studies' median daily pilferage.
const FIELD_MEDIAN: f64 = 0.09;
/// Minds 6's find anchor, for the winter-stumbling switch.
const FIND: f64 = 0.25;

/// Group indices: by watching (watchers, others) and by caching (hoarders,
/// cheaters).
const W: usize = 0;
const O: usize = 1;
const H: usize = 0;
const C: usize = 1;
/// Seasons: steps from ticks 0–99 and 100–199.
const SUMMER: usize = 0;
const WINTER: usize = 1;

// ------------------------------------------------------------------- the run

/// What a run sets beyond its config.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Setup {
    /// `World::probe_raid_harvests`.
    probe: bool,
    /// `theft.find` set live at tick 100.
    find_from_100: Option<f64>,
}

/// One run's measures (see the module's list). Arrays of two are by group
/// or by season, as named.
#[derive(Clone, Default)]
struct Run {
    founders_w: [f64; 2],
    alive100_w: [f64; 2],
    alive200_w: [f64; 2],
    founders_c: [f64; 2],
    alive100_c: [f64; 2],
    alive200_c: [f64; 2],
    /// By season.
    gathered: [f64; 2],
    raids: [f64; 2],
    raided: [f64; 2],
    wasted: [f64; 2],
    seen_arrivals: [f64; 2],
    pilfers: [f64; 2],
    pilfered: [f64; 2],
    moves: [f64; 2],
    seen_moves: [f64; 2],
    /// Σ over ticks' starts of caches known to some agent, and of caches.
    known: [f64; 2],
    caches: [f64; 2],
    buried: [f64; 2],
    burials: [f64; 2],
    burials_seen: [f64; 2],
    sightings: [f64; 2],
    dug: [f64; 2],
    lost: [f64; 2],
    deaths: [f64; 2],
    caches_pilfered: f64,
    candidates: f64,
    /// Σ holdings + caches + stomach of the living at 100.
    wealth100: f64,
    /// Raided sugar by the memory's age at the raid (index 0..=span), and
    /// all the sugar so attributed.
    raid_age: Vec<f64>,
    attributed: f64,
    /// Σ metabolism of the agents that died: the bound on `gathered`'s
    /// overstatement.
    gathered_slack: f64,
    /// Σ detected burial amounts (checked against `buried`).
    burial_amounts: f64,
}

impl Run {
    fn total(x: [f64; 2]) -> f64 {
        x[0] + x[1]
    }
    fn surv_w(&self, g: usize) -> f64 {
        nan_div(self.alive200_w[g], self.founders_w[g])
    }
    fn surv_c(&self, g: usize) -> f64 {
        nan_div(self.alive200_c[g], self.founders_c[g])
    }
    /// Alive at 200 ÷ alive at 100, by caching group (Minds 6's survival).
    fn surv100_c(&self, g: usize) -> f64 {
        nan_div(self.alive200_c[g], self.alive100_c[g])
    }
    fn surv100_w(&self, g: usize) -> f64 {
        nan_div(self.alive200_w[g], self.alive100_w[g])
    }
    fn surv_all(&self) -> f64 {
        nan_div(Self::total(self.alive200_w), Self::total(self.founders_w))
    }
    fn surv100_all(&self) -> f64 {
        nan_div(Self::total(self.alive200_w), Self::total(self.alive100_w))
    }
    /// Watcher − non-watcher survival per founder.
    fn watcher_adv(&self) -> f64 {
        self.surv_w(W) - self.surv_w(O)
    }
    /// Hoarder − cheater survival per founder.
    fn hoarder_adv(&self) -> f64 {
        self.surv_c(H) - self.surv_c(C)
    }
    fn rate(&self) -> f64 {
        nan_div(self.caches_pilfered, self.candidates)
    }
    fn dug_all(&self) -> f64 {
        Self::total(self.dug)
    }
    fn pilfered_all(&self) -> f64 {
        Self::total(self.pilfered)
    }
    fn p_s(&self) -> f64 {
        nan_div(self.dug_all(), self.dug_all() + self.pilfered_all())
    }
    fn p_o(&self) -> f64 {
        nan_div(
            self.pilfered_all(),
            self.dug_all() + self.pilfered_all() + Self::total(self.lost),
        )
    }
    /// p_s ÷ p_o: infinite when p_o = 0 with p_s > 0, NaN when either is
    /// undefined (no ended sugar) or both are 0.
    fn ps_po(&self) -> f64 {
        let (ps, po) = (self.p_s(), self.p_o());
        if !ps.is_finite() || !po.is_finite() {
            f64::NAN
        } else if po == 0.0 {
            if ps > 0.0 {
                f64::INFINITY
            } else {
                f64::NAN
            }
        } else {
            ps / po
        }
    }
    fn seen_per_burial(&self) -> f64 {
        nan_div(Self::total(self.burials_seen), Self::total(self.burials))
    }
    fn raids_per_sighting(&self) -> f64 {
        nan_div(Self::total(self.raids), Self::total(self.sightings))
    }
    fn wasted_per_sighting(&self) -> f64 {
        nan_div(Self::total(self.wasted), Self::total(self.sightings))
    }
    fn known_share(&self, s: usize) -> f64 {
        nan_div(self.known[s], self.caches[s])
    }
    fn seen_move_share(&self, s: usize) -> f64 {
        nan_div(self.seen_moves[s], self.moves[s])
    }
    fn wealth100_per_living(&self) -> f64 {
        nan_div(self.wealth100, Self::total(self.alive100_w))
    }
    /// Amount-weighted mean memory age at raiding.
    fn raid_age_mean(&self) -> f64 {
        let total: f64 = self.raid_age.iter().sum();
        nan_div(
            self.raid_age
                .iter()
                .enumerate()
                .map(|(a, q)| a as f64 * q)
                .sum(),
            total,
        )
    }
    /// The share of attributed raided sugar with memory age in `lo..=hi`.
    fn raid_age_share(&self, lo: usize, hi: usize) -> f64 {
        let total: f64 = self.raid_age.iter().sum();
        let part: f64 = self
            .raid_age
            .iter()
            .enumerate()
            .filter(|(a, _)| (lo..=hi).contains(a))
            .map(|(_, q)| q)
            .sum();
        nan_div(part, total)
    }
}

fn wealth(a: &Agent) -> f64 {
    a.holdings[0] + a.caches.values().sum::<f64>() + a.fed
}

fn run(mut w: World, setup: Setup) -> Run {
    w.probe_raid_harvests = setup.probe;
    let torus = w.torus;
    let span = u64::from(w.config.watching.span);
    let mut r = Run {
        raid_age: vec![0.0; span as usize + 1],
        ..Run::default()
    };
    let gw = |a: &Agent| usize::from(!a.watches);
    let gc = |a: &Agent| usize::from(a.cheater);
    for a in w.agents() {
        r.founders_w[gw(a)] += 1.0;
        r.founders_c[gc(a)] += 1.0;
    }
    while w.tick < TICKS {
        let t0 = w.tick;
        if t0 == 100 {
            if let Some(f) = setup.find_from_100 {
                w.config.theft.find = f;
            }
        }
        let s = usize::from(t0 >= 100);
        // The tick's start: wealth, stolen, metabolism; caches; fresh
        // entries.
        let mut before: HashMap<AgentId, (f64, f64, f64)> = HashMap::new();
        let mut caches0: HashMap<(AgentId, u32), f64> = HashMap::new();
        let mut fresh: HashMap<AgentId, BTreeMap<(u32, AgentId), u64>> = HashMap::new();
        for a in w.agents() {
            before.insert(
                a.id,
                (wealth(a), a.stolen_by_me, f64::from(a.metabolism[0])),
            );
            for (&site, &q) in &a.caches {
                caches0.insert((a.id, site), q);
            }
            let f: BTreeMap<(u32, AgentId), u64> = a
                .seen
                .iter()
                .filter(|(_, e)| t0.saturating_sub(e.tick) <= span)
                .map(|(&k, e)| (k, e.tick))
                .collect();
            if !f.is_empty() {
                fresh.insert(a.id, f);
            }
        }
        let known: HashSet<(u32, AgentId)> = fresh
            .values()
            .flat_map(|f| f.keys().copied())
            .filter(|&(site, owner)| caches0.contains_key(&(owner, site)))
            .collect();
        r.known[s] += known.len() as f64;
        r.caches[s] += caches0.len() as f64;

        w.step();

        let e = w.events();
        r.raids[s] += f64::from(e.raids);
        r.raided[s] += e.raided;
        r.wasted[s] += f64::from(e.raids_wasted);
        r.seen_arrivals[s] += f64::from(e.seen_arrivals);
        r.pilfers[s] += f64::from(e.pilfers);
        r.pilfered[s] += e.pilfered;
        r.buried[s] += e.buried;
        r.burials_seen[s] += f64::from(e.burials_seen);
        r.sightings[s] += f64::from(e.sightings);
        r.dug[s] += e.dug;
        r.lost[s] += e.cache_lost;
        r.deaths[s] += e.deaths.len() as f64;
        r.caches_pilfered += f64::from(e.caches_pilfered);
        r.candidates += f64::from(e.pilfer_candidates);
        let (lost, fed_lost, bury_cost) = (e.cache_lost, e.fed_lost, e.bury_cost);

        let mut w1 = 0.0;
        for a in w.agents() {
            let Some(&(_, by0, _)) = before.get(&a.id) else {
                continue;
            };
            w1 += wealth(a);
            let site = torus.index(a.pos) as u32;
            let mine = fresh.get(&a.id);
            if let Some(t) = a.plan.target {
                r.moves[s] += 1.0;
                let ts = torus.index(t) as u32;
                if mine.is_some_and(|f| f.keys().any(|&(x, _)| x == ts)) {
                    r.seen_moves[s] += 1.0;
                }
            }
            let pre = caches0.get(&(a.id, site)).copied().unwrap_or(0.0);
            let post = a.caches.get(&site).copied().unwrap_or(0.0);
            if post > pre {
                r.burials[s] += 1.0;
                r.burial_amounts += post - pre;
            }
            let took = a.stolen_by_me - by0;
            if took > 0.0 {
                let hit = mine.and_then(|f| {
                    f.iter()
                        .filter(|(&(x, _), _)| x == site)
                        .find(|(&(_, owner), _)| {
                            let q0 = caches0.get(&(owner, site)).copied().unwrap_or(0.0);
                            let q1 = w
                                .agent(owner)
                                .and_then(|o| o.caches.get(&site))
                                .copied()
                                .unwrap_or(0.0);
                            q0 > 0.0 && q1 < q0
                        })
                        .map(|(_, &seen_at)| seen_at)
                });
                if let Some(seen_at) = hit {
                    let age = t0.saturating_sub(seen_at).min(span) as usize;
                    r.raid_age[age] += took;
                    r.attributed += took;
                }
            }
        }
        let (mut w0, mut burned, mut dead_burn) = (0.0, 0.0, 0.0);
        for (id, &(wealth0, _, metab)) in &before {
            w0 += wealth0;
            burned += metab;
            if w.agent(*id).is_none() {
                dead_burn += metab;
            }
        }
        r.gathered[s] += w1 + lost + fed_lost - w0 + burned + bury_cost;
        r.gathered_slack += dead_burn;

        match w.tick {
            100 => {
                for a in w.agents() {
                    r.alive100_w[gw(a)] += 1.0;
                    r.alive100_c[gc(a)] += 1.0;
                    r.wealth100 += wealth(a);
                }
            }
            200 => {
                for a in w.agents() {
                    r.alive200_w[gw(a)] += 1.0;
                    r.alive200_c[gc(a)] += 1.0;
                }
            }
            _ => {}
        }
    }
    r
}

/// Cached runs: (key, seeds, runs).
type RunCache = Vec<(String, Vec<u64>, Vec<Run>)>;

/// Runs of `c` under `setup`, shared by the claims.
fn runs_of(c: &Config, setup: Setup, seeds: &[u64]) -> Vec<Run> {
    static CACHE: Mutex<RunCache> = Mutex::new(Vec::new());
    let key = format!(
        "{setup:?}{}",
        serde_json::to_string(c).expect("a config serializes")
    );
    if let Some(hit) = CACHE
        .lock()
        .unwrap()
        .iter()
        .find(|(k, s, _)| *k == key && s == seeds)
    {
        return hit.2.clone();
    }
    let out = each_seed(c, seeds, |w| run(w, setup));
    CACHE
        .lock()
        .unwrap()
        .push((key, seeds.to_vec(), out.clone()));
    out
}

fn runs(c: &Config, seeds: &[u64]) -> Vec<Run> {
    runs_of(c, Setup::default(), seeds)
}

fn probed(c: &Config, seeds: &[u64]) -> Vec<Run> {
    runs_of(
        c,
        Setup {
            probe: true,
            ..Setup::default()
        },
        seeds,
    )
}

fn find_from_100(c: &Config, find: f64, seeds: &[u64]) -> Vec<Run> {
    runs_of(
        c,
        Setup {
            find_from_100: Some(find),
            ..Setup::default()
        },
        seeds,
    )
}

// ------------------------------------------------------------------ worlds

/// A preset at `span`.
fn at_span(id: &str, span: u32) -> Config {
    let mut c = preset(id);
    c.watching.span = span;
    c
}

/// Claim 3's worlds: watchers at share `s` who also bury, at `span`.
fn scroungers(s: f64, span: u32) -> Config {
    let mut c = at_span("watch-scroungers", span);
    c.watching.watchers = s;
    c
}

/// Claim 3's worlds: pure scroungers (watchers and cheaters at share `s`,
/// the same agents), at `span`.
fn scroungers_only(s: f64, span: u32) -> Config {
    let mut c = at_span("watch-scroungers-only", span);
    c.watching.watchers = s;
    c.theft.cheaters = s;
    c
}

/// The watching presets claim 4 judges (`watch-arena` is a check).
const USAGE: [&str; 5] = [
    "watch-winter",
    "watch-winter-stumble",
    "watch-half",
    "watch-scroungers",
    "watch-scroungers-only",
];

// ----------------------------------------------------------------- helpers

fn pc(x: f64) -> String {
    format!("{:.2} %", 100.0 * x)
}

/// Median over seeds as a percentage.
fn mp(v: &[f64]) -> String {
    pc(m(v))
}

/// Survival per founder (and of those alive at 100) for the groups that
/// exist, for the detail.
fn groups(r: &[Run]) -> String {
    let mut out = vec![format!(
        "everyone: survival per founder {}, of those alive at 100 {}",
        med(&col(r, Run::surv_all)),
        med(&col(r, Run::surv100_all)),
    )];
    let split = |f: fn(&Run) -> [f64; 2]| r.iter().all(|x| f(x)[0] > 0.0 && f(x)[1] > 0.0);
    if split(|x| x.founders_w) {
        out.push(format!(
            "watchers {} (of those alive at 100 {}), others {} ({}), watcher advantage {}",
            med(&col(r, |x| x.surv_w(W))),
            med(&col(r, |x| x.surv100_w(W))),
            med(&col(r, |x| x.surv_w(O))),
            med(&col(r, |x| x.surv100_w(O))),
            med(&col(r, Run::watcher_adv)),
        ));
    }
    if split(|x| x.founders_c) {
        out.push(format!(
            "hoarders {} (of those alive at 100 {}), cheaters {} ({}), hoarder advantage {}",
            med(&col(r, |x| x.surv_c(H))),
            med(&col(r, |x| x.surv100_c(H))),
            med(&col(r, |x| x.surv_c(C))),
            med(&col(r, |x| x.surv100_c(C))),
            med(&col(r, Run::hoarder_adv)),
        ));
    }
    out.join("; ")
}

/// Pilfers by source, seen against stumbled (medians per seed).
fn sources(r: &[Run]) -> String {
    let t = Run::total;
    format!(
        "pilfers seen {} against stumbled {}; sugar raided {} against stumbled {} (summer {} / {}, winter {} / {})",
        med(&col(r, |x| t(x.raids))),
        med(&col(r, |x| t(x.pilfers) - t(x.raids))),
        med(&col(r, |x| t(x.raided))),
        med(&col(r, |x| t(x.pilfered) - t(x.raided))),
        med(&col(r, |x| x.raided[SUMMER])),
        med(&col(r, |x| x.pilfered[SUMMER] - x.raided[SUMMER])),
        med(&col(r, |x| x.raided[WINTER])),
        med(&col(r, |x| x.pilfered[WINTER] - x.raided[WINTER])),
    )
}

/// The decomposition and raid counts of claim 1.
fn decomposition(r: &[Run]) -> String {
    let t = Run::total;
    format!(
        "burials per seed {}, burials seen per burial {}, sightings per seed {}, raids per sighting {}, wasted raids per sighting {}; raids per seed {} (summer {}, winter {}), wasted {}, seen arrivals {}",
        med(&col(r, |x| t(x.burials))),
        med(&col(r, Run::seen_per_burial)),
        med(&col(r, |x| t(x.sightings))),
        med(&col(r, Run::raids_per_sighting)),
        med(&col(r, Run::wasted_per_sighting)),
        med(&col(r, |x| t(x.raids))),
        med(&col(r, |x| x.raids[SUMMER])),
        med(&col(r, |x| x.raids[WINTER])),
        med(&col(r, |x| t(x.wasted))),
        med(&col(r, |x| t(x.seen_arrivals))),
    )
}

/// Raid freshness: the memory's age at raiding.
fn freshness(r: &[Run]) -> String {
    let t = Run::total;
    let span = r.first().map_or(0, |x| x.raid_age.len() - 1);
    format!(
        "raided sugar's memory age (ticks since the watcher last saw a burial there; span {span}): mean {}, at age 0–1 {}, 2–3 {}, 4–7 {}, 8 or more {}; attributed ÷ raided {}",
        med(&col(r, Run::raid_age_mean)),
        mp(&col(r, |x| x.raid_age_share(0, 1))),
        mp(&col(r, |x| x.raid_age_share(2, 3))),
        mp(&col(r, |x| x.raid_age_share(4, 7))),
        mp(&col(r, |x| x.raid_age_share(8, usize::MAX))),
        med(&col(r, |x| nan_div(x.attributed, t(x.raided)))),
    )
}

/// The mechanism's measures for a world.
fn mechanism(label: &str, r: &[Run]) -> String {
    let t = Run::total;
    format!(
        "{label}: sugar gathered from sites per seed, summer {}, winter {} (overstated by under {} a seed, the metabolism of the dead); wealth at tick 100 per living agent {}; moves targeting a seen cache, summer {}, winter {}; raids per seed, summer {}, winter {}; caches known to some agent (seen buried within span), summer {}, winter {}; sugar buried per seed {}, dug {}, pilfered {}, lost with the dead {}; deaths, summer {}, winter {}; {}",
        med(&col(r, |x| x.gathered[SUMMER])),
        med(&col(r, |x| x.gathered[WINTER])),
        med(&col(r, |x| x.gathered_slack)),
        med(&col(r, Run::wealth100_per_living)),
        mp(&col(r, |x| x.seen_move_share(SUMMER))),
        mp(&col(r, |x| x.seen_move_share(WINTER))),
        med(&col(r, |x| x.raids[SUMMER])),
        med(&col(r, |x| x.raids[WINTER])),
        mp(&col(r, |x| x.known_share(SUMMER))),
        mp(&col(r, |x| x.known_share(WINTER))),
        med(&col(r, |x| t(x.buried))),
        med(&col(r, Run::dug_all)),
        med(&col(r, Run::pilfered_all)),
        med(&col(r, |x| t(x.lost))),
        med(&col(r, |x| x.deaths[SUMMER])),
        med(&col(r, |x| x.deaths[WINTER])),
        groups(r),
    )
}

/// A world's row across the span sweep: rate, decomposition, sources,
/// freshness, the moves share and survival.
fn span_row(label: &str, r: &[Run]) -> String {
    format!(
        "{label}: rate {}, at least 9 % in {} of {} seeds; {}; {}; {}; moves targeting a seen cache {} (summer) and {} (winter); caches known {} and {}; {}",
        medp(&col(r, Run::rate)),
        r.iter().filter(|x| x.rate() >= FIELD_MEDIAN).count(),
        r.len(),
        decomposition(r),
        sources(r),
        freshness(r),
        mp(&col(r, |x| x.seen_move_share(SUMMER))),
        mp(&col(r, |x| x.seen_move_share(WINTER))),
        mp(&col(r, |x| x.known_share(SUMMER))),
        mp(&col(r, |x| x.known_share(WINTER))),
        groups(r),
    )
}

/// Holds when the 95 % t interval of the mean of `slopes` lies wholly below
/// 0; fails when it includes 0 or lies above it.
fn ci_below_zero(slopes: &[f64]) -> Outcome {
    let v = stats::finite(slopes);
    if v.len() < 5 {
        return untestable(&format!(
            "only {} seeds gave a finite slope (need 5)",
            v.len()
        ));
    }
    let (mu, lo, hi) = ci95(&v);
    Outcome {
        verdict: if hi < 0.0 {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!(
            "mean per-seed slope {mu:.4}, 95 % CI {lo:.4} to {hi:.4} (t, df {}); {} of {} slopes below 0",
            v.len() - 1,
            v.iter().filter(|&&x| x < 0.0).count(),
            v.len()
        ),
        detail: String::new(),
    }
}

/// The first sign change of `ys` from above 0 to at most 0, interpolated.
fn down_crossing(xs: &[f64], ys: &[f64]) -> Option<f64> {
    let neg: Vec<f64> = ys.iter().map(|y| -y).collect();
    crossing(xs, &neg)
}

// --------------------------------------------------------- 1. the field band

fn pilferage_claim(seeds: &[u64]) -> Outcome {
    let anchor = runs(&preset("watch-winter"), seeds);
    let flags = col(&anchor, |x| flag(x.rate() >= FIELD_MEDIAN));
    let mut sweep = Vec::new();
    for id in ["watch-winter", "watch-winter-stumble"] {
        for span in SPANS {
            sweep.push(span_row(
                &format!("{id}, span {span}"),
                &runs(&at_span(id, span), seeds),
            ));
        }
    }
    let theft = runs(&preset("theft-winter"), seeds);
    let probe: Vec<String> = ["watch-winter", "watch-winter-stumble"]
        .iter()
        .map(|&id| {
            let (on, off) = (probed(&preset(id), seeds), runs(&preset(id), seeds));
            format!(
                "{id}: with the probe, rate {}, {}; as recorded, rate {}, {}",
                medp(&col(&on, Run::rate)),
                groups(&on),
                medp(&col(&off, Run::rate)),
                groups(&off),
            )
        })
        .collect();
    range(&flags, 1.0, 1.0, false)
        .with(&format!(
            "watch-winter (span {ANCHOR}, find 0, every agent a watcher, no cheaters), ticks 1–200, per seed: rate (%) {}; at least 9 % in {} of {} seeds. Decomposition: {}. {}.",
            list(&col(&anchor, |x| 100.0 * x.rate()), 2),
            flags.iter().filter(|&&f| f == 1.0).count(),
            flags.len(),
            decomposition(&anchor),
            sources(&anchor),
        ))
        .with(&format!(
            "Across the span sweep (medians over seeds): {}.",
            sweep.join("; ")
        ))
        .with(&format!(
            "Minds 6's stumbling only (theft-winter, find 0.25): rate {}; {}.",
            medp(&col(&theft, Run::rate)),
            groups(&theft),
        ))
        .with(&format!(
            "The probe, not a setting (World::probe_raid_harvests: a raid that took something also harvests its site that tick, instead of replacing the harvest; Compensate's weight update is skipped on such ticks): {}.",
            probe.join("; ")
        ))
        .with("Context, not judged. The 9 % median and the 2–30 % band are the field studies Minds 6 used (Vander Wall and Jenkins 2003, pp.656, 663). The judge and its threshold were fixed in the spec (commit 57ec71c) before any run; the Task 5 and 5b sanity runs (seeds 1–5) later showed watch-winter near 0.9 %, which changed no threshold. Bugnyar and Kotrschal: of 29 caches made with observers present and not taken back by the cacher, \"all were found during retrieval sessions by conspecifics that had been in the pathway during caching\" (p.188); Heinrich and Pepper: observers recovered seen caches the next day and \"No caches other than the birds' own were recovered during the 14- and 28-day trials\" (p.1086).")
}

// ------------------------------------------------------ 2. the threshold

fn threshold_flags(theft: &[Run], watch: &[Run]) -> (Vec<f64>, Vec<f64>) {
    let ratio: Vec<f64> = theft
        .iter()
        .zip(watch)
        .map(|(t, w)| {
            if t.p_o() == 0.0 && w.p_o() == 0.0 {
                return f64::NAN;
            }
            let (rt, rw) = (t.ps_po(), w.ps_po());
            if rt.is_nan() || rw.is_nan() {
                f64::NAN
            } else {
                flag(rw < rt)
            }
        })
        .collect();
    let adv: Vec<f64> = theft
        .iter()
        .zip(watch)
        .map(|(t, w)| {
            let (at, aw) = (t.hoarder_adv(), w.hoarder_adv());
            if at.is_nan() || aw.is_nan() {
                f64::NAN
            } else {
                flag(aw < at)
            }
        })
        .collect();
    (ratio, adv)
}

fn threshold_row(label: &str, r: &[Run]) -> String {
    format!(
        "{label}: p_s {}, p_o {}, p_s ÷ p_o {}, p_s > p_o in {} of {} seeds; rate {}; {}; {}",
        med(&col(r, Run::p_s)),
        med(&col(r, Run::p_o)),
        med(&col(r, Run::ps_po)),
        r.iter().filter(|x| x.p_s() > x.p_o()).count(),
        r.len(),
        medp(&col(r, Run::rate)),
        sources(r),
        groups(r),
    )
}

fn threshold_claim(seeds: &[u64]) -> Outcome {
    let theft = runs(&preset("theft-winter-half"), seeds);
    let watch = runs(&preset("watch-half"), seeds);
    let (ratio, adv) = threshold_flags(&theft, &watch);
    let sweep: Vec<String> = SPANS
        .iter()
        .map(|&span| {
            let w = runs(&at_span("watch-half", span), seeds);
            let (rf, af) = threshold_flags(&theft, &w);
            format!(
                "{}; p_s ÷ p_o lower than theft-winter-half's in {} of {} seeds with a value, hoarder advantage lower in {} of {}",
                threshold_row(&format!("watch-half, span {span}"), &w),
                rf.iter().filter(|&&f| f == 1.0).count(),
                stats::finite(&rf).len(),
                af.iter().filter(|&&f| f == 1.0).count(),
                stats::finite(&af).len(),
            )
        })
        .collect();
    let probe = probed(&preset("watch-half"), seeds);
    all_of(vec![
        (
            "p_s ÷ p_o lower under watching".into(),
            range(&ratio, 1.0, 1.0, false),
        ),
        (
            "hoarder advantage lower under watching".into(),
            range(&adv, 1.0, 1.0, false),
        ),
    ])
    .with(&format!(
        "theft-winter-half (half cheaters, find 0.25) against watch-half (the same, every agent a watcher, span {ANCHOR}), paired on seeds, ticks 1–200. Per seed p_s ÷ p_o, theft {}; watching {}. Per seed hoarder advantage (hoarder − cheater survival per founder), theft {}; watching {}. {}. {}.",
        list(&col(&theft, Run::ps_po), 4),
        list(&col(&watch, Run::ps_po), 4),
        list(&col(&theft, Run::hoarder_adv), 3),
        list(&col(&watch, Run::hoarder_adv), 3),
        threshold_row("theft-winter-half", &theft),
        threshold_row("watch-half", &watch),
    ))
    .with(&format!("Across the span sweep: {}.", sweep.join("; ")))
    .with(&format!(
        "The probe, not a setting (a raid also harvests its site): {}.",
        threshold_row("watch-half with the probe", &probe)
    ))
    .with("Andersson and Krebs's condition with free burying is p_s > p_o (p.708). In Minds 6's theft-winter-half it already failed: owners dug 1.5 % of their ended caches' sugar and thieves took 98 %.")
}

// --------------------------------------------------- 3. producers and scroungers

fn share_worlds(only: bool, span: u32, seeds: &[u64]) -> Vec<Vec<Run>> {
    SHARES
        .iter()
        .map(|&s| {
            runs(
                &if only {
                    scroungers_only(s, span)
                } else {
                    scroungers(s, span)
                },
                seeds,
            )
        })
        .collect()
}

fn frequency_claim_for(only: bool, seeds: &[u64]) -> Outcome {
    let worlds = share_worlds(only, ANCHOR, seeds);
    let slopes = seed_slopes(&SHARES, &worlds, Run::watcher_adv);
    let crossings: Vec<String> = (0..seeds.len())
        .map(|i| {
            let ys: Vec<f64> = worlds.iter().map(|w| w[i].watcher_adv()).collect();
            down_crossing(&SHARES, &ys).map_or("none".into(), |x| format!("{x:.2}"))
        })
        .collect();
    let curve: Vec<f64> = worlds
        .iter()
        .map(|r| m(&col(r, Run::watcher_adv)))
        .collect();
    let rows: Vec<String> = SHARES
        .iter()
        .zip(&worlds)
        .map(|(s, r)| {
            format!(
                "share {s}: {}; rate {}; {}",
                groups(r),
                medp(&col(r, Run::rate)),
                sources(r)
            )
        })
        .collect();
    let sweep: Vec<String> = SPANS
        .iter()
        .map(|&span| {
            let w = share_worlds(only, span, seeds);
            let sl = seed_slopes(&SHARES, &w, Run::watcher_adv);
            let (mu, lo, hi) = ci95(&sl);
            let c: Vec<f64> = w.iter().map(|r| m(&col(r, Run::watcher_adv))).collect();
            format!(
                "span {span}: mean slope {mu:.4} (95 % CI {lo:.4} to {hi:.4}); median advantage by share {} (down-crossing {})",
                list(&c, 3),
                down_crossing(&SHARES, &c).map_or("none".into(), |x| format!("{x:.2}")),
            )
        })
        .collect();
    let world = if only {
        "watch-scroungers-only's world (find 0; watchers and cheaters at the same share s, the same agents, who never bury; the rest bury and never watch)"
    } else {
        "watch-scroungers' world (find 0, no cheaters; every agent buries, watchers at share s)"
    };
    ci_below_zero(&slopes)
        .with(&format!(
            "{world}, span {ANCHOR}, s = 0.1–0.9, ticks 1–200; the advantage is watcher − non-watcher survival per founder (alive at 200 ÷ founders). Per-seed slopes {}. Per-seed down-crossings (first sign change from above 0 to at most 0): {}. Median advantage by share: {} (down-crossing {}). By share (medians): {}.",
            list(&slopes, 3),
            crossings.join(" "),
            list(&curve, 3),
            down_crossing(&SHARES, &curve).map_or("none".into(), |x| format!("{x:.2}")),
            rows.join("; "),
        ))
        .with(&format!("Across the span sweep: {}.", sweep.join("; ")))
        .with("Barnard and Sibly: \"the hypothetical pay-off to scroungers increases with the number of producers\", with a stable mix at the ESS point (1981). Our expectation, recorded in the spec before any run: pure scroungers show the prediction, and watchers who also bury don't, because watching costs them nothing.")
}

fn frequency_claim(seeds: &[u64]) -> Outcome {
    frequency_claim_for(false, seeds)
}

fn frequency_only_claim(seeds: &[u64]) -> Outcome {
    frequency_claim_for(true, seeds)
}

// ------------------------------------------------------------ 4. usage

fn usage_claim(seeds: &[u64]) -> Outcome {
    let mut parts = Vec::new();
    let mut rows = Vec::new();
    for id in USAGE {
        let r = runs(&preset(id), seeds);
        parts.push((
            id.to_string(),
            range(
                &col(&r, |x| flag(Run::total(x.raided) > 0.0)),
                1.0,
                1.0,
                false,
            ),
        ));
        rows.push(format!(
            "{id}: rate {}; {}; {}; {}; {}",
            medp(&col(&r, Run::rate)),
            decomposition(&r),
            sources(&r),
            freshness(&r),
            groups(&r)
        ));
    }
    let spans: Vec<String> = USAGE
        .iter()
        .flat_map(|&id| {
            SPANS.iter().map(move |&span| (id, span))
        })
        .map(|(id, span)| {
            let r = runs(&at_span(id, span), seeds);
            format!(
                "{id}, span {span}: raids per seed {}, wasted {}, sugar raided {}; {}; moves targeting a seen cache {} (summer), {} (winter)",
                med(&col(&r, |x| Run::total(x.raids))),
                med(&col(&r, |x| Run::total(x.wasted))),
                med(&col(&r, |x| Run::total(x.raided))),
                freshness(&r),
                mp(&col(&r, |x| x.seen_move_share(SUMMER))),
                mp(&col(&r, |x| x.seen_move_share(WINTER))),
            )
        })
        .collect();
    let arena = runs(&preset("watch-arena"), seeds);
    let even = runs(&preset("cache-winter-even"), seeds);
    let theft = runs(&preset("theft-winter"), seeds);
    let watch = runs(&preset("watch-winter"), seeds);
    let probes: Vec<String> = ["watch-winter", "watch-winter-stumble", "watch-half"]
        .iter()
        .map(|&id| mechanism(&format!("{id} with the probe"), &probed(&preset(id), seeds)))
        .collect();
    let checks: Vec<String> = USAGE
        .iter()
        .map(|&id| {
            let r = runs(&preset(id), seeds);
            format!(
                "{id}: attributed ÷ raided {}, detected burial sugar ÷ buried {}",
                med(&col(&r, |x| nan_div(x.attributed, Run::total(x.raided)))),
                med(&col(&r, |x| nan_div(
                    x.burial_amounts,
                    Run::total(x.buried)
                ))),
            )
        })
        .collect();
    all_of(parts)
        .with(&format!(
            "Per preset (span {ANCHOR}, medians over seeds, ticks 1–200): {}.",
            rows.join(". ")
        ))
        .with(&format!(
            "Raids, wasted raids and freshness against span: {}.",
            spans.join("; ")
        ))
        .with(&format!(
            "The check world (watch-arena, not judged: its rules are pinned by Rust tests): rate {}; {}; {}; {}.",
            medp(&col(&arena, Run::rate)),
            decomposition(&arena),
            sources(&arena),
            groups(&arena),
        ))
        .with(&format!(
            "The mechanism (reported; causes likely unless a switch isolates them): {}; {}; {}.",
            mechanism("cache-winter-even (no theft)", &even),
            mechanism("theft-winter (stumbling only)", &theft),
            mechanism("watch-winter", &watch),
        ))
        .with(&format!(
            "Switches. The probe (a raid also harvests its site): {}. Winter stumbling (find set live at tick 100): watch-winter with find 0.25 from tick 100, {}; theft-winter with find 0 from tick 100, {}.",
            probes.join("; "),
            groups(&find_from_100(&preset("watch-winter"), FIND, seeds)),
            groups(&find_from_100(&preset("theft-winter"), 0.0, seeds)),
        ))
        .with(&format!(
            "Measurement checks (each should be 1): {}.",
            checks.join("; ")
        ))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "watch-winter.pilferage",
            item: "watch-winter",
            source: Source::Comment,
            citation: "Bugnyar & Kotrschal 2002, p.188; Heinrich & Pepper 1998, p.1086; Vander Wall & Jenkins 2003, pp.656, 663; the spec",
            text: "Watching reaches the field's pilferage rates: in watch-winter (every agent a watcher, no stumbling) the pilferage rate is at least 9 % a day in at least 80 % of seeds, at span 7",
            check: pilferage_claim,
        },
        Claim {
            id: "watch-half.threshold",
            item: "watch-half",
            source: Source::Book,
            citation: "Andersson & Krebs 1978, condition (3), p.708",
            text: "Watching breaks Andersson and Krebs's condition: against theft-winter-half, watch-half lowers p_s ÷ p_o and the hoarders' advantage (survival per founder), each in at least 80 % of seeds",
            check: threshold_claim,
        },
        Claim {
            id: "watch-scroungers.frequency",
            item: "watch-scroungers",
            source: Source::Book,
            citation: "Barnard & Sibly 1981",
            text: "Watching is a scrounger strategy, worth less as it becomes common (watchers who also bury): the watcher advantage's per-seed slope on the watcher share (0.1–0.9) has a 95 % confidence interval wholly below 0",
            check: frequency_claim,
        },
        Claim {
            id: "watch-scroungers-only.frequency",
            item: "watch-scroungers-only",
            source: Source::Book,
            citation: "Barnard & Sibly 1981",
            text: "Watching is a scrounger strategy, worth less as it becomes common (pure scroungers, who never bury): the watcher advantage's per-seed slope on the watcher share (0.1–0.9) has a 95 % confidence interval wholly below 0",
            check: frequency_only_claim,
        },
        Claim {
            id: "watch-winter.usage",
            item: "watch-winter",
            source: Source::Comment,
            citation: SPEC,
            text: "Watching acts in every Minds 8 preset but the arena: some sugar is taken by a raid in at least 80 % of seeds (raid freshness, pilfers by source, the moves share and the mechanism reported)",
            check: usage_claim,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_worlds_are_the_presets() {
        for id in USAGE.iter().chain(&["watch-arena"]) {
            assert_eq!(at_span(id, ANCHOR), preset(id), "{id}");
            for span in SPANS {
                at_span(id, span).validate().expect("valid");
            }
        }
        assert_eq!(scroungers(0.5, ANCHOR), preset("watch-scroungers"));
        assert_eq!(
            scroungers_only(0.5, ANCHOR),
            preset("watch-scroungers-only")
        );
        for s in SHARES {
            for span in SPANS {
                let c = scroungers(s, span);
                c.validate().expect("valid");
                assert_eq!((c.theft.find, c.theft.cheaters), (0.0, 0.0));
                let c = scroungers_only(s, span);
                c.validate().expect("valid");
                assert_eq!((c.theft.find, c.theft.cheaters), (0.0, s));
            }
        }
        // Pure scroungers: at equal shares the watchers are the cheaters.
        let w = World::new(scroungers_only(0.3, ANCHOR), 1).unwrap();
        assert!(w.agents().all(|a| a.watches == a.cheater));
        // Claim 2's pair differs only by watching.
        let mut a = preset("watch-half");
        a.watching = Default::default();
        assert_eq!(a, preset("theft-winter-half"));
    }

    #[test]
    fn the_measures_follow_their_definitions() {
        let r = Run {
            founders_w: [10.0, 10.0],
            alive100_w: [8.0, 5.0],
            alive200_w: [6.0, 4.0],
            founders_c: [10.0, 10.0],
            alive200_c: [7.0, 3.0],
            dug: [0.0, 10.0],
            pilfered: [20.0, 10.0],
            lost: [0.0, 10.0],
            caches_pilfered: 9.0,
            candidates: 100.0,
            burials: [8.0, 2.0],
            burials_seen: [4.0, 1.0],
            sightings: [20.0, 0.0],
            raids: [4.0, 1.0],
            wasted: [1.0, 1.0],
            raid_age: vec![2.0, 0.0, 6.0],
            ..Run::default()
        };
        assert_eq!(r.surv_w(W), 0.6, "alive at 200 ÷ founders");
        assert_eq!(r.surv100_w(W), 0.75, "alive at 200 ÷ alive at 100");
        assert!((r.watcher_adv() - 0.2).abs() < 1e-12);
        assert!((r.hoarder_adv() - 0.4).abs() < 1e-12);
        assert_eq!(r.rate(), 0.09);
        assert_eq!(r.p_s(), 0.25, "dug ÷ (dug + pilfered)");
        assert_eq!(r.p_o(), 0.6, "pilfered ÷ everything ended");
        assert!((r.ps_po() - 0.25 / 0.6).abs() < 1e-12);
        assert_eq!(r.seen_per_burial(), 0.5);
        assert_eq!(r.raids_per_sighting(), 0.25);
        assert_eq!(r.wasted_per_sighting(), 0.1);
        assert_eq!(r.raid_age_mean(), 1.5);
        assert_eq!(r.raid_age_share(2, usize::MAX), 0.75);
        // p_o = 0: infinite with p_s > 0; no ended sugar: undefined.
        let dug_only = Run {
            dug: [5.0, 0.0],
            ..Run::default()
        };
        assert_eq!(dug_only.ps_po(), f64::INFINITY);
        assert!(Run::default().ps_po().is_nan());
    }

    #[test]
    fn claim_2s_flags_follow_the_spec() {
        let world = |dug: f64, pilfered: f64, h: f64, c: f64| Run {
            dug: [dug, 0.0],
            pilfered: [pilfered, 0.0],
            founders_c: [10.0, 10.0],
            alive200_c: [h, c],
            ..Run::default()
        };
        let theft = vec![
            world(1.0, 9.0, 8.0, 9.0),
            world(1.0, 0.0, 8.0, 9.0),
            world(1.0, 0.0, 8.0, 9.0),
            world(1.0, 9.0, 8.0, 9.0),
        ];
        let watch = vec![
            world(1.0, 19.0, 7.0, 9.0), // lower ratio, lower advantage
            world(1.0, 9.0, 9.0, 9.0),  // p_o = 0 under theft only: lower
            world(1.0, 0.0, 8.0, 9.0),  // p_o = 0 under both: no value
            world(1.0, 0.0, 8.0, 9.0),  // p_o = 0 under watching only: higher
        ];
        let (ratio, adv) = threshold_flags(&theft, &watch);
        assert_eq!(ratio[..2], [1.0, 1.0]);
        assert!(ratio[2].is_nan());
        assert_eq!(ratio[3], 0.0);
        assert_eq!(adv, vec![1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn the_judges_helpers() {
        let down = [-1.0, -1.1, -0.9, -1.2, -1.0];
        assert_eq!(ci_below_zero(&down).verdict, Verdict::Holds);
        let flat = [0.1, -0.1, 0.05, -0.05, 0.0, 0.02];
        assert_eq!(ci_below_zero(&flat).verdict, Verdict::Fails);
        let up = [1.0, 1.1, 0.9, 1.2, 1.0];
        assert_eq!(ci_below_zero(&up).verdict, Verdict::Fails);
        assert_eq!(ci_below_zero(&down[..3]).verdict, Verdict::Untestable);
        assert_eq!(
            down_crossing(&[0.1, 0.2, 0.3], &[1.0, 0.5, -0.5]),
            Some(0.25)
        );
        assert_eq!(down_crossing(&[0.1, 0.2], &[-1.0, 1.0]), None);
        assert_eq!(down_crossing(&[0.1, 0.2], &[1.0, 0.0]), Some(0.2));
    }
}
