# SugarScape WASM Playground Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A browser playground implementing Epstein & Axtell's Sugarscape (Chapters II–III: G, seasons, M, pollution, R, S, I, K, C) with a pure-Rust core compiled to WASM and a Vite + TypeScript front end with presets, live charts, inspection, editing, share URLs and export.

**Architecture:** `crates/sugarscape-core` is a dependency-light, fully unit-tested simulation (no wasm deps). `crates/sugarscape-wasm` wraps it in a single `Sim` class via `wasm-bindgen`; Rust renders the grid to an RGBA buffer that JS views zero-copy. `web/` is a Vite + TypeScript app that owns only UI: canvas blit + overlays, controls generated from a schema, uPlot charts, URL sharing, downloads.

**Tech Stack:** Rust 2021 (cargo 1.96), `rand 0.8` (no default features) + `rand_pcg 0.3`, `serde`/`serde_json`, `proptest`; `wasm-bindgen`, `console_error_panic_hook`, `wasm-pack`; Vite, TypeScript (strict), Vitest, uPlot; GitHub Actions + Pages.

**Spec:** `docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md`

**Source text:** Rule semantics come from the book's Appendix B and Chapters II–III, as quoted in this plan's module doc comments and the spec.

## Global Constraints

- The core crate must not depend on `wasm-bindgen`, `js-sys`, `web-sys`, or `getrandom`. `rand` is used with `default-features = false, features = ["alloc"]` (the `std` feature pulls `getrandom`, which fails on `wasm32-unknown-unknown`).
- Determinism: all randomness flows through the single `World.rng` (`Pcg64Mcg` seeded from the user seed). Never iterate a `HashMap`/`HashSet` in simulation code; use `BTreeMap` / `Vec`.
- Lattice: `y = 0` is the northern (top) row; the lattice is a torus. Vision and neighbors use only the four principal directions (N, S, E, W).
- Agents act asynchronously in a freshly shuffled order each tick; an agent completes move → metabolize → death check → sex → culture before the next agent acts. Newborns first act the tick after birth.
- Sugar (site and agent) is `f64`.
- JS seeds are `u32` (avoid BigInt at the WASM boundary).
- Errors crossing into JS are JSON strings of `[{ "field": string, "message": string }]`.
- Every commit message ends with the line `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3` (preceded by a blank line).
- Run `cargo fmt` before each Rust commit; `cargo clippy --all-targets -- -D warnings` must pass at the end of each Rust task.

## File Structure

```
Cargo.toml                                  workspace
crates/sugarscape-core/
  Cargo.toml
  assets/sugar-map.txt                      50×50 two-peak capacity map (row 0 = north)
  src/lib.rs                                module list
  src/geometry.rs                           Pos, Torus, DIRECTIONS, sight()
  src/rng.rs                                SimRng type + seeded()
  src/config.rs                             Config + rule structs, FieldError, validate(), structural_changes()
  src/landscape.rs                          Site, capacities() for each LandscapeKind
  src/agent.rs                              Agent, AgentId, Sex, Tribe, Tags
  src/world.rs                              World: storage, placement, step(), kill(), fingerprint()
  src/rules/mod.rs                          agent_turn(): the per-agent rule sequence
  src/rules/growback.rs                     G_α + seasonal growback
  src/rules/movement.rs                     M (with pollution welfare)
  src/rules/lifecycle.rs                    metabolism, pollution formation, death check
  src/rules/pollution.rs                    diffusion D_α
  src/rules/replacement.rs                  R_[a,b]
  src/rules/sex.rs                          S
  src/rules/culture.rs                      K
  src/rules/combat.rs                       C_α
  src/stats.rs                              Snapshot, Stats, gini/lorenz/histogram
  src/render.rs                             ColorMode, Layer, render() → RGBA
  src/edit.rs                               paint/place/remove/inspect/set_config
  src/export.rs                             CSV exports
  src/presets.rs                            book rule systems
  src/testkit.rs                            (cfg(test)) tiny-world builders
  tests/invariants.rs                       proptest invariants + determinism
  tests/book.rs                             #[ignore] reproductions of book results
crates/sugarscape-wasm/
  Cargo.toml
  src/lib.rs                                `Sim` + free functions
  tests/web.rs                              wasm-bindgen-test smoke tests (node)
web/
  package.json, tsconfig.json, vite.config.ts, index.html
  src/main.ts                               bootstrap + wiring
  src/types.ts                              TS mirrors of Rust JSON shapes
  src/engine.ts                             Engine: owns Sim, run loop state, events
  src/paths.ts (+ paths.test.ts)            getPath/setPath for config objects
  src/schema.ts                             control definitions for the Rules panel
  src/share.ts (+ share.test.ts)            URL encode/decode
  src/downloads.ts                          file download helpers
  src/style.css                             tokens (light/dark) + layout
  src/ui/dom.ts                             h() element helper
  src/ui/grid-view.ts                       canvas blit, overlays, pointer → cell
  src/ui/toolbar.ts                         run controls, seed, share, export
  src/ui/display.ts                         color mode + layer selects
  src/ui/tools.ts                           tool picker + tool options
  src/ui/rules-panel.ts                     presets, toggles, parameters, errors
  src/ui/charts-panel.ts                    uPlot charts
  src/ui/inspect-panel.ts                   agent/site details + lineage links
.github/workflows/ci.yml, .github/workflows/pages.yml
```

---

### Task 1: Workspace, geometry and RNG

**Files:**
- Create: `Cargo.toml`, `crates/sugarscape-core/Cargo.toml`, `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/geometry.rs`, `crates/sugarscape-core/src/rng.rs`
- Modify: `.gitignore`

**Interfaces:**
- Produces: `geometry::{Pos, Torus, DIRECTIONS}`, `Pos::new(x, y)`, `Torus::{new, len, index, pos, offset, neighbors, sight}`; `rng::{SimRng, seeded(u64) -> SimRng}`.

- [ ] **Step 1: Create the workspace**

`Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/sugarscape-core"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
rand = { version = "0.8", default-features = false, features = ["alloc"] }
rand_pcg = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
proptest = "1"

[profile.release]
lto = true
opt-level = 3
```

`crates/sugarscape-core/Cargo.toml`:
```toml
[package]
name = "sugarscape-core"
description = "Sugarscape (Epstein & Axtell, 1996) simulation core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
rand.workspace = true
rand_pcg.workspace = true
serde.workspace = true
serde_json.workspace = true

[dev-dependencies]
proptest.workspace = true
```

`crates/sugarscape-core/src/lib.rs`:
```rust
//! Sugarscape, after Epstein & Axtell, *Growing Artificial Societies* (1996).
//!
//! Rule semantics follow Appendix B ("Summary of Rule Notation") and the
//! rule statements in Chapters II–III.

pub mod geometry;
pub mod rng;
```

Append to `.gitignore`:
```
# Web
node_modules/
web/dist/
web/src/wasm-pkg/
```

- [ ] **Step 2: Write failing geometry tests**

`crates/sugarscape-core/src/geometry.rs`:
```rust
//! Lattice positions and wraparound (toroidal) geometry.

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_and_pos_round_trip() {
        let t = Torus::new(7, 5);
        for i in 0..t.len() {
            assert_eq!(t.index(t.pos(i)), i);
        }
        assert_eq!(t.pos(8), Pos::new(1, 1));
    }

    #[test]
    fn offset_wraps_in_both_axes() {
        let t = Torus::new(10, 8);
        assert_eq!(t.offset(Pos::new(0, 0), -1, -1), Pos::new(9, 7));
        assert_eq!(t.offset(Pos::new(9, 7), 1, 1), Pos::new(0, 0));
        assert_eq!(t.offset(Pos::new(3, 3), 25, -17), Pos::new(8, 2));
    }

    #[test]
    fn neighbors_are_north_south_east_west() {
        let t = Torus::new(10, 10);
        assert_eq!(
            t.neighbors(Pos::new(5, 5)),
            [Pos::new(5, 4), Pos::new(5, 6), Pos::new(6, 5), Pos::new(4, 5)]
        );
    }

    #[test]
    fn sight_covers_four_directions_without_diagonals() {
        let t = Torus::new(11, 11);
        let seen = t.sight(Pos::new(5, 5), 2);
        assert_eq!(seen.len(), 8);
        assert!(seen.contains(&(Pos::new(5, 3), 2)));
        assert!(seen.contains(&(Pos::new(7, 5), 2)));
        assert!(seen.contains(&(Pos::new(4, 5), 1)));
        assert!(!seen.iter().any(|&(p, _)| p == Pos::new(6, 6)));
        assert!(!seen.iter().any(|&(p, _)| p == Pos::new(5, 5)));
    }

    #[test]
    fn sight_is_sorted_by_distance_and_deduplicated_on_small_tori() {
        let t = Torus::new(4, 4);
        let seen = t.sight(Pos::new(0, 0), 3);
        let mut positions: Vec<Pos> = seen.iter().map(|&(p, _)| p).collect();
        positions.sort();
        positions.dedup();
        assert_eq!(positions.len(), seen.len(), "no duplicates");
        assert!(seen.windows(2).all(|w| w[0].1 <= w[1].1), "nearest first");
        // (0,3) is one step north (wrapping), not three steps south.
        assert!(seen.contains(&(Pos::new(0, 3), 1)));
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p sugarscape-core geometry`
Expected: FAIL to compile — `Torus`, `Pos` not found.

- [ ] **Step 4: Implement geometry**

Insert above the `#[cfg(test)]` block in `geometry.rs`:
```rust
/// A lattice position. `y = 0` is the northern (top) row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

impl Pos {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// The four principal lattice directions, in order: north, south, east, west.
pub const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (0, 1), (1, 0), (-1, 0)];

/// Wraparound lattice geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Torus {
    pub width: u32,
    pub height: u32,
}

impl Torus {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "torus must be non-empty");
        Self { width, height }
    }

    pub fn len(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn index(&self, p: Pos) -> usize {
        (p.y * self.width + p.x) as usize
    }

    pub fn pos(&self, i: usize) -> Pos {
        Pos::new(i as u32 % self.width, i as u32 / self.width)
    }

    pub fn offset(&self, p: Pos, dx: i32, dy: i32) -> Pos {
        let x = (i64::from(p.x) + i64::from(dx)).rem_euclid(i64::from(self.width));
        let y = (i64::from(p.y) + i64::from(dy)).rem_euclid(i64::from(self.height));
        Pos::new(x as u32, y as u32)
    }

    /// Von Neumann neighbors in `DIRECTIONS` order.
    pub fn neighbors(&self, p: Pos) -> [Pos; 4] {
        DIRECTIONS.map(|(dx, dy)| self.offset(p, dx, dy))
    }

    /// Every site visible from `p` with `vision`, with its distance, nearest
    /// first. Excludes `p`; on small tori where lines of sight wrap onto the
    /// same site, each site appears once at its shortest distance.
    pub fn sight(&self, p: Pos, vision: u32) -> Vec<(Pos, u32)> {
        let mut seen = Vec::with_capacity(4 * vision as usize);
        for (dx, dy) in DIRECTIONS {
            for d in 1..=vision as i32 {
                seen.push((self.offset(p, dx * d, dy * d), d as u32));
            }
        }
        seen.sort_by_key(|&(_, d)| d);
        let mut out: Vec<(Pos, u32)> = Vec::with_capacity(seen.len());
        for (q, d) in seen {
            if q != p && !out.iter().any(|&(r, _)| r == q) {
                out.push((q, d));
            }
        }
        out
    }
}
```

`crates/sugarscape-core/src/rng.rs`:
```rust
//! The simulation's single random stream.

use rand::SeedableRng;

pub type SimRng = rand_pcg::Pcg64Mcg;

pub fn seeded(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core geometry`
Expected: 5 passed.

- [ ] **Step 6: Verify the core builds for wasm**

