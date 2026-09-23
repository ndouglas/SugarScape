# Chapter IV — Sugar and Spice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the book's Chapter IV economy — spice, multicommodity movement, trade (T), credit (L), foresight, scheduled rule changes, supply & demand, trade/credit networks — to the SugarScape playground without changing any milestone-1 run.

**Architecture:** Sugar and spice are explicit fields on sites and agents; every new rule is a config toggle and is skipped (including its RNG draws) when off. New rule modules (`rules/trade.rs`, `rules/credit.rs`), an `econ.rs` of pure welfare/valuation functions and a `network.rs` of queries sit beside the existing ones; the WASM `Sim` gains a few methods; the web UI gains rule groups, a schedule list, layers, network overlays and an Economy chart group.

**Tech Stack:** unchanged — Rust (`sugarscape-core`, `sugarscape-wasm`, wasm-bindgen, proptest), Vite + TypeScript + uPlot + Vitest.

**Spec:** `docs/superpowers/specs/2026-09-22-chapter-iv-sugar-and-spice-design.md` (binding), building on `docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md`.

## Global Constraints

- **Milestone-1 invariance:** with `spice`, `trade`, `credit`, `foresight` off and an empty `schedule`, every existing preset must produce the same `World::fingerprint()` after 200 ticks (seed 1) as before this milestone (`tests/golden.rs`, Task 1). New RNG draws happen only when their rule is on; new per-turn steps are skipped when off; `fingerprint()` hashes new state only when its rule is on.
- Determinism: all randomness through `World.rng`; iterate only `Vec`/`BTreeMap`.
- Lattice: `y = 0` is north; torus; four directions only.
- Rule order inside an agent's turn: move (M or C) → metabolize → [credit income] → death check → S → K → T → L-borrow. After all turns: settle due loans (whenever loans exist), then growback, diffusion, replacement, ageing, stats. Scheduled changes apply at the start of `step()` when `entry.tick == world.tick` (completed ticks).
- JS seeds are `u32`; errors crossing into JS are JSON `[{field, message}]`.
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3`.
- Rust tasks end with `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` clean and `cargo test -p sugarscape-core` passing (including `tests/golden.rs`). Web tasks end with `cd web && npm run build && npm test` passing.
- Web tasks cannot be browser-verified by implementers; the controller runs a puppeteer check.

## File Structure

```
crates/sugarscape-core/
  tests/golden.rs            NEW  milestone-1 fingerprints
  src/config.rs              MOD  SpiceRule, CreditRule, Foresight, ScheduledChange, schedule validation, with_path/apply_change
  src/landscape.rs           MOD  Site spice fields, spice_capacities()
  src/agent.rs               MOD  spice/foresight/income fields, conditional draws, fertility
  src/econ.rs                NEW  welfare, foresight welfare, MRS, sugar demand
  src/world.rs               MOD  schedule application, loans storage, Trade events, fingerprint, bequeath both goods
  src/rules/mod.rs           MOD  Harvest, agent_turn order
  src/rules/movement.rs      MOD  multicommodity M
  src/rules/lifecycle.rs     MOD  spice metabolism/death, sugar-dirty pollution
  src/rules/combat.rs        MOD  returns Harvest
  src/rules/growback.rs      MOD  spice growback
  src/rules/sex.rs           MOD  spice endowment, spice metabolism/foresight inheritance
  src/rules/trade.rs         NEW  T
  src/rules/credit.rs        NEW  L
  src/network.rs             NEW  trade/credit edges, credit roles
  src/stats.rs               MOD  new series, supply_demand()
  src/render.rs              MOD  spice layers, credit color mode
  src/edit.rs                MOD  inspection of spice/foresight/loans
  src/export.rs              MOD  new agent columns
  src/presets.rs             MOD  six Chapter IV presets, ii-8 schedule
  src/testkit.rs             MOD  new Agent fields
  tests/invariants.rs        MOD  new rules in the strategy + invariants
  tests/book.rs              MOD  trade price, carrying capacity, foresight
crates/sugarscape-wasm/src/lib.rs, tests/web.rs   MOD  networks(), supply_demand()
web/src/types.ts, schema.ts                        MOD
web/src/ui/rules-panel.ts                          MOD  schedule section
web/src/ui/display.ts, engine.ts, grid-view.ts     MOD  layers, credit mode, overlays
web/src/ui/overlay.ts (+ overlay.test.ts)          NEW  torus-wrapped segments
web/src/ui/charts-panel.ts                         MOD  Economy group
web/src/ui/inspect-panel.ts                        MOD
README.md                                          MOD
```

---

### Task 1: Golden fingerprints for milestone-1 presets

**Files:**
- Create: `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `presets::by_id`, `World::{new, run, fingerprint}`.
- Produces: `GOLDEN: &[(&str, u64)]` — later tasks must keep it passing (Task 13 updates `ii-8-pollution` intentionally).

- [ ] **Step 1: Write the recorder and the test**

`crates/sugarscape-core/tests/golden.rs`:
```rust
//! Milestone-1 invariance: with every Chapter IV rule off, the milestone-1
//! presets evolve exactly as they did before Chapter IV was added.

use sugarscape_core::presets;
use sugarscape_core::world::World;

/// (preset id, fingerprint after 200 ticks from seed 1).
const GOLDEN: &[(&str, u64)] = &[];

fn fingerprint(id: &str) -> u64 {
    let preset = presets::by_id(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    let mut world = World::new(preset.config, 1).unwrap();
    world.run(200);
    world.fingerprint()
}

#[test]
fn milestone_one_presets_are_unchanged() {
    assert!(!GOLDEN.is_empty(), "record the golden values first");
    for &(id, expected) in GOLDEN {
        assert_eq!(fingerprint(id), expected, "preset {id} changed");
    }
}

/// Prints `GOLDEN` entries: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_golden() {
    for p in presets::all() {
        println!("    (\"{}\", {:#x}),", p.id, fingerprint(p.id));
    }
}
```

- [ ] **Step 2: Record the values**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`
Paste the 13 printed lines into `GOLDEN` (between the brackets).

- [ ] **Step 3: Verify**

Run: `cargo test -p sugarscape-core --test golden`
Expected: 1 passed, 1 ignored.

- [ ] **Step 4: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/tests/golden.rs
git commit -m "Record milestone-1 golden fingerprints"
```

---

### Task 2: Configuration for spice, trade, credit and foresight

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs`

**Interfaces:**
- Produces: `config::{SpiceRule { enabled, metabolism: URange, endowment: URange }, CreditRule { enabled, duration: u32, rate: f64 }, Foresight { enabled, range: URange }}`; `Pollution.spice_pollutes: bool` (`#[serde(default)]`); `Config.{spice, trade: Toggle, credit, foresight}`.

- [ ] **Step 1: Write failing tests** (append to `config.rs` tests)

```rust
    #[test]
    fn chapter_four_defaults_are_off_and_old_json_still_loads() {
        let c = Config::default();
        assert!(!c.spice.enabled && !c.trade.enabled && !c.credit.enabled && !c.foresight.enabled);
        assert_eq!(c.spice.metabolism, URange::new(1, 4));
        assert_eq!(c.spice.endowment, URange::new(5, 25));
        assert_eq!((c.credit.duration, c.credit.rate), (10, 10.0));
        assert_eq!(c.foresight.range, URange::new(0, 10));
        assert!(!c.pollution.spice_pollutes);
        let old = r#"{"pollution":{"enabled":true,"production":1.0,"consumption":1.0}}"#;
        let loaded = Config::from_json(old).unwrap();
        assert!(loaded.pollution.enabled && !loaded.pollution.spice_pollutes);
    }

    #[test]
    fn chapter_four_rule_dependencies_are_validated() {
        let with = |f: fn(&mut Config)| {
            let mut c = Config::default();
            f(&mut c);
            fields(c.validate())
        };
        assert!(with(|c| c.trade.enabled = true).contains(&"trade.enabled".to_string()));
        assert!(with(|c| c.foresight.enabled = true).contains(&"foresight.enabled".to_string()));
        assert!(with(|c| c.credit.enabled = true).contains(&"credit.enabled".to_string()));
        assert!(with(|c| {
            c.spice.enabled = true;
            c.combat.enabled = true;
        })
        .contains(&"combat.enabled".to_string()));
        assert!(with(|c| c.credit.duration = 0).contains(&"credit.duration".to_string()));
        assert!(with(|c| c.credit.rate = -1.0).contains(&"credit.rate".to_string()));
        assert!(with(|c| c.spice.metabolism = URange::new(3, 1)).contains(&"spice.metabolism".to_string()));
        assert!(with(|c| {
            c.spice.enabled = true;
            c.trade.enabled = true;
            c.foresight.enabled = true;
        })
        .is_empty());
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core config`
Expected: FAIL to compile — `spice`, `trade`, … not found.

- [ ] **Step 3: Implement**

Add after `CombatRule`:
```rust
/// Chapter IV's second commodity. When off, agents never draw spice traits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpiceRule {
    pub enabled: bool,
    pub metabolism: URange,
    pub endowment: URange,
}

/// L_{d,r}: sugar loans of `duration` (d) ticks at `rate` (r) percent simple
/// interest per tick.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreditRule {
    pub enabled: bool,
    pub duration: u32,
    pub rate: f64,
}

/// Book eq. 6: agents value holdings as if `φ` periods of metabolism were
/// already spent; `range` is φ's initial distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Foresight {
    pub enabled: bool,
    pub range: URange,
}
```

In `Pollution` add, after `consumption`:
```rust
    /// Chapter IV makes sugar the only "dirty" good; set to pollute spice too.
    #[serde(default)]
    pub spice_pollutes: bool,
```

Append to `Config` (after `combat`):
```rust
    pub spice: SpiceRule,
    pub trade: Toggle,
    pub credit: CreditRule,
    pub foresight: Foresight,
```

In `Default for Config`, set `pollution.spice_pollutes: false` and append:
```rust
            spice: SpiceRule {
                enabled: false,
                metabolism: URange::new(1, 4),
                endowment: URange::new(5, 25),
            },
            trade: Toggle { enabled: false },
            credit: CreditRule {
                enabled: false,
                duration: 10,
                rate: 10.0,
            },
            foresight: Foresight {
                enabled: false,
                range: URange::new(0, 10),
            },
```

In `validate()`, before `e.finish()`:
```rust
        e.range(self.spice.metabolism, "spice.metabolism");
        e.range(self.spice.endowment, "spice.endowment");
        e.range(self.foresight.range, "foresight.range");
        e.check(
            !self.trade.enabled || self.spice.enabled,
            "trade.enabled",
            "trade (T) needs spice on",
        );
        e.check(
            !self.foresight.enabled || self.spice.enabled,
            "foresight.enabled",
            "foresight needs spice on",
        );
        e.check(
            !self.credit.enabled || self.sex.enabled,
            "credit.enabled",
            "credit (L) needs sex (S) on",
        );
        e.check(
            !(self.combat.enabled && self.spice.enabled),
            "combat.enabled",
            "combat (C) and spice are mutually exclusive",
        );
        e.check(self.credit.duration >= 1, "credit.duration", "must be ≥ 1");
        e.non_negative(self.credit.rate, "credit.rate");
```

Fix every other `Pollution { .. }` / `Config { .. }` literal the compiler flags (presets and tests use `..Config::default()` or field assignment, so there should be none).

- [ ] **Step 4: Verify**

Run: `cargo test -p sugarscape-core`
Expected: all pass, including `tests/golden.rs` (serialization-only change).

- [ ] **Step 5: Commit** — `git commit -m "Add Chapter IV rule configuration"` (after fmt/clippy).

---

