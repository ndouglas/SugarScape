# Democratic Peace prospective follow-up

The approved specification is [the separately dated follow-up design](../../docs/superpowers/specs/2026-10-04-democratic-peace-precision-design.md). This tooling is preparation, not a report of new measurements. The historical literal study remains immutable. Each phase contains108arms ×100fresh histories; PhaseL runs before PhaseP regardless of its findings. Both phase declarations must freeze before anyLperiod.

`followup.py` defines exact schema2 manifests, rosters, readings and129analysis jobs. `historical.py` verifies all13preserved study files, the original341file source tar, executable, own original envelopes and3240complete histories. `followup_analysis.py` validates separate datasets and reuses the original numerical functions:105source predictions, six primary and six secondary contrasts per reading, with100000draws and independent whole histories. Prose eligibility has no prose-original30population. Reading differences are descriptive.

Prepare each phase with `python3 -m survey.democratic_peace.followup_registration prepare --phase PHASE --source TABLE --historical-study-root STUDY --historical-inventory INVENTORY --historical-source-archive TAR --historical-binary BINARY --output MANIFEST`. These paths are explicit authorities; identities cannot be exchanged. Use `probes` with source/phase/output to prepare the fixed unregistered schedule. Preparation commands never start periods. The schedule is frozen in followup-probe-schedule.json:8full native probes per reading, seeds420000001/430000001 plus index, one worker. No probe belongs to a registered population.

Runtime evidence must retain exact commands, source/binary/toolchain/output hashes, CPU/wall and peakRSS. `followup_runtime` measures synthetic100000draw predictive and contrast-pair workloads and fullphase writer overhead; a paired contrast measurement counts12times, so both24contrast jobs count once each. Generation estimates use the maximum8complete probes ×10800. Forecasts double generation+analysis+writer CPU, require12CPUhours perphase,24total,2GiB perworker. Failure requires prospective scheduling revision; never reduce arms/samples/draws. Walltime is not guaranteed.

Freeze only after allwriters stop, then build with a successful unchanged inventory and produce the native receipt. Native `--validate-only` exports exact108arm configs without world construction. `phase_binding` validates each manifest/resolution/receipt/binary. `declare` consumes both phase bindings and benchmark-evidence hash; the generated declaration must be committed outside its own sourceinventory. A later runtime attestation binds declaration+benchmark hashes; independent external review binds declaration+attestation. Native execution requires `--declaration`, `--declaration-review`, `--runtime-receipt` and allfour historical authority paths. PhaseP additionally requires the complete bound `--literal-sessions` attempted census. Clean committed normative sources and committed declaration bytes are checked before either outputwrite. A failed job resumes only existing declared keys; duplicate/wrongconfig/wrongbinding/alias inputs fail before writes. Never replace a seed.

Numerical environment remains Python3.13.5/NumPy2.4.6. Scientific plotting uses a separate environment:

```sh
python3 -m venv survey/out/democratic-peace-report-env
survey/out/democratic-peace-report-env/bin/python -m pip install -r survey/democratic_peace/report-requirements.txt
```

