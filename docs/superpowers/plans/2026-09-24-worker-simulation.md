# Worker Simulation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the playground's simulation off the main thread into a module Web Worker, add a Max speed, and keep charts fast on long runs by drawing Largest-Triangle-Three-Buckets–downsampled history — without changing what the simulation computes or what the user can do.

**Architecture:** One message handler, `web/src/sim-host.ts` (`SimHost` + `serve`), holds the WASM `Sim` and answers typed commands (`web/src/protocol.ts`) one at a time with `{ ok, snapshot? } | { ok: false, errors } | { ok: false, fatal }`. It runs in `web/src/sim-worker.ts` (the app) or on the page through `InlineTransport` (tests and the fallback when a module worker cannot start); both sit behind the `Transport` interface (`web/src/transport.ts`), whose requests carry an id, the current *wants* and a lent frame buffer. `web/src/engine.ts` keeps its public shape: reads are synchronous fields served from the latest `WorldSnapshot`, writes return promises, events fire when replies arrive, panels declare what they need with `engine.want(provider)`, and `engine.sim` disappears. Two frame buffers ping-pong between engine and host. The core gains `stats::downsample` (LTTB) and `stats::downsample_union`; WASM gains `Sim.series_downsampled`, `Sim.series_group` and `Sim.fingerprint`; the Charts panel draws the host's downsampled groups with real ticks as x. Max speed is a host loop of ~16 ms batches posting a snapshot about every 33 ms while it holds a free buffer.

**Tech Stack:** Rust (`sugarscape-core`, `sugarscape-wasm`), wasm-bindgen, Vite (`worker.format: 'es'`) + TypeScript + uPlot + Vitest. No new dependencies.

**Spec:** docs/superpowers/specs/2026-09-24-worker-simulation-design.md

## Global Constraints

- **No simulation change.** `crates/sugarscape-core/tests/golden.rs`, `tests/legacy.rs` and `tests/fixtures/*` are never edited and pass after every task. The only core change is two new pure functions in `stats.rs` (Task 1); the WASM additions only read the world (Task 2). The worker runs the same `Sim`, stepping it with the same `step(n)` calls; Task 7 proves through the engine, with the real WASM, that `ii-2-unit` from seed 1 reaches the golden fingerprint after 200 ticks however the ticks are split into frames, and Task 11 has the controller repeat it through the worker.
- **The app builds and works after every task.** Tasks 1–5 add code nothing uses yet. Task 6 swaps the engine onto a transport (still on the page) and keeps a temporary, clearly marked `engine.sim` escape hatch plus three deprecated sync helpers so the panels not yet migrated keep working; Tasks 8–10 move them area by area and Task 10 deletes the hatch. Task 11 moves the host into the worker; Tasks 12–13 add Max.
- **Protocol names are binding across tasks:** `HostRequest`, `HostReply`, `HostMessage`, `Result`, `Command`, `Wants`, `WorldSnapshot`, `Selected`, `SelectQuery`, `ChartGroup`, `DisplayState`, `Overlay`, `PlaceOverrides`, `chartKey`, `mergeWants`, `transfers`, `CHART_POINTS`; host constants `CHART_MS`, `CHART_GROWTH`, `BATCH_MS`, `POST_MS`.
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3`; the commit commands below pass it as a second `-m`. Stage **only** the files named in the task (`git add <paths>`, never `-A`/`.`). No dependency changes, so neither `Cargo.lock` nor `web/package-lock.json` is staged.
- Rust tasks run `cargo fmt --all` **before** `cargo fmt --all --check`, then `cargo clippy --all-targets -- -D warnings` and `cargo test` (the whole workspace, including `golden.rs`, `legacy.rs` and the CLI tests). Task 2 also runs `wasm-pack test --node crates/sugarscape-wasm`. If clippy flags a loop (`needless_range_loop`) or `map_or(true, …)` (`unnecessary_map_or`), rewrite it with iterators / `is_none_or` without changing what is computed.
- Web tasks run `(cd web && npm run build && npm test)`. The build regenerates `web/src/wasm-pkg` (gitignored) with `wasm-pack` and runs `tsc --noEmit`; Vitest imports that package (engine.ts, sim-module.ts, the determinism test), so always build before testing. Single test files run with `(cd web && npx vitest run src/<file>.test.ts)`.
- **TypeScript:** `strict`, `noUnusedLocals`, `noUnusedParameters` (unused imports fail the build — each task lists its import changes). Never pass a possibly-null child to `replaceChildren`/`append`: use `h()`, which skips `null`/`false`, or filter the list first. Prefix deliberately unused parameters with `_`.
- **Browser checks are the controller's**, not the implementer's. Each web task names the scenarios it affects; the controller runs its puppeteer pass on those (and the full list after Tasks 11, 13 and 14).

## Why this task order

Bottom-up so every layer is tested before anything consumes it, and so the app never has a broken window:

- **Core then WASM (Tasks 1–2):** LTTB and the grouped variant are pure functions with property-style tests; the WASM getters (`series_downsampled`, `series_group`, `fingerprint`) are the host's only new needs from Rust. Doing them first means the regenerated `Sim` type already satisfies `SimLike` when Task 6 needs it.
- **Protocol, host, transports (Tasks 3–5):** plain TypeScript tested with a fake `Sim` in Node (Vitest cannot start a worker, and the host must not depend on WASM to be tested). Nothing in the app imports them yet.
- **Engine swap (Task 6):** the one cross-cutting step. It runs the new host on the page through `InlineTransport` (same thread, same WASM instance as today, so behavior is easiest to compare), converts every write to a promise, adds the frame loop's `pump`, and updates only the call sites that the new types force (writes whose errors are shown, the frame loop, the readout). A temporary `engine.sim` getter plus deprecated `inspect`, `diseaseList` and `creditGraph` keep the remaining panels working unchanged.
- **Determinism (Task 7):** as soon as the real engine path exists, a Vitest run with the real WASM pins the golden fingerprint, before any further refactor.
- **Call-site migration (Tasks 8–10):** selection/inspection/follow/overlays, then edits/disease list/credit/exports, then charts on downsampled groups — each leaves the app working, and Task 10 deletes the hatch (a `grep` proves no `engine.sim` remains).
- **Worker (Task 11):** only now does the host move threads; nothing above the transport changes. The controller runs every scenario and the determinism check through the worker here.
- **Max (Tasks 12–13):** host loop and `serve` scheduling first (testable with a fake clock), then the engine's run/stop/quiesce and the speed menu.
- **Docs and full verification (Task 14),** including the controller's performance check.

## Decisions (where the spec leaves room)

These are binding; each is repeated in the task that implements it.

1. **Envelope.** Requests are `HostRequest { id, cmd, wants?, frame? }`; replies `HostReply { id, result, spare? }` where `result` is `{ ok: true, snapshot?, value? } | { ok: false, errors: FieldError[] } | { ok: false, fatal: string }`; the host's own messages are `{ id: null, post: WorldSnapshot } | { id: null, fatal }`. Every request carries the engine's current merged wants, so any reply's snapshot has what is drawn (selection, trail, overlays) and what open panels need. Three commands are added to the spec's list: `ready` (the worker's WASM loaded; Task 11's start-up handshake), `refresh` (a snapshot with the current wants and no world change — used while paused, after resets and config changes, and when a panel opens) and `frame` (hands a buffer back to the Max loop, with fresh wants). `exportConfig` and `exportLandscape` are not added: every snapshot that can change them carries `config` and `editedLandscapes`.
2. **Snapshot fields.** Always: `width`, `height`, `tick`, `population`, `latest` (the stats `Snapshot`), `followed` (id or null) and `followedAlive` (replaces `sim.locate` for the toolbar chip). On change only: `config` (after init, reset, setConfig and a scheduled change), `editedLandscapes` (after init, reset, setConfig, paint and import) and `display` (when the host clamped it). The engine keeps the last value of each. `modified` stays on the main thread (`engine.isModified()`), because it depends on which preset the user picked, which only the engine knows. On request (`Wants`): `inspection`, `trail`, `networks`, `charts`, `lorenz`, `wealthHist`, `supplyDemand`, `creditGraph`, `diseaseList`; an extra the world cannot give (a chart line or a selected site a config change just removed — a thrown field error) is left out rather than failing the command.
3. **Selection.** `wants.select = { x, y, agentId }`: the host locates the agent (when not null and alive) and inspects its site, returning `Selected { x, y, agentId, alive, view }`; the engine moves `selection` to it. This replaces `trackSelection`, the Inspect panel's `sim.locate` and `engine.inspect`. The `inspect` command's target is `{ x, y }` (select a site; `agentId` = the agent there) or `{ agentId }` (select an agent; a dead one gives `inspection: null` and the engine keeps its selection, as `selectAgent` does today). Init and reset ignore `wants.select`; the engine clears its selection on a reset reply.
4. **Chart groups.** `wants.charts = { groups: string[][], max: CHART_POINTS (2000) }`: one group per time chart (its line keys; the Trade price chart's is `['mean_log_price', 'sd_log_price']`), keyed by `chartKey(names) = names.join('|')`. The host answers with `Sim.series_group(namesJson, max)` → core `stats::downsample_union`: the sorted union of each line's LTTB points (at most `max` per line), with every line's value at every chosen tick, so a multi-line chart and the ±SD band share one x axis and each line keeps its own spikes. `Sim.series_downsampled(name, max)` is exposed and tested exactly as specified (single series, `[tick, value, …]`), but the panel uses groups (spec gap 1). The throttle is per group: sent when never sent, or when the history (`tick + 1`) has grown and either `CHART_MS` (250 ms) have passed since the last send or it grew by more than `CHART_GROWTH` (1 %); a group whose history has not grown is never re-sent; the record is cleared whenever the snapshot carries `config` (init, reset, setConfig, a scheduled change).
5. **LTTB.** `stats::downsample(values, max) -> Vec<(u32, f64)>` where the `u32` is the index into `values`, which is the tick (the history holds one snapshot per tick from 0); WASM maps indices through the `tick` series anyway. `n ≤ max` or `n ≤ 2` returns every point; `max < 3` returns the two endpoints; otherwise exactly `max` points: the endpoints plus one per bucket, with integer bucket bounds `1 + b·(n−2)/(max−2)`. The triangle's first corner is the last *finite* point kept; the third is the mean of the next bucket's finite points (the last point after the final bucket). Non-finite values are never a bucket's choice; a bucket with no finite value keeps its first index, so a gap at least a bucket wide stays a gap (a narrower one closes at that zoom — invisible, a bucket being well under a pixel). The panel turns NaN into `null` (a uPlot gap).
6. **Frame buffers.** The engine lends a buffer with every command that changes what is drawn — `init`, `reset`, `setConfig`, `step`, `setDisplay`, the edits, `run` and `frame` — and none with `refresh`, `inspect`, `follow`, exports or `stop`. The host renders into the lent buffer (or into a new one when the grid size differs) and returns it as `snapshot.frame`; a buffer it did not use comes back in `spare`. The engine draws from `shown`; when a new frame arrives the old `shown` goes to its spare list (at most 4, all of the current size), and the next request takes a spare (allocating only when none fits). In steady state exactly two buffers alternate (Task 6 tests it); during Max the host pools the buffers it receives and posts only when it holds one, so a slow page gets fewer posts, never a backlog.
7. **Errors.** A thrown **string** is the core's JSON `FieldError[]` and becomes `{ ok: false, errors }` in the existing `[{ field, message }]` shape; anything else thrown (a panic's `RuntimeError`, a bug) makes the host dead: that reply and every later one is `{ ok: false, fatal: 'The simulation stopped: …' }`. A worker `error` event does the same. The transport calls `onFatal` once, the engine sets `crashed`, stops running and emits `'crash'`, and `main.ts` shows the existing "The simulation crashed." banner with Reload. Engine writes on a dead host resolve `[{ field: 'simulation', message }]`.
8. **Quiet world.** `reset`, `loadPreset` and `applyConfig` run inside `quiet()`: it holds the frame loop, stops Max and waits for the stop acknowledgement (Task 13), waits for the outstanding step, then builds and sends the command, and afterwards releases the loop and restarts Max if it is still wanted. `applyConfig` is included (the spec names only reset and preset) because it builds the next config from `engine.config`: a step still in flight could fire a scheduled change that the new config would silently undo. Edits, display changes, follow and inspect simply queue behind the current step or batch.
9. **Providers.** `engine.want(provider)` registers `(now) => Wants`, called before every request (it must not change state; panels record what *arrived*). The engine adds its own: `select` (while something is selected), `trail` (while following) and `networks` (overlays that are on). Charts asks for its visible groups while its tab is shown and for the three distributions at most every 250 ms when the tick moved or something invalidated them; Credit asks for the graph at most every 500 ms when the tick or panel width moved; the disease tools ask for the list at most every 250 ms while open. While paused, `pump` sends a `refresh` at most every 250 ms, and only when some provider wants something.
10. **Page WASM.** The page keeps its own WASM instance (`init()`), as today, for `presets_json`, the Experiments view's parsing and aggregation, and the inline fallback. The worker loads its own. `startWorker()` resolves after the worker answers `ready`; if it errors first (no module workers, a load failure), `Engine.create` logs a warning and uses `InlineTransport` on the page instance.
11. **Max.** Speed `'max'`: `run` starts a host loop scheduled with a `MessageChannel` (no 4 ms timer clamp; queued messages are handled between batches); each batch steps one tick at a time until `BATCH_MS` (16 ms) have passed, and posts a snapshot when `POST_MS` (33 ms) have passed since the last post and a pooled buffer is free. Each post's displaced `shown` buffer goes straight back in a `frame` command carrying the current wants. `stop` replies with a final snapshot (in a pooled buffer when one is left) and returns the rest in `spare`. The page fallback schedules batches with `setTimeout(0)`.
12. **Temporary hatch (Tasks 6–10).** `engine.sim` returns the inline host's `Sim` (via `SimHost.currentSim()`), and `engine.inspect`, `engine.diseaseList`, `engine.creditGraph` are sync wrappers over it, all marked `@deprecated` with the task that removes them. Task 10 deletes them and `currentSim`; Task 11 (the worker) cannot start until they are gone.
13. **Landscapes.** The engine's `customLandscapes` becomes the host's `editedLandscapes` (each good's map where it differs from the generated one — what share links already use). Resets keep them under the same rule as today; a map painted back to exactly its generated state now counts as unmodified.
14. **Debug hook.** With `?debug` in the URL, `main.ts` sets `window.sugarscape = { engine }` so the controller can drive the determinism check through the worker (`loadPreset`, `reset(baseConfig, 1)`, `advance(200)`, `fingerprint()`).
15. **Determinism in Vitest.** `web/src/determinism.test.ts` loads the real `sugarscape_bg.wasm` with `initSync` and `node:fs` (typed by a three-line `web/src/node-shims.d.ts`, since the web build has no `@types/node`).

## Migration map

**Reads** (sync before → sync after, served from the latest snapshot):

| Today | After | Task |
|---|---|---|
| `engine.sim.tick()` (main slug, toolbar, charts, credit) | `engine.tick` | 6 (main, toolbar), 9 (credit), 10 (charts) |
| `engine.sim.population()` (toolbar) | `engine.population` | 6 |
| `engine.size()` (`sim.width()/height()`) | `engine.size()` from `snapshot.width/height` | 6 |
| `engine.frame()` (view into WASM memory) | `engine.frame()`: a view of the `shown` buffer, or null before the first frame | 6 |
| `engine.trackSelection()` (main loop) | removed: `selection` follows `snapshot.inspection` | 6 |
| `engine.inspect(x, y)` (Inspect panel) | `engine.inspection.view` | 6 (deprecated wrapper), 8 |
| `engine.sim.locate(sel.agentId)` (Inspect "has died") | `!engine.inspection.alive` | 8 |
| `engine.sim.locate(id)` (toolbar chip) | `engine.followedAlive()` | 8 |
| `engine.followed()`, `engine.trail()` | same names, from `snapshot.followed` / `snapshot.trail` | 6 |
| `engine.sim.networks(kind)` (grid) | `engine.networks(kind)` from `snapshot.networks` | 8 |
| `engine.diseaseList()` (tools) | `engine.last.diseaseList` via a provider | 6 (deprecated wrapper), 9 |
| `engine.creditGraph()` (Credit tab) | `engine.last.creditGraph` via a provider | 6 (deprecated wrapper), 9 |
| `engine.sim.series(name)` (charts) | `engine.last.charts[chartKey(group)]` via a provider | 10 |
| `engine.sim.lorenz/wealth_hist/supply_demand` | `engine.last.lorenz/wealthHist/supplyDemand` via a provider | 10 |
| `engine.editedLandscapes()` (share, Experiments) | same name, from `snapshot.editedLandscapes` | 6 |
| `engine.config`, `baseConfig`, `presetId`, `seed`, `isModified()`, `colorMode`, `layer`, `overlays`, `selection` | unchanged fields (config/display updated from snapshots) | 6 |

**Writes and async reads** (now promises):

| Method | Returns | Callers that **await** (they show errors or a result) | Callers that fire and forget |
|---|---|---|---|
| `reset`, `applyConfig`, `loadPreset` | `Promise<FieldError[] \| null>` | Rules panel (`commit`, preset select) | toolbar Reset / 🎲 (errors were ignored before too) |
| `importLandscape` | `Promise<FieldError[] \| null>` | tools' image import (shows errors) | — |
| `paint`, `place`, `erase`, `infect`, `vaccinate` | `Promise<FieldError[] \| null>` | — | tools (errors were ignored before) |
| `advance(n)` | `Promise<void>` | tests | toolbar Step |
| `select`, `selectAgent`, `followAgent`, `unfollow`, `refresh` | `Promise<void>` | — | tools, Inspect, Credit, toolbar chip, panels |
| `seriesCsv`, `agentsCsv`, `fingerprint` | `Promise<string>` | Export menu, tests, debug hook | — |
| `setDisplay`, `setRunning`, `setSpeed`, `pump` | `void` (local state changes at once; the host follows) | — | display, tools, toolbar, main loop |

**Events** (each fires after the reply's snapshot has been adopted; every reply with a snapshot then fires `'display'` if the host clamped the display, then `'snapshot'`):

| Reply to | Events |
|---|---|
| `init` | none (the engine is being constructed) |
| `reset` (also `loadPreset`) | `'reset'` |
| `setConfig` | `'config'` |
| `step`, a Max post, the `stop` reply | `'config'` if the snapshot carries `config` (a scheduled change fired), then `'tick'` |
| `paint`, `importLandscape`, `place`, `erase`, `infect`, `vaccinate` | `'edit'` |
| `follow` | `'follow'` |
| `inspect` (non-null inspection) | `'select'` |
| `setDisplay`, `refresh` | only `'display'`/`'snapshot'` (`setDisplay()` itself emits `'display'` at once, locally) |
| a fatal result or worker error | `'run'` (now paused) and `'crash'`, once |
| `setRunning` | `'run'`, locally and at once |

**Frame ping-pong:** see Decision 6. **Max quiesce:** `quiet()` (Decision 8) calls `stopMax()`, which sends `stop` and resolves only when its reply has been adopted — the FIFO transport guarantees every post sent before it has been handled too — so the reset/preset/config request that follows meets a host that is not looping; `finally` restarts Max when `running && speed === 'max'`.

## File Structure

```
crates/sugarscape-core/src/stats.rs        MOD  downsample, downsample_union (+ tests) (1)
crates/sugarscape-wasm/src/lib.rs          MOD  Sim.series_downsampled, series_group, fingerprint (2)
crates/sugarscape-wasm/tests/web.rs        MOD  (2)
web/src/protocol.ts, protocol.test.ts      NEW  commands, wants, snapshots, mergeWants, transfers (3); run/stop/frame (12)
web/src/layers.ts, layers.test.ts          MOD  clampDisplay (3)
web/src/sim-host.ts, sim-host.test.ts      NEW  SimHost (4); serve (5); currentSim (6, removed 10); Max loop (12)
web/src/fake-sim.fixture.ts                NEW  FakeSim, fakeModule (4)
web/src/transport.ts, transport.test.ts    NEW  Transport, PortTransport, InlineTransport (5); startWorker (11); inline Max scheduling (12)
web/src/sim-module.ts                      NEW  wasmSimModule (6)
web/src/engine.ts, engine.test.ts          REWRITE (6); MOD (8, 9, 10, 11, 13)
web/src/main.ts                            MOD  (6, 9, 10, 11)
web/src/ui/toolbar.ts                      MOD  (6, 8, 13)
web/src/ui/rules-panel.ts                  MOD  (6)
web/src/ui/tools.ts                        MOD  (6, 9)
web/src/ui/grid-view.ts                    MOD  (6, 8)
web/src/ui/inspect-panel.ts                MOD  (8)
web/src/ui/credit-panel.ts                 MOD  (9)
web/src/ui/series-data.ts, series-data.test.ts  NEW (10)
web/src/ui/charts-panel.ts                 REWRITE (10)
web/src/node-shims.d.ts                    NEW  (7)
web/src/determinism.test.ts                NEW  (7)
web/src/sim-worker.ts                      NEW  (11); MOD (12)
README.md, docs/roadmap.md                 MOD  (14)
```

---

### Task 1: LTTB downsampling in the core

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/stats.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces (module `sugarscape_core::stats`):
  - `pub fn downsample(values: &[f64], max: usize) -> Vec<(u32, f64)>` (Decision 5)
  - `pub fn downsample_union(columns: &[Vec<f64>], max: usize) -> Vec<usize>` — sorted, deduplicated union of each column's `downsample` indices (Decision 4)

- [ ] **Step 1: Confirm the baseline**

Run: `cargo test -p sugarscape-core --test golden --test legacy`
Expected: both pass (golden `2 passed; 0 failed; 1 ignored`). If anything fails, stop: the branch is not at the expected state.

- [ ] **Step 2: Write the failing tests**

Append inside `mod tests` in `crates/sugarscape-core/src/stats.rs`:
```rust
    #[test]
    fn downsample_keeps_short_series_and_both_endpoints() {
        let v: Vec<f64> = (0..10).map(f64::from).collect();
        let all: Vec<(u32, f64)> = (0..10u32).map(|i| (i, f64::from(i))).collect();
        assert_eq!(downsample(&v, 10), all);
        assert_eq!(downsample(&v, 50), all);
        assert_eq!(downsample(&v, 2), vec![(0, 0.0), (9, 9.0)]);
        assert_eq!(downsample(&v, 0), vec![(0, 0.0), (9, 9.0)]);
        assert!(downsample(&[], 5).is_empty());
        assert_eq!(downsample(&[4.0], 0), vec![(0, 4.0)]);
        let long: Vec<f64> = (0..10_000).map(|i| (f64::from(i) / 50.0).sin()).collect();
        for max in [3, 7, 100, 2000] {
            let d = downsample(&long, max);
            assert_eq!(d.len(), max, "max {max}");
            assert_eq!(d[0], (0, long[0]));
            assert_eq!(d[max - 1], (9_999, long[9_999]));
            assert!(d.windows(2).all(|w| w[0].0 < w[1].0), "indices ascend");
            assert!(d.iter().all(|&(i, y)| long[i as usize] == y), "points are real");
        }
    }

    #[test]
    fn downsample_keeps_a_single_sharp_spike() {
        let mut v = vec![1.0; 10_000];
        v[4_321] = 50.0;
        let d = downsample(&v, 100);
        assert!(d.len() <= 100);
        assert!(d.contains(&(4_321, 50.0)), "{d:?}");
    }

    #[test]
    fn downsample_skips_nan_but_keeps_gaps() {
        let v: Vec<f64> = (0..1000)
            .map(|i: i32| {
                if (400..600).contains(&i) {
                    f64::NAN
                } else {
                    f64::from(i % 7)
                }
            })
            .collect();
        let d = downsample(&v, 50);
        assert_eq!(d.len(), 50);
        assert_eq!((d[0].0, d[49].0), (0, 999));
        let nan: Vec<u32> = d.iter().filter(|p| p.1.is_nan()).map(|p| p.0).collect();
        assert!(!nan.is_empty(), "the gap survives");
        assert!(nan.iter().all(|i| (400..600).contains(i)), "{nan:?}");
        assert!(d.iter().any(|p| p.0 < 400 && p.1.is_finite()));
        assert!(d.iter().any(|p| p.0 >= 600 && p.1.is_finite()));
    }

    #[test]
    fn downsample_union_keeps_each_columns_spike() {
        let mut a = vec![0.0; 5_000];
        let mut b = vec![0.0; 5_000];
        a[1_000] = 9.0;
        b[3_000] = -9.0;
        let keep = downsample_union(&[a, b], 50);
        assert!(keep.contains(&1_000) && keep.contains(&3_000), "{keep:?}");
        assert!(keep.windows(2).all(|w| w[0] < w[1]), "sorted, no duplicates");
        assert_eq!((keep[0], keep[keep.len() - 1]), (0, 4_999));
        assert!(keep.len() <= 100);
    }
