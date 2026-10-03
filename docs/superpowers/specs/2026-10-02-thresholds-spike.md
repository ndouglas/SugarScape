# Spike: thresholds (Following the Crowd, episode 7, "The riot that needs one person")

**Date:** 2026-10-02
**Status:** source and numerical audit complete; corrected survey and independent Opus review approved. Storyboard and tune approved; measured episode, still review and preview complete.
**Sources:** Mark Granovetter, *Threshold Models of Collective Behavior*, AJS 83(6),
1420–1443 (1978); Duncan J. Watts, *A Simple Model of Global Cascades on Random Networks*,
PNAS 99(9), 5766–5771 (2002). Both original local PDFs in `papers/thresholds/` were read,
including the scanned Granovetter paper by OCR and visual inspection. Milestone 25's
design, README, paper ledger, core rules and survey were checked first.

## What the source audit establishes

Granovetter's central example reproduces exactly: one agent at each threshold from
0 to 99 produces a riot of 100. Replace the threshold of 1 with another 2 and only
the instigator acts (pp. 1424–1425). The two means differ by just 0.01 percentage point.
His argument concerns aggregation: an outcome does not reveal the crowd's individual
dispositions. Thresholds remain fixed during this example; nobody needs to change opinions.

Figure 2 is a continuous CDF equilibrium, explicitly explained by intersections and
tangency (p. 1427). It reproduces: critical spread 12.2220868, tangency at 6.200325 agents;
the equilibrium is 5.479220 at spread 12.2 and almost 100 at 12.3. Finite quantile
constructions and randomly sampled crowds are separate experiments. Their rounding
or smoother probability curve does not refute his continuous figure.

Granovetter already explains why sampling from the uniform city changes the result
(p. 1431). Our 5,000-crowd survey gives 50.5 % ending at zero or one, and 2.3 % reaching
100. An independent 100,000-crowd sample gives 50.064 % and 2.726 %. Exact probabilities
for zero and one sum to 50.1358 %. These support his instability argument. The 0.51
in his paper is an approximation, not a demand that a finite sample exceed 51 %.

His friendship examples (pp. 1429–1430) reproduce under our declared random-tie sampling
and denominator. His movement and ceiling discussions are extensions with unspecified
details. Our chosen rules illustrate them. Figure 3 itself shows one net-benefit zero
crossing; the two-crossing extension is discussed beside it (pp. 1438–1439).

Watts's fixed-threshold cascade window, connected-component size and lower-edge power-law
slope reproduce. At threshold 18 %, the analytic window is approximately mean degree
1.0207–5.7647. The finite network's tail can extend beyond that boundary. A degree of
five or fewer makes an agent vulnerable to one active neighbor; high degree also makes
one active neighbor a smaller fraction of the group.

His Figure 3 upper-critical frequency remains unresolved. Its caption prints 1,000 nodes
and mean degree 6.14, without restating the threshold; we inherit 18 % from Figure 2.
The actual core gives 232/1,000 near-system-size cascades in fresh-world trials, and a
separate C++ model gives 224/1,000. For 10,000 nodes these counts are 20 and 17. The
original reports one, and its smallest plotted positive fraction is 0.0001, which is
inconsistent with a one-node seed in a 1,000-node network. A separate 100,000-node check
gives 0/1,000, statistically compatible with a rate of one in 1,000. That diagnostic
does not establish the original configuration. The core validates at most 20,000 agents;
the larger check belongs to the independent implementation. We do not narrate the
printed frequency as reproduced, or guess the cause of the mismatch.

Figure 4a plots analytic regions, not frequency at every interior point. A normal
conditioned and normalized on [0, 1], reconciling the model's stated support, expands
the dense edge and maximum threshold extent while moving the sparse edge inward.
At normal location 0.18 and spread 0.1 the analytic degree interval is approximately
1.1479–39.4107. Our core's clipped-normal/contact-required experiment is a separately
declared sensitivity test. Neither lower frequency at one sparse setting nor the
smooth-versus-stepped appearance of the reconstructed contours establishes a failure
of the qualitative range claim. Conditioning changes the distribution's realized mean;
0.18 here denotes the input normal location.

