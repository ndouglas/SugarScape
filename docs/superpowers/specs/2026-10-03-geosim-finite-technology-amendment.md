# GeoSim premeasurement finite-technology validity amendment

Date: 2026-10-03. Actual public valid configurations exposed a runtime validity gap before any registered measurement. This amendment preserves the source technology equation, its current operation order, parameter bounds, named readings, scientific workload and default trajectories.

## Reproduction

A2x2/one-founder world with initialization0, observation1, base threshold1e308, shock shift1e308 and shock probability1 previously stored an infinite technology frontier and reported valid horizon completion. With observation2 and base threshold1, period2's intermediate multiplication `2*1e308` overflowed although the mathematical endpoint threshold is finite. Grouping source periods did not make that result scientifically valid. These public configurations validate because their input values are finite and positive; finite inputs do not guarantee finite derived quantities.

Separately, four finite thresholds1e308 overflowed their naive summed mean and exposed null JSON for an available finite statistic. This is a display aggregation defect, separate from state-transition validity.

## Runtime policy

The existing frontier expression remains `base+(period-initialization)*shift/observation`; its order is not algebraically rearranged to rescue overflow. Compute the frontier, require a finite positive result before any shock random draw or threshold assignment, and propagate descriptive invalidity through the existing period failure path. Previously completed periods remain completed; the attempted partial period is not counted as complete. Its already-produced finite resource/fighting/structural ledgers remain available, while the last finite state thresholds are retained. Finished invalid worlds are immutable. This is consistent with the source's finite-state reconstruction policy rather than an arbitrary tighter input bound or new scientific axis.

Mean threshold retains the original sum/division when the sum is finite. Only a positive finite sum overflow uses online `mean+=(x-mean)/n`, which avoids overflow for these positive finite values. Keeping the finite-sum path also preserves ordinary display rounding and unequal-subnormal behavior; always applying the online formula can double-round a mean of adjacent subnormals. Equal smallest-subnormal thresholds keep a nonzero mean.

## Evidence and scope

Actual fixture RED evidence is retained in survey/out/geosim-core-technology-finite-red.txt (three failures and one passing subnormal characterization). A separate mixed-subnormal RED covers the online-only double-rounding case. Final focused mechanics, prior/new golden, native/WASM host parity, formatter and strict Clippy evidence belongs in the scratch validity report. No prior model's state transition, API, math backend or golden is changed. No registered history, full-default benchmark, fit or scientific finding preceded the correction.

## Invalid experiment measurements

The actual WASM grouped negative-capacity fixture (four founders, four observation periods in one display tick, seed1, adjustment1, damage1, attack1, superiority0.1) finishes invalid after three completed periods and attempted period4. Its raw completed-war display statistic is finite. Previously the experiment runner treated that finite diagnostic as a completed measurement (n1/nan0). Source evidence is retained in survey/out/geosim-host-invalid-final-experiment-RED.json and native scalar/timeseries RED fixtures.

When a GeoSim terminal Outcome is invalid, the experiment adapter now masks the whole run's metric history with missing values before final, window or timeseries measurement. This prevents an early finite window from admitting the invalid run. Raw snapshots, inspection and partial Outcome ledgers remain available. Valid GeoSim runs and other models retain their existing experiment behavior. This is validity enforcement, with no new scientific reading or registered arm.
