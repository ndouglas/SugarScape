# SugarScape Milestone 11 — Civil violence — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–10 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds (`2026-09-25-other-artificial-societies-design.md`).
**Source text:** Joshua M. Epstein, "Modeling civil violence: An agent-based computational approach", *PNAS* 99 suppl. 3 (2002), 7243–7250 (Model I: generalized rebellion against central authority; Model II: inter-group violence; Table 2's inputs). **Reference implementation for the quirks:** Wilensky's NetLogo *Rebellion* (2004), `Sample Models/Social Science/Rebellion.nlogox` in the NetLogo models library.

## Goal

Add Epstein's civil violence model — both Model I (rebellion) and Model II (inter-group violence) — as a fifth model kind, a full citizen of the playground (worker engine, Max speed, replay and share links, Compare, recording, Experiments, the CLI and the survey), with the paper's runs as presets, NetLogo's departures as named switches, and the paper's claims checked over 20 seeds.

## Non-negotiable constraints

- **Earlier models unchanged.** Every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **Faithful to the paper** where it is specific (quoted below); where it is silent, the choice is stated here and in the module docs.
- **One engine path.** Host, engine, replay, Compare, sweeps and CLI stay single-path over the `Model` trait.
- **Deterministic.** A function of (config, seed); each preset's fingerprint pinned by a golden entry.
- **Truthful descriptions.** Each preset's description says what it measurably reproduces and what it does not (the milestone survey's standard).

## Source summary

- **Agents:** "H is the agent's perceived hardship … drawn from U(0,1)"; "L, the perceived legitimacy … exogenous and equal across agents"; "G = H(1 − L)"; "R as the agent's level of risk aversion … drawn from U(0,1) and is fixed"; "P = 1 − exp[−k(C/A)v] … k is set to ensure … P = 0.9 when C = 1 and A = 1 … A is always at least 1, because the agent always counts himself as active"; "N = RP"; "Agent rule A: If G − N > T be active; otherwise, be quiet."
- **Cops:** "Cop rule C: Inspect all sites within v* and arrest a random active agent."
- **Movement and jail:** "Movement rule M: Move to a random site within your vision"; "any arrested active is assigned a jail term drawn randomly from U(0, J_max)"; "agents leave jail exactly as aggrieved as when they entered".
- **Schedule:** "An agent or cop is selected at random (asynchronous activation) and, under rule M, moves to a random site within his vision, where he acts in accord with rule C (if a cop) or A (if an agent)"; "agent activation (once per period, random order)".
- **Table 2** (all models: 40 × 40 torus, cloning probability 0.05, k = 2.3, max age 200, T = 0.1, initial density 0.7):

  | | Run 1 | Run 2 | Runs 3 & 4 | Run 5 | Run 6 | Run 7 | Run 8 |
  |---|---|---|---|---|---|---|---|
  | Model | I | I | I | I | II | II | II |
  | Cop vision | 1.7 | 7 | 7 | 7 | 1.7 | 1.7 | 1.7 |
  | Agent vision | 1.7 | 7 | 7 | 7 | 1.7 | 1.7 | 1.7 |
  | Legitimacy | 0.89 | 0.82 | 0.9 | 0.8 | 0.9 | 0.8 | 0.8 |
  | Max. jail term | 15 | 30 | ∞ | ∞ | 15 | 15 | 15 |
  | Movement | none | random site in vision | ← | ← | ← | ← | ← |
  | Initial cop density | 0.04 | 0.04 | 0.074 | 0.074 | 0 | 0 | 0.04 |

