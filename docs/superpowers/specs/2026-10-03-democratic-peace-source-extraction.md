# Democratic peace source extraction

Date: 2026-10-03. Status: premeasurement extraction with first-reader visual audit complete; independent second-reader visual review complete. No registered periods, simulations or scientific resampling informed these readings.

The [source table](2026-10-03-democratic-peace-source-table.json) preserves all 105 potential targets: Figure 9 (36), Figure 10 (33), and Figure 11 (36). There are 78 readable inferred curve values, 26 unreadable overlaps and one absent scan trace. Three Figure 10 density-zero structural exclusions remain separately recorded outside the inferential family. Exact source equivalence remains Unresolved.

## Paper and raster evidence

Source PDF: `papers/geopolitics/cederman-2001-jcr-democratic-peace-kantian-selection-process.pdf`, SHA256 `069ad22da938727b2020c5d2c3a52aa383687c74085f5bc0af8cea74817b5215`. It was read from the main checkout at `/Users/nathan/Projects/ndouglas/SugarScape/`. Printed486/488/489 correspond to PDF pages17/19/20. Original full-page renders were visually inspected, then reproduced independently using Poppler at scale-to1800; new PNGs match the original inspected bytes exactly.

| Figure | Printed/PDF page | PNG SHA256 | x0/x1 pixels | y0/ymax pixels | ymax |
|---|---|---|---|---|---|
| fig9 | 486/17 | `b2b22e44405aa6df4267ab110577d9180ba68c6c2ae210a582d106c9cbff85c1` | [326, 884] | [892, 310] | 1.0 |
| fig10 | 488/19 | `a3fcec8687f1414b7eca9e1b3a8a0a6fb857e78b372b1dd77df5460660a90002` | [329, 798] | [852, 337] | 15.0 |
| fig11 | 489/20 | `1aaa6b48ddd3a6437116be4e901b7c54dd45d2d5025d89f268fe9f692873dd8a` | [317, 759] | [793, 300] | 1.0 |

Images are retained locally under `survey/out/democratic-peace-preparation/page-17.png`, `page-19.png`, and `page-20.png`; each is1120×1800 pixels. The prefix was explicitly verified ignored before generation. Source table figure metadata retains exact render argv, source-paper location, image hashes, axis calibration, mechanism trace anchors and per-target pixel centers/envelopes. The source scan/raster is research evidence, not a product asset.

## Reading method and uncertainty

The figures contain continuous lines without marked sampled means. Every numerical ordinate is attributed **inferred_curve_value**, never raw_original_mean. The described sweep frequencies supply the target x roster; the plot alone does not establish105 original point observations or source error bars. Figure9/11 display mean democratic territory; Figure10 displays survivor-conditioned mean clustering under incompletely specified original exposure/RNG semantics.

The first reader inspected all three full pages and enlarged charts, then checked local dark-pixel neighborhoods to distinguish dashed strokes, labels, reference diagonals and mechanism curves. Manual coordinates remain in `survey/democratic_peace/extraction.py` (carried by the implementation-plan packet). Pixel detection only checks an already-attributed manual neighborhood; it does not classify curves or invent missing targets.

Each readable target carries an x±2pixel envelope and a conservative ordinate band (at least7pixels for thin lines and8 for dashed lines). Wider20–32pixel bands protect steep or interrupted dashed segments. These bands include stroke width, horizontal calibration and trace ambiguity. Derived y intervals additionally evaluate both ordinate limits against all±2pixel top/bottom axis calibrations and clamp to the statistic support. No result-selected rounding or simulated fit sets their width.

Shared zero/one territory endpoints are readable only as visually traceable common endpoints, with pixel envelopes retained. They are not certified raw original sample means or zero-width statistical intervals. Figure10 density1 uses the visible shared endpoint at the same-as-initial baseline with support clamped to1. These choices remain inferred source readings.

