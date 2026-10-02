# Q-learning Auctions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Banchio & Skrzypacz's fixed-value Q-learning auctions as the `auctions` kind, with inspectable first/second-price learning, information interventions, the source extensions and preregistered reproduction checks.

**Architecture:** A separate core module implements the auction mechanism, stateless learners, fixed-horizon world, terminal analysis and views. The same sequential engine serves the CLI, WASM playground, twelve finite sweeps and standalone survey; pricing collusion remains unchanged. Thirteen presets expose literal source settings and explicitly named reconstruction choices.

**Tech Stack:** Existing Rust core/CLI, wasm-bindgen, TypeScript/Vite/Vitest/uPlot and standalone survey; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-02-q-learning-auctions-design.md`, approved and committed before learning measurements. Read its reading notes and recovered source-figure JSON alongside it. The approved base is `4b72b039bd1bd0352f948a533d2ebb2815dd817c`.

## Global Constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add new golden entries; do not refactor the pricing engine.
- One sequential learner path serves native and WASM: `f64` random draws, portable `exp_neg` on nonpositive arguments, explicit weighted Q updates, no FMA or separate accelerated path.
- Baseline: two bidders, value 1, nineteen bids k/20, learning rate .05, discount .99, epsilon `.025*exp(-.0002*t)`, horizon 1,000,000, window 1,000, periods per tick 1,000.
- Every run reaches its configured horizon; final stability selects terminal summaries and never stops a session early. Preserve all outcomes and discarded seeds.
- Keep page cap 1,000,000 ticks; sweeps at most 100 seeds and 100,000 ticks. A 100m-period preset uses 100,000 ticks. Partial final batches execute exactly the remaining periods.
- Defaults for unspecified details are reconstruction choices, not recovered author code: discounted optimism, scale 1, sampled auction ties, expected hindsight ties, lowest greedy tie, zero time origin, post-update stability. Every ambiguity stays a named switch.
- Disclosure and using disclosure are separate fields. Chosen updates with unused rival feedback must be economically identical under the same seed.
- `.90/.90` is a weak FPA equilibrium; below-top profiles and profitable deviations are separate measures. Source Figure 1 FPA .2265 and textual .24 are scored separately; SPA figure .9471 and prose .95 are separate targets.
- No original simulator was located in the searched public sources. Do not claim author-code docking. PDFs/source archives remain ignored in `papers/`.
- Use the committed fixed rules: terminal coverage >=95%, figure TV<=.10 and largest-bin gap<=.05, specified mean/extension tolerances and fixed published-count ensembles. No switch or rule is selected by fit.
- Whole/late occupancy counts every auction; the last window is completed periods `floor(.8*T)+1..T`. Figure 1 terminal-policy histograms are a different statistic.
- Chart history is bounded at 1,000,001 snapshots: beyond the page tick cap retain the initial prefix and latest snapshot with actual tick stamps; economic totals/occupancy remain complete. Native checkpoints retain the latest snapshot across a history gap.
- Full surveys and duration studies are outside CI. Binary proportions use session SE and Wilson 95% intervals; paired effects use a labelled covariance-aware delta-method interval without changing point-based verdict rules.
- Use subagents for execution without asking which method to select. Obtain a fresh whole-branch review of rules and findings before integration. Assign the milestone number and move Queue #1 only at merge.

## Review Focus

1. Invalid inactive numeric fields and extreme grid sizes must return field errors, without overflow or long loops: `config::tests::oversized_grid_returns_field_error_without_overflow_or_membership_search` and finite/range validation cases in Task 1.
2. A final partial tick, transient stability or fixed positive epsilon must not shorten a horizon: `batching_preserves_economic_state_and_finishes_partial_tick`, `transient_stability_does_not_finish_session`, CLI partial-batch test and real-WASM tests in Tasks 1–2.
3. Small grids with three bidders and a fringe must keep Values readable and hypothetical competitors inspectable: pixel-marker/fringe tests, third-series tests and real `GridView.draw` tests in Tasks 1–2.
4. Full-feedback updates must use one frozen old maximum and coupled realized priorities; unused disclosure must not consume extra randomness: frozen-bootstrap, hindsight-priority and ten-seed identity tests in Task 1, full 100-seed control in Task 4.
5. Different convergence subsets must preserve each conditional population and its denominator, rather than silently pairing only selected intersections: paired influence/coverage fixtures, unordered off-diagonal TV, Wilson boundaries and whole-played-pair denominator tests in Task 3.

## Verified code delivery

The complete executable code is in six adjacent `.patch` files, split into tests and implementation for three tasks. These are source code artifacts, not instructions to invent the implementation. Each task's file inventory below is exact. Read its patch before applying it; the patch carries all implementation and test bodies, including the full configuration schema and every survey decision rule.

`manifest.json` records SHA-256 preimages and final bytes for all 57 product files and every patch. `replay.py` checks them and reconstructs the code on a descendant of the approved base. The original verified scratch worktree is `~/.config/superpowers/worktrees/SugarScape/auctions-scratch`; a separate clean worktree verifies the plan's staged red/green cycles and byte equality. Product changes remain there until plan review; the approved branch contains this plan and its code artifacts.

Set the task variable once, from the repository root:

```bash
task_assets=docs/superpowers/plans/2026-10-02-q-learning-auctions
```

When replaying from a base without the plan artifacts, set it to the absolute artifact directory in the approved `auctions` worktree. Check manifest preimages before applying; an integration conflict must be resolved explicitly, preserving changes already on main. Do not overwrite unrelated work to make a hash match.

### Task 1: Auction engine and model discovery

**Files:**
- Create: `crates/sugarscape-core/src/auctions/analysis.rs`
- Create: `crates/sugarscape-core/src/auctions/config.rs`
- Create: `crates/sugarscape-core/src/auctions/learner.rs`
- Create: `crates/sugarscape-core/src/auctions/mechanism.rs`
- Create: `crates/sugarscape-core/src/auctions/mod.rs`
- Create: `crates/sugarscape-core/src/auctions/presets.rs`
- Create: `crates/sugarscape-core/src/auctions/stats.rs`
- Create: `crates/sugarscape-core/src/auctions/view.rs`
- Create: `crates/sugarscape-core/src/auctions/world.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`
- Modify: `crates/sugarscape-core/src/model.rs`
- Modify: `crates/sugarscape-core/src/presets.rs`
- Modify: `crates/sugarscape-core/src/titles.rs`
- Create: `crates/sugarscape-core/tests/auctions_discovery.rs`
- Modify: `crates/sugarscape-core/tests/golden.rs`

**Interfaces consumed:** `Model`, `ModelConfig`, `ModelKind`, `FieldError`, `Param`, `Stats<AuctionsSnapshot>`, `Series`, existing seeded `SimRng`, `portable::exp_neg`, canvas/export helpers.

**Interfaces produced:**

```rust
AuctionsWorld::new(config: AuctionsConfig, seed: u64) -> Result<AuctionsWorld, Vec<FieldError>>
AuctionsWorld::run(&mut self, ticks: u32)
AuctionsWorld::step(&mut self)
AuctionsWorld::period(&self) -> u64
AuctionsWorld::is_finished(&self) -> bool
AuctionsWorld::outcome(&self) -> Option<&Outcome>
AuctionsWorld::grid(&self) -> &[f64]
AuctionsWorld::learners(&self) -> &[Learner]
analysis::evaluate(config: &AuctionsConfig, bids: &[f64]) -> Evaluation
analysis::equilibria(config: &AuctionsConfig) -> Vec<Vec<f64>>
```

`Outcome` serializes config/seed, final policies/Q/action counts, stability/activation, static equilibria, expected terminal payoffs/deviation gains, whole/late realized means and histograms. `ModelConfig::Auctions` and `ModelWorld::Auctions` provide the existing generic interfaces. Third-bidder series are unavailable with two bidders. Inspect exposes current fringe bid/share alongside strategic actions. The complete enums and field types are in `01-core-implementation.patch`.

- [ ] **Step 1: Create four-stage tracking.** Write `IMPLEMENTATION_PLAN.md` with the exact tracking content below. Stage 1 becomes In Progress now; update subsequent stages when their work begins.

```markdown
# Q-learning auctions