### Task 3: Scheduled rule changes

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs`, `crates/sugarscape-core/src/world.rs`

**Interfaces:**
- Produces: `config::ScheduledChange { tick: u64, set: BTreeMap<String, serde_json::Value> }`; `Config.schedule: Vec<ScheduledChange>`; `Config::with_path(&self, path: &str, value: &serde_json::Value) -> Result<Config, FieldError>`; `Config::apply_change(&self, &ScheduledChange) -> Result<Config, FieldError>`; `config::STRUCTURAL_FIELDS`; schedule applied at the start of `World::step`.

- [ ] **Step 1: Write failing tests**

Append to `config.rs` tests:
```rust
    fn change(tick: u64, path: &str, value: serde_json::Value) -> ScheduledChange {
        ScheduledChange {
            tick,
            set: [(path.to_string(), value)].into_iter().collect(),
        }
    }

    #[test]
    fn with_path_sets_nested_fields_and_rejects_unknown_ones() {
        let c = Config::default();
        let on = c.with_path("pollution.enabled", &serde_json::json!(true)).unwrap();
        assert!(on.pollution.enabled);
        assert_eq!(c.with_path("pollution.nope", &serde_json::json!(1)).unwrap_err().field, "schedule");
        assert_eq!(c.with_path("pollution.enabled", &serde_json::json!("yes")).unwrap_err().field, "schedule");
    }

    #[test]
    fn schedule_entries_are_validated() {
        let mut c = Config::default();
        c.schedule = vec![change(50, "pollution.enabled", serde_json::json!(true))];
        c.validate().unwrap();
        c.schedule = vec![change(0, "pollution.enabled", serde_json::json!(true))];
        assert_eq!(fields(c.validate()), vec!["schedule"]);
        c.schedule = vec![change(5, "width", serde_json::json!(60))];
        assert_eq!(fields(c.validate()), vec!["schedule"]);
        c.schedule = vec![change(5, "trade.enabled", serde_json::json!(true))];
        assert_eq!(fields(c.validate()), vec!["schedule"], "trade without spice is invalid");
    }
```

Append to `world.rs` tests:
```rust
    #[test]
    fn scheduled_changes_apply_when_their_tick_is_reached() {
        use crate::config::ScheduledChange;
        let mut c = crate::testkit::blank_config(10, 10);
        c.schedule = vec![ScheduledChange {
            tick: 2,
            set: [("pollution.enabled".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        }];
        let mut w = World::new(c, 1).unwrap();
        w.step(); // tick 0 → 1
        assert!(!w.config.pollution.enabled);
        w.step(); // tick 1 → 2
        assert!(!w.config.pollution.enabled);
        w.step(); // starts at tick 2: applied
        assert!(w.config.pollution.enabled);
        assert_eq!(w.config.schedule.len(), 1, "the schedule itself is kept");
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sugarscape-core`
Expected: FAIL to compile — `ScheduledChange`, `with_path` missing.

- [ ] **Step 3: Implement**

In `config.rs` add `use std::collections::BTreeMap;` and:
```rust
/// Fields a schedule may not change (they shape the world's storage or setup).
pub const STRUCTURAL_FIELDS: [&str; 6] =
    ["width", "height", "tag_length", "landscape", "population", "placement"];

/// At the start of the tick when `World::tick == tick`, set each dotted config
/// path in `set` to its value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledChange {
    pub tick: u64,
    pub set: BTreeMap<String, serde_json::Value>,
}
```
Append `pub schedule: Vec<ScheduledChange>,` to `Config` (last field) and `schedule: Vec::new(),` to `Default`.

Split validation so the schedule can be checked without recursion:
```rust
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        self.validate_fields()?;
        self.validate_schedule()
    }

    /// Everything except the schedule (renamed from the old `validate` body).
    fn validate_fields(&self) -> Result<(), Vec<FieldError>> {
        // … the existing validate() body, unchanged, including Task 2's checks …
    }

    fn validate_schedule(&self) -> Result<(), Vec<FieldError>> {
        let mut entries: Vec<&ScheduledChange> = self.schedule.iter().collect();
        entries.sort_by_key(|c| c.tick);
        let mut patched = self.clone();
        for change in entries {
            if change.tick == 0 {
                return Err(vec![FieldError::new("schedule", "scheduled ticks start at 1")]);
            }
            patched = patched.apply_change(change).map_err(|e| vec![e])?;
        }
        Ok(())
    }

    /// A copy with one dotted `path` set to `value`.
    pub fn with_path(&self, path: &str, value: &serde_json::Value) -> Result<Config, FieldError> {
        let mut json = serde_json::to_value(self).expect("config serializes");
        let mut slot = &mut json;
        for key in path.split('.') {
            slot = slot
                .get_mut(key)
                .ok_or_else(|| FieldError::new("schedule", format!("unknown field {path}")))?;
        }
        *slot = value.clone();
        serde_json::from_value(json).map_err(|e| FieldError::new("schedule", format!("{path}: {e}")))
    }

    /// This config with every path in `change` set, checked for validity
    /// (excluding the schedule itself).
    pub fn apply_change(&self, change: &ScheduledChange) -> Result<Config, FieldError> {
        let mut next = self.clone();
        for (path, value) in &change.set {
            let root = path.split('.').next().unwrap_or_default();
            if STRUCTURAL_FIELDS.contains(&root) {
                return Err(FieldError::new("schedule", format!("{path} changes only on reset")));
            }
            next = next.with_path(path, value)?;
        }
        next.validate_fields().map_err(|errs| {
            let e = &errs[0];
            FieldError::new("schedule", format!("at t={}: {}: {}", change.tick, e.field, e.message))
        })?;
        Ok(next)
    }
```

In `world.rs`, first lines of `step()` (after resetting `events`):
```rust
        self.apply_schedule();
```
and add:
```rust
    /// Applies scheduled changes due at the tick about to run. Entries were
    /// validated with the config, so failures are impossible; they are ignored.
    fn apply_schedule(&mut self) {
        let due: Vec<_> = self
            .config
            .schedule
            .iter()
            .filter(|c| c.tick == self.tick)
            .cloned()
            .collect();
        for change in due {
            if let Ok(next) = self.config.apply_change(&change) {
                self.config = next;
            }
        }
    }
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (all pass, golden unchanged).
- [ ] **Step 5: Commit** — `git commit -m "Add scheduled rule changes"`.

---

### Task 4: Spice on the landscape

**Files:**
- Modify: `crates/sugarscape-core/src/landscape.rs`, `src/world.rs`, `src/rules/growback.rs`

**Interfaces:**
- Produces: `Site.{spice, spice_capacity}`; `Site::with_spice(self, capacity) -> Site`; `landscape::spice_capacities(&LandscapeKind, w, h) -> Vec<f64>` (TwoPeaks mirrored left↔right); growback of spice when `spice.enabled`; `fingerprint()` hashes site spice only when spice is on.

- [ ] **Step 1: Failing tests**

`landscape.rs` tests:
```rust
    #[test]
    fn spice_map_mirrors_sugar_map_into_northwest_and_southeast() {
        let sugar = capacities(&LandscapeKind::TwoPeaks, 50, 50);
        let spice = spice_capacities(&LandscapeKind::TwoPeaks, 50, 50);
        let at = |v: &[f64], x: usize, y: usize| v[y * 50 + x];
        assert_eq!(at(&spice, 12, 5), 4.0, "northwest peak");
        assert_eq!(at(&spice, 34, 40), 4.0, "southeast peak");
        for y in 0..50 {
            for x in 0..50 {
                assert_eq!(at(&spice, x, y), at(&sugar, 49 - x, y));
            }
        }
        assert_eq!(spice_capacities(&LandscapeKind::Flat { capacity: 3.0 }, 5, 5), vec![3.0; 25]);
    }
```
`growback.rs` tests:
```rust
    #[test]
    fn spice_grows_back_only_when_enabled() {
        let mut w = world_with_empty_site(3.0);
        w.site_mut(Pos::new(3, 3)).spice_capacity = 3.0;
        apply(&mut w);
        assert_eq!(w.site(Pos::new(3, 3)).spice, 0.0);
        w.config.spice.enabled = true;
        apply(&mut w);
        assert_eq!(w.site(Pos::new(3, 3)).spice, 1.0);
    }
```

- [ ] **Step 2: Run** — expect compile failure (`spice_capacities`, `spice` field).

- [ ] **Step 3: Implement**

`landscape.rs` — extend `Site` and add the mirror map:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Site {
    pub sugar: f64,
    pub capacity: f64,
    pub pollution: f64,
    pub spice: f64,
    pub spice_capacity: f64,
}

impl Site {
    /// A site whose sugar starts at capacity (no spice).
    pub fn full(capacity: f64) -> Self {
        Self { sugar: capacity, capacity, ..Self::default() }
    }

    /// The same site with spice at `capacity`.
    pub fn with_spice(self, capacity: f64) -> Self {
        Self { spice: capacity, spice_capacity: capacity, ..self }
    }
}

/// Spice capacities: the two-peak sugar map mirrored left↔right, putting spice
/// mountains in the northwest and southeast (the book's Figure IV-1).
pub fn spice_capacities(kind: &LandscapeKind, width: u32, height: u32) -> Vec<f64> {
    let sugar = capacities(kind, width, height);
    match kind {
        LandscapeKind::TwoPeaks => {
            let (w, h) = (width as usize, height as usize);
            (0..w * h).map(|i| sugar[(i / w) * w + (w - 1 - i % w)]).collect()
        }
        LandscapeKind::Flat { .. } => sugar,
    }
}
```

`world.rs` `with_capacities`: build sites with spice (spice always from the configured landscape; painting affects sugar only):
```rust
        let spice = landscape::spice_capacities(&config.landscape, config.width, config.height);
        …
            sites: caps
                .into_iter()
                .zip(spice)
                .map(|(sugar, spice)| Site::full(sugar).with_spice(spice))
                .collect(),
```
`fingerprint()` — after the site loop's existing three `eat`s, inside the same loop:
```rust
            if self.config.spice.enabled {
                eat(s.spice.to_bits());
                eat(s.spice_capacity.to_bits());
            }
```
(Read `self.config.spice.enabled` into a local before the closure if the borrow checker requires.)

`growback.rs` `apply` — after updating sugar in the loop:
```rust
        if spice {
            site.spice = if instant {
                site.spice_capacity
            } else {
                (site.spice + rate).min(site.spice_capacity)
            };
        }
```
with `let spice = world.config.spice.enabled;` at the top.

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged).
- [ ] **Step 5: Commit** — `git commit -m "Add spice to the landscape"`.

---

### Task 5: Agent spice, foresight and income

**Files:**
- Modify: `src/agent.rs`, `src/testkit.rs`, `src/rules/sex.rs`, `src/world.rs`

**Interfaces:**
- Produces: `Agent.{spice: f64, initial_spice: f64, spice_metabolism: u32, foresight: u32, income: f64}`; `Agent::random` draws spice traits only if `spice.enabled` and foresight only if `foresight.enabled`, **after** all existing draws; `is_fertile` also requires `spice >= initial_spice`; births split spice endowments and inherit `spice_metabolism` / `foresight` from a random parent (drawn only when the rule is on); inheritance splits both goods; `fingerprint()` hashes agent spice when spice is on and foresight when foresight is on.

- [ ] **Step 1: Failing tests**

`agent.rs` tests:
```rust
    #[test]
    fn spice_and_foresight_are_drawn_only_when_enabled() {
        use crate::config::{Config, URange};
        use crate::rng::seeded;
        let mut off = Config::default();
        off.spice.metabolism = URange::new(3, 3);
        let a = Agent::random(&off, Pos::new(0, 0), 0, &mut seeded(4));
        let b = Agent::random(&Config::default(), Pos::new(0, 0), 0, &mut seeded(4));
        assert_eq!(a, b, "spice parameters don't matter while spice is off");
        assert_eq!((a.spice, a.spice_metabolism, a.foresight), (0.0, 0, 0));
        let mut on = Config::default();
        on.spice.enabled = true;
        on.foresight.enabled = true;
        let c = Agent::random(&on, Pos::new(0, 0), 0, &mut seeded(4));
        assert!((1..=4).contains(&c.spice_metabolism));
        assert!((5.0..=25.0).contains(&c.spice) && c.spice == c.initial_spice);
        assert!(c.foresight <= 10);
        assert_eq!((c.vision, c.metabolism, c.sugar), (b.vision, b.metabolism, b.sugar),
            "new draws come after the existing ones");
    }
```
`sex.rs` tests:
```rust
    #[test]
    fn with_spice_fertility_needs_both_goods_and_children_get_both() {
        let mut w = blank_world(10, 10);
        w.config.spice.enabled = true;
        let (mom, dad) = couple(&mut w);
        for id in [mom, dad] {
            let a = w.agent_mut(id).unwrap();
            a.spice = 8.0;
            a.initial_spice = 8.0;
            a.spice_metabolism = if id == mom { 2 } else { 3 };
        }
        w.agent_mut(dad).unwrap().spice = 7.0; // below its spice endowment
        act(&mut w, mom);
        assert_eq!(w.population(), 2, "infertile without enough spice");
        w.agent_mut(dad).unwrap().spice = 8.0;
        act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap().clone();
        assert_eq!((child.spice, child.initial_spice), (8.0, 8.0));
        assert!([2, 3].contains(&child.spice_metabolism));
        assert_eq!(w.agent(mom).unwrap().spice, 4.0);
    }
```
`lifecycle.rs` tests (inheritance lives in `world.rs` `kill`):
```rust
    #[test]
    fn inheritance_splits_spice_too() {
        let mut w = blank_world(5, 5);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 0, 0);
        let child = spawn(&mut w, 1, 0);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.agent_mut(parent).unwrap().spice = 6.0;
        w.kill(parent, DeathCause::OldAge);
        assert_eq!(w.agent(child).unwrap().spice, 16.0);
    }
