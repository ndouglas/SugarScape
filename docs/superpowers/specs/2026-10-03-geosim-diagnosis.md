# GeoSim source fit diagnosis

Date: 2026-10-03. Status: retrospective research notes for completed milestone 37; no future milestone design approved.

The registered source-fit unavailability follows from the frozen raw-damage cutoff, strict CCDF and complete joint-population rule. It is separate from unresolved attribution of the source definitions. The grid75 arm has complete eligible populations and six corrected rejections, so its conditional reconstruction verdict remains **Incompatible**. Exact source equivalence remains **Unresolved** for every arm.

This diagnosis reads existing outputs only. It introduces no registered runs, fits, resamples, amended judges, dropped histories or product edits. The original findings remain preserved.

## Measured identity and evidence

The measured object is the independently specified portable paper reconstruction, not a certified identity of the 2003 APSR executable, a seed-identical docking of the recovered later GeoSim2 port, or a literal Clauset Supplement S1 reproduction. Model severity is abstract resource damage; empirical battle fatalities have a distinct definition.

The registered matrix contains 37 arms and 1,490 attempted history keys: 1,486 completed and four invalid. All **1,265 histories in the 11 original and 11 precision source arms completed**. The four invalid histories belong only to reading.printed_increasing and do not explain source-arm fit unavailability. Original populations have 15 histories per arm; precision populations have 100. They remain separate.

The [compact findings](2026-10-03-geosim-findings.json) bind the actual measured inputs below. These are recorded execution identities, not fresh rerun verification in this diagnosis.

| Binding | SHA256 |
|---|---|
| data | `6388068c85f2525d18b781d011a3c605686ca73456f3c9b6239ff92a3ecae4ad` |
| manifest_sha256 | `7e6baf8445230187f1d4169b114fecb9bd698b3569a3af79188230b41eb287d4` |
| binary_sha256 | `d2144adf6d4b24c12239d08d05b3aa7ea8596c08724009d53a9be2f8da9f24ff` |
| source_inventory_sha256 | `5d7f78abd30f8dee86abb63f11f110b36a7119f554c0ae0d663a9aab7f579181` |
| build_receipt_sha256 | `3dada7cf7217b57296dc78f258bd0b7f37912722a519283dcdb199ac2770deb8` |
| resolved_configs_sha256 | `176583d2823ced2f898adcddbe486ab86fde68f6188a5c077f5769f0fde46f75` |
| source_table | `44a7d45d5206081ed179febae5285546924a894e55ebfb25e93222e59a1ed976` |

The retained final-commit receipt records execution source HEAD `d64d3becd3682df4edeab90e62a1ee9f0526e1d5` and tested findings commit `6a1b36f03fb603f7d231d9895692f928e7325640`. These historical identities should not be replaced by the current main or research-worktree HEAD.

Inputs were read from absolute main-checkout paths rooted at `/Users/nathan/Projects/ndouglas/SugarScape/`. The retained full output is `survey/out/geosim-publication-evidence/execution-before-final-report/out/geosim-execution-task5/findings.json`; it includes actual source OLS fit-point arrays omitted from the compact matrix. The same retained task5 directory contains `analysis-receipt.json` and `final-commit-receipt.json`. Original task4 bindings are preserved under the sibling `geosim-execution-task4` directory. No whole raw session dataset was needed.

## Failure hierarchy

1. **Definition attribution:** 66 slope/R² targets have severity_scale_unverified; 22 range/count targets have inferred definitions. The printed table does not establish severity units, range endpoints or the N-wars census. All 88 targets retain source equivalence Unresolved.
2. **History fit eligibility:** The frozen primary estimator uses selected completed positive exported severities, raw damage by default, strict count(S>s)/N, unique thresholds and log10(s)>=2.5. N is all positive selected completions. The maximum has zero strict CCDF and is excluded. OLS requires at least three distinct fitted x values and positive x/y variance.
3. **Joint population eligibility:** A history needs finite slope, R², range and count to supply its joint vector. All 15 original vectors are needed for the eight-summary original estimate, and all 100 precision vectors for the predictive comparison. Defined range/count alone do not rescue an incomplete joint population.
4. **Judging:** Each incomplete precision arm has all eight targets unavailable with incomplete_precision_fit_population. Missing/failed histories cannot be removed to reduce n. Available p-values maximize over the printed rounding interval and receive Holm correction across all 88 targets. At least one corrected rejection gives conditional Incompatible; complete availability with no rejection gives Compatible; otherwise Unresolved.

The full retained fit arrays show that **every unavailable primary OLS has fewer than three actual strict fit points**. None of these failures reaches the variance-degeneracy gate; none is an optimizer failure. There are 35 original plus 242 precision ineligible fits, totaling **277**.

## Exact eligibility counts

Failure columns give actual strict fit-point count:number of failed histories. All listed histories completed their registered horizons.

