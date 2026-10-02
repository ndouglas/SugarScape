# Emergent Polarity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Cederman's original EPM and provincial extensions as the `polarity` kind, then report the registered reproduction and sensitivity findings without tuning the reconstruction to the figures.

**Architecture:** One seeded Rust engine owns territory, actions, resources, alliances, combat and episodes. Existing generic adapters expose it through CLI/WASM/playground; a standalone native binary records fixed-count sessions and Python applies the registered judges offline. Six presets and an exploratory predator-share sweep expose the mechanisms without substituting those short runs for source studies.

**Tech Stack:** Existing Rust core/CLI, rand/PCG, serde, wasm-bindgen, TypeScript/Vite/Vitest and Python standard library. No new dependencies. The survey crate remains outside the workspace.

**Spec:** `docs/superpowers/specs/2026-10-02-emergent-polarity-design.md`. Read the corresponding reading notes, source-figure JSON and extraction notes. Base `614858ed5d7b0bc612a366054553542431beae1f` includes the approved spec and dated clarifications, fixed before any native study.

## Global Constraints

- Existing configs, model fingerprints and fixtures remain unchanged. Add new golden cases.
- Native and WASM execute the same seeded mechanics with portable arithmetic. No survey-only fast engine, platform-dependent iteration order or imported third-party model source.
- No original EPM executable was recovered; do not claim author-code docking. PDFs and third-party archives stay ignored.
- Every ambiguity has an enum or numeric parameter, a selected default, source citation and description identifying the choice as reconstruction. Export the resolved configuration.
- Source figure extraction, study manifest and analysis decisions are fixed before native measurement. Later amendments retain the original verdict and identify the amendment.
- Long studies stay outside CI. One economic period is one full source iteration; batching repeats complete periods. Finished worlds are immutable.
- Original settings: 10×10 bounded cardinal grid, exact predator placement, N(50,10) initial stocks, N(2,5) primitive harvest, strict superiority/victory 2 or 3, damage .05. Chapter 4 CD/DC damage is enabled; chapter 5 disables it.
- Default EPM uses literal TFT, own-current-defections initiation guard, equal allocation, stored paths, before-damage victory, snapshot updates, affected-cell locks, collapse-only capital fall and signed stocks.
- Two-level/overextension require chapter 5 and PRA. Two-level tax_discount must be 1; overextension uses .4×.7^distance, stochastic threshold 3/exponent 5 and continues through hegemony.
- Source categories are 1 / 2 / 3–10 / 11–90 / 91–100 sovereigns. Hegemony is excluded from power politics. Larger non-source grids have unavailable source categories above 100.
- Invalid sessions make a stratum unresolved for source compatibility. Report the fraction, reason and affected policy; do not rerun replacement seeds to conceal invalidity.
- Use the frozen 572-arm, 28,520-session manifest. Analysis uses 100,000 draws and seed 2026100202; multiplicity families, source uncertainty and original/precision samples remain separate. Compatibility is not equivalence.
- Störmer's study is an attributed text-prior adaptation with 110 configuration means across 10 repeats, not 1,100 independent configurations or execution of her listing. Unquantified claims remain descriptive/Untestable.
- Execution remains subagent-driven. Final independent review examines source fidelity, judges, negative stocks, collapse/accounting and all hosts before integration. Completion includes findings, queue/index update, war-study handoff, CI and Pages at the final integrated head.

## Review Focus

1. Extreme finite stocks, tiny denominators and aborted periods must retain invalid outcomes without counting unfinished periods; Task 1's overflow/clock tests and Task 3 invalid-record fixtures pin this.
2. Capturing a province or capital must conserve the appropriate corporate/provincial stock, release disconnected enclaves once and restore latent types; Task 1's territory fixtures pin these cases.
3. Late sequential actors and coalition obligations must not invent unresolved attacks or recursively enlist another coalition; Task 1's resolved-dyad, deferred-obligation and nonrecursive fixtures pin the timing.
4. Partial JSON profiles and the final batched tick must export resolved choices and actual periods identically in CLI/native/WASM; Task 1 discovery/default tests and Task 2 real Engine/parity tests pin these.
5. Duplicates, mixed manifests, missing precision arms and uncertain figure boundaries must remain visible and unresolved rather than be silently pooled; Task 3's reporting fixtures pin each boundary.

