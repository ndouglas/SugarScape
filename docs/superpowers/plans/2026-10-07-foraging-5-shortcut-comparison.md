# F5 Shortcut Comparison Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user's standing preference already selects subagent-driven execution.

**Goal:** Build a bounded native survey harness that collects immutable F4 construction evidence and analyzes the approved shortcut comparison from saved data.

**Architecture:** Keep all F1–F4 core behavior unchanged. A scenario/manifest layer feeds the existing public F4 runner; strict local wire types validate its saved output, an archive layer binds identities/bytes/provenance, and a saved-only report layer uses existing paired statistics. Scientific collection remains blocked for the draft executable manifest.

**Tech Stack:** Existing Rust survey crate, public sugarscape-core construction API, serde/serde_json, sha2, standard filesystem/Git helpers and `survey::stats::paired_summary`. Enable only serde_json's existing `raw_value` feature for exact numeric-token retention; add no package dependency.

**Spec:** [approved F5 design](../specs/2026-10-07-foraging-5-shortcut-comparison-design.md), user approved 2026-10-07. [Source audit](../../studies/2026-10-07-foraging-shortcut-reading.md) remains explicit about inaccessible biological data. The user approved this plan on2026-10-07; engineering execution is starting. Scientific collection remains blocked.

## Global Constraints

- Core production code/public APIs, worker policy, information boundaries, RNG and scheduler remain unchanged. No private World access, restoration, roles/relay/drop/pile behavior, generic model registration or browser/WASM integration.
- Grid25×25; nest x2..4/y20..22; outlet(1,21); food x2..5/y2..5, IDs0..15 by y then x; exact eight-spawn list from spec. Interior mask x1..23/y1..23, protected outer border.
- Route lengths/open counts: Straight15/40, Detour48/73, Twisting60/85. AlreadyOpen adds straight corridor and has distance15 with empty mask. Paid and Protected have identical initial open geometry; Paid uses the interior mask and Protected uses an empty mask.
- Nine primary route cases; six secondary sealed-access cases. Remove(3,6) for Straight or(6,5) for Detour/Twisting. Access cases have exposed/disconnected food and no AlreadyOpen regime. Canonical case ordering is route then access, geometries straight/detour/twisting, regimes paid/protected/already_open where legal.
- All cases explicitly set p_search0.05, p_return0.01, lambda_fidelity1.0, lambda_publish1.0, lambda_waypoint0.01; ticks512/sample_every128/snapshots true; frames0/128/256/384/512 and opportunities4096. Execute full horizon, including after depletion.
- Scientific seeds10001..10040:600 episodes/2,457,600 opportunities. Construction seeds7,8:30 episodes/122,880 opportunities. Never execute a scientific seed in engineering tests; synthetic statistics fixtures are not simulation episodes. No seed/parameter/horizon overrides or adaptive selection.
- Primary delivered-food contrast Paid−Protected within each route geometry; references and sealed-access contrasts remain secondary. Use existing paired Student-t descriptive intervals/sign counts. No pooled efficacy score, energy conversion, animal uncertainty or significance/threshold verdict.
- Preserve zeros, censored events, unfinished cargo, all work categories and separate worker/researcher diagnostics. Routes use all original resource positions after delivery. Sealed gain/first-shortening-from-initial stay null; checkpoint shortening is an interval, not an action timestamp.
- Protected/AlreadyOpen Straight core Episodes match; mask-disabled cases have zero excavation/spoil; Protected sealed has zero pickup/delivery/access; all physical distances are at least15 when connected. These are programmed controls, not discovered outcomes.
- Raw envelope4MiB; cumulative raw-envelope JSON1GiB; F4 snapshot JSON retains its independent64MiB definition. Metadata/operational overhead is excluded from raw totals. One raw episode at a time in analysis; bounded rows retained, no Worlds/full histories.
- Default/help/manifest printing create no Worlds. Draft scientific run rejects before output creation. Construction requires explicit run/provenance/new output. Exclusive files, no overwrite/resume; incomplete evidence cannot become a reduced-denominator analysis.
- Saved-only analysis validates hashes/identities/observed physical constraints, not every intervening action/controller/RNG history. Reanalysis deterministically reproduces analysis.json/results.md; native operational timing remains separate.
- Changes stay on foraging-5-design, global worktree `/Users/nathan/.config/superpowers/worktrees/SugarScape/foraging-5-design`. Preserve concurrent work. No merge, push, executable scientific registration, scientific collection or R/dependency installation is part of this implementation.

## Review Focus

1. Missing/duplicate/extra runs or rewritten hash-consistent malformed records: reject the archive, retaining every expected key; never report a favorable subset. Tasks2/3/5 own tests.
2. Float token changes, clock/cargo/tag/ledger corruption and inconsistent snapshots: strict bounded decoding and field-context errors, with exact snapshot-byte checks and no core deserialization. Task2 owns tests.
3. Sealed null distances, delivered-token route records, checkpoint intervals and globally opened remembered faces: keep censoring and private-belief limits correct, without omniscient target pruning. Tasks2/5 own tests.
4. Mixed study selectors, contradictory modes, tuning flags or draft scientific runs: reject before world/output side effects; preserve other study dispatch. Task4 owns tests.
5. Absolute/parent/alias/symlink raw paths, oversized metadata/records, checked-byte overflow and failed writes: reject safely and preserve incomplete evidence without a completion index. Tasks3/4/5 own tests.

## Execution process and five-stage tracker

Parent creates IMPLEMENTATION_PLAN.md only after plan approval, with five stages corresponding to Tasks1–5, Goal/Success Criteria/Tests/Status; implementers mark In Progress and parent marks Complete after review. Parent removes completed tracker only after final review and evidence preservation. Keep briefs/reports/logs/diff packages in this plan's own ignored SDD workspace, never another plan's workspace.

Fresh implementer per task, fresh combined spec/quality reviewer per gate, final whole-branch reviewer on the most capable model. Implementers/reviewers do not dispatch agents. Parent owns review/fix loops and bookkeeping; no controller runtime fixes. Preserve all architectural/boundary rulings chronologically with their rework costs for final reporting.

Study pertinent existing patterns: construction runner/view/access, Burrow survey archive/report, protection survey archive and stats::paired_summary. Per task write tests first, await/inspect completed RED, implement minimally, focused GREEN, self-review and commit. Disclose API-compilation/scaffold reds and supplemental coverage honestly. Construction matrix size means thirty distinct case/seed keys per collection; covering reruns after source changes do not authorize new seeds or scientific draws. Max3 unexpected attempts per issue then document/reassess/escalate. Never bypass hooks/disable tests.

Use CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target with Cargo locks. Before each task commit freeze changed Rust/tests plus its candidate-data file, run focused checks, survey fmt/Clippy and complete survey tests once on final revision; inspect output and preserve full commands/logs/exits/hashes. Core workspace gates run at final integration Task5; broaden earlier only for a named concern. Evidence-only documentation commits do not justify repeating successful unchanged-source suites. Narrow dead-code allowances name later consumers and disappear by Task5.

## File map and shared contracts

New namespace: `survey/src/claims/foraging_shortcuts/`. Root claims/mod.rs exports this namespace for CLI dispatch only; never add it to claims::all().

