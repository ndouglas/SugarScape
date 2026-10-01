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
//!
//! # The judges
//!
//! The judges below are the spec's "Questions and judges", word for word,
//! committed alone before any survey run ("Minds 8b: second-round judges,
//! committed before any run"). Thresholds are never tuned. The spec's
//! pre-mortem, copied here as the spec requires:
//!
//! ## The pre-mortem
//!
//! This is the step the first round lacked. For each judge below, the table gives what each hypothesis
//! predicts, checked against Minds 6, the first round and the audits' numbers. Where a number was already
//! seen, it says so.
//!
//! | Judge | If true | If false | Already seen? |
//! |---|---|---|---|
//! | 1a | Watching's fresh-cache hazard is above stumbling's. | It is at or below stumbling's. | Partly. Under `raid_if: always` the audit estimated about 4.9 %/day for watching against "about half" for stumbling. Under `better`, raids fall about 20 % (audit). The direction is likely but not certain. Disclosed. |
//! | 2a | Watching flips A&K's condition in a world where it held. | It doesn't flip. | No. No world with p_s > p_o has been run with watching. |
//! | 2b | The sign of the hoarders' advantage follows the sign of p_s/p_o − 1 across the span sweep. | The signs don't agree. | No. |
//! | 2c | Being watched harms hoarders. | It doesn't. | No, since the factorial hasn't been run. |
//! | 2d | Hoarders' own raiding harms them. | It doesn't. | No. |
//! | 3a | Scrounger advantage falls with share. | It is flat or rises. | Not for `forgo`. For watchers who also bury, the first round saw negative slopes at every span (not detected). |
//! | 3b | There is a stable mix: scroungers fitter when rare, less fit when common. | No crossing. | No for `forgo`. |
//! | 3c | A social dilemma: watchers individually ahead while the world's survival falls with share. | One part or both fail. | Yes, under `always`, by the design audit (survival 0.714 → 0.551). Under `better` the audit's variant cut the lead to 1 point. Judged, but flagged as partly seen. |
//!
//! Floors, ceilings and caps checked:
//! - **Claim 2's world must have p_s > p_o before watching.** That's the precondition (below).
//! - **Survival at tick 200 saturates near 1 in arenas and floors near 0.46** where nobody caches. Fitness is now
//!   agent-ticks alive per founder over ticks 1–200, divided by 200, which is continuous.
//! - **The carrying limit capped scrounger loot.** The scrounger world uses `loot: eat`.
//! - **In an arena nearly every agent survives,** so ticks alive can't discriminate there. For claim 2 in an arena
//!   world, fitness is wealth per founder at tick 200, Minds 6's arena rule (holdings, caches and stomach of the
//!   living ÷ founders). In a field world it is ticks alive per founder.
//!
//! **Setup.** 200 ticks. Claims 1 and 2 run seeds 1–20 and claim 3 seeds
//! 1–60, whatever `--seeds` says (the spec fixes them, as for the
//! calibration). Span 7 is judged. Thresholds are ≥ 80 % of seeds: 16 of
//! 20, 48 of 60. Verdicts are Holds, Fails, Untestable (a precondition not
//! met, or fewer than 5 seeds with a value, as in Minds 6–8) or
//! Inconclusive (claim 3a only). `Verdict::Inconclusive` was added for 3a
//! (`claim.rs`); like Weak it is not a hold, and `all_of` ranks it between
//! Weak and Untestable. No judge here reads Weak: "in ≥ 16 of 20 seeds"
//! holds or fails. A seed whose value is undefined does not count toward
//! "≥ 16 of 20" (it is not a hold); 2b, whose spec excludes undefined runs,
//! counts the share of the runs left.
//!
//! **Fitness** in claims 2 and 3 is the world's kind (see above): `watch-ak`
//! is calibration item 4, a field world (commit b966312), so ticks alive per
//! founder; claim 3's winter fields too. Fitness units are those.
//!
//! ## Claim 1: fresh caches (`watch-winter.fresh`)
//!
//! **The cohort hazard,** from the fate log (`World::record_fates`, which
//! draws nothing and changes no world; a run whose log reached its cap,
//! `cache_log_full`, is skipped and counted, as in Minds 6).
//! - **A cache** is a (site, owner) cache: one owner's cache at one site,
//!   from the burial that creates it (the owner had none there) until it is
//!   emptied (dug, taken or lost). From the log, an (owner, site)'s records
//!   in burial order form one cache while each next record was buried no
//!   later than the tick the cache's records so far last ended (a record
//!   still buried never ends); a record buried after that starts a new cache.
//!   So a cache emptied and buried again at the same site on the same tick
//!   reads as one cache (rare: the owner would have to bury on the tick it
//!   was emptied). A later cache at the same site is a new cache.
//! - **"First created in ticks 1–90".** The world at tick t is the world
//!   after t steps (the convention of "ticks 1–200" above), and a burial
//!   during the step from tick t to t + 1 has log tick t. A cache is
//!   created in tick t + 1 when its first record has log tick t, so the
//!   cohort is the caches whose first record has log tick 0–89. Ages are
//!   differences of log ticks: a take on the tick of creation is at age 0.
//!   The last cohort cache can be followed for 7 ticks, to log tick 96,
//!   before winter (log tick 100).
//! - **Taken within d** (d = 1, 3, 7): some record of the cache has the fate
//!   `Pilfered { by }`, with `by` not the owner, at a log tick at most d
//!   after its creation. A cache dug up or lost before any pilfer has
//!   ended, so it can't be taken and counts as not taken; a part dug while
//!   the rest stays buried doesn't end it.
//! - P_d = taken within d ÷ the cohort; h = 1 − (1 − P_7)^(1/7). Undefined
//!   with an empty cohort or a full log.
//! - The first round's stock rate (Σ `caches_pilfered` ÷ Σ
//!   `pilfer_candidates`), kept as context, counts only the caches present
//!   at a tick's start, so a cache buried and taken within one tick is
//!   missed, and the winter stock, which watching can't reach, dilutes it
//!   (the first-round audit). The cohort hazard has neither problem: every
//!   cache created in the window is followed from its creation.
//!
//! **1a** holds when h(`watch-winter`) > h(`theft-winter`), paired on
//! seeds, in ≥ 16 of 20 seeds (an undefined h on either side is not a
//! hold).
//!
//! **The bottleneck** (reported, classified by the rule fixed in the spec):
//! - **P(seen)** = burials seen ÷ burials. Burials seen is Σ `burials_seen`
//!   (burials with at least one watcher). Burials are the log's distinct
//!   (owner, site, burial tick) triples: an agent buries at most once a
//!   tick, where it stands, a burial opens a record whenever watching or
//!   theft is on, and splits keep their burial tick. (A backfill would add
//!   one, but these worlds pilfer from tick 0, so none occurs.)
//! - **Caches seen**: caches (as above) with at least one burial into them
//!   seen by a watcher. A sighting is read after each tick from the agents'
//!   memories: an entry (site, owner) last seen on that tick's log tick
//!   (the burial and the entry share it). A watcher that dies on the tick
//!   of the sighting takes its entry with it; the count of distinct
//!   sightings against Σ `burials_seen` is reported as a check.
//! - **Raided within span**: some record of a cache seen has the fate
//!   `Pilfered { by }` from a raid, not a stumble. A take by agent X from
//!   owner O at site S on a tick is a raid when, at the tick's start, X held
//!   a fresh entry (age ≤ `span`) for (S, O), and after the tick X holds no
//!   entry at S: a raid forgets the site's entries, while an arrival that
//!   declines to raid (`raid_if: better`) keeps them and may stumble.
//!   Nobody can raid a cache on the tick it is seen buried (its owner stands
//!   on it), so the tick-start entries miss no raid. Raids act only on
//!   fresh entries, so every raid is within `span` of the raider's own
//!   sighting. A raider that dies on the tick of its raid is counted as a
//!   stumble (rare: the raid fed it).
//! - **P(raid | seen)** = caches seen raided within span ÷ caches seen.
//! - **"Knowledge-bound"** iff P(seen) < P(raid | seen), else
//!   "action-bound". A world is classified on the medians over seeds; the
//!   per-seed counts are reported beside it.
//!
//! ## Claim 2: Andersson and Krebs under watching
//!
//! The calibration (above) chose item 4, so `watch-ak` exists and 2a–2d are
//! judged; had none qualified they would read Untestable. Watching off is
//! `watch-ak` with `watching.on` false (calibration item 4 itself).
//! - **p_s ÷ p_o** is Minds 8's (`minds8::ps_po`): infinite when p_o = 0
//!   with p_s > 0 (p_s > p_o holds), undefined when no sugar ended or both
//!   are 0.
//! - **2a** (`watch-ak.flip`): in `watch-ak`, p_s ÷ p_o < 1 in ≥ 16 of 20
//!   seeds (undefined or infinite: not below 1).
//! - **2b** (`watch-ak.sign`): the runs of watching off and of `watch-ak` at
//!   spans 1, 3, 7 and 13, seeds 1–20 (100 runs). Per run, the sign (−, 0
//!   or +) of the hoarders' advantage (hoarder − cheater fitness) against
//!   the sign of p_s ÷ p_o − 1 (+ when infinite). A run whose ratio is
//!   undefined is excluded, and so is one whose advantage is (a group with
//!   no founders; none in this world). Holds when the signs are equal in
//!   ≥ 80 % of the runs left; a zero agrees only with a zero.
//! - **The factorial**: `watch-ak`'s world with watching off (`who: none`),
//!   `who: hoarders`, `who: cheaters` and everyone (`watch-ak` itself).
//! - **2c** (`watch-ak.watched`): per seed, the hoarders' advantage under
//!   `who: cheaters` is below its value with watching off by at least 0.05
//!   (off − cheaters ≥ 0.05), in ≥ 16 of 20 seeds.
//! - **2d** (`watch-ak.raiding`): per seed, hoarder fitness under `who:
//!   hoarders` is below its value with watching off by at least 0.05, in
//!   ≥ 16 of 20 seeds. 2c and 2d may both hold.
//!
//! ## Claim 3: producers and scroungers (seeds 1–60)
//!
//! Variant forgo: `watch-scroungers-forgo` with `cheaters` = `watchers` = s
//! (the same agents, the scroungers); variant bury: `watch-scroungers` with
//! `watchers` = s and no cheaters; s = 0.1, 0.2, …, 0.9. The advantage is
//! watcher (scrounger) − other fitness; the world's fitness is everyone's.
//! - **3a** (one claim id per variant: `watch-scroungers.frequency-forgo`
//!   and `watch-scroungers.frequency-bury`; the spec's stem
//!   `watch-scroungers.frequency` is the first round's claim id, which stays
//!   on record): the OLS slope of the advantage on s per seed (Minds 6's
//!   `seed_slopes`); the 95 % and 90 % t intervals of the mean slope.
//!   **Holds** when the 95 % interval lies wholly below 0 (upper < 0);
//!   otherwise **Flat**, reported as Fails labeled "flat", when the 90 %
//!   interval lies within ±0.05 (−0.05 ≤ lower and upper ≤ 0.05); otherwise
//!   **Fails, labeled "rising"**, when the 95 % interval lies wholly above 0
//!   (the pre-mortem's "flat or rises"; clarified in the spec before any
//!   run); otherwise **Inconclusive**. The order is Holds, flat, rising,
//!   Inconclusive: a tight negative slope inside ±0.05 Holds, and an
//!   interval both flat and above 0 (all inside (0, 0.05]) reads flat.
//!   Fewer than 5 finite slopes: Untestable.
//! - **3b** (`watch-scroungers.mix`, forgo only): the advantage > 0 at
//!   s = 0.1 in ≥ 48 of 60 seeds, and < 0 at s = 0.9 in ≥ 48 of 60 seeds;
//!   both. The per-seed crossing (Minds 8's `down_crossing`: the first sign
//!   change from above 0 to at most 0, interpolated) is reported.
//! - **3c** (`watch-scroungers.dilemma`, bury only): both of: the watchers'
//!   advantage pooled over shares (per seed, the mean over the shares with a
//!   value) has a 95 % t interval wholly above 0; and the per-seed OLS slope
//!   of the world's fitness on s has a 95 % interval wholly below 0.
//!
//! ## Usage, and what is reported
//!
//! - **Usage** is a check, not a claim: `survey --usage` writes
//!   `survey/out/minds8b-usage.md`. In every preset with watching on, raided
//!   sugar (Σ `raided`) is above 0 in ≥ 16 of 20 seeds.
//! - **Reported, not judged**, in each claim's detail: every claim under
//!   `raid_if: always` and `value: room` (at span 7) and at spans 1, 2, 3, 7
//!   and 13; the first round's probe (`World::probe_raid_harvests`, survey
//!   only: a raid that took something also harvests its site) for
//!   `watch-winter`, `watch-winter-stumble` and `watch-ak`; P_1, P_3, P_7 and
//!   h for `watch-winter`, `watch-winter-stumble` and `theft-winter`;
//!   Heinrich and Pepper's next-day figure and Vander Wall and Jenkins's
//!   band as context; the bottleneck; the stock rate.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::Mutex;