## Complete code artifacts and replay

Six adjacent patches contain the complete source/test bodies for Tasks 1–3. Read the patches when reviewing code; they are executable source artifacts, not pseudocode. `manifest.json` lists SHA-256 preimages/final bytes for all 48 product files and hashes every patch. `replay.py` checks the base ancestry, preimages, patch bytes and final bytes. The scratch worktree is `~/.config/superpowers/worktrees/SugarScape/polarity-scratch`; the clean replay worktree is `polarity-plan-replay`. Product implementation is held there until this written plan is reviewed. Another task's staged main checkout is untouched.

Set this from the repository root (use the absolute directory in `polarity-plan` when replaying from a base without artifacts):

```bash
task_assets=docs/superpowers/plans/2026-10-02-emergent-polarity
```

If unrelated main changes conflict, integrate explicitly and preserve them. Do not replace aggregate files wholesale merely to satisfy a hash. Reverify affected protocols and fingerprints after integration.

### Task 1: Source mechanics, model discovery and exports

**Files:**
- Modify: `crates/sugarscape-core/src/lib.rs`
- Modify: `crates/sugarscape-core/src/model.rs`
- Create: `crates/sugarscape-core/src/polarity/alliance.rs`
- Create: `crates/sugarscape-core/src/polarity/analysis.rs`
- Create: `crates/sugarscape-core/src/polarity/combat.rs`
- Create: `crates/sugarscape-core/src/polarity/config.rs`
- Create: `crates/sugarscape-core/src/polarity/decision.rs`
- Create: `crates/sugarscape-core/src/polarity/mod.rs`
- Create: `crates/sugarscape-core/src/polarity/presets.rs`
- Create: `crates/sugarscape-core/src/polarity/resources.rs`
- Create: `crates/sugarscape-core/src/polarity/stats.rs`
- Create: `crates/sugarscape-core/src/polarity/territory.rs`
- Create: `crates/sugarscape-core/src/polarity/tests.rs`
- Create: `crates/sugarscape-core/src/polarity/view.rs`
- Create: `crates/sugarscape-core/src/polarity/world.rs`
- Modify: `crates/sugarscape-core/src/presets.rs`
- Modify: `crates/sugarscape-core/src/titles.rs`
- Modify: `crates/sugarscape-core/tests/golden.rs`
- Create: `crates/sugarscape-core/tests/polarity_discovery.rs`

**Interfaces consumed:** Existing `Model`, `ModelConfig`, `FieldError`, `Param`, `Stats`, `Series`, seeded `SimRng`, portable normal/arithmetic and canvas helpers.

**Interfaces produced:**

```rust
PolarityWorld::new(config: PolarityConfig, seed: u64) -> Result<PolarityWorld, Vec<FieldError>>
PolarityWorld::run(&mut self, ticks: u32)
PolarityWorld::completed_periods(&self) -> u64
PolarityWorld::outcome(&self) -> Option<&Outcome>
PolarityWorld::economic_fingerprint(&self) -> u64
config::schema() -> Vec<Param>
presets::presets() -> Vec<ModelPreset>
stats::series_names() -> Vec<String>
analysis::category(n: u32) -> Option<Category>
```

`PolarityWorld` implements existing `Model`; generic config/world dispatch handles the new kind. `Outcome` serializes resolved config, seed, completed/attempted periods, validity, endpoint sovereign/category/predator statistics, ledgers and censored episodes. Cell IDs remain fixed; fronts/trust/coalitions use sorted keyed collections. The implementation patch supplies every public enum/config field and adapter method, including Inspect and CSV.

- [ ] **Step 1: Create the four-stage ledger.**