- **Model I results:** deceptive behavior; free assembly catalyzes outbursts; punctuated equilibrium (Figs. 3, 4); waiting times between outbursts above 50 actives with mean 60, s.d. 55 (Fig. 5), truncated at 30 and logged, slope −0.07; total activation per outburst mean 708, s.d. 230 (Fig. 7); tension Ḡ·B̄/R̄ builds before outbursts (Fig. 8); legitimacy 0.9 → 0.2 "in small increments (of a percent per cycle)" gives no spike and a smoothly rising jail population (Fig. 9), while 0.9 → 0.7 "in one jump" at t = 77 gives "an explosion of actives" and a jailed population that exceeds the first run's (Fig. 10); walking cops down "does 'tip' society into rebellion" (Fig. 11).
- **Model II:** two groups, blue and green; "'going active' means killing an agent of the other ethnic group … indiscriminant"; "agents clone offspring onto unoccupied neighboring sites with probability p each period. Offspring inherit the parent's ethnic identity and grievance"; "a random death age from U(0, max_age)"; cops "arrest — evenhandedly — red agents within their vision". Results: high L and no cops gives peaceful coexistence (Fig. 12); L = 0.8 and no cops gives ethnic cleansing and genocide, "Over a large number of runs (n = 30), genocide is always observed. The victor is random" (Fig. 13); peacekeepers deployed at t = 50 "to random unoccupied sites" typically give safe havens (Fig. 14); cop density 0.04 from the start gives "a stable, but nasty, regime", and withdrawing the cops reverts to genocide; cop densities 0 to 0.1 by 0.002, 50 runs each, capped at 15 000 cycles: mean time to genocide rises with density, and so does its s.d. (Figs. 15–17).

## Finding: the stated arrest rule gives no rebellion

Found while planning (a dry run of this spec's rules, Run 2's inputs, 3 seeds × 3 000 ticks): with the paper's P = 1 − exp(−k·C/A), Run 2 never has an outburst — at most 34 actives, no tick above 50. At L = 0.82, G = 0.18·H ≤ 0.18, and even a crowd outnumbering the cops three to one faces P = 1 − e^(−2.3/3) ≈ 0.54, so R·P swamps G for almost every agent. With ⌊C/A⌋ (NetLogo *Rebellion*'s change; its Info tab: "Without this change, the model does not exhibit punctuated equilibrium") P is ≈ 0 once actives outnumber the cops in view and ≈ 0.9 otherwise, and Run 2 shows the paper's punctuated equilibrium: ~114 outbursts per 3 000 ticks, mean total activation ~830 (paper 708 ± 230), mean wait ~20 ticks (paper 60 ± 55). Epstein's Ascape code may have divided integers; the paper does not say.

**Decision (user, 2026-09-25):** the config default stays the paper's literal rule; the Model I presets turn on `floor_ratio` (only that switch) and their descriptions say why, with the measurement; NetLogo's double count is its own switch, `active_counts_twice`, used only by `cv-netlogo`; the built-in sweep `cv-ratio-rules` shows the dependence; the survey first checks whether plausible ordering or vision changes rescue the literal rule before recording it as unreproduced. Showing where a paper's stated rules fail to produce its results is part of this project's purpose: the README says so plainly.

## Architecture

