# Altruistic Punishment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Boyd, Gintis, Bowles and Richerson's altruistic punishment (PNAS 2003) as one model kind, `punishment` ("Altruistic Punishment"): groups of contributors, defectors and punishers, payoff-biased imitation with mixing, intergroup conflict and mutation, the paper's three structural variants, Cooney's (2024) victory rules and Janssen's readings, with every unstated rule a switch; eleven titled presets, eighteen measured sweeps and a 25-claim survey, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/punishment/` — `config.rs` (parameters, the ten reading enums, validation, schema), `stats.rs`, `view.rs` (the group mosaic and time strip), `world.rs` (`PunishmentWorld`: acts, payoffs, imitation, pairing and victory, refill, mutation, the long-run window, rendering, Inspect, `payoff_range`, portable `tanh`), `presets.rs`, `mod.rs` — wired into `ModelConfig`/`ModelWorld` like the other models, with titles in `titles.rs`. The page adds the model's types, color modes, charts, Inspect rows, a Compare entry and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-28-punishment-design.md` (binding, as amended in Task 5). Sources: `papers/punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf`, `papers/punishment/cooney-2024-arxiv-altruistic-punishment-pde-multilevel-selection.pdf`; Janssen's NetLogo (`papers/punishment/janssen-comses-2223-netlogo/`, GPL-3.0) is read for its readings only — nothing is copied.

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited (`MODEL_GOLDEN` gains eleven `bg-*` entries).
- **One engine path; deterministic; portable:** native and WASM fingerprints identical (verified in planning by `wasm-pack test`, including a tanh config). Draws use `u32` ranges and `f64` samples only; `tanh` is built on `crate::portable::exp_neg`, never `f64::tanh`; no other `ln`/`exp`/trigonometry in the module.
- **One fit rule:** every survey claim that a reading "reproduces the figure" uses a mean absolute gap of at most 0.05 over group sizes 4–256, against values read from the PDF at 300 dpi (the constants in `survey/src/claims/punishment.rs`).
- **Literal defaults, named departures, honest descriptions and titles:** the defaults are the text's (paired conflict at ε, d by type, all-at-once imitation, a defeated group copied, a punisher who errs still punishing the others); the unstated baseline is 1 (their calibration); `either`, `challenge`, `acts`, `in_turn`, `self`, `split`, the benefit and the payoff rules are switches; every description and title says what was measured.
- **Copy (verbatim):** model label **Altruistic Punishment**; preset ids `bg-base`, `bg-either`, `bg-none`, `bg-large`, `bg-weak`, `bg-fixed`, `bg-mixing`, `bg-benefit`, `bg-continuous`, `bg-ring`, `bg-janssen`; Compare entry **With vs without punishment — Altruistic Punishment (Compare)** (id `bg-base-vs-none`); color modes **Type**, **Acts**, **Payoff**, **Group**; schema groups **Groups**, **Game**, **Imitation**, **Conflict**, **Variants**, **Stopping**; charts **Types**, **Cooperation**, **Payoff**, **Conflict**; time axis **Periods**; sweeps `bg-fig1a`, `bg-fig1b`, `bg-fig1-caption`, `bg-fig1-either`, `bg-fig2a`, `bg-fig2b`, `bg-fig3`, `bg-fig4`, `bg-baseline`, `bg-readings`, `bg-mutation`, `bg-error`, `bg-groups`, `bg-benefit`, `bg-continuous`, `bg-ring`, `bg-cooney-fine`, `bg-cooney-cost`; series `cooperation, contributors, punishers, defectors, punishment, acts, payoff, conflicts, extinctions, spread, long_run`; notice `This run has reached its last period (N) — Reset to run it again`; CLI `(its last period)`.
- Every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4`. Stage only the task's files; never `.claude/` or `papers/`.
- Rust: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. In `survey/`, format only `survey/src/claims/punishment.rs`; do not commit `survey/out/results-*.json`.
- Web: `(cd web && npm run build && npm test)` (run `npm ci` and `npm run wasm` first in a fresh worktree).
- **Browser checks are the controller's** (Task 3's Step 6; the full pass in Task 5).

## Review Focus

1. **Who is fined and who pays** — a defector pays p/n per *other* punisher (and itself too only under `self`); a punisher pays k/n per *other* defector; a punisher who defected punishes the others (`others`), nobody (`none`), or also itself (`self`); a fixed cost is paid whatever the group; payoffs floor at 0 and two zero payoffs imitate at ½. Pinned in Task 1 by `payoffs_follow_the_game` and `a_punisher_who_errs_follows_its_rule`.
2. **Imitation's draws** — the model is another member of the group (never itself), or with probability m a member of another group (on a ring, a neighbor); `together` copies the period's starting traits, `in_turn` the current ones. Pinned by `imitation_copies_with_the_payoff_ratio`, `together_imitates_from_the_starting_traits_and_in_turn_does_not`, `migrants_come_from_other_groups_and_on_a_ring_from_neighbors`.
3. **Conflict rates and victory** — under `paired` each pair fights with probability ε (a group dies at about ε/2 a period), under `either` with 2ε − ε², under `challenge` each group starts about ε; victory clamps to [0, 1], a zero payoff range is a draw, `payoff_range` is exact. Pinned by `paired_groups_fight_at_half_the_rate_and_either_or_challenge_about_the_whole`, `victory_follows_each_rule`, `the_payoff_range_matches_a_grid`.
4. **The long-run window under live edits** — changing `stop_at` or `window` mid-run rebuilds the average from the history; NaN before the window. Pinned by `the_long_run_average_covers_its_window`.
5. **Portability of the tanh rule and every preset** — WASM fingerprints equal native. Pinned in Task 2 by `punishment_sims_match_the_native_golden_entries` (six presets and a tanh config).

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --workspace` (1 040), `cargo clippy --all-targets -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (51), `npm run build && npm test` (728, 49 files) and the survey (25 claims, about 3.5 minutes).

1. **`pairing: either`** (amends the spec): the reading the figures fit — either group of a pair starts the conflict. It reproduces all six of Fig. 1's curves (mean gaps 0.010–0.033) and Figs. 2–3 (0.006–0.022); Fig. 4's fixed cost misses at 0.051. Hence the preset `bg-either` and the sweep `bg-fig1-either`.
2. **Figures digitized** at 300 dpi by marker centers; a hidden marker takes the overlapping curve's value.
3. **Sweeps:** eighteen (the spec's `bg-sensitivity` split into three; `bg-readings`, `bg-continuous` and the Cooney sweeps gain a series each).
4. **The Conflict chart** shows `conflicts` and `spread` (`extinctions` always equals `conflicts`).
5. **Planning's findings** (the survey reproduces them): the figures' shapes hold under the text's rules but their reach does not; neither the caption's nor the legend's rates, nor any baseline, reproduces Fig. 1; `either` does; the mixing calibration is off (0.58 of the difference left at 50 periods); continuous traits are not similar (0.94 against 0.69 at n 32); the ring keeps half cooperating in groups of 4; Cooney's dip appears under every victory rule and a higher k never raises punishment.

---

### Task 1: The punishment model in the core

**Files:**
- Create: `crates/sugarscape-core/src/punishment/{config,stats,view,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{lerp, Rgb}`, `crate::rng::{self, SimRng}`, `crate::portable::exp_neg`, `crate::schema::{Apply, Param}`, `crate::presets::ModelPreset`, `crate::opinions::Canvas`.
- Produces: `punishment::{PunishmentConfig, Punishing, Pairing, Victory, Counted, Erring, Imitation, Refill, Traits, Structure, Start, MOST_AGENTS, schema, presets, PunishmentSnapshot, SERIES, Mosaic, STRIP, STRIP_GAP, payoff_range, AgentView, GroupView, Kind, PunishmentCell, PunishmentInspection, PunishmentMode, PunishmentWorld}`; `PunishmentWorld::{new, step, run, groups, size, traits, is_finished, inspect}` and `pub tick`; `ModelKind::Punishment` (`"punishment"`), `ModelConfig::Punishment`, `ModelWorld::Punishment`; eleven titles.

- [ ] **Step 1: Write the module**

The tests are in each file (config: defaults, validation, reset fields, `erring: "self"`, the schema; view: the mosaic and colors; world: nineteen, listed in the Review Focus).

Create `crates/sugarscape-core/src/punishment/config.rs` with exactly this content:

````rust
//! Altruistic Punishment's parameters: Boyd, Gintis, Bowles and Richerson's
//! (2003) groups of contributors, defectors and punishers, with payoff-biased
//! imitation, mixing, intergroup conflict and mutation, the variants their
//! text describes, and every detail it leaves open as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// What a punisher pays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Punishing {
    /// k/n for each defector it punishes (the text).
    Variable,
    /// A flat cost each period, whatever the group (Fig. 4).
    Fixed,
}

/// How groups meet in conflict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pairing {
    /// "groups are paired at random, and with probability ε" each pair fights.
    Paired,
    /// Groups paired at random, and either group of a pair can start the
    /// conflict, each with probability ε: a pair fights with probability
    /// 2ε − ε² (the reading the figures fit).
    Either,
    /// Each group, in random order, challenges a random group not yet
    /// fighting with probability ε (Janssen's reading; about twice the rate).
    Challenge,
}

/// Who wins a conflict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Victory {
    /// ½(1 + dⱼ − dᵢ): the group with fewer defectors (the text).
    Defectors,
    /// ½ + ½(Ḡᵢ − Ḡⱼ)/(G_max − G_min): average payoffs normalized by the
    /// widest possible difference (Cooney's eq. 3.18).
    Payoff,
    /// ½ + ½ tanh(s(Ḡᵢ − Ḡⱼ)): Cooney's group-level Fermi rule.
    Tanh,
}

/// What d, a group's share of defectors, counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Counted {
    /// Defector types (1 − the mean cooperation trait).
    Types,
    /// Those who defected this period (Janssen).
    Acts,
}

/// What a punisher who errs and defects does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Erring {
    /// Still punishes the other defectors.
    Others,
    /// Punishes nobody.
    None,
    /// Punishes the other defectors and itself (Janssen).
    #[serde(rename = "self")]
    Itself,
}

/// When imitation takes effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Imitation {
    /// Everyone imitates from the period's starting traits.
    Together,
    /// In turn: later agents see earlier agents' changes (Janssen).
    InTurn,
}

/// What replaces a defeated group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refill {
    /// A copy of the winners, member for member.
    Copy,
    /// Both groups refilled by drawing the winners' members with replacement.
    Split,
}

/// What an agent's traits can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Traits {
    /// Contributors, defectors and punishers.
    Discrete,
    /// Cooperation and punishment in [0, 1]; mutants uniform.
    Continuous,
}

/// How groups are arranged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Structure {
    /// Migrants from any group; conflict between random pairs.
    Groups,
    /// A ring: migrants only from the two neighbors, and no conflict.
    Ring,
}

/// The first period's population.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// "one group consisted of all altruistic punishers and the other 127
    /// groups were all defectors".
    OnePunisherGroup,
    AllDefectors,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PunishmentConfig {
    /// N.
    pub groups: u32,
    /// n.
    pub size: u32,
    /// c: the cost of cooperating.
    pub cost: f64,
    /// k: a punisher pays k/n per defector.
    pub punish_cost: f64,
    /// p: a defector pays p/n per punisher.
    pub fine: f64,
    pub punishing: Punishing,
    /// The flat cost under `Punishing::Fixed`.
    pub fixed_cost: f64,
    /// b: each cooperative act gives b/n to every other member.
    pub benefit: f64,
    /// The payoff the game's costs and benefits are added to.
    pub baseline: f64,
    /// e.
    pub error: f64,
    /// m.
    pub mixing: f64,
    /// μ.
    pub mutation: f64,
    /// ε.
    pub conflict: f64,
    pub pairing: Pairing,
    pub victory: Victory,
    /// s, for `Victory::Tanh`.
    pub sensitivity: f64,
    pub counted: Counted,
    pub erring: Erring,
    pub imitation: Imitation,
    pub refill: Refill,
    pub traits: Traits,
    pub structure: Structure,
    pub start: Start,
    /// `long_run` averages over the last `window` periods before `stop_at`.
    pub window: u32,
    /// Stop at this period (0: never).
    pub stop_at: u32,
}

impl Default for PunishmentConfig {
    /// The base case: 128 groups of 32, c = k = 0.2, p = 0.8, e = 0.02,
    /// m = 0.01, μ = 0.01, ε = 0.015, 2 000 periods.
    fn default() -> Self {
        PunishmentConfig {
            groups: 128,
            size: 32,
            cost: 0.2,
            punish_cost: 0.2,
            fine: 0.8,
            punishing: Punishing::Variable,
            fixed_cost: 0.2,
            benefit: 0.0,
            baseline: 1.0,
            error: 0.02,
            mixing: 0.01,
            mutation: 0.01,
            conflict: 0.015,
            pairing: Pairing::Paired,
            victory: Victory::Defectors,
            sensitivity: 10.0,
            counted: Counted::Types,
            erring: Erring::Others,
            imitation: Imitation::Together,
            refill: Refill::Copy,
            traits: Traits::Discrete,
            structure: Structure::Groups,
            start: Start::OnePunisherGroup,
            window: 1000,
            stop_at: 2000,
        }
    }
}

/// The most agents a world holds.
pub const MOST_AGENTS: u32 = 131_072;

impl PunishmentConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        let amount = |v: f64| (0.0..=10.0).contains(&v);
        check(
            (2..=512).contains(&self.groups),
            "groups",
            "must be between 2 and 512",
        );
        check(
            (2..=512).contains(&self.size) && self.groups * self.size <= MOST_AGENTS,
            "size",
            "must be between 2 and 512, with at most 131072 agents in all",
        );
        check(amount(self.cost), "cost", "must be between 0 and 10");
        check(
            amount(self.punish_cost),
            "punish_cost",
            "must be between 0 and 10",
        );
        check(amount(self.fine), "fine", "must be between 0 and 10");
        check(
            amount(self.fixed_cost),
            "fixed_cost",
            "must be between 0 and 10",
        );
        check(amount(self.benefit), "benefit", "must be between 0 and 10");
        check(
            (0.0..=100.0).contains(&self.baseline),
            "baseline",
            "must be between 0 and 100",
        );
        check(unit(self.error), "error", "must be between 0 and 1");
        check(unit(self.mixing), "mixing", "must be between 0 and 1");
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        check(unit(self.conflict), "conflict", "must be between 0 and 1");
        check(
            (0.0..=1000.0).contains(&self.sensitivity),
            "sensitivity",
            "must be between 0 and 1000",
        );
        check(
            (1..=1_000_000).contains(&self.window),
            "window",
            "must be between 1 and 1000000",
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
    pub(crate) fn structural_changes(&self, next: &PunishmentConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("groups", self.groups == next.groups),
            ("size", self.size == next.size),
            ("traits", self.traits == next.traits),
            ("structure", self.structure == next.structure),
            ("start", self.start == next.start),
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
        Param::integer("Groups", "groups", "Groups (N)", (2, 512), Reset)
            .with_help("Boyd and coauthors: 128."),
        Param::integer("Groups", "size", "Group size (n)", (2, 512), Reset)
            .with_help("Their figures run from 4 to 256."),
        Param::choice(
            "Groups",
            "start",
            "At the start",
            &[
                ("one_punisher_group", "One group of punishers, the rest defectors"),
                ("all_defectors", "Everyone defects"),
            ],
            Reset,
        ),
        Param::number("Game", "cost", "Cost of cooperating (c)", (0.0, 2.0, 0.01), Live),
        Param::number("Game", "fine", "Cost of being punished (p)", (0.0, 4.0, 0.01), Live)
            .with_help("A defector pays p/n for each punisher."),
        Param::choice(
            "Game",
            "punishing",
            "Punishers pay",
            &[
                ("variable", "k/n for each defector (the text)"),
                ("fixed", "A fixed cost every period (Fig. 4)"),
            ],
            Live,
        ),
        Param::number(
            "Game",
            "punish_cost",
            "Cost of punishing (k)",
            (0.0, 2.0, 0.01),
            Live,
        )
        .shown_if("punishing", "variable"),
        Param::number("Game", "fixed_cost", "Fixed cost", (0.0, 2.0, 0.01), Live)
            .shown_if("punishing", "fixed"),
        Param::number("Game", "error", "Errors (e)", (0.0, 1.0, 0.01), Live)
            .with_help("Contributors and punishers defect by mistake."),
        Param::choice(
            "Game",
            "erring",
            "A punisher who errs",
            &[
                ("others", "Still punishes the other defectors"),
                ("none", "Punishes nobody"),
                ("self", "Punishes itself too (Janssen)"),
            ],
            Live,
        ),
        Param::number("Game", "benefit", "Benefit to others (b)", (0.0, 4.0, 0.01), Live)
            .with_help("Each cooperative act gives b/n to every other member. The base model: 0."),
        Param::number("Game", "baseline", "Baseline payoff", (0.0, 10.0, 0.1), Live)
            .with_help(
                "What costs and fines are subtracted from. Never stated; 1 fits their 50-period calibration.",
            ),
        Param::number("Imitation", "mixing", "Mixing (m)", (0.0, 1.0, 0.001), Live)
            .with_help("The chance the one imitated comes from another group."),
        Param::choice(
            "Imitation",
            "imitation",
            "Imitation happens",
            &[
                ("together", "All at once"),
                ("in_turn", "In turn (Janssen)"),
            ],
            Live,
        ),
        Param::number("Imitation", "mutation", "Mutation (μ)", (0.0, 1.0, 0.001), Live),
        Param::number("Conflict", "conflict", "Conflict (ε)", (0.0, 1.0, 0.001), Live),
        Param::choice(
            "Conflict",
            "pairing",
            "Groups meet",
            &[
                ("paired", "In random pairs (the text)"),
                ("either", "In random pairs; either can start it (the figures)"),
                ("challenge", "Each challenges one (Janssen)"),
            ],
            Live,
        ),
        Param::choice(
            "Conflict",
            "victory",
            "Groups fight over",
            &[
                ("defectors", "Their share of defectors (the text)"),
                ("payoff", "Their payoffs, normalized (Cooney)"),
                ("tanh", "Their payoffs, through tanh (Cooney)"),
            ],
            Live,
        ),
        Param::number(
            "Conflict",
            "sensitivity",
            "Sensitivity (s)",
            (0.0, 100.0, 0.1),
            Live,
        )
        .shown_if("victory", "tanh"),
        Param::choice(
            "Conflict",
            "counted",
            "Defectors are counted by",
            &[
                ("types", "Type"),
                ("acts", "What they did this period (Janssen)"),
            ],
            Live,
        ),
        Param::choice(
            "Conflict",
            "refill",
            "A defeated group",
            &[
                ("copy", "Becomes a copy of the winners"),
                ("split", "Is refilled, with the winners, from the winners"),
            ],
            Live,
        ),
        Param::choice(
            "Variants",
            "traits",
            "Traits",
            &[
                ("discrete", "Contributors, defectors, punishers"),
                ("continuous", "Cooperate and punish by degrees"),
            ],
            Reset,
        ),
        Param::choice(
            "Variants",
            "structure",
            "Groups are",
            &[
                ("groups", "Anywhere, with conflict"),
                ("ring", "On a ring, without conflict"),
            ],
            Reset,
        ),
        Param::integer(
            "Stopping",
            "window",
            "Long-run window (periods)",
            (1, 1_000_000),
            Live,
        )
        .with_help("The long-run average covers this many periods before the stop."),
        Param::integer(
            "Stopping",
            "stop_at",
            "Stop at period",
            (0, 1_000_000),
            Live,
        )
        .with_help("0: never. Boyd and coauthors: 2000."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_the_base_case() {
        let c = PunishmentConfig::default();
        assert_eq!(
            (c.groups, c.size, c.stop_at, c.window),
            (128, 32, 2000, 1000)
        );
        assert_eq!((c.cost, c.punish_cost, c.fine), (0.2, 0.2, 0.8));
        assert_eq!(
            (c.error, c.mixing, c.mutation, c.conflict),
            (0.02, 0.01, 0.01, 0.015)
        );
        assert_eq!((c.benefit, c.baseline), (0.0, 1.0));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = PunishmentConfig {
            groups: 1,
            size: 600,
            cost: -1.0,
            punish_cost: 11.0,
            fine: f64::NAN,
            fixed_cost: 20.0,
            benefit: -0.5,
            baseline: 101.0,
            error: 1.5,
            mixing: -0.1,
            mutation: 2.0,
            conflict: 1.1,
            sensitivity: -1.0,
            window: 0,
            stop_at: 2_000_000,
            ..PunishmentConfig::default()
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
                "groups",
                "size",
                "cost",
                "punish_cost",
                "fine",
                "fixed_cost",
                "benefit",
                "baseline",
                "error",
                "mixing",
                "mutation",
                "conflict",
                "sensitivity",
                "window",
                "stop_at"
            ]
        );
        let crowded = PunishmentConfig {
            groups: 512,
            size: 512,
            ..PunishmentConfig::default()
        };
        assert_eq!(crowded.validate().unwrap_err()[0].field, "size");
    }

    #[test]
    fn the_population_changes_only_on_reset() {
        let next = PunishmentConfig {
            traits: Traits::Continuous,
            fine: 0.4,
            victory: Victory::Payoff,
            ..PunishmentConfig::default()
        };
        let changes = PunishmentConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "traits");
    }

    #[test]
    fn erring_self_reads_and_writes_as_self() {
        let c: PunishmentConfig = serde_json::from_str(r#"{"erring": "self"}"#).unwrap();
        assert_eq!(c.erring, Erring::Itself);
        assert!(serde_json::to_string(&c)
            .unwrap()
            .contains(r#""erring":"self""#));
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Punishment(PunishmentConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
````

Create `crates/sugarscape-core/src/punishment/stats.rs` with exactly this content:

````rust
//! Altruistic Punishment's statistics: the shares of each type, what agents
//! did this period, payoffs, conflict, how different the groups are, and the
//! long-run average Boyd and coauthors plot.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 11] = [
    "cooperation",
    "contributors",
    "punishers",
    "defectors",
    "punishment",
    "acts",
    "payoff",
    "conflicts",
    "extinctions",
    "spread",
    "long_run",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PunishmentSnapshot {
    pub tick: u64,
    /// The mean cooperation trait: contributors plus punishers.
    pub cooperation: f64,
    /// The shares of each type (continuous traits: the means of x(1 − y),
    /// xy and 1 − x).
    pub contributors: f64,
    pub punishers: f64,
    pub defectors: f64,
    /// The mean punishment trait.
    pub punishment: f64,
    /// The share who cooperated this period.
    pub acts: f64,
    /// The mean payoff this period.
    pub payoff: f64,
    /// Conflicts fought, and groups replaced, this period.
    pub conflicts: u32,
    pub extinctions: u32,
    /// The standard deviation of the groups' cooperation.
    pub spread: f64,
    /// The mean `cooperation` over the long-run window so far; NaN (null)
    /// before it.
    pub long_run: f64,
}

impl Series for PunishmentSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "cooperation" => self.cooperation,
            "contributors" => self.contributors,
            "punishers" => self.punishers,
            "defectors" => self.defectors,
            "punishment" => self.punishment,
            "acts" => self.acts,
            "payoff" => self.payoff,
            "conflicts" => f64::from(self.conflicts),
            "extinctions" => f64::from(self.extinctions),
            "spread" => self.spread,
            "long_run" => self.long_run,
            _ => return None,
        })
    }
}
````

Create `crates/sugarscape-core/src/punishment/view.rs` with exactly this content:

````rust
//! The frame: every group as a block of its agents in a mosaic, and below it
//! cooperation and punishment over time.

use crate::render::{lerp, Rgb};

/// Pixels between groups (a defeated group's frame is drawn in them).
pub const GAP: usize = 2;
/// The time strip's height, and the pixels above it.
pub const STRIP: usize = 60;
pub const STRIP_GAP: usize = 6;
/// The narrowest frame (the time strip's periods).
pub const LEAST_WIDE: usize = 301;

pub const CONTRIBUTOR: Rgb = [0x4a, 0x7c, 0xd8];
pub const PUNISHER: Rgb = [0x3c, 0xa8, 0x5a];
pub const DEFECTOR: Rgb = [0xe0, 0x3c, 0x31];
/// A punisher who defected this period.
pub const ERRED: Rgb = [0xf2, 0x9a, 0x3a];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0xf6, 0xd8, 0x6a];
pub const LOST: Rgb = [0x1a, 0x18, 0x14];
pub const MARK: Rgb = [0xd8, 0xd4, 0xca];

/// The mosaic's shape for `groups` groups of `size`: groups per row, rows,
/// cells per group row and column, and each cell's side in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mosaic {
    pub cols: usize,
    pub rows: usize,
    pub wide: usize,
    pub tall: usize,
    pub cell: usize,
}

impl Mosaic {
    pub fn new(groups: u32, size: u32) -> Self {
        let (g, n) = (groups as usize, size as usize);
        let cols = ceil_sqrt(2 * g);
        let rows = g.div_ceil(cols);
        let wide = ceil_sqrt(n);
        let tall = n.div_ceil(wide);
        let cell = (480 / (cols * wide)).clamp(1, 8);
        Mosaic {
            cols,
            rows,
            wide,
            tall,
            cell,
        }
    }

    /// A group's block, in pixels.
    pub fn block(&self) -> (usize, usize) {
        (self.wide * self.cell, self.tall * self.cell)
    }

    /// The top-left pixel of group `g`.
    pub fn origin(&self, g: usize) -> (usize, usize) {
        let (bw, bh) = self.block();
        (
            GAP + (g % self.cols) * (bw + GAP),
            GAP + (g / self.cols) * (bh + GAP),
        )
    }

    /// The mosaic's width and height.
    pub fn extent(&self) -> (usize, usize) {
        let (bw, bh) = self.block();
        (GAP + self.cols * (bw + GAP), GAP + self.rows * (bh + GAP))
    }
}

fn ceil_sqrt(v: usize) -> usize {
    let mut s = 1;
    while s * s < v {
        s += 1;
    }
    s
}

/// The time strip's row of share `x`: 1 at the top.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (STRIP - 1) as f64).round() as usize
}

/// An agent's color by its traits: red for defectors, blue for contributors,
/// green for punishers, blended for continuous traits.
pub fn traits_color(cooperate: f64, punish: f64) -> Rgb {
    lerp(
        DEFECTOR,
        lerp(CONTRIBUTOR, PUNISHER, punish.clamp(0.0, 1.0)),
        cooperate.clamp(0.0, 1.0),
    )
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_mosaic_is_16_groups_wide() {
        let m = Mosaic::new(128, 32);
        assert_eq!((m.cols, m.rows, m.wide, m.tall, m.cell), (16, 8, 6, 6, 5));
        assert_eq!(m.block(), (30, 30));
        assert_eq!(m.origin(17), (2 + 32, 2 + 32));
        assert_eq!(m.extent(), (2 + 16 * 32, 2 + 8 * 32));
        let big = Mosaic::new(128, 256);
        assert_eq!((big.wide, big.tall, big.cell), (16, 16, 1));
        let tiny = Mosaic::new(2, 2);
        assert_eq!(
            (tiny.cols, tiny.rows, tiny.wide, tiny.tall, tiny.cell),
            (2, 1, 2, 1, 8)
        );
    }

    #[test]
    fn colors_are_the_types_at_the_corners() {
        assert_eq!(traits_color(0.0, 0.0), DEFECTOR);
        assert_eq!(traits_color(0.0, 1.0), DEFECTOR);
        assert_eq!(traits_color(1.0, 0.0), CONTRIBUTOR);
        assert_eq!(traits_color(1.0, 1.0), PUNISHER);
        assert_eq!((row(1.0), row(0.0)), (0, STRIP - 1));
    }
}
````

Create `crates/sugarscape-core/src/punishment/world.rs` with exactly this content:

````rust
//! The Altruistic Punishment world. Each period: every agent cooperates or
//! defects and punishers fine the defectors; everyone imitates someone, from
//! its own group or (with probability m) another, with probability
//! Wⱼ/(Wⱼ + Wᵢ); groups meet in conflict and the losers are replaced by the
//! winners; and a few agents mutate.

use std::collections::VecDeque;
use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{
    Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, Refill, Start, Structure,
    Traits, Victory,
};
use super::stats::PunishmentSnapshot;
use super::view::{
    row, scale, traits_color, Mosaic, CONTRIBUTOR, DEFECTOR, ERRED, HIGH, LEAST_WIDE, LOST, LOW,
    MARK, PUNISHER, STRIP, STRIP_GAP,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::portable::exp_neg;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// A discrete agent's type (continuous agents have none).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Contributor,
    Defector,
    Punisher,
}

/// The three types as (cooperate, punish).
const TYPES: [(f64, f64); 3] = [(1.0, 0.0), (0.0, 0.0), (1.0, 1.0)];

fn kind(cooperate: f64, punish: f64) -> usize {
    if cooperate < 0.5 {
        1
    } else if punish < 0.5 {
        0
    } else {
        2
    }
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunishmentMode {
    Type,
    Acts,
    Payoff,
    Group,
}

impl std::str::FromStr for PunishmentMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "type" => Self::Type,
            "acts" => Self::Acts,
            "payoff" => Self::Payoff,
            "group" => Self::Group,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PunishmentInspection {
    pub site: PunishmentCell,
    /// `groups` or `time`; null elsewhere.
    pub panel: Option<&'static str>,
    /// The group under the cell, and the agent in it.
    pub group: Option<GroupView>,
    pub agent: Option<AgentView>,
    /// Time: the period and its cooperation and punishment.
    pub period: Option<u64>,
    pub cooperation: Option<f64>,
    pub punishment: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PunishmentCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub group: u32,
    /// Discrete traits: the type; continuous: null.
    pub kind: Option<Kind>,
    pub cooperate: f64,
    pub punish: f64,
    /// This period: whether it cooperated and punished, and its payoff.
    pub cooperated: bool,
    pub punished: bool,
    pub payoff: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GroupView {
    pub index: u32,
    pub contributors: f64,
    pub punishers: f64,
    pub defectors: f64,
    /// The share who cooperated this period, and the mean payoff.
    pub acts: f64,
    pub payoff: f64,
    /// The last period it fought, and whether it lost then.
    pub last_conflict: Option<u64>,
    pub lost: bool,
}

#[derive(Clone)]
pub struct PunishmentWorld {
    pub config: PunishmentConfig,
    /// Completed periods.
    pub tick: u64,
    rng: SimRng,
    /// Each agent's traits, group by group (agent a is in group a / n).
    cooperate: Vec<f64>,
    punish: Vec<f64>,
    /// This period's acts and payoffs.
    cooperated: Vec<bool>,
    punished: Vec<bool>,
    payoff: Vec<f64>,
    /// Each group's last conflict, and whether it lost it.
    fought: Vec<Option<(u64, bool)>>,
    conflicts: u32,
    extinctions: u32,
    /// The last periods' cooperation and punishment, for the time strip.
    recent: VecDeque<(f64, f64)>,
    /// The long-run window's first period, and its sum of cooperation so far.
    window_from: u64,
    window_sum: f64,
    pub stats: Stats<PunishmentSnapshot>,
}

impl PunishmentWorld {
    pub fn new(config: PunishmentConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let (g, n) = (config.groups as usize, config.size as usize);
        let mut cooperate = vec![0.0; g * n];
        let mut punish = vec![0.0; g * n];
        if config.start == Start::OnePunisherGroup {
            for a in 0..n {
                cooperate[a] = 1.0;
                punish[a] = 1.0;
            }
        }
        let mut world = PunishmentWorld {
            config,
            tick: 0,
            rng: rng::seeded(seed),
            cooperate,
            punish,
            cooperated: vec![false; g * n],
            punished: vec![false; g * n],
            payoff: vec![0.0; g * n],
            fought: vec![None; g],
            conflicts: 0,
            extinctions: 0,
            recent: VecDeque::new(),
            window_from: 0,
            window_sum: 0.0,
            stats: Stats::default(),
        };
        world.window_from = world.window_start();
        world.record();
        Ok(world)
    }

    pub fn groups(&self) -> usize {
        self.config.groups as usize
    }

    pub fn size(&self) -> usize {
        self.config.size as usize
    }

    /// Agent `a`'s traits (cooperate, punish).
    pub fn traits(&self, a: usize) -> (f64, f64) {
        (self.cooperate[a], self.punish[a])
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.act();
        let (mean_payoff, defectors) = self.payoffs();
        self.imitate();
        self.conflict(&mean_payoff, &defectors);
        self.mutate();
        self.record();
    }

    /// Who cooperates, and who punishes, this period.
    fn act(&mut self) {
        let e = self.config.error;
        for a in 0..self.cooperate.len() {
            let x = self.cooperate[a];
            let c = x > 0.0 && self.rng.gen::<f64>() < x * (1.0 - e);
            let y = self.punish[a];
            let p = y >= 1.0 || (y > 0.0 && self.rng.gen::<f64>() < y);
            self.cooperated[a] = c;
            self.punished[a] = p && (c || self.config.erring != Erring::None);
        }
    }

    /// Everyone's payoff this period; each group's mean payoff and d.
    fn payoffs(&mut self) -> (Vec<f64>, Vec<f64>) {
        let c = &self.config;
        let (g, n) = (self.groups(), self.size());
        let nf = n as f64;
        let itself = c.erring == Erring::Itself;
        let mut means = vec![0.0; g];
        let mut ds = vec![0.0; g];
        for gi in 0..g {
            let s = gi * n;
            let nc = (s..s + n).filter(|&a| self.cooperated[a]).count() as f64;
            let np = (s..s + n).filter(|&a| self.punished[a]).count() as f64;
            let nd = nf - nc;
            let mut total = 0.0;
            for a in s..s + n {
                let coop = self.cooperated[a];
                let pun = self.punished[a];
                let mut w = c.baseline + c.benefit * (nc - f64::from(u8::from(coop))) / nf;
                if coop {
                    w -= c.cost;
                } else {
                    let by = np - f64::from(u8::from(pun && !itself));
                    w -= c.fine * by / nf;
                }
                if pun {
                    match c.punishing {
                        Punishing::Variable => {
                            let of = nd - f64::from(u8::from(!coop && !itself));
                            w -= c.punish_cost * of / nf;
                        }
                        Punishing::Fixed => w -= c.fixed_cost,
                    }
                }
                let w = w.max(0.0);
                self.payoff[a] = w;
                total += w;
            }
            means[gi] = total / nf;
            ds[gi] = match c.counted {
                Counted::Types => (s..s + n).map(|a| 1.0 - self.cooperate[a]).sum::<f64>() / nf,
                Counted::Acts => nd / nf,
            };
        }
        (means, ds)
    }

    /// Payoff-biased imitation, within groups and (with probability m) across.
    fn imitate(&mut self) {
        let (g, n) = (self.groups(), self.size());
        let m = self.config.mixing;
        let ring = self.config.structure == Structure::Ring;
        let together = self.config.imitation == Imitation::Together;
        let (from_c, from_p) = if together {
            (self.cooperate.clone(), self.punish.clone())
        } else {
            (Vec::new(), Vec::new())
        };
        for a in 0..g * n {
            let gi = a / n;
            let j = if self.rng.gen::<f64>() >= m {
                let k = self.rng.gen_range(0..n as u32 - 1) as usize;
                let own = a - gi * n;
                gi * n + if k >= own { k + 1 } else { k }
            } else {
                let h = if ring {
                    if self.rng.gen::<f64>() < 0.5 {
                        (gi + 1) % g
                    } else {
                        (gi + g - 1) % g
                    }
                } else {
                    let k = self.rng.gen_range(0..g as u32 - 1) as usize;
                    if k >= gi {
                        k + 1
                    } else {
                        k
                    }
                };
                h * n + self.rng.gen_range(0..n as u32) as usize
            };
            let (wi, wj) = (self.payoff[a], self.payoff[j]);
            let chance = if wi + wj > 0.0 { wj / (wi + wj) } else { 0.5 };
            if self.rng.gen::<f64>() < chance {
                if together {
                    self.cooperate[a] = from_c[j];
                    self.punish[a] = from_p[j];
                } else {
                    self.cooperate[a] = self.cooperate[j];
                    self.punish[a] = self.punish[j];
                }
            }
        }
    }

    /// The chance group i beats group j.
    fn victory(&self, i: usize, j: usize, means: &[f64], ds: &[f64]) -> f64 {
        let c = &self.config;
        let p = match c.victory {
            Victory::Defectors => 0.5 * (1.0 + ds[j] - ds[i]),
            Victory::Payoff => {
                let range = payoff_range(c);
                if range > 0.0 {
                    0.5 + 0.5 * (means[i] - means[j]) / range
                } else {
                    0.5
                }
            }
            Victory::Tanh => 0.5 + 0.5 * tanh(c.sensitivity * (means[i] - means[j])),
        };
        p.clamp(0.0, 1.0)
    }

    /// Groups meet; the losers are replaced by the winners.
    fn conflict(&mut self, means: &[f64], ds: &[f64]) {
        self.conflicts = 0;
        self.extinctions = 0;
        if self.config.structure == Structure::Ring {
            return;
        }
        let g = self.groups();
        let eps = self.config.conflict;
        let mut order: Vec<usize> = (0..g).collect();
        for i in (1..g).rev() {
            let j = self.rng.gen_range(0..=i as u32) as usize;
            order.swap(i, j);
        }
        let mut pairs = Vec::new();
        match self.config.pairing {
            Pairing::Paired => {
                for pair in order.chunks_exact(2) {
                    if self.rng.gen::<f64>() < eps {
                        pairs.push((pair[0], pair[1]));
                    }
                }
            }
            Pairing::Either => {
                for pair in order.chunks_exact(2) {
                    let (a, b) = (self.rng.gen::<f64>(), self.rng.gen::<f64>());
                    if a < eps || b < eps {
                        pairs.push((pair[0], pair[1]));
                    }
                }
            }
            Pairing::Challenge => {
                let mut busy = vec![false; g];
                for &i in &order {
                    if busy[i] || self.rng.gen::<f64>() >= eps {
                        continue;
                    }
                    let free: Vec<usize> = (0..g).filter(|&h| h != i && !busy[h]).collect();
                    if free.is_empty() {
                        continue;
                    }
                    let h = free[self.rng.gen_range(0..free.len() as u32) as usize];
                    busy[i] = true;
                    busy[h] = true;
                    pairs.push((i, h));
                }
            }
        }
        for (i, j) in pairs {
            let win_i = self.rng.gen::<f64>() < self.victory(i, j, means, ds);
            let (win, lose) = if win_i { (i, j) } else { (j, i) };
            self.replace(win, lose);
            self.fought[win] = Some((self.tick, false));
            self.fought[lose] = Some((self.tick, true));
            self.conflicts += 1;
            self.extinctions += 1;
        }
    }

    fn replace(&mut self, win: usize, lose: usize) {
        let n = self.size();
        match self.config.refill {
            Refill::Copy => {
                for k in 0..n {
                    self.cooperate[lose * n + k] = self.cooperate[win * n + k];
                    self.punish[lose * n + k] = self.punish[win * n + k];
                }
            }
            Refill::Split => {
                let c: Vec<f64> = self.cooperate[win * n..win * n + n].to_vec();
                let p: Vec<f64> = self.punish[win * n..win * n + n].to_vec();
                for site in [win, lose] {
                    for k in 0..n {
                        let s = self.rng.gen_range(0..n as u32) as usize;
                        self.cooperate[site * n + k] = c[s];
                        self.punish[site * n + k] = p[s];
                    }
                }
            }
        }
    }

    fn mutate(&mut self) {
        let mu = self.config.mutation;
        let continuous = self.config.traits == Traits::Continuous;
        for a in 0..self.cooperate.len() {
            if self.rng.gen::<f64>() >= mu {
                continue;
            }
            if continuous {
                self.cooperate[a] = self.rng.gen::<f64>();
                self.punish[a] = self.rng.gen::<f64>();
            } else {
                let now = kind(self.cooperate[a], self.punish[a]);
                let next = (now + 1 + self.rng.gen_range(0..2u32) as usize) % 3;
                (self.cooperate[a], self.punish[a]) = TYPES[next];
            }
        }
    }

    /// The long-run window's first period.
    fn window_start(&self) -> u64 {
        let (stop, window) = (
            u64::from(self.config.stop_at),
            u64::from(self.config.window),
        );
        if stop > window {
            stop - window + 1
        } else {
            1
        }
    }

    fn record(&mut self) {
        let (g, n) = (self.groups(), self.size());
        let total = (g * n) as f64;
        let (mut coop, mut pun, mut contributors, mut punishers, mut defectors) =
            (0.0, 0.0, 0.0, 0.0, 0.0);
        for a in 0..g * n {
            let (x, y) = (self.cooperate[a], self.punish[a]);
            coop += x;
            pun += y;
            contributors += x * (1.0 - y);
            punishers += x * y;
            defectors += 1.0 - x;
        }
        let cooperation = coop / total;
        let group_coop: Vec<f64> = (0..g)
            .map(|gi| self.cooperate[gi * n..gi * n + n].iter().sum::<f64>() / n as f64)
            .collect();
        let mean = group_coop.iter().sum::<f64>() / g as f64;
        let spread = (group_coop
            .iter()
            .map(|v| (v - mean) * (v - mean))
            .sum::<f64>()
            / g as f64)
            .sqrt();
        // The long-run window moves when `stop_at` or `window` changes; its
        // sum is then rebuilt from the history.
        let from = self.window_start();
        if from != self.window_from {
            self.window_from = from;
            self.window_sum = self
                .stats
                .history()
                .iter()
                .filter(|s| s.tick >= from)
                .map(|s| s.cooperation)
                .sum();
        }
        let long_run = if self.tick >= from {
            self.window_sum += cooperation;
            self.window_sum / (self.tick - from + 1) as f64
        } else {
            f64::NAN
        };
        let acts = if self.tick == 0 {
            f64::NAN
        } else {
            self.cooperated.iter().filter(|&&c| c).count() as f64 / total
        };
        let payoff = if self.tick == 0 {
            f64::NAN
        } else {
            self.payoff.iter().sum::<f64>() / total
        };
        let wide = self.frame_wide();
        if self.recent.len() == wide {
            self.recent.pop_front();
        }
        self.recent.push_back((cooperation, pun / total));
        self.stats.push(PunishmentSnapshot {
            tick: self.tick,
            cooperation,
            contributors: contributors / total,
            punishers: punishers / total,
            defectors: defectors / total,
            punishment: pun / total,
            acts,
            payoff,
            conflicts: self.conflicts,
            extinctions: self.extinctions,
            spread,
            long_run,
        });
    }

    fn mosaic(&self) -> Mosaic {
        Mosaic::new(self.config.groups, self.config.size)
    }

    /// The frame's width: the mosaic, or the time strip if wider.
    fn frame_wide(&self) -> usize {
        self.mosaic().extent().0.max(LEAST_WIDE)
    }

    fn view(&self, a: usize) -> AgentView {
        let (x, y) = self.traits(a);
        AgentView {
            id: a as u64 + 1,
            group: (a / self.size()) as u32,
            kind: (self.config.traits == Traits::Discrete)
                .then(|| [Kind::Contributor, Kind::Defector, Kind::Punisher][kind(x, y)]),
            cooperate: x,
            punish: y,
            cooperated: self.cooperated[a],
            punished: self.punished[a],
            payoff: self.payoff[a],
        }
    }

    fn group_view(&self, gi: usize) -> GroupView {
        let n = self.size();
        let nf = n as f64;
        let range = gi * n..gi * n + n;
        let (mut c, mut p, mut d) = (0.0, 0.0, 0.0);
        for a in range.clone() {
            let (x, y) = self.traits(a);
            c += x * (1.0 - y);
            p += x * y;
            d += 1.0 - x;
        }
        GroupView {
            index: gi as u32,
            contributors: c / nf,
            punishers: p / nf,
            defectors: d / nf,
            acts: range.clone().filter(|&a| self.cooperated[a]).count() as f64 / nf,
            payoff: range.map(|a| self.payoff[a]).sum::<f64>() / nf,
            last_conflict: self.fought[gi].map(|f| f.0),
            lost: self.fought[gi].is_some_and(|f| f.1),
        }
    }

    fn color(&self, mode: PunishmentMode, a: usize, group_d: f64) -> [u8; 3] {
        let (x, y) = self.traits(a);
        match mode {
            PunishmentMode::Type => traits_color(x, y),
            PunishmentMode::Acts => match (self.cooperated[a], self.punished[a]) {
                (true, true) => PUNISHER,
                (true, false) => CONTRIBUTOR,
                (false, true) => ERRED,
                (false, false) => DEFECTOR,
            },
            PunishmentMode::Payoff => {
                let top = (self.config.baseline + self.config.benefit).max(f64::MIN_POSITIVE);
                scale(self.payoff[a] / top, LOW, HIGH)
            }
            PunishmentMode::Group => scale(group_d, CONTRIBUTOR, DEFECTOR),
        }
    }

    /// The agent at pixel (x, y) of the mosaic, if any.
    fn agent_at(&self, x: usize, y: usize) -> Option<usize> {
        let m = self.mosaic();
        let (bw, bh) = m.block();
        for gi in 0..self.groups() {
            let (ox, oy) = m.origin(gi);
            if (ox..ox + bw).contains(&x) && (oy..oy + bh).contains(&y) {
                let k = (y - oy) / m.cell * m.wide + (x - ox) / m.cell;
                return (k < self.size()).then_some(gi * self.size() + k);
            }
        }
        None
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<PunishmentInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let mut out = PunishmentInspection {
            site: PunishmentCell { x, y },
            panel: None,
            group: None,
            agent: None,
            period: None,
            cooperation: None,
            punishment: None,
        };
        let (cx, cy) = (x as usize, y as usize);
        let (_, mh) = self.mosaic().extent();
        if cy < mh {
            if let Some(a) = self.agent_at(cx, cy) {
                out.panel = Some("groups");
                out.agent = Some(self.view(a));
                out.group = Some(self.group_view(a / self.size()));
            }
        } else if cy >= mh + STRIP_GAP {
            if let Some(&(coop, pun)) = self.recent.get(cx) {
                out.panel = Some("time");
                out.period = Some(self.tick + 1 + cx as u64 - self.recent.len() as u64);
                out.cooperation = Some(coop);
                out.punishment = Some(pun);
            }
        }
        Ok(out)
    }
}

/// tanh from `+ − × ÷` and `exp_neg` (portable).
fn tanh(u: f64) -> f64 {
    let e = exp_neg(-2.0 * u.abs());
    ((1.0 - e) / (1.0 + e)).copysign(u)
}

/// The widest possible difference between two groups' expected mean payoffs
/// (Cooney's G_max − G_min). With y defectors and z punishers the expected
/// mean is baseline + (b − c)(1 − y) − a·y·z − q·z (a = p + k and q = 0 for
/// variable costs; a = p and q the fixed cost for fixed ones): linear in z,
/// so its extremes lie at the corners or on the edge z = 1 − y, a quadratic
/// in y with one stationary point.
pub fn payoff_range(c: &PunishmentConfig) -> f64 {
    let (a, q) = match c.punishing {
        Punishing::Variable => (c.fine + c.punish_cost, 0.0),
        Punishing::Fixed => (c.fine, c.fixed_cost),
    };
    let g = |y: f64, z: f64| (c.benefit - c.cost) * (1.0 - y) - a * y * z - q * z;
    let mut points = vec![g(0.0, 0.0), g(1.0, 0.0), g(0.0, 1.0)];
    if a > 0.0 {
        let y = (a + c.benefit - c.cost - q) / (2.0 * a);
        if (0.0..=1.0).contains(&y) {
            points.push(g(y, 1.0 - y));
        }
    }
    let hi = points.iter().copied().fold(f64::MIN, f64::max);
    let lo = points.iter().copied().fold(f64::MAX, f64::min);
    hi - lo
}

impl Model for PunishmentWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Punishment(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        PunishmentWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.cooperate.len()
    }

    /// FNV-1a over the tick and every agent's traits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        for a in 0..self.cooperate.len() {
            eat(&self.cooperate[a].to_bits().to_le_bytes());
            eat(&self.punish[a].to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (_, mh) = self.mosaic().extent();
        (self.frame_wide() as u32, (mh + STRIP_GAP + STRIP) as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: PunishmentMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let m = self.mosaic();
        let (bw, bh) = m.block();
        let n = self.size();
        for gi in 0..self.groups() {
            let (ox, oy) = m.origin(gi);
            let d = (gi * n..gi * n + n)
                .map(|a| 1.0 - self.cooperate[a])
                .sum::<f64>()
                / n as f64;
            if self.fought[gi] == Some((self.tick, true)) {
                for x in ox - 1..=ox + bw {
                    c.put(x, oy - 1, LOST);
                    c.put(x, oy + bh, LOST);
                }
                c.column(ox - 1, oy - 1, oy + bh, LOST);
                c.column(ox + bw, oy - 1, oy + bh, LOST);
            }
            for k in 0..n {
                let color = self.color(mode, gi * n + k, d);
                let (px, py) = (ox + (k % m.wide) * m.cell, oy + (k / m.wide) * m.cell);
                for dy in 0..m.cell {
                    for dx in 0..m.cell {
                        c.put(px + dx, py + dy, color);
                    }
                }
            }
        }
        // Cooperation (blue) and punishment (green) over the last periods.
        let top = m.extent().1 + STRIP_GAP;
        for x in 0..fw as usize {
            c.put(x, top + row(0.5), MARK);
        }
        for (k, &(coop, pun)) in self.recent.iter().enumerate() {
            let next = self.recent.get(k + 1).copied().unwrap_or((coop, pun));
            c.column(k, top + row(pun), top + row(next.1), PUNISHER);
            c.column(k, top + row(coop), top + row(next.0), CONTRIBUTOR);
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
        let mut out = String::from("id,group,kind,cooperate,punish,cooperated,punished,payoff\n");
        for a in 0..self.cooperate.len() {
            let v = self.view(a);
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                v.id,
                v.group,
                v.kind
                    .map_or(String::new(), |k| format!("{k:?}").to_lowercase()),
                v.cooperate,
                v.punish,
                u8::from(v.cooperated),
                u8::from(v.punished),
                v.payoff
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Agents stay where they are: agent `id`'s cell in its group's block.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let a = usize::try_from(id.checked_sub(1)?).ok()?;
        if a >= self.cooperate.len() {
            return None;
        }
        let m = self.mosaic();
        let (ox, oy) = m.origin(a / self.size());
        let k = a % self.size();
        Some((
            (ox + (k % m.wide) * m.cell) as u32,
            (oy + (k / m.wide) * m.cell) as u32,
        ))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Punishment(next) = next else {
            return Err(wrong_model(ModelKind::Punishment, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped at `stop_at`: a sweep reads its last period (the long-run average).
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(edit: impl FnOnce(&mut PunishmentConfig)) -> PunishmentConfig {
        let mut c = PunishmentConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut PunishmentConfig)) -> PunishmentWorld {
        PunishmentWorld::new(config(edit), 1).unwrap()
    }

    /// A world whose first group holds `types` (0 contributor, 1 defector,
    /// 2 punisher) and whose acts are set by hand.
    fn hand(
        edit: impl FnOnce(&mut PunishmentConfig),
        types: &[usize],
        acts: &[bool],
    ) -> PunishmentWorld {
        let mut w = world(|c| {
            c.groups = 2;
            c.size = types.len() as u32;
            edit(c);
        });
        for (a, &t) in types.iter().enumerate() {
            (w.cooperate[a], w.punish[a]) = TYPES[t];
            w.cooperated[a] = acts[a];
            let punisher = t == 2 && (acts[a] || w.config.erring != Erring::None);
            w.punished[a] = punisher;
        }
        w
    }

    #[test]
    fn it_starts_with_one_group_of_punishers() {
        let w = world(|_| {});
        let n = w.size();
        assert!((0..n).all(|a| w.traits(a) == (1.0, 1.0)));
        assert!((n..w.cooperate.len()).all(|a| w.traits(a) == (0.0, 0.0)));
        let s = w.stats.latest().unwrap();
        assert_eq!(s.cooperation, 1.0 / 128.0);
        assert!(s.long_run.is_nan() && s.acts.is_nan());
        let all = world(|c| c.start = Start::AllDefectors);
        assert_eq!(all.stats.latest().unwrap().cooperation, 0.0);
    }

    #[test]
    fn payoffs_follow_the_game() {
        // A contributor, a defector, two punishers; everyone acts to type.
        let mut w = hand(|_| {}, &[0, 1, 2, 2], &[true, false, true, true]);
        let (means, ds) = w.payoffs();
        let n = 4.0;
        let p = &w.payoff;
        assert!((p[0] - (1.0 - 0.2)).abs() < 1e-12);
        assert!((p[1] - (1.0 - 0.8 * 2.0 / n)).abs() < 1e-12);
        assert!((p[2] - (1.0 - 0.2 - 0.2 / n)).abs() < 1e-12);
        assert!((means[0] - p[..4].iter().sum::<f64>() / n).abs() < 1e-12);
        assert!((ds[0] - 0.25).abs() < 1e-12);
        // With a benefit, each cooperator gives b/n to every other member.
        let mut b = hand(
            |c| c.benefit = 0.4,
            &[0, 1, 2, 2],
            &[true, false, true, true],
        );
        b.payoffs();
        assert!((b.payoff[1] - (1.0 + 0.4 * 3.0 / n - 0.8 * 2.0 / n)).abs() < 1e-12);
        assert!((b.payoff[0] - (1.0 + 0.4 * 2.0 / n - 0.2)).abs() < 1e-12);
        // A fixed cost is paid whatever the group.
        let mut f = hand(
            |c| c.punishing = Punishing::Fixed,
            &[0, 0, 2, 2],
            &[true; 4],
        );
        f.payoffs();
        assert!((f.payoff[2] - (1.0 - 0.2 - 0.2)).abs() < 1e-12);
        // Payoffs never fall below 0.
        let mut z = hand(|c| c.fine = 10.0, &[1, 2, 2, 2], &[false, true, true, true]);
        z.payoffs();
        assert_eq!(z.payoff[0], 0.0);
    }

    #[test]
    fn a_punisher_who_errs_follows_its_rule() {
        // Two punishers; the first defects by mistake.
        let types = [2, 2, 1, 0];
        let acts = [false, true, false, true];
        let n = 4.0;
        let mut others = hand(|_| {}, &types, &acts);
        others.payoffs();
        // It punishes the other defector and is punished by the other punisher.
        assert!((others.payoff[0] - (1.0 - 0.8 / n - 0.2 / n)).abs() < 1e-12);
        assert!((others.payoff[2] - (1.0 - 0.8 * 2.0 / n)).abs() < 1e-12);
        let mut none = hand(|c| c.erring = Erring::None, &types, &acts);
        none.payoffs();
        assert!((none.payoff[0] - (1.0 - 0.8 / n)).abs() < 1e-12);
        assert!((none.payoff[2] - (1.0 - 0.8 / n)).abs() < 1e-12);
        let mut itself = hand(|c| c.erring = Erring::Itself, &types, &acts);
        itself.payoffs();
        assert!((itself.payoff[0] - (1.0 - 0.8 * 2.0 / n - 0.2 * 2.0 / n)).abs() < 1e-12);
    }

    #[test]
    fn imitation_copies_with_the_payoff_ratio() {
        // Two agents per group; agent 0 earns 1, agent 1 earns 3: agent 0
        // copies agent 1 three times in four.
        let mut copied = 0;
        for seed in 0..4000 {
            let mut w = PunishmentWorld::new(
                config(|c| {
                    c.groups = 2;
                    c.size = 2;
                    c.mixing = 0.0;
                }),
                seed,
            )
            .unwrap();
            (w.cooperate[0], w.punish[0]) = (0.0, 0.0);
            (w.cooperate[1], w.punish[1]) = (1.0, 1.0);
            w.payoff[..2].copy_from_slice(&[1.0, 3.0]);
            w.imitate();
            if w.cooperate[0] == 1.0 {
                copied += 1;
            }
        }
        assert!((2850..=3150).contains(&copied), "{copied} of 4000");
    }

    #[test]
    fn together_imitates_from_the_starting_traits_and_in_turn_does_not() {
        // A group of a defector and a punisher, earning the same: together,
        // both can copy the other's old type and swap; in turn, the second
        // copies the first's new type, so they never swap.
        let swaps = |imitation| {
            (0..400)
                .filter(|&seed| {
                    let mut w = PunishmentWorld::new(
                        config(|c| {
                            c.groups = 2;
                            c.size = 2;
                            c.mixing = 0.0;
                            c.imitation = imitation;
                        }),
                        seed,
                    )
                    .unwrap();
                    (w.cooperate[0], w.punish[0]) = TYPES[1];
                    (w.cooperate[1], w.punish[1]) = TYPES[2];
                    w.payoff[..2].copy_from_slice(&[1.0, 1.0]);
                    w.imitate();
                    kind(w.cooperate[0], w.punish[0]) == 2 && kind(w.cooperate[1], w.punish[1]) == 1
                })
                .count()
        };
        let together = swaps(Imitation::Together);
        assert!((70..=130).contains(&together), "{together} of 400");
        assert_eq!(swaps(Imitation::InTurn), 0);
    }

    #[test]
    fn migrants_come_from_other_groups_and_on_a_ring_from_neighbors() {
        for seed in 0..20 {
            // Group 0: a defector earning 0 beside a punisher earning 100;
            // every other group contributors earning 1. With m = 1 the
            // defector copies a contributor, never its own group's punisher.
            let mut w = PunishmentWorld::new(
                config(|c| {
                    c.groups = 6;
                    c.size = 2;
                    c.mixing = 1.0;
                }),
                seed,
            )
            .unwrap();
            for a in 0..12 {
                (w.cooperate[a], w.punish[a]) = TYPES[if a == 0 {
                    1
                } else if a == 1 {
                    2
                } else {
                    0
                }];
                w.payoff[a] = match a {
                    0 => 0.0,
                    1 => 100.0,
                    _ => 1.0,
                };
            }
            w.imitate();
            assert_eq!(kind(w.cooperate[0], w.punish[0]), 0);
            // On a ring, group 0's neighbors (1 and 5) are punishers and the
            // rest contributors: its defectors become punishers.
            let mut r = PunishmentWorld::new(
                config(|c| {
                    c.groups = 6;
                    c.size = 2;
                    c.mixing = 1.0;
                    c.structure = Structure::Ring;
                }),
                seed,
            )
            .unwrap();
            for a in 0..12 {
                let t = match a / 2 {
                    0 => 1,
                    1 | 5 => 2,
                    _ => 0,
                };
                (r.cooperate[a], r.punish[a]) = TYPES[t];
                r.payoff[a] = if a < 2 { 0.0 } else { 1.0 };
            }
            r.imitate();
            assert_eq!(kind(r.cooperate[0], r.punish[0]), 2);
            assert_eq!(kind(r.cooperate[1], r.punish[1]), 2);
        }
    }

    #[test]
    fn paired_groups_fight_at_half_the_rate_and_either_or_challenge_about_the_whole() {
        let rate = |pairing| {
            let mut w = world(|c| {
                c.pairing = pairing;
                c.conflict = 0.1;
                c.groups = 64;
                c.size = 2;
            });
            let (means, ds) = (vec![1.0; 64], vec![0.5; 64]);
            let mut fights = 0;
            for _ in 0..200 {
                w.conflict(&means, &ds);
                fights += w.conflicts;
            }
            f64::from(fights) / (200.0 * 64.0)
        };
        let paired = rate(Pairing::Paired);
        let either = rate(Pairing::Either);
        let challenge = rate(Pairing::Challenge);
        assert!((paired - 0.05).abs() < 0.01, "{paired}");
        assert!((either - 0.095).abs() < 0.012, "{either}");
        assert!((challenge - 0.09).abs() < 0.015, "{challenge}");
    }

    #[test]
    fn victory_follows_each_rule() {
        let w = world(|_| {});
        let (means, ds) = (vec![1.0, 0.5], vec![0.2, 0.6]);
        assert!((w.victory(0, 1, &means, &ds) - 0.7).abs() < 1e-12);
        let p = world(|c| c.victory = Victory::Payoff);
        let range = payoff_range(&p.config);
        assert!((range - 0.36).abs() < 1e-12);
        let close = [1.0, 0.95];
        assert!((p.victory(0, 1, &close, &ds) - (0.5 + 0.025 / range)).abs() < 1e-12);
        assert_eq!(p.victory(0, 1, &means, &ds), 1.0);
        let t = world(|c| {
            c.victory = Victory::Tanh;
            c.sensitivity = 2.0;
        });
        assert!((t.victory(0, 1, &means, &ds) - (0.5 + 0.5 * 1f64.tanh())).abs() < 1e-12);
        assert!((t.victory(1, 0, &means, &ds) - (0.5 - 0.5 * 1f64.tanh())).abs() < 1e-12);
        // No possible difference: a draw.
        let flat = world(|c| {
            c.victory = Victory::Payoff;
            c.cost = 0.0;
            c.fine = 0.0;
            c.punish_cost = 0.0;
        });
        assert_eq!(payoff_range(&flat.config), 0.0);
        assert_eq!(flat.victory(0, 1, &means, &ds), 0.5);
    }

    #[test]
    fn the_payoff_range_matches_a_grid() {
        for (b, p, k, fixed) in [
            (0.0, 0.8, 0.2, false),
            (0.4, 0.8, 0.2, false),
            (1.6, 0.4, 0.2, true),
            (0.8, 2.4, 0.05, false),
        ] {
            let c = config(|c| {
                c.benefit = b;
                c.fine = p;
                c.punish_cost = k;
                if fixed {
                    c.punishing = Punishing::Fixed;
                    c.fixed_cost = 0.1;
                }
            });
            let (a, q) = if fixed { (p, 0.1) } else { (p + k, 0.0) };
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for i in 0..=400 {
                for j in 0..=(400 - i) {
                    let (y, z) = (i as f64 / 400.0, j as f64 / 400.0);
                    let g = (b - 0.2) * (1.0 - y) - a * y * z - q * z;
                    lo = lo.min(g);
                    hi = hi.max(g);
                }
            }
            assert!((payoff_range(&c) - (hi - lo)).abs() < 1e-4, "{b} {p} {k}");
        }
    }

    #[test]
    fn losers_become_the_winners() {
        let mut w = world(|c| {
            c.groups = 4;
            c.size = 3;
        });
        for a in 0..3 {
            (w.cooperate[a], w.punish[a]) = TYPES[[0, 2, 2][a]];
        }
        w.replace(0, 2);
        assert_eq!(&w.cooperate[6..9], &w.cooperate[0..3]);
        assert_eq!(&w.punish[6..9], &w.punish[0..3]);
        let mut s = world(|c| {
            c.groups = 4;
            c.size = 3;
            c.refill = Refill::Split;
        });
        for a in 0..3 {
            (s.cooperate[a], s.punish[a]) = TYPES[[0, 2, 2][a]];
        }
        s.replace(0, 2);
        for a in (0..3).chain(6..9) {
            assert!(matches!(kind(s.cooperate[a], s.punish[a]), 0 | 2));
        }
    }

    #[test]
    fn mutants_switch_to_another_type_or_draw_new_traits() {
        let mut w = world(|c| {
            c.mutation = 1.0;
            c.groups = 2;
            c.size = 50;
            c.start = Start::AllDefectors;
        });
        w.mutate();
        let kinds: Vec<usize> = (0..100)
            .map(|a| kind(w.cooperate[a], w.punish[a]))
            .collect();
        assert!(kinds.iter().all(|&k| k != 1));
        assert!(kinds.contains(&0) && kinds.contains(&2));
        let mut c = world(|c| {
            c.mutation = 1.0;
            c.traits = Traits::Continuous;
        });
        c.mutate();
        assert!(c.cooperate.iter().all(|x| (0.0..1.0).contains(x)));
        assert!(c.cooperate.iter().any(|&x| x > 0.0 && x < 1.0));
    }

    #[test]
    fn the_long_run_average_covers_its_window() {
        let mut w = world(|c| {
            c.stop_at = 40;
            c.window = 10;
            c.groups = 8;
            c.size = 4;
        });
        w.run(100);
        assert_eq!(w.tick, 40);
        assert!(w.is_finished());
        let coop = w.series("cooperation").unwrap();
        let want = coop[31..=40].iter().sum::<f64>() / 10.0;
        assert!((w.latest_value("long_run").unwrap() - want).abs() < 1e-12);
        assert!(w.series("long_run").unwrap()[30].is_nan());
        // Moving the window rebuilds the average from the history.
        let mut next = w.config.clone();
        next.stop_at = 50;
        next.window = 30;
        Model::set_config(&mut w, ModelConfig::Punishment(next)).unwrap();
        w.run(10);
        let coop = w.series("cooperation").unwrap();
        let want = coop[21..=50].iter().sum::<f64>() / 30.0;
        assert!((w.latest_value("long_run").unwrap() - want).abs() < 1e-12);
    }

    #[test]
    fn punishment_sustains_cooperation_that_its_absence_loses() {
        let base = |c: &mut PunishmentConfig| {
            c.groups = 64;
            c.size = 16;
            c.stop_at = 1000;
        };
        let with = world(base);
        let without = world(|c| {
            base(c);
            c.fine = 0.0;
            c.punish_cost = 0.0;
        });
        let (mut a, mut b) = (with, without);
        a.run(1000);
        b.run(1000);
        let (x, y) = (
            a.latest_value("cooperation").unwrap(),
            b.latest_value("cooperation").unwrap(),
        );
        assert!(x > 0.5 && y < 0.3, "{x} against {y}");
    }

    #[test]
    fn stats_add_up() {
        let mut w = world(|c| {
            c.groups = 16;
            c.size = 8;
            c.traits = Traits::Continuous;
            c.mutation = 0.2;
        });
        w.run(20);
        let s = w.stats.latest().unwrap().clone();
        assert!((s.contributors + s.punishers + s.defectors - 1.0).abs() < 1e-9);
        assert!((s.contributors + s.punishers - s.cooperation).abs() < 1e-9);
        assert!(s.spread >= 0.0 && s.payoff > 0.0 && (0.0..=1.0).contains(&s.acts));
        assert_eq!(s.conflicts, s.extinctions);
    }

    #[test]
    fn the_view_and_inspect_read_agents_groups_and_periods() {
        let mut w = world(|_| {});
        w.run(5);
        let mut buf = Vec::new();
        for mode in ["type", "acts", "payoff", "group"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw, fh), (514, 258 + 6 + 60));
        let (x, y) = Model::locate(&w, 33).unwrap();
        assert_eq!((x, y), (34, 2));
        let i = w.inspect(x, y).unwrap();
        assert_eq!(i.panel, Some("groups"));
        assert_eq!(i.agent.as_ref().unwrap().id, 33);
        assert_eq!(i.group.as_ref().unwrap().index, 1);
        let t = w.inspect(3, 258 + 6 + 10).unwrap();
        assert_eq!((t.panel, t.period), (Some("time"), Some(3)));
        assert_eq!(w.inspect(0, 0).unwrap().panel, None);
        assert!(w.inspect(fw, 0).is_err());
        assert_eq!(Model::locate(&w, 0), None);
        assert_eq!(Model::locate(&w, 128 * 32 + 1), None);
    }

    #[test]
    fn tanh_is_portable_and_right() {
        for u in [-5.0, -1.0, -0.1, 0.0, 0.3, 2.0, 30.0] {
            assert!((tanh(u) - f64::tanh(u)).abs() < 1e-14, "{u}");
        }
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let mut next = w.config.clone();
        next.fine = 0.4;
        next.victory = Victory::Tanh;
        Model::set_config(&mut w, ModelConfig::Punishment(next.clone())).unwrap();
        assert_eq!(w.config.fine, 0.4);
        next.size = 16;
        assert!(Model::set_config(&mut w, ModelConfig::Punishment(next)).is_err());
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for edit in [
            (|c: &mut PunishmentConfig| {
                c.groups = 2;
                c.size = 2;
            }) as fn(&mut PunishmentConfig),
            |c| {
                c.error = 1.0;
                c.mixing = 1.0;
                c.mutation = 1.0;
                c.conflict = 1.0;
            },
            |c| {
                c.error = 0.0;
                c.mixing = 0.0;
                c.mutation = 0.0;
                c.conflict = 0.0;
            },
            |c| c.baseline = 0.0,
            |c| {
                c.fine = 0.0;
                c.cost = 0.0;
                c.punish_cost = 0.0;
                c.victory = Victory::Payoff;
            },
            |c| {
                c.structure = Structure::Ring;
                c.groups = 2;
            },
            |c| {
                c.pairing = Pairing::Challenge;
                c.conflict = 1.0;
                c.groups = 3;
                c.refill = Refill::Split;
            },
        ] {
            let mut w = world(|c| {
                c.groups = c.groups.min(16);
                c.size = c.size.min(8);
                edit(c);
            });
            w.run(30);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.cooperation));
        }
    }
}
````

Create `crates/sugarscape-core/src/punishment/presets.rs` with exactly this content:

````rust
//! Boyd, Gintis, Bowles and Richerson's base case and figures, the variants
//! their text describes, and Janssen's readings of what it leaves open.

use super::config::{
    Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, Structure, Traits, Victory,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const BGBR: &str = "Boyd, Gintis, Bowles & Richerson 2003, PNAS 100: 3531";
const JANSSEN: &str = "Janssen's replication, CoMSES 2223 (readings only)";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut PunishmentConfig),
) -> ModelPreset {
    let mut c = PunishmentConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Punishment(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "bg-base",
            "The base case, groups of 32",
            BGBR,
            "Boyd, Gintis, Bowles and Richerson's model: 128 groups of 32. Each period contributors and punishers cooperate at a cost c = 0.2 (erring 2 % of the time), defectors do not, and each punisher fines every defector p/n = 0.8/n at a cost k/n = 0.2/n to itself. Then everyone meets someone, from another group with probability m = 0.01, and copies them with probability Wⱼ/(Wⱼ + Wᵢ); groups are paired at random and each pair fights with probability ε = 0.015, the group with fewer defectors more likely to win and replace the loser; 1 % mutate. It starts with one group of punishers among 127 of defectors. The payoff costs are subtracted from is never stated; here 1, which fits their calibration (a trait with advantage c spreads from 10 % to 90 % in about 40 periods). Colors: contributors blue, punishers green, defectors red. Measured (10 seeds, the last 1 000 of 2 000 periods): 69 % cooperate (the figure: 83 %), 44 % are punishers. The stated model's cooperation holds to groups of about 32; the figure's reaches 256 — see bg-either.",
            |_| {},
        ),
        preset(
            "bg-either",
            "Fig. 1b as plotted: either group starts a conflict",
            BGBR,
            "The base case, with one reading changed: 'groups are paired at random, and with probability ε, intergroup conflict results' — either group of a pair can start the conflict, each with probability ε, so a pair fights with probability 2ε − ε², about twice the text's. Their Methods derive ε from an extinction rate of 0.0075 as if pairs fought at ε; the figures fit this reading instead. Measured (10 seeds, the last 1 000 of 2 000 periods): 84 % cooperate at n 32 (the figure: 83 %); across Fig. 1's six curves the mean gap to the figure is 0.010–0.033 (bg-fig1-either), and it reproduces Figs. 2 and 3 as well — all but Fig. 4's fixed cost, which falls a group size sooner.",
            |c| c.pairing = Pairing::Either,
        ),
        preset(
            "bg-none",
            "Fig. 1a: no punishment",
            BGBR,
            "Fig. 1a: no punishment (p = k = 0), the base case otherwise. Cooperation costs c and brings nothing to the cooperator; only conflict between groups favors it. Measured (10 seeds, the last 1 000 of 2 000 periods): 10 % cooperate, the floor mutation keeps up — as in the figure (0.09 from n 32). Group selection alone holds cooperation only in groups of 4 or 8.",
            |c| {
                c.fine = 0.0;
                c.punish_cost = 0.0;
            },
        ),
        preset(
            "bg-large",
            "Fig. 1b: groups of 128",
            BGBR,
            "Fig. 1b's groups of 128. Measured (10 seeds, the last 1 000 of 2 000 periods): 17 % cooperate (from 7 % to 43 % by seed), where the figure has 64 % and the Discussion 'cooperation is sustained in groups on the order of 100 individuals'. When either group can start a conflict (bg-either's reading), 59 %.",
            |c| c.size = 128,
        ),
        preset(
            "bg-weak",
            "Fig. 3: p = 0.4",
            BGBR,
            "Fig. 3: the cost of being punished p = 0.4, twice the cost of cooperating, where the base case has four times. 'Lower values of p result in much lower levels of cooperation.' Measured (10 seeds, the last 1 000 of 2 000 periods): 9 % cooperate at n 32 (the figure: 21 %; 83 % with p = 0.8) — much lower, as stated.",
            |c| c.fine = 0.4,
        ),
        preset(
            "bg-fixed",
            "Fig. 4: a fixed cost",
            BGBR,
            "Fig. 4: punishers pay a fixed cost equal to c every period, whether or not anyone defects, instead of k/n for each defector. 'Punishment does not aid in the evolution of cooperation when the costs born by punishers are fixed.' Measured (10 seeds, the last 1 000 of 2 000 periods): 9 % cooperate at n 32 — no better than no punishment (10 %), as stated.",
            |c| c.punishing = Punishing::Fixed,
        ),
        preset(
            "bg-mixing",
            "Fig. 2: m = 0.05",
            BGBR,
            "Fig. 2: mixing m = 0.05, five times the base case: an agent's model comes from another group one time in 20. 'When the migration rate increases, levels of cooperation fall precipitously.' Measured (10 seeds, the last 1 000 of 2 000 periods): 28 % cooperate at n 32 (from 15 % to 47 % by seed), against 69 % at m = 0.01 — falling, as stated (the figure keeps 72 % here, at n 32 its last size before the fall).",
            |c| c.mixing = 0.05,
        ),
        preset(
            "bg-benefit",
            "A benefit, and conflict over payoffs",
            BGBR,
            "One of the paper's structural variants: each cooperative act gives b/n = 0.8/n (b = 4c) to every other member, and conflicts are decided by groups' average payoffs, normalized by the widest difference possible (Cooney's eq. 3.18; the paper's form is not given). 'For reasonable values of b (2c, 4c, and 8c), the results … are qualitatively similar.' Measured (10 seeds, the last 1 000 of 2 000 periods): 63 % cooperate at n 32, against 11 % without punishment — similar, as stated.",
            |c| {
                c.benefit = 0.8;
                c.victory = Victory::Payoff;
            },
        ),
        preset(
            "bg-continuous",
            "Continuous traits",
            BGBR,
            "Continuous traits: each agent cooperates with probability x and punishes with probability y, both in [0, 1]; mutants draw both uniformly. 'The steady-state mean levels of cooperation in this model are similar to the base model.' Measured (10 seeds, the last 1 000 of 2 000 periods): 94 % cooperate at n 32 (the base model: 69 %), and still 90 % at n 256 (the base model: 12 %) — not similar. Uniform mutants keep the mean punishment near ½, and a defector then pays about p/2 = 0.4, more than c: cooperation pays within groups. Starting from all defectors it still collapses from n 128.",
            |c| c.traits = Traits::Continuous,
        ),
        preset(
            "bg-ring",
            "A ring without extinction",
            BGBR,
            "The paper's last variant: groups on a ring, no conflict, migrants only from the two neighboring groups, and each cooperative act giving b/n = 0.4/n (b = 2c) to the others, so that cooperative groups earn more and are imitated. 'We could find no reasonable parameter combination that led to significant long run average levels of cooperation.' Measured (10 seeds, the last 1 000 of 2 000 periods): 18 % cooperate at n 32 — low, as stated — but about half in groups of 4 or 8 (bg-ring).",
            |c| {
                c.structure = Structure::Ring;
                c.benefit = 0.4;
            },
        ),
        preset(
            "bg-janssen",
            "Janssen's readings",
            JANSSEN,
            "Marco Janssen's NetLogo replication (CoMSES 2223) fills the paper's gaps differently: payoffs 1 plus a benefit 0.5 × the share cooperating; each group challenges a random group with probability ε (about twice the text's conflict); groups fight over the share who cooperated this period; agents imitate in turn; a punisher who errs fines itself. His readings, not his code, are used here. Measured (10 seeds, the last 1 000 of 2 000 periods): 84 % cooperate at n 32 (the figure: 83 %). Of his readings the doubled conflict does the work (bg-readings); his benefit lifts cooperation in small groups without punishment.",
            |c| {
                c.benefit = 0.5;
                c.pairing = Pairing::Challenge;
                c.counted = Counted::Acts;
                c.imitation = Imitation::InTurn;
                c.erring = Erring::Itself;
            },
        ),
    ]
}
````

Create `crates/sugarscape-core/src/punishment/mod.rs` with exactly this content:

````rust
//! Altruistic Punishment (milestone 27): Boyd, Gintis, Bowles and Richerson,
//! "The evolution of altruistic punishment" (PNAS 100: 3531–3535, 2003),
//! with Cooney's PDE model (arXiv:2405.18419, 2024) as the critique. See
//! docs/superpowers/specs/2026-09-28-punishment-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, Refill, Start,
    Structure, Traits, Victory, MOST_AGENTS,
};
pub use presets::presets;
pub use stats::{PunishmentSnapshot, SERIES};
pub use view::{Mosaic, STRIP, STRIP_GAP};
pub use world::{
    payoff_range, AgentView, GroupView, Kind, PunishmentCell, PunishmentInspection, PunishmentMode,
    PunishmentWorld,
};
````

- [ ] **Step 2: Wire the model kind and title its presets**

Modify `crates/sugarscape-core/src/lib.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index cbe8b98..a0159e3 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -31,6 +31,7 @@ pub mod norms;
 pub mod opinions;
 pub mod portable;
 pub mod presets;
+pub mod punishment;
 pub mod render;
 pub mod retirement;
 pub mod ring;
````

Modify `crates/sugarscape-core/src/model.rs` — every match gains `Punishment`; the reader gains its `"punishment"` arm; a round-trip test pins it (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index 8c0221c..5973f19 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -18,6 +18,7 @@ use crate::farol::{FarolConfig, FarolWorld};
 use crate::image::{ImageConfig, ImageWorld};
 use crate::norms::{NormsConfig, NormsWorld};
 use crate::opinions::{OpinionsConfig, OpinionsWorld};
+use crate::punishment::{PunishmentConfig, PunishmentWorld};
 use crate::render::{self, ColorMode, Layer};
 use crate::retirement::{RetirementConfig, RetirementWorld};
 use crate::ring::{RingConfig, RingWorld};
@@ -30,7 +31,7 @@ use crate::thresholds::{ThresholdsConfig, ThresholdsWorld};
 use crate::world::World;
 use crate::{
     agreement, anasazi, ants, civil, classes, culture, dpd, ethno, export, farol, image, norms,
-    opinions, retirement, ring, schelling, spatial, stats, structure, tags, thresholds,
+    opinions, punishment, retirement, ring, schelling, spatial, stats, structure, tags, thresholds,
 };
 
 /// Which model a config or world is.
@@ -57,10 +58,11 @@ pub enum ModelKind {
     Ants,
     Thresholds,
     Retirement,
+    Punishment,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 20] = [
+    pub const ALL: [ModelKind; 21] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -81,6 +83,7 @@ impl ModelKind {
         ModelKind::Ants,
         ModelKind::Thresholds,
         ModelKind::Retirement,
+        ModelKind::Punishment,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -105,6 +108,7 @@ impl ModelKind {
             ModelKind::Ants => "ants",
             ModelKind::Thresholds => "thresholds",
             ModelKind::Retirement => "retirement",
+            ModelKind::Punishment => "punishment",
         }
     }
 
@@ -132,6 +136,7 @@ impl ModelKind {
             ModelKind::Ants => ants::schema(),
             ModelKind::Thresholds => thresholds::schema(),
             ModelKind::Retirement => retirement::schema(),
+            ModelKind::Punishment => punishment::schema(),
         }
     }
 }
@@ -165,6 +170,7 @@ pub enum ModelConfig {
     Ants(AntsConfig),
     Thresholds(ThresholdsConfig),
     Retirement(RetirementConfig),
+    Punishment(PunishmentConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -190,6 +196,7 @@ enum Tagged<'a> {
     Ants(&'a AntsConfig),
     Thresholds(&'a ThresholdsConfig),
     Retirement(&'a RetirementConfig),
+    Punishment(&'a PunishmentConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -222,6 +229,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Ants(c) => Tagged::Ants(c).serialize(s),
             ModelConfig::Thresholds(c) => Tagged::Thresholds(c).serialize(s),
             ModelConfig::Retirement(c) => Tagged::Retirement(c).serialize(s),
+            ModelConfig::Punishment(c) => Tagged::Punishment(c).serialize(s),
         }
     }
 }
@@ -249,6 +257,7 @@ impl ModelConfig {
             ModelConfig::Ants(_) => ModelKind::Ants,
             ModelConfig::Thresholds(_) => ModelKind::Thresholds,
             ModelConfig::Retirement(_) => ModelKind::Retirement,
+            ModelConfig::Punishment(_) => ModelKind::Punishment,
         }
     }
 
@@ -343,10 +352,13 @@ impl ModelConfig {
             "retirement" => serde_json::from_value(value)
                 .map(ModelConfig::Retirement)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "punishment" => serde_json::from_value(value)
+                .map(ModelConfig::Punishment)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds or retirement)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds, retirement or punishment)"
                 ),
             )),
         }
