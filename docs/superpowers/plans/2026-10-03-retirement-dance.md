# Retirement dance implementation plan

> Use superpowers:subagent-driven-development. The user approved the storyboard and appearance; execute without an additional approval question. No commits.

**Goal:** Replace the retirement dashboard presentation with a personal dance narrative and a verified 98-second preview.
**Spec:** `docs/superpowers/specs/2026-10-03-retirement-dance-design.md`.
**Architecture:** Pure pose/cast helpers in `studio/retirement_dance.py`; model-specific character/age renderer in `studio/blender/retirement_cast.py`; existing retirement overlays and beats consume them. Keep native recordings, measurements, parser and tune unchanged. Root owns ledger, visual inspection and delivery.

## Task 1: Stage 1 — Recorded cast, elderly appearance and polska motion
**Goal:** Test-first reusable helpers and real elderly rig/pose rendering.
**Success Criteria:** Working and retired elders share age cues; only retirees take dance steps; young replacements lose old cues; stable actual slot/birth identities;104 BPM 3/4 continuous phases with varied modest styles. Recurring watcher has spectacles/silver eyebrows and unobscured expressive eyes.
**Tests:** Pure pose tests at phrase boundaries, retired/working invariance, renewed identity, actual eligible-kind selection. Blender elderly/young still and accessory/blink geometry checks.
**Status:** Complete

- [x] Snapshot before, write assertion-based failing behavioral tests, implement `dance_pose(seconds,member)` with immutable explicit pose values and `age_style(member,hero=False)` derived from age/identity. Do not alter generic rig behavior for other episodes.
- [x] Implement model-specific `build_character(name,member,hero=False,low=False)`/`pose_character(rig,member,seconds,dancing=None)` using existing yarn/eye rig and actual recorded state. Optional `dancing` override only for the recorded before/after teaching event. Spectacles frame glossy eyes without covering pupils; age cues are outside eye blink hierarchy. Large close-up face/readability proof required.
- [x] Run targeted Python and actual Blender fixture checks; independent scoped reviewer gates the helper interface before Stage 2.

## Task 2: Stage 2 — Approved fourteen-beat film revision
**Goal:** Personal felt-stage scenes using actual recorded characters and brief measurements.
**Success Criteria:** Exact revised caption 1/other 13 captions and 98 s 2940 frames; real 13-neighbor teaching; complete 8,100 background marks without duplicated focus agents; initial-state return after cold open; mode 65/share 13.8/native 5; real group links/configs and policy/censor clocks; quiet same watcher closing with one blink.
**Tests:** All 14 first/mid/end Blender projected bounds and source IDs/status/aging/dance assertions; exact caption/duration test; native rolling 263 events/rate 55/439 and legends retained in any graphs.
**Status:** Complete

- [x] Implement scene/overlay revision in retirement-specific files. Current `scene.build_beat` retirement branch already skips generic population. Choose actual eligible members for habit demonstrations; use actual event before-decision friends for teaching/cold open and recorded period-end frames for populations. Workers do not dance, retired agents never unretire while same birth identity persists.
- [x] Use stage-focused cameras, large faces, gaze and dance motion. Replace permanent cards with brief graphs/readouts and adequate method labels. Retain all 8,100 in batched distant rendering; close-up rigs replace those slots rather than add new members. Avoid 8,100 detailed rigs.
- [x] Update `beats.py`, episode/layout tests and original spike to revised approved storyboard, citing this design. Source measurement products and tune must remain byte-identical. Run full Studio suite; reviewer assesses source-data/interface correctness before full preview.

## Task 3: Stage 3 — Composited stills and technical checks
**Goal:** 42 reviewed actual composites with clear age and retirement expressions.
**Success Criteria:**30%16 sample renders, captions unobscured, grandparents recognizable and alive; full repository checks appropriate to files touched green.
**Tests:** Existing full gates Rustfmt/strictClippy/native/WASM/web/Studio/survey; no repeated broad testing after unchanged passes. Real Blender all 14 source/layout/closing guards.
**Status:** Complete

- [x] Root renders 42 first/mid/end samples with copied ignored scratch runner; each is inspected at readable size. Fix concrete findings through the implementer and independent review; rerender affected samples.
- [x] Root verifies required full gates once, and absence of native/config/measurement/tune modifications. Retain unchanged audio/MIDI and source provenance.

## Task 4: Stage 4 — Verified replacement preview and final review
**Goal:** User-reviewable canonical 028 dance revision.
**Success Criteria:**98 s 2940 frames 960×540, 30 fps with audio; full decode and PCM continuity; decoded transitions and elderly watcher’s single late blink; independent whole-change review clean. Human subjective listening/playback is explicitly pending.
**Tests:** `python3 studio/build.py retirement --preview`; ffprobe, full FFmpeg decode, PCM check, decoded frame samples. No commit/staging.
**Status:** Complete

- [x] Preserve old canonical movie in ignored scratch, build revision, verify before replacing canonical movie. One delivery export remains in Movies.
- [x] Final independent review checks code, full source integrity, stills and media. Update status, remove `IMPLEMENTATION_PLAN.md` when agent delivery is done, preserve uncommitted branch and ledger.

## Delivery acceptance

The user accepted the final dance and transition revision and authorized committing the episode on October4. The original no-staging/no-commit delivery constraint above is superseded by this authorization. Automated and sampled visual verification do not claim subjective listening or full normal-speed viewing by the agent.