## Stage 1: Auction engine
**Goal**: Shared deterministic engine, config, analysis and model dispatch.
**Success Criteria**: Hand-derived mechanism/protocol tests and existing goldens pass.
**Tests**: Core suite, model discovery, portable short fingerprints.
**Status**: In Progress

## Stage 2: Playground and hosts
**Goal**: CLI/WASM/web integration, thirteen presets and twelve sweeps.
**Success Criteria**: Exact horizons, Inspect, conditional charts and finite sweeps agree with core.
**Tests**: CLI, WASM, web tests, TypeScript and production build.
**Status**: Not Started

## Stage 3: Preregistered survey
**Goal**: Fixed-count source checks, controls and complete session exports.
**Success Criteria**: Coverage, statistic definitions and uncertainty match the approved spec.
**Tests**: Synthetic judging fixtures and short identity checks.
**Status**: Not Started

## Stage 4: Findings and integration
**Goal**: Measure registered workloads, document outcomes, review and publish.
**Success Criteria**: Findings preserve source discrepancies and unknown conventions; required checks and deployed Pages pass.
**Tests**: Native ensembles outside CI, fresh review, CI and deployment smoke.
**Status**: Not Started
```

- [ ] **Step 2: Install the failing model-boundary test.**

```bash
git apply "$task_assets/01-core-tests.patch"
cargo test -p sugarscape-core --test auctions_discovery
```

Expected before implementation: FAIL because config parsing rejects `model: auctions`. This test uses only existing generic APIs; its assertion follows the economic clock to exactly 23 periods.

- [ ] **Step 3: Install the verified implementation.**

```bash
git apply "$task_assets/01-core-implementation.patch"
```

This patch contains `config`, `mechanism`, `learner`, `analysis`, `stats`, `view`, `world`, `presets`, `mod`, all model dispatch/checkpoint/schema/title/catalog wiring, 31 focused tests and thirteen new golden entries. It caches static equilibria at reset, integrates fringe payoffs by affine intervals, and preserves the exact per-period draw schedule. Its frozen-max fixture puts the old maximum first, so recomputing it inside an all-action loop fails the test.

- [ ] **Step 4: Verify the native engine and legacy behavior.**

```bash
cargo test -p sugarscape-core
cargo clippy -p sugarscape-core --all-targets -- -D warnings
cargo fmt --all --check
```

Expected: all tests pass; legacy ignored tests remain ignored. The new five 23-period fingerprints are baseline `8736a8664d834a2d`, SPA `1ba7cb1e9d3ff993`, all-feedback `949a7d400b127f20`, fringe `5f621ad6482502e7`, three-bidder `5c5c0013fee7f42b`. Existing golden values stay unchanged.

- [ ] **Step 5: Review and commit.** A fresh task reviewer checks the source mechanism, switch semantics and determinism against the spec. Stage 1 becomes Complete.

```bash
git add crates/sugarscape-core IMPLEMENTATION_PLAN.md
git commit -m "feat: add fixed-horizon Q-learning auctions"
```

### Task 2: CLI, WASM, playground and finite sweeps

**Files:**
- Modify: `crates/sugarscape-cli/src/main.rs`
- Modify: `crates/sugarscape-cli/tests/cli.rs`
- Modify: `crates/sugarscape-core/src/sweep.rs`
- Modify: `crates/sugarscape-wasm/tests/web.rs`
- Create: `sweeps/auctions-biased.json`
- Create: `sweeps/auctions-bidders.json`
- Create: `sweeps/auctions-downward.json`
- Create: `sweeps/auctions-duration.json`
- Create: `sweeps/auctions-feedback.json`
- Create: `sweeps/auctions-formats.json`
- Create: `sweeps/auctions-hindsight.json`
- Create: `sweeps/auctions-initialization.json`
- Create: `sweeps/auctions-local.json`
- Create: `sweeps/auctions-market.json`
- Create: `sweeps/auctions-persistent.json`
- Create: `sweeps/auctions-ties.json`
- Create: `web/src/auctions-grid.test.ts`
- Create: `web/src/auctions-inspection.test.ts`
- Create: `web/src/auctions.test.ts`
- Create: `web/src/auctions.ts`
- Modify: `web/src/compare-presets.ts`
- Modify: `web/src/compare/compare-view.ts`
- Modify: `web/src/determinism.test.ts`
- Modify: `web/src/engine.test.ts`
- Modify: `web/src/engine.ts`
- Modify: `web/src/experiments/form-view.ts`
- Modify: `web/src/experiments/form.ts`
- Modify: `web/src/main.ts`
- Modify: `web/src/models.ts`
- Modify: `web/src/types.ts`
- Modify: `web/src/ui/charts-panel.ts`
- Modify: `web/src/ui/grid-view.ts`
- Modify: `web/src/ui/inspect-panel.ts`
- Modify: `web/src/ui/series-data.ts`

**Interfaces consumed:** Task 1's generic model/schema/preset/series/Inspect interfaces and `Outcome`. Inspect fields include `period`, `horizon`, `grid`, both histograms and denominators, `learners`, `greedy`, `played`, `shares`, optional `fringe_bid`/`fringe_share`, `stable`, `equilibria`, optional outcome.

**Interfaces produced:** Existing `Sim` and CLI routes; `auctions` model metadata, config/stats/inspection types; `auctionRows`, `auctionLegend`, `auctionChartCaption`, `auctionChartLines`; two Compare entries; twelve `auctions-*` built-ins.

- [ ] **Step 1: Install host/view tests and confirm the CLI boundary fails.** Stage 2 becomes In Progress.

```bash
git apply "$task_assets/02-hosts-web-sweeps-tests.patch"
cargo test -p sugarscape-cli an_auction_session_finishes_its_partial_final_batch -- --nocapture
```

Expected: FAIL, actual stop reason says `its end year`, expected `its last auction`. Web test bodies additionally pin registration, conditional chart lines, visible numeric axes/counts, fringe labels and selected-window denominators; WASM fixtures pin the same five native outcomes. Their missing new helpers are supplied by the next patch.

- [ ] **Step 2: Install the full host/view/sweep code.**

```bash
git apply "$task_assets/02-hosts-web-sweeps-implementation.patch"
```

The built-ins use finite multi-field axis values, with no new treatment language: formats, feedback, initialization, ties, hindsight, local, biased, downward, market, bidders, persistent and duration. Persistent's metric is whole-run realized revenue; terminal-policy and played-occupancy denominators stay distinct. Two-bidder chart requests omit third-bidder lines. The visible legend shows bid direction/range/count or Q/chosen/update rows with gold greedy and cyan played edges.

- [ ] **Step 3: Verify hosts and portable fingerprints.**

```bash
cargo test -p sugarscape-cli
wasm-pack test --node crates/sugarscape-wasm
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Expected: pass, including exact 23-period/7-period batching, immutable completion, all five native/WASM fingerprints and every built-in's validated seed/tick caps. New tests run short sessions; none runs a registered paper ensemble.

