# Episode 9 numerical probe (exploratory audit)

These audit probes do not alter simulation behavior. Native core runs use deterministic seeds 1–20, except SMR seeds 1–32. Independent programs use different RNGs; trajectories and hashes are not expected to match. Comparison is across ensemble summaries, not exact seed pairing. No failure threshold was selected or changed by these probes.

## Inverse payoff, CZ97 Figure 4

N=1001, M=4, S=5; 5000 rounds; summaries and histograms use rounds 1001–5000. Native and independent redraw uniformly among best-score ties each round. Independent retain keeps the active strategy if tied for best; changes select a best tie randomly. Tables are uniformly independent Bernoulli(0.5); histories start random. Zero winners give zero payoff. The standalone Rust independent implementation does not call core payoff, tables, selection, or stepping.

| Variant | Mean A | Center fluctuation / N | Central share |
|---|---:|---:|---:|
| fig4-native-rounded | 500.051 | 0.25413 | 99.814% |
| fig4-independent-rounded-redraw | 500.513 | 0.24816 | 99.877% |
| fig4-independent-rounded-retain | 502.108 | 0.29477 | 100.000% |
| fig4-native-exact | 499.917 | 21.73042 | 71.506% |
| fig4-independent-exact-redraw | 500.544 | 21.89613 | 71.780% |
| fig4-independent-exact-retain | 500.832 | 22.02605 | 71.285% |

Zero-payoff update shares (directly counted from retained tail80 histograms; min(A,N−A)≥401): fig4-native-rounded: 100.00000%, fig4-independent-rounded-redraw: 100.00000%, fig4-independent-rounded-retain: 100.00000%.

Central share is attendance within 5% of N of N/2; it is the existing survey metric, not an adequate definition of bimodality. Published Figure 4 visibly has lobes near A≈350 and A≈650. Rounded native and independent runs concentrate near the center and do not reconstruct those lobes. Exact payoff generates a central peak with multiple smaller side peaks; most rounds remain central. Keeping tied active strategies does not rescue the rounded result. The nearest-integer rule yields zero reward when x>400, making the centered initial regime score-frozen until a sufficiently extreme attendance fluctuation. This is an implementation-independent mismatch with the printed description/figure, not proof of a core coding bug. The paper does not state the Figure 4 horizon, RNG, full initialization, or tie rule.

See figure4-comparison.png: published count axis alongside independently normalized histograms; vertical scales and horizons differ explicitly.

## Arthur independently rebuilt 48-predictor experiment

The bank is the native constructed bank (12 same, 8 mirror, 11 rounded mean, 10 least-squares trend, 7 mirror mean), not an identified historical Arthur bank. Independent NumPy implementation derives forecasts and scoring without core calls. It seeds 12 random attendance weeks, draws 12 distinct predictors per agent, redraws uniform best ties, predicts <60 to attend, regards A≥60 as crowded, uses decayed absolute error λ=.9 or cumulative payoff-advice accuracy. Twenty seeds, 2000 rounds, last 1600 summarized.

| Engine / scoring | Mean A | RMS around 60 | Variance around own mean | Lag1 |
|---|---:|---:|---:|---:|
| Native arthur-accuracy | 59.142 | 26.361 | 694.336 | -0.4326 |
| Native arthur-payoff | 59.922 | 16.517 | 272.901 | -0.0557 |
| Independent accuracy | 59.114 | 26.325 | 692.402 | -0.4338 |
| Independent payoff | 60.135 | 16.591 | 275.341 | -0.0479 |

Independent ensemble closely agrees with native. Accuracy scoring retains substantial negative lag1; payoff advice scoring is near zero. This supports a result specific to the constructed predictor bank and scoring conventions. Arthur says the dynamics are deterministic after starting conditions and predictor assignment; native random tie redraw is a reconstruction choice. Lag1 alone does not establish presence or absence of persistent cycles. With default forecast-at-capacity stays home, mean active-forecast above share = 1−mean attendance/N−mean forecast-exactly-60 share; therefore above share cannot be inferred as exactly 40% merely from mean attendance near60. Forecast-exactly60 mass was not logged in this probe. arthur-figure1-comparison.png shows published first 100 weeks beside native and independent seed1 first-100 traces. No original figure statistics were digitized.

