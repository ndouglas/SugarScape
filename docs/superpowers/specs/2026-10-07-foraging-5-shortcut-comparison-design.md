# F5: bounded shortcut and food-return comparison

**Date:** 2026-10-07.
**Status:** mechanism-first scope and collection/analysis architecture approved conversationally; this written specification and its candidate workload await user review. No implementation plan, executable scientific registration or scientific run is approved yet.
**Programme:** [F1–F5 construction sequence](2026-10-04-foraging-construction-design.md).
**Baseline:** [F4 construction](2026-10-06-foraging-4-construction-design.md), locally integrated at `568a3fa`, recorded at `a2ebf4f`; fresh merged-tree workspace 2,852 passed/0 failed/103 ignored, formatter and core Clippy clean. This design worktree includes subsequent main `0d2bc3b` shared-surface work.
**Sources:** [preliminary shortcut audit](../../studies/2026-10-07-foraging-shortcut-reading.md), including retrieved author material and unresolved workbook/figure access.

## Purpose, claims and source boundary

The user selected a mechanism-oriented comparison: does paid excavation improve finite-horizon food return under the existing F4 rules across straight, detour and twisting initial routes? The primary comparison is excavation eligibility with the same initial terrain, not a claim that any condition must win. An initially shortened route is a separate reference. Structural route shortening, learning about a route and delivering food remain distinct.

This is a supplied computational study motivated by the termite shortcut phenomenon. All grid coordinates, populations, food units, transition probabilities, horizons and seed budgets below are supplied choices. They are not centimeter/hour conversions, fitted termite behavior, source-paper quantitative reproduction, energy estimates, architectural intelligence or altruism measures. The source experiment's variable width, backfill, continuous feeding and simultaneous colony allocation among arenas are not added to F4.

The workbook/README and source figure have not been inspected, and the author's script is not a standalone clean-session reproducer. Those gaps remain required follow-up for source-data or quantitative biological comparison. They do not supply hidden calibration values or stop engineering this explicitly supplied mechanism harness. This spec neither claims complete biological reconciliation nor waives its gate for biological reproduction.

## Architecture and boundaries

Implement a focused `survey` study using the existing native experiment conventions. F4 production code, public APIs, worker policy, RNG stream, scheduler, information boundaries and physical rules remain unchanged. No generic world abstraction, model registration, browser/WASM integration, role allocation or relay/drop/pile transport is added.

Four responsibilities:

1. A scenario builder resolves literal geometry and regimes into normalized public F4 `Setup` values, checking route/access invariants independently of worker policy.
2. A versioned manifest pins all conditions, full parameters/options, seed lists, roles and budgets. No mutable engine default resolves a research input.
3. A serial collector invokes public F4 `run`, stores complete bounded Episodes in immutable raw envelopes and records code/protocol/manifest identities and saved-byte hashes.
4. A saved-only analyzer validates the full archive and computes deterministic tables/paired summaries using `survey::stats::paired_summary`. It never runs stochastic episodes or restores private World state.

F4 values currently expose serialization without a restoration interface. The survey decoder owns versioned wire types/validation; it does not widen core fields or add core deserialization. Its expected setup is rebuilt from the frozen scenario definition. Global geometry and cached access are researcher outputs, never worker inputs.

## Common geometry and resources

Coordinates use F4's integer x/y convention; movement remains cardinal and boundaries do not wrap. Grid is 25 × 25. All geometry lists are sets before F4 normalization; inclusive axis-aligned segments include both endpoint cells.

- Nest: all nine cells `x=2..4, y=20..22`.
- Waste outlet: `(1,21)`, directly adjacent to nest `(2,21)`.
- Food patch: all sixteen cells `x=2..5, y=2..5`, initially open. Food IDs 0..15 are assigned by increasing y then x. Tokens remain finite/nonregenerating.
- Spawn list, preserving worker identity: `(2,21),(2,21),(3,21),(3,21),(4,21),(4,21),(3,20),(3,20)`. Eight workers, never more than two initially per cell.
- Common initial open cells are nest, outlet and food patch, union the chosen route.
- Excavatable substrate, where enabled: every interior cell `x=1..23, y=1..23`. The outer border is protected. Mask entries may include initially open cells, matching F4; capacity is initially solid masked cells only.