Figure9 tagging/alliances at.05 and all mechanisms at.7 are not separately attributable; their slots remain unreadable_overlap. In Figure10 alliances/security merge at.2–.3; all mechanisms merge at.4–.7, so those individual targets remain unreadable_overlap. Figure11 tagging at.3–.5 merges with the horizontal axis, while its faint trace at.15 lacks an independently readable ordinate and is absent_source_point. No value is assigned by interpolating across these missing/overlapping targets.

[Independent second-reader review](2026-10-03-democratic-peace-source-review.json) checked all three pages, calibrations, readable target centers/envelopes and exclusions. It accepted the inferred values without changing intervals. An absent scan trace does not establish absence of original data. Source interval edits after measurement would require a dated amendment preserving the old table and findings.

## Reproduction command

```bash
python3 -m survey.democratic_peace.extraction \
  --paper /Users/nathan/Projects/ndouglas/SugarScape/papers/geopolitics/cederman-2001-jcr-democratic-peace-kantian-selection-process.pdf \
  --image-root survey/out/democratic-peace-preparation \
  --output docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json \
  --render
```

This regenerates the first-reader extraction and pending second-reader metadata; preserve any later review receipt rather than overwrite a frozen table. No simulation, source judge or resampling is performed by extraction.

## Target mask and ordinate envelopes