| File | Responsibility | Owner |
| --- | --- | --- |
| mod.rs, scenario.rs, manifest.rs | Namespace/shared identities; literal geometry; immutable candidate | 1 |
| wire.rs, wire_state.rs, wire_view.rs | Strict episode/input DTOs; material/metric DTOs; view/access DTOs | 2 |
| validate.rs, agents.rs, physical.rs, access.rs | Episode orchestration; agents/counter checks; sampled materials/history; deterministic terrain/access replay | 2 |
| archive.rs, io.rs | Exclusive writer/bounded reader/index; path/byte/fs helpers | 3 |
| run.rs, cli.rs | Provenance/preflight/F4 collection; pure command parser and dispatch | 4 |
| report.rs, report_rows.rs | Saved dataset analysis/contrasts/Markdown; route-row projections | 5 |
| tests/mod.rs, tests/scenario.rs | Registrations and hand-checked scene/manifest tests | 1 |
| tests/support.rs, tests/wire.rs, tests/physical.rs, tests/access.rs | Cached construction-only fixtures and adversarial wire/state tests | 2 |
| tests/archive.rs, tests/run.rs, tests/report.rs | Corresponding archive/collector/report tests | 3/4/5 |
| survey/tests/foraging_shortcuts_cli.rs | External binary print/deny/saved-analysis acceptance | 4/5 |
| docs/superpowers/specs/2026-10-07-foraging-5-draft-manifest.json | Full factory-resolved executable candidate, draft/unauthorized | 4 |
| docs/foraging.md, spec/plan/tracker | Accurate engineering/status/evidence handoff | 5/parent |

Shared root types below use serde with snake_case on study enums, strict unknown-field denial on decoded structs, pub(super) visibility for sibling consumers. All values are Clone/Debug/PartialEq; RunKey also Eq/Ord. No external public library API is introduced.

```rust
// Task1, mod.rs; only this module's CLI export is pub(crate).
enum Panel { Route, Access }
enum Geometry { Straight, Detour, Twisting }
enum Regime { Paid, Protected, AlreadyOpen }
enum CollectionMode { Construction, Scientific }
struct RunKey { condition:String, seed:u64 }
struct Provenance {
    code_revision:String, protocol_revision:String,
    manifest_sha256:String, collector_sha256:String,
}
fn sha256(bytes:&[u8]) -> String; // lowercase SHA-256
// Task1, manifest.rs. Serialize only because core input types Serialize only.
struct Condition {
    id:String,panel:Panel,geometry:Geometry,regime:Regime,
    setup:sugarscape_core::foraging::construction::Setup,
    options:sugarscape_core::foraging::construction::RunOptions,
}
struct Manifest {
    schema:String,version:u32,status:String,execution_authorized:bool,protocol:String,
    conditions:Vec<Condition>,construction_seeds:Vec<u64>,scientific_seeds:Vec<u64>,
    raw_record_limit:u64,raw_total_limit:u64,metadata_limit:u64,
}
fn candidate() -> Result<Manifest,String>;
fn manifest_bytes() -> Result<Vec<u8>,String>; // pretty JSON plus final newline
fn condition<'a>(m:&'a Manifest,id:&str) -> Result<&'a Condition,String>;
fn expected_keys(m:&Manifest,mode:CollectionMode) -> Vec<RunKey>;
fn authorize(m:&Manifest,mode:CollectionMode) -> Result<(),String>;
// Task1, scenario.rs; geometry validation requires no World/RNG.
fn build(panel:Panel,geometry:Geometry,regime:Regime) -> Result<core::Setup,String>;
fn patch_distances(setup:&core::Setup) -> Result<Vec<(u64,Option<u32>)>,String>;
```

Use `core` as the local alias for sugarscape_core::foraging::construction, not Rust's core crate. Manifest status is exactly `draft`, execution_authorized false; schema `foraging-shortcut-manifest-v1`, version1, protocol path the approved F5 spec. raw limits match spec. Planning refinement: metadata_limit4MiB bounds hostile index/progress inputs independently of raw totals; valid fixed metadata fits this limit. No new tuning flag exposes that limit.

```rust
// Task2, wire.rs. Encode using the ORIGINAL core Episode, never a parsed float reconstruction.
struct WireEnvelope { schema:String,key:RunKey,mode:CollectionMode,provenance:Provenance,episode:WireEpisode }
struct WireCondition { id:String,panel:Panel,geometry:Geometry,regime:Regime,setup:WireSetup,options:WireRunOptions }
struct WireManifest {
    schema:String,version:u32,status:String,execution_authorized:bool,protocol:String,
    conditions:Vec<WireCondition>,construction_seeds:Vec<u64>,scientific_seeds:Vec<u64>,
    raw_record_limit:u64,raw_total_limit:u64,metadata_limit:u64,
}
struct WireFloat(String); // finite canonical numeric token; raw_value-backed custom serde
fn encode_envelope(key:&RunKey,mode:CollectionMode,p:&Provenance,e:&core::Episode,limit:u64) -> Result<Vec<u8>,String>;
fn decode_envelope(bytes:&[u8],key:&RunKey,mode:CollectionMode,p:&Provenance,limit:u64) -> Result<WireEnvelope,String>;
// Task2, validate.rs. Condition/key bind exact normalized input, seed, options and frames.
fn validate_episode(c:&Condition,key:&RunKey,e:&WireEpisode) -> Result<(),String>;
// Task2, physical.rs; smaller hand-worked core scenes allowed in component unit tests.
fn validate_frames(setup:&WireSetup,frames:&[WireSnapshot]) -> Result<(),String>;
// Task2, access.rs. Returns one record per frame with cached distances checked by geometry replay.
struct AccessPoint { completed_ticks:u32,distance:Option<u32>,per_food:Vec<(u64,Option<u32>)> }
fn validate_access(setup:&WireSetup,frames:&[WireSnapshot]) -> Result<Vec<AccessPoint>,String>;
```

Wire DTO field schemas are fully specified in Appendix A. All core f64 fields become WireFloat; default float parsing must not silently change producer numeric tokens before snapshot_bytes or canonical input verification. Enable serde_json raw_value in survey Cargo.toml; ordinary parsing/formatting in other studies remains unchanged. Raw numeric tokens are finite, at most64 bytes, and compared with the canonical original-core serializer for expected setup/waypoint strengths. Never add core Deserialize. Frame serialization preserves struct field order and numeric tokens, so summed compact Snapshot JSON bytes exactly match the core count.

