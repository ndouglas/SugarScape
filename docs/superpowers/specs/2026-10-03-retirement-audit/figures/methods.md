# Source versus native figure comparisons

Local audit evidence only; not a publication-ready claim of exact reproduction. `plots.py` reads native summary and original trace CSVs directly. Run with `python3 plots.py`; NumPy, matplotlib and Pillow required. No generated or edited simulated values, no ImageGen, and no tracked changes.

## Source anchors and crops

All source image crops preserve the axes, plotted points, tick labels, and figure legends without alteration. Crop coordinates in `plots.py` are pixels of locally rendered PDFs. Only figure regions retained; no full paper text copied here.

| Comparison | Original anchor | Native input |
|---|---|---|
| comparison-64 | AE printed p8, PDF9, Fig6-4 | trajectory_0.15 and trajectory_0.2 |
| comparison-65 | AE printed p9, PDF10, Fig6-5 | trajectory_0.05 |
| comparison-66 | AE printed p10, PDF11, Fig6-6 | critical_Slot_0/.02/.05/.1 |
| comparison-67 | AE printed p11, PDF12, Fig6-7 | spread-source positive grid, R=.10 |
| comparison-69 | AE printed p13, PDF14, Fig6-9 | extent_6..10, R=.05 |
| comparison-policy-ae | AE printed p15, PDF16, Fig6-10 | policy_ae_*_auto |
| comparison-policy-gss | AE original figure retained as reference; GSS printed p163, PDF186 supplies changed threshold distribution | policy_gss_*_auto |
| comparison-611 | AE printed p16, PDF17, Fig6-11 | groups-source_* only |

GSS parameter anchor: printed p163, PDF186; full-page excerpt is not included. Revised policy comparison is explicitly to an AE original figure, not a claim that its curve was digitized from GSS. Approximate AE policy means at rational1/2/3/4% are70/40/30/20; shown dotted as visual estimates, without invented SD. Original crop retains actual error bars. No flat20–40 acceptance band is used.

## Frozen native choices and uncertainty

Exact config JSONs in `../native/<case>-config.json` are authoritative. Common rules are C100, random5%, network10–25, threshold.5, extent5, eligible counting, Slot renewal, ByCohort activation, Literal initial mortality unless named. Seeds1001–1050 are fixed across comparisons; summary cells must have n50 and attained+censored50 or plotting aborts. Sample SD, not SE/CI, accompanies conditional means. Labels retain attained/50, and all remaining seeds are right-censored at stated horizons. A conditional mean is not an unbiased unconditional mean under censoring.

Trajectory horizon500, critical horizon2000, threshold/extent/groups horizon600. Policy pre-switch limit1000 and100 periods after actual switch; mandatory retirement70. AE threshold homogeneous.5; GSS threshold U[.5,1] (mean.75, SD.25/sqrt3). Automatic warmup switches at native95% proxy and can switch from a state that does not exhibit an age65 norm. `policy-age-diagnostics.png` therefore shows actual rolling event mode and earliest-event fraction before/after the actual switch at R5%, seed1001. Exact age counts are checked to sum to new_events. Fraction is age-at-current-eligibility retirement events divided by all new retirement events, missing when no events occur; it is not retirement hazard or prevalence.

Representative trajectory seed1001 was fixed before figure viewing. No seed is selected to resemble the paper. Only two full traces per case exist, so a50-run time-series envelope cannot be drawn honestly. The plotted shaded band is50-seed mean±SD of first95 crossing TIMES, not a band for retired/eligible VALUES. Fast comparison64 displays a0–25-period zoom of retained500-period traces; slow comparison65 displays0–500. Source/native axes retain their independent labels/scales. AE fast prose15% and caption20% disagree: both native cases are included.

Threshold uses restored source base R10% and inferred positive uniform halfwidth grid .05,.10,.20,.30,.40,.50, mapped to SD≈.0289,.0577,.1155,.1732,.2309,.2887; zero extension excluded from source comparison. Extent uses only source R5% branch6..10. Groups use corrected config rational=.10, giving expected B10%/global5% after A suppression, and explicitly show BOTH source/native rational curves slowing.

Native first95, group95 and post-switch95 are unpublished reconstruction stopping criteria, never author-defined age65/62 norm. Rolling mode62×10 is another reconstruction diagnostic. The paper provides no numeric stopping criterion for sensitivities. Critical native0/2% cases are finite-horizon extensions, with no source5%-random points there; the source branches for other random shares are not matched by these native cases. No mathematically infinite-time criticality is inferred. No Replace critical curve is plotted; the selected critical comparison is Slot only.

## Checks and limits

`plots.py` asserts n50, retained censor counts, and exact histogram sums for policy seed1001. [`../provenance.json`](../provenance.json) records SHA256 for retained config/summary/trace/source-crop inputs. Rendered policy and group comparison figures visually inspected: source axes/legends remain readable and the two group curves are labelled correctly. Figure-only original crops are retained in this durable package as `source-*.png`, alongside the comparison images; full pages and full paper text are not distributed. No original numeric dataset is available: source curve comparisons are visual, not formal goodness-of-fit tests. Native and source stopping semantics differ, so apparent time disagreement does not identify an omitted causal rule.

Native `previous_tick_mode_at_switch` is the rolling retirement-event mode from the tick immediately BEFORE the recorded switch tick. It excludes switching-tick events. Independent `mode_at_switch` includes events through the actual switch tick. These diagnostics have different observation windows and must not be compared directly. Native representative trace `modal_age` at the switch tick is a separate observation; no per-seed switch-tick values are inferred from the single retained trace.
