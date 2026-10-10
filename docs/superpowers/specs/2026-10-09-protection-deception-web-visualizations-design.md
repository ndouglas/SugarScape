# Protection and deception web visualizations — Batch C

**Date:** 2026-10-09
**Status:** conversational design approved by user LGTM; written spec awaiting review. No implementation plan or runtime changes are authorized by this artifact yet.
**Builds on:** the [visualization inventory](2026-10-08-campaign-visualization-inventory.md), [Batch B design](2026-10-08-spatial-campaign-web-visualizations-design.md), and [episode viewer](../../experiment-viewer.md).

## Intended outcome

Finish the recent campaign visualization backfill by making P3 re-caching and P4 supplied caching gestures playable in Experiments → Episodes. Readers should be able to follow physical actions, private evidence, costs, and outcomes while distinguishing what a selected Agent recorded from what a researcher can inspect.

Batch A and Batch B are delivered. This increment adds two viewers to their shared interface; it does not rerun a scientific campaign or change its treatment definitions, controllers, schedules, random streams, original exports, measured reports, or frozen settings. The existing Burrow observation-capture limitation remains a separate instrumentation proposal. Surface discovery and the future gate/key/message-board integration remain separate work.

## Approaches and selected design

| Approach | Benefit | Tradeoff |
| --- | --- | --- |
| Extend the existing episode shell — selected | Reuses playback, spatial rendering, perspectives, sharing, validation, and retention | Requires explicit projections of research DTOs into Agent views |
| Add dedicated campaign pages | More freedom for campaign-specific presentation | Duplicates lifecycle and playback infrastructure |
| Expand native sensory capture first | Could support richer sensory maps | Adds instrumentation beyond the records needed for this backfill |

Use additive adapters around the existing native runners and extend the current spatial presentation. TypeScript renders projected records; it does not implement controller decisions, perception, cache transactions, or relocation rules.

## Entries, controls, and examples

Add proposed study IDs `protection_recaching` and `deception_gestures` to the common catalog, preserving the existing thirteen entries and their links.

P3 exposes the existing LabConfig fields: policy (`off`, `selective`, `indiscriminate`, `erased`), fixture, mirror, reburial cost, discovery probability, exposure span, and observer span. Supported fixtures remain the native single, mixed, visible-nonwatcher, unseen-watcher, and stumble variants with their original fields. P4 exposes sender (`ordinary`, `matched_neutral`, `sham`), view (`ambiguous`, `clear`), display visibility, layout (`on_route`, `off_route`), effort cost, and mirror. P4 effort cost remains exactly 0 or 3. All other domains and cross-field constraints delegate to the existing checked rig constructors and Config validation.

Both use decimal-string seeds and the native fixed 64-tick requested schedule. There is no horizon or sampling control: retain the initial frame and every native completed-step frame through termination. Playback speed changes presentation only. Fixed 9×9 arenas and two initial Agents remain part of the native rigs; arbitrary map painting, extra roles, new policies, and ordinary-world feature knobs are outside scope.

Provide named explanatory examples drawn from existing checked fixtures: P3 observed/unobserved caches, selective versus indiscriminate or erased exposure memory, and a cue mismatch; P4 sham versus matched-neutral behavior, ambiguous versus clear evidence, and a seen versus unseen display. Record each preset's exact inputs and fixture provenance before examining new viewer outputs. Label presets as explanatory single episodes. They neither replace the registered campaign nor establish aggregate effects.

Opened custom input must retain its actual fields and validation errors. Complete lab JSON may be inspected/edited using the existing controls pattern, but invalid text must block Run and Share rather than submit a stale valid draft. Editing or failed replacement preserves the last successful record and its descriptor.

## Native records and checkpoint authority

P3 uses `minds::protection::runner::run_episode` with its original ledger-enabled research record. P4 uses `minds::deception::run_episode` with its original diagnostics/ledger-enabled record. Construct and execute one native episode per request; project its retained frames without a second simulation. Preserve the existing `protection_config_json`, `protection_episode_json`, `deception_config_json`, and `deception_episode_json` exports and their original numeric JSON contracts.

