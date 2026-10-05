# F2 implementation stages

Plan: docs/superpowers/plans/2026-10-05-foraging-2-fixed-world.md
Approved: 2026-10-05. Execution: subagent-driven development.

## Stage 1: Checked setup and angular movement
**Goal**: Validated geometry and explicit-draw movement helpers.
**Success Criteria**: Setup errors aggregate; headings and movement remain bounded; sampling conventions are tested.
**Tests**: setup_movement unit tests; core suite, fmt and clippy.
**Status**: Complete

## Stage 2: Conserved resources and accounting
**Goal**: Identity-preserving claim/deposit ledger and agent work types.
**Success Criteria**: One claim/deposit per resource; frozen local counts; ownership invariants.
**Tests**: ledger unit tests and foraging regression tests.
**Status**: Complete

## Stage 3: Nest server decisions
**Goal**: Publication, decay and private/recruited departures.
**Success Criteria**: Publication precedes selection; stale advice permitted; independent draws and correct expiry.
**Tests**: server unit tests and F1 regression tests.
**Status**: In Progress

## Stage 4: Ordered atomic ticks
**Goal**: Seeded state transitions with rollback of state and RNG on failure.
**Success Criteria**: Exact delay/detection ordering; resource conservation; sequential visibility and bounded stepping.
**Tests**: controller timing, contention, rollback and invariant tests.
**Status**: Not Started

## Stage 5: Runner and acceptance
**Goal**: Bounded runs, read-only snapshots, replay and public documentation.
**Success Criteria**: Fixed horizon; output byte cap; sample cadence cannot alter behavior; final review approved.
**Tests**: runner and public acceptance tests; cargo fmt, core clippy and cargo test --workspace.
**Status**: Not Started
