# Burrow measured harness implementation tracker

Plan: docs/superpowers/plans/2026-10-04-burrow-1-measured-harness.md

## Stage 1: Validate saved physical episodes
**Goal**: Checked deterministic reconstruction without RNG or controller stepping.
**Success Criteria**: Accepted valid construction records and rejected forged transactions, round schedules and summaries.
**Tests**: `cargo test -p sugarscape-core burrow::tests::validation`
**Status**: Complete

## Stage 2: Declare manifest and non-running route
**Goal**: Strict full candidate identity and default manifest-only survey route.
**Success Criteria**: 1280 scientific keys and 36 construction keys, explicit parameters, no implicit execution.
**Tests**: `cargo test --manifest-path survey/Cargo.toml burrow_manifest`
**Status**: Complete

## Stage 3: Archive immutable raw episodes
**Goal**: Exclusive provenance/hash-bound writes and strict complete-archive loading.
**Success Criteria**: Construction archives complete; missing/duplicate/extra/tampered/escaping raw records reject.
**Tests**: `cargo test --manifest-path survey/Cargo.toml burrow_archive`
**Status**: Complete

## Stage 4: Analyze saved evidence
**Goal**: Deterministic paired reports with exact reductions and censoring.
**Success Criteria**: Byte-identical reanalysis, six contrasts, complete diagnostics, no scientific execution.
**Tests**: `cargo test --manifest-path survey/Cargo.toml burrow_report`
**Status**: In Progress
