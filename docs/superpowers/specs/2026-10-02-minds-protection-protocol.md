# Minds P3: proposed measured protocol

**Status:** ready for protocol review with the implementation plan; no campaign has run.
**Design:** [approved protection spec](2026-10-02-minds-protection-design.md).
**Route/schema:** `survey --protection`, `minds-protection-measured-v1`.

This protocol fixes the first laboratory campaign. Implementation verification can expose an invalid fixture; correct and version the protocol before scientific execution. Do not tune it against outcome estimates. Approval of the implementation plan permits implementation and deterministic construction checks; scientific execution follows successful checks and review of the committed executable manifest.

## World and roles

All episodes have a 9×9 torus with opaque one-cell border walls: `(0,0,9,1)`, `(0,8,9,1)`, `(0,1,1,7)`, `(8,1,1,7)`. Coordinates are zero-based; no interior walls. The border prevents wraparound movement and sight. All interior sites initially have zero capacity/resource except the two finite four-unit harvest patches `(2,2)` and `(2,3)` in single-cache, cue-error and stumble fixtures. Growback is zero. Mixed-history fixtures have no harvest patches.

Owner is id 1, observer id 2, created at the specified positions before tick zero. Their initial sugar is 44 and 96, metabolism is 1 each, owner vision is 2 and observer vision is 6. One good only; carrying capacity is 128 for both roles. The large common cap leaves room for both endowments and is not a reserve. Reserve is 4 (`goap.horizon=4`, decision rule Book); hungry recovery uses `caching.dig_below=reserve`. Both walk at speed 1. Owner cheats/watches false; observer cheats/watches true except the visible-nonwatcher control. `watching.on=true`, span 64, `raid_when=always`, `raid_if=always`, `scrounge=harvest`, `value=room`; theft loot is Keep and owner memory On. Discovery probability is zero except the declared stumble contrast. No other minds, general site memory, larders, previous caching lab, reproduction, replacement, lifespan, seasons, pollution, truffles, combat, trade, credit, disease, inheritance or culture are enabled.

Normal surplus caching is suppressed in all lab episodes. Scheduled initial burial and completed relocation use the ordinary scatter burial transfer. Initial burial cost is zero; at tick 8 set the live cost to either 0 or 0.25, including Off and Erased controls. Do not replenish holdings or caches. A single original cohort totals 12 units; mixed-history cohorts total 6+6. Initial preparation consumes eight metabolism ticks, so both fixture families reach tick 8 with owner holdings 24 and original stock 12. This value is a construction assertion, not an outcome target.

Run ticks 0–63 inclusive (64 completed ticks). Continue after owner death to preserve observer opportunities; stop early only if both agents die, recording the actual completed horizon. Every living actor gets one action and metabolism per tick; shuffle their turn order with the ordinary world RNG. A scheduled hold is an action with zero gathering and ordinary metabolism. Scripted walking uses legal speed-one walking; it gathers nothing during preparation. A blocked script is recorded as fixture-invalid, never silently repaired or teleported. All exposed memories sweep by existing freshness semantics; owner exposure span is 64.

An episode has no policy parameter fitting. All policies are frozen: Off, Selective, Indiscriminate, Erased. Erased clears only the owner's exposure entries at tick 8, before either actor acts; it does not clear prepared-source metadata, observer memory, caches or holdings. Reburials do not become new prepared sources.

## Single-cache schedule

Owner starts at source `(3,3)`. Observer starts at `(3,5)` for observed preparation and `(7,5)` for private preparation. On tick 0 the owner buries 12 at its actual position; observer holds. At owner vision 2, observed preparation gives both perceived exposure and a real sighting; private preparation gives neither. On ticks 1–7 owner holds. The observer follows this predeclared route, then holds through tick 11:

| Initial condition | Later opportunity | Positions after successive walking actions starting at tick 1 |
| --- | --- | --- |
| Observed | Observed redeposit opportunity | `(3,6)` |
| Observed | Private redeposit opportunity | `(4,5),(5,5),(6,5),(7,5),(7,6)` |
| Private | Observed redeposit opportunity | `(6,5),(5,5),(4,5),(3,5),(3,6)` |
| Private | Private redeposit opportunity | `(7,6)` |

At tick 8 owner policy begins. Its source is underfoot and nearest legal destination is `(3,2)`: retrieve on tick 8, walk on tick 9, deposit on tick 10 when an attempt occurs. At `(3,6)`, the observer is outside the owner's sight but can see the redeposit with vision 6. At `(7,6)`, it cannot see that deposit. The owner's cue, observer vision and distance mismatch are explicit supplied properties; no controller reads actual sightings. Policies without an attempt have no redeposit, so the assigned opportunity is not a realized observation.

Owner resumes ordinary behavior after completion; both four-unit patches are visible in sequence and allow it to leave the new cache rather than accidentally block pilfering by occupying it. Off can also collect both patches. The opportunity check asserts that the new site is unoccupied when the observer can reach it; it does not assert equal subsequent harvest or successful theft.

