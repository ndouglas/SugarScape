**Require equality between the live capital counter and its sovereign generation — ADDRESSED.** `crates/sugarscape-core/src/democratic_peace/territory.rs:194` now rejects either direction of mismatch using `!=`, and the error at `:196` identifies the state generation, capital cell, and counter. The comparison remains scoped to the live state's capital, so it introduces no equality requirement for occupied noncapital cells. The new corruption case at `tests.rs:848` sets live capital cell 0's counter to 1 while its sovereign remains generation 0 and requires validation to reject it. The existing inverse-mismatch case at `tests.rs:834` still requires rejection.

### New Breakage in the Fix Diff

None. The only production change strengthens the live-capital comparison and updates its error context. The existing inverse-case assertion is updated to the new diagnostic; the added counter-ahead fixture directly exercises the residual defect.

### Out-of-Scope Observations

None.

### Verification Evidence and Limits

Read the supplied `review-39d4164..4f6e791.diff` once, the Task 1 brief, prior fix review, appended Fix loop 2 report, and scoped re-review template. Read narrow source context for line references and the existing census rollback connection. No tests were rerun, no subagents were dispatched, and no Git commands or checkout/index/HEAD mutations were performed. Only this requested review artifact was written.

Verified all six named fix-round logs against the amended code:

- `/tmp/dp-task1-fix2-red.log` names `census_rejects_state_key_and_generation_counter_mismatches` and shows its failure at `tests.rs:850` because `unwrap_err()` received `Ok(())`; result: 0 passed, 1 failed.
- `/tmp/dp-task1-fix2-census.log` shows that same focused test passing; result: 1 passed, 0 failed.
- `/tmp/dp-task1-fix2-domain.log` shows all 53 domain tests passing, including the amended generation fixture, structural release fixtures, and `census_corruption_invalidates_candidate_without_advancing_world`.
- `/tmp/dp-task1-fix2-golden.log` shows 8 passed, 3 existing ignored, including democratic-peace reproducibility and unchanged other-model fixtures. No golden constants changed in the fix diff.
- `/tmp/dp-task1-fix2-clippy.log` records successful completion with no warning/error diagnostics.
- `/tmp/dp-task1-fix2-fmt.log` is empty, consistent with a successful format check; its exit status is not independently retained.

The new counter-ahead case tests validation directly rather than adding a second case-specific world rollback test. The unchanged `world.rs:369` propagates that validator error from census, and the existing passing rollback fixture at `tests.rs:781` verifies a census rejection preserves the engine and completed-period count. Together these cover rejection and the unchanged commit gate without requiring another suite execution.

### Verdict

**Fix round: All findings addressed, no new Critical/Important breakage.** The sole residual Important finding from fix round 1 is closed.
