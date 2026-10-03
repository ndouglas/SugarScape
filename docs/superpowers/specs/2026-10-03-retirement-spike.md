# Spike: retirement

**Following the Crowd, episode 10: “When to retire”**

**Date:** 2026-10-03  
**Status:** source audit complete; exact storyboard and original tune approved October 3. Episode build authorized.  
**Sources:** Axtell & Epstein (1999), *Coordination in Transient Social Networks: An Agent-Based Computational Model of the Timing of Retirement*; Epstein (2006), *Generative Social Science*, chapter 7. Milestone 26 is `retirement`.

## What the audit establishes

A small minority retiring at the first eligible age can accompany a much wider retirement cascade among agents who watch their own networks. This is a model of social coordination, not advice about when people should retire. The sources describe an age norm but do not publish its numerical stopping rule. Our first period with 95% of eligible agents retired is an **eligible-retired proxy**, distinct from the most frequent retirement-event age and from a persistent age norm.

The [retained audit](2026-10-03-retirement-audit/README.md), [native results](2026-10-03-retirement-audit/native/results.md), [independent results](2026-10-03-retirement-audit/independent/report.md), and [corrected design](2026-09-27-retirement-design.md) supply the scientific scope. Native source comparisons use 50 seeds per point. The rapid setup uses the revised 15/80/5 shares; the original figure caption instead says 20% rational. First proxy crossing averages 7.78 periods at 15% rational and 4.96 at 20%. At 5% rational, a wavering cascade reaches it at 61.54 ± 3.92 periods, 50/50 attained. That is qualitative agreement, not reproduction of the source slow figure's approximately 375-period timing or perfect absorption.

Eligible-only versus all-member counting changes the cascade under the retained Slot reconstruction. Over 600 periods the base eligible-only setup attains the proxy in 50/50 runs; all-member counting in 0/50. Nevertheless, all-member imitators do retire, averaging 91.48 events per run. A rolling event mode of 65 can describe a small retiring minority. These observations establish sensitivity to the denominator and the observable, without adjudicating the source's unspecified qualitative norm. Slot renewal and oldest-cohort-first traversal are explicit reconstruction choices, not uniquely prescribed by the sources.

Two communities contain 0% and 10% rational agents respectively, an expected 5% globally. At coupling .05, group A/B proxy times average 64.06/24.72; at .20, 58.24/57.90, all 50/50 attained over 600. Contact can draw the slower group toward the faster group and also slow the faster group. The source shows both directions too; exact quantitative equivalence is not established.

Policy has two source-specific threshold setups: original homogeneous .5 and revised U[.5,1] (mean .75, SD .25/√3). For a coherent film comparison, both proposed policy recordings use **fixed 100-period warm-up**, mandatory retirement at 70, 5% rational and 5% random, then eligibility 65→62 with 100 post-switch periods. Fixed warm-up is our declared protocol; it does not guarantee the authors' established age-65 norm. Original post-switch proxy attainment is 50/50 with conditional mean 1 period; revised is 19/50, 31 right-censored, with conditional mean 41.05 ± 26.54. Independent revised fixed100 reaches 21/50, conditional mean 54.05 ± 28.15. This supports a qualified contrast between these reconstructions, not a source failure or an unconditional mean response time.

The separate automatic-switch diagnostic changes policy at first 95% crossing. That initialization does not demonstrate an established age-65 norm. Native `previous_tick_mode_at_switch` excludes switching-tick events; independent `mode_at_switch` includes them. Their windows differ and cannot be compared directly. No native switch-tick ensemble mode is inferred here.

## Survey verdicts and their scope

`cd survey && cargo run --release -q -- --only retirement` retains all 15 IDs: **10 Holds, five Weak**. These are operational tests of stated reconstructions. Changed rules disclose that they were revised after earlier outcomes were known. The current survey corpus and the fresh native audit are different seed sets; their numerical summaries are not interchangeable.

| Survey ID | Verdict | What it supports |
|---|---|---|
| `retirement.ae.rapid` | Holds | Four of 50 six-period trajectories compatible with a published realization; no population success probability or exact figure replication |
| `retirement.ae.slow` | Holds | Wavering then cascade, conditional mean 60.9, 50/50 attained; not the source's approximately 375-period timing |
| `retirement.ae.footnote5` | Weak | Eligible/all denominator sensitivity under Slot; all-member imitators can retire; no categorical judgment of the source norm |
| `retirement.ae.fig6-rationals` | Holds | Faster proxy crossings as rational share increases across the tested grid |
| `retirement.ae.fig6-minimum` | Weak | Crossings at 0% and 2% rational under chosen renewal/order; no infinite-time critical threshold |
| `retirement.ae.fig6-randoms` | Holds | Faster crossings with more random agents at the tested shares |
| `retirement.ae.cohort` | Holds | Proxy equivalence at C200/C300 under declared 20% margin; revised claim concerns C>100 |
| `retirement.ae.fig7` | Holds | Overall decline across positive source-domain threshold spreads, with final uptick; zero is a separate extension |
| `retirement.ae.fig8` | Holds | Qualitative network-size, size-spread and maximum-size trends; no exact timing scale reproduction |
| `retirement.ae.fig9` | Holds | Wider extent speeds proxy crossing on the tested source ranges, including 5%-rational extent6–10 |
| `retirement.ae.as-if` | Weak | Aggregate retirement at several rational shares; norm and criticality remain unspecified |
| `retirement.ae.mandatory` | Weak | Mandatory70 speeds the aggregate proxy; does not establish a persistent age65 norm |
| `retirement.ae.policy` | Weak | Original/revised automatic proxy contrast with attained/censored counts; no demonstrated established age65 initialization |
| `retirement.ae.groups-pull` | Holds | A reaches the proxy earlier at coupling .10 than .05 |
| `retirement.ae.groups-rational` | Holds | B slows as coupling rises .05→.20 and approaches A, as the source also shows |