Figure 4b's literal normalized integer family, with degree at least 1, exponent 2.5,
and only the exponential cutoff varying, has mean below 1.9474. It cannot span the
figure's degree axis to 30. A different minimum degree, scaling or family could produce
other windows; the paper does not specify it. This limitation applies to the printed
family, rather than every possible intended network model.

Selecting the maximum-degree seed has a per-node advantage at both tested degrees.
Watts's abstract says that advantage is absent in the second regime, and the body
says sharply peaked networks near the upper boundary do not display disproportionate
hub susceptibility. Our selected-node results are in tension with that language.
His body also discusses relative advantage, provides a formula allowing residual
degree dependence, and notes the greater population frequency of average-degree
triggers (p. 5771). These conditioning distinctions do not erase his per-node claim.
Our degree 5.5 comparison is not at a rare-cascade boundary. A follow-up at 2,000
nodes and degree 6.6 does reach a rare finite upper point: the core gives 16/1,000
random successes versus 97/1,000 hub successes; the independent paired model gives
26 versus 96. In both, success means at least 10 % of the population, and the large
events also exceed 90 %. A per-trigger hub advantage persists at these finite settings.
The ratio in the core is about 6.06, compared with average maximum-degree/mean-degree
ratio 2.58; the simple source approximation does not fit this finite treatment.
These degrees are beyond the infinite-network analytic window, so finite nucleation,
maximum-node conditioning and asymptotic approximation must be distinguished. The
historical assertion remains unresolved; the episode does not narrate its falsification.

## Audit method and figures

The actual Rust core was run from a separate throwaway helper. Every network cell uses
1,000 fresh worlds with seeds 100001–101000, separate from the survey's repeated draws.
The independent implementation uses its own PRNG and monotone event-queue closure.
Its edge probability is the paper's z/n; the core uses z/(n − 1). Both settle each
irreversible run; update order changes timing but not its final closure.

Side-by-side originals and reconstructions were inspected for Granovetter Figure 2,
Watts Figures 3, 4a and 4b. Additional plots compare normal-tail conventions and hub
targeting. Artifacts, raw CSVs and scripts are in `/tmp/thresholds-audit/figures/`;
the independent check is in `/tmp/thresholds-audit/minimal/`. Reports:
`source-report.md`, `figure-report.md` and `minimal-report.md` in `/tmp/thresholds-audit/`.
The network-size diagnostic tests compatibility, not exact historical reconstruction.

## Corrected survey verdicts

Both commands completed successfully: `(cd survey && cargo run --release -q -- --only thresholds)`
and the corresponding `--only watts` command. The prefixes select separate groups:
thirteen Granovetter checks and seven Watts checks. Together, 19 Holds and 1 Fails.
The failed check is the qualified Figure 3 upper-frequency reconstruction. No core
behavior was changed. The finite-quantile, sampled-normal, analytic-region and selected-hub
rules explicitly disclose revision after their earlier results were known. `App` denotes
an added realization or illustrative rule, rather than a failed paper claim.

| Check | Scope | Verdict |
|---|---|---|
| Uniform and perturbed crowds | Granovetter's exact example | Holds |
| Figure 2 | Continuous CDF equilibrium | Holds |
| Quantile rounding | App finite realization | Holds |
| Sampled normal transition | App finite realization | Holds |
| City sampling | Granovetter's instability calculation | Holds |
| Friendship weight | Declared tie sampling | Holds |
| Friendship density | Declared tie sampling | Holds |
| Friendship symmetry | Declared tie sampling | Holds |
| Small friendship rescue | Declared tie sampling | Holds |
| Uniform crowd with friends | Declared tie sampling | Holds |
| No oscillation without removal | 5,000 completed episodes for each of three configs | Holds |
| Ceiling pulses | App constructed extension | Holds |
| Movement between crowds | App illustration of conjecture | Holds |
| Cascade window | Watts's fixed-threshold settings | Holds |
| Connected-network size | Watts's fixed-threshold settings | Holds |
| Figure 3 lower-edge slope | Declared finite ensemble | Holds |
| Figure 3 upper frequency | Printed size and degree, inherited threshold | Fails |
| Figure 4a range | Analytic extent, explicit unit-normal convention | Holds |
| Figure 4b vulnerability | Literal degree family and declared grid | Holds |
| Maximum-degree triggers | App paired per-trigger comparison | Holds |

