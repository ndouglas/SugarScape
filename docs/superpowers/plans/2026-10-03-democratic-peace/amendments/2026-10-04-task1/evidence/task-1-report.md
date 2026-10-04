# Task 1 report: independent 2001 source engine

Status: DONE. Implemented only the approved Task 1 patches through replay checkpoint 2 on branch `democratic-peace`. No scientific population or runtime probe runs were performed.

## TDD and replay evidence

- Created the four-stage `IMPLEMENTATION_PLAN.md` before code application.
- Applied checkpoint 1 with `replay.py --target . --through 1`, then ran the discovery integration test. The actual RED was 3 failing tests with `unknown model "democratic_peace"`, as expected before discovery wiring. Full transcript: `/tmp/dp-task1-discovery-red.log`. This is discovery/interface RED only; it is not reported as behavioral characterization.
- Applied checkpoint 2 with `replay.py --target . --through 2`. Replay verified 578 product files and 897 protected files, including bytes and Git modes.
- Post-implementation focused run: `CARGO_TARGET_DIR=/Users/nathan/.config/superpowers/worktrees/SugarScape/democratic-peace-scratch/target cargo test -p sugarscape-core democratic_peace`: 47 domain tests and 3 discovery tests passed. Log: `/tmp/dp-task1-focused.log`.
- Golden run: `CARGO_TARGET_DIR=.../target cargo test -p sugarscape-core --test golden`: 8 passed, 3 existing print helpers ignored. Log: `/tmp/dp-task1-golden.log`.
- Full core run: `CARGO_TARGET_DIR=.../target cargo test -p sugarscape-core`: passed, including unchanged prior suites. Log: `/tmp/dp-task1-core.log`.
- `cargo fmt --all -- --check`: passed. Log: `/tmp/dp-task1-fmt.log`.
- `CARGO_TARGET_DIR=.../target cargo clippy -p sugarscape-core --all-targets -- -D warnings`: passed with no warnings. Log: `/tmp/dp-task1-clippy.log`.
- One full workspace run, `CARGO_TARGET_DIR=.../target cargo test --workspace`: passed for CLI, core, WASM and all workspace integration/doc tests. Log: `/tmp/dp-task1-workspace.log`.
- `git diff --check`: passed. Self-review confirmed the patch adds only democratic-peace core modules, model discovery/preset/title/golden wiring and this task ledger; no TODO/FIXME/unimplemented markers were found in the new core or discovery test files.

Rust commands used the per-command scratch target path required by the task. No persistent environment change or production build was made.

## Files

Added `crates/sugarscape-core/src/democratic_peace/{alliances,claims,combat,config,decisions,mod,presets,resources,stats,territory,tests,types,view,world}.rs` and `crates/sugarscape-core/tests/democratic_peace_discovery.rs`. Updated `crates/sugarscape-core/src/{lib,model,presets,titles}.rs` and `crates/sugarscape-core/tests/golden.rs`. Added/updated `IMPLEMENTATION_PLAN.md`; Stage 1 is Complete and Stage 2 is In Progress.

## Concerns

No implementation or verification failures remain. These fixtures establish the approved reconstruction behavior and deterministic seed-1/ten-period fingerprints; they do not establish original executable identity or scientific compatibility. Stages 2–4 remain outside this task.

## Fix loop 1 (FIX_BASE 64636d7)

Addressed the two Important review findings; the deferred JSON formatting Minor remains untouched.

### RED → GREEN

Added the pool-pruning, exact-member-set, and candidate rollback fixtures before changing production code. `/tmp/dp-task1-fix-red.log` records three expected failures: the post-pruning pool stayed 3 instead of 2, same-length incorrect membership passed validation, and a corrupt candidate was committed. Existing tests otherwise passed at that checkpoint.

The structural-pruning fix recomputes each surviving alliance pool from retained members and their current directional commitments. The census gate now checks grid width/cell layout, cell index and live ownership, state map key versus stored ID, capital identity, state generation against the cell counter, exact row-major member ownership, connectivity, front endpoint/key identity, exact territorial front topology, and alliance identity/membership/pool consistency. The rollback fixture confirms invalid candidates fail at census without changing the committed engine or period.

