# Sessions, Comparison and Recording Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On the 7a worker engine, add a replayable edit log (share links and session files reproduce a whole session exactly), a side-by-side Compare mode (two worlds stepped in lockstep, charts overlaid), and recording the grid as WebM or GIF.

**Architecture:** The `SimHost` (web/src/sim-host.ts) records every world-changing command that succeeds as `{ tick, cmd }` and replays a session's log inside `init`/`reset`, `step n` and Max batches, reporting `replayLeft`; the `Engine` keeps the session's origin (config, seed, starting maps), answers `session()`, and its Reset replays. Share links gain wire v3 (`e` = compact log), session files, and `#c=` compare links (web/src/share.ts, web/src/sessions.ts). Compare runs a second `Engine` (its own worker) beside the page's engine; a `Lockstep` coordinator (web/src/compare/lockstep.ts) sends `advance n` to both and waits for both; per-world panels are separate instances behind `WorldSlot`s; `ChartsPanel` draws a list of worlds on shared axes; "Keep B" moves B's transport and state into the page's engine (`Engine.takeWorld`). Recording (web/src/recording/) composes the drawn grid canvases into a fixed-size frame; WebM goes through `captureStream(0)` + `MediaRecorder`, GIF frames go to a worker running `gifenc`.

**Tech Stack:** TypeScript + Vite (`worker.format: 'es'`) + uPlot + Vitest; the existing Rust/WASM `Sim` (unchanged); one new runtime dependency, `gifenc` 1.0.3 (MIT).

**Spec:** docs/superpowers/specs/2026-09-24-sessions-compare-recording-design.md (earlier specs in docs/superpowers/specs/ stay binding where not changed, in particular 2026-09-24-worker-simulation-design.md).

## Global Constraints

- **No simulation change.** Nothing under `crates/` is edited in this milestone. `crates/sugarscape-core/tests/golden.rs`, `tests/legacy.rs` and `tests/fixtures/*` stay unedited and green; Task 14 checks `git diff main -- crates/` is empty.
- **Exact replay.** Replaying a session gives the same world (same `fingerprint`) at the same tick as the live session, at every speed (1×…100×, Max). Task 4 proves it with the real WASM; Task 9 proves B's copy.
- **Old links keep working.** Every existing `#s=` link decodes as before (its painted maps become starting maps with an empty log). `web/src/legacy-share.fixture.ts` is never edited.
- **Deploys as today:** no COOP/COEP, no SharedArrayBuffer. The only new runtime dependency is `gifenc` (MIT); it is added in Task 7, the only task that stages `web/package.json` and `web/package-lock.json`.
- **Main-thread budget:** no main-thread task over 50 ms. GIF encoding runs in a worker; the compare coordinator only sends requests and awaits promises; B's copy steps in B's worker.
- **Limits (verbatim from the spec):** edit log at most 50 000 entries; long-link threshold 32 000 characters; Max in compare mode doubles `n` while both replies arrive within 25 ms, halves above 40 ms, 1 ≤ n ≤ 10 000; recording cell ≥ 8 px, long side ≤ 1080 px; GIF about 15 fps, at most 900 frames; WebM MIME order `video/webm;codecs=vp9`, `video/webm;codecs=vp8`, `video/webm`, `video/mp4`; file names `…-t<from>-t<to>.<ext>`.
- **Copy (verbatim):** chip "Replaying · N edits left ✕"; notice "Replay ended — your edit starts a new branch"; progress "Copying A… t / T"; leave prompt "Keep A / Keep B"; Rules switch "Rules for: A | B"; chart legends "A · <series>" / "B · <series>"; Record button "● Record", while recording "■ m:ss"; menu items **Session (JSON)** (Export) and **Open session…** (Share); stamp text `t = <tick>`.
- **Names are binding across tasks** (each task's Interfaces block repeats the ones it uses): protocol `EditCommand`, `LogEntry`, `Session`, `SessionLog`, commands `session` and `endReplay`, snapshot fields `replayLeft` and `forked`; host `LOG_CAP`; engine `session()`, `replay()`, `open()`, `endReplay()`, `takeWorld()`, `close()`, `replayLeft`, events `'replay'` and `'fork'`, `RunControls`; share `encodeLog`, `decodeLog`, `encodeCompare`, `decodeCompare`, `readCompareHash`, `sessionFileText`, `parseSessionFile`, `CompareState`, `SessionFile`; sessions `SessionSource`, `shareable`, `sessionLink`, `compareLink`, `LONG_LINK`, `LONG_NOTICE`, `LOG_FULL_NOTICE`; compare `Lockstep`, `AdaptiveBatch`, `copyWorld`, `FAST_MS`, `SLOW_MS`, `MAX_BATCH`, `CompareView`, `compareShell`, `askKeep`, `Playground`, `WorldName`; recording `frameLayout`, `pickMime`, `recordingName`, `formatClock`, `Stopwatch`, `GifSampler`, `gifDelay`, `GifBuilder`, `startRecording`, `webmSupport`, `Recording`, `RecordSource`; UI `showNotice`, `buildShareMenu`, `buildExportMenu`, `ExportWorld`, `buildRecordControl`, `Toolbar`, `followChip`, `replayChip`, `WorldSlot`, `buildTools`, `Tools`, `ToolTarget`.
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3`; the commit commands below pass it as a second `-m`. Stage **only** the files named in the task (`git add <paths>`, never `-A`/`.`).
- Web tasks run `(cd web && npm run build && npm test)`. The build regenerates `web/src/wasm-pkg` (gitignored) with `wasm-pack` and runs `tsc --noEmit` then `vite build`; Vitest imports that package (engine.ts, sim-module.ts, determinism.test.ts), so always build before testing. Single test files run with `(cd web && npx vitest run src/<file>.test.ts)`.
- **TypeScript:** `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch` (grouped empty `case`s are fine). Unused imports fail the build — each task lists its import changes. Never pass a possibly-null child to `replaceChildren`/`append`: use `h()`, which skips `null`/`false`. Prefix deliberately unused parameters with `_`.
- **Browser checks are the controller's**, not the implementer's. Each task's "Browser (controller):" line lists the scenarios it affects; the controller runs its puppeteer pass on them (and the full list in Task 14). `?debug` exposes `window.sugarscape = { engine, compare }` (from Task 10; `engine` only before).

## Why this task order

- **Edit log first (Tasks 1–4):** the host records and replays (Task 1, fake `Sim`), the wire carries logs, compare pairs and session files (Task 2, pure), the engine exposes sessions and a replaying Reset (Task 3), and a real-WASM Vitest proves exact replay through encode/decode at mixed speeds including Max (Task 4) before any UI depends on it.
- **Replay UI (Task 5):** the chip, the fork notice, Reset replaying, the Share menu (Copy link with a long-link notice and the log-full fallback, Open session…) and Export → Session (JSON).
- **Recording (Tasks 6–8):** pure frame/timing helpers, then the GIF encoder and its worker (with the `gifenc` dependency), then the recorder and the ● Record control — single-world recording is a complete deliverable before Compare exists.
- **Comparison (Tasks 9–13):** coordinator and engine plumbing with tests first (Task 9: `Lockstep`, `copyWorld`, `Engine.takeWorld/close`), then the Compare mode shell (Task 10: toggle, copy progress, B's grid and headers, lockstep toolbar, Keep A/B), then per-world editing (Task 11: Rules/Inspect/Credit per world, tools per grid), then overlaid charts (Task 12), then the compare-aware exports, links, files and side-by-side recording (Task 13). A reviewer can reject any of 10–13 without the others.
- **Docs and full verification (Task 14).**

## Decisions (where the spec leaves room)

These are binding; each is repeated in the task that implements it.

1. **What the host logs.** The world-changing commands are exactly `EditCommand` = `setConfig | paint | importLandscape | place | erase | infect | vaccinate`. The host applies one through a single `edit(sim, cmd)` path; only if it returns without throwing does it append `{ tick: sim.tick(), cmd }` (so a failed edit — erasing an empty site — is neither logged nor a fork). `setConfig` logs the full new config the page sent. `init`/`reset` start an empty log. At `LOG_CAP` (50 000) entries recording stops and `logFull` is set; edits still apply. The `session` command answers `SessionLog { log, full, tick }` where `log` is the applied entries **followed by the entries still pending**, so a link made mid-replay carries the whole session, and `tick` is the host's tick when it answered (the copy target for Compare).
2. **Replay in the host.** `init` and `reset` both accept `log?` (the spec names `init`; the engine rebuilds with `reset`, so it takes one too). After building, entries with tick ≤ the world's tick (0) apply at once; `step n` and Max batches then advance and, after each tick, apply every pending entry whose tick equals the new tick, in log order. "One tick at a time" is implemented as stepping straight to the next pending entry's tick (`sim.step(k)` gives the same world as `k` × `step(1)` — 7a's determinism test pins that), then applying it; Max batches already step one tick per call. A replayed entry the world rejects with a field error (only possible in a hand-made link) is skipped; a panic stays fatal. Applied entries go through `edit`, so they are re-logged and a replayed session shares back identically. Snapshots carry `replayLeft` after every `init`/`reset` and whenever it changes. `endReplay` drops the pending entries and answers with a snapshot.
3. **Forking.** A page edit that succeeds while entries are pending drops them (pending entries all have ticks after the current one, so dropping before or after applying is the same world); the snapshot of that reply carries `forked: true`, the engine emits `'fork'`, and the page shows "Replay ended — your edit starts a new branch".
4. **Sessions in the engine; what Reset does.** The engine keeps `origin` = the config, seed and starting maps it last built with (`Engine.create`, every rebuild). `session()` (inside `quiet()`) returns `{ session: { ...origin, log }, full, tick }`. The toolbar's Reset calls `engine.replay()` when the seed box still shows the world's seed: it rebuilds from `origin` with the session's log and **keeps** `baseConfig` and `presetId` (a rewind is not a new setup). With a different seed typed, Reset is today's `reset(undefined, seed)` — a new world, empty log; 🎲, a preset change and a reset-requiring rule change likewise start an empty log from `baseConfig` (+ kept painted maps). If the log is full, `replay()` falls back to today's reset. Live changes replayed from a log are part of the log, not the setup: they are not folded into `baseConfig` (only `applyConfig` from this page folds, as today). `open(state)` builds a session opened from a file.
5. **Wire v3.** `#s=` stays base64url(deflate-raw(JSON)); `Wire = { v: 3, c, s, g?, e? }` where `e` is the log as arrays `[tickDelta, code, …args]` (ticks as deltas from the previous entry, starting at 0): `paint` `[dt,'p',x,y,radius,value,good]`, `importLandscape` `[dt,'i',good,base64url]`, `place` `[dt,'a',x,y]` or `[dt,'a',x,y,{sex?,tribe?}]`, `erase` `[dt,'x',x,y]`, `infect` `[dt,'f',x,y,disease]`, `vaccinate` `[dt,'v',x,y,radius,disease]`, `setConfig` `[dt,'c',config]`. Decoding validates every entry (integer non-negative deltas, x, y, good; finite non-negative radius; finite value; disease ≥ −1 for infect, ≥ 0 for vaccinate; known overrides; an object config) and rejects the link otherwise. v1/v2 decode as before with `log: []`; `e` is omitted when the log is empty. The inflate cap rises from 1 MiB to 16 MiB (a 50 000-entry log with live changes can exceed 1 MiB); two existing share tests change accordingly (the "unknown version" test uses v4, the size-cap test inflates past 16 MiB) and the round-trip test expects `log: []`.
6. **Links, files and the Share menu.** Share becomes a menu: **Copy link** and **Open session…**. Copy link encodes the whole session; if the token is longer than `LONG_LINK` (32 000 characters) it still copies it and shows `LONG_NOTICE`; if the log is full it encodes today's setup + painted maps (`baseConfig`, seed, `editedLandscapes()`) and shows `LOG_FULL_NOTICE`. Export gains **Session (JSON)**: `{ "sugarscape": "session", ...Wire }` (the same content as the link, not deflated); in Compare, `{ "sugarscape": "compare", "v": 3, "a": Wire, "b": Wire }`. Open session… rebuilds the page's world from the file (`engine.open`) and clears the address-bar hash; a comparison file enters Compare (Task 13). `#c=` = base64url(deflate-raw(`{ v: 3, a: Wire, b: Wire }`)).
7. **Notices.** Non-error messages (fork, long link, log full, GIF cap, open results) go to a status strip `#notice` (`showNotice(text, ms = 5000)`), not the red crash banner.
8. **Compare engines.** A stays the page's `Engine` object for the whole time (every panel is bound to it); B is a second `Engine` from `Engine.create(state)` with its own worker. "Keep A" calls `b.close()`. "Keep B" calls `a.takeWorld(b)`: with both quiet, A closes its own transport, adopts B's transport (rebinding `onFatal`/`onPost`) and copies B's state (seed, configs, preset, counters, last snapshot, frames, charts cache, selection, follow/trail, overlays, edited maps, origin, `replayLeft`, display), marks B dead, then emits `'reset'`, `'follow'`, `'display'`, `'replay'`, `'select'` (if something is selected) and `'snapshot'` so every panel redraws from B's world.
9. **Lockstep.** In Compare neither engine runs itself (`running` stays false). `Lockstep.pump(now)` (called by the frame loop instead of `engine.pump`) sends `advance(n)` to both and waits for both (`Promise.all`) before the next pair; paused, it calls both engines' `pump` so panels still refresh. `n` = the speed for 1×…100×; at Max `AdaptiveBatch`: start at 1, double when the pair took ≤ `FAST_MS` (25 ms), halve when > `SLOW_MS` (40 ms), clamp to [1, `MAX_BATCH` = 10 000]. Step and Reset (both worlds `replay()`) run exclusively after the pair in flight. A `'reset'` event the coordinator did not cause (🎲, a preset or reset-requiring change on one world) triggers `realign`: after the pair in flight, every world whose tick is not 0 replays (up to three rounds); a coordinator started with unequal ticks realigns at once. It emits `'run'` and `'tick'` (after each pair and each rewind). Entering Compare pauses A and holds the toolbar until B exists; if A's log is full, Compare does not start (a notice explains why). B is built by `copyWorld(session, T, create, progress)`: A's session with its log truncated to ticks ≤ T (A's pending entries beyond T are not copied, as specified), then advanced to T in `AdaptiveBatch`-sized requests with the progress line "Copying A… t / T".
10. **Compare UI.** `#grids` holds figure A (header + `#grid`) and figure B. Each header: the world's label, `seed <n>`, 🎲 (rebuilds that world with a random seed; the other rewinds), and that world's follow and replay chips (the toolbar's chips, seed box and 🎲 hide in Compare; Play/Step/speed/Reset drive the `Lockstep`; the readout shows `t = T · A n · B m agents`). One display for both worlds: A's display controls act on A and are mirrored to B on every A `'display'` event. Rules, Inspect and Credit are separate panel instances per world behind `WorldSlot`s: Rules shows the "Rules for: A | B" switch; Inspect and Credit show the world last clicked (pointerdown on its grid, or a Credit/Inspect link in its panel) under a "World A"/"World B" label. Tools route each grid's clicks to that grid's world (`Tools.attach`); the disease picker lists, and image import writes to, the focused world; the paint tool's good list and display changes use A. The Credit tab shows while either world has credit on. Switching to Experiments pauses the lockstep.
11. **Charts for two worlds.** `ChartsPanel` becomes table-driven (one `CHARTS` list of chart definitions whose lines are functions of a config) and draws `worlds` = `[A]` or `[A, B]` (`setCompare(b)`). It registers one wants provider and its listeners per world (B's removed on leave) and keeps per-world distribution state. Each plot's series are A's then B's (B dashed `[6, 4]`, ±SD `[2, 3]`, hollow points), labeled "A · <series>" / "B · <series>" (legends always shown in Compare). Multi-world data use `overlayData(tables)`: the sorted union of the tables' x values, each line `undefined` where its table has no point (uPlot draws through `undefined`) and `null` where the data has a gap (uPlot breaks the line) — so two worlds' different downsampled ticks and price grids share one axis. The wealth histogram in Compare is two step outlines (`histTable` = bin edges → counts, `paths.stepped({ align: 1 })`); single-world keeps bars. A chart (and its section) shows when it would show for either world. Plots are rebuilt only when the lines signature of any world changes or Compare starts/ends (the same effect as 7a's goods/group rebuilds).
12. **Exports in Compare.** Statistics (CSV), Agents (CSV) and Grid (PNG) become rows "Statistics (CSV) [A] [B]" (the menu is rebuilt each time it opens); file stems gain `-A`/`-B`. Charts (PNG) exports the overlaid canvases. Session (JSON) exports the comparison. Opening a session file while comparing first leaves Compare keeping A.
13. **Recording frames.** `frameLayout(grids)` places the grids side by side (4 px gap), top-aligned, at `scale` = clamp(floor((1080 − gaps) / Σwidth), floor(1080 / max height)) to [1, 8] px per cell. The recording canvas keeps the size computed at start; a later layout of a different size (a grid resized by a reset) is scaled to fit, centered. Each grid's canvas **as drawn** (overlays, trail, selection) is copied with `imageSmoothingEnabled = false`; Compare draws "A"/"B" tags; the stamp draws `t = <tick>` bottom-left. A frame is captured once per displayed snapshot: in single mode after a frame loop draw that followed an engine `'snapshot'`; in Compare after each lockstep pair (`'tick'`). Recording follows the run state (the controls' `'run'` event and a 500 ms timer call `sync()`); a reset just continues. The first frame is captured when recording starts, so the file is never empty.
14. **WebM and GIF.** WebM: `captureStream(0)`, `track.requestFrame()` per captured frame, `MediaRecorder` with the first supported MIME of the spec's list (extension `webm`, or `mp4` for `video/mp4`), `start(1000)`, `pause()`/`resume()` with the world; `webmSupport()` null disables the WebM item. GIF: frames sampled on *recorded* time (a `Stopwatch` that excludes pauses) at `1000/15 − 2` ms spacing (the 2 ms absorbs 60 Hz jitter); each frame is held until the next is taken so its delay is the recorded time between them (`gifDelay` rounds to 10 ms, minimum 20 ms; the last frame gets 1000/15); frames are `getImageData` RGBA buffers transferred to `gif-worker.ts`, which runs `GifBuilder` (`quantize` 256 colors + `applyPalette` + `writeFrame` with a per-frame palette) and acknowledges each frame; while 3 frames are unacknowledged capture skips (bounded memory; the next frame's delay spans the gap); at 900 frames recording stops with a notice; stopping shows "Finishing GIF… k / n" until the worker returns the file. Names: `<base>-t<from>-t<to>.<ext>` with base `sugarscape-<preset|custom>-seed<seed>` (Compare: `sugarscape-compare-seed<a>-vs-seed<b>`). The ● Record control sits in the toolbar's end group, before Share.
15. **`gifenc` typing.** The package ships no types: `web/src/gifenc.d.ts` declares the three functions used (`GIFEncoder`, `quantize`, `applyPalette`). Vitest imports it (verified: named imports resolve through Vitest's CJS interop) and Vite bundles it into the ES worker (verified).
16. **Determinism tests** run in Vitest with the real WASM exactly like 7a's (`initSync` + `node:fs`), on `v-2-endemic` (so infections and vaccinations are real edits), over `InlineTransport`, including a Max run (real timers).

## File Structure

```
web/src/protocol.ts                      MOD  EditCommand, LogEntry, Session, SessionLog; session/endReplay; init/reset log; replayLeft/forked (1)
web/src/sim-host.ts, sim-host.test.ts    MOD  edit log, replay, fork, LOG_CAP (1)
web/src/share.ts, share.test.ts          MOD  wire v3, encodeLog/decodeLog, compare links, session files (2)
web/src/engine.ts, engine.test.ts        MOD  origin, session/replay/open/endReplay, 'replay'/'fork' (3); takeWorld/close (9); RunControls (10)
web/src/determinism.test.ts              MOD  sessions replay exactly (4); B's copy (9)
web/src/sessions.ts, sessions.test.ts    NEW  shareable, sessionLink, compareLink, notices (5)
web/src/ui/notice.ts                     NEW  showNotice (5)
web/src/ui/share-menu.ts                 NEW  Copy link / Open session… (5)
web/src/ui/export-menu.ts                NEW  Export menu with per-world rows and Session (JSON) (5)
web/src/ui/toolbar.ts                    MOD  followChip, replayChip, replaying Reset (5); Toolbar class + compare (10)
web/src/main.ts                          MOD  (5, 8, 10, 11, 12, 13)
web/index.html, web/src/style.css        MOD  (5, 8, 10, 11)
web/src/recording/frames.ts, frames.test.ts        NEW  layout, MIME, names, clock, sampler (6)
web/src/gifenc.d.ts                      NEW  (7)
web/src/recording/gif-encode.ts, gif-encode.test.ts NEW  GifBuilder + worker messages (7)
web/src/recording/gif-worker.ts          NEW  (7)
web/package.json, web/package-lock.json  MOD  gifenc (7)
web/src/recording/recorder.ts            NEW  Composer, WebM and GIF recordings (8)
web/src/ui/record-control.ts             NEW  ● Record menu / ■ m:ss (8)
web/src/compare/lockstep.ts, lockstep.test.ts      NEW  Lockstep, AdaptiveBatch, copyWorld (9)
web/src/compare/compare-view.ts          NEW  (10); MOD (11, 12)
web/src/ui/world-slot.ts                 NEW  (11)
web/src/ui/tools.ts                      REWRITE  per-grid targets (11)
web/src/ui/series-data.ts, series-data.test.ts     MOD  overlayData, histTable, barsData, supplyDemandTable (12)
web/src/ui/charts-panel.ts               REWRITE  table-driven, one or two worlds (12)
README.md, docs/roadmap.md               MOD  (14)
```

---
### Task 1: The edit log and replay in the host

*Mechanical (full code).* Browser: nothing to check (the engine does not ask for sessions yet; behavior is unchanged).

**Files:**
- Modify: `web/src/protocol.ts`
- Modify: `web/src/sim-host.ts`
- Test: `web/src/sim-host.test.ts`

**Interfaces:**
- Consumes: `SimHost`, `SimLike`, `Command`, `Result`, `WorldSnapshot` (7a).
- Produces:
  - `protocol.ts`: `type EditCommand = Extract<Command, { type: 'setConfig' | 'paint' | 'importLandscape' | 'place' | 'erase' | 'infect' | 'vaccinate' }>`; `interface LogEntry { tick: number; cmd: EditCommand }`; `interface Session { config: Config; seed: number; landscapes: (Uint8Array | null)[]; log: LogEntry[] }`; `interface SessionLog { log: LogEntry[]; full: boolean; tick: number }`; commands `{ type: 'session' }`, `{ type: 'endReplay' }`; `init` and `reset` gain `log?: LogEntry[]`; `Result` ok gains `session?: SessionLog`; `WorldSnapshot` gains `replayLeft?: number` and `forked?: true`.
  - `sim-host.ts`: `export const LOG_CAP = 50_000`.

- [ ] **Step 1: Write the failing tests**

In `web/src/sim-host.test.ts`, change the imports to:
```ts
import { describe, expect, it, vi } from 'vitest';
import { fakeModule } from './fake-sim.fixture';
import type { Command, DisplayState, EditCommand, HostMessage, HostReply, LogEntry, SessionLog, Wants, WorldSnapshot } from './protocol';
import { BATCH_MS, channelDefer, LOG_CAP, serve, SimHost } from './sim-host';
import type { Config } from './types';
```
and append at the end of the file:
```ts
describe('SimHost edit log and replay', () => {
  const place = (x: number, y: number): EditCommand => ({ type: 'place', x, y, overrides: {} });
  const paint: EditCommand = { type: 'paint', x: 0, y: 0, radius: 1, value: 3, good: 0 };
  const session = (t: ReturnType<typeof start>): SessionLog => {
    const r = t.send({ type: 'session' }).result;
    if (!r.ok || !r.session) throw new Error(JSON.stringify(r));
    return r.session;
  };

  it('logs world-changing commands that succeed, with the tick they were applied at', () => {
    const t = start();
    t.send({ type: 'step', n: 2 });
    t.send(place(0, 2));
    expect(t.send({ type: 'erase', x: 3, y: 2 }).result.ok).toBe(false); // nobody there: not logged
    t.send({ type: 'follow', id: 1 });
    t.send({ type: 'inspect', target: { x: 1, y: 1 } });
    t.send({ type: 'setDisplay', display });
    t.send({ type: 'step', n: 1 });
    t.send(paint);
    t.send({ type: 'importLandscape', good: 0, capacities: new Uint8Array(12) });
    t.send({ type: 'infect', x: 0, y: 2, disease: -1 });
    t.send({ type: 'vaccinate', x: 0, y: 2, radius: 1, disease: 0 });
    t.send({ type: 'setConfig', config });
    t.send({ type: 'refresh' });
    t.send({ type: 'seriesCsv' });
    const s = session(t);
    expect(s.full).toBe(false);
    expect(s.tick).toBe(3);
    expect(s.log.map((e) => [e.tick, e.cmd.type])).toEqual([
      [2, 'place'],
      [3, 'paint'],
      [3, 'importLandscape'],
      [3, 'infect'],
      [3, 'vaccinate'],
      [3, 'setConfig'],
    ]);
    expect(s.log[5].cmd).toEqual({ type: 'setConfig', config });
  });

  it('starts an empty log with every new world', () => {
    const t = start();
    t.send(place(0, 2));
    t.send({ type: 'reset', config, seed: 2, landscapes: [] });
    expect(session(t).log).toEqual([]);
  });

  it(`stops recording past ${LOG_CAP} entries and says so`, () => {
    const t = start();
    const infect: Command = { type: 'infect', x: 0, y: 0, disease: -1 };
    for (let i = 0; i < LOG_CAP; i++) t.send(infect);
    expect(session(t).full).toBe(false);
    t.send(infect);
    const s = session(t);
    expect(s.full).toBe(true);
    expect(s.log).toHaveLength(LOG_CAP);
  });

  it('replays tick-0 entries at once and later ones inside a step, stopping at each entry’s tick', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 0, cmd: paint },
      { tick: 3, cmd: place(0, 2) },
      { tick: 3, cmd: { type: 'erase', x: 0, y: 2 } },
      { tick: 5, cmd: place(0, 0) },
    ];
    const built = t.snap(t.send({ type: 'reset', config, seed: 1, landscapes: [], log }));
    expect(built.replayLeft).toBe(3);
    expect(built.editedLandscapes).toEqual([new Uint8Array(12).fill(7)]); // painted at tick 0
    const sim = t.module.sims.at(-1)!;
    expect(t.snap(t.send({ type: 'step', n: 2 })).replayLeft).toBeUndefined(); // unchanged
    expect(sim.stepCalls).toBe(1);
    expect(t.snap(t.send({ type: 'step', n: 2 }))).toMatchObject({ tick: 4, population: 1, replayLeft: 1 });
    expect(sim.stepCalls).toBe(3); // 2 → 3, apply both entries at 3, 3 → 4
    expect(t.log.filter((line) => line.startsWith('place'))).toHaveLength(1);
    expect(t.snap(t.send({ type: 'step', n: 10 }))).toMatchObject({ tick: 14, population: 2, replayLeft: 0 });
    expect(sim.stepCalls).toBe(5); // 4 → 5, apply, 5 → 14
    expect(session(t).log).toEqual(log); // re-logged identically
  });

  it('replays inside Max batches', () => {
    let clock = 0;
    const t = start(() => clock++);
    const log: LogEntry[] = [{ tick: 4, cmd: place(0, 2) }];
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log });
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch();
    expect(post.tick).toBeGreaterThan(4);
    expect(post).toMatchObject({ population: 2, replayLeft: 0 });
    t.send({ type: 'stop' });
    expect(session(t).log).toEqual(log);
  });

  it('forks: a page edit that succeeds while entries are pending drops them, then is applied and logged', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 1, cmd: place(0, 2) },
      { tick: 5, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log });
    expect(t.snap(t.send({ type: 'step', n: 2 })).replayLeft).toBe(1);
    expect(t.send({ type: 'erase', x: 3, y: 0 }).result.ok).toBe(false); // failed: no fork
    expect(session(t).log.map((e) => e.tick)).toEqual([1, 5]);
    expect(t.snap(t.send(paint))).toMatchObject({ forked: true, replayLeft: 0 });
    const later = t.snap(t.send({ type: 'step', n: 5 }));
    expect(later.forked).toBeUndefined();
    expect(later.population).toBe(2); // the erase at 5 was dropped
    expect(session(t).log.map((e) => [e.tick, e.cmd.type])).toEqual([
      [1, 'place'],
      [2, 'paint'],
    ]);
  });

  it('endReplay drops the pending entries and keeps the world', () => {
    const t = start();
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log: [{ tick: 2, cmd: place(0, 2) }] });
    const s = t.snap(t.send({ type: 'endReplay' }));
    expect(s).toMatchObject({ tick: 0, replayLeft: 0 });
    expect(s.forked).toBeUndefined();
    expect(t.snap(t.send({ type: 'step', n: 3 })).population).toBe(1);
    expect(session(t).log).toEqual([]);
  });

  it('includes pending entries in the session, after the applied ones', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 1, cmd: place(0, 2) },
      { tick: 9, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log });
    t.send({ type: 'step', n: 3 });
    expect(session(t)).toEqual({ log, full: false, tick: 3 });
  });

  it('skips a replayed entry the world rejects, but a panic is still fatal', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 0, cmd: { type: 'erase', x: 3, y: 2 } },
      { tick: 0, cmd: place(0, 2) },
    ];
    expect(t.snap(t.send({ type: 'reset', config, seed: 1, landscapes: [], log }))).toMatchObject({ population: 2, replayLeft: 0 });
    expect(session(t).log.map((e) => e.cmd.type)).toEqual(['place']);
    const panic: LogEntry[] = [{ tick: 0, cmd: { ...paint, value: -1 } as EditCommand }];
    const r = t.send({ type: 'reset', config, seed: 1, landscapes: [], log: panic });
    expect(r.result).toEqual({ ok: false, fatal: 'The simulation stopped: unreachable executed' });
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/sim-host.test.ts)`
Expected: FAIL — the import of `LOG_CAP` (and the types `EditCommand`, `LogEntry`, `SessionLog`) does not exist; the new describe's tests fail (`session` is not a command).

- [ ] **Step 3: Extend the protocol**

In `web/src/protocol.ts`:
- in `WorldSnapshot`, after `diseaseList?: DiseaseEntry[];` add:
```ts
  /** Edits still to replay: after every init and reset, and whenever it changes. */
  replayLeft?: number;
  /** A page edit just dropped the edits still to replay (the session branched here). */
  forked?: true;
```
- in `Command`, replace the `init` and `reset` members with:
```ts
  | { type: 'init'; config: Config; seed: number; landscapes: (Uint8Array | null)[]; display: DisplayState; log?: LogEntry[] }
  | { type: 'reset'; config: Config; seed: number; landscapes: (Uint8Array | null)[]; log?: LogEntry[] }
```
  and insert before `| { type: 'run' }`:
```ts
  | { type: 'session' }
  | { type: 'endReplay' }
```
- directly after the `Command` type add:
```ts
/** The commands that change the world: logged with the tick they were applied at, and replayed (Decision 1). */
export type EditCommand = Extract<
  Command,
  { type: 'setConfig' | 'paint' | 'importLandscape' | 'place' | 'erase' | 'infect' | 'vaccinate' }
>;

/** One edit of a session: applied after tick `tick` was computed, before tick + 1 is. */
export interface LogEntry { tick: number; cmd: EditCommand }

/** What a world was built from and every edit since: replaying it rebuilds the world exactly. */
export interface Session { config: Config; seed: number; landscapes: (Uint8Array | null)[]; log: LogEntry[] }

/**
 * The `session` command's answer: the log (the entries applied so far, then those still to
 * replay), whether it overflowed `LOG_CAP`, and the tick when it was taken.
 */
export interface SessionLog { log: LogEntry[]; full: boolean; tick: number }
```
- replace the `Result` type's first member with:
```ts
  | { ok: true; snapshot?: WorldSnapshot; value?: string; session?: SessionLog }
```

- [ ] **Step 4: Record, replay and fork in the host**

In `web/src/sim-host.ts`:
- extend the protocol import list with `type EditCommand,` and `type LogEntry,` (keep it alphabetical: after `type DisplayState,` and after `type HostRequest,` respectively).
- after `export const POST_MS = 33;` add:
```ts
/** The edit log holds at most this many entries; past it, edits still apply but are not recorded (Decision 1). */
export const LOG_CAP = 50_000;
```
- in `class SimHost`, after the `max` field, add:
```ts
  /** Edits applied to this world since it was built, with their ticks: the session's log (Decision 1). */
  private log: LogEntry[] = [];
  /** Set once the log reached LOG_CAP: later edits apply but are not recorded. */
  private logFull = false;
  /** A session's entries still to replay: `pending[cursor…]`, in log order (Decision 2). */
  private pending: LogEntry[] = [];
  private cursor = 0;
  /** The `replayLeft` last sent; -1 sends it with the next snapshot. */
  private replaySent = -1;
  /** The next snapshot says a page edit dropped the pending entries (Decision 3). */
  private forkDue = false;
```
- replace the body of `batch()` from `const start = this.now();` to the end of the method with:
```ts
    const start = this.now();
    const cap = max.pool.length > 0 ? Math.min(start + BATCH_MS, max.posted + POST_MS) : start + BATCH_MS;
    // One tick at a time, applying any edit due at each (Decision 2).
    do this.advance(sim, 1);
    while (this.now() < cap);
    const now = this.now();
    const frame = now - max.posted >= POST_MS ? max.pool.pop() : undefined;
    if (!frame) return null;
    max.posted = now;
    return this.snapshot(sim, max.wants, frame);
  }
```
- replace the whole `private apply(req: HostRequest, spare: ArrayBuffer[]): Result { … }` method with:
```ts
  private apply(req: HostRequest, spare: ArrayBuffer[]): Result {
    const { cmd, frame } = req;
    const wants = req.wants ?? {};
    if (cmd.type === 'ready') return { ok: true };
    // init, reset and setConfig keep Max running on the (possibly new) world if it was running;
    // in practice the engine quiesces (sends `stop`) before any of them, so this doesn't happen.
    if (cmd.type === 'init' || cmd.type === 'reset') {
      // Built before the old world is freed: a bad config keeps the world (and its log).
      const next = this.module.create(JSON.stringify(cmd.config), cmd.seed, cmd.landscapes);
      this.sim?.free();
      this.sim = next;
      if (cmd.type === 'init') this.display = cmd.display;
      this.configDue = true;
      this.landscapesDue = true;
      // A new world starts a new log; a session's log is replayed into it (Decision 2).
      this.log = [];
      this.logFull = false;
      this.pending = cmd.log ?? [];
      this.cursor = 0;
      this.replaySent = -1;
      this.forkDue = false;
      this.replayDue(next);
      // The engine clears its selection on a new world.
      return this.reply(next, { ...wants, select: undefined }, frame);
    }
    const sim = this.sim;
    if (!sim) throw NO_WORLD;
    switch (cmd.type) {
      case 'setConfig':
      case 'paint':
      case 'importLandscape':
      case 'place':
      case 'erase':
      case 'infect':
      case 'vaccinate':
        this.edit(sim, cmd);
        // Only an edit that succeeded branches a replay (Decision 3).
        this.fork();
        return this.reply(sim, wants, frame);
      case 'step':
        this.advance(sim, cmd.n);
        return this.reply(sim, wants, frame);
      case 'refresh':
        // The only command exempt from the charts throttle: it is already paced client-side by
        // REFRESH_MS (Engine.pump), and it is the only path the paused catch-up needs. Every other
        // command — including drag-driven ones like paint, which carry no debounce of their own —
        // stays throttled, or a drag would resend full chart groups on nearly every pointermove.
        return this.reply(sim, wants, frame, undefined, false);
      case 'setDisplay':
        this.display = cmd.display;
        return this.reply(sim, wants, frame);
      case 'follow': {
        if (cmd.id === null) sim.unfollow();
        else sim.follow(cmd.id);
        // At send time the engine's own wants still describe the old follow state (it only
        // learns the new one from this reply), so while Max is running the loop's own copy is
        // patched here too — or its posts would keep reporting the trail as it was before this.
        if (this.max) this.max.wants = { ...this.max.wants, trail: cmd.id !== null };
        return this.reply(sim, { ...wants, trail: cmd.id !== null }, frame);
      }
      case 'inspect': {
        const { target } = cmd;
        const at = 'agentId' in target ? sim.locate(target.agentId) : Uint32Array.of(target.x, target.y);
        const selected = at ? this.selectAt(sim, at[0], at[1]) : null;
        // Same reasoning as `follow`: the new selection is this command's own result, not
        // yet reflected in `wants.select`, so the loop's copy is patched from it directly.
        if (this.max) {
          this.max.wants = {
            ...this.max.wants,
            select: selected ? { x: selected.x, y: selected.y, agentId: selected.agentId } : undefined,
          };
        }
        return this.reply(sim, wants, frame, selected);
      }
      case 'seriesCsv':
        return { ok: true, value: sim.export_series_csv() };
      case 'agentsCsv':
        return { ok: true, value: sim.export_agents_csv() };
      case 'fingerprint':
        return { ok: true, value: sim.fingerprint() };
      case 'session':
        // Applied entries, then those still to replay: a link made mid-replay carries the whole session.
        return {
          ok: true,
          session: { log: [...this.log, ...this.pending.slice(this.cursor)], full: this.logFull, tick: sim.tick() },
        };
      case 'endReplay':
        this.pending = [];
        this.cursor = 0;
        return this.reply(sim, wants, frame);
      case 'run':
        // A second `run` while already running keeps the pooled buffers (and adds this one, if
        // any) instead of replacing the pool and losing them; `handle` has already applied this
        // request's `wants` to the loop, same as any other command.
        if (this.max) {
          if (frame) this.max.pool.push(frame);
        } else {
          this.max = { wants, pool: frame ? [frame] : [], posted: this.now() };
        }
        return { ok: true };
      case 'frame':
        if (this.max && frame) this.max.pool.push(frame);
        return { ok: true };
      case 'stop': {
        const max = this.max;
        this.max = null;
        if (!max) return this.reply(sim, wants);
        // A pooled buffer is preferred; with none pooled, the buffer this `stop` itself just lent
        // (if any) is used instead of being left unrendered and handed straight back as spare.
        const last = max.pool.pop() ?? frame;
        spare.push(...max.pool);
        return this.reply(sim, max.wants, last);
      }
    }
  }

  /** Applies a world-changing command and records it with the tick (Decision 1); throws the core's field errors. */
  private edit(sim: SimLike, cmd: EditCommand): void {
    switch (cmd.type) {
      case 'setConfig':
        sim.set_config(JSON.stringify(cmd.config));
        this.configDue = true;
        this.landscapesDue = true;
        break;
      case 'paint':
        sim.paint_capacity(cmd.x, cmd.y, cmd.radius, cmd.value, cmd.good);
        this.landscapesDue = true;
        break;
      case 'importLandscape':
        sim.set_landscape(cmd.good, cmd.capacities);
        this.landscapesDue = true;
        break;
      case 'place':
        sim.place_agent(cmd.x, cmd.y, JSON.stringify(cmd.overrides));
        break;
      case 'erase':
        sim.remove_agent(cmd.x, cmd.y);
        break;
      case 'infect':
        sim.infect(cmd.x, cmd.y, cmd.disease);
        break;
      case 'vaccinate':
        sim.vaccinate(cmd.x, cmd.y, cmd.radius, cmd.disease);
        break;
    }
    if (this.log.length < LOG_CAP) this.log.push({ tick: sim.tick(), cmd });
    else this.logFull = true;
  }

  /** A page edit while entries are pending drops them: the session branches here (Decision 3). */
  private fork(): void {
    if (this.cursor >= this.pending.length) return;
    this.pending = [];
    this.cursor = 0;
    this.forkDue = true;
  }

  /** Applies every pending entry whose tick has been reached, in log order (Decision 2). */
  private replayDue(sim: SimLike): void {
    const tick = sim.tick();
    while (this.cursor < this.pending.length && this.pending[this.cursor].tick <= tick) {
      const { cmd } = this.pending[this.cursor++];
      try {
        this.edit(sim, cmd);
      } catch (e) {
        // An entry the world rejects (only a hand-made link has one) is skipped; a panic is fatal.
        if (typeof e !== 'string') throw e;
      }
    }
    if (this.pending.length > 0 && this.cursor >= this.pending.length) {
      this.pending = [];
      this.cursor = 0;
    }
  }

  /**
   * Steps `n` ticks. While entries are pending it stops at each entry's tick and applies it
   * (`step(k)` is the same world as k single steps, so it steps straight there).
   */
  private advance(sim: SimLike, n: number): void {
    let left = n;
    while (left > 0) {
      const next = this.pending[this.cursor]?.tick;
      const k = next === undefined ? left : Math.min(left, Math.max(1, next - sim.tick()));
      const from = sim.tick();
      sim.step(k);
      this.fired(from, sim.tick());
      this.replayDue(sim);
      left -= k;
    }
  }
```
- in `snapshot(…)`, directly after the `const s: WorldSnapshot = { … };` object, add:
```ts
    const left = this.pending.length - this.cursor;
    if (left !== this.replaySent) {
      s.replayLeft = left;
      this.replaySent = left;
    }
    if (this.forkDue) {
      s.forked = true;
      this.forkDue = false;
    }
```

- [ ] **Step 5: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/sim-host.test.ts)`
Expected: every test passes, including the 9 in "SimHost edit log and replay" (the earlier host tests are unchanged: a plain `step` snapshot carries no `replayLeft`, since `init` already sent 0).

- [ ] **Step 6: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 7: Commit**

```bash
git add web/src/protocol.ts web/src/sim-host.ts web/src/sim-host.test.ts
git commit -m "Record and replay the edit log in the simulation host" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 2: Session wire — v3 share links, compare links and session files

*Mechanical (full code).* Browser (controller): Share → reload still reproduces the setup and painted maps (the link is now v3 without `e`); the legacy link `#s=<LEGACY_SHARE_TOKEN>` still opens with its map.

**Files:**
- Modify: `web/src/share.ts`
- Test: `web/src/share.test.ts`

**Interfaces:**
- Consumes: `EditCommand`, `LogEntry`, `PlaceOverrides` (Task 1, protocol.ts).
- Produces (`share.ts`):
  - `interface ShareState { config: Config; seed: number; landscapes?: (Uint8Array | null)[]; log?: LogEntry[] }` (decoding always sets `log`)
  - `interface CompareState { a: ShareState; b: ShareState }`
  - `type SessionFile = { kind: 'session'; state: ShareState } | { kind: 'compare'; state: CompareState }`
  - `encodeLog(log: LogEntry[]): unknown[][]`, `decodeLog(e: unknown): LogEntry[]` (throws on anything malformed)
  - `encodeShare(state: ShareState): Promise<string>` (v3), `decodeShare(token: string): Promise<ShareState>`
  - `encodeCompare(state: CompareState): Promise<string>`, `decodeCompare(token: string): Promise<CompareState>`, `readCompareHash(hash?: string): string | null` (`#c=`)
  - `sessionFileText(file: SessionFile): string`, `parseSessionFile(text: string): SessionFile`
  - unchanged: `bytesToBase64Url`, `base64UrlToBytes`, `readHash`, `encodeSweep`, `decodeSweep`, `readSweepHash`

- [ ] **Step 1: Write the failing tests**

In `web/src/share.test.ts`:
- replace the imports with:
```ts
import { describe, expect, it } from 'vitest';
import type { Sweep } from './experiments/types';
import { LEGACY_SHARE_TOKEN } from './legacy-share.fixture';
import type { LogEntry } from './protocol';
import {
  base64UrlToBytes,
  bytesToBase64Url,
  decodeCompare,
  decodeLog,
  decodeShare,
  decodeSweep,
  encodeCompare,
  encodeLog,
  encodeShare,
  encodeSweep,
  parseSessionFile,
  readCompareHash,
  readHash,
  readSweepHash,
  sessionFileText,
} from './share';
import type { Config } from './types';
```
- below `const config = …;` add:
```ts
/** A share token for any wire object (to test what the encoder would never write). */
async function tokenOf(wire: unknown): Promise<string> {
  const compressed = new Blob([JSON.stringify(wire)]).stream().pipeThrough(new CompressionStream('deflate-raw'));
  return bytesToBase64Url(new Uint8Array(await new Response(compressed).arrayBuffer()));
}

const log: LogEntry[] = [
  { tick: 0, cmd: { type: 'paint', x: 3, y: 4, radius: 1.5, value: 2, good: 1 } },
  { tick: 0, cmd: { type: 'importLandscape', good: 0, capacities: Uint8Array.from({ length: 2500 }, (_, i) => i % 11) } },
  { tick: 7, cmd: { type: 'place', x: 1, y: 2, overrides: {} } },
  { tick: 7, cmd: { type: 'place', x: 2, y: 2, overrides: { sex: 'female', tribe: 'red' } } },
  { tick: 12, cmd: { type: 'erase', x: 1, y: 2 } },
  { tick: 40, cmd: { type: 'infect', x: 9, y: 9, disease: -1 } },
  { tick: 41, cmd: { type: 'vaccinate', x: 9, y: 9, radius: 2, disease: 3 } },
  { tick: 1000, cmd: { type: 'setConfig', config } },
];
```
- in `it('round-trips config and seed', …)` change the last expectation to `expect(back).toEqual({ config, seed: 123456789, log: [] });`
- in `it('rejects an unknown version', …)` change `v: 3` to `v: 4`.
- replace `it('rejects a payload that decompresses past the size cap', …)` with:
```ts
  it('rejects a payload that decompresses past the size cap', async () => {
    const wire = JSON.stringify({ v: 1, s: 1, c: {}, l: 'A'.repeat(17 * 1024 * 1024) });
    const compressed = new Blob([wire]).stream().pipeThrough(new CompressionStream('deflate-raw'));
    const token = bytesToBase64Url(new Uint8Array(await new Response(compressed).arrayBuffer()));
    expect(token.length).toBeLessThan(40_000);
    await expect(decodeShare(token)).rejects.toThrow('not a SugarScape share link');
  });
```
- append at the end of the file:
```ts
describe('session links', () => {
  it('round-trips an edit log of every kind', async () => {
    const back = await decodeShare(await encodeShare({ config, seed: 5, log }));
    expect(back).toEqual({ config, seed: 5, log });
  });

  it('writes entries compactly, with ticks as deltas', () => {
    expect(encodeLog(log.slice(2, 5))).toEqual([
      [7, 'a', 1, 2],
      [0, 'a', 2, 2, { sex: 'female', tribe: 'red' }],
      [5, 'x', 1, 2],
    ]);
    expect(decodeLog(encodeLog(log))).toEqual(log);
  });

  it('decodes links made before the edit log with an empty log', async () => {
    expect(await decodeShare(await tokenOf({ v: 2, c: config, s: 9, g: [null] }))).toEqual({
      config,
      seed: 9,
      landscapes: [null],
      log: [],
    });
    expect((await decodeShare(LEGACY_SHARE_TOKEN)).log).toEqual([]);
  });

  it('rejects a malformed edit log', async () => {
    const bad: unknown[] = [
      [[0, 'q', 1]],
      [[-1, 'x', 1, 1]],
      [[0, 'x', 1.5, 1]],
      [[0, 'p', 1, 1, -1, 2, 0]],
      [[0, 'a', 1, 1, { sex: 'other' }]],
      [[0, 'f', 1, 1, -2]],
      [[0, 'i', 0, 7]],
      [[0, 'c', []]],
      'x',
    ];
    for (const e of bad) {
      await expect(decodeShare(await tokenOf({ v: 3, c: config, s: 1, e }))).rejects.toThrow('not a SugarScape share link');
    }
  });
});

describe('compare links and session files', () => {
  const b = { config: { ...config, population: 10 } as Config, seed: 6, landscapes: [null, new Uint8Array(2500).fill(3)], log: log.slice(0, 3) };

  it('round-trips two sessions in a #c= link', async () => {
    const token = await encodeCompare({ a: { config, seed: 5, log }, b });
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    const back = await decodeCompare(token);
    expect(back.a).toEqual({ config, seed: 5, log });
    expect(back.b.seed).toBe(6);
    expect(back.b.log).toEqual(b.log);
    expect(back.b.landscapes?.[0]).toBeNull();
    expect(Array.from(back.b.landscapes![1]!)).toEqual(Array.from(b.landscapes[1]!));
    await expect(decodeCompare(await encodeShare({ config, seed: 1 }))).rejects.toThrow('not a SugarScape compare link');
    await expect(decodeCompare('garbage')).rejects.toThrow('not a SugarScape compare link');
  });

  it('reads #c= apart from #s= and #x=', () => {
    expect(readCompareHash('#c=ab_-9')).toBe('ab_-9');
    expect(readCompareHash('#s=abc')).toBeNull();
    expect(readHash('#c=abc')).toBeNull();
    expect(readSweepHash('#c=abc')).toBeNull();
  });

  it('writes and reads session files of one world or two', () => {
    expect(parseSessionFile(sessionFileText({ kind: 'session', state: { config, seed: 5, log } }))).toEqual({
      kind: 'session',
      state: { config, seed: 5, log },
    });
    const two = parseSessionFile(sessionFileText({ kind: 'compare', state: { a: { config, seed: 5, log }, b } }));
    if (two.kind !== 'compare') throw new Error(two.kind);
    expect(two.state.a.log).toEqual(log);
    expect(two.state.b.seed).toBe(6);
    expect(() => parseSessionFile('{"v":3}')).toThrow('not a SugarScape session file');
    expect(() => parseSessionFile('nope')).toThrow('not a SugarScape session file');
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/share.test.ts)`
Expected: FAIL — `decodeCompare`, `decodeLog`, `encodeCompare`, `encodeLog`, `parseSessionFile`, `readCompareHash`, `sessionFileText` are not exported.

- [ ] **Step 3: Implement**

Replace `web/src/share.ts` with:
```ts
import { classifyFile } from './experiments/file';
import type { Sweep } from './experiments/types';
import type { EditCommand, LogEntry, PlaceOverrides } from './protocol';
import type { Config } from './types';

/**
 * `landscapes[i]` is good i's painted map (null: generated) — the starting maps — and `log` the
 * edits to replay (Decision 5). Decoding always sets `log` (empty for links made before it).
 */
export interface ShareState { config: Config; seed: number; landscapes?: (Uint8Array | null)[]; log?: LogEntry[] }

/** Two sessions side by side: a `#c=` link or a comparison file (Decision 6). */
export interface CompareState { a: ShareState; b: ShareState }

/** What Export → Session (JSON) writes and Share → Open session… reads. */
export type SessionFile = { kind: 'session'; state: ShareState } | { kind: 'compare'; state: CompareState };

/** v1: before N goods (`l` = sugar's map). v2: `g` = one entry per good. v3: `e` = the edit log. */
interface Wire { v: 1 | 2 | 3; c: Config; s: number; l?: string; g?: (string | null)[]; e?: unknown[][] }

export function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export function base64UrlToBytes(text: string): Uint8Array {
  const b64 = text.replace(/-/g, '+').replace(/_/g, '/') + '==='.slice((text.length + 3) % 4);
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

async function deflate(bytes: Uint8Array): Promise<Uint8Array> {
  const out = new Blob([new Uint8Array(bytes)]).stream().pipeThrough(new CompressionStream('deflate-raw'));
  return new Uint8Array(await new Response(out).arrayBuffer());
}

/**
 * Upper bound on a decompressed link payload (guards against deflate bombs). A full 50 000-entry
 * log with live rule changes can pass 1 MiB, so the cap is 16 MiB (Decision 5).
 */
const MAX_DECODED_BYTES = 16 * 1024 * 1024;

async function inflateCapped(bytes: Uint8Array): Promise<Uint8Array> {
  const reader = new Blob([new Uint8Array(bytes)])
    .stream()
    .pipeThrough(new DecompressionStream('deflate-raw'))
    .getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    total += value.byteLength;
    if (total > MAX_DECODED_BYTES) {
      await reader.cancel();
      throw new Error('share payload too large');
    }
    chunks.push(value);
  }
  const out = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    out.set(c, offset);
    offset += c.byteLength;
  }
  return out;
}

