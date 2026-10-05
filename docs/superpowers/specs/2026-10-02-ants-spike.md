# Spike: ants (Following the Crowd, episode 8, "Ants at two food piles")

**Date:** 2026-10-02
**Status:** built and verified after source, storyboard, and tune approval.
**Sources:** Alan Kirman, *Ants, Rationality, and Recruitment*, QJE 108(1), 137–156 (1993); Simone Alfarano and Mishael Milaković, *Should Network Structure Matter in Agent-Based Finance?*, Warwick WP07-02 (2007), subsequently published under a different title in JEDC (2009). The local 2007 working paper is the network source used here.

## Intent and constraints

Continue the established series with model-generated crowds choosing between identical food sources. Explain recruitment, spontaneous switches, and how population size and contact rules affect herding. Separate a displayed random realization from a typical outcome, and an empirical temporary split from a model's stationary distribution. Preserve the existing closing-card format and test every frame for hidden or clipped text. Use American spelling and "agent" in code and documentation; the video calls the characters Flumps. Prepare the storyboard and original score for user approval before building. Commit only after user authorization.

## What the source audit establishes

Kirman's real-ant observation is an approximate 80–20 imbalance **for a while** (pp. 138–139), followed occasionally by a switch. His chain explains asymmetric crowds and spontaneous switches. He explicitly describes Figure IIb as nearly all agents on one side and distinguishes its changing states from its equilibrium distribution (pp. 145–147). A transient 80–20 observation does not require stationary density modes at 80 and 20.

For the valid printed equation (1), the exact stationary distribution is beta-binomial, with alpha = epsilon*(N−1)/(1−delta). It is U-shaped below alpha = 1, uniform at 1, and centered above. Consequently it does not select an interior preferred 80–20 split, although finite trajectories can pass through or spend time near such splits. The episode must state this stronger quantitative distinction without calling the qualitative explanation a failure.

Figure IIb's near-half average describes its illustrated 100,000-meeting realization, sampled every fiftieth meeting. It does not claim every short realization averages half. Current core seed 1 averages 0.529835 at the printed horizon. The original 20-seed survey has 4 near-half means (0.4–0.6); the expanded canonical core audit has 229/1,000 (22.9%, Wilson 95% interval 20.4–25.6%); an independent dependency-free scalar implementation has 278/1,000, Wilson 95% interval 25.11–30.66%. A separate NumPy ensemble has 233/1,000, interval 20.79–26.02%. The initial independent estimates differ by about 2.3 standard errors. An additional 10,000 disjoint scalar seeds give 2,493 near-half means (24.93%, Wilson 95% interval 24.09–25.79%). Matching supplied random draws produce identical Python and Rust trajectories; initialization, rates, horizon, and sampling agree. All audit corpora are retained, rather than selecting a favorable estimate. Longer records improve convergence to the symmetric mean; they are not required for a compatible Figure IIb example. The compatibility interpretation and interval are a retrospective audit rule, revised after the earlier result was known.

The independent log-gamma stationary calculation matches equation (1) detailed balance within 2e−14. Figure IIb's alpha is 0.2 and exact share variance 0.179285714286. Figures Ia, Ib, and Ic have variances 0.12625, 0.084442567568, and 0.008199013158. Figure Ib's printed parameters are nearly uniform, rather than exactly at the threshold.

Kirman proposes stronger recruiting toward a majority but supplies no numerical formula (p. 149). Our explicit majority-pull factor and probability cap, with Figure Ic parameters and pull 1, yield exact modes at 18 and 82. This is our specified version of his suggested extension, not a reproduced numeric prediction from the paper. Positive spontaneous switching keeps the finite chain irreducible: a long record without flips is a finite-horizon near-lock, not permanent absorption (p. 147).

The exact current split determines the future transition distribution. An operational 80%-to-20% regime groups different exact states together; its age-conditioned residual lifetime does not directly test the chain's Markov property (p. 152). Keep the pooled residence diagnostic distinct from the mathematical property.

