# Playground Controls Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Step back and scrub through a run (keyframes + replay), stop a run at a tick or when a series crosses a threshold, show measured ticks per second, and drive the toolbar from the keyboard.

**Architecture:** The core gains world copies (`ModelWorld::checkpoint`/`restore`, history kept out of the copy) and a cheap `latest_value`. The worker host (`web/src/sim-host.ts`) keeps thinned keyframes, answers a new `seek` command by restoring the nearest keyframe and replaying the edit log forward, and checks stop rules after every tick. The engine and the Compare lockstep expose `seek`, `reached`, `seekable` and `setStops` through `RunControls`; small UI modules (`timeline`, `stop-control`, `rate`, `shortcuts`) sit on top.

**Tech Stack:** Rust (sugarscape-core, wasm-bindgen), TypeScript, Vite, Vitest (Node environment — no DOM in tests).

**Spec:** `docs/superpowers/specs/2026-09-25-playground-controls-design.md`

## Global Constraints

- Golden and legacy tests (`crates/sugarscape-core/tests/golden.rs`, `legacy.rs`, `web/src/*legacy*`) stay unedited and green.
- No required `Model` trait method and no exhaustive `ModelWorld` match that a new variant would break (the civil-violence branch adds `ModelWorld::Civil`); new matches end in a wildcard arm with `#[allow(unreachable_patterns)]`.
- Share links, session files and Compare links keep their format; `seek` and `setStops` are never logged.
- CI runs `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` (latest stable), `cargo test --workspace`, `wasm-pack test --node crates/sugarscape-wasm`, `npm run build`, `npm test` — all must pass.
- American spelling in all text (color, behavior, neighbor).
- Keyframes: first interval `KEYFRAME_EVERY = 50` ticks, at most `MAX_KEYFRAMES = 32`.
- Commit messages end with the line `Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc`.

## Review Focus

1. **Seeking back past an edit made at the same tick as a keyframe** — the world must equal the original run (the keyframe's `applied` count, not its tick, splits the log). Pinned in Task 4.
2. **Dragging the slider fast** — many `input` events must not queue many seeks; only the newest position is sent after the one in flight. Pinned in Task 8.
3. **A stop condition true before Play** (population already below the threshold) — must not stop on the first tick. Pinned in Task 5.
4. **Seeking with a followed agent** — the agent stays followed after the seek. Pinned in Task 4.
5. **Pressing Space while typing in the seed box or a Rules field** — must type, not toggle Play. Pinned in Task 11.

## File Structure

- `crates/sugarscape-core/src/stats.rs` — `Stats::truncate`.
- `crates/sugarscape-core/src/model.rs` — `Checkpoint`, `ModelWorld::checkpoint`/`restore`, `Model::latest_value` (default) and overrides for `World`.
- `crates/sugarscape-core/src/{world,schelling,ring}.rs`, `anasazi/world.rs` — `#[derive(Clone)]` on the world structs; `latest_value` overrides.
- `crates/sugarscape-core/tests/checkpoint.rs` — new: restore/latest-value equivalence for every model.
- `crates/sugarscape-wasm/src/lib.rs` — `Checkpoint` wrapper, `Sim::checkpoint`/`restore`/`latest_value`.
- `web/src/protocol.ts` — `seek`, `setStops` commands, `StopRules`, snapshot fields `reached`, `seekable`, `stopped`.
- `web/src/sim-host.ts` — keyframes, seek, stop rules.
- `web/src/fake-sim.fixture.ts` — fake checkpoint/restore/latest_value.
- `web/src/engine.ts` — `seek`, `reached`, `seekable`, `setStops`, `stops`, `lastStop`, `'stopped'` event, extended `RunControls`.
- `web/src/compare/lockstep.ts` — the same `RunControls` members for two worlds.
- `web/src/ui/timeline.ts` — new: ⟲1, slider, `SeekQueue`.
- `web/src/ui/stop-control.ts` — new: stop rules UI and `parseStopRules`.
- `web/src/ui/rate.ts` — new: `RateMeter`.
- `web/src/ui/shortcuts.ts` — new: `shortcutFor`, `nextSpeed`, help card, `installShortcuts`.
- `web/src/ui/toolbar.ts` — hosts the timeline, stop control and rate readout; exports `SPEEDS`.
- `web/src/main.ts` — notices for `'stopped'`, installs shortcuts.
- `web/src/style.css` — styles for the new controls and the help card.
- `README.md`, `docs/roadmap.md` — docs.

Build note: web tests that use the real WASM (`determinism.test.ts`) need the package rebuilt after core/wasm changes: `cd web && npm run wasm` (release) before `npm test`.

---

### Task 1: Core world copies

**Files:**
- Modify: `crates/sugarscape-core/src/stats.rs` (the `Stats` impl, ~line 283)
- Modify: `crates/sugarscape-core/src/world.rs:82` (`pub struct World`)
- Modify: `crates/sugarscape-core/src/schelling.rs:290`, `crates/sugarscape-core/src/ring.rs:232`, `crates/sugarscape-core/src/anasazi/world.rs:228`
- Modify: `crates/sugarscape-core/src/model.rs` (after `impl ModelWorld`, ~line 339)
- Test: `crates/sugarscape-core/tests/checkpoint.rs` (create)

**Interfaces:**
- Produces: `pub struct Checkpoint` (fields private) with `pub fn tick(&self) -> u64`; `ModelWorld::checkpoint(&mut self) -> Option<Checkpoint>`; `ModelWorld::restore(&mut self, cp: &Checkpoint) -> Result<(), String>`; `Stats::truncate(&mut self, len: usize)`.

- [ ] **Step 1: Write the failing test**

Create `crates/sugarscape-core/tests/checkpoint.rs`:

```rust
//! Keyframes: a world restored from a checkpoint and run on is the world
//! that ran straight there, for every model.

use sugarscape_core::model::ModelWorld;
use sugarscape_core::presets;

/// One preset per model kind.
const IDS: &[&str] = &["vi-1-everything", "vi-4-schelling-25", "vi-8-ring-world", "lhv-published"];

fn world(id: &str) -> ModelWorld {
    let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    ModelWorld::new(preset.config, 1).unwrap()
}

fn all_series(w: &ModelWorld) -> Vec<Option<Vec<f64>>> {
    let m = w.model();
    let mut names = m.series_names();
    names.push("tick".into());
    names.iter().map(|n| m.series(n)).collect()
}

#[test]
fn restore_then_run_equals_a_straight_run() {
    for &id in IDS {
        let mut straight = world(id);
        straight.model_mut().run(60);

        let mut w = world(id);
        w.model_mut().run(20);
        let cp = w.checkpoint().expect("every current model has keyframes");
        assert_eq!(cp.tick(), 20);
        w.model_mut().run(25);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 20, "{id}");
        assert_eq!(w.model().series("tick").unwrap().len(), 21, "{id}: history cut to the keyframe");
        w.model_mut().run(40);

        assert_eq!(w.model().fingerprint(), straight.model().fingerprint(), "{id}");
        assert_eq!(all_series(&w), all_series(&straight), "{id}");
    }
}

#[test]
fn a_checkpoint_leaves_the_live_world_untouched() {
    for &id in IDS {
        let mut a = world(id);
        let mut b = world(id);
        a.model_mut().run(15);
        b.model_mut().run(15);
        let _ = a.checkpoint();
        a.model_mut().run(15);
        b.model_mut().run(15);
        assert_eq!(a.model().fingerprint(), b.model().fingerprint(), "{id}");
        assert_eq!(all_series(&a), all_series(&b), "{id}");
    }
}

#[test]
fn restore_refuses_another_models_checkpoint() {
    let mut sugar = world("vi-1-everything");
    let mut ring = world("vi-8-ring-world");
    let cp = ring.checkpoint().unwrap();
    assert!(sugar.restore(&cp).is_err());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p sugarscape-core --test checkpoint`
Expected: compile error — `no method named checkpoint found for enum ModelWorld`.

- [ ] **Step 3: Implement**

In `stats.rs`, inside `impl<S: Series + Default> Stats<S>` add:

```rust
    /// Keeps the first `len` snapshots (a restored keyframe's history: ticks 0 to its tick).
    pub fn truncate(&mut self, len: usize) {
        self.history.truncate(len);
    }
```

Add `Clone` to the derives of `World` (`world.rs`), `SchellingWorld`, `RingWorld` and `AnasaziWorld` — each currently has no derive line, so add `#[derive(Clone)]` directly above `pub struct …`. Run `cargo build -p sugarscape-core`; for every "the trait `Clone` is not implemented for X" error, add `Clone` to X's derive list (all field types checked so far — `Agent`, `Site`, `Loan`, `TickEvents`, `Bits`, `Household`, `Walker`, `Resident`, `SimRng` = `Pcg64Mcg`, configs — already derive it; `AnasaziWorld::valley` is `&'static Valley`, which is `Copy`).

In `model.rs`, below `impl ModelWorld { … }` add:

```rust
/// A copy of a world's state without its statistics history (a keyframe):
/// about one copy of the current state, however long the run.
pub struct Checkpoint {
    world: ModelWorld,
    tick: u64,
}

impl Checkpoint {
    /// The tick the copy was taken at.
    pub fn tick(&self) -> u64 {
        self.tick
    }
}

/// Moves `$w`'s history out, clones it, and moves the history back.
macro_rules! copy_without_history {
    ($variant:ident, $w:expr) => {{
        let stats = std::mem::take(&mut $w.stats);
        let copy = (**$w).clone();
        $w.stats = stats;
        ModelWorld::$variant(Box::new(copy))
    }};
}

/// Replaces `$live` by a copy of `$kept`, giving it `$live`'s history cut to `$kept`'s tick.
macro_rules! restore_into {
    ($live:expr, $kept:expr) => {{
        let mut stats = std::mem::take(&mut $live.stats);
        stats.truncate($kept.tick as usize + 1);
        let mut next = (**$kept).clone();
        next.stats = stats;
        **$live = next;
    }};
}

impl ModelWorld {
    /// A keyframe of this world, or `None` for a model without them.
    #[allow(unreachable_patterns)]
    pub fn checkpoint(&mut self) -> Option<Checkpoint> {
        let tick = self.model().tick();
        let world = match self {
            ModelWorld::Sugarscape(w) => copy_without_history!(Sugarscape, w),
            ModelWorld::Schelling(w) => copy_without_history!(Schelling, w),
            ModelWorld::Ring(w) => copy_without_history!(Ring, w),
            ModelWorld::Anasazi(w) => copy_without_history!(Anasazi, w),
            _ => return None,
        };
        Some(Checkpoint { world, tick })
    }

    /// Returns this world to `cp`, keeping its statistics history up to `cp`'s tick. The world
    /// must have reached that tick (its history must hold it) and be of the same model.
    #[allow(unreachable_patterns)]
    pub fn restore(&mut self, cp: &Checkpoint) -> Result<(), String> {
        if self.model().tick() < cp.tick {
            return Err(format!("this world has not reached tick {}", cp.tick));
        }
        match (self, &cp.world) {
            (ModelWorld::Sugarscape(live), ModelWorld::Sugarscape(kept)) => restore_into!(live, kept),
            (ModelWorld::Schelling(live), ModelWorld::Schelling(kept)) => restore_into!(live, kept),
            (ModelWorld::Ring(live), ModelWorld::Ring(kept)) => restore_into!(live, kept),
            (ModelWorld::Anasazi(live), ModelWorld::Anasazi(kept)) => restore_into!(live, kept),
            _ => return Err("the keyframe is of another model".into()),
        }
        Ok(())
    }
}
```

`Stats` needs `Default` for `mem::take`: it has a manual `impl<S> Default`, so this compiles. If a world's `stats` field is private to its module, it is `pub` already for all four (`pub stats:`).

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --test checkpoint && cargo test -p sugarscape-core --test golden`
Expected: PASS (3 tests; golden unchanged).

- [ ] **Step 5: Lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core
git commit -m "Copy and restore worlds as keyframes, without their history

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 2: Latest value, and the WASM bindings

**Files:**
- Modify: `crates/sugarscape-core/src/model.rs` (`trait Model`, ~line 218; `impl Model for World`)
- Modify: the `impl Model for` blocks in `schelling.rs`, `ring.rs`, `anasazi/world.rs`
- Modify: `crates/sugarscape-wasm/src/lib.rs` (after `pub struct Sim`, ~line 194; `#[wasm_bindgen] impl Sim`)
- Test: `crates/sugarscape-core/tests/checkpoint.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `ModelWorld::checkpoint`/`restore`, `Checkpoint` (Task 1).
- Produces: `Model::latest_value(&self, name: &str) -> Option<f64>`; JS `Sim.checkpoint(): Checkpoint | undefined`, `Sim.restore(cp: Checkpoint): void` (throws a field-error JSON string on refusal), `Sim.latest_value(name: string): number | undefined`, `Checkpoint.free()`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/sugarscape-core/tests/checkpoint.rs`:

```rust
#[test]
fn latest_value_is_the_last_element_of_every_series() {
    for &id in IDS {
        let mut w = world(id);
        w.model_mut().run(12);
        let m = w.model();
        let mut names = m.series_names();
        names.push("tick".into());
        for name in names {
            let last = m.series(&name).and_then(|v| v.last().copied());
            let got = m.latest_value(&name);
            // NaN (an undefined statistic) compares by bits.
            assert_eq!(got.map(f64::to_bits), last.map(f64::to_bits), "{id}: {name}");
        }
        assert_eq!(m.latest_value("no-such-series"), None, "{id}");
    }
}
```

Look at `crates/sugarscape-wasm/tests/web.rs` for how it builds a `Sim` (it uses `wasm_bindgen_test`), and append a test in the same style:

```rust
#[wasm_bindgen_test]
fn checkpoint_restores_the_same_world() {
    let config = sugarscape_core::presets::find("ii-2-unit").unwrap().config;
    let json = serde_json::to_string(&config).unwrap();
    let mut straight = Sim::new(&json, 1, JsValue::UNDEFINED).unwrap();
    straight.step(30);
    let mut sim = Sim::new(&json, 1, JsValue::UNDEFINED).unwrap();
    sim.step(10);
    let cp = sim.checkpoint().unwrap();
    sim.step(7);
    sim.restore(&cp).unwrap();
    assert_eq!(sim.tick(), 10.0);
    sim.step(20);
    assert_eq!(sim.fingerprint(), straight.fingerprint());
    assert_eq!(sim.latest_value("population"), Some(f64::from(sim.population())));
}
```

(If `web.rs` constructs configs differently — e.g. from `default_config_json()` — follow its pattern for building `json`; the assertions stay the same.)

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p sugarscape-core --test checkpoint latest_value`
Expected: compile error — `no method named latest_value`.

- [ ] **Step 3: Implement**

In `trait Model` (model.rs), after `fn series(...)`:

```rust
    /// The latest value of series `name` (or `"tick"`), or `None` if unknown or there is no history yet.
    fn latest_value(&self, name: &str) -> Option<f64> {
        self.series(name).and_then(|v| v.last().copied())
    }
```

Override it in `impl Model for World` (model.rs), and in the `impl Model for` blocks of `SchellingWorld`, `RingWorld` and `AnasaziWorld` (each has `pub stats: Stats<…>` whose snapshot type implements `Series`):

```rust
    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }
