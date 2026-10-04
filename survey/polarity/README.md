# Emergent polarity: frozen studies

These studies use the normal core engine. They are outside CI. The manifest froze 572 arms and 28,520 sessions before measurement. The complete
[measured findings](../../docs/superpowers/specs/2026-10-02-emergent-polarity-findings.md)
retain every registered seed, invalid record and precision population.

From the repository root:

```bash
cargo build --release --manifest-path survey/Cargo.toml --bin polarity
python3 survey/polarity/run.py --validate --resolved-out survey/out/polarity-resolved.json --manifest survey/polarity/studies.json --out survey/out/polarity-sessions.jsonl
python3 survey/polarity/run.py --manifest survey/polarity/studies.json --out survey/out/polarity-sessions.jsonl
python3 survey/polarity/analysis.py --resolved survey/out/polarity-resolved.json --manifest survey/polarity/studies.json --source docs/superpowers/specs/2026-10-02-emergent-polarity-source-figures.json --sessions survey/out/polarity-sessions.jsonl --output survey/out/polarity-findings
```

The wrapper's default binary is `survey/target/release/polarity`. Pass `--binary`
when using a different `CARGO_TARGET_DIR`. Validation resolves every configuration
and seed range without executing periods or creating sessions. Execution appends
one flushed record per session and resumes existing keys; duplicate keys, mixed
manifest hashes and changed resolved configurations are rejected. Invalid outcomes
remain in the file and are never replaced. `--arm PREFIX` selects work while still
validating the complete manifest. Concurrent processes must use separate output
files; use one process for resumable output.

Analysis writes JSON and Markdown, preserving raw records, source uncertainty,
invalidity and incomplete arms. The usual `survey` binary remains the default;
its generic `--only` registry does not execute these polarity studies. This
separate pipeline applies bootstrap and multinomial judges offline to frozen
sessions, rather than silently shrinking source counts with a generic seed flag.

Regenerate and verify the manifest without running a world:

```bash
python3 survey/polarity/manifest.py --source docs/superpowers/specs/2026-10-02-emergent-polarity-source-figures.json --output survey/polarity/studies.json
python3 -m unittest discover -s survey/polarity -p 'test_*.py'
cargo test --manifest-path survey/Cargo.toml --bin polarity
```

See the committed design and extraction notes for interpretations, exact source
counts, direction versus counterexample rows, multiplicity families and evidence
limits. Python version and analysis draws are recorded in results. Mean/count
compatibility is not equivalence; source and adapted critic protocols remain
separate. Generated sessions/results stay in ignored `survey/out/` until the
findings note deliberately commits the reviewable summary.

Native `--resolved-out PATH` exports every fully resolved arm with the exact manifest
SHA during validation, before measurement. Preserve that file for analysis and pass
it with `--resolved`; missing or mismatched resolution leaves arms unresolved.
Python does not reconstruct Rust defaults. Reports require exact outer resolved
configs and consistent embedded Outcome config/seed, clocks, finish/status, category
and accounting. Resume rejects inconsistent records without modifying the file;
valid invalid-session records remain completed keys and are never replaced.

Core catches period panics while retaining its live completed/attempted clocks,
partial-period ledger and episodes. The native boundary also catches construction
and host execution panics. A construction panic preserves configuration, seed and
reason with `state_available:false`; unknown clocks/counts/accounting are null,
never measured zeros. Execution panics export the complete available core Outcome.
Signed ledger totals retain negative and zero contributions using precise summation.

## 2026-10-03 diagnostic telemetry amendment

The first workload was interrupted after 4,272 complete records because native
Outcome did not export the binding design's nonpositive-stock frequency. Those
original bytes, executable and provenance remain local and unchanged; no complete
original result or judge verdict existed. A separate full rerun uses every same
registered seed, unchanged core mechanics, configuration resolution, manifest,
source extraction and scientific judges. No invalid session receives a replacement.

Native records now add versioned `stock_diagnostics` outside unchanged Outcome.
Initial and terminal observations read the existing shared snapshots without
advancing the world or consuming RNG. EPM counts sovereign-capital stocks;
provincial variants count all primitive-cell stocks. A terminal observation is
available only when both snapshot clocks match Outcome. Construction panic and
stale state retain unavailable/null observations with reasons.

After the separate full telemetry run, produce the reproducible descriptive summary:

```bash
python3 survey/polarity/stock_diagnostics.py --manifest survey/polarity/studies.json --resolved survey/out/polarity-telemetry-resolved.json --sessions survey/out/polarity-telemetry-sessions.jsonl --output survey/out/polarity-telemetry-stock-diagnostics.json
```

The helper binds exact manifest and data hashes, authoritative complete configs,
units, counts and clocks. It reports stock frequency as summed nonpositive stocks
/ summed observed stocks and session frequency as sessions with any / sessions
with an available observation. Valid and invalid end states are separate;
unavailable observations are never measured zeros. These are endpoint frequencies,
not period-exposure rates. Registered analysis.py judges remain byte-identical.


## Complete execution and retained amendments

The authoritative current findings use `survey/out/polarity-domestic-sessions.jsonl`:
all 27,340 nonprovincial records from the reviewed complete pre-domestic dataset,
plus all 1,180 provincial records from the complete same-seed 14-arm rerun.
All 1,160 two-level complete records match; all 20 overextension Outcomes change.
The 27,340 retained raw lines are byte-identical. Complete authority is
`survey/out/polarity-domestic-resolved.json`, identical to every prior authority.
The earlier `polarity-final-*`, telemetry, sequential and original partial files
remain unchanged; they retain every original verdict, all 237 sequential panics
and the 83 previously valid complete sequential records. Provenance binds both
engine heads, every component hash, full contract audits and all before/after
results. New execution evidence uses only `polarity-domestic-*` ignored paths.

The dated [domestic source-fidelity amendment](../../docs/superpowers/specs/2026-10-03-emergent-polarity-domestic-amendment.md)
removes episode bookkeeping as a veto on per-period provincial decisions; selected
action memory remains explicit. Voluntary revolt actions and contiguous episodes
are distinct counts. Its source clarification identifies the actual illustrative
overextension grid as 10×10 (100 primitive units), with unknown seed; no trajectory
fit is registered. Frozen research notes, source manifest and all 18 computation
functions remain unchanged. Native studies remain outside CI.

```bash
python3 survey/polarity/analysis.py --manifest survey/polarity/studies.json --source docs/superpowers/specs/2026-10-02-emergent-polarity-source-figures.json --resolved survey/out/polarity-domestic-resolved.json --sessions survey/out/polarity-domestic-sessions.jsonl --output survey/out/polarity-domestic-findings
python3 survey/polarity/stock_diagnostics.py --manifest survey/polarity/studies.json --resolved survey/out/polarity-domestic-resolved.json --sessions survey/out/polarity-domestic-sessions.jsonl --output survey/out/polarity-domestic-stock-diagnostics.json
```

Both `analysis.py` and `export_report.py` use the tested strict JSON serializer:
`sort_keys=False`, `allow_nan=False`. Scientific computation functions remain
byte-identical to the preregistered analysis; only the final CLI export changed.
The `"null"` map key in episode_end_causes counts null end causes; actual episode
end_cause values remain JSON null. The original sorted-key export failure and
full original recomputation are retained. Dated export and sequential amendments
explain the corrections without treating an implementation panic as a source
failure or replacing a registered invalid-stock/ratio session.
