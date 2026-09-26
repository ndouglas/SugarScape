# Roadmap: future campaigns

Ideas beyond the current milestone, roughly in order of how much they add. Each item gets its own brainstorm → spec → plan cycle when picked up.

## Milestone 3: Chapter V — disease (done)

Immune and disease bit strings, immune response and transmission (E), metabolic symptoms,
immune-genome inheritance, disease mutation, outbreaks, Infect/Vaccinate tools, a disease
network overlay and the book's presets (V-1 near-eradication, V-2 endemic, the McNeill
outbreak), plus `vi-1-everything`. See
`docs/superpowers/specs/2026-09-23-chapter-v-disease-design.md`.

## Milestone 4: N goods (done)

Goods and pollutants as lists (1–8 and 1–4), n-dimensional welfare, pairwise trade (widest
valuation gap first), per-good credit, per-good maps (two-peak transforms, peaks, flat),
pollution matrices, per-good charts, layers, painting and inspector rows, and the
`n-3-trade`, `n-4-peaks` and `n-2-pollutants` presets. Earlier presets run unchanged except
`vi-1-everything`. See `docs/superpowers/specs/2026-09-23-n-goods-design.md`.

## Milestone 5: Experiments (done)

Parameter sweeps (config values × seeds, each run summarized by one statistic) in the core,
a native `sugarscape` CLI (`presets`, `sweeps`, `run`, `sweep`) and a browser Experiments
view on a Web Worker pool, with measured built-in sweeps for Figures II-5, IV-6 and
IV-10/11 and for carrying capacity vs the number of goods. See
`docs/superpowers/specs/2026-09-23-experiments-design.md`.

## Milestone 6: Model extensions (done)

User-defined tag groups with the book's three-tribe scheme (`iii-6-three-tribes`), a pluggable
bargaining rule (geometric mean or a random price in [MRS_A, MRS_B]) with the measured
`bargaining-rules` sweep, seeded fractal-noise maps and image import for landscapes, agent
trails, and a layered credit-hierarchy tab. Earlier presets run unchanged. See
`docs/superpowers/specs/2026-09-23-model-extensions-design.md`.

## Milestone 7a: Worker simulation (done)

The simulation runs in a Web Worker behind a command/snapshot protocol (with an on-page
fallback), a Max speed runs it flat out, and charts draw downsampled history (LTTB, about
2 000 points per line) while CSV exports keep every tick. Runs are unchanged. See
`docs/superpowers/specs/2026-09-24-worker-simulation-design.md`.

## Milestone 7b: Sessions, comparison and recording (done)

A replayable edit log (share links and session files reproduce a whole session exactly, at any
speed), a side-by-side Compare mode (two worlds in lockstep with per-world rules and overlaid
charts), and recording the grid as WebM or GIF. Runs are unchanged. See
`docs/superpowers/specs/2026-09-24-sessions-compare-recording-design.md`.

## Milestone 8: Chapter VI — indecomposability and the emergent society (done)

The indecomposability presets `vi-2-no-trade` / `vi-3-trade` with Chapter IV's traits and a
presets-menu entry that opens them in Compare: VI-3 follows the book's curve, but VI-2 does not
crash — the book's stated rules do not produce the crash, which most likely depended on unreported
details of the original software. The neighbor, friends and family network overlays, the Lineage
color mode, the age and cultural-tag histograms, per-good wealth histograms and total-wealth Lorenz
curve and Gini (`gini_total`), so `vi-1-everything` offers all eighteen of the book's views. Runs are
unchanged. See `docs/superpowers/specs/2026-09-24-chapter-vi-design.md`.

## Milestone 9: Other artificial societies (done)

Chapter VI's Schelling segregation variant (VI-4 to VI-7, with random acceptable relocation and a
maximum residence) and Ring World (VI-8, VI-9, with its ring view and space–time diagram) as model
kinds beside the sugarscape, on a model-tagged config and a `Model` trait that later models can
reuse: every speed, links, Compare, recording, Experiments (with the measured `schelling-tipping`
sweep) and the CLI. Sugarscape runs, configs and links are unchanged. See
`docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md`.