Alfarano and Milaković use equation (18), probability (a + lambda*n_opposite)/(a + lambda*N), with immediate sequential updates and N updates per sweep (pp. 15–17). This counts all opposite-state neighbors, whereas Kirman draws one partner per meeting. These time units and contact rules must be labeled explicitly when comparing them.

Figure 3 specifies alpha 0.5, 1, and 2 and 100,000 sweeps, but does not state N. At our stated N = 100, an independent implementation gives ring variances 0.08533, 0.05665, and 0.03586 across 20 seeds, versus continuous-beta predictions 0.125, 0.08333, and 0.05; random networks are much closer. The ring shortfall persists through one million sweeps. At N = 50 the same model meets the survey's 15% agreement criterion at all three alpha values. N = 50 was a retrospective sensitivity check after seeing the N = 100 result, using the preexisting 15% criterion. The missing N therefore prevents calling the published figure an exact failed reproduction. The authors already acknowledge a smaller same-direction ring and small-world deviation in their Figure 4 discussion (pp. 17–18). Report any ring disagreement conditionally on our N and configuration.

Figure 4 uses 300,000 sweeps, N starting at 50 and increasing by 500, a = 0.5, lambda = 1 (pp. 17–18). Its qualitative N-dependence comparison is conditional on those parameters and network families. Footnote 18 says displayed variance is divided by 3, and displayed inverse variance multiplied by 3; remove this display scaling when comparing raw values. The three audit sizes check the qualitative trend; they do not reproduce the paper's full regression over sizes up to roughly 5,000. The audit survey horizon is being matched to the paper; episode measurements will use at least 20 seeds.

Kirman's majority-shrinking observation follows the U-shaped regime and relatively small self-conversion discussion (p. 144). Under epsilon < (1−delta)/(N−1), P(k, k−1) decreases throughout the majority range. Figure Ic lies outside that premise; its small-majority behavior is a contrast, not a failed prediction in the stated herding regime. The corrected scope is a retrospective source interpretation.

## Audit evidence

Original Figures I, IIa, and IIb and AM Figure 3 were rendered and compared beside reconstructed distributions and actual core traces/densities. Independent scalar and sequential network implementations share no model simulation code. Reports, scripts, seed records, and figures are retained in `/tmp/ants-audit/` during review. The initial survey was run with both `--only ants` and `--only am`: AM claim IDs have a separate prefix, so the former alone does not select the full milestone.

## Corrected survey verdicts

Both selectors completed successfully after the corrections: 15 `ants.*` checks and four `am.*` checks, all Hold. These verdicts apply to the claim text's stated scope, not to stronger interpretations. Application diagnostics are explicitly distinguished from source claims and disclose retrospective revisions.

| Group | Verdict and scope |
|---|---|
| Equation (1), stationary law, threshold and special cases | Hold: independent exact calculation and long-run simulations agree |
| Figure IIa | Hold: near-half fluctuations at the printed settings |
| Figure IIb | Hold: extremes, rapid crossings and a compatible near-half illustrative mean; 229/1,000 core records match the diagnostic interval |
| Stationary 80–20 modes | Hold, application diagnostic: no preferred off-center interior mode in the valid base-chain grid; transient empirical plateaus were not tested by that grid |
| Majority-shrinking probability | Hold in the source's stated herding regime, Figures Ia and IIb; Figure Ic is an outside-premise application contrast |
| Pooled regime residence | Hold, application diagnostic: approximate age comparison at the tested ages, not a test of exact-state Markov conditioning |
| Majority pull and additional sources | Hold for our stated extension formula and occupancy statistic; no permanent lock-in or invariance of every statistic asserted |
| Population growth | Hold: weaker herding under fixed pairwise habits |
| AM Figure 3 neighborhood comparison | Hold, retrospective application diagnostic at N = 100; omitted paper N prevents an exact figure-failure verdict |
| AM Figure 4 | Hold: three-size qualitative trend at 300,000 sweeps; raw inverse-variance slopes 0.463, 0.482, 0.387 and 0.003 (ring, small world, scale-free and random). This is not the paper's full-size regression |
| Pairwise rule on random graphs | Hold, application contrast: extra links do not make a single encounter count multiple partners |
| Independent agents | Hold: less variance when non-herders participate in the network |

