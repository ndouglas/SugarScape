# The Timing of Retirement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Axtell and Epstein's retirement model as one model kind, `retirement` ("The Timing of Retirement"): 81 cohorts with deaths and newborns, rational, random and imitating agents in transient social networks, the policy switch from 65 to 62 with mandatory retirement, and two coupled sub-populations, with every unstated rule a switch; seven titled presets, eight measured sweeps and a 16-claim survey, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/retirement/` — `config.rs` (parameters, validation, schema), `stats.rs`, `view.rs`, `world.rs` (`RetirementWorld`: cohorts as birth periods, activation-time aging, deaths and newborns in the same slot, networks with a reverse index, both renewals, the norm and the policy switch, groups, rendering, Inspect), `presets.rs`, `mod.rs` — wired into `ModelConfig`/`ModelWorld` like the other models, with titles in `titles.rs`. The page adds the model's types, color modes, charts, Inspect rows, a Compare entry and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-27-retirement-design.md` (binding, as amended in Task 5). Sources: `papers/retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` and `papers/demographic-pd/epstein-2006-generative-social-science.pdf` (ch. 7).

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited (`MODEL_GOLDEN` gains seven `ae-*` entries).
- **One engine path; deterministic; portable:** native and WASM fingerprints identical (verified in planning by `wasm-pack test`). Draws use `u32` ranges and `f64` samples only; no `ln`/`exp`/trigonometry anywhere in the module.
- **Exact imitation:** retired × 1 000 000 ≥ τ × 1 000 000 × counted, in integers; no members counted, no retirement.
- **Literal defaults, named departures, honest descriptions and titles:** the defaults are the texts' (eligible members counted, the pseudo-code's slot renewal and activation-time aging, cohorts oldest first); footnote 5's alternative, friends replaced, one shuffled order and survivors' death ages are switches; every description and title says what was measured.
- **Copy (verbatim):** model label **The Timing of Retirement**; preset ids `ae-rapid`, `ae-base`, `ae-slow`, `ae-policy`, `ae-groups`, `ae-all-members`, `ae-replace`; Compare entry **15 % vs 5 % rational — Retirement (Compare)** (id `ae-rapid-vs-slow`); color modes **Status**, **Type**, **Threshold**, **Group**; schema groups **Population**, **Agents**, **Networks**, **Policy**, **Groups**, **Stopping**; charts **Retired share**, **Retirement age**, **Transition**, **Group transitions**; time axis **Periods**; sweeps `ae-rational`, `ae-rational-replace`, `ae-threshold`, `ae-size`, `ae-extent`, `ae-policy`, `ae-coupling`, `ae-coupling-rational`; series `retired, retired_a, retired_b, transition, transition_new, transition_a, transition_b, modal_age, mean_age, rational_share, eligibility`; notices `This run has reached its last period (N) — Reset to run it again` and `The retirement norm has set in at t = N — Reset to run it again`; CLI `(its last period)` and `(the norm set in)`.
- Every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4`. Stage only the task's files; never `.claude/` or `papers/`.
- Rust: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. In `survey/`, format only `survey/src/claims/retirement.rs`; do not commit `survey/out/results-*.json`.
- Web: `(cd web && npm run build && npm test)` (run `npm ci` first in a fresh worktree).
- **Browser checks are the controller's** (Task 3's Step 6; the full pass in Task 5).

## Review Focus

1. **Who is eligible, when** — an agent ages at its own activation, so a member not yet activated this period is a year younger; eligibility, deaths and the share retired must all use `age()`. Pinned in Task 1 by `the_base_case_reaches_the_norm_and_counting_all_members_never_does` and `each_type_follows_its_rule`; in Task 4 by `retirement.ae.rapid`.
2. **Exact imitation and empty counts** — 3 of 10 reaches τ 0.3 exactly and not 0.300001; nobody counted means no retirement; counting all members includes the young. Pinned in Task 1 by `imitators_compare_retired_members_with_their_threshold_exactly`.
3. **Death and renewal bookkeeping** — the cohort index, each network's distinct members and the reverse index must match after deaths under both renewals, with groups and with tiny or empty networks. Pinned in Task 1 by `consistent` in `deaths_bring_20_year_olds_into_the_slot`, `replace_renewal_keeps_networks_full_and_indexes_right`, `groups_split_every_cohort_and_rationals_only_in_the_second` and `degenerate_configs_run_without_panicking`.
4. **The norm and the switch** — the transition is recorded once; with the policy, eligibility changes after that period and the new norm is timed from it; a live edit of the eligibility age applies only before the switch. Pinned in Task 1 by `mandatory_retirement_and_the_policy_switch` and `live_edits_apply_and_the_population_waits_for_reset`.
5. **Runs that stop at the norm** — `stop_at_norm` finishes at the first norm (the second with the policy), sweeps read the final `transition`, and the page treats such a run as finishing unpredictably. Pinned in Task 2 by `a_retirement_run_stops_at_the_norm_or_its_last_period`; in Task 3 by the models and engine tests.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --workspace` (974), `cargo clippy --all-targets -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (49), `npm run build && npm test` (713) and the survey (16 claims, 25 s).

1. **Aging at activation** (amends the spec's "everyone ages first"): the literal pseudo-code; aging everyone first doubled the transition times.
2. **`transition_a`/`transition_b`** added for Fig. 6-11's two sweeps.
3. **`mandatory` any age to 100**; the Network color mode dropped (no selection in the frame).
4. **Planning's findings** (the survey reproduces them): the realizations and network effects hold; footnote 5 fails; Fig. 6-6's minimum needs `replace`; the policy switch takes 2 periods; coupling slows the rational group; a little threshold spread doubles the time; extent has no effect at 5 % rational.

---

### Task 1: The retirement model in the core

**Files:**
- Create: `crates/sugarscape-core/src/retirement/{config,stats,view,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{lerp, Rgb}`, `crate::rng::{self, SimRng}`, `crate::schema::{Apply, Param}`, `crate::presets::ModelPreset`, `crate::opinions::Canvas`.
- Produces: `retirement::{RetirementConfig, Counts, Renewal, Order, InitialDeaths, Size, Policy, Groups, COHORTS, OLDEST, YOUNGEST, schema, presets, RetirementSnapshot, AGES_WINDOW, SERIES, population, row, AGES_W, GAP, ROW, SHOWN, TALL, TIME_W, AgentView, Kind, RetirementCell, RetirementInspection, RetirementMode, RetirementWorld}`; `RetirementWorld::{new, step, run, agents, age, eligibility, transition, transition_new, is_finished, inspect}`; `ModelKind::Retirement` (`"retirement"`), `ModelConfig::Retirement`, `ModelWorld::Retirement`; seven titles.

- [ ] **Step 1: Write the module**

Create `crates/sugarscape-core/src/retirement/config.rs` with exactly this content:

````rust
//! The Timing of Retirement's parameters: Axtell and Epstein's (1999) cohorts,
//! rationals, randoms and imitators in transient social networks, the
//! policy switch from 65 to 62, and two loosely coupled sub-populations, with
//! every detail the texts leave open as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Whom an imitator counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Counts {
    /// "some fraction f of eligibles who have actually retired" (the text).
    Eligible,
    /// Every member of its network (footnote 5's alternative).
    All,
}

/// What becomes of a dead member's place in someone's network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Renewal {
    /// It passes to the 20-year-old reborn in the dead agent's slot (the
    /// pseudo-code's reused agent objects).
    Slot,
    /// The holder picks someone new within its own extent.
    Replace,
}

/// The order agents act in each period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// Cohorts oldest first, randomly within each (the footnote: "randomized
    /// within cohorts"; the cohort order is not given).
    ByCohort,
    /// One random order over everyone (the pseudo-code).
    Shuffled,
}

/// The initial population's death ages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialDeaths {
    /// U[60, 100] for everyone: those already past it die in period 1.
    Literal,
    /// U[max(age, 60), 100]: only ages still ahead.
    Survivors,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Size {
    pub min: u32,
    pub max: u32,
}

impl Default for Size {
    /// Table 6-1: U[10, 25].
    fn default() -> Self {
        Size { min: 10, max: 25 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    pub enabled: bool,
    /// The new eligibility age, once the norm is reached.
    pub to: u32,
}

impl Default for Policy {
    /// Congress's 1961 change: 65 to 62.
    fn default() -> Self {
        Policy {
            enabled: false,
            to: 62,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Groups {
    pub enabled: bool,
    /// The chance each network member is drawn from the other group.
    pub coupling: f64,
}

impl Default for Groups {
    /// AE's animation 6-4: "10% of each agent's network belongs to the other sub-population".
    fn default() -> Self {
        Groups {
            enabled: false,
            coupling: 0.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RetirementConfig {
    /// C: agents in each of the 81 initial cohorts.
    pub per_cohort: u32,
    pub rational: f64,
    pub random: f64,
    /// A random agent's chance of retiring each eligible period.
    pub p: f64,
    /// τ: the imitators' mean threshold.
    pub threshold: f64,
    /// τ's standard deviation, spread uniformly.
    pub spread: f64,
    pub size: Size,
    /// E ~ U[0, extent].
    pub extent: u32,
    pub counts: Counts,
    pub renewal: Renewal,
    pub order: Order,
    pub initial_deaths: InitialDeaths,
    pub eligibility: u32,
    /// Everyone still working retires at this age (0: none).
    pub mandatory: u32,
    pub policy: Policy,
    pub groups: Groups,
    /// The share of eligible agents retired that marks the norm.
    pub norm: f64,
    /// Stop once the norm (with the policy, the new norm) is reached.
    pub stop_at_norm: bool,
    /// Stop at this period (0: never).
    pub stop_at: u32,
}

impl Default for RetirementConfig {
    /// Table 6-1's base case.
    fn default() -> Self {
        RetirementConfig {
            per_cohort: 100,
            rational: 0.10,
            random: 0.05,
            p: 0.5,
            threshold: 0.5,
            spread: 0.0,
            size: Size::default(),
            extent: 5,
            counts: Counts::Eligible,
            renewal: Renewal::Slot,
            order: Order::ByCohort,
            initial_deaths: InitialDeaths::Literal,
            eligibility: 65,
            mandatory: 0,
            policy: Policy::default(),
            groups: Groups::default(),
            norm: 0.95,
            stop_at_norm: false,
            stop_at: 0,
        }
    }
}

/// The youngest and oldest ages, and the cohorts between them.
pub const YOUNGEST: u32 = 20;
pub const OLDEST: u32 = 100;
pub const COHORTS: u32 = OLDEST - YOUNGEST + 1;

impl RetirementConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (1..=500).contains(&self.per_cohort),
            "per_cohort",
            "must be between 1 and 500",
        );
        check(
            unit(self.rational) && unit(self.random) && self.rational + self.random <= 1.0,
            "rational",
            "the rational and random shares must be between 0 and 1 and sum to at most 1",
        );
        check(unit(self.p), "p", "must be between 0 and 1");
        check(unit(self.threshold), "threshold", "must be between 0 and 1");
        check(
            (0.0..=0.5).contains(&self.spread),
            "spread",
            "must be between 0 and 0.5",
        );
        check(
            self.size.min <= self.size.max && self.size.max <= 200,
            "size",
            "needs min ≤ max ≤ 200",
        );
        check(self.extent <= 40, "extent", "must be at most 40");
        let age = |a: u32| (YOUNGEST..=OLDEST).contains(&a);
        check(
            age(self.eligibility),
            "eligibility",
            "must be between 20 and 100",
        );
        check(
            self.mandatory <= OLDEST,
            "mandatory",
            "must be at most 100 (0: none)",
        );
        check(
            !self.policy.enabled || age(self.policy.to),
            "policy",
            "the new eligibility age must be between 20 and 100",
        );
        check(
            unit(self.groups.coupling),
            "groups",
            "the coupling must be between 0 and 1",
        );
        check(
            self.norm > 0.0 && self.norm <= 1.0,
            "norm",
            "must be above 0 and at most 1",
        );
        check(
            self.stop_at <= 1_000_000,
            "stop_at",
            "must be at most 1000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &RetirementConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("per_cohort", self.per_cohort == next.per_cohort),
            ("rational", self.rational == next.rational),
            ("random", self.random == next.random),
            ("threshold", self.threshold == next.threshold),
            ("spread", self.spread == next.spread),
            ("size", self.size == next.size),
            ("extent", self.extent == next.extent),
            ("renewal", self.renewal == next.renewal),
            ("initial_deaths", self.initial_deaths == next.initial_deaths),
            ("groups", self.groups == next.groups),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer(
            "Population",
            "per_cohort",
            "Agents per cohort (C)",
            (1, 500),
            Reset,
        )
        .with_help("81 cohorts, ages 20 to 100. Axtell and Epstein: 100."),
        Param::choice(
            "Population",
            "initial_deaths",
            "The first agents' death ages",
            &[
                ("literal", "U[60, 100] (those past it die at once)"),
                ("survivors", "Only ages still ahead"),
            ],
            Reset,
        ),
        Param::choice(
            "Population",
            "order",
            "Each period, agents act",
            &[
                ("by_cohort", "Cohort by cohort, oldest first (the footnote)"),
                ("shuffled", "In one random order (the pseudo-code)"),
            ],
            Live,
        ),
        Param::number(
            "Agents",
            "rational",
            "Rational share",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .with_help("Retire as soon as they may."),
        Param::number("Agents", "random", "Random share", (0.0, 1.0, 0.01), Reset)
            .with_help("Retire with probability p each eligible period. The rest imitate."),
        Param::number(
            "Agents",
            "p",
            "Random agents' chance (p)",
            (0.0, 1.0, 0.01),
            Live,
        ),
        Param::number(
            "Agents",
            "threshold",
            "Imitation threshold (τ)",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .with_help("An imitator retires once this share of its network has."),
        Param::number(
            "Agents",
            "spread",
            "Threshold spread (sd)",
            (0.0, 0.5, 0.01),
            Reset,
        )
        .with_help("Uniform, as every random variable in the paper."),
        Param::choice(
            "Agents",
            "counts",
            "Imitators count",
            &[
                ("eligible", "Eligible members (the text)"),
                ("all", "Every member (footnote 5)"),
            ],
            Live,
        ),
        Param::range(
            "Networks",
            "size",
            "Network size (S)",
            (0.0, 200.0, 1.0),
            Reset,
        ),
        Param::integer(
            "Networks",
            "extent",
            "Extent (E up to, cohorts)",
            (0, 40),
            Reset,
        ),
        Param::choice(
            "Networks",
            "renewal",
            "When a member dies",
            &[
                ("slot", "The newborn in its slot takes its place"),
                ("replace", "Replaced within the holder's extent"),
            ],
            Reset,
        ),
        Param::integer("Policy", "eligibility", "Eligibility age", (20, 100), Live),
        Param::integer("Policy", "mandatory", "Mandatory age", (0, 100), Live)
            .with_help("0: none. Axtell and Epstein's policy runs: 70."),
        Param::bool(
            "Policy",
            "policy.enabled",
            "Lower the age once the norm is reached",
            Live,
        ),
        Param::integer("Policy", "policy.to", "To", (20, 100), Live)
            .shown_if("policy.enabled", "true"),
        Param::number(
            "Policy",
            "norm",
            "The norm is reached at",
            (0.01, 1.0, 0.01),
            Live,
        )
        .with_help(
            "The share of eligible agents retired. The paper never defines its transition time.",
        ),
        Param::bool("Groups", "groups.enabled", "Two sub-populations", Reset)
            .with_help("Every cohort halved; rationals only in the second half."),
        Param::number(
            "Groups",
            "groups.coupling",
            "Coupling",
            (0.0, 1.0, 0.01),
            Reset,
        )
        .shown_if("groups.enabled", "true"),
        Param::bool("Stopping", "stop_at_norm", "Stop at the norm", Live),
        Param::integer(
            "Stopping",
            "stop_at",
            "Stop at period",
            (0, 1_000_000),
            Live,
        )
        .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_table_6_1() {
        let c = RetirementConfig::default();
        assert_eq!(
            (c.per_cohort, c.rational, c.random, c.p),
            (100, 0.10, 0.05, 0.5)
        );
        assert_eq!(
            (c.threshold, c.size.min, c.size.max, c.extent),
            (0.5, 10, 25, 5)
        );
        assert_eq!((c.eligibility, c.mandatory), (65, 0));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = RetirementConfig {
            per_cohort: 0,
            rational: 0.7,
            random: 0.5,
            p: 2.0,
            threshold: -0.1,
            spread: 0.6,
            size: Size { min: 30, max: 20 },
            extent: 50,
            eligibility: 10,
            mandatory: 150,
            policy: Policy {
                enabled: true,
                to: 120,
            },
            groups: Groups {
                enabled: true,
                coupling: 1.5,
            },
            norm: 0.0,
            stop_at: 2_000_000,
            ..RetirementConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            [
                "per_cohort",
                "rational",
                "p",
                "threshold",
                "spread",
                "size",
                "extent",
                "eligibility",
                "mandatory",
                "policy",
                "groups",
                "norm",
                "stop_at"
            ]
        );
    }

    #[test]
    fn the_population_changes_only_on_reset() {
        let next = RetirementConfig {
            renewal: Renewal::Replace,
            counts: Counts::All,
            ..RetirementConfig::default()
        };
        let changes = RetirementConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "renewal");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Retirement(RetirementConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
````

Create `crates/sugarscape-core/src/retirement/stats.rs` with exactly this content:

````rust
//! The Timing of Retirement's statistics: how many eligible agents have
//! retired, by group, when the norm set in, and at what ages people retire.

use serde::Serialize;

use crate::stats::Series;

/// Periods the retirement ages look back over.
pub const AGES_WINDOW: usize = 10;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 11] = [
    "retired",
    "retired_a",
    "retired_b",
    "transition",
    "transition_new",
    "transition_a",
    "transition_b",
    "modal_age",
    "mean_age",
    "rational_share",
    "eligibility",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct RetirementSnapshot {
    pub tick: u64,
    /// The share of eligible agents retired.
    pub retired: f64,
    /// The same in the first and second group (both `retired` without groups).
    pub retired_a: f64,
    pub retired_b: f64,
    /// The period the norm was reached; NaN (null) before.
    pub transition: f64,
    /// Periods from the policy switch to the new norm; NaN (null) before.
    pub transition_new: f64,
    /// The period each group's eligible agents reached the norm (both
    /// `transition` without groups); NaN (null) before.
    pub transition_a: f64,
    pub transition_b: f64,
    /// The most common and the mean age of retirement over the last
    /// `AGES_WINDOW` periods; NaN (null) with no retirements.
    pub modal_age: f64,
    pub mean_age: f64,
    /// Rationals among the living.
    pub rational_share: f64,
    /// The eligibility age now (it drops at the policy switch).
    pub eligibility: u32,
}

impl Series for RetirementSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "retired" => self.retired,
            "retired_a" => self.retired_a,
            "retired_b" => self.retired_b,
            "transition" => self.transition,
            "transition_new" => self.transition_new,
            "transition_a" => self.transition_a,
            "transition_b" => self.transition_b,
            "modal_age" => self.modal_age,
            "mean_age" => self.mean_age,
            "rational_share" => self.rational_share,
            "eligibility" => f64::from(self.eligibility),
            _ => return None,
        })
    }
}
````

Create `crates/sugarscape-core/src/retirement/view.rs` with exactly this content:

````rust
//! The frame: Axtell and Epstein's picture of the population (one row per
//! age, 20 at the top), retirement by age (their Fig. 6-1) on the same rows,
//! and the share of eligible agents retired over time.

use crate::render::{lerp, Rgb};

/// Pixels per age row, and every panel's height (81 ages).
pub const ROW: usize = 2;
pub const TALL: usize = 81 * ROW;
/// Cells between panels.
pub const GAP: usize = 8;
/// The retirement-by-age panel's width.
pub const AGES_W: usize = 101;
/// Periods the time panel shows, and its width.
pub const SHOWN: usize = 300;
pub const TIME_W: usize = SHOWN + 1;

pub const RATIONAL: Rgb = [0xff, 0x9e, 0xc4];
pub const IMITATOR: Rgb = [0x4a, 0x7c, 0xd8];
pub const RANDOM: Rgb = [0xf2, 0xc1, 0x4e];
pub const RETIRED: Rgb = [0xe0, 0x3c, 0x31];
pub const EMPTY: Rgb = [0xee, 0xec, 0xe6];
pub const MARK: Rgb = [0x5a, 0x55, 0x4c];
pub const BAR: Rgb = [0x8f, 0xb8, 0xde];
pub const LINE: Rgb = [0xe8, 0xe4, 0xda];
pub const LOW: Rgb = [0xff, 0x5a, 0x3c];
pub const HIGH: Rgb = [0x3c, 0x6e, 0xff];
pub const GROUP_A: Rgb = [0x6c, 0xd0, 0x7a];
pub const GROUP_B: Rgb = [0xd0, 0x6c, 0xe0];

/// The population panel's columns (agents shown per age) and cell width:
/// twice C (newborn cohorts run above C), 2 pixels a cell up to C 100, 1 above.
pub fn population(per_cohort: u32) -> (usize, usize) {
    let cols = (2 * per_cohort as usize).min(600);
    (cols, if per_cohort <= 100 { 2 } else { 1 })
}

/// The time panel's row of share `x`: 1 at the top.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (TALL - 1) as f64).round() as usize
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_line_up() {
        assert_eq!(TALL, 162);
        assert_eq!((row(1.0), row(0.0)), (0, 161));
        assert_eq!(population(100), (200, 2));
        assert_eq!(population(500), (600, 1));
    }
}
````

Create `crates/sugarscape-core/src/retirement/world.rs` with exactly this content:

````rust
//! The Timing of Retirement world. Each period, in activation order, each
//! agent ages a year (the pseudo-code: "select an agent … increment its
//! age"); one past its death age dies and a 20-year-old takes its place, and
//! every other agent who may retire decides: rationals at once, randoms by
//! chance, imitators once enough of their network has. Those not yet
//! activated this period are still a year younger.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{
    Counts, InitialDeaths, Order, Renewal, RetirementConfig, COHORTS, OLDEST, YOUNGEST,
};
use super::stats::{RetirementSnapshot, AGES_WINDOW};
use super::view::{
    population, row, scale, AGES_W, BAR, EMPTY, GAP, GROUP_A, GROUP_B, HIGH, IMITATOR, LINE, LOW,
    MARK, RANDOM, RATIONAL, RETIRED, ROW, SHOWN, TALL, TIME_W,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Thresholds are kept to six decimals and compared in integers.
const SCALE: u64 = 1_000_000;

/// An agent's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Rational,
    Random,
    Imitator,
}

#[derive(Clone, Debug)]
pub struct Agent {
    /// The period of its birth at 20 (the initial agents: 0 − (age − 20)).
    pub born: i64,
    /// The last period it was activated (and aged).
    pub aged: u64,
    pub death: f64,
    pub kind: Kind,
    /// τ × 1 000 000.
    pub threshold: u64,
    pub extent: u32,
    pub group: u8,
    pub retired: bool,
    pub retired_at: Option<u32>,
    pub network: Vec<u32>,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetirementMode {
    Status,
    Type,
    Threshold,
    Group,
}

impl std::str::FromStr for RetirementMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "status" => Self::Status,
            "type" => Self::Type,
            "threshold" => Self::Threshold,
            "group" => Self::Group,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RetirementInspection {
    pub site: RetirementCell,
    /// `population`, `ages` or `time`; null between panels.
    pub panel: Option<&'static str>,
    /// The age of a population row or an ages bar.
    pub age: Option<u32>,
    /// Ages: retirements and those who could have retired, over the last 10 periods.
    pub retirements: Option<u32>,
    pub exposed: Option<u32>,
    /// Time: the period and its share retired.
    pub period: Option<u64>,
    pub retired: Option<f64>,
    /// The agent at a population cell.
    pub member: Option<AgentView>,
    /// Always null: cells are read where they are.
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct RetirementCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub age: u32,
    pub kind: Kind,
    pub threshold: f64,
    pub death_age: f64,
    pub group: u8,
    pub network: u32,
    /// Members now eligible, and retired.
    pub eligible: u32,
    pub retired_members: u32,
    pub retired: bool,
    pub retired_at: Option<u32>,
}

#[derive(Clone)]
pub struct RetirementWorld {
    pub config: RetirementConfig,
    /// Completed periods.
    pub tick: u64,
    rng: SimRng,
    agents: Vec<Agent>,
    /// Slots by birth period, in order of birth.
    cohorts: BTreeMap<i64, Vec<u32>>,
    /// Each cohort's size at birth.
    born_size: BTreeMap<i64, u32>,
    /// Who holds each slot in its network.
    known_by: Vec<Vec<u32>>,
    eligibility: u32,
    transition: Option<u64>,
    switched_at: Option<u64>,
    transition_new: Option<u64>,
    /// The period each group reached the norm.
    transition_group: [Option<u64>; 2],
    /// The last periods' share retired, and whether the policy switched there.
    recent: VecDeque<(f64, bool)>,
    /// The last periods' retirements and exposures by age (index age − 20).
    ages: VecDeque<(Vec<u32>, Vec<u32>)>,
    pub stats: Stats<RetirementSnapshot>,
}

impl RetirementWorld {
    pub fn new(config: RetirementConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let c = config.per_cohort;
        let mut world = RetirementWorld {
            eligibility: config.eligibility,
            config,
            tick: 0,
            rng: rng::seeded(seed),
            agents: Vec::new(),
            cohorts: BTreeMap::new(),
            born_size: BTreeMap::new(),
            known_by: Vec::new(),
            transition: None,
            switched_at: None,
            transition_new: None,
            transition_group: [None, None],
            recent: VecDeque::new(),
            ages: VecDeque::new(),
            stats: Stats::default(),
        };
        for k in 0..COHORTS {
            let born = -i64::from(k);
            for m in 0..c {
                let age = YOUNGEST + k;
                let group = if world.config.groups.enabled {
                    (m % 2) as u8
                } else {
                    0
                };
                let from = match world.config.initial_deaths {
                    InitialDeaths::Literal => 60.0,
                    InitialDeaths::Survivors => f64::from(age.max(60)),
                };
                let agent = world.newborn(born, group, from);
                let slot = world.agents.len() as u32;
                world.agents.push(agent);
                world.cohorts.entry(born).or_default().push(slot);
            }
            world.born_size.insert(born, c);
        }
        world.known_by = vec![Vec::new(); world.agents.len()];
        for i in 0..world.agents.len() {
            world.draw_network(i);
        }
        world.remember(false);
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Agent `i`'s age now (a year less until it is activated this period).
    pub fn age(&self, i: usize) -> u32 {
        let a = &self.agents[i];
        (i64::from(YOUNGEST) + a.aged as i64 - a.born) as u32
    }

    pub fn eligibility(&self) -> u32 {
        self.eligibility
    }

    pub fn transition(&self) -> Option<u64> {
        self.transition
    }

    pub fn transition_new(&self) -> Option<u64> {
        self.transition_new
    }

    pub fn is_finished(&self) -> bool {
        let c = &self.config;
        (c.stop_at > 0 && self.tick >= u64::from(c.stop_at))
            || (c.stop_at_norm
                && if c.policy.enabled {
                    self.transition_new.is_some()
                } else {
                    self.transition.is_some()
                })
    }

    /// A new agent born at `born` in `group`, its death age drawn from
    /// U[`from`, 100].
    fn newborn(&mut self, born: i64, group: u8, from: f64) -> Agent {
        let c = &self.config;
        let death = from + (f64::from(OLDEST) - from) * self.rng.gen::<f64>();
        let u: f64 = self.rng.gen();
        let mut kind = if u < c.rational {
            Kind::Rational
        } else if u < c.rational + c.random {
            Kind::Random
        } else {
            Kind::Imitator
        };
        if c.groups.enabled && group == 0 && kind == Kind::Rational {
            // AE: "The 50 agents on the left do not include any rational agents".
            kind = Kind::Imitator;
        }
        let half = 3f64.sqrt() * c.spread;
        let tau = (c.threshold + half * (2.0 * self.rng.gen::<f64>() - 1.0)).clamp(0.0, 1.0);
        let extent = self.rng.gen_range(0..=c.extent);
        Agent {
            born,
            aged: born.max(0) as u64,
            death,
            kind,
            threshold: (tau * SCALE as f64).round() as u64,
            extent,
            group,
            retired: false,
            retired_at: None,
            network: Vec::new(),
        }
    }

    /// Slots within `extent` cohorts of `born`, in `group` (or the other
    /// group), excluding `me`.
    fn pool(&self, born: i64, extent: u32, me: usize, same: Option<(u8, bool)>) -> Vec<u32> {
        let e = i64::from(extent);
        self.cohorts
            .range(born - e..=born + e)
            .flat_map(|(_, v)| v.iter().copied())
            .filter(|&j| {
                j as usize != me
                    && same.is_none_or(|(g, own)| (self.agents[j as usize].group == g) == own)
            })
            .collect()
    }

    /// Draws agent `i`'s network: S distinct members within its extent (with
    /// groups, each from the other group with probability `coupling`).
    fn draw_network(&mut self, i: usize) {
        let (lo, hi) = (self.config.size.min, self.config.size.max);
        let s = self.rng.gen_range(lo..=hi) as usize;
        let (born, extent, group) = (
            self.agents[i].born,
            self.agents[i].extent,
            self.agents[i].group,
        );
        let groups = self.config.groups.enabled;
        let mut own = self.pool(born, extent, i, groups.then_some((group, true)));
        let mut other = if groups {
            self.pool(born, extent, i, Some((group, false)))
        } else {
            Vec::new()
        };
        let mut net = Vec::with_capacity(s);
        for _ in 0..s {
            let cross = groups && self.rng.gen::<f64>() < self.config.groups.coupling;
            let from = if (cross && !other.is_empty()) || own.is_empty() {
                &mut other
            } else {
                &mut own
            };
            if from.is_empty() {
                break;
            }
            let k = self.rng.gen_range(0..from.len() as u32) as usize;
            net.push(from.swap_remove(k));
        }
        for &m in &net {
            self.known_by[m as usize].push(i as u32);
        }
        self.agents[i].network = net;
    }

    /// Agent `i` dies; a 20-year-old takes its slot.
    fn die(&mut self, i: usize) {
        let old = self.agents[i].born;
        if let Some(v) = self.cohorts.get_mut(&old) {
            v.retain(|&j| j as usize != i);
            if v.is_empty() {
                self.cohorts.remove(&old);
            }
        }
        for m in std::mem::take(&mut self.agents[i].network) {
            self.known_by[m as usize].retain(|&h| h as usize != i);
        }
        let group = self.agents[i].group;
        let born = self.tick as i64;
        self.agents[i] = self.newborn(born, group, 60.0);
        self.cohorts.entry(born).or_default().push(i as u32);
        *self.born_size.entry(born).or_default() += 1;
        if self.config.renewal == Renewal::Replace {
            // Everyone who knew the dead agent picks someone new within their extent.
            for h in std::mem::take(&mut self.known_by[i]) {
                let h = h as usize;
                let (hb, he, hg) = (
                    self.agents[h].born,
                    self.agents[h].extent,
                    self.agents[h].group,
                );
                let same = self.config.groups.enabled.then_some((hg, true));
                let pool: Vec<u32> = self
                    .pool(hb, he, h, same)
                    .into_iter()
                    .filter(|&k| k as usize != i && !self.agents[h].network.contains(&k))
                    .collect();
                let pick = if pool.is_empty() {
                    None
                } else {
                    Some(pool[self.rng.gen_range(0..pool.len() as u32) as usize])
                };
                let net = &mut self.agents[h].network;
                if let Some(k) = pick {
                    for m in net.iter_mut().filter(|m| **m as usize == i) {
                        *m = k;
                    }
                    self.known_by[k as usize].push(h as u32);
                } else {
                    net.retain(|&m| m as usize != i);
                }
            }
        }
        self.draw_network(i);
    }

    fn retire(&mut self, i: usize, age: u32, log: &mut [u32]) {
        self.agents[i].retired = true;
        self.agents[i].retired_at = Some(age);
        log[(age - YOUNGEST) as usize] += 1;
    }

    /// Whether imitator `i` sees enough of its network retired.
    fn imitates(&self, i: usize) -> bool {
        let (mut counted, mut retired) = (0u64, 0u64);
        for &m in &self.agents[i].network {
            let m = m as usize;
            if self.config.counts == Counts::Eligible && self.age(m) < self.eligibility {
                continue;
            }
            counted += 1;
            if self.agents[m].retired {
                retired += 1;
            }
        }
        counted > 0 && retired * SCALE >= self.agents[i].threshold * counted
    }

    pub fn step(&mut self) {
        self.tick += 1;
        let n = self.agents.len();
        let mut order: Vec<u32> = (0..n as u32).collect();
        for i in (1..n).rev() {
            let j = self.rng.gen_range(0..=i as u32) as usize;
            order.swap(i, j);
        }
        if self.config.order == Order::ByCohort {
            // Oldest cohorts first; the shuffle's order kept within each.
            let born: Vec<i64> = self.agents.iter().map(|a| a.born).collect();
            order.sort_by_key(|&i| born[i as usize]);
        }
        let mut retirements = vec![0u32; COHORTS as usize];
        let mut exposed = vec![0u32; COHORTS as usize];
        for i in order {
            let i = i as usize;
            self.agents[i].aged = self.tick;
            let age = self.age(i);
            if f64::from(age) >= self.agents[i].death || age > OLDEST {
                self.die(i);
                continue;
            }
            if self.agents[i].retired {
                continue;
            }
            let c = &self.config;
            if c.mandatory > 0 && age >= c.mandatory {
                self.retire(i, age, &mut retirements);
                continue;
            }
            if age < self.eligibility {
                continue;
            }
            exposed[(age - YOUNGEST) as usize] += 1;
            let go = match self.agents[i].kind {
                Kind::Rational => true,
                Kind::Random => self.rng.gen::<f64>() < self.config.p,
                Kind::Imitator => self.imitates(i),
            };
            if go {
                self.retire(i, age, &mut retirements);
            }
        }
        if self.ages.len() == AGES_WINDOW {
            self.ages.pop_front();
        }
        self.ages.push_back((retirements, exposed));
        let share = self.share(None);
        if self.config.groups.enabled && self.switched_at.is_none() {
            for g in 0..2u8 {
                if self.transition_group[g as usize].is_none()
                    && self.share(Some(g)) >= self.config.norm
                {
                    self.transition_group[g as usize] = Some(self.tick);
                }
            }
        }
        let mut switched = false;
        if self.transition.is_none() {
            if share >= self.config.norm {
                self.transition = Some(self.tick);
                if self.config.policy.enabled {
                    self.eligibility = self.config.policy.to;
                    self.switched_at = Some(self.tick);
                    switched = true;
                }
            }
        } else if let (Some(at), None) = (self.switched_at, self.transition_new) {
            if share >= self.config.norm {
                self.transition_new = Some(self.tick - at);
            }
        }
        self.remember(switched);
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// The share of eligible agents retired (in `group`, if given).
    fn share(&self, group: Option<u8>) -> f64 {
        let (mut eligible, mut retired) = (0u32, 0u32);
        for i in 0..self.agents.len() {
            if group.is_some_and(|g| self.agents[i].group != g) || self.age(i) < self.eligibility {
                continue;
            }
            eligible += 1;
            if self.agents[i].retired {
                retired += 1;
            }
        }
        if eligible == 0 {
            0.0
        } else {
            f64::from(retired) / f64::from(eligible)
        }
    }

    fn remember(&mut self, switched: bool) {
        if self.recent.len() == SHOWN {
            self.recent.pop_front();
        }
        self.recent.push_back((self.share(None), switched));
    }

    fn record(&mut self) {
        let nan_or = |x: Option<u64>| x.map_or(f64::NAN, |v| v as f64);
        let mut counts = vec![0u32; COHORTS as usize];
        for (r, _) in &self.ages {
            for (k, v) in r.iter().enumerate() {
                counts[k] += v;
            }
        }
        let total: u32 = counts.iter().sum();
        let (modal, mean) = if total == 0 {
            (f64::NAN, f64::NAN)
        } else {
            let top = (0..counts.len())
                .max_by_key(|&k| (counts[k], std::cmp::Reverse(k)))
                .unwrap();
            let sum: u64 = counts
                .iter()
                .enumerate()
                .map(|(k, &v)| u64::from(v) * u64::from(YOUNGEST + k as u32))
                .sum();
            (
                f64::from(YOUNGEST + top as u32),
                sum as f64 / f64::from(total),
            )
        };
        let all = self.share(None);
        let groups = self.config.groups.enabled;
        let rationals = self
            .agents
            .iter()
            .filter(|a| a.kind == Kind::Rational)
            .count();
        let s = RetirementSnapshot {
            tick: self.tick,
            retired: all,
            retired_a: if groups { self.share(Some(0)) } else { all },
            retired_b: if groups { self.share(Some(1)) } else { all },
            transition: nan_or(self.transition),
            transition_new: nan_or(self.transition_new),
            transition_a: nan_or(if groups {
                self.transition_group[0]
            } else {
                self.transition
            }),
            transition_b: nan_or(if groups {
                self.transition_group[1]
            } else {
                self.transition
            }),
            modal_age: modal,
            mean_age: mean,
            rational_share: rationals as f64 / self.agents.len() as f64,
            eligibility: self.eligibility,
        };
        self.stats.push(s);
    }

    fn view(&self, i: usize) -> AgentView {
        let a = &self.agents[i];
        let (mut eligible, mut retired) = (0, 0);
        for &m in &a.network {
            if self.age(m as usize) >= self.eligibility {
                eligible += 1;
            }
            if self.agents[m as usize].retired {
                retired += 1;
            }
        }
        AgentView {
            id: i as u64 + 1,
            age: self.age(i),
            kind: a.kind,
            threshold: a.threshold as f64 / SCALE as f64,
            death_age: a.death,
            group: a.group,
            network: a.network.len() as u32,
            eligible,
            retired_members: retired,
            retired: a.retired,
            retired_at: a.retired_at,
        }
    }

    fn color(&self, mode: RetirementMode, i: usize) -> [u8; 3] {
        let a = &self.agents[i];
        let kind = match a.kind {
            Kind::Rational => RATIONAL,
            Kind::Imitator => IMITATOR,
            Kind::Random => RANDOM,
        };
        match mode {
            RetirementMode::Status => {
                if a.retired {
                    RETIRED
                } else {
                    kind
                }
            }
            RetirementMode::Type => kind,
            RetirementMode::Threshold => scale(a.threshold as f64 / SCALE as f64, LOW, HIGH),
            RetirementMode::Group => {
                if a.group == 0 {
                    GROUP_A
                } else {
                    GROUP_B
                }
            }
        }
    }

    /// The population panel's width, and where the other two panels start.
    fn layout(&self) -> (usize, usize, usize) {
        let (cols, cell) = population(self.config.per_cohort);
        let pop = cols * cell;
        let ages_x = pop + GAP;
        (pop, ages_x, ages_x + AGES_W + GAP)
    }

    /// The hazard of retiring at each age over the last `AGES_WINDOW` periods.
    fn hazards(&self) -> (Vec<u32>, Vec<u32>) {
        let mut r = vec![0u32; COHORTS as usize];
        let mut e = vec![0u32; COHORTS as usize];
        for (rs, es) in &self.ages {
            for k in 0..COHORTS as usize {
                r[k] += rs[k];
                e[k] += es[k];
            }
        }
        (r, e)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<RetirementInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let (pop, ages_x, time_x) = self.layout();
        let age = YOUNGEST + (cy / ROW) as u32;
        let mut out = RetirementInspection {
            site: RetirementCell { x, y },
            panel: None,
            age: None,
            retirements: None,
            exposed: None,
            period: None,
            retired: None,
            member: None,
            agent: None,
        };
        if cx < pop {
            out.panel = Some("population");
            out.age = Some(age);
            let (_, cell) = population(self.config.per_cohort);
            let born = self.tick as i64 - i64::from(age - YOUNGEST);
            if let Some(&slot) = self.cohorts.get(&born).and_then(|v| v.get(cx / cell)) {
                out.member = Some(self.view(slot as usize));
            }
        } else if (ages_x..ages_x + AGES_W).contains(&cx) {
            let (r, e) = self.hazards();
            let k = (age - YOUNGEST) as usize;
            out.panel = Some("ages");
            out.age = Some(age);
            out.retirements = Some(r[k]);
            out.exposed = Some(e[k]);
        } else if cx >= time_x {
            if let Some(&(share, _)) = self.recent.get(cx - time_x) {
                out.panel = Some("time");
                out.period = Some(self.tick + 1 + (cx - time_x) as u64 - self.recent.len() as u64);
                out.retired = Some(share);
            }
        }
        Ok(out)
    }
}

impl Model for RetirementWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Retirement(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        RetirementWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick, the eligibility and every agent's state.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.eligibility.to_le_bytes());
        for a in &self.agents {
            eat(&a.born.to_le_bytes());
            eat(&a.death.to_bits().to_le_bytes());
            eat(&a.threshold.to_le_bytes());
            eat(&[a.kind as u8, a.group, u8::from(a.retired), a.extent as u8]);
            for m in &a.network {
                eat(&m.to_le_bytes());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (_, _, time_x) = self.layout();
        ((time_x + TIME_W) as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: RetirementMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let (pop, ages_x, time_x) = self.layout();
        let (cols, cell) = population(self.config.per_cohort);
        // The population: one row per age, agents in order of birth; empty
        // places (the cohort's dead) light.
        for k in 0..COHORTS {
            let born = self.tick as i64 - i64::from(k);
            let members = self.cohorts.get(&born).map_or(&[][..], Vec::as_slice);
            let places = (*self.born_size.get(&born).unwrap_or(&0) as usize).min(cols);
            for col in 0..places.max(members.len().min(cols)) {
                let color = members
                    .get(col)
                    .map_or(EMPTY, |&i| self.color(mode, i as usize));
                for dy in 0..ROW {
                    for dx in 0..cell {
                        c.put(col * cell + dx, k as usize * ROW + dy, color);
                    }
                }
            }
        }
        let _ = pop;
        // Retirement by age: the hazard over the last periods; the
        // eligibility and mandatory ages marked.
        let (r, e) = self.hazards();
        for k in 0..COHORTS as usize {
            let hz = if e[k] == 0 {
                0.0
            } else {
                f64::from(r[k]) / f64::from(e[k])
            };
            let len = (hz * (AGES_W - 1) as f64).round() as usize;
            for x in 0..len {
                for dy in 0..ROW {
                    c.put(ages_x + x, k * ROW + dy, BAR);
                }
            }
        }
        for age in [self.eligibility, self.config.mandatory] {
            if (YOUNGEST..=OLDEST).contains(&age) {
                let y = (age - YOUNGEST) as usize * ROW;
                for x in 0..AGES_W {
                    c.put(ages_x + x, y, MARK);
                }
            }
        }
        // The share retired over time; the norm and the policy switch marked.
        let norm = row(self.config.norm);
        for x in 0..TIME_W {
            c.put(time_x + x, norm, MARK);
        }
        for (k, &(share, switched)) in self.recent.iter().enumerate() {
            if switched {
                c.column(time_x + k, 0, TALL - 1, MARK);
            }
            let y = row(share);
            let to = self.recent.get(k + 1).map_or(y, |next| row(next.0));
            c.column(time_x + k, y, to, LINE);
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        super::SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out =
            String::from("id,age,kind,threshold,death_age,group,network,eligible,retired_members,retired,retired_at\n");
        for i in 0..self.agents.len() {
            let v = self.view(i);
            writeln!(
                out,
                "{},{},{:?},{},{},{},{},{},{},{},{}",
                v.id,
                v.age,
                v.kind,
                v.threshold,
                v.death_age,
                v.group,
                v.network,
                v.eligible,
                v.retired_members,
                u8::from(v.retired),
                v.retired_at.map_or(String::new(), |a| a.to_string())
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Nothing to follow: Inspect reads a cell.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Retirement(next) = next else {
            return Err(wrong_model(ModelKind::Retirement, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        if self.switched_at.is_none() {
            self.eligibility = next.eligibility;
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped at the norm or at `stop_at`: a sweep reads its last period.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retirement::config::{Groups, Policy, Size};

    fn config(edit: impl FnOnce(&mut RetirementConfig)) -> RetirementConfig {
        let mut c = RetirementConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut RetirementConfig)) -> RetirementWorld {
        RetirementWorld::new(config(edit), 1).unwrap()
    }

    /// The cohort and reverse indexes match the agents.
    fn consistent(w: &RetirementWorld) {
        let mut seen = 0;
        for (&born, v) in &w.cohorts {
            for &i in v {
                assert_eq!(w.agents[i as usize].born, born);
                seen += 1;
            }
        }
        assert_eq!(seen, w.agents.len());
        for (i, a) in w.agents.iter().enumerate() {
            let mut sorted = a.network.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), a.network.len(), "distinct members of {i}");
            assert!(!a.network.contains(&(i as u32)));
            for &m in &a.network {
                assert!(
                    w.known_by[m as usize].contains(&(i as u32)),
                    "{m} known by {i}"
                );
            }
        }
        for (m, holders) in w.known_by.iter().enumerate() {
            for &h in holders {
                assert!(w.agents[h as usize].network.contains(&(m as u32)));
            }
        }
    }

    #[test]
    fn the_population_is_81_cohorts_of_c_with_networks_in_range() {
        let w = world(|c| c.per_cohort = 20);
        assert_eq!(w.agents.len(), 81 * 20);
        assert_eq!((w.age(0), w.age(81 * 20 - 1)), (20, 100));
        for (i, a) in w.agents.iter().enumerate() {
            assert!(a.extent <= 5 && a.death >= 60.0 && a.death <= 100.0);
            assert!(a.network.len() <= 25);
            for &m in &a.network {
                assert!((w.agents[m as usize].born - a.born).unsigned_abs() <= u64::from(a.extent));
            }
            assert!(!a.retired, "{i} starts working");
        }
        consistent(&w);
        let kinds = |k: Kind| w.agents.iter().filter(|a| a.kind == k).count() as f64 / 1620.0;
        assert!((kinds(Kind::Rational) - 0.10).abs() < 0.03);
        assert!((kinds(Kind::Random) - 0.05).abs() < 0.02);
    }

    #[test]
    fn deaths_bring_20_year_olds_into_the_slot() {
        let mut w = world(|c| c.per_cohort = 20);
        w.step();
        // Everyone past a literal death age died in period 1.
        let newborns: Vec<usize> = (0..w.agents.len()).filter(|&i| w.age(i) == 20).collect();
        assert!(newborns.len() > 300, "{}", newborns.len());
        for &i in &newborns {
            assert_eq!(w.agents[i].born, 1);
            assert!(!w.agents[i].retired);
        }
        consistent(&w);
        let mut s = world(|c| {
            c.per_cohort = 20;
            c.initial_deaths = InitialDeaths::Survivors;
        });
        s.step();
        let fewer = (0..s.agents.len()).filter(|&i| s.age(i) == 20).count();
        assert!(fewer < newborns.len() / 3, "{fewer}");
    }

    #[test]
    fn each_type_follows_its_rule() {
        let mut all_rational = world(|c| {
            c.per_cohort = 10;
            c.rational = 1.0;
            c.random = 0.0;
        });
        all_rational.step();
        for i in 0..all_rational.agents.len() {
            let a = &all_rational.agents[i];
            assert_eq!(a.retired, all_rational.age(i) >= 65 && a.born < 1, "{i}");
        }
        let mut randoms = world(|c| {
            c.per_cohort = 100;
            c.rational = 0.0;
            c.random = 1.0;
        });
        randoms.step();
        let s = randoms.stats.latest().unwrap().retired;
        assert!((s - 0.5).abs() < 0.05, "{s}");
    }

    #[test]
    fn imitators_compare_retired_members_with_their_threshold_exactly() {
        let mut w = world(|c| {
            c.per_cohort = 10;
            c.rational = 0.0;
            c.random = 0.0;
        });
        // Agent 800 (age 100) with a hand network: 3 of 10 eligible members retired.
        let me = 80 * 10;
        let members: Vec<u32> = (70 * 10..70 * 10 + 10).collect();
        w.agents[me].network = members.clone();
        for (k, &m) in members.iter().enumerate() {
            w.agents[m as usize].retired = k < 3;
        }
        w.agents[me].threshold = 300_000;
        assert!(w.imitates(me), "3 of 10 reaches 0.3 exactly");
        w.agents[me].threshold = 300_001;
        assert!(!w.imitates(me));
        // Counting all members: two young members join the count.
        w.agents[me].network.extend([0, 1]);
        w.agents[me].threshold = 300_000;
        assert!(w.imitates(me), "eligible only: still 3 of 10");
        w.config.counts = Counts::All;
        assert!(!w.imitates(me), "all members: 3 of 12");
        w.agents[me].network.clear();
        assert!(!w.imitates(me), "nobody counted: no retirement");
    }

    #[test]
    fn the_base_case_reaches_the_norm_and_counting_all_members_never_does() {
        let mut w = world(|_| {});
        w.run(200);
        let t = w.transition().expect("the 65 norm");
        assert!((5..120).contains(&t), "{t}");
        let mut all = world(|c| c.counts = Counts::All);
        all.run(300);
        assert!(
            all.transition().is_none(),
            "footnote 5's alternative never sets in"
        );
    }

    #[test]
    fn replace_renewal_keeps_networks_full_and_indexes_right() {
        let mut w = world(|c| {
            c.per_cohort = 20;
            c.renewal = Renewal::Replace;
        });
        let sizes: Vec<usize> = w.agents.iter().map(|a| a.network.len()).collect();
        w.run(30);
        consistent(&w);
        // Nobody in a network is a newborn from a slot someone died in, unless
        // it was drawn there anew within the holder's extent.
        for a in &w.agents {
            for &m in &a.network {
                assert!((w.agents[m as usize].born - a.born).unsigned_abs() <= u64::from(a.extent));
            }
        }
        let _ = sizes;
        let mut s = world(|c| c.per_cohort = 20);
        s.run(30);
        consistent(&s);
        let stale = s
            .agents
            .iter()
            .flat_map(|a| a.network.iter().map(move |&m| (a, m)))
            .filter(|(a, m)| {
                (s.agents[*m as usize].born - a.born).unsigned_abs() > u64::from(a.extent)
            })
            .count();
        assert!(stale > 0, "slot renewal leaves newborns in old networks");
    }

    #[test]
    fn mandatory_retirement_and_the_policy_switch() {
        let mut w = world(|c| {
            c.rational = 0.05;
            c.random = 0.05;
            c.mandatory = 70;
            c.policy = Policy {
                enabled: true,
                to: 62,
            };
            c.stop_at_norm = true;
        });
        w.run(500);
        assert!(w.is_finished());
        let t = w.transition().unwrap();
        assert_eq!(w.eligibility(), 62);
        let t2 = w.transition_new().unwrap();
        assert!(
            t2 < 30,
            "the new norm in {t2} periods after the switch at {t}"
        );
        for i in 0..w.agents.len() {
            if w.age(i) >= 70 {
                assert!(w.agents[i].retired, "{i}");
            }
        }
    }

    #[test]
    fn groups_split_every_cohort_and_rationals_only_in_the_second() {
        let w = world(|c| {
            c.per_cohort = 40;
            c.groups = Groups {
                enabled: true,
                coupling: 0.2,
            };
        });
        assert!(w
            .agents
            .iter()
            .filter(|a| a.group == 0)
            .all(|a| a.kind != Kind::Rational));
        assert!(w
            .agents
            .iter()
            .any(|a| a.group == 1 && a.kind == Kind::Rational));
        let (mut cross, mut total) = (0, 0);
        for a in &w.agents {
            for &m in &a.network {
                total += 1;
                if w.agents[m as usize].group != a.group {
                    cross += 1;
                }
            }
        }
        let share = f64::from(cross) / f64::from(total);
        assert!((share - 0.2).abs() < 0.03, "{share}");
        consistent(&w);
        let mut w = w;
        w.run(300);
        let s = w.stats.latest().unwrap();
        assert!(
            s.transition_a.is_finite() && s.transition_b.is_finite(),
            "{s:?}"
        );
        assert!(
            s.transition_b <= s.transition_a,
            "the group with rationals first"
        );
    }

    #[test]
    fn statistics_track_shares_ages_and_the_transition() {
        let mut w = world(|c| c.rational = 0.2);
        w.run(40);
        let s = w.stats.latest().unwrap().clone();
        assert!(s.retired > 0.9);
        assert_eq!(s.transition, w.transition().unwrap() as f64);
        assert!(s.transition_new.is_nan());
        assert!(s.modal_age >= 65.0 && s.mean_age >= 65.0, "{s:?}");
        assert!((s.retired_a - s.retired).abs() < 1e-12);
        assert!(Model::latest_json(&w).contains("\"transition_new\":null"));
    }

    #[test]
    fn activation_order_is_by_cohort_or_shuffled() {
        let a = world(|c| c.per_cohort = 20);
        let b = world(|c| {
            c.per_cohort = 20;
            c.order = Order::Shuffled;
        });
        let (mut a, mut b) = (a, b);
        a.run(20);
        b.run(20);
        assert_ne!(Model::fingerprint(&a), Model::fingerprint(&b));
    }

    #[test]
    fn the_frame_draws_the_population_ages_and_time() {
        let mut w = world(|_| {});
        w.run(10);
        let mut buf = Vec::new();
        w.render("status", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw, fh), ((400 + 8 + 101 + 8 + 301) as u32, 162));
        for mode in ["type", "threshold", "group"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
    }

    #[test]
    fn inspect_reads_agents_ages_and_periods() {
        let mut w = world(|_| {});
        w.run(10);
        let a = w.inspect(0, (45 * ROW) as u32).unwrap();
        assert_eq!((a.panel, a.age), (Some("population"), Some(65)));
        let m = a.member.unwrap();
        assert_eq!(m.age, 65);
        assert!(m.network >= 10 || m.network == 0);
        let g = w.inspect(409, (45 * ROW) as u32).unwrap();
        assert_eq!(g.panel, Some("ages"));
        assert!(g.exposed.unwrap() > 0);
        let t = w.inspect(517 + 10, 0).unwrap();
        assert_eq!((t.panel, t.period), (Some("time"), Some(10)));
        assert_eq!(Model::locate(&w, 1), None);
    }

    #[test]
    fn keyframes_restore_the_world_and_its_view() {
        let c = config(|c| {
            c.per_cohort = 20;
            c.renewal = Renewal::Replace;
        });
        let mut any = crate::model::ModelWorld::new(ModelConfig::Retirement(c.clone()), 6).unwrap();
        any.model_mut().run(10);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("status", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("status", "", &mut after).unwrap();
        assert_eq!(before, after);
        any.model_mut().run(10);
        let mut fresh = crate::model::ModelWorld::new(ModelConfig::Retirement(c), 6).unwrap();
        fresh.model_mut().run(20);
        assert_eq!(
            any.model().fingerprint(),
            fresh.model().fingerprint(),
            "replays the same"
        );
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|c| c.per_cohort = 20);
        let next = config(|c| {
            c.per_cohort = 20;
            c.counts = Counts::All;
            c.mandatory = 70;
            c.stop_at = 5;
        });
        Model::set_config(&mut w, ModelConfig::Retirement(next)).unwrap();
        w.run(100);
        assert_eq!(w.tick, 5);
        for (field, edit) in [
            ("per_cohort", config(|c| c.per_cohort = 30)),
            (
                "renewal",
                config(|c| {
                    c.per_cohort = 20;
                    c.renewal = Renewal::Replace;
                }),
            ),
            (
                "size",
                config(|c| {
                    c.per_cohort = 20;
                    c.size = Size { min: 5, max: 6 };
                }),
            ),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Retirement(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.per_cohort = 1),
            config(|c| {
                c.per_cohort = 5;
                c.size = Size { min: 0, max: 0 };
            }),
            config(|c| {
                c.per_cohort = 5;
                c.extent = 0;
                c.renewal = Renewal::Replace;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.rational = 0.0;
                c.random = 0.0;
                c.spread = 0.5;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.groups = Groups {
                    enabled: true,
                    coupling: 1.0,
                };
                c.renewal = Renewal::Replace;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.eligibility = 20;
                c.mandatory = 20;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.size = Size { min: 200, max: 200 };
            }),
        ] {
            let mut w = RetirementWorld::new(c.clone(), 1).unwrap();
            w.run(30);
            consistent(&w);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.retired), "{c:?}");
            let mut buf = Vec::new();
            w.render("status", "", &mut buf).unwrap();
        }
    }
}
````

Create `crates/sugarscape-core/src/retirement/presets.rs` with exactly this content:

````rust
//! Axtell and Epstein's realizations, policy switch and sub-populations, and
//! the two readings that decide their results.

use super::config::{Counts, Groups, Policy, Renewal, RetirementConfig};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const AE: &str = "Axtell & Epstein 1999, Brookings CSED WP 1";
const GSS: &str = "Epstein 2006, Generative Social Science, ch. 7";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut RetirementConfig),
) -> ModelPreset {
    let mut c = RetirementConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Retirement(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ae-rapid",
            "Fig. 6-4: 15 % rational",
            GSS,
            "Axtell and Epstein's retirement model: 81 cohorts of 100 people aged 20 to 100, each with a random death age (U[60, 100]) and replaced at death by a 20-year-old. From 65 people may retire. 15 % are rational and retire at 65; 5 % retire at random (half each year); the other 80 % imitate: each has a network of 10 to 25 people within five years of its own age and retires once half its eligible friends have. The picture is theirs: one row per age, 20 at the top, rationals pink, imitators blue, randoms yellow, the retired red. 'Within the first 6 periods essentially all of the eligible population has retired.' Measured (20 seeds): 95 % of those eligible retired by period 7.7 on average — by period 6 in 4 runs of 20 (this one), by period 10 in all — rising steadily every time. Close: a period or two later than 'the first 6'.",
            |c| {
                c.rational = 0.15;
            },
        ),
        preset(
            "ae-base",
            "Table 6-1: the base case",
            AE,
            "Table 6-1's base case: 10 % rational, 85 % imitators, 5 % random, thresholds 0.5, networks of 10–25 people within up to five cohorts. Axtell and Epstein never define their 'transition time'; here it is the first period in which 95 % of the eligible have retired. Measured (20 seeds): 16 periods (± 3). Their other choices matter too: activating everyone in one random order instead of cohort by cohort, oldest first, it takes 26; counting every friend instead of the eligible ones, never.",
            |_| {},
        ),
        preset("ae-slow", "Fig. 6-5: 5 % rational", AE, "Figure 6-5: only 5 % rational, 90 % imitators. Retirement stalls at a fifth or so of those eligible, wavers — 'the trajectory is not monotone' — and then, once the retired old have spread their example, sweeps to 100 %: 'It is as if retirement percolates up from older to younger agents.' Measured (20 seeds): 61 periods (± 3); the first run climbs to 31 % by period 40, falls back to 21 %, and completes at period 70. Reproduced.", |c| {
            c.rational = 0.05;
        }),
        preset(
            "ae-policy",
            "Animation 6-3: 65 to 62",
            AE,
            "Axtell and Epstein's policy experiment: retirement mandatory at 70, and once the age 65 norm is established, the eligibility age drops to 62, as Congress's did in 1961. The paper: the new norm 'emerges after twenty to thirty periods', and 'in about 35 periods if between 1 and 4 percent of the population responds rationally' — the sluggish response the model was built to explain. Measured (20 seeds): the new norm is reached 2 periods after the switch, at every rational share from 0 to 14 % (the ae-policy sweep). Imitators just turned 62 count their eligible friends, who now include the retired 65-to-67-year-olds; half have retired, so they retire at once. The stated rules do not produce the decades.",
            |c| {
                c.rational = 0.05;
                c.mandatory = 70;
                c.policy = Policy {
                    enabled: true,
                    ..Policy::default()
                };
            },
        ),
        preset(
            "ae-groups",
            "Animation 6-4: two sub-populations",
            AE,
            "Figure 6-11's two sub-populations: every cohort split in half, rationals (10 %) only in the second half, and each network drawing 10 % of its members from the other half. Axtell and Epstein: this 'loose coupling is sufficient for the group containing some rationals to pull the other into conformity'. Measured (the ae-coupling sweeps, 10 seeds): the group without rationals reaches the norm at period 46 instead of 75 — pulled in, as they say — but the group with rationals slows from 19 to 34, and at couplings of 0.2 and more both take about 58. The coupling pulls both ways; in their figure the rational group stays fast.",
            |c| {
                c.groups = Groups {
                    enabled: true,
                    ..Groups::default()
                };
            },
        ),
        preset(
            "ae-all-members",
            "Footnote 5: counting every member",
            AE,
            "Footnote 5: 'It makes a difference to the numerical results whether an agent considers all agents in its social network, or only those who are eligible to retire. However, the qualitative character of the results … do not depend on this distinction.' Here imitators count every friend. A network spans up to five years either side, so the young friends who cannot retire hold the share below one half, and imitators never retire. Measured (20 seeds, 600 periods): no norm, ever; only the rationals and randoms retire.",
            |c| {
                c.counts = Counts::All;
            },
        ),
        preset(
            "ae-replace",
            "Friends replaced, 5 % rational",
            AE,
            "A choice the paper never states: when a friend dies, is its place in the network taken by the 20-year-old reborn in its slot (the default; their pseudo-code reuses agent objects) or does the agent find a replacement of about its own age (here)? With friends replaced, networks keep their eligible members and imitation is harder: 5 % rationality never establishes the norm (10 seeds, 600 periods), and 10 % takes 70 ± 75 periods — the 'minimum proportions … rational' and the rapidly growing variance of Axtell and Epstein's Figure 6-6, which the default reading does not produce.",
            |c| {
                c.rational = 0.05;
                c.renewal = Renewal::Replace;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, RetirementConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Retirement(c) => (p.id, c),
                _ => panic!("{} is not a retirement preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("ae-base"), RetirementConfig::default());
        assert_eq!(find("ae-rapid").rational, 0.15);
        let p = find("ae-policy");
        assert_eq!((p.mandatory, p.policy.enabled, p.policy.to), (70, true, 62));
        assert_eq!(find("ae-all-members").counts, Counts::All);
        assert_eq!(find("ae-replace").renewal, Renewal::Replace);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
````

Create `crates/sugarscape-core/src/retirement/mod.rs` with exactly this content:

````rust
//! The Timing of Retirement (milestone 26): Axtell and Epstein, "Coordination
//! in Transient Social Networks: An Agent-Based Computational Model of the
//! Timing of Retirement" (Brookings CSED Working Paper 1, 1999), with the
//! revised text in Epstein, *Generative Social Science* (2006), chapter 7.
//! See docs/superpowers/specs/2026-09-27-retirement-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Counts, Groups, InitialDeaths, Order, Policy, Renewal, RetirementConfig, Size, COHORTS,
    OLDEST, YOUNGEST,
};
pub use presets::presets;
pub use stats::{RetirementSnapshot, AGES_WINDOW, SERIES};
pub use view::{population, row, AGES_W, GAP, ROW, SHOWN, TALL, TIME_W};
pub use world::{
    AgentView, Kind, RetirementCell, RetirementInspection, RetirementMode, RetirementWorld,
};
````

- [ ] **Step 2: Wire the model kind and title its presets**

Modify `crates/sugarscape-core/src/lib.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index 7741f44..cbe8b98 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -32,6 +32,7 @@ pub mod opinions;
 pub mod portable;
 pub mod presets;
 pub mod render;
+pub mod retirement;
 pub mod ring;
 pub mod rng;
 pub mod rules;
````

Modify `crates/sugarscape-core/src/model.rs` — every match gains `Retirement`; the reader gains its `"retirement"` arm; a round-trip test pins it (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index fc70618..8c0221c 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -19,6 +19,7 @@ use crate::image::{ImageConfig, ImageWorld};
 use crate::norms::{NormsConfig, NormsWorld};
 use crate::opinions::{OpinionsConfig, OpinionsWorld};
 use crate::render::{self, ColorMode, Layer};
+use crate::retirement::{RetirementConfig, RetirementWorld};
 use crate::ring::{RingConfig, RingWorld};
 use crate::schelling::{SchellingConfig, SchellingWorld};
 use crate::schema::Param;
@@ -29,7 +30,7 @@ use crate::thresholds::{ThresholdsConfig, ThresholdsWorld};
 use crate::world::World;
 use crate::{
     agreement, anasazi, ants, civil, classes, culture, dpd, ethno, export, farol, image, norms,
-    opinions, ring, schelling, spatial, stats, structure, tags, thresholds,
+    opinions, retirement, ring, schelling, spatial, stats, structure, tags, thresholds,
 };
 
 /// Which model a config or world is.
@@ -55,10 +56,11 @@ pub enum ModelKind {
     Farol,
     Ants,
     Thresholds,
+    Retirement,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 19] = [
+    pub const ALL: [ModelKind; 20] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -78,6 +80,7 @@ impl ModelKind {
         ModelKind::Farol,
         ModelKind::Ants,
         ModelKind::Thresholds,
+        ModelKind::Retirement,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -101,6 +104,7 @@ impl ModelKind {
             ModelKind::Farol => "farol",
             ModelKind::Ants => "ants",
             ModelKind::Thresholds => "thresholds",
+            ModelKind::Retirement => "retirement",
         }
     }
 
@@ -127,6 +131,7 @@ impl ModelKind {
             ModelKind::Farol => farol::schema(),
             ModelKind::Ants => ants::schema(),
             ModelKind::Thresholds => thresholds::schema(),
+            ModelKind::Retirement => retirement::schema(),
         }
     }
 }
@@ -159,6 +164,7 @@ pub enum ModelConfig {
     Farol(FarolConfig),
     Ants(AntsConfig),
     Thresholds(ThresholdsConfig),
+    Retirement(RetirementConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -183,6 +189,7 @@ enum Tagged<'a> {
     Farol(&'a FarolConfig),
     Ants(&'a AntsConfig),
     Thresholds(&'a ThresholdsConfig),
+    Retirement(&'a RetirementConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -214,6 +221,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Farol(c) => Tagged::Farol(c).serialize(s),
             ModelConfig::Ants(c) => Tagged::Ants(c).serialize(s),
             ModelConfig::Thresholds(c) => Tagged::Thresholds(c).serialize(s),
+            ModelConfig::Retirement(c) => Tagged::Retirement(c).serialize(s),
         }
     }
 }
@@ -240,6 +248,7 @@ impl ModelConfig {
             ModelConfig::Farol(_) => ModelKind::Farol,
             ModelConfig::Ants(_) => ModelKind::Ants,
             ModelConfig::Thresholds(_) => ModelKind::Thresholds,
+            ModelConfig::Retirement(_) => ModelKind::Retirement,
         }
     }
 
@@ -331,10 +340,13 @@ impl ModelConfig {
             "thresholds" => serde_json::from_value(value)
                 .map(ModelConfig::Thresholds)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "retirement" => serde_json::from_value(value)
+                .map(ModelConfig::Retirement)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants or thresholds)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds or retirement)"
                 ),
             )),
         }
