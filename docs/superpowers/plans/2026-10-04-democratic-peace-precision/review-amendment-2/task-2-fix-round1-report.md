# Task 2 fix round 1

Status: committed, ready for independent review.
BASE: 9f9a266. HEAD: f225a3d28c4b5bf7779afed7141cd4617d12b586.

Replaced the vacuous truncated-dictionary activation test with five complete-chain characterization tests in test_followup_runtime.py only. Fixtures reuse the existing synthetic retained-evidence builder (unused 800/810 million seed ranges), generate a real runtime_attestation from all bound artifacts, generate complete independent review fields, and commit exact normative inventory and declaration bytes into a temporary Git repository. Both phases successfully pass actual verify_activation, including source inventory and committed-byte checks; no binding validator is mocked.

Negative cases vary four external receipt hashes and all five phase binding hashes independently, set a complete review's independent flag false, and supply a coherent overbudget phase forecast with an updated total and rebound review hash. Assertions check the specific identity, independence or scheduling rejection message. No production behavior change was required.

## Verification

These are passing characterization tests of already-correct production behavior, not a fabricated implementation RED/GREEN cycle.

- Focused: exit 0, five tests passed in 34.424 seconds.
- Full Python discovery: exit 0, 80 tests passed in 92.492 seconds (concurrent mutation checking increased elapsed time).
- Controlled mutation: two individual guard removals performed only in temporary in-memory function namespaces. Missing external-chain guard caused all four mismatch subtests to fail with ValueError not raised; missing phase-binding guard caused all five phase mismatch subtests to fail identically. Zero errors; mutation wrapper exit 0 confirms both guards are meaningfully tested. Product source files never held mutated code.
- Whitespace check: exit 0.
- Commit: exit 0, only test_followup_runtime.py, 105 additions and four removals.

Actual outputs and exit codes are retained in task-2-fix-round1-*.log/.json. Exact test-only diff: task-2-fix-round1-tests.patch.

## Self-review

Verified valid baseline activation for both phases precedes rejection characterization, complete structures reach the intended guards, review hashes are rebound in the budget test, and mutation failures identify missing enforcement rather than malformed fixture fields. Fixture Git commits occur only in auto-cleaned temporary repositories. No registered worlds, actual probes, builds, empirical analyses, retained scientific evidence rewrites, original packet edits or amendment edits ran. No material concerns remain; independent review and controller amendment2 are pending.

Final status:

```
?? IMPLEMENTATION_PLAN.md
?? docs/superpowers/plans/2026-10-04-democratic-peace-precision.md
?? docs/superpowers/plans/2026-10-04-democratic-peace-precision/
```
