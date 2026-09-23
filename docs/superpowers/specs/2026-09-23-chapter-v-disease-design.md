# SugarScape Milestone 3 — Chapter V: Disease — Design

**Date:** 2026-09-23
**Builds on:** the milestone-1 spec (`2026-09-22-sugarscape-wasm-playground-design.md`) and the Chapter IV spec (`2026-09-22-chapter-iv-sugar-and-spice-design.md`); both remain binding where not changed here.
**Source text:** Epstein & Axtell, *Growing Artificial Societies*, Chapter V ("Disease Processes"), Appendix B (agent immune response, disease transmission), and Chapter VI's culminating "everything on" run.

## Goal

Add the book's disease model — immune-system and disease bit strings, immune response, transmission, metabolic symptoms, genotype/phenotype inheritance — plus a disease-transmission network overlay, novel-disease outbreaks (the McNeill scenario), medicine/vaccination/mutation knobs, and an "everything on" preset combining every rule from Chapters II–V.

## Non-negotiable constraint: earlier runs are unchanged

With disease off, every existing preset (milestone 1 and Chapter IV) must evolve byte-identically. Before any core change, golden fingerprints (200 ticks, seed 1) are recorded for **all** current presets; the Chapter IV ones join `tests/golden.rs`. New RNG draws (disease list, immune genomes, initial infections, mutation, transmission choices) happen only when disease is on; new per-turn steps are skipped when off; `fingerprint()` hashes disease state only when disease is on.

## Approach

Disease state lives **on the agent** (immune genome, trained immune string, current diseases, last infector); the numbered master list of diseases lives on the `World`. A small generic `Bits { bits: u64, len: u32 }` type (1–64 bits) provides get/set, substring windows and Hamming distance; culture `Tags` are unchanged.

## Configuration