## How to film it

Keep **C100 and all 8,100 real agents** in the main recordings. An overview may use level of detail, with an inset magnifying a selected actual agent and its actual network. Never shrink C for convenience and present its outcome as the source-population result. All marks and movements correspond to recorded agents. Reserve separate regions for captions, population, and measurements; no floating overlay covers captions.

Initially 81 age cohorts contain 100 agents apiece. Mortality replaces an agent with a newborn aged 20, preserving total population but not uniform age/cohort counts: renewed cohorts can contain thousands. Show actual counts and distinguish slot identity from birth identity. Network drawings follow real directed links, actual eligibility and retired status at the selected decision. Slot renewal can introduce younger agents into an older holder's network; do not promise enduring age proximity.

The future CLI retirement dump needs tests first, then all-agent age, kind, retired status, birth/cohort identity, group, network members, current eligibility and **actual before-decision neighbor state**, followed by the recorded outcome. Test mortality/newborn identity, activation-local age, a threshold-equality decision, no counted neighbors, and a policy switch. This is a future build requirement, not implemented in the spike.

Keep method detail in reserved labels: rational/random/imitator shares, eligible denominator, thresholds, periods, selected seed, ensemble size, and fixed100 policy warm-up. Explain imitation by slowing one actual decision: counted eligible neighbors, retired numerator, positive denominator, threshold comparison. The age panel separately plots retirement-event mode and exposure-based retirement rates; the aggregate panel labels “eligible retired / eligible” and its 95% proxy line. Conditional timing panels always display attained/total, right-censored count and horizon. The group comparison uses .05 and .20 coupling, base thresholds, and identical scales. Its edges are actual Bernoulli-drawn contacts, not an exact percentage in every individual's network.

## Proposed storyboard (14 beats, 98 seconds)

Captions below are exact proposed text; `\n` means a line break. Every event is an actual recording. On-screen method labels belong in the reserved measurement region.

| # | Seconds | Beat | Caption | Filming |
|---|---:|---|---|---|
| 1 | 7 | ages | “8,100 Flumps start in age groups,\nfrom 20 to 100.” | Initial age overview, actual cohort counts |
| 2 | 6 | renewal | “Each year they age. When one dies,\na 20-year-old takes its place.” | Actual death/newborn, birth identity label |
| 3 | 8 | habits | “Some retire as soon as they can.\nSome decide by chance. Most watch their friends.” | Three actual kinds; baseline eligibility65 |
| 4 | 8 | decision | “An eligible Flump who imitates retires\nwhen enough eligible friends have retired.” | Actual imitator's before-decision neighborhood and threshold |
| 5 | 7 | quick | “With more early retirees,\nretirement spreads quickly.” | 15% rational; trace and ensemble proxy times |
| 6 | 7 | slow | “With fewer, it wavers\nbefore spreading through the crowd.” | 5% rational; same scales, qualitative source scope |
| 7 | 7 | denominator | “Count younger friends too,\nand the cascade changes.” | Eligible/all paired setup; actual networks and censor counts |
| 8 | 7 | observable | “The most common retirement age\ncan describe only a small retiring minority.” | All-member events, exposure and retired-share panels |
| 9 | 7 | groups | “Two communities meet. Only one includes\nFlumps who always retire as soon as they can.” | A0%/B10% rational; both retain random agents |
| 10 | 7 | contact | “Contact helps one catch up,\nwhile slowing the other.” | .05/.20 actual-link comparison; group proxy distributions |
| 11 | 6 | policy | “After 100 years, eligibility\nmoves from 65 to 62.” | Fixed100 explicit; mandatory70; original/revised side by side |
| 12 | 8 | thresholds | “Raise the imitation thresholds,\nand many runs miss our target within 100 years.” | Original .5/revised U[.5,1]; 100-post-period horizon, 50/50 versus19/50 |
| 13 | 7 | meaning | “How many retire, and at what age,\nare different questions.” | Aggregate proxy, mode and age-specific exposure remain distinct |
| 14 | 6 | end | “When to retire - After Axtell & Epstein, 1999; Epstein, 2006\nndouglas.github.io/SugarScape” | One camera-facing actual agent; one late blink |