@@ -361,6 +373,7 @@ impl ModelConfig {
             ModelConfig::Farol(c) => c.validate(),
             ModelConfig::Ants(c) => c.validate(),
             ModelConfig::Thresholds(c) => c.validate(),
+            ModelConfig::Retirement(c) => c.validate(),
         }
     }
 
@@ -387,6 +400,7 @@ impl ModelConfig {
             ModelConfig::Farol(c) => set_path(c, path, value).map(ModelConfig::Farol),
             ModelConfig::Ants(c) => set_path(c, path, value).map(ModelConfig::Ants),
             ModelConfig::Thresholds(c) => set_path(c, path, value).map(ModelConfig::Thresholds),
+            ModelConfig::Retirement(c) => set_path(c, path, value).map(ModelConfig::Retirement),
         }
     }
 
@@ -412,7 +426,8 @@ impl ModelConfig {
             | ModelConfig::Agreement(_)
             | ModelConfig::Farol(_)
             | ModelConfig::Ants(_)
-            | ModelConfig::Thresholds(_) => None,
+            | ModelConfig::Thresholds(_)
+            | ModelConfig::Retirement(_) => None,
         }
     }
 
@@ -440,6 +455,9 @@ impl ModelConfig {
             ModelConfig::Thresholds(_) => {
                 thresholds::SERIES.iter().map(|s| s.to_string()).collect()
             }
+            ModelConfig::Retirement(_) => {
+                retirement::SERIES.iter().map(|s| s.to_string()).collect()
+            }
         }
     }
 }