| Arm | Original eligible | Precision eligible | Failed original fit points | Failed precision fit points |
|---|---:|---:|---|---|
| base | 15/15 | 97/100 | None | 1:2, 2:1 |
| shock10 | 13/15 | 80/100 | 0:1, 2:1 | 1:8, 2:12 |
| shock0 | 0/15 | 0/100 | 0:15 | 0:100 |
| context_off | 0/15 | 0/100 | 0:15 | 0:100 |
| shadow10 | 15/15 | 99/100 | None | 1:1 |
| shadow40 | 15/15 | 97/100 | None | 1:1, 2:2 |
| thresholds2_5_shock10 | 14/15 | 94/100 | 2:1 | 0:1, 1:1, 2:4 |
| mobile0_9 | 15/15 | 99/100 | None | 2:1 |
| distance_offset0_2 | 13/15 | 93/100 | 1:1, 2:1 | 0:2, 2:5 |
| distance_exponent5 | 15/15 | 99/100 | None | 2:1 |
| grid75 | 15/15 | 100/100 | None | None |

The shock0 and context_off populations account for 230 failures. Every one of their 115 histories per arm has **zero positive tail observations at raw severity >=10^2.5 (approximately 316.227766)**. They still contain many completed positive wars: shock0 count bounds are 204–294 original and 158–311 precision; context_off bounds are 1,378–1,558 original and 1,346–1,591 precision. Zero fit points does not mean zero wars.

All other failed histories have one to three distinct tail severities; excluding the maximum leaves zero to two fitted points. Each failed history still has finite range and count. No zero completed severities occur in these source-arm selected censuses. Both range/count targets become unavailable through the joint-vector rule, not because those quantities lack values.

Both registered source mechanism contrasts also remain Unresolved: precision base is 97/100 eligible, while shock0 and context_off are 0/100. The fixed six-comparison family is preserved. Available modern parameter summaries or pooled iid diagnostics do not supply missing source vectors or establish these source contrasts.

## Grid75 conditional disagreement

All 15 original and 100 precision histories are joint eligible. The registered predictive comparison draws 15 whole precision histories with replacement 100,000 times, preserving their joint four-metric vectors. Original summaries are displayed separately; the predictive p-values compare Table1 rounding intervals against the precision-generated 15-history summaries.

| Target | Source rounding interval | Original estimate | Predictive 95% interval | Raw p | Holm p | Corrected rejection |
|---|---|---:|---|---:|---:|---|
| slope_min | [-0.675, -0.665] | -0.825963375622 | [-1.0912504262198353, -0.7700327173779339] | 7.9999200008e-05 | 0.00663993360066 | Yes |
| slope_median | [-0.595, -0.585] | -0.675023651609 | [-0.7568323916182964, -0.5946573788263146] | 0.0526594734053 | 1 | No |
| slope_max | [-0.545, -0.535] | -0.449722780408 | [-0.5634603285092902, -0.3997311259143233] | 0.171278287217 | 1 | No |
| r2_min | [0.9865, 0.9875] | 0.778030436349 | [0.7224854984111381, 0.8751746767321206] | 1.9999800002e-05 | 0.00175998240018 | Yes |
| r2_median | [0.9925, 0.9935] | 0.949860286299 | [0.8822802427066262, 0.9524594424946033] | 1.9999800002e-05 | 0.00175998240018 | Yes |
| r2_max | [0.9955, 0.9965] | 0.984938398817 | [0.9567222268518931, 0.9874379442390178] | 1.9999800002e-05 | 0.00175998240018 | Yes |
| log_range_median | [4.55, 4.65] | 5.25205644249 | [5.026869578361355, 5.324683495937205] | 1.9999800002e-05 | 0.00175998240018 | Yes |
| war_count_median | [502, 502] | 975 | [951.0, 1062.0] | 1.9999800002e-05 | 0.00175998240018 | Yes |

The five equal minimum raw p-values are 2/100001 = 0.000019999800002. The first Holm multiplier is 88, producing 0.00175998240018; the running maximum retains that adjusted value across the five tied tests. Slope-min is the sixth ordered test with multiplier 83: its raw p=8/100001 gives Holm p=0.00663993360066. The 80 unavailable targets remain in the fixed family, internally represented as p=1 for correction while their reported adjusted values stay unavailable. These are therefore **six rejections after the full 88-target correction**, not an eight-test correction. Slope median and slope max do not reject.

The observed precision R² maximum is 0.987437944239, below the source R² median/max rounding intervals. Every precision range is at least 4.70336452714, above the source range interval; every precision war count is at least 815, above 502. This explains finite-bootstrap support separation. It does not establish that source values are impossible model events. The reference support comprises 100 measured histories, especially limiting interpretation of extrema.

## Retained descriptive definition checks

The following counts are descriptive inspection of already-retained alternative fits. They neither replace primary vectors nor define new inference tests.

