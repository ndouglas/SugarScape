# Whole-preparation code review

Reviewed range: `5092277c77270508c986ea4022cc507de532569e..6ba1331458c9f879637f7f570fe77fee3406738f`.

Authority: the approved `docs/superpowers/specs/2026-10-04-democratic-peace-precision-design.md`, preparation/review briefs, the complete supplied diff, and the corresponding source files. This is a review of scratch preparation and readiness for the controller's next steps, not approval of scientific activation or a claim that measurements exist.

Review method: read all changed code, tests, configuration and documentation; inspect the unchanged validators and numerical functions they call; inspect the retained verification logs. Covering tests were not rerun. Three small novel probes inspected Matplotlib artist geometry/strokes, exercised runtime attestation with entirely synthetic objects supplied through an in-memory `Path.read_bytes` mock, and compared the requested virtual-environment executable with its symlink target. No probe generated worlds, empirical inference, benchmark receipts, real declarations or plot files. Only this review report was written; no product edits, commits, branch changes or agents were made.

## Strengths

- The original engine, `source.py`, `contrasts.py`, `numerics.py`, baseline method contract, source table and historical published findings are unchanged in the reviewed range. Schema dispatch is concentrated at existing boundaries, with the schema 2 helpers separate from the original numerical implementation.
- Both fresh rosters preserve canonical factor order, contain exactly 10,800 independent keys, use disjoint phase seed ranges, and carry explicit phase envelopes. The literal and prose eligibility rules remain distinct; neither phase pools with the historical 30, and there is no fictitious prose-original population or cross-reading significance test.
- Historical validation checks the exact preservation inventory, all 13 entries, frozen archive and executable hashes, archive members, original build/manifest/resolved bindings, and the complete original census. It never relabels historical rows with current bindings.
- Native resolution restricts the fresh configurations and probability reading; Python compares full resolved configurations with the frozen default payload. Source and contrast analysis reuse the unchanged estimators, conditional handling, 100,000 draws, fixed families, child streams, and draw ordering. The 129-job forecast correctly counts 105 predictions plus twelve measurements that each include one permutation and one bootstrap job.
- The noncircular declaration → runtime attestation → independent review chain is structurally sound. Both prospective schedules are included before L, the declaration must be committed outside its source inventory, native activation checks clean committed normative sources, and P checks L's full bound attempted census without requiring favorable or complete scientific outcomes.
- The plots retain source mask gaps, density-zero clustering gaps, valid-zero territory, separate populations, explicit complete/required and extinction/defined counts, traceable observations, and the required uncertainty meanings. A suspected isolated-envelope defect was investigated and withdrawn: `fill_between(color=...)` gives even a zero-width polygon a one-point edge stroke, so the isolated readable intervals are represented. Actual full-figure visual QA remains the controller's task.
- Existing logs show Python 68 tests, reporting 3 tests, native 9 unit plus 7 integration tests, and the reported locked workspace run passing. Those checks are meaningful evidence for the covered cases, but the new tests do not cover the three failures below.

## Issues

### Critical (Must Fix)

None found.

### Important (Should Fix before preparation freeze)

#### R1. Runtime attestation does not verify that retained evidence represents the declared workloads

- **Locations:** `survey/democratic_peace/followup_runtime.py:31`, `:45`, `:259`, especially `:274`–`:280`.
- **Failure:** `runtime_attestation` recomputes arithmetic and checks summary identities, but never reads the native/synthetic output files or validates their hashes, scheduled coordinate/seed, reading, workload kind or 100,000-draw payload. `validate_measurement` accepts any nonempty command and nonempty toolchain dictionary. Eight copies of one measurement satisfy the complete-native-probe test. Thus a copied/wrong-workload summary, or a summary whose primary outputs have disappeared or changed, is still attested as complete retained evidence.
- **Concrete reproduction:** An in-memory synthetic evidence object used `command=['/usr/bin/true']`, the same arbitrary `output_sha256` on every measurement, eight duplicate measurements per phase, and `toolchain={'synthetic_test': True}`. No native or synthetic output files existed. Matching synthetic declaration hashes and recomputed forecasts were supplied only through mocked reads. `runtime_attestation` returned `status='approved_complete_phase_forecast'`. No declaration or receipt file was created. This demonstrates missing workload validation, not an attempt to authenticate a dishonest reviewer.
- **Impact:** The attestation is the machine-checked link that is supposed to establish complete bounded preparation before independent declaration review and native activation. Its current acceptance means the forecast can be based on missing, duplicate, unrelated, or wrong-draw measurements while retaining the approved status. A summary hash proves the summary's identity, not that its stated primary evidence is present and appropriate.
- **Smallest correction:** Give retained measurements explicit primary artifact paths/hashes and workload identity (phase, kind, and probe index where applicable). Before attestation, reread and verify the referenced bound probe manifest/receipt/resolution/raw artifacts and synthetic outputs; require exactly the eight scheduled distinct seeds/coordinates for each reading, the declared analysis kinds/draw counts, and writer evidence. Keep the existing arithmetic and noncircular hash chain. An explicit benchmark artifact inventory can provide these bindings without adding artifacts to their own inventory.
- **Required focused coverage:** Reject a changed/missing primary output, duplicate probe index/seed, substituted phase or coordinate, wrong synthetic kind/draw count, and unrelated command/summary; accept a small fully bound synthetic receipt fixture. These are validation fixtures, not benchmark reruns.

