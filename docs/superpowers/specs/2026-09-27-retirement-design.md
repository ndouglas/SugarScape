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
- **Policy:** "Now we require that all agents retire at age 70. This increases the speed at which the age 65 retirement norm is established … once the age 65 norm is established, we throw a 'policy switch' and lower the retirement age from 65 to 62 … Animation 6-3 … shows that a new norm indeed emerges after twenty to thirty periods"; Fig. 6-10: "a new norm is instituted in about 35 periods if between 1 and 4 percent of the population responds rationally."
- **Two sub-populations:** "The 50 agents on the left do not include any rational agents, while those on the right include 10% rationals … 10% of each agent's network belongs to the other sub-population … Even this rather loose coupling is sufficient for the group containing some rationals to pull the other into conformity"; Fig. 6-11: "very little coupling is needed for the non-rational sub-population to be pulled into conformity with the more rational sub-population."
- **Transition time** — "the time required for … the age 65 retirement norm to emerge" — is never defined in either text.

## Measured in planning

A throwaway prototype (Rust) of the rules below; 20 runs each unless stated; "transition" is the first period with 95 % of eligible agents retired; shuffled activation unless stated; the survey reproduces each with the implementation.

- **Realizations:** 15 % rational: 95 % retired at period 8 (19 %, 27 %, 37 %, 47 %, 65 %, 80 % over periods 1–6), monotone; 20 % rational: 6 periods (the AE caption's 20 % fits "the first 6 periods"); 5 % rational: 67 periods, not monotone (23 % at 20, 19 % at 50).
- **Footnote 5 fails:** counting all network members instead of eligible ones, no norm forms at all (0 of 20 runs in 2 000 periods, base case): an imitator's network spans five years either side, and its younger members hold f below ½.
- **Fig. 6-6 depends on an unstated rule.** When a dead member's place passes to the 20-year-old reborn in its slot (`slot`), there is no minimum of rationals and no long transitions: with 5 % randoms, 83, 79, 69, 23, 10, 6, 4 periods at 0, 2, 5, 10, 15, 20, 25 % rational; with no randoms, never at 0 but 92 at 2 %. When the holder replaces a dead member within its own age range (`replace`), the paper's shape appears: with 5 % randoms, 0 and 5 % rational never converge in 1 500 periods; 10 % takes 139 ± 195 periods; 15 %, 15; 20 %, 8 (C 50, 10 runs).
- **Fig. 6-7:** transition 23, 55, 28, 13, 9, 9 at threshold spreads 0, 0.05, 0.1, 0.15, 0.2, 0.25 — decreasing overall, but a little spread first slows it.
- **Fig. 6-8:** mean size 10, 15, 20, 25, 30, 40 (± 7): 8, 14, 45, 67, 75, 80 — rising steeply, then leveling; size 17 ± 0, 3, 7, 10, 14: 27, 25, 19, 17, 14 — weakly falling; U[10, S̄] for S̄ 10–80: 9, 15, 37, 58, 73, 78 — rising. As stated.
- **Fig. 6-9:** at 10 % rational, 21, 22, 22, 23, 20, 18 for extents 1, 2, 3, 5, 7, 10 (weak); at 5 %, 65 ± 25, 81, 76, 69, 64, 59 — falling from extent 2.
- **C:** 21, 21, 23, 24 at C 25, 50, 100, 200 — no effect, as stated.
- **Order:** activating cohorts oldest first, randomly within each (the footnote's reading), is faster than one shuffled order: 58, 15, 8 against 69, 23, 10 at 5, 10, 15 % rational.
- **Initial death ages:** no effect (23 against 25 when initial agents draw only ages still ahead of them).
- **Mandatory retirement at 70:** 65 norm in 6 periods against 69 (5 % rational) — faster, as stated.
- **The policy switch fails:** with retirement mandatory at 70 and eligibility cut from 65 to 62 when the 65 norm is reached, 95 % of those 62 and older are retired within 2–7 periods at every rational share (0–14 %), 2–11 under `replace` — not the 20–35 the paper reports. A 62-year-old's eligible network already includes retired 65-to-67-year-olds; imitators tip at once.
- **Two sub-populations:** the group without rationals is pulled in (82 periods uncoupled, 70 at coupling 0.05, 56 at 0.1), as stated; but the coupling drags the rational group just as hard (17, 24, 41 at 0, 0.05, 0.1; about 65 for both from 0.15) — the paper's figure keeps it fast.

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
| `order` | `by_cohort` | live | each period: cohorts oldest first, randomly within each (the footnote), or `shuffled` (one random order: the pseudo-code) |
| `initial_deaths` | `literal` | reset | initial agents draw U[60, 100] (those past it die in period 1), or `survivors` (U[max(age, 60), 100]) |
| `eligibility` | 65 | live | the earliest age of retirement |
| `mandatory` | 0 | live | the age everyone retires (0: none) |
| `policy` | off | live | `policy.enabled`, `to` 62: once the norm is reached, eligibility becomes `to` |
| `groups` | off | reset | `groups.enabled`, `coupling` 0.1: each cohort halved; rationals only in the second half; each network member drawn from the other half with probability `coupling` |
| `norm` | 0.95 | live | the share of eligible agents retired that marks the norm (transition time) |
| `stop_at_norm` | false | live | `finished()` once the norm is reached (with `policy`, the new norm) |
| `stop_at` | 0 | live | `finished()` at this period (0: never) |

## Step (one period)

In activation order, each agent: ages a year; if its age reaches its death age it dies and a 20-year-old takes its slot (new type, threshold, death age and network; under `replace`, everyone whose network held the dead agent replaces it); otherwise, if working and at or past `mandatory` (when set), it retires; else if eligible it decides: rationals retire; randoms retire with probability p; imitators retire when retired ≥ τ × counted among their network members (counted: eligible members or all; none counted: they do not retire). After the period, if the norm is not yet recorded and the share of eligible agents retired reaches `norm`, the transition period is recorded; with `policy`, eligibility then becomes `to` and the second transition is timed from there.

## Statistics

`SERIES`: `retired` (the share of eligible agents retired), `retired_a` and `retired_b` (by group; equal to `retired` without groups), `transition` (the period the norm was reached; NaN before), `transition_new` (periods from the policy switch to the new norm; NaN before), `modal_age` and `mean_age` (retirement ages over the last 10 periods), `rational_share` (among the living).

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
| `ae-rapid` | 15 % decide rationally, and retiring at 65 sets in within a few years | 15/80/5 (Fig. 6-4, GSS's shares) |
| `ae-base` | A tenth decide rationally, and the norm takes a generation | Table 6-1 (the kind's default) |
| `ae-slow` | 5 % rational: retiring at 65 spreads slowly, up from the old | 5/90/5 (Fig. 6-5) |
| `ae-policy` | Congress lowers the age to 62: here the new norm comes in a few years | 5 % rational, mandatory 70, policy |
| `ae-groups` | Two communities, one with no rational agents, loosely linked | groups, coupling 0.1 |
| `ae-all-members` | Count every friend, not just the eligible, and no norm ever forms | `counts: all` |
| `ae-replace` | Replace friends who die, and a minimum of rationality appears | `renewal: replace`, 5 % rational |

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

A `retirement` claims module: the realizations (15 % within six periods and monotone; 5 % slow and not monotone); footnote 5; Fig. 6-6 under both renewals (the minimum of rationals, randoms speeding it, variance growing with the mean); C; Figs. 6-7, 6-8a–c, 6-9; "as if"; mandatory 70; the policy switch (20–35 periods; 35 at 1–4 %); the sub-populations (the non-rational group pulled in; the rational group's time). Claims that fail are reported, and the descriptions, titles and README say so.

## Page

The presets menu gains a **The Timing of Retirement** group and the Compare entry; the Rules panel is generated from the schema in groups Population, Agents, Networks, Policy, Groups and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `retirement` preset; titles for every preset.
- **Core unit:** cohorts and death ages (both readings); newborns in the slot; network sizes, extents and distinct members; `slot` and `replace` renewal; each type's rule; f over eligible or all members (exact, no members counted); `by_cohort` and `shuffled` order; mandatory retirement; the norm and transition; the policy switch; groups and coupling; statistics on hand populations; the view and Inspect; keyframes; live and reset fields; degenerate configs (C 1, all rational, all random, no imitators, size 0–0, extent 0).
- **Web:** schema groups and visibility, charts, the Compare entry, a sweep over a `retirement` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, Compare, recording, Experiments.

## Docs

README: a Timing of Retirement section (the model, the stated choices, switches, presets, sweeps, and the findings: the realizations reproduce; footnote 5 fails — counting all members, no norm forms; Fig. 6-6's minimum of rationals and long times need the unstated `replace` rule; the network-size and extent effects hold; the policy switch's decades-long response does not follow — the new norm comes in a few periods; the rational group is slowed by the coupling that pulls the other in). `docs/papers.md`: the milestone's row (with GSS ch. 7); the Queue's first entry removed; roadmap: Milestone 26 done.

## Amendments (implementation planning)

The model was implemented in full while planning (`docs/superpowers/plans/2026-09-28-retirement.md`) and measured with it; these change or extend the sections above.

- **Each agent ages when it is activated** (the pseudo-code: "select an agent … increment its age"); those not yet activated in a period are a year younger. A first implementation that aged everyone at the start of the period took twice as long (31 periods for the base case, 55 shuffled): an imitator's peers who had just turned 65 were already eligible, and still working, when it decided. Cohorts are birth periods, so a network's extent compares birth periods, which never change.
- **`mandatory` may be any age up to 100** (0: none); below 20 everyone retires at once.
- **Statistics** add `transition_a` and `transition_b`, each group's first period at the norm (both `transition` without groups), for Fig. 6-11's two sweeps (`ae-coupling`, `ae-coupling-rational`) — a sweep reads one series. `modal_age` and `mean_age` are NaN (null) with no retirements in the window; `eligibility` is the age now (it drops at the switch).
- **Color modes** are Status, Type, Threshold and Group; the Network mode (a selected agent's network marked) is dropped — the frame has no selection. Inspect lists an agent's network with how many members are eligible and retired.
- **Sweeps:** `ae-rational`, `ae-rational-replace`, `ae-threshold`, `ae-size` (S ~ U[10, max], Fig. 6-8c), `ae-extent`, `ae-policy`, `ae-coupling`, `ae-coupling-rational`; the metric is the final `transition` (or `transition_new`, `transition_a`, `transition_b`), which keeps its value once reached.
- **With retirement mandatory at 70**, those forced out are most of the eligible, so the 95 % measure reaches the 65 "norm" in about 3 periods whatever the rationality; the claim that a mandatory age speeds the norm holds but says little about retiring at 65.
- **Measured with the implementation** (the survey, 15 claims; 8 hold, 7 fail): 15 % rational reaches 95 % by period 6 in 4 of 20 runs (mean 7.7), monotone in all; 5 % takes 61 periods, not monotone in 20 of 20; counting all members, no norm in 20 of 20; transition 69.5, 61.1, 16.4, 7.7, 5.2, 3.9 at 2–25 % rational (5 % random); no minimum of rationals (0 %: 72; 2 %: 69) under `slot`, none of 0 and 5 % reaching it under `replace`; randoms speed it (72, 61, 18 at 0, 5, 10 % random, 5 % rational); C 100 and 200 equivalent (17 and 17; C 25: 18); threshold spread 16.4, 36.6, 19.7, 11.8, 8.3, 7.9 (not monotone); network size 7 → 73 (mean), 18 → 12 (spread), 7 → 70 (maximum); extent 1 against 10: 18 against 13.5 at 10 %, 53.5 against 54.5 at 5 % (no effect); every run from 2 % rational reaches the norm; mandatory 70: 3 against 61 periods; the policy switch 2.0 periods at 1, 2, 4 % rational; the group without rationals 75 → 49 at coupling 0.1, the group with rationals 17.5 → 32.5 (60 at 0.25).
