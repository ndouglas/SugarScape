# SugarScape Milestone 15 — The Emergence of Classes — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–14 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds and the literal-default-plus-named-switch pattern of milestones 11–14.
**Source text:** Robert Axtell, Joshua M. Epstein and H. Peyton Young, "The Emergence of Classes in a Multi-Agent Bargaining Model", Center on Social and Economic Dynamics Working Paper No. 9 (February 2000); published in S. Durlauf and H. P. Young (eds.), *Social Dynamics* (MIT Press, 2001) (AEY below).
**Replication:** David J. Poza, Félix A. Villafáñez, Javier Pajares, Adolfo López-Paredes and Cesáreo Hernández, "New Insights on the Emergence of Classes Model", *Discrete Dynamics in Nature and Society* 2011, 915279 (PVPLH below). Related: Poza, Santos, Galán and López-Paredes, "Mesoscopic Effects in an Agent-Based Bargaining Model in Regular Lattices", *PLoS ONE* 6(3) e17661 (2011); Weisbuch, "Persistence of discrimination: revisiting Axtell, Epstein and Young", *Physica A* (2017).

The milestone number is provisional: another session is building ethnocentrism in parallel, and whichever merges second renumbers.

## Goal

AEY's bargaining model — agents who remember their last m opponents' demands and best-reply to them, with and without a meaningless tag — as a ninth model kind, `classes`, a full citizen of the playground, with the paper's figures as presets and sweeps, PVPLH's departures as named switches, and the claims measured over 20 seeds.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical).
- **Truthful descriptions:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **The game (Nash demand):** demands L = 30, M = 50, H = 70 percent of a pie; "each party gets his demand if the sum of the two demands is not more than 100 percent of the pie, otherwise each gets nothing." Pure equilibria (L, H), (M, M), (H, L).
- **One agent type:** "Let the population consist of N agents. Each time period consists of N/2 'matches.' In each match, one pair of agents is drawn at random from the population"; "Some agents may be active more than once in a particular period, while others are inactive." "Every agent remembers the demands … played by each of her last m opponents"; she takes the relative frequencies as the opponent's probabilities. "With probability 1 − ε an agent makes a demand that maximizes her expected payoff … If several demands maximize expected payoff, they are chosen with equal probability. With probability ε the agent … chooses one of the three demands, H, M, or L, at random." Footnote 9: the realized error rate is ε·2/3 ("0.1333…" at ε = 0.2).
- **Simplex:** agents plotted by memory content (L at lower right, M at lower left, H at top), shaded by best reply.
- **Fig. 2:** N = 100, m = 10, ε = 0.2, "the initial state is random": equity after 80 periods. **Fig. 3:** a fractious state after 150 periods (another start), persisting "in excess of 10⁹ time periods", mean payoff "only about one-quarter". "M is never a best response for someone who has never experienced an opponent who played M."
- **Transitions:** from "the fractious regime with N = 10, ε = 0.10" to equity, defined as "a state where all agents have at least (1 − ε)m instances of M in their memories": exponential in m ("when m = 13 it takes in excess of 10⁵ periods"; "for m = 18 … O(10⁶)") (Fig. 4, ε = 5 %, 10 %); exponential in N at m = 10 (Fig. 5, ε = 2, 5, 10 %).
- **Tags:** "each agent records in his memory the tag of his opponent and the demand that he made. Faced with a new dark opponent, the agent demands an amount that maximizes the expected payoff against his remembered distribution of dark opponents." Footnote 15: "In the event that an agent has no memory of Blue opponents it picks a random strategy." "100 agents in total, 50 of each type … memory length 20 and the noise level ε = 0.2"; from random starts, Figs. 6–9 at t = 150–260 show equity within and between types, equity between but not within, classes (equity within, discrimination between: darks 70, lights 30), and "equity above, division below". Transitions out of classes are "very rare" even at 10 agents per type, m = 10, ε = 0.1 — not measured.

## Replication (PVPLH)

- They describe AEY's tagged agents as keeping "two memory sets depending on the opponent's tag", and contacted Axtell to confirm the decision rule; the transition-time result reproduced (their Fig. 3). They started transition runs by "forcing" memories into a fractious state (details not given).
- **Tags:** "when we tried the same parameters that AEY used in their simulation (100 agents, memory size = 20), segregation never emerged … We needed to reduce the number of agents and the memory length" — N = 20 (10 per type), m = 5, ε = 0.05 (their Figs. 9–10).
- **Mode rule:** best reply to the most frequent demand in memory; "segregation emerged spontaneously much more often"; fewer runs reach equity first (their Figs. 4–5: share of runs whose first attractor is fractious, against N and m, ε = 0.2).
- **Payoffs:** L from 5 to 45 with H = 100 − L (Table 1): "the higher the reward assigned to low, the longer it took … to reach the equitable equilibrium" (their Fig. 7, ε = 0.1, m = 10).
- **Progressive memory:** memories start empty and grow to m; first demands random; longer time to equity (their Fig. 8, ε = 0.1, m = 12), same long run.
- **Lattice:** 10 × 10 torus, Moore neighbors, 50 of each tag laid out at random, in four zones or in two zones; the well-mixed attractors recur; with two zones, new border equilibria appear.

