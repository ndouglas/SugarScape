# Minds 9: declared measured-analysis amendment

Status: **approved by the user for campaign execution**.

Approval recorded from the user’s response, “Yeah, let's continue!”, to the separate amendment-review gate. The declared protocol below is unchanged.
This amendment binds the analysis before any campaign, diagnostic, pilot, seed search or timing run.
Correctness tests use deterministic synthetic records and pure configuration/cohort construction.
The [approved design](2026-10-02-minds-9-spatial-hoarding-design.md) remains binding.

This campaign reports measurements, with **no Holds/Fails or stability verdicts**. There are no
post-hoc thresholds, no fitted apparency breakpoint, no V&J 0.219 calibration and no imported Minds 7
endpoint classification. Future formal claims require another separately reviewed amendment.
A measured interval or an interior inherited share does not by itself establish a biological result.

## Exact cells and seed roles

Every condition uses theft-winter's single-good world: 175 founders, capacity 50, reserve horizon 20,
100-tick summer and 100-tick winter, winter divisor 32, half-surplus allocation, even caching,
owner memory, walk speed 1; ordinary reproduction, replacement and other incompatible mechanisms
remain disabled. Each season is 200 requested ticks. The spatial extension is enabled;
D=0.5, slope=10, scatter/larder find=0.25, guard on, raid_if=better, ordinary stumble precedence
and ordinary paid guarding unless a cell explicitly changes them. Off watching uses stored span 2;
on strata use span 2 or 7. Different panels retain distinct ids even for duplicated settings;
copies are never pooled as independent samples.

| Panel | Exact cells | Generations | Count |
|---|---|---:|---:|
| Fixed strategy | L={0,0.5,1} × guard={off,on} × watching={off,2,7}; D=0.5; supplied exact traits; no cheaters | 1 | 18 |
| Discovery evolution | scatter find={0,0.02,0.05,0.25,1} × watching={off,2,7}; larder find=0.25; sampled L/D; Survival | 60 | 15 |
| Continuous evolution and selection controls | watching={off,2,7} × selection={Survival,Neutral,Stores}; sampled L/D | 60 | 9 |
| Frozen contests | strategy family={forgoing scrounger vs producer, burying watcher vs nonwatcher} × initial flagged share={0.1,0.5,0.9} × watching mechanism={off,2,7}; exact L=0,D=0.5; Survival; traits fixed | 60 | 18 |
| Sensitivities | watching={off,2,7} × change={Stores, h²=1/Vseg=0, Neutral+h²=1/Vseg=0, slope=5, slope=20, scatter-first stumble, guard-harvest probe}; sampled L/D; otherwise continuous Survival baseline | 60 | 21 |

There are **81 comparison cells × run seeds 1–40 = 3,240 full-run records**. The requested upper
bound is 151,920 comparison generation records (720 fixed plus 151,200 multi-generation), before
terminal stopping. Fixed and multi-generation records are analyzed within their declared contrasts.

Timing has a separate `timing` role: the 18 fixed cells and the three continuous Survival baselines,
each using seeds 1–5: **21 cells × 5 = 105 timing runs**, up to 990 returned generation records.
Timing condition/seed pairs may match comparison pairs; role keeps them distinct. Timing runs also
retain full raw envelopes. There are 3,345 total run envelopes in the complete campaign.

For all continuous/discovery/sensitivity initial cohorts, L and D use the documented independent
logit-normal sampler at centers 0.15/0.5 with variance 0.5. Initial, episode and breeding seeds each
use the run seed in separate PCG64Mcg streams. Episode seed repeats every generation; breeding uses
one continuing stream. Default h²=0.8 and segregation variance=0.5. These are distribution centers,
not guaranteed realized arithmetic means. The actual starting cohort, realized L/D means, cheater
and watcher shares, sampler configuration and versioned draw order are saved. Every enabled all-watch
cell starts with every founder's watches=true; ordinary off cells start with every watches=false.