```

Import `crate::stats::Series` where the file does not already (`use crate::stats::{Series, Stats};` exists in the three model files; add `Series` to `model.rs`'s imports if needed).

In `crates/sugarscape-wasm/src/lib.rs`, after the `Sim` struct:

```rust
/// A keyframe of a `Sim`'s world (`ModelWorld::checkpoint`): the host keeps a few and frees them.
#[wasm_bindgen]
pub struct Checkpoint(sugarscape_core::model::Checkpoint);
```

and inside `#[wasm_bindgen] impl Sim`:

```rust
    /// A keyframe of the world now, or `undefined` for a model without them.
    pub fn checkpoint(&mut self) -> Option<Checkpoint> {
        self.world.checkpoint().map(Checkpoint)
    }

    /// Returns the world to `cp` (see `ModelWorld::restore`); throws a field error if it cannot.
    pub fn restore(&mut self, cp: &Checkpoint) -> Result<(), JsValue> {
        self.world.restore(&cp.0).map_err(edit_error)
    }

    /// The latest value of series `name` (or `"tick"`), or `undefined`.
    pub fn latest_value(&self, name: &str) -> Option<f64> {
        self.model().latest_value(name)
    }
```

(`edit_error` is the existing helper `sugar_mut` uses; it takes a `String`.)

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --test checkpoint && wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS.

- [ ] **Step 5: Rebuild the web package, lint, commit**

```bash
(cd web && npm run wasm)
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates
git commit -m "Read a series' latest value cheaply; expose keyframes to the page

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 3: Host keyframes

**Files:**
- Modify: `web/src/sim-host.ts` (`SimLike`, constants, `SimHost` fields, `apply` init/reset, `advance`)
- Modify: `web/src/fake-sim.fixture.ts`
- Test: `web/src/sim-host.test.ts`

**Interfaces:**
- Produces: `interface CheckpointLike { free(): void }`; `SimLike.checkpoint(): CheckpointLike | undefined`, `SimLike.restore(cp: CheckpointLike): void`, `SimLike.latest_value(name: string): number | undefined`; exported `KEYFRAME_EVERY = 50`, `MAX_KEYFRAMES = 32`; `SimHost.keyframeTicks(): number[]` (for tests).
- Fake: `FakeSim.checkpoint()` returns a `FakeCheckpoint` holding `{ ticks, agents, config }` copies; `restore` copies them back; `latest_value('population')` = agent count, `latest_value('tick')` = ticks, others `undefined`; `FakeSim.restores` counts restores; `fakeModule(log, { keyframes: false })` makes worlds whose `checkpoint()` returns `undefined`.

- [ ] **Step 1: Write the failing tests**

Append to `web/src/sim-host.test.ts` (reuse its existing imports and helpers; `request`-style helpers already exist there — follow the file's pattern for sending commands, e.g. `host.handle({ id, cmd })`):

```ts
describe('SimHost keyframes', () => {
  const init = (host: SimHost) =>
    host.handle({ id: 1, cmd: { type: 'init', config: { width: 4, height: 3 } as never, seed: 1, landscapes: [], display: { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() } } });

  it('keeps one at t = 0 and one every KEYFRAME_EVERY ticks', () => {
    const host = new SimHost(fakeModule());
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: 175 } });
    expect(host.keyframeTicks()).toEqual([0, 50, 100, 150]);
  });

  it('thins to every other one and doubles the interval past MAX_KEYFRAMES', () => {
    const host = new SimHost(fakeModule());
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: KEYFRAME_EVERY * MAX_KEYFRAMES } });
    const ticks = host.keyframeTicks();
    expect(ticks.length).toBeLessThanOrEqual(MAX_KEYFRAMES);
    expect(ticks.every((t) => t % (2 * KEYFRAME_EVERY) === 0)).toBe(true);
    expect(ticks.at(-1)).toBe(KEYFRAME_EVERY * MAX_KEYFRAMES);
  });

  it('frees thinned keyframes, and all of them on reset', () => {
    const log: string[] = [];
    const host = new SimHost(fakeModule(log));
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: KEYFRAME_EVERY * MAX_KEYFRAMES } });
    const freed = log.filter((l) => l === 'free checkpoint').length;
    expect(freed).toBeGreaterThan(0);
    host.handle({ id: 3, cmd: { type: 'reset', config: { width: 4, height: 3 } as never, seed: 2, landscapes: [] } });
    expect(host.keyframeTicks()).toEqual([0]);
  });

  it('keeps none for a model without keyframes', () => {
    const host = new SimHost(fakeModule([], { keyframes: false }));
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: 120 } });
    expect(host.keyframeTicks()).toEqual([]);
  });
});
```

Add `KEYFRAME_EVERY, MAX_KEYFRAMES` to the file's import from `./sim-host` and `noOverlays` from `./protocol` if not imported.

- [ ] **Step 2: Run to verify they fail**

Run: `cd web && npx vitest run src/sim-host.test.ts`
Expected: FAIL — `KEYFRAME_EVERY` is not exported / `keyframeTicks` is not a function.

- [ ] **Step 3: Implement the fake**

In `fake-sim.fixture.ts`:

```ts
/** A fake keyframe: copies of the state a restore puts back. */
export class FakeCheckpoint {
  constructor(
    readonly ticks: number,
    readonly agents: Map<number, [number, number]>,
    readonly config: FakeConfig,
    private log: string[],
  ) {}
  free(): void {
    this.log.push('free checkpoint');
  }
}
```

In `FakeSim` add a constructor option and methods (add `private keyframes = true` set from the module, and `restores = 0`):

```ts
  checkpoint(): FakeCheckpoint | undefined {
    if (!this.keyframes) return undefined;
    const agents = new Map([...this.agents].map(([id, p]) => [id, [p[0], p[1]] as [number, number]]));
    return new FakeCheckpoint(this.ticks, agents, structuredClone(this.config), this.log);
  }
  restore(cp: FakeCheckpoint): void {
    if (cp.ticks > this.ticks) throw fieldError('tick', `this world has not reached tick ${cp.ticks}`);
    this.restores++;
    this.ticks = cp.ticks;
    this.agents = new Map([...cp.agents].map(([id, p]) => [id, [p[0], p[1]] as [number, number]]));
    this.config = structuredClone(cp.config);
  }
  latest_value(name: string): number | undefined {
    if (name === 'population') return this.agents.size;
    if (name === 'tick') return this.ticks;
    return undefined;
  }
