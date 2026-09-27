# Minds 1: the utility mind and the ideal free distribution (design)

**Date:** 2026-09-27
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 1. Our own experiment, numbered apart
from the reproductions' milestones.
**Builds on:** the milestone 1–23 specs; all remain binding where not changed here. In particular:
the Sugarscape config and its legacy loading, the literal-default-plus-named-switch pattern, and the
preset titles of `crates/sugarscape-core/src/titles.rs`.

**Sources:**
- S. D. Fretwell and H. L. Lucas, "On territorial behavior and other factors influencing habitat
  distribution in birds. I. Theoretical development", *Acta Biotheoretica* 19 (1969; usually cited
  as 1970), 16–36 (*not in `papers/`*).
- G. A. Parker, "Searching for mates", in Krebs and Davies (eds.), *Behavioural Ecology* (1978),
  214–244: the input-matching rule (*not in `papers/`*).
- M. Milinski, "An evolutionarily stable feeding strategy in sticklebacks", *Z. Tierpsychol.* 51
  (1979), 36–40 (*not in `papers/`*).
- D. G. C. Harper, "Competitive foraging in mallards: 'ideal free' ducks", *Anim. Behav.* 30 (1982),
  575–584 (*not in `papers/`*).
- M. Kennedy and R. D. Gray, "Can ecological theory predict the distribution of foraging animals? A
  critical analysis of experiments on the ideal free distribution", *Oikos* 68 (1993), 158–166
  (*not in `papers/`*).

**Critique and follow-ups** (local copies where open, in `papers/ideal-free/`):
- D. J. D. Earn and R. A. Johnstone, "A systematic error in tests of ideal free theory", *Proc. R.
  Soc. B* 264 (1997), 1671–1675 (PMC1688719; *not in `papers/`*: PMC and Europe PMC serve a
  challenge page instead of the PDF).
- W. M. Baum and J. R. Kraft, "Group choice: competition, travel, and the ideal free distribution",
  *JEAB* 69 (1998), 227–245 (PMC1284661; *not in `papers/`*: a scan behind the same challenge).
- W. J. Sutherland, "Aggregation and the 'ideal free' distribution", *J. Anim. Ecol.* 52 (1983),
  821–828 (*not in `papers/`*).
- E. J. Collins, A. I. Houston and A. Lang, "The ideal free distribution: an analysis of the
  perceptual limit model", *Evol. Ecol. Res.* 4 (2002), 471–493: a secondary statement of
  Fretwell–Lucas and Parker, and of the matching regression (copy in `papers/ideal-free/`).

**The utility mind:**
- D. Mark, *Behavioral Mathematics for Game AI* (2009).
- K. Dill and D. Mark, "Improving AI Decision Modeling Through Utility Theory", GDC 2010.
- D. Mark and M. Lewis, "Building a Better Centaur: AI at Massive Scale", GDC 2015 (the
  infinite-axis utility system).
- M. Lewis, "Choosing Effective Utility-Based Considerations", *Game AI Pro 3* (2017), ch. 13
  (free at gameaipro.com; copy in `papers/utility/`).

## Goal

Add a decision seam to the Sugarscape and the first engine behind it: a **utility mind** that
scores candidate sites by multiplying consideration scores. Show that with rule M's single
consideration it *is* rule M. Then measure the book's rule and the utility mind against the ideal
free distribution: input matching, the undermatching of field data, Sutherland's interference
prediction and Baum and Kraft's travel result.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited.
  Every existing config, link, session and sweep reads and runs as before. `decision.rule: book` is
  the default and draws nothing new from `World.rng`.
- **The reduction holds exactly:** under `utility` with every consideration neutral and `idle:
  stay`, every golden Sugarscape preset without combat has its golden fingerprint.
- **One engine path; deterministic; portable.** Native and WASM fingerprints match. `ln` and `exp`
  come from `portable.rs`.
- **Truthful titles and descriptions:** each preset and sweep says what it measurably shows.

## Source summary

- **The ideal free distribution** (Fretwell and Lucas, as restated by Collins et al.): "identical
  animals have complete knowledge of their environment (i.e. are ideal) and are free to move to the
  habitat that offers the highest fitness gains … a stable distribution results when no animal can
  increase its intake rate by switching."
