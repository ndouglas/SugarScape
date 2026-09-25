# SugarScape Milestone 12 — Tag-based cooperation — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–11 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds (`2026-09-25-other-artificial-societies-design.md`) and milestone 11's pattern of literal defaults with named departures (`2026-09-25-civil-violence-design.md`).
**Source text:** Rick L. Riolo, Michael D. Cohen and Robert Axelrod, "Evolution of cooperation without reciprocity", *Nature* 414 (2001), 441–443 (RCA below).
**Replications:** Bruce Edmonds and David Hales, "Replication, Replication and Replication: Some Hard Lessons from Model Alignment", *JASSS* 6(4) 11 (2003) (E&H); Gilbert Roberts and Thomas N. Sherratt, "Does similarity breed cooperation?", *Nature* 418 (2002), 499–500, with RCA's reply on p. 500 (R&S; its change and result as reported by Shutters and Hales, *JASSS* 16(1) 4, 2013).

The milestone number is provisional: Nowak–May spatial games are being built in parallel, and whichever merges second renumbers.

## Goal

Add RCA's tag-based donation model as a sixth model kind, a full citizen of the playground (worker engine, Max speed, timeline, replay and share links, Compare, recording, Experiments, the CLI and the survey), with the paper's runs as presets, the replications' departures as named switches, and the paper's and replications' claims checked over 20 seeds.

## Non-negotiable constraints

- **Earlier models unchanged.** Every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **Faithful to the paper** where it is specific (quoted below); where it is silent, the choice is stated here and in the module docs.
- **One engine path.** Host, engine, replay, Compare, sweeps and CLI stay single-path over the `Model` trait.
- **Deterministic.** A function of (config, seed); each preset's fingerprint pinned by a golden entry.
- **Truthful descriptions.** Each preset's description says what it measurably reproduces and what it does not.

## Source summary

- **Agents:** "Each agent has two traits, a tag τ ∈ [0, 1], and a tolerance threshold T ≥ 0. Initially, tags and tolerance levels are assigned to agents at random, uniformly sampled from [0, 1]." (Other experiments started at T = 0.5 and T = 0.005, "not substantially different".)
- **Pairing:** "In each generation, each agent acts as a potential donor with P others chosen at random, with replacement."
- **Donation:** "A donates only when B's tag is within A's tolerance threshold, T_A, namely when |τ_A − τ_B| ≤ T_A … If A does donate to B, A pays a cost, c, and B receives a benefit, b."
- **Reproduction:** "The least fit, median fit, and most fit agents have respectively 0, 1 and 2 as the expected number of their offspring. This is accomplished by comparing each agent with another randomly chosen agent, and giving an offspring to the one with the higher score." Learning reading: "each agent compares itself to another agent, and adopts the other's tag and tolerance if the other's score is higher than its own."
- **Mutation:** "With probability 0.1, the offspring receives a new tag with a value drawn at random in [0, 1]. Also with probability 0.1, the tolerance is mutated by adding mean 0, standard deviation 0.01 gaussian noise to the old tolerance. If the new T < 0, it is set to 0."
- **Runs:** "100 agents and 30,000 generations. Each experimental condition is replicated 30 times." Defaults P = 3, c = 0.1, b = 1.0: donation rate 73.6 %.
- **Fig. 1** (first 500 generations): donation rate starts ≈ 67 %, falls to 43 % with mean tolerance 0.020 by generation 70, then high with dips at takeovers (generations 226 and 356); mean tolerance 0–0.04.
- **Clusters:** a dominant cluster holds "about 75–80 %" of agents; "the relatedness of a cluster when it first becomes dominant averages 79 %. Ten periods later … 97 %" (relatedness = "the proportion that has its modal tag", first 100 generations excluded); a cluster's mean tolerance is 0.010 at its start and 0.027 at its end.
- **Table 1** (c = 0.1): P = 1, 2, 3, 4, 6, 8, 10 → donation 2.1, 4.3, 73.6, 76.8, 78.7, 79.2, 79.2 %; tolerance 0.009, 0.007, 0.019, 0.021, 0.024, 0.025, 0.024.
- **Table 2** (P = 3): c = 0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6 → donation 73.7, 73.6, 73.6, 73.5, 60.1, 24.7, 2.2 %; tolerance 0.019, 0.019, 0.018, 0.018, 0.011, 0.007, 0.005.
- **Adoption variant:** "adopts the better agent's tag and tolerance with probability proportional to how much better the other agent is. With this method, even one pairing is sufficient to achieve a donation rate of 49 %."