```

Change `fakeModule(log: string[] = [])` to `fakeModule(log: string[] = [], opts: { keyframes?: boolean } = {})` and set `sim.keyframes = opts.keyframes ?? true` after creating each sim (make the field public: `keyframes = true`).

Also enrich the fake's `fingerprint()` so seek tests see edits, not just the tick:

```ts
  fingerprint(): string {
    const agents = [...this.agents].map(([id, p]) => `${id}@${p[0]},${p[1]}`).join(';');
    return `0x${this.ticks.toString(16)}|${agents}|${this.config.population}`;
  }
```

Then run `npx vitest run` and update any existing assertion that compared the fake's fingerprint to a literal `0x…` string to use the new form (search: `grep -rn "fingerprint" web/src/*.test.ts web/src/**/*.test.ts`); real-WASM fingerprints (determinism tests) are unaffected.

- [ ] **Step 4: Implement keyframes in the host**

In `sim-host.ts`, extend `SimLike`:

```ts
  /** A keyframe of the world now, or undefined for a model without them. */
  checkpoint(): CheckpointLike | undefined;
  /** Returns the world to a keyframe taken from it (throws a field error if it cannot). */
  restore(cp: CheckpointLike): void;
  /** The latest value of series `name` (or `'tick'`), or undefined. */
  latest_value(name: string): number | undefined;
```

and above it:

```ts
/** A keyframe the host holds (the WASM `Checkpoint`); freed when dropped. */
export interface CheckpointLike { free(): void }
```

Constants (next to `LOG_CAP`):

```ts
/** The host keeps a keyframe every this many ticks at first… */
export const KEYFRAME_EVERY = 50;
/** …and at most this many: past it, every other one goes and the interval doubles. */
export const MAX_KEYFRAMES = 32;
```

Fields in `SimHost`:

```ts
  /** Keyframes of this world, oldest first: the tick, how many log entries were applied then, the copy. */
  private keyframes: { tick: number; applied: number; cp: CheckpointLike }[] = [];
  /** Keyframes are taken every this many ticks (doubling as they thin out). */
  private every = KEYFRAME_EVERY;
  /** The world gave no keyframe (a model without them): stop asking. */
  private noKeyframes = false;
```

Add:

```ts
  /** The keyframes' ticks (tests). */
  keyframeTicks(): number[] {
    return this.keyframes.map((k) => k.tick);
  }

  private dropKeyframes(keep: (tick: number) => boolean): void {
    const kept = [];
    for (const k of this.keyframes) {
      if (keep(k.tick)) kept.push(k);
      else k.cp.free();
    }
    this.keyframes = kept;
  }

  /** Takes a keyframe if one is due at the world's tick (and none is held there yet). */
  private keyframe(sim: SimLike): void {
    const tick = sim.tick();
    if (this.noKeyframes || tick % this.every !== 0 || this.keyframes.some((k) => k.tick === tick)) return;
    const cp = sim.checkpoint();
    if (!cp) {
      this.noKeyframes = true;
      return;
    }
    this.keyframes.push({ tick, applied: this.log.length, cp });
    this.keyframes.sort((a, b) => a.tick - b.tick);
    if (this.keyframes.length > MAX_KEYFRAMES) {
      this.every *= 2;
      this.dropKeyframes((t) => t % this.every === 0);
    }
  }
```

In `apply`, the `init`/`reset` branch — after `this.replayDue(next);` insert:

```ts
      this.dropKeyframes(() => false);
      this.every = KEYFRAME_EVERY;
      this.noKeyframes = false;
      this.keyframe(next);
```

In `advance`, make each chunk also stop at the next keyframe tick, and take one after each chunk:

```ts
  private advance(sim: SimLike, n: number): void {
    let left = Math.min(n, MAX_TICKS - sim.tick());
    while (left > 0) {
      const tick = sim.tick();
      const next = this.pending[this.cursor]?.tick;
      let k = next === undefined ? left : Math.min(left, Math.max(1, next - tick));
      // Stop at the next keyframe tick too (`step(k)` is the same world as k single steps).
      if (!this.noKeyframes) k = Math.min(k, this.every - (tick % this.every));
      sim.step(k);
      this.fired(tick, sim.tick());
      this.replayDue(sim);
      this.keyframe(sim);
      left -= k;
    }
  }
```

(A world that stops short — `finished()` — makes `sim.tick()` stop moving; the loop still ends because `left` decreases by `k` each pass, as before.)

- [ ] **Step 5: Run the tests**

Run: `cd web && npx vitest run`
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src
git commit -m "Keep thinned keyframes of the world in the host

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 4: The `seek` command

**Files:**
- Modify: `web/src/protocol.ts` (`WorldSnapshot`, `Command`)
- Modify: `web/src/sim-host.ts`
- Test: `web/src/sim-host.test.ts`

**Interfaces:**
- Consumes: keyframes (Task 3).
- Produces: `Command` member `{ type: 'seek'; tick: number }`; `WorldSnapshot.reached?: number` and `WorldSnapshot.seekable?: boolean` (sent on change and after every init/reset/seek); host errors `field: 'seek'` for a refused seek.

Semantics (from the spec): seeking forward just advances; seeking back restores the nearest keyframe at or before the target (never after the current tick) or rebuilds from the kept setup; the full log (applied + pending) is split at the keyframe's `applied` count; stop rules do not fire during a seek; `reached` is the furthest tick on this branch, lowered to the current tick by a successful page edit, which also frees keyframes after the current tick; seeking is refused with a full log or past `reached`; the followed agent stays followed.

- [ ] **Step 1: Write the failing tests**

Append to `web/src/sim-host.test.ts`:

```ts
describe('SimHost seek', () => {
  const display = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() } as const;
  let id = 0;
  const send = (host: SimHost, cmd: Command) => host.handle({ id: ++id, cmd }).result;
  const setup = (opts: { keyframes?: boolean } = {}) => {
    const module = fakeModule([], opts);
    const host = new SimHost(module);
    send(host, { type: 'init', config: { width: 8, height: 3 } as never, seed: 1, landscapes: [], display });
    return { host, module, sim: () => module.sims.at(-1)! };
  };
  const fp = (host: SimHost) => (send(host, { type: 'fingerprint' }) as { value: string }).value;
  const place = (x: number): Command => ({ type: 'place', x, y: 0, overrides: {} });

  /** A reference run: the same edits at the same ticks, straight through to `to`. */
  function straight(edits: [number, Command][], to: number): string {
    const { host } = setup();
    for (const [t, cmd] of edits) {
      send(host, { type: 'step', n: t - (host as unknown as { sim: FakeSim }).sim.ticks });
      send(host, cmd);
    }
    send(host, { type: 'step', n: to - (host as unknown as { sim: FakeSim }).sim.ticks });
    return fp(host);
  }

  it('seeks back through keyframes and edits (before, at and after the target) to the straight run', () => {
    const edits: [number, Command][] = [[30, place(2)], [50, place(3)], [120, place(4)]];
    const { host, sim } = setup();
    for (const [t, cmd] of edits) {
      send(host, { type: 'step', n: t - sim().ticks });
      send(host, cmd);
    }
    send(host, { type: 'step', n: 200 - sim().ticks });
    for (const target of [130, 120, 51, 50, 49, 0, 175]) {
      const r = send(host, { type: 'seek', tick: target });
      expect(r.ok).toBe(true);
      expect(sim().ticks).toBe(target);
      expect(fp(host)).toBe(straight(edits.filter(([t]) => t <= target), target));
    }
  });

  it('restores a keyframe rather than rebuilding when one is at or before the target', () => {
    const { host, module, sim } = setup();
    send(host, { type: 'step', n: 180 });
    send(host, { type: 'seek', tick: 120 });
    expect(module.sims.length).toBe(1);
    expect(sim().restores).toBe(1);
  });

  it('rebuilds from the setup when the model has no keyframes', () => {
    const { host, module } = setup({ keyframes: false });
    send(host, { type: 'place', x: 2, y: 0, overrides: {} });
    send(host, { type: 'step', n: 80 });
    const before = fp(host);
    send(host, { type: 'seek', tick: 10 });
    send(host, { type: 'seek', tick: 80 });
    expect(module.sims.length).toBe(2);
    expect(fp(host)).toBe(before);
  });

  it('replays the later edits after seeking back, and reports them pending', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 60 });
    send(host, place(5));
    send(host, { type: 'step', n: 40 });
    const end = fp(host);
    const r = send(host, { type: 'seek', tick: 20 }) as { ok: true; snapshot: WorldSnapshot };
    expect(r.snapshot.replayLeft).toBe(1);
    expect(r.snapshot.reached).toBe(100);
    send(host, { type: 'step', n: 80 });
    expect(fp(host)).toBe(end);
  });

  it('branches on a page edit: pending entries and later keyframes go, reached drops', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 200 });
    send(host, { type: 'seek', tick: 70 });
    const r = send(host, place(6)) as { ok: true; snapshot: WorldSnapshot };
    expect(r.snapshot.reached).toBe(70);
    expect(host.keyframeTicks().every((t) => t <= 70)).toBe(true);
    expect((send(host, { type: 'seek', tick: 71 }) as { ok: false; errors: FieldError[] }).errors[0].field).toBe('seek');
  });

  it('keeps a keyframe taken before a same-tick edit valid', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 50 }); // keyframe at 50 taken with no edits
    send(host, place(7)); // an edit at tick 50, after the keyframe
    send(host, { type: 'step', n: 30 });
    const end = fp(host);
    send(host, { type: 'seek', tick: 50 });
    send(host, { type: 'seek', tick: 80 });
    expect(fp(host)).toBe(end);
  });

  it('keeps the followed agent followed', () => {
    const { host, sim } = setup();
    send(host, { type: 'follow', id: 1 });
    send(host, { type: 'step', n: 120 });
    send(host, { type: 'seek', tick: 10 });
    expect(sim().followed()).toBe(1);
  });

  it('refuses a seek past reached, a negative or fractional tick, and any seek with a full log', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 10 });
    for (const tick of [11, -1, 2.5]) {
      const r = send(host, { type: 'seek', tick }) as { ok: false; errors: FieldError[] };
      expect(r.ok).toBe(false);
      expect(r.errors[0].field).toBe('seek');
    }
    (host as unknown as { logFull: boolean }).logFull = true;
    expect(send(host, { type: 'seek', tick: 5 }).ok).toBe(false);
  });

  it('sends reached and seekable', () => {
    const { host } = setup();
    const r = send(host, { type: 'step', n: 10 }) as { ok: true; snapshot: WorldSnapshot };
    expect(r.snapshot.reached).toBe(10);
    const init = send(host, { type: 'refresh' }) as { ok: true; snapshot: WorldSnapshot };
    expect(init.snapshot.reached).toBeUndefined(); // unchanged: not resent
  });
});
```

Imports to add to the test file as needed: `type Command, type WorldSnapshot` from `./protocol`, `type FieldError` from `./types`, `FakeSim` from `./fake-sim.fixture`.

Note the `straight` helper and the seek tests reach into `host.sim` for the tick: if you prefer, add a public `get tick(): number` to `SimHost` (`return this.sim?.tick() ?? 0`) and use it instead of the cast.

- [ ] **Step 2: Run to verify they fail**

Run: `cd web && npx vitest run src/sim-host.test.ts -t "SimHost seek"`
Expected: FAIL (TypeScript error on `'seek'` command / unknown command).

- [ ] **Step 3: Implement the protocol**

In `protocol.ts` add to `WorldSnapshot` (after `forked?`):

```ts
  /** The furthest tick on this world's branch (the timeline's end): after init, reset, seek and on change. */
  reached?: number;
  /** Whether `seek` can rebuild this world exactly (false once the log is full): with `reached`. */
  seekable?: boolean;