@@ -628,6 +646,7 @@ pub enum ModelWorld {
     Farol(Box<FarolWorld>),
     Ants(Box<AntsWorld>),
     Thresholds(Box<ThresholdsWorld>),
+    Retirement(Box<RetirementWorld>),
 }
 
 impl ModelWorld {
@@ -674,6 +693,9 @@ impl ModelWorld {
             ModelConfig::Thresholds(c) => {
                 ModelWorld::Thresholds(Box::new(ThresholdsWorld::new(c, seed)?))
             }
+            ModelConfig::Retirement(c) => {
+                ModelWorld::Retirement(Box::new(RetirementWorld::new(c, seed)?))
+            }
         })
     }
 
@@ -698,6 +720,7 @@ impl ModelWorld {
             ModelWorld::Farol(_) => ModelKind::Farol,
             ModelWorld::Ants(_) => ModelKind::Ants,
             ModelWorld::Thresholds(_) => ModelKind::Thresholds,
+            ModelWorld::Retirement(_) => ModelKind::Retirement,
         }
     }
 
@@ -722,6 +745,7 @@ impl ModelWorld {
             ModelWorld::Farol(w) => w.as_ref(),
             ModelWorld::Ants(w) => w.as_ref(),
             ModelWorld::Thresholds(w) => w.as_ref(),
+            ModelWorld::Retirement(w) => w.as_ref(),
         }
     }
 