## Replications

- **E&H, tie rule (their Table 6).** RCA do not say who reproduces on equal scores. *No bias*: a coin flip (both E&H re-implementations assumed this). *Selected bias*: the current agent (`score(a) >= score(b)`). *Random bias*: the randomly chosen agent. Only selected bias matches RCA (Table 7, P = 1, 2, 3: no bias 5.1, 42.6, 73.7 %; selected 2.1, 4.4, 73.7 %; random 6.0, 49.6, 73.7 %; Table 8, c = 0.5, 0.6: no bias 45.9, 8.1 %; selected 26.2, 2.1 %; random 46.7, 9.9 %).
- **E&H, strict test (Tables 9–10):** with |τ_A − τ_B| < T_A, donation is 0.0 % at every P and c.
- **E&H, zero tolerance (Tables 11–12):** tolerance fixed at 0: selected bias 0.0 % at every P and c; no bias 3.1, 65.4, 75.3, 78.8 % at P = 1, 2, 3, 6. Their reading: all cooperation is donation between exact tag clones, which selected bias cannot create from all-zero scores.
- **E&H, tag noise (Tables 13–14):** Gaussian noise with s.d. 10⁻⁶ on every offspring's tag: donation 1.5–5.1 % under all three tie rules.
- **E&H, population:** 200 agents: "the donations rates vanished" — they do not say under which rules; we measure it.
- **R&S:** tolerance floor −10⁻⁶ instead of 0, so identical tags need not donate: donation 1.48 %. RCA's reply conceded cooperation holds under "some conditions".

## Finding: the literal rule is not the published model

E&H measured (30 runs × 30 000 generations; to be confirmed by our survey): with ties decided by a coin flip — the natural reading of "giving an offspring to the one with the higher score" — P = 2 gives ≈ 43 % donation, not 4.3 %, and c = 0.5 gives ≈ 46 %, not 24.7 %; only "the current agent wins ties" reproduces the paper's tables. Under every tie rule, cooperation exists only because agents with identical tags must donate to each other (the ≤ test with T ≥ 0): take that away by the strict test, a floor below zero or tag noise, and donation collapses. The mechanism the paper credits — tolerance — does no work: with tolerance fixed at 0, coin-flip ties still give 75 %.

**Decision (user, 2026-09-25):** the config default is the literal reading (coin-flip ties, ≤, floor 0, no noise); presets that reproduce the paper's tables set `tie_rule: current` and say why; every departure is its own switch; built-in sweeps show each dependence; the survey measures all of it over 20 seeds and states what holds.

## Measured in planning

20 seeds × 30 000 generations (the survey reproduces each):

- Table 1 under `current`: P = 1, 2, 3, 10 → 2.1, 2.0 (median), 73.7, 79.3 % (paper 2.1, 4.3, 73.6, 79.2); under `random` P = 2 → 42 %, under `other` 49 % (E&H 42.6, 49.6).
- Table 2 under `current`: c = 0.5 → 26 %, 0.6 → 2.2 % (paper 24.7, 2.2); under `random` c = 0.5 → 45 % (E&H 45.9).
- `below` 1.4 % (E&H report 0.0 %); floor −10⁻⁶ 1.4 % (R&S 1.48 %); tag noise 1.4 % (E&H 1.5–1.9 %); tolerance fixed at 0: 0.0 % under `current`, 75.3 % under `random` (E&H 0.0, 75.3).
- 200 agents under `current`: 3 of 20 runs never cooperate; the rest ≈ 74 %.
- Clusters: a median of 29 takeovers per run (Fig. 1 implies ≈ 120); share 0.86 while dominant (0.75–0.80); relatedness 0.91 at takeover and 0.96 ten generations later (0.79, 0.97); cluster tolerance 0.010 → 0.015 (0.010 → 0.027).
- Adoption at P = 1: 48.8 % scaled by b + c (paper 49 %), 7 % scaled by the score range.