- **Input matching** (Parker, 1978, via Collins et al.): with continuous input where all food is
  eaten, "the stable distribution is found when the ratio of the number of animals approximates
  the ratio of the input rates at the two sites": N_A/N_B = r_A/r_B.
- **Milinski (1979):** six sticklebacks offered two drift-food patches "distributed themselves
  between the two patches in the ratio of patch profitabilities" (a 2:1 ratio in the experiment).
- **Harper (1982):** mallards "distribute themselves between two patches of food in a close
  approximation to the distribution predicted by the ideal free model", although despots take
  unequal shares.
- **Kennedy and Gray (1993):** across reanalyzed experiments, animals underuse the richer patch
  and overuse the poorer one (undermatching). Causes discussed: perceptual limits, unequal
  competitors and travel costs. Their study count and mean slope are not verified here and are not
  used.
- **The matching regression** (Collins et al.; Fagen, 1987): N_A/N_B = b·(r_A/r_B)^s, so s = 1 is
  input matching, s < 1 undermatching and s > 1 overmatching. Verified human-group values: s 0.59
  to 0.86 (Madden et al., 2002).
- **Earn and Johnstone (1997):** "Tests of this prediction have inappropriately compared ratios of
  mean resource levels and mean consumer densities, rather than means of ratios … the theory will
  appear to underestimate the number of consumers occupying poor patches." We average per-sample
  log ratios, never the ratio of mean counts.
- **Sutherland (1983)**, via Doncaster (1999): with mutual interference m (intake ∝ Q·n^(−m)),
  predator density is proportional to input^(1/m). So m = 1 gives input matching, m > 1
  undermatching and m < 1 overmatching.
- **Baum and Kraft (1998):** about 30 pigeons, two continuous-input patches. "When travel was
  required to switch patches, undermatching decreased slightly … the flock's distribution was a
  truly emergent phenomenon."
- **The utility mind** (Lewis, 2017): "Each consideration is essentially a raw numeric input,
  normalized to the interval [0, 1]. This score is processed through a response curve … These scores
  are then multiplied together to obtain the overall score."

## Measured in planning

A throwaway probe used the current CLI (no code) and 20 seeds. The setup:
- a 60 × 40 torus with two cone patches (`peaks`, height 4) centered at (15, 20) with radius 10 and
  at (42, 20) with radius 10, 7 or 5, giving 305 sites against 305, 145 or 69 (input ratio R 1.00,
  2.10, 4.42);
- growback 0.25 per tick, so about 112 Flumps can be fed in all;
- 100 Flumps, all with metabolism 1;
- counts taken at tick 1000, with *s* fitted to the mean of per-seed log(N₁/N₂).

| Vision | Endowment | R 1.00 | R 2.10 | R 4.42 | *s* | Alive | Off both patches |
|---|---|---|---|---|---|---|---|
| 1–6 | 50 | 0.98 | 1.71 | 2.96 | 0.745 | 45, 36, 30 | 0 |
| 1–6 | 100 000 | 0.98 | 1.71 | 2.96 | 0.745 | 100 | 55, 65, 70 |
| 10–20 | 50 | 1.00 | 1.86 | 3.90 | 0.918 | 80, 73, 67 | 0 |
| 10–20 | 100 000 | 1.02 | 1.86 | 3.95 | 0.909 | 100 | 20, 27, 33 |

The R columns give the geometric mean of N₁/N₂.

- **Rule M undermatches**, the direction Kennedy and Gray report. At short vision it's strong
  (*s* 0.75); at long vision it's close to matching (*s* 0.91).
- **Survival plays no part.** The on-patch counts are identical with and without starvation. The
  Flumps who die are exactly those who never saw sugar.
- **"Free" fails.** Rule M keeps a Flump in place when nothing it sees is better (the current site
  wins ties at distance 0). A Flump that starts out of sight of sugar never moves. That's 55–70 of
  100 at vision 1–6, and still 20–33 at vision 10–20, where a filled patch's occupied edge hides the
  sugar behind it.