## How to film it

Use two identical food piles on one felt stage, with clear left/right source labels. Each agent's source, identity and independent status must come from the actual model dump. Source membership determines which pile it stands by; keep identity through moves. Food quality, quantity and distance convey symmetry and do not change. This is a visual representation of source choices, not a model of ant walking paths.

Extend the existing frame exporter test-first with an `ants` shot. Export each agent's source and independent status, the counts, actual network links where applicable, and the model time unit. Use one meeting per tick for the teaching close-ups, and the usual batches for the crowd shots. Any highlighted recruitment pair or spontaneous-switch event must be recorded by the real update, not inferred or invented. Select illustrative event windows by a stated deterministic rule after measuring; their selection is not evidence of frequency.

Reuse the felt histogram and time diagrams to show the source-share trace and measured time spent at each split. Label exact stationary curves separately from measured histograms and selected traces. A selected short run may show a near-half mean; a small distribution of all measured short-run means explains why that is possible without typical half-and-half attendance.

Reserve distinct screen regions for the stage, one explanatory diagram, and captions. Avoid floating cards over world labels. Show links only while teaching the neighbor rule; replace dense links with measured diagrams in the growth comparison. Retain text bounds/occlusion checks over every rendered frame and caption, including camera-move endpoints. Inspect scratchpad stills at 50% size and sixteen samples before the preview.

The closing card follows the established format: title and source attribution on the first line, `ndouglas.github.io/SugarScape` on the second, above one camera-facing agent that blinks once near the end.

## Proposed storyboard

About 98 seconds after dissolves. Captions below are exact proposed text; `\n` is a line break. They are conditional on the prospective episode rules below passing.

| # | Beat | Exact caption | Image |
|---|---|---|---|
| 1 | equal piles | "Two identical food piles.\nReal ants sometimes crowded one, about 80–20." | Identical piles; brief source-attributed observational premise, without pretending this is a new model measurement |
| 2 | choices | "Kirman's Flumps choose between two sources.\nNeither is better." | Real 100-agent source choices and live counts |
| 3 | meet | "Meet another Flump, and you may copy its choice." | One actual recruitment encounter, slowed, with a single connector |
| 4 | independent switch | "Occasionally, a Flump switches on its own." | A recorded spontaneous event, with the food unchanged |
| 5 | crowd | "With strong recruiting,\nnearly everyone crowds one source." | Figure IIb settings, actual crowd and a short trace |
| 6 | flip | "Then the crowd can flip,\nwithout the food changing." | Actual change from one 80% regime to the other; no outside push |
| 7 | average | "Both sides get turns.\nA short run needn't average half and half." | Selected short trace beside the distribution of all short-run means; compatible examples identified as examples |
| 8 | preferred splits | "The long-run peaks are at all-or-nothing.\n80–20 is not a preferred split." | Measured histogram plus the separately labeled exact distribution at Figure IIb settings |
| 9 | majority pull | "Kirman suggested stronger attraction to the majority.\nOur version favors splits near 18–82." | Our specified Becker-style extension; exact modes and measured histogram |
| 10 | more agents | "Ten times the Flumps, with the same habits:\nless time crowded at one source." | N = 100 and 1,000; equal total meetings, measured extreme occupancy |
| 11 | neighbors | "Alfarano and Milaković counted neighbors,\nrather than one partner per meeting." | Actual links and opposite-state neighbors; label the changed rule and sweep unit |
| 12 | growth and links | "With their rule, growth weakens herding on rings.\nRandom networks keep their swings." | Fixed ring degree 10 versus random link probability 0.1, with measured variance at three stated sizes |
| 13 | non-herders | "A few Flumps who never copy\ncan calm the crowd." | Same network rule, 5% visibly marked independent agents, measured variation with/without them |
| 14 | end | "Ants at two food piles - After Kirman, 1993; Alfarano & Milaković, 2007\nndouglas.github.io/SugarScape" | Standard closing card, one gaze and one late blink |

## Prospective episode measurement rules