### Final verification

- `CARGO_TARGET_DIR=.../target cargo test -p sugarscape-core democratic_peace`: 53 domain tests passed; matching democratic-peace preset fingerprint passed. Log: `/tmp/dp-task1-fix-focused-all.log`.
- `CARGO_TARGET_DIR=.../target cargo test -p sugarscape-core --test democratic_peace_discovery`: 3 passed. Log: `/tmp/dp-task1-fix-discovery.log`.
- `CARGO_TARGET_DIR=.../target cargo test -p sugarscape-core --test golden`: 8 passed, 3 existing ignored; existing other-model fixtures passed unchanged. Log: `/tmp/dp-task1-fix-golden.log`.
- `cargo fmt --all -- --check`: passed. Log: `/tmp/dp-task1-fix-fmt.log`.
- `CARGO_TARGET_DIR=.../target cargo clippy -p sugarscape-core --all-targets -- -D warnings`: passed. Log: `/tmp/dp-task1-fix-clippy.log`.
- `git diff --check`: passed. No full workspace rerun was needed for this core-only fix loop.

The corrected final-state hashes changed four democratic-peace fixtures because alliance pools are part of the canonical state. Unchanged values are retained here as well:

| Fixture | Before FIX_BASE | Corrected |
|---|---:|---:|
| democratic-peace-printed-2001 | 0x51c87aa05ae89971 | 0x3388ec029f1bf609 |
| democratic-peace-prose-probability | 0x10578253f9fff482 | 0x10578253f9fff482 |
| democratic-peace-tagging | 0x3c0c1a219cbe94cc | 0x3c0c1a219cbe94cc |
| democratic-peace-alliances | 0x20e6355f23f1e9c4 | 0x4da0cabf985c6040 |
| democratic-peace-collective-security | 0x51c87aa05ae89971 | 0x3388ec029f1bf609 |
| democratic-peace-nondemocratic | 0x444b1ebb87249f09 | 0xc68d58478b6b238d |

Changed product files: `crates/sugarscape-core/src/democratic_peace/claims.rs`, `territory.rs`, `tests.rs`, and `crates/sugarscape-core/tests/golden.rs`. Self-review confirms no edits to other-model golden constants or any host/protocol/scientific files. Fix commit: `39d4164` (`fix: validate democratic peace state after conquest`).

## Fix loop 2 (FIX_BASE 39d4164)

Closed the residual Important census finding: only a live capital's generation counter must equal its current sovereign ID generation. Counters on occupied noncapital cells may remain higher after release and are intentionally not compared with their current owner generation. The error names the state, its generation, the capital cell, and the counter value.

### RED → GREEN

Added the counter-ahead case to `census_rejects_state_key_and_generation_counter_mismatches`. Before the fix, `cargo test -p sugarscape-core census_rejects_state_key_and_generation_counter_mismatches --lib` failed because `validate()` returned `Ok(())` for cell 0's live generation 0 with counter 1. RED log: `/tmp/dp-task1-fix2-red.log`. After changing the capital check from less-than to equality, the focused census test passed; log: `/tmp/dp-task1-fix2-census.log`.

### Final verification

- `CARGO_TARGET_DIR=.../target cargo test -p sugarscape-core democratic_peace --lib`: 53 passed. Log: `/tmp/dp-task1-fix2-domain.log`.
- `CARGO_TARGET_DIR=.../target cargo test -p sugarscape-core --test golden`: 8 passed, 3 existing ignored. Log: `/tmp/dp-task1-fix2-golden.log`.
- `cargo fmt --all -- --check`: passed. Log: `/tmp/dp-task1-fix2-fmt.log`.
- `CARGO_TARGET_DIR=.../target cargo clippy -p sugarscape-core --all-targets -- -D warnings`: passed. Log: `/tmp/dp-task1-fix2-clippy.log`.

Only `territory.rs` and `tests.rs` changed in this loop. No golden values changed. Fix commit: `4f6e791` (`fix: require exact live sovereign generation`).