use sugarscape_core::agent::{Agent, AgentId};
use sugarscape_core::config::{Config, DigBelow, RaidIf, Scrounge, SeenValue, Who};
use sugarscape_core::minds::caching::fates::{CacheRecord, Fate};
use sugarscape_core::presets;
use sugarscape_core::world::World;

use crate::claim::{all_of, untestable, Claim, Outcome, Source, Verdict};
use crate::claims::minds4::list;
use crate::claims::minds5::med;
use crate::claims::minds6::{ci95, col, flag, m, nan_div, p_o, p_s, seed_slopes, t_crit, wealth};
use crate::claims::minds8::{down_crossing, ps_po};
use crate::runner::{each_seed, preset};
use crate::stats;

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
pub(crate) const W: usize = 0;
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

/// What a run records beyond its config.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Setup {
    /// The fate log, sightings and raids (claim 1's measures).
    pub(crate) fates: bool,
    /// `World::probe_raid_harvests`.
    pub(crate) probe: bool,
}

/// The d of P_d: 1, 3 and 7 ticks.
pub(crate) const DS: [u64; 3] = [1, 3, 7];

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
    /// Raids and raided sugar (tick events).
    pub(crate) raids: f64,
    pub(crate) raided: f64,
    /// The first round's stock rate: Σ `caches_pilfered`, Σ
    /// `pilfer_candidates`.
    pub(crate) caches_pilfered: f64,
    pub(crate) candidates: f64,
    /// Σ `burials_seen`.
    pub(crate) burials_seen: f64,
    /// Whether the fate log reached its cap (its measures are then NaN).
    pub(crate) log_full: bool,
    /// From the fate log (NaN unless `Setup::fates` and the log is whole).
    pub(crate) fresh: Fresh,
    /// Distinct sightings read from the memories (the check against
    /// `burials_seen`).
    pub(crate) sighted: f64,
    /// The group's living founders at ticks 100 and 200 (reported only:
    /// the presets' survival, `presets_report`).
    pub(crate) alive100_c: [f64; 2],
    pub(crate) alive100_w: [f64; 2],
    pub(crate) alive200_c: [f64; 2],
    pub(crate) alive200_w: [f64; 2],
}

/// Claim 1's measures from the fate log.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Fresh {
    /// Caches created in ticks 1–90.
    pub(crate) cohort: f64,
    /// Of those, taken by a non-owner within d = 1, 3, 7 ticks.
    pub(crate) taken: [f64; 3],
    /// Distinct (owner, site, burial tick).
    pub(crate) burials: f64,
    /// Caches with at least one burial seen, and of those, raided.
    pub(crate) caches_seen: f64,
    pub(crate) seen_raided: f64,
    /// Distinct takes (raider, site, owner, tick) in the log classified as
    /// raids: the check against Σ `raids`.
    pub(crate) raid_takes: f64,
}

impl Default for Fresh {
    fn default() -> Self {
        Self {
            cohort: f64::NAN,
            taken: [f64::NAN; 3],
            burials: f64::NAN,
            caches_seen: f64::NAN,
            seen_raided: f64::NAN,
            raid_takes: f64::NAN,
        }
    }
}

/// Field fitness from a group's agent-ticks alive and founders.
fn field(ticks_alive: f64, founders: f64) -> f64 {
    nan_div(ticks_alive, founders) / TICKS as f64
}

/// Arena fitness from a group's wealth at 200 and founders.
fn arena(wealth200: f64, founders: f64) -> f64 {
    nan_div(wealth200, founders)
}

/// The hazard from P_7: h = 1 − (1 − P_7)^(1/7).
pub(crate) fn hazard(p7: f64) -> f64 {
    1.0 - (1.0 - p7).powf(1.0 / 7.0)
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
    pub(crate) fn fit_all(&self, kind: Fitness) -> f64 {
        let sum = |x: [f64; 2]| x[0] + x[1];
        match kind {
            Fitness::Field => field(sum(self.ticks_alive_c), sum(self.founders_c)),
            Fitness::Arena => arena(sum(self.wealth200_c), sum(self.founders_c)),
        }
    }
    /// The hoarders' advantage: hoarder − cheater fitness.
    pub(crate) fn hoarder_adv(&self, kind: Fitness) -> f64 {
        self.fit_c(kind, H) - self.fit_c(kind, C)
    }
    /// The watchers' advantage: watcher − other fitness.
    pub(crate) fn watcher_adv(&self, kind: Fitness) -> f64 {
        self.fit_w(kind, W) - self.fit_w(kind, O)
    }
    pub(crate) fn p_s(&self) -> f64 {
        p_s(self.dug, self.pilfered)
    }
    pub(crate) fn p_o(&self) -> f64 {
        p_o(self.dug, self.pilfered, self.lost)
    }
    /// p_s ÷ p_o (Minds 8's: infinite when p_o = 0 with p_s > 0).
    pub(crate) fn ratio(&self) -> f64 {
        ps_po(self.p_s(), self.p_o())
    }
    /// P_d for d = `DS[i]`.
    pub(crate) fn p_d(&self, i: usize) -> f64 {
        nan_div(self.fresh.taken[i], self.fresh.cohort)
    }
    /// The cohort hazard.
    pub(crate) fn hazard(&self) -> f64 {
        hazard(self.p_d(2))
    }
    pub(crate) fn p_seen(&self) -> f64 {
        nan_div(self.burials_seen, self.fresh.burials)
    }
    pub(crate) fn p_raid_seen(&self) -> f64 {
        nan_div(self.fresh.seen_raided, self.fresh.caches_seen)
    }
    /// The first round's stock rate.
    pub(crate) fn stock_rate(&self) -> f64 {
        nan_div(self.caches_pilfered, self.candidates)
    }
}

/// A pilfer taken by a raid: (raider, site, owner, log tick).
type RaidKey = (AgentId, u32, AgentId, u64);
/// A burial seen: (owner, site, log tick).
type SightKey = (AgentId, u32, u64);

pub(crate) fn measure(mut w: World, setup: Setup) -> Run {
    w.record_fates = setup.fates;
    w.probe_raid_harvests = setup.probe;
    let torus = w.torus;
    let span = u64::from(w.config.watching.span);
    let watching = w.config.watching.on;
    let gc = |a: &Agent| usize::from(a.cheater);
    let gw = |a: &Agent| usize::from(!a.watches);
    let mut r = Run::default();
    let founders: HashSet<AgentId> = w.agents().map(|a| a.id).collect();
    for a in w.agents() {
        r.founders_c[gc(a)] += 1.0;
        r.founders_w[gw(a)] += 1.0;
    }
    let mut sighted: HashSet<SightKey> = HashSet::new();
    let mut raids: HashSet<RaidKey> = HashSet::new();
    while w.tick < TICKS {
        let t0 = w.tick;
        // The tick's start: each agent's stolen sugar and fresh entries.
        let mut fresh0: HashMap<AgentId, (f64, Vec<(u32, AgentId)>)> = HashMap::new();
        if setup.fates && watching {
            for a in w.agents() {
                let f: Vec<(u32, AgentId)> = a
                    .seen
                    .iter()
                    .filter(|(_, e)| t0.saturating_sub(e.tick) <= span)
                    .map(|(&k, _)| k)
                    .collect();
                if !f.is_empty() {
                    fresh0.insert(a.id, (a.stolen_by_me, f));
                }
            }
        }
        w.step();
        let e = w.events();
        r.dug += e.dug;
        r.pilfered += e.pilfered;
        r.lost += e.cache_lost;
        r.raids += f64::from(e.raids);
        r.raided += e.raided;
        r.caches_pilfered += f64::from(e.caches_pilfered);
        r.candidates += f64::from(e.pilfer_candidates);
        r.burials_seen += f64::from(e.burials_seen);
        for a in w.agents().filter(|a| founders.contains(&a.id)) {
            r.ticks_alive_c[gc(a)] += 1.0;
            r.ticks_alive_w[gw(a)] += 1.0;
            if w.tick == 100 {
                r.alive100_c[gc(a)] += 1.0;
                r.alive100_w[gw(a)] += 1.0;
            }
            if w.tick == TICKS {
                r.wealth200_c[gc(a)] += wealth(a);
                r.wealth200_w[gw(a)] += wealth(a);
                r.alive200_c[gc(a)] += 1.0;
                r.alive200_w[gw(a)] += 1.0;
            }
        }
        if setup.fates && watching {
            for a in w.agents() {
                for (&(site, owner), e) in &a.seen {
                    if e.tick == t0 {
                        sighted.insert((owner, site, t0));
                    }
                }
                let Some((by0, f)) = fresh0.get(&a.id) else {
                    continue;
                };
                if a.stolen_by_me > *by0 {
                    let site = torus.index(a.pos) as u32;
                    if !a.seen.keys().any(|&(s, _)| s == site) {
                        for &(s, owner) in f {
                            if s == site {
                                raids.insert((a.id, site, owner, t0));
                            }
                        }
                    }
                }
            }
        }
    }
    if setup.fates {
        r.log_full = w.cache_log_full;
        r.sighted = sighted.len() as f64;
        if !r.log_full {
            r.fresh = from_log(&w.cache_log, &sighted, &raids);
        }
    }
    r
}

/// A (site, owner) cache from the log: its creation (log tick) and records.
#[derive(Debug)]
struct Cache<'a> {
    created: u64,
    records: Vec<&'a CacheRecord>,
}

/// The log's caches (see the module's list): each (owner, site)'s records in
/// burial order, a new cache starting at a record buried after every record
/// so far has ended.
fn caches(log: &[CacheRecord]) -> Vec<Cache<'_>> {
    let mut by_key: BTreeMap<(AgentId, u32), Vec<(usize, &CacheRecord)>> = BTreeMap::new();
    for (i, rec) in log.iter().enumerate() {
        by_key
            .entry((rec.owner, rec.site))
            .or_default()
            .push((i, rec));
    }
    let mut out = Vec::new();
    for (_, mut recs) in by_key {
        recs.sort_by_key(|&(i, rec)| (rec.buried, i));
        let mut cur: Option<(Cache, Option<u64>)> = None;
        for (_, rec) in recs {
            let ends = rec.fate.map(|(t, _)| t);
            match &mut cur {
                Some((c, end)) if end.is_none_or(|e| rec.buried <= e) => {
                    c.records.push(rec);
                    *end = match (*end, ends) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        _ => None,
                    };
                }
                _ => {
                    if let Some((c, _)) = cur.take() {
                        out.push(c);
                    }
                    cur = Some((
                        Cache {
                            created: rec.buried,
                            records: vec![rec],
                        },
                        ends,
                    ));
                }
            }
        }
        if let Some((c, _)) = cur {
            out.push(c);
        }
    }
    out
}