```

- [ ] **Step 2: Run** — compile failures (new fields).

- [ ] **Step 3: Implement**

`agent.rs` — add to `Agent` (after `born`):
```rust
    /// Spice holdings and birth endowment (0 while spice is off).
    pub spice: f64,
    pub initial_spice: f64,
    pub spice_metabolism: u32,
    /// Book eq. 6's φ (0 while foresight is off).
    pub foresight: u32,
    /// Sugar gathered minus sugar metabolism minus per-tick loan obligations,
    /// this turn (credit's creditworthiness input).
    pub income: f64,
```
`Agent::random`: build the struct exactly as today with the new fields zeroed, then:
```rust
        let mut agent = Self { /* existing fields … */, spice: 0.0, initial_spice: 0.0, spice_metabolism: 0, foresight: 0, income: 0.0 };
        if config.spice.enabled {
            let spice = f64::from(config.spice.endowment.sample(rng));
            agent.spice = spice;
            agent.initial_spice = spice;
            agent.spice_metabolism = config.spice.metabolism.sample(rng);
        }
        if config.foresight.enabled {
            agent.foresight = config.foresight.range.sample(rng);
        }
        agent
```
`is_fertile`: `… && self.sugar >= self.initial_sugar && self.spice >= self.initial_spice`.

`testkit::spawn`: add `spice: 10.0, initial_spice: 10.0, spice_metabolism: 0, foresight: 0, income: 0.0`.

`sex.rs` `birth`: add the new fields to the child literal as `spice: from_a_spice + from_b_spice, initial_spice: from_a_spice + from_b_spice, spice_metabolism: 0, foresight: 0, income: 0.0`, where `let (from_a_spice, from_b_spice) = (a.initial_spice / 2.0, b.initial_spice / 2.0);`. After the literal (still holding `rng`):
```rust
    if world.config.spice.enabled {
        child.spice_metabolism = pick(rng, a.spice_metabolism, b.spice_metabolism);
    }
    if world.config.foresight.enabled {
        child.foresight = pick(rng, a.foresight, b.foresight);
    }
```
(make `child` `let mut`), and deduct spice from the parents alongside sugar:
```rust
    let pa = world.agent_mut(a_id).expect("parent");
    pa.sugar -= from_a;
    pa.spice -= from_a_spice;
    let pb = world.agent_mut(b_id).expect("parent");
    pb.sugar -= from_b;
    pb.spice -= from_b_spice;
```

`world.rs` `bequeath` — split each good independently:
```rust
    fn bequeath(&mut self, agent: &Agent) {
        let heirs: Vec<AgentId> = agent
            .children
            .iter()
            .copied()
            .filter(|c| self.agents.contains_key(c))
            .collect();
        let n = heirs.len() as f64;
        let sugar = if agent.sugar > 0.0 { agent.sugar / n } else { 0.0 };
        let spice = if agent.spice > 0.0 { agent.spice / n } else { 0.0 };
        if heirs.is_empty() || (sugar == 0.0 && spice == 0.0) {
            return;
        }
        for heir in heirs {
            let h = self.agents.get_mut(&heir).expect("living heir");
            h.sugar += sugar;
            h.spice += spice;
        }
    }
```
`fingerprint()` agent loop, after `eat(a.tags.bits())`:
```rust
            if spice {
                eat(a.spice.to_bits());
            }
            if foresight {
                eat(u64::from(a.foresight));
            }
```
with `let (spice, foresight) = (self.config.spice.enabled, self.config.foresight.enabled);` before the closure.

Update any other `Agent { … }` literals the compiler flags (tests in `render.rs`/`edit.rs` use `spawn`).

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged: no new draws or hashing while off).
- [ ] **Step 5: Commit** — `git commit -m "Give agents spice, foresight and income"`.

---

### Task 6: Welfare and valuation functions

**Files:**
- Create: `crates/sugarscape-core/src/econ.rs`
- Modify: `src/lib.rs` (`pub mod econ;`)

**Interfaces:**
- Produces: `econ::{welfare(w1, w2, m1, m2) -> f64, foresight_welfare(w1, w2, m1, m2, phi) -> f64, mrs(w1, w2, m1, m2) -> f64, sugar_demand(p, w1, w2, m1, m2) -> f64}` (all `f64` arguments).

- [ ] **Step 1: Failing tests**

`econ.rs`:
```rust
//! Chapter IV's Cobb–Douglas welfare and the valuations derived from it.
//! Metabolisms are weights; with both zero the agent weighs goods equally.

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn welfare_is_cobb_douglas_in_metabolism_weights() {
        assert!(close(welfare(4.0, 9.0, 1.0, 1.0), 6.0));
        assert!(close(welfare(8.0, 1.0, 3.0, 0.0), 8.0));
        assert!(close(welfare(4.0, 9.0, 0.0, 0.0), 6.0));
        assert_eq!(welfare(0.0, 9.0, 1.0, 1.0), 0.0);
    }

    #[test]
    fn foresight_subtracts_future_metabolism_and_floors_at_zero() {
        assert!(close(foresight_welfare(10.0, 10.0, 1.0, 1.0, 2), welfare(8.0, 8.0, 1.0, 1.0)));
        assert_eq!(foresight_welfare(5.0, 50.0, 1.0, 1.0, 10), 0.0);
    }

    #[test]
    fn mrs_is_spice_per_sugar() {
        assert!(close(mrs(10.0, 20.0, 1.0, 1.0), 2.0));
        assert!(close(mrs(10.0, 20.0, 2.0, 1.0), 4.0));
        assert_eq!(mrs(10.0, 20.0, 0.0, 1.0), 0.0);
        assert!(mrs(10.0, 20.0, 1.0, 0.0).is_infinite());
    }

    #[test]
    fn sugar_demand_is_the_cobb_douglas_share_of_wealth() {
        // Wealth at p = 1 is 30 spice-units; half of it is spent on sugar.
        assert!(close(sugar_demand(1.0, 10.0, 20.0, 1.0, 1.0), 15.0));
        assert!(close(sugar_demand(2.0, 10.0, 20.0, 1.0, 1.0), 10.0));
    }
}
```

- [ ] **Step 2: Run** — compile failure.

- [ ] **Step 3: Implement** (above the tests)
```rust
/// Exponents m₁/m_T and m₂/m_T (½, ½ when both metabolisms are zero).
fn weights(m1: f64, m2: f64) -> (f64, f64) {
    let mt = m1 + m2;
    if mt > 0.0 { (m1 / mt, m2 / mt) } else { (0.5, 0.5) }
}

/// Book eq. 1: W = w₁^(m₁/m_T) · w₂^(m₂/m_T); negative holdings count as 0.
pub fn welfare(w1: f64, w2: f64, m1: f64, m2: f64) -> f64 {
    let (a, b) = weights(m1, m2);
    w1.max(0.0).powf(a) * w2.max(0.0).powf(b)
}

/// Book eq. 6: welfare as if `phi` periods of metabolism were already spent.
pub fn foresight_welfare(w1: f64, w2: f64, m1: f64, m2: f64, phi: u32) -> f64 {
    let phi = f64::from(phi);
    welfare(w1 - phi * m1, w2 - phi * m2, m1, m2)
}

/// Book eq. 3: MRS = (w₂/m₂)/(w₁/m₁), the spice value of one unit of sugar.
pub fn mrs(w1: f64, w2: f64, m1: f64, m2: f64) -> f64 {
    if m1 == 0.0 && m2 == 0.0 {
        return w2 / w1;
    }
    (w2 * m1) / (m2 * w1)
}

/// Sugar an agent would hold at price `p` (spice per sugar) if it could
/// re-trade its whole bundle: the Cobb–Douglas share of its wealth.
pub fn sugar_demand(p: f64, w1: f64, w2: f64, m1: f64, m2: f64) -> f64 {
    let (a, _) = weights(m1, m2);
    a * (p * w1 + w2) / p
}
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core econ`.
- [ ] **Step 5: Commit** — `git commit -m "Add Chapter IV welfare and valuation functions"`.

---

### Task 7: Multicommodity movement, metabolism and death

**Files:**
- Modify: `src/rules/mod.rs`, `src/rules/movement.rs`, `src/rules/combat.rs`, `src/rules/lifecycle.rs`

**Interfaces:**
- Produces: `rules::Harvest { sugar: f64, spice: f64 }` (`Clone, Copy, Debug, Default, PartialEq`); `movement::act -> Harvest`; `combat::act -> Harvest` (spice 0); `lifecycle::metabolize(world, id, Harvest)`.

- [ ] **Step 1: Failing tests** (append to `movement.rs` tests)

```rust
    fn spicy(w: &mut World, vision: u32) -> AgentId {
        w.config.spice.enabled = true;
        let id = mover(w, vision);
        let a = w.agent_mut(id).unwrap();
        a.metabolism = 1;
        a.spice_metabolism = 1;
        id
    }

    #[test]
    fn with_spice_agents_seek_the_good_they_lack() {
        let mut w = blank_world(11, 11);
        let id = spicy(&mut w, 3);
        w.agent_mut(id).unwrap().sugar = 30.0;
        w.agent_mut(id).unwrap().spice = 2.0;
        set_sugar(&mut w, 5, 7, 4.0);
        w.site_mut(Pos::new(7, 5)).spice = 2.0;
        let h = act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(7, 5), "spice-poor agent picks spice");
        assert_eq!(h, crate::rules::Harvest { sugar: 0.0, spice: 2.0 });
        assert_eq!(w.agent(id).unwrap().spice, 4.0);
    }

    #[test]
    fn with_spice_both_goods_are_gathered() {
        let mut w = blank_world(11, 11);
        let id = spicy(&mut w, 1);
        set_sugar(&mut w, 5, 6, 2.0);
        w.site_mut(Pos::new(5, 6)).spice = 3.0;
        let h = act(&mut w, id);
        assert_eq!((h.sugar, h.spice), (2.0, 3.0));
        assert_eq!(w.site(Pos::new(5, 6)).spice, 0.0);
    }
```
Append to `lifecycle.rs` tests:
```rust
    #[test]
    fn with_spice_agents_burn_both_and_die_of_either() {
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().spice_metabolism = 10;
        metabolize(&mut w, id, crate::rules::Harvest::default());
        assert_eq!(w.agent(id).unwrap().spice, 0.0);
        assert!(check_death(&mut w, id), "spice starvation");
    }

    #[test]
    fn only_sugar_pollutes_unless_spice_pollutes() {
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        w.config.pollution.enabled = true;
        let id = spawn(&mut w, 2, 2);
        metabolize(&mut w, id, crate::rules::Harvest { sugar: 1.0, spice: 4.0 });
        assert_eq!(w.site(crate::geometry::Pos::new(2, 2)).pollution, 1.0);
        w.config.pollution.spice_pollutes = true;
        metabolize(&mut w, id, crate::rules::Harvest { sugar: 1.0, spice: 4.0 });
        assert_eq!(w.site(crate::geometry::Pos::new(2, 2)).pollution, 1.0 + 5.0);
    }
```
Update existing movement/lifecycle/combat tests that use the `f64` return: e.g. `let gathered = act(&mut w, id); assert_eq!(gathered, 3.0)` → `assert_eq!(act(&mut w, id).sugar, 3.0)`; `metabolize(&mut w, id, 4.0)` → `metabolize(&mut w, id, Harvest { sugar: 4.0, spice: 0.0 })`.

- [ ] **Step 2: Run** — compile failure (`Harvest`).

- [ ] **Step 3: Implement**

`rules/mod.rs` — add:
```rust
/// Resources an agent collected from its site this turn.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Harvest {
    pub sugar: f64,
    pub spice: f64,
}
```
and in `agent_turn` rename `gathered` to `harvest`.

`movement.rs` — keep the single-good code path byte-for-byte for spice off; branch at the top:
```rust
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    if world.config.spice.enabled {
        return act_two_goods(world, id);
    }
    // … existing body …
    Harvest { sugar: gathered, spice: 0.0 }
}

