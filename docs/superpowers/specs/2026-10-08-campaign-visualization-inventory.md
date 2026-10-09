# Recent campaign browser-visualization inventory

**Date:** 2026-10-08
**Status:** read-only inventory and proposed delivery batches; no runtime work approved.

The user asked to defer raw experiment discovery and expose recent completed
campaigns in the web interface so their behavior can be observed directly.
This inventory distinguishes checked WASM support from an actual browser viewer.

## Missing dedicated viewers

| Campaign family | Existing executable support | Browser gap | Proposed batch |
| --- | --- | --- | --- |
| Deduction capabilities/Wink | Core Engine, built-in controllers, CLI run/play/replay/diagnose | No browser adapter or game viewer | A |
| Noisy testimony and testimony decision game | Exact core models and CLI diagnostics | No observation/posterior/decision viewer | A |
| Strategic reporting | Exact evaluation, searches, frozen policies and diagnostics | No selected-policy or retained-search visualization | A |
| Strategy-aware listeners | Exact calibration and live policy inference | No joint policy/truth-belief visualization | A |
| Adversarial reporting audit | Exact attack basis, listener decisions, witnesses | No attack/listener comparison viewer | A |
| Shared surface | Checked per-episode evaluation and frozen diagnostic | No WASM adapter or surface trace viewer | A |
| Active surface | Exact compiled policies, chronological replay and frozen diagnostic | No WASM adapter or paid-choice trace viewer | A |
| Burrow excavation and resource access | Existing checked WASM replay functions and native parity tests | No dedicated map/action replay controls | B |
| Dedicated CPFA F2 fixed world, F3 passage, F4 construction | Standalone core world/snapshot/run interfaces | No dedicated WASM adapters or lab viewers | B |
| P3 re-caching and P4 supplied caching gestures | Existing checked WASM construction/episode exports; ordinary Minds rendering foundation | No dedicated registered-lab setup and episode viewer | C |

Generic Minds central-place foraging controls do not expose the standalone CPFA
labs. The existing sweep view does not ingest deduction or surface diagnostic
schemas. Existing parity tests do not mean those campaigns are discoverable or
visualized in the browser.

F1 is a stateless rule utility feeding F2-F4, not a separate animated world.
F5 is active concurrent work and is excluded from the completed-campaign backfill
until its producer delivers a stable approved interface. Registered scientific
execution and campaign publication remain distinct from engineering availability.

## Proposed delivery order

Batch A returns first to the deduction and surface campaigns from the current
conversation. Introduce a shared experiment-browser shell and separate game,
testimony, and surface renderers. Deliver it as an independently reviewed task
with its own spec and plan before starting another batch.

Batch B exposes spatial burrow and dedicated foraging episodes, reusing the
shared shell and existing checked replay/snapshot APIs. Batch C exposes the
P3/P4 lab construction and episode interfaces without presenting exploratory
viewer selections as new registered scientific results.

Each batch requires its own reviewed scope, bounded inputs, native/WASM parity,
actual browser interaction checks, and exact-commit delivery verification.
The user may select a different first batch. This order is a proposal, not a
commitment to implement all families in one change.

## Common requirements

- Run through the existing Rust engine; TypeScript renders and navigates records.
- Play/pause/step/reset, timeline navigation, Agent selection, and cost/outcome
  inspection where those concepts exist in the campaign.
- Keep Agent-local information distinct from explicitly labeled researcher truth.
- Use bounded episode records rather than loading full multi-GB diagnostics.
- Preserve frozen reports/settings and compare replay exports with their retained
  references. A replay example does not substitute for aggregate campaign results.
- Keep current ordinary simulations and sweeps available.
- Preserve concurrent staged main work and the existing crowd worktree/evidence.

## Inspection basis

Checked main and crowd Git status, web main/Experiments view, web types/schema,
WASM exports and parity tests, core public campaign interfaces, and product
guides. At inspection main contained another campaign's staged F5 files, which
this task neither edited nor staged. Existing crowd runtime remained untouched.
The unapproved raw-discovery proposal is deferred by the user's new direction.

## Delivery update — 2026-10-08

Batch A is delivered at main `74feeed12216262f2ffefca90487b8f665f5c715`, with exact-commit CI and Pages success. Its eight working entries are documented in the [viewer guide](../../experiment-viewer.md). The table above retains the original inspection state.

Batch B now has a [proposed spatial-viewer design](2026-10-08-spatial-campaign-web-visualizations-design.md) covering Burrow excavation/access and CPFA F2–F4. Its written spec and implementation plan still require review before runtime work. Batch C and raw surface discovery remain deferred. F5 native archive/comparison engineering is integrated, while its scientific candidate remains unapproved; it is outside Batch B.
