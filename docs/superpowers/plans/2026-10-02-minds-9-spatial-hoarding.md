# Minds 9 Spatial Hoarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add inspectable spatial larders, paid defense, and seasonal inheritance experiments to the existing Sugarscape world.

**Architecture:** Extend `World` with an opt-in controller and kind-aware storage accounting; retain existing movement, capacity, metabolism, and scatter-cache behavior. A separate core runner constructs fresh, checked cohorts and breeds between episodes. Browser presets run single episodes; the survey consumes generation records.

**Tech Stack:** Existing Rust workspace, serde, rand/PCG and portable math; existing TypeScript/Vite/Vitest/WASM frontend; separate Rust survey crate. No new dependencies.

**Spec:** [Approved Minds 9 design](../specs/2026-10-02-minds-9-spatial-hoarding-design.md). Read it alongside this plan; equations, precedence, and panel definitions there are normative.

## Global Constraints

- New reset-only `spatial_hoarding` extension of `World`; no new ModelKind.
- Single good, walk speed 1, positive carrying capacity, central off, lab none, mixed caching false, caching rule even; ordinary sex/replacement/combat/disease/credit/trade/lifespan deaths disabled.
- Default extension disabled; L=0.15, D=0.5, guard enabled, defense slope 10, larder discovery 0.25. L/D/probabilities finite in [0,1]; slope finite and positive.
- Carrying capacity constrains holdings, including pending deliveries. No remote deposits, free return travel, or defense by occupancy.
- Tick-start guard intentions in agent-id order before turn shuffling. Owner death ends protection immediately.
- Survival selection primary; explicit neutral control; stores weighting sensitivity; extinction and zero fitness are distinct terminal outcomes.
- Episodes: 175 founders, 200 ticks; 60 generations. Separate recorded episode/breeding seeds. Inheritance h²=0.8 and segregation variance 0.5 by default.
- Initial distributions use Minds 7's centers 0.15/0.5; realized means must be reported. Binary strategy flags inherit together from first parent.
- All off-path fingerprints and existing golden expectations remain unchanged. New behavior draws nothing while disabled.
- Seeds 1–40 for comparisons; timing seeds 1–5. Run seed is the sampling unit, generations are not independent samples.
- A separately reviewed, committed judge amendment is required before campaign diagnostic or survey runs. Deterministic correctness tests are permitted before that gate.
- Execute in `/Users/nathan/.config/superpowers/worktrees/SugarScape/minds-9`; preserve unrelated main-worktree and auctions work.

## Review Focus

1. Older/partial config objects and invalid scheduled changes: load defaults, reject incompatible extension combinations, preserve old output and simulation behavior (Task 1).
2. Same-site scatter and larder stocks, partial takes and ledger saturation: distinguish kinds, conserve food, retain explicit incomplete-ledger status (Task 2).
3. Occupied or unreachable home access and depleted delivery intentions: no teleportation, no occupancy immunity, deterministic endpoint ranking, no negative holdings (Tasks 2–3).
4. Guard death and stale/denied observations: no lingering immunity or repeatedly targeted forbidden stock; no accidental double recovery/harvest (Tasks 3–4).
5. All-dead/zero-stock cohorts and missing paired runs: preserve terminal records, never substitute fitness or drop unmatched runs silently (Tasks 5 and 8).

---

## File responsibilities and contracts

Existing seams are `World::step`, `rules::agent_turn`, `movement::go_and_gather`/`arrive`, `caching::rules::act`, `caching::fates`, and `caching::watching`. Do not fork a second copy of the movement controller.