/// Multicommodity M: maximize (foresight) welfare after gathering. Pollution
/// discounts a site's sugar (and spice, if it pollutes) by 1/(1 + p).
fn act_two_goods(world: &mut World, id: AgentId) -> Harvest {
    let a = world.agent(id).expect("live agent");
    let (pos, vision, phi) = (a.pos, a.vision, a.foresight);
    let (w1, w2) = (a.sugar, a.spice);
    let (m1, m2) = (f64::from(a.metabolism), f64::from(a.spice_metabolism));
    let pollution = world.config.pollution;
    let value = |w: &World, p: Pos| {
        let s = w.site(p);
        let discount = if pollution.enabled { 1.0 / (1.0 + s.pollution) } else { 1.0 };
        let x1 = s.sugar * discount;
        let x2 = if pollution.spice_pollutes { s.spice * discount } else { s.spice };
        econ::foresight_welfare(w1 + x1, w2 + x2, m1, m2, phi)
    };
    let mut candidates = vec![(pos, 0, value(world, pos))];
    for (q, d) in world.torus.sight(pos, vision) {
        if !world.is_occupied(q) {
            candidates.push((q, d, value(world, q)));
        }
    }
    let target = choose(&candidates, &mut world.rng);
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let harvest = Harvest { sugar: site.sugar, spice: site.spice };
    site.sugar = 0.0;
    site.spice = 0.0;
    let a = world.agent_mut(id).expect("live agent");
    a.sugar += harvest.sugar;
    a.spice += harvest.spice;
    harvest
}
```
(imports: `crate::econ`, `crate::rules::Harvest`.)

`combat.rs`: return `Harvest { sugar: gathered, spice: 0.0 }`.

`lifecycle.rs`:
```rust
/// Burns sugar (and spice when it's on). With rule P on, the agent's site
/// gains α·gathered + β·burned — sugar only, unless spice pollutes too.
pub(crate) fn metabolize(world: &mut World, id: AgentId, harvest: Harvest) {
    let spice_on = world.config.spice.enabled;
    let agent = world.agent_mut(id).expect("live agent");
    let burned = f64::from(agent.metabolism);
    agent.sugar -= burned;
    let burned_spice = if spice_on { f64::from(agent.spice_metabolism) } else { 0.0 };
    if spice_on {
        agent.spice -= burned_spice;
    }
    let pos = agent.pos;
    let p = world.config.pollution;
    if p.enabled {
        let (gathered, burned) = if p.spice_pollutes {
            (harvest.sugar + harvest.spice, burned + burned_spice)
        } else {
            (harvest.sugar, burned)
        };
        world.site_mut(pos).pollution += p.production * gathered + p.consumption * burned;
    }
}
```
**Keep the spice-off pollution arithmetic identical** — `p.production * harvest.sugar + p.consumption * burned` in the same order as before (the expression above reduces to it when `spice_pollutes` is false).

`check_death`: starvation when `agent.sugar <= 0.0 || (world.config.spice.enabled && agent.spice <= 0.0)`.

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged).
- [ ] **Step 5: Commit** — `git commit -m "Add multicommodity movement and spice metabolism"`.

---

### Task 8: Trade (T)

**Files:**
- Create: `src/rules/trade.rs`
- Modify: `src/rules/mod.rs`, `src/world.rs`

**Interfaces:**
- Produces: `world::Trade { buyer: AgentId, seller: AgentId, price: f64, sugar: f64 }` (Copy, Debug, PartialEq); `TickEvents.trades: Vec<Trade>`; `trade::act(world, id)`; `trade::trade_pair(world, a, b)` (pub(crate)); `agent_turn` calls `trade::act` after culture when `trade.enabled`.

- [ ] **Step 1: Failing tests** (`trade.rs`)

```rust
//! Agent trade rule T (Chapter IV). With each neighbor, in random order:
//! while their MRSs differ, the higher-MRS agent buys sugar with spice at
//! p = √(MRS_A·MRS_B) — 1 sugar for p spice if p ≥ 1, else 1/p sugar for 1
//! spice — as long as both agents' welfare strictly rises, their MRSs don't
//! cross, and both keep positive holdings. Each exchange is one trade.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::econ::{mrs, welfare};
use crate::world::{Trade, World};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn trader(w: &mut World, x: u32, sugar: f64, spice: f64) -> AgentId {
        let id = spawn(w, x, 0);
        let a = w.agent_mut(id).unwrap();
        a.sugar = sugar;
        a.spice = spice;
        a.metabolism = 1;
        a.spice_metabolism = 1;
        id
    }

    fn state(w: &World, id: AgentId) -> (f64, f64, f64, f64) {
        let a = w.agent(id).unwrap();
        let (m1, m2) = (f64::from(a.metabolism), f64::from(a.spice_metabolism));
        (a.sugar, a.spice, welfare(a.sugar, a.spice, m1, m2), mrs(a.sugar, a.spice, m1, m2))
    }

    #[test]
    fn edgeworth_box_example_converges_without_crossing() {
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        let a = trader(&mut w, 0, 5.0, 8.0);
        let b = trader(&mut w, 1, 15.0, 2.0);
        let (_, _, wa0, _) = state(&w, a);
        let (_, _, wb0, _) = state(&w, b);
        trade_pair(&mut w, a, b);
        let (sa, pa, wa, ma) = state(&w, a);
        let (sb, pb, wb, mb) = state(&w, b);
        assert!(wa > wa0 && wb > wb0, "both better off");
        assert!(ma >= mb, "MRSs did not cross (A valued sugar more)");
        assert!((sa + sb - 20.0).abs() < 1e-9 && (pa + pb - 10.0).abs() < 1e-9, "goods conserved");
        assert!(!w.events().trades.is_empty());
        let first = w.events().trades[0];
        assert_eq!((first.buyer, first.seller), (a, b), "A buys sugar");
        assert!((first.price - (1.6f64 * (2.0 / 15.0)).sqrt()).abs() < 1e-12);
    }

    #[test]
    fn equal_valuations_do_not_trade() {
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        let a = trader(&mut w, 0, 10.0, 10.0);
        let b = trader(&mut w, 1, 20.0, 20.0);
        trade_pair(&mut w, a, b);
        assert!(w.events().trades.is_empty());
    }

    #[test]
    fn act_trades_with_neighbors_only() {
        let mut w = blank_world(10, 10);
        w.config.spice.enabled = true;
        let a = trader(&mut w, 0, 5.0, 8.0);
        trader(&mut w, 5, 15.0, 2.0); // not adjacent
        act(&mut w, a);
        assert!(w.events().trades.is_empty());
    }
}
```

- [ ] **Step 2: Run** — compile failure.

- [ ] **Step 3: Implement** (above the tests)

`world.rs`: add
```rust
/// One exchange under rule T: `buyer` received `sugar` sugar and paid
/// `sugar × price` spice to `seller`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trade {
    pub buyer: AgentId,
    pub seller: AgentId,
    pub price: f64,
    pub sugar: f64,
}
```
and `pub trades: Vec<Trade>,` in `TickEvents`.

`trade.rs`:
```rust
/// Safety bound on exchanges per pair per turn (welfare strictly rises each
/// time, so the loop ends long before this in practice).
const MAX_EXCHANGES: usize = 10_000;

pub(crate) fn act(world: &mut World, id: AgentId) {
    let pos = world.agent(id).expect("live agent").pos;
    let mut neighbors = world.torus.neighbors(pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        if let Some(other) = world.occupant(q) {
            trade_pair(world, id, other);
        }
    }
}

struct Holdings {
    sugar: f64,
    spice: f64,
    m1: f64,
    m2: f64,
}

impl Holdings {
    fn of(world: &World, id: AgentId) -> Self {
        let a = world.agent(id).expect("live agent");
        Self {
            sugar: a.sugar,
            spice: a.spice,
            m1: f64::from(a.metabolism),
            m2: f64::from(a.spice_metabolism),
        }
    }
    fn welfare(&self, sugar: f64, spice: f64) -> f64 {
        welfare(sugar, spice, self.m1, self.m2)
    }
    fn mrs(&self, sugar: f64, spice: f64) -> f64 {
        mrs(sugar, spice, self.m1, self.m2)
    }
}

pub(crate) fn trade_pair(world: &mut World, a: AgentId, b: AgentId) {
    for _ in 0..MAX_EXCHANGES {
        let (ha, hb) = (Holdings::of(world, a), Holdings::of(world, b));
        let (ma, mb) = (ha.mrs(ha.sugar, ha.spice), hb.mrs(hb.sugar, hb.spice));
        if !(ma.is_finite() && mb.is_finite() && ma > 0.0 && mb > 0.0) || ma == mb {
            return;
        }
        let p = (ma * mb).sqrt();
        let ((buyer, hbuy), (seller, hsell)) = if ma > mb { ((a, ha), (b, hb)) } else { ((b, hb), (a, ha)) };
        let (sugar, spice) = if p >= 1.0 { (1.0, p) } else { (1.0 / p, 1.0) };
        let buy = (hbuy.sugar + sugar, hbuy.spice - spice);
        let sell = (hsell.sugar - sugar, hsell.spice + spice);
        let positive = buy.0 > 0.0 && buy.1 > 0.0 && sell.0 > 0.0 && sell.1 > 0.0;
        let better = hbuy.welfare(buy.0, buy.1) > hbuy.welfare(hbuy.sugar, hbuy.spice)
            && hsell.welfare(sell.0, sell.1) > hsell.welfare(hsell.sugar, hsell.spice);
        let no_cross = hbuy.mrs(buy.0, buy.1) >= hsell.mrs(sell.0, sell.1);
        if !(positive && better && no_cross) {
            return;
        }
        let x = world.agent_mut(buyer).expect("buyer");
        x.sugar = buy.0;
        x.spice = buy.1;
        let y = world.agent_mut(seller).expect("seller");
        y.sugar = sell.0;
        y.spice = sell.1;
        world.events.trades.push(Trade { buyer, seller, price: p, sugar });
    }
}
```
`rules/mod.rs`: `pub mod trade;` and, after the culture block in `agent_turn`:
```rust
    if world.config.trade.enabled {
        trade::act(world, id);
    }
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged).
- [ ] **Step 5: Commit** — `git commit -m "Add trade rule T"`.

---

### Task 9: Credit (L)

**Files:**
- Create: `src/rules/credit.rs`
- Modify: `src/rules/mod.rs`, `src/world.rs`

**Interfaces:**
- Produces: `world::{LoanId = u64, Loan { id, lender, borrower, principal: f64, due: f64, due_tick: u64 }}` (Clone, Copy, Debug, PartialEq, Serialize); `World::loans() -> impl Iterator<Item = &Loan>`; `pub(crate) World::originate_loan(lender, borrower, principal) -> LoanId`; `TickEvents.{loans_made: u32, amount_lent: f64, defaults: u32}`; `credit::{lendable(&Agent) -> f64, record_income(world, id, sugar_gathered), borrow(world, id), settle(world)}`; loans resolved on `remove`/`kill`.

- [ ] **Step 1: Failing tests** (`credit.rs`)

