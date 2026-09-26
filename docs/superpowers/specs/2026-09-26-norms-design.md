# SugarScape Milestone 20 — Norms and Metanorms (Axelrod; Galán & Izquierdo) — Design

**Date:** 2026-09-26
**Builds on:** the milestone 1–19 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds and the literal-default-plus-named-switch pattern of milestones 11–18.
**Source text:** Robert Axelrod, "An Evolutionary Approach to Norms", *American Political Science Review* 80(4) (1986), 1095–1111 (Axelrod below; `papers/norms/axelrod-1986-apsr-evolutionary-approach-to-norms.pdf`).
**Replication:** José Manuel Galán and Luis R. Izquierdo, "Appearances Can Be Deceiving: Lessons Learned Re-Implementing Axelrod's 'Evolutionary Approach to Norms'", *JASSS* 8(3) 2 (2005) (G&I below; `papers/norms/galan-izquierdo-2005-jasss-appearances-can-be-deceiving.html`).

## Goal

Axelrod's norms and metanorms games — 20 agents evolving boldness and vengefulness — as a model kind `norms` ("Norms and Metanorms"), a full citizen of the playground: Axelrod's figures and his dominance variant as presets, G&I's departures (run length, mutation, meta-payoffs, temptation, three other selection rules, and the unstated refill and tie details) as named switches and sweeps, and both papers' claims measured over many seeds.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical).
- **Truthful descriptions:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **The norms game (Axelrod Fig. 1):** "It begins when an individual (i) has an opportunity to defect … accompanied by a known chance of being observed … called S. If S is .5, each of the other players has an even chance of observing a defection." Defecting gives "T … equal to 3, and each of the others are hurt (H) … H equal to −1". Observers "may choose to punish the defector … P = −9 … the punisher has to pay an enforcement cost (E) equal to −2". Boldness: "The player will defect whenever the chance of being seen by someone is less than the player's boldness … S < Bᵢ"; vengefulness "is the probability that the player will punish someone who is defecting".
- **Simulation:** boldness and vengefulness "each allowed to take one of eight levels, from 0/7 to 7/7 … six bits". "(1) The strategies for the initial population of 20 players are chosen at random … (2) … Each individual gets four opportunities to defect. For each of these opportunities, the chance of being seen, S, is drawn from a uniform distribution between 0 and 1." Table 1's worked example (B = 2/7, V = 4/7): one defection (3), punished once (−9), hurt by 36 defections (−36), punished 9 of them (−18); total −60. "(3) … give an average individual one offspring and to give two offspring to an individual who is one standard deviation more effective than the average. An individual who is one standard deviation below the population average will not have his or her strategy reproduced at all. For convenience, the number of offspring is adjusted to maintain a constant population of 20 … a 1% chance that each bit of an individual's new strategy will be altered." "(4) … repeated for 100 generations … (5) … five complete runs."
- **Fig. 2 (norms, 5 runs × 100 generations):** "In one of the runs, there was a moderate level of vengefulness and almost no boldness … On two other runs there was little boldness and little vengefulness, and on the remaining two runs, there was a great deal of boldness and almost no vengefulness." The dynamics: boldness falls, then vengefulness, then boldness rises — "a sad but stable state".
- **Metanorms (Fig. 3, note 2):** "the other players have a chance to see and punish" someone who saw but did not punish; "a player's vengefulness against nonpunishment is the same as the player's vengefulness against an original defection"; "the chance of being seen not punishing is the same as the chance of the original defection being seen"; "P′ = −9 … E′ = −2". Fig. 4: "In all five runs a norm against defection was established"; "the metanorms game can prevent defections if the initial conditions are favorable enough".
- **Dominance:** "letting P = −3 for whites while retaining P = −9 for blacks … letting the population be 20 whites and 10 blacks … resistance to punishment and increased size can help a group, but only if there are metanorms. Without metanorms, even members of the stronger group tend to be free riders … low vengefulness and high boldness in both groups. When metanorms are added, it becomes relatively easier for the strong group to keep the weak group from being bold."

## Replication (G&I)