```markdown
# Emergent polarity

## Stage 1: Core mechanics
**Goal**: Faithful named reconstruction and generic model adapter.
**Success Criteria**: Mechanism, accounting, invalidity, clocks and deterministic exports pass.
**Tests**: Core motifs, discovery, existing and new goldens.
**Status**: In Progress

## Stage 2: Hosts and playground
**Goal**: CLI/WASM/web controls, views, comparisons and exploratory sweep.
**Success Criteria**: Resolved configs, complete periods and readable mechanisms agree across hosts.
**Tests**: CLI, real WASM Engine, parity, web build and browser smoke.
**Status**: Not Started

## Stage 3: Frozen survey protocol
**Goal**: Native fixed-count recording and source-aware offline judges.
**Success Criteria**: Manifest validates without periods; synthetic fixtures preserve every evidence boundary.
**Tests**: Native runner fixtures, Python tests, manifest reproducibility.
**Status**: Not Started

## Stage 4: Findings and integration
**Goal**: Execute registered studies, report results, review, integrate and publish.
**Success Criteria**: Complete findings matrix and provenance, fresh review, CI and deployed Pages pass.
**Tests**: Native ensembles outside CI, final checks and deployment smoke.
**Status**: Not Started
```

- [ ] **Step 2: Install the generic failing discovery test.**

```bash
git apply "$task_assets/01-core-tests.patch"
cargo test -p sugarscape-core --test polarity_discovery
```

Expected: FAIL because `model:polarity` is unknown. The test uses only existing APIs and expects exactly 3 completed periods from a 2-period tick, 100 peaceful sovereigns and immutable completion.

- [ ] **Step 3: Install the verified engine.**

```bash
git apply "$task_assets/01-core-implementation.patch"
```

This installs separate config, territory, decisions, alliance, resources, combat, world, analysis, stats, view, presets and test modules. It includes strict thresholds, TFT reinitiation, two-stage paths, PRA conditional commitments, signed damage/harvest, provincial taxes/revolt, simultaneous losses, shuffled claims and provincial continuation. Completed-period metadata distinguishes an aborted attempt without concealing its ledgers.

Freeze this draw schedule before measurement: initial resources are sampled in row-major order before placement; exact placement uses descending Fisher-Yates with explicit u32 bounded draws, Bernoulli uses one f64 per cell. Normal draws use existing uncached Marsaglia rejection sampling even at SD 0; uniform controls use one f64. Sorted actors/fronts determine conditional target ties, two-stage border paths, initiation and stochastic combat draws. Random tie/path controls draw only when applicable. Single-draw combat uses one f64; independent draws use two. Harvest samples once per primitive in cell order. Snapshot structural claims receive one shuffle; sequential actors receive one shuffle and each unordered dyad is resolved once. Prime-threat ties are sorted or explicitly randomized. Rendering/Inspect clone or consume state without consuming the world's random stream. Treatments with different conditional paths are not claimed to share random tapes.

- [ ] **Step 4: Verify behavior and old models.**

```bash
cargo test -p sugarscape-core
cargo clippy -p sugarscape-core --all-targets -- -D warnings
cargo fmt --all --check
```

Expected: pass. Inspect the concrete tests for finite aggregate/ratio/ledger overflow and invalid completed periods; `primitive_provincial_capture_preserves_stock_at_new_province`, `corporate_provincial_capital_capture_transfers_only_center_stock`, capital/enclave conservation; `later_actor_cannot_initiate_against_already_resolved_dyads`, nonrecursive/deferred obligations; grouped-period equality, schema/default round trips and read-only rendering. All existing golden entries are unchanged; six new 200-tick values are frozen in the patch and verified again through real WASM in Task 2.

Some scratch motifs were characterization tests added after implementation; their passing first run is not represented as test-first evidence. The replay's discovery, CLI and native-boundary tests provide actual behavioral red/green cycles. Retain that limitation in the verification receipt.

- [ ] **Step 5: Review and commit.** A task reviewer checks the spec, source-reading switches, clocks/accounting and Review Focus 1–3. Mark Stage 1 Complete.

```bash
git add crates/sugarscape-core IMPLEMENTATION_PLAN.md
git commit -m "feat: reconstruct emergent polarity and provincial extensions"
```

### Task 2: CLI, WASM playground and experiments

