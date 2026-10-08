# Deduction and surface experiments in the browser

**Date:** 2026-10-08
**Status:** approved October 8, 2026 (LGTM!). Written implementation plan awaits separate review.
**Execution:** subagent-driven implementation with independent reviews after
written-spec and implementation-plan approval.

## Goal and scope

Expose the completed deduction and surface campaigns as visual experiments the
user can run, pause, step through, and compare in SugarScape's web interface.
Preserve their existing rules, supplied knowledge, policies, prices, outcomes,
and retained scientific evidence.

The user requested this backfill before further raw experiment discovery. The
[inventory](2026-10-08-campaign-visualization-inventory.md) records the other missing
families. This first batch is proposed because deduction and surfaces are the
campaigns from the current conversation; spatial and caching viewers receive
separate subsequent designs.

Include the following study selections:

1. Deduction capabilities/Wink.
2. Noisy testimony channel and testimony decision game.
3. Strategic reporting against frozen listeners.
4. Strategy-aware listener inference.
5. Exact adversarial reporting audit.
6. Shared surface learning.
7. Active surface experiments.

These are several experiments in three rendering families, not seven new
simulation engines. There is no new scientific campaign, retraining, raw probe
selection, gate, language learning, or policy change in this increment.

## Browser navigation and controls

Inside Experiments, offer a visible choice between existing sweeps and episode
experiments. Keep ordinary Playground and sweep behavior available. An episode
catalog lists the included studies with a short explanation of what is supplied,
what the Agent decides, and what the user can observe.

Each study starts with a bounded documented example and appropriate controls.
Use seeds for stochastic games and explicit case/history/bit-sequence selection
for finite enumerated studies. Do not invent seed semantics for an exact case.
Offer existing declared policy and mechanism controls only; label a user-chosen
example as exploratory rather than a new measured campaign outcome.

Run produces a checked bounded episode. Play, pause, step, rewind, reset and a
timeline then navigate that episode. Rewinding affects display only and consumes
no new randomness. Changing inputs marks the current output as belonging to its
original inputs until a replacement run succeeds. A canceled or failed run never
appears as a newly completed episode.

Support exporting the bounded episode and sharing its versioned input selection.
An input link opens the selection without automatically starting expensive work.
Malformed or incompatible input/export files report a contextual error and leave
the last successful display available. This first batch does not accept arbitrary
full diagnostic reports as episode files.

## Rendering families

### Deduction game

Show Agents in a compact interaction diagram with active/inactive status and
watching, capability use, reports, accusations, and consequences along a phase
timeline. Respect delayed effects and the distinction between submitted actions
and committed phase outcomes. Inspect an Agent's actual TurnRequest observations,
retained memory, legal opportunities, and accusation budget at that checkpoint.

Offer existing built-in policies and documented seed examples. Distinguish the
ordinary CLI run's policy assignment from the diagnostic's separately fixed
threat controller. Label the selected mode so an ordinary run is not silently
presented as a diagnostic replication. Wink's Evidence controller uses direct
sightings; do not display an invented Bayesian belief for it.

### Testimony, reporting, and listener decisions

Show the ordered information flow: reporter signals, calibration reports, live
report, listener evidence, belief and accusation/abstention decision, and resulting
payoff. Reporter private state is available only in Researcher view. The listener
panel displays only its actual allowed observations and model outputs.

Preserve each study's mathematical meaning. Noisy-channel testimony uses its
existing finite posterior; strategic reporting evaluates a supplied or retained
policy against its chosen listener; strategy-aware inference displays its joint
policy/truth conditioning and catalog weights; adversarial audit compares its
existing listener controls and canonical attack witnesses. Unknown actual policy
or private reporter state never enters the listener API.

Use exact case details and existing decision functions. A diagram's stages must
not imply there was an evolving game or sequential belief update where the study
only evaluates a static complete history. If intermediate conditioning is not
exposed by an existing model, display the observation stages and the model's
available checked result, rather than fabricate intermediate probabilities.

Show the retained GA/random-search comparison in a separate recorded-results
panel. Include all retained seeds and declared comparisons with their source
identity; do not rerun training in the browser or select only favorable examples.
Single selected-policy demonstrations remain separate from aggregate results.

### Shared and active surfaces

Show Agent A, Agent B, the surface fields and an ordered event strip. Visualize
reads, writes, accepted writes, waits, round/phase resets, private bit issuance,
public stopping, inspection, prediction and failure at their actual checkpoints.
The spatial arrangement is explanatory: it does not add distance or travel costs.

For shared surfaces show the supplied calibration schedule, beliefs and declared
controller. For active surfaces show paid round purchases, chosen live routines,
alternative continuation values and spent/remaining credits. An accepted write
must not visually imply that another Agent received it. Distinguish shared,
private, resetting and inert behavior through actual episode state.

Allow matched policy comparison using the same mechanism, role and bit sequence.
Unsupported history ends at its recorded checkpoint, retaining costs and partial
observations; missing terminal accuracy/net is shown as unavailable. Supported
wrong predictions remain completed predictions, not engineering failures.