/// The last log tick of claim 1's cohort: caches created in ticks 1–90 have
/// first records at log ticks 0–89.
const COHORT_LAST: u64 = 89;

/// Claim 1's measures from a whole log, the sightings and the raids.
fn from_log(log: &[CacheRecord], sighted: &HashSet<SightKey>, raids: &HashSet<RaidKey>) -> Fresh {
    let burials: HashSet<SightKey> = log.iter().map(|r| (r.owner, r.site, r.buried)).collect();
    let mut f = Fresh {
        cohort: 0.0,
        taken: [0.0; 3],
        burials: burials.len() as f64,
        caches_seen: 0.0,
        seen_raided: 0.0,
        raid_takes: 0.0,
    };
    let takes: HashSet<RaidKey> = log
        .iter()
        .filter_map(|r| match r.fate {
            Some((t, Fate::Pilfered { by })) => Some((by, r.site, r.owner, t)),
            _ => None,
        })
        .filter(|k| raids.contains(k))
        .collect();
    f.raid_takes = takes.len() as f64;
    for c in caches(log) {
        let pilfers = || {
            c.records.iter().filter_map(|r| match r.fate {
                Some((t, Fate::Pilfered { by })) if by != r.owner => Some((t, by, r)),
                _ => None,
            })
        };
        if c.created <= COHORT_LAST {
            f.cohort += 1.0;
            let first = pilfers().map(|(t, _, _)| t - c.created).min();
            for (i, d) in DS.iter().enumerate() {
                if first.is_some_and(|age| age <= *d) {
                    f.taken[i] += 1.0;
                }
            }
        }
        if c.records
            .iter()
            .any(|r| sighted.contains(&(r.owner, r.site, r.buried)))
        {
            f.caches_seen += 1.0;
            if pilfers().any(|(t, by, r)| raids.contains(&(by, r.site, r.owner, t))) {
                f.seen_raided += 1.0;
            }
        }
    }
    f
}

/// Cached runs: (key, seeds, runs).
type RunCache = Vec<(String, Vec<u64>, Vec<Run>)>;

/// Runs of `c` under `setup`, shared by the claims.
pub(crate) fn runs_of(c: &Config, setup: Setup, seeds: &[u64]) -> Vec<Run> {
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
    let out = each_seed(c, seeds, |w| measure(w, setup));
    CACHE
        .lock()
        .unwrap()
        .push((key, seeds.to_vec(), out.clone()));
    out
}

pub(crate) fn runs(c: &Config, seeds: &[u64]) -> Vec<Run> {
    runs_of(c, Setup::default(), seeds)
}

