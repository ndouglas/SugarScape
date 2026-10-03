# Retirement source and reconstruction audit

2026-10-03. This compact evidence package compares Axtell and Epstein's original retirement figures with fresh ensembles from the native reconstruction, and selected cases with an independently written C++ reconstruction. The numerical stopping criteria are explicitly reconstruction diagnostics: the authors describe an age65/62 norm but do not publish its numeric stopping rule.

The original figure crops preserve axes, labels and legends. [Figure methods](figures/methods.md) document source pages, exact configurations, horizons, uncertainty and censoring. [Contact sheet](figures/contact-sheet.png) previews eight side-by-side comparisons. [Policy age diagnostics](figures/policy-age-diagnostics.png) show why an aggregate95% crossing and a retirement-event age mode need separate interpretation.

The revised policy setup uses threshold U[.5,1], whereas the original uses homogeneous.5. Original Fig6-10 means visually read about70/40/30/20 periods at rational1/2/3/4%; these are approximate source readings, not a flat20–40 target. Fig6-11's rational group also slows as coupling rises. The threshold comparison uses base rational10% and positive source-domain spreads; extent uses only the source rational5% branch6–10. Groups use rational10% in groupB, expected5% globally. Native per-case JSONs state exact reconstruction settings.

[Native results](native/results.md) and [independent results](independent/report.md) report current conditional means, sample SD and attained/censored counts. Fifty runs per source sensitivity point match the original's stated sample size. Native seeds1001–1050 and independent seeds2001–2050 are fixed; RNGs and draw order differ. Only representative native seed1001 traces needed to reproduce plots are distributed. Crossing-time SD bands are not invented50-seed trajectory envelopes.

## Reproduce retained plots and summaries

From this directory, with Python3 plus NumPy, matplotlib and Pillow:

```sh
python3 figures/plots.py
python3 native/summarize.py
```

Plots assert50 observations per cell, retained censor counts and exact policy event histogram totals. Native `selected-seeds.csv` is the union of retained cases; `policy-seeds.csv` is a convenience input for the figure script and is not counted twice by summarization. `summary.csv` can be regenerated from the selected rows. Retained native CSV values are unchanged from the audit outputs.

## Run the native engine probe

The isolated Cargo project depends on the repository core at `../../../../../crates/sugarscape-core`; no engine source is copied or dynamics changed. From the repository root:

```sh
cargo test --manifest-path docs/superpowers/specs/2026-10-03-retirement-audit/native-probe/Cargo.toml
cargo build --release --manifest-path docs/superpowers/specs/2026-10-03-retirement-audit/native-probe/Cargo.toml
```

Run the resulting `native-probe/target/release/retirement-native-audit` with arguments `BATCH 50 OUTPUT_DIRECTORY 1001`. Corrected batches are `trajectories`, `policy`, `groups-source`, `spread-source`, `sensitivities`, `critical`, `counts`, and `cohort-source`. Use a separate output directory. `sensitivities` includes the extent source grid plus unused threshold diagnostics; `critical` also emits Replace. The durable selected comparisons use extent only from that batch, corrected `spread-source`, and Slot critical cells. `select-results.py OUTPUT_DIRECTORY` creates the compact native selected CSV union and summary there, excluding unused batches/diagnostics. Do not replace the durable data with a partial run.

Native source-grid rules: C100, random5%, positive denominator, eligible counting, Slot renewal, oldest cohorts first with cohort-internal shuffle, activation-local aging, literal continuous death age60+40u, newborn age20 and immediate network drawing. Network size integer10–25, extent0–5 sampled per agent; threshold.5 unless varied. Trajectory horizon500; extent/spread/groups600; critical2000. Counts600. Cohort-size cases200/300 stop at the first95% crossing with cap600. Policy mandatory70, pre-switch cap1000,100 rounds after actual switch; automatic95%, fixed100, and rolling eventmode65 for10 consecutive periods are declared alternative warmups. Exact defaults remain defined by the linked repository core and case JSONs. No Replace+groups case is used.

## Run the independent check

From `independent/`, build and run in a separate directory to preserve supplied CSVs:

```sh
c++ -std=c++17 -O3 -Wall -Wextra probe.cpp -o /tmp/retirement-independent-probe
mkdir -p /tmp/retirement-independent-results
cd /tmp/retirement-independent-results
/tmp/retirement-independent-probe
python3 /absolute/path/to/this/package/independent/summarize.py
```

The standalone C++17 engine uses continuous uniform death ages60+40u, independent mt19937_64, continuous revised thresholds U[.5,1], and the explicitly specified Slot/activation conventions. It runs six cases ×50 seeds. Full traces are generated by rerunning; only seed2001 representative traces are retained here. The analysis script expects the full freshly generated `trace.csv` for all seeds and must not substitute `selected_trace.csv`. Its fixtures include positive denominator, equality, mortality precedence, activation-local aging and continuous death65.01 surviving to65 then dying at66.

## Provenance and limits

`provenance.json` records package-relative hashes and source artifact mappings; full papers and full-page text excerpts are not distributed. Figure anchors are original AE printed pp8–16, Figs6-4/5/6/7/9/10/11; revised GSS printed p163 supplies the distinct policy thresholds. Local ignored source artifacts supplied the crops; source figure pixels were not redrawn. No original numeric dataset is available.

Conditional means exclude right-censored seeds and are not unconditional response times. A momentary95% crossing, event-mode peak or ten-period diagnostic can reverse and does not demonstrate the authors' norm. Mortality, renewal and cohort traversal are named reconstruction choices; agreement between two engines does not identify the unpublished author implementation. This package contains current source-compatible measurement evidence, not a formal goodness-of-fit test or proof of infinite-time criticality.

Native `previous_tick_mode_at_switch` is the rolling retirement-event mode from the tick immediately BEFORE the recorded switch tick. It excludes switching-tick events. Independent `mode_at_switch` includes events through the actual switch tick. These diagnostics have different observation windows and must not be compared directly. Native representative trace `modal_age` at the switch tick is a separate observation; no per-seed switch-tick values are inferred from the single retained trace.