| Target | Read status | y interval | Pixel x/y center |
|---|---|---|---|
| fig9.tagging.density0 | readable | [0.000000, 0.015464] | 326/892 |
| fig9.tagging.density0_05 | unreadable_overlap | Unavailable | Unavailable |
| fig9.tagging.density0_1 | readable | [0.142612, 0.173540] | 381.8/800 |
| fig9.tagging.density0_15 | readable | [0.235395, 0.266323] | 409.7/746 |
| fig9.tagging.density0_2 | readable | [0.226804, 0.257732] | 437.6/751 |
| fig9.tagging.density0_25 | readable | [0.132302, 0.163230] | 465.5/806 |
| fig9.tagging.density0_3 | readable | [0.151203, 0.182131] | 493.4/795 |
| fig9.tagging.density0_4 | readable | [0.420962, 0.451890] | 549.2/638 |
| fig9.tagging.density0_5 | readable | [0.716495, 0.747423] | 605/466 |
| fig9.tagging.density0_6 | readable | [0.850515, 0.881443] | 660.8/388 |
| fig9.tagging.density0_7 | unreadable_overlap | Unavailable | Unavailable |
| fig9.tagging.density1 | readable | [0.984536, 1.000000] | 884/310 |
| fig9.alliances.density0 | readable | [0.000000, 0.017182] | 326/892 |
| fig9.alliances.density0_05 | unreadable_overlap | Unavailable | Unavailable |
| fig9.alliances.density0_1 | readable | [0.288660, 0.323024] | 381.8/714 |
| fig9.alliances.density0_15 | readable | [0.469072, 0.585911] | 409.7/585 |
| fig9.alliances.density0_2 | readable | [0.840206, 0.874570] | 437.6/393 |
| fig9.alliances.density0_25 | readable | [0.867698, 0.902062] | 465.5/377 |
| fig9.alliances.density0_3 | readable | [0.958763, 0.993127] | 493.4/324 |
| fig9.alliances.density0_4 | readable | [0.953608, 0.987973] | 549.2/327 |
| fig9.alliances.density0_5 | readable | [0.953608, 0.987973] | 605/327 |
| fig9.alliances.density0_6 | readable | [0.965636, 1.000000] | 660.8/320 |
| fig9.alliances.density0_7 | unreadable_overlap | Unavailable | Unavailable |
| fig9.alliances.density1 | readable | [0.982818, 1.000000] | 884/310 |
| fig9.collective_security.density0 | readable | [0.000000, 0.015464] | 326/892 |
| fig9.collective_security.density0_05 | readable | [0.147766, 0.178694] | 353.9/797 |
| fig9.collective_security.density0_1 | readable | [0.621993, 0.704467] | 381.8/506 |
| fig9.collective_security.density0_15 | readable | [0.878007, 0.926117] | 409.7/367 |
| fig9.collective_security.density0_2 | readable | [0.969072, 1.000000] | 437.6/319 |
| fig9.collective_security.density0_25 | readable | [0.972509, 1.000000] | 465.5/317 |
| fig9.collective_security.density0_3 | readable | [0.977663, 1.000000] | 493.4/314 |
| fig9.collective_security.density0_4 | readable | [0.977663, 1.000000] | 549.2/314 |
| fig9.collective_security.density0_5 | readable | [0.979381, 1.000000] | 605/313 |
| fig9.collective_security.density0_6 | readable | [0.981100, 1.000000] | 660.8/312 |
| fig9.collective_security.density0_7 | unreadable_overlap | Unavailable | Unavailable |
| fig9.collective_security.density1 | readable | [0.984536, 1.000000] | 884/310 |
| fig10.tagging.density0_05 | readable | [5.388350, 5.912621] | 352.45/658 |
| fig10.tagging.density0_1 | readable | [4.398058, 4.922330] | 375.9/692 |
| fig10.tagging.density0_15 | readable | [3.815534, 4.339806] | 399.35/712 |
| fig10.tagging.density0_2 | readable | [3.436893, 3.961165] | 422.8/725 |
| fig10.tagging.density0_25 | readable | [2.475728, 3.000000] | 446.25/758 |
| fig10.tagging.density0_3 | readable | [2.708738, 3.233010] | 469.7/750 |
| fig10.tagging.density0_4 | unreadable_overlap | Unavailable | Unavailable |
| fig10.tagging.density0_5 | unreadable_overlap | Unavailable | Unavailable |
| fig10.tagging.density0_6 | unreadable_overlap | Unavailable | Unavailable |
| fig10.tagging.density0_7 | unreadable_overlap | Unavailable | Unavailable |
| fig10.tagging.density1 | readable | [0.757282, 1.000000] | 798/817 |
| fig10.alliances.density0_05 | readable | [6.495146, 7.077670] | 352.45/619 |
| fig10.alliances.density0_1 | readable | [6.233010, 6.815534] | 375.9/628 |
| fig10.alliances.density0_15 | readable | [4.456311, 5.038835] | 399.35/689 |
| fig10.alliances.density0_2 | unreadable_overlap | Unavailable | Unavailable |
| fig10.alliances.density0_25 | unreadable_overlap | Unavailable | Unavailable |
| fig10.alliances.density0_3 | unreadable_overlap | Unavailable | Unavailable |
| fig10.alliances.density0_4 | unreadable_overlap | Unavailable | Unavailable |
| fig10.alliances.density0_5 | unreadable_overlap | Unavailable | Unavailable |
| fig10.alliances.density0_6 | unreadable_overlap | Unavailable | Unavailable |
| fig10.alliances.density0_7 | unreadable_overlap | Unavailable | Unavailable |
| fig10.alliances.density1 | readable | [0.728155, 1.000000] | 798/817 |
| fig10.collective_security.density0_05 | readable | [10.281553, 10.805825] | 352.45/490 |
| fig10.collective_security.density0_1 | readable | [8.097087, 9.378641] | 375.9/552 |
| fig10.collective_security.density0_15 | readable | [5.737864, 6.666667] | 399.35/639 |
| fig10.collective_security.density0_2 | unreadable_overlap | Unavailable | Unavailable |
| fig10.collective_security.density0_25 | unreadable_overlap | Unavailable | Unavailable |
| fig10.collective_security.density0_3 | unreadable_overlap | Unavailable | Unavailable |
| fig10.collective_security.density0_4 | unreadable_overlap | Unavailable | Unavailable |
| fig10.collective_security.density0_5 | unreadable_overlap | Unavailable | Unavailable |
| fig10.collective_security.density0_6 | unreadable_overlap | Unavailable | Unavailable |
| fig10.collective_security.density0_7 | unreadable_overlap | Unavailable | Unavailable |
| fig10.collective_security.density1 | readable | [0.757282, 1.000000] | 798/817 |
| fig11.tagging.density0 | readable | [0.000000, 0.018256] | 317/793 |
| fig11.tagging.density0_05 | readable | [0.014199, 0.050710] | 339.1/777 |
| fig11.tagging.density0_1 | readable | [0.038540, 0.083164] | 361.2/763 |
| fig11.tagging.density0_15 | absent_source_point | Unavailable | Unavailable |
| fig11.tagging.density0_2 | readable | [0.032454, 0.077079] | 405.4/766 |
| fig11.tagging.density0_25 | readable | [0.014199, 0.058824] | 427.5/775 |
| fig11.tagging.density0_3 | unreadable_overlap | Unavailable | Unavailable |
| fig11.tagging.density0_4 | unreadable_overlap | Unavailable | Unavailable |
| fig11.tagging.density0_5 | unreadable_overlap | Unavailable | Unavailable |
| fig11.tagging.density0_6 | readable | [0.109533, 0.146045] | 582.2/730 |
| fig11.tagging.density0_7 | readable | [0.434077, 0.470588] | 626.4/570 |
| fig11.tagging.density1 | readable | [0.981744, 1.000000] | 759/300 |
| fig11.alliances.density0 | readable | [0.000000, 0.020284] | 317/793 |
| fig11.alliances.density0_05 | readable | [0.032454, 0.073022] | 339.1/767 |
| fig11.alliances.density0_1 | readable | [0.097363, 0.137931] | 361.2/735 |
| fig11.alliances.density0_15 | readable | [0.160243, 0.200811] | 383.3/704 |
| fig11.alliances.density0_2 | readable | [0.223124, 0.263692] | 405.4/673 |
| fig11.alliances.density0_25 | readable | [0.237323, 0.277890] | 427.5/666 |
| fig11.alliances.density0_3 | readable | [0.288032, 0.328600] | 449.6/641 |
| fig11.alliances.density0_4 | readable | [0.472617, 0.513185] | 493.8/550 |
| fig11.alliances.density0_5 | readable | [0.697769, 0.738337] | 538/439 |
| fig11.alliances.density0_6 | readable | [0.831643, 0.872211] | 582.2/373 |
| fig11.alliances.density0_7 | readable | [0.939148, 0.979716] | 626.4/320 |
| fig11.alliances.density1 | readable | [0.979716, 1.000000] | 759/300 |
| fig11.collective_security.density0 | readable | [0.000000, 0.018256] | 317/793 |
| fig11.collective_security.density0_05 | readable | [0.089249, 0.125761] | 339.1/740 |
| fig11.collective_security.density0_1 | readable | [0.227181, 0.324544] | 361.2/657 |
| fig11.collective_security.density0_15 | readable | [0.488844, 0.578093] | 383.3/530 |
| fig11.collective_security.density0_2 | readable | [0.693712, 0.774848] | 405.4/431 |
| fig11.collective_security.density0_25 | readable | [0.918864, 0.955375] | 427.5/331 |
| fig11.collective_security.density0_3 | readable | [0.900609, 0.937120] | 449.6/340 |
| fig11.collective_security.density0_4 | readable | [0.951318, 0.987830] | 493.8/315 |
| fig11.collective_security.density0_5 | readable | [0.943205, 0.979716] | 538/319 |
| fig11.collective_security.density0_6 | readable | [0.924949, 0.961460] | 582.2/328 |
| fig11.collective_security.density0_7 | readable | [0.969574, 1.000000] | 626.4/306 |
| fig11.collective_security.density1 | readable | [0.981744, 1.000000] | 759/300 |

## Relationship to judging

The readable mask supplies the [approved design](2026-10-03-democratic-peace-design.md) conditional comparisons. All105 slots remain in Holm multiplicity; unavailable targets retain null reported p-values and reasons. Numeric compatibility would be conditional on reconstruction/statistic definitions and source envelopes. Mobile.15 has text-only source comparisons; it contributes no fabricated figure targets. Source curves lack original seeds, per-history values, variance or a certified2001 executable/RNG, and digitization cannot supply them.
