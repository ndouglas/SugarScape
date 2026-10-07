### Spec Compliance

- ❌ Issues found: The brief requires complete tests for mixed activation identities and the noncircular declaration → runtime attestation → independent review gate. `survey/democratic_peace/test_followup_runtime.py:49-52` never calls `verify_activation`; its two `ValueError` assertions use incomplete dictionaries that fail required-field validation before the intended conditions are checked. The production chain is present at `survey/democratic_peace/followup_registration.py:106-116`, but this required behavior lacks a meaningful test.
- ⚠️ Cannot verify from this task diff: Native enforcement of declaration, review, runtime and historical bindings before output writes, and native Phase P literal census validation, live in Task 1's Rust changes. Final branch review should check their integration with the arguments forwarded at `survey/democratic_peace/followup_run.py:52-61`. Applicability of the separately approved validate-only alias fix to the retained scientific binary is a future declaration-review question, outside this Task 2 diff.

### Strengths

- The exact fixed probe coordinates, seeds and both phase identities are declared in `survey/democratic_peace/followup-probe-schedule.json:1-64`; retained validation reconstructs the expected manifests, checks native rows and logs, and rejects aliased artifacts at `survey/democratic_peace/followup_evidence.py:238-270,289-310`.
- The build context and explicit flags/features are validated and passed into probe receipts at `survey/democratic_peace/followup_evidence.py:44-109` and `survey/democratic_peace/followup_runtime.py:204-213`. Selected Python is launched before preparation writes at `survey/democratic_peace/followup_runtime.py:187-225`.
- Both canonical phase rosters and 21,600 attempts are validated before declaration at `survey/democratic_peace/followup_registration.py:65-80`; the declaration, attestation and external review hashes are checked at `survey/democratic_peace/followup_registration.py:106-116`.
- Named focused check, normative inventory completeness: `survey/democratic_peace/provenance.py:44-69` includes every `.py` and `.json` in this package, so the new runtime and activation modules enter the source inventory. No other unchanged source was inspected.
- Retained logs show the expected red run (64 tests, four failures and three errors), clean green run (76 tests), exact checkpoint 6, and successful real retained-evidence validation; see `.superpowers/sdd/2026-10-04-democratic-peace-precision/task-2-red.log`, `task-2-green.log`, `task-2-exact-checkpoint.log`, and `task-2-retained-validation.json`. I did not rerun the suite or actual probes.

### Issues

#### Important (Should Fix)

- **Plan-mandated activation test is vacuous** — `survey/democratic_peace/test_followup_runtime.py:49-52`. Required-field validation rejects both truncated inputs regardless of the `independent` flag or budget. The test cannot catch a regression in `verify_activation` that accepts a well-formed review tied to another declaration or runtime receipt. Construct complete valid chain fixtures, then vary one binding, review independence, and forecast budget in separate assertions; verify failure for the intended reason.

### Assessment

**Task quality:** Needs fixes.

**Reasoning:** The implementation and retained real validation provide a credible Task 2 evidence and activation path, with original engine/math untouched by the nine-file diff. The mandated mixed-identity activation test does not exercise that path, so the task is not ready to pass its review gate.