At tick 12 observer resumes ordinary rule M/raid behavior. It retains old source memories and any witnessed new deposit; expired/emptied entries follow existing sweep/arrival rules. There is no forced raid and no deletion of an old memory because food moved.

## Mixed-history schedule

Source A is `(3,3)` (observed); B is `(5,3)` (private); observer starts at `(3,5)`. Owner starts at A for A-first and B for B-first. Deposit 6 at the starting source on tick 0; walk to `(4,3)` on tick 1; walk to the other source on tick 2 and deposit 6 after that move; walk back to `(4,3)` on tick 3; hold ticks 4–7. Those two distinct deposits are the only preparation burials. Observer holds through tick 2, then walks ticks 3–7 along `(4,5),(5,5),(6,5),(7,5),(7,6)`.

At tick 8 owner policy begins from the same midpoint. Each source is one step away; each first relocation's destination is one step from its source. Selection order is oldest burial tick, then site index. The second indiscriminate attempt can have different path geometry after the first; report per-source travel and do not mistake that for exposure selectivity. Observer holds through tick 19 and resumes ordinary behavior on tick 20. This panel verifies selective handling and reports costs; it supplies no primary economic claim.

Both order reversal and horizontal reflection are mandatory. Each cell reports source A/B initial amount, perceived exposure, actual sightings, amount withdrawn, amount redeposited, attempts, cancellations and action distance. Selectivity is `(A withdrawn/6) − (B withdrawn/6)`; include zeros and distinguish withdrawal from completed relocation. The selective rule's intended selectivity is programmed behavior, not a scientific discovery.

## Cue-error and stumble schedules

**Cue errors:** use the single-cache schedule, private later opportunity and two controls. Visible nonwatcher starts `(3,5)`, never watches, and walks the observed-to-private route. Unseen watcher starts `(3,6)`, has vision 6 and watches; on ticks 1–4 walks `(4,6),(5,6),(6,6),(7,6)`. Owner holds until tick 8 in both. These create, respectively, a perceived exposure without a real initial sighting and a real sighting without perceived exposure. Observer release remains tick 12.

**Stumble encounter contrast:** use observed or private single-cache preparation, private later opportunity and observer at `(7,6)`. Hold observer through tick 17. On ticks 18–25, give it fixed neighboring targets `(6,6),(5,6),(4,6),(3,6),(3,5),(3,4),(3,3),(3,2)`. These are supplied encounter opportunities, not emergent search. Unlike preparation walks, each uses normal arrival gathering/raid/stumble semantics. At tick 26 resume ordinary decisions. Compare `theft.find=0` with `0.25` on this identical schedule. The source and redeposit site must be reachable/unoccupied in the intended construction checks; do not replace a blocked arrival with remote pilfering. This contrast can establish residual vulnerability during contact, not a field discovery hazard.

## Matrix, seeds and construction separation

Horizontal reflection is `x -> 8-x` for every role, source, patch and scripted target, with y unchanged. Apply the controller's ordinary site-index tie-breaking on the reflected map; do not override its destination to force reflected behavior. Reflected single-cache destination is `(5,2)`; mixed routes/second destinations can differ and must remain visible in the report.

| Panel | Factors | Cells |
| --- | --- | ---: |
| Single-cache | 4 policies × 2 initial visibility × 2 redeposit opportunity × 2 costs × 2 orientations | 64 |
| Mixed-history | 4 policies × 2 deposition orders × 2 costs × 2 orientations | 32 |
| Cue errors | 4 policies × 2 error types × 2 costs × 2 orientations | 32 |
| Stumble encounters | 4 policies × 2 initial visibility × 2 costs × 2 orientations × 2 discovery probabilities | 64 |
| Total | 40 seeds for every cell | **192 cells / 7,680 episodes** |

Scientific seeds are exactly 10001–10040. Construction tests use 7 and 8 with base orientation; reflection geometry tests check transformed coordinates separately. Native/WASM and checkpoint verification use seed 7. Construction results are not included in campaign estimates. Reflected full-episode outcomes remain the predeclared transfer panel, not a source for tuning. The 40 replicates support complete paired descriptive summaries; no power or independence claim about animal data is implied. Seed variation mainly changes turn ordering and discovery draws in this small lab; report duplicate trajectories.

## Outcomes and comparisons

Per-episode primary outcomes are **original food first transferred to the thief**, in units, and **owner ticks alive at tick start**, capped at 64. A death tick counts once; a survivor has 64. Also report alive at the horizon as a secondary endpoint. Initial deposition is the denominator for per-source food fractions.

For each of the 16 single-cache visibility/opportunity/cost/orientation strata, compare Selective minus Off and Selective minus Indiscriminate on both primary outcomes: 64 primary estimates. Also report Erased minus Off as an intervention check. No pooling of orientations or post hoc favorable strata. Within each original-visibility/cost/orientation stratum, report the observed-minus-private redeposit opportunity difference in Selective-minus-Off effects. Costs and opportunity treatments are descriptive interactions, not claims of causal benefit across unmeasured conditions.