## Architecture

Model kind `classes` ("Emergence of Classes"): `ModelKind::Classes`, `ModelConfig::Classes(ClassesConfig)` tagged `"model": "classes"`, a `ClassesWorld` implementing `Model`, schema, `SERIES`, presets and golden entries — the same wiring as the culture model. Code in `crates/sugarscape-core/src/classes/` (`config.rs`, `world.rs`, `stats.rs` for regimes, `simplex.rs` for the view, `presets.rs`, `mod.rs`).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `agents` | 100 | reset | N (2–1000; even); with `lattice`, width × height |
| `memory` | 10 | reset | m (1–100) |
| `noise` | 0.2 | live | ε |
| `tags` | false | reset | two types, half each (dark: ids 0..N/2 unless laid out) |
| `tag_memory` | `per_tag` | reset | `per_tag` (a memory of length m for each tag; PVPLH's reading of AEY) or `shared` (one memory of the last m opponents with their tags) |
| `decision` | `expected` | live | `expected` (AEY: maximize expected payoff) or `mode` (PVPLH: best reply to the most frequent demand; ties uniformly) |
| `low` | 30 | live | the L demand; H = 100 − L; M = 50 (PVPLH Table 1: 5–45) |
| `start` | `random` | reset | `random` (each memory slot uniform over L, M, H), `fractious` (each memory ⌈m/2⌉ H and ⌊m/2⌋ L, shuffled), `progressive` (memories start empty and grow; PVPLH), `classes` (tags: within-type memories all M; darks' memories of lights all L, lights' of darks all H) |
| `interaction` | `random` | reset | `random` (AEY) or `lattice` (PVPLH: a torus) |
| `lattice.width`, `lattice.height` | 10, 10 | reset | with `lattice`; `agents` = width × height |
| `lattice.neighborhood` | `moore` | reset | `moore` or `von_neumann` |
| `lattice.layout` | `random` | reset | tag layout: `random`, `four_zones` (quadrants), `two_zones` (halves) |
| `stop_at_equity` | false | live | `finished()` at the first equity tick |

## Step (one period)

N/2 matches. A match: with `random`, two distinct agents uniformly; with `lattice`, a uniform agent and a uniform neighbor. Each agent demands (below), each is paid its demand if the demands sum to at most 100, else 0, and each appends the opponent's demand (and, with tags, to the memory for the opponent's tag) and drops its oldest entry once the memory holds m.

**A demand** against an opponent of tag t: the memory is the agent's memory (one type or `shared`: the entries whose tag is t; `per_tag`: the memory for t). With probability ε, uniform over L, M, H. Otherwise, if that memory is empty, uniform (AEY footnote 15; PVPLH's first progressive match). Else with `expected`: the demand maximizing expected payoff under the memory's frequencies (L: L; M: 50·(1 − p_H); H: H·p_L), ties uniform; with `mode`: the best reply to the most frequent demand (L → H, M → M, H → L), ties among modes uniform.

## Regimes

For a set S of memories (all memories in one type; with tags the intra- and inter-type memories per tag), our reading of AEY's pictures:
- **equity:** every memory in S holds at least (1 − ε)·(its length) M's (AEY's transition target, per memory);
- **fractious:** no memory in S has M as a best reply (every agent is in the L or H region);
- **aggressive / submissive:** every memory in S best-replies H / L.

The world's regime: one type — `equity`, `fractious` or `mixed`; tags — `equity` (all three contexts equity), `classes` (both intra equity, inter: one tag all aggressive, the other all submissive), `equity-between` (inter equity, an intra not), `divided-below` (inter discriminatory, one intra equity and the other fractious), `fractious`, `mixed`.

## Statistics