## SMR sample scale

Native N101 S2 M2,6,12; seeds1–32; 10000 rounds; tail80. These are three points, not a reproduction of the entire original curve.

| M | variance / N (own mean) | squared deviation / N (N/2) | Mean A |
|---|---:|---:|---:|
| smr-m2 | 1.343721 | 1.367601 | 50.73914 |
| smr-m6 | 0.063130 | 0.063447 | 50.47717 |
| smr-m12 | 0.242438 | 0.242500 | 50.52309 |

For each seed, E[(A−N/2)²]/N = Var(A)/N + (E[A]−N/2)²/N. The native survey reports center squared deviation, whereas variance about the realized mean is distinct. At M2 the ensemble difference is ≈.0239; at M6/M12 it is small. The three points show a lower intermediate-memory fluctuation than the small and large memory points.

## CZ97 Figure 10 pure-population mechanism check

Standalone NumPy minimal minority game: N1001 M6 S5, all agents hold the SAME five tables drawn once per seed, no replacement or mutation. Scores start zero; sticky variant starts every active index at 0 and changes only for strictly better score; redraw variant chooses uniform best ties every round. Twenty seeds, 2000 rounds; paper displayed window 1000–2000 assessed (here rounds1001–2000). This is a pure-population mechanism example, not an exact Figure10 replication.

| Ties | Mean A | RMS around N/2 | Central share |
|---|---:|---:|---:|
| redraw | 547.354 | 408.408 | 13.220% |
| sticky | 548.953 | 487.754 | 3.665% |

Both conventions produce substantial waste. The sticky convention is more extreme. The existing survey compares evolving heterogeneous populations with versus without mutation over the final tenth of 40000 rounds; that differs from the paper’s described already-pure population and 1000–2000 displayed interval. Native evolution chooses best/worst over each replacement interval, ties to smallest index; the original text does not fully specify this selection window, timing, or mutation probability. No failure claim should be attached to the native Figure10 proxy as an exact source reconstruction. See monoclonal-comparison.png.

## Native evolution sensitivity (existing reconstruction only)

N1001/101 M6 S5 every10, mutation probability0/.1; 20 seeds, 40000 rounds. Final-tenth fluctuation below is exploratory and not source equivalence evidence.

| N | Mutation | Final-tenth fluctuation/N |
|---|---|---:|
| 1001 | fig10-every10-mutation0 | 2.22344 (20 completed seeds) |
| 1001 | fig10-every10-mutation0.1 | 2.21342 (20 completed seeds) |
| 101 | fig10-every10-mutation0 | 0.29835 (20 completed seeds) |
| 101 | fig10-every10-mutation0.1 | 0.23356 (20 completed seeds) |

## Chance and shared forecast bank

Native N100 k12, twenty seeds, 2000 rounds, tail80. Random agents attend independently with probability .6. Shared agents all hold all48 constructed forecasts and rate by accuracy; ties remain random per agent.

| Behavior | Mean A | Variance | Lag1 | All0 or100 share |
|---|---:|---:|---:|---:|
| arthur-random | 59.981 | 23.849 | -0.0042 | 0.000% |
| arthur-shared | 49.903 | 2499.337 | 0.0257 | 100.000% |

Sharing this forecast bank strongly synchronizes choices. All0/100 denotes realized attendance exactly0 or100. Score ties can produce mixed decisions, so this does not imply every shared-bank frame is unanimous. All twenty baseline seed traces are retained.

## Reproduction

Run from `docs/superpowers/specs/2026-10-02-farol-audit/`:
```sh
cargo build --release
./target/release/farol-probe inverse
./target/release/farol-probe smr
./target/release/farol-probe arthur
./target/release/farol-probe baseline
./target/release/farol-probe evolution
python3 independent_arthur.py
python3 monoclonal.py
python3 plots.py
```

Native probes retain per-seed/window means, variance (newer modes), lag1, central shares, histogram bin counts. Fig4 histogram counts retain each seed and exact attendance bin. First100/2000 Arthur and pure-population traces are retained for plotting. SHA256 manifest covers all retained scripts, Cargo inputs, CSV, source figure extracts and plots; target build directory excluded.