Run: `cargo build -p sugarscape-core --target wasm32-unknown-unknown`
Expected: builds (proves no `getrandom` pulled in).

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add Cargo.toml crates .gitignore
git commit -m "Add workspace, lattice geometry and seeded RNG"
```
(End the message with the Claude-Session line from Global Constraints; this applies to every commit below.)

---

### Task 2: Configuration and validation

**Files:**
- Create: `crates/sugarscape-core/src/config.rs`, `crates/sugarscape-core/src/agent.rs` (only the `Sex`/`Tribe` enums for now)
- Modify: `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `config::URange { min: u32, max: u32 }`, `URange::new`, `URange::sample(&self, &mut impl Rng) -> u32`
  - `config::LandscapeKind::{TwoPeaks, Flat { capacity: f64 }}` (serde tag `kind`, snake_case)
  - `config::Placement::{Random, Block { x, y, width, height }, Tribes { size }}` (serde tag `kind`)
  - `config::{Growback { rate, instant }, Seasons { enabled, winter_divisor, period }, Pollution { enabled, production, consumption }, Diffusion { enabled, every }, Lifespan { enabled, max_age }, Toggle { enabled }, SexRule { enabled, fertility_onset, female_end, male_end }, CombatRule { enabled, unlimited, reward }}`
  - `SexRule::end_for(&self, agent::Sex) -> URange`
  - `config::Config` (fields below; `Default` = the book's ({G₁},{M}) setup), `Config::validate() -> Result<(), Vec<FieldError>>`, `Config::from_json(&str) -> Result<Config, Vec<FieldError>>`, `Config::structural_changes(&self, next: &Config) -> Vec<FieldError>`
  - `config::FieldError { field: String, message: String }` (Serialize), `FieldError::new`
  - `agent::{Sex::{Female, Male}, Tribe::{Blue, Red}}` (serde lowercase)

- [ ] **Step 1: Add the Sex/Tribe enums**

`crates/sugarscape-core/src/agent.rs`:
```rust
//! Agents and their genetic and cultural attributes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Female,
    Male,
}

/// Group membership (Chapter III): Blue when zeros outnumber ones on the tag string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tribe {
    Blue,
    Red,
}
```

Update `lib.rs` module list:
```rust
pub mod agent;
pub mod config;
pub mod geometry;
pub mod rng;
```

- [ ] **Step 2: Write failing config tests**

`crates/sugarscape-core/src/config.rs` (tests at the bottom; the implementation goes above in Step 4):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn fields(r: Result<(), Vec<FieldError>>) -> Vec<String> {
        r.err().unwrap_or_default().into_iter().map(|e| e.field).collect()
    }

    #[test]
    fn default_is_the_books_chapter_two_setup_and_valid() {
        let c = Config::default();
        assert_eq!((c.width, c.height, c.population), (50, 50, 400));
        assert_eq!(c.vision, URange::new(1, 6));
        assert_eq!(c.metabolism, URange::new(1, 4));
        assert_eq!(c.endowment, URange::new(5, 25));
        assert_eq!(c.tag_length, 11);
        assert_eq!(c.growback.rate, 1.0);
        c.validate().unwrap();
    }

    #[test]
    fn rejects_inverted_ranges_with_field_names() {
        let mut c = Config::default();
        c.metabolism = URange::new(4, 1);
        assert_eq!(fields(c.validate()), vec!["metabolism"]);
    }

    #[test]
    fn two_peak_map_requires_50_by_50() {
        let mut c = Config::default();
        c.width = 40;
        assert!(fields(c.validate()).contains(&"landscape".to_string()));
    }

    #[test]
    fn vision_cannot_exceed_half_the_grid() {
        let mut c = Config::default();
        c.vision.max = 26;
        assert!(fields(c.validate()).contains(&"vision.max".to_string()));
    }

    #[test]
    fn replacement_excludes_sex_and_needs_lifespan() {
        let mut c = Config::default();
        c.replacement.enabled = true;
        assert!(fields(c.validate()).contains(&"replacement.enabled".to_string()));
        c.lifespan.enabled = true;
        c.validate().unwrap();
        c.sex.enabled = true;
        assert!(fields(c.validate()).contains(&"replacement.enabled".to_string()));
    }

    #[test]
    fn tribes_placement_must_fit() {
        let mut c = Config::default();
        c.placement = Placement::Tribes { size: 30 };
        assert!(fields(c.validate()).contains(&"placement.size".to_string()));
    }

    #[test]
    fn json_round_trips_and_missing_fields_default() {
        let c = Config::default();
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(Config::from_json(&json).unwrap(), c);
        let partial = Config::from_json(r#"{"population": 100}"#).unwrap();
        assert_eq!(partial.population, 100);
        assert_eq!(partial.width, 50);
        assert!(json.contains(r#""landscape":{"kind":"two_peaks"}"#));
    }

    #[test]
    fn bad_json_is_a_config_field_error() {
        let errs = Config::from_json("{not json").unwrap_err();
        assert_eq!(errs[0].field, "config");
    }

    #[test]
    fn structural_changes_are_reported() {
        let a = Config::default();
        let mut b = a.clone();
        b.sex.enabled = true;
        assert!(a.structural_changes(&b).is_empty());
        b.tag_length = 5;
        assert_eq!(a.structural_changes(&b)[0].field, "tag_length");
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p sugarscape-core config`
Expected: FAIL to compile — `Config` not found.

- [ ] **Step 4: Implement config**

Top of `config.rs`:
```rust
//! Every rule parameter and toggle. Rule names follow the book's notation:
//! G_α growback, S_{α,β,γ} seasons, P/D pollution, R_[a,b] replacement,
//! S sex, I inheritance, K culture, C_α combat.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::agent::Sex;

/// Inclusive integer range sampled uniformly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct URange {
    pub min: u32,
    pub max: u32,
}

impl URange {
    pub const fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> u32 {
        rng.gen_range(self.min..=self.max)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LandscapeKind {
    /// The book's 50×50 map with sugar mountains in the northeast and southwest.
    TwoPeaks,
    Flat { capacity: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Placement {
    /// Uniformly random distinct sites.
    Random,
    /// Random sites inside a rectangle (Animation II-6's block of agents).
    Block { x: u32, y: u32, width: u32, height: u32 },
    /// Blues in a `size`×`size` southwest block, Reds in the northeast block.
    Tribes { size: u32 },
}

/// G_α: grow back `rate` per tick up to capacity; `instant` is G_∞.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Growback {
    pub rate: f64,
    pub instant: bool,
}

/// S_{α,β,γ}: summer in the north first; flip every `period` (γ) ticks;
/// winter grows at α / `winter_divisor` (β) per tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seasons {
    pub enabled: bool,
    pub winter_divisor: u32,
    pub period: u32,
}

/// P_{α,β}: pollution += α·gathered + β·metabolized, on the agent's site.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pollution {
    pub enabled: bool,
    pub production: f64,
    pub consumption: f64,
}

/// D_α: every `every` ticks each site's pollution becomes its neighbors' mean.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diffusion {
    pub enabled: bool,
    pub every: u32,
}

/// Death from old age; `max_age` is also R_[a,b]'s [a, b].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lifespan {
    pub enabled: bool,
    pub max_age: URange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Toggle {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SexRule {
    pub enabled: bool,
    pub fertility_onset: URange,
    pub female_end: URange,
    pub male_end: URange,
}

impl SexRule {
    pub fn end_for(&self, sex: Sex) -> URange {
        match sex {
            Sex::Female => self.female_end,
            Sex::Male => self.male_end,
        }
    }
}

/// C_α: reward is min(α, victim's sugar); `unlimited` is C_∞.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatRule {
    pub enabled: bool,
    pub unlimited: bool,
    pub reward: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub landscape: LandscapeKind,
    pub population: u32,
    pub placement: Placement,
    pub vision: URange,
    pub metabolism: URange,
    pub endowment: URange,
    pub tag_length: u32,
    pub growback: Growback,
    pub seasons: Seasons,
    pub pollution: Pollution,
    pub diffusion: Diffusion,
    pub lifespan: Lifespan,
    pub replacement: Toggle,
    pub sex: SexRule,
    pub inheritance: Toggle,
    pub culture: Toggle,
    pub combat: CombatRule,
}

impl Default for Config {
    /// ({G₁}, {M}) on the two-peak map with Chapter II's agent distributions;
    /// every other rule's parameters preset to the book's values but off.
    fn default() -> Self {
        Self {
            width: 50,
            height: 50,
            landscape: LandscapeKind::TwoPeaks,
            population: 400,
            placement: Placement::Random,
            vision: URange::new(1, 6),
            metabolism: URange::new(1, 4),
            endowment: URange::new(5, 25),
            tag_length: 11,
            growback: Growback { rate: 1.0, instant: false },
            seasons: Seasons { enabled: false, winter_divisor: 8, period: 50 },
            pollution: Pollution { enabled: false, production: 1.0, consumption: 1.0 },
            diffusion: Diffusion { enabled: false, every: 1 },
            lifespan: Lifespan { enabled: false, max_age: URange::new(60, 100) },
            replacement: Toggle { enabled: false },
            sex: SexRule {
                enabled: false,
                fertility_onset: URange::new(12, 15),
                female_end: URange::new(40, 50),
                male_end: URange::new(50, 60),
            },
            inheritance: Toggle { enabled: false },
            culture: Toggle { enabled: false },
            combat: CombatRule { enabled: false, unlimited: true, reward: 2.0 },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self { field: field.into(), message: message.into() }
    }
}

#[derive(Default)]
struct Errors(Vec<FieldError>);

impl Errors {
    fn check(&mut self, ok: bool, field: &str, message: impl Into<String>) {
        if !ok {
            self.0.push(FieldError::new(field, message));
        }
    }

    fn range(&mut self, r: URange, field: &str) {
        self.check(r.min <= r.max, field, "min must be ≤ max");
    }

    fn non_negative(&mut self, v: f64, field: &str) {
        self.check(v.is_finite() && v >= 0.0, field, "must be a number ≥ 0");
    }

    fn finish(self) -> Result<(), Vec<FieldError>> {
        if self.0.is_empty() { Ok(()) } else { Err(self.0) }
    }
}

impl Config {
    pub fn from_json(json: &str) -> Result<Self, Vec<FieldError>> {
        let config: Config =
            serde_json::from_str(json).map_err(|e| vec![FieldError::new("config", e.to_string())])?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Errors::default();
        e.check((10..=500).contains(&self.width), "width", "must be between 10 and 500");
        e.check((10..=500).contains(&self.height), "height", "must be between 10 and 500");
        match self.landscape {
            LandscapeKind::TwoPeaks => e.check(
                self.width == 50 && self.height == 50,
                "landscape",
                "the two-peak map is 50×50; set width and height to 50",
            ),
            LandscapeKind::Flat { capacity } => e.non_negative(capacity, "landscape.capacity"),
        }
        let pop = u64::from(self.population);
        match self.placement {
            Placement::Random => e.check(
                pop <= u64::from(self.width) * u64::from(self.height),
                "population",
                "cannot exceed the number of sites",
            ),
            Placement::Block { x, y, width, height } => {
                e.check(
                    width > 0
                        && height > 0
                        && x.saturating_add(width) <= self.width
                        && y.saturating_add(height) <= self.height,
                    "placement",
                    "block must fit inside the grid",
                );
                e.check(
                    pop <= u64::from(width) * u64::from(height),
                    "population",
                    "cannot exceed the block's area",
                );
            }
            Placement::Tribes { size } => {
                e.check(
                    size > 0 && size.saturating_mul(2) <= self.width.min(self.height),
                    "placement.size",
                    "the two corner blocks must fit without overlapping",
                );
                e.check(
                    pop <= 2 * u64::from(size) * u64::from(size),
                    "population",
                    "cannot exceed the two blocks' area",
                );
            }
        }
        let max_vision = self.width.min(self.height) / 2;
        e.range(self.vision, "vision");
        e.check(self.vision.min >= 1, "vision.min", "must be ≥ 1");
        e.check(
            self.vision.max <= max_vision,
            "vision.max",
            format!("must be ≤ {max_vision} (half the grid)"),
        );
        e.range(self.metabolism, "metabolism");
        e.range(self.endowment, "endowment");
        e.check((1..=64).contains(&self.tag_length), "tag_length", "must be between 1 and 64");
        e.check(
            self.growback.rate.is_finite() && self.growback.rate > 0.0,
            "growback.rate",
            "must be a number > 0",
        );
        e.check(self.seasons.winter_divisor >= 1, "seasons.winter_divisor", "must be ≥ 1");
        e.check(self.seasons.period >= 1, "seasons.period", "must be ≥ 1");
        e.non_negative(self.pollution.production, "pollution.production");
        e.non_negative(self.pollution.consumption, "pollution.consumption");
        e.check(self.diffusion.every >= 1, "diffusion.every", "must be ≥ 1");
        e.range(self.lifespan.max_age, "lifespan.max_age");
        e.range(self.sex.fertility_onset, "sex.fertility_onset");
        e.range(self.sex.female_end, "sex.female_end");
        e.range(self.sex.male_end, "sex.male_end");
        e.check(
            !(self.replacement.enabled && self.sex.enabled),
            "replacement.enabled",
            "replacement (R) and sex (S) are mutually exclusive",
        );
        e.check(
            !self.replacement.enabled || self.lifespan.enabled,
            "replacement.enabled",
            "replacement R[a,b] needs lifespan on (it supplies [a,b])",
        );
        e.non_negative(self.combat.reward, "combat.reward");
        e.finish()
    }

    /// Fields that cannot change on a running world (they shape its storage).
    pub fn structural_changes(&self, next: &Config) -> Vec<FieldError> {
        let mut out = Vec::new();
        let msg = "changes only on reset";
        if self.width != next.width {
            out.push(FieldError::new("width", msg));
        }
        if self.height != next.height {
            out.push(FieldError::new("height", msg));
        }
        if self.tag_length != next.tag_length {
            out.push(FieldError::new("tag_length", msg));
        }
        if self.landscape != next.landscape {
            out.push(FieldError::new("landscape", msg));
        }
        out
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core config`
Expected: 9 passed.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add simulation config with validation"
```

---

### Task 3: Landscape, agents and world construction

**Files:**
- Create: `crates/sugarscape-core/assets/sugar-map.txt`, `crates/sugarscape-core/src/landscape.rs`, `crates/sugarscape-core/src/world.rs`, `crates/sugarscape-core/src/testkit.rs`
- Modify: `crates/sugarscape-core/src/agent.rs`, `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Consumes: `Config`, `Placement`, `LandscapeKind`, `URange`, `Torus`, `Pos`, `SimRng`.
- Produces:
  - `landscape::Site { sugar: f64, capacity: f64, pollution: f64 }`, `Site::full(capacity)`, `landscape::capacities(&LandscapeKind, width, height) -> Vec<f64>`
  - `agent::AgentId = u64`; `agent::Tags` with `new(bits, len)`, `random(len, rng)`, `len()`, `bits()`, `get(i)`, `set(i, v)`, `ones()`, `zeros()`, `tribe()`, `forced_to(tribe)`, `to_bit_string()`
  - `agent::Agent` (fields below), `Agent::random(&Config, Pos, born: u64, &mut impl Rng)`, `Agent::tribe()`, `Agent::is_fertile()`
  - `world::World` with pub fields `config, torus, tick, sites, landscape_edited` and `pub(crate) rng, events`; methods `new(Config, u64)`, `with_capacities(Config, u64, Option<&[f64]>)`, `agent`, `agent_mut`, `agents`, `population`, `occupant`, `agent_at`, `is_occupied`, `site`, `site_mut`, `empty_sites`, `insert_agent`, `pub(crate) move_agent`, `events`, `fingerprint`
  - `world::{TickEvents { births: u32, deaths: Vec<Death> }, Death { id, tribe, cause }, DeathCause::{Starvation, OldAge, Combat}}`
  - `testkit::{blank_config, blank_world, spawn, set_sugar}` (cfg(test) only)

- [ ] **Step 1: Add the sugar map asset**

Copy the classic 50×50 two-peak map (50 lines × 50 space-separated integers 0–4, row 0 = north; peaks of capacity 4 around row 5–15/col 34–38 (northeast) and row 40/col 15 (southwest)):
```bash
mkdir -p crates/sugarscape-core/assets
curl -sfL -o crates/sugarscape-core/assets/sugar-map.txt \
  "https://raw.githubusercontent.com/NetLogo/models/master/Sample%20Models/Social%20Science/Economics/Sugarscape/sugar-map.txt"
wc -l crates/sugarscape-core/assets/sugar-map.txt   # expect 50
```
Add a header comment line to `landscape.rs` (not the asset) crediting the source: "Transcription of the book's Figure II-1 map, as distributed with the NetLogo Sugarscape models."

- [ ] **Step 2: Write failing tests for landscape, tags and world construction**

`crates/sugarscape-core/src/landscape.rs`:
```rust
//! Sites of the sugarscape and the built-in capacity maps.
//!
//! The two-peak map is a transcription of the book's Figure II-1, as
//! distributed with the NetLogo Sugarscape models.

use crate::config::LandscapeKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_peak_map_has_peaks_northeast_and_southwest() {
        let caps = capacities(&LandscapeKind::TwoPeaks, 50, 50);
        assert_eq!(caps.len(), 2500);
        let at = |x: usize, y: usize| caps[y * 50 + x];
        assert_eq!(at(37, 5), 4.0, "northeast peak");
        assert_eq!(at(15, 40), 4.0, "southwest peak");
        assert_eq!(at(0, 0), 0.0, "northwest badlands");
        assert!(caps.iter().all(|&c| (0.0..=4.0).contains(&c)));
    }

    #[test]
    fn flat_map_is_uniform() {
        let caps = capacities(&LandscapeKind::Flat { capacity: 2.5 }, 10, 12);
        assert_eq!(caps.len(), 120);
        assert!(caps.iter().all(|&c| c == 2.5));
    }
}
```

Append to `agent.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tribe_is_blue_when_zeros_outnumber_ones() {
        assert_eq!(Tags::new(0b00000000011, 11).tribe(), Tribe::Blue);
        assert_eq!(Tags::new(0b00000111111, 11).tribe(), Tribe::Red);
        // Ties are Red (zeros do not outnumber ones).
        assert_eq!(Tags::new(0b0011, 4).tribe(), Tribe::Red);
    }

    #[test]
    fn set_get_and_bit_string() {
        let mut t = Tags::new(0, 5);
        t.set(1, true);
        t.set(4, true);
        assert!(t.get(1) && !t.get(0));
        assert_eq!(t.to_bit_string(), "01001");
        assert_eq!((t.ones(), t.zeros()), (2, 3));
    }

    #[test]
    fn new_masks_bits_beyond_length() {
        assert_eq!(Tags::new(u64::MAX, 3).ones(), 3);
    }

    #[test]
    fn forced_to_flips_tribe_by_inversion() {
        let blue = Tags::new(0b00001, 5);
        assert_eq!(blue.forced_to(Tribe::Blue), blue);
        let red = blue.forced_to(Tribe::Red);
        assert_eq!(red.tribe(), Tribe::Red);
        assert_eq!(red.to_bit_string(), "01111");
    }
}
```

`crates/sugarscape-core/src/world.rs` (tests at the bottom):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Placement};

    #[test]
    fn default_world_places_400_agents_on_distinct_sites() {
        let w = World::new(Config::default(), 1).unwrap();
        assert_eq!(w.population(), 400);
        let mut positions: Vec<Pos> = w.agents().map(|a| a.pos).collect();
        positions.sort();
        positions.dedup();
        assert_eq!(positions.len(), 400);
        for a in w.agents() {
            assert_eq!(w.occupant(a.pos), Some(a.id));
            assert!((1..=6).contains(&a.vision));
            assert!((1..=4).contains(&a.metabolism));
            assert!((5.0..=25.0).contains(&a.sugar));
            assert_eq!(a.sugar, a.initial_sugar);
        }
        assert_eq!(w.site(Pos::new(37, 5)).sugar, 4.0, "sugar starts at capacity");
    }

    #[test]
    fn tribes_placement_puts_blues_southwest_and_reds_northeast() {
        let mut c = Config::default();
        c.placement = Placement::Tribes { size: 20 };
        let w = World::new(c, 3).unwrap();
        assert_eq!(w.population(), 400);
        for a in w.agents() {
            match a.tribe() {
                Tribe::Blue => assert!(a.pos.x < 20 && a.pos.y >= 30),
                Tribe::Red => assert!(a.pos.x >= 30 && a.pos.y < 20),
            }
        }
    }

    #[test]
    fn same_seed_same_world_different_seed_different_world() {
        let a = World::new(Config::default(), 9).unwrap();
        let b = World::new(Config::default(), 9).unwrap();
        let c = World::new(Config::default(), 10).unwrap();
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), c.fingerprint());
    }

    #[test]
    fn custom_capacities_must_match_grid() {
        let err = World::with_capacities(Config::default(), 1, Some(&[1.0; 10])).err().unwrap();
        assert_eq!(err[0].field, "landscape");
        let w = World::with_capacities(Config::default(), 1, Some(&[2.0; 2500])).unwrap();
        assert!(w.landscape_edited);
        assert_eq!(w.site(Pos::new(0, 0)).capacity, 2.0);
    }

    #[test]
    fn invalid_config_is_rejected() {
        let mut c = Config::default();
        c.population = 10_000;
        assert!(World::new(c, 1).is_err());
    }

    #[test]
    fn insert_rejects_occupied_site() {
        let mut w = crate::testkit::blank_world(5, 5);
        crate::testkit::spawn(&mut w, 2, 2);
        let clone = w.agent_at(Pos::new(2, 2)).unwrap().clone();
        assert!(w.insert_agent(clone).is_err());
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Update `lib.rs`:
```rust
pub mod agent;
pub mod config;
pub mod geometry;
pub mod landscape;
pub mod rng;
pub mod world;

#[cfg(test)]
pub(crate) mod testkit;
```
Run: `cargo test -p sugarscape-core`
Expected: FAIL to compile — `capacities`, `Tags`, `World`, `testkit` missing.

- [ ] **Step 4: Implement landscape**

Insert above tests in `landscape.rs`:
```rust
const TWO_PEAKS: &str = include_str!("../assets/sugar-map.txt");

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Site {
    pub sugar: f64,
    pub capacity: f64,
    pub pollution: f64,
}

impl Site {
    /// A site whose sugar starts at capacity.
    pub fn full(capacity: f64) -> Self {
        Self { sugar: capacity, capacity, pollution: 0.0 }
    }
}

/// Row-major capacities (row 0 = north). `TwoPeaks` is only valid at 50×50
/// (enforced by `Config::validate`).
pub fn capacities(kind: &LandscapeKind, width: u32, height: u32) -> Vec<f64> {
    match *kind {
        LandscapeKind::TwoPeaks => TWO_PEAKS
            .split_whitespace()
            .map(|t| t.parse::<f64>().expect("sugar map holds integers"))
            .collect(),
        LandscapeKind::Flat { capacity } => vec![capacity; (width * height) as usize],
    }
}
```

- [ ] **Step 5: Implement tags and agents**

In `agent.rs`, add below the `Tribe` enum (keep the existing imports; add `use rand::Rng;`, `use crate::config::Config;`, `use crate::geometry::Pos;`):
```rust
pub type AgentId = u64;

/// A cultural tag string of `len` bits (1..=64); bit `i` is tag position `i`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tags {
    bits: u64,
    len: u32,
}

impl Tags {
    pub fn new(bits: u64, len: u32) -> Self {
        assert!((1..=64).contains(&len), "tag length must be 1..=64");
        Self { bits: bits & Self::mask(len), len }
    }

    fn mask(len: u32) -> u64 {
        if len == 64 { u64::MAX } else { (1u64 << len) - 1 }
    }

    pub fn random(len: u32, rng: &mut impl Rng) -> Self {
        Self::new(rng.gen(), len)
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn bits(&self) -> u64 {
        self.bits
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn get(&self, i: u32) -> bool {
        (self.bits >> i) & 1 == 1
    }

    pub fn set(&mut self, i: u32, value: bool) {
        if value {
            self.bits |= 1 << i;
        } else {
            self.bits &= !(1 << i);
        }
    }

    pub fn ones(&self) -> u32 {
        self.bits.count_ones()
    }

    pub fn zeros(&self) -> u32 {
        self.len - self.ones()
    }

    pub fn tribe(&self) -> Tribe {
        if self.zeros() > self.ones() { Tribe::Blue } else { Tribe::Red }
    }

    /// These tags if they already belong to `tribe`, otherwise every bit
    /// inverted. Inversion swaps the zero and one counts, so it changes the
    /// tribe except on an even-length tie, which stays Red.
    pub fn forced_to(self, tribe: Tribe) -> Self {
        if self.tribe() == tribe { self } else { Self::new(!self.bits, self.len) }
    }

    /// Tag position 0 first.
    pub fn to_bit_string(&self) -> String {
        (0..self.len).map(|i| if self.get(i) { '1' } else { '0' }).collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: AgentId,
    pub pos: Pos,
    pub vision: u32,
    pub metabolism: u32,
    pub sugar: f64,
    /// Endowment at birth; the fertility threshold and the basis of a parent's
    /// contribution to a child (half of it).
    pub initial_sugar: f64,
    pub age: u32,
    /// Drawn from `lifespan.max_age` at birth; enforced only while lifespan is on.
    pub max_age: u32,
    pub sex: Sex,
    pub fertility_onset: u32,
    pub fertility_end: u32,
    pub tags: Tags,
    pub parents: Option<[AgentId; 2]>,
    pub children: Vec<AgentId>,
    /// Tick of birth.
    pub born: u64,
}

impl Agent {
    /// A first-generation agent with random genetics, endowment and tags.
    /// `id` is assigned by `World::insert_agent`.
    pub fn random(config: &Config, pos: Pos, born: u64, rng: &mut impl Rng) -> Self {
        let sex = if rng.gen_bool(0.5) { Sex::Female } else { Sex::Male };
        let endowment = f64::from(config.endowment.sample(rng));
        Self {
            id: 0,
            pos,
            vision: config.vision.sample(rng),
            metabolism: config.metabolism.sample(rng),
            sugar: endowment,
            initial_sugar: endowment,
            age: 0,
            max_age: config.lifespan.max_age.sample(rng),
            sex,
            fertility_onset: config.sex.fertility_onset.sample(rng),
            fertility_end: config.sex.end_for(sex).sample(rng),
            tags: Tags::random(config.tag_length, rng),
            parents: None,
            children: Vec::new(),
            born,
        }
    }

    pub fn tribe(&self) -> Tribe {
        self.tags.tribe()
    }

    /// Of childbearing age and holding at least the endowment it was born with.
    pub fn is_fertile(&self) -> bool {
        (self.fertility_onset..=self.fertility_end).contains(&self.age)
            && self.sugar >= self.initial_sugar
    }
}
```

- [ ] **Step 6: Implement the world**

Top of `world.rs`:
```rust
//! The world: lattice, agents, and the tick loop.

use std::collections::BTreeMap;

use rand::seq::SliceRandom;

use crate::agent::{Agent, AgentId, Tribe};
use crate::config::{Config, FieldError, Placement};
use crate::geometry::{Pos, Torus};
use crate::landscape::{self, Site};
use crate::rng::{self, SimRng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathCause {
    Starvation,
    OldAge,
    Combat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Death {
    pub id: AgentId,
    pub tribe: Tribe,
    pub cause: DeathCause,
}

/// What happened during the current (or last completed) tick.
#[derive(Clone, Debug, Default)]
pub struct TickEvents {
    pub births: u32,
    pub deaths: Vec<Death>,
}

pub struct World {
    pub config: Config,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    pub sites: Vec<Site>,
    /// True once the capacities differ from the configured landscape.
    pub landscape_edited: bool,
    agents: BTreeMap<AgentId, Agent>,
    occupancy: Vec<Option<AgentId>>,
    pub(crate) rng: SimRng,
    next_id: AgentId,
    pub(crate) events: TickEvents,
}

impl World {
    pub fn new(config: Config, seed: u64) -> Result<Self, Vec<FieldError>> {
        Self::with_capacities(config, seed, None)
    }

    /// Like `new`, but with explicit row-major capacities (a painted map).
    pub fn with_capacities(
        config: Config,
        seed: u64,
        capacities: Option<&[f64]>,
    ) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let torus = Torus::new(config.width, config.height);
        let caps = match capacities {
            Some(c) if c.len() != torus.len() => {
                return Err(vec![FieldError::new(
                    "landscape",
                    format!("expected {} capacities, got {}", torus.len(), c.len()),
                )])
            }
            Some(c) => c.to_vec(),
            None => landscape::capacities(&config.landscape, config.width, config.height),
        };
        let mut world = World {
            torus,
            tick: 0,
            sites: caps.into_iter().map(Site::full).collect(),
            landscape_edited: capacities.is_some(),
            agents: BTreeMap::new(),
            occupancy: vec![None; torus.len()],
            rng: rng::seeded(seed),
            next_id: 1,
            events: TickEvents::default(),
            config,
        };
        world.populate();
        Ok(world)
    }

    fn populate(&mut self) {
        let n = self.config.population as usize;
        let (w, h) = (self.config.width, self.config.height);
        match self.config.placement {
            Placement::Random => {
                let cells = (0..self.torus.len()).map(|i| self.torus.pos(i)).collect();
                self.place(cells, n, None);
            }
            Placement::Block { x, y, width, height } => {
                self.place(rect(x, y, width, height), n, None);
            }
            Placement::Tribes { size } => {
                let blues = n.div_ceil(2);
                self.place(rect(0, h - size, size, size), blues, Some(Tribe::Blue));
                self.place(rect(w - size, 0, size, size), n - blues, Some(Tribe::Red));
            }
        }
    }

    fn place(&mut self, mut cells: Vec<Pos>, n: usize, tribe: Option<Tribe>) {
        cells.shuffle(&mut self.rng);
        for pos in cells.into_iter().take(n) {
            let mut agent = Agent::random(&self.config, pos, self.tick, &mut self.rng);
            if let Some(t) = tribe {
                agent.tags = agent.tags.forced_to(t);
            }
            self.insert_agent(agent).expect("placement cells are distinct and empty");
        }
    }

    pub fn agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(&id)
    }

    pub fn agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    /// Living agents in id order.
    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.agents.values()
    }

    pub(crate) fn agent_ids(&self) -> Vec<AgentId> {
        self.agents.keys().copied().collect()
    }

    pub fn population(&self) -> usize {
        self.agents.len()
    }

    pub fn occupant(&self, pos: Pos) -> Option<AgentId> {
        self.occupancy[self.torus.index(pos)]
    }

    pub fn agent_at(&self, pos: Pos) -> Option<&Agent> {
        self.occupant(pos).and_then(|id| self.agents.get(&id))
    }

    pub fn is_occupied(&self, pos: Pos) -> bool {
        self.occupant(pos).is_some()
    }

    pub fn site(&self, pos: Pos) -> &Site {
        &self.sites[self.torus.index(pos)]
    }

    pub fn site_mut(&mut self, pos: Pos) -> &mut Site {
        let i = self.torus.index(pos);
        &mut self.sites[i]
    }

    pub fn empty_sites(&self) -> Vec<Pos> {
        (0..self.torus.len())
            .filter(|&i| self.occupancy[i].is_none())
            .map(|i| self.torus.pos(i))
            .collect()
    }

    /// Adds `agent` at its position with a fresh id.
    pub fn insert_agent(&mut self, mut agent: Agent) -> Result<AgentId, String> {
        let i = self.torus.index(agent.pos);
        if self.occupancy[i].is_some() {
            return Err(format!("site ({}, {}) is occupied", agent.pos.x, agent.pos.y));
        }
        let id = self.next_id;
        self.next_id += 1;
        agent.id = id;
        self.occupancy[i] = Some(id);
        self.agents.insert(id, agent);
        Ok(id)
    }

    pub(crate) fn move_agent(&mut self, id: AgentId, to: Pos) {
        let from = self.agents[&id].pos;
        if from == to {
            return;
        }
        let (fi, ti) = (self.torus.index(from), self.torus.index(to));
        assert!(self.occupancy[ti].is_none(), "move onto occupied site");
        self.occupancy[fi] = None;
        self.occupancy[ti] = Some(id);
        self.agents.get_mut(&id).expect("live agent").pos = to;
    }

    pub fn events(&self) -> &TickEvents {
        &self.events
    }

    /// FNV-1a hash of the full dynamic state, for determinism checks.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for s in &self.sites {
            eat(s.sugar.to_bits());
            eat(s.capacity.to_bits());
            eat(s.pollution.to_bits());
        }
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(a.sugar.to_bits());
            eat(u64::from(a.age));
            eat(a.tags.bits());
        }
        h
    }
}

fn rect(x: u32, y: u32, width: u32, height: u32) -> Vec<Pos> {
    (y..y + height).flat_map(|yy| (x..x + width).map(move |xx| Pos::new(xx, yy))).collect()
}
```

`crates/sugarscape-core/src/testkit.rs`:
```rust
//! Builders for tiny hand-crafted worlds in unit tests.

use crate::agent::{Agent, AgentId, Sex, Tags};
use crate::config::{Config, LandscapeKind, Placement, URange};
use crate::geometry::Pos;
use crate::world::World;

/// Every rule off, a flat zero-capacity landscape and no agents.
pub fn blank_config(width: u32, height: u32) -> Config {
    Config {
        width,
        height,
        landscape: LandscapeKind::Flat { capacity: 0.0 },
        population: 0,
        placement: Placement::Random,
        vision: URange::new(1, 1),
        ..Config::default()
    }
}

pub fn blank_world(width: u32, height: u32) -> World {
    World::new(blank_config(width, height), 7).expect("blank config is valid")
}

/// A fertile-aged female with vision 1, metabolism 0, 10 sugar (endowment 10),
/// all-zero tags (Blue). Tweak fields through `world.agent_mut(id)`.
pub fn spawn(world: &mut World, x: u32, y: u32) -> AgentId {
    let agent = Agent {
        id: 0,
        pos: Pos::new(x, y),
        vision: 1,
        metabolism: 0,
        sugar: 10.0,
        initial_sugar: 10.0,
        age: 20,
        max_age: 100,
        sex: Sex::Female,
        fertility_onset: 12,
        fertility_end: 50,
        tags: Tags::new(0, world.config.tag_length),
        parents: None,
        children: Vec::new(),
        born: 0,
    };
    world.insert_agent(agent).expect("test site is empty")
}

/// Puts `sugar` on a site, raising its capacity to match if needed.
pub fn set_sugar(world: &mut World, x: u32, y: u32, sugar: f64) {
    let site = world.site_mut(Pos::new(x, y));
    site.capacity = site.capacity.max(sugar);
    site.sugar = sugar;
}
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all tests pass (geometry 5, config 9, landscape 2, agent 4, world 6).

- [ ] **Step 8: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add landscape, agents and world construction"
```

---

### Task 4: Growback and seasons (G_α, S_{α,β,γ})

**Files:**
- Create: `crates/sugarscape-core/src/rules/mod.rs`, `crates/sugarscape-core/src/rules/growback.rs`
- Modify: `crates/sugarscape-core/src/lib.rs` (add `pub mod rules;`)

**Interfaces:**
- Consumes: `World { config, torus, tick, sites }`.
- Produces: `rules::growback::{rate_at(&Config, tick: u64, y: u32) -> f64, apply(&mut World)}` (both `pub(crate)`).

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/rules/mod.rs`:
```rust
//! The rules of Appendix B, one module each.

pub mod growback;
```

`crates/sugarscape-core/src/rules/growback.rs`:
```rust
//! G_α (growback) and S_{α,β,γ} (seasonal growback).

use crate::config::Config;
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pos;
    use crate::testkit::*;

    fn world_with_empty_site(capacity: f64) -> World {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(3, 3)).capacity = capacity;
        w
    }

    #[test]
    fn grows_by_rate_up_to_capacity() {
        let mut w = world_with_empty_site(3.0);
        apply(&mut w);
        assert_eq!(w.site(Pos::new(3, 3)).sugar, 1.0);
        for _ in 0..5 {
            apply(&mut w);
        }
        assert_eq!(w.site(Pos::new(3, 3)).sugar, 3.0);
    }

    #[test]
    fn instant_growback_refills_immediately() {
        let mut w = world_with_empty_site(4.0);
        w.config.growback.instant = true;
        apply(&mut w);
        assert_eq!(w.site(Pos::new(3, 3)).sugar, 4.0);
    }

    #[test]
    fn seasons_start_with_summer_in_the_north_and_flip_every_period() {
        let mut c = blank_config(10, 10);
        c.seasons.enabled = true;
        c.seasons.winter_divisor = 8;
        c.seasons.period = 50;
        assert_eq!(rate_at(&c, 0, 0), 1.0, "north summer");
        assert_eq!(rate_at(&c, 0, 9), 0.125, "south winter");
        assert_eq!(rate_at(&c, 49, 4), 1.0);
        assert_eq!(rate_at(&c, 50, 4), 0.125, "north winter after γ ticks");
        assert_eq!(rate_at(&c, 50, 5), 1.0, "south summer after γ ticks");
        assert_eq!(rate_at(&c, 100, 0), 1.0, "back to north summer");
    }

    #[test]
    fn without_seasons_rate_is_uniform() {
        let c = blank_config(10, 10);
        assert_eq!(rate_at(&c, 75, 9), 1.0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod rules;` to `lib.rs`. Run: `cargo test -p sugarscape-core growback`
Expected: FAIL to compile — `apply`, `rate_at` missing.

- [ ] **Step 3: Implement**

Insert above tests in `growback.rs`:
```rust
/// Growback rate for row `y` during tick `tick`. With seasons on, the north
/// has summer while `tick mod 2γ < γ` (the book's footnote 33) and the other
/// half has winter, growing at α/β per tick.
pub(crate) fn rate_at(config: &Config, tick: u64, y: u32) -> f64 {
    let base = config.growback.rate;
    let s = &config.seasons;
    if !s.enabled {
        return base;
    }
    let period = u64::from(s.period);
    let north = y < config.height / 2;
    let north_summer = tick % (2 * period) < period;
    if north == north_summer {
        base
    } else {
        base / f64::from(s.winter_divisor)
    }
}

pub(crate) fn apply(world: &mut World) {
    let instant = world.config.growback.instant;
    for i in 0..world.sites.len() {
        let rate = rate_at(&world.config, world.tick, world.torus.pos(i).y);
        let site = &mut world.sites[i];
        site.sugar = if instant { site.capacity } else { (site.sugar + rate).min(site.capacity) };
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core growback`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add growback and seasonal growback rules"
```

---

### Task 5: Movement, metabolism, death and the tick loop

**Files:**
- Create: `crates/sugarscape-core/src/rules/movement.rs`, `crates/sugarscape-core/src/rules/lifecycle.rs`
- Modify: `crates/sugarscape-core/src/rules/mod.rs`, `crates/sugarscape-core/src/world.rs`

**Interfaces:**
- Consumes: Task 3 world API, `growback::apply`.
- Produces:
  - `rules::agent_turn(&mut World, AgentId)` (pub(crate))
  - `rules::movement::act(&mut World, AgentId) -> f64` (sugar gathered from the site)
  - `rules::movement::choose(candidates: &[(Pos, u32, f64)], rng) -> Pos` — max value, then nearest, then random (reused by combat)
  - `rules::lifecycle::{metabolize(&mut World, AgentId, gathered: f64), check_death(&mut World, AgentId) -> bool}`
  - `World::kill(&mut self, AgentId, DeathCause) -> Option<Agent>` (pub(crate)), `World::step(&mut self)`, `World::run(&mut self, ticks: u32)`

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/rules/movement.rs`:
```rust
//! Agent movement rule M (Chapter II), with the pollution-modified welfare
//! s / (1 + p) when pollution is on.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::geometry::Pos;
use crate::rng::SimRng;
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn mover(w: &mut World, vision: u32) -> AgentId {
        let id = spawn(w, 5, 5);
        w.agent_mut(id).unwrap().vision = vision;
        id
    }

    #[test]
    fn moves_to_the_richest_visible_site_and_gathers_it() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 5, 8, 3.0);
        set_sugar(&mut w, 7, 5, 2.0);
        let gathered = act(&mut w, id);
        assert_eq!(gathered, 3.0);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 8));
        assert_eq!(w.agent(id).unwrap().sugar, 13.0);
        assert_eq!(w.site(Pos::new(5, 8)).sugar, 0.0);
        assert_eq!(w.occupant(Pos::new(5, 5)), None);
    }

    #[test]
    fn prefers_the_nearest_of_equal_sites() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 5, 2, 2.0); // distance 3
        set_sugar(&mut w, 7, 5, 2.0); // distance 2
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(7, 5));
    }

    #[test]
    fn cannot_see_diagonally() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 6, 6, 4.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
    }

    #[test]
    fn skips_occupied_sites() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        spawn(&mut w, 5, 7);
        set_sugar(&mut w, 5, 7, 4.0);
        set_sugar(&mut w, 3, 5, 1.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(3, 5));
    }

    #[test]
    fn sees_across_the_wraparound_edge() {
        let mut w = blank_world(11, 11);
        let id = spawn(&mut w, 0, 5);
        set_sugar(&mut w, 10, 5, 1.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(10, 5));
    }

    #[test]
    fn pollution_devalues_sites_when_enabled() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 6, 5, 4.0);
        w.site_mut(Pos::new(6, 5)).pollution = 3.0; // welfare 1
        set_sugar(&mut w, 5, 7, 2.0); // welfare 2
        w.config.pollution.enabled = true;
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7));
    }

    #[test]
    fn choose_breaks_ties_by_distance_then_randomly() {
        let mut rng = crate::rng::seeded(1);
        let a = (Pos::new(0, 0), 2, 3.0);
        let b = (Pos::new(1, 0), 1, 3.0);
        let c = (Pos::new(2, 0), 1, 3.0);
        let mut picks = std::collections::BTreeSet::new();
        for _ in 0..50 {
            picks.insert(choose(&[a, b, c], &mut rng));
        }
        assert_eq!(picks.into_iter().collect::<Vec<_>>(), vec![b.0, c.0]);
    }
}
```

`crates/sugarscape-core/src/rules/lifecycle.rs`:
```rust
//! Metabolism (with pollution formation) and death.

use crate::agent::AgentId;
use crate::world::{DeathCause, World};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    #[test]
    fn metabolism_burns_sugar() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 3;
        metabolize(&mut w, id, 0.0);
        assert_eq!(w.agent(id).unwrap().sugar, 7.0);
    }

    #[test]
    fn starving_agents_die_and_free_their_site() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().sugar = 0.0;
        assert!(check_death(&mut w, id));
        assert!(w.agent(id).is_none());
        assert_eq!(w.occupant(crate::geometry::Pos::new(2, 2)), None);
        assert_eq!(w.events().deaths[0].cause, DeathCause::Starvation);
    }

    #[test]
    fn healthy_agents_live() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        assert!(!check_death(&mut w, id));
        assert!(w.agent(id).is_some());
    }
}
```

Append to `world.rs` tests module:
```rust
    #[test]
    fn step_is_deterministic_for_a_seed() {
        let mut a = World::new(Config::default(), 42).unwrap();
        let mut b = World::new(Config::default(), 42).unwrap();
        a.run(50);
        b.run(50);
        assert_eq!(a.tick, 50);
        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn population_declines_toward_carrying_capacity() {
        let mut w = World::new(Config::default(), 5).unwrap();
        w.run(100);
        assert!(w.population() < 400 && w.population() > 100, "got {}", w.population());
        for a in w.agents() {
            assert!(a.sugar > 0.0);
            assert_eq!(a.age, 100, "immortal first generation ages every tick");
        }
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Update `rules/mod.rs`:
```rust
//! The rules of Appendix B, one module each.

pub mod growback;
pub mod lifecycle;
pub mod movement;

use crate::agent::AgentId;
use crate::world::World;

/// One agent's turn, in the book's order: move, metabolize, maybe die.
/// Later rules (sex, culture, combat) extend this sequence.
pub(crate) fn agent_turn(world: &mut World, id: AgentId) {
    let gathered = movement::act(world, id);
    lifecycle::metabolize(world, id, gathered);
    lifecycle::check_death(world, id);
}
```
Run: `cargo test -p sugarscape-core`
Expected: FAIL to compile — `act`, `choose`, `metabolize`, `check_death`, `kill`, `run` missing.

- [ ] **Step 3: Implement movement**

Insert above tests in `movement.rs`:
```rust
/// Picks among `(site, distance, value)` candidates: highest value, then
/// nearest, then uniformly at random.
pub(crate) fn choose(candidates: &[(Pos, u32, f64)], rng: &mut SimRng) -> Pos {
    let best_value = candidates.iter().map(|c| c.2).fold(f64::NEG_INFINITY, f64::max);
    let nearest = candidates
        .iter()
        .filter(|c| c.2 == best_value)
        .map(|c| c.1)
        .min()
        .expect("at least one candidate");
    let ties: Vec<Pos> = candidates
        .iter()
        .filter(|c| c.2 == best_value && c.1 == nearest)
        .map(|c| c.0)
        .collect();
    *ties.choose(rng).expect("non-empty ties")
}

/// Rule M: look along the four lattice directions as far as vision permits,
/// go to the nearest unoccupied site of maximum welfare and collect its sugar.
/// The agent's current site competes at distance 0, so it stays put when
/// nothing visible is better. Returns the sugar gathered.
pub(crate) fn act(world: &mut World, id: AgentId) -> f64 {
    let agent = world.agent(id).expect("live agent");
    let (pos, vision) = (agent.pos, agent.vision);
    let polluted = world.config.pollution.enabled;
    let welfare = |w: &World, p: Pos| {
        let s = w.site(p);
        if polluted { s.sugar / (1.0 + s.pollution) } else { s.sugar }
    };
    let mut candidates = vec![(pos, 0, welfare(world, pos))];
    for (q, d) in world.torus.sight(pos, vision) {
        if !world.is_occupied(q) {
            candidates.push((q, d, welfare(world, q)));
        }
    }
    let target = choose(&candidates, &mut world.rng);
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let gathered = site.sugar;
    site.sugar = 0.0;
    world.agent_mut(id).expect("live agent").sugar += gathered;
    gathered
}
```

- [ ] **Step 4: Implement lifecycle and the world's tick loop**

Insert above tests in `lifecycle.rs`:
```rust
/// Burns `metabolism` sugar. (Pollution formation is added with rule P.)
pub(crate) fn metabolize(world: &mut World, id: AgentId, _gathered: f64) {
    let agent = world.agent_mut(id).expect("live agent");
    agent.sugar -= f64::from(agent.metabolism);
}

/// Kills the agent if its sugar is at or below zero or, with lifespan on, it
/// has outlived its maximum age. Returns whether it died.
pub(crate) fn check_death(world: &mut World, id: AgentId) -> bool {
    let agent = world.agent(id).expect("live agent");
    let cause = if agent.sugar <= 0.0 {
        Some(DeathCause::Starvation)
    } else if world.config.lifespan.enabled && agent.age > agent.max_age {
        Some(DeathCause::OldAge)
    } else {
        None
    };
    match cause {
        Some(cause) => {
            world.kill(id, cause);
            true
        }
        None => false,
    }
}
```

Add to `impl World` in `world.rs` (and `use crate::rules;` at the top):
```rust
    /// Removes an agent from play and records its death.
    pub(crate) fn kill(&mut self, id: AgentId, cause: DeathCause) -> Option<Agent> {
        let agent = self.agents.remove(&id)?;
        let i = self.torus.index(agent.pos);
        self.occupancy[i] = None;
        self.events.deaths.push(Death { id, tribe: agent.tribe(), cause });
        Some(agent)
    }

    /// One tick: every living agent takes a turn in a fresh random order
    /// (agents born or killed during the tick are skipped), then the
    /// environment updates and everyone ages.
    pub fn step(&mut self) {
        self.events = TickEvents::default();
        let mut order = self.agent_ids();
        order.shuffle(&mut self.rng);
        for id in order {
            if self.agents.contains_key(&id) {
                rules::agent_turn(self, id);
            }
        }
        rules::growback::apply(self);
        for agent in self.agents.values_mut() {
            agent.age += 1;
        }
        self.tick += 1;
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass (movement 7, lifecycle 3, world 8 plus earlier).

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add movement rule M, metabolism, death and the tick loop"
```

---
### Task 6: Pollution formation and diffusion (P_{α,β}, D_α)

**Files:**
- Create: `crates/sugarscape-core/src/rules/pollution.rs`
- Modify: `crates/sugarscape-core/src/rules/lifecycle.rs`, `crates/sugarscape-core/src/rules/mod.rs`, `crates/sugarscape-core/src/world.rs`

**Interfaces:**
- Consumes: `lifecycle::metabolize`, `World::step`.
- Produces: `rules::pollution::diffuse(&mut World)` (pub(crate)); `metabolize` now adds pollution.

- [ ] **Step 1: Write failing tests**

Append to `lifecycle.rs` tests:
```rust
    #[test]
    fn gathering_and_metabolism_pollute_the_site_when_enabled() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 2;
        w.config.pollution.enabled = true;
        w.config.pollution.production = 0.5;
        w.config.pollution.consumption = 3.0;
        metabolize(&mut w, id, 4.0);
        assert_eq!(w.site(crate::geometry::Pos::new(2, 2)).pollution, 0.5 * 4.0 + 3.0 * 2.0);
    }

    #[test]
    fn no_pollution_when_disabled() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 2;
        metabolize(&mut w, id, 4.0);
        assert_eq!(w.site(crate::geometry::Pos::new(2, 2)).pollution, 0.0);
    }
```

`crates/sugarscape-core/src/rules/pollution.rs`:
```rust
//! Pollution diffusion rule D_α: every α ticks, each site's pollution becomes
//! the mean of its four von Neumann neighbors' pollution.

use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pos;
    use crate::testkit::*;

    #[test]
    fn diffusion_spreads_to_neighbors() {
        let mut w = blank_world(10, 10);
        w.config.diffusion.enabled = true;
        w.site_mut(Pos::new(4, 4)).pollution = 4.0;
        diffuse(&mut w);
        assert_eq!(w.site(Pos::new(4, 4)).pollution, 0.0);
        for p in [Pos::new(4, 3), Pos::new(4, 5), Pos::new(3, 4), Pos::new(5, 4)] {
            assert_eq!(w.site(p).pollution, 1.0);
        }
        let total: f64 = w.sites.iter().map(|s| s.pollution).sum();
        assert_eq!(total, 4.0, "diffusion conserves pollution");
    }

    #[test]
    fn diffusion_runs_every_alpha_ticks() {
        let mut w = blank_world(10, 10);
        w.config.diffusion.enabled = true;
        w.config.diffusion.every = 2;
        w.site_mut(Pos::new(4, 4)).pollution = 4.0;
        diffuse(&mut w); // tick 0: (0 + 1) % 2 != 0
        assert_eq!(w.site(Pos::new(4, 4)).pollution, 4.0);
        w.tick = 1;
        diffuse(&mut w);
        assert_eq!(w.site(Pos::new(4, 4)).pollution, 0.0);
    }

    #[test]
    fn no_diffusion_when_disabled() {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(4, 4)).pollution = 4.0;
        diffuse(&mut w);
        assert_eq!(w.site(Pos::new(4, 4)).pollution, 4.0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod pollution;` to `rules/mod.rs`. Run: `cargo test -p sugarscape-core pollution`
Expected: FAIL — `diffuse` missing; the pollution-formation test fails its assertion once compiled.

- [ ] **Step 3: Implement**

Insert above tests in `pollution.rs`:
```rust
pub(crate) fn diffuse(world: &mut World) {
    let d = world.config.diffusion;
    if !d.enabled || (world.tick + 1) % u64::from(d.every) != 0 {
        return;
    }
    let next: Vec<f64> = (0..world.sites.len())
        .map(|i| {
            let p = world.torus.pos(i);
            world.torus.neighbors(p).iter().map(|&q| world.site(q).pollution).sum::<f64>() / 4.0
        })
        .collect();
    for (site, p) in world.sites.iter_mut().zip(next) {
        site.pollution = p;
    }
}
```

Replace `metabolize` in `lifecycle.rs`:
```rust
/// Burns `metabolism` sugar. With rule P on, the agent's site gains
/// production pollution α·gathered plus consumption pollution β·metabolism.
pub(crate) fn metabolize(world: &mut World, id: AgentId, gathered: f64) {
    let agent = world.agent_mut(id).expect("live agent");
    let burned = f64::from(agent.metabolism);
    agent.sugar -= burned;
    let pos = agent.pos;
    let p = world.config.pollution;
    if p.enabled {
        world.site_mut(pos).pollution += p.production * gathered + p.consumption * burned;
    }
}
```

In `World::step`, after `rules::growback::apply(self);` add:
```rust
        rules::pollution::diffuse(self);
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add pollution formation and diffusion"
```

---

### Task 7: Lifespan, replacement and inheritance (R_[a,b], I)

**Files:**
- Create: `crates/sugarscape-core/src/rules/replacement.rs`
- Modify: `crates/sugarscape-core/src/rules/mod.rs`, `crates/sugarscape-core/src/rules/lifecycle.rs`, `crates/sugarscape-core/src/world.rs`

**Interfaces:**
- Consumes: `World::kill`, `TickEvents.deaths`, `Agent::random`, `Tags::forced_to`.
- Produces: `rules::replacement::apply(&mut World)` (pub(crate)); `World::kill` now bequeaths to living children when inheritance is on.

- [ ] **Step 1: Write failing tests**

Append to `lifecycle.rs` tests:
```rust
    #[test]
    fn agents_older_than_max_age_die_only_with_lifespan_on() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().age = 101;
        assert!(!check_death(&mut w, id), "immortal while lifespan is off");
        w.config.lifespan.enabled = true;
        assert!(check_death(&mut w, id));
        assert_eq!(w.events().deaths[0].cause, DeathCause::OldAge);
    }

    #[test]
    fn inheritance_splits_wealth_among_living_children() {
        let mut w = blank_world(5, 5);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 0, 0);
        let c1 = spawn(&mut w, 1, 0);
        let c2 = spawn(&mut w, 2, 0);
        let c3 = spawn(&mut w, 3, 0);
        w.agent_mut(parent).unwrap().children = vec![c1, c2, c3];
        w.agent_mut(parent).unwrap().sugar = 9.0;
        w.kill(c3, DeathCause::Starvation);
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(c1).unwrap().sugar, 14.5);
        assert_eq!(w.agent(c2).unwrap().sugar, 14.5);
    }

    #[test]
    fn no_inheritance_when_disabled() {
        let mut w = blank_world(5, 5);
        let parent = spawn(&mut w, 0, 0);
        let child = spawn(&mut w, 1, 0);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(child).unwrap().sugar, 10.0);
    }