## Milestone 10: Artificial Anasazi (done)

The Long House Valley, AD 800–1350, as a fourth model kind, written from Janssen's ODD and JASSS
paper with the published replication's ten departures as named switches: its data bundled under
GPL-2.0, the published, default and documented presets (the published one follows JASSS Figure 10;
the documented model does not reproduce it), the valley's views, overlays, year readout and end
year, the historical record on its chart, a replication-vs-documented Compare entry and the
measured `lhv-calibration` and `lhv-quirks` sweeps. Earlier models are unchanged. See
`docs/superpowers/specs/2026-09-25-anasazi-design.md`.

## Playground controls (done)

Step back and a timeline slider (keyframes in the worker, replay of the edit log), stop rules (at a
tick, or when a series crosses a value, on the exact tick at every speed), a measured ticks-per-second
readout, and keyboard shortcuts. Runs are unchanged. See
`docs/superpowers/specs/2026-09-25-playground-controls-design.md`.

## Milestone 11: Civil violence (done)

Epstein's civil violence (2002), Models I and II, as a fifth model kind, with NetLogo Rebellion's five
departures as switches and the paper's runs as presets. The paper's stated arrest rule gives Model I no
rebellion at its own inputs; only NetLogo's rounded-down C/A reproduces its punctuated equilibrium, so the
Model I presets round down and say so (the sweep `cv-ratio-rules` shows it). The salami-tactics, cop-reduction,
coexistence and cleansing results reproduce; the mean wait is a third of the paper's, and Run 8's stable
regime and the safe havens do not reproduce. See `docs/superpowers/specs/2026-09-25-civil-violence-design.md`.

## Milestone 12: Tag-based cooperation (done)

Riolo, Cohen & Axelrod's tag-based donation (Nature 2001) as a sixth model kind, with Edmonds &
Hales' (JASSS 2003) and Roberts & Sherratt's (Nature 2002) departures as switches and the paper's
runs as presets. The paper's tables reproduce only with an unstated tie rule (the current agent
wins); read literally, two pairings give 42 % donation, not 4.3 %. Under any tie rule cooperation
rests on forced donation between identical tags: the strict test, a floor below zero or tag noise
collapse it, and tolerance fixed at zero raises it. The paper's cycle of clusters rising and being
invaded is far slower than it describes. See `docs/superpowers/specs/2026-09-25-tags-design.md`.

## Milestone 13: Spatial games (done)

Nowak and May's spatial Prisoner's Dilemma (1992) as a seventh model kind, with Huberman and Glance's
asynchronous updating (1993) and Nowak, Bonhoeffer and May's probabilistic winning, continuous time, random
arrays and cubes (1994). The kaleidoscope reproduces exactly and the chaotic regime settles at 12 ln 2 − 8 to
three decimals; Huberman and Glance's "always all D" holds only above b = 1.8, a value they never state; and
the random arrays' r_c ≈ 9 depends on an unreported starting mix. See
`docs/superpowers/specs/2026-09-25-spatial-games-design.md`.

## Milestone 14: Axelrod's culture model and its docking (done)

Axelrod's dissemination of culture (1997) as an eighth model kind, with Axtell, Axelrod, Epstein and
Cohen's docking departures (activation, who changes, soup) and Castellano et al.'s and Klemm et al.'s
results as switches and sweeps, and Axelrod's rule as a Sugarscape culture rule running the docking's
mobility experiment. Table 2, the neighborhoods, the territory curve, the torus, the time to stability
and the docking's activation gap reproduce; the sample setup is a little more diverse than reported;
Axelrod's size result holds only below Castellano's transition; and the docked mobility experiment's
near-single culture does not reproduce. See `docs/superpowers/specs/2026-09-25-culture-design.md`.

## Milestone 15: The Emergence of Classes (done)

Axtell, Epstein and Young's bargaining society (2000) as a ninth model kind, with Poza et al.'s
departures (the mode rule, the low demand, growing memories, a lattice) as switches and sweeps. The
error rate, the way to equity and the growth of transition times with memory and population
reproduce; the persistent fractious state and the transition times' magnitude do not; and, as Poza
et al. found, classes never emerge under the paper's rule at its parameters — they do under the mode
rule, and persist once planted. See `docs/superpowers/specs/2026-09-25-classes-design.md`.

