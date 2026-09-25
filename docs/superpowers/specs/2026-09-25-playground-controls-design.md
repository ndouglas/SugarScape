# SugarScape — Playground controls: step back, stop rules, speed readout, shortcuts — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–10 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular 7a's worker engine (`sim-host`, `transport`, `engine`) and 7b's edit log, replay and lockstep.

## Goal

Four playground features: stepping back and scrubbing through a run's history, stopping a run at a tick or when a statistic crosses a threshold, a live ticks-per-second readout, and keyboard shortcuts.

## Non-negotiable constraints

- **No simulation change.** Golden and legacy tests stay unedited and green; nothing that runs a world changes behavior. The core gains only copy/restore and a cheap latest-value read.
- **Exact rewind.** A world reached by seeking to tick T has the same `fingerprint` as the same session run straight to T, at every speed, including when the log holds edits before or after T.
- **Links and sessions unchanged.** Share links, session files and Compare links keep their format; seeking is not logged (the log already describes the whole branch).
- **Civil violence keeps compiling.** The parallel civil-violence branch adds a `ModelWorld` variant and a `Model` impl. Nothing here adds a required trait method or an exhaustive match that a new variant would break; a model without keyframes falls back to replaying from t = 0.

## Behavior

### Step back and the timeline

- The toolbar gains **⟲1** (back one tick) and a **timeline slider** from 0 to `reached`, the furthest tick on this world's current branch. Dragging the slider or pressing ⟲1 moves the world to that exact tick; the grid, charts, Inspect and the followed trail show what they showed then.
- **Playing after going back replays the same future.** Going back turns the log's later entries into edits still to replay (7b's chip counts them), so Play or Step reproduces the recorded future deterministically. Seeking forward (up to `reached`) does the same.
- **An edit branches.** A successful edit after going back drops the pending entries (7b's fork, with its notice), drops keyframes after the current tick and sets `reached` to the current tick.
- A share link or session file taken after going back carries the whole session (applied entries, then pending ones), as mid-replay today.
- **Unavailable:** seeking is refused while the log is full (past `LOG_CAP`, the session no longer rebuilds exactly); ⟲1 and the slider are then disabled with a tooltip saying why. At t = 0 ⟲1 is disabled.
- **Compare:** the lockstep seeks both worlds to the same tick; the slider's end is the smaller of the two `reached`.

### Stop rules

- A **Stop at** control beside Play holds two independent, optional rules:
  - **at tick N**;
  - **when ⟨series⟩ ⟨< | >⟩ ⟨value⟩**, the series chosen from the current model's chartable series.
- Rules are checked after every tick at every speed, Max included, and stop the run on the exact tick. The run pauses and a notice names the rule ("Stopped at tick 812: population < 100").
- **A rule fires on becoming true**, not while it stays true: a condition already true when the rules are set or the run starts does not fire until it has been false and turns true again. "At tick N" fires when a step reaches tick N.
- Rules persist until cleared, survive seeks and resets, and are cleared when a new model kind is loaded (its series differ). They are not part of links or sessions.
- **Compare:** "at tick N" works (the lockstep caps each pair's step at N). The condition rule is single-world only in this version and is disabled in Compare, with a tooltip saying so.

### Ticks-per-second readout

- Beside the speed menu, while running: measured ticks per second over the last second, sampled every ~250 ms from the controls' tick. Hidden while paused. In Compare it counts lockstep pairs.

### Keyboard shortcuts

| Key | Action |
| --- | --- |
| Space | Play / Pause |
| → | Step one tick |
| ← | Back one tick |
| `[` / `]` | Slower / faster (the next speed in the menu) |
| R | Reset |
| ? | Show / hide a card listing these |

Shortcuts are ignored while focus is in an input, select or textarea, while a modifier (Ctrl, Alt, Meta) is held, and while the controls are disabled (a write outstanding). They act on whatever Play and Step drive (one world, or Compare's lockstep).

## Core (Rust)

- **Clone:** `World`, `SchellingWorld`, `RingWorld` and `AnasaziWorld` (and whatever they own) derive `Clone`. `Stats` gains `take_history`/`put_history` (move the history out and back) and `truncate(len)`.
- **Keyframes:** `ModelWorld::checkpoint(&mut self) -> Option<Checkpoint>` moves the stats history out, clones the world, and moves the history back; the `Checkpoint` (an opaque wrapper of a history-less `ModelWorld`) therefore costs about one copy of the current state. `ModelWorld::restore(&mut self, cp: &Checkpoint)` clones the keyframe's world, gives it the live history cut to the keyframe's tick + 1 snapshots, and replaces the live world. Both match the four existing variants and end in `_ => None` / a refusal, so a new variant compiles and has no keyframes.
- **Latest value:** `Model::latest_value(&self, name) -> Option<f64>`, defaulting to the last element of `series(name)`; the four existing models override it to read only the latest snapshot.
- **Wasm:** `Sim::checkpoint() -> Option<Checkpoint>` (a `wasm_bindgen` struct the host holds and frees), `Sim::restore(&Checkpoint)`, `Sim::latest_value(name) -> Option<f64>`.

## Worker host (`sim-host.ts`)

- **Setup kept:** the host keeps the `init`/`reset` command's config, seed, landscapes and log, so it can rebuild the world without the page.
- **Keyframes:** after each step lands on a multiple of K (and once at t = 0 after the initial replay), the host stores `{ tick, applied: log.length, cp }`. K starts at 50; at most 32 keyframes are kept: when a 33rd is due, every other one is freed and K doubles. A world that returns no keyframe (a model without them) keeps none.
- **Full log:** the session is the host's `log` (applied) followed by `pending[cursor…]`. `reached` is the furthest tick on this branch.
- **`seek { tick }`:** refused (a field error) when the log is full or `tick > reached`. Seeking forward (`tick` ≥ the current tick) just advances. Seeking back takes the nearest keyframe at or before `tick` — never one after the current tick, whose history the live world no longer holds — (or rebuilds from the kept setup at t = 0 when there is none), restores it, sets `log` to the full log's first `applied` entries and `pending` to the rest (cursor 0), applies pending entries due at the keyframe's tick, then advances to `tick` (pending entries fire through `replayDue` as usual). Stop rules do not fire during a seek. The reply is a snapshot with the config, landscapes and `replayLeft` resent. A seek to the current tick with nothing to do is a no-op snapshot.
- **Edits:** a successful edit (page or replayed) frees keyframes after the current tick; a page edit also sets `reached` to the current tick (with 7b's fork of pending entries).
- **`setStops { tick?: number; when?: { series: string; op: '<' | '>'; value: number } }`:** replaces the rules and re-evaluates the condition's truth. While a rule is set, `advance` and Max batches step one tick at a time and check the rules after each; a firing rule ends the step (or the Max loop) at that tick, and the snapshot carries `stopped: { reason }`. The condition's truth is also re-evaluated after a seek and on `run`.
- **Snapshots** gain `reached` and `seekable` (sent on change) and `stopped` (once).

## Engine and UI

- **Engine:** `seek(tick)` runs through `quiet()` (Max stops first), drops chart caches as a reset does and keeps the selection and followed agent; `reached`, `seekable`; `setStops(rules)`; a `'stopped'` event with the reason, after which `running` is false. `RunControls` gains `seek`, `reached`, `seekable` and `stops`, so the toolbar drives one engine or the lockstep alike.
- **Lockstep:** `seek(tick)` seeks both worlds (exclusive, like `advance`); `reached` is the smaller world's; "at tick N" caps each pair at N and then pauses.
- **Timeline** (`ui/timeline.ts`): ⟲1 and the slider. While dragging, one seek is in flight at a time and the newest position replaces any not yet sent.
- **Stop control** (`ui/stop-control.ts`): the two rules, the series list rebuilt when the model changes.
- **Speed readout:** in `ui/toolbar.ts`.
- **Shortcuts** (`ui/shortcuts.ts`): one `keydown` listener on the document and the help card.

## Testing

- **Core:** for every model, run to T, checkpoint, run on, restore, run to T′: the fingerprint and every series equal a fresh run to T′. `latest_value` equals the last element of `series` for every series of every model. Golden and legacy suites unchanged.
- **Host (Vitest, fake module):** seek back and forward gives the fingerprint of a fresh run to the same tick, with log entries before, at and after the target and at a keyframe's own tick; an edit after seeking forks and frees later keyframes and lowers `reached`; thinning keeps ≤ 32 keyframes and doubles K; seeking is refused with a full log and past `reached`; stop rules fire on the exact tick at 1×, 100× and Max, and only on becoming true.
- **Engine / lockstep:** seek keeps the selection and refetches charts; Compare seeks both worlds to one tick; "at tick N" in Compare stops both at N.
- **Determinism:** a session with seeks, edits after seeks and mixed speeds, shared through a link, replays to the same world at the same tick.
- **UI:** shortcuts are ignored in inputs and with modifiers; `[`/`]` walk the speed list and stop at its ends.

## Docs

README: the timeline, stop rules and shortcuts in the playground section. Roadmap: this milestone under "Playground and infrastructure".