The new common envelope carries versioned input, source/rules identity, original frame clock/fingerprint, projected Agent state, separate researcher state, and the original episode payload under the existing complete-result gate. Compare that payload against the original runner, applying only the established lossless wire conversion of integer atoms. Do not alter floating values or rewrite original records to suit the display.

Frames represent native boundaries. Initial state precedes actions; subsequent frames contain state after the native step and that step's diagnostic buffers. The adapter must establish and test the association between frame tick, action interval, and recorded observation tick. Show original clocks explicitly when they differ. Never move a future observation into an earlier checkpoint, imply simultaneous actions, interpolate unrecorded paths, or draw randomness during replay. Agent death/absence and early termination remain native outcomes, not fabricated zero-valued state.

## Visibility and capture contract

All existing episode DTOs are researcher records. Selection by actor ID alone is insufficient: fields must be explicitly allowlisted for each perspective.

| Data | Selected Agent projection | Researcher projection |
| --- | --- | --- |
| Position and holdings | Selected role's captured values | All captured roles |
| Cache ownership | Selected role's own cache entries | All role caches and available physical stocks |
| Seen-cache evidence | Selected role's retained `seen` entries, with recorded tick/age | All retained evidence, separately labeled |
| P3 exposure and relocation | Selected role's ProtectionState, including sources, perceived exposure memory, and intent | All captured state, relocation diagnostics, and actual watchers |
| P4 sender state | Selected role's own SenderState when present | All captured sender state |
| P4 received gesture | `Observation` addressed to the selected receiver | Observation plus actual transfer and actual stock |
| Actions and choices | Own committed actions and supported own decision evidence through this boundary | Complete native action/choice diagnostics |
| Global outcomes | No terminal or global diagnostics in an early Agent checkpoint | Ledger, restrictions, deaths, transfers, fixture errors, and terminal results |

P3 actual-watchers lists and P4 `ObservedRecord.actual_transfer`/`actual_stock` stay researcher-only. P4 clear-view signals may legitimately contain the visible transfer amount; ambiguous signals contain the nominal cue, not hidden physical transfer. Choice fields such as `actual_value`, inspected stock, target occupant, and recovery diagnostics must remain researcher-only unless a specific native controller/sensor boundary proves them available to that Agent at that time. The implementation plan must contain a field-by-field allowlist with source provenance and tests; unsupported fields remain absent with an explicit availability explanation. A research diagnostic does not become Agent knowledge merely because it describes that Agent's action.

Draw the fixed arena as a labeled **reference layout**, with the selected Agent's actual captured position, own caches, and remembered cache evidence. This is setup scaffolding, not a learned topology map or reconstructed current visual field. Do not place peer markers, peer caches, actual global stocks, or private protocol source/display markers into an Agent map. Public reference geometry must be separately justified and must not expose randomized/private contents from the input.

Memory stays memory: show the original evidence tick and elapsed age at the current boundary, even when its amount differs from physical stock. P4 receivers use bounded seen-cache evidence rather than map beliefs. P3 exposure memory is perceived exposure and can disagree with actual watching. Unknown, absent, expired, and unrecorded data must have distinct wording where the native record supports that distinction.

Renderers receive only a projected checkpoint and safe descriptor. Full inputs, future frames, other Agents' private state, and terminal results never decorate the Agent perspective. Switching to Researcher is an explicit presentational disclosure; this interface is not an access-control system protecting the exported episode from its owner.

## Presentation and comparison

Reuse the existing map, keyboard playback, stepping, fit/zoom, responsive inspector, focus treatment, reduced-motion behavior, and light/dark styles. Symbols and text must carry distinctions independently of color.

P3 emphasizes cache retrieval/redeposit, perceived exposure, relocation intent/stage, action costs, and researcher-visible actual watching. P4 emphasizes the received gesture, retained remembered value, observer response, actual transfer/stock in Researcher perspective, and effort/outcome. Describe gestures as supplied experimental behavior; no learned deception, intention, or theory-of-mind claim follows from this viewer.

