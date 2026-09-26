# Social Structure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Cohen, Riolo and Axelrod's adaptive agents playing short iterated Prisoner's Dilemmas under six social structures as a new model kind, `structure` ("Social Structure"), with Table 2's structures as nine presets, the substitution dial live, the paper's two readings of its own method as switches, five measured sweeps and a 29-claim survey, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/structure/` — `config.rs` (parameters, validation, schema), `graph.rs` (the torus, FRN and FRNE drawn at reset, and Table A1's fan-out), `stats.rs` (the snapshot, payoffs, the regression), `view.rs` (the block-and-plane frame's geometry and colors), `world.rs` (`StructureWorld`: partners, games, adaptation, Inspect), `presets.rs`, `mod.rs` — wired into `ModelConfig`/`ModelWorld` like the other models. The page adds the model's types, color modes, charts, Inspect, a Compare entry and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies (the normal draws reuse `crate::anasazi::random::normal`, portable across native and WASM).

**Spec:** `docs/superpowers/specs/2026-09-26-social-structure-design.md` (binding, as amended in Task 5). Source: Cohen, Riolo & Axelrod, "The Role of Social Structure in the Maintenance of Cooperative Regimes", *Rationality and Society* 13(1) (2001), 5–32.

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited (`MODEL_GOLDEN` gains nine `cra-*` entries).
- **One engine path; deterministic; portable:** native and WASM fingerprints identical (verified in planning by `wasm-pack test` and the web determinism test).
- **Literal defaults, named departures, honest descriptions.**
- **Copy (verbatim):** model label **Social Structure**; preset ids `cra-rwr`, `cra-2dk`, `cra-frne`, `cra-frn`, `cra-ffr-01`, `cra-ffr-03`, `cra-ffr-05`, `cra-random-start`, `cra-copy-noise`; Compare entry **Random mixing vs fixed random neighbors — Social Structure (Compare)** (id `cra-rwr-vs-frn`); color modes **Friendliness**, **Provocability**, **Payoff**, **Strategy**; schema groups **Population**, **Structure**, **Adaptation**, **Measures**; charts **Mean payoff**, **Cooperation**, **Strategy**, **High cooperation**, **Copying**; time axis **Periods**; sweeps `cra-table-2`, `cra-dial`, `cra-threshold`, `cra-noise`, `cra-population`; series `mean_payoff, cooperation, mean_y, mean_p, mean_q, high, attained_high, share_high_since, copied, partner_p_slope`; notice `This run has reached its last period (2500) — Reset to run it again`; CLI `(its last period)`.
- Every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE`. Stage only the task's files; never `.claude/`.
- Rust: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. In `survey/`, format only `survey/src/claims/structure.rs` (`rustfmt --edition 2021`); its `ch6.rs` clippy warnings are not ours.
- Web: `(cd web && npm run build && npm test)`.
- **Browser checks are the controller's** (Task 3's list; the full pass in Task 5).

## Review Focus

1. **Who counts as "played"** — both roles, repeated partners (FRN draws with replacement; the torus and FRNE play each pair twice), and an agent nobody chose that chose nobody: scores must be per move over every game, the best partner found among distinct agents met, and an agent with no games kept (mutated under `always`). Pinned in Task 1 by `a_period_plays_every_chosen_game_in_both_roles`, `agents_copy_only_a_strictly_better_partner_unless_they_misjudge`.
2. **FRNE's construction** — simple, symmetric, regular, and actually mixed away from its starting ring; odd `partners` or too few agents rejected. Pinned in Task 1 by `frne_is_regular_simple_and_symmetric`, `validation_names_fields`, `fanout_matches_the_papers_table_a1_shape`.
3. **Live edits and keyframes** — substitution, errors, noise, the threshold and the stop apply live; agents, structure, partners and start need Reset; stepping back restores strategies, the fixed network and the plane's trail. Pinned in Task 1 by `live_edits_apply_and_the_population_waits_for_reset`, `keyframes_restore_strategies_and_the_trail`, `schema_paths_exist_and_match_what_set_config_allows`.
4. **The threshold's bookkeeping** — `attained_high` and `share_high_since` when the threshold is edited mid-run, and period 0 (no games) never counting as high. Pinned in Task 1 by `high_cooperation_is_attained_and_remembered`.
5. **Inspect across the frame** — a block cell (torus site vs index order), a plane point, the gap, non-square populations' empty cells, off the frame; Follow's `locate`. Pinned in Task 1 by `the_view_draws_block_and_plane_and_inspect_finds_both`, `degenerate_configs_run_without_panicking` (250 agents); in Task 3 by `stops at its last period and inspects an agent and its partners`.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --workspace`, `cargo clippy --all-targets -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (39), `npm run build && npm test` (574) and the survey (29 claims in under a minute).

1. **The threshold is 2.3 (fixes the spec's open default):** at 2.3 every row of Table 2's "Remain High" lands within 0.026 of the paper (FRN 0.940 vs 0.942, FFR-0.1 0.843 vs 0.844); at 2.2 the largest miss is 0.14, at 2.4 0.26. The paper does not state it; this is the survey's `structure.table-2.threshold`.
2. **FRNE (amends the spec):** a ring lattice (each agent linked to the `partners`/2 nearest on each side) mixed by CRA's own procedure — each agent, n times, swaps a neighbor with a random other agent's neighbor (a double-edge swap), kept when it creates no self-link or repeat. No retries are needed; `partners` must be even. Table A1's fan-out comes out at 4.00, 11.74, 32.1, 72.9, 97.7, 35.7 against the paper's 4.00, 11.74, 32.38, 74.59, 98.42, 33.23.
3. **Charts (amends the spec):** chart reference lines are series, so the threshold is not drawn on Mean payoff; the High cooperation chart shows `high` and `share_high_since`.
4. **The CLI names the stop** `(its last period)` explicitly (the existing default says "its end year").
5. **Payoff color** scales a score of 0–3 to red–green (mutual cooperation's 3 is the practical ceiling).
6. **The web golden list** gains all nine presets (each stops at 2500, after the list's 200 ticks); an engine test runs `cra-2dk` with `stop_at` 30 to its stop and inspects an agent.
7. **Planning's findings** (30 seeds × 2500 periods, as the paper; the survey reproduces them): Table 2's mean payoffs within 0.07 in every row (RWR 1.089, 2DK 2.553, FRNE 2.574, FRN 2.478, FFR-0.1 2.405, FFR-0.3 2.036, FFR-0.5 1.325 against 1.091, 2.557, 2.575, 2.480, 2.385, 2.100, 1.257) and its Remain High within 0.03 at 2.3; Attain High C 0.43 for RWR against 0.30 (within sampling); FFR-0.3 bi-stable in 25 of 30 runs; Fig. 1's 2.25 first period and the fixed structures' recovery; the crucial region's Δp −0.012 (RWR) and +0.051 (FRN) against −0.016 and +0.052; the partner-p slope 0.179 (F 1087) for FRN against 0.158 (F 717), RWR not significant (F 2.1); note 5's FRNE > 2DK (p = 5e-11); note 1 at 4096 agents; Table A1. **The paper's two starts are equivalent (2.473 vs 2.478), but its two noise rules are not: noise only on copying gives FRN 2.530 — the Appendix's rule (2.478) is the one Table 2 reflects.**

---

### Task 1: The social-structure model in the core

**Files:**
- Create: `crates/sugarscape-core/src/structure/{config,graph,stats,view,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `model.rs`, `presets.rs`, `tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{lerp, Rgb, BACKGROUND}`, `crate::rng`, `crate::anasazi::random::normal`, `crate::schema::{Apply, Param}`, `crate::presets::ModelPreset`.
- Produces: `structure::{StructureConfig, Structure, NoiseOn, Start, schema, square_side, presets, SERIES, StructureSnapshot, payoff, regression, Graph, fanout, block_side, cell, class, frame, plane_cell, plane_x, PLANE, TRAIL, StructureWorld, Strategy, StructureMode, StructureInspection, StructureCell, AgentView, PartnerView}`; `StructureWorld::{strategies, graph, partner_p_pairs, step, run}`; `ModelKind::Structure` (`"structure"`), `ModelConfig::Structure`, `ModelWorld::Structure`.

- [ ] **Step 1: Write the module**

Create `crates/sugarscape-core/src/structure/config.rs` with exactly this content:

```rust
//! The Social Structure model's parameters: Cohen, Riolo and Axelrod's
//! (2001) population, structures and adaptation, with the points the paper
//! leaves unstated or states twice as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// Who plays whom (CRA's Appendix).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Structure {
    /// Random With Replacement: fresh random partners every period.
    Rwr,
    /// 2DK: a torus, each agent playing its four NEWS neighbors.
    Torus,
    /// Fixed Random Neighbors, Equal: a fixed random regular symmetric graph.
    Frne,
    /// Fixed Random Neighbors: partners drawn once, with replacement, one-way.
    Frn,
}

/// Who gets the copying noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseOn {
    /// Every agent every period, "regardless of which of the two strategies
    /// … is adopted" (the Appendix).
    Always,
    /// Only an agent that copied ("errors in the actual copying process", §2).
    Copy,
}

/// How strategies start (y = p either way).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// "Evenly distributing the agents throughout the strategy space" (the
    /// Appendix): p and q on an evenly spaced grid.
    Grid,
    /// "Strategies that were initialized randomly" (§3.1): p, q uniform.
    Random,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StructureConfig {
    /// n; a perfect square on the torus.
    pub agents: u32,
    pub structure: Structure,
    /// FFR-x: each fixed link replaced for the period by a random partner
    /// with this probability (ignored under RWR).
    pub substitution: f64,
    /// Partners each agent chooses (4 on the torus).
    pub partners: u32,
    /// Moves per game.
    pub moves: u32,
    /// The chance an agent misjudges whether its best partner did better.
    pub judge_error: f64,
    /// The chance per strategy parameter of Gaussian noise, and its s.d.
    pub mutation: f64,
    pub mutation_sd: f64,
    pub noise_on: NoiseOn,
    pub start: Start,
    /// "High cooperation": a mean payoff per move of at least this (the
    /// paper does not state its threshold).
    pub high: f64,
    /// Stop at this period (0: never).
    pub stop_at: u32,
}

impl Default for StructureConfig {
    /// CRA's Table 2, row 1: 256 agents, random partners each period.
    fn default() -> Self {
        StructureConfig {
            agents: 256,
            structure: Structure::Rwr,
            substitution: 0.0,
            partners: 4,
            moves: 4,
            judge_error: 0.1,
            mutation: 0.1,
            mutation_sd: 0.4,
            noise_on: NoiseOn::Always,
            start: Start::Grid,
            high: 2.3,
            stop_at: 0,
        }
    }
}

/// √n when n is a perfect square.
pub fn square_side(n: u32) -> Option<u32> {
    let s = (f64::from(n)).sqrt().round() as u32;
    (s * s == n).then_some(s)
}

impl StructureConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (4..=4096).contains(&self.agents),
            "agents",
            "must be between 4 and 4096",
        );
        check(
            (1..=16).contains(&self.partners) && self.partners < self.agents,
            "partners",
            "must be between 1 and 16, and fewer than the agents",
        );
        check(
            (1..=100).contains(&self.moves),
            "moves",
            "must be between 1 and 100",
        );
        check(
            unit(self.substitution),
            "substitution",
            "must be between 0 and 1",
        );
        check(
            unit(self.judge_error),
            "judge_error",
            "must be between 0 and 1",
        );
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        check(
            (0.0..=2.0).contains(&self.mutation_sd),
            "mutation_sd",
            "must be between 0 and 2",
        );
        check(
            (0.0..=5.0).contains(&self.high),
            "high",
            "must be between 0 and 5",
        );
        match self.structure {
            Structure::Torus => {
                check(
                    square_side(self.agents).is_some_and(|s| s >= 3),
                    "agents",
                    "must be a square of at least 3 × 3 on the torus",
                );
                check(self.partners == 4, "partners", "must be 4 on the torus");
            }
            Structure::Frne => check(
                self.partners.is_multiple_of(2) && self.partners + 1 < self.agents,
                "partners",
                "must be even and at most the agents − 2 for FRNE",
            ),
            Structure::Rwr | Structure::Frn => {}
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &StructureConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("structure", self.structure == next.structure),
            ("partners", self.partners == next.partners),
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
        Param::integer("Population", "agents", "Agents (n)", (4, 4096), Reset)
            .with_help("CRA: 256 (note 1: up to 4096 'very similar'). A square on the torus."),
        Param::choice(
            "Population",
            "start",
            "Strategies start",
            &[
                ("grid", "Evenly spread (Appendix)"),
                ("random", "At random (§3.1)"),
            ],
            Reset,
        )
        .with_help("The paper says both: 'evenly distributing … throughout the strategy space' and 'initialized randomly'."),
        Param::choice(
            "Structure",
            "structure",
            "Who plays whom",
            &[
                ("rwr", "Random each period (RWR)"),
                ("torus", "Torus neighbors (2DK)"),
                ("frne", "Fixed random, symmetric (FRNE)"),
                ("frn", "Fixed random, one-way (FRN)"),
            ],
            Reset,
        ),
        Param::number(
            "Structure",
            "substitution",
            "Random substitution (FFR)",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("CRA: with FRN, each fixed partner is replaced for the period with this probability; 0.3 is where 'the dynamics shift'. Ignored under RWR."),
        Param::integer("Structure", "partners", "Partners chosen", (1, 16), Reset)
            .with_help("CRA: 4. The torus always has 4; FRNE needs an even number."),
        Param::integer("Adaptation", "moves", "Moves per game", (1, 100), Live)
            .with_help("CRA: 4, 'short enough to make cooperation difficult … but still possible'."),
        Param::number(
            "Adaptation",
            "judge_error",
            "Misjudging the best",
            (0.0, 1.0, 0.01),
            Live,
        ),
        Param::number("Adaptation", "mutation", "Noise chance", (0.0, 1.0, 0.01), Live),
        Param::number("Adaptation", "mutation_sd", "Noise s.d.", (0.0, 2.0, 0.05), Live),
        Param::choice(
            "Adaptation",
            "noise_on",
            "Noise on",
            &[
                ("always", "Every agent (Appendix)"),
                ("copy", "Only on copying (§2)"),
            ],
            Live,
        )
        .with_help("The Appendix adds noise 'regardless of which … is adopted'; §2 describes 'errors in the actual copying process'."),
        Param::number(
            "Measures",
            "high",
            "High cooperation at",
            (0.0, 5.0, 0.05),
            Live,
        )
        .with_help("Mean payoff per move. CRA do not state their threshold; see the cra-threshold sweep."),
        Param::integer("Measures", "stop_at", "Stop at period", (0, 1_000_000), Live)
            .with_help("CRA ran 2500 periods. 0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_table_2_row_1() {
        let c = StructureConfig::default();
        assert_eq!(
            (c.agents, c.structure, c.partners, c.moves),
            (256, Structure::Rwr, 4, 4)
        );
        assert_eq!((c.judge_error, c.mutation, c.mutation_sd), (0.1, 0.1, 0.4));
        assert_eq!((c.noise_on, c.start), (NoiseOn::Always, Start::Grid));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = StructureConfig {
            agents: 2,
            moves: 0,
            substitution: 1.5,
            judge_error: -0.1,
            mutation: 2.0,
            mutation_sd: 3.0,
            high: 6.0,
            ..StructureConfig::default()
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
                "agents",
                "partners",
                "moves",
                "substitution",
                "judge_error",
                "mutation",
                "mutation_sd",
                "high"
            ]
        );
        let torus = StructureConfig {
            agents: 250,
            structure: Structure::Torus,
            ..StructureConfig::default()
        };
        assert_eq!(torus.validate().unwrap_err()[0].field, "agents");
        let frne = StructureConfig {
            partners: 3,
            structure: Structure::Frne,
            ..StructureConfig::default()
        };
        assert_eq!(frne.validate().unwrap_err()[0].field, "partners");
        assert_eq!((square_side(256), square_side(250)), (Some(16), None));
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Structure(StructureConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

Create `crates/sugarscape-core/src/structure/graph.rs` with exactly this content:

```rust
//! The fixed social structures, drawn at reset: the torus (2DK), fixed
//! random neighbors (FRN) and the symmetric random regular graph (FRNE),
//! and the fan-out CRA measure in Table A1.

use std::collections::VecDeque;

use rand::seq::SliceRandom;
use rand::Rng;

use super::config::{square_side, Structure, StructureConfig};
use crate::rng::SimRng;

/// Each agent's fixed chosen partners (empty under RWR), and on the torus
/// where each agent sits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub chosen: Vec<Vec<u32>>,
    /// Torus only: `site[agent]` and `agent_at[site]`, row-major.
    pub site: Vec<u32>,
    pub agent_at: Vec<u32>,
}

impl Graph {
    pub fn new(c: &StructureConfig, rng: &mut SimRng) -> Self {
        let n = c.agents as usize;
        match c.structure {
            Structure::Rwr => Graph::default(),
            Structure::Frn => Graph {
                chosen: (0..n)
                    .map(|a| (0..c.partners).map(|_| other(rng, n, a)).collect())
                    .collect(),
                ..Graph::default()
            },
            Structure::Torus => torus(n, rng),
            Structure::Frne => Graph {
                chosen: regular(n, c.partners as usize, rng),
                ..Graph::default()
            },
        }
    }
}

