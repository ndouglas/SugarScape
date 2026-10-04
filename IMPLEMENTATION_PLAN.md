# Burrow 1 implementation stages

Detailed plan: [Burrow 1](docs/superpowers/plans/2026-10-03-burrow-1-excavation.md).

## Stage 1: Checked world and material actions
**Goal**: Construct checked fixtures and resolve excavation, movement and spoil transactions.
**Success Criteria**: Exact action cases and material/geometry conservation pass.
**Tests**: cargo test -p sugarscape-core burrow::tests::material
**Status**: Complete

## Stage 2: Local observations and controllers
**Goal**: Compare direct/relay transport and cue-blind/responsive local decisions.
**Success Criteria**: Local sensing, exact choice weights, freshness and relay legs pass.
**Tests**: cargo test -p sugarscape-core burrow::tests::controller
**Status**: Not Started

## Stage 3: Deterministic episodes and accounting
**Goal**: Execute worker opportunities and export replay, work and censored delivery records.
**Success Criteria**: Same seed reproduces full records; diagnostics leave actions unchanged.
**Tests**: cargo test -p sugarscape-core burrow::tests::runner
**Status**: Not Started

## Stage 4: CLI acceptance and WASM parity
**Goal**: Expose checked CLI replay and verify native/WASM full-record agreement.
**Success Criteria**: CLI exports and errors, four treatments, parity and repository checks pass.
**Tests**: cargo test --workspace; focused Vitest parity; cargo fmt and clippy; web build.
**Status**: Not Started
