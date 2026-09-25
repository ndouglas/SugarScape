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

Chapter VI: the indecomposability demonstration and the emergent society's views. Presets
`vi-2-no-trade` and `vi-3-trade` are one society — 500 agents with Chapter IV's traits on the sugar
and spice landscape that move and reproduce — without and with trade, and the presets menu's
**Indecomposability — VI-2 vs VI-3 (Compare)** opens them side by side in Compare at the seed box's
seed. `vi-3-trade` follows the book's VI-3 curve (a dip by t ≈ 100, recovery to about 1.7–2.0 times the
initial population, minima near 700), but `vi-2-no-trade` does the same instead of crashing as the
book's does: every stated rule matches the book, so the crash most likely depended on unreported
details of the original software (see Notes). `vi-1-everything` offers the book's eighteen views
(its description says where each lives). Three more overlays: **Neighbor
network** (Chapter II: each agent → the agents that were its von Neumann neighbors after its last
move, with a direction marker; lists may be one-sided), **Friends network** (Chapter III: each agent →
the up to five culturally closest neighbors it has met, never rechecked; with culture on) and
**Family network** (parent → child; with sex on). The **Lineage** color mode shows Animation III-5's
genealogy: founders grey (the book's black, lightened for the dark grid), founders with children red,
the born green, born parents yellow. Charts gain the **Age histogram** (5-tick bins, while lifetimes
are finite) and **Cultural tags** (the percentage of agents with a 0 at each tag position, while
culture is on). With two or more goods the Goods section adds each good's **Wealth distribution**
and **total wealth** (every good's holdings summed): its **Lorenz curve** and **Gini coefficient**,
also the new `gini_total` statistic (equal to `gini` with one good; the sugar-only `gini`,
`mean_wealth`, Lorenz curve and wealth histogram are unchanged). In Compare the histograms are drawn
as outlines. Neighbor lists, friends and lineage are views only: they never change a run and are not
exported or shared.

Chapter VI's other artificial societies — the book's Schelling segregation variant and Ring World —
run as their own model kinds beside the sugarscape (see [Other artificial societies](#other-artificial-societies)).

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
- The book's VI-2 crash is not reproduced. M, S, T, death and the landscape were checked against the
  text and Appendix B and match; with Chapter IV's traits `vi-3-trade` follows the book's VI-3 curve
  on seeds 1–5 (dip to 100–175 by t ≈ 100–150, peak 1.7–2.0 × 500, minima near 700), and
  `vi-2-no-trade` follows much the same curve (dip to 150–235, peak about 1.8 ×). Of 216
  configurations tried (vision, metabolism, endowment, four fertility tests, trade before sex), none
  made VI-2 die out and VI-3 survive on all of seeds 1–5 except on a knife edge: under these rules
  trade moves holdings toward each agent's metabolism ratio and does not raise fertility. The
  original software most likely had details the book does not report. `presets.rs` records the
  measured populations.
- Total wealth is the sum of an agent's holdings of every good, a reading of VI-1's "total wealth"
  (the book does not define it for two goods). The statistics CSV gains a `gini_total` column after
  `trade_pairs`.

## Other artificial societies

The presets menu groups its presets by model: **Sugarscape**, **Schelling** and **Ring World**.
Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
(Max included), Share, Export, Record, Compare, Experiments and the CLI work the same for every
model. A config without a `model` key is a sugarscape config, so every older config, link, session
file and sweep reads as before. The other models' Rules panels are built from their parameter
schemas (each section says whether its fields rebuild the world or apply as it runs, and live
changes replay from links); they have no editing tools, overlays, trails or Credit tab. Compare
pairs two worlds of one model: choosing another model's preset while comparing leaves Compare
keeping A. See `docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md`.

### Schelling segregation (animations VI-4 to VI-7)

The book's variant of Schelling's model: 2 000 Red and Blue agents on a 50 × 50 torus, each
wanting at least a share of its von Neumann neighbors to be its own color (an agent with no
neighbors is satisfied). Agents act in random order; an unsatisfied one moves to a site chosen at
random among every empty site where it would be satisfied (its own site not counted as a
neighbor), or stays. With a maximum residence an agent leaves when it reaches it, and a newcomer of
random color takes a random site where it is satisfied. Segregation is the mean share of like
neighbors (over agents with neighbors).

- `vi-4-schelling-25`: 25 %. Nobody moves after 2–3 ticks; segregation rises from about 0.50 to 0.63.
- `vi-5-schelling-25-residence`: 25 %, residence 80–100 ticks. It never settles; segregation
  climbs to about 0.76, above VI-4's (the book calls the two comparable).
- `vi-6-schelling-50-residence`: 50 %: about 0.95.
- `vi-7-schelling-mixed`: preferences 25–50 %: about 0.93, close to VI-6.

Agents are drawn by **Colour**, **Satisfaction** (the unsatisfied in yellow) or **Preference**;
the charts are Segregation, Unsatisfied, Moves and Red share; Inspect shows an agent's color,
preference, alike neighbors and residence. The built-in sweep `schelling-tipping` asks the book's
"how little racism is enough to tip a society": with every agent wanting the same share, from 0 to
60 %, segregation rises in steps (about 0.50, 0.63, 0.73, 0.83 and 0.93), because with four
neighbors only the shares 1/4, 1/3, 1/2 and 2/3 can matter.

### Ring World (animations VI-8 and VI-9)

40 sugar harvesters with vision 15–30 on a ring of 150 sites (sugar 0–4, growing back 1 a tick)
look only counterclockwise, move to the nearest richest empty site they see and eat it. Started
scattered (`vi-8-ring-world`) they fall from 20–25 flocks of about 2 into 7–8 flocks of 5–6 that
tread around the ring; started as one group of 40 (`vi-9-ring-megagroup`) they break up into as
many. A flock is a run of agents at most one empty site apart (the book does not define one). The
page draws the ring — sugar shaded, agents as blue dots, site 0 at the top — above a space–time
diagram of the last 150 ticks (the current tick at the bottom), which the simulation keeps, so every
tick shows even at Max. Capacity and growback apply to the running world. Charts: Flocks, Flock
size (mean and largest) and Distance moved; recordings show the ring beside the diagram.

## Experiments

The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
world is paused and kept). A sweep runs every combination of config values × seeds for a
number of ticks and summarizes each run by one statistic: its value at the last tick, its
mean over a window of ticks, or its means over blocks of ticks. The chart shows one line per
value of the second axis: the mean over seeds, with a ±1 sd band; hovering shows mean, sd,
range and the number of runs.

- **Built-in sweeps** (`sweeps/`): `fig-ii-5` (carrying capacity vs vision, one line per
  metabolism), `fig-iv-6` (with and without trade), `fig-iv-10-11` (price dispersion over
  time for short and long lifetimes), `n-goods-carrying-capacity`, `bargaining-rules`
  (carrying capacity vs vision under the two price rules) and `schelling-tipping` (segregation vs
  a fixed Schelling preference). Each file's description records its measured settings; in the
  browser only seeds and ticks can be changed.
- **From current world**: a config path (the input suggests every number and on/off setting),
  values as `1, 2, 3`, `true, false` or `from:to:step`, an optional second axis, seeds, ticks
  and the metric. The path suggestions, statistics and starting axis follow the current world's
  model.
- **Open file…**: a sweep, or a result from the CLI or an earlier export, which is shown
  without running.

Runs are spread over Web Workers (one per core, less one). Cancel stops them and keeps the
partial results, which export marked incomplete. Exports: the result JSON (the CLI's
format), runs and summary CSVs and the chart as PNG. **Share link** copies a `#x=` link that
opens the sweep (not its results).

## How the playground runs

The simulation runs in a Web Worker: the page sends it commands (steps, edits, rule changes)
and draws the frame and statistics it sends back, so the page stays responsive on large grids
and at high speeds. Where a module worker cannot start, the same code runs on the page. The
speed menu's **Max** runs the simulation as fast as it goes and redraws about 30 times a
second; the other speeds step a fixed number of ticks per frame, as before. A run does not
depend on the speed: the same setup and seed give the same world at the same tick. The
page's worlds stop at 1 000 000 ticks, whatever the model, so the whole history fits in memory:
the world pauses there and says so (export its data, or Reset). The command-line tool has no
such limit.

Charts draw a downsampled history, with the tick on the x axis: Largest-Triangle-Three-Buckets
keeps about 2 000 points of each line, so spikes survive on long runs, and so do gaps
(stretches with no value, such as no trades) longer than a bucket, about 1/2 000 of the run;
shorter gaps are bridged. The full per-tick history stays with the simulation: Export →
Statistics (CSV), share links and Experiments use all of it.

## Sessions and share links

The playground records every edit you make — painting, image imports, placing and erasing
agents, infections, vaccinations and live rule changes — with the tick it happened at. **Share →
Copy link** carries the whole session: the setup, the painted maps it started from and that
edit log. Opening the link rebuilds the world and replays each edit at its tick as the world
runs (a chip counts down the edits left), so it reaches exactly the same world at the same tick,
at any speed. Editing during a replay starts a new branch from there; the chip's ✕ ends the
replay and keeps the world. **Reset** with the same seed rewinds and replays the session; a new
seed, 🎲, a preset or a rule change that needs a reset starts a new session. Very long sessions
still make a link (the page says when it is long); **Export → Session (JSON)** saves the same
content as a file and **Share → Open session…** loads it. After 50 000 edits recording stops and
links carry the setup and painted maps only. Links from earlier versions still open.

## Compare

**Compare** runs a copy of the current world beside it, each in its own worker: B starts as an
exact copy of A at the current tick, and both step in lockstep (Play, Step, the speeds and Max
act on both; ticks always match). Each grid has its own seed and 🎲; the Rules tab's **Rules
for: A | B** switch applies changes to one world, tools act on the grid you click (an infection
or vaccination press on the other grid switches the disease picker to that world without acting;
the next press does), and Inspect and Credit show the world you clicked last. While B is being
copied from A, A is locked against edits. A rebuilt world (🎲, a preset, a reset-requiring
change) rewinds the other to t = 0 so the two stay comparable. Charts overlay A (solid) and B
(dashed). Exports ask which world; Share makes a link that opens straight into Compare. Leaving
asks which world to keep.

## Recording

**● Record** records the grid as drawn (overlays, trails, selection) as WebM video or an
animated GIF, optionally stamped with the tick. Cells are 8 px (smaller for grids over 135
cells, keeping the frame within 1080 px, with even width and height); in Compare both grids are
recorded side by side. Ring World records its ring beside its space–time diagram. Recording pauses
while the world is paused. GIFs are sampled at about 15
frames a second, encoded off the page in a worker, and stop — with a notice — at 900 frames or
if encoding fails. Files are named after the setup and the ticks they cover, e.g.
`sugarscape-ii-2-unit-seed7-t0-t800.webm`.

## Command line

`crates/sugarscape-cli` builds a native `sugarscape` binary over the same core
(`cargo install --path crates/sugarscape-cli`, or `cargo run --release -p sugarscape-cli -- …`):

    sugarscape presets                                  # preset ids
    sugarscape sweeps                                   # built-in sweeps
    sugarscape run --preset ii-5-wealth --seed 7 --ticks 1000 --series-csv series.csv
    sugarscape run --config my-config.json --agents-csv agents.csv --fingerprint
    sugarscape run --preset vi-8-ring-world --ticks 500 --series-csv ring.csv       # any model
    sugarscape sweep --builtin fig-ii-5 --out fig-ii-5.json --summary-csv fig-ii-5.csv
    sugarscape sweep my-sweep.json --jobs 4 --seeds 3 --ticks 300 --runs-csv runs.csv

`run` runs a preset or config of any model; it defaults to seed 1 and 1000 ticks; `--config-out` writes the config it ran and
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
    cd web && npm run build && npm test                             # front-end logic (build first: the engine tests load the real WASM package)

The book reproduction tests include a carrying-capacity check that reproduces the book's
claim that the population stabilizes at approximately 224 agents on the default 50×50
map (observed mean 224.8 across 5 seeds).

## Deployment

`.github/workflows/pages.yml` publishes `web/dist` to GitHub Pages after CI passes on `main`.
In the repository settings, Pages must be set to Source: "GitHub Actions".

## Credits

The 50×50 two-peak sugar map is a transcription of the book's Figure II-1 as distributed
with the NetLogo Sugarscape models.
