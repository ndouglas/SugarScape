### Spec Compliance

- ❌ Issues found. Task 3 contains the required interfaces and numerical protocol, but the native recorder does not fully validate existing scientific records or canonical registered manifest identities, and analysis can overwrite its own provenance inputs. These are material failures of the strict identity/corruption/output-protection requirements: `survey/src/bin/democratic_peace.rs:440`, `survey/src/bin/democratic_peace.rs:957`, `survey/democratic_peace/analysis.py:137`.
- ✅ All 27 explicitly required product files have matching hunks in the supplied review package; `IMPLEMENTATION_PLAN.md` is the additional stage-bookkeeping change. Scope is BASE `8b2c7f2500b6d52db9bdfa71a41715060d61084b` to HEAD `edb15b6c489959af778560f33d9928b80459f697`, Task 3 only. Required-file evidence: `.superpowers/sdd/2026-10-03-democratic-peace/review-8b2c7f2..edb15b6.diff:6` and task brief file roster at `.superpowers/sdd/2026-10-03-democratic-peace/task-3-brief.md:3`.
- ⚠️ Cannot verify from this task diff: unchanged engine atomic state/RNG behavior and prior host parity, future empirical census/findings, final whole-branch review, exact-final-head CI/Pages, deployed smoke, and evidence-preserving cleanup. These remain their respective task/integration gates, not reasons to broaden this review. The original PDF is absent here; the independently audited table/receipt is retained, so this review cannot independently reconfirm visual extraction against the original PDF (`.superpowers/sdd/2026-10-03-democratic-peace/task3-evidence/source-audit.json:1`).

### Strengths

- Exact Python population constructors preserve all 108 canonical arms, original 30 and independent precision 100 sessions, fixed disjoint seeds, and 129 deterministic jobs; original-only analysis explicitly marks precision unavailable instead of substituting 30 histories (`survey/democratic_peace/manifest.py:26`, `survey/democratic_peace/manifest.py:110`, `survey/democratic_peace/source.py:143`).
- Source prediction retains all attempted undefined draws, including zero/one-survivor populations; interval maximization accounts for inclusive jumps and analytical open gaps; unavailable slots remain in Holm105. Primary and secondary contrast families use separate Holm6 adjustments and independent whole-history permutations/bootstrap (`survey/democratic_peace/numerics.py:16`, `survey/democratic_peace/numerics.py:65`, `survey/democratic_peace/contrasts.py:28`, `survey/democratic_peace/source.py:143`).
- Python science validation is substantially stronger than the native resume gate: it validates exact science/metric fields, atomic clocks, conserved census, availability reasons, final/current equality, and exact resolved payload bindings (`survey/democratic_peace/records.py:141`, `survey/democratic_peace/records.py:270`). The independent source-review receipt binds paper, slots, raster, and axes before the Python freeze (`survey/democratic_peace/provenance.py:72`, `survey/democratic_peace/provenance.py:121`).
- Actual preparation evidence supports the reported bounded scope. The retained six-key fixture is four completed plus two invalid attempts; invalid attempts have attempted period 1/completed period 0, and byte-identical resume is recorded (`survey/out/democratic-peace-preparation-final/fixture-receipt.json:1`, `survey/out/democratic-peace-preparation-final/fixture-sessions.jsonl:5`). Eleven tested rejection cases exited 1 with protected raw/input bytes unchanged (`survey/out/democratic-peace-preparation-final/rejections.json:1`). These cases are useful but do not establish comprehensive science validation; see Issue 1.
- Build evidence is an actual normal-target locked release Cargo command, exit 0, with source inventory, toolchain, effective flags and both lock hashes. I independently compared the current binary and manifest bytes with their retained receipt hashes; both match (`survey/out/democratic-peace-preparation-final/build.command.json:1`, `survey/out/democratic-peace-preparation-final/build.stderr.log:1`, `survey/out/democratic-peace-preparation-final/fixture-build-receipt.json:1`, `survey/out/democratic-peace-preparation-final/cargo-configuration.json:1`).
- The full provisional declaration retained a successful 216-arm/14,040-key validation without construction, and its declaration directory contains no sessions file (`survey/out/democratic-peace-preparation-final/provisional-full-validate.stdout.log:1`). Source audit retains 78 readable/26 overlap/1 absent and three external density-zero exclusions (`.superpowers/sdd/2026-10-03-democratic-peace/task3-evidence/source-audit.json:1`).
- Runtime receipts agree with actual per-probe timing logs and raw rows: six unregistered 1,000-period completions; worst CPU 1.55 seconds, peak RSS 18,464,768 bytes; conservative original/full forecasts 2.79/12.09 CPU-hours. The retained prospective choice is original-only, one process, no frozen registered manifest and zero registered histories (`survey/out/democratic-peace-preparation-final/runtime-receipt.json:1`, `survey/out/democratic-peace-preparation-final/probe-3.stderr.log:1`, `survey/out/democratic-peace-preparation-final/prospective-population-selection.json:1`). No finding changes that scientific selection.
- CI adds only bounded native/Python fixtures and the release CLI build required for web parity; no registered population runner or full inference is added (`.github/workflows/ci.yml:74`, `.github/workflows/ci.yml:97`).

