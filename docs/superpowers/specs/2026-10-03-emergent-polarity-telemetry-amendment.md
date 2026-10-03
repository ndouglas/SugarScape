# Emergent polarity: endpoint stock telemetry amendment

**Date:** 2026-10-03 UTC. Diagnostic export correction before the complete registered
study. The 2026-10-02 scientific design, source extraction, 572-arm manifest, seed
ranges, core mechanics, RNG schedule, configurations, Outcome contract and
registered analysis judges are unchanged. This amendment adds measured endpoint
stock frequencies required by the binding design's numerical-policy section.

## Retained original execution

The first execution used integrated engine head
`b7f453abcc4752db9166a3d8fceae44c7aa560b8` and executable SHA256
`4f4d268475a9a7f1e4ac9899d62d94a6ecba354374dd40f25aa8e695f41250b1`.
At 2026-10-03T04:04:30.447303+00:00 the native process was stopped. Its
`survey/out/polarity-sessions.jsonl` contains 4,272 complete JSON records,
107,830,705 bytes, a final newline and no unparseable record. SHA256:
`8abfb332f67f10a12aa8fc51fc63e8c6b3052f4ecfdcd2d78f116fcc0c655bc6`.
Original executable, resolved export, sessions and provenance are preserved;
`polarity-original-partial-provenance.json` is a byte-identical provenance copy.
A separate pause/termination receipt records the process boundary. The paused
native process was never resumed; wrappers were terminated before it, preventing
late writes. No complete original study result or scientific verdict existed.

The missing export was identified while this original workload was running.
Signed resource creation and rejection events cannot substitute for the required
direct stock frequency. Reporting that frequency merely as unavailable would not
fulfill the approved design, so the controller authorized this minimal diagnostic
repair and a full rerun. The original partial dataset is retained as incomplete
pre-amendment evidence, with no pooling into final findings.

## Added observations and denominators

Native records add `stock_diagnostics` schema version 1 outside unchanged Outcome.
The host reads existing shared initial and final statistics snapshots; reading
these snapshots does not advance periods, consume random numbers or mutate state.
The existing core count includes stocks at or below zero among sovereign-capital
cells for EPM, or among all primitive cells for the provincial variants. Absorbed
EPM provinces with accounting zeros are excluded.

Initial stock-count denominator is width × height. Terminal EPM denominator is
Outcome's sovereign count; provincial denominator remains width × height. Both
completed and attempted snapshot clocks must match the observed state. For an
invalid end state, an available terminal observation describes the aborted state
at its actual attempted clock, without counting the aborted period as completed.
Construction panic or stale state has explicit unavailable/null observations and
reasons, never invented zero counts. Native resume checks the version, unit,
counts, denominators and clocks, rejecting legacy/malformed telemetry without
modifying existing data.

The separate standard-library `survey/polarity/stock_diagnostics.py` helper binds
exact manifest and session-file hashes to the authoritative complete resolved
configs. It validates observation units, integer counts, denominator bounds,
clocks, registered seed counts and provenance; reports received, unavailable and
invalid-telemetry counts; and keeps valid versus invalid model outcomes separate.
Stock frequency is summed nonpositive stocks / summed observed stocks. Session
frequency is sessions with any nonpositive stock / sessions with an available
observation. These endpoint frequencies condition on observed states and are
**not period-exposure frequencies**. They are descriptive, with no new hypothesis,
threshold, bootstrap judge or multiplicity family.

## Separate complete execution and verification

The amended executable and full resolved export are frozen before execution into
`survey/out/polarity-telemetry-*`, with their own hashes, engine head, commands,
UTC timestamps and runtime. The workload repeats every same registered arm and
seed: 572 arms, 28,520 sessions. All invalid outcomes remain retained; no replacement
seed, reduced sample, changed parameter or fit-directed adjustment is permitted.
Final scientific judges use only this complete authoritative dataset with the
unchanged 100,000 draws and seed 2026100202.

Behavioral verification covers direct signed/floor/rejection counts, provincial
and sovereign denominators, construction/stale unavailability, corrupt-resume
rejection, weighted stock versus session frequencies, invalid-state separation,
exact data/manifest binding, and Outcome/fingerprint equivalence for all six
presets. Original RED failures and fresh GREEN commands/results are retained in
the execution report and ignored run logs. The registered analysis.py bytes are
unchanged. This is a telemetry correction, not an amendment to scientific judges.
