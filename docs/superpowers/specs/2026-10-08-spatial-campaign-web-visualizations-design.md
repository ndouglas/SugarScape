# Spatial campaign web visualizations — Batch B

**Date:** 2026-10-08
**Status:** written spec approved by user LGTM on 2026-10-08; implementation plan under review, runtime work not yet approved.
**Builds on:** the [visualization inventory](2026-10-08-campaign-visualization-inventory.md), [episode viewer](../../experiment-viewer.md), [Burrow labs](../../burrow.md), and [standalone foraging labs](../../foraging.md).

## Intended outcome

The user wants to see recent labs operate in the web interface before resuming surface experiment discovery. Batch A delivered eight deduction/testimony/surface entries at main `74feeed12216262f2ffefca90487b8f665f5c715`, with exact-commit CI and Pages verification. Batch B adds spatial playback to that same Episodes interface.

A successful viewer lets a reader observe excavation, movement, material handling, information use, and paid work; distinguish an Agent's recorded state or memory from physical researcher state; and reproduce a bounded selected example through the existing Rust implementation. Engineering acceptance examples do not establish biological fidelity, coordination, learning, optimality, or scientific treatment effectiveness.

This is architectural integration of five lab adapters with one shared spatial renderer. Existing engines, controller parameters, schedules, random streams, original exports, reports, and settings remain unchanged. P3/P4 caching stays Batch C. F5's integrated native archive/comparison engineering and its unapproved scientific campaign remain separate; F5 is not a sixth entry here. Raw surface experiment discovery remains deferred.

## Approaches and recommendation

| Approach | Benefit | Cost and scope |
| --- | --- | --- |
| Extend the checked episode shell with additive lab adapters and a spatial renderer — recommended | Reuses cancellation, playback, perspectives, validation, input links, exports, and two-record retention | Requires an explicit spatial checkpoint contract and CPFA bridges |
| Add separate lab pages around the existing Burrow exports | Direct route to the existing Burrow demonstrations | Duplicates navigation/lifecycle and still needs new CPFA bridges |
| Unify the lab worlds with the ordinary simulation Model interface | A single broad model framework | Restructures existing engines and expands this presentation increment substantially |

Use the first approach. TypeScript displays Rust-produced projections and navigates retained records; it does not implement terrain transactions, routing, inference, material accounting, or controllers.

## Scope and catalog

Add five independently selectable entries, each with a working bounded default and named acceptance examples:

| Proposed study ID | Entry | Authoritative implementation |
| --- | --- | --- |
| `burrow_excavation` | Burrow excavation | `burrow::run_episode`, existing ordinary replay/validation |
| `burrow_access` | Burrow resource access | `burrow::run_access_episode` |
| `foraging_fixed` | CPFA F2 fixed world | `foraging::fixed::World` and checked runner |
| `foraging_passage` | CPFA F3 passages | `foraging::passage::World` and checked runner |
| `foraging_construction` | CPFA F4 construction | `foraging::construction::World` and checked runner |

These are engineering labs. Burrow's archive/protocol implementations and CPFA F2–F4 do not become registered scientific campaigns by receiving web controls. No complete survey execution, aggregate efficacy claim, or campaign collection belongs to this batch.

Defaults derive from the checked examples and tests already in the repository. Burrow exposes the direct/relay and blind/responsive examples and Explore/KnownGoal access examples. CPFA exposes compact fixed-world, passage, and construction acceptance setups, including an inaccessible/censored case and compatible no-dig case. Preserve each example's original supplied values; choose readable browser horizons and sampling explicitly, label them as display examples, and record their provenance before judging outputs.

Controls select a named setup, decimal-string seed, requested ticks, and sampling interval. Burrow additionally exposes its original declared transport/cue/objective choices where valid. CPFA exposes the family's original parameters through named numeric controls, retaining their actual field names and domains. Arbitrary map painting, new controllers, new roles, relay rules for CPFA, and a setup-builder editor are excluded. Opened compatible custom inputs remain inspectable and validated on Run; missing or invalid selections are never replaced silently by defaults.

## Existing interface facts and display limits

The implementation has three different evidence contracts:

- Burrow exports complete action/choice history, sampled ASCII frames, aggregate snapshots, material histories and final diagnostics. Its public World does not expose controller observation snapshots or a full worker-state getter. The ASCII projection can hide spoil and represent two workers with one glyph.
- F2 snapshots expose physical resources/advice plus individual positions, phases, targets, cargo, retained finds and work. A private current food-observation history is not exported.
- F3/F4 provide read-only physical snapshots and `World::knowledge(agent)` for private topology classifications. Those maps do not retain occupancy/food histories or observation timestamps. A remembered solid wall can remain stale after another worker digs it.

