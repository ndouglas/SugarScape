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
initial population, minima near 750), but `vi-2-no-trade` does the same instead of crashing as the
book's does: every stated rule matches the book, so the crash most likely depended on unreported
details of the original software (see Notes). `vi-1-everything` offers the book's eighteen views
(its description says where each lives). Three more overlays: **Neighbor
network** (Chapter II: each agent → the agents that were its von Neumann neighbors after its last
move, with a direction marker; lists may be one-sided), **Friends network** (Chapter III: each agent →
the up to five culturally closest neighbors it has met, never rechecked; with culture on) and
**Family network** (parent → child; with sex on). The **Lineage** color mode shows Animation III-5's
genealogy: founders gray (the book's black, lightened for the dark grid), founders with children red,
the born green, born parents yellow. Charts gain the **Age histogram** (5-tick bins, while lifetimes
are finite) and **Cultural tags** (the percentage of agents with a 0 at each tag position, while
culture is on). With two or more goods the Goods section adds each good's **Wealth distribution**
and **total wealth** (every good's holdings summed): its **Lorenz curve** and **Gini coefficient**,
also the new `gini_total` statistic (equal to `gini` with one good; the sugar-only `gini`,
`mean_wealth`, Lorenz curve and wealth histogram are unchanged). In Compare the histograms are drawn
as outlines. Neighbor lists, friends and lineage are views only: they never change a run and are not
exported or shared.

Chapter VI's other artificial societies — the book's Schelling segregation variant and Ring World —
and Artificial Anasazi, the Long House Valley model that Chapter VI's "Computational Archaeology"
anticipates, run as their own model kinds beside the sugarscape (see
[Other artificial societies](#other-artificial-societies)).

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
- Above 400 loans, a Credit-tab node's color (lender, borrower or both) reflects all of its
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

The presets menu groups its presets by model: **Sugarscape**, **Schelling**, **Ring World**,
**Artificial Anasazi**, **Civil Violence** and **Tag Cooperation**.
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

Agents are drawn by **Color**, **Satisfaction** (the unsatisfied in yellow) or **Preference**;
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

### Artificial Anasazi: the Long House Valley, AD 800–1350

Households of five farm maize in the Long House Valley in northeastern Arizona on an 80 × 120
grid of hectare cells, one tick a year from AD 800 to 1350, against the archaeological record of
how many households lived there. Each year a household harvests its plot's base yield (the zone's
yield for that year's Palmer drought index × its soil quality × the harvest adjustment) with some
noise, stores corn for up to two years and eats the oldest first; one that cannot eat its 800 kg or
is too old is removed; one that expects too little next year moves to the nearest free plot that can
feed it (and a home nearby, closest to water), or leaves the valley; and a household of
childbearing age splits off a new one with some probability. The model is written from Janssen's
ODD of his NetLogo replication (2013) and "Understanding Artificial Anasazi" (*JASSS* 12(4) 13,
2009), which replicates Axtell et al. (2002) only approximately — so these presets reproduce
Janssen's replication, not the original model.

The written description leaves several things undefined, which are taken from the replication in
every preset: which cells are water and when, the hydrology that keeps homes off the valley floors,
which drought series each zone follows (the Dunes always yield 855 kg), the data's 93.5 m grid for
water points, random tie-breaks and unclamped negative harvests. Ten places where the replication
departs from the text are **Replication quirks** in the Rules panel, each with a one-line
explanation (all on in the published presets, all off in `lhv-documented`); the most important is
that the replication gives a new household a fresh store of corn instead of a third of its
parent's.

- `lhv-published` (JASSS Table 4's calibration: death age 38, fission until 34 with probability
  0.155, harvest adjustment 0.56, harvest s.d. 0.4): measured over seeds 1–15, 172 households in
  1050–1130 against a record of 156, 94 in the 1140s–60s dip (133), 180 in 1180–1265 (172), then
  59 in 1300 and 22 in 1350 where the record falls to 0 — the shape of JASSS Figure 10, including
  its failure to empty the valley.
- `lhv-published-defaults` (the ODD's defaults): about 1 050 households at the plateaus, about five
  times the record's peak (216 households in 1269), as JASSS Figure 2.
- `lhv-documented` (the text, with the calibrated values): 41 and 80 households at the plateaus and
  7 of 15 runs empty by 1350 — the written model does not reproduce the published curve. Turning
  the fresh endowment back on alone brings its mean fit to 1 169 811, 1.27 times the replication's.
- **Replication vs documented — Anasazi (Compare)** opens the two side by side.

The valley is drawn by **Occupation** (farms green, homes yellow), **Zones** or this year's
**Yield**, with **Water sources**, **Settlements** (dots sized by households) and **Farm–home
links** as overlays; the toolbar shows the year (`AD 1142`) and the run pauses at the end year with
a notice. Inspect shows a cell's zone, PDSI class, yields, soil and water, and its household's age,
corn, harvest and expectation. Charts: Households vs historical (the record as a reference line),
Carrying capacity (plots that can feed a household this year), Fit (the running sum of squared
differences from the record), Mean stored corn, and Births, moves and departures. The harvest
adjustment and yearly harvest s.d. apply to the running world. The data files in `data/anasazi/`
come unmodified from *Artificial Anasazi* v1.1.0 (Janssen, CoMSES,
[doi:10.25937/krp4-g724](https://doi.org/10.25937/krp4-g724)) and are GPL-2.0, separate from this
repository's MIT code (see `data/anasazi/NOTICE`). See
`docs/superpowers/specs/2026-09-25-anasazi-design.md` and its source extraction.

### Civil violence (Epstein 2002)

Model I is generalized rebellion against a central authority, on a 40 × 40 torus of agents and
cops. Each agent draws a hardship H and a risk aversion R once from U(0,1) and computes a
grievance G = H(1 − L) against a legitimacy L set for the whole run; a cop counts the active
agents within its vision and arrests one at random (rule C), giving it a jail term drawn from
U(0, J_max); an agent estimates its arrest risk from the cops and actives it can see and goes
active when its grievance exceeds that risk plus a threshold T (rule A); everyone, cops included,
moves to a random empty site within its own vision each turn (rule M). Agents and cops act once
each per tick, in random order.

Model II turns the same rules on two groups, Blue and Green, instead of on a central authority:
going active means killing a random member of the other group within vision rather than merely
showing color, cops arrest either group's actives evenhandedly, and the population itself
changes — each free agent clones onto an empty neighboring site with probability 0.05 a tick (the
child keeping its parent's group and hardship, drawing its own R), and every agent dies at a
random age fixed at birth (up to 200 ticks).

The paper leaves several things unstated; the choices made here are stated here and in the
module docs (the Rules panel's help for vision names the first): vision is Euclidean (never
specified; NetLogo Rebellion's `in-radius` agrees), a jail term is a whole number of ticks from 1
to J_max, counted down at the end of every tick including the arresting one (as NetLogo does), so
an agent arrested at tick t with a term of n is freed at the end of tick t + n − 1 (a term of 1
releases it at the end of the very tick it was arrested, before it acts again, and it is never
counted as jailed), a released agent reappears on a random empty site near where it was arrested
(or anywhere on the lattice if none is free nearby), and a Model II clone lands on one of its
parent's eight Moore neighbors.

**The Finding.** The paper's stated arrest rule, P = 1 − exp(−k·C/A), does not produce the paper's
own punctuated equilibrium at the paper's own Run 2 inputs: over 20 seeds of 3 000 ticks it gives
zero outbursts above 50 actives in every seed (peaks of 13–34 actives). At L = 0.82 the grievance
ceiling is only 0.18, and even a crowd that outnumbers the cops three to one faces P ≈ 0.54 under
the literal formula — enough to suppress nearly everyone. Only NetLogo Rebellion's departure,
rounding C/A down before applying the formula, produces the pattern: 101–122 outbursts, peaks of
290–368. Checks run during planning that vary the ordering (agents deciding before moving: peaks
11–15), the schedule (cops acting after all agents: peaks 14–21) and the vision shape (reading the
paper's "north, south, east, and west" as a Sugarscape cross of 4·⌊v⌋ sites: mean actives ~40 at
every tick, constant unrest and never calm) all fail to rescue the literal rule. Because of this,
every Model I preset here rounds C/A down and says so in its description; the built-in sweep
`cv-ratio-rules` charts outbursts against legitimacy under the literal rule, the rounded-down rule
and NetLogo's further departure of counting an already-active agent twice, at Run 2's other inputs.

What else does and does not reproduce, measured over 20 seeds each:

- Run 2's shape reproduces (mean total activation 779 against the paper's 708 ± 230) but its
  timing does not: the mean wait between outbursts is 22 ticks, a third of the paper's 60 ± 55.
- Runs 3 and 4 (salami tactics vs. one jump): the jump's peak beats salami's in 17 of 20 seeds as
  the paper predicts, but its jail ends larger only in 7 of 20 — the explosion reproduces, the
  larger jail mostly does not.
- Run 5 (cop reductions) tips every one of 20 seeds into rebellion, as the paper's text says a
  marginal cut in cops does.
- Run 6 (coexistence) never kills in any of 20 seeds; Run 7 (cleansing) reaches genocide in all
  20, the victor about even (Blue 12, Green 8).
- Run 8's text calls its outcome "a stable, but nasty, regime," but measured, one group is gone in
  every one of 20 seeds (t = 75–882) — the paper's own Fig. 15 shows the same rapid genocide it
  describes in words. Peacekeepers deployed at t = 50 (`cv-safe-havens`) don't produce safe havens
  either: genocide in all 20 seeds, only delayed by denser forces.
- The paper's own worked example gives outburst sizes "60, 100, 120, 95, and 80" summing to "500"
  and averaging "100"; they sum to 455 and average 91. The test suite uses 455.

NetLogo Rebellion (Wilensky, 2004) departs from the paper's text in five ways, each its own switch
(all off by default, all on in `cv-netlogo`): C/A rounded down before the arrest formula, above
(`floor_ratio`); an already-active agent counting itself twice toward A
(`active_counts_twice`); the arresting cop stepping onto the vacated site (`cop_moves_to_arrest`);
a jailed agent keeping its site instead of leaving the lattice (`jailed_stay`); and a jail term
drawn as a whole number from 0 to J_max − 1, so a term of 0 releases immediately
(`netlogo_jail_term`). `cv-netlogo` runs NetLogo's own defaults (70 % agents, 4 % cops, vision 7,
L = 0.82, J_max = 30) with all five on: 93–109 outbursts, mean total activation 1107, mean wait 23
ticks over 20 seeds.

Three built-in sweeps: `cv-ratio-rules` (above); `cv-peacekeeping` reruns Run 7 with cops from the
start at densities 0–0.1 for 3 000 ticks (Figs. 15–17): genocide in all 220 runs, mean time to
extinction rising with density (94 ticks with no cops, 106–119 at 0.01–0.03, 156–196 at 0.04–0.07,
223–263 at 0.08–0.1) and its spread widening too (s.d. 42 up to 142–326, longest 1596 at density
0.08) — the rise and the widening reproduce, the paper's long delays past 15 000 cycles do not;
`cv-jail-waits` asks the paper's open conjecture whether a longer jail term raises the mean wait
between outbursts, and it does, from 3.5 ticks at J_max = 10 to 40.4 at 60 (about 0.66–0.74 ×
J_max from 15 up; at J_max = 5 no outburst ever ends, so no wait is measured).

The grid is drawn by **Action** (quiet agents blue, active red, cops light gray — the paper's
black, lightened for the dark grid; in Model II quiet agents show their group's color) or
**Grievance** (shaded by G), and in Model II also **Group**; charts split **Actives, quiet and
jailed**, **Legitimacy** and **Cops** onto their own axes (Figs. 9–11), plus **Tension** (Fig. 8),
**Outbursts**, **Wait between outbursts** and **Activation per outburst**, and Model II's
**Groups** and **Killed**. **Compare** entries: "Salami tactics vs one jump — Civil Violence
(Compare)" and "Ethnic cleansing vs peacekeepers — Civil Violence (Compare)." Scheduled and ramped
values show in the Rules panel as they take effect; a schedule entry on a field a ramp is moving,
or a live edit to it, is overwritten by the ramp on the next tick of its window. Credit:
Joshua M. Epstein, "Modeling civil violence: An agent-based computational approach," *PNAS* 99
suppl. 3 (2002), 7243–7250, and NetLogo *Rebellion* (Wilensky, 2004). See
`docs/superpowers/specs/2026-09-25-civil-violence-design.md`.

### Tag cooperation (Riolo, Cohen & Axelrod 2001)

100 agents each carry a tag and a tolerance, both drawn from [0, 1]. Each generation every agent
meets P = 3 others at random and donates to each (cost c = 0.1 to itself, benefit b = 1 to the
other) when the other's tag lies within its tolerance of its own: |τ_A − τ_B| ≤ T_A. Then each
agent faces a random other and the higher score has the offspring, which inherits its tag and
tolerance; with probability 0.1 the offspring's tag is redrawn, and with probability 0.1 its
tolerance gets Gaussian noise (s.d. 0.01), floored at 0. There is no space and no memory — pairs
rarely meet twice and nobody knows what anyone did before — yet the paper finds 73.6 % of
pairings end in a donation, through clusters of similar tags that rise, are invaded by less
tolerant mutants and are replaced.

**The published tables depend on a rule the paper does not state.** "Giving an offspring to the one
with the higher score" says nothing about equal scores. Read literally — a coin flip — the model
gives 42 % donation at two pairings where the paper reports 4.3 %, and 45 % at cost 0.5 where it
reports 24.7 %; only "the current agent wins ties" reproduces the paper, as Edmonds & Hales (2003)
found. And under any tie rule, cooperation exists only because agents with *identical* tags must
donate to each other (≤ with T ≥ 0): donating only when |Δtag| < T, letting tolerance fall to
−10⁻⁶ (Roberts & Sherratt 2002) or adding 10⁻⁶ of noise to every tag each collapse donation to
1–5 %, while fixing every tolerance at zero *raises* it to 75 %. The tolerance mechanism the paper
credits does none of the work. The config default is the literal rule (coin-flip ties); the
presets that reproduce the paper set `tie_rule: current` and say so.

The Rules panel's **Replications** section holds the switches: `tie_rule` (coin flip, current
agent, opponent — Edmonds & Hales' "no bias", "selected bias" and "random bias"), `donation_test`
(≤ or <), `tolerance_floor`, `tag_noise` and `selection` — a tournament, or the paper's learning
variant, adopting a better agent's traits with probability (s_other − s_self) / (b + c). The
paper gives that probability no scale; b + c, the most one donation can move two scores apart,
reproduces its 49 % at one pairing, where the generation's range of scores gives 7 %.
`initial_tolerance` (`"uniform"` or `{ "fixed": x }`) is set by presets, files and links.

- `rca-published`: the paper's setup, ties to the current agent: 73.7 % donation (the paper: 73.6 %).
- `rca-literal`: ties by coin flip: 73.7 % at P = 3, the same as published.
- `rca-published-p2` / `rca-literal-p2`: at P = 2, 2.0 % vs 42 % (the paper: 4.3 %).
- `rca-strict`: donate only when |Δtag| < T: 1.4 % (Edmonds & Hales report 0.0 %).
- `rs-no-forced-clones`: tolerance floor −10⁻⁶: 1.4 % (Roberts & Sherratt: 1.48 %).
- `eh-clones-only`: tolerance fixed at 0 with coin-flip ties: 75.3 %; with ties to the current
  agent the same setup gives 0 %, because no agent can ever have two offspring.
- `eh-no-exact-clones`: tag noise 10⁻⁶ on every offspring: 1.4 %.
- `rca-adopt-p1`: the adoption variant with one pairing: 48.8 % (the paper: 49 %).

What does not reproduce (20 seeds × 30 000 generations): the paper's picture of clusters
continually rising and being invaded. Takeovers are rare — a median of 29 per run, where its
Fig. 1 shows two in 500 generations — and dominant clusters hold 86 % of the agents (the paper:
75–80 %), are 91 % one exact tag when they take over (79 %) and 96 % ten generations later (97 %),
and their mean tolerance drifts up, as the paper says, but further: from 0.010 when they take
over to 0.039 in their last dominant generation (the paper: 0.027). Edmonds & Hales' "with 200 agents
the donation rates vanished" is partly true: with ties to the current agent 3 of 20 runs never
cooperate, and the rest do, at 74 %.

The page draws a tag × generation diagram — 100 columns of tags 0.01 wide, the last 200
generations, the newest at the bottom (Edmonds & Hales' Fig. 4) — shaded by **Count**,
**Tolerance** (the mean, 0 to 0.05) or **Clones** (the share with tolerance 0). Inspect shows a
cell's generation, tags, agents, tolerances and donations, and in the newest row each agent; agents
live one generation, so there is nothing to follow. The charts are Donation rate, Tolerance,
Clusters, Distinct tags and Takeovers, against the generation. A cluster is our definition (the
paper gives none): the agents within 0.01 of the most common exact tag, dominant above half the
population; a takeover is a new dominant cluster more than 0.01 from the last. A run stops at
generation 30 000 (`end`; 0 for never). **Compare** entry: "Published vs literal ties at P = 2 —
Tag Cooperation (Compare)". Four built-in sweeps: `rca-pairings` (Table 1 under each tie rule),
`rca-cost` (Table 2), `rca-clones` (the paper against the four ways of taking forced donation
between identical tags away) and `rca-population` (50 to 400 agents). Credit: Rick L. Riolo,
Michael D. Cohen and Robert Axelrod, "Evolution of cooperation without reciprocity," *Nature* 414
(2001), 441–443; Bruce Edmonds and David Hales, "Replication, Replication and Replication: Some
Hard Lessons from Model Alignment," *JASSS* 6(4) 11 (2003); Gilbert Roberts and Thomas N. Sherratt,
"Does similarity breed cooperation?", *Nature* 418 (2002), 499–500. See
`docs/superpowers/specs/2026-09-25-tags-design.md`.

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
  (carrying capacity vs vision under the two price rules), `schelling-tipping` (segregation vs
  a fixed Schelling preference), `lhv-calibration` (the Long House Valley's fit vs the harvest
  adjustment: best at 0.56, as JASSS finds) and `lhv-quirks` (the fit with each replication quirk
  turned off: the fresh endowment matters most by far, 4.7×; fission needing a free farm is next, 1.34×). Each file's description records its measured settings; in the
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
speed menu runs 1/s, 2/s, 5/s, 10/s, 20/s and 30/s below one tick a frame, then 1×, 2×, 5×,
10×, 25× and 100× ticks a frame, and **Max**, which runs the simulation as fast as it goes and
redraws about 30 times a second. A run does not depend on the speed: the same setup and seed
give the same world at the same tick. The page's worlds stop at 1 000 000 ticks, whatever the
model, so the whole history fits in memory: the world pauses there and says so (export its
data, or Reset). The command-line tool has no such limit.

The toolbar's **⟲1** steps back a tick, and a slider ranges over 0 to the furthest tick this
branch has reached; dragging it or pressing ⟲1 moves the world to that exact tick, with the
grid, charts and Inspect showing what they showed then. Playing on replays the same recorded
future until you make an edit, which drops it and starts a new branch from there. Seeking is
disabled, with a tooltip saying why, once the edit log is full and can no longer rebuild the
session exactly; in Compare, seeking moves both worlds together and the slider's end is the
smaller of the two worlds' reached ticks.

A **Stop at** control beside Play holds two independent, optional rules — at a tick, and/or
when a chosen series crosses `<` or `>` a value — checked after every tick at every speed, Max
included, so a run stops on the exact tick with a notice naming the rule. A rule fires only on
becoming true, not while it is already true, so a condition already met when Play starts does
not stop the first tick. Rules persist across seeks and resets and are cleared when a new model
kind loads (its series differ); they are not part of links or sessions. In Compare only the
tick rule applies — the condition rule is disabled there, since the two worlds could cross it
on different ticks.

Beside the speed menu, a **t/s** readout shows the measured ticks per second while a run plays
(hidden while paused); in Compare it counts lockstep pairs.

Keyboard shortcuts act on whatever Play and Step drive — one world, or Compare's lockstep — and
are ignored while typing in a field or with Ctrl, Alt or Meta held:

| Key | Action |
| --- | --- |
| Space | Play / Pause |
| → | Step one tick |
| ← | Back one tick |
| `[` / `]` | Slower / faster |
| R | Reset |
| ? | Show / hide a card listing these |

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
    sugarscape run --preset lhv-published --ticks 550 --series-csv lhv.csv          # AD 800–1350
    sugarscape sweep --builtin fig-ii-5 --out fig-ii-5.json --summary-csv fig-ii-5.csv
    sugarscape sweep my-sweep.json --jobs 4 --seeds 3 --ticks 300 --runs-csv runs.csv

`run` runs a preset or config of any model; it defaults to seed 1 and 1000 ticks (an anasazi run
stops at its end year and says so on stderr); `--config-out` writes the config it ran and
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

The Long House Valley data (`data/anasazi/`) are Marco Janssen's *Artificial Anasazi* v1.1.0,
CoMSES Computational Model Library, doi:10.25937/krp4-g724, under the GPL-2.0 (see
`data/anasazi/NOTICE`, `LICENSE` and `CITATION.cff` there). They are compiled into the page's
WASM, so the web build also serves those three files at `anasazi-data/NOTICE`,
`anasazi-data/LICENSE` and `anasazi-data/CITATION.cff` beside the page, and the Rules panel of an
Artificial Anasazi world credits the data and links the notice.
