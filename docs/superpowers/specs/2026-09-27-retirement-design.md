# SugarScape Milestone 26 — The Timing of Retirement — Design

**Date:** 2026-09-27
**Builds on:** the milestone 1–25 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–25, and the preset titles of `crates/sugarscape-core/src/titles.rs`.
**Source texts** (local copies):
- Robert L. Axtell and Joshua M. Epstein, "Coordination in Transient Social Networks: An Agent-Based Computational Model of the Timing of Retirement", Brookings CSED Working Paper No. 1 (May 1999), `papers/retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` (AE below; published in H. Aaron, ed., *Behavioral Dimensions of Retirement Economics*, 1999).
- The revised text: Joshua M. Epstein, *Generative Social Science* (Princeton, 2006), chapter 7, "Coordination in Transient Social Networks", `papers/demographic-pd/epstein-2006-generative-social-science.pdf` (GSS below).

## Goal

Axtell and Epstein's retirement model as one model kind, `retirement` ("The Timing of Retirement"), a full citizen of the playground: their realizations (Figs. 6-4, 6-5), the sensitivity analysis (Figs. 6-6 to 6-9, the cohort size), the policy switch from 65 to 62 with a mandatory age of 70 (Fig. 6-10), and two loosely coupled sub-populations (Fig. 6-11); every detail the texts leave open — the denominator of f, what happens to a dead member of someone's network, the order of activation, the initial agents' death ages, the shape of a threshold spread, and above all what "transition time" measures — a named switch or a stated choice, and every claim measured.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only). Imitation compares retired·1 ≥ τ·counted without dividing by anything but a whole number of members.
- **Truthful descriptions and titles:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **The puzzle:** "In 1961, Congress reduced—from 65 to 62—the minimum age at which workers could claim social security benefits … Yet it took nearly three decades for the modal retirement age to fall from 65 to 62."
- **Agents and cohorts:** "The agent population is divided into age cohorts ranging from age 20 to 100 … 81 cohorts. Each contains C agents … Each agent is assigned a random death age drawn from U[60, 100] … When an agent dies it is replaced by a 20 year old agent. Each time period each agent is activated exactly once and, if it is eligible to retire but not yet retired, decides whether or not to retire." Footnote: "the order of agent activation is randomized within cohorts each period." The pseudo-code: "select an agent at random; increment its age; if age < death_age then do retirement_decision." Footnote 2: "In all cases below the random variables are assumed to be uniformly distributed."
- **Social networks:** "A social network is simply a list of other agents, specified randomly and fixed over the agent's lifetime. The number of other agents is set by drawing a random network size, S, from U[a, b] … E, represents how far, in the cohort dimension, the agent's social network extends above and below its own cohort; E is drawn from U[0, c]." The pseudo-code keeps "an array of pointers to … the agents who constitute the network". What becomes of a member who dies is not said.
- **Types:** "Rational agents retire at the earliest possible age allowed by government policy. Random agents retire with probability p each period once they reach the retirement eligibility age." Imitators: "Within this individual network, there is some fraction f of eligibles who have actually retired … If f ≥ τ, the agent retires." Footnote 5: "It makes a difference to the numerical results whether an agent considers all agents in its social network, or only those who are eligible to retire. However, the qualitative character of the results described below do not depend on this distinction." (GSS: "τ, representing the minimum proportion of members of the agent's social network who must be retired".)
- **Realizations:** the first with "15% of the agents are rational, 75% are imitators, and 5% are random" (AE; they sum to 95 %; GSS: 15/80/5; AE's Fig. 6-4 caption says 20 %): "Within the first 6 periods essentially all of the eligible population has retired … essentially monotone." The second, 5 % rational, 90 % imitators: "extensive fluctuation … It is as if retirement 'percolates up' from older to younger agents … It takes a long time for the absorbing state to be achieved … the trajectory is not monotone" (Fig. 6-5, 500 periods).
- **Base case (Table 6-1):** C 100, 10 % rational, 85 % imitators, 5 % random, τ 0.50, S U[10, 25], E U[0, 5], p 0.50.
- **Sensitivity (50 realizations each):** "C … was found to have no effect on the average transition time" (GSS: "for C > 100"); Fig. 6-6: "Reducing the proportion of rationals … increases transition time. When randoms comprise 0 percent or 5 percent of the population, certain minimum proportions of the population must be rational for a retirement age norm to arise. For a given fraction of rationals, the transition time decreases as the proportion of randoms increases … the variances increase rapidly with transition times" (times to 2 000, log scale); Fig. 6-7: "Increasing the variance in the threshold decreases the average transition time"; Fig. 6-8a: "the time required to transit to a uniform retirement age increases very rapidly with increasing social network size"; 6-8b: "as the variance increases the transition time decreases, although this is a relatively weak effect"; 6-8c: "the transition time increases very rapidly with S̄" (U[10, S̄]); Fig. 6-9: "the effect of increasing the extent … is to decrease the transition times" (10 % and 5 % rational).
- **"As if":** "The attainment per se of the age 65 retirement norm is compatible with any rationality fraction above a critical level."
- **Original policy (AE, homogeneous threshold .5):** "Now we require that all agents retire at age 70. This increases the speed at which the age 65 retirement norm is established … once the age 65 norm is established, we throw a 'policy switch' and lower the retirement age from 65 to 62 … Animation 6-3 … shows that a new norm indeed emerges after twenty to thirty periods"; Fig. 6-10: "a new norm is instituted in about 35 periods if between 1 and 4 percent of the population responds rationally."
- **Revised policy (GSS p.163):** rational and random agents each 5 %, thresholds U[.5,1], otherwise base networks; illustrative new norm after some twenty periods, and about 35 at 1–4 % rational. The original figure's approximate 1/2/3/4 % means are 70/40/30/20.
- **Two sub-populations:** "The 50 agents on the left do not include any rational agents, while those on the right include 10% rationals … 10% of each agent's network belongs to the other sub-population … Even this rather loose coupling is sufficient for the group containing some rationals to pull the other into conformity"; Fig. 6-11: "very little coupling is needed for the non-rational sub-population to be pulled into conformity with the more rational sub-population."
- **Transition time** — "the time required for … the age 65 retirement norm to emerge" — is never defined in either text.

## Measured reconstruction

The native audit uses 50 fresh seeds 1001–1050 per source sensitivity point (the survey uses 1–50), oldest-cohort-first activation with within-cohort shuffle, continuous death ages, and Slot renewal unless stated. First95 means the first period with 95 % of eligible agents retired. It is an operational proxy: neither the modal retirement age nor a persistent all-retired state. Timing means and sample SDs are conditional on attainment; nonattainment is right-censored at the stated horizon.

- **Realizations:** at 15 % rational the mean first 95 % crossing is 7.78 ± 1.25 periods; at the original caption's 20 %, 4.96 ± .64. The source describes one six-period realization, not a success probability. At 5 %, wavering followed by a cascade occurs, with first 95 % crossing 61.54 ± 3.92. This matches qualitative shape, not the displayed source plateau near 375 or perfect absorption.
- **Counting:** eligible-only Slot reaches first 95 % crossing in 50/50 over 600; all-members in 0/50. All-member imitators nevertheless retire (mean 91.48 events per run), and rolling mode 65 can arise from a small retiring minority. Under Replace, eligible-only reaches 37/50 and all-members 3/50 over 600. These selected-rule proxy differences do not categorically refute footnote 5's undefined qualitative norm.
- **Rationality:** Slot with 0 or 2 % rational reaches first 95 % crossing in 50/50 over 2000 (means 73.14 and 69.84). Finite-horizon first crossings cannot identify the source's critical norm or an infinite-time minimum. Slot/newborn pointers and Replace are explicit reconstruction choices; neither is proved by the pseudocode.
- **Thresholds:** at base 10 % rational, positive uniform half-widths .05,.10,.20,.30,.40,.50 (SD half-width/√3) give conditional first 95 % crossing means 43.18,36.36,17.34,10.08,7.96,10.20 over 600, all 50/50. The overall decline and final uptick match the source pattern. Zero spread (16.60) is an extension, not a plotted source point.
- **Extent and cohorts:** the source 5 %-rational branch spans 6–10, with endpoint means 57.80→52.78 (50/50 each over 600). Its 10 % branch spans 2–10. The revised C > 100 comparison uses 200 and 300, means 16.66 and 15.82,50/50 each over 600; the declared 20 % equivalence margin supports proxy equivalence, not every norm definition. Network-size surveys test qualitative trend only.
- **Policy:** original AE uses homogeneous threshold .5; revised GSS uses U[.5,1], represented by mean .75 and SD .14433756729740646, with random 5 %. At rational 5 %, mandatory 70 and the automatic first 95 % crossing switch, original post-switch first 95 % crossing is about 2 periods. Revised reaches 22/50 within 100 post-switch periods, conditional mean 43.73 ±32.90. An independent implementation with matched continuous death ages reaches 28/50, conditional 49.61 ±31.88. For revised automatic switching, native previous-tick event mode is 70 in 50/50 and independent actual switch-tick mode is 70 in 50/50. These observation windows differ and must not be compared directly; a first 95 % crossing initialization is not a demonstrated established age 65 norm. First modal crossings can reverse, and persistence is weak. Neither diagnostic establishes categorical source failure. Original figure means near 70/40/30/20 at 1/2/3/4 % are visual approximations, not a flat 20–40 acceptance band; revised text describes twenty and about 35 without defining termination.
- **Groups:** config rational .10 gives 10 % rational within B,0 % within A and expected 5 % globally. On source coupling .05–.20, B slows 24.72→57.90 and A converges 64.06→58.24 (50/50 each over 600). Both source and reconstruction show rational-group slowing; quantitative equivalence is not asserted.

## Architecture

Model kind `retirement` ("The Timing of Retirement"): `ModelKind::Retirement`, `ModelConfig::Retirement(RetirementConfig)` tagged `"model": "retirement"`, a `RetirementWorld` implementing `Model`, schema, `SERIES`, presets, titles and golden entries. Code in `crates/sugarscape-core/src/retirement/` (`config.rs`, `world.rs`, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `per_cohort` | 100 | reset | C (1–500): agents in each of the 81 initial cohorts |
| `rational`, `random` | 0.10, 0.05 | reset | the shares of rationals and randoms (imitators the rest) |
| `p` | 0.5 | live | a random agent's chance of retiring each eligible period |
| `threshold` | 0.5 | reset | τ, the imitators' mean threshold |
| `spread` | 0 | reset | the standard deviation of τ, spread uniformly (width √12 × spread) |
| `size` | {min 10, max 25} | reset | S ~ U[min, max], whole numbers |
| `extent` | 5 | reset | E ~ U[0, extent], whole numbers |
| `counts` | `eligible` | live | f over eligible members (the text), or `all` members (footnote 5) |
| `renewal` | `slot` | reset | a dead member's place in a network: `slot` (the newborn in its slot), or `replace` (the holder picks a new member within its own extent) |
| `order` | `by_cohort` | live | each period: cohorts oldest first, randomly within each (cohort direction is a reconstruction choice), or `shuffled` (global-order alternative) |
| `initial_deaths` | `literal` | reset | initial agents draw U[60, 100] (those past it die in period 1), or `survivors` (U[max(age, 60), 100]) |
| `eligibility` | 65 | live | the earliest age of retirement |
| `mandatory` | 0 | live | the age everyone retires (0: none) |
| `policy` | off | live | `policy.enabled`, `to` 62: once the aggregate proxy is reached, eligibility becomes `to` |
| `groups` | off | reset | `groups.enabled`, `coupling` 0.1: each cohort halved; rationals only in the second half; each network member drawn from the other half with probability `coupling` |
| `norm` | 0.95 | live | the share of eligible agents retired marking the operational first-crossing proxy |
| `stop_at_norm` | false | live | `finished()` once the aggregate proxy is reached (with `policy`, the post-switch proxy) |
| `stop_at` | 0 | live | `finished()` at this period (0: never) |

## Step (one period)

In activation order, each agent: ages a year; if its age reaches its death age it dies and a 20-year-old takes its slot (new type, threshold, death age and network; under `replace`, everyone whose network held the dead agent replaces it); otherwise, if working and at or past `mandatory` (when set), it retires; else if eligible it decides: rationals retire; randoms retire with probability p; imitators retire when retired ≥ τ × counted among their network members (counted: eligible members or all; none counted: they do not retire). After the period, if the norm is not yet recorded and the share of eligible agents retired reaches `norm`, the transition period is recorded; with `policy`, eligibility then becomes `to` and the second transition is timed from there.

## Statistics

`SERIES`: `retired` (the share of eligible agents retired), `retired_a` and `retired_b` (by group; equal to `retired` without groups), `transition` (the first aggregate proxy-crossing period; NaN before), `transition_new` (periods from the policy switch to the new aggregate proxy crossing; NaN before), `modal_age` and `mean_age` (retirement ages over the last 10 periods), `rational_share` (among the living).

## Views

- **Population** (left): one row per age, 20 at the top to 100 at the bottom, agents of that age left to right (up to the widest cohort); working agents by type (rational pink, imitator blue, random yellow), retired red, empty places white.
- **Retirement by age** (middle): the share retiring at each age over the last 10 periods, eligibility and mandatory ages marked.
- **Retired share over time** (right): the last 300 periods of `retired`, the norm level and the policy switch marked.
- **Color modes:** **Status**, **Type**, **Threshold**, **Group**, **Network** (a selected agent's network marked, as in AE's Fig. 6-2).
- **Inspect:** an agent's age, type, threshold, death age, network (members, eligible, retired) and status; a bar of the middle panel; a period of the right panel.
- **Charts:** Retired share (`retired`, `retired_a`, `retired_b`); Retirement age (`modal_age`, `mean_age`); Transition (`transition`, `transition_new`). Time axis: Periods.

## Presets

Titles follow `titles.rs`'s style; drafts.

| Preset | Title | Setup |
|---|---|---|
| `ae-rapid` | 15 % decide rationally, and retirement spreads quickly | 15/80/5 (Fig. 6-4, GSS's shares) |
| `ae-base` | A tenth decide rationally: aggregate retirement grows | Table 6-1 (the kind's default) |
| `ae-slow` | 5 % rational: retirement wavers before spreading | 5/90/5 (Fig. 6-5) |
| `ae-policy` | Eligibility 65→62 after the aggregate proxy crossing | 5 % rational, mandatory 70, policy |
| `ae-groups` | Two communities, one with no rational agents, loosely linked | groups, coupling 0.1 |
| `ae-all-members` | Count every network member: a denominator sensitivity | `counts: all` |
| `ae-replace` | Replace friends who die: a renewal sensitivity | `renewal: replace`, 5 % rational |

**Compare entry:** "15 % vs 5 % rational — Retirement (Compare)": `ae-rapid` and `ae-slow`.

## Experiments and CLI

Seeds and horizons measured to fit a browser run and recorded in each description; the metric is the final `transition` with runs stopped at the norm (a run that never reaches it reads NaN).
- `ae-rational`: transition against the rational share (0–25 %), series random 0, 5, 10 % (Fig. 6-6), `slot`.
- `ae-rational-replace`: the same under `replace`.
- `ae-threshold`: against the threshold spread (Fig. 6-7).
- `ae-size`: against the mean network size (Fig. 6-8a).
- `ae-extent`: against extent, series 10 %, 5 % rational (Fig. 6-9).
- `ae-policy`: `transition_new` against the rational share (Fig. 6-10).
- `ae-coupling`: the group transitions against coupling (Fig. 6-11), series each group.
The CLI names the stop `(its last period)`.

## Survey

A `retirement` claims module retains 15 IDs. Source-domain grids and 50-run ensembles judge qualitative operational trends. Footnote 5, criticality, the “as if” inference, mandatory retirement and policy remain Weak diagnostics because first 95 % crossing does not identify the authors' unspecified age norm. Each changed claim states that its operational rule was revised after the earlier result was known. Rapid timing checks compatibility with one published realization; slow timing does not assert quantitative reproduction. Censoring and conditional means remain visible.

## Page

The presets menu gains a **The Timing of Retirement** group and the Compare entry; the Rules panel is generated from the schema in groups Population, Agents, Networks, Policy, Groups and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `retirement` preset; titles for every preset.
- **Core unit:** cohorts and death ages (both readings); newborns in the slot; network sizes, extents and distinct members; `slot` and `replace` renewal; each type's rule; f over eligible or all members (exact, no members counted); `by_cohort` and `shuffled` order; mandatory retirement; the norm and transition; the policy switch; groups and coupling; statistics on hand populations; the view and Inspect; keyframes; live and reset fields; degenerate configs (C 1, all rational, all random, no imitators, size 0–0, extent 0).
- **Web:** schema groups and visibility, charts, the Compare entry, a sweep over a `retirement` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, Compare, recording, Experiments.

## Docs

README and `docs/papers.md` describe the source-specific parameterizations, explicit reconstruction choices, operational trends and limits summarized above.

## Implementation details

Each agent ages on its own activation; agents awaiting activation remain a year younger. Cohorts are birth periods, so network geometry uses birth-period distances. Initial networks draw peers from available nearby birth cohorts; newborn networks cannot include future younger cohorts. Under Slot renewal, later newborns inherit dead friends' places, so age proximity need not persist. Oldest-first traversal and sequential newborn sampling are explicit choices; the source specifies within-cohort randomization but leaves traversal direction and dead pointers unresolved.

Group transitions are first aggregate crossings and retain their values after later declines. `modal_age` and `mean_age` describe the last 10 periods of retirement events and are NaN/null when the window has no retirements. An event mode is distinct from retirement hazard by age and from the fraction of current eligible agents retired. Color modes are Status, Type, Threshold and Group; Inspect lists network members and their eligibility/retirement counts. Sweeps report retained first-crossing values and censored runs; policy presets and built-in sweeps use the original homogeneous threshold setup. Revised policy requires the explicit threshold/spread settings above.