### Issues

#### Critical (Must Fix)

None identified in this task scope.

#### Important (Should Fix)

1. **Native resume accepts corrupt scientific rows and writes resolved output before validating existing sessions.** `survey/src/bin/democratic_peace.rs:957` validates only a few fields for completed/invalid science; at line 977 any object is accepted as completed final metrics. It does not validate nested schema, metric/census consistency, terminal clocks, or current/final equality. Implementation-panic/incomplete science skips even this shallow science block. Atomic invalid clocks can also have gaps larger than one because line 968 only bounds attempted time between completed time and horizon. This contradicts the strict-record and all-existing-attempt validation contract. A focused reproduction changed only the first completed retained fixture row's `outcome.outcome.final_metrics` to `{}` in a temporary six-row copy. Native exited **0**, printing `recorded 6 attempted histories`; Python rejected the identical file with `raw line 1: complete final metrics differ from committed census`. This can let resume skip corrupt attempts and append remaining histories, only failing during later analysis. Bring native validation to the same science/availability/schema contract before accepting any existing key, including partial/panic outcomes, and add actual-recorder regressions for these cases. Also move `atomic_write` at line 1061 after successful existing-row validation at line 1077 for execution: current duplicate/truncated rejection evidence already contains newly created resolved exports (`survey/out/democratic-peace-preparation-final/reject-duplicate-resolved.json:1`, `survey/out/democratic-peace-preparation-final/reject-truncated-resolved.json:1`), so corrupt input rejection is not write-free.

2. **Native registered-manifest validation does not enforce canonical arm IDs or exact job payloads.** `survey/src/bin/democratic_peace.rs:440` checks only that arm IDs are nonempty and unique; registered factor/index/seed checks never verify the prescribed ID. Job checks beginning at line 545 accept arbitrary unique IDs and any four u32 seed-state words, rather than the frozen 129-job roster/state. These are semantically checked in Python (`survey/democratic_peace/manifest.py:110`, `survey/democratic_peace/analysis.py:11`) but the native CLI can execute directly, so Python's later rejection is too late for the required before-first-period gate. Focused reproduction: change only the first arm ID in a temporary full provisional manifest to `arbitrary-noncanonical-name`; the actual binary exits **0** and prints `validated 216 arms/14040 keys without world construction`. Frozen execution uses the same arm/job checks. Enforce the canonical registered IDs and complete fixed job payload in native preparation, or make an equally strict preflight an unavoidable part of native execution, with changed-ID and changed-job negative fixtures. This is an identity-validation fix, not a change to scientific populations or methods.