/// A uniform agent other than `a`.
pub fn other(rng: &mut SimRng, n: usize, a: usize) -> u32 {
    let b = rng.gen_range(0..n as u32 - 1) as usize;
    (if b >= a { b + 1 } else { b }) as u32
}

/// Agents at random on the √n × √n torus; each chooses its NEWS neighbors
/// (north, east, west, south), so every pair plays twice.
fn torus(n: usize, rng: &mut SimRng) -> Graph {
    let s = square_side(n as u32).expect("validated: a square") as i32;
    let mut agent_at: Vec<u32> = (0..n as u32).collect();
    agent_at.shuffle(rng);
    let mut site = vec![0; n];
    for (k, &a) in agent_at.iter().enumerate() {
        site[a as usize] = k as u32;
    }
    let chosen = (0..n)
        .map(|a| {
            let k = site[a] as i32;
            let (x, y) = (k % s, k / s);
            [(0, -1), (1, 0), (-1, 0), (0, 1)]
                .iter()
                .map(|&(dx, dy)| {
                    agent_at[((y + dy).rem_euclid(s) * s + (x + dx).rem_euclid(s)) as usize]
                })
                .collect()
        })
        .collect();
    Graph {
        chosen,
        site,
        agent_at,
    }
}

/// A random `k`-regular simple symmetric graph (k even): a ring lattice
/// (each agent linked to the k/2 nearest on each side) mixed by CRA's
/// procedure — each agent, n times, swaps one of its neighbors with a random
/// other agent's neighbor (a double-edge swap), kept only if no agent gains
/// itself or a repeated neighbor.
fn regular(n: usize, k: usize, rng: &mut SimRng) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = (0..n)
        .map(|a| {
            (1..=k / 2)
                .flat_map(|d| [(a + d) % n, (a + n - d) % n])
                .map(|b| b as u32)
                .collect()
        })
        .collect();
    for a in 0..n {
        for _ in 0..n {
            let c = other(rng, n, a) as usize;
            let bi = rng.gen_range(0..k as u32) as usize;
            let di = rng.gen_range(0..k as u32) as usize;
            let (b, d) = (adj[a][bi] as usize, adj[c][di] as usize);
            // a–b, c–d → a–d, c–b
            if d == a || b == c || b == d {
                continue;
            }
            if adj[a].contains(&(d as u32)) || adj[c].contains(&(b as u32)) {
                continue;
            }
            let replace = |adj: &mut Vec<Vec<u32>>, x: usize, from: usize, to: usize| {
                let i = adj[x].iter().position(|&v| v as usize == from).unwrap();
                adj[x][i] = to as u32;
            };
            replace(&mut adj, a, b, d);
            replace(&mut adj, b, a, c);
            replace(&mut adj, c, d, b);
            replace(&mut adj, d, c, a);
        }
    }
    adj
}

/// The number of agents exactly d links away (d = 1, 2, …), averaged over
/// all agents, treating links as undirected (Table A1's distribution
/// sequence).
pub fn fanout(chosen: &[Vec<u32>], max_d: usize) -> Vec<f64> {
    let n = chosen.len();
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (a, list) in chosen.iter().enumerate() {
        for &b in list {
            for (x, y) in [(a, b as usize), (b as usize, a)] {
                if !adj[x].contains(&(y as u32)) {
                    adj[x].push(y as u32);
                }
            }
        }
    }
    let mut totals = vec![0usize; max_d];
    for s in 0..n {
        let mut dist = vec![usize::MAX; n];
        dist[s] = 0;
        let mut queue = VecDeque::from([s]);
        while let Some(u) = queue.pop_front() {
            if dist[u] >= max_d {
                continue;
            }
            for &v in &adj[u] {
                let v = v as usize;
                if dist[v] == usize::MAX {
                    dist[v] = dist[u] + 1;
                    totals[dist[v] - 1] += 1;
                    queue.push_back(v);
                }
            }
        }
    }
    totals.iter().map(|&t| t as f64 / n as f64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    fn config(structure: Structure) -> StructureConfig {
        StructureConfig {
            structure,
            ..StructureConfig::default()
        }
    }

    #[test]
    fn the_torus_wraps_and_neighbors_are_mutual() {
        let g = Graph::new(&config(Structure::Torus), &mut rng::seeded(1));
        let at = |x: i32, y: i32| g.agent_at[(y.rem_euclid(16) * 16 + x.rem_euclid(16)) as usize];
        let corner = at(0, 0) as usize;
        assert_eq!(g.chosen[corner], [at(0, -1), at(1, 0), at(-1, 0), at(0, 1)]);
        for (a, list) in g.chosen.iter().enumerate() {
            for &b in list {
                assert!(g.chosen[b as usize].contains(&(a as u32)));
            }
        }
    }

    #[test]
    fn frn_draws_others_with_replacement() {
        let g = Graph::new(&config(Structure::Frn), &mut rng::seeded(2));
        assert!(g.chosen.iter().all(|l| l.len() == 4));
        assert!(g
            .chosen
            .iter()
            .enumerate()
            .all(|(a, l)| !l.contains(&(a as u32))));
        let repeats = g
            .chosen
            .iter()
            .filter(|l| (1..4).any(|i| l[..i].contains(&l[i])))
            .count();
        assert!(
            repeats > 0,
            "with replacement, some agent picks a partner twice"
        );
    }

    #[test]
    fn frne_is_regular_simple_and_symmetric() {
        let g = Graph::new(&config(Structure::Frne), &mut rng::seeded(3));
        for (a, list) in g.chosen.iter().enumerate() {
            assert_eq!(list.len(), 4);
            assert!(!list.contains(&(a as u32)));
            for (i, &b) in list.iter().enumerate() {
                assert!(!list[..i].contains(&b), "no repeated neighbor");
                assert!(g.chosen[b as usize].contains(&(a as u32)), "symmetric");
            }
        }
        // Mixed: few links are left from the ring lattice.
        let ring = g
            .chosen
            .iter()
            .enumerate()
            .flat_map(|(a, l)| l.iter().map(move |&b| (a, b as usize)))
            .filter(|&(a, b)| (a + 256 - b) % 256 <= 2 || (b + 256 - a) % 256 <= 2)
            .count();
        assert!(ring < 20, "{ring} ring links survive");
    }

    #[test]
    fn fanout_matches_the_papers_table_a1_shape() {
        let torus = Graph::new(&config(Structure::Torus), &mut rng::seeded(4));
        assert_eq!(
            fanout(&torus.chosen, 3),
            [4.0, 8.0, 12.0],
            "N(d) = 4d on the torus"
        );
        let frne = Graph::new(&config(Structure::Frne), &mut rng::seeded(4));
        let f = fanout(&frne.chosen, 6);
        assert_eq!(f[0], 4.0);
        assert!(
            (f[1] - 11.74).abs() < 0.5 && (f[2] - 32.4).abs() < 2.0,
            "{f:?}"
        );
    }
}
```

Create `crates/sugarscape-core/src/structure/stats.rs` with exactly this content:

```rust
//! The Social Structure model's statistics, and the pieces of the game and
//! regression they rest on.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 10] = [
    "mean_payoff",
    "cooperation",
    "mean_y",
    "mean_p",
    "mean_q",
    "high",
    "attained_high",
    "share_high_since",
    "copied",
    "partner_p_slope",
];

/// One period's statistics (period 0: the starting population, before any
/// game).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct StructureSnapshot {
    pub tick: u64,
    /// Payoff per move over every game this period.
    pub mean_payoff: f64,
    /// The share of moves that cooperated.
    pub cooperation: f64,
    /// Population means after this period's adaptation.
    pub mean_y: f64,
    pub mean_p: f64,
    pub mean_q: f64,
    /// 1 when `mean_payoff` reached the `high` threshold.
    pub high: u8,
    /// The first high period, else −1.
    pub attained_high: i64,
    /// The share of periods from `attained_high` on that were high (0 before).
    pub share_high_since: f64,
    /// The share of agents that copied a partner this period.
    pub copied: f64,
    /// The least-squares slope of partners' mean p on the agent's own p, as
    /// played this period (CRA's Figs. 5–6).
    pub partner_p_slope: f64,
}

impl Series for StructureSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "mean_payoff" => self.mean_payoff,
            "cooperation" => self.cooperation,
            "mean_y" => self.mean_y,
            "mean_p" => self.mean_p,
            "mean_q" => self.mean_q,
            "high" => f64::from(self.high),
            "attained_high" => self.attained_high as f64,
            "share_high_since" => self.share_high_since,
            "copied" => self.copied,
            "partner_p_slope" => self.partner_p_slope,
            _ => return None,
        })
    }
}

/// The Prisoner's Dilemma payoff to a player (Table 1: R 3, S 0, T 5, P 1).
pub fn payoff(me: bool, them: bool) -> u32 {
    match (me, them) {
        (true, true) => 3,
        (true, false) => 0,
        (false, true) => 5,
        (false, false) => 1,
    }
}

/// The least-squares slope of `y` on `x` (0 when `x` does not vary), and
/// the regression's F statistic (0 with fewer than three points).
pub fn regression(points: &[(f64, f64)]) -> (f64, f64) {
    let n = points.len() as f64;
    if points.len() < 3 {
        return (0.0, 0.0);
    }
    let (mx, my) = points
        .iter()
        .fold((0.0, 0.0), |(a, b), &(x, y)| (a + x / n, b + y / n));
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for &(x, y) in points {
        sxx += (x - mx) * (x - mx);
        sxy += (x - mx) * (y - my);
        syy += (y - my) * (y - my);
    }
    if sxx == 0.0 {
        return (0.0, 0.0);
    }
    let slope = sxy / sxx;
    let explained = slope * sxy;
    let residual = (syy - explained).max(0.0);
    let f = if residual == 0.0 {
        f64::INFINITY
    } else {
        explained / (residual / (n - 2.0))
    };
    (slope, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payoffs_are_table_1() {
        assert_eq!(
            [
                payoff(true, true),
                payoff(true, false),
                payoff(false, true),
                payoff(false, false)
            ],
            [3, 0, 5, 1]
        );
    }

    #[test]
    fn the_regression_recovers_a_line() {
        let pts: Vec<(f64, f64)> = (0..10).map(|i| (i as f64, 2.0 * i as f64 + 1.0)).collect();
        let (slope, f) = regression(&pts);
        assert!((slope - 2.0).abs() < 1e-12 && f.is_infinite());
        assert_eq!(
            regression(&[(1.0, 2.0), (1.0, 3.0), (1.0, 4.0)]),
            (0.0, 0.0)
        );
        let noisy = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 3.0)];
        let (s, f) = regression(&noisy);
        assert!((s - 0.8).abs() < 1e-12 && f > 0.0, "{s} {f}");
    }
}
```

Create `crates/sugarscape-core/src/structure/view.rs` with exactly this content:

```rust
//! The Social Structure frame: the agents as a block of cells on the left
//! (the torus itself under 2DK, index order otherwise) and CRA's p–q plane
//! on the right (Figs. 2–4), with the population's recent trail.

use crate::render::{lerp, Rgb, BACKGROUND};

/// The p–q plane's side in cells (p = 0 … 1 in steps of 0.01).
pub const PLANE: usize = 101;
/// Cells between the block and the plane.
pub const GAP: usize = 6;
/// Periods of the population's mean (p, q) the plane keeps.
pub const TRAIL: usize = 200;

pub const LOW: Rgb = [0xd9, 0x48, 0x3b];
pub const HIGH: Rgb = [0x3d, 0xd6, 0x6b];
pub const TFT: Rgb = [0x3d, 0xd6, 0x6b];
pub const ALLD: Rgb = [0xd9, 0x48, 0x3b];
pub const ALLC: Rgb = [0x4f, 0x9d, 0xff];
pub const OTHER: Rgb = [0x8a, 0x86, 0x7a];
pub const PLANE_BG: Rgb = [0x22, 0x21, 0x1d];
pub const TRAIL_OLD: Rgb = [0x5a, 0x4a, 0x1a];
pub const TRAIL_NEW: Rgb = [0xff, 0xd8, 0x4d];
pub const DOT: Rgb = [0xb8, 0xb4, 0xa8];
pub const DOT_MANY: Rgb = [0xff, 0xff, 0xff];

/// Cells a side for n agents (⌈√n⌉).
pub fn block_side(n: usize) -> usize {
    let mut s = (n as f64).sqrt() as usize;
    while s * s < n {
        s += 1;
    }
    s
}

/// Cells per agent, so the block is about as tall as the plane.
pub fn cell(n: usize) -> usize {
    (PLANE / block_side(n)).max(1)
}

/// The frame: the block, a gap, the plane.
pub fn frame(n: usize) -> (usize, usize) {
    let block = block_side(n) * cell(n);
    (block + GAP + PLANE, block.max(PLANE))
}

/// Where the plane starts.
pub fn plane_x(n: usize) -> usize {
    block_side(n) * cell(n) + GAP
}

/// The plane cell of (p, q): p to the right, q up.
pub fn plane_cell(p: f64, q: f64) -> (usize, usize) {
    let at = |v: f64| (v.clamp(0.0, 1.0) * (PLANE - 1) as f64).round() as usize;
    (at(p), PLANE - 1 - at(q))
}

/// A strategy's class: the nearest of TFT (1,1,0), ALLD (0,0,0), ALLC
/// (1,1,1) if within 0.5, else other.
pub fn class(y: f64, p: f64, q: f64) -> &'static str {
    let d = |a: [f64; 3]| ((y - a[0]).powi(2) + (p - a[1]).powi(2) + (q - a[2]).powi(2)).sqrt();
    let near = [
        ("tft", d([1.0, 1.0, 0.0])),
        ("alld", d([0.0, 0.0, 0.0])),
        ("allc", d([1.0, 1.0, 1.0])),
    ]
    .into_iter()
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .unwrap();
    if near.1 <= 0.5 {
        near.0
    } else {
        "other"
    }
}

pub fn class_color(name: &str) -> Rgb {
    match name {
        "tft" => TFT,
        "alld" => ALLD,
        "allc" => ALLC,
        _ => OTHER,
    }
}

/// A 0–1 value on the low–high scale.
pub fn scale(v: f64) -> Rgb {
    lerp(LOW, HIGH, v)
}

/// A frame being drawn.
pub struct Canvas<'a> {
    pub buf: &'a mut Vec<u8>,
    pub wide: usize,
}