- [ ] **Step 4: Build and test the real playground.**

```bash
cd web
npm ci
npm run build
npm test
npm run typecheck
cd ..
```

Expected: pass with fresh WASM. Real Engine/GridView tests reach a fixed horizon with a partial batch and show bid axes/counts without selecting a cell. Check the default and persistent presets in the browser: horizon/progress uses economic periods, Values/whole/late views render, third-bidder projection is labelled, comparisons change both feedback and updating where appropriate. Transient stability never displays completion.

- [ ] **Step 5: Review and commit.** Stage 2 becomes Complete.

```bash
git add crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli crates/sugarscape-wasm sweeps web IMPLEMENTATION_PLAN.md
git commit -m "feat: expose auction feedback experiments across hosts"
```

### Task 3: Fixed-count survey and source-aware documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/studies/2026-09-27-swarm-coordination.md`
- Modify: `survey/.gitignore`
- Create: `survey/src/claims/auctions-source-figures.json`
- Create: `survey/src/claims/auctions.rs`
- Modify: `survey/src/claims/mod.rs`
- Modify: `survey/src/main.rs`

**Interfaces consumed:** `AuctionsConfig`, `AuctionsWorld::new/run/is_finished/outcome`, complete serialized `Outcome`, source figure JSON, existing `Claim/Outcome/Verdict`, threaded native runner. The survey crate remains outside the workspace.