```

and to `Command`:

```ts
  | { type: 'seek'; tick: number }
```

- [ ] **Step 4: Implement in the host**

Fields:

```ts
  /** What the world was built from (the `init`/`reset` command), to rebuild it for a seek. */
  private setup: { config: ModelConfig; seed: number; landscapes: (Uint8Array | null)[] } | null = null;
  /** The furthest tick on this branch; the `reached`/`seekable` last sent (-1: send with the next snapshot). */
  private reached = 0;
  private reachedSent = -1;
  private seekableSent: boolean | null = null;
```

Error constants next to `FULL`:

```ts
const seekError = (message: string) => JSON.stringify([{ field: 'seek', message }]);
```

In the `init`/`reset` branch, before creating the world keep the setup, and reset the counters (put these beside the other resets):

```ts
      this.setup = { config: cmd.config, seed: cmd.seed, landscapes: cmd.landscapes };
      this.reached = 0;
      this.reachedSent = -1;
      this.seekableSent = null;
```

and after `this.keyframe(next);` add `this.reached = next.tick();`.

In `advance`, after `left -= k;` add `this.reached = Math.max(this.reached, sim.tick());`.

In the edit case of `apply` (`setConfig`…`vaccinate`), after `this.fork();`:

```ts
        // A page edit starts a new branch here: the old future's keyframes and end go.
        this.dropKeyframes((t) => t <= sim.tick());
        this.reached = sim.tick();
```

Add the `seek` case to the `switch`:

```ts
      case 'seek':
        this.seek(sim, cmd.tick);
        this.configDue = true;
        this.landscapesDue = true;
        this.replaySent = -1;
        this.reachedSent = -1;
        // Every chart group's history changed: send them afresh.
        this.sent.clear();
        return this.reply(this.sim!, wants, frame);
```

and the method:

```ts
  /**
   * Moves the world to `tick` (≤ `reached`): forward by stepping; back by restoring the nearest
   * keyframe at or before it (or rebuilding the world) and replaying the log from there.
   */
  private seek(sim: SimLike, tick: number): void {
    if (this.logFull) throw seekError('this session’s log is full, so it can no longer be rebuilt exactly');
    if (!Number.isInteger(tick) || tick < 0 || tick > this.reached) {
      throw seekError(`tick ${tick} is not between 0 and ${this.reached}`);
    }
    const now = sim.tick();
    if (tick >= now) {
      this.advance(sim, tick - now);
      return;
    }
    const all = [...this.log, ...this.pending.slice(this.cursor)];
    const followed = sim.followed();
    const kf = this.keyframes.filter((k) => k.tick <= tick).at(-1);
    let world = sim;
    let applied = 0;
    if (kf) {
      sim.restore(kf.cp);
      applied = kf.applied;
    } else {
      const setup = this.setup!;
      world = this.module.create(JSON.stringify(setup.config), setup.seed, setup.landscapes);
      sim.free();
      this.sim = world;
    }
    this.log = all.slice(0, applied);
    this.pending = all.slice(applied);
    this.cursor = 0;
    if (world.followed() !== followed) {
      if (followed < 0) world.unfollow();
      else world.follow(followed);
    }
    this.replayDue(world);
    this.advance(world, tick - world.tick());
  }
```

`advance` raises `reached` with `Math.max`, so a seek never lowers it. The `reached`/`seekable` fields go into every snapshot when changed — in `snapshot()`, after the `replayLeft` block:

```ts
    if (this.reached !== this.reachedSent) {
      s.reached = this.reached;
      this.reachedSent = this.reached;
    }
    const seekable = !this.logFull;
    if (seekable !== this.seekableSent) {
      s.seekable = seekable;
      this.seekableSent = seekable;
    }
```

Also add `s.seekable`'s partner: when `reachedSent` is -1 (init/reset/seek) send both — set `this.seekableSent = null` in the seek case alongside `this.reachedSent = -1`.

Note: `replayDue` → `edit()` appends replayed entries back onto `this.log`; the page-edit branch (not `edit()`) is what drops keyframes and lowers `reached`, so replayed edits never do.

- [ ] **Step 5: Run the tests**

Run: `cd web && npx vitest run`
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src
git commit -m "Seek a world to any tick on its branch through keyframes and the edit log

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 5: Stop rules in the host

**Files:**
- Modify: `web/src/protocol.ts`, `web/src/sim-host.ts`
- Test: `web/src/sim-host.test.ts`

**Interfaces:**
- Produces: `export interface StopRules { tick?: number; when?: { series: string; op: '<' | '>'; value: number } }` in `protocol.ts`; `Command` member `{ type: 'setStops'; stops: StopRules }`; `WorldSnapshot.stopped?: string` (the reason, once); reason texts `Stopped at tick ${t}` and `Stopped at tick ${t}: ${series} ${op} ${value}`.

Semantics: rules are checked after every tick of `step` and of Max batches; a firing rule ends the step or the Max loop at that tick. "At tick N" fires when a step moves the world from below N to N. The condition fires only when it goes from false to true; its last truth is re-evaluated when the rules are set, after init/reset, after every edit and after a seek. Seeks never fire rules.

- [ ] **Step 1: Write the failing tests**

```ts
describe('SimHost stop rules', () => {
  const display = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() } as const;
  let id = 0;
  const send = (host: SimHost, cmd: Command) => host.handle({ id: ++id, cmd }).result as { ok: true; snapshot?: WorldSnapshot };
  const setup = (now?: () => number) => {
    const module = fakeModule();
    const host = new SimHost(module, now);
    send(host, { type: 'init', config: { width: 8, height: 3 } as never, seed: 1, landscapes: [], display });
    return { host, sim: () => module.sims.at(-1)! };
  };

  it('stops a step at tick N and says so', () => {
    const { host, sim } = setup();
    send(host, { type: 'setStops', stops: { tick: 37 } });
    const r = send(host, { type: 'step', n: 100 });
    expect(sim().ticks).toBe(37);
    expect(r.snapshot?.stopped).toBe('Stopped at tick 37');
    // Past N the rule is spent: the next step runs in full.
    expect(send(host, { type: 'step', n: 10 }).snapshot?.stopped).toBeUndefined();
    expect(sim().ticks).toBe(47);
  });

  it('fires a condition only when it becomes true', () => {
    const { host, sim } = setup();
    // One agent: population > 1 is false; place two more at tick 5 → true.
    send(host, { type: 'setStops', stops: { when: { series: 'population', op: '>', value: 1 } } });
    send(host, { type: 'step', n: 5 });
    send(host, { type: 'place', x: 3, y: 1, overrides: {} });
    // Already true after the edit: re-evaluated, so stepping does not fire.
    const r = send(host, { type: 'step', n: 20 });
    expect(r.snapshot?.stopped).toBeUndefined();
    expect(sim().ticks).toBe(25);
  });

  it('fires on a false → true transition at the exact tick', () => {
    const { host, sim } = setup();
    send(host, { type: 'setStops', stops: { when: { series: 'tick', op: '>', value: 41 } } });
    const r = send(host, { type: 'step', n: 100 });
    expect(sim().ticks).toBe(42);
    expect(r.snapshot?.stopped).toBe('Stopped at tick 42: tick > 41');
  });

  it('ends the Max loop on the tick a rule fires', () => {
    let t = 0;
    const { host, sim } = setup(() => (t += 1));
    send(host, { type: 'setStops', stops: { tick: 90 } });
    host.handle({ id: ++id, cmd: { type: 'run' }, frame: new ArrayBuffer(8 * 3 * 4) });
    let post: WorldSnapshot | null = null;
    for (let i = 0; i < 1000 && host.running; i++) post = host.batch() ?? post;
    expect(host.running).toBe(false);
    expect(sim().ticks).toBe(90);
    expect(post?.stopped).toBe('Stopped at tick 90');
  });

  it('does not fire during a seek', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 100 });
    send(host, { type: 'setStops', stops: { tick: 50 } });
    send(host, { type: 'seek', tick: 10 });
    const r = send(host, { type: 'seek', tick: 80 });
    expect(r.snapshot?.stopped).toBeUndefined();
  });
});
```

- [ ] **Step 2: Run to verify they fail**

Run: `cd web && npx vitest run src/sim-host.test.ts -t "stop rules"`
Expected: FAIL (unknown command `setStops`).

- [ ] **Step 3: Implement**

`protocol.ts`:

```ts
/** When a run stops by itself: at a tick, and/or when a series becomes less or greater than a value. */
export interface StopRules {
  tick?: number;
  when?: { series: string; op: '<' | '>'; value: number };
}
```

`Command`: `| { type: 'setStops'; stops: StopRules }`. `WorldSnapshot`: `/** Why the run just stopped by itself (a stop rule fired): once. */ stopped?: string;`

`sim-host.ts` fields:

```ts
  private stops: StopRules = {};
  /** Whether the stop condition held after the last tick (it fires only on becoming true). */
  private held = false;
  /** Why a rule stopped the run, for the next snapshot. */
  private stoppedDue: string | null = null;
  /** Seeks step without firing rules. */
  private seeking = false;
```

Helpers:

```ts
  private condition(sim: SimLike): boolean {
    const when = this.stops.when;
    if (!when) return false;
    const v = sim.latest_value(when.series);
    return v !== undefined && (when.op === '<' ? v < when.value : v > when.value);
  }

  /** After a tick from `from`: the reason a rule fires, or null. Keeps the condition's last truth. */
  private checkStops(sim: SimLike, from: number): string | null {
    const tick = sim.tick();
    const now = this.condition(sim);
    const fired = now && !this.held;
    this.held = now;
    const at = this.stops.tick;
    if (at !== undefined && from < at && tick >= at) return `Stopped at tick ${tick}`;
    if (fired) {
      const w = this.stops.when!;
      return `Stopped at tick ${tick}: ${w.series} ${w.op} ${w.value}`;
    }
    return null;
  }
