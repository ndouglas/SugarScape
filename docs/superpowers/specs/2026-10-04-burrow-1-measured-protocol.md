# Burrow 1 measured excavation protocol

**Date:** 2026-10-04
**Status:** protocol design approved by the user on 2026-10-04; [implementation plan](../plans/2026-10-04-burrow-1-measured-harness.md) approved 2026-10-04; engineering harness implemented and verified on its dedicated branch, with independent branch review complete; integration pending. Candidate manifest remains unregistered; scientific execution is not authorized. No campaign has run.
**Engine baseline:** `4ef7e5794fc12138cf7f112947e34666095f6131`.
**Design:** [implemented excavation lab](2026-10-03-burrow-1-excavation-design.md).
**Candidate manifest:** [fully specified conditions and seeds](2026-10-04-burrow-1-draft-manifest.json).
**Next design:** [Burrow 2 resource access](2026-10-04-burrow-2-resource-access-design.md).

## Purpose and scope

Estimate the downstream effects of two supplied rules: direct versus relay spoil transport, and blind versus responsive frontier selection. The question is whether they change excavation and external disposal per paid worker opportunity in this bounded model. No treatment is presumed superior. A preference encoded in the controller is an intervention check, not evidence that the model reproduces animal behavior.

This is a computational mechanism study, not a reproduction of source-paper numerical results, an optimization claim, or a test of architectural intelligence. It measures the implemented horizontal world. Physical time, energy, body variation, learning, room purposes and social institutions remain outside its scope.

The common source anchor is Pielström and Roces, [Sequential Soil Transport and Its Influence on the Spatial Organisation of Collective Digging in Leaf-Cutting Ants](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0057040). Their experiments motivate sequential soil transport and fresh accumulated spoil as a local digging cue. Their findings do not specify our lattice, three-move relay legs, deterministic freshness boundary or integer preference coefficient. Those are supplied model choices. See the [source and capability audit](../../studies/2026-10-04-burrow-studies-reading.md).

## Existing information and prospective boundary

The design and engine are already public in repository history. Construction tests and native/WASM checks use seed 7 and the maximum u64 seed. A seed-7 relay-responsive demonstration at 512 ticks was inspected before drafting this protocol: 129 digs, 116 disposals, two carried units and eleven loose units. This is disclosed prior engineering information and is excluded from all estimates. The primary default geometry, horizon and controls were specified before that demonstration.

Scientific seeds 10001 through 10040 have not been executed for this study. Forty seeds are a fixed descriptive Monte Carlo budget, not an animal-data sample size or power calculation. Do not extend the seed set, change the horizon, select favorable strata or tune parameters after viewing its outcomes. Any amendment requires a new version with its reason and prior inspected information recorded.

## World and action costs

Use the shared `LabConfig`, fixture constructor and `run_episode`, without altering core defaults or controller code. Every condition contains all configuration fields; do not resolve research parameters from mutable defaults.

Growing geometry is 41 by 25, exit `(0,12)`, initial open cells `x=0..2,y=10..14`. Worker IDs and row-major spawn order are supplied. Soil is uniformly diggable, boundaries do not wrap, and no piles obstruct passages. At most two workers occupy an open cell; each carries at most one unit. Dig creates one unit and opens one adjacent cell without moving the worker. Moves, digs, pickups, drops, disposals, blocked attempts and waits each consume one opportunity. Birth times survive handoffs.

Each completed tick supplies one fresh seeded permutation and one sequential opportunity per worker. All growing episodes run their entire fixed horizon; do not stop when a visually appealing structure appears or at a favorable material count. There is no imposed excavation amount. Record requested and actual clocks and opportunities separately.

Unloaded workers observe only two open-cell hops and adjoining diggable faces. Pickup probability underfoot is one half in both cue modes. Loaded navigation receives the supplied global exit-distance field. Direct transport never internally drops; relay drops after three successful loaded moves unless already at the exit. Responsive target weights are three when at least two recent units occur on observed approach cells, otherwise one; blind weights are always one. Recent means age strictly less than 32 ticks. All other physical rules and random tie-breaking remain common.