The survey's upper frequency is 199/1,000 at 1,000 nodes and 29/1,000 at 10,000 nodes;
the separate fresh-world corpus gives 232 and 20. These are distinct seed protocols,
not alternative counts from the same batch. At the survey's estimated 0.199 rate,
the plug-in log10 probability of at most one event in 1,000 is −93.97. This is a
compatibility diagnostic with an estimated probability, not a formal p-value for
the authors' unknown settings. The tolerance was made explicit after results were known.

The paired hub survey uses 1,000 identical graph pairs at each degree. At degree 1.3,
counts are 948 hub successes versus 396 random successes; at 5.5 they are 889 versus
477. The exact one-sided paired McNemar tests use the discordant pairs, with probabilities
approximately 6.93e−143 and 2.64e−82. Neither those tests nor the rare-point diagnostic
tests the historical figure's undocumented implementation directly.

## How to film it

A 10 × 10 felt square holds the 100-agent crowd, ordered by threshold. Calm bodies are
blue; agents who join turn coral and hop once. A persistent counter gives actual
participation. The opening close-up labels thresholds 0, 1 and 2 as the number of
other agents required. A ring identifies the instigator. Thresholds stay visible in
a small ruler panel as the action changes; the single changed threshold is highlighted.
No invented meeting or influence event is recorded as a simulation event.

The sampled-city and friendship beats use actual example runs beside measured outcome
histograms. A displayed example is labeled as an example; the batch counts establish
frequency. The ceiling beat marks the agents who can leave, displays its simultaneous
update rule and draws participation against actual simulation step.

For Watts, use a fixed grid of 1,000 agents with actual graph links. Show only the
selected agent's neighborhood in the explanatory close-up; the full board and counter
still represent the full population. A panel gives actual participation over time and
the distribution across 1,000 measured trials. Label the population and mean degree.
The sparse scene favors small outcomes, the middle scene a typical large cascade, and
the dense scene pairs an example that stops with an example that grows. It must never
imply that the selected large event is typical. Avoid attributing the printed Figure 3
frequency to the filmed finite-size configuration. Beat 13 explicitly identifies our
1,000-node networks; its panel says "1,000 measured runs · mean degree 6.14".

New studio support will be a test-first `thresholds` frame dump with each agent's exact
threshold, acting state, forced-seed flag, actual neighbors and crowd, plus true step,
completed-episode count and config. Existing grid boards, body-state colors, caption
renderer and measured panels can be reused. Record asynchronous timing as actual model
steps; do not depict it as one perfectly synchronized wave per generation.

## Approved storyboard (about 100 seconds)

| # | Beat | Exact caption | Shot |
|---|---|---|---|
| 1 | crowd | "100 Flumps, each waiting for enough others to join." | Uniform thresholds, calm crowd, distribution ruler |
| 2 | instigator | "One needs nobody. Another needs one.\nThe next needs two." | Close-up of thresholds 0, 1, 2 |
| 3 | chain | "One starts. Each new arrival brings in the next.\nAll 100 join." | Uniform crowd's complete trajectory |
| 4 | change | "Raise just one threshold from 1 to 2." | Same arrangement, one threshold highlighted |
| 5 | stalled | "Now only the instigator joins.\nAlmost the same crowd. A very different outcome." | Perturbed crowd beside uniform final state |
| 6 | city | "Granovetter imagined random crowds from a uniform city.\nHe calculated that about half would stop at zero or one." | Our sampled-city examples and batch histogram, labeled as our simulations |
| 7 | friends | "Count friends twice, and the chain usually stops early." | Uniform crowd, random friendships, weight 2 |
| 8 | rescue | "In the stalled crowd, stronger friends\ncan bring a few more along." | Perturbed crowd, weight 5, example and frequency |
| 9 | ceilings | "Let some leave when the crowd gets too big,\nand participation can rise and fall." | Declared ceiling extension, 10 % leave above 90 % |
| 10 | network | "Watts put the rule on a network.\nEach Flump watches only its neighbors." | 1,000 nodes, threshold 18 %, one forced seed |
| 11 | sparse | "With few links, most sparks stay small." | Mean degree 1.05, participation and outcome histogram |
| 12 | middle | "With a middling number of links,\nmost sparks spread through nearly the whole network." | Mean degree 3, large cascade and batch counts |
| 13 | dense | "In our 1,000-node networks, many sparks die out.\nSome still sweep nearly the whole network." | Mean degree 6.14, stopped and large examples; panel identifies our 1,000 measured runs |
| 14 | end | "The riot that needs one person - After Granovetter, 1978; Watts, 2002\nndouglas.github.io/SugarScape" | Established closing card over one agent facing the camera, with one blink near the end |

