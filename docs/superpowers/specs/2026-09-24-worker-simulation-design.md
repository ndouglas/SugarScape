# SugarScape Milestone 7a — Worker simulation and downsampled charts — Design

**Date:** 2026-09-24
**Builds on:** the milestone 1–6 specs in `docs/superpowers/specs/`; all remain binding where not changed here.
**Follow-up:** milestone 7b (side-by-side comparison, recording, replayable edit log) builds on this engine.

## Goal

Move the playground's simulation off the main thread into a Web Worker, and keep charts fast on long runs by downsampling their history, without changing what the simulation computes or what the user can do.

## Non-negotiable constraints

- **No simulation change.** Golden and legacy tests stay unedited and green; the worker runs the same `Sim`.
- **Same behaviour.** Every existing feature (rules, tools, painting, inspection, charts, share links, exports, disease, credit, groups, trails, Experiments) works as before; step-per-frame speeds keep their meaning and determinism.
- **Deploys as today:** no COOP/COEP headers, no SharedArrayBuffer.

## Architecture

- **`web/src/sim-host.ts`:** a message handler written once. It holds a `Sim` (or none before `init`), handles one message at a time in arrival order, and returns `{ ok: true, snapshot? } | { ok: false, errors: FieldError[] }` (errors in the existing `[{field, message}]` shape).
- **`web/src/sim-worker.ts`:** a module worker that loads the WASM and runs `sim-host`.
- **`web/src/transport.ts`:** `Transport` with `WorkerTransport` (used by the app) and `InlineTransport` (runs `sim-host` on the main thread — used by tests, and as the fallback when a module worker cannot start). Requests carry an id; replies resolve the matching promise.
- **`web/src/engine.ts`:** keeps its public shape where possible. Reads are synchronous and served from the latest snapshot; writes return promises; events fire when replies arrive. `engine.sim` is removed; its ten UI call sites move to snapshot fields or async engine methods.

## Protocol

Commands: `init { config, seed, landscapes }`, `reset { config, seed, landscapes }`, `setConfig { config }`, `step { n, wants }`, `run { wants }` (Max speed), `stop`, `setDisplay { colorMode, layer, overlays }`, edits (`paint`, `importLandscape`, `place`, `erase`, `infect`, `vaccinate`), `follow { id | null }`, `inspect { x, y }`, exports (`seriesCsv`, `agentsCsv`, `exportConfig`, `exportLandscape { good }`), and `fingerprint` (tests).

**Snapshot** (with every `step` reply, every `run` post, and after any command that changes the world):
- always: `frame` (RGBA `ArrayBuffer`, transferred), `width`, `height`, `tick`, `population`, `latest` stats (`Snapshot`), `config` (normalized), `modified`, `editedLandscapes`;
- on request via `wants`: `inspection` (for the selected site), `trail`, `charts` (downsampled series, below), `lorenz`, `wealthHist`, `supplyDemand` (while Charts is visible), `networks` (per enabled overlay), `creditGraph` (while the Credit tab is visible), `diseaseList` (while a disease tool is open).

**Frame buffers:** the engine owns two `ArrayBuffer`s and sends one with each `step`/`run` request; the worker renders into it and transfers it back. The engine draws from the returned buffer and passes it back with the next request (ping-pong), so neither side allocates per frame. If the grid size changes, new buffers of the right size are made.

## Running

- **Speeds 1×…N×:** on each `requestAnimationFrame`, if no `step` is outstanding and the world is running, the engine sends `step { n: stepsPerFrame, wants }`; otherwise it skips the frame. The main thread never waits synchronously.
- **Max:** a new speed option. The engine sends `run { wants }`; the worker steps in batches of about 16 ms of work and posts a snapshot about every 33 ms, checking for queued commands between batches. `stop` (Pause, reset, preset change, leaving Max) ends it; the engine waits for the stop acknowledgement before sending anything that depends on a quiet world.
- **Commands while running:** queue behind the current step or batch.
- **Worker errors:** an uncaught worker error or a panic surfaces through the existing "simulation crashed" banner with Reload.

## Downsampled chart history

- **Core:** `stats::downsample(values: &[f64], max: usize) -> Vec<(u32, f64)>` — Largest-Triangle-Three-Buckets over (tick, value) points; the first and last points are always kept; `values.len() <= max` returns every point; NaN values are dropped from the candidate set but leave the surrounding points (so gaps remain gaps in the chart); `max < 3` returns the endpoints.
- **WASM:** `Sim.series_downsampled(name: &str, max: u32) -> Float64Array` — alternating tick, value.
- **Worker:** `wants.charts = { names, max: 2000 }`; the worker recomputes a series only when at least 250 ms have passed since its last send **or** the history grew by more than 1 %, and always after `reset`/`setConfig`.
- **Charts panel:** x values are the returned ticks (not array positions); the full history stays in the worker (CSV export, share links and Experiments unchanged). Tooltips show the nearest displayed point.

## Testing

- **Core:** LTTB keeps both endpoints, keeps a single sharp spike, returns at most `max` points, returns short series unchanged, handles NaN gaps; golden and legacy unchanged.
- **WASM:** `series_downsampled` length and ticks.
- **Vitest:** `sim-host` with a fake `Sim` — snapshot fields only as requested by `wants`; chart data only when new; errors in the existing shape. Engine over `InlineTransport` — at most one outstanding `step`; commands queue while running; writes resolve with errors; events fire on replies; frame buffers ping-pong.
- **Determinism (browser):** `ii-2-unit`, seed 1, 200 ticks through the worker → `fingerprint` equals the golden entry.
- **Browser (controller):** every existing puppeteer scenario passes (rules, tools, painting, inspection, charts, share links, disease, credit tab, groups, trails, Experiments); a performance check at Max speed on a 200 × 200 grid with 2 000 agents: no main-thread task longer than 50 ms (PerformanceObserver `longtask`) and a display rate near 30 frames per second.

## Docs

README: the worker architecture (one paragraph), the Max speed, and chart downsampling (full data in CSV). Roadmap: mark "Web Worker simulation" and "Downsampled chart history" done; keep 7b's items.
