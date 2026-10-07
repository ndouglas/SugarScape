### Spec Compliance

- ✅ Spec compliant within the scoped correction: the original Important validate-only input-protection finding is addressed. The schema-2 validate-only guard rejects all nine follow-up gate options before receipt loading, destination validation or output writes (`survey/src/bin/democratic_peace.rs:1449`). Rejecting unsupported gate arguments is the remedy explicitly allowed in the original review.
- ✅ No new breakage identified in this fix: the guard applies only when both schema 2 and validate-only are selected, so ordinary validation, schema 1, and schema-2 execution retain their existing paths (`survey/src/bin/democratic_peace.rs:1449`). The retained covering run includes the existing successful two-phase resolution and schema-1/native integration checks (`task-1-fix-round1-native.log:22`).
- ⚠️ Cannot verify from this scoped diff: applicability of the deliberately untouched scientific scratch binary to eventual measurement remains the controller's independent prospective declaration gate, as instructed. External activation integration remains Task 2 scope; this correction does not establish that gate (`task-1-fix-round1-report.md:5`).

### Strengths

- The correction is a small guard on the mode boundary, without introducing evidence loading or activation requirements into validate-only (`survey/src/bin/democratic_peace.rs:1449`).
- The regression exercises observable behavior: it demands the specific unsupported-option error, preservation of sentinel bytes, preservation of symlink/hardlink identity, and absence of newly created outputs. It covers exact, symlink and hardlink binary aliases, outputs inside a historical study root, and every rejected gate option (`survey/tests/democratic_peace_native.rs:614`, `:660`).
- Retained red evidence fails on the original exact-path alias because the command succeeds; retained green evidence passes the same regression. This demonstrates that the new test detects the original defect rather than merely reflecting the guard's implementation (`task-1-fix-round1-red.log:7`; `task-1-fix-round1-green.log:8`).
- Retained covering evidence reports 9 passing unit tests and 8 passing integration tests, with no warnings. The formatter log is empty and the report records successful formatting and diff checks (`task-1-fix-round1-native.log:17`, `:32`; `task-1-fix-round1-report.md:14`).

### Issues

#### Critical (Must Fix)

- None identified within the fix scope.

#### Important (Should Fix)

- None remaining from the reviewed finding; none introduced by this fix.

#### Minor (Nice to Have)

- None identified within the fix scope.

### Assessment

**Task quality:** Approved for Task 1 with correction `a58eaa9`.

**Reasoning:** The early rejection closes the confirmed evidence-overwrite path before any write and preserves the intended ordinary validate-only interface. The behavioral regression and retained native covering results support the correction without extending this verdict to later scientific activation.

**Review checks:** Read the complete `8b02b9d..a58eaa9` fix package once and read the correction report plus retained red, green, native and formatter logs. No unchanged code was inspected, no tests or covering suites were rerun, no scientific scratch artifacts were accessed or modified, and the only write is this owned review report.