## Prospective episode measurement rules

These rules are fixed before the episode measurement run. The audit informed their
selection; episode measurements will use a disjoint seed corpus, seeds 200001 upward.
Retain every outcome and config, stop criterion and choice of showcased example.
If a rule fails, narrow the caption and record that the revision came after the result.
Do not change the counting threshold to make a caption pass.

- Beats 1–5: at least 20 seeds, same 100-agent configs. Verify exact threshold lists,
  no initial participation with the instigator rule, thresholds unchanged, uniform
  final count 100 and perturbed final count 1. Under synchronous updating the uniform
  count grows by one per step until 100. Means differ by 0.01 percentage point.
- Beat 6: 5,000 independently drawn city crowds, at least 20 independent seed streams;
  zero-or-one share within 0.03 of the exact 0.5013584. Record full distribution,
  complete-riot share and the true number of independent draws.
- Beat 7: 1,000 graphs over at least 20 seed streams, uniform crowd, friendship
  probability 0.25 and weight 2; at least 80 % finish below 10 agents. This precise
  random-tie rule is ours; the paper does not give its sampling grid.
- Beat 8: 1,000 graphs over at least 20 seed streams, perturbed crowd, friendship
  probability 0.25 and weight 5; at least 20 % finish above one, and at least 90 %
  finish at ten or fewer. Choose an example with more than one from that corpus.
- Beat 9: 40 seeds, uniform crowd, synchronous updates, 10 % with ceilings at 90 %,
  `max_steps: 100000`, measured through step 600;
  at least 20/40 have participation range of at least 0.05 during steps 501–600.
  Label this a constructed extension, not a reproduction of Figure 3's drawn curve.
- Beat 10: 1,000 fresh 1,000-agent ER graphs, threshold 0.18, asynchronous updates;
  verify one forced seed and actual neighbor-only integer threshold comparisons.
- Beat 11: 1,000 fresh graphs at mean degree 1.05; at least 60 % finish below 1 %
  of the population. Display counts rather than promising no large cascade.
- Beat 12: 1,000 fresh graphs at mean degree 3; at least 75 % reach 90 % participation.
- Beat 13: 1,000 fresh graphs at mean degree 6.14; at least 55 % finish below 1 %,
  and at least 10 % reach 90 %. The caption is about these settings and finite size.
- Beat 14: supported by the fixed-threshold uniform/perturbed pair, without claiming
  that thresholds measure an empirical crowd's actual opinions or motives.

## Original tune proposal: *The Missing Rung*

A light 4/4 piece in A minor for flute, vibraphone, pizzicato strings and bassoon.
A four-note rising cell is passed between voices: a voice's last note cues the next
entrance. The listener hears the chain accumulating rather than a steadily louder
march. For the perturbed crowd, remove one connecting entrance; the first cell repeats
alone over the same harmony. The musical material is almost unchanged, but its
continuation is gone, matching the episode's central comparison.

Friends alter which instrument answers. The ceiling passage builds the texture and
removes voices again. Sparse-network sections leave gaps; the middle fills them;
the dense section alternates a short unanswered call with one complete chain. Finish
with the initial cell played together, then an open A–E fifth. Dynamics follow the
filmed examples, while measured frequencies remain on screen.

Proposed eight-bar subject; its development and tempo will be fitted to the approved cut:

