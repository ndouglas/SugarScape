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

## Experiments and science

- **Parameter sweeps / batch runs**: done (Milestone 5).
- **Headless CLI**: done (Milestone 5).
- **Chapter VI "artificial history" presets**: done (Milestone 8); the flocking / group-formation aside (Animation VI-8) is Ring World (Milestone 9).
- **Artificial Anasazi** (Chapter VI's "Computational Archaeology"): done (Milestone 10).
- **Nowak–May spatial games**: a candidate model kind (spatial Prisoner's Dilemma on a lattice).
- **Epstein's civil violence**: done (Milestone 11).
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
