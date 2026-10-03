# Episode 9: El Farol Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development task by task. The user has approved the storyboard and music and requires this execution method. Do not stage or commit.

**Goal:** Build and verify “Nobody goes, it's too crowded,” with native recorded decisions, prospectively measured captions, the approved original score, and a 960 × 540 preview.

**Architecture:** Freeze and measure a fresh corpus, then export actual before-decision inputs and after-decision outcomes without changing native rules. Adapt the existing studio crowd, felt histogram, reserved diagrams, and standard closing animation. Use one continuous original score at one tempo.

**Tech Stack:** Rust native shot exporter and CLI; Python unittest, measurement and animation; Blender; ABC/abc2midi/FluidSynth; FFmpeg.

**Spec:** `docs/superpowers/specs/2026-10-02-farol-spike.md` (approved October 3).

## Global Constraints

- Existing `.claude/worktrees/crowd`, branch `crowd`; preserve all preceding dirty work.
- No staging or commits; never stage `.claude/`, `papers/`, or `survey/out/`.
- American spelling; Agent in code/docs and Flump only in captions/video.
- All decisions from the native model, one visible agent per real agent; simultaneous decisions.
- N = 100 bar scenes; N = 101, S = 2 minority comparisons at M = 2, 6, 12.
- Fresh bar seeds 1001–1020, 2000 rounds, burn-in 400; fresh memory seeds 2001–2032, 10000 rounds, burn-in 2000.
- Rules and example selection frozen before measuring; retain failures; change unsupported captions instead of data. Disclose post-result rule revisions.
- Caption layout separate from all panels; normal closing `<title> - After <Names>, <Year>\nndouglas.github.io/SugarScape`, one camera-facing agent, one late blink.
- Original schottische-inspired G-major 4/4 music: clarinet, quiet accordion, plucked guitar; one continuous fitted score, no independently timed stings or ducks.
- Full native/WASM/web/studio checks; rustfmt only edited survey files if any (none planned).

## Review Focus

1. Before/after mismatch: selected forecast, virtual score and public history must be those actually used, not the next round's inspection.
2. Sampled clocks: selected film windows preserve actual rounds and saved frame indexes; initial tick is not a played round.
3. Semantic bounds: A = 60 is crowded, default forecast = 60 advises staying; odd-N minority selection matches native rules.
4. Visibility: every agent and dynamic readout stays visible; captions, panel cards and teaching markers never occlude one another.
5. Music: single tempo/stream, coherent handoffs, audible continuity and no clipping; closing card matches prior episodes.

### Task 1: Freeze and measure the corpus

**Files:** Create `studio/episodes/farol/claims.py`, `measurements.json`, `measurements.md`, `shots/*.json`, `studio/tests/test_farol_claims.py`.
**Consumes:** Native CLI `run`, full configs, `studio/measure.py`, approved spike.
**Produces:** `measure(tmp) -> (lines, verdicts, data)`; data keys `protocol`, `runs`, `cases`, `selected`, `histograms`; shot names `accuracy`, `advice`, `random`, `shared`, `m2`, `m6`, `m12`, plus a slow single-round teaching shot if necessary. `selected` includes seed, true start/end round, sampling interval, source config, and representative-run rationale. Each run retains mean, variance about its own mean, center squared deviation, lag-1, and histogram; complete measured outcome counts sum to the post-burn horizon.

- [ ] Freeze all case configs, seed lists, burn-in, rules, selection methods and provenance before first run. Accuracy/advice/random/shared use complete explicit El Farol configs; memory uses complete explicit plain minority configs.
- [ ] Write failing tests for exact seed counts/horizons, hand-series statistics, exclusions of initial/burn-in rows, histogram totals and deterministic median-variance selection. Pin ±2 mean and ±0.05 normalized variance margins; comparisons use retained paired ordering/effect sizes and existing statistical support.
- [ ] Implement native-only CLI collection/reduction, frozen rules and deterministic selection. Retain selected traces and all per-seed outcome summaries/configs/hashes; raw cache stays ignored.
- [ ] Run `python3 studio/measure.py farol`; verify all caption claims or return unsupported captions with evidence for correction.
- [ ] Generate shot specs using measured selections, keeping every frame window and complete config reproducible; export waits for Task 2.
- [ ] Run targeted tests and self-review; write scoped report and delta in this plan's ignored workspace. No commit.

### Task 2: Export actual decision snapshots and load them