The viewer must honor these interfaces. Full controller observations, per-action CPFA logs, observation ages, private food memories, and exact Burrow worker multiplicities/holdings at a sampled frame are unavailable unless already exported. Do not derive fictitious observations from researcher snapshots or modify an engine merely to fill an inspector.

## Checkpoints and native authority

Extend the common catalog with family `spatial` and additive input/study variants. Preserve the existing versioned envelope, eight Batch A entries and old links. A spatial checkpoint carries the original lab clock, explicit sampling/initial/terminal stage, public protocol metadata, local Agent projections where supported, and separately labeled researcher data. Full original episode output and final quantities remain in the existing gated result pane.

Burrow uses its original sampled frames, matching snapshots and committed event prefixes. Decode its exported ASCII cells in the Rust adapter for a readable map, retaining the original frame fingerprint and overlay/multiplicity caveat. Keep native frame order: a choice fixture may have initial and terminal frames at the same tick and stop after a choice before an action. Tick equality does not collapse those distinct checkpoints. Access milestones and private completion observations use the native opportunity clocks; initial configuration alone does not grant Explore workers the goal coordinate.

For CPFA, the adapter constructs one World and calls its original atomic `step` through the entire requested horizon. At actual initial/sample/final boundaries it captures read-only `snapshot`, `summary`, and, for F3/F4, `knowledge` for the bounded Agent set. Reproduce the original runner's initial/final sample rules and normalized setup. Verify complete native payload equality with the original runner in tests; do not run two worlds to produce one browser episode.

CPFA frames are completed-tick boundaries, not individual opportunities. All workers commit in ascending-ID order within a tick. A sampled interval can contain several actions and moves. Display interval work-count changes as intervals, never invented individual actions, exact paths, interpolated motion, or simultaneous schedules. The initial boundary need not represent a just-delivered controller observation.

Run and replay remain separate. Playback reads retained checkpoints and never steps a world or draws randomness. Read-only capture must leave native counters, lazy server records, milestones, fingerprints where available, and RNG continuation unchanged. A failing run yields a contextual error and retains the previous successful record; CPFA's existing runner does not promise a successful partial Episode.

## Agent and Researcher perspectives

Default to the selected Agent perspective and show the actual availability of its data:

- Burrow: own committed action/choice prefix and native private goal-completion markers where exported. Identify event origin/clock as historical. Controller observations, retained target state, current holdings and full local map remain explicitly unavailable. The physical ASCII map appears only in Researcher perspective; an unavailable local map is an honest result rather than an empty learned map.
- F2: selected Agent's own exported position, phase, heading/target, cargo, retained find, and work. The arena's supplied dimensions/nest are setup scaffolding. Global food locations and all server advice remain researcher data; local food sensing is unavailable.
- F3/F4: selected Agent's own exported state and private known-cell map captured at that boundary. Unknown cells stay unknown. Memory is labeled as memory; no fresh physical wall classification or invented observed-at timestamp replaces a stale belief. Global food, peer positions, access caches and retained server advice remain researcher data.

Researcher perspective adds current physical terrain, all exported Agents, resources/material, advice and structural-access diagnostics. It can show selected private memory beside physical state with clear labels. Every renderer receives only the projected checkpoint and safe descriptor; input setup, future snapshots, terminal milestones and full episode payload do not decorate early Agent views. The existing separately labeled final/Researcher complete-result boundary remains presentational.

## Spatial experience

Keep existing typography, light/dark variables, control styles and Experiments navigation. Add a shared grid renderer with lab-specific layer interpretation:

- Solid/open terrain, nest/exit and waste outlet use distinct labeled symbols.
- Food, carried food, loose/carried spoil and disposed material keep their actual namespaces and statuses. Equal numeric food/spoil IDs do not imply the same item.
- Worker markers show exported identity, cargo and phase only when available. Burrow glyphs indicate presence, not invented identities or occupant counts.
- A selected cell/Agent inspector explains the native values and their availability. Cargo quantities, work and inventory remain exact text.
- A compact timeline strip shows real Burrow events or labeled CPFA sample intervals. No smooth path animation fills unrecorded steps.

Map fit/zoom and readable coordinate labels support compact and narrow views. Color supplements symbols and text. Keyboard playback, visible focus, manual stepping, reduced motion and responsive inspector layout remain required. Leaving Episodes pauses and retains the successful record. Editing the next run preserves the shown run's original descriptor/input.