```

`crates/sugarscape-core/src/rules/replacement.rs`:
```rust
//! Agent replacement rule R_[a,b]: each agent that dies is replaced by a
//! random age-0 agent at a random empty site, with max age drawn from [a,b].
//! As in Chapter III's combat runs, the replacement joins the dead agent's
//! tribe.

use rand::seq::SliceRandom;

use crate::agent::{Agent, Tribe};
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn replacing_world() -> World {
        let mut w = blank_world(5, 5);
        w.config.lifespan.enabled = true;
        w.config.replacement.enabled = true;
        w
    }

    #[test]
    fn each_death_is_replaced_by_a_fresh_agent_of_the_same_tribe() {
        let mut w = replacing_world();
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 50;
        w.step();
        assert!(w.agent(id).is_none());
        assert_eq!(w.population(), 1);
        let newcomer = w.agents().next().unwrap();
        assert_ne!(newcomer.id, id);
        assert_eq!(newcomer.tribe(), Tribe::Blue);
        assert_eq!(newcomer.age, 1, "born this tick, then aged with everyone");
        assert!((60..=100).contains(&newcomer.max_age));
    }

    #[test]
    fn no_replacement_when_disabled() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().metabolism = 50;
        w.step();
        assert_eq!(w.population(), 0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod replacement;` to `rules/mod.rs`. Run: `cargo test -p sugarscape-core`
Expected: FAIL — `apply` missing; inheritance test fails its assertion.

- [ ] **Step 3: Implement**

Insert above tests in `replacement.rs`:
```rust
pub(crate) fn apply(world: &mut World) {
    if !world.config.replacement.enabled {
        return;
    }
    let tribes: Vec<Tribe> = world.events.deaths.iter().map(|d| d.tribe).collect();
    for tribe in tribes {
        let empty = world.empty_sites();
        let Some(&pos) = empty.choose(&mut world.rng) else { return };
        let mut agent = Agent::random(&world.config, pos, world.tick, &mut world.rng);
        agent.tags = agent.tags.forced_to(tribe);
        world.insert_agent(agent).expect("chosen site is empty");
    }
}
```

In `world.rs`, replace `kill` and add `bequeath`:
```rust
    /// Removes an agent from play and records its death. With rule I on, its
    /// remaining sugar is split equally among its living children.
    pub(crate) fn kill(&mut self, id: AgentId, cause: DeathCause) -> Option<Agent> {
        let agent = self.agents.remove(&id)?;
        let i = self.torus.index(agent.pos);
        self.occupancy[i] = None;
        self.events.deaths.push(Death { id, tribe: agent.tribe(), cause });
        if self.config.inheritance.enabled {
            self.bequeath(&agent);
        }
        Some(agent)
    }

    fn bequeath(&mut self, agent: &Agent) {
        if agent.sugar <= 0.0 {
            return;
        }
        let heirs: Vec<AgentId> =
            agent.children.iter().copied().filter(|c| self.agents.contains_key(c)).collect();
        if heirs.is_empty() {
            return;
        }
        let share = agent.sugar / heirs.len() as f64;
        for heir in heirs {
            self.agents.get_mut(&heir).expect("living heir").sugar += share;
        }
    }
```

In `World::step`, immediately **before** the ageing loop add:
```rust
        rules::replacement::apply(self);
```
(Replacements then age with everyone and end the tick at age 1, like babies born during agent turns.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add lifespan, replacement and inheritance"
```

---

### Task 8: Sexual reproduction (S)

**Files:**
- Create: `crates/sugarscape-core/src/rules/sex.rs`
- Modify: `crates/sugarscape-core/src/rules/mod.rs`

**Interfaces:**
- Consumes: `Agent::is_fertile`, `World::{insert_agent, occupant, agent, agent_mut}`, `SexRule::end_for`.
- Produces: `rules::sex::act(&mut World, AgentId)` (pub(crate)); `agent_turn` calls it when `config.sex.enabled`.

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/rules/sex.rs`:
```rust
//! Agent sex rule S (Chapter III). For each neighbor, in random order: if the
//! neighbor is fertile and of the opposite sex and either partner has an empty
//! neighboring site, a child is born there.
//!
//! Child: endowment = half of each parent's initial endowment (deducted from
//! the parents); vision, metabolism, max age and fertility onset each come from
//! a random parent (Table III-1); fertility end is drawn from the child's
//! sex-specific range (the ranges differ by sex, so it cannot be inherited
//! across sexes); each culture tag is the parents' shared value or, where they
//! differ, a random parent's.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::agent::{Agent, AgentId, Sex};
use crate::geometry::Pos;
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Tags;
    use crate::testkit::*;

    fn couple(w: &mut World) -> (AgentId, AgentId) {
        let mom = spawn(w, 2, 2);
        let dad = spawn(w, 3, 2);
        {
            let d = w.agent_mut(dad).unwrap();
            d.sex = Sex::Male;
            d.vision = 4;
            d.metabolism = 3;
            d.initial_sugar = 6.0;
            d.sugar = 6.0;
            d.tags = Tags::new(u64::MAX, d.tags.len());
        }
        (mom, dad)
    }

    #[test]
    fn fertile_opposite_sex_neighbors_have_a_child() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        act(&mut w, mom);
        assert_eq!(w.population(), 3);
        assert_eq!(w.events().births, 1);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap().clone();
        assert_eq!(child.parents, Some([mom, dad]));
        assert_eq!(child.initial_sugar, 5.0 + 3.0);
        assert_eq!(child.sugar, 8.0);
        assert_eq!(child.age, 0);
        assert!([1, 4].contains(&child.vision));
        assert!([0, 3].contains(&child.metabolism));
        assert_eq!(w.agent(mom).unwrap().sugar, 5.0);
        assert_eq!(w.agent(dad).unwrap().sugar, 3.0);
        assert_eq!(w.agent(mom).unwrap().children, vec![child.id]);
        assert_eq!(w.agent(dad).unwrap().children, vec![child.id]);
        let near = |p: Pos| {
            w.torus.neighbors(Pos::new(2, 2)).contains(&p)
                || w.torus.neighbors(Pos::new(3, 2)).contains(&p)
        };
        assert!(near(child.pos));
    }

    #[test]
    fn same_sex_neighbors_do_not_reproduce() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        w.agent_mut(dad).unwrap().sex = Sex::Female;
        act(&mut w, mom);
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn infertile_agents_do_not_reproduce() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        w.agent_mut(dad).unwrap().age = 5; // too young
        act(&mut w, mom);
        assert_eq!(w.population(), 2);
        w.agent_mut(dad).unwrap().age = 20;
        w.agent_mut(dad).unwrap().sugar = 5.0; // below its endowment of 6
        act(&mut w, mom);
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn no_child_without_an_empty_neighboring_site() {
        // On a 2×1-ish crowded patch: fill every neighbor of both parents.
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        for p in [(2, 1), (2, 3), (1, 2), (3, 1), (3, 3), (4, 2)] {
            spawn(&mut w, p.0, p.1);
        }
        let before = w.population();
        act(&mut w, mom);
        assert_eq!(w.population(), before);
        let _ = dad;
    }

    #[test]
    fn child_tags_agree_where_parents_agree() {
        let mut w = blank_world(10, 10);
        let (mom, dad) = couple(&mut w);
        w.agent_mut(mom).unwrap().tags = Tags::new(0b101, 11);
        w.agent_mut(dad).unwrap().tags = Tags::new(0b111, 11);
        act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap();
        assert!(child.tags.get(0) && child.tags.get(2));
        assert!((3..11).all(|i| !child.tags.get(i)));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod sex;` to `rules/mod.rs`. Run: `cargo test -p sugarscape-core sex`
Expected: FAIL — `act` missing.

- [ ] **Step 3: Implement**

Insert above tests in `sex.rs`:
```rust
pub(crate) fn act(world: &mut World, id: AgentId) {
    let pos = world.agent(id).expect("live agent").pos;
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        let me = world.agent(id).expect("live agent");
        if !me.is_fertile() {
            return;
        }
        let Some(mate) = world.agent_at(q) else { continue };
        if mate.sex == me.sex || !mate.is_fertile() {
            continue;
        }
        let mate_id = mate.id;
        let mut cradles: Vec<Pos> = world
            .torus
            .neighbors(pos)
            .into_iter()
            .chain(world.torus.neighbors(q))
            .filter(|&p| !world.is_occupied(p))
            .collect();
        cradles.sort();
        cradles.dedup();
        let Some(&cradle) = cradles.choose(&mut world.rng) else { continue };
        birth(world, id, mate_id, cradle);
    }
}