```

- [ ] **Step 3: Run the tests to see them fail**

Run: `cargo test -p sugarscape-core --lib stats::tests::downsample`
Expected: compile error `cannot find function 'downsample' in this scope` (and `downsample_union`).

- [ ] **Step 4: Implement**

In `crates/sugarscape-core/src/stats.rs`, add `use std::ops::Range;` to the imports (after `use serde::Serialize;`, as its own group), and insert before `#[cfg(test)]`:
```rust
/// Largest-Triangle-Three-Buckets: at most `max` of `values`' points as
/// `(index, value)` — the index is the tick, since the history holds one
/// snapshot per tick from 0 — chosen so the line keeps its shape. The first
/// and last points are always kept, and `values.len() <= max` keeps every
/// point. Non-finite values (NaN: no trades, no agents) are never a bucket's
/// choice, but a bucket holding nothing else keeps its first index, so a gap
/// stays a gap. `max < 3` keeps the two endpoints only.
pub fn downsample(values: &[f64], max: usize) -> Vec<(u32, f64)> {
    let n = values.len();
    let point = |i: usize| (i as u32, values[i]);
    if n <= max || n <= 2 {
        return (0..n).map(point).collect();
    }
    if max < 3 {
        return vec![point(0), point(n - 1)];
    }
    let buckets = max - 2;
    // Bucket b holds indices start(b)..start(b + 1) of the interior 1..n - 1;
    // each holds at least one, since n - 2 > buckets.
    let start = |b: usize| 1 + b * (n - 2) / buckets;
    let mut out = Vec::with_capacity(max);
    out.push(point(0));
    // The triangle's first corner: the last finite point kept.
    let mut anchor = values[0].is_finite().then_some((0.0, values[0]));
    for b in 0..buckets {
        // Its third corner: the mean of the next bucket (the last point after the final one).
        let next = if b + 1 < buckets {
            start(b + 1)..start(b + 2)
        } else {
            n - 1..n
        };
        let third = finite_mean(values, next);
        let mut best: Option<(usize, f64)> = None;
        for (i, &y) in values.iter().enumerate().take(start(b + 1)).skip(start(b)) {
            if !y.is_finite() {
                continue;
            }
            let x = i as f64;
            let area = match (anchor, third) {
                (Some((ax, ay)), Some((cx, cy))) => {
                    ((ax - cx) * (y - ay) - (ax - x) * (cy - ay)).abs()
                }
                (Some((_, ay)), None) => (y - ay).abs(),
                (None, Some((_, cy))) => (y - cy).abs(),
                (None, None) => 0.0,
            };
            if best.is_none_or(|(_, a)| area > a) {
                best = Some((i, area));
            }
        }
        match best {
            Some((i, _)) => {
                out.push(point(i));
                anchor = Some((i as f64, values[i]));
            }
            // Nothing finite here: keep the gap.
            None => out.push(point(start(b))),
        }
    }
    out.push(point(n - 1));
    out
}

/// The mean position of the finite values in `range`, or `None` if it has none.
fn finite_mean(values: &[f64], range: Range<usize>) -> Option<(f64, f64)> {
    let (mut sx, mut sy, mut k) = (0.0, 0.0, 0.0);
    for (i, &y) in range.clone().zip(&values[range]) {
        if y.is_finite() {
            sx += i as f64;
            sy += y;
            k += 1.0;
        }
    }
    if k > 0.0 {
        Some((sx / k, sy / k))
    } else {
        None
    }
}

/// The sorted union of each column's `downsample(column, max)` indices: one
/// x axis on which every column keeps its own shape (a multi-line chart).
pub fn downsample_union(columns: &[Vec<f64>], max: usize) -> Vec<usize> {
    let mut keep: Vec<usize> = columns
        .iter()
        .flat_map(|c| downsample(c, max).into_iter().map(|(i, _)| i as usize))
        .collect();
    keep.sort_unstable();
    keep.dedup();
    keep
}
```

- [ ] **Step 5: Run the tests to see them pass**

Run: `cargo test -p sugarscape-core --lib stats::tests::downsample`
Expected: `4 passed`.

- [ ] **Step 6: Full verification**

```bash
cargo fmt --all && cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```
All pass; `golden` and `legacy` unchanged and green.

- [ ] **Step 7: Commit**

