# GeoSim scientific protocol

This offline protocol consumes the real Rust `GeosimConfig`, `GeosimWorld` and `Outcome`. Python never recreates engine defaults. The standalone native recorder is `survey/src/bin/geosim.rs`; the survey crate remains outside the workspace. Source definitions, method contract and actual source inventory must be frozen before registered execution.

Pin Python3.13.5 and the libraries in `requirements.txt`. `NUMERICAL_METHODS.md` and `methods.py` give the full premeasurement estimator/resampling contract and license/reference limits. Run small tests from the repository root:

```sh
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s survey/geosim -p 'test_*.py'
```

Generate a **provisional** declaration (no periods or source inventory freeze):

```sh
python3 -m survey.geosim.manifest --source docs/superpowers/specs/2026-10-03-geosim-source-table.json --output survey/out/geosim-provisional-manifest.json
```

The declaration has37 arms/1490 keys and72 fixed analysis jobs. Original11×15 and precision11×100 remain separate; fourteen one-field readings and the named artifact reference each have15 histories. All configs have common periods_per_tick100; Rust resolves the paper/artifact presets. Row7 includes both thresholds2.5 and shock10. Reciprocal defender/attacked-party damage are corrected paper defaults; same-threshold/acting-party alternatives are exposed but unmeasured.

After all product writers stop, the controller freezes actual scientific sources:

```sh
python3 -m survey.geosim.provenance --manifest survey/out/geosim-provisional-manifest.json --source docs/superpowers/specs/2026-10-03-geosim-source-table.json --source-root . --output survey/out/geosim-frozen-manifest.json
```

`provenance.py::REQUIRED_FILES` is the normative path policy. `required_inventory_paths(root)` adds complete Rust core source/tests, native integration tests, authored protocol Python/Java, and existing Cargo/toolchain build configuration. It excludes reference raw downloads/cache, process ledgers, plan packets, replay directories and generated outputs. Normative source/spec/audit documents use an explicit roster so publishing later findings/provenance does not alter this inventory. `source_inventory(root)` captures sorted path/bytes/SHA256 entries; `freeze_manifest(provisional,root,source_bytes)` verifies the exact registered declaration/method payload and table bytes before binding them. Required files must exist without source or directory symlinks. Verifiers reject empty, unsorted or truncated scientific inventories; modified normative sources or newly added runtime code invalidate old bindings.

`run.py::make_build_receipt` binds a supplied completed build and equal pre/post source snapshots; it executes no compiler. Native independently checks exact manifest/method bytes, current source files, own executable, resolved configs and every existing row before resume. Preserve `resolved_configs_json` exact native UTF8 bytes instead of guessing Rust float JSON spelling. Construction failure/incomplete Outcome stays explicit unavailable state; invalid attempted keys are never replaced. One process owns each JSONL output path.

`python3 -m survey.geosim.run --help` shows native invocation flags. `python3 -m survey.geosim.analysis --help` shows required manifest/source/raw/resolved/build/binary/source-root/output inputs. Registered analysis refuses provisional sources, changed libraries/methods/jobs/bindings and malformed/duplicate/nonfinite evidence. It has no reduced-draw CLI:100000 history bootstraps/1000generated-refitted KS diagnostics remain outside CI. Source freeze, native benchmark and registered execution are separate controller-authorized stages, not consequences of green fixtures.

Every registered slot is exported, including missing, invalid, insufficient, degenerate and unresolved fits. Completed wars at end_period>Outcome.periods belong to an invalid partial period and enter no fit; earlier completed wars of an invalid world remain separate partial diagnostics. Positive raw completions partition visible/backlog; integer subunit export0 still has positive raw collector membership. Completed/censored identities are disjoint. Source strict-CCDF/OLS uses selected visible/all-completed export; modern diagnostics use complete biological raw damage, including completed FIFO backlog once. Censored/partial events never become completed-fit observations.

A recorder implementation panic after a valid terminal engine Outcome retains that Outcome verbatim with nonempty panic context. Attempt availability is primary Unresolved despite complete engine clocks; its completed events remain partial diagnostics and cannot enter source, pooled KS or parameter-bootstrap populations. Censored elapsed clocks use the attempted period, while completed-fit eligibility uses the successful period.

Fixed88 source and6 mechanism families remain full-sized when entries are unavailable. Conditional Compatible is distinct from source equivalence, which stays Unresolved for unverified printed definitions. Modern pooled KS/ratios are iid diagnostics; nested half-mixture calibration remains declared/unvalidated, especially alpha<=3. Parameter treatment comparisons give descriptive independent whole-history intervals. Source extrema have finite bootstrap support, not impossible-model-event interpretation.

CoW v4 input/codebook are available; Thailand war170 fatalities−9 remain unknown. Exact Clauset95-war/SupplementS1 reconstruction is Unresolved, and Cederman's older historical population is separate. Runnable later-port GeoSim2 evidence does not establish full archived mechanics or PCG/two-MT seed identity. Reference-only artifacts/large logs stay ignored under `survey/out/geosim-*`; preserve their original bytes/hashes before cleanup.

`--resolved-out` is optional: native appends `.resolved.json` to the complete `--out` path, including during `--validate`. For `--out sessions.jsonl`, the default is `sessions.jsonl.resolved.json`; it contains the authoritative native configs and byte binding. The history file is not created by zero-world validation. See the dated premeasurement correction in `NUMERICAL_METHODS.md` for strict nested evidence validation, stable Pareto evaluation and separately labeled descriptive source definitions/counters.