## Fixed panels and candidate workload

Scientific seeds are exactly the forty decimal u64 values 10001–10040 for every condition. Seeds match treatments but do not force identical RNG streams after their decisions diverge. Analyze whole episodes as replicates; workers, materials and ticks are not independent samples.

| Panel | Conditions | Per-condition budget | Episodes | Role |
| --- | ---: | --- | ---: | --- |
| Primary growing | 2 transport × 2 cue | 8 workers × 512 ticks = 4096 opportunities | 160 | Downstream primary estimates |
| Workforce sensitivity | 2 transport × 2 cue × workers 2, 4, 16 | Respectively 2048, 1024, 256 ticks; always 4096 opportunities | 480 | Separate transfer estimates |
| Cue reduction | 2 transport × 2 cue, response weight 1 | 8 workers × 512 ticks | 160 | Programmed reduction check |
| Choice verification | 2 cue × 2 pile sides × 3 pile types; transport fixed direct | One requested tick; stop on first selection before action | 480 | Programmed intervention check |
| Total | 32 conditions × 40 seeds | Mixed panel roles retained | **1280** | No pooled overall verdict |

Growing panels sample every 32 completed ticks. Choice samples every tick. Every panel retains initial and terminal frames plus all actions and selections. Worker sensitivity preserves the 32-tick freshness window and three-move relay distance. Thus population changes total opportunities inside a cue window and the number of elapsed rounds before the endpoint; this is sensitivity to the implemented time semantics, not an isolated causal estimate of workforce size. Compare treatments within each population stratum. Do not pool those strata or interpret a lone population advantage as emergent coordination.

The externally tagged manifest configurations match the current engine. For choices, use `fresh_accumulation`, `old_accumulation`, `single_fresh`; place the pile at left or right approach. The worker starts at `(4,3)`, with only `(1,3)` and `(7,3)` diggable. Old material has supplied prior age. A choice record has one selection, zero actions, zero completed ticks and zero action opportunities; its final clock remains the fixture start clock. Transport is fixed because no transport action is committed in this rig.

## Primary outcomes and contrasts

Primary outcomes per growing episode are `digs / opportunities` and `disposals / opportunities`, derived from integer totals. Expected denominator is 4096. A zero or short denominator in a growing scientific record is a validation failure, not a rate to silently filter. Choice rates remain null.

Write `D_B`, `D_R`, `R_B`, `R_R` for direct-blind, direct-responsive, relay-blind and relay-responsive. Within the primary eight-worker panel report three signed primary contrasts for both outcomes:

1. Cue effect under relay transport: `R_R - R_B`.
2. Transport effect under blindness: `R_B - D_B`.
3. Transport effect under responsiveness: `R_R - D_R`.

These are six prespecified primary estimates. Direct cue response is a structural negative control: growing starts without loose spoil, direct never drops, and no other process creates loose material. Therefore `D_R - D_B` is programmed to be zero and their action/choice projections must be identical at every population. Report it as a correctness check, not a discovered null effect. If it fails, stop analysis and investigate implementation or archive validity before interpreting estimates.

Also show the interaction `(R_R - R_B) - (D_R - D_B)` per seed as a secondary derived quantity. When the negative control holds, it equals the relay cue effect and is not an independent result. Repeat the six primary-style estimates separately at each sensitivity population, labelled secondary. No positive-direction assumption, significance-selected reporting, multiplicity-adjusted verdict or overall Holds/Fails label is authorized.

Reuse `survey::stats::paired_summary` with exact identical seed sets, mean paired differences, descriptive Student-t 95% intervals and positive/zero/negative counts. For interaction, provide the two per-seed cue-effect maps to that helper. Show every condition's mean and seed-level values, and report duplicate action trajectories using an explicitly configuration-independent action/choice projection. Fingerprints include configuration, so unequal fingerprints alone do not imply different behavior. Intervals describe stochastic variation of this computational model; they do not certify calibration, external validity or animal-population uncertainty.