This keeps nest/outlet/food coordinates, resource identities, population and Euclidean separation common. Open area and route shape intentionally vary; do not interpret between-geometry effects as pure distance effects with every other geometric feature held constant.

| Geometry | Inclusive route vertices in order | Initial shortest nest-to-patch distance | Initial open cells |
| --- | --- | ---: | ---: |
| Straight | `(3,20) -> (3,5)` | 15 | 40 |
| Detour | `(4,20) -> (21,20) -> (21,5) -> (5,5)` | 48 | 73 |
| Twisting | `(4,20) -> (21,20) -> (21,17) -> (18,17) -> (18,14) -> (21,14) -> (21,11) -> (18,11) -> (18,8) -> (21,8) -> (21,5) -> (5,5)` | 60 | 85 |

Multi-source graph calculations verified these distances, common initial food connectivity and outlet reachability without worker simulation. The temporary hand-count of twisting as 66 was corrected to 60 after segment summation and independent BFS; no geometry was tuned to match the guess. Evidence: `/tmp/sugarscape-f5-source-audit-20261007/geometry-preflight.json`.

## Regimes and panels

Primary route panel has three regimes in each geometry:

- **Paid:** baseline open cells and the full interior diggable mask. Every excavation creates spoil and pays F4's direct transport/disposal costs.
- **Protected:** identical baseline open cells; empty diggable mask. All remaining solid cells are protected.
- **Already open:** baseline open cells union the straight corridor `(3,20) -> (3,5)`; empty diggable mask. Initial shortest patch distance is 15 in all geometries. No construction is available.

Paid-minus-Protected is the primary intervention within a geometry. Eligibility is locally observable and legitimately changes with the mask; common observation rules do not imply identical realized observations. The already-open reference changes initial route availability; comparing it with Paid does not isolate construction cost alone or establish a delivery upper bound.

A separate sealed-access panel retains exposed-but-inaccessible controls. Remove gate `(3,6)` from the straight baseline or `(6,5)` from detour/twisting baseline. The food patch remains open/exposed but disconnected from the nest; the outlet remains connected. Compare Paid and Protected for each geometry, with the same masks defined above. A successful opening can connect available food without a first-exposure event. These six conditions are secondary access/censoring controls, not pooled with the nine primary conditions. No already-open duplicate is needed in this panel.

Canonical ordering: route panel first, geometry Straight/Detour/Twisting, regime Paid/Protected/AlreadyOpen; then sealed-access panel, same geometry order, regime Paid/Protected. IDs are versioned strings such as `route.straight.paid`, `route.detour.already_open`, `access.twisting.protected`. There are exactly fifteen conditions.

## Fixed candidate workload and controller

All conditions explicitly supply:

- `p_search=0.05`, `p_return=0.01`.
- `lambda_fidelity=1.0`, `lambda_publish=1.0`, `lambda_waypoint=0.01`.
- `ticks=512`, `sample_every=128`, `snapshots=true`.

These are an explicit first fixed controller setting, not optimized/source-calibrated values. F4 retains the five domains, F1 arrival functions and their actual per-opportunity cadence. Each episode has 4,096 paid worker opportunities and five frames at completed ticks 0/128/256/384/512. Every episode runs the full horizon even after all delivery or exhaustion; final work may therefore include construction after food depletion.

Scientific candidate seeds are the forty u64 values 10001..10040 for each condition. This is a descriptive Monte Carlo budget, not a power calculation or biological sample size. Whole episodes are replicate records; workers, food tokens, ticks and checkpoint frames are not independent samples. Seed labels pair contrasts, without maintaining identical later RNG streams when decisions diverge.

| Panel | Conditions | Candidate scientific episodes | Opportunities |
| --- | ---: | ---: | ---: |
| Primary route | 9 | 360 | 1,474,560 |
| Sealed access | 6 | 240 | 983,040 |
| Total | 15 | 600 | 2,457,600 |