impl Canvas<'_> {
    pub fn clear(&mut self, wide: usize, tall: usize) {
        self.wide = wide;
        self.buf.clear();
        self.buf.resize(wide * tall * 4, 0);
        for k in 0..wide * tall {
            self.buf[k * 4..k * 4 + 4].copy_from_slice(&[
                BACKGROUND[0],
                BACKGROUND[1],
                BACKGROUND[2],
                255,
            ]);
        }
    }

    pub fn put(&mut self, x: usize, y: usize, c: Rgb) {
        let k = (y * self.wide + x) * 4;
        self.buf[k..k + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_fits_block_and_plane() {
        assert_eq!((block_side(256), cell(256)), (16, 6));
        assert_eq!(frame(256), (96 + GAP + PLANE, PLANE));
        assert_eq!((block_side(4096), cell(4096)), (64, 1));
        assert_eq!(block_side(250), 16);
    }

    #[test]
    fn the_plane_puts_q_up_and_p_right() {
        assert_eq!(plane_cell(0.0, 0.0), (0, PLANE - 1));
        assert_eq!(plane_cell(1.0, 1.0), (PLANE - 1, 0));
        assert_eq!(plane_cell(0.5, 0.25), (50, 75));
    }

    #[test]
    fn strategies_fall_into_classes() {
        assert_eq!(class(1.0, 0.95, 0.05), "tft");
        assert_eq!(class(0.1, 0.0, 0.1), "alld");
        assert_eq!(class(0.9, 1.0, 0.9), "allc");
        assert_eq!(class(0.5, 0.5, 0.5), "other");
    }
}
```

Create `crates/sugarscape-core/src/structure/world.rs` with exactly this content:

```rust
//! The Social Structure world: each period every agent plays short iterated
//! Prisoner's Dilemmas with the partners its social structure gives it, then
//! copies its best partner if that partner did strictly better, with CRA's
//! two kinds of error.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{NoiseOn, Start, Structure, StructureConfig};
use super::graph::{other, Graph};
use super::stats::{payoff, regression, StructureSnapshot};
use super::view::{
    block_side, cell, class, class_color, frame, plane_cell, plane_x, scale, Canvas, DOT, DOT_MANY,
    PLANE, PLANE_BG, TRAIL, TRAIL_NEW, TRAIL_OLD,
};
use crate::anasazi::random::normal;
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::lerp;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// A strategy (Nowak & Sigmund's): cooperate first with probability y,
/// after the other's cooperation with p, after its defection with q.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Strategy {
    pub y: f64,
    pub p: f64,
    pub q: f64,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureMode {
    /// p on a red–green scale.
    Friendliness,
    /// 1 − q on a red–green scale.
    Provocability,
    /// This period's payoff per move (0–5 scaled to 0–3).
    Payoff,
    /// The nearest of TFT, ALLD, ALLC, or other.
    Strategy,
}

impl std::str::FromStr for StructureMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "friendliness" => Self::Friendliness,
            "provocability" => Self::Provocability,
            "payoff" => Self::Payoff,
            "strategy" => Self::Strategy,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StructureInspection {
    pub site: StructureCell,
    /// The block cell clicked, null off the block.
    pub block: Option<StructureCell>,
    /// The (p, q) of the plane cell clicked, null off the plane.
    pub plane: Option<[f64; 2]>,
    /// The agents at the plane cell (none on the block).
    pub agents: Vec<AgentView>,
    /// The agent in the block cell, with its partners.
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct StructureCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub y: f64,
    pub p: f64,
    pub q: f64,
    /// `"tft"`, `"alld"`, `"allc"` or `"other"`.
    pub class: &'static str,
    /// This period's payoff per move.
    pub score: f64,
    /// The partner copied this period.
    pub copied: Option<u64>,
    /// The agents played this period (empty in plane lists).
    pub partners: Vec<PartnerView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PartnerView {
    pub id: u64,
    /// The strategy's p as played this period.
    pub p: f64,
    pub score: f64,
    pub games: u32,
}

#[derive(Clone)]
pub struct StructureWorld {
    pub config: StructureConfig,
    /// Completed periods.
    pub tick: u64,
    strategies: Vec<Strategy>,
    graph: Arc<Graph>,
    rng: SimRng,
    /// This period's: strategies as played, scores, the distinct partners
    /// met with game counts, and whom each agent copied.
    played: Vec<Strategy>,
    scores: Vec<f64>,
    met: Vec<Vec<(u32, u32)>>,
    copied: Vec<Option<u32>>,
    /// The population's mean (p, q), newest last.
    trail: VecDeque<(f64, f64)>,
    attained_high: Option<u64>,
    high_since: u64,
    pub stats: Stats<StructureSnapshot>,
}

impl StructureWorld {
    pub fn new(config: StructureConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.agents as usize;
        let strategies: Vec<Strategy> = match config.start {
            Start::Grid => {
                let k = block_side(n);
                (0..n)
                    .map(|i| {
                        let p = ((i % k) as f64 + 0.5) / k as f64;
                        let q = ((i / k) as f64 + 0.5) / k as f64;
                        Strategy { y: p, p, q }
                    })
                    .collect()
            }
            Start::Random => (0..n)
                .map(|_| {
                    let p = rng.gen::<f64>();
                    let q = rng.gen::<f64>();
                    Strategy { y: p, p, q }
                })
                .collect(),
        };
        let graph = Arc::new(Graph::new(&config, &mut rng));
        let mut world = StructureWorld {
            config,
            tick: 0,
            played: strategies.clone(),
            strategies,
            graph,
            rng,
            scores: vec![0.0; n],
            met: vec![Vec::new(); n],
            copied: vec![None; n],
            trail: VecDeque::with_capacity(TRAIL),
            attained_high: None,
            high_since: 0,
            stats: Stats::default(),
        };
        world.record(0.0, 0.0);
        Ok(world)
    }

    pub fn strategies(&self) -> &[Strategy] {
        &self.strategies
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Whether the run has stopped at `stop_at`.
    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    /// This period's partners chosen by each agent.
    fn partners(&mut self) -> Vec<Vec<u32>> {
        let n = self.strategies.len();
        let k = self.config.partners;
        let x = self.config.substitution;
        match self.config.structure {
            Structure::Rwr => (0..n)
                .map(|a| (0..k).map(|_| other(&mut self.rng, n, a)).collect())
                .collect(),
            _ => {
                let graph = self.graph.clone();
                (0..n)
                    .map(|a| {
                        graph.chosen[a]
                            .iter()
                            .map(|&b| {
                                if x > 0.0 && self.rng.gen::<f64>() < x {
                                    other(&mut self.rng, n, a)
                                } else {
                                    b
                                }
                            })
                            .collect()
                    })
                    .collect()
            }
        }
    }

    /// One game of `moves` moves: each player's payoff and cooperations.
    fn game(&mut self, a: Strategy, b: Strategy) -> ([u32; 2], [u32; 2]) {
        let (mut pay, mut coop) = ([0; 2], [0; 2]);
        let (mut last_a, mut last_b) = (true, true);
        for m in 0..self.config.moves {
            let (pa, pb) = if m == 0 {
                (a.y, b.y)
            } else {
                (
                    if last_b { a.p } else { a.q },
                    if last_a { b.p } else { b.q },
                )
            };
            let ca = self.rng.gen::<f64>() < pa;
            let cb = self.rng.gen::<f64>() < pb;
            pay[0] += payoff(ca, cb);
            pay[1] += payoff(cb, ca);
            coop[0] += u32::from(ca);
            coop[1] += u32::from(cb);
            (last_a, last_b) = (ca, cb);
        }
        (pay, coop)
    }

    /// One period: games, scores, adaptation.
    // `a` indexes met, scores, played and copied as well as next.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self) {
        let n = self.strategies.len();
        let chosen = self.partners();
        self.played = self.strategies.clone();
        let mut payoffs = vec![0u64; n];
        let mut moves = vec![0u64; n];
        let (mut coop_total, mut move_total) = (0u64, 0u64);
        let mut met: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n];
        let meet = |met: &mut Vec<Vec<(u32, u32)>>, a: usize, b: u32| match met[a]
            .iter_mut()
            .find(|e| e.0 == b)
        {
            Some(e) => e.1 += 1,
            None => met[a].push((b, 1)),
        };
        let m = u64::from(self.config.moves);
        for (a, list) in chosen.iter().enumerate() {
            for &b in list {
                let (pay, coop) = self.game(self.played[a], self.played[b as usize]);
                payoffs[a] += u64::from(pay[0]);
                payoffs[b as usize] += u64::from(pay[1]);
                moves[a] += m;
                moves[b as usize] += m;
                coop_total += u64::from(coop[0] + coop[1]);
                move_total += 2 * m;
                meet(&mut met, a, b);
                meet(&mut met, b as usize, a as u32);
            }
        }
        self.scores = (0..n)
            .map(|a| {
                if moves[a] == 0 {
                    0.0
                } else {
                    payoffs[a] as f64 / moves[a] as f64
                }
            })
            .collect();
        self.met = met;
        // Adaptation, from this period's strategies and scores.
        let mut next = self.played.clone();
        self.copied = vec![None; n];
        for a in 0..n {
            let mut best: Vec<u32> = Vec::new();
            let mut top = f64::NEG_INFINITY;
            for &(b, _) in &self.met[a] {
                let s = self.scores[b as usize];
                if s > top {
                    top = s;
                    best.clear();
                    best.push(b);
                } else if s == top {
                    best.push(b);
                }
            }
            let mut copy = false;
            if !best.is_empty() {
                let pick = best[self.rng.gen_range(0..best.len() as u32) as usize];
                let better = self.scores[pick as usize] > self.scores[a];
                let wrong = self.rng.gen::<f64>() < self.config.judge_error;
                if better != wrong {
                    next[a] = self.played[pick as usize];
                    self.copied[a] = Some(pick);
                    copy = true;
                }
            }
            if self.config.noise_on == NoiseOn::Always || copy {
                let s = &mut next[a];
                for v in [&mut s.y, &mut s.p, &mut s.q] {
                    if self.rng.gen::<f64>() < self.config.mutation {
                        *v = (*v + normal(&mut self.rng) * self.config.mutation_sd).clamp(0.0, 1.0);
                    }
                }
            }
        }
        self.strategies = next;
        self.tick += 1;
        let mean_payoff = if move_total == 0 {
            0.0
        } else {
            payoffs.iter().sum::<u64>() as f64 / moves.iter().sum::<u64>() as f64
        };
        let cooperation = if move_total == 0 {
            0.0
        } else {
            coop_total as f64 / move_total as f64
        };
        self.record(mean_payoff, cooperation);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// (own p, partners' mean p) for every agent that played this period,
    /// with strategies as played (CRA's Figs. 5–6).
    pub fn partner_p_pairs(&self) -> Vec<(f64, f64)> {
        (0..self.played.len())
            .filter(|&a| !self.met[a].is_empty())
            .map(|a| {
                let (sum, count) = self.met[a].iter().fold((0.0, 0u32), |(s, c), &(b, g)| {
                    (s + self.played[b as usize].p * f64::from(g), c + g)
                });
                (self.played[a].p, sum / f64::from(count))
            })
            .collect()
    }

    fn record(&mut self, mean_payoff: f64, cooperation: f64) {
        let n = self.strategies.len() as f64;
        let mean = |f: fn(&Strategy) -> f64| self.strategies.iter().map(f).sum::<f64>() / n;
        let (mean_y, mean_p, mean_q) = (mean(|s| s.y), mean(|s| s.p), mean(|s| s.q));
        let high = self.tick > 0 && mean_payoff >= self.config.high;
        if high && self.attained_high.is_none() {
            self.attained_high = Some(self.tick);
        }
        if high && self.attained_high.is_some() {
            self.high_since += 1;
        }
        let share_high_since = match self.attained_high {
            Some(t) => self.high_since as f64 / (self.tick - t + 1) as f64,
            None => 0.0,
        };
        if self.trail.len() == TRAIL {
            self.trail.pop_front();
        }
        self.trail.push_back((mean_p, mean_q));
        let copied = self.copied.iter().filter(|c| c.is_some()).count() as f64 / n;
        let partner_p_slope = regression(&self.partner_p_pairs()).0;
        self.stats.push(StructureSnapshot {
            tick: self.tick,
            mean_payoff,
            cooperation,
            mean_y,
            mean_p,
            mean_q,
            high: u8::from(high),
            attained_high: self.attained_high.map_or(-1, |t| t as i64),
            share_high_since,
            copied,
            partner_p_slope,
        });
    }

    /// The block cell of agent `a`: its torus site, else its index.
    fn block_of(&self, a: usize) -> (usize, usize) {
        let k = if self.graph.site.is_empty() {
            a
        } else {
            self.graph.site[a] as usize
        };
        let s = block_side(self.strategies.len());
        (k % s, k / s)
    }

    /// The agent at block cell (bx, by), if any.
    fn agent_at(&self, bx: usize, by: usize) -> Option<usize> {
        let s = block_side(self.strategies.len());
        let k = by * s + bx;
        if k >= self.strategies.len() {
            return None;
        }
        Some(if self.graph.agent_at.is_empty() {
            k
        } else {
            self.graph.agent_at[k] as usize
        })
    }

    fn view(&self, a: usize, with_partners: bool) -> AgentView {
        let s = self.strategies[a];
        AgentView {
            id: a as u64 + 1,
            y: s.y,
            p: s.p,
            q: s.q,
            class: class(s.y, s.p, s.q),
            score: self.scores[a],
            copied: self.copied[a].map(|b| u64::from(b) + 1),
            partners: if with_partners {
                self.met[a]
                    .iter()
                    .map(|&(b, g)| PartnerView {
                        id: u64::from(b) + 1,
                        p: self.played[b as usize].p,
                        score: self.scores[b as usize],
                        games: g,
                    })
                    .collect()
            } else {
                Vec::new()
            },
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<StructureInspection, String> {
        let n = self.strategies.len();
        let (fw, fh) = frame(n);
        if x as usize >= fw || y as usize >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let mut out = StructureInspection {
            site: StructureCell { x, y },
            block: None,
            plane: None,
            agents: Vec::new(),
            agent: None,
        };
        let (cx, cy) = (x as usize, y as usize);
        let c = cell(n);
        let side = block_side(n);
        if cx < side * c && cy < side * c {
            let (bx, by) = (cx / c, cy / c);
            out.block = Some(StructureCell {
                x: bx as u32,
                y: by as u32,
            });
            out.agent = self.agent_at(bx, by).map(|a| self.view(a, true));
        } else if cx >= plane_x(n) && cy < PLANE {
            let (px, py) = (cx - plane_x(n), cy);
            let p = px as f64 / (PLANE - 1) as f64;
            let q = (PLANE - 1 - py) as f64 / (PLANE - 1) as f64;
            out.plane = Some([p, q]);
            out.agents = (0..n)
                .filter(|&a| plane_cell(self.strategies[a].p, self.strategies[a].q) == (px, py))
                .map(|a| self.view(a, false))
                .collect();
        }
        Ok(out)
    }
}

impl Model for StructureWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Structure(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        StructureWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.strategies.len()
    }

    /// FNV-1a over the tick and every strategy's bits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: [u8; 8]| {
            for b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick.to_le_bytes());
        for s in &self.strategies {
            for v in [s.y, s.p, s.q] {
                eat(v.to_bits().to_le_bytes());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (w, h) = frame(self.strategies.len());
        (w as u32, h as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: StructureMode = mode.parse()?;
        let n = self.strategies.len();
        let (fw, fh) = frame(n);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw, fh);
        let size = cell(n);
        for a in 0..n {
            let s = self.strategies[a];
            let color = match mode {
                StructureMode::Friendliness => scale(s.p),
                StructureMode::Provocability => scale(1.0 - s.q),
                StructureMode::Payoff => scale(self.scores[a] / 3.0),
                StructureMode::Strategy => class_color(class(s.y, s.p, s.q)),
            };
            let (bx, by) = self.block_of(a);
            for dy in 0..size {
                for dx in 0..size {
                    c.put(bx * size + dx, by * size + dy, color);
                }
            }
        }
        let ox = plane_x(n);
        for y in 0..PLANE {
            for x in 0..PLANE {
                c.put(ox + x, y, PLANE_BG);
            }
        }
        let len = self.trail.len();
        for (i, &(p, q)) in self.trail.iter().enumerate() {
            let (x, y) = plane_cell(p, q);
            c.put(
                ox + x,
                y,
                lerp(TRAIL_OLD, TRAIL_NEW, (i + 1) as f64 / len as f64),
            );
        }
        let mut counts = vec![0u32; PLANE * PLANE];
        for s in &self.strategies {
            let (x, y) = plane_cell(s.p, s.q);
            counts[y * PLANE + x] += 1;
        }
        for (k, &count) in counts.iter().enumerate() {
            if count > 0 {
                let t = (f64::from(count - 1) / 8.0).min(1.0);
                c.put(ox + k % PLANE, k / PLANE, lerp(DOT, DOT_MANY, t));
            }
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
        let mut out = String::from("id,y,p,q,class,score,copied,partners\n");
        for a in 0..self.strategies.len() {
            let v = self.view(a, false);
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                v.id,
                v.y,
                v.p,
                v.q,
                v.class,
                v.score,
                v.copied.map_or(String::new(), |c| c.to_string()),
                self.met[a].len()
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// An agent's block cell (its center).
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let a = usize::try_from(id.checked_sub(1)?).ok()?;
        if a >= self.strategies.len() {
            return None;
        }
        let (bx, by) = self.block_of(a);
        let c = cell(self.strategies.len());
        Some(((bx * c + c / 2) as u32, (by * c + c / 2) as u32))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Structure(next) = next else {
            return Err(wrong_model(ModelKind::Structure, &next));
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(edit: impl FnOnce(&mut StructureConfig)) -> StructureConfig {
        let mut c = StructureConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut StructureConfig)) -> StructureWorld {
        StructureWorld::new(config(edit), 1).unwrap()
    }

    const TFT: Strategy = Strategy {
        y: 1.0,
        p: 1.0,
        q: 0.0,
    };
    const ALLD: Strategy = Strategy {
        y: 0.0,
        p: 0.0,
        q: 0.0,
    };
    const ALLC: Strategy = Strategy {
        y: 1.0,
        p: 1.0,
        q: 1.0,
    };

    #[test]
    fn games_follow_the_strategies() {
        let mut w = world(|_| {});
        assert_eq!(w.game(TFT, TFT), ([12, 12], [4, 4]));
        assert_eq!(w.game(ALLD, ALLC), ([20, 0], [0, 4]));
        // TFT against ALLD: cooperates once, then defects: 0 + 1 + 1 + 1.
        assert_eq!(w.game(TFT, ALLD), ([3, 8], [1, 0]));
    }

    #[test]
    fn starts_spread_evenly_or_at_random_with_y_equal_to_p() {
        let g = world(|_| {});
        assert_eq!(
            g.strategies()[0],
            Strategy {
                y: 1.0 / 32.0,
                p: 1.0 / 32.0,
                q: 1.0 / 32.0
            }
        );
        assert_eq!(g.strategies()[17].p, 1.5 / 16.0);
        assert_eq!(g.strategies()[17].q, 1.5 / 16.0);
        let r = world(|c| c.start = Start::Random);
        assert!(r.strategies().iter().all(|s| s.y == s.p));
        assert_ne!(r.strategies()[0], g.strategies()[0]);
    }

    #[test]
    fn a_period_plays_every_chosen_game_in_both_roles() {
        let mut w = world(|c| {
            c.mutation = 0.0;
            c.judge_error = 0.0;
        });
        w.step();
        let games: u32 = w.met.iter().flatten().map(|e| e.1).sum();
        assert_eq!(games, 2 * 256 * 4, "each game counted for both players");
        assert!(w
            .met
            .iter()
            .enumerate()
            .all(|(a, l)| l.iter().all(|e| e.0 as usize != a)));
        let s = w.stats.latest().unwrap();
        // Fig. 1: a spread-out start realizes each cell about a quarter of the time.
        assert!((s.mean_payoff - 2.25).abs() < 0.1, "{}", s.mean_payoff);
        let mut t = world(|c| c.structure = Structure::Torus);
        t.step();
        assert!(
            t.met
                .iter()
                .all(|l| l.len() == 4 && l.iter().all(|e| e.1 == 2)),
            "torus pairs play twice"
        );
    }

    #[test]
    fn agents_copy_only_a_strictly_better_partner_unless_they_misjudge() {
        let mut w = world(|c| {
            c.agents = 4;
            c.partners = 1;
            c.structure = Structure::Frn;
            c.mutation = 0.0;
            c.judge_error = 0.0;
        });
        w.strategies = vec![ALLD, ALLC, ALLC, ALLC];
        w.step();
        // ALLD beats every ALLC it meets; ALLCs that met ALLD copy it.
        for a in 0..4 {
            match w.copied[a] {
                Some(b) => {
                    assert!(w.scores[b as usize] > w.scores[a]);
                    assert_eq!(w.strategies[a], w.played[b as usize]);
                }
                None => assert_eq!(w.strategies[a], w.played[a]),
            }
        }
        assert_eq!(w.copied[0], None, "the best copies no one");
        let mut e = world(|c| {
            c.agents = 4;
            c.partners = 1;
            c.structure = Structure::Frn;
            c.mutation = 0.0;
            c.judge_error = 1.0;
        });
        e.strategies = vec![ALLD, ALLC, ALLC, ALLC];
        e.step();
        assert!(
            e.copied[0].is_some(),
            "always misjudging, the best copies a worse partner"
        );
    }

    #[test]
    fn noise_hits_everyone_or_only_copiers() {
        let run = |noise_on| {
            let mut w = world(|c| {
                c.mutation = 1.0;
                c.judge_error = 0.0;
                c.noise_on = noise_on;
            });
            w.step();
            (0..256)
                .filter(|&a| w.copied[a].is_none() && w.strategies[a] != w.played[a])
                .count()
        };
        assert!(run(NoiseOn::Always) > 0);
        assert_eq!(run(NoiseOn::Copy), 0);
        let mut w = world(|c| {
            c.mutation = 1.0;
            c.mutation_sd = 2.0;
        });
        w.run(3);
        assert!(w
            .strategies()
            .iter()
            .all(|s| [s.y, s.p, s.q].iter().all(|v| (0.0..=1.0).contains(v))));
    }

    #[test]
    fn substitution_replaces_fixed_links_only_for_the_period() {
        let mut w = world(|c| {
            c.structure = Structure::Frn;
            c.substitution = 1.0;
        });
        let fixed = w.graph.chosen.clone();
        let chosen = w.partners();
        let same = chosen.iter().zip(&fixed).filter(|(a, b)| a == b).count();
        assert!(same < 5, "{same} agents kept every link");
        assert_eq!(w.graph.chosen, fixed, "the network itself is untouched");
        let mut none = world(|c| c.structure = Structure::Frn);
        assert_eq!(none.partners(), none.graph.chosen);
    }

    #[test]
    fn high_cooperation_is_attained_and_remembered() {
        let mut w = world(|c| {
            c.mutation = 0.0;
            c.judge_error = 0.0;
            c.structure = Structure::Frne;
            c.high = 2.9;
        });
        w.strategies = vec![TFT; 256];
        w.step();
        let s = w.stats.latest().unwrap().clone();
        assert_eq!(
            (s.mean_payoff, s.cooperation, s.high, s.attained_high),
            (3.0, 1.0, 1, 1)
        );
        w.config.high = 3.5;
        w.step();
        let s = w.stats.latest().unwrap();
        assert_eq!((s.high, s.attained_high, s.share_high_since), (0, 1, 0.5));
        assert_eq!(
            w.stats.history()[0].mean_payoff,
            0.0,
            "no games before period 1"
        );
    }

    #[test]
    fn the_partner_regression_reads_the_strategies_as_played() {
        let mut w = world(|c| {
            c.structure = Structure::Frne;
            c.mutation = 0.0;
            c.judge_error = 0.0;
        });
        w.step();
        let pairs = w.partner_p_pairs();
        assert_eq!(pairs.len(), 256);
        let a = 7;
        let mean: f64 = w.met[a]
            .iter()
            .map(|&(b, g)| w.played[b as usize].p * f64::from(g))
            .sum::<f64>()
            / w.met[a].iter().map(|e| f64::from(e.1)).sum::<f64>();
        assert_eq!(pairs[a], (w.played[a].p, mean));
        assert_eq!(
            w.stats.latest().unwrap().partner_p_slope,
            regression(&pairs).0
        );
    }

    #[test]
    fn the_view_draws_block_and_plane_and_inspect_finds_both() {
        let mut w = world(|c| c.structure = Structure::Torus);
        w.run(3);
        let (fw, fh) = frame(256);
        assert_eq!(Model::size(&w), (fw as u32, fh as u32));
        let mut buf = Vec::new();
        for mode in ["friendliness", "provocability", "payoff", "strategy"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), fw * fh * 4);
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let agent = w.graph.agent_at[17] as usize;
        let v = w.inspect((6 + 1) as u32, (6 + 1) as u32).unwrap();
        assert_eq!(v.block, Some(StructureCell { x: 1, y: 1 }));
        let seen = v.agent.unwrap();
        assert_eq!(seen.id, agent as u64 + 1);
        assert_eq!(seen.partners.len(), 4);
        assert_eq!(Model::locate(&w, seen.id), Some((6 + 3, 6 + 3)));
        let s = w.strategies()[0];
        let (px, py) = plane_cell(s.p, s.q);
        let on = w.inspect((plane_x(256) + px) as u32, py as u32).unwrap();
        assert!(on.plane.is_some() && on.agents.iter().any(|a| a.id == 1));
        let gap = w.inspect(97, 0).unwrap();
        assert!(gap.block.is_none() && gap.plane.is_none());
        assert!(w.inspect(fw as u32, 0).is_err());
    }

    #[test]
    fn keyframes_restore_strategies_and_the_trail() {
        let mut any = crate::model::ModelWorld::new(
            ModelConfig::Structure(config(|c| c.structure = Structure::Frn)),
            6,
        )
        .unwrap();
        any.model_mut().run(5);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("friendliness", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("friendliness", "", &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(any.model().series("mean_payoff").unwrap().len(), 6);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.substitution = 0.3;
            c.judge_error = 0.0;
            c.noise_on = NoiseOn::Copy;
            c.high = 2.5;
        });
        Model::set_config(&mut w, ModelConfig::Structure(next.clone())).unwrap();
        w.step();
        let e = Model::set_config(
            &mut w,
            ModelConfig::Structure(StructureConfig {
                structure: Structure::Frn,
                ..next
            }),
        )
        .unwrap_err();
        assert_eq!(e[0].field, "structure");
    }

    #[test]
    fn the_stop_ends_the_run() {
        let mut w = world(|c| c.stop_at = 5);
        w.run(100);
        assert_eq!(w.tick, 5);
        assert!(Model::finished(&w));
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| {
                c.agents = 4;
                c.partners = 3;
            }),
            config(|c| {
                c.agents = 9;
                c.structure = Structure::Torus;
            }),
            config(|c| {
                c.agents = 4;
                c.partners = 2;
                c.structure = Structure::Frne;
            }),
            config(|c| c.judge_error = 1.0),
            config(|c| c.mutation = 1.0),
            config(|c| {
                c.structure = Structure::Frn;
                c.substitution = 1.0;
            }),
            config(|c| c.moves = 1),
            config(|c| c.agents = 250),
        ] {
            let mut w = StructureWorld::new(c.clone(), 1).unwrap();
            w.run(20);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=5.0).contains(&s.mean_payoff), "{c:?}");
            let mut buf = Vec::new();
            w.render("strategy", "", &mut buf).unwrap();
        }
    }
}
```

Create `crates/sugarscape-core/src/structure/presets.rs` with exactly this content:

```rust
//! Table 2's structures and the paper's two readings of its own method.

use super::config::{NoiseOn, Start, Structure, StructureConfig};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const CRA: &str = "Cohen, Riolo & Axelrod 2001";

fn preset(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut StructureConfig),
) -> ModelPreset {
    let mut c = StructureConfig {
        stop_at: 2500,
        ..StructureConfig::default()
    };
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source: CRA,
        description,
        config: ModelConfig::Structure(c),
    }
}

