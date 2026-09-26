# SugarScape Milestone 17 — Demographic Prisoner's Dilemma (Epstein and its replication) — Design

**Date:** 2026-09-26
**Milestone number:** 17 if it lands before the `hk` worktree's model; otherwise renumbered at merge.
**Builds on:** the milestone 1–16 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds and milestone 11's conventions (portable math, named switches for unstated choices, measured descriptions).
**Sources:**
- **WP:** Epstein, "Zones of Cooperation in Demographic Prisoner's Dilemma", SFI Working Paper 97-12-094 (1997; the user supplied the PDF).
- **GSS:** Epstein, *Generative Social Science* (Princeton, 2006), chapter 9 (pp. 199–221; "published previously in *Complexity* 4(2): 36–48", 1998) and its appendix (pp. 222–224, the demographic coordination game). The chapter adds Table 9.1 (Run 1's assumptions) and Table 9.3 (payoff sensitivity) and changes the movement/play rule (below).
- **CD:** the Ascape reimplementation shipped on GSS's CD, known through Radax & Rengs' screenshots of its parameter dialog (`Death Age`, `Fission Wealth = 11`, `Initial Wealth = 6`, `Number Of Agents = 100`, `Size = 30`, the four payoffs).
- **RR:** Radax & Rengs, "Replication of the Demographic Prisoner's Dilemma", MPRA 14419 (2009); published as "Prospects and Pitfalls of Statistical Testing: Insights from Replicating the Demographic Prisoner's Dilemma", *JASSS* 13(4) 1 (2010).

Not recoverable: the original C++ code and the Brookings Java applet (only the 1999 wrapper page is archived). Dorofeenko & Shorish (2002) and Gibaud (2017) formalize altered models and give no comparable numbers.

## Goal

Add Epstein's Demographic Prisoner's Dilemma as an eleventh model kind (after milestone 16's ethnocentrism), with the working paper's and the published text's rules, Radax and Rengs' unstated timing choices, soup, metabolism and the coordination game as named switches and presets — a full citizen of the playground (worker engine, Max speed, replay and links, keyframes and the timeline, stop rules, Compare, recording, Experiments, the CLI and the survey) — and measure every claim, including Tables 1, 2 and 9.3 and RR's factorial test.

## Non-negotiable constraints

- **Earlier models unchanged.** Every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **The default is the sources' stated rule** (the published text, GSS Table 9.1, the CD's settings where the prose is silent); each disagreement or unstated choice is a named switch.
- **One engine path** over the `Model` trait.
- **Deterministic and portable.** A function of (config, seed); all draws from the world's seeded RNG with `u32` ranges; no platform transcendental functions; fingerprints identical native and WASM.
- **Truthful descriptions**, with measurements, including what does not reproduce.

## Source summary

- **Rule (GSS, the published text):** "Choose a random unoccupied site within your vision; go there and play your strategy against each neighbor. (If there is no unoccupied site within the agent's vision, the agent remains in place and plays all current neighbors.)" **WP:** "Choose a random site within your vision; go there and play your strategy against a random neighbor."
- **Setup (GSS Table 9.1):** T 6, R 5, P −5, S −6; 30 × 30 torus; von Neumann; vision 1; mutation 0; metabolism 0; no maximum lifetime; "accumulation needed to clone" 10; "initial endowment to offspring" 6; 100 initial agents at random locations with random strategies; asynchronous updating; random call order. **CD:** `Fission Wealth = 11` (the threshold "exceeds 10" as an integer test), `Initial Wealth = 6` (the only statement of the initial agents' wealth).
- **Demography (WP/GSS):** payoffs accumulate; "if accumulated payoffs are negative, agents die and are removed"; above the threshold, with an unoccupied site within vision, "the agent has an offspring, who begins life on one of these sites with a nominal initial endowment subtracted from the parent's wealth"; offspring inherit the strategy, flipped with probability `mutation`; "an agent's initial age is a random integer between 1 and the maximum age". **Call order (GSS p. 206):** "a pair of agents is selected at random and the agents swap positions in the list. This random swapping is done N/2 times after each cycle."
- **Runs:** Run 1 (no maximum age): about 5 to 1 by t = 50; Table 1 / 9.2 (30 runs, t = 500): cooperators range (752, 806), mean 779, s.d. 15; defectors (93, 148), 121, 15. Run 2 (maximum age 100): Table 2 / 9.4: cooperators (708, 846), 784, 29; defectors (45, 160), 99, 25. Run 3 (R = 2): more oscillatory. Run 4 (R = 1): predator–prey cycles; outcomes differ by seed (coexistence, cooperator monopoly or extinction); "cooperators ultimately do better with a low payoff (R = 1) than with a high one (R = 5)!… But it is not robust." Run 5 (Run 2 with 50 % mutation): cooperation persists through 10,000 cycles. Soup ("equiprobable random agent pairings") runs to pure defection. Payoffs shifted by 6 (12, 11, 1, 0) run to pure defection (Fig. 13). "Equivalent mathematically": the shifted payoffs with a metabolism of 6 "recover, in effect, our initial payoffs" (metabolism = "a fixed decrement to accumulated payoff per cycle", WP note 29); the passage calls the pure-defection run "figure 12" (the 50 %-mutation run; it means Fig. 13). Footnote 27: payoffs 16, 11, 5, 4 with maximum lifetime 10 evolve to cooperative monopoly. Surrounded cooperators: cooperators all eight of whose Moore neighbours are cooperators (Fig. 10–11).
- **GSS Table 9.3:** 45 payoff vectors (T = 2 … 10, R = 1 … T − 1, S = −T, P = −R), 30 runs each at t = 500, Run 1's other settings: range, mean, s.d. and 95 % CI for cooperators and defectors (e.g. (10, 9): 809 / 77; (6, 5): 779 / 121; (4, 1): 0 / 0). Row (4, 2)'s defector CI "(254, 376)" contradicts its mean 265 and s.d. 32 (≈ (253, 277)).
- **GSS appendix:** the same mechanics with payoffs [1, −3, −3, 1], death age 1,000, mutation 0 give persistent "norm maps" (regions of each convention, accidents at their borders).
- **RR:** seven unstated binary choices — dead removed immediately vs at the end of the cycle; death immediately (even on another's turn) vs on one's own turn; endowment taken from the parent vs granted; newborns' age random vs 0; asynchronous vs synchronous (all move, then all play, then all reproduce); RNG library; Repast's shuffle vs Epstein's swaps — a 2⁷ factorial × 30 runs, t-tests against Tables 1 and 2: **1 of 128 settings reproduces Run 1 (a synchronous one, which they set aside as "radically different from the original model"; their summary counts none), 7 of 128 reproduce Run 2, none both**; best Run 2 fit: removal at end of cycle, immediate death, endowment granted, random newborn age, asynchronous, Epstein's swaps → 780 (25) / 97 (22).

## Architecture

- **Model kind** `dpd` ("Demographic PD"): `ModelKind::Dpd`, `ModelConfig::Dpd(DpdConfig)` tagged `"model": "dpd"`, `DpdWorld` implementing `Model`, keyframes (the world derives `Clone`), a schema, `SERIES`, presets and golden entries. Code in `crates/sugarscape-core/src/dpd/` (`config.rs`, `world.rs`, `stats.rs`, `presets.rs`, `mod.rs`).
- **Lattice:** the shared square periodic geometry (as ethnocentrism reuses the spatial games' tables), von Neumann for movement and play, Moore for the surrounded index.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `width` | 30 | reset | the torus is `width` × `width` |
| `agents` | 100 | reset | initial agents (random empty sites) |
| `initial_cooperators` | 0.5 | reset | chance an initial agent cooperates |
| `initial_wealth` | 6 | reset | the initial agents' wealth (CD) |
| `t`, `r`, `p`, `s` | 6, 5, −5, −6 | live | payoffs: T to a defector against a cooperator, R to mutual cooperators, P to mutual defectors, S to a cooperator against a defector (any finite values: the coordination game breaks the PD ordering) |
| `fission_wealth` | 11 | live | an agent with wealth ≥ this may clone (CD's integer form of "exceeds 10") |
| `endowment` | 6 | live | an offspring's starting wealth |
| `max_age` | 0 | live | 0: no maximum |
| `metabolism` | 0 | live | wealth charged per cycle (or per interaction) |
| `metabolism_per` | `cycle` | live | `cycle` or `interaction` |
| `mutation` | 0 | live | chance an offspring's strategy flips |
| `vision` | 1 | reset | movement radius (von Neumann) |
| `play` | `each_neighbor` | live | `each_neighbor` (GSS) or `random_neighbor` (WP) |
| `pairing` | `space` | live | `space` or `soup` (random partners and random placement anywhere) |
| `death_timing` | `immediate` | live | `immediate` (as wealth goes negative, even on another's turn) or `own_turn` |
| `removal` | `immediate` | live | `immediate` or `end_of_cycle` |
| `endowment_from` | `parent` | live | `parent` (subtracted) or `granted` |
| `newborn_age` | `random` | live | `random` (1 … max age) or `zero` |
| `updating` | `asynchronous` | live | `asynchronous` or `synchronous` (all move, then all play, then all reproduce) |
| `shuffle` | `swaps` | live | `swaps` (N/2 random pair swaps, GSS) or `full` (Fisher–Yates) |
| `newborns_act` | `next_cycle` | live | `next_cycle` or `this_cycle` |
| `end` | 0 | live | the last cycle (0: never) |
| `schedule` | `[]` | — | as milestone 11's (live paths only) |

Validation rejects on the field: non-finite payoffs (no ordering or sign is imposed: several runs use non-negative payoffs and the coordination game is not a PD); `width` 3–200; `agents` 0 … width²; probabilities outside [0, 1]; `fission_wealth` and `endowment` negative; `vision` 1–10.

## Rules

A cycle (asynchronous): the agent list, in its current order; for each agent still alive:

1. **Move** (`space`): a uniformly chosen unoccupied site within `vision` (von Neumann distance ≤ vision, excluding its own); none → stay. (`soup`: a uniformly chosen unoccupied site anywhere; none → stay.)
2. **Play**: `each_neighbor`: one game with each occupied von Neumann neighbour, in direction order; `random_neighbor`: with one uniformly chosen occupied neighbour, if any; `soup`: with one uniformly chosen other living agent. Both players add their payoff; `metabolism_per: interaction` charges `metabolism` to both per game. With `death_timing: immediate` a player whose wealth goes below 0 dies at once (and, with `removal: immediate`, leaves its site at once; else it is marked and removed at the cycle's end, taking no further part); with `own_turn` a partner below 0 dies only at the end of its own next turn.
3. **Reproduce**: wealth ≥ `fission_wealth` and an unoccupied von Neumann neighbour (`soup`: an unoccupied site anywhere) → an offspring on a uniformly chosen one, with `endowment` (subtracted from the parent with `endowment_from: parent`), the parent's strategy flipped with probability `mutation`, age per `newborn_age`, inserted per `newborns_act` (`next_cycle`: appended after this cycle; `this_cycle`: appended to the current list and processed in its turn).
4. **Age and die**: age + 1; `metabolism_per: cycle` charges `metabolism`; wealth below 0 or (with `max_age` > 0) age above `max_age` → death.

After the list: dead agents still present are removed; the list is reordered per `shuffle`. `updating: synchronous` runs the phases for all agents in turn (all move, then all play, then all reproduce, then age and death). Initial ages (and, with `newborn_age: random`, newborns') are uniform in 1 … `max_age` (0 with no maximum).

## Choices the sources leave open

1. **Rule** — WP vs GSS: default GSS (`each_neighbor`, unoccupied destination, stay and play if blocked); `play: random_neighbor` is the WP.
2. **Initial wealth** — never in the prose; the CD's 6.
3. **Threshold** — "exceeds 10": the CD's `Fission Wealth = 11` as `≥ 11`.
4. **RR's six model choices** — the switches above; defaults read the text literally: death as wealth goes negative (`immediate`), removal at once (asynchronous updating), endowment "subtracted from the parent's wealth" (`parent`), "an agent's initial age is a random integer…" (`random`, every agent), asynchronous, Epstein's swaps.
5. **Newborns acting this cycle** — no source says: `next_cycle`.
6. **Metabolism timing** — "per cycle" (WP note 29): default; `interaction` tests the equivalence claim.
7. **Soup placement** — "equiprobable random agent pairings": partners uniform over living agents; offspring placed on a random empty site.
8. **Age and death order** — ageing and metabolism after reproduction, then the death checks.

## Statistics

`SERIES`: `cooperators`, `defectors`, `population`, `cooperator_share` (NaN when empty), `surrounded` (surrounded cooperators), `wealth_c`, `wealth_d` (means; NaN with none), `births`, `deaths`.

## Views

- **Colour modes:** **Strategy** (default; cooperators blue, defectors red, GSS's colours), **Wealth** (heat), **Age** (heat), **Surrounded** (surrounded cooperators highlighted). Empty sites dark.
- **Inspect:** an agent's strategy, wealth, age and maximum age, whether surrounded, this cycle's payoffs and games, and each neighbour with what one game between them pays each under the current payoffs (agents move, so per-partner totals are not kept: plan Decision 13); an empty site says so.
- **Charts:** **Population** (cooperators, defectors), **Cooperator share**, **Surrounded cooperators**, **Wealth** (mean C, mean D), **Births and deaths**, against the **cycle** (Epstein's word). No cooperator-vs-defector phase diagram (Figs. 4 and 9): the model charts are time charts only, and a phase diagram would be a new chart type (plan Decision 22).

## Presets

| Preset | Setup | Source |
|---|---|---|
| `dpd-run-1` | the defaults | Run 1, Table 1 |
| `dpd-run-2` | max age 100 | Run 2, Table 2 |
| `dpd-run-3` | max age 100, R 2 | Run 3 |
| `dpd-run-4` | max age 100, R 1 | Run 4 |
| `dpd-run-5` | max age 100, mutation 0.5 | Run 5 |
| `dpd-working-paper` | `play: random_neighbor` | WP's rule |
| `dpd-closest` | `play: random_neighbor`, `initial_wealth: 0`, `newborns_act: this_cycle` | the readings that reproduce Tables 1 and 2 together (plan Decision 17) |
| `dpd-soup` | `pairing: soup` | the soup variant |
| `dpd-shifted` | max age 100, payoffs 12, 11, 1, 0 (GSS: "maximum age of 100, zero mutation" — Run 2's settings; plan Decision 10) | Fig. 13 |
| `dpd-metabolism` | `dpd-shifted` with metabolism 6 | the "equivalent" run |
| `dpd-footnote-27` | payoffs 16, 11, 5, 4, max age 10 | footnote 27 |
| `dpd-rr-best` | RR's best Run 2 fit | RR |
| `dpd-coordination` | CD's payoffs [1, −3, −3, 1] as (CC, CD, DC, DD): R 1, S −3, T −3, P 1; max age 1,000 | GSS appendix |

(`dpd-shifted` and `dpd-metabolism` take Run 2's settings, as GSS states them — not Run 5's, which the working paper's context and its "figure 12" suggest: with 50 % mutation pure defection is impossible. No reading converges to pure defection; plan Decision 10 and the measurements.)

**Compare entries:** "Working paper vs published rule — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-working-paper`), "Negative payoffs vs shifted with metabolism — Demographic PD (Compare)" (`dpd-run-2` vs `dpd-metabolism`: `dpd-metabolism` has Run 2's settings), "Space vs soup — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-soup`), "Published rule vs closest reading — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-closest`; plan Decision 21).

## Experiments and CLI

Thirty seeds, 500 cycles, metric the final value unless noted.

- `dpd-payoffs`: Table 9.3's 45 cells (T = 2 … 10 as series, R = 1 … 9 as x, each value setting `p = −r` and `s = −t`); the sweep format cannot skip a cell, so the 36 cells with R ≥ T also run and the description says so (plan Decision 15); metric `cooperators` (the survey reads `defectors` itself).
- `dpd-mutation`: base Run 2; x = mutation 0 … 0.5.
- `dpd-metabolism`: base `dpd-shifted`; x = metabolism 0 … 6; series `metabolism_per` cycle vs interaction; metric `cooperators`.
- `dpd-max-age`: base Run 1; x = max age 10 … 1,000.
- `sugarscape presets | run | sweep` accept `dpd`.

## Claims to test (survey and book-style tests, 30 seeds as Epstein)

- **Epstein:** Table 1 and Table 2 (means and ranges); Table 9.3 cell by cell (our mean inside Epstein's 95 % CI, and within his range); Run 1's ~5 : 1 by t = 50; Run 4's oscillation (a measured cycle count) and seed-dependent outcomes; the R = 1 paradox (share of seeds ending in cooperator monopoly at R = 1 vs R = 5); Run 5's persistence through 10,000 cycles; soup → pure defection; shifted payoffs → pure defection; the metabolism equivalence (per cycle, per interaction: exact per interaction, measured); footnote 27's monopoly (printed R = 11, though "hiked by ten" gives 15: both measured); the coordination game's persistent regions.
- **RR:** the 64 combinations of the six model switches (RNG excluded) × 30 seeds against Tables 1 and 2 (RR: 1 / 7 of 128, none both); RR's best fit. The core's book-style test runs all 64; the survey runs subsets (the 32 asynchronous settings for Run 1, RR's own seven Run 2 fits — six without the RNG column), plan Decision 24.

Tolerances come from the measurements (as milestones 11–16).

## Page

- A **Demographic PD** presets group; the schema panel in groups **Game** (T, R, P, S), **Population** (width, agents, initial cooperators, initial wealth, fission wealth, endowment, max age, metabolism and its timing, vision), **Evolution** (mutation), **Timing** (death timing, removal, endowment from, newborn age, newborns act, updating, shuffle), **Interaction** (play, pairing), **Run** (end); the four colour modes; the charts; Inspect; the four Compare entries; `defaultForm('dpd')` (x = R 1–5, the form's usual 3 seeds, final `cooperators` at 500; plan Decision 23).
- Keyframes, the timeline, stop rules, share links, sessions, recording and Compare work unchanged.

## Testing

- **Core unit:** hand-built worlds for each rule and switch — movement to an unoccupied site within vision and staying when blocked; play with each neighbour vs one random neighbour, both players paid; the fission threshold (≥ 11) and endowment from the parent or granted; death timing and removal; initial and newborn ages; metabolism per cycle vs per interaction; soup partners and placement; mutation; the swap shuffle; synchronous phases; newborns acting this cycle; the surrounded index; validation errors on the named field.
- **Golden:** entries for every preset; earlier entries untouched; WASM equal to native.
- **Book-style (`#[ignore]`, release):** the claims above, thresholds measured.
- **Web (Vitest):** schema fields; charts; Inspect rows; Compare entries; `defaultForm('dpd')`; determinism fingerprints.
- **Browser (controller):** every preset, the colour modes, Inspect, the Compare entries, the sweeps, Max speed.

## Docs

README: a Demographic PD section in the civil-violence style — the rules, the WP/published difference, the choices above, what reproduces and what does not (Tables 1, 2 and 9.3; RR's factorial result as we measure it), the metabolism claim, the text's slips (the "figure 12" reference, Table 9.3's (4, 2) interval), the Compare entries and sweeps — crediting Epstein (WP, *Complexity*, GSS) and Radax & Rengs. Roadmap: this milestone done; the demographic PD entry marked done.
