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

## Experiments and science

- **Parameter sweeps / batch runs**: reproduce the book's parametric figures (Fig. II-5 carrying capacity vs. vision and metabolism; Fig. IV-6 carrying capacity with and without trade). A headless runner (native CLI over `sugarscape-core`, or a Web Worker pool in the browser) that runs a grid of configs × seeds and plots summary curves.
- **Headless CLI**: `sugarscape run --preset ii-5-wealth --seed 7 --ticks 1000 --csv out.csv` for scripted experiments and notebooks.
- **Chapter VI "artificial history" presets**: the book's culminating combined rule systems, and its flocking / group-formation asides (Animation VI-8).
- **Credit hierarchy view**: the book's layered lender → borrower tree (Animation IV-5, second half), beyond the spatial overlay.

## Model extensions

- **More than two tribes**: the book's three-group tag scheme (Blue 0–3 zeros, Green 4–7, Red 8–11) and user-defined group rules.
- **Alternative bargaining rules**: the book notes a random price in [MRS_A, MRS_B] gives qualitatively similar results — make the bargaining rule pluggable.
- **Custom landscape generators**: noise and images → capacity maps (per-good two-peak transforms, peaks and flat maps exist).
- **Observational agent trails**: Animation IV-1's "black tail" following one agent's trajectory.

## Playground and infrastructure

- **Replayable edit log**: record hand edits (place/erase agents, paints, mid-run toggles) with their ticks so share links reproduce a whole session, not just its setup.
- **Web Worker simulation**: move the sim off the main thread for large grids and high speeds; transfer frames via `SharedArrayBuffer` or transferables.
- **Downsampled chart history**: cap per-series points (e.g. LTTB) so very long runs stay fast.
- **Side-by-side comparison**: two worlds with different configs/seeds stepped in lockstep, charts overlaid.
- **Recording**: export a run as an animated GIF/WebM of the grid.
