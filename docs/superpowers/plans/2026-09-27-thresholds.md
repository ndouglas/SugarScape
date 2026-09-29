# Threshold Models Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Granovetter's threshold model as one model kind, `thresholds` ("Threshold Models"), with the four extensions he sketches (friends, crowds sampled from a city, clusters with movement, ceilings) and Watts's cascades on random networks as switches; fifteen titled presets, seven measured sweeps and a 20-claim survey, in every playground surface, without changing any existing run.

**Architecture:** `crate::graph` gains a sparse random graph (geometric skipping) and a configuration model. A new core module `crates/sugarscape-core/src/thresholds/` — `config.rs` (parameters, validation, schema), `crowd.rs` (exact fractional thresholds, portable normal quantiles and draws, power-law degrees), `theory.rs` (Granovetter's continuous recursion, Watts's cascade condition), `stats.rs`, `view.rs`, `world.rs` (`ThresholdsWorld`: episodes, synchronous and asynchronous updates, friends, networks, ceilings, clusters, rendering, Inspect), `presets.rs`, `mod.rs` — is wired into `ModelConfig`/`ModelWorld` like the other models, with titles in `titles.rs`. The page adds the model's types, color modes, charts, Inspect rows, a Compare entry and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-27-thresholds-design.md` (binding, as amended in Task 5). Sources in `papers/thresholds/`: `granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (a scan; read by OCR), `watts-2002-pnas-simple-model-of-global-cascades-on-random-networks.pdf`.

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited (`MODEL_GOLDEN` gains fifteen `gr-*`/`watts-*` entries).
- **One engine path; deterministic; portable:** native and WASM fingerprints identical (verified in planning by `wasm-pack test`, including sampled normal crowds, and the web determinism test). State uses `u32` ranges, `f64` samples and `crate::portable::ln`/`exp_neg` only; `f64::ln`/`exp` appear only in `theory.rs` (display and survey).
- **Exact thresholds:** every comparison is a·den ≥ num·g in integers (`Th::reached`/`exceeded`); no division.
- **Literal defaults, named departures, honest descriptions and titles:** the default is Granovetter's uniform crowd of 100 seen whole, synchronous, the actor counted in his own group; Watts's reading, rounding, zero thresholds, friends, networks, ceilings and clusters are switches; every description and title says what was measured.
- **Copy (verbatim):** model label **Threshold Models**; preset ids `gr-uniform`, `gr-perturbed`, `gr-normal-12`, `gr-normal-13`, `gr-normal-sampled`, `gr-city`, `gr-friends`, `gr-friends-perturbed`, `gr-ceilings`, `gr-clusters`, `watts-lower`, `watts-middle`, `watts-upper`, `watts-hetero`, `watts-hub`; Compare entry **Uniform vs perturbed crowd — Threshold Models (Compare)** (id `gr-uniform-vs-perturbed`); color modes **State**, **Threshold**, **Degree**, **Crowd**; schema groups **Crowd**, **Thresholds**, **Friends**, **Network**, **Ceilings**, **Clusters**, **Episodes**, **Stopping**; charts **Participation**, **Episodes**, **Last cascade**, **Swing**; time axis **Steps**; sweeps `gr-sd`, `gr-friends`, `gr-movement`, `gr-ceilings`, `watts-window`, `watts-hetero`, `watts-targeting`; series `acting, step, episodes, last_size, mean_size, global_share, theory, recent_mean, swing`; notice `This run has reached its last step (N) — Reset to run it again`; CLI `(its last step)`.
- Every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4`. Stage only the task's files; never `.claude/` or `papers/`.
- Rust: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. In `survey/`, format only `survey/src/claims/thresholds.rs` (`rustfmt --edition 2021`); do not commit `survey/out/results-*.json`.
- Web: `(cd web && npm run build && npm test)` (run `npm ci` first in a fresh worktree).
- **Browser checks are the controller's** (Task 3's Step 6; the full pass in Task 5).

## Review Focus

1. **Exact threshold arithmetic** — a threshold of k/N must be reached by exactly k of N others at every N (planning's prototype stopped the uniform crowd at 29 through 29/100 × 100 < 29). Pinned in Task 1 by `the_uniform_crowd_riots_to_the_last_person_and_the_perturbed_stops_at_one` (N 100, 37, 1 000) and `exact_thresholds_compare_in_integers`.
2. **Who is in the group** — the actor counts himself as a nonacting member (Granovetter's 63/120), or not; friends are weighted in both numerator and denominator. Pinned in Task 1 by `granovetter_s_friend_example_perceives_63_of_120` and `thresholds_count_oneself_or_not`.
3. **Isolated and zero-threshold actors** — an actor with no neighbors (0 of 0) acts only if its threshold is 0 and zero thresholds act at once; never a division by zero. Pinned in Task 1 by `zero_thresholds_act_at_once_or_when_reached` and the degree-0 case of `degenerate_configs_run_without_panicking`.
4. **Incremental counts under movement, one-way ties and reversibility** — crowd sizes, crowd acting counts and each actor's watched-acting count must match a recount after every step. Pinned in Task 1 by `consistent` in `friends_symmetric_or_one_way_keep_counts`, `clusters_move_and_rioters_can_stop`, `watts_s_networks_and_triggers` and `degenerate_configs_run_without_panicking`.
5. **Episodes that never settle** — ceilings and clusters may oscillate forever; an episode must end at `max_steps` (or never, for clusters) without hanging, and settled worlds must rest without `repeat`. Pinned in Task 1 by `ceilings_make_most_riots_pulse_under_synchronous_updating` and `episodes_record_sizes_and_global_shares`; in Task 4 by `thresholds.gr.no-oscillation`.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --workspace` (950), `cargo clippy --all-targets -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (48), `npm run build && npm test` (708) and the survey (20 claims).

1. **Watts's presets and sweeps are `watts-*`**: `w-scale-free` is already Weisbuch's.
2. **`gr-ceilings` gives 10 % ceilings** (the spec said 30 %): at 30 % seed 1 settles, at 10 % 34 of 40 crowds pulse; the survey states the dependence on who holds them.
3. **`recent_mean` and `swing`** replace the spec's `clusters_mean` (the ceilings sweep needs the swing).
4. **Friends are limited to 2 000 actors** (one-way ties draw every ordered pair).
5. **Normal thresholds use the portable logarithm** (Acklam's quantile, the anasazi's polar draw), so WASM matches native.
6. **Planning's findings** (the survey reproduces them): Granovetter's crowds and continuous Fig. 2 hold; a crowd of 100 tips at a σ set by rounding (12.55, 11.89, 12.23) and sampled crowds do not jump; the city's riot of 100 comes 2.3 % of the time; the friends claims hold under our reading; middling movement is most incendiary; ceilings pulse in most crowds. Watts's window, size and slope ½ hold; his upper edge depends on n; Fig. 4a narrows at the sparse side; Fig. 4b cannot be built above z 1.95; hubs help in both regimes.

---

### Task 1: The threshold models in the core

**Files:**
- Create: `crates/sugarscape-core/src/thresholds/{config,crowd,theory,stats,view,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/graph.rs`, `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `crate::graph::{Graph, gnp_sparse, configuration}` (Step 1), `crate::portable::{ln, exp_neg}`, `crate::anasazi::random::normal`, `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{lerp, Rgb}`, `crate::rng::{self, SimRng}`, `crate::schema::{Apply, Param}`, `crate::presets::ModelPreset`, `crate::opinions::Canvas`.
- Produces: `crate::graph::{gnp_sparse, configuration}`; `thresholds::{ThresholdsConfig, Distribution, Crowd, Rounding, Population, Network, Trigger, Zero, Update, Friends, Ceilings, Clusters, MAX_ACTORS, POWER_LAW_CAP, schema, presets, Th, SCALE, draw, city, inverse_normal, power_law, degrees, granovetter, normal_cdf, poisson, poisson_ratio, poisson_window, power_law_ratio, cascade_ratio, vulnerable, ThresholdsSnapshot, RECENT, SERIES, grid, row, GRID_X, MID_X, SHOWN, TALL, TIME_W, ActorView, ThresholdsCell, ThresholdsInspection, ThresholdsMode, ThresholdsWorld}`; `ModelKind::Thresholds` (`"thresholds"`), `ModelConfig::Thresholds`, `ModelWorld::Thresholds`; fifteen titles in `titles::TITLES`.

- [ ] **Step 1: Two graph builders**

Modify `crates/sugarscape-core/src/graph.rs` — `gnp_sparse` (geometric skipping, portable) and `configuration`, with their tests (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/graph.rs b/crates/sugarscape-core/src/graph.rs
index 857f53e..2f94a09 100644
--- a/crates/sugarscape-core/src/graph.rs
+++ b/crates/sugarscape-core/src/graph.rs
@@ -156,6 +156,66 @@ pub fn gnp(n: usize, p: f64, rng: &mut SimRng) -> Vec<Vec<u32>> {
     adj
 }
 
+/// A random graph drawn in expected time proportional to its links
+/// (Batagelj and Brandes's geometric skipping): each pair (a, b), b < a, in
+/// order, is linked with probability `p`, the gaps between links drawn as
+/// geometric variates with the portable logarithm.
+pub fn gnp_sparse(n: usize, p: f64, rng: &mut SimRng) -> Vec<Vec<u32>> {
+    let mut adj = vec![Vec::new(); n];
+    if p <= 0.0 || n < 2 {
+        return adj;
+    }
+    if p >= 1.0 {
+        for a in 0..n {
+            for b in 0..a {
+                adj[a].push(b as u32);
+                adj[b].push(a as u32);
+            }
+        }
+        return adj;
+    }
+    let lp = crate::portable::ln(1.0 - p);
+    let (mut v, mut w) = (1usize, -1i64);
+    while v < n {
+        let r: f64 = rng.gen();
+        let skip = (crate::portable::ln(1.0 - r) / lp).floor();
+        w += 1 + skip as i64;
+        while w >= v as i64 && v < n {
+            w -= v as i64;
+            v += 1;
+        }
+        if v < n {
+            adj[v].push(w as u32);
+            adj[w as usize].push(v as u32);
+        }
+    }
+    adj
+}
+
+/// The configuration model: each agent gets `degrees[i]` link ends, the
+/// ends are shuffled and paired in order, and self-links and repeated links
+/// are dropped.
+pub fn configuration(degrees: &[u32], rng: &mut SimRng) -> Vec<Vec<u32>> {
+    let n = degrees.len();
+    let mut ends: Vec<u32> = Vec::new();
+    for (i, &d) in degrees.iter().enumerate() {
+        ends.extend(std::iter::repeat_n(i as u32, d as usize));
+    }
+    for i in (1..ends.len()).rev() {
+        let j = rng.gen_range(0..=i as u32) as usize;
+        ends.swap(i, j);
+    }
+    let mut adj = vec![Vec::new(); n];
+    for pair in ends.chunks_exact(2) {
+        let (a, b) = (pair[0], pair[1]);
+        if a != b && !adj[a as usize].contains(&b) {
+            adj[a as usize].push(b);
+            adj[b as usize].push(a);
+        }
+    }
+    adj
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -205,4 +265,29 @@ mod tests {
             45
         );
     }
+
+    #[test]
+    fn sparse_random_graphs_link_about_p_of_all_pairs() {
+        let g = Graph::from_lists(gnp_sparse(4000, 0.001, &mut rng::seeded(3)));
+        simple(&g, 4000);
+        let links = g.edges().len() as f64;
+        let want = 0.001 * 4000.0 * 3999.0 / 2.0;
+        assert!((links - want).abs() < 4.0 * want.sqrt(), "{links}");
+        assert!(gnp_sparse(10, 0.0, &mut rng::seeded(3))
+            .iter()
+            .all(Vec::is_empty));
+        let full = Graph::from_lists(gnp_sparse(10, 1.0, &mut rng::seeded(3)));
+        assert_eq!(full.edges().len(), 45);
+        simple(&full, 10);
+    }
+
+    #[test]
+    fn the_configuration_model_keeps_simple_links_near_the_degrees() {
+        let degrees: Vec<u32> = (0..1000).map(|i| 1 + i % 5).collect();
+        let g = Graph::from_lists(configuration(&degrees, &mut rng::seeded(4)));
+        simple(&g, 1000);
+        let ends: u32 = degrees.iter().sum();
+        let kept = 2 * g.edges().len() as u32;
+        assert!(kept <= ends && kept + 20 >= ends, "{kept} of {ends}");
+    }
 }
````

Run: `cargo test -p sugarscape-core --lib graph`
Expected: PASS (11, including `sparse_random_graphs_link_about_p_of_all_pairs` and `the_configuration_model_keeps_simple_links_near_the_degrees`).

- [ ] **Step 2: Write the module**

Create `crates/sugarscape-core/src/thresholds/config.rs` with exactly this content:

````rust
//! The Threshold Models' parameters: Granovetter's (1978) crowds with the
//! four extensions he sketches (friends, crowds sampled from a city, clusters
//! with movement, ceilings), and Watts's (2002) cascades on random networks,
//! as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How the thresholds are distributed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Distribution {
    /// Granovetter: one each at 0, 1/N, …, (N − 1)/N.
    Uniform,
    /// The uniform crowd with the person at 1/N moved to 2/N.
    Perturbed,
    /// Normal with `mean` and `sd` (below 0 is 0; above 1 never acts).
    Normal,
    /// Everyone at `mean` (Watts's φ*).
    Fixed,
}

/// How a normal crowd is realized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Crowd {
    /// The normal's (i + ½)/N quantiles: Granovetter's population.
    Quantiles,
    /// N independent draws.
    Sampled,
}

/// Whether thresholds stay fractions or become whole people.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rounding {
    Exact,
    Floor,
    Nearest,
}

/// Where each episode's crowd comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Population {
    /// The crowd as drawn from `distribution`.
    Fixed,
    /// N drawn from a city with uniform thresholds 0–99 % (Granovetter).
    City,
}

/// Who sees whom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// Granovetter: everyone sees the whole crowd.
    Everyone,
    /// Watts: Poisson degrees with mean `degree`.
    Random,
    /// Watts's Fig. 4b: p_k ∝ k^−2.5 e^−k/κ (k ≥ 1), κ set for `degree`.
    PowerLaw,
}

/// What starts an episode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    /// Granovetter: whoever has threshold 0 acts at once.
    Instigators,
    /// Watts: one random actor switched on.
    Random,
    /// The highest-degree actor (the lower index on a tie).
    Hub,
}

/// What an actor with threshold 0 or less does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zero {
    /// Acts spontaneously (the rule read literally: 0 ≥ 0).
    Acts,
    /// Acts once at least one neighbor does.
    WhenReached,
}

/// How actors update within a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Everyone from the last step's states: r(t + 1) = F[r(t)].
    Synchronous,
    /// A random order, each seeing the latest states (Watts).
    Asynchronous,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Friends {
    pub enabled: bool,
    /// The chance two actors are friends (Granovetter's "acquaintance volume").
    pub acquaintance: f64,
    /// How many strangers a friend counts for.
    pub weight: u32,
    pub symmetric: bool,
}