```rust
//! Agent credit rule L_{d,r} (Chapter IV), in the book's single-commodity
//! (sugar) form. Lenders: agents too old to have children lend up to half their
//! sugar; fertile agents lend sugar above their birth endowment. Borrowers:
//! fertile agents short of their endowment with positive income this turn.
//! A loan of P is creditworthy when income × d ≥ P × (1 + r/100 × d)
//! (interpretation of the book's "credit-worthy for a loan written at terms
//! specified by the lender"). At the due tick the borrower pays in full, or
//! pays half its sugar and the remainder is re-lent on the same terms (a
//! default). A dead borrower's loans are the lender's loss; a dead lender's
//! loans are cancelled unless inheritance (I) is on, when its living children
//! split the claim.

use rand::seq::SliceRandom;

use crate::agent::{Agent, AgentId};
use crate::world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;
    use crate::world::DeathCause;

    fn credit_world() -> World {
        let mut w = blank_world(5, 5);
        w.config.sex.enabled = true;
        w.config.credit.enabled = true;
        w.config.credit.duration = 10;
        w.config.credit.rate = 10.0;
        w
    }

    /// A fertile borrower short of its endowment (sugar 4 of 10) with income 5.
    fn borrower(w: &mut World) -> AgentId {
        let id = spawn(w, 2, 2);
        let a = w.agent_mut(id).unwrap();
        a.sugar = 4.0;
        a.income = 5.0;
        id
    }

    /// An old lender (past fertility) holding 40 sugar.
    fn lender(w: &mut World) -> AgentId {
        let id = spawn(w, 2, 1);
        let a = w.agent_mut(id).unwrap();
        a.age = 70;
        a.sugar = 40.0;
        id
    }

    #[test]
    fn lendable_amounts_follow_the_book() {
        let mut w = credit_world();
        let old = lender(&mut w);
        assert_eq!(lendable(w.agent(old).unwrap()), 20.0);
        let young = spawn(&mut w, 0, 0); // fertile, sugar 10 = endowment 10
        w.agent_mut(young).unwrap().sugar = 16.0;
        assert_eq!(lendable(w.agent(young).unwrap()), 6.0);
        w.agent_mut(young).unwrap().age = 5;
        assert_eq!(lendable(w.agent(young).unwrap()), 0.0, "too young to lend");
    }

    #[test]
    fn borrowing_fills_the_need_from_neighbors() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        assert_eq!(w.agent(b).unwrap().sugar, 10.0, "borrowed its 6 shortfall");
        assert_eq!(w.agent(l).unwrap().sugar, 34.0);
        let loan = *w.loans().next().unwrap();
        assert_eq!((loan.lender, loan.borrower, loan.principal), (l, b, 6.0));
        assert!((loan.due - 12.0).abs() < 1e-12, "6 × (1 + 0.1 × 10)");
        assert_eq!(loan.due_tick, w.tick + 10);
        assert_eq!((w.events().loans_made, w.events().amount_lent), (1, 6.0));
    }

    #[test]
    fn creditworthiness_caps_the_principal() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        w.agent_mut(b).unwrap().income = 0.6; // 0.6 × 10 / 2 = 3 at most
        lender(&mut w);
        borrow(&mut w, b);
        assert!((w.agent(b).unwrap().sugar - 7.0).abs() < 1e-12);
    }

    #[test]
    fn settlement_repays_or_rolls_over() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        w.tick += 10;
        w.agent_mut(b).unwrap().sugar = 20.0;
        settle(&mut w);
        assert_eq!(w.agent(b).unwrap().sugar, 8.0);
        assert_eq!(w.agent(l).unwrap().sugar, 46.0);
        assert_eq!(w.loans().count(), 0);

        let b2 = spawn(&mut w, 0, 0);
        w.agent_mut(b2).unwrap().sugar = 4.0;
        let id = w.originate_loan(l, b2, 6.0);
        let due_tick = w.loans().find(|x| x.id == id).unwrap().due_tick;
        w.tick = due_tick;
        settle(&mut w);
        assert_eq!(w.agent(b2).unwrap().sugar, 2.0, "paid half");
        let rolled = w.loans().next().unwrap();
        assert!((rolled.principal - 10.0).abs() < 1e-12, "12 due − 2 paid");
        assert_eq!(rolled.due_tick, due_tick + 10);
        assert_eq!(w.events().defaults, 1);
    }

    #[test]
    fn deaths_resolve_loans() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        borrow(&mut w, b);
        w.kill(b, DeathCause::Starvation);
        assert_eq!(w.loans().count(), 0, "lender takes the loss");

        let b = borrower(&mut w);
        borrow(&mut w, b);
        let child = spawn(&mut w, 4, 4);
        w.agent_mut(l).unwrap().children = vec![child];
        w.config.inheritance.enabled = true;
        w.kill(l, DeathCause::OldAge);
        let loan = w.loans().next().expect("claim passes to the child");
        assert_eq!((loan.lender, loan.borrower), (child, b));
    }

    #[test]
    fn income_is_gathered_minus_metabolism_minus_obligations() {
        let mut w = credit_world();
        let b = borrower(&mut w);
        let l = lender(&mut w);
        w.agent_mut(b).unwrap().metabolism = 2;
        w.originate_loan(l, b, 5.0); // due 10 over 10 ticks → 1 per tick
        record_income(&mut w, b, 6.0);
        assert!((w.agent(b).unwrap().income - 3.0).abs() < 1e-12);
    }
}
```

- [ ] **Step 2: Run** — compile failure.

- [ ] **Step 3: Implement**

`world.rs` — types, storage and helpers:
```rust
pub type LoanId = u64;

/// A sugar loan under rule L: `due` sugar owed at `due_tick`.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Loan {
    pub id: LoanId,
    pub lender: AgentId,
    pub borrower: AgentId,
    pub principal: f64,
    pub due: f64,
    pub due_tick: u64,
}
```
`TickEvents` gains `pub loans_made: u32, pub amount_lent: f64, pub defaults: u32,`. `World` gains `loans: BTreeMap<LoanId, Loan>, next_loan_id: LoanId,` (init `BTreeMap::new()`, `1`). Add:
```rust
    pub fn loans(&self) -> impl Iterator<Item = &Loan> {
        self.loans.values()
    }

    /// Records a loan of `principal` on the current credit terms (no transfer).
    pub(crate) fn originate_loan(&mut self, lender: AgentId, borrower: AgentId, principal: f64) -> LoanId {
        let c = self.config.credit;
        let id = self.next_loan_id;
        self.next_loan_id += 1;
        let factor = 1.0 + c.rate / 100.0 * f64::from(c.duration);
        self.loans.insert(id, Loan {
            id,
            lender,
            borrower,
            principal,
            due: principal * factor,
            due_tick: self.tick + u64::from(c.duration),
        });
        id
    }

    pub(crate) fn remove_loan(&mut self, id: LoanId) -> Option<Loan> {
        self.loans.remove(&id)
    }
```
In `remove()`, after clearing occupancy, cancel every loan involving the agent:
```rust
        if !self.loans.is_empty() {
            self.loans.retain(|_, l| l.lender != id && l.borrower != id);
        }
```
In `kill()`, **before** `self.remove(id)`, capture the lender-side loans for inheritance:
```rust
        let claims: Vec<Loan> = if self.config.inheritance.enabled {
            self.loans.values().filter(|l| l.lender == id).copied().collect()
        } else {
            Vec::new()
        };
```
and after `bequeath`:
```rust
        if !claims.is_empty() {
            self.pass_on_claims(&agent, claims);
        }
```
with
```rust
    /// Splits a dead lender's claims equally among its living children.
    fn pass_on_claims(&mut self, lender: &Agent, claims: Vec<Loan>) {
        let heirs: Vec<AgentId> = lender.children.iter().copied().filter(|c| self.agents.contains_key(c)).collect();
        if heirs.is_empty() {
            return;
        }
        let n = heirs.len() as f64;
        for claim in claims {
            if !self.agents.contains_key(&claim.borrower) {
                continue;
            }
            for &heir in &heirs {
                let id = self.next_loan_id;
                self.next_loan_id += 1;
                self.loans.insert(id, Loan {
                    id,
                    lender: heir,
                    principal: claim.principal / n,
                    due: claim.due / n,
                    ..claim
                });
            }
        }
    }
```
`step()` — after the agent-turn loop, before growback:
```rust
        if !self.loans.is_empty() {
            rules::credit::settle(self);
        }
```
`fingerprint()` — after the agent loop:
```rust
        for l in self.loans.values() {
            eat(l.id);
            eat(l.due.to_bits());
            eat(l.due_tick);
        }
```
(no loans exist while credit is off, so milestone-1 hashes are unchanged).

`credit.rs` (above the tests):
```rust
fn fertile_age(a: &Agent) -> bool {
    (a.fertility_onset..=a.fertility_end).contains(&a.age)
}

/// How much sugar `a` may lend now.
pub(crate) fn lendable(a: &Agent) -> f64 {
    if a.age > a.fertility_end {
        (a.sugar / 2.0).max(0.0)
    } else if fertile_age(a) && a.sugar > a.initial_sugar {
        a.sugar - a.initial_sugar
    } else {
        0.0
    }
}

fn obligations(world: &World, id: AgentId) -> f64 {
    let d = f64::from(world.config.credit.duration);
    world.loans().filter(|l| l.borrower == id).map(|l| l.due / d).sum()
}

/// Sets `income` = sugar gathered − sugar metabolism − per-tick obligations.
pub(crate) fn record_income(world: &mut World, id: AgentId, sugar_gathered: f64) {
    let owed = obligations(world, id);
    let a = world.agent_mut(id).expect("live agent");
    a.income = sugar_gathered - f64::from(a.metabolism) - owed;
}

/// A borrower asks its neighbors, in random order, for its shortfall.
pub(crate) fn borrow(world: &mut World, id: AgentId) {
    let c = world.config.credit;
    let me = world.agent(id).expect("live agent");
    if !(fertile_age(me) && me.sugar < me.initial_sugar && me.income > 0.0) {
        return;
    }
    let d = f64::from(c.duration);
    let mut need = me.initial_sugar - me.sugar;
    let mut capacity = me.income * d / (1.0 + c.rate / 100.0 * d);
    let mut neighbors = world.torus.neighbors(me.pos);
    neighbors.shuffle(&mut world.rng);
    for q in neighbors {
        if need <= 0.0 || capacity <= 0.0 {
            break;
        }
        let Some(lender) = world.occupant(q) else { continue };
        let amount = lendable(world.agent(lender).expect("occupant")).min(need).min(capacity);
        if amount <= 0.0 {
            continue;
        }
        world.agent_mut(lender).expect("lender").sugar -= amount;
        world.agent_mut(id).expect("borrower").sugar += amount;
        world.originate_loan(lender, id, amount);
        world.events.loans_made += 1;
        world.events.amount_lent += amount;
        need -= amount;
        capacity -= amount;
    }
}

/// Settles loans due this tick, oldest first.
pub(crate) fn settle(world: &mut World) {
    let due: Vec<_> = world.loans().filter(|l| l.due_tick == world.tick).map(|l| l.id).collect();
    for loan_id in due {
        let loan = world.remove_loan(loan_id).expect("due loan");
        let available = world.agent(loan.borrower).expect("borrowers' loans die with them").sugar;
        // Paying in full must leave the borrower alive (sugar > 0); otherwise
        // it pays half and the rest rolls over.
        let paid = if available > loan.due { loan.due } else { available / 2.0 };
        world.agent_mut(loan.borrower).expect("borrower").sugar -= paid;
        world.agent_mut(loan.lender).expect("lenders' loans die with them").sugar += paid;
        if paid < loan.due {
            world.originate_loan(loan.lender, loan.borrower, loan.due - paid);
            world.events.defaults += 1;
        }
    }
}
```
`rules/mod.rs`: `pub mod credit;`; in `agent_turn`, right after `metabolize`:
```rust
    if world.config.credit.enabled {
        credit::record_income(world, id, harvest.sugar);
    }
```
and after the trade block:
```rust
    if world.config.credit.enabled {
        credit::borrow(world, id);
    }
```

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core` (golden unchanged).
- [ ] **Step 5: Commit** — `git commit -m "Add credit rule L"`.

---

### Task 10: Statistics, supply & demand, and networks

**Files:**
- Modify: `src/stats.rs`
- Create: `src/network.rs` (+ `pub mod network;` in `lib.rs`)

**Interfaces:**
- Produces: `SERIES` extended (append, in order): `mean_log_price, sd_log_price, trade_volume, sugar_traded, loans_made, amount_lent, defaults, debt_outstanding, mean_foresight, mean_spice, mean_spice_metabolism` (`SERIES: [&str; 19]`); matching `Snapshot` fields and `value()` arms; `stats::{SupplyDemand { prices, demand, supply: Vec<f64>, equilibrium_price, equilibrium_quantity, actual_price, actual_quantity: f64 }, supply_demand(&World) -> SupplyDemand}`; `network::{CreditRole::{None, Lender, Borrower, Both}, credit_roles(&World) -> BTreeMap<AgentId, CreditRole>, trade_edges(&World) -> Vec<(Pos, Pos)>, credit_edges(&World) -> Vec<(Pos, Pos)>}`.

- [ ] **Step 1: Failing tests**

`stats.rs` tests:
```rust
    #[test]
    fn trade_and_credit_series_come_from_the_tick_events() {
        use crate::testkit::*;
        use crate::world::Trade;
        let mut w = blank_world(5, 5);
        w.events.trades = vec![
            Trade { buyer: 1, seller: 2, price: 2.0, sugar: 1.0 },
            Trade { buyer: 1, seller: 2, price: 0.5, sugar: 2.0 },
        ];
        w.events.loans_made = 3;
        let s = Snapshot::of(&w);
        assert!(s.mean_log_price.abs() < 1e-12, "ln 2 and ln ½ average to 0");
        assert!((s.sd_log_price - 2f64.ln()).abs() < 1e-12);
        assert_eq!((s.trade_volume, s.sugar_traded, s.loans_made), (2, 3.0, 3));
        for name in SERIES {
            assert!(s.value(name).is_some(), "{name}");
        }
    }

    #[test]
    fn symmetric_market_clears_near_one() {
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.spice.enabled = true;
        for (x, sugar, spice) in [(0, 30.0, 10.0), (1, 10.0, 30.0)] {
            let id = spawn(&mut w, x, 0);
            let a = w.agent_mut(id).unwrap();
            (a.sugar, a.spice, a.metabolism, a.spice_metabolism) = (sugar, spice, 1, 1);
        }
        let sd = supply_demand(&w);
        assert_eq!(sd.prices.len(), 41);
        assert!((sd.equilibrium_price - 1.0).abs() < 0.05, "{}", sd.equilibrium_price);
        assert!((sd.equilibrium_quantity - 10.0).abs() < 0.5);
        assert!(sd.actual_price.is_nan(), "no trades this tick");
    }
