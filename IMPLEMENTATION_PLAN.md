# F3 Passage Implementation Tracker

Approved spec and plan: 2026-10-06. Execution: subagent-driven. Detailed evidence lives in docs/superpowers/plans/2026-10-06-foraging-3-passage.md.

## Stage 1: Checked setup and local learning
**Goal**: Validated chambers, spawns, food and bounded one-hop private knowledge.
**Success Criteria**: Aggregate validation, no second-hop leaks, immutable geometry and checked metrics.
**Tests**: `cargo test -p sugarscape-core foraging::passage::tests::setup_learning`
**Status**: Not Started

## Stage 2: Private navigation
**Goal**: BFS/frontiers based only on each worker map.
**Success Criteria**: Detours and unknown targets use private topology; congestion retains route commitments.
**Tests**: `cargo test -p sugarscape-core foraging::passage::tests::navigation`
**Status**: Not Started

## Stage 3: Food and nest advice
**Goal**: Conserved food identities, agent accounting and F1 nest decisions.
**Success Criteria**: Cargo ownership and inventory agree; self-visible/stale advice and checked counters.
**Tests**: `cargo test -p sugarscape-core foraging::passage::tests::ledger_server`
**Status**: Not Started

## Stage 4: Atomic ticks
**Goal**: Worker-only phase decisions and ordered physical transactions.
**Success Criteria**: Separate pickup/deposit, one action per opportunity and complete state/RNG rollback.
**Tests**: `cargo test -p sugarscape-core foraging::passage::tests::controller`
**Status**: Not Started

## Stage 5: Bounded runner and acceptance
**Goal**: Replay inputs, summaries, snapshots, read-only knowledge and public documentation.
**Success Criteria**: Budget boundaries, observer purity, replay and independent whole-branch review pass.
**Tests**: `cargo test --workspace; cargo fmt --all --check; cargo clippy -p sugarscape-core --all-targets -- -D warnings`
**Status**: Not Started