3. **Analysis output can overwrite a validated manifest or other provenance input.** `survey/democratic_peace/analysis.py:137` and line 138 write the derived JSON/Markdown destinations without checking aliases against inputs or inventoried sources. All source verification happens before those writes. A focused fixture-only reproduction copied the valid fixture manifest to a temporary `manifest.json`, invoked analysis with both `--manifest` and `--output` set to that path, and observed exit **0** with the manifest replaced by the findings JSON. The same error can destroy a frozen registered manifest or another JSON provenance artifact; source-tree output can also invalidate the frozen inventory after the final check. Resolve both actual output paths and reject canonical, symlink and hardlink aliases against all input artifacts, the executable and inventoried source files before creating directories or writing either output. Add a bounded CLI fixture proving input bytes and both output destinations remain unchanged on rejection. The native destination protection is a useful existing pattern, but it does not cover this separate Python writer.

#### Minor (Nice to Have)

None added; existing deferred formatting/chatter issues are outside this Task 3 scope.

### Verification and evidence inspected

- Inspected the named requirements, verbatim global constraints, approved design and implementer report. Read the supplied diff in one complete sequential review pass; no Git diff rederivation, changed-file rereads, broader codebase crawl, or subagents. Line references were mechanically indexed from that supplied diff.
- Inspected discovery RED logs: missing native target and seven missing Python implementation imports. Inspected retained GREEN logs/command receipts: Python 36 tests in 0.192 seconds, native 8 unit + 2 integration tests, strict Clippy, formatting, replay 18 tests, complete 613-product/897-protected-file verification and clean diff-check. All reported exit codes agree with visible logs; no unexpected warnings were observed (`.superpowers/sdd/2026-10-03-democratic-peace/task3-evidence/python-green.log:1`, `native-green.log:1`, `survey-clippy.log:1`, `replay-tests.log:1`, `packet-final.log:1` in that same evidence directory). Empty formatter/diff-check logs have successful command receipts.
- Inspected actual preparation/build/fixture/rejection/runtime/source audit and prospective-selection receipts and relevant raw session/timing logs. These support the stated preparation result; they do not cover the three failure cases above. Source table/raster re-extraction was neither needed nor performed.
- Ran only three targeted checks prompted by concrete unanswered risks, outside the checkout in automatically removed temporary directories: corrupted final metrics accepted by native/rejected by Python; noncanonical registered arm ID accepted by native validate-only; analysis manifest-output alias overwrites its temporary input. The first reused all six existing keys and executed no new histories; the second was validation-only; the third generated only a synthetic fixture report with no eligible inference. No registered freeze, history, source comparison, full resampling or suite rerun occurred. `PYTHONDONTWRITEBYTECODE=1` prevented imports from changing checkout artifacts. Existing evidence, product files, index and HEAD were not changed; only this explicitly requested review report was written.

### Assessment

**Spec compliance:** ❌ Needs fixes before registered histories.

**Task quality:** Needs fixes.

**Reasoning:** The numerical methods and bounded preparation evidence are strong and the original-only decision is supported. The remaining defects are boundary failures that allow invalid identities/records to be accepted or validated evidence to be overwritten, so a registered run should wait for these focused integrity fixes and their corresponding packet/review updates.

### Focused reproduction commands retained

Exact Python stdin programs, invocation, checkout, reviewed commit pair and observed outputs are retained in `/tmp/dp-task3-review-focused-evidence.json`. Each was invoked as `PYTHONDONTWRITEBYTECODE=1 python3 -` from the reviewed checkout. This is a transcript of the actual checks, not a new execution; the temporary input/output copies were automatically deleted by `TemporaryDirectory`, while the original fixture evidence remains available. The first program reproduces Issues 1 and 2; the second reproduces Issue 3. Resolve/write ordering is explicitly part of Issue 1 and is corroborated by already-retained rejection resolved exports.

To extract the two commands without executing them:

```bash
python3 - <<'PYREPRO'
import json
from pathlib import Path
data = json.loads(Path('/tmp/dp-task3-review-focused-evidence.json').read_text())
for index, check in enumerate(data['checks'], 1):
    path = Path(f'/tmp/dp-task3-reproduce-{index}.py')
    path.write_text(check['stdin'])
    print(f"PYTHONDONTWRITEBYTECODE=1 python3 {path}")
PYREPRO
```

Run extracted commands only against the reviewed revision/bound preparation identity; after fixes, rebuild/rebind bounded fixtures first. Zero registered histories were executed during this review.