Construction/engineering checks use seeds 7 and 8 separately, thirty complete candidate episodes at most for the geometry matrix; smaller hand-worked scenes can test validators. Existing F4 engineering outcomes/seeds are disclosed in history; no candidate F5 scientific episode has been inspected or run. No post-outcome seed extension, horizon/parameter tuning, favorable-stratum selection or pooled overall verdict is authorized by this design. Any amendment records prior inspected information and the reason before new collection.

## Outcomes, clocks and contrasts

Primary episode outcome is food delivered by the fixed cutoff, integer 0..16. Report Paid-minus-Protected separately for each primary geometry: three signed primary estimates. Include all forty paired seeds, condition means/seed rows, mean paired differences, existing descriptive Student-t 95% intervals and positive/zero/negative counts. These describe computational seed variation, not animal-population uncertainty, and carry no significance-selected or Holds/Fails conclusion.

AlreadyOpen-minus-Protected and Paid-minus-AlreadyOpen are secondary references, separately labelled. The sealed panel reports secondary Paid-minus-Protected contrasts for delivered food and terminal accessible-food count, plus every run's first-access/delivery censoring. Do not pool primary/secondary panels or geometries into one preferred efficacy score.

Preserve complete F4 final summaries and checkpoint observations. Report food/spoil/terrain inventory, all disjoint work categories, per-worker work, advice/publication diagnostics and worker/researcher computation. Moves/digs/pickups/deposits/disposals/waits are physical opportunities; computational calls/visits and optional native elapsed time are different costs. No invented energy weighting combines them into net benefit.

Route diagnostic at each saved frame is minimum current nest distance over all sixteen original food positions, using access records even after their tokens are delivered. Also retain per-resource distances and initial flags. Physical distance ignores temporary occupancy and private knowledge. Initial all-accessible routes have no first-access events. The engine's aggregate first-access milestone identifies the lowest newly accessible food ID, so its distance is not substituted for the minimum-to-patch diagnostic.

Report initial/final distance, nonnegative structural gain and the five-point curve. If a first shorter saved distance is reported, mark it as a checkpoint observation with the preceding checkpoint interval; it is not an exact excavation time or responsible-worker event. Straight distance cannot fall below the common Manhattan lower bound 15. Sealed distance is null while disconnected, not zero; access creation is not a shortening from an invented infinite/zero baseline. Initial-to-final gain and first shortening relative to the initial state are null for sealed episodes even when terminal access exists; use their curves and access milestones instead.

F4 milestones use zero-based processing ticks, while frames count completed ticks. Preserve those clocks and one-based milestone opportunity indexes. Missing pickup/access/delivery/all-food milestones are censored. Delivered-only timing, if shown, is conditional with censor counts beside it; never replace missing events with zero or omit failed legal runs. Congestion and carried food/spoil at cutoff remain in all reports.

## Verification controls and scope of saved validation

The Protected straight setup and AlreadyOpen straight setup are identical after normalization. Their core Episodes must match byte-for-byte within each construction seed; envelope condition IDs differ. This is a programmed identity check, not a discovered null effect. Protected/AlreadyOpen cases have zero digs/spoil, and all initially connected foods stay structurally accessible. Protected sealed cases have no accessible food, pickups or delivery throughout; these zeros follow from the supplied geometry/rules. Paid straight cannot exhibit structural distance below 15, though its realized delivery/work need not match Protected.

Builder tests independently assert exact route lengths/open counts, shared coordinates/spawns/IDs, initial exposure/access flags, border/mask/capacity and gate disconnection. Test independent multi-source BFS against cached distances in every stored frame. Hand-worked checks establish material and action equations and timing/censoring; no positive delivery result is demanded of an arbitrary seed.

Archive validation rejects duplicate/missing/extra condition/seed keys; wrong code/protocol/manifest references, bytes or hashes; schema/field/number violations; noncanonical setup/options; short horizons; wrong frames; impossible capacity/cargo/ledger combinations; invalid IDs/origins/times; and inconsistent work/access/terrain inventories. Check monotone open sets, unique spoil origins, retained initial geometry, allowed food-state progression and cumulative counters between frames. All error messages identify condition/seed and affected record/field.