## Milestone 16: Ethnocentrism (done)

Hammond and Axelrod's evolution of ethnocentrism (2006) as a tenth model kind, with its appendix's and
its archived code's departures as switches and presets, and the variants of Hartshorn, Kaznatcheev and
Shultz (2013) and Jansson (2013). The standard case and most of Table 1 reproduce within 3 points, and
Hartshorn, Kaznatcheev and Shultz's shares, early patterns and Study 2 orders almost exactly; the appendix's
5 % mutation is a slip, the code draws five colors for four (which Table 1 cannot tell apart), and the
color-blind agents' 14 % cooperation does not reproduce under any reading (41.8 %). Ethnocentrics take over
later than Table 1 l says, and Jansson's kin discriminators win by far less than his Table 5 unless the
kin basis never mutates, which he does not say. See
`docs/superpowers/specs/2026-09-25-ethnocentrism-design.md`.

## Milestone 17: Bounded Confidence (done)

Hegselmann and Krause's opinion dynamics under bounded confidence (2002) as an eleventh model kind, with
symmetric, asymmetric and opinion-dependent confidence as settings and the paper's two unfigured
claims — random serial updating, lattice neighborhoods — as switches. The survivors at small
confidence, the walk from plurality through polarization to consensus, the evenly spaced figures,
the asymmetric drift and the bias's break reproduce; Fig. 2b's two camps are the exception at its
confidence, and the lattice claim holds. See
`docs/superpowers/specs/2026-09-25-bounded-confidence-design.md`.

## Milestone 18: Social Structure (done)

Cohen, Riolo and Axelrod's adaptive agents playing short iterated Prisoner's Dilemmas under six social
structures (2001) as a new model kind, with the paper's substitution dial live and its two readings of
its own method as switches. Table 2, Fig. 1, the crucial p–q region, the partner regression, notes 1
and 5 and Table A1 all reproduce closely; the unstated high-cooperation threshold is recovered as 2.3;
the two starts are equivalent, and of the two noise rules only the Appendix's reproduces Table 2. See
`docs/superpowers/specs/2026-09-26-social-structure-design.md`.

## Experiments and science

- **Parameter sweeps / batch runs**: done (Milestone 5).
- **Headless CLI**: done (Milestone 5).
- **Chapter VI "artificial history" presets**: done (Milestone 8); the flocking / group-formation aside (Animation VI-8) is Ring World (Milestone 9).
- **Artificial Anasazi** (Chapter VI's "Computational Archaeology"): done (Milestone 10).
- **Nowak–May spatial games**: done (Milestone 13).
- **Epstein's civil violence**: done (Milestone 11).
- **Tag-based cooperation** (Riolo, Cohen & Axelrod 2001): done (Milestone 12).
- **Axelrod's culture model and its docking with Sugarscape**: done (Milestone 14).
- **Axtell, Epstein & Young's emergence of classes**: done (Milestone 15).
- **Hammond–Axelrod ethnocentrism** (and Hartshorn, Kaznatcheev & Shultz's and Jansson's critiques): done (Milestone 16).
- **Hegselmann & Krause's bounded confidence**: done (Milestone 17).
- **Cohen, Riolo & Axelrod's social structure**: done (Milestone 18).
- **Credit hierarchy view**: done (Milestone 6).

## Model extensions

- **More than two tribes**: done (Milestone 6).
- **Alternative bargaining rules**: done (Milestone 6).
- **Custom landscape generators**: done (Milestone 6: noise maps and image import).
- **Observational agent trails**: done (Milestone 6).

## Playground and infrastructure

- **Replayable edit log**: done (Milestone 7b).
- **Web Worker simulation**: done (Milestone 7a).
- **Downsampled chart history**: done (Milestone 7a).
- **Side-by-side comparison**: done (Milestone 7b).
- **Recording**: done (Milestone 7b).
- **Step back / timeline, stop rules, shortcuts**: done (Playground controls).