| Files | Responsibility |
|---|---|
| `crates/sugarscape-core/src/config.rs`, `legacy.rs` | Config defaults, validation, schema and reset-only change restrictions; complete legacy config literals |
| `src/minds/spatial_hoarding/mod.rs`, `state.rs`, `access.rs` (under core) | Controller dispatch, extension state and deterministic home/contact geometry |
| `src/minds/spatial_hoarding/stores.rs`, `delivery.rs`, `guard.rs`, `watching.rs` | Transfers, delivery state machine, guard intentions, kind-specific observations/raids |
| `src/minds/spatial_hoarding/runner.rs`, `inheritance.rs` | Archived cohorts, seasonal execution, parent weights and continuous inheritance |
| `src/agent.rs`, `world.rs`, `rules/mod.rs`, `rules/movement.rs`, `minds/mod.rs`, `testkit.rs` | Extension state initialization, action hooks, removal, hashing and deterministic fixtures |
| `src/minds/caching/fates.rs`, `stats.rs`, `export.rs`, `render.rs` | Shared kind-aware FIFO, extension-gated measures/CSV/inspection |
| `src/hoard/world.rs` | Expose reusable finite sampler/math with identical old draw order, without changing old selection rules |
| `src/presets.rs`, `tests/minds.rs`, `tests/checkpoint.rs` | Single-episode presets, reductions, checkpoint regression |
| `web/src/types.ts`, `schema.ts`, `minds.ts`, `ui/inspect-panel.ts`, `ui/grid-view.ts`, `ui/series-data.ts` and their existing tests | Reset controls, neutral names, home/store/guard inspection and series |
| `web/src/determinism.test.ts`, `crates/sugarscape-cli/src/main.rs` | Native/WASM episode trace comparison through the existing CLI/config route |
| `survey/src/claims/minds9.rs`, `claims/mod.rs`, `stats.rs`, `main.rs` | Declared panels, strict paired analysis, measured-report entry point |
| Minds study, roadmap, judge amendment, `survey/out/minds9-results.md` | Protocol, reviewed judging gate, reproducible results and limitations |

All core paths abbreviated `src/...` above are relative to `crates/sugarscape-core/`.