@@ -374,6 +386,7 @@ impl ModelConfig {
             ModelConfig::Ants(c) => c.validate(),
             ModelConfig::Thresholds(c) => c.validate(),
             ModelConfig::Retirement(c) => c.validate(),
+            ModelConfig::Punishment(c) => c.validate(),
         }
     }
 
@@ -401,6 +414,7 @@ impl ModelConfig {
             ModelConfig::Ants(c) => set_path(c, path, value).map(ModelConfig::Ants),
             ModelConfig::Thresholds(c) => set_path(c, path, value).map(ModelConfig::Thresholds),
             ModelConfig::Retirement(c) => set_path(c, path, value).map(ModelConfig::Retirement),
+            ModelConfig::Punishment(c) => set_path(c, path, value).map(ModelConfig::Punishment),
         }
     }
 
@@ -427,7 +441,8 @@ impl ModelConfig {
             | ModelConfig::Farol(_)
             | ModelConfig::Ants(_)
             | ModelConfig::Thresholds(_)
-            | ModelConfig::Retirement(_) => None,
+            | ModelConfig::Retirement(_)
+            | ModelConfig::Punishment(_) => None,
         }
     }
 
@@ -458,6 +473,9 @@ impl ModelConfig {
             ModelConfig::Retirement(_) => {
                 retirement::SERIES.iter().map(|s| s.to_string()).collect()
             }
+            ModelConfig::Punishment(_) => {
+                punishment::SERIES.iter().map(|s| s.to_string()).collect()
+            }
         }
     }
 }