```
`network.rs` tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;
    use crate::world::Trade;

    #[test]
    fn edges_and_roles_follow_trades_and_loans() {
        let mut w = blank_world(5, 5);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        let c = spawn(&mut w, 2, 0);
        w.events.trades = vec![
            Trade { buyer: a, seller: b, price: 1.0, sugar: 1.0 },
            Trade { buyer: b, seller: a, price: 1.0, sugar: 1.0 },
        ];
        assert_eq!(trade_edges(&w), vec![(Pos::new(0, 0), Pos::new(1, 0))], "deduplicated");
        w.originate_loan(a, b, 1.0);
        w.originate_loan(b, c, 1.0);
        let roles = credit_roles(&w);
        assert_eq!(roles[&a], CreditRole::Lender);
        assert_eq!(roles[&b], CreditRole::Both);
        assert_eq!(roles[&c], CreditRole::Borrower);
        assert_eq!(credit_edges(&w).len(), 2);
    }
}
```

- [ ] **Step 2: Run** — compile failure.

- [ ] **Step 3: Implement**

`stats.rs`: extend `SERIES` (append the 11 names above), add the 11 fields to `Snapshot` (`trade_volume`, `loans_made`, `defaults` as `u32`; the rest `f64`), compute them in `Snapshot::of`:
```rust
        let events = world.events();
        let logs: Vec<f64> = events.trades.iter().map(|t| t.price.ln()).collect();
        let (mean_log_price, sd_log_price) = if logs.is_empty() {
            (0.0, 0.0)
        } else {
            let m = logs.iter().sum::<f64>() / logs.len() as f64;
            let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / logs.len() as f64;
            (m, var.sqrt())
        };
```
and set `trade_volume: events.trades.len() as u32`, `sugar_traded: events.trades.iter().map(|t| t.sugar).sum()`, `loans_made: events.loans_made`, `amount_lent: events.amount_lent`, `defaults: events.defaults`, `debt_outstanding: world.loans().map(|l| l.due).sum()`, `mean_foresight: mean(&|a| f64::from(a.foresight))`, `mean_spice: mean(&|a| a.spice)`, `mean_spice_metabolism: mean(&|a| f64::from(a.spice_metabolism))`; add the matching `value()` arms (u32 via `f64::from`).

Supply & demand:
```rust
/// Aggregate sugar supply and demand over 41 log-spaced prices in [0.1, 10]
/// (spice per sugar), the interpolated market-clearing point, and the tick's
/// actual geometric-mean price and sugar traded (NaN where undefined).
#[derive(Clone, Debug, PartialEq)]
pub struct SupplyDemand {
    pub prices: Vec<f64>,
    pub demand: Vec<f64>,
    pub supply: Vec<f64>,
    pub equilibrium_price: f64,
    pub equilibrium_quantity: f64,
    pub actual_price: f64,
    pub actual_quantity: f64,
}

pub fn supply_demand(world: &World) -> SupplyDemand {
    let prices: Vec<f64> = (0..41).map(|k| 10f64.powf(-1.0 + k as f64 / 20.0)).collect();
    let mut demand = vec![0.0; prices.len()];
    let mut supply = vec![0.0; prices.len()];
    for a in world.agents() {
        let (m1, m2) = (f64::from(a.metabolism), f64::from(a.spice_metabolism));
        for (k, &p) in prices.iter().enumerate() {
            let excess = crate::econ::sugar_demand(p, a.sugar, a.spice, m1, m2) - a.sugar;
            if excess > 0.0 { demand[k] += excess } else { supply[k] -= excess }
        }
    }
    let (mut equilibrium_price, mut equilibrium_quantity) = (f64::NAN, f64::NAN);
    for k in 0..prices.len() - 1 {
        let (e0, e1) = (demand[k] - supply[k], demand[k + 1] - supply[k + 1]);
        if e0 >= 0.0 && e1 <= 0.0 && e0 != e1 {
            let t = e0 / (e0 - e1);
            equilibrium_price = (prices[k].ln() + t * (prices[k + 1].ln() - prices[k].ln())).exp();
            equilibrium_quantity = demand[k] + t * (demand[k + 1] - demand[k]);
            break;
        }
    }
    let trades = &world.events().trades;
    let (actual_price, actual_quantity) = if trades.is_empty() {
        (f64::NAN, f64::NAN)
    } else {
        let m = trades.iter().map(|t| t.price.ln()).sum::<f64>() / trades.len() as f64;
        (m.exp(), trades.iter().map(|t| t.sugar).sum())
    };
    SupplyDemand { prices, demand, supply, equilibrium_price, equilibrium_quantity, actual_price, actual_quantity }
}
```
(Note `pub(crate) events` is readable from `stats.rs` tests via `w.events` since both are in the crate.)

`network.rs`:
```rust
//! Trade and credit networks (Animations IV-4 and IV-5).

use std::collections::{BTreeMap, BTreeSet};

use crate::agent::AgentId;
use crate::geometry::Pos;
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditRole {
    None,
    Lender,
    Borrower,
    Both,
}

/// Every living agent's role in the outstanding loans.
pub fn credit_roles(world: &World) -> BTreeMap<AgentId, CreditRole> {
    let lenders: BTreeSet<AgentId> = world.loans().map(|l| l.lender).collect();
    let borrowers: BTreeSet<AgentId> = world.loans().map(|l| l.borrower).collect();
    world
        .agents()
        .map(|a| {
            let role = match (lenders.contains(&a.id), borrowers.contains(&a.id)) {
                (true, true) => CreditRole::Both,
                (true, false) => CreditRole::Lender,
                (false, true) => CreditRole::Borrower,
                (false, false) => CreditRole::None,
            };
            (a.id, role)
        })
        .collect()
}

fn edges(world: &World, pairs: impl Iterator<Item = (AgentId, AgentId)>) -> Vec<(Pos, Pos)> {
    let unique: BTreeSet<(AgentId, AgentId)> = pairs.map(|(a, b)| (a.min(b), a.max(b))).collect();
    unique
        .into_iter()
        .filter_map(|(a, b)| Some((world.agent(a)?.pos, world.agent(b)?.pos)))
        .collect()
}

/// Pairs of living agents who traded this tick.
pub fn trade_edges(world: &World) -> Vec<(Pos, Pos)> {
    edges(world, world.events().trades.iter().map(|t| (t.buyer, t.seller)))
}

/// Lender–borrower pairs with an outstanding loan.
pub fn credit_edges(world: &World) -> Vec<(Pos, Pos)> {
    edges(world, world.loans().map(|l| (l.lender, l.borrower)))
}
```
Update `export.rs`'s series-CSV test header expectation to include the 11 new names (append `,mean_log_price,…,mean_spice_metabolism`).

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core`.
- [ ] **Step 5: Commit** — `git commit -m "Add Chapter IV statistics, supply and demand, and networks"`.

---

### Task 11: Rendering spice and credit

**Files:**
- Modify: `src/render.rs`

**Interfaces:**
- Produces: `Layer::{Spice, SpiceCapacity}` (`"spice"`, `"spice_capacity"`); `ColorMode::Credit` (`"credit"`); colors `SPICE, LENDER, BORROWER, BOTH, NEUTRAL`.

- [ ] **Step 1: Failing tests** (append to `render.rs` tests)
```rust
    #[test]
    fn spice_layer_and_credit_colors() {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(1, 1)).spice = 4.0;
        w.site_mut(Pos::new(1, 1)).spice_capacity = 4.0;
        let a = spawn(&mut w, 4, 4);
        let b = spawn(&mut w, 5, 4);
        let c = spawn(&mut w, 6, 4);
        w.originate_loan(a, b, 1.0);
        let mut buf = Vec::new();
        render(&w, ColorMode::Credit, Layer::Spice, &mut buf);
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], SPICE);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], LENDER);
        assert_eq!(pixel(&buf, &w, 5, 4)[..3], BORROWER);
        assert_eq!(pixel(&buf, &w, 6, 4)[..3], NEUTRAL);
        let _ = c;
        assert_eq!("spice_capacity".parse::<Layer>().unwrap(), Layer::SpiceCapacity);
        assert_eq!("credit".parse::<ColorMode>().unwrap(), ColorMode::Credit);
    }
```

- [ ] **Step 2: Run** — compile failure.

- [ ] **Step 3: Implement**

Constants:
```rust
pub const SPICE: Rgb = [0xe0, 0x7a, 0x3f];
pub const LENDER: Rgb = [0x3d, 0xd6, 0x6b];
pub const BORROWER: Rgb = [0xff, 0x4d, 0x4d];
pub const BOTH: Rgb = [0xff, 0xe0, 0x4d];
pub const NEUTRAL: Rgb = [0x8a, 0x86, 0x7a];
```
Add enum variants and `FromStr` arms. In `render`, compute `max_spice_capacity` like `max_capacity`, add layer arms:
```rust
            Layer::Spice => lerp(BACKGROUND, SPICE, site.spice / max_spice_capacity),
            Layer::SpiceCapacity => lerp(BACKGROUND, SPICE, site.spice_capacity / max_spice_capacity),
```
Compute `let roles = if mode == ColorMode::Credit { network::credit_roles(world) } else { BTreeMap::new() };` once, and in the agent loop use:
```rust
        let rgb = if mode == ColorMode::Credit {
            match roles.get(&a.id).copied().unwrap_or(CreditRole::None) {
                CreditRole::Lender => LENDER,
                CreditRole::Borrower => BORROWER,
                CreditRole::Both => BOTH,
                CreditRole::None => NEUTRAL,
            }
        } else {
            agent_color(a, mode, &scales)
        };
```
(`agent_color` gains an unreachable-by-construction `ColorMode::Credit => NEUTRAL` arm to stay exhaustive.)

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core render`.
- [ ] **Step 5: Commit** — `git commit -m "Render spice layers and credit roles"`.

---

### Task 12: Inspection and export of spice, foresight and loans

**Files:**
- Modify: `src/edit.rs`, `src/export.rs`

**Interfaces:**
- Produces: `SiteView.{spice, spice_capacity}`; `AgentView.{spice, initial_spice, spice_metabolism, foresight, loans: Vec<LoanView>}`; `edit::LoanView { id: LoanId, role: &'static str ("lender" | "borrower"), counterparty: LinkView, due: f64, due_tick: u64 }`; agents CSV appends `,spice,initial_spice,spice_metabolism,foresight`.

- [ ] **Step 1: Failing tests**

`edit.rs` tests:
```rust
    #[test]
    fn inspection_shows_spice_foresight_and_loans() {
        let mut w = blank_world(10, 10);
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 1);
        w.agent_mut(a).unwrap().foresight = 3;
        w.originate_loan(a, b, 2.0);
        let view = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!((view.spice, view.foresight), (10.0, 3));
        assert_eq!(view.loans.len(), 1);
        assert_eq!(view.loans[0].role, "lender");
        assert_eq!(view.loans[0].counterparty.id, b);
        let other = w.inspect(2, 1).unwrap().agent.unwrap();
        assert_eq!(other.loans[0].role, "borrower");
        assert_eq!(w.inspect(1, 1).unwrap().site.spice_capacity, 0.0);
    }
```
`export.rs`: update `agents_csv_has_a_row_per_agent` to expect the header `id,x,y,sex,age,max_age,vision,metabolism,sugar,initial_sugar,tribe,tags,spice,initial_spice,spice_metabolism,foresight\n`.

- [ ] **Step 2: Run** — failures.