Define these public types in `state.rs`; use them unchanged across tasks:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FounderTraits {
    pub larder: f64,
    pub defense: f64,
    pub cheater: bool,
    pub watches: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StoreKind { Scatter, Larder }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Delivery { pub amount: f64 }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeenLarder { pub home: crate::geometry::Pos, pub amount: f64, pub tick: u64 }
#[derive(Clone, Debug, PartialEq)]
pub struct SpatialState {
    pub home: crate::geometry::Pos,
    pub traits: FounderTraits,
    pub larder: f64,
    pub larder_since: Option<u64>,
    pub delivery: Option<Delivery>,
    pub guarding: bool,
    pub seen_larders: std::collections::BTreeMap<crate::agent::AgentId, SeenLarder>,
}
```

`SeenLarder` retains only observed home coordinates, amount and tick, including a fresh empty/dead-owner target until an attempted raid or expiry clears it. This execution ruling corrects the original owner-only `SeenCache` map; see the task ledger. Keep `SpatialState` public because the existing `Agent` is public; inspection uses the explicit representation in Task 6. Public `FounderTraits` is required for runner callers. Store authoritative cheating/watching on the existing agent fields; `traits` must match them at construction and breeding. No live editing of founder flags in this extension.

`Agent` gains `spatial: Option<SpatialState>`; existing `home` remains the central controller's state. Ordinary agents get `None`.

## Staging and execution

At execution start create `IMPLEMENTATION_PLAN.md` with the following five stages, using the user-required Goal / Success Criteria / Tests / Status format. All begin Not Started. Mark the active stage In Progress and finished stages Complete; remove the file when all five stages are complete. This durable plan remains.

| Stage | Tasks | Goal and success criteria | Verification |
|---|---|---|---|
| 1: Storage and access | 1–2 | Valid gated config; independent conserved store kinds; deterministic contact | Config/off regressions, FIFO/geometry fixtures |
| 2: Episode behavior | 3–4 | Real delivery, paid guarding, observation and arrival precedence | Delivery/guard/raid fixtures, L=0 reduction |
| 3: Seasonal runner | 5 | Checked tick-zero cohorts and archived, reproducible inheritance | Selection, terminal, reset and finite inheritance fixtures |
| 4: User surfaces and replay | 6–7 | Usable single-episode presets, inspection and exact restoration/parity | Core checkpoint, real WASM, web build/tests |
| 5: Research protocol and results | 8–9 | Reviewed judges, complete panels and reproducible measured report | Analysis fixtures, protocol review, full survey/timings |

Use a fresh implementation subagent for each task and the required fresh reviewers. Tasks share world/controller seams, so apply them serially; do not assign concurrent writers to the same files. Resolve review findings before the next task. Commit only passing increments and never update an old golden merely to accommodate changed off behavior.

### Task 1: Checked configuration and initialized extension state

**Files:** Create `src/minds/spatial_hoarding/{mod,state}.rs`; modify `config.rs`, `agent.rs`, `world.rs`, `minds/mod.rs`, `legacy.rs`, `testkit.rs`; tests in config and new module.

**Produces:** `SpatialHoarding` config struct; `FounderTraits`, `SpatialState`, `Delivery`, `StoreKind`; `World::new_with_spatial_cohort(config: Config, seed: u64, cohort: &[FounderTraits]) -> Result<World, Vec<FieldError>>`. Ordinary `World::new` uses uniform config traits when enabled. Both paths initialize before the first snapshot.

- [x] Add failing default/partial JSON tests through existing `Config::from_json`; test invalid finite probabilities, NaN via direct config, slope 0, wrong cohort length, and every incompatible rule named under Global Constraints. Assert scheduled changes to each extension field are rejected as reset-only.

```rust
#[test]
fn older_config_disables_spatial_hoarding() {
    let c = Config::from_json("{}").unwrap();
    assert!(!c.spatial_hoarding.enabled);
    assert_eq!(c.spatial_hoarding.larder, 0.15);
}
#[test]
fn invalid_defense_slope_is_rejected() {
    let mut c = Config::default();
    c.spatial_hoarding.defense_slope = f64::NAN;
    assert!(c.validate().is_err());
}
```

- [x] Run `cargo test -p sugarscape-core spatial_hoarding`; confirm the new field/API is missing or the intended validation assertion fails.
- [x] Add serde-default config fields and schema entries. Validate scalars before enabled-only compatibility checks. Add the `Agent` option to all construction sites discovered with `rg 'Agent \{'`. Refactor initialization internally so the checked constructor assigns homes/traits/flags before initial Snapshot; do not patch a world after recording tick zero.

```rust
pub struct SpatialHoarding {
    pub enabled: bool,
    pub larder: f64,
    pub defense: f64,
    pub guard: bool,
    pub defense_slope: f64,
    pub find_larder: f64,
}
```

Defaults are exactly those in Global Constraints. Keep the constructor's generation-slot assignment in existing ascending founder-id order. Reject invalid traits with field paths including cohort slot.
- [x] Add initialization tests asserting per-slot traits, home equal to founder position, empty stores/intents/memory, flags present in tick-zero snapshot. Run `cargo test -p sugarscape-core` and preserve existing golden expectations.
- [x] Commit explicit changed files: `feat(minds): configure checked spatial hoarding cohorts`.

### Task 2: Kind-aware stores, conservation and contact

**Files:** Create `access.rs`, `stores.rs`; modify caching `fates.rs`, `world.rs` removal/accounting gates, `stats.rs`; test within access/stores modules.

**Consumes:** Task 1 state and `StoreKind`.
**Produces:** `pub(crate) fn in_contact(world: &World, at: Pos, home: Pos) -> bool`; `pub(crate) fn return_endpoint(world: &World, id: AgentId) -> Option<Pos>`; `deposit(world: &mut World, owner: AgentId, amount: f64) -> f64`; `recover(world: &mut World, owner: AgentId, amount: f64) -> f64`; `take(world: &mut World, owner: AgentId, taker: AgentId, amount: f64) -> f64` in stores. Transfer functions enforce contact, room, guard and positive finite quantities themselves, not just in callers. Returned quantity is actual stored/transferred food; caller must not add holdings twice.

- [x] Add failing fixtures: same owner/site with scatter 4 and larder 6; partial owner recovery 2 leaves larder 4/scatter 4; foreign keep/eat loot; burial cost; death; saturated ledger status. Check per-kind open + dug + pilfered + lost equals buried within relative tolerance. Use existing `testkit::blank_world`, `spawn`, and config setup from Task 1.

```rust
#[test]
fn home_contact_excludes_diagonals() {
    let w = crate::testkit::blank_world(7, 7);
    assert!(in_contact(&w, Pos::new(3, 2), Pos::new(3, 3)));
    assert!(!in_contact(&w, Pos::new(2, 2), Pos::new(3, 3)));
}
```

Add wall-separated component fixtures, torus-edge contact, occupied home with accessible neighbor, all endpoints occupied/unreachable, unequal path lengths and equal-length index ties. Use existing bounded pathfinder and wall/region APIs.
- [x] Run `cargo test -p sugarscape-core spatial_hoarding`; confirm missing functions/failed conservation before implementation.
- [x] Add `kind` to `CacheRecord` and `(owner,site,kind)` FIFO keys; retain scatter wrappers, old log cap/backfill semantics and gated allocation. Generalize shared FIFO internals; do not duplicate a second ledger. Larder death closes Lost before deleting stock. Extend `pilfering_on` for enabled larder discovery even with scatter find 0 and watching off.

```rust
// Shared ledger identity, including stores at the same home cell.
pub(crate) type OpenRecords = std::collections::BTreeMap<
    (AgentId, u32, StoreKind), std::collections::VecDeque<usize>>;
```

Select reachable free home/contact endpoints by `(path_length, site_index)`, using the existing search bound; `None` means stay/retry, never teleport. Entry geometry itself does not depend on occupancy.
- [x] Run new transfer/access fixtures plus `cargo test -p sugarscape-core`; assert disabled worlds retain empty extension index/log and unchanged aggregate scatter counters.
- [x] Commit: `feat(minds): add separate spatial larders and contact accounting`.

### Task 3: Delivery state machine and paid tick-start guards

**Files:** Create `delivery.rs`, `guard.rs`; modify `mod.rs`, `world.rs`, `rules/mod.rs`, `minds/mod.rs`, `stats.rs`; unit fixtures alongside new helpers.

**Consumes:** Task 2 contact/transfers; existing `caching::{reserve,surplus,hungry}`, movement arrival.
**Produces:** `prepare_guards(world: &mut World)` before shuffle; `guard_turn(world: &mut World, id: AgentId) -> Option<Harvest>` returns the consumed guard action (extended in Task 4 to carry probe intake); `delivery_target(world: &World, id: AgentId) -> Option<Pos>`; `finish_turn(world: &mut World, id: AgentId)` clamps/deposits/allocates once. Explicitly pass the completion-turn flag internally so no second allocation occurs.

- [x] Add red behavior fixtures: L=1 away from home creates an intent without reducing holdings; movement takes successive ordinary steps; metabolism shrinks/cancels intent; later food does not enlarge it; contact deposit charges burial cost once; contact on allocation turn deposits immediately; pending intent prevents further batches; no return endpoint means no transfer. Test capacity and exact-zero surplus.

```rust
// Extract this pure helper in delivery.rs and exercise the state boundary.
#[test]
fn depleted_delivery_is_cancelled() {
    assert_eq!(clamped_delivery(Some(Delivery { amount: 8.0 }), 0.0), None);
}
#[test]
fn delivery_cannot_expand_after_new_harvest() {
    assert_eq!(clamped_delivery(Some(Delivery { amount: 3.0 }), 9.0),
               Some(Delivery { amount: 3.0 }));
}
```

- [x] Add guard fixtures with forced/near-certain intention and explicit state: intention frozen before shuffled turns; guarding displaces harvest and foreign theft; hungry own recovery allowed; metabolism still occurs; owner dies and subsequent take is unprotected. Guard off + occupied home allows theft. Assert intended/executed counts and recovery quantities separately.
- [x] Run `cargo test -p sugarscape-core spatial_hoarding` for red.
- [x] Implement the action state machine. Allocation is one L Bernoulli per positive ordinary batch, with no draws at L=0/1. Before deposit cap by current surplus and burial-cost affordability; otherwise keep food in ordinary holdings. Select return endpoint via Task 2 and use ordinary walk/arrival. Leave metabolism in `agent_turn` and clamp intent again afterward.

```rust
fn clamped_delivery(old: Option<Delivery>, surplus: f64) -> Option<Delivery> {
    old.and_then(|d| {
        let amount = d.amount.min(surplus.max(0.0));
        (amount > 0.0).then_some(Delivery { amount })
    })
}
```

For guards use `T=max(R,1)+D*capacity` and portable logistic of `slope*(stock/T-0.5)`. Clear all previous intentions, then draw eligible agents in id order before shuffling. Transfer helpers check owner still alive. Ordinary guard turns skip new allocation. Record delivery starts/completions/cancellations, return turns, deposit quantities/cost and guard outcomes.
- [x] Run focused fixtures and all core tests. Add L=0/guard-off/no-larder comparison against existing scatter controller including subsequent RNG-dependent trace, not only stock totals.
- [x] Commit: `feat(minds): charge spatial delivery and guarding action costs`.

### Task 4: Observation, larder raids and unified arrival precedence

**Files:** Create spatial `watching.rs`; modify movement arrival, caching watching/theft, spatial controller, stats and fate exposure records.

**Consumes:** Contact, guarded transfers and tick intentions.
**Produces:** `observe_deposit(world: &mut World, owner: AgentId, amount: f64)`; `raid(world: &mut World, id: AgentId, at: Pos) -> f64`; `stumble(world: &mut World, id: AgentId, at: Pos) -> f64`; spatial arrival hook returning whether food was positively recovered/taken. All transfer functions return quantities already accounted for in holdings/loot.

- [x] Add failing scenarios with both kinds at one home; simultaneous fresh targets; denied guarded raid clears only larder entry; empty target clears; span exactly 2 vs 3; opaque wall; deposit observation reports deposit 2 despite total stock 20; no watcher means no remembered foreign home; carrying room 0; hungry/better raid gates; owner-id tie ordering. Assert no site harvest or later stumble after positive recovery.

```rust
#[test]
fn larder_memory_expires_after_span_not_on_it() {
    assert!(fresh_larder(5, 7, 2));
    assert!(!fresh_larder(5, 8, 2));
}
// Implement and use in sweep and candidate generation:
fn fresh_larder(observed: u64, now: u64, span: u64) -> bool {
    now.saturating_sub(observed) <= span
}
```

- [x] Run `cargo test -p sugarscape-core spatial_hoarding` for red.
- [x] Factor movement's existing gathering phase just enough to insert the approved sequence: own scatter, own larder, observed larder, observed scatter, larder stumble, scatter stumble, harvest. Preserve old path exactly when disabled. Route delivery arrivals through the same hook. Reuse ordinary raid gates and visibility geometry; sum fresh larder observations by accessible endpoint without leaking current stocks. Draw larder discovery once per nonempty contacting owner in id order until a take; blocked hits continue. Record skipped scatter draws and denied/empty/blocked attempts separately.

```rust
// Controller boundary: only positive transfers consume arrival food action.
let taken = raid(world, id, at);
if taken > 0.0 { return true; }
let found = stumble(world, id, at);
found > 0.0
```

The snippet belongs after own recovery and before scatter fallbacks; it does not replace the full normative ordering.
- [x] Define the following runner-only probe in `state.rs`, deriving Clone, Copy, Debug, Default, PartialEq and serde traits:

```rust
pub struct EpisodeProbe { pub guard_harvest: bool, pub scatter_first: bool }
```

Both defaults are false; expose `World::new_with_spatial_probe(config, seed, cohort, probe)` with the same checked initialization as Task 1. Probe guard harvest only after zero own recovery; track it separately. Test both stumble orders with explicit inventories and seeded outcomes; probe cannot affect disabled worlds.
- [x] Run all core tests and existing Minds 8 watching/arrival reductions.
- [x] Commit: `feat(minds): observe and raid spatial larders through ordinary arrivals`.

### Task 5: Archived seasonal cohorts and inheritance

**Files:** Create `runner.rs`, `inheritance.rs`; modify hoard sampler visibility with unchanged operations, spatial module exports; tests in runner/inheritance.

**Consumes:** `FounderTraits`, checked cohort/probe constructors, per-kind stocks/counters.
**Produces:** Public runner records with serde serialization (derive serde traits on the referenced trait/probe/selection/terminal types too). Define the following before implementing selection:

```rust
pub enum Selection { Survival, Stores, Neutral }
pub enum Terminal { Extinct, ZeroFitness }
pub struct FounderRecord {
    pub slot: usize, pub traits: FounderTraits, pub alive: bool,
    pub ticks_alive: u64, pub holdings: f64, pub scatter: f64,
    pub larder: f64, pub parent_weight: f64,
}
pub struct GenerationRecord {
    pub generation: u32, pub episode_seed: u64, pub breeding_seed: u64,
    pub founders: Vec<FounderRecord>, pub terminal: Option<Terminal>,
}
pub struct RunnerConfig {
    pub episode: Config, pub founders: usize, pub ticks: u64,
    pub generations: u32, pub episode_seed: u64, pub breeding_seed: u64,
    pub selection: Selection, pub heritability: f64,
    pub segregation_variance: f64, pub fixed_traits: bool,
    pub probe: EpisodeProbe,
}
pub fn run_generations(config: &RunnerConfig,
    initial: &[FounderTraits]) -> Result<Vec<GenerationRecord>, String>;
```

Add per-generation summary fields for actual mean L/D, shares, per-kind flows/exposure, guards/deliveries and elapsed/completed ticks. Record lineage `(generation,slot)` and both parent slots; local ids alone are insufficient. Keep full config/initial cohort in serialized run envelope.

- [x] Write red tests for exact weights and termination using this extracted pure boundary:

```rust
pub fn parent_weights(selection: Selection, founders: &[FounderRecord])
    -> Result<Vec<f64>, Terminal>;
```

Construct two survivors (scatter/larder 0/0 and 3/4) plus one dead founder with positive closing stock: Survival weights `[1,1,0]`, Stores `[0,7,0]`, Neutral `[1,1,1]`. All survivors with zero stores return ZeroFitness under Stores; all dead return Extinct before selection even under Neutral. One survivor is a valid mating pool.
- [x] Add red inheritance/reset tests: zero variance/h²=1 yields exact parent logit midpoint; h²=0 reference mean includes dead founders; endpoints remain finite; flags both follow first parent; fixed traits do not regress/mutate; fresh episode restores endowment and clears all stores/memories/paths/guards/intents. Terminal generation stays in returned records. Reject 0 founders/ticks/generations, malformed cohorts, invalid h²/variance.
- [x] Run `cargo test -p sugarscape-core spatial_hoarding` for red.
- [x] Expose/reuse the finite clamp, portable inverse logit and normal sampler from Minds 7; preserve its draw order and old golden behavior. Do not reuse its zero-weight uniform fallback or cheater subsidy. Weighted parents sampled independently with replacement in founder-slot order; categorical inheritance from first parent. Document draws: parent1, parent2, L normal, D normal, then next slot. Zero variance consumes no normal draws. Initial L then D samples per slot follow Minds 7 initialization.

```rust
fn regressed_logit(midparent: f64, mean: f64, h2: f64, noise: f64) -> f64 {
    h2 * midparent + (1.0 - h2) * mean + noise
}
```

Archive every founder including removals and time alive. Execute 200 steps for reporting unless cohort extinct; retain completed ticks. Episode boundaries report closing stock without recording loss. Breeding uses independent PCG stream, same recorded episode seed every generation. Add deterministic repeat-run and serialized cohort replay tests without campaign diagnostics.
- [x] Run all core tests; verify old hoard sampler/inheritance fixtures and goldens.
- [x] Commit: `feat(minds): breed archived spatial cohorts between seasons`.

### Task 6: Presets, inspection and browser controls

**Files:** Modify core presets/schema/stats/export/render; web types/schema/minds/inspect-panel/grid-view/series-data and existing adjacent tests.

**Consumes:** Config/state/counters; ordinary ModelWorld dispatch.
**Produces:** Presets `spatial-scatter`, `spatial-larder`, `spatial-larder-guard`; conditional inspect object `spatial_hoarding` with home, L/D, larder, delivery, guard and observed-larder count. Per-kind stock, burial, recovery, pilferage, loss, exposure and delivery/guard series; retain legacy totals.

- [x] Add red preset tests: derive all three from `theft-winter`, apply approved capacity/horizon/season/share settings; L=0/1/1, guard false/false/true, D=.5. Existing preset count-dependent tests must filter their original model/preset domains rather than excluding new behavior arbitrarily. Add frontend schema/preset tests for reset-only fields and older configs lacking extension object.

```ts
// In the real-WASM preset test in determinism.test.ts:
const presets = JSON.parse(presets_json()) as { id: string }[];
const spatialPresetIds = presets.map(p => p.id).filter(id => id.startsWith('spatial-'));
expect(spatialPresetIds).toEqual([
  'spatial-scatter', 'spatial-larder', 'spatial-larder-guard',
]);
```

Use the existing real WASM initialization and imported `presets_json` in that file; rebuilding WASM is required before its test run. Keep presentation-helper tests in `minds.test.ts`.
- [x] Run core preset tests and `npm run wasm:dev --prefix web`, then `npm test --prefix web -- determinism.test.ts minds.test.ts` for red.
- [x] Extend existing config/schema/preset route, conditional CSV columns and inspect rendering. Home marker/guard indication should be legible and use existing grid overlay conventions. Display scatter/larder separately and disclose pending amount remains carried. No generation controls or new model menu. Reuse CLI `--config` route; verify actual argument syntax from `--help` at execution rather than inventing a flag.

```ts
// Extension inspect fields are optional for old recordings/configs.
type SpatialHoardingInspect = {
  home: { x: number; y: number };
  larder_trait: number; defense_trait: number; larder: number;
  delivery: number | null; guarding: boolean; observed_larders: number;
};
```

Pin serialized field names in core render tests and TypeScript inspect tests. Keep disabled CSV headers/series unchanged; kind totals should agree with aggregates where applicable. Include unused/zero exposure as undefined ratios, not fabricated zeros.
- [x] Run focused core/export tests, `npm run build --prefix web`, `npm test --prefix web`. Inspect one deterministic handcrafted episode's home/guard/store display; use visual companion only if showing a concrete visual, per user preference.
- [x] Commit: `feat(minds): expose spatial hoarding episodes and inspection`.

### Task 7: Checkpoint, hash and native/WASM replay

**Files:** Modify `world.rs` fingerprint, existing checkpoint tests and `web/src/determinism.test.ts`; use existing ModelWorld/WASM adapters, modifying them only if required for serialization/inspection.

**Consumes:** Entire episode state; existing clone keyframes.
**Produces:** Disabled-world compatibility and enabled-world exact replay.

- [x] Add red checkpoint tests at pending delivery, active guard and fresh observed larder; clone/restore then step both branches and compare complete fingerprints, snapshots, inspection and stocks. Mutating each authoritative extension field must change enabled fingerprint. State irrelevant while disabled must not change the legacy fingerprint.

```rust
// Extend tests/checkpoint.rs using its existing checkpoint/restore helpers.
// For each of delivery, guard and sighting fixtures, compare after restore:
assert_eq!(original.fingerprint(), restored.fingerprint());
original.step();
restored.step();
assert_eq!(original.fingerprint(), restored.fingerprint());
```

- [x] Run `cargo test -p sugarscape-core --test checkpoint` for red. Use deterministic hand-crafted fixtures, not a search over campaign seeds.
- [x] Hash enabled home/traits/larder/age/intent/guard/observations in deterministic field/key order. Keep fates out of hashes as existing optional diagnostics. Keyframe cloning must retain full extension state; no rebuild that loses intentions or observations.
- [x] Extend existing real WASM determinism tests with all three spatial presets and an explicit cohort fixture if WASM constructor exposure is needed; compare native CLI and WASM tick traces including tick zero and tick 200. Ensure portable inheritance math has native deterministic tests even though generation playback is outside frontend scope.
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm run build --prefix web`, `npm test --prefix web`. Diagnose failures; do not rebaseline old fingerprints.
- [x] Commit: `test(minds): verify spatial hoarding restoration and wasm parity`.

### Task 8: Declared panels, strict analysis and judge amendment

**Files:** Create `survey/src/claims/minds9.rs`; modify `claims/mod.rs`, `main.rs`, `stats.rs`; create `docs/superpowers/specs/2026-10-02-minds-9-judge-amendment.md`.

**Consumes:** Runner records and initial cohort sampler; full approved panel list.
**Produces:** Measured `--minds9` report route independent of existing Holds/Fails registry; deterministic panel manifests with condition ids/config/cohort/seeds; pure strict pairing function and protocol amendment. Formal verdicts only if amendment defines and user approves them.

- [x] Add red analysis tests and panel manifest tests without running simulations: all five approved panels, spans 2/7 plus watching off, discovery settings, rare/common shares, survival/stores/neutral inheritance controls, fixed traits and probes are present exactly. Reject duplicate condition/seed pairs, missing terminal rows, missing comparison partners and nonfinite measured inputs.

```rust
pub struct PairedSummary {
    pub n: usize, pub mean: f64, pub ci95: Option<(f64, f64)>,
    pub positive: usize, pub zero: usize, pub negative: usize,
}
pub fn paired_summary(a: &std::collections::BTreeMap<u64, f64>,
    b: &std::collections::BTreeMap<u64, f64>) -> Result<PairedSummary, String>;

#[test]
fn identical_pairs_have_zero_width_interval() {
    let a = [(1, 2.0), (2, 3.0)].into_iter().collect();
    let s = paired_summary(&a, &a).unwrap();
    assert_eq!(s.ci95, Some((0.0, 0.0)));
}
```

Also test differences `[1,2,3]` give mean2/positive3, CI using Student t df2; n1 yields no CI; unmatched seed is an error; undefined per-kind exposure ratio stays absent.
- [x] Run `cargo test --manifest-path survey/Cargo.toml minds9` for red.
- [x] Implement pure manifest and report analysis. Use existing `student_t_cdf` to invert two-sided .975 quantile with bounded bisection; no normal approximation for 40 seeds. Keep terminal counts, all trajectories, endpoint/parent-weight distributions and denominator/missingness metadata. Return machine-readable envelopes with config/cohort/seed/probe/version/draw-order information and Markdown summary.
- [x] Draft amendment before invoking any campaign run. Define primary paired contrasts, units/denominators, seedwise signs/95% intervals, incomplete-ledger handling, extinction/zero-fitness reporting, undefined/unused mechanisms, sensitivity separation and timing procedure. Prefer measured contrasts with no Holds/Fails claims; if claims are proposed, explicitly define Untestable and thresholds and get approval. Include rare/common frozen-trait requirements for any frequency-dependence language. No imported .219 threshold or Minds 7 endpoint classification.
- [x] Run survey analysis tests and formatting/clippy for the separate crate. Commit: `feat(survey): declare spatial hoarding panels and judging protocol`.
- [ ] Present the concrete amendment for review and wait for approval before Task 9. This is a specific scientific gate in the approved spec, separate from ordinary implementation review. Commit any approved revisions before running diagnostics. Do not mark Stage 5 complete while this gate is pending.

### Task 9: Run, report and close the campaign

**Files:** Tracked `survey/out/minds9-results.md`, Minds study and existing roadmap; raw run envelopes in existing ignored survey output location. Remove completed `IMPLEMENTATION_PLAN.md`.

**Consumes:** Approved and committed amendment, passing implementation, panel manifests.
**Produces:** Reproducible report with all declared outcomes and limits; next campaign entry remains protection/deception.

- [ ] Verify amendment approval is recorded and committed. Record revision, manifest, toolchain and exact actual `--minds9` command/arguments. Before the full run, execute the deterministic analysis suite:
  `cargo test --manifest-path survey/Cargo.toml minds9`.
- [ ] Run all declared panels at seeds1–40 using the reviewed CLI; save raw envelopes before rendering. Run whole-tick and completed-generation timing probes for seeds1–5. Use CLI help/manifest output to establish command syntax; no ad hoc parameter changes to rescue unfavorable results.
- [ ] Check every manifest cell has all40 seed records, including terminal records. Verify kind conservation/ledger fullness, initialization matches, generation boundary resets, and timing denominators. If a correctness defect is found, fix/test/review/commit and rerun affected cells; document the discarded revision. Unexpected scientific results are reportable outcomes, not defects.
- [ ] Render the report with paired survival contrasts, per-seed signs/95% intervals, flows/exposures/guard opportunity cost/return travel, evolution trajectories/endpoints, survival/terminal rates/parent weights, neutral and inheritance controls, strategy-contest rare/common contrasts and separate probes. Distinguish concentration/defense/travel effects; do not infer stable mixtures from interior frequencies or regulation conclusions from toy agents.
- [ ] Update the Minds study with exact mechanism choices, empirical results, open questions and links to report/amendment/program spec. Long-term affordances, communications and collective agency remain later campaigns; this implementation does not claim they exist.
- [ ] Run final required checks (`cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --manifest-path survey/Cargo.toml -- --check`, `cargo test --manifest-path survey/Cargo.toml`, `cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings`, `npm run build --prefix web`, `npm test --prefix web`), review final diff and status, remove completed staging file, and commit: `docs(minds): report spatial hoarding campaign outcomes`.
- [ ] Conduct required whole-branch review and resolve findings. Deliver commit/report links, scientific conclusions with limitations, verification and next campaign. Integrate only with user authorization or established repository workflow; never overwrite unrelated changes.

## Plan self-review

- Spec coverage: configuration/initialization (1); access/FIFO/owner death (2); allocations/delivery/guard timing (3); full arrival order/watch memory/discovery/probes (4); all selection/inheritance/reset/lineage rules (5); presets/UI/exports (6); replay/portable parity (7); all panels/analysis/protocol gate (8); executions/timings/results (9).
- Contracts: transfer quantities include holdings effects; own scatter helper's existing return convention is adapted only at the arrival seam. Delivery completion and post-metabolism clamping have separate hooks. Cohorts are checked before initial snapshots.
- Review Focus: all five failure classes have named fixtures in their owning tasks.
- Research boundary: no diagnostic campaign simulations until Task 8 amendment review; no claim threshold selected from observed output.