@@ -746,6 +770,7 @@ impl ModelWorld {
             ModelWorld::Farol(w) => w.as_mut(),
             ModelWorld::Ants(w) => w.as_mut(),
             ModelWorld::Thresholds(w) => w.as_mut(),
+            ModelWorld::Retirement(w) => w.as_mut(),
         }
     }
 
@@ -839,6 +864,7 @@ impl ModelWorld {
             ModelWorld::Farol(w) => copy_without_history!(Farol, w),
             ModelWorld::Ants(w) => copy_without_history!(Ants, w),
             ModelWorld::Thresholds(w) => copy_without_history!(Thresholds, w),
+            ModelWorld::Retirement(w) => copy_without_history!(Retirement, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -877,6 +903,9 @@ impl ModelWorld {
             (ModelWorld::Thresholds(live), ModelWorld::Thresholds(kept)) => {
                 restore_into!(live, kept)
             }
+            (ModelWorld::Retirement(live), ModelWorld::Retirement(kept)) => {
+                restore_into!(live, kept)
+            }
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1253,6 +1282,30 @@ mod tests {
         assert_eq!(w.model().tick(), 0);
     }
 
+    #[test]
+    fn retirement_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "retirement", "per_cohort": 20, "renewal": "replace", "size": {"min": 5, "max": 9}}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Retirement);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(
+            (json["model"].as_str(), json["extent"].as_u64()),
+            (Some("retirement"), Some(5))
+        );
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["retired", "retired_a"]);
+        let e = ModelConfig::from_json(r#"{"model": "retirement", "norm": 0}"#).unwrap_err();
+        assert_eq!(e[0].field, "norm");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!(w.kind(), ModelKind::Retirement);
+        let cp = w.checkpoint().expect("retirement worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
     #[test]
     fn only_the_anasazi_finishes() {
         let mut w = ModelWorld::new(
@@ -1295,7 +1348,8 @@ mod tests {
                 "image",
                 "farol",
                 "ants",
-                "thresholds"
+                "thresholds",
+                "retirement"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
````

Modify `crates/sugarscape-core/src/presets.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index 7f7476b..29f9f6e 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -829,6 +829,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::farol::presets());
     out.extend(crate::ants::presets());
     out.extend(crate::thresholds::presets());
+    out.extend(crate::retirement::presets());
     out
 }
````

Modify `crates/sugarscape-core/src/titles.rs` — seven titles (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/titles.rs b/crates/sugarscape-core/src/titles.rs
index c1970c4..48a3e13 100644
--- a/crates/sugarscape-core/src/titles.rs
+++ b/crates/sugarscape-core/src/titles.rs
@@ -3,7 +3,7 @@
 //! paper (`source`) stay on the preset as its reference.
 
 /// Titles by preset id, in catalog order.
-pub const TITLES: [(&str, &str); 238] = [
+pub const TITLES: [(&str, &str); 245] = [
     (
         "ii-1-instant",
         "Sugar grows back at once: agents climb the best ridges and the poorly endowed starve",
@@ -917,6 +917,34 @@ pub const TITLES: [(&str, &str); 238] = [
         "watts-hub",
         "Light the best-connected node",
     ),
+    (
+        "ae-rapid",
+        "15 % decide rationally, and retiring at 65 sets in within a few years",
+    ),
+    (
+        "ae-base",
+        "A tenth decide rationally, and the norm takes a generation",
+    ),
+    (
+        "ae-slow",
+        "5 % rational: retiring at 65 spreads slowly, up from the old",
+    ),
+    (
+        "ae-policy",
+        "Congress lowers the age to 62: here the new norm comes in a few years",
+    ),
+    (
+        "ae-groups",
+        "Two communities, one with no rational agents, loosely linked",
+    ),
+    (
+        "ae-all-members",
+        "Count every friend, not just the eligible, and no norm ever forms",
+    ),
+    (
+        "ae-replace",
+        "Replace friends who die, and 5 % rationality is no longer enough",
+    ),
 ];
 
 /// The title of preset `id`, or "" if it has none.
````

- [ ] **Step 3: Run the model's tests**

Run: `cargo test -p sugarscape-core --lib retirement`
Expected: PASS (22).

- [ ] **Step 4: Watch the golden check fail, then record the entries**

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: FAIL with `record a golden fingerprint for ae-rapid (run print_golden)`.

Modify `crates/sugarscape-core/tests/golden.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 6780f6c..e5f8a50 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -254,6 +254,14 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("watts-upper", 0xea32457929c916d8),
     ("watts-hetero", 0x1ebbdf6d37320885),
     ("watts-hub", 0x994dd1ff0bedf61b),
+    // Milestone 26: the timing of retirement.
+    ("ae-rapid", 0x1d6b4cb389aa872f),
+    ("ae-base", 0x2d5c384cbc8ebd2f),
+    ("ae-slow", 0x63110dc295184e1b),
+    ("ae-policy", 0x96c030d4a1280cd4),
+    ("ae-groups", 0xadde267c611d5392),
+    ("ae-all-members", 0x8f6694a3282613e3),
+    ("ae-replace", 0x90d96b846be612f2),
 ];
 
 fn fingerprint(id: &str) -> u64 {
````

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: PASS.

- [ ] **Step 5: Format, lint, run everything, commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace
git add crates/sugarscape-core/src/retirement crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs crates/sugarscape-core/tests/golden.rs
```
```bash
git commit -m "Add The Timing of Retirement (Axtell & Epstein) as a model kind

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/{ae-rational,ae-rational-replace,ae-threshold,ae-size,ae-extent,ae-policy,ae-coupling,ae-coupling-rational}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets and series (`transition`, `transition_new`, `transition_a`, `transition_b`).
- Produces: eight built-in sweeps; the CLI's stops `(its last period)` and `(the norm set in)`.

- [ ] **Step 1: Write the failing tests**

Modify `crates/sugarscape-cli/tests/cli.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index edf09c7..e9396dc 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -144,6 +144,14 @@ fn presets_and_sweeps_are_listed() {
         "watts-window",
         "watts-hetero",
         "watts-targeting",
+        "ae-rational",
+        "ae-rational-replace",
+        "ae-threshold",
+        "ae-size",
+        "ae-extent",
+        "ae-policy",
+        "ae-coupling",
+        "ae-coupling-rational",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -567,6 +575,38 @@ fn a_thresholds_run_stops_at_its_last_step() {
     assert_eq!(stderr(&out), "finished at tick 25 (its last step)\n");
 }
 
+#[test]
+fn a_retirement_run_stops_at_the_norm_or_its_last_period() {
+    let dir = scratch("retirement");
+    let config = dir.join("norm.json");
+    std::fs::write(
+        &config,
+        r#"{"model": "retirement", "per_cohort": 20, "rational": 0.3, "stop_at_norm": true}"#,
+    )
+    .unwrap();
+    let out = sugarscape(&[
+        "run",
+        "--config",
+        config.to_str().unwrap(),
+        "--ticks",
+        "100",
+    ]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    assert!(
+        stderr(&out).ends_with("(the norm set in)\n"),
+        "{}",
+        stderr(&out)
+    );
+    let stop = dir.join("stop.json");
+    std::fs::write(
+        &stop,
+        r#"{"model": "retirement", "per_cohort": 20, "stop_at": 25}"#,
+    )
+    .unwrap();
+    let out = sugarscape(&["run", "--config", stop.to_str().unwrap(), "--ticks", "100"]);
+    assert_eq!(stderr(&out), "finished at tick 25 (its last period)\n");
+}
+
 #[test]
 fn a_social_structure_run_stops_at_its_last_period() {
     let out = sugarscape(&["run", "--preset", "cra-rwr", "--ticks", "3000"]);
````

Modify `crates/sugarscape-wasm/tests/web.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index 09f416c..140af2a 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -347,7 +347,15 @@ fn builtins_and_series_names_are_listed() {
             "gr-ceilings",
             "watts-window",
             "watts-hetero",
-            "watts-targeting"
+            "watts-targeting",
+            "ae-rational",
+            "ae-rational-replace",
+            "ae-threshold",
+            "ae-size",
+            "ae-extent",
+            "ae-policy",
+            "ae-coupling",
+            "ae-coupling-rational"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -945,6 +953,23 @@ fn thresholds_sims_match_the_native_golden_entries() {
     }
 }
 
+#[wasm_bindgen_test]
+fn retirement_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the base case,
+    // the policy switch, two groups, and friends replaced.
+    for (id, fp) in [
+        ("ae-base", "0x2d5c384cbc8ebd2f"),
+        ("ae-policy", "0x96c030d4a1280cd4"),
+        ("ae-groups", "0xadde267c611d5392"),
+        ("ae-replace", "0x90d96b846be612f2"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "retirement");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
 #[wasm_bindgen_test]
 fn dpd_sims_match_the_native_golden_entries() {
     // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: wealth sums and
````

Run: `cargo test -p sugarscape-cli`
Expected: FAIL (`a_retirement_run_stops_at_the_norm_or_its_last_period`; `presets_and_sweeps_are_listed`: `ae-rational` missing).

- [ ] **Step 2: Write the sweeps, register them and name the stops**

Create `sweeps/ae-rational.json` with exactly this content:

````json
{
  "name": "Retirement: transition time against rationality",
  "description": "Axtell and Epstein's Figure 6-6: the period by which 95 % of eligible agents have retired, against the share of rationals, for 0, 5 and 10 % randoms (blank: never within 300 periods). Measured (release, seeds 1–10, recorded 2026-09-28): no randoms: never at 0, then 79, 72, 54, 14, 7, 5 at 2, 5, 10, 15, 20, 25 %; 5 % random: 72, 69, 61, 16, 8, 5, 4; 10 % random: 62, 43, 18, 8, 6, 4, 3. Fewer rationals, slower; more randoms, faster — as stated. But with 5 % randoms no minimum of rationals is needed, and nothing takes the paper's hundreds to thousands of periods: slow runs finish once the initial population has died out. See ae-rational-replace.",
  "base": {
    "preset": "ae-base"
  },
  "set": {
    "stop_at_norm": true
  },
  "x": {
    "label": "Rational share",
    "path": "rational",
    "values": [
      0,
      0.02,
      0.05,
      0.1,
      0.15,
      0.2,
      0.25
    ]
  },
  "series": {
    "label": "Random share",
    "values": [
      {
        "at": 0,
        "name": "No randoms",
        "set": {
          "random": 0
        }
      },
      {
        "at": 0.05,
        "name": "5 % random",
        "set": {
          "random": 0.05
        }
      },
      {
        "at": 0.1,
        "name": "10 % random",
        "set": {
          "random": 0.1
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 300,
  "metric": {
    "kind": "final",
    "series": "transition"
  }
}
````

Create `sweeps/ae-rational-replace.json` with exactly this content:

````json
{
  "name": "Retirement: rationality when friends who die are replaced",
  "description": "Figure 6-6 again, with friends who die replaced by agents of about the same age instead of by the newborn in their slot (C 50). Measured (release, seeds 1–5, recorded 2026-09-28): no randoms: never at 0, 5 and 10 %, 20 at 15 %, 13 at 20 %; 5 % random: never at 0 and 5 %, 103 at 10 % (± 75 in the planning measurements), 15, 9. The paper's 'certain minimum proportions of the population must be rational' and its long, highly variable times appear only under this unstated rule.",
  "base": {
    "preset": "ae-replace"
  },
  "set": {
    "stop_at_norm": true,
    "per_cohort": 50
  },
  "x": {
    "label": "Rational share",
    "path": "rational",
    "values": [
      0,
      0.05,
      0.1,
      0.15,
      0.2
    ]
  },
  "series": {
    "label": "Random share",
    "values": [
      {
        "at": 0,
        "name": "No randoms",
        "set": {
          "random": 0
        }
      },
      {
        "at": 0.05,
        "name": "5 % random",
        "set": {
          "random": 0.05
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 5
  },
  "ticks": 600,
  "metric": {
    "kind": "final",
    "series": "transition"
  }
}
````

Create `sweeps/ae-threshold.json` with exactly this content:

````json
{
  "name": "Retirement: spread of imitation thresholds",
  "description": "Figure 6-7: transition time against the spread of imitation thresholds around 0.5 (uniformly spread, as all the paper's random variables are). Measured (release, seeds 1–10, recorded 2026-09-28): 16, 34, 20, 12, 8, 8 at spreads 0, 0.05, 0.1, 0.15, 0.2, 0.25. More spread is faster overall, as stated — but a little spread first slows the transition to twice the uniform threshold's.",
  "base": {
    "preset": "ae-base"
  },
  "set": {
    "stop_at_norm": true
  },
  "x": {
    "label": "Threshold spread (sd)",
    "path": "spread",
    "values": [
      0,
      0.05,
      0.1,
      0.15,
      0.2,
      0.25
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 300,
  "metric": {
    "kind": "final",
    "series": "transition"
  }
}
````

Create `sweeps/ae-size.json` with exactly this content:

````json
{
  "name": "Retirement: network size",
  "description": "Figure 6-8c: transition time against the largest network, sizes drawn from U[10, max]. Measured (release, seeds 1–10, recorded 2026-09-28): 8, 13, 23, 39, 60, 69 at 10, 20, 30, 40, 60, 80 — rising steeply, as stated ('in large networks it is difficult for a new norm to establish itself'), then leveling as runs finish when the initial population has died out.",
  "base": {
    "preset": "ae-base"
  },
  "set": {
    "stop_at_norm": true
  },
  "x": {
    "label": "Largest network (S ~ U[10, max])",
    "path": "size.max",
    "values": [
      10,
      20,
      30,
      40,
      60,
      80
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 300,
  "metric": {
    "kind": "final",
    "series": "transition"
  }
}
````

Create `sweeps/ae-extent.json` with exactly this content:

````json
{
  "name": "Retirement: network extent across cohorts",
  "description": "Figure 6-9: transition time against how many cohorts a network spans either way, at 10 % and 5 % rational. Measured (release, seeds 1–10, recorded 2026-09-28): 10 %: 20, 20, 16, 16, 16, 12 at extents 1, 2, 3, 5, 7, 10; 5 %: 49, 59, 64, 61, 58, 49. Wider networks are faster, as stated, at 10 % and from extent 3 at 5 %; at 5 % the narrowest networks are fast too.",
  "base": {
    "preset": "ae-base"
  },
  "set": {
    "stop_at_norm": true
  },
  "x": {
    "label": "Extent (cohorts)",
    "path": "extent",
    "values": [
      1,
      2,
      3,
      5,
      7,
      10
    ]
  },
  "series": {
    "label": "Rational share",
    "values": [
      {
        "at": 0.1,
        "name": "10 % rational",
        "set": {
          "rational": 0.1
        }
      },
      {
        "at": 0.05,
        "name": "5 % rational",
        "set": {
          "rational": 0.05
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 300,
  "metric": {
    "kind": "final",
    "series": "transition"
  }
}
````

Create `sweeps/ae-policy.json` with exactly this content:

````json
{
  "name": "Retirement: from 65 to 62",
  "description": "Figure 6-10: with retirement mandatory at 70, the periods from lowering eligibility to 62 (once the 65 norm is established) until 95 % of those 62 and older have retired, against the share of rationals. The paper: 'about 35 periods if between 1 and 4 percent of the population responds rationally'. Measured (release, seeds 1–10, recorded 2026-09-28): 2.0 periods at 0, 1, 2, 4, 8 %, 1.8 at 14 %. The retired 65-to-67-year-olds in every 62-year-old's network tip the imitators at once.",
  "base": {
    "preset": "ae-policy"
  },
  "set": {
    "stop_at_norm": true
  },
  "x": {
    "label": "Rational share",
    "path": "rational",
    "values": [
      0,
      0.01,
      0.02,
      0.04,
      0.08,
      0.14
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 200,
  "metric": {
    "kind": "final",
    "series": "transition_new"
  }
}
````

Create `sweeps/ae-coupling.json` with exactly this content:

````json
{
  "name": "Retirement: the group without rationals, against coupling",
  "description": "Figure 6-11: the period the group without rationals reaches the norm, against the share of each network drawn from the other group. Measured (release, seeds 1–10, recorded 2026-09-28): 75, 63, 46, 46, 58, 59 at couplings 0, 0.05, 0.1, 0.15, 0.2, 0.25. A little coupling pulls it in, as stated; more does not pull it further — see ae-coupling-rational.",
  "base": {
    "preset": "ae-groups"
  },
  "x": {
    "label": "Coupling",
    "path": "groups.coupling",
    "values": [
      0,
      0.05,
      0.1,
      0.15,
      0.2,
      0.25
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 300,
  "metric": {
    "kind": "final",
    "series": "transition_a"
  }
}
````

Create `sweeps/ae-coupling-rational.json` with exactly this content:

````json
{
  "name": "Retirement: the group with rationals, against coupling",
  "description": "Figure 6-11's other group: the period the group with 10 % rationals reaches the norm, against coupling. Measured (release, seeds 1–10, recorded 2026-09-28): 19, 27, 34, 44, 57, 59 at couplings 0 to 0.25. The coupling that pulls the other group in slows this one just as much; by 0.2 both take about 58 periods. The paper's figure keeps this group fast.",
  "base": {
    "preset": "ae-groups"
  },
  "x": {
    "label": "Coupling",
    "path": "groups.coupling",
    "values": [
      0,
      0.05,
      0.1,
      0.15,
      0.2,
      0.25
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 300,
  "metric": {
    "kind": "final",
    "series": "transition_b"
  }
}
````

Modify `crates/sugarscape-core/src/sweep.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index 033d288..f464c62 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -965,7 +965,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 97] = [
+const BUILTINS: [Builtin; 105] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1354,6 +1354,38 @@ const BUILTINS: [Builtin; 97] = [
         id: "watts-targeting",
         json: include_str!("../../../sweeps/watts-targeting.json"),
     },
+    Builtin {
+        id: "ae-rational",
+        json: include_str!("../../../sweeps/ae-rational.json"),
+    },
+    Builtin {
+        id: "ae-rational-replace",
+        json: include_str!("../../../sweeps/ae-rational-replace.json"),
+    },
+    Builtin {
+        id: "ae-threshold",
+        json: include_str!("../../../sweeps/ae-threshold.json"),
+    },
+    Builtin {
+        id: "ae-size",
+        json: include_str!("../../../sweeps/ae-size.json"),
+    },
+    Builtin {
+        id: "ae-extent",
+        json: include_str!("../../../sweeps/ae-extent.json"),
+    },
+    Builtin {
+        id: "ae-policy",
+        json: include_str!("../../../sweeps/ae-policy.json"),
+    },
+    Builtin {
+        id: "ae-coupling",
+        json: include_str!("../../../sweeps/ae-coupling.json"),
+    },
+    Builtin {
+        id: "ae-coupling-rational",
+        json: include_str!("../../../sweeps/ae-coupling-rational.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2273,7 +2305,15 @@ mod tests {
                 "gr-ceilings",
                 "watts-window",
                 "watts-hetero",
-                "watts-targeting"
+                "watts-targeting",
+                "ae-rational",
+                "ae-rational-replace",
+                "ae-threshold",
+                "ae-size",
+                "ae-extent",
+                "ae-policy",
+                "ae-coupling",
+                "ae-coupling-rational"
             ]
         );
         for b in builtins() {
````

Modify `crates/sugarscape-cli/src/main.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index 797e799..dd5e0a1 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -236,6 +236,10 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             ModelKind::Image => "its last generation",
             ModelKind::Farol => "its last round",
             ModelKind::Ants | ModelKind::Thresholds => "its last step",
+            ModelKind::Retirement => match &config {
+                ModelConfig::Retirement(c) if c.stop_at_norm => "the norm set in",
+                _ => "its last period",
+            },
             _ => "its end year",
         };
         eprintln!("finished at tick {} ({why})", world.tick());
````

- [ ] **Step 3: Run the tests and measure the sweeps**

Run: `cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm`, then `cargo build --release -p sugarscape-cli` and each `./target/release/sugarscape sweep --builtin <id> --quiet --summary-csv /tmp/<id>.csv --out /dev/null`.
Expected: PASS (WASM 49); each summary's means as its description records (about 17 s in all; a blank mean is a point where no run reached the norm).

- [ ] **Step 4: Format, lint, commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add sweeps/ae-rational.json sweeps/ae-rational-replace.json sweeps/ae-threshold.json sweeps/ae-size.json sweeps/ae-extent.json sweeps/ae-policy.json sweeps/ae-coupling.json sweeps/ae-coupling-rational.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli crates/sugarscape-wasm/tests/web.rs
```
```bash
git commit -m "Measure The Timing of Retirement: eight sweeps, the CLI's stops and WASM agreement

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/compare-presets.test.ts`, `web/src/engine.test.ts`, `web/src/experiments/form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the WASM build of Tasks 1–2.
- Produces: `RetirementConfig`, `RetirementStats`, `RetireeView`, `RetirementInspection` (types.ts); `isRetirementView` (checked first: it tests `exposed`); `hasRetirementGroups`; `MODEL_CHARTS.retirement`; the Compare entry `ae-rapid-vs-slow`.

- [ ] **Step 1: Write the failing tests**

Modify `web/src/models.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index 326ce04..d924fa8 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -6,6 +6,7 @@ import {
   isAgreementView,
   isAntsView,
   isThresholdsView,
+  isRetirementView,
   isFarolView,
   isCivilView,
   isClassesView,
@@ -119,6 +120,29 @@ describe('the presets menu', () => {
   });
 });
 
+describe('the retirement model', () => {
+  it('is read by its tag, and its inspections by `exposed`, before the others with a panel', () => {
+    const c = { model: 'retirement', stop_at: 0, stop_at_norm: false } as unknown as ModelConfig;
+    expect(modelOf(c)).toBe('retirement');
+    const cell = { site: { x: 1, y: 2 }, panel: 'population', age: 65, retirements: null, exposed: null, period: null, retired: null, member: null, agent: null } as unknown as AnyInspection;
+    const th = { site: { x: 1, y: 2 }, panel: 'actors', step: null, crowds: null, share: null, cdf: null, count: null, member: null, agent: null } as unknown as AnyInspection;
+    expect([cell, th].map(isRetirementView)).toEqual([true, false]);
+    expect(isThresholdsView(cell)).toBe(false);
+  });
+
+  it('colors four ways, has no overlays, and stops unpredictably only at the norm', () => {
+    expect(COLOR_MODES.retirement).toEqual([
+      ['status', 'Status'],
+      ['type', 'Type'],
+      ['threshold', 'Threshold'],
+      ['group', 'Group'],
+    ]);
+    expect(MODEL_OVERLAYS.retirement).toEqual([]);
+    const c = (stop_at: number, stop_at_norm: boolean) => ({ model: 'retirement', stop_at, stop_at_norm }) as unknown as ModelConfig;
+    expect([finishesUnpredictably(c(100, false)), finishesUnpredictably(c(0, true)), ticksLeft(c(100, false), 40), ticksLeft(c(0, false), 40)]).toEqual([false, true, 60, Infinity]);
+  });
+});
+
 describe('the thresholds model', () => {
   it('is read by its tag, and its inspections by their cdf, before the ants’ and El Farol’s', () => {
     const c = { model: 'thresholds', stop_at: 0 } as unknown as ModelConfig;
````

Modify `web/src/compare-presets.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index e82f5ff..c438ab9 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -43,6 +43,11 @@ describe('compare presets', () => {
     expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
   });
 
+  it('pairs 15 % and 5 % rational retirees', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['ae-rapid-vs-slow', 'ae-rapid', 'ae-slow', '15 % vs 5 % rational — Retirement (Compare)']);
+  });
+
   it('pairs Granovetter’s uniform and perturbed crowds', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['gr-uniform-vs-perturbed', 'gr-uniform', 'gr-perturbed', 'Uniform vs perturbed crowd — Threshold Models (Compare)']);
````

Modify `web/src/engine.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index ffaba8a..fe035c3 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1154,6 +1154,8 @@ describe('Engine with other models', () => {
     expect(finishedNotice({ model: 'farol', stop_at: 100 } as unknown as ModelConfig, 100)).toBe('This run has reached its last round (100) — Reset to run it again');
     expect(finishedNotice({ model: 'ants', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last step (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'thresholds', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last step (50) — Reset to run it again');
+    expect(finishedNotice({ model: 'retirement', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last period (50) — Reset to run it again');
+    expect(finishedNotice({ model: 'retirement', stop_at_norm: true } as unknown as ModelConfig, 16)).toBe('The retirement norm has set in at t = 16 — Reset to run it again');
     expect(finishedNotice({ model: 'agreement', stop_at: 200 } as unknown as ModelConfig, 200)).toBe('This run has reached its last period (200) — Reset to run it again');
     expect(finishedNotice({ model: 'agreement', stop_at: 20000 } as unknown as ModelConfig, 376)).toBe(
       'Stable at t = 376: no opinion or uncertainty moves any more — Reset, or change the rule, to run it again',
````

Modify `web/src/experiments/form.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index 43fea0b..b5a42be 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -147,6 +147,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('retirement')).toMatchObject({
+      x: { path: 'rational', values: '0.02,0.05,0.1,0.15,0.2,0.25' },
+      ticks: 400,
+      metric: { kind: 'final', series: 'transition' },
+    });
     expect(defaultForm('thresholds')).toMatchObject({
       x: { path: 'sd', values: '0.1:0.2:0.01' },
       ticks: 200,
````

Modify `web/src/ui/series-data.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index 1afd74a..9ba46f5 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,17 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('retirement charts', () => {
+  it('chart the retired share (by group when there are groups), retirement ages and the transition over periods', () => {
+    expect(MODEL_CHARTS.retirement.map((c) => c.title)).toEqual(['Retired share', 'Retired share', 'Retirement age', 'Transition', 'Group transitions']);
+    const one = { model: 'retirement', groups: { enabled: false } } as unknown as ModelConfig;
+    const two = { model: 'retirement', groups: { enabled: true } } as unknown as ModelConfig;
+    const [byGroup, single] = MODEL_CHARTS.retirement;
+    expect([byGroup.shown!(one), byGroup.shown!(two), single.shown!(one), single.shown!(two)]).toEqual([false, true, true, false]);
+    expect(timeAxisLabel('retirement')).toBe('Periods');
+  });
+});
+
 describe('thresholds charts', () => {
   it('chart participation against theory, episodes, the last cascade and the swing over steps', () => {
     expect(MODEL_CHARTS.thresholds.map((c) => c.title)).toEqual(['Participation', 'Episodes', 'Last cascade', 'Swing']);
````

Modify `web/src/determinism.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index f1d5a4c..1b2167c 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -13,6 +13,9 @@ import { InlineTransport } from './transport';
 import { decodeShare, encodeShare } from './share';
 import type {
   AgreementConfig,
+  RetirementConfig,
+  RetirementInspection,
+  RetirementStats,
   ThresholdsConfig,
   ThresholdsInspection,
   ThresholdsStats,
@@ -724,6 +727,28 @@ describe('the social-structure model through the engine', () => {
   });
 });
 
+describe('the retirement model through the engine', () => {
+  it('stops at the norm and inspects an agent, an age and a period', async () => {
+    const r = presets.find((p) => p.id === 'ae-rapid')!;
+    const config = { ...structuredClone(r.config as RetirementConfig), stop_at_norm: true };
+    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'type' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as RetirementStats;
+    expect([e.finished, ends, s.transition]).toEqual([true, 1, e.tick]);
+    expect(s.retired).toBeGreaterThanOrEqual(0.95);
+    // Row 45 (2 pixels an age) is age 65.
+    await e.select(0, 90);
+    const v = e.inspection!.view as RetirementInspection;
+    expect([v.panel, v.age]).toEqual(['population', 65]);
+    expect(e.inspection!.agentId).toBeNull();
+    await e.select(410, 90);
+    expect((e.inspection!.view as RetirementInspection).panel).toBe('ages');
+  });
+});
+
 describe('the thresholds model through the engine', () => {
   it('stops at its last step and inspects an actor, a step and Figure 1', async () => {
     const r = presets.find((p) => p.id === 'gr-uniform')!;
````

- [ ] **Step 2: Run them to see them fail**

Run: `cd web && npm ci && npm run wasm && npx vitest run`
Expected: failures in the six files above.

- [ ] **Step 3: Carry the model through the page**

Modify `web/src/types.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/types.ts b/web/src/types.ts
index c88cac5..c5fa564 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -95,7 +95,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -494,7 +494,7 @@ export interface AgreementConfig {
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig;
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig;
 
 /**
  * Arthur's El Farol bar and Challet and Zhang's minority game (milestone 23), with Challet, Marsili
@@ -676,6 +676,79 @@ export interface ThresholdsInspection {
   agent: null;
 }
 
+/**
+ * Axtell and Epstein's timing of retirement (milestone 26): cohorts, rationals, randoms and
+ * imitators in transient social networks, the policy switch and two coupled sub-populations.
+ */
+export interface RetirementConfig {
+  model: 'retirement';
+  per_cohort: number;
+  rational: number;
+  random: number;
+  p: number;
+  threshold: number;
+  spread: number;
+  size: { min: number; max: number };
+  extent: number;
+  counts: 'eligible' | 'all';
+  renewal: 'slot' | 'replace';
+  order: 'by_cohort' | 'shuffled';
+  initial_deaths: 'literal' | 'survivors';
+  eligibility: number;
+  mandatory: number;
+  policy: { enabled: boolean; to: number };
+  groups: { enabled: boolean; coupling: number };
+  norm: number;
+  stop_at_norm: boolean;
+  stop_at: number;
+}
+
+export interface RetirementStats {
+  tick: number;
+  retired: number;
+  retired_a: number;
+  retired_b: number;
+  /** The period the norm set in, and periods from the policy switch to the new norm (null before). */
+  transition: number | null;
+  transition_new: number | null;
+  /** The period each group reached the norm (both `transition` without groups). */
+  transition_a: number | null;
+  transition_b: number | null;
+  modal_age: number | null;
+  mean_age: number | null;
+  rational_share: number;
+  eligibility: number;
+}
+
+export interface RetireeView {
+  id: number;
+  age: number;
+  kind: 'rational' | 'random' | 'imitator';
+  threshold: number;
+  death_age: number;
+  group: number;
+  network: number;
+  eligible: number;
+  retired_members: number;
+  retired: boolean;
+  retired_at: number | null;
+}
+/**
+ * A cell of the retirement frame: an agent of the population, an age's retirement bar, or a period
+ * of the time panel. `agent` is always null.
+ */
+export interface RetirementInspection {
+  site: { x: number; y: number };
+  panel: 'population' | 'ages' | 'time' | null;
+  age: number | null;
+  retirements: number | null;
+  exposed: number | null;
+  period: number | null;
+  retired: number | null;
+  member: RetireeView | null;
+  agent: null;
+}
+
 /** A preset: `title` is the menu's plain headline; `source` and `name` are its figure or paper and its rules. */
 export interface Preset { id: string; title: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -985,7 +1058,7 @@ export interface AgreementStats {
   stable_at: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats;
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -1310,7 +1383,7 @@ export interface AgreementInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection;
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -1368,7 +1441,10 @@ export type ColorMode =
   | 'degree'
   | 'state'
   | 'threshold'
-  | 'crowd';
+  | 'crowd'
+  | 'status'
+  | 'type'
+  | 'group';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
````

Modify `web/src/models.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.ts b/web/src/models.ts
index dce7e84..3876917 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -1,6 +1,8 @@
 // Which model a config is (milestones 9–21), and what each model offers the page.
 import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
 import type {
+  RetirementConfig,
+  RetirementInspection,
   ThresholdsConfig,
   ThresholdsInspection,
   AntsConfig,
@@ -41,7 +43,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -64,12 +66,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   farol: 'El Farol and the Minority Game',
   ants: 'Ants and Recruitment',
   thresholds: 'Threshold Models',
+  retirement: 'The Timing of Retirement',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement'
     ? tag
     : 'sugarscape';
 }
@@ -163,6 +166,11 @@ export function isImageView(v: AnyInspection): v is ImageInspection {
   return 'cell' in v && 'group' in v;
 }
 
+/** A cell of the retirement frame (a panel, an agent as `member`, and an age's `exposed`); check it first. */
+export function isRetirementView(v: AnyInspection): v is RetirementInspection {
+  return 'panel' in v && 'exposed' in v;
+}
+
 /** A cell of the thresholds frame (a panel, an actor as `member`, and Figure 1's `cdf`); check it first. */
 export function isThresholdsView(v: AnyInspection): v is ThresholdsInspection {
   return 'panel' in v && 'cdf' in v;
@@ -201,6 +209,7 @@ export function ticksLeft(c: ModelConfig, tick: number): number {
   if (modelOf(c) === 'farol' && (c as FarolConfig).stop_at > 0) return Math.max(0, (c as FarolConfig).stop_at - tick);
   if (modelOf(c) === 'ants' && (c as AntsConfig).stop_at > 0) return Math.max(0, (c as AntsConfig).stop_at - tick);
   if (modelOf(c) === 'thresholds' && (c as ThresholdsConfig).stop_at > 0) return Math.max(0, (c as ThresholdsConfig).stop_at - tick);
+  if (modelOf(c) === 'retirement' && (c as RetirementConfig).stop_at > 0) return Math.max(0, (c as RetirementConfig).stop_at - tick);
   return Infinity;
 }
 
@@ -215,6 +224,7 @@ export function finishesUnpredictably(c: ModelConfig): boolean {
   if (model === 'classes') return (c as ClassesConfig).stop_at_equity;
   if (model === 'opinions') return (c as OpinionsConfig).stop_when_stable;
   if (model === 'agreement') return (c as AgreementConfig).stop_when_stable;
+  if (model === 'retirement') return (c as RetirementConfig).stop_at_norm;
   if (model === 'sugarscape') return (c as Config).culture.rule === 'axelrod' && (c as Config).culture.stop_when_settled === true;
   return model === 'civil' && (c as CivilConfig).variant === 'ethnic' && (c as CivilConfig).stop_at_extinction;
 }
@@ -360,6 +370,13 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['degree', 'Degree'],
     ['crowd', 'Crowd'],
   ],
+  // Working by type, or retired; each agent's type; its threshold; its sub-population.
+  retirement: [
+    ['status', 'Status'],
+    ['type', 'Type'],
+    ['threshold', 'Threshold'],
+    ['group', 'Group'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -383,4 +400,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   farol: [],
   ants: [],
   thresholds: [],
+  retirement: [],
 };
````

Modify `web/src/engine.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.ts b/web/src/engine.ts
index 9df4501..7ed663b 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -54,6 +54,10 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
   if (modelOf(config) === 'farol') return `This run has reached its last round (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'ants' || modelOf(config) === 'thresholds') return `This run has reached its last step (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'retirement')
+    return (config as { stop_at_norm?: boolean }).stop_at_norm
+      ? `The retirement norm has set in at t = ${tick} — Reset to run it again`
+      : `This run has reached its last period (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'tags' || modelOf(config) === 'image') return `This run has reached its last generation (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'classes') return `Equity reached at t = ${tick}: every agent remembers mostly M — Reset to run it again`;
   if (modelOf(config) === 'opinions')
````

Modify `web/src/compare-presets.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index 7c1b5bc..d278148 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -170,6 +170,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'gr-uniform',
     b: 'gr-perturbed',
   },
+  {
+    id: 'ae-rapid-vs-slow',
+    label: '15 % vs 5 % rational — Retirement (Compare)',
+    a: 'ae-rapid',
+    b: 'ae-slow',
+  },
 ];
 
 /**
````

Modify `web/src/experiments/form.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index 3cacc54..2374930 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -104,6 +104,15 @@ export function defaultForm(model: ModelKind = 'sugarscape', config?: ModelConfi
     // The built-in ef-predictors' axis: how far attendance swings against predictors per agent.
     return { ...form, x: { path: 'strategies', values: '2:24:2' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'fluctuation' } };
   }
+  if (model === 'retirement') {
+    // The built-in ae-rational's axis: the period the age 65 norm sets in (kept once reached) against the rational share.
+    return {
+      ...form,
+      x: { path: 'rational', values: '0.02,0.05,0.1,0.15,0.2,0.25' },
+      ticks: 400,
+      metric: { ...form.metric, kind: 'final', series: 'transition' },
+    };
+  }
   if (model === 'thresholds') {
     // The built-in gr-sd's axis: the share rioting at equilibrium against the spread of thresholds.
     return { ...form, x: { path: 'sd', values: '0.1:0.2:0.01' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'acting' } };
````

Modify `web/src/ui/series-data.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index ce9cb89..277a209 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -142,6 +142,9 @@ const hasK = (c: ModelConfig): boolean => 'strategies' in c && (c.strategies as
  * image scoring's help rate and cooperative strategies, mean k, strategy shares (the binary scorers
  * apart) and mean payoff.
  */
+/** A retirement config with two sub-populations (its by-group lines show). */
+export const hasRetirementGroups = (c: ModelConfig): boolean => (c as { groups?: { enabled?: unknown } }).groups?.enabled === true && (c as { model?: unknown }).model === 'retirement';
+
 /** An El Farol config playing Arthur's game (its forecasts chart shows). */
 export const isElFarol = (c: ModelConfig): boolean => 'game' in c && (c as { game: unknown }).game === 'el_farol';
 
@@ -588,6 +591,42 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
     { title: 'Last cascade', lines: [{ key: 'last_size', label: 'Final share of the last episode', color: '--c3' }], range: [0, 1] },
     { title: 'Swing', lines: [{ key: 'swing', label: 'Range over the last 100 steps', color: '--c4' }], range: [0, 1] },
   ],
+  retirement: [
+    {
+      title: 'Retired share',
+      lines: [
+        { key: 'retired', label: 'Of those eligible', color: '--red' },
+        { key: 'retired_a', label: 'First group', color: '--c2' },
+        { key: 'retired_b', label: 'Second group', color: '--c3' },
+      ],
+      range: [0, 1],
+      shown: hasRetirementGroups,
+    },
+    { title: 'Retired share', lines: [{ key: 'retired', label: 'Of those eligible', color: '--red' }], range: [0, 1], shown: (c) => !hasRetirementGroups(c) },
+    {
+      title: 'Retirement age',
+      lines: [
+        { key: 'modal_age', label: 'Most common, last 10 periods', color: '--c1' },
+        { key: 'mean_age', label: 'Mean, last 10 periods', color: '--c4' },
+        { key: 'eligibility', label: 'Eligibility', color: '--c2' },
+      ],
+    },
+    {
+      title: 'Transition',
+      lines: [
+        { key: 'transition', label: 'The period the norm set in', color: '--c1' },
+        { key: 'transition_new', label: 'Periods from the switch to the new norm', color: '--red' },
+      ],
+    },
+    {
+      title: 'Group transitions',
+      lines: [
+        { key: 'transition_a', label: 'First group (no rationals)', color: '--c2' },
+        { key: 'transition_b', label: 'Second group', color: '--c3' },
+      ],
+      shown: hasRetirementGroups,
+    },
+  ],
 };
 
 /**
@@ -595,7 +634,7 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
  * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
+  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' ? 'Periods' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
````

Modify `web/src/ui/inspect-panel.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index 3a6b4e2..c907cdf 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -3,11 +3,12 @@ import { dpdRows } from '../dpd';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
 import { imageRows } from '../image-scoring';
-import { isAgreementView, isAntsView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isAgreementView, isAntsView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
   AntsInspection,
+  RetirementInspection,
   ThresholdsInspection,
   FarolInspection,
   AgreementInspection,
@@ -271,6 +272,27 @@ export class InspectPanel {
     return rows;
   }
 
+  /** An agent of the retirement population, an age's retirements, or a period. */
+  private retirementRows(view: RetirementInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    const pct = (x: number) => `${fmt(100 * x)} %`;
+    if (view.panel === 'time') return [row('Period', String(view.period)), row('Eligible retired', pct(view.retired ?? 0))];
+    if (view.panel === 'ages') {
+      const e = view.exposed ?? 0;
+      return [row('Age', String(view.age)), row('Retired, last 10 periods', `${view.retirements} of ${e}${e > 0 ? ` (${pct((view.retirements ?? 0) / e)})` : ''}`)];
+    }
+    const a = view.member;
+    if (!a) return [row('Age', view.age === null ? '—' : String(view.age)), row('Point', 'an empty place')];
+    const rows = [
+      row('Agent', `#${a.id} · ${a.kind}${a.group > 0 ? ' · second group' : ''}`),
+      row('Age', `${a.age} (dies at ${fmt(a.death_age)})`),
+      row('Status', a.retired ? `retired${a.retired_at !== null ? ` at ${a.retired_at}` : ''}` : 'working'),
+      row('Network', `${a.network} · ${a.eligible} eligible · ${a.retired_members} retired`),
+    ];
+    if (a.kind === 'imitator') rows.push(row('Threshold', pct(a.threshold)));
+    return rows;
+  }
+
   /** A step of the thresholds' time panel, a point of Figure 1, a histogram row, or an actor. */
   private thresholdsRows(view: ThresholdsInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -487,6 +509,8 @@ export class InspectPanel {
             ? this.normsRows(view)
           : isAgreementView(view)
             ? this.agreementRows(view)
+          : isRetirementView(view)
+            ? this.retirementRows(view)
           : isThresholdsView(view)
             ? this.thresholdsRows(view)
           : isAntsView(view)
````

- [ ] **Step 4: Run the page's build and tests**

Run: `cd web && npm run build && npm test`
Expected: the build succeeds; 713 tests pass (47 files).

- [ ] **Step 5: Commit**

```bash
git add web/src
```
```bash
git commit -m "Carry The Timing of Retirement through the page

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

- [ ] **Step 6 (controller): check it in the browser**

`cd web && npm run build && npx vite preview`, then with `?debug`: `ae-rapid` fills with red from the old rows up within about 8 periods (rows are ages, 20 at the top); `ae-slow` wavers for dozens of periods; `ae-policy` marks the switch in the time panel and its ages bars move up to 62; `ae-groups` in Group colors, with the Group transitions chart; `ae-base` in Type colors. Inspect an agent (age, type, network eligible and retired), an ages bar and a period. Rules panel groups as listed. Compare "15 % vs 5 % rational — Retirement (Compare)". Experiments with a retirement preset: the default axis rational 0.02–0.25 against transition; run the built-in `ae-rational`.

---

### Task 4: The survey's retirement claims

**Files:**
- Create: `survey/src/claims/retirement.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `sugarscape_core::retirement::{Counts, Groups, Policy, Renewal, RetirementConfig, Size}`, `crate::runner::model_after`, `crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict}`.
- Produces: 16 claims (`retirement.ae.*`).

- [ ] **Step 1: Write the claims**

Create `survey/src/claims/retirement.rs` with exactly this content:

````rust
//! The timing of retirement (milestone 26): Axtell and Epstein (1999), with
//! the revised text in Epstein (2006, ch. 7). Transition time is the first
//! period with 95 % of eligible agents retired (the texts never define it);
//! a run that never reaches it reads NaN.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::retirement::{Counts, Groups, Policy, Renewal, RetirementConfig};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const AE: &str = "Axtell & Epstein 1999, Brookings CSED WP 1";
const GSS: &str = "Epstein 2006, Generative Social Science, ch. 7";

fn outcome(holds: bool, measured: String) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: String::new(),
    }
}

fn config(edit: impl FnOnce(&mut RetirementConfig)) -> RetirementConfig {
    let mut c = RetirementConfig::default();
    edit(&mut c);
    c
}

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

/// `series` at the last period of runs stopped at the norm (NaN if never).
fn at_norm(c: RetirementConfig, n: u64, periods: u32, series: &str) -> Vec<f64> {
    let c = RetirementConfig {
        stop_at_norm: true,
        ..c
    };
    let name = series.to_string();
    model_after(&ModelConfig::Retirement(c), &seeds(n), periods, move |w| {
        w.model().latest_value(&name).unwrap()
    })
}

fn transitions(c: RetirementConfig, n: u64, periods: u32) -> Vec<f64> {
    at_norm(c, n, periods, "transition")
}

/// The whole `retired` series of runs (not stopped).
fn series(c: RetirementConfig, n: u64, periods: u32) -> Vec<Vec<f64>> {
    model_after(
        &ModelConfig::Retirement(c),
        &seeds(n),
        periods,
        |w: &ModelWorld| w.model().series("retired").unwrap(),
    )
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if f.is_empty() {
        f64::NAN
    } else {
        f.iter().sum::<f64>() / f.len() as f64
    }
}

fn reached(v: &[f64]) -> usize {
    v.iter().filter(|x| x.is_finite()).count()
}

fn describe(v: &[f64]) -> String {
    format!("{:.1} ({} of {} reached)", mean(v), reached(v), v.len())
}

/// A falling mean along `x` (each at least as fast as the last, within `slack`).
fn falling(means: &[f64], slack: f64) -> bool {
    means.windows(2).all(|w| w[1] <= w[0] + slack)
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "retirement.ae.rapid",
            item: "ae-rapid",
            source: Source::Book,
            citation: GSS,
            text: "15 % rational, 80 % imitators, 5 % random: 'Within the first 6 periods essentially all of the eligible population has retired … this trajectory is essentially monotone' (95 % retired by period 6 in most of 20 runs; never falling by more than 0.01)",
            check: |_| {
                let runs = series(config(|c| c.rational = 0.15), 20, 10);
                let by6 = runs.iter().filter(|s| s[6] >= 0.95).count();
                let monotone = runs.iter().filter(|s| s.windows(2).all(|w| w[1] >= w[0] - 0.01)).count();
                all_of(vec![
                    ("by period 6".into(), outcome(by6 >= 16, format!("{by6} of 20 at 95 % by period 6"))),
                    ("monotone".into(), outcome(monotone >= 16, format!("{monotone} of 20 never fall"))),
                ])
                .with("AE's text gives 15/75/5 (95 %) and its caption 20 %; GSS gives 15/80/5.")
            },
        },
        Claim {
            id: "retirement.ae.slow",
            item: "ae-slow",
            source: Source::Book,
            citation: AE,
            text: "5 % rational: 'It takes a long time for the absorbing state to be achieved … the trajectory is not monotone' (transition past 30 periods; the share falls by 0.02 or more somewhere in most of 20 runs)",
            check: |_| {
                let runs = series(config(|c| c.rational = 0.05), 20, 150);
                let dips = runs
                    .iter()
                    .filter(|s| {
                        let mut top = 0.0f64;
                        s.iter().any(|&x| {
                            top = top.max(x);
                            x < top - 0.02 && top < 0.9
                        })
                    })
                    .count();
                let t = transitions(config(|c| c.rational = 0.05), 20, 600);
                all_of(vec![
                    ("slow".into(), outcome(mean(&t) > 30.0, format!("transition {}", describe(&t)))),
                    ("not monotone".into(), outcome(dips >= 11, format!("{dips} of 20 fall back"))),
                ])
            },
        },
        Claim {
            id: "retirement.ae.footnote5",
            item: "ae-all-members",
            source: Source::Book,
            citation: AE,
            text: "Footnote 5: whether an agent counts all its network or only the eligible 'makes a difference to the numerical results … However, the qualitative character of the results … do not depend on this distinction' (the base case reaches the norm either way; 20 runs of 600 periods)",
            check: |_| {
                let e = transitions(RetirementConfig::default(), 20, 600);
                let a = transitions(config(|c| c.counts = Counts::All), 20, 600);
                outcome(reached(&a) >= 16, format!("eligible only: {}; all members: {}", describe(&e), describe(&a)))
            },
        },
        Claim {
            id: "retirement.ae.fig6-rationals",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-6: 'Reducing the proportion of rationals, while holding constant the proportion of randoms, increases transition time' (5 % random; 2 to 25 % rational; 20 runs)",
            check: |_| {
                let xs = [0.02, 0.05, 0.1, 0.15, 0.2, 0.25];
                let m: Vec<f64> = xs.iter().map(|&r| mean(&transitions(config(|c| c.rational = r), 20, 600))).collect();
                outcome(falling(&m, 1.0), format!("{:?} at {xs:?}", m.iter().map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>()))
            },
        },
        Claim {
            id: "retirement.ae.fig6-minimum",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-6: 'When randoms comprise 0 percent or 5 percent of the population, certain minimum proportions of the population must be rational for a retirement age norm to arise' (with 5 % randoms, runs with 0 % and 2 % rational never reach the norm in 1 500 periods; the default renewal)",
            check: |_| {
                let zero = transitions(config(|c| c.rational = 0.0), 10, 1500);
                let two = transitions(config(|c| c.rational = 0.02), 10, 1500);
                let rz = transitions(
                    config(|c| {
                        c.rational = 0.0;
                        c.per_cohort = 50;
                        c.renewal = Renewal::Replace;
                    }),
                    5,
                    1500,
                );
                let r5 = transitions(
                    config(|c| {
                        c.rational = 0.05;
                        c.per_cohort = 50;
                        c.renewal = Renewal::Replace;
                    }),
                    5,
                    1500,
                );
                outcome(reached(&zero) == 0 && reached(&two) == 0, format!("0 %: {}; 2 %: {}", describe(&zero), describe(&two)))
                    .with(&format!("With friends who die replaced within the holder's age range (C 50): 0 %: {}; 5 %: {}.", describe(&rz), describe(&r5)))
            },
        },
        Claim {
            id: "retirement.ae.fig6-randoms",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-6: 'For a given fraction of rationals, the transition time decreases as the proportion of randoms increases' (5 % rational: 0, 5, 10 % random; 20 runs)",
            check: |_| {
                let t = |r: f64| transitions(config(|c| {
                    c.rational = 0.05;
                    c.random = r;
                }), 20, 600);
                let (a, b, c) = (t(0.0), t(0.05), t(0.1));
                all_of(vec![
                    ("0 → 5 %".into(), greater(&a, &b, "no randoms", "5 %")),
                    ("5 → 10 %".into(), greater(&b, &c, "5 %", "10 %")),
                ])
            },
        },
        Claim {
            id: "retirement.ae.cohort",
            item: "ae-base",
            source: Source::Book,
            citation: GSS,
            text: "'The first parameter, the number of agents per cohort (C), was found to have no effect on the average transition time for C > 100' (C 100 and 200 the same within 20 %; 20 runs)",
            check: |_| {
                let a = transitions(RetirementConfig::default(), 20, 600);
                let b = transitions(config(|c| c.per_cohort = 200), 20, 600);
                let s = transitions(config(|c| c.per_cohort = 25), 20, 600);
                equivalent(&a, &b, Some(0.2 * mean(&a)), "C 100", "C 200").with(&format!("C 25: {}.", describe(&s)))
            },
        },
        Claim {
            id: "retirement.ae.fig7",
            item: "ae-threshold",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-7: 'Increasing the variance in the threshold decreases the average transition time' (spreads 0, 0.05, 0.1, 0.15, 0.2, 0.25, uniform around 0.5; 20 runs)",
            check: |_| {
                let xs = [0.0, 0.05, 0.1, 0.15, 0.2, 0.25];
                let m: Vec<f64> = xs.iter().map(|&s| mean(&transitions(config(|c| c.spread = s), 20, 600))).collect();
                outcome(falling(&m, 1.0), format!("{:?}", m.iter().map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>()))
                    .with("Falling overall, but a little spread first doubles the time.")
            },
        },
        Claim {
            id: "retirement.ae.fig8",
            item: "ae-size",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-8: transition time 'increases very rapidly with increasing social network size' (mean 10 to 40, ± 7); falls weakly with the size's spread (17 ± 0 to ± 14); increases with S̄ in U[10, S̄] (20 runs)",
            check: |_| {
                let size = |min: u32, max: u32| mean(&transitions(config(|c| c.size = sugarscape_core::retirement::Size { min, max }), 20, 600));
                let a: Vec<f64> = [10u32, 15, 20, 25, 30, 40].iter().map(|&m| size(m.saturating_sub(7), m + 7)).collect();
                let b: Vec<f64> = [0u32, 3, 7, 10, 14].iter().map(|&h| size(17 - h, 17 + h)).collect();
                let c: Vec<f64> = [10u32, 20, 30, 40, 60, 80].iter().map(|&m| size(10, m)).collect();
                let r = |v: &[f64]| format!("{:?}", v.iter().map(|x| x.round()).collect::<Vec<_>>());
                all_of(vec![
                    ("mean size".into(), outcome(a.windows(2).all(|w| w[1] >= w[0] - 1.0) && a[5] > 3.0 * a[0], r(&a))),
                    ("spread".into(), outcome(falling(&b, 1.0) && b[4] < b[0], r(&b))),
                    ("maximum".into(), outcome(c.windows(2).all(|w| w[1] >= w[0] - 1.0) && c[5] > 3.0 * c[0], r(&c))),
                ])
            },
        },
        Claim {
            id: "retirement.ae.fig9",
            item: "ae-extent",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-9: 'the effect of increasing the extent (in the age dimension) of agent social networks is to decrease the transition times' (extent 1 against 10, at 10 % and 5 % rational; 20 runs)",
            check: |_| {
                let t = |r: f64, e: u32| transitions(config(|c| {
                    c.rational = r;
                    c.extent = e;
                }), 20, 600);
                all_of(vec![
                    ("10 % rational".into(), greater(&t(0.1, 1), &t(0.1, 10), "extent 1", "extent 10")),
                    ("5 % rational".into(), greater(&t(0.05, 1), &t(0.05, 10), "extent 1", "extent 10")),
                ])
                .with(&format!("At 5 %, extent 3: {}.", describe(&t(0.05, 3))))
            },
        },
        Claim {
            id: "retirement.ae.as-if",
            item: "ae-rational",
            source: Source::Book,
            citation: AE,
            text: "'The attainment per se of the age 65 retirement norm is compatible with any rationality fraction above a critical level' (every run from 2 % rational up reaches the norm; 5 % random; 10 runs of 600 periods)",
            check: |_| {
                let v: Vec<usize> = [0.02, 0.05, 0.1, 0.2].iter().map(|&r| reached(&transitions(config(|c| c.rational = r), 10, 600))).collect();
                outcome(v.iter().all(|&k| k == 10), format!("{v:?} of 10 at 2, 5, 10, 20 %"))
            },
        },
        Claim {
            id: "retirement.ae.mandatory",
            item: "ae-policy",
            source: Source::Book,
            citation: AE,
            text: "'Now we require that all agents retire at age 70. This increases the speed at which the age 65 retirement norm is established' (5 % rational; 20 runs)",
            check: |_| {
                let free = transitions(config(|c| c.rational = 0.05), 20, 600);
                let forced = transitions(config(|c| {
                    c.rational = 0.05;
                    c.mandatory = 70;
                }), 20, 600);
                greater(&free, &forced, "no mandatory age", "mandatory at 70")
                    .with("With 70+ forced out, they are most of the eligible: the 95 % measure then says little about retiring at 65.")
            },
        },
        Claim {
            id: "retirement.ae.policy",
            item: "ae-policy",
            source: Source::Book,
            citation: AE,
            text: "The policy switch (mandatory 70; eligibility 65 → 62 once the norm is established): 'a new norm indeed emerges after twenty to thirty periods'; 'in about 35 periods if between 1 and 4 percent of the population responds rationally' (periods from the switch to 95 % of those 62+ retired, at 1 %, 2 %, 4 % rational; within 20–40)",
            check: |_| {
                let parts = [0.01, 0.02, 0.04]
                    .into_iter()
                    .map(|r| {
                        let v = at_norm(
                            config(|c| {
                                c.rational = r;
                                c.mandatory = 70;
                                c.policy = Policy { enabled: true, to: 62 };
                            }),
                            20,
                            600,
                            "transition_new",
                        );
                        let m = mean(&v);
                        (format!("{} % rational", r * 100.0), outcome((20.0..=40.0).contains(&m), format!("{m:.1} periods")))
                    })
                    .collect();
                all_of(parts).with("Imitators just turned 62 count the retired 65-to-67-year-olds among their eligible friends and retire at once; the same under friends replaced.")
            },
        },
        Claim {
            id: "retirement.ae.groups-pull",
            item: "ae-coupling",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-11: 'very little coupling is needed for the non-rational sub-population to be pulled into conformity' (the group without rationals reaches the norm sooner at coupling 0.1 than uncoupled; 20 runs of 300 periods)",
            check: |_| {
                let t = |k: f64, s: &str| {
                    let name = s.to_string();
                    model_after(
                        &ModelConfig::Retirement(config(|c| c.groups = Groups { enabled: true, coupling: k })),
                        &seeds(20),
                        300,
                        move |w| w.model().latest_value(&name).unwrap(),
                    )
                };
                greater(&t(0.0, "transition_a"), &t(0.1, "transition_a"), "uncoupled", "coupling 0.1")
            },
        },
        Claim {
            id: "retirement.ae.groups-rational",
            item: "ae-coupling-rational",
            source: Source::Book,
            citation: AE,
            text: "Figure 6-11: the sub-population with rational agents keeps its transition time as coupling rises (coupling 0 and 0.1 the same within 25 %; 20 runs of 300 periods)",
            check: |_| {
                let t = |k: f64| {
                    model_after(
                        &ModelConfig::Retirement(config(|c| c.groups = Groups { enabled: true, coupling: k })),
                        &seeds(20),
                        300,
                        |w| w.model().latest_value("transition_b").unwrap(),
                    )
                };
                let (a, b, c) = (t(0.0), t(0.1), t(0.25));
                equivalent(&a, &b, Some(0.25 * mean(&a)), "uncoupled", "coupling 0.1").with(&format!("At 0.25: {}.", describe(&c)))
            },
        },
    ]
}
````

Modify `survey/src/claims/mod.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index 74a583c..c335dee 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -15,6 +15,7 @@ mod norms;
 mod image;
 mod minds1;
 mod opinions;
+mod retirement;
 mod spatial;
 mod structure;
 mod tags;
@@ -41,6 +42,7 @@ pub fn all() -> Vec<Claim> {
         image::claims(),
         minds1::claims(),
         opinions::claims(),
+        retirement::claims(),
         spatial::claims(),
         structure::claims(),
         tags::claims(),
````

- [ ] **Step 2: Run them**

Run: `cd survey && rustfmt --edition 2021 src/claims/retirement.rs && cargo build --release && ./target/release/survey --only retirement`
Expected (about 25 s): 9 Holds (slow, fig6-rationals, fig6-randoms, cohort, fig8, as-if, mandatory, groups-pull) and 7 Fails (rapid — 4 of 20 by period 6; footnote5 — no norm counting all; fig6-minimum — 0 % reaches it under `slot`, detail: `replace` needs more; fig7 — not monotone; fig9 — no effect at 5 %; policy — 2 periods; groups-rational — 17.5 → 32.5).

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/retirement.rs survey/src/claims/mod.rs
```
```bash
git commit -m "Survey The Timing of Retirement: realizations, sensitivity, policy and sub-populations

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 5: README, roadmap, papers index, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-27-retirement-design.md`

- [ ] **Step 1: Write the docs**

Modify `README.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/README.md b/README.md
index d0b7a49..4743f76 100644
--- a/README.md
+++ b/README.md
@@ -154,7 +154,7 @@ The presets menu groups its presets by model: **Sugarscape**, **Schelling**, **R
 **Artificial Anasazi**, **Civil Violence**, **Tag Cooperation**, **Spatial Games**, **Axelrod Culture**,
 **Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence**, **Social Structure**,
 **Demographic PD**, **Norms and Metanorms**, **Relative Agreement**,
-**Image Scoring**, **El Farol and the Minority Game**, **Ants and Recruitment** and **Threshold Models**.
+**Image Scoring**, **El Farol and the Minority Game**, **Ants and Recruitment**, **Threshold Models** and **The Timing of Retirement**.
 Each preset is listed by a plain title saying what happens in it; under the menu, the chosen
 preset's source (the book's figure or animation, or the paper) and its rules sit above its description.
 Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
@@ -1545,6 +1545,59 @@ Credit: Mark Granovetter, "Threshold Models of Collective Behavior," *American J
 83(6) (1978), 1420–1443; Duncan J. Watts, "A Simple Model of Global Cascades on Random Networks,"
 *PNAS* 99(9) (2002), 5766–5771. See `docs/superpowers/specs/2026-09-27-thresholds-design.md`.
 
+### The Timing of Retirement (Axtell & Epstein 1999)
+
+**The model.** In 1961 Congress let workers claim Social Security at 62 instead of 65, yet it took
+nearly three decades for the most common retirement age to follow. Axtell and Epstein's agents live
+in 81 one-year cohorts, die at random between 60 and 100, and are replaced by 20-year-olds. A few are
+rational and retire as soon as they may; a few retire at random; most imitate, retiring once half the
+eligible members of their own small network — people within a few years of their age — have.
+
+Measured (the survey and the presets' descriptions):
+
+- **The realizations reproduce.** With 15 % rational, 95 % of those eligible have retired by period 8
+  on average, rising steadily (the text says "within the first 6 periods"); with 5 %, retirement
+  stalls, wavers and "percolates up" from the old, finishing near period 61. Larger networks slow the
+  transition, a spread of network sizes speeds it, the cohort size does not matter, and retirement
+  mandatory at 70 speeds it — all as stated.
+- **Footnote 5 is false.** Counting every friend instead of the eligible ones is said to leave the
+  results' "qualitative character" unchanged; counting every friend, no norm ever forms — the young
+  friends hold the share retired below one half.
+- **Figure 6-6 needs an unstated rule.** Under the pseudo-code's reading — a dead friend's place
+  passes to the newborn in its slot — no minimum of rationality is needed (72 periods with no
+  rationals at all) and nothing takes the paper's hundreds of periods. Only if friends who die are
+  replaced by someone of about the same age do the paper's "minimum proportions" and long, erratic
+  transitions appear (no norm at 0 or 5 % rational; 10 % takes 70 ± 75 periods).
+- **The policy switch does not reproduce.** Lowering eligibility to 62 once the norm is established,
+  the paper's new norm "emerges after twenty to thirty periods"; here it comes in 2, at every share of
+  rationals, under either rule: an imitator just turned 62 counts its retired 65-to-67-year-old friends
+  and retires at once. The decades the model was built to explain do not follow from its rules.
+- **Coupling pulls both ways.** A little coupling between a community without rationals and one with
+  them pulls the first into line (75 → 46 periods at 0.1), as the paper says, but slows the second just
+  as much (19 → 34), until both take about 58; the paper's figure keeps the rational group fast. A
+  little spread in the thresholds first doubles the transition time before more spread shortens it.
+
+Switches: **Agents per cohort**, **The first agents' death ages**, **Each period, agents act** (cohort by
+cohort, oldest first, or in one random order), **Rational share**, **Random share**, **Random agents'
+chance**, **Imitation threshold**, **Threshold spread**, **Imitators count** (eligible members, or every
+member), **Network size**, **Extent**, **When a member dies** (the newborn in its slot takes its place,
+or it is replaced within the holder's age range), **Eligibility age**, **Mandatory age**, **Lower the
+age once the norm is reached** with **To**, **The norm is reached at** (the paper never defines its
+transition time; here, the first period with that share of the eligible retired), **Two
+sub-populations** with **Coupling**, and **Stop at the norm**. The view is Axtell and Epstein's: one row
+per age from 20 at the top, agents colored by type while working and red once retired; beside it, the
+share retiring at each age over the last 10 periods, and the share of the eligible retired over time.
+Color modes: **Status**, **Type**, **Threshold**, **Group**. Charts: Retired share (by group with two
+sub-populations); Retirement age; Transition; Group transitions. Presets: `ae-rapid`, `ae-base`,
+`ae-slow`, `ae-policy`, `ae-groups`, `ae-all-members`, `ae-replace`. **Compare** entry: "15 % vs 5 %
+rational — Retirement (Compare)". Built-in sweeps: `ae-rational`, `ae-rational-replace`,
+`ae-threshold`, `ae-size`, `ae-extent`, `ae-policy`, `ae-coupling`, `ae-coupling-rational`.
+
+Credit: Robert L. Axtell and Joshua M. Epstein, "Coordination in Transient Social Networks: An
+Agent-Based Computational Model of the Timing of Retirement," Brookings CSED Working Paper No. 1 (1999),
+in H. Aaron, ed., *Behavioral Dimensions of Retirement Economics* (1999); Joshua M. Epstein, *Generative
+Social Science* (Princeton, 2006), chapter 7. See `docs/superpowers/specs/2026-09-27-retirement-design.md`.
+
 ## Experiments
 
 The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
````

Modify `docs/roadmap.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/roadmap.md b/docs/roadmap.md
index 0d8aa9b..2c8dbe7 100644
--- a/docs/roadmap.md
+++ b/docs/roadmap.md
@@ -238,6 +238,17 @@ claims hold under a stated reading; middling movement is most incendiary; ceilin
 Watts's window and power law reproduce; his upper edge depends on n, his Fig. 4b cannot be built as
 stated, and hubs help in both regimes. See `docs/superpowers/specs/2026-09-27-thresholds-design.md`.
 
+## Milestone 26: The Timing of Retirement (done)
+
+Axtell and Epstein's retirement model (1999; Epstein 2006, ch. 7) as a model kind: cohorts, deaths and
+newborns, rational, random and imitating agents in transient networks, the policy switch from 65 to 62
+and two coupled sub-populations, with the unstated rules — whom an imitator counts, what becomes of a
+dead friend's place, activation order, transition time — as switches. The realizations and the
+network-size effects reproduce; footnote 5 is false (counting every friend, no norm forms); Figure
+6-6's minimum of rationality needs an unstated renewal rule; the policy switch's slow response does
+not reproduce (the new norm comes in 2 periods); coupling slows the rational group as much as it
+speeds the other. See `docs/superpowers/specs/2026-09-27-retirement-design.md`.
+
 ## Experiments and science
 
 - **Parameter sweeps / batch runs**: done (Milestone 5).
@@ -259,6 +270,7 @@ stated, and hubs help in both regimes. See `docs/superpowers/specs/2026-09-27-th
 - **Arthur's El Farol and Challet & Zhang's minority game** (and the memory transition, and Challet, Marsili & Ottino's critique): done (Milestone 23).
 - **Kirman's ants and recruitment** (and Alfarano & Milaković's network critique): done (Milestone 24).
 - **Granovetter's threshold models** (and Watts's global cascades): done (Milestone 25).
+- **Axtell and Epstein's timing of retirement**: done (Milestone 26).
 - **Minds 1: the utility mind and the ideal free distribution** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Credit hierarchy view**: done (Milestone 6).
````

Modify `docs/papers.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/papers.md b/docs/papers.md
index 3f9be75..88fe837 100644
--- a/docs/papers.md
+++ b/docs/papers.md
@@ -33,6 +33,7 @@ read online or from another copy; add it when found. Scanned PDFs (no text layer
 | 23 | `farol` | `el-farol/arthur-1994-aer-inductive-reasoning-and-bounded-rationality.pdf`, `el-farol/challet-zhang-1997-emergence-of-cooperation-minority-game.pdf`; follow-ups `el-farol/savit-manuca-riolo-1999-prl-adaptive-competition-market-efficiency-phase-transitions.pdf`, `el-farol/challet-zhang-1998-physica-a-on-the-minority-game-analytical-and-numerical.pdf`, `el-farol/challet-marsili-ottino-2004-physica-a-shedding-light-on-el-farol.pdf` | Arthur's mean of 60 is trivial and his agents swing far more than coin-flippers; his cycles persist under accuracy scoring; the memory transition reproduces; CZ97's Fig. 4 two peaks and Fig. 10 waste do not |
 | 24 | `ants` | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf`; the critique `ants/alfarano-milakovic-2007-warwick-wp-should-network-structure-matter.pdf` (published as JEDC 33(1), 2009) | the chain is exactly beta-binomial but never rests at the ants' 80–20 (Becker's pull does); Fig. IIb's time average needs 100× the figure; herding fades with N, cured by a random network only under AM's rule; AM's mean field fails on rings |
 | 25 | `thresholds` | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*, read by OCR); the follow-up `thresholds/watts-2002-pnas-simple-model-of-global-cascades-on-random-networks.pdf` (from the Internet Archive's copy of PNAS) | the crowds and Fig. 2's continuous jump reproduce, but a crowd of people tips at a σ set by rounding and sampled crowds do not jump; the city's riot of 100 comes 2 % of the time; Watts's upper edge depends on n, his Fig. 4b cannot be built, hubs help in both regimes |
+| 26 | `retirement` | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf`; the revised text `demographic-pd/epstein-2006-generative-social-science.pdf` (ch. 7) | the realizations and network effects reproduce; footnote 5 is false; Fig. 6-6's minimum rationality needs an unstated rule (friends replaced); the 65 → 62 switch takes 2 periods, not 20–35; coupling slows the rational group |
 
 ## Queue
 
@@ -41,12 +42,11 @@ worth doing; "size" is a guess at the milestone's scale.
 
 | # | Model | Original | Critique or follow-up | Size | Shape |
 |---|---|---|---|---|---|
-| 1 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
-| 2 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
-| 3 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
-| 4 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
-| 5 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
-| 6 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
+| 1 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
+| 2 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
+| 3 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
+| 4 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
+| 5 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
 
 ## Wanted
````

Modify `docs/superpowers/specs/2026-09-27-retirement-design.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/superpowers/specs/2026-09-27-retirement-design.md b/docs/superpowers/specs/2026-09-27-retirement-design.md
index 6ac50b0..ff94f15 100644
--- a/docs/superpowers/specs/2026-09-27-retirement-design.md
+++ b/docs/superpowers/specs/2026-09-27-retirement-design.md
@@ -138,3 +138,15 @@ The presets menu gains a **The Timing of Retirement** group and the Compare entr
 ## Docs
 
 README: a Timing of Retirement section (the model, the stated choices, switches, presets, sweeps, and the findings: the realizations reproduce; footnote 5 fails — counting all members, no norm forms; Fig. 6-6's minimum of rationals and long times need the unstated `replace` rule; the network-size and extent effects hold; the policy switch's decades-long response does not follow — the new norm comes in a few periods; the rational group is slowed by the coupling that pulls the other in). `docs/papers.md`: the milestone's row (with GSS ch. 7); the Queue's first entry removed; roadmap: Milestone 26 done.
+
+## Amendments (implementation planning)
+
+The model was implemented in full while planning (`docs/superpowers/plans/2026-09-28-retirement.md`) and measured with it; these change or extend the sections above.
+
+- **Each agent ages when it is activated** (the pseudo-code: "select an agent … increment its age"); those not yet activated in a period are a year younger. A first implementation that aged everyone at the start of the period took twice as long (31 periods for the base case, 55 shuffled): an imitator's peers who had just turned 65 were already eligible, and still working, when it decided. Cohorts are birth periods, so a network's extent compares birth periods, which never change.
+- **`mandatory` may be any age up to 100** (0: none); below 20 everyone retires at once.
+- **Statistics** add `transition_a` and `transition_b`, each group's first period at the norm (both `transition` without groups), for Fig. 6-11's two sweeps (`ae-coupling`, `ae-coupling-rational`) — a sweep reads one series. `modal_age` and `mean_age` are NaN (null) with no retirements in the window; `eligibility` is the age now (it drops at the switch).
+- **Color modes** are Status, Type, Threshold and Group; the Network mode (a selected agent's network marked) is dropped — the frame has no selection. Inspect lists an agent's network with how many members are eligible and retired.
+- **Sweeps:** `ae-rational`, `ae-rational-replace`, `ae-threshold`, `ae-size` (S ~ U[10, max], Fig. 6-8c), `ae-extent`, `ae-policy`, `ae-coupling`, `ae-coupling-rational`; the metric is the final `transition` (or `transition_new`, `transition_a`, `transition_b`), which keeps its value once reached.
+- **With retirement mandatory at 70**, those forced out are most of the eligible, so the 95 % measure reaches the 65 "norm" in about 3 periods whatever the rationality; the claim that a mandatory age speeds the norm holds but says little about retiring at 65.
+- **Measured with the implementation** (the survey, 16 claims; 9 hold, 7 fail): 15 % rational reaches 95 % by period 6 in 4 of 20 runs (mean 7.7), monotone in all; 5 % takes 61 periods, not monotone in 20 of 20; counting all members, no norm in 20 of 20; transition 69.5, 61.1, 16.4, 7.7, 5.2, 3.9 at 2–25 % rational (5 % random); no minimum of rationals (0 %: 72; 2 %: 69) under `slot`, none of 0 and 5 % reaching it under `replace`; randoms speed it (72, 61, 18 at 0, 5, 10 % random, 5 % rational); C 100 and 200 equivalent (17 and 17; C 25: 18); threshold spread 16.4, 36.6, 19.7, 11.8, 8.3, 7.9 (not monotone); network size 7 → 73 (mean), 18 → 12 (spread), 7 → 70 (maximum); extent 1 against 10: 18 against 13.5 at 10 %, 53.5 against 54.5 at 5 % (no effect); every run from 2 % rational reaches the norm; mandatory 70: 3 against 61 periods; the policy switch 2.0 periods at 1, 2, 4 % rational; the group without rationals 75 → 49 at coupling 0.1, the group with rationals 17.5 → 32.5 (60 at 0.25).
````

- [ ] **Step 2: Verify everything**

Run: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm && (cd web && npm run build && npm test) && (cd survey && cargo build --release && ./target/release/survey --only retirement)`
Expected: all green; the survey's verdicts as in Task 4.

- [ ] **Step 3 (controller): the full browser pass** — Task 3's Step 6 list again, on the final build.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-09-27-retirement-design.md
```
```bash
git commit -m "Document The Timing of Retirement and what does not reproduce; mark milestone 26 done

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

## Self-review (planning)

- **Spec coverage:** Architecture, Config, Step, Statistics, Views, Presets, Compare, Experiments and CLI → Tasks 1–3; Survey → Task 4; Docs → Task 5; departures in the spec's Amendments.
- **Placeholders:** none; every file is given in full or as a diff against `9121c34`.
- **Types:** `RetirementInspection` and `RetireeView` (types.ts) match `RetirementInspection` and `AgentView` (world.rs); `isRetirementView` tests `exposed`, which no other inspection has.