**Files:**
- Modify: `crates/sugarscape-cli/src/main.rs`
- Modify: `crates/sugarscape-cli/tests/cli.rs`
- Modify: `crates/sugarscape-core/src/sweep.rs`
- Create: `sweeps/polarity-predators.json`
- Modify: `web/src/compare-presets.ts`
- Modify: `web/src/engine.ts`
- Modify: `web/src/experiments/form.ts`
- Modify: `web/src/models.ts`
- Create: `web/src/polarity-engine.test.ts`
- Create: `web/src/polarity-inspection.test.ts`
- Create: `web/src/polarity.test.ts`
- Create: `web/src/polarity.ts`
- Modify: `web/src/types.ts`
- Modify: `web/src/ui/charts-panel.ts`
- Modify: `web/src/ui/grid-view.ts`
- Modify: `web/src/ui/inspect-panel.ts`
- Modify: `web/src/ui/series-data.ts`
- Modify: `web/src/valley.ts`

**Interfaces consumed:** Task 1 generic schema/preset/config/model/series/Inspect routes and resolved `Outcome`. Inspection preserves provincial stock separately from corporate resources and ownership separately from coalitions.

**Interfaces produced:** Kind metadata/config/stats types; six readable source presets; toolbar actual periods/governments; grid capitals/latent strategy/coalition modes; front/trust/path Inspect text; source legend/Compare; one finite predator-share built-in. Existing WASM generic dispatch uses the same engine, without another simulation path.

- [ ] **Step 1: Install host/view tests.** Mark Stage 2 In Progress.

```bash
git apply "$task_assets/02-hosts-web-sweeps-tests.patch"
cargo test -p sugarscape-cli polarity_cli_reports_economic_periods_instead_of_only_batched_ticks
```

Expected: FAIL because the old endpoint reports ticks without `15 completed periods`. The fixture runs only a 2×2 peaceful grid, horizon 15, 7 periods/tick. Web tests additionally exercise resolved partial JSON after create/reset, invalidity visibility, readable fronts, stock distinctions and real WASM immutable final ticks.

- [ ] **Step 2: Install host/view/sweep code.**

```bash
git apply "$task_assets/02-hosts-web-sweeps-implementation.patch"
```

The built-in couples superiority/victory and uses 3 seeds; it is exploratory rather than the source 20/200 seed studies. Compare keeps per-world clock labels. Engine normalizes polarity's exported origin after creation/reset so resolved defaults travel with session exports. Registry titles and source legends describe selected reconstructions, without claiming measured fidelity.

- [ ] **Step 3: Verify native hosts and produce fresh release WASM.**

```bash
cargo test -p sugarscape-cli
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cd web
npm ci
npm run build
npm test
npm run typecheck
cd ..
```

Expected: pass, including six native/WASM 200-tick fingerprints and existing fixtures. Use release WASM for the full determinism suite, whose older long tests can time out under a debug build. Do not extend test timeouts to hide that build mistake. Review Focus 4 is covered by partial config Engine/reset tests, partial tick real WASM tests and CLI period reporting.

- [ ] **Step 4: Smoke the real browser.** Start existing web development tooling, then inspect original, alliance/PRA and provincial presets. Verify readable titles, all reset choices, 6px capital/latent markers, government/period toolbar, stock/front/trust Inspect, comparison source labels and the Experiments “From current world” configuration. Do not launch a full source study in the browser. Confirm console errors absent.

- [ ] **Step 5: Review and commit.** Mark Stage 2 Complete.

```bash
git add crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli web sweeps IMPLEMENTATION_PLAN.md
git commit -m "feat: expose polarity mechanics and experiments across hosts"
```

### Task 3: Registered native sessions and offline judges

**Files:**
- Modify: `survey/Cargo.toml`
- Create: `survey/polarity/README.md`
- Create: `survey/polarity/analysis.py`
- Create: `survey/polarity/manifest.py`
- Create: `survey/polarity/run.py`
- Create: `survey/polarity/studies.json`
- Create: `survey/polarity/test_analysis.py`
- Create: `survey/polarity/test_manifest.py`
- Create: `survey/polarity/test_reporting.py`
- Create: `survey/polarity/test_run.py`
- Create: `survey/src/bin/polarity.rs`

**Interfaces consumed:** `PolarityConfig`, generic `ModelConfig` validation, `PolarityWorld::new/run`, existing `Model::finished/latest_json`, complete `Outcome`, committed source JSON and design.

