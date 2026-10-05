# El Farol numerical audit

These are exploratory reconstruction checks for the Episode 9 spike. They compare native simulation ensembles with separate implementations and published figure extracts. They are not exact historical replications: original predictor banks, initial scores, RNGs, and some tie/evolution conventions are incompletely specified. See [numerical-review.md](numerical-review.md) for the retained protocol and interpretation.

Run commands from this directory. Rust uses the repository core dependency and a standalone Cargo workspace; Python requires NumPy, Matplotlib, and Pillow.

```sh
cargo run --release -- inverse
cargo run --release -- smr
cargo run --release -- arthur
cargo run --release -- baseline
cargo run --release -- evolution
python3 independent_arthur.py
python3 monoclonal.py
python3 plots.py
```

`inverse` uses 20 seeds, N1001 M4 S5, 5000 rounds, rounded/exact payoff and independent redraw/retain tie variants. `smr` uses 32 seeds, N101 S2, M2/6/12, 10000 rounds. `arthur` uses 20 seeds, N100 k12, 2000 rounds, accuracy/payoff scoring. `baseline` uses the same Arthur horizon for random attendance and a shared predictor bank. `evolution` uses 20 seeds, N1001/101 M6 S5, replacement every 10, mutation 0/.1, 40000 rounds. The independent Python Arthur experiment reconstructs the chosen 48-predictor bank; the pure-population experiment compares redraw and sticky ties for 20 seeds over 2000 rounds.

CSV files retain per-seed summaries and attendance histograms. Individual trace CSVs support the figure comparisons and baseline unanimous-frame counts. Plots use the retained figure crops directly; full paper pages/PDFs are not included. `manifest.json` records SHA256 hashes of retained files and source provenance. Regenerated outputs may change if simulation source or library versions change.