```

Change `advance` to return whether a rule fired, stepping one tick at a time while a condition is set and never past a tick rule:

```ts
  /** … (existing doc) … Returns true if a stop rule ended it early (its reason is in `stoppedDue`). */
  private advance(sim: SimLike, n: number): boolean {
    let left = Math.min(n, MAX_TICKS - sim.tick());
    const rules = !this.seeking && (this.stops.tick !== undefined || this.stops.when !== undefined);
    while (left > 0) {
      const tick = sim.tick();
      const next = this.pending[this.cursor]?.tick;
      let k = next === undefined ? left : Math.min(left, Math.max(1, next - tick));
      if (!this.noKeyframes) k = Math.min(k, this.every - (tick % this.every));
      if (rules && this.stops.when) k = 1;
      const at = this.stops.tick;
      if (rules && at !== undefined && tick < at) k = Math.min(k, at - tick);
      sim.step(k);
      this.fired(tick, sim.tick());
      this.replayDue(sim);
      this.keyframe(sim);
      this.reached = Math.max(this.reached, sim.tick());
      left -= k;
      if (rules) {
        const reason = this.checkStops(sim, tick);
        if (reason) {
          this.stoppedDue = reason;
          return true;
        }
      }
    }
    return false;
  }
```

In `seek`, wrap the stepping: set `this.seeking = true` at the start of the method body and `finally { this.seeking = false; this.held = this.condition(this.sim!); }` (use try/finally around the body).

In `apply`: add

```ts
      case 'setStops':
        this.stops = cmd.stops;
        this.held = this.condition(sim);
        return this.reply(sim, wants, frame);