- **A likely mechanism: catchment, not choice.** At vision 1–6 no Flump can see across the gap
  between patches, so nobody ever switches. A patch's count is set by how many Flumps started within
  sight of it, a catchment growing roughly as (radius + vision)². That grows more slowly than the
  patch's area. The survey tests this by comparing *s* to the catchment prediction.

## Architecture

- **The seam:** `rules::agent_turn` calls `minds::decide(world, id)` where it now calls
  `movement::act`. Under `decision.rule: book` that is `movement::act` itself, unchanged. Under
  `utility` it is `minds::utility::act`. Rule C still decides moves under combat.
- **Shared welfare:** rule M's site welfare (single-good with the pollution discount, and
  multicommodity foresight welfare) moves into one function in `movement.rs` that both callers use.
  The candidate list (the current site at distance 0, then `torus.sight` order, skipping occupied
  sites), `choose`, the move, `social.moved` and the harvest stay rule M's code, shared.
- **Code:** `crates/sugarscape-core/src/minds/mod.rs` (the seam) and `minds/utility.rs` (the
  considerations and the score). Later Minds steps add engines beside it.

## Config

A new object `decision` on the Sugarscape config. `#[serde(default)]`, so older configs load as the
book. `legacy::convert` gets the default.

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `decision.rule` | `book` | reset | `book` (rule M) or `utility` |
| `decision.travel` | 0 | live | k in the travel consideration 1 / (1 + k·d); d is the site's distance |
| `decision.crowding` | 0 | live | m in the crowding consideration (1 + n)^(−m); n is the number of Flumps on the site's von Neumann neighbors, not counting the mover |
| `decision.idle` | `stay` | live | when every candidate scores 0: `stay` (the book) or `wander` (move to a uniformly random unoccupied site in sight) |

**Validation:**
- `travel` is 0–10 and `crowding` is 0–10.
- `utility` with combat on is an error: "rule C decides moves under combat".
- Under `book`, `travel`, `crowding` and `idle` are kept but ignored, as a disabled rule's fields
  are. The Rules panel's note says so.

## The utility mind

Score of candidate site q at distance d: **W(q) × T(d) × C(q)**.
- **W** is rule M's welfare, unchanged: sugar (discounted by pollution when on), or foresight
  welfare over n goods.
- **T(d) = 1 / (1 + k·d)**, the hyperbolic form of delay discounting. At k = 0 it isn't computed,
  and the score is W exactly.
- **C(q) = exp(−m · ln(1 + n))**, Sutherland's intake ∝ n^(−m) applied to local neighbors (n + 1
  counts the mover). At m = 0 it isn't computed. It uses portable `exp` and `ln`.
- **Selection:** `choose` over (site, distance, score), which is rule M's tie rule and draw. Under
  `idle: wander`, when the best score is 0, one uniform draw picks among unoccupied sites in sight
  instead (staying put when there are none).

**Stated choices:**
- **No normalization.** Lewis normalizes each consideration to [0, 1]. Dividing W by a constant
  shared by every candidate doesn't change the ordering, so W stays raw. This is what makes the
  reduction exact.
- **No compensation factor.** The infinite-axis system's compensation factor (secondary sources:
  s + (1 − s)(1 − 1/n)s) corrects products across decisions with different numbers of
  considerations. Every candidate here has the same considerations, so it's left out.
- **Local crowding.** Crowding is local interference, not patch-wide density as in Sutherland.
  That's the analog a Flump can perceive.

## Statistics

These series appear only when goods[0]'s map is `peaks` with at least two peaks, following the
Axelrod pattern of optional columns:
- `on_first_patch`
- `on_other_patches`
- `off_patch`
- `first_patch_share` (`on_first_patch` / (`on_first_patch` + `on_other_patches`), or 0 when both
  are 0)

A Flump is on the patch of its nearest peak when its torus distance to it is less than the radius.
Each patch's nominal input is growback rate × its sites with capacity ≥ 1. It's computed in the
survey and shown in preset descriptions. It's a nominal rate: a site at its capacity stops growing
back, so input equals nominal only while Flumps harvest often enough.

## Presets

The base for all: the probe's world (60 × 40, patches at (15, 20) and (42, 20), height 4,
growback 0.25, 100 Flumps, metabolism 1, endowment 50, vision 1–6). Titles are drafts, to be
replaced by measured wording.