Contest flags use the existing deterministic founder-id share assignment (floor(175×share) founders,
no flag RNG), so the realized shares are 17/175, 87/175 and 157/175. Continuous traits remain frozen,
including endpoints, and both flags follow the first parent together. In the scrounger family,
flagged founders cheat and use the forgoing scrounge rule; only scroungers watch in enabled strata;
producers never watch. The off scrounger stratum retains cheating but has no watcher flags. Enabled
scrounger comparisons therefore describe a **joint cheating/watching strategy**, not a pure cheating
effect. Forgo retains its existing fresh-memory semantics; absence of remembered targets can still
permit ordinary fallback. In the burying-watcher family nobody cheats; the flagged share watches
in spans 2/7. Its mechanism-off control **retains the same inherited watcher flags**, while disabling
observation/raiding. Flag trajectories remain real, but watching advantage/spread is marked unused
off. All watcher-contest founders continue to bury.

All 81×40 constructions are deterministically materialized and validated before stepping any world.
No manifest cell selection, seed-count override or output-guided parameter choice is supported.

## Declared contrasts

The machine manifest contains the complete contrast list (129 declarations). All measurements use
**A minus B**, matched by run seed. A reference of zero is explicit for within-run inherited-share
change. A primary designation labels the question; it does not create a verdict.

- **Does a larder pay?** Three primary one-season survival contrasts: L=1 minus L=0, guard off,
  separately for watching off, span 2 and span 7. Report deliveries, returns, stock exposure,
  contact, consumption and closing stocks alongside survival; “home-travel” identifies this combined
  larder/travel contrast, not an isolated estimate of travel cost.
- **Does defense pay?** Three primary one-season survival contrasts: guard on minus off at L=1,
  separately by watching stratum. Report guard intentions/executions, prevented attempts,
  discoveries, exposed stocks and consumption. Guard-harvest is a separate sensitivity.
- **Does discovery alter evolution?** Twelve primary observed-endpoint L contrasts: each
  scatter find {0,0.02,0.05,1} minus find 0.25 within each watching stratum. Retain all L/D trajectories,
  survival, terminal outcomes and completed generations. No favorable sweep cell is promoted to a
  general mechanism claim.
- **Selection and inheritance controls:** Survival minus Neutral, and Stores minus Survival,
  for endpoint L, endpoint D and survival in every watching stratum (18 secondary contrasts).
  Neutral removes mortality selection by giving every archived founder weight 1, including dead
  founders; complete extinction still terminates. Stores uses living closing scatter+larder only,
  not holdings, and never adds a scrounger subsidy or uniform fallback.
- **Contest share changes:** Eighteen primary endpoint-minus-initial changes against explicit
  zero references: cheater share in the scrounger family and watcher share in the watcher family.
  Watcher changes with mechanism off are unavailable/unused for spread interpretation, while actual
  flag changes remain in the machine trajectory. Within-generation flagged-minus-unflagged survival
  gaps are secondary explanations, with each group's archived founder count as its own denominator.
  Twelve secondary whole-cohort survival contrasts compare spans 2/7 against the same family's
  share-matched mechanism-off cell.
- **Sensitivities:** 63 secondary endpoint contrasts (L, D, survival) against the watching-matched
  continuous Survival baseline. Each changes exactly the stated factor (Neutral+inheritance is the
  declared combined control). Neutral+inheritance helps separate quantitative regression/noise from
  mortality selection; no sensitivity produces a new main verdict.

Restoring frequency-dependence discussion requires the **fixed-trait** rare (0.1) and common (0.9)
starting shares within a declared strategy family and watching stratum, directional endpoint changes,
repeatability across seeds and the associated survival/terminal evidence. The middle start (0.5)
is descriptive. There is no automatic “stable mixture” classification, no selected endpoint band,
and no survival sufficiency threshold invented after seeing output. Continuous L/D evolution with
regression/noise cannot establish categorical-strategy stability. Joint-strategy confounding and
mechanism-off unused controls must remain visible in any narrative.