#### R2. Benchmark probe build receipts discard the effective compiler flags

- **Locations:** `survey/democratic_peace/followup_runtime.py:195`, `:223`–`:224`; unchanged callee `survey/democratic_peace/run.py:31` and its `build_flags or []` result.
- **Failure:** The exact build-evidence field set has no place for effective features or build flags, and `benchmark_preparation` omits those arguments when calling `make_build_receipt`. Every generated probe receipt therefore records empty feature/flag lists regardless of the actual build environment.
- **Concrete current trigger:** The controller's retained `survey/out/democratic-peace-precision-preparation/build-context.json` records effective flags `["--cfg", "tokio_unstable"]`. Its `cargo-configuration.json` identifies `/Users/nathan/.cargo/config.toml` as their source. Passing these effective flags as a new top-level build-evidence field is rejected at line 195; omitting them produces `build_flags=[]` at line 224. No build or benchmark was run by this reviewer.
- **Impact:** The exact native executable remains hash-bound, but the generated build receipt misstates how it was built. A build command alone cannot recover flags inherited from the user's Cargo configuration. This breaks the promised reproducible toolchain/build provenance before the actual preparation is frozen.
- **Smallest correction:** Extend the preparation build-evidence contract with validated effective build flags/features and pass them unchanged to `make_build_receipt`. Bind the effective Cargo/environment configuration evidence, or retain its explicit identity, so inherited settings are auditable. Update the command-interface documentation and keep schema 1 receipt semantics unchanged.
- **Required focused coverage:** Generate a receipt through the preparation path using nonempty effective flags/features and assert exact preservation; reject missing/malformed required build context before output writes or probes.

#### R3. Resolving `--python` follows virtual-environment symlinks and launches the base interpreter

- **Location:** `survey/democratic_peace/followup_runtime.py:241`.
- **Failure:** `str(Path(python).resolve())` dereferences the executable symlink. Virtual environments commonly use a symlink to the base Python executable; selecting the interpreter through the environment's `bin/python` path is what activates that environment. Replacing it with its real target bypasses the environment the caller explicitly selected.
- **Concrete reproduction:** For the existing reporting environment, invoking its `bin/python` reports `sys.prefix` under `survey/out/democratic-peace-report-env`. Invoking the exact `.resolve()` result reports `/opt/homebrew/opt/python@3.13/Frameworks/Python.framework/Versions/3.13`. This tiny interpreter-identity check executed no benchmark. The two prefixes differ on the actual host.
- **Impact:** On a host where the pinned NumPy is installed only in the selected environment, all eight native L probes can finish before the first synthetic child fails to import NumPy or rejects the base environment's version. When the base interpreter happens to have matching packages, the selected environment is still silently ignored. This undermines both a usable bounded preparation sequence and the pinned-runtime contract.
- **Smallest correction:** Make the executable path absolute without resolving its final symlink and invoke that path. Preflight the selected interpreter's Python/NumPy identity before the first native probe or output write, so an invalid environment cannot consume the bounded schedule before detection.
- **Required focused coverage:** Use a temporary symlink-based virtual environment or an equivalent executable-path fixture to verify that command construction preserves the requested path and that runtime validation occurs before native measurements.

### Minor (Nice to Have)

No additional actionable findings.

## Recommendations

Fix R1–R3 with the original implementation worker before freezing preparation evidence. Each fix can be exercised with small protocol/path tests; none requires registered worlds, empirical analysis, or another broad test rerun merely to reproduce the finding. Preserve the existing mathematical and historical identities. Then run the changed-byte checks and proceed with the controller's already assigned bounded runtime, original plot inspection, exact replay and written-plan work.

The three controller rulings are otherwise implemented consistently: historical keys bind the native resolved payload SHA; activation gates are explicit external arguments; and declaration/runtime/review identities form a noncircular chain. R1 concerns what the runtime link verifies, not the direction of that chain.

## Declined to judge

- Whether either phase meets 12 CPU-hours and 2 GiB, or the combined 24 CPU-hour limit: actual bounded native/synthetic preparation belongs to the controller and has not been executed by this review.
- The actual full-figure appearance of the original descriptive exports: the controller still owns producing and visually inspecting those artifacts; the reviewer inspected plotting code and a tiny in-memory artist probe only.
- Exact byte replay and adequacy of the final written implementation plan: controller-owned deliverables not yet supplied for this code review.
- The independence or substantive approval of a future declaration reviewer, and the future committed declaration bytes: no real scientific declaration exists in the reviewed preparation.
- Scientific outcomes, completion of the 21,600 later attempts, empirical inference, and future publication/archive completeness: those are deliberately gated future work, not missing preparation measurements.
- Main reconciliation, final-head CI, GitHub Pages and served artifact checks: these are later integration/publication requirements, outside this scratch preparation range.

## Assessment

**Ready to merge/freeze this preparation? No — with fixes R1–R3.**

The core protocol, separate-population analysis, historical preservation and plotting semantics align well with the approved specification. The current preparation should not be certified as ready for bounded-runtime freeze or later activation until retained workload evidence is actually validated, effective build context is preserved, and the selected numerical environment is honored. Successful corrections do not themselves authorize scientific activation; the controller's remaining runtime, visual, replay, plan and declaration gates still apply.