Report all mixed-history selectivity and cue-error cells. In every stumble policy/initial-visibility/cost/orientation stratum, report find-0.25 minus find-0 for theft, owner survival time and discovered cache encounters. Use `survey::stats::paired_summary` with exact seed sets, Student-t descriptive 95% intervals, sign counts and no multiplicity-adjusted verdicts. Missing/capped lineage makes food estimates unavailable with the affected seeds listed; do not drop them to shrink the denominator. A malformed/incomplete archive is an analysis error. Do not force a Holds/Fails or overall-success classification.

Secondary exports include total harvest, closing holdings/live stock, owner-consumed tagged food, burial/metabolic costs, attempts, withdrawals/redeposits, protective action ticks/distance, cue errors, sightings, old/new-site arrivals/raids/wasted raids, cancellations, restriction ticks, actual phase boundaries and completed horizon. Preserve per-tick positions, holdings, stock, protection state and fingerprint for each living role and every death event; researcher truth stays out of controller input.

## Original-food ledger

Label each prepared source deposit as a cohort. When tagged cache food is withdrawn, move labels into holdings without changing the total. When holdings outflows occur, allocate them proportionally among labelled cohorts and unlabelled holdings, using the balances immediately before that operation. Debit shares in ascending cohort order and assign the floating-point remainder to unlabelled holdings, or the last positive labelled cohort if none is unlabelled. Costs, metabolism and deposits are separate operations in their actual engine order. Actual consumption is capped at nonnegative available food; starvation's negative holdings are not negative cohort consumption.

Reburial moves those shares back to a site; the scalar pending intent is never a second ledger balance. First pilfer transfer is absorbing for the owner's cohort, including kept loot; later use by the thief cannot count again. Owner death marks remaining tagged holdings/caches as terminal loss, with location recorded. Unlabelled environmental harvest stays unlabelled. Require, for each cohort and tick, `abs(initial - live - consumed - transferred - cost - lost) <= 1e-9 * max(1, initial)`. Also reconcile labelled cached/carried quantities against corresponding engine balances. Preserve legacy gross Dug/Buried flows without calling them unique beneficial recovery or production.

This small ledger has at most two cohorts and no cap-induced truncation. Collection is optional diagnostics; disabling it must leave fingerprints, RNG and all biological outcomes unchanged. If an implementation cannot maintain this contract, do not replace it with the legacy terminal fate log or proceed to a food-outcome claim.

## Verification and pre-mortem

Before scientific execution verify exact preparation holdings/stocks and owner cues, scripted routes without teleports, actual sightings at preparation/redeposit, separate retrieval/walk/deposit turns, nonblocking owner departures in the single-cache rig, ordinary observer opportunities, proportional ledger reconciliation and all source/destination cancellations. Verify default-off golden reductions, checkpoint continuation and native/WASM fingerprints. Units can use smaller hand-worked worlds for cap, wall, source-loss and expiry cases; never include these in scientific data.

Pre-mortem risks: selective behavior is guaranteed by its rule; Erased may be identical to Off; vision asymmetry is supplied; no-watchers waste protection; time restrictions suppress early raids; finite harvest and horizons constrain survival; an owner standing on a cache can create free blocking; stale old memories can delay renewed-information raids; carrying room, incidental partial retrieval or consumed pending stock can reduce reburial; a stumble effect requires contact; paired seeds diverge in random draws; repeated handling inflates gross flows; two-agent seeds can yield identical results; mirrored index ties need not mirror paths. Report realized opportunities and failures without filtering favorable runs.

## Archive and execution boundary

Default `--protection`/`--manifest`/`--help` print without simulations. Run command is `--protection --run --protocol-revision COMMIT --out NEW_DIR`; require a full committed 40-hex revision, clean tracked tree, new output directory and the full fixed matrix. Provenance flags record approval context; they are not approval themselves. Execute serially in canonical condition/seed order for this first small campaign. Save each complete raw envelope before analysis, then an index with schema, manifest, code/protocol revisions, actual relative raw paths and completion status. Do not overwrite or silently resume conflicting outputs.

`--protection --analyze INDEX --out NEW_DIR` validates identities, seed sets, configuration, phase schedules, horizons, reconciliation and raw references before rendering `analysis.json` and `results.md`. Reanalysis must reproduce those two artifacts byte-for-byte from identical saved data; wall-clock diagnostics are stored raw but excluded from comparative biological payloads and ordering. Record run/step time as descriptive computational cost, with construction and file-I/O boundaries stated.

No source-paper numeric target, tuned theft/survival band, threshold verdict, trait evolution, animal-cognition conclusion, deception claim or safety/regulation conclusion belongs to this campaign.