fn birth(world: &mut World, a_id: AgentId, b_id: AgentId, cradle: Pos) {
    let a = world.agent(a_id).expect("parent").clone();
    let b = world.agent(b_id).expect("parent").clone();
    let rng = &mut world.rng;
    let pick = |rng: &mut crate::rng::SimRng, x: u32, y: u32| if rng.gen_bool(0.5) { x } else { y };
    let sex = if rng.gen_bool(0.5) { Sex::Female } else { Sex::Male };
    let mut tags = a.tags;
    for i in 0..tags.len() {
        if a.tags.get(i) != b.tags.get(i) && rng.gen_bool(0.5) {
            tags.set(i, b.tags.get(i));
        }
    }
    let (from_a, from_b) = (a.initial_sugar / 2.0, b.initial_sugar / 2.0);
    let child = Agent {
        id: 0,
        pos: cradle,
        vision: pick(rng, a.vision, b.vision),
        metabolism: pick(rng, a.metabolism, b.metabolism),
        sugar: from_a + from_b,
        initial_sugar: from_a + from_b,
        age: 0,
        max_age: pick(rng, a.max_age, b.max_age),
        sex,
        fertility_onset: pick(rng, a.fertility_onset, b.fertility_onset),
        fertility_end: world.config.sex.end_for(sex).sample(rng),
        tags,
        parents: Some([a_id, b_id]),
        children: Vec::new(),
        born: world.tick,
    };
    world.agent_mut(a_id).expect("parent").sugar -= from_a;
    world.agent_mut(b_id).expect("parent").sugar -= from_b;
    let child_id = world.insert_agent(child).expect("cradle was empty");
    world.agent_mut(a_id).expect("parent").children.push(child_id);
    world.agent_mut(b_id).expect("parent").children.push(child_id);
    world.events.births += 1;
}
```

Update `agent_turn` in `rules/mod.rs` so the death check gates the social rules:
```rust
/// One agent's turn, in the book's order: move, metabolize, maybe die, then
/// (if still alive) mate with each neighbor.
pub(crate) fn agent_turn(world: &mut World, id: AgentId) {
    let gathered = movement::act(world, id);
    lifecycle::metabolize(world, id, gathered);
    if lifecycle::check_death(world, id) {
        return;
    }
    if world.config.sex.enabled {
        sex::act(world, id);
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add sexual reproduction rule S"
```

---

### Task 9: Cultural transmission (K)

**Files:**
- Create: `crates/sugarscape-core/src/rules/culture.rs`
- Modify: `crates/sugarscape-core/src/rules/mod.rs`

**Interfaces:**
- Produces: `rules::culture::act(&mut World, AgentId)` (pub(crate)); `agent_turn` calls it after sex when `config.culture.enabled`.

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/rules/culture.rs`:
```rust
//! Agent culture rule K: for each neighbor, pick a random tag position; if
//! the neighbor disagrees there, flip the neighbor's tag to match the agent's.
//! Group membership (Blue/Red) is derived from tags (`Tags::tribe`).

use rand::seq::SliceRandom;
use rand::Rng;

use crate::agent::AgentId;
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Tags;
    use crate::testkit::*;

    #[test]
    fn flips_exactly_one_tag_of_each_disagreeing_neighbor() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5); // all zeros
        let n1 = spawn(&mut w, 5, 4);
        let n2 = spawn(&mut w, 6, 5);
        for n in [n1, n2] {
            w.agent_mut(n).unwrap().tags = Tags::new(u64::MAX, 11);
        }
        act(&mut w, me);
        assert_eq!(w.agent(n1).unwrap().tags.ones(), 10);
        assert_eq!(w.agent(n2).unwrap().tags.ones(), 10);
        assert_eq!(w.agent(me).unwrap().tags.ones(), 0, "the agent itself is unchanged");
    }

    #[test]
    fn agreeing_neighbors_are_unchanged() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let n = spawn(&mut w, 5, 6);
        act(&mut w, me);
        assert_eq!(w.agent(n).unwrap().tags.ones(), 0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod culture;` to `rules/mod.rs`. Run: `cargo test -p sugarscape-core culture`
Expected: FAIL — `act` missing.

- [ ] **Step 3: Implement**

Insert above tests in `culture.rs`:
```rust
pub(crate) fn act(world: &mut World, id: AgentId) {
    let me = world.agent(id).expect("live agent");
    let (pos, tags) = (me.pos, me.tags);
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        let Some(n) = world.occupant(q) else { continue };
        let i = world.rng.gen_range(0..tags.len());
        world.agent_mut(n).expect("occupant").tags.set(i, tags.get(i));
    }
}
```

Append to `agent_turn` in `rules/mod.rs` (after the sex block):
```rust
    if world.config.culture.enabled {
        culture::act(world, id);
    }
```
and update its doc comment to "…then (if still alive) mate with each neighbor and spread culture to them."

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add cultural transmission rule K"
```

---

### Task 10: Combat (C_α)

**Files:**
- Create: `crates/sugarscape-core/src/rules/combat.rs`
- Modify: `crates/sugarscape-core/src/rules/mod.rs`

**Interfaces:**
- Consumes: `movement::choose`, `World::kill`, `Torus::sight`.
- Produces: `rules::combat::act(&mut World, AgentId) -> f64` (pub(crate); returns sugar gathered from the site, excluding loot); `agent_turn` uses it instead of `movement::act` when `config.combat.enabled`.

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/rules/combat.rs`:
```rust
//! Agent combat rule C_α (Chapter III, Appendix B). Replaces M when on.
//!
//! Candidate sites lie along the four lattice directions within vision.
//! Sites held by the agent's own tribe, or by other-tribe agents at least as
//! wealthy as the agent, are discarded. A site's reward is its sugar plus,
//! if occupied, min(α, occupant's sugar). Sites vulnerable to retaliation are
//! discarded: after taking the site, would some other-tribe agent visible
//! (with the attacker's vision) from the target site be wealthier than the
//! attacker's new wealth? The agent moves to the nearest maximum-reward site,
//! collects the reward, and the former occupant is killed. The victim's sugar
//! beyond the reward passes to its children if rule I is on.
//! The current site competes at distance 0 (the agent may stay put).

use crate::agent::{AgentId, Tribe};
use crate::geometry::Pos;
use crate::rules::movement::choose;
use crate::world::{DeathCause, World};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Tags;
    use crate::testkit::*;

    fn fighting_world() -> World {
        let mut w = blank_world(15, 15);
        w.config.combat.enabled = true;
        w
    }

    fn red(w: &mut World, x: u32, y: u32, sugar: f64) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.tags = Tags::new(u64::MAX, a.tags.len());
        a.sugar = sugar;
        id
    }

    fn blue(w: &mut World, x: u32, y: u32, sugar: f64, vision: u32) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.sugar = sugar;
        a.vision = vision;
        id
    }

    #[test]
    fn unlimited_combat_takes_the_victims_whole_wealth() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let victim = red(&mut w, 5, 7, 3.0);
        set_sugar(&mut w, 5, 7, 1.0);
        let gathered = act(&mut w, me);
        assert_eq!(gathered, 1.0);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(5, 7));
        assert_eq!(w.agent(me).unwrap().sugar, 14.0);
        assert!(w.agent(victim).is_none());
        assert_eq!(w.events().deaths[0].cause, DeathCause::Combat);
    }

    #[test]
    fn capped_reward_takes_at_most_alpha() {
        let mut w = fighting_world();
        w.config.combat.unlimited = false;
        w.config.combat.reward = 2.0;
        let me = blue(&mut w, 5, 5, 10.0, 2);
        red(&mut w, 5, 7, 3.0);
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().sugar, 12.0);
    }

    #[test]
    fn capped_reward_leftovers_pass_to_children_with_inheritance() {
        let mut w = fighting_world();
        w.config.combat.unlimited = false;
        w.config.combat.reward = 2.0;
        w.config.inheritance.enabled = true;
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let victim = red(&mut w, 5, 7, 3.0);
        let orphan = red(&mut w, 12, 12, 1.0);
        w.agent_mut(victim).unwrap().children = vec![orphan];
        act(&mut w, me);
        assert_eq!(w.agent(orphan).unwrap().sugar, 2.0);
    }

    #[test]
    fn never_attacks_own_tribe() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let friend = blue(&mut w, 5, 7, 3.0, 1);
        act(&mut w, me);
        assert!(w.agent(friend).is_some());
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(5, 5));
    }

    #[test]
    fn never_attacks_an_equal_or_wealthier_agent() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let rich = red(&mut w, 5, 7, 10.0);
        act(&mut w, me);
        assert!(w.agent(rich).is_some());
    }

    #[test]
    fn avoids_sites_vulnerable_to_retaliation() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        let victim = red(&mut w, 5, 7, 3.0);
        red(&mut w, 5, 9, 20.0); // visible from (5,7) with vision 2; 20 > 13
        act(&mut w, me);
        assert!(w.agent(victim).is_some());
    }

    #[test]
    fn moves_to_empty_sugar_like_rule_m() {
        let mut w = fighting_world();
        let me = blue(&mut w, 5, 5, 10.0, 2);
        set_sugar(&mut w, 7, 5, 2.0);
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(7, 5));
        assert_eq!(w.agent(me).unwrap().sugar, 12.0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod combat;` to `rules/mod.rs`. Run: `cargo test -p sugarscape-core combat`
Expected: FAIL — `act` missing.

- [ ] **Step 3: Implement**

Insert above tests in `combat.rs`:
```rust
pub(crate) fn act(world: &mut World, id: AgentId) -> f64 {
    let me = world.agent(id).expect("live agent");
    let (pos, vision, tribe, wealth) = (me.pos, me.vision, me.tribe(), me.sugar);
    let cap = if world.config.combat.unlimited { f64::INFINITY } else { world.config.combat.reward };

    let mut candidates = vec![(pos, 0, world.site(pos).sugar)];
    for (q, d) in world.torus.sight(pos, vision) {
        let site_sugar = world.site(q).sugar;
        let reward = match world.agent_at(q) {
            Some(o) if o.tribe() == tribe || o.sugar >= wealth => continue,
            Some(o) => site_sugar + cap.min(o.sugar),
            None => site_sugar,
        };
        if vulnerable(world, id, q, vision, tribe, wealth + reward) {
            continue;
        }
        candidates.push((q, d, reward));
    }
    let target = choose(&candidates, &mut world.rng);

    let mut loot = 0.0;
    if let Some(victim_id) = world.occupant(target).filter(|&v| v != id) {
        let victim = world.agent_mut(victim_id).expect("occupant");
        loot = cap.min(victim.sugar);
        victim.sugar -= loot;
        world.kill(victim_id, DeathCause::Combat);
    }
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let gathered = site.sugar;
    site.sugar = 0.0;
    world.agent_mut(id).expect("live agent").sugar += gathered + loot;
    gathered
}

/// Whether some other-tribe agent visible from `target` would be wealthier
/// than the attacker after the attack.
fn vulnerable(world: &World, attacker: AgentId, target: Pos, vision: u32, tribe: Tribe, after: f64) -> bool {
    world.torus.sight(target, vision).into_iter().any(|(q, _)| {
        world
            .agent_at(q)
            .is_some_and(|o| o.id != attacker && o.tribe() != tribe && o.sugar > after)
    })
}
```

Replace the first line of `agent_turn` in `rules/mod.rs`:
```rust
    let gathered = if world.config.combat.enabled {
        combat::act(world, id)
    } else {
        movement::act(world, id)
    };
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add combat rule C"
```

---

### Task 11: Statistics (series, Gini, Lorenz, histogram)

**Files:**
- Create: `crates/sugarscape-core/src/stats.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/world.rs`

**Interfaces:**
- Consumes: `World::{agents, events, tick}`.
- Produces:
  - `stats::Snapshot { tick: u64, population: u32, gini, mean_wealth, mean_vision, mean_metabolism, blue_fraction: f64, births: u32, deaths: u32 }` (Serialize), `Snapshot::of(&World)`, `Snapshot::value(&str) -> Option<f64>`
  - `stats::SERIES: [&str; 8]` = `population, gini, mean_wealth, mean_vision, mean_metabolism, blue_fraction, births, deaths`
  - `stats::Stats` with `push`, `latest() -> Option<&Snapshot>`, `history() -> &[Snapshot]`, `series(&str) -> Option<Vec<f64>>` (also accepts `"tick"`)
  - `stats::{wealths(&World) -> Vec<f64>, gini(&[f64]) -> f64, lorenz(&[f64], points: usize) -> Vec<f64>, histogram(&[f64], bins: usize) -> (f64, Vec<f64>)}`
  - `World.stats: Stats` (pub field) — initial snapshot at tick 0, one per step.

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/stats.rs`:
```rust
//! Per-tick summary statistics (Chapter II's Gini coefficient and Lorenz
//! curve, plus population and trait means).

use serde::Serialize;

use crate::agent::Tribe;
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn gini_of_equal_wealth_is_zero_and_of_total_concentration_is_high() {
        assert_eq!(gini(&[5.0, 5.0, 5.0, 5.0]), 0.0);
        assert!((gini(&[0.0, 0.0, 0.0, 1.0]) - 0.75).abs() < 1e-12);
        assert_eq!(gini(&[]), 0.0);
        assert_eq!(gini(&[3.0, 1.0, 2.0]), gini(&[1.0, 2.0, 3.0]), "order-independent");
    }

    #[test]
    fn lorenz_curve_of_equal_wealth_is_the_diagonal() {
        let l = lorenz(&[2.0, 2.0, 2.0, 2.0], 5);
        assert_eq!(l, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(lorenz(&[0.0, 0.0, 0.0, 1.0], 5), vec![0.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn histogram_bins_from_zero_to_max() {
        let (width, counts) = histogram(&[0.5, 1.5, 3.9, 4.0], 4);
        assert_eq!(width, 1.0);
        assert_eq!(counts, vec![1.0, 1.0, 0.0, 2.0]);
    }

    #[test]
    fn world_records_a_snapshot_per_tick() {
        let mut w = World::new(Config::default(), 3).unwrap();
        assert_eq!(w.stats.history().len(), 1);
        assert_eq!(w.stats.latest().unwrap().population, 400);
        w.run(3);
        assert_eq!(w.stats.history().len(), 4);
        assert_eq!(w.stats.series("tick").unwrap(), vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(w.stats.series("population").unwrap().len(), 4);
        assert!(w.stats.series("nonsense").is_none());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.population as usize, w.population());
        assert!((1.0..=6.0).contains(&s.mean_vision));
        for name in SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod stats;` to `lib.rs`. Run: `cargo test -p sugarscape-core stats`
Expected: FAIL — items missing.

- [ ] **Step 3: Implement**

Insert above tests in `stats.rs`:
```rust
pub const SERIES: [&str; 8] = [
    "population",
    "gini",
    "mean_wealth",
    "mean_vision",
    "mean_metabolism",
    "blue_fraction",
    "births",
    "deaths",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Snapshot {
    pub tick: u64,
    pub population: u32,
    pub gini: f64,
    pub mean_wealth: f64,
    pub mean_vision: f64,
    pub mean_metabolism: f64,
    pub blue_fraction: f64,
    pub births: u32,
    pub deaths: u32,
}

impl Snapshot {
    pub fn of(world: &World) -> Self {
        let n = world.population();
        let mean = |f: &dyn Fn(&crate::agent::Agent) -> f64| {
            if n == 0 { 0.0 } else { world.agents().map(f).sum::<f64>() / n as f64 }
        };
        let w = wealths(world);
        Self {
            tick: world.tick,
            population: n as u32,
            gini: gini(&w),
            mean_wealth: mean(&|a| a.sugar),
            mean_vision: mean(&|a| f64::from(a.vision)),
            mean_metabolism: mean(&|a| f64::from(a.metabolism)),
            blue_fraction: mean(&|a| if a.tribe() == Tribe::Blue { 1.0 } else { 0.0 }),
            births: world.events().births,
            deaths: world.events().deaths.len() as u32,
        }
    }

    pub fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "gini" => self.gini,
            "mean_wealth" => self.mean_wealth,
            "mean_vision" => self.mean_vision,
            "mean_metabolism" => self.mean_metabolism,
            "blue_fraction" => self.blue_fraction,
            "births" => f64::from(self.births),
            "deaths" => f64::from(self.deaths),
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    history: Vec<Snapshot>,
}

impl Stats {
    pub fn push(&mut self, s: Snapshot) {
        self.history.push(s);
    }

    pub fn latest(&self) -> Option<&Snapshot> {
        self.history.last()
    }

    pub fn history(&self) -> &[Snapshot] {
        &self.history
    }

    /// The full history of one series (or `"tick"`), or `None` if unknown.
    pub fn series(&self, name: &str) -> Option<Vec<f64>> {
        Snapshot::default().value(name)?;
        Some(self.history.iter().map(|s| s.value(name).expect("known series")).collect())
    }
}

pub fn wealths(world: &World) -> Vec<f64> {
    world.agents().map(|a| a.sugar).collect()
}

/// G = 2·Σ i·x₍ᵢ₎ / (n·Σx) − (n+1)/n over ascending wealth, i from 1.
pub fn gini(values: &[f64]) -> f64 {
    let n = values.len();
    let total: f64 = values.iter().sum();
    if n == 0 || total <= 0.0 {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let weighted: f64 = v.iter().enumerate().map(|(i, x)| (i + 1) as f64 * x).sum();
    let n = n as f64;
    (2.0 * weighted / (n * total) - (n + 1.0) / n).max(0.0)
}

/// Share of total wealth held by the poorest k/(points−1) of agents, for
/// k = 0..points.
pub fn lorenz(values: &[f64], points: usize) -> Vec<f64> {
    assert!(points >= 2);
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let total: f64 = v.iter().sum();
    let mut cumulative = vec![0.0];
    for x in &v {
        cumulative.push(cumulative.last().unwrap() + x);
    }
    (0..points)
        .map(|k| {
            if total <= 0.0 {
                return k as f64 / (points - 1) as f64;
            }
            let m = (k as f64 / (points - 1) as f64 * v.len() as f64).round() as usize;
            cumulative[m] / total
        })
        .collect()
}

/// `bins` equal-width bins over [0, max]; returns (bin width, counts).
pub fn histogram(values: &[f64], bins: usize) -> (f64, Vec<f64>) {
    assert!(bins >= 1);
    let max = values.iter().copied().fold(0.0, f64::max);
    let width = if max > 0.0 { max / bins as f64 } else { 1.0 };
    let mut counts = vec![0.0; bins];
    for &x in values {
        let i = ((x.max(0.0) / width) as usize).min(bins - 1);
        counts[i] += 1.0;
    }
    (width, counts)
}
```

In `world.rs`: add `use crate::stats::{Snapshot, Stats};`, a field `pub stats: Stats,` (initialize `stats: Stats::default(),` in the struct literal), push the initial snapshot at the end of `with_capacities` before `Ok(world)`:
```rust
        world.stats.push(Snapshot::of(&world));
```
and at the end of `step()` after `self.tick += 1;`:
```rust
        let snapshot = Snapshot::of(self);
        self.stats.push(snapshot);
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add per-tick statistics, Gini, Lorenz and histograms"
```

---
### Task 12: Rendering to an RGBA frame

**Files:**
- Create: `crates/sugarscape-core/src/render.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Produces:
  - `render::ColorMode::{Tribe, Wealth, Sex, Age, Vision}` and `render::Layer::{Sugar, Capacity, Pollution}`, both `FromStr` from snake_case names (`"tribe"`, `"wealth"`, `"sex"`, `"age"`, `"vision"`; `"sugar"`, `"capacity"`, `"pollution"`), error `String`
  - `render::render(&World, ColorMode, Layer, &mut Vec<u8>)` — resizes to `width*height*4`, one RGBA pixel per site, row-major, row 0 = north
  - Color constants `BACKGROUND, SUGAR, POLLUTION, BLUE, RED, FEMALE, MALE, COOL, HOT: [u8; 3]`

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/render.rs`:
```rust
//! Rasterizes the world into an RGBA buffer: the landscape layer as a
//! background, agents drawn over their sites in the chosen color mode.

use std::str::FromStr;

use crate::agent::{Agent, Sex, Tribe};
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pos;
    use crate::testkit::*;

    fn pixel(buf: &[u8], w: &World, x: u32, y: u32) -> [u8; 4] {
        let i = w.torus.index(Pos::new(x, y)) * 4;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn landscape_shades_from_background_to_sugar() {
        let mut w = blank_world(10, 10);
        set_sugar(&mut w, 1, 1, 4.0);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Sugar, &mut buf);
        assert_eq!(buf.len(), 10 * 10 * 4);
        assert_eq!(pixel(&buf, &w, 1, 1), [SUGAR[0], SUGAR[1], SUGAR[2], 255]);
        assert_eq!(pixel(&buf, &w, 2, 2), [BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]);
    }

    #[test]
    fn pollution_layer_uses_pollution_color() {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(3, 3)).pollution = 2.0;
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Pollution, &mut buf);
        assert_eq!(pixel(&buf, &w, 3, 3)[..3], POLLUTION);
    }

    #[test]
    fn agents_are_drawn_in_their_mode_color() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 4, 4);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Sugar, &mut buf);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], BLUE);
        render(&w, ColorMode::Sex, Layer::Sugar, &mut buf);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], FEMALE);
        w.agent_mut(id).unwrap().sex = Sex::Male;
        render(&w, ColorMode::Sex, Layer::Sugar, &mut buf);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], MALE);
    }

    #[test]
    fn modes_and_layers_parse_from_names() {
        assert_eq!("wealth".parse::<ColorMode>().unwrap(), ColorMode::Wealth);
        assert_eq!("capacity".parse::<Layer>().unwrap(), Layer::Capacity);
        assert!("plaid".parse::<ColorMode>().is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod render;` to `lib.rs`. Run: `cargo test -p sugarscape-core render`
Expected: FAIL — items missing.

- [ ] **Step 3: Implement**

Insert above tests in `render.rs`:
```rust
pub type Rgb = [u8; 3];

pub const BACKGROUND: Rgb = [0x16, 0x15, 0x12];
pub const SUGAR: Rgb = [0xf2, 0xc1, 0x4e];
pub const POLLUTION: Rgb = [0x9b, 0x6b, 0xd6];
pub const BLUE: Rgb = [0x3d, 0x7e, 0xff];
pub const RED: Rgb = [0xff, 0x4d, 0x4d];
pub const FEMALE: Rgb = [0xff, 0x7a, 0xc6];
pub const MALE: Rgb = [0x36, 0xd6, 0xc3];
/// Ends of the ramp used for wealth, age and vision.
pub const COOL: Rgb = [0x4f, 0x9d, 0xff];
pub const HOT: Rgb = [0xff, 0x3d, 0x8b];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Tribe,
    Wealth,
    Sex,
    Age,
    Vision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Sugar,
    Capacity,
    Pollution,
}

impl FromStr for ColorMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "tribe" => Self::Tribe,
            "wealth" => Self::Wealth,
            "sex" => Self::Sex,
            "age" => Self::Age,
            "vision" => Self::Vision,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

impl FromStr for Layer {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "sugar" => Self::Sugar,
            "capacity" => Self::Capacity,
            "pollution" => Self::Pollution,
            _ => return Err(format!("unknown layer {s:?}")),
        })
    }
}

pub fn lerp(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
    std::array::from_fn(|i| (f64::from(a[i]) + (f64::from(b[i]) - f64::from(a[i])) * t).round() as u8)
}

struct Scales {
    log_max_wealth: f64,
    vision_min: f64,
    vision_span: f64,
}

fn agent_color(a: &Agent, mode: ColorMode, s: &Scales) -> Rgb {
    match mode {
        ColorMode::Tribe => match a.tribe() {
            Tribe::Blue => BLUE,
            Tribe::Red => RED,
        },
        ColorMode::Sex => match a.sex {
            Sex::Female => FEMALE,
            Sex::Male => MALE,
        },
        ColorMode::Wealth => lerp(COOL, HOT, a.sugar.max(0.0).ln_1p() / s.log_max_wealth),
        ColorMode::Age => lerp(COOL, HOT, f64::from(a.age) / f64::from(a.max_age.max(1))),
        ColorMode::Vision => lerp(COOL, HOT, (f64::from(a.vision) - s.vision_min) / s.vision_span),
    }
}

