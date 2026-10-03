# Emergent polarity: frozen studies

These studies use the normal core engine. They are outside CI. No native study
has been measured during plan preparation; the manifest freezes 572 arms and
28,520 sessions before measurement.

From the repository root:

```bash
cargo build --release --manifest-path survey/Cargo.toml --bin polarity
python3 survey/polarity/run.py --validate --manifest survey/polarity/studies.json --out survey/out/polarity-sessions.jsonl
python3 survey/polarity/run.py --manifest survey/polarity/studies.json --out survey/out/polarity-sessions.jsonl
python3 survey/polarity/analysis.py --manifest survey/polarity/studies.json --source docs/superpowers/specs/2026-10-02-emergent-polarity-source-figures.json --sessions survey/out/polarity-sessions.jsonl --output survey/out/polarity-findings
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