```rust
// Task3, archive.rs/io.rs.
struct RawRef { key:RunKey,path:String,sha256:String,bytes:u64 }
struct Index {
    schema:String,manifest:WireManifest,mode:CollectionMode,provenance:Provenance,
    approval_context:String,expected_keys:Vec<RunKey>,completed:bool,runs:Vec<RawRef>,raw_bytes:u64,
}
struct Limits { record:u64,total_raw:u64,metadata:u64 }
struct ArchiveWriter {
    out:PathBuf,manifest:Manifest,mode:CollectionMode,provenance:Provenance,
    approval_context:String,limits:Limits,keys:Vec<RunKey>,next:usize,
    refs:Vec<RawRef>,raw_bytes:u64,failed:bool,
}
impl ArchiveWriter {
    fn create(out:&Path,m:&Manifest,mode:CollectionMode,p:&Provenance,approval:&str,limits:Limits) -> Result<Self,String>;
    fn put(&mut self,key:&RunKey,bytes:&[u8]) -> Result<(),String>;
    fn fail(&mut self,message:&str) -> Result<(),String>;
    fn finish(self) -> Result<Index,String>;
}
struct ArchiveReader { root:PathBuf,index:Index,manifest:Manifest,next:usize,bytes_seen:u64 }
impl ArchiveReader {
    fn open(index:&Path) -> Result<Self,String>;
    fn index(&self) -> &Index;
    fn next(&mut self) -> Result<Option<(RunKey,WireEpisode)>,String>;
}
fn validate_record(root:&Path,index:&Index,m:&Manifest,reference:&RawRef) -> Result<WireEpisode,String>;
fn raw_path(key:&RunKey) -> String; // raw/{condition}/{seed}.json
fn read_bounded(path:&Path,limit:u64) -> Result<Vec<u8>,String>;
fn checked_total(previous:u64,next:u64,limit:u64) -> Result<u64,String>;
// Task4, run.rs/cli.rs. Native-only repo/executable context, internal injection for owned test fixtures.
struct ExecutionContext { repo:PathBuf,executable:PathBuf }
struct RunRequest { mode:CollectionMode,protocol_revision:String,approval_context:String,out:PathBuf }
fn preflight(ctx:&ExecutionContext,m:&Manifest,r:&RunRequest) -> Result<Provenance,String>;
fn collect(ctx:&ExecutionContext,r:&RunRequest) -> Result<Index,String>;
enum Command { Manifest,Help,Run(RunRequest),Analyze { index:PathBuf,out:PathBuf } }
fn parse(args:&[String]) -> Result<Command,String>;
pub(crate) fn cli(args:&[String]) -> Result<(),String>; // root reexport; helpers remain pub(super)
// Task5, report_rows.rs/report.rs; no engine run import in production reporting.
struct ShorteningWindow { after_completed_tick:u32,by_completed_tick:u32 }
struct EpisodeRow {
    key:RunKey,panel:Panel,geometry:Geometry,regime:Regime,summary:WireSummary,
    route:Vec<AccessPoint>,initial_distance:Option<u32>,final_distance:Option<u32>,
    gain:Option<u32>,first_shortening:Option<ShorteningWindow>,sampled_physical_sha256:String,
}
fn project_row(c:&Condition,key:&RunKey,e:&WireEpisode) -> Result<EpisodeRow,String>;
struct PairedStats { n:usize,mean:f64,ci95:Option<(f64,f64)>,positive:usize,zero:usize,negative:usize }
enum ContrastRole { Primary,Secondary }
struct Contrast { id:String,panel:Panel,geometry:Geometry,outcome:String,role:ContrastRole,stats:PairedStats }
struct Analysis { schema:String,mode:CollectionMode,provenance:Provenance,rows:Vec<EpisodeRow>,contrasts:Vec<Contrast> }
fn build_contrasts(rows:&[EpisodeRow]) -> Result<Vec<Contrast>,String>; // pure synthetic unit tests allowed
fn analyze(index:&Path,out:&Path) -> Result<Analysis,String>;
fn markdown(a:&Analysis) -> Result<String,String>;
```

Private ArchiveWriter state is explicitly: out PathBuf, manifest Manifest, mode CollectionMode, provenance Provenance, approval_context String, limits Limits, keys Vec<RunKey>, next usize, refs Vec<RawRef>, raw_bytes u64, failed bool. Reader state is root PathBuf, index Index, manifest Manifest, next usize, bytes_seen u64. Use canonical expected-key Vec order, not RunKey's lexicographic BTreeMap ordering (access IDs would otherwise precede route IDs). Reader parses/validates one episode at a time; finish consumers require next()=None only after the exact complete set.

### Task 1: Literal scenarios and immutable manifest

**Files:** Create mod.rs, scenario.rs, manifest.rs, tests/mod.rs, tests/scenario.rs under the new namespace; register only this namespace in survey/src/claims/mod.rs. No CLI/collector or candidate JSON file yet.
**Consumes:** Public core Setup/Parameters/Pos/Resource/RunOptions; binding geometry/control tables.
**Produces:** Shared root types, exact scene builder/independent BFS, Manifest/Condition/candidate/bytes/condition/keys/authorize contracts above.

- [x] Write scene/manifest REDs with independently specified assertions. Example:

```rust
#[test]
fn detour_and_sealed_gate_match_hand_geometry() {
    let s=build(Panel::Route,Geometry::Detour,Regime::Paid).unwrap();
    assert_eq!(s.open.len(),73);
    assert_eq!(patch_distances(&s).unwrap().iter().filter_map(|(_,d)|*d).min(),Some(48));
    let sealed=build(Panel::Access,Geometry::Detour,Regime::Paid).unwrap();
    assert!(patch_distances(&sealed).unwrap().iter().all(|(_,d)|d.is_none()));
}
#[test]
fn scientific_keys_are_declared_but_draft_run_is_denied() {
    let m=candidate().unwrap();
    assert_eq!(expected_keys(&m,CollectionMode::Scientific).len(),600);
    assert!(authorize(&m,CollectionMode::Scientific).is_err());
    assert_eq!(expected_keys(&m,CollectionMode::Construction).len(),30);
}
```

- [x] Await `cargo test --manifest-path survey/Cargo.toml foraging_shortcuts::tests::scenario` RED before production implementation. Missing API compilation is disclosed; where a signature scaffold is used, await its assertion/error RED before replacing it.
- [x] Implement inclusive cardinal route expansion with the literal vertices from spec. Minimal independent segment builder:

```rust
fn segment(a:core::Pos,b:core::Pos,out:&mut BTreeSet<core::Pos>) -> Result<(),String> {
    if a.x!=b.x && a.y!=b.y { return Err("route segment must be cardinal".into()); }
    for x in a.x.min(b.x)..=a.x.max(b.x) {
        for y in a.y.min(b.y)..=a.y.max(b.y) { out.insert(core::Pos {x,y}); }
    }
    Ok(())
}
```

Build all nine nest cells, outlet and sixteen food cells first; expand segments; remove only the declared gate for Access, or union Straight for AlreadyOpen. Reject Access+AlreadyOpen. Assign IDs by y then x, preserve exact spawn order, set all five parameters/options explicitly, apply mask by regime and call Setup::normalized. Private helper `vertices(Geometry)->Vec<core::Pos>` contains every literal vertex, never inferred shortest paths. BFS marks on enqueue and visits each open cell once, starting all nest cells; return all food IDs with Some/None shortest distances. No calls to core World or run.
- [x] Build candidate in canonical panel/geometry/regime order with exact statuses/seeds/limits/protocol path. manifest_bytes renders pretty JSON plus newline; sha256 uses existing sha2. Test fifteen unique IDs, exact ordering/keys/seeds/full config, stable bytes/digest, 40/73/85 open counts, 15/48/60 distances, all references15, sealed0-access, masks489/456/444 solid capacities (+1 sealed), food identities, same nest/spawns/outlet, protected border and Straight Protected==AlreadyOpen. Test unsupported combination and malformed segment errors. No scientific episode is run to obtain expectations.
- [x] Focused GREEN, survey fmt/Clippy/full survey on frozen final task; fresh review; commit `feat(foraging): define immutable shortcut comparison scenarios`.

### Task 2: Strict wire decoding and observed-state validation

**Files:** Create wire.rs, wire_state.rs, wire_view.rs, validate.rs, agents.rs, physical.rs, access.rs, tests/support.rs, tests/wire.rs, tests/physical.rs, tests/access.rs; module/test registrations. Modify only survey/Cargo.toml's existing serde_json declaration to enable raw_value. No core or other-study code changes.
**Consumes:** Task1 identities/scenes/manifest/sha256 and public core Episode serialization/F1 waypoint_strength.
**Produces:** All Appendix A wire DTOs; bounded encoder/strict decoder; validate_episode, validate_frames, validate_access/AccessPoint. Tests/support.rs exports cached_candidate_episode(id:&str,seed:u64)->Result<core::Episode,String>, test_provenance()->Provenance, encoded_fixture(id:&str,seed:u64)->Result<Vec<u8>,String>, decode_fixture(id:&str,seed:u64)->Result<WireEnvelope,String>. Cache exact construction-only case/seed values via a test-local Mutex map; reject any seed outside7/8 before invoking core run. No cache or test hook in production.