For source-compatible comparisons, retain at most two successful records and run requests serially. Match actual setup, seed, horizon and sampling while varying only a declared compatible control/parameter. Burrow comparisons preserve the same fixture/goal; CPFA comparisons preserve geometry, resources, worker identity/order and all other parameters. Refuse incompatible changes. Join actual completed-tick/stage clocks where available, mark absent frames, and show opportunity totals/schedules separately. Same seed does not promise paired random draws after policies consume different draws.

## Bounds, transport and imports

Retain the shell's 64 KiB raw/normalized input and decompressed-link limits, 4,096 checkpoints, 16 MiB complete episode limit, one request/worker, 60-second cancellation/timeout, and two successful-record retention. Native stricter limits still apply. Validate the requested sample count and conservative native-retention bounds before stepping. Burrow retains its original bounded runner and checks the additional browser record while projecting its output; CPFA checks frame/private-map serialization incrementally while capturing. Reject oversized retention requests before allocating their traces rather than relying solely on a final JSON check. These limits bound retained records, not all engine/process memory. Default setups/horizons must pass a source-bound browser feasibility gate before release.

A valid native request may be too large for this browser profile. Reject it contextually with explicit choices to reduce horizon or increase sampling; never silently change scientific inputs. No universal process-memory or throughput guarantee is added. Full lab/diagnostic archives are not downloaded to show a selected episode.

Seeds and arbitrary u64 material/resource IDs cross input and output boundaries as decimal strings. Bounded indexes, dimensions, coordinates and protocol counters retain validated ordinary numeric input fields as appropriate. The Rust adapter converts ID strings into original typed setups and delegates scientific validation to the existing APIs; it does not duplicate controller/physics rules. Payload integer atoms retain the existing lossless string transport.

Preserve existing Burrow WASM exports and their old records. New selected episodes use the common worker-backed run/validate/catalog APIs with source/rules identity and fresh reconstruction. Import caps are checked before parsing; matching versions/identities do not authorize trusting an imported success flag. Exports are freshly validated. Invalid input/link/import, cancellation, timeout and failed replacement preserve the prior record.

## Parity and preservation

Existing Burrow tests cover full native/WASM records at seeds7 and u64max. Extend them to the new envelope without changing that core behavior. Test the CPFA adapters against original World/runner outputs and actual WASM on all declared bounded acceptance fixtures, including nonzero information parameters and censored/blocked examples.

CPFA's existing contract promises supported-platform replay, not universal cross-platform bit identity. Actions, geometry, identities, inventories, costs, clocks, shapes and availability must compare exactly in the new checks. Retain actual floating outputs and report any cross-target difference explicitly. Batch A's 1e-12 exception is confined to its named testimony paths; it does not extend to headings, waypoint strength or CPFA decisions. Same-target imports remain exact fresh reconstructions. If a required target/fixture comparison fails, investigate before release; no automatic tolerance widening, source rewrite, or advertised universal portability follows from this spec.

Bind old Burrow/foraging engine sources and saved acceptance references before implementation. Verify original exports, conservation and read-only/RNG behavior remain unchanged. Add source identities for new adapters rather than rewriting original measured identities. Preserve first/repeat artifacts, settings, source receipts and old product-guide prefixes; disclose any separately authorized post-measurement revision. No new scientific execution or raw evidence bundle is authorized.

## Verification and delivery

Implementation will use fresh producer agents and independent task/whole-branch reviews in the existing crowd worktree. The reviewed plan must cover:

1. Common spatial input/catalog/checkpoint contracts and Burrow adapters.
2. CPFA F2–F4 snapshot/private-memory adapters and native/WASM bridges.
3. Spatial rendering, setup controls, perspective privacy, matched comparison and imports.
4. Original-source/export preservation, all five defaults, actual browser acceptance and documentation.
5. Required full local gates, reviewed commits, safe main integration, normal push and exact final-commit CI/Pages verification.

Tests include zero-tick Burrow, choice stop without commit, off-cadence terminal frames, blocked/wait accounting, censored access/delivery, material conservation/namespaces, stale private walls, no future/peer/global-food leakage, unchanged capture RNG/counters, full native payload comparisons, large IDs/seeds, byte/sample bounds, invalid/missing opened controls, cancellation and disposal. Browser acceptance covers all five entries plus Batch A regression, real keyboard/narrow/reduced-motion interactions, source-at-execution evidence, and comparison clock availability. Distinguish engineering examples and saved scientific measurements throughout.

The existing deferred surface-discovery draft remains untracked. Preserve crowd and all evidence. Commit only scoped owned files; never stage `.claude/`, `papers/`, `survey/out/` or ignored evidence. Rustfmt only edited Rust files; no survey formatting. American spelling; Agent in code/docs. No implementation plan or runtime change starts until this written spec and subsequent plan have been reviewed.