| Population | Primary raw strict eligible | Raw inclusive eligible | Integer100 strict eligible | Integer100 inclusive/rank eligible |
|---|---:|---:|---:|---:|
| original.base | 15/15 | 15/15 | 15/15 | 15/15 |
| precision.base | 97/100 | 98/100 | 100/100 | 100/100 |
| original.shock10 | 13/15 | 14/15 | 15/15 | 15/15 |
| precision.shock10 | 80/100 | 92/100 | 100/100 | 100/100 |
| original.shock0 | 0/15 | 0/15 | 14/15 | 14/15 |
| precision.shock0 | 0/100 | 0/100 | 92/100 | 96/100 |
| original.context_off | 0/15 | 0/15 | 15/15 | 15/15 |
| precision.context_off | 0/100 | 0/100 | 100/100 | 100/100 |

Integer100 multiplies raw damage by 100 and applies Java truncation/saturation. Thus the same printed cutoff selects a substantially different raw support. Inclusive CCDF also retains a maximum point excluded by strict CCDF. The stored alternatives demonstrate definition sensitivity, but do not establish which convention generated Table1; even integer100 leaves shock0 incomplete.

The [artifact audit](2026-10-03-geosim-artifact-audit.md) records that the recovered later port uses integer100 export, inclusive CCDF and event-weighted tied observations, differing OLS/R² cutoff-boundary handling, and a fitted-positive count. It does not certify the original 2003 executable, and no recovered named range/N-wars statistic establishes Table1 equivalence. A simple postmeasurement switch to those conventions would therefore change the registered judge without resolving source identity.

## Limits and evidence needed to reopen reproduction

Preserve the completed findings: grid75 is conditionally Incompatible under the frozen definitions; the other ten source arms remain conditionally Unresolved because complete precision fit populations are absent; all arms remain Unresolved for source equivalence. Descriptive disagreements in available original summaries do not create corrected rejection claims for unavailable targets. Modern pooled tests remain iid diagnostics, not validation of dependent histories or substitutes for source reproduction.

The next useful evidence would be original author-attributed collector/data material establishing severity units, cutoff, strict/inclusive/rank convention, tie weighting, range endpoints and N-wars census. Original per-history event exports, source-version attribution and experiment configuration would strengthen that attribution. Numerical docking additionally requires original RNG/runtime details and non-consuming stream/callsite traces; seed equality or execution of a later port is insufficient.

A verified definition mismatch could justify a separately approved, prospectively specified reproduction effort retaining these findings. Additional seeds, a lowered cutoff, a changed export scale or deletion of failed histories cannot repair this registered result by themselves. No such effort, amended judge or future milestone is approved by these notes.

## Failed seed appendix

Exact keys and strict fit-point counts from the retained full findings. Zero-fit shock0/context_off ranges list every registered key in those populations.

| Arm | Failed seed:number of strict fit points |
|---|---|
| original.shock10 | 370010010:2, 370010014:0 |
| original.shock0 | 370020001–370020015 inclusive: 0 points each |
| original.context_off | 370030001–370030015 inclusive: 0 points each |
| original.thresholds2_5_shock10 | 370060004:2 |
| original.distance_offset0_2 | 370080013:1, 370080014:2 |
| precision.base | 370110005:2, 370110021:1, 370110057:1 |
| precision.shock10 | 370120001:2, 370120008:1, 370120014:2, 370120017:2, 370120019:2, 370120027:2, 370120029:2, 370120032:1, 370120033:1, 370120034:1, 370120036:2, 370120040:2, 370120050:1, 370120056:1, 370120070:1, 370120079:2, 370120081:2, 370120082:1, 370120095:2, 370120099:2 |
| precision.shock0 | 370130001–370130100 inclusive: 0 points each |
| precision.context_off | 370140001–370140100 inclusive: 0 points each |
| precision.shadow10 | 370150022:1 |
| precision.shadow40 | 370160007:2, 370160033:1, 370160079:2 |
| precision.thresholds2_5_shock10 | 370170031:2, 370170032:2, 370170038:2, 370170063:0, 370170067:2, 370170077:1 |
| precision.mobile0_9 | 370180022:2 |
| precision.distance_offset0_2 | 370190017:0, 370190029:2, 370190043:2, 370190070:2, 370190074:0, 370190085:2, 370190095:2 |
| precision.distance_exponent5 | 370200035:2 |

## Source locations

- [Findings report](2026-10-03-geosim-findings.md): arm eligibility, grid75 numeric targets and source contrasts.
- [Compact findings matrix](2026-10-03-geosim-findings.json): histories[*].source, source.arms, source.targets and provenance.
- [Design](2026-10-03-geosim-design.md): frozen source definitions, whole-history populations and judging rules.
- [Source extraction](2026-10-03-geosim-source-extraction.md) and [source table](2026-10-03-geosim-source-table.json): printed targets, rounding assumptions and attribution limits.
- [Artifact audit](2026-10-03-geosim-artifact-audit.md) and [author code notes](2026-10-03-geosim-author-code-reading-notes.md): recovered collector conventions and source-identity limits.
- [Source estimator](../../../survey/geosim/source.py): source_fit, complete_vectors, source_findings, fixed_holm and arm_verdict.
- [Record census](../../../survey/geosim/records.py): completion availability and selected completed-war censuses.
- [Provenance inventory](2026-10-03-geosim-provenance.json): original ignored output locations, omission paths and retained execution receipts.