fn frn(c: &mut StructureConfig) {
    c.structure = Structure::Frn;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("cra-rwr", "Random partners each period (RWR)", "Cohen, Riolo and Axelrod's population: 256 agents each period play four-move Prisoner's Dilemmas with four partners (a strategy: cooperate first with probability y, after a cooperation with p, after a defection with q), then copy their best-scoring partner if it did strictly better — misjudging 10 % of the time, with 10 % noise on each of y, p, q. Here partners are drawn afresh every period (RWR, Table 2 row 1). The left block is the agents colored by friendliness p; the right plane is p against q with the population's recent trail. The paper: high cooperation reached in 0.30 of runs and held 1.5 % of the time after, a mean payoff of 1.091. Measured (30 seeds, 2500 periods, high = 2.3): a mean payoff of 1.089; reached in 0.43 of runs, held 2.0 % of the time.", |_| {}),
        preset("cra-2dk", "Torus neighbors (2DK)", "The same agents on a 16 × 16 torus, each playing its four NEWS neighbors — every pair twice a period — for the whole run (2DK, row 2). The block is the torus itself. The paper: every run reaches high cooperation and holds it 99.7 % of the time, a mean payoff of 2.557. Measured (30 seeds, 2500 periods, high = 2.3): 2.553; every run; held 99.8 %.", |c| {
            c.structure = Structure::Torus
        }),
        preset(
            "cra-frne",
            "Fixed random neighbors, symmetric (FRNE)",
            "Fixed random neighbors, four each and symmetric (FRNE, row 3): the torus's fixity without its clustering. The block's positions mean nothing here. The paper: 2.575, every run, held 99.5 % — and (note 5) better than 2DK. Measured (30 seeds, 2500 periods, high = 2.3): 2.574; every run; held 99.7 %; above 2DK's 2.553, as the paper says.",
            |c| c.structure = Structure::Frne,
        ),
        preset("cra-frn", "Fixed random neighbors (FRN)", "Fixed random neighbors, drawn once with replacement, one-way (FRN, row 4): context preservation alone. The paper: 2.480, every run, held 94.2 %. Measured (30 seeds, 2500 periods, high = 2.3): 2.478; every run; held 94.0 %. In the crucial region (population p 0.30–0.35, q 0.05–0.10) p rises 0.051 a period (the paper: 0.052) and an agent's partners' p tracks its own (slope 0.18; the paper 0.158).", frn),
        preset(
            "cra-ffr-01",
            "Fixed, 10 % substituted (FFR-0.1)",
            "FRN with each fixed partner replaced, for the period, by a random one with probability 0.1 (FFR-0.1, row 5). The paper: 2.385, held 84.4 %. Measured (30 seeds, 2500 periods, high = 2.3): 2.405; every run; held 84.3 %.",
            |c| {
                frn(c);
                c.substitution = 0.1;
            },
        ),
        preset(
            "cra-ffr-03",
            "Fixed, 30 % substituted (FFR-0.3)",
            "FFR-0.3 (row 6): 'around a parameter value of 0.3 the dynamics shift'; the paper calls it bi-stable. The paper: 2.100, held 40.2 %. Measured (30 seeds, 2500 periods, high = 2.3): 2.036; every run; held 37.6 %; in 25 of 30 runs stretches of at least 50 periods both high and low.",
            |c| {
                frn(c);
                c.substitution = 0.3;
            },
        ),
        preset(
            "cra-ffr-05",
            "Fixed, 50 % substituted (FFR-0.5)",
            "FFR-0.5 (row 7): 'at levels of 0.5 and above, it collapses'. The paper: reached in 0.93 of runs, 1.257, held 6.1 %. Measured (30 seeds, 2500 periods, high = 2.3): reached in 0.97, 1.325, held 7.3 %.",
            |c| {
                frn(c);
                c.substitution = 0.5;
            },
        ),
        preset("cra-random-start", "FRN from a random start", "FRN from random strategies (§3.1's 'initialized randomly') instead of the Appendix's even spread. Measured (30 seeds, 2500 periods, high = 2.3): 2.473 and held 93.8 % against 2.478 and 94.0 % from the even spread — the paper's two descriptions of its start make no difference.", |c| {
            frn(c);
            c.start = Start::Random;
        }),
        preset("cra-copy-noise", "FRN, noise only on copying", "FRN with noise only on agents that copy (§2's 'errors in the actual copying process') instead of on every agent every period (the Appendix). Measured (30 seeds, 2500 periods, high = 2.3): 2.530 and held 98.4 % against 2.478 and 94.0 % — less noise, more cooperation; the Appendix's reading is the one that matches Table 2.", |c| {
            frn(c);
            c.noise_on = NoiseOn::Copy;
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, StructureConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Structure(c) => (p.id, c),
                _ => panic!("{} is not a social-structure preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("cra-rwr").structure, Structure::Rwr);
        assert_eq!(find("cra-2dk").structure, Structure::Torus);
        let f = find("cra-ffr-03");
        assert_eq!((f.structure, f.substitution), (Structure::Frn, 0.3));
        assert_eq!(find("cra-random-start").start, Start::Random);
        assert_eq!(find("cra-copy-noise").noise_on, NoiseOn::Copy);
        assert!(got
            .iter()
            .all(|(_, c)| c.stop_at == 2500 && c.validate().is_ok()));
    }
}
```

Create `crates/sugarscape-core/src/structure/mod.rs` with exactly this content:

```rust
//! Social Structure (milestone 18): Cohen, Riolo and Axelrod, "The Role of
//! Social Structure in the Maintenance of Cooperative Regimes" (Rationality
//! and Society 2001), with the points the paper leaves unstated or states
//! twice as named switches. See docs/superpowers/specs/2026-09-26-social-structure-design.md.

mod config;
mod graph;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{schema, square_side, NoiseOn, Start, Structure, StructureConfig};
pub use graph::{fanout, Graph};
pub use presets::presets;
pub use stats::{payoff, regression, StructureSnapshot, SERIES};
pub use view::{block_side, cell, class, frame, plane_cell, plane_x, PLANE, TRAIL};
pub use world::{
    AgentView, PartnerView, Strategy, StructureCell, StructureInspection, StructureMode,
    StructureWorld,
};
```

- [ ] **Step 2: See its tests not run yet**

Run: `cargo test -p sugarscape-core --lib structure::`
Expected: 0 tests (the module is not in the crate).

- [ ] **Step 3: Wire it in**

Apply to `crates/sugarscape-core/src/lib.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index ca57dbb..849466d 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -31,6 +31,7 @@ pub mod schema;
 pub mod social;
 pub mod spatial;
 pub mod stats;
+pub mod structure;
 pub mod sweep;
 pub mod tags;
 pub mod world;
```

Apply to `crates/sugarscape-core/src/model.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index bf3e434..c634d8c 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -17,11 +17,12 @@ use crate::ring::{RingConfig, RingWorld};
 use crate::schelling::{SchellingConfig, SchellingWorld};
 use crate::schema::Param;
 use crate::spatial::{SpatialConfig, SpatialWorld};
+use crate::structure::{StructureConfig, StructureWorld};
 use crate::tags::{TagsConfig, TagsWorld};
 use crate::world::World;
 use crate::{
     anasazi, civil, classes, culture, ethno, export, opinions, ring, schelling, spatial, stats,
-    tags,
+    structure, tags,
 };
 
 /// Which model a config or world is.
@@ -39,10 +40,11 @@ pub enum ModelKind {
     Classes,
     Ethno,
     Opinions,
+    Structure,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 11] = [
+    pub const ALL: [ModelKind; 12] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -54,6 +56,7 @@ impl ModelKind {
         ModelKind::Classes,
         ModelKind::Ethno,
         ModelKind::Opinions,
+        ModelKind::Structure,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -69,6 +72,7 @@ impl ModelKind {
             ModelKind::Classes => "classes",
             ModelKind::Ethno => "ethno",
             ModelKind::Opinions => "opinions",
+            ModelKind::Structure => "structure",
         }
     }
 
@@ -87,6 +91,7 @@ impl ModelKind {
             ModelKind::Classes => classes::schema(),
             ModelKind::Ethno => ethno::schema(),
             ModelKind::Opinions => opinions::schema(),
+            ModelKind::Structure => structure::schema(),
         }
     }
 }
@@ -111,6 +116,7 @@ pub enum ModelConfig {
     Classes(ClassesConfig),
     Ethno(EthnoConfig),
     Opinions(OpinionsConfig),
+    Structure(StructureConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -127,6 +133,7 @@ enum Tagged<'a> {
     Classes(&'a ClassesConfig),
     Ethno(&'a EthnoConfig),
     Opinions(&'a OpinionsConfig),
+    Structure(&'a StructureConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -150,6 +157,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Classes(c) => Tagged::Classes(c).serialize(s),
             ModelConfig::Ethno(c) => Tagged::Ethno(c).serialize(s),
             ModelConfig::Opinions(c) => Tagged::Opinions(c).serialize(s),
+            ModelConfig::Structure(c) => Tagged::Structure(c).serialize(s),
         }
     }
 }
@@ -168,6 +176,7 @@ impl ModelConfig {
             ModelConfig::Classes(_) => ModelKind::Classes,
             ModelConfig::Ethno(_) => ModelKind::Ethno,
             ModelConfig::Opinions(_) => ModelKind::Opinions,
+            ModelConfig::Structure(_) => ModelKind::Structure,
         }
     }
 
@@ -235,10 +244,13 @@ impl ModelConfig {
             "opinions" => serde_json::from_value(value)
                 .map(ModelConfig::Opinions)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "structure" => serde_json::from_value(value)
+                .map(ModelConfig::Structure)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno or opinions)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions or structure)"
                 ),
             )),
         }