- [ ] **Step 3: Implement** — add the fields; build `loans` in `inspect`:
```rust
            loans: self
                .loans()
                .filter(|l| l.lender == a.id || l.borrower == a.id)
                .map(|l| {
                    let lender = l.lender == a.id;
                    LoanView {
                        id: l.id,
                        role: if lender { "lender" } else { "borrower" },
                        counterparty: link(if lender { l.borrower } else { l.lender }),
                        due: l.due,
                        due_tick: l.due_tick,
                    }
                })
                .collect(),
```
and append the four columns to `agents_csv` (header and row format `…,{},{},{},{}` with `a.spice, a.initial_spice, a.spice_metabolism, a.foresight`).

- [ ] **Step 4: Verify** — `cargo test -p sugarscape-core`.
- [ ] **Step 5: Commit** — `git commit -m "Inspect and export spice, foresight and loans"`.

---

### Task 13: Presets, invariants and book reproductions

**Files:**
- Modify: `src/presets.rs`, `tests/golden.rs`, `tests/invariants.rs`, `tests/book.rs`

**Interfaces:**
- Produces preset ids `iv-1-spice, iv-3-trade, iv-15-trade-sex, iv-3-pollution, iv-18-foresight, iv-5-credit` (19 presets total); `ii-8-pollution` now starts with pollution and diffusion off and schedules them on at t=50 and t=100.

- [ ] **Step 1: Presets**

In `presets.rs` add helpers and entries (keep existing entries; update `ii-8-pollution`):
```rust
fn schedule(c: &mut Config, tick: u64, path: &str, value: serde_json::Value) {
    c.schedule.push(ScheduledChange { tick, set: [(path.to_string(), value)].into_iter().collect() });
}

/// Chapter IV's neoclassical market: 200 immortal agents, symmetric goods.
fn market(c: &mut Config) {
    c.population = 200;
    c.vision = URange::new(1, 5);
    c.metabolism = URange::new(1, 5);
    c.endowment = URange::new(25, 50);
    c.spice.enabled = true;
    c.spice.metabolism = URange::new(1, 5);
    c.spice.endowment = URange::new(25, 50);
    c.trade.enabled = true;
}
```
`ii-8-pollution` edit closure becomes:
```rust
            |c| {
                schedule(c, 50, "pollution.enabled", serde_json::json!(true));
                schedule(c, 100, "diffusion.enabled", serde_json::json!(true));
            },
```
with description "Gathering and eating pollute from t = 50; diffusion spreads it from t = 100 (scheduled, as in the book)."

New presets (append):
```rust
        preset("iv-1-spice", "({G₁}, {M}) with spice", "Animation IV-1",
            "Two goods on opposite mountains: agents shuttle between sugar and spice to stay alive.",
            |c| {
                c.vision = URange::new(1, 10);
                c.metabolism = URange::new(1, 5);
                c.endowment = URange::new(25, 50);
                c.spice.enabled = true;
                c.spice.metabolism = URange::new(1, 5);
                c.spice.endowment = URange::new(25, 50);
            }),
        preset("iv-3-trade", "({G₁}, {M, T})", "Figures IV-3 to IV-5",
            "Bilateral barter between neighbors: prices converge toward the market-clearing level of 1.",
            market),
        preset("iv-15-trade-sex", "({G₁}, {M, S, T})", "Figure IV-15",
            "Finite lives and evolving preferences keep prices from settling.",
            |c| {
                market(c);
                c.sex.enabled = true;
                c.lifespan.enabled = true;
            }),
        preset("iv-3-pollution", "({G₁, D₁}, {M, T, P})", "Animation IV-3",
            "Sugar becomes a dirty good at t = 100, driving its price up; at t = 150 pollution stops and diffuses away.",
            |c| {
                market(c);
                schedule(c, 100, "pollution.enabled", serde_json::json!(true));
                schedule(c, 150, "pollution.enabled", serde_json::json!(false));
                schedule(c, 150, "diffusion.enabled", serde_json::json!(true));
            }),
        preset("iv-18-foresight", "({G₁}, {M, S}) with foresight", "Figure IV-18",
            "Agents plan φ periods ahead; evolution keeps a modest, non-zero foresight.",
            |c| {
                demography(c);
                c.spice.enabled = true;
                c.spice.endowment = URange::new(50, 100);
                c.foresight.enabled = true;
            }),
        preset("iv-5-credit", "({G₁}, {M, S, L₁₀,₁₀})", "Animation IV-5",
            "Old agents lend to young ones for childbearing; lender–borrower hierarchies emerge.",
            |c| {
                demography(c);
                c.credit.enabled = true;
            }),
```
Two `schedule` calls at the same tick create two entries — both apply at t=150. Update the preset-count test to 19. Imports: `ScheduledChange` from `config`.

- [ ] **Step 2: Golden update for ii-8**

Run `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`, replace only the `ii-8-pollution` value, and add the comment `// Chapter IV: now scheduled (pollution at t=50, diffusion at t=100).` Run `cargo test -p sugarscape-core --test golden` — pass.

- [ ] **Step 3: Invariants**

In `tests/invariants.rs` extend the strategy with booleans `spice, trade, credit, foresight` and constrain after sampling:
```rust
            c.spice.enabled = spice && !combat;
            c.trade.enabled = trade && c.spice.enabled;
            c.foresight.enabled = foresight && c.spice.enabled;
            c.credit.enabled = credit && sex;
            if c.spice.enabled {
                c.spice.endowment = URange::new(25, 50);
            }
```
(the tuple strategy may need nesting — group the booleans into two sub-tuples to stay within proptest's tuple arity). Extend `check`:
```rust
        prop_assert!(site.spice <= site.spice_capacity + 1e-9);
        …
    for a in world.agents() {
        prop_assert!(a.sugar > 0.0);
        if world.config.spice.enabled {
            prop_assert!(a.spice > 0.0, "living agent {} has spice {}", a.id, a.spice);
        }
    }
    for l in world.loans() {
        prop_assert!(world.agent(l.lender).is_some() && world.agent(l.borrower).is_some());
        prop_assert!(l.due > 0.0);
    }
```

- [ ] **Step 4: Book reproductions** (append to `tests/book.rs`; measure first, then set bounds and record the observed values in comments)

```rust
#[test]
#[ignore]
fn trade_prices_cluster_near_one() {
    // Figure IV-3: prices bunch around the market-clearing level of 1.
    let config = presets::by_id("iv-3-trade").unwrap().config;
    let means: Vec<f64> = (1..=3)
        .map(|seed| {
            let w = run(config.clone(), seed, 1000);
            let s = w.stats.series("mean_log_price").unwrap();
            s[500..].iter().sum::<f64>() / s[500..].len() as f64
        })
        .collect();
    let mean = means.iter().sum::<f64>() / means.len() as f64;
    assert!(mean.abs() < 0.25, "mean ln price {mean} ({means:?})");
}

#[test]
#[ignore]
fn trade_raises_carrying_capacity() {
    // Figure IV-6: carrying capacity is higher with trade than without.
    let with = presets::by_id("iv-3-trade").unwrap().config;
    let mut without = with.clone();
    without.trade.enabled = false;
    let pop = |c: &Config| (1..=5).map(|s| run(c.clone(), s, 500).population() as f64).sum::<f64>() / 5.0;
    let (p_with, p_without) = (pop(&with), pop(&without));
    assert!(p_with > p_without, "with {p_with}, without {p_without}");
}

#[test]
#[ignore]
fn foresight_evolves_to_a_modest_nonzero_level() {
    // Figure IV-18: some foresight is fit; large foresight is not.
    let config = presets::by_id("iv-18-foresight").unwrap().config;
    for seed in 1..=3 {
        let w = run(config.clone(), seed, 1000);
        let f = w.stats.series("mean_foresight").unwrap();
        let (start, end) = (f[0], *f.last().unwrap());
        assert!(end > 0.1 && end < start, "seed {seed}: {start} → {end}");
    }
}
```
If a bound fails, debug against the rule text before adjusting; any adjusted bound needs a comment with the observed values and why the book still supports it.

- [ ] **Step 5: Verify**

```bash
cargo test -p sugarscape-core
cargo test -p sugarscape-core --release --test invariants
cargo test -p sugarscape-core --release --test book -- --ignored
```
- [ ] **Step 6: Commit** — `git commit -m "Add Chapter IV presets, invariants and book checks"`.

---

### Task 14: WASM bindings

**Files:**
- Modify: `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Produces JS: `Sim.networks(kind: "trade" | "credit"): Uint32Array` (`[x1, y1, x2, y2, …]`; throws on unknown kind); `Sim.supply_demand(): Float64Array` (`[n, prices(n), demand(n), supply(n), eq_price, eq_quantity, actual_price, actual_quantity]`). Existing `render()` accepts the new mode/layer names automatically.

- [ ] **Step 1: Failing test** (`tests/web.rs`)
```rust
#[wasm_bindgen_test]
fn trade_preset_exposes_networks_and_supply_demand() {
    let presets = presets_json();
    assert!(presets.contains("\"iv-3-trade\""));
    let config = r#"{"population":200,"vision":{"min":1,"max":5},"metabolism":{"min":1,"max":5},
        "endowment":{"min":25,"max":50},"spice":{"enabled":true,"metabolism":{"min":1,"max":5},
        "endowment":{"min":25,"max":50}},"trade":{"enabled":true}}"#;
    let mut sim = Sim::new(config, 1, None).unwrap();
    sim.step(5);
    let edges = sim.networks("trade").unwrap();
    assert_eq!(edges.len() % 4, 0);
    let sd = sim.supply_demand();
    assert_eq!(sd[0] as usize, 41);
    assert_eq!(sd.len(), 1 + 3 * 41 + 4);
    assert!(sim.networks("gossip").is_err());
    sim.render("credit", "spice").unwrap();
}
```

- [ ] **Step 2: Run** — `wasm-pack test --node crates/sugarscape-wasm` fails to compile.

- [ ] **Step 3: Implement** (in `impl Sim`)
```rust
    /// Edges as `[x1, y1, x2, y2, …]` for `"trade"` (this tick) or `"credit"` (outstanding).
    pub fn networks(&self, kind: &str) -> Result<Vec<u32>, JsValue> {
        let edges = match kind {
            "trade" => network::trade_edges(&self.world),
            "credit" => network::credit_edges(&self.world),
            _ => return Err(edit_error(format!("unknown network {kind:?}"))),
        };
        Ok(edges.into_iter().flat_map(|(a, b)| [a.x, a.y, b.x, b.y]).collect())
    }

    /// `[n, prices(n), demand(n), supply(n), eq_price, eq_quantity, actual_price, actual_quantity]`.
    pub fn supply_demand(&self) -> Vec<f64> {
        let sd = stats::supply_demand(&self.world);
        let mut out = vec![sd.prices.len() as f64];
        out.extend(sd.prices);
        out.extend(sd.demand);
        out.extend(sd.supply);
        out.extend([sd.equilibrium_price, sd.equilibrium_quantity, sd.actual_price, sd.actual_quantity]);
        out
    }
```
(import `sugarscape_core::network`.)

- [ ] **Step 4: Verify** — `wasm-pack test --node crates/sugarscape-wasm` (6 passed); `cargo clippy --all-targets -- -D warnings`.
- [ ] **Step 5: Commit** — `git commit -m "Expose networks and supply and demand to JavaScript"`.

---

### Task 15: Web types, rules and schedule

**Files:**
- Modify: `web/src/types.ts`, `web/src/schema.ts`, `web/src/ui/rules-panel.ts`, `web/src/style.css`

**Interfaces:**
- Produces TS: `Config.{spice: {enabled; metabolism: URange; endowment: URange}, trade: {enabled}, credit: {enabled; duration; rate}, foresight: {enabled; range: URange}, schedule: ScheduledChange[]}`, `Config.pollution.spice_pollutes`, `ScheduledChange { tick: number; set: Record<string, unknown> }`; `Snapshot` + the 11 new series; `SiteView.{spice, spice_capacity}`; `AgentView.{spice, initial_spice, spice_metabolism, foresight, loans: LoanView[]}`; `LoanView { id; role: 'lender' | 'borrower'; counterparty: LinkView; due; due_tick }`; `ColorMode` + `'credit'`; `Layer` + `'spice' | 'spice_capacity'`; schema groups Spice, Trade (T), Credit (L), Foresight; Pollution gains a "Spice pollutes too" toggle; RulesPanel Schedule section.

- [ ] **Step 1: Types** — add exactly the shapes above to `types.ts`.

- [ ] **Step 2: Schema** — in `schema.ts`, add to the Pollution group `{ kind: 'toggle', path: 'pollution.spice_pollutes', label: 'Spice pollutes too' }`, and append groups:
```ts
  {
    title: 'Spice', enable: 'spice.enabled',
    note: 'A second good on mountains opposite the sugar. Changes apply to agents born from now on.',
    controls: [
      { kind: 'range', path: 'spice.metabolism', label: 'Spice metabolism', min: 0, max: 10 },
      { kind: 'range', path: 'spice.endowment', label: 'Initial spice', min: 0, max: 500 },
    ],
  },
  { title: 'Trade (T)', enable: 'trade.enabled', note: 'Needs spice.', controls: [] },
  {
    title: 'Credit (L)', enable: 'credit.enabled', note: 'Sugar loans for childbearing; needs sex.',
    controls: [
      { kind: 'number', path: 'credit.duration', label: 'Duration d (ticks)', min: 1, max: 50, step: 1 },
      { kind: 'number', path: 'credit.rate', label: 'Interest r (% per tick)', min: 0, max: 100, step: 1 },
    ],
  },
  {
    title: 'Foresight', enable: 'foresight.enabled', note: 'Needs spice.',
    controls: [{ kind: 'range', path: 'foresight.range', label: 'Foresight φ', min: 0, max: 20 }],
  },
