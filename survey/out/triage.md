# Survey triage

Every Weak, Fails and Untestable verdict from `results.json` (156 claims: 125 Holds, 11 Weak, 15 Fails,
5 Untestable, 0 Error), with the cause assigned after reading the check and, where marked, confirming the
result independently.

Causes: the spec's three (**model bug**, **description**, **book**), plus two the triage needed: **setup** (the
rules work but the preset's configuration can't produce what it describes, so from a user's view the preset is
broken) and **check** (the claim's check, threshold or test doesn't test what its source says; the model is fine).
No verdict was traced to a model bug in the rules.

Checks changed after their first run (bugs in the check, thresholds untouched): `iv-3-pollution.stops` (read the
series one tick early; first result Fails 0/20, now Holds), `vi-3.book-period` (the first period finder returned the
smallest lag, 20, on every seed; first result Fails 0/20, still Fails), and the three sweep settlement claims
`fig-ii-5.settled`, `fig-iv-6.settled`, `n-goods-carrying-capacity.settled-1000` (first run panicked, all Error; now
Holds). Each change is described in a comment beside the check.

| id | verdict | cause | evidence |
|---|---|---|---|
| ii-6.propagates | Fails | setup | Independently confirmed (CLI, seed 1, NW/NE/SW/SE counts): t=0 0/0/400/0, t=100 14/4/177/18, t=400 11/17/168/17. The block (x 0–24, y 25–49) starts on the SW sugar peak (15, 40); half starve by t=50, the rest stay. No gradient, no wave. |
| ii-6.waves | Untestable | — | Visual wave-front claim with no agreed metric; moot while ii-6.propagates fails. |
| iii-12.toward-center | Fails | setup | Same cause as ii-6. `Placement::Tribes` (world.rs) puts Blues on the SW peak and Reds on the NE peak (x 30–49, y 0–19; peak (37, 5)). Mean distance to centre never below ~79% of the start. |
| iii-12.interpenetrate | Fails | setup | Same. No Blue–Red adjacency in t=1..100 on any seed. Likely also why iii-9 has few combat deaths (median 5 in 500 ticks): the tribes rarely meet. |
| iii-6.one-tribe | Weak | book | Local homogeneity reaches ~1.0, but the larger tribe holds ≥ 90% of agents at t=3000 on only 10/20 seeds (median 0.85). The two mountains often settle on different tribes, as tests/book.rs already notes. |
| iii-14.together | Untestable | — | "Conquest and conversion together" names no outcome beyond both happening (both hold separately). |
| iv-1.shuttle | Fails | check (threshold) | 49% (median) of agents switch between sugar and spice sites twice in t=100..200. The ≥ 50% bar was the check author's reading of "agents shuttle", not the description's: about half shuttle, so the verdict sits on a knife edge. |
| iv-3.measured-ln-price | Fails | check | The comment records seeds 1–3 exactly; rerun via CLI: 0.008676, 0.007983, 0.009746, identical. The claim judged 20 seeds against the 3-seed span. |
| iv-15.mean-price-wanders | Fails | check | The metric (tick-to-tick sd of a tick's *mean* price) is steadier when more trades average in, so it doesn't isolate "prices don't settle". The dispersion claim (iv-15.prices-unsettled) holds. |
| iv-15.dispersion-level | Fails | book (low confidence) | From-memory claim that dispersion stays level; it *rises* 0.15 → 0.29, which supports the description. Separately, 3/20 iv-15 seeds go extinct by t=1000, which the description doesn't mention. |
| iv-18.falls | Weak | book | Mean foresight 4.98 → 4.41 by t=1000: unpaired p=0.13; paired Wilcoxon on the same 20 seeds p=0.027 (CLI + scipy). Right direction, not significant: not reliably selected down. |
| iv-18.modest-kept | Weak | description | At t=2000 foresight is in [0.1, 5] on 12/20 seeds; 8/20 are above 5.0 (the mean of the initial 0–10 draw). "Evolution keeps a modest foresight" overstates a drift that often doesn't happen. |
| iv-18.plan-ahead | Untestable | — | States the movement rule itself; rule audits are out of scope. |
| v-1.residue | Weak | description | Median 1.8% is inside "~1–3%", but only 10/20 seeds are in [0.9%, 3.3%]; seeds range from near 0 to above 3.3%. |
| v-2.measured-level | Fails | check | The comment records seeds 1–3 exactly; rerun via CLI: 0.045334, 0.042674, 0.078412, identical. |
| v-mcneill.familiar | Fails | description | 0/20 seeds have any disease in circulation at t=300; tests/book.rs's own comment says the society "has already learned away everything it carries". The description's "carrying its familiar diseases" is false. |
| vi-1.flares | Weak | description | Disease flares after all three outbreaks on 15/20 seeds (after most of them on the rest). |
| vi-1.chart-views | Untestable | — | UI menu placement; the network data behind it holds (vi-1.network-views). |
| vi-2.book-crash | Fails | book | Known and documented: VI-2 settles near 800 on 20/20 seeds; the book's crash doesn't happen. Separating sugar and spice further doesn't produce it either (this session's overlap experiment). |
| vi-3.minima-near-700 | Weak | description | Minima median 751 (IQR 736–767); 15/20 within about 700. "Near 750" would be accurate. |
| vi-3.book-twice | Fails | book | Peak exceeds 2× the initial 500 on 1/20 seeds (median 914). The description's own "1.7–2.0×" holds. |
| vi-3.book-period | Fails | book (metric crude) | Autocorrelation-peak period median 184 ticks, not ~115; the metric is the author's own and the spread is wide (IQR 156–286). |
| vi-3.book-trade-raises | Weak | book | Trade vs no-trade plateau 826 vs 809: unpaired p=0.16; paired Wilcoxon (same seeds) p=0.11. Equivalence holds (vi-2.like-vi-3). Known. |
| n-3.widest-gap | Untestable | — | Needs each encounter's pre-trade MRSs, which the public API doesn't expose; covered by core unit tests. |
| n-4.no-site-has-all | Fails | description | 128 sites grow all four goods: the corner peaks wrap to within 14.85 of (−0.5, −0.5) and overlap there (this session's N-goods analysis). "Must travel or trade to hold all four" is false for agents in that corner. |
| fig-iv-6.trade-raises | Weak | check (unpaired test) | The unpaired Mann–Whitney ignores that each seed runs under both settings. Paired Wilcoxon on the file's own 10 seeds (CLI + scipy): trade beats no trade on every seed at every vision, p = 9.8e-4 at visions 1 and 3–6, 2.0e-3 at vision 2. Unpaired at 20 seeds: p = 8e-5 … 1.3e-2. The book's result reproduces; the gain narrows with vision (+8 → +5). |
| fig-iv-6.measured-no-trade-v1 | Weak | check | Sweep means reproduce exactly (33.85, recorded 33.8); the claim judged each of the same 10 seeds against ±10% of their mean. |
| fig-iv-10-11.measured-long-last | Weak | check | Same per-seed-vs-mean mismatch; the mean reproduces (0.1377, recorded 0.138). |
| n-goods-carrying-capacity.measured-no-trade-2 | Fails | check | Same mismatch; the mean reproduces exactly (25.84, recorded 25.8). |
| bargaining-rules.measured-geometric-v1 | Fails | check | Same runs as fig-iv-6's trade line; the mean reproduces exactly (41.78, recorded 41.8). |
| bargaining-rules.measured-random-v6 | Weak | check | Same mismatch; the mean reproduces (75.53, recorded 75.5). |