/** base64url(deflate-raw(JSON)). */
async function packJson(value: unknown): Promise<string> {
  return bytesToBase64Url(await deflate(new TextEncoder().encode(JSON.stringify(value))));
}

async function unpackJson(token: string): Promise<unknown> {
  return JSON.parse(new TextDecoder().decode(await inflateCapped(base64UrlToBytes(token)))) as unknown;
}

function bad(): never {
  throw new Error('malformed session');
}
const isObject = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v);
const int = (v: unknown, min = 0): number => (typeof v === 'number' && Number.isInteger(v) && v >= min ? v : bad());
const finite = (v: unknown, min = -Infinity): number => (typeof v === 'number' && Number.isFinite(v) && v >= min ? v : bad());

/** The log as compact arrays `[tickDelta, code, …args]` (Decision 5). */
export function encodeLog(log: LogEntry[]): unknown[][] {
  let prev = 0;
  return log.map(({ tick, cmd }) => {
    const dt = tick - prev;
    prev = tick;
    switch (cmd.type) {
      case 'paint':
        return [dt, 'p', cmd.x, cmd.y, cmd.radius, cmd.value, cmd.good];
      case 'importLandscape':
        return [dt, 'i', cmd.good, bytesToBase64Url(cmd.capacities)];
      case 'place':
        return Object.keys(cmd.overrides).length > 0 ? [dt, 'a', cmd.x, cmd.y, cmd.overrides] : [dt, 'a', cmd.x, cmd.y];
      case 'erase':
        return [dt, 'x', cmd.x, cmd.y];
      case 'infect':
        return [dt, 'f', cmd.x, cmd.y, cmd.disease];
      case 'vaccinate':
        return [dt, 'v', cmd.x, cmd.y, cmd.radius, cmd.disease];
      case 'setConfig':
        return [dt, 'c', cmd.config];
    }
  });
}

function overrides(v: unknown): PlaceOverrides {
  if (v === undefined) return {};
  if (!isObject(v)) bad();
  const out: PlaceOverrides = {};
  if (v.sex !== undefined) out.sex = v.sex === 'female' || v.sex === 'male' ? v.sex : bad();
  if (v.tribe !== undefined) out.tribe = v.tribe === 'blue' || v.tribe === 'red' ? v.tribe : bad();
  return out;
}

function decodeCommand(code: unknown, a: unknown[]): EditCommand {
  switch (code) {
    case 'p':
      return { type: 'paint', x: int(a[0]), y: int(a[1]), radius: finite(a[2], 0), value: finite(a[3]), good: int(a[4]) };
    case 'i':
      return { type: 'importLandscape', good: int(a[0]), capacities: typeof a[1] === 'string' ? base64UrlToBytes(a[1]) : bad() };
    case 'a':
      return { type: 'place', x: int(a[0]), y: int(a[1]), overrides: overrides(a[2]) };
    case 'x':
      return { type: 'erase', x: int(a[0]), y: int(a[1]) };
    case 'f':
      return { type: 'infect', x: int(a[0]), y: int(a[1]), disease: int(a[2], -1) };
    case 'v':
      return { type: 'vaccinate', x: int(a[0]), y: int(a[1]), radius: finite(a[2], 0), disease: int(a[3]) };
    case 'c':
      return isObject(a[0]) ? { type: 'setConfig', config: a[0] as unknown as Config } : bad();
    default:
      return bad();
  }
}

/** The log from its compact form; throws on anything malformed (the link is then rejected). */
export function decodeLog(e: unknown): LogEntry[] {
  if (!Array.isArray(e)) bad();
  let tick = 0;
  return e.map((item: unknown) => {
    if (!Array.isArray(item)) bad();
    const [dt, code, ...args] = item as unknown[];
    tick += int(dt);
    return { tick, cmd: decodeCommand(code, args) };
  });
}

function toWire(state: ShareState): Wire {
  const wire: Wire = { v: 3, c: state.config, s: state.seed };
  if (state.landscapes?.some((l) => l !== null)) wire.g = state.landscapes.map((l) => (l ? bytesToBase64Url(l) : null));
  if (state.log && state.log.length > 0) wire.e = encodeLog(state.log);
  return wire;
}

function fromWire(value: unknown): ShareState {
  if (!isObject(value)) bad();
  const { v, c, s, l, g, e } = value;
  if ((v !== 1 && v !== 2 && v !== 3) || typeof s !== 'number' || !isObject(c)) bad();
  // A v1 config is in the pre-N-goods shape; the WASM side converts it.
  const state: ShareState = { config: c as unknown as Config, seed: s >>> 0, log: [] };
  if (v === 1 && typeof l === 'string') state.landscapes = [base64UrlToBytes(l)];
  if (v !== 1 && Array.isArray(g)) state.landscapes = g.map((x) => (typeof x === 'string' ? base64UrlToBytes(x) : null));
  if (v === 3 && e !== undefined) state.log = decodeLog(e);
  return state;
}

/** `#s=` links: base64url(deflate-raw(Wire v3)). */
export async function encodeShare(state: ShareState): Promise<string> {
  return packJson(toWire(state));
}

export async function decodeShare(token: string): Promise<ShareState> {
  try {
    return fromWire(await unpackJson(token));
  } catch {
    throw new Error('not a SugarScape share link');
  }
}