## Secondary diagnostics and censored observations

Report all integer action totals, loaded/unloaded travel, carried and loose spoil, actual completed horizon, per-worker work and unique excavated cells, event-time dig distances and spatial work. The new-open count is digs; initial staging is supplied geometry. Connected-open area is guaranteed by legal frontier excavation and must not be promoted to evidence of coordination. Chamber counts, branch counts and room labels have no registered extraction method and are not outcomes here.

Every unit remains in the analysis, including units never disposed. Report delivered count and the carried/loose censored counts. Delivered latency in ticks is disposal clock minus birth clock; observed ages of undelivered units are endpoint clock minus birth clock and remain marked censored. For growing episodes, also derive opportunity-index latency from the matching dig and disposal event indices. Tick latency can be zero for within-round transfers, so it is not interchangeable with opportunity-index latency.

Show delivered-only latency and carrier distributions as conditional descriptions with the censor count beside them, never as all-unit mean delivery times. Report observed loose waiting, including terminal censoring. Choice material's earlier birth is supplied history; observed waiting starts at fixture start, not birth. Do not treat researcher-seeded material as newly excavated work.

Report disjoint exit-field, observation and controller BFS calls/visits/peak queues and logical storage counts. Canonical comparative JSON contains no wall-clock timing. Optional elapsed time and raw byte sizes belong in a separately labelled operational log with hardware, build mode, execution order and I/O boundaries; they are not biological or action costs.

## Verification panels and exact expectations

The direct negative control above applies at both response weights one and three. Cue reduction must yield identical physical action and choice projections between blind and responsive conditions at response weight one, within each transport mode and seed. Full Episode bytes and fingerprints differ because configuration differs; do not compare those as a reduction assertion.

Choice expectations follow the supplied weights: responsive fresh accumulation has probability 3/4 of the pile side; blind, old accumulation and single fresh conditions have probability 1/2. Exact ticket enumeration already verifies the selection rule. Report counts by cue, pile type and side without a sampled-fit acceptance band, source-paper numeric target or discovery claim. Forty realized choices need not equal their expected fractions. Mirrored choice sides are compulsory; growing-map rotations and reflections are not implemented and are not silently assumed.

Separately run the 18 declared corridor construction configurations at seeds 7 and 8 only: direct/relay, lengths 4/8/16, workers 1/2/4; cue blind. Give each 512 worker opportunities, hence 512/256/128 ticks, sample every 16 ticks. There is exactly one available new dig cell. Retain all 36 results as construction evidence, including undelivered endpoints; do not call this a repeated-production transport benchmark or include it in growing estimates. Conservation, legality, record consistency and replay are pass conditions; disposal by an arbitrarily selected deadline is not.

## Archive, validation and implemented engineering harness

The committed candidate JSON describes fully resolved conditions and options, scientific seeds and separate construction seeds. It remains pinned and unregistered, with no scientific execution permission. The implemented harness checks strict manifest identity, constructs immutable archives, validates physical records and performs deterministic saved-only analysis using the existing survey conventions.

Implemented interface: `survey --burrow` prints the resolved manifest without simulating; `--run --construction --protocol-revision COMMIT --approval-context TEXT --out NEW_DIR` records all 36 construction episodes at seeds 7 and 8; `--analyze INDEX --out NEW_DIR` uses saved raw records only. Scientific `--run` is gated and rejects the current unregistered candidate before output creation. Saved analysis rejects run, seed and panel overrides. Execution requires reviewed protocol and executable manifest, a clean tracked checkout, recorded full code/protocol revisions, manifest SHA-256, normalized configs/options and a new output directory. Record approval provenance explicitly; a revision flag is not itself approval.