`SERIES`: `mean_payoff` (per agent per match this period), `m_share` (M's among all memory entries), `outcome_mm`, `outcome_hl` (H against L), `outcome_fail` (demands summing over 100), `outcome_waste` (under 100: LL, LM), `regime` (a code: 0 mixed, 1 equity, 2 fractious, 3 classes, 4 equity-between, 5 divided-below), `equity_at` (the first equity tick, else the current tick), `first_attractor` (0 none yet, 1 equity, 2 fractious — PVPLH Figs. 4–5), and with tags `payoff_dark`, `payoff_light`, `payoff_inter` (dark's minus light's mean payoff in inter matches).

## Views

- **Simplexes** rasterized on the grid canvas: one triangle (no tags) or two side by side (intra left, inter right), 121 cells wide each; vertices H (top), M (lower left), L (lower right) as in AEY; background shaded by the best reply at each point under the current `low` and `decision` (L region, M region, H region, three grays); agents drawn at their memory's barycenter (dark and light tags in two colors; brighter where several coincide).
- **Color modes:** **Best reply** (the shading above) and **Payoff** (agents colored by their last period's mean payoff).
- **Inspect:** a point lists the agents there (id, tag, memory counts, last demand and payoff); `locate` returns nothing; Follow hidden.
- **Charts:** Mean payoff; Outcomes (MM, H–L, failures, waste); M in memory; Regime; with tags Payoffs by tag and Inter-type advantage. Time axis: Periods.

## Presets

| Preset | Setup | Source |
|---|---|---|
| `aey-equity` | N 100, m 10, ε 0.2, random start | Fig. 2 |
| `aey-fractious` | the same, fractious start | Fig. 3 |
| `aey-transition` | N 10, m 10, ε 0.1, fractious start, stop at equity | Figs. 4–5 |
| `aey-tags` | N 100, m 20, ε 0.2, tags | Figs. 6–9 |
| `aey-classes` | `aey-tags` with a classes start | the unrun experiment |
| `pvplh-small-tags` | N 20, m 5, ε 0.05, tags | PVPLH Figs. 9–10 |
| `pvplh-mode` | `aey-tags` with the mode rule | PVPLH §3.2, §4 |
| `pvplh-progressive` | N 20, m 12, ε 0.1, progressive start | PVPLH Fig. 8 |
| `pvplh-lattice` | 10 × 10 Moore torus, tags in two zones, m 5, ε 0.05, mode rule | PVPLH §5 |

**Compare entry:** "AEY's rule vs the mode rule, with tags — Emergence of Classes (Compare)": `aey-tags` and `pvplh-mode`.

## Experiments and CLI

Tick caps and seeds measured to fit a browser run and recorded in each description; the survey runs longer.
- `aey-memory`: final `equity_at` from a fractious start, N 10, against m (6–18), series ε 0.05 / 0.1 (Fig. 4).
- `aey-population`: the same against N (10–100) at m 10, series ε 0.02 / 0.05 / 0.1 (Fig. 5).
- `aey-first-attractor`: final `first_attractor` share against N and m, series `expected` / `mode` (PVPLH Figs. 4–5).
- `aey-tag-regimes`: final `regime` against N and m with tags, series `per_tag` / `shared` — how often classes appear.
- `pvplh-payoffs`: `equity_at` against `low` (5–45) and N (PVPLH Fig. 7).

## Survey

A `classes` claims module, 20 seeds (30 where frequencies are small):
- AEY: the realized error rate is 2ε/3; M is never a best reply without M in memory (a property test); Fig. 2's random start reaches equity (and how soon); Fig. 3's fractious regime has a mean payoff near a quarter; Figs. 4–5's transition times grow with m and N, with magnitudes checked where runs allow; with tags at N 100, m 20, ε 0.2 classes emerge — how often (PVPLH: never); classes persist (measured).
- PVPLH: at N 20, m 5, ε 0.05 segregation appears; the mode rule makes it more frequent and makes fractious-first more likely; a higher L lengthens the transition; progressive memory lengthens it; a lattice reaches the same attractors.

Claims that fail are reported, and the descriptions and README say so.

## Page

The presets menu gains an **Emergence of Classes** group and the Compare entry; the Rules panel is generated from the schema in groups Population, Bargaining, Start, Lattice and Departures (each departure's help naming its source). Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged; `finished()` pauses at equity when asked. Editing tools, overlays, trails, Follow and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `classes` preset.
- **Core unit:** the payoff rule (sums ≤ 100) for every `low`; expected-payoff best replies at the region borders and their ties; the mode rule and its ties; M never best-replies without M; memory append/drop, per-tag and shared lookups, empty memories; every start; random and lattice matching (no self-match; neighbors on the torus); regime classification on hand-built memories; `equity_at`, `first_attractor`, stop at equity; simplex rasterization (vertices, shading, dot placement) and Inspect; keyframes; live and reset fields; degenerate configs (N = 2, m = 1, ε 0 and 1, `low` 5 and 45).
- **Web:** schema groups, charts, the Compare entry, a sweep over a `classes` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, the stop, Compare, recording, Experiments, every existing scenario.

## Docs

README: an Emergence of Classes section (rules, stated choices, switches and sources, presets and what they reproduce, sweeps); roadmap: Milestone 15 done.