pub fn render(world: &World, mode: ColorMode, layer: Layer, buf: &mut Vec<u8>) {
    buf.resize(world.sites.len() * 4, 0);
    let max_capacity = world.sites.iter().map(|s| s.capacity).fold(0.0, f64::max).max(1.0);
    let max_pollution = world.sites.iter().map(|s| s.pollution).fold(0.0, f64::max).max(1e-9);
    for (i, site) in world.sites.iter().enumerate() {
        let rgb = match layer {
            Layer::Sugar => lerp(BACKGROUND, SUGAR, site.sugar / max_capacity),
            Layer::Capacity => lerp(BACKGROUND, SUGAR, site.capacity / max_capacity),
            Layer::Pollution => lerp(BACKGROUND, POLLUTION, site.pollution / max_pollution),
        };
        buf[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    let v = world.config.vision;
    let scales = Scales {
        log_max_wealth: world.agents().map(|a| a.sugar).fold(0.0, f64::max).ln_1p().max(1e-9),
        vision_min: f64::from(v.min),
        vision_span: f64::from(v.max.saturating_sub(v.min).max(1)),
    };
    for a in world.agents() {
        let i = world.torus.index(a.pos) * 4;
        let rgb = agent_color(a, mode, &scales);
        buf[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core render`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Render the world to an RGBA frame"
```

---

### Task 13: Editing, inspection, live config changes and CSV export

**Files:**
- Create: `crates/sugarscape-core/src/edit.rs`, `crates/sugarscape-core/src/export.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Consumes: World API, `Config::{validate, structural_changes}`, `Stats`, `SERIES`.
- Produces (all `impl World` in `edit.rs` unless noted):
  - `edit::AgentOverrides { vision, metabolism: Option<u32>, sugar: Option<f64>, sex: Option<Sex>, tribe: Option<Tribe> }` (Deserialize, `#[serde(default)]`)
  - `edit::{Inspection { site: SiteView, agent: Option<AgentView> }, SiteView, AgentView, LinkView { id, alive }}` (Serialize)
  - `World::paint_capacity(x, y, radius: u32, value: f64) -> Result<(), String>`
  - `World::place_agent(x, y, &AgentOverrides) -> Result<AgentId, String>`
  - `World::remove_agent(x, y) -> Result<(), String>`
  - `World::inspect(x, y) -> Result<Inspection, String>`
  - `World::locate(AgentId) -> Option<Pos>`
  - `World::set_config(Config) -> Result<(), Vec<FieldError>>`
  - `World::capacities() -> Vec<f64>`
  - `export::{series_csv(&Stats) -> String, agents_csv(&World) -> String}`

- [ ] **Step 1: Write failing tests**

`crates/sugarscape-core/src/edit.rs`:
```rust
//! Interactive edits and inspection used by the playground UI.

use serde::{Deserialize, Serialize};

use crate::agent::{Agent, AgentId, Sex, Tribe};
use crate::config::{Config, FieldError};
use crate::geometry::Pos;
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    #[test]
    fn painting_sets_capacity_in_a_disc_and_clamps_sugar() {
        let mut w = blank_world(20, 20);
        for i in 0..w.sites.len() {
            w.sites[i].capacity = 4.0;
            w.sites[i].sugar = 4.0;
        }
        w.paint_capacity(10, 10, 1, 1.0).unwrap();
        assert!(w.landscape_edited);
        for p in [(10, 10), (10, 9), (11, 10), (9, 10), (10, 11)] {
            let s = w.site(Pos::new(p.0, p.1));
            assert_eq!((s.capacity, s.sugar), (1.0, 1.0), "{p:?}");
        }
        assert_eq!(w.site(Pos::new(11, 11)).capacity, 4.0, "radius 1 disc excludes diagonals");
        assert!(w.paint_capacity(20, 0, 1, 1.0).is_err());
    }

    #[test]
    fn placing_and_removing_agents() {
        let mut w = blank_world(10, 10);
        let overrides = AgentOverrides { vision: Some(3), sex: Some(Sex::Male), tribe: Some(Tribe::Red), ..Default::default() };
        let id = w.place_agent(2, 3, &overrides).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!((a.vision, a.sex, a.tribe()), (3, Sex::Male, Tribe::Red));
        assert!(w.place_agent(2, 3, &AgentOverrides::default()).is_err(), "occupied");
        w.remove_agent(2, 3).unwrap();
        assert_eq!(w.population(), 0);
        assert!(w.events().deaths.is_empty(), "removal is not a death");
        assert!(w.remove_agent(2, 3).is_err());
    }

    #[test]
    fn inspect_reports_site_agent_and_lineage() {
        let mut w = blank_world(10, 10);
        let parent = spawn(&mut w, 1, 1);
        let child = spawn(&mut w, 2, 1);
        w.agent_mut(child).unwrap().parents = Some([parent, 999]);
        w.agent_mut(parent).unwrap().children = vec![child];
        let i = w.inspect(2, 1).unwrap();
        let a = i.agent.unwrap();
        assert_eq!(a.id, child);
        assert_eq!(a.tags, "00000000000");
        assert_eq!(a.parents.len(), 2);
        assert!(a.parents[0].alive && !a.parents[1].alive);
        assert!(w.inspect(5, 5).unwrap().agent.is_none());
        assert_eq!(w.locate(child), Some(Pos::new(2, 1)));
        assert_eq!(w.locate(999), None);
    }

    #[test]
    fn set_config_applies_rule_changes_but_rejects_structural_ones() {
        let mut w = blank_world(10, 10);
        let mut next = w.config.clone();
        next.culture.enabled = true;
        w.set_config(next.clone()).unwrap();
        assert!(w.config.culture.enabled);
        next.width = 12;
        let errs = w.set_config(next).unwrap_err();
        assert_eq!(errs[0].field, "width");
        let mut bad = w.config.clone();
        bad.metabolism.min = 9;
        bad.metabolism.max = 1;
        assert_eq!(w.set_config(bad).unwrap_err()[0].field, "metabolism");
    }
}
```

`crates/sugarscape-core/src/export.rs`:
```rust
//! CSV exports of the statistics history and the current agents.

use std::fmt::Write;

use crate::stats::{Stats, SERIES};
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn series_csv_has_a_header_and_a_row_per_tick() {
        let mut w = World::new(Config::default(), 1).unwrap();
        w.run(2);
        let csv = series_csv(&w.stats);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "tick,population,gini,mean_wealth,mean_vision,mean_metabolism,blue_fraction,births,deaths");
        assert_eq!(lines.len(), 4);
        assert!(lines[1].starts_with("0,400,"));
    }

    #[test]
    fn agents_csv_has_a_row_per_agent() {
        let w = World::new(Config::default(), 1).unwrap();
        let csv = agents_csv(&w);
        assert_eq!(csv.lines().count(), 401);
        assert!(csv.starts_with("id,x,y,sex,age,max_age,vision,metabolism,sugar,initial_sugar,tribe,tags\n"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Add `pub mod edit;` and `pub mod export;` to `lib.rs`. Run: `cargo test -p sugarscape-core edit export`
Expected: FAIL — items missing. (If cargo rejects two filters, run `cargo test -p sugarscape-core` instead.)

- [ ] **Step 3: Implement editing**

Insert above tests in `edit.rs`:
```rust
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct AgentOverrides {
    pub vision: Option<u32>,
    pub metabolism: Option<u32>,
    pub sugar: Option<f64>,
    pub sex: Option<Sex>,
    pub tribe: Option<Tribe>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteView {
    pub x: u32,
    pub y: u32,
    pub sugar: f64,
    pub capacity: f64,
    pub pollution: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinkView {
    pub id: AgentId,
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentView {
    pub id: AgentId,
    pub x: u32,
    pub y: u32,
    pub sex: Sex,
    pub tribe: Tribe,
    pub tags: String,
    pub vision: u32,
    pub metabolism: u32,
    pub sugar: f64,
    pub initial_sugar: f64,
    pub age: u32,
    pub max_age: u32,
    pub fertile: bool,
    pub fertility_onset: u32,
    pub fertility_end: u32,
    pub born: u64,
    pub parents: Vec<LinkView>,
    pub children: Vec<LinkView>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Inspection {
    pub site: SiteView,
    pub agent: Option<AgentView>,
}

impl World {
    fn checked_pos(&self, x: u32, y: u32) -> Result<Pos, String> {
        if x < self.torus.width && y < self.torus.height {
            Ok(Pos::new(x, y))
        } else {
            Err(format!("({x}, {y}) is outside the grid"))
        }
    }

    /// Sets capacity to `value` on every site within Euclidean `radius` of
    /// (x, y) (wrapping), clamping sugar to the new capacity.
    pub fn paint_capacity(&mut self, x: u32, y: u32, radius: u32, value: f64) -> Result<(), String> {
        let center = self.checked_pos(x, y)?;
        if !(value.is_finite() && value >= 0.0) {
            return Err("capacity must be ≥ 0".into());
        }
        let r = radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let site = self.site_mut(self.torus.offset(center, dx, dy));
                site.capacity = value;
                site.sugar = site.sugar.min(value);
            }
        }
        self.landscape_edited = true;
        Ok(())
    }

    pub fn place_agent(&mut self, x: u32, y: u32, o: &AgentOverrides) -> Result<AgentId, String> {
        let pos = self.checked_pos(x, y)?;
        let mut agent = Agent::random(&self.config, pos, self.tick, &mut self.rng);
        if let Some(v) = o.vision {
            agent.vision = v;
        }
        if let Some(m) = o.metabolism {
            agent.metabolism = m;
        }
        if let Some(s) = o.sugar {
            agent.sugar = s;
            agent.initial_sugar = s;
        }
        if let Some(sex) = o.sex {
            agent.sex = sex;
            agent.fertility_end = self.config.sex.end_for(sex).sample(&mut self.rng);
        }
        if let Some(t) = o.tribe {
            agent.tags = agent.tags.forced_to(t);
        }
        self.insert_agent(agent)
    }

    /// Removes the agent at (x, y) without counting a death.
    pub fn remove_agent(&mut self, x: u32, y: u32) -> Result<(), String> {
        let pos = self.checked_pos(x, y)?;
        let id = self.occupant(pos).ok_or_else(|| format!("no agent at ({x}, {y})"))?;
        self.kill(id, crate::world::DeathCause::Starvation);
        self.events.deaths.pop();
        Ok(())
    }

    pub fn locate(&self, id: AgentId) -> Option<Pos> {
        self.agent(id).map(|a| a.pos)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<Inspection, String> {
        let pos = self.checked_pos(x, y)?;
        let s = self.site(pos);
        let link = |id: AgentId| LinkView { id, alive: self.agent(id).is_some() };
        let agent = self.agent_at(pos).map(|a| AgentView {
            id: a.id,
            x: a.pos.x,
            y: a.pos.y,
            sex: a.sex,
            tribe: a.tribe(),
            tags: a.tags.to_bit_string(),
            vision: a.vision,
            metabolism: a.metabolism,
            sugar: a.sugar,
            initial_sugar: a.initial_sugar,
            age: a.age,
            max_age: a.max_age,
            fertile: a.is_fertile(),
            fertility_onset: a.fertility_onset,
            fertility_end: a.fertility_end,
            born: a.born,
            parents: a.parents.map(|p| p.iter().map(|&id| link(id)).collect()).unwrap_or_default(),
            children: a.children.iter().map(|&id| link(id)).collect(),
        });
        Ok(Inspection {
            site: SiteView { x, y, sugar: s.sugar, capacity: s.capacity, pollution: s.pollution },
            agent,
        })
    }

    /// Swaps in a new config mid-run. Rule toggles and parameters take effect
    /// on the next tick; grid size, tag length and landscape need a reset.
    pub fn set_config(&mut self, next: Config) -> Result<(), Vec<FieldError>> {
        next.validate()?;
        let structural = self.config.structural_changes(&next);
        if !structural.is_empty() {
            return Err(structural);
        }
        self.config = next;
        Ok(())
    }

    pub fn capacities(&self) -> Vec<f64> {
        self.sites.iter().map(|s| s.capacity).collect()
    }
}
```
Note: `remove_agent` would, with inheritance on, bequeath the removed agent's sugar. That is acceptable (the UI's "erase" behaves like a death for the family) and keeps one removal path; the popped death record keeps replacement from respawning it.

- [ ] **Step 4: Implement export**

Insert above tests in `export.rs`:
```rust
pub fn series_csv(stats: &Stats) -> String {
    let mut out = String::from("tick");
    for name in SERIES {
        out.push(',');
        out.push_str(name);
    }
    out.push('\n');
    for s in stats.history() {
        out.push_str(&s.tick.to_string());
        for name in SERIES {
            write!(out, ",{}", s.value(name).expect("known series")).unwrap();
        }
        out.push('\n');
    }
    out
}

pub fn agents_csv(world: &World) -> String {
    let mut out =
        String::from("id,x,y,sex,age,max_age,vision,metabolism,sugar,initial_sugar,tribe,tags\n");
    for a in world.agents() {
        writeln!(
            out,
            "{},{},{},{:?},{},{},{},{},{},{},{:?},{}",
            a.id,
            a.pos.x,
            a.pos.y,
            a.sex,
            a.age,
            a.max_age,
            a.vision,
            a.metabolism,
            a.sugar,
            a.initial_sugar,
            a.tribe(),
            a.tags.to_bit_string()
        )
        .unwrap();
    }
    out
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add editing, inspection, live config changes and CSV export"
```

---

### Task 14: Presets, invariants and book reproductions

**Files:**
- Create: `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/invariants.rs`, `crates/sugarscape-core/tests/book.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Produces: `presets::Preset { id: &'static str, name: &'static str, source: &'static str, description: &'static str, config: Config }` (Serialize), `presets::all() -> Vec<Preset>`, `presets::by_id(&str) -> Option<Preset>`.

Preset ids (used by the web UI and tests): `ii-1-instant`, `ii-2-unit`, `ii-5-wealth`, `ii-6-waves`, `ii-7-seasons`, `ii-8-pollution`, `iii-2-sex`, `iii-4-inheritance`, `iii-6-culture`, `iii-9-combat`, `iii-11-combat-fixed`, `iii-12-collision`, `iii-14-combat-culture`.

- [ ] **Step 1: Write failing preset tests**

`crates/sugarscape-core/src/presets.rs`:
```rust
//! The book's named rule systems. Parameters come from the text of
//! Chapters II–III; `source` cites the animation each reproduces.

use serde::Serialize;

use crate::config::{Config, Placement, URange};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[test]
    fn every_preset_is_valid_and_runs() {
        let presets = all();
        assert_eq!(presets.len(), 13);
        for p in presets {
            p.config.validate().unwrap_or_else(|e| panic!("{}: {e:?}", p.id));
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
}
```

- [ ] **Step 2: Run to verify failure**

Add `pub mod presets;` to `lib.rs`. Run: `cargo test -p sugarscape-core presets`
Expected: FAIL — `all`, `by_id` missing.

- [ ] **Step 3: Implement presets**

Insert above tests in `presets.rs`:
```rust
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
    Preset { id, name, source, description, config }
}

/// The two-tribe setup used by the combat runs: Blues southwest, Reds northeast.
fn tribes(c: &mut Config) {
    c.placement = Placement::Tribes { size: 20 };
}

/// Chapter III's demographic parameters (sex with finite lifetimes).
fn demography(c: &mut Config) {
    c.sex.enabled = true;
    c.lifespan.enabled = true;
    c.endowment = URange::new(50, 100);
}

pub fn all() -> Vec<Preset> {
    vec![
        preset("ii-1-instant", "({G∞}, {M})", "Animation II-1",
            "Instant growback: agents climb to the best ridge they can see and settle; the poorly endowed starve.",
            |c| c.growback.instant = true),
        preset("ii-2-unit", "({G₁}, {M})", "Animation II-2",
            "Unit growback: continuous hiving on the two sugar mountains; population falls to a carrying capacity near 224.",
            |_| {}),
        preset("ii-5-wealth", "({G₁}, {M, R[60,100]})", "Animations II-4/II-5",
            "Finite lifetimes with replacement: a skewed wealth distribution emerges; watch the Lorenz curve and Gini coefficient.",
            |c| {
                c.lifespan.enabled = true;
                c.replacement.enabled = true;
            }),
        preset("ii-6-waves", "Diagonal waves", "Animation II-6",
            "A block of high-vision agents in the southwest propagates northeast in collective waves no individual can move in.",
            |c| {
                c.placement = Placement::Block { x: 0, y: 25, width: 25, height: 25 };
                c.vision = URange::new(1, 10);
            }),
        preset("ii-7-seasons", "({S₁,₈,₅₀}, {M})", "Animation II-7",
            "Seasons flip every 50 ticks: high-vision agents migrate, low-vision low-metabolism agents hibernate.",
            |c| c.seasons.enabled = true),
        preset("ii-8-pollution", "({G₁, D₁}, {M, P₁₁})", "Animation II-8",
            "Gathering and eating pollute; diffusion spreads it. (The book switches pollution on at t = 50 and diffusion at t = 100; toggle them yourself to replay that.)",
            |c| {
                c.pollution.enabled = true;
                c.diffusion.enabled = true;
            }),
        preset("iii-2-sex", "({G₁}, {M, S})", "Animations III-1/III-2",
            "Sexual reproduction with finite lifetimes: a roughly stable population made of many generations.",
            demography),
        preset("iii-4-inheritance", "({G₁}, {M, S, I})", "Animation III-4",
            "Inheritance passes wealth to children; compare the Gini coefficient with and without it.",
            |c| {
                demography(c);
                c.inheritance.enabled = true;
            }),
        preset("iii-6-culture", "({G₁}, {M, K})", "Animations III-6/III-7",
            "Cultural transmission: tag-flipping converts spatially separated groups toward uniform tribes.",
            |c| c.culture.enabled = true),
        preset("iii-9-combat", "({G₁}, {C∞})", "Animation III-9",
            "Unlimited combat between two tribes: the winner accumulates its victims' whole wealth.",
            |c| {
                tribes(c);
                c.combat.enabled = true;
            }),
        preset("iii-11-combat-fixed", "({G₁}, {C₂, R[60,100]})", "Animation III-11",
            "Fixed reward of 2 per kill, population held at 400 by replacement: prolonged battle fronts.",
            |c| {
                tribes(c);
                c.combat.enabled = true;
                c.combat.unlimited = false;
                c.combat.reward = 2.0;
                c.lifespan.enabled = true;
                c.replacement.enabled = true;
            }),
        preset("iii-12-collision", "Colliding waves", "Animation III-12",
            "Opposed blocks of high-vision Blues and Reds propagate toward the center and interpenetrate (combat off).",
            |c| {
                tribes(c);
                c.vision = URange::new(1, 10);
            }),
        preset("iii-14-combat-culture", "({G₁}, {C∞, K})", "Animation III-14",
            "Combat with cultural transmission: conquest and conversion together.",
            |c| {
                tribes(c);
                c.combat.enabled = true;
                c.culture.enabled = true;
            }),
    ]
}

pub fn by_id(id: &str) -> Option<Preset> {
    all().into_iter().find(|p| p.id == id)
}
```
(`cargo fmt` will reflow the `preset(...)` calls; that is fine.)

- [ ] **Step 4: Run preset tests**

Run: `cargo test -p sugarscape-core presets`
Expected: 2 passed.

- [ ] **Step 5: Write invariant property tests**

`crates/sugarscape-core/tests/invariants.rs`:
```rust
//! Properties that must hold after every tick for any rule combination.

use proptest::prelude::*;
use sugarscape_core::config::Config;
use sugarscape_core::world::World;

fn config_strategy() -> impl Strategy<Value = Config> {
    (
        50u32..=400,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
        proptest::bool::ANY,
    )
        .prop_map(|(pop, seasons, pollution, diffusion, lifespan, sex, inheritance, culture, combat, replacement)| {
            let mut c = Config::default();
            c.population = pop;
            c.seasons.enabled = seasons;
            c.pollution.enabled = pollution;
            c.diffusion.enabled = diffusion;
            c.lifespan.enabled = lifespan;
            c.sex.enabled = sex;
            c.inheritance.enabled = inheritance;
            c.culture.enabled = culture;
            c.combat.enabled = combat;
            c.replacement.enabled = replacement && lifespan && !sex;
            if sex {
                c.endowment.min = 50;
                c.endowment.max = 100;
            }
            c
        })
}

fn check(world: &World) -> Result<(), TestCaseError> {
    let mut occupied = 0;
    for i in 0..world.sites.len() {
        let pos = world.torus.pos(i);
        let site = world.site(pos);
        prop_assert!(site.sugar <= site.capacity + 1e-9, "sugar above capacity at {pos:?}");
        prop_assert!(site.sugar >= 0.0 && site.pollution >= 0.0);
        if let Some(id) = world.occupant(pos) {
            occupied += 1;
            prop_assert_eq!(world.agent(id).map(|a| a.pos), Some(pos));
        }
    }
    prop_assert_eq!(occupied, world.population(), "one agent per site, all indexed");
    for a in world.agents() {
        prop_assert!(a.sugar > 0.0, "living agent {} has sugar {}", a.id, a.sugar);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn invariants_hold_every_tick(config in config_strategy(), seed in any::<u64>()) {
        let mut world = World::new(config.clone(), seed).unwrap();
        check(&world)?;
        for _ in 0..40 {
            world.step();
            check(&world)?;
        }
        if config.replacement.enabled && !config.combat.enabled {
            prop_assert_eq!(world.population(), config.population as usize, "R keeps population constant");
        }
    }

    #[test]
    fn runs_are_deterministic(config in config_strategy(), seed in any::<u64>()) {
        let mut a = World::new(config.clone(), seed).unwrap();
        let mut b = World::new(config, seed).unwrap();
        a.run(25);
        b.run(25);
        prop_assert_eq!(a.fingerprint(), b.fingerprint());
    }
}
```
Note on the "living agent has sugar > 0" invariant: a parent's sugar drops by half its endowment when it has a child but stays ≥ half its endowment, which is > 0 whenever endowments are ≥ 1 (the default ranges guarantee that). The replacement-constant check excludes combat because combat victims are replaced too, so it would also hold; it is excluded only to keep the assertion obviously true — remove the `!combat` guard if it passes with it.

- [ ] **Step 6: Run the invariants**

Run: `cargo test -p sugarscape-core --test invariants --release`
Expected: 2 passed. If a case fails, proptest prints a minimal config and seed; debug it with superpowers:systematic-debugging — do not weaken the invariant.

- [ ] **Step 7: Write book reproduction tests**

`crates/sugarscape-core/tests/book.rs`:
```rust
//! Slow checks that the model reproduces the book's headline results.
//! Run with `cargo test -p sugarscape-core --release --test book -- --ignored`.

use sugarscape_core::config::Config;
use sugarscape_core::presets;
use sugarscape_core::world::World;

fn run(config: Config, seed: u64, ticks: u32) -> World {
    let mut w = World::new(config, seed).unwrap();
    w.run(ticks);
    w
}

#[test]
#[ignore]
fn carrying_capacity_is_near_224() {
    // Chapter II: "although 400 agents begin the simulation, a carrying
    // capacity of approximately 224 is eventually reached."
    let mean = (1..=5).map(|s| run(Config::default(), s, 500).population() as f64).sum::<f64>() / 5.0;
    assert!((190.0..=260.0).contains(&mean), "mean population {mean}");
}

#[test]
#[ignore]
fn replacement_produces_a_skewed_wealth_distribution() {
    let config = presets::by_id("ii-5-wealth").unwrap().config;
    let w = run(config, 1, 500);
    let gini = w.stats.latest().unwrap().gini;
    assert!(gini > 0.45, "gini {gini}");
}

#[test]
#[ignore]
fn sexual_reproduction_sustains_a_population() {
    let config = presets::by_id("iii-2-sex").unwrap().config;
    let w = run(config, 1, 600);
    let pops = w.stats.series("population").unwrap();
    let late = &pops[300..];
    let (min, max) = late.iter().fold((f64::MAX, 0.0f64), |(lo, hi), &p| (lo.min(p), hi.max(p)));
    assert!(min > 100.0, "population collapsed to {min}");
    assert!(max / min < 1.6, "population swings from {min} to {max}");
}
```

- [ ] **Step 8: Run the book tests**

Run: `cargo test -p sugarscape-core --release --test book -- --ignored`
Expected: 3 passed. If one fails, investigate with superpowers:systematic-debugging against the rule text before touching bounds; report the observed values in the task summary either way.

- [ ] **Step 9: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Add book presets, invariant property tests and reproduction checks"
```

---

### Task 15: WASM bindings

**Files:**
- Create: `crates/sugarscape-wasm/Cargo.toml`, `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`
- Modify: `Cargo.toml` (workspace members)

**Interfaces:**
- Consumes: the whole core API.
- Produces (JS names as exported by wasm-bindgen):
  - free functions `presets_json(): string`, `default_config_json(): string`, `series_names_json(): string`
  - `class Sim`:
    - `new Sim(config_json: string, seed: number, capacities?: Uint8Array)` — throws a JSON `FieldError[]` string
    - `step(n: number)`, `tick(): number`, `width(): number`, `height(): number`, `population(): number`
    - `render(color_mode: string, layer: string): number` (byte offset into `memory`), `frame_len(): number`
    - `stats_latest(): string` (JSON `Snapshot`), `series(name: string): Float64Array`, `lorenz(points: number): Float64Array`, `wealth_hist(bins: number): Float64Array` (`[bin_width, ...counts]`)
    - `inspect(x, y): string` (JSON `Inspection`), `locate(id: number): Uint32Array | undefined`
    - `paint_capacity(x, y, radius, value)`, `place_agent(x, y, overrides_json): number`, `remove_agent(x, y)`
    - `set_config(json: string)`, `export_config(): string`, `export_landscape(): Uint8Array`, `landscape_edited(): boolean`
    - `export_series_csv(): string`, `export_agents_csv(): string`
    - `free()` (generated)

- [ ] **Step 1: Install tooling**

```bash
cargo install wasm-pack --locked
wasm-pack --version
```
Expected: prints a version.

- [ ] **Step 2: Create the crate**

Add `"crates/sugarscape-wasm"` to `members` in the root `Cargo.toml`.

`crates/sugarscape-wasm/Cargo.toml`:
```toml
[package]
name = "sugarscape-wasm"
description = "WASM bindings for the Sugarscape playground"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
sugarscape-core = { path = "../sugarscape-core" }
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
serde_json.workspace = true

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

- [ ] **Step 3: Write failing smoke tests**

`crates/sugarscape-wasm/tests/web.rs`:
```rust
//! Run with `wasm-pack test --node crates/sugarscape-wasm`.

use sugarscape_wasm::{presets_json, Sim};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn default_sim_steps_and_renders() {
    let mut sim = Sim::new("{}", 1, None).unwrap();
    assert_eq!((sim.width(), sim.height()), (50, 50));
    sim.step(3);
    assert_eq!(sim.tick(), 3.0);
    let ptr = sim.render("tribe", "sugar").unwrap();
    assert_ne!(ptr, 0);
    assert_eq!(sim.frame_len(), 50 * 50 * 4);
    assert_eq!(sim.series("population").unwrap().len(), 4);
    assert_eq!(sim.wealth_hist(10).len(), 11);
}

#[wasm_bindgen_test]
fn invalid_config_is_a_json_field_error() {
    let err = Sim::new(r#"{"population": 99999}"#, 1, None).err().unwrap();
    let text = err.as_string().unwrap();
    assert!(text.contains(r#""field":"population""#), "{text}");
}

#[wasm_bindgen_test]
fn painted_landscape_round_trips() {
    let mut sim = Sim::new("{}", 1, None).unwrap();
    sim.paint_capacity(10, 10, 2, 4.0).unwrap();
    assert!(sim.landscape_edited());
    let caps = sim.export_landscape();
    let again = Sim::new("{}", 1, Some(caps.clone())).unwrap();
    assert_eq!(again.export_landscape(), caps);
}

#[wasm_bindgen_test]
fn presets_are_listed() {
    assert!(presets_json().contains("ii-2-unit"));
}
```

- [ ] **Step 4: Run to verify failure**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: FAIL to compile — `Sim` missing.

- [ ] **Step 5: Implement**

`crates/sugarscape-wasm/src/lib.rs`:
```rust
//! JavaScript-facing wrapper around the Sugarscape core. Errors cross the
//! boundary as JSON strings of `[{ field, message }]`.

use sugarscape_core::config::{Config, FieldError};
use sugarscape_core::edit::AgentOverrides;
use sugarscape_core::render::{self, ColorMode, Layer};
use sugarscape_core::world::World;
use sugarscape_core::{export, presets, stats};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn field_errors(errors: Vec<FieldError>) -> JsValue {
    JsValue::from_str(&serde_json::to_string(&errors).expect("errors serialize"))
}

fn edit_error(message: String) -> JsValue {
    field_errors(vec![FieldError::new("edit", message)])
}

#[wasm_bindgen]
pub fn presets_json() -> String {
    serde_json::to_string(&presets::all()).expect("presets serialize")
}

#[wasm_bindgen]
pub fn default_config_json() -> String {
    serde_json::to_string(&Config::default()).expect("config serializes")
}

#[wasm_bindgen]
pub fn series_names_json() -> String {
    serde_json::to_string(&stats::SERIES).expect("names serialize")
}

#[wasm_bindgen]
pub struct Sim {
    world: World,
    frame: Vec<u8>,
}

#[wasm_bindgen]
impl Sim {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str, seed: u32, capacities: Option<Vec<u8>>) -> Result<Sim, JsValue> {
        let config = Config::from_json(config_json).map_err(field_errors)?;
        let caps: Option<Vec<f64>> = capacities.map(|c| c.into_iter().map(f64::from).collect());
        let world = World::with_capacities(config, u64::from(seed), caps.as_deref()).map_err(field_errors)?;
        Ok(Sim { world, frame: Vec::new() })
    }

    pub fn step(&mut self, n: u32) {
        self.world.run(n);
    }

    pub fn tick(&self) -> f64 {
        self.world.tick as f64
    }

    pub fn width(&self) -> u32 {
        self.world.torus.width
    }

    pub fn height(&self) -> u32 {
        self.world.torus.height
    }

    pub fn population(&self) -> u32 {
        self.world.population() as u32
    }

    /// Renders into the internal frame and returns a pointer into WASM memory.
    /// Re-create any JS view after each call: memory may have grown.
    pub fn render(&mut self, color_mode: &str, layer: &str) -> Result<usize, JsValue> {
        let mode: ColorMode = color_mode.parse().map_err(edit_error)?;
        let layer: Layer = layer.parse().map_err(edit_error)?;
        render::render(&self.world, mode, layer, &mut self.frame);
        Ok(self.frame.as_ptr() as usize)
    }

    pub fn frame_len(&self) -> usize {
        self.frame.len()
    }

    pub fn stats_latest(&self) -> String {
        serde_json::to_string(&self.world.stats.latest()).expect("snapshot serializes")
    }

    pub fn series(&self, name: &str) -> Result<Vec<f64>, JsValue> {
        self.world
            .stats
            .series(name)
            .ok_or_else(|| edit_error(format!("unknown series {name:?}")))
    }

    pub fn lorenz(&self, points: usize) -> Vec<f64> {
        stats::lorenz(&stats::wealths(&self.world), points.max(2))
    }

    /// `[bin_width, count_0, …, count_{bins-1}]`.
    pub fn wealth_hist(&self, bins: usize) -> Vec<f64> {
        let (width, counts) = stats::histogram(&stats::wealths(&self.world), bins.max(1));
        std::iter::once(width).chain(counts).collect()
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<String, JsValue> {
        let inspection = self.world.inspect(x, y).map_err(edit_error)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    pub fn locate(&self, id: f64) -> Option<Vec<u32>> {
        self.world.locate(id as u64).map(|p| vec![p.x, p.y])
    }

    pub fn paint_capacity(&mut self, x: u32, y: u32, radius: u32, value: f64) -> Result<(), JsValue> {
        self.world.paint_capacity(x, y, radius, value).map_err(edit_error)
    }

    pub fn place_agent(&mut self, x: u32, y: u32, overrides_json: &str) -> Result<f64, JsValue> {
        let overrides: AgentOverrides =
            serde_json::from_str(overrides_json).map_err(|e| edit_error(e.to_string()))?;
        self.world.place_agent(x, y, &overrides).map(|id| id as f64).map_err(edit_error)
    }

    pub fn remove_agent(&mut self, x: u32, y: u32) -> Result<(), JsValue> {
        self.world.remove_agent(x, y).map_err(edit_error)
    }

    pub fn set_config(&mut self, json: &str) -> Result<(), JsValue> {
        let config = Config::from_json(json).map_err(field_errors)?;
        self.world.set_config(config).map_err(field_errors)
    }

    pub fn export_config(&self) -> String {
        serde_json::to_string(&self.world.config).expect("config serializes")
    }

    /// Capacities rounded to bytes, row-major.
    pub fn export_landscape(&self) -> Vec<u8> {
        self.world.capacities().into_iter().map(|c| c.round().clamp(0.0, 255.0) as u8).collect()
    }

    pub fn landscape_edited(&self) -> bool {
        self.world.landscape_edited
    }

    pub fn export_series_csv(&self) -> String {
        export::series_csv(&self.world.stats)
    }

    pub fn export_agents_csv(&self) -> String {
        export::agents_csv(&self.world)
    }
}
```

- [ ] **Step 6: Run the smoke tests and a release build**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: 4 passed.
Run: `wasm-pack build crates/sugarscape-wasm --target web --release --out-dir ../../web/src/wasm-pkg --out-name sugarscape && ls web/src/wasm-pkg`
Expected: `sugarscape.js`, `sugarscape_bg.wasm`, `sugarscape.d.ts` present. Check `sugarscape.d.ts` exports `Sim` with the methods listed above and that `InitOutput` includes `memory: WebAssembly.Memory`.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings
git add Cargo.toml Cargo.lock crates
git commit -m "Add wasm-bindgen Sim wrapper"
```

---
### Task 16: Web scaffold — engine, grid canvas, run controls

**Files:**
- Create: `web/package.json` (via npm), `web/tsconfig.json`, `web/vite.config.ts`, `web/index.html`, `web/src/main.ts`, `web/src/types.ts`, `web/src/engine.ts`, `web/src/style.css`, `web/src/ui/dom.ts`, `web/src/ui/grid-view.ts`, `web/src/ui/toolbar.ts`, `web/src/ui/display.ts`

**Interfaces:**
- Consumes: the `Sim` API and free functions from Task 15 (`web/src/wasm-pkg/sugarscape.js`).
- Produces:
  - `types.ts`: `Config`, `URange`, `LandscapeKind`, `Placement`, `Preset`, `FieldError`, `Snapshot`, `Inspection`, `AgentView`, `SiteView`, `LinkView`, `ColorMode`, `Layer`, `parseErrors(e: unknown): FieldError[]`
  - `engine.ts`: `class Engine` with `presets`, `sim`, `config`, `seed`, `presetId`, `running`, `stepsPerFrame`, `colorMode`, `layer`, `selection`; `static create(initial?)`, `on(event, fn): () => void`, `size()`, `reset(config?, seed?, landscape?)`, `applyConfig(config)`, `loadPreset(id)`, `isModified()`, `setRunning(on)`, `advance(n?)`, `frame()`, `setDisplay({colorMode?, layer?})`, `select(x, y)`, `follow(id)`, `trackSelection()`, `paint(x, y, radius, value)`, `place(x, y, overrides)`, `erase(x, y)`; `EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit'`; `randomSeed()`
  - `ui/dom.ts`: `h(tag, props?, ...children)`
  - `ui/grid-view.ts`: `class GridView` with `draw()`, `onCell`, `brushRadius`, `toPngBlob(scale?)`
  - `ui/toolbar.ts`: `buildToolbar(engine): HTMLElement` (Task 20 appends share/export controls to the returned element's `.toolbar-end` slot)
  - `ui/display.ts`: `buildDisplay(engine): HTMLElement`

- [ ] **Step 1: Scaffold the package**

```bash
mkdir -p web/src/ui
cd web
npm init -y
npm install uplot
npm install -D typescript vite vitest
npm pkg set name=sugarscape-web private=true type=module
npm pkg delete main
npm pkg set scripts.wasm="wasm-pack build ../crates/sugarscape-wasm --target web --release --out-dir ../../web/src/wasm-pkg --out-name sugarscape"
npm pkg set scripts.wasm:dev="wasm-pack build ../crates/sugarscape-wasm --target web --dev --out-dir ../../web/src/wasm-pkg --out-name sugarscape"
npm pkg set scripts.dev="npm run wasm:dev && vite"
npm pkg set scripts.build="npm run wasm && tsc --noEmit && vite build"
npm pkg set scripts.typecheck="tsc --noEmit"
npm pkg set scripts.test="vitest run"
cd ..
```

`web/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "types": ["vite/client"]
  },
  "include": ["src", "vite.config.ts"]
}
```

`web/vite.config.ts`:
```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  // Relative asset URLs so the build works under a GitHub Pages sub-path.
  base: './',
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
```

`web/index.html`:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SugarScape Playground</title>
  </head>
  <body>
    <div id="banner" class="banner" hidden></div>
    <header id="toolbar"></header>
    <main class="layout">
      <section class="stage">
        <div id="display" class="display"></div>
        <canvas id="grid" aria-label="Sugarscape grid"></canvas>
        <div id="tools"></div>
      </section>
      <aside class="panel">
        <nav id="tabs" class="tabs" role="tablist"></nav>
        <div id="panel-body" class="panel-body"></div>
      </aside>
    </main>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 2: Types and DOM helper**

`web/src/types.ts`:
```ts
// Mirrors of the JSON shapes produced by sugarscape-core (serde).

export interface URange { min: number; max: number }

export type LandscapeKind = { kind: 'two_peaks' } | { kind: 'flat'; capacity: number };

export type Placement =
  | { kind: 'random' }
  | { kind: 'block'; x: number; y: number; width: number; height: number }
  | { kind: 'tribes'; size: number };

export interface Config {
  width: number;
  height: number;
  landscape: LandscapeKind;
  population: number;
  placement: Placement;
  vision: URange;
  metabolism: URange;
  endowment: URange;
  tag_length: number;
  growback: { rate: number; instant: boolean };
  seasons: { enabled: boolean; winter_divisor: number; period: number };
  pollution: { enabled: boolean; production: number; consumption: number };
  diffusion: { enabled: boolean; every: number };
  lifespan: { enabled: boolean; max_age: URange };
  replacement: { enabled: boolean };
  sex: { enabled: boolean; fertility_onset: URange; female_end: URange; male_end: URange };
  inheritance: { enabled: boolean };
  culture: { enabled: boolean };
  combat: { enabled: boolean; unlimited: boolean; reward: number };
}

export interface Preset { id: string; name: string; source: string; description: string; config: Config }

export interface FieldError { field: string; message: string }

export interface Snapshot {
  tick: number;
  population: number;
  gini: number;
  mean_wealth: number;
  mean_vision: number;
  mean_metabolism: number;
  blue_fraction: number;
  births: number;
  deaths: number;
}

export interface SiteView { x: number; y: number; sugar: number; capacity: number; pollution: number }
export interface LinkView { id: number; alive: boolean }
export interface AgentView {
  id: number;
  x: number;
  y: number;
  sex: 'female' | 'male';
  tribe: 'blue' | 'red';
  tags: string;
  vision: number;
  metabolism: number;
  sugar: number;
  initial_sugar: number;
  age: number;
  max_age: number;
  fertile: boolean;
  fertility_onset: number;
  fertility_end: number;
  born: number;
  parents: LinkView[];
  children: LinkView[];
}
export interface Inspection { site: SiteView; agent: AgentView | null }

export type ColorMode = 'tribe' | 'wealth' | 'sex' | 'age' | 'vision';
export type Layer = 'sugar' | 'capacity' | 'pollution';

/** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
export function parseErrors(e: unknown): FieldError[] {
  const text = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
  try {
    const parsed: unknown = JSON.parse(text);
    if (Array.isArray(parsed)) return parsed as FieldError[];
  } catch {
    // not JSON
  }
  return [{ field: 'config', message: text }];
}
```

`web/src/ui/dom.ts`:
```ts
type Child = Node | string | number | null | undefined | false;

/** Creates an element. `on*` function props become listeners; `class` sets className. */
export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Record<string, unknown> = {},
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (value === undefined || value === null || value === false) continue;
    if (key.startsWith('on') && typeof value === 'function') {
      el.addEventListener(key.slice(2).toLowerCase(), value as EventListener);
    } else if (key === 'class') {
      el.className = String(value);
    } else if (key in el) {
      (el as unknown as Record<string, unknown>)[key] = value;
    } else {
      el.setAttribute(key, value === true ? '' : String(value));
    }
  }
  for (const child of children) {
    if (child === null || child === undefined || child === false) continue;
    el.append(typeof child === 'number' ? String(child) : child);
  }
  return el;
}
```

- [ ] **Step 3: Engine**

`web/src/engine.ts`:
```ts
import init, { Sim, presets_json } from './wasm-pkg/sugarscape.js';
import type { ColorMode, Config, FieldError, Inspection, Layer, Preset } from './types';
import { parseErrors } from './types';

export type EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit';

export interface Selection { x: number; y: number; agentId: number | null }

export interface PlaceOverrides { sex?: 'female' | 'male'; tribe?: 'blue' | 'red' }

export interface InitialState { config: Config; seed: number; landscape?: Uint8Array }

export function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** Owns the WASM simulation and the playground's run/display/selection state. */
export class Engine {
  running = false;
  stepsPerFrame = 1;
  colorMode: ColorMode = 'tribe';
  layer: Layer = 'sugar';
  selection: Selection | null = null;
  presetId: string | null;
  private listeners = new Map<EngineEvent, Set<() => void>>();

  private constructor(
    private memory: WebAssembly.Memory,
    readonly presets: Preset[],
    public sim: Sim,
    public config: Config,
    public seed: number,
  ) {
    this.presetId = this.matchPreset();
  }

  /** Throws parsed FieldError[] (as an Error message) if `initial` is invalid. */
  static async create(initial?: InitialState): Promise<Engine> {
    const wasm = await init();
    const presets = JSON.parse(presets_json()) as Preset[];
    const fallback = presets.find((p) => p.id === 'ii-2-unit') ?? presets[0];
    const config = initial?.config ?? structuredClone(fallback.config);
    const seed = initial?.seed ?? randomSeed();
    let sim: Sim;
    try {
      sim = new Sim(JSON.stringify(config), seed, initial?.landscape);
    } catch (e) {
      throw new Error(parseErrors(e).map((x) => `${x.field}: ${x.message}`).join('; '));
    }
    return new Engine(wasm.memory, presets, sim, config, seed);
  }

  on(event: EngineEvent, fn: () => void): () => void {
    let set = this.listeners.get(event);
    if (!set) this.listeners.set(event, (set = new Set()));
    set.add(fn);
    return () => set.delete(fn);
  }

  private emit(event: EngineEvent): void {
    this.listeners.get(event)?.forEach((fn) => fn());
  }

  size(): { width: number; height: number } {
    return { width: this.sim.width(), height: this.sim.height() };
  }

  /** Rebuilds the world. On error the current world is kept and errors returned. */
  reset(config: Config = this.config, seed: number = this.seed, landscape?: Uint8Array): FieldError[] | null {
    let next: Sim;
    try {
      next = new Sim(JSON.stringify(config), seed, landscape);
    } catch (e) {
      return parseErrors(e);
    }
    this.sim.free();
    this.sim = next;
    this.config = config;
    this.seed = seed;
    this.selection = null;
    this.presetId = this.matchPreset();
    this.emit('reset');
    return null;
  }

  /** Applies rule/parameter changes to the running world. */
  applyConfig(config: Config): FieldError[] | null {
    try {
      this.sim.set_config(JSON.stringify(config));
    } catch (e) {
      return parseErrors(e);
    }
    this.config = config;
    this.emit('config');
    return null;
  }

  loadPreset(id: string): FieldError[] | null {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    const errors = this.reset(structuredClone(preset.config));
    if (!errors) this.presetId = id;
    return errors;
  }

  /** The preset whose config equals the current one, if any. */
  private matchPreset(): string | null {
    const json = JSON.stringify(this.config);
    return this.presets.find((p) => JSON.stringify(p.config) === json)?.id ?? null;
  }

  /** True when the config differs from the last chosen preset. */
  isModified(): boolean {
    const preset = this.presets.find((p) => p.id === this.presetId);
    return !preset || JSON.stringify(preset.config) !== JSON.stringify(this.config);
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
  }

  advance(n: number = this.stepsPerFrame): void {
    this.sim.step(n);
    this.emit('tick');
  }

  /** A fresh view over the rendered RGBA frame (views die when memory grows). */
  frame(): Uint8ClampedArray<ArrayBuffer> {
    const ptr = this.sim.render(this.colorMode, this.layer);
    return new Uint8ClampedArray(this.memory.buffer, ptr, this.sim.frame_len());
  }

  setDisplay(d: { colorMode?: ColorMode; layer?: Layer }): void {
    if (d.colorMode) this.colorMode = d.colorMode;
    if (d.layer) this.layer = d.layer;
    this.emit('display');
  }

  inspect(x: number, y: number): Inspection {
    return JSON.parse(this.sim.inspect(x, y)) as Inspection;
  }

  select(x: number, y: number): void {
    this.selection = { x, y, agentId: this.inspect(x, y).agent?.id ?? null };
    this.emit('select');
  }

  follow(id: number): void {
    const p = this.sim.locate(id);
    if (p) this.select(p[0], p[1]);
  }

  /** Keeps the selection on a followed agent as it moves. */
  trackSelection(): void {
    const s = this.selection;
    if (s?.agentId == null) return;
    const p = this.sim.locate(s.agentId);
    if (p) {
      s.x = p[0];
      s.y = p[1];
    }
  }

  private edit(fn: () => void): FieldError[] | null {
    try {
      fn();
    } catch (e) {
      return parseErrors(e);
    }
    this.emit('edit');
    return null;
  }

  paint(x: number, y: number, radius: number, value: number): FieldError[] | null {
    return this.edit(() => this.sim.paint_capacity(x, y, radius, value));
  }

  place(x: number, y: number, overrides: PlaceOverrides): FieldError[] | null {
    return this.edit(() => void this.sim.place_agent(x, y, JSON.stringify(overrides)));
  }

  erase(x: number, y: number): FieldError[] | null {
    return this.edit(() => this.sim.remove_agent(x, y));
  }
}
```
If `tsc` reports that `Uint8ClampedArray` is not generic (TypeScript < 5.7), drop the `<ArrayBuffer>` type argument.

- [ ] **Step 4: Grid view**

`web/src/ui/grid-view.ts`:
```ts
import type { Engine } from '../engine';

const CELL = 12;

export type CellEvent = 'down' | 'drag';

/** Draws the Rust-rendered frame scaled up with crisp pixels, plus overlays. */
export class GridView {
  onCell: ((x: number, y: number, kind: CellEvent) => void) | null = null;
  /** When set, a brush outline of this radius follows the pointer. */
  brushRadius: number | null = null;
  private hover: { x: number; y: number } | null = null;
  private ctx: CanvasRenderingContext2D;
  private buffer = document.createElement('canvas');
  private bctx: CanvasRenderingContext2D;

  constructor(readonly canvas: HTMLCanvasElement, private engine: Engine) {
    this.ctx = canvas.getContext('2d')!;
    this.bctx = this.buffer.getContext('2d')!;
    canvas.addEventListener('pointerdown', (e) => {
      canvas.setPointerCapture(e.pointerId);
      const c = this.cellAt(e);
      this.hover = c;
      this.onCell?.(c.x, c.y, 'down');
    });
    canvas.addEventListener('pointermove', (e) => {
      const c = this.cellAt(e);
      const moved = !this.hover || c.x !== this.hover.x || c.y !== this.hover.y;
      this.hover = c;
      if (moved && e.buttons & 1) this.onCell?.(c.x, c.y, 'drag');
      if (moved) this.draw();
    });
    canvas.addEventListener('pointerleave', () => {
      this.hover = null;
      this.draw();
    });
  }

  private cellAt(e: PointerEvent): { x: number; y: number } {
    const r = this.canvas.getBoundingClientRect();
    const { width, height } = this.engine.size();
    const clamp = (v: number, n: number) => Math.min(n - 1, Math.max(0, Math.floor(v)));
    return {
      x: clamp(((e.clientX - r.left) / r.width) * width, width),
      y: clamp(((e.clientY - r.top) / r.height) * height, height),
    };
  }

  private blit(): { width: number; height: number } {
    const { width, height } = this.engine.size();
    if (this.buffer.width !== width || this.buffer.height !== height) {
      this.buffer.width = width;
      this.buffer.height = height;
    }
    this.bctx.putImageData(new ImageData(this.engine.frame(), width, height), 0, 0);
    return { width, height };
  }

  draw(): void {
    const { width, height } = this.blit();
    if (this.canvas.width !== width * CELL || this.canvas.height !== height * CELL) {
      this.canvas.width = width * CELL;
      this.canvas.height = height * CELL;
    }
    const ctx = this.ctx;
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(this.buffer, 0, 0, width * CELL, height * CELL);

    const accent = getComputedStyle(this.canvas).getPropertyValue('--accent').trim() || '#fff';
    const sel = this.engine.selection;
    if (sel) {
      ctx.strokeStyle = '#000';
      ctx.lineWidth = 4;
      ctx.strokeRect(sel.x * CELL - 2, sel.y * CELL - 2, CELL + 4, CELL + 4);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 2;
      ctx.strokeRect(sel.x * CELL - 2, sel.y * CELL - 2, CELL + 4, CELL + 4);
    }
    if (this.hover && this.brushRadius !== null) {
      ctx.save();
      ctx.setLineDash([4, 3]);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.arc((this.hover.x + 0.5) * CELL, (this.hover.y + 0.5) * CELL, (this.brushRadius + 0.5) * CELL, 0, Math.PI * 2);
      ctx.stroke();
      ctx.restore();
    }
  }

  /** The grid without overlays, scaled up, as a PNG. */
  toPngBlob(scale = 10): Promise<Blob> {
    const { width, height } = this.blit();
    const out = document.createElement('canvas');
    out.width = width * scale;
    out.height = height * scale;
    const ctx = out.getContext('2d')!;
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(this.buffer, 0, 0, out.width, out.height);
    return new Promise((resolve, reject) =>
      out.toBlob((b) => (b ? resolve(b) : reject(new Error('PNG encoding failed'))), 'image/png'),
    );
  }
}
```

- [ ] **Step 5: Toolbar and display controls**

`web/src/ui/toolbar.ts`:
```ts
import { randomSeed, type Engine } from '../engine';
import { h } from './dom';

const SPEEDS = [1, 2, 5, 10, 25, 100];

export function buildToolbar(engine: Engine): HTMLElement {
  const play = h('button', { class: 'primary', onclick: () => engine.setRunning(!engine.running) });
  const step = h('button', { onclick: () => engine.advance(1), title: 'Advance one tick' }, 'Step');
  const speed = h(
    'select',
    { title: 'Ticks per frame', onchange: () => (engine.stepsPerFrame = Number(speed.value)) },
    ...SPEEDS.map((s) => h('option', { value: String(s) }, `${s}×`)),
  );
  const seed = h('input', { type: 'number', min: 0, max: 4294967295, class: 'seed', title: 'Seed' });
  const reset = h('button', { onclick: () => engine.reset(engine.config, Number(seed.value) >>> 0) }, 'Reset');
  const dice = h(
    'button',
    { title: 'Random seed and reset', onclick: () => engine.reset(engine.config, randomSeed()) },
    '🎲',
  );
  const readout = h('span', { class: 'readout' });

  const sync = () => {
    play.textContent = engine.running ? 'Pause' : 'Play';
    step.disabled = engine.running;
    seed.value = String(engine.seed);
  };
  const tick = () => {
    readout.textContent = `t = ${engine.sim.tick()} · ${engine.sim.population()} agents`;
  };
  engine.on('run', sync);
  engine.on('reset', () => {
    sync();
    tick();
  });
  engine.on('tick', tick);
  engine.on('edit', tick);
  sync();
  tick();

  return h(
    'div',
    { class: 'toolbar' },
    h('h1', {}, 'SugarScape'),
    h('div', { class: 'group' }, play, step, speed),
    h('div', { class: 'group' }, h('label', {}, 'Seed ', seed), reset, dice),
    readout,
    h('div', { class: 'toolbar-end' }),
  );
}
```

`web/src/ui/display.ts`:
```ts
import type { Engine } from '../engine';
import type { ColorMode, Layer } from '../types';
import { h } from './dom';

const MODES: [ColorMode, string][] = [
  ['tribe', 'Tribe'],
  ['wealth', 'Wealth'],
  ['sex', 'Sex'],
  ['age', 'Age'],
  ['vision', 'Vision'],
];
const LAYERS: [Layer, string][] = [
  ['sugar', 'Sugar'],
  ['capacity', 'Capacity'],
  ['pollution', 'Pollution'],
];

export function buildDisplay(engine: Engine): HTMLElement {
  const mode = h(
    'select',
    { onchange: () => engine.setDisplay({ colorMode: mode.value as ColorMode }) },
    ...MODES.map(([v, l]) => h('option', { value: v }, l)),
  );
  const layer = h(
    'select',
    { onchange: () => engine.setDisplay({ layer: layer.value as Layer }) },
    ...LAYERS.map(([v, l]) => h('option', { value: v }, l)),
  );
  const sync = () => {
    mode.value = engine.colorMode;
    layer.value = engine.layer;
  };
  engine.on('display', sync);
  sync();
  return h('div', { class: 'display-controls' }, h('label', {}, 'Agents ', mode), h('label', {}, 'Landscape ', layer));
}
```

- [ ] **Step 6: Styles**

`web/src/style.css`:
```css
:root {
  color-scheme: light dark;
  --bg: #f6f4ee;
  --surface: #ffffff;
  --text: #1d1b16;
  --muted: #6b665a;
  --border: #ddd8cc;
  --grid: #e9e5da;
  --accent: #c7461a;
  --error: #b3261e;
  --blue: #3d7eff;
  --red: #ff4d4d;
  --c1: #2f6fdb;
  --c2: #c7461a;
  --c3: #2f9e6e;
  --c4: #8e5bd6;
  font-family: ui-sans-serif, system-ui, -apple-system, 'Segoe UI', sans-serif;
  font-size: 14px;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme='light']) {
    --bg: #12110f;
    --surface: #1c1b18;
    --text: #ece8dd;
    --muted: #a39d8e;
    --border: #34312b;
    --grid: #2a2823;
    --accent: #f2c14e;
    --error: #ff8a80;
    --c1: #6ea8ff;
    --c2: #ff8a5c;
    --c3: #5fd3a1;
    --c4: #b894f0;
  }
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--text); }
button, select, input { font: inherit; color: inherit; background: var(--surface); border: 1px solid var(--border); border-radius: 6px; padding: 4px 10px; }
button { cursor: pointer; }
button:disabled { opacity: 0.5; cursor: default; }
button.primary { background: var(--accent); color: var(--bg); border-color: var(--accent); min-width: 5em; }
button[aria-pressed='true'] { border-color: var(--accent); box-shadow: inset 0 0 0 1px var(--accent); }
.toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 16px; padding: 10px 16px; border-bottom: 1px solid var(--border); }
.toolbar h1 { font-size: 16px; margin: 0; letter-spacing: 0.02em; }
.toolbar .group { display: flex; gap: 6px; align-items: center; }
.toolbar .seed { width: 9em; }
.toolbar .readout { color: var(--muted); font-variant-numeric: tabular-nums; }
.toolbar-end { margin-left: auto; display: flex; gap: 6px; align-items: center; }
.layout { display: grid; grid-template-columns: minmax(0, 1fr) 400px; gap: 16px; padding: 16px; }
.stage { display: flex; flex-direction: column; gap: 10px; min-width: 0; }
.display-controls { display: flex; gap: 12px; flex-wrap: wrap; }
#grid { width: 100%; max-width: min(100%, 82vh); aspect-ratio: 1; image-rendering: pixelated; border-radius: 6px; touch-action: none; cursor: crosshair; align-self: center; }
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; min-width: 0; display: flex; flex-direction: column; max-height: calc(100vh - 90px); }
.tabs { display: flex; border-bottom: 1px solid var(--border); }
.tabs button { flex: 1; border: 0; border-radius: 0; background: none; padding: 10px; color: var(--muted); }
.tabs button[aria-selected='true'] { color: var(--text); box-shadow: inset 0 -2px 0 var(--accent); }
.panel-body { overflow: auto; padding: 12px 14px; }
.banner { background: var(--error); color: #fff; padding: 8px 16px; display: flex; gap: 12px; align-items: center; }
.banner button { background: transparent; color: #fff; border-color: #fff; }
.hint { color: var(--muted); }
.error { color: var(--error); font-size: 12px; margin: 2px 0 0; }
@media (max-width: 860px) {
  .layout { grid-template-columns: 1fr; padding: 16px; }
  .panel { max-height: none; }
}
```

- [ ] **Step 7: Bootstrap**

`web/src/main.ts`:
```ts
import './style.css';
import { Engine } from './engine';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { GridView } from './ui/grid-view';
import { buildToolbar } from './ui/toolbar';

export function showBanner(message: string, action?: { label: string; run: () => void }): void {
  const banner = document.querySelector<HTMLElement>('#banner')!;
  banner.replaceChildren(
    h('span', {}, message),
    action ? h('button', { onclick: action.run }, action.label) : null,
    h('button', { onclick: () => (banner.hidden = true), 'aria-label': 'Dismiss' }, '×'),
  );
  banner.hidden = false;
}

async function main(): Promise<void> {
  const engine = await Engine.create();
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  document.querySelector('#toolbar')!.append(buildToolbar(engine));
  document.querySelector('#display')!.append(buildDisplay(engine));

  let dirty = true;
  for (const event of ['reset', 'tick', 'config', 'display', 'select', 'edit'] as const) {
    engine.on(event, () => (dirty = true));
  }
  const loop = () => {
    try {
      if (engine.running) engine.advance();
      if (dirty) {
        engine.trackSelection();
        grid.draw();
        dirty = false;
      }
    } catch (e) {
      // A Rust panic leaves the WASM instance unusable; reloading keeps any #s= share state.
      engine.running = false;
      console.error(e);
      showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}

main().catch((e) => showBanner(`Failed to start: ${e instanceof Error ? e.message : String(e)}`));
```

- [ ] **Step 8: Build and check it runs**

Run: `cd web && npm run build`
Expected: wasm builds, `tsc --noEmit` passes, `vite build` writes `web/dist`.
Run: `cd web && npm run dev` (background), open the printed URL (use the `run` skill if available, otherwise report the URL to the user). Check: grid shows two yellow sugar mountains with blue/red agents; Play animates; Step advances `t`; changing Seed + Reset changes placement; Agents/Landscape selects recolor. Stop the dev server afterwards.

- [ ] **Step 9: Commit**

```bash
git add web .gitignore
git commit -m "Add web front end: engine, grid canvas and run controls"
```

---

### Task 17: Rules panel — presets, toggles, parameters, errors

**Files:**
- Create: `web/src/paths.ts`, `web/src/paths.test.ts`, `web/src/schema.ts`, `web/src/ui/tabs.ts`, `web/src/ui/rules-panel.ts`
- Modify: `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `Engine.{presets, config, presetId, reset, applyConfig, loadPreset, isModified, on}`.
- Produces:
  - `paths.ts`: `getPath(obj, path): unknown`, `setPath(obj, path, value)` (throws on unknown path), `errorsFor(errors, path): FieldError[]`
  - `schema.ts`: `Control`, `Group`, `GROUPS: Group[]`
  - `ui/tabs.ts`: `class Tabs { constructor(nav, body); add(label, el, onShow?: (visible: boolean) => void); show(label) }`
  - `ui/rules-panel.ts`: `class RulesPanel { el }`

- [ ] **Step 1: Write failing path tests**

`web/src/paths.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { errorsFor, getPath, setPath } from './paths';

describe('paths', () => {
  const obj = () => ({ a: 1, sex: { enabled: false, fertility_onset: { min: 12, max: 15 } } });

  it('reads nested values', () => {
    expect(getPath(obj(), 'sex.fertility_onset.max')).toBe(15);
    expect(getPath(obj(), 'nope.x')).toBeUndefined();
  });

  it('writes nested values in place', () => {
    const o = obj();
    setPath(o, 'sex.enabled', true);
    setPath(o, 'sex.fertility_onset', { min: 1, max: 2 });
    expect(o.sex.enabled).toBe(true);
    expect(o.sex.fertility_onset).toEqual({ min: 1, max: 2 });
  });

  it('rejects unknown paths', () => {
    expect(() => setPath(obj(), 'sex.bogus', 1)).toThrow(/unknown field/);
    expect(() => setPath(obj(), 'a.b', 1)).toThrow();
  });

  it('matches errors to a control and its sub-fields', () => {
    const errors = [
      { field: 'vision.max', message: 'too big' },
      { field: 'vision', message: 'min > max' },
      { field: 'visionary', message: 'unrelated' },
    ];
    expect(errorsFor(errors, 'vision').map((e) => e.message)).toEqual(['too big', 'min > max']);
  });
});
```

- [ ] **Step 2: Run to verify failure**

Run: `cd web && npx vitest run src/paths.test.ts`
Expected: FAIL — cannot resolve `./paths`.

- [ ] **Step 3: Implement paths**

`web/src/paths.ts`:
```ts
import type { FieldError } from './types';

type Obj = Record<string, unknown>;

export function getPath(obj: unknown, path: string): unknown {
  return path.split('.').reduce<unknown>((o, k) => (o && typeof o === 'object' ? (o as Obj)[k] : undefined), obj);
}

/** Sets an existing field (so key order, and preset comparison, is preserved). */
export function setPath<T extends object>(obj: T, path: string, value: unknown): T {
  const keys = path.split('.');
  const last = keys.pop()!;
  let o = obj as Obj;
  for (const k of keys) {
    const next = o[k];
    if (!next || typeof next !== 'object') throw new Error(`no object at "${k}" in ${path}`);
    o = next as Obj;
  }
  if (!(last in o)) throw new Error(`unknown field ${path}`);
  o[last] = value;
  return obj;
}

/** Errors for a control at `path`, including errors on its sub-fields. */
export function errorsFor(errors: FieldError[], path: string): FieldError[] {
  return errors.filter((e) => e.field === path || e.field.startsWith(`${path}.`));
}
```

- [ ] **Step 4: Run tests**

Run: `cd web && npx vitest run src/paths.test.ts`
Expected: 4 passed.

- [ ] **Step 5: Schema**

`web/src/schema.ts`:
```ts
import type { Config } from './types';

interface Base { path: string; label: string; reset?: boolean; hint?: string }
export type Control =
  | (Base & { kind: 'toggle' })
  | (Base & { kind: 'number'; min: number; max: number; step: number })
  | (Base & { kind: 'range'; min: number; max: number })
  | (Base & { kind: 'select'; options: { value: string; label: string; apply: (c: Config) => void }[]; current: (c: Config) => string });

export interface Group {
  title: string;
  /** Path of the boolean that switches this rule on (shown in the header). */
  enable?: string;
  note?: string;
  controls: Control[];
}

export const GROUPS: Group[] = [
  {
    title: 'Setup',
    note: 'Changing these rebuilds the world.',
    controls: [
      {
        kind: 'select', path: 'landscape', label: 'Landscape', reset: true,
        current: (c) => c.landscape.kind,
        options: [
          { value: 'two_peaks', label: 'Two sugar mountains (50×50)', apply: (c) => { c.landscape = { kind: 'two_peaks' }; c.width = 50; c.height = 50; } },
          { value: 'flat', label: 'Flat', apply: (c) => { c.landscape = { kind: 'flat', capacity: 2 }; } },
        ],
      },
      { kind: 'number', path: 'width', label: 'Width', min: 10, max: 200, step: 1, reset: true },
      { kind: 'number', path: 'height', label: 'Height', min: 10, max: 200, step: 1, reset: true },
      { kind: 'number', path: 'population', label: 'Initial agents', min: 0, max: 4000, step: 10, reset: true },
      {
        kind: 'select', path: 'placement', label: 'Placement', reset: true,
        current: (c) => c.placement.kind,
        options: [
          { value: 'random', label: 'Random', apply: (c) => { c.placement = { kind: 'random' }; } },
          { value: 'block', label: 'Southwest block', apply: (c) => { c.placement = { kind: 'block', x: 0, y: Math.floor(c.height / 2), width: Math.floor(c.width / 2), height: Math.ceil(c.height / 2) }; } },
          { value: 'tribes', label: 'Two tribes in corners', apply: (c) => { c.placement = { kind: 'tribes', size: Math.floor(Math.min(c.width, c.height) * 0.4) }; } },
        ],
      },
      { kind: 'number', path: 'tag_length', label: 'Tag length', min: 1, max: 64, step: 1, reset: true },
    ],
  },
  {
    title: 'New agents',
    note: 'Initial and replacement agents draw traits uniformly from these ranges.',
    controls: [
      { kind: 'range', path: 'vision', label: 'Vision', min: 1, max: 25 },
      { kind: 'range', path: 'metabolism', label: 'Metabolism', min: 0, max: 10 },
      { kind: 'range', path: 'endowment', label: 'Initial sugar', min: 0, max: 500 },
    ],
  },
  {
    title: 'Growback (G)',
    controls: [
      { kind: 'number', path: 'growback.rate', label: 'Rate α', min: 0.1, max: 10, step: 0.1 },
      { kind: 'toggle', path: 'growback.instant', label: 'Instant (G∞)' },
    ],
  },
  {
    title: 'Seasons', enable: 'seasons.enabled',
    controls: [
      { kind: 'number', path: 'seasons.period', label: 'Season length γ', min: 1, max: 500, step: 1 },
      { kind: 'number', path: 'seasons.winter_divisor', label: 'Winter slowdown β', min: 1, max: 20, step: 1 },
    ],
  },
  {
    title: 'Pollution (P)', enable: 'pollution.enabled',
    controls: [
      { kind: 'number', path: 'pollution.production', label: 'Per sugar gathered α', min: 0, max: 5, step: 0.1 },
      { kind: 'number', path: 'pollution.consumption', label: 'Per sugar eaten β', min: 0, max: 5, step: 0.1 },
    ],
  },
  {
    title: 'Diffusion (D)', enable: 'diffusion.enabled',
    controls: [{ kind: 'number', path: 'diffusion.every', label: 'Every α ticks', min: 1, max: 50, step: 1 }],
  },
  {
    title: 'Lifespan', enable: 'lifespan.enabled',
    controls: [{ kind: 'range', path: 'lifespan.max_age', label: 'Max age [a, b]', min: 1, max: 300 }],
  },
  { title: 'Replacement (R)', enable: 'replacement.enabled', note: 'Needs lifespan; excludes sex.', controls: [] },
  {
    title: 'Sex (S)', enable: 'sex.enabled',
    controls: [
      { kind: 'range', path: 'sex.fertility_onset', label: 'Fertility begins', min: 0, max: 100 },
      { kind: 'range', path: 'sex.female_end', label: 'Female fertility ends', min: 0, max: 150 },
      { kind: 'range', path: 'sex.male_end', label: 'Male fertility ends', min: 0, max: 150 },
    ],
  },
  { title: 'Inheritance (I)', enable: 'inheritance.enabled', controls: [] },
  { title: 'Culture (K)', enable: 'culture.enabled', controls: [] },
  {
    title: 'Combat (C)', enable: 'combat.enabled',
    controls: [
      { kind: 'toggle', path: 'combat.unlimited', label: 'Unlimited reward (C∞)' },
      { kind: 'number', path: 'combat.reward', label: 'Reward cap α', min: 0, max: 50, step: 0.5 },
    ],
  },
];
```
(`npx prettier` is not part of the toolchain; keep the formatting readable by hand.)

- [ ] **Step 6: Tabs and the rules panel**

`web/src/ui/tabs.ts`:
```ts
import { h } from './dom';

export class Tabs {
  private entries: { label: string; button: HTMLButtonElement; el: HTMLElement; onShow?: (v: boolean) => void }[] = [];

  constructor(private nav: HTMLElement, private body: HTMLElement) {}

  add(label: string, el: HTMLElement, onShow?: (visible: boolean) => void): void {
    const button = h('button', { role: 'tab', onclick: () => this.show(label) }, label);
    this.nav.append(button);
    this.body.append(el);
    this.entries.push({ label, button, el, onShow });
    if (this.entries.length === 1) this.show(label);
    else el.hidden = true;
  }

  show(label: string): void {
    for (const e of this.entries) {
      const on = e.label === label;
      e.button.setAttribute('aria-selected', String(on));
      e.el.hidden = !on;
      e.onShow?.(on);
    }
  }
}
```

`web/src/ui/rules-panel.ts`:
```ts
import type { Engine } from '../engine';
import { errorsFor, getPath, setPath } from '../paths';
import { GROUPS, type Control } from '../schema';
import type { Config, FieldError, URange } from '../types';
import { h } from './dom';

/** Preset picker plus one section per rule, generated from GROUPS. */
export class RulesPanel {
  readonly el = h('div', { class: 'rules' });
  private errors: FieldError[] = [];
  private syncers: (() => void)[] = [];
  private errorSlots: { path: string; el: HTMLElement }[] = [];
  private general = h('div', { class: 'error' });

  constructor(private engine: Engine) {
    this.el.append(this.presetSection(), this.general, ...GROUPS.map((g) => this.groupSection(g)));
    engine.on('reset', () => this.sync());
    engine.on('config', () => this.sync());
    this.sync();
  }

  private commit(mutate: (c: Config) => void, reset: boolean): void {
    const next = structuredClone(this.engine.config);
    mutate(next);
    this.errors = (reset ? this.engine.reset(next) : this.engine.applyConfig(next)) ?? [];
    this.renderErrors();
  }

  private sync(): void {
    this.syncers.forEach((s) => s());
  }

  private renderErrors(): void {
    const claimed = new Set<FieldError>();
    for (const slot of this.errorSlots) {
      const mine = errorsFor(this.errors, slot.path);
      mine.forEach((e) => claimed.add(e));
      slot.el.replaceChildren(...mine.map((e) => h('p', {}, e.message)));
    }
    const rest = this.errors.filter((e) => !claimed.has(e));
    this.general.replaceChildren(...rest.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }

  private errorSlot(path: string): HTMLElement {
    const el = h('div', { class: 'error' });
    this.errorSlots.push({ path, el });
    return el;
  }

  private presetSection(): HTMLElement {
    const select = h(
      'select',
      {
        onchange: () => {
          this.errors = this.engine.loadPreset(select.value) ?? [];
          this.renderErrors();
        },
      },
      ...this.engine.presets.map((p) => h('option', { value: p.id }, `${p.name} — ${p.source}`)),
    );
    const badge = h('span', { class: 'badge' }, 'modified');
    const desc = h('p', { class: 'hint' });
    this.syncers.push(() => {
      const p = this.engine.presets.find((x) => x.id === this.engine.presetId);
      select.value = p?.id ?? '';
      badge.hidden = !this.engine.isModified();
      desc.textContent = p ? p.description : 'Custom configuration.';
    });
    return h('section', { class: 'presets' }, h('label', {}, 'Rule system ', badge), select, desc);
  }

  private groupSection(group: (typeof GROUPS)[number]): HTMLElement {
    const header = h('h3', {}, group.title);
    if (group.enable) {
      const path = group.enable;
      const box = h('input', {
        type: 'checkbox',
        onchange: () => this.commit((c) => setPath(c, path, box.checked), false),
      });
      this.syncers.push(() => (box.checked = getPath(this.engine.config, path) === true));
      header.replaceChildren(h('label', { class: 'switch' }, box, ` ${group.title}`));
    }
    return h(
      'section',
      { class: 'group' },
      header,
      group.enable ? this.errorSlot(group.enable) : null,
      group.note ? h('p', { class: 'hint' }, group.note) : null,
      ...group.controls.map((c) => this.control(c)),
    );
  }

  private control(c: Control): HTMLElement {
    const reset = c.reset === true;
    const wrap = (input: HTMLElement) =>
      h('div', { class: 'control' }, h('label', {}, c.label), input, this.errorSlot(c.path));
    switch (c.kind) {
      case 'toggle': {
        const box = h('input', {
          type: 'checkbox',
          onchange: () => this.commit((cfg) => setPath(cfg, c.path, box.checked), reset),
        });
        this.syncers.push(() => (box.checked = getPath(this.engine.config, c.path) === true));
        return h('div', { class: 'control' }, h('label', { class: 'switch' }, box, ` ${c.label}`), this.errorSlot(c.path));
      }
      case 'number': {
        const slider = h('input', { type: 'range', min: c.min, max: c.max, step: c.step });
        const num = h('input', { type: 'number', min: c.min, max: c.max, step: c.step, class: 'num' });
        const apply = (v: string) => this.commit((cfg) => setPath(cfg, c.path, Number(v)), reset);
        slider.addEventListener('input', () => (num.value = slider.value));
        slider.addEventListener('change', () => apply(slider.value));
        num.addEventListener('change', () => apply(num.value));
        this.syncers.push(() => {
          const v = String(getPath(this.engine.config, c.path));
          slider.value = v;
          num.value = v;
        });
        return wrap(h('div', { class: 'row' }, slider, num));
      }
      case 'range': {
        const lo = h('input', { type: 'number', min: c.min, max: c.max, class: 'num' });
        const hi = h('input', { type: 'number', min: c.min, max: c.max, class: 'num' });
        const apply = () =>
          this.commit((cfg) => setPath(cfg, c.path, { min: Number(lo.value), max: Number(hi.value) } satisfies URange), reset);
        lo.addEventListener('change', apply);
        hi.addEventListener('change', apply);
        this.syncers.push(() => {
          const r = getPath(this.engine.config, c.path) as URange;
          lo.value = String(r.min);
          hi.value = String(r.max);
        });
        return wrap(h('div', { class: 'row' }, lo, h('span', { class: 'hint' }, 'to'), hi));
      }
      case 'select': {
        const select = h(
          'select',
          {
            onchange: () => {
              const opt = c.options.find((o) => o.value === select.value);
              if (opt) this.commit(opt.apply, reset);
            },
          },
          ...c.options.map((o) => h('option', { value: o.value }, o.label)),
        );
        this.syncers.push(() => (select.value = c.current(this.engine.config)));
        return wrap(select);
      }
    }
  }
}
```

Append to `style.css`:
```css
.rules section { padding: 10px 0; border-bottom: 1px solid var(--border); }
.rules h3 { font-size: 13px; margin: 0 0 6px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); }
.rules h3 .switch { color: var(--text); text-transform: none; letter-spacing: 0; font-size: 14px; }
.rules .presets select { width: 100%; margin-top: 4px; }
.rules .control { display: grid; gap: 2px; margin: 6px 0; }
.rules .row { display: flex; gap: 8px; align-items: center; }
.rules .row input[type='range'] { flex: 1; }
.rules .num { width: 5.5em; }
.badge { font-size: 11px; background: var(--accent); color: var(--bg); padding: 1px 6px; border-radius: 99px; }
.error p { margin: 2px 0; }
```

- [ ] **Step 7: Wire tabs and panel into main**

In `main.ts`, add imports `import { RulesPanel } from './ui/rules-panel';` and `import { Tabs } from './ui/tabs';`, and after building the display controls:
```ts
  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  tabs.add('Rules', new RulesPanel(engine).el);
```
Export `tabs` for later tasks by keeping it in `main()` scope (later tasks add more `tabs.add` calls right after this one).

- [ ] **Step 8: Verify**

Run: `cd web && npm run typecheck && npm test`
Expected: typecheck clean; vitest 4 passed.
Manual (dev server): choosing presets resets the world and updates the description; toggling Sex on while Replacement is on shows the "mutually exclusive" error under Replacement and leaves the world unchanged; toggling Culture mid-run takes effect without a reset; editing Width resets the world; the "modified" badge appears after any change and disappears when a preset is re-chosen.

- [ ] **Step 9: Commit**

```bash
git add web
git commit -m "Add rules panel with presets, rule toggles and parameters"
```

---

### Task 18: Charts panel

**Files:**
- Create: `web/src/ui/charts-panel.ts`
- Modify: `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `Engine.sim.{series, lorenz, wealth_hist}`, `Engine.on('reset')`, `Tabs.add(label, el, onShow)`.
- Produces: `class ChartsPanel { el; setVisible(v: boolean); maybeRefresh(now: number); canvases(): { name: string; canvas: HTMLCanvasElement }[] }`.

- [ ] **Step 1: Implement the panel**

`web/src/ui/charts-panel.ts`:
```ts
import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type { Engine } from '../engine';
import { h } from './dom';

interface Line { key: string; label: string; color: string }
interface TimeChart { title: string; lines: Line[]; range?: [number, number] }

const TIME_CHARTS: TimeChart[] = [
  { title: 'Population', lines: [{ key: 'population', label: 'Agents', color: '--c1' }] },
  { title: 'Gini coefficient', lines: [{ key: 'gini', label: 'Gini', color: '--c2' }], range: [0, 1] },
  {
    title: 'Mean traits',
    lines: [
      { key: 'mean_vision', label: 'Vision', color: '--c1' },
      { key: 'mean_metabolism', label: 'Metabolism', color: '--c3' },
    ],
  },
  { title: 'Blue share', lines: [{ key: 'blue_fraction', label: 'Blue', color: '--blue' }], range: [0, 1] },
  {
    title: 'Births and deaths',
    lines: [
      { key: 'births', label: 'Births', color: '--c3' },
      { key: 'deaths', label: 'Deaths', color: '--c2' },
    ],
  },
];

const HEIGHT = 150;
const REFRESH_MS = 250;

export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private plots: { name: string; plot: uPlot; update: () => void }[] = [];
  private visible = false;
  private last = 0;

  constructor(private engine: Engine) {
    this.build();
    engine.on('reset', () => this.refresh());
    new ResizeObserver(() => this.resize()).observe(this.el);
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) {
      this.resize();
      this.refresh();
    }
  }

  /** Called every animation frame; redraws at most every REFRESH_MS while visible. */
  maybeRefresh(now: number): void {
    if (this.visible && now - this.last > REFRESH_MS) this.refresh();
  }

  canvases(): { name: string; canvas: HTMLCanvasElement }[] {
    return this.plots.map((p) => ({ name: p.name, canvas: p.plot.ctx.canvas }));
  }

  private refresh(): void {
    this.last = performance.now();
    if (this.visible) this.plots.forEach((p) => p.update());
  }

  private width(): number {
    return Math.max(240, this.el.clientWidth - 4);
  }

  private resize(): void {
    if (!this.visible) return;
    this.plots.forEach((p) => p.plot.setSize({ width: this.width(), height: HEIGHT }));
  }

  private add(title: string, opts: Omit<uPlot.Options, 'width' | 'height'>, data: uPlot.AlignedData, update: (plot: uPlot) => void): void {
    const figure = h('figure', { class: 'chart' }, h('figcaption', {}, title));
    const plot = new uPlot({ ...opts, width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ name: title, plot, update: () => update(plot) });
    this.el.append(figure);
  }

  private build(): void {
    const css = getComputedStyle(document.documentElement);
    const color = (v: string) => css.getPropertyValue(v).trim() || '#888';
    const axes: uPlot.Axis[] = [
      { stroke: color('--muted'), grid: { stroke: color('--grid') }, ticks: { stroke: color('--grid') } },
      { stroke: color('--muted'), grid: { stroke: color('--grid') }, ticks: { stroke: color('--grid') }, size: 44 },
    ];
    const series = (name: string) => Array.from(this.engine.sim.series(name));

    for (const chart of TIME_CHARTS) {
      this.add(
        chart.title,
        {
          scales: { x: { time: false }, y: chart.range ? { range: chart.range } : {} },
          axes,
          legend: { show: chart.lines.length > 1 },
          series: [{ label: 'Tick' }, ...chart.lines.map((l) => ({ label: l.label, stroke: color(l.color), width: 1.5 }))],
        },
        [[], ...chart.lines.map(() => [])],
        (plot) => plot.setData([series('tick'), ...chart.lines.map((l) => series(l.key))]),
      );
    }

    const xs = Array.from({ length: 101 }, (_, i) => i / 100);
    this.add(
      'Lorenz curve',
      {
        scales: { x: { time: false, range: [0, 1] }, y: { range: [0, 1] } },
        axes,
        legend: { show: false },
        series: [
          { label: 'Population share' },
          { label: 'Equality', stroke: color('--muted'), dash: [4, 4], width: 1 },
          { label: 'Wealth share', stroke: color('--c2'), width: 2 },
        ],
      },
      [xs, xs, xs],
      (plot) => plot.setData([xs, xs, Array.from(this.engine.sim.lorenz(101))]),
    );

    const bars = uPlot.paths.bars!({ size: [0.9, 64] });
    this.add(
      'Wealth distribution',
      {
        scales: { x: { time: false } },
        axes,
        legend: { show: false },
        series: [{ label: 'Sugar' }, { label: 'Agents', fill: color('--c1'), stroke: color('--c1'), paths: bars, points: { show: false } }],
      },
      [[], []],
      (plot) => {
        const hist = Array.from(this.engine.sim.wealth_hist(20));
        const width = hist[0];
        const counts = hist.slice(1);
        plot.setData([counts.map((_, i) => (i + 0.5) * width), counts]);
      },
    );
  }
}
```

Append to `style.css`:
```css
.charts { display: grid; gap: 12px; }
.chart { margin: 0; }
.chart figcaption { font-size: 12px; color: var(--muted); margin-bottom: 2px; }
.u-legend { font-size: 12px; color: var(--text); }
```

- [ ] **Step 2: Wire into main**

In `main.ts`, import `ChartsPanel`; after the Rules tab:
```ts
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
```
In the loop, after the draw block (inside `try`), add:
```ts
      charts.maybeRefresh(performance.now());
```

- [ ] **Step 3: Verify**

Run: `cd web && npm run typecheck && npm run build`
Expected: clean. If uPlot's types reject `paths`/`dash` options, check `node_modules/uplot/dist/uPlot.d.ts` for the current option names and adapt (the concepts — bar path builder, dashed series — are stable).
Manual: on the Charts tab with ({G₁}, {M}) running, population falls from 400 toward ~224; Gini rises; the Lorenz curve sits below the dashed diagonal; the histogram updates; resizing the window resizes the charts.

- [ ] **Step 4: Commit**

```bash
git add web
git commit -m "Add live charts: time series, Lorenz curve and wealth histogram"
```

---

### Task 19: Tools and the inspect panel

**Files:**
- Create: `web/src/ui/tools.ts`, `web/src/ui/inspect-panel.ts`
- Modify: `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `Engine.{select, follow, paint, place, erase, inspect, selection, setDisplay, on}`, `GridView.{onCell, brushRadius, draw}`, `Tabs`.
- Produces: `buildTools(engine, grid, onInspect: () => void): HTMLElement`; `class InspectPanel { el; setVisible(v) }`.

- [ ] **Step 1: Tools**

`web/src/ui/tools.ts`:
```ts
import type { Engine, PlaceOverrides } from '../engine';
import { h } from './dom';
import type { GridView } from './grid-view';

type Tool = 'inspect' | 'paint' | 'place' | 'erase';

const TOOLS: [Tool, string][] = [
  ['inspect', 'Inspect'],
  ['paint', 'Paint capacity'],
  ['place', 'Place agent'],
  ['erase', 'Erase agent'],
];

/** Tool picker; routes grid clicks/drags to the active tool. Edit errors (e.g. occupied site) are ignored. */
export function buildTools(engine: Engine, grid: GridView, onInspect: () => void): HTMLElement {
  let tool: Tool = 'inspect';
  let radius = 1;
  let value = 4;
  let sex: '' | 'female' | 'male' = '';
  let tribe: '' | 'blue' | 'red' = '';

  const buttons = TOOLS.map(([t, label]) => h('button', { onclick: () => choose(t) }, label));
  const options = h('div', { class: 'tool-options' });

  const number = (label: string, min: number, max: number, get: () => number, set: (v: number) => void) => {
    const input = h('input', { type: 'number', min, max, value: get(), class: 'num' });
    input.addEventListener('change', () => {
      set(Math.min(max, Math.max(min, Number(input.value))));
      input.value = String(get());
      if (tool === 'paint') grid.brushRadius = radius;
    });
    return h('label', {}, `${label} `, input);
  };
  const select = <T extends string>(label: string, values: [T, string][], set: (v: T) => void) => {
    const s = h('select', {}, ...values.map(([v, l]) => h('option', { value: v }, l)));
    s.addEventListener('change', () => set(s.value as T));
    return h('label', {}, `${label} `, s);
  };

  function choose(next: Tool): void {
    tool = next;
    buttons.forEach((b, i) => b.setAttribute('aria-pressed', String(TOOLS[i][0] === tool)));
    grid.brushRadius = tool === 'paint' ? radius : null;
    if (tool === 'paint') engine.setDisplay({ layer: 'capacity' });
    options.replaceChildren(
      ...(tool === 'paint'
        ? [number('Radius', 0, 10, () => radius, (v) => (radius = v)), number('Capacity', 0, 4, () => value, (v) => (value = v))]
        : tool === 'place'
          ? [
              select('Sex', [['', 'Random'], ['female', 'Female'], ['male', 'Male']], (v) => (sex = v)),
              select('Tribe', [['', 'Random'], ['blue', 'Blue'], ['red', 'Red']], (v) => (tribe = v)),
            ]
          : tool === 'inspect'
            ? [h('span', { class: 'hint' }, 'Click an agent or site.')]
            : [h('span', { class: 'hint' }, 'Click or drag over agents to remove them.')]),
    );
    grid.draw();
  }

  grid.onCell = (x, y, kind) => {
    switch (tool) {
      case 'inspect':
        if (kind === 'down') {
          engine.select(x, y);
          onInspect();
        }
        break;
      case 'paint':
        engine.paint(x, y, radius, value);
        break;
      case 'place':
        if (kind === 'down') {
          const o: PlaceOverrides = {};
          if (sex) o.sex = sex;
          if (tribe) o.tribe = tribe;
          engine.place(x, y, o);
        }
        break;
      case 'erase':
        engine.erase(x, y);
        break;
    }
  };

  choose('inspect');
  return h('div', { class: 'tools' }, h('div', { class: 'tool-buttons' }, ...buttons), options);
}
```

- [ ] **Step 2: Inspect panel**

`web/src/ui/inspect-panel.ts`:
```ts
import type { Engine } from '../engine';
import type { AgentView, LinkView } from '../types';
import { h } from './dom';

const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));

export class InspectPanel {
  readonly el = h('div', { class: 'inspect' });
  private visible = false;

  constructor(private engine: Engine) {
    for (const event of ['select', 'tick', 'reset', 'edit'] as const) engine.on(event, () => this.render());
    this.render();
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) this.render();
  }

  private links(links: LinkView[]): HTMLElement {
    if (links.length === 0) return h('span', { class: 'hint' }, 'none');
    return h(
      'span',
      { class: 'links' },
      ...links.map((l) =>
        l.alive
          ? h('button', { class: 'link', onclick: () => this.engine.follow(l.id) }, `#${l.id}`)
          : h('span', { class: 'hint', title: 'deceased' }, `#${l.id}†`),
      ),
    );
  }

  private agentRows(a: AgentView): HTMLElement[] {
    const row = (k: string, v: HTMLElement | string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    return [
      row('Agent', `#${a.id} · ${a.sex} · ${a.tribe}`),
      row('Sugar', `${fmt(a.sugar)} (born with ${fmt(a.initial_sugar)})`),
      row('Vision', String(a.vision)),
      row('Metabolism', String(a.metabolism)),
      row('Age', `${a.age} / ${a.max_age}`),
      row('Fertile', `${a.fertile ? 'yes' : 'no'} (ages ${a.fertility_onset}–${a.fertility_end})`),
      row('Culture tags', h('code', {}, a.tags)),
      row('Born', `tick ${a.born}`),
      row('Parents', this.links(a.parents)),
      row('Children', this.links(a.children)),
    ];
  }

  private render(): void {
    if (!this.visible) return;
    const sel = this.engine.selection;
    if (!sel) {
      this.el.replaceChildren(h('p', { class: 'hint' }, 'Choose the Inspect tool and click an agent or site.'));
      return;
    }
    const gone = sel.agentId !== null && !this.engine.sim.locate(sel.agentId);
    const { site, agent } = this.engine.inspect(sel.x, sel.y);
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    this.el.replaceChildren(
      gone ? h('p', { class: 'error' }, `Agent #${sel.agentId} has died.`) : null,
      h(
        'table',
        {},
        row('Site', `(${site.x}, ${site.y})`),
        row('Sugar', `${fmt(site.sugar)} / ${fmt(site.capacity)}`),
        row('Pollution', fmt(site.pollution)),
        ...(agent && !gone ? this.agentRows(agent) : []),
      ),
    );
  }
}
```

Append to `style.css`:
```css
.tools { display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: center; }
.tool-buttons { display: flex; gap: 4px; flex-wrap: wrap; }
.tool-options { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.tool-options .num { width: 4.5em; }
.inspect table { border-collapse: collapse; width: 100%; }
.inspect th { text-align: left; color: var(--muted); font-weight: normal; padding: 3px 10px 3px 0; white-space: nowrap; vertical-align: top; }
.inspect td { padding: 3px 0; font-variant-numeric: tabular-nums; }
.inspect code { font-size: 12px; letter-spacing: 0.08em; }
.links { display: flex; flex-wrap: wrap; gap: 4px; }
button.link { padding: 0 6px; font-size: 12px; }
```

- [ ] **Step 3: Wire into main**

In `main.ts`, import `buildTools` and `InspectPanel`; after the Charts tab:
```ts
  const inspect = new InspectPanel(engine);
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  document.querySelector('#tools')!.append(buildTools(engine, grid, () => tabs.show('Inspect')));
```

- [ ] **Step 4: Verify**

Run: `cd web && npm run typecheck && npm run build`
Expected: clean.
Manual: Inspect → clicking an agent opens the Inspect tab with its traits, and the selection ring follows it while running; enable Sex (preset iii-2-sex), run ~50 ticks, inspect a child and click a parent link to jump to it; Paint capacity switches the layer to Capacity and dragging paints a disc; Place agent with Tribe = Red drops a red agent; Erase removes agents under a drag; clicking an occupied site with Place does nothing.

- [ ] **Step 5: Commit**

```bash
git add web
git commit -m "Add editing tools and agent inspector"
```

---

### Task 20: Share links and exports

**Files:**
- Create: `web/src/share.ts`, `web/src/share.test.ts`, `web/src/downloads.ts`
- Modify: `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `Engine.{config, seed, sim.landscape_edited, sim.export_landscape, sim.export_series_csv, sim.export_agents_csv}`, `Engine.create(initial)`, `GridView.toPngBlob`, `ChartsPanel.canvases`.
- Produces: `share.ts`: `ShareState`, `encodeShare(state): Promise<string>`, `decodeShare(token): Promise<ShareState>`, `readHash(hash?): string | null`, `bytesToBase64Url`, `base64UrlToBytes`; `downloads.ts`: `downloadText(name, text, type?)`, `downloadBlob(name, blob)`, `canvasBlob(canvas): Promise<Blob>`.

- [ ] **Step 1: Write failing share tests**

`web/src/share.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import type { Config } from './types';
import { base64UrlToBytes, bytesToBase64Url, decodeShare, encodeShare, readHash } from './share';

const config = { width: 50, height: 50, population: 400, sex: { enabled: true } } as unknown as Config;

describe('share links', () => {
  it('round-trips config and seed', async () => {
    const token = await encodeShare({ config, seed: 123456789 });
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    const back = await decodeShare(token);
    expect(back).toEqual({ config, seed: 123456789 });
  });

  it('round-trips a painted landscape', async () => {
    const landscape = Uint8Array.from({ length: 2500 }, (_, i) => i % 5);
    const back = await decodeShare(await encodeShare({ config, seed: 1, landscape }));
    expect(Array.from(back.landscape!)).toEqual(Array.from(landscape));
  });

  it('compresses a mostly-uniform landscape well', async () => {
    const token = await encodeShare({ config, seed: 1, landscape: new Uint8Array(2500).fill(2) });
    expect(token.length).toBeLessThan(400);
  });

  it('rejects garbage', async () => {
    await expect(decodeShare('not-a-real-token')).rejects.toThrow();
    const wrong = bytesToBase64Url(new TextEncoder().encode('{}'));
    await expect(decodeShare(wrong)).rejects.toThrow();
  });

  it('base64url round-trips every padding length', () => {
    for (const n of [0, 1, 2, 3, 4, 5]) {
      const bytes = Uint8Array.from({ length: n }, (_, i) => 250 - i);
      expect(Array.from(base64UrlToBytes(bytesToBase64Url(bytes)))).toEqual(Array.from(bytes));
    }
  });

  it('reads the share token from the hash', () => {
    expect(readHash('#s=abc_-1')).toBe('abc_-1');
    expect(readHash('#other')).toBeNull();
    expect(readHash('')).toBeNull();
  });
});
```

- [ ] **Step 2: Run to verify failure**

Run: `cd web && npx vitest run src/share.test.ts`
Expected: FAIL — cannot resolve `./share`.

- [ ] **Step 3: Implement share and downloads**

`web/src/share.ts`:
```ts
import type { Config } from './types';

export interface ShareState { config: Config; seed: number; landscape?: Uint8Array }

interface Wire { v: 1; c: Config; s: number; l?: string }

export function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export function base64UrlToBytes(text: string): Uint8Array {
  const b64 = text.replace(/-/g, '+').replace(/_/g, '/') + '==='.slice((text.length + 3) % 4);
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

async function pipe(bytes: Uint8Array, stream: CompressionStream | DecompressionStream): Promise<Uint8Array> {
  const out = new Blob([new Uint8Array(bytes)]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(out).arrayBuffer());
}

/** base64url(deflate-raw(JSON)). */
export async function encodeShare(state: ShareState): Promise<string> {
  const wire: Wire = { v: 1, c: state.config, s: state.seed };
  if (state.landscape) wire.l = bytesToBase64Url(state.landscape);
  const json = new TextEncoder().encode(JSON.stringify(wire));
  return bytesToBase64Url(await pipe(json, new CompressionStream('deflate-raw')));
}

export async function decodeShare(token: string): Promise<ShareState> {
  const json = await pipe(base64UrlToBytes(token), new DecompressionStream('deflate-raw'));
  const wire = JSON.parse(new TextDecoder().decode(json)) as Partial<Wire>;
  if (wire.v !== 1 || typeof wire.s !== 'number' || typeof wire.c !== 'object' || wire.c === null) {
    throw new Error('not a SugarScape share link');
  }
  const state: ShareState = { config: wire.c, seed: wire.s >>> 0 };
  if (wire.l) state.landscape = base64UrlToBytes(wire.l);
  return state;
}

export function readHash(hash: string = location.hash): string | null {
  return /^#s=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}
```

`web/src/downloads.ts`:
```ts
export function downloadBlob(name: string, blob: Blob): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function downloadText(name: string, text: string, type = 'text/csv'): void {
  downloadBlob(name, new Blob([text], { type }));
}

export function canvasBlob(canvas: HTMLCanvasElement): Promise<Blob> {
  return new Promise((resolve, reject) =>
    canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('PNG encoding failed'))), 'image/png'),
  );
}
```

- [ ] **Step 4: Run tests**

Run: `cd web && npm test`
Expected: all vitest suites pass (paths 4, share 6).

- [ ] **Step 5: Load shared state and add Share/Export controls**

In `main.ts`:
1. Import `decodeShare, encodeShare, readHash` from `./share` and `downloadBlob, downloadText, canvasBlob` from `./downloads`.
2. Replace `const engine = await Engine.create();` with:
```ts
  let engine: Engine;
  const token = readHash();
  try {
    engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
  } catch (e) {
    showBanner(`That share link could not be loaded (${e instanceof Error ? e.message : String(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
```
3. After the tools are wired, add the share/export controls into the toolbar's end slot:
```ts
  const slug = () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}-t${engine.sim.tick()}`;
  const shareButton = h('button', {
    onclick: async () => {
      const landscape = engine.sim.landscape_edited() ? engine.sim.export_landscape() : undefined;
      const token = await encodeShare({ config: engine.config, seed: engine.seed, landscape });
      history.replaceState(null, '', `#s=${token}`);
      try {
        await navigator.clipboard.writeText(location.href);
        shareButton.textContent = 'Link copied';
      } catch {
        shareButton.textContent = 'Link in address bar';
      }
      setTimeout(() => (shareButton.textContent = 'Share'), 2000);
    },
    title: 'Copy a link to this setup (config, seed and painted landscape; hand-placed agents are not included)',
  }, 'Share');
  const menu = h(
    'details',
    { class: 'menu' },
    h('summary', {}, 'Export'),
    h('div', { class: 'menu-items' },
      h('button', { onclick: () => downloadText(`${slug()}-series.csv`, engine.sim.export_series_csv()) }, 'Statistics (CSV)'),
      h('button', { onclick: () => downloadText(`${slug()}-agents.csv`, engine.sim.export_agents_csv()) }, 'Agents (CSV)'),
      h('button', { onclick: async () => downloadBlob(`${slug()}-grid.png`, await grid.toPngBlob()) }, 'Grid (PNG)'),
      h('button', {
        onclick: async () => {
          tabs.show('Charts');
          for (const { name, canvas } of charts.canvases()) {
            downloadBlob(`${slug()}-${name.toLowerCase().replace(/\W+/g, '-')}.png`, await canvasBlob(canvas));
          }
        },
      }, 'Charts (PNG)'),
    ),
  );
  document.querySelector('.toolbar-end')!.append(shareButton, menu);
```
(The Charts export switches to the Charts tab first so the canvases are laid out and freshly drawn.)

Append to `style.css`:
```css
.menu { position: relative; }
.menu summary { list-style: none; cursor: pointer; border: 1px solid var(--border); border-radius: 6px; padding: 4px 10px; background: var(--surface); }
.menu summary::-webkit-details-marker { display: none; }
.menu-items { position: absolute; right: 0; top: calc(100% + 4px); display: grid; gap: 4px; padding: 6px; background: var(--surface); border: 1px solid var(--border); border-radius: 8px; z-index: 10; min-width: 12em; }
.menu-items button { text-align: left; }
```

- [ ] **Step 6: Verify**

Run: `cd web && npm run typecheck && npm test && npm run build`
Expected: clean; all tests pass.
Manual: pick a preset, change a parameter, paint a patch, click Share; open the copied URL in a new tab → the same config (Rules panel shows the same values, "modified" as appropriate), the same seed, the painted patch present, and an identical initial world as the original tab after Reset. Open `…#s=garbage` → banner explains and default preset loads. Each Export item downloads a non-empty file.

- [ ] **Step 7: Commit**

```bash
git add web
git commit -m "Add share links and CSV/PNG exports"
```

---

### Task 21: CI, GitHub Pages deploy and README

**Files:**
- Create: `.github/workflows/ci.yml`, `.github/workflows/pages.yml`
- Modify: `README.md`

- [ ] **Step 1: CI workflow**

`.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test -p sugarscape-core
      - run: cargo test -p sugarscape-core --release --test book -- --ignored
      - run: cargo build -p sugarscape-core --target wasm32-unknown-unknown
      - uses: jetli/wasm-pack-action@v0.4.0
      - run: wasm-pack test --node crates/sugarscape-wasm

  web:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: web
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - uses: jetli/wasm-pack-action@v0.4.0
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm
          cache-dependency-path: web/package-lock.json
      - run: npm ci
      - run: npm run build
      - run: npm test
```

- [ ] **Step 2: Pages workflow**

`.github/workflows/pages.yml`:
```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - uses: jetli/wasm-pack-action@v0.4.0
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm
          cache-dependency-path: web/package-lock.json
      - run: npm ci
        working-directory: web
      - run: npm run build
        working-directory: web
      - uses: actions/upload-pages-artifact@v3
        with:
          path: web/dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 3: README**

Replace `README.md` with:
```markdown
# SugarScape

A browser playground for the Sugarscape model from Joshua M. Epstein and Robert Axtell,
*Growing Artificial Societies: Social Science from the Bottom Up* (1996).

The simulation is written in Rust (`crates/sugarscape-core`), compiled to WebAssembly
(`crates/sugarscape-wasm`), and driven by a small TypeScript front end (`web/`).

## Rules implemented

Chapters II–III of the book: sugar growback (G) and seasons, movement (M), pollution
formation and diffusion (P, D), replacement (R), sexual reproduction (S), inheritance (I),
cultural transmission and tribes (K), and combat (C). Presets reproduce the book's
animations. Where the book is ambiguous, the choice made is documented in the rule's
module (see `crates/sugarscape-core/src/rules/`) and in
`docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md`.

## Running locally

Requirements: Rust with the `wasm32-unknown-unknown` target, `wasm-pack`, Node 22+.

    cd web
    npm install
    npm run dev

## Tests

    cargo test -p sugarscape-core                                   # unit + property tests
    cargo test -p sugarscape-core --release --test book -- --ignored # book reproductions
    wasm-pack test --node crates/sugarscape-wasm                    # bindings
    cd web && npm test                                              # front-end logic

## Credits

The 50×50 two-peak sugar map is a transcription of the book's Figure II-1 as distributed
with the NetLogo Sugarscape models.
```

- [ ] **Step 4: Full verification**

Run each and confirm success:
```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test -p sugarscape-core
cargo test -p sugarscape-core --release --test book -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm ci && npm run build && npm test)
```
Then serve `web/dist` (`cd web && npx vite preview`) and run through the manual checklists of Tasks 16–20 once more against the production build.

- [ ] **Step 5: Commit**

```bash
git add .github README.md
git commit -m "Add CI, GitHub Pages deployment and README"
```
Do not push. Tell the user that Pages needs "Build and deployment → Source: GitHub Actions" enabled in the repository settings before the deploy workflow can publish.
