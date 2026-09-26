# SugarScape Milestone 14 — Axelrod's culture model and its docking with Sugarscape — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–13 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds (`2026-09-25-other-artificial-societies-design.md`) and the literal-default-plus-named-switch pattern of milestones 11 and 12 (`2026-09-25-civil-violence-design.md`, `2026-09-25-tags-design.md`).
**Source text:** Robert Axelrod, "The Dissemination of Culture: A Model with Local Convergence and Global Polarization", *Journal of Conflict Resolution* 41(2) (1997), 203–226 (Axelrod below; the user's copy is a scan, OCR'd for this spec).
**Docking:** Robert Axtell, Robert Axelrod, Joshua M. Epstein and Michael D. Cohen, "Aligning Simulation Models: A Case Study and Results", *Computational and Mathematical Organization Theory* 1(2) (1996), 123–141 (AAEC below; Axelrod's copy at `public.websites.umich.edu/~axe/research/Aligning_Sim.pdf`).
**Later work:** Claudio Castellano, Matteo Marsili and Alessandro Vespignani, "Nonequilibrium phase transition in a model for social influence", *PRL* 85 (2000), 3536 (CMV); Konstantin Klemm, Víctor M. Eguíluz, Raúl Toral and Maxi San Miguel, "Global culture: A noise-induced transition in finite systems", *PRE* 67 (2003), 045101(R) (KETS).

## Goal

Two phases in one milestone:

- **A.** Axelrod's model as an eighth model kind, `culture`, a full citizen of the playground (worker engine, Max speed, timeline, links, Compare, recording, Experiments, the CLI and the survey), with the paper's runs as presets, the docking paper's and later literature's departures as named switches, and the claims of Axelrod, AAEC, CMV and KETS measured over 20 seeds.
- **B.** Axelrod's rule as an alternative culture rule inside the Sugarscape, so AAEC's mobility experiment runs in the real Sugarscape.

## Non-negotiable constraints

- **Earlier models unchanged.** Every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads and runs as before. In phase B every new field defaults to today's behavior, new random draws happen only when the new rule is on, and new agent state enters the fingerprint only then.
- **Faithful where the sources are specific** (quoted below); where they are silent, the choice is stated here, in the module docs and in the descriptions.
- **One engine path; deterministic; portable** (as milestones 9–13).
- **Truthful descriptions.** Each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **Culture:** "a list of features or dimensions of culture. For each feature there is a set of traits … five features, and each feature can take on any one of 10 traits"; "cultural similarity between two individuals as the percentage of their features that have the identical trait."
- **Space:** "100 sites, arrayed on a 10 by 10 grid … there is no movement … Each site can interact only with its immediate neighbors … four neighbors (north, east, south, and west). Sites on the edge of the map have only three neighbors, and sites in the corners have only two."
- **Dynamics:** "Step 1. At random, pick a site to be active, and pick one of its neighbors. Step 2. With probability equal to their cultural similarity, these two sites interact. An interaction consists of selecting at random a feature on which the active site and its neighbor differ (if there is one) and changing the active site's trait on this feature to the neighbor's trait." Footnote 4: "Select a random site (s), a random neighbor of that site (n), and a random feature (f) … If c(s,f) = c(n,f) and G is not empty, then select a random feature, g, in G(s,n) and set c(s,g) to c(n,g)." "The simulation is done one event at a time"; "the activated site, rather than its neighbor, is the one that may undergo change … to guarantee that each site has an equal chance".
- **Regions and stability:** "a cultural region can be defined as a set of contiguous sites with an identical culture"; "no further change is possible … when every pair of neighboring sites has cultures that are either identical or completely different"; a **cultural zone** is "a set of contiguous sites, each of which has a neighbor with a 'compatible' culture" (at least one feature in common).
- **Results:** the sample run (Table 1, Fig. 1) settles at 3 regions after about 81 000 events; "In a set of 100 runs of this type, the median number of stable regions was three … In 14% of the runs there was only one stable region, whereas in 10% of the runs there were more than six."
- **Table 2** (10 × 10, four neighbors, 10 runs each), average stable regions, features × traits 5 / 10 / 15: F = 5 → 1.0, 3.2, 20.0; F = 10 → 1.0, 1.0, 1.4; F = 15 → 1.0, 1.0, 1.2.
- **Range of interaction:** 8 neighbors (the Moore square) and 12 (those plus the four sites two away in the cardinal directions, "a diamond-shaped neighborhood"); averaged over the nine cultures, 4 / 8 / 12 neighbors give 3.4 / 2.5 / 1.5 stable regions.
- **Territory:** 5 × 5, 10 × 10, 15 × 15 averaged over cultures and neighborhoods give 2.4, 2.5, 2.2. With F = 5, q = 15, four neighbors (Fig. 2; 40 runs per size, 10 at 50 × 50 and 100 × 100): regions rise to "a maximum of about 23, when the territory has 12 × 12 sites", then fall "to about 6 for a territory of 50 × 50 sites and about 2 for a territory of 100 × 100 sites". Footnote 10: 18.6 at 10 × 10 and 2.1 at 100 × 100. With a torus "the peak occurs earlier … and is not as high".
- **Time:** "the time to stability is almost exactly proportional to the number of sites … with 1,024 sites (in a 32 × 32 territory), each site needs an average of 10,036 events … (in a 50 × 50 territory) … 25,900 events"; 100 × 100 needs "a little more than 100,000 events" per site. Fig. 3 (100 × 100): zones fall to two long before regions do ("more than four times as long").
- **Extensions:** "cultural drift (modeled as spontaneous change in a trait)"; "mobility" (footnote 11: confirmed by AAEC to reduce regions).

## Docking and later results

- **AAEC's docked Sugarscape** (vision to the four neighbors, no movement, an agent on every cell, bounded square, Axelrod's rule): Table 1 and Fig. 1's 5 × 5 and 10 × 10 matched; **20 × 20 did not: 16.25 regions (Axelrod) vs 9.23 (Sugarscape)**, traced to activation — Sugarscape activates agents in shuffled sweeps ("without replacement"), Axelrod picks a random site each event ("with replacement"). With random activation the Sugarscape gave Table 1 as 1.2 ± 0.4, 4.1 ± 1.3, 18.8 ± 9.7, 1.0, 1.0, 1.9 ± 1.0, 1.0, 1.0, 1.0 and Fig. 1's three sizes as 9.8 ± 2.8, 20.4 ± 7.9, 14.8 ± 7.0, all statistically indistinguishable.
- **Which site changes:** "Whereas the ACM altered the active agent when a cultural borrowing happened, the original Sugarscape model altered the agent's neighbor" — noticed two months after the docking meeting.
- **Counting:** the Sugarscape counted "distinct cultures (surrogate for counting regions)".
- **Mobility:** 50 × 50, one Gaussian sugar mountain, 100 mobile agents, vision 5–10, move to the best site, eat, growback 1, then Axelrod exchange with one neighbor; a global stop (all agents pairwise identical or completely different). F = 5, q = 15: 1.1 ± 0.3 cultures over 10 runs (fixed agents on 10 × 10: 20.0 ± 10.1 in ACM, 21.3 ± 12.5 in Sugarscape); q = 30: 2.2 ± 1.2.
- **Soup:** random pairing regardless of location; q = 15: never more than one culture in 10 runs; q = 30: 1.4 (7 × 1, 2 × 2, 1 × 3). "If there is any probability of interaction (or if there is any point mutation rate) the long-run attractor is one culture."
- **CMV:** an order–disorder transition at a critical number of traits q_c: below it one culture spans the lattice as it grows; above it the lattice fragments. Axelrod's "large territories have fewer regions" should then hold only for q < q_c.
- **KETS:** drift at rate r below about 1/T (T the relaxation time) drives a finite lattice to one culture; above it, disorder persists.