/// Runs with the fate log, sightings and raids.
fn logged(c: &Config, seeds: &[u64]) -> Vec<Run> {
    runs_of(
        c,
        Setup {
            fates: true,
            probe: false,
        },
        seeds,
    )
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

// ------------------------------------------------------------- the judges

/// Claims 1 and 2's seeds (1–20) and claim 3's (1–60).
fn seeds_upto(n: u64) -> Vec<u64> {
    (1..=n).collect()
}
const SEEDS_3: u64 = 60;
/// Claim 3's threshold: ≥ 48 of 60 seeds.
const QUALIFY_3: usize = 48;
/// The share threshold (2b).
const SHARE: f64 = 0.8;
/// Claims 2c and 2d: the least drop, in fitness units.
const DROP: f64 = 0.05;
/// Claim 3a: the flat band.
const FLAT: f64 = 0.05;
/// The judged span.
const ANCHOR: u32 = 7;
/// Claim 2b's spans (with watching off).
const SIGN_SPANS: [u32; 4] = [1, 3, 7, 13];
/// The reported spans.
const SPANS: [u32; 5] = [1, 2, 3, 7, 13];
/// Claim 3's shares.
const SHARES: [f64; 9] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
/// Fewer seeds (or runs) with a value than this: Untestable (Minds 6–8).
const MIN_VALUES: usize = 5;

/// The calibration's committed result (b966312): item 4 (index 3). `None`
/// would make claims 2a–2d Untestable.
const WATCH_AK_ITEM: Option<usize> = Some(3);

/// `watch-ak`'s fitness: the chosen item's kind.
fn ak_fitness() -> Option<Fitness> {
    WATCH_AK_ITEM.map(|i| CALIBRATION[i].fitness)
}

/// Holds when at least `need` of the per-seed flags (1.0, 0.0, or NaN for
/// no value) are 1; a seed with no value is not a hold. Fewer than 5 seeds
/// with a value: Untestable.
pub(crate) fn at_least(flags: &[f64], need: usize, what: &str) -> Outcome {
    let valued = stats::finite(flags).len();
    if valued < MIN_VALUES {
        return untestable(&format!(
            "only {valued} seeds gave a value (need {MIN_VALUES})"
        ));
    }
    let n = flags.iter().filter(|&&f| f == 1.0).count();
    Outcome {
        verdict: if n >= need {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!(
            "{what} in {n} of {} seeds (need {need}; {} without a value)",
            flags.len(),
            flags.len() - valued
        ),
        detail: String::new(),
    }
}

/// Holds when at least 80 % of the runs with a value (flags 1.0 or 0.0;
/// NaN excluded) are 1. Fewer than 5: Untestable.
pub(crate) fn share_of(flags: &[f64], what: &str) -> Outcome {
    let v = stats::finite(flags);
    if v.len() < MIN_VALUES {
        return untestable(&format!(
            "only {} runs gave a value (need {MIN_VALUES})",
            v.len()
        ));
    }
    let n = v.iter().filter(|&&f| f == 1.0).count();
    let share = n as f64 / v.len() as f64;
    Outcome {
        verdict: if share >= SHARE {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!(
            "{what} in {n} of {} runs, {:.1} % (need 80 %; {} excluded)",
            v.len(),
            100.0 * share,
            flags.len() - v.len()
        ),
        detail: String::new(),
    }
}

/// (mean, lower, upper) of the 90 % t interval of the finite `v`.
pub(crate) fn ci90(v: &[f64]) -> (f64, f64, f64) {
    let v = stats::finite(v);
    let n = v.len() as f64;
    let mu = stats::mean(&v);
    let sd = (v.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let h = t_crit(0.95, n - 1.0) * sd / n.sqrt();
    (mu, mu - h, mu + h)
}

/// The sign of `x`: −1, 0 or 1; None when NaN (∞ has a sign).
fn sign(x: f64) -> Option<i8> {
    if x.is_nan() {
        None
    } else if x > 0.0 {
        Some(1)
    } else if x < 0.0 {
        Some(-1)
    } else {
        Some(0)
    }
}

// --------------------------------------------------------------- variants

/// A reported setting of the watching switches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Variant {
    pub(crate) span: u32,
    pub(crate) raid_if: RaidIf,
    pub(crate) value: SeenValue,
}

/// The judged setting: span 7, `raid_if: better`, `value: amount`.
pub(crate) const JUDGED: Variant = Variant {
    span: ANCHOR,
    raid_if: RaidIf::Better,
    value: SeenValue::Amount,
};

impl Variant {
    const fn at_span(span: u32) -> Self {
        Self { span, ..JUDGED }
    }
    fn label(&self) -> String {
        let mut s = format!("span {}", self.span);
        if self.raid_if != JUDGED.raid_if {
            s.push_str(", raid_if always");
        }
        if self.value != JUDGED.value {
            s.push_str(", value room");
        }
        if *self == JUDGED {
            s.push_str(" (judged)");
        }
        s
    }
    /// `c` under this setting; a world without watching is unchanged.
    pub(crate) fn apply(&self, mut c: Config) -> Config {
        if c.watching.on {
            c.watching.span = self.span;
            c.watching.raid_if = self.raid_if;
            c.watching.value = self.value;
        }
        c
    }
}

/// The reported settings: `raid_if: always` and `value: room` at span 7,
/// then spans 1, 2, 3, 7 (the judged one) and 13.
pub(crate) fn reported() -> Vec<Variant> {
    let mut v = vec![
        Variant {
            raid_if: RaidIf::Always,
            ..JUDGED
        },
        Variant {
            value: SeenValue::Room,
            ..JUDGED
        },
    ];
    v.extend(SPANS.iter().map(|&s| Variant::at_span(s)));
    v
}

// ----------------------------------------------------------------- formats

fn pc(x: f64) -> String {
    if x.is_nan() {
        "undefined".into()
    } else {
        format!("{:.2} %", 100.0 * x)
    }
}

/// Median over seeds as a percentage.
fn mp(v: &[f64]) -> String {
    pc(m(v))
}

// ------------------------------------------------------- 1. fresh caches

/// 1a's per-seed flags: h(watch) > h(theft); NaN when either is undefined.
pub(crate) fn fresh_flags(watch: &[f64], theft: &[f64]) -> Vec<f64> {
    watch
        .iter()
        .zip(theft)
        .map(|(&w, &t)| {
            if w.is_nan() || t.is_nan() {
                f64::NAN
            } else {
                flag(w > t)
            }
        })
        .collect()
}

/// The bottleneck rule: knowledge-bound iff P(seen) < P(raid | seen).
pub(crate) fn bottleneck(p_seen: f64, p_raid_seen: f64) -> &'static str {
    if p_seen.is_nan() || p_raid_seen.is_nan() {
        "unclassified"
    } else if p_seen < p_raid_seen {
        "knowledge-bound"
    } else {
        "action-bound"
    }
}

/// A world's claim 1 measures: P_1, P_3, P_7, h, the cohort, the
/// bottleneck and the stock rate.
fn fresh_row(label: &str, r: &[Run]) -> String {
    let full = r.iter().filter(|x| x.log_full).count();
    let ps = col(r, Run::p_seen);
    let pr = col(r, Run::p_raid_seen);
    let knowledge = ps
        .iter()
        .zip(&pr)
        .filter(|(&a, &b)| bottleneck(a, b) == "knowledge-bound")
        .count();
    format!(
        "{label}: P_1 {}, P_3 {}, P_7 {}, h {} a day (medians over seeds); cohort per seed {}; P(seen) {}, P(raid | seen) {} (caches seen per seed {}): {} on the medians, knowledge-bound in {knowledge} of {} seeds; raids per seed {}, raided sugar {}; the first round's stock rate {}; logs skipped as full {full} of {}; checks (each should be about 1): sightings read ÷ burials seen {}, takes classified as raids ÷ raids {}",
        mp(&col(r, |x| x.p_d(0))),
        mp(&col(r, |x| x.p_d(1))),
        mp(&col(r, |x| x.p_d(2))),
        mp(&col(r, Run::hazard)),
        med(&col(r, |x| x.fresh.cohort)),
        med(&ps),
        med(&pr),
        med(&col(r, |x| x.fresh.caches_seen)),
        bottleneck(m(&ps), m(&pr)),
        r.len(),
        med(&col(r, |x| x.raids)),
        med(&col(r, |x| x.raided)),
        mp(&col(r, Run::stock_rate)),
        r.len(),
        med(&col(r, |x| nan_div(x.sighted, x.burials_seen))),
        med(&col(r, |x| nan_div(x.fresh.raid_takes, x.raids))),
    )
}

fn fresh_claim(_seeds: &[u64]) -> Outcome {
    let seeds = seeds_upto(SEEDS_1_2);
    let theft = logged(&preset("theft-winter"), &seeds);
    let at = |id: &str, v: Variant| logged(&v.apply(preset(id)), &seeds);
    let watch = at("watch-winter", JUDGED);
    let flags = fresh_flags(&col(&watch, Run::hazard), &col(&theft, Run::hazard));
    let rows = [
        fresh_row("watch-winter", &watch),
        fresh_row("watch-winter-stumble", &at("watch-winter-stumble", JUDGED)),
        fresh_row("theft-winter (stumbling only)", &theft),
    ];
    let sweep: Vec<String> = reported()
        .into_iter()
        .flat_map(|v| {
            ["watch-winter", "watch-winter-stumble"].map(|id| {
                let r = at(id, v);
                let f = fresh_flags(&col(&r, Run::hazard), &col(&theft, Run::hazard));
                format!(
                    "{}; h above theft-winter's in {} of {} seeds",
                    fresh_row(&format!("{id}, {}", v.label()), &r),
                    f.iter().filter(|&&x| x == 1.0).count(),
                    f.len()
                )
            })
        })
        .collect();
    let probe: Vec<String> = ["watch-winter", "watch-winter-stumble"]
        .iter()
        .map(|&id| {
            let r = runs_of(
                &preset(id),
                Setup {
                    fates: true,
                    probe: true,
                },
                &seeds,
            );
            format!(
                "{}; fitness (ticks alive per founder ÷ 200) {}",
                fresh_row(&format!("{id} with the probe"), &r),
                med(&col(&r, |x| x.fit_all(Fitness::Field))),
            )
        })
        .collect();
    at_least(&flags, QUALIFY, "h(watch-winter) > h(theft-winter)")
        .with(&format!(
            "Seeds 1–{SEEDS_1_2}, ticks 1–200; the cohort is each (site, owner) cache first created in ticks 1–90, from the fate log. Per seed h (% a day), watch-winter {}; theft-winter {}.",
            list(&col(&watch, |x| 100.0 * x.hazard()), 2),
            list(&col(&theft, |x| 100.0 * x.hazard()), 2),
        ))
        .with(&format!("Reported: {}.", rows.join("; ")))
        .with(&format!(
            "Under the reported settings: {}.",
            sweep.join("; ")
        ))
        .with(&format!(
            "The probe, not a setting (World::probe_raid_harvests: a raid that took something also harvests its site): {}.",
            probe.join("; ")
        ))
        .with("Context, not targets. Heinrich and Pepper's within-species next-day figure, for P_1: at most 15 of 42 caches, 36 %. Vander Wall and Jenkins's 2–30 % a day (2003, pp. 656, 658, 663) is loss of artificial caches to all pilferers, mostly rodents, per item per day; the same paper calls intraspecific pilferage \"usually low\" in jays and nutcrackers. The stock rate counts only caches present at a tick's start and is diluted by the winter stock.")
}

// --------------------------------------------- 2. Andersson and Krebs

/// `watch-ak`'s world with watching off.
fn ak_off() -> Config {
    let mut c = preset("watch-ak");
    c.watching.on = false;
    c
}

/// `watch-ak` with `who` set.
fn ak_who(who: Who) -> Config {
    let mut c = preset("watch-ak");
    c.watching.who = who;
    c
}

/// 2a's per-seed flags: p_s ÷ p_o < 1 (undefined or infinite: 0).
pub(crate) fn flip_flags(ratios: &[f64]) -> Vec<f64> {
    ratios.iter().map(|&x| flag(x < 1.0)).collect()
}

/// 2b's per-run flags: the sign of the advantage equals the sign of
/// p_s ÷ p_o − 1; NaN when either is undefined.
pub(crate) fn sign_flags(adv: &[f64], ratios: &[f64]) -> Vec<f64> {
    adv.iter()
        .zip(ratios)
        .map(|(&a, &q)| match (sign(a), sign(q - 1.0)) {
            (Some(x), Some(y)) => flag(x == y),
            _ => f64::NAN,
        })
        .collect()
}

/// 2c and 2d's per-seed flags: `off` − `on` ≥ 0.05; NaN when either is
/// undefined.
pub(crate) fn drop_flags(off: &[f64], on: &[f64]) -> Vec<f64> {
    off.iter()
        .zip(on)
        .map(|(&a, &b)| {
            if a.is_nan() || b.is_nan() {
                f64::NAN
            } else {
                flag(a - b >= DROP)
            }
        })
        .collect()
}

/// The four factorial worlds under a setting: off, hoarders, cheaters,
/// everyone.
struct Factorial {
    off: Vec<Run>,
    hoarders: Vec<Run>,
    cheaters: Vec<Run>,
    everyone: Vec<Run>,
}

fn factorial(v: Variant, seeds: &[u64]) -> Factorial {
    Factorial {
        off: runs(&ak_off(), seeds),
        hoarders: runs(&v.apply(ak_who(Who::Hoarders)), seeds),
        cheaters: runs(&v.apply(ak_who(Who::Cheaters)), seeds),
        everyone: runs(&v.apply(preset("watch-ak")), seeds),
    }
}

/// 2b's runs under a setting: watching off, then `watch-ak` at each span.
fn sign_runs(v: Variant, seeds: &[u64]) -> Vec<Run> {
    let mut out = runs(&ak_off(), seeds);
    for &span in &SIGN_SPANS {
        out.extend(runs(
            &Variant { span, ..v }.apply(preset("watch-ak")),
            seeds,
        ));
    }
    out
}

fn ak_row(label: &str, r: &[Run], k: Fitness) -> String {
    format!(
        "{label}: p_s {}, p_o {}, p_s ÷ p_o {} (below 1 in {} of {} seeds); hoarder fitness {}, cheater fitness {}, hoarders' advantage {}; raided sugar per seed {}",
        med(&col(r, Run::p_s)),
        med(&col(r, Run::p_o)),
        med(&col(r, Run::ratio)),
        r.iter().filter(|x| x.ratio() < 1.0).count(),
        r.len(),
        med(&col(r, |x| x.fit_c(k, H))),
        med(&col(r, |x| x.fit_c(k, C))),
        med(&col(r, |x| x.hoarder_adv(k))),
        med(&col(r, |x| x.raided)),
    )
}

/// The factorial's three judged comparisons as counts, for the reported
/// settings.
fn factorial_row(f: &Factorial, k: Fitness) -> String {
    let flip = flip_flags(&col(&f.everyone, Run::ratio));
    let watched = drop_flags(
        &col(&f.off, |x| x.hoarder_adv(k)),
        &col(&f.cheaters, |x| x.hoarder_adv(k)),
    );
    let raiding = drop_flags(
        &col(&f.off, |x| x.fit_c(k, H)),
        &col(&f.hoarders, |x| x.fit_c(k, H)),
    );
    let ones = |v: &[f64]| v.iter().filter(|&&x| x == 1.0).count();
    format!(
        "2a p_s ÷ p_o < 1 in {} of {}; 2c advantage dropped by ≥ 0.05 under who: cheaters in {} of {}; 2d hoarder fitness dropped by ≥ 0.05 under who: hoarders in {} of {}. {}; {}; {}; {}",
        ones(&flip),
        flip.len(),
        ones(&watched),
        watched.len(),
        ones(&raiding),
        raiding.len(),
        ak_row("watching off", &f.off, k),
        ak_row("who: hoarders", &f.hoarders, k),
        ak_row("who: cheaters", &f.cheaters, k),
        ak_row("everyone", &f.everyone, k),
    )
}

/// The reported settings for claim 2, in one string.
fn ak_reported(seeds: &[u64], k: Fitness) -> String {
    let mut out = Vec::new();
    for v in reported() {
        let mut s = format!("{}: {}", v.label(), factorial_row(&factorial(v, seeds), k));
        if v.span == ANCHOR {
            let r = sign_runs(v, seeds);
            let f = sign_flags(&col(&r, |x| x.hoarder_adv(k)), &col(&r, Run::ratio));
            s.push_str(&format!(
                "; 2b signs agree in {} of {} runs with a value",
                f.iter().filter(|&&x| x == 1.0).count(),
                stats::finite(&f).len()
            ));
        }
        out.push(s);
    }
    let probe = runs_of(
        &preset("watch-ak"),
        Setup {
            fates: false,
            probe: true,
        },
        seeds,
    );
    format!(
        "Under the reported settings (2b at span 7 only, its own sweep under each switch): {}. The probe, not a setting (a raid also harvests its site): {}.",
        out.join(". "),
        ak_row("watch-ak with the probe", &probe, k)
    )
}

/// The common detail of claims 2a–2d.
fn ak_context() -> &'static str {
    "Andersson and Krebs's condition with free burying is p_s > p_o (1978, p.708). watch-ak is the calibration's item 4 (theft-winter-half at find 0.02, owners digging below their whole reserve), with every agent watching; p_s and p_o are Minds 6's, amount-weighted, sugar still buried at 200 excluded. Fitness is ticks alive per founder over ticks 1–200 ÷ 200 (a field world)."
}

/// Runs `judge` when the calibration chose a world; Untestable otherwise.
fn with_ak(judge: impl Fn(Fitness, &[u64]) -> Outcome) -> Outcome {
    match ak_fitness() {
        Some(k) => judge(k, &seeds_upto(SEEDS_1_2)),
        None => untestable("the calibration found no world with p_s > p_o in 16 of 20 seeds"),
    }
}

fn flip_claim(_seeds: &[u64]) -> Outcome {
    with_ak(|k, seeds| {
        let r = runs(&preset("watch-ak"), seeds);
        at_least(&flip_flags(&col(&r, Run::ratio)), QUALIFY, "p_s ÷ p_o < 1")
            .with(&format!(
                "Per seed p_s ÷ p_o, watch-ak {}; watching off {}. {}; {}.",
                list(&col(&r, Run::ratio), 4),
                list(&col(&runs(&ak_off(), seeds), Run::ratio), 4),
                ak_row("watch-ak", &r, k),
                ak_row("watching off", &runs(&ak_off(), seeds), k),
            ))
            .with(&ak_reported(seeds, k))
            .with(ak_context())
    })
}

fn sign_claim(_seeds: &[u64]) -> Outcome {
    with_ak(|k, seeds| {
        let r = sign_runs(JUDGED, seeds);
        let flags = sign_flags(&col(&r, |x| x.hoarder_adv(k)), &col(&r, Run::ratio));
        let n = seeds.len();
        let parts: Vec<String> = ["watching off"]
            .into_iter()
            .map(String::from)
            .chain(SIGN_SPANS.iter().map(|s| format!("span {s}")))
            .enumerate()
            .map(|(i, label)| {
                let f = &flags[i * n..(i + 1) * n];
                format!(
                    "{label}: agree in {} of {} with a value; {}",
                    f.iter().filter(|&&x| x == 1.0).count(),
                    stats::finite(f).len(),
                    ak_row(&label, &r[i * n..(i + 1) * n], k),
                )
            })
            .collect();
        share_of(&flags, "the signs agree")
            .with(&format!(
                "Runs: watching off and watch-ak at spans 1, 3, 7 and 13, seeds 1–{SEEDS_1_2}; per run the sign of the hoarders' advantage against the sign of p_s ÷ p_o − 1 (+ when p_o = 0 with p_s > 0); a run with an undefined ratio is excluded. {}.",
                parts.join("; ")
            ))
            .with(&ak_reported(seeds, k))
            .with(ak_context())
    })
}

fn watched_claim(_seeds: &[u64]) -> Outcome {
    with_ak(|k, seeds| {
        let f = factorial(JUDGED, seeds);
        let (off, on) = (
            col(&f.off, |x| x.hoarder_adv(k)),
            col(&f.cheaters, |x| x.hoarder_adv(k)),
        );
        at_least(
            &drop_flags(&off, &on),
            QUALIFY,
            "the hoarders' advantage under who: cheaters ≥ 0.05 below watching off",
        )
        .with(&format!(
            "Per seed hoarders' advantage, watching off {}; who: cheaters {}. The factorial: {}.",
            list(&off, 3),
            list(&on, 3),
            factorial_row(&f, k)
        ))
        .with(&ak_reported(seeds, k))
        .with(ak_context())
    })
}

fn raiding_claim(_seeds: &[u64]) -> Outcome {
    with_ak(|k, seeds| {
        let f = factorial(JUDGED, seeds);
        let (off, on) = (
            col(&f.off, |x| x.fit_c(k, H)),
            col(&f.hoarders, |x| x.fit_c(k, H)),
        );
        at_least(
            &drop_flags(&off, &on),
            QUALIFY,
            "hoarder fitness under who: hoarders ≥ 0.05 below watching off",
        )
        .with(&format!(
            "Per seed hoarder fitness, watching off {}; who: hoarders {}. The factorial: {}.",
            list(&off, 3),
            list(&on, 3),
            factorial_row(&f, k)
        ))
        .with(&ak_reported(seeds, k))
        .with(ak_context())
    })
}

// ------------------------------------------ 3. producers and scroungers

/// Claim 3's worlds: variant forgo (scroungers at share `s`) or bury
/// (watchers at share `s`, no cheaters).
pub(crate) fn share_world(forgo: bool, s: f64) -> Config {
    if forgo {
        let mut c = preset("watch-scroungers-forgo");
        c.theft.cheaters = s;
        c.watching.watchers = s;
        c
    } else {
        let mut c = preset("watch-scroungers");
        c.watching.watchers = s;
        c
    }
}

fn share_runs(forgo: bool, v: Variant, seeds: &[u64]) -> Vec<Vec<Run>> {
    SHARES
        .iter()
        .map(|&s| runs(&v.apply(share_world(forgo, s)), seeds))
        .collect()
}

const K3: Fitness = Fitness::Field;

fn adv3(r: &Run) -> f64 {
    r.watcher_adv(K3)
}

fn world3(r: &Run) -> f64 {
    r.fit_all(K3)
}

/// 3a's rule on the per-seed slopes.
pub(crate) fn frequency_verdict(slopes: &[f64]) -> Outcome {
    let v = stats::finite(slopes);
    if v.len() < MIN_VALUES {
        return untestable(&format!(
            "only {} seeds gave a finite slope (need {MIN_VALUES})",
            v.len()
        ));
    }
    let (mu, lo, hi) = ci95(&v);
    let (_, lo90, hi90) = ci90(&v);
    let (verdict, label) = if hi < 0.0 {
        (Verdict::Holds, "falls")
    } else if -FLAT <= lo90 && hi90 <= FLAT {
        (Verdict::Fails, "flat")
    } else if lo > 0.0 {
        (Verdict::Fails, "rising")
    } else {
        (Verdict::Inconclusive, "inconclusive")
    };
    Outcome {
        verdict,
        measured: format!(
            "{label}: mean per-seed slope {mu:.4}, 95 % CI {lo:.4} to {hi:.4}, 90 % CI {lo90:.4} to {hi90:.4} (t, df {}); {} of {} slopes below 0",
            v.len() - 1,
            v.iter().filter(|&&x| x < 0.0).count(),
            v.len()
        ),
        detail: String::new(),
    }
}

/// 3b's rule: the advantage above 0 at the lowest share and below 0 at the
/// highest, each in ≥ 48 of 60 seeds.
pub(crate) fn mix_verdict(low: &[f64], high: &[f64]) -> Outcome {
    let above: Vec<f64> = low
        .iter()
        .map(|&x| if x.is_nan() { f64::NAN } else { flag(x > 0.0) })
        .collect();
    let below: Vec<f64> = high
        .iter()
        .map(|&x| if x.is_nan() { f64::NAN } else { flag(x < 0.0) })
        .collect();
    all_of(vec![
        (
            "above 0 at s = 0.1".into(),
            at_least(&above, QUALIFY_3, "advantage > 0"),
        ),
        (
            "below 0 at s = 0.9".into(),
            at_least(&below, QUALIFY_3, "advantage < 0"),
        ),
    ])
}

/// A 95 % t interval's verdict: Holds when it lies wholly above 0 (`above`)
/// or wholly below 0.
fn ci_side(v: &[f64], above: bool, what: &str) -> Outcome {
    let v = stats::finite(v);
    if v.len() < MIN_VALUES {
        return untestable(&format!(
            "only {} seeds gave a value (need {MIN_VALUES})",
            v.len()
        ));
    }
    let (mu, lo, hi) = ci95(&v);
    let holds = if above { lo > 0.0 } else { hi < 0.0 };
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!(
            "{what}: mean {mu:.4}, 95 % CI {lo:.4} to {hi:.4} (t, df {})",
            v.len() - 1
        ),
        detail: String::new(),
    }
}