**Interfaces produced:** `polarity` native executable with `--manifest`, `--manifest-sha256`, `--out`, optional `--arm PREFIX`/`--validate`; Python `run_native(binary,manifest,out,arm=None,validate=False)`; `build_manifest(source)`; `report(manifest,source,records,manifest_bytes)`; analysis CLI emitting JSON/Markdown. Generic `survey` remains the Cargo default binary; polarity uses its documented separate fixed-count pipeline.

- [ ] **Step 1: Install the native-boundary tests and behavioral stub.** Mark Stage 3 In Progress.

```bash
git apply "$task_assets/03-survey-tests.patch"
python3 -m unittest discover -s survey/polarity -p test_run.py
```

Expected: two behavioral FAILs, because the stub returns 1 instead of invoking either fixture executable. Space-containing paths and the exact manifest-byte SHA are asserted; the failure-return fixture expects native exit 2. This runs no model periods.

- [ ] **Step 2: Install the complete runner, manifest and judges.**

```bash
git apply "$task_assets/03-survey-implementation.patch"
```

The wrapper binds exact manifest bytes, invokes argv without shell interpolation and propagates failure. Rust resolves all arms and seed ranges before execution; resumes checked existing keys, flushes each completed record and retains invalid/panicking outcomes without replacement. Python keeps all raw records and separate denominators, verifies source/manifests, marks missing/duplicate/mixed/inconsistent data unresolved and applies the fixed bootstrap/multinomial/Holm rules. PRA source matching uses registered matching chapter 5 factorial precision. Unmatched inferred tax knots lack a 200 seed extension and remain Unresolved. Stocks-support controls are separate from the primary interaction family.

The README documents the standalone pipeline and ignored outputs. `default-run="survey"` preserves existing Cargo invocations after adding the second binary. No paper index/queue claim is advanced before measurements.

- [ ] **Step 3: Verify protocol fixtures and a zero-period manifest check.**

```bash
python3 -m unittest discover -s survey/polarity -p 'test_*.py'
cargo test --manifest-path survey/Cargo.toml
cargo clippy --manifest-path survey/Cargo.toml --bin polarity -- -D warnings
rustfmt --edition 2021 --check survey/src/bin/polarity.rs
cargo build --manifest-path survey/Cargo.toml --bin polarity
python3 survey/polarity/run.py --binary survey/target/debug/polarity --validate --manifest survey/polarity/studies.json --out survey/out/polarity-sessions.jsonl
git diff --check
```

Expected: 26 Python fixtures and the existing survey tests plus 4 native runner fixtures pass; validator reports 572 arms/28,520 sessions and executes zero periods. Pass the actual binary path if `CARGO_TARGET_DIR` differs. Review Focus 5 fixtures cover duplicates, invalid category denominators, wrong manifests/configs/seeds, mixed datasets, empty sessions, missing precision and all admissible source vectors. Störmer aggregates configuration means. Native fixture sessions are 3-period peace only.

- [ ] **Step 4: Verify manifest reproducibility without measurement.**

```bash
python3 survey/polarity/manifest.py --source docs/superpowers/specs/2026-10-02-emergent-polarity-source-figures.json --output /tmp/polarity-studies-recreated.json
cmp survey/polarity/studies.json /tmp/polarity-studies-recreated.json
```

Expected: byte identity. Fixed source contains 72 positions/312 admissible count vectors, including exact 5/9/5/0/1. Tax abscissae are explicitly inferred with uncertainty. Do not silently swap source images or upgrade uncertain positions to exact observations.

- [ ] **Step 5: Review and commit.** Mark Stage 3 Complete.

```bash
git add survey IMPLEMENTATION_PLAN.md
git commit -m "feat: preregister polarity studies and source-aware judges"
```

### Task 4: Native findings, independent review and publication

**Files:** Create `docs/superpowers/specs/2026-10-02-emergent-polarity-findings.md` and reviewable summary JSON; update `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/studies/2026-09-26-war-and-society.md` and the geopolitics campaign plan. Raw sessions/source PDFs remain ignored. The findings are not prefilled from scratch tests.

- [ ] **Step 1: Confirm reviewed product bytes before measurement.** Mark Stage 4 In Progress.

```bash
python3 "$task_assets/replay.py" --target . --verify-only
```

Expected: `Verified 48 product files byte for byte.` If integration changed shared aggregate files, inspect/resolve explicitly, then retain a revised verification receipt with affected tests; do not overwrite another milestone.