```bash
git add crates/sugarscape-core/src/stats.rs
git commit -m "Add Largest-Triangle-Three-Buckets downsampling to the statistics" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 2: Downsampled series and the fingerprint in WASM

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-wasm/src/lib.rs`
- Modify: `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `stats::downsample`, `stats::downsample_union` (Task 1); `World::fingerprint`; the existing `Sim::series`.
- Produces (`#[wasm_bindgen] impl Sim`):
  - `pub fn series_downsampled(&self, name: &str, max: u32) -> Result<Vec<f64>, JsValue>` → JS `Float64Array` `[tick, value, tick, value, …]`
  - `pub fn series_group(&self, names_json: &str, max: u32) -> Result<Vec<f64>, JsValue>` → `[n, ticks (n), then n values per name, in order]`
  - `pub fn fingerprint(&self) -> String` → `"0x…"` (`{:#x}`, the golden file's format)

- [ ] **Step 1: Write the failing tests**

Append to `crates/sugarscape-wasm/tests/web.rs`:
```rust
#[wasm_bindgen_test]
fn series_downsampled_pairs_ticks_and_values() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    sim.step(50);
    let pop = sim.series("population").unwrap();
    let all = sim.series_downsampled("population", 100).unwrap();
    assert_eq!(all.len(), 2 * 51);
    for (i, (pair, p)) in all.chunks(2).zip(&pop).enumerate() {
        assert_eq!(pair, [i as f64, *p]);
    }
    let few = sim.series_downsampled("population", 10).unwrap();
    assert_eq!(few.len(), 2 * 10);
    assert_eq!((few[0], few[18]), (0.0, 50.0));
    assert!(few.chunks(2).all(|p| pop[p[0] as usize] == p[1]));
    assert!(sim.series_downsampled("nope", 10).is_err());
}

#[wasm_bindgen_test]
fn series_group_shares_one_tick_axis() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    sim.step(300);
    let g = sim
        .series_group(r#"["population","gini"]"#, 20)
        .unwrap();
    let n = g[0] as usize;
    assert!((20..=40).contains(&n), "{n}");
    assert_eq!(g.len(), 1 + 3 * n);
    let ticks = &g[1..1 + n];
    assert_eq!((ticks[0], ticks[n - 1]), (0.0, 300.0));
    assert!(ticks.windows(2).all(|w| w[0] < w[1]));
    let pop = sim.series("population").unwrap();
    let gini = sim.series("gini").unwrap();
    for (k, &t) in ticks.iter().enumerate() {
        assert_eq!(g[1 + n + k], pop[t as usize]);
        assert_eq!(g[1 + 2 * n + k], gini[t as usize]);
    }
    assert!(sim.series_group(r#"["population","nope"]"#, 20).is_err());
    assert!(sim.series_group("not json", 20).is_err());
}

#[wasm_bindgen_test]
fn fingerprint_matches_the_golden_entry() {
    let preset = sugarscape_core::presets::by_id("ii-2-unit").unwrap();
    let json = serde_json::to_string(&preset.config).unwrap();
    let mut sim = Sim::new(&json, 1, JsValue::NULL).unwrap();
    sim.step(200);
    // crates/sugarscape-core/tests/golden.rs
    assert_eq!(sim.fingerprint(), "0x75b93943813545e4");
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: compile error `no method named 'series_downsampled' found for struct 'Sim'` (and `series_group`, `fingerprint`).

- [ ] **Step 3: Implement**

In `crates/sugarscape-wasm/src/lib.rs`, insert inside `#[wasm_bindgen] impl Sim`, right after `pub fn series`:
```rust
    /// Series `name` cut to at most `max` points (`stats::downsample`) as
    /// `[tick, value, tick, value, …]`.
    pub fn series_downsampled(&self, name: &str, max: u32) -> Result<Vec<f64>, JsValue> {
        let values = self.series(name)?;
        let ticks = self.world.stats.series("tick").unwrap_or_default();
        Ok(stats::downsample(&values, max as usize)
            .into_iter()
            .flat_map(|(i, v)| [ticks[i as usize], v])
            .collect())
    }

    /// Several series on one x axis (`stats::downsample_union`: each keeps
    /// at most `max` points of its own shape) as `[n, ticks (n), then n
    /// values per name]`. `names_json` is a JSON array of series names.
    pub fn series_group(&self, names_json: &str, max: u32) -> Result<Vec<f64>, JsValue> {
        let names: Vec<String> =
            serde_json::from_str(names_json).map_err(|e| edit_error(e.to_string()))?;
        let columns = names
            .iter()
            .map(|name| self.series(name))
            .collect::<Result<Vec<_>, _>>()?;
        let ticks = self.world.stats.series("tick").unwrap_or_default();
        let keep = stats::downsample_union(&columns, max as usize);
        let mut out = Vec::with_capacity(1 + keep.len() * (1 + columns.len()));
        out.push(keep.len() as f64);
        out.extend(keep.iter().map(|&i| ticks[i]));
        for column in &columns {
            out.extend(keep.iter().map(|&i| column[i]));
        }
        Ok(out)
    }

    /// `World::fingerprint` as `0x…` hex, the golden tests' format.
    pub fn fingerprint(&self) -> String {
        format!("{:#x}", self.world.fingerprint())
    }
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: all pass, including the three new tests. If `fingerprint_matches_the_golden_entry` fails, stop and report: the JSON round trip of a preset must not change a run.

- [ ] **Step 5: Full verification**

```bash
cargo fmt --all && cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
```
All pass (the web build regenerates `web/src/wasm-pkg` with the three new methods; nothing uses them yet).

- [ ] **Step 6: Commit**

```bash
git add crates/sugarscape-wasm/src/lib.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Expose downsampled series and the fingerprint from WASM" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 3: The protocol and display clamping

*Mechanical (full code).* Browser: nothing to check (no UI change).

**Files:**
- Create: `web/src/protocol.ts`, `web/src/protocol.test.ts`
- Modify: `web/src/layers.ts`, `web/src/layers.test.ts`

**Interfaces:**
- Consumes: `CreditGraph` (credit.ts); `ColorMode`, `Config`, `DiseaseEntry`, `FieldError`, `Inspection`, `Layer`, `Snapshot` (types.ts); `validLayer` (layers.ts).
- Produces (`web/src/protocol.ts`): the types `Overlay`, `PlaceOverrides`, `DisplayState`, `SelectQuery`, `Selected`, `ChartGroup`, `Wants`, `WorldSnapshot`, `Command`, `HostRequest`, `Result`, `HostReply`, `HostMessage`; `const OVERLAYS: Overlay[]`, `const CHART_POINTS = 2000`; `chartKey(names: string[]): string`; `mergeWants(parts: Wants[]): Wants`; `transfers(m: HostRequest | HostMessage): ArrayBuffer[]`.
- Produces (`web/src/layers.ts`): `clampDisplay(d: DisplayState, config: Config): DisplayState` — returns `d` itself when nothing changes.

- [ ] **Step 1: Write the failing tests**

Create `web/src/protocol.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { chartKey, mergeWants, transfers, type WorldSnapshot } from './protocol';
import type { Snapshot } from './types';

describe('mergeWants', () => {
  it('ORs flags, unions networks (in overlay order) and chart groups (by key), and keeps the first selection', () => {
    const merged = mergeWants([
      { select: { x: 1, y: 2, agentId: null }, networks: ['disease'], charts: { groups: [['population']], max: 100 } },
      {
        select: { x: 9, y: 9, agentId: 3 },
        lorenz: true,
        networks: ['trade', 'disease'],
        charts: { groups: [['population'], ['gini', 'births']], max: 2000 },
      },
      {},
    ]);
    expect(merged).toEqual({
      select: { x: 1, y: 2, agentId: null },
      lorenz: true,
      networks: ['trade', 'disease'],
      charts: { groups: [['population'], ['gini', 'births']], max: 2000 },
    });
  });

  it('is empty when nothing is wanted', () => {
    expect(mergeWants([{}, {}])).toEqual({});
  });
});

describe('chartKey', () => {
  it('joins the names', () => {
    expect(chartKey(['mean_log_price', 'sd_log_price'])).toBe('mean_log_price|sd_log_price');
  });
});

describe('transfers', () => {
  it('lists the buffers a message carries', () => {
    const a = new ArrayBuffer(4);
    const b = new ArrayBuffer(4);
    const snapshot: WorldSnapshot = {
      width: 1,
      height: 1,
      tick: 0,
      population: 0,
      latest: {} as Snapshot,
      followed: null,
      followedAlive: false,
      frame: a,
    };
    const request = transfers({ id: 1, cmd: { type: 'refresh' }, frame: a });
    expect(request).toHaveLength(1);
    expect(request[0]).toBe(a);
    const reply = transfers({ id: 1, result: { ok: true, snapshot }, spare: [b] });
    expect(reply).toHaveLength(2);
    expect(reply[0]).toBe(b);
    expect(reply[1]).toBe(a);
    expect(transfers({ id: null, post: snapshot })[0]).toBe(a);
    expect(transfers({ id: 2, result: { ok: false, errors: [] } })).toEqual([]);
    expect(transfers({ id: 3, cmd: { type: 'fingerprint' } })).toEqual([]);
  });
});
```
Append to `web/src/layers.test.ts` (and add `clampDisplay` to its `./layers` import and `import type { DisplayState } from './protocol';`):
```ts
describe('clampDisplay', () => {
  const display: DisplayState = { colorMode: 'disease', layer: 'pollution:0', overlays: { trade: true, credit: false, disease: true } };
  const withDisease = (enabled: boolean) => ({ ...config, disease: { enabled } }) as unknown as Config;

  it('keeps a valid display as the same object', () => {
    expect(clampDisplay(display, withDisease(true))).toBe(display);
  });

  it('drops the disease mode and overlay while disease is off, and a layer the world lacks', () => {
    expect(clampDisplay(display, withDisease(false))).toEqual({
      colorMode: 'tribe',
      layer: 'pollution:0',
      overlays: { trade: true, credit: false, disease: false },
    });
    expect(clampDisplay({ ...display, layer: 'capacity:2' }, withDisease(true)).layer).toBe('resource:0');
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/protocol.test.ts src/layers.test.ts)`
Expected: FAIL — `Failed to resolve import "./protocol"` and `clampDisplay is not a function` (or not exported).

- [ ] **Step 3: Implement**

Create `web/src/protocol.ts`:
```ts
// Messages between the engine (page) and the SimHost (simulation worker, or the page as a fallback).
import type { CreditGraph } from './credit';
import type { ColorMode, Config, DiseaseEntry, FieldError, Inspection, Layer, Snapshot } from './types';

export type Overlay = 'trade' | 'credit' | 'disease';
export const OVERLAYS: Overlay[] = ['trade', 'credit', 'disease'];

export interface PlaceOverrides { sex?: 'female' | 'male'; tribe?: 'blue' | 'red' }

/** How the host renders frames and which network overlays are drawn. */
export interface DisplayState { colorMode: ColorMode; layer: Layer; overlays: Record<Overlay, boolean> }

/** The selection to report on: a site, or an agent (tracked while it lives; x, y are its last known site). */
export interface SelectQuery { x: number; y: number; agentId: number | null }

/** The selected site as the host last saw it; `alive` is false once a selected agent has died (or none was selected). */
export interface Selected { x: number; y: number; agentId: number | null; alive: boolean; view: Inspection }

/** Several series downsampled onto one tick axis: `columns[k][i]` is series k at `ticks[i]` (NaN = no value). */
export interface ChartGroup { ticks: Float64Array; columns: Float64Array[] }

/** Points per line in a chart group. */
export const CHART_POINTS = 2000;

/** What a snapshot should carry besides the always-present fields (Decision 2). */
export interface Wants {
  select?: SelectQuery;
  trail?: boolean;
  networks?: Overlay[];
  charts?: { groups: string[][]; max: number };
  lorenz?: boolean;
  wealthHist?: boolean;
  supplyDemand?: boolean;
  creditGraph?: boolean;
  diseaseList?: boolean;
}

export interface WorldSnapshot {
  /** RGBA pixels, when the request lent a buffer (transferred back). */
  frame?: ArrayBuffer;
  width: number;
  height: number;
  tick: number;
  population: number;
  latest: Snapshot;
  /** The followed agent's id (alive or not), or null. */
  followed: number | null;
  followedAlive: boolean;
  /** The normalized live config: after init, reset, setConfig and a scheduled change. */
  config?: Config;
  /** Each good's map where it differs from the generated one: after init, reset, setConfig, paint and import. */
  editedLandscapes?: (Uint8Array | null)[];
  /** The display, when the host had to clamp it to the config. */
  display?: DisplayState;
  /** The selection (from `wants.select` or an `inspect` command); null when an inspect by id found no agent. */
  inspection?: Selected | null;
  trail?: Uint32Array;
  networks?: Partial<Record<Overlay, Uint32Array>>;
  /** Chart groups, by `chartKey`, that have news since they were last sent. */
  charts?: Record<string, ChartGroup>;
  lorenz?: Float64Array;
  wealthHist?: Float64Array;
  supplyDemand?: Float64Array;
  creditGraph?: CreditGraph;
  diseaseList?: DiseaseEntry[];
}

export type Command =
  | { type: 'ready' }
  | { type: 'init'; config: Config; seed: number; landscapes: (Uint8Array | null)[]; display: DisplayState }
  | { type: 'reset'; config: Config; seed: number; landscapes: (Uint8Array | null)[] }
  | { type: 'setConfig'; config: Config }
  | { type: 'step'; n: number }
  | { type: 'refresh' }
  | { type: 'setDisplay'; display: DisplayState }
  | { type: 'paint'; x: number; y: number; radius: number; value: number; good: number }
  | { type: 'importLandscape'; good: number; capacities: Uint8Array }
  | { type: 'place'; x: number; y: number; overrides: PlaceOverrides }
  | { type: 'erase'; x: number; y: number }
  | { type: 'infect'; x: number; y: number; disease: number }
  | { type: 'vaccinate'; x: number; y: number; radius: number; disease: number }
  | { type: 'follow'; id: number | null }
  | { type: 'inspect'; target: { x: number; y: number } | { agentId: number } }
  | { type: 'seriesCsv' }
  | { type: 'agentsCsv' }
  | { type: 'fingerprint' };

export interface HostRequest { id: number; cmd: Command; wants?: Wants; frame?: ArrayBuffer }

export type Result =
  | { ok: true; snapshot?: WorldSnapshot; value?: string }
  | { ok: false; errors: FieldError[] }
  | { ok: false; fatal: string };

export interface HostReply { id: number; result: Result; spare?: ArrayBuffer[] }

/** A reply, or something the host sends on its own: a Max-speed snapshot, or news that it died. */
export type HostMessage = HostReply | { id: null; post: WorldSnapshot } | { id: null; fatal: string };

export function chartKey(names: string[]): string {
  return names.join('|');
}

const FLAGS = ['trail', 'lorenz', 'wealthHist', 'supplyDemand', 'creditGraph', 'diseaseList'] as const;

/** Combines wants: flags OR, networks and chart groups are unioned, the first selection wins. */
export function mergeWants(parts: Wants[]): Wants {
  const out: Wants = {};
  const networks = new Set<Overlay>();
  const groups = new Map<string, string[]>();
  let max = 0;
  for (const w of parts) {
    if (w.select && !out.select) out.select = w.select;
    for (const flag of FLAGS) if (w[flag]) out[flag] = true;
    w.networks?.forEach((k) => networks.add(k));
    if (w.charts) {
      max = Math.max(max, w.charts.max);
      for (const g of w.charts.groups) groups.set(chartKey(g), g);
    }
  }
  if (networks.size > 0) out.networks = OVERLAYS.filter((k) => networks.has(k));
  if (groups.size > 0) out.charts = { groups: [...groups.values()], max };
  return out;
}

/** The buffers a message carries, to transfer rather than copy. */
export function transfers(m: HostRequest | HostMessage): ArrayBuffer[] {
  if ('cmd' in m) return m.frame ? [m.frame] : [];
  if (m.id === null) return 'post' in m && m.post.frame ? [m.post.frame] : [];
  const out = [...(m.spare ?? [])];
  if (m.result.ok && m.result.snapshot?.frame) out.push(m.result.snapshot.frame);
  return out;
}
```
In `web/src/layers.ts`, change the first line to
```ts
import type { DisplayState } from './protocol';
import type { Config, Layer } from './types';
```
and append:
```ts
/**
 * `d` kept valid for `config` (the host applies it to every snapshot): a layer the world lacks falls
 * back to good 0's level; with disease off the Disease color mode and overlay fall back to Tribe and
 * hidden. Returns `d` itself when nothing changes.
 */
export function clampDisplay(d: DisplayState, config: Config): DisplayState {
  const layer = validLayer(d.layer, config);
  const off = !config.disease.enabled;
  const colorMode = off && d.colorMode === 'disease' ? 'tribe' : d.colorMode;
  const disease = off ? false : d.overlays.disease;
  if (layer === d.layer && colorMode === d.colorMode && disease === d.overlays.disease) return d;
  return { colorMode, layer, overlays: { ...d.overlays, disease } };
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/protocol.test.ts src/layers.test.ts)`
Expected: `Test Files 2 passed`.

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/protocol.ts web/src/protocol.test.ts web/src/layers.ts web/src/layers.test.ts
git commit -m "Define the simulation host protocol and display clamping" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 4: The simulation host

*Mechanical (full code).* Browser: nothing to check (not used yet).

**Files:**
- Create: `web/src/sim-host.ts`, `web/src/sim-host.test.ts`, `web/src/fake-sim.fixture.ts`

**Interfaces:**
- Consumes: Task 3's protocol types, `clampDisplay`; `parseErrors`, `Config`, `DiseaseEntry`, `Inspection`, `Snapshot` (types.ts); `CreditGraph` (credit.ts).
- Produces (`web/src/sim-host.ts`):
  - `interface SimLike` — the `Sim` methods the host calls (the real class satisfies it structurally; checked in Task 6's `sim-module.ts`)
  - `interface SimModule { create(configJson: string, seed: number, landscapes: (Uint8Array | null)[]): SimLike; frameBytes(ptr: number, len: number): Uint8Array }`
  - `const CHART_MS = 250`, `const CHART_GROWTH = 0.01`
  - `class SimHost { constructor(module: SimModule, now?: () => number); handle(req: HostRequest): HostReply; fail(e: unknown): string }`
- Produces (`web/src/fake-sim.fixture.ts`): `class FakeSim implements SimLike` (fields `ticks`, `stepCalls`, `config`, `agents`), `fakeModule(log?: string[]): SimModule & { sims: FakeSim[] }`.

Host rules (Decisions 1–4, 6, 7): commands are handled in order; `init`/`reset` build a new `Sim` (the old one is freed only after the new one exists, so a bad config keeps the world) and ignore `wants.select`; `setConfig`, `step`, `refresh`, `setDisplay`, edits, `follow` (which always adds the trail when following), `inspect` answer with a snapshot; exports and `fingerprint` answer with `value`. Thrown strings are field errors; anything else kills the host.

- [ ] **Step 1: Write the fake `Sim`**

Create `web/src/fake-sim.fixture.ts`:
```ts
// A stand-in for the WASM `Sim` in host, transport and engine tests (Vitest runs in Node, without WASM).
import type { SimLike, SimModule } from './sim-host';

const fieldError = (field: string, message: string): string => JSON.stringify([{ field, message }]);

interface FakeConfig {
  width: number;
  height: number;
  population: number;
  schedule: { tick: number; set: Record<string, unknown> }[];
  goods: { name: string }[];
  pollution: { enabled: boolean; pollutants: { name: string }[] };
  disease: { enabled: boolean };
}

/** The core fills in missing fields; so does the fake (enough for the host and `clampDisplay`). */
function normalize(c: Partial<FakeConfig>): FakeConfig {
  return {
    width: 4,
    height: 3,
    population: 10,
    schedule: [],
    goods: [{ name: 'sugar' }],
    pollution: { enabled: false, pollutants: [] },
    disease: { enabled: false },
    ...c,
  };
}

/**
 * A 4 × 3 world (by default) with one agent, #1, starting at (1, 1) and walking one site right per
 * tick. `population > 1000` is a field error; painting a negative capacity "panics" (throws an
 * Error, not a string). Frames are filled with `tick % 256`.
 */
export class FakeSim implements SimLike {
  ticks = 0;
  stepCalls = 0;
  config: FakeConfig;
  agents = new Map<number, [number, number]>([[1, [1, 1]]]);
  private edited: boolean[];
  private followedId = -1;

  constructor(
    config: Partial<FakeConfig>,
    readonly seed: number,
    private log: string[],
  ) {
    this.config = normalize(config);
    this.edited = this.config.goods.map(() => false);
  }

  step(n: number): void {
    this.stepCalls++;
    this.ticks += n;
    const a = this.agents.get(1);
    if (a) a[0] = (a[0] + n) % this.config.width;
  }
  tick(): number {
    return this.ticks;
  }
  width(): number {
    return this.config.width;
  }
  height(): number {
    return this.config.height;
  }
  population(): number {
    return this.agents.size;
  }
  render(colorMode: string, layer: string): number {
    this.log.push(`render ${colorMode} ${layer}`);
    return 0;
  }
  frame_len(): number {
    return this.config.width * this.config.height * 4;
  }
  stats_latest(): string {
    return JSON.stringify({ tick: this.ticks, population: this.agents.size });
  }
  series_group(namesJson: string, max: number): Float64Array {
    const names = JSON.parse(namesJson) as string[];
    for (const n of names) if (n !== 'population' && n !== 'gini') throw fieldError('edit', `unknown series "${n}"`);
    const k = Math.min(this.ticks + 1, max);
    const ticks = Array.from({ length: k }, (_, i) => (k === 1 ? 0 : Math.round((i * this.ticks) / (k - 1))));
    return Float64Array.from([k, ...ticks, ...names.flatMap((n) => ticks.map((t) => (n === 'gini' ? 0.5 : t)))]);
  }
  lorenz(points: number): Float64Array {
    return new Float64Array(points);
  }
  wealth_hist(bins: number): Float64Array {
    return new Float64Array(bins + 1);
  }
  supply_demand(): Float64Array {
    return Float64Array.of(0, NaN, NaN, NaN, NaN);
  }
  private agentAt(x: number, y: number): number | undefined {
    return [...this.agents].find(([, p]) => p[0] === x && p[1] === y)?.[0];
  }
  inspect(x: number, y: number): string {
    if (x >= this.config.width || y >= this.config.height) throw fieldError('edit', `(${x}, ${y}) is off the grid`);
    const id = this.agentAt(x, y);
    return JSON.stringify({ site: { x, y, resources: [1], capacities: [4], pollution: [] }, agent: id === undefined ? null : { id } });
  }
  locate(id: number): Uint32Array | undefined {
    const p = this.agents.get(id);
    return p && Uint32Array.from(p);
  }
  follow(id: number): void {
    this.followedId = id;
  }
  unfollow(): void {
    this.followedId = -1;
  }
  trail(): Uint32Array {
    const p = this.agents.get(this.followedId);
    return p ? Uint32Array.from(p) : new Uint32Array(0);
  }
  followed(): number {
    return this.followedId;
  }
  paint_capacity(_x: number, _y: number, _radius: number, value: number, good: number): void {
    if (value < 0) throw new Error('unreachable executed');
    if (good >= this.config.goods.length) throw fieldError('edit', `there is no good ${good}`);
    this.edited[good] = true;
    this.log.push(`paint ${good}`);
  }
  set_landscape(good: number, capacities: Uint8Array): void {
    if (capacities.length !== this.config.width * this.config.height) throw fieldError('edit', 'wrong size');
    this.edited[good] = true;
  }
  place_agent(x: number, y: number, overridesJson: string): number {
    if (this.agentAt(x, y) !== undefined) throw fieldError('edit', 'site is occupied');
    const id = Math.max(0, ...this.agents.keys()) + 1;
    this.agents.set(id, [x, y]);
    this.log.push(`place ${overridesJson}`);
    return id;
  }
  remove_agent(x: number, y: number): void {
    const id = this.agentAt(x, y);
    if (id === undefined) throw fieldError('edit', `no agent at (${x}, ${y})`);
    this.agents.delete(id);
  }
  infect(): boolean {
    return true;
  }
  vaccinate(): number {
    return 1;
  }
  set_config(json: string): void {
    const c = JSON.parse(json) as Partial<FakeConfig>;
    if ((c.population ?? 0) > 1000) throw fieldError('population', 'too many');
    this.config = normalize(c);
    this.log.push('setConfig');
  }
  export_config(): string {
    return JSON.stringify(this.config);
  }
  export_landscape(): Uint8Array {
    return new Uint8Array(this.config.width * this.config.height).fill(7);
  }
  landscape_edited(good: number): boolean {
    return this.edited[good] ?? false;
  }
  export_series_csv(): string {
    return `tick,population\n${this.ticks},${this.agents.size}\n`;
  }
  export_agents_csv(): string {
    return 'id\n1\n';
  }
  networks(kind: string): Uint32Array {
    return kind === 'trade' ? Uint32Array.of(0, 0, 1, 1) : new Uint32Array(0);
  }
  credit_graph(): string {
    return '{"agents":[],"loans":[]}';
  }
  disease_list(): string {
    return '[{"id":0,"bits":"01","carriers":2}]';
  }
  fingerprint(): string {
    return `0x${this.ticks.toString(16)}`;
  }
  free(): void {
    this.log.push('free');
  }
}

/** A module whose worlds are `FakeSim`s (all kept in `sims`, newest last). */
export function fakeModule(log: string[] = []): SimModule & { sims: FakeSim[] } {
  const sims: FakeSim[] = [];
  return {
    sims,
    create(configJson, seed) {
      const config = JSON.parse(configJson) as Partial<FakeConfig>;
      if ((config.population ?? 0) > 1000) throw fieldError('population', 'too many');
      const sim = new FakeSim(config, seed, log);
      sims.push(sim);
      log.push(`create ${seed}`);
      return sim;
    },
    frameBytes: (_ptr, len) => new Uint8Array(len).fill((sims.at(-1)?.ticks ?? 0) % 256),
  };
}
```

- [ ] **Step 2: Write the failing tests**

Create `web/src/sim-host.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { fakeModule } from './fake-sim.fixture';
import type { Command, DisplayState, HostReply, Wants, WorldSnapshot } from './protocol';
import { SimHost } from './sim-host';
import type { Config } from './types';

const config = { width: 4, height: 3 } as unknown as Config;
const display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: { trade: false, credit: false, disease: false } };

/** A host with a world (`init` already answered); `send` numbers requests. */
function start(clock?: () => number) {
  const log: string[] = [];
  const module = fakeModule(log);
  const host = new SimHost(module, clock);
  let id = 0;
  const send = (cmd: Command, extra: { wants?: Wants; frame?: ArrayBuffer } = {}): HostReply =>
    host.handle({ id: ++id, cmd, ...extra });
  const snap = (reply: HostReply): WorldSnapshot => {
    if (!reply.result.ok || !reply.result.snapshot) throw new Error(JSON.stringify(reply.result));
    return reply.result.snapshot;
  };
  send({ type: 'init', config, seed: 1, landscapes: [], display });
  return { host, log, module, send, snap };
}

describe('SimHost', () => {
  it('needs init before anything but ready', () => {
    const host = new SimHost(fakeModule());
    expect(host.handle({ id: 1, cmd: { type: 'ready' } }).result).toEqual({ ok: true });
    expect(host.handle({ id: 2, cmd: { type: 'step', n: 1 } }).result).toEqual({
      ok: false,
      errors: [{ field: 'world', message: 'no world yet' }],
    });
  });

  it('answers init with the world, its config and landscapes, rendered into the lent buffer', () => {
    const host = new SimHost(fakeModule());
    const frame = new ArrayBuffer(48);
    const reply = host.handle({ id: 7, cmd: { type: 'init', config, seed: 3, landscapes: [], display }, frame });
    expect(reply.id).toBe(7);
    expect(reply.spare).toBeUndefined();
    const s = (reply.result as { snapshot: WorldSnapshot }).snapshot;
    expect(s).toMatchObject({ width: 4, height: 3, tick: 0, population: 1, followed: null, followedAlive: false, editedLandscapes: [null] });
    expect(s.latest).toEqual({ tick: 0, population: 1 });
    expect(s.config?.goods).toHaveLength(1);
    expect(s.frame).toBe(frame);
  });

  it('renders into the lent buffer, or a new one when the size differs, and returns what it did not use', () => {
    const t = start();
    t.send({ type: 'step', n: 3 });
    const right = new ArrayBuffer(48);
    const r1 = t.send({ type: 'refresh' }, { frame: right });
    expect(t.snap(r1).frame).toBe(right);
    expect(new Uint8Array(right)[0]).toBe(3);
    const small = new ArrayBuffer(8);
    const r2 = t.send({ type: 'refresh' }, { frame: small });
    expect(t.snap(r2).frame?.byteLength).toBe(48);
    expect(r2.spare?.[0]).toBe(small);
    expect(t.snap(t.send({ type: 'refresh' })).frame).toBeUndefined();
  });

  it('adds extras only when wanted', () => {
    const t = start();
    const bare = t.snap(t.send({ type: 'step', n: 1 }));
    expect(bare.tick).toBe(1);
    const keys = [
      'inspection', 'trail', 'networks', 'charts', 'lorenz', 'wealthHist', 'supplyDemand',
      'creditGraph', 'diseaseList', 'config', 'editedLandscapes', 'display', 'frame',
    ] as const;
    for (const key of keys) expect(bare[key], key).toBeUndefined();
    const all: Wants = {
      select: { x: 0, y: 0, agentId: null },
      trail: true,
      networks: ['trade'],
      charts: { groups: [['population']], max: 10 },
      lorenz: true,
      wealthHist: true,
      supplyDemand: true,
      creditGraph: true,
      diseaseList: true,
    };
    const full = t.snap(t.send({ type: 'step', n: 1 }, { wants: all }));
    expect(full.inspection?.view.site).toMatchObject({ x: 0, y: 0 });
    expect(full.trail).toBeDefined();
    expect(full.networks?.trade).toEqual(Uint32Array.of(0, 0, 1, 1));
    expect(Object.keys(full.charts ?? {})).toEqual(['population']);
    expect(full.lorenz).toHaveLength(101);
    expect(full.wealthHist).toHaveLength(21);
    expect(full.supplyDemand).toHaveLength(5);
    expect(full.creditGraph).toEqual({ agents: [], loans: [] });
    expect(full.diseaseList).toEqual([{ id: 0, bits: '01', carriers: 2 }]);
  });

  it('sends a chart group only with news: 250 ms later or 1 % more history, and afresh after a config change', () => {
    let clock = 0;
    const t = start(() => clock);
    const charts = { groups: [['population', 'gini']], max: 50 };
    const key = 'population|gini';
    const step = (n: number) => t.snap(t.send({ type: 'step', n }, { wants: { charts } })).charts?.[key];
    const first = step(999); // history: 1000 ticks
    expect(first?.ticks).toHaveLength(50);
    expect(first?.columns).toHaveLength(2);
    clock = 10;
    expect(step(1)).toBeUndefined(); // 0.1 % more, 10 ms later
    expect(step(10)).toBeDefined(); // 1011 > 1010: more than 1 % more
    clock = 300;
    expect(step(1)).toBeDefined(); // 290 ms later
    clock = 600;
    expect(t.snap(t.send({ type: 'refresh' }, { wants: { charts } })).charts).toBeUndefined(); // no new ticks
    expect(t.snap(t.send({ type: 'setConfig', config }, { wants: { charts } })).charts?.[key]).toBeDefined();
    const reset = t.snap(t.send({ type: 'reset', config, seed: 2, landscapes: [] }, { wants: { charts } }));
    expect(reset.charts?.[key]?.ticks).toEqual(Float64Array.of(0));
  });

  it('leaves out extras the world cannot give instead of failing the command', () => {
    const t = start();
    const s = t.snap(
      t.send({ type: 'setConfig', config }, { wants: { charts: { groups: [['nope']], max: 10 }, select: { x: 99, y: 99, agentId: null } } }),
    );
    expect(s.config).toBeDefined();
    expect(s.charts).toBeUndefined();
    expect(s.inspection).toBeUndefined();
  });

  it('answers field errors in the core shape, keeps the world and returns the buffer', () => {
    const t = start();
    t.send({ type: 'step', n: 2 });
    const frame = new ArrayBuffer(48);
    const bad = { ...config, population: 5000 };
    const r = t.send({ type: 'setConfig', config: bad }, { frame });
    expect(r.result).toEqual({ ok: false, errors: [{ field: 'population', message: 'too many' }] });
    expect(r.spare?.[0]).toBe(frame);
    expect(t.send({ type: 'reset', config: bad, seed: 1, landscapes: [] }).result.ok).toBe(false);
    expect(t.module.sims).toHaveLength(1);
    expect(t.snap(t.send({ type: 'refresh' })).tick).toBe(2);
    expect(t.send({ type: 'erase', x: 3, y: 2 }).result).toEqual({
      ok: false,
      errors: [{ field: 'edit', message: 'no agent at (3, 2)' }],
    });
  });

  it('turns a panic into a fatal reply and refuses every later command', () => {
    const t = start();
    const r = t.send({ type: 'paint', x: 0, y: 0, radius: 1, value: -1, good: 0 });
    expect(r.result).toEqual({ ok: false, fatal: 'The simulation stopped: unreachable executed' });
    expect(t.send({ type: 'step', n: 1 }).result).toEqual(r.result);
  });

  it('inspects by site or agent id and keeps a selected agent tracked', () => {
    const t = start();
    expect(t.snap(t.send({ type: 'inspect', target: { x: 1, y: 1 } })).inspection).toMatchObject({ x: 1, y: 1, agentId: 1, alive: true });
    const moved = t.snap(t.send({ type: 'step', n: 2 }, { wants: { select: { x: 1, y: 1, agentId: 1 } } })).inspection;
    expect(moved).toMatchObject({ x: 3, y: 1, agentId: 1, alive: true });
    expect(t.snap(t.send({ type: 'inspect', target: { agentId: 1 } })).inspection).toMatchObject({ x: 3, y: 1 });
    t.send({ type: 'erase', x: 3, y: 1 });
    const gone = t.snap(t.send({ type: 'refresh' }, { wants: { select: { x: 3, y: 1, agentId: 1 } } })).inspection;
    expect(gone).toMatchObject({ x: 3, y: 1, agentId: 1, alive: false });
    expect(t.snap(t.send({ type: 'inspect', target: { agentId: 1 } })).inspection).toBeNull();
  });

  it('follows with the trail included, and clamps the display to the config', () => {
    const t = start();
    const followed = t.snap(t.send({ type: 'follow', id: 1 }));
    expect(followed).toMatchObject({ followed: 1, followedAlive: true });
    expect(Array.from(followed.trail ?? [])).toEqual([1, 1]);
    const clamped = t.snap(t.send({ type: 'setDisplay', display: { ...display, colorMode: 'disease', layer: 'capacity:3' } }));
    expect(clamped.display).toEqual(display);
  });

  it('reports a scheduled change as a config change, and edited landscapes after edits', () => {
    const t = start();
    t.send({ type: 'setConfig', config: { ...config, schedule: [{ tick: 5, set: {} }] } });
    expect(t.snap(t.send({ type: 'step', n: 4 })).config).toBeUndefined(); // ticks 0–3 started
    expect(t.snap(t.send({ type: 'step', n: 2 })).config?.schedule).toHaveLength(1); // the step from 5 started
    expect(t.snap(t.send({ type: 'paint', x: 0, y: 0, radius: 1, value: 3, good: 0 })).editedLandscapes).toEqual([
      new Uint8Array(12).fill(7),
    ]);
    expect(t.snap(t.send({ type: 'place', x: 0, y: 2, overrides: {} })).editedLandscapes).toBeUndefined();
  });

  it('answers exports and the fingerprint with values', () => {
    const t = start();
    t.send({ type: 'step', n: 26 });
    expect(t.send({ type: 'seriesCsv' }).result).toEqual({ ok: true, value: 'tick,population\n26,1\n' });
    expect(t.send({ type: 'agentsCsv' }).result).toEqual({ ok: true, value: 'id\n1\n' });
    expect(t.send({ type: 'fingerprint' }).result).toEqual({ ok: true, value: '0x1a' });
  });
});
```

- [ ] **Step 3: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/sim-host.test.ts)`
Expected: FAIL — `Failed to resolve import "./sim-host"`.

- [ ] **Step 4: Implement**

Create `web/src/sim-host.ts`:
```ts
import type { CreditGraph } from './credit';
import { clampDisplay } from './layers';
import {
  chartKey,
  type ChartGroup,
  type DisplayState,
  type HostReply,
  type HostRequest,
  type Overlay,
  type Result,
  type SelectQuery,
  type Selected,
  type Wants,
  type WorldSnapshot,
} from './protocol';
import { parseErrors, type Config, type DiseaseEntry, type Inspection, type Snapshot } from './types';

/** The part of the WASM `Sim` a host uses. The real class satisfies it; tests pass `FakeSim`. */
export interface SimLike {
  step(n: number): void;
  tick(): number;
  width(): number;
  height(): number;
  population(): number;
  render(colorMode: string, layer: string): number;
  frame_len(): number;
  stats_latest(): string;
  series_group(namesJson: string, max: number): Float64Array;
  lorenz(points: number): Float64Array;
  wealth_hist(bins: number): Float64Array;
  supply_demand(): Float64Array;
  inspect(x: number, y: number): string;
  locate(id: number): Uint32Array | undefined;
  follow(id: number): void;
  unfollow(): void;
  trail(): Uint32Array;
  followed(): number;
  paint_capacity(x: number, y: number, radius: number, value: number, good: number): void;
  set_landscape(good: number, capacities: Uint8Array): void;
  place_agent(x: number, y: number, overridesJson: string): number;
  remove_agent(x: number, y: number): void;
  infect(x: number, y: number, disease: number): boolean;
  vaccinate(x: number, y: number, radius: number, disease: number): number;
  set_config(json: string): void;
  export_config(): string;
  export_landscape(good: number): Uint8Array;
  landscape_edited(good: number): boolean;
  export_series_csv(): string;
  export_agents_csv(): string;
  networks(kind: string): Uint32Array;
  credit_graph(): string;
  disease_list(): string;
  fingerprint(): string;
  free(): void;
}

/** Makes worlds and reads their frames; `wasmSimModule` (sim-module.ts) is the real one. */
export interface SimModule {
  create(configJson: string, seed: number, landscapes: (Uint8Array | null)[]): SimLike;
  /** The `len` bytes `render` just wrote at `ptr` (a view into WASM memory: copy it at once). */
  frameBytes(ptr: number, len: number): Uint8Array;
}

/** A chart group is sent again once this long has passed… */
export const CHART_MS = 250;
/** …or once its history has grown by more than this fraction. */
export const CHART_GROWTH = 0.01;

const NO_WORLD = JSON.stringify([{ field: 'world', message: 'no world yet' }]);

/**
 * An extra the world cannot give — a chart line or a selected site that a config change just
 * removed — is left out of the snapshot instead of failing the command. Only field errors (thrown
 * strings) are dropped; anything else is a panic and propagates.
 */
function optional<T>(get: () => T): T | undefined {
  try {
    return get();
  } catch (e) {
    if (typeof e === 'string') return undefined;
    throw e;
  }
}

/**
 * Holds the simulation and answers one command at a time, in arrival order. It runs in the
 * simulation worker (sim-worker.ts) or, in tests and as a fallback, on the page (InlineTransport).
 */
export class SimHost {
  private sim: SimLike | null = null;
  private config: Config | null = null;
  private display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: { trade: false, credit: false, disease: false } };
  /** The next snapshot carries the config: after init, reset, setConfig or a scheduled change. */
  private configDue = false;
  /** The next snapshot carries the edited landscapes: after init, reset, setConfig, paint or import. */
  private landscapesDue = false;
  /** Set by a panic: every later command is answered with it. */
  private dead: string | null = null;
  /** When, and at which history length, each chart group was last sent. */
  private sent = new Map<string, { at: number; length: number }>();

  constructor(
    private module: SimModule,
    private now: () => number = () => performance.now(),
  ) {}

  handle(req: HostRequest): HostReply {
    let result: Result;
    if (this.dead) {
      result = { ok: false, fatal: this.dead };
    } else {
      try {
        result = this.apply(req);
      } catch (e) {
        // The core throws field errors as JSON strings; anything else is a panic or a bug.
        result = typeof e === 'string' ? { ok: false, errors: parseErrors(e) } : { ok: false, fatal: this.fail(e) };
      }
    }
    // A lent buffer the snapshot did not use goes straight back.
    const used = result.ok && result.snapshot?.frame === req.frame;
    return req.frame && !used ? { id: req.id, result, spare: [req.frame] } : { id: req.id, result };
  }

  /** Stops the host for good; returns the message every later command gets. */
  fail(e: unknown): string {
    this.dead = `The simulation stopped: ${e instanceof Error ? e.message : String(e)}`;
    return this.dead;
  }

  private apply(req: HostRequest): Result {
    const { cmd, frame } = req;
    const wants = req.wants ?? {};
    if (cmd.type === 'ready') return { ok: true };
    if (cmd.type === 'init' || cmd.type === 'reset') {
      // Built before the old world is freed: a bad config keeps the world.
      const next = this.module.create(JSON.stringify(cmd.config), cmd.seed, cmd.landscapes);
      this.sim?.free();
      this.sim = next;
      if (cmd.type === 'init') this.display = cmd.display;
      this.configDue = true;
      this.landscapesDue = true;
      // The engine clears its selection on a new world.
      return this.reply(next, { ...wants, select: undefined }, frame);
    }
    const sim = this.sim;
    if (!sim) throw NO_WORLD;
    switch (cmd.type) {
      case 'setConfig':
        sim.set_config(JSON.stringify(cmd.config));
        this.configDue = true;
        this.landscapesDue = true;
        return this.reply(sim, wants, frame);
      case 'step': {
        const from = sim.tick();
        sim.step(cmd.n);
        this.fired(from, sim.tick());
        return this.reply(sim, wants, frame);
      }
      case 'refresh':
        return this.reply(sim, wants, frame);
      case 'setDisplay':
        this.display = cmd.display;
        return this.reply(sim, wants, frame);
      case 'paint':
        sim.paint_capacity(cmd.x, cmd.y, cmd.radius, cmd.value, cmd.good);
        this.landscapesDue = true;
        return this.reply(sim, wants, frame);
      case 'importLandscape':
        sim.set_landscape(cmd.good, cmd.capacities);
        this.landscapesDue = true;
        return this.reply(sim, wants, frame);
      case 'place':
        sim.place_agent(cmd.x, cmd.y, JSON.stringify(cmd.overrides));
        return this.reply(sim, wants, frame);
      case 'erase':
        sim.remove_agent(cmd.x, cmd.y);
        return this.reply(sim, wants, frame);
      case 'infect':
        sim.infect(cmd.x, cmd.y, cmd.disease);
        return this.reply(sim, wants, frame);
      case 'vaccinate':
        sim.vaccinate(cmd.x, cmd.y, cmd.radius, cmd.disease);
        return this.reply(sim, wants, frame);
      case 'follow':
        if (cmd.id === null) sim.unfollow();
        else sim.follow(cmd.id);
        return this.reply(sim, { ...wants, trail: cmd.id !== null }, frame);
      case 'inspect': {
        const { target } = cmd;
        const at = 'agentId' in target ? sim.locate(target.agentId) : Uint32Array.of(target.x, target.y);
        return this.reply(sim, wants, frame, at ? this.selectAt(sim, at[0], at[1]) : null);
      }
      case 'seriesCsv':
        return { ok: true, value: sim.export_series_csv() };
      case 'agentsCsv':
        return { ok: true, value: sim.export_agents_csv() };
      case 'fingerprint':
        return { ok: true, value: sim.fingerprint() };
    }
  }

  /** An entry at tick t fires when the step from t to t + 1 starts. */
  private fired(from: number, to: number): void {
    if (this.config?.schedule.some((c) => c.tick >= from && c.tick < to)) this.configDue = true;
  }

  private reply(sim: SimLike, wants: Wants, frame?: ArrayBuffer, selected?: Selected | null): Result {
    return { ok: true, snapshot: this.snapshot(sim, wants, frame, selected) };
  }

  /** `selected` is an `inspect` command's answer; it replaces `wants.select`. */
  private snapshot(sim: SimLike, wants: Wants, frame: ArrayBuffer | undefined, selected?: Selected | null): WorldSnapshot {
    const id = sim.followed();
    const s: WorldSnapshot = {
      width: sim.width(),
      height: sim.height(),
      tick: sim.tick(),
      population: sim.population(),
      latest: JSON.parse(sim.stats_latest()) as Snapshot,
      followed: id < 0 ? null : id,
      followedAlive: id >= 0 && sim.locate(id) !== undefined,
    };
    let config = this.config;
    if (this.configDue || !config) {
      config = JSON.parse(sim.export_config()) as Config;
      this.config = config;
      s.config = config;
      this.configDue = false;
      // Chart lines follow the config: send every group afresh.
      this.sent.clear();
    }
    if (this.landscapesDue) {
      s.editedLandscapes = config.goods.map((_, i) => (sim.landscape_edited(i) ? sim.export_landscape(i) : null));
      this.landscapesDue = false;
    }
    const display = clampDisplay(this.display, config);
    if (display !== this.display) {
      this.display = display;
      s.display = display;
    }
    if (frame) s.frame = this.render(sim, frame);
    const select = wants.select;
    if (selected !== undefined) s.inspection = selected;
    else if (select) s.inspection = optional(() => this.track(sim, select));
    if (wants.trail) s.trail = sim.trail();
    if (wants.networks) {
      const networks: Partial<Record<Overlay, Uint32Array>> = {};
      for (const kind of wants.networks) networks[kind] = sim.networks(kind);
      s.networks = networks;
    }
    if (wants.charts) {
      const charts = this.charts(sim, wants.charts.groups, wants.charts.max);
      if (charts) s.charts = charts;
    }
    if (wants.lorenz) s.lorenz = sim.lorenz(101);
    if (wants.wealthHist) s.wealthHist = sim.wealth_hist(20);
    if (wants.supplyDemand) s.supplyDemand = sim.supply_demand();
    if (wants.creditGraph) s.creditGraph = JSON.parse(sim.credit_graph()) as CreditGraph;
    if (wants.diseaseList) s.diseaseList = JSON.parse(sim.disease_list()) as DiseaseEntry[];
    return s;
  }

  /** Copies the rendered frame into the lent buffer, or a new one when the grid size changed. */
  private render(sim: SimLike, frame: ArrayBuffer): ArrayBuffer {
    const ptr = sim.render(this.display.colorMode, this.display.layer);
    const bytes = this.module.frameBytes(ptr, sim.frame_len());
    const out = frame.byteLength === bytes.length ? frame : new ArrayBuffer(bytes.length);
    new Uint8Array(out).set(bytes);
    return out;
  }

  private selectAt(sim: SimLike, x: number, y: number): Selected {
    const view = JSON.parse(sim.inspect(x, y)) as Inspection;
    const agentId = view.agent?.id ?? null;
    return { x, y, agentId, alive: agentId !== null, view };
  }

  /** The selection now: a selected agent's current site while it lives, else the selected site. */
  private track(sim: SimLike, q: SelectQuery): Selected {
    const at = q.agentId === null ? undefined : sim.locate(q.agentId);
    const x = at ? at[0] : q.x;
    const y = at ? at[1] : q.y;
    return { x, y, agentId: q.agentId, alive: at !== undefined, view: JSON.parse(sim.inspect(x, y)) as Inspection };
  }

  /** The groups with news (Decision 4); undefined when none has any. */
  private charts(sim: SimLike, groups: string[][], max: number): Record<string, ChartGroup> | undefined {
    const length = sim.tick() + 1;
    const now = this.now();
    let out: Record<string, ChartGroup> | undefined;
    for (const names of groups) {
      const key = chartKey(names);
      const last = this.sent.get(key);
      if (last && (length === last.length || (now - last.at < CHART_MS && length <= last.length * (1 + CHART_GROWTH)))) continue;
      const flat = optional(() => sim.series_group(JSON.stringify(names), max));
      if (!flat) continue;
      const n = flat[0];
      (out ??= {})[key] = {
        ticks: flat.slice(1, 1 + n),
        columns: names.map((_, k) => flat.slice(1 + n * (k + 1), 1 + n * (k + 2))),
      };
      this.sent.set(key, { at: now, length });
    }
    return out;
  }
}
```

- [ ] **Step 5: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/sim-host.test.ts)`
Expected: `12 passed`.

- [ ] **Step 6: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds (tsc checks `FakeSim implements SimLike`); all test files pass.

- [ ] **Step 7: Commit**

```bash
git add web/src/sim-host.ts web/src/sim-host.test.ts web/src/fake-sim.fixture.ts
git commit -m "Add the simulation host that answers commands with snapshots" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 5: Transports

*Mechanical (full code).* Browser: nothing to check (not used yet).

**Files:**
- Modify: `web/src/sim-host.ts`, `web/src/sim-host.test.ts`
- Create: `web/src/transport.ts`, `web/src/transport.test.ts`

**Interfaces:**
- Consumes: `SimHost` (Task 4); `transfers`, `HostMessage`, `HostReply`, `HostRequest`, `Command`, `Wants`, `WorldSnapshot` (Task 3).
- Produces:
  - `sim-host.ts`: `serve(host: SimHost, send: (message: HostMessage, transfer: Transferable[]) => void): (req: HostRequest) => void` (Task 12 adds a third parameter)
  - `transport.ts`: `interface Transport { request(cmd, extra?): Promise<HostReply>; onPost; onFatal; close(): void }`, `interface PortLike` (the part of `Worker` used), `class PortTransport implements Transport` (`constructor(port: PortLike)`), `class InlineTransport extends PortTransport` (`constructor(readonly host: SimHost)`)

Transport rules (Decisions 1, 7): ids count from 1; a reply resolves the request with its id; unsolicited messages go to `onPost` or kill the transport; after a fatal reply, a fatal host message or a port `error`, every pending and later request resolves `{ ok: false, fatal }` at once and `onFatal` is called exactly once. `InlineTransport` delivers in both directions with `queueMicrotask`, so replies are asynchronous and in order, like a worker's.

- [ ] **Step 1: Write the failing tests**

Append to `web/src/sim-host.test.ts` (and add `serve` to its `./sim-host` import and `HostMessage` to its `./protocol` type import):
```ts
describe('serve', () => {
  it('answers each request in order and lists its buffers for transfer', () => {
    const sent: [HostMessage, Transferable[]][] = [];
    const handle = serve(new SimHost(fakeModule()), (m, t) => sent.push([m, t]));
    const frame = new ArrayBuffer(48);
    handle({ id: 1, cmd: { type: 'init', config, seed: 1, landscapes: [], display }, frame });
    handle({ id: 2, cmd: { type: 'fingerprint' } });
    expect(sent.map(([m]) => m.id)).toEqual([1, 2]);
    expect(sent[0][1]).toHaveLength(1);
    expect(sent[0][1][0]).toBe(frame);
    expect((sent[1][0] as HostReply).result).toEqual({ ok: true, value: '0x0' });
  });
});
```
Create `web/src/transport.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { fakeModule } from './fake-sim.fixture';
import type { Command, DisplayState } from './protocol';
import { SimHost } from './sim-host';
import { InlineTransport, PortTransport, type PortLike } from './transport';
import type { Config } from './types';

const config = { width: 4, height: 3 } as unknown as Config;
const display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: { trade: false, credit: false, disease: false } };
const init: Command = { type: 'init', config, seed: 1, landscapes: [], display };

describe('InlineTransport', () => {
  it('resolves each request with its own reply, asynchronously and in order', async () => {
    const t = new InlineTransport(new SimHost(fakeModule()));
    const order: number[] = [];
    const a = t.request(init).then((r) => (order.push(r.id), r));
    const b = t.request({ type: 'fingerprint' }).then((r) => (order.push(r.id), r));
    expect(order).toEqual([]);
    const [ra, rb] = await Promise.all([a, b]);
    expect(ra.id).toBeLessThan(rb.id);
    expect(order).toEqual([ra.id, rb.id]);
    expect(ra.result.ok).toBe(true);
    expect(rb.result).toEqual({ ok: true, value: '0x0' });
  });

  it('reports a fatal reply once and answers later requests as fatal', async () => {
    const t = new InlineTransport(new SimHost(fakeModule()));
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    await t.request(init);
    const r = await t.request({ type: 'paint', x: 0, y: 0, radius: 0, value: -1, good: 0 });
    expect(r.result).toEqual({ ok: false, fatal: 'The simulation stopped: unreachable executed' });
    expect((await t.request({ type: 'fingerprint' })).result).toEqual(r.result);
    expect(fatal).toEqual(['The simulation stopped: unreachable executed']);
  });
});

describe('PortTransport', () => {
  it('fails pending requests when the port errors', async () => {
    const port: PortLike = { onmessage: null, onerror: null, postMessage: () => {}, terminate: () => {} };
    const t = new PortTransport(port);
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    const pending = t.request({ type: 'ready' });
    port.onerror?.({ message: 'boom' } as ErrorEvent);
    expect((await pending).result).toEqual({ ok: false, fatal: 'boom' });
    expect(fatal).toEqual(['boom']);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/sim-host.test.ts src/transport.test.ts)`
Expected: FAIL — `serve` is not exported; `Failed to resolve import "./transport"`.

- [ ] **Step 3: Implement**

In `web/src/sim-host.ts`, add `transfers` and `type HostMessage` to the `./protocol` import, and append:
```ts
/** Wires a host to a message channel: each request is answered in order, its buffers transferred. */
export function serve(host: SimHost, send: (message: HostMessage, transfer: Transferable[]) => void): (req: HostRequest) => void {
  return (req) => {
    const reply = host.handle(req);
    send(reply, transfers(reply));
  };
}
```
Create `web/src/transport.ts`:
```ts
import { transfers, type Command, type HostMessage, type HostReply, type HostRequest, type Wants, type WorldSnapshot } from './protocol';
import { serve, type SimHost } from './sim-host';

/** How the engine talks to a SimHost: each request resolves with the reply that carries its id. */
export interface Transport {
  request(cmd: Command, extra?: { wants?: Wants; frame?: ArrayBuffer }): Promise<HostReply>;
  /** Snapshots the host sends on its own (Max speed). */
  onPost: ((snapshot: WorldSnapshot) => void) | null;
  /** Called once when the host dies (a panic, or the worker failed); pending and later requests resolve as fatal. */
  onFatal: ((message: string) => void) | null;
  close(): void;
}

/** The part of a `Worker` a transport uses (the in-page stand-in and tests implement it too). */
export interface PortLike {
  onmessage: ((event: MessageEvent) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  postMessage(message: unknown, transfer: Transferable[]): void;
  terminate(): void;
}

/** Numbers requests and resolves each with its reply; buffers are transferred both ways. */
export class PortTransport implements Transport {
  onPost: ((snapshot: WorldSnapshot) => void) | null = null;
  onFatal: ((message: string) => void) | null = null;
  private next = 1;
  private pending = new Map<number, (reply: HostReply) => void>();
  private dead: string | null = null;

  constructor(private port: PortLike) {
    port.onmessage = (event) => this.receive(event.data as HostMessage);
    port.onerror = (event) => this.fail(event.message || 'the simulation worker failed');
  }

  request(cmd: Command, extra: { wants?: Wants; frame?: ArrayBuffer } = {}): Promise<HostReply> {
    const id = this.next++;
    if (this.dead) return Promise.resolve({ id, result: { ok: false, fatal: this.dead } });
    const req: HostRequest = { id, cmd, ...extra };
    return new Promise((resolve) => {
      this.pending.set(id, resolve);
      this.port.postMessage(req, transfers(req));
    });
  }

  close(): void {
    this.port.terminate();
  }

  private receive(message: HostMessage): void {
    if (message.id === null) {
      if ('fatal' in message) this.fail(message.fatal);
      else this.onPost?.(message.post);
      return;
    }
    const resolve = this.pending.get(message.id);
    this.pending.delete(message.id);
    resolve?.(message);
    if (!message.result.ok && 'fatal' in message.result) this.fail(message.result.fatal);
  }

  private fail(message: string): void {
    if (this.dead) return;
    this.dead = message;
    for (const [id, resolve] of this.pending) resolve({ id, result: { ok: false, fatal: message } });
    this.pending.clear();
    this.onFatal?.(message);
  }
}

/** A worker stand-in on this thread: messages go both ways asynchronously and in order. */
function inlinePort(host: SimHost): PortLike {
  const port: PortLike = {
    onmessage: null,
    onerror: null,
    postMessage: (message) => queueMicrotask(() => handle(message as HostRequest)),
    terminate: () => {},
  };
  const handle = serve(host, (message) => queueMicrotask(() => port.onmessage?.({ data: message } as MessageEvent)));
  return port;
}

/** Runs the host on the page: in tests, and when a module worker cannot start. */
export class InlineTransport extends PortTransport {
  constructor(readonly host: SimHost) {
    super(inlinePort(host));
  }
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/sim-host.test.ts src/transport.test.ts)`
Expected: `Test Files 2 passed`.

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/sim-host.ts web/src/sim-host.test.ts web/src/transport.ts web/src/transport.test.ts
git commit -m "Add the page and port transports to the simulation host" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 6: The engine on a transport

*Needs judgment (full code given; the point is the ordering of awaits, adoption and events — read the Migration map first).* The host still runs on the page (`InlineTransport`), so this task changes the engine's shape, not where the simulation runs.

Browser (controller): every playground scenario — Play/Pause/Step and each speed, Reset and 🎲, preset change, rule edits (live and reset-requiring, including an invalid value showing its error), painting and image import, place/erase/infect/vaccinate, Inspect and Follow, overlays, Charts, Credit tab, Share + reload, all four exports, the Experiments switch.

**Files:**
- Create: `web/src/sim-module.ts`, `web/src/engine.test.ts`
- Rewrite: `web/src/engine.ts`
- Modify: `web/src/sim-host.ts` (temporary `currentSim`), `web/src/main.ts`, `web/src/ui/toolbar.ts`, `web/src/ui/rules-panel.ts`, `web/src/ui/tools.ts`, `web/src/ui/grid-view.ts`

**Interfaces:**
- Consumes: `SimHost`, `SimModule`, `SimLike` (Task 4); `Transport`, `InlineTransport` (Task 5); protocol types, `mergeWants`, `OVERLAYS` (Task 3); `Sim`, `init`, `presets_json` (wasm-pkg, with Task 2's methods).
- Produces:
  - `sim-module.ts`: `wasmSimModule(memory: WebAssembly.Memory): SimModule`
  - `sim-host.ts`: `SimHost.currentSim(): SimLike | null` — **temporary**, removed in Task 10
  - `engine.ts` (see the Migration map): `type EngineEvent` (adds `'snapshot' | 'crash'`), `interface EngineDeps { presets: Preset[]; transport: Transport }`, `type WantsProvider = (now: number) => Wants`, re-exports `Overlay`, `PlaceOverrides`; `class Engine` with fields `running, stepsPerFrame, colorMode, layer, overlays, selection, inspection, presetId, baseConfig, config, tick, population, latest, last, crashed, seed, presets`; `static create(initial?: InitialState, deps?: EngineDeps): Promise<Engine>`; `on`, `want(provider): () => void`, `size()`, `frame(): Uint8ClampedArray<ArrayBuffer> | null`, `pump(now?)`, `refresh()`, `reset`, `applyConfig`, `loadPreset`, `isModified()`, `setRunning`, `advance(n?)`, `setDisplay`, `select`, `selectAgent`, `followAgent`, `unfollow`, `followed()`, `followedAlive()`, `trail()`, `networks(kind)`, `editedLandscapes()`, `paint`, `importLandscape`, `place`, `erase`, `infect`, `vaccinate`, `seriesCsv`, `agentsCsv`, `fingerprint`; **temporary** `get sim(): Sim`, `inspect(x, y)`, `diseaseList()`, `creditGraph()` (Decision 12).

- [ ] **Step 1: Write the failing tests**

Create `web/src/engine.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { Engine, type EngineEvent } from './engine';
import { fakeModule } from './fake-sim.fixture';
import { SimHost } from './sim-host';
import { InlineTransport } from './transport';
import type { Config, Preset } from './types';

const config = { width: 4, height: 3 } as unknown as Config;
const presets: Preset[] = [{ id: 'ii-2-unit', name: 'Unit', source: 'II-2', description: '', config }];
/** Lets queued microtasks and zero-delay timers run. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

async function setup() {
  const log: string[] = [];
  const module = fakeModule(log);
  const engine = await Engine.create({ config, seed: 7 }, { presets, transport: new InlineTransport(new SimHost(module)) });
  return { engine, log, module };
}

describe('Engine', () => {
  it('starts from the init snapshot', async () => {
    const { engine } = await setup();
    expect(engine.tick).toBe(0);
    expect(engine.population).toBe(1);
    expect(engine.size()).toEqual({ width: 4, height: 3 });
    expect(engine.frame()).toHaveLength(48);
    expect(engine.config.width).toBe(4);
    expect(engine.baseConfig).toEqual(engine.config);
    expect(engine.seed).toBe(7);
  });

  it('keeps at most one step outstanding', async () => {
    const { engine, module } = await setup();
    engine.setRunning(true);
    engine.pump(0);
    engine.pump(1);
    engine.pump(2);
    await settle();
    expect(module.sims[0].stepCalls).toBe(1);
    engine.pump(3);
    await settle();
    expect(module.sims[0].stepCalls).toBe(2);
    expect(engine.tick).toBe(2);
  });

  it('holds the frame loop while a reset waits for the step in flight', async () => {
    const { engine, module } = await setup();
    engine.setRunning(true);
    engine.pump(0);
    const done = engine.reset();
    engine.pump(1);
    expect(await done).toBeNull();
    expect(module.sims).toHaveLength(2);
    expect(module.sims[0].stepCalls).toBe(1);
    expect(module.sims[1].stepCalls).toBe(0);
    expect(engine.tick).toBe(0);
    engine.pump(2);
    await settle();
    expect(module.sims[1].stepCalls).toBe(1);
  });

  it('fires events only when replies arrive', async () => {
    const { engine } = await setup();
    const seen: EngineEvent[] = [];
    for (const event of ['edit', 'snapshot'] as const) engine.on(event, () => seen.push(event));
    const pending = engine.place(0, 2, {});
    expect(seen).toEqual([]);
    expect(await pending).toBeNull();
    expect(seen).toEqual(['edit', 'snapshot']);
    expect(engine.population).toBe(2);
  });

  it("resolves writes with the core's errors", async () => {
    const { engine } = await setup();
    expect(await engine.applyConfig((c) => void (c.population = 5000))).toEqual([{ field: 'population', message: 'too many' }]);
    expect(engine.baseConfig.population).toBe(10);
    expect(await engine.reset({ ...engine.baseConfig, population: 5000 })).toEqual([{ field: 'population', message: 'too many' }]);
    expect(await engine.erase(3, 2)).toEqual([{ field: 'edit', message: 'no agent at (3, 2)' }]);
    expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
    expect(engine.config.population).toBe(20);
    expect(engine.baseConfig.population).toBe(20);
  });

  it('ping-pongs two frame buffers', async () => {
    const { engine } = await setup();
    const shown: ArrayBuffer[] = [];
    for (let i = 0; i < 4; i++) {
      await engine.advance(1);
      shown.push(engine.frame()!.buffer);
    }
    expect(shown[0]).not.toBe(shown[1]);
    expect(shown[2]).toBe(shown[0]);
    expect(shown[3]).toBe(shown[1]);
    expect(engine.frame()![0]).toBe(4);
  });

  it('keeps the selection on a selected agent as it moves', async () => {
    const { engine } = await setup();
    await engine.select(1, 1);
    expect(engine.selection).toEqual({ x: 1, y: 1, agentId: 1 });
    await engine.advance(1);
    expect(engine.selection).toEqual({ x: 2, y: 1, agentId: 1 });
    expect(engine.inspection?.alive).toBe(true);
    await engine.reset();
    expect(engine.selection).toBeNull();
    expect(engine.inspection).toBeNull();
  });

  it('stops for good after a panic', async () => {
    const { engine } = await setup();
    const crashes: string[] = [];
    engine.on('crash', () => crashes.push(engine.crashed ?? ''));
    engine.setRunning(true);
    const message = 'The simulation stopped: unreachable executed';
    expect(await engine.paint(0, 0, 1, -1)).toEqual([{ field: 'simulation', message }]);
    expect(crashes).toEqual([message]);
    expect(engine.running).toBe(false);
    expect(await engine.place(0, 2, {})).toEqual([{ field: 'simulation', message }]);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: FAIL — TypeScript/Vitest errors such as `Engine.create` taking no second argument's transport, `engine.tick` undefined, `engine.pump is not a function`.

- [ ] **Step 3: The WASM module adapter and the temporary hatch in the host**

Create `web/src/sim-module.ts`:
```ts
import type { SimLike, SimModule } from './sim-host';
import { Sim } from './wasm-pkg/sugarscape.js';

/** The real WASM `Sim` behind a SimHost; frames are read from `memory`, the instance's memory. */
export function wasmSimModule(memory: WebAssembly.Memory): SimModule {
  return {
    create: (configJson, seed, landscapes): SimLike => new Sim(configJson, seed, landscapes),
    frameBytes: (ptr, len) => new Uint8Array(memory.buffer, ptr, len),
  };
}
```
In `web/src/sim-host.ts`, add inside `class SimHost`, after the constructor:
```ts
  /** @deprecated Temporary (Task 10 removes it): the world, for `engine.sim` while panels migrate. */
  currentSim(): SimLike | null {
    return this.sim;
  }
```

- [ ] **Step 4: Rewrite the engine**

Replace `web/src/engine.ts` with:
```ts
import type { CreditGraph } from './credit';
import {
  mergeWants,
  OVERLAYS,
  type Command,
  type DisplayState,
  type Overlay,
  type PlaceOverrides,
  type Result,
  type Selected,
  type Wants,
  type WorldSnapshot,
} from './protocol';
import { SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import { InlineTransport, type Transport } from './transport';
import type { ColorMode, Config, DiseaseEntry, FieldError, Inspection, Layer, Preset, Snapshot } from './types';
import init, { presets_json, type Sim } from './wasm-pkg/sugarscape.js';

export type { Overlay, PlaceOverrides } from './protocol';

export type EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit' | 'follow' | 'snapshot' | 'crash';

export interface Selection { x: number; y: number; agentId: number | null }

export interface InitialState { config: Config; seed: number; landscapes?: (Uint8Array | null)[] }

/** What an engine runs on: the presets and a transport to a SimHost (tests pass fakes). */
export interface EngineDeps { presets: Preset[]; transport: Transport }

/**
 * What a panel needs in the next snapshot. It is called before every request, so it must not
 * change state: a panel records what it received when the snapshot arrives.
 */
export type WantsProvider = (now: number) => Wants;

/** While paused, extras a panel wants are fetched at most this often. */
const REFRESH_MS = 250;
/** Frame buffers kept for reuse besides the one on screen. */
const MAX_SPARE = 4;
const NO_CELLS = new Uint32Array(0);

export function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** A failed result's errors in the core's shape (a dead simulation is one error), or null on success. */
function failure(result: Result): FieldError[] | null {
  if (result.ok) return null;
  return 'errors' in result ? result.errors : [{ field: 'simulation', message: result.fatal }];
}

/** The page keeps its own WASM instance for presets and Experiments (Decision 10); for now the host runs on it too. */
async function defaultDeps(): Promise<EngineDeps> {
  const wasm = await init();
  const presets = JSON.parse(presets_json()) as Preset[];
  return { presets, transport: new InlineTransport(new SimHost(wasmSimModule(wasm.memory))) };
}

/**
 * The playground's view of the simulation, which runs behind a transport: reads are served from
 * the latest snapshot, writes return promises, and events fire when replies arrive.
 */
export class Engine {
  running = false;
  stepsPerFrame = 1;
  colorMode: ColorMode = 'tribe';
  layer: Layer = 'resource:0';
  overlays: Record<Overlay, boolean> = { trade: false, credit: false, disease: false };
  selection: Selection | null = null;
  /** The selected site (and agent) as of the latest snapshot. */
  inspection: Selected | null = null;
  presetId: string | null = null;
  /**
   * The setup as chosen (preset, share link, reset): what a reset rebuilds and a share link
   * carries; preset matching and `isModified()` compare against it.
   */
  baseConfig!: Config;
  /**
   * The live config: what the running world uses now, including scheduled changes that have
   * fired. The Rules panel shows this one.
   */
  config!: Config;
  tick = 0;
  population = 0;
  latest: Snapshot | null = null;
  /** The latest snapshot; its extras are there only when something wanted them. */
  last: WorldSnapshot | null = null;
  /** Why the simulation stopped for good, or null. */
  crashed: string | null = null;
  private width = 0;
  private height = 0;
  private followedId: number | null = null;
  private followedLive = false;
  private trailCells = NO_CELLS;
  private edges: Partial<Record<Overlay, Uint32Array>> = {};
  /** Each good's map where it differs from the generated one (share links; kept across resets). */
  private landscapes: (Uint8Array | null)[] = [];
  /** The frame on screen, and buffers free to lend to the host (Decision 6). */
  private shown: ArrayBuffer | null = null;
  private spare: ArrayBuffer[] = [];
  private providers = new Set<WantsProvider>();
  private listeners = new Map<EngineEvent, Set<() => void>>();
  /** The frame loop's step or refresh while it is outstanding. */
  private inFlight: Promise<void> | null = null;
  /** Writes that need a quiet world hold the frame loop while they run (Decision 8). */
  private holds = 0;
  private lastRefresh = -Infinity;

  private constructor(
    private transport: Transport,
    readonly presets: Preset[],
    public seed: number,
  ) {
    transport.onFatal = (message) => this.crash(message);
  }

  /** Throws the core's errors (as an Error message) if `initial` is invalid. */
  static async create(initial?: InitialState, deps?: EngineDeps): Promise<Engine> {
    const { presets, transport } = deps ?? (await defaultDeps());
    const fallback = presets.find((p) => p.id === 'ii-2-unit') ?? presets[0];
    const engine = new Engine(transport, presets, initial?.seed ?? randomSeed());
    const config = initial?.config ?? structuredClone(fallback.config);
    const landscapes = initial?.landscapes ?? [];
    const result = await engine.send({ type: 'init', config, seed: engine.seed, landscapes, display: engine.displayState() }, true);
    if (!result.ok || !result.snapshot) {
      transport.close();
      throw new Error((failure(result) ?? []).map((x) => `${x.field}: ${x.message}`).join('; '));
    }
    engine.adopt(result.snapshot);
    engine.baseConfig = structuredClone(engine.config);
    engine.presetId = engine.matchPreset();
    return engine;
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

  /** Registers what a panel needs in snapshots (Decision 9); returns its removal. */
  want(provider: WantsProvider): () => void {
    this.providers.add(provider);
    return () => {
      this.providers.delete(provider);
    };
  }

  size(): { width: number; height: number } {
    return { width: this.width, height: this.height };
  }

  /** The frame on screen (RGBA), or null before the first one. */
  frame(): Uint8ClampedArray<ArrayBuffer> | null {
    return this.shown ? new Uint8ClampedArray(this.shown) : null;
  }

  /** Called every animation frame: asks for the next frame's steps, or (paused) for extras a panel still wants. */
  pump(now: number = performance.now()): void {
    if (this.crashed || this.inFlight || this.holds > 0) return;
    let next: Promise<void> | null = null;
    if (this.running) next = this.stepNow(this.stepsPerFrame);
    else if (now - this.lastRefresh >= REFRESH_MS && this.providersWant(now)) next = this.refresh();
    if (next) this.inFlight = next.finally(() => (this.inFlight = null));
  }

  /** A snapshot with the current wants, without changing the world. */
  async refresh(): Promise<void> {
    this.lastRefresh = performance.now();
    const result = await this.send({ type: 'refresh' });
    if (result.ok && result.snapshot) this.accept(result.snapshot);
  }

  /**
   * Rebuilds the world, keeping each good's painted/shared map unless the grid size, the number
   * of goods or that good's map changes. On error the current world is kept and errors returned.
   */
  reset(config: Config = this.baseConfig, seed: number = this.seed): Promise<FieldError[] | null> {
    return this.quiet(() => this.rebuild(config, seed, this.keptLandscapes(config)));
  }

  /**
   * Applies a rule/parameter change to the running world: `mutate` edits a copy of the live
   * config for the world and, on success, a copy of the base config too, so scheduled changes
   * that already fired are not undone.
   */
  applyConfig(mutate: (c: Config) => void): Promise<FieldError[] | null> {
    return this.quiet(async () => {
      const next = structuredClone(this.config);
      mutate(next);
      const result = await this.send({ type: 'setConfig', config: next }, true);
      if (!result.ok || !result.snapshot) return failure(result);
      const base = structuredClone(this.baseConfig);
      mutate(base);
      this.baseConfig = base;
      this.accept(result.snapshot, ['config']);
      void this.refresh();
      return null;
    });
  }

  async loadPreset(id: string): Promise<FieldError[] | null> {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    return this.quiet(() => this.rebuild(structuredClone(preset.config), this.seed, [], id));
  }

  /** True when the base config differs from the last chosen preset or a landscape is custom. */
  isModified(): boolean {
    const preset = this.presets.find((p) => p.id === this.presetId);
    return !preset || this.landscapes.some((l) => l !== null) || JSON.stringify(preset.config) !== JSON.stringify(this.baseConfig);
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
  }

  advance(n: number = this.stepsPerFrame): Promise<void> {
    return this.stepNow(n);
  }

  setDisplay(d: { colorMode?: ColorMode; layer?: Layer; overlays?: Partial<Record<Overlay, boolean>> }): void {
    if (d.colorMode) this.colorMode = d.colorMode;
    if (d.layer) this.layer = d.layer;
    if (d.overlays) this.overlays = { ...this.overlays, ...d.overlays };
    this.emit('display');
    void this.send({ type: 'setDisplay', display: this.displayState() }, true).then((result) => {
      if (result.ok && result.snapshot) this.accept(result.snapshot);
    });
  }

  select(x: number, y: number): Promise<void> {
    return this.inspectTarget({ x, y });
  }

  /** Selects agent `id` if it is alive. */
  selectAgent(id: number): Promise<void> {
    return this.inspectTarget({ agentId: id });
  }

  /** Records and draws agent `id`'s trail from now on (a new trail replaces any other; reset clears it). */
  followAgent(id: number): Promise<void> {
    return this.follow(id);
  }

  unfollow(): Promise<void> {
    return this.follow(null);
  }

  /** The followed agent's id (alive or not), or null. */
  followed(): number | null {
    return this.followedId;
  }

  followedAlive(): boolean {
    return this.followedLive;
  }

  /** The followed agent's trail as `[x0, y0, x1, y1, …]`, oldest first. */
  trail(): Uint32Array {
    return this.trailCells;
  }

  /** Edges `[x1, y1, x2, y2, …]` of an overlay that is on (empty until its first snapshot). */
  networks(kind: Overlay): Uint32Array {
    return this.edges[kind] ?? NO_CELLS;
  }

  /** Each good's map where it differs from the generated one (for share links). */
  editedLandscapes(): (Uint8Array | null)[] | undefined {
    return this.landscapes.some((m) => m !== null) ? this.landscapes : undefined;
  }

  paint(x: number, y: number, radius: number, value: number, good = 0): Promise<FieldError[] | null> {
    return this.edit({ type: 'paint', x, y, radius, value, good });
  }

  /** Replaces good `good`'s capacity map (one byte per site, 0–10). */
  importLandscape(good: number, capacities: Uint8Array): Promise<FieldError[] | null> {
    return this.edit({ type: 'importLandscape', good, capacities });
  }

  place(x: number, y: number, overrides: PlaceOverrides): Promise<FieldError[] | null> {
    return this.edit({ type: 'place', x, y, overrides });
  }

  erase(x: number, y: number): Promise<FieldError[] | null> {
    return this.edit({ type: 'erase', x, y });
  }

  /** `disease` −1 infects with a brand-new random disease. */
  infect(x: number, y: number, disease: number): Promise<FieldError[] | null> {
    return this.edit({ type: 'infect', x, y, disease });
  }

  vaccinate(x: number, y: number, radius: number, disease: number): Promise<FieldError[] | null> {
    return this.edit({ type: 'vaccinate', x, y, radius, disease });
  }

  seriesCsv(): Promise<string> {
    return this.value({ type: 'seriesCsv' });
  }

  agentsCsv(): Promise<string> {
    return this.value({ type: 'agentsCsv' });
  }

  /** The world's fingerprint as `0x…` (the golden tests' format). */
  fingerprint(): Promise<string> {
    return this.value({ type: 'fingerprint' });
  }

  /** @deprecated Temporary (Task 10 removes it): the page's `Sim`, for panels not yet on snapshots. */
  get sim(): Sim {
    if (!(this.transport instanceof InlineTransport)) throw new Error('the simulation is not on this page');
    const sim = this.transport.host.currentSim();
    if (!sim) throw new Error('no world yet');
    return sim as unknown as Sim;
  }

  /** @deprecated Temporary (Task 8 moves the Inspect panel to `inspection`). */
  inspect(x: number, y: number): Inspection {
    return JSON.parse(this.sim.inspect(x, y)) as Inspection;
  }

  /** @deprecated Temporary (Task 9 moves the disease tools to a provider). */
  diseaseList(): DiseaseEntry[] {
    return JSON.parse(this.sim.disease_list()) as DiseaseEntry[];
  }

  /** @deprecated Temporary (Task 9 moves the Credit tab to a provider). */
  creditGraph(): CreditGraph {
    return JSON.parse(this.sim.credit_graph()) as CreditGraph;
  }

  private displayState(): DisplayState {
    return { colorMode: this.colorMode, layer: this.layer, overlays: { ...this.overlays } };
  }

  /** The engine's own wants (Decision 9) merged with every provider's. */
  private wants(now: number): Wants {
    const own: Wants = {};
    if (this.selection) own.select = { ...this.selection };
    if (this.followedId !== null) own.trail = true;
    const networks = OVERLAYS.filter((k) => this.overlays[k]);
    if (networks.length > 0) own.networks = networks;
    return mergeWants([own, ...Array.from(this.providers, (p) => p(now))]);
  }

  private providersWant(now: number): boolean {
    return Array.from(this.providers).some((p) => Object.keys(p(now)).length > 0);
  }

  /** Sends a command with the current wants, lending a frame buffer when it changes what is drawn. */
  private async send(cmd: Command, withFrame = false): Promise<Result> {
    if (this.crashed) return { ok: false, fatal: this.crashed };
    const reply = await this.transport.request(cmd, {
      wants: this.wants(performance.now()),
      frame: withFrame ? this.takeBuffer() : undefined,
    });
    for (const b of reply.spare ?? []) this.recycle(b);
    return reply.result;
  }

  /** A spare buffer of the grid's size, or a new one. */
  private takeBuffer(): ArrayBuffer {
    const size = this.width * this.height * 4;
    for (let b = this.spare.pop(); b; b = this.spare.pop()) if (b.byteLength === size) return b;
    return new ArrayBuffer(size);
  }

  /** Keeps a returned buffer for reuse while it still fits the grid. */
  private recycle(b: ArrayBuffer): void {
    if (b.byteLength > 0 && b.byteLength === this.width * this.height * 4 && this.spare.length < MAX_SPARE) this.spare.push(b);
  }

  /** Takes in a snapshot: counters, the frame, and whatever extras it carries. */
  private adopt(s: WorldSnapshot): void {
    this.last = s;
    this.width = s.width;
    this.height = s.height;
    this.tick = s.tick;
    this.population = s.population;
    this.latest = s.latest;
    this.followedId = s.followed;
    this.followedLive = s.followedAlive;
    if (s.frame) {
      if (this.shown) this.recycle(this.shown);
      this.shown = s.frame;
    }
    if (s.config) this.config = s.config;
    if (s.editedLandscapes) this.landscapes = s.editedLandscapes;
    if (s.display) {
      this.colorMode = s.display.colorMode;
      this.layer = s.display.layer;
      this.overlays = { ...s.display.overlays };
    }
    if (s.inspection) {
      this.inspection = s.inspection;
      this.selection = { x: s.inspection.x, y: s.inspection.y, agentId: s.inspection.agentId };
    }
    this.trailCells = s.trail ?? (s.followed === null ? NO_CELLS : this.trailCells);
    if (s.networks) Object.assign(this.edges, s.networks);
  }

  /** Fires `events`, then `'display'` if the host clamped the display, then `'snapshot'`. */
  private announce(s: WorldSnapshot, events: EngineEvent[]): void {
    for (const e of events) this.emit(e);
    if (s.display) this.emit('display');
    this.emit('snapshot');
  }

  private accept(s: WorldSnapshot, events: EngineEvent[] = []): void {
    this.adopt(s);
    this.announce(s, events);
  }

  private async stepNow(n: number): Promise<void> {
    const result = await this.send({ type: 'step', n }, true);
    if (!result.ok || !result.snapshot) return;
    const events: EngineEvent[] = result.snapshot.config ? ['config', 'tick'] : ['tick'];
    this.accept(result.snapshot, events);
  }

  /** Runs `fn` once the frame loop's step has settled, holding the loop until `fn` finishes (Decision 8). */
  private async quiet<T>(fn: () => Promise<T>): Promise<T> {
    this.holds++;
    try {
      await this.inFlight;
      return await fn();
    } finally {
      this.holds--;
    }
  }

  private keptLandscapes(config: Config): (Uint8Array | null)[] {
    const base = this.baseConfig;
    const same = config.width === base.width && config.height === base.height && config.goods.length === base.goods.length;
    // A changed number of goods drops every painted map: Sim needs one entry per good (or none).
    return same
      ? this.landscapes
          .slice(0, config.goods.length)
          .map((l, i) => (JSON.stringify(config.goods[i]?.map) === JSON.stringify(base.goods[i]?.map) ? l : null))
      : [];
  }

  private async rebuild(
    config: Config,
    seed: number,
    landscapes: (Uint8Array | null)[],
    presetId?: string,
  ): Promise<FieldError[] | null> {
    const result = await this.send({ type: 'reset', config, seed, landscapes }, true);
    if (!result.ok || !result.snapshot) return failure(result);
    this.seed = seed;
    this.selection = null;
    this.inspection = null;
    this.adopt(result.snapshot);
    this.baseConfig = structuredClone(this.config);
    this.presetId = presetId ?? this.matchPreset();
    this.announce(result.snapshot, ['reset']);
    // Panels' wants may have changed with the config (new chart lines, say).
    void this.refresh();
    return null;
  }

  /** The preset whose config equals the base config, if any. */
  private matchPreset(): string | null {
    const json = JSON.stringify(this.baseConfig);
    return this.presets.find((p) => JSON.stringify(p.config) === json)?.id ?? null;
  }

  private async inspectTarget(target: { x: number; y: number } | { agentId: number }): Promise<void> {
    const result = await this.send({ type: 'inspect', target });
    if (result.ok && result.snapshot?.inspection) this.accept(result.snapshot, ['select']);
  }

  private async follow(id: number | null): Promise<void> {
    const result = await this.send({ type: 'follow', id });
    if (result.ok && result.snapshot) this.accept(result.snapshot, ['follow']);
  }

  private async edit(cmd: Command): Promise<FieldError[] | null> {
    const result = await this.send(cmd, true);
    if (!result.ok) return failure(result);
    if (result.snapshot) this.accept(result.snapshot, ['edit']);
    return null;
  }

  private async value(cmd: Command): Promise<string> {
    const result = await this.send(cmd);
    if (!result.ok) throw new Error((failure(result) ?? []).map((e) => `${e.field}: ${e.message}`).join('; '));
    return result.value ?? '';
  }

  /** The host died (Decision 7): stop for good and tell the page once. */
  private crash(message: string): void {
    if (this.crashed) return;
    this.crashed = message;
    console.error(message);
    this.running = false;
    this.emit('run');
    this.emit('crash');
  }
}
```

- [ ] **Step 5: Run the engine tests**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: `8 passed` (Vitest prints the panic test's `console.error`; that is expected).

- [ ] **Step 6: Update the callers the new types require**

`web/src/ui/rules-panel.ts` — replace `commit` and the preset select's `onchange`:
```ts
  /** Reset-required changes rebuild from the base setup; others edit the running world. */
  private async commit(mutate: (c: Config) => void, reset: boolean): Promise<void> {
    if (reset) {
      const next = structuredClone(this.engine.baseConfig);
      mutate(next);
      this.errors = (await this.engine.reset(next)) ?? [];
    } else {
      this.errors = (await this.engine.applyConfig(mutate)) ?? [];
    }
    if (this.errors.length > 0) this.sync();
    this.renderErrors();
  }
```
```ts
        onchange: async () => {
          this.errors = (await this.engine.loadPreset(select.value)) ?? [];
          if (this.errors.length > 0) this.sync();
          this.renderErrors();
        },
```
(The other `this.commit(...)` call sites and the `Commit` type, `(…) => void`, accept the promise unchanged.)

`web/src/ui/tools.ts` — in the image import's `change` listener:
```ts
      const errors = await engine.importLandscape(good, capacities);
```

`web/src/ui/toolbar.ts` — replace the `step`, `reset` and `dice` buttons and the `tick` readout:
```ts
  const step = h('button', { onclick: () => void engine.advance(1), title: 'Advance one tick' }, 'Step');
```
```ts
  const reset = h('button', { onclick: () => void engine.reset(engine.baseConfig, Number(seed.value) >>> 0) }, 'Reset');
  const dice = h(
    'button',
    { title: 'Random seed and reset', onclick: () => void engine.reset(engine.baseConfig, randomSeed()) },
    '🎲',
  );
```
```ts
  const tick = () => {
    readout.textContent = `t = ${engine.tick} · ${engine.population} agents`;
  };
```

`web/src/ui/grid-view.ts` — replace `blit`'s `putImageData` line:
```ts
    const frame = this.engine.frame();
    // Until the reply that resized the grid brings a frame of the new size, the old one is not drawn.
    if (frame && frame.length === width * height * 4) this.bctx.putImageData(new ImageData(frame, width, height), 0, 0);
```

`web/src/main.ts` — in `slug`, replace `engine.sim.tick()` with `engine.tick`; replace everything from `let dirty = true;` to the end of `main` (before its closing `}`) with:
```ts
  engine.on('crash', () => showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() }));
  let dirty = true;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  const loop = (now: number) => {
    try {
      engine.pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
      charts.maybeRefresh(now);
      credit.maybeRefresh(now);
    } catch (e) {
      // A Rust panic in the page's WASM leaves it unusable; reloading keeps any #s= share state.
      engine.setRunning(false);
      console.error(e);
      showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
```
(`engine.trackSelection()` is gone: the selection follows `snapshot.inspection`.)

- [ ] **Step 7: Check what still uses the hatch**

Run: `grep -rn "engine\.sim\b\|engine\.inspect(\|engine\.diseaseList\|engine\.creditGraph\|this\.engine\.sim\b\|this\.engine\.inspect(\|this\.engine\.creditGraph" web/src --include=*.ts`
Expected: only `ui/inspect-panel.ts` (`sim.locate`, `inspect`), `ui/toolbar.ts` (`sim.locate`), `ui/grid-view.ts` (`sim.networks`), `ui/tools.ts` (`diseaseList`), `ui/credit-panel.ts` (`sim.tick`, `creditGraph`), `ui/charts-panel.ts` (`sim.tick`, `sim.series`, `sim.lorenz`, `sim.wealth_hist`, `sim.supply_demand`) and `main.ts` (`sim.export_series_csv`, `sim.export_agents_csv`). Tasks 8–10 remove them.

- [ ] **Step 8: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds (tsc also checks that the regenerated `Sim` satisfies `SimLike` in `sim-module.ts`); all test files pass.

- [ ] **Step 9: Commit**

```bash
git add web/src/sim-module.ts web/src/engine.ts web/src/engine.test.ts web/src/sim-host.ts web/src/main.ts web/src/ui/toolbar.ts web/src/ui/rules-panel.ts web/src/ui/tools.ts web/src/ui/grid-view.ts
git commit -m "Run the engine on a transport to the simulation host" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 7: Determinism through the engine, with the real WASM

*Mechanical (full code); if the fingerprint differs, stop and report — never edit the expected value.* Browser: nothing to check.

**Files:**
- Create: `web/src/node-shims.d.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: `Engine.create(initial, deps)`, `advance`, `setDisplay`, `select`, `want`, `fingerprint` (Task 6); `SimHost`, `InlineTransport`, `wasmSimModule`; wasm-pkg `initSync`, `presets_json`.
- Produces: tests only.

- [ ] **Step 1: Write the test**

Create `web/src/node-shims.d.ts`:
```ts
// The one Node API the Vitest-only WASM test uses (the web build has no @types/node).
declare module 'node:fs' {
  export function readFileSync(path: URL): Uint8Array<ArrayBuffer>;
}
```
Create `web/src/determinism.test.ts`:
```ts
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { Engine } from './engine';
import type { Overlay } from './protocol';
import { SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import { InlineTransport } from './transport';
import type { Preset } from './types';
import { initSync, presets_json } from './wasm-pkg/sugarscape.js';

// Built by `npm run build` (wasm-pack) before `npm test`.
const wasm = initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const presets = JSON.parse(presets_json()) as Preset[];
const unit = presets.find((p) => p.id === 'ii-2-unit')!;
/** crates/sugarscape-core/tests/golden.rs: ii-2-unit after 200 ticks from seed 1. */
const GOLDEN = '0x75b93943813545e4';

async function engine(): Promise<Engine> {
  const transport = new InlineTransport(new SimHost(wasmSimModule(wasm.memory)));
  return Engine.create({ config: structuredClone(unit.config), seed: 1 }, { presets, transport });
}

describe('determinism through the engine', () => {
  it('reproduces the golden ii-2-unit fingerprint', async () => {
    const e = await engine();
    await e.advance(200);
    expect(e.tick).toBe(200);
    expect(await e.fingerprint()).toBe(GOLDEN);
  });

  it('does not depend on how ticks are split into frames', async () => {
    for (const chunks of [Array.from({ length: 40 }, () => 5), [1, 2, 5, 10, 25, 100, 57]]) {
      const e = await engine();
      for (const n of chunks) await e.advance(n);
      expect(await e.fingerprint()).toBe(GOLDEN);
    }
  });

  it('is unchanged by rendering, inspection, overlays and charts', async () => {
    const e = await engine();
    e.setDisplay({ colorMode: 'wealth', overlays: { trade: true } });
    await e.select(10, 10);
    const networks: Overlay[] = ['trade', 'credit'];
    e.want(() => ({ charts: { groups: [['population', 'gini']], max: 50 }, lorenz: true, wealthHist: true, networks }));
    for (let i = 0; i < 20; i++) await e.advance(10);
    expect(await e.fingerprint()).toBe(GOLDEN);
  });
});
```

- [ ] **Step 2: Run it**

Run: `(cd web && npm run build && npx vitest run src/determinism.test.ts)`
Expected: `3 passed`. A different fingerprint means the engine path changes a run: stop and report (do not change `GOLDEN`).

- [ ] **Step 3: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds (tsc accepts the `node:fs` shim); all test files pass.

- [ ] **Step 4: Commit**

```bash
git add web/src/node-shims.d.ts web/src/determinism.test.ts
git commit -m "Check the golden fingerprint through the engine with the real WASM" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 8: Selection, following and overlays from snapshots

*Mechanical (full code).* Browser (controller): Inspect a site and an agent (the panel follows a moving agent; "Agent #n has died." after it dies), links to parents/children/loan partners, Follow → trail and the toolbar chip (with † after death; ✕ stops), the three network overlays on `iv-3-trade`, `iv-5-credit`, `v-2-endemic`.

**Files:**
- Modify: `web/src/ui/inspect-panel.ts`, `web/src/ui/grid-view.ts`, `web/src/ui/toolbar.ts`, `web/src/engine.ts`, `web/src/engine.test.ts`

**Interfaces:**
- Consumes: `engine.selection`, `engine.inspection`, `engine.followedAlive()`, `engine.networks(kind)` (Task 6).
- Produces: nothing new; removes `Engine.inspect` (and its `Inspection` import).

- [ ] **Step 1: Write the failing tests**

Append inside `describe('Engine', …)` in `web/src/engine.test.ts`:
```ts
  it('follows an agent and keeps its trail', async () => {
    const { engine } = await setup();
    await engine.followAgent(1);
    expect(engine.followed()).toBe(1);
    expect(engine.followedAlive()).toBe(true);
    expect(Array.from(engine.trail())).toEqual([1, 1]);
    await engine.advance(1);
    expect(Array.from(engine.trail())).toEqual([2, 1]);
    await engine.erase(2, 1);
    expect(engine.followedAlive()).toBe(false);
    await engine.unfollow();
    expect(engine.followed()).toBeNull();
    expect(engine.trail()).toHaveLength(0);
  });

  it('fetches the networks of the overlays that are on', async () => {
    const { engine } = await setup();
    expect(engine.networks('trade')).toHaveLength(0);
    engine.setDisplay({ overlays: { trade: true } });
    await settle();
    expect(Array.from(engine.networks('trade'))).toEqual([0, 0, 1, 1]);
  });

  it('keeps a selected agent that died selected, as gone', async () => {
    const { engine } = await setup();
    await engine.selectAgent(1);
    expect(engine.selection).toEqual({ x: 1, y: 1, agentId: 1 });
    await engine.erase(1, 1);
    expect(engine.inspection).toMatchObject({ agentId: 1, alive: false });
    await engine.selectAgent(1); // no longer alive: nothing changes
    expect(engine.selection).toEqual({ x: 1, y: 1, agentId: 1 });
  });
```

- [ ] **Step 2: Run the tests**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: `11 passed` — the engine already provides these (Task 6); they pin the behavior the panels now rely on. If any fails, fix the engine (not the test) before moving on.

- [ ] **Step 3: Move the panels onto snapshots**

`web/src/ui/inspect-panel.ts` — replace the start of `render()` up to and including the line `const { site, agent } = this.engine.inspect(sel.x, sel.y);` with:
```ts
  private render(): void {
    if (!this.visible) return;
    const sel = this.engine.selection;
    const shown = this.engine.inspection;
    if (!sel || !shown) {
      this.el.replaceChildren(h('p', { class: 'hint' }, 'Choose the Inspect tool and click an agent or site.'));
      return;
    }
    // The host tracks a selected agent while it lives (Decision 3).
    const gone = shown.agentId !== null && !shown.alive;
    const { site, agent } = shown.view;
```
and in the `replaceChildren` call below it change `` `Agent #${sel.agentId} has died.` `` to `` `Agent #${shown.agentId} has died.` ``.

`web/src/ui/grid-view.ts` — in `draw()`, replace `const e = this.engine.sim.networks(kind);` with:
```ts
      const e = this.engine.networks(kind);
```

`web/src/ui/toolbar.ts` — in `syncFollow`, replace `const alive = engine.sim.locate(id) !== undefined;` with:
```ts
    const alive = engine.followedAlive();
```

`web/src/engine.ts` — delete the deprecated `inspect(x, y)` method and remove `Inspection` from the `./types` import.

- [ ] **Step 4: Check**

Run: `grep -rn "engine\.sim\b\|\.inspect(" web/src/ui/inspect-panel.ts web/src/ui/grid-view.ts web/src/ui/toolbar.ts`
Expected: no output.

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/ui/inspect-panel.ts web/src/ui/grid-view.ts web/src/ui/toolbar.ts web/src/engine.ts web/src/engine.test.ts
git commit -m "Draw selection, trails and overlays from host snapshots" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 9: Disease list, credit graph and exports from the host

*Mechanical (full code).* Browser (controller): `v-2-endemic` — choose Infect and Vaccinate (the picker fills, grows after "New disease" infections, also while paused); `iv-5-credit` — Credit tab draws within half a second of opening (running and paused), redraws as loans change, clicking a node inspects the agent, focus survives redraws; Export → Statistics (CSV) and Agents (CSV) download complete files; Share → reload reproduces the setup including painted maps.

**Files:**
- Modify: `web/src/ui/tools.ts`, `web/src/ui/credit-panel.ts`, `web/src/main.ts`, `web/src/engine.ts`, `web/src/engine.test.ts`

**Interfaces:**
- Consumes: `engine.want`, `engine.last`, `engine.refresh`, `engine.tick`, `engine.seriesCsv/agentsCsv` (Task 6).
- Produces: `CreditPanel` no longer has `maybeRefresh`; removes `Engine.diseaseList`, `Engine.creditGraph` (and the `CreditGraph`, `DiseaseEntry` imports in engine.ts).

- [ ] **Step 1: Write the failing tests**

Append inside `describe('Engine', …)` in `web/src/engine.test.ts`:
```ts
  it('returns exports as values', async () => {
    const { engine } = await setup();
    await engine.advance(3);
    expect(await engine.seriesCsv()).toBe('tick,population\n3,1\n');
    expect(await engine.agentsCsv()).toBe('id\n1\n');
    expect(await engine.fingerprint()).toBe('0x3');
  });

  it('carries what providers want, and refreshes while paused only for them', async () => {
    const { engine } = await setup();
    const before = engine.last;
    engine.pump(1000);
    await settle();
    expect(engine.last).toBe(before);
    const stop = engine.want(() => ({ diseaseList: true, creditGraph: true }));
    engine.pump(2000);
    await settle();
    expect(engine.last?.diseaseList).toEqual([{ id: 0, bits: '01', carriers: 2 }]);
    expect(engine.last?.creditGraph).toEqual({ agents: [], loans: [] });
    stop();
    await engine.refresh();
    expect(engine.last?.diseaseList).toBeUndefined();
  });
```

- [ ] **Step 2: Run the tests**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: `13 passed` (engine behavior from Task 6, pinned before the panels depend on it).

- [ ] **Step 3: The disease picker**

In `web/src/ui/tools.ts`:
- add `import type { DiseaseEntry } from '../types';` and, below the `DISEASE_TOOLS` constant:
```ts
/** The disease list is fetched at most this often while a disease tool is open. */
const LIST_MS = 250;
```
- below `let known = -1;` add:
```ts
  /** The latest disease list the host sent (while a disease tool is open) and when it came. */
  let diseases: DiseaseEntry[] = [];
  let listAt = -Infinity;
```
- in `refreshPicker`, replace the four lines from `const list = engine.diseaseList();` through the `picker.replaceChildren(…)` line with:
```ts
    if (!force && diseases.length === known) return;
    known = diseases.length;
    picker.replaceChildren(...diseaseOptions(diseases, tool === 'infect').map(([v, l]) => h('option', { value: v }, l)));
```
- in `choose`, after `refreshPicker(true);` add:
```ts
    if (DISEASE_TOOLS.includes(tool)) {
      // Fetch the list now, even while paused.
      listAt = -Infinity;
      void engine.refresh();
    }
```
- replace `engine.on('tick', () => refreshPicker());` and `engine.on('edit', () => refreshPicker());` with:
```ts
  engine.want((now) =>
    DISEASE_TOOLS.includes(tool) && engine.config.disease.enabled && now - listAt >= LIST_MS ? { diseaseList: true } : {},
  );
  engine.on('snapshot', () => {
    const list = engine.last?.diseaseList;
    if (!list) return;
    diseases = list;
    listAt = performance.now();
    refreshPicker();
  });
```

- [ ] **Step 4: The Credit tab**

In `web/src/ui/credit-panel.ts`, change `private last = 0;` to `private last = -Infinity;`, and replace the constructor, `setVisible`, `maybeRefresh` and the head of `refresh` (down to and including `const layout = creditLayout(this.engine.creditGraph());`) with:
```ts
  constructor(private engine: Engine, private onSelect: () => void) {
    this.el.append(this.header, this.graph);
    engine.want((now) => (this.visible && now - this.last >= REFRESH_MS && this.changed() ? { creditGraph: true } : {}));
    engine.on('snapshot', () => {
      const graph = engine.last?.creditGraph;
      if (graph) this.draw(graph);
    });
    // Resets, edits and config changes can change loans without a tick.
    for (const event of ['reset', 'edit', 'config'] as const) engine.on(event, () => (this.drawn = null));
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) {
      this.drawn = null;
      this.last = -Infinity;
      void this.engine.refresh();
    }
  }

  private panelWidth(): number {
    return Math.max(240, this.graph.clientWidth || 360);
  }

  /** Whether the tick or the panel width moved since the last drawing (or nothing is drawn). */
  private changed(): boolean {
    return !this.drawn || this.drawn.tick !== this.engine.tick || this.drawn.width !== this.panelWidth();
  }

  private draw(graph: CreditGraph): void {
    this.last = performance.now();
    if (!this.visible) {
      this.drawn = null;
      return;
    }
    // Redrawing replaces every node, so skip it while nothing changed:
    // otherwise a click's mousedown and mouseup land on different elements.
    if (!this.changed()) return;
    const panelWidth = this.panelWidth();
    this.drawn = { tick: this.engine.tick, width: panelWidth };
    const layout = creditLayout(graph);
```
The rest of the former `refresh` body (from `const summary = creditHeader(layout);` on) stays as it is; the import line becomes `import { creditHeader, creditLayout, rowPositions, type CreditGraph } from '../credit';`.

- [ ] **Step 5: Exports and the frame loop**

In `web/src/main.ts`, replace the two CSV buttons with:
```ts
      h('button', { onclick: async () => downloadText(`${slug()}-series.csv`, await engine.seriesCsv()) }, 'Statistics (CSV)'),
      h('button', { onclick: async () => downloadText(`${slug()}-agents.csv`, await engine.agentsCsv()) }, 'Agents (CSV)'),
```
and delete the line `credit.maybeRefresh(now);` from the loop.

In `web/src/engine.ts`, delete the deprecated `diseaseList()` and `creditGraph()` methods, the `import type { CreditGraph } from './credit';` line, and `DiseaseEntry` from the `./types` import.

- [ ] **Step 6: Check**

Run: `grep -rn "diseaseList()\|creditGraph()\|maybeRefresh\|export_series_csv\|export_agents_csv" web/src --include=*.ts`
Expected: only `ui/charts-panel.ts` (`maybeRefresh`) and `main.ts` (`charts.maybeRefresh`) — Task 10 removes them.

- [ ] **Step 7: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 8: Commit**

```bash
git add web/src/ui/tools.ts web/src/ui/credit-panel.ts web/src/main.ts web/src/engine.ts web/src/engine.test.ts
git commit -m "Fetch the disease list, credit graph and exports from the host" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 10: Charts on downsampled history; the hatch goes

*Needs judgment (full code given; check each chart still appears under the same conditions).* Browser (controller): Charts tab on `ii-2-unit` (every time chart moves with real ticks on x; Lorenz and wealth histogram update ~4×/s and after a paint while paused), `iv-3-trade` (Trade price line with its ±SD band; gaps where no trade happened; supply & demand), `iv-5-credit` (loans, debt), `v-2-endemic` (disease section), `n-3-trade` (goods section follows the goods), `iii-6-three-tribes` (Group shares), a config change that adds a good (charts rebuild and fill at once, paused too), a run of 20 000+ ticks (charts stay smooth; x axis reaches the current tick), Export → Charts (PNG).

**Files:**
- Create: `web/src/ui/series-data.ts`, `web/src/ui/series-data.test.ts`
- Rewrite: `web/src/ui/charts-panel.ts`
- Modify: `web/src/main.ts`, `web/src/engine.ts`, `web/src/sim-host.ts`

**Interfaces:**
- Consumes: `ChartGroup`, `chartKey`, `CHART_POINTS`, `Wants`, `WorldSnapshot` (Task 3); `engine.want`, `engine.last`, `engine.refresh`, `engine.tick` (Task 6).
- Produces: `series-data.ts`: `type LineData = [number[], ...(number | null)[][]]`, `lineData(g: ChartGroup): LineData`, `bandData(g: ChartGroup): LineData`. `ChartsPanel` loses `maybeRefresh`. Removes `Engine.sim` and `SimHost.currentSim`.

Chart rules (Decisions 4, 9): each time chart asks for its group while the tab is shown and the chart is visible; the Trade price chart's group is `['mean_log_price', 'sd_log_price']`, drawn as mean, mean + SD, mean − SD; the Lorenz curve, wealth histogram and (with two goods) supply & demand are asked for at most every 250 ms when the tick moved, or after an edit, reset, config change or the tab opening. Data that arrives updates the plot even while the tab is hidden (so it never shows stale lines when reopened).

- [ ] **Step 1: Write the failing tests**

Create `web/src/ui/series-data.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { bandData, lineData } from './series-data';

const group = { ticks: Float64Array.of(0, 5, 9), columns: [Float64Array.of(1, NaN, 3), Float64Array.of(0.5, 0.5, NaN)] };

describe('series data', () => {
  it('uses the ticks as x and turns NaN into gaps', () => {
    expect(lineData(group)).toEqual([
      [0, 5, 9],
      [1, null, 3],
      [0.5, 0.5, null],
    ]);
  });

  it('draws the price band around the mean', () => {
    expect(bandData(group)).toEqual([
      [0, 5, 9],
      [1, null, 3],
      [1.5, null, null],
      [0.5, null, null],
    ]);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/ui/series-data.test.ts)`
Expected: FAIL — `Failed to resolve import "./series-data"`.

- [ ] **Step 3: Implement the data helpers**

Create `web/src/ui/series-data.ts`:
```ts
import type { ChartGroup } from '../protocol';

/** uPlot data: x values, then one array per line (null is a gap). */
export type LineData = [number[], ...(number | null)[][]];

/** NaN (no trades, no agents) is a gap in the line. */
const gap = (v: number): number | null => (Number.isNaN(v) ? null : v);

/** A time chart's data: the group's ticks as x, then one line per series. */
export function lineData(g: ChartGroup): LineData {
  return [Array.from(g.ticks), ...g.columns.map((c) => Array.from(c, gap))];
}

/** The Trade price chart (`[mean_log_price, sd_log_price]`): the mean, mean + SD and mean − SD. */
export function bandData(g: ChartGroup): LineData {
  const [mean, sd] = g.columns;
  const line = (f: (m: number, s: number) => number) => Array.from(mean, (m, i) => gap(f(m, sd[i])));
  return [Array.from(g.ticks), line((m) => m), line((m, s) => m + s), line((m, s) => m - s)];
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/ui/series-data.test.ts)`
Expected: `2 passed`.

- [ ] **Step 5: Rewrite the Charts panel**

Replace `web/src/ui/charts-panel.ts` with:
```ts
import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type { Engine } from '../engine';
import { chartsSignature } from '../goods';
import { groupSharesSignature } from '../groups';
import { CHART_POINTS, chartKey, type Wants, type WorldSnapshot } from '../protocol';
import { h } from './dom';
import { compactNumber } from './format';
import { bandData, lineData } from './series-data';

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
  {
    title: 'Births and deaths',
    lines: [
      { key: 'births', label: 'Births', color: '--c3' },
      { key: 'deaths', label: 'Deaths', color: '--c2' },
    ],
  },
];

const HEIGHT = 150;
/** The Lorenz curve, wealth histogram and supply & demand are fetched at most this often. */
const REFRESH_MS = 250;
const POLLUTANT_COLORS = ['--c1', '--c2', '--c3', '--c4'];
/** The Trade price chart's series: the mean and its ± SD band share one x axis. */
const PRICE_GROUP = ['mean_log_price', 'sd_log_price'];

export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private plots: {
    name: string;
    plot: uPlot;
    figure: HTMLElement;
    /** The host chart group a time chart draws (Decision 4). */
    group?: string[];
    update: (s: WorldSnapshot) => void;
    /** Hidden (and not redrawn) while false. */
    visible: () => boolean;
  }[] = [];
  private visible = false;
  /** When, and at which tick, the distributions last arrived; `distStale` asks for them regardless of the tick. */
  private distAt = -Infinity;
  private distTick = -1;
  private distStale = true;
  private color!: (v: string) => string;
  private axes!: uPlot.Axis[];
  private addTimeChart!: (chart: TimeChart, container?: HTMLElement, visible?: () => boolean) => void;
  private goodsSection = h('section', { class: 'goods' });
  private pollutionSection = h('section', { class: 'pollution' });
  /** Plots whose lines follow the goods and pollutants; rebuilt when `chartsSignature` changes. */
  private dynamic = new Set<uPlot>();
  /** Holds the Group shares chart, whose lines follow `culture.groups`. */
  private groupsSection = h('section', { class: 'group-shares' });
  private groupPlots = new Set<uPlot>();
  /** Captions that name the traded pair (0, 1). */
  private pairCaptions: { el: HTMLElement; title: string }[] = [];

  constructor(private engine: Engine) {
    this.build();
    engine.want((now) => this.wants(now));
    engine.on('snapshot', () => this.receive());
    // Edits, resets and config changes move the distributions without a tick.
    for (const event of ['edit', 'reset', 'config'] as const) engine.on(event, () => (this.distStale = true));
    new ResizeObserver(() => this.resize()).observe(this.el);
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) {
      this.resize();
      this.distStale = true;
      void this.engine.refresh();
    }
  }

  canvases(): { name: string; canvas: HTMLCanvasElement }[] {
    return this.plots.map((p) => ({ name: p.name, canvas: p.plot.ctx.canvas }));
  }

  private twoGoods(): boolean {
    return this.engine.config.goods.length >= 2;
  }

  /** The visible time charts' groups, and the distributions when due; nothing while the tab is hidden. */
  private wants(now: number): Wants {
    if (!this.visible) return {};
    const groups = this.plots.flatMap((p) => (p.group && p.visible() ? [p.group] : []));
    const w: Wants = { charts: { groups, max: CHART_POINTS } };
    if ((this.distStale || this.engine.tick !== this.distTick) && now - this.distAt >= REFRESH_MS) {
      w.lorenz = true;
      w.wealthHist = true;
      if (this.twoGoods()) w.supplyDemand = true;
    }
    return w;
  }

  /** Draws whatever the latest snapshot brought. */
  private receive(): void {
    const s = this.engine.last;
    if (!s) return;
    if (s.lorenz) {
      this.distTick = s.tick;
      this.distAt = performance.now();
      this.distStale = false;
    }
    for (const p of this.plots) if (p.visible()) p.update(s);
  }

  private width(): number {
    return Math.max(240, this.el.clientWidth - 4);
  }

  private resize(): void {
    if (!this.visible) return;
    this.plots.forEach((p) => p.plot.setSize({ width: this.width(), height: HEIGHT }));
  }

  private rebuildGoodsCharts(): void {
    this.plots = this.plots.filter((p) => {
      if (!this.dynamic.has(p.plot)) return true;
      p.plot.destroy();
      return false;
    });
    this.dynamic.clear();
    this.goodsSection.replaceChildren(h('h3', {}, 'Goods'));
    this.pollutionSection.replaceChildren(h('h3', {}, 'Pollution'));
    const c = this.engine.config;
    const perGood = (prefix: string) => c.goods.map((g, i) => ({ key: `${prefix}${i}`, label: g.name, color: g.color }));
    const before = this.plots.length;
    this.addTimeChart({ title: 'Mean holdings', lines: perGood('mean_holding_') }, this.goodsSection);
    this.addTimeChart({ title: 'Mean metabolism', lines: perGood('mean_metabolism_') }, this.goodsSection);
    this.addTimeChart({ title: 'Units traded', lines: perGood('traded_') }, this.goodsSection, () => this.engine.config.trade.enabled);
    this.addTimeChart(
      {
        title: 'Mean pollution',
        lines: c.pollution.pollutants.map((p, k) => ({ key: `mean_pollution_${k}`, label: p.name, color: POLLUTANT_COLORS[k] })),
      },
      this.pollutionSection,
      () => this.engine.config.pollution.enabled,
    );
    for (const p of this.plots.slice(before)) this.dynamic.add(p.plot);
    this.resize();
  }

  private rebuildGroupChart(): void {
    this.plots = this.plots.filter((p) => {
      if (!this.groupPlots.has(p.plot)) return true;
      p.plot.destroy();
      return false;
    });
    this.groupPlots.clear();
    this.groupsSection.replaceChildren();
    const before = this.plots.length;
    const lines = this.engine.config.culture.groups.map((g, k) => ({ key: `group_share_${k}`, label: g.name, color: g.color }));
    this.addTimeChart({ title: 'Group shares', lines, range: [0, 1] }, this.groupsSection);
    for (const p of this.plots.slice(before)) this.groupPlots.add(p.plot);
    this.resize();
  }

  private add(
    title: string,
    opts: Omit<uPlot.Options, 'width' | 'height'>,
    data: uPlot.AlignedData,
    update: (plot: uPlot, s: WorldSnapshot) => void,
    container: HTMLElement = this.el,
    visible: () => boolean = () => true,
    group?: string[],
  ): HTMLElement {
    const figcaption = h('figcaption', {}, title);
    const figure = h('figure', { class: 'chart' }, figcaption);
    const plot = new uPlot({ ...opts, width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ name: title, plot, figure, group, update: (s) => update(plot, s), visible });
    container.append(figure);
    return figcaption;
  }

  private build(): void {
    const css = getComputedStyle(document.documentElement);
    this.color = (v: string) => (v.startsWith('#') ? v : css.getPropertyValue(v).trim() || '#888');
    this.axes = [
      { stroke: this.color('--muted'), grid: { stroke: this.color('--grid') }, ticks: { stroke: this.color('--grid') } },
      {
        stroke: this.color('--muted'),
        grid: { stroke: this.color('--grid') },
        ticks: { stroke: this.color('--grid') },
        size: 44,
        values: (_self, splits) => splits.map(compactNumber),
      },
    ];

    this.addTimeChart = (chart: TimeChart, container?: HTMLElement, visible?: () => boolean) => {
      const group = chart.lines.map((l) => l.key);
      const key = chartKey(group);
      this.add(
        chart.title,
        {
          scales: { x: { time: false }, y: chart.range ? { range: chart.range } : {} },
          axes: this.axes,
          legend: { show: chart.lines.length > 1 },
          series: [{ label: 'Tick' }, ...chart.lines.map((l) => ({ label: l.label, stroke: this.color(l.color), width: 1.5 }))],
        },
        [[], ...chart.lines.map(() => [])],
        (plot, s) => {
          const g = s.charts?.[key];
          if (g) plot.setData(lineData(g));
        },
        container,
        visible,
        group,
      );
    };

    for (const chart of TIME_CHARTS) {
      this.addTimeChart(chart);
      // Group shares sit where the Blue share chart was.
      if (chart.title === 'Mean traits') this.el.append(this.groupsSection);
    }
    let groupLines = '';
    const syncGroupChart = () => {
      const next = groupSharesSignature(this.engine.config);
      if (next === groupLines) return;
      groupLines = next;
      this.rebuildGroupChart();
    };
    this.engine.on('reset', syncGroupChart);
    this.engine.on('config', syncGroupChart);
    syncGroupChart();

    const xs = Array.from({ length: 101 }, (_, i) => i / 100);
    this.add(
      'Lorenz curve',
      {
        scales: { x: { time: false, range: [0, 1] }, y: { range: [0, 1] } },
        axes: this.axes,
        legend: { show: false },
        series: [
          { label: 'Population share' },
          { label: 'Equality', stroke: this.color('--muted'), dash: [4, 4], width: 1 },
          { label: 'Wealth share', stroke: this.color('--c2'), width: 2 },
        ],
      },
      [xs, xs, xs],
      (plot, s) => {
        if (s.lorenz) plot.setData([xs, xs, s.lorenz]);
      },
    );

    const bars = uPlot.paths.bars!({ size: [0.9, 64] });
    this.add(
      'Wealth distribution',
      {
        scales: { x: { time: false } },
        axes: this.axes,
        legend: { show: false },
        series: [{ label: 'Sugar' }, { label: 'Agents', fill: this.color('--c1'), stroke: this.color('--c1'), paths: bars, points: { show: false } }],
      },
      [[], []],
      (plot, s) => {
        const hist = s.wealthHist;
        if (!hist) return;
        const width = hist[0];
        const counts = hist.slice(1);
        plot.setData([counts.map((_, i) => (i + 0.5) * width), counts]);
      },
    );

    // Market charts need two goods; loan charts need credit; the section needs
    // either. The disease section needs disease.
    const economy = h('section', { class: 'economy' }, h('h3', {}, 'Economy'));
    const disease = h('section', { class: 'disease' }, h('h3', {}, 'Disease'));
    this.el.append(this.goodsSection, this.pollutionSection, economy, disease);
    const twoGoods = () => this.twoGoods();
    const creditOn = () => this.engine.config.credit.enabled;
    const diseaseOn = () => this.engine.config.disease.enabled;
    const syncSection = () => {
      economy.hidden = !(twoGoods() || creditOn());
      disease.hidden = !diseaseOn();
      this.pollutionSection.hidden = !this.engine.config.pollution.enabled;
      const g = this.engine.config.goods;
      const pair = g.length >= 2 ? ` · ${g[0].name}/${g[1].name}` : '';
      for (const p of this.pairCaptions) p.el.textContent = p.title + pair;
      for (const p of this.plots) p.figure.hidden = !p.visible();
      this.distStale = true;
    };
    // Rebuilt only when the goods' or pollutants' lines change, not on every config event.
    let lines = '';
    const syncGoodsCharts = () => {
      const next = chartsSignature(this.engine.config);
      if (next === lines) return;
      lines = next;
      this.rebuildGoodsCharts();
    };
    this.engine.on('reset', syncGoodsCharts);
    this.engine.on('config', syncGoodsCharts);
    syncGoodsCharts();
    this.engine.on('reset', syncSection);
    this.engine.on('config', syncSection);

    const priceKey = chartKey(PRICE_GROUP);
    const priceCaption = this.add(
      'Trade price (ln)',
      {
        scales: { x: { time: false }, y: {} },
        axes: this.axes,
        legend: { show: true },
        series: [
          { label: 'Tick' },
          { label: 'Mean', stroke: this.color('--c2'), width: 1.5 },
          { label: '+SD', stroke: this.color('--muted'), width: 1, dash: [4, 4] },
          { label: '-SD', stroke: this.color('--muted'), width: 1, dash: [4, 4] },
        ],
      },
      [[], [], [], []],
      (plot, s) => {
        const g = s.charts?.[priceKey];
        if (g) plot.setData(bandData(g));
      },
      economy,
      twoGoods,
      PRICE_GROUP,
    );
    this.pairCaptions.push({ el: priceCaption, title: 'Trade price (ln)' });

    this.addTimeChart({ title: 'Trade volume', lines: [{ key: 'trade_volume', label: 'Volume', color: '--c1' }] }, economy, twoGoods);

    const sdCaption = this.add(
      'Supply & demand',
      {
        scales: { x: { time: false, distr: 3 }, y: {} },
        axes: this.axes,
        legend: { show: true },
        series: [
          { label: 'Price' },
          { label: 'Demand', stroke: this.color('--c1'), width: 1.5 },
          { label: 'Supply', stroke: this.color('--c2'), width: 1.5 },
          { label: 'Equilibrium', stroke: this.color('--c3'), points: { show: true, size: 9 }, paths: () => null },
          { label: 'Actual', stroke: this.color('--text'), points: { show: true, size: 9 }, paths: () => null },
        ],
      },
      [[], [], [], [], []],
      (plot, s) => {
        const sd = s.supplyDemand;
        if (!sd) return;
        const n = sd[0];
        const prices = Array.from(sd.subarray(1, 1 + n));
        const demand = Array.from(sd.subarray(1 + n, 1 + 2 * n));
        const supply = Array.from(sd.subarray(1 + 2 * n, 1 + 3 * n));
        const [eqP, eqQ, actP, actQ] = Array.from(sd.subarray(1 + 3 * n));
        const nearest = (p: number) =>
          prices.reduce((best, q, i) => (Math.abs(Math.log(q / p)) < Math.abs(Math.log(prices[best] / p)) ? i : best), 0);
        const point = (p: number, q: number) => {
          const col: (number | null)[] = prices.map(() => null);
          if (Number.isFinite(p) && Number.isFinite(q)) col[nearest(p)] = q;
          return col;
        };
        plot.setData([prices, demand, supply, point(eqP, eqQ), point(actP, actQ)]);
      },
      economy,
      twoGoods,
    );
    this.pairCaptions.push({ el: sdCaption, title: 'Supply & demand' });

    this.addTimeChart(
      {
        title: 'Loans',
        lines: [
          { key: 'loans_made', label: 'Loans made', color: '--c1' },
          { key: 'defaults', label: 'Defaults', color: '--c2' },
        ],
      },
      economy,
      creditOn,
    );

    this.addTimeChart({ title: 'Debt outstanding', lines: [{ key: 'debt_outstanding', label: 'Debt', color: '--c3' }] }, economy, creditOn);

    this.addTimeChart({ title: 'Foresight', lines: [{ key: 'mean_foresight', label: 'Foresight φ', color: '--c3' }] }, economy, () => this.engine.config.foresight.enabled);

    this.addTimeChart(
      { title: 'Infected', lines: [{ key: 'infected_fraction', label: 'Infected share', color: '--red' }], range: [0, 1] },
      disease,
      diseaseOn,
    );
    this.addTimeChart({ title: 'Diseases per agent', lines: [{ key: 'mean_diseases', label: 'Mean', color: '--c2' }] }, disease, diseaseOn);
    this.addTimeChart(
      { title: 'Diseases in circulation', lines: [{ key: 'diseases_in_circulation', label: 'Distinct diseases', color: '--c4' }] },
      disease,
      diseaseOn,
    );
    this.addTimeChart({ title: 'New infections', lines: [{ key: 'new_infections', label: 'Infections', color: '--c1' }] }, disease, diseaseOn);

    syncSection();
  }
}
```

- [ ] **Step 6: Remove the hatch**

In `web/src/main.ts`, delete the line `charts.maybeRefresh(now);` from the loop.

In `web/src/engine.ts`, delete the deprecated `get sim()` getter and change the wasm-pkg import to `import init, { presets_json } from './wasm-pkg/sugarscape.js';`.

In `web/src/sim-host.ts`, delete `currentSim()`.

- [ ] **Step 7: Check that nothing reads the simulation directly any more**

Run: `grep -rn "\.sim\.\|engine\.sim\b\|currentSim\|maybeRefresh\|trackSelection" web/src --include=*.ts`
Expected: no output.

- [ ] **Step 8: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 9: Commit**

```bash
git add web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/ui/charts-panel.ts web/src/main.ts web/src/engine.ts web/src/sim-host.ts
git commit -m "Draw charts from downsampled history with real ticks" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 11: The simulation worker

*Mechanical (full code).* Browser (controller): **every** existing puppeteer scenario (rules, tools, painting, image import, inspection, charts, share links, exports, disease, credit tab, groups, trails, Experiments) now runs through the worker; DevTools shows a `sim-worker` thread and the page's main thread idle between frames. Determinism: open `/?debug`, then in the page run `const e = window.sugarscape.engine; await e.loadPreset('ii-2-unit'); await e.reset(e.baseConfig, 1); await e.advance(200); await e.fingerprint()` → `'0x75b93943813545e4'`. Fallback: with `Worker` stubbed to throw before load (`window.Worker = function () { throw new Error('no'); }` via `evaluateOnNewDocument`) the app still works (on the page) and logs the warning.

**Files:**
- Create: `web/src/sim-worker.ts`
- Modify: `web/src/transport.ts`, `web/src/transport.test.ts`, `web/src/engine.ts`, `web/src/main.ts`

**Interfaces:**
- Consumes: `serve`, `SimHost` (Tasks 4–5), `wasmSimModule` (Task 6), `PortTransport`, `PortLike`.
- Produces: `transport.ts`: `startWorker(create?: () => PortLike): Promise<Transport>` (resolves once the worker answers `ready`; rejects if it errors first or its WASM fails to load). `engine.ts`: `defaultDeps` uses the worker, falling back to `InlineTransport` (Decision 10). `main.ts`: the `?debug` hook (Decision 14).

- [ ] **Step 1: Write the failing test**

Append to `web/src/transport.test.ts` (and add `startWorker` to its `./transport` import and `HostRequest` to its `./protocol` type import):
```ts
describe('startWorker', () => {
  it('resolves once the worker is ready and rejects when it cannot start', async () => {
    const ready: PortLike = {
      onmessage: null,
      onerror: null,
      postMessage: (m) => queueMicrotask(() => ready.onmessage?.({ data: { id: (m as HostRequest).id, result: { ok: true } } } as MessageEvent)),
      terminate: () => {},
    };
    await expect(startWorker(() => ready)).resolves.toBeInstanceOf(PortTransport);
    const broken: PortLike = {
      onmessage: null,
      onerror: null,
      postMessage: () => queueMicrotask(() => broken.onerror?.({ message: 'import failed' } as ErrorEvent)),
      terminate: () => {},
    };
    await expect(startWorker(() => broken)).rejects.toThrow('import failed');
    const noWasm: PortLike = {
      onmessage: null,
      onerror: null,
      postMessage: (m) =>
        queueMicrotask(() => noWasm.onmessage?.({ data: { id: (m as HostRequest).id, result: { ok: false, fatal: 'WASM failed to load' } } } as MessageEvent)),
      terminate: () => {},
    };
    await expect(startWorker(() => noWasm)).rejects.toThrow('WASM failed to load');
  });
});
```

- [ ] **Step 2: Run the test to see it fail**

Run: `(cd web && npx vitest run src/transport.test.ts)`
Expected: FAIL — `startWorker` is not exported.

- [ ] **Step 3: Implement**

Create `web/src/sim-worker.ts`:
```ts
// The playground's simulation worker: its own WASM instance and one SimHost, answering in order.
import type { HostMessage, HostRequest } from './protocol';
import { serve, SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import init from './wasm-pkg/sugarscape.js';

const post = (message: HostMessage, transfer: Transferable[]) => postMessage(message, { transfer });
/** Requests that arrive while the WASM loads wait here, in order. */
const queue: HostRequest[] = [];
let handle: ((req: HostRequest) => void) | null = null;

addEventListener('message', (event: MessageEvent<HostRequest>) => {
  if (handle) handle(event.data);
  else queue.push(event.data);
});

init().then(
  (wasm) => {
    handle = serve(new SimHost(wasmSimModule(wasm.memory)), post);
    for (const req of queue.splice(0)) handle(req);
  },
  (e: unknown) => {
    const fatal = `WASM failed to load: ${e instanceof Error ? e.message : String(e)}`;
    handle = (req) => post({ id: req.id, result: { ok: false, fatal } }, []);
    for (const req of queue.splice(0)) handle(req);
  },
);
```
Append to `web/src/transport.ts`:
```ts
/**
 * Starts the simulation worker and waits until its WASM is ready; rejects if a module worker
 * cannot start or its WASM fails to load (the engine then simulates on the page, Decision 10).
 */
export async function startWorker(
  create: () => PortLike = () => new Worker(new URL('./sim-worker.ts', import.meta.url), { type: 'module' }),
): Promise<Transport> {
  const transport = new PortTransport(create());
  const { result } = await transport.request({ type: 'ready' });
  if (!result.ok) {
    transport.close();
    throw new Error('fatal' in result ? result.fatal : 'the simulation worker did not start');
  }
  return transport;
}
```
In `web/src/engine.ts`, change the transport import to `import { InlineTransport, startWorker, type Transport } from './transport';` and replace `defaultDeps` with:
```ts
/**
 * The simulation runs in a worker; the page keeps its own WASM instance for presets, Experiments
 * and, if a module worker cannot start, the simulation itself (Decision 10).
 */
async function defaultDeps(): Promise<EngineDeps> {
  const wasm = await init();
  const presets = JSON.parse(presets_json()) as Preset[];
  let transport: Transport;
  try {
    transport = await startWorker();
  } catch (e) {
    console.warn('The simulation worker could not start; simulating on the page instead.', e);
    transport = new InlineTransport(new SimHost(wasmSimModule(wasm.memory)));
  }
  return { presets, transport };
}
```
In `web/src/main.ts`, right after the `try … catch` that creates `engine`, add:
```ts
  // Browser checks drive the engine through this handle (Decision 14).
  if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { sugarscape: { engine } });
```

- [ ] **Step 4: Run the test to see it pass**

Run: `(cd web && npx vitest run src/transport.test.ts)`
Expected: `4 passed`.

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds and `web/dist/assets` contains a `sim-worker-*.js` chunk (check with `ls web/dist/assets | grep sim-worker`); all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/sim-worker.ts web/src/transport.ts web/src/transport.test.ts web/src/engine.ts web/src/main.ts
git commit -m "Run the simulation in a Web Worker" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 12: Max speed in the host

*Mechanical (full code).* Browser: nothing new to check (the engine does not send `run` until Task 13).

**Files:**
- Modify: `web/src/protocol.ts`, `web/src/sim-host.ts`, `web/src/sim-host.test.ts`, `web/src/transport.ts`, `web/src/sim-worker.ts`

**Interfaces:**
- Consumes: Tasks 3–5 and 11.
- Produces:
  - `protocol.ts`: `Command` gains `{ type: 'run' } | { type: 'stop' } | { type: 'frame' }`
  - `sim-host.ts`: `const BATCH_MS = 16`, `const POST_MS = 33`; `SimHost.running: boolean` (getter); `SimHost.batch(): WorldSnapshot | null`; `serve(host, send, defer: (fn: () => void) => void)` (new third parameter); `channelDefer(): (fn: () => void) => void`
  - `transport.ts`: the page host schedules batches with `setTimeout(fn, 0)`; `sim-worker.ts` with `channelDefer()`

Host rules (Decision 11): `run` starts the loop with the request's wants and pools its buffer (no snapshot in the reply); every later request's wants replace the loop's; `frame` pools its buffer (returned in `spare` if the loop is not running); `batch()` steps one tick at a time until `BATCH_MS` have passed, then returns a snapshot only if `POST_MS` have passed since the last post and a pooled buffer is free (a scheduled change that fired in a batch without a post is reported by the next snapshot); `stop` ends the loop and replies with a snapshot in a pooled buffer (if any), returning the other pooled buffers in `spare`. A panic in a batch makes `serve` post `{ id: null, fatal }`.

- [ ] **Step 1: Write the failing tests**

Append to `web/src/sim-host.test.ts`:
```ts
describe('SimHost at Max speed', () => {
  it('steps in batches and posts about every 33 ms while it holds a free buffer', () => {
    let clock = 0;
    const t = start(() => clock++); // each reading of the clock is 1 ms later
    const a = new ArrayBuffer(48);
    const b = new ArrayBuffer(48);
    const run = t.send({ type: 'run' }, { frame: a });
    expect(run.result).toEqual({ ok: true });
    expect(run.spare).toBeUndefined();
    expect(t.host.running).toBe(true);
    const posts: WorldSnapshot[] = [];
    for (let i = 0; i < 6; i++) {
      const post = t.host.batch();
      if (post) posts.push(post);
    }
    expect(posts).toHaveLength(1);
    expect(posts[0].frame).toBe(a);
    expect(posts[0].tick).toBeGreaterThan(16);
    t.send({ type: 'frame' }, { frame: b });
    expect(t.host.batch()?.frame).toBe(b);
  });

  it('handles commands between batches and answers stop with the last frame and the unused buffers', () => {
    let clock = 0;
    const t = start(() => clock++);
    const a = new ArrayBuffer(48);
    const b = new ArrayBuffer(48);
    t.send({ type: 'run' }, { frame: a });
    t.send({ type: 'frame' }, { frame: b });
    expect(t.host.batch()).toBeNull();
    expect(t.snap(t.send({ type: 'place', x: 0, y: 2, overrides: {} })).population).toBe(2);
    const stop = t.send({ type: 'stop' });
    expect(t.host.running).toBe(false);
    expect(t.snap(stop).frame).toBe(b);
    expect(stop.spare).toHaveLength(1);
    expect(stop.spare?.[0]).toBe(a);
    expect(t.host.batch()).toBeNull();
    const late = new ArrayBuffer(48);
    expect(t.send({ type: 'frame' }, { frame: late }).spare?.[0]).toBe(late);
  });

  it('reports a scheduled change in the next post', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'setConfig', config: { ...config, schedule: [{ tick: 3, set: {} }] } });
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch();
    expect(post.config?.schedule).toHaveLength(1);
  });

  it('serve keeps posting between requests until stop', async () => {
    let clock = 0;
    const messages: HostMessage[] = [];
    const handle = serve(new SimHost(fakeModule(), () => clock++), (m) => messages.push(m), (fn) => setTimeout(fn, 0));
    const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
    const posts = () => messages.filter((m) => m.id === null).length;
    handle({ id: 1, cmd: { type: 'init', config, seed: 1, landscapes: [], display } });
    handle({ id: 2, cmd: { type: 'run' }, frame: new ArrayBuffer(48) });
    await wait(20);
    expect(posts()).toBe(1); // one buffer: one post until it comes back
    handle({ id: 3, cmd: { type: 'frame' }, frame: new ArrayBuffer(48) });
    await wait(20);
    expect(posts()).toBe(2);
    handle({ id: 4, cmd: { type: 'stop' } });
    const count = messages.length;
    await wait(20);
    expect(messages.length).toBe(count);
  });
});
```
In the existing `serve` test, change `serve(new SimHost(fakeModule()), (m, t) => sent.push([m, t]))` to `serve(new SimHost(fakeModule()), (m, t) => sent.push([m, t]), () => {})`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/sim-host.test.ts)`
Expected: FAIL — type errors / `'run'` not a command, `host.batch is not a function`.

- [ ] **Step 3: Implement**

`web/src/protocol.ts` — in the `Command` union, replace `  | { type: 'fingerprint' };` with:
```ts
  | { type: 'fingerprint' }
  | { type: 'run' }
  | { type: 'stop' }
  | { type: 'frame' };
```

`web/src/sim-host.ts`:
- below `CHART_GROWTH` add:
```ts
/** Max speed: a batch steps for about this long… */
export const BATCH_MS = 16;
/** …and a snapshot is posted about this often (while the host holds a free buffer). */
export const POST_MS = 33;
```
- add a field to `SimHost`:
```ts
  /** Max speed: the wants to answer with, the buffers to post frames in, when it last posted; null when not running. */
  private max: { wants: Wants; pool: ArrayBuffer[]; posted: number } | null = null;
```
- replace `handle` and `fail` with:
```ts
  get running(): boolean {
    return this.max !== null;
  }

  handle(req: HostRequest): HostReply {
    if (this.max && req.wants) this.max.wants = req.wants;
    const spare: ArrayBuffer[] = [];
    let result: Result;
    if (this.dead) {
      result = { ok: false, fatal: this.dead };
    } else {
      try {
        result = this.apply(req, spare);
      } catch (e) {
        // The core throws field errors as JSON strings; anything else is a panic or a bug.
        result = typeof e === 'string' ? { ok: false, errors: parseErrors(e) } : { ok: false, fatal: this.fail(e) };
      }
    }
    // A lent buffer the snapshot did not use and the Max loop did not keep goes straight back.
    const frame = req.frame;
    const used = result.ok && result.snapshot?.frame === frame;
    if (frame && !used && !this.max?.pool.includes(frame)) spare.push(frame);
    return spare.length > 0 ? { id: req.id, result, spare } : { id: req.id, result };
  }

  /** Stops the host for good; returns the message every later command gets. */
  fail(e: unknown): string {
    this.max = null;
    this.dead = `The simulation stopped: ${e instanceof Error ? e.message : String(e)}`;
    return this.dead;
  }

  /** One Max-speed batch (Decision 11): a snapshot to post, or null. */
  batch(): WorldSnapshot | null {
    const max = this.max;
    const sim = this.sim;
    if (!max || !sim || this.dead) return null;
    const start = this.now();
    const from = sim.tick();
    do sim.step(1);
    while (this.now() - start < BATCH_MS);
    this.fired(from, sim.tick());
    const now = this.now();
    const frame = now - max.posted >= POST_MS ? max.pool.pop() : undefined;
    if (!frame) return null;
    max.posted = now;
    return this.snapshot(sim, max.wants, frame);
  }
```
- change `apply`'s signature to `private apply(req: HostRequest, spare: ArrayBuffer[]): Result {` and add these cases to its `switch` (after `case 'fingerprint'`):
```ts
      case 'run':
        this.max = { wants, pool: frame ? [frame] : [], posted: this.now() };
        return { ok: true };
      case 'frame':
        if (this.max && frame) this.max.pool.push(frame);
        return { ok: true };
      case 'stop': {
        const max = this.max;
        this.max = null;
        if (!max) return this.reply(sim, wants);
        const last = max.pool.pop();
        spare.push(...max.pool);
        return this.reply(sim, max.wants, last);
      }
```
- replace `serve` with:
```ts
/**
 * Wires a host to a message channel: each request is answered in order, its buffers transferred;
 * while Max runs, batches are scheduled with `defer` so queued requests are handled between them.
 */
export function serve(
  host: SimHost,
  send: (message: HostMessage, transfer: Transferable[]) => void,
  defer: (fn: () => void) => void,
): (req: HostRequest) => void {
  let looping = false;
  const loop = (): void => {
    if (!host.running) {
      looping = false;
      return;
    }
    let message: HostMessage | null = null;
    try {
      const post = host.batch();
      if (post) message = { id: null, post };
    } catch (e) {
      message = { id: null, fatal: host.fail(e) };
    }
    if (message) send(message, transfers(message));
    defer(loop);
  };
  return (req) => {
    const reply = host.handle(req);
    send(reply, transfers(reply));
    if (host.running && !looping) {
      looping = true;
      defer(loop);
    }
  };
}

/** Runs `fn` as a new task without `setTimeout`'s 4 ms clamp; messages already queued run first or between. */
export function channelDefer(): (fn: () => void) => void {
  const channel = new MessageChannel();
  const queue: (() => void)[] = [];
  channel.port1.onmessage = () => queue.shift()?.();
  return (fn) => {
    queue.push(fn);
    channel.port2.postMessage(null);
  };
}
```

`web/src/transport.ts` — in `inlinePort`, pass the page scheduler as `serve`'s third argument:
```ts
  const handle = serve(
    host,
    (message) => queueMicrotask(() => port.onmessage?.({ data: message } as MessageEvent)),
    (fn) => setTimeout(fn, 0),
  );
```

`web/src/sim-worker.ts` — import `channelDefer` alongside `serve` and change the `serve` call to `serve(new SimHost(wasmSimModule(wasm.memory)), post, channelDefer())`.

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/sim-host.test.ts src/transport.test.ts)`
Expected: `Test Files 2 passed` (sim-host: 17 tests).

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/protocol.ts web/src/sim-host.ts web/src/sim-host.test.ts web/src/transport.ts web/src/sim-worker.ts
git commit -m "Run the simulation flat out in the host at Max speed" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 13: Max speed in the engine and the speed menu

*Needs judgment (full code given; the quiesce ordering is the point).* Browser (controller): choose **Max** on `ii-2-unit` and on a 200 × 200 world with 2 000 agents (Setup section) — the grid redraws smoothly, the readout climbs fast, the page stays responsive (open tabs, hover charts); Pause stops at once; Reset, 🎲, a preset change and a reset-requiring rule change while at Max rebuild and carry on at Max; a live rule change and painting apply while at Max; switching from Max to 5× and back; switching to Experiments pauses; a Charts/Credit/Inspect tab keeps updating at Max.

**Files:**
- Modify: `web/src/engine.ts`, `web/src/engine.test.ts`, `web/src/ui/toolbar.ts`

**Interfaces:**
- Consumes: `run`/`stop`/`frame` (Task 12), `Transport.onPost`.
- Produces: `engine.ts`: `export type Speed = number | 'max'`; `Engine.speed: Speed` (replaces `stepsPerFrame`), `Engine.setSpeed(speed: Speed): void`; `advance(n = 1)`.

Engine rules (Decisions 8, 11): Max runs while `running && speed === 'max'` and no quiet-world write holds the loop; each post is adopted (`'config'` if a scheduled change fired, `'tick'`, `'snapshot'`) and its displaced buffer goes straight back with a `frame` command; `quiet()` stops Max and waits for the acknowledgement before the write, and restarts it afterwards; a crash ends it.

- [ ] **Step 1: Write the failing tests**

Append to `web/src/engine.test.ts`:
```ts
describe('Engine at Max speed', () => {
  const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

  /** A host on a fake clock (1 ms per reading) and a log of the commands the engine sends. */
  async function maxSetup() {
    let clock = 0;
    const transport = new InlineTransport(new SimHost(fakeModule(), () => clock++));
    const sent: string[] = [];
    const request = transport.request.bind(transport);
    transport.request = (cmd, extra) => {
      sent.push(cmd.type);
      return request(cmd, extra);
    };
    const engine = await Engine.create({ config, seed: 7 }, { presets, transport });
    engine.setSpeed('max');
    return { engine, sent };
  }

  it('runs until paused, handing each posted buffer back', async () => {
    const { engine, sent } = await maxSetup();
    let ticks = 0;
    engine.on('tick', () => ticks++);
    engine.setRunning(true);
    await wait(30);
    for (let i = 0; i < 3; i++) engine.pump();
    expect(ticks).toBeGreaterThan(1);
    expect(sent.filter((c) => c === 'frame').length).toBeGreaterThanOrEqual(ticks - 1);
    engine.setRunning(false);
    await wait(5);
    const at = engine.tick;
    await wait(20);
    expect(engine.tick).toBe(at);
    expect(sent.filter((c) => c === 'run')).toHaveLength(1);
    expect(sent).toContain('stop');
    expect(sent).not.toContain('step');
  });

  it('stops before a reset and starts again after it', async () => {
    const { engine, sent } = await maxSetup();
    engine.setRunning(true);
    await wait(10);
    expect(await engine.reset()).toBeNull();
    const at = sent.lastIndexOf('reset');
    expect(sent.lastIndexOf('stop', at)).toBeGreaterThan(sent.indexOf('run'));
    expect(sent.indexOf('run', at)).toBeGreaterThan(at);
    const t = engine.tick;
    await wait(20);
    expect(engine.tick).toBeGreaterThan(t);
    engine.setRunning(false);
  });

  it('applies edits while running', async () => {
    const { engine } = await maxSetup();
    engine.setRunning(true);
    await wait(5);
    const edits: number[] = [];
    engine.on('edit', () => edits.push(engine.population));
    expect(await engine.place(0, 2, {})).toBeNull();
    expect(edits).toEqual([2]);
    const t = engine.tick;
    await wait(20);
    expect(engine.tick).toBeGreaterThan(t);
    engine.setRunning(false);
  });

  it('leaves Max for a step speed', async () => {
    const { engine, sent } = await maxSetup();
    engine.setRunning(true);
    await wait(5);
    engine.setSpeed(5);
    await wait(5);
    expect(sent).toContain('stop');
    const at = engine.tick;
    engine.pump();
    await wait(5);
    expect(engine.tick).toBe(at + 5);
    engine.setRunning(false);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: FAIL — `engine.setSpeed is not a function` (and a type error for `'max'`).

- [ ] **Step 3: Implement**

In `web/src/engine.ts`:
- below `export type WantsProvider …` add:
```ts
/** Ticks per animation frame, or 'max': the host steps flat out and posts about 30 snapshots a second. */
export type Speed = number | 'max';
```
- replace the field `stepsPerFrame = 1;` with `speed: Speed = 1;`, and add private fields next to `holds`:
```ts
  /** Whether the host's Max loop should be running (a `run` was sent and no `stop` since). */
  private maxOn = false;
  /** The pending `stop`, while one is. */
  private stopping: Promise<void> | null = null;
```
- in the constructor, add after the `onFatal` line:
```ts
    transport.onPost = (s) => this.onPost(s);
```
- replace `pump`, `setRunning` and `advance` with:
```ts
  /** Called every animation frame: asks for the next frame's steps, or (paused) for extras a panel still wants. */
  pump(now: number = performance.now()): void {
    if (this.crashed || this.inFlight || this.holds > 0) return;
    let next: Promise<void> | null = null;
    if (this.running) {
      // At Max the host runs its own loop (Decision 11).
      if (this.speed !== 'max') next = this.stepNow(this.speed);
    } else if (now - this.lastRefresh >= REFRESH_MS && this.providersWant(now)) {
      next = this.refresh();
    }
    if (next) this.inFlight = next.finally(() => (this.inFlight = null));
  }
```
```ts
  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
    this.syncMax();
  }

  setSpeed(speed: Speed): void {
    this.speed = speed;
    this.syncMax();
  }

  advance(n = 1): Promise<void> {
    return this.stepNow(n);
  }
```
- replace `quiet` with:
```ts
  /**
   * Runs `fn` in a quiet world (Decision 8): the frame loop is held, Max is stopped (and its
   * acknowledgement awaited) and the loop's step has settled; Max restarts afterwards if wanted.
   */
  private async quiet<T>(fn: () => Promise<T>): Promise<T> {
    this.holds++;
    try {
      await this.stopMax();
      await this.inFlight;
      return await fn();
    } finally {
      this.holds--;
      this.syncMax();
    }
  }

  /** Starts or stops the host's Max loop to match `running`, `speed`, holds and crashes. */
  private syncMax(): void {
    const want = this.running && this.speed === 'max' && this.holds === 0 && !this.crashed;
    if (want && !this.maxOn) {
      this.maxOn = true;
      void this.send({ type: 'run' }, true);
    } else if (!want && this.maxOn) {
      void this.stopMax();
    }
  }

  /** Stops the Max loop; resolves once the stop is acknowledged (so every earlier post has been handled). */
  private stopMax(): Promise<void> {
    if (!this.maxOn) return this.stopping ?? Promise.resolve();
    this.maxOn = false;
    this.stopping = this.send({ type: 'stop' }).then((result) => {
      this.stopping = null;
      if (!result.ok || !result.snapshot) return;
      const events: EngineEvent[] = result.snapshot.config ? ['config', 'tick'] : ['tick'];
      this.accept(result.snapshot, events);
    });
    return this.stopping;
  }

  /** A Max-speed snapshot: adopt it, then hand the displaced buffer straight back with fresh wants. */
  private onPost(s: WorldSnapshot): void {
    const events: EngineEvent[] = s.config ? ['config', 'tick'] : ['tick'];
    this.accept(s, events);
    if (this.maxOn) void this.send({ type: 'frame' }, true);
  }
```
- in `crash`, add `this.maxOn = false;` after `this.crashed = message;`.

In `web/src/ui/toolbar.ts`, change the import to `import { randomSeed, type Engine, type Speed } from '../engine';`, `SPEEDS` to
```ts
const SPEEDS: Speed[] = [1, 2, 5, 10, 25, 100, 'max'];
```
and the speed select to:
```ts
  const speed = h(
    'select',
    {
      title: 'Ticks per frame; Max runs the simulation as fast as it goes and redraws about 30 times a second',
      onchange: () => engine.setSpeed(speed.value === 'max' ? 'max' : Number(speed.value)),
    },
    ...SPEEDS.map((s) => h('option', { value: String(s) }, s === 'max' ? 'Max' : `${s}×`)),
  );
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: `17 passed`.

- [ ] **Step 5: Check**

Run: `grep -rn "stepsPerFrame" web/src --include=*.ts`
Expected: no output.

- [ ] **Step 6: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 7: Commit**

```bash
git add web/src/engine.ts web/src/engine.test.ts web/src/ui/toolbar.ts
git commit -m "Add the Max speed" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 14: README, roadmap and full verification

*Mechanical.*

**Files:**
- Modify: `README.md`, `docs/roadmap.md`

- [ ] **Step 1: README**

In `README.md`, insert before `## Command line`:
```markdown
## How the playground runs

The simulation runs in a Web Worker: the page sends it commands (steps, edits, rule changes)
and draws the frame and statistics it sends back, so the page stays responsive on large grids
and at high speeds. Where a module worker cannot start, the same code runs on the page. The
speed menu's **Max** runs the simulation as fast as it goes and redraws about 30 times a
second; the other speeds step a fixed number of ticks per frame, as before. A run does not
depend on the speed: the same setup and seed give the same world at the same tick.

Charts draw a downsampled history, with the tick on the x axis: Largest-Triangle-Three-Buckets
keeps about 2 000 points of each line, so spikes and gaps survive on long runs. The full per-tick history stays with the
simulation: Export → Statistics (CSV), share links and Experiments use all of it.
```

- [ ] **Step 2: Roadmap**

In `docs/roadmap.md`, insert after the Milestone 6 section:
```markdown
## Milestone 7a: Worker simulation (done)

The simulation runs in a Web Worker behind a command/snapshot protocol (with an on-page
fallback), a Max speed runs it flat out, and charts draw downsampled history (LTTB, about
2 000 points per line) while CSV exports keep every tick. Runs are unchanged. See
`docs/superpowers/specs/2026-09-24-worker-simulation-design.md`.
```
In "Playground and infrastructure", replace the "Web Worker simulation" and "Downsampled chart history" bullets with
```markdown
- **Web Worker simulation**: done (Milestone 7a).
- **Downsampled chart history**: done (Milestone 7a).
```
and leave the other bullets (replayable edit log, side-by-side comparison, recording — milestone 7b) as they are.

- [ ] **Step 3: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
git diff main -- crates/sugarscape-core/tests/
grep -rn "engine\.sim\b\|\.sim\.\|currentSim\|stepsPerFrame\|maybeRefresh\|trackSelection" web/src --include=*.ts
```
All must pass; the last two commands must print nothing.

The controller then runs the full puppeteer pass through the worker (implementers don't): every existing scenario (rules, tools, painting, image import, inspection, charts, share links, exports, disease, credit tab, groups, trails, Experiments); the determinism check of Task 11 (`/?debug` → `0x75b93943813545e4`); a Max determinism check (from a fresh `ii-2-unit` seed-1 reset, run at Max, pause, read `e.tick` = T and `await e.fingerprint()` = F; reset with seed 1 again and `await e.advance(T)` → F again); and the performance check: a 200 × 200 world with 2 000 agents at **Max** for 20 s with a `PerformanceObserver({ type: 'longtask', buffered: true })` recording — no main-thread task over 50 ms — and a `requestAnimationFrame` counter of grid redraws (`'snapshot'` events) near 30 per second.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md
git commit -m "Document the worker simulation, Max speed and chart downsampling" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

## Spec coverage

| Spec requirement | Task |
|---|---|
| No simulation change; golden and legacy unedited and green | Global Constraints; every task's verification; 7 (golden fingerprint through the engine with the real WASM, however ticks are split); 11/14 (through the worker, controller); 14 (`git diff main -- crates/sugarscape-core/tests/`) |
| Same behavior; step-per-frame speeds keep meaning and determinism | 6 (engine API kept; `advance(n)` = `step(n)`), 7, 13 (`pump` steps `speed` ticks per frame) |
| No COOP/COEP, no SharedArrayBuffer | 5, 11 (transferable `ArrayBuffer`s only) |
| `sim-host.ts`: holds a `Sim`, one message at a time, `{ ok, snapshot? } \| { ok: false, errors }` | 4 (plus `fatal`, Decision 7), 5 (`serve`), 12 (Max) |
| `sim-worker.ts`: module worker loading WASM | 11 (Vite `worker.format: 'es'` already set) |
| `transport.ts`: `WorkerTransport` / `InlineTransport` (fallback), ids | 5 (`PortTransport`, `InlineTransport`), 11 (`startWorker` = the worker transport, fallback in `defaultDeps`) |
| Engine: sync reads from the snapshot, async writes, events on replies, `engine.sim` removed | 6 (Migration map), 8, 9, 10 (hatch removed, grep) |
| Protocol commands | 3, 4 (all but Max), 12 (`run`, `stop`); additions `ready`, `refresh`, `frame` (Decision 1) |
| Snapshot always-fields and `wants` extras | 3, 4 (Decision 2) |
| Frame buffer ping-pong, resize | 4 (host), 6 (engine test "ping-pongs two frame buffers"), 12 (Max pool) |
| Speeds: one outstanding `step` per frame, skip otherwise | 6 (`pump`; test "keeps at most one step outstanding") |
| Max: ~16 ms batches, ~33 ms posts, commands between batches, `stop` ends it, engine waits for the acknowledgement | 12 (host, fake-clock tests, `serve` test), 13 (engine tests: pause, reset, edits, leaving Max) |
| Commands queue while running | 6 (test "holds the frame loop…"), 12 (test "handles commands between batches"), 13 (test "applies edits while running") |
| Worker errors / panics → crash banner with Reload | 4, 5 (fatal replies, port errors), 6 (`'crash'` → banner), 11 (worker `error` event; WASM load failure) |
| Core `stats::downsample` (LTTB, endpoints, NaN, `max < 3`, short series) | 1 |
| WASM `Sim.series_downsampled` | 2 (plus `series_group`, `fingerprint`) |
| Worker chart throttle (250 ms or > 1 %, always after reset/setConfig), `max: 2000` | 4 (Decision 4) |
| Charts: x = returned ticks, full history stays in the worker, tooltips on displayed points | 10 (uPlot's cursor reads the displayed data) |
| Tests: core, WASM, Vitest host / engine | 1, 2, 4, 5, 6, 8, 9, 12, 13 |
| Determinism (browser) | 7 (Vitest, real WASM), 11 and 14 (controller via `?debug`) |
| Browser scenarios and performance check | each web task's list; 14 (full pass, long tasks, ~30 fps) |
| Docs: README and roadmap | 14 |

## Spec gaps and conflicts found

1. **Per-series downsampling cannot align multi-line charts.** uPlot needs one x array per chart, and the Trade price chart computes mean ± SD point by point; LTTB picks different ticks per series. The host sends one downsampled group per chart (`Sim.series_group` → `stats::downsample_union`: the union of each line's LTTB points, every line's value at each), while `series_downsampled` is still exposed and tested exactly as specified (Decision 4).
2. **`modified` in the snapshot** depends on which preset the user chose, which only the engine knows; it stays `engine.isModified()` on the main thread (Decision 2).
3. **`config` and `editedLandscapes` "always"** would mean several kilobytes of JSON and a map per good in every frame; they are sent only when they may have changed, and the engine keeps the last ones (Decision 2).
4. **Missing commands:** a paused world never sends `step`, so opening a panel, selecting, or reacting to a config change while paused needs `refresh`; the worker start-up needs `ready`; Max needs `frame` to return buffers. `exportConfig`/`exportLandscape` are unnecessary with snapshot `config`/`editedLandscapes` and are not added (Decision 1).
5. **The toolbar chip needs `followed`/`followedAlive`**, which the spec's snapshot list lacks; both are always sent (Decision 2).
6. **`applyConfig` also needs a quiet world:** it builds the next config from `engine.config`, so a step in flight could fire a scheduled change the new config would undo. The spec names only reset and preset changes (Decision 8).
7. **"NaN values are dropped from the candidate set but leave the surrounding points"** is read as: never chosen, but a bucket with nothing else keeps a NaN point so the gap stays; gaps narrower than a bucket close at that zoom (Decision 5).
8. **`downsample` returns `(u32, f64)` "ticks"** from a plain value slice: the `u32` is the index, which equals the tick because the history holds one snapshot per tick from 0; WASM maps indices through the `tick` series regardless (Decision 5).
9. **"Tooltips show the nearest displayed point"** needs no code: uPlot's cursor and legend already read the displayed (downsampled) data.
10. **The page still loads WASM** for presets, Experiments and the fallback, so the app holds two WASM instances (page and worker), as it already does while a sweep runs (Decision 10).