@@ -257,6 +269,7 @@ impl ModelConfig {
             ModelConfig::Classes(c) => c.validate(),
             ModelConfig::Ethno(c) => c.validate(),
             ModelConfig::Opinions(c) => c.validate(),
+            ModelConfig::Structure(c) => c.validate(),
         }
     }
 
@@ -275,6 +288,7 @@ impl ModelConfig {
             ModelConfig::Classes(c) => set_path(c, path, value).map(ModelConfig::Classes),
             ModelConfig::Ethno(c) => set_path(c, path, value).map(ModelConfig::Ethno),
             ModelConfig::Opinions(c) => set_path(c, path, value).map(ModelConfig::Opinions),
+            ModelConfig::Structure(c) => set_path(c, path, value).map(ModelConfig::Structure),
         }
     }
 
@@ -292,7 +306,8 @@ impl ModelConfig {
             | ModelConfig::Spatial(_)
             | ModelConfig::Culture(_)
             | ModelConfig::Classes(_)
-            | ModelConfig::Opinions(_) => None,
+            | ModelConfig::Opinions(_)
+            | ModelConfig::Structure(_) => None,
         }
     }
 
@@ -310,6 +325,7 @@ impl ModelConfig {
             ModelConfig::Classes(_) => classes::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Ethno(_) => ethno::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Opinions(_) => opinions::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Structure(_) => structure::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -490,6 +506,7 @@ pub enum ModelWorld {
     Classes(Box<ClassesWorld>),
     Ethno(Box<EthnoWorld>),
     Opinions(Box<OpinionsWorld>),
+    Structure(Box<StructureWorld>),
 }
 
 impl ModelWorld {
@@ -522,6 +539,9 @@ impl ModelWorld {
             ModelConfig::Opinions(c) => {
                 ModelWorld::Opinions(Box::new(OpinionsWorld::new(c, seed)?))
             }
+            ModelConfig::Structure(c) => {
+                ModelWorld::Structure(Box::new(StructureWorld::new(c, seed)?))
+            }
         })
     }
 
@@ -538,6 +558,7 @@ impl ModelWorld {
             ModelWorld::Classes(_) => ModelKind::Classes,
             ModelWorld::Ethno(_) => ModelKind::Ethno,
             ModelWorld::Opinions(_) => ModelKind::Opinions,
+            ModelWorld::Structure(_) => ModelKind::Structure,
         }
     }
 
@@ -554,6 +575,7 @@ impl ModelWorld {
             ModelWorld::Classes(w) => w.as_ref(),
             ModelWorld::Ethno(w) => w.as_ref(),
             ModelWorld::Opinions(w) => w.as_ref(),
+            ModelWorld::Structure(w) => w.as_ref(),
         }
     }
 
@@ -570,6 +592,7 @@ impl ModelWorld {
             ModelWorld::Classes(w) => w.as_mut(),
             ModelWorld::Ethno(w) => w.as_mut(),
             ModelWorld::Opinions(w) => w.as_mut(),
+            ModelWorld::Structure(w) => w.as_mut(),
         }
     }
 