The source audit informs these rules, but the episode corpus has not been measured. Fix the full configurations, horizons, thresholds and seed lists before running. Use core seeds 200001 upward, disjoint from this episode's audit core seeds 1–1,000. Preserve all outcomes. A revised caption or judge must explicitly say the revision came after the result; do not change a threshold silently.

- **Observed premise and rules (beats 1–4):** the empirical 80–20 observation is source attribution, not a simulator verdict. Validate symmetry, exactly two sources and the actual event-export semantics with deterministic tests. For micro-shot configurations, use at least 20 seeds; one meeting per tick changes the batching, not the encounter rule. Choose the smallest eligible measured seed and earliest qualifying event window for a recruitment example and a spontaneous-switch example.
- **Strong recruiting (beats 5–8):** N = 100, epsilon = 0.002, delta = 0.01, pull = 0, complete graph, random start, equation (1) conversion, 50 meetings per step. Twenty seeds, 200,000 measured steps each (10 million meetings), no omitted outcomes. Require mean extreme occupancy (share <= 0.2 or >= 0.8) > 0.65 and at least one 80%-to-20% flip in at least 18/20 long records. Aggregate histogram must match the exact distribution within total variation 0.03. Exact stationary modes must be 0 and 100. Short illustrative films retain these behavioral parameters and explicitly show their finite recorded horizon.
- **Finite-record averages (beat 7):** 1,000 seeds 200001–201000, 2,000 steps each, same configuration. Require at least 15% and at most 35% of means in 0.4–0.6, and interquartile range of run means > 0.20. This is an audit-informed prospective episode rule, not Kirman's asserted prevalence. Show the actual count if a numeric label is used. Long-run and short-run samples are paired by seed where they overlap; they are not pooled as independent replicates.
- **Our majority-pull formula (beat 9):** N = 100, epsilon = 0.15, delta = 0.3, pull = 1, other settings as above. Twenty seeds, 20,000 steps each. Require the exact capped-rule stationary modes at 18 and 82, and aggregate empirical total variation from that exact distribution <= 0.04. The stated factor is ours and was selected before this episode's measurements, informed by the milestone audit.
- **Larger pairwise crowd (beat 10):** compare the 20 strong-recruiting seeds with N = 1,000, 500 meetings per step and 20,000 steps (the same 10 million meetings). Other behavioral parameters are unchanged. Require larger-population mean extreme occupancy < half the smaller population's, and lower median time variance. Display both N and meeting counts; the 80% criterion stays fixed.
- **Neighbor-count growth (beats 11–12):** AM equation (18), a = 0.5, lambda = 1, fixed-index sequential immediate updates. Ring degree 10 and random link probability 0.1; N = 50, 550, 1,050; 20 seeds per configuration, 300,000 sweeps per seed to match the paper's duration. Require ring mean variance at N = 1,050 < 0.15 times its N = 50 value; random variance ratio in 0.7–1.4, and random/ring variance ratio > 5 at N = 1,050. Report each mean and independent-seed uncertainty. This is a three-size comparison, not the paper's full regression or a Figure 3 reproduction claim.
- **Independent agents (beat 13):** AM random graph, N = 1,000, p = 0.1, a = 0.5, lambda = 1; q = 0 and 0.05, 20 seeds per configuration, 300,000 sweeps. Require mean variance with non-herders < half that without them. Compare independently generated graphs honestly unless the implementation provides genuinely shared graph draws; do not claim a matched graph just from matching numeric seeds. Export and visibly mark the actual independent agents.

All filming examples must be selected by reproducible criteria from the retained episode corpus, with an example label where needed. No displayed behavior may be staged to make a threshold pass.

## Original tune: *The Turning Chain*

The listening revision follows the approved Breton direction: an original Breton-inspired instrumental dance in D Dorian and 2/4. Oboe and clarinet, equally placed on opposite sides of the stereo field, pass a repeated-note subject over a quiet accordion drone/pulse. The answering reed joins during the caller's last beat, then carries the phrase alone. This adapts the overlapping handoff of [kan ha diskan](https://orchestrenationaldebretagne.bzh/lexique/kan-ha-diskan/); it does not claim to reproduce that vocal practice.