**Interfaces produced:** `claims::auctions::claims() -> Vec<Claim>` registered by `claims::all()`, sixteen `auctions.bs.*` checks and eight `auctions.controls.*` records; complete raw sessions in ignored `survey/out/auctions-sessions.jsonl`; the existing survey result JSON route. Sessions cache by full config and seed within each invocation.

- [ ] **Step 1: Install the survey registry boundary test.** Stage 3 becomes In Progress.

```bash
git apply "$task_assets/03-survey-docs-tests.patch"
cargo test --manifest-path survey/Cargo.toml the_registered_auction_baseline_can_be_selected_without_running_it
```

Expected: FAIL because no auction baseline claim is registered. This test does not execute learners.

- [ ] **Step 2: Install the survey and descriptive documentation.**

```bash
git apply "$task_assets/03-survey-docs-implementation.patch"
```

Read the complete `check`, `control`, `judge`, histogram and uncertainty functions in the patch. The generic `--seeds` flag cannot shrink fixed-count claims. Selected populations normalize by converged n, pair bins sort bidder labels, all-out/off-diagonal bins remain present, missing coverage yields Inconclusive, and controls retain every declared arm. Binary intervals use Wilson bounds; paired influence values preserve separate convergence subsets instead of selecting their intersection. Mean statistics use sessions, never periods, as independent observations. Source readings include explicit underdetermination caveats.