- Parameters as Axelrod (their Table 1). "Agents in the old generation with a payoff exceeding the population average by at least one standard deviation are replicated twice … at least one standard deviation below … eliminated … The number of agents is kept constant, but Axelrod does not specify exactly how. The particular algorithm … randomly eliminating (if there are more than 20 agents) or replicating (if there are fewer …)". Note 4: when every agent has the same payoff, "every agent is replicated twice, and then half of the newly created agents are randomly eliminated … Proceeding in a different way … can alter the long-term results significantly." "Whenever a bitstring is replicated … every bit has a certain probability to be flipped." Metanorms: seen "by each of the other 18 agents (excluding the defector)".
- Regions (§5.12): **norm collapse** — average boldness ≥ 6/7 and average vengefulness ≤ 1/7; **norm establishment** — average boldness ≤ 2/7 and average vengefulness ≥ 5/7.
- Norms game (Fig. 4, 1,000 runs to 10⁶ generations): "the norm collapses almost always", and after 100 generations the outcomes vary, as Axelrod saw. The only ESS is total collapse (bᵢ = 1, vᵢ = 0).
- Metanorms (Fig. 5): "Even though after 100 generations the norm is almost always established, as time goes by … the norm usually collapses." Mutation 0.001 (Fig. 7): collapse "much more quickly" and more stable. ME = −0.2, MP = −0.9 (Fig. 9): "the norm quickly collapses and such state is sustained in the long term. Axelrod's conclusions are reversed." T = 10 (Fig. 11): "the norm is clearly established in almost all runs" (the norm-established ESS is bᵢ = 11/169, vᵢ = 1).
- Other selection rules (Fig. 12): **random tournament** (pick two at random, copy the higher payoff, ties at random, 20 times), **roulette wheel** (probability ∝ payoff − the generation's minimum; all equal → tournament), **average** (≥ mean twice, else eliminated; refill randomly). "If, for instance, random tournament is chosen, the states where the norm has collapsed are quickly reached."

## Measured in planning

100 seeds at generation 100; 50 seeds for long runs unless stated; the survey reproduces each.

- Axelrod Fig. 2 (norms, generation 100): low boldness with vengefulness ≥ 3/7 in 35, both low in 31, high boldness with low vengefulness in 27; established 4, collapsed 19. Fig. 4 (metanorms): established 92, collapsed 0.
- Dominance (generation 100): without metanorms boldness 0.94 (strong) and 0.86 (weak), vengefulness 0.01 and 0.03; with metanorms boldness 0.06 and 0.02, vengefulness 0.93 and 0.90.
- G&I, 100 seeds, established / collapsed at 100, 10³, 10⁴, 10⁵ generations: norms 0.04/0.19, 0/0.99, 0/0.99, 0/1.00; metanorms 0.92/0, 0.84/0.01, 0.81/0.07, 0.52/0.43; mutation 0.001 0.93/0, 0.57/0.12, 0.11/0.83, 0/1.00; meta-payoffs ÷ 10 0.06/0.12, 0/0.92, 0/1.00, 0/1.00; T = 10 1.00/0 throughout; tournament 0.50/0.01, 0.08/0.80, 0/0.98, 0/0.98; roulette 0.73/0, 0.61/0.23, 0.06/0.88, 0.01/0.93; above-the-mean 0.92/0, 0.72/0.18, 0.09/0.90, 0/1.00. At 10⁶ (20 seeds): metanorms established 0.10, collapsed 0.90.
- Readings (metanorms, 100 seeds at 10⁵): ties drift (G&I) 0.52/0.43; ties kept 0.92/0.03; ranked refill 0/1.00; ranked refill with ties kept 0/0.99.

## Architecture

Model kind `norms` ("Norms and Metanorms"): `ModelKind::Norms`, `ModelConfig::Norms(NormsConfig)` tagged `"model": "norms"`, a `NormsWorld` implementing `Model`, schema, `SERIES`, presets and golden entries — the same wiring as the other models. Code in `crates/sugarscape-core/src/norms/` (`config.rs`, `world.rs` for the game and evolution, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `agents` | 20 | reset | population (2–200); ignored under `groups` |
| `metanorms` | false | live | the metanorms game |
| `rounds` | 4 | live | opportunities to defect per agent per generation |
| `temptation` | 3 | live | T |
| `hurt` | −1 | live | H |
| `punishment` | −9 | live | P |
| `enforcement` | −2 | live | E |
| `meta_punishment` | −9 | live | MP (P′) |
| `meta_enforcement` | −2 | live | ME (E′) |
| `mutation` | 0.01 | live | per bit of each offspring |
| `selection` | `axelrod` | live | `axelrod`, `tournament`, `roulette`, `average` |
| `refill` | `random` | live | `random` (G&I) or `ranked` |
| `all_equal` | `drift` | live | `drift` (G&I's note 4) or `keep` |
| `groups` | off | reset | Axelrod's dominance: `groups.enabled`, `strong` 20, `weak` 10, `strong_punishment` −3 (the weak group uses `punishment`) |
| `stop_at` | 0 | live | `finished()` at this generation (0: never) |

## Step (one generation)

1. **Rounds.** For each round, for each agent i in index order: draw S ~ U[0, 1); i defects iff S < Bᵢ. If it defects: i gets T, every other agent H; for each other agent j in index order, j sees with probability S; if j sees, j punishes with probability Vⱼ (i gets P — the strong group's P under `groups` — and j gets E); with `metanorms`, if j saw and did not punish, each agent k ∉ {i, j} in index order sees j with probability S and, if it sees, metapunishes with probability Vₖ (j gets MP, k gets ME).
2. **Selection** on the generation's payoffs (within each group under `groups`), producing offspring (copies of parents' 6-bit strategies):
   - `axelrod`: s.d. is the population standard deviation (÷ n). If s.d. = 0 (all equal): `drift` — two offspring each; `keep` — one each. Otherwise payoff ≥ mean + s.d. → two, ≤ mean − s.d. → none, else one.
   - `average`: payoff ≥ mean → two, else none.
   - `tournament`: n times, two uniform picks (with replacement), copy the higher payoff (ties uniform).
   - `roulette`: n times, a pick with probability ∝ payoff − min; all equal → tournament.
3. **Refill** (`axelrod` and `average`) to n: `random` — remove uniformly random offspring while above n, duplicate uniformly random offspring while below; `ranked` — remove offspring of the lowest-payoff parents first (ties by index), duplicate those of the highest first.
4. **Mutation:** each bit of each offspring flips with probability `mutation`. The new generation's strategies replace the old; payoffs reset.

Each period plays the current generation, records it, and breeds the next: the statistics, the plane, the strip and Inspect describe the generation that just played (period 0: the starting population, no play). The fingerprint covers the next generation.

Random draws follow this order, so runs are deterministic. Strategies start with uniform random bits.

## Statistics

`SERIES`: `mean_boldness`, `mean_vengefulness` (levels ÷ 7), `mean_payoff`, `defections`, `punishments`, `metapunishments` (this generation's counts), `established`, `collapsed` (0/1, G&I's regions on the new generation's means), `copied_equal` (1 when every payoff tied and the `all_equal` reading applied), and under `groups` `strong_boldness`, `weak_boldness`, `strong_vengefulness`, `weak_vengefulness` (0 without groups).

## Views

- **B–V plane** (left): 8 × 8 levels, 12 cells each (96 × 96); boldness right, vengefulness up; each cell shaded by the number of agents holding that strategy; G&I's regions outlined (established top-left, collapsed bottom-right); the mean (B, V) of the last 200 generations as a fading trail.
- **Agent strip** (right, 6 cells gap): one row per agent (4 cells tall): a boldness bar (red), a vengefulness bar (blue) and a payoff (or, under **Group**, group) swatch; the strong group first, a gap, then the weak. The event counts are in Inspect and the Events chart. The frame is 166 cells wide (the 96-cell plane, a 6-cell gap, a 64-cell strip).
- **Color modes:** **Agents** (density), **Payoff** (plane cells and rows by this generation's payoff), **Group** (listed only under `groups`).
- **Inspect:** a plane cell — its (B, V) and agents; a strip row — the agent's bits, B, V, payoff, event counts and parent. `locate` returns an agent's strip row; Follow available.
- **Charts:** Boldness and vengefulness; Events (defections, punishments, metapunishments); Mean payoff; Norm state (`established`, `collapsed`); By group (under `groups`). Time axis: Generations.

## Presets

| Preset | Setup | Source |
|---|---|---|
| `ax-norms` | norms, stop at 100 | Axelrod Fig. 2 |
| `ax-metanorms` | metanorms, stop at 100 | Fig. 4 |
| `ax-dominance` | groups, norms, stop at 100 | Dominance |
| `ax-dominance-metanorms` | groups, metanorms, stop at 100 | Dominance |
| `gi-metanorms-long` | metanorms, stop at 20 000 | G&I Fig. 5 |
| `gi-low-mutation` | metanorms, mutation 0.001, stop at 20 000 | Fig. 7 |
| `gi-mild-metanorms` | metanorms, ME −0.2, MP −0.9, stop at 20 000 | Fig. 9 |
| `gi-temptation-10` | metanorms, T 10, stop at 20 000 | Fig. 11 |
| `gi-tournament` | metanorms, tournament, stop at 20 000 | Fig. 12 |

**Compare entry:** "Axelrod's selection vs a random tournament — Norms and Metanorms (Compare)": `gi-metanorms-long` and `gi-tournament`.

## Experiments and CLI

Seeds and horizons measured to fit a browser run and recorded in each description; the survey runs longer.
- `norms-horizon`: share of runs in each region (`established`, `collapsed`) at generations 100, 1 000, 10⁴, 10⁵, series norms/metanorms (G&I Figs. 4–5).
- `norms-mutation`: metanorms, against mutation (0.001 … 0.05).
- `norms-meta-payoffs`: metanorms, against a scale on ME and MP (0.1 … 1).
- `norms-temptation`: metanorms, against T (1 … 12).
- `norms-selection`: metanorms, collapse share per selection rule.
- `norms-readings`: metanorms, `refill` × `all_equal`.
- `norms-dominance`: each group's boldness, series norms/metanorms.
The CLI names the stop `(its last generation)`. A run stopped at `stop_at` holds its values (`holds_when_finished`), so a sweep over `stop_at` reads each run at its stop; the built-in sweeps use `stop_at` as their horizon.

## Survey

A `norms` claims module, 100 seeds unless stated:
- Axelrod: Table 1's arithmetic (a hand-built generation); Fig. 2 — at generation 100 the norms game's outcomes spread across the regions and establishment is rare; Fig. 4 — at generation 100 the metanorms game establishes the norm in (nearly) every run; dominance — without metanorms both groups end bold and unvengeful; with metanorms the weak group is kept less bold than without.
- G&I: norms game collapsed in nearly all runs by 10⁵ generations; metanorms established at 100 and mostly collapsed by 10⁵; mutation 0.001 collapses faster than 0.01; mild meta-payoffs collapse quickly; T = 10 keeps the norm; tournament, roulette and average collapse sooner than `axelrod`; `refill`/`all_equal` change the long-run result (note 4).
Claims that fail are reported, and the descriptions and README say so.

## Page

The presets menu gains a **Norms and Metanorms** group and the Compare entry; the Rules panel is generated from the schema in groups Population, Payoffs, Evolution and Readings. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged. Editing tools, overlays, trails and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `norms` preset.
- **Core unit:** defection iff S < B; observation and punishment probabilities at 0 and 1; Table 1's payoff arithmetic on a hand-built world; metanorms only on non-punishers and never by the defector or the non-punisher; each selection rule (the s.d. boundaries, the all-equal case both ways, average, tournament ties, roulette's shift and all-equal fallback); refill both ways; mutation at 0 and 1; groups (selection within groups, group sizes kept, the strong P); statistics and regions; the view and Inspect; keyframes; live and reset fields; degenerate configs (2 agents, 1 round, mutation 0 and 1).
- **Web:** schema groups, charts, the Compare entry, a sweep over a `norms` base, determinism through the engine (the five `gi-*` presets in the golden list; the `ax-*` presets stop at 100, before its 200 ticks, and `ax-metanorms` is run to its stop and inspected).
- **Browser (controller):** every preset's view and charts, Inspect, the stop, Compare, recording, Experiments, every existing scenario.

## Docs

README: a Norms and Metanorms section (the rules, stated choices, switches and sources, presets and what they reproduce, sweeps); roadmap: Milestone 20 done; `docs/papers.md`: the queue's first item moves to Reproduced.
