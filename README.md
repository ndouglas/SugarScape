# SugarScape

A browser playground for the Sugarscape model from Joshua M. Epstein and Robert Axtell,
*Growing Artificial Societies: Social Science from the Bottom Up* (1996).

The simulation is written in Rust (`crates/sugarscape-core`), compiled to WebAssembly
(`crates/sugarscape-wasm`), and driven by a small TypeScript front end (`web/`).

## Rules implemented

Chapters II–III of the book: sugar growback (G) and seasons, movement (M), pollution
formation and diffusion (P, D), replacement (R), sexual reproduction (S), inheritance (I),
cultural transmission and tribes (K), and combat (C). Presets reproduce the book's
animations. Where the book is ambiguous, the choice made is documented in the rule's
module (see `crates/sugarscape-core/src/rules/`) and in
`docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md`.

Chapter IV: spice and multicommodity movement, trade (T), credit (L), foresight,
sugar-as-dirty-good pollution, scheduled rule changes, supply and demand, and
trade/credit network overlays. See `docs/roadmap.md` for future work.

Chapter V: immune and disease bit strings, immune response and transmission (E), a
metabolic fee per carried disease, immune-genome inheritance with optional mutation,
disease mutation, outbreaks of novel diseases (the McNeill scenario), Infect and Vaccinate
tools, and a disease-network overlay. `vi-1-everything` runs every rule from Chapters II–V
together.

N goods (the book's own software, Chapter IV footnote 7): 1–8 goods and 1–4 pollutants.
Welfare is the n-dimensional Cobb–Douglas, trade bargains over the pair of goods two
neighbors value most differently, credit lends every good (footnote 55), and pollution
follows Appendix B's matrices (each pollutant forms from the goods gathered and eaten and
devalues the goods it marks). Each good has its own map: a turned copy of the two-peak map,
a set of peaks, or flat. Presets `n-3-trade`, `n-4-peaks` and `n-2-pollutants` show them.

Model extensions:

- **Tag groups (tribes).** The Culture section lists the groups: an agent belongs to the first
  group whose range holds the number of zeros in its tags. The default is the book's two tribes
  (Blue when zeros outnumber ones, else Red); "Three tribes (book)" gives Chapter III note 20's
  Blue 0–3, Green 4–7 and Red 8–11 zeros, and groups can be added, removed, renamed and
  recolored. Combat treats every other group as an enemy, the Tribe color mode uses each group's
  color, and the Group shares chart and `group_share_K` statistics follow them. Preset
  `iii-6-three-tribes` runs culture with three tribes.
- **Bargaining rule.** Trade's Price rule is the book's geometric mean √(MRS_A·MRS_B) or, as
  Chapter IV note 15 suggests, a price drawn uniformly from [MRS_A, MRS_B]. The built-in sweep
  `bargaining-rules` compares their carrying capacities; its description records the measured
  settings and the tolerance within which they agree.
- **Noise maps and image import.** A good's map can be seeded fractal noise (seed, scale in
  cells, octaves, height), which tiles the torus seamlessly and is identical on every platform.
  The paint tool's "Import image…" sets the selected good's capacities from an image's
  brightness (max capacity 0–10, optionally inverted); transparent pixels (alpha < 128) import
  as capacity 0 whether or not Invert is on. Like painted maps, imported maps travel with share
  links and survive resets.
- **Agent trails.** Inspect an agent and press **Follow** to draw its last 500 positions on the
  grid (Animation IV-1's tail), fading with age and broken where it wraps around the torus. The
  toolbar chip stops following. Trails are views only: they never change a run and are not
  exported or shared.
- **Credit hierarchy.** With credit on, the **Credit** tab draws Animation IV-5's lender →
  borrower hierarchy: one row per level (pure lenders on top; loans that close a cycle are
  ignored), lenders green, borrowers red, both yellow. Clicking an agent inspects it. Above 400
  loans only the 400 largest are drawn.

### Notes

- Painted landscapes and share links carry a map for each good that differs from its
  generated one. Share links and configs saved before N goods still load (sugar and spice
  become goods 0 and 1; the pollution coefficients and scheduled paths are converted).
- Two-good runs use floating-point `powf`/`ln`, so results can differ slightly between the
  native (test) build and the browser build. Share links reproduce a run browser to browser.
- The `ii-8-pollution` preset is now scheduled (pollution at t = 50, diffusion at t = 100),
  so older share links to it load as a custom setup.
- Adding or removing a good or pollutant, or changing a good's map, rebuilds the world;
  names, colors, trait ranges and pollution coefficients apply to the running world
  without undoing scheduled changes that have already fired.
- Switching disease on or off, and changing the number of diseases, their lengths or the
  immune-string length, rebuilds the world; the fee, flips per tick ("medicine") and
  mutation rates apply to the running world. Outbreaks are listed in the Schedule section.
- `v-1-rid` reaches near-eradication rather than exactly zero infected: learning one disease
  can overwrite the immune-string window that cured another, so a residue of about 1–3%
  persists.
- The disease fee counts as metabolism everywhere metabolism is used, including consumption
  pollution: sick agents pollute more than healthy ones when pollution is on.
- `vi-1-everything`'s disease flares after each scheduled outbreak (t = 150, 400, 650) and
  tends to die out again before the next one, rather than staying endemic.
- `vi-1-everything` changed with N goods: credit now lends spice as well as sugar.
- Legacy links that set "spice pollutes too" with coefficients other than 1 can differ from
  their old runs in the last bits (α·g₀ + α·g₁ is not always α·(g₀ + g₁) in floating point).
- With custom tag groups, `blue_fraction` is the share of group 0. The corner "Two tribes"
  placement, replacement's same-tribe newcomers, the Place tool's Tribe choice and the agents
  CSV `tribe` column still use the book's two-tribe rule (Blue when zeros outnumber ones).
- Changing the tag length rebuilds the default groups; custom groups are kept and must still
  cover every zero count, or the world is not rebuilt and the Culture section explains why.
- `culture` is reset-only as a whole: a schedule entry that sets `culture` itself (as an
  object) is rejected, same as the goods and pollutant lists. Schedule subpaths instead, e.g.
  `culture.enabled` or a group's `name`/`color`; a whole group or its `zeros` range is
  reset-only too.
- Above 400 loans, a Credit-tab node's colour (lender, borrower or both) reflects all of its
  loans, including those left out of the drawing.

## Experiments

The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
world is paused and kept). A sweep runs every combination of config values × seeds for a
number of ticks and summarizes each run by one statistic: its value at the last tick, its
mean over a window of ticks, or its means over blocks of ticks. The chart shows one line per
value of the second axis: the mean over seeds, with a ±1 sd band; hovering shows mean, sd,
range and the number of runs.