export function readHash(hash: string = location.hash): string | null {
  return /^#s=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}

/** `#c=` links: base64url(deflate-raw({ v: 3, a: Wire, b: Wire })) (Decision 6). */
export async function encodeCompare(state: CompareState): Promise<string> {
  return packJson({ v: 3, a: toWire(state.a), b: toWire(state.b) });
}

export async function decodeCompare(token: string): Promise<CompareState> {
  try {
    const json = await unpackJson(token);
    if (!isObject(json) || json.v !== 3) bad();
    return { a: fromWire(json.a), b: fromWire(json.b) };
  } catch {
    throw new Error('not a SugarScape compare link');
  }
}

export function readCompareHash(hash: string = location.hash): string | null {
  return /^#c=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}

/** A session file: the link's content as plain JSON (Decision 6). */
export function sessionFileText(file: SessionFile): string {
  if (file.kind === 'session') return JSON.stringify({ sugarscape: 'session', ...toWire(file.state) });
  return JSON.stringify({ sugarscape: 'compare', v: 3, a: toWire(file.state.a), b: toWire(file.state.b) });
}

export function parseSessionFile(text: string): SessionFile {
  try {
    const json = JSON.parse(text) as unknown;
    if (isObject(json) && json.sugarscape === 'session') return { kind: 'session', state: fromWire(json) };
    if (isObject(json) && json.sugarscape === 'compare') return { kind: 'compare', state: { a: fromWire(json.a), b: fromWire(json.b) } };
  } catch {
    // Falls through to the error below.
  }
  throw new Error('not a SugarScape session file');
}

/** `#x=` links (Decision 22): base64url(deflate-raw(sweep JSON)). */
export async function encodeSweep(sweep: Sweep): Promise<string> {
  return packJson(sweep);
}

export async function decodeSweep(token: string): Promise<Sweep> {
  try {
    const opened = classifyFile(await unpackJson(token));
    if (opened.kind !== 'sweep') throw new Error(opened.kind);
    return opened.sweep;
  } catch {
    throw new Error('not a SugarScape experiment link');
  }
}

export function readSweepHash(hash: string = location.hash): string | null {
  return /^#x=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/share.test.ts)`
Expected: all pass (the 7 new ones, the 3 adjusted ones, and the rest unchanged, including the legacy link and experiment links).

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds (main.ts still calls `encodeShare({ config, seed, landscapes })` and `Engine.create(await decodeShare(token))` — both type-check, since `ShareState`'s new `log` is optional and `InitialState` accepts the extra property from a variable); all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/share.ts web/src/share.test.ts
git commit -m "Carry the edit log in share links; add compare links and session files" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 3: Sessions in the engine

*Needs judgment (full code given; the point is which rebuilds keep the setup and which start a new log — Decision 4).* Browser (controller): regression only — Reset, 🎲, a preset change and a reset-requiring rule change still rebuild as before (the toolbar does not call `replay()` until Task 5); every existing share link still opens.

**Files:**
- Modify: `web/src/engine.ts`
- Test: `web/src/engine.test.ts`

**Interfaces:**
- Consumes: `LogEntry`, `Session`, `SessionLog`, commands `session`/`endReplay`, snapshot `replayLeft`/`forked` (Task 1).
- Produces (`engine.ts`):
  - `interface InitialState { config: Config; seed: number; landscapes?: (Uint8Array | null)[]; log?: LogEntry[] }`
  - `EngineEvent` gains `'replay'` (after `replayLeft` changed) and `'fork'` (a page edit branched a replay)
  - `engine.replayLeft: number`
  - `session(): Promise<{ session: Session; full: boolean; tick: number }>`
  - `replay(): Promise<FieldError[] | null>` — Reset with the same seed
  - `open(state: InitialState): Promise<FieldError[] | null>` — build a session opened from a file
  - `endReplay(): Promise<void>`

- [ ] **Step 1: Write the failing tests**

In `web/src/engine.test.ts`, change the protocol import to `import type { Command, HostReply, LogEntry, Wants } from './protocol';`, and append:
```ts
describe('Engine sessions', () => {
  const deps = () => ({ presets, transport: new InlineTransport(new SimHost(fakeModule())) });

  it('logs edits with their ticks and returns the session it was built from', async () => {
    const { engine } = await setup();
    await engine.advance(2);
    expect(await engine.place(0, 2, {})).toBeNull();
    expect(await engine.erase(3, 2)).not.toBeNull(); // failed: not logged
    await engine.advance(1);
    expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
    const { session, full, tick } = await engine.session();
    expect(full).toBe(false);
    expect(tick).toBe(3);
    expect(session.config).toEqual(config);
    expect(session.seed).toBe(7);
    expect(session.landscapes).toEqual([]);
    expect(session.log.map((e) => [e.tick, e.cmd.type])).toEqual([
      [2, 'place'],
      [3, 'setConfig'],
    ]);
  });

  it('replays a session on a new engine: counts down and shares back the same log', async () => {
    const first = await setup();
    await first.engine.advance(2);
    await first.engine.place(0, 2, {});
    await first.engine.advance(3);
    await first.engine.paint(0, 0, 1, 3);
    const { session } = await first.engine.session();
    const engine = await Engine.create(session, deps());
    expect(engine.replayLeft).toBe(2);
    const counts: number[] = [];
    engine.on('replay', () => counts.push(engine.replayLeft));
    await engine.advance(2);
    expect(engine.population).toBe(2);
    await engine.advance(3);
    expect(counts).toEqual([1, 0]);
    expect((await engine.session()).session.log).toEqual(session.log);
    expect(await engine.fingerprint()).toBe(await first.engine.fingerprint());
  });

  it('forks on an edit during a replay; endReplay keeps the world', async () => {
    const log: LogEntry[] = [
      { tick: 3, cmd: { type: 'place', x: 0, y: 2, overrides: {} } },
      { tick: 5, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    const a = await Engine.create({ config, seed: 7, log }, deps());
    const events: EngineEvent[] = [];
    a.on('fork', () => events.push('fork'));
    await a.advance(1);
    expect(await a.paint(0, 0, 1, 3)).toBeNull();
    expect(events).toEqual(['fork']);
    expect(a.replayLeft).toBe(0);
    expect((await a.session()).session.log.map((e) => e.cmd.type)).toEqual(['paint']);

    const b = await Engine.create({ config, seed: 7, log }, deps());
    await b.advance(3);
    expect(b.replayLeft).toBe(1);
    await b.endReplay();
    expect(b.replayLeft).toBe(0);
    expect(b.population).toBe(2);
    await b.advance(3);
    expect(b.population).toBe(2);
    expect((await b.session()).session.log).toHaveLength(1);
  });

  it('Reset (replay) rebuilds the session and keeps the setup; a new seed starts an empty log', async () => {
    const { engine, module } = await setup();
    const preset = engine.presetId;
    await engine.advance(2);
    await engine.place(0, 2, {});
    await engine.applyConfig((c) => void (c.population = 20));
    await engine.advance(3);
    expect(await engine.replay()).toBeNull();
    expect(module.sims).toHaveLength(2);
    expect(engine.tick).toBe(0);
    expect(engine.replayLeft).toBe(2);
    expect(engine.baseConfig.population).toBe(20);
    expect(engine.presetId).toBe(preset);
    await engine.advance(2);
    expect(engine.population).toBe(2);
    expect(engine.config.population).toBe(20);
    expect(await engine.reset(undefined, 99)).toBeNull();
    expect(engine.replayLeft).toBe(0);
    const fresh = await engine.session();
    expect(fresh.session.log).toEqual([]);
    expect(fresh.session.seed).toBe(99);
  });

  it('opens a session on the running page', async () => {
    const { engine } = await setup();
    await engine.advance(4);
    const log: LogEntry[] = [{ tick: 0, cmd: { type: 'place', x: 0, y: 2, overrides: {} } }];
    expect(await engine.open({ config, seed: 3, log })).toBeNull();
    expect([engine.tick, engine.seed, engine.population, engine.replayLeft]).toEqual([0, 3, 2, 0]);
    expect((await engine.session()).session.log).toEqual(log);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: FAIL — type errors / `engine.session is not a function` in the new describe.

- [ ] **Step 3: Implement**

In `web/src/engine.ts`:
- add `type LogEntry,` and `type Session,` to the `./protocol` import list (alphabetical: after `type DisplayState,` and after `type SelectQuery,`).
- replace the `EngineEvent` type with:
```ts
export type EngineEvent =
  | 'reset'
  | 'tick'
  | 'config'
  | 'run'
  | 'select'
  | 'display'
  | 'edit'
  | 'follow'
  | 'snapshot'
  | 'crash'
  | 'replay'
  | 'fork';
```
- replace `InitialState` with:
```ts
/** A world to build: its setup, starting maps and (a session's) edits to replay. */
export interface InitialState { config: Config; seed: number; landscapes?: (Uint8Array | null)[]; log?: LogEntry[] }
```
- after the field `crashed: string | null = null;` add:
```ts
  /** Edits still to replay (the toolbar's chip counts them down). */
  replayLeft = 0;
  /** What the world was last built from: a session's config, seed and starting maps (Decision 4). */
  private origin!: { config: Config; seed: number; landscapes: (Uint8Array | null)[] };
  /** `replayLeft` changed in the snapshot being adopted: announce 'replay'. */
  private replayMoved = false;
```
- in `create`, replace from `const landscapes = initial?.landscapes ?? [];` through `engine.adopt(result.snapshot);` with:
```ts
    const landscapes = initial?.landscapes ?? [];
    const log = initial?.log ?? [];
    const result = await engine.send(
      { type: 'init', config, seed: engine.seed, landscapes, display: engine.displayState(), log },
      true,
    );
    if (!result.ok || !result.snapshot) {
      transport.close();
      throw new Error((failure(result) ?? []).map((x) => `${x.field}: ${x.message}`).join('; '));
    }
    engine.origin = { config, seed: engine.seed, landscapes };
    engine.adopt(result.snapshot);
    engine.replayMoved = false;
```
- in `loadPreset`, change the rebuild call to `this.rebuild(structuredClone(preset.config), this.seed, [], { presetId: id })`.
- after `fingerprint()` add:
```ts
  /**
   * The world's session — the config, seed and starting maps it was built from, and its edit log
   * (including edits still to replay) — whether the log overflowed, and the tick (Decision 4).
   */
  session(): Promise<{ session: Session; full: boolean; tick: number }> {
    return this.quiet(async () => {
      const result = await this.send({ type: 'session' });
      if (!result.ok || !result.session) {
        const errors = failure(result) ?? [{ field: 'simulation', message: 'the simulation sent no session' }];
        throw new Error(errors.map((e) => `${e.field}: ${e.message}`).join('; '));
      }
      const { log, full, tick } = result.session;
      const { config, seed, landscapes } = this.origin;
      return { session: { config: structuredClone(config), seed, landscapes, log }, full, tick };
    });
  }

  /**
   * Reset with the same seed: rebuilds the session's world and replays its log from the start,
   * keeping the setup (base config and preset). With a full log it is a plain reset (Decision 4).
   */
  replay(): Promise<FieldError[] | null> {
    return this.quiet(async () => {
      const result = await this.send({ type: 'session' });
      if (!result.ok) return failure(result);
      if (!result.session || result.session.full) {
        return this.rebuild(this.baseConfig, this.seed, this.keptLandscapes(this.baseConfig));
      }
      const { config, seed, landscapes } = this.origin;
      return this.rebuild(config, seed, landscapes, { log: result.session.log, keepSetup: true });
    });
  }

  /** Builds a session opened on this page (a session file) and replays its log. */
  open(state: InitialState): Promise<FieldError[] | null> {
    return this.quiet(() => this.rebuild(state.config, state.seed, state.landscapes ?? [], { log: state.log ?? [] }));
  }

  /** Drops the edits still to replay, keeping the world as it is. */
  async endReplay(): Promise<void> {
    const result = await this.send({ type: 'endReplay' });
    if (result.ok && result.snapshot) this.accept(result.snapshot);
  }
```
- in `adopt`, after `this.latest = s.latest;` add:
```ts
    if (s.replayLeft !== undefined && s.replayLeft !== this.replayLeft) {
      this.replayLeft = s.replayLeft;
      this.replayMoved = true;
    }
```
- replace `announce` with:
```ts
  private announce(s: WorldSnapshot, events: EngineEvent[], clamped: boolean): void {
    if (s.config && !events.includes('reset')) this.emit('config');
    for (const e of events) this.emit(e);
    if (this.replayMoved) {
      this.replayMoved = false;
      this.emit('replay');
    }
    if (s.forked) this.emit('fork');
    if (clamped) this.emit('display');
    this.emit('snapshot');
  }
```
  and extend its doc comment's first sentence to: "Fires `'config'` if the snapshot carries a config (except for a new world, which fires `'reset'` instead), then `events`, then `'replay'` if `replayLeft` moved and `'fork'` if the session branched, then `'display'` if the host clamped the display, then `'snapshot'`."
- replace `rebuild` with:
```ts
  /**
   * Builds a new world. `log` is replayed into it (a session); `keepSetup` keeps the base config
   * and preset (a replay rewinds the same setup rather than choosing a new one — Decision 4).
   */
  private async rebuild(
    config: Config,
    seed: number,
    landscapes: (Uint8Array | null)[],
    opts: { presetId?: string; log?: LogEntry[]; keepSetup?: boolean } = {},
  ): Promise<FieldError[] | null> {
    // Replies to requests sent before this carry the old world's selection; a selection made
    // after it (a click while it is outstanding) is kept.
    const gen = ++this.selectionGen;
    this.resetting++;
    let result: Result;
    try {
      result = await this.send({ type: 'reset', config, seed, landscapes, log: opts.log }, true);
    } finally {
      this.resetting--;
    }
    if (!result.ok || !result.snapshot) return writeFailure(result);
    this.seed = seed;
    this.origin = { config, seed, landscapes };
    // Unless a selection made after the reset was sent has already arrived.
    if (this.selectedUnder < gen) {
      this.selection = null;
      this.inspection = null;
    }
    const clamped = this.adopt(result.snapshot);
    if (!opts.keepSetup) {
      this.baseConfig = structuredClone(this.config);
      this.presetId = opts.presetId ?? this.matchPreset();
    }
    this.announce(result.snapshot, ['reset'], clamped);
    // Panels' wants may have changed with the config (new chart lines, say).
    void this.refresh();
    return null;
  }
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/engine.test.ts)`
Expected: all pass (the 5 new "Engine sessions" tests and every earlier engine test).

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/engine.ts web/src/engine.test.ts
git commit -m "Give the engine sessions: origin, session(), a replaying reset and open()" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 4: Determinism — a recorded session replays exactly

*Mechanical (full code); if a fingerprint differs, stop and report — never weaken the assertion.* Browser: nothing to check.

**Files:**
- Test: `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: `Engine.create/advance/paint/place/erase/infect/vaccinate/applyConfig/setSpeed/setRunning/pump/session/replay/fingerprint`, `replayLeft` (Task 3); `encodeShare`, `decodeShare` (Task 2).
- Produces: nothing.

- [ ] **Step 1: Write the test**

In `web/src/determinism.test.ts`:
- change `import { Engine } from './engine';` to `import { Engine, type Speed } from './engine';` and add `import { decodeShare, encodeShare } from './share';` after the `./transport` import.
- replace the `engine()` helper with:
```ts
const inline = () => new InlineTransport(new SimHost(wasmSimModule(wasm.memory)));

async function engine(): Promise<Engine> {
  return Engine.create({ config: structuredClone(unit.config), seed: 1 }, { presets, transport: inline() });
}
```
- append:
```ts
describe('sessions replay exactly', () => {
  const endemic = presets.find((p) => p.id === 'v-2-endemic')!;
  const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
  const create = (seed: number) => Engine.create({ config: structuredClone(endemic.config), seed }, { presets, transport: inline() });

  /** Runs `e` at `speed` for a few animation frames (Max: for `ms`), then pauses it and waits until it is quiet. */
  async function run(e: Engine, speed: Speed, frames = 4, ms = 40): Promise<void> {
    e.setSpeed(speed);
    e.setRunning(true);
    if (speed === 'max') await wait(ms);
    else
      for (let i = 0; i < frames; i++) {
        e.pump();
        await wait(0);
      }
    e.setRunning(false);
    // Waits for the Max stop (or the frame's step) to be answered.
    await e.session();
  }

  /** Steps `e` to `tick` in uneven requests. */
  async function reach(e: Engine, tick: number): Promise<void> {
    for (const n of [1, 7, 100, 3]) if (e.tick < tick) await e.advance(Math.min(n, tick - e.tick));
    while (e.tick < tick) await e.advance(Math.min(250, tick - e.tick));
  }

  it('replays a session recorded at mixed speeds, through a share link, to the same world at the same tick', async () => {
    const live = await create(5);
    expect(await live.paint(10, 10, 2, 0, 0)).toBeNull(); // tick 0
    await run(live, 1);
    await live.place(3, 3, {}); // may be occupied: then there is an agent to infect anyway
    expect(await live.infect(3, 3, -1)).toBeNull();
    await run(live, 25);
    await live.erase(3, 3);
    await live.place(0, 0, { sex: 'female' });
    expect(await live.applyConfig((c) => void (c.growback.rate = 2))).toBeNull();
    await run(live, 'max');
    await live.vaccinate(20, 20, 3, 0);
    // Edits while Max runs land between batches.
    live.setSpeed('max');
    live.setRunning(true);
    await wait(20);
    await live.place(1, 1, {});
    await live.paint(30, 30, 1, 4, 0);
    await wait(20);
    live.setRunning(false);
    await run(live, 100, 2);
    const { session, full, tick } = await live.session();
    expect(full).toBe(false);
    // Places and erases may meet an occupied or empty site (then they are not logged); these always land:
    // the paint at 0, the infection, the live change and the paint during Max, at four different ticks.
    expect(session.log.length).toBeGreaterThanOrEqual(4);
    expect(new Set(session.log.map((e) => e.tick)).size).toBeGreaterThanOrEqual(4);
    const expected = await live.fingerprint();

    const replayed = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await reach(replayed, tick);
    expect(replayed.tick).toBe(tick);
    expect(replayed.replayLeft).toBe(0);
    expect(await replayed.fingerprint()).toBe(expected);
    expect((await replayed.session()).session.log).toEqual(session.log);

    // Reset with the same seed replays it again on the same engine.
    expect(await live.replay()).toBeNull();
    expect(live.tick).toBe(0);
    await reach(live, tick);
    expect(await live.fingerprint()).toBe(expected);
  });

  it('replays at Max to the same world as the live run', async () => {
    const live = await create(8);
    await live.advance(2);
    await live.place(4, 4, {});
    await live.infect(4, 4, -1);
    await live.advance(3);
    await live.paint(12, 30, 3, 1, 0);
    await live.applyConfig((c) => void (c.growback.rate = 3));
    await live.advance(4);
    const { session } = await live.session();
    const replayed = await Engine.create(session, { presets, transport: inline() });
    await run(replayed, 'max', 0, 60);
    // Bring whichever is behind up to the other: after the log ends, both just step.
    const at = Math.max(replayed.tick, live.tick);
    await reach(replayed, at);
    await reach(live, at);
    expect(replayed.replayLeft).toBe(0);
    expect(await replayed.fingerprint()).toBe(await live.fingerprint());
  });
});
```

- [ ] **Step 2: Run the test**

Run: `(cd web && npm run build && npx vitest run src/determinism.test.ts)`
Expected: `5 passed` (the three golden tests and the two new ones). A fingerprint mismatch means replay is not exact: stop and report which assertion failed.

- [ ] **Step 3: Full verification**

Run: `(cd web && npm test)`
Expected: all test files pass.

- [ ] **Step 4: Commit**

```bash
git add web/src/determinism.test.ts
git commit -m "Prove sessions replay exactly through a share link, at mixed speeds and at Max" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 5: Replay in the page — chip, fork notice, replaying Reset, Share menu and session files

*Needs judgment (full code given; check the Share flow and notices read well).* Browser (controller), `/?debug`:
- On `ii-2-unit`: paint (drag), place, erase, a live rule change (e.g. growback rate), each at a different tick while playing at 5× → Share → **Copy link** → open the copied URL in a new tab: the toolbar shows "Replaying · N edits left ✕" and counts down while playing; once it disappears, pause and step both tabs to the same tick: `await window.sugarscape.engine.fingerprint()` is equal in both.
- In the replaying tab, paint before the chip reaches 0: notice "Replay ended — your edit starts a new branch", the chip disappears. Reload and instead click the chip's ✕: the chip disappears, the world stays as it is.
- Reset with the seed box unchanged: the world rewinds to t = 0 and the chip reappears with the full count; type another seed + Reset, 🎲, a preset change, a reset-requiring change (e.g. Width): no chip.
- Export → **Session (JSON)** downloads `sugarscape-…-session.json`; Share → **Open session…** with it rebuilds that session (chip appears, the address bar hash clears); an unrelated JSON file gives a notice "… could not be opened".
- Long link: in the console `const e = window.sugarscape.engine; for (let i = 0; i < 20000; i++) await e.paint(i % 50, (i * 7) % 50, i % 4, i % 11, 0)` → Copy link: the link is copied and the notice about a long link shows.
- Old links: a link from `main`'s build (v2) and `#s=<LEGACY_SHARE_TOKEN>` open with their painted maps and no chip; every existing Share/Export scenario still works (Export menu items unchanged in single mode).

**Files:**
- Create: `web/src/ui/notice.ts`, `web/src/sessions.ts`, `web/src/sessions.test.ts`, `web/src/ui/share-menu.ts`, `web/src/ui/export-menu.ts`
- Modify: `web/src/ui/toolbar.ts`, `web/src/main.ts`, `web/index.html`, `web/src/style.css`

**Interfaces:**
- Consumes: `Engine.session/replay/open/endReplay/replayLeft`, events `'replay'`/`'fork'` (Task 3); `encodeShare`, `encodeCompare`, `sessionFileText`, `parseSessionFile`, `ShareState` (Task 2); `Session`, `LogEntry` (Task 1).
- Produces:
  - `ui/notice.ts`: `showNotice(message: string, ms?: number): void`
  - `sessions.ts`: `LONG_LINK = 32_000`, `LONG_NOTICE`, `LOG_FULL_NOTICE`; `interface SessionSource { baseConfig: Config; seed: number; editedLandscapes(): (Uint8Array | null)[] | undefined; session(): Promise<{ session: Session; full: boolean; tick: number }> }` (an `Engine` is one); `shareable(source): Promise<{ state: ShareState; full: boolean }>`; `sessionLink(source): Promise<{ hash: string; notice?: string }>`; `compareLink(a, b): Promise<{ hash: string; notice?: string }>`
  - `ui/share-menu.ts`: `buildShareMenu(opts: { link: () => Promise<{ hash: string; notice?: string }>; open: (file: File) => Promise<void> }): HTMLElement`
  - `ui/export-menu.ts`: `interface ExportWorld { label: string; engine: Engine; grid: GridView }`; `buildExportMenu(opts: { worlds: () => ExportWorld[]; slug: (world: ExportWorld) => string; charts: () => Promise<void>; session: () => Promise<void> }): HTMLElement`
  - `ui/toolbar.ts`: `interface Chip { el: HTMLElement; off: () => void }`; `followChip(engine: Engine): Chip`; `replayChip(engine: Engine): Chip`; `buildToolbar(engine: Engine): HTMLElement` (Reset replays when the seed is unchanged)

- [ ] **Step 1: Write the failing tests**

Create `web/src/sessions.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import type { LogEntry, Session } from './protocol';
import { compareLink, LOG_FULL_NOTICE, LONG_LINK, LONG_NOTICE, sessionLink, shareable, type SessionSource } from './sessions';
import { decodeCompare, decodeShare } from './share';
import type { Config } from './types';

const config = { width: 50, height: 50 } as unknown as Config;
const painted = [new Uint8Array(2500).fill(2)];

function source(log: LogEntry[], full = false, seed = 3): SessionSource {
  const session: Session = { config, seed, landscapes: [], log };
  return {
    baseConfig: { ...config, population: 99 } as Config,
    seed,
    editedLandscapes: () => painted,
    session: async () => ({ session, full, tick: 10 }),
  };
}

/** Edits that deflate poorly: a pseudo-random paint trail (MINSTD). */
function noisy(n: number): LogEntry[] {
  let x = 1;
  const next = () => (x = (x * 48271) % 2147483647) % 50;
  return Array.from({ length: n }, (_, i): LogEntry => ({
    tick: i,
    cmd: { type: 'paint', x: next(), y: next(), radius: next() % 5, value: next() % 11, good: 0 },
  }));
}

describe('sessionLink', () => {
  it('links the whole session', async () => {
    const log: LogEntry[] = [{ tick: 2, cmd: { type: 'erase', x: 1, y: 1 } }];
    const { hash, notice } = await sessionLink(source(log));
    expect(notice).toBeUndefined();
    expect(hash).toMatch(/^#s=[A-Za-z0-9_-]+$/);
    expect(await decodeShare(hash.slice(3))).toEqual({ config, seed: 3, log });
  });

  it('still links a long session, and says the link is long', async () => {
    const { hash, notice } = await sessionLink(source(noisy(20_000)));
    expect(hash.length - 3).toBeGreaterThan(LONG_LINK);
    expect(notice).toBe(LONG_NOTICE);
    expect((await decodeShare(hash.slice(3))).log).toHaveLength(20_000);
  });

  it('falls back to the setup and painted maps when the log is full', async () => {
    const { state, full } = await shareable(source(noisy(10), true));
    expect(full).toBe(true);
    expect(state.log).toBeUndefined();
    const { hash, notice } = await sessionLink(source(noisy(10), true));
    expect(notice).toBe(LOG_FULL_NOTICE);
    const back = await decodeShare(hash.slice(3));
    expect(back.log).toEqual([]);
    expect(back.config.population).toBe(99);
    expect(back.landscapes).toEqual(painted);
  });
});

describe('compareLink', () => {
  it('links both sessions', async () => {
    const { hash, notice } = await compareLink(source([], false, 1), source([{ tick: 4, cmd: { type: 'erase', x: 0, y: 0 } }], false, 2));
    expect(notice).toBeUndefined();
    expect(hash).toMatch(/^#c=[A-Za-z0-9_-]+$/);
    const back = await decodeCompare(hash.slice(3));
    expect([back.a.seed, back.b.seed]).toEqual([1, 2]);
    expect(back.b.log).toHaveLength(1);
  });

  it('says when either log is full', async () => {
    expect((await compareLink(source([]), source([], true))).notice).toBe(LOG_FULL_NOTICE);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/sessions.test.ts)`
Expected: FAIL — `Failed to resolve import "./sessions"`.

- [ ] **Step 3: Sessions and the notice strip**

Create `web/src/sessions.ts`:
```ts
import type { Session } from './protocol';
import { encodeCompare, encodeShare, type ShareState } from './share';
import type { Config } from './types';

/** Links longer than this still work in browsers, but some apps cut them (Decision 6). */
export const LONG_LINK = 32_000;
export const LONG_NOTICE =
  'This link is long, and some apps cut long links. Export → Session (JSON) saves the same session as a file.';
export const LOG_FULL_NOTICE =
  'The edit log is full (50 000 edits), so this has the setup and painted maps only, not the edits.';

/** What a link or session file is made from; an `Engine` is one. */
export interface SessionSource {
  baseConfig: Config;
  seed: number;
  editedLandscapes(): (Uint8Array | null)[] | undefined;
  session(): Promise<{ session: Session; full: boolean; tick: number }>;
}

/** The whole session, or — when the log overflowed — the setup and painted maps (today's link). */
export async function shareable(source: SessionSource): Promise<{ state: ShareState; full: boolean }> {
  const { session, full } = await source.session();
  if (!full) return { state: session, full };
  return { state: { config: source.baseConfig, seed: source.seed, landscapes: source.editedLandscapes() }, full };
}

function notice(token: string, full: boolean): string | undefined {
  if (full) return LOG_FULL_NOTICE;
  return token.length > LONG_LINK ? LONG_NOTICE : undefined;
}

/** `#s=` for one world's session, and a notice when the link is long or the log was full. */
export async function sessionLink(source: SessionSource): Promise<{ hash: string; notice?: string }> {
  const { state, full } = await shareable(source);
  const token = await encodeShare(state);
  return { hash: `#s=${token}`, notice: notice(token, full) };
}

/** `#c=` for a comparison: both sessions, opening straight into Compare. */
export async function compareLink(a: SessionSource, b: SessionSource): Promise<{ hash: string; notice?: string }> {
  const sa = await shareable(a);
  const sb = await shareable(b);
  const token = await encodeCompare({ a: sa.state, b: sb.state });
  return { hash: `#c=${token}`, notice: notice(token, sa.full || sb.full) };
}
```

Create `web/src/ui/notice.ts`:
```ts
import { h } from './dom';

let timer: ReturnType<typeof setTimeout> | undefined;

/** Shows a short, non-error message in the status strip (#notice) for `ms` milliseconds (Decision 7). */
export function showNotice(message: string, ms = 5000): void {
  const el = document.querySelector<HTMLElement>('#notice');
  if (!el) return;
  el.replaceChildren(
    h('span', {}, message),
    h('button', { class: 'link', 'aria-label': 'Dismiss', onclick: () => (el.hidden = true) }, '×'),
  );
  el.hidden = false;
  clearTimeout(timer);
  timer = setTimeout(() => (el.hidden = true), ms);
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/sessions.test.ts)`
Expected: `5 passed`.

- [ ] **Step 5: The Share and Export menus**

Create `web/src/ui/share-menu.ts`:
```ts
import { h } from './dom';
import { showNotice } from './notice';

/**
 * Share: Copy link (the whole session, or a comparison) and Open session… (a session file).
 * A long link is still copied, with a notice (Decision 6).
 */
export function buildShareMenu(opts: {
  link: () => Promise<{ hash: string; notice?: string }>;
  open: (file: File) => Promise<void>;
}): HTMLElement {
  const copy = h(
    'button',
    { title: 'Copy a link that replays this session: the setup, the painted maps and every edit with its tick' },
    'Copy link',
  );
  copy.addEventListener('click', async () => {
    const { hash, notice } = await opts.link();
    history.replaceState(null, '', hash);
    try {
      await navigator.clipboard.writeText(location.href);
      copy.textContent = 'Link copied';
    } catch {
      copy.textContent = 'Link in address bar';
    }
    if (notice) showNotice(notice, 10_000);
    setTimeout(() => (copy.textContent = 'Copy link'), 2000);
  });
  const file = h('input', { type: 'file', accept: '.json,application/json', hidden: true, 'aria-label': 'Session file to open' });
  file.addEventListener('change', async () => {
    const chosen = file.files?.[0];
    file.value = '';
    if (chosen) await opts.open(chosen);
  });
  const open = h(
    'button',
    { title: 'Open a session file saved with Export → Session (JSON)', onclick: () => file.click() },
    'Open session…',
  );
  return h('details', { class: 'menu' }, h('summary', {}, 'Share'), h('div', { class: 'menu-items' }, copy, open, file));
}
```

Create `web/src/ui/export-menu.ts`:
```ts
import { downloadBlob, downloadText } from '../downloads';
import type { Engine } from '../engine';
import { h } from './dom';
import type { GridView } from './grid-view';

/** A world the Export menu exports from; `label` ('A', 'B') is empty outside Compare. */
export interface ExportWorld { label: string; engine: Engine; grid: GridView }

export interface ExportOptions {
  worlds: () => ExportWorld[];
  /** A world's file-name stem. */
  slug: (world: ExportWorld) => string;
  /** Charts (PNG): the charts as drawn. */
  charts: () => Promise<void>;
  /** Session (JSON): the session (or comparison) as a file. */
  session: () => Promise<void>;
}

/**
 * The Export menu. Per-world exports show one button, or in Compare a row with one button per
 * world ("Statistics (CSV) [A] [B]", Decision 12); the menu is rebuilt each time it opens.
 */
export function buildExportMenu(opts: ExportOptions): HTMLElement {
  const items = h('div', { class: 'menu-items' });
  const menu = h('details', { class: 'menu' }, h('summary', {}, 'Export'), items);
  const perWorld = (name: string, run: (world: ExportWorld) => Promise<void>): HTMLElement => {
    const worlds = opts.worlds();
    if (worlds.length === 1) return h('button', { onclick: () => void run(worlds[0]) }, name);
    return h(
      'div',
      { class: 'menu-row' },
      h('span', {}, name),
      ...worlds.map((w) => h('button', { title: `${name} of world ${w.label}`, onclick: () => void run(w) }, w.label)),
    );
  };
  const fill = (): void => {
    items.replaceChildren(
      perWorld('Statistics (CSV)', async (w) => downloadText(`${opts.slug(w)}-series.csv`, await w.engine.seriesCsv())),
      perWorld('Agents (CSV)', async (w) => downloadText(`${opts.slug(w)}-agents.csv`, await w.engine.agentsCsv())),
      perWorld('Grid (PNG)', async (w) => downloadBlob(`${opts.slug(w)}-grid.png`, await w.grid.toPngBlob())),
      h('button', { onclick: () => void opts.charts() }, 'Charts (PNG)'),
      h(
        'button',
        {
          title: 'The whole session — setup, painted maps and every edit — as a file; Share → Open session… loads it',
          onclick: () => void opts.session(),
        },
        'Session (JSON)',
      ),
    );
  };
  menu.addEventListener('toggle', () => {
    if (menu.open) fill();
  });
  fill();
  return menu;
}
```

- [ ] **Step 6: The toolbar's chips and a replaying Reset**

Replace `web/src/ui/toolbar.ts` with:
```ts
import { randomSeed, type Engine, type Speed } from '../engine';
import { h } from './dom';

const SPEEDS: Speed[] = [1, 2, 5, 10, 25, 100, 'max'];

/** A chip and the removal of its listeners. */
export interface Chip { el: HTMLElement; off: () => void }

/** "Following #id ✕" while `engine` draws an agent's trail; † once it has died. */
export function followChip(engine: Engine): Chip {
  const el = h('span', { class: 'chip' });
  const sync = () => {
    const id = engine.followed();
    el.hidden = id === null;
    if (id === null) return;
    const alive = engine.followedAlive();
    const text = `Following #${id}${alive ? '' : ' †'}`;
    el.title = text;
    el.replaceChildren(
      h('span', { class: 'chip-text' }, text),
      h('button', { class: 'link', title: 'Stop following', 'aria-label': 'Stop following', onclick: () => engine.unfollow() }, '✕'),
    );
  };
  const offs = (['follow', 'reset', 'tick', 'edit'] as const).map((event) => engine.on(event, sync));
  sync();
  return { el, off: () => offs.forEach((off) => off()) };
}

/** "Replaying · N edits left ✕" while `engine` has edits to replay; ✕ keeps the world and drops the rest. */
export function replayChip(engine: Engine): Chip {
  const el = h('span', { class: 'chip replay-chip' });
  const sync = () => {
    const left = engine.replayLeft;
    el.hidden = left === 0;
    if (left === 0) return;
    const text = `Replaying · ${left} edit${left === 1 ? '' : 's'} left`;
    el.title = text;
    el.replaceChildren(
      h('span', { class: 'chip-text' }, text),
      h(
        'button',
        { class: 'link', title: 'Stop replaying (keep the world as it is)', 'aria-label': 'Stop replaying', onclick: () => void engine.endReplay() },
        '✕',
      ),
    );
  };
  const offs = (['replay', 'reset'] as const).map((event) => engine.on(event, sync));
  sync();
  return { el, off: () => offs.forEach((off) => off()) };
}

export function buildToolbar(engine: Engine): HTMLElement {
  const play = h('button', { class: 'primary', onclick: () => engine.setRunning(!engine.running) });
  const step = h('button', { onclick: () => void engine.advance(1), title: 'Advance one tick' }, 'Step');
  const speed = h(
    'select',
    {
      title: 'Ticks per frame; Max runs the simulation as fast as it goes and redraws about 30 times a second',
      onchange: () => engine.setSpeed(speed.value === 'max' ? 'max' : Number(speed.value)),
    },
    ...SPEEDS.map((s) => h('option', { value: String(s) }, s === 'max' ? 'Max' : `${s}×`)),
  );
  const seed = h('input', { type: 'number', min: 0, max: 4294967295, class: 'seed', title: 'Seed' });
  const reset = h(
    'button',
    {
      title: 'Rebuild this world and replay its edits; with another seed typed, build a new world',
      onclick: () => {
        const s = Number(seed.value) >>> 0;
        // The same seed rewinds and replays the session; another seed builds a new world (Decision 4).
        void (s === engine.seed ? engine.replay() : engine.reset(undefined, s));
      },
    },
    'Reset',
  );
  const dice = h(
    'button',
    { title: 'Random seed and reset', onclick: () => void engine.reset(undefined, randomSeed()) },
    '🎲',
  );
  const readout = h('span', { class: 'readout' });

  const sync = () => {
    play.textContent = engine.running ? 'Pause' : 'Play';
    step.disabled = engine.running;
    seed.value = String(engine.seed);
  };
  const tick = () => {
    readout.textContent = `t = ${engine.tick} · ${engine.population} agents`;
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
    followChip(engine).el,
    replayChip(engine).el,
    h('div', { class: 'toolbar-end' }),
  );
}
```

- [ ] **Step 7: Wire it into the page**

In `web/index.html`, add after `<div id="banner" class="banner" hidden></div>`:
```html
    <div id="notice" class="notice" role="status" hidden></div>
```

Append to `web/src/style.css`:
```css
.notice { position: fixed; left: 50%; bottom: 16px; transform: translateX(-50%); z-index: 20; display: flex; gap: 10px; align-items: center; max-width: min(92vw, 44em); padding: 8px 12px; background: var(--surface); color: var(--text); border: 1px solid var(--border); border-radius: 8px; box-shadow: 0 6px 20px rgb(0 0 0 / 0.18); }
.notice button { flex-shrink: 0; }
.replay-chip { max-width: 17em; }
.menu-row { display: flex; gap: 6px; align-items: center; }
.menu-row span { flex: 1; }
```

Replace `web/src/main.ts` with:
```ts
import './style.css';
import { canvasBlob, downloadBlob, downloadText } from './downloads';
import { Engine } from './engine';
import { ExperimentsView } from './experiments/view';
import { LOG_FULL_NOTICE, sessionLink, shareable } from './sessions';
import { decodeShare, decodeSweep, parseSessionFile, readHash, readSweepHash, sessionFileText } from './share';
import { ChartsPanel } from './ui/charts-panel';
import { CreditPanel } from './ui/credit-panel';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { buildExportMenu } from './ui/export-menu';
import { GridView } from './ui/grid-view';
import { InspectPanel } from './ui/inspect-panel';
import { showNotice } from './ui/notice';
import { RulesPanel } from './ui/rules-panel';
import { buildShareMenu } from './ui/share-menu';
import { Tabs } from './ui/tabs';
import { buildToolbar } from './ui/toolbar';
import { buildTools } from './ui/tools';

export function showBanner(message: string, action?: { label: string; run: () => void }): void {
  const banner = document.querySelector<HTMLElement>('#banner')!;
  banner.replaceChildren(
    ...[
      h('span', {}, message),
      action ? h('button', { onclick: action.run }, action.label) : null,
      h('button', { onclick: () => (banner.hidden = true), 'aria-label': 'Dismiss' }, '×'),
    ].filter((child): child is HTMLElement => child !== null),
  );
  banner.hidden = false;
}

const message = (e: unknown): string => (e instanceof Error ? e.message : String(e));

async function main(): Promise<void> {
  let engine: Engine;
  const token = readHash();
  try {
    // A link's session replays its edits as the world runs (Decision 2).
    engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
  } catch (e) {
    showBanner(`That share link could not be loaded (${message(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
  // Browser checks drive the engine through this handle (7a Decision 14).
  if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { sugarscape: { engine } });
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  document.querySelector('#toolbar')!.append(buildToolbar(engine));
  document.querySelector('#display')!.append(buildDisplay(engine));
  const experiments = new ExperimentsView(engine);
  document.querySelector('#experiments')!.append(experiments.el);
  const views = { playground: 'Playground', experiments: 'Experiments' } as const;
  type View = keyof typeof views;
  const viewButtons = (Object.keys(views) as View[]).map((view) =>
    h('button', { 'data-view': view, onclick: () => showView(view) }, views[view]),
  );
  const showView = (view: View): void => {
    // The playground's world is kept, paused, while Experiments is shown.
    if (view === 'experiments') engine.setRunning(false);
    document.body.dataset.view = view;
    document.querySelector<HTMLElement>('#playground')!.hidden = view !== 'playground';
    document.querySelector<HTMLElement>('#experiments')!.hidden = view !== 'experiments';
    for (const b of viewButtons) b.setAttribute('aria-pressed', String(b.dataset.view === view));
  };
  document
    .querySelector('.toolbar h1')!
    .after(h('div', { class: 'view-switch', role: 'group', 'aria-label': 'View' }, ...viewButtons));
  showView('playground');

  const sweepToken = readSweepHash();
  if (sweepToken) {
    try {
      experiments.openSweep(await decodeSweep(sweepToken));
      showView('experiments');
    } catch (e) {
      showBanner(`That experiment link could not be loaded (${message(e)}).`);
    }
  }

  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  tabs.add('Rules', new RulesPanel(engine).el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new InspectPanel(engine);
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  const credit = new CreditPanel(engine, () => tabs.show('Inspect'));
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on.
  const syncCreditTab = () => tabs.setHidden('Credit', !engine.config.credit.enabled);
  engine.on('reset', syncCreditTab);
  engine.on('config', syncCreditTab);
  syncCreditTab();
  document.querySelector('#tools')!.append(buildTools(engine, grid, () => tabs.show('Inspect')));

  const slug = () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}-t${engine.tick}`;
  const exportMenu = buildExportMenu({
    worlds: () => [{ label: '', engine, grid }],
    slug: () => slug(),
    charts: async () => {
      tabs.show('Charts');
      await engine.refresh();
      // Let the panel draw the fresh snapshot before the canvases are captured.
      await new Promise((resolve) => requestAnimationFrame(resolve));
      for (const { name, canvas } of charts.canvases()) {
        downloadBlob(`${slug()}-${name.toLowerCase().replace(/\W+/g, '-')}.png`, await canvasBlob(canvas));
      }
    },
    session: async () => {
      const { state, full } = await shareable(engine);
      if (full) showNotice(LOG_FULL_NOTICE, 10_000);
      downloadText(`${slug()}-session.json`, sessionFileText({ kind: 'session', state }), 'application/json');
    },
  });
  const shareMenu = buildShareMenu({
    link: () => sessionLink(engine),
    open: async (file) => {
      try {
        const opened = parseSessionFile(await file.text());
        if (opened.kind !== 'session') throw new Error('it holds a comparison, which this page cannot open yet');
        const errors = await engine.open(opened.state);
        if (errors) throw new Error(errors.map((x) => `${x.field}: ${x.message}`).join('; '));
        // The address bar no longer describes this world.
        history.replaceState(null, '', location.pathname + location.search);
        showNotice(`Opened ${file.name}`);
      } catch (e) {
        showNotice(`${file.name} could not be opened (${message(e)})`, 10_000);
      }
    },
  });
  document.querySelector('.toolbar-end')!.append(shareMenu, exportMenu);

  engine.on('crash', () => showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() }));
  engine.on('fork', () => showNotice('Replay ended — your edit starts a new branch'));
  let dirty = true;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  const loop = (now: number) => {
    try {
      engine.pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
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
}

main().catch((e) => showBanner(`Failed to start: ${message(e)}`));
```

- [ ] **Step 8: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.
Run: `grep -n "encodeShare\|'Share')" web/src/main.ts`
Expected: no output (links go through `sessionLink`; Share is a menu).

- [ ] **Step 9: Commit**

```bash
git add web/src/sessions.ts web/src/sessions.test.ts web/src/ui/notice.ts web/src/ui/share-menu.ts web/src/ui/export-menu.ts web/src/ui/toolbar.ts web/src/main.ts web/index.html web/src/style.css
git commit -m "Replay sessions in the page: chip, fork notice, replaying Reset, session links and files" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 6: Recording — frame layout, codec choice, names and timing

*Mechanical (full code).* Browser: nothing to check (not used yet).

**Files:**
- Create: `web/src/recording/frames.ts`
- Test: `web/src/recording/frames.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces (`recording/frames.ts`):
  - constants `MIN_CELL_PX = 8`, `MAX_SIDE_PX = 1080`, `GAP_PX = 4`, `GIF_FPS = 15`, `GIF_MAX_FRAMES = 900`, `GIF_BACKLOG = 3`, `MIMES` (the spec's four, in order)
  - `interface Rect { x: number; y: number; width: number; height: number }`, `interface FrameLayout { width: number; height: number; scale: number; rects: Rect[] }`
  - `frameLayout(grids: { width: number; height: number }[]): FrameLayout`
  - `pickMime(supported: (type: string) => boolean): { mime: string; ext: 'webm' | 'mp4' } | null`
  - `recordingName(base: string, from: number, to: number, ext: string): string`
  - `formatClock(ms: number): string` (`m:ss`)
  - `class Stopwatch { constructor(now: number); pause(now: number): void; resume(now: number): void; elapsed(now: number): number }`
  - `class GifSampler { frames: number; readonly full: boolean; due(t: number, backlog: number): boolean; take(t: number): number | null }`
  - `gifDelay(ms: number): number`

- [ ] **Step 1: Write the failing tests**

Create `web/src/recording/frames.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import {
  formatClock,
  frameLayout,
  GIF_BACKLOG,
  GIF_MAX_FRAMES,
  gifDelay,
  GifSampler,
  pickMime,
  recordingName,
  Stopwatch,
} from './frames';

describe('frameLayout', () => {
  it('makes a cell 8 px when the frame fits in 1080 px', () => {
    expect(frameLayout([{ width: 50, height: 50 }])).toEqual({
      width: 400,
      height: 400,
      scale: 8,
      rects: [{ x: 0, y: 0, width: 400, height: 400 }],
    });
  });

  it('shrinks the cell so the long side stays within 1080 px', () => {
    expect(frameLayout([{ width: 200, height: 200 }])).toMatchObject({ width: 1000, height: 1000, scale: 5 });
    expect(frameLayout([{ width: 150, height: 40 }])).toMatchObject({ width: 1050, height: 280, scale: 7 });
  });

  it('puts two grids side by side, top-aligned, with a 4 px gap', () => {
    expect(frameLayout([{ width: 50, height: 50 }, { width: 50, height: 50 }])).toEqual({
      width: 804,
      height: 400,
      scale: 8,
      rects: [
        { x: 0, y: 0, width: 400, height: 400 },
        { x: 404, y: 0, width: 400, height: 400 },
      ],
    });
    expect(frameLayout([{ width: 200, height: 200 }, { width: 100, height: 150 }])).toEqual({
      width: 904,
      height: 600,
      scale: 3,
      rects: [
        { x: 0, y: 0, width: 600, height: 600 },
        { x: 604, y: 0, width: 300, height: 450 },
      ],
    });
  });

  it('never goes below one pixel per cell', () => {
    expect(frameLayout([{ width: 2000, height: 10 }]).scale).toBe(1);
  });
});

describe('pickMime', () => {
  it('takes the first supported type in the spec’s order, with a matching extension', () => {
    expect(pickMime(() => true)).toEqual({ mime: 'video/webm;codecs=vp9', ext: 'webm' });
    expect(pickMime((t) => t === 'video/webm;codecs=vp8' || t === 'video/mp4')).toEqual({ mime: 'video/webm;codecs=vp8', ext: 'webm' });
    expect(pickMime((t) => t === 'video/webm')).toEqual({ mime: 'video/webm', ext: 'webm' });
    expect(pickMime((t) => t === 'video/mp4')).toEqual({ mime: 'video/mp4', ext: 'mp4' });
    expect(pickMime(() => false)).toBeNull();
  });
});

describe('names and clock', () => {
  it('names a recording by its first and last tick', () => {
    expect(recordingName('sugarscape-ii-2-unit-seed7', 10, 250, 'webm')).toBe('sugarscape-ii-2-unit-seed7-t10-t250.webm');
  });

  it('formats elapsed time as m:ss', () => {
    expect(formatClock(0)).toBe('0:00');
    expect(formatClock(61_500)).toBe('1:01');
    expect(formatClock(600_000)).toBe('10:00');
  });

  it('counts recorded time, not pauses', () => {
    const w = new Stopwatch(1000);
    expect(w.elapsed(1500)).toBe(500);
    w.pause(3000);
    w.pause(3500); // already paused
    expect(w.elapsed(5000)).toBe(2000);
    w.resume(6000);
    w.resume(6500); // already running
    expect(w.elapsed(7000)).toBe(3000);
  });
});

describe('GifSampler', () => {
  it('takes about 15 frames a second from 60 Hz snapshots; each delay is the time to the next frame', () => {
    const s = new GifSampler();
    const delays: number[] = [];
    for (let k = 0; k <= 60; k++) {
      const t = (k * 1000) / 60;
      if (!s.due(t, 0)) continue;
      const d = s.take(t);
      if (d !== null) delays.push(d);
    }
    expect(s.frames).toBe(16);
    expect(delays).toHaveLength(15);
    for (const d of delays) expect(gifDelay(d)).toBe(70);
  });

  it('holds back while the encoder is behind, and stops at the frame cap', () => {
    const s = new GifSampler();
    expect(s.due(0, GIF_BACKLOG)).toBe(false);
    expect(s.due(0, GIF_BACKLOG - 1)).toBe(true);
    for (let i = 0; i < GIF_MAX_FRAMES; i++) s.take(i * 100);
    expect(s.full).toBe(true);
    expect(s.due(1e9, 0)).toBe(false);
  });

  it('rounds delays to the 10 ms a GIF stores, at least 20 ms', () => {
    expect(gifDelay(66.7)).toBe(70);
    expect(gifDelay(134)).toBe(130);
    expect(gifDelay(5)).toBe(20);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/recording/frames.test.ts)`
Expected: FAIL — `Failed to resolve import "./frames"`.

- [ ] **Step 3: Implement**

Create `web/src/recording/frames.ts`:
```ts
// The pure parts of recording (Decisions 13 and 14): frame layout, codec choice, names and timing.

/** A recorded cell is this many pixels… */
export const MIN_CELL_PX = 8;
/** …unless that would make the frame's long side longer than this. */
export const MAX_SIDE_PX = 1080;
/** Pixels between the two grids in Compare. */
export const GAP_PX = 4;
/** GIF frames per second of recorded time (about). */
export const GIF_FPS = 15;
/** A GIF stops recording at this many frames. */
export const GIF_MAX_FRAMES = 900;
/** Frames the GIF worker may hold unacknowledged before capture skips frames. */
export const GIF_BACKLOG = 3;
/** Video formats in order of preference (the spec's). */
export const MIMES = ['video/webm;codecs=vp9', 'video/webm;codecs=vp8', 'video/webm', 'video/mp4'] as const;

export interface Rect { x: number; y: number; width: number; height: number }
export interface FrameLayout { width: number; height: number; scale: number; rects: Rect[] }

/**
 * Where each grid goes in a recorded frame: side by side, top-aligned, `scale` pixels per cell —
 * 8, or less so the long side stays within 1080 px, but at least 1.
 */
export function frameLayout(grids: { width: number; height: number }[]): FrameLayout {
  const gaps = GAP_PX * Math.max(0, grids.length - 1);
  const wide = Math.max(1, grids.reduce((sum, g) => sum + g.width, 0));
  const high = Math.max(1, ...grids.map((g) => g.height));
  const fit = Math.min(Math.floor((MAX_SIDE_PX - gaps) / wide), Math.floor(MAX_SIDE_PX / high));
  const scale = Math.max(1, Math.min(MIN_CELL_PX, fit));
  const rects: Rect[] = [];
  let x = 0;
  for (const g of grids) {
    rects.push({ x, y: 0, width: g.width * scale, height: g.height * scale });
    x += g.width * scale + GAP_PX;
  }
  return { width: Math.max(1, x - GAP_PX), height: high * scale, scale, rects };
}

/** The first video type the browser can record, and its file extension. */
export function pickMime(supported: (type: string) => boolean): { mime: string; ext: 'webm' | 'mp4' } | null {
  const mime = MIMES.find((t) => supported(t));
  if (!mime) return null;
  return { mime, ext: mime.startsWith('video/mp4') ? 'mp4' : 'webm' };
}

/** `<base>-t<from>-t<to>.<ext>`. */
export function recordingName(base: string, from: number, to: number, ext: string): string {
  return `${base}-t${from}-t${to}.${ext}`;
}

/** `m:ss`. */
export function formatClock(ms: number): string {
  const s = Math.floor(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

/** Recorded time: wall-clock time since the start, not counting pauses. */
export class Stopwatch {
  private pausedAt: number | null = null;
  private pausedFor = 0;

  constructor(private readonly started: number) {}

  pause(now: number): void {
    if (this.pausedAt === null) this.pausedAt = now;
  }

  resume(now: number): void {
    if (this.pausedAt === null) return;
    this.pausedFor += now - this.pausedAt;
    this.pausedAt = null;
  }

  elapsed(now: number): number {
    return (this.pausedAt ?? now) - this.started - this.pausedFor;
  }
}

/**
 * Picks GIF frames at about GIF_FPS on recorded time (the 2 ms slack absorbs 60 Hz jitter), holds
 * back while the encoder is GIF_BACKLOG frames behind, and stops at GIF_MAX_FRAMES (Decision 14).
 */
export class GifSampler {
  frames = 0;
  private last: number | null = null;

  get full(): boolean {
    return this.frames >= GIF_MAX_FRAMES;
  }

  /** Whether to take a frame at recorded time `t` with `backlog` frames still being encoded. */
  due(t: number, backlog: number): boolean {
    if (this.full || backlog >= GIF_BACKLOG) return false;
    return this.last === null || t - this.last >= 1000 / GIF_FPS - 2;
  }

  /** Takes a frame at `t`; returns the frame before it's delay (the time between them), or null for the first. */
  take(t: number): number | null {
    this.frames++;
    const prev = this.last;
    this.last = t;
    return prev === null ? null : t - prev;
  }
}

/** A GIF stores delays in hundredths of a second, and browsers slow anything under 20 ms. */
export function gifDelay(ms: number): number {
  return Math.max(20, Math.round(ms / 10) * 10);
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/recording/frames.test.ts)`
Expected: `11 passed`.

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/recording/frames.ts web/src/recording/frames.test.ts
git commit -m "Add recording's frame layout, codec choice, file names and timing" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 7: The GIF encoder and its worker

*Mechanical (full code).* Browser: nothing to check (the worker is not started until Task 8).

**Files:**
- Modify: `web/package.json`, `web/package-lock.json` (via npm)
- Create: `web/src/gifenc.d.ts`, `web/src/recording/gif-encode.ts`, `web/src/recording/gif-worker.ts`
- Test: `web/src/recording/gif-encode.test.ts`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces (`recording/gif-encode.ts`):
  - `type GifRequest = { type: 'frame'; rgba: ArrayBuffer; width: number; height: number; delay: number } | { type: 'finish' }`
  - `type GifReply = { type: 'ack'; frames: number } | { type: 'done'; bytes: ArrayBuffer }`
  - `class GifBuilder { frames: number; add(rgba: Uint8Array | Uint8ClampedArray, width: number, height: number, delay: number): void; finish(): Uint8Array }`
  - `recording/gif-worker.ts`: a module worker answering `GifRequest` with `GifReply` (an `ack` per frame; `done` with the file after `finish`)

- [ ] **Step 1: Add the dependency**

Run: `(cd web && npm install gifenc@^1.0.3)`
Expected: `web/package.json` gains `"gifenc": "^1.0.3"` under `dependencies` (next to `uplot`), and `web/package-lock.json` is updated. No other dependency changes (`git diff --stat web/package.json` shows one added line).

- [ ] **Step 2: Type it**

Create `web/src/gifenc.d.ts`:
```ts
// gifenc 1.0.3 ships no types; these are the parts the GIF worker uses (Decision 15).
declare module 'gifenc' {
  export type Palette = number[][];
  export type Format = 'rgb565' | 'rgb444' | 'rgba4444';
  export interface GifStream {
    writeFrame(
      index: Uint8Array,
      width: number,
      height: number,
      opts?: { palette?: Palette; delay?: number; repeat?: number; transparent?: boolean; transparentIndex?: number; dispose?: number },
    ): void;
    finish(): void;
    bytes(): Uint8Array;
  }
  export function GIFEncoder(opts?: { auto?: boolean; initialCapacity?: number }): GifStream;
  export function quantize(rgba: Uint8Array | Uint8ClampedArray, maxColors: number, options?: { format?: Format }): Palette;
  export function applyPalette(rgba: Uint8Array | Uint8ClampedArray, palette: Palette, format?: Format): Uint8Array;
}
```

- [ ] **Step 3: Write the failing test**

Create `web/src/recording/gif-encode.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { GifBuilder } from './gif-encode';

/** Walks a GIF's blocks and counts its images (throws on anything malformed). */
function countFrames(b: Uint8Array): number {
  let i = 6; // "GIF89a"
  const packed = b[i + 4];
  i += 7; // logical screen descriptor
  if (packed & 0x80) i += 3 * 2 ** ((packed & 7) + 1); // global color table
  const skipSubBlocks = () => {
    while (b[i] !== 0) i += b[i] + 1;
    i++;
  };
  let frames = 0;
  for (;;) {
    const kind = b[i++];
    if (kind === 0x3b) return frames; // trailer
    if (kind === 0x21) {
      i++; // extension label
      skipSubBlocks();
    } else if (kind === 0x2c) {
      const flags = b[i + 8];
      i += 9; // image descriptor
      if (flags & 0x80) i += 3 * 2 ** ((flags & 7) + 1); // local color table
      i++; // LZW minimum code size
      skipSubBlocks();
      frames++;
    } else {
      throw new Error(`unexpected block 0x${kind.toString(16)} at ${i - 1}`);
    }
  }
}

describe('GifBuilder', () => {
  it('writes a GIF89a file with one image per frame', () => {
    const builder = new GifBuilder();
    for (let k = 0; k < 3; k++) {
      const rgba = new Uint8ClampedArray(8 * 6 * 4);
      for (let p = 0; p < 8 * 6; p++) rgba.set([k * 80, 255 - k * 80, (p % 8) * 30, 255], p * 4);
      builder.add(rgba, 8, 6, 70);
    }
    expect(builder.frames).toBe(3);
    const bytes = builder.finish();
    expect(new TextDecoder().decode(bytes.subarray(0, 6))).toBe('GIF89a');
    expect(countFrames(bytes)).toBe(3);
    expect(bytes.at(-1)).toBe(0x3b);
  });
});
```

- [ ] **Step 4: Run the test to see it fail**

Run: `(cd web && npx vitest run src/recording/gif-encode.test.ts)`
Expected: FAIL — `Failed to resolve import "./gif-encode"`.

- [ ] **Step 5: Implement**

Create `web/src/recording/gif-encode.ts`:
```ts
import { applyPalette, GIFEncoder, quantize } from 'gifenc';

/** Page → GIF worker: a frame's RGBA pixels (transferred) and its delay in ms, or "finish the file". */
export type GifRequest = { type: 'frame'; rgba: ArrayBuffer; width: number; height: number; delay: number } | { type: 'finish' };

/** GIF worker → page: a frame was encoded (`frames` so far), or the finished file (transferred). */
export type GifReply = { type: 'ack'; frames: number } | { type: 'done'; bytes: ArrayBuffer };

/** An animated GIF built a frame at a time, each with its own 256-color palette (Decision 14). */
export class GifBuilder {
  frames = 0;
  private readonly gif = GIFEncoder();

  add(rgba: Uint8Array | Uint8ClampedArray, width: number, height: number, delay: number): void {
    const palette = quantize(rgba, 256);
    this.gif.writeFrame(applyPalette(rgba, palette), width, height, { palette, delay });
    this.frames++;
  }

  /** The finished file. */
  finish(): Uint8Array {
    this.gif.finish();
    return this.gif.bytes();
  }
}
```

Create `web/src/recording/gif-worker.ts`:
```ts
// Encodes GIF frames off the main thread (Decision 14): each frame is acknowledged, so the page can
// hold back while this worker is behind; `finish` answers with the file.
import { GifBuilder, type GifReply, type GifRequest } from './gif-encode';

let builder = new GifBuilder();
const reply = (message: GifReply, transfer: Transferable[] = []) => postMessage(message, { transfer });

addEventListener('message', (event: MessageEvent<GifRequest>) => {
  const m = event.data;
  if (m.type === 'frame') {
    builder.add(new Uint8ClampedArray(m.rgba), m.width, m.height, m.delay);
    reply({ type: 'ack', frames: builder.frames });
    return;
  }
  const bytes = builder.finish().buffer as ArrayBuffer;
  reply({ type: 'done', bytes }, [bytes]);
  builder = new GifBuilder();
});
```

- [ ] **Step 6: Run the test to see it pass**

Run: `(cd web && npx vitest run src/recording/gif-encode.test.ts)`
Expected: `1 passed`.

- [ ] **Step 7: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds (`tsc` accepts `gifenc` through `gifenc.d.ts`); all test files pass.

- [ ] **Step 8: Commit**

```bash
git add web/package.json web/package-lock.json web/src/gifenc.d.ts web/src/recording/gif-encode.ts web/src/recording/gif-encode.test.ts web/src/recording/gif-worker.ts
git commit -m "Add a GIF encoder worker on gifenc" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 8: Recording the grid — WebM and GIF, and the ● Record control

*Needs judgment (full code given; the recorder is DOM-only, so the controller's browser pass is its test).* Browser (controller), on `ii-2-unit` at 5×:
- ● Record → WebM (stamp on): the button becomes "■ 0:00" and counts; pause the world → the clock stops; play → it continues; Reset during the recording → it continues; click ■ → downloads `sugarscape-ii-2-unit-seed<s>-t<from>-t<to>.webm` (or `.mp4`), non-empty, type `video/webm` (MIME from `MediaRecorder.isTypeSupported`); it plays, with `t = <tick>` bottom-left; with the stamp off, no text.
- ● Record → GIF at **Max** for 10 s: a `PerformanceObserver({ type: 'longtask', buffered: true })` records no main-thread task over 50 ms; ■ shows "Finishing GIF… k / n" then downloads `….gif` starting `GIF89a` whose frame count is about 15 per recorded second (parse or open it).
- A GIF left recording at 1× for over 60 s stops by itself at 900 frames with the notice "The GIF reached 900 frames, so recording stopped." and downloads.
- A 200 × 200 world records at 5 px per cell (1000 × 1000); overlays, a followed trail and the selection box appear in the frames.
- Everything else still works (Share, Export, tabs).

**Files:**
- Create: `web/src/recording/recorder.ts`, `web/src/ui/record-control.ts`
- Modify: `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `frameLayout`, `pickMime`, `recordingName`, `formatClock`, `Stopwatch`, `GifSampler`, `gifDelay`, `GIF_FPS` (Task 6); `GifRequest`, `GifReply`, `gif-worker.ts` (Task 7); `downloadBlob`; `showNotice` (Task 5).
- Produces:
  - `recording/recorder.ts`: `type RecordFormat = 'webm' | 'gif'`; `interface RecordedGrid { canvas: HTMLCanvasElement; cells: () => { width: number; height: number }; label?: string }`; `interface RecordSource { grids: () => RecordedGrid[]; tick: () => number; running: () => boolean; base: () => string }`; `interface Recording { capture(): void; sync(): void; elapsed(): number; stop(progress?: (done: number, total: number) => void): Promise<void> }`; `webmSupport(): { mime: string; ext: 'webm' | 'mp4' } | null`; `startRecording(format: RecordFormat, source: RecordSource, stamp: boolean, onLimit: () => void): Recording`
  - `ui/record-control.ts`: `interface RecordControl { readonly el: HTMLElement; capture(): void; sync(): void }`; `buildRecordControl(source: RecordSource): RecordControl`

- [ ] **Step 1: The recorder**

Create `web/src/recording/recorder.ts`:
```ts
import { downloadBlob } from '../downloads';
import type { GifReply, GifRequest } from './gif-encode';
import { frameLayout, GIF_FPS, gifDelay, GifSampler, pickMime, recordingName, Stopwatch } from './frames';

export type RecordFormat = 'webm' | 'gif';

/** A grid to record: its canvas as drawn (overlays, trail, selection) and its size in cells. */
export interface RecordedGrid { canvas: HTMLCanvasElement; cells: () => { width: number; height: number }; label?: string }

/** What a recording shows and follows: the grids, the tick for the stamp, the run state, the file-name base. */
export interface RecordSource {
  grids: () => RecordedGrid[];
  tick: () => number;
  running: () => boolean;
  /** The file name before `-t<from>-t<to>.<ext>`. */
  base: () => string;
}

export interface Recording {
  /** A displayed snapshot has been drawn: record a frame (if running, and for a GIF if one is due). */
  capture(): void;
  /** Pauses or resumes with the world. */
  sync(): void;
  /** Recorded time so far, in ms (pauses excluded). */
  elapsed(): number;
  /** Stops and downloads the file; `progress` reports the GIF worker finishing its queue. */
  stop(progress?: (done: number, total: number) => void): Promise<void>;
}

/** The first video type this browser records, or null. */
export function webmSupport(): { mime: string; ext: 'webm' | 'mp4' } | null {
  if (typeof MediaRecorder === 'undefined') return null;
  return pickMime((type) => MediaRecorder.isTypeSupported(type));
}

/** Starts recording; throws if this browser cannot. `onLimit` is called when a GIF reaches its frame cap. */
export function startRecording(format: RecordFormat, source: RecordSource, stamp: boolean, onLimit: () => void): Recording {
  if (format === 'gif') return new GifRecording(source, stamp, onLimit);
  const mime = webmSupport();
  if (!mime) throw new Error('this browser cannot record video');
  return new WebmRecording(source, stamp, mime);
}

/**
 * Draws the grids, as shown on screen, into a recording frame whose size is fixed when recording
 * starts; a later layout of another size (a grid resized by a reset) is scaled to fit (Decision 13).
 */
class Composer {
  readonly canvas = document.createElement('canvas');
  readonly ctx: CanvasRenderingContext2D;

  constructor(private readonly source: RecordSource, private readonly stamp: boolean, willReadFrequently: boolean) {
    const layout = frameLayout(source.grids().map((g) => g.cells()));
    this.canvas.width = layout.width;
    this.canvas.height = layout.height;
    this.ctx = this.canvas.getContext('2d', { willReadFrequently })!;
  }

  draw(): void {
    const { canvas, ctx } = this;
    const grids = this.source.grids();
    const layout = frameLayout(grids.map((g) => g.cells()));
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--bg').trim() || '#000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const f = Math.min(canvas.width / layout.width, canvas.height / layout.height);
    ctx.setTransform(f, 0, 0, f, (canvas.width - layout.width * f) / 2, (canvas.height - layout.height * f) / 2);
    ctx.imageSmoothingEnabled = false;
    grids.forEach((g, i) => {
      const r = layout.rects[i];
      ctx.drawImage(g.canvas, r.x, r.y, r.width, r.height);
      if (g.label) this.tag(g.label, r.x + 4, r.y + 4);
    });
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    if (this.stamp) this.tag(`t = ${this.source.tick()}`, 4, canvas.height - this.fontSize() - 10);
  }

  private fontSize(): number {
    return Math.max(12, Math.round(this.canvas.height / 40));
  }

  /** White text on a dark box, readable on any landscape. */
  private tag(text: string, x: number, y: number): void {
    const { ctx } = this;
    const size = this.fontSize();
    ctx.font = `600 ${size}px ui-sans-serif, system-ui, sans-serif`;
    ctx.textBaseline = 'top';
    const width = ctx.measureText(text).width;
    ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
    ctx.fillRect(x, y, width + 8, size + 6);
    ctx.fillStyle = '#fff';
    ctx.fillText(text, x + 4, y + 3);
  }
}

/** WebM (or MP4): one video frame per captured snapshot, paused with the world (Decision 14). */
class WebmRecording implements Recording {
  private readonly composer: Composer;
  private readonly recorder: MediaRecorder;
  private readonly track: CanvasCaptureMediaStreamTrack;
  private readonly chunks: Blob[] = [];
  private readonly watch = new Stopwatch(performance.now());
  private readonly from: number;
  private stopped = false;

  constructor(private readonly source: RecordSource, stamp: boolean, private readonly type: { mime: string; ext: string }) {
    this.composer = new Composer(source, stamp, false);
    const stream = this.composer.canvas.captureStream(0);
    this.track = stream.getVideoTracks()[0] as CanvasCaptureMediaStreamTrack;
    this.recorder = new MediaRecorder(stream, { mimeType: type.mime });
    this.recorder.ondataavailable = (e) => {
      if (e.data.size > 0) this.chunks.push(e.data);
    };
    this.from = source.tick();
    this.recorder.start(1000);
    // The first frame, even while paused, so the file is never empty; the next sync pauses it.
    this.frame();
  }

  capture(): void {
    if (this.stopped) return;
    this.sync();
    if (this.source.running()) this.frame();
  }

  sync(): void {
    if (this.stopped) return;
    const now = performance.now();
    if (this.source.running()) {
      this.watch.resume(now);
      if (this.recorder.state === 'paused') this.recorder.resume();
    } else {
      this.watch.pause(now);
      if (this.recorder.state === 'recording') this.recorder.pause();
    }
  }

  elapsed(): number {
    return this.watch.elapsed(performance.now());
  }

  async stop(): Promise<void> {
    if (this.stopped) return;
    this.stopped = true;
    const to = this.source.tick();
    await new Promise<void>((resolve) => {
      this.recorder.onstop = () => resolve();
      this.recorder.stop();
    });
    this.track.stop();
    downloadBlob(recordingName(this.source.base(), this.from, to, this.type.ext), new Blob(this.chunks, { type: this.type.mime }));
  }

  private frame(): void {
    this.composer.draw();
    this.track.requestFrame();
  }
}

/** A GIF: frames sampled at about 15 fps of recorded time, encoded in a worker (Decision 14). */
class GifRecording implements Recording {
  private readonly composer: Composer;
  private readonly worker = new Worker(new URL('./gif-worker.ts', import.meta.url), { type: 'module' });
  private readonly watch = new Stopwatch(performance.now());
  private readonly sampler = new GifSampler();
  /** The latest frame, sent once the next one is taken (its delay is the time until then). */
  private held: { rgba: ArrayBuffer; width: number; height: number } | null = null;
  private sent = 0;
  private acked = 0;
  private failed: string | null = null;
  private readonly from: number;
  private stopped = false;

  constructor(private readonly source: RecordSource, stamp: boolean, private readonly onLimit: () => void) {
    this.composer = new Composer(source, stamp, true);
    this.worker.onmessage = (e: MessageEvent<GifReply>) => {
      if (e.data.type === 'ack') this.acked = e.data.frames;
    };
    this.worker.onerror = (e) => {
      this.failed = e.message || 'the GIF encoder failed';
    };
    this.from = source.tick();
    this.grab(0);
  }

  capture(): void {
    if (this.stopped) return;
    this.sync();
    if (!this.source.running()) return;
    const t = this.elapsed();
    if (this.sampler.due(t, this.sent - this.acked)) this.grab(t);
  }

  sync(): void {
    if (this.stopped) return;
    const now = performance.now();
    if (this.source.running()) this.watch.resume(now);
    else this.watch.pause(now);
  }

  elapsed(): number {
    return this.watch.elapsed(performance.now());
  }

  async stop(progress?: (done: number, total: number) => void): Promise<void> {
    if (this.stopped) return;
    this.stopped = true;
    const to = this.source.tick();
    if (this.held) this.send(this.held, 1000 / GIF_FPS);
    this.held = null;
    const total = this.sent;
    try {
      const bytes = await new Promise<ArrayBuffer>((resolve, reject) => {
        if (this.failed) {
          reject(new Error(this.failed));
          return;
        }
        this.worker.onmessage = (e: MessageEvent<GifReply>) => {
          if (e.data.type === 'ack') progress?.(e.data.frames, total);
          else resolve(e.data.bytes);
        };
        this.worker.onerror = (e) => reject(new Error(e.message || 'the GIF encoder failed'));
        const finish: GifRequest = { type: 'finish' };
        this.worker.postMessage(finish);
      });
      downloadBlob(recordingName(this.source.base(), this.from, to, 'gif'), new Blob([bytes], { type: 'image/gif' }));
    } finally {
      this.worker.terminate();
    }
  }

  private grab(t: number): void {
    this.composer.draw();
    const { width, height } = this.composer.canvas;
    const rgba = this.composer.ctx.getImageData(0, 0, width, height).data.buffer as ArrayBuffer;
    const delay = this.sampler.take(t);
    if (this.held && delay !== null) this.send(this.held, delay);
    this.held = { rgba, width, height };
    if (this.sampler.full) this.onLimit();
  }

  private send(frame: { rgba: ArrayBuffer; width: number; height: number }, delay: number): void {
    const message: GifRequest = { type: 'frame', rgba: frame.rgba, width: frame.width, height: frame.height, delay: gifDelay(delay) };
    this.worker.postMessage(message, [frame.rgba]);
    this.sent++;
  }
}
```

- [ ] **Step 2: The ● Record control**

Create `web/src/ui/record-control.ts`:
```ts
import { formatClock } from '../recording/frames';
import { startRecording, webmSupport, type RecordFormat, type Recording, type RecordSource } from '../recording/recorder';
import { h } from './dom';
import { showNotice } from './notice';

export interface RecordControl {
  readonly el: HTMLElement;
  /** A displayed snapshot has been drawn: record it (while recording and running). */
  capture(): void;
  /** The run state changed: pause or resume the recording. */
  sync(): void;
}

const message = (e: unknown): string => (e instanceof Error ? e.message : String(e));

/** "● Record" with a menu (WebM | GIF, "Stamp the tick"); while recording "■ m:ss" stops and downloads. */
export function buildRecordControl(source: RecordSource): RecordControl {
  let recording: Recording | null = null;
  let stopping = false;
  let timer: ReturnType<typeof setInterval> | undefined;
  const video = webmSupport();
  const stamp = h('input', { type: 'checkbox', checked: true });
  const webm = h(
    'button',
    {
      disabled: !video,
      title: video ? `Record the grid as video (${video.mime})` : 'This browser cannot record video',
      onclick: () => start('webm'),
    },
    'WebM',
  );
  const gif = h(
    'button',
    { title: 'Record the grid as an animated GIF (about 15 frames a second, at most 900 frames)', onclick: () => start('gif') },
    'GIF',
  );
  const menu = h(
    'details',
    { class: 'menu' },
    h('summary', { title: 'Record the grid as it runs' }, '● Record'),
    h('div', { class: 'menu-items' }, webm, gif, h('label', {}, stamp, ' Stamp the tick')),
  );
  const stopButton = h('button', { class: 'record-stop', hidden: true, title: 'Stop recording and download the file', onclick: () => void stop() });

  const show = (): void => {
    menu.hidden = recording !== null;
    stopButton.hidden = recording === null;
    if (recording && !stopping) stopButton.textContent = `■ ${formatClock(recording.elapsed())}`;
  };

  function start(format: RecordFormat): void {
    menu.open = false;
    try {
      recording = startRecording(format, source, stamp.checked, () => {
        showNotice('The GIF reached 900 frames, so recording stopped.', 10_000);
        void stop();
      });
    } catch (e) {
      showNotice(`Recording could not start (${message(e)})`, 10_000);
      return;
    }
    timer = setInterval(() => {
      recording?.sync();
      show();
    }, 500);
    show();
  }

  async function stop(): Promise<void> {
    const r = recording;
    if (!r || stopping) return;
    stopping = true;
    clearInterval(timer);
    stopButton.disabled = true;
    stopButton.textContent = 'Saving…';
    try {
      await r.stop((done, total) => (stopButton.textContent = `Finishing GIF… ${done} / ${total}`));
    } catch (e) {
      showNotice(`The recording could not be saved (${message(e)})`, 10_000);
    } finally {
      recording = null;
      stopping = false;
      stopButton.disabled = false;
      show();
    }
  }

  return {
    el: h('span', { class: 'record' }, menu, stopButton),
    capture: () => {
      if (!stopping) recording?.capture();
    },
    sync: () => {
      if (!stopping) recording?.sync();
    },
  };
}
```

- [ ] **Step 3: Wire it into the page**

In `web/src/main.ts`:
- add `import { buildRecordControl } from './ui/record-control';` after the `import { InspectPanel } …` line.
- replace `document.querySelector('.toolbar-end')!.append(shareMenu, exportMenu);` with:
```ts
  const record = buildRecordControl({
    grids: () => [{ canvas: grid.canvas, cells: () => engine.size() }],
    tick: () => engine.tick,
    running: () => engine.running,
    base: () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}`,
  });
  engine.on('run', () => record.sync());
  document.querySelector('.toolbar-end')!.append(record.el, shareMenu, exportMenu);
```
- replace
```ts
  let dirty = true;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  const loop = (now: number) => {
    try {
      engine.pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
    } catch (e) {
```
with
```ts
  let dirty = true;
  /** A snapshot arrived since the last frame: once drawn, the recording captures it (Decision 13). */
  let fresh = false;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  engine.on('snapshot', () => (fresh = true));
  const loop = (now: number) => {
    try {
      engine.pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
      if (fresh) {
        fresh = false;
        record.capture();
      }
    } catch (e) {
```
(the rest of `loop` stays as it is).

Append to `web/src/style.css`:
```css
.record { display: inline-flex; }
.record-stop { color: var(--error); font-variant-numeric: tabular-nums; }
.menu-items label { display: flex; gap: 6px; align-items: center; padding: 2px 4px; }
```

- [ ] **Step 4: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds (the build output lists a separate `gif-worker-*.js` chunk); all test files pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/recording/recorder.ts web/src/ui/record-control.ts web/src/main.ts web/src/style.css
git commit -m "Record the grid as WebM or GIF" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 9: The lockstep coordinator, B's copy, and handing a world between engines

*Needs judgment (full code given; the point is that ticks stay equal through steps, adaptive Max, Reset and one world rebuilding — Decision 9 — and that "Keep B" moves a live world — Decision 8).* Browser: nothing to check (nothing in the page uses it until Task 10).

**Files:**
- Create: `web/src/compare/lockstep.ts`
- Test: `web/src/compare/lockstep.test.ts`
- Modify: `web/src/engine.ts`
- Test: `web/src/engine.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: `Engine.create/advance/replay/pump/setRunning/on/session/tick/crashed`, `InitialState`, `Speed` (Tasks 3, 7a); `Session`, `LogEntry` (Task 1).
- Produces:
  - `compare/lockstep.ts`: `FAST_MS = 25`, `SLOW_MS = 40`, `MAX_BATCH = 10_000`; `class AdaptiveBatch { n: number; update(ms: number): void }`; `type LockstepEvent = 'run' | 'tick'`; `class Lockstep { constructor(worlds: [Engine, Engine], speed: Speed, now?: () => number); readonly worlds; running: boolean; speed: Speed; on(event: LockstepEvent, fn: () => void): () => void; setRunning(on: boolean): void; setSpeed(speed: Speed): void; pump(now?: number): void; advance(n?: number): Promise<void>; reset(): Promise<void>; settled(): Promise<void>; dispose(): void }`; `copyWorld(session: Session, tick: number, create: (initial: InitialState) => Promise<Engine>, progress?: (at: number, of: number) => void, now?: () => number): Promise<Engine>`
  - `engine.ts`: `takeWorld(other: Engine): Promise<void>`; `close(): void`

- [ ] **Step 1: Write the failing tests**

Create `web/src/compare/lockstep.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { Engine, type InitialState, type Speed } from '../engine';
import { fakeModule } from '../fake-sim.fixture';
import type { LogEntry } from '../protocol';
import { SimHost } from '../sim-host';
import { InlineTransport } from '../transport';
import type { Config, Preset } from '../types';
import { AdaptiveBatch, copyWorld, Lockstep, MAX_BATCH } from './lockstep';

const config = { width: 4, height: 3 } as unknown as Config;
const presets: Preset[] = [{ id: 'ii-2-unit', name: 'Unit', source: 'II-2', description: '', config }];
/** Lets queued microtasks and zero-delay timers run. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const create = (initial: InitialState) =>
  Engine.create(initial, { presets, transport: new InlineTransport(new SimHost(fakeModule())) });

async function pair(speed: Speed = 1, now?: () => number) {
  const a = await create({ config, seed: 1 });
  const b = await create({ config, seed: 2 });
  return { a, b, lock: new Lockstep([a, b], speed, now) };
}

/** Runs `k` animation frames of the lockstep loop. */
async function frames(lock: Lockstep, k: number): Promise<void> {
  for (let i = 0; i < k; i++) {
    lock.pump(i);
    await settle();
  }
}

describe('AdaptiveBatch', () => {
  it('doubles while pairs take ≤ 25 ms, halves above 40 ms, and stays within 1…10 000', () => {
    const batch = new AdaptiveBatch();
    expect(batch.n).toBe(1);
    batch.update(25);
    expect(batch.n).toBe(2);
    for (let i = 0; i < 20; i++) batch.update(1);
    expect(batch.n).toBe(MAX_BATCH);
    batch.update(30);
    expect(batch.n).toBe(MAX_BATCH);
    batch.update(41);
    expect(batch.n).toBe(MAX_BATCH / 2);
    for (let i = 0; i < 20; i++) batch.update(100);
    expect(batch.n).toBe(1);
  });
});

describe('Lockstep', () => {
  it('steps both worlds together, one pair of requests at a time', async () => {
    const { a, b, lock } = await pair(5);
    const ticks: number[] = [];
    lock.on('tick', () => ticks.push(a.tick));
    lock.setRunning(true);
    lock.pump(0);
    lock.pump(1); // a pair is still in flight: skipped
    await settle();
    expect([a.tick, b.tick]).toEqual([5, 5]);
    await frames(lock, 3);
    expect([a.tick, b.tick]).toEqual([20, 20]);
    expect(ticks).toEqual([5, 10, 15, 20]);
    lock.setRunning(false);
    await frames(lock, 2);
    expect([a.tick, b.tick]).toEqual([20, 20]);
    expect(a.running || b.running).toBe(false);
  });

  it('at Max, doubles the ticks per pair while replies are quick', async () => {
    const { a, b, lock } = await pair('max', () => 0); // every pair "takes" 0 ms
    lock.setRunning(true);
    await frames(lock, 4);
    expect([a.tick, b.tick]).toEqual([15, 15]); // 1 + 2 + 4 + 8
  });

  it('Step advances both; Reset rewinds both to t = 0, each replaying its log', async () => {
    const { a, b, lock } = await pair();
    await lock.advance(3);
    expect(await b.place(0, 2, {})).toBeNull();
    await lock.advance(2);
    await lock.reset();
    expect([a.tick, b.tick]).toEqual([0, 0]);
    expect([a.replayLeft, b.replayLeft]).toEqual([0, 1]);
    await lock.advance(3);
    expect([a.population, b.population]).toEqual([1, 2]);
  });

  it('rewinds the other world when one is rebuilt, even with a pair in flight', async () => {
    const { a, b, lock } = await pair(2);
    lock.setRunning(true);
    await frames(lock, 3);
    lock.pump(10); // a pair in flight
    expect(await a.resetWith((c) => void (c.height = 5))).toBeNull();
    await lock.settled();
    expect([a.tick, b.tick]).toEqual([0, 0]);
    expect(a.size()).toEqual({ width: 4, height: 5 });
    await frames(lock, 2);
    expect(a.tick).toBe(b.tick);
    lock.setRunning(false);
    await lock.settled();
    expect(await b.reset(undefined, 9)).toBeNull();
    await lock.settled();
    expect([a.tick, b.tick, b.seed]).toEqual([0, 0, 9]);
  });

  it('realigns worlds that start at different ticks', async () => {
    const a = await create({ config, seed: 1 });
    const b = await create({ config, seed: 2 });
    await a.advance(3);
    const lock = new Lockstep([a, b], 1);
    await lock.settled();
    expect([a.tick, b.tick]).toEqual([0, 0]);
  });
});

describe('copyWorld', () => {
  it('builds B from A’s session up to A’s tick and brings it to that tick', async () => {
    // A replays a session: the place at 3 has happened by tick 9, the erase at 20 is still pending.
    const log: LogEntry[] = [
      { tick: 3, cmd: { type: 'place', x: 0, y: 2, overrides: {} } },
      { tick: 20, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    const a = await create({ config, seed: 1, log });
    await a.advance(9);
    const { session, tick } = await a.session();
    expect(session.log).toEqual(log);
    const seen: [number, number][] = [];
    const b = await copyWorld(session, tick, create, (at, of) => seen.push([at, of]));
    expect(b.tick).toBe(9);
    expect([b.population, a.population]).toEqual([2, 2]);
    expect(await b.fingerprint()).toBe(await a.fingerprint());
    // The place at 3 is copied; A's pending erase at 20 is not (Decision 9).
    expect((await b.session()).session.log).toEqual([log[0]]);
    expect(b.replayLeft).toBe(0);
    expect(seen[0]).toEqual([0, 9]);
    expect(seen.at(-1)).toEqual([9, 9]);
  });
});
```

In `web/src/engine.test.ts`, append:
```ts
describe('Engine handing over a world', () => {
  it('takes over another engine’s world: its transport, session and state', async () => {
    const a = await setup();
    const b = await setup();
    await b.engine.advance(4);
    expect(await b.engine.place(0, 2, {})).toBeNull();
    await b.engine.select(0, 2);
    const seen: EngineEvent[] = [];
    for (const e of ['reset', 'select', 'snapshot'] as const) a.engine.on(e, () => seen.push(e));
    await a.engine.takeWorld(b.engine);
    expect(seen).toEqual(['reset', 'select', 'snapshot']);
    expect([a.engine.tick, a.engine.population]).toEqual([4, 2]);
    expect(a.engine.selection).toEqual({ x: 0, y: 2, agentId: 2 });
    await a.engine.advance(1);
    expect(b.module.sims[0].ticks).toBe(5);
    expect(a.module.sims[0].ticks).toBe(0);
    expect((await a.engine.session()).session.log.map((e) => e.cmd.type)).toEqual(['place']);
    expect(b.engine.crashed).not.toBeNull();
    expect(a.engine.crashed).toBeNull();
  });

  it('close stops an engine for good, without a crash event', async () => {
    const { engine } = await setup();
    let crashes = 0;
    engine.on('crash', () => crashes++);
    engine.close();
    expect(engine.crashed).not.toBeNull();
    await engine.advance(1);
    expect(engine.tick).toBe(0);
    expect(crashes).toBe(0);
  });
});
```

In `web/src/determinism.test.ts`, add `import { copyWorld } from './compare/lockstep';` as the first local import (before `./engine`), and append inside `describe('sessions replay exactly', …)` (after the Max test):
```ts
  it('copies a world for Compare: B reaches A’s tick with A’s fingerprint', async () => {
    const a = await create(9);
    await a.advance(5);
    await a.place(3, 3, {});
    await a.infect(3, 3, -1);
    await a.advance(20);
    await a.paint(8, 8, 2, 1, 0);
    await a.advance(15);
    const { session, tick } = await a.session();
    const b = await copyWorld(session, tick, (s) => Engine.create(s, { presets, transport: inline() }));
    expect(b.tick).toBe(tick);
    expect(await b.fingerprint()).toBe(await a.fingerprint());
  });
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/compare/lockstep.test.ts src/engine.test.ts)`
Expected: FAIL — `Failed to resolve import "./lockstep"`; `a.engine.takeWorld is not a function`, `engine.close is not a function`.

- [ ] **Step 3: Hand a world between engines**

In `web/src/engine.ts`, after `endReplay()` add:
```ts
  /**
   * Takes over `other`'s world — its transport (worker), session and state — as Compare's
   * "Keep B" does (Decision 8). Both must be paused; `other` is dead afterwards. Panels bound to
   * this engine redraw from the events that follow.
   */
  async takeWorld(other: Engine): Promise<void> {
    await this.quiet(() =>
      other.quiet(async () => {
        const mine = this.transport;
        mine.onFatal = null;
        mine.onPost = null;
        mine.close();
        this.transport = other.transport;
        this.transport.onFatal = (message) => this.crash(message);
        this.transport.onPost = (s) => this.onPost(s);
        other.crashed = 'This world now runs in another engine.';
        this.seed = other.seed;
        this.presetId = other.presetId;
        this.baseConfig = other.baseConfig;
        this.config = other.config;
        this.tick = other.tick;
        this.population = other.population;
        this.latest = other.latest;
        this.last = other.last;
        this.width = other.width;
        this.height = other.height;
        this.followedId = other.followedId;
        this.followedLive = other.followedLive;
        this.trailCells = other.trailCells;
        this.edges = { ...other.edges };
        this.charts = new Map(other.charts);
        this.landscapes = other.landscapes;
        this.shown = other.shown;
        this.spare = [...other.spare];
        this.selection = other.selection;
        this.inspection = other.inspection;
        this.replayLeft = other.replayLeft;
        this.origin = other.origin;
        this.colorMode = other.colorMode;
        this.layer = other.layer;
        this.overlays = { ...other.overlays };
        // Nothing sent before this belongs to the new world.
        this.selectionGen++;
        this.displayGen++;
        this.lastRefresh = -Infinity;
      }),
    );
    for (const event of ['reset', 'follow', 'display', 'replay'] as const) this.emit(event);
    if (this.inspection) this.emit('select');
    this.emit('snapshot');
  }

  /** Stops this engine for good and closes its transport (Compare's "Keep A" discards B this way). */
  close(): void {
    this.crashed ??= 'This world was closed.';
    this.running = false;
    this.maxOn = false;
    this.transport.onFatal = null;
    this.transport.onPost = null;
    this.transport.close();
  }
```

- [ ] **Step 4: The coordinator**

Create `web/src/compare/lockstep.ts`:
```ts
import type { Engine, InitialState, Speed } from '../engine';
import type { Session } from '../protocol';

/** At Max in Compare, the ticks per pair double while a pair takes at most this long… */
export const FAST_MS = 25;
/** …and halve when it takes longer than this… */
export const SLOW_MS = 40;
/** …within 1 … MAX_BATCH. */
export const MAX_BATCH = 10_000;

/** Ticks per request, adapted to how long requests take (Decision 9). */
export class AdaptiveBatch {
  n = 1;

  update(ms: number): void {
    if (ms <= FAST_MS) this.n = Math.min(MAX_BATCH, this.n * 2);
    else if (ms > SLOW_MS) this.n = Math.max(1, Math.floor(this.n / 2));
  }
}

export type LockstepEvent = 'run' | 'tick';

/**
 * Steps two worlds together (Decision 9): each step sends `advance n` to both and waits for both
 * replies before the next, so their ticks are always equal. The worlds never run on their own
 * meanwhile. When one world is rebuilt (a 'reset' the coordinator did not cause), every world not at
 * t = 0 rewinds by replaying its session.
 */
export class Lockstep {
  running = false;
  private inFlight: Promise<void> | null = null;
  /** Steps, rewinds and realigns run one after another, each after the loop's pair settles. */
  private chain: Promise<unknown> = Promise.resolve();
  private holds = 0;
  /** Replays the coordinator started itself: their 'reset' events are not rebuilds. */
  private rewinding = 0;
  private readonly batch = new AdaptiveBatch();
  private readonly listeners = new Map<LockstepEvent, Set<() => void>>();
  private offs: (() => void)[] = [];

  constructor(
    readonly worlds: [Engine, Engine],
    public speed: Speed,
    private readonly now: () => number = () => performance.now(),
  ) {
    for (const w of worlds) {
      w.setRunning(false);
      this.offs.push(
        w.on('reset', () => {
          if (this.rewinding === 0) void this.realign();
        }),
      );
    }
    if (worlds[0].tick !== worlds[1].tick) void this.realign();
  }

  on(event: LockstepEvent, fn: () => void): () => void {
    let set = this.listeners.get(event);
    if (!set) this.listeners.set(event, (set = new Set()));
    set.add(fn);
    return () => {
      set.delete(fn);
    };
  }

  private emit(event: LockstepEvent): void {
    this.listeners.get(event)?.forEach((fn) => fn());
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
  }

  setSpeed(speed: Speed): void {
    this.speed = speed;
  }

  /** Called every animation frame instead of the engines' own pump. */
  pump(now: number = this.now()): void {
    if (this.holds > 0 || this.inFlight) return;
    if (!this.running) {
      // Paused: the engines still refresh what their panels want.
      for (const w of this.worlds) w.pump(now);
      return;
    }
    if (this.worlds.some((w) => w.crashed)) {
      this.setRunning(false);
      return;
    }
    const max = this.speed === 'max';
    const n = max ? this.batch.n : (this.speed as number);
    this.inFlight = this.stepBoth(n, max).finally(() => (this.inFlight = null));
  }

  /** Step: both worlds advance `n` ticks. */
  advance(n = 1): Promise<void> {
    return this.exclusive(() => this.stepBoth(n, false));
  }

  /** Reset: both worlds rewind to t = 0, each replaying its log. */
  reset(): Promise<void> {
    return this.exclusive(() => this.rewind(this.worlds));
  }

  /** Resolves once every step, rewind and realign queued so far has finished. */
  settled(): Promise<void> {
    return this.exclusive(async () => {});
  }

  dispose(): void {
    for (const off of this.offs) off();
    this.offs = [];
    this.listeners.clear();
  }

  private async stepBoth(n: number, adapt: boolean): Promise<void> {
    const start = this.now();
    await Promise.all(this.worlds.map((w) => w.advance(n)));
    if (adapt) this.batch.update(this.now() - start);
    this.emit('tick');
  }

  private exclusive<T>(fn: () => Promise<T>): Promise<T> {
    this.holds++;
    const run = this.chain.then(async () => {
      await this.inFlight;
      return fn();
    });
    this.chain = run.catch(() => undefined);
    return run.finally(() => this.holds--);
  }

  private async rewind(worlds: Engine[]): Promise<void> {
    this.rewinding++;
    try {
      await Promise.all(worlds.map((w) => w.replay()));
    } finally {
      this.rewinding--;
    }
    this.emit('tick');
  }

  /** After a rebuild: every world not at t = 0 replays its session (a few rounds, in case of races). */
  private realign(): Promise<void> {
    return this.exclusive(async () => {
      for (let round = 0; round < 3; round++) {
        const behind = this.worlds.filter((w) => w.tick !== 0);
        if (behind.length === 0) return;
        await this.rewind(behind);
      }
    });
  }
}

/**
 * Builds Compare's B (Decision 9): A's session with its log truncated to ticks ≤ `tick`, advanced to
 * `tick` in adaptive batches (its host replays the log on the way). Closes B if it cannot get there.
 */
export async function copyWorld(
  session: Session,
  tick: number,
  create: (initial: InitialState) => Promise<Engine>,
  progress: (at: number, of: number) => void = () => {},
  now: () => number = () => performance.now(),
): Promise<Engine> {
  const b = await create({ ...session, log: session.log.filter((e) => e.tick <= tick) });
  try {
    const batch = new AdaptiveBatch();
    progress(b.tick, tick);
    while (b.tick < tick) {
      const before = b.tick;
      const start = now();
      await b.advance(Math.min(batch.n, tick - b.tick));
      batch.update(now() - start);
      if (b.crashed) throw new Error(b.crashed);
      if (b.tick === before) throw new Error('the copy stopped advancing');
      progress(b.tick, tick);
    }
    return b;
  } catch (e) {
    b.close();
    throw e;
  }
}
```

- [ ] **Step 5: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/compare/lockstep.test.ts src/engine.test.ts)`
Expected: all pass (6 in lockstep.test.ts; the 2 new engine tests and every earlier one).
Run: `(cd web && npm run build && npx vitest run src/determinism.test.ts)`
Expected: `6 passed`.

- [ ] **Step 6: Full verification**

Run: `(cd web && npm test)`
Expected: all test files pass.

- [ ] **Step 7: Commit**

```bash
git add web/src/compare/lockstep.ts web/src/compare/lockstep.test.ts web/src/engine.ts web/src/engine.test.ts web/src/determinism.test.ts
git commit -m "Add the lockstep coordinator, B's copy and handing a world between engines" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 10: Compare mode — toggle, B's copy, grid headers, lockstep toolbar, Keep A / Keep B

*Needs judgment (full code given; watch the order of teardown in `leave` and that A stays paused while B is copied).* In this task B's grid does not take tool clicks yet, and Rules/Inspect/Credit/Charts still show A (Tasks 11–12). Browser (controller), `/?debug`:
- On `ii-2-unit`, play to t ≈ 300 with a paint and a placed agent along the way; click **Compare**: A pauses, the toolbar's run controls are disabled, "Copying A… t / 300" counts up in B's place, then B's grid appears identical to A's; `await sugarscape.compare().b.fingerprint() === await sugarscape.engine.fingerprint()`.
- Each grid has a header "A · seed n · 🎲" / "B · seed n · 🎲"; the toolbar's seed box, 🎲 and chips are hidden; the readout reads `t = T · A n · B m agents`.
- Play at 5×, 100× and Max: both ticks stay equal (readout) and both grids animate; at Max the readout climbs fast and no long task > 50 ms (PerformanceObserver); Pause and Step (Step advances both by 1).
- 🎲 on B's header: B gets a new seed and both show t = 0 (A rewound by replaying); Reset in the toolbar: both at t = 0; after A's rewind A's paint and placed agent reappear at their ticks.
- Display controls (e.g. Agents: Wealth, an overlay) change both grids.
- Leaving: click **Compare** again → the dialog "Keep A / Keep B / Cancel": Cancel stays; Keep A returns to A as it was; Keep B (after making B differ with 🎲) shows B's world in the playground — seed, tick, readout, Rules panel, charts and a following Play all continue from B's state; the address bar and Share still work afterwards.
- Switching to Experiments while comparing pauses both; switching back keeps Compare.
- Recording in Compare (still A only in this task) works; a GIF of a Max compare run keeps both ticks equal.

**Files:**
- Modify: `web/src/engine.ts` (RunControls), `web/src/ui/toolbar.ts`, `web/src/main.ts`, `web/index.html`, `web/src/style.css`
- Create: `web/src/compare/compare-view.ts`

**Interfaces:**
- Consumes: `Lockstep`, `copyWorld` (Task 9); `Engine.takeWorld/close/session` (Tasks 3, 9); `followChip`, `replayChip`, `Chip` (Task 5); `GridView`; `showNotice` (Task 5); `buildRecordControl` (Task 8).
- Produces:
  - `engine.ts`: `interface RunControls { readonly running: boolean; readonly speed: Speed; setRunning(on: boolean): void; setSpeed(speed: Speed): void; advance(n?: number): Promise<void>; on(event: 'run', fn: () => void): () => void }` (an `Engine` and a `Lockstep` are ones)
  - `ui/toolbar.ts`: `class Toolbar { readonly el: HTMLElement; constructor(engine: Engine); setCompare(lock: Lockstep | null, b: Engine | null): void; hold(on: boolean): void }` (replaces `buildToolbar`; `followChip`, `replayChip`, `Chip` unchanged)
  - `compare/compare-view.ts`: `type WorldName = 'A' | 'B'`; `interface Playground { engine: Engine; grid: GridView; toolbar: Toolbar; syncCreditTab: (b: Engine | null) => void; onRun: () => void; onFrame: () => void; onCrash: () => void }`; `interface CompareShell { figure: HTMLElement; header: HTMLElement; progress: HTMLElement; canvas: HTMLCanvasElement }`; `compareShell(): CompareShell`; `askKeep(): Promise<WorldName | null>`; `class CompareView { constructor(p: Playground, b: Engine, shell: CompareShell); readonly lock: Lockstep; readonly b: Engine; readonly gridB: GridView; draw(): void; leave(keep: WorldName): Promise<void> }`

- [ ] **Step 1: RunControls**

In `web/src/engine.ts`, after `export type Speed = number | 'max';` add:
```ts
/** What the toolbar's Play, Step and speed drive: one engine, or Compare's lockstep (Decision 9). */
export interface RunControls {
  readonly running: boolean;
  readonly speed: Speed;
  setRunning(on: boolean): void;
  setSpeed(speed: Speed): void;
  advance(n?: number): Promise<void>;
  on(event: 'run', fn: () => void): () => void;
}
```

- [ ] **Step 2: The toolbar as a class that Compare can drive**

In `web/src/ui/toolbar.ts`:
- change the imports to:
```ts
import type { Lockstep } from '../compare/lockstep';
import { randomSeed, type Engine, type RunControls, type Speed } from '../engine';
import { h } from './dom';
```
- keep `SPEEDS`, `Chip`, `followChip` and `replayChip` as they are, and replace `export function buildToolbar(…) { … }` with:
```ts
/**
 * Play, Step, speed, seed, Reset and 🎲, the readout and the world's chips. In Compare (Decision 10)
 * Play/Step/speed/Reset drive the lockstep, the seed box, 🎲 and chips move to the grid headers,
 * and the readout shows both populations.
 */
export class Toolbar {
  readonly el: HTMLElement;
  private controls: RunControls;
  private lock: Lockstep | null = null;
  private b: Engine | null = null;
  private offs: (() => void)[] = [];
  private held = false;
  private readonly play: HTMLButtonElement;
  private readonly step: HTMLButtonElement;
  private readonly speed: HTMLSelectElement;
  private readonly seed: HTMLInputElement;
  private readonly resetButton: HTMLButtonElement;
  private readonly dice: HTMLButtonElement;
  private readonly readout = h('span', { class: 'readout' });
  /** Hidden in Compare: the headers carry them. */
  private readonly singleOnly: HTMLElement[];

  constructor(private readonly engine: Engine) {
    this.controls = engine;
    this.play = h('button', { class: 'primary', onclick: () => this.controls.setRunning(!this.controls.running) });
    this.step = h('button', { onclick: () => void this.controls.advance(1), title: 'Advance one tick' }, 'Step');
    this.speed = h(
      'select',
      {
        title: 'Ticks per frame; Max runs the simulation as fast as it goes and redraws about 30 times a second',
        onchange: () => this.controls.setSpeed(this.speed.value === 'max' ? 'max' : Number(this.speed.value)),
      },
      ...SPEEDS.map((s) => h('option', { value: String(s) }, s === 'max' ? 'Max' : `${s}×`)),
    );
    this.seed = h('input', { type: 'number', min: 0, max: 4294967295, class: 'seed', title: 'Seed' });
    const seedLabel = h('label', {}, 'Seed ', this.seed);
    this.resetButton = h(
      'button',
      { title: 'Rebuild this world and replay its edits; with another seed typed, build a new world', onclick: () => this.reset() },
      'Reset',
    );
    this.dice = h('button', { title: 'Random seed and reset', onclick: () => void engine.reset(undefined, randomSeed()) }, '🎲');
    const chips = h('span', { class: 'chips' }, followChip(engine).el, replayChip(engine).el);
    this.singleOnly = [seedLabel, this.dice, chips];
    this.el = h(
      'div',
      { class: 'toolbar' },
      h('h1', {}, 'SugarScape'),
      h('div', { class: 'group' }, this.play, this.step, this.speed),
      h('div', { class: 'group' }, seedLabel, this.resetButton, this.dice),
      this.readout,
      chips,
      h('div', { class: 'toolbar-end' }),
    );
    engine.on('run', () => this.sync());
    engine.on('reset', () => {
      this.sync();
      this.tick();
    });
    engine.on('tick', () => this.tick());
    engine.on('edit', () => this.tick());
    this.sync();
    this.tick();
  }

  /** Compare on (`lock` and `b`) or off (nulls). */
  setCompare(lock: Lockstep | null, b: Engine | null): void {
    for (const off of this.offs) off();
    this.offs = [];
    this.lock = lock;
    this.b = b;
    this.controls = lock ?? this.engine;
    if (lock) this.offs.push(lock.on('run', () => this.sync()));
    if (b) for (const event of ['tick', 'edit', 'reset'] as const) this.offs.push(b.on(event, () => this.tick()));
    for (const el of this.singleOnly) el.hidden = lock !== null;
    this.speed.value = String(this.controls.speed);
    this.sync();
    this.tick();
  }

  /** Disables the run controls while Compare copies A (A must stay at the tick B is copying). */
  hold(on: boolean): void {
    this.held = on;
    this.sync();
  }

  private reset(): void {
    if (this.lock) {
      void this.lock.reset();
      return;
    }
    const s = Number(this.seed.value) >>> 0;
    // The same seed rewinds and replays the session; another seed builds a new world (Decision 4).
    void (s === this.engine.seed ? this.engine.replay() : this.engine.reset(undefined, s));
  }

  private sync(): void {
    this.play.textContent = this.controls.running ? 'Pause' : 'Play';
    this.play.disabled = this.held;
    this.step.disabled = this.held || this.controls.running;
    this.speed.disabled = this.held;
    this.resetButton.disabled = this.held;
    this.dice.disabled = this.held;
    this.seed.value = String(this.engine.seed);
  }

  private tick(): void {
    const a = this.engine;
    this.readout.textContent = this.b
      ? `t = ${a.tick} · A ${a.population} · B ${this.b.population} agents`
      : `t = ${a.tick} · ${a.population} agents`;
  }
}
```

- [ ] **Step 3: The Compare view**

Create `web/src/compare/compare-view.ts`:
```ts
import { randomSeed, type Engine } from '../engine';
import { h } from '../ui/dom';
import { GridView } from '../ui/grid-view';
import { showNotice } from '../ui/notice';
import { followChip, replayChip, type Toolbar } from '../ui/toolbar';
import { Lockstep } from './lockstep';

export type WorldName = 'A' | 'B';

/** The playground's parts that Compare extends to a second world. */
export interface Playground {
  engine: Engine;
  grid: GridView;
  toolbar: Toolbar;
  /** Shows the Credit tab while either world on screen has credit on. */
  syncCreditTab: (b: Engine | null) => void;
  /** The run state changed (recording follows it). */
  onRun: () => void;
  /** Both worlds took a step: a frame for the recording. */
  onFrame: () => void;
  onCrash: () => void;
}

/** B's figure: its header, and the copy's progress until its grid can be shown. */
export interface CompareShell { figure: HTMLElement; header: HTMLElement; progress: HTMLElement; canvas: HTMLCanvasElement }

/** Puts B's figure beside A's (the grid area splits at once) with "Copying A…" in it. */
export function compareShell(): CompareShell {
  const header = h('header', { class: 'world-header' });
  const progress = h('p', { class: 'hint copy-progress', role: 'status' }, 'Copying A…');
  const canvas = h('canvas', { class: 'grid-b', 'aria-label': 'Sugarscape grid B', hidden: true });
  const figure = h('figure', { class: 'world', id: 'world-b' }, header, progress, canvas);
  document.querySelector('#grids')!.append(figure);
  document.body.dataset.compare = 'on';
  return { figure, header, progress, canvas };
}

/** Asks which world stays when leaving Compare; null (Cancel, Escape) stays in Compare. */
export function askKeep(): Promise<WorldName | null> {
  return new Promise((resolve) => {
    const dialog = h(
      'dialog',
      { class: 'keep' },
      h(
        'form',
        { method: 'dialog' },
        h('p', {}, 'Leave Compare: which world stays in the playground?'),
        h(
          'div',
          { class: 'row' },
          h('button', { value: 'A' }, 'Keep A'),
          h('button', { value: 'B' }, 'Keep B'),
          h('button', { value: '' }, 'Cancel'),
        ),
      ),
    );
    dialog.addEventListener('close', () => {
      dialog.remove();
      resolve(dialog.returnValue === 'A' || dialog.returnValue === 'B' ? dialog.returnValue : null);
    });
    document.body.append(dialog);
    dialog.showModal();
  });
}

/** Two worlds side by side, stepped in lockstep (Decisions 8–10). */
export class CompareView {
  readonly lock: Lockstep;
  readonly gridB: GridView;
  private readonly offs: (() => void)[] = [];
  private dirty = true;

  constructor(
    private readonly p: Playground,
    readonly b: Engine,
    private readonly shell: CompareShell,
  ) {
    const a = p.engine;
    shell.progress.remove();
    shell.canvas.hidden = false;
    this.gridB = new GridView(shell.canvas, b);
    this.lock = new Lockstep([a, b], a.speed);
    this.header(document.querySelector<HTMLElement>('#world-a .world-header')!, 'A', a);
    this.header(shell.header, 'B', b);
    p.toolbar.setCompare(this.lock, b);
    // One display for both worlds: A's, mirrored to B (Decision 10).
    const mirror = () => b.setDisplay({ colorMode: a.colorMode, layer: a.layer, overlays: { ...a.overlays } });
    mirror();
    this.offs.push(
      a.on('display', mirror),
      b.on('snapshot', () => (this.dirty = true)),
      b.on('display', () => (this.dirty = true)),
      b.on('reset', () => p.syncCreditTab(b)),
      b.on('config', () => p.syncCreditTab(b)),
      b.on('crash', p.onCrash),
      b.on('fork', () => showNotice('Replay ended in B — your edit starts a new branch')),
      this.lock.on('run', p.onRun),
      this.lock.on('tick', p.onFrame),
    );
    p.syncCreditTab(b);
    p.onRun();
  }

  /** Called every animation frame: redraws B's grid when it has news. */
  draw(): void {
    if (!this.dirty) return;
    this.gridB.draw();
    this.dirty = false;
  }

  /** Leaves Compare: `keep` becomes (or stays) the playground's world (Decision 8). */
  async leave(keep: WorldName): Promise<void> {
    const { p, b } = this;
    const a = p.engine;
    this.lock.setRunning(false);
    await this.lock.settled();
    this.lock.dispose();
    for (const off of this.offs) off();
    a.setSpeed(this.lock.speed);
    p.toolbar.setCompare(null, null);
    this.shell.figure.remove();
    delete document.body.dataset.compare;
    if (keep === 'B') await a.takeWorld(b);
    else b.close();
    p.syncCreditTab(null);
    p.onRun();
  }

  /** "A · seed n · 🎲" and the world's follow and replay chips. */
  private header(el: HTMLElement, name: WorldName, engine: Engine): void {
    const seed = h('span', { class: 'hint' });
    const syncSeed = () => (seed.textContent = `seed ${engine.seed}`);
    syncSeed();
    const follow = followChip(engine);
    const replay = replayChip(engine);
    el.replaceChildren(
      h('strong', {}, name),
      seed,
      h(
        'button',
        {
          title: `Random seed and rebuild ${name} (the other world rewinds to t = 0)`,
          onclick: () => void engine.reset(undefined, randomSeed()),
        },
        '🎲',
      ),
      follow.el,
      replay.el,
    );
    el.hidden = false;
    this.offs.push(engine.on('reset', syncSeed), follow.off, replay.off, () => {
      el.hidden = true;
      el.replaceChildren();
    });
  }
}
```

- [ ] **Step 4: The grid area and styles**

In `web/index.html`, replace
```html
        <canvas id="grid" aria-label="Sugarscape grid"></canvas>
```
with
```html
        <div id="grids" class="grids">
          <figure class="world" id="world-a">
            <header class="world-header" hidden></header>
            <canvas id="grid" aria-label="Sugarscape grid"></canvas>
          </figure>
        </div>
```

Append to `web/src/style.css`:
```css
.grids { display: grid; gap: 12px; min-width: 0; }
body[data-compare] .grids { grid-template-columns: repeat(2, minmax(0, 1fr)); align-items: start; }
.world { margin: 0; display: flex; flex-direction: column; gap: 6px; min-width: 0; }
.world-header { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; min-height: 28px; }
.world-header .chip { max-width: 14em; }
body[data-compare] #grid { max-width: 100%; }
.grid-b { width: 100%; aspect-ratio: 1; image-rendering: pixelated; border-radius: 6px; touch-action: none; cursor: crosshair; }
.copy-progress { margin: 0; padding: 24px 0; text-align: center; font-variant-numeric: tabular-nums; }
.chips { display: contents; }
.compare-toggle { margin-left: -8px; }
body[data-view='experiments'] .compare-toggle { display: none; }
dialog.keep { border: 1px solid var(--border); border-radius: 8px; background: var(--surface); color: var(--text); padding: 16px 18px; }
dialog.keep::backdrop { background: rgb(0 0 0 / 0.4); }
dialog.keep p { margin: 0 0 12px; }
dialog.keep .row { display: flex; gap: 8px; justify-content: flex-end; }
```

- [ ] **Step 5: Wire Compare into the page**

Replace `web/src/main.ts` with:
```ts
import './style.css';
import { askKeep, CompareView, compareShell, type Playground, type WorldName } from './compare/compare-view';
import { copyWorld } from './compare/lockstep';
import { canvasBlob, downloadBlob, downloadText } from './downloads';
import { Engine } from './engine';
import { ExperimentsView } from './experiments/view';
import { LOG_FULL_NOTICE, sessionLink, shareable } from './sessions';
import { decodeShare, decodeSweep, parseSessionFile, readHash, readSweepHash, sessionFileText } from './share';
import { ChartsPanel } from './ui/charts-panel';
import { CreditPanel } from './ui/credit-panel';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { buildExportMenu } from './ui/export-menu';
import { GridView } from './ui/grid-view';
import { InspectPanel } from './ui/inspect-panel';
import { showNotice } from './ui/notice';
import { buildRecordControl } from './ui/record-control';
import { RulesPanel } from './ui/rules-panel';
import { buildShareMenu } from './ui/share-menu';
import { Tabs } from './ui/tabs';
import { Toolbar } from './ui/toolbar';
import { buildTools } from './ui/tools';

export function showBanner(message: string, action?: { label: string; run: () => void }): void {
  const banner = document.querySelector<HTMLElement>('#banner')!;
  banner.replaceChildren(
    ...[
      h('span', {}, message),
      action ? h('button', { onclick: action.run }, action.label) : null,
      h('button', { onclick: () => (banner.hidden = true), 'aria-label': 'Dismiss' }, '×'),
    ].filter((child): child is HTMLElement => child !== null),
  );
  banner.hidden = false;
}

const message = (e: unknown): string => (e instanceof Error ? e.message : String(e));

async function main(): Promise<void> {
  let engine: Engine;
  const token = readHash();
  try {
    // A link's session replays its edits as the world runs (Decision 2).
    engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
  } catch (e) {
    showBanner(`That share link could not be loaded (${message(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
  /** Compare mode's second world and its coordinator, while Compare is on. */
  let compare: CompareView | null = null;
  // Browser checks drive the engines through this handle (7a Decision 14).
  if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { sugarscape: { engine, compare: () => compare } });
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  const toolbar = new Toolbar(engine);
  document.querySelector('#toolbar')!.append(toolbar.el);
  document.querySelector('#display')!.append(buildDisplay(engine));
  const experiments = new ExperimentsView(engine);
  document.querySelector('#experiments')!.append(experiments.el);
  const views = { playground: 'Playground', experiments: 'Experiments' } as const;
  type View = keyof typeof views;
  const viewButtons = (Object.keys(views) as View[]).map((view) =>
    h('button', { 'data-view': view, onclick: () => showView(view) }, views[view]),
  );
  const showView = (view: View): void => {
    // The playground's worlds are kept, paused, while Experiments is shown.
    if (view === 'experiments') (compare?.lock ?? engine).setRunning(false);
    document.body.dataset.view = view;
    document.querySelector<HTMLElement>('#playground')!.hidden = view !== 'playground';
    document.querySelector<HTMLElement>('#experiments')!.hidden = view !== 'experiments';
    for (const b of viewButtons) b.setAttribute('aria-pressed', String(b.dataset.view === view));
  };
  const compareButton = h(
    'button',
    {
      class: 'compare-toggle',
      'aria-pressed': 'false',
      title: 'Run a copy of this world beside it, both stepped in lockstep',
      onclick: () => void toggleCompare(),
    },
    'Compare',
  );
  document
    .querySelector('.toolbar h1')!
    .after(h('div', { class: 'view-switch', role: 'group', 'aria-label': 'View' }, ...viewButtons), compareButton);
  showView('playground');

  const sweepToken = readSweepHash();
  if (sweepToken) {
    try {
      experiments.openSweep(await decodeSweep(sweepToken));
      showView('experiments');
    } catch (e) {
      showBanner(`That experiment link could not be loaded (${message(e)}).`);
    }
  }

  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  tabs.add('Rules', new RulesPanel(engine).el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new InspectPanel(engine);
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  const credit = new CreditPanel(engine, () => tabs.show('Inspect'));
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on in a world on screen.
  const syncCreditTab = (b: Engine | null) => tabs.setHidden('Credit', ![engine, b].some((e) => e?.config.credit.enabled));
  engine.on('reset', () => syncCreditTab(compare?.b ?? null));
  engine.on('config', () => syncCreditTab(compare?.b ?? null));
  syncCreditTab(null);
  document.querySelector('#tools')!.append(buildTools(engine, grid, () => tabs.show('Inspect')));

  const slug = () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}-t${engine.tick}`;
  const exportMenu = buildExportMenu({
    worlds: () => [{ label: '', engine, grid }],
    slug: () => slug(),
    charts: async () => {
      tabs.show('Charts');
      await engine.refresh();
      // Let the panel draw the fresh snapshot before the canvases are captured.
      await new Promise((resolve) => requestAnimationFrame(resolve));
      for (const { name, canvas } of charts.canvases()) {
        downloadBlob(`${slug()}-${name.toLowerCase().replace(/\W+/g, '-')}.png`, await canvasBlob(canvas));
      }
    },
    session: async () => {
      const { state, full } = await shareable(engine);
      if (full) showNotice(LOG_FULL_NOTICE, 10_000);
      downloadText(`${slug()}-session.json`, sessionFileText({ kind: 'session', state }), 'application/json');
    },
  });
  const shareMenu = buildShareMenu({
    link: () => sessionLink(engine),
    open: async (file) => {
      try {
        const opened = parseSessionFile(await file.text());
        if (opened.kind !== 'session') throw new Error('it holds a comparison, which this page cannot open yet');
        if (compare) await leaveCompare('A');
        const errors = await engine.open(opened.state);
        if (errors) throw new Error(errors.map((x) => `${x.field}: ${x.message}`).join('; '));
        // The address bar no longer describes this world.
        history.replaceState(null, '', location.pathname + location.search);
        showNotice(`Opened ${file.name}`);
      } catch (e) {
        showNotice(`${file.name} could not be opened (${message(e)})`, 10_000);
      }
    },
  });
  const record = buildRecordControl({
    grids: () => [{ canvas: grid.canvas, cells: () => engine.size() }],
    tick: () => engine.tick,
    running: () => (compare?.lock ?? engine).running,
    base: () => `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}`,
  });
  engine.on('run', () => record.sync());
  document.querySelector('.toolbar-end')!.append(record.el, shareMenu, exportMenu);

  const crashed = () => showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
  engine.on('crash', crashed);
  engine.on('fork', () => showNotice('Replay ended — your edit starts a new branch'));

  let dirty = true;
  /** A snapshot arrived since the last frame (Compare: a lockstep pair): once drawn, the recording captures it. */
  let fresh = false;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  engine.on('snapshot', () => {
    if (!compare) fresh = true;
  });
  const playground: Playground = {
    engine,
    grid,
    toolbar,
    syncCreditTab,
    onRun: () => record.sync(),
    onFrame: () => (fresh = true),
    onCrash: crashed,
  };

  let entering = false;
  const syncCompareButton = () => {
    compareButton.setAttribute('aria-pressed', String(compare !== null));
    compareButton.disabled = entering;
  };
  /** Starts Compare with B a copy of A at its current tick (Decision 9). */
  async function enterCompare(): Promise<void> {
    if (compare || entering) return;
    entering = true;
    syncCompareButton();
    engine.setRunning(false);
    toolbar.hold(true);
    const shell = compareShell();
    try {
      const { session, full, tick } = await engine.session();
      if (full) throw new Error('A’s edit log is full (50 000 edits), so B cannot copy it exactly');
      const b = await copyWorld(session, tick, (s) => Engine.create(s), (at, of) => {
        shell.progress.textContent = `Copying A… ${at} / ${of}`;
      });
      compare = new CompareView(playground, b, shell);
    } catch (e) {
      shell.figure.remove();
      delete document.body.dataset.compare;
      showNotice(`Compare could not start (${message(e)})`, 10_000);
    } finally {
      entering = false;
      toolbar.hold(false);
      syncCompareButton();
    }
  }
  async function leaveCompare(keep: WorldName): Promise<void> {
    const c = compare;
    if (!c) return;
    compare = null;
    await c.leave(keep);
    syncCompareButton();
  }
  async function toggleCompare(): Promise<void> {
    if (!compare) return enterCompare();
    const keep = await askKeep();
    if (keep) await leaveCompare(keep);
  }

  const loop = (now: number) => {
    try {
      (compare?.lock ?? engine).pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
      compare?.draw();
      if (fresh) {
        fresh = false;
        record.capture();
      }
    } catch (e) {
      // A Rust panic in the page's WASM leaves it unusable; reloading keeps any #s= share state.
      (compare?.lock ?? engine).setRunning(false);
      console.error(e);
      crashed();
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}

main().catch((e) => showBanner(`Failed to start: ${message(e)}`));
```

- [ ] **Step 6: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.
Run: `grep -rn "buildToolbar" web/src`
Expected: no output.

- [ ] **Step 7: Commit**

```bash
git add web/src/engine.ts web/src/ui/toolbar.ts web/src/compare/compare-view.ts web/src/main.ts web/index.html web/src/style.css
git commit -m "Add Compare: B copied from A, stepped in lockstep, with Keep A / Keep B" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 11: Editing in Compare — Rules for A | B, per-world Inspect and Credit, tools per grid

*Needs judgment (full code given; the tools rewrite must keep every single-world behavior).* Browser (controller), `/?debug`:
- Single mode first (regression): every tool on `ii-2-unit`, `v-2-endemic` (Infect/Vaccinate picker fills and grows, also paused) and `n-3-trade` (paint good picker, image import); Inspect, Follow, Credit tab on `iv-5-credit` — all as before; the Rules tab shows no switch and Inspect no label.
- In Compare on `v-2-endemic`: the Rules tab shows "Rules for: A | B"; with B selected, a live change (growback rate) applies to B only and the worlds diverge (fingerprints differ) while ticks stay equal; a reset-requiring change on B (Width) rebuilds B and rewinds A to t = 0; a preset change on B does the same.
- Paint on B's grid changes only B; Place/Erase/Infect/Vaccinate on each grid edit that grid's world; Inspect on B's grid shows "World B" and B's site; clicking A's grid switches Inspect back to "World A"; the disease picker lists the clicked world's diseases.
- On `iv-5-credit` vs a B with credit off (Rules for B → Credit off): the Credit tab stays while A has credit; with A's credit off too it disappears; Credit shows the last-clicked world, and a node click opens Inspect on that world.
- Follow an agent in B (Inspect → Follow): B's header shows its follow chip; ✕ stops it. Keep B afterwards: the Rules, Inspect and Credit tabs show B's world and have no switch/label.

**Files:**
- Create: `web/src/ui/world-slot.ts`
- Rewrite: `web/src/ui/tools.ts`
- Modify: `web/src/compare/compare-view.ts`, `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `CompareView`, `Playground`, `WorldName` (Task 10); `RulesPanel`, `InspectPanel`, `CreditPanel`, `Tabs`, `GridView`, `CellEvent`, `DiseaseListPoll`, `diseaseOptions` (existing).
- Produces:
  - `ui/world-slot.ts`: `class WorldSlot<P extends { el: HTMLElement; setVisible?(visible: boolean): void }> { constructor(a: P, kind: 'switch' | 'label', title?: string); readonly el: HTMLElement; readonly a: P; setB(b: P | null): void; show(which: WorldName): void; setVisible(visible: boolean): void }`
  - `ui/tools.ts`: `interface ToolTarget { engine: Engine; grid: GridView }`; `interface Tools { readonly el: HTMLElement; attach(target: ToolTarget): () => void; focus(engine: Engine): void }`; `buildTools(primary: ToolTarget, onInspect: (engine: Engine) => void): Tools`
  - `compare/compare-view.ts`: `Playground` gains `tools: Tools; tabs: Tabs; rules: WorldSlot<RulesPanel>; inspect: WorldSlot<InspectPanel>; credit: WorldSlot<CreditPanel>`; `CompareView.focus(world: WorldName): void`

- [ ] **Step 1: WorldSlot**

Create `web/src/ui/world-slot.ts`:
```ts
import type { WorldName } from '../compare/compare-view';
import { h } from './dom';

interface SlotPanel { el: HTMLElement; setVisible?(visible: boolean): void }

/**
 * A tab's content for one world, or for A and B in Compare (Decision 10): the chosen world's panel
 * shows, under a "Rules for: A | B" switch (`kind: 'switch'`) or a "World A" label (`'label'`,
 * which follows the world last clicked). Outside Compare only A's panel shows, with no header.
 */
export class WorldSlot<P extends SlotPanel> {
  readonly el: HTMLElement;
  private readonly head: HTMLElement;
  private readonly buttons: HTMLButtonElement[] = [];
  private b: P | null = null;
  private which: WorldName = 'A';
  private visible = false;

  constructor(
    readonly a: P,
    private readonly kind: 'switch' | 'label',
    title = '',
  ) {
    if (kind === 'switch') {
      this.buttons = (['A', 'B'] as const).map((w) => h('button', { onclick: () => this.show(w) }, w));
      this.head = h('div', { class: 'world-switch', role: 'group', 'aria-label': title }, h('span', {}, `${title}:`), ...this.buttons);
    } else {
      this.head = h('p', { class: 'world-label' });
    }
    this.head.hidden = true;
    this.el = h('div', { class: 'world-slot' }, this.head, a.el);
  }

  /** Compare's B panel, or null to go back to A alone. */
  setB(b: P | null): void {
    this.b?.el.remove();
    this.b = b;
    if (b) this.el.append(b.el);
    this.head.hidden = b === null;
    this.show(b ? this.which : 'A');
  }

  show(which: WorldName): void {
    this.which = this.b ? which : 'A';
    this.a.el.hidden = this.which !== 'A';
    if (this.b) this.b.el.hidden = this.which !== 'B';
    if (this.kind === 'label') this.head.textContent = `World ${this.which}`;
    for (const button of this.buttons) button.setAttribute('aria-pressed', String(button.textContent === this.which));
    this.a.setVisible?.(this.visible && this.which === 'A');
    this.b?.setVisible?.(this.visible && this.which === 'B');
  }

  /** The tab was shown or hidden. */
  setVisible(visible: boolean): void {
    this.visible = visible;
    this.show(this.which);
  }
}
```

- [ ] **Step 2: Tools per grid**

Replace `web/src/ui/tools.ts` with:
```ts
import type { Engine, PlaceOverrides } from '../engine';
import { capacitiesFromPixels } from '../image';
import type { DiseaseEntry } from '../types';
import { DiseaseListPoll, diseaseOptions } from './disease-picker';
import { h } from './dom';
import type { CellEvent, GridView } from './grid-view';
import { readImagePixels } from './image-import';

type Tool = 'inspect' | 'paint' | 'place' | 'erase' | 'infect' | 'vaccinate';

const TOOLS: [Tool, string][] = [
  ['inspect', 'Inspect'],
  ['paint', 'Paint capacity'],
  ['place', 'Place agent'],
  ['erase', 'Erase agent'],
  ['infect', 'Infect'],
  ['vaccinate', 'Vaccinate'],
];

/** Tools that exist only while disease is on (in some world on screen). */
const DISEASE_TOOLS: Tool[] = ['infect', 'vaccinate'];

/** A world's disease list is fetched at most this often while a disease tool is open. */
const LIST_MS = 250;

/** A grid and the world its clicks edit. */
export interface ToolTarget { engine: Engine; grid: GridView }

export interface Tools {
  readonly el: HTMLElement;
  /** Routes `target.grid`'s clicks to `target.engine` (Compare's B); returns the detach. */
  attach(target: ToolTarget): () => void;
  /** The world whose diseases the picker lists and whose map an image import sets (the grid last clicked). */
  focus(engine: Engine): void;
}

/**
 * Tool picker; routes each grid's clicks/drags to the active tool on that grid's world (Decision 10).
 * Edit errors (e.g. an occupied site) are ignored. Display changes (the paint layer, the disease
 * colors) and the paint tool's goods follow `primary`, whose display Compare mirrors to B.
 */
export function buildTools(primary: ToolTarget, onInspect: (engine: Engine) => void): Tools {
  let tool: Tool = 'inspect';
  let radius = 1;
  let value = 4;
  let good = 0;
  let sex: '' | 'female' | 'male' = '';
  let tribe: '' | 'blue' | 'red' = '';
  /** Selected disease id; −1 is a new random disease (Infect only). */
  let disease = -1;
  /** Length of the disease list the picker was last filled from. */
  let known = -1;
  const targets: ToolTarget[] = [];
  /** Each world's latest disease list (while a disease tool is open) and when to ask again. */
  const lists = new Map<Engine, { diseases: DiseaseEntry[]; poll: DiseaseListPoll }>();
  let focused = primary.engine;
  const diseases = (): DiseaseEntry[] => lists.get(focused)?.diseases ?? [];
  const brushed = () => tool === 'paint' || tool === 'vaccinate';
  const diseaseOn = () => targets.some((t) => t.engine.config.disease.enabled);
  const grids = () => targets.map((t) => t.grid);

  const buttons = TOOLS.map(([t, label]) => h('button', { onclick: () => choose(t) }, label));
  const options = h('div', { class: 'tool-options' });
  const picker = h('select', { onchange: () => (disease = Number(picker.value)) });

  const number = (label: string, min: number, max: number, get: () => number, set: (v: number) => void) => {
    const input = h('input', { type: 'number', min, max, value: get(), class: 'num' });
    input.addEventListener('change', () => {
      set(Math.min(max, Math.max(min, Number(input.value))));
      input.value = String(get());
      if (brushed()) for (const g of grids()) g.brushRadius = radius;
    });
    return h('label', {}, `${label} `, input);
  };
  /** The paint tool's good picker, refilled only when the goods' names change. */
  const goodSelect = h('select', {
    onchange: () => {
      good = Number(goodSelect.value);
      primary.engine.setDisplay({ layer: `capacity:${good}` });
    },
  });
  const goodLabel = h('label', {}, 'Good ', goodSelect);
  let goodNames = '';
  function refreshGoods(): void {
    const names = primary.engine.config.goods.map((g) => g.name);
    const signature = JSON.stringify(names);
    if (good >= names.length) good = 0;
    if (signature !== goodNames) {
      goodNames = signature;
      goodSelect.replaceChildren(...names.map((name, i) => h('option', { value: String(i) }, name)));
    }
    goodSelect.value = String(good);
  }

  /** Image import (Decision 12 of milestone 6): for the paint tool's good, into the focused world. */
  let importMax = 4;
  let invert = false;
  const importStatus = h('span', { class: 'hint', 'aria-live': 'polite' });
  const fileInput = h('input', { type: 'file', accept: 'image/*', hidden: true, 'aria-label': 'Image to import' });
  fileInput.addEventListener('change', async () => {
    const file = fileInput.files?.[0];
    fileInput.value = '';
    if (!file) return;
    const engine = focused;
    try {
      const { width, height } = engine.size();
      const capacities = capacitiesFromPixels(await readImagePixels(file, width, height), importMax, invert);
      const errors = await engine.importLandscape(good, capacities);
      importStatus.textContent = errors ? errors.map((e) => e.message).join('; ') : `Imported ${file.name}`;
    } catch (e) {
      importStatus.textContent = `Could not read ${file.name}: ${e instanceof Error ? e.message : String(e)}`;
    }
  });
  const invertBox = h('input', { type: 'checkbox', onchange: () => (invert = invertBox.checked) });
  const importControls = h(
    'span',
    { class: 'tool-options' },
    h('button', { onclick: () => fileInput.click(), title: 'Set the good’s capacities from an image’s brightness' }, 'Import image…'),
    fileInput,
    number('Max', 0, 10, () => importMax, (v) => (importMax = Math.round(v))),
    h('label', {}, invertBox, ' Invert'),
    importStatus,
  );

  const select = <T extends string>(label: string, values: [T, string][], set: (v: T) => void) => {
    const s = h('select', {}, ...values.map(([v, l]) => h('option', { value: v }, l)));
    s.addEventListener('change', () => set(s.value as T));
    return h('label', {}, `${label} `, s);
  };

  /** Refills the picker when the focused world's disease list has grown, or when forced. */
  function refreshPicker(force = false): void {
    if (!DISEASE_TOOLS.includes(tool) || !diseaseOn()) return;
    const list = diseases();
    if (!force && list.length === known) return;
    known = list.length;
    picker.replaceChildren(...diseaseOptions(list, tool === 'infect').map(([v, l]) => h('option', { value: v }, l)));
    const values = Array.from(picker.options, (o) => Number(o.value));
    if (!values.includes(disease)) disease = values[0] ?? -1;
    picker.value = String(disease);
  }

  function choose(next: Tool): void {
    tool = next;
    buttons.forEach((b, i) => b.setAttribute('aria-pressed', String(TOOLS[i][0] === tool)));
    for (const g of grids()) g.brushRadius = brushed() ? radius : null;
    if (tool === 'paint') {
      refreshGoods();
      primary.engine.setDisplay({ layer: `capacity:${good}` });
    }
    if (DISEASE_TOOLS.includes(tool)) primary.engine.setDisplay({ colorMode: 'disease' });
    const pick = h('label', {}, 'Disease ', picker);
    options.replaceChildren(
      ...(tool === 'paint'
        ? [goodLabel, number('Radius', 0, 10, () => radius, (v) => (radius = v)), number('Capacity', 0, 10, () => value, (v) => (value = v)), importControls]
        : tool === 'place'
          ? [
              select('Sex', [['', 'Random'], ['female', 'Female'], ['male', 'Male']], (v) => (sex = v)),
              select('Tribe', [['', 'Random'], ['blue', 'Blue'], ['red', 'Red']], (v) => (tribe = v)),
            ]
          : tool === 'infect'
            ? [pick, h('span', { class: 'hint' }, 'Click an agent to infect it.')]
            : tool === 'vaccinate'
              ? [number('Radius', 0, 10, () => radius, (v) => (radius = v)), pick]
              : tool === 'inspect'
                ? [h('span', { class: 'hint' }, 'Click an agent or site.')]
                : [h('span', { class: 'hint' }, 'Click or drag over agents to remove them.')]),
    );
    refreshPicker(true);
    if (DISEASE_TOOLS.includes(tool)) {
      // Fetch the lists now, even while paused.
      for (const t of targets) {
        lists.get(t.engine)?.poll.expedite();
        void t.engine.refresh();
      }
    }
    for (const g of grids()) g.draw();
  }

  /** Hides the disease tools while no world on screen has disease (leaving them if one was active). */
  function syncAvailability(): void {
    const on = diseaseOn();
    buttons.forEach((b, i) => {
      if (DISEASE_TOOLS.includes(TOOLS[i][0])) b.hidden = !on;
    });
    if (!on && DISEASE_TOOLS.includes(tool)) choose('inspect');
    else refreshPicker(true);
    // Keeps the paint tool's layer and inputs; only the good list follows the config.
    refreshGoods();
  }

  /** A grid's clicks and drags, on its own world. */
  const route = (engine: Engine) => (x: number, y: number, kind: CellEvent) => {
    switch (tool) {
      case 'inspect':
        if (kind === 'down') {
          void engine.select(x, y);
          onInspect(engine);
        }
        break;
      case 'paint':
        void engine.paint(x, y, radius, value, good);
        break;
      case 'place':
        if (kind === 'down') {
          const o: PlaceOverrides = {};
          if (sex) o.sex = sex;
          if (tribe) o.tribe = tribe;
          void engine.place(x, y, o);
        }
        break;
      case 'erase':
        void engine.erase(x, y);
        break;
      case 'infect':
        if (kind === 'down') void engine.infect(x, y, disease);
        break;
      case 'vaccinate':
        void engine.vaccinate(x, y, radius, disease);
        break;
    }
  };

  function focus(engine: Engine): void {
    if (focused === engine) return;
    focused = engine;
    refreshPicker(true);
  }

  function attach(target: ToolTarget): () => void {
    const { engine, grid } = target;
    const entry = { diseases: [] as DiseaseEntry[], poll: new DiseaseListPoll(LIST_MS) };
    targets.push(target);
    lists.set(engine, entry);
    grid.onCell = route(engine);
    grid.brushRadius = brushed() ? radius : null;
    const offs = [
      engine.on('reset', syncAvailability),
      engine.on('config', syncAvailability),
      // A reset can change which diseases exist: drop the stale list and ask for a fresh one promptly.
      engine.on('reset', () => {
        entry.poll.expedite();
        entry.diseases = [];
        if (engine === focused) refreshPicker();
      }),
      // Infect, Vaccinate and a rule change can change the list without a tick.
      engine.on('edit', () => entry.poll.invalidate()),
      engine.on('config', () => entry.poll.invalidate()),
      engine.want((now) =>
        DISEASE_TOOLS.includes(tool) && engine.config.disease.enabled && entry.poll.due(now, engine.tick) ? { diseaseList: true } : {},
      ),
      engine.on('snapshot', () => {
        const list = engine.last?.diseaseList;
        if (!list) return;
        entry.diseases = list;
        entry.poll.received(performance.now(), engine.tick);
        if (engine === focused) refreshPicker();
      }),
    ];
    syncAvailability();
    return () => {
      for (const off of offs) off();
      targets.splice(targets.indexOf(target), 1);
      lists.delete(engine);
      grid.onCell = null;
      grid.brushRadius = null;
      if (focused === engine) focus(primary.engine);
      syncAvailability();
    };
  }

  const el = h('div', { class: 'tools' }, h('div', { class: 'tool-buttons' }, ...buttons), options);
  attach(primary);
  choose('inspect');
  syncAvailability();
  return { el, attach, focus };
}
```

- [ ] **Step 3: Per-world panels in the Compare view**

In `web/src/compare/compare-view.ts`:
- replace the imports with:
```ts
import { randomSeed, type Engine } from '../engine';
import { CreditPanel } from '../ui/credit-panel';
import { h } from '../ui/dom';
import { GridView } from '../ui/grid-view';
import { InspectPanel } from '../ui/inspect-panel';
import { showNotice } from '../ui/notice';
import { RulesPanel } from '../ui/rules-panel';
import type { Tabs } from '../ui/tabs';
import { followChip, replayChip, type Toolbar } from '../ui/toolbar';
import type { Tools } from '../ui/tools';
import type { WorldSlot } from '../ui/world-slot';
import { Lockstep } from './lockstep';
```
- in `interface Playground`, after `toolbar: Toolbar;` add:
```ts
  tools: Tools;
  tabs: Tabs;
  rules: WorldSlot<RulesPanel>;
  inspect: WorldSlot<InspectPanel>;
  credit: WorldSlot<CreditPanel>;
```
- in the constructor, replace `p.toolbar.setCompare(this.lock, b);` with:
```ts
    p.toolbar.setCompare(this.lock, b);
    // Each world has its own Rules, Inspect and Credit panels; tools act on the grid clicked (Decision 10).
    this.offs.push(p.tools.attach({ engine: b, grid: this.gridB }));
    p.rules.setB(new RulesPanel(b));
    p.inspect.setB(new InspectPanel(b));
    p.credit.setB(
      new CreditPanel(b, () => {
        this.focus('B');
        p.tabs.show('Inspect');
      }),
    );
    this.listen(p.grid.canvas, 'A');
    this.listen(this.gridB.canvas, 'B');
```
  and replace the constructor's last two lines (`p.syncCreditTab(b);` and `p.onRun();`) with:
```ts
    p.syncCreditTab(b);
    this.focus('A');
    p.onRun();
```
- in `leave`, replace `p.toolbar.setCompare(null, null);` with:
```ts
    p.toolbar.setCompare(null, null);
    p.rules.setB(null);
    p.inspect.setB(null);
    p.credit.setB(null);
    p.tools.focus(a);
```
- after `draw()` add:
```ts
  /** Inspect and Credit show the world last clicked; the disease picker and image import follow it. */
  focus(world: WorldName): void {
    this.p.inspect.show(world);
    this.p.credit.show(world);
    this.p.tools.focus(world === 'A' ? this.p.engine : this.b);
  }

  /** A pointerdown on a world's grid focuses that world. */
  private listen(canvas: HTMLCanvasElement, world: WorldName): void {
    const on = () => this.focus(world);
    canvas.addEventListener('pointerdown', on);
    this.offs.push(() => canvas.removeEventListener('pointerdown', on));
  }
```

- [ ] **Step 4: The page**

In `web/src/main.ts`:
- add `import { WorldSlot } from './ui/world-slot';` after the `buildTools` import.
- replace the block from `tabs.add('Rules', new RulesPanel(engine).el);` through `document.querySelector('#tools')!.append(buildTools(engine, grid, () => tabs.show('Inspect')));` with:
```ts
  // Each tab holds A's panel, and B's beside it in Compare (Decision 10).
  const rules = new WorldSlot(new RulesPanel(engine), 'switch', 'Rules for');
  tabs.add('Rules', rules.el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new WorldSlot(new InspectPanel(engine), 'label');
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  const credit = new WorldSlot(
    new CreditPanel(engine, () => {
      compare?.focus('A');
      tabs.show('Inspect');
    }),
    'label',
  );
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on in a world on screen.
  const syncCreditTab = (b: Engine | null) => tabs.setHidden('Credit', ![engine, b].some((e) => e?.config.credit.enabled));
  engine.on('reset', () => syncCreditTab(compare?.b ?? null));
  engine.on('config', () => syncCreditTab(compare?.b ?? null));
  syncCreditTab(null);
  const tools = buildTools({ engine, grid }, (world) => {
    compare?.focus(world === engine ? 'A' : 'B');
    tabs.show('Inspect');
  });
  document.querySelector('#tools')!.append(tools.el);
```
- in the `playground` object literal, after `toolbar,` add:
```ts
    tools,
    tabs,
    rules,
    inspect,
    credit,
```

Append to `web/src/style.css`:
```css
.world-switch { display: flex; gap: 6px; align-items: center; margin: 0 0 10px; }
.world-label { margin: 0 0 8px; font-weight: 600; color: var(--muted); }
```

- [ ] **Step 5: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass (the engine test's `DiseaseListPoll` use is unchanged).

- [ ] **Step 6: Commit**

```bash
git add web/src/ui/world-slot.ts web/src/ui/tools.ts web/src/compare/compare-view.ts web/src/main.ts web/src/style.css
git commit -m "Edit each world in Compare: Rules for A | B, per-world Inspect and Credit, tools per grid" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 12: Overlaid charts for two worlds

*Needs judgment (the Charts panel is rewritten table-driven; full code given — check each chart still appears under the same conditions and draws the same data in single mode).* Browser (controller):
- Single mode (regression, as 7a Task 10): Charts on `ii-2-unit` (time charts with real ticks; Lorenz and wealth bars update ~4×/s and after a paint while paused), `iv-3-trade` (price band with gaps, supply & demand), `iv-5-credit` (loans, debt), `v-2-endemic` (disease section), `n-3-trade` (goods section follows the goods), `iii-6-three-tribes` (group shares), a live change that adds a good (charts rebuild and fill at once, paused too), a 20 000-tick run (smooth, x reaches the tick), Export → Charts (PNG).
- Compare on `ii-2-unit`, B diverged by 🎲: every time chart shows A solid and B dashed, legends "A · Agents" / "B · Agents"; Lorenz shows equality plus both curves; the wealth histogram shows two step outlines; the lines keep up at Max.
- Compare on `iv-3-trade` vs B with Trade off (Rules for B): the Economy charts still show (A has two goods), A's lines draw and B's price lines have gaps/are flat; supply & demand overlays both markets on the log price axis.
- A with disease off and B with disease on (`v-2-endemic` in B via its preset select): the Disease section appears.
- Keep A or Keep B: the charts return to single-world form (no "A ·" prefixes) showing the kept world; hovering shows values.

**Files:**
- Modify: `web/src/ui/series-data.ts`, `web/src/compare/compare-view.ts`, `web/src/main.ts`
- Test: `web/src/ui/series-data.test.ts`
- Rewrite: `web/src/ui/charts-panel.ts`

**Interfaces:**
- Consumes: `ChartGroup`, `CHART_POINTS`, `Wants` (7a); `Engine.chartGroup/want/on/last/tick/config`; `lineData`, `bandData`, `chartsBehind` (7a); `Playground`, `CompareView` (Tasks 10–11).
- Produces:
  - `ui/series-data.ts`: `type OverlayData = [number[], ...(number | null | undefined)[][]]`; `overlayData(tables: LineData[]): OverlayData`; `histTable(hist: Float64Array | null | undefined): LineData`; `barsData(hist: Float64Array | null | undefined): LineData`; `supplyDemandTable(sd: Float64Array | null | undefined): LineData`; `emptyTable(lines: number): LineData`
  - `ui/charts-panel.ts`: `class ChartsPanel { constructor(engine: Engine); readonly el: HTMLElement; setVisible(visible: boolean): void; setCompare(b: Engine | null): void; canvases(): { name: string; canvas: HTMLCanvasElement }[] }`
  - `compare/compare-view.ts`: `Playground` gains `charts: ChartsPanel`

- [ ] **Step 1: Write the failing tests**

In `web/src/ui/series-data.test.ts`, change the series-data import to:
```ts
import { bandData, barsData, chartsBehind, emptyTable, histTable, lineData, overlayData, supplyDemandTable, type LineData } from './series-data';
```
and append:
```ts
describe('two worlds on one axis', () => {
  it('puts tables on the union of their x values: holes are undefined (drawn through), gaps stay null', () => {
    const a: LineData = [[0, 2, 4], [1, null, 3]];
    const b: LineData = [[0, 3, 4], [5, 6, 7], [8, 9, 10]];
    expect(overlayData([a, b])).toEqual([
      [0, 2, 3, 4],
      [1, null, undefined, 3],
      [5, undefined, 6, 7],
      [8, undefined, 9, 10],
    ]);
  });

  it('gives a missing group an empty table with the right number of lines', () => {
    expect(emptyTable(2)).toEqual([[], [], []]);
    expect(overlayData([emptyTable(1), [[1, 2], [5, 6]]])).toEqual([[1, 2], [undefined, undefined], [5, 6]]);
  });

  it('draws a histogram as bars or as a step outline over its bin edges', () => {
    const hist = Float64Array.of(2, 5, 0, 1);
    expect(barsData(hist)).toEqual([[1, 3, 5], [5, 0, 1]]);
    expect(histTable(hist)).toEqual([[0, 2, 4, 6], [5, 0, 1, 0]]);
    expect(histTable(null)).toEqual([[], []]);
  });

  it('turns supply and demand into two curves and two marked points', () => {
    const sd = Float64Array.of(3, 0.5, 1, 2, 9, 5, 1, 1, 5, 9, 1.1, 5, 2, 9);
    expect(supplyDemandTable(sd)).toEqual([
      [0.5, 1, 2],
      [9, 5, 1],
      [1, 5, 9],
      [null, 5, null],
      [null, null, 9],
    ]);
    const none = Float64Array.of(3, 0.5, 1, 2, 9, 5, 1, 1, 5, 9, NaN, NaN, NaN, NaN);
    expect(supplyDemandTable(none).slice(3)).toEqual([
      [null, null, null],
      [null, null, null],
    ]);
    expect(supplyDemandTable(undefined)).toEqual([[], [], [], [], []]);
  });
});
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `(cd web && npx vitest run src/ui/series-data.test.ts)`
Expected: FAIL — `overlayData`, `histTable`, `barsData`, `supplyDemandTable`, `emptyTable` are not exported.

- [ ] **Step 3: The data helpers**

Append to `web/src/ui/series-data.ts`:
```ts
/** uPlot data for several worlds on one x axis: `undefined` is a hole uPlot draws through, `null` a gap. */
export type OverlayData = [number[], ...(number | null | undefined)[][]];

/**
 * Several tables' lines on the sorted union of their x values (Decision 11): each line is
 * `undefined` where its own table has no point (so two worlds' different downsampled ticks, or
 * price grids, share an axis and each line is drawn through the other's points) and keeps `null`
 * where its data has a gap.
 */
export function overlayData(tables: LineData[]): OverlayData {
  const xs = [...new Set(tables.flatMap((t) => t[0]))].sort((p, q) => p - q);
  const at = new Map(xs.map((x, i) => [x, i]));
  const out: OverlayData = [xs];
  for (const [x, ...lines] of tables) {
    for (const line of lines) {
      const column: (number | null | undefined)[] = new Array(xs.length).fill(undefined);
      line.forEach((v, i) => (column[at.get(x[i])!] = v));
      out.push(column);
    }
  }
  return out;
}

/** A table with no points yet and `lines` lines (a world whose group has not arrived). */
export function emptyTable(lines: number): LineData {
  return [[], ...Array.from({ length: lines }, () => [])] as LineData;
}

/** The wealth histogram `[binWidth, counts…]` as bars: bin centers and counts. */
export function barsData(hist: Float64Array | null | undefined): LineData {
  if (!hist) return [[], []];
  const width = hist[0];
  const counts = Array.from(hist.subarray(1));
  return [counts.map((_, i) => (i + 0.5) * width), counts];
}

/** The wealth histogram as a step outline: each bin's left edge and count, closed at the right edge. */
export function histTable(hist: Float64Array | null | undefined): LineData {
  if (!hist) return [[], []];
  const width = hist[0];
  const counts = Array.from(hist.subarray(1));
  return [Array.from({ length: counts.length + 1 }, (_, k) => k * width), [...counts, 0]];
}

/**
 * Supply & demand `[n, prices(n), demand(n), supply(n), eqP, eqQ, actP, actQ]` as price, demand,
 * supply, and the equilibrium and actual points marked at their nearest price.
 */
export function supplyDemandTable(sd: Float64Array | null | undefined): LineData {
  if (!sd) return [[], [], [], [], []];
  const n = sd[0];
  const prices = Array.from(sd.subarray(1, 1 + n));
  const demand = Array.from(sd.subarray(1 + n, 1 + 2 * n));
  const supply = Array.from(sd.subarray(1 + 2 * n, 1 + 3 * n));
  const [eqP, eqQ, actP, actQ] = Array.from(sd.subarray(1 + 3 * n));
  const nearest = (p: number) =>
    prices.reduce((best, q, i) => (Math.abs(Math.log(q / p)) < Math.abs(Math.log(prices[best] / p)) ? i : best), 0);
  const point = (p: number, q: number) => {
    const column: (number | null)[] = prices.map(() => null);
    if (Number.isFinite(p) && Number.isFinite(q)) column[nearest(p)] = q;
    return column;
  };
  return [prices, demand, supply, point(eqP, eqQ), point(actP, actQ)];
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npx vitest run src/ui/series-data.test.ts)`
Expected: all pass (the 4 new tests and the 6 earlier ones).

- [ ] **Step 5: The panel, table-driven, for one or two worlds**

Replace `web/src/ui/charts-panel.ts` with:
```ts
import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type { Engine } from '../engine';
import { CHART_POINTS, type ChartGroup, type Wants } from '../protocol';
import type { Config } from '../types';
import { h } from './dom';
import { compactNumber } from './format';
import {
  bandData,
  barsData,
  chartsBehind,
  emptyTable,
  histTable,
  lineData,
  overlayData,
  supplyDemandTable,
  type LineData,
} from './series-data';

interface Line { key: string; label: string; color: string }
type Section = 'top' | 'goods' | 'pollution' | 'economy' | 'disease';
type Kind = 'time' | 'band' | 'lorenz' | 'wealth' | 'supplyDemand';

/** One chart: its lines follow a world's config; it shows when its section and `shown` hold for either world. */
interface ChartDef {
  title: string;
  kind: Kind;
  section: Section;
  lines?: (c: Config) => Line[];
  range?: [number, number];
  shown?: (c: Config) => boolean;
  /** The caption names the traded pair (goods 0 and 1). */
  pair?: boolean;
}

const HEIGHT = 150;
/** The Lorenz curve, wealth histogram and supply & demand are fetched at most this often (per world). */
const REFRESH_MS = 250;
const POLLUTANT_COLORS = ['--c1', '--c2', '--c3', '--c4'];
/** The Trade price chart's series: the mean and its ± SD band share one x axis. */
const PRICE_GROUP = ['mean_log_price', 'sd_log_price'];
const LABELS = ['A', 'B'];
/** B's lines are dashed; A's are solid (Decision 11). */
const B_DASH = [6, 4];
const XS = Array.from({ length: 101 }, (_, i) => i / 100);
const X_LABEL: Record<Kind, string> = { time: 'Tick', band: 'Tick', lorenz: 'Population share', wealth: 'Sugar', supplyDemand: 'Price' };

const twoGoods = (c: Config) => c.goods.length >= 2;
const fixed = (lines: Line[]) => () => lines;
const perGood = (prefix: string) => (c: Config): Line[] =>
  c.goods.map((g, i) => ({ key: `${prefix}${i}`, label: g.name, color: g.color }));

const SECTIONS: { id: Section; title?: string; shown: (c: Config) => boolean }[] = [
  { id: 'top', shown: () => true },
  { id: 'goods', title: 'Goods', shown: () => true },
  { id: 'pollution', title: 'Pollution', shown: (c) => c.pollution.enabled },
  // Market charts need two goods; loan charts need credit; the section needs either.
  { id: 'economy', title: 'Economy', shown: (c) => twoGoods(c) || c.credit.enabled },
  { id: 'disease', title: 'Disease', shown: (c) => c.disease.enabled },
];

const CHARTS: ChartDef[] = [
  { title: 'Population', kind: 'time', section: 'top', lines: fixed([{ key: 'population', label: 'Agents', color: '--c1' }]) },
  { title: 'Gini coefficient', kind: 'time', section: 'top', lines: fixed([{ key: 'gini', label: 'Gini', color: '--c2' }]), range: [0, 1] },
  {
    title: 'Mean traits',
    kind: 'time',
    section: 'top',
    lines: fixed([
      { key: 'mean_vision', label: 'Vision', color: '--c1' },
      { key: 'mean_metabolism', label: 'Metabolism', color: '--c3' },
    ]),
  },
  // Group shares sit where the Blue share chart was.
  {
    title: 'Group shares',
    kind: 'time',
    section: 'top',
    lines: (c) => c.culture.groups.map((g, k) => ({ key: `group_share_${k}`, label: g.name, color: g.color })),
    range: [0, 1],
  },
  {
    title: 'Births and deaths',
    kind: 'time',
    section: 'top',
    lines: fixed([
      { key: 'births', label: 'Births', color: '--c3' },
      { key: 'deaths', label: 'Deaths', color: '--c2' },
    ]),
  },
  { title: 'Lorenz curve', kind: 'lorenz', section: 'top' },
  { title: 'Wealth distribution', kind: 'wealth', section: 'top' },
  { title: 'Mean holdings', kind: 'time', section: 'goods', lines: perGood('mean_holding_') },
  { title: 'Mean metabolism', kind: 'time', section: 'goods', lines: perGood('mean_metabolism_') },
  { title: 'Units traded', kind: 'time', section: 'goods', lines: perGood('traded_'), shown: (c) => c.trade.enabled },
  {
    title: 'Mean pollution',
    kind: 'time',
    section: 'pollution',
    lines: (c) => c.pollution.pollutants.map((p, k) => ({ key: `mean_pollution_${k}`, label: p.name, color: POLLUTANT_COLORS[k] })),
    shown: (c) => c.pollution.enabled,
  },
  { title: 'Trade price (ln)', kind: 'band', section: 'economy', shown: twoGoods, pair: true },
  { title: 'Trade volume', kind: 'time', section: 'economy', lines: fixed([{ key: 'trade_volume', label: 'Volume', color: '--c1' }]), shown: twoGoods },
  { title: 'Supply & demand', kind: 'supplyDemand', section: 'economy', shown: twoGoods, pair: true },
  {
    title: 'Loans',
    kind: 'time',
    section: 'economy',
    lines: fixed([
      { key: 'loans_made', label: 'Loans made', color: '--c1' },
      { key: 'defaults', label: 'Defaults', color: '--c2' },
    ]),
    shown: (c) => c.credit.enabled,
  },
  {
    title: 'Debt outstanding',
    kind: 'time',
    section: 'economy',
    lines: fixed([{ key: 'debt_outstanding', label: 'Debt', color: '--c3' }]),
    shown: (c) => c.credit.enabled,
  },
  {
    title: 'Foresight',
    kind: 'time',
    section: 'economy',
    lines: fixed([{ key: 'mean_foresight', label: 'Foresight φ', color: '--c3' }]),
    shown: (c) => c.foresight.enabled,
  },
  {
    title: 'Infected',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'infected_fraction', label: 'Infected share', color: '--red' }]),
    range: [0, 1],
    shown: (c) => c.disease.enabled,
  },
  {
    title: 'Diseases per agent',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'mean_diseases', label: 'Mean', color: '--c2' }]),
    shown: (c) => c.disease.enabled,
  },
  {
    title: 'Diseases in circulation',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'diseases_in_circulation', label: 'Distinct diseases', color: '--c4' }]),
    shown: (c) => c.disease.enabled,
  },
  {
    title: 'New infections',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'new_infections', label: 'Infections', color: '--c1' }]),
    shown: (c) => c.disease.enabled,
  },
];

/** The host chart group a chart draws for a world (7a Decision 4); none for the distributions. */
function groupOf(def: ChartDef, c: Config): string[] {
  if (def.kind === 'band') return PRICE_GROUP;
  return def.kind === 'time' ? def.lines!(c).map((l) => l.key) : [];
}

interface Plot {
  def: ChartDef;
  plot: uPlot;
  figure: HTMLElement;
  caption: HTMLElement;
  /** Each world's group (time and band charts), by world index. */
  groups: string[][];
  /** Each world's line count, for an empty table until its group arrives. */
  counts: number[];
  /** Each world's group as last drawn (a new copy from the engine means redraw). */
  drawn: (ChartGroup | undefined)[];
  /** The distribution versions last drawn. */
  drawnDist: string;
}

/** A world's latest distributions, and when (and at which tick) they arrived. */
interface Dist {
  lorenz: Float64Array | null;
  wealthHist: Float64Array | null;
  supplyDemand: Float64Array | null;
  version: number;
  at: number;
  tick: number;
  stale: boolean;
}

const freshDist = (): Dist => ({ lorenz: null, wealthHist: null, supplyDemand: null, version: 0, at: -Infinity, tick: -1, stale: true });

/**
 * The Charts tab (Decision 11): one list of charts drawn for `worlds` — the playground's engine, and
 * Compare's B while comparing — on shared axes, A solid and B dashed.
 */
export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private worlds: Engine[];
  private dist: Dist[];
  private plots: Plot[] = [];
  private visible = false;
  /** The lines signature the plots were built for. */
  private built = '';
  private readonly sections = new Map<Section, HTMLElement>();
  private offB: (() => void)[] = [];
  private readonly color: (v: string) => string;
  private readonly axes: uPlot.Axis[];

  constructor(private readonly engine: Engine) {
    this.worlds = [engine];
    this.dist = [freshDist()];
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
    for (const s of SECTIONS) if (s.id !== 'top') this.sections.set(s.id, h('section', { class: s.id }));
    this.attach(engine, 0);
    this.sync();
    new ResizeObserver(() => this.resize()).observe(this.el);
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    // Nothing is fetched here: the charts keep what they drew while hidden, `wants` asks for
    // whatever has fallen behind since, and the frame loop sends that (and only that) while paused.
    if (visible) {
      this.resize();
      this.redraw();
    }
  }

  /** Compare on (B's lines join A's) or off. */
  setCompare(b: Engine | null): void {
    for (const off of this.offB) off();
    this.offB = [];
    this.worlds = b ? [this.engine, b] : [this.engine];
    this.dist = this.worlds.map(() => freshDist());
    if (b) this.offB = this.attach(b, 1);
    this.built = '';
    this.sync();
    this.redraw();
  }

  canvases(): { name: string; canvas: HTMLCanvasElement }[] {
    return this.plots.map((p) => ({ name: p.def.title, canvas: p.plot.ctx.canvas }));
  }

  /** World `i`'s provider and listeners; returns their removal. */
  private attach(w: Engine, i: number): (() => void)[] {
    return [
      w.want((now) => this.wants(i, now)),
      w.on('snapshot', () => this.receive(i)),
      // Edits, resets and config changes move the distributions without a tick.
      ...(['edit', 'reset', 'config'] as const).map((event) => w.on(event, () => (this.dist[i].stale = true))),
      ...(['reset', 'config'] as const).map((event) => w.on(event, () => this.sync())),
    ];
  }

  private shown(def: ChartDef): boolean {
    const section = SECTIONS.find((s) => s.id === def.section)!;
    return this.worlds.some((w) => section.shown(w.config) && (def.shown?.(w.config) ?? true));
  }

  /**
   * World `i`'s groups for the charts on show (only while the engine's copy of some group is missing
   * or behind the tick, so a paused, caught-up panel asks for nothing), and its distributions when
   * due; nothing while hidden.
   */
  private wants(i: number, now: number): Wants {
    const w = this.worlds[i];
    if (!this.visible || !w) return {};
    const groups = this.plots.filter((p) => !p.figure.hidden && p.groups[i].length > 0).map((p) => p.groups[i]);
    const out: Wants = {};
    if (groups.length > 0 && chartsBehind(groups, w.tick, (g) => w.chartGroup(g))) out.charts = { groups, max: CHART_POINTS };
    const d = this.dist[i];
    if ((d.stale || w.tick !== d.tick) && now - d.at >= REFRESH_MS) {
      out.lorenz = true;
      out.wealthHist = true;
      if (twoGoods(w.config)) out.supplyDemand = true;
    }
    return out;
  }

  /** Takes world `i`'s snapshot's distributions and redraws what changed. */
  private receive(i: number): void {
    const s = this.worlds[i]?.last;
    if (!s) return;
    // The host answers in order and computes these fresh, so no reply after an edit's own reply
    // can carry pre-edit data: clearing `stale` on any snapshot that has them is safe.
    if (s.lorenz) {
      const d = this.dist[i];
      d.lorenz = s.lorenz;
      d.wealthHist = s.wealthHist ?? d.wealthHist;
      d.supplyDemand = s.supplyDemand ?? d.supplyDemand;
      d.version++;
      d.tick = s.tick;
      d.at = performance.now();
      d.stale = false;
    }
    this.redraw();
  }

  /**
   * Rebuilds the plots when any world's chart lines changed (or Compare started or ended), then
   * shows the charts and sections either world would show and names the traded pair.
   */
  private sync(): void {
    const signature = JSON.stringify(this.worlds.map((w) => CHARTS.map((d) => d.lines?.(w.config) ?? null)));
    if (signature !== this.built) {
      this.built = signature;
      this.build();
    }
    const configs = this.worlds.map((w) => w.config);
    for (const s of SECTIONS) {
      const el = this.sections.get(s.id);
      if (el) el.hidden = !configs.some((c) => s.shown(c));
    }
    const goods = configs.find(twoGoods)?.goods;
    for (const p of this.plots) {
      p.figure.hidden = !this.shown(p.def);
      p.caption.textContent = p.def.pair && goods ? `${p.def.title} · ${goods[0].name}/${goods[1].name}` : p.def.title;
    }
    for (const d of this.dist) d.stale = true;
  }

  private build(): void {
    for (const p of this.plots) p.plot.destroy();
    this.plots = [];
    for (const [id, el] of this.sections) el.replaceChildren(h('h3', {}, SECTIONS.find((s) => s.id === id)!.title!));
    const top: HTMLElement[] = [];
    for (const def of CHARTS) {
      const figure = this.plotFor(def);
      if (def.section === 'top') top.push(figure);
      else this.sections.get(def.section)!.append(figure);
    }
    this.el.replaceChildren(...top, ...this.sections.values());
    this.resize();
  }

  private plotFor(def: ChartDef): HTMLElement {
    const caption = h('figcaption', {}, def.title);
    const figure = h('figure', { class: 'chart' }, caption);
    const groups = this.worlds.map((w) => groupOf(def, w.config));
    const counts = groups.map((g) => (def.kind === 'band' ? 3 : g.length));
    const data = def.kind === 'time' || def.kind === 'band' ? this.merge(counts.map(emptyTable)) : this.distData(def.kind);
    const plot = new uPlot({ ...this.options(def), width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ def, plot, figure, caption, groups, counts, drawn: this.worlds.map(() => undefined), drawnDist: '' });
    return figure;
  }

  private options(def: ChartDef): Omit<uPlot.Options, 'width' | 'height'> {
    const multi = this.worlds.length > 1;
    const series: uPlot.Series[] = [{ label: X_LABEL[def.kind] }];
    if (def.kind === 'lorenz') series.push({ label: 'Equality', stroke: this.color('--muted'), dash: [4, 4], width: 1 });
    this.worlds.forEach((w, i) => series.push(...this.seriesFor(def, w.config, multi ? `${LABELS[i]} · ` : '', i === 1)));
    const x: uPlot.Scale = { time: false };
    if (def.kind === 'lorenz') x.range = [0, 1];
    if (def.kind === 'supplyDemand') x.distr = 3;
    const y: uPlot.Scale = {};
    if (def.kind === 'lorenz') y.range = [0, 1];
    else if (def.range) y.range = def.range;
    const lines = def.kind === 'time' ? def.lines!(this.engine.config).length : 0;
    const legend = multi || def.kind === 'band' || def.kind === 'supplyDemand' || lines > 1;
    return { scales: { x, y }, axes: this.axes, legend: { show: legend }, series };
  }

  /** One world's series: labeled "A · …"/"B · …" in Compare, B dashed and its points hollow. */
  private seriesFor(def: ChartDef, c: Config, tag: string, b: boolean): uPlot.Series[] {
    const dash = b ? B_DASH : undefined;
    switch (def.kind) {
      case 'time':
        return def.lines!(c).map((l) => ({ label: tag + l.label, stroke: this.color(l.color), width: 1.5, dash }));
      case 'band': {
        const sd = b ? [2, 3] : [4, 4];
        return [
          { label: `${tag}Mean`, stroke: this.color('--c2'), width: 1.5, dash },
          { label: `${tag}+SD`, stroke: this.color('--muted'), width: 1, dash: sd },
          { label: `${tag}-SD`, stroke: this.color('--muted'), width: 1, dash: sd },
        ];
      }
      case 'lorenz':
        return [{ label: `${tag}Wealth share`, stroke: this.color('--c2'), width: 2, dash }];
      case 'wealth':
        return this.worlds.length > 1
          ? [{ label: `${tag}Agents`, stroke: this.color('--c1'), width: 1.5, dash, paths: uPlot.paths.stepped!({ align: 1 }), points: { show: false } }]
          : [{ label: 'Agents', fill: this.color('--c1'), stroke: this.color('--c1'), paths: uPlot.paths.bars!({ size: [0.9, 64] }), points: { show: false } }];
      case 'supplyDemand': {
        const fill = b ? this.color('--surface') : undefined;
        return [
          { label: `${tag}Demand`, stroke: this.color('--c1'), width: 1.5, dash },
          { label: `${tag}Supply`, stroke: this.color('--c2'), width: 1.5, dash },
          { label: `${tag}Equilibrium`, stroke: this.color('--c3'), points: { show: true, size: 9, fill }, paths: () => null },
          { label: `${tag}Actual`, stroke: this.color('--text'), points: { show: true, size: 9, fill }, paths: () => null },
        ];
      }
    }
  }

  /** One table as is; several on the union of their x values (Decision 11). */
  private merge(tables: LineData[]): uPlot.AlignedData {
    return tables.length === 1 ? tables[0] : overlayData(tables);
  }

  private distData(kind: Kind): uPlot.AlignedData {
    switch (kind) {
      case 'lorenz':
        return [XS, XS, ...this.dist.map((d) => (d.lorenz ? Array.from(d.lorenz) : XS.map(() => null)))] as uPlot.AlignedData;
      case 'wealth':
        return this.worlds.length > 1 ? overlayData(this.dist.map((d) => histTable(d.wealthHist))) : barsData(this.dist[0].wealthHist);
      default:
        return this.merge(this.dist.map((d) => supplyDemandTable(d.supplyDemand)));
    }
  }

  /** Redraws each chart on show whose data changed: a new group copy, or new distributions. */
  private redraw(): void {
    if (!this.visible) return;
    for (const p of this.plots) if (!p.figure.hidden) this.draw(p);
  }

  private draw(p: Plot): void {
    if (p.def.kind === 'time' || p.def.kind === 'band') {
      const groups = this.worlds.map((w, i) => w.chartGroup(p.groups[i]));
      if (groups.every((g, i) => g === p.drawn[i])) return;
      p.drawn = groups;
      const band = p.def.kind === 'band';
      p.plot.setData(this.merge(groups.map((g, i) => (g ? (band ? bandData(g) : lineData(g)) : emptyTable(p.counts[i])))));
      return;
    }
    const version = this.dist.map((d) => d.version).join();
    if (version === p.drawnDist) return;
    p.drawnDist = version;
    p.plot.setData(this.distData(p.def.kind));
  }

  private width(): number {
    return Math.max(240, this.el.clientWidth - 4);
  }

  private resize(): void {
    if (!this.visible) return;
    for (const p of this.plots) p.plot.setSize({ width: this.width(), height: HEIGHT });
  }
}
```

- [ ] **Step 6: Charts in the Compare view**

In `web/src/compare/compare-view.ts`:
- add `import type { ChartsPanel } from '../ui/charts-panel';` before the `CreditPanel` import.
- in `interface Playground`, after `tabs: Tabs;` add `charts: ChartsPanel;`.
- in the constructor, after `p.toolbar.setCompare(this.lock, b);` add `p.charts.setCompare(b);`.
- in `leave`, after `p.toolbar.setCompare(null, null);` add `p.charts.setCompare(null);`.

In `web/src/main.ts`, in the `playground` object literal, after `tabs,` add `charts,`.

- [ ] **Step 7: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.
Run: `grep -n "chartsSignature\|groupSharesSignature" web/src/ui/charts-panel.ts`
Expected: no output (the lines signature replaces both; `goods.ts` and `groups.ts` keep them, tested).

- [ ] **Step 8: Commit**

```bash
git add web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/ui/charts-panel.ts web/src/compare/compare-view.ts web/src/main.ts
git commit -m "Overlay both worlds' charts in Compare" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 13: Compare's exports, links and files, and recording both grids

*Needs judgment (full code given; the page's final wiring).* Browser (controller), `/?debug`, in Compare on `ii-2-unit` with B diverged by 🎲:
- Export shows rows "Statistics (CSV) [A] [B]", "Agents (CSV) [A] [B]", "Grid (PNG) [A] [B]": each button downloads that world's file (`…-A-series.csv`, `…-B-grid.png`, …) with that world's data; Charts (PNG) downloads the overlaid charts as `sugarscape-compare-t<T>-<chart>.png`; Session (JSON) downloads `sugarscape-compare-t<T>-session.json` holding `"sugarscape":"compare"`.
- Share → Copy link gives a `#c=` URL; opening it in a new tab goes straight into Compare with both worlds at t = 0, each replaying its log (per-header chips count down); played to the original tick, both fingerprints (`sugarscape.engine`, `sugarscape.compare().b`) equal the original tab's.
- Share → Open session… with the comparison file (from single mode or from Compare) opens Compare with both worlds; with a single-world session file while comparing, Compare ends keeping A and the session opens.
- ● Record → WebM and GIF in Compare: frames show both grids side by side with "A"/"B" tags and the tick stamp; the file is named `sugarscape-compare-seed<a>-vs-seed<b>-t<from>-t<to>.<ext>`; at Max the GIF recording shows no main-thread task over 50 ms.
- Leaving Compare (Keep A/Keep B) returns the Export menu to single buttons and Share to `#s=` links.

**Files:**
- Modify: `web/src/main.ts`

**Interfaces:**
- Consumes: `compareLink`, `shareable`, `LOG_FULL_NOTICE` (Task 5); `decodeCompare`, `readCompareHash`, `SessionFile`, `sessionFileText`, `parseSessionFile` (Task 2); `ExportWorld`, `buildExportMenu` (Task 5); `CompareView.b/gridB/lock` (Task 10); `InitialState` (Task 3).
- Produces: the page's final wiring (no new exports).

- [ ] **Step 1: The page's final wiring**

Replace `web/src/main.ts` with:
```ts
import './style.css';
import { askKeep, CompareView, compareShell, type Playground, type WorldName } from './compare/compare-view';
import { copyWorld } from './compare/lockstep';
import { canvasBlob, downloadBlob, downloadText } from './downloads';
import { Engine, type InitialState } from './engine';
import { ExperimentsView } from './experiments/view';
import { compareLink, LOG_FULL_NOTICE, sessionLink, shareable } from './sessions';
import {
  decodeCompare,
  decodeShare,
  decodeSweep,
  parseSessionFile,
  readCompareHash,
  readHash,
  readSweepHash,
  sessionFileText,
  type SessionFile,
} from './share';
import { ChartsPanel } from './ui/charts-panel';
import { CreditPanel } from './ui/credit-panel';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { buildExportMenu, type ExportWorld } from './ui/export-menu';
import { GridView } from './ui/grid-view';
import { InspectPanel } from './ui/inspect-panel';
import { showNotice } from './ui/notice';
import { buildRecordControl } from './ui/record-control';
import { RulesPanel } from './ui/rules-panel';
import { buildShareMenu } from './ui/share-menu';
import { Tabs } from './ui/tabs';
import { Toolbar } from './ui/toolbar';
import { buildTools } from './ui/tools';
import { WorldSlot } from './ui/world-slot';

export function showBanner(message: string, action?: { label: string; run: () => void }): void {
  const banner = document.querySelector<HTMLElement>('#banner')!;
  banner.replaceChildren(
    ...[
      h('span', {}, message),
      action ? h('button', { onclick: action.run }, action.label) : null,
      h('button', { onclick: () => (banner.hidden = true), 'aria-label': 'Dismiss' }, '×'),
    ].filter((child): child is HTMLElement => child !== null),
  );
  banner.hidden = false;
}

const message = (e: unknown): string => (e instanceof Error ? e.message : String(e));

async function main(): Promise<void> {
  let engine: Engine;
  /** A `#c=` link's B: Compare starts with it once the page is built. */
  let startB: InitialState | null = null;
  const token = readHash();
  const compareToken = readCompareHash();
  try {
    if (compareToken) {
      const { a, b } = await decodeCompare(compareToken);
      engine = await Engine.create(a);
      startB = b;
    } else {
      // A link's session replays its edits as the world runs (Decision 2).
      engine = token ? await Engine.create(await decodeShare(token)) : await Engine.create();
    }
  } catch (e) {
    showBanner(`That share link could not be loaded (${message(e)}). Showing the default rule system.`);
    engine = await Engine.create();
  }
  /** Compare mode's second world and its coordinator, while Compare is on. */
  let compare: CompareView | null = null;
  // Browser checks drive the engines through this handle (7a Decision 14).
  if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { sugarscape: { engine, compare: () => compare } });
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  const toolbar = new Toolbar(engine);
  document.querySelector('#toolbar')!.append(toolbar.el);
  document.querySelector('#display')!.append(buildDisplay(engine));
  const experiments = new ExperimentsView(engine);
  document.querySelector('#experiments')!.append(experiments.el);
  const views = { playground: 'Playground', experiments: 'Experiments' } as const;
  type View = keyof typeof views;
  const viewButtons = (Object.keys(views) as View[]).map((view) =>
    h('button', { 'data-view': view, onclick: () => showView(view) }, views[view]),
  );
  const showView = (view: View): void => {
    // The playground's worlds are kept, paused, while Experiments is shown.
    if (view === 'experiments') (compare?.lock ?? engine).setRunning(false);
    document.body.dataset.view = view;
    document.querySelector<HTMLElement>('#playground')!.hidden = view !== 'playground';
    document.querySelector<HTMLElement>('#experiments')!.hidden = view !== 'experiments';
    for (const b of viewButtons) b.setAttribute('aria-pressed', String(b.dataset.view === view));
  };
  const compareButton = h(
    'button',
    {
      class: 'compare-toggle',
      'aria-pressed': 'false',
      title: 'Run a copy of this world beside it, both stepped in lockstep',
      onclick: () => void toggleCompare(),
    },
    'Compare',
  );
  document
    .querySelector('.toolbar h1')!
    .after(h('div', { class: 'view-switch', role: 'group', 'aria-label': 'View' }, ...viewButtons), compareButton);
  showView('playground');

  const sweepToken = readSweepHash();
  if (sweepToken) {
    try {
      experiments.openSweep(await decodeSweep(sweepToken));
      showView('experiments');
    } catch (e) {
      showBanner(`That experiment link could not be loaded (${message(e)}).`);
    }
  }

  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  // Each tab holds A's panel, and B's beside it in Compare (Decision 10).
  const rules = new WorldSlot(new RulesPanel(engine), 'switch', 'Rules for');
  tabs.add('Rules', rules.el);
  const charts = new ChartsPanel(engine);
  tabs.add('Charts', charts.el, (visible) => charts.setVisible(visible));
  const inspect = new WorldSlot(new InspectPanel(engine), 'label');
  tabs.add('Inspect', inspect.el, (visible) => inspect.setVisible(visible));
  const credit = new WorldSlot(
    new CreditPanel(engine, () => {
      compare?.focus('A');
      tabs.show('Inspect');
    }),
    'label',
  );
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on in a world on screen.
  const syncCreditTab = (b: Engine | null) => tabs.setHidden('Credit', ![engine, b].some((e) => e?.config.credit.enabled));
  engine.on('reset', () => syncCreditTab(compare?.b ?? null));
  engine.on('config', () => syncCreditTab(compare?.b ?? null));
  syncCreditTab(null);
  const tools = buildTools({ engine, grid }, (world) => {
    compare?.focus(world === engine ? 'A' : 'B');
    tabs.show('Inspect');
  });
  document.querySelector('#tools')!.append(tools.el);

  const slug = (e: Engine) => `sugarscape-${e.presetId ?? 'custom'}-seed${e.seed}-t${e.tick}`;
  /** File stems for what covers both worlds in Compare (charts, the session). */
  const stem = () => (compare ? `sugarscape-compare-t${engine.tick}` : slug(engine));
  const worlds = (): ExportWorld[] => {
    const c = compare;
    return c
      ? [
          { label: 'A', engine, grid },
          { label: 'B', engine: c.b, grid: c.gridB },
        ]
      : [{ label: '', engine, grid }];
  };
  const exportMenu = buildExportMenu({
    worlds,
    slug: (w) => (w.label ? `${slug(w.engine)}-${w.label}` : slug(w.engine)),
    charts: async () => {
      const name = stem();
      tabs.show('Charts');
      await Promise.all([engine.refresh(), compare?.b.refresh()]);
      // Let the panel draw the fresh snapshots before the canvases are captured.
      await new Promise((resolve) => requestAnimationFrame(resolve));
      for (const chart of charts.canvases()) {
        downloadBlob(`${name}-${chart.name.toLowerCase().replace(/\W+/g, '-')}.png`, await canvasBlob(chart.canvas));
      }
    },
    session: async () => {
      const c = compare;
      const name = stem();
      const a = await shareable(engine);
      let file: SessionFile = { kind: 'session', state: a.state };
      let full = a.full;
      if (c) {
        const b = await shareable(c.b);
        file = { kind: 'compare', state: { a: a.state, b: b.state } };
        full ||= b.full;
      }
      if (full) showNotice(LOG_FULL_NOTICE, 10_000);
      downloadText(`${name}-session.json`, sessionFileText(file), 'application/json');
    },
  });
  const shareMenu = buildShareMenu({
    link: () => (compare ? compareLink(engine, compare.b) : sessionLink(engine)),
    open: async (file) => {
      try {
        const opened = parseSessionFile(await file.text());
        if (compare) await leaveCompare('A');
        const errors = await engine.open(opened.kind === 'session' ? opened.state : opened.state.a);
        if (errors) throw new Error(errors.map((x) => `${x.field}: ${x.message}`).join('; '));
        // The address bar no longer describes this world.
        history.replaceState(null, '', location.pathname + location.search);
        if (opened.kind === 'compare') await enterCompare(opened.state.b);
        showNotice(`Opened ${file.name}`);
      } catch (e) {
        showNotice(`${file.name} could not be opened (${message(e)})`, 10_000);
      }
    },
  });
  const record = buildRecordControl({
    grids: () => {
      const c = compare;
      return c
        ? [
            { canvas: grid.canvas, cells: () => engine.size(), label: 'A' },
            { canvas: c.gridB.canvas, cells: () => c.b.size(), label: 'B' },
          ]
        : [{ canvas: grid.canvas, cells: () => engine.size() }];
    },
    tick: () => engine.tick,
    running: () => (compare?.lock ?? engine).running,
    base: () => {
      const c = compare;
      return c ? `sugarscape-compare-seed${engine.seed}-vs-seed${c.b.seed}` : `sugarscape-${engine.presetId ?? 'custom'}-seed${engine.seed}`;
    },
  });
  engine.on('run', () => record.sync());
  document.querySelector('.toolbar-end')!.append(record.el, shareMenu, exportMenu);

  const crashed = () => showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
  engine.on('crash', crashed);
  engine.on('fork', () => showNotice('Replay ended — your edit starts a new branch'));

  let dirty = true;
  /** A snapshot arrived since the last frame (Compare: a lockstep pair): once drawn, the recording captures it. */
  let fresh = false;
  for (const event of ['snapshot', 'display'] as const) engine.on(event, () => (dirty = true));
  engine.on('snapshot', () => {
    if (!compare) fresh = true;
  });
  const playground: Playground = {
    engine,
    grid,
    toolbar,
    tools,
    tabs,
    charts,
    rules,
    inspect,
    credit,
    syncCreditTab,
    onRun: () => record.sync(),
    onFrame: () => (fresh = true),
    onCrash: crashed,
  };

  let entering = false;
  const syncCompareButton = () => {
    compareButton.setAttribute('aria-pressed', String(compare !== null));
    compareButton.disabled = entering;
  };
  /** Starts Compare with B built from `bState` (a link or file), or a copy of A at its current tick (Decision 9). */
  async function enterCompare(bState?: InitialState): Promise<void> {
    if (compare || entering) return;
    entering = true;
    syncCompareButton();
    engine.setRunning(false);
    toolbar.hold(true);
    const shell = compareShell();
    try {
      let b: Engine;
      if (bState) {
        b = await Engine.create(bState);
      } else {
        const { session, full, tick } = await engine.session();
        if (full) throw new Error('A’s edit log is full (50 000 edits), so B cannot copy it exactly');
        b = await copyWorld(session, tick, (s) => Engine.create(s), (at, of) => {
          shell.progress.textContent = `Copying A… ${at} / ${of}`;
        });
      }
      compare = new CompareView(playground, b, shell);
    } catch (e) {
      shell.figure.remove();
      delete document.body.dataset.compare;
      showNotice(`Compare could not start (${message(e)})`, 10_000);
    } finally {
      entering = false;
      toolbar.hold(false);
      syncCompareButton();
    }
  }
  async function leaveCompare(keep: WorldName): Promise<void> {
    const c = compare;
    if (!c) return;
    compare = null;
    await c.leave(keep);
    syncCompareButton();
  }
  async function toggleCompare(): Promise<void> {
    if (!compare) return enterCompare();
    const keep = await askKeep();
    if (keep) await leaveCompare(keep);
  }

  const loop = (now: number) => {
    try {
      (compare?.lock ?? engine).pump(now);
      if (dirty) {
        grid.draw();
        dirty = false;
      }
      compare?.draw();
      if (fresh) {
        fresh = false;
        record.capture();
      }
    } catch (e) {
      // A Rust panic in the page's WASM leaves it unusable; reloading keeps any #s= share state.
      (compare?.lock ?? engine).setRunning(false);
      console.error(e);
      crashed();
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
  // A #c= link opens straight into Compare, both worlds at t = 0 replaying their logs.
  if (startB) await enterCompare(startB);
}

main().catch((e) => showBanner(`Failed to start: ${message(e)}`));
```

- [ ] **Step 2: Full verification**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all test files pass.
Run: `grep -n "cannot open yet" web/src/main.ts`
Expected: no output (comparison files open now).

- [ ] **Step 3: Commit**

```bash
git add web/src/main.ts
git commit -m "Export, link, open and record both worlds in Compare" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 14: README, roadmap and full verification

*Mechanical.* Browser (controller): the full pass below.

**Files:**
- Modify: `README.md`, `docs/roadmap.md`

- [ ] **Step 1: README**

In `README.md`, insert before `## Command line`:
```markdown
## Sessions and share links

The playground records every edit you make — painting, image imports, placing and erasing
agents, infections, vaccinations and live rule changes — with the tick it happened at. **Share →
Copy link** carries the whole session: the setup, the painted maps it started from and that
edit log. Opening the link rebuilds the world and replays each edit at its tick as the world
runs (a chip counts down the edits left), so it reaches exactly the same world at the same tick,
at any speed. Editing during a replay starts a new branch from there; the chip's ✕ ends the
replay and keeps the world. **Reset** with the same seed rewinds and replays the session; a new
seed, 🎲, a preset or a rule change that needs a reset starts a new session. Very long sessions
still make a link (the page says when it is long); **Export → Session (JSON)** saves the same
content as a file and **Share → Open session…** loads it. After 50 000 edits recording stops and
links carry the setup and painted maps only. Links from earlier versions still open.

## Compare

**Compare** runs a copy of the current world beside it, each in its own worker: B starts as an
exact copy of A at the current tick, and both step in lockstep (Play, Step, the speeds and Max
act on both; ticks always match). Each grid has its own seed and 🎲; the Rules tab's **Rules
for: A | B** switch applies changes to one world, tools act on the grid you click, and Inspect
and Credit show the world you clicked last. A rebuilt world (🎲, a preset, a reset-requiring
change) rewinds the other to t = 0 so the two stay comparable. Charts overlay A (solid) and B
(dashed). Exports ask which world; Share makes a link that opens straight into Compare. Leaving
asks which world to keep.

## Recording

**● Record** records the grid as drawn (overlays, trails, selection) as WebM video or an
animated GIF, optionally stamped with the tick. Cells are 8 px (smaller for grids over 135
cells, keeping the frame within 1080 px); in Compare both grids are recorded side by side.
Recording pauses while the world is paused. GIFs are sampled at about 15 frames a second,
encoded off the page in a worker, and stop at 900 frames. Files are named after the setup and
the ticks they cover, e.g. `sugarscape-ii-2-unit-seed7-t0-t800.webm`.
```

- [ ] **Step 2: Roadmap**

In `docs/roadmap.md`, insert after the Milestone 7a section:
```markdown
## Milestone 7b: Sessions, comparison and recording (done)

A replayable edit log (share links and session files reproduce a whole session exactly, at any
speed), a side-by-side Compare mode (two worlds in lockstep with per-world rules and overlaid
charts), and recording the grid as WebM or GIF. Runs are unchanged. See
`docs/superpowers/specs/2026-09-24-sessions-compare-recording-design.md`.
```
In "Playground and infrastructure", replace the "Replayable edit log", "Side-by-side comparison" and "Recording" bullets with:
```markdown
- **Replayable edit log**: done (Milestone 7b).
- **Side-by-side comparison**: done (Milestone 7b).
- **Recording**: done (Milestone 7b).
```

- [ ] **Step 3: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
git diff main -- crates/
git diff main --stat -- web/package.json
```
All must pass; `git diff main -- crates/` prints nothing; the package diff shows only the `gifenc` line.

The controller then runs the full puppeteer pass (implementers don't): every 7a scenario (rules, tools, painting, image import, inspection, charts, share links, exports, disease, credit tab, groups, trails, Experiments, Max, the worker determinism check `0x75b93943813545e4`); every scenario listed in Tasks 5, 8 and 10–13; and the performance checks — a 200 × 200 world with 2 000 agents at Max for 20 s while recording a GIF, and Compare at Max on the same world, each with a `PerformanceObserver({ type: 'longtask', buffered: true })`: no main-thread task over 50 ms.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md
git commit -m "Document sessions, Compare and recording" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

## Spec coverage

| Spec requirement | Task |
|---|---|
| No simulation change; golden and legacy green | Global Constraints; no task edits `crates/`; 14 (`git diff main -- crates/`) |
| Exact replay at every speed (1×…100×, Max) | 1 (host replay inside `step n` and Max batches), 4 (real WASM: mixed speeds, Max, through encode/decode, Reset), 9 (B's copy) |
| Old links keep working (maps become starting maps, empty log) | 2 (v1/v2 decode with `log: []`; legacy fixture test), 5 (controller) |
| No COOP/COEP, no SharedArrayBuffer; only `gifenc` added | 7 (the only dependency change), 8 (transferable buffers) |
| Edit log entries `{ tick, cmd }`, which commands, view commands not logged | 1 (Decision 1; test "logs world-changing commands that succeed") |
| Recording after success, edit at t applied before t + 1, init/reset start a new log | 1 |
| 50 000 cap, `logFull`, link fallback with a notice | 1 (`LOG_CAP` test), 5 (`sessionLink` fallback, `LOG_FULL_NOTICE`), 3 (`replay()` falls back) |
| Session `{ config, seed, landscapes, log }` (build config, starting maps) | 1 (types), 3 (`origin`, `session()`) |
| Replay: tick-0 entries at build, later ones one tick at a time in `step n` and Max, re-logged, `replayLeft` on change | 1 (Decision 2), 3 (`'replay'` event) |
| Forking drops pending entries, notice | 1 (Decision 3), 3 (`'fork'`), 5 (notice) |
| Chip "Replaying · N edits left ✕", ✕ = `endReplay` | 5 (`replayChip`), 10 (per-world headers) |
| Reset (same seed) replays; 🎲, preset, reset-requiring change: empty log | 3 (Decision 4), 5 (toolbar), 10 (Compare Reset) |
| Commands `init { log? }`, `session`, `endReplay` | 1 |
| Wire version with `e`, deflated with the rest | 2 (Decision 5) |
| Token > 32 000: still copied, "link is long"; Export → Session (JSON); Share → Open session… | 5 (Decision 6) |
| `#c=` links opening into Compare at t = 0 | 2 (codec), 5 (`compareLink`), 13 (open, copy) |
| Compare toggle, A and B each a full `Engine` with its own worker; Keep A / Keep B | 9 (`takeWorld`, `close`), 10 (Decision 8) |
| B from A's session truncated at T, advanced to T with "Copying A… t / T"; B's fingerprint = A's | 9 (`copyWorld`, FakeSim and real-WASM tests), 10 (progress) |
| Lockstep: step n to both, wait for both; speeds; adaptive Max (25/40 ms, 1…10 000) | 9 (Decision 9; tests) |
| Rebuilds: Reset rewinds both; one world rebuilt rewinds the other | 9 (tests), 10 (headers' 🎲, toolbar Reset) |
| Headers (label, seed, 🎲); Rules for: A \| B; tools per grid; Inspect/Credit show last-clicked world, labeled | 10, 11 (Decision 10) |
| Charts overlaid A solid / B dashed, legends "A · …"/"B · …"; Lorenz and S&D overlay; wealth as step outlines; a chart shows if either world shows it | 12 (Decision 11) |
| Export: CSV and Grid PNG ask A or B; Charts PNG overlaid | 5 (menu), 13 (Decision 12) |
| Experiments pauses both | 10 (`showView`) |
| ● Record menu (WebM \| GIF, stamp on), ■ m:ss, stop downloads `…-t<from>-t<to>.<ext>` | 6 (names, clock), 8 (control) |
| Frames: grid as drawn, integer scale ≥ 8 px capped at 1080 px, nearest-neighbor, stamp, side by side in Compare with labels, one frame per displayed snapshot, pauses with the world, reset continues | 6 (`frameLayout`), 8 (Decision 13), 13 (Compare) |
| WebM: `captureStream(0)` + `requestFrame()`, MIME order, pause/resume | 6 (`pickMime`), 8 (Decision 14) |
| GIF: ~15 fps, worker with `gifenc` (quantize + palette per frame), 900 cap with notice, progress while finishing | 6 (`GifSampler`), 7 (`GifBuilder`, worker; GIF89a frame-count test), 8 |
| Tests: host, determinism, share, comparison, recording | 1; 4, 9; 2, 5; 9; 6, 7 |
| Browser checks incl. no main-thread task > 50 ms recording a GIF at Max | each task's list; 8, 14 |
| Docs: README sections; roadmap items done | 14 |

## Spec gaps and conflicts found

1. **`reset` also takes `log`.** The spec gives `init { …, log? }`; the engine rebuilds with `reset`, so Reset's replay, `open()` and Compare's rewinds send `reset { …, log }` (Decision 2).
2. **What `session` returns mid-replay.** It returns the applied entries followed by those still pending, so a link or copy made mid-replay carries the whole session (Decision 1). Compare's copy then truncates to ticks ≤ T as specified, so A's later pending entries are not copied (Decision 9).
3. **"One tick at a time" in `step n`** is implemented as stepping straight to the next pending entry's tick — the same world, since `step(k)` equals k single steps (pinned by 7a's determinism test) — then applying it (Decision 2).
4. **Which edits fork, and which are logged.** Only a page edit that succeeds is logged or forks a replay; a failed one (erasing an empty site) changes nothing (Decisions 1, 3).
5. **Reset's two meanings.** The toolbar's Reset replays when the seed box still shows the world's seed; with another seed typed it builds a new world with an empty log (as 🎲 does). A replay keeps the base config and preset; replayed live changes are not folded into the base config. With a full log Reset is today's reset (Decision 4).
6. **Where "Open session…" lives and what a session file is.** Share becomes a menu (Copy link, Open session…); a session file is the link's wire object as plain JSON tagged `"sugarscape": "session"` (or `"compare"` with `a`/`b`). Opening a file replaces the page's world and clears the hash; in Compare, Session (JSON) exports the comparison and opening a single session first leaves Compare keeping A (Decisions 6, 12).
7. **The inflate cap** rises from 1 MiB to 16 MiB so full logs decode; two share tests change to match (Decision 5).
8. **Keep B** is implemented by moving B's transport and state into the page's `Engine` object (`takeWorld`), so every panel bound to it keeps working (Decision 8).
9. **Entering Compare** pauses A and holds the run controls until B exists; if A's log is full, Compare does not start, since B could not be an exact copy (Decision 9).
10. **Display, chips and seed in Compare.** One display (A's, mirrored to B); the toolbar's seed box, 🎲 and chips move to the grid headers; the paint tool's goods and display changes follow A; the disease picker and image import follow the last-clicked world (Decision 10).
11. **Overlaying different x axes.** Two worlds' downsampled ticks (and price grids) differ; the charts join them on the union of x values with `undefined` holes that uPlot draws through and `null` gaps it breaks at (Decision 11).
12. **Recording's frame size when a grid is resized mid-recording** (by a reset, or entering Compare): the frame keeps its starting size and the new layout is scaled to fit (Decision 13). GIF delays come from recorded time; while the encoder is 3 frames behind, capture skips frames (bounded memory). In Compare a frame is captured per lockstep pair.
13. **Notices vs the banner.** Non-error messages use a new status strip rather than the red crash banner (Decision 7).