Matplotlib3.10.7 is justified by scientific axes, bands and standaloneSVG/PNG quality. Its exact release and Python3.13 support were verified using [official package metadata](https://pypi.org/pypi/matplotlib/3.10.7/json). The renderer fixesAgg, bundledDejaVuSans, SVGhashsalt, DPI and timestamp metadata. Chartinputs retain individual original values, counts, sourceenvelopes, independent fresh100means and95%predictive intervals for simulated30history replications. Sourceenvelopes are digitizationuncertainty, not paperconfidence intervals; predictivebands are not100mean confidence intervals. Clustering conditions on surviving democracies, keeps extinction/undefinedcounts and gaps, and excludes density0 from its105slot sourcefamily. Validzero territory remainszero.

Seconds-scale tests exercise synthetic fixtures only. Actual probes, scientific declarations,21600fresh attempts and100000draw empirical inference remain separately gated later work. No inference of historical executable/RNG/statistic identity or causality is justified by matching a sourcecurve.

## Command interfaces

Run the complete bounded preparation only after the controller has reviewed the frozen schedule and code:

```sh
python3 -m survey.democratic_peace.followup_runtime --preparation-benchmarks --binary BINARY --source-root ROOT --source TABLE --build-evidence BUILD_EVIDENCE --python PYTHON --output survey/out/democratic-peace-followup-runtime
```

Build evidence has exactly `build_command`, `build_exit_code`, `binary_sha256`, `source_inventory_sha256`, `toolchain`, `effective_build_flags`, `features` and `cargo_configuration`. The toolchain has exactly `target`, `rustc_version` and `cargo_version`; flags/features are explicit string arrays, including empty arrays when none apply. `cargo_configuration` binds an absolute retained file with `path`, `bytes` and `sha256`. That file has exactly `config_inputs` and `environment`: each captured input has its absolute `path`, original byte `sha256` and parsed configuration object; environment records compiler/Cargo variable names and values. Capture inputs in Cargo's low-to-high precedence order. Direct compiler environment flags and ordinary inherited `build.rustflags` must agree with effective flags; target/cfg overrides and command configuration remain retained for review. Effective flags/features pass unchanged into every probe build receipt. Preserve the successful build's context rather than reconstructing it from a later host configuration.

The selected `--python` path becomes absolute while retaining its executable symlink. Before any probe or output write, the helper launches that path and requires Python3.13.5/NumPy2.4.6, retaining the executable path and environment prefix. Invalid numerical runtimes or missing/malformed build evidence fail before consuming the bounded probe schedule.

The orchestration prepares eight separately bound unregistered manifests perreading, retains native commands/logs/rows/resources, runs synthetic100000draw workloads and fullphasewriter measurement, then emits benchmark-evidence.json. It retains a copy of the captured Cargo context and complete build evidence in the output directory. Every measurement binds phase/kind/probeindex and explicit artifact path/size/SHA descriptors. Native probes bind manifest, receipt, resolved export, raw row and stdout/stderr; synthetic jobs bind the complete payload and stdout/stderr. Existing evidence paths are rejected to prevent accidental probe repetition. It never creates registered phase declarations.

`runtime_attestation(declaration_path, benchmark_path)` rereads every retained artifact before approving the forecast. It verifies all eight distinct scheduled indices/seeds/coordinates perphase, exact reading/defaults and one complete1000period native row, build/toolchain context, exact commands, synthetic phase/runtime/job identities, four100000draw predictive cases, the permutation/bootstrap pair and the10800history/105target writer payload. Missing/changed/aliased/substituted evidence fails even when summary arithmetic is consistent. The existing declaration → runtime attestation → independent review chain remains noncircular; artifact bindings are separate from their own inventories.

Analyze a registered phase with `python3 -m survey.democratic_peace.followup_analysis --manifest MANIFEST --source TABLE --sessions RAW --resolved RESOLVED --build-receipt RECEIPT --binary BINARY --source-root ROOT --historical-study-root STUDY --historical-inventory INVENTORY --historical-source-archive TAR --historical-binary ORIGINAL_BINARY --output FINDINGS_PREFIX`. The JSON retains every attempted slot, missing/invalid/partial contexts, perarm extinction/defined census and separate original/fresh bindings.

Export the validated original descriptive comparison:

```sh
survey/out/democratic-peace-report-env/bin/python -m survey.democratic_peace.followup_plots --source TABLE --historical-study-root STUDY --historical-inventory INVENTORY --historical-source-archive TAR --historical-binary ORIGINAL_BINARY --output-dir NEW_PLOT_DIRECTORY
```

After measurements, add `--phase-inputs PATH` pointing to a JSON array of at most two objects with explicit manifest/source/sessions/resolved/receipt/binary/source_root/findings paths. Each source/raw/binary/receipt/config is validated independently. Findings must match the exact phase and old/fresh dataset provenance. Exports include Figures9/10/11 as PNG/SVG, chart-inputs.json and plot-inventory.json with renderer/font and byte hashes. Bothreadings appear sidebyside; pendingphases, unreadablepoints and undefinedclustering remain visible. Original30observations appear faintly and everydensity has exact complete/required and extinction/defined labels in a separate density-indexed census table below each panel; O denotes original and F denotes fresh. The uncertainty footer wraps within the canvas. Inspect allstandalone figures visually before publication.

`python3 -m survey.democratic_peace.followup_run` requires exact native/declaration/review/runtime/historical paths. `--max-new-histories N` checkpoints between histories and prints the remainingpendingcensus without changing declared keys. PhaseP requires `--literal-sessions PATH`; invalidLattempts remainattempts and never block ordering by a favorable scientificverdict. Source or analysis corrections require separately dated identities and review.