- **Built-in sweeps** (`sweeps/`): `fig-ii-5` (carrying capacity vs vision, one line per
  metabolism), `fig-iv-6` (with and without trade), `fig-iv-10-11` (price dispersion over
  time for short and long lifetimes), `n-goods-carrying-capacity` and `bargaining-rules`
  (carrying capacity vs vision under the two price rules). Each file's description records
  its measured settings; in the browser only seeds and ticks can be changed.
- **From current world**: a config path (the input suggests every number and on/off setting),
  values as `1, 2, 3`, `true, false` or `from:to:step`, an optional second axis, seeds, ticks
  and the metric.
- **Open file…**: a sweep, or a result from the CLI or an earlier export, which is shown
  without running.

Runs are spread over Web Workers (one per core, less one). Cancel stops them and keeps the
partial results, which export marked incomplete. Exports: the result JSON (the CLI's
format), runs and summary CSVs and the chart as PNG. **Share link** copies a `#x=` link that
opens the sweep (not its results).

## Command line

`crates/sugarscape-cli` builds a native `sugarscape` binary over the same core
(`cargo install --path crates/sugarscape-cli`, or `cargo run --release -p sugarscape-cli -- …`):

    sugarscape presets                                  # preset ids
    sugarscape sweeps                                   # built-in sweeps
    sugarscape run --preset ii-5-wealth --seed 7 --ticks 1000 --series-csv series.csv
    sugarscape run --config my-config.json --agents-csv agents.csv --fingerprint
    sugarscape sweep --builtin fig-ii-5 --out fig-ii-5.json --summary-csv fig-ii-5.csv
    sugarscape sweep my-sweep.json --jobs 4 --seeds 3 --ticks 300 --runs-csv runs.csv

`run` defaults to seed 1 and 1000 ticks; `--config-out` writes the config it ran and
`--fingerprint` prints the final world's fingerprint. `sweep` uses every core unless `--jobs`
says otherwise, prints the result JSON unless `--out` is given, and reports progress on
stderr unless `--quiet`. Exit codes: 0 success, 1 I/O error, 2 usage or validation error
(printed as `field: message`, one per line).

A run is a function of its config and seed, so a sweep's output files are byte-identical for
any `--jobs` (and in the browser, for any number of workers). Native and browser builds can
differ in the last bits of `powf`/`ln`, so runs with trade or several goods can give slightly
different numbers in the CLI and in the Experiments view.

## Running locally

Requirements: Rust with the `wasm32-unknown-unknown` target, `wasm-pack`, Node 22+.

    cd web
    npm install
    npm run dev

## Tests

    cargo test                                                      # whole workspace, incl. the CLI
    cargo test -p sugarscape-core                                   # unit + property tests
    cargo test -p sugarscape-core --release --test book -- --ignored # book reproductions
    wasm-pack test --node crates/sugarscape-wasm                    # bindings
    cd web && npm test                                              # front-end logic

The book reproduction tests include a carrying-capacity check that reproduces the book's
claim that the population stabilizes at approximately 224 agents on the default 50×50
map (observed mean 224.8 across 5 seeds).

## Deployment

`.github/workflows/pages.yml` publishes `web/dist` to GitHub Pages after CI passes on `main`.
In the repository settings, Pages must be set to Source: "GitHub Actions".

## Credits

The 50×50 two-peak sugar map is a transcription of the book's Figure II-1 as distributed
with the NetLogo Sugarscape models.