Communal repetition supplies continuity while the lead changes, matching identical sources whose crowds switch endogenously. A solo passage concentrates the phrase in one voice, and later sections reverse the musical roles. Shared phrases vary the texture within the continuous arrangement. The independent-agent passage leaves a small, steady D line that does not imitate the reeds. The harmony stays grounded in a drone, without a triumphant winner or a sinister losing side.

The eight-bar opening call and answer (the rising strain later uses Dorian B natural):

```abc
X:1
T:The Turning Chain
C:Original instrumental episode dance
M:2/4
L:1/8
Q:1/4=109
K:Ddor
V:oboe clef=treble
V:clarinet clef=treble
V:drone clef=bass
[V:oboe] D2 D E | F E D2 | A2 A G | F2 E D | z4 | z4 | z4 | z4 |
[V:clarinet] z4 | z4 | z4 | z2 D E | D2 D E | F E D2 | A2 A G | F2 E D |
[V:drone] D,2 A,2 | D,2 A,2 | D,2 A,2 | D,2 A,2 |
 D,2 A,2 | D,2 A,2 | D,2 A,2 | D,2 A,2 |
```

The 88-bar form `AABBCDDEEFG` fits the 98.2-second cut at approximately 109 quarter notes per minute. One continuous score carries the arrangement at that tempo, without additional stings or volume ducks. Each call-and-answer section limits the two reeds’ overlap to the caller’s final beat; the answering reed then repeats the phrase alone. The solo section retains its uninterrupted lead.

## Verification and approval

The storyboard and initial *Equal Portions* score were approved before building;
the user subsequently approved trying the Breton-inspired direction used in *The Turning Chain*. The source
correction suite passes 58 survey tests; both source selectors report Hold for
all 15 ants and four AM claims within their corrected scope. Independent
correction review and Opus source/numerical review approved the corrected
interpretation. The model dynamics are unchanged.

The prospective episode corpus retains all 1,220 native series and 54,800,000
post-initial samples, plus twenty 1,000-meeting event records. All six numerical
judges pass unchanged. Full configurations, seed lists, outcomes, histograms,
selection rules, and trace limits are in `studio/episodes/ants/measurements.json`.
Chronological data is reduced as it streams; every sample contributes to the
retained statistics and histogram. Background traces are sampled separately;
filmed traces use their actual short frame dumps.

| Episode check | Measured result |
|---|---|
| Strong recruitment | 79.35% extreme occupancy; flips in 20/20 records; histogram TV 0.01164; exact modes 0/100 |
| Finite-record means | 247/1,000 means in 0.4–0.6; IQR 0.39192 |
| Our majority pull | Exact modes 18/82; histogram TV 0.02745 |
| Larger pairwise population | 21.57% extreme occupancy, versus 79.35%; lower median time variance; equal 10 million meetings/run |
| Neighbor-count growth | Ring variance ratio 0.04695; random ratio 0.93601; random/ring 12.20392 at N = 1,050 |
| Five percent independent | Variance ratio 0.31254, using separately generated graphs |

The implemented fourteen beats render every agent at the actual population
size. Recorded micro events and the selected flip retain their actual source,
partner, and model clocks. *The Turning Chain* uses one continuous score with overlapping reed replies
limited to the caller’s final beat. The cut is 98.2 seconds.

Fresh verification passes workspace formatting and strict Clippy, 1,575
workspace tests (100 existing ignored), 72 WASM Node tests, web WASM build and
TypeScript, 847 web tests, and 386 studio tests. Four real Blender regressions
check every frame and caption, curve geometry, visible teaching markers, and
the actual single late closing blink. All fourteen first/middle/end stills and
separate captions were checked at 50% size and sixteen samples. Each task's
independent review approved both compliance and quality.

The preview build and full decode pass: 2,946 frames at 30 fps, 960×540, with
video and stereo audio both exactly 98.2 seconds. The encoded movie's captions,
recorded transitions, diagrams, and open/blink/reopen closing frames were
inspected. The verified copy is
`~/Movies/Flump Studio/sugarscape-026-ants.mp4`. Final independent integration and artifact review approved the episode.