Use canonical manifest condition order then ascending seeds, serial execution, fixed relative output names, exclusive creation and no overwrite. Save complete core Episode JSON before analysis, an index of expected keys, actual relative paths and SHA-256 bytes, and completion or failure status. No silent resume or replacement. All scientific conditions are required. An interrupted archive stays incomplete; a deliberate continuation requires its own identity-preserving policy review.

Analysis rejects duplicate/missing/extra keys, mismatched seed/config/options/setup, malformed numbers, noncanonical condition identities, unexpected stop reasons, short growing horizons, bad hashes or inconsistent material/action summaries. Reconstruct recorded physical transitions and unit histories from supplied setup to verify legality, ownership, one-opportunity-per-worker-per-completed-round, conservation and final quantities. This consumes no new random decisions and creates no new scientific episodes. Fingerprints are retained state identifiers, not a standalone proof of controller fidelity; code provenance and existing engine verification supply that guarantee.

Save `analysis.json` and `results.md` in a new directory. Reanalysis of identical saved inputs must reproduce both byte-for-byte without rerunning stochastic episodes. Missing or invalid data produces an analysis error with exact condition/seed identities, never a reduced denominator or a favorable subset. Archives use `burrow-archive-v1`, complete Episode envelopes, an initial `index.incomplete.json`, exclusive `progress/` receipts, and final `index.json` only after complete saved-byte validation; failure retains `failure.json` and incomplete evidence. Analysis uses `burrow-analysis-v1`. Construction reports retain all 36 rows and censoring but omit growing scientific contrasts and condition means. Physical validation reconstructs transitions and accounting; controller search counters and fingerprints remain provenance-bound diagnostics rather than policy/RNG replay. No scientific result exists until fresh harness review, separate committed executable registration and separately authorized scientific execution are complete.

## Pre-mortem and readiness checks

Relay may add handling and fail to improve disposal. Freshness may recruit to a site that increases digging but leaves spoil behind. Supplied navigation can conceal navigation costs. Spawn order, boundaries, four-neighbor geometry, retained targets, two-worker capacity and the tick-based freshness clock can influence results. Seeds can produce duplicate trajectories, and equal seeds diverge after treatment-specific draws. Fixed horizons censor more distant or slow deliveries. Reporting only disposed-unit latency biases comparison toward completed transfers. None of these is grounds to tune after seeing the matrix.

Before execution, review the exact candidate keys, full parameter values, six primary contrasts, equal action budgets, construction/scientific seed separation, null-rate behavior, archived transition validator, reductions and saved-only reanalysis. Matched excavation endpoints, growing-map orientation controls, corridor replenishment, cue-window sensitivity, relay-distance sensitivity, local navigation and chamber extraction remain separately designed follow-ups. They are not implied by this first registration.

## Draft review record

The candidate has 32 unique fully specified conditions and 1280 proposed episode keys. Static checks confirm all growing action budgets are 4096, construction budgets are 512, seeds remain separate, and six primary outcome/contrast estimates are defined. All fifty configurations passed baseline native zero-tick constructor/normalization checks at seed 7, with no actions or scientific seed execution. See the audit for checks and candidate SHA-256. These are readiness checks for the draft; the engineering plan was subsequently approved and implemented; executable registration and scientific execution remain separate unapproved gates.


## Engineering acceptance record

The harness runtime commit `a4e0effd0532194621afbdd5672bc21cd0594235` passed 2,111 workspace tests (102 preexisting ignored), 234 survey tests, both Cargo formatting checks and both all-targets clippy checks with warnings denied. Clean-tree construction execution produced 36 validated raw episodes; saved-only analysis ran twice with byte-identical `analysis.json` and `results.md`. The earlier Task3 archive also reanalyzed twice identically. Tampered raw bytes rejected with no output directory. This is construction/legality/conservation/consistency evidence, not a scientific transport estimate. No scientific seed was executed. Exact commands, hashes, paths and review rulings are preserved in the [plan closure](../plans/2026-10-04-burrow-1-measured-harness.md#engineering-closure) and [lab usage](../../burrow.md#measured-archive-and-saved-analysis).