## Units, denominators and missingness

The machine analysis carries definitions for every measured metric. Raw `EpisodeEvents` mirrors
all StoreEvents fields; none are omitted. Counts widen to u64; food values remain f64.

| Measurement | Unit and denominator |
|---|---|
| Survival | living founders / all 175 archived founders at episode close |
| Ticks alive per founder | total live-at-tick-start agent-ticks / all 175 founders; death tick included |
| L/D means, categorical shares | arithmetic means/counts across all archived founders, including dead founders |
| Observed endpoint | last returned generation's founder cohort, labeled actual generation and terminal status; no generation-59 carry-forward |
| Share change | observed endpoint founder share minus actual initial share, not requested nominal share |
| Founder endpoint distributions | archived L, D and selection parent weights; n, zeros, positives, min, median, max, sum, plus exact values in raw founders |
| Closing holdings/scatter/larder | food units, separate stocks; dead founders have zero closing stocks |
| Kind recovery | cumulative dug food / cumulative buried food of that kind |
| Kind pilferage rate | cumulative caches_pilfered / positive cache-ticks of that kind |
| Kind loss rate | (cumulative pilfered food + owner-death lost food) / food-unit-ticks of that kind |
| Cache/stock exposure | always-on pre-step positive stores/food across executed ticks; empty initial stores contribute zero; final closing stock adds no tick |
| Discovery success | discovery_hits / discovery_draws; report contacts, blocked discoveries and draws separately |
| Metabolism | demand and actually consumed food; starvation demand is not assumed consumed; burial costs separate |
| Delivery | starts, completions, cancellations, elapsed return turns; delivered food and deposit burial cost separately |
| Guarding | intentions, executed paid turns, recovery food, blocked raids/discoveries; probe-only harvest is explicit food units |
| Observations/raids | every runner observation counter is retained with its original name; events/draws/entries or raided food as defined in raw schema |

Zero denominators yield **absent** ratios (`null` in machine analysis), never zero or infinity.
Mechanism counters of zero remain actual measured zeros; absent per-kind exposure and missing
strategy groups are marked separately. The gate-dependent `pilfer_candidates` field is retained
but is never substituted for always-on exposure. Ordinary guard-probe harvest zero is distinct from
an active probe's additional harvest. The guard-cost probe permits current-cell harvest after a
paid guard turn only when own recovery took no food; it is never relabeled ordinary guarding.

If **any declared seed** lacks either defined metric in a paired contrast, that entire contrast's
mean/interval/sign summary is unavailable, with every missing seed listed and each defined seed
value retained. Do not silently shrink n. An unpaired run, duplicate condition/seed pair, unknown
condition, missing generation or terminal row, configuration/cohort/seed/draw-order mismatch,
nonfinite measured number, contradictory archived counts/means or incorrect parent weight is an
analysis error, not an omitted row. Output paths are validated against declared role/id/seed names.

The detailed capped FIFO fate ledger is **not collected** by this runner. Raw metadata says
`not_collected`; aggregate flows and uncapped authoritative stocks/exposures support the measurements
above. No fate-age/fate-fraction estimate is made from that absence. A future collected-but-capped
or incomplete ledger must be explicitly labeled and cannot support detailed fate inference;
the current route rejects a substituted ledger-status schema rather than silently treating it as
complete. No fate-log omissions are interpreted as food loss, metabolism or survival.

## Paired uncertainty and terminal handling

The run seed is the sampling unit: ordinarily n=40. Generations, founder slots and duplicated
panel settings are never independent samples. Compute seedwise full-run differences, arithmetic
mean, positive/zero/negative counts and two-sided 95% Student t intervals with df=n−1. The .975
quantile is obtained by bounded bisection of the existing Student t CDF; no normal approximation.
Identical pairs have an exact zero-width interval. n=1 has no interval; an empty sample is an error.
Intervals are descriptive and unadjusted across the declared contrasts. They do not authorize
selection of significant/favorable cells or causal generalization across strata.