- **Model kind** `civil` ("Civil Violence"): `ModelKind::Civil`, `ModelConfig::Civil(CivilConfig)` tagged `"model": "civil"`, a `CivilWorld` implementing `Model`, a schema for the Rules panel, `SERIES`, presets and golden entries — the same wiring as Schelling, Ring World and the anasazi. Code lives in `crates/sugarscape-core/src/civil/` (`config.rs`, `world.rs`, `stats.rs` for the outburst bookkeeping, `mod.rs` for the schema and presets' helpers).
- **Variants:** one kind with `variant: rebellion | ethnic`; Model II is Model I plus groups, killing and population dynamics.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `variant` | `rebellion` | reset | Model I or Model II |
| `width`, `height` | 40, 40 | reset | torus size (≥ 5) |
| `agent_density` | 0.7 | reset | agents = round(density × sites) |
| `cop_density` | 0.04 | live | cops = round(density × sites); agent + cop count ≤ sites |
| `legitimacy` | 0.82 | live | L in [0, 1] |
| `threshold` | 0.1 | live | T (may be negative, ≥ −1) |
| `k` | 2.3 | live | arrest constant (> 0) |
| `vision.agent`, `vision.cop` | 7, 7 | live | Euclidean radius in cells (> 0, < half the smaller side) |
| `movement` | true | live | agents move (cops always move) |
| `jail.max`, `jail.infinite` | 30, false | live | J_max in ticks (≥ 1); infinite terms never end |
| `clone_probability` | 0.05 | live | Model II only |
| `max_age` | 200 | live | Model II only (≥ 1) |
| `stop_at_extinction` | false | live | Model II: `finished()` once a group is gone |
| `outburst_threshold` | 50 | live | actives above which an outburst is under way |
| `quirks.floor_ratio` | false | live | NetLogo's `floor(C / A)`: P = 1 − exp(−k·⌊C/A⌋) (see Finding) |
| `quirks.active_counts_twice` | false | live | NetLogo's A = 1 + actives in vision *including the agent itself*, so an already-active agent counts twice |
| `quirks.cop_moves_to_arrest` | false | live | the arresting cop steps onto the arrested agent's site |
| `quirks.jailed_stay` | false | live | jailed agents keep their site, which counts as empty; released agents may share a site |
| `quirks.netlogo_jail_term` | false | live | term = uniform integer in 0..J_max−1 (NetLogo's `random`) |
| `schedule` | `[]` | — | `[{ tick, set: { path: value } }]`, applied before tick `tick` runs (the sugarscape's shape) |
| `ramps` | `[]` | — | `[{ path, start, end, to }]`: a numeric live field moves linearly from its value at `start` to `to` at `end` |

- **Validation:** schedule and ramp paths must name live fields (reset fields, `schedule`, `ramps` and `variant` are rejected); ramp paths must be numeric fields; `start < end`; values are validated as if set directly. Overlapping ramps on the same path are rejected.
- **Ramps:** before tick t runs, for every ramp with start < t ≤ end the field is set to v_start + (to − v_start)·(t − start)/(end − start), where v_start is the field's value when tick `start + 1` begins (recorded on first application, part of the fingerprinted state). Schedule entries at the same tick apply before ramps. Scheduled and ramped values show in `config()` like any live change.
- **Live `cop_density`:** the world adds cops on uniformly random empty sites (free of agents and cops; with `jailed_stay`, a jailed agent's site counts as empty) or removes uniformly random cops until the count matches.

## Setup

Cops first, then agents, each on a uniformly random empty site (NetLogo's order). Each agent draws H and R from U(0,1) and starts quiet; in Model II its group is Blue or Green with probability ½ each, its age 0 and its death age a uniform integer in 1..=max_age (the paper's U(0, max_age), made a whole number of ticks).

## Step (one tick)

1. Apply schedule entries for this tick, then ramps.
2. Collect every free agent and every cop in one list and shuffle it. For each in turn (skipping any killed or arrested earlier this tick):
   - **Move (rule M):** if a cop, or an agent with `movement` on, move to a uniformly random empty site within vision; stay if there is none. The own site is not a candidate.
   - **Agent (rule A):** G = H(1 − L); C = cops within `vision.agent`; A = 1 + active free agents within vision (not counting itself); P = 1 − exp(−k·C/A); N = R·P; active iff G − N > T. With `active_counts_twice`, an already-active agent adds 1 more to A (NetLogo); with `floor_ratio`, P = 1 − exp(−k·⌊C/A⌋).
   - **Model II, if active:** kill one uniformly random free agent of the other group within `vision.agent`, if any; it is removed at once.
   - **Cop (rule C):** among active free agents within `vision.cop`, arrest one uniformly at random, if any: it becomes quiet and jailed with term ⌈U(0, J_max)⌉ ticks (1..=J_max), or J uniform in 0..J_max−1 with `netlogo_jail_term`, or never released when `jail.infinite`. It leaves the lattice unless `jailed_stay`. With `cop_moves_to_arrest` the cop moves onto the arrest site (vacated, or shared with the jailed agent under `jailed_stay`).
3. **Jail:** every jailed agent's term counts down by 1; an agent whose remaining term is 0 is released at the end of the tick, in id order. Without `jailed_stay` it is placed on a uniformly random empty site within `vision.agent` of its arrest site, or a uniformly random empty site anywhere if none (never released if the lattice is full; it waits). With `jailed_stay` it simply becomes free where it is. A term drawn as 0 (NetLogo quirk) releases at the end of the arresting tick.
4. **Model II population:** in a shuffled order, each free agent clones with probability `clone_probability` onto a uniformly random empty Moore neighbor (the eight adjacent sites), if any: the child keeps the parent's group and H, draws a new R, starts quiet at age 0 with a new death age. Then every agent (free or jailed, children excepted) ages by 1 and dies when its age reaches its death age.
5. Record statistics.

**Vision (not defined precisely by the paper; stated here):** the sites (x + dx, y + dy) on the torus with dx² + dy² ≤ v², excluding (0, 0) — radius 1.7 gives the eight Moore neighbors, radius 7 gives 148 sites. NetLogo's `in-radius` is the same set (plus its own patch).

## Statistics

`SERIES`: `population` (free + jailed agents), `active`, `quiet` (free and quiet), `jailed`, `cops`, `legitimacy`, `mean_grievance` (over free agents), `tension` (Ḡ·B̄/R̄ over free agents, B̄ = quiet share of free agents; 0 when there are none), `outbursts`, `mean_wait`, `mean_activation`; in both variants also `blue`, `green`, `killed` (this tick), `extinction` (Model I records `blue` = `green` = 0, `killed` = 0 and `extinction` = tick, so one series list serves both).

- **Outbursts:** an outburst starts on the first tick with `active > outburst_threshold` and ends on the first later tick with `active < outburst_threshold` (the paper's "exceeds 50" and "falls below 50"). Its total activation is the sum of `active` over its ticks (start through the last tick above or at the threshold). A wait is the ticks between an outburst's end and the next one's start. `outbursts` counts finished outbursts; `mean_wait` and `mean_activation` are running means over finished waits and outbursts (NaN before the first).
- **Extinction (Model II):** the first tick at which `blue` or `green` is 0, held thereafter; the current tick while both groups survive (so a capped sweep reads the cap, as the paper's 15 000-cycle runs did).

## Views

- **Color modes:** **Action** (the paper's left screen): quiet agents blue, active red, cops black, empty sand; in Model II quiet agents are drawn in their group's color (blue or green) and actives red. **Grievance** (the right screen): agents shaded from pale to dark red by G, cops black, empty sand. **Group** (Model II): blue or green, cops black. Jailed agents are not drawn (with `jailed_stay` the site shows its free occupant or cop, else empty).
- **Inspect:** an agent's id, H, R, G, estimated P and N at its site now, state (quiet / active / jailed with remaining term), and in Model II its group, age and death age; or a cop.
- **Charts:** Actives, quiet and jailed, Legitimacy and Cops — Figs. 9–11; Actives and tension — Fig. 8; Outbursts (count, mean wait, mean activation); Groups (blue, green, kills) — Model II only.
- **Agents CSV:** id, x, y, state, jail term, H, R, G, group, age, death age.

## Presets

Descriptions state the Table 2 inputs and what the survey measured; the mapping of runs to figures is our reading (the paper does not tie runs to figures). Runs 1–5 set `quirks.floor_ratio` (see Finding); runs 6–8 and the Model II scenarios keep the literal rule (with no cops it never matters; with cops the survey reports both).

The spec originally called for `cv-peacekeepers-withdrawn` (Run 8 with cops withdrawn at a measured tick, expecting reversion to genocide). Measured, Run 8 never stabilizes: one group is already gone by t ≤ 882 in 20 of 20 seeds, so there is no stable regime to withdraw cops from. That preset is dropped.

| Preset | Setup | Paper |
|---|---|---|
| `cv-run-1-no-movement` | Run 1 | Figs. 1–2 (deception) |
| `cv-run-2-punctuated` | Run 2 | Figs. 3–8 |
| `cv-run-3-salami` | Runs 3 & 4; ramp `legitimacy` 0.9 → 0.2 from t = 77 to t = 147 (0.01 per tick) | Fig. 9 |
| `cv-run-4-one-jump` | Runs 3 & 4; schedule `legitimacy` 0.7 at t = 77 | Fig. 10 |
| `cv-run-5-cop-reductions` | Run 5; ramp `cop_density` 0.074 → 0 over a span chosen by measurement | Fig. 11 |
| `cv-run-6-coexistence` | Run 6 (ethnic) | Fig. 12 |
| `cv-run-7-cleansing` | Run 7 (ethnic), `stop_at_extinction` | Fig. 13 |
| `cv-run-8-nasty-regime` | Run 8 (ethnic) | "stable, but nasty" |
| `cv-safe-havens` | Run 7; schedule `cop_density` at t = 50 (density chosen by measurement, stated) | Fig. 14 |
| `cv-netlogo` | NetLogo's defaults (70 % agents, 4 % cops, vision 7, L 0.82, J_max 30) and all five quirks | NetLogo *Rebellion* |

**Compare entries:** "Salami tactics vs one jump — runs 3 and 4" and "Ethnic cleansing vs safe havens — run 7 and peacekeepers".

## Experiments and CLI

- `sugarscape presets | run | sweep` accept `civil`; sweep `set` paths and metric series are validated against the civil config and `SERIES`.
- **`cv-peacekeeping`** (Figs. 15–16): base `cv-run-7-cleansing`, x = `cop_density` from 0 to 0.1, metric final `extinction`, with steps, a tick cap and seeds chosen by measurement to fit a browser run, recorded in its description.
- **`cv-ratio-rules`**: base `cv-run-2-punctuated`, series: the literal rule, `floor_ratio`, `floor_ratio` + `active_counts_twice`; x = `legitimacy` over a measured range; metric final `outbursts` — the Finding as a chart.
- **`cv-jail-waits`**: base `cv-run-2-punctuated`, x = `jail.max`, metric final `mean_wait` — the paper's open conjecture that longer terms raise the mean wait; the description records what was measured.

## Survey

A `civil` claims module in `survey/`, with judges over 20 seeds: Run 2's actives are punctuated (quiet stretches and outbursts above 50; `mean_wait` and `mean_activation` reported against 60 ± 55 and 708 ± 230); run 4's peak actives exceed run 3's and its final jailed count exceeds run 3's; run 5 tips (actives jump during the cop ramp); run 6 keeps both groups alive with few kills; run 7 reaches extinction in every seed with the victor split; peacekeepers delay extinction relative to run 7; run 8's groups coexist with ongoing kills. Claims that fail are reported as unreproduced and the descriptions say so. The literal-rule check: Run 2 without `floor_ratio` under each of (a) the rules as specified, (b) agents deciding before moving, (c) von Neumann vision of radius ⌊v⌋ (the paper's "north, south, east, and west"), (d) cops acting after all agents — the claim "the stated rule gives no outbursts" stands only if none of them produces outbursts above 50. These rescue checks were run during planning (Run 2, seeds 1–5, 3000 ticks), before this spec's Finding was written: (a) as specified, peaks 13–34; (b) agents deciding before moving, peaks 11–15; (d) cops acting after all agents, peaks 14–21; (c) the paper's "north, south, east, and west" read as a Sugarscape cross of 4·⌊v⌋ sites, mean actives ~40 at every tick — constant unrest, never calm, not punctuated. None of the four produces an outburst above 50, so the claim stands.

## Page

- The presets menu gains a **Civil Violence** group; its Rules panel is generated from the schema in the groups World, Agents, Cops, Jail, Population (ethnic only), Outbursts and NetLogo quirks; `schedule` and `ramps` are listed read-only (like the sugarscape's Schedule section) and travel in links and sessions.
- Worker host, Max speed, replay, share/compare links, session files, Compare (same model), recording and Experiments work unchanged. The `finished()` pause (anasazi's end-year path) stops a run at extinction. Editing tools, overlays, trails and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries and fixtures untouched; new golden entries for every civil preset.
- **Core unit:** Euclidean vision on the torus (1.7 → 8 sites, 7 → 148, wrap); P with self counted, C = 0 → P = 0, C = A = 1 → P ≈ 0.9; the T boundary (G − N = T is quiet); `floor_ratio` including the double count; movement to empty sites only and staying when none; no movement for agents with `movement` off while cops still move; arrests only of actives within cop vision; jail-term ranges for both term rules and infinite terms; countdown and release placement (near the arrest site, anywhere, waiting when full); `jailed_stay` sharing and counting as empty; `cop_moves_to_arrest`; live `cop_density` adding and removing cops; schedule then ramp order and ramp interpolation; outburst bookkeeping on a hand-built series; Model II killing only the other group within vision, cloning onto empty Moore neighbors with inheritance and a new R, death by age in and out of jail, `extinction` and `finished()`.
- **Config:** tag round trip; validation (densities, schedule and ramp paths, overlapping ramps); `with_path`.
- **Book-style (`#[ignore]`, release):** the survey's judges over 20 seeds, thresholds measured and recorded in `presets.rs`.
- **Web (Vitest):** schema panel groups and fields (live vs reset, ethnic-only group); civil chart visibility; the two Compare entries; a sweep over a civil base.
- **Browser (controller):** each civil preset renders both color modes and its charts; Inspect; Compare salami vs one jump; the extinction pause; recording; Experiments `cv-peacekeeping`; Max-speed performance; every existing scenario.

## Docs

README: a Civil Violence section (rules, the stated choices, the quirks, the presets and what they reproduce, the sweeps), crediting the paper and NetLogo *Rebellion*. Roadmap: Milestone 11 done.