## Information boundaries and visual design

Default to an explicitly selected Agent's perspective. A labeled Researcher view
reveals privileged truth, other Agents' private state and write lineage where the
study exports them. Perspective selection is purely presentational; it cannot
alter the episode, inference or controller inputs.

Filter display data to the chosen checkpoint. Future private bits, target truth,
later observations and terminal outcomes do not leak into an earlier Agent view.
Public status can remain visible where the experiment makes it public. Do not
invent model probabilities or knowledge about other Agents.

Use the existing application typography, layout rhythm and controls. The central
visual is the current interaction and its chronological history; beliefs and
costs sit beside the selected Agent. Stable Agent colors and labeled action
symbols support inspection without making color the sole source of meaning.
Avoid presenting every value as a separate decorative card.

Provide keyboard stepping, visible focus, reduced-motion behavior, screen-reader
descriptions for the current event, and a usable narrow-screen layout. Layout
mockups can be reviewed once this scope is agreed; a running visual companion is
not needed for the textual scope discussion.

## Core, WASM, worker, and data contracts

Implement additive bounded episode adapters around existing public core APIs.
Rust supplies policies, transitions, exact inference, outcomes and checked records.
TypeScript selects inputs, renders records, and navigates the timeline. Do not
duplicate world rules, posterior calculations or decisions in JavaScript.

The adapter envelope identifies study and adapter version, normalized input,
source/rules identity, ordered checkpoints, local observations, available model
outputs and optional privileged display fields. Renderer families retain their
study-specific payloads; do not flatten every study into one universal event
schema that loses its clock or observation semantics.

Keep seeds, fingerprints and arbitrary u64 identifiers as decimal/hex strings
across JavaScript. Preserve exact rational values in the transport; approximate
display formatting never feeds a controller or comparison calculation.

Use a dedicated worker for run/validation work. Requests identify their run so
stale results cannot replace a later selection. Cancel terminates the worker and
releases compiled-policy memory. Expensive active-surface compilation happens
off the page thread; record performance on a bounded browser example before
promising interactive response time. Cache by complete normalized input and
version, with bounded lifetime and size.

For this batch, accept at most 64 KiB of normalized input and 4,096 display
checkpoints, and cap a complete episode export at 16 MiB of compact JSON. One
worker runs one requested episode at a time, with one retained compiled engine
and a 60-second wall-clock timeout that terminates an incomplete request. Keep
at most two successful episode records for matched comparison; release compiled
engines and old records when replaced. These are proposed browser resource
limits, not scientific settings. Existing stricter core bounds remain in force.
Measure cold compilation and episode generation during the feasibility gate;
if documented examples exceed these limits, disclose and review a limit change
before implementation acceptance. Do not truncate a record and call it complete,
or execute the full scientific diagnostic merely to render one episode.

Compact recorded-results assets may be derived from retained first reports using
a bounded extractor with explicit input hashes and transformation receipts. They
must include the complete declared aggregate comparisons. Never package ignored
raw evidence, a 250 MB surface report or a multi-GB shared report into the web
bundle. New adapter identities are distinct from the original frozen measurement
source manifest; original artifacts and their manifests are never rewritten.

## Verification and acceptance

Before implementation, the staged plan must identify exact per-study inputs,
record contracts and limits, alongside representative existing patterns. Each
adapter has native/WASM parity checks on full bounded records, including a normal
case, a wrong prediction/decision where applicable, and an unsupported/invalid
case. Existing frozen study namespaces and measured settings stay unchanged.

Compare bounded demonstrations with retained diagnostic/replay references.
Recorded summary assets bind to their source report hashes. Additions to shared
exports or CLI boundaries are reviewed as additive; do not claim the full old
source manifest still matches after adding new WASM APIs.

Web tests cover selection, ordering, phase resets, perspectives, unavailable
outcomes, matched inputs, cancellation, stale results, exact integer transport,
sharing and export validation. Use actual WASM in parity checks. Perform browser
verification of every rendering family, keyboard operation, reduced motion and
narrow-screen behavior; a green unit suite alone is not visual acceptance.

Success means all included studies are discoverable and a user can run and step
through their documented bounded examples. Agent views agree with actual local
observations; privileged display controls never change behavior. Existing ordinary
simulations/sweeps continue to work. No unimplemented catalog entry appears as a
working experiment.

Use subagent-driven implementation and independent task/whole-branch reviews.
Run relevant Rust/WASM/web checks and the required repository quality gates.
Preserve first reports, settings, evidence and unrelated work. Commit incremental
working stages, merge the finished task into main, push normally, and verify CI
and applicable Pages deployment for the exact delivered commit.

## Subsequent batches and approval

Burrow/CPFA viewers and P3/P4 caching viewers follow as separately scoped batches
using the same browser shell. Their existing WASM exports and core snapshots are
starting points, not claims that a viewer already exists. Active concurrent F5
work is excluded until its producer delivers a stable reviewed interface.

This design is approved. The written implementation plan now awaits review;
runtime/UI implementation starts after that review. Raw experiment discovery remains deferred and its draft is preserved.
