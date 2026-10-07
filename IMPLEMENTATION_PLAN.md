# F4 Construction Implementation Tracker

Spec/plan approved 2026-10-06. Subagent-driven execution. Detailed evidence: docs/superpowers/plans/2026-10-06-foraging-4-construction.md.

## Stage 1: Terrain and local learning
**Goal**: Validated indexed terrain and dynamic private classifications.
**Success Criteria**: Stale walls remain private; legitimate revisions/own digs and counters are atomic.
**Tests**: `cargo test -p sugarscape-core foraging::construction::tests::setup_learning`
**Status**: Complete
**Evidence**: Task 1 implementer: focused 12 passed; workspace 2726 passed/0 failed/103 ignored; fmt/core all-target Clippy clean. Full logs/report: `.superpowers/sdd/2026-10-06-foraging-4-construction/task-1-report.md`. Parent review approved after scoped signature/fixture correction; final focused 13 passed.

## Stage 2: Private navigation
**Goal**: Known-open routes, dig faces and unknown-outlet exploration.
**Success Criteria**: No global route inputs; stable ranks/draws and capacity waits.
**Tests**: `cargo test -p sugarscape-core foraging::construction::tests::navigation`
**Status**: Complete
**Evidence**: Task 2 implementer: focused 30 passed; workspace 2757 passed/0 failed/103 ignored; fmt/core all-target Clippy clean on frozen final revision. Full logs/report: `.superpowers/sdd/2026-10-06-foraging-4-construction/task-2-report.md`. Parent independent review approved with no findings.

## Stage 3: Materials and access
**Goal**: Typed hands, conserved ledgers/advice and researcher access records.
**Success Criteria**: No namespace collision; exposure/access/delivery separate; provenance and cache checked.
**Tests**: `cargo test -p sugarscape-core foraging::construction::tests::material_access`
**Status**: Complete

## Stage 4: Atomic coupled ticks
**Goal**: Cargo-first worker policy and ordered physical transactions.
**Success Criteria**: Paused food intent, one action, and complete terrain/ledger/observer/RNG rollback.
**Tests**: `cargo test -p sugarscape-core foraging::construction::tests::controller`
**Status**: Not Started

## Stage 5: Output and acceptance
**Goal**: Bounded public views/run and compatible no-dig F3 reduction.
**Success Criteria**: Replay/purity/storage/budget checks and task/whole-branch reviews pass.
**Tests**: `cargo test --workspace; cargo fmt --all --check; cargo clippy -p sugarscape-core --all-targets -- -D warnings`
**Status**: Not Started