| Preset | Title | Setup |
|---|---|---|
| `ifd-even` | Two equal sugar patches: the Flumps split evenly | radii 10 and 10 |
| `ifd-two-to-one` | One patch yields twice as much: it draws fewer than twice the Flumps | radii 10 and 7 (R 2.10; Milinski's ratio) |
| `ifd-four-to-one` | One patch yields four times as much: it draws under three times the Flumps | radii 10 and 5 (R 4.42) |
| `ifd-far-sighted` | Flumps who can see across the gap come close to matching the yields | R 2.10, vision 10–20 |
| `ifd-no-starving` | Nobody starves, and most Flumps never find sugar | R 2.10, endowment 100 000 |
| `ifd-wander` | Flumps who wander when they see nothing | R 2.10, endowment 100 000, `utility`, `idle: wander` |
| `ifd-crowding` | Flumps who avoid crowds | R 2.10, `utility`, crowding 1 |
| `ifd-travel` | Flumps who prefer nearby sugar | R 2.10, `utility`, travel 0.5 |

## Sweeps

Twenty seeds, 1000 ticks, metric `window_mean` of `first_patch_share` from tick 500:
- `ifd-matching`: x is the input ratio (the second radius 10, 8.5, 7, 6, 5); series are vision
  1–6, 5–10 and 10–20.
- `ifd-idle`: x is the input ratio; series are `stay` and `wander` (endowment 100 000).
- `ifd-crowding`: x is crowding m (0, 0.5, 1, 2, 4) at R 2.10 and R 4.42.
- `ifd-travel`: x is travel k (0, 0.1, 0.5, 1, 2) at R 2.10 and R 4.42.

## Survey

A `minds1` claims module. *s* is fitted from means of per-seed log ratios (Earn and Johnstone), and
the claims are reported whether they hold or fail.

- **Parker:** under the book's rule, N₁/N₂ matches R₁/R₂ (s within 0.9–1.1) at vision 1–6 and at
  10–20.
- **Kennedy and Gray:** the book's rule undermatches (s < 1), at each vision.
- **Survival:** s is the same with and without starvation (no-starving against starving).
- **"Free":** under `stay`, a stated share of Flumps is still off patch at tick 1000. Under
  `wander` the off-patch share falls to under 5 %, and s moves toward 1.
- **Catchment:** at vision 1–6, s is within 0.1 of the catchment prediction. That prediction is
  the log ratio of the two patches' sight-catchments (sites from which a patch site is in sight)
  over the log input ratio.
- **Sutherland:** m = 1 gives input matching (s within 0.9–1.1). The expectation is that this
  fails: local crowding pushes Flumps apart, so it can only lower an s already below 1. The
  measured direction, with s against m, is reported.
- **Baum and Kraft:** travel cost reduces undermatching. The direction is measured and reported
  either way. Here travel is a distance preference under the book's one-tick jump, not a cost of
  switching, and the survey says so.

## Page

- **Rules panel:** a **Decision** group in `web/src/schema.ts`, with `rule` (select, reset),
  `travel` and `crowding` (numbers) and `idle` (select). Its note says the last three apply under
  the utility rule.
- **Charts:** a **Patches** chart (`on_first_patch`, `on_other_patches`, `off_patch`), shown when
  the series exist, with `first_patch_share` in the same section.
- **Types:** `web/src/types.ts` gains `decision?` and the optional snapshot fields.
- Inspect is unchanged in this step.

## Testing

- **Golden and legacy:** existing entries untouched; new entries for the eight presets; titles for
  each (the titles test, with the counts in `TITLES` and `every_preset_is_valid_and_runs` bumped).
- **Reduction** (`tests/minds.rs`): every Sugarscape preset without combat, run with `decision.rule:
  utility` and neutral considerations, has its golden fingerprint.
- **Core unit tests:**
  - each consideration on hand values, and neutral values giving W exactly;
  - crowding counts that exclude the mover;
  - `wander` drawing only when every score is 0, and staying when no site is free;
  - no extra draws under `book` or `stay`;
  - validation, live and reset fields, and legacy loading;
  - the patch series on and off, patch membership at radius edges and on the torus seam;
  - `first_patch_share` at 0 Flumps.
- **WASM:** a pinned golden fingerprint for `ifd-crowding` (it uses `exp` and `ln`).
- **Web:** the Decision group and the Patches chart; a sweep over an `ifd-*` base.

## Docs

- **README:** a Minds section covering the seam, the utility mind, its switches, the presets and
  sweeps, and the findings as measured.
- **`docs/studies/2026-09-27-minds.md`:** Minds 1's status and results.
- **Roadmap:** a Minds line.

## Amendments (implementation)

These change or extend the sections above. The measured values are the survey's (20 seeds, tick
1000) and the sweeps'.