- [ ] **Step 2: Record runtime/provenance and execute the entire frozen workload outside CI.**

```bash
mkdir -p survey/out
git rev-parse HEAD > survey/out/polarity-code-head.txt
python3 --version > survey/out/polarity-python-version.txt
rustc --version > survey/out/polarity-rust-version.txt
cargo build --release --manifest-path survey/Cargo.toml --bin polarity
python3 survey/polarity/run.py --manifest survey/polarity/studies.json --out survey/out/polarity-sessions.jsonl
```

Do not pass generic reduced seed counts, choose replacement seeds or select a better-fitting default. A single process resumes this output; prefixes may partition work into separate files if needed. Retain all records and actual invocation/runtime metadata. The workload includes original 640, separate precision 6,400, chapter 5 allocation factorial 12,800, critics/adaptations, numerical/readings controls, tax and 20 overextension sessions. Run duration has not been inferred from a scratch ensemble; report actual progress rather than promising a completion time.

- [ ] **Step 3: Apply the registered judges and create the findings artifact.**

```bash
python3 survey/polarity/analysis.py --manifest survey/polarity/studies.json --source docs/superpowers/specs/2026-10-02-emergent-polarity-source-figures.json --sessions survey/out/polarity-sessions.jsonl --output survey/out/polarity-findings
cp survey/out/polarity-findings.md docs/superpowers/specs/2026-10-02-emergent-polarity-findings.md
python3 - <<'PY_SUMMARY'
import json
from pathlib import Path
p=Path('survey/out/polarity-findings.json')
report=json.loads(p.read_text())
assert len(report['arms']) == 572
assert len({r['id'] for r in report['rows']}) == len(report['rows'])
summary={k:v for k,v in report.items() if k!='raw_records'}
Path('docs/superpowers/specs/2026-10-02-emergent-polarity-findings.json').write_text(json.dumps(summary,indent=2,sort_keys=True,allow_nan=False)+'\n')
PY_SUMMARY
```

Inspect received/registered counts and integrity issues before drawing conclusions. Keep source compatibility and P2/P3 positive hypotheses separate from reproduced negative counterexamples; defense/alliance hegemony and 2–10 contrasts are different measures. Report signed creation, invalid stocks and resource policy sensitivity, source uncertainty, original/precision populations and all named alternative readings. Discuss attribution and limits: no original code docking, no unknown-seed overextension trajectory fit, no exact 75% inert gate, no AI deception/intent inference. Bootstrap compatibility conditions on an estimated reconstruction distribution and is not equivalence.

- [ ] **Step 4: Request one fresh whole-branch review.** Give the reviewer the committed spec/readings/source extraction, implementation diff, raw provenance, result JSON and findings. Check scientific definitions and critics' own protocols. Resolve substantive problems, rerunning only affected studies, retaining the original record and dated amendments when rules change.

- [ ] **Step 5: Verify, integrate and publish.**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --manifest-path survey/Cargo.toml
python3 -m unittest discover -s survey/polarity -p 'test_*.py'
cd web
npm run build
npm test
npm run typecheck
cd ..
```

Assign the milestone number against the actual integrated roadmap (36 was provisional). Add a paper index result with measured findings and then move the completed queue entry; update campaign progress and truthful preset/docs text as warranted. Hand off abstract resource costs, episodes, inclusive durations/end causes and censoring to the war study, explicitly distinguishing resources from casualties. Follow `superpowers:finishing-a-development-branch` within the authorized merge/push/CI/Pages workflow; inspect actual checks/deployment at the integrated head and smoke deployed polarity/Inspect/Compare/Experiments. Stage 4 becomes Complete and remove `IMPLEMENTATION_PLAN.md` only when required work is complete.

## Planning verification record

See adjacent `verification.json` for commands, exitcodes, evidence limits and byte-replay hashes. Scratch verification has not executed a native scientific ensemble. Short unitfixtures,200 tick fingerprints and browser checks establish implementation behavior; findings remain the purpose of execution, not a claim inferred from green tests.

Self-review checked all spec sections against the tasks/artifacts, exact public APIs, placeholder absence, draw schedule, five Review Focus boundaries and the fixed measurement rules. Execution method remains the user's requested subagent-driven workflow.