- [x] Write actual wire/counter/null-state REDs and separate lexical-float tests before implementing. Example:

```rust
#[test]
fn sealed_baseline_is_null_after_food_remains_available() {
    let m=candidate().unwrap();let c=condition(&m,"access.straight.protected").unwrap();
    let e=decode_fixture(&c.id,7).unwrap();
    validate_episode(c,&e.key,&e.episode).unwrap();
    assert!(validate_access(&e.episode.setup,&e.episode.snapshots).unwrap()
        .iter().all(|p|p.distance.is_none()));
}
#[test]
fn total_opportunities_must_match_completed_clock() {
    let m=candidate().unwrap();let c=condition(&m,"route.straight.protected").unwrap();
    let mut e=decode_fixture(&c.id,7).unwrap();
    e.episode.snapshots.last_mut().unwrap().summary.work.opportunities=4097;
    e.episode.summary=e.episode.snapshots.last().unwrap().summary.clone();
    assert!(validate_episode(c,&e.key,&e.episode).unwrap_err().contains("work.opportunities"));
}
```

- [x] Await `cargo test --manifest-path survey/Cargo.toml foraging_shortcuts::tests::wire` RED, plus corresponding physical/access filters. Initial helper/API reds may fail compilation; disclose whether deeper assertions ran. No scientific seeds enter fixture helpers.
- [x] Implement Appendix A strict typed DTOs with exact core JSON variant spelling; deny unknown and duplicate fields, invalid numeric kinds/ranges, oversized records, invalid UTF-8 and trailing input. WireFloat preserves numeric tokens with existing RawValue and finite checking; do not serialize its String as a quoted string. Encoder serializes borrowed ORIGINAL core Episode into a bounded Vec-backed Write; check size before extend, then return complete bytes only. Wire schemas preserve source field order, so compact reserialization matches snapshot_bytes without float token loss. A valid collector envelope round-trips byte-for-byte; test adversarial long numeric token, NaN/null/nonfinite/type errors and exact/one-byte-short encoder caps.
- [x] Bind schema/key/mode/provenance plus episode.seed. compare reserialized WireSetup/WireRunOptions with expected normalized core input/options. Require frames0/128/256/384/512 once; final summary==last frame summary, zero initial opportunities/material work, each agent opportunities=completed_ticks and total8×clock. Generic component helpers derive worker count/grid size from their WireSetup; candidate-level checks enforce the fixed eight-worker values. Check all three physical category equations, per-agent sum and computation peak-by-max, observations8×(clock+1), inspected cells40×(clock+1) in these interior-only cases, learned classifications/known counts, confirmation totals and checked overflow.
- [x] Physical validation: bounds/sorted unique open/nest/food/spoil/advice/agent IDs; exact initial spawn/phase/cargo/advice/counter state from literal observations; worker positions nest-connected; all-cell capacity2; tagged cargo bijection across equal numeric namespaces; food conservation and monotone Available/Carried/Delivered progression (all candidate foods initially Available); spoil carried/disposed/digs equations, unique masked noninitial origins, creator==carrier while carried, direct one-action birth/disposal clocks and no duplicate worker/clock dig-dispose slots; open==initial+spoil origins. Retained prior records' ID/origin/creator/birth never change; disposal cannot reverse. Per-worker moves between frames must cover Manhattan displacement with matching grid parity; persistently carried tags retain their movement category and prohibit intervening handling/dig actions. Verify all per-frame totals against records, Work deposits/digs/pickups/disposals/publications, and advice count+expired==publications. Advice sites may be depleted/duplicate: no omniscient filtering. Check finite exact canonical waypoint strength via public waypoint_strength(rate,age) and WireFloat token, created_tick<completed_ticks. A remembered face may be globally open: validate bounds/immutable mask/empty hands, not current solidity or unseen map state. Never claim private-map truth from counts.
- [x] Access validation uses spoil ID/birth tick/creator to derive birth opportunity `W*born_tick+creator+1`, with W=setup.workers.len() (eight for candidate cases); order/unique slots must hold. Replay only monotone geometry: begin from initial opens, each new origin must adjoin nest-reachable prior opens, then add it and BFS all nest sources once. Compare cached current distances/flags and access calls/visits at each stored prefix. Freeze/verify per-food first-access contexts and distances at the exact opening prefix, including earliest connectivity and lowest-ID aggregate tie; initial flags are not events. Candidate food is initially exposed, so first-exposure remains None; generic component checks derive exposure from initial opens and validate a hidden-food exposure against its opening prefix. Verify milestone geometry/time/work bounds and disposal outlet/distance1; where food handling times are not recorded per token, verify observable bounds rather than invent an exact history. Event-time BFS is researcher validation, never worker work or core replay.
- [x] Add mutated valid-body tests for bad tags/owners/origins, cargo undercount, state reversals, future/old context, missing initial/final/duplicate frames, shortened distance cache or frozen first-distance changes, inconsistent counters/peak overflow, face on opened masked cell accepted, food delivered yet route still counted, and hash-consistent malicious payloads. Component tests may use smaller hand-worked core scenes but full-envelope validation always binds the candidate.
A deterministic small component fixture avoids searching for a favorable excavation seed; use it only for validator tests, not candidate estimates:

```rust
fn component_episode()->core::Episode {
    let pos=|x,y|core::Pos {x,y};
    core::run(core::Setup {
        width:3,height:3,open:vec![pos(0,0),pos(0,1),pos(1,1)],
        diggable:vec![pos(2,1)],nest:vec![pos(0,1),pos(1,1)],waste:pos(0,0),
        workers:vec![pos(1,1)],food:vec![core::Resource {id:0,pos:pos(2,1)}],
        parameters:core::Parameters {p_search:1.0,p_return:0.0,lambda_fidelity:0.0,
            lambda_publish:0.0,lambda_waypoint:0.0},
    },7,core::RunOptions {ticks:16,sample_every:4,snapshots:true}).unwrap()
}
fn action_total(w:&WireWorkCounts)->Result<u64,String> {
    [w.moves,w.digs,w.pickups,w.deposits,w.disposals,w.waits].into_iter()
        .try_fold(0u64,|a,b|a.checked_add(b).ok_or_else(||"work action sum overflow".into()))
}
```

Its one eligible face gives independent Food0/Spoil0 namespace and exposure/access/handling cases; never place it in candidate archives. Do not hardcode W=8 or all-food-exposed in generic physical/access components. Full candidate validation still pins the approved population/input.

- [x] GREEN/fmt/Clippy/full survey on final freeze; fresh review; commit `feat(foraging): validate bounded shortcut evidence without world restoration`.

### Task 3: Immutable archive writer and bounded streaming reader

**Files:** Create archive.rs, io.rs, tests/archive.rs; modify tests/support.rs for owned filesystem fixtures; registrations. Consume validated bytes, not worker simulation. Reuse existing public protection_archive::write_new/new_directory/validate_revision helpers where applicable without modifying them; other F5 helpers stay local.
**Consumes:** Task1 manifest/keys/authorize; Task2 wire codec/observed validators; root provenance/hash types.
**Produces:** RawRef/Index/Limits, writer/reader/read_bounded/checked_total/raw_path contracts. metadata_limit4MiB bounds index/progress reads, independent of raw4MiB and cumulative1GiB.