## Architecture

- **Model kind** `tags` ("Tag Cooperation"): `ModelKind::Tags`, `ModelConfig::Tags(TagsConfig)` tagged `"model": "tags"`, a `TagsWorld` implementing `Model`, a schema for the Rules panel, `SERIES`, presets and golden entries — the same wiring as civil violence. Code in `crates/sugarscape-core/src/tags/` (`config.rs`, `world.rs`, `stats.rs` for clusters and takeovers, `presets.rs`, `mod.rs`). The wasm crate's source is unchanged.
- **No space.** The model has no lattice; its main view is a tag × generation diagram drawn through `size()` / `render()` like Ring World's space–time diagram, so the page, Compare and recording need no new view plumbing.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `agents` | 100 | reset | population N (≥ 2) |
| `pairings` | 3 | live | P, donation chances per agent per generation (≥ 0) |
| `cost` | 0.1 | live | c, paid by a donor (≥ 0) |
| `benefit` | 1.0 | live | b, gained by a recipient (≥ 0) |
| `initial_tolerance` | `uniform` | reset | `uniform` (U[0, 1]) or `{ fixed: x }` (x ≥ 0) |
| `tag_mutation` | 0.1 | live | probability an offspring gets a fresh U[0, 1] tag |
| `tolerance_mutation` | 0.1 | live | probability an offspring's tolerance gets Gaussian noise |
| `tolerance_sd` | 0.01 | live | that noise's s.d. |
| `end` | 30 000 | live | generations; `finished()` pauses there (0 = never) |
| `tie_rule` | `random` | live | equal scores: `random` (E&H no bias), `current` (selected bias), `other` (random bias) |
| `donation_test` | `at_most` | live | `at_most` (≤, the paper) or `below` (<, E&H) |
| `tolerance_floor` | 0 | live | mutated tolerance is clamped to ≥ this (R&S: −10⁻⁶) |
| `tag_noise` | 0 | live | s.d. of Gaussian noise added to every offspring's tag, then clamped to [0, 1] (E&H: 10⁻⁶) |
| `selection` | `tournament` | live | `tournament` (the paper) or `adopt` (the paper's proportional variant, scaled by b + c) |

Validation: probabilities in [0, 1]; s.d.s, costs, benefits ≥ 0; `tolerance_floor` ≤ 0; `agents` ≥ 2. E&H's zero-tolerance runs are `initial_tolerance: { fixed: 0 }` with `tolerance_mutation: 0` — no extra switch.

## Setup

N agents with id, tag U[0, 1] and tolerance per `initial_tolerance`; scores 0.

## Step (one generation)

Generation 0 is played at setup: agents get their tags and tolerances, then play step 3 below (pairing), and tick 0 records it. Each later tick:

1. **Selection** builds the next generation all at once from the current generation's scores. A score is `received · b − given · c`, from the generation's counts, so equal counts give bit-identical scores. For each agent a in id order, draw b uniformly from the other N − 1:
   - `tournament`: the higher score gives the offspring; on a tie, `random` flips a coin, `current` picks a, `other` picks b. The offspring copies the winner's tag and tolerance.
   - `adopt`: a takes b's traits with probability (s_b − s_a) / (b + c) when s_b > s_a (capped at 1), else keeps its own. The paper says only "proportional to how much better"; b + c — the most one donation can move two scores apart — reproduces its 49 % at one pairing (planning measured 48.8 %), where the generation's range of scores, the reading this spec first chose, gives 7 %.
2. **Mutation**, independently for each new agent: with `tag_mutation` a fresh U[0, 1) tag; then `tag_noise` (if > 0) adds N(0, s.d.) and clamps to [0, 1]; then with `tolerance_mutation` add N(0, `tolerance_sd`) and raise to at least `tolerance_floor`. Under `adopt`, mutation applies to every agent each generation.
3. **Pairing:** scores reset; for each agent in id order, P times: draw a recipient uniformly from the other N − 1 agents (with replacement across draws; "P others" read as never itself). The donor donates when the test passes (`at_most`: |τ_A − τ_B| ≤ T_A; `below`: < T_A). Count pairings and donations.
4. **Statistics** are recorded for this generation, and the diagram gains a row.

## Statistics

`SERIES`:
- `donation_rate`: donations ÷ pairings this generation (0 when P = 0).
- `mean_tolerance`: over all agents.
- `cluster_share`: the share of agents in the dominant cluster.
- `relatedness`: the share of the dominant cluster holding its modal tag exactly (RCA's measure).
- `cluster_tolerance`: mean tolerance of the dominant cluster.
- `zero_tolerance_share`: the share with tolerance ≤ 0.
- `distinct_tags`: the number of distinct exact tags.
- `takeovers`: cumulative count of changes of dominant cluster.

**Clusters (our reading, stated in the module docs):** the modal tag is the most common exact tag (ties to the lower value); the cluster is every agent whose tag is within 0.01 of it; it is *dominant* when it holds more than half the population. A takeover is counted when a dominant cluster's modal tag is more than 0.01 from the previous dominant cluster's. RCA give no definition beyond "tags so similar that they are within each other's tolerance range"; with a half-width of 0.01, members lie within 0.02 of each other, about a dominant cluster's mean tolerance (0.010–0.027), and 0.01 is the diagram's bin width. With `tag_noise`, exact tags never repeat, so `relatedness` reads ≈ 1/cluster size — itself a finding.

## Views

- **Diagram:** 100 columns (tag bins of width 0.01, the last closed) × 200 rows, newest generation at the bottom, scrolling up. Each row stores its bins' count, mean tolerance, zero-tolerance count, donations given and received, so any row can be inspected; the history travels in keyframes (`copy_without_history` / `restore_into` as Ring World's).
- **Color modes:** **Count** (dark to bright by agents in the bin, on a square-root scale so single agents show); **Tolerance** (mean tolerance of the bin, 0 to 0.05 clamped, empty bins dark); **Clones** (the bin's share of zero-tolerance agents).
- **Inspect:** a row and column: its generation, tag range, count, distinct tags, tolerance min / mean / max, donations given and received; for the newest row also the agents (id, parent, tag, tolerance, score). `locate` returns nothing (agents live one generation) and the page hides Follow. The inspection's `agent` is always null (the host's selection finds nobody to follow).
- **Charts:** Donation rate (Fig. 1a); Tolerance: mean and cluster (Fig. 1b); Clusters: share, relatedness, zero-tolerance share; Distinct tags; Takeovers — against the generation.
- `initial_tolerance` is not on the Rules panel (its `fixed` form is no panel kind).
- **Agents CSV:** id, parent, tag, tolerance, score.

## Presets

Descriptions state the setup and what the survey measured.

| Preset | Setup | Source |
|---|---|---|
| `rca-published` | paper defaults, `tie_rule: current` | Fig. 1, Tables 1–2 |
| `rca-literal` | paper defaults (coin-flip ties) | E&H Table 7a |
| `rca-published-p2` | `rca-published` with P = 2 | Table 1 |
| `rca-literal-p2` | `rca-literal` with P = 2 | E&H Table 7 |
| `rca-strict` | `rca-published` with `donation_test: below` | E&H Tables 9–10 |
| `rs-no-forced-clones` | `rca-published` with `tolerance_floor: -1e-6` | R&S |
| `eh-clones-only` | `rca-literal` with tolerance fixed at 0 | E&H Tables 11–12 |
| `eh-no-exact-clones` | `rca-published` with `tag_noise: 1e-6` | E&H Tables 13–14 |
| `rca-adopt-p1` | `selection: adopt`, P = 1 | RCA p. 442 |

**Compare entry:** "Published vs literal ties at P = 2 — Tag Cooperation (Compare)": `rca-published-p2` and `rca-literal-p2`.

## Experiments and CLI

- `sugarscape presets | run | sweep` accept `tags`; sweep paths and metric series validated against the tags config and `SERIES`.
- Metric for all four: `window_mean` of `donation_rate` over the whole run (RCA average over all 30 000 generations). Generations and seeds are chosen by measurement to fit a browser run and recorded in each description; the survey runs the paper's full length.
- **`rca-pairings`:** x = `pairings` ∈ {1, 2, 3, 4, 6, 8, 10}, series = `tie_rule` — Table 1 and E&H Table 7.
- **`rca-cost`:** x = `cost` ∈ {0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6}, series = `tie_rule` — Table 2 and E&H Table 8.
- **`rca-clones`:** at P = 3, series = `tie_rule`, x = the variants {paper, strict, floor −10⁻⁶, tolerance fixed at 0, tag noise} — the Finding as one chart.
- **`rca-population`:** x = `agents` ∈ {50, 100, 200, 400}, series = `tie_rule` — E&H's population claim, measured.

## Survey

A `tags` claims module in `survey/`, 20 seeds, 30 000 generations. Each judge reports the measured mean ± s.d. beside the published value:
- Tables 1 and 2 under `current` ties: donation and tolerance within measured tolerances of RCA's.
- E&H Tables 7 and 8 under `random` and `other` ties (P = 2 ≈ 43 % and 50 %; c = 0.5 ≈ 46 %).
- `below`, the −10⁻⁶ floor and tag noise each give donation below 6 % under every tie rule.
- Tolerance fixed at 0: ≈ 0 % under `current`, ≥ 65 % at P ≥ 2 under `random`.
- 200 agents: reported for each tie rule.
- Clusters: `cluster_share` 75–80 % while dominant; relatedness at a cluster's first dominant generation ≈ 79 % and ten later ≈ 97 %; `cluster_tolerance` at a cluster's start ≈ 0.010 and end ≈ 0.027 (first 100 generations excluded).
- Fig. 1's early dip: donation ≈ 43 % and mean tolerance ≈ 0.02 around generation 70.
- Adoption at P = 1 ≈ 49 %.

Claims that fail are reported as unreproduced and the descriptions and README say so.

## Page

- The presets menu gains a **Tag Cooperation** group; the Rules panel is generated from the schema in groups Population, Donation, Mutation and Replications (`tie_rule`, `donation_test`, `tolerance_floor`, `tag_noise`, `selection`), each switch's help naming its source.
- Worker host, Max speed, timeline, replay, share/compare links, session files, Compare (same model), recording and Experiments work unchanged; `finished()` pauses at `end`. Editing tools, overlays, trails, Follow and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries and fixtures untouched; new golden entries for every tags preset.
- **Core unit:** the donation test at equality under `at_most` and `below`; zero-tolerance clones donate under ≤ and not under <; each tie rule on hand-set equal scores (`current` with all scores equal reproduces the population exactly before mutation); higher score always wins; recipients never the donor; `adopt` probability at the extremes and with equal scores; tag mutation range, tag noise clamping, tolerance floor clamping; P = 0 gives rate 0; cluster, relatedness and takeover bookkeeping on hand-built populations; diagram binning (tag 1.0 in the last bin) and the 200-row window; keyframe restore keeps the diagram.
- **Config:** tag round trip; validation; `with_path`.
- **Book-style (`#[ignore]`, release):** a shortened Table 1 at P = 2 separating `current` (< 10 %) from `random` (> 30 %).
- **Web (Vitest):** schema panel groups; tags charts; the Compare entry; a sweep over a tags base; Follow hidden.
- **Browser (controller):** each preset renders the three color modes and its charts; Inspect on old and newest rows; Compare; the end pause; recording; Experiments `rca-pairings`; Max-speed performance; every existing scenario.

## Docs

README: a Tag Cooperation section (rules, the stated choices, the switches and their sources, the presets and what they reproduce, the sweeps), crediting RCA, E&H and R&S, and saying plainly that the published tables depend on an unstated tie rule and that cooperation rests on forced donation between identical tags. Roadmap: Milestone 12 done.