/// 3c's rule: the pooled advantage's interval above 0 and the world
/// fitness slope's below 0.
pub(crate) fn dilemma_verdict(pooled: &[f64], world_slopes: &[f64]) -> Outcome {
    all_of(vec![
        (
            "watchers ahead".into(),
            ci_side(pooled, true, "pooled watcher advantage"),
        ),
        (
            "the world's fitness falls with s".into(),
            ci_side(world_slopes, false, "per-seed slope of world fitness on s"),
        ),
    ])
}

/// Per seed, the mean over shares (with a value) of the advantage.
pub(crate) fn pooled(worlds: &[Vec<Run>]) -> Vec<f64> {
    (0..worlds[0].len())
        .map(|i| {
            let v = stats::finite(&worlds.iter().map(|w| adv3(&w[i])).collect::<Vec<_>>());
            if v.is_empty() {
                f64::NAN
            } else {
                stats::mean(&v)
            }
        })
        .collect()
}

fn crossings(worlds: &[Vec<Run>]) -> Vec<String> {
    (0..worlds[0].len())
        .map(|i| {
            let ys: Vec<f64> = worlds.iter().map(|w| adv3(&w[i])).collect();
            down_crossing(&SHARES, &ys).map_or("none".into(), |x| format!("{x:.2}"))
        })
        .collect()
}