## Measurement protocol for the proposed build

Freeze configurations, fresh seed lists, horizons, exact caption tests and deterministic example-selection rules before production measurement. At least20 seeds per production claim; use50 wherever comparing the papers' 50-realization sensitivity figures. Preserve every failed or censored outcome. Retain per-seed timing, retirement-event ages, age-specific working exposure, retired/eligible share, and attainment counts; timing means conditional on attainment are labeled as such. Source figure pixels and retained audit evidence remain distinct from newly collected production measurements.

For the quick/slow comparison use identical C100, random5%, thresholds.5, Slot renewal and cohort traversal, varying rational15% versus5%. Measure proxy timing ordering and pre-cascade declines; do not test a literal 375-period reproduction with this reconstruction. Denominator comparison uses rational10% and 600 periods; mode, event counts and aggregate share must support the minority explanation. Groups use the source .05/.20 points and 600 periods, recording both group times. Policy uses the two fixed100 setups described above and exactly100 periods after switching; compare attainment and time distributions with censoring visible. Never treat an early first crossing as stable age62 retirement.

Select explanatory decisions by a fixed actual-event rule. Select comparison runs nearest the declared ensemble median observable, tie-breaking by smallest seed; for censored policy outcomes select the smallest censored seed and label it. Establish this rule before collecting data. If a caption fails, revise the caption and disclose the changed rule rather than selecting a more favorable run. A numeric method label must say first95 is the eligible-retired proxy the first time it appears.

## Proposed original tune: *The Later Step*

An original **Swedish polska-inspired dance in D major and 3/4**, with fiddle lead (MIDI40), quiet accordion (21) and plucked acoustic guitar (24). Repeated turns suggest habits passed through a neighborhood; a relaxed pulse fits changing routines without mocking age. This is an intentional folk-inspired composition, not an ethnomusicological reconstruction or quotation of a traditional tune.

One continuous score at one fitted tempo, with one lead and quiet accompaniment on the same grid. The eight-bar approval sample uses108 quarter notes per minute; the proposed full score allows100–110, preferring108. No independently timed cue recordings or backing ducks. The proposed56-bar form `A A B A C B A` fits the98-second cut through the existing studio fitter: its1.5-second ring-out gives104.456 quarter notes per minute, emitted as104 in ABC. The final cadence replaces the last bar; there is no extra scored tail. The rounded score lasts about96.923 seconds before its release, which must be checked when audio is rendered. B varies the subject's middle turn; C thins it to longer notes while accompaniment keeps the pulse. Any later variations must preserve full3/4 bars and be compiled and inspected before audio render. The eight-bar subject is the approval sample, not a completed episode arrangement.

```abc
X:1
T:The Later Step
C:Original episode proposal
M:3/4
L:1/8
Q:1/4=108
K:D
V:lead name="fiddle"
%%MIDI program 40
%%MIDI control 7 88
V:chords name="accordion"
%%MIDI program 21
%%MIDI control 7 45
V:bass name="guitar"
%%MIDI program 24
%%MIDI control 7 58
[V:lead] D2 FA dA | B2 AF ED | F2 Ad fd | e2 cA G2 |
 F A d A F D | G B e B G E | F2 E2 C2 | D4 z2 |
[V:chords] [DFA]2 z4 | [GBd]2 z4 | [DFA]2 z4 | [Ace]2 z4 |
 [DFA]2 z4 | [GBd]2 z4 | [Ace]2 z4 | [DFA]4 z2 |
[V:bass] D,2 A,2 D2 | G,2 D2 G,2 | D,2 A,2 D2 | A,,2 E,2 A,2 |
 D,2 A,2 D2 | G,2 D2 G,2 | A,,2 E,2 A,2 | D,4 z2 |
```

All24 voice-bars contain six eighth-note units, exactly3/4. `abc2midi`5.03 compiled the subject with exit0 and no warnings. ABC and MIDI scratch files remain in the ignored `.superpowers/sdd/2026-10-03-retirement-audit/spike/` directory. A scratch full-form score verifies compatibility with the existing fitter:56 bars per voice, all168 voice-bars valid, emitted tempo104, compiled without warnings. This is a proposal check; the episode arrangement and audio render await approval.

## Verification and approval

Fresh repository checks pass: workspace formatting, strict Clippy,1,587 native tests (100 existing ignored),72 WASM Node tests, WASM build, TypeScript,847 web tests across53 files,412 studio tests and62 survey unit tests. These validate the current repository and corrections; they do not verify an unbuilt episode. The scientific/spec evidence gate is clean within its declared limits.

The user approved the exact storyboard, original tune, retirement correction commit and episode build on October 3. Production measurements, full arrangement, rendering and composited-frame review follow the approved protocol.