Every returned generation is retained, including **extinct** and **zero_fitness**, with every archived
founder, parent linkage and weight. Extinction and live zero Stores fitness remain distinct. No
uniform fitness fallback, invented survivor, zero substitution for an undefined ratio, last-value
trajectory extension or survivor-only endpoint is allowed. Terminal weights are zero as produced
by the runner. Per-condition terminal counts use all 40 runs as their denominator; report actual
completed-generation ranges. Endpoint contrasts include terminal observed endpoints and are
explicitly descriptive when partners have different completed-generation counts. Raw records and
machine summaries preserve each partner's actual generation; no terminal run is dropped.

## Timing and persistence procedure

Comparison and timing files have separate roles and condition/seed identities. First validate the
full manifest. For each run, materialize config and starting cohort, run the exact runner, then save
its **full raw envelope** before analysis or report rendering. Raw JSON records schema, actual code
revision, amendment revision/path via index, probe in config, all initial/archived cohorts, sampler,
seed conventions and versioned draw order. Each file is flushed/synced; an incrementally updated
index records completed files. Partial indexes are rejected. Avoid holding all founder-level runs
in memory: analyze each file into compact trajectories and distributions, retaining original files.

For timing only, the independent full-run wall clock wraps the runner, **including** world
construction, biological steps, breeding and deterministic bookkeeping; **excluding** initial
cohort sampling, subsequent timing replay, file writes, analysis and rendering. Divide by the
actual number of returned generation records (including the terminal episode), not requested 60.

After saving the completed timing envelope, reconstruct each generation from its archived founder
traits and recorded episode seed. Time **only each World::step** call, excluding constructors,
cohort extraction, loop population checks, serialization and reporting. Sum those elapsed times
and divide by actual live-at-tick-start agent-ticks; the death tick counts. Check replay tick count,
survivors and live-agent-ticks against archived records. Report raw seconds, elapsed ticks, actual
completed generations, survival and terminal outcome. The replay is a measurement of identical
whole-tick work, separate from full-run time. Retain all seeds 1–5; no warm-up/pilot search or
favorable-seed selection. Any undefined timing denominator is absent and labeled.

## Actual commands and scientific gate

These commands inspect help/manifest without simulations:

```sh
cargo run --manifest-path survey/Cargo.toml -- --minds9 --help
cargo run --manifest-path survey/Cargo.toml -- --minds9 --manifest
```

After this committed amendment is separately reviewed and approved by the user, Task 9 records that
approval and executes the exact declared release route, with a full amendment commit revision:

```sh
cargo run --release --manifest-path survey/Cargo.toml -- --minds9 --run \
  --amendment-revision AMENDMENT_COMMIT_SHA --out NEW_CAMPAIGN_DIRECTORY
```

`--run` is explicit; default `--minds9` prints only the manifest. Execution requires clean committed
tracked code and verifies the named commit contains this amendment. The revision is **provenance,
not authentication of user approval**; the external scientific gate remains mandatory. The command
has no cell or seed override. The output directory must be new; partial files are retained on error
and never silently replaced. Keep untracked temporary output under the plan's ignored workspace.

To reproduce analysis without stepping any world, use a complete index and a fresh report directory:

```sh
cargo run --manifest-path survey/Cargo.toml -- --minds9 --analyze CAMPAIGN/index.json \
  --out NEW_ANALYSIS_DIRECTORY
```

The route writes reproducible `analysis.json` and `results.md`; raw source files remain authoritative.
The existing survey registry and its Holds/Fails behavior are unchanged. No Minds 9 results or
scientific verdicts are produced by this amendment implementation. Stage 5 remains incomplete while
this separate approval gate is pending.
