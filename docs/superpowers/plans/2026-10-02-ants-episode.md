# Ants episode implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. No staging or commits.

**Goal:** Build and deliver the approved fourteen-beat Episode 8 preview, measured against the approved prospective rules.
**Architecture:** Extend the existing shot export with actual source memberships and optional recorded update events. The episode measures the native model, then uses those results to choose short filmed runs; Blender renders two equal food piles and reserved diagrams, and the shared music system renders the approved original subject.
**Tech Stack:** Rust core and CLI; Python unittest and studio; Blender; ABC/MIDI, FluidSynth and FFmpeg.
**Spec:** docs/superpowers/specs/2026-10-02-ants-spike.md (approved October 2).

## Global Constraints

- Leave all changes uncommitted. No staging, merge or push.
- Work in the existing crowd worktree; preserve every prior change.
- Use American spelling and "agent" in code and documentation; the video calls the characters Flumps.
- Every filmed behavior comes from a real run at the shot's actual population size.
- Prospective episode measurement rules and seed lists are fixed before measuring. Preserve all outcomes; explicitly disclose revisions after results, never widen a threshold silently.
- Use at least20seeds,1,000short-run means, and300,000AMsweeps as specified in the approved spike.
- Exact captions and the Equal Portions subject are those in the approved spike.
- Any highlighted recruitment pair or spontaneous-switch event must be recorded by the real update, not inferred or invented.
- Closing title/attribution plus URL float above a single camera-facing agent with one late blink.
- Check every animated frame and separate caption for clipping, text overlap and opaque card coverage.
- Scratchpad stills:50%size,16samples. Final preview:960x540 in ~/Movies/Flump Studio/.
- Never stage .claude/, papers/ or survey/out/. Never cargo fmt inside survey/.

## Review Focus

- Event recording must not consume random draws or change model trajectories; compare recording on/off across both conversion modes and AM.
- Strided dumps preserve true ticks/time units and all actual agents; no animation-time clock substitutions.
- Measurement baselines and non-herders retain correct horizons, population denominators, independent samples and graph identities.
- Text remains readable at all camera movement endpoints and during diagram changes.
- Music cues follow actual filmed flips; closing animation has exactly one late blink and required two-line text.

### Task 1: Actual ants shots and Python loading

**Files:** Modify crates/sugarscape-core/src/ants/world.rs, crates/sugarscape-core/src/ants/mod.rs if needed, crates/sugarscape-core/src/frames.rs, studio/dump.py; create studio/tests/test_ants_dump.py. Preserve existing exporter branches.
**Consumes:** Shot(config/seed/ticks/every/gifts), AntsWorld actual source, independence, neighbor data.
**Produces:** Dump.model='ants'; per-frame members[id] with id/source/independent/degree/elsewhere; actual counts; period=true model tick; ants_events list with kind/agent/partner/from_source/to_source and true update ordinal. Retain fields under `Frame.members`, add `Frame.ants_events`, `Frame.counts` if needed. Source numbers remain1-based in public data. Use one meeting per tick for micro shots; optional gifts=true activates recording. A recruit event includes sampled partner, a spontaneous event has partner null. Record actual changes only; no fabricated encounters.

- [x] Write failing Rust tests comparing dumped memberships/counts/clock against separately stepped AntsWorld at every saved frame; include every=3,stop_at,invalidstride and unsupported placements. Python fixture checks identity/source/independence/clock/events.
- [x] Run focused exporter and Python tests to demonstrate RED.
- [x] Add opt-in update recording without any extra RNG consumption; clear per-step event buffer, retain ordered actual transition metadata. Add readable views/accessors for exporter without exposing unrelated internals.
- [x] Implement `run_ants`, serde dump enum branch and Python `_ants` loader. Place agents in deterministic slots at their actual chosen source; keep identity through switches. Export real links for noncomplete graphs, no dense complete-graph link list.
- [x] Prove recording-on/off equality across seeds and both Kirman conversion modes plus AM, including identical config and exact source sequences. Check no independent agent is recorded as recruited. Run exporter tests,core ants tests and studio tests. Capture isolated diff/report; no commit.

### Task 2: Prospective measurements and selected shot inputs

