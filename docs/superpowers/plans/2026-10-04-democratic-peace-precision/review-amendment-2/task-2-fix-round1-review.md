### Spec Compliance

- ✅ Scoped fix compliant. The original vacuous-test finding is addressed: `survey/democratic_peace/test_followup_runtime.py:52-108` builds complete retained evidence, obtains an actual runtime attestation, creates a full review, commits the normative inventory and declaration in a temporary repository, and invokes the real `verify_activation`. Both phases pass the valid baseline at `test_followup_runtime.py:109-112`.
- ✅ The negatives reach the intended guards: external declaration/runtime/benchmark hashes at `test_followup_runtime.py:114-123`, all five phase binding hashes at `test_followup_runtime.py:125-130`, review independence at `test_followup_runtime.py:132-136`, and a coherent overbudget forecast with the review hash rebound at `test_followup_runtime.py:138-152`.

### Strengths

- The test keeps its fixture Git commits and artifacts in temporary directories (`test_followup_runtime.py:52-102`), leaving production behavior and the retained scientific evidence untouched.
- Retained focused and full logs show five and 80 tests passing, respectively, with no warnings; the whitespace check passed. Controlled in-memory guard removals made all four external-hash and all five phase-hash subtests fail specifically because the expected `ValueError` was absent. See `.superpowers/sdd/2026-10-04-democratic-peace-precision/task-2-fix-round1-focused.log`, `task-2-fix-round1-full.log`, and `task-2-fix-round1-mutation.json`. I did not rerun covering suites.

### Issues

- None in this scoped fix. No new breakage is visible in the one-file test diff or retained checks.

### Assessment

**Task quality:** Approved.

**Reasoning:** The complete valid chain and targeted negative cases now exercise production activation behavior. This resolves the prior test-quality block without modifying production code; broader scientific preparation and native integration remain outside this re-review.
