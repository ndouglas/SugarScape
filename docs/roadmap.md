# Roadmap: future campaigns

Ideas beyond the current milestone, roughly in order of how much they add. Each item gets its own brainstorm → spec → plan cycle when picked up.

## Milestone 3: Chapter V — disease (planned)

Disease and immune-system bit strings, immune response rule, disease transmission between neighbors, and the book's presets (society rids itself of disease / cannot rid itself of disease). Charts: fraction infected, mean immune-disease Hamming distance. Optional "sick network" overlay like the trade/credit networks.

## N-commodity generalization

Replace the explicit sugar + spice fields with a list of N goods, matching the book's own software ("in the Sugarscape software system the number of commodities, n, is a user adjustable parameter"). Welfare becomes the N-dimensional Cobb–Douglas `Π wᵢ^(mᵢ/m_T)`; trade becomes pairwise bargaining over a chosen good pair (or a round-robin over pairs); credit generalizes as in the book's footnote 55. Needs: per-good landscapes (map generators, not just two fixed maps), per-good pollution coefficients (the book's general `p = Δr`, `c = Χm` matrices from Appendix B), per-good charts and color layers. Constraint to keep: with N = 1 or 2 the runs must stay byte-identical to the explicit implementation, so existing presets, share links and book tests keep passing.

## Experiments and science

- **Parameter sweeps / batch runs**: reproduce the book's parametric figures (Fig. II-5 carrying capacity vs. vision and metabolism; Fig. IV-6 carrying capacity with and without trade). A headless runner (native CLI over `sugarscape-core`, or a Web Worker pool in the browser) that runs a grid of configs × seeds and plots summary curves.
- **Headless CLI**: `sugarscape run --preset ii-5-wealth --seed 7 --ticks 1000 --csv out.csv` for scripted experiments and notebooks.
- **Chapter VI "artificial history" presets**: the book's culminating combined rule systems, and its flocking / group-formation asides (Animation VI-8).
- **Credit hierarchy view**: the book's layered lender → borrower tree (Animation IV-5, second half), beyond the spatial overlay.

## Model extensions

- **More than two tribes**: the book's three-group tag scheme (Blue 0–3 zeros, Green 4–7, Red 8–11) and user-defined group rules.
- **Alternative bargaining rules**: the book notes a random price in [MRS_A, MRS_B] gives qualitatively similar results — make the bargaining rule pluggable.
- **Custom landscape generators**: procedural peaks, noise, images → capacity maps; per-good maps once N goods exist.
- **Observational agent trails**: Animation IV-1's "black tail" following one agent's trajectory.

## Playground and infrastructure

- **Replayable edit log**: record hand edits (place/erase agents, paints, mid-run toggles) with their ticks so share links reproduce a whole session, not just its setup.
- **Web Worker simulation**: move the sim off the main thread for large grids and high speeds; transfer frames via `SharedArrayBuffer` or transferables.
- **Downsampled chart history**: cap per-series points (e.g. LTTB) so very long runs stay fast.
- **Side-by-side comparison**: two worlds with different configs/seeds stepped in lockstep, charts overlaid.
- **Recording**: export a run as an animated GIF/WebM of the grid.