## Measured in planning

20 seeds unless stated; the survey reproduces each.

- Table 2 (10 seeds in the sweep, 20 in the survey): 1.1, 3.5–5.2, 22–23 regions at five features; 1.0, 1.0, 1.6–1.8 at ten; 1.0, 1.0, 1.0–1.1 at fifteen.
- The sample setup over 1 000 seeds: mean 4.33, median 4, 10 % one region, 18 % above six (the paper: 3.2, 3, 14 %, 10 %). Picking one of four directions and skipping off-map ones (edge sites acting less) gives a median of 3 but a mean of 4.1 — not adopted.
- Neighborhoods over the nine cultures: 4.05, 2.11, 1.40 (the paper 3.4, 2.5, 1.5); soup 1.02.
- Territory (bounded; 10 seeds): 9.8, 17.7, 22.0, 23.9, 23.0, 18.2, 15.9, 10.2 regions at 5, 8, 10, 12, 15, 20, 25, 30 a side; 50 × 50: 4.9; 100 × 100 (8 seeds): 2.25. Torus: 6.8, 9.1, 6.7, 6.7, 5.1, 4.7, 3.9, 3.4.
- Time to stability: 9 090 events per site at 32 × 32, 24 500 at 50 × 50 (the paper: 10 036, 25 900). Zones settle a median 3.6 times sooner than stability at 50 × 50.
- Docking: 20 × 20 random vs sweep medians 16.5 vs 11 (p = 0.01; AAEC 16.25 vs 9.23); neighbor changes 4.90 vs 5.15 (no difference); soup 1 culture in 19 of 20 runs at 15 traits, 1.65 at 30 (AAEC: 1.0, 1.4); cultures equal regions at stability in every run.
- Castellano: at 15 traits regions fall from 20 × 20 to 30 × 30 (medians 16.5 → 9); at 25 they rise (207 → 424); the transition lies between 20 and 25 traits for five features.
- Klemm: after 20 000 ticks on the 12 × 12, 15-trait lattice, medians of 17 cultures without drift, 5 at 10⁻⁴ per event, 55 at 10⁻².
- Mobility (the docked Sugarscape, 20 000 ticks): 4.4 ± 1.4 cultures at 15 traits (3 of 20 runs settled) and 5.7 ± 1.6 at 30 (7 of 20), against AAEC's 1.1 ± 0.3 and 2.2 ± 1.2. Narrower mountains are worse (radius 20: 23.5; radius 12: 58.7).

