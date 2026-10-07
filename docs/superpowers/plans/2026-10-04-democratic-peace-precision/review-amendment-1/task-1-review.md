### Spec Compliance

- ❌ Issues found: the schema 2 validate-only path can overwrite explicitly supplied historical evidence because those arguments are omitted from protected inputs (`survey/src/bin/democratic_peace.rs:1511`, `:1518`, `:1539`). This violates the design's immutable-input and before-write alias requirements.
- ✅ All ten Task 1 files have corresponding hunks in the supplied `5092277..8b02b9d` review package. Canonical phase rosters, readings, 100-history schedules, historical identities and separate method/phase bindings are implemented (`survey/democratic_peace/followup.py:57`, `:102`; `survey/src/democratic_peace_followup.rs:6`, `:68`).
- ✅ Schema 1 remains a separate accepted envelope: manifest and record dispatch explicitly select schema 2 without admitting its extra fields into schema 1 (`survey/democratic_peace/manifest.py:111`; `survey/democratic_peace/records.py:222`). The actual diff changes no engine or numerical estimator files (review package file summary).
- ⚠️ Cannot verify from this task: complete external activation/runtime/declaration enforcement, actual preserved-archive execution, and production callers' use of `followup.validate_resolved` remain Task 2 integration gates (`survey/src/democratic_peace_followup.rs:186`; `survey/democratic_peace/historical.py:22`; `survey/democratic_peace/followup.py:138`). No registered histories or probes were run during review.

### Strengths

- Canonical roster equality uses JSON type-sensitive comparison, including explicit probability overrides and exact jobs, preventing boolean indices and undeclared arm/config fields from passing Python validation (`survey/democratic_peace/followup.py:102`; `survey/democratic_peace/test_followup.py:51`).
- Native resolution checks the literal configuration after normalizing only the probability-direction field; its integration fixture confirms both 108-arm/10,800-key phases and their exact one-field configuration difference (`survey/src/bin/democratic_peace.rs:580`; `survey/tests/democratic_peace_native.rs:528`).
- Original records retain their independently computed manifest/receipt/resolved bindings and frozen archive/executable identity; preservation paths and bytes are checked again after loading (`survey/democratic_peace/historical.py:22`).
- Retained TDD evidence has the expected feature-absence import failure followed by 55 passing Python tests and 9 unit plus 7 native integration passes; green logs have no warnings (`task-1-red.log:14`; `task-1-green.log:3`; `task-1-native.log:42`, `:55`). Both formatter logs are empty, matching the report's successful checks (`task-1-report.md:17`).

### Issues

#### Critical (Must Fix)

- None identified within Task 1 scope.

#### Important (Should Fix)

- **Validate-only output can replace an explicitly declared historical input** — `survey/src/bin/democratic_peace.rs:1511` adds declaration/runtime/historical input paths only when `!validate`. Consequently `validate_destinations` at `:1518` sees neither `--historical-binary` nor the other external follow-up arguments during validate-only, and `atomic_write` at `:1539` replaces an aliased file. This is an immutable-evidence violation even before any world construction. The exact replay bytes prescribed by the plan contain this defect (plan-mandated bytes); reconcile the focused correction with the checkpoint rather than treating replay identity as proof of correctness. Protect every supplied file argument in both modes, expanding a supplied historical study root to protect its evidence files, or reject unsupported gate arguments in validate-only before writing. Add a behavioral sentinel regression that verifies rejection and byte preservation.

  Focused reproducer, executed only in a temporary directory with a fake historical sentinel:

  ```python
  from pathlib import Path
  from tempfile import TemporaryDirectory
  import json, subprocess
  from survey.democratic_peace.followup import build_manifest, EXPECTED_HISTORICAL
  from survey.democratic_peace.records import strict_json
  root = Path.cwd()
  data = (root / 'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json').read_bytes()
  with TemporaryDirectory() as td:
      directory = Path(td)
      manifest = directory / 'manifest.json'
      sentinel = directory / 'historical-binary'
      manifest.write_text(json.dumps(build_manifest(strict_json(data), data, 'literal_precision', EXPECTED_HISTORICAL)))
      sentinel.write_bytes(b'historical sentinel\n')
      result = subprocess.run([
          str(root / 'survey/target/debug/democratic_peace'),
          '--manifest', str(manifest), '--resolved', str(sentinel),
          '--repo', str(root), '--historical-binary', str(sentinel),
          '--validate-only'], capture_output=True, text=True)
      print(result.returncode, result.stdout.strip(), result.stderr.strip())
      print(sentinel.read_bytes() == b'historical sentinel\n')
  ```

  Observed: exit `0`; stdout `validated 108 arms/10800 keys without world construction; provenance=provisional_unfrozen`; stderr empty; sentinel preserved `False`. No real historical artifact was used or changed. Existing covering logs do not answer this mode-specific alias case (`survey/tests/democratic_peace_native.rs:528`).

#### Minor (Nice to Have)

- None identified.

### Assessment

**Task quality:** Needs fixes.

**Reasoning:** The protocol and independent historical loader are well separated and have clean retained covering evidence, but the confirmed validate-only alias defect can destroy a declared evidence input. Resolve that narrow before-write protection defect before accepting Task 1; broader activation integration remains a separate Task 2 review.

**Review checks:** Read the supplied actual diff in contiguous portions once; used a metadata-only hunk-to-line lookup for precise citations. Read the brief, report, binding design and retained logs. No unchanged caller inspection was needed for the confirmed risk; no covering suites were rerun. The only execution was the focused temporary-sentinel reproducer above. The only write is this owned review report.