- [x] Write filesystem/reader REDs using owned temporary directories and existing construction-only fixture bytes. Example:

```rust
#[test]
fn cumulative_raw_budget_is_checked_without_overflow() {
    assert_eq!(checked_total(8,2,10).unwrap(),10);
    assert!(checked_total(8,3,10).is_err());
    assert!(checked_total(u64::MAX,1,u64::MAX).is_err());
}
#[test]
fn draft_scientific_writer_creates_nothing() {
    let tmp=owned_tempdir();let out=tmp.path().join("new");let m=candidate().unwrap();
    assert!(ArchiveWriter::create(&out,&m,CollectionMode::Scientific,&test_provenance(),
        "engineering test",limits(&m)).is_err());
    assert!(!out.exists());
}
```

Define tests' owned_tempdir()->OwnedTempdir in support.rs using std process ID plus AtomicU64 unique suffix, exclusive creation, and Drop cleanup of only its own directory; path()->&Path. Define limits(&Manifest)->Limits in archive.rs from frozen fields.
- [x] Await `cargo test --manifest-path survey/Cargo.toml foraging_shortcuts::tests::archive` RED. Then implement authorization/key/provenance/limit preflight before output creation; initial incomplete index; canonical raw paths; sequential keys; complete-record validation before write; SHA/bytes/progress receipts after successful exclusive write/sync. Publish final index only after a fully written/synced pending index, using a same-directory no-overwrite hard link to index.json; keep the pending copy as declared metadata receipt. Never expose partially written index.json as completed. Shared validate_record checks each saved ref both before final publication and during Reader.next; do not call Reader.open on a pending index or duplicate the validation block. checked_total uses checked_add/filter <=limit. Encode metadata bounded as well. Any rejected/failed put marks writer failed; the caller invokes fail once to record error context; finish only after all exact keys and saved-byte validation, never after a previous error.
- [x] Reader opens only the canonical index.json final completed index, strictly parses WireManifest/identities/key list/ref order, compares complete manifest representation with candidate, checks mode authorization/expected seed set and exact generated relative paths. Reject absolute paths, dot/parent aliases, alternative basenames, symlink files/directories, nonfiles, duplicate paths and escaping canonical roots before record access. Bounded reads use take(limit+1) with checked limit arithmetic and reject excess; UTF-8/JSON errors are contextual. next validates each raw byte count/hash, envelope and observed episode before yielding, increments checked byte count, and verifies final aggregate on None.
- [x] Tests cover existing outputs/symlinks, wrong order/duplicates/missing/extra keys, fake code/protocol/manifest/collector digest, incorrect raw bytes/hash plus re-signed invalid body, incomplete/failed writer, failures injected privately before write/receipt/completion, full/one-byte-short/cumulative budgets and u64 overflow, unsafe/symlink record path, oversized index/record, out-of-range seed and terminal final-total mismatch. No forced overwrite or silent resume. Full success fixtures use the30 construction keys only; cache engine episodes within a test process.
The core byte helpers are concrete standard-library operations; callers add condition/seed/path context:

```rust
fn checked_total(previous:u64,next:u64,limit:u64)->Result<u64,String> {
    previous.checked_add(next).filter(|n|*n<=limit)
        .ok_or_else(||"raw byte budget exceeded or overflowed".into())
}
fn read_bounded(path:&Path,limit:u64)->Result<Vec<u8>,String> {
    use std::io::Read;
    let ceiling=limit.checked_add(1).ok_or_else(||"read bound overflow".to_owned())?;
    let mut bytes=Vec::new();
    std::fs::File::open(path).map_err(|e|e.to_string())?.take(ceiling)
        .read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    if bytes.len() as u64>limit {return Err("input byte limit exceeded".into());}
    Ok(bytes)
}
```

Final index receipt is written/synced completely before hard_link to the final name; readers accept only final index.json, not the pending/incomplete receipts. A failure can leave raw or pending files, but never a successfully published partial completion index.

- [x] GREEN/fmt/Clippy/full survey final freeze; fresh review; commit `feat(foraging): preserve immutable bounded shortcut archives`.

### Task 4: Explicit collector, provenance and standalone CLI

**Files:** Create run.rs, cli.rs, tests/run.rs, survey/tests/foraging_shortcuts_cli.rs; modify tests/support.rs for owned Git/executable fixtures; registrations; modify survey/src/main.rs dispatch/help only for the new selector. Create the full resolved draft-manifest JSON in docs from manifest_bytes after the pure printer is implemented. Existing other-study code/dependencies unchanged.
**Consumes:** Task1 factory/keys/status; Task2 encoder; Task3 writer; known public F4 `run(Setup,u64,RunOptions)->Result<Episode,Vec<FieldError>>`.
**Produces:** ExecutionContext/RunRequest/preflight/collect/Command/parse/cli contracts. Task5 fills only Analyze dispatch after its own tests; Task4 Analyze returns a clear unsupported-yet error, with narrow temporary allowance removed by5.

- [ ] Write parser/denial/provenance REDs first. Example:

```rust
#[test]
fn analysis_rejects_run_or_seed_overrides() {
    let args=["--analyze","index.json","--out","new","--seeds","1"]
        .map(str::to_owned).to_vec();
    assert!(parse(&args).is_err());
}
#[test]
fn collector_rejects_draft_scientific_request_before_output() {
    let tmp=owned_tempdir();let r=RunRequest { mode:CollectionMode::Scientific,
        protocol_revision:"a".repeat(40),approval_context:"test".into(),out:tmp.path().join("new") };
    let ctx=ExecutionContext {repo:tmp.path().to_owned(),executable:tmp.path().join("collector")};
    assert!(collect(&ctx,&r).is_err());assert!(!r.out.exists());
}
```

- [ ] Await run filter RED and `cargo test --manifest-path survey/Cargo.toml --test foraging_shortcuts_cli` RED. Implement strict args: default manifest, --help, --run with optional --construction and required protocol-revision/approval-context/out, or --analyze INDEX with out. Reject unknown/repeated/incompatible flags, missing/empty values, tuning overrides and mixtures of the new selector with existing study selectors before dispatch. Do not modify behavior when the new selector is absent; do not add to generic claims::all(). Exit2 contextual error, no hidden run on help/printing.
- [ ] Native preflight authorizes mode first, validates full40-hex revisions and resolves to canonical lowercase Git commit, tracked-clean repo, exact committed protocol bytes, exact candidate JSON matching factory+HEAD bytes, nonempty approval context, new output (including symlink refusal), executable SHA and manifest SHA. Production context uses manifest's repo root and current executable. Private tests define owned_git_context()->(OwnedTempdir,ExecutionContext,RunRequest), supplying owned temporary Git repos and a fixture executable file; no public repo/provenance override flags. Do not bypass hooks; fixture commits use local test identity only.
- [ ] Collector creates writer after preflight; loops canonical condition then ascending construction seeds; executes full F4 run, encodes original Episode, saves validated bytes, stops with preserved failure evidence on error. Run collection/failure ordering skeleton:

```rust
let m=candidate()?;let p=preflight(ctx,&m,r)?;
let mut writer=ArchiveWriter::create(&r.out,&m,r.mode,&p,&r.approval_context,limits(&m))?;
for key in expected_keys(&m,r.mode) {
    let c=condition(&m,&key.condition)?;
    let result=core::run(c.setup.clone(),key.seed,c.options.clone())
        .map_err(|e|format!("{} seed {}: {e:?}",key.condition,key.seed))
        .and_then(|e|encode_envelope(&key,r.mode,&p,&e,m.raw_record_limit))
        .and_then(|bytes|writer.put(&key,&bytes));
    if let Err(error)=result {
        if let Err(saved_error)=writer.fail(&error) {
            return Err(format!("{error}; also failed to preserve failure evidence: {saved_error}"));
        }
        return Err(error);
    }
}
writer.finish()
```

Preserve primary error if fail-evidence writing also fails. Optional operational timing is outside deterministic raw comparative payload. No scientific collection branch may pass current draft authorization.
- [ ] Generate candidate JSON using the pure default printer, before final freeze, and test byte identity with manifest_bytes. Tests check canonical input/config/seed/options all30 core episodes, full horizons/conservation/identity/null controls without guaranteed positive outcomes; construction/scientific separation; clean/dirty/nonexistent/wrong protocol commit, wrong checkout/committed/factory candidate, malformed revision, existing output and executable errors. External binary tests cover default/help no files/world work, draft --run rejection and selector collision. Owned Git fixture permits in-process genuine collection despite the implementer worktree being uncommitted during tests; the real production preflight retains clean-tree enforcement.
External default-print test uses the existing binary test facility, not a new test dependency:

```rust
#[test]
fn default_shortcut_command_prints_draft_manifest() {
    let output=std::process::Command::new(env!("CARGO_BIN_EXE_survey"))
        .arg("--foraging-shortcuts").output().unwrap();
    assert!(output.status.success());
    let m:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(m["schema"],"foraging-shortcut-manifest-v1");
    assert_eq!(m["status"],"draft");
    assert_eq!(m["execution_authorized"],false);
}
```

Use write_all/flush error propagation for stdout instead of panic on a broken pipe. Test parse/dispatch boundaries structurally and inspect the manifest path's call graph: it must not invoke collection or construct World.

- [ ] GREEN/fmt/Clippy/full survey final freeze including manifest artifact; fresh review; commit `feat(foraging): collect explicit shortcut construction evidence`.

### Task 5: Saved-only rows, paired reports and acceptance

**Files:** Create report_rows.rs, report.rs, tests/report.rs; modify tests/support.rs for complete archive fixtures; modify cli.rs only Analyze dispatch; extend external CLI tests; docs/foraging.md/spec/plan evidence. Remove every staged allowance. Parent owns final review/archive/tracker removal.
**Consumes:** Task1 roles/expected keys; Task2 validated episode/access; Task3 streaming reader/index; existing stats signature `paired_summary(&BTreeMap<u64,f64>,&BTreeMap<u64,f64>)->Result<PairedSummary,String>` with fields n/mean/ci95/positive/zero/negative.
**Produces:** All row/window/contrast/analysis/project_row/analyze/markdown contracts above, deterministic outputs from saved archives.

- [ ] Write saved-only/null-window/paired synthetic REDs before implementation. Example:

```rust
#[test]
fn construction_analysis_keeps_every_zero_and_emits_no_scientific_contrasts() {
    let archive=complete_construction_archive();let out=archive.temp.path().join("analysis");
    let a=analyze(&archive.index,&out).unwrap();
    assert_eq!(a.rows.len(),30);assert!(a.contrasts.is_empty());
    assert_eq!(a.rows.iter().filter(|r|r.key.condition=="access.straight.protected")
        .map(|r|r.summary.food.delivered).collect::<Vec<_>>(),vec![0,0]);
}
```

Define support complete_construction_archive()->TestArchiveFixture, with fields temp:OwnedTempdir,index:PathBuf, via the owned Git+collector context from Task4; construction seeds7/8 only. Test local fixtures cache underlying core Episodes where pure byte-writing validation permits it; real collector integration still calls public F4 run.
- [ ] Await report filter and external CLI acceptance RED. Implement project_row using validated access points and full final WireSummary; min distances over original resources after delivery; Some(d0)-Some(df) checked nonnegative gain; initial None implies gain/window None. First saved route shortening yields (previous completed checkpoint,current checkpoint], never engine processing time. Sampled physical digest serializes clock/open/worker id-position-phase-mode-cargo/work/food/spoil states without condition/input/compute/advice fields; explicitly call it sampled projection, not full trajectory. Preserve all per-resource distances/final milestones/censoring/work metrics.
- [ ] Analyze streams every exact expected key and validates whole archive before creating report output directory; keep bounded rows only. Construction emits all30 rows and controls/censoring, no condition means or scientific contrasts. Draft Scientific archive stays refused. Pure build_contrasts tests use synthetic40-seed rows/maps, never engine runs: three primary delivery differences; six secondary route references; six secondary sealed delivered/access differences; exact matched seed sets, signed means/Student-t intervals/sign counts. Copy existing PairedSummary into serializable PairedStats without altering stats.rs. Reject duplicate/missing seeds or ambiguous row roles; no pooling/threshold verdict.
- [ ] Render stable analysis JSON/Markdown in manifest order, deterministic number formatting, complete seed rows and separate panels/roles. Means/contrasts for future registered scientific use are descriptive; no construction results promoted to scientific estimates. Relative raw references/provenance are stable; output directory and optional elapsed metadata never enter comparative payload. Exclusive output creation returns error on existing paths; failure cannot claim both artifacts completed. Reanalysis invokes no core run and reproduces both files byte-for-byte from identical raw data.
- [ ] Tests mutate saved complete archives (hash mismatches and re-signed bad state), drop/duplicate/extra keys, corrupt cache/event distances, replay clocks/tags/ownership, parser byte budgets and metadata paths; reject before successful output. Seed-test null versus zero, delivered-resource route retention, first-window bounds, coarse projection aliases not labelled trajectories, all-null/tied/negative synthetic contrasts and stable ordering. Reanalyze into two new directories and compare exact outputs; modify only operational metadata and verify comparative outputs unchanged. Compare full core Episodes for Straight identity and core run/repeated-step observational equivalence with construction-only seeds; no scientific example run.
- [ ] Freeze final sources/tests/manifest before required final checks, save/inspect outputs:

```bash
cargo test --manifest-path survey/Cargo.toml
cargo fmt --manifest-path survey/Cargo.toml -- --check
cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings
cargo test --workspace
cargo fmt --all --check
cargo clippy -p sugarscape-core --all-targets -- -D warnings
git diff --check
```

No core source or dependency package change; verify legacy study surfaces and no implicit execution. The approved construction matrix may be exercised as engineering acceptance; scientific seeds remain metadata or synthetic statistics only. Document exact CLI, candidate/unregistered status, row/schema/budget definitions, byte/provenance/validator limits and source gaps. Fresh Task5 review then parent whole-branch review; resolve findings through consolidated scoped corrections with covering reds/greens/re-review. Parent archives own SDD evidence/rulings, removes completed tracker and commits evidence-only bookkeeping. Commit `feat(foraging): analyze saved shortcut evidence deterministically`. No automatic merge/push/scientific run.

## Planning refinements and preflight self-review

A metadata4MiB read/write bound protects index/progress inputs without changing raw budget definitions. Numeric tokens use existing serde_json raw_value rather than changing global float parsing or adding core Deserialize. Deterministic geometry replay from retained spoil birth/creator data verifies first-access distances while preserving the stated coarse-action/controller/RNG limit. Canonical order uses expected-key vectors rather than lexical RunKey ordering. These refinements do not add worker input, scientific execution or new package dependencies.