## Architecture

- **Phase A:** model kind `culture` ("Axelrod Culture"): `ModelKind::Culture`, `ModelConfig::Culture(CultureConfig)` tagged `"model": "culture"`, a `CultureWorld` implementing `Model`, schema, `SERIES`, presets and golden entries — the same wiring as tags. Code in `crates/sugarscape-core/src/culture/` (`config.rs`, `world.rs`, `stats.rs` for regions, zones and bonds, `presets.rs`, `mod.rs`). The WASM crate's source is unchanged.
- **Phase B:** the Sugarscape's `CultureRule` gains fields; agents gain an optional trait vector; `rules/culture.rs` gains the Axelrod event; `World` gains the settled check and `Model::finished()`.

## Phase A: config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `width`, `height` | 10, 10 | reset | 1–200, at least 2 sites (the paper's 6 × 1 dialects strip) |
| `features` | 5 | reset | F, 1–32 |
| `traits` | 10 | reset | q, 2–255 |
| `neighborhood` | `von_neumann` | reset | `von_neumann` (4), `moore` (8), `diamond` (12: Moore plus the four sites two away in the cardinal directions), `soup` (every other site) |
| `boundary` | `bounded` | reset | `bounded` or `torus` |
| `activation` | `random` | live | `random`: each event picks a site uniformly (with replacement); `sweep`: each tick is one shuffled pass over every site (AAEC's Sugarscape) |
| `changes` | `active` | live | `active` (Axelrod) or `neighbor` (the original Sugarscape: the chosen neighbor copies from the active site) |
| `drift` | 0 | live | per event, the probability that the active site instead gets a uniformly random trait on a uniformly random feature (KETS' noise, Axelrod's "cultural drift") |
| `stop_when_stable` | true | live | `finished()` once no neighboring pair can interact and `drift` is 0 |

## Phase A: setup and step

- **Setup:** every site's traits uniform in 0..q, site by site (row-major), feature by feature.
- **Tick:** N events (N = width × height), so a tick is "one event per site" (Axelrod's time unit). With `random` each event picks its site uniformly; with `sweep` the N events visit a shuffled permutation of the sites.
- **Event:** with probability `drift`, set a random feature of the active site to a random trait and stop. Otherwise pick a uniformly random neighbor n of the active site s (in the neighborhood and boundary; `soup`: any other site); pick a random feature f; if c(s, f) = c(n, f) and the sites differ somewhere, pick a uniformly random differing feature g and copy it — into s from n (`active`) or into n from s (`neighbor`).
- **Stability** is tracked incrementally: `active_bonds` counts neighboring pairs sharing between 1 and F − 1 features; after each change only the changed site's pairs are recounted. A world is stable when `active_bonds` is 0; with `stop_when_stable` and no drift, `run` stops at the tick that makes it stable. (`soup` has N(N − 1)/2 pairs; its stability is checked as "every two distinct cultures share nothing", over the distinct cultures.)

## Phase A: statistics

`SERIES`:
- `regions`: connected sets of identical sites, connected through the interaction neighborhood (our reading: Axelrod's "contiguous" is undefined for 8 and 12 neighbors; with `soup` regions equal cultures).
- `zones`: connected sets through neighboring pairs that share at least one feature.
- `cultures`: distinct cultures (AAEC's surrogate).
- `largest_region`: the largest region's share of the sites.
- `mean_similarity`: the mean shared share over neighboring pairs (`soup`: exact, Σ over (feature, trait) of n(n − 1)/2 ÷ (N(N − 1)/2 · F)).
- `active_bonds` (`soup`: the (feature, trait) values held by two or more distinct cultures).
- `changes`: traits changed this tick.
- `stable_at`: the tick the world became stable, else the current tick (like civil's `extinction`, so a capped sweep reads the cap).

## Phase A: views

- **Frame:** (3W − 1) × (3H − 1): each site a 2 × 2 block at (3x, 3y), with one-cell lanes between horizontal and vertical neighbors; lane crossings take the darker of their lanes. With `torus` the wrap lanes are not drawn.
- **Culture:** each distinct culture a stable color from a hash of its traits; a lane between identical sites takes their color (a region reads as one blob); other lanes shaded from black (nothing shared) to light gray (F − 1 shared).
- **Similarity:** Axelrod's Fig. 1: sites neutral, lanes shaded as above (identical: white).
- **Zones:** sites colored by zone; lanes black where nothing is shared.
- **Inspect:** any cell maps to its site (a lane to the pair): the site's traits, its region's and zone's sizes, each neighbor's shared features; a lane shows the pair's shared features. `locate` returns nothing (sites do not move; the page hides Follow).
- **Charts:** Regions, zones and cultures (Fig. 3); Largest region; Mean similarity; Active bonds; Changes. The time axis reads "Events per site".

## Phase A: presets

| Preset | Setup | Source |
|---|---|---|
| `ac-sample-run` | 10 × 10, F 5, q 10 | Fig. 1, the 100-run distribution |
| `ac-many-regions` | 12 × 12, F 5, q 15 | Fig. 2's peak |
| `ac-large-territory` | 100 × 100, F 5, q 15 | Figs. 2–3 (a long run: about 10⁹ events) |
| `ac-torus` | `ac-many-regions` on a torus | "the peak occurs earlier" |
| `ac-sweep-activation` | 20 × 20, F 5, q 15, `sweep` | AAEC's 9.23 |
| `ac-random-activation-20` | 20 × 20, F 5, q 15 | AAEC's 16.25 (Axelrod's) |
| `ac-neighbor-changes` | `ac-sample-run`, `changes: neighbor` | AAEC's original Sugarscape |
| `ac-soup` | 10 × 10, F 5, q 30, `soup` | AAEC's 1.4 |
| `ac-drift` | `ac-many-regions`, drift at a rate chosen by measurement | KETS |

**Compare entry:** "Literal vs Sugarscape activation, 20 × 20 — Axelrod Culture (Compare)": `ac-random-activation-20` and `ac-sweep-activation`.

## Phase A: experiments and CLI

All metrics are final `regions` unless stated, runs stop when stable, and each built-in's tick cap and seeds are chosen by measurement to fit a browser run and recorded in its description (the survey runs the full sizes).
- `ac-table-2`: F × q ∈ {5, 10, 15}², 10 × 10.
- `ac-neighborhoods`: neighborhood ∈ {4, 8, 12, soup} × the nine cultures (as series).
- `ac-territory`: width = height from 2 to 50, series `bounded` / `torus`, F 5, q 15.
- `ac-activation`: width from 5 to 30, series `random` / `sweep`.
- `ac-traits-transition`: q from 5 to 40 at two sizes (20 × 20 and 50 × 50): CMV's transition; whether regions fall or rise with size depends on q.
- `ac-drift`: drift from 0 to 10⁻², metric final `cultures` after a fixed number of ticks (never stable).

Sweeps read a world that stopped on its own at its last values for the ticks it did not run.

## Phase B: Axelrod's rule in the Sugarscape

- **Config:** `culture.rule`: `flip` (the book's rule K; default) or `axelrod`; `culture.features` [5], `culture.traits` [15]; `culture.stop_when_settled` [false]. Read only under `axelrod`; absent in JSON → defaults. `rule`, `features`, `traits` are reset-only; the fields always serialize (older configs lack them and read as `flip`); agents carry traits whenever `culture.rule` is `axelrod`, whether or not K is on.
- **Agents:** under `axelrod` an agent carries F traits, drawn uniformly after every existing draw of `Agent::new` and only when the rule is on. A child takes each feature from a random parent; a replacement newcomer draws fresh traits (AAEC's agents neither reproduce nor die; our reading).
- **Rule:** at rule K's place in the agent's turn, the agent picks one uniformly random occupied von Neumann neighbor (none: nothing) and runs one Axelrod event in which the agent changes. Tags and groups are untouched.
- **Statistics** (only under `axelrod`): `distinct_cultures`; `settled` (1 when every two distinct cultures among living agents share no feature — AAEC's global criterion — else 0).
- **Stop:** with `stop_when_settled`, `World` is finished at the first settled tick; `Model::finished()` returns it and `run` stops.
- **Fingerprint:** traits hashed only under `axelrod`.
- **Page:** the Culture (K) section gains Rule, Features, Traits and Stop when settled; a **Culture** color mode (agents colored by Axelrod culture; always listed, agents gray under `flip`) and a Distinct cultures chart, shown only under `axelrod`.
- **Map:** AAEC's "single (Gaussian) sugar mountain" is the existing single cone peak centered on 50 × 50, radius 35, height 4 (AAEC give neither height nor width).
- **Presets:** `dock-mobility-15` — 50 × 50, 100 agents, vision 5–10, metabolism 0 (nobody starves, as AAEC's agents never die), growback 1, movement on, Axelrod F 5 q 15, stop when settled; `dock-mobility-30` — q 30. Sweep `dock-mobility`: q ∈ {5, 10, 15, 30}, final `distinct_cultures`.

## Survey

A `culture` claims module, 20 seeds (fewer for 100 × 100 if the runtime demands, stated):
- Axelrod: Table 2's nine cells; the sample setup's distribution (median 3, about 14 % one region, about 10 % more than six); neighborhoods 3.4 / 2.5 / 1.5; Fig. 2's rise to about 23 near 12 × 12 and fall to about 6 at 50 × 50 (and about 2 at 100 × 100); the torus's earlier, lower peak; time to stability about 10 036 and 25 900 events per site at 32 × 32 and 50 × 50; zones settle before regions.
- AAEC: 20 × 20 `random` vs `sweep` (16.25 vs 9.23); soup at q 15 and 30 (1.0 and 1.4); `changes: neighbor` against `active`; regions vs cultures; mobility at q 15 and 30 (1.1 and 2.2) against the fixed lattice.
- CMV and KETS: the size trend's reversal above q_c; drift driving the lattice to one culture.

Claims that fail are reported, and the descriptions and README say so.

## Page

- The presets menu gains an **Axelrod Culture** group and the Compare entry; the Rules panel is generated from the schema in groups World, Culture, Interaction and Departures (each departure's help naming its source).
- Worker host, Max speed, timeline, replay, links, sessions, Compare, recording and Experiments work unchanged; `finished()` pauses at stability. Editing tools, overlays, trails, Follow and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `culture` and `dock-*` preset.
- **Core unit (A):** each neighborhood's neighbor sets on bounded and torus lattices (corners 2, edges 3 for four neighbors; 8 and 12); the event rule (copy only when the drawn feature agrees; identical and fully different pairs never change; `neighbor` changes the neighbor); drift; `random` vs `sweep` visits; incremental `active_bonds` against a full recount after many events; regions, zones and cultures on hand-built lattices; `stable_at` and `finished()`; frame layout, lanes and Inspect mapping; keyframe restore.
- **Core unit (B):** `flip` configs unchanged (golden); traits drawn only under `axelrod`; the agent changes itself; `settled` on hand-built populations; the stop.
- **Web:** schema groups, charts, the Compare entry, a sweep over a `culture` base, the Sugarscape panel's new controls and conditional color mode and chart; determinism through the engine.
- **Browser (controller):** each preset's three modes, Inspect on sites and lanes, Compare, the stop, recording, Experiments, the Sugarscape mobility presets, every existing scenario.

## Docs

README: an Axelrod Culture section (rules, stated choices, switches and sources, presets and what they reproduce, sweeps, the docking) and a note in the Sugarscape section on the `axelrod` culture rule; roadmap: Milestone 14 done.