**Files:** Create studio/episodes/ants/claims.py, measurements.json, measurements.md, shots/*.json, studio/tests/test_ants_claims.py. No exporter or visual edits.
**Consumes:** nativeCLI/model and Task1 ants dump. `claims.measure(tmp)` returns(lines,verdicts,data) following studio/measure.py.
**Produces:** retained configs/rules/horizons/seed lists, per-run outcomes and useful recorded traces in measurements.json; metadata under `protocols`, `selected`; shotfiles micro-recruit,micro-self,strong,short,pull,large,ring,random,independent with actual seeds/frames/time units. Exact choices are communicated to Task3 via report, never assume filenames silently.

- [x] Implement tests of boundary judges using fabricated summaries, verify samplecounts/horizons/seed disjointness and deterministic smallesteligible-seed/earliestevent selection. Test every sample remains in summaries and counts.
- [x] Implement fixed configs and rule constants copied verbatim from approved spike before launching any measurement. Document micro horizon1,000meetings per seed20streams, seeds200001–200020; count actual recruitment/spontaneous transitions and select earliest actual qualifying events. Long/short seeds exactly approved; same numeric seeds across configurations are identified as paired only where actual graph equality is verified.
- [x] Measure native CLI series with at most4workers and stream/reduce large CSVs rather than retain all parsed dictionaries. Keep per-run scalar statistics and histograms/selected short traces; all outcomes retained. Reuse long1000baseline for q0 only if full configs/horizons are identical. Never replace native runs with the independent audit simulator.
- [x] Run `python3 studio/measure.py ants`; all approved prospective judges must be evaluated unchanged. If a judge fails, report exact result to controller before revising any caption/rule. Retain all outcomes and any disclosed revisions.
- [x] Produce actual short shotJSONs with bounded ticks/every to avoid enormous animation dumps. Histogram overlays may use separately labeled20-run measured aggregates. Recorded event windows and flip examples come from retained corpus by documented selection; export their actual event metadata. No handplaced choices or invented graph links.
- [x] Run focused claims tests and full studio suite; self-review config/time-unit alignment. Save isolated patch/report, no commit.

### Task 3: Approved visuals, score and text layout regression

**Files:** Create studio/episodes/ants/beats.py,tune.py, studio/ants_visual.py, studio/blender/overlays/ants.py, studio/tests/test_ants_episode.py,test_ants_visual.py,blender_ants_layout.py. Modify studio/blender/overlays/__init__.py,scene.py,animate.py only where required by model integration; existing episodes remain unchanged.
**Consumes:** Task1 dump fields, Task2 protocols/selected/shotfiles. Produces complete fourteen approved beats and original score in existing studio build system.

- [x] Write failing tests for source-position/identity mapping,event highlight from recorded data, honest histogram/trace inputs, exact fourteen captions,normalclosing format andoneblink. Use current overlay conventions; do not rewrite other episodes.
- [x] Stage two identical food piles. Preserve agent identities and actual source choices; distinctly mark actual non-herders. Teach events with recorded pair/switch only. Allocate stage,diagram and caption regions; no floating panels over world text. Diagram labels distinguish selected run,exact stationary prediction and multi-run measurement.
- [x] Build all fourteen beats from approved caption table (about98seconds afterdissolves), reasonable matching cameras/timing. For the growth comparison show all realpopulation-size shots or explicitly labeled measurement diagrams, never pretend a100-agent board is1000agents. Actual opposite-neighbor count and links teach AM; diagrams summarize large networks.
- [x] Implement Equal Portions using approved eight-bar6/8D-Dorian subject,clarinet,bassoon,vibraphone with stereo roles,development and ending. Cue roleswap to the actual filmed flip time. Add stings/ducks only if needed to make approved musical actions audible; no dynamics change or invented flip. Validate barlengths,forms,ABC/MIDI and final synth.
- [x] Adapt the persistent real-Blender all-frame text projection test from thresholds: every beat frame plus each separatecaption,viewport clipping,text-text and unrelatedopaque-card coverage. Check animation endpoints and requiredclosingblink. Run RED before minimallayoutfix if failing; persistGREEN tests.
- [x] Render scratchpad first/middle/end stills at50%size16samples for all fourteenbeats; save contactsheet. Run full studio suite and actual Blender layout regression; save isolated diff/report, no commit.

### Task 4: Integration verification and delivery

**Files:** Update approved spike status, studio/README.md and seriesplan completedrow8; remove temporary IMPLEMENTATION_PLAN.md when done. Root owns delivery.

- [x] Independently review each task delta and address concrete findings before its dependent task begins. Final reviewer checks episode integration,measuredcaptions,actualworld-data,musiccues,closingcard,tests; no duplicate fullsuite run by reviewers.
- [x] Run full required checks once finalcode is stable: cargo fmt; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace; wasm-pack test --node crates/sugarscape-wasm; web npm run wasm,npx tsc --noEmit,npx vitest run; studio unittest discover with -s studio/tests -t studio. Survey filesunchangedbybuild; no surveycargo fmt.
- [x] Inspect50%16sample contactsheet; fix material readability problems via implementer and coveringtests. Build `python3 studio/build.py ants --preview`.
- [x] Verify960x540 framecount,duration andmatching audio; inspect actualfinalmovie samples for recordedrecruitment/spontaneousswitch,flip,largerpopulation,network/nonherderdiagrams andlateblink. Copy verifiedpreview to ~/Movies/Flump Studio/sugarscape-026-ants.mp4. No commit or staging.
- [x] Mark completedepisode docs with actualmeasurements/testevidence anddeliver movie link plus concise results andlimitations.