Coverage ownership: scenarios/full input/controls ->1; strict schemas/bytes/observed legality/cache/event/censor validation ->2; archive/path/hash/budget/failure ->3; provenance/CLI/explicit collection/gates ->4; projections/stats/complete saved-only report/acceptance/docs ->5. Every Review Focus case has its owning tests above. Type names/signatures agree across consumers, including core-only serialization and independent survey workspace checks. Before dispatch, parent scans every task pair's shared types/files and internal assumptions; unresolved architecture issues escalate rather than being guessed. Execution approval is recorded; tasks1–5 are beginning with the baseline/preflight gate.

## Appendix A: Exact local core-output wire fields

Types below mirror the current public core serialization fields only. Every struct receives Clone/Debug/PartialEq/Serialize/Deserialize and deny_unknown_fields; WirePos additionally Copy/Eq/Ord. Work/compute/inventory structs may derive Default. Enums preserve core PascalCase variant names and reject unknown variant fields. Struct fields/types are pub(super); enum fields inherit enum visibility without field qualifiers. No methods/private map/world types are imported or mirrored. WireEpisode and nested DTOs are for saved observations, not restoration.

The exact finite-token implementation is:

```rust
#[derive(Clone,Debug,PartialEq)]
pub(super) struct WireFloat(pub(super) String);
fn finite_token(token:&str)->Result<(),String> {
    if token.len()>64 {return Err("numeric token exceeds64 bytes".into());}
    let value=serde_json::from_str::<f64>(token).map_err(|e|e.to_string())?;
    if !value.is_finite(){return Err("numeric token must be finite".into());}
    Ok(())
}
impl serde::Serialize for WireFloat {
    fn serialize<S:serde::Serializer>(&self,s:S)->Result<S::Ok,S::Error> {
        finite_token(&self.0).map_err(serde::ser::Error::custom)?;
        let raw=serde_json::value::RawValue::from_string(self.0.clone())
            .map_err(serde::ser::Error::custom)?;
        serde::Serialize::serialize(&raw,s)
    }
}
impl<'de> serde::Deserialize<'de> for WireFloat {
    fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error> {
        let raw=<Box<serde_json::value::RawValue> as serde::Deserialize>::deserialize(d)?;
        finite_token(raw.get()).map_err(serde::de::Error::custom)?;
        Ok(Self(raw.get().to_owned()))
    }
}
```

This retains producer lexical tokens without enabling global float_roundtrip behavior. The cached survey dependency serde_json1.0.151 exposes raw_value without a new dependency package. Expected setup/waypoint token comparison remains strict; integer/enum/schema fields retain normal typed decoding.

File allocation: wire.rs owns WirePos/Resource/Parameters/Setup/RunOptions/Episode plus envelope/manifest/float. wire_state.rs owns cargo/phase/mode/material/inventory/work/compute/terrain types. wire_view.rs owns event/access/milestone/find/waypoint/agent/summary/snapshot types. Import sibling DTOs explicitly; the typed data graph is finite, not recursive.

```rust
pub(super) struct WirePos {
    pub(super) x: u32,
    pub(super) y: u32,
}
pub(super) struct WireResource {
    pub(super) id: u64,
    pub(super) pos: WirePos,
}
pub(super) struct WireParameters {
    pub(super) p_search: WireFloat,
    pub(super) p_return: WireFloat,
    pub(super) lambda_fidelity: WireFloat,
    pub(super) lambda_publish: WireFloat,
    pub(super) lambda_waypoint: WireFloat,
}
pub(super) struct WireSetup {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) open: Vec<WirePos>,
    pub(super) diggable: Vec<WirePos>,
    pub(super) waste: WirePos,
    pub(super) nest: Vec<WirePos>,
    pub(super) workers: Vec<WirePos>,
    pub(super) food: Vec<WireResource>,
    pub(super) parameters: WireParameters,
}
pub(super) struct WireRunOptions {
    pub(super) ticks: u32,
    pub(super) sample_every: u32,
    pub(super) snapshots: bool,
}
pub(super) struct WireEpisode {
    pub(super) setup: WireSetup,
    pub(super) seed: u64,
    pub(super) options: WireRunOptions,
    pub(super) summary: WireSummary,
    pub(super) snapshots: Vec<WireSnapshot>,
    pub(super) snapshot_bytes: u64,
}
pub(super) struct WireWorkCounts {
    pub(super) opportunities: u64,
    pub(super) moves: u64,
    pub(super) digs: u64,
    pub(super) pickups: u64,
    pub(super) deposits: u64,
    pub(super) disposals: u64,
    pub(super) waits: u64,
    pub(super) departure_moves: u64,
    pub(super) search_moves: u64,
    pub(super) empty_return_moves: u64,
    pub(super) food_moves: u64,
    pub(super) spoil_moves: u64,
    pub(super) transition_waits: u64,
    pub(super) empty_arrival_waits: u64,
    pub(super) no_neighbor_waits: u64,
    pub(super) empty_congestion_waits: u64,
    pub(super) food_congestion_waits: u64,
    pub(super) spoil_congestion_waits: u64,
    pub(super) search_entries: u64,
    pub(super) empty_returns: u64,
    pub(super) fidelity_departures: u64,
    pub(super) recruited_departures: u64,
    pub(super) uninformed_departures: u64,
    pub(super) publications: u64,
    pub(super) abandoned_targets: u64,
    pub(super) spoil_hauls: u64,
}
pub(super) enum WireCargo {
    Food(u64),
    Spoil(u64),
}
pub(super) enum WireFoodPhase {
    Departing,
    Searching,
    Returning,
}
pub(super) enum WireMode {
    Departing,
    Searching,
    EmptyReturning,
    FoodReturning,
    SpoilHauling,
}
pub(super) struct WireComputeCounts {
    pub(super) observations: u64,
    pub(super) cells_inspected: u64,
    pub(super) cells_learned: u64,
    pub(super) observed_revisions: u64,
    pub(super) dig_confirmations: u64,
    pub(super) face_scans: u64,
    pub(super) route_calls: u64,
    pub(super) route_visits: u64,
    pub(super) peak_queue: u64,
    pub(super) frontier_scans: u64,
}
pub(super) struct WireAccessCompute {
    pub(super) calls: u64,
    pub(super) visits: u64,
    pub(super) peak_queue: u64,
}
pub(super) struct WireTerrainInventory {
    pub(super) initial_open: u32,
    pub(super) open: u32,
    pub(super) excavated: u32,
}
pub(super) enum WireFoodState {
    Hidden,
    Available,
    Carried { agent: u32 },
    Delivered,
}
pub(super) struct WireFoodView {
    pub(super) resource: WireResource,
    pub(super) state: WireFoodState,
}
pub(super) struct WireFoodInventory {
    pub(super) initial: u32,
    pub(super) hidden: u32,
    pub(super) available: u32,
    pub(super) carried: u32,
    pub(super) delivered: u32,
}
pub(super) enum WireSpoilState {
    Carried { agent: u32 },
    Disposed { tick: u32 },
}
pub(super) struct WireSpoilView {
    pub(super) id: u64,
    pub(super) origin: WirePos,
    pub(super) creator: u32,
    pub(super) born_tick: u32,
    pub(super) state: WireSpoilState,
}
pub(super) struct WireSpoilInventory {
    pub(super) excavated: u32,
    pub(super) carried: u32,
    pub(super) disposed: u32,
}
pub(super) struct WireEventContext {
    pub(super) tick: u32,
    pub(super) opportunity: u64,
    pub(super) worker: u32,
    pub(super) excavated: u32,
    pub(super) spoil_disposed: u32,
    pub(super) food_delivered: u32,
}
pub(super) struct WireEventMilestone {
    pub(super) context: WireEventContext,
    pub(super) pos: WirePos,
    pub(super) nest_distance: u32,
}
pub(super) struct WireFoodAccessRecord {
    pub(super) id: u64,
    pub(super) initially_exposed: bool,
    pub(super) initially_accessible: bool,
    pub(super) first_exposure: Option<WireEventMilestone>,
    pub(super) first_access: Option<WireEventMilestone>,
    pub(super) accessible: bool,
    pub(super) distance: Option<u32>,
}
pub(super) struct WireAccessSummary {
    pub(super) initially_exposed: u32,
    pub(super) initially_accessible: u32,
    pub(super) accessible: u32,
    pub(super) records: Vec<WireFoodAccessRecord>,
    pub(super) compute: WireAccessCompute,
}
pub(super) struct WireMilestones {
    pub(super) first_excavation: Option<WireEventMilestone>,
    pub(super) first_exposure: Option<WireEventMilestone>,
    pub(super) first_access: Option<WireEventMilestone>,
    pub(super) first_disposal: Option<WireEventMilestone>,
    pub(super) first_pickup_tick: Option<u32>,
    pub(super) first_delivery_tick: Option<u32>,
    pub(super) all_food_delivered_tick: Option<u32>,
}
pub(super) struct WireFindView {
    pub(super) site: WirePos,
    pub(super) count: u32,
}
pub(super) struct WireWaypointView {
    pub(super) id: u64,
    pub(super) site: WirePos,
    pub(super) created_tick: u32,
    pub(super) strength: WireFloat,
}
pub(super) struct WireAgentView {
    pub(super) id: u32,
    pub(super) pos: WirePos,
    pub(super) phase: WireFoodPhase,
    pub(super) mode: WireMode,
    pub(super) cargo: Option<WireCargo>,
    pub(super) find: Option<WireFindView>,
    pub(super) site: Option<WirePos>,
    pub(super) frontier: Option<WirePos>,
    pub(super) face: Option<WirePos>,
    pub(super) known_open: u32,
    pub(super) known_solid: u32,
    pub(super) known_diggable: u32,
    pub(super) work: WireWorkCounts,
    pub(super) compute: WireComputeCounts,
}
pub(super) struct WireSummary {
    pub(super) completed_ticks: u32,
    pub(super) food: WireFoodInventory,
    pub(super) spoil: WireSpoilInventory,
    pub(super) terrain: WireTerrainInventory,
    pub(super) work: WireWorkCounts,
    pub(super) compute: WireComputeCounts,
    pub(super) per_agent_work: Vec<WireWorkCounts>,
    pub(super) per_agent_compute: Vec<WireComputeCounts>,
    pub(super) expired_records: u64,
    pub(super) access: WireAccessSummary,
    pub(super) milestones: WireMilestones,
}
pub(super) struct WireSnapshot {
    pub(super) summary: WireSummary,
    pub(super) open: Vec<WirePos>,
    pub(super) nest: Vec<WirePos>,
    pub(super) waste: WirePos,
    pub(super) agents: Vec<WireAgentView>,
    pub(super) food: Vec<WireFoodView>,
    pub(super) spoil: Vec<WireSpoilView>,
    pub(super) waypoints: Vec<WireWaypointView>,
}
```

