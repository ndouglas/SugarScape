### Spec Compliance

- ❌ Issues found: post-claim alliance pools can disagree with surviving membership (`crates/sugarscape-core/src/democratic_peace/claims.rs:143`); the census validator does not implement all expressly required ownership, identity, membership, and front-topology invariants (`crates/sugarscape-core/src/democratic_peace/territory.rs:151`). These are Task 1 obligations, not host/protocol work.
- ✅ All files enumerated in the task brief have corresponding changes. The implementation introduces the separate model without modifying GeoSim/polarity mechanics or existing golden constants (`crates/sugarscape-core/src/model.rs:77`, `crates/sugarscape-core/tests/golden.rs:561`).
- ⚠️ Cannot verify from this diff: native/WASM parity, host presentation of source-reading explanations, scientific recorder/inference integrity, registered populations, runtime limits, and publication/deployment. The controller should verify these in their assigned later tasks; they are not failures of this task-scoped gate.

### Strengths

- Printed/prose probability direction, analytic zero cases, finite-positive-denominator overflow rejection, and stable software log/exp evaluation are explicit (`crates/sugarscape-core/src/democratic_peace/resources.rs:3`). The allocation code uses eligible distinct sovereign neighbors and old commitment/action buffers, including the repeated-first-inactive alternative (`crates/sugarscape-core/src/democratic_peace/decisions.rs:9`).
- The engine keeps allocation, alignment, decisions, interaction, pre-claim extraction, structure, and census in the approved order. State and RNG are cloned together, with both error and panic paths retaining the committed engine (`crates/sugarscape-core/src/democratic_peace/world.rs:168`, `crates/sugarscape-core/src/democratic_peace/world.rs:302`). Focused fixtures exercise late structural failure and candidate panic rollback (`crates/sugarscape-core/src/democratic_peace/tests.rs:214`, `crates/sugarscape-core/src/democratic_peace/tests.rs:242`).
- Claims derive the complete release footprint before testing locks; ownership is read directly while claims are processed, avoiding stale membership buffers. Release uses checked generation increments and fresh resource endowments (`crates/sugarscape-core/src/democratic_peace/claims.rs:9`, `crates/sugarscape-core/src/democratic_peace/claims.rs:83`). The prospective-footprint test exercises the particularly important locked-agent/disconnection case (`crates/sugarscape-core/src/democratic_peace/tests.rs:705`).
- Alliance formation uses exact threat comparisons and deterministic ordering. Current-plan obligations read one fixed action snapshot, avoiding recursive cascades (`crates/sugarscape-core/src/democratic_peace/alliances.rs:29`, `crates/sugarscape-core/src/democratic_peace/alliances.rs:116`). Combat preserves mutual-only resolution and records opposing independent claims (`crates/sugarscape-core/src/democratic_peace/combat.rs:8`).
- Statistics distinguish legitimate undefined clustering, absent-regime sizes, first extinction, and final survival; isolated democracy exposure is one. Grouping/event limits are excluded from economic fingerprints, while the RNG cursor remains included (`crates/sugarscape-core/src/democratic_peace/world.rs:269`, `crates/sugarscape-core/src/democratic_peace/world.rs:368`, `crates/sugarscape-core/src/democratic_peace/world.rs:379`). Grouped-horizon equality has a behavioral fixture (`crates/sugarscape-core/src/democratic_peace/tests.rs:59`).

### Issues

#### Critical (Must Fix)

- None found.

#### Important (Should Fix)

1. **Recompute surviving alliance pools after structural pruning.** `crates/sugarscape-core/src/democratic_peace/claims.rs:143` removes dead/nonadjacent members and retains groups of at least two, but never updates `pooled_resources`. For example, a three-member alliance with directional commitments 1/1/1 retains a pool of 3 after one member is conquered, even though its surviving commitments sum to 2. `alliances.rs:96` establishes the sum correctly during alignment, but structural changes invalidate it before the committed state is exported. Inspection, state JSON, and final-state hashing then describe an internally inconsistent alliance. The next alignment recomputes the pool, so this finding does **not** claim that the stale number is reused for next-period attack decisions. Recompute the pool from the retained membership/fronts after pruning, and add a fixture that conquers one member of a three-member alliance while leaving the other two valid.

2. **Implement the promised complete census invariant gate.** `crates/sugarscape-core/src/democratic_peace/territory.rs:151` checks capital ownership/regime, resource finiteness, and only the *length* of reachable territory versus `members`; it does not check that each cell has a live owner, that cell IDs match their fixed indices, that each state's ID matches its map key/generation, that the actual sorted member set equals owned cells, or that fronts exactly match territorial adjacency with matching live endpoint identities. A wrong same-length membership list, a mismatched `State.id`, or a missing territorial front can pass this gate. `world.rs:368` relies on it immediately before committing a successful scientific period, and the approved census requirement explicitly names ownership, IDs, fronts, and capital membership. Add direct set/identity/topology checks with descriptive entity context and focused corruption fixtures proving invalid candidates cannot commit. This is a missing validation requirement; I have not established that normal current transitions generate these corruptions.

#### Minor (Nice to Have)

1. **Make large JSON export expressions reviewable.** `crates/sugarscape-core/src/democratic_peace/view.rs:199`, `crates/sugarscape-core/src/democratic_peace/world.rs:278`, and `crates/sugarscape-core/src/democratic_peace/world.rs:281` encode substantial public payloads in single physical lines. Expand the object entries or use small explicit serialization records so future host/science changes can be reviewed field by field without changing the payload contract.

### Assessment

**Spec compliance:** Needs fixes.

**Task quality:** Needs fixes.

**Reasoning:** The independent mechanics are well separated, and the important formula, timing, rollback, locking, and metric distinctions match the approved design. The stale committed alliance pool and incomplete mandated census gate should be corrected before using this engine as the basis for scientific registration.

### Checks and Review Limits

- Read the supplied `review-df21b02..64636d7.diff` in consecutive chunks once for code review; used a mechanical diff line-number lookup afterward. No changed source file was read separately, no broader source crawl was performed, and no tests were rerun.
- Read the task brief, constraints, implementer report, and approved design. Re-read the design mechanics section because the initial combined tool output truncated it; the implementer report itself was visible in full.
- Inspected the existing `/tmp/dp-task1-{focused,golden,core,fmt,clippy,workspace}.log` evidence. All six files exist; no warning/error/panic/FAILED matches were present. Golden evidence reports 8 passed and 3 ignored; clippy reports successful completion. The controller's existing full test executions were not repeated.
- No source checks outside the diff were needed. No product, index, HEAD, or branch changes were made; only this explicitly requested review artifact was written.