README, roadmap and the swarm study describe the model, source discrepancies and safety framing without claiming measured reproduction. Queue #1 remains unchanged until Task 4 integration.

- [ ] **Step 3: Verify synthetic judges and protocol controls.**

```bash
cargo test --manifest-path survey/Cargo.toml
cargo fmt --manifest-path survey/Cargo.toml -- --check
git diff --check
```

Expected: pass, including coverage boundaries, unequal convergence subsets, unordered off-diagonal TV, single/empty/all-success Wilson cases, whole-played-pair occupancy and ten short unused-feedback seeds. Standalone survey strict clippy has seven pre-existing warnings in other modules; record that baseline rather than making unrelated changes. Workspace clippy remains required and clean.

- [ ] **Step 4: Run documentary controls without an ensemble.**

```bash
cd survey
cargo run --release -- --only auctions.controls.source-consistency
cargo run --release -- --only auctions.controls.stage-game
cd ..
```

Expected: Holds for documentary arithmetic and the finite-game checks. These source/benchmark tests do not imply that learner ensembles reproduce the paper.

- [ ] **Step 5: Review and commit.** Stage 3 becomes Complete.

```bash
git add survey README.md docs/roadmap.md docs/studies/2026-09-27-swarm-coordination.md IMPLEMENTATION_PLAN.md
git commit -m "feat: preregister auction reproduction checks and controls"
```

### Task 4: Native findings, fresh review and integration

**Files:** survey result JSON and a findings note under `docs/superpowers/specs/`; updates to README, roadmap, paper index, swarm study and truthful preset descriptions/titles when warranted. Raw sessions/PDFs remain ignored. These findings are intentionally not prefilled from a short smoke run.

- [ ] **Step 1: Confirm replay integrity before native measurements.** Stage 4 becomes In Progress.

```bash
python3 "$task_assets/replay.py" --target . --verify-only
```

Expected: `Verified 57 product files byte for byte.` This checks the reviewed code before new findings/document edits. If main has advanced, integrate its unrelated changes explicitly, then verify auction-specific protocols/goldens; do not replace a conflicting existing file wholesale with a scratch version.