Final plan self-review: all five stages have their own RED/GREEN/review/commit boundary; file ownership includes shared test-support modifications. Twenty-nine core-output schemas are explicit, all wire references resolve, CLI export visibility matches main dispatch, generic component checks derive population while candidate checks pin eight workers, and the compiler-only draft scientific gate cannot be overridden through flags or archive metadata. All Review Focus cases have named task tests. Metadata bounds/token retention/atomic index publication are the disclosed implementation refinements; no unresolved architecture placeholder remains. Written plan review passed; subagent-driven execution is already selected.

## Execution preflight — 2026-10-07

Spec and plan approved; worktree and branch verified clean at `937ba9c`. Parent scanned all ten shared task pairs plus each task internally, preserving the five-stage/source-contract boundaries. One internal archive helper is added: `validate_record(root,index,manifest,reference)->WireEpisode` is shared by Writer.finish and Reader.next, since final-only Reader.open cannot validate an unpublished pending index and duplicating its logic would be fragile. If wrong, rework this small internal interface; no schema, core policy or public surface changes. Detailed baseline/implementation/review evidence belongs to the plan-specific ignored SDD workspace and `/tmp/sugarscape-f5-evidence-20261007/`. No scientific collection or implementation task has completed.

Baseline verification: survey269 passed/0 failed/0 ignored; workspace2,919 passed/0 failed/103 existing ignored. Survey/core formatting and all-target Clippy passed. Full logs, exits, timings and starting source hashes are preserved at `/tmp/sugarscape-f5-evidence-20261007/`. No runtime source changed during preflight.

Task1 runtime `7f1f290`: focused9/survey278 passed,0 failed,0 ignored; survey fmt/all-target Clippy clean on frozen hashes. Independent task review approved with no findings; parent hash audit matched. API-compilation RED and supplemental timing are disclosed. Collector/identity controls belong to4/5, wire/cache/archive validation to2/3 and allowance removal/core workspace regression to5.

Task2 file-boundary ruling: agents.rs owns the existing work/initial_agent/agents logic, with validate_agents(setup:&WireSetup,frame:&WireSnapshot,open:&BTreeSet<WirePos>)->Result<(),String>. physical.rs keeps setup/material/cumulative/frame orchestration and its require/sum/in_bounds/birth helpers; its public internal validate_frames contract is unchanged. Register agents module; existing physical tests cover the extraction. No schema/core/report behavior changes. If wrong, small internal module rework. Strict DTO Option fields require presence (explicit null is valid); deserialize_with avoids serde omission defaults, with observed missing-cargo RED coverage.

### Task 2 verification and review

Runtime `14079bb`; scoped timing correction `aab51bf`. Corrected-tree survey:313 passed,0 failed,0 ignored; focused44,fmt/all-targetClippy clean. The subsequent two-file correction passed45 focused tests (including all30 construction keys),fmt/all-targetClippy with matching frozen hashes; full-suite repetition was unnecessary for that scoped fix. Independent task review found a relative pickup-to-delivery timing gap; scoped re-review approved its correction with no new breakage. One Minor observation about isolated cumulative-guard test coverage remains recorded for final whole-branch triage. Compilation-only lexical RED and corrected invalid/no-op fixture probes are disclosed in the local evidence, rather than counted as assertion REDs. Stage2 complete; later archive/CLI/report requirements remain assigned to stages3–5.

### Task 3 verification and review

Runtime `778eadb`. Frozen focused69/full survey338 passed,0 failed,0 ignored; formatting/all-targetClippy clean. Parent matched all573 Rust/Cargo source hashes across the four final gates. Independent task review approved spec and quality with no findings. Initial compilation-only RED, supplemental shared-validator mutation RED, and an intermediate skip-filter omission are disclosed; final focused/full runs used no skip. Prior Task1/2 producer checks remain reviewed; actual provenance acquisition is Task4, terminal saved analysis and allowance removal are Task5.