Allow the shell's two-record comparison for compatible source identities and inputs differing only in explicitly declared treatment fields. Preserve seed, fixed schedule, fixture/layout geometry, and all unselected settings. Display input differences and match actual native boundary clocks; mark missing checkpoints after death/termination. Same seed does not promise paired random draws after differing decisions. Single-episode differences remain illustrative and link to the existing campaign findings for measured conclusions.

## Bounds, transport, validation, and lifecycle

Retain the common 64 KiB input/decompressed-link limit, 16 MiB complete-record limit, 4,096 checkpoint limit, one worker request, 60-second timeout/cancellation, and two successful retained records. The fixed native schedule normally yields at most 65 frames. Bound role counts and retained field shapes against the fixed rigs and native memory caps before projection/retention; cap serialized output incrementally where needed. A valid native request exceeding this browser profile is rejected contextually without silently changing its scientific inputs.

Unbounded u64 seeds, IDs, and clock fields use the established canonical decimal-string transport. Validated bounded coordinates/site indexes may use ordinary numeric fields. Keep all floating outputs unchanged and finite checks delegated to native validation. Imports are size-checked before parsing and freshly reconstructed under compatible versions/source identities; matching metadata or an imported success flag is insufficient. No new floating tolerance is authorized, and Batch A's named testimony tolerance does not extend here.

Cancellation, timeout, invalid input/import/link, or failed episode replacement retains the prior successful record. Navigation pauses playback; disposal clears workers, retained records, projections, comparison references, and pending editors using the existing shell's lifecycle. Failed native episodes surface contextual errors, including recorded fixture/ledger failure where applicable; do not present a failure as a successfully verified example.

## Preservation and verification

Bind source hashes and original acceptance fixtures before implementation. Preserve original scientific reports, first/repeat outputs, frozen settings, original WASM exports, and product-guide content outside scoped additions. Disclose any separately authorized revision after seeing results. Source-identity additions must follow the common identity contract, with workspace integrity guards kept consistent; never silently rebind old scientific records to new source.

The implementation plan must provide behavioral tests for:

- Original-runner payload equality, every native frame fingerprint, original-export preservation, and ledger/fixture validation on supported P3/P4 examples.
- Native/WASM parity, including u64-max seeds, mirrors, cue mismatch, erased exposure memory, ambiguous/clear signals, seen/unseen displays, and paid/zero-cost gestures.
- No peer, actual-watcher, hidden-stock/transfer, future, or terminal leakage into Agent projections; legitimate clear-view signals remain visible.
- Initial/post-step timing, stale evidence ages, own-state availability, death/termination, perspective switches, and compatible/incompatible comparisons.
- Bounds, lossless integer transport, fresh import reconstruction, malformed/edited JSON, cancellation, timeout, retention, and disposal.
- Actual browser operation of both entries and existing-entry regressions, keyboard/narrow/reduced-motion use, and evidence binding the tested UI/WASM to the reviewed source.

Run required formatting/lint/build/test gates on the final source and independently review tasks and the whole increment. Keep engineering viewer acceptance separate from scientific campaign execution. No new campaign collection or bundling of raw scientific evidence belongs to this scope.

## Delivery and review boundary

Execute the approved implementation plan through fresh implementer agents with independent reviews in the existing crowd worktree. Commit working increments, integrate completed reviewed work safely into main, push normally, and check CI/Pages for the exact delivered commit under the user's standing authorization. Inspect concurrent main/worktree changes before integration; preserve unrelated work and all retained evidence.

Never stage `.claude/`, `papers/`, `survey/out/`, ignored evidence, or the deferred untracked surface-discovery draft. Rustfmt only edited Rust files with child traversal disabled; do not format survey files. Use American spelling and Agent in code/docs.

This conversationally approved design is now presented as a written spec. Written-spec approval permits preparing the implementation plan; implementation begins after plan review under the brainstorming workflow.