@@ -647,6 +665,7 @@ pub enum ModelWorld {
     Ants(Box<AntsWorld>),
     Thresholds(Box<ThresholdsWorld>),
     Retirement(Box<RetirementWorld>),
+    Punishment(Box<PunishmentWorld>),
 }
 
 impl ModelWorld {
@@ -696,6 +715,9 @@ impl ModelWorld {
             ModelConfig::Retirement(c) => {
                 ModelWorld::Retirement(Box::new(RetirementWorld::new(c, seed)?))
             }
+            ModelConfig::Punishment(c) => {
+                ModelWorld::Punishment(Box::new(PunishmentWorld::new(c, seed)?))
+            }
         })
     }
 
@@ -721,6 +743,7 @@ impl ModelWorld {
             ModelWorld::Ants(_) => ModelKind::Ants,
             ModelWorld::Thresholds(_) => ModelKind::Thresholds,
             ModelWorld::Retirement(_) => ModelKind::Retirement,
+            ModelWorld::Punishment(_) => ModelKind::Punishment,
         }
     }
 
@@ -746,6 +769,7 @@ impl ModelWorld {
             ModelWorld::Ants(w) => w.as_ref(),
             ModelWorld::Thresholds(w) => w.as_ref(),
             ModelWorld::Retirement(w) => w.as_ref(),
+            ModelWorld::Punishment(w) => w.as_ref(),
         }
     }
 
