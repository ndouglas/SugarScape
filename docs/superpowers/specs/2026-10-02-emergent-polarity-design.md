# SugarScape milestone — Emergent polarity — Design

**Date:** 2026-10-02. **Milestone:** 36, subject to assignment at integration.
**Kind:** `polarity`; displayed as **Emergent Polarity**.
**Design basis:** scope and architectural direction approved in chat on 2026-10-02.
**Research:** `2026-10-02-emergent-polarity-reading-notes.md`, including author-code and critic
protocol audits; October 1 book notes remain supplementary with the documented corrections.
**Campaign:** `2026-10-01-geopolitics-lineage-plan.md`, phase 1.
**Sources:** Cederman (1994), *Emergent Polarity*, ISQ 38(4):501–533,
[publication record](https://icr.ethz.ch/publications/emergent-polarity/); Cederman (1997),
*Emergent Actors in World Politics*, chapters 4–5. Page references use printed pages.

## Intended outcome and scope

Build an inspectable reconstruction of states emerging through conquest on a grid. Reproduce
originator experiments; investigate whether defense and alliances preserve power politics
or instead protect fragmentation and hegemonic growth; then investigate provincial revolt
and overextension. Success is a faithful experiment and a truthful findings report, including
reconstruction dependence and failed reproduction. Matching the plots is not a tuning goal.

Include the original model, behavioral alliances, proportional resource allocation (PRA),
two-level tax/revolt, and stochastic overextension. PRA is needed both for the chapter 5
variants and the author's allocation/alliance sensitivity question. Compare its no-alliance
Fig.5.6 before interpreting that interaction. Exclude chapter 5 strategic adaptation,
strategic blurring and capability-based alliances, chapter 7 mobilization, later GeoSim
technology shocks/war-size fitting, democratic peace, and a generalized conquest framework.
These are distinct experiments with distinct source rules.

The model serves the existing playground, CLI, WASM and survey. The war-and-society study
receives resource destruction, conflict episodes and end causes. These quantities are
abstract model resources and events; they do not measure people killed. Emergent political
coordination supplies a comparative mechanism, without a new claim about AI behavior.

## Binding constraints and evidence boundaries

- Existing configs, model fingerprints and fixtures remain unchanged. Add new golden cases.
- Native and WASM execute the same seeded mechanics with portable arithmetic. No survey-only
  fast engine, platform-dependent iteration order or imported third-party model source.
- The 2004 GeoSim0 code explicitly implements a later descendant. The GROWLab distribution
  differs again. No original EPM executable was recovered; do not claim author-code docking.
- PDFs and third-party archives stay ignored. Read-only external comparisons may inform
  attributed alternative rules; they may not silently replace original rules.
- Every ambiguity has an enum or numeric parameter, a selected default, source citation and
  description identifying the choice as reconstruction. Export the resolved configuration.
- Source figure extraction, study manifest and analysis decisions are fixed before native
  measurement. Later amendments retain the original verdict and identify the amendment.
- Long studies stay outside CI. Planning follows the previous milestone's verified scratch
  implementation, byte-for-byte reconstruction check, native subagent execution and fresh
  final review. No scratch implementation starts until this spec is approved.

## Architecture and state

Add `ModelKind::Polarity`, `ModelConfig::Polarity(PolarityConfig)`, and `PolarityWorld` under
`crates/sugarscape-core/src/polarity/`. Follow existing schema, model, RNG, canvas, statistics,
export and experiment interfaces. Integration covers kind dispatch, reset validation,
presets, CLI, WASM, page controls/charts/Inspect/Compare/Experiments and survey reports.

| Unit | Responsibility |
|---|---|
| `config.rs`, `presets.rs` | Validated source profiles, named readings and attributed treatments |
| `territory.rs` | Grid adjacency, government membership, capitals, contiguity and structural claims |
| `decision.rs` | Foreign/domestic actions, initiation guards, border paths and prior action memory |
| `alliance.rs` | Behavioral trust, prime threats, coalition membership, deterrence and obligations |
| `resources.rs` | Stocks, harvest, equal/PRA commitments, damage, taxes and transfers |
| `combat.rs` | Dyadic encounters, deterministic/stochastic victories and claims |
| `analysis.rs`, `stats.rs` | Categories, session outcomes, episodes and survey comparisons |
| `world.rs`, `view.rs` | Period orchestration/model adapter; bounded rendering and inspection |

Use fixed cell IDs in row-major order. Each cell retains its latent type, government capital,
and resource stock where its profile permits provincial stocks. Capitals retain foreign
relations and corporate stocks. Government membership is derived consistently from ownership;
no actor is deleted and renumbered. Sovereign neighbors are distinct capitals touching any
owned cell. Fronts are canonical ordered capital pairs with directional action, prior
commitment, stored path and episode metadata. Two-level domestic fronts use capital/province
IDs in a separate relation namespace. Trust is directed; coalitions are identified by their
common threat, not a mutable random ID. All reductions use sorted IDs.

Structural operations accept explicit snapshot claims and return changed cells, stock
transfers and episode terminations. They never make new strategic decisions. Rendering and
analysis consume state; they do not consume randomness or modify the simulation.

## Source profiles and explicit readings

`variant` is `epm` (default), `two_level` or `overextension`. `source_profile` is `chapter4`
(default) or `chapter5`; the latter is required for the two extended variants. Source-profile
resolution sets source-defined mechanics, including CD/DC damage. Explicit reading overrides
remain visible. Reset is required for all parameters; there are no live parameter ramps in
this milestone.

| Parameters | Default / semantics |
|---|---|
| `width`, `height` | 10,10; each 2..100, at most 10,000 cells |
| `topology` | `bounded`; alternative `torus`, both four-neighbor |
| `predator_share`, `placement` | .2, `exact_count`; .2 is a playground selection, not a unique paper default. Use floor(share×cells+.5); shuffle cells. Alternative `bernoulli` |
| `initial_mean`, `initial_sd` | 50,10 |
| `harvest_mean`, `harvest_sd` | 2,5, independent draw per primitive unit per period |
| `resource_distribution` | `normal`; `bounded_uniform` is an attributed sensitivity, not original default |
| `superiority`, `victory` | 2,2; positive thresholds greater than 1; defense preset 3,3 |
| `damage_rate`, `asymmetric_damage` | .05, `source`: chapter4 damages CD/DC, chapter5 does not. Overrides `on`, `off` |
| `allocation` | `equal`; alternative `pra` |
| `action_memory` | `previous_action`; alternative `war_until_victory` |
| `schlieffen_gate` | `own_current_defections`; alternatives `previous_hostilities`, `unresolved_war` |
| `combat_path` | `stored_episode`; alternative `redraw_each_period` |
| `tie_break` | `lowest_id`; alternative `random`, used for equal strategic targets and prime threats |
| `path_collision` | `lowest_id`; alternative `random` initiator path |
| `pra_active` | `either_defection`; alternative `mutual_defection` |
| `victory_timing` | `before_damage`; alternatives `after_damage`, `after_harvest` |
| `update` | `snapshot`; alternative `sequential`, defined below |
| `locking` | `affected_cells`; alternative `affected_states` |
| `capital_capture` | `collapse_only`; alternative `capture_and_fragment` |
| `province_transfer` | `equal_share`; alternative `primitive_stock_only` |
| `resource_policy` | `signed`; alternatives `floor_zero`, `reject_nonpositive` |
| `alliances` | false; boolean behavioral-alliance treatment |
| `trust_initial`, `threat_threshold` | 0,0; prime threat requires trust strictly below threshold |
| `negative_trust_rate`, `positive_trust_rate` | .5,.01; trust targets −1000,+1000 |
| `threat_observation` | `neighbor_aggression`; alternative `dyadic_aggression` |
| `obligation_timing` | `same_period`; alternative `next_period` |
| `pra_attack_rule` | `diagram`; alternative `literal_prose` |
| `pra_alliance_support` | `front_commitments`; alternative `stocks`, defined below |
| `tax_rate`, `tax_discount` | .4,.7; only extended variants use taxes, two-level discount defaults to 1 |
| `tax_distance` | `manhattan`; alternative `territorial_path` |
| `stochastic_threshold`, `stochastic_exponent` | 3,5 for overextension; threshold is an explicitly inferred run setting from Fig.5.9 |
| `stochastic_resolution` | `single_draw`; alternative `independent_draws` |
| `horizon`, `stop_at_hegemony` | 1000,true for EPM/two-level; 4000,false for overextension (4000 is our observation horizon) |
| `periods_per_tick` | 1; integer 1..10,000, executing complete periods |
| `event_log`, `event_log_limit` | false,1000; cap 0..1,000,000, positive when logging. Include monotonic event IDs and dropped-record count |

Means must be finite, SDs nonnegative, rates/shares/taxes/discounts in [0,1], exponent positive,
trust parameters within [−1000,1000], horizon 1..1,000,000,000. Reject irrelevant/inconsistent
combinations such as provincial variants with equal allocation, or stochastic combat outside
overextension, or sequential updates with after-harvest victory. Validation messages identify fields and corrective choices. The plan will
record exact schema field representations without changing these semantics.

### Decisions, paths and allocation

At each period, status-quo states play D against neighbors whose prior action was D, else C.
Predators use the same provisional actions, then may initiate one new attack. Default guard
allows initiation iff none of their provisional own actions is D. `previous_hostilities`
blocks if either side played D on any existing front in the previous period; `unresolved_war`
blocks while any episode is open. An initiation selects the weakest eligible neighbor by
total stock, with ties resolved by `tie_break`, and requires strict total superiority.
Behavioral-alliance deterrence substitutes supported coalition stock for victim stock;
attacker support enters only when its coalition names the victim as threat.

For `war_until_victory`, a previous own D or an open episode preserves D until an explicit
termination. This is a named persistence interpretation supported by later code and Radax,
not recovered original Pascal behavior. With literal TFT, initiation can create DD through
reinitiation; a mechanistic test must demonstrate that rather than assuming inevitable echo.

A new path samples an attacking border cell uniformly, then one adjacent defender cell
uniformly. The capital is eligible. Retaliation reverses an existing path; if a new foreign
relation has no valid path, sample one by the same two-stage rule. Opposed new attacks share
one canonical encounter; choose the lower-ID initiator's sampled path under `lowest_id`, or sample one of the two
under `random`, and record the collision.
Stored paths remain only while the cells retain their roles and adjacency. Structural
invalidity ends the old episode; any later fight begins a new one. A defender may capture
the attacking cell when its local ratio wins; a defender-stalemate interpretation of later
GeoSim is outside this original-model profile.

Equal allocation is corporate stock divided by distinct sovereign-neighbor count. PRA uses
previous opponent commitments: active-front commitment is `R_i * r_ji / A_i`, with `A_i` the
sum of opposing commitments on active fronts; passive-front commitment is
`R_i * r_ji / (A_i + r_ji)`. Default active status means either side played D last period; `mutual_defection` requires DD.
Initialize old commitments with equal allocation. If a denominator is zero, divide the
stock equally across active fronts, or announce full stock on a passive front when no
active fronts exist. Signed-denominator arithmetic otherwise follows the displayed equations.
A newly created relation receives equal-allocation old commitment. PRA commitments on
passive fronts are conditional; do not normalize them to an additive budget.

For PRA deterrence, `front_commitments` sums coalition members' conditional commitments
toward the aggressor; a member without that front contributes zero. Attacker support is
eligible only when its coalition names the victim as threat. `stocks` instead adds other
eligible members' total stocks to the focal actor's front commitment. With no coalition,
both retain the focal commitment. Actual combat and stock accounting remain individual.
The source gives no PRA/alliance pooling formula; these are explicit reconstruction readings.
A small stocks-support control repeats the chapter5 allocation/PRA/alliance strata with20
seeds; the200-seed interaction uses front commitments and reports its selected reading.

PRA initiation maximizes the front ratio and uses strict `>` (`diagram`). `literal_prose`
instead minimizes and uses `<`, reproducing the printed inconsistency. Both choices retain
the initiation guard. The Fig.5.5 formula yields 26.1 in the disputed cell, while the figure
prints 21.4; record that discrepancy rather than inserting a special-case number.

### Stocks, damage and numerical policy

Only encounters with at least one D evaluate combat or victory; CC cannot generate a claim.
All source normal draws remain untruncated under `signed`. Use the existing portable
Marsaglia normal sampler, reused without changing its draw sequence or other model behavior.
For the bounded-uniform control use continuous `mean + sqrt(3)*sd*(2u-1)`, matching variance;
this is our control, not Störmer's integer law. Harvest is drawn independently per cell.

Default victory checks use frozen allocations before damage. Apply all period damage as a
simultaneous ledger, then add harvest, then apply structural transfers. Timing alternatives
recompute the victory comparison using the same front allocation rule and old neighbor graph
with after-damage or after-harvest stocks; PRA weights remain frozen. Nonfinite arithmetic
terminates the session as invalid with period/front/config context; invalid sessions remain
in reports and never silently enter terminal category distributions.

Under `signed`, the displayed source matrix may imply a resource increase from a negative
allocation. Preserve and count these signed effects explicitly. Report positive destruction,
negative-damage creation and frequency of nonpositive stocks; never label their net sum as
pure destruction. Define division by zero as +infinity for positive numerator, −infinity
for negative numerator and 0 for 0/0, used only in ratio comparisons. Negative finite ratios
follow ordinary arithmetic. This is numerical reconstruction, not a claim that the source
specified sensible negative-power behavior. `floor_zero` clamps initial and period-final
stocks and records clipped amounts. `reject_nonpositive` rejects at reset or the first
nonpositive stock. Register the numerical-policy control before interpreting substantive
claims; an invalid or materially policy-dependent result must be disclosed.

### Conquest, collapse and locking

Primitive capture transfers its remaining stock and government to the winner in EPM.
Province transfer under `equal_share` moves its pre-event per-cell share of the losing
corporate stock; `primitive_stock_only` moves no share for an already-dependent province.
This distinction is unreported in the source. Disconnected cells become independent with
pre-event equal per-cell stock shares; debit those shares once from the remaining state.
Restore latent strategy on independence. Capital fall under `collapse_only` releases every
unit, including the old capital, with equal stock shares; no cell is annexed in that event.
`capture_and_fragment` releases the provinces and incorporates the capital into the winner.
The first policy follows the original's collapse-then-subsequent-capture example.

Shuffle snapshot claims once per period and apply them in that order. Check current ownership
against snapshot identities before every application; stale claims are skipped and counted.
`affected_cells` locks targets, released/transferred cells and capitals whose stocks change;
if a candidate touches any locked cell, skip it, without deferral. `affected_states` locks
all pre-event members of both involved sovereign states. Compute all transfers from the
candidate's current pre-event stock snapshot. Conservation tests cover simultaneous losses,
multiple disconnected provinces and capital fall. Rebuild adjacency after structural changes,
remove invalid fronts/trust and rebuild coalitions; no stale front may drive a later capture.

### Alliances and time order

Trust at the beginning of a period uses the previous period's behavior. Under default
`neighbor_aggression`, every current neighbor observes whether an actor initiated or conquered
in the preceding period; `dyadic_aggression` observes only aggression against the observer.
Aggression is a negative update, otherwise positive. Retaliatory D alone is not unprovoked
aggression. New relations initialize at `trust_initial`; after lost sovereignty, old trust
is discarded. Threat ties use `tie_break`. Coalitions form from two or more states
with the same negative-enough prime threat; membership may span disconnected territories.

Deterrence uses the coalition formed at the preceding period's end. `same_period` obligations
expand voluntary actions once: all victim coalition members that neighbor the aggressor play
D against it; newly obligated D does not recursively trigger more coalitions. `next_period`
records the obligation for the following decision phase. An intra-coalition initiator is
expelled before that expansion. Commitments are calculated after obligation expansion.
Coalitions do not pool stocks or transfer sovereignty. These timing rules are reconstruction.

Default snapshot period: update trust → choose provisional actions/initiation → expand
obligations → allocate → resolve dyads/damage/harvest → randomized structural claims →
rebuild fronts → form coalitions → record statistics. Sequential alternative uses one shuffled
capital list and recomputes that actor's actions and allocations against the current state,
resolving each unordered foreign dyad once at its first visit and applying its claims
immediately. Trust/coalitions remain period-boundary operations; primitive harvest happens
once at period end. Preserve the same number of actor opportunities; do not label Duffy's
one-conflict-per-iteration protocol as this update-order control.

### Provincial action and stochastic overextension

In two-level variants, conquered cells retain their own stock rather than automatically
transferring it to the center. The center receives each province's harvest tax; the province
retains the remainder. Apply tax even to negative harvest, except when the province has zero
stock, following p.124. The capital receives its own whole harvest. Ownership transfer preserves
provincial stock. On center collapse provinces retain stock, rather than equal splitting.

PRA covers the center's domestic and foreign fronts. A province compares its stock to the
center's commitment toward it; it initiates revolt at strict ratio above 2 and then uses
previous-action TFT. The center retaliates on domestic fronts. Foreign initiation is blocked
by domestic D or an unresolved revolt under the selected guard. Domestic victory by a
province grants independence; victory by the center ends the revolt without annexing an
extra cell. Disconnecting provinces preserves their own stocks. A conquered corporate capital transfers
its own stock if captured under the selected capital policy; a primitive sovereign captured
as a province retains its stock; it does not transfer the sum
of independent provincial stocks.

Two-level uses constant tax; overextension uses `tax_rate * tax_discount^distance` and
probabilistic initiation/victory `p(r)=1/(1+(r/threshold)^(-exponent))`, with p=0 for r≤0
and p=1 for positive infinity. Evaluate with portable log/exponential, without fused
multiply-add or native transcendental alternatives. Manhattan distance is the default;
territorial-path distance is shortest cardinal path within the state. The deterministic
foreign superiority predicate is replaced by this probability while guards remain. Domestic
revolt/victory uses threshold 2 in that formula; foreign encounters use `stochastic_threshold`.

`single_draw` samples attacker victory in an interval of length p(r), defender victory in
an adjacent interval of length p(reverse_ratio), and otherwise no victory. Compute the
reverse ratio from stocks with the same zero policy, rather than inverting 0/0. Probabilities sum
to at most 1 for thresholds >1. `independent_draws` samples both, with neither a stalemate
and a double success also a stalemate, counted separately. This competing-victory protocol
is unreported and must be exported. Overextension continues through one-state periods so
provincial revolt can end hegemony. The unknown-seed narrative run is illustrative evidence,
not a trajectory acceptance target.

## Clock, randomness, statistics and inspection

One economic period is one full source iteration. `periods_per_tick` repeats complete periods;
hegemony/horizon may shorten the final tick. Finished worlds are immutable. Exports include
tick, completed periods, actual last-tick period count, finish reason and invalidity context.
The page's million-tick cap is not a source horizon; the native survey enforces economic
periods. Overextension must not stop merely because one sovereign remains.

Use `SimRng` with the standard u64 seed. Reset samples resources in cell order and shuffles
placement using existing seeded conventions. Snapshot actions, paths and dyads are processed
in sorted IDs; structural claims alone receive the documented shuffle. Normal sampling is
variable-draw Marsaglia without caching its second normal. No claim of common random tapes
across treatments with different conditional paths: same seeds identify paired initial
conditions only where the implementation verifies identical initialization. The written plan
must enumerate and freeze the draw schedule before measurements; tests compare native/WASM
and grouped-period execution, not an unavailable author RNG.

Core series: period, sovereign count, the five terminal-category indicators, largest and
second-largest territory, predator capital share, total/capital/province stocks, nonpositive
stock count, positive destruction and negative-damage creation, attacks, DD encounters,
conquests, capital collapses, disconnections, revolts, coalitions and open episodes.
Tick-end state series remain distinct from summed tick events. Terminal category is unavailable
until completion, and remains unavailable for counts above100 on larger non-source grids. Hegemonic termination contributes count 1. Invalid sessions are retained
outside valid category denominators and reported prominently.

A conflict episode begins on first D on a foreign front; ends on a CC period, decisive victory,
sovereignty loss or border loss. Domestic episodes have a separate namespace. Duration counts
inclusive active periods, not a count of DD battles. Episodes open at a horizon are censored.
Record initiator, action pattern, cells/path, positive loss, outcome/end cause and involved
state sizes. Resource accounting separates harvest, taxes/transfers, destruction, signed
creation and numerical clipping. Tax/transfer ledgers cancel at system level.

Territory view colors sovereign ownership, outlines borders, marks capitals and distinguishes
latent predator/status-quo cells. Additional resource, strategy and coalition modes expose
mechanisms without treating coalition membership as sovereignty. Inspect shows cell/capital,
province stock, corporate stock, members/neighbors, trust/threat, coalition, domestic/foreign
front commitments/actions and last structural event. Charts show polarity and event/cost
history; Compare uses existing infrastructure. Raw sessions retain config, seed, endpoint,
categories, summaries, censoring and invalidity. Event logs are opt-in and bounded on the page;
the survey can retain complete requested traces without storing every frame by default.

## Registered studies and decision rules

Publish a machine-readable manifest and source-figure JSON before any study. Reused book
figures are cross-checks of the original results, not independent datasets. Extract original
Figs.10/11/13 at the eight stated abscissae, plus book Figs.5.6/5.8. Render at ≥300 dpi; record
image hash, axis calibration, coordinates and intervals. For stacked categories store all
integer count vectors consistent with boundary uncertainty and total 20. Exact textual
5/9/5/0/1 overrides image uncertainty at its known cell. Fig.5.8's extraction freezes its
recoverable tax knots; do not assume the old notes' guessed list. Unrecoverable knots/bands
are marked unavailable, not interpolated into invented observations.

| Study | Design and evidence status |
|---|---|
| Original | 8 predator shares × ratios2/3 × alliances off/on ×20 seeds; ≤1000 periods/hegemony. First640 valid-or-invalid sessions are a literal-size reconstruction sample |
| Precision extension | Same32 strata, 200 additional seeds per stratum. Our precision extension; original runs are reported separately |
| Radax persistence comparison | Original no-alliance16 strata ×20 seeds, previous-action vs persistent memory. Recovered design, without available exact output/source code; reuse matching baseline sessions |
| PRA | Book chapter5 damage convention; 8 shares ×2 ratios ×20 seeds, no alliances, Fig.5.6 targets |
| Allocation/alliance interaction | 2 allocations ×2 alliance states ×8 shares ×2 ratios ×200 seeds; use chapter5 damage consistently across all four arms. Report category-wise difference of alliance contrasts |
| Two-level | Extracted Fig.5.8 tax knots ×20 seeds, all predators, ratios2, PRA, ≤1000 periods; plus 200-seed extension at tax0,.1,.2,.4,1 |
| Overextension | 20 seeds ×4000 periods, all predators, no alliances; continuation/collapse/episode results descriptive, with no unknown-seed trajectory fit |
| Ambiguity families | Original16 no-alliance strata ×20 seeds, change one dimension from baseline: guard, persistence, victory timing, stock policy, capital policy, stock transfer, lock scope, topology and update order. Alliance 16 strata additionally test trust initial values −1000/0/1000, threat thresholds −100/0/100 and obligation timing; retain baseline references |
| Störmer-inspired sensitivity | 110 configurations ×10 seeds,1000 periods, her text's discrete parameter prior restricted to original EPM fields. An adaptation, not her Python replication; record discarded/nonapplicable dimensions and distributions |

The Störmer-inspired study uses initial mean I and harvest mean H uniform integers5..200,
SDs uniform integers0..I and0..H, winning/attack ratios uniform{1.1,1.2,…,5}, predator percentage
uniform integers0..100, and damage fraction uniform integers1..100 divided by100. Alliances
are enabled with the frozen EPM trust reconstruction. Keep original normal distributions,
cardinal adjacency and strict superior attack, rather than importing the listing's reversed
inequality, Moore neighbors, uniform draws or sampled-threshold bugs. Aggregate observations
as configuration means across ten runs. Report exact zero structural-event fraction and
near-corner scatter; no recovered exact “75% inert” gate exists. Literal execution of
Störmer's listing, its 600/1000 discrepancy and Radax's external code are outside this milestone.

All survey seeds are recorded; assign nonoverlapping deterministic ranges by manifest arm.
Paired controls may reuse a range only when labeled paired. Do not treat ten repeats within
one sampled configuration as 1100 independent configurations. Never increase sample count
selectively because a desired claim failed; follow the fixed precision extensions.

Decision rules:

1. **Mechanics:** exact assertions on prescribed motifs, conservation and category definitions.
   These are implementation checks, separately listed from empirical findings.
2. **Figure compatibility:** report digitization intervals, reconstruction means/distributions,
   category total-variation distances and source-size variability. From each precision-extension
   stratum, bootstrap 100,000 size 20 batches using a separate recorded analysis RNG. Compare
   observed source mean with the central 95% interval of batch means; compare category vectors
   by multinomial deviance against reconstruction probabilities. Generate 100,000 multinomial
   size 20 reference samples for that deviance using Jeffreys-smoothed probabilities
   `(count+.5)/(N+2.5)`. Means use unsmoothed whole-session bootstrap batches. The predictive
   check conditions on the estimated reconstruction distribution; its finite-sample uncertainty
   is shown separately, not claimed to be absent. Apply Holm correction at .05 separately
   to original means and original category strata; PRA/tax each form separate families. For these figure checks,
   mean p-values use twice the smaller inclusive bootstrap tail, category p-values use the
   deviance upper tail; both use add-one Monte Carlo counts and are capped at 1. With uncertain
   digitization, use the largest p among admissible targets for Holm, and reject only if every
   target rejects. Label results compatible/incompatible/
   unresolved. Compatibility is not equivalence or proof of identical implementation.
3. **Directional claims:** in each stratum, report defense-minus-offense and alliance-minus-none
   contrasts in mean polarity, hegemony and 2–10 probability. Compute 95% bootstrap intervals
   from whole sessions (100,000 resamples; separate analysis RNG). A positive claim Holds
   when its interval is entirely above zero and corrected p<.05, Fails when entirely below
   zero and corrected p<.05, otherwise Inconclusive. Calculate the two-sided percentile
   bootstrap p as twice the smaller resampled probability of a contrast at or below / at or
   above zero, capped at 1, using add-one Monte Carlo counts. Apply Holm across all strata and
   the three metrics within each defense/alliance family, including the registered aggregate.
   Pool uniformly across the seven nonzero predator settings only as a
   labeled registered aggregate; never hide a conflicting stratum behind that aggregate.
4. **Allocation dependence:** category-wise difference of alliance contrasts under PRA versus
   equal allocation, bootstrap whole sessions within strata, Holm correction across categories
   and strata. A corrected exclusion of zero supports dependence; no reversal or minimum
   effect is assumed. No detected difference is Inconclusive, not proof of independence.
5. **Selection:** compare terminal predator-capital fraction to initial share for mixed-type
   settings. Use session-level differences and the same registered bootstrap/Holm procedure.
   Pure-type settings cannot test selection. Report survivors by capital type, not all cells.
6. **Unquantified claims:** general frequent power politics, exponential takeoff, representative
   overextension, Störmer's≈75% near-corner and Duffy's effect sign are descriptive or Untestable
   without a recovered numeric definition. No post-hoc curvature/inertness threshold is added.

Keep two distinct rows for the initial stabilization hypotheses and the author's reported
counterexamples. P2/P3 hypothesize a positive defense/alliance contrast in 2–10 probability;
the reported figures can contradict those hypotheses. Agreement with a reported negative
contrast is a successful reproduction of that counterexample, not a failed reproduction.
Evaluate source direction at each recoverable plotted setting; informal “low/high density”
wording supplies no extra cutoff. The mean-polarity and hegemony contrasts remain separate
metrics, since raising survival need not reduce hegemony.

Invalid sessions make a stratum unresolved for source compatibility. Report the fraction,
reason and affected policy; do not rerun replacement seeds to conceal invalidity. Conclusions
must identify baseline readings and sensitivities; “robust” is used only when the registered
alternatives agree, with their sample-size limits stated. The findings matrix carries source
citation, target/protocol, judge, observed result, uncertainty, verdict and limitations.

## Verification and delivery

Test prescribed worlds before implementing each mechanic: no-predator peace; TFT echo and
predator reinitiation to DD; threshold equality; two-stage path support; neighbor-count
allocation; Fig.8 ledger; simultaneous damage; victory timing; capital-only and corporate
capture; disconnected enclaves; locking/stale claims; trust recurrence; coalition deterrence,
obligations and expulsion; PRA equations and wrong worked-example annotation; domestic revolt,
negative-harvest taxes; distance decay; probabilistic endpoints and resolution; censored
conflict episodes. Use explicit draws or injected RNG at mechanic boundaries.

Test grouped periods against repeated single periods, immutable finish, hegemony versus
provincial continuation, schema/preset/export round trips, category partition, determinism,
Inspect/render read-only behavior and native/WASM parity. CI contains small motifs and short
runs only. Retain full survey sessions, analysis manifests, source hashes and reproducible
commands. Use existing format/lint/build checks, workspace tests and browser smoke checks.

The implementation plan must have 3–5 stages in `IMPLEMENTATION_PLAN.md`, with goals, success
criteria, tests and statuses. Its verified scratch changes must reproduce byte for byte in
an isolated execution worktree. Execute through subagents as already requested; do not ask
again for execution method. Final independent review examines source fidelity, judges,
negative stocks, collapse/accounting and all hosts before integration. Completion includes
findings docs, queue/index update, war-study handoff, CI and Pages at the final integrated
head. Remove the stage ledger when all stages are complete; retain the durable plan and
research artifacts. No source-fidelity claim exceeds the recovered evidence.

## Implementation clarifications before measurement (2026-10-02)

During approved scratch preparation, variant-aware JSON defaults were made explicit: missing
fields use the selected variant defaults; explicitly incompatible fields are rejected. Event
log caps/IDs/drop counts, source categories above100, provincial primitive-versus-corporate
capital accounting, PRA coalition support and the sequential/after-harvest validation rule
were clarified above. No native studies were run before these additions. They resolve
implementation gaps; source claims and statistical decision thresholds are unchanged.