New block `disease` (all defaults from the book's Animation V-1):

| Field | Meaning | Default |
|---|---|---|
| `enabled` | rule E (immune response + transmission) | false |
| `count` | size of the initial master disease list | 10 |
| `length: URange` | disease string lengths | 1–10 |
| `initial` | distinct random diseases given to each new agent | 4 |
| `immune_length` | immune string length (≤ 64) | 50 |
| `fee` | extra metabolism per carried disease (both goods) | 1.0 |
| `flips_per_tick` | immune bits flipped per carried disease per tick (medicine) | 1 |
| `genome_mutation` | per-bit mutation probability of a child's immune genome | 0.0 |
| `disease_mutation` | probability that a transmitted disease mutates one random bit | 0.0 |
| `outbreaks: Vec<Outbreak { tick: u64, agents: u32, length: Option<URange> }>` | at `tick`, a brand-new random disease (length drawn from `length` if given, else from `disease.length`) infects `agents` random living agents | empty |

**Validation:** `immune_length` 1–64; `length.min ≥ 1`; `length.max < immune_length`; `initial ≤ count`; `count` 1–1000; `fee` finite ≥ 0; `flips_per_tick ≥ 1`; mutation probabilities in [0, 1]; each outbreak `tick ≥ 1`, `agents ≥ 1`; each outbreak's `length`, when given, `1 ≤ min ≤ max < immune_length`.

**Reset-only (structural mid-run, not schedulable):** `disease.enabled`, `disease.count`, `disease.length`, `disease.immune_length`. `disease.initial` only matters for new agents. `disease.outbreaks` may not be set by a schedule entry (it is its own schedule); it can change mid-run through `set_config` (only future ticks matter).

## Model

- **World:** `diseases: Vec<Bits>` (a disease's id is its index). Generated from the RNG at world creation when disease is on: `count` random strings with lengths drawn from `length`. Outbreaks and mutations append new, distinct strings.
- **Agent:** `immune_genome: Bits` (untrained template, inherited), `immune: Bits` (trained phenotype, starts as a copy), `diseases: Vec<DiseaseId>` (currently carried, distinct), `infected_by: Option<AgentId>` (most recent infector). New agents (initial population, replacements, placed agents) draw a random genome and `initial` distinct random diseases from the current list, skipping any they're already immune to. Newborns inherit a genome (below) and start healthy.
- **Effective metabolism:** `metabolism + fee × diseases.len()` for sugar, and likewise for spice when spice is on. It replaces the base metabolism everywhere metabolism is used: burning, multicommodity welfare, trade MRS/welfare, credit income. (Single-good movement doesn't use metabolism.)
- **Immune response (Appendix B), per carried disease, `flips_per_tick` times per tick:** if the disease is a substring of `immune`, remove it (cured); otherwise find the window of `immune` with the smallest Hamming distance to the disease (leftmost on ties) and flip the first differing bit. After the flips, re-check and remove if now a substring.
- **Transmission (Appendix B):** for each von Neumann neighbor (in random order), pick one of the agent's diseases uniformly; if `disease_mutation` fires, flip one random bit and use (appending if new) the resulting disease instead. The neighbor contracts it unless it already carries it or it's a substring of the neighbor's `immune`. Record `(infector, infected, disease)` for the network and set `infected_by`.
- **Inheritance:** child genome bit = parents' shared value, or a random parent's where they differ (as culture tags), then each bit flips with probability `genome_mutation`. `immune` = copy of the genome.
- **Outbreaks:** applied at the start of `step()` right after scheduled changes, when `outbreak.tick == world.tick`: create a new random disease (length from the outbreak's own `length` if given, else from `disease.length`; redrawn up to 100 times until it differs from every listed disease; if no distinct string is found, the last draw's existing id is reused), choose `min(agents, population)` random living agents, infect each unless immune.

## Tick order (changes in **bold**)

1. Apply scheduled changes; **apply due outbreaks**.
2. For each agent in shuffled order: move → metabolize (effective metabolism) → [credit income] → death check → S → K → T → L-borrow → **E (immune response, then transmission)**.
3. Settle loans; growback; diffusion; replacement; ageing; stats.

## Tools and API

- **Infect tool:** click an agent to infect it with the selected disease from the list, or a new random disease (appended). No effect if it's immune or already carries it.
- **Vaccinate tool (brush):** for every agent within the brush, write the selected disease into its `immune` at the closest window (instant immunity to that disease; may disturb other learned immunities, as in the book), and cure it if carried.
- **WASM additions:** `networks("disease")`; `disease_list(): string` (JSON `[{ id, bits, carriers }]`); `infect(x, y, disease: number)` (−1 = new random); `vaccinate(x, y, radius, disease)`.
- **Inspection:** `AgentView` gains `immune`, `immune_genome` (bit strings), `diseases: [{ id, bits, distance }]` (distance = smallest Hamming distance to `immune`), `infected_by: LinkView | null`. Agents CSV appends `immune,diseases`.

## Statistics (appended to `SERIES`)

`infected_fraction`, `mean_diseases`, `diseases_in_circulation` (distinct diseases carried by anyone), `new_infections` (this tick).

## UI

- **Rules panel:** a Disease group (enable toggle resets the world; count, length range and immune length reset; initial, fee, flips per tick, mutation rates apply live); outbreaks listed read-only beside the schedule ("t = 300 · new disease → 5 agents").
- **Display:** "Disease" color mode (sick red, healthy blue); "Disease network" overlay (infector → infected this tick).
- **Tools:** Infect (disease picker: list entries by id and bits, plus "New disease") and Vaccinate (brush radius + disease picker).
- **Charts:** a Disease group shown when disease is on: infected fraction, mean diseases per agent, diseases in circulation, new infections.
- **Inspector:** immune string vs genome, carried diseases with distances, infected-by link.

## Presets

| id | Rules | Parameters |
|---|---|---|
| `v-1-rid` | ({G₁}, {M, E}) | Chapter II agents; disease on: 10 diseases, lengths 1–10, 4 per agent, immune length 50 (Animation V-1: the immune response drives near-eradication — a residue of ~1–3% persists because learning one disease can overwrite the window that cured another) |
| `v-2-endemic` | ({G₁}, {M, E}) | as `v-1-rid` with 25 diseases, 10 per agent (Animation V-2: endemic disease) |
| `v-mcneill` | ({G₁}, {M, S, E}) + outbreak | `v-1-rid` disease setup with Chapter III demography; outbreak at t = 300 infecting 5 agents with a forced 10-bit `length` so the novel disease reliably takes hold rather than sometimes already being a substring of existing immune strings by chance |
| `vi-1-everything` | ({G₁}, {M, S, I, K, T, L, E}) | spice, sex, lifespan, inheritance, culture, trade, credit and disease together; endowments measured during implementation so the population survives (recorded, as with `iv-18-foresight`) |

## Testing

- **Golden:** Chapter IV presets added to `tests/golden.rs` before the first core change; all golden entries must stay green.
- **Unit:** the book's worked example (immune `1011101001`, disease `10011` → learned in one tick, immune becomes `1001101001`); cure on substring; leftmost window on ties; `flips_per_tick > 1`; transmission skips immune/already-infected neighbors and records the network; effective metabolism with fee; genome crossover and mutation; disease mutation appends distinct variants; outbreak at its tick; infect and vaccinate edits.
- **Property:** with disease on, no agent carries a disease that is a substring of its `immune`, and no duplicates; disease ids are valid indices.
- **Book reproductions (`#[ignore]`, release, measured values in comments):** `v-1-rid` falls to near-zero infected (not exactly zero — see Presets); `v-2-endemic` still has infected agents at t = 1000; `v-mcneill` transmissions (agent-to-agent spread, excluding the outbreak's own seeding) rise after the t = 300 outbreak.
- **Browser:** the controller checks the new group, color mode, overlay, tools, charts, inspector and presets with the puppeteer harness.
