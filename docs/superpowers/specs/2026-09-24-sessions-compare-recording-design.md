# SugarScape Milestone 7b — Edit log, comparison and recording — Design

**Date:** 2026-09-24
**Builds on:** the milestone 1–7a specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular 7a's worker engine (`sim-host`, `transport`, `engine`).

## Goal

Three playground features on the 7a engine: a replayable edit log so share links reproduce a whole session, a side-by-side comparison of two worlds stepped in lockstep, and recording the grid as WebM or GIF.

## Non-negotiable constraints

- **No simulation change.** Golden and legacy tests stay unedited and green; nothing in `sugarscape-core` changes behavior.
- **Exact replay.** Replaying a session gives the same world (same `fingerprint`) at the same tick as the live session, at every speed (1×…100×, Max).
- **Old links keep working.** Every existing `#s=` link decodes as before (its painted maps become starting maps with an empty log).
- **Deploys as today:** no COOP/COEP, no SharedArrayBuffer. One new runtime dependency is allowed: `gifenc` (MIT).

## Edit log

- **Entries:** `{ tick, cmd }` where `cmd` is one of the host's world-changing commands: `paint`, `importLandscape`, `place`, `erase`, `infect`, `vaccinate`, `setConfig` (the full new config). View commands (`follow`, `inspect`, `setDisplay`, exports, `refresh`, `frame`) are not logged.
- **Recording (host):** after a world-changing command succeeds, the host appends `{ tick: sim.tick(), cmd }` to its log. An edit at tick t is applied before tick t + 1 is computed. `init`/`reset` start a new, empty log. The log holds at most 50 000 entries; past that, recording stops, the host reports `logFull`, and a share link falls back to setup + current painted maps (today's behavior) with a notice.
- **Session:** `{ config, seed, landscapes, log }`, where `config` is the config the world was built with (not the current one — live changes are in the log), `landscapes` are the painted maps passed to that build (starting maps) and `log` the entries. Scheduled changes stay in `config.schedule`, as today.
- **Replay (host):** `init` accepts `log`. Entries with tick 0 are applied right after the world is built; otherwise, while entries are pending, `step n` and Max batches advance one tick at a time and apply every entry whose tick equals the new tick, in log order. Applied entries are appended to the host's own log, so a replayed session shares back identically. Snapshots carry `replayLeft` (sent on change).
- **Forking:** a world-changing command from the page while entries are pending drops the pending entries first (`replayLeft` → 0), then applies and logs the command. The page shows a notice "Replay ended — your edit starts a new branch".
- **Page:** the toolbar shows a chip "Replaying · N edits left ✕" while `replayLeft > 0`; ✕ sends `endReplay` (drops pending entries, keeps the world). Reset (same seed) rebuilds from the session and replays it from the start; 🎲, a preset change and a reset-requiring rule change build a new world with an empty log.
- **Commands:** `init { …, log? }`, `session` (returns the host's log and `logFull`), `endReplay`.

## Share links and session files

- **Wire:** the `#s=` payload gains a version with `e` (the log: entries encoded compactly, deflated with the rest) alongside `c` (config), seed and `g` (starting maps). Links of earlier versions decode with `log = []`.
- **Size:** when the token exceeds 32 000 characters, Share still copies it and says the link is long; Export gains **Session (JSON)** (the same content) and the Share menu an **Open session…** file picker that loads one.
- **Compare links:** `#c=` = deflated `{ a: session, b: session }`, opening straight into Compare with both worlds at t = 0 (each replaying its log).

## Comparison

- **Mode:** a **Compare** toggle (beside Playground | Experiments) splits the grid area into A and B, each a full `Engine` with its own worker. Leaving asks "Keep A / Keep B"; the kept world becomes the playground (its session and state intact).
- **Starting B:** B is built from A's session with A's log truncated to entries with tick ≤ A's current tick T, then advanced to T (the host replays); a progress line "Copying A… t / T" shows until B reaches T. B's `fingerprint` at T equals A's.
- **Lockstep:** a coordinator drives both engines; Play/Pause/Step/speed act on both. It sends `step n` to A and B and waits for both replies before the next step (so their ticks are always equal). Speeds 1×…100× use their `n` per frame; Max in compare mode picks `n` adaptively (double while both replies arrive within 25 ms, halve above 40 ms, 1 ≤ n ≤ 10 000), not the free-running loop.
- **Rebuilds:** Reset rewinds both to t = 0 (each replays its log). A reset-requiring change (or 🎲) to one world rebuilds it and rewinds the other to t = 0.
- **Editing:** each grid has a header (label, seed, 🎲). The Rules tab has a **Rules for: A | B** switch; live changes apply to that world from the current tick. Tools and inspection act on the grid clicked; Inspect and Credit show the last-clicked world, labeled.
- **Charts:** time charts overlay A (solid) and B (dashed), legends "A · <series>" / "B · <series>"; Lorenz and supply & demand overlay the same way; the wealth histogram shows both as step outlines. A chart appears when it would appear for either world.
- **Export:** Statistics/Agents CSV and Grid PNG ask A or B; Charts PNG exports the overlaid charts.
- **Experiments:** switching to Experiments pauses both.

## Recording

- **Controls:** a **● Record** toolbar button with a menu (WebM | GIF, "Stamp the tick", on by default). While recording it shows **■ m:ss**; clicking stops and downloads `…-t<from>-t<to>.<ext>`.
- **Frames:** the grid as drawn (overlays, trail, selection) is copied to an offscreen recording canvas scaled by the smallest integer that makes a cell ≥ 8 px, capped so the long side ≤ 1080 px (nearest-neighbor); the stamp draws `t = <tick>` in a corner. In compare mode both grids go side by side with A/B labels. One frame is captured per displayed snapshot; recording pauses while the world is paused; a reset continues the recording.
- **WebM:** `captureStream(0)` + `requestFrame()` per snapshot into `MediaRecorder`, using the first supported of `video/webm;codecs=vp9`, `video/webm;codecs=vp8`, `video/webm`, `video/mp4` (extension to match); `MediaRecorder.pause/resume` follow the world.
- **GIF:** frames sampled to about 15 fps and transferred (RGBA buffers) to a dedicated worker running `gifenc` (quantize + palette per frame); capped at 900 frames, after which recording stops with a notice; a progress line shows while the file is finished.

## Testing

- **Host (Vitest, fake Sim):** logging (which commands, ticks, cap), replay at tick 0 and later ticks inside `step n` and Max batches, `replayLeft`, forking, `endReplay`, a replayed log re-logged identically.
- **Determinism (real WASM, Vitest):** a session recorded through the engine (paints, places, erases, infections, a live rule change, across ticks at mixed speeds including Max) → encode → decode → replay on a fresh engine → same `fingerprint` at the same tick.
- **Share:** version round trip with a log; earlier-version links decode with an empty log; compare links round-trip; the long-link fallback.
- **Comparison:** coordinator keeps equal ticks (steps, adaptive Max, Reset, one world rebuilding); B's copy reaches A's tick with A's fingerprint.
- **Recording:** frame sampling, scale choice, caps, file names, MIME choice; the GIF worker produces a `GIF89a` file with the expected frame count.
- **Browser (controller):** a shared session replays (chip counts down, final state matches); forking; Compare (copy, diverge by seed and by a live rule, overlaid charts, tools per grid, Keep A/B, compare link); recording WebM and GIF (non-empty files of the right type; no main-thread task > 50 ms while recording a GIF at Max); every existing scenario still passes.

## Docs

README: sessions and share links, Compare, Recording. Roadmap: mark "Replayable edit log", "Side-by-side comparison" and "Recording" done.