**Files:** Modify `crates/sugarscape-core/src/farol/world.rs`, `crates/sugarscape-core/src/farol/mod.rs`, `crates/sugarscape-core/src/frames.rs`, `studio/dump.py`; create `studio/tests/test_farol_dump.py`.
**Consumes:** Task 1 shot specs; native `FarolWorld` and format-1 shot conventions.
**Produces:** `farol` dump with config, population, game, sampling interval, frames and stats. Each frame stores true round, attendance/capacity, agent IDs/choices/memory, actual selected strategy index and score at decision time, forecasts or table advice at decision time, pre-decision public input history, and winning-side outcome. Initial frame has no fabricated decision. Python loads this into existing `Dump`/`Frame` structures with domain metadata/members used by Task 3. Layout coordinates are illustrative and stable by agent ID and recorded choice.

- [ ] Write failing Rust tests: actual post-round membership counts; boundary at exact capacity; selected forecast/action agreement; pre-decision history vs post-outcome; selected score before update; sampled periods; recorded/non-recorded RNG and series parity; reject incompatible hand placement and every=0.
- [ ] Implement a read-only decision snapshot/recording wrapper around actual native steps. Never select or draw RNG twice; keep regular runs/golden outputs unchanged. Use existing exporter branch patterns.
- [ ] Write failing Python tests for metadata/membership/history loading, stable unique IDs, populations and sampled true rounds.
- [ ] Implement loader with stable home/bar or A/B coordinates, felt attendance histogram support and actual member data. Communicate exact field schema in report.
- [ ] Run targeted core and Python tests; build release CLI and generate all selected shot dumps. Verify dump statistics reproduce measured windows and CLI trajectory.
- [ ] Self-review and scoped report/delta; no commit.

### Task 3: Build the approved episode and original score

**Files:** Create `studio/episodes/farol/beats.py`, `tune.py`, `studio/farol_visual.py`, `studio/blender/overlays/farol.py`, `studio/tests/test_farol_episode.py`, `test_farol_visual.py`, `blender_farol_layout.py`; modify shared scene/animation/overlay registry only where needed for farol integration.
**Consumes:** Tasks 1–2 data/schema; fourteen exact approved captions in spike; standard Beat/camera/timing and closing helpers.
**Produces:** Complete episode, simultaneous recorded crowd motion, bar/home or A/B felt areas, reserved lookup and attendance diagrams, actual histogram and ensemble labels, and `TUNE` original continuous score.

- [ ] Write failing pure tests for simultaneous transition timing, actual forecast advice, memory bit ordering/table lookup, even/odd attendance, histogram aggregation, approved caption list and measured case linkage.
- [ ] Implement minimal pure adapters, fourteen beats, population-preserving positions/colors and domain overlays. Dynamic panels use actual recorded inputs; labels distinguish our forecast bank, known coin probability, selected run and measured ensemble.
- [ ] Build one shared-tempo G-major 4/4 original score from approved ABC subject, five-strain development and G–D ending. No stings/cues/ducks. Test voice bar lengths, form fit and melodic handoffs.
- [ ] Add actual Blender regressions across every frame: caption/text bounds, panel separation, diagram geometry, all actual agents represented, synchronized decision poses, one closing agent and one late blink.
- [ ] Run pure and Blender tests, compile ABC/MIDI and synthesize music; inspect peak and interior continuity. Save scoped report/delta; no commit.

### Task 4: Verify, review, render and deliver

**Files:** Update studio README, approved spike status and series plan; generated stills/preview stay in ignored studio/out and final movie directory.
**Consumes:** Complete tested episode.
**Produces:** Current `~/Movies/Flump Studio/sugarscape-027-farol.mp4`, measurement report, verified first/middle/end stills, final checks and review.

- [ ] Use the existing scratchpad stills workflow at 50% and 16 samples; inspect all fourteen first/middle/end views. Correct real layout problems through implementer and scoped review.
- [ ] Run `cargo fmt --all -- --check`, strict workspace Clippy, `cargo test --workspace`, `wasm-pack test --node crates/sugarscape-wasm`, web WASM build/TypeScript/vitest, and studio unittest suite. Run real Blender regressions after final visual changes.
- [ ] Build `python3 studio/build.py farol --preview`; verify dimensions, frame count, matched stream duration, complete decode, musical continuity and final-card/blink samples.
- [ ] Independent final reviewer checks episode integration, source qualifications, measured captions, data timing, music and closing. Fix findings through scoped worker/re-review.
- [ ] Deliver one current movie without duplicate exports. Mark docs built, complete ledger, remove temporary IMPLEMENTATION_PLAN.md. No stage, commit, merge or push.
