# SugarScape Milestone 14 — Ethnocentrism (Hammond & Axelrod and its critics) — Design

**Date:** 2026-09-25
**Milestone number:** 14 if it lands before the `culture` worktree's model; otherwise renumbered at merge.
**Builds on:** the milestone 1–13 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds and milestone 11's conventions (portable math, named switches for unstated choices, measured descriptions).
**Sources:**
- **HA06:** Hammond & Axelrod, "The Evolution of Ethnocentrism", *J. Conflict Resolution* 50(6) (2006) 926–936 (the user supplied the PDF).
- **HA-Java:** the authors' archived Java/Ascape code (`EthnoModelCode.zip`, package `edu.brook.ethnoadd`, "last updated July 17, 2003", via the Wayback Machine from `umich.edu/~axe/Shared_Files/Axelrod.Hammond/`) and its documentation memo (7/16/03).
- **NetLogo:** Wilensky's Models Library "Ethnocentrism", the "independent validated replication" HA06 cite.
- **SHH08:** Shultz, Hartshorn & Hammond, "Stages in the evolution of ethnocentrism", *CogSci 2008*, 1244–1249.
- **SHK09:** Shultz, Hartshorn & Kaznatcheev, "Why is ethnocentrism more common than humanitarianism?", *CogSci 2009*, 2100–2105.
- **HKS13:** Hartshorn, Kaznatcheev & Shultz, "The Evolutionary Dominance of Ethnocentric Cooperation", *JASSS* 16(3) 7 (2013).
- **J13:** Jansson, "Pitfalls in Spatial Modelling of Ethnocentrism: A Simulation Analysis of the Model of Hammond and Axelrod", *JASSS* 16(3) 2 (2013).

Not available: Hammond & Axelrod 2006, *Theor. Pop. Biol.* 69 (the companion paper), and Kaznatcheev 2010 (both CogSci and AAAI); nothing here relies on them.

## Goal

Add Hammond and Axelrod's ethnocentrism model as an eighth model kind (after the tags model and the spatial games), with the variants HA06 claim in the text and the critics' variants as named switches — a full citizen of the playground (worker engine, Max speed, replay and links, keyframes and the timeline, stop rules, Compare, recording, Experiments, the CLI and the survey), with each source's claims measured, including where the paper, its appendix and its code disagree.

## Non-negotiable constraints

- **Earlier models unchanged.** Every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **Faithful where the sources are specific; stated where they are silent** (every silent point is listed under "Choices the sources leave open").
- **The default is HA06's text**, not its appendix or its code; each disagreement is a named switch or preset.
- **One engine path** over the `Model` trait.
- **Deterministic and portable.** A function of (config, seed); all draws from the world's seeded RNG; fingerprints identical native and WASM.
- **Truthful descriptions**, with measurements, including what does not reproduce.

## Source summary