@@ -654,6 +677,7 @@ impl ModelWorld {
             ModelWorld::Classes(w) => copy_without_history!(Classes, w),
             ModelWorld::Ethno(w) => copy_without_history!(Ethno, w),
             ModelWorld::Opinions(w) => copy_without_history!(Opinions, w),
+            ModelWorld::Structure(w) => copy_without_history!(Structure, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -680,6 +704,7 @@ impl ModelWorld {
             (ModelWorld::Classes(live), ModelWorld::Classes(kept)) => restore_into!(live, kept),
             (ModelWorld::Ethno(live), ModelWorld::Ethno(kept)) => restore_into!(live, kept),
             (ModelWorld::Opinions(live), ModelWorld::Opinions(kept)) => restore_into!(live, kept),
+            (ModelWorld::Structure(live), ModelWorld::Structure(kept)) => restore_into!(live, kept),
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -966,7 +991,8 @@ mod tests {
                 "culture",
                 "classes",
                 "ethno",
-                "opinions"
+                "opinions",
+                "structure"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
```

Apply to `crates/sugarscape-core/src/presets.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index 7898fd2..a725aa9 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -684,6 +684,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::culture::presets());
     out.extend(crate::classes::presets());
     out.extend(crate::opinions::presets());
+    out.extend(crate::structure::presets());
     out.extend(crate::spatial::presets());
     out.extend(crate::ethno::presets());
     out
```

Run: `cargo test -p sugarscape-core --lib structure::`
Expected: PASS (26 tests).

- [ ] **Step 4: Golden entries (fail first)**

Run: `cargo test -p sugarscape-core --release --test golden`
Expected: FAIL — `every_model_preset_has_a_golden_entry` asks for `cra-rwr`.

Apply to `crates/sugarscape-core/tests/golden.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 4b5366f..8dfbc00 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -143,6 +143,15 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("hk-bias", 0x9a2ed969514cbda2),
     ("hk-serial", 0xb06db73333504889),
     ("hk-lattice", 0xe33359f120b204d9),
+    ("cra-rwr", 0xc7f45f59d9b25490),
+    ("cra-2dk", 0x3b8c19aab4aae805),
+    ("cra-frne", 0xdf2fc96965742a81),
+    ("cra-frn", 0x924d4b2fe686ae18),
+    ("cra-ffr-01", 0x424eda2182150e01),
+    ("cra-ffr-03", 0xbc09206184dc7003),
+    ("cra-ffr-05", 0xf9573021f9848025),
+    ("cra-random-start", 0x1871afd34df774a9),
+    ("cra-copy-noise", 0xd8871da3505ee758),
 ];
 
 fn fingerprint(id: &str) -> u64 {
```

Run it again. Expected: PASS.

- [ ] **Step 5: Format, lint, test, commit**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test -p sugarscape-core`

```bash
git add crates/sugarscape-core/src/structure/ crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs
git commit -m "Add Social Structure (Cohen, Riolo & Axelrod) as a model kind" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/cra-table-2.json`, `sweeps/cra-dial.json`, `sweeps/cra-threshold.json`, `sweeps/cra-noise.json`, `sweeps/cra-population.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: presets and series from Task 1.
- Produces: the five built-in sweeps.

- [ ] **Step 1: The sweeps** (descriptions record what planning measured with exactly these files)

Create `sweeps/cra-table-2.json` with exactly this content:

```json
{
  "name": "Social Structure: Table 2's mean payoffs",
  "description": "Cohen, Riolo and Axelrod's Table 2: the mean payoff per move over the last 1000 of 2500 periods under each social structure. The paper (30 runs): RWR 1.091, 2DK 2.557, FRNE 2.575, FRN 2.480, FFR-0.1 2.385, FFR-0.3 2.100, FFR-0.5 1.257. Measured (release, seeds 1–10, 2500 periods, recorded 2026-09-26): 1.089, 2.554, 2.573, 2.479, 2.412, 2.008, 1.302 — within 0.1 everywhere; the survey's 30 runs come closer still.",
  "base": {
    "preset": "cra-frn"
  },
  "x": {
    "label": "Social structure",
    "values": [
      {
        "at": 0,
        "name": "RWR",
        "set": {
          "structure": "rwr",
          "substitution": 0.0
        }
      },
      {
        "at": 1,
        "name": "2DK",
        "set": {
          "structure": "torus",
          "substitution": 0.0
        }
      },
      {
        "at": 2,
        "name": "FRNE",
        "set": {
          "structure": "frne",
          "substitution": 0.0
        }
      },
      {
        "at": 3,
        "name": "FRN",
        "set": {
          "structure": "frn",
          "substitution": 0.0
        }
      },
      {
        "at": 4,
        "name": "FFR 0.1",
        "set": {
          "structure": "frn",
          "substitution": 0.1
        }
      },
      {
        "at": 5,
        "name": "FFR 0.3",
        "set": {
          "structure": "frn",
          "substitution": 0.3
        }
      },
      {
        "at": 6,
        "name": "FFR 0.5",
        "set": {
          "structure": "frn",
          "substitution": 0.5
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2500,
  "metric": {
    "kind": "window_mean",
    "series": "mean_payoff",
    "from": 1501
  }
}
```

Create `sweeps/cra-dial.json` with exactly this content:

```json
{
  "name": "Social Structure: turning the dial (FFR)",
  "description": "'Turning the dial': FRN with each fixed partner replaced for the period by a random one with probability x, from 0 (FRN) to 1 (random mixing each period). The paper: 'Around a parameter value of 0.3 the dynamics shift … at levels of 0.5 and above, it collapses.' Measured (release, seeds 1–10, 2500 periods, recorded 2026-09-26): 2.48, 2.41, 2.28, 2.01, 1.77, 1.30, 1.15, 1.11, 1.10, 1.09, 1.10 at x = 0 … 1 — the steepest fall between 0.3 and 0.5, flat from 0.6.",
  "base": {
    "preset": "cra-frn"
  },
  "x": {
    "label": "Random substitution",
    "path": "substitution",
    "values": [
      0.0,
      0.1,
      0.2,
      0.3,
      0.4,
      0.5,
      0.6,
      0.7,
      0.8,
      0.9,
      1.0
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2500,
  "metric": {
    "kind": "window_mean",
    "series": "mean_payoff",
    "from": 1501
  }
}
```

Create `sweeps/cra-threshold.json` with exactly this content:

```json
{
  "name": "Social Structure: how much the unstated threshold matters",
  "description": "The paper never states what counts as 'high cooperation'. The metric is Table 2's 'Remain High' — the share of periods at or above the threshold once a run first reaches it — against the threshold. Measured (release, seeds 1–10, 2500 periods, recorded 2026-09-26): at 2.3, RWR 0.014, FRN 0.933, FFR-0.3 0.370 (the paper: 0.015, 0.942, 0.402) — and every other row of Table 2 lands within 0.03 of the paper at 2.3 too (the survey), so 2.3 is the default. At 2.5 FRN would hold only 0.46, at 2.0 FFR-0.3 would hold 0.66.",
  "base": {
    "preset": "cra-frn"
  },
  "x": {
    "label": "High cooperation at",
    "path": "high",
    "values": [
      2.0,
      2.1,
      2.2,
      2.3,
      2.4,
      2.5,
      2.6
    ]
  },
  "series": {
    "label": "Structure",
    "values": [
      {
        "at": 0,
        "name": "RWR",
        "set": {
          "structure": "rwr"
        }
      },
      {
        "at": 1,
        "name": "FRN",
        "set": {
          "structure": "frn"
        }
      },
      {
        "at": 2,
        "name": "FFR 0.3",
        "set": {
          "structure": "frn",
          "substitution": 0.3
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2500,
  "metric": {
    "kind": "final",
    "series": "share_high_since"
  }
}
```

Create `sweeps/cra-noise.json` with exactly this content:

```json
{
  "name": "Social Structure: the paper's two readings of its method",
  "description": "The paper describes its own method twice. Its start: 'evenly distributing the agents throughout the strategy space' (Appendix) or 'initialized randomly' (§3.1). Its copying noise: on every agent 'regardless of which … is adopted' (Appendix) or as 'errors in the actual copying process' (§2). Measured (release, seeds 1–10, 2500 periods, recorded 2026-09-26): the start makes no difference (RWR 1.089 either way; FRN 2.479 and 2.470); noise only on copying raises every fixed structure's payoff (2DK 2.554 → 2.651, FRNE 2.573 → 2.652, FRN 2.479 → 2.529) and moves it away from Table 2 — the Appendix's reading is the one the table reflects.",
  "base": {
    "preset": "cra-frn"
  },
  "x": {
    "label": "Social structure",
    "values": [
      {
        "at": 0,
        "name": "RWR",
        "set": {
          "structure": "rwr",
          "substitution": 0.0
        }
      },
      {
        "at": 1,
        "name": "2DK",
        "set": {
          "structure": "torus",
          "substitution": 0.0
        }
      },
      {
        "at": 2,
        "name": "FRNE",
        "set": {
          "structure": "frne",
          "substitution": 0.0
        }
      },
      {
        "at": 3,
        "name": "FRN",
        "set": {
          "structure": "frn",
          "substitution": 0.0
        }
      }
    ]
  },
  "series": {
    "label": "Reading",
    "values": [
      {
        "at": 0,
        "name": "Noise on everyone, even start (Appendix)",
        "set": {
          "noise_on": "always",
          "start": "grid"
        }
      },
      {
        "at": 1,
        "name": "Noise only on copying",
        "set": {
          "noise_on": "copy",
          "start": "grid"
        }
      },
      {
        "at": 2,
        "name": "Random start",
        "set": {
          "noise_on": "always",
          "start": "random"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 2500,
  "metric": {
    "kind": "window_mean",
    "series": "mean_payoff",
    "from": 1501
  }
}
```

Create `sweeps/cra-population.json` with exactly this content:

```json
{
  "name": "Social Structure: population size (note 1)",
  "description": "Note 1: 'our simulation runs for populations up to 4096 agents display very similar aggregate statistics'. Measured (release, seeds 1–5, 2500 periods, recorded 2026-09-26): RWR 1.23, 1.18, 1.10, 1.08, 1.08 and FRN 2.07, 2.45, 2.47, 2.49, 2.50 at 64, 144, 256, 576, 1024 agents — alike from 256 up; small populations differ (64 agents: RWR higher, FRN lower). The survey checks 4096.",
  "base": {
    "preset": "cra-frn"
  },
  "x": {
    "label": "Agents (n)",
    "path": "agents",
    "values": [
      64,
      144,
      256,
      576,
      1024
    ]
  },
  "series": {
    "label": "Structure",
    "values": [
      {
        "at": 0,
        "name": "RWR",
        "set": {
          "structure": "rwr"
        }
      },
      {
        "at": 1,
        "name": "FRN",
        "set": {
          "structure": "frn"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 5
  },
  "ticks": 2500,
  "metric": {
    "kind": "window_mean",
    "series": "mean_payoff",
    "from": 1501
  }
}
```

Apply to `crates/sugarscape-core/src/sweep.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index a8660c0..d01f110 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -959,7 +959,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 45] = [
+const BUILTINS: [Builtin; 50] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1140,6 +1140,26 @@ const BUILTINS: [Builtin; 45] = [
         id: "hk-population",
         json: include_str!("../../../sweeps/hk-population.json"),
     },
+    Builtin {
+        id: "cra-table-2",
+        json: include_str!("../../../sweeps/cra-table-2.json"),
+    },
+    Builtin {
+        id: "cra-dial",
+        json: include_str!("../../../sweeps/cra-dial.json"),
+    },
+    Builtin {
+        id: "cra-threshold",
+        json: include_str!("../../../sweeps/cra-threshold.json"),
+    },
+    Builtin {
+        id: "cra-noise",
+        json: include_str!("../../../sweeps/cra-noise.json"),
+    },
+    Builtin {
+        id: "cra-population",
+        json: include_str!("../../../sweeps/cra-population.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2007,7 +2027,12 @@ mod tests {
                 "hk-bias",
                 "hk-updating",
                 "hk-lattice",
-                "hk-population"
+                "hk-population",
+                "cra-table-2",
+                "cra-dial",
+                "cra-threshold",
+                "cra-noise",
+                "cra-population"
             ]
         );
         for b in builtins() {
```

Run: `cargo test -p sugarscape-core --lib sweep::` — Expected: PASS.

- [ ] **Step 2: Measure natively**

Run: `cargo build --release -p sugarscape-cli && for s in cra-table-2 cra-dial cra-threshold cra-noise cra-population; do /usr/bin/time -p target/release/sugarscape sweep --builtin $s --quiet --out /dev/null --summary-csv /tmp/$s.csv; done`
Expected: each under about 10 s; means as the descriptions state.

- [ ] **Step 3: The CLI names the stop (test first)**

Apply to `crates/sugarscape-cli/tests/cli.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index a0b8a1e..f7668b2 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -93,6 +93,11 @@ fn presets_and_sweeps_are_listed() {
         "hk-updating",
         "hk-lattice",
         "hk-population",
+        "cra-table-2",
+        "cra-dial",
+        "cra-threshold",
+        "cra-noise",
+        "cra-population",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -447,6 +452,13 @@ fn a_classes_run_stops_at_equity() {
     );
 }
 
+#[test]
+fn a_social_structure_run_stops_at_its_last_period() {
+    let out = sugarscape(&["run", "--preset", "cra-rwr", "--ticks", "3000"]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    assert_eq!(stderr(&out), "finished at tick 2500 (its last period)\n");
+}
+
 #[test]
 fn a_bounded_confidence_run_stops_when_stable() {
     let out = sugarscape(&["run", "--preset", "hk-regular-50", "--ticks", "1000"]);
```

Run: `cargo test -p sugarscape-cli a_social_structure_run` — Expected: FAIL (`(its end year)`).

Apply to `crates/sugarscape-cli/src/main.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index 525e06a..024a54d 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -189,6 +189,7 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             ModelKind::Culture => "the lattice is stable",
             ModelKind::Classes => "equity reached",
             ModelKind::Opinions => "stable",
+            ModelKind::Structure => "its last period",
             ModelKind::Sugarscape => "the cultures have settled",
             ModelKind::Ethno => "its last period",
             _ => "its end year",
```

Run: `cargo test -p sugarscape-cli` — Expected: PASS.

- [ ] **Step 4: WASM agrees**

Apply to `crates/sugarscape-wasm/tests/web.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index f25852c..240f23f 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -295,7 +295,12 @@ fn builtins_and_series_names_are_listed() {
             "hk-bias",
             "hk-updating",
             "hk-lattice",
-            "hk-population"
+            "hk-population",
+            "cra-table-2",
+            "cra-dial",
+            "cra-threshold",
+            "cra-noise",
+            "cra-population"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -724,6 +729,21 @@ fn classes_sims_match_the_native_golden_entries() {
     }
 }
 
+#[wasm_bindgen_test]
+fn structure_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
+    for (id, fp) in [
+        ("cra-rwr", "0xc7f45f59d9b25490"),
+        ("cra-2dk", "0x3b8c19aab4aae805"),
+        ("cra-frne", "0xdf2fc96965742a81"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "structure");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
 #[wasm_bindgen_test]
 fn opinions_sims_match_the_native_golden_entries() {
     // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
```

Run: `wasm-pack test --node crates/sugarscape-wasm` — Expected: PASS (39), including `structure_sims_match_the_native_golden_entries`.

- [ ] **Step 5: Commit**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace`

```bash
git add sweeps/cra-table-2.json sweeps/cra-dial.json sweeps/cra-threshold.json sweeps/cra-noise.json sweeps/cra-population.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/src/main.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure Social Structure: five sweeps, the CLI's stop and WASM agreement" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/ui/series-data.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/compare-presets.test.ts`, `web/src/experiments/form.test.ts`, `web/src/engine.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the core's JSON (`StructureConfig`, `StructureSnapshot`, `StructureInspection` with `block`, `plane`, `agents`, `agent`).
- Produces: `StructureConfig`, `StructureStats`, `StructureInspection`, `StructureAgentView`, `PartnerView` (types.ts); `isStructureView` (models.ts); the modes, charts, Inspect rows, Compare entry and Experiments default.

- [ ] **Step 1: The failing tests**

Apply to `web/src/models.test.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index a2aff6f..1d7a3b9 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -6,6 +6,7 @@ import {
   isCivilView,
   isClassesView,
   isOpinionsView,
+  isStructureView,
   isCultureView,
   isEthnoView,
   isRingView,
@@ -101,6 +102,29 @@ describe('the anasazi model', () => {
   });
 });
 
+describe('the social-structure model', () => {
+  it('is read by its tag, and its inspections by their block cell and plane point', () => {
+    const c = { model: 'structure', stop_at: 2500 } as unknown as ModelConfig;
+    expect(modelOf(c)).toBe('structure');
+    const cell = { site: { x: 1, y: 2 }, block: { x: 0, y: 0 }, plane: null, agents: [], agent: null } as unknown as AnyInspection;
+    const line = { site: { x: 1, y: 2 }, period: 3, opinion: 0.5, lattice_site: null, agents: [], agent: null } as unknown as AnyInspection;
+    expect([cell, line].map(isStructureView)).toEqual([true, false]);
+    expect(isOpinionsView(cell) || isClassesView(cell) || isCultureView(cell) || isTagsView(cell) || isSugarView(cell)).toBe(false);
+  });
+
+  it('colors agents four ways, has no overlays, and stops predictably at its last period', () => {
+    expect(COLOR_MODES.structure).toEqual([
+      ['friendliness', 'Friendliness'],
+      ['provocability', 'Provocability'],
+      ['payoff', 'Payoff'],
+      ['strategy', 'Strategy'],
+    ]);
+    expect(MODEL_OVERLAYS.structure).toEqual([]);
+    const c = (stop_at: number) => ({ model: 'structure', stop_at }) as unknown as ModelConfig;
+    expect([finishesUnpredictably(c(2500)), ticksLeft(c(2500), 500), ticksLeft(c(0), 500)]).toEqual([false, 2000, Infinity]);
+  });
+});
+
 describe('the bounded-confidence model', () => {
   it('is read by its tag, and its inspections by their period and lattice site', () => {
     const c = { model: 'opinions', stop_when_stable: true } as unknown as ModelConfig;
```

Apply to `web/src/ui/series-data.test.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index 8228a4f..d543a4a 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,13 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('social-structure charts', () => {
+  it('chart payoff, cooperation, strategy, high cooperation and copying over periods', () => {
+    expect(MODEL_CHARTS.structure.map((c) => c.title)).toEqual(['Mean payoff', 'Cooperation', 'Strategy', 'High cooperation', 'Copying']);
+    expect(timeAxisLabel('structure')).toBe('Periods');
+  });
+});
+
 describe('bounded-confidence charts', () => {
   it('chart clusters, camps, the center, splits and change over periods', () => {
     expect(MODEL_CHARTS.opinions.map((c) => c.title)).toEqual(['Clusters', 'Largest camps', 'Mean and median', 'Splits', 'Change']);
```

Apply to `web/src/compare-presets.test.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index 9b392d2..64e3dc8 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -42,6 +42,11 @@ describe('compare presets', () => {
     expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
   });
 
+  it('pairs random mixing and fixed random neighbors', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['cra-rwr-vs-frn', 'cra-rwr', 'cra-frn', 'Random mixing vs fixed random neighbors — Social Structure (Compare)']);
+  });
+
   it('pairs simultaneous and serial updating', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['hk-simultaneous-vs-serial', 'hk-polarisation', 'hk-serial', 'Simultaneous vs serial updating — Bounded Confidence (Compare)']);
```

Apply to `web/src/experiments/form.test.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index 817fee0..0bd6d76 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -147,6 +147,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('structure')).toMatchObject({
+      x: { path: 'substitution', values: '0:1:0.1' },
+      ticks: 2500,
+      metric: { kind: 'window_mean', series: 'mean_payoff', from: 1501 },
+    });
     expect(defaultForm('opinions')).toMatchObject({
       x: { path: 'epsilon', values: '0.05:0.3:0.05' },
       ticks: 1000,
```

Apply to `web/src/engine.test.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index 93cdc7e..0d2133a 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1150,6 +1150,7 @@ describe('Engine with other models', () => {
     expect(finishedNotice(e.config, 10)).toBe('This run has reached its end year (AD 810) — Reset to run it again');
     expect(finishedNotice(ring, 10)).toBe('This run has reached its end year — Reset to run it again');
     expect(finishedNotice({ model: 'civil' } as unknown as ModelConfig, 94)).toBe('A group has died out at t = 94 — Reset to run it again');
+    expect(finishedNotice({ model: 'structure' } as unknown as ModelConfig, 2500)).toBe('This run has reached its last period (2500) — Reset to run it again');
     expect(finishedNotice({ model: 'opinions' } as unknown as ModelConfig, 8)).toBe(
       'Stable at t = 8: no opinion moves any more — Reset, or change confidence or updating, to run it again',
     );
```

Apply to `web/src/determinism.test.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index dfe2927..6a043f7 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -26,6 +26,9 @@ import type {
   OpinionsConfig,
   OpinionsInspection,
   OpinionsStats,
+  StructureConfig,
+  StructureInspection,
+  StructureStats,
   Param,
   Preset,
   Snapshot,
@@ -431,6 +434,15 @@ describe('other models through the engine', () => {
     ['jansson-kin-fixed', '0x3fac090571612879'],
     ['hks-no-ethnocentrics', '0xbe867e7210bad2d2'],
     ['hk-lattice', '0xe33359f120b204d9'],
+    ['cra-rwr', '0xc7f45f59d9b25490'],
+    ['cra-2dk', '0x3b8c19aab4aae805'],
+    ['cra-frne', '0xdf2fc96965742a81'],
+    ['cra-frn', '0x924d4b2fe686ae18'],
+    ['cra-ffr-01', '0x424eda2182150e01'],
+    ['cra-ffr-03', '0xbc09206184dc7003'],
+    ['cra-ffr-05', '0xf9573021f9848025'],
+    ['cra-random-start', '0x1871afd34df774a9'],
+    ['cra-copy-noise', '0xd8871da3505ee758'],
   ];
 
   it.each(GOLDEN_MODELS)('%s reproduces its golden fingerprint, whatever is watched', async (id, golden) => {
@@ -512,6 +524,25 @@ describe('the anasazi through the engine', () => {
   });
 });
 
+describe('the social-structure model through the engine', () => {
+  it('stops at its last period and inspects an agent and its partners', async () => {
+    const r = presets.find((p) => p.id === 'cra-2dk')!;
+    const config = { ...structuredClone(r.config as StructureConfig), stop_at: 30 };
+    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'strategy' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as StructureStats;
+    expect([e.finished, ends, e.tick, s.tick]).toEqual([true, 1, 30, 30]);
+    // The torus: every agent played its four neighbors twice.
+    await e.select(1, 1);
+    const v = e.inspection!.view as StructureInspection;
+    expect(v.block).toEqual({ x: 0, y: 0 });
+    expect(v.agent!.partners.map((p) => p.games)).toEqual([2, 2, 2, 2]);
+  });
+});
+
 describe('the bounded-confidence model through the engine', () => {
   it('stops once when stable, matches the native golden entry and inspects a line', async () => {
     const r = presets.find((p) => p.id === 'hk-regular-50')!;
```

Run: `(cd web && npm run build)` — Expected: FAIL in `tsc` (`'structure'` is not a `ModelKind`; `isStructureView`, `StructureStats` missing).

- [ ] **Step 2: Implement**

Apply to `web/src/types.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/types.ts b/web/src/types.ts
index 60c1548..9e01764 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -81,7 +81,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -314,7 +314,28 @@ export interface OpinionsConfig {
   stop_when_stable: boolean;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig;
+/**
+ * Cohen, Riolo and Axelrod's population of adaptive agents playing short iterated Prisoner's
+ * Dilemmas under a social structure (milestone 18), with the paper's two readings of its method as
+ * switches.
+ */
+export interface StructureConfig {
+  model: 'structure';
+  agents: number;
+  structure: 'rwr' | 'torus' | 'frne' | 'frn';
+  substitution: number;
+  partners: number;
+  moves: number;
+  judge_error: number;
+  mutation: number;
+  mutation_sd: number;
+  noise_on: 'always' | 'copy';
+  start: 'grid' | 'random';
+  high: number;
+  stop_at: number;
+}
+
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig;
 
 export interface Preset { id: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -521,7 +542,22 @@ export interface OpinionsStats {
   stable_at: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats;
+export interface StructureStats {
+  tick: number;
+  mean_payoff: number;
+  cooperation: number;
+  mean_y: number;
+  mean_p: number;
+  mean_q: number;
+  high: number;
+  /** The first period at or above the threshold, else −1. */
+  attained_high: number;
+  share_high_since: number;
+  copied: number;
+  partner_p_slope: number;
+}
+
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -727,7 +763,29 @@ export interface OpinionsInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection;
+/** An agent played this period: its p as played, its score and how many games. */
+export interface PartnerView { id: number; p: number; score: number; games: number }
+/** An agent: its strategy, class, this period's score, whom it copied and (in the block) its partners. */
+export interface StructureAgentView {
+  id: number;
+  y: number;
+  p: number;
+  q: number;
+  class: 'tft' | 'alld' | 'allc' | 'other';
+  score: number;
+  copied: number | null;
+  partners: PartnerView[];
+}
+/** A cell of the agents' block (its agent) or of the p–q plane (its (p, q) and the agents there). */
+export interface StructureInspection {
+  site: { x: number; y: number };
+  block: { x: number; y: number } | null;
+  plane: [number, number] | null;
+  agents: StructureAgentView[];
+  agent: StructureAgentView | null;
+}
+
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -766,7 +824,10 @@ export type ColorMode =
   | 'best_reply'
   | 'payoff'
   | 'start'
-  | 'opinion';
+  | 'opinion'
+  | 'friendliness'
+  | 'provocability'
+  | 'strategy';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
```

Apply to `web/src/models.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/models.ts b/web/src/models.ts
index 60598da..e1f2993 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -11,6 +11,8 @@ import type {
   CultureInspection,
   OpinionsConfig,
   OpinionsInspection,
+  StructureConfig,
+  StructureInspection,
   ColorMode,
   Config,
   EthnoConfig,
@@ -25,7 +27,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -40,12 +42,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   classes: 'Emergence of Classes',
   ethno: 'Ethnocentrism',
   opinions: 'Bounded Confidence',
+  structure: 'Social Structure',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure'
     ? tag
     : 'sugarscape';
 }
@@ -90,6 +93,11 @@ export function isClassesView(v: AnyInspection): v is ClassesInspection {
 }
 
 /** A cell of the bounded-confidence frame (it names its period and lattice site). */
+/** A cell of the social-structure frame (it names its block cell and plane point). */
+export function isStructureView(v: AnyInspection): v is StructureInspection {
+  return 'block' in v && 'plane' in v;
+}
+
 export function isOpinionsView(v: AnyInspection): v is OpinionsInspection {
   return 'period' in v && 'lattice_site' in v;
 }
@@ -121,6 +129,7 @@ export function ticksLeft(c: ModelConfig, tick: number): number {
   if ('model' in c && c.model === 'anasazi') return Math.max(0, c.end_year - c.start_year - tick);
   if (modelOf(c) === 'tags' && (c as TagsConfig).end > 0) return Math.max(0, (c as TagsConfig).end - tick);
   if (modelOf(c) === 'ethno' && (c as EthnoConfig).end > 0) return Math.max(0, (c as EthnoConfig).end - tick);
+  if (modelOf(c) === 'structure' && (c as StructureConfig).stop_at > 0) return Math.max(0, (c as StructureConfig).stop_at - tick);
   return Infinity;
 }
 
@@ -217,6 +226,13 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['start', 'Start'],
     ['opinion', 'Opinion'],
   ],
+  // Friendliness first (the paper's p); provocability is 1 − q.
+  structure: [
+    ['friendliness', 'Friendliness'],
+    ['provocability', 'Provocability'],
+    ['payoff', 'Payoff'],
+    ['strategy', 'Strategy'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -232,4 +248,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   classes: [],
   ethno: [],
   opinions: [],
+  structure: [],
 };
```

Apply to `web/src/engine.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/engine.ts b/web/src/engine.ts
index 9290a1e..6f23dbd 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -58,7 +58,7 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
     return `Stable at t = ${tick}: no opinion moves any more — Reset, or change confidence or updating, to run it again`;
   if (modelOf(config) === 'culture') return `The lattice is stable at t = ${tick}: no two neighbors can interact — Reset to run it again`;
   if (modelOf(config) === 'sugarscape') return `The cultures have settled at t = ${tick}: every two share all or nothing — Reset to run it again`;
-  if (modelOf(config) === 'ethno') return `This run has reached its last period (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'ethno' || modelOf(config) === 'structure') return `This run has reached its last period (${tick}) — Reset to run it again`;
   const year = calendarYear(config, tick);
   return `This run has reached its end year${year === null ? '' : ` (AD ${year})`} — Reset to run it again`;
 }
```

Apply to `web/src/ui/series-data.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index 8173e50..455442b 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -332,11 +332,39 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
     },
     { title: 'Change', lines: [{ key: 'max_change', label: 'Largest move this period', color: '--c4' }] },
   ],
+  structure: [
+    { title: 'Mean payoff', lines: [{ key: 'mean_payoff', label: 'Per move', color: '--c1' }], range: [0, 5] },
+    { title: 'Cooperation', lines: [{ key: 'cooperation', label: 'Share of moves', color: '--c2' }], range: [0, 1] },
+    {
+      title: 'Strategy',
+      lines: [
+        { key: 'mean_p', label: 'Friendliness (p)', color: '--c2' },
+        { key: 'mean_q', label: 'Forgiveness (q)', color: '--c3' },
+        { key: 'mean_y', label: 'First move (y)', color: '--muted' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'High cooperation',
+      lines: [
+        { key: 'high', label: 'At the threshold', color: '--c1' },
+        { key: 'share_high_since', label: 'Share since first reached', color: '--c4' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'Copying',
+      lines: [
+        { key: 'copied', label: 'Agents copying', color: '--c3' },
+        { key: 'partner_p_slope', label: 'Partners’ p on own p (slope)', color: '--c4' },
+      ],
+    },
+  ],
 };
 
 /** A model's time charts count calendar years (the anasazi's), generations (tags), periods (ethnocentrism, HA06's word) or ticks. */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' ? 'Periods' : model === 'ethno' ? 'Period' : 'Tick';
+  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' ? 'Periods' : model === 'ethno' ? 'Period' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
```

Apply to `web/src/compare-presets.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index 1c8f039..062d202 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -86,6 +86,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'hk-polarisation',
     b: 'hk-serial',
   },
+  {
+    id: 'cra-rwr-vs-frn',
+    label: 'Random mixing vs fixed random neighbors — Social Structure (Compare)',
+    a: 'cra-rwr',
+    b: 'cra-frn',
+  },
 ];
 
 /**
```

Apply to `web/src/experiments/form.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index af9541f..737a04e 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -47,6 +47,10 @@ export function defaultForm(model: ModelKind = 'sugarscape'): SweepForm {
       metric: { ...form.metric, kind: 'final', series: 'fit' },
     };
   }
+  if (model === 'structure') {
+    // The paper's dial (the built-in cra-dial): Table 2's mean payoff against random substitution.
+    return { ...form, x: { path: 'substitution', values: '0:1:0.1' }, ticks: 2500, metric: { ...form.metric, kind: 'window_mean', series: 'mean_payoff', from: 1501, to: null } };
+  }
   if (model === 'opinions') {
     // Fig. 3's axis (the built-in hk-diagonal): surviving opinions against confidence.
     return { ...form, x: { path: 'epsilon', values: '0.05:0.3:0.05' }, ticks: 1000, metric: { ...form.metric, kind: 'final', series: 'clusters' } };
```

Apply to `web/src/ui/inspect-panel.ts` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index 0176088..b6df2d6 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -1,7 +1,7 @@
 import { citizenRows, shownCitizen } from '../civil';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
-import { isCivilView, isClassesView, isCultureView, isEthnoView, isOpinionsView, isRingView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isCivilView, isClassesView, isCultureView, isEthnoView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
@@ -9,6 +9,7 @@ import type {
   CivilInspection,
   ClassesInspection,
   OpinionsInspection,
+  StructureInspection,
   CultureInspection,
   CultureSiteView,
   EthnoConfig,
@@ -168,6 +169,27 @@ export class InspectPanel {
   }
 
   /** A cell of the opinion × time diagram (its period, opinion and the agents passing) or of the lattice. */
+  private structureRows(view: StructureInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    const names = { tft: 'near Tit-for-Tat', alld: 'near Always Defect', allc: 'near Always Cooperate', other: 'mixed' };
+    const strategy = (a: { y: number; p: number; q: number; class: keyof typeof names }) =>
+      `y ${fmt(a.y)} · p ${fmt(a.p)} · q ${fmt(a.q)} (${names[a.class]})`;
+    if (view.agent) {
+      const a = view.agent;
+      const rows = [row('Agent', `#${a.id}`), row('Strategy', strategy(a)), row('Payoff per move', fmt(a.score)), row('Copied', a.copied === null ? 'no one' : `#${a.copied}`)];
+      for (const p of a.partners) rows.push(row(`Played #${p.id}`, `${p.games}× · p ${fmt(p.p)} · payoff ${fmt(p.score)}`));
+      return rows;
+    }
+    if (view.plane) {
+      const rows = [row('Point', `p ${fmt(view.plane[0])} · q ${fmt(view.plane[1])}`)];
+      if (view.agents.length === 0) return [...rows, row('Agents', 'none here')];
+      for (const a of view.agents.slice(0, 12)) rows.push(row(`#${a.id}`, `${strategy(a)} · payoff ${fmt(a.score)}`));
+      if (view.agents.length > 12) rows.push(row('', `and ${view.agents.length - 12} more`));
+      return rows;
+    }
+    return [row('Point', 'between the agents and the plane')];
+  }
+
   private opinionsRows(view: OpinionsInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
     const rows: HTMLElement[] = [];
@@ -311,6 +333,8 @@ export class InspectPanel {
       // First: an empty ethnocentrism site is shaped like an empty Schelling site.
       const rows = isEthnoView(view, this.engine.model)
         ? this.ethnoSiteRows(view, gone)
+        : isStructureView(view)
+          ? this.structureRows(view)
         : isOpinionsView(view)
           ? this.opinionsRows(view)
         : isClassesView(view)
```

- [ ] **Step 3: Build and test**

Run: `(cd web && npm run build && npm test)` — Expected: 574 pass (44 files).

- [ ] **Step 4: Commit**

```bash
git add web/src/types.ts web/src/models.ts web/src/engine.ts web/src/ui/series-data.ts web/src/compare-presets.ts web/src/experiments/form.ts web/src/ui/inspect-panel.ts web/src/models.test.ts web/src/ui/series-data.test.ts web/src/compare-presets.test.ts web/src/experiments/form.test.ts web/src/engine.test.ts web/src/determinism.test.ts
git commit -m "Carry Social Structure through the page" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

**Browser (controller):**
1. Presets menu: a **Social Structure** group (9) and the Compare entry.
2. `cra-2dk` after a few hundred periods: the block (the torus) mostly green under **Friendliness**; the plane's trail settled in the high-p, low-q corner (bottom right). `cra-rwr`: a red block, the trail at low p.
3. **Provocability**, **Payoff**, **Strategy** recolor the block.
4. Inspect a block cell: strategy, class, payoff, whom it copied, its partners (games, p, payoff); Follow tracks it; a plane point lists the agents there; the gap says so.
5. Rules panel groups Population, Structure, Adaptation, Measures; substitution and the threshold apply live; Structure asks for Reset.
6. The run stops at 2500 with the notice; the timeline steps back and the plane's trail with it; the Compare pair runs.
7. Experiments: `cra-dial` runs; "From current world" starts at `substitution`, `0:1:0.1`, window mean `mean_payoff` from 1501.
8. Recording works; every other model's preset renders.

---

### Task 4: The survey's social-structure claims

**Files:**
- Create: `survey/src/claims/structure.rs`
- Modify: `survey/src/claims/mod.rs`

- [ ] **Step 1: The claims**

Create `survey/src/claims/structure.rs` with exactly this content:

```rust
//! Cohen, Riolo & Axelrod's social structure (milestone 18): Table 2, Fig. 1,
//! the crucial p–q region and Figs. 5–6, notes 1 and 5, Table A1, and the
//! paper's two readings of its own method. Runs use the paper's 30 seeds and
//! 2500 periods and are memoized per process, keyed by the config.

use std::sync::{Arc, Mutex};

use sugarscape_core::structure::{
    fanout, regression, NoiseOn, Start, Structure, StructureConfig, StructureWorld,
};

use crate::claim::{greater, Claim, Outcome, Source, Verdict};
use crate::stats::mean;

const CRA: &str = "Cohen, Riolo & Axelrod 2001, Rationality and Society 13(1)";
const PERIODS: u32 = 2500;
/// The paper's replications per case.
const SEEDS: u64 = 30;
/// The crucial region of population means (§3.3).
const P_RANGE: (f64, f64) = (0.30, 0.35);
const Q_RANGE: (f64, f64) = (0.05, 0.10);

/// One run, summarized.
#[derive(Clone, Debug, Default)]
struct Run {
    /// Mean payoff per move in every period (index 0: the start, no games).
    payoff: Vec<f64>,
    /// In the crucial region: the change of mean p in each visit, and the
    /// (own p, partners' mean p) pairs of those periods.
    deltas: Vec<f64>,
    pairs: Vec<(f64, f64)>,
}

impl Run {
    /// Table 2's mean payoff: periods 1501–2500.
    fn late_mean(&self) -> f64 {
        mean(&self.payoff[1501..=2500])
    }

    /// The first period at or above `high`, if any.
    fn attained(&self, high: f64) -> Option<usize> {
        (1..self.payoff.len()).find(|&t| self.payoff[t] >= high)
    }

    /// Table 2's "Remain High": the share of periods at or above `high` from
    /// the first one on.
    fn remain(&self, high: f64) -> Option<f64> {
        let t = self.attained(high)?;
        let since = &self.payoff[t..];
        Some(since.iter().filter(|&&v| v >= high).count() as f64 / since.len() as f64)
    }

    /// Stretches of at least `len` periods both above and below `high`.
    fn bistable(&self, high: f64, len: usize) -> bool {
        let (mut up, mut down, mut run, mut cur) = (false, false, 0, self.payoff[1] >= high);
        for &v in &self.payoff[1..] {
            let h = v >= high;
            if h == cur {
                run += 1;
            } else {
                run = 1;
                cur = h;
            }
            if run >= len {
                if cur {
                    up = true;
                } else {
                    down = true;
                }
            }
        }
        up && down
    }
}

fn simulate(c: &StructureConfig, seed: u64) -> Run {
    let mut w = StructureWorld::new(c.clone(), seed).expect("a valid config");
    let mut run = Run {
        payoff: vec![0.0],
        ..Run::default()
    };
    for _ in 0..PERIODS {
        let before = w.stats.latest().expect("a period").clone();
        w.step();
        let after = w.stats.latest().expect("a period");
        run.payoff.push(after.mean_payoff);
        if (P_RANGE.0..=P_RANGE.1).contains(&before.mean_p)
            && (Q_RANGE.0..=Q_RANGE.1).contains(&before.mean_q)
        {
            run.deltas.push(after.mean_p - before.mean_p);
            run.pairs.extend(w.partner_p_pairs());
        }
    }
    run
}

/// The paper's defaults (256 agents, grid start, noise on everyone, no stop)
/// with `edit`, over `seeds` seeds.
fn runs(seeds: u64, edit: impl FnOnce(&mut StructureConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u64, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = StructureConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, _)| *k == key && *s == seeds)
    {
        return v.clone();
    }
    let v: Vec<Run> = std::thread::scope(|scope| {
        let handles: Vec<_> = (1..=seeds)
            .map(|seed| {
                let c = &c;
                scope.spawn(move || simulate(c, seed))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("a run"))
            .collect()
    });
    let v = Arc::new(v);
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds, v.clone()));
    v
}

/// Table 2's rows: (name, structure, substitution, attain, mean, remain).
const TABLE_2: [(&str, Structure, f64, f64, f64, f64); 7] = [
    ("RWR", Structure::Rwr, 0.0, 0.30, 1.091, 0.015),
    ("2DK", Structure::Torus, 0.0, 1.00, 2.557, 0.997),
    ("FRNE", Structure::Frne, 0.0, 1.00, 2.575, 0.995),
    ("FRN", Structure::Frn, 0.0, 1.00, 2.480, 0.942),
    ("FFR 0.1", Structure::Frn, 0.1, 1.00, 2.385, 0.844),
    ("FFR 0.3", Structure::Frn, 0.3, 1.00, 2.100, 0.402),
    ("FFR 0.5", Structure::Frn, 0.5, 0.93, 1.257, 0.061),
];

fn row(i: usize) -> Arc<Vec<Run>> {
    let (_, structure, x, ..) = TABLE_2[i];
    runs(SEEDS, move |c| {
        c.structure = structure;
        c.substitution = x;
    })
}

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

/// The mean over runs of each run's late mean payoff.
fn late(runs: &[Run]) -> f64 {
    mean(&runs.iter().map(Run::late_mean).collect::<Vec<_>>())
}

/// Remain High over the runs that attained high (NaN if none did).
fn remain(runs: &[Run], high: f64) -> f64 {
    mean(
        &runs
            .iter()
            .filter_map(|r| r.remain(high))
            .collect::<Vec<_>>(),
    )
}

/// The mean payoff of row `i` within 0.1 of Table 2.
fn mean_claim(i: usize) -> Outcome {
    let (name, _, _, _, paper, _) = TABLE_2[i];
    let m = late(&row(i));
    outcome(
        (m - paper).abs() <= 0.1,
        format!("{name}: {m:.3} over periods 1501–2500 (the paper {paper:.3}; within 0.1)"),
    )
}

/// Remain High of row `i` at the 2.3 threshold, within 0.05 of Table 2.
fn remain_claim(i: usize) -> Outcome {
    let (name, _, _, _, _, paper) = TABLE_2[i];
    let r = remain(&row(i), 2.3);
    outcome(
        (r - paper).abs() <= 0.05,
        format!("{name}: {r:.3} at the 2.3 threshold (the paper {paper:.3}; within 0.05)"),
    )
}

pub fn claims() -> Vec<Claim> {
    macro_rules! table_rows {
        ($($i:literal => $mean_id:literal, $remain_id:literal, $item:literal;)*) => {
            vec![$(
                Claim {
                    id: $mean_id,
                    item: $item,
                    source: Source::Book,
                    citation: CRA,
                    text: "Table 2: the mean payoff per move over the last 1000 of 2500 periods (30 runs; within 0.1)",
                    check: |_| mean_claim($i),
                },
                Claim {
                    id: $remain_id,
                    item: $item,
                    source: Source::Book,
                    citation: CRA,
                    text: "Table 2: 'Remain High', the share of time at high cooperation once reached (the unstated threshold read as 2.3; within 0.05)",
                    check: |_| remain_claim($i),
                },
            )*]
        };
    }
    let mut out = table_rows! {
        0 => "structure.table-2.rwr.mean", "structure.table-2.rwr.remain", "cra-rwr";
        1 => "structure.table-2.2dk.mean", "structure.table-2.2dk.remain", "cra-2dk";
        2 => "structure.table-2.frne.mean", "structure.table-2.frne.remain", "cra-frne";
        3 => "structure.table-2.frn.mean", "structure.table-2.frn.remain", "cra-frn";
        4 => "structure.table-2.ffr-01.mean", "structure.table-2.ffr-01.remain", "cra-ffr-01";
        5 => "structure.table-2.ffr-03.mean", "structure.table-2.ffr-03.remain", "cra-ffr-03";
        6 => "structure.table-2.ffr-05.mean", "structure.table-2.ffr-05.remain", "cra-ffr-05";
    };
    out.extend(vec![
        Claim {
            id: "structure.table-2.attain",
            item: "cra-table-2",
            source: Source::Book,
            citation: CRA,
            text: "Table 2: 'Attain High C', the share of runs ever reaching high cooperation (each row within two binomial standard deviations of the paper's, at least 0.07)",
            check: |_| {
                let mut ok = true;
                let mut parts = Vec::new();
                for (i, &(name, _, _, paper, _, _)) in TABLE_2.iter().enumerate() {
                    let rs = row(i);
                    let share = rs.iter().filter(|r| r.attained(2.3).is_some()).count() as f64 / rs.len() as f64;
                    let tol = (2.0 * (paper * (1.0 - paper) / rs.len() as f64).sqrt()).max(0.07);
                    ok &= (share - paper).abs() <= tol;
                    parts.push(format!("{name} {share:.2} ({paper:.2})"));
                }
                outcome(ok, parts.join(", "))
            },
        },
        Claim {
            id: "structure.table-2.threshold",
            item: "cra-threshold",
            source: Source::Book,
            citation: CRA,
            text: "the paper never states its 'high cooperation' threshold; 2.3 fits every row's Remain High better than 2.2 or 2.4 (largest miss over the seven rows)",
            check: |_| {
                let miss = |high: f64| {
                    (0..7)
                        .map(|i| (remain(&row(i), high) - TABLE_2[i].5).abs())
                        .fold(0.0, f64::max)
                };
                let (a, b, c) = (miss(2.2), miss(2.3), miss(2.4));
                outcome(
                    b < a && b < c,
                    format!("largest miss {a:.3} at 2.2, {b:.3} at 2.3, {c:.3} at 2.4"),
                )
            },
        },
        Claim {
            id: "structure.table-2.dial",
            item: "cra-dial",
            source: Source::Book,
            citation: CRA,
            text: "'Around a parameter value of 0.3 the dynamics shift … at levels of 0.5 and above, it collapses' (mean payoffs fall RWR < FFR-0.5 < FFR-0.3 < FFR-0.1 < FRN, and FFR-0.5 within 0.3 of RWR)",
            check: |_| {
                let m: Vec<f64> = [0, 6, 5, 4, 3].iter().map(|&i| late(&row(i))).collect();
                outcome(
                    m.windows(2).all(|w| w[0] < w[1]) && m[1] - m[0] <= 0.3,
                    format!("RWR {:.3}, FFR-0.5 {:.3}, FFR-0.3 {:.3}, FFR-0.1 {:.3}, FRN {:.3}", m[0], m[1], m[2], m[3], m[4]),
                )
            },
        },
        Claim {
            id: "structure.ffr-03.bistable",
            item: "cra-ffr-03",
            source: Source::Book,
            citation: CRA,
            text: "FFR-0.3 shows 'a bi-stable condition in which long stretches at high-p alternate with long stretches at low-p' (most runs spend 50 periods or more both high and low)",
            check: |_| {
                let rs = row(5);
                let k = rs.iter().filter(|r| r.bistable(2.3, 50)).count();
                outcome(2 * k > rs.len(), format!("{k}/{} runs", rs.len()))
            },
        },
        Claim {
            id: "structure.fig-1.first-period",
            item: "cra-table-2",
            source: Source::Book,
            citation: CRA,
            text: "Fig. 1: 'each of the four cells … is realized one quarter of the time, making average payoff 2.25' in the first period (RWR, 2DK, FRNE, FRN; 2.2–2.3)",
            check: |_| {
                let firsts: Vec<f64> = (0..4).map(|i| mean(&row(i).iter().map(|r| r.payoff[1]).collect::<Vec<_>>())).collect();
                outcome(
                    firsts.iter().all(|&f| (2.2..=2.3).contains(&f)),
                    format!("{:.3}, {:.3}, {:.3}, {:.3}", firsts[0], firsts[1], firsts[2], firsts[3]),
                )
            },
        },
        Claim {
            id: "structure.fig-1.recovery",
            item: "cra-table-2",
            source: Source::Book,
            citation: CRA,
            text: "Fig. 1: every structure collapses from the start; the context-preserving ones recover within 50 periods 'while the random pairing system never does' (period 50 above the dip by 0.4 for 2DK, FRNE, FRN; RWR within 0.1 of its dip)",
            check: |_| {
                let curve = |i: usize| -> Vec<f64> {
                    let rs = row(i);
                    (1..=50).map(|t| mean(&rs.iter().map(|r| r.payoff[t]).collect::<Vec<_>>())).collect()
                };
                let mut parts = Vec::new();
                let mut ok = true;
                for (i, name) in [(0, "RWR"), (1, "2DK"), (2, "FRNE"), (3, "FRN")] {
                    let c = curve(i);
                    let dip = c.iter().cloned().fold(f64::INFINITY, f64::min);
                    let rise = c[49] - dip;
                    ok &= if i == 0 { rise <= 0.1 } else { rise >= 0.4 };
                    parts.push(format!("{name}: dip {dip:.2}, period 50 {:.2}", c[49]));
                }
                outcome(ok, parts.join("; "))
            },
        },
        Claim {
            id: "structure.region.direction",
            item: "cra-frn",
            source: Source::Book,
            citation: CRA,
            text: "§3.3: with population p in [0.30, 0.35] and q in [0.05, 0.10], p moves by −0.016 under RWR and +0.052 under FRN (signs as stated; each within 0.02)",
            check: |_| {
                let d = |i: usize| {
                    let all: Vec<f64> = row(i).iter().flat_map(|r| r.deltas.clone()).collect();
                    (mean(&all), all.len())
                };
                let ((r, rn), (f, fn_)) = (d(0), d(3));
                outcome(
                    r < 0.0 && f > 0.0 && (r + 0.016).abs() <= 0.02 && (f - 0.052).abs() <= 0.02,
                    format!("RWR {r:+.4} over {rn} visits, FRN {f:+.4} over {fn_} visits"),
                )
            },
        },
        Claim {
            id: "structure.fig-5-6.slope",
            item: "cra-frn",
            source: Source::Book,
            citation: CRA,
            text: "Figs. 5–6: in that region an agent's partners' mean p regressed on its own p is 'not significant for RWR' and 'highly significant for FRN (slope = 0.1580; F = 717.20; N = 5376)' (FRN slope within 0.05, F > 100; RWR F below 3.84)",
            check: |_| {
                let pooled = |i: usize| {
                    let pairs: Vec<(f64, f64)> = row(i).iter().flat_map(|r| r.pairs.clone()).collect();
                    let (s, f) = regression(&pairs);
                    (s, f, pairs.len())
                };
                let ((rs, rf, rn), (fs, ff, fn_)) = (pooled(0), pooled(3));
                outcome(
                    (fs - 0.158).abs() <= 0.05 && ff > 100.0 && rf < 3.84,
                    format!("FRN slope {fs:.3}, F {ff:.0}, N {fn_}; RWR slope {rs:.3}, F {rf:.1}, N {rn}"),
                )
            },
        },
        Claim {
            id: "structure.note-5",
            item: "cra-frne",
            source: Source::Book,
            citation: CRA,
            text: "note 5: 'the FRNE populations actually have a better average score than the 2DK populations'",
            check: |_| {
                let per = |i: usize| row(i).iter().map(Run::late_mean).collect::<Vec<_>>();
                greater(&per(2), &per(1), "FRNE", "2DK")
            },
        },
        Claim {
            id: "structure.note-1",
            item: "cra-population",
            source: Source::Book,
            citation: CRA,
            text: "note 1: 'populations up to 4096 agents display very similar aggregate statistics' (RWR and FRN mean payoffs within 0.05 of 256 agents'; 8 runs at 4096)",
            check: |_| {
                let big = |s: Structure| {
                    late(&runs(8, move |c| {
                        c.structure = s;
                        c.agents = 4096;
                    }))
                };
                let (r, f) = (big(Structure::Rwr), big(Structure::Frn));
                let (r0, f0) = (late(&row(0)), late(&row(3)));
                outcome(
                    (r - r0).abs() <= 0.05 && (f - f0).abs() <= 0.05,
                    format!("RWR {r:.3} (256: {r0:.3}), FRN {f:.3} (256: {f0:.3})"),
                )
            },
        },
        Claim {
            id: "structure.table-a1",
            item: "cra-frne",
            source: Source::Book,
            citation: CRA,
            text: "Table A1: agents d links away in FRNE — 4.00, 11.74, 32.38, 74.59, 98.42, 33.23 for d = 1…6 (averaged over 10 graphs; d ≤ 5 within 5 %)",
            check: |_| {
                let paper = [4.00, 11.74, 32.38, 74.59, 98.42, 33.23];
                let mut f = [0.0; 6];
                for seed in 1..=10 {
                    let c = StructureConfig {
                        structure: Structure::Frne,
                        ..StructureConfig::default()
                    };
                    let w = StructureWorld::new(c, seed).expect("a valid config");
                    for (d, v) in fanout(&w.graph().chosen, 6).iter().enumerate() {
                        f[d] += v / 10.0;
                    }
                }
                outcome(
                    (0..5).all(|d| (f[d] - paper[d]).abs() <= 0.05 * paper[d]),
                    format!("{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}", f[0], f[1], f[2], f[3], f[4], f[5]),
                )
            },
        },
        Claim {
            id: "structure.readings.start",
            item: "cra-random-start",
            source: Source::Book,
            citation: CRA,
            text: "the paper's two starts ('evenly distributing … throughout the strategy space' and 'initialized randomly') give the same FRN result (mean payoffs within 0.02)",
            check: |_| {
                let r = late(&runs(SEEDS, |c| {
                    c.structure = Structure::Frn;
                    c.start = Start::Random;
                }));
                let g = late(&row(3));
                outcome((r - g).abs() <= 0.02, format!("random {r:.3}, even {g:.3}"))
            },
        },
        Claim {
            id: "structure.readings.noise",
            item: "cra-copy-noise",
            source: Source::Book,
            citation: CRA,
            text: "the paper's two noise rules ('regardless of which … is adopted' and 'errors in the actual copying process') give the same FRN result (mean payoffs within 0.02)",
            check: |_| {
                let copy = late(&runs(SEEDS, |c| {
                    c.structure = Structure::Frn;
                    c.noise_on = NoiseOn::Copy;
                }));
                let always = late(&row(3));
                outcome(
                    (copy - always).abs() <= 0.02,
                    format!("noise only on copying {copy:.3}, on everyone {always:.3} (Table 2: 2.480)"),
                )
            },
        },
    ]);
    out
}
```

Apply to `survey/src/claims/mod.rs` (a unified diff against `main` at `d898828`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index 7acb5fb..795bbcd 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -9,6 +9,7 @@ mod culture;
 mod ethno;
 mod opinions;
 mod spatial;
+mod structure;
 mod tags;
 
 use crate::claim::Claim;
@@ -26,6 +27,7 @@ pub fn all() -> Vec<Claim> {
         ethno::claims(),
         opinions::claims(),
         spatial::claims(),
+        structure::claims(),
         tags::claims(),
     ]
     .into_iter()
```

- [ ] **Step 2: Format the new file; run**

Run: `rustfmt --edition 2021 survey/src/claims/structure.rs && (cd survey && cargo run --release -- --only structure)`
Expected (as planning measured; stop and report any difference): every claim **Holds** except `structure.readings.noise`, which **Fails** (noise only on copying 2.530, on everyone 2.478; Table 2: 2.480). Table 2: 2.553/0.998, 2.574/0.997, 2.478/0.940, 2.405/0.843, 2.036/0.376, 1.325/0.073, 1.089/0.020; threshold misses 0.140 / 0.026 / 0.263 at 2.2 / 2.3 / 2.4; region RWR −0.0118, FRN +0.0510; FRN slope 0.179, F 1087; FFR-0.3 bi-stable 25/30.

Delete `survey/out/results-structure.json`.

- [ ] **Step 3: Test, check clippy, commit**

Run: `(cd survey && cargo test --release && cargo clippy --all-targets 2>&1 | grep -c 'claims/structure.rs')` — Expected: pass; `0`.

```bash
git add survey/src/claims/structure.rs survey/src/claims/mod.rs
git commit -m "Survey Social Structure and the paper's two readings of its method" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---
### Task 5: README, roadmap, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/superpowers/specs/2026-09-26-social-structure-design.md`

- [ ] **Step 1: README — the intro names the model**

Replace `**Emergence of Classes**, **Ethnocentrism** and **Bounded Confidence**.` with `**Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence** and **Social Structure**.`

- [ ] **Step 2: README — the Social Structure section**

Insert immediately before the line `## Experiments`:

```markdown
### Social Structure (Cohen, Riolo & Axelrod 2001)

256 agents each period play four-move Prisoner's Dilemmas (payoffs 3, 0, 5, 1) with four partners.
A strategy is three probabilities: cooperate on the first move (y), after the other cooperated (p,
"friendliness") and after it defected (q; a low q is "provocable"). At the end of a period each agent
copies the best-scoring agent it played if that one did strictly better — misjudging 10 % of the
time — and each of y, p and q has a 10 % chance of Gaussian noise. What changes between runs is the
social structure, who plays whom: fresh random partners every period (**RWR**), four neighbors on a
16 × 16 torus (**2DK**), fixed random neighbors, four each and symmetric (**FRNE**), fixed random
neighbors drawn once, one-way (**FRN**), or FRN with each partner swapped for a random one with
probability x each period (**FFR-x**, the paper's "dial"). The paper's point: what sustains
cooperation is not the torus's clustering but its continuity — "context preservation".

It reproduces closely (30 runs of 2500 periods, as the paper). Table 2's mean payoffs over the last
1000 periods: RWR 1.089, 2DK 2.553, FRNE 2.574, FRN 2.478, FFR-0.1 2.405, FFR-0.3 2.036, FFR-0.5
1.325, against 1.091, 2.557, 2.575, 2.480, 2.385, 2.100, 1.257. The first period averages 2.25 and
every structure collapses before the fixed ones recover (Fig. 1). In the paper's crucial region of
the p–q plane the average p moves −0.012 under RWR and +0.051 under FRN (the paper: −0.016, +0.052),
because under FRN an agent's partners share its friendliness (slope 0.179, F 1087; the paper 0.158,
F 717; not significant under RWR). FRNE does beat 2DK (note 5), 4096 agents behave like 256 (note 1),
and FRNE's fan-out matches Table A1 to within 3 % out to five links. FFR-0.3 is bi-stable, as stated: 25 of 30 runs
spend 50 periods or more both high and low.

The paper never says what "high cooperation" means. At a mean payoff of 2.3 every row of Table 2's
"Remain High" lands within 0.03 of the paper (FRN 0.940 against 0.942, FFR-0.1 0.843 against 0.844);
2.2 or 2.4 miss by 0.14 and 0.26 — so **High cooperation at** defaults to 2.3. It also describes its
own method twice, and the two readings are switches. **Strategies start** "evenly distributed …
throughout the strategy space" (the Appendix) or "initialized randomly" (§3.1): no difference.
**Noise on** every agent every period, "regardless of which … is adopted" (the Appendix), or only as
"errors in the actual copying process" (§2): these differ — noise only on copying gives FRN 2.530
instead of 2.478 — and only the Appendix's rule reproduces Table 2.

The view is the agents as a block of cells (the torus itself under 2DK; index order otherwise) next
to the paper's p–q plane, with the population's average over the last 200 periods as a fading trail
and every agent as a dot. Color modes: **Friendliness** (p), **Provocability** (1 − q), **Payoff**
and **Strategy** (near Tit-for-Tat, Always Defect, Always Cooperate, or mixed). Inspect an agent for
its strategy, payoff, whom it copied and the partners it played (and Follow it), or a point of the
plane for the agents there. Charts: Mean payoff; Cooperation; Strategy (p, q, y); High cooperation;
Copying (and the partners' p slope). Presets: `cra-rwr`, `cra-2dk`, `cra-frne`, `cra-frn`,
`cra-ffr-01`, `cra-ffr-03`, `cra-ffr-05`, `cra-random-start`, `cra-copy-noise`, each stopping at
2500. **Compare** entry: "Random mixing vs fixed random neighbors — Social Structure (Compare)".
Built-in sweeps: `cra-table-2`, `cra-dial`, `cra-threshold`, `cra-noise`, `cra-population`.

Credit: Michael D. Cohen, Rick L. Riolo and Robert Axelrod, "The Role of Social Structure in the
Maintenance of Cooperative Regimes," *Rationality and Society* 13(1) (2001), 5–32. See
`docs/superpowers/specs/2026-09-26-social-structure-design.md`.
```

- [ ] **Step 3: Roadmap**

Insert before `## Experiments and science`:

```markdown
## Milestone 18: Social Structure (done)

Cohen, Riolo and Axelrod's adaptive agents playing short iterated Prisoner's Dilemmas under six social
structures (2001) as a new model kind, with the paper's substitution dial live and its two readings of
its own method as switches. Table 2, Fig. 1, the crucial p–q region, the partner regression, notes 1
and 5 and Table A1 all reproduce closely; the unstated high-cooperation threshold is recovered as 2.3;
the two starts are equivalent, and of the two noise rules only the Appendix's reproduces Table 2. See
`docs/superpowers/specs/2026-09-26-social-structure-design.md`.
```

and after `- **Hegselmann & Krause's bounded confidence**: done (Milestone 17).` add:

```markdown
- **Cohen, Riolo & Axelrod's social structure**: done (Milestone 18).
```

- [ ] **Step 4: Spec amendments**

In `docs/superpowers/specs/2026-09-26-social-structure-design.md`:
1. In **Config**, the `high` row's Default `chosen in planning` → `2.3`, and its Meaning ends "… planning found 2.3: every row's Remain High within 0.03 of Table 2 there, against 0.14 and 0.26 at 2.2 and 2.4)".
2. In **Config**, the `partners` row's Meaning gains "; 1–16 and fewer than the agents; even under `frne`"; the `agents` row: "a perfect square of at least 3 × 3 under `torus`".
3. In **Fixed structures**, replace the `frne` bullet with: "`frne`: a random `partners`-regular simple symmetric graph (`partners` even): a ring lattice (each agent linked to the `partners`/2 nearest on each side) mixed by CRA's procedure — each agent, n times, swaps one of its neighbors with a random other agent's neighbor (a double-edge swap), kept only when it creates no self-link or repeated link. (CRA describe their construction only in outline; this is a stated choice. No retries are needed.)"
4. In **Views**, the **Charts** bullet: "Mean payoff (with the threshold as a reference line)" → "Mean payoff (chart reference lines are series, so the threshold is not drawn)"; add to **Color modes**: "Payoff scales a score of 0–3 red to green."
5. In **Experiments and CLI**: "The CLI names the stop `(its last period)` (the existing default)." → "The CLI names the stop `(its last period)` (the existing default says `its end year`)."
6. In **Testing**, Web: "determinism through the engine" → "determinism through the engine (all nine presets in the golden list; `cra-2dk` with `stop_at` 30 run to its stop and inspected)".
7. Add before **Architecture**:

```markdown
## Measured in planning

30 seeds × 2500 periods unless stated (the paper's counts); the survey reproduces each.

- Table 2 (mean payoff, periods 1501–2500 / Remain High at 2.3 / Attain High C): RWR 1.089 / 0.020 / 0.43; 2DK 2.553 / 0.998 / 1.00; FRNE 2.574 / 0.997 / 1.00; FRN 2.478 / 0.940 / 1.00; FFR-0.1 2.405 / 0.843 / 1.00; FFR-0.3 2.036 / 0.376 / 1.00; FFR-0.5 1.325 / 0.073 / 0.97. Remain High's largest miss over the rows: 0.140 at 2.2, 0.026 at 2.3, 0.263 at 2.4.
- FFR-0.3: 25 of 30 runs spend 50 periods or more both high and low (bi-stable).
- Fig. 1: first-period means 2.259, 2.253, 2.253, 2.259 (RWR, 2DK, FRNE, FRN); dips 1.11, 1.50, 1.51, 1.30; period 50: 1.12, 2.50, 2.57, 2.05.
- The crucial region (population p 0.30–0.35, q 0.05–0.10): Δp −0.0118 over 21 visits (RWR), +0.0510 over 22 (FRN); pooled partner-p regression FRN slope 0.179, F 1087, N 5632; RWR slope 0.007, F 2.1, N 5376.
- Note 5: FRNE above 2DK (medians 2.572, 2.555; p = 5e-11). Note 1: at 4096 agents RWR 1.083, FRN 2.500 (8 runs). Table A1 (10 FRNE graphs): 4.00, 11.74, 32.14, 72.87, 97.71, 35.74.
- The readings: random start FRN 2.473 (even 2.478); noise only on copying FRN 2.530 (on everyone 2.478).
```

- [ ] **Step 5: Full verification**

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test book -- --ignored
cargo build -p sugarscape-core --target wasm32-unknown-unknown
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && cargo test --release)
```

Expected: all clean and passing. **Browser (controller):** Task 3's list in full, plus one preset of every other model.

- [ ] **Step 6: Commit**

```bash
git add README.md docs/roadmap.md docs/superpowers/specs/2026-09-26-social-structure-design.md
git commit -m "Document Social Structure and the paper's two readings; mark milestone 18 done" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---

## Self-review (planning)

- **Spec coverage:** config, step, fixed structures, statistics, views, Inspect, presets → Task 1; experiments and CLI → Task 2; page and Compare → Task 3; survey → Task 4; docs → Task 5; testing → each task.
- **Amendments** (the 2.3 threshold, FRNE's construction, the chart without a reference line, payoff color, validation limits, the CLI stop text, the web golden list, the measurements) → Task 5 Step 4.
- **Placeholders:** none; code is the verified prototype's, diffs are exact.