- **Crowding and travel are tested at vision 10–20, not 1–6.** At vision 1–6 no Flump sees both
  patches, so neither knob has anything to weigh, and the patch split is unchanged: the share is
  0.6366 at every value of either knob. The `ifd-crowding` and `ifd-travel` presets, their sweeps,
  and the Sutherland and Baum–Kraft claims all use vision 10–20, in both arms.
- **The survival claim is judged paired.** The same seeds run with and without starvation, so the
  claim is judged on per-seed differences in *s* (within 0.05 in at least 80 % of seeds). *s* is
  identical in 20 of 20 seeds: Holds. An unpaired equivalence test, which sees only the spread
  across seeds, had called it Weak.
- **The catchment claim keeps a strict per-seed judge, and fails.** Prediction 0.709; the seeds'
  median 0.722 (IQR 0.59–0.92); only 6 of 20 within 0.1, against the 80 % asked.
- **Sutherland: the expectation was wrong.** The spec expected the claim to fail and crowding to
  lower *s*. Measured: crowding m = 1 raises *s* from 0.896 to 0.954 (one-sided Mann–Whitney
  p ≈ 0.0003), with 18 of 20 seeds within 0.9–1.1, so Sutherland's matching holds. The sweep's
  share at R 2.10 rises from 0.649 at m 0 to about 0.659 from m 0.5 to 4.
- **Baum and Kraft: the direction is reversed.** Travel k = 0.5 lowers *s* from 0.90 to 0.73
  (their claim fails). Their travel was a cost of switching; ours is a preference for nearby sugar
  under rule M's one-tick jump. The sweep's share at R 2.10 falls from 0.649 at k 0 to 0.602 at
  k 0.5, then recovers a little (0.613 at k 2).
- **Wander frees the Flumps but lowers *s*.** A median 0.005 of Flumps are off patch ("free"
  holds), but *s* falls from 0.72 to 0.40, so "toward matching" fails. The wanderers overfill the
  poorer patch (41 Flumps on its 36 sugar a tick), so the split likely follows where they arrive.
  That cause is not measured.
- **Other survey results.** Parker fails at vision 1–6 (median *s* 0.72, 4 of 20 within 0.9–1.1)
  and at 10–20 (0.90, 8 of 20). Undermatching holds at 1–6 (17 of 20), 5–10 (median 0.63, 20 of
  20) and 10–20 (20 of 20). Stuck holds: a median 66 % off both patches with nobody starving,
  over half in 20 of 20 seeds. These replace the planning probe's numbers.
- **Patch membership** is the nearest peak, when the site is within that peak's radius. That
  equals "capacity ≥ 1" only when patches don't overlap, as in every Minds 1 preset. On
  overlapping peaks the two can differ; that's out of scope here.
- **The reduction test skips presets whose own rule is the utility mind** (`ifd-wander`,
  `ifd-crowding`, `ifd-travel`): they are not rule M. Their book counterparts are covered.
- **Sources.** `papers/` is gitignored (local copies). Collins, Houston and Lang (2002) and Lewis
  (2017) were saved. Earn and Johnstone (1997) and Baum and Kraft (1998) could not be fetched: the
  hosts served a challenge page.
- **Titles** follow the measurements: `ifd-wander` "Flumps who see no sugar wander: nearly all find
  a patch, but the split strays from the yields"; `ifd-crowding` "Flumps who avoid crowded sugar
  come closer to matching the yields"; `ifd-travel` "Flumps who prefer nearby sugar stray further
  from matching the yields".