impl Default for Friends {
    /// Granovetter's example: friends count twice; people know a quarter of the crowd.
    fn default() -> Self {
        Friends {
            enabled: false,
            acquaintance: 0.25,
            weight: 2,
            symmetric: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Ceilings {
    /// The share of actors (chosen at random) who also leave.
    pub share: f64,
    /// They stop once the proportion of others acting exceeds this.
    pub at: f64,
}

impl Default for Ceilings {
    /// Granovetter: "leave when the total passed 90%".
    fn default() -> Self {
        Ceilings {
            share: 0.0,
            at: 0.9,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Clusters {
    pub enabled: bool,
    pub count: u32,
    /// Each actor's chance per step of moving to a random other crowd.
    pub movement: f64,
}

impl Default for Clusters {
    fn default() -> Self {
        Clusters {
            enabled: false,
            count: 10,
            movement: 0.05,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThresholdsConfig {
    /// N, per crowd.
    pub actors: u32,
    pub distribution: Distribution,
    /// The normal's mean (and `fixed`'s threshold), as a fraction.
    pub mean: f64,
    pub sd: f64,
    pub crowd: Crowd,
    pub rounding: Rounding,
    pub population: Population,
    pub network: Network,
    /// Watts's z.
    pub degree: f64,
    /// The actor counts as a nonacting member of the group he divides by.
    pub counts_self: bool,
    pub friends: Friends,
    pub trigger: Trigger,
    pub zero: Zero,
    pub update: Update,
    pub ceilings: Ceilings,
    pub clusters: Clusters,
    /// Start a new episode at each equilibrium.
    pub repeat: bool,
    /// The share of actors that makes a cascade global.
    pub global: f64,
    /// An episode ends here if it has not settled.
    pub max_steps: u32,
    /// Stop at this step (0: never).
    pub stop_at: u32,
}

impl Default for ThresholdsConfig {
    /// Granovetter's uniform crowd: 100 people, thresholds 0 to 99.
    fn default() -> Self {
        ThresholdsConfig {
            actors: 100,
            distribution: Distribution::Uniform,
            mean: 0.25,
            sd: 0.122,
            crowd: Crowd::Quantiles,
            rounding: Rounding::Exact,
            population: Population::Fixed,
            network: Network::Everyone,
            degree: 3.0,
            counts_self: true,
            friends: Friends::default(),
            trigger: Trigger::Instigators,
            zero: Zero::Acts,
            update: Update::Synchronous,
            ceilings: Ceilings::default(),
            clusters: Clusters::default(),
            repeat: false,
            global: 0.1,
            max_steps: 1000,
            stop_at: 0,
        }
    }
}

/// The most actors in a world.
pub const MAX_ACTORS: u32 = 20_000;
/// The power-law family's largest mean degree (k ≥ 1, τ 2.5): ζ(1.5)/ζ(2.5).
pub const POWER_LAW_CAP: f64 = 1.95;

impl ThresholdsConfig {
    /// Actors in the world: N, or K crowds of N.
    pub fn population_size(&self) -> u32 {
        if self.clusters.enabled {
            self.actors * self.clusters.count
        } else {
            self.actors
        }
    }

    /// Whether acting can stop (ceilings or clusters): Granovetter's
    /// "removal", which makes oscillation possible.
    pub fn reversible(&self) -> bool {
        self.ceilings.share > 0.0 || self.clusters.enabled
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (2..=MAX_ACTORS).contains(&self.actors),
            "actors",
            "must be between 2 and 20000",
        );
        check(unit(self.mean), "mean", "must be between 0 and 1");
        check(unit(self.sd), "sd", "must be between 0 and 1");
        check(
            match self.network {
                Network::Everyone => true,
                Network::Random => {
                    self.degree >= 0.0
                        && self.degree <= 100.0
                        && self.degree < f64::from(self.actors - 1)
                }
                Network::PowerLaw => self.degree > 1.0 && self.degree < POWER_LAW_CAP,
            },
            "degree",
            match self.network {
                Network::PowerLaw => {
                    "must be between 1 and 1.95: with k ≥ 1 and τ 2.5 the power law's mean degree cannot exceed ζ(1.5)/ζ(2.5)"
                }
                _ => "must be between 0 and 100 and less than the number of actors",
            },
        );
        let f = &self.friends;
        check(
            !f.enabled
                || (self.network == Network::Everyone
                    && !self.clusters.enabled
                    && unit(f.acquaintance)
                    && (1..=20).contains(&f.weight)
                    && self.actors <= 2000),
            "friends",
            "need everyone to see everyone, no clusters, an acquaintance between 0 and 1, a weight of 1 to 20 and at most 2000 actors",
        );
        check(
            self.trigger != Trigger::Hub || self.network != Network::Everyone,
            "trigger",
            "the hub needs a network",
        );
        let c = &self.ceilings;
        check(
            unit(c.share) && unit(c.at),
            "ceilings",
            "the share and the level must be between 0 and 1",
        );
        let k = &self.clusters;
        check(
            !k.enabled
                || ((2..=50).contains(&k.count)
                    && unit(k.movement)
                    && self.population == Population::City
                    && self.network == Network::Everyone
                    && self.actors * k.count <= MAX_ACTORS),
            "clusters",
            "need 2 to 50 crowds drawn from the city, everyone seeing their own crowd, a movement between 0 and 1 and at most 20000 actors in all",
        );
        check(
            self.global > 0.0 && self.global <= 1.0,
            "global",
            "must be above 0 and at most 1",
        );
        check(
            (1..=100_000).contains(&self.max_steps),
            "max_steps",
            "must be between 1 and 100000",
        );
        check(
            self.stop_at <= 10_000_000,
            "stop_at",
            "must be at most 10000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &ThresholdsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("actors", self.actors == next.actors),
            ("distribution", self.distribution == next.distribution),
            ("mean", self.mean == next.mean),
            ("sd", self.sd == next.sd),
            ("crowd", self.crowd == next.crowd),
            ("rounding", self.rounding == next.rounding),
            ("population", self.population == next.population),
            ("network", self.network == next.network),
            ("degree", self.degree == next.degree),
            ("counts_self", self.counts_self == next.counts_self),
            ("friends", self.friends == next.friends),
            ("trigger", self.trigger == next.trigger),
            ("zero", self.zero == next.zero),
            ("ceilings", self.ceilings == next.ceilings),
            ("clusters", self.clusters == next.clusters),
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
        Param::integer("Crowd", "actors", "Actors (N)", (2, MAX_ACTORS), Reset)
            .with_help("Granovetter's examples: 100. Watts: 10 000."),
        Param::choice(
            "Crowd",
            "population",
            "Each crowd",
            &[
                ("fixed", "As drawn"),
                ("city", "Sampled from a city (uniform 0–99 %)"),
            ],
            Reset,
        ),
        Param::choice(
            "Crowd",
            "trigger",
            "Started by",
            &[
                ("instigators", "Those with threshold 0 (Granovetter)"),
                ("random", "One random actor (Watts)"),
                ("hub", "The best-connected actor"),
            ],
            Reset,
        ),
        Param::choice(
            "Crowd",
            "update",
            "Actors decide",
            &[
                ("synchronous", "Together, from the last step"),
                ("asynchronous", "One at a time (Watts)"),
            ],
            Live,
        ),
        Param::bool("Crowd", "counts_self", "Count oneself in the group", Reset)
            .shown_if("network", "everyone")
            .with_help("Granovetter's example divides by the whole crowd, himself included: 63/120."),
        Param::choice(
            "Thresholds",
            "distribution",
            "Thresholds",
            &[
                ("uniform", "Uniform: 0, 1, …, N − 1"),
                ("perturbed", "Uniform, the 1 moved to 2"),
                ("normal", "Normal"),
                ("fixed", "Everyone the same"),
            ],
            Reset,
        ),
        Param::number("Thresholds", "mean", "Mean (or the threshold)", (0.0, 1.0, 0.01), Reset)
            .with_help("A fraction of the group: Granovetter's 25 % is 0.25; Watts's φ* 0.18."),
        Param::number("Thresholds", "sd", "Spread (sd)", (0.0, 1.0, 0.001), Reset)
            .shown_if("distribution", "normal"),
        Param::choice(
            "Thresholds",
            "crowd",
            "A normal crowd is",
            &[
                ("quantiles", "The normal's quantiles (Granovetter)"),
                ("sampled", "Drawn at random"),
            ],
            Reset,
        )
        .shown_if("distribution", "normal"),
        Param::choice(
            "Thresholds",
            "rounding",
            "Thresholds are",
            &[
                ("exact", "Fractions"),
                ("floor", "Whole people, rounded down"),
                ("nearest", "Whole people, rounded"),
            ],
            Reset,
        )
        .shown_if("distribution", "normal"),
        Param::choice(
            "Thresholds",
            "zero",
            "A threshold of 0",
            &[
                ("acts", "Acts at once"),
                ("when_reached", "Acts once a neighbor does"),
            ],
            Reset,
        ),
        Param::bool("Friends", "friends.enabled", "Friends count more", Reset)
            .shown_if("network", "everyone"),
        Param::number("Friends", "friends.acquaintance", "Acquaintance", (0.0, 1.0, 0.01), Reset)
            .shown_if("friends.enabled", "true")
            .with_help("The chance two actors are friends. Granovetter: the largest effect at about a quarter."),
        Param::integer("Friends", "friends.weight", "A friend counts as", (1, 20), Reset)
            .shown_if("friends.enabled", "true"),
        Param::bool("Friends", "friends.symmetric", "Friendship is mutual", Reset)
            .shown_if("friends.enabled", "true"),
        Param::choice(
            "Network",
            "network",
            "Who sees whom",
            &[
                ("everyone", "The whole crowd (Granovetter)"),
                ("random", "A random network (Watts)"),
                ("power_law", "A network with hubs (Watts, Fig. 4b)"),
            ],
            Reset,
        ),
        Param::number("Network", "degree", "Mean degree (z)", (0.0, 100.0, 0.01), Reset)
            .with_help("For the networks. With hubs it must be below 1.95."),
        Param::number("Ceilings", "ceilings.share", "Share who also leave", (0.0, 1.0, 0.01), Reset)
            .with_help("Granovetter's Fig. 3: join at a crowd, leave at a mob."),
        Param::number("Ceilings", "ceilings.at", "They leave above", (0.0, 1.0, 0.01), Reset),
        Param::bool("Clusters", "clusters.enabled", "Several crowds", Reset)
            .with_help("Crowds from the city, with people moving between them."),
        Param::integer("Clusters", "clusters.count", "Crowds", (2, 50), Reset)
            .shown_if("clusters.enabled", "true"),
        Param::number("Clusters", "clusters.movement", "Movement per step", (0.0, 1.0, 0.001), Reset)
            .shown_if("clusters.enabled", "true"),
        Param::bool("Episodes", "repeat", "Start again at each equilibrium", Live),
        Param::number("Episodes", "global", "A cascade is global above", (0.001, 1.0, 0.001), Live),
        Param::integer("Episodes", "max_steps", "An episode ends after", (1, 100_000), Live),
        Param::integer("Stopping", "stop_at", "Stop at step", (0, 10_000_000), Live)
            .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_granovetter_s_uniform_crowd() {
        let c = ThresholdsConfig::default();
        assert_eq!((c.actors, c.distribution), (100, Distribution::Uniform));
        assert_eq!(
            (c.network, c.trigger),
            (Network::Everyone, Trigger::Instigators)
        );
        assert!(c.counts_self && !c.reversible());
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = ThresholdsConfig {
            actors: 1,
            mean: 1.5,
            sd: -0.1,
            network: Network::PowerLaw,
            degree: 3.0,
            trigger: Trigger::Instigators,
            ceilings: Ceilings {
                share: 2.0,
                at: 0.9,
            },
            global: 0.0,
            max_steps: 0,
            stop_at: 20_000_000,
            ..ThresholdsConfig::default()
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
                "actors",
                "mean",
                "sd",
                "degree",
                "ceilings",
                "global",
                "max_steps",
                "stop_at"
            ]
        );
    }

    #[test]
    fn combinations_that_cannot_run_are_refused() {
        let friends_on_a_network = ThresholdsConfig {
            network: Network::Random,
            friends: Friends {
                enabled: true,
                ..Friends::default()
            },
            ..ThresholdsConfig::default()
        };
        assert_eq!(
            friends_on_a_network.validate().unwrap_err()[0].field,
            "friends"
        );
        let hub_without_network = ThresholdsConfig {
            trigger: Trigger::Hub,
            ..ThresholdsConfig::default()
        };
        assert_eq!(
            hub_without_network.validate().unwrap_err()[0].field,
            "trigger"
        );
        let clusters_of_a_fixed_crowd = ThresholdsConfig {
            clusters: Clusters {
                enabled: true,
                ..Clusters::default()
            },
            ..ThresholdsConfig::default()
        };
        assert_eq!(
            clusters_of_a_fixed_crowd.validate().unwrap_err()[0].field,
            "clusters"
        );
        let clusters = ThresholdsConfig {
            population: Population::City,
            ..clusters_of_a_fixed_crowd
        };
        assert!(clusters.validate().is_ok());
        assert_eq!(clusters.population_size(), 1000);
        assert!(clusters.reversible());
        let hubs = ThresholdsConfig {
            network: Network::PowerLaw,
            degree: 1.5,
            ..ThresholdsConfig::default()
        };
        assert!(hubs.validate().is_ok());
    }

    #[test]
    fn the_crowd_changes_only_on_reset() {
        let next = ThresholdsConfig {
            distribution: Distribution::Perturbed,
            repeat: true,
            ..ThresholdsConfig::default()
        };
        let changes = ThresholdsConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "distribution");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Thresholds(ThresholdsConfig {
            distribution: Distribution::Normal,
            ..ThresholdsConfig::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
````

Create `crates/sugarscape-core/src/thresholds/crowd.rs` with exactly this content:

````rust
//! Drawing a crowd's thresholds, and power-law degrees, bit-for-bit the same
//! on every platform (the portable logarithm and exponential only). A
//! threshold is kept as an exact fraction `num/den`, so "the others acting
//! reach θ of the group" is compared in integers: floating division stopped
//! the planning prototype's uniform crowd at 29 (29/100 × 100 < 29).

use rand::Rng;

use super::config::{Crowd, Distribution, Rounding, ThresholdsConfig};
use crate::anasazi::random::normal;
use crate::portable::{exp_neg, ln};
use crate::rng::SimRng;

/// Real thresholds are kept to six decimals.
pub const SCALE: u64 = 1_000_000;

/// A threshold θ = num/den; above 1 (num > den) the actor never acts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Th {
    pub num: u64,
    pub den: u64,
}

impl Th {
    pub const ZERO: Th = Th { num: 0, den: 1 };
    pub const NEVER: Th = Th { num: 2, den: 1 };

    /// A real fraction: 0 or less is 0, above 1 is never, else six decimals.
    pub fn from_fraction(x: f64) -> Th {
        if x <= 0.0 {
            Th::ZERO
        } else if x > 1.0 {
            Th::NEVER
        } else {
            Th {
                num: (x * SCALE as f64).round() as u64,
                den: SCALE,
            }
        }
    }

    /// `k` people out of `n` (more than `n`: never).
    pub fn people(k: i64, n: u32) -> Th {
        if k <= 0 {
            Th::ZERO
        } else if k > i64::from(n) {
            Th::NEVER
        } else {
            Th {
                num: k as u64,
                den: u64::from(n),
            }
        }
    }

    pub fn value(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// Whether `a` of `g` reaches θ: a·den ≥ num·g, exactly.
    pub fn reached(self, a: u64, g: u64) -> bool {
        u128::from(a) * u128::from(self.den) >= u128::from(self.num) * u128::from(g)
    }

    /// Whether `a` of `g` exceeds θ: a·den > num·g.
    pub fn exceeded(self, a: u64, g: u64) -> bool {
        u128::from(a) * u128::from(self.den) > u128::from(self.num) * u128::from(g)
    }
}

/// The standard normal's quantile (Acklam's rational approximation, relative
/// error under 1.2·10⁻⁹), from the portable logarithm and `sqrt` only.
pub fn inverse_normal(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    const LOW: f64 = 0.02425;
    if p < LOW {
        tail((-2.0 * ln(p)).sqrt())
    } else if p > 1.0 - LOW {
        -tail((-2.0 * ln(1.0 - p)).sqrt())
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    }
}

/// A real threshold `x` made into a `Th` under `rounding` for `n` people.
fn rounded(x: f64, rounding: Rounding, n: u32) -> Th {
    let people = x * f64::from(n);
    match rounding {
        Rounding::Exact => Th::from_fraction(x),
        Rounding::Floor => Th::people(people.floor() as i64, n),
        Rounding::Nearest => Th::people(people.round() as i64, n),
    }
}

/// The crowd's thresholds (N of them), drawn from `c`'s distribution.
pub fn draw(c: &ThresholdsConfig, rng: &mut SimRng) -> Vec<Th> {
    let n = c.actors;
    match c.distribution {
        Distribution::Uniform => (0..n).map(|i| Th::people(i64::from(i), n)).collect(),
        Distribution::Perturbed => (0..n)
            .map(|i| Th::people(i64::from(if i == 1 { 2 } else { i }), n))
            .collect(),
        Distribution::Fixed => vec![Th::from_fraction(c.mean); n as usize],
        Distribution::Normal => (0..n)
            .map(|i| {
                let z = match c.crowd {
                    Crowd::Quantiles => inverse_normal((f64::from(i) + 0.5) / f64::from(n)),
                    Crowd::Sampled => normal(rng),
                };
                rounded(c.mean + c.sd * z, c.rounding, n)
            })
            .collect(),
    }
}

/// Thresholds for `n` actors drawn from Granovetter's city: uniform 0–99 %.
pub fn city(n: u32, rng: &mut SimRng) -> Vec<Th> {
    (0..n)
        .map(|_| Th::people(i64::from(rng.gen_range(0..100u32)), 100))
        .collect()
}

/// p_k ∝ k^−2.5 e^−k/κ for k = 1..=`kmax`, normalized, with κ set so the
/// mean is `z` (1 < z < 1.95), portably: k^−2.5 = 1/(k² √k).
pub fn power_law(z: f64, kmax: u32) -> Vec<f64> {
    let dist = |kappa: f64| {
        let mut p: Vec<f64> = (1..=kmax)
            .map(|k| {
                let k = f64::from(k);
                exp_neg(-k / kappa) / (k * k * k.sqrt())
            })
            .collect();
        let s: f64 = p.iter().sum();
        for v in &mut p {
            *v /= s;
        }
        p
    };
    let mean = |p: &[f64]| {
        p.iter()
            .enumerate()
            .map(|(i, v)| (i + 1) as f64 * v)
            .sum::<f64>()
    };
    let (mut lo, mut hi) = (0.01f64, 1.0e9f64);
    for _ in 0..100 {
        let mid = (lo * hi).sqrt();
        if mean(&dist(mid)) < z {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    dist((lo * hi).sqrt())
}

/// `n` degrees drawn from `p` (p[0] is k = 1) by inverse cumulative lookup.
pub fn degrees(p: &[f64], n: u32, rng: &mut SimRng) -> Vec<u32> {
    let mut cum = Vec::with_capacity(p.len());
    let mut s = 0.0;
    for v in p {
        s += v;
        cum.push(s);
    }
    (0..n)
        .map(|_| {
            let u: f64 = rng.gen::<f64>() * s;
            cum.partition_point(|&c| c <= u).min(p.len() - 1) as u32 + 1
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    #[test]
    fn exact_thresholds_compare_in_integers() {
        // 29/100 of 100 is reached by 29, which floating division misses.
        let t = Th::people(29, 100);
        assert!(t.reached(29, 100) && !t.reached(28, 100));
        assert!(Th::ZERO.reached(0, 5) && Th::ZERO.reached(0, 0));
        assert!(!Th::NEVER.reached(10, 10));
        assert!(
            Th::from_fraction(0.9).exceeded(91, 100) && !Th::from_fraction(0.9).exceeded(90, 100)
        );
        assert_eq!(Th::from_fraction(-0.2), Th::ZERO);
        assert_eq!(Th::from_fraction(1.2), Th::NEVER);
    }

    #[test]
    fn the_uniform_and_perturbed_crowds_are_granovetter_s() {
        let c = ThresholdsConfig::default();
        let t = draw(&c, &mut rng::seeded(1));
        assert_eq!(t.len(), 100);
        assert_eq!(
            (t[0], t[1], t[99]),
            (Th::ZERO, Th::people(1, 100), Th::people(99, 100))
        );
        let p = draw(
            &ThresholdsConfig {
                distribution: Distribution::Perturbed,
                ..c
            },
            &mut rng::seeded(1),
        );
        assert_eq!((p[1], p[2]), (Th::people(2, 100), Th::people(2, 100)));
    }

    #[test]
    fn the_inverse_normal_matches_known_quantiles() {
        for (p, z) in [
            (0.5, 0.0),
            (0.975, 1.959_963_985),
            (0.1, -1.281_551_566),
            (0.001, -3.090_232_306),
        ] {
            assert!((inverse_normal(p) - z).abs() < 1e-8, "{p}");
        }
    }

    #[test]
    fn normal_crowds_follow_quantiles_or_samples_and_round() {
        let c = ThresholdsConfig {
            distribution: Distribution::Normal,
            sd: 0.12,
            ..ThresholdsConfig::default()
        };
        let q = draw(&c, &mut rng::seeded(1));
        // The median actors sit at the mean; the lowest below 0 is 0.
        assert!((q[50].value() - (0.25 + 0.12 * inverse_normal(0.505))).abs() < 1e-6);
        assert_eq!(q[0], Th::ZERO);
        let near = draw(
            &ThresholdsConfig {
                rounding: Rounding::Nearest,
                ..c.clone()
            },
            &mut rng::seeded(1),
        );
        assert!(near.iter().all(|t| t.den == 1 || t.den == 100));
        let floor = draw(
            &ThresholdsConfig {
                rounding: Rounding::Floor,
                ..c.clone()
            },
            &mut rng::seeded(1),
        );
        assert!(floor.iter().zip(&near).all(|(f, n)| f.value() <= n.value()));
        let sampled = |seed| {
            draw(
                &ThresholdsConfig {
                    crowd: Crowd::Sampled,
                    ..c.clone()
                },
                &mut rng::seeded(seed),
            )
        };
        assert_ne!(sampled(1), sampled(2));
        assert_eq!(sampled(3), sampled(3));
    }

    #[test]
    fn the_city_is_uniform_on_whole_percents() {
        let t = city(10_000, &mut rng::seeded(2));
        assert!(t.iter().all(|t| t.den == 100 || *t == Th::ZERO));
        let zeros = t.iter().filter(|t| **t == Th::ZERO).count();
        assert!((60..=140).contains(&zeros), "{zeros}");
    }

    #[test]
    fn power_laws_hit_their_mean_degree() {
        for z in [1.2, 1.5, 1.9] {
            let p = power_law(z, 5000);
            let m: f64 = p.iter().enumerate().map(|(i, v)| (i + 1) as f64 * v).sum();
            assert!((m - z).abs() < 1e-6, "{z}: {m}");
        }
        let d = degrees(&power_law(1.5, 5000), 20_000, &mut rng::seeded(3));
        let mean = d.iter().map(|&k| f64::from(k)).sum::<f64>() / 20_000.0;
        assert!((mean - 1.5).abs() < 0.1, "{mean}");
        assert!(d.iter().all(|&k| k >= 1));
    }
}
````

Create `crates/sugarscape-core/src/thresholds/theory.rs` with exactly this content:

````rust
//! What theory expects. Granovetter's continuous equilibrium: the forward
//! recursion r ← N·F(r/N) from r = 0, F the normal c.d.f. of thresholds with
//! the mass below 0 at 0 (his Fig. 1–2). Watts's cascade condition (Eq. 5):
//! global cascades are possible where Σ k(k − 1)ρ_k p_k exceeds z, ρ_k = F(1/k)
//! the chance a degree-k node is vulnerable. For display and the survey
//! only; nothing here feeds a run.

use super::crowd::power_law;

/// The standard normal c.d.f. (Φ), from the complementary error function
/// (Numerical Recipes' Chebyshev fit, fractional error under 1.2·10⁻⁷).
pub fn normal_cdf(x: f64) -> f64 {
    let z = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.5 * z);
    let erfc = t
        * (-z * z - 1.265_512_23
            + t * (1.000_023_68
                + t * (0.374_091_96
                    + t * (0.096_784_18
                        + t * (-0.186_288_06
                            + t * (0.278_868_07
                                + t * (-1.135_203_98
                                    + t * (1.488_515_87
                                        + t * (-0.822_152_23 + t * 0.170_872_77)))))))))
            .exp();
    if x >= 0.0 {
        1.0 - 0.5 * erfc
    } else {
        0.5 * erfc
    }
}

/// Granovetter's continuous equilibrium, in people, for N people with normal
/// thresholds (mean, sd as fractions): r ← N·Φ((r/N − mean)/sd).
pub fn granovetter(n: u32, mean: f64, sd: f64) -> f64 {
    let nf = f64::from(n);
    let f = |r: f64| {
        let x = r / nf;
        if sd == 0.0 {
            if x >= mean {
                nf
            } else {
                0.0
            }
        } else {
            nf * normal_cdf((x - mean) / sd)
        }
    };
    let mut r = 0.0;
    for _ in 0..1_000_000 {
        let next = f(r);
        if (next - r).abs() < 1e-9 {
            return next;
        }
        r = next;
    }
    r
}

/// Poisson degrees with mean z: p_k for k = 0..=`kmax`.
pub fn poisson(z: f64, kmax: u32) -> Vec<f64> {
    let mut p = vec![(-z).exp()];
    for k in 1..=kmax {
        let prev = p[k as usize - 1];
        p.push(prev * z / f64::from(k));
    }
    p
}

/// ρ_k: the chance a node of degree k is vulnerable (θ ≤ 1/k), for
/// thresholds normal around `phi` with `sd` (0: all at `phi`).
pub fn vulnerable(k: u32, phi: f64, sd: f64) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let edge = 1.0 / f64::from(k);
    if sd == 0.0 {
        if phi <= edge + 1e-12 {
            1.0
        } else {
            0.0
        }
    } else {
        normal_cdf((edge - phi) / sd)
    }
}

/// Watts's G₀″(1)/z = Σ k(k − 1)ρ_k p_k / z for degrees `p` (p[i] is degree
/// i + `first`): above 1, a vulnerable cluster percolates.
pub fn cascade_ratio(p: &[f64], first: u32, phi: f64, sd: f64) -> f64 {
    let (mut top, mut z) = (0.0, 0.0);
    for (i, &q) in p.iter().enumerate() {
        let k = i as u32 + first;
        let kf = f64::from(k);
        z += kf * q;
        top += kf * (kf - 1.0) * vulnerable(k, phi, sd) * q;
    }
    top / z
}

/// The ratio on a uniform random graph (Poisson) of mean degree z.
pub fn poisson_ratio(z: f64, phi: f64, sd: f64) -> f64 {
    cascade_ratio(&poisson(z, 200), 0, phi, sd)
}

/// The ratio on Watts's power-law graph (τ 2.5) of mean degree z (< 1.95).
pub fn power_law_ratio(z: f64, phi: f64, sd: f64) -> f64 {
    cascade_ratio(&power_law(z, 5000), 1, phi, sd)
}

/// Where the Poisson ratio crosses 1, scanning z from `from` to `to` in
/// steps of 0.001: the analytic cascade window's lower and upper edges.
pub fn poisson_window(phi: f64, sd: f64, from: f64, to: f64) -> Option<(f64, f64)> {
    let mut edges = None;
    let mut z = from;
    while z <= to {
        if poisson_ratio(z, phi, sd) > 1.0 {
            edges = Some(match edges {
                None => (z, z),
                Some((lo, _)) => (lo, z),
            });
        }
        z += 0.001;
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_normal_cdf_is_accurate() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7);
        assert!((normal_cdf(1.959_964) - 0.975).abs() < 1e-6);
        assert!((normal_cdf(-1.0) - 0.158_655_25).abs() < 1e-6);
    }

    #[test]
    fn figure_2_jumps_between_sd_12_2_and_12_3() {
        let below = granovetter(100, 0.25, 0.122);
        let above = granovetter(100, 0.25, 0.123);
        assert!(below > 5.0 && below < 6.0, "{below}");
        assert!(above > 99.9, "{above}");
        // As σ grows without bound the equilibrium falls toward 50.
        assert!((granovetter(100, 0.25, 10.0) - 50.0).abs() < 1.5);
    }

    #[test]
    fn watts_s_poisson_window_at_phi_0_18() {
        // zQ(K* − 1, z) = 1 with K* = 1/0.18: nodes of degree ≤ 5 are vulnerable.
        let (lo, hi) = poisson_window(0.18, 0.0, 0.5, 10.0).unwrap();
        assert!(
            (lo - 1.02).abs() < 0.02 && (hi - 5.76).abs() < 0.03,
            "{lo} {hi}"
        );
        assert!(poisson_ratio(3.0, 0.18, 0.0) > 1.0);
        assert!(poisson_ratio(7.0, 0.18, 0.0) < 1.0);
        // Varied thresholds widen the window's upper side.
        let (_, wide) = poisson_window(0.18, 0.1, 0.5, 20.0).unwrap();
        assert!(wide > 10.0, "{wide}");
    }

    #[test]
    fn the_power_law_window_is_empty_at_phi_0_18() {
        for z in [1.2, 1.5, 1.9] {
            assert!(power_law_ratio(z, 0.18, 0.0) < 0.7, "{z}");
        }
        assert!(power_law_ratio(1.78, 0.05, 0.0) > 1.5);
    }
}
````

Create `crates/sugarscape-core/src/thresholds/stats.rs` with exactly this content:

````rust
//! The Threshold Models' statistics: how many act now, how each episode
//! ends, and how widely participation swings.

use serde::Serialize;

use crate::stats::Series;

/// Steps the recent mean and swing look back over.
pub const RECENT: usize = 100;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 9] = [
    "acting",
    "step",
    "episodes",
    "last_size",
    "mean_size",
    "global_share",
    "theory",
    "recent_mean",
    "swing",
];

/// One step's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ThresholdsSnapshot {
    pub tick: u64,
    /// The share acting now.
    pub acting: f64,
    /// Steps into the current episode.
    pub step: u32,
    /// Episodes finished so far.
    pub episodes: u32,
    /// The last finished episode's final share (0 before the first).
    pub last_size: f64,
    /// The mean final share over finished episodes (0 before the first).
    pub mean_size: f64,
    /// The share of finished episodes that were global.
    pub global_share: f64,
    /// Granovetter's continuous equilibrium share (a normal crowd seen by
    /// everyone); NaN (null) otherwise.
    pub theory: f64,
    /// The mean share acting over the last `RECENT` steps.
    pub recent_mean: f64,
    /// Its range (largest − smallest) over the last `RECENT` steps.
    pub swing: f64,
}

impl Series for ThresholdsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "acting" => self.acting,
            "step" => f64::from(self.step),
            "episodes" => f64::from(self.episodes),
            "last_size" => self.last_size,
            "mean_size" => self.mean_size,
            "global_share" => self.global_share,
            "theory" => self.theory,
            "recent_mean" => self.recent_mean,
            "swing" => self.swing,
            _ => return None,
        })
    }
}
````

Create `crates/sugarscape-core/src/thresholds/view.rs` with exactly this content:

````rust
//! The frame: participation over time, Granovetter's Figure 1 (the
//! thresholds' c.d.f. against the 45° line, with the episode's staircase) or
//! the histogram of episode sizes, and a grid of the actors.

use crate::render::{lerp, Rgb};

/// Steps the time panel shows.
pub const SHOWN: usize = 400;
/// The time panel's width and every panel's height.
pub const TIME_W: usize = SHOWN + 1;
pub const TALL: usize = 201;
/// Cells between panels.
pub const GAP: usize = 8;
/// The middle panel's position and width.
pub const MID_X: usize = TIME_W + GAP;
pub const MID_W: usize = 101;
/// Where the actor grid starts.
pub const GRID_X: usize = MID_X + MID_W + GAP;

pub const LINE: Rgb = [0xe8, 0xe4, 0xda];
pub const MARK: Rgb = [0x5a, 0x55, 0x4c];
pub const BAR: Rgb = [0x8f, 0xb8, 0xde];
pub const PATH: Rgb = [0xff, 0x8a, 0x5c];
pub const ACTING: Rgb = [0xff, 0x6b, 0x4a];
pub const IDLE: Rgb = [0x3a, 0x40, 0x4c];
pub const SEED: Rgb = [0xf2, 0xc1, 0x4e];
pub const LOW: Rgb = [0xff, 0x5a, 0x3c];
pub const HIGH: Rgb = [0x3c, 0x6e, 0xff];
pub const FEW: Rgb = [0x2a, 0x2e, 0x3a];
pub const MANY: Rgb = [0x7c, 0xe0, 0x8a];
/// Crowds' colors, cycled.
pub const CROWDS: [Rgb; 8] = [
    [0xf2, 0xc1, 0x4e],
    [0x4a, 0x9c, 0xff],
    [0x6c, 0xd0, 0x7a],
    [0xd0, 0x6c, 0xe0],
    [0xff, 0x7a, 0x59],
    [0x5c, 0xd6, 0xd6],
    [0xc8, 0xc8, 0x70],
    [0xa0, 0x88, 0xff],
];

/// The row of share `x`: 1 at the top, 0 at the bottom.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (TALL - 1) as f64).round() as usize
}

/// The share a row stands for.
pub fn share_at(y: usize) -> f64 {
    1.0 - y as f64 / (TALL - 1) as f64
}

/// The middle panel's column of share `x` (Figure 1's horizontal axis).
pub fn col(x: f64) -> usize {
    (x.clamp(0.0, 1.0) * (MID_W - 1) as f64).round() as usize
}

/// The actor grid's side (actors per row) and cells per actor.
pub fn grid(n: u32) -> (usize, usize) {
    let side = (f64::from(n).sqrt().ceil() as usize).max(1);
    (side, (TALL / side).max(1))
}

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_line_up() {
        assert_eq!((row(1.0), row(0.0), row(0.5)), (0, 200, 100));
        assert!((share_at(100) - 0.5).abs() < 1e-12);
        assert_eq!((col(0.0), col(1.0), col(0.25)), (0, 100, 25));
        assert_eq!(grid(100), (10, 20));
        assert_eq!(grid(10_000), (100, 2));
        assert_eq!((MID_X, GRID_X), (409, 518));
    }
}
````

Create `crates/sugarscape-core/src/thresholds/world.rs` with exactly this content:

````rust
//! The Threshold Models world. An episode is one crowd (or network) and one
//! trigger: each step, actors act when the others acting reach their
//! threshold of the group they watch, until nothing changes. Without ceilings
//! or clusters nobody stops (Granovetter's model without "removal"); with
//! them every state is recomputed each step, and the crowd may never settle.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{Distribution, Network, Population, ThresholdsConfig, Trigger, Update, Zero};
use super::crowd::{self, Th};
use super::stats::{ThresholdsSnapshot, RECENT};
use super::theory;
use super::view::{
    col, grid, row, scale, share_at, ACTING, BAR, CROWDS, FEW, GRID_X, HIGH, IDLE, LINE, LOW, MANY,
    MARK, MID_W, MID_X, PATH, SEED, SHOWN, TALL, TIME_W,
};
use crate::config::FieldError;
use crate::export;
use crate::graph;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThresholdsMode {
    State,
    Threshold,
    Degree,
    Crowd,
}

impl std::str::FromStr for ThresholdsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "state" => Self::State,
            "threshold" => Self::Threshold,
            "degree" => Self::Degree,
            "crowd" => Self::Crowd,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ThresholdsInspection {
    pub site: ThresholdsCell,
    /// `time`, `figure` (Granovetter's Fig. 1), `histogram` or `actors`;
    /// null between panels.
    pub panel: Option<&'static str>,
    /// The step of a time-panel column.
    pub step: Option<u64>,
    /// Each crowd's share acting at that step.
    pub crowds: Option<Vec<f64>>,
    /// The share a Figure 1 column or histogram row stands for.
    pub share: Option<f64>,
    /// Figure 1: the share whose threshold is at most that share.
    pub cdf: Option<f64>,
    /// Histogram: the episodes that ended there.
    pub count: Option<u32>,
    /// The actor at a grid cell.
    pub member: Option<ActorView>,
    /// Always null: cells are read where they are.
    pub agent: Option<ActorView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ThresholdsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActorView {
    pub id: u64,
    /// Its threshold, as a fraction (null: never acts).
    pub threshold: Option<f64>,
    /// The level above which it stops, if it has a ceiling.
    pub ceiling: Option<f64>,
    /// Whom it watches: its neighbors or friends (the whole crowd: null).
    pub degree: Option<u32>,
    /// The others acting it sees (weighted by friendship), and of how many.
    pub sees: u64,
    pub of: u64,
    pub acting: bool,
    /// Switched on to start the episode.
    pub seed: bool,
    /// Its crowd, from 1.
    pub crowd: u32,
}

/// One step of the time panel: whether an episode began there, and each
/// crowd's share acting.
#[derive(Clone, Debug, PartialEq)]
struct Column {
    start: bool,
    shares: Vec<f64>,
}

#[derive(Clone)]
pub struct ThresholdsWorld {
    pub config: ThresholdsConfig,
    /// Completed steps.
    pub tick: u64,
    rng: SimRng,
    th: Vec<Th>,
    ceiling: Vec<bool>,
    /// Whom each actor watches (friends or neighbors; empty under
    /// `everyone` without friends), and, when ties are one-way, who watches it.
    sees: Arc<Vec<Vec<u32>>>,
    watched_by: Option<Arc<Vec<Vec<u32>>>>,
    crowd_of: Vec<u32>,
    crowd_size: Vec<u32>,
    crowd_acting: Vec<u32>,
    acting: Vec<bool>,
    seed: Vec<bool>,
    /// For each actor, how many of those it watches act.
    seen_acting: Vec<u32>,
    total: u32,
    step: u32,
    settled: bool,
    /// This episode's acting count after each step (Figure 1's staircase).
    path: Vec<u32>,
    recent: VecDeque<Column>,
    window: VecDeque<u32>,
    /// Finished episodes by final share (percent).
    sizes: [u32; 101],
    episodes: u32,
    size_sum: f64,
    globals: u32,
    last_size: f64,
    theory: f64,
    pub stats: Stats<ThresholdsSnapshot>,
}

/// Granovetter's continuous equilibrium share, when the crowd is a normal
/// one seen whole.
fn theory_share(c: &ThresholdsConfig) -> f64 {
    if c.distribution == Distribution::Normal
        && c.network == Network::Everyone
        && !c.friends.enabled
        && c.population == Population::Fixed
        && !c.reversible()
    {
        theory::granovetter(c.actors, c.mean, c.sd) / f64::from(c.actors)
    } else {
        f64::NAN
    }
}

impl ThresholdsWorld {
    pub fn new(config: ThresholdsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let n = config.population_size() as usize;
        let mut world = ThresholdsWorld {
            theory: theory_share(&config),
            config,
            tick: 0,
            rng: rng::seeded(seed),
            th: Vec::new(),
            ceiling: vec![false; n],
            sees: Arc::new(Vec::new()),
            watched_by: None,
            crowd_of: vec![0; n],
            crowd_size: Vec::new(),
            crowd_acting: Vec::new(),
            acting: vec![false; n],
            seed: vec![false; n],
            seen_acting: vec![0; n],
            total: 0,
            step: 0,
            settled: false,
            path: Vec::new(),
            recent: VecDeque::new(),
            window: VecDeque::new(),
            sizes: [0; 101],
            episodes: 0,
            size_sum: 0.0,
            globals: 0,
            last_size: 0.0,
            stats: Stats::default(),
        };
        world.begin();
        world.remember(true);
        world.record();
        Ok(world)
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    /// Actors in the world.
    pub fn size(&self) -> usize {
        self.acting.len()
    }

    pub fn thresholds(&self) -> &[Th] {
        &self.th
    }

    pub fn acting(&self) -> &[bool] {
        &self.acting
    }

    /// Whom each actor watches (empty lists under `everyone` without friends).
    pub fn watches(&self) -> &[Vec<u32>] {
        &self.sees
    }

    /// Finished episodes by final share, in percent.
    pub fn sizes(&self) -> &[u32; 101] {
        &self.sizes
    }

    /// Starts an episode: draws the crowd (and its friends or network),
    /// the ceilings and the seed.
    fn begin(&mut self) {
        let c = self.config.clone();
        let n = c.population_size();
        let nu = n as usize;
        self.th = match c.population {
            Population::City => crowd::city(n, &mut self.rng),
            Population::Fixed => crowd::draw(&c, &mut self.rng),
        };
        self.ceiling = vec![false; nu];
        let q = (c.ceilings.share * f64::from(n)).round() as usize;
        if q > 0 {
            let mut order: Vec<u32> = (0..n).collect();
            for i in 0..q.min(nu) {
                let j = i + self.rng.gen_range(0..(nu - i) as u32) as usize;
                order.swap(i, j);
                self.ceiling[order[i] as usize] = true;
            }
        }
        self.watched_by = None;
        self.sees = Arc::new(match c.network {
            Network::Everyone if c.friends.enabled => {
                let a = c.friends.acquaintance;
                if c.friends.symmetric {
                    graph::gnp_sparse(nu, a, &mut self.rng)
                } else {
                    let mut sees = vec![Vec::new(); nu];
                    let mut by = vec![Vec::new(); nu];
                    for (i, row) in sees.iter_mut().enumerate() {
                        for (j, watchers) in by.iter_mut().enumerate() {
                            if i != j && self.rng.gen::<f64>() < a {
                                row.push(j as u32);
                                watchers.push(i as u32);
                            }
                        }
                    }
                    self.watched_by = Some(Arc::new(by));
                    sees
                }
            }
            Network::Everyone => Vec::new(),
            Network::Random => graph::gnp_sparse(nu, c.degree / f64::from(n - 1), &mut self.rng),
            Network::PowerLaw => {
                let p = crowd::power_law(c.degree, (n - 1).min(10_000));
                let d = crowd::degrees(&p, n, &mut self.rng);
                graph::configuration(&d, &mut self.rng)
            }
        });
        let k = if c.clusters.enabled {
            c.clusters.count
        } else {
            1
        };
        self.crowd_of = (0..n).map(|i| i / c.actors).collect();
        self.crowd_size = vec![c.actors; k as usize];
        self.crowd_acting = vec![0; k as usize];
        self.acting = vec![false; nu];
        self.seed = vec![false; nu];
        self.seen_acting = vec![0; nu];
        self.total = 0;
        let start = match c.trigger {
            Trigger::Instigators => None,
            Trigger::Random => Some(self.rng.gen_range(0..n) as usize),
            Trigger::Hub => (0..nu).rev().max_by_key(|&i| self.sees[i].len()),
        };
        if let Some(s) = start {
            self.seed[s] = true;
            self.set(s, true);
        }
        self.step = 0;
        self.settled = false;
        self.path = vec![self.total];
    }

    fn set(&mut self, i: usize, on: bool) {
        if self.acting[i] == on {
            return;
        }
        self.acting[i] = on;
        let d: i64 = if on { 1 } else { -1 };
        self.total = (i64::from(self.total) + d) as u32;
        let k = self.crowd_of[i] as usize;
        self.crowd_acting[k] = (i64::from(self.crowd_acting[k]) + d) as u32;
        let watchers = match &self.watched_by {
            Some(by) => Arc::clone(by),
            None => Arc::clone(&self.sees),
        };
        if !watchers.is_empty() {
            for &j in &watchers[i] {
                let j = j as usize;
                self.seen_acting[j] = (i64::from(self.seen_acting[j]) + d) as u32;
            }
        }
    }

    /// What actor `i` perceives: the others acting (A) and the group (G),
    /// weighted by friendship, as integers.
    fn perceived(&self, i: usize) -> (u64, u64) {
        let c = &self.config;
        let me = u64::from(self.acting[i]);
        match c.network {
            Network::Random | Network::PowerLaw => {
                (u64::from(self.seen_acting[i]), self.sees[i].len() as u64)
            }
            Network::Everyone => {
                let k = self.crowd_of[i] as usize;
                let others = u64::from(self.crowd_acting[k]) - me;
                let size = u64::from(self.crowd_size[k]);
                let group = if c.counts_self { size } else { size - 1 };
                if c.friends.enabled {
                    let w = u64::from(c.friends.weight);
                    let fr = u64::from(self.seen_acting[i]);
                    let nf = self.sees[i].len() as u64;
                    (w * fr + (others - fr), w * nf + (group - nf))
                } else {
                    (others, group)
                }
            }
        }
    }

    /// Whether actor `i` would act now.
    fn decide(&self, i: usize) -> bool {
        let t = self.th[i];
        let (a, g) = self.perceived(i);
        let zero = t.num == 0;
        let on = if g == 0 {
            zero && self.config.zero == Zero::Acts
        } else if zero {
            self.config.zero == Zero::Acts || a >= 1
        } else {
            t.reached(a, g)
        };
        if on && self.ceiling[i] && g > 0 {
            !Th::from_fraction(self.config.ceilings.at).exceeded(a, g)
        } else {
            on
        }
    }

    /// The state actor `i` takes this step.
    fn next(&self, i: usize) -> bool {
        if self.seed[i] {
            true
        } else if self.config.reversible() {
            self.decide(i)
        } else {
            self.acting[i] || self.decide(i)
        }
    }

    /// Clusters: each actor moves to a random other crowd with probability m.
    fn move_about(&mut self) {
        let k = self.config.clusters.count;
        let m = self.config.clusters.movement;
        if m <= 0.0 {
            return;
        }
        for i in 0..self.acting.len() {
            if self.rng.gen::<f64>() >= m {
                continue;
            }
            let from = self.crowd_of[i];
            let mut to = self.rng.gen_range(0..k - 1);
            if to >= from {
                to += 1;
            }
            self.crowd_size[from as usize] -= 1;
            self.crowd_size[to as usize] += 1;
            if self.acting[i] {
                self.crowd_acting[from as usize] -= 1;
                self.crowd_acting[to as usize] += 1;
            }
            self.crowd_of[i] = to;
        }
    }

    /// The episode is over: record its size.
    fn finish(&mut self) {
        self.settled = true;
        let n = self.acting.len() as f64;
        let share = f64::from(self.total) / n;
        self.last_size = share;
        self.size_sum += share;
        self.episodes += 1;
        if share >= self.config.global {
            self.globals += 1;
        }
        self.sizes[(share * 100.0).round() as usize] += 1;
    }

    pub fn step(&mut self) {
        let mut start = false;
        if self.settled {
            if self.config.repeat {
                self.begin();
                start = true;
            }
        } else {
            if self.config.clusters.enabled {
                self.move_about();
            }
            let n = self.acting.len();
            let mut changed = false;
            match self.config.update {
                Update::Synchronous => {
                    let next: Vec<bool> = (0..n).map(|i| self.next(i)).collect();
                    for (i, on) in next.into_iter().enumerate() {
                        if on != self.acting[i] {
                            self.set(i, on);
                            changed = true;
                        }
                    }
                }
                Update::Asynchronous => {
                    let mut order: Vec<u32> = (0..n as u32).collect();
                    for i in (1..n).rev() {
                        let j = self.rng.gen_range(0..=i as u32) as usize;
                        order.swap(i, j);
                    }
                    for i in order {
                        let i = i as usize;
                        let on = self.next(i);
                        if on != self.acting[i] {
                            self.set(i, on);
                            changed = true;
                        }
                    }
                }
            }
            self.step += 1;
            if self.path.len() < 10_000 {
                self.path.push(self.total);
            }
            if !self.config.clusters.enabled && (!changed || self.step >= self.config.max_steps) {
                self.finish();
            }
        }
        self.tick += 1;
        self.remember(start);
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

    fn remember(&mut self, start: bool) {
        if self.recent.len() == SHOWN {
            self.recent.pop_front();
        }
        let shares = self
            .crowd_acting
            .iter()
            .zip(&self.crowd_size)
            .map(|(&a, &s)| {
                if s == 0 {
                    0.0
                } else {
                    f64::from(a) / f64::from(s)
                }
            })
            .collect();
        self.recent.push_back(Column { start, shares });
        if self.window.len() == RECENT {
            self.window.pop_front();
        }
        self.window.push_back(self.total);
    }

    fn record(&mut self) {
        let n = self.acting.len() as f64;
        let (lo, hi) = self
            .window
            .iter()
            .fold((u32::MAX, 0), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        let per = |x: u32| f64::from(x) / n;
        let e = f64::from(self.episodes);
        let s = ThresholdsSnapshot {
            tick: self.tick,
            acting: per(self.total),
            step: self.step,
            episodes: self.episodes,
            last_size: self.last_size,
            mean_size: if self.episodes == 0 {
                0.0
            } else {
                self.size_sum / e
            },
            global_share: if self.episodes == 0 {
                0.0
            } else {
                f64::from(self.globals) / e
            },
            theory: self.theory,
            recent_mean: self.window.iter().map(|&v| per(v)).sum::<f64>()
                / self.window.len() as f64,
            swing: per(hi - lo),
        };
        self.stats.push(s);
    }

    /// Whether the middle panel draws Granovetter's Figure 1: one fixed
    /// crowd, seen whole.
    fn figure_one(&self) -> bool {
        let c = &self.config;
        c.network == Network::Everyone
            && !c.friends.enabled
            && !c.clusters.enabled
            && c.population == Population::Fixed
    }

    /// The share of actors whose threshold is at most `x`.
    fn cdf(&self, x: f64) -> f64 {
        let n = self.th.len();
        let (num, den) = ((x * 1e6).round() as u64, 1_000_000u64);
        let below = self
            .th
            .iter()
            .filter(|t| u128::from(t.num) * u128::from(den) <= u128::from(num) * u128::from(t.den))
            .count();
        below as f64 / n as f64
    }

    fn view(&self, i: usize) -> ActorView {
        let (a, g) = self.perceived(i);
        let t = self.th[i];
        ActorView {
            id: i as u64 + 1,
            threshold: (t.num <= t.den).then(|| t.value()),
            ceiling: self.ceiling[i].then_some(self.config.ceilings.at),
            degree: (!self.sees.is_empty()).then(|| self.sees[i].len() as u32),
            sees: a,
            of: g,
            acting: self.acting[i],
            seed: self.seed[i],
            crowd: self.crowd_of[i] + 1,
        }
    }

    fn color(&self, mode: ThresholdsMode, i: usize, most: usize) -> [u8; 3] {
        match mode {
            ThresholdsMode::State => {
                if self.seed[i] {
                    SEED
                } else if self.acting[i] {
                    ACTING
                } else {
                    IDLE
                }
            }
            ThresholdsMode::Threshold => {
                let t = self.th[i];
                if t.num > t.den {
                    HIGH
                } else {
                    scale(t.value(), LOW, HIGH)
                }
            }
            ThresholdsMode::Degree => {
                let d = if self.sees.is_empty() {
                    0
                } else {
                    self.sees[i].len()
                };
                scale(
                    if most > 0 {
                        d as f64 / most as f64
                    } else {
                        0.0
                    },
                    FEW,
                    MANY,
                )
            }
            ThresholdsMode::Crowd => CROWDS[self.crowd_of[i] as usize % CROWDS.len()],
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<ThresholdsInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let mut out = ThresholdsInspection {
            site: ThresholdsCell { x, y },
            panel: None,
            step: None,
            crowds: None,
            share: None,
            cdf: None,
            count: None,
            member: None,
            agent: None,
        };
        if cx < TIME_W {
            if let Some(column) = self.recent.get(cx) {
                out.panel = Some("time");
                out.step = Some(self.tick + 1 - self.recent.len() as u64 + cx as u64);
                out.crowds = Some(column.shares.clone());
            }
        } else if (MID_X..MID_X + MID_W).contains(&cx) {
            if self.figure_one() {
                let share = (cx - MID_X) as f64 / (MID_W - 1) as f64;
                out.panel = Some("figure");
                out.share = Some(share);
                out.cdf = Some(self.cdf(share));
            } else {
                let share = share_at(cy);
                out.panel = Some("histogram");
                out.share = Some(share);
                out.count = Some(
                    (0..=100)
                        .filter(|&p| row(p as f64 / 100.0) == cy)
                        .map(|p| self.sizes[p])
                        .sum(),
                );
            }
        } else if cx >= GRID_X {
            let n = self.acting.len();
            let (side, cell) = grid(n as u32);
            let (gx, gy) = ((cx - GRID_X) / cell, cy / cell);
            let i = gy * side + gx;
            if gx < side && i < n {
                out.panel = Some("actors");
                out.member = Some(self.view(i));
            }
        }
        Ok(out)
    }
}

impl Model for ThresholdsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Thresholds(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        ThresholdsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.acting.len()
    }

    /// FNV-1a over the tick, every actor's threshold, crowd and state, and the
    /// episodes' record.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.step.to_le_bytes());
        for i in 0..self.acting.len() {
            eat(&self.th[i].num.to_le_bytes());
            eat(&self.th[i].den.to_le_bytes());
            eat(&self.crowd_of[i].to_le_bytes());
            eat(&[
                u8::from(self.acting[i]),
                u8::from(self.seed[i]),
                u8::from(self.ceiling[i]),
            ]);
        }
        for s in self.sees.iter() {
            eat(&(s.len() as u32).to_le_bytes());
        }
        eat(&self.episodes.to_le_bytes());
        eat(&self.globals.to_le_bytes());
        eat(&self.size_sum.to_bits().to_le_bytes());
        h
    }

    fn size(&self) -> (u32, u32) {
        let (side, cell) = grid(self.acting.len() as u32);
        ((GRID_X + side * cell) as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ThresholdsMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        // Time: each crowd's share acting; episode starts marked by a tick at the top.
        let crowds = self.crowd_size.len();
        for (k, column) in self.recent.iter().enumerate() {
            if column.start && k > 0 {
                c.column(k, 0, 6, MARK);
            }
        }
        for s in (0..crowds).rev() {
            let color = if crowds == 1 {
                LINE
            } else {
                CROWDS[s % CROWDS.len()]
            };
            for (k, column) in self.recent.iter().enumerate() {
                let y = row(column.shares[s]);
                let to = self
                    .recent
                    .get(k + 1)
                    .filter(|next| !next.start)
                    .map_or(y, |next| row(next.shares[s]));
                c.column(k, y, to, color);
            }
        }
        if self.figure_one() {
            // Granovetter's Figure 1: F against the 45° line, and the staircase.
            for x in 0..MID_W {
                let share = x as f64 / (MID_W - 1) as f64;
                c.put(MID_X + x, row(share), MARK);
                c.put(MID_X + x, row(self.cdf(share)), BAR);
            }
            let n = self.acting.len() as f64;
            for w in self.path.windows(2) {
                let (a, b) = (f64::from(w[0]) / n, f64::from(w[1]) / n);
                c.column(MID_X + col(a), row(a), row(b), PATH);
            }
        } else {
            let mut per_row = vec![0u32; TALL];
            for (p, &k) in self.sizes.iter().enumerate() {
                per_row[row(p as f64 / 100.0)] += k;
            }
            let top = per_row.iter().copied().max().unwrap_or(0).max(1);
            for (y, &k) in per_row.iter().enumerate() {
                let len = (f64::from(k) / f64::from(top) * (MID_W - 1) as f64).round() as usize;
                for x in 0..len {
                    c.put(MID_X + x, y, BAR);
                }
            }
            let g = row(self.config.global);
            for x in 0..MID_W {
                c.put(MID_X + x, g, MARK);
            }
        }
        let n = self.acting.len();
        let (side, cell) = grid(n as u32);
        let most = if self.sees.is_empty() {
            0
        } else {
            self.sees.iter().map(Vec::len).max().unwrap_or(0)
        };
        for i in 0..n {
            let color = self.color(mode, i, most);
            let (gx, gy) = (i % side, i / side);
            for dy in 0..cell {
                for dx in 0..cell {
                    c.put(GRID_X + gx * cell + dx, gy * cell + dy, color);
                }
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
        let mut out = String::from("id,threshold,ceiling,degree,sees,of,acting,seed,crowd\n");
        for i in 0..self.acting.len() {
            let v = self.view(i);
            let opt = |x: Option<f64>| x.map_or(String::new(), |x| x.to_string());
            writeln!(
                out,
                "{},{},{},{},{},{},{},{},{}",
                v.id,
                opt(v.threshold),
                opt(v.ceiling),
                v.degree.map_or(String::new(), |d| d.to_string()),
                v.sees,
                v.of,
                u8::from(v.acting),
                u8::from(v.seed),
                v.crowd
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
        let ModelConfig::Thresholds(next) = next else {
            return Err(wrong_model(ModelKind::Thresholds, &next));
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

    /// Stopped at `stop_at`: a sweep reads the run at its last step.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thresholds::config::{Ceilings, Clusters, Crowd, Friends, Rounding};

    fn config(edit: impl FnOnce(&mut ThresholdsConfig)) -> ThresholdsConfig {
        let mut c = ThresholdsConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut ThresholdsConfig)) -> ThresholdsWorld {
        ThresholdsWorld::new(config(edit), 1).unwrap()
    }

    /// Runs to the end of the first episode.
    fn settle(w: &mut ThresholdsWorld) -> u32 {
        for _ in 0..100_000 {
            if w.settled {
                break;
            }
            w.step();
        }
        w.total
    }

    /// The incrementally kept counts match a recount.
    fn consistent(w: &ThresholdsWorld) {
        let total = w.acting.iter().filter(|&&a| a).count() as u32;
        assert_eq!(w.total, total);
        for k in 0..w.crowd_size.len() {
            let members = (0..w.acting.len()).filter(|&i| w.crowd_of[i] as usize == k);
            assert_eq!(w.crowd_size[k] as usize, members.clone().count());
            assert_eq!(
                w.crowd_acting[k] as usize,
                members.filter(|&i| w.acting[i]).count()
            );
        }
        if !w.sees.is_empty() {
            for i in 0..w.acting.len() {
                let seen = w.sees[i].iter().filter(|&&j| w.acting[j as usize]).count();
                assert_eq!(w.seen_acting[i] as usize, seen, "actor {i}");
            }
        }
    }

    #[test]
    fn the_uniform_crowd_riots_to_the_last_person_and_the_perturbed_stops_at_one() {
        for n in [100, 37, 1000] {
            let mut w = world(|c| c.actors = n);
            assert_eq!(settle(&mut w), n, "N {n}: exact comparisons all the way up");
        }
        let mut p = world(|c| c.distribution = Distribution::Perturbed);
        assert_eq!(settle(&mut p), 1);
        // One person per step: the bandwagon.
        let mut u = world(|_| {});
        u.run(5);
        assert_eq!(u.total, 5);
    }

    #[test]
    fn granovetter_s_friend_example_perceives_63_of_120() {
        // 48 of 100 act; our actor knows 20, 15 of them acting, friends count 2.
        let mut w = world(|c| {
            c.friends = Friends {
                enabled: true,
                acquaintance: 0.0,
                ..Friends::default()
            }
        });
        let me = 99;
        let friends: Vec<u32> = (0..20).collect();
        let mut sees = vec![Vec::new(); 100];
        sees[me] = friends.clone();
        for &f in &friends {
            sees[f as usize].push(me as u32);
        }
        w.sees = Arc::new(sees);
        for i in 0..100 {
            w.acting[i] = false;
        }
        w.total = 0;
        w.crowd_acting = vec![0];
        w.seen_acting = vec![0; 100];
        for i in (0..15).chain(20..53) {
            w.set(i, true);
        }
        assert_eq!(w.total, 48);
        assert_eq!(w.perceived(me), (63, 120));
        w.th[me] = Th::people(50, 100);
        assert!(w.decide(me), "0.525 is above his 50 %");
    }

    #[test]
    fn thresholds_count_oneself_or_not() {
        // With 49 of 100 others... an actor at 50 % among 99 others needs 50
        // of 100 counting himself, 50 of 99 not.
        let mut w = world(|_| {});
        for i in 0..100 {
            w.acting[i] = false;
        }
        w.total = 0;
        w.crowd_acting = vec![0];
        for i in 0..49 {
            w.set(i, true);
        }
        w.th[99] = Th::people(49, 100);
        assert!(w.decide(99), "49 of 100");
        w.th[99] = Th::from_fraction(0.4945);
        assert!(!w.decide(99), "49 of 100 is below 49.45 %");
        w.config.counts_self = false;
        assert!(w.decide(99), "49 of 99 is above 49.45 %");
    }

    #[test]
    fn normal_crowds_tip_where_the_rounding_puts_them() {
        let tip = |sd: f64, rounding| {
            let mut w = world(|c| {
                c.distribution = Distribution::Normal;
                c.sd = sd;
                c.rounding = rounding;
            });
            settle(&mut w)
        };
        assert!(tip(0.122, Rounding::Nearest) < 10);
        assert!(tip(0.123, Rounding::Nearest) > 90);
        assert!(
            tip(0.12, Rounding::Floor) > 90,
            "rounding down tips earlier"
        );
        assert!(
            tip(0.124, Rounding::Exact) < 10,
            "exact fractions tip later"
        );
        let w = world(|c| {
            c.distribution = Distribution::Normal;
            c.sd = 0.122;
        });
        let s = w.stats.latest().unwrap();
        assert!((s.theory * 100.0 - 5.5).abs() < 0.5, "{}", s.theory);
    }

    #[test]
    fn city_crowds_differ_each_episode() {
        let mut w = world(|c| {
            c.population = Population::City;
            c.repeat = true;
        });
        w.run(3000);
        assert!(w.episodes > 100);
        let small = w.sizes[0] + w.sizes[1];
        let share = f64::from(small) / f64::from(w.episodes);
        assert!((0.35..0.65).contains(&share), "{share}");
        consistent(&w);
    }

    #[test]
    fn friends_symmetric_or_one_way_keep_counts() {
        for symmetric in [true, false] {
            let mut w = world(|c| {
                c.friends = Friends {
                    enabled: true,
                    symmetric,
                    ..Friends::default()
                };
                c.repeat = true;
            });
            w.run(300);
            consistent(&w);
            assert_eq!(w.watched_by.is_some(), !symmetric);
        }
    }

    #[test]
    fn watts_s_networks_and_triggers() {
        let mut w = world(|c| {
            c.actors = 2000;
            c.distribution = Distribution::Fixed;
            c.mean = 0.18;
            c.network = Network::Random;
            c.degree = 3.0;
            c.trigger = Trigger::Random;
            c.update = Update::Asynchronous;
            c.repeat = true;
        });
        let links = w.sees.iter().map(Vec::len).sum::<usize>() as f64 / 2000.0;
        assert!((links - 3.0).abs() < 0.2, "{links}");
        assert_eq!(w.seed.iter().filter(|&&s| s).count(), 1);
        w.run(400);
        consistent(&w);
        assert!(w.episodes > 5);
        let hub = world(|c| {
            c.actors = 500;
            c.network = Network::Random;
            c.trigger = Trigger::Hub;
        });
        let s = (0..500).find(|&i| hub.seed[i]).unwrap();
        let most = hub.sees.iter().map(Vec::len).max().unwrap();
        assert_eq!(hub.sees[s].len(), most);
        let p = world(|c| {
            c.actors = 3000;
            c.network = Network::PowerLaw;
            c.degree = 1.5;
        });
        let mean = p.sees.iter().map(Vec::len).sum::<usize>() as f64 / 3000.0;
        assert!((mean - 1.5).abs() < 0.2, "{mean}");
    }

    #[test]
    fn zero_thresholds_act_at_once_or_when_reached() {
        let isolated = |zero| {
            let mut w = world(|c| {
                c.actors = 50;
                c.distribution = Distribution::Fixed;
                c.mean = 0.0;
                c.network = Network::Random;
                c.degree = 0.0;
                c.zero = zero;
            });
            settle(&mut w)
        };
        assert_eq!(isolated(Zero::Acts), 50, "0 ≥ 0 read literally");
        assert_eq!(isolated(Zero::WhenReached), 0, "nobody to reach them");
    }

    #[test]
    fn synchronous_and_asynchronous_reach_the_same_equilibrium() {
        for d in [Distribution::Uniform, Distribution::Perturbed] {
            let mut a = world(|c| c.distribution = d);
            let mut b = world(|c| {
                c.distribution = d;
                c.update = Update::Asynchronous;
            });
            assert_eq!(settle(&mut a), settle(&mut b));
        }
    }

    #[test]
    fn ceilings_make_most_riots_pulse_under_synchronous_updating() {
        // Whether a crowd pulses depends on who holds the ceilings: with 10 %
        // leaving above 90 %, most of 20 crowds never settle; those that do
        // rest at 91.
        let mut pulsing = 0;
        for seed in 1..=20 {
            let mut w = ThresholdsWorld::new(
                config(|c| {
                    c.ceilings = Ceilings {
                        share: 0.1,
                        at: 0.9,
                    }
                }),
                seed,
            )
            .unwrap();
            w.run(600);
            if w.settled {
                assert_eq!(w.total, 91, "seed {seed}");
            } else {
                pulsing += 1;
                // It swings by about the share holding ceilings (83 to 92).
                assert!(w.stats.latest().unwrap().swing > 0.05, "seed {seed}");
            }
        }
        assert!(pulsing >= 12, "{pulsing} of 20");
        // Asynchronous updating hovers near the ceiling instead.
        let mut a = world(|c| {
            c.ceilings = Ceilings {
                share: 0.1,
                at: 0.9,
            };
            c.update = Update::Asynchronous;
        });
        a.run(300);
        assert!(a.stats.latest().unwrap().recent_mean > 0.85);
    }

    #[test]
    fn clusters_move_and_rioters_can_stop() {
        let mut w = world(|c| {
            c.population = Population::City;
            c.clusters = Clusters {
                enabled: true,
                count: 10,
                movement: 0.05,
            };
        });
        let before = w.crowd_of.clone();
        w.run(50);
        consistent(&w);
        assert_ne!(before, w.crowd_of);
        assert!(!w.settled, "clusters are one long episode");
        assert_eq!(w.crowd_size.iter().sum::<u32>(), 1000);
    }

    #[test]
    fn episodes_record_sizes_and_global_shares() {
        let mut w = world(|c| {
            c.population = Population::City;
            c.repeat = true;
            c.global = 0.5;
        });
        w.run(2000);
        let s = w.stats.latest().unwrap().clone();
        assert_eq!(s.episodes, w.episodes);
        assert_eq!(w.sizes.iter().sum::<u32>(), w.episodes);
        let big: u32 = w.sizes[50..].iter().sum();
        assert!((s.global_share - f64::from(big) / f64::from(w.episodes)).abs() < 0.02);
        assert!(s.mean_size > 0.0 && s.mean_size < 0.5);
        assert!(s.theory.is_nan());
        assert!(Model::latest_json(&w).contains("\"theory\":null"));
        let mut once = world(|_| {});
        once.run(300);
        assert_eq!(
            (once.episodes, once.total),
            (1, 100),
            "without repeat the world rests"
        );
    }

    #[test]
    fn the_frame_draws_time_figure_or_histogram_and_actors() {
        let mut w = world(|_| {});
        w.run(30);
        let mut buf = Vec::new();
        w.render("state", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw as usize, fh as usize), (GRID_X + 200, TALL));
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * fw as usize + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        assert_eq!(px(&buf, 30, row(0.30)), LINE);
        assert_eq!(px(&buf, GRID_X, 0), ACTING);
        for mode in ["threshold", "degree", "crowd"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let mut h = world(|c| {
            c.population = Population::City;
            c.repeat = true;
        });
        h.run(100);
        h.render("state", "", &mut buf).unwrap();
    }

    #[test]
    fn inspect_reads_steps_the_figure_bins_and_actors() {
        let mut w = world(|_| {});
        w.run(5);
        let v = w.inspect(4, 0).unwrap();
        assert_eq!((v.panel, v.step), (Some("time"), Some(4)));
        let f = w.inspect((MID_X + 50) as u32, 0).unwrap();
        assert_eq!(f.panel, Some("figure"));
        assert!(
            (f.cdf.unwrap() - 0.51).abs() < 1e-9,
            "51 of 100 at or below 50 %"
        );
        let a = w.inspect(GRID_X as u32, 0).unwrap().member.unwrap();
        assert_eq!((a.id, a.threshold, a.acting), (1, Some(0.0), true));
        let mut h = world(|c| {
            c.population = Population::City;
            c.repeat = true;
        });
        h.run(200);
        let b = h.inspect(MID_X as u32, 200).unwrap();
        assert_eq!(b.panel, Some("histogram"));
        assert!(
            b.count.unwrap() > 0,
            "episodes that ended with no one acting"
        );
        assert_eq!(Model::locate(&h, 1), None);
    }

    #[test]
    fn keyframes_restore_the_world_and_its_view() {
        let c = config(|c| {
            c.population = Population::City;
            c.friends.enabled = true;
            c.repeat = true;
        });
        let mut any = crate::model::ModelWorld::new(ModelConfig::Thresholds(c.clone()), 6).unwrap();
        any.model_mut().run(20);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("state", "", &mut before).unwrap();
        any.model_mut().run(30);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("state", "", &mut after).unwrap();
        assert_eq!(before, after);
        any.model_mut().run(30);
        let mut fresh = crate::model::ModelWorld::new(ModelConfig::Thresholds(c), 6).unwrap();
        fresh.model_mut().run(50);
        assert_eq!(
            any.model().fingerprint(),
            fresh.model().fingerprint(),
            "replays the same"
        );
    }

    #[test]
    fn live_edits_apply_and_the_crowd_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.update = Update::Asynchronous;
            c.repeat = true;
            c.stop_at = 10;
        });
        Model::set_config(&mut w, ModelConfig::Thresholds(next)).unwrap();
        w.run(100);
        assert_eq!(w.tick, 10);
        for (field, edit) in [
            ("actors", config(|c| c.actors = 101)),
            (
                "distribution",
                config(|c| c.distribution = Distribution::Perturbed),
            ),
            ("network", config(|c| c.network = Network::Random)),
            ("crowd", config(|c| c.crowd = Crowd::Sampled)),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Thresholds(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.actors = 2),
            config(|c| {
                c.distribution = Distribution::Normal;
                c.sd = 0.0;
            }),
            config(|c| {
                c.distribution = Distribution::Normal;
                c.sd = 1.0;
                c.crowd = Crowd::Sampled;
                c.repeat = true;
            }),
            config(|c| {
                c.friends = Friends {
                    enabled: true,
                    acquaintance: 1.0,
                    weight: 20,
                    symmetric: false,
                };
            }),
            config(|c| c.counts_self = false),
            config(|c| {
                c.network = Network::Random;
                c.degree = 0.0;
                c.trigger = Trigger::Random;
            }),
            config(|c| {
                c.network = Network::PowerLaw;
                c.degree = 1.9;
                c.trigger = Trigger::Hub;
                c.repeat = true;
            }),
            config(|c| {
                c.ceilings = Ceilings {
                    share: 1.0,
                    at: 0.0,
                };
                c.update = Update::Asynchronous;
            }),
            config(|c| {
                c.population = Population::City;
                c.clusters = Clusters {
                    enabled: true,
                    count: 2,
                    movement: 1.0,
                };
            }),
            config(|c| c.max_steps = 1),
        ] {
            let mut w = ThresholdsWorld::new(c.clone(), 1).unwrap();
            w.run(60);
            consistent(&w);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.acting), "{c:?}");
            assert!(s.swing.is_finite() && s.recent_mean.is_finite(), "{c:?}");
            let mut buf = Vec::new();
            w.render("degree", "", &mut buf).unwrap();
        }
    }
}
````

Create `crates/sugarscape-core/src/thresholds/presets.rs` with exactly this content:

````rust
//! Granovetter's crowds and their extensions, and Watts's cascade window.

use super::config::{
    Ceilings, Clusters, Crowd, Distribution, Friends, Network, Population, Rounding,
    ThresholdsConfig, Trigger, Update, Zero,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const GRANOVETTER: &str = "Granovetter 1978, AJS 83(6)";
const WATTS: &str = "Watts 2002, PNAS 99(9)";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut ThresholdsConfig),
) -> ModelPreset {
    let mut c = ThresholdsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Thresholds(c),
    }
}

/// Granovetter's Fig. 2: 100 people, normal thresholds around 25 %.
fn normal(c: &mut ThresholdsConfig, sd: f64) {
    c.distribution = Distribution::Normal;
    c.mean = 0.25;
    c.sd = sd;
    c.rounding = Rounding::Nearest;
}

/// Watts: 10 000 nodes, φ* 0.18, one random seed, random order, repeated.
fn watts(c: &mut ThresholdsConfig, z: f64) {
    c.actors = 10_000;
    c.distribution = Distribution::Fixed;
    c.mean = 0.18;
    c.network = Network::Random;
    c.degree = z;
    c.trigger = Trigger::Random;
    c.update = Update::Asynchronous;
    c.repeat = true;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "gr-uniform",
            "Uniform thresholds, 0 to 99",
            GRANOVETTER,
            "Granovetter's crowd: 100 people milling around a square, each with a threshold — the share of the crowd he must see join before he joins a riot. Here the thresholds are 0, 1, 2, … 99 %. The instigator (threshold 0) breaks a window; that brings in the person at 1 %, the two bring in the one at 2 %, and so on: one more person each step until all 100 riot. The middle panel is his Figure 1: the share whose threshold is at most x (blue) against the 45° line, with the riot climbing the staircase (orange). Measured: 100, as Granovetter says.",
            |_| {},
        ),
        preset(
            "gr-perturbed",
            "The 1 replaced by a 2",
            GRANOVETTER,
            "The same crowd with one change: the person at 1 % is replaced by one at 2 %. By any average the two crowds are the same; but now the instigator riots alone — nobody else's threshold is reached. Granovetter: 'A demented troublemaker broke a window while a group of solid citizens looked on.' Measured: one rioter. Compare it with gr-uniform from the presets menu.",
            |c| {
                c.distribution = Distribution::Perturbed;
            },
        ),
        preset(
            "gr-normal-12",
            "Fig. 2: normal, σ = 12",
            GRANOVETTER,
            "Granovetter's Figure 2: 100 people with normally distributed thresholds, mean 25 %, spread 12 %, thresholds rounded to whole people. Below his critical spread of about 12.2, the equilibrium is a handful of rioters. Measured: 4 rioters (his continuous calculation: 4.0). The Figure 1 panel shows the c.d.f. crossing the 45° line just above the start.",
            |c| normal(c, 0.12),
        ),
        preset(
            "gr-normal-13",
            "Fig. 2: normal, σ = 13",
            GRANOVETTER,
            "The same crowd with a spread of 13 %: past the critical point the c.d.f. never crosses the 45° line low, and nearly everyone riots. Measured: 100 (the continuous calculation: 100.0). A difference of one point in the spread of a crowd's dispositions, and a riot instead of a broken window. The tipping point for a crowd of 100 depends on how the normal thresholds become people: 12.23 rounded to the nearest person (Granovetter's 12.2), 11.89 rounded down, 12.55 kept as fractions (the gr-sd sweep).",
            |c| normal(c, 0.13),
        ),
        preset(
            "gr-normal-sampled",
            "Fig. 2's crowd, sampled",
            GRANOVETTER,
            "Figure 2's crowd with σ 12.2, but each crowd drawn at random from the normal distribution, as real crowds would be, and one crowd after another. Granovetter's jump — 'a wholly discontinuous, striking qualitative effect' — is a property of the idealized distribution: drawn crowds riot past half 15 % of the time at σ 12 and 25 % at 12.5, rising smoothly (the survey, 1 000 crowds each). Measured here (10 seeds, 3 000 steps): 21 % of about 600 crowds riot past a tenth; the mean crowd ends at 21 %. The histogram of outcomes is split: most crowds end with a few rioters, a fifth with nearly everyone.",
            |c| {
                normal(c, 0.122);
                c.crowd = Crowd::Sampled;
                c.rounding = Rounding::Exact;
                c.repeat = true;
            },
        ),
        preset(
            "gr-city",
            "Crowds sampled from a city",
            GRANOVETTER,
            "Granovetter's sampled crowds: a city whose thresholds are uniform from 0 to 99 %, and crowds of 100 drawn from it. The uniform crowd riots to the last person, but a random crowd usually does not: with no instigator (37 %) nobody riots, with an instigator but nobody at 1 % (14 %) one does — 'in over half the cases (.37 + .14 = .51) the equilibrium result is either no rioters or one rioter.' Measured (the survey, 5 000 crowds): 36.9 % + 13.7 % = 50.5 %. And the whole crowd riots in only 2.3 % of them; the mean is 12 rioters. The histogram shows the pile at 0–1 and the long, thin tail.",
            |c| {
                c.population = Population::City;
                c.repeat = true;
            },
        ),
        preset(
            "gr-friends",
            "Friends count twice",
            GRANOVETTER,
            "The uniform crowd, but friends count twice: each pair are friends with probability ¼, and a person weighs what his friends do double, dividing by the whole crowd with friends counted twice (Granovetter's 63/120 example). Now the person at 1 % joins only if the instigator is his friend. Granovetter: the uniform crowd's riot of 100 'is unstable against almost any kind of social structural influence … the modal equilibrium result is one rioter.' Measured (10 seeds, 3 000 steps, about 850 crowds): the mean crowd ends at 1.5 rioters; the mode is one rioter in 8 of 10 weight-and-acquaintance settings (the survey).",
            |c| {
                c.friends = Friends {
                    enabled: true,
                    ..Friends::default()
                };
                c.repeat = true;
            },
        ),
        preset(
            "gr-friends-perturbed",
            "Perturbed crowd, friends count five times",
            GRANOVETTER,
            "The perturbed crowd, whose lone instigator leaves everyone else unmoved, with friends counting five times. When the instigator's friends include the person at 2 %, the riot can spread a little. Granovetter: the change 'rarely exceeds five to 10 rioters'. Measured (the survey, 1 000 crowds): more than one rioter in 43 % of crowds at acquaintance ¼ — against 19 % at 0.05 and 1 % at 0.5, the largest effect at a quarter, as he says — and the 95th percentile at 7 rioters. With friends counting only twice, never.",
            |c| {
                c.distribution = Distribution::Perturbed;
                c.friends = Friends {
                    enabled: true,
                    weight: 5,
                    ..Friends::default()
                };
                c.repeat = true;
            },
        ),
        preset(
            "gr-ceilings",
            "Fig. 3: 10 % leave above 90 %",
            GRANOVETTER,
            "Granovetter's Figure 3: a threshold model needs each person's net benefit to cross zero once. Some 'might join a riot when 50% of the others had but leave when the total passed 90% for fear that so large a riot would bring official reprisals.' Here a random 10 % of the uniform crowd leave once more than 90 of the others riot, and everyone decides together each step. The riot climbs past 90, the cautious leave, those near the top follow them out, and it climbs again: it pulses between 83 and 92 and never settles, in 34 of 40 crowds (the survey); in the rest it rests at 91. Which people hold the ceilings decides it. Deciding one at a time instead, it hovers near 90.",
            |c| {
                c.ceilings = Ceilings {
                    share: 0.1,
                    at: 0.9,
                };
            },
        ),
        preset(
            "gr-clusters",
            "Ten crowds, 5 % moving each step",
            GRANOVETTER,
            "Granovetter's clusters: ten crowds of 100 drawn from the uniform city, with each person moving to another crowd with probability 0.05 each step and reconsidering there — so rioters who wander into a calm crowd can stop. He asks 'what level of movement among clusters would have the most incendiary effect'. Measured (the survey, 20 runs): 39 % rioting at this movement, against 12 % with no movement and 11 % with everyone moving every step — a middling movement spreads instigators without dissolving the riots they start. The time panel shows each crowd's share in its own color.",
            |c| {
                c.population = Population::City;
                c.clusters = Clusters {
                    enabled: true,
                    count: 10,
                    movement: 0.05,
                };
            },
        ),
        preset(
            "watts-lower",
            "Fig. 3: z = 1.05",
            WATTS,
            "Watts's cascades: 10 000 people on a random network with 1.05 links each on average, every threshold 18 % of one's neighbors; one random person switched on, everyone updating in random order, again and again. Just above z = 1 the network barely holds together: most cascades are tiny, and their sizes follow a power law — Watts: cumulative slope ½. Measured (the survey, 3 000 seeds at n 1 000): slope −0.48. Here: 2.6 % of cascades reach a tenth of the network.",
            |c| watts(c, 1.05),
        ),
        preset(
            "watts-middle",
            "Fig. 2: z = 3",
            WATTS,
            "Watts at z = 3, inside his cascade window: nodes with 5 or fewer links are 'vulnerable' — one active neighbor tips them — and they percolate. Most sparks spread everywhere. Measured (10 seeds, about 175 cascades): 87 % reach a tenth of the network, and those that do cover 94 % of it — the size of the whole connected network (S = 0.94), as Watts found.",
            |c| watts(c, 3.0),
        ),
        preset(
            "watts-upper",
            "Fig. 3: z = 6.14",
            WATTS,
            "Watts at z = 6.14, near the window's upper edge: most people have too many neighbors to be tipped by one, so nearly every spark dies — and very rarely one sweeps the whole network. Watts: at this z, at n 1 000, 'a single cascade occurring in 1,000 random trials'. Measured: at n 1 000, 20 % of seeds go global (the survey) — the upper edge moves with the network's size; here, at n 10 000, 2.5 %.",
            |c| watts(c, 6.14),
        ),
        preset(
            "watts-hetero",
            "Fig. 4a: φ normal, σ = 0.1, z = 8",
            WATTS,
            "Watts's Figure 4a: thresholds normal around 18 % with spread 0.1, at z = 8, where uniform thresholds (18 % each) never cascade. With a spread, some people have low thresholds and many links, and cascades return. Measured: 83 % of cascades global (10 seeds). Watts says varied thresholds widen the window in z both ways; at the sparse end they narrow it (the survey: 10 % against 23 % at z 1.2). Thresholds at or below 0 act only once a neighbor does here; read literally (0 ≥ 0), they would all act at once.",
            |c| {
                watts(c, 8.0);
                c.distribution = Distribution::Normal;
                c.sd = 0.1;
                c.crowd = Crowd::Sampled;
                c.zero = Zero::WhenReached;
            },
        ),
        preset(
            "watts-hub",
            "The best-connected seed, z = 1.3",
            WATTS,
            "Watts's targeting: the spark is the best-connected person instead of a random one, at z = 1.3. 'The most connected nodes are far more likely than average nodes to trigger cascades'. Measured: 96 % of cascades global, against 39 % from random sparks (the watts-targeting sweep). Watts says the advantage vanishes in the dense regime; it does not (88 % against 44 % at z 5.5).",
            |c| {
                watts(c, 1.3);
                c.trigger = Trigger::Hub;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, ThresholdsConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Thresholds(c) => (p.id, c),
                _ => panic!("{} is not a thresholds preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("gr-uniform"), ThresholdsConfig::default());
        let n = find("gr-normal-13");
        assert_eq!(
            (n.distribution, n.sd, n.rounding),
            (Distribution::Normal, 0.13, Rounding::Nearest)
        );
        let w = find("watts-upper");
        assert_eq!(
            (w.actors, w.degree, w.mean, w.network),
            (10_000, 6.14, 0.18, Network::Random)
        );
        assert_eq!(find("watts-hetero").zero, Zero::WhenReached);
        assert!(find("gr-clusters").clusters.enabled);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
````

Create `crates/sugarscape-core/src/thresholds/mod.rs` with exactly this content:

````rust
//! Threshold Models (milestone 25): Granovetter, "Threshold Models of
//! Collective Behavior" (AJS 1978), with the extensions he sketches (friends,
//! crowds sampled from a city, clusters with movement, ceilings) and Watts's
//! "A Simple Model of Global Cascades on Random Networks" (PNAS 2002) as
//! presets and switches. See docs/superpowers/specs/2026-09-27-thresholds-design.md.

mod config;
mod crowd;
mod presets;
mod stats;
mod theory;
mod view;
mod world;

pub use config::{
    schema, Ceilings, Clusters, Crowd, Distribution, Friends, Network, Population, Rounding,
    ThresholdsConfig, Trigger, Update, Zero, MAX_ACTORS, POWER_LAW_CAP,
};
pub use crowd::{city, degrees, draw, inverse_normal, power_law, Th, SCALE};
pub use presets::presets;
pub use stats::{ThresholdsSnapshot, RECENT, SERIES};
pub use theory::{
    cascade_ratio, granovetter, normal_cdf, poisson, poisson_ratio, poisson_window,
    power_law_ratio, vulnerable,
};
pub use view::{grid, row, GRID_X, MID_X, SHOWN, TALL, TIME_W};
pub use world::{ActorView, ThresholdsCell, ThresholdsInspection, ThresholdsMode, ThresholdsWorld};
````

- [ ] **Step 3: Wire the model kind and title its presets**

Modify `crates/sugarscape-core/src/lib.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index 5d657b5..7741f44 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -43,6 +43,7 @@ pub mod stats;
 pub mod structure;
 pub mod sweep;
 pub mod tags;
+pub mod thresholds;
 pub mod titles;
 pub mod world;
````

Modify `crates/sugarscape-core/src/model.rs` — every match gains `Thresholds`; the reader gains its `"thresholds"` arm; a round-trip test pins it (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index d712b59..fc70618 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -25,10 +25,11 @@ use crate::schema::Param;
 use crate::spatial::{SpatialConfig, SpatialWorld};
 use crate::structure::{StructureConfig, StructureWorld};
 use crate::tags::{TagsConfig, TagsWorld};
+use crate::thresholds::{ThresholdsConfig, ThresholdsWorld};
 use crate::world::World;
 use crate::{
     agreement, anasazi, ants, civil, classes, culture, dpd, ethno, export, farol, image, norms,
-    opinions, ring, schelling, spatial, stats, structure, tags,
+    opinions, ring, schelling, spatial, stats, structure, tags, thresholds,
 };
 
 /// Which model a config or world is.
@@ -53,10 +54,11 @@ pub enum ModelKind {
     Image,
     Farol,
     Ants,
+    Thresholds,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 18] = [
+    pub const ALL: [ModelKind; 19] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -75,6 +77,7 @@ impl ModelKind {
         ModelKind::Image,
         ModelKind::Farol,
         ModelKind::Ants,
+        ModelKind::Thresholds,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -97,6 +100,7 @@ impl ModelKind {
             ModelKind::Image => "image",
             ModelKind::Farol => "farol",
             ModelKind::Ants => "ants",
+            ModelKind::Thresholds => "thresholds",
         }
     }
 
@@ -122,6 +126,7 @@ impl ModelKind {
             ModelKind::Image => image::schema(),
             ModelKind::Farol => farol::schema(),
             ModelKind::Ants => ants::schema(),
+            ModelKind::Thresholds => thresholds::schema(),
         }
     }
 }
@@ -153,6 +158,7 @@ pub enum ModelConfig {
     Image(ImageConfig),
     Farol(FarolConfig),
     Ants(AntsConfig),
+    Thresholds(ThresholdsConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -176,6 +182,7 @@ enum Tagged<'a> {
     Image(&'a ImageConfig),
     Farol(&'a FarolConfig),
     Ants(&'a AntsConfig),
+    Thresholds(&'a ThresholdsConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -206,6 +213,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Image(c) => Tagged::Image(c).serialize(s),
             ModelConfig::Farol(c) => Tagged::Farol(c).serialize(s),
             ModelConfig::Ants(c) => Tagged::Ants(c).serialize(s),
+            ModelConfig::Thresholds(c) => Tagged::Thresholds(c).serialize(s),
         }
     }
 }
@@ -231,6 +239,7 @@ impl ModelConfig {
             ModelConfig::Image(_) => ModelKind::Image,
             ModelConfig::Farol(_) => ModelKind::Farol,
             ModelConfig::Ants(_) => ModelKind::Ants,
+            ModelConfig::Thresholds(_) => ModelKind::Thresholds,
         }
     }
 
@@ -319,10 +328,13 @@ impl ModelConfig {
             "ants" => serde_json::from_value(value)
                 .map(ModelConfig::Ants)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "thresholds" => serde_json::from_value(value)
+                .map(ModelConfig::Thresholds)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol or ants)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants or thresholds)"
                 ),
             )),
         }
@@ -348,6 +360,7 @@ impl ModelConfig {
             ModelConfig::Image(c) => c.validate(),
             ModelConfig::Farol(c) => c.validate(),
             ModelConfig::Ants(c) => c.validate(),
+            ModelConfig::Thresholds(c) => c.validate(),
         }
     }
 
@@ -373,6 +386,7 @@ impl ModelConfig {
             ModelConfig::Image(c) => set_path(c, path, value).map(ModelConfig::Image),
             ModelConfig::Farol(c) => set_path(c, path, value).map(ModelConfig::Farol),
             ModelConfig::Ants(c) => set_path(c, path, value).map(ModelConfig::Ants),
+            ModelConfig::Thresholds(c) => set_path(c, path, value).map(ModelConfig::Thresholds),
         }
     }
 
@@ -397,7 +411,8 @@ impl ModelConfig {
             | ModelConfig::Norms(_)
             | ModelConfig::Agreement(_)
             | ModelConfig::Farol(_)
-            | ModelConfig::Ants(_) => None,
+            | ModelConfig::Ants(_)
+            | ModelConfig::Thresholds(_) => None,
         }
     }
 
@@ -422,6 +437,9 @@ impl ModelConfig {
             ModelConfig::Image(_) => image::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Farol(_) => farol::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Ants(_) => ants::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Thresholds(_) => {
+                thresholds::SERIES.iter().map(|s| s.to_string()).collect()
+            }
         }
     }
 }
@@ -609,6 +627,7 @@ pub enum ModelWorld {
     Image(Box<ImageWorld>),
     Farol(Box<FarolWorld>),
     Ants(Box<AntsWorld>),
+    Thresholds(Box<ThresholdsWorld>),
 }
 
 impl ModelWorld {
@@ -652,6 +671,9 @@ impl ModelWorld {
             ModelConfig::Image(c) => ModelWorld::Image(Box::new(ImageWorld::new(c, seed)?)),
             ModelConfig::Farol(c) => ModelWorld::Farol(Box::new(FarolWorld::new(c, seed)?)),
             ModelConfig::Ants(c) => ModelWorld::Ants(Box::new(AntsWorld::new(c, seed)?)),
+            ModelConfig::Thresholds(c) => {
+                ModelWorld::Thresholds(Box::new(ThresholdsWorld::new(c, seed)?))
+            }
         })
     }
 
@@ -675,6 +697,7 @@ impl ModelWorld {
             ModelWorld::Image(_) => ModelKind::Image,
             ModelWorld::Farol(_) => ModelKind::Farol,
             ModelWorld::Ants(_) => ModelKind::Ants,
+            ModelWorld::Thresholds(_) => ModelKind::Thresholds,
         }
     }
 
@@ -698,6 +721,7 @@ impl ModelWorld {
             ModelWorld::Image(w) => w.as_ref(),
             ModelWorld::Farol(w) => w.as_ref(),
             ModelWorld::Ants(w) => w.as_ref(),
+            ModelWorld::Thresholds(w) => w.as_ref(),
         }
     }
 
@@ -721,6 +745,7 @@ impl ModelWorld {
             ModelWorld::Image(w) => w.as_mut(),
             ModelWorld::Farol(w) => w.as_mut(),
             ModelWorld::Ants(w) => w.as_mut(),
+            ModelWorld::Thresholds(w) => w.as_mut(),
         }
     }
 
@@ -813,6 +838,7 @@ impl ModelWorld {
             ModelWorld::Image(w) => ModelWorld::Image(Box::new(w.keyframe())),
             ModelWorld::Farol(w) => copy_without_history!(Farol, w),
             ModelWorld::Ants(w) => copy_without_history!(Ants, w),
+            ModelWorld::Thresholds(w) => copy_without_history!(Thresholds, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -848,6 +874,9 @@ impl ModelWorld {
             (ModelWorld::Image(live), ModelWorld::Image(kept)) => restore_into!(live, kept),
             (ModelWorld::Farol(live), ModelWorld::Farol(kept)) => restore_into!(live, kept),
             (ModelWorld::Ants(live), ModelWorld::Ants(kept)) => restore_into!(live, kept),
+            (ModelWorld::Thresholds(live), ModelWorld::Thresholds(kept)) => {
+                restore_into!(live, kept)
+            }
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1200,6 +1229,30 @@ mod tests {
         assert_eq!(w.model().tick(), 0);
     }
 
+    #[test]
+    fn thresholds_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "thresholds", "distribution": "normal", "sd": 0.13, "friends": {"enabled": true}}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Thresholds);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(
+            (json["model"].as_str(), json["actors"].as_u64()),
+            (Some("thresholds"), Some(100))
+        );
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["acting", "step"]);
+        let e = ModelConfig::from_json(r#"{"model": "thresholds", "trigger": "hub"}"#).unwrap_err();
+        assert_eq!(e[0].field, "trigger");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!(w.kind(), ModelKind::Thresholds);
+        let cp = w.checkpoint().expect("thresholds worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
     #[test]
     fn only_the_anasazi_finishes() {
         let mut w = ModelWorld::new(
@@ -1241,7 +1294,8 @@ mod tests {
                 "agreement",
                 "image",
                 "farol",
-                "ants"
+                "ants",
+                "thresholds"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
````

Modify `crates/sugarscape-core/src/presets.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index d152033..7f7476b 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -828,6 +828,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::image::presets());
     out.extend(crate::farol::presets());
     out.extend(crate::ants::presets());
+    out.extend(crate::thresholds::presets());
     out
 }
````

Modify `crates/sugarscape-core/src/titles.rs` — fifteen titles (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/titles.rs b/crates/sugarscape-core/src/titles.rs
index 34f7c8c..04e24b4 100644
--- a/crates/sugarscape-core/src/titles.rs
+++ b/crates/sugarscape-core/src/titles.rs
@@ -3,7 +3,7 @@
 //! paper (`source`) stay on the preset as its reference.
 
 /// Titles by preset id, in catalog order.
-pub const TITLES: [(&str, &str); 223] = [
+pub const TITLES: [(&str, &str); 238] = [
     (
         "ii-1-instant",
         "Sugar grows back at once: agents climb the best ridges and the poorly endowed starve",
@@ -857,6 +857,66 @@ pub const TITLES: [(&str, &str); 223] = [
         "am-independent",
         "Five ants in a hundred who ignore everyone calm the whole colony",
     ),
+    (
+        "gr-uniform",
+        "One instigator, and all 100 riot",
+    ),
+    (
+        "gr-perturbed",
+        "Move one person up one notch, and only the instigator riots",
+    ),
+    (
+        "gr-normal-12",
+        "Mean threshold 25, spread 12: a handful riot",
+    ),
+    (
+        "gr-normal-13",
+        "Mean threshold 25, spread 13: nearly everyone riots",
+    ),
+    (
+        "gr-normal-sampled",
+        "The same crowd drawn from real people: no sharp tipping point",
+    ),
+    (
+        "gr-city",
+        "Crowds drawn from a city that should riot: half end with one rioter",
+    ),
+    (
+        "gr-friends",
+        "Count friends double, and the crowd that should riot mostly doesn't",
+    ),
+    (
+        "gr-friends-perturbed",
+        "Close friends rescue the stalled crowd, now and then",
+    ),
+    (
+        "gr-ceilings",
+        "Join a crowd, leave a mob: the riot builds and collapses",
+    ),
+    (
+        "gr-clusters",
+        "Ten crowds with people drifting between them",
+    ),
+    (
+        "watts-lower",
+        "Few links: mostly small cascades, now and then a large one",
+    ),
+    (
+        "watts-middle",
+        "A middling network: most sparks spread everywhere",
+    ),
+    (
+        "watts-upper",
+        "Many links: almost never, then everything",
+    ),
+    (
+        "watts-hetero",
+        "Varied thresholds keep dense networks cascading",
+    ),
+    (
+        "watts-hub",
+        "Light the best-connected node",
+    ),
 ];
 
 /// The title of preset `id`, or "" if it has none.
````

- [ ] **Step 4: Run the model's tests**

Run: `cargo test -p sugarscape-core --lib thresholds`
Expected: PASS (37, plus `model::tests::thresholds_configs_round_trip_with_their_tag` under `cargo test -p sugarscape-core --lib model`).

- [ ] **Step 5: Watch the golden check fail, then record the entries**

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: FAIL with `record a golden fingerprint for gr-uniform (run print_golden)`; every existing entry passes.

Modify `crates/sugarscape-core/tests/golden.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 94944d6..6780f6c 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -238,6 +238,22 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("am-random", 0x3e7b4009dc784788),
     ("am-scale-free", 0x9e1c3fd99aa8a637),
     ("am-independent", 0x23127da92f6506f),
+    // Milestone 25: threshold models.
+    ("gr-uniform", 0x85b7748fd9f64ab0),
+    ("gr-perturbed", 0xbea93b91e6680639),
+    ("gr-normal-12", 0xd933d5ae864ab70b),
+    ("gr-normal-13", 0xcda701fbbdaa19ce),
+    ("gr-normal-sampled", 0xa9bca3821a0f4774),
+    ("gr-city", 0xa587dcb16521cf3c),
+    ("gr-friends", 0x37ae8bba4d07be7c),
+    ("gr-friends-perturbed", 0x6acc92415fc58d6e),
+    ("gr-ceilings", 0x1cc528db96973a5a),
+    ("gr-clusters", 0xd493a3251cde5fbe),
+    ("watts-lower", 0xedb45df7d6e152de),
+    ("watts-middle", 0x1ed3157ee5e9ac61),
+    ("watts-upper", 0xea32457929c916d8),
+    ("watts-hetero", 0x1ebbdf6d37320885),
+    ("watts-hub", 0x994dd1ff0bedf61b),
 ];
 
 fn fingerprint(id: &str) -> u64 {
````

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: PASS.

- [ ] **Step 6: Format, lint and run everything**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace`
Expected: no warnings; every test passes.

- [ ] **Step 7: Commit**

```bash
git add crates/sugarscape-core/src/graph.rs crates/sugarscape-core/src/thresholds crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs crates/sugarscape-core/tests/golden.rs
```
```bash
git commit -m "Add Threshold Models (Granovetter; Watts) as a model kind

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/{gr-sd,gr-friends,gr-movement,gr-ceilings,watts-window,watts-hetero,watts-targeting}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets (`gr-normal-13`, `gr-friends-perturbed`, `gr-clusters`, `gr-ceilings`, `watts-middle`, `watts-hetero`) and series (`acting`, `global_share`, `recent_mean`, `swing`).
- Produces: seven built-in sweeps; the CLI's stop `(its last step)` for `thresholds`.

- [ ] **Step 1: Write the failing tests**

Modify `crates/sugarscape-cli/tests/cli.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index e2070fd..edf09c7 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -137,6 +137,13 @@ fn presets_and_sweeps_are_listed() {
         "ants-pull",
         "ants-sources",
         "am-independent",
+        "gr-sd",
+        "gr-friends",
+        "gr-movement",
+        "gr-ceilings",
+        "watts-window",
+        "watts-hetero",
+        "watts-targeting",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -540,6 +547,26 @@ fn an_ants_run_stops_at_its_last_step() {
     assert_eq!(stderr(&out), "finished at tick 25 (its last step)\n");
 }
 
+#[test]
+fn a_thresholds_run_stops_at_its_last_step() {
+    let dir = scratch("thresholds");
+    let config = dir.join("stop.json");
+    std::fs::write(
+        &config,
+        r#"{"model": "thresholds", "repeat": true, "stop_at": 25}"#,
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
+    assert_eq!(stderr(&out), "finished at tick 25 (its last step)\n");
+}
+
 #[test]
 fn a_social_structure_run_stops_at_its_last_period() {
     let out = sugarscape(&["run", "--preset", "cra-rwr", "--ticks", "3000"]);
````

Modify `crates/sugarscape-wasm/tests/web.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index 21caebe..09f416c 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -340,7 +340,14 @@ fn builtins_and_series_names_are_listed() {
             "ifd-matching",
             "ifd-idle",
             "ifd-crowding",
-            "ifd-travel"
+            "ifd-travel",
+            "gr-sd",
+            "gr-friends",
+            "gr-movement",
+            "gr-ceilings",
+            "watts-window",
+            "watts-hetero",
+            "watts-targeting"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -916,6 +923,28 @@ fn ants_sims_match_the_native_golden_entries() {
     }
 }
 
+#[wasm_bindgen_test]
+fn thresholds_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: normal crowds by
+    // quantiles and samples (the portable logarithm), city crowds, friends,
+    // ceilings, clusters and Watts's networks.
+    for (id, fp) in [
+        ("gr-normal-13", "0xcda701fbbdaa19ce"),
+        ("gr-normal-sampled", "0xa9bca3821a0f4774"),
+        ("gr-city", "0xa587dcb16521cf3c"),
+        ("gr-friends", "0x37ae8bba4d07be7c"),
+        ("gr-ceilings", "0x1cc528db96973a5a"),
+        ("gr-clusters", "0xd493a3251cde5fbe"),
+        ("watts-middle", "0x1ed3157ee5e9ac61"),
+        ("watts-hetero", "0x1ebbdf6d37320885"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "thresholds");
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
Expected: FAIL (`a_thresholds_run_stops_at_its_last_step`: "its end year"; `presets_and_sweeps_are_listed`: `gr-sd` missing).

- [ ] **Step 2: Write the sweeps, register them and name the stop**

Create `sweeps/gr-sd.json` with exactly this content:

````json
{
  "name": "Threshold models: Granovetter's Figure 2",
  "description": "Granovetter's Figure 2 for a crowd of 100: the share rioting at equilibrium against the spread of normal thresholds (mean 25 %). Measured (release, seeds 1–20, recorded 2026-09-27): rounded to whole people, 4 % at σ 0.12 and 5 % at 0.122, then 100 % from 0.124 — the jump at Granovetter's 12.2; kept as fractions, the jump waits until between 0.124 and 0.126; drawn at random, no jump: 1 %, 7 %, 22 %, 32 %, 32 %, 32 %, 32 %, 38 %, 76 %, 81 %, 100 % at σ 0.10 … 0.20. The critical point is a property of the idealized distribution and of how its thresholds become people.",
  "base": {
    "preset": "gr-normal-13"
  },
  "x": {
    "label": "Spread of thresholds (sd)",
    "path": "sd",
    "values": [
      0.1,
      0.11,
      0.115,
      0.12,
      0.122,
      0.124,
      0.126,
      0.13,
      0.14,
      0.16,
      0.2
    ]
  },
  "series": {
    "label": "The crowd",
    "values": [
      {
        "at": 0,
        "name": "Quantiles, rounded to people",
        "set": {
          "crowd": "quantiles",
          "rounding": "nearest"
        }
      },
      {
        "at": 1,
        "name": "Quantiles, as fractions",
        "set": {
          "crowd": "quantiles",
          "rounding": "exact"
        }
      },
      {
        "at": 2,
        "name": "Drawn at random",
        "set": {
          "crowd": "sampled",
          "rounding": "exact"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 200,
  "metric": {
    "kind": "final",
    "series": "acting"
  }
}
````

Create `sweeps/gr-friends.json` with exactly this content:

````json
{
  "name": "Threshold models: friends and the stalled crowd",
  "description": "Friends and the perturbed crowd (whose instigator riots alone): the share of crowds in which more than one person riots, against acquaintance, for friends counting two and five strangers. Measured (release, seeds 1–5, recorded 2026-09-27): weight 2, never; weight 5, 16 %, 26 %, 44 %, 0.5 %, 0 at acquaintance 0.05, 0.1, 0.25, 0.5, 0.9 — the largest effect at about a quarter, as Granovetter says.",
  "base": {
    "preset": "gr-friends-perturbed"
  },
  "set": {
    "global": 0.02
  },
  "x": {
    "label": "Acquaintance",
    "path": "friends.acquaintance",
    "values": [
      0.05,
      0.1,
      0.25,
      0.5,
      0.9
    ]
  },
  "series": {
    "label": "A friend counts as",
    "values": [
      {
        "at": 2,
        "name": "Two strangers",
        "set": {
          "friends.weight": 2
        }
      },
      {
        "at": 5,
        "name": "Five strangers",
        "set": {
          "friends.weight": 5
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 5
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "global_share"
  }
}
````

Create `sweeps/gr-movement.json` with exactly this content:

````json
{
  "name": "Threshold models: movement between crowds",
  "description": "Granovetter's clusters: ten crowds of 100 from the uniform city, the mean share rioting over the last 100 of 400 steps against how often people move between crowds. Measured (release, seeds 1–10, recorded 2026-09-27): 12 %, 20 %, 29 %, 42 %, 41 %, 33 %, 29 % at movement 0, 0.001, 0.005, 0.01, 0.05, 0.2, 1 — a middling movement is the most incendiary.",
  "base": {
    "preset": "gr-clusters"
  },
  "x": {
    "label": "Movement per step",
    "path": "clusters.movement",
    "values": [
      0,
      0.001,
      0.005,
      0.01,
      0.05,
      0.2,
      1
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 400,
  "metric": {
    "kind": "final",
    "series": "recent_mean"
  }
}
````

Create `sweeps/gr-ceilings.json` with exactly this content:

````json
{
  "name": "Threshold models: ceilings and pulsing riots",
  "description": "Ceilings (Granovetter's Figure 3): how far the uniform crowd's riot swings over the last 100 of 600 steps against the share who leave once more than 90 % of the others riot. Measured (release, seeds 1–10, recorded 2026-09-27): deciding together, 0, 0.07, 0.14, 0.16, 0.27, 0.26 at shares 0, 0.1, 0.2, 0.3, 0.5, 0.7 — the riot pulses by about the share holding ceilings, in most crowds; one at a time, 0.04 or less — it hovers near 90 %.",
  "base": {
    "preset": "gr-ceilings"
  },
  "x": {
    "label": "Share who leave above 90 %",
    "path": "ceilings.share",
    "values": [
      0,
      0.1,
      0.2,
      0.3,
      0.5,
      0.7
    ]
  },
  "series": {
    "label": "Actors decide",
    "values": [
      {
        "at": 0,
        "name": "Together",
        "set": {
          "update": "synchronous"
        }
      },
      {
        "at": 1,
        "name": "One at a time",
        "set": {
          "update": "asynchronous"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 600,
  "metric": {
    "kind": "final",
    "series": "swing"
  }
}
````

Create `sweeps/watts-window.json` with exactly this content:

````json
{
  "name": "Threshold models: Watts's cascade window",
  "description": "Watts's Figure 1: the share of single-node sparks that become global cascades (a tenth of the network or more) against mean degree, 2 000 nodes, for thresholds of 14 %, 18 % and 24 %. Measured (release, seeds 1–4, recorded 2026-09-27): at 18 %, 0 at z 0.5, 1 % at 1, 53 % at 1.5, 71 %, 88 %, 88 %, 73 % at 2–5, 19 % at 6, 0.3 % at 7, 0 at 8 — the analytic window is 1.02–5.76 for an infinite network; lower thresholds widen it (14 %: still 82 % at z 8), higher narrow it (24 %: gone by z 5).",
  "base": {
    "preset": "watts-middle"
  },
  "set": {
    "actors": 2000
  },
  "x": {
    "label": "Mean degree (z)",
    "path": "degree",
    "values": [
      0.5,
      1,
      1.5,
      2,
      3,
      4,
      5,
      6,
      7,
      8
    ]
  },
  "series": {
    "label": "Threshold (φ*)",
    "values": [
      {
        "at": 0.14,
        "name": "φ* = 0.14",
        "set": {
          "mean": 0.14
        }
      },
      {
        "at": 0.18,
        "name": "φ* = 0.18",
        "set": {
          "mean": 0.18
        }
      },
      {
        "at": 0.24,
        "name": "φ* = 0.24",
        "set": {
          "mean": 0.24
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 4
  },
  "ticks": 600,
  "metric": {
    "kind": "final",
    "series": "global_share"
  }
}
````

Create `sweeps/watts-hetero.json` with exactly this content:

````json
{
  "name": "Threshold models: varied thresholds",
  "description": "Watts's Figure 4a: global cascades against mean degree when thresholds are normal around 18 % with spread σ (thresholds at or below 0 act once a neighbor does), 2 000 nodes. Measured (release, seeds 1–4, recorded 2026-09-27): σ 0: 23 % at z 6, 0 from 8; σ 0.05: 74 % at 6, 26 % at 8; σ 0.1: 85 % at 6 and still 80 % at 12 — spread widens the window's dense side, as Watts says. But at the sparse side it narrows it: 28 %, 20 %, 9 % at z 1.2 for σ 0, 0.05, 0.1.",
  "base": {
    "preset": "watts-hetero"
  },
  "set": {
    "actors": 2000
  },
  "x": {
    "label": "Mean degree (z)",
    "path": "degree",
    "values": [
      1,
      1.2,
      1.5,
      3,
      6,
      8,
      10,
      12
    ]
  },
  "series": {
    "label": "Spread of thresholds",
    "values": [
      {
        "at": 0,
        "name": "σ = 0",
        "set": {
          "sd": 0
        }
      },
      {
        "at": 0.05,
        "name": "σ = 0.05",
        "set": {
          "sd": 0.05
        }
      },
      {
        "at": 0.1,
        "name": "σ = 0.1",
        "set": {
          "sd": 0.1
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 4
  },
  "ticks": 600,
  "metric": {
    "kind": "final",
    "series": "global_share"
  }
}
````

Create `sweeps/watts-targeting.json` with exactly this content:

````json
{
  "name": "Threshold models: lighting the hub",
  "description": "Watts on targeting: global cascades from a random spark and from the best-connected node, against mean degree, 2 000 nodes, threshold 18 %. Measured (release, seeds 1–4, recorded 2026-09-27): random 13 %, 39 %, 53 %, 88 %, 73 %, 44 %, 19 % at z 1.1, 1.3, 1.5, 3, 5, 5.5, 6; the hub 50 %, 95 %, 100 %, 100 %, 99 %, 89 %, 63 %. The hub helps most in the sparse regime, as Watts says — but it still doubles the chance in the dense one, where he says it does not.",
  "base": {
    "preset": "watts-middle"
  },
  "set": {
    "actors": 2000
  },
  "x": {
    "label": "Mean degree (z)",
    "path": "degree",
    "values": [
      1.1,
      1.3,
      1.5,
      3,
      5,
      5.5,
      6
    ]
  },
  "series": {
    "label": "Seed",
    "values": [
      {
        "at": 0,
        "name": "A random node",
        "set": {
          "trigger": "random"
        }
      },
      {
        "at": 1,
        "name": "The best-connected node",
        "set": {
          "trigger": "hub"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 4
  },
  "ticks": 600,
  "metric": {
    "kind": "final",
    "series": "global_share"
  }
}
````

Modify `crates/sugarscape-core/src/sweep.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index f54e9bf..033d288 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -965,7 +965,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 90] = [
+const BUILTINS: [Builtin; 97] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1326,6 +1326,34 @@ const BUILTINS: [Builtin; 90] = [
         id: "ifd-travel",
         json: include_str!("../../../sweeps/ifd-travel.json"),
     },
+    Builtin {
+        id: "gr-sd",
+        json: include_str!("../../../sweeps/gr-sd.json"),
+    },
+    Builtin {
+        id: "gr-friends",
+        json: include_str!("../../../sweeps/gr-friends.json"),
+    },
+    Builtin {
+        id: "gr-movement",
+        json: include_str!("../../../sweeps/gr-movement.json"),
+    },
+    Builtin {
+        id: "gr-ceilings",
+        json: include_str!("../../../sweeps/gr-ceilings.json"),
+    },
+    Builtin {
+        id: "watts-window",
+        json: include_str!("../../../sweeps/watts-window.json"),
+    },
+    Builtin {
+        id: "watts-hetero",
+        json: include_str!("../../../sweeps/watts-hetero.json"),
+    },
+    Builtin {
+        id: "watts-targeting",
+        json: include_str!("../../../sweeps/watts-targeting.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2238,7 +2266,14 @@ mod tests {
                 "ifd-matching",
                 "ifd-idle",
                 "ifd-crowding",
-                "ifd-travel"
+                "ifd-travel",
+                "gr-sd",
+                "gr-friends",
+                "gr-movement",
+                "gr-ceilings",
+                "watts-window",
+                "watts-hetero",
+                "watts-targeting"
             ]
         );
         for b in builtins() {
````

Modify `crates/sugarscape-cli/src/main.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index 1f3fe2c..797e799 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -235,7 +235,7 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             },
             ModelKind::Image => "its last generation",
             ModelKind::Farol => "its last round",
-            ModelKind::Ants => "its last step",
+            ModelKind::Ants | ModelKind::Thresholds => "its last step",
             _ => "its end year",
         };
         eprintln!("finished at tick {} ({why})", world.tick());
````

- [ ] **Step 3: Run the tests**

Run: `cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS (WASM: 48, including `thresholds_sims_match_the_native_golden_entries`).

- [ ] **Step 4: Measure the sweeps against their descriptions**

Run: `cargo build --release -p sugarscape-cli && for s in gr-sd gr-friends gr-movement gr-ceilings watts-window watts-hetero watts-targeting; do ./target/release/sugarscape sweep --builtin $s --quiet --summary-csv /tmp/$s.csv --out /dev/null; done` and read each summary's `mean` column (series names contain commas: read them with a CSV reader).
Expected: the means each description records (about 3 s in all).

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add sweeps/gr-sd.json sweeps/gr-friends.json sweeps/gr-movement.json sweeps/gr-ceilings.json sweeps/watts-window.json sweeps/watts-hetero.json sweeps/watts-targeting.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli crates/sugarscape-wasm/tests/web.rs
```
```bash
git commit -m "Measure Threshold Models: seven sweeps, the CLI's stop and WASM agreement

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/compare-presets.test.ts`, `web/src/engine.test.ts`, `web/src/experiments/form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the WASM build of Tasks 1–2.
- Produces: `ThresholdsConfig`, `ThresholdsStats`, `ActorView`, `ThresholdsInspection` (types.ts); `isThresholdsView` (models.ts; checked before `isAntsView` and `isFarolView`, whose test `panel` + `member` a thresholds inspection also passes); `MODEL_CHARTS.thresholds`; the Compare entry `gr-uniform-vs-perturbed`.

- [ ] **Step 1: Write the failing tests**

Modify `web/src/models.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index fb51371..326ce04 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -5,6 +5,7 @@ import {
   finishesUnpredictably,
   isAgreementView,
   isAntsView,
+  isThresholdsView,
   isFarolView,
   isCivilView,
   isClassesView,
@@ -118,6 +119,29 @@ describe('the presets menu', () => {
   });
 });
 
+describe('the thresholds model', () => {
+  it('is read by its tag, and its inspections by their cdf, before the ants’ and El Farol’s', () => {
+    const c = { model: 'thresholds', stop_at: 0 } as unknown as ModelConfig;
+    expect(modelOf(c)).toBe('thresholds');
+    const cell = { site: { x: 1, y: 2 }, panel: 'actors', step: null, crowds: null, share: null, cdf: null, count: null, member: null, agent: null } as unknown as AnyInspection;
+    const ants = { site: { x: 1, y: 2 }, panel: 'ants', step: null, shares: null, share: null, count: null, theory: null, member: null, agent: null } as unknown as AnyInspection;
+    expect([cell, ants].map(isThresholdsView)).toEqual([true, false]);
+    expect(isAntsView(cell)).toBe(false);
+  });
+
+  it('colors four ways, has no overlays, and stops predictably at its last step', () => {
+    expect(COLOR_MODES.thresholds).toEqual([
+      ['state', 'State'],
+      ['threshold', 'Threshold'],
+      ['degree', 'Degree'],
+      ['crowd', 'Crowd'],
+    ]);
+    expect(MODEL_OVERLAYS.thresholds).toEqual([]);
+    const c = (stop_at: number) => ({ model: 'thresholds', stop_at }) as unknown as ModelConfig;
+    expect([finishesUnpredictably(c(50)), ticksLeft(c(50), 20), ticksLeft(c(0), 20)]).toEqual([false, 30, Infinity]);
+  });
+});
+
 describe('the ants model', () => {
   it('is read by its tag, and its inspections by their shares', () => {
     const c = { model: 'ants', stop_at: 0 } as unknown as ModelConfig;
````

Modify `web/src/compare-presets.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index ce1c8c8..e82f5ff 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -43,6 +43,11 @@ describe('compare presets', () => {
     expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
   });
 
+  it('pairs Granovetter’s uniform and perturbed crowds', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['gr-uniform-vs-perturbed', 'gr-uniform', 'gr-perturbed', 'Uniform vs perturbed crowd — Threshold Models (Compare)']);
+  });
+
   it('pairs Kirman’s colony with ten times the ants', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['ants-colony-size', 'ants-2b', 'ants-crowd', 'Colony size — Ants (Compare)']);
````

Modify `web/src/engine.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index 96572a0..ffaba8a 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1153,6 +1153,7 @@ describe('Engine with other models', () => {
     expect(finishedNotice({ model: 'norms' } as unknown as ModelConfig, 100)).toBe('This run has reached its last generation (100) — Reset to run it again');
     expect(finishedNotice({ model: 'farol', stop_at: 100 } as unknown as ModelConfig, 100)).toBe('This run has reached its last round (100) — Reset to run it again');
     expect(finishedNotice({ model: 'ants', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last step (2000) — Reset to run it again');
+    expect(finishedNotice({ model: 'thresholds', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last step (50) — Reset to run it again');
     expect(finishedNotice({ model: 'agreement', stop_at: 200 } as unknown as ModelConfig, 200)).toBe('This run has reached its last period (200) — Reset to run it again');
     expect(finishedNotice({ model: 'agreement', stop_at: 20000 } as unknown as ModelConfig, 376)).toBe(
       'Stable at t = 376: no opinion or uncertainty moves any more — Reset, or change the rule, to run it again',
````

Modify `web/src/experiments/form.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index 8f9aacf..43fea0b 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -147,6 +147,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('thresholds')).toMatchObject({
+      x: { path: 'sd', values: '0.1:0.2:0.01' },
+      ticks: 200,
+      metric: { kind: 'final', series: 'acting' },
+    });
     expect(defaultForm('ants')).toMatchObject({
       x: { path: 'epsilon', values: '0.001,0.002,0.003,0.005,0.01' },
       ticks: 20000,
````

Modify `web/src/ui/series-data.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index 6537c65..1afd74a 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,13 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('thresholds charts', () => {
+  it('chart participation against theory, episodes, the last cascade and the swing over steps', () => {
+    expect(MODEL_CHARTS.thresholds.map((c) => c.title)).toEqual(['Participation', 'Episodes', 'Last cascade', 'Swing']);
+    expect(timeAxisLabel('thresholds')).toBe('Steps');
+  });
+});
+
 describe('ants charts', () => {
   it('chart the split, its variance against theory, flips and extremes over steps', () => {
     expect(MODEL_CHARTS.ants.map((c) => c.title)).toEqual(['Share', 'Variance', 'Flips', 'Extremes']);
````

Modify `web/src/determinism.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index 0763c4c..f1d5a4c 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -13,6 +13,9 @@ import { InlineTransport } from './transport';
 import { decodeShare, encodeShare } from './share';
 import type {
   AgreementConfig,
+  ThresholdsConfig,
+  ThresholdsInspection,
+  ThresholdsStats,
   AntsConfig,
   AntsInspection,
   AntsStats,
@@ -498,6 +501,12 @@ describe('other models through the engine', () => {
     ['mg-arms-race', '0x4c1e9852373241c9'],
     ['cmo-binary', '0x2081105c24039d0c'],
     ['ants-2b', '0xf9258e5dd9d1d673'],
+    ['gr-normal-sampled', '0xa9bca3821a0f4774'],
+    ['gr-city', '0xa587dcb16521cf3c'],
+    ['gr-friends', '0x37ae8bba4d07be7c'],
+    ['gr-ceilings', '0x1cc528db96973a5a'],
+    ['gr-clusters', '0xd493a3251cde5fbe'],
+    ['watts-middle', '0x1ed3157ee5e9ac61'],
     ['ants-becker', '0x4ebaae97020b8902'],
     ['ants-three', '0x4a2871368f4ab782'],
     ['am-ring', '0x0104a02f3c5f5011'],
@@ -715,6 +724,28 @@ describe('the social-structure model through the engine', () => {
   });
 });
 
+describe('the thresholds model through the engine', () => {
+  it('stops at its last step and inspects an actor, a step and Figure 1', async () => {
+    const r = presets.find((p) => p.id === 'gr-uniform')!;
+    const config = { ...structuredClone(r.config as ThresholdsConfig), stop_at: 50 };
+    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'threshold' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as ThresholdsStats;
+    expect([e.finished, ends, e.tick, s.tick, s.acting]).toEqual([true, 1, 50, 50, 0.5]);
+    // The actor grid starts at x 518; actor 1 (threshold 0) is its top-left cell.
+    await e.select(518, 0);
+    const v = e.inspection!.view as ThresholdsInspection;
+    expect([v.panel, v.member!.id, v.member!.threshold, v.member!.acting]).toEqual(['actors', 1, 0, true]);
+    expect(e.inspection!.agentId).toBeNull();
+    await e.select(459, 100);
+    const f = e.inspection!.view as ThresholdsInspection;
+    expect([f.panel, f.cdf]).toEqual(['figure', 0.51]);
+  });
+});
+
 describe('the ants model through the engine', () => {
   it('stops at its last step and inspects an ant, a step and a histogram row', async () => {
     const r = presets.find((p) => p.id === 'am-ring')!;
````

- [ ] **Step 2: Run them to see them fail**

Run: `cd web && npm ci && npm run wasm && npx vitest run`
Expected: failures in the six files above (`isThresholdsView` not exported, no `thresholds` charts or colors, no Compare entry).

- [ ] **Step 3: Carry the model through the page**

Modify `web/src/types.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/types.ts b/web/src/types.ts
index 125ea7c..c88cac5 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -95,7 +95,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -494,7 +494,7 @@ export interface AgreementConfig {
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig;
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig;
 
 /**
  * Arthur's El Farol bar and Challet and Zhang's minority game (milestone 23), with Challet, Marsili
@@ -607,6 +607,75 @@ export interface AntsInspection {
   agent: null;
 }
 
+/**
+ * Granovetter's threshold models (milestone 25): crowds, friends, sampled crowds, clusters and
+ * ceilings, with Watts's cascades on random networks.
+ */
+export interface ThresholdsConfig {
+  model: 'thresholds';
+  actors: number;
+  distribution: 'uniform' | 'perturbed' | 'normal' | 'fixed';
+  mean: number;
+  sd: number;
+  crowd: 'quantiles' | 'sampled';
+  rounding: 'exact' | 'floor' | 'nearest';
+  population: 'fixed' | 'city';
+  network: 'everyone' | 'random' | 'power_law';
+  degree: number;
+  counts_self: boolean;
+  friends: { enabled: boolean; acquaintance: number; weight: number; symmetric: boolean };
+  trigger: 'instigators' | 'random' | 'hub';
+  zero: 'acts' | 'when_reached';
+  update: 'synchronous' | 'asynchronous';
+  ceilings: { share: number; at: number };
+  clusters: { enabled: boolean; count: number; movement: number };
+  repeat: boolean;
+  global: number;
+  max_steps: number;
+  stop_at: number;
+}
+
+export interface ThresholdsStats {
+  tick: number;
+  acting: number;
+  step: number;
+  episodes: number;
+  last_size: number;
+  mean_size: number;
+  global_share: number;
+  /** Granovetter's continuous equilibrium share (a normal crowd seen whole), or null. */
+  theory: number | null;
+  recent_mean: number;
+  swing: number;
+}
+
+export interface ActorView {
+  id: number;
+  threshold: number | null;
+  ceiling: number | null;
+  degree: number | null;
+  sees: number;
+  of: number;
+  acting: boolean;
+  seed: boolean;
+  crowd: number;
+}
+/**
+ * A cell of the thresholds frame: a step of the time panel, a point of Granovetter's Figure 1, a
+ * histogram row, or an actor of the grid (`member`). `agent` is always null.
+ */
+export interface ThresholdsInspection {
+  site: { x: number; y: number };
+  panel: 'time' | 'figure' | 'histogram' | 'actors' | null;
+  step: number | null;
+  crowds: number[] | null;
+  share: number | null;
+  cdf: number | null;
+  count: number | null;
+  member: ActorView | null;
+  agent: null;
+}
+
 /** A preset: `title` is the menu's plain headline; `source` and `name` are its figure or paper and its rules. */
 export interface Preset { id: string; title: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -916,7 +985,7 @@ export interface AgreementStats {
   stable_at: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats;
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -1241,7 +1310,7 @@ export interface AgreementInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection;
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -1296,7 +1365,10 @@ export type ColorMode =
   | 'memory'
   | 'source'
   | 'independent'
-  | 'degree';
+  | 'degree'
+  | 'state'
+  | 'threshold'
+  | 'crowd';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
````

Modify `web/src/models.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/models.ts b/web/src/models.ts
index 2867741..dce7e84 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -1,6 +1,8 @@
 // Which model a config is (milestones 9–21), and what each model offers the page.
 import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
 import type {
+  ThresholdsConfig,
+  ThresholdsInspection,
   AntsConfig,
   AntsInspection,
   FarolConfig,
@@ -39,7 +41,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -61,12 +63,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   image: 'Image Scoring',
   farol: 'El Farol and the Minority Game',
   ants: 'Ants and Recruitment',
+  thresholds: 'Threshold Models',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds'
     ? tag
     : 'sugarscape';
 }
@@ -160,6 +163,11 @@ export function isImageView(v: AnyInspection): v is ImageInspection {
   return 'cell' in v && 'group' in v;
 }
 
+/** A cell of the thresholds frame (a panel, an actor as `member`, and Figure 1's `cdf`); check it first. */
+export function isThresholdsView(v: AnyInspection): v is ThresholdsInspection {
+  return 'panel' in v && 'cdf' in v;
+}
+
 /** A cell of the ants frame (a panel, a grid ant as `member`, and each source's `shares`). */
 export function isAntsView(v: AnyInspection): v is AntsInspection {
   return 'panel' in v && 'shares' in v;
@@ -192,6 +200,7 @@ export function ticksLeft(c: ModelConfig, tick: number): number {
   if (modelOf(c) === 'image' && (c as ImageConfig).end > 0) return Math.max(0, (c as ImageConfig).end - tick);
   if (modelOf(c) === 'farol' && (c as FarolConfig).stop_at > 0) return Math.max(0, (c as FarolConfig).stop_at - tick);
   if (modelOf(c) === 'ants' && (c as AntsConfig).stop_at > 0) return Math.max(0, (c as AntsConfig).stop_at - tick);
+  if (modelOf(c) === 'thresholds' && (c as ThresholdsConfig).stop_at > 0) return Math.max(0, (c as ThresholdsConfig).stop_at - tick);
   return Infinity;
 }
 
@@ -344,6 +353,13 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['independent', 'Independent'],
     ['degree', 'Degree'],
   ],
+  // Who acts (and the seed); each actor's threshold; how many it watches; its crowd.
+  thresholds: [
+    ['state', 'State'],
+    ['threshold', 'Threshold'],
+    ['degree', 'Degree'],
+    ['crowd', 'Crowd'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -366,4 +382,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   image: [],
   farol: [],
   ants: [],
+  thresholds: [],
 };
````

Modify `web/src/engine.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.ts b/web/src/engine.ts
index 5c6691c..9df4501 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -53,7 +53,7 @@ export const FULL_NOTICE = 'This world has reached 1,000,000 ticks, the most its
 export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
   if (modelOf(config) === 'farol') return `This run has reached its last round (${tick}) — Reset to run it again`;
-  if (modelOf(config) === 'ants') return `This run has reached its last step (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'ants' || modelOf(config) === 'thresholds') return `This run has reached its last step (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'tags' || modelOf(config) === 'image') return `This run has reached its last generation (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'classes') return `Equity reached at t = ${tick}: every agent remembers mostly M — Reset to run it again`;
   if (modelOf(config) === 'opinions')
````

Modify `web/src/compare-presets.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index 86ec0c8..7c1b5bc 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -164,6 +164,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'ants-2b',
     b: 'ants-crowd',
   },
+  {
+    id: 'gr-uniform-vs-perturbed',
+    label: 'Uniform vs perturbed crowd — Threshold Models (Compare)',
+    a: 'gr-uniform',
+    b: 'gr-perturbed',
+  },
 ];
 
 /**
````

Modify `web/src/experiments/form.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index 09614c6..3cacc54 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -104,6 +104,10 @@ export function defaultForm(model: ModelKind = 'sugarscape', config?: ModelConfi
     // The built-in ef-predictors' axis: how far attendance swings against predictors per agent.
     return { ...form, x: { path: 'strategies', values: '2:24:2' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'fluctuation' } };
   }
+  if (model === 'thresholds') {
+    // The built-in gr-sd's axis: the share rioting at equilibrium against the spread of thresholds.
+    return { ...form, x: { path: 'sd', values: '0.1:0.2:0.01' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'acting' } };
+  }
   if (model === 'ants') {
     // The built-in ants-flips' axis: flips between sources against self-conversion ε.
     return { ...form, x: { path: 'epsilon', values: '0.001,0.002,0.003,0.005,0.01' }, ticks: 20000, metric: { ...form.metric, kind: 'final', series: 'flips' } };
````

Modify `web/src/ui/series-data.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index 5f34db0..ce9cb89 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -568,6 +568,26 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
     },
     { title: 'Extremes', lines: [{ key: 'extreme', label: 'Steps with a source at 80 % or more', color: '--c1' }], range: [0, 1] },
   ],
+  thresholds: [
+    {
+      title: 'Participation',
+      lines: [
+        { key: 'acting', label: 'Acting now', color: '--c1' },
+        { key: 'theory', label: "Granovetter's continuous equilibrium", color: '--c4' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'Episodes',
+      lines: [
+        { key: 'mean_size', label: 'Mean final share', color: '--c2' },
+        { key: 'global_share', label: 'Share that went global', color: '--red' },
+      ],
+      range: [0, 1],
+    },
+    { title: 'Last cascade', lines: [{ key: 'last_size', label: 'Final share of the last episode', color: '--c3' }], range: [0, 1] },
+    { title: 'Swing', lines: [{ key: 'swing', label: 'Range over the last 100 steps', color: '--c4' }], range: [0, 1] },
+  ],
 };
 
 /**
@@ -575,7 +595,7 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
  * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'farol' ? 'Rounds' : model === 'ants' ? 'Steps' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
+  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
````

Modify `web/src/ui/inspect-panel.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index 975420d..3a6b4e2 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -3,11 +3,12 @@ import { dpdRows } from '../dpd';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
 import { imageRows } from '../image-scoring';
-import { isAgreementView, isAntsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isAgreementView, isAntsView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
   AntsInspection,
+  ThresholdsInspection,
   FarolInspection,
   AgreementInspection,
   AnasaziInspection,
@@ -270,6 +271,25 @@ export class InspectPanel {
     return rows;
   }
 
+  /** A step of the thresholds' time panel, a point of Figure 1, a histogram row, or an actor. */
+  private thresholdsRows(view: ThresholdsInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    const pct = (x: number) => `${fmt(100 * x)} %`;
+    if (view.panel === 'time') return [row('Step', String(view.step)), row('Acting', (view.crowds ?? []).map(pct).join(' · '))];
+    if (view.panel === 'figure') return [row('Share acting', pct(view.share ?? 0)), row('Thresholds at or below', pct(view.cdf ?? 0))];
+    if (view.panel === 'histogram') return [row('Final share', `about ${pct(view.share ?? 0)}`), row('Episodes', String(view.count))];
+    const a = view.member;
+    if (!a) return [row('Point', 'between the panels')];
+    const rows = [
+      row('Actor', `#${a.id}${a.seed ? ' · the spark' : ''}${a.crowd > 1 ? ` · crowd ${a.crowd}` : ''}`),
+      row('Threshold', a.threshold === null ? 'never acts' : pct(a.threshold)),
+      row('Sees acting', `${a.sees} of ${a.of}${a.degree !== null ? ` · watches ${a.degree}` : ''}`),
+      row('Now', a.acting ? 'acting' : 'not acting'),
+    ];
+    if (a.ceiling !== null) rows.push(row('Leaves above', pct(a.ceiling)));
+    return rows;
+  }
+
   /** A step of the ants' time panel, a row of their histogram, or an ant. */
   private antsRows(view: AntsInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -467,6 +487,8 @@ export class InspectPanel {
             ? this.normsRows(view)
           : isAgreementView(view)
             ? this.agreementRows(view)
+          : isThresholdsView(view)
+            ? this.thresholdsRows(view)
           : isAntsView(view)
             ? this.antsRows(view)
           : isFarolView(view)
````

- [ ] **Step 4: Run the page's build and tests**

Run: `cd web && npm run build && npm test`
Expected: the build succeeds; 708 tests pass (47 files).

- [ ] **Step 5: Commit**

```bash
git add web/src
```
```bash
git commit -m "Carry Threshold Models through the page

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

- [ ] **Step 6 (controller): check it in the browser**

`cd web && npm run build && npx vite preview`, then with `?debug`: `gr-uniform` climbs one person a step to 100 with Figure 1's c.d.f. on the 45° line and an orange staircase; `gr-normal-12` stops at 4 with the c.d.f. crossing low (Threshold colors red to blue); `gr-city` repeats crowds with ticks at each start and a histogram piled at 0–1; `gr-clusters` draws ten colored lines and a Crowd-colored grid; `gr-ceilings` saws between about 83 and 92 %; `watts-middle` shows repeated cascades on a 100 × 100 grid; `watts-hub` in Degree mode. Inspect a step, a Figure 1 point, a histogram row and an actor. Rules panel groups as listed; Friends' and Clusters' fields only when enabled; Spread, crowd and rounding only under a normal distribution. Charts: Participation (with Granovetter's line only for normal crowds), Episodes, Last cascade, Swing. Compare "Uniform vs perturbed crowd — Threshold Models (Compare)". Experiments with a thresholds preset: the default axis sd 0.1–0.2 against acting; run the built-in `gr-sd`.

---

### Task 4: The survey's threshold claims

**Files:**
- Create: `survey/src/claims/thresholds.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `sugarscape_core::thresholds::{self, …}` (`granovetter`, `poisson_window`, `power_law_ratio`), `crate::runner::model_after`, `crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict}`.
- Produces: 20 claims (`thresholds.gr.*`, `watts.*`).

- [ ] **Step 1: Write the claims**

Create `survey/src/claims/thresholds.rs` with exactly this content:

````rust
//! Threshold models (milestone 25): Granovetter's crowds (1978) and Watts's
//! global cascades (2002). Episodes are run in parallel, one world per seed,
//! each until it has finished its share of episodes; an episode's size is
//! read from the `last_size` series when `episodes` counts it.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::thresholds::{
    self, Ceilings, Clusters, Crowd, Distribution, Friends, Network, Population, Rounding,
    ThresholdsConfig, Trigger, Update, Zero,
};

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const GRANOVETTER: &str = "Granovetter 1978, AJS 83(6)";
const WATTS: &str = "Watts 2002, PNAS 99(9)";

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

fn config(edit: impl FnOnce(&mut ThresholdsConfig)) -> ThresholdsConfig {
    let mut c = ThresholdsConfig::default();
    edit(&mut c);
    c
}

/// Final shares of `per_seed` episodes on each of `seeds` worlds (repeat on).
fn episodes(c: &ThresholdsConfig, seeds: u64, per_seed: u32) -> Vec<f64> {
    let c = ThresholdsConfig {
        repeat: true,
        ..c.clone()
    };
    let out: Vec<Vec<f64>> = std::thread::scope(|s| {
        let hs: Vec<_> = (1..=seeds)
            .map(|seed| {
                let c = c.clone();
                s.spawn(move || {
                    let mut w = ModelWorld::new(ModelConfig::Thresholds(c), seed).unwrap();
                    let mut sizes = Vec::new();
                    let mut seen = 0.0;
                    while sizes.len() < per_seed as usize {
                        w.model_mut().run(1);
                        let m = w.model();
                        let e = m.latest_value("episodes").unwrap();
                        if e > seen {
                            seen = e;
                            sizes.push(m.latest_value("last_size").unwrap());
                        }
                    }
                    sizes
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    out.concat()
}

/// The first episode's final number acting (one world, run to rest).
fn settle(c: &ThresholdsConfig) -> f64 {
    let mut w = ModelWorld::new(ModelConfig::Thresholds(c.clone()), 1).unwrap();
    for _ in 0..100_000 {
        w.model_mut().run(1);
        if w.model().latest_value("episodes").unwrap() >= 1.0 {
            break;
        }
    }
    w.model().latest_value("acting").unwrap() * f64::from(c.actors)
}

fn share(v: &[f64], f: impl Fn(f64) -> bool) -> f64 {
    v.iter().filter(|&&x| f(x)).count() as f64 / v.len() as f64
}

/// The smallest σ (to 0.0001) at which a quantile crowd of 100 riots past half.
fn tipping(rounding: Rounding) -> f64 {
    let rioting = |sd: f64| {
        settle(&config(|c| {
            c.distribution = Distribution::Normal;
            c.sd = sd;
            c.rounding = rounding;
        })) > 50.0
    };
    let (mut lo, mut hi) = (0.10, 0.15);
    while hi - lo > 1e-4 {
        let mid = (lo + hi) / 2.0;
        if rioting(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

fn watts(n: u32, z: f64) -> ThresholdsConfig {
    config(|c| {
        c.actors = n;
        c.distribution = Distribution::Fixed;
        c.mean = 0.18;
        c.network = Network::Random;
        c.degree = z;
        c.trigger = Trigger::Random;
        c.update = Update::Asynchronous;
    })
}

fn friends(distribution: Distribution, a: f64, w: u32, symmetric: bool) -> ThresholdsConfig {
    config(|c| {
        c.distribution = distribution;
        c.friends = Friends {
            enabled: true,
            acquaintance: a,
            weight: w,
            symmetric,
        };
    })
}

/// The most common number acting among episodes (sizes as shares of 100).
fn mode(v: &[f64]) -> usize {
    let mut counts = [0u32; 101];
    for x in v {
        counts[(x * 100.0).round() as usize] += 1;
    }
    (0..=100)
        .max_by_key(|&k| (counts[k], std::cmp::Reverse(k)))
        .unwrap()
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "thresholds.gr.uniform",
            item: "gr-uniform",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The uniform crowd (thresholds 0 to 99): 'The equilibrium is 100'; replacing the person at 1 by one at 2, 'the riot ends at that point, with one rioter'",
            check: |_| {
                let u = settle(&ThresholdsConfig::default());
                let p = settle(&config(|c| c.distribution = Distribution::Perturbed));
                outcome(u == 100.0 && p == 1.0, format!("{u:.0} and {p:.0}"))
            },
        },
        Claim {
            id: "thresholds.gr.fig2",
            item: "gr-normal-13",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "Figure 2 (100 people, normal thresholds, mean 25): below σc ≈ 12.2 the equilibrium 'increases gradually to about six', then 'jumps to nearly 100, after which it declines' toward 50 (the continuous calculation)",
            check: |_| {
                let r = |sd: f64| thresholds::granovetter(100, 0.25, sd);
                let (lo, hi) = (r(0.122), r(0.123));
                all_of(vec![
                    ("σc between 12.2 and 12.3".into(), outcome(lo < 10.0 && hi > 90.0, format!("{lo:.2} at 12.2, {hi:.2} at 12.3"))),
                    ("about six below".into(), outcome((4.5..=7.0).contains(&lo), format!("{lo:.2}"))),
                    ("nearly 100 above".into(), outcome(r(0.15) > 99.0, format!("{:.2} at σ 15", r(0.15)))),
                    ("declines toward 50".into(), outcome(r(0.5) < r(0.3) && (r(10.0) - 50.0).abs() < 2.0, format!("{:.1}, {:.1}, {:.1}, {:.1} at σ 30, 50, 100, 1 000", r(0.3), r(0.5), r(1.0), r(10.0)))),
                ])
            },
        },
        Claim {
            id: "thresholds.gr.people",
            item: "gr-sd",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The same for a crowd of 100 people (the normal's quantiles as thresholds): it tips near σ 12.2 whether thresholds stay fractions or become whole people (rounded down, or to the nearest)",
            check: |_| {
                let parts = [("fractions", Rounding::Exact), ("rounded down", Rounding::Floor), ("rounded", Rounding::Nearest)]
                    .into_iter()
                    .map(|(name, r)| {
                        let t = tipping(r) * 100.0;
                        (name.to_string(), outcome((t - 12.2).abs() <= 0.1, format!("σc {t:.2}")))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "thresholds.gr.jump",
            item: "gr-normal-sampled",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'a slight perturbation of the normal distribution around the critical standard deviation should have a wholly discontinuous, striking qualitative effect' (crowds of 100 drawn from the normal: the chance of a riot past half jumps by at least one half between σ 12 and 12.5; 1 000 crowds each)",
            check: |_| {
                let p = |sd: f64| {
                    let v = episodes(
                        &config(|c| {
                            c.distribution = Distribution::Normal;
                            c.sd = sd;
                            c.crowd = Crowd::Sampled;
                        }),
                        10,
                        100,
                    );
                    share(&v, |x| x > 0.5)
                };
                let (a, b, lo, hi) = (p(0.12), p(0.125), p(0.10), p(0.16));
                outcome(b - a >= 0.5, format!("{a:.2} at σ 12, {b:.2} at 12.5 ({lo:.2} at 10, {hi:.2} at 16)"))
            },
        },
        Claim {
            id: "thresholds.gr.city",
            item: "gr-city",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "Crowds of 100 drawn from a uniform city: no instigator (.37), one instigator and no one at 1 % (.14), 'in over half the cases (.37 + .14 = .51) the equilibrium result is either no rioters or one rioter' (within .03; 5 000 crowds)",
            check: |_| {
                let v = episodes(&config(|c| c.population = Population::City), 10, 500);
                let (none, one) = (share(&v, |x| x == 0.0), share(&v, |x| x == 0.01));
                let all = share(&v, |x| x == 1.0);
                let mean = v.iter().sum::<f64>() / v.len() as f64;
                outcome(
                    (none - 0.37).abs() <= 0.03 && (one - 0.14).abs() <= 0.03 && (none + one - 0.51).abs() <= 0.03,
                    format!("{none:.3} + {one:.3} = {:.3}", none + one),
                )
                .with(&format!("But everyone riots in only {:.1} % of crowds; the mean is {:.1} rioters. Spilerman's (.90)^10 = {:.3}.", 100.0 * all, 100.0 * mean, 0.9f64.powi(10)))
            },
        },
        Claim {
            id: "thresholds.gr.friends-weight",
            item: "gr-friends-perturbed",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The perturbed crowd with friends: 'the null hypothesis [one rioter] becomes increasingly improbable as the weight attached to friends' behavior increases' (acquaintance ¼; weights 1, 2, 5, 10; 1 000 crowds each)",
            check: |_| {
                let p: Vec<f64> = [1, 2, 5, 10]
                    .into_iter()
                    .map(|w| share(&episodes(&friends(Distribution::Perturbed, 0.25, w, true), 10, 100), |x| x > 0.01))
                    .collect();
                outcome(p.windows(2).all(|w| w[1] >= w[0]) && p[3] > p[0], format!("P(more than one) {:.2}, {:.2}, {:.2}, {:.2}", p[0], p[1], p[2], p[3]))
            },
        },
        Claim {
            id: "thresholds.gr.friends-quarter",
            item: "gr-friends",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'the largest effects occur where people know, on the average, about one-quarter of the rest of the group' (the perturbed crowd, weight 5: the chance of more than one rioter peaks at acquaintance ¼ among 0.05, 0.1, 0.25, 0.5, 0.9)",
            check: |_| {
                let a = [0.05, 0.1, 0.25, 0.5, 0.9];
                let p: Vec<f64> = a
                    .iter()
                    .map(|&a| share(&episodes(&friends(Distribution::Perturbed, a, 5, true), 10, 100), |x| x > 0.01))
                    .collect();
                let top = (0..5).max_by(|&i, &j| p[i].total_cmp(&p[j])).unwrap();
                outcome(a[top] == 0.25, format!("{:?} at {:?}", p.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>(), a))
            },
        },
        Claim {
            id: "thresholds.gr.friends-symmetry",
            item: "gr-friends",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'the symmetry of ties has little effect on outcomes' (the perturbed crowd, acquaintance ¼, weight 5: the chance of more than one rioter the same within 0.1, symmetric or one-way; 10 × 100 crowds)",
            check: |_| {
                let per_seed = |sym: bool| {
                    let v = episodes(&friends(Distribution::Perturbed, 0.25, 5, sym), 10, 100);
                    v.chunks(100).map(|c| share(c, |x| x > 0.01)).collect::<Vec<f64>>()
                };
                equivalent(&per_seed(true), &per_seed(false), Some(0.1), "mutual", "one-way")
            },
        },
        Claim {
            id: "thresholds.gr.friends-small",
            item: "gr-friends-perturbed",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The perturbed crowd with friends: 'the equilibrium changes very little and rarely exceeds five to 10 rioters' (the 95th percentile at most 10; acquaintance ¼, weight 5)",
            check: |_| {
                let mut v = episodes(&friends(Distribution::Perturbed, 0.25, 5, true), 10, 100);
                v.sort_by(f64::total_cmp);
                let p95 = v[(0.95 * v.len() as f64) as usize] * 100.0;
                outcome(p95 <= 10.0, format!("95th percentile {p95:.0} rioters; largest {:.0}", v.last().unwrap() * 100.0))
            },
        },
        Claim {
            id: "thresholds.gr.friends-uniform",
            item: "gr-friends",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "The uniform crowd with friends: 'its equilibrium of 100 rioters is unstable against almost any kind of social structural influence. For most combinations of weights and acquaintance volume tested, the modal equilibrium result is one rioter' (weights 2, 5 × acquaintance 0.05, 0.1, 0.25, 0.5, 0.9)",
            check: |_| {
                let mut ones = 0;
                let mut modes = Vec::new();
                for w in [2, 5] {
                    for a in [0.05, 0.1, 0.25, 0.5, 0.9] {
                        let m = mode(&episodes(&friends(Distribution::Uniform, a, w, true), 10, 50));
                        modes.push(m);
                        if m == 1 {
                            ones += 1;
                        }
                    }
                }
                outcome(ones * 2 > modes.len(), format!("the mode is one rioter in {ones} of {} settings: {modes:?}", modes.len()))
            },
        },
        Claim {
            id: "thresholds.gr.no-oscillation",
            item: "gr-city",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'where no provision has been made for removal of participants, oscillatory behavior of r(t) is not possible, and an equilibrium will always be reached' (5 000 crowds from the city, with friends, and sampled normal crowds all settle before 1 000 steps)",
            check: |_| {
                let v: Vec<f64> = [
                    config(|c| c.population = Population::City),
                    friends(Distribution::Uniform, 0.25, 2, false),
                    config(|c| {
                        c.distribution = Distribution::Normal;
                        c.crowd = Crowd::Sampled;
                    }),
                ]
                .iter()
                .map(|c| {
                    let mut w = ModelWorld::new(ModelConfig::Thresholds(ThresholdsConfig { repeat: true, ..c.clone() }), 1).unwrap();
                    w.model_mut().run(20_000);
                    w.model().series("step").unwrap().into_iter().fold(0.0, f64::max)
                })
                .collect();
                outcome(v.iter().all(|&s| s < 1000.0), format!("longest episodes {:.0}, {:.0}, {:.0} steps", v[0], v[1], v[2]))
            },
        },
        Claim {
            id: "thresholds.gr.ceilings",
            item: "gr-ceilings",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "Figure 3's curve crossing zero twice (a share leave once participation passes 90 %): 'equilibria cannot be guaranteed … aggregate behavior oscillated' (the uniform crowd, synchronous updating: most of 40 crowds never settle, at shares 0.1, 0.3, 0.5)",
            check: |_| {
                let parts = [0.1, 0.3, 0.5]
                    .into_iter()
                    .map(|q| {
                        let c = config(|c| c.ceilings = Ceilings { share: q, at: 0.9 });
                        let pulsing: Vec<(bool, f64)> = model_after(&ModelConfig::Thresholds(c), &(1..=40).collect::<Vec<u64>>(), 600, |w| {
                            let m = w.model();
                            (m.latest_value("episodes").unwrap() == 0.0, m.latest_value("swing").unwrap())
                        });
                        let n = pulsing.iter().filter(|p| p.0).count();
                        let swing = pulsing.iter().filter(|p| p.0).map(|p| p.1).sum::<f64>() / n.max(1) as f64;
                        (format!("share {q}"), outcome(n > 20, format!("{n} of 40 pulse, swinging by {swing:.2}")))
                    })
                    .collect();
                all_of(parts).with("Crowds that settle rest at 91; whether one pulses depends on who holds the ceilings. Asynchronous updating hovers near 90 instead.")
            },
        },
        Claim {
            id: "thresholds.gr.movement",
            item: "gr-movement",
            source: Source::Book,
            citation: GRANOVETTER,
            text: "'what level of movement among clusters would have the most incendiary effect … small movements among clusters may have greater effects than large ones' (10 crowds of 100 from the city: more rioters at movement 0.05 than at 0 and at 1; the mean over the last 100 of 400 steps, 20 runs)",
            check: |_| {
                let at = |m: f64| -> Vec<f64> {
                    let c = config(|c| {
                        c.population = Population::City;
                        c.clusters = Clusters { enabled: true, count: 10, movement: m };
                    });
                    model_after(&ModelConfig::Thresholds(c), &(1..=20).collect::<Vec<u64>>(), 400, |w| w.model().latest_value("recent_mean").unwrap())
                };
                let (none, some, all) = (at(0.0), at(0.05), at(1.0));
                all_of(vec![
                    ("against none".into(), greater(&some, &none, "m 0.05", "m 0")),
                    ("against full mixing".into(), greater(&some, &all, "m 0.05", "m 1")),
                ])
            },
        },
        Claim {
            id: "watts.window",
            item: "watts-window",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 1 (φ* 0.18, n 10 000, single seeds): global cascades (over 10 %) occur inside the analytic window zQ(K* − 1, z) = 1 and not outside it (100 seeds at each z)",
            check: |_| {
                let (lo, hi) = thresholds::poisson_window(0.18, 0.0, 0.5, 10.0).unwrap();
                let f = |z: f64| share(&episodes(&watts(10_000, z), 10, 10), |x| x >= 0.1);
                let inside: Vec<f64> = [1.5, 3.0, 5.0].into_iter().map(f).collect();
                let outside: Vec<f64> = [0.8, 7.0].into_iter().map(f).collect();
                let edge = f(6.14);
                outcome(inside.iter().all(|&x| x > 0.3) && outside.iter().all(|&x| x < 0.05), format!("analytic window z {lo:.2}–{hi:.2}; global in {inside:?} of seeds at z 1.5, 3, 5; {outside:?} at 0.8, 7"))
                    .with(&format!("Past the analytic upper edge, at z 6.14: {:.2}. The simulated window reaches further, as Watts says (\"do not agree perfectly\").", edge))
            },
        },
        Claim {
            id: "watts.size",
            item: "watts-middle",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 2: the average size of global cascades is governed by 'S, the connectivity of the network as a whole' (z 3: within 0.03 of S = 1 − e^−zS)",
            check: |_| {
                let v = episodes(&watts(10_000, 3.0), 10, 10);
                let g: Vec<f64> = v.into_iter().filter(|&x| x >= 0.1).collect();
                let mean = g.iter().sum::<f64>() / g.len() as f64;
                let mut s: f64 = 0.5;
                for _ in 0..200 {
                    s = 1.0 - (-3.0 * s).exp();
                }
                outcome((mean - s).abs() <= 0.03, format!("{mean:.3} against S {s:.3} ({} global of 100)", g.len()))
            },
        },
        Claim {
            id: "watts.fig3-lower",
            item: "watts-lower",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 3 (n 1 000, z 1.05): 'cascades at the lower critical point are power-law distributed, with slope 3/2 (the cumulative distribution has slope 1/2)' (slope of log P(size ≥ s) over s 2–64 within 0.1 of −½; 3 000 seeds)",
            check: |_| {
                let v = episodes(&watts(1000, 1.05), 10, 300);
                let xs = [2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
                let pts: Vec<(f64, f64)> = xs.iter().map(|&x| (x, share(&v, |s| s * 1000.0 >= x - 1e-9))).filter(|p| p.1 > 0.0).map(|(x, p)| (f64::ln(x), p.ln())).collect();
                let n = pts.len() as f64;
                let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
                let b = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
                outcome((b + 0.5).abs() <= 0.1, format!("cumulative slope {b:.2}"))
            },
        },
        Claim {
            id: "watts.fig3-upper",
            item: "watts-upper",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 3 (n 1 000, z 6.14): the distribution is bimodal, 'a single cascade occurring in 1,000 random trials' (at most 1 % of 1 000 seeds global)",
            check: |_| {
                let v = episodes(&watts(1000, 6.14), 10, 100);
                let g = share(&v, |x| x >= 0.5);
                let tiny = share(&v, |x| x < 0.01);
                outcome(g <= 0.01, format!("{:.1} % global (half the network or more), {:.1} % under 1 %", 100.0 * g, 100.0 * tiny))
                    .with(&format!("At n 10 000: {:.1} %. The upper edge moves with the network's size.", 100.0 * share(&episodes(&watts(10_000, 6.14), 10, 10), |x| x >= 0.5)))
            },
        },
        Claim {
            id: "watts.fig4a",
            item: "watts-hetero",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 4a: normally distributed thresholds (σ 0.1) 'cause the system to be less stable, yielding cascades over a greater range of both φ and z' (more global cascades than fixed thresholds at z 8 and at z 1.2; n 2 000, zero thresholds acting when reached)",
            check: |_| {
                let hetero = |z: f64| {
                    let mut c = watts(2000, z);
                    c.distribution = Distribution::Normal;
                    c.sd = 0.1;
                    c.crowd = Crowd::Sampled;
                    c.zero = Zero::WhenReached;
                    c
                };
                let f = |c: &ThresholdsConfig| {
                    let v = episodes(c, 10, 30);
                    v.chunks(30).map(|c| share(c, |x| x >= 0.1)).collect::<Vec<f64>>()
                };
                all_of(vec![
                    ("dense (z 8)".into(), greater(&f(&hetero(8.0)), &f(&watts(2000, 8.0)), "σ 0.1", "fixed")),
                    ("sparse (z 1.2)".into(), greater(&f(&hetero(1.2)), &f(&watts(2000, 1.2)), "σ 0.1", "fixed")),
                ])
            },
        },
        Claim {
            id: "watts.fig4b",
            item: "watts-window",
            source: Source::Book,
            citation: WATTS,
            text: "Fig. 4b: power-law degrees p_k = Ck^−τe^−k/κ, τ 2.5, with 'κ0 … adjusted to generate graphs with variable z', are 'much less vulnerable' (the cascade condition at φ* 0.18 fails for every buildable z, and no simulated global cascades at z 1.5)",
            check: |_| {
                let best = (101..195).map(|z| thresholds::power_law_ratio(f64::from(z) / 100.0, 0.18, 0.0)).fold(0.0, f64::max);
                let mut c = watts(10_000, 1.5);
                c.network = Network::PowerLaw;
                let g = share(&episodes(&c, 10, 20), |x| x >= 0.1);
                outcome(best < 1.0 && g == 0.0, format!("largest G0''(1)/z {best:.2}; {:.0} % global at z 1.5", 100.0 * g))
                    .with("But the family's mean degree cannot exceed ζ(1.5)/ζ(2.5) ≈ 1.95 for any κ: the figure's z beyond that cannot be built as stated.")
            },
        },
        Claim {
            id: "watts.hubs",
            item: "watts-targeting",
            source: Source::Book,
            citation: WATTS,
            text: "'the most connected nodes are far more likely than average nodes to trigger cascades, but not in the second regime' (n 2 000: the hub triggers more global cascades than a random seed at z 1.3, and not at z 5.5; 10 × 30 trials)",
            check: |_| {
                let f = |z: f64, t: Trigger| {
                    let mut c = watts(2000, z);
                    c.trigger = t;
                    episodes(&c, 10, 30).chunks(30).map(|c| share(c, |x| x >= 0.1)).collect::<Vec<f64>>()
                };
                let low = greater(&f(1.3, Trigger::Hub), &f(1.3, Trigger::Random), "hub", "random");
                let high = greater(&f(5.5, Trigger::Hub), &f(5.5, Trigger::Random), "hub", "random");
                let not_high = Outcome {
                    verdict: if high.verdict == Verdict::Holds { Verdict::Fails } else { Verdict::Holds },
                    measured: high.measured,
                    detail: String::new(),
                };
                all_of(vec![("sparse (z 1.3)".into(), low), ("dense (z 5.5): no advantage".into(), not_high)])
            },
        },
    ]
}
````

Modify `survey/src/claims/mod.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index faf9772..74a583c 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -18,6 +18,7 @@ mod opinions;
 mod spatial;
 mod structure;
 mod tags;
+mod thresholds;
 
 use crate::claim::Claim;
 
@@ -43,6 +44,7 @@ pub fn all() -> Vec<Claim> {
         spatial::claims(),
         structure::claims(),
         tags::claims(),
+        thresholds::claims(),
     ]
     .into_iter()
     .flatten()
````

- [ ] **Step 2: Run them**

Run: `cd survey && rustfmt --edition 2021 src/claims/thresholds.rs && cargo build --release && ./target/release/survey --only thresholds && ./target/release/survey --only watts`
Expected (a few minutes on 10 cores): the verdicts below.

| Claim | Verdict | Measured in planning |
|---|---|---|
| thresholds.gr.uniform | Holds | 100 and 1 |
| thresholds.gr.fig2 | Holds | 5.48 at σ 12.2, 100.00 at 12.3; 99.3, 90.5, 65.9, 51.0 at 30–1 000 |
| thresholds.gr.people | Fails | σc 12.55 (fractions), 11.89 (rounded down), 12.23 (rounded) |
| thresholds.gr.jump | Fails | 0.15 at σ 12, 0.25 at 12.5 |
| thresholds.gr.city | Holds | 0.369 + 0.137 = 0.505; everyone 2.3 %, mean 12.4 |
| thresholds.gr.friends-weight | Holds | 0.00, 0.00, 0.43, 0.52 |
| thresholds.gr.friends-quarter | Holds | 0.19, 0.27, 0.43, 0.01, 0.0 |
| thresholds.gr.friends-symmetry | Holds | 0.43 against 0.41, TOST p 0.011 |
| thresholds.gr.friends-small | Holds | 95th percentile 7; largest 24 |
| thresholds.gr.friends-uniform | Holds | mode one in 8 of 10 settings |
| thresholds.gr.no-oscillation | Holds | longest episodes 35, 15, 18 steps |
| thresholds.gr.ceilings | Holds | 34, 29, 22 of 40 pulse at 0.1, 0.3, 0.5 |
| thresholds.gr.movement | Holds | 0.39 against 0.12 and 0.11 |
| watts.window | Holds | window 1.02–5.76; 0.59, 0.91, 0.76 inside; 0, 0 outside |
| watts.size | Holds | 0.941 against 0.940 |
| watts.fig3-lower | Holds | slope −0.48 |
| watts.fig3-upper | Fails | 19.9 % global at n 1 000 (3.0 % at n 10 000) |
| watts.fig4a | Fails | 0.85 against 0 at z 8; 0.10 against 0.23 at z 1.2 |
| watts.fig4b | Holds | largest ratio 0.65; no global cascades at z 1.5 |
| watts.hubs | Fails | 0.95 against 0.37 at z 1.3; 0.88 against 0.52 at z 5.5 |

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/thresholds.rs survey/src/claims/mod.rs
```
```bash
git commit -m "Survey Threshold Models: Granovetter's crowds and extensions, and Watts's cascades

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 5: README, roadmap, papers index, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-27-thresholds-design.md`

- [ ] **Step 1: Write the docs**

Modify `README.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/README.md b/README.md
index fa9e85d..d0b7a49 100644
--- a/README.md
+++ b/README.md
@@ -154,7 +154,7 @@ The presets menu groups its presets by model: **Sugarscape**, **Schelling**, **R
 **Artificial Anasazi**, **Civil Violence**, **Tag Cooperation**, **Spatial Games**, **Axelrod Culture**,
 **Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence**, **Social Structure**,
 **Demographic PD**, **Norms and Metanorms**, **Relative Agreement**,
-**Image Scoring**, **El Farol and the Minority Game** and **Ants and Recruitment**.
+**Image Scoring**, **El Farol and the Minority Game**, **Ants and Recruitment** and **Threshold Models**.
 Each preset is listed by a plain title saying what happens in it; under the menu, the chosen
 preset's source (the book's figure or animation, or the paper) and its rules sit above its description.
 Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
@@ -1480,6 +1480,71 @@ Model," *Evol. Ecol. Res.* 4 (2002); D. Mark, *Behavioral Mathematics for Game A
 "Choosing Effective Utility-Based Considerations," *Game AI Pro 3* (2017). See
 `docs/superpowers/specs/2026-09-27-minds-1-utility-design.md`.
 
+### Threshold Models (Granovetter 1978; Watts 2002)
+
+**The crowd.** Each person has a threshold: the share of the crowd he must see join before he joins
+(Granovetter). An instigator (threshold 0) acts; whoever's threshold that reaches acts next; and so
+on until nobody new is tipped. Everything turns on the exact distribution of thresholds, not its
+average. **Watts** put the same rule on a sparse random network — each person watches only his
+neighbors — and asked when a single spark becomes a cascade that sweeps the network.
+
+Measured (the survey and the presets' descriptions):
+
+- **Granovetter's crowds reproduce.** Thresholds 0 to 99 give a riot of 100; move the person at 1
+  up to 2 and only the instigator riots. His Figure 2's continuous calculation jumps between σ 12.2
+  and 12.3 — "about six" rioters below (5.5), "nearly 100" above, 50 in the limit.
+- **A crowd of real people has no single tipping point.** A crowd of 100 whose thresholds are the
+  normal's quantiles tips at σ 12.23 when thresholds are rounded to whole people (his 12.2), 11.89
+  when rounded down, 12.55 when kept as fractions. And crowds drawn at random from the normal
+  distribution show no jump at all: they riot past half 15 % of the time at σ 12, 25 % at 12.5.
+- **The "equilibrium of 100" is rare.** Of crowds drawn from his uniform city, 36.9 % + 13.7 % =
+  50.5 % end with no rioters or one ("over half … .51"), as he says — but everyone riots in only
+  2.3 %, and the mean is 12 rioters.
+- **The friends claims hold under our reading** (friends at random, counted w times, the actor
+  dividing by the whole crowd with himself included, as in his 63/120 example): the uniform crowd's
+  most common outcome becomes one rioter; the perturbed crowd spreads more often as friends weigh
+  more (0, 0, 43 %, 52 % at weights 1, 2, 5, 10), most at an acquaintance of a quarter, rarely past
+  seven rioters; one-way friendships change little.
+- **A middling movement between crowds is the most incendiary** (12 % rioting with no movement, 41 %
+  at 0.05, 29 % with everyone moving every step), as he suggests.
+- **Ceilings make riots pulse.** With some people leaving once more than 90 % riot (his Figure 3),
+  most crowds never settle — the riot climbs, the cautious leave, it climbs again — but whether a
+  given crowd pulses depends on who holds the ceilings; decided one at a time, it hovers near 90 %.
+- **Watts's window reproduces**: cascades between z ≈ 1 and 6 at threshold 18 % (the analytic window
+  1.02–5.76), global cascades filling the connected network (0.941 against S = 0.940), and a
+  power law of slope ½ at the lower edge (−0.48).
+- **His upper edge depends on network size.** At his n 1 000 and z 6.14, 20 % of sparks go global,
+  not "a single cascade in 1,000 trials" (3 % at n 10 000).
+- **Varied thresholds widen only the dense side of the window**; at the sparse side they narrow it
+  (9 % against 28 % at z 1.2). **His Figure 4b cannot be built as stated**: with τ 2.5 and k ≥ 1 a
+  power law's mean degree cannot exceed 1.95, and at threshold 18 % no such network cascades.
+- **Hubs help in both regimes**: the best-connected spark goes global far more often at z 1.3 (95 %
+  against 39 %) and still twice as often at z 5.5 (89 % against 44 %), where he says it does not.
+
+Switches: **Actors**, **Each crowd** (as drawn, or sampled from the city), **Started by**
+(instigators, one random actor, the hub), **Actors decide** (together, or one at a time), **Count
+oneself in the group**, **Thresholds** (uniform, perturbed, normal, everyone the same), **Mean**,
+**Spread**, **A normal crowd is** (quantiles, or drawn), **Thresholds are** (fractions, or whole
+people rounded down or to the nearest), **A threshold of 0** (acts at once, or once a neighbor does),
+**Friends count more** with **Acquaintance**, **A friend counts as** and **Friendship is mutual**,
+**Who sees whom** (the whole crowd, a random network, a network with hubs) with **Mean degree**,
+**Ceilings** (the share who leave, and above what), **Several crowds** with **Crowds** and **Movement
+per step**, and **Episodes** (start again at each equilibrium; what counts as global; the longest
+episode). The view: the share acting over the last 400 steps (one line per crowd), Granovetter's
+Figure 1 (the thresholds' c.d.f. against the 45° line, with the riot's staircase) for a single crowd
+seen whole or the histogram of episode sizes otherwise, and the actors as a grid. Color modes:
+**State**, **Threshold**, **Degree**, **Crowd**. Charts: Participation (with Granovetter's
+continuous equilibrium); Episodes; Last cascade; Swing. Presets: `gr-uniform`, `gr-perturbed`,
+`gr-normal-12`, `gr-normal-13`, `gr-normal-sampled`, `gr-city`, `gr-friends`,
+`gr-friends-perturbed`, `gr-ceilings`, `gr-clusters`, `watts-lower`, `watts-middle`, `watts-upper`,
+`watts-hetero`, `watts-hub`. **Compare** entry: "Uniform vs perturbed crowd — Threshold Models
+(Compare)". Built-in sweeps: `gr-sd`, `gr-friends`, `gr-movement`, `gr-ceilings`, `watts-window`,
+`watts-hetero`, `watts-targeting`.
+
+Credit: Mark Granovetter, "Threshold Models of Collective Behavior," *American Journal of Sociology*
+83(6) (1978), 1420–1443; Duncan J. Watts, "A Simple Model of Global Cascades on Random Networks,"
+*PNAS* 99(9) (2002), 5766–5771. See `docs/superpowers/specs/2026-09-27-thresholds-design.md`.
+
 ## Experiments
 
 The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
````

Modify `docs/roadmap.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/roadmap.md b/docs/roadmap.md
index b2edc45..0d8aa9b 100644
--- a/docs/roadmap.md
+++ b/docs/roadmap.md
@@ -228,6 +228,16 @@ IIb's "average about one-half" needs a hundred times the figure's run; the herdi
 colony grows; a random network cures that under Alfarano and Milaković's rule but not Kirman's, and
 their mean field fails on rings. See `docs/superpowers/specs/2026-09-27-ants-design.md`.
 
+## Milestone 25: Threshold Models (done)
+
+Granovetter's threshold model (1978) as a model kind, with the four extensions he sketches — friends,
+crowds sampled from a city, clusters with movement, ceilings — and Watts's cascades on random
+networks (2002). His crowds and Figure 2's continuous jump reproduce, but a crowd of real people has
+no single tipping point, and the city's "equilibrium of 100" happens in 2 % of crowds; the friends
+claims hold under a stated reading; middling movement is most incendiary; ceilings make riots pulse.
+Watts's window and power law reproduce; his upper edge depends on n, his Fig. 4b cannot be built as
+stated, and hubs help in both regimes. See `docs/superpowers/specs/2026-09-27-thresholds-design.md`.
+
 ## Experiments and science
 
 - **Parameter sweeps / batch runs**: done (Milestone 5).
@@ -248,6 +258,7 @@ their mean field fails on rings. See `docs/superpowers/specs/2026-09-27-ants-des
 - **Deffuant et al.'s relative agreement and extremism** (and Meadows & Cliff's replication): done (Milestone 22).
 - **Arthur's El Farol and Challet & Zhang's minority game** (and the memory transition, and Challet, Marsili & Ottino's critique): done (Milestone 23).
 - **Kirman's ants and recruitment** (and Alfarano & Milaković's network critique): done (Milestone 24).
+- **Granovetter's threshold models** (and Watts's global cascades): done (Milestone 25).
 - **Minds 1: the utility mind and the ideal free distribution** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Credit hierarchy view**: done (Milestone 6).
````

Modify `docs/papers.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/papers.md b/docs/papers.md
index dac7518..3f9be75 100644
--- a/docs/papers.md
+++ b/docs/papers.md
@@ -32,6 +32,7 @@ read online or from another copy; add it when found. Scanned PDFs (no text layer
 | 22 | `agreement` | `bounded-confidence/deffuant-neau-amblard-weisbuch-2000-acs-mixing-beliefs.pdf`, `bounded-confidence/deffuant-amblard-weisbuch-faure-2002-jasss-how-can-extremism-prevail.html`, `bounded-confidence/amblard-deffuant-2004-network-topology-and-extremism.pdf`, `bounded-confidence/weisbuch-2003-bounded-confidence-and-social-networks.pdf`; the replication and reply `bounded-confidence/meadows-cliff-2012-jasss-reexamining-the-relative-agreement-model.html`, `bounded-confidence/deffuant-amblard-weisbuch-2013-jasss-meadows-and-cliff-are-wrong.html` | both of the reply's fixes are needed, and the paper as stated agrees with it; balanced extremists' single extreme vanishes as N grows; Figs. 5 and 7 and eq. 11 as printed do not reproduce; the unstated cutoff decides the network results |
 | 23 | `farol` | `el-farol/arthur-1994-aer-inductive-reasoning-and-bounded-rationality.pdf`, `el-farol/challet-zhang-1997-emergence-of-cooperation-minority-game.pdf`; follow-ups `el-farol/savit-manuca-riolo-1999-prl-adaptive-competition-market-efficiency-phase-transitions.pdf`, `el-farol/challet-zhang-1998-physica-a-on-the-minority-game-analytical-and-numerical.pdf`, `el-farol/challet-marsili-ottino-2004-physica-a-shedding-light-on-el-farol.pdf` | Arthur's mean of 60 is trivial and his agents swing far more than coin-flippers; his cycles persist under accuracy scoring; the memory transition reproduces; CZ97's Fig. 4 two peaks and Fig. 10 waste do not |
 | 24 | `ants` | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf`; the critique `ants/alfarano-milakovic-2007-warwick-wp-should-network-structure-matter.pdf` (published as JEDC 33(1), 2009) | the chain is exactly beta-binomial but never rests at the ants' 80–20 (Becker's pull does); Fig. IIb's time average needs 100× the figure; herding fades with N, cured by a random network only under AM's rule; AM's mean field fails on rings |
+| 25 | `thresholds` | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*, read by OCR); the follow-up `thresholds/watts-2002-pnas-simple-model-of-global-cascades-on-random-networks.pdf` (from the Internet Archive's copy of PNAS) | the crowds and Fig. 2's continuous jump reproduce, but a crowd of people tips at a σ set by rounding and sampled crowds do not jump; the city's riot of 100 comes 2 % of the time; Watts's upper edge depends on n, his Fig. 4b cannot be built, hubs help in both regimes |
 
 ## Queue
 
@@ -40,13 +41,12 @@ worth doing; "size" is a guess at the milestone's scale.
 
 | # | Model | Original | Critique or follow-up | Size | Shape |
 |---|---|---|---|---|---|
-| 1 | Threshold models | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*) | — | small | new kind, or a Sugarscape rule |
-| 2 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
-| 3 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
-| 4 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
-| 5 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
-| 6 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
-| 7 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
+| 1 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
+| 2 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
+| 3 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
+| 4 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
+| 5 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
+| 6 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
 
 ## Wanted
````

Modify `docs/superpowers/specs/2026-09-27-thresholds-design.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

````diff
diff --git a/docs/superpowers/specs/2026-09-27-thresholds-design.md b/docs/superpowers/specs/2026-09-27-thresholds-design.md
index 75a98b0..3155951 100644
--- a/docs/superpowers/specs/2026-09-27-thresholds-design.md
+++ b/docs/superpowers/specs/2026-09-27-thresholds-design.md
@@ -157,3 +157,18 @@ The presets menu gains a **Threshold Models** group and the Compare entry; the R
 ## Docs
 
 README: a Threshold Models section (the model, the stated choices, switches, presets, sweeps, and the findings: the uniform and perturbed crowds and Figure 2 reproduce, but a real crowd of 100 has no sharp tipping point and its critical σ depends on rounding; "equilibrium 100" happens in 2.6 % of sampled crowds; the friends claims hold under our reading; intermediate movement is most incendiary; ceilings make riots pulse; Watts's window and slope ½ reproduce, his upper edge and "one in 1,000" depend on n, Fig. 4b cannot be built above z 1.95, and hubs still help in the dense regime). `docs/papers.md`: the milestone's row with Watts; roadmap: Milestone 25 done.
+
+## Amendments (implementation planning)
+
+The model was implemented in full while planning (`docs/superpowers/plans/2026-09-27-thresholds.md`) and measured with it; these change or extend the sections above.
+
+- **Watts's presets and sweeps are `watts-*`**, not `w-*`: `w-scale-free` is already Weisbuch's (milestone 22).
+- **Thresholds are exact fractions** (`num/den`): whole-people thresholds as k/N, real ones to six decimals; every comparison is a·den ≥ num·g in integers.
+- **Normal thresholds are portable**: quantiles by Acklam's approximation and draws by the anasazi's polar method, both on `crate::portable::ln`, so native and WASM agree bit for bit (the WASM golden test covers sampled normals).
+- **Networks** use a new sparse G(n, p) (geometric skipping, portable) and a configuration model for power-law degrees (degrees drawn from the table, self-links and repeats dropped); the power law's κ is solved by bisection with the portable exponential.
+- **Friends need at most 2 000 actors** (one-way ties draw every ordered pair).
+- **`gr-ceilings` gives 10 % ceilings**, not 30 %: whether a crowd pulses depends on who holds the ceilings (at 30 % seed 1 settles; at 10 %, 34 of 40 crowds pulse), and the swing is about the share holding them (83–92 at 10 %).
+- **Statistics:** `recent_mean` and `swing` (the mean and range of the share acting over the last 100 steps) replace `clusters_mean`; `theory` is Granovetter's continuous equilibrium for a normal crowd seen whole, null otherwise.
+- **The view** marks episode starts with a short tick at the top of the time panel; the Figure 1 panel shows the c.d.f. (blue), the 45° line and the episode's staircase (orange).
+- **Charts:** Participation (`acting`, `theory`), Episodes (`mean_size`, `global_share`), Last cascade (`last_size`), Swing (`swing`).
+- **Measured with the implementation** (the survey, 20 claims; 14 hold, 6 fail): the uniform and perturbed crowds 100 and 1; Fig. 2's continuous 5.48 at σ 12.2 and 100.0 at 12.3; the crowd of 100 tipping at 12.55, 11.89, 12.23 (fractions, rounded down, rounded); sampled crowds past half 0.15 at σ 12 and 0.25 at 12.5; the city's 0.369 + 0.137 = 0.505, everyone in 2.3 %, mean 12.4; friends' weight 0, 0, 0.43, 0.52 at w 1, 2, 5, 10; the quarter peak 0.19, 0.27, 0.43, 0.01, 0 at a 0.05–0.9; symmetry equivalent (0.43 against 0.41); the perturbed crowd's 95th percentile 7; the uniform crowd's mode one in 8 of 10 settings; no episode past 35 steps without removal; ceilings pulsing in 34, 29, 22 of 40 at shares 0.1, 0.3, 0.5; movement 0.39 at m 0.05 against 0.12 and 0.11; Watts's window 1.02–5.76 analytic, 0.59, 0.91, 0.76 global at z 1.5, 3, 5 and 0 at 0.8 and 7; global size 0.941 against S 0.940; the lower edge's slope −0.48; the upper edge 19.9 % global at n 1 000 (3.0 % at n 10 000); Fig. 4a 0.85 against 0 at z 8 but 0.10 against 0.23 at z 1.2; Fig. 4b's largest ratio 0.65 and no global cascades at z 1.5; hubs 0.95 against 0.37 at z 1.3 and 0.88 against 0.52 at z 5.5.
````

- [ ] **Step 2: Verify everything**

Run: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace && wasm-pack test --node crates/sugarscape-wasm && (cd web && npm run build && npm test) && (cd survey && cargo build --release && ./target/release/survey --only thresholds && ./target/release/survey --only watts)`
Expected: all green; the survey's verdicts as in Task 4.

- [ ] **Step 3 (controller): the full browser pass** — Task 3's Step 6 list again, on the final build.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-09-27-thresholds-design.md
```
```bash
git commit -m "Document Threshold Models and what does not reproduce; mark milestone 25 done

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

## Self-review (planning)

- **Spec coverage:** Architecture, the world's episodes, Config, Step, Theory, Statistics, Views, Presets, Compare, Experiments and CLI → Tasks 1–3; Survey → Task 4; Docs → Task 5; departures are in the spec's Amendments (Task 5).
- **Placeholders:** none; every file is given in full or as a diff against `1eeb658`.
- **Types:** `ThresholdsInspection`'s fields (`panel`, `step`, `crowds`, `share`, `cdf`, `count`, `member`, `agent`) and `ActorView`'s match between `world.rs` and `types.ts`; `isThresholdsView` tests `cdf`, which no other inspection has.