@@ -771,6 +795,7 @@ impl ModelWorld {
             ModelWorld::Ants(w) => w.as_mut(),
             ModelWorld::Thresholds(w) => w.as_mut(),
             ModelWorld::Retirement(w) => w.as_mut(),
+            ModelWorld::Punishment(w) => w.as_mut(),
         }
     }
 
@@ -865,6 +890,7 @@ impl ModelWorld {
             ModelWorld::Ants(w) => copy_without_history!(Ants, w),
             ModelWorld::Thresholds(w) => copy_without_history!(Thresholds, w),
             ModelWorld::Retirement(w) => copy_without_history!(Retirement, w),
+            ModelWorld::Punishment(w) => copy_without_history!(Punishment, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -906,6 +932,9 @@ impl ModelWorld {
             (ModelWorld::Retirement(live), ModelWorld::Retirement(kept)) => {
                 restore_into!(live, kept)
             }
+            (ModelWorld::Punishment(live), ModelWorld::Punishment(kept)) => {
+                restore_into!(live, kept)
+            }
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1306,6 +1335,30 @@ mod tests {
         assert_eq!(w.model().tick(), 0);
     }
 
+    #[test]
+    fn punishment_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "punishment", "size": 8, "groups": 16, "victory": "tanh", "erring": "self"}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Punishment);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(
+            (json["model"].as_str(), json["fine"].as_f64()),
+            (Some("punishment"), Some(0.8))
+        );
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["cooperation", "contributors"]);
+        let e = ModelConfig::from_json(r#"{"model": "punishment", "error": 2}"#).unwrap_err();
+        assert_eq!(e[0].field, "error");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!(w.kind(), ModelKind::Punishment);
+        let cp = w.checkpoint().expect("punishment worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
     #[test]
     fn only_the_anasazi_finishes() {
         let mut w = ModelWorld::new(
@@ -1349,7 +1402,8 @@ mod tests {
                 "farol",
                 "ants",
                 "thresholds",
-                "retirement"
+                "retirement",
+                "punishment"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
````

Modify `crates/sugarscape-core/src/presets.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index fba5277..4037930 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -946,6 +946,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::ants::presets());
     out.extend(crate::thresholds::presets());
     out.extend(crate::retirement::presets());
+    out.extend(crate::punishment::presets());
     out
 }
````

Modify `crates/sugarscape-core/src/titles.rs` — eleven titles (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/titles.rs b/crates/sugarscape-core/src/titles.rs
index a63dbce..9398580 100644
--- a/crates/sugarscape-core/src/titles.rs
+++ b/crates/sugarscape-core/src/titles.rs
@@ -3,7 +3,7 @@
 //! paper (`source`) stay on the preset as its reference.
 
 /// Titles by preset id, in catalog order.
-pub const TITLES: [(&str, &str); 253] = [
+pub const TITLES: [(&str, &str); 264] = [
     (
         "ii-1-instant",
         "Sugar grows back at once: agents climb the best ridges and the poorly endowed starve",
@@ -977,6 +977,50 @@ pub const TITLES: [(&str, &str); 253] = [
         "ae-replace",
         "Replace friends who die, and 5 % rationality is no longer enough",
     ),
+    (
+        "bg-base",
+        "Punishers keep about 70 % of groups of 32 cooperating",
+    ),
+    (
+        "bg-either",
+        "Either group can start a war, and the paper's figures appear",
+    ),
+    (
+        "bg-none",
+        "Without punishment, groups of 32 fall to defection",
+    ),
+    (
+        "bg-large",
+        "Groups of 128: punishment no longer holds",
+    ),
+    (
+        "bg-weak",
+        "A fine only twice the cost, and cooperation fades",
+    ),
+    (
+        "bg-fixed",
+        "Punishers who pay whether or not anyone defects, and cooperation fails",
+    ),
+    (
+        "bg-mixing",
+        "More mixing between groups, and cooperation falls",
+    ),
+    (
+        "bg-benefit",
+        "Cooperation benefits the group, and groups fight over payoffs",
+    ),
+    (
+        "bg-continuous",
+        "Cooperate and punish by degrees, and nearly everyone cooperates",
+    ),
+    (
+        "bg-ring",
+        "A ring of groups with no wars: cooperation stays low",
+    ),
+    (
+        "bg-janssen",
+        "Janssen's readings of the gaps together",
+    ),
 ];
 
 /// The title of preset `id`, or "" if it has none.
````

- [ ] **Step 3: Run the model's tests**

Run: `cargo test -p sugarscape-core --lib punishment`
Expected: PASS (26).

- [ ] **Step 4: Watch the golden check fail, then record the entries**

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: FAIL with `record a golden fingerprint for bg-base (run print_golden)`.

Modify `crates/sugarscape-core/tests/golden.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 355e828..283d16e 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -271,6 +271,17 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("ae-groups", 0xadde267c611d5392),
     ("ae-all-members", 0x8f6694a3282613e3),
     ("ae-replace", 0x90d96b846be612f2),
+    ("bg-base", 0x18a87a7bb2ab932d),
+    ("bg-either", 0xa966f403050db40d),
+    ("bg-none", 0x8082577a230b3670),
+    ("bg-large", 0x7d2e95c561d9fdb0),
+    ("bg-weak", 0x1ce6ad7a771157d0),
+    ("bg-fixed", 0x58e1a7a64780e030),
+    ("bg-mixing", 0x5d9e1aa7e6f5a06d),
+    ("bg-benefit", 0xe5fec27869f67010),
+    ("bg-continuous", 0xf763b0b75f9a2298),
+    ("bg-ring", 0xc4e75af13ebdeaed),
+    ("bg-janssen", 0xe95bb859afd7e9b0),
 ];
 
 fn fingerprint(id: &str) -> u64 {
````

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: PASS.

- [ ] **Step 5: Format, lint, run everything, commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace
git add crates/sugarscape-core/src/punishment crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs crates/sugarscape-core/tests/golden.rs
```
```bash
git commit -m "Add Altruistic Punishment (Boyd, Gintis, Bowles & Richerson) as a model kind

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/{bg-fig1a,bg-fig1b,bg-fig1-caption,bg-fig1-either,bg-fig2a,bg-fig2b,bg-fig3,bg-fig4,bg-baseline,bg-readings,bg-mutation,bg-error,bg-groups,bg-benefit,bg-continuous,bg-ring,bg-cooney-fine,bg-cooney-cost}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets and series (`long_run`, `payoff`, `punishment`).
- Produces: eighteen built-in sweeps; the CLI's stop `(its last period)`.

- [ ] **Step 1: Write the failing tests**

Modify `crates/sugarscape-cli/tests/cli.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index 0ed21fb..1644894 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -152,6 +152,24 @@ fn presets_and_sweeps_are_listed() {
         "ae-policy",
         "ae-coupling",
         "ae-coupling-rational",
+        "bg-fig1a",
+        "bg-fig1b",
+        "bg-fig1-caption",
+        "bg-fig1-either",
+        "bg-fig2a",
+        "bg-fig2b",
+        "bg-fig3",
+        "bg-fig4",
+        "bg-baseline",
+        "bg-readings",
+        "bg-mutation",
+        "bg-error",
+        "bg-groups",
+        "bg-benefit",
+        "bg-continuous",
+        "bg-ring",
+        "bg-cooney-fine",
+        "bg-cooney-cost",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -575,6 +593,26 @@ fn a_thresholds_run_stops_at_its_last_step() {
     assert_eq!(stderr(&out), "finished at tick 25 (its last step)\n");
 }
 
+#[test]
+fn a_punishment_run_stops_at_its_last_period() {
+    let dir = scratch("punishment");
+    let config = dir.join("punishment.json");
+    std::fs::write(
+        &config,
+        r#"{"model": "punishment", "groups": 8, "size": 4, "stop_at": 30}"#,
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
+    assert_eq!(stderr(&out), "finished at tick 30 (its last period)\n");
+}
+
 #[test]
 fn a_retirement_run_stops_at_the_norm_or_its_last_period() {
     let dir = scratch("retirement");
````

Modify `crates/sugarscape-wasm/tests/web.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index 23d60db..9ae51c6 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -362,7 +362,25 @@ fn builtins_and_series_names_are_listed() {
             "ae-extent",
             "ae-policy",
             "ae-coupling",
-            "ae-coupling-rational"
+            "ae-coupling-rational",
+            "bg-fig1a",
+            "bg-fig1b",
+            "bg-fig1-caption",
+            "bg-fig1-either",
+            "bg-fig2a",
+            "bg-fig2b",
+            "bg-fig3",
+            "bg-fig4",
+            "bg-baseline",
+            "bg-readings",
+            "bg-mutation",
+            "bg-error",
+            "bg-groups",
+            "bg-benefit",
+            "bg-continuous",
+            "bg-ring",
+            "bg-cooney-fine",
+            "bg-cooney-cost"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -988,6 +1006,31 @@ fn retirement_sims_match_the_native_golden_entries() {
     }
 }
 
+#[wasm_bindgen_test]
+fn punishment_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the base case,
+    // either group starting a conflict, payoff conflict, continuous traits,
+    // the ring, and Janssen's readings.
+    for (id, fp) in [
+        ("bg-base", "0x18a87a7bb2ab932d"),
+        ("bg-either", "0xa966f403050db40d"),
+        ("bg-benefit", "0xe5fec27869f67010"),
+        ("bg-continuous", "0xf763b0b75f9a2298"),
+        ("bg-ring", "0xc4e75af13ebdeaed"),
+        ("bg-janssen", "0xe95bb859afd7e9b0"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "punishment");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+    // The tanh victory rule (portable exp_neg): the native CLI's fingerprint.
+    let tanh = r#"{"model": "punishment", "groups": 32, "size": 16, "benefit": 0.4, "victory": "tanh", "sensitivity": 3, "conflict": 0.1}"#;
+    let mut sim = Sim::new(tanh, 1, JsValue::NULL).unwrap();
+    sim.step(200);
+    assert_eq!(sim.fingerprint(), "0x41e7fa5fab5ba690");
+}
+
 #[wasm_bindgen_test]
 fn dpd_sims_match_the_native_golden_entries() {
     // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: wealth sums and
````

Run: `cargo test -p sugarscape-cli`
Expected: FAIL (`a_punishment_run_stops_at_its_last_period`: `(its end year)`; `presets_and_sweeps_are_listed`: `bg-fig1a` missing).

- [ ] **Step 2: Write the sweeps, register them and name the stop**

Create `sweeps/bg-fig1a.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 1a, no punishment",
  "description": "Boyd and coauthors' Figure 1a: cooperation against group size without punishment (p = k = 0), for the legend's conflict rates. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): ε 0.0075: 0.49, 0.26, 0.13, 0.10 and 0.09 from n 64; ε 0.015: 0.62, 0.33, 0.14, 0.10, 0.09 …; ε 0.03: 0.75, 0.56, 0.19, 0.10, 0.09 … at n 4, 8, 16, 32. 'Group selection is ineffective unless groups are quite small', as stated — but each curve is the figure's next lower one: 0.62 at ε 0.015 where the figure has 0.76 (and 0.63 at 0.0075). See bg-fig1-either.",
  "base": {
    "preset": "bg-none"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Conflict rate (ε)",
    "values": [
      {
        "at": 0.0075,
        "name": "ε = 0.0075",
        "set": {
          "conflict": 0.0075
        }
      },
      {
        "at": 0.015,
        "name": "ε = 0.015",
        "set": {
          "conflict": 0.015
        }
      },
      {
        "at": 0.03,
        "name": "ε = 0.03",
        "set": {
          "conflict": 0.03
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-fig1b.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 1b, with punishment",
  "description": "Figure 1b: cooperation against group size with punishment (p = 0.8, k = 0.2), for the legend's conflict rates. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): ε 0.0075: 0.76, 0.76, 0.66, 0.45, 0.10, 0.07, 0.07; ε 0.015: 0.82, 0.83, 0.78, 0.69, 0.44, 0.17, 0.12; ε 0.03: 0.88, 0.89, 0.87, 0.83, 0.79, 0.61, 0.53 at n 4–256. Punishment sustains cooperation in larger groups, and more conflict more — but where the figure's base curve holds 0.64 at n 128 and 0.55 at 256, this one has 0.17 and 0.12. Each curve here is the figure's next lower one: ε 0.03 here is the figure's 0.015 (0.87, 0.88, 0.86, 0.83, 0.79, 0.64, 0.55). See bg-fig1-either.",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Conflict rate (ε)",
    "values": [
      {
        "at": 0.0075,
        "name": "ε = 0.0075",
        "set": {
          "conflict": 0.0075
        }
      },
      {
        "at": 0.015,
        "name": "ε = 0.015",
        "set": {
          "conflict": 0.015
        }
      },
      {
        "at": 0.03,
        "name": "ε = 0.03",
        "set": {
          "conflict": 0.03
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-fig1-caption.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 1 at the caption's rates",
  "description": "Figure 1's caption gives the conflict rates as 0.075, 0.015 and 0.003, the legend as 0.0075, 0.015 and 0.03. The caption's outer rates, with and without punishment. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): with punishment, ε 0.075: 0.94, 0.94, 0.93, 0.92, 0.91, 0.81, 0.84; ε 0.003: 0.71, 0.69, 0.56, 0.24, 0.08, 0.07, 0.07; without, 0.89, 0.81, 0.61, 0.13 … and 0.43, 0.22, 0.13, 0.10 …. Summed over the four outer curves the caption's rates miss the figure by 2.21, the legend's by 2.51, and twice the legend's by 0.45: neither stated set is what was plotted.",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Conflict rate (the caption's)",
    "values": [
      {
        "at": 1,
        "name": "ε = 0.075, punishment",
        "set": {
          "conflict": 0.075
        }
      },
      {
        "at": 2,
        "name": "ε = 0.003, punishment",
        "set": {
          "conflict": 0.003
        }
      },
      {
        "at": 3,
        "name": "ε = 0.075, none",
        "set": {
          "conflict": 0.075,
          "fine": 0,
          "punish_cost": 0
        }
      },
      {
        "at": 4,
        "name": "ε = 0.003, none",
        "set": {
          "conflict": 0.003,
          "fine": 0,
          "punish_cost": 0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-fig1-either.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 1 when either group starts a conflict",
  "description": "Figure 1 when either group of a pair can start the conflict (each with probability ε; a pair fights with probability 2ε − ε², about twice the text's), at the legend's rates, with and without punishment. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): with punishment, ε 0.0075: 0.82, 0.82, 0.77, 0.69, 0.51, 0.20, 0.07 (the figure: 0.81, 0.82, 0.77, 0.68, 0.47, 0.16, 0.07); ε 0.015: 0.88, 0.89, 0.86, 0.84, 0.75, 0.59, 0.49 (0.87, 0.88, 0.86, 0.83, 0.79, 0.64, 0.55); ε 0.03: 0.92, 0.93, 0.92, 0.91, 0.89, 0.85, 0.81 (0.92, 0.93, 0.92, 0.91, 0.89, 0.79, 0.66). Without: 0.59, 0.34, 0.14 …; 0.75, 0.54, 0.18 …; 0.86, 0.76, 0.49, 0.11 … (0.63, 0.37, 0.14; 0.76, 0.57, 0.20; 0.87, 0.79, 0.49, 0.11). The mean gap to the figure is 0.010–0.033 per curve: this reading, with a payoff baseline of 1, reproduces Figure 1.",
  "base": {
    "preset": "bg-either"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Conflict rate (ε)",
    "values": [
      {
        "at": 1,
        "name": "ε = 0.0075, punishment",
        "set": {
          "conflict": 0.0075
        }
      },
      {
        "at": 2,
        "name": "ε = 0.015, punishment",
        "set": {
          "conflict": 0.015
        }
      },
      {
        "at": 3,
        "name": "ε = 0.03, punishment",
        "set": {
          "conflict": 0.03
        }
      },
      {
        "at": 4,
        "name": "ε = 0.0075, none",
        "set": {
          "conflict": 0.0075,
          "fine": 0,
          "punish_cost": 0
        }
      },
      {
        "at": 5,
        "name": "ε = 0.015, none",
        "set": {
          "conflict": 0.015,
          "fine": 0,
          "punish_cost": 0
        }
      },
      {
        "at": 6,
        "name": "ε = 0.03, none",
        "set": {
          "conflict": 0.03,
          "fine": 0,
          "punish_cost": 0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-fig2a.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 2a, mixing without punishment",
  "description": "Figure 2a: cooperation against group size without punishment, for mixing rates m 0.002, 0.01, 0.05. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): m 0.002: 0.67, 0.46, 0.19, 0.10 …; m 0.01: 0.62, 0.33, 0.14, 0.10 …; m 0.05: 0.32, 0.16, 0.11, 0.10 … at n 4, 8, 16, 32. More mixing, less cooperation, as stated (the figure: 0.79, 0.66, 0.34; 0.75, 0.57, 0.20; 0.55, 0.21, 0.11).",
  "base": {
    "preset": "bg-none"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Mixing (m)",
    "values": [
      {
        "at": 0.002,
        "name": "m = 0.002",
        "set": {
          "mixing": 0.002
        }
      },
      {
        "at": 0.01,
        "name": "m = 0.01",
        "set": {
          "mixing": 0.01
        }
      },
      {
        "at": 0.05,
        "name": "m = 0.05",
        "set": {
          "mixing": 0.05
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-fig2b.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 2b, mixing with punishment",
  "description": "Figure 2b: with punishment, for mixing rates m 0.002, 0.01, 0.05. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): m 0.002: 0.82, 0.82, 0.77, 0.72, 0.65, 0.43, 0.38; m 0.01: 0.82, 0.83, 0.78, 0.69, 0.44, 0.17, 0.12; m 0.05: 0.81, 0.83, 0.77, 0.28, 0.08, 0.07, 0.07 at n 4–256. 'At higher rates of mixing, cooperation does not persist in the largest groups', as stated; but the figure's m 0.002 keeps 0.71 and 0.73 at n 128 and 256, where this has 0.43 and 0.38. When either group can start a conflict: 0.70 and 0.73.",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Mixing (m)",
    "values": [
      {
        "at": 0.002,
        "name": "m = 0.002",
        "set": {
          "mixing": 0.002
        }
      },
      {
        "at": 0.01,
        "name": "m = 0.01",
        "set": {
          "mixing": 0.01
        }
      },
      {
        "at": 0.05,
        "name": "m = 0.05",
        "set": {
          "mixing": 0.05
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-fig3.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 3, the cost of being punished",
  "description": "Figure 3: the cost of being punished p = 0.4 against the base case's 0.8. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): p 0.4: 0.66, 0.51, 0.23, 0.09, 0.07, 0.07, 0.07; p 0.8: 0.82, 0.83, 0.78, 0.69, 0.44, 0.17, 0.12 at n 4–256. 'Lower values of p result in much lower levels of cooperation', as stated (the figure's p 0.4: 0.80, 0.72, 0.58, 0.21, 0.07 …).",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Cost of being punished (p)",
    "values": [
      {
        "at": 0.4,
        "name": "p = 0.4",
        "set": {
          "fine": 0.4
        }
      },
      {
        "at": 0.8,
        "name": "p = 0.8",
        "set": {
          "fine": 0.8
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-fig4.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: Fig. 4, fixed against variable costs",
  "description": "Figure 4: no punishment, a fixed punishing cost (equal to c, paid every period), and the variable cost (k/n per defector). Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): none: 0.62, 0.33, 0.14, 0.10, 0.09 …; fixed: 0.70, 0.51, 0.21, 0.09, 0.07 …; variable: 0.82, 0.83, 0.78, 0.69, 0.44, 0.17, 0.12 at n 4–256. A fixed cost helps only in the smallest groups and not at all from n 32, as stated (the figure's fixed: 0.84, 0.78, 0.50, 0.10, 0.07 …).",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Punishment",
    "values": [
      {
        "at": 0,
        "name": "None",
        "set": {
          "fine": 0,
          "punish_cost": 0
        }
      },
      {
        "at": 1,
        "name": "Fixed cost",
        "set": {
          "punishing": "fixed"
        }
      },
      {
        "at": 2,
        "name": "Variable cost",
        "set": {
          "punishing": "variable"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-baseline.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: the unstated baseline",
  "description": "The paper never says what payoff the game's costs and fines are subtracted from; imitation needs payoffs above 0. Baselines 1, 2, 4, with and without punishment. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): with punishment, 1: 0.82, 0.83, 0.78, 0.69, 0.44, 0.17, 0.12; 2: 0.80, 0.79, 0.79, 0.75, 0.69, 0.51, 0.18; 4: 0.79, 0.78, 0.77, 0.76, 0.73, 0.70, 0.66. Without: 1: 0.62, 0.33, 0.14, 0.10, 0.09 …; 2: 0.73, 0.60, 0.42, 0.26, 0.20, 0.19, 0.18; 4: 0.77, 0.72, 0.62, 0.50, 0.41, 0.35, 0.33. A higher baseline weakens selection, so punishment reaches larger groups — but cooperation without punishment then stays far above the figure's 0.09: no baseline fits both panels. Baseline 1 does, when either group can start a conflict (bg-fig1-either).",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Baseline payoff",
    "values": [
      {
        "at": 1,
        "name": "1, punishment",
        "set": {
          "baseline": 1
        }
      },
      {
        "at": 2,
        "name": "2, punishment",
        "set": {
          "baseline": 2
        }
      },
      {
        "at": 4,
        "name": "4, punishment",
        "set": {
          "baseline": 4
        }
      },
      {
        "at": -1,
        "name": "1, none",
        "set": {
          "baseline": 1,
          "fine": 0,
          "punish_cost": 0
        }
      },
      {
        "at": -2,
        "name": "2, none",
        "set": {
          "baseline": 2,
          "fine": 0,
          "punish_cost": 0
        }
      },
      {
        "at": -4,
        "name": "4, none",
        "set": {
          "baseline": 4,
          "fine": 0,
          "punish_cost": 0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-readings.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: readings of the gaps",
  "description": "The readings of what the paper leaves open, with punishment. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): the text: 0.82, 0.83, 0.78, 0.69, 0.44, 0.17, 0.12; each group challenging one (Janssen's pairing, about twice the conflict): 0.88, 0.89, 0.86, 0.83, 0.79, 0.53, 0.48; imitation in turn: 0.81, 0.82, 0.81, 0.73, 0.61, 0.09, 0.10; Janssen's readings together: 0.84, 0.83, 0.85, 0.84, 0.80, 0.77, 0.56, and without punishment 0.77, 0.67, 0.42, 0.13, 0.09 … at n 4–256. The doubled conflict does the work; Janssen's benefit lifts cooperation without punishment above the figure's (0.42 at n 16 against 0.20).",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Reading",
    "values": [
      {
        "at": 0,
        "name": "The text",
        "set": {}
      },
      {
        "at": 1,
        "name": "Each group challenges",
        "set": {
          "pairing": "challenge"
        }
      },
      {
        "at": 2,
        "name": "Imitation in turn",
        "set": {
          "imitation": "in_turn"
        }
      },
      {
        "at": 3,
        "name": "Janssen's readings",
        "set": {
          "benefit": 0.5,
          "pairing": "challenge",
          "counted": "acts",
          "imitation": "in_turn",
          "erring": "self"
        }
      },
      {
        "at": 4,
        "name": "Janssen's, no punishment",
        "set": {
          "benefit": 0.5,
          "pairing": "challenge",
          "counted": "acts",
          "imitation": "in_turn",
          "erring": "self",
          "fine": 0,
          "punish_cost": 0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-mutation.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: mutation",
  "description": "'Decreasing the mutation rate substantially increases the long run average levels of cooperation.' Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28), n 32: 0.96, 0.83, 0.69, 0.57, 0.64 at μ 0.001, 0.005, 0.01, 0.02, 0.05 — as stated, from 0.69 to 0.96.",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Mutation (μ)",
    "path": "mutation",
    "values": [
      0.001,
      0.005,
      0.01,
      0.02,
      0.05
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-error.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: errors",
  "description": "'Increasing e, the error rate, reduces the long run average amount of cooperation.' Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28), n 32: 0.73, 0.69, 0.65, 0.55, 0.38 at e 0, 0.02, 0.05, 0.1, 0.2 — as stated.",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Errors (e)",
    "path": "error",
    "values": [
      0,
      0.02,
      0.05,
      0.1,
      0.2
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-groups.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: the number of groups",
  "description": "'Reducing the number of groups, N, adds random noise to the results.' Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28), n 32: 0.56, 0.65, 0.69, 0.70, 0.69 at N 8, 16, 32, 64, 128 — noise, and at 8 groups a lower mean too.",
  "base": {
    "preset": "bg-base"
  },
  "x": {
    "label": "Groups (N)",
    "path": "groups",
    "values": [
      8,
      16,
      32,
      64,
      128
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-benefit.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: a benefit, and conflict over payoffs",
  "description": "The structural variant: each cooperative act gives b/n to every other member, and groups fight over their average payoffs (normalized by the widest possible difference, Cooney's eq. 3.18). Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): b 2c: 0.58, 0.61, 0.57, 0.40, 0.12, 0.08, 0.07; b 4c: 0.58, 0.62, 0.64, 0.63, 0.46, 0.20, 0.08; b 8c: 0.47, 0.52, 0.58, 0.57, 0.50, 0.25, 0.09 at n 4–256. Punishment sustains cooperation in groups of 16–64, 'qualitatively similar', as stated; a larger benefit lowers it in small groups.",
  "base": {
    "preset": "bg-benefit"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Benefit (b)",
    "values": [
      {
        "at": 0.4,
        "name": "b = 2c",
        "set": {
          "benefit": 0.4
        }
      },
      {
        "at": 0.8,
        "name": "b = 4c",
        "set": {
          "benefit": 0.8
        }
      },
      {
        "at": 1.6,
        "name": "b = 8c",
        "set": {
          "benefit": 1.6
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-continuous.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: continuous traits",
  "description": "Continuous traits (cooperate with probability x, punish with probability y; mutants uniform), with punishment, without, from all defectors, and without conflict. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): 0.70, 0.83, 0.92, 0.94, 0.96, 0.96, 0.90; without punishment 0.48, 0.39, 0.26, 0.17, 0.12, 0.10, 0.10; from all defectors 0.70, 0.81, 0.88, 0.89, 0.83, 0.14, 0.05; without conflict 0.59, 0.71, 0.79, 0.42, 0.10 … at n 4–256. Not 'similar to the base model': with punishment, cooperation rises with group size. Uniform mutants keep the mean punishment near ½, so defecting costs about p/2 = 0.4 > c within a group.",
  "base": {
    "preset": "bg-continuous"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Continuous traits",
    "values": [
      {
        "at": 0,
        "name": "Punishment",
        "set": {}
      },
      {
        "at": 1,
        "name": "No punishment",
        "set": {
          "fine": 0,
          "punish_cost": 0
        }
      },
      {
        "at": 2,
        "name": "From all defectors",
        "set": {
          "start": "all_defectors"
        }
      },
      {
        "at": 3,
        "name": "No conflict",
        "set": {
          "conflict": 0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-ring.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: a ring without extinction",
  "description": "A ring of groups without conflict, migrants only from the two neighbors, and a per-capita benefit b/n. Measured (release, seeds 1–10, the mean cooperation over the last 1 000 of 2 000 periods, recorded 2026-09-28): b 2c: 0.51, 0.49, 0.37, 0.17, 0.09, 0.08, 0.07; b 8c: 0.28, 0.28, 0.25, 0.18, 0.11, 0.08, 0.08 at n 4–256. Low in groups of 32 and more, but about half cooperate in groups of 4 or 8 — 'no reasonable parameter combination … led to significant long run average levels of cooperation' holds only for larger groups. A larger benefit lowers it.",
  "base": {
    "preset": "bg-ring"
  },
  "x": {
    "label": "Group size (n)",
    "path": "size",
    "values": [
      4,
      8,
      16,
      32,
      64,
      128,
      256
    ]
  },
  "series": {
    "label": "Benefit (b)",
    "values": [
      {
        "at": 0.4,
        "name": "b = 2c",
        "set": {
          "benefit": 0.4
        }
      },
      {
        "at": 1.6,
        "name": "b = 8c",
        "set": {
          "benefit": 1.6
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "long_run"
  }
}
````

Create `sweeps/bg-cooney-fine.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: payoff against the fine (Cooney)",
  "description": "Cooney's claim: under conflict decided by average payoffs normalized by the widest possible difference, the long-run average payoff depends non-monotonically on the strength of punishment p — and (his Remark 6.1) not under the Fermi (tanh) rule. b 2c, n 32. Measured (release, seeds 1–10, the mean payoff over the last 1 000 of 2 000 periods, recorded 2026-09-28): normalized: 1.010, 1.006, 1.004, 1.002, 1.002, 1.003, 1.041, 1.095, 1.113, 1.131; tanh: 1.010, 1.006, 1.005, 1.003, 1.004, 1.034, 1.090, 1.118, 1.125, 1.136; by defectors: 1.010, 1.006, 1.004, 1.003, 1.006, 1.046, 1.091, 1.118, 1.128, 1.135 at p 0, 0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.2, 1.6, 2.4. A shallow dip at p 0.4–0.5 under every rule: weak punishment costs punishers without deterring anyone. The dip is there, but not only where Cooney's analysis puts it.",
  "base": {
    "preset": "bg-benefit"
  },
  "set": {
    "benefit": 0.4
  },
  "x": {
    "label": "Cost of being punished (p)",
    "path": "fine",
    "values": [
      0,
      0.2,
      0.3,
      0.4,
      0.5,
      0.6,
      0.8,
      1.2,
      1.6,
      2.4
    ]
  },
  "series": {
    "label": "Groups fight over",
    "values": [
      {
        "at": 0,
        "name": "Payoffs, normalized",
        "set": {
          "victory": "payoff"
        }
      },
      {
        "at": 1,
        "name": "Payoffs, tanh",
        "set": {
          "victory": "tanh"
        }
      },
      {
        "at": 2,
        "name": "Defectors",
        "set": {
          "victory": "defectors"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "window_mean",
    "series": "payoff",
    "from": 1001
  }
}
````

Create `sweeps/bg-cooney-cost.json` with exactly this content:

````json
{
  "name": "Altruistic Punishment: punishers against the cost of punishing (Cooney)",
  "description": "Cooney's claim: under normalized payoff conflict, 'increasing the cost of punishing defectors can increase the level of altruistic punishment at steady state'. b 2c, n 32. Measured (release, seeds 1–10, the mean punishment over the last 1 000 of 2 000 periods, recorded 2026-09-28): normalized: 0.511, 0.441, 0.247, 0.017, 0.006; tanh: 0.533, 0.494, 0.417, 0.084, 0.006; by defectors: 0.550, 0.520, 0.438, 0.202, 0.008 at k 0.05, 0.1, 0.2, 0.4, 0.8. Punishment falls with its cost under every rule; the rise does not appear.",
  "base": {
    "preset": "bg-benefit"
  },
  "set": {
    "benefit": 0.4
  },
  "x": {
    "label": "Cost of punishing (k)",
    "path": "punish_cost",
    "values": [
      0.05,
      0.1,
      0.2,
      0.4,
      0.8
    ]
  },
  "series": {
    "label": "Groups fight over",
    "values": [
      {
        "at": 0,
        "name": "Payoffs, normalized",
        "set": {
          "victory": "payoff"
        }
      },
      {
        "at": 1,
        "name": "Payoffs, tanh",
        "set": {
          "victory": "tanh"
        }
      },
      {
        "at": 2,
        "name": "Defectors",
        "set": {
          "victory": "defectors"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2000,
  "metric": {
    "kind": "window_mean",
    "series": "punishment",
    "from": 1001
  }
}
````

Modify `crates/sugarscape-core/src/sweep.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index 627c968..2b19752 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -965,7 +965,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 108] = [
+const BUILTINS: [Builtin; 126] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1398,6 +1398,78 @@ const BUILTINS: [Builtin; 108] = [
         id: "ae-coupling-rational",
         json: include_str!("../../../sweeps/ae-coupling-rational.json"),
     },
+    Builtin {
+        id: "bg-fig1a",
+        json: include_str!("../../../sweeps/bg-fig1a.json"),
+    },
+    Builtin {
+        id: "bg-fig1b",
+        json: include_str!("../../../sweeps/bg-fig1b.json"),
+    },
+    Builtin {
+        id: "bg-fig1-caption",
+        json: include_str!("../../../sweeps/bg-fig1-caption.json"),
+    },
+    Builtin {
+        id: "bg-fig1-either",
+        json: include_str!("../../../sweeps/bg-fig1-either.json"),
+    },
+    Builtin {
+        id: "bg-fig2a",
+        json: include_str!("../../../sweeps/bg-fig2a.json"),
+    },
+    Builtin {
+        id: "bg-fig2b",
+        json: include_str!("../../../sweeps/bg-fig2b.json"),
+    },
+    Builtin {
+        id: "bg-fig3",
+        json: include_str!("../../../sweeps/bg-fig3.json"),
+    },
+    Builtin {
+        id: "bg-fig4",
+        json: include_str!("../../../sweeps/bg-fig4.json"),
+    },
+    Builtin {
+        id: "bg-baseline",
+        json: include_str!("../../../sweeps/bg-baseline.json"),
+    },
+    Builtin {
+        id: "bg-readings",
+        json: include_str!("../../../sweeps/bg-readings.json"),
+    },
+    Builtin {
+        id: "bg-mutation",
+        json: include_str!("../../../sweeps/bg-mutation.json"),
+    },
+    Builtin {
+        id: "bg-error",
+        json: include_str!("../../../sweeps/bg-error.json"),
+    },
+    Builtin {
+        id: "bg-groups",
+        json: include_str!("../../../sweeps/bg-groups.json"),
+    },
+    Builtin {
+        id: "bg-benefit",
+        json: include_str!("../../../sweeps/bg-benefit.json"),
+    },
+    Builtin {
+        id: "bg-continuous",
+        json: include_str!("../../../sweeps/bg-continuous.json"),
+    },
+    Builtin {
+        id: "bg-ring",
+        json: include_str!("../../../sweeps/bg-ring.json"),
+    },
+    Builtin {
+        id: "bg-cooney-fine",
+        json: include_str!("../../../sweeps/bg-cooney-fine.json"),
+    },
+    Builtin {
+        id: "bg-cooney-cost",
+        json: include_str!("../../../sweeps/bg-cooney-cost.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2328,7 +2400,25 @@ mod tests {
                 "ae-extent",
                 "ae-policy",
                 "ae-coupling",
-                "ae-coupling-rational"
+                "ae-coupling-rational",
+                "bg-fig1a",
+                "bg-fig1b",
+                "bg-fig1-caption",
+                "bg-fig1-either",
+                "bg-fig2a",
+                "bg-fig2b",
+                "bg-fig3",
+                "bg-fig4",
+                "bg-baseline",
+                "bg-readings",
+                "bg-mutation",
+                "bg-error",
+                "bg-groups",
+                "bg-benefit",
+                "bg-continuous",
+                "bg-ring",
+                "bg-cooney-fine",
+                "bg-cooney-cost"
             ]
         );
         for b in builtins() {
````

Modify `crates/sugarscape-cli/src/main.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index 9266c35..8b6764f 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -236,6 +236,7 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             ModelKind::Image => "its last generation",
             ModelKind::Farol => "its last round",
             ModelKind::Ants | ModelKind::Thresholds => "its last step",
+            ModelKind::Punishment => "its last period",
             ModelKind::Retirement => match &config {
                 ModelConfig::Retirement(c)
                     if c.stop_at_norm
````

- [ ] **Step 3: Run the tests and measure the sweeps**

Run: `cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm`, then `cargo build --release -p sugarscape-cli` and each `./target/release/sugarscape sweep --builtin <id> --quiet --summary-csv /tmp/<id>.csv --out /dev/null`.
Expected: PASS (WASM 51, including `punishment_sims_match_the_native_golden_entries`); each summary's means as its description records (about 13 minutes in all; the size sweeps take 40–120 s each).

- [ ] **Step 4: Format, lint, commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add sweeps/bg-fig1a.json sweeps/bg-fig1b.json sweeps/bg-fig1-caption.json sweeps/bg-fig1-either.json sweeps/bg-fig2a.json sweeps/bg-fig2b.json sweeps/bg-fig3.json sweeps/bg-fig4.json sweeps/bg-baseline.json sweeps/bg-readings.json sweeps/bg-mutation.json sweeps/bg-error.json sweeps/bg-groups.json sweeps/bg-benefit.json sweeps/bg-continuous.json sweeps/bg-ring.json sweeps/bg-cooney-fine.json sweeps/bg-cooney-cost.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli crates/sugarscape-wasm/tests/web.rs
```
```bash
git commit -m "Measure Altruistic Punishment: eighteen sweeps, the CLI's stop and WASM agreement

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/compare-presets.test.ts`, `web/src/engine.test.ts`, `web/src/experiments/form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the WASM build of Tasks 1–2.
- Produces: `PunishmentConfig`, `PunishmentStats`, `PunisherView`, `PunishmentGroupView`, `PunishmentInspection` (types.ts); `isPunishmentView` (checked first: it tests `punishment`); `MODEL_CHARTS.punishment`; the Compare entry `bg-base-vs-none`; the color mode `acts`.

- [ ] **Step 1: Write the failing tests**

Modify `web/src/models.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index d924fa8..84cbac3 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -6,6 +6,7 @@ import {
   isAgreementView,
   isAntsView,
   isThresholdsView,
+  isPunishmentView,
   isRetirementView,
   isFarolView,
   isCivilView,
@@ -120,6 +121,29 @@ describe('the presets menu', () => {
   });
 });
 
+describe('the punishment model', () => {
+  it('is read by its tag, and its inspections by `punishment`, before the others with a panel', () => {
+    const c = { model: 'punishment', stop_at: 2000 } as unknown as ModelConfig;
+    expect(modelOf(c)).toBe('punishment');
+    const cell = { site: { x: 1, y: 2 }, panel: 'groups', group: null, agent: null, period: null, cooperation: null, punishment: null } as unknown as AnyInspection;
+    const ret = { site: { x: 1, y: 2 }, panel: 'population', age: 65, retirements: null, exposed: null, period: null, retired: null, member: null, agent: null } as unknown as AnyInspection;
+    expect([cell, ret].map(isPunishmentView)).toEqual([true, false]);
+    expect([isRetirementView(cell), isThresholdsView(cell)]).toEqual([false, false]);
+  });
+
+  it('colors four ways, has no overlays, and ends at its last period', () => {
+    expect(COLOR_MODES.punishment).toEqual([
+      ['type', 'Type'],
+      ['acts', 'Acts'],
+      ['payoff', 'Payoff'],
+      ['group', 'Group'],
+    ]);
+    expect(MODEL_OVERLAYS.punishment).toEqual([]);
+    const c = (stop_at: number) => ({ model: 'punishment', stop_at }) as unknown as ModelConfig;
+    expect([finishesUnpredictably(c(2000)), ticksLeft(c(2000), 500), ticksLeft(c(0), 500)]).toEqual([false, 1500, Infinity]);
+  });
+});
+
 describe('the retirement model', () => {
   it('is read by its tag, and its inspections by `exposed`, before the others with a panel', () => {
     const c = { model: 'retirement', stop_at: 0, stop_at_norm: false } as unknown as ModelConfig;
````

Modify `web/src/compare-presets.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index c438ab9..fefc285 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -43,6 +43,11 @@ describe('compare presets', () => {
     expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
   });
 
+  it('pairs punishment with its absence', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['bg-base-vs-none', 'bg-base', 'bg-none', 'With vs without punishment — Altruistic Punishment (Compare)']);
+  });
+
   it('pairs 15 % and 5 % rational retirees', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['ae-rapid-vs-slow', 'ae-rapid', 'ae-slow', '15 % vs 5 % rational — Retirement (Compare)']);
````

Modify `web/src/engine.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index c2ccd86..131e92e 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1154,6 +1154,7 @@ describe('Engine with other models', () => {
     expect(finishedNotice({ model: 'farol', stop_at: 100 } as unknown as ModelConfig, 100)).toBe('This run has reached its last round (100) — Reset to run it again');
     expect(finishedNotice({ model: 'ants', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last step (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'thresholds', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last step (50) — Reset to run it again');
+    expect(finishedNotice({ model: 'punishment', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last period (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'retirement', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last period (50) — Reset to run it again');
     expect(finishedNotice({ model: 'retirement', stop_at_norm: true } as unknown as ModelConfig, 16)).toBe('The retirement norm has set in at t = 16 — Reset to run it again');
     expect(finishedNotice({ model: 'retirement', stop_at_norm: true, stop_at: 20 } as unknown as ModelConfig, 20)).toBe('This run has reached its last period (20) — Reset to run it again');
````

Modify `web/src/experiments/form.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index b5a42be..ea9d819 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -147,6 +147,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('punishment')).toMatchObject({
+      x: { path: 'size', values: '4,8,16,32,64,128,256' },
+      ticks: 2000,
+      metric: { kind: 'final', series: 'long_run' },
+    });
     expect(defaultForm('retirement')).toMatchObject({
       x: { path: 'rational', values: '0.02,0.05,0.1,0.15,0.2,0.25' },
       ticks: 400,
````

Modify `web/src/ui/series-data.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index 9ba46f5..4d0c3e8 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,14 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('punishment charts', () => {
+  it('chart the types, cooperation with its long-run average, payoff and conflict over periods', () => {
+    expect(MODEL_CHARTS.punishment.map((c) => c.title)).toEqual(['Types', 'Cooperation', 'Payoff', 'Conflict']);
+    expect(MODEL_CHARTS.punishment[1].lines.map((l) => l.key)).toEqual(['cooperation', 'long_run', 'acts']);
+    expect(timeAxisLabel('punishment')).toBe('Periods');
+  });
+});
+
 describe('retirement charts', () => {
   it('chart the retired share (by group when there are groups), retirement ages and the transition over periods', () => {
     expect(MODEL_CHARTS.retirement.map((c) => c.title)).toEqual(['Retired share', 'Retired share', 'Retirement age', 'Transition', 'Group transitions']);
````

Modify `web/src/determinism.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index 1b2167c..da3ad3f 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -13,6 +13,9 @@ import { InlineTransport } from './transport';
 import { decodeShare, encodeShare } from './share';
 import type {
   AgreementConfig,
+  PunishmentConfig,
+  PunishmentInspection,
+  PunishmentStats,
   RetirementConfig,
   RetirementInspection,
   RetirementStats,
@@ -727,6 +730,28 @@ describe('the social-structure model through the engine', () => {
   });
 });
 
+describe('the punishment model through the engine', () => {
+  it('stops at its last period and inspects an agent, its group and a period', async () => {
+    const r = presets.find((p) => p.id === 'bg-base')!;
+    const config = { ...structuredClone(r.config as PunishmentConfig), groups: 16, size: 8, stop_at: 50, window: 20 };
+    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'acts' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as PunishmentStats;
+    expect([e.finished, ends, e.tick]).toEqual([true, 1, 50]);
+    expect(s.long_run).not.toBeNull();
+    expect(s.contributors + s.punishers + s.defectors).toBeCloseTo(1, 9);
+    // 16 groups of 8: six groups a row, 3 × 3 cells of 8 pixels, the first at (2, 2).
+    await e.select(3, 3);
+    const v = e.inspection!.view as PunishmentInspection;
+    expect([v.panel, v.agent?.id, v.group?.index]).toEqual(['groups', 1, 0]);
+    await e.select(10, 100);
+    expect((e.inspection!.view as PunishmentInspection).panel).toBe('time');
+  });
+});
+
 describe('the retirement model through the engine', () => {
   it('stops at the norm and inspects an agent, an age and a period', async () => {
     const r = presets.find((p) => p.id === 'ae-rapid')!;
````

- [ ] **Step 2: Run them to see them fail**

Run: `cd web && npm ci && npm run wasm && npx vitest run`
Expected: failures in the six files above.

- [ ] **Step 3: Carry the model through the page**

Modify `web/src/types.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/types.ts b/web/src/types.ts
index f8c97f9..8c8a5ca 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -112,7 +112,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -511,7 +511,7 @@ export interface AgreementConfig {
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig;
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig;
 
 /**
  * Arthur's El Farol bar and Challet and Zhang's minority game (milestone 23), with Challet, Marsili
@@ -766,6 +766,89 @@ export interface RetirementInspection {
   agent: null;
 }
 
+/**
+ * Boyd, Gintis, Bowles and Richerson's altruistic punishment (milestone 27): groups of contributors,
+ * defectors and punishers, payoff-biased imitation, intergroup conflict and mutation.
+ */
+export interface PunishmentConfig {
+  model: 'punishment';
+  groups: number;
+  size: number;
+  cost: number;
+  punish_cost: number;
+  fine: number;
+  punishing: 'variable' | 'fixed';
+  fixed_cost: number;
+  benefit: number;
+  baseline: number;
+  error: number;
+  mixing: number;
+  mutation: number;
+  conflict: number;
+  pairing: 'paired' | 'either' | 'challenge';
+  victory: 'defectors' | 'payoff' | 'tanh';
+  sensitivity: number;
+  counted: 'types' | 'acts';
+  erring: 'others' | 'none' | 'self';
+  imitation: 'together' | 'in_turn';
+  refill: 'copy' | 'split';
+  traits: 'discrete' | 'continuous';
+  structure: 'groups' | 'ring';
+  start: 'one_punisher_group' | 'all_defectors';
+  window: number;
+  stop_at: number;
+}
+
+export interface PunishmentStats {
+  tick: number;
+  cooperation: number;
+  contributors: number;
+  punishers: number;
+  defectors: number;
+  punishment: number;
+  /** This period's share cooperating and mean payoff (null before the first period). */
+  acts: number | null;
+  payoff: number | null;
+  conflicts: number;
+  extinctions: number;
+  spread: number;
+  /** The mean cooperation over the long-run window so far (null before it). */
+  long_run: number | null;
+}
+
+export interface PunisherView {
+  id: number;
+  group: number;
+  kind: 'contributor' | 'defector' | 'punisher' | null;
+  cooperate: number;
+  punish: number;
+  cooperated: boolean;
+  punished: boolean;
+  payoff: number;
+}
+
+export interface PunishmentGroupView {
+  index: number;
+  contributors: number;
+  punishers: number;
+  defectors: number;
+  acts: number;
+  payoff: number;
+  last_conflict: number | null;
+  lost: boolean;
+}
+
+/** A cell of the punishment frame: an agent and its group, or a period of the time strip. */
+export interface PunishmentInspection {
+  site: { x: number; y: number };
+  panel: 'groups' | 'time' | null;
+  group: PunishmentGroupView | null;
+  agent: PunisherView | null;
+  period: number | null;
+  cooperation: number | null;
+  punishment: number | null;
+}
+
 /** A preset: `title` is the menu's plain headline; `source` and `name` are its figure or paper and its rules. */
 export interface Preset { id: string; title: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -1075,7 +1158,7 @@ export interface AgreementStats {
   stable_at: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats;
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -1405,7 +1488,7 @@ export interface AgreementInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection;
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -1466,7 +1549,8 @@ export type ColorMode =
   | 'crowd'
   | 'status'
   | 'type'
-  | 'group';
+  | 'group'
+  | 'acts';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
````

Modify `web/src/models.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.ts b/web/src/models.ts
index 3876917..dcd4c6f 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -1,6 +1,8 @@
 // Which model a config is (milestones 9–21), and what each model offers the page.
 import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
 import type {
+  PunishmentConfig,
+  PunishmentInspection,
   RetirementConfig,
   RetirementInspection,
   ThresholdsConfig,
@@ -43,7 +45,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -67,12 +69,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   ants: 'Ants and Recruitment',
   thresholds: 'Threshold Models',
   retirement: 'The Timing of Retirement',
+  punishment: 'Altruistic Punishment',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment'
     ? tag
     : 'sugarscape';
 }
@@ -166,6 +169,11 @@ export function isImageView(v: AnyInspection): v is ImageInspection {
   return 'cell' in v && 'group' in v;
 }
 
+/** A cell of the punishment frame (a panel, an agent and its group, and a period's `punishment`); check it first. */
+export function isPunishmentView(v: AnyInspection): v is PunishmentInspection {
+  return 'panel' in v && 'punishment' in v;
+}
+
 /** A cell of the retirement frame (a panel, an agent as `member`, and an age's `exposed`); check it first. */
 export function isRetirementView(v: AnyInspection): v is RetirementInspection {
   return 'panel' in v && 'exposed' in v;
@@ -210,6 +218,7 @@ export function ticksLeft(c: ModelConfig, tick: number): number {
   if (modelOf(c) === 'ants' && (c as AntsConfig).stop_at > 0) return Math.max(0, (c as AntsConfig).stop_at - tick);
   if (modelOf(c) === 'thresholds' && (c as ThresholdsConfig).stop_at > 0) return Math.max(0, (c as ThresholdsConfig).stop_at - tick);
   if (modelOf(c) === 'retirement' && (c as RetirementConfig).stop_at > 0) return Math.max(0, (c as RetirementConfig).stop_at - tick);
+  if (modelOf(c) === 'punishment' && (c as PunishmentConfig).stop_at > 0) return Math.max(0, (c as PunishmentConfig).stop_at - tick);
   return Infinity;
 }
 
@@ -377,6 +386,13 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['threshold', 'Threshold'],
     ['group', 'Group'],
   ],
+  // Each agent's type (blended for continuous traits); what it did this period; its payoff; its group's share of defectors.
+  punishment: [
+    ['type', 'Type'],
+    ['acts', 'Acts'],
+    ['payoff', 'Payoff'],
+    ['group', 'Group'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -401,4 +417,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   ants: [],
   thresholds: [],
   retirement: [],
+  punishment: [],
 };
````

Modify `web/src/engine.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.ts b/web/src/engine.ts
index a8f7031..3fee776 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -54,6 +54,7 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
   if (modelOf(config) === 'farol') return `This run has reached its last round (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'ants' || modelOf(config) === 'thresholds') return `This run has reached its last step (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'punishment') return `This run has reached its last period (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'retirement') {
     const c = config as { stop_at_norm?: boolean; stop_at?: number };
     const capped = (c.stop_at ?? 0) > 0 && tick >= (c.stop_at ?? 0);
````

Modify `web/src/compare-presets.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index d278148..906c128 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -176,6 +176,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'ae-rapid',
     b: 'ae-slow',
   },
+  {
+    id: 'bg-base-vs-none',
+    label: 'With vs without punishment — Altruistic Punishment (Compare)',
+    a: 'bg-base',
+    b: 'bg-none',
+  },
 ];
 
 /**
````

Modify `web/src/experiments/form.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index 2374930..0689056 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -104,6 +104,10 @@ export function defaultForm(model: ModelKind = 'sugarscape', config?: ModelConfi
     // The built-in ef-predictors' axis: how far attendance swings against predictors per agent.
     return { ...form, x: { path: 'strategies', values: '2:24:2' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'fluctuation' } };
   }
+  if (model === 'punishment') {
+    // The built-in bg-fig1b's axis: the long-run cooperation (the last 1 000 of 2 000 periods) against group size.
+    return { ...form, x: { path: 'size', values: '4,8,16,32,64,128,256' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'long_run' } };
+  }
   if (model === 'retirement') {
     // The built-in ae-rational's axis: the period the age 65 norm sets in (kept once reached) against the rational share.
     return {
````

Modify `web/src/ui/series-data.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index 277a209..3c1e284 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -627,6 +627,34 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
       shown: hasRetirementGroups,
     },
   ],
+  punishment: [
+    {
+      title: 'Types',
+      lines: [
+        { key: 'contributors', label: 'Contributors', color: '--blue' },
+        { key: 'punishers', label: 'Punishers', color: '--c2' },
+        { key: 'defectors', label: 'Defectors', color: '--red' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'Cooperation',
+      lines: [
+        { key: 'cooperation', label: 'Contributors and punishers', color: '--c1' },
+        { key: 'long_run', label: 'Long-run average', color: '--c4' },
+        { key: 'acts', label: 'Cooperated this period', color: '--c3' },
+      ],
+      range: [0, 1],
+    },
+    { title: 'Payoff', lines: [{ key: 'payoff', label: 'Mean payoff', color: '--c1' }] },
+    {
+      title: 'Conflict',
+      lines: [
+        { key: 'conflicts', label: 'Conflicts', color: '--red' },
+        { key: 'spread', label: 'Spread of groups’ cooperation', color: '--c2' },
+      ],
+    },
+  ],
 };
 
 /**
@@ -634,7 +662,7 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
  * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' ? 'Periods' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
+  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
````

Modify `web/src/ui/inspect-panel.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index 4630037..efde89a 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -3,11 +3,12 @@ import { dpdRows } from '../dpd';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
 import { imageRows } from '../image-scoring';
-import { isAgreementView, isAntsView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isAgreementView, isAntsView, isPunishmentView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
   AntsInspection,
+  PunishmentInspection,
   RetirementInspection,
   ThresholdsInspection,
   FarolInspection,
@@ -290,6 +291,26 @@ export class InspectPanel {
     return rows;
   }
 
+  /** An agent and its group, or a period of the time strip. */
+  private punishmentRows(view: PunishmentInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    const pct = (x: number) => `${fmt(100 * x)} %`;
+    if (view.panel === 'time')
+      return [row('Period', String(view.period)), row('Cooperation', pct(view.cooperation ?? 0)), row('Punishment', pct(view.punishment ?? 0))];
+    const a = view.agent;
+    const g = view.group;
+    if (!a || !g) return [row('Point', 'between groups')];
+    const traits = a.kind ?? `cooperates ${pct(a.cooperate)} · punishes ${pct(a.punish)}`;
+    const did = `${a.cooperated ? 'cooperated' : 'defected'}${a.punished ? ' and punished' : ''}`;
+    return [
+      row('Agent', `#${a.id} · ${traits}`),
+      row('This period', `${did} · payoff ${fmt(a.payoff)}`),
+      row('Group', `#${g.index + 1} · ${pct(g.contributors)} contributors · ${pct(g.punishers)} punishers · ${pct(g.defectors)} defectors`),
+      row('Group this period', `${pct(g.acts)} cooperated · mean payoff ${fmt(g.payoff)}`),
+      row('Last conflict', g.last_conflict === null ? 'none yet' : `period ${g.last_conflict} (${g.lost ? 'lost: replaced' : 'won'})`),
+    ];
+  }
+
   /** An agent of the retirement population, an age's retirements, or a period. */
   private retirementRows(view: RetirementInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -527,6 +548,8 @@ export class InspectPanel {
             ? this.normsRows(view)
           : isAgreementView(view)
             ? this.agreementRows(view)
+          : isPunishmentView(view)
+            ? this.punishmentRows(view)
           : isRetirementView(view)
             ? this.retirementRows(view)
           : isThresholdsView(view)
````

- [ ] **Step 4: Run the page's build and tests**

Run: `cd web && npm run build && npm test`
Expected: the build succeeds; 728 tests pass (49 files).

- [ ] **Step 5: Commit**

```bash
git add web/src
```
```bash
git commit -m "Carry Altruistic Punishment through the page

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

- [ ] **Step 6 (controller): check it in the browser**

`cd web && npm run build && npx vite preview`, then with `?debug`: `bg-base` fills its 16 × 8 mosaic of groups from one green block, blue and green spreading, red groups replaced; `bg-either` in Group colors, a defeated group framed; `bg-continuous` blended colors; `bg-ring` in Acts colors; `bg-large` in Payoff colors. Inspect an agent (traits, act, payoff, its group's shares and last conflict) and a period of the strip. Rules panel groups as listed, with the sensitivity shown only under tanh and the fixed cost only when fixed. Compare "With vs without punishment — Altruistic Punishment (Compare)". Experiments with a punishment preset: the default axis group size 4–256 against `long_run`; run the built-in `bg-fig1b`.

---

### Task 4: The survey's punishment claims

**Files:**
- Create: `survey/src/claims/punishment.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `sugarscape_core::punishment::{Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, PunishmentWorld, Start, Structure, Traits, Victory}`, `sugarscape_core::model::{Model, ModelConfig, ModelWorld}`, `crate::runner::model_after`, `crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict}`.
- Produces: 25 claims (`punishment.bg.*`, `punishment.janssen.*`, `punishment.cooney.*`).

- [ ] **Step 1: Write the claims**

Create `survey/src/claims/punishment.rs` with exactly this content:

````rust
//! Altruistic punishment (milestone 27): Boyd, Gintis, Bowles and Richerson
//! (2003), with Cooney's PDE model (2024) as the critique and Janssen's
//! NetLogo replication's readings. Every figure is the mean cooperation over
//! the last 1 000 of 2 000 periods (`long_run`), as in the paper. The
//! figures' values were read from the PDF at 300 dpi by marker (± 0.01).

use sugarscape_core::model::{Model, ModelConfig, ModelWorld};
use sugarscape_core::punishment::{
    Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, PunishmentWorld, Start,
    Structure, Traits, Victory,
};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const BGBR: &str = "Boyd, Gintis, Bowles & Richerson 2003, PNAS 100: 3531";
const COONEY: &str = "Cooney 2024, arXiv:2405.18419 (Bull. Math. Biol. 2025)";
const JANSSEN: &str = "Janssen, CoMSES 2223 (a NetLogo replication)";

/// The group sizes of the figures.
const SIZES: [u32; 7] = [4, 8, 16, 32, 64, 128, 256];

/// Fig. 1b (and Figs. 3 and 4's base curve), ε 0.015, read from the figure.
const FIG1B: [f64; 7] = [0.87, 0.88, 0.86, 0.83, 0.79, 0.64, 0.55];
/// Fig. 1b's lowest and highest conflict rates (the legend's 0.0075 and 0.03).
const FIG1B_LOW: [f64; 7] = [0.81, 0.82, 0.77, 0.68, 0.47, 0.16, 0.07];
const FIG1B_HIGH: [f64; 7] = [0.92, 0.93, 0.92, 0.91, 0.89, 0.79, 0.66];
/// Fig. 1a at ε 0.015 (the lines merge at about 0.09 from n 32).
const FIG1A: [f64; 7] = [0.76, 0.57, 0.20, 0.09, 0.09, 0.09, 0.09];
const FIG1A_LOW: [f64; 7] = [0.63, 0.37, 0.14, 0.09, 0.09, 0.09, 0.09];
const FIG1A_HIGH: [f64; 7] = [0.87, 0.79, 0.49, 0.11, 0.09, 0.08, 0.08];

/// Figs. 2–4, read the same way; a marker hidden under another curve takes
/// that curve's value.
const FIG2A: [[f64; 7]; 3] = [
    [0.79, 0.66, 0.34, 0.10, 0.09, 0.09, 0.09],
    [0.75, 0.57, 0.20, 0.10, 0.09, 0.09, 0.09],
    [0.55, 0.21, 0.11, 0.10, 0.08, 0.08, 0.08],
];
const FIG2B: [[f64; 7]; 3] = [
    [0.89, 0.88, 0.87, 0.85, 0.82, 0.71, 0.73],
    [0.87, 0.88, 0.86, 0.83, 0.77, 0.61, 0.57],
    [0.86, 0.87, 0.85, 0.72, 0.12, 0.06, 0.06],
];
const FIG3_WEAK: [f64; 7] = [0.80, 0.72, 0.58, 0.21, 0.07, 0.06, 0.06];
const FIG4_FIXED: [f64; 7] = [0.84, 0.78, 0.50, 0.10, 0.07, 0.06, 0.06];

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

fn config(edit: impl FnOnce(&mut PunishmentConfig)) -> PunishmentConfig {
    let mut c = PunishmentConfig::default();
    edit(&mut c);
    c
}

fn none(c: &mut PunishmentConfig) {
    c.fine = 0.0;
    c.punish_cost = 0.0;
}

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

/// `series` at the end of 2 000 periods, from 10 seeds.
fn final_of(c: PunishmentConfig, series: &str) -> Vec<f64> {
    let name = series.to_string();
    model_after(&ModelConfig::Punishment(c), &seeds(10), 2000, move |w| {
        w.model().latest_value(&name).unwrap()
    })
}

/// The long-run cooperation (the last 1 000 of 2 000 periods), 10 seeds.
fn long_run(c: PunishmentConfig) -> Vec<f64> {
    final_of(c, "long_run")
}

/// The mean of `series` over the last 1 000 of 2 000 periods, 10 seeds.
fn window(c: PunishmentConfig, series: &str) -> Vec<f64> {
    let name = series.to_string();
    model_after(&ModelConfig::Punishment(c), &seeds(10), 2000, move |w| {
        let s = w.model().series(&name).unwrap();
        s[1001..].iter().sum::<f64>() / (s.len() - 1001) as f64
    })
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The mean long-run cooperation at each of `sizes`.
fn curve(edit: impl Fn(&mut PunishmentConfig), sizes: &[u32]) -> Vec<f64> {
    sizes
        .iter()
        .map(|&n| {
            mean(&long_run(config(|c| {
                edit(c);
                c.size = n;
            })))
        })
        .collect()
}

fn show(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| format!("{x:.2}")).collect();
    format!("[{}]", parts.join(", "))
}

/// The largest gap between a measured curve and the figure's.
fn gap(measured: &[f64], figure: &[f64]) -> f64 {
    measured
        .iter()
        .zip(figure)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max)
}

/// The mean gap: the fit rule every "reproduces Fig. 1" claim uses (≤ 0.05).
fn mean_gap(measured: &[f64], figure: &[f64]) -> f64 {
    distance(measured, figure) / measured.len() as f64
}

/// The summed gap.
fn distance(measured: &[f64], figure: &[f64]) -> f64 {
    measured
        .iter()
        .zip(figure)
        .map(|(a, b)| (a - b).abs())
        .sum()
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "punishment.bg.fig1a",
            item: "bg-fig1a",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 1a, without punishment: 'Group selection is ineffective unless groups are quite small' (ε 0.015: long-run cooperation above 0.4 at n 4, below 0.15 from n 32)",
            check: |_| {
                let v = curve(none, &[4, 32, 64]);
                outcome(v[0] > 0.4 && v[1] < 0.15 && v[2] < 0.15, format!("{} at n 4, 32, 64 (figure 0.76, 0.09, 0.09)", show(&v)))
            },
        },
        Claim {
            id: "punishment.bg.fig1b",
            item: "bg-fig1b",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 1b: 'When there is punishment … group selection can maintain cooperation in substantially larger groups' (ε 0.015: the mean gap to the figure over n 4–256 at most 0.05 — the figure has 0.79 at n 64, 0.64 at 128, 0.55 at 256)",
            check: |_| {
                let v = curve(|_| {}, &SIZES);
                let g = mean_gap(&v, &FIG1B);
                outcome(g <= 0.05, format!("{} at n 4–256; figure {}; mean gap {g:.2}", show(&v), show(&FIG1B)))
                    .with("The shape holds (more cooperation with punishment at every size); the reach does not.")
            },
        },
        Claim {
            id: "punishment.bg.fig1b-helps",
            item: "bg-fig1b",
            source: Source::Book,
            citation: BGBR,
            text: "Punishment sustains more cooperation than its absence in groups too large for group selection alone (n 32, ε 0.015: long-run cooperation with punishment greater)",
            check: |_| {
                let with = long_run(config(|c| c.size = 32));
                let without = long_run(config(|c| {
                    none(c);
                    c.size = 32;
                }));
                greater(&with, &without, "punishment", "none")
            },
        },
        Claim {
            id: "punishment.bg.extinction",
            item: "bg-fig1b",
            source: Source::Book,
            citation: BGBR,
            text: "'increasing the rate of extinction increases the long run average amount of cooperation' (n 64 with punishment: ε 0.03 greater than 0.015, and 0.015 greater than 0.0075)",
            check: |_| {
                let at = |eps: f64| {
                    long_run(config(|c| {
                        c.size = 64;
                        c.conflict = eps;
                    }))
                };
                let (lo, mid, hi) = (at(0.0075), at(0.015), at(0.03));
                all_of(vec![
                    ("0.03 over 0.015".into(), greater(&hi, &mid, "ε 0.03", "ε 0.015")),
                    ("0.015 over 0.0075".into(), greater(&mid, &lo, "ε 0.015", "ε 0.0075")),
                ])
            },
        },
        Claim {
            id: "punishment.bg.caption",
            item: "bg-fig1-caption",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 1's caption: the conflict rates plotted are '0.075, 0.015, and 0.003' (the legend reads 0.0075, 0.015, 0.03). The caption's rates reproduce the figure's outer curves more closely than the legend's, and than twice the legend's (summed gap over both panels at n 4–256)",
            check: |_| {
                let at = |eps: f64, punish: bool| {
                    curve(
                        |c| {
                            c.conflict = eps;
                            if !punish {
                                none(c);
                            }
                        },
                        &SIZES,
                    )
                };
                let fig = [FIG1A_LOW, FIG1A_HIGH, FIG1B_LOW, FIG1B_HIGH];
                let total = |low: f64, high: f64| {
                    let m = [at(low, false), at(high, false), at(low, true), at(high, true)];
                    m.iter().zip(&fig).map(|(a, b)| distance(a, b)).sum::<f64>()
                };
                let legend = total(0.0075, 0.03);
                let caption = total(0.003, 0.075);
                let doubled = total(0.015, 0.06);
                outcome(
                    caption < legend.min(doubled),
                    format!("summed gap over 28 points: caption's rates {caption:.2}, legend's {legend:.2}, twice the legend's {doubled:.2}"),
                )
                .with("At the stated rates the model's cooperation collapses sooner than the figure's, so the caption's higher top rate looks closer than the legend's; twice the legend's fits best (see punishment.bg.either).")
            },
        },
        Claim {
            id: "punishment.bg.either",
            item: "bg-fig1-either",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: Fig. 1 is the stated model when either group of a pair can start the conflict, each with probability ε — a pair fights with probability 2ε − ε², about twice the text's, and a group dies at about ε a period, not the ε/2 = 0.0075 their Methods derive. At the legend's ε 0.0075, 0.015 and 0.03, all six curves (both panels) within a mean gap of 0.05 of the figure over n 4–256",
            check: |_| {
                let figures = [
                    (0.0075, false, FIG1A_LOW),
                    (0.015, false, FIG1A),
                    (0.03, false, FIG1A_HIGH),
                    (0.0075, true, FIG1B_LOW),
                    (0.015, true, FIG1B),
                    (0.03, true, FIG1B_HIGH),
                ];
                let parts = figures
                    .iter()
                    .map(|&(eps, punish, fig)| {
                        let v = curve(
                            |c| {
                                c.pairing = Pairing::Either;
                                c.conflict = eps;
                                if !punish {
                                    none(c);
                                }
                            },
                            &SIZES,
                        );
                        let g = mean_gap(&v, &fig);
                        let name = format!("{} ε {eps}", if punish { "1b" } else { "1a" });
                        (name, outcome(g <= 0.05, format!("{} against {} (mean gap {g:.3})", show(&v), show(&fig))))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "punishment.bg.either-others",
            item: "bg-fig1-either",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: the reading found from Fig. 1 (either group starts the conflict) reproduces the figures it was not found from — Fig. 2a and 2b (m 0.002, 0.01, 0.05), Fig. 3 (p 0.4) and Fig. 4 (a fixed cost) at ε 0.015, each curve within a mean gap of 0.05 over n 4–256",
            check: |_| {
                let either = |c: &mut PunishmentConfig| c.pairing = Pairing::Either;
                let mut parts = Vec::new();
                for (k, &m) in [0.002, 0.01, 0.05].iter().enumerate() {
                    for punish in [false, true] {
                        let v = curve(
                            |c| {
                                either(c);
                                c.mixing = m;
                                if !punish {
                                    none(c);
                                }
                            },
                            &SIZES,
                        );
                        let fig = if punish { FIG2B[k] } else { FIG2A[k] };
                        let g = mean_gap(&v, &fig);
                        let name = format!("{} m {m}", if punish { "2b" } else { "2a" });
                        parts.push((name, outcome(g <= 0.05, format!("{} against {} ({g:.3})", show(&v), show(&fig)))));
                    }
                }
                for (name, fig, edit) in [
                    ("3 p 0.4", FIG3_WEAK, (|c: &mut PunishmentConfig| c.fine = 0.4) as fn(&mut PunishmentConfig)),
                    ("4 fixed", FIG4_FIXED, |c| c.punishing = Punishing::Fixed),
                ] {
                    let v = curve(
                        |c| {
                            either(c);
                            edit(c);
                        },
                        &SIZES,
                    );
                    let g = mean_gap(&v, &fig);
                    parts.push((name.into(), outcome(g <= 0.05, format!("{} against {} ({g:.3})", show(&v), show(&fig)))));
                }
                all_of(parts)
            },
        },
        Claim {
            id: "punishment.bg.fig2",
            item: "bg-fig2b",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 2: 'when the migration rate increases, levels of cooperation fall precipitously … at higher rates of mixing, cooperation does not persist in the largest groups' (with punishment at n 64: m 0.002 greater than 0.01 greater than 0.05; m 0.05 below 0.2 at n 128)",
            check: |_| {
                let at = |m: f64, n: u32| {
                    long_run(config(|c| {
                        c.mixing = m;
                        c.size = n;
                    }))
                };
                let (lo, mid, hi) = (at(0.002, 64), at(0.01, 64), at(0.05, 64));
                let big = mean(&at(0.05, 128));
                all_of(vec![
                    ("0.002 over 0.01".into(), greater(&lo, &mid, "m 0.002", "m 0.01")),
                    ("0.01 over 0.05".into(), greater(&mid, &hi, "m 0.01", "m 0.05")),
                    ("gone in large groups".into(), outcome(big < 0.2, format!("m 0.05, n 128: {big:.2}"))),
                ])
            },
        },
        Claim {
            id: "punishment.bg.fig2-reach",
            item: "bg-fig2b",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 2b: 'group selection can maintain cooperation in larger groups for all rates of mixing' — at m 0.002 about 0.71 at n 128 and 0.73 at 256 (within 0.1)",
            check: |_| {
                let v = curve(|c| c.mixing = 0.002, &[128, 256]);
                outcome(gap(&v, &[0.71, 0.73]) <= 0.1, format!("{} at n 128, 256", show(&v)))
            },
        },
        Claim {
            id: "punishment.bg.fig3",
            item: "bg-fig3",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 3: 'Lower values of p result in much lower levels of cooperation' (n 16: p 0.8 greater than 0.4; p 0.4 below 0.25 at n 32 — the figure's 0.21)",
            check: |_| {
                let hi = long_run(config(|c| c.size = 16));
                let lo = long_run(config(|c| {
                    c.size = 16;
                    c.fine = 0.4;
                }));
                let at32 = mean(&long_run(config(|c| {
                    c.size = 32;
                    c.fine = 0.4;
                })));
                all_of(vec![
                    ("lower".into(), greater(&hi, &lo, "p 0.8", "p 0.4")),
                    ("much lower".into(), outcome(at32 < 0.25, format!("p 0.4, n 32: {at32:.2}"))),
                ])
            },
        },
        Claim {
            id: "punishment.bg.fig4",
            item: "bg-fig4",
            source: Source::Book,
            citation: BGBR,
            text: "Fig. 4: 'Punishment does not aid in the evolution of cooperation when the costs born by punishers are fixed' (n 32: a fixed cost c is no better than no punishment — both below 0.2 — while the variable cost is above 0.5)",
            check: |_| {
                let fixed = mean(&long_run(config(|c| {
                    c.size = 32;
                    c.punishing = Punishing::Fixed;
                })));
                let without = mean(&long_run(config(|c| {
                    c.size = 32;
                    none(c);
                })));
                let variable = mean(&long_run(config(|c| c.size = 32)));
                outcome(
                    fixed < 0.2 && without < 0.2 && variable > 0.5,
                    format!("fixed {fixed:.2}, none {without:.2}, variable {variable:.2}"),
                )
            },
        },
        Claim {
            id: "punishment.bg.calibration-spread",
            item: "bg-base",
            source: Source::Book,
            citation: BGBR,
            text: "'we set the cost of cooperation, c, and punishing, k, so that traits with this cost advantage would spread in 50 time periods' (at baseline 1: defectors, with an advantage c, go from 10 % to 90 % of a group of 512 in 35–65 periods)",
            check: |_| {
                // Group 0 starts all punishers (contributors here, with no
                // fines); mutation seeds it with defectors until they pass
                // 10 %, then stops. Groups never mix, so group 0 stands alone.
                let c = config(|c| {
                    c.groups = 2;
                    c.size = 512;
                    none(c);
                    c.error = 0.0;
                    c.mixing = 0.0;
                    c.conflict = 0.0;
                    c.mutation = 0.1;
                });
                let v = model_after(&ModelConfig::Punishment(c.clone()), &seeds(10), 0, move |w| {
                    let ModelWorld::Punishment(start) = w else { unreachable!() };
                    let mut pw = start.as_ref().clone();
                    let share = |pw: &PunishmentWorld| (0..pw.size()).filter(|&a| pw.traits(a).0 < 0.5).count() as f64 / pw.size() as f64;
                    while share(&pw) < 0.1 {
                        pw.step();
                    }
                    let mut still = c.clone();
                    still.mutation = 0.0;
                    Model::set_config(&mut pw, ModelConfig::Punishment(still)).unwrap();
                    let from = pw.tick;
                    while share(&pw) < 0.9 && pw.tick < from + 1000 {
                        pw.step();
                    }
                    (pw.tick - from) as f64
                });
                let m = mean(&v);
                outcome((35.0..=65.0).contains(&m), format!("{m:.1} periods from 10 % to 90 % (10 runs)"))
            },
        },
        Claim {
            id: "punishment.bg.calibration-mixing",
            item: "bg-base",
            source: Source::Book,
            citation: BGBR,
            text: "'The migration rate, m, was set so that … passive diffusion will cause two neighboring groups that are initially as different as possible to achieve the same trait frequencies in ≈50 time periods (m = 0.01)' (two groups of 512, one all punishers, one all defectors, no costs, errors, conflict or mutation: within 0.1 of each other by period 50)",
            check: |_| {
                let c = config(|c| {
                    c.groups = 2;
                    c.size = 512;
                    c.cost = 0.0;
                    none(c);
                    c.error = 0.0;
                    c.conflict = 0.0;
                    c.mutation = 0.0;
                });
                // With two groups, their difference is twice `spread`.
                let d = model_after(&ModelConfig::Punishment(c), &seeds(10), 50, |w| {
                    2.0 * w.model().latest_value("spread").unwrap()
                });
                let m = mean(&d);
                outcome(m <= 0.1, format!("difference {m:.2} after 50 periods (from 1)"))
                    .with("Each period a member meets the other group with probability m and copies it half the time, so the difference shrinks by about m a period: e^(−0.5) ≈ 0.6 remains at 50.")
            },
        },
        Claim {
            id: "punishment.bg.mutation",
            item: "bg-mutation",
            source: Source::Book,
            citation: BGBR,
            text: "'Decreasing the mutation rate substantially increases the long run average levels of cooperation' (n 32: μ 0.001 greater than 0.01, by at least 0.1)",
            check: |_| {
                let lo = long_run(config(|c| c.mutation = 0.001));
                let base = long_run(PunishmentConfig::default());
                let gain = mean(&lo) - mean(&base);
                all_of(vec![
                    ("greater".into(), greater(&lo, &base, "μ 0.001", "μ 0.01")),
                    ("substantially".into(), outcome(gain >= 0.1, format!("gain {gain:.2}"))),
                ])
            },
        },
        Claim {
            id: "punishment.bg.error",
            item: "bg-error",
            source: Source::Book,
            citation: BGBR,
            text: "'Increasing e, the error rate, reduces the long run average amount of cooperation' (n 32: e 0.02 greater than 0.1)",
            check: |_| {
                let base = long_run(PunishmentConfig::default());
                let hi = long_run(config(|c| c.error = 0.1));
                greater(&base, &hi, "e 0.02", "e 0.1")
            },
        },
        Claim {
            id: "punishment.bg.groups",
            item: "bg-groups",
            source: Source::Book,
            citation: BGBR,
            text: "'Reducing the number of groups, N, adds random noise to the results' — noise, not a shift (n 32: the long-run cooperation of 16 groups equivalent to 128's within 0.05)",
            check: |_| {
                let few = long_run(config(|c| c.groups = 16));
                let base = long_run(PunishmentConfig::default());
                equivalent(&few, &base, Some(0.05), "N 16", "N 128")
            },
        },
        Claim {
            id: "punishment.bg.benefit",
            item: "bg-benefit",
            source: Source::Book,
            citation: BGBR,
            text: "A per-capita benefit b/n with conflict decided by payoffs: 'For reasonable values of b (2c, 4c, and 8c), the results of this model are qualitatively similar' (at n 32, punishment sustains more cooperation than its absence at each b)",
            check: |_| {
                let parts = [0.4, 0.8, 1.6]
                    .iter()
                    .map(|&b| {
                        let set = move |c: &mut PunishmentConfig| {
                            c.benefit = b;
                            c.victory = Victory::Payoff;
                        };
                        let with = long_run(config(set));
                        let without = long_run(config(|c| {
                            set(c);
                            none(c);
                        }));
                        (format!("b {b}"), greater(&with, &without, "punishment", "none"))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "punishment.bg.continuous",
            item: "bg-continuous",
            source: Source::Book,
            citation: BGBR,
            text: "Continuous traits: 'The steady-state mean levels of cooperation in this model are similar to the base model' (within 0.15 of the base model at n 4, 32, 128, 256)",
            check: |_| {
                let sizes = [4, 32, 128, 256];
                let cont = curve(|c| c.traits = Traits::Continuous, &sizes);
                let base = curve(|_| {}, &sizes);
                let g = gap(&cont, &base);
                outcome(g <= 0.15, format!("continuous {}; base {} (largest gap {g:.2})", show(&cont), show(&base)))
                    .with("Uniform mutants keep the mean punishment trait near ½, and a defector then pays about p/2 = 0.4 > c: cooperation pays within groups.")
            },
        },
        Claim {
            id: "punishment.bg.ring",
            item: "bg-ring",
            source: Source::Book,
            citation: BGBR,
            text: "A ring of groups without extinction: 'We could find no reasonable parameter combination that led to significant long run average levels of cooperation in this last model' (b 2c and 8c, m 0.01, n 4, 8, 16: all below 0.3)",
            check: |_| {
                let mut worst = (0.0, 0.0, 0);
                for b in [0.4, 1.6] {
                    let v = curve(
                        |c| {
                            c.structure = Structure::Ring;
                            c.benefit = b;
                        },
                        &[4, 8, 16],
                    );
                    for (i, &x) in v.iter().enumerate() {
                        if x > worst.0 {
                            worst = (x, b, [4, 8, 16][i]);
                        }
                    }
                }
                outcome(worst.0 < 0.3, format!("highest {:.2} (b {}, n {})", worst.0, worst.1, worst.2))
            },
        },
        Claim {
            id: "punishment.bg.hundred",
            item: "bg-large",
            source: Source::Book,
            citation: BGBR,
            text: "'With parameter values chosen to represent cultural evolution in small-scale societies, cooperation is sustained in groups on the order of 100 individuals' (n 128: long-run cooperation at least 0.5)",
            check: |_| {
                let v = mean(&long_run(config(|c| c.size = 128)));
                outcome(v >= 0.5, format!("{v:.2} at n 128 (figure 0.64)"))
            },
        },
        Claim {
            id: "punishment.bg.baseline",
            item: "bg-baseline",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: the paper never states the payoff costs are subtracted from; at the stated conflict rate, some baseline reproduces Figs. 1a and 1b together (ε 0.015: a mean gap of at most 0.05 to both panels over n 4–256, for a baseline of 1, 2, 3 or 4)",
            check: |_| {
                let mut lines = Vec::new();
                let mut any = false;
                for base in [1.0, 2.0, 3.0, 4.0] {
                    let b = curve(|c| c.baseline = base, &SIZES);
                    let a = curve(
                        |c| {
                            c.baseline = base;
                            none(c);
                        },
                        &SIZES,
                    );
                    let (ga, gb) = (mean_gap(&a, &FIG1A), mean_gap(&b, &FIG1B));
                    any |= ga <= 0.05 && gb <= 0.05;
                    lines.push(format!("baseline {base}: 1a {} ({ga:.2}), 1b {} ({gb:.2})", show(&a), show(&b)));
                }
                outcome(any, lines.join("; "))
                    .with("A higher baseline weakens selection: punishment reaches larger groups, but without it the floor rises above the figure's 0.09. When either group can start a conflict, baseline 1 fits both (punishment.bg.either).")
            },
        },
        Claim {
            id: "punishment.janssen.readings",
            item: "bg-readings",
            source: Source::Comment,
            citation: JANSSEN,
            text: "Janssen's readings together — a benefit 0.5 × the share cooperating, every group challenging one (about twice the conflict), d counted by acts, imitation in turn, erring punishers punishing themselves — reproduce Fig. 1 at ε 0.015 (a mean gap of at most 0.05 to both panels over n 4–256)",
            check: |_| {
                let janssen = |c: &mut PunishmentConfig| {
                    c.benefit = 0.5;
                    c.pairing = Pairing::Challenge;
                    c.counted = Counted::Acts;
                    c.imitation = Imitation::InTurn;
                    c.erring = Erring::Itself;
                };
                let b = curve(janssen, &SIZES);
                let a = curve(
                    |c| {
                        janssen(c);
                        none(c);
                    },
                    &SIZES,
                );
                let (ga, gb) = (mean_gap(&a, &FIG1A), mean_gap(&b, &FIG1B));
                outcome(ga <= 0.05 && gb <= 0.05, format!("1a {} ({ga:.3}); 1b {} ({gb:.3})", show(&a), show(&b)))
                    .with("His challenge pairing roughly doubles the conflict; his benefit lifts cooperation in small groups without punishment.")
            },
        },
        Claim {
            id: "punishment.cooney.fine",
            item: "bg-cooney-fine",
            source: Source::Comment,
            citation: COONEY,
            text: "Under conflict decided by average payoffs normalized by the widest possible difference, 'a non-monotonic dependence of long-time average payoff on the strength of punishment' — and (Remark 6.1) not under the Fermi (tanh) rule (b 2c, n 32: the mean payoff over the last 1 000 periods dips below p = 0's and recovers above the dip, each by Mann–Whitney p < 0.01; under tanh, no such dip)",
            check: |_| {
                let fines = [0.0, 0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.2, 1.6, 2.4];
                let dip = |victory: Victory| {
                    let runs: Vec<Vec<f64>> = fines
                        .iter()
                        .map(|&p| {
                            window(
                                config(|c| {
                                    c.benefit = 0.4;
                                    c.victory = victory;
                                    c.fine = p;
                                }),
                                "payoff",
                            )
                        })
                        .collect();
                    let means: Vec<f64> = runs.iter().map(|r| mean(r)).collect();
                    let low = (1..fines.len() - 1)
                        .min_by(|&a, &b| means[a].total_cmp(&means[b]))
                        .unwrap();
                    let down = greater(&runs[0], &runs[low], "p 0", "the dip");
                    let up = greater(&runs[fines.len() - 1], &runs[low], "p 2.4", "the dip");
                    let found = down.verdict == Verdict::Holds && up.verdict == Verdict::Holds;
                    (found, format!("{} at p {:?}; lowest at p {}", show(&means), fines, fines[low]))
                };
                let (normalized, a) = dip(Victory::Payoff);
                let (tanh, b) = dip(Victory::Tanh);
                all_of(vec![
                    ("a dip, normalized".into(), outcome(normalized, a)),
                    ("none under tanh".into(), outcome(!tanh, b)),
                ])
                .with("In this finite, mutating population, weak punishment costs its punishers without deterring anyone, whichever rule decides conflicts.")
            },
        },
        Claim {
            id: "punishment.cooney.cost",
            item: "bg-cooney-cost",
            source: Source::Comment,
            citation: COONEY,
            text: "Under normalized payoff conflict, 'increasing the cost of punishing defectors can increase the level of altruistic punishment at steady state' (b 2c, n 32: for some k₁ < k₂ in 0.05, 0.1, 0.2, 0.4, 0.8, the mean punishment over the last 1 000 periods is greater at k₂, Mann–Whitney p < 0.01)",
            check: |_| {
                let ks = [0.05, 0.1, 0.2, 0.4, 0.8];
                let runs: Vec<Vec<f64>> = ks
                    .iter()
                    .map(|&k| {
                        window(
                            config(|c| {
                                c.benefit = 0.4;
                                c.victory = Victory::Payoff;
                                c.punish_cost = k;
                            }),
                            "punishment",
                        )
                    })
                    .collect();
                let mut rises = Vec::new();
                for i in 0..ks.len() {
                    for j in i + 1..ks.len() {
                        if greater(&runs[j], &runs[i], "", "").verdict == Verdict::Holds {
                            rises.push(format!("{} → {}", ks[i], ks[j]));
                        }
                    }
                }
                let means: Vec<f64> = runs.iter().map(|r| mean(r)).collect();
                outcome(!rises.is_empty(), format!("{} at k {:?}; rises: {}", show(&means), ks, if rises.is_empty() { "none".into() } else { rises.join(", ") }))
            },
        },
        Claim {
            id: "punishment.bg.start",
            item: "bg-base",
            source: Source::Comment,
            citation: BGBR,
            text: "Ours: the paper's start — one group of punishers among 127 of defectors, 'Various random processes could cause such an initial shift' — does not decide the long run (n 32: from all defectors, equivalent within 0.05)",
            check: |_| {
                let base = long_run(PunishmentConfig::default());
                let from = long_run(config(|c| c.start = Start::AllDefectors));
                equivalent(&from, &base, Some(0.05), "all defectors", "one punisher group")
            },
        },
    ]
}
````

Modify `survey/src/claims/mod.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index 7637250..b4b7d4c 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -16,6 +16,7 @@ mod image;
 mod minds1;
 mod minds2;
 mod opinions;
+mod punishment;
 mod retirement;
 mod spatial;
 mod structure;
@@ -44,6 +45,7 @@ pub fn all() -> Vec<Claim> {
         minds1::claims(),
         minds2::claims(),
         opinions::claims(),
+        punishment::claims(),
         retirement::claims(),
         spatial::claims(),
         structure::claims(),
````

- [ ] **Step 2: Run them**

Run: `cd survey && rustfmt --edition 2021 src/claims/punishment.rs && cargo build --release && ./target/release/survey --only punishment`
Expected (about 3.5 minutes): 12 Holds (fig1a, fig1b-helps, extinction, either — mean gaps 0.010–0.033, fig2, fig3, fig4, calibration-spread — 40 periods, mutation, error, benefit, start), 1 Weak (groups: 0.67 at N 16 against 0.70), 12 Fails (fig1b — mean gap 0.22; caption — caption 2.21, legend 2.51, twice the legend 0.45; either-others — only Fig. 4's fixed cost misses, at 0.051; fig2-reach — 0.43, 0.38; calibration-mixing — 0.58 left; continuous; ring — 0.51 at n 4; hundred — 0.17; baseline; janssen.readings — 1a at 0.056; cooney.fine — a dip under tanh too; cooney.cost — no rise).

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/punishment.rs survey/src/claims/mod.rs
```
```bash
git commit -m "Survey Altruistic Punishment: the figures, their readings, the calibrations, the variants and Cooney

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 5: README, roadmap, papers index, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-28-punishment-design.md`

- [ ] **Step 1: Write the docs**

Modify `README.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/README.md b/README.md
index 09c8315..3c110ed 100644
--- a/README.md
+++ b/README.md
@@ -1719,6 +1719,83 @@ Agent-Based Computational Model of the Timing of Retirement," Brookings CSED Wor
 in H. Aaron, ed., *Behavioral Dimensions of Retirement Economics* (1999); Joshua M. Epstein, *Generative
 Social Science* (Princeton, 2006), chapter 7. See `docs/superpowers/specs/2026-09-27-retirement-design.md`.
 
+### Altruistic Punishment (Boyd, Gintis, Bowles & Richerson 2003)
+
+**The model.** People punish free riders even when it costs them and brings them nothing, and group
+selection was thought to sustain costly cooperation only in small groups. Boyd, Gintis, Bowles and
+Richerson's answer: punishment is cheap once defectors are rare. 128 groups of contributors, defectors
+and punishers play a one-shot game (cooperating costs c = 0.2; punishers fine each defector p/n = 0.8/n at
+a cost of 0.2/n); everyone copies someone who earns more, sometimes from another group; groups fight,
+the one with fewer defectors more likely to win and replace the loser; a few agents mutate. Their
+figures plot cooperation, averaged over the last 1 000 of 2 000 periods, against group size.
+
+**How this reproduction handles the paper's gaps and contradictions.** Every reading is a named
+switch, every figure was read from the PDF at 300 dpi by marker, and every "reproduces the figure"
+claim uses one rule, fixed in advance: a mean gap of at most 0.05 over group sizes 4–256.
+- **The payoff baseline is never stated.** Imitation needs payoffs above 0; a baseline of 1 fits the
+  paper's own calibration (a trait with advantage c spreads from 10 % to 90 % in about 40 periods;
+  "50" stated).
+- **The conflict rate contradicts itself.** Fig. 1's caption gives 0.075, 0.015, 0.003; its legend 0.0075,
+  0.015, 0.03. Under the text's rules neither reproduces the figure: cooperation collapses a group size
+  or two too soon (0.17 at n 128 where the figure has 0.64). No baseline fixes it — a higher one lets
+  punishment reach larger groups but lifts cooperation without punishment far above the figure.
+- **One reading reproduces all of it.** "Groups are paired at random, and with probability ε,
+  intergroup conflict results" can mean either group of a pair starts the conflict, each with
+  probability ε: a pair then fights at about 2ε. Their Methods derive ε = 0.015 from an extinction rate
+  of 0.0075 as if pairs fought at ε; but under the "either" reading, with baseline 1, all six of Fig. 1's
+  curves reproduce (mean gaps 0.010–0.033), and so do Figs. 2 and 3, which that reading was not fitted
+  to — all but Fig. 4's fixed cost, which falls a group size sooner (a mean gap of 0.051). The switch is
+  **Groups meet: in random pairs; either can start it (the figures)**, and the preset `bg-either`.
+- **Janssen's NetLogo replication** (CoMSES 2223) fills the gaps differently — a benefit, every group
+  challenging one, conflict over this period's acts, imitation in turn; his readings are switches here
+  (not his code). Together they come close too (84 % at n 32) because his pairing also doubles conflict.
+
+Measured (the survey and the presets' descriptions; the text's readings unless stated):
+
+- **The figures' shapes hold.** Without punishment, cooperation survives only in groups of 4 or 8
+  (Fig. 1a); punishment sustains more at every size (Fig. 1b); more conflict, more cooperation; more
+  mixing, less (Fig. 2); a fine only twice the cost gives much less (Fig. 3); a fixed punishing cost gives
+  nothing from n 32 (Fig. 4). Lower mutation raises cooperation substantially, more errors lower it, and
+  where the population starts does not matter — all as stated.
+- **The reach does not, under the text's reading:** "cooperation is sustained in groups on the order of
+  100 individuals" — 17 % at n 128. Under the "either" reading, 59 %.
+- **The mixing calibration is off.** m = 0.01 is said to equalize two groups in about 50 periods; after
+  50, 58 % of the difference remains. A member meets the other group with probability m and copies it
+  half the time, so the gap shrinks by about m a period.
+- **Fewer groups add more than noise:** 0.56 at 8 groups against 0.69 at 128.
+- **Continuous traits are not similar.** Cooperation rises with group size (94 % at n 32, 90 % at 256,
+  against the base model's 69 % and 12 %): uniform mutants keep the mean punishment near ½, and a
+  defector then pays about p/2 = 0.4, more than c.
+- **The ring is not cooperation-free**: about half cooperate in groups of 4 or 8, though little from 32.
+- **The per-capita benefit with payoff conflict** is qualitatively similar, as stated.
+- **Cooney's PDE claims (2024):** a shallow dip in payoff at weak punishment appears — but under every
+  victory rule, not only the normalized one his Remark 6.1 blames; a higher cost of punishing never
+  raises the share of punishers here.
+
+Switches: **Groups (N)**, **Group size (n)**, **At the start**, **Cost of cooperating (c)**, **Cost of being
+punished (p)**, **Punishers pay** (k/n per defector, or a fixed cost), **Cost of punishing (k)**, **Fixed
+cost**, **Errors (e)**, **A punisher who errs** (punishes the others, nobody, or itself too), **Benefit to
+others (b)**, **Baseline payoff**, **Mixing (m)**, **Imitation happens** (all at once, or in turn),
+**Mutation (μ)**, **Conflict (ε)**, **Groups meet** (in random pairs; either can start it; each challenges
+one), **Groups fight over** (defectors, payoffs normalized, payoffs through tanh) with **Sensitivity**,
+**Defectors are counted by** (type, or this period's acts), **A defeated group** (becomes a copy of the
+winners, or is refilled with them from the winners), **Traits** (discrete or continuous), **Groups are**
+(anywhere with conflict, or on a ring without), **Long-run window** and **Stop at period**. The view: every
+group a block of its agents, contributors blue, punishers green, defectors red, a group that just lost
+framed; below, cooperation and punishment over time. Color modes: **Type**, **Acts**, **Payoff**,
+**Group**. Charts: Types; Cooperation (with the long-run average); Payoff; Conflict. Presets: `bg-base`,
+`bg-either`, `bg-none`, `bg-large`, `bg-weak`, `bg-fixed`, `bg-mixing`, `bg-benefit`, `bg-continuous`,
+`bg-ring`, `bg-janssen`. **Compare** entry: "With vs without punishment — Altruistic Punishment
+(Compare)". Built-in sweeps: `bg-fig1a`, `bg-fig1b`, `bg-fig1-caption`, `bg-fig1-either`, `bg-fig2a`,
+`bg-fig2b`, `bg-fig3`, `bg-fig4`, `bg-baseline`, `bg-readings`, `bg-mutation`, `bg-error`, `bg-groups`,
+`bg-benefit`, `bg-continuous`, `bg-ring`, `bg-cooney-fine`, `bg-cooney-cost`.
+
+Credit: Robert Boyd, Herbert Gintis, Samuel Bowles and Peter J. Richerson, "The evolution of altruistic
+punishment," *PNAS* 100(6): 3531–3535 (2003); Daniel B. Cooney, "Exploring the Evolution of Altruistic
+Punishment with a PDE Model of Cultural Multilevel Selection," arXiv:2405.18419 (2024; *Bull. Math.
+Biol.* 2025); Marco Janssen's replication, CoMSES Net 2223 (GPL-3.0, read for its readings only). See
+`docs/superpowers/specs/2026-09-28-punishment-design.md`.
+
 ## Experiments
 
 The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
````

Modify `docs/roadmap.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/roadmap.md b/docs/roadmap.md
index 9ce56cd..aa969b0 100644
--- a/docs/roadmap.md
+++ b/docs/roadmap.md
@@ -250,6 +250,19 @@ effect at 5 % rational; footnote 5 is false (counting every friend, no norm form
 not reproduce (the new norm comes in 2 periods); coupling slows the rational group as much as it
 speeds the other. See `docs/superpowers/specs/2026-09-27-retirement-design.md`.
 
+## Milestone 27: Altruistic Punishment (done)
+
+Boyd, Gintis, Bowles and Richerson's altruistic punishment (PNAS 2003) as a model kind: groups of
+contributors, defectors and punishers, payoff-biased imitation with mixing, intergroup conflict and
+mutation, with the paper's structural variants (a per-capita benefit with payoff conflict, continuous
+traits, a ring without extinction), Cooney's PDE critique (2024) and Janssen's NetLogo replication's
+readings as switches. The figures' shapes hold under the text's rules, but not their reach; the
+payoff baseline is unstated and Fig. 1's caption and legend disagree on the conflict rates. One
+reading — either group of a pair can start the conflict, about twice the stated rate — reproduces
+Figs. 1–3 closely (all but Fig. 4's fixed cost). Continuous traits are not "similar"; the mixing
+calibration is off by five; Cooney's payoff dip appears under every victory rule. See
+`docs/superpowers/specs/2026-09-28-punishment-design.md`.
+
 ## Experiments and science
 
 - **Parameter sweeps / batch runs**: done (Milestone 5).
@@ -272,6 +285,7 @@ speeds the other. See `docs/superpowers/specs/2026-09-27-retirement-design.md`.
 - **Kirman's ants and recruitment** (and Alfarano & Milaković's network critique): done (Milestone 24).
 - **Granovetter's threshold models** (and Watts's global cascades): done (Milestone 25).
 - **Axtell and Epstein's timing of retirement**: done (Milestone 26).
+- **Boyd, Gintis, Bowles and Richerson's altruistic punishment** (and Cooney's PDE critique): done (Milestone 27).
 - **Minds 1: the utility mind and the ideal free distribution** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Minds 2: A\* and walking; which of the book's results need the jump** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Credit hierarchy view**: done (Milestone 6).
````

Modify `docs/papers.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/papers.md b/docs/papers.md
index 07f64ce..05addc7 100644
--- a/docs/papers.md
+++ b/docs/papers.md
@@ -34,6 +34,7 @@ read online or from another copy; add it when found. Scanned PDFs (no text layer
 | 24 | `ants` | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf`; the critique `ants/alfarano-milakovic-2007-warwick-wp-should-network-structure-matter.pdf` (published as JEDC 33(1), 2009) | the chain is exactly beta-binomial but never rests at the ants' 80–20 (Becker's pull does); Fig. IIb's time average needs 100× the figure; herding fades with N, cured by a random network only under AM's rule; AM's mean field fails on rings |
 | 25 | `thresholds` | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*, read by OCR); the follow-up `thresholds/watts-2002-pnas-simple-model-of-global-cascades-on-random-networks.pdf` (from the Internet Archive's copy of PNAS) | the crowds and Fig. 2's continuous jump reproduce, but a crowd of people tips at a σ set by rounding and sampled crowds do not jump; the city's riot of 100 comes 2 % of the time; Watts's upper edge depends on n, his Fig. 4b cannot be built, hubs help in both regimes |
 | 26 | `retirement` | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf`; the revised text `demographic-pd/epstein-2006-generative-social-science.pdf` (ch. 7) | the realizations (a little slower) and network-size effects reproduce, extent only at 10 % rational; footnote 5 is false; Fig. 6-6's minimum rationality needs an unstated rule (friends replaced); the 65 → 62 switch takes 2 periods, not 20–35; coupling slows the rational group |
+| 27 | `punishment` | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf`; the critique `punishment/cooney-2024-arxiv-altruistic-punishment-pde-multilevel-selection.pdf` (published in *Bull. Math. Biol.* 2025); Janssen's NetLogo replication `punishment/janssen-comses-2223-netlogo/` (GPL-3.0: readings only) | the shapes hold but not the reach; the baseline is unstated and the caption contradicts the legend; either group starting a conflict (twice the stated rate) reproduces Figs. 1–3; continuous traits are not similar; Cooney's dip appears under every victory rule |
 
 ## Queue
 
@@ -42,11 +43,10 @@ worth doing; "size" is a guess at the milestone's scale.
 
 | # | Model | Original | Critique or follow-up | Size | Shape |
 |---|---|---|---|---|---|
-| 1 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
-| 2 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
-| 3 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
-| 4 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
-| 5 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
+| 1 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
+| 2 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
+| 3 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
+| 4 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
 
 ## Wanted
````

Modify `docs/superpowers/specs/2026-09-28-punishment-design.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/superpowers/specs/2026-09-28-punishment-design.md b/docs/superpowers/specs/2026-09-28-punishment-design.md
index 4d2f32d..50e3f9f 100644
--- a/docs/superpowers/specs/2026-09-28-punishment-design.md
+++ b/docs/superpowers/specs/2026-09-28-punishment-design.md
@@ -166,3 +166,15 @@ The presets menu gains an **Altruistic Punishment** group and the Compare entry;
 ## Docs
 
 README: an Altruistic Punishment section (the model, the stated choices, switches, presets, sweeps, and the findings). `docs/papers.md`: the milestone's row (with Cooney as the critique and Janssen's code as reference); the Queue's first entry removed; roadmap: Milestone 27 done.
+
+## Amendments (implementation planning)
+
+Found while implementing and measuring, in the plan `docs/superpowers/plans/2026-09-28-punishment.md`:
+
+- **A fourth pairing, `either`**, the reading the figures fit: groups are paired at random and either group of a pair can start the conflict, each with probability ε (a pair fights with probability 2ε − ε²). Measured with the implementation, it reproduces all six of Fig. 1's curves (mean gaps 0.010–0.033) and Figs. 2 and 3 (0.006–0.022), which it was not found from; Fig. 4's fixed cost misses at 0.051. The preset `bg-either` and the sweep `bg-fig1-either` show it. Every "reproduces the figure" claim uses one rule, fixed before measuring: a mean gap of at most 0.05 over n 4–256.
+- **The figures' values** were read from the PDF at 300 dpi by marker centers (a marker hidden under another curve takes that curve's value).
+- **Presets:** eleven, `bg-either` added; the titles as measured (`bg-base` "Punishers keep about 70 % of groups of 32 cooperating"; `bg-fixed`, `bg-continuous` and `bg-ring` say what happens).
+- **Sweeps:** eighteen. `bg-sensitivity` is `bg-mutation`, `bg-error` and `bg-groups`; `bg-readings` compares the text, `challenge`, `in_turn` and Janssen's readings with and without punishment; `bg-continuous` adds a run without conflict; `bg-cooney-fine` and `bg-cooney-cost` add the `defectors` rule as a third series and read window means of `payoff` and `punishment`.
+- **The Conflict chart** shows `conflicts` and `spread`: every conflict replaces a group, so `extinctions` always equals `conflicts` (it stays a series).
+- **The mosaic:** ⌈√(2N)⌉ groups a row; cells of 480/(groups a row × ⌈√n⌉) pixels, from 1 to 8.
+- **Measured with the implementation** (the survey, 25 claims: 12 hold, 1 weak, 12 fail): Figs. 1a, 2, 3 and 4's shapes, punishment helping at n 32, conflict raising cooperation, the spread calibration (40 periods), mutation, errors, the benefit variant and the start all hold; Fig. 1b's reach, Fig. 2b's reach, "groups on the order of 100" (0.17 at n 128), the caption's rates, the mixing calibration (0.58 of the difference left after 50 periods), continuous traits (0.94 against 0.69 at n 32), the ring (0.51 at n 4), every baseline, Janssen's readings (1a's mean gap 0.056), the Fig. 2–4 generalization of `either` (Fig. 4 at 0.051) and both of Cooney's claims (the dip appears under tanh too; a higher k never raises punishment) fail; fewer groups is weak (a lower mean at 8, not only noise).
````

- [ ] **Step 2: Verify everything**

Run: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm && (cd web && npm run build && npm test) && (cd survey && cargo build --release && ./target/release/survey --only punishment)`
Expected: all green (1 040 Rust tests, 51 WASM, 728 web); the survey's verdicts as in Task 4.

- [ ] **Step 3 (controller): the full browser pass** — Task 3's Step 6 list again, on the final build.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-09-28-punishment-design.md
```
```bash
git commit -m "Document Altruistic Punishment, how its gaps and contradictions were read, and what reproduces; mark milestone 27 done

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

## Self-review (planning)

- **Spec coverage:** Architecture, Config, Step, Statistics, Views, Presets, Compare, Experiments and CLI → Tasks 1–3; Survey → Task 4; Docs → Task 5; departures in the spec's Amendments.
- **Placeholders:** none; every file is given in full or as a diff against `4450d5d`.
- **Types:** `PunishmentInspection`, `PunisherView` and `PunishmentGroupView` (types.ts) match `PunishmentInspection`, `AgentView` and `GroupView` (world.rs); `isPunishmentView` tests `punishment`, which no other inspection has (norms' is `punishments`).
