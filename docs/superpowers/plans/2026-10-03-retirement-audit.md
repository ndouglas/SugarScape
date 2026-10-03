# Episode 10 retirement source and numerical audit

> Use subagent-driven development for any corrections. The user authorized continuing the established series; the storyboard and tune require approval before building. Do not commit new corrections without asking.

**Goal:** Recheck milestone 26 and prepare an accurate episode proposal.
**Architecture:** Read original and revised sources separately, reconstruct disputed figures from the native engine, cross-check core mechanisms with an independent minimal implementation, then correct claims and write the spike.
**Tech Stack:** Rust, native CLI, Python analysis/plots, local PDF extraction/rendering.
**Spec:** docs/superpowers/specs/2026-09-27-retirement-design.md; series process in docs/superpowers/specs/2026-09-29-crowd-series-plan.md.

## Constraints

- Existing crowd worktree. Previous completed episodes committed as 457db4b.
- American spelling; Agent in code/docs, Flump in video.
- Sources before storyboard; finite-horizon proxy failures cannot establish that a paper is false.
- Original AE and revised GSS policy setups differ: homogeneous threshold .5 versus uniform [.5,1].
- Freeze probe rules before runs; 50 seeds for sensitivity figures where the source uses 50; retain censored outcomes and distinguish conditional means.
- Track retired/eligible, actual retirement-age mode and age-specific events separately. A 95% proxy crossing does not establish a retirement-age norm.
- Compare paper grids: extent 6–10 for 5% rational, positive threshold spreads, coupling .05–.20.
- Keep Slot and Replace explicit reconstruction choices. Do not use Replace+groups until cross-group replacement is correct.
- Store raw outputs and full paper text only in ignored workspace/survey/out. Retain compact reproducible audit evidence in docs later.
- No commit of retirement corrections before user approval. Never stage .claude/, papers/, survey/out/.

## Task 1: Source review

- [x] Read paper, revised chapter, design, README, ledger, survey and model.
- [x] Render figures and anchor each disputed reading to the source.
- [x] Distinguish source claims from reconstruction metrics and unspecified choices.

## Task 2: Frozen numerical probes

- [x] Native runs: original/revised policy thresholds, automatic and fixed/sustained warm-up diagnostics; per-age outcomes and imitator events; 50 seeds for source sensitivities.
- [x] Native figure reconstructions: trajectories, rationality/renewal, threshold spread, extent, coupling and policy. Preserve nonattainment and uncertainty.
- [x] Run survey --only retirement and retain all verdicts unchanged before corrections.
- [x] Separately implement a minimal source mechanism and cross-check selected base, all-members and revised-policy ensembles. RNGs may differ; compare distributions.
- [x] Create side-by-side original/native plots, declared source approximations and evidence summaries.

## Task 3: Corrected implementation and claims

- [x] For definite engine defects, write a failing regression, implement minimal correction, verify native/WASM/golden preservation.
- [x] Correct reading errors and qualify operational tests. State in changed claim text when its rule was revised after the result was known.
- [x] Have an independent reviewer check the corrected scientific claims.
- [x] Run required full checks after code changes. Corrections remain uncommitted for user approval.

## Task 4: Episode spike and approval

- [x] Write docs/superpowers/specs/2026-10-03-retirement-spike.md with survey verdicts and corrected evidence.
- [x] Choose filming that preserves actual agents and makes aging, imitation and age norms readable.
- [x] Write about fourteen exact captions and an original ABC tune; compile the draft.
- [x] Present storyboard and tune for approval before build.