```abc
X:1
T:The Missing Rung
C:Original episode proposal
M:4/4
L:1/8
Q:1/4=112
K:Am
V:flute clef=treble
V:vibes clef=treble
V:pizz clef=treble
V:bassoon clef=bass
[V:flute] E2 A2 B2 c2 | d2 c2 B2 A2 | G2 A2 B2 c2 | B4 E4 |
 E2 A2 c2 e2 | d2 c2 B2 A2 | G2 B2 A2 E2 | A8 |
[V:vibes] z8 | E2 A2 B2 c2 | d2 c2 B2 A2 | G2 A2 B2 c2 |
 B4 E4 | E2 A2 c2 e2 | d2 c2 B2 A2 | E4 A4 |
[V:pizz] z8 | z8 | E2 A2 B2 c2 | d2 c2 B2 A2 |
 G2 A2 B2 c2 | B4 E4 | E2 A2 c2 e2 | A4 E4 |
[V:bassoon] A,,2 E,2 A,2 E,2 | A,,2 E,2 A,2 E,2 | G,,2 D,2 G,2 D,2 | E,,2 B,,2 E,2 B,,2 |
 A,,2 E,2 A,2 E,2 | F,,2 C,2 F,2 C,2 | E,,2 B,,2 E,2 B,,2 | A,,4 E,4 |
```

## Verification and approval

The survey suite passes 54 tests, including independent-reference checks for the
normalized CDF, analytic range, rare-event probability and paired judge. Workspace
formatting and strict Clippy passed; workspace tests: 1,567 passed, 100 existing ignored;
WASM Node tests: 72 passed; web WASM build, TypeScript and Vitest: 847 passed; studio
unittests: 348 passed. The proposed eight-bar ABC subject parses to MIDI without warnings.

Independent review checked source attribution, sample counts and paired comparisons.
The final Opus review approved the audit and exact captions. Its remaining minor count
split was checked against the actual claim IDs: thirteen Granovetter and seven Watts
checks. The user approved the storyboard and tune; the episode is built and verified.

Storyboard and tune approved by the user on October 2. The closing card uses the established title, attribution and URL over one agent facing the camera, with a single blink near the end.

## Episode measurements and rendering

The approved rules hold in the separate episode corpus: 20 seeds for each exact crowd, 5,000 fresh city crowds, 1,000 fresh graphs for each friendship and network configuration, and 40 ceiling seeds. The city stops at zero or one in 2,471/5,000 draws; friends counted twice stop below ten in 999/1,000; stronger friendship brings in more than one in 459/1,000 and at most ten in 981/1,000. Late ceiling participation varies by at least five percentage points in 35/40 seeds. Sparse networks finish below 1% in 746/1,000; middle networks reach 90% in 859/1,000; dense networks finish below 1% in 626/1,000 and reach 90% in 220/1,000. These are our finite-network frequencies. No caption or measurement rule was revised after this result. Full configurations, outcomes, traces and selected seeds are retained in `studio/episodes/thresholds/measurements.json`.

All fourteen beats passed scratchpad still rendering at 50% size and sixteen samples. The selected ceiling sequence shows actual steps 500–600 with a labeled 85–95% vertical axis, and the network inset shows the real forced seed and all its recorded neighbors. The closing card follows the established title, attribution and URL above one camera-facing agent with one late blink.

Final software checks: workspace formatting and strict Clippy passed; 1,570 workspace tests passed, with 100 existing ignored; 72 WASM Node tests and 847 web tests passed; web WASM build and TypeScript passed; studio suite passed 365 tests. Independent task and integration reviews approved the implementation.

The original score uses beat-aligned cues for the chain, changed threshold, stalled crowd, ceilings, sparse, middle and dense network scenes. The backing fades to silence over the 0.8 seconds before each cue onset. It stays silent for the longest notated voice, including intentional rests, plus 0.3 seconds of note release; backing recovery and cue fade-out share the following 0.8 seconds. Complete cue WAVs retain their rendered reverb, while quiet file tails do not delay the backing return. The shared ABC/MIDI synthesis and audio mix pass, and the studio suite passes 393 tests.

The preview build completed successfully: 960 × 540, 2,844 video frames, 94.8 seconds, with matching audio duration. The final cut's chain, ceiling pulse, network examples and closing card were inspected; the single late blink closes and reopens. The current movie is `~/Movies/Flump Studio/sugarscape-025-thresholds.mp4`.

Individual threshold values are shown in the instigator and changed-threshold teaching close-ups, whose 30 mm framing retains the labels throughout the camera move. Overview shots use the distribution ruler and measured panels. The persistent Blender layout regression checks all 2,850 filmed frames and all fourteen separate captions for clipping, unrelated card occlusion and text intersections; both tests pass. The standard studio suite passes 365 tests.