```

After the edit case's `this.edit(sim, cmd);` add `this.held = this.condition(sim);`; in the init/reset branch after `this.replayDue(next);` add `this.held = this.condition(next);`. Stop rules persist across init/reset (not cleared there).

In `batch()`: replace the loop and end check with

```ts
    let stopped = false;
    do stopped = this.advance(sim, 1);
    while (!stopped && this.now() < cap && sim.tick() < MAX_TICKS && !sim.finished());
    if (stopped || sim.tick() >= MAX_TICKS || sim.finished()) {
```

(the body of that `if` is unchanged: it ends the loop and posts with the last buffer).

In `snapshot()`, after the `forked` block:

```ts
    if (this.stoppedDue) {
      s.stopped = this.stoppedDue;
      this.stoppedDue = null;
    }
```

Edge: if `batch()` stops but no buffer is free (`max.pool.pop()` undefined) it returns null and keeps `max` set — the existing code already does this for the cap; the next `frame` command gives it a buffer and the following `batch()` posts. Since `advance` already fired, guard the next batch: at the top of the `do` loop the rule is spent (`held` true / tick past N), so it would keep running. Fix: store `private stopPending = false`; set it when `stopped` and no buffer; at the start of `batch()`, `if (this.stopPending)` skip stepping and go straight to the end-and-post branch. Add a test:

```ts
  it('ends Max on the stop tick even with no buffer free when the rule fires', () => {
    let t = 0;
    const { host, sim } = setup(() => (t += 1));
    send(host, { type: 'setStops', stops: { tick: 20 } });
    host.handle({ id: ++id, cmd: { type: 'run' } }); // no buffer lent
    for (let i = 0; i < 100; i++) host.batch();
    expect(sim().ticks).toBe(20);
    host.handle({ id: ++id, cmd: { type: 'frame' }, frame: new ArrayBuffer(8 * 3 * 4) });
    const post = host.batch();
    expect(post?.stopped).toBe('Stopped at tick 20');
    expect(host.running).toBe(false);
  });
```

- [ ] **Step 4: Run the tests**

Run: `cd web && npx vitest run`
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src
git commit -m "Stop a run at a tick or when a series crosses a value, on the exact tick

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 6: Engine: seek, stops, and the run controls

**Files:**
- Modify: `web/src/engine.ts`
- Test: `web/src/engine.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: `seek`/`setStops` commands and snapshot fields (Tasks 4–5), `StopRules`.
- Produces on `Engine`: `reached: number`, `seekable: boolean`, `stops: StopRules`, `lastStop: string | null`, `seek(tick: number): Promise<FieldError[] | null>`, `setStops(stops: StopRules): void`, event `'stopped'`. `RunControls` becomes:

```ts
export interface RunControls {
  readonly running: boolean;
  readonly speed: Speed;
  readonly tick: number;
  readonly reached: number;
  readonly seekable: boolean;
  /** Whether a condition rule (`StopRules.when`) is supported (false in Compare). */
  readonly conditionStops: boolean;
  readonly lastStop: string | null;
  setRunning(on: boolean): void;
  setSpeed(speed: Speed): void;
  advance(n?: number): Promise<void>;
  seek(tick: number): Promise<unknown>;
  setStops(stops: StopRules): void;
  on(event: 'run' | 'tick' | 'stopped', fn: () => void): () => void;
}
```

- [ ] **Step 1: Write the failing tests**

In `engine.test.ts` (inside `describe('Engine', …)`, using its `setup()`):

```ts
  it('seeks back and forward, keeping the selection and resending charts', async () => {
    const { engine } = await setup();
    await engine.select(1, 1);
    await engine.advance(120);
    expect(engine.reached).toBe(120);
    expect(engine.seekable).toBe(true);
    let configs = 0;
    engine.on('config', () => configs++);
    expect(await engine.seek(30)).toBeNull();
    expect(engine.tick).toBe(30);
    expect(engine.reached).toBe(120);
    expect(engine.selection).not.toBeNull();
    expect(configs).toBe(1);
    await engine.seek(120);
    expect(engine.tick).toBe(120);
  });

  it('returns the host’s errors for a refused seek', async () => {
    const { engine } = await setup();
    await engine.advance(5);
    const errors = await engine.seek(6);
    expect(errors?.[0].field).toBe('seek');
  });

  it('pauses and fires stopped when a stop rule fires', async () => {
    const { engine } = await setup();
    engine.setStops({ tick: 3 });
    engine.setSpeed(2);
    engine.setRunning(true);
    let stopped = 0;
    engine.on('stopped', () => stopped++);
    for (let i = 0; i < 5; i++) {
      engine.pump(i);
      await settle();
    }
    expect(engine.tick).toBe(3);
    expect(engine.running).toBe(false);
    expect(stopped).toBe(1);
    expect(engine.lastStop).toBe('Stopped at tick 3');
  });
```

And in `describe('Engine at Max speed', …)` (using its `maxSetup()`):

```ts
  it('pauses on a stop rule at Max', async () => {
    const { engine } = await maxSetup();
    engine.setStops({ tick: 40 });
    engine.setSpeed('max');
    engine.setRunning(true);
    while (engine.running) await wait(5);
    expect(engine.tick).toBe(40);
    expect(engine.lastStop).toBe('Stopped at tick 40');
  });
```

In `determinism.test.ts`, inside `describe('determinism through the engine', …)`:

```ts
  it('seeks through keyframes to the golden world, with edits on both sides of the target', async () => {
    const reference = await engine();
    await reference.advance(60);
    await reference.place(3, 3, {});
    await reference.advance(140);
    const want = await reference.fingerprint();

    const e = await engine();
    await e.advance(60);
    await e.place(3, 3, {});
    await e.advance(240);
    for (const t of [200, 61, 60, 59, 0]) {
      await e.seek(t);
      expect(e.tick).toBe(t);
    }
    await e.seek(0);
    await e.advance(200); // replays the edit at 60
    expect(await e.fingerprint()).toBe(want);
    await e.seek(150);
    await e.seek(200);
    expect(await e.fingerprint()).toBe(want);
  });

  it('shares a session taken after a seek back', async () => {
    const e = await engine();
    await e.advance(50);
    await e.place(4, 4, {});
    await e.advance(100);
    await e.seek(20);
    const token = encodeShare(await e.session());
    const opened = await Engine.create(decodeShare(token), { presets, transport: inline() });
    await opened.advance(150);
    await e.advance(130);
    expect(await opened.fingerprint()).toBe(await e.fingerprint());
  });
```

Check the exact names first: `grep -n "place(\|async session\|encodeShare\|decodeShare" web/src/engine.ts web/src/share.ts web/src/determinism.test.ts` — use the engine's actual method for placing an agent and for getting a session (the existing determinism test "replays a session recorded at mixed speeds, through a share link" shows both); adjust the calls above to those names, keeping the assertions.

- [ ] **Step 2: Run to verify they fail**

Run: `cd web && npx vitest run src/engine.test.ts`
Expected: FAIL (`engine.seek is not a function`).

- [ ] **Step 3: Implement**

Add `'stopped'` to `EngineEvent`. Import `type StopRules` from `./protocol`. Replace `RunControls` with the interface above. On `Engine` add fields:

```ts
  /** The furthest tick on this world's branch (the timeline's end). */
  reached = 0;
  /** Whether the world can seek (false once its log is full). */
  seekable = true;
  /** The stop rules sent to the host (kept across resets). */
  stops: StopRules = {};
  /** Why the run last stopped by itself, or null. */
  lastStop: string | null = null;
  readonly conditionStops = true;
```

In `adopt()`, after the `replayLeft` block:

```ts
    if (s.reached !== undefined) this.reached = s.reached;
    if (s.seekable !== undefined) this.seekable = s.seekable;
    if (s.stopped) this.lastStop = s.stopped;
```

In `accept()` (every snapshot, including Max posts, flows through it), after `this.announce(...)`:

```ts
    if (s.stopped) {
      if (this.running) this.setRunning(false);
      this.emit('stopped');
    }
```

Methods:

```ts
  /** Moves the world to `tick` on its branch (0 … `reached`); null, or the host's errors. */
  seek(tick: number): Promise<FieldError[] | null> {
    return this.quiet(async () => {
      const result = await this.send({ type: 'seek', tick }, true);
      if (!result.ok || !result.snapshot) return writeFailure(result);
      // The reply carries the config (fires 'config') and every chart group afresh.
      this.charts.clear();
      this.accept(result.snapshot, ['tick']);
      return null;
    });
  }

  /** Sets the rules that stop a run by itself; they hold until changed. */
  setStops(stops: StopRules): void {
    this.stops = structuredClone(stops);
    void this.send({ type: 'setStops', stops: this.stops });
  }
```

`writeFailure` is the existing helper used by `applyModelConfig`; check that it returns field errors for `{ ok: false, errors }`.

In `takeWorld`, copy the new fields too (`this.reached = other.reached; this.seekable = other.seekable; this.lastStop = other.lastStop;`) and then re-send this engine's rules to the adopted host: after the `quiet` block, `this.setStops(this.stops);`.

- [ ] **Step 4: Run the tests**

Run: `cd web && npm run wasm && npx vitest run && npx tsc --noEmit`
Expected: all PASS; `tsc` will report `Lockstep` no longer satisfying `RunControls` — that is Task 7. If it blocks `vitest`, it won't (Vitest does not type-check); proceed.

- [ ] **Step 5: Commit**

```bash
git add web/src
git commit -m "Seek and stop rules in the engine

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 7: Lockstep: seek and the tick rule in Compare

**Files:**
- Modify: `web/src/compare/lockstep.ts`
- Test: `web/src/compare/lockstep.test.ts`

**Interfaces:**
- Consumes: `Engine.seek`, `Engine.reached`, `Engine.seekable`, `Engine.setStops` (Task 6).
- Produces: `Lockstep` implements the full `RunControls`: `tick` (A's), `reached` (the smaller world's), `seekable` (both), `conditionStops = false`, `lastStop`, `seek(tick)`, `setStops(stops)` (keeps only `tick`), events `'stopped'`; `LockstepEvent` gains `'stopped'`.

- [ ] **Step 1: Write the failing tests**

```ts
describe('Lockstep seek and stops', () => {
  it('seeks both worlds to one tick', async () => {
    const { a, b, lock } = await pair(5);
    await lock.advance(80);
    expect(lock.reached).toBe(80);
    await lock.seek(33);
    expect([a.tick, b.tick]).toEqual([33, 33]);
    await lock.seek(80);
    expect([a.tick, b.tick]).toEqual([80, 80]);
  });

  it('stops both at tick N and ignores a condition', async () => {
    const { a, b, lock } = await pair(7);
    lock.setStops({ tick: 20, when: { series: 'population', op: '<', value: 99 } });
    let stopped = 0;
    lock.on('stopped', () => stopped++);
    lock.setRunning(true);
    await frames(lock, 10);
    expect([a.tick, b.tick]).toEqual([20, 20]);
    expect(lock.running).toBe(false);
    expect(stopped).toBe(1);
    expect(lock.lastStop).toBe('Stopped at tick 20');
  });

  it('clears the worlds’ own rules, so neither host stops alone', async () => {
    const a = await create({ config, seed: 1 });
    a.setStops({ tick: 3 });
    const b = await create({ config, seed: 2 });
    const lock = new Lockstep([a, b], 5);
    await lock.settled();
    await lock.advance(10);
    expect([a.tick, b.tick]).toEqual([10, 10]);
  });
});
```

- [ ] **Step 2: Run to verify they fail**

Run: `cd web && npx vitest run src/compare/lockstep.test.ts`
Expected: FAIL (`lock.seek is not a function`).

- [ ] **Step 3: Implement**

`LockstepEvent = 'run' | 'tick' | 'finished' | 'stopped'`. Fields and getters:

```ts
  readonly conditionStops = false;
  lastStop: string | null = null;
  /** Only the tick rule applies in Compare (Decision: conditions are single-world). */
  private stopAt: number | undefined;

  get tick(): number {
    return this.worlds[0].tick;
  }
  get reached(): number {
    return Math.min(...this.worlds.map((w) => w.reached));
  }
  get seekable(): boolean {
    return this.worlds.every((w) => w.seekable);
  }
```

In the constructor loop, after `w.setRunning(false);` add `w.setStops({});` (the engines keep no rules of their own while in Compare; the toolbar re-applies its rules to whichever controls it drives — Task 9).

Methods:

```ts
  setStops(stops: StopRules): void {
    this.stopAt = stops.tick;
  }

  /** Both worlds to `tick`; throws if either refuses. */
  seek(tick: number): Promise<void> {
    return this.exclusive(async () => {
      if (this.crashed()) return;
      const errors = await Promise.all(this.worlds.map((w) => w.seek(tick)));
      this.emit('tick');
      const failed = errors.flatMap((e) => e ?? []);
      if (failed.length > 0) throw new Error(`a world could not seek: ${fieldErrorsMessage(failed)}`);
    });
  }
```

In `pump`, after computing `n`:

```ts
    const at = this.stopAt;
    const tick = this.tick;
    const capped = at !== undefined && tick < at ? Math.min(n, at - tick) : n;
    this.inFlight = this.stepBoth(capped, speed === 'max')
      .then(() => {
        if (at !== undefined && tick < at && this.tick >= at) {
          this.lastStop = `Stopped at tick ${this.tick}`;
          this.setRunning(false);
          this.emit('stopped');
        }
      })
      .finally(() => (this.inFlight = null));
```

(replacing the existing `this.inFlight = this.stepBoth(n, …)…` line; `speed` is the local from the slow-speed change). Import `type StopRules` from `../protocol`.

- [ ] **Step 4: Run the tests and type-check**

Run: `cd web && npx vitest run && npx tsc --noEmit`
Expected: all PASS; no type errors.

- [ ] **Step 5: Commit**

```bash
git add web/src
git commit -m "Seek both worlds and stop at a tick in Compare

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 8: The timeline (⟲1 and the slider)

**Files:**
- Create: `web/src/ui/timeline.ts`, `web/src/ui/timeline.test.ts`
- Modify: `web/src/ui/toolbar.ts`, `web/src/main.ts` (notices), `web/src/style.css`

**Interfaces:**
- Consumes: `RunControls.seek/reached/seekable/tick` (Tasks 6–7).
- Produces: `class SeekQueue { constructor(seek: (t: number) => Promise<unknown>); request(t: number): void; settled(): Promise<void> }`; `class Timeline { readonly el: HTMLElement; constructor(onError: (e: unknown) => void); bind(controls: RunControls): void; sync(): void; back(): void }`.

- [ ] **Step 1: Write the failing test** (`web/src/ui/timeline.test.ts`)

```ts
import { describe, expect, it } from 'vitest';
import { SeekQueue } from './timeline';

describe('SeekQueue', () => {
  it('keeps one seek in flight and sends only the newest position after it', async () => {
    const sent: number[] = [];
    const gates: (() => void)[] = [];
    const q = new SeekQueue((t) => {
      sent.push(t);
      return new Promise<void>((resolve) => gates.push(resolve));
    });
    q.request(10);
    q.request(20);
    q.request(30);
    q.request(40);
    expect(sent).toEqual([10]);
    gates.shift()!();
    await Promise.resolve();
    await Promise.resolve();
    expect(sent).toEqual([10, 40]);
    gates.shift()!();
    await q.settled();
    expect(sent).toEqual([10, 40]);
  });

  it('keeps going after a failed seek', async () => {
    const sent: number[] = [];
    const q = new SeekQueue(async (t) => {
      sent.push(t);
      if (t === 1) throw new Error('no');
    });
    q.request(1);
    q.request(2);
    await q.settled();
    expect(sent).toEqual([1, 2]);
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd web && npx vitest run src/ui/timeline.test.ts`
Expected: FAIL (cannot find `./timeline`).

- [ ] **Step 3: Implement `web/src/ui/timeline.ts`**

```ts
import type { RunControls } from '../engine';
import { h } from './dom';

/** One seek at a time; while one is in flight, only the newest request waits to follow it (a fast drag). */
export class SeekQueue {
  private busy: Promise<void> | null = null;
  private next: number | null = null;

  constructor(
    private readonly seek: (tick: number) => Promise<unknown>,
    private readonly onError: (e: unknown) => void = () => {},
  ) {}

  request(tick: number): void {
    this.next = tick;
    if (!this.busy) this.busy = this.drain();
  }

  /** Resolves once every request so far has been sent and answered. */
  async settled(): Promise<void> {
    while (this.busy) await this.busy;
  }

  private async drain(): Promise<void> {
    while (this.next !== null) {
      const t = this.next;
      this.next = null;
      try {
        await this.seek(t);
      } catch (e) {
        this.onError(e);
      }
    }
    this.busy = null;
  }
}

/** ⟲1 and a slider over the ticks this world's branch has reached (0 … reached). */
export class Timeline {
  readonly el: HTMLElement;
  private controls: RunControls | null = null;
  private readonly backButton: HTMLButtonElement;
  private readonly slider: HTMLInputElement;
  private readonly queue: SeekQueue;
  private dragging = false;
  held = false;

  constructor(onError: (e: unknown) => void) {
    this.queue = new SeekQueue((t) => this.controls!.seek(t), onError);
    this.backButton = h('button', { title: 'Back one tick (←)', 'aria-label': 'Back one tick', onclick: () => this.back() }, '⟲1');
    this.slider = h('input', { type: 'range', min: 0, max: 0, step: 1, class: 'timeline', 'aria-label': 'Tick' });
    this.slider.addEventListener('pointerdown', () => (this.dragging = true));
    this.slider.addEventListener('pointerup', () => (this.dragging = false));
    this.slider.addEventListener('input', () => {
      if (this.controls?.running) this.controls.setRunning(false);
      this.queue.request(Number(this.slider.value));
    });
    this.el = h('div', { class: 'group timeline-group' }, this.backButton, this.slider);
  }

  bind(controls: RunControls): void {
    this.controls = controls;
    this.sync();
  }

  back(): void {
    const c = this.controls;
    if (!c || this.held || !c.seekable || c.tick === 0) return;
    if (c.running) c.setRunning(false);
    this.queue.request(c.tick - 1);
  }

  sync(): void {
    const c = this.controls;
    if (!c) return;
    const why = c.seekable ? '' : 'This session’s log is full, so it can no longer be rebuilt exactly';
    this.slider.max = String(c.reached);
    // Leave the thumb where the user is dragging it.
    if (!this.dragging) this.slider.value = String(c.tick);
    this.slider.disabled = this.held || !c.seekable;
    this.slider.title = why || `Tick ${c.tick} of ${c.reached} — drag to go back and forth`;
    this.backButton.disabled = this.held || !c.seekable || c.tick === 0;
    if (why) this.backButton.title = why;
  }
}
```

Check `h`'s attribute handling in `ui/dom.ts` accepts numbers (it stringifies non-boolean values — line 19).

- [ ] **Step 4: Wire it into the toolbar**

In `toolbar.ts`: `import { Timeline } from './timeline';`, add a field `private readonly timeline = new Timeline((e) => showNotice(\`Could not go to that tick (${errorMessage(e)})\`, 10_000));` Put `this.timeline.el` in the toolbar after the play/step/speed group. In the constructor after `this.controls = engine;` call `this.timeline.bind(engine)`. In `setCompare`, after `this.controls = lock ?? this.engine;` call `this.timeline.bind(this.controls)` and push `lock.on('tick', () => this.timeline.sync())` when `lock`. In `sync()` add `this.timeline.held = this.held; this.timeline.sync();`. In `tick()` add `this.timeline.sync();`. Export a method `back(): void { this.timeline.back(); }` (for shortcuts, Task 11).

Engine seek replies fire `'tick'` (Task 6), so the existing `engine.on('tick', …)` keeps the slider current; `engine.on('snapshot', () => this.timeline.sync())` also covers `reached`/`seekable` arriving on edits.

`style.css`: add

```css
.timeline-group { flex: 1 1 12rem; min-width: 8rem; }
.timeline { flex: 1; min-width: 6rem; }
```

- [ ] **Step 5: Verify**

Run: `cd web && npx vitest run && npx tsc --noEmit`
Expected: PASS. Then run the app (`npm run dev` from `web/`), load II-2, Play at 5× to ~300, Pause, drag the slider back and forth, press ⟲1: the grid, tick readout and charts move back; Play resumes forward.

- [ ] **Step 6: Commit**

```bash
git add web/src
git commit -m "Add the timeline: back one tick and a slider over the run

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 9: The Stop at control

**Files:**
- Create: `web/src/ui/stop-control.ts`, `web/src/ui/stop-control.test.ts`
- Modify: `web/src/ui/toolbar.ts`, `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `RunControls.setStops/conditionStops/lastStop`, event `'stopped'`; `StopRules`.
- Produces: `parseStopRules(input: { tick: string; series: string; op: string; value: string }): StopRules | string` (a rules object, or an error message); `class StopControl { readonly el: HTMLElement; constructor(engine: Engine); bind(controls: RunControls): void; clear(): void }`.

The series list is the current model's chartable series: `engine.config` → the names the Charts panel offers. Find the helper with `grep -n "export function\|export const" web/src/ui/series-data.ts web/src/models.ts | grep -i series` (e.g. `MODEL_CHARTS` in `series-data.ts`, used by `determinism.test.ts`) and use the one that lists series names for a config; include `population` first.

- [ ] **Step 1: Write the failing test** (`web/src/ui/stop-control.test.ts`)

```ts
import { describe, expect, it } from 'vitest';
import { parseStopRules } from './stop-control';

describe('parseStopRules', () => {
  const blank = { tick: '', series: 'population', op: '<', value: '' };
  it('makes no rules from blank fields', () => {
    expect(parseStopRules(blank)).toEqual({});
  });
  it('reads a tick and a condition', () => {
    expect(parseStopRules({ tick: '500', series: 'gini', op: '>', value: '0.5' })).toEqual({
      tick: 500,
      when: { series: 'gini', op: '>', value: 0.5 },
    });
  });
  it('refuses a negative or fractional tick and a non-number value', () => {
    expect(typeof parseStopRules({ ...blank, tick: '-3' })).toBe('string');
    expect(typeof parseStopRules({ ...blank, tick: '2.5' })).toBe('string');
    expect(typeof parseStopRules({ ...blank, value: 'lots' })).toBe('string');
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd web && npx vitest run src/ui/stop-control.test.ts`
Expected: FAIL (cannot find module).

- [ ] **Step 3: Implement `web/src/ui/stop-control.ts`**

```ts
import type { Engine, RunControls } from '../engine';
import type { StopRules } from '../protocol';
import { h } from './dom';

/** The rules the fields describe, or what is wrong with them. Blank fields make no rule. */
export function parseStopRules(input: { tick: string; series: string; op: string; value: string }): StopRules | string {
  const rules: StopRules = {};
  const tick = input.tick.trim();
  if (tick !== '') {
    const t = Number(tick);
    if (!Number.isInteger(t) || t < 0) return 'The tick must be a whole number, 0 or more';
    rules.tick = t;
  }
  const value = input.value.trim();
  if (value !== '') {
    const v = Number(value);
    if (!Number.isFinite(v)) return 'The value must be a number';
    rules.when = { series: input.series, op: input.op === '>' ? '>' : '<', value: v };
  }
  return rules;
}

/** "Stop at tick [ ] · when [series] [<|>] [ ]": sets the run controls' stop rules as they are edited. */
export class StopControl {
  readonly el: HTMLElement;
  private controls: RunControls;
  private readonly tick = h('input', { type: 'number', min: 0, step: 1, class: 'stop-tick', placeholder: 'tick', 'aria-label': 'Stop at tick' });
  private readonly series = h('select', { 'aria-label': 'Series' });
  private readonly op = h('select', { 'aria-label': 'Comparison' }, h('option', { value: '<' }, '<'), h('option', { value: '>' }, '>'));
  private readonly value = h('input', { type: 'number', step: 'any', class: 'stop-value', placeholder: 'value', 'aria-label': 'Value' });
  private readonly when: HTMLElement;
  private readonly error = h('span', { class: 'stop-error', role: 'alert' });
  private model: string;

  constructor(private readonly engine: Engine) {
    this.controls = engine;
    this.model = engine.model;
    this.when = h('span', { class: 'stop-when' }, ' when ', this.series, this.op, this.value);
    for (const el of [this.tick, this.value]) el.addEventListener('change', () => this.apply());
    for (const el of [this.series, this.op]) el.addEventListener('change', () => this.apply());
    this.el = h('div', { class: 'group stop-control', title: 'Pause by itself at a tick, or when a series crosses a value' }, 'Stop at ', this.tick, this.when, this.error);
    this.fillSeries();
    // A new model has other series: its rules go.
    engine.on('reset', () => {
      if (engine.model !== this.model) {
        this.model = engine.model;
        this.fillSeries();
        this.clear();
      }
    });
  }

  bind(controls: RunControls): void {
    this.controls = controls;
    const on = controls.conditionStops;
    this.series.disabled = this.op.disabled = this.value.disabled = !on;
    this.when.title = on ? '' : 'In Compare a run stops only at a tick: the two worlds could meet a condition on different ticks';
    this.apply();
  }

  clear(): void {
    this.tick.value = '';
    this.value.value = '';
    this.apply();
  }

  private apply(): void {
    const rules = parseStopRules({ tick: this.tick.value, series: this.series.value, op: this.op.value, value: this.value.value });
    this.error.textContent = typeof rules === 'string' ? rules : '';
    if (typeof rules === 'string') return;
    if (!this.controls.conditionStops) delete rules.when;
    this.controls.setStops(rules);
  }

  private fillSeries(): void {
    const names = seriesNames(this.engine);
    this.series.replaceChildren(...names.map((n) => h('option', { value: n }, n)));
  }
}
```

Implement `seriesNames(engine: Engine): string[]` at the bottom of the file from the helper found above (population first, no duplicates).

- [ ] **Step 4: Wire it in**

`toolbar.ts`: create `private readonly stopControl = new StopControl(engine);` in the constructor (after `this.controls = engine`), place `this.stopControl.el` after the timeline group; in `setCompare` call `this.stopControl.bind(this.controls)`; in `sync()` set `this.stopControl.el.inert = this.held`.

`main.ts`, next to the `'finished'` notice (line ~255):

```ts
  engine.on('stopped', () => showNotice(engine.lastStop ?? 'Stopped', 10_000));
```

and where Compare's lockstep is created (`compare-view.ts` sets up listeners on `lock` — find its `lock.on('finished', …)` and add the same for `'stopped'` with `lock.lastStop`).

`style.css`:

```css
.stop-control input { width: 5rem; }
.stop-error { color: var(--danger, #b00020); margin-left: 0.5rem; }
```

(use the stylesheet's existing error color token if there is one: `grep -n "error\|danger" web/src/style.css`).

- [ ] **Step 5: Verify**

Run: `cd web && npx vitest run && npx tsc --noEmit`
Expected: PASS. In the app: set "Stop at 200", Play at 25× → pauses at 200 with the notice; set "when population < 300" on II-2 at Max → pauses on the tick the population drops below 300.

- [ ] **Step 6: Commit**

```bash
git add web/src
git commit -m "Add the Stop at control: a tick and a series threshold

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 10: Ticks-per-second readout

**Files:**
- Create: `web/src/ui/rate.ts`, `web/src/ui/rate.test.ts`
- Modify: `web/src/ui/toolbar.ts`, `web/src/style.css`

**Interfaces:**
- Produces: `class RateMeter { constructor(windowMs = 1000); sample(now: number, tick: number): void; rate(): number | null; reset(): void }` — `rate()` is ticks per second over the samples within the window, null with fewer than two samples or no elapsed time; a tick lower than the previous sample (a seek or reset) resets the meter.

- [ ] **Step 1: Write the failing test** (`web/src/ui/rate.test.ts`)

```ts
import { describe, expect, it } from 'vitest';
import { RateMeter } from './rate';

describe('RateMeter', () => {
  it('measures ticks per second over the last window', () => {
    const m = new RateMeter(1000);
    expect(m.rate()).toBeNull();
    m.sample(0, 0);
    m.sample(250, 15);
    m.sample(500, 30);
    expect(m.rate()).toBeCloseTo(60);
    m.sample(1500, 40); // older samples fall out of the window
    expect(m.rate()).toBeCloseTo(10);
  });
  it('starts over when the tick goes back', () => {
    const m = new RateMeter();
    m.sample(0, 100);
    m.sample(250, 200);
    m.sample(500, 5);
    expect(m.rate()).toBeNull();
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd web && npx vitest run src/ui/rate.test.ts`
Expected: FAIL (cannot find module).

- [ ] **Step 3: Implement `web/src/ui/rate.ts`**

```ts
/** Ticks per second, measured from (time, tick) samples over a sliding window. */
export class RateMeter {
  private samples: { now: number; tick: number }[] = [];

  constructor(private readonly windowMs = 1000) {}

  sample(now: number, tick: number): void {
    const last = this.samples.at(-1);
    if (last && tick < last.tick) this.samples = [];
    this.samples.push({ now, tick });
    // Keep one sample at or beyond the window's start, so the window is always covered.
    while (this.samples.length > 2 && this.samples[1].now <= now - this.windowMs) this.samples.shift();
  }

  rate(): number | null {
    const first = this.samples[0];
    const last = this.samples.at(-1);
    if (!first || !last || last.now <= first.now) return null;
    return ((last.tick - first.tick) * 1000) / (last.now - first.now);
  }

  reset(): void {
    this.samples = [];
  }
}
```

Check the first test by hand: after the sample at 1500 the window starts at 500, so samples before it (0, 250) drop while `samples[1].now <= 500` — (0) drops, then (250) drops since samples[1] = (500) ≤ 500 — leaving (500,30),(1500,40): 10 ticks/s. ✓

- [ ] **Step 4: Wire it in**

`toolbar.ts`: a `private readonly rate = h('span', { class: 'rate', title: 'Measured ticks per second' });` placed right after `this.speed` in the play group, a `private readonly meter = new RateMeter();` and a timer started in the constructor:

```ts
    setInterval(() => {
      if (!this.controls.running) {
        this.meter.reset();
        this.rate.hidden = true;
        return;
      }
      this.meter.sample(performance.now(), this.controls.tick);
      const r = this.meter.rate();
      this.rate.hidden = r === null;
      if (r !== null) this.rate.textContent = `${r < 10 ? r.toFixed(1) : Math.round(r).toLocaleString()} t/s`;
    }, 250);
```

and `this.meter.reset()` in `setCompare`. `style.css`: `.rate { font-variant-numeric: tabular-nums; min-width: 5.5rem; color: var(--muted, #666); }` (reuse the stylesheet's muted-text token if it has one).

- [ ] **Step 5: Verify**

Run: `cd web && npx vitest run && npx tsc --noEmit`
Expected: PASS. In the app, 1/s shows ~1.0 t/s, 10× shows ~600 t/s on a 60 Hz display, Max shows its rate; hidden while paused.

- [ ] **Step 6: Commit**

```bash
git add web/src
git commit -m "Show measured ticks per second beside the speed menu

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 11: Keyboard shortcuts

**Files:**
- Create: `web/src/ui/shortcuts.ts`, `web/src/ui/shortcuts.test.ts`
- Modify: `web/src/ui/toolbar.ts` (export `SPEEDS`; expose `playPause()`, `stepOnce()`, `back()`, `slower()`, `faster()`, `reset()` as public), `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Produces: `type Shortcut = 'play' | 'step' | 'back' | 'slower' | 'faster' | 'reset' | 'help'`; `shortcutFor(e: { key: string; ctrlKey: boolean; altKey: boolean; metaKey: boolean; target: { tagName?: string; isContentEditable?: boolean } | null }): Shortcut | null`; `nextSpeed(speeds: Speed[], current: Speed, dir: -1 | 1): Speed`; `installShortcuts(toolbar: Toolbar): () => void`.

- [ ] **Step 1: Write the failing test** (`web/src/ui/shortcuts.test.ts`)

```ts
import { describe, expect, it } from 'vitest';
import { nextSpeed, shortcutFor } from './shortcuts';

const key = (k: string, target: { tagName?: string; isContentEditable?: boolean } | null = { tagName: 'BODY' }, mods = {}) => ({
  key: k,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
  target,
  ...mods,
});

describe('shortcutFor', () => {
  it('maps the keys', () => {
    expect(shortcutFor(key(' '))).toBe('play');
    expect(shortcutFor(key('ArrowRight'))).toBe('step');
    expect(shortcutFor(key('ArrowLeft'))).toBe('back');
    expect(shortcutFor(key('['))).toBe('slower');
    expect(shortcutFor(key(']'))).toBe('faster');
    expect(shortcutFor(key('r'))).toBe('reset');
    expect(shortcutFor(key('R'))).toBe('reset');
    expect(shortcutFor(key('?'))).toBe('help');
    expect(shortcutFor(key('x'))).toBeNull();
  });
  it('leaves typing alone', () => {
    for (const tagName of ['INPUT', 'SELECT', 'TEXTAREA']) expect(shortcutFor(key(' ', { tagName }))).toBeNull();
    expect(shortcutFor(key(' ', { tagName: 'DIV', isContentEditable: true }))).toBeNull();
  });
  it('leaves modified keys alone', () => {
    expect(shortcutFor(key('r', undefined, { metaKey: true }))).toBeNull();
    expect(shortcutFor(key('r', undefined, { ctrlKey: true }))).toBeNull();
    expect(shortcutFor(key('ArrowLeft', undefined, { altKey: true }))).toBeNull();
  });
});

describe('nextSpeed', () => {
  const speeds = [1 / 60, 1, 5, 'max'] as const;
  it('walks the list and stops at its ends', () => {
    expect(nextSpeed([...speeds], 1, 1)).toBe(5);
    expect(nextSpeed([...speeds], 1, -1)).toBe(1 / 60);
    expect(nextSpeed([...speeds], 'max', 1)).toBe('max');
    expect(nextSpeed([...speeds], 1 / 60, -1)).toBe(1 / 60);
  });
  it('snaps a speed not in the list to the nearest step in that direction', () => {
    expect(nextSpeed([...speeds], 3, 1)).toBe(5);
    expect(nextSpeed([...speeds], 3, -1)).toBe(1);
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd web && npx vitest run src/ui/shortcuts.test.ts`
Expected: FAIL (cannot find module).

- [ ] **Step 3: Implement `web/src/ui/shortcuts.ts`**

```ts
import type { Speed } from '../engine';
import { h } from './dom';
import type { Toolbar } from './toolbar';

export type Shortcut = 'play' | 'step' | 'back' | 'slower' | 'faster' | 'reset' | 'help';

const KEYS: Record<string, Shortcut> = {
  ' ': 'play',
  ArrowRight: 'step',
  ArrowLeft: 'back',
  '[': 'slower',
  ']': 'faster',
  r: 'reset',
  R: 'reset',
  '?': 'help',
};

const HELP: [string, string][] = [
  ['Space', 'Play / Pause'],
  ['→', 'Step one tick'],
  ['←', 'Back one tick'],
  ['[ / ]', 'Slower / faster'],
  ['R', 'Reset'],
  ['?', 'Show / hide this card'],
];

const TYPING = new Set(['INPUT', 'SELECT', 'TEXTAREA']);

/** The shortcut a key press means, or null (typing in a field, a modifier held, another key). */
export function shortcutFor(e: {
  key: string;
  ctrlKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  target: { tagName?: string; isContentEditable?: boolean } | null;
}): Shortcut | null {
  if (e.ctrlKey || e.altKey || e.metaKey) return null;
  const t = e.target;
  if (t && (TYPING.has(t.tagName ?? '') || t.isContentEditable)) return null;
  return KEYS[e.key] ?? null;
}

const rank = (s: Speed): number => (s === 'max' ? Infinity : s);

/** The speed one step slower (−1) or faster (1) than `current` in `speeds` (ascending), stopping at the ends. */
export function nextSpeed(speeds: Speed[], current: Speed, dir: -1 | 1): Speed {
  const r = rank(current);
  if (dir === 1) return speeds.find((s) => rank(s) > r) ?? speeds[speeds.length - 1];
  return [...speeds].reverse().find((s) => rank(s) < r) ?? speeds[0];
}

/** Listens for the shortcuts on the document; returns the removal. */
export function installShortcuts(toolbar: Toolbar): () => void {
  const card = h(
    'div',
    { class: 'shortcuts-card', role: 'dialog', 'aria-label': 'Keyboard shortcuts', hidden: true },
    h('h2', {}, 'Keyboard shortcuts'),
    h('dl', {}, ...HELP.flatMap(([k, what]) => [h('dt', {}, h('kbd', {}, k)), h('dd', {}, what)])),
  );
  document.body.append(card);
  const onKey = (e: KeyboardEvent): void => {
    if (e.key === 'Escape' && !card.hidden) {
      card.hidden = true;
      return;
    }
    const s = shortcutFor({ key: e.key, ctrlKey: e.ctrlKey, altKey: e.altKey, metaKey: e.metaKey, target: e.target as HTMLElement | null });
    if (!s) return;
    e.preventDefault();
    if (s === 'help') card.hidden = !card.hidden;
    else toolbar.shortcut(s);
  };
  document.addEventListener('keydown', onKey);
  return () => {
    document.removeEventListener('keydown', onKey);
    card.remove();
  };
}
```

- [ ] **Step 4: Toolbar entry point**

In `toolbar.ts`: `export const SPEEDS` (it is `const SPEEDS` now), import `nextSpeed` and `type Shortcut` from `./shortcuts`, and add:

```ts
  /** A keyboard shortcut (ui/shortcuts.ts): does what its button does, and nothing while held. */
  shortcut(s: Exclude<Shortcut, 'help'>): void {
    if (this.held) return;
    const c = this.controls;
    switch (s) {
      case 'play':
        c.setRunning(!c.running);
        break;
      case 'step':
        if (!c.running) this.step.click();
        break;
      case 'back':
        this.timeline.back();
        break;
      case 'slower':
      case 'faster': {
        const next = nextSpeed(SPEEDS, c.speed, s === 'faster' ? 1 : -1);
        c.setSpeed(next);
        this.speed.value = String(next);
        break;
      }
      case 'reset':
        this.reset();
        break;
    }
  }
```

`main.ts`: after `document.querySelector('#toolbar')!.append(toolbar.el);` add `installShortcuts(toolbar);` (import from `./ui/shortcuts`). Add a `?` hint: give the toolbar's `h1` a `title: 'Press ? for keyboard shortcuts'`.

`style.css`:

```css
.shortcuts-card {
  position: fixed; right: 1rem; bottom: 1rem; z-index: 20;
  background: var(--panel, #fff); border: 1px solid var(--border, #ccc); border-radius: 6px;
  padding: 0.75rem 1rem; box-shadow: 0 4px 16px rgb(0 0 0 / 0.2);
}
.shortcuts-card h2 { margin: 0 0 0.5rem; font-size: 1rem; }
.shortcuts-card dl { display: grid; grid-template-columns: auto 1fr; gap: 0.25rem 0.75rem; margin: 0; }
.shortcuts-card dd { margin: 0; }
```

(use the stylesheet's existing panel/border tokens: `grep -n "^\s*--" web/src/style.css`).

- [ ] **Step 5: Verify**

Run: `cd web && npx vitest run && npx tsc --noEmit`
Expected: PASS. In the app: Space toggles Play; typing a space in the seed box does not; `]` twice moves 1× → 5×; ← steps back; `?` shows the card, Esc hides it.

- [ ] **Step 6: Commit**

```bash
git add web/src
git commit -m "Drive the toolbar from the keyboard; ? shows the shortcuts

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```

---

### Task 12: Docs and full verification

**Files:**
- Modify: `README.md` (the playground section), `docs/roadmap.md`

- [ ] **Step 1: Docs**

README: in the playground feature list, add one paragraph each for the timeline ("⟲1 steps back a tick and the slider moves anywhere on the run so far; playing on replays the same future until you edit, which starts a new branch"), Stop at ("a tick and/or a series threshold; checked every tick at every speed; in Compare only the tick"), the ticks-per-second readout, and a shortcuts table (the one in the spec). Roadmap: add under the milestones

```markdown
## Playground controls (done)

Step back and a timeline slider (keyframes in the worker, replay of the edit log), stop rules (at a
tick, or when a series crosses a value, on the exact tick at every speed), a measured ticks-per-second
readout, and keyboard shortcuts. Runs are unchanged. See
`docs/superpowers/specs/2026-09-25-playground-controls-design.md`.
```

and under "Playground and infrastructure": `- **Step back / timeline, stop rules, shortcuts**: done (Playground controls).`

- [ ] **Step 2: Full verification**

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cd web && npm run build && npm test
```

Expected: every command succeeds; golden/legacy tests unchanged and passing.

- [ ] **Step 3: Commit**

```bash
git add README.md docs/roadmap.md
git commit -m "Document the timeline, stop rules, rate readout and shortcuts

Claude-Session: https://claude.ai/code/session_01NLZURonhfRemPscMkS9uVc"
```