Snapshots omit intervening actions and private maps. The saved validator can verify observed state/accounting/connectivity constraints, not reconstruct every intermediate physical action or prove controller/RNG fidelity from coarse frames. Verified F4 collection and immutable code/build/manifest provenance support that boundary. Do not claim a complete trajectory replay, add full histories, or rerun stochastic episodes during analysis. An identical sampled physical projection is only a duplicate sampled projection unless an identity control independently proves more.

Engineering acceptance also covers normalized replay, core reduction controls, disabled/read-only observation purity where applicable, parser and exact byte-cap errors, interrupted/failed archives, missing/corrupt inputs and byte-identical saved-only reanalysis. Native supported-platform identity is the promise; cross-version/platform trajectory equivalence is not.

## Collection, bounds and deterministic artifacts

Follow `survey`'s Burrow/protection archive conventions, keeping this study's types/modules focused. Candidate interface: `survey --foraging-shortcuts` prints the manifest without creating worlds. `--run --construction --protocol-revision COMMIT --approval-context TEXT --out NEW_DIR` collects the fixed engineering matrix only. Scientific `--run` rejects an unregistered candidate before output creation. `--analyze INDEX --out NEW_DIR` accepts saved records only and no seed/config/run overrides. Flags record provenance, not permission.

Execution uses a clean tracked checkout, full committed code/protocol revisions, SHA-256 of the fully resolved manifest and collector executable identity. Store canonical condition order then ascending seeds serially, with fresh F4 worlds. Create raw records exclusively, never overwrite or silently resume. Preserve `index.incomplete.json`, progress receipts and failure evidence on interruption; create final `index.json` only after the whole expected matrix is saved and validated. A failed/partial archive is never a successful study or a smaller denominator.

Raw envelopes (`foraging-shortcut-episode-v1`) contain identity plus the complete F4 Episode. Index schema is `foraging-shortcut-archive-v1`; analysis schema is `foraging-shortcut-analysis-v1`. Require the exact manifest/config/options/seed and saved relative paths/hashes. Bound raw envelopes to 4 MiB each and cumulative raw-envelope JSON to 1 GiB; index/progress/operational overhead is excluded from that cumulative measure and stored separately. Preserve F4's independent 64 MiB snapshot-JSON definition. These are serialized-output caps, not RSS guarantees. Inputs parse under the per-record bound; analysis handles one raw episode at a time and retains bounded result rows, not worlds or all raw records in memory. Budget failure preserves an incomplete archive and produces no completion index. The raw-byte cap is a declared recording limit, not a reason to omit large or incomplete legal episodes. Construction mode uses the same fifteen definitions and two construction seeds (122,880 opportunities total), with scientific contrasts omitted from its report.

Write deterministic `analysis.json` and `results.md` into a new directory. Reanalysis of identical saved inputs must reproduce both byte-for-byte. Optional runtime/hardware/build/execution-order/file-I/O diagnostics belong in separately labelled operational records, outside comparative deterministic payloads. No new dependency, external service or scientific R execution is required for this native harness.

## Review gates and next step

The user approved scope and collection/analysis architecture; this concrete geometry/workload/measurement specification remains draft until reviewed. After written-spec approval, write a staged implementation plan and use the standing subagent-driven preference. Preserve F1–F4 production regression gates; include `cargo test --manifest-path survey/Cargo.toml`, survey formatting/Clippy, core workspace tests/format/Clippy and independent task/whole-branch reviews appropriate to the eventual plan.

Engineering fixtures, harness integration, executable scientific manifest registration and actual scientific collection retain separate approvals. This written design authorizes no campaign execution. Future controller robustness, biological geometry calibration, widening/backfill/feeding, workforce/rotation sensitivity, role allocation and relay transport require explicit later designs.

Spec self-review checked literal geometry/capacity, primary versus secondary roles, intervention confounds, null route baselines, clock/censoring definitions, serialization-only core boundaries, coarse-frame validation limits, prospective seed/parameter choices and existing survey archive/statistics conventions. Independent geometry checks passed after the disclosed hand-count correction. Source access gaps remain explicit; no worker simulation was used to select these inputs. Local documentation links and whitespace are checked before the documentation commit.