- [ ] **Step 2: Run the registered native studies outside CI.**

```bash
cd survey
cargo run --release -- --only auctions.bs.
cargo run --release -- --only auctions.controls.
cd ..
```

This deliberately executes every fixed-N source/reading check and control, including the one 100m run per format, twenty-seed duration studies and ambiguity tables. Keep the source and control result JSON files and invocation/runtime metadata. The single-session planning smoke completed exactly 1m periods in 1,000 ticks in .752s on this machine; it is a workload check, not an ensemble verdict or a promised total runtime. Full studies can take roughly an hour and use the native threaded runner. Prefixes permit separately invoking a check; cache reuse is within one invocation, and raw JSONL is append-only across invocations.

- [ ] **Step 3: Produce a reviewable findings table from the actual registry output.**

```bash
python3 - <<'PY_FINDINGS'
import json
from pathlib import Path
paths = [Path('survey/out/results-auctions.bs..json'), Path('survey/out/results-auctions.controls..json')]
rows = [r for path in paths for r in json.loads(path.read_text())]
assert len(rows) == 24
assert len({r['id'] for r in rows}) == 24
lines = ['# Q-learning Auctions — registered findings', '',
         'The committed spec defines all decision rules. Source readings and controls are separate; discarded sessions remain in unconditional summaries.', '',
         '| Check | Verdict | Measured | Detail |', '|---|---|---|---|']
def cell(value):
    return str(value).replace('|', '\\|').replace('\n', ' ')
for row in rows:
    lines.append('| ' + ' | '.join(cell(row[key]) for key in ['id', 'verdict', 'measured', 'detail']) + ' |')
Path('docs/superpowers/specs/2026-10-02-q-learning-auctions-findings.md').write_text('\n'.join(lines) + '\n')
PY_FINDINGS
```

The filenames contain two adjacent dots because the selected prefixes end in a dot and the existing runner appends `.json`. Record requested/converged/discarded counts, conditional/unconditional means, histogram TV/largest gaps, the separately scored prose/figure targets and uncertainty. Explain failures and source underdetermination candidly. Keep primary reconstruction and all ambiguity arms; do not choose a better-fitting default. New scientific rules require a dated amendment and a separate verdict. The study's safety interpretation concerns interaction/feedback outcomes and cannot establish deception, communication or punishment.

- [ ] **Step 4: Request one fresh whole-branch review of implementation and measurements.** Give the reviewer the committed spec, code diff, raw-session/export provenance, result JSON and findings note. Resolve substantive issues using the critics' actual protocols and source statistic definitions. Re-run only affected studies when fixes require it; preserve earlier registered verdicts and explain any amended run.

- [ ] **Step 5: Verify the final branch, integrate and publish.**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo test --manifest-path survey/Cargo.toml
cd web
npm run build
npm test
npm run typecheck
cd ..
```

Update the roadmap heading from prepared to the assigned milestone, append the Reproduced paper row using measured outcomes, then remove Queue #1. Finalize truthful README/study/preset copy from findings, retaining unknown conventions and source discrepancies. Follow `superpowers:finishing-a-development-branch` with the user's authorized merge/push/CI/Pages workflow; preserve unrelated main changes and inspect actual CI/deployment status. Smoke the deployed auction page and Inspect/Compare/Experiments. Stage 4 becomes Complete and remove `IMPLEMENTATION_PLAN.md` only when all required work is done.

## Planning verification record

See adjacent `verification.json` for commands, counts and manifest hashes. Both the staged red/green replay and automated replay reconstructed all 57 files byte for byte. Final checks passed: 1,710 native workspace tests, 79 WASM tests, 881 web tests, 100 survey tests, TypeScript, production build, formatting and workspace clippy. A fresh independent code reviewer confirmed all findings resolved. Full registered ensembles remain for Task 4.

The plan's self-review checked every spec section against the tasks/patches, verified exact API names, scanned for placeholders and pinned the five Review Focus cases. Implementation execution remains subagent-driven as requested.