/// Medians by share: advantage, the groups' and the world's fitness, raids.
fn share_rows(worlds: &[Vec<Run>]) -> String {
    SHARES
        .iter()
        .zip(worlds)
        .map(|(s, r)| {
            format!(
                "s {s}: advantage {}, watchers {}, others {}, world {}, raided sugar per seed {}",
                med(&col(r, adv3)),
                med(&col(r, |x| x.fit_w(K3, W))),
                med(&col(r, |x| x.fit_w(K3, O))),
                med(&col(r, world3)),
                med(&col(r, |x| x.raided)),
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// All three judges' outcomes for a variant's worlds, as one line.
fn share_summary(forgo: bool, worlds: &[Vec<Run>]) -> String {
    let a = frequency_verdict(&seed_slopes(&SHARES, worlds, adv3));
    let mut s = format!("3a {:?} ({})", a.verdict, a.measured);
    if forgo {
        let b = mix_verdict(&col(&worlds[0], adv3), &col(&worlds[8], adv3));
        s.push_str(&format!("; 3b {:?} ({})", b.verdict, b.measured));
    } else {
        let c = dilemma_verdict(&pooled(worlds), &seed_slopes(&SHARES, worlds, world3));
        s.push_str(&format!("; 3c {:?} ({})", c.verdict, c.measured));
    }
    s
}

fn share_reported(forgo: bool, seeds: &[u64]) -> String {
    let rows: Vec<String> = reported()
        .into_iter()
        .map(|v| {
            let w = share_runs(forgo, v, seeds);
            format!(
                "{}: {}; median advantage by share {}",
                v.label(),
                share_summary(forgo, &w),
                list(&w.iter().map(|r| m(&col(r, adv3))).collect::<Vec<_>>(), 3),
            )
        })
        .collect();
    format!("Under the reported settings: {}.", rows.join("; "))
}

fn world_text(forgo: bool) -> &'static str {
    if forgo {
        "watch-scroungers-forgo's world (the winter field, find 0, loot eaten; scroungers, who watch and never bury and forgo producing while they hold a fresh entry, at share s = cheaters = watchers; the rest bury and never watch)"
    } else {
        "watch-scroungers' world (the winter field, find 0, no cheaters; every agent buries, watchers at share s)"
    }
}

fn frequency_claim_for(forgo: bool) -> Outcome {
    let seeds = seeds_upto(SEEDS_3);
    let worlds = share_runs(forgo, JUDGED, &seeds);
    let slopes = seed_slopes(&SHARES, &worlds, adv3);
    frequency_verdict(&slopes)
        .with(&format!(
            "{}, span {ANCHOR}, s = 0.1–0.9, seeds 1–{SEEDS_3}, ticks 1–200; the advantage is watcher − other fitness (ticks alive per founder ÷ 200). Per-seed slopes {}. Per-seed down-crossings: {}. By share (medians): {}.",
            world_text(forgo),
            list(&slopes, 3),
            crossings(&worlds).join(" "),
            share_rows(&worlds),
        ))
        .with(&share_reported(forgo, &seeds))
        .with("Barnard and Sibly (1981): the payoff to scroungers falls as they become common. Holds when the 95 % CI of the mean per-seed slope lies below 0; Flat (Fails) when the 90 % CI lies within ±0.05; rising (Fails) when the 95 % CI lies above 0; Inconclusive otherwise.")
}

fn frequency_forgo_claim(_seeds: &[u64]) -> Outcome {
    frequency_claim_for(true)
}

fn frequency_bury_claim(_seeds: &[u64]) -> Outcome {
    frequency_claim_for(false)
}

fn mix_claim(_seeds: &[u64]) -> Outcome {
    let seeds = seeds_upto(SEEDS_3);
    let worlds = share_runs(true, JUDGED, &seeds);
    mix_verdict(&col(&worlds[0], adv3), &col(&worlds[8], adv3))
        .with(&format!(
            "{}, span {ANCHOR}, seeds 1–{SEEDS_3}. Per seed advantage at s = 0.1 {}; at s = 0.9 {}. Per-seed crossing (first sign change from above 0 to at most 0, interpolated): {}. By share (medians): {}.",
            world_text(true),
            list(&col(&worlds[0], adv3), 3),
            list(&col(&worlds[8], adv3), 3),
            crossings(&worlds).join(" "),
            share_rows(&worlds),
        ))
        .with(&share_reported(true, &seeds))
        .with("Barnard and Sibly (1981): a stable mix, scroungers fitter when rare and less fit when common.")
}

fn dilemma_claim(_seeds: &[u64]) -> Outcome {
    let seeds = seeds_upto(SEEDS_3);
    let worlds = share_runs(false, JUDGED, &seeds);
    let p = pooled(&worlds);
    let ws = seed_slopes(&SHARES, &worlds, world3);
    dilemma_verdict(&p, &ws)
        .with(&format!(
            "{}, span {ANCHOR}, seeds 1–{SEEDS_3}. Per seed pooled advantage {}; per-seed world fitness slopes {}. By share (medians): {}.",
            world_text(false),
            list(&p, 3),
            list(&ws, 3),
            share_rows(&worlds),
        ))
        .with(&share_reported(false, &seeds))
        .with("Watchers who also bury are Vickery et al.'s cost-free opportunists, not Barnard and Sibly's scroungers. Partly seen before the run: under raid_if always the design audit saw world survival fall from 0.714 to 0.551; under better its variant cut the watchers' lead to 1 point.")
}

// ------------------------------------------------------------------ usage

/// The presets with watching on.
pub(crate) fn watching_presets() -> Vec<String> {
    presets::all()
        .into_iter()
        .filter(|p| p.config.watching.on)
        .map(|p| p.id.to_string())
        .collect()
}

/// `survey --usage`: in each watching preset, raided sugar above 0 in ≥ 16
/// of 20 seeds. Writes `survey/out/minds8b-usage.md` and returns its text.
pub(crate) fn usage_report() -> String {
    let seeds = seeds_upto(SEEDS_1_2);
    let mut s = String::new();
    writeln!(s, "# Minds 8b: usage\n").unwrap();
    writeln!(
        s,
        "Generated by `cargo run --release -- --usage` in `survey/` (`survey/src/claims/minds8b.rs`). \
         A check, not a claim ({SPEC}, \"Usage\"): in every preset with watching on, raids take \
         sugar (Σ `raided` over ticks 1–200 above 0) in at least {QUALIFY} of {SEEDS_1_2} seeds.\n"
    )
    .unwrap();
    writeln!(
        s,
        "| preset | seeds with raided sugar | passes | raids per seed | raided sugar per seed |\n|---|---|---|---|---|"
    )
    .unwrap();
    let mut all = true;
    for id in watching_presets() {
        let r = runs(&preset(&id), &seeds);
        let n = r.iter().filter(|x| x.raided > 0.0).count();
        all &= n >= QUALIFY;
        writeln!(
            s,
            "| `{id}` | {n} of {} | {} | {} | {} |",
            r.len(),
            if n >= QUALIFY { "yes" } else { "no" },
            med(&col(&r, |x| x.raids)),
            med(&col(&r, |x| x.raided)),
        )
        .unwrap();
    }
    writeln!(
        s,
        "\n**Result:** {}.",
        if all {
            "raids take sugar in every watching preset"
        } else {
            "some watching preset fails the check"
        }
    )
    .unwrap();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/out");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(format!("{dir}/minds8b-usage.md"), &s).unwrap();
    s
}

// ---------------------------------------------------------------- presets

/// Survival per founder at 200 of a group (`alive200` ÷ founders).
fn surv(alive: [f64; 2], founders: [f64; 2], g: Option<usize>) -> f64 {
    match g {
        Some(g) => nan_div(alive[g], founders[g]),
        None => nan_div(alive[0] + alive[1], founders[0] + founders[1]),
    }
}

/// Reported only (not a claim): a preset's world and the comparisons that
/// isolate a cause, each a (label, config, setup).
fn preset_worlds(id: &str) -> Vec<(String, Config, Setup)> {
    let base = preset(id);
    let mut out = vec![("as set".to_string(), base.clone(), Setup::default())];
    let mut off = base.clone();
    off.watching.on = false;
    out.push(("watching off".into(), off, Setup::default()));
    let mut always = base.clone();
    always.watching.raid_if = RaidIf::Always;
    out.push(("raid_if: always".into(), always, Setup::default()));
    let mut room = base.clone();
    room.watching.value = SeenValue::Room;
    out.push(("value: room".into(), room, Setup::default()));
    out.push((
        "the probe (a raid also harvests its site)".into(),
        base.clone(),
        Setup {
            fates: false,
            probe: true,
        },
    ));
    if base.theft.cheaters > 0.0 && base.watching.watchers > 0.0 && base.watching.who == Who::Share
    {
        let mut sc = base.clone();
        let (to, label) = match base.watching.scrounge {
            Scrounge::Forgo => (Scrounge::Harvest, "scrounge: harvest"),
            Scrounge::Harvest => (Scrounge::Forgo, "scrounge: forgo"),
        };
        sc.watching.scrounge = to;
        out.push((label.into(), sc, Setup::default()));
    }
    out
}

/// Formats a median, or "—" when no seed has a value.
fn md(v: &[f64]) -> String {
    let f = stats::finite(v);
    if f.is_empty() {
        "—".into()
    } else {
        format!("{:.3}", m(&f))
    }
}

/// The lowest finite value, or "—".
fn lowest(v: &[f64]) -> String {
    let f = stats::finite(v);
    if f.is_empty() {
        "—".into()
    } else {
        format!("{:.3}", f.iter().copied().fold(f64::INFINITY, f64::min))
    }
}

/// The mean paired difference `b − a` with its 95 % t interval, and the
/// seeds where it is above 0, or "—".
fn paired(a: &[f64], b: &[f64]) -> String {
    let d: Vec<f64> = a.iter().zip(b).map(|(x, y)| y - x).collect();
    let f = stats::finite(&d);
    if f.len() < MIN_VALUES {
        return "—".into();
    }
    let (mu, lo, hi) = ci95(&f);
    format!(
        "{mu:+.3} ({lo:+.3} to {hi:+.3}; above 0 in {} of {})",
        f.iter().filter(|&&x| x > 0.0).count(),
        f.len()
    )
}

/// `survey --presets`: every watching preset measured the same way (seeds
/// 1–20, ticks 1–200), reported and not judged, for the presets'
/// descriptions. Writes `survey/out/minds8b-presets.md`.
pub(crate) fn presets_report() -> String {
    let seeds = seeds_upto(SEEDS_1_2);
    let mut s = String::new();
    writeln!(s, "# Minds 8b: the watching presets, measured\n").unwrap();
    writeln!(
        s,
        "Generated by `cargo run --release -- --presets` in `survey/` (`survey/src/claims/minds8b.rs`). \
         Reported, not judged: the measures the presets' descriptions are written from. Seeds 1–{SEEDS_1_2}, \
         ticks 1–200, each preset as set (`raid_if: better`, `value: amount` unless it says otherwise) \
         and beside it the same world with one thing changed. Fitness is the world's kind ({SPEC}): \
         agent-ticks alive per founder over ticks 1–200 ÷ 200 in the field, wealth per founder at 200 \
         in the arena. Survival is per founder (alive at 200 ÷ founders) unless marked \"of those alive at 100\". \
         The pilferage rate is the first round's stock rate (Σ caches pilfered ÷ Σ caches present at a tick's start). \
         Stumbled sugar is pilfered − raided. Medians over seeds; paired differences are the mean of \
         (changed − as set) with its 95 % t interval.\n"
    )
    .unwrap();
    for id in watching_presets() {
        let k = if id.contains("arena") {
            Fitness::Arena
        } else {
            Fitness::Field
        };
        let worlds: Vec<(String, Vec<Run>)> = preset_worlds(&id)
            .into_iter()
            .map(|(l, c, st)| (l, runs_of(&c, st, &seeds)))
            .collect();
        writeln!(
            s,
            "## `{id}` ({} fitness)\n",
            match k {
                Fitness::Field => "field: ticks alive per founder ÷ 200",
                Fitness::Arena => "arena: wealth per founder at 200",
            }
        )
        .unwrap();
        let r0 = &worlds[0].1;
        writeln!(
            s,
            "Founders per seed: hoarders {}, cheaters {}, watchers {}, others {}.\n",
            md(&col(r0, |x| x.founders_c[H])),
            md(&col(r0, |x| x.founders_c[C])),
            md(&col(r0, |x| x.founders_w[W])),
            md(&col(r0, |x| x.founders_w[O])),
        )
        .unwrap();
        writeln!(s, "| world | fitness: all | hoarders | cheaters | watchers | others | survival: all | hoarders | cheaters | watchers | others | all, of those alive at 100 | survival: all, lowest seed |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|").unwrap();
        for (l, r) in &worlds {
            writeln!(
                s,
                "| {l} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                md(&col(r, |x| x.fit_all(k))),
                md(&col(r, |x| x.fit_c(k, H))),
                md(&col(r, |x| x.fit_c(k, C))),
                md(&col(r, |x| x.fit_w(k, W))),
                md(&col(r, |x| x.fit_w(k, O))),
                md(&col(r, |x| surv(x.alive200_c, x.founders_c, None))),
                md(&col(r, |x| surv(x.alive200_c, x.founders_c, Some(H)))),
                md(&col(r, |x| surv(x.alive200_c, x.founders_c, Some(C)))),
                md(&col(r, |x| surv(x.alive200_w, x.founders_w, Some(W)))),
                md(&col(r, |x| surv(x.alive200_w, x.founders_w, Some(O)))),
                md(&col(r, |x| surv(x.alive200_c, x.alive100_c, None))),
                lowest(&col(r, |x| surv(x.alive200_c, x.founders_c, None))),
            )
            .unwrap();
        }
        writeln!(s, "\n| world | pilferage % a tick | raids | raided sugar | stumbled sugar | dug | p_s | p_o | seeds with p_s > p_o |\n|---|---|---|---|---|---|---|---|---|").unwrap();
        for (l, r) in &worlds {
            writeln!(
                s,
                "| {l} | {} | {} | {} | {} | {} | {} | {} | {} of {} |",
                md(&col(r, |x| 100.0 * x.stock_rate())),
                md(&col(r, |x| x.raids)),
                md(&col(r, |x| x.raided)),
                md(&col(r, |x| x.pilfered - x.raided)),
                md(&col(r, |x| x.dug)),
                md(&col(r, Run::p_s)),
                md(&col(r, Run::p_o)),
                r.iter().filter(|x| x.p_s() > x.p_o()).count(),
                r.len(),
            )
            .unwrap();
        }
        writeln!(s, "\nPaired against as set (changed − as set, mean, 95 % CI, seeds above 0):\n\n| world | fitness: all | hoarders | cheaters | watchers | others | hoarders' advantage | watchers' advantage | survival: all |\n|---|---|---|---|---|---|---|---|---|").unwrap();
        for (l, r) in worlds.iter().skip(1) {
            let p = |f: &dyn Fn(&Run) -> f64| paired(&col(r0, f), &col(r, f));
            writeln!(
                s,
                "| {l} | {} | {} | {} | {} | {} | {} | {} | {} |",
                p(&|x| x.fit_all(k)),
                p(&|x| x.fit_c(k, H)),
                p(&|x| x.fit_c(k, C)),
                p(&|x| x.fit_w(k, W)),
                p(&|x| x.fit_w(k, O)),
                p(&|x| x.hoarder_adv(k)),
                p(&|x| x.watcher_adv(k)),
                p(&|x| surv(x.alive200_c, x.founders_c, None)),
            )
            .unwrap();
        }
        writeln!(s).unwrap();
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/out");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(format!("{dir}/minds8b-presets.md"), &s).unwrap();
    s
}

pub fn claims() -> Vec<Claim> {
    vec![Claim {
        id: "watch-ak.calibration",
        item: "watch-ak",
        source: Source::Comment,
        citation: SPEC,
        text: "Claim 2's precondition: some world on the calibration list (theft-arena-2; theft-winter-half at find 0.02, 0.05; the same with owners digging below their whole reserve) has p_s > p_o in at least 16 of 20 seeds without watching; the first such world is watch-ak's",
        check: calibration_claim,
    },
    Claim {
        id: "watch-winter.fresh",
        item: "watch-winter",
        source: Source::Comment,
        citation: "the spec, claim 1a; Heinrich & Pepper 1998; Vander Wall & Jenkins 2003, pp. 656, 658, 663 (context)",
        text: "Watching takes fresh caches faster than stumbling: the cohort hazard h = 1 − (1 − P_7)^(1/7) of caches first created in ticks 1–90 is higher in watch-winter than in theft-winter, paired, in at least 16 of 20 seeds",
        check: fresh_claim,
    },
    Claim {
        id: "watch-ak.flip",
        item: "watch-ak",
        source: Source::Book,
        citation: "Andersson & Krebs 1978, p.708; the spec, claim 2a",
        text: "Watching flips Andersson and Krebs's condition: in watch-ak, p_s ÷ p_o < 1 in at least 16 of 20 seeds",
        check: flip_claim,
    },
    Claim {
        id: "watch-ak.sign",
        item: "watch-ak",
        source: Source::Book,
        citation: "Andersson & Krebs 1978, p.708; the spec, claim 2b",
        text: "Fitness follows the condition: over watching off and spans 1, 3, 7 and 13 (seeds 1–20), the sign of the hoarders' advantage agrees with the sign of p_s ÷ p_o − 1 in at least 80 % of runs (runs with an undefined ratio excluded)",
        check: sign_claim,
    },
    Claim {
        id: "watch-ak.watched",
        item: "watch-ak",
        source: Source::Comment,
        citation: "the spec, claim 2c",
        text: "Being watched costs hoarders: the hoarders' advantage under who: cheaters is below its value with watching off by at least 0.05 fitness units, paired, in at least 16 of 20 seeds",
        check: watched_claim,
    },
    Claim {
        id: "watch-ak.raiding",
        item: "watch-ak",
        source: Source::Comment,
        citation: "the spec, claim 2d",
        text: "Their own raiding costs hoarders: hoarder fitness under who: hoarders is below its value with watching off by at least 0.05, paired, in at least 16 of 20 seeds",
        check: raiding_claim,
    },
    Claim {
        id: "watch-scroungers.frequency-forgo",
        item: "watch-scroungers-forgo",
        source: Source::Book,
        citation: "Barnard & Sibly 1981; the spec, claim 3a",
        text: "Scrounger advantage falls with share (scroungers who forgo producing): the 95 % CI of the mean per-seed slope of the advantage on s (0.1–0.9, seeds 1–60) lies wholly below 0; fails as flat when the 90 % CI lies within ±0.05, and as rising when the 95 % CI lies wholly above 0; inconclusive otherwise",
        check: frequency_forgo_claim,
    },
    Claim {
        id: "watch-scroungers.frequency-bury",
        item: "watch-scroungers",
        source: Source::Book,
        citation: "Barnard & Sibly 1981; the spec, claim 3a",
        text: "Watcher advantage falls with share (watchers who also bury): the 95 % CI of the mean per-seed slope of the advantage on s (0.1–0.9, seeds 1–60) lies wholly below 0; fails as flat when the 90 % CI lies within ±0.05, and as rising when the 95 % CI lies wholly above 0; inconclusive otherwise",
        check: frequency_bury_claim,
    },
    Claim {
        id: "watch-scroungers.mix",
        item: "watch-scroungers-forgo",
        source: Source::Book,
        citation: "Barnard & Sibly 1981; the spec, claim 3b",
        text: "A stable mix: the scroungers' advantage is above 0 at s = 0.1 and below 0 at s = 0.9, each in at least 48 of 60 seeds",
        check: mix_claim,
    },
    Claim {
        id: "watch-scroungers.dilemma",
        item: "watch-scroungers",
        source: Source::Comment,
        citation: "Vickery et al. 1991; the spec, claim 3c",
        text: "A social dilemma: the watchers' advantage pooled over shares has a 95 % CI above 0, and the world's fitness falls with s (the per-seed slope's 95 % CI lies below 0)",
        check: dilemma_claim,
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
            ..Run::default()
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
    fn a_founder_counts_each_tick_it_is_alive_and_nothing_after_it_starves() {
        // The two-agent arena with no sugar: both founders starve within a
        // few ticks. The last tick each is alive after a step is found by
        // stepping the same world independently.
        let mut c = preset("theft-arena-2");
        c.goods[0].map = sugarscape_core::config::Map::Flat { capacity: 0.0 };
        c.goods[0].endowment = sugarscape_core::config::URange::new(3, 8);
        let mut w = World::new(c.clone(), 1).unwrap();
        let ids: Vec<(u64, usize)> = w.agents().map(|a| (a.id, usize::from(a.cheater))).collect();
        let mut last = [0u64; 2];
        while w.tick < TICKS {
            w.step();
            for &(id, g) in &ids {
                if w.agent(id).is_some() {
                    last[g] = w.tick;
                }
            }
        }
        assert!(
            last[H] > 0 && last[H] < 20,
            "the hoarder starves early: {last:?}"
        );
        assert!(
            last[C] > 0 && last[C] < 20,
            "the cheater starves early: {last:?}"
        );
        let r = &runs(&c, &[1])[0];
        assert_eq!(r.founders_c, [1.0, 1.0]);
        assert_eq!(r.ticks_alive_c, [last[H] as f64, last[C] as f64]);
        assert_eq!(r.fit_c(Fitness::Field, H), last[H] as f64 / 200.0);
        assert_eq!(r.wealth200_c, [0.0, 0.0], "the dead have no wealth at 200");
        assert_eq!(r.fit_c(Fitness::Arena, C), 0.0);
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

    // ------------------------------------------------------ the judges

    fn rec(owner: AgentId, site: u32, buried: u64, fate: Option<(u64, Fate)>) -> CacheRecord {
        CacheRecord {
            owner,
            site,
            amount: 1.0,
            buried,
            fate,
        }
    }

    fn pilfered(t: u64, by: AgentId) -> Option<(u64, Fate)> {
        Some((t, Fate::Pilfered { by }))
    }

    #[test]
    fn a_cache_runs_from_its_creation_until_every_record_has_ended() {
        let log = vec![
            rec(1, 5, 3, Some((5, Fate::Dug))),
            // Buried on the tick the first ended: the same cache.
            rec(1, 5, 5, Some((6, Fate::Dug))),
            // Buried after both ended: a new cache.
            rec(1, 5, 8, None),
            // Still buried, so everything after joins it.
            rec(1, 5, 20, Some((30, Fate::Dug))),
            // Another site, another owner: their own caches.
            rec(1, 6, 3, None),
            rec(2, 5, 3, None),
            // A split part, logged later, buried with the first.
            rec(1, 5, 3, pilfered(4, 9)),
        ];
        let mut got: Vec<(AgentId, u32, u64, usize)> = caches(&log)
            .iter()
            .map(|c| {
                (
                    c.records[0].owner,
                    c.records[0].site,
                    c.created,
                    c.records.len(),
                )
            })
            .collect();
        got.sort();
        assert_eq!(
            got,
            vec![(1, 5, 3, 3), (1, 5, 8, 2), (1, 6, 3, 1), (2, 5, 3, 1)]
        );
    }

    #[test]
    fn the_cohort_counts_takes_by_age_and_ends_at_a_dig_or_a_loss() {
        let log = vec![
            // Taken on the tick it was created (age 0): in every P_d.
            rec(1, 1, 0, pilfered(0, 7)),
            // Taken at age 2: in P_3 and P_7.
            rec(1, 2, 10, pilfered(12, 7)),
            // Taken at age 7: in P_7 only.
            rec(1, 3, 20, pilfered(27, 7)),
            // Taken at age 8: in none.
            rec(1, 4, 30, pilfered(38, 7)),
            // Dug up at age 1, then a new cache there taken at age 0 of the
            // new one: the first is not taken, the second is created after
            // the cohort's window.
            rec(1, 5, 40, Some((41, Fate::Dug))),
            rec(1, 5, 95, pilfered(95, 7)),
            // Lost with its owner before any take.
            rec(2, 6, 50, Some((51, Fate::Lost))),
            // Part dug, the rest taken at age 3: taken (a part dug doesn't
            // end the cache).
            rec(3, 7, 60, Some((61, Fate::Dug))),
            rec(3, 7, 60, pilfered(63, 8)),
            // Created at log tick 89 (the world's tick 90): the last in.
            rec(4, 8, 89, None),
            // Created at log tick 90: out.
            rec(4, 9, 90, pilfered(90, 7)),
        ];
        let f = from_log(&log, &HashSet::new(), &HashSet::new());
        assert_eq!(f.cohort, 8.0);
        assert_eq!(f.taken, [1.0, 3.0, 4.0]);
        let r = Run {
            fresh: f,
            ..Run::default()
        };
        assert_eq!(r.p_d(2), 0.5);
        assert!((r.hazard() - (1.0 - 0.5f64.powf(1.0 / 7.0))).abs() < 1e-15);
        assert_eq!(f.burials, 10.0, "the split part shares its burial");
        assert_eq!(f.caches_seen, 0.0);
    }

    #[test]
    fn the_hazard_is_the_daily_rate_that_gives_p_7() {
        assert_eq!(hazard(0.0), 0.0);
        assert_eq!(hazard(1.0), 1.0);
        assert!((hazard(1.0 - 0.9f64.powi(7)) - 0.1).abs() < 1e-12);
        assert!(hazard(f64::NAN).is_nan());
        let empty = Run {
            fresh: Fresh {
                cohort: 0.0,
                taken: [0.0; 3],
                ..Fresh::default()
            },
            ..Run::default()
        };
        assert!(empty.hazard().is_nan(), "an empty cohort has no hazard");
        assert!(Run::default().hazard().is_nan(), "no log, no hazard");
    }

    #[test]
    fn caches_seen_and_raided_follow_the_sightings_and_the_raids() {
        let log = vec![
            // Seen at its second burial, raided by 7.
            rec(1, 1, 0, pilfered(5, 7)),
            rec(1, 1, 2, pilfered(5, 7)),
            // Seen, then taken by a stumble (8 isn't a raider here).
            rec(1, 2, 3, pilfered(4, 8)),
            // Seen, never taken.
            rec(1, 3, 3, None),
            // Not seen, raided (a raid on an old memory): not counted.
            rec(2, 4, 3, pilfered(6, 7)),
        ];
        let sighted: HashSet<SightKey> = [(1, 1, 2), (1, 2, 3), (1, 3, 3), (1, 9, 3)].into();
        let raids: HashSet<RaidKey> = [(7, 1, 1, 5), (7, 4, 2, 6), (7, 2, 1, 4)].into();
        let f = from_log(&log, &sighted, &raids);
        assert_eq!((f.caches_seen, f.seen_raided), (3.0, 1.0));
        assert_eq!(f.raid_takes, 2.0, "one take per raid, whatever its records");
        let r = Run {
            burials_seen: 3.0,
            fresh: f,
            ..Run::default()
        };
        assert_eq!(f.burials, 5.0);
        assert_eq!(r.p_seen(), 0.6);
        assert_eq!(r.p_raid_seen(), 1.0 / 3.0);
        assert_eq!(bottleneck(r.p_seen(), r.p_raid_seen()), "action-bound");
    }

    #[test]
    fn the_bottleneck_rule() {
        assert_eq!(bottleneck(0.2, 0.3), "knowledge-bound");
        assert_eq!(
            bottleneck(0.3, 0.3),
            "action-bound",
            "a tie is action-bound"
        );
        assert_eq!(bottleneck(0.88, 0.33), "action-bound");
        assert_eq!(bottleneck(f64::NAN, 0.3), "unclassified");
    }

    #[test]
    fn a_count_judge_needs_its_number_of_all_the_seeds() {
        let mut v = vec![1.0; 16];
        v.extend([0.0; 4]);
        assert_eq!(at_least(&v, 16, "x").verdict, Verdict::Holds);
        v[0] = 0.0;
        assert_eq!(at_least(&v, 16, "x").verdict, Verdict::Fails);
        // A seed with no value is not a hold, but the rest are still judged.
        let mut w = vec![1.0; 15];
        w.extend([f64::NAN; 5]);
        let o = at_least(&w, 16, "x");
        assert_eq!(o.verdict, Verdict::Fails);
        assert!(o.measured.contains("15 of 20"), "{}", o.measured);
        assert!(o.measured.contains("5 without a value"), "{}", o.measured);
        let few = [1.0, 1.0, 1.0, 1.0, f64::NAN, f64::NAN];
        assert_eq!(at_least(&few, 4, "x").verdict, Verdict::Untestable);
        let mut big = vec![1.0; 48];
        big.extend([0.0; 12]);
        assert_eq!(at_least(&big, QUALIFY_3, "x").verdict, Verdict::Holds);
        big[0] = 0.0;
        assert_eq!(at_least(&big, QUALIFY_3, "x").verdict, Verdict::Fails);
    }

    #[test]
    fn a_share_judge_excludes_runs_without_a_value() {
        let mut v = vec![1.0; 8];
        v.extend([0.0, 0.0, f64::NAN, f64::NAN]);
        assert_eq!(share_of(&v, "x").verdict, Verdict::Holds, "8 of 10 is 80 %");
        v[0] = 0.0;
        assert_eq!(share_of(&v, "x").verdict, Verdict::Fails);
        let few = [1.0, 1.0, 1.0, 1.0, f64::NAN];
        assert_eq!(share_of(&few, "x").verdict, Verdict::Untestable);
    }

    #[test]
    fn claim_1a_needs_a_strictly_higher_hazard() {
        let f = fresh_flags(
            &[0.2, 0.1, 0.1, f64::NAN, 0.3],
            &[0.1, 0.1, 0.2, 0.1, f64::NAN],
        );
        assert_eq!(f[..3], [1.0, 0.0, 0.0]);
        assert!(f[3].is_nan() && f[4].is_nan());
    }

    #[test]
    fn claim_2a_flips_only_below_1() {
        let f = flip_flags(&[0.5, 0.999, 1.0, 2.0, f64::INFINITY, f64::NAN]);
        assert_eq!(f, vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
        // The ratio itself is Minds 8's.
        let r = |dug: f64, pilfered: f64| Run {
            dug,
            pilfered,
            ..Run::default()
        };
        assert_eq!(r(1.0, 3.0).ratio(), 0.25 / 0.75);
        assert_eq!(r(1.0, 0.0).ratio(), f64::INFINITY);
        assert!(r(0.0, 0.0).ratio().is_nan());
    }

    #[test]
    fn claim_2b_compares_three_valued_signs() {
        let adv = [0.1, -0.1, 0.1, 0.0, 0.0, 0.1, f64::NAN, 0.2];
        let ratio = [2.0, 0.5, 0.5, 1.0, 2.0, f64::INFINITY, 2.0, f64::NAN];
        let f = sign_flags(&adv, &ratio);
        assert_eq!(f[..6], [1.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
        assert!(f[6].is_nan() && f[7].is_nan(), "undefined: excluded");
    }

    #[test]
    fn claims_2c_and_2d_need_a_drop_of_at_least_005() {
        let f = drop_flags(
            &[0.5, 0.5, 0.5, 0.5, f64::NAN],
            &[0.4, 0.46, 0.6, 0.45 - 1e-9, 0.1],
        );
        assert_eq!(f[..4], [1.0, 0.0, 0.0, 1.0]);
        assert!(f[4].is_nan());
        assert_eq!(
            drop_flags(&[1.0], &[0.95]),
            vec![1.0],
            "0.05 itself (to rounding)"
        );
    }

    #[test]
    fn claim_3a_holds_falls_flat_or_is_inconclusive() {
        let v = |o: Outcome| (o.verdict, o.measured.split(':').next().unwrap().to_string());
        let down = [-0.2, -0.25, -0.15, -0.22, -0.18];
        assert_eq!(
            v(frequency_verdict(&down)),
            (Verdict::Holds, "falls".into())
        );
        let flat = [0.01, -0.01, 0.005, -0.005, 0.0, 0.002];
        assert_eq!(v(frequency_verdict(&flat)), (Verdict::Fails, "flat".into()));
        // All equal: the intervals are points.
        assert_eq!(
            v(frequency_verdict(&[0.0; 6])),
            (Verdict::Fails, "flat".into())
        );
        assert_eq!(
            v(frequency_verdict(&[-0.1; 6])),
            (Verdict::Holds, "falls".into())
        );
        assert_eq!(
            v(frequency_verdict(&[0.1; 6])),
            (Verdict::Fails, "rising".into())
        );
        // Flat and above 0 at once (inside (0, 0.05]): flat comes first.
        let low = [0.02, 0.021, 0.019, 0.02, 0.02];
        assert_eq!(v(frequency_verdict(&low)), (Verdict::Fails, "flat".into()));
        // Rising, but noisy enough that the 95 % CI reaches 0: neither.
        let noisy_up = [0.3, -0.05, 0.4, 0.0, 0.35, -0.02];
        assert_eq!(
            v(frequency_verdict(&noisy_up)),
            (Verdict::Inconclusive, "inconclusive".into())
        );
        // A tight small fall inside the flat band: Holds comes first.
        let small = [-0.01, -0.012, -0.011, -0.009, -0.01];
        assert_eq!(
            v(frequency_verdict(&small)),
            (Verdict::Holds, "falls".into())
        );
        // Wide around 0: neither.
        let wide = [0.3, -0.3, 0.2, -0.25, 0.1, -0.1];
        assert_eq!(
            v(frequency_verdict(&wide)),
            (Verdict::Inconclusive, "inconclusive".into())
        );
        let mut nan = vec![f64::NAN; 10];
        nan.extend([-1.0; 4]);
        assert_eq!(frequency_verdict(&nan).verdict, Verdict::Untestable);
    }

    #[test]
    fn claim_3a_uses_t_intervals() {
        // n = 5, mean 0, sd 1: t(0.975, 4) = 2.776, t(0.95, 4) = 2.132.
        let x = [-1.0, -0.5, 0.0, 0.5, 1.0];
        let sd = (2.5f64 / 4.0).sqrt();
        let (_, lo95, hi95) = ci95(&x);
        let (_, lo90, hi90) = ci90(&x);
        assert!((hi95 - 2.776 * sd / 5f64.sqrt()).abs() < 1e-3);
        assert!((hi90 - 2.132 * sd / 5f64.sqrt()).abs() < 1e-3);
        assert_eq!((lo95, lo90), (-hi95, -hi90));
    }

    #[test]
    fn claim_3b_needs_both_ends_in_48_of_60() {
        let ends = |above: usize, below: usize| {
            let low: Vec<f64> = (0..60).map(|i| if i < above { 0.1 } else { 0.0 }).collect();
            let high: Vec<f64> = (0..60)
                .map(|i| if i < below { -0.1 } else { 0.0 })
                .collect();
            mix_verdict(&low, &high).verdict
        };
        assert_eq!(ends(48, 48), Verdict::Holds);
        assert_eq!(
            ends(47, 60),
            Verdict::Fails,
            "an advantage of 0 isn't above 0"
        );
        assert_eq!(ends(60, 47), Verdict::Fails);
        let nan = vec![f64::NAN; 60];
        assert_eq!(mix_verdict(&nan, &nan).verdict, Verdict::Untestable);
    }

    #[test]
    fn claim_3c_needs_both_parts() {
        let ahead = [0.1, 0.12, 0.08, 0.11, 0.09];
        let falls = [-0.2, -0.22, -0.18, -0.21, -0.19];
        let wide = [0.3, -0.3, 0.2, -0.25, 0.1];
        assert_eq!(dilemma_verdict(&ahead, &falls).verdict, Verdict::Holds);
        assert_eq!(dilemma_verdict(&wide, &falls).verdict, Verdict::Fails);
        assert_eq!(dilemma_verdict(&ahead, &wide).verdict, Verdict::Fails);
        assert_eq!(
            dilemma_verdict(&ahead, &[-0.1; 3]).verdict,
            Verdict::Untestable
        );
        assert_eq!(
            dilemma_verdict(&ahead, &[-0.1; 5]).verdict,
            Verdict::Holds,
            "all equal"
        );
    }

    #[test]
    fn the_pooled_advantage_is_the_mean_over_shares_with_a_value() {
        let at = |w: f64, o: f64| Run {
            founders_w: [1.0, 1.0],
            ticks_alive_w: [w, o],
            ..Run::default()
        };
        let worlds = vec![
            vec![at(200.0, 100.0), at(100.0, 100.0)],
            vec![at(100.0, 200.0), at(100.0, 100.0)],
            vec![at(200.0, 0.0), Run::default()],
        ];
        let p = pooled(&worlds);
        assert!((p[0] - (0.5 - 0.5 + 1.0) / 3.0).abs() < 1e-12);
        assert_eq!(p[1], 0.0, "the share with no founders is left out");
        assert!(pooled(&[vec![Run::default()]])[0].is_nan());
    }

    #[test]
    fn the_judged_setting_is_the_presets_and_the_reported_ones_are_the_specs() {
        for id in watching_presets() {
            assert_eq!(JUDGED.apply(preset(&id)), preset(&id), "{id}");
        }
        for id in ["theft-winter", "theft-winter-half"] {
            for v in reported() {
                assert_eq!(v.apply(preset(id)), preset(id), "no watching: unchanged");
            }
        }
        let labels: Vec<String> = reported().iter().map(Variant::label).collect();
        assert_eq!(
            labels,
            [
                "span 7, raid_if always",
                "span 7, value room",
                "span 1",
                "span 2",
                "span 3",
                "span 7 (judged)",
                "span 13",
            ]
        );
        for v in reported() {
            v.apply(preset("watch-ak")).validate().expect("valid");
        }
    }

    #[test]
    fn the_watching_presets_are_the_eight() {
        let mut got = watching_presets();
        got.sort();
        assert_eq!(
            got,
            [
                "watch-ak",
                "watch-arena",
                "watch-half",
                "watch-scroungers",
                "watch-scroungers-forgo",
                "watch-scroungers-only",
                "watch-winter",
                "watch-winter-stumble",
            ]
        );
    }

    #[test]
    fn claim_2s_worlds() {
        assert_eq!(ak_fitness(), Some(Fitness::Field));
        assert_eq!(ak_off(), CALIBRATION[3].config(), "watching off is item 4");
        let ak = preset("watch-ak");
        assert_eq!((ak.watching.who, ak.watching.watchers), (Who::Share, 1.0));
        for who in [Who::Hoarders, Who::Cheaters] {
            let c = ak_who(who);
            c.validate().expect("valid");
            let w = World::new(c, 1).unwrap();
            assert!(w.agents().any(|a| a.cheater) && w.agents().any(|a| !a.cheater));
            assert!(w
                .agents()
                .all(|a| a.watches == (a.cheater == (who == Who::Cheaters))));
        }
        let mut back = ak_who(Who::Cheaters);
        back.watching.who = Who::Share;
        assert_eq!(back, ak);
    }

    #[test]
    fn claim_3s_worlds() {
        assert_eq!(share_world(true, 0.5), preset("watch-scroungers-forgo"));
        assert_eq!(share_world(false, 0.5), preset("watch-scroungers"));
        for s in SHARES {
            let c = share_world(true, s);
            c.validate().expect("valid");
            let w = World::new(c, 1).unwrap();
            assert!(
                w.agents().all(|a| a.watches == a.cheater),
                "scroungers at {s}"
            );
            let c = share_world(false, s);
            c.validate().expect("valid");
            assert_eq!(c.theft.cheaters, 0.0);
            assert!(World::new(c, 1).unwrap().agents().any(|a| a.watches));
        }
    }

    #[test]
    fn every_take_classified_as_a_raid_is_one_and_every_sighting_is_read() {
        // One seed of the four-agent arena, which raids and stumbles
        // (find 0.25), with the fate log.
        let r = &runs_of(
            &preset("watch-arena"),
            Setup {
                fates: true,
                probe: false,
            },
            &[1],
        )[0];
        assert!(!r.log_full);
        assert!(r.raids > 0.0, "the arena raids");
        assert_eq!(r.fresh.raid_takes, r.raids, "raids found in the log");
        assert_eq!(r.sighted, r.burials_seen, "sightings read");
        assert!(r.fresh.seen_raided <= r.fresh.caches_seen);
        assert!(r.fresh.cohort > 0.0);
        assert!(r.fresh.taken[0] <= r.fresh.taken[1] && r.fresh.taken[1] <= r.fresh.taken[2]);
        // The log changes no world: fitness and events as without it.
        let plain = &runs(&preset("watch-arena"), &[1])[0];
        assert_eq!(plain.ticks_alive_c, r.ticks_alive_c);
        assert_eq!(plain.wealth200_c, r.wealth200_c);
        assert_eq!(
            (plain.dug, plain.pilfered, plain.raided),
            (r.dug, r.pilfered, r.raided)
        );
    }
}