- **HA06 text:** an empty 50 × 50 torus, four neighbours each. Each period: (1) "An immigrant with random traits enters at a random empty site." (2) "Each agent has its potential to reproduce (PTR) set to 12 percent. Each pair of neighbors then interacts in a one-move prisoner's dilemma in which each chooses (independently) whether to help the other. Giving help has a cost—namely, a decrease in the agent's PTR by 1 percent. Receiving help has a benefit—namely, an increase in the agent's PTR by 3 percent." (3) "Each agent is chosen in a random order and given a chance to reproduce with probability equal to its PTR … into an adjacent empty site, if there is one. An offspring receives the traits of its parent, with a mutation rate of 0.5 percent per trait." (4) "Each agent has a 10 percent chance of dying." Traits: a tag (one of four colours) and two strategy bits (help own colour, help other colours); ethnocentric = help own only.
- **HA06 results:** in the last 100 of ten 2,000-period runs, 76 % ethnocentric (25 % neutral), 74 % of interactions cooperative (Table 1 row a). Table 1 (ethnocentric % / cooperative %, ± s.e. over ten runs): a standard 76.3/74.2; b cost 0.5 % 76.0/77.8; c cost 2 % 61.8/56.1; d colours 2 69.4/78.1; e colours 8 79.1/71.7; f mutation 0.25 % 82.8/79.8; g mutation 1 % 67.1/69.0; h immigration 0.5 77.5/75.5; i immigration 2 74.4/71.4; j 25 × 25 70.5/69.9; k 100 × 100 78.2/76.0; l run length 500 73.9/73.4; m "run length 2,000" 77.3/74.4. Text claims: a full lattice of egoists with no immigration becomes "just as dominant"; distinguishing all four colours gives "80 percent ethnocentric strategies"; with 10 % misperception of same/other colour, "more than two-thirds ethnocentric"; at doubled cost cooperation is 56 %, but "when agents are unable to distinguish their own color from others, cooperation in the doubled-cost case falls to 14 percent". Figure 1 uses mutation 0.25 % (row f).
- **HA06 appendix:** `MutationRate = 0.05`; interaction: "For each adjacent neighboring agent N of each existing agent A: A decides whether to donate to N … N decides whether to donate to A".
- **HA-Java:** `mutationRate = 0.005`; `baseOTR 0.12`, cost 0.01, gain 0.03, death 0.10; `ScapeArray2DVonNeumann`; rules in order update (reset PTR), `PLAY_NEIGHBORS_RULE` (each agent calls `play(neighbour)` for each neighbour; `play` is one-directional: only the caller decides and pays), fission, death, run in `RULE_ORDER`. Fission: into `findRandomAvailableNeighbor()` if any; each strategy bit flips with the mutation rate; a tag mutation redraws `randomInRange(0, numColors)` until it differs from the parent's. Initial tags: `randomInRange(0, numColors)` — Ascape's integer `randomInRange` is **inclusive of both bounds**, so `colorSetting = 4` draws five tags (0–4) and 8 draws nine; the code's comments ("0,3 normal / 0,7 for 8 colors") show the bound was once written inclusively; the colour counters cover tags 0–3 only and `getColor()` returns black for others ("if appears means error"). Two colours use a separate branch (`randomIs()`) and are correct. Noise: with probability `noiseLevel` the agent uses the other strategy bit. `locateChildNearParent` switch (random empty site otherwise). **The archived `iterateScape()` has the immigration block commented out, and `onSetup()` fills all 50 × 50 sites with random agents** — so the archive, run as is, is a full-random-start, no-immigration variant, contrary to its own memo ("First, one or more immigrants are created …"). PTR is never clamped.
- **NetLogo:** mutation 0.005, immigrants per day 1, PTR 0.12, cost 0.01, gain 0.03, death 0.10; immigrate, reset PTR, interact, reproduce, die; newborns can die in the period they are born; PTR not clamped.
- **SHH08/SHK09/HKS13:** the same model (mutation 0.005). HKS13: ethnocentric dominance "at around 300 evolutionary cycles", when the population saturates just under 1,600; final (last 100 of 1,000) shares .08 selfish, .02 traitorous, .73 ethnocentric, .17 humanitarian; of 50 worlds, 16 showed early humanitarian dominance, 16 early ethnocentric, 18 strong early competition. Study 2 (restricted strategy sets; "a new immigrant is created with random in-group and out-group strategy traits. If these traits result in a strategy that is disallowed, that agent is aborted and a new agent is created …"; "If a particular mutation results in an offspring with a disallowed strategy, that mutation is ignored"): the order ethnocentric > humanitarian > selfish > traitorous in every subset except HST, where traitorous beats selfish; Table 3 full-set counts E 1183, H 229, S 123, T 47.
- **J13:** offspring on a random site gives results "similar to the null model" (≈ 12 % cooperators, §3.3, §3.6); relatives (a common founding immigrant) make 71.2 % of neighbour pairs with the same marker; P(ingroup | relative) = 95.3 %, P(relative | ingroup) = 89.2 % (Table 4); 89 % of an ethnocentric's donations go to kin (§4.3); raising the marker's mutation rate alone, altruists pass ethnocentrics at 30 % (§4.4); kin discriminators (a kin marker naming the family's founder, mutating at 0.005, a mutated offspring founding a new family; agents discriminate on either the tag or the kin marker) take 76.2 % against 16.4 % for tag-ethnocentrics (Table 5: none 2.0, outgroup 1.3, ingroup 16.4, nonkin 1.3, kin 76.2, all 2.8); the gap falls below ten points at 36 markers (§5.3).

## Architecture

- **Model kind** `ethno` ("Ethnocentrism"): `ModelKind::Ethno`, `ModelConfig::Ethno(EthnoConfig)` tagged `"model": "ethno"`, `EthnoWorld` implementing `Model`, keyframes (`ModelWorld::checkpoint`/`restore`; the world derives `Clone`), a schema, `SERIES`, presets and golden entries. Code in `crates/sugarscape-core/src/ethno/` (`config.rs`, `world.rs`, `stats.rs`, `presets.rs`, `mod.rs`).
- **Lattice:** the spatial games' square periodic von Neumann neighbour tables (`spatial::geometry`), shared by `Arc`.
- **Randomness:** the world's `SimRng`; `u32` ranges; no platform transcendental functions (none are needed).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `width` | 50 | reset | the torus is `width` × `width` (Table 1 j/k) |
| `colors` | 4 | reset | number of tags, 1–40 (5 = the Java's draw) |
| `start` | `empty` | reset | `empty`, `random` (every site, random traits) or `selfish` (every site, random tags, help no one) |
| `immigration` | 1 | live | immigrants per period: ⌊r⌋, plus one more with probability r − ⌊r⌋ |
| `base_ptr` | 0.12 | live | PTR at the start of each interaction step |
| `cost` | 0.01 | live | PTR the helper loses |
| `benefit` | 0.03 | live | PTR the helped gains |
| `death` | 0.10 | live | per-period death probability |
| `mutation` | 0.005 | live | per strategy trait (and per tag unless `tag_mutation` is set) |
| `tag_mutation` | null | live | the tag's own mutation rate (J13 §4.4); null = `mutation` |
| `pair_play` | `once` | live | `once`: each agent decides once per neighbour per period; `twice`: the appendix's loop read literally, every direction decided twice |
| `discrimination` | `same_other` | reset | `same_other` (two bits), `none` (one bit: help everyone or no one; HA06's "unable to distinguish") or `each_color` (one bit per colour) |
| `misperception` | 0 | live | probability a decision misjudges same/other (`same_other` only) |
| `offspring` | `adjacent` | live | `adjacent` (a random empty neighbour) or `anywhere` (a random empty site) |
| `allowed` | `["E","H","S","T"]` | reset | the strategies immigrants and mutations may produce (`same_other`, no kin strategies) |
| `kin_strategies` | false | reset | adds a basis bit: discriminate on the tag or on the kin marker (J13 §5.2) |
| `kin_basis` | `mutates` | reset | how the basis bit is inherited: `mutates` (a trait mutating at `mutation`) or `fixed` (drawn at immigration, never mutating); J13 does not say (plan Decision 22) |
| `kin_mutation` | 0.005 | live | the kin marker's mutation rate |
| `end` | 2000 | live | the last period (0: never) |
| `schedule` | `[]` | — | as milestone 11's (live paths only) |

Validation rejects on the field: `each_color` with `kin_strategies` or `misperception` > 0; `allowed` other than all four with `discrimination` ≠ `same_other` or with `kin_strategies`; an empty `allowed`; `width` outside 3–200; probabilities outside [0, 1]; negative cost, benefit or base PTR.

## Rules

A period:

1. **Immigration.** For each immigrant (see `immigration`): if an empty site exists, one is chosen uniformly and the immigrant placed there with a uniformly random tag and uniformly random strategy bits (and basis bit), redrawn while its strategy is not in `allowed`; it founds a new lineage and a new family (kin marker). If the lattice is full the immigrant is lost.
2. **Interaction.** Every agent's PTR is set to `base_ptr`. Then, in site order, each agent decides, for each occupied neighbour, whether to help it: its basis (tag, or kin marker with `kin_strategies`) says whether the neighbour is "same" or "other"; with probability `misperception` the judgment is inverted; the agent helps if its bit for that judgment is set (`each_color`: its bit for the neighbour's tag; `none`: its one bit). Helping subtracts `cost` from the helper's PTR and adds `benefit` to the neighbour's. With `pair_play: twice` every agent makes each decision twice (independent misperception draws). One direction per decision; each neighbouring pair thus plays one one-shot game per period (as HA-Java and NetLogo), not two.
3. **Reproduction.** The agents alive at the start of this step, in a uniformly random order (Fisher–Yates): each reproduces with probability PTR (a draw below PTR; PTR is not clamped). An offspring goes to a uniformly chosen empty neighbour (`adjacent`, none → no offspring) or empty site (`anywhere`). It copies its parent, then: each strategy bit flips with probability `mutation` (with `allowed`, a flip that yields a disallowed strategy is ignored; bits in a fixed order); the basis bit (with `kin_strategies`) flips with probability `mutation`; with probability `tag_mutation` (or `mutation`) the tag is redrawn uniformly among the other `colors − 1` tags; with probability `kin_mutation` the offspring founds a new family. Lineage (the founding immigrant) is inherited and never mutates. Offspring do not reproduce in the period they are born.
4. **Death.** In site order every agent, including this period's immigrants and offspring, dies with probability `death`.

Full starts (`random`, `selfish`) fill every site before the first period; each agent founds its own lineage and family; `allowed` applies to `random`.

**Strategies.** With `same_other`: ethnocentric (E: help same only), humanitarian (H: both), selfish (S: neither), traitorous (T: other only). With `kin_strategies` the basis bit distinguishes the tag-based E/T from kin-based "kin" (help kin only) and "nonkin" (help non-kin only); H and S are the same behaviour on either basis and are counted once (J13's "all" and "none"). With `none`: H or S. With `each_color`: E = helps its own colour only, H = all, S = none, T = all but its own, and the rest "mixed".

## Choices the sources leave open

1. **Mutation rate** — HA06's appendix says 0.05, its text and Table 1 (halved 0.25 %, doubled 1 %), HA-Java, NetLogo and every later paper 0.005. Default 0.005; `ha-appendix-mutation` runs 0.05.
2. **Interaction count** — the appendix's loop, read literally, decides each direction twice per period; the code decides once. Default once; `pair_play: twice` and `ha-appendix-double-play`.
3. **Colours** — HA-Java draws five tags for "four" (see above). Default four; `ha-java-five-colors`; the `ha-colors` sweep tests which count fits Table 1.
4. **Start** — HA06: empty with immigration; the archived Java: full random, no immigration. Default empty; `ha-java-archive`.
5. **Immigrants' first period** (silent in HA06): they interact, reproduce and can die in the period they arrive (as NetLogo).
6. **Offspring's first period** (silent): they do not reproduce, and can die (HA-Java and NetLogo agree on death).
7. **Tag mutation** (silent in HA06): always a different tag (HA-Java).
8. **Fractional immigration** (Table 1 h, 0.5): a 50 % chance of one immigrant (HA-Java's `halfImmigrant`).
9. **Misperception** (HA06: "misperceive whether the other agent … has the same color"): per decision, the agent uses its other bit (HA-Java's noise).
10. **Each-colour strategies** (HA06: "distinguish all four colors", no detail): one help bit per colour, each mutating at `mutation`; "ethnocentric" = helps its own colour only.
11. **Blind agents** (HA06: "unable to distinguish"): one help bit, mutating at `mutation`.
12. **Kin strategies** (J13 does not say how the basis is inherited): a basis bit mutating at `mutation` by default; `kin_basis: fixed` draws it at immigration and never mutates it, which fits Table 5 better (kin 65.5 % against 52.1 %; J13 76.2 %) but is not what the sources say either (plan Decision 22).
13. **The run's summary** (HA06, HKS13): the mean over the last 100 periods (1,901–2,000).

## Statistics

`SERIES`: `population`; `ethnocentric`, `humanitarian`, `selfish`, `traitorous`, `kin`, `nonkin`, `mixed` (shares of the population; 0 where the strategy cannot occur); `cooperation` (helps ÷ decisions this period, HA06's "percent cooperative behavior"); `same_tag` (share of decisions toward the same tag); `relatives` (share of neighbouring pairs with a common lineage); `kin_help` (share of helps given to relatives); `tag_given_relative` (P(same tag | related pair)); `relative_given_tag` (P(related | same-tag pair)). Ratios with a zero denominator are NaN.

## Views

- **Colour modes:** **Strategy** (the default: E, H, S, T, kin, nonkin, mixed each a distinct colour), **Tag** (a palette of up to 40), **Lineage** (a colour hashed from the founding immigrant), **PTR** (this period's PTR as heat). Empty sites dark.
- **Inspect:** the agent's tag, strategy and basis; this period's PTR, helps given and received; lineage, kin marker and age; each neighbour's tag, strategy, whether related, and who helped whom. An empty site says so.
- **Charts:** **Strategies** (the strategy shares, in the Strategy mode's colours), **Cooperation** (`cooperation`, `same_tag`), **Population**, **Kin** (`relatives`, `kin_help`, `tag_given_relative`, `relative_given_tag`: J13 Table 4's three numbers and §4.3's).

## Presets

| Preset | Setup | Source |
|---|---|---|
| `ha-standard` | the defaults | HA06 Table 1 a |
| `ha-figure-1` | mutation 0.0025 | HA06 Fig. 1 |
| `ha-appendix-mutation` | mutation 0.05 | HA06 appendix |
| `ha-appendix-double-play` | `pair_play: twice` | HA06 appendix |
| `ha-java-five-colors` | colours 5 | HA-Java's draw |
| `ha-java-archive` | colours 5, `start: random`, immigration 0 | HA-Java as archived |
| `ha-egoist-start` | `start: selfish`, immigration 0 | HA06 text |
| `ha-cost-2` | cost 0.02 | HA06 Table 1 c |
| `ha-cost-2-blind` | cost 0.02, `discrimination: none` | HA06 text (14 %) |
| `ha-misperception` | misperception 0.1 | HA06 text |
| `ha-each-color` | `discrimination: each_color` | HA06 text |
| `jansson-offspring-anywhere` | `offspring: anywhere` | J13 §3.6 |
| `jansson-tag-mutation-30` | tag mutation 0.3 | J13 §4.4 |
| `jansson-kin` | `kin_strategies: true` | J13 §5.2 |
| `jansson-kin-fixed` | `kin_strategies: true`, `kin_basis: fixed` | J13 §5.2, the basis fixed (plan Decision 22) |
| `hks-no-ethnocentrics` | allowed H, S, T | HKS13 Study 2 |

**Compare entries:** "Four colors vs five (the Java's draw) — Ethnocentrism (Compare)" (`ha-standard` vs `ha-java-five-colors`), "Next to the parent vs anywhere — Ethnocentrism (Compare)" (`ha-standard` vs `jansson-offspring-anywhere`), "Tags vs kin — Ethnocentrism (Compare)" (`ha-standard` vs `jansson-kin`).

## Experiments and CLI

Ten seeds, 2,000 periods, metric the window mean over periods 1,901–2,000 unless noted.

- `ha-cost`: x = cost 0.005 … 0.03; series discriminating vs blind; metric `cooperation` (HA06's 56 % vs 14 % at 0.02).
- `ha-colors`: x = colours 2 … 9; metric `ethnocentric` (Table 1 d/a/e against 2/4/8 and 2/5/9).
- `ha-mutation`: x = mutation 0.0025 … 0.05; series `pair_play` once vs twice.
- `ha-immigration`: x = immigration 0.5 … 2.
- `ha-lattice`: x = width 25 … 100.
- `jansson-tag-mutation`: x = tag mutation 0.005 … 0.9; metric `ethnocentric` (the survey reads `humanitarian` from the same runs for the crossing).
- `jansson-markers`: base `jansson-kin`; x = colours 4 … 40; series `kin_basis` mutates vs fixed; metric `kin`.
- `sugarscape presets | run | sweep` accept `ethno`.

## Claims to test (survey and book-style tests; ten seeds as HA06, 50 where HKS13 used 50)

- **HA06:** each Table 1 row, both columns; the egoist start "just as dominant"; each-colour strategies ≈ 80 %; misperception 10 % > two-thirds ethnocentric; cost 2 %: cooperation 56 % discriminating vs 14 % blind.
- **HKS13:** ethnocentric dominance around period 300; final shares .08/.02/.73/.17; early humanitarian dominance in about a third of worlds; Study 2's order E > H > S > T and the HST reversal.
- **J13:** offspring anywhere ≈ the null model; humanitarians pass ethnocentrics at 30 % tag mutation; kin 76.2 % vs ingroup 16.4 %; the gap below ten points at ≥ 36 markers; Table 4's 95.3 % and 89.2 %.
- **Ours:** whether Table 1 d/a/e fit 2/4/8 or 2/5/9 colours; the appendix's 5 % mutation and double play against Table 1 a; the archived Java's full random start against the standard case.

Tolerances come from the measurements (as milestones 11–13).

## Page

- An **Ethnocentrism** presets group; the schema panel in groups **Game** (cost, benefit, base PTR, pair play), **Population** (width, start, immigration, death, offspring), **Traits** (colours, discrimination, misperception shown only for `same_other`, kin strategies, kin basis and kin mutation shown only with kin strategies), **Mutation** (mutation, tag mutation — empty means the mutation rate), **Run** (end); the four colour modes; the four charts, against the **period**; Inspect; the three Compare entries; `defaultForm('ethno')` (x = cost 0.005–0.03 by 0.0025, 2,000 periods, window-mean `ethnocentric` over 1,901–2,000).
- `allowed` is not on the panel (a list is not a panel kind; presets, files and links set it and it round-trips through live edits, resets, links and sessions), so "allowed shown only for `same_other` without kin" has no field to hide; validation still rejects a restricted `allowed` with another discrimination or with kin strategies. The panel shows a nullable number (`tag_mutation`'s null) as an empty box and sends an empty box as null (the schema's `nullable`), and `show_if` compares a bool field as `"true"`/`"false"`.
- Keyframes, the timeline, stop rules, share links, sessions, recording and Compare work unchanged.

## Testing

- **Core unit:** PTR arithmetic on hand-built neighbourhoods (each strategy pair; `twice` doubles; misperception 1 inverts); tag mutation never repeats the parent's tag and is uniform over the rest; `allowed` never violated by immigrants or offspring; kin marker and lineage bookkeeping; immigration (fractional, full lattice); offspring placement (adjacent only into empty neighbours; anywhere); reproduction snapshot (offspring do not reproduce the period they are born); death includes newborns; the statistics on hand-built worlds; each-colour and blind strategy classification; validation errors on the named field.
- **Golden:** entries for every preset; earlier entries untouched; WASM equal to native.
- **Book-style (`#[ignore]`, release):** the claims above, thresholds measured.
- **Web (Vitest):** schema fields and `show_if`; charts; Inspect rows; the Compare entries; `defaultForm('ethno')`; determinism fingerprints.
- **Browser (controller):** every preset, the colour modes, Inspect, the Compare entries, the sweeps, Max speed at 100 × 100.

## Docs

README: an Ethnocentrism section in the civil-violence style — the rules, the choices above, the paper/appendix/code disagreements, what reproduces and what does not, the critics' variants, the Compare entries and sweeps — crediting HA06, the NetLogo replication, SHH08, SHK09, HKS13 and J13. Roadmap: this milestone done; the ethnocentrism entry marked done.