```

- [ ] **Step 3: Schedule section** — in `RulesPanel`, after the preset section:
```ts
  private scheduleSection(): HTMLElement {
    const list = h('ul', { class: 'schedule' });
    const clear = h('button', { onclick: () => this.commit((c) => (c.schedule = []), false) }, 'Clear schedule');
    const section = h('section', { class: 'group' }, h('h3', {}, 'Schedule'), list, clear, this.errorSlot('schedule'));
    this.syncers.push(() => {
      const entries = [...this.engine.config.schedule].sort((a, b) => a.tick - b.tick);
      section.hidden = entries.length === 0;
      list.replaceChildren(
        ...entries.flatMap((e) =>
          Object.entries(e.set).map(([path, value]) => h('li', {}, `t = ${e.tick} · ${path} = ${JSON.stringify(value)}`)),
        ),
      );
    });
    return section;
  }
```
and include it in the constructor's `this.el.append(this.presetSection(), this.scheduleSection(), this.general, …)`. CSS: `.rules .schedule { margin: 0 0 6px; padding-left: 1.2em; font-size: 12px; }`.

- [ ] **Step 4: Verify** — `cd web && npm run build && npm test`.
- [ ] **Step 5: Commit** — `git commit -m "Add Chapter IV rules and schedule to the Rules panel"`.

---

### Task 16: Layers, credit colors and network overlays

**Files:**
- Create: `web/src/ui/overlay.ts`, `web/src/ui/overlay.test.ts`
- Modify: `web/src/engine.ts`, `web/src/ui/display.ts`, `web/src/ui/grid-view.ts`

**Interfaces:**
- Produces: `overlay.ts`: `wrappedSegments(x1, y1, x2, y2, width, height): [number, number, number, number][]` — one segment, or two when the edge crosses the torus seam; `Engine.overlays: { trade: boolean; credit: boolean }`; `Engine.setDisplay({ overlays? })`.

- [ ] **Step 1: Failing vitest** (`overlay.test.ts`)
```ts
import { describe, expect, it } from 'vitest';
import { wrappedSegments } from './overlay';

describe('wrappedSegments', () => {
  it('keeps nearby edges as one segment', () => {
    expect(wrappedSegments(1, 1, 2, 1, 50, 50)).toEqual([[1, 1, 2, 1]]);
  });
  it('splits an edge that wraps horizontally', () => {
    expect(wrappedSegments(0, 5, 49, 5, 50, 50)).toEqual([[0, 5, -1, 5], [50, 5, 49, 5]]);
  });
  it('splits an edge that wraps vertically', () => {
    expect(wrappedSegments(3, 49, 3, 0, 50, 50)).toEqual([[3, 49, 3, 50], [3, -1, 3, 0]]);
  });
});
```
- [ ] **Step 2: Run** — `npx vitest run src/ui/overlay.test.ts` fails.
- [ ] **Step 3: Implement**

`overlay.ts`:
```ts
/** Line segments (in cell units) for an edge on a torus: one if the endpoints
 *  are within half the grid, otherwise two segments running off each side. */
export function wrappedSegments(
  x1: number, y1: number, x2: number, y2: number, width: number, height: number,
): [number, number, number, number][] {
  let dx = x2 - x1;
  let dy = y2 - y1;
  if (dx > width / 2) dx -= width;
  if (dx < -width / 2) dx += width;
  if (dy > height / 2) dy -= height;
  if (dy < -height / 2) dy += height;
  if (x1 + dx === x2 && y1 + dy === y2) return [[x1, y1, x2, y2]];
  return [[x1, y1, x1 + dx, y1 + dy], [x2 - dx, y2 - dy, x2, y2]];
}
```
`engine.ts`: `overlays = { trade: false, credit: false };` and `setDisplay(d: { colorMode?: ColorMode; layer?: Layer; overlays?: Partial<{ trade: boolean; credit: boolean }> })` merging `d.overlays` into `this.overlays`.

`display.ts`: add `['credit', 'Credit']` to `MODES`, `['spice', 'Spice'], ['spice_capacity', 'Spice capacity']` to `LAYERS`, and two checkboxes:
```ts
  const overlay = (kind: 'trade' | 'credit', label: string) => {
    const box = h('input', { type: 'checkbox', onchange: () => engine.setDisplay({ overlays: { [kind]: box.checked } }) });
    engine.on('display', () => (box.checked = engine.overlays[kind]));
    return h('label', {}, box, ` ${label}`);
  };
```
appended to the returned `display-controls` element as `overlay('trade', 'Trade network'), overlay('credit', 'Credit network')`.

`grid-view.ts` — in `draw()`, after drawing the frame and before the selection ring:
```ts
    for (const [kind, color] of [['trade', '--c3'], ['credit', '--c2']] as const) {
      if (!this.engine.overlays[kind]) continue;
      const e = this.engine.sim.networks(kind);
      ctx.save();
      ctx.strokeStyle = getComputedStyle(this.canvas).getPropertyValue(color).trim() || '#fff';
      ctx.globalAlpha = 0.8;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      for (let i = 0; i < e.length; i += 4) {
        for (const [ax, ay, bx, by] of wrappedSegments(e[i], e[i + 1], e[i + 2], e[i + 3], width, height)) {
          ctx.moveTo((ax + 0.5) * CELL, (ay + 0.5) * CELL);
          ctx.lineTo((bx + 0.5) * CELL, (by + 0.5) * CELL);
        }
      }
      ctx.stroke();
      ctx.restore();
    }
```
(overlays are not included in `toPngBlob`).

- [ ] **Step 4: Verify** — `cd web && npm run build && npm test`.
- [ ] **Step 5: Commit** — `git commit -m "Add spice layers, credit colors and network overlays"`.

---

### Task 17: Economy charts

**Files:**
- Modify: `web/src/ui/charts-panel.ts`, `web/src/style.css`

**Interfaces:**
- Produces: an **Economy** section (`<section class="economy">` inside the charts panel, shown when `config.spice.enabled || config.credit.enabled`, re-evaluated on `reset`/`config`) with charts: "Trade price (ln)" (mean, mean ± SD dashed), "Trade volume", "Supply & demand" (x = price on a log scale; demand, supply; single-point series for equilibrium and actual), "Loans" (loans made, defaults), "Debt outstanding", "Spice & foresight" (mean spice metabolism, mean foresight). Hidden charts are not updated.

- [ ] **Step 1: Implement**

Refactor `add()` to take a target container (default `this.el`) and a `visible: () => boolean` guard stored on each plot; `refresh()` skips plots whose guard is false. Build the economy section in `build()`:
```ts
    const economy = h('section', { class: 'economy' }, h('h3', {}, 'Economy'));
    this.el.append(economy);
    const econOn = () => this.engine.config.spice.enabled || this.engine.config.credit.enabled;
    const syncSection = () => { economy.hidden = !econOn(); this.drawnTick = null; };
    this.engine.on('reset', syncSection);
    this.engine.on('config', syncSection);
    syncSection();
```
Price band chart update:
```ts
      (plot, series) => {
        const m = series('mean_log_price');
        const sd = series('sd_log_price');
        plot.setData([series('tick'), m, m.map((v, i) => v + sd[i]), m.map((v, i) => v - sd[i])]);
      },
```
(series: mean solid `--c2`; ±SD dashed `--muted`). Time charts for volume (`trade_volume`), loans (`loans_made`, `defaults`), debt (`debt_outstanding`), and traits (`mean_spice_metabolism`, `mean_foresight`) reuse the existing time-chart builder with the economy container and guard.

Supply & demand:
```ts
    this.add('Supply & demand', {
      scales: { x: { time: false, distr: 3 }, y: {} },
      axes,
      legend: { show: true },
      series: [
        { label: 'Price' },
        { label: 'Demand', stroke: color('--c1'), width: 1.5 },
        { label: 'Supply', stroke: color('--c2'), width: 1.5 },
        { label: 'Equilibrium', stroke: color('--c3'), points: { show: true, size: 9 }, paths: () => null },
        { label: 'Actual', stroke: color('--text'), points: { show: true, size: 9 }, paths: () => null },
      ],
    }, [[], [], [], [], []], (plot) => {
      const sd = this.engine.sim.supply_demand();
      const n = sd[0];
      const prices = Array.from(sd.subarray(1, 1 + n));
      const demand = Array.from(sd.subarray(1 + n, 1 + 2 * n));
      const supply = Array.from(sd.subarray(1 + 2 * n, 1 + 3 * n));
      const [eqP, eqQ, actP, actQ] = Array.from(sd.subarray(1 + 3 * n));
      const nearest = (p: number) => prices.reduce((best, q, i) => (Math.abs(Math.log(q / p)) < Math.abs(Math.log(prices[best] / p)) ? i : best), 0);
      const point = (p: number, q: number) => {
        const col: (number | null)[] = prices.map(() => null);
        if (Number.isFinite(p) && Number.isFinite(q)) col[nearest(p)] = q;
        return col;
      };
      plot.setData([prices, demand, supply, point(eqP, eqQ), point(actP, actQ)]);
    }, economy, econOn);
```
If uPlot's types reject `paths: () => null`, use the documented points-only idiom from `node_modules/uplot/dist/uPlot.d.ts` (e.g. `paths: () => null` cast, or `width: 0`).

CSS: `.charts .economy h3 { font-size: 13px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); margin: 8px 0 0; }`.

- [ ] **Step 2: Verify** — `cd web && npm run build && npm test`.
- [ ] **Step 3: Commit** — `git commit -m "Add Economy charts"`.

---

### Task 18: Inspector and README

**Files:**
- Modify: `web/src/ui/inspect-panel.ts`, `README.md`

- [ ] **Step 1: Inspector** — in `agentRows`, after the Sugar row:
```ts
      ...(this.engine.config.spice.enabled
        ? [row('Spice', `${fmt(a.spice)} (born with ${fmt(a.initial_spice)})`), row('Spice metabolism', String(a.spice_metabolism))]
        : []),
      ...(this.engine.config.foresight.enabled ? [row('Foresight φ', String(a.foresight))] : []),
```
and after Children:
```ts
      ...(a.loans.length
        ? [row('Loans', h('span', { class: 'links' }, ...a.loans.map((l) =>
            h('span', {}, `${l.role === 'lender' ? 'lent to' : 'owes'} `,
              l.counterparty.alive
                ? h('button', { class: 'link', onclick: () => this.engine.follow(l.counterparty.id) }, `#${l.counterparty.id}`)
                : h('span', { class: 'hint' }, `#${l.counterparty.id}†`),
              ` ${fmt(l.due)} by t=${l.due_tick}`))))]
        : []),
```
In `render()`, add a Spice row for the site when spice is on: `row('Spice', `${fmt(site.spice)} / ${fmt(site.spice_capacity)}`)`.

- [ ] **Step 2: README** — in "Rules implemented", add: "Chapter IV: spice and multicommodity movement, trade (T), credit (L), foresight, sugar-as-dirty-good pollution, scheduled rule changes, supply and demand, and trade/credit network overlays." Mention `docs/roadmap.md` for future work.

- [ ] **Step 3: Full verification**
```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test -p sugarscape-core
cargo test -p sugarscape-core --release --test book -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
```
- [ ] **Step 4: Commit** — `git commit -m "Show spice, foresight and loans in the inspector; update README"`.
