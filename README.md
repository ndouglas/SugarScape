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
- Immune learning follows the book's footnote 16 by default (`disease.learning: per_agent`): an
  agent flips one bit a tick in all, toward the oldest disease it still carries, and a cured
  disease is shed at the start of its next turn (`disease.cure: next_tick`, as in the worked
  example). Under this reading `v-1-rid` reaches zero infected in 20 of 20 runs, and so does
  `v-2-endemic`: the book's endemic level does not reproduce. The earlier reading, one flip per
  carried disease each tick (`per_disease`), leaves V-1 a residue of about 1–3% and keeps V-2
  endemic; `disease.cure: immediate` sheds a disease the moment it is matched.
- Chapter VI leaves details unstated, so each is a switch that defaults to the earlier behavior:
  founders' ages (`lifespan.founders`: `newborn` or `random`), what wealth makes an agent
  fertile (`sex.fertile_wealth`: `each_good`, `total`, `welfare` or `sugar`), and per-placement
  spice, spice metabolism, age and endowment. Across all 16 combinations, trade never turns
  VI-2's crash into VI-3's doubling in more than 1 run of 20.
- `iv-15-trade-sex` follows Figure IV-14's fertility ends (women 35–45, men 45–55); under the
  book's rules its population dies out in 15 of 20 runs.
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

The presets menu groups its presets by model: **Sugarscape**, **Minds**, **Schelling**, **Ring World**,
**Artificial Anasazi**, **Civil Violence**, **Tag Cooperation**, **Spatial Games**, **Axelrod Culture**,
**Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence**, **Social Structure**,
**Demographic PD**, **Norms and Metanorms**, **Relative Agreement**,
**Image Scoring**, **El Farol and the Minority Game**, **Ants and Recruitment**, **Threshold Models**,
**The Timing of Retirement**, **Altruistic Punishment**, **Zero-Intelligence Traders**, **Balinese Water Temples** and **The Emergence of Firms**.
Each preset is listed by a plain title saying what happens in it; under the menu, the chosen
preset's source (the book's figure or animation, or the paper) and its rules sit above its description.
Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
(Max included), Share, Export, Record, Compare, Experiments and the CLI work the same for every
model. **Minds** runs on the sugarscape model but is its own entry: it holds the Minds experiments'
presets (grouped by milestone, Minds 1–8), and its Rules panel adds the Decision, Movement, Memory, Truffles,
Caching, Theft and Watching sections, which the book's Sugarscape entry leaves out. A world counts as Minds if it came from a Minds
preset or uses any Minds rule (a decision other than rule M, walking, memory, walls, truffles,
caching, a carrying limit, central-place foraging, a winter everywhere at once, theft or watching), so
a Minds share link opens under Minds. A config without a `model` key is a sugarscape config, so every older config, link, session
file and sweep reads as before. The other models' Rules panels are built from their parameter
schemas (each section says whether its fields rebuild the world or apply as it runs, and live
changes replay from links); they have no editing tools, overlays, trails or Credit tab. Compare
pairs two worlds of one model: choosing another model's preset while comparing leaves Compare
keeping A. See `docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md`.

### Schelling segregation (Schelling 1971; Epstein & Axtell's variant, VI-4 to VI-7)

The model is Schelling's own by default ("Dynamic Models of Segregation", *J. Math. Sociol.* 1,
1971, pp. 154–166): a 13-row, 16-column board with edges, 69 stars (Red), 69 zeros (Blue) and 70
blanks placed at random; each wants "no fewer than half" of its neighbors — the eight surrounding
squares, occupied ones counted (a corner has three, an edge five) — to be its color; the
discontented move "to the nearest satisfactory vacant square", nearest "measured by the number of
squares one traverses horizontally and vertically", a round at a time: those discontented at the
round's start, from the upper left, each skipping its turn if it has become content (the newly
discontented wait for the next round). What he leaves unstated is ours and named: ties at random,
nowhere acceptable means staying, and no neighbors means content. Every difference from Epstein &
Axtell's variant is a switch: **Neighborhood** (eight around, radius 1–3, or four), **Edges**
(bounded or torus), **Where the discontented go** (nearest or random), **Who moves when** (rounds,
from the upper left or the center out, or everyone in a random order), **Red share** and **Exact
numbers**. Presets may give each color a **demand table** (for each number of neighbors, the
least and most alike wanted), Schelling's "eight denominators and therefore eight numerators".
Old configs without these fields read as Schelling's rules.

Schelling's 2-D numbers come from boards he worked by hand ("My samples have been too small, so
far, to allow serious generalizations"), so the survey (`s71.*`, 20 seeds, rules from his numbers)
asks whether they are typical. On the board his direction holds throughout, and his boards sit at
the segregated end of what his rules give:

- `s71-board` (Figs. 7–8): still after 3–5 rounds, neighbors 0.80 alike and 38 % with none of the
  other color — his Fig. 9 has four-fifths to five-sixths and 40 %, Fig. 8 90 % and two-thirds; the
  like-to-unlike ratio is 3.6, not "upwards of four to one".
- `s71-center-out` (Fig. 9): 0.80, as from the upper left, and 0.79 in a random order: "the
  *character* of the outcome not very much" holds.
- `s71-third` (Fig. 11): a ratio of 1.35, "slight", under his 1.5. Across demands (`s71-demand`) the
  like share climbs 0.56, 0.66, 0.72, 0.80 from 20 % to 50 %: rising, but faster from 20 % to 35 %
  than in his "rapidly rising" range of 35 % to 50 %.
- `s71-unequal-demands` (Fig. 12): the more demanding end up no more alike (0.67 each) but more
  crowded (5.6 neighbors against 4.8), as he says.
- `s71-minority` (Fig. 13): the minority goes from 0.35 to 0.57 alike — a ratio of 1.3, short of
  his 2:1 — and is denser (5.6 neighbors against 4.8).
- `s71-wide` (p. 154's 24 squares), measured on one ruler, the like share among the eight surrounding
  squares (a wider window dilutes its own count): at half alike, 0.86 against 0.80 — more sorting;
  at a moderate third, 0.55 against 0.66 — less, as he conjectured "for moderate demands", adding
  that it "may not stand up when less tolerant demands … are put into the larger neighborhoods".
- `s71-congregate` (Fig. 16): wanting three alike and indifferent to the other color, 0.79 alike and
  37 % unmixed — his "just over 75%" and 38 %.
- `s71-integrate` (Fig. 17's bands): 96 moves against 46 for the same population wanting half alike,
  and 9 % still unsatisfied when moving stops, as he expected. His ranked preferences are left to the variations milestone.

Epstein & Axtell's variant (*Growing Artificial Societies*, pp. 165–171) differs as they say
("he moves agents to the nearest satisfactory site, whereas our agents simply select an acceptable
site at random"): 2 000 agents on a 50 × 50 torus, four von Neumann neighbors, a coin for each
agent's color, everyone acting in a random order each turn. With a maximum residence an agent
leaves when it reaches it, and a newcomer of random color takes a random site where it is satisfied.

- `vi-4-schelling-25`: 25 %. Nobody moves after 2–3 ticks; segregation rises from about 0.50 to 0.63.
- `vi-5-schelling-25-residence`: 25 %, residence 80–100 ticks. It never settles; segregation
  climbs to about 0.76, well below VI-6's (the book calls the degree "comparable" to VI-4's without
  a measure, and p. 170 calls VI-5 "modestly segregated").
- `vi-6-schelling-50-residence`: 50 %: about 0.95.
- `vi-7-schelling-mixed`: preferences 25–50 %: about 0.93, close to VI-6. With four neighbors a
  preference above ¼ acts as ⅓ or ½, depending on how many neighbors an agent has (0.3 needs one of
  three but two of four).

Agents are drawn by **Color**, **Satisfaction** (the unsatisfied in yellow) or **Preference**;
the charts are Segregation, Like neighbors by color, No opposite neighbor, Unsatisfied, Moves,
Red share, Clusters (each paper's count), Segregation coefficient and Mixed pairs; Inspect shows an agent's color, preference, alike neighbors and residence. Sweeps:
`schelling-tipping` (the book's "how little racism is enough to tip a society", on VI-4: with four
neighbors only the shares 1/4, 1/3, 1/2 and 2/3 can matter, so segregation rises in steps: about
0.50, 0.63, 0.73, 0.83 and 0.93), and on Schelling's board `s71-demand`, `s71-vacancy`
(vacancy barely matters until the board is nearly full), `s71-radius` (on the eight-square ruler)
and `s71-order`. Series include `like_near`, the like share among the eight surrounding squares
whatever the neighborhood.

### Schelling's line (Schelling 1971, pp. 149–154)

Schelling's first model, a row of stars (Red) and zeros (Blue) with no gaps, as he typed it: 70
people, a coin for each (his "odd and even digits in a column of random numbers"); each counts "the
four nearest neighbors on either side" (fewer near the ends) and wants at least half alike; the
discontented, a round at a time "counting from left to right", move to "the nearest point that
meets his minimum demand", nearest counted in people passed, and intrude between two others. Left–
right ties are at random, and one with nowhere to go stays. **Most people passed** limits travel
(his "restricted movement", radius not given), with **Accepted within reach instead** his fallback
of "three out of eight". The page wraps the row every **People per row**; charts are Groups, Group
size, Like neighbors and Unsatisfied; Inspect shows a person's place and neighbors alike.

- `s71-line` (Figs. 1–2): 7 groups of 10, neighbors 0.78 alike (medians of 20; a mean of each
  person's share, where his 81.5 % pools all neighbors) — within his tabletop range, "from about five
  groupings with an average of 14 members to seven or eight groupings with an average of 9 or 10";
  his Fig. 2 has six of 12. In about half the seeds one or two people
  are still moving at round 30: his "no guarantee" of equilibrium, seen.
- `s71-line-3`: three neighbors each side: groups of 8.75 and 0.79 alike, against his "7 or 8" and
  "75% to 80%".
- `s71-line-minority` (p. 152's die-roll halving of the zeros): the zeros end 0.77 alike, against
  0.78 on the even line, in shorter runs: here the halved minority is no more segregated, where he
  says "the minority itself tends to become more segregated … as its relative size diminishes"
  (everyone together is 0.85 alike, because the majority's share rises).
- `s71-line-reach`: a 20 % minority passing at most 10 people: 6 % still unsatisfied (3 % with no
  limit), where he found "everybody achieves his desired neighborhood". Sweeps: `line-radius` (groups
  of 4.0, 6.3, 8.4, 10.5, 11.9 and 13.3 for one to six neighbors each side) and `line-reach`.

### Schelling's tipping (Schelling 1971, pp. 167–186; 1969)

Schelling's second model, the "bounded neighborhood": one area everybody prefers to its
alternatives, each person inside or outside. Each has a **tolerance**, the most of the other color
per person of their own they will accept inside (his whites and blacks are Red and Blue here); a
color's tolerances form a **tolerance schedule** (straight lines, as in his Figs. 18 and 19, a
hyperbola or tiers), and the least tolerant share can be made intolerant. "the least tolerant leave
first and the most tolerant enter first": each step, each color's least tolerant insider leaves if
the ratio inside exceeds its tolerance, or else its most tolerant outsider enters if, counting
itself, the ratio would suit it (his curves: n people tolerate n·R(n) of the other color). The
relative speeds he leaves open ("we can watch and see how they matter") are switches, with the
order of moves, the reading of entry and limits on each color or the total. The page draws his own
picture: Red inside across, Blue inside up, each color's contented region tinted, the path so far.

With his exact schedules the model is deterministic, and everything he states holds (the survey's
`tipping.*`):

- `tipping-fig18`: only the two one-color states are stable; every start on a grid ends with one
  color gone.
- `tipping-fig19`: a stable mixture at 80 and 80, reached from every start with over 40 % of each;
  from all Red, 28 Blue entering together reach it, 26 do not ("more than 25%").
- `tipping-fig20`: two to one, and the mixture is lost from every start.
- `tipping-fig21`: equal numbers, straight lines: from 420 unequal starts, none ends mixed at
  intercepts up to 2.9; some do from 2.95 (104 at 3.0, 219 at 3.5) — his 3.0, to the step of a
  discrete count (equal starts sit on the knife edge, where nobody moves, stable or not).
- `tipping-fig22` (his scale: 100 Red, 50 Blue): Red limited to 40 gives a stable mixture at 40 and
  40, where his figure's curves cross.
- `tipping-intolerant` (1969): the least tolerant 60 % intolerant: 40 and 40, exactly.
- `tipping-minority`: a 5:1 minority with the majority's schedule is always pushed out; five times
  as tolerant, it can hold a mix ("the minority must be the more tolerant").
- `tipping-less-tolerant` (Fig. 22's numbers): making the least tolerant two-thirds of Red intolerant
  holds a mixture (33 Red, 42 Blue) where the same numbers as they are, or all Red less tolerant,
  end all Red.

Sweeps: `tipping-start`, `tipping-entry`, `tipping-intercept`, `tipping-speeds` (Blue moving up to
five times as fast still loses Fig. 18: here the numbers decide) and `tipping-limit`.

### Variations on Schelling (milestone 32)

Later researchers' versions of Schelling's board, each under its authors' own rules and measured
with its own cluster count, as switches on `schelling` and `line`: **Who may move** (the
discontented, or anyone: Pancs & Vriend's "no inertia"), **Where they go** adds **best** (the empty
square of highest utility, staying included, ties at random) and **swap** (Zhang: two occupied
squares, not neighbors, trade places with logit odds e^(β·gain) on their summed utility), a
**Utility** (Pancs & Vriend's flat, p50, p100 and spiked over the unlike share; Zhang's tent,
best at half alike) with **β**, a **Start** (random, checkerboard, or Singh et al.'s checkerboard
with the vacancies removed), and on the line a **ring**. New series: `clusters` (side by side),
`pv_clusters` (Pancs & Vriend's, joined through blanks that touch one color only), `clusters8`
(Singh et al.'s, joined at sides or corners), `seg_s` (Gauvin et al.'s s) and `mixed_pairs`
(Zhang's ρ, always on the eight around). The survey (`variations.*`, 20 seeds, rules from each
paper's numbers):

- Pancs & Vriend 2007 (5 × 5, 10 of each, 100,000 turns): random boards average 7.68 clusters
  (theirs 7.82); flat 2.15, 17 of 20 completely segregated (2.10, 91 %); p50 2.05, 19 of 20 (2.04,
  98 %); p100 4.69 over 500 seeds (4.99), 85 % ending in a strict equilibrium (their 854 of 1,000).
  One of their statements does not reproduce: footnote 23 calls spiked preferences "very similar"
  to p100, but spiked boards end at 6.99 clusters, near a random board's 7.8 (500 seeds). On the ring of
  20, even those liking half and half best end in two groups, 20 of 20 (`pv-ring`).
- Gauvin, Vannimenus & Nadal 2009 (50 × 50, 5 % vacant, anyone moving to a random acceptable
  vacancy): frozen at a tolerance of 0.35 (nobody can move), s = 1.00 at ½, still 1.00 at 0.7 and
  0.05 at 0.8: their "segregated phase … up to a tolerance T as high as 3/4". The sweep `gvn-phase`
  draws their diagram at 5, 20 and 40 % vacant; across vacancies our frozen line sits where their
  Table 1 has it, within one step of T (2/7 at 20 %, where they have 1/3).
- Singh, Vainchtein & Weiss 2009 (a torus, 3 of 8 alike with blanks counting against, a deleted
  checkerboard): two clusters on 8 × 8 in 20 of 20, 20.5 on 100 × 100 at 24 % vacant and 57.5 at
  33 % (their 22 and 55): "strictly a small city phenomenon" holds. Sweep `svw-city`.
- Zhang 2004 (JEBO; 100 × 100 torus, no vacancies, everyone liking half and half best): from a
  checkerboard, mixed pairs fall from 20,000 to about 2,000, and stay there. Zhang's own identity
  holds exactly: two traders' summed gain is 0.075 per mixed pair removed, so his model is a
  conserved Ising model. But his waiting times do not reproduce. Fig. 8's cutoff, 600 mixed pairs, is
  the least a 100 × 100 torus split in half can have (two perfectly straight bands), so "below 600"
  cannot be literal; runs round into blobs near 2,000 by 10 million draws. Read as his scaled
  potential (0.075 per pair, so 8,000 pairs), all 20 get there in about 100,000 draws, some 400
  times sooner than his 40 million. Below β = 2 nothing sorts, as he says, but the sharp change comes between β = 4 and 8,
  not near 2. More neighbors sort faster (16, 10 and 8.5 steps with four, eight and twelve), but
  eight take 0.62 of four's time, not "less than half". Sweeps `zhang-beta`, `zhang-neighborhood`.

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

**The published tables depend on how ties are read.** "Giving an offspring to the one with the
higher score" (p. 441) says nothing about equal scores; read as a coin flip, the model gives 42 %
donation at two pairings where the paper reports 4.3 %, and 45 % at cost 0.5 where it reports
24.7 %. But the paper's p. 442 settles it: an agent "adopts the other's tag and tolerance if the
other's score is higher than its own", so a tie keeps the current agent, and that rule reproduces
the paper, as Edmonds & Hales (2003) found. (Corrected 2026-09-28: this section first called the
rule unstated.) And under any tie rule, cooperation exists only because agents with *identical* tags must
donate to each other (≤ with T ≥ 0): donating only when |Δtag| < T, letting tolerance fall to
−10⁻⁶ (Roberts & Sherratt 2002) or adding 10⁻⁶ of noise to every tag each collapse donation to
1–5 %, while fixing every tolerance at zero *raises* it to 75 %. The tolerance mechanism the paper
credits does none of the work, though the paper itself reports the clusters of identical tags
and their high relatedness (p. 442). The config default is coin-flip ties; the presets that
reproduce the paper set `tie_rule: current` and say so.

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

### Spatial games (Nowak & May 1992, and its critics)

Nowak and May's spatial Prisoner's Dilemma: every site of a lattice holds a player who either
cooperates (C) or defects (D) and plays the game with its eight neighbors and with itself. Two
cooperators get 1 each; a defector against a cooperator gets the temptation b (b > 1) and the
cooperator 0; two defectors get ε (0 in the paper). A player's score is the sum over those games.
Each generation every site is taken over by the highest-scoring player among its previous owner and
its neighbors, all at once. The lattice's edges are fixed (edge players simply have fewer
neighbors) or periodic; a four-neighbor (von Neumann) lattice and play without self-interaction
(a = 0) are the paper's variants. There is no randomness once the start is drawn.

Huberman and Glance (1993) replaced the synchronous generation with asynchronous updating: one
player at a time, chosen at random, is rescored from the current strategies and replaced by the
best in its neighborhood, N such microsteps a generation. Nowak, Bonhoeffer and May (1994) answered
with probabilistic winning, their Eq. 1 — a site becomes C with probability Σ A_i^m s_i / Σ A_i^m over
itself and its neighbors (s_i = 1 for C), so m → ∞ is the deterministic rule, m = 1 "proportional
winning" and m = 0 "random drift" — in both discrete and continuous time (continuous time is Huberman
and Glance's asynchronous updating), and with irregular arrays (a share of a grid's cells occupied at
random, each player playing everyone within radius r) and three-dimensional lattices (a cube with
26 neighbors, viewed one z-slice at a time).

The papers leave several things open; the choices made here are stated here and in the module docs.
A tie for the highest score between a C and a D (only when both score 0, or when b sits exactly on a
threshold ratio) leaves the owner its strategy. Eq. 1 with every candidate scoring 0 (0/0) also
keeps the strategy, and m = 0 counts every candidate once (0^0 = 1). Asynchronous updating picks
players with replacement. NBM94 never state the start of their Figs. 1–2; their arena presets start
from 50 % defectors at random, because their m = 0 row, pure drift that keeps its start, shows
roughly even colors (a 10 % start would leave it ~90 % blue), and the random array starts there too.
A random array's cells are drawn once at setup, with self-interaction and distances between cell
centers. The cube, for which NBM94 give no detail, uses b = 1.6 and 10 % defectors, chosen by
measurement. Hexagonal lattices, which NM92 describe only qualitatively, are not implemented. Every
power in Eq. 1 goes through the crate's portable ln and exp, so runs are identical natively and in
the browser.

What reproduces, measured (release, seeds 1–20 unless noted):

- **The kaleidoscope** (`nm-3-kaleidoscope`: one defector at the center of a 99 × 99 lattice of
  cooperators, b = 1.9) reproduces exactly: the pattern is four-fold symmetric at t = 30, 217, 219
  and 221 and at every generation checked, and it reaches the edges at t = 49, as NM92 say.
- **Spatial chaos settles at 12 ln 2 − 8** (0.31777): on 400 × 400 from 40 % defectors
  (`nm-2a-universal`) the mean f_C over t = 201–300 is 0.3179 ± 0.0005, the constant to three
  decimals. On 200 × 200 every seed settles at 0.318–0.323 from 5 %, 30 % and 60 % defectors, and
  from 80 % so do 19 of 20 (seed 12 ends all C).
- **The cluster thresholds** hold exactly on hand-built worlds: a 6 × 6 D block grows at b = 1.85
  and shrinks at 1.75, a 2 × 2 D cluster grows at 1.85, and a 2 × 2 C cluster grows at 1.95 and not at
  2.05.
- Fig. 1a's static network (`nm-1a-static`, b = 1.77): f_C 0.737–0.748 at t = 200, inside the paper's
  "usually between 0.7 and 0.95", with 3 % of sites still blinking. Without self-interaction
  (`nm-no-self`, b = 1.62): 0.301–0.305 against the paper's ~0.299.
- **NBM94's regimes** (the sweeps below, 80 × 80 periodic, 50 % defectors, 5 seeds): all C near
  b = 1, all D approaching 2 and coexistence between, for every m and in both discrete and continuous
  time; continuous time removes only the 1.8 < b < 2 chaos (at b = 1.9, deterministic, all D in 20 of
  20 seeds in continuous time, still chaotic in 16 of 20 in discrete time) and helps C at m = 1 (0.86
  at b = 1.35 against 0.30 in discrete time).
- **r_c ≈ 9** (`nbm-random-array`, b = 1.6, 50 % defectors): all D in 0, 10 and 20 of 20 seeds at
  r = 5, 9 and 11.
- The cube (`nbm-cube`) is "similar to the two-dimensional" lattice, as NBM94 say: coexistence for
  b from 1.1 to 1.8, all but all D at 1.9; at b = 1.6, f_C 0.331–0.342 over t = 101–200 with a quarter
  of the cube changing every generation.

What does not, or only partly:

- **Four neighbors** (`nm-four-neighbors`, b = 1.8): 0.379–0.382 over t = 501–1000 (the survey's
  0.380), the same at every b in (5/3, 2), against the paper's ~0.374 — close, but not within 0.005.
- **Huberman and Glance's "always".** Their kaleidoscope with asynchronous updating
  (`hg-async-kaleidoscope`, b = 1.9) is all D at t = 56–149 (mean 101), "within a hundred generations
  or so," as they say: their case reproduces. They ran NM92's Fig. 3 regime (their synchronous panel
  "corresponds to figure 3b", where 1.8 < b < 2), in which the kaleidoscope is the same for any b, and
  their "always" is about starting states ("as long as there is at least one defector in the initial
  state"). What doesn't carry over is the broader conclusion they drew, as NBM94 showed (HG93 "consider
  only this single case"): at b = 1.7 the defectors stay a small stuck cluster and never take over (f_C
  0.974–0.999), and across NBM94's b values one defector takes over (or nearly) only at 1.9 and 2.01 (f_C 0
  and 0.04; 0.61 at 1.55, 0.99–1.00 elsewhere).
- **"C cannot persist" at m = 1 without self-interaction** (NBM94): C is gone (f_C ≤ 0.007) at
  b = 1.13 and 1.35, but at b = 1.05 it keeps 0.16–0.33. (Deterministic winning without
  self-interaction keeps C, 0.85–0.95, as they say.)
- **r_c depends on an unreported start.** From 50 % defectors r_c ≈ 9 reproduces; from NM92's 10 %
  no radius up to 11 ends all D (f_C above 0.3). NBM94 do not report their start, so their r_c is a
  property of it as much as of b.
- NBM94's figure captions list nine b values and omit 1.77, which both figures show; the sweeps
  include it. The figure's m = 1 discrete thumbnails show scattered C at b = 1.9 and 2.01 where these
  runs are all D (0 from 1.9; 0.002 at 1.77). In discrete time at b = 1.9 two of 20 seeds end all C
  and two nearly all D, a small-lattice collapse the paper does not mention.

The survey measures 13 of these claims: 10 hold and 3 fail (four neighbors, m = 1 without
self-interaction, and Huberman and Glance's "always").

Five built-in sweeps: `nm-universal` varies the starting defectors from 5 % to 95 % at b = 1.9
(0.320–0.321 from 5 % to 80 %, 0.21 from 90 %, 0 from 95 %: "almost all starting proportions" holds
up to 80 %); `hg-async` runs the kaleidoscope at NBM94's ten b values, synchronous and asynchronous
(asynchronous 0.99–1.00 for b ≤ 1.42 and at 1.71–1.77, 0.61 at 1.55, 0 at 1.9 and 0.04 at 2.01;
synchronous 1.00 through 1.77, 0.34 at 1.9, 0.91 at 2.01); `nbm-grid-discrete` and
`nbm-grid-continuous` are NBM94's Figs. 1 and 2 as numbers, the final f_C for their ten b values
(1.77 included) and seven values of m (in discrete time m = ∞ gives 0.99 at 1.05, 0.88–0.94 from 1.13
to 1.77, 0.30 at 1.9 and 0 at 2.01, and m = 0 gives 0.56 at every b; in continuous time m = ∞ gives
0.59–0.99 up to 1.77, 0 at 1.9 and ≈ 0 at 2.01, and C persists to 2.01 at m = 1, 0.02, and m = 0.5, 0.23);
and `nbm-radius` runs the random array's radius from 2 to 11 from 10 % and from 50 % defectors (from
50 %, 0.27–0.58 up to r = 9 and 0 at 10 and 11; from 10 %, 0.73–0.86 at every radius).

Players are drawn by **Change** (NM92's colors: blue C after C, red D after D, yellow D after C,
green C after D; the default), **Strategy** (blue C, red D) or **Payoff** (low to high heat), with a
random array's empty cells dark. A cube shows one z-slice, chosen by the **Slice** menu (in place of
Landscape); the slice changes only the view, not the run. Inspect shows a player's strategy now and
before, its score, and its candidates summarized (the C and D counts and best scores) with the
winner (deterministic) or P(C) (probabilistic). Charts: **Cooperators**, **Changes** (the share of
players that switched), **Switches** (C to D and D to C, as counts) and **Payoffs** (the mean score
of C and of D). **Compare** entries: "Synchronous vs asynchronous — Spatial Games (Compare)" (the
kaleidoscope and Huberman and Glance's asynchronous twin) and "Discrete vs continuous time — Spatial
Games (Compare)" (`nbm-discrete` and `nbm-continuous`: 80 × 80, 50 % defectors, b = 1.71; f_C
0.86–0.92 against 0.71–0.73 at t = 200). b, ε, a, the update, the winning rule and m apply to the
running world. Credit: Martin A. Nowak and Robert M. May, "Evolutionary games and spatial chaos,"
*Nature* 359 (1992), 826–829; Bernardo A. Huberman and Natalie S. Glance, "Evolutionary games and
computer simulations," *PNAS* 90 (1993), 7716–7718; and Martin A. Nowak, Sebastian Bonhoeffer and
Robert M. May, "Spatial games and the maintenance of cooperation," *PNAS* 91 (1994), 4877–4881. See
`docs/superpowers/specs/2026-09-25-spatial-games-design.md`.

### Axelrod culture (Axelrod 1997, and its docking with Sugarscape)

A lattice of sites, each with F cultural features that take one of q traits (the paper's 10 × 10,
five features of ten traits, four neighbors, edges bounded). An event picks a random site and a
random neighbor; with probability equal to the share of features they have in common, the site
copies one feature on which they differ. Similar neighbors grow more similar; neighbors with nothing
in common never interact, so the lattice freezes into stable **regions** — contiguous sites with
identical cultures — once every two neighbors are identical or share nothing. A tick is one event
per site, the paper's time unit.

Axelrod's claims mostly reproduce (means over 20 seeds unless stated): more features mean fewer
regions and more traits more (Table 2, 10 seeds as in the paper: 1.1 / 3.5 / 22.0 regions at five
features); wider neighborhoods fewer
(4.05, 2.11, 1.40 with 4, 8 and 12 neighbors averaged over the nine cultures, against 3.4, 2.5, 1.5); the territory's surprising
curve (Fig. 2) — 21 regions at 12 × 12, 4.9 at 50 × 50, 2.25 at 100 × 100 — and a torus's earlier,
lower peak; time to stability of 9 090 and 24 500 events per site at 32 × 32 and 50 × 50 (the
paper: 10 036 and 25 900); and zones settling long before regions. The sample setup is a little
more diverse than reported: over 1 000 seeds a mean of 4.3 regions and a median of 4, with 18 %
above six (the paper: 3.2, 3 and 10 %).

The docking paper (Axtell, Axelrod, Epstein and Cohen 1996) made the Sugarscape reproduce this
model and found two places where two readings of the same prose diverge; both are switches here.
**Activation**: Axelrod picks a random site each event, the Sugarscape shuffled sweeps; at 20 × 20
that alone gave 16.25 vs 9.23 regions (measured: medians 16.5 vs 11). **Who changes**: the original
Sugarscape changed the neighbor, not the active site, caught two months into the docking — measured,
it makes no difference at 10 × 10. Their **soup** (any two sites can meet) leaves about one culture,
as they found. Later work supplies two more: Castellano, Marsili and Vespignani (2000) found a
transition in the number of traits, and indeed Axelrod's "large territories have fewer regions"
holds only below it — at 25 traits regions grow with the territory (medians over 20 seeds: 207 at
20 × 20, 424 at 30 × 30);
he happened to use 15. And Klemm et al. (2003): any **drift** — here one random trait change in
10 000 events — melts the frozen borders toward one culture.

The page draws each site as a block with lanes between neighbors: **Culture** colors each culture
and shades the lanes by what neighbors share (a region reads as one blob), **Similarity** is the
paper's Fig. 1 (only the lanes), **Zones** colors the cultural zones. Inspect a site for its traits,
region, zone and each neighbor's shared features, or a lane for its pair. Charts: Regions, zones and
cultures; Largest region; Mean similarity; Active bonds (pairs that can still interact — zero means
stable); Changes. A run stops when stable. Presets: `ac-sample-run`, `ac-many-regions`,
`ac-large-territory` (100 × 100: about 10⁹ events, a long run), `ac-torus`,
`ac-random-activation-20`, `ac-sweep-activation`, `ac-neighbor-changes`, `ac-soup`, `ac-drift`.
**Compare** entry: "Literal vs Sugarscape activation, 20 × 20 — Axelrod Culture (Compare)". Built-in
sweeps: `ac-table-2`, `ac-neighborhoods`, `ac-territory`, `ac-activation`, `ac-traits-transition`,
`ac-drift`.

**In the Sugarscape**, the Culture (K) rule can be Axelrod's: agents carry features of several
traits and, after moving and eating, copy from one random neighbor as above (the book's rule flips
their neighbors' tags instead); the Culture color mode draws them, a Distinct cultures chart counts
them, and "Stop when cultures settle" ends a run once every two agents are identical or share
nothing. The docking paper's mobility experiment is `dock-mobility-15` and `dock-mobility-30`: 100
mobile agents with vision 5–10 on their "single (Gaussian) sugar mountain" (a **Gaussian** map:
capacities round(height · e^(−d²/2σ²))). They report 1.1 ± 0.3 and 2.2 ± 1.2 cultures, every run
settling; with a mountain as wide as the board (σ = 25) at the Sugarscape's usual height of 4,
measured over 20 seeds: 1.15 ± 0.37 and 2.25 ± 0.85, every run settling. The paper gives neither
width nor height, and the landscape decides it: any mountain broad or tall enough reproduces their
number (σ ≥ 20 at height 4, σ ≥ 16 at heights 6–10), while narrow, low ones leave many cultures
(24 at σ = 25 and height 2; 42 at σ = 8 and height 4). Sweeps: `dock-mobility`, `dock-mountain`.

Axelrod's own sample setup runs a little more diverse here (mean 4.33 regions and median 4 over
1 000 seeds, against his 3.2 over 10 runs and median 3 over 100), but his samples are within
sampling error of it (the closest call: 10 of his 100 runs above six regions against 18 % here,
p = 0.03). His archived demo program (CULTURE.P) copies the first differing feature it meets,
scanning two at a time, rather than a random one; as a switch (`pick: scan`) it changes nothing.
Having the neighbor copy instead (`changes: neighbor`) happens to fit his samples more closely
(mean 3.87, median 3), but his program, like his paper, changes the active site.

Credit: Robert Axelrod, "The Dissemination of Culture: A Model with Local Convergence and Global
Polarization," *Journal of Conflict Resolution* 41 (1997), 203–226; Robert Axtell, Robert Axelrod,
Joshua M. Epstein and Michael D. Cohen, "Aligning Simulation Models: A Case Study and Results,"
*Computational and Mathematical Organization Theory* 1 (1996), 123–141; Claudio Castellano, Matteo
Marsili and Alessandro Vespignani, *Physical Review Letters* 85 (2000), 3536; Konstantin Klemm,
Víctor M. Eguíluz, Raúl Toral and Maxi San Miguel, *Physical Review E* 67 (2003), 045101. See
`docs/superpowers/specs/2026-09-25-culture-design.md`.

### Emergence of Classes (Axtell, Epstein & Young 2000)

A bargaining society. Each period, N/2 pairs drawn at random play the Nash demand game: each
demands L, M or H (30, 50 or 70 percent of a pie) and both get their demands if they fit in 100,
else nothing. Every agent remembers its last m opponents' demands and, with probability 1 − ε,
makes the demand with the highest expected payoff against them (ties at random); otherwise a random
one. The defaults are the paper's Fig. 2: 100 agents, memory 10, ε = 0.2, random memories. With
**Two tags** the agents come in two types marked by a meaningless tag, and each best-replies to what
the opponent's type did — a memory per tag, as Poza et al. read AEY (**Tagged memory** switches to
one shared memory). The **Regime** chart reads the best-reply regions agents sit in, as AEY's
pictures do: equity (everyone's best reply is M), fractious (no one's is), and with tags classes
(equity within types, one type demanding H of the other), equity between types only, or "equity
above, division below". The stop, like AEY's transition target, waits for every agent to hold at
least (1 − ε)·m M's.

What reproduces (20 seeds unless stated): the realized error rate is 2ε/3 (0.1335 against the
paper's 0.1333); a random start reaches equity, by period 14 (median); transition times from a
fractious start grow steeply with memory and population. What does not: **the fractious state of
Fig. 3 is never reached from a random start, and started fractious it does not persist** (the paper:
over 10⁹ periods) — it lasts 1 to 10 periods, paying about 18 rather than a quarter of the pie, and
every run reaches equity by period 53; the transition times are about two orders of
magnitude below Fig. 4's (a median of 600 periods at m = 13, ε = 0.1, against "in excess of 10⁵");
and **with tags at AEY's parameters, classes and "equity above, division below" never appear**
(0 of 20), confirming Poza, Villafáñez, Pajares, López-Paredes and Hernández (2011) — nor at the
smaller society (20 agents, memory 5, ε = 0.05) where they report seeing it. Planted, a class
system does persist (18 of 20 at 20 000 periods).

Poza et al.'s departures are switches: **Decision** — best-reply to the most frequent remembered
demand, under which segregation appears in 10 of 20 runs and half the runs reach the fractious state
first; **Low demand** (5–45; a higher L slows the way to equity, as they found); **Memories start**
empty and growing (they report a longer transition; measured, no difference); and **Who meets whom**
— a torus of lattice neighbors, tags laid out at random, in four zones or in two (with tags at
random, 12 of 20 runs reach classes or equity between types without equity within).

The view is the paper's memory simplex: H at the top, M at the lower left, L at the lower right,
each agent a dot at its memory's mix, the background shaded by the best reply there (**Best reply**)
or the dots by their last payoff (**Payoff**). With tags, two simplexes: memories of one's own type
and of the other. Inspect a point for its mix, best reply and the agents there. Charts: Mean payoff;
Outcomes (M–M, H–L, failures, waste); M in memory; Regime; with tags Payoffs by tag and Inter-type
advantage. Presets: `aey-equity`, `aey-fractious`, `aey-transition` (stops at equity), `aey-tags`,
`aey-classes`, `pvplh-small-tags`, `pvplh-mode`, `pvplh-progressive`, `pvplh-lattice`. **Compare**
entry: "AEY’s rule vs the mode rule, with tags — Emergence of Classes (Compare)". Built-in sweeps:
`aey-memory`, `aey-population`, `aey-first-attractor`, `aey-tag-regimes`, `pvplh-payoffs`.

Credit: Robert Axtell, Joshua M. Epstein and H. Peyton Young, "The Emergence of Classes in a
Multi-Agent Bargaining Model," in S. Durlauf and H. P. Young (eds.), *Social Dynamics* (MIT Press,
2001); David J. Poza, Félix A. Villafáñez, Javier Pajares, Adolfo López-Paredes and Cesáreo
Hernández, "New Insights on the Emergence of Classes Model," *Discrete Dynamics in Nature and
Society* 2011, 915279. See `docs/superpowers/specs/2026-09-25-classes-design.md`.

### Ethnocentrism (Hammond & Axelrod 2006, and its critics)

Hammond and Axelrod's model of in-group favoritism: an empty 50 × 50 torus on which every site has
four neighbors. Each period an immigrant with random traits arrives at a random empty site; its
traits are a tag (one of four colors) and two strategy bits, whether to help an agent of its own
color and whether to help one of another. Every agent's potential to reproduce (PTR) is reset to
12 %; then each agent decides, for each occupied neighbor, whether to help it — helping costs the
helper 1 % of PTR and gives the neighbor 3 %. In random order each agent then reproduces with
probability PTR into an empty neighboring site, if there is one, the offspring copying its parent
with a 0.5 % chance of mutation per trait; finally every agent dies with probability 10 %. An agent
that helps only its own color is ethnocentric (E), one that helps everyone humanitarian (H), one that
helps no one selfish (S) and one that helps only other colors traitorous (T). The paper reports the
mean over the last 100 of 2,000 periods, over ten runs: 76.3 % ethnocentric, 74.2 % of decisions
cooperative (its Table 1 a).

**The paper, its appendix and its code disagree**, and each disagreement is a switch or a preset.
The appendix gives `MutationRate = 0.05` where the text, Table 1, the authors' code and NetLogo's
replication use 0.005: at 5 % the lattice never sorts (36.0 % ethnocentric, 56.4 % cooperative;
`ha-appendix-mutation`), so the appendix's figure is a slip. The appendix's interaction loop ("A
decides whether to donate to N … N decides whether to donate to A", for each neighbor N of each agent
A), read literally, decides every direction twice a period; the code decides once, and twice gives
80.9 % ethnocentric and 77.3 % cooperative, five points above Table 1 a (`pair_play: twice`,
`ha-appendix-double-play`). The archived Java draws a new agent's tag with Ascape's inclusive
`randomInRange(0, 4)`, so "four colors" are five (`ha-java-five-colors`); and its archived main loop
has the immigration block commented out and fills the lattice with random agents at the start, so
the code as archived is a full random start with no immigration (`ha-java-archive`) — contrary to its
own documentation, and it lands on the same outcome as the paper, faster: 77.7 % ethnocentric
(43 % by period 100, against 35 % in the standard case).

The paper leaves several things open; the choices made here are stated here and in the module docs.
Immigrants interact, reproduce and can die in the period they arrive (as NetLogo); offspring do not
reproduce in the period they are born but can die in it (the code and NetLogo agree); a tag mutation
always gives a different color (the code); a fractional immigration rate is a chance of one more
immigrant (the code's `halfImmigrant`); misperception, "misperceiving whether the other agent has the
same color", is per decision, the agent then using its other bit (the code's noise). Agents that
"distinguish all four colors" carry one help bit per color, each mutating like any other trait, and
count as ethnocentric when they help their own color only; agents "unable to distinguish their own
color from others" carry a single bit (help everyone or no one). Each period's statistics are
shares of the agents alive at its end, and cooperation is helps ÷ decisions that period; a run's
summary is the mean over its last 100 periods, as the paper's.

What reproduces, measured (release, seeds 1–10, periods 1,901–2,000 unless noted):

- **Table 1 in both columns within 3 points** for rows a (75.9 % ethnocentric, 76.0 % cooperative),
  b (cost 0.5 %), d (two colors), e (eight), f (mutation 0.25 %, Figure 1: 83.4 / 80.2 against
  82.8 / 79.8), h (immigration 0.5) and m ("run length 2,000", read as 4,000 periods since the
  standard is already 2,000: 77.1 / 75.9 against 77.3 / 74.4).
- **A lattice of egoists** with no immigration becomes "just as dominant" (`ha-egoist-start`): 78.6 %
  ethnocentric, though slowly — 7 % at period 100, the population dipping to 786 at period 200, and
  70 % at 500.
- **Misperception 10 %** (`ha-misperception`): 71.7 % ethnocentric, "more than two-thirds".
- **Hartshorn, Kaznatcheev and Shultz (2013)** reproduce almost exactly, over their 50 worlds of
  1,000 cycles: final shares 7.7 % selfish, 2.6 % traitorous, 72.4 % ethnocentric and 17.3 %
  humanitarian (theirs .08 / .02 / .73 / .17) with 1,568 agents ("just under 1,600"); by their
  chi-square tests, 17 worlds show early humanitarian dominance, 18 early ethnocentric dominance and
  15 strong competition (theirs 16 / 16 / 18); and in their Study 2 (only some strategies allowed:
  `allowed`, which presets, files and links set) ethnocentric > humanitarian > selfish > traitorous
  in every subset but HST, where traitors beat the selfish (`hks-no-ethnocentrics`: 84.7 %
  humanitarian, 8.4 % traitorous, 7.0 % selfish), all fifteen orders as their Table 3, most counts
  within 5 %.
- **Jansson (2013)**: with offspring placed anywhere instead of next to the parent
  (`jansson-offspring-anywhere`) cooperation collapses to 4.5 % (88.7 % selfish), "similar to the null
  model"; relatives (a common founding immigrant) are 75.4 % of neighboring pairs, P(same tag |
  relatives) 95.1 % and P(relatives | same tag) 90.1 % (his Table 4: 74.7, 95.3, 89.2), and 86.8 % of
  all help goes to relatives (his 89 % of an ethnocentric's); raising the tag's own mutation rate
  (`tag_mutation`), humanitarians pass ethnocentrics between 25 % and 30 % (44.9 against 39.9 at 30 %,
  `jansson-tag-mutation-30`; he says 30 %).

What does not, or only partly:

- **The color-blind 14 %.** At doubled cost HA06 report 56 % cooperation for agents that see color and
  14 % for agents "unable to distinguish their own color from others". Here seeing agents cooperate
  64.7 % and blind ones (`ha-cost-2-blind`) 41.8 %, three times 14 %. No reading of "unable to
  distinguish" gets there: one color 40.2 %, a coin flip per decision 44.9 %, every decision twice
  24.5 %. Only a harsher game does — cost 3 %, 12.7 % (or the benefit halved, 11.6 %) — and then
  seeing agents fall to 29.8 % (17.7 %), far below 56 %. And blind agents cooperate *more* than seeing
  ones whenever helping is cheap (81.2 % against 76.0 % at the standard cost, 89.0 against 78.5 at
  0.5 %): seeing color helps cooperation only somewhere between a cost of 1 % and 1.5 %. Neither of
  the doubled-cost figures reproduces, the seeing one included, and cooperation there is steep in the
  cost, so a modest unstated difference in the game could explain both; HA06 build this comparison on
  their *Theoretical Population Biology* paper (2006), not yet read here.
- **Ethnocentrics take over later than Table 1 l says.** After 500 periods 57.3 % are ethnocentric,
  not 73.9 %; the last-100 mean is 70.3 % by period 1,000 and 72.4 % by 1,500. Hartshorn, Kaznatcheev
  and Shultz's "around 300 cycles" holds for the median world (282) but worlds range from 21 to 596:
  11 of 50 settle before period 100 and 11 after 400.
- **"80 percent ethnocentric"** with each-color strategies (`ha-each-color`) holds only loosely: 27.0 %
  help their own color alone, while 84.3 % help their own color and refuse at least one other.
- Rows c, g, i, j and k are off by more than 3 points: at cost 2 % cooperation is 64.7 %, not 56.1 %;
  mutation 1 %, immigration 2 and a 25 × 25 lattice leave 4–6 points fewer ethnocentrics (63.0, 70.5
  and 64.4 against 67.1, 74.4 and 70.5); a 100 × 100 lattice cooperates 3.1 points more. Cooperation
  runs about 2 points above HA06 in most rows (mean +2.4).
- **Four colors or five?** Table 1 cannot tell: the ethnocentric share is flat from 3 to 6 colors
  (76.2, 75.9, 75.4, 76.1 %), and rows d, a and e fit 2/4/8 colors and the code's 2/5/9 about equally
  (root-mean-square error over the thirteen rows 5.2 against 5.8 points ethnocentric, 3.3 against
  4.0 cooperative).
- **Jansson's kin discriminators** (`kin_strategies`: a basis bit says whether same and other are
  judged by the tag or by a kin marker naming the family's founder) win, but by far less than his
  Table 5: 52.1 % kin and 26.7 % tag-ethnocentric, against 76.2 % and 16.4 % (`jansson-kin`). He does
  not say how the basis is inherited; fixed at immigration instead of mutating (`kin_basis: fixed`,
  `jansson-kin-fixed`), kin take 65.5 % and tag-ethnocentrics 12.6 %, nearer his table. With more
  colors the kin–tag gap closes, as he says, but below ten points from about 12 colors, not 36; with a
  fixed basis it hovers at 10–16 points from 16 to 36 colors (one dip at 24) and closes near 40. At 60 % tag mutation
  traitors only draw level with ethnocentrics (20.8 against 21.8 %; they pass by 75 %), and at 90 %
  they do not outnumber humanitarians (39.8 against 43.6 %), both of which he says they do.

The survey measures 38 of these claims (20 seeds; 50 for Hartshorn, Kaznatcheev and Shultz's own
worlds): 15 hold, 11 are weak and 12 fail. Most Table 1 rows are weak because their cooperation,
about 2 points high, puts too few worlds within 3 points of HA06's.

Seven built-in sweeps (ten seeds, 2,000 periods, the mean over periods 1,901–2,000): `ha-cost`
(cooperation against the cost of helping, seeing and blind: 78.5 / 89.0 % at 0.5 %, 76.0 / 81.2 % at
1 %, 64.7 / 41.8 % at 2 %, 29.8 / 12.7 % at 3 %), `ha-colors` (the ethnocentric share for 2 to 9
colors: 68.9, 76.2, 75.9, 75.4, 76.1, 78.6, 77.9, 81.8 %), `ha-mutation` (0.25 % to 5 %, once and
twice: 83.4 / 86.6 % down to 36.0 / 42.7 %), `ha-immigration` (0.5 to 2 immigrants: 77.9 down to
70.5 %), `ha-lattice` (25 to 100 wide: 64.4, 75.9, 76.6, 76.5 %), `jansson-tag-mutation` (0.5 % to
90 %: humanitarians pass ethnocentrics between 25 % and 30 %) and `jansson-markers` (kin strategies
with 4 to 40 colors, the basis mutating or fixed).

Agents are drawn by **Strategy** (the default: ethnocentric green, humanitarian blue, selfish red,
traitorous yellow, kin purple, non-kin orange, other each-color patterns gray), **Tag** (the code's
blue, red, green and yellow, then up to 40 hues), **Lineage** (a color per founding immigrant) or
**PTR** (this period's, as heat), with empty sites dark. Inspect shows an agent's tag, strategy (and,
with kin strategies, what it judges by), this period's PTR and helps, its lineage, kin marker and
age, and each neighbor's tag and strategy, whether they are related, and who helped whom; an empty
site says so. Charts: **Strategies**, **Cooperation** (helps per decision and the share of decisions
toward the same tag), **Population** and **Kin** (related neighbors, help to relatives, and the two
conditional probabilities of Jansson's Table 4), against the period. A run stops at period 2,000
(`end`; 0 for never); immigration, PTR, cost, benefit, death, the mutation rates, pair play,
misperception and offspring placement apply to the running world. **Compare** entries: "Four colors
vs five (the Java's draw) — Ethnocentrism (Compare)" (`ha-standard` and `ha-java-five-colors`),
"Next to the parent vs anywhere — Ethnocentrism (Compare)" (`ha-standard` and
`jansson-offspring-anywhere`: 75.9 % ethnocentric against 8.3 %) and "Tags vs kin — Ethnocentrism
(Compare)" (`ha-standard` and `jansson-kin`). Credit: Ross A. Hammond and Robert Axelrod, "The
Evolution of Ethnocentrism," *Journal of Conflict Resolution* 50(6) (2006), 926–936, and their
archived Java/Ascape code (2003); Uri Wilensky's NetLogo *Ethnocentrism*, the replication they cite;
Thomas R. Shultz, Max Hartshorn and Ross A. Hammond, "Stages in the evolution of ethnocentrism,"
*CogSci 2008*; Thomas R. Shultz, Max Hartshorn and Artem Kaznatcheev, "Why is ethnocentrism more
common than humanitarianism?", *CogSci 2009*; Max Hartshorn, Artem Kaznatcheev and Thomas R. Shultz,
"The Evolutionary Dominance of Ethnocentric Cooperation," *JASSS* 16(3) 7 (2013); and Fredrik
Jansson, "Pitfalls in Spatial Modelling of Ethnocentrism: A Simulation Analysis of the Model of
Hammond and Axelrod," *JASSS* 16(3) 2 (2013). See
`docs/superpowers/specs/2026-09-25-ethnocentrism-design.md`.

### Bounded Confidence (Hegselmann & Krause 2002)

Opinions between 0 and 1. Each period every agent moves to the mean of the opinions within its
reach, its own included; everything further away is ignored. The defaults are the paper's: 625
opinions drawn uniformly, updated all at once, each agent reaching ε = 0.15 either way. Agents who
cannot reach each other drift apart for good, so the profile freezes into camps — many with little
confidence (plurality), two or three in between (polarization), one with much (consensus). A period
is stable when no opinion moves more than 10⁻¹⁰; opinions within 10⁻⁶ count as one.

What reproduces (20 seeds unless stated): Fig. 2a's "exactly 38" surviving opinions at ε = 0.01
(median 37.5); consensus at 0.25; Fig. 3's walk along ε — plurality, then camps, then a consensus
that takes over between 0.21 and 0.25 (50 runs: 13, 30 and 50 in consensus at 0.21, 0.22, 0.25),
always above 0.4; the evenly spaced figures exactly (50 opinions at 0.2 split in period 6 and are
still from period 8; 100 at 0.05 split 8 times; 100 at 0.25 agree). What does not quite: **Fig. 2b's
two camps at ε = 0.15, which the paper calls "a fairly typical result", come up in about a third of
runs** (16 of 50 end with exactly two opinions); most keep a third camp in the middle, about 30 % of
the agents — a central ridge the paper's own Fig. 3 shows as well, and a separate implementation
gives the same (two camps are the rule only from 0.16 to 0.21); and "less than 15 periods to a stable
pattern" holds for 53 runs of 60, the slowest taking 168 while two nearly merged camps close.

The paper's asymmetries are settings. **Asymmetric** confidence, the same for everyone (§4.2.1):
the mean drifts toward the side agents listen to (at εr = 0.2, from 0.53 with εl = 0.18 to 0.94
with εl = 0.02), and one-sided splits — a gap one side reaches across and the other does not —
close again, as the paper says two-sided ones never do. Confidence **leaning with one's opinion**
(§4.2.2, bias m): camps grow and move outward, reaching 0 and 1 at m = 1; at ε = 0.6 consensus holds
to m = 0.36 and breaks between 0.44 (18 of 20 runs in consensus) and 0.52 (8 of 20) — the paper
says "m ≈ 0.4" — and takes longer before it breaks.

Two of the paper's claims have no figure, so they are switches here. **Updating**: "none of the
results … depends crucially on simultaneous updating", without saying which serial order — each
agent once per period in random order, or n random draws. Measured, the phases keep their places
under both, and serial updating leaves slightly more opinions at small ε (at ε = 0.05, 8.4 with a
shuffled order and 9.0 with random draws against 7.9, 50 runs), as the paper says. **Who listens to whom**: on a torus where agents hear only their neighbors,
"polarization … disappears". Measured on a 25 × 25 torus, it does: one big camp with dozens of
stranded local minorities, settling only after thousands of periods; a second camp of a fifth of
the agents in 9 of 200 lattice runs (ε 0.1–0.3, both neighborhoods) against 57 of 100 among
everyone. Lorenz (2006) showed that the consensus threshold depends on the number of agents: at
ε = 0.22 consensus in 1 run of 20 with 50 agents, 12 with 1000.

The view is the paper's opinion × time diagram: each agent a line, red where it started at 0 to
magenta at 1 (**Start**; **Opinion** colors by where it is now), gray between neighbors still
within each other's reach, the last 60 periods; with a lattice the torus is drawn to the right.
Inspect a point for its period, opinion and the agents passing (start, reach, how many they
hear), or a site. Charts: Clusters; Largest camps; Mean and median; Splits (two-sided,
one-sided); Change. A run stops when stable; changing confidence or updating resumes it. Presets:
`hk-plurality`, `hk-polarisation`, `hk-consensus`, `hk-regular-50`, `hk-regular-plurality`,
`hk-regular-consensus`, `hk-asym-a`, `hk-asym-b`, `hk-asym-c`, `hk-one-sided` (Fig. 13's caption
says εl = 0.8, read as 0.08), `hk-bias`, `hk-serial`, `hk-lattice`. **Compare** entry:
"Simultaneous vs serial updating — Bounded Confidence (Compare)". Built-in sweeps: `hk-diagonal`,
`hk-asymmetry`, `hk-bias`, `hk-updating`, `hk-lattice`, `hk-population`.

Credit: Rainer Hegselmann and Ulrich Krause, "Opinion Dynamics and Bounded Confidence: Models,
Analysis, and Simulation," *Journal of Artificial Societies and Social Simulation* 5(3) (2002), 2;
Jan Lorenz, "Consensus Strikes Back in the Hegselmann-Krause Model of Continuous Opinion Dynamics
Under Bounded Confidence," *JASSS* 9(1) (2006), 8. See
`docs/superpowers/specs/2026-09-25-bounded-confidence-design.md`.

### Social Structure (Cohen, Riolo & Axelrod 2001)

256 agents each period play four-move Prisoner's Dilemmas (payoffs 3, 0, 5, 1) with four partners.
A strategy is three probabilities: cooperate on the first move (y), after the other cooperated (p,
"friendliness") and after it defected (q; a low q is "provocable"). At the end of a period each agent
copies the best-scoring agent it played if that one did strictly better — misjudging 10 % of the
time — and each of y, p and q has a 10 % chance of Gaussian noise. What changes between runs is the
social structure, who plays whom: fresh random partners every period (**RWR**), four neighbors on a
16 × 16 torus (**2DK**), fixed random neighbors, four each and symmetric (**FRNE**), fixed random
neighbors drawn once, one-way (**FRN**), or FRN with each partner swapped for a random one with
probability x each period (**FFR-x**, the paper's "dial"). The paper's point: what sustains
cooperation is not the torus's clustering but its continuity — "context preservation".

It reproduces closely (30 runs of 2500 periods, as the paper). Table 2's mean payoffs over the last
1000 periods: RWR 1.089, 2DK 2.553, FRNE 2.574, FRN 2.478, FFR-0.1 2.405, FFR-0.3 2.036, FFR-0.5
1.325, against 1.091, 2.557, 2.575, 2.480, 2.385, 2.100, 1.257. The first period averages 2.25 and
every structure collapses before the fixed ones recover (Fig. 1). In the paper's crucial region of
the p–q plane the average p moves −0.012 under RWR and +0.051 under FRN (the paper: −0.016, +0.052),
because under FRN an agent's partners share its friendliness (slope 0.179, F 1087; the paper 0.158,
F 717; not significant under RWR). FRNE does beat 2DK (note 5), 4096 agents behave like 256 (note 1),
and FRNE's fan-out matches Table A1 to within 3 % out to five links. The paper calls FFR-0.3 bi-stable, with
long stretches at high and low p (p. 21); our test of that is our own, and lenient: 25 of 30 runs
spend 50 periods or more both high and low over the whole run (11 of 20 after period 1000).

The paper doesn't define "high cooperation" formally, but p. 20's "9 of our 30 histories reached an
average score over 2.3" (Table 2's RWR 0.30) implies 2.3, and fitting agrees: at 2.3 every row of Table 2's
"Remain High" lands within 0.03 of the paper (FRN 0.940 against 0.942, FFR-0.1 0.843 against 0.844);
2.2 or 2.4 miss by 0.14 and 0.26. **High cooperation at** defaults to 2.3. It also describes its
own method twice, and the two readings are switches. **Strategies start** "evenly distributed …
throughout the strategy space" (the Appendix) or "initialized randomly" (§3.1): no difference.
**Noise on** every agent every period, "regardless of which … is adopted" (the Appendix), or only as
"errors in the actual copying process" (§2; loose where the Appendix is explicit, rather than a
contradiction): these differ — noise only on copying gives FRN 2.530 instead of 2.478, overshoots every
fixed structure by 0.05–0.09 and shrinks FRNE's edge over 2DK by about two thirds (+0.007 over 40 seeds,
against +0.021; still significant). The Appendix's rule matches Table 2 more closely.

The view is the agents as a block of cells (the torus itself under 2DK; index order otherwise) next
to the paper's p–q plane, with the population's average over the last 200 periods as a fading trail
and every agent as a dot. Color modes: **Friendliness** (p), **Provocability** (1 − q), **Payoff**
and **Strategy** (near Tit-for-Tat, Always Defect, Always Cooperate, or mixed). Inspect an agent for
its strategy, payoff, whom it copied and the partners it played (and Follow it), or a point of the
plane for the agents there. Charts: Mean payoff; Cooperation; Strategy (p, q, y); High cooperation;
Copying (and the partners' p slope). Presets: `cra-rwr`, `cra-2dk`, `cra-frne`, `cra-frn`,
`cra-ffr-01`, `cra-ffr-03`, `cra-ffr-05`, `cra-random-start`, `cra-copy-noise`, each stopping at
2500. **Compare** entry: "Random mixing vs fixed random neighbors — Social Structure (Compare)".
Built-in sweeps: `cra-table-2`, `cra-dial`, `cra-threshold`, `cra-noise`, `cra-population`.

Credit: Michael D. Cohen, Rick L. Riolo and Robert Axelrod, "The Role of Social Structure in the
Maintenance of Cooperative Regimes," *Rationality and Society* 13(1) (2001), 5–32. See
`docs/superpowers/specs/2026-09-26-social-structure-design.md`.

### Demographic Prisoner's Dilemma (Epstein 1998, and its replication)

Epstein's demographic Prisoner's Dilemma: 100 agents on a 30 × 30 torus, each with a fixed strategy,
cooperate or defect, and a wealth of 6. In a random order each agent in turn moves to a random
unoccupied site within its vision (one site, von Neumann), plays the Prisoner's Dilemma with each
neighbor (T 6, R 5, P −5, S −6; both players are paid), has an offspring on a free neighboring site
once its wealth reaches 11 (giving it 6 from its own wealth; the offspring keeps the strategy), and
dies when its wealth goes negative. After every cycle N/2 random pairs of agents swap places in the
list. Cooperators find each other, clone into zones of cooperation, and dominate: the published tables
give 779 cooperators and 121 defectors after 500 cycles (Table 9.2, Run 1), and 784 and 99 with a
maximum age of 100 (Table 9.4, Run 2).

**The working paper and the published text disagree** on the rule, and the sources leave much open;
each choice is a switch or a preset. The 1997 working paper moves to "a random site within your
vision" and plays "a random neighbor", once; the 1998 article and *Generative Social Science* (2006)
move to an unoccupied site and play each neighbor (`play`, `dpd-working-paper`). The prose never gives
the initial agents' wealth or states the cloning threshold as a number to reach: the defaults take
the 2006 CD's `Initial Wealth = 6` and `Fission Wealth = 11` ("exceeds 10"). Radax and Rengs (2009)
list six more choices the text leaves open and the defaults read literally: death as wealth goes
negative, even on another agent's turn (`death_timing`), removal at once (`removal`), the endowment
taken from the parent (`endowment_from`), every newborn's age random up to the maximum (`newborn_age`),
asynchronous updating (`updating`) and Epstein's swaps (`shuffle`). No source says whether a newborn
acts in the cycle it is born (`newborns_act`: next cycle), or whether metabolism, "a fixed decrement to
accumulated payoff per cycle", is charged per cycle or, as another sentence says, "after every
interaction" (`metabolism_per`: per cycle). Soup (`pairing: soup`) pairs each agent with a random
other agent and places moves and offspring anywhere.

What reproduces, measured (release, seeds 1–30, Epstein's 30 runs; each run's count at cycle 500
unless noted; t is Radax and Rengs' two-sample test against the source's mean and s.d., |t| < 2.0017
to pass):

- **Cooperation dominates.** Run 1: 729 ± 17 cooperators against 171 ± 17 defectors; Run 2: 695 ± 29
  against 196 ± 28. The ratio stabilizes from about cycle 30, as Epstein says.
- **Soup runs to pure defection** (`dpd-soup`): the last cooperator dies by cycle 8 on average (4–14)
  in 29 of 30 runs; the defectors then kill one another, leaving one agent or nobody.
- **Run 5's cooperation persists through 10,000 cycles** (`dpd-run-5`, 50 % mutation) in 27 of 30
  runs (the other three populations die out entirely): 260 cooperators against 295 defectors over
  cycles 5,001–10,000. At 25 % mutation the means over cycles 1,001–2,000 are 420 and 393 (Epstein:
  "around 350 and … around 400").
- **Table 9.3's pattern**: cooperators dominate when R is near T and die out as R falls, at the same R
  in every row (R ≤ 3 at T = 10, R ≤ 2 at T = 7–9, R = 1 at T ≤ 6).
- **The metabolism equivalence, charged per game.** The payoffs shifted up by 6 with a metabolism of 6
  charged after every game are Run 2 exactly, run for run (`metabolism_per: interaction`); more
  metabolism does mean more cooperators, from 414 at 1 to 661 at 5 per cycle.

What does not, or only partly:

- **Tables 1 and 2.** Both counts are rejected in both runs: 729 / 171 against 779 / 121 (t = 11.9 and
  −11.8), and 695 / 196 against 784 / 99 (t = 11.8 and −14.1) — 50–90 fewer cooperators and 50–100
  more defectors than Epstein's. By cycle 50 the ratio is 4.3 to 1, not "approximately 5 to 1".
- **Radax and Rengs' factorial**, repeated over the six switches (their random-number library
  excluded): no setting reproduces Run 1 and one of 64 reproduces Run 2 (removal at once, death on the
  agent's own turn, a full shuffle: 785 / 110), none both. They found 1 of 128 for Run 1 (a
  synchronous setting they set aside) and 7 for Run 2; six of their seven were run here (the
  random-number library column is dropped), and none reproduces Table 2 (0/6); their best
  (`dpd-rr-best`: 780 / 97 in Repast) gives 703 / 164. The same switches in two implementations give
  different models; their pseudo-code fixes details the text does not (neighbors played in random
  order, deaths checked after all games, "age ≥ maximum").
- **What does reproduce both tables** is three choices no source settles: the working paper's one
  random neighbor a turn, initial agents with no wealth, and newborns acting at once (`dpd-closest`):
  786 / 114 (t = −1.5) and 792 / 101 (t = −1.2). Of 512 combinations of those three choices with the six
  switches, 46 reproduce Run 1 — every one with no initial wealth — 24 Run 2 and 7 both, all 7 with the
  working paper's rule. It is fragile: over seeds 31–60 and 61–90 Run 2's defectors fail (t = −2.6,
  −3.2). The working paper's rule alone (`dpd-working-paper`) is nearer Table 1 but still rejected:
  759 / 141 (t = 4.8).
- **Table 9.3 cell by cell**: of the 90 means (45 payoff vectors, cooperators and defectors) 22 fall
  inside Epstein's 95 % confidence intervals and 51 inside his ranges; both means are inside the
  intervals in 1 cell of 45, and 5 cells pass both t-tests. Along the diagonal (R = T − 1) our
  cooperators run 13–92 below his and our defectors 33–106 above (bar T = 2). Where his populations die
  out entirely (ranges (0, 0)), 1–5 lone defectors survive here: once the cooperators are gone they have
  nobody to play, and with no maximum age and no metabolism nothing kills them. Row (4, 2)'s defector
  interval, "(254, 376)", is a misprint: its mean 265 and s.d. 32 give (253, 277).
- **Run 4 (R = 1) dies out** instead of cycling (`dpd-run-4`): 26 of 30 populations are extinct by
  cycle 500 and all 30 by 2,000, after at most two swings of the cooperators (above 400, then below
  100: 0.6 a run). Epstein's figure shows a cycle every 300–500 cycles. There is no cooperator monopoly
  at R = 1 (nor at R = 5, where all 30 coexist), so the paradox that "cooperators ultimately do better
  with a low payoff (R = 1) than with a high one (R = 5)" has nothing to stand on. No timing setting
  gives sustained cycles.
- **The shifted payoffs (12, 11, 1, 0) never converge to pure defection** (`dpd-shifted`, Fig. 13):
  all 30 runs coexist at 500 (418 cooperators, 478 defectors) and at 2,000 (409, 487). GSS gives the
  settings — "maximum age of 100, zero mutation" — and Run 5's 50 % mutation or Run 1's unlimited
  lives do not converge either. With no negative payoff only old age kills, the lattice stays full
  and everyone can afford to clone. So the premise of the metabolism argument ("in which cooperators
  are annihilated") fails, and charged per cycle, as the chapter's note defines it, a metabolism of 6
  gives another model (`dpd-metabolism`: 644 ± 97 cooperators, 253 ± 97 defectors against Run 2's
  695 ± 29, t = 2.7). The passage calls the pure-defection run "figure 12", which is Run 5's; it
  means Fig. 13.
- **Footnote 27** (T 16, R 11, P 5, S 4, maximum lifetime 10: "an evolution to cooperative monopoly")
  gives a monopoly in 1 of 30 runs by cycle 2,000 and none by 500 (`dpd-footnote-27`); "hiked by ten"
  would make R 15, which gives the same.
- **The coordination game's norm maps** (GSS appendix, `dpd-coordination`: payoffs [1, −3, −3, 1],
  death age 1,000) form where both conventions persist — at cycle 500, 3.6 % of neighboring pairs
  differ, against 35 % if mixed at random — but both persist in only 17 of 30 runs at cycle 500, 16 at
  2,000 and 11 at 5,000.

The survey measures 21 of these claims: 6 hold, 4 are weak and 11 fail. Four built-in sweeps (30
seeds, cooperators at cycle 500): `dpd-payoffs` (Table 9.3: T 2–10 as lines, R 1–9 on the x axis; the
sweep format cannot skip the 36 cells with R ≥ T, which are not Prisoner's Dilemmas and fill with
774–876 cooperators), `dpd-mutation` (Run 2 with mutation 0–50 %: 695 down to 268), `dpd-metabolism`
(the shifted payoffs with metabolism 0–6 per cycle and per game: 418 at none, 644 and 695 at 6) and
`dpd-max-age` (maximum age 10–1,000: 666–735, barely mattering).

Agents are drawn by **Strategy** (the default: cooperators blue, defectors red, as Epstein's),
**Wealth** and **Age** (heat) or **Surrounded** (cooperators all eight of whose neighbors cooperate, in
blue; everyone else dimmed), with empty sites dark. Inspect shows an agent's strategy, wealth, age and
maximum age, whether it is surrounded, this cycle's payoffs and games, and each neighbor with what one
game between them pays each; an empty site says so. Charts: **Population** (cooperators and
defectors), **Cooperator share**, **Surrounded cooperators**, **Wealth** (the mean of each strategy)
and **Births and deaths**, against the cycle. A run never stops by default (`end`: a last cycle, 0
for never); the payoffs, the demography, the timing switches and the pairing apply to the running
world. **Compare** entries: "Working paper vs published rule — Demographic PD (Compare)" (`dpd-run-1`
and `dpd-working-paper`), "Negative payoffs vs shifted with metabolism — Demographic PD (Compare)"
(`dpd-run-2` and `dpd-metabolism`), "Space vs soup — Demographic PD (Compare)" (`dpd-run-1` and
`dpd-soup`) and "Published rule vs closest reading — Demographic PD (Compare)" (`dpd-run-1` and
`dpd-closest`). Credit: Joshua M. Epstein, "Zones of Cooperation in Demographic Prisoner's Dilemma,"
Santa Fe Institute Working Paper 97-12-094 (1997), *Complexity* 4(2) (1998), 36–48, and *Generative
Social Science* (Princeton, 2006), chapter 9 and its appendix; and Andreas Radax and Bernhard Rengs,
"Replication of the Demographic Prisoner's Dilemma," MPRA 14419 (2009), published as "Prospects and
Pitfalls of Statistical Testing: Insights from Replicating the Demographic Prisoner's Dilemma,"
*JASSS* 13(4) 1 (2010). See `docs/superpowers/specs/2026-09-26-demographic-pd-design.md`.

### Norms and Metanorms (Axelrod 1986; Galán & Izquierdo 2005)

Twenty agents, each with a boldness and a vengefulness (eight levels, 0/7 to 7/7). Four times a
generation each gets a chance to defect, seen by each of the others with a random chance S: it
defects when S is below its boldness, gaining 3 and costing everyone else 1. Whoever sees it punishes
with probability equal to their vengefulness — the defector loses 9, the punisher 2. With
**Metanorms**, whoever sees a defection and lets it pass can be seen and punished for that too (the
same −9 and −2, the same vengefulness). Then the more successful breed: one standard deviation above
the mean payoff, two offspring; one below, none; each bit mutates at 1 %.

Axelrod's claims hold at his horizon (100 seeds, generation 100): the norms game ends spread across
his three outcomes and the norm is rarely established (4 runs); metanorms establish it in 92. His
dominance variant (20 strong agents punished less, 10 weak) follows his rule: a group's defections
hurt, and are punished by, only the other group, metapunishment happens within groups, and each group
breeds within itself (p. 1103). **Who a defection touches** switches to everyone, this engine's first
version. With metanorms his claims hold (100 seeds): the weak group is kept from being bold (0.04) and
the strong one less so (0.15), "not so easy" as he says; under the everyone rule both are kept down
(0.02 and 0.06). Without metanorms he says "even members of the stronger group tend to be free riders …
high boldness in both groups": the strong group ends bold in 97 of 100, but the weak group only in about
half (50 of 100 at 0.6 or more; kept below 0.2 in 39, by a strong group left slightly vengeful), a
mean of 0.52. He gives no counts; under the everyone rule both end bold (0.94 and 0.86).
Galán and Izquierdo re-implemented it and ran it longer. Their results reproduce: the norms game
collapses in 99 of 100 runs by 1 000 generations (the ax-norms preset's 100-seed figure); metanorms
decay — 92 of 100 runs established at 100 generations, 52 established and 43 collapsed by 10⁵ (100
seeds, measured in planning; the survey's own 50 seeds: 43 established at 100, 0 collapsed by
1 000, 20 by 10⁵), and at 10⁶ generations 18 of 20 runs had collapsed (8 of 10 in the survey's own
run); the norm collapses far sooner with a mutation rate of 0.001, with meta-payoffs a tenth as
large, or under any of their three other selection rules (random tournament, roulette wheel,
above-the-mean); and with a temptation of 10 it held in every run we measured (they say "almost all").
Galán and Izquierdo stress that this "should not be understood as a critique": Axelrod's conclusions
hold at his horizon, and 1986's computers could not run a million generations.

Axelrod's text is ambiguous in two places Galán and Izquierdo flag, and both are switches. **When every
payoff ties** (so the standard deviation is zero), his text gives "an average individual one
offspring" (p. 1099), so everyone keeps one; Galán and Izquierdo call the case ambiguous (their note 4)
and give everyone two offspring, then remove a random half, which is the engine's default. They warn
this 'can alter the long-term results significantly'. It does: under his words, giving everyone one
offspring keeps the metanorm — 92 of 100 runs established at 10⁵, 3 collapsed (the 100-seed planning
figure). And **how the offspring are brought back to 20** ('For convenience, the number of offspring
is adjusted') is not a convenience: removing the worst parents' copies first and copying the best
collapses all 50 runs by 10⁵, against 20 with random removal (the survey's own 50-seed figure); that
ranked reading is ours, in neither paper. Several further readings come from Galán and Izquierdo, not
Axelrod, and are fixed here: the step rule for offspring (Axelrod fixes only three points on the curve),
the population standard deviation, that the defector can't catch a non-punisher, and that a
metapunisher must have seen the defection. Whether Axelrod's metanorm lasts depends on details his
paper leaves open. The Norms episode films this: at 10⁶ generations the norm has collapsed in 18 of 20
worlds under Galán and Izquierdo's tie reading, and still holds in 14 of 20 under Axelrod's.

The view is the papers' boldness–vengefulness plane — boldness to the right, vengefulness up, each
strategy a cell shaded by how many agents hold it, Galán and Izquierdo's norm-established (green) and
norm-collapsed (red) corners outlined, the population's mean over the last 200 generations as a trail —
and a strip of the agents: boldness and vengefulness bars and a payoff swatch, the strong group first
under dominance. Color modes: **Agents**, **Payoff**, **Group**. Inspect a strategy for the agents
holding it, or an agent for its bits, payoff, events and parent (and Follow it). Charts: Boldness and
vengefulness; Events; Mean payoff; Norm state; By group. Presets: `ax-norms`, `ax-metanorms`,
`ax-dominance`, `ax-dominance-metanorms` (100 generations), `gi-metanorms-long`, `gi-low-mutation`,
`gi-mild-metanorms`, `gi-temptation-10`, `gi-tournament` (20 000). **Compare** entry: "Axelrod’s
selection vs a random tournament — Norms and Metanorms (Compare)". Built-in sweeps: `norms-horizon`,
`norms-mutation`, `norms-meta-payoffs`, `norms-temptation`, `norms-selection`, `norms-readings`,
`norms-dominance`.

Credit: Robert Axelrod, "An Evolutionary Approach to Norms," *American Political Science Review*
80(4) (1986), 1095–1111; José Manuel Galán and Luis R. Izquierdo, "Appearances Can Be Deceiving:
Lessons Learned Re-Implementing Axelrod's 'Evolutionary Approach to Norms'," *JASSS* 8(3) 2 (2005).
See `docs/superpowers/specs/2026-09-26-norms-design.md`.

### Image Scoring (Nowak & Sigmund 1998, and its critics)

Nowak and Sigmund's indirect reciprocity by image scoring: 100 players, each with an image score that
starts every generation at 0. In each generation 125 random donor–recipient pairs are drawn; a donor
with strategy k helps if the recipient's score is at least k (cost 0.1 to the donor, benefit 1 to the
recipient), and helping raises the donor's score by one and refusing lowers it, within −5 … +5. k runs
from −5 (always help) to +6 (never), drawn at random at the start; players leave offspring in
proportion to their payoffs, and an offspring may mutate to another strategy. A tick is a generation.
Leimar and Hammerstein (2001) put the same game in an island model (100 groups of 100, a parent drawn
from the offspring's own group with probability p), add execution and perception errors, strategies
that look only at their own score (h), both scores (AND, OR), q strategies and Sugden's standing, and
argue that image scoring is not evolutionarily stable.

**The sources leave choices open;** each is a switch or a preset. "To avoid negative payoffs we add 0.1
in each interaction": Leimar and Hammerstein say c is added to donor and recipient every round, as does
FAIR23's NetLogo model, and the default follows them (`offset: none` is our ablation, `ns-no-offset`).
Scores are clamped at ±5 (`clamp`, 0 for none), reproduction is payoff-proportional roulette and
mutation is uniform over the allowed strategies. With observers (Fig. 3: "each interaction is
observed, on average, by 10 randomly chosen players"), the recipient always sees and each other member
with probability 10/(n − 2) (`observers`), and each keeps its own record, 0 until it has seen someone
act; NS98 say only that onlookers "update their perception", so how an observer records is a switch:
its own tally one up or down (`records: tally`, FAIR23's, the default) or the donor's new score, the
whole score at one sighting (`records: score`). Rounds are fixed or random (`rounds_kind`: each the
last with probability 1/m, for Leimar and Hammerstein's analysis). The classes a run allows
(`strategies`: k, h, AND, OR, own score only, binary scorers, standing, q) and its start (uniform, or
everyone on one strategy with an invader at a share of each group) come from presets, files and links.

What reproduces, measured (release, seeds 1–10 unless noted; a window is the mean over the stated
generations, per seed, then over seeds; NS98 averaged over 10⁷ generations and Leimar and Hammerstein
over 10⁵–10⁶, so these runs are 10–100 times shorter):

- **The universal constant, to every printed digit, and its unstated start.** With everyone at k = 0
  and unbounded scores, NS98 give 0.7380294688360… as the largest fraction below 0 from which the
  population still reaches all-out cooperation, without saying how the scores start. All the negatives
  at −1 and the rest so high that they never fall below 0 gives 0.7380294688360038. Any finite start
  lower than about +80 gives less: 0.5 with the rest at 0, 0.642 at +1, 0.688 at +2; spreading the
  negatives over −1 … −5 gives 0.338.
- **Fig. 2's endless cycles** (`ns-fig-2`: m 300, mutation 0.001): 172 collapses of cooperation (k ≤ 0
  falling from at least 90 % to at most 10 %) and 167 recoveries in 10⁶ generations (seeds 1–10 ×
  10⁵), 12–24 per seed; the unconditional cooperators (k ≤ −4) hold 68 % of the population over the 51
  generations up to a collapse's last cooperative generation against 8 % in cooperative phases, as NS98
  describe. Cooperative strategies hold 67 % of the time.
- **Fig. 3's group-size effect, with each observer's own tally** (`ns-fig-3-n20`, `-n50`, `-n100`: ten
  observers, m = 10n): cooperative strategies 86 %, 44 % and 20 % of generations 1,001–20,000 at n =
  20, 50 and 100, against NS98's 90 %, 47 % and 18 % (to 100,000 at n = 20: 91 %). The survey holds the
  fall with n and grades each size Weak, its seeds too noisy to pin the percentage.
- **Fig. 4a** (`ns-fig-4a`, AND strategies): 53 % of interactions cooperative (NS98 55 %) over
  generations 1,001–50,000, with (k 0, h 1) the most frequent strategy (22 %; Weak in the survey).
- **The Methods' thresholds**: discriminators are stable against defectors above (bq + c)/(bq − c) =
  1.2222 rounds ("about 1.2"), and only when q > c/b; with cooperators, defectors win below c(2 −
  w)/(bwq) (0.1222 at w = 0.9).
- **Leimar and Hammerstein's Fig. 1a**: h = 1 invades a population of k = 0 (`lh-fig-1a`: 37 % by
  generation 50, fixed by 150 in every run; faster than their 80 % at 150). **Fig. 2a** (one group, AND
  strategies, c 0.25): help in 37 % of rounds (they report 39 %), (k 0, h 1) the most frequent (38 %;
  Weak in the survey). **Fig. 3b's help rate** with q strategies: 17 % (15 %) over generations
  1,001–3,000 (to 10,000: 15 %; Weak in the survey, which also judges the q share below).
- **Standing** (Fig. 4): it invades binary discriminators (`lh-fig-4a`: 69 % by generation 500, 98 % by
  1,000) and still does with perception errors (`lh-fig-4b`: 35 % and 75 %, seeds 1–5), and from a
  uniform start dominates with cooperators beside it (`lh-fig-4c`, seeds 1–3, generations 1,001–1,500:
  standing 53 %, cooperators 39 %, discriminators 8 %, defectors 0.02 %) — although their condition for
  standing with perception errors, vrb < c < rb, is not met at these parameters (v = 0.5, r = 0.833, vrb
  = 0.417 > c = 0.25), as they say.

What does not, or only partly:

- **Fig. 1's victory of k = 0 is a minority outcome.** NS98 show one run in which k = 0 is fixed after
  166 generations. Of seeds 1–100 run to fixation, k = 0 wins 20 (median generation 56) and some k ≤ 0
  40; defection wins the other 60. More rounds do help, as NS98 say: some k ≤ 0 wins 91 of 100 at m =
  300 and all 100 at m = 1,000.
- **"It suffices that each player is chosen only for about 2 interactions per life-time"** (m ≈ n):
  with Fig. 2's settings, cooperative strategies hold 18 % of generations 1,001–20,000 at m = 100 and
  reach half only at m = 200, four interactions per lifetime (the sweep `ns-rounds`).
- **Fig. 3 depends on how an observer records.** If one sighting revealed the donor's whole score
  (`records: score`), n = 20, 50 and 100 would cooperate alike: 97 %, 93 %, 92 %. And FAIR23's fixed
  visibility (each member sees with probability 0.1: 1.8 and 4.8 observers at n = 20 and 50, not ten)
  flattens it the other way: 28 % and 16 %.
- **Fig. 4's other panels**: 53 % (NS98 57 %) in 4b, with (k 0, h 5) most frequent (NS98 (k 0, h 4));
  78 % and 85 % (70 %, 80 %) in 4c and 4d, where the most frequent strategies are cooperative ORs, (k 3,
  h 4) and (k 2, h 5), and NS98's defectors (k 6, h −5) come second and third. 4b and 4d are "as in
  figure 3 with n = 20", so m = 200 (at m = 500, 55 % and 94 %).
- **Own-score strategies help in 0.19 % of rounds** (`ns-own-only`), not "less than 0.1 %": uniform
  mutants keep a floor, 5 of 12 of them (h ≥ 1) helping at a generation's start.
- **The Methods' x_min in a simulation**: over five of the Methods' rounds discriminators should beat
  defectors from a share of 0.123; in NS98's random pairs (n 100, m 250, 2,000 seeds a share) the
  payoff gap turns positive only at 0.16. The Methods' rounds, in which everyone plays once, are not
  random pairs.
- **Leimar and Hammerstein's island model does not undo image scoring here.** Fig. 2b (`lh-fig-2b`: p
  0.9, execution errors 0.02): help in 44 % of rounds over generations 1,001–5,000 (to 20,000: 35 %,
  runs from 2 % to 54 %) against their 9 %; Fig. 2c (p 0.5): 15 % against 2 %. Help is not even
  monotone in gene flow (the sweep `lh-gene-flow`: 27 % in isolated groups, p = 1; 50 % at p = 0.8),
  and at c 0.1–0.25 the island model helps more than one group (`lh-cost`: 50 % against 44 % at c =
  0.1), though at c = 0.05 one group helps more (58 % against 53 %).
- **Fig. 1b's invasion is ten times slower**: h = 1 invades (k 0, h 1) with execution errors 0.05, but
  holds 1.8 % by generation 150, 13 % by 500 and 42 % by 1,000 (`lh-fig-1b`). **Fig. 3a's help rate**
  (`lh-fig-3a`: c 0.1, u₀ 5, p 0.5, execution errors 0.02) is 52 % over generations 1,001–3,000 against
  their 45 %, outside the survey's 5-point margin, and the survey fails it (to 10,000: 47 %). **Fig.
  3b's q strategies** hold 26 % of the population over generations 1,001–3,000 (18 % to 10,000),
  against their 12 %.
- **The offset lowers cooperation.** Without it (`ns-no-offset`) some k ≤ 0 wins Fig. 1 in 63 runs of
  100 (k = 0 in 10) against 40, and Fig. 2's cooperative strategies hold 78 % of the time against 67 %:
  adding c to both players weakens selection, and cooperation loses by it.

The survey measures 34 of these claims: 14 hold, 8 are weak, 11 fail, and one (Fig. 4c's long run, too slow
for it) is left to the book test. Four built-in sweeps (seeds 1–10):
`ns-rounds` (Fig. 2's settings, m 25–500, cooperative strategies over generations 1,001–20,000: 1 %,
10 %, 8 %, 18 %, 15 %, 23 %, 50 %, 65 %, 91 %), `ns-group-size` (Fig. 3, n 20–100 with m = 10n: 86 %,
83 %, 44 %, 35 %, 20 %), `lh-cost` (c 0.05–0.5, one group against 100 groups, help over generations
1,001–5,000: 58 % and 53 % at 0.05, 35 % and 44 % at 0.25, 6 % and 2 % at 0.5) and `lh-gene-flow` (p
0.5–1: 15 %, 35 %, 39 %, 50 %, 44 %, 27 %).

Each group is a tile of cells, the groups in a grid with a gap between tiles. Agents are drawn by
**Strategy** (the default: k on a blue–red scale from −5 to +6, h, own-score, standing and q
strategies each their own color), **Score** (below 0 red, above blue) or **Payoff** (heat). Inspect shows
a cell's group, its agent's strategy and whether it helps at a generation's start, its score (with
observers, how many others have seen it act and their mean record of it), its standing when standing
plays, its payoff and the help it gave and received; a gap or a tile's unused cell says so. An agent
lives one generation, so a followed agent's offspring are new agents. Charts, against the generation:
**Help rate** (with the share of cooperative strategies), **Mean k**, **Strategy shares** (the binary
scorers and standing in their own chart) and **Mean payoff**. A run never stops by default (`end`: a
last generation, 0 for never); the game, the rounds, the observers, the records, the errors, mutation
and `local` apply to the running world from the next generation. **Compare** entries: "One group vs
the island model — Image Scoring (Compare)" (`lh-fig-2a` and `lh-fig-2b`), "Image scoring vs standing
— Image Scoring (Compare)" (`lh-fig-2b` and `lh-fig-4c`: the island model with AND strategies or with
standing), "With vs without the offset — Image Scoring (Compare)" (`ns-fig-1` and `ns-no-offset`) and
"Small vs large groups with observers — Image Scoring (Compare)" (`ns-fig-3-n20` and
`ns-fig-3-n100`). Credit: Martin A. Nowak and Karl Sigmund, "Evolution of indirect reciprocity by
image scoring," *Nature* 393 (1998), 573–577 (IIASA IR-98-040, with "The Dynamics of Indirect
Reciprocity," *J. Theor. Biol.* 194 (1998), 561–574); Olof Leimar and Peter Hammerstein, "Evolution of
cooperation through indirect reciprocity," *Proc. R. Soc. Lond. B* 268 (2001), 745–753; Karthik
Panchanathan and Robert Boyd, "A tale of two defectors," *J. Theor. Biol.* 224 (2003), 115–126; Arnon
Lotem, Michael A. Fishman and Lewi Stone, "Evolution of cooperation between individuals," *Nature* 400
(1999), 226–227; Hisashi Ohtsuki and Yoh Iwasa, "The leading eight," *J. Theor. Biol.* 239 (2006),
435–444; and Marco Janssen's NetLogo model of image scoring (2010), republished by Make Models FAIR
(2023), whose donor, with visibility below 1, reads the recipient's record of itself. See
`docs/superpowers/specs/2026-09-26-image-scoring-design.md`.

### Relative Agreement (Deffuant et al. 2000, 2002; Meadows & Cliff 2012)

Random pairs meet, N meetings a period. In Deffuant, Neau, Amblard and Weisbuch's pairwise bounded
confidence (2000), two agents whose opinions differ by less than d each move a fraction μ of the way
toward the other. In Deffuant, Amblard, Weisbuch and Faure's **relative agreement** (2002) each agent
also has an uncertainty — a segment around its opinion — and a partner moves it by μ times the overlap
of their segments beyond half the influencer's width, divided by that width: confident agents sway
uncertain ones, and uncertainties move too. A few **extremists** — the most extreme opinions, very
confident — can then leave the majority in the center, split it between the two extremes, or pull it
all to one extreme. The paper maps which happens with an indicator y (the squared shares of moderates
that end up extremists at each end, summed: 0 central, 0.5 both extremes, 1 a single extreme) over
the moderates' uncertainty U and the share of extremists pe (Fig. 9). Opinions run from −1 to 1;
Deffuant 2000's d on [0, 1] is U = 2d here.

Meadows and Cliff (2012) reimplemented the model twice and could not reproduce Fig. 9; the authors
replied (2013) that Meadows and Cliff measured y before the model converged, and counted too few
moderates as extremists — neither detail is in the 2002 paper. Both readings are presets, and the
paper as stated is the default. Measured (planning and the survey):

- **Both fixes are needed.** At Fig. 9's corner (pe 0.05, U 1.4, 200 agents) Meadows and Cliff's
  reading gives y 0.00; their horizon fixed, 0.40; their cutoff fixed, 0.27; both fixed, 0.98. The
  majority has drifted most of the way to one extreme by their stop and settles between 0.7 and 0.8.
  The paper as stated — the extremists drawn, run until nothing moves — agrees with the reply.
- **The single extreme is a finite-size effect when the extremists are balanced.** At pe 0.1, U 1.6
  it comes in 79 % of runs with 100 agents, 16 % with 1000 and none with 4000; Fig. 9's single-extreme
  zone at its stated 1000 agents covers 14 of the 35 cells it shows at pe ≤ 0.075, U ≥ 1.4, and all 35
  with 200 agents. With a lean (δ 0.1) it holds and strengthens with N. Meadows and Cliff were right
  that it shrinks with N; the reply's 'any large number of agents' holds only at the smallest pe.
- **Figs. 5 and 7 do not reproduce at their stated parameters.** Fig. 5's 'only … 4%' of moderates
  becoming extremists is 48 % (22 % with extremists at ±1). Fig. 7's single extreme — and Fig. 8's
  central convergence 'for the same parameters' — never appear at the stated μ = 0.5: both extremes in
  39 of 40 runs. At Fig. 9's μ = 0.2 the pair appears (single 24, central 16), as §4.8's own μ result
  predicts.
- **Eq. 11 as printed reproduces none of §6.** The bounded-confidence window 'If |x − x′| < u′', u′
  being the influencer's uncertainty, lets uncertain moderates pull the confident extremists in: y is
  0.00 everywhere. With the listener's own uncertainty (**BC window** switch) §6's claims hold: a single
  extreme only around U = 1, a central band at U 0.8 with averaged uncertainties, none with the variance
  rule.
- **ue matters, and the cutoff misreads it.** §4.8 finds no influence of the extremists' uncertainty;
  at N 1000 (pe 0.05, U 1.4) the population drifts to one extreme far less often at ue 0.05 than at 0.2
  (median |mean opinion| 0.03 against 0.75) — and at 0.2 the
  extreme cluster settles at ±0.75, inside the reply's 'innermost extremist less 0.1', so y reads 0.
- **Networks.** On a Moore lattice there is never a single extreme (Amblard and Deffuant, 2004). On
  small-world rings it needs a critical number of neighbors that falls as rewiring rises — as they say,
  but at k 32 to 256 (most runs single from k 32 at p 0.8, 64 at p 1, 256 at p 0.2) rather than
  'around 8' — and whether sparse rings end in both extremes or the center
  depends, again, on the unstated cutoff. Weisbuch's scale-free networks (2004) reproduce: no steps in
  the dispersion, close to the square lattice, closer to well mixed with twice the links; hubs end in
  the big cluster; 16 % of agents never move. Deffuant 2000's lattice picture appears only when run to
  stability (a median period of about 1 800), not after the caption's '100 000 iterations' (119
  periods).

Switches for what the papers leave open: **Placement** (the most extreme draws; set to ±1; Meadows and
Cliff's band), **New-extremist margin** (the reply's 0.1; 0 for Meadows and Cliff), **A meeting
updates** (both from their old values; one after the other; only the first, as Weisbuch), **Pairs** on
a network (a random link; an agent then a neighbor), **BC window**, and the stop (**Stop when stable**,
**Stop at period**). Networks: anyone, a lattice (four or eight neighbors), a small world grown from a
ring or the lattice, a scale-free network.

The view has three panels: **opinion × time** (lines are agents, +1 at the top; the history halves
when full, so a run of any length stays in view), **start against now** (each agent a dot; the diagonal
marks those that never moved) and, on a lattice, the **torus** colored by opinion. Color modes:
**Uncertainty** (confident red to uncertain green, the papers' coloring), **Role** (the initial
extremists by side), **Start**. Inspect a column, dot or site for the agents there. Charts:
Convergence (y, p₊, p₋); Clusters; Dispersion; Opinion and uncertainty; Change. Presets:
`dnaw-consensus`, `dnaw-clusters`, `dnaw-lattice`, `dnaw-lattice-clusters`, `ra-uniform`, `ra-central`,
`ra-both`, `ra-single`, `ra-literal`, `ra-meadows-cliff`, `ra-deffuant-2013`, `ra-bc-extremists`,
`ra-bc-printed`, `ad-moore`, `ad-small-world`, `w-scale-free`. **Compare** entry: "Meadows and Cliff vs
Deffuant et al.’s reply — Relative Agreement (Compare)". Built-in sweeps: `ra-clusters`, `ra-map`,
`ra-readings`, `ra-population`, `ra-rules`, `ra-delta`, `ad-connectivity`, `w-dispersion`.

Credit: Guillaume Deffuant, David Neau, Frédéric Amblard and Gérard Weisbuch, "Mixing Beliefs Among
Interacting Agents," *Advances in Complex Systems* 3 (2000), 87–98; Guillaume Deffuant, Frédéric
Amblard, Gérard Weisbuch and Thierry Faure, "How Can Extremism Prevail? A Study Based on the Relative
Agreement Interaction Model," *JASSS* 5(4) 1 (2002); Frédéric Amblard and Guillaume Deffuant, "The
Role of Network Topology on Extremism Propagation with the Relative Agreement Opinion Dynamics,"
*Physica A* 343 (2004); Gérard Weisbuch, "Bounded Confidence and Social Networks," *European Physical
Journal B* 38 (2004); Michael Meadows and Dave Cliff, "Reexamining the Relative Agreement Model of
Opinion Dynamics," *JASSS* 15(4) 4 (2012); Guillaume Deffuant, Frédéric Amblard and Gérard Weisbuch,
"The Results of Meadows and Cliff Are Wrong Because They Compute Indicator y Before Model
Convergence," *JASSS* 16(1) 11 (2013). See `docs/superpowers/specs/2026-09-26-relative-agreement-design.md`.

### El Farol and the Minority Game (Arthur 1994; Challet & Zhang 1997)

**El Farol.** 100 people decide each week whether to go to a bar that is fun only if fewer than 60
come (Arthur). Nobody knows who else is going; each holds a few simple forecasts of next week's
attendance from past weeks' (the same as some week ago, a mirror image, an average, a trend — a
stated library of 48, Arthur's "several dozen") and goes if the forecast that has lately been most
accurate says fewer than 60. **The minority game** (Challet and Zhang) strips this to its core: an odd
number of players each choose side A or B, and the smaller side wins. Each player holds a few
strategies — tables from the last M winning sides to a choice — and plays the one that would have won
most so far. With a capacity other than half, the minority game is also Challet, Marsili and Ottino's
yes-or-no version of El Farol.

Measured (the survey and the presets' descriptions):

- **The mean at 60 is trivial; the swings are not.** Arthur's attendance does average 58–60 at every
  k from 2 to 32, as he says — but agents going at random with probability 0.6 average 60 too
  (Challet, Marsili and Ottino's point), and Arthur's agents swing 20 to 50 times as widely (σ²/N
  5–11 against 0.24). Rated by the advice they give instead of their accuracy, the swings halve, but
  stay well above chance.
- **Arthur's robustness needs a large library.** The library here is fixed at 48, and his numbers
  hold across k; but in planning, a library of 25 gave a mean of 52 at k 23, with attendance swinging
  between near-empty and near-full (σ²/N 22). His "robust to changes in types of predictors created"
  is not tested here, and on that evidence it would not hold for small libraries.
- **Arthur's cycles do not go away.** Rated by accuracy, attendance alternates high and low (a lag-1
  autocorrelation of −0.23 to −0.57) where he says "no persistent cycles"; rated by payoff the cycle
  is gone. And 30–36 % of the forecasts in use are above 60, not his 40 %.
- **The memory transition reproduces** (Savit, Manuca and Riolo): with 2 strategies, fluctuations are
  worst with short memories, lowest where 2^M/N ≈ 0.63 and back to chance with long ones, and the
  minimum moves one memory step per doubling of N. Players beat a coin just above the transition
  (the best of 101 wins 54 % at M 6), not far above it (49 % at M 10), where the paper says they
  still do.
- **Challet and Zhang's figures mostly hold.** Fluctuations fall through memories 6, 8 and 10 at
  1001 players (Fig. 1 — with 5 strategies each; with 2, M 10 is already past the minimum); mixed
  memories win more up to about 6, then level off (Fig. 2); more strategies make players worse
  (Fig. 5); frequent switchers do worse (Fig. 6); the Darwinian version cuts fluctuations (Fig. 9);
  memory evolves upward and settles, higher for 1001 players than 101 (Fig. 11). Two do not: the
  "win more, the smaller the minority" payoff gives one peak, not two (Fig. 4 — rounded as stated,
  an even split pays nothing and nothing is learned; unrounded, still one peak), and cloning without
  mutation gives no "tremendous waste" (Fig. 10).

Switches: **Game**; **Agents decide** (by their best strategy, or at random); **Predictors rated by**
(accuracy, or the advice they give), **Accuracy memory**, **Forecast exactly L**, **Everyone holds
the whole library** (El Farol); **Memory**, **Mixed memories**, **Winners get** (a point, or N/x − 2),
**N/x − 2** (rounded, or exact), **Bias**, **History** (the real one, or random), and **Replace the
worst** with its interval and mutations (the minority game). The view: attendance over the last 240
rounds (crowded rounds shaded, the capacity marked), the attendance histogram on the same scale, and
the agents as a grid. Color modes: **Choice**, **Gain**, **Strategy**, **Memory**. Charts: Attendance;
Fluctuations (with coin-flippers' level); Success; Forecasts; Switching; Memory. Presets: `ef-arthur`,
`ef-payoff`, `ef-random`, `ef-shared`, `mg-m6`, `mg-m8`, `mg-m10`, `mg-mixed`, `mg-inverse`,
`mg-evolution`, `mg-inbred`, `mg-arms-race`, `mg-crowded`, `mg-critical`, `mg-random-like`,
`cmo-binary`. **Compare** entry: "Accuracy vs payoff scoring — El Farol (Compare)". Built-in sweeps:
`ef-predictors`, `ef-capacity`, `mg-memory`, `mg-fig-1`, `mg-strategies`, `mg-information`, `cmo-bias`.

Credit: W. Brian Arthur, "Inductive Reasoning and Bounded Rationality," *American Economic Review*
84(2) (1994), 406–411; Damien Challet and Yi-Cheng Zhang, "Emergence of Cooperation and Organization
in an Evolutionary Game," *Physica A* 246 (1997); Robert Savit, Radu Manuca and Rick Riolo, "Adaptive
Competition, Market Efficiency, and Phase Transitions," *Physical Review Letters* 82 (1999); Damien
Challet and Yi-Cheng Zhang, "On the Minority Game: Analytical and Numerical Studies," *Physica A* 256
(1998); Damien Challet, Matteo Marsili and Gabriele Ottino, "Shedding Light on El Farol," *Physica A*
332 (2004). See `docs/superpowers/specs/2026-09-27-el-farol-design.md`.

### Ants and Recruitment (Kirman 1993; Alfarano & Milaković 2007)

**The ants.** Entomologists gave a colony two identical, constantly refilled food sources and found
the ants crowding one — "some 80 percent at one source and 20 percent at the other" — and then, now
and then, flipping to the other. Kirman's explanation needs no difference between the sources: each
time two ants meet, the first joins the second's source with probability 1 − δ, and now and then
(probability ε) an ant switches on its own. The state is just how many ants are at each source, and
its long-run distribution follows exactly from the chain. **Alfarano and Milaković** rebuilt the
chain from individual agents — each switches at a rate that grows with its neighbors at the other
source — and asked what the network of who meets whom does to it.

Measured (the survey and the presets' descriptions):

- **The chain reproduces exactly — but not the ants' 80–20.** Its long-run distribution is the
  beta-binomial with α = ε(N − 1)/(1 − δ): U-shaped below Kirman's threshold ε = (1 − δ)/(N − 1),
  flat at it (to 10⁻¹⁷), centered above; long runs match it within total variation 0.004–0.011.
  But it never peaks near 80–20 at any ε and δ: it piles up at 0 and 100 %, is flat, or centers.
  **Becker's majority pull**, which Kirman suggests but does not run, does it: with recruiting scaled
  by the recruiter's lead, Figure Ic's settings peak at 18 % and 82 %.
- **Figure IIb's "average … about one-half" needs a hundred times the figure.** Over its 100 000
  meetings the colony flips 0–4 times, and the time average is between 0.4 and 0.6 in 4 runs of 20
  (0.15 to 0.95); over 10⁷ meetings, in all 20. The rest of Figure II holds: little time near half,
  77 % of it with one source at 80 % or more, switches taking 8 % of a regime, and the time to the
  next switch not depending on how long the colony has held one source.
- **A majority is less likely to shrink the larger it is — only while recruiting is strong.** At
  Figure Ic's weak recruiting a slight majority is at first more likely to shrink as it grows.
- **More sources change nothing, as Kirman says**: with 2 to 6 sources one holds 80 % or more 77–79 %
  of the time.
- **Herding fades as the colony grows** (Alfarano and Milaković's N-dependence): at Figure IIb's ε and
  δ, one source holds 80 % 79 % of the time with 100 ants, 21 % with 1 000. Under their rule a random
  network cures it — the variance stays flat from 50 to 1 050 ants (inverse-variance slope 0.004; theirs:
  indistinguishable from 0) while it falls on rings, small worlds and networks with hubs (slopes 0.52,
  0.43, 0.38; theirs 0.51, 0.51, 0.40). Under Kirman's pairwise meetings a random network does not
  cure it: a meeting is one partner however many an ant knows.
- **Their mean field fails on rings.** "Irrespective of the underlying network structure" holds for
  random and scale-free networks (variance within 8 % of the Beta), not for the ring or the small
  world, whose variance falls 19–34 % short: neighbors agree with each other.
- **A few who never herd calm everyone** (their Fig. 6): 5 % of ants who never herd, on the network,
  leave a third of the variance the same ants off the network would.

Switches: **Ants**, **Ants change source by** (meeting another ant, or counting their neighbors),
**Start**, **Food sources** (2–6), **Self-conversion (ε)**, **Resistance to recruiting (δ)**, **ε and δ
combine** (eq. 1, or footnote 9), **Meetings per step**, **Majority pull** (Becker), **a** and **λ**
(Alfarano and Milaković), **Who meets whom** (anyone, a ring, a ring with shortcuts, a random network, a
network with hubs), **Degree**, **Link probability** and **Ants who never herd**. The view: the share at
the first source over the last 400 steps (one line per source with more than two), the time spent at
each share with theory's long-run distribution as dots (Kirman's exact chain, or Alfarano and
Milaković's mean field), and the ants as a grid. Color modes: **Source**, **Independent**, **Degree**.
Charts: Share; Variance (with theory's); Flips; Extremes. Presets: `ants-1a`, `ants-1b`, `ants-1c`,
`ants-2a`, `ants-2b`, `ants-crowd`, `ants-becker`, `ants-lock`, `ants-three`, `am-ring`, `am-random`,
`am-scale-free`, `am-independent`. **Compare** entry: "Colony size — Ants (Compare)". Built-in sweeps:
`ants-alpha`, `ants-n`, `ants-flips`, `ants-pull`, `ants-sources`, `am-independent`.

Credit: Alan Kirman, "Ants, Rationality, and Recruitment," *Quarterly Journal of Economics* 108(1)
(1993), 137–156; Simone Alfarano and Mishael Milaković, "Should Network Structure Matter in
Agent-Based Finance?", Warwick working paper WP07-02 (2007), published as "Network Structure and
N-Dependence in Agent-Based Herding Models," *Journal of Economic Dynamics and Control* 33(1) (2009),
78–92. See `docs/superpowers/specs/2026-09-27-ants-design.md`.

### Minds 1: the utility mind and the ideal free distribution

This is our own experiment, not a reproduction: the first step of the Minds program
(`docs/studies/2026-09-27-minds.md`), which builds decision engines one at a time and tests each
against a known answer and an attested regularity before any social result rests on it.

**The seam.** A new switch, **decision.rule**, picks what decides where an agent moves. `book` (the
default) is rule M, unchanged. `utility` is a **utility mind** in the style of Mark and Lewis: each
candidate site at distance d scores W × T(d) × C, the product of three considerations.

- **W** is rule M's welfare: sugar (discounted by pollution when on), or foresight welfare over
  several goods.
- **Travel**, T(d) = 1 / (1 + k·d): hyperbolic discounting of distance.
- **Crowding**, C = (1 + n)^−m, with n the agents on the site's four neighbors, not counting the
  mover: Sutherland's interference, made local.
- **Idle**: when every candidate scores 0, `stay` (the book) or `wander` to a random free site in
  sight.

W stays raw (Lewis normalizes each consideration, but a constant shared by every candidate does not
change the order), and ties and draws are rule M's. So with k = 0, m = 0 and `stay` the utility mind
*is* rule M: every book-rule Sugarscape preset without combat gives the same fingerprint under
either rule. Rule C still decides moves under combat, and `utility` with combat on is an error.

**The target.** In the **ideal free distribution** (Fretwell and Lucas, 1969), animals that know
every patch and can move freely spread so that none can do better by switching. With continuous
input, the counts on two patches match their input rates (Parker's input matching). Experiments
mostly **undermatch**: the richer patch draws fewer than its share (Kennedy and Gray, 1993).
Fit N₁/N₂ = b·(R₁/R₂)^s: s = 1 is matching, s < 1 undermatching. Following Earn and Johnstone
(1997), s here comes from means of per-sample log ratios (ln N₁/N₂ every 10 ticks from 500 to
1000), never from the ratio of mean counts, which makes the theory appear to underestimate the
consumers on poorer patches. Each seed gets its own s, the slope across five input ratios (1, 1.36,
2.10, 2.80, 4.42).

The world: two cone patches of sugar on a 60 × 40 torus, growback 0.25, and 100 agents of
metabolism 1. An agent is on a patch when it is inside the radius of its nearest peak.

Measured (20 seeds, tick 1000; the survey and the sweeps):

- **Rule M undermatches, and Parker's matching fails.** s has median 0.72 at vision 1–6 (IQR
  0.59–0.92; 4 of 20 seeds within 0.9–1.1), 0.63 at 5–10 and 0.90 at 10–20 (IQR 0.87–0.93; 8 of
  20). Every seed undermatches at 5–10 and 10–20, and 17 of 20 at 1–6. s does not rise steadily
  with vision: 5–10 is the lowest. At 2.10 : 1 the richer patch holds 63 %, 61 % and 65 % of the
  on-patch agents at the three visions, against matching's 68 %.
- **"Free" fails before "ideal" does.** Rule M keeps an agent in place when nothing it sees is
  better, so an agent that starts out of sight of sugar never moves. With nobody starving, a median
  66 of 100 are off both patches at tick 1000, over half in all 20 seeds.
- **Survival plays no part.** Starving or not, s is identical seed by seed in all 20 seeds. The
  agents who die are the ones who never find sugar.
- **Wandering frees the agents but moves s away from matching.** Under `wander` a median 0.5 % are
  off patch. But s falls from 0.72 to 0.40, so the claim that wandering moves s toward 1 fails.
  The wanderers overfill the poorer patch (41 agents on its 36 sugar a tick), so the split likely
  follows where they arrive, not the inputs. That cause is not measured.
- **Catchment is close on the median, but fails as judged.** At vision 1–6 no agent can see across
  the gap between the patches, so a patch's count may be set by how many agents start within sight
  of it. Counting those sites predicts s = 0.709. The seeds' median is 0.722, but they scatter, and
  only 6 of 20 land within 0.1 of the prediction (the claim asks for 80 %).
- **Sutherland holds, in the direction we did not expect.** At vision 10–20 under the utility mind,
  crowding m = 1 gives s median 0.95 (18 of 20 seeds within 0.9–1.1), as Sutherland's m = 1
  predicts. We expected local crowding only to push agents apart and lower s. Instead it raises s
  from 0.90 (one-sided Mann–Whitney p = 0.0003). In the sweep the share rises from 0.649 at m 0 to
  about 0.659 at m 0.5 and stays there to m 4.
- **Baum and Kraft fail here.** They found that travel between patches slightly reduced
  undermatching. At vision 10–20, travel k = 0.5 lowers s from 0.90 to 0.73. Their travel was a
  cost of switching; ours is a preference for nearby sugar under rule M's one-tick jump, which is
  not the same thing. In the sweep the share falls from 0.649 at k 0 to 0.602 at k 0.5, then
  recovers a little (0.613 at k 2).
- **At vision 1–6, crowding and travel do nothing to the split.** No agent sees both patches, so
  there is nothing to weigh. A 10-seed check during implementation (2.10 : 1, crowding 0, 1 and 4,
  travel 0, 0.5 and 2, ticks 500–1000) found the share unchanged (0.637) at every value. That is
  why their presets, sweeps and claims use vision 10–20.

Switches (the Rules panel's **Decision (Minds 1)** group): **Rule** (rule M or the utility mind;
rebuilds the world), **Travel k** and **Crowding m** (0–10), and **When nothing in sight scores**
(stay or wander). Under rule M the last three are kept but ignored. On a map of two or more peaks
two charts appear: **Patches** (agents on the first patch, on other patches and off patch) and
**First patch share**. Presets: `ifd-even`, `ifd-two-to-one`, `ifd-four-to-one`, `ifd-far-sighted`,
`ifd-no-starving`, `ifd-wander`, `ifd-crowding`, `ifd-travel`. Built-in sweeps: `ifd-matching`
(share against input ratio at three visions), `ifd-idle` (stay against wander), `ifd-crowding`,
`ifd-travel`.

Credit: S. D. Fretwell and H. L. Lucas, "On Territorial Behavior and Other Factors Influencing
Habitat Distribution in Birds," *Acta Biotheoretica* 19 (1969); G. A. Parker, "Searching for Mates,"
in *Behavioural Ecology* (1978); M. Kennedy and R. D. Gray, "Can Ecological Theory Predict the
Distribution of Foraging Animals?," *Oikos* 68 (1993); D. J. D. Earn and R. A. Johnstone, "A
Systematic Error in Tests of Ideal Free Theory," *Proc. R. Soc. B* 264 (1997); W. J. Sutherland,
"Aggregation and the 'Ideal Free' Distribution," *J. Anim. Ecol.* 52 (1983); W. M. Baum and J. R.
Kraft, "Group Choice: Competition, Travel, and the Ideal Free Distribution," *JEAB* 69 (1998); E. J.
Collins, A. I. Houston and A. Lang, "The Ideal Free Distribution: An Analysis of the Perceptual Limit
Model," *Evol. Ecol. Res.* 4 (2002); D. Mark, *Behavioral Mathematics for Game AI* (2009); M. Lewis,
"Choosing Effective Utility-Based Considerations," *Game AI Pro 3* (2017). See
`docs/superpowers/specs/2026-09-27-minds-1-utility-design.md`.

### Minds 2: A* and walking

This is our own experiment, not a reproduction: the second step of the Minds program
(`docs/studies/2026-09-27-minds.md`). Rule M jumps an agent to the best site in sight in one tick,
however far away it is. Minds 2 adds an A* engine, walls and fences, and a switch that makes agents
walk instead. Then it measures which of the book's results need the jump, and what happens to the
ideal free distribution when switching patches truly costs a walk.

**A\*, verified.** A generic A* (Hart, Nilsson and Raphael, 1968) searches any graph with an
expansion limit. It draws no random numbers. Ties go to the lowest f, then the lowest h, then the
earliest pushed, and the torus pushes its neighbors north, south, east, west. Two checks:

- **Against Dijkstra:** on 1 000 random walled tori (4-way) and 1 000 random octile maps (8-way,
  no corner cutting), with 0–40 % walls, A*'s cost equals Dijkstra's, and both agree when there
  is no path. A case whose start or goal falls on a wall is skipped, so 665 tori and 659 maps
  (1 324 of 2 000) are checked; the tests assert at least 600 of each. The heuristics, torus
  Manhattan and octile distance, are consistent, and a closed set keeps any site from being
  expanded twice. Unit tests pin the tie order and the exact expansion count on a tie case, the
  start = goal case, an unreachable goal and the limit's exact boundary.
- **Against Sturtevant's benchmarks** (2012): one random map (`random512-10-0`, 89 scenarios) and
  one maze (`maze512-4-0`, 106 scenarios) from the Moving AI synthetic sets. A*'s octile cost
  equals each scenario's published optimal length within 1e-6. The subset is in
  `crates/sugarscape-core/tests/fixtures/movingai/` under the Open Data Commons Attribution
  License, with a README giving the source.

**Walls and fences.** `walls` is a list of rectangles (set on reset; presets supply them). A wall
site holds no sugar, never grows back and never holds an agent, so placement, moves, children,
replacement and combat all skip it. Pollution neither lands on it nor diffuses into it. An opaque
wall also stops each of rule M's four lines of sight; a fence blocks only movement. With no walls,
sight, placement and diffusion are exactly as before. The grid draws walls in stone and fences in
wood.

**Walking.** Under `movement.mode: walk`, the decision rule (rule M or the utility mind) picks its
target as before. Then A* finds a 4-way path around walls and other agents, and the agent takes
`speed` steps along it (1–50) and gathers only where it stops. It plans again every tick. If there
is no path within 4 096 expanded sites, it stays and gathers where it is. At vision 1 every target
is one step away, so walking *is* jumping: every golden Sugarscape preset without combat, with
vision forced to 1, gives the same fingerprint under walk as under jump. Walking with combat on is
an error, since rule C jumps. Inspect shows where the agent is heading and how many steps are left
(or that it can't reach its target), and draws its planned path as a dashed line.

Measured (20 seeds; the survey unless a sweep is named). "Holds" and "Fails" are the survey's
verdicts on claims we set before running:

- **The book's carrying capacity needs the jump (Fails, as expected).** On `ii-2-unit` the mean
  population over ticks 300–500 has median 181 under walking against 228 under the jump. No seed
  lands within 214–234, around the book's 224. Walking is lower in all 20 seeds, by a median 47
  agents. The sweeps (not a judged claim) show capacity tracking how far an agent gets in a tick,
  roughly min(speed, vision): walking at speed 1 and vision 1–6 (181.5) is about jumping at
  vision 1 (182.7); walking at speed 3 (212.8) is near jumping at vision 1–3 (205.5); walking at
  speed 6 (225.2) is near jumping at vision 1–6 (228.5).
- **Speed brings it back (Holds).** Capacity rises toward the jump's as speed rises. The
  `walk-speed` sweep's means: 181.5 at one step a tick, 201.4 at 2, 212.8 at 3, 218.4 at 4, 225.2
  at 6 and 227.5 at 10, against 228.5 jumping. In the survey, speed 10 (median 227.4) beats
  speed 1 in every seed.
- **Walking wipes out the gain from vision.** In the `walk-vision` sweep, the jump's capacity rises
  with vision (means 182.7 at vision 1, 205.5 at 1–3, 228.5 at 1–6, 241.5 at 1–10). Walking stays
  flat (182.7, 188.8, 181.5, 182.2). At vision 1 the two are the same rule.
- **Skewed wealth doesn't need the jump (Holds).** Under walking the wealth at tick 500 is
  right-skewed in every seed: skewness median 1.26 against 1.27 jumping, Gini 0.46 against 0.48.
- **Seasonal migration survives walking, with fewer migrants (Holds).** A median 55 % of the agents
  alive over ticks 100–300 change hemisphere at least twice, against 83 % jumping. Likely cause: a
  walker needs many ticks to cross, so fewer finish before the season or their target changes.
- **Walking doesn't bring back the waves (Fails, as expected).** The book's II-6 block sends waves
  toward the far mountain. At tick 100 a median 0.6 % of agents are farther than 25 sites (torus
  distance) from the starting block's center, against 0.8 % jumping and the quarter the claim
  asks for. Walking isn't the missing mechanism.
- **Baum and Kraft's travel claim fails, in the opposite direction.** They found that requiring
  travel to switch patches slightly reduced undermatching. Here a fence separates Minds 1's two
  patches (vision 10–20), with a two-site gap. Moving the gap to the far end (`ifd-fence-far`)
  gives s median 0.85 against 0.90 walking with no fence, lower in 18 of 20 seeds. So undermatching
  grows. The `ifd-detour` sweep compares gap offsets, not a fence against no fence, so it can't
  back that comparison, and no test was run on it. Behind a fence the richer patch's share at
  2.10 : 1 is flat within noise (means 0.6638, 0.6649, 0.6569, 0.6577 at offsets 0, 5, 10, 15;
  sd 0.007–0.014, n 20). Behind a wall it falls slightly, from 0.668 to 0.652 (about 0.016,
  roughly 3 standard errors).
- **A confound in the fenced worlds.** The fences at x = 2 and x = 28 split the torus into 25
  columns on the richer patch's side and 33 on the poorer's, so random placement starts about
  57 % of agents on the poorer side. Its effect isn't measured.
- **The visual barrier: Weak.** Baum and Kraft found a visual barrier had no effect. With an opaque
  wall instead of a fence (same gap), s has median 0.91 against 0.88, and only 10 of 20 seeds are
  within 0.05 of their fence's s. If anything, the wall raises s.
- **At the single 2.10 : 1 ratio the walking arms can't be told apart.** At the presets' own ratio
  (median mean N₁/N₂ over ticks 500–1000): far fence 1.96, near fence 1.99, wall 1.99, walking
  with no fence 1.97, jumping 1.88. For the far gap the ratio (1.96 against 1.97) and s (0.846
  against 0.899) point the same way. None of the ratio differences among the walking arms is
  tested, and their medians are within 0.03 of each other, so at this one ratio they can't be told
  apart. All of them sit above the jump's 1.88. s measures how the split tracks the input across
  five patch sizes; the ratio is one point on that line.

**Cost** (µs per agent-tick: wall-clock over the whole tick with all rules, from the release CLI,
2 000 ticks, seeds 1–5, divided by the population summed over the ticks):

| Preset | Movement | µs per agent-tick |
|---|---|---|
| `ii-2-unit` | jump (book) | 1.05 |
| `walk-capacity` | walk, speed 1 | 4.66 |
| `walk-fast` | walk, speed 3 | 5.10 |
| `ifd-far-sighted` | jump, vision 10–20 | 2.65 |
| `ifd-fence` | walk, vision 10–20, fences | 7.77 |

Minds 1's baseline from planning: `ii-2-unit` 1.09 (book) and 1.13 (utility mind). Walking costs
about 4.4 times the jump on `ii-2-unit`, and about 2.9 times at vision 10–20 behind fences.

Switches (the Rules panel's **Movement (Minds 2)** group): **Mode** (Jump (book) or Walk) and
**Speed** (cells per tick), both live. Walls come only from presets. Presets: `walk-capacity`,
`walk-wealth`, `walk-seasons`, `walk-waves`, `walk-fast`, `ifd-fence`, `ifd-fence-far`, `ifd-wall`.
Built-in sweeps: `walk-speed` (capacity against speed), `walk-vision` (capacity against vision,
walking and jumping), `ifd-detour` (the richer patch's share against the gap's offset, fence and
wall).

Credit: P. E. Hart, N. J. Nilsson and B. Raphael, "A Formal Basis for the Heuristic Determination
of Minimum Cost Paths," *IEEE Trans. Systems Science and Cybernetics* 4(2) (1968); N. R.
Sturtevant, "Benchmarks for Grid-Based Pathfinding," *IEEE Trans. Computational Intelligence and AI
in Games* 4(2) (2012), with the Moving AI benchmark data (movingai.com, ODC-By); W. M. Baum and
J. R. Kraft, "Group Choice: Competition, Travel, and the Ideal Free Distribution," *JEAB* 69
(1998). See `docs/superpowers/specs/2026-09-27-minds-2-walking-design.md`.

### Minds 3: memory, belief and truffles

This is our own experiment, not a reproduction: the third step of the Minds program
(`docs/studies/2026-09-27-minds.md`). Rule M sees only what is in sight, and a walker forgets a
target as soon as it drops out of view. Minds 3 gives agents memory of the sites they've seen and a
belief about what a remembered site holds now. It adds hidden **truffle** spots that only memory can
exploit. Then it measures memory's value as an information asymmetry: rememberers against
non-rememberers in the same world, paired seed by seed.

**Memory.** Under `memory.span` > 0 a share of agents (`memory.share`, drawn at birth) remember.
Each tick, after moving, a rememberer records every site in sight and its own site, with the levels
it saw and the tick. An entry not seen again for `span` ticks is forgotten, and an agent holds at
most 4 096 sites: past that, the ones seen longest ago go first. Children start with empty
memories. Memory needs walking, since a remembered site out of sight can only be walked to.

- **Belief.** `recall` believes a remembered site holds what it held when seen. `project` believes
  that plus growback since, capped at the most ever seen there (the docking with Hornvale's
  "dynamics" beliefs).
- **The choice.** Remembered sites out of sight join the candidates with their believed value, at
  their torus distance. Rule M and the utility mind then choose as before. The agent can't see who
  stands on a remembered site, so occupancy doesn't filter it; if the target turns out occupied, the
  walker stops one site short, or stays. Remembered sites carry no pollution discount, since the
  agent can't see pollution out of sight. Under idle `wander`, an agent wanders only when nothing in
  sight or in memory scores above 0, and then only among sites in sight.
- **The reduction.** With `span` 0 every preset keeps its fingerprint, and memory draws nothing.

**Truffles.** A share of sites (`truffles.share`) hold a hidden spot, placed by a hash of the site
and `truffles.seed`, never by the world's random numbers. So the layout is the same with memory on
or off. Nobody sees a spot. An agent that stops on a ripe one gathers `truffles.value` sugar, and the
spot ripens again `truffles.regrow` ticks later. Anyone can find one by chance; only a rememberer can
come back.

Measured (20 seeds, ticks 200–500 unless named; the survey, which judges claims we set before
running):

- **Memory mostly hurts under rule M.** Rememberers end up poorer in five of the six worlds. On the
  open sugarscape (`mem-open`) they hold a median 324 sugar against the others' 438, an advantage of
  −113; we expected about 0, and only 2 of 20 seeds land within 10 %. Behind the wall
  (`mem-walled`) it's −114, and only 7 % of rememberers are alive at tick 500 against 74 % of the
  others. Through the seasons it's −48, among truffles −82, and on the two patches out of sight
  (`mem-catchment`) −90, poorer in every seed, though rememberers reach a patch sooner (median tick
  27.5 against 33). That −90 is on holdings near 100 000 (an endowment so large nobody starves), so
  under 0.1 %. Memory pays only on the trapline world: +69, in every seed (measured with half the
  agents remembering; the preset has everyone remember).
- **Pricing travel removes the loss, largely by using memory less; no gain was shown.** Rule M
  values a site by its sugar alone, so a far remembered site believed full beats a near one: rule M
  prices no travel, a likely contributor to the loss. Under the utility mind with travel k = 0.5
  (memory as before), the advantage on `mem-open` rises in every seed, by a median 119, to about
  neutral: +7.4 (IQR −18 to +37), rememberers richer in only 10 of 20 seeds (Weak). But the share
  of rememberers' choices aimed at a remembered site out of sight falls from 0.68 to 0.10 (median
  over seeds, on ticks 200–500 with any such choice), so the travel price mostly works by leaving
  memory unused. Across the arms, the loss tracks how often memory is used: `project` uses it about
  twice as often as `recall` (0.68 against 0.34 on `mem-open`, 0.73 against 0.34 on `mem-truffles`)
  and loses more (−113 against −34, and −82 against −25). The same switch takes `mem-walled` from
  −114 to −55 (higher in every seed, still a loss) and `mem-truffles` from −82 to +1.5 (higher in
  18 of 20), and there too memory is used less: the share falls from 0.96 to 0.65 behind the wall
  and from 0.73 to 0.25 among truffles.
- **Projection is worse than recall, the opposite of Hornvale.** On `mem-open` the advantage is −113
  under `project` against −34 under `recall`, worse in every seed, with a larger belief error (2.67
  against 2.46 sugar). On `mem-truffles` it's −82 against −25, with twice the belief error (4.99
  against 2.46). The belief error is measured only on chosen targets, the ones believed best, so it
  carries a selection bias. Hornvale's goblins gained from projection; likely because Hornvale's planner prices
  what competitors take in the meantime, and ours doesn't. Staleness alone doesn't separate the two
  beliefs: on `mem-open`, 93 % of choices of a remembered site out of sight find less there than
  believed under `project`, and 95 % under `recall`.
- **Rememberers find the truffles but lose overall.** On `mem-truffles` they gather 0.0147 truffles
  an agent-tick against 0.0065 (ticks 1–500), about 2.3 times as many, more in every seed, yet end
  poorer in 18 of 20 seeds.
- **Traplining appears, and pays.** On `mem-trapline` (truffle spots the main food, everyone
  remembering) Thomson, Slatkin and Thomson's index of return variability (0 for a perfect
  trapliner, 1 for random revisits) has a per-seed median of 0.15, below 0.8 in every seed. With half
  remembering, the non-rememberers' index is 0.36, also well below 1; likely the sparse map channels
  anyone's wanderings through the same spots. Rememberers gather 0.050 truffles an agent-tick against
  0.011 and hold 144 sugar against 76, in every seed, as Ohashi and Thomson's "more competitive"
  predicts.
- **Gill's competition effect fails.** The median interval between visits to the same spot is 50
  ticks with 5 agents and with 20 (regrowth takes 40), and about 12 % of revisits come sooner than
  40 ticks either way.
- **Forgetting tracking regrowth isn't shown.** Under `recall` (the `mem-span-recall` sweep,
  recomputed per seed in the survey) the advantage is negative at every span and growback rate, so
  the "best" span is only the least-harmful one. At growback 1 it sits at the shortest span tested
  (10 ticks, IQR 10–10), the floor of the grid; at 0.25 and 0.5 it's 25. The survey scores it Weak
  for Bracis et al. (shorter at the fast rate in 12 of 20 seeds), but a best span stuck at the floor
  can't show forgetting tracking regrowth: only that memory hurts least when it's shortest. Under
  `project` the median advantage is highest at span 10 at every rate, so projection doesn't make
  longer memories pay (Fails).
- **Memory doesn't restore walking's lost capacity.** On `walk-capacity` with memory for everyone,
  the population (mean over ticks 300–500) has median 154 against 181 without memory, lower in every
  seed (175 under `recall`; 228 jumping).
- **The marginal value theorem is untestable here.** On `mem-mvt` (nine rich patches, the utility
  mind with travel) foragers who find a patch never leave: 0 departures over 20 seeds, and a median
  5 of 10 alive at tick 1000, each settled on a patch. Two depleting redesigns were tried and
  withdrawn: with 10 agents nobody is alive after tick 200, and with 3 nobody is alive at tick 1000
  and only 3 departures happen across 20 seeds. Likely reason: these minds compare the values of
  sites, not rates of intake, and hold no estimate of the habitat's average, so the theorem's
  leave-when-your-rate-falls-to-the-average can't be expressed. With sight only along rows and
  columns, a forager that has emptied a patch often sees no other. The theorem moves to Minds 4.

The `mem-share` sweep (20 seeds, an observation, not a judged claim) asks whether memory is worth
more when rare. It isn't, on `mem-truffles`: the mean advantage is −88, −73, −76, −86 and −98 at
shares 0.1, 0.25, 0.5, 0.75 and 0.9 (sd 46–74), negative at every share. No test was run.

**Cost** (µs per agent-tick, measured as in Minds 2; the machine was loaded, so Minds 2's presets
were re-timed in the same session):

| Preset | µs per agent-tick |
|---|---|
| `mem-open` | 19.7 |
| `mem-truffles` | 28.9 |
| `mem-mvt` | 14.5 |
| `mem-walled` | 8.7 |
| `walk-capacity` (re-timed; 4.66 in Minds 2) | 4.91 |
| `ifd-fence` (re-timed; 7.77 in Minds 2) | 14.74 |
| `ii-2-unit` (re-timed; 1.05 in Minds 2) | 1.62 |

Against `walk-capacity` timed now, memory costs 4.0 times on the open sugarscape and 5.9 times with
truffles. `mem-walled` is cheaper than `ifd-fence` because its rememberers die; `ifd-fence`'s
figure also looks inflated by the load. `mem-mvt` ran only 39 519 agent-ticks, since most agents die
early.

Switches: the Rules panel's **Memory (Minds 3)** group (**Span**, **Share born remembering**, both
on reset, and **Belief**, live) and **Truffles** group (**Share of sites with a spot** and **Layout
seed** on reset, **Value** and **Regrow time** live; a live change to regrow time applies to future
harvests only). Inspect shows "Remembers: n sites (m truffle spots)" or "Doesn't remember", and the
grid draws the inspected agent's remembered sites as a faint overlay fading with age, with its known
truffle spots as circles (filled when believed ripe). Charts: **Memory** (`remembered_moves` and
`stale_choices`, both shares), **Belief error (sugar)**, **Rememberers vs others** and
**Truffles**. Truffles need rule M's move to be gathered, so they can't be combined with combat
(rule C). Presets: `mem-open`,
`mem-catchment`, `mem-walled`, `mem-seasons`, `mem-truffles`, `mem-trapline`, `mem-mvt`. Built-in
sweeps: `mem-span-recall` and `mem-span-project` (the advantage against span at three growback
rates) and `mem-share` (the advantage against the share remembering, on `mem-truffles`).

Credit: E. L. Charnov, "Optimal Foraging, the Marginal Value Theorem," *Theoretical Population
Biology* 9(2) (1976); D. W. Stephens and J. R. Krebs, *Foraging Theory* (1986); C. Bracis, E.
Gurarie, B. Van Moorter and R. A. Goodwin, "Memory Effects on Movement Behavior in Animal Foraging,"
*PLoS ONE* 10(8) (2015); D. Boyer and P. D. Walsh, "Modelling the Mobility of Living Organisms in
Heterogeneous Landscapes," *Phil. Trans. R. Soc. A* 368 (2010); J. D. Thomson, M. Slatkin and B. A.
Thomson, "Trapline Foraging by Bumble Bees: II," *Behavioral Ecology* 8(2) (1997); K. Ohashi and
J. D. Thomson, "Efficient Harvesting of Renewing Resources," *Behavioral Ecology* 16(3) (2005); F. B.
Gill, "Trapline Foraging by Hermit Hummingbirds," *Ecology* 69(6) (1988). See
`docs/superpowers/specs/2026-09-28-minds-3-memory-design.md`.

### Minds 4: GOAP and the marginal value theorem

This is our own experiment, not a reproduction: the fourth step of the Minds program
(`docs/studies/2026-09-27-minds.md`). Rule M and the utility mind choose one site at a time and hold
no estimate of what the habitat yields on average, so Minds 3 couldn't test the marginal value
theorem, and rule M prices no travel. Minds 4 adds the program's first planner, goal-oriented action
planning (GOAP), and a learned marginal-value rule. It then runs the theorem's tests and asks Minds
3's open question again: does memory pay a mind that prices travel?

**GOAP, verified.** A generic planner searches any domain that supplies states, actions
(preconditions, effects, cost) and an admissible heuristic, with Minds 2's A\* over the state graph
and an expansion limit. It draws no random numbers. The search runs forward, not backward as in
Orkin's F.E.A.R.: our states hold amounts of sugar, and regression over numeric effects is awkward.
Two checks, on a STRIPS encoding (Fikes and Nilsson):

- **Known optimal plan lengths** (Helmert and Mattmüller): gripper, 3n − 1 for even n and 3n for
  odd, for n = 1–8; two logistics families, 4n (one truck per city for n = 1–6, one truck in one
  city for n = 1–5; n = 6 runs about a minute and is left to an ignored test, which gives 24); and a
  blocks-world family, 4n − 2, for n = 1–6. Every plan is replayed from the start.
- **Against Dijkstra:** on 500 random STRIPS instances (6–12 facts, 4–12 actions, costs 1–3) the
  plan's cost equals Dijkstra's, or both find none: 130 non-empty plans, 107 starts already at the
  goal and 263 unsolvable instances.

**The foraging domain.** Under `decision.rule: goap` an agent plans a run of harvests that gathers
G = metabolism × `goap.horizon` (H ticks of food, default 10) in the fewest ticks.

- **Candidates:** its own site and K others it sees or remembers (`goap.k`, default 8), at Minds 3's
  believed values.
- **Actions:** harvest site i, costing the torus distance + 1 ticks (harvesting where it stands
  costs 1). Regrowth during a plan is ignored.
- **State:** where it is and which candidates it has harvested; gathered sugar is derived, so a plan
  has at most (K + 1)·2^K states.
- **Heuristic:** ⌈(G − gathered) / the best unharvested value⌉, admissible. On 400 random instances
  the plan's cost equals an uninformed search's.
- **Executing.** The agent walks to the plan's first site and keeps the plan until it's invalidated
  (the next site is taken, holds less than half what was planned, or can't be reached) or finished.
  Then it replans, following Orkin.
- **Fallback.** If the known sugar can't reach G, or the search passes 4 096 expansions, it takes
  the single site with the best value ÷ (distance + 1). That's its only random draw, rule M's tie
  draw.
- **One good only:** G is in sugar, and foresight welfare over several goods isn't.
- **The reduction.** With travel priced at 0 (a test-only hook), K covering sight and H = 1, GOAP's
  target has rule M's best value and nearest distance on 200 random neighborhoods (where one harvest
  meets G). Among equal-cost plans GOAP takes A\*'s fixed order, not rule M's draw, so the two can
  pick different sites among equals.

**Which sites a plan considers decides the result.** The spec shortlisted the K best known sites by
value. With the map known, those are the far, full peak centers, so the planner can only plan long
walks. It shuttles between centers and starves: on the MVT world at spacing 20, 4 of 60 agents are
alive at tick 1000 (20 seeds, 3 each), against 28 of 60 when the shortlist ranks by value ÷
(distance + 1). So the default is that rate shortlist, and the spec's value ranking is kept as the
switch `goap.shortlist: value`.

**The marginal-value rule** (`decision.rule: mvt`) is Constantino and Daw's rule at site level. Each
agent keeps ρ, a running mean of its gain per tick (travel ticks count 0; ρ ← ρ + α(gain − ρ),
α = `mvt.alpha`, default 0.05; ρ starts at the metabolism). It stays while the best site within one
step is believed to yield at least ρ; otherwise it commits to the best site it knows and walks there.

**Knowing the map.** `memory.prior: map` gives every founding rememberer every non-wall site at
tick 0, with its starting level (the theorem's ideal forager); children start empty. It needs a
memory span. Past Minds 3's cap of 4 096 sites, the richest are kept.

**The theorem's world** (`goap-mvt`, `mvt-rule`): nine patches (peaks of radius 3, height 4, 25
sites and 56 sugar each) 20 apart on a 60 × 60 torus, with 3 agents of metabolism 1, vision 1–6,
endowment 50, walking, memory for all (span 1 000, `project`, the map known). The design asked that
a patch run out under one forager while the whole world feeds everyone. At the spec's growback 0.05
a patch regrows 1.25 a tick, more than one forager eats, so it never runs out. So growback became
0.02 (0.5 a tick per patch) and the population 5 became 3 (regrowth 1.5 times the need). A lone
planner then empties its patch by tick 33 and leaves at tick 36. Even so only 1–2 of the 3 are
alive at tick 1000 in each seed.

Measured (20 seeds; the survey, which judges claims we set before running, unless a sweep is named).
The travel tests use the same world at spacings s = 12, 16, 20 and 24 (a 3s × 3s torus):

- **Longer travel, longer stays, under GOAP (Holds).** The per-seed slope of mean patch residence on
  spacing is positive in 20 of 20 seeds (median 0.41). Median residence is 18.6, 22.6, 25.7 and
  22.8 ticks at s = 12–24. Under the marginal-value rule it holds too, but the effect is small:
  positive in 19 of 20 (median 0.096), with 11.8, 12.4, 12.7 and 12.9 ticks. Rule M with the same
  knowledge reverses the theorem, with shorter stays at longer travel in 20 of 20 seeds (6.3 down to
  5.0), and so does GOAP with the value shortlist (20 of 20; 7.6, 8.6, 6.8, 5.9). Caveat: survival
  falls with spacing under every rule, so at s ≥ 20 the residences rest on few agents (a median of
  1 or fewer of the 3 alive at tick 1000), and death cuts visits short.
- **Overstaying: GOAP Holds as judged, likely only through the walk out; the marginal-value rule
  Fails.**
  The measure, set before running: at each departure, the gain on the agent's last tick in the
  patch against its mean gain per tick so far. At s = 20, GOAP overstays at a median 57 % of
  departures, over half in 18 of 20 seeds. The marginal-value rule overstays at 39 %, over half in
  0 of 20. That last tick is often the first step out. Without the ticks in transit (reported, not
  judged; fallback ticks that stay put count as transit, 0.1–0.7 % of ticks), GOAP overstays at
  20–33 % and the marginal-value rule at 2–7 %, and no seed is above half at any spacing under
  either. So GOAP's literal overstaying is likely the walk out.
  Overstaying also falls as travel grows under both (GOAP 61 % to 46 %, the marginal-value rule
  56 % to 35 %, from s = 12 to 24). That's the opposite of Constantino and Daw, whose people
  overstayed significantly only when travel was long.
- **Memory pays a planner, among the living (Holds).** Under GOAP the rememberers' wealth advantage
  (ticks 200–500) is +69 on the open sugarscape (19 of 20 seeds), +99 among truffles (20 of 20) and
  +33 behind the wall (20 of 20), against rule M's −113, −82 and −114 in the same worlds. Minds 3's
  travel-priced utility mind (travel 0.5), the fairer comparator, gave about +7 open, +1.5 among
  truffles and −55 behind the wall. The memory is used: 98–99 % of rememberers' plans include a
  remembered site out of sight.
- **Part of it is survivorship.** Counting the dead as 0 (sugar per founding member, ticks
  200–500), the advantage is +35 open (18 of 20 seeds), +26 among truffles (14 of 20) and +10 behind
  the wall (15 of 20), short of 80 % of seeds in the last two. Fewer rememberers are alive at tick
  500 than others in every world (48 % against 50 %, 41 % against 53 %, 60 % against 74 %). The
  judged advantage is conditional on survival.
- **The gain is relative to planners who don't remember, not to rule M.** GOAP's non-rememberers
  are much poorer than rule M's (240 against 438 on the open sugarscape, 254 against 430 among
  truffles), while a GOAP rememberer on the open sugarscape (319) is no richer than rule M's (324).
  GOAP also holds more agents (population 200.5 against 176.5 open, 67.3 against 45.8 behind the
  wall); more agents sharing the same sugar is a likely cause, not isolated. A possible bias isn't
  isolated either: a plan's next site must still be a candidate, and a non-rememberer loses an
  off-axis site from its candidates as soon as it's out of sight, so its plans are dropped sooner.

The sweeps (20 seeds, observations, not judged claims):

- **Horizon** (`goap-horizon`, `ii-2-unit` walking under GOAP, mean population over ticks 300–500):
  143.3 at H = 2, 159.0 at 5, 206.4 at 10, 206.3 at 20 and 195.7 at 40, against 181 for walking
  rule M and 224 for the book's jump. So planning ten ticks of food brings back over half of what
  walking lost. But on `ii-2-unit` most GOAP decisions are the rate fallback (52 % of agent-ticks
  against 21 % planned, one seed over 200 ticks), so the gain is likely as much the rate choice as
  the planning. Not isolated.
- **K** (`goap-k`): 196.5, 207.9, 206.4 and 204.9 at K = 2, 4, 8 and 12 (sd 9.5–11.5): flat within
  noise past four sites (untested).
- **Memory's share** (`goap-memory`, on `goap-truffles`): the mean advantage is +90, +100, +104,
  +100 and +113 at shares 0.1, 0.25, 0.5, 0.75 and 0.9 (sd 41–52), against Minds 3's `mem-share`
  under rule M (−73 to −98). Positive at every share, and not worth more when rare. No test was
  run.

**Cost** (µs per agent-tick, measured as in Minds 2 and 3; Minds 3's presets re-timed in the same
run):

| Preset | µs per agent-tick |
|---|---|
| `goap-mvt` | 80.7 |
| `mvt-rule` | 54.2 |
| `goap-open` | 82.2 |
| `goap-truffles` | 74.7 |
| `goap-walled` | 149.0 |
| `mem-open` (re-timed) | 14.2 |
| `mem-truffles` (re-timed) | 24.3 |
| `mem-mvt` (re-timed) | 14.6 |
| `mem-walled` (re-timed) | 9.1 |
| `walk-capacity` (re-timed) | 5.58 |
| `ii-2-unit` (re-timed) | 1.20 |

Against rule M in the same world, planning costs 5.8 times on the open sugarscape, 3.1 times with
truffles and 16 times behind the wall, where rule M's rememberers die. The search never passed its
limit in any survey world. `goap-mvt` and `mvt-rule` run only 3 agents on 3 600 sites, which
inflates both figures.

Switches: the Rules panel's **Decision (Minds 1, 4)** group adds GOAP and the marginal-value rule to
**Rule** (on reset; both need Movement: Walk), and live **K**, **Horizon**, **Which known sites**
(rate or value) and **α**. The **Memory** group adds the prior (on reset). Inspect shows a planner's
"Plan: n steps, gathers ~x of G" and its next target, with its route drawn on the grid, or a
marginal-value agent's ρ. Charts: **Planning** (mean plan length) and **Plan use** (plans and
fallbacks per agent; plans using memory as a share of plans) under GOAP; **Average rate** and
**Leaving** under the marginal-value rule. Presets: `goap-mvt`, `mvt-rule`, `goap-open`,
`goap-truffles`, `goap-walled`.
Built-in sweeps: `goap-horizon`, `goap-k`, `goap-memory`.

Credit: R. E. Fikes and N. J. Nilsson, "STRIPS: A New Approach to the Application of Theorem Proving
to Problem Solving," *Artificial Intelligence* 2 (1971); J. Orkin, "Applying Goal-Oriented Action
Planning to Games," *AI Game Programming Wisdom 2* (2004), "Symbolic Representation of Game World
State: Toward Real-Time Planning in Games," AAAI workshop WS-04-04 (2004), and "Three States and a
Plan: The A.I. of F.E.A.R.," GDC (2006); M. Helmert and R. Mattmüller, "Accuracy of Admissible
Heuristic Functions in Selected Planning Domains" (2007); J. Slaney and S. Thiébaux, "Blocks World
Revisited," *Artificial Intelligence* 125 (2001); E. L. Charnov, "Optimal Foraging, the Marginal
Value Theorem," *Theoretical Population Biology* 9(2) (1976); S. M. Constantino and N. D. Daw,
"Learning the Opportunity Cost of Time in a Patch-Foraging Task," *Cognitive, Affective, &
Behavioral Neuroscience* 15(4) (2015). See `docs/superpowers/specs/2026-09-28-minds-4-goap-design.md`.

### Minds 5: caching for the future

This is our own experiment, not a reproduction: the fifth step of the Minds program
(`docs/studies/2026-09-27-minds.md`). Two laboratory studies of jays read caching differently. Raby
et al. (2007) found scrub-jays caching more where they had gone without breakfast, and read it as
planning for a future need. Amodio et al. (2021) tested that planning hypothesis against a
compensatory one, and their Eurasian jays' caches were best fit by neither: the model in which
caching doesn't depend on the compartment won. Minds 5 gives agents a carrying limit, caches they
bury and dig, and a winter everywhere at once. It builds three caching rules, each the mechanism one
hypothesis names, runs each through both labs' protocols, and asks whether each rule leaves its
predicted signature and which signature the birds' data resemble. Then it puts the same rules in a
winter field. The same mechanics make central-place foraging possible, so Minds 5 tests that too.
Pilfering, being watched and re-caching (anything that needs one agent to perceive another's
caches) are left for a future campaign.

**What was built.**

- **A carrying limit** (`caching.capacity`, 0 = none, the default). An agent never holds more sugar
  than the limit; harvest beyond it stays on the site. Caching needs exactly one good and walking
  (a cache out of sight is memory, and Minds memory already walks), and it can't run with combat.
- **Caches.** Each agent owns its caches (site → amount). Nobody else sees or takes them, they don't
  decay, and they die with it (counted as lost). Burying where it stands costs no tick.
- **Digging is foraging, with hysteresis.** When an agent holds less than half its reserve R
  (R = metabolism × `goap.horizon`), its caches join its candidate sites, valued at their amounts,
  under any decision rule, and arriving at one digs it. Burying stops at R, so the band between R/2
  and R keeps an agent from digging back what it just buried. With the threshold at R, `plan` buried
  44 842 and dug back 41 986 in one seed's first summer; with R/2, it buried 14 579 and dug 58. A
  central-place world keeps the threshold at R, one tick's need there, because an agent between R/2
  and R that didn't dig would starve. Dug sugar isn't a new harvest: it makes no pollution, earns
  no credit income and doesn't feed a marginal-value agent's ρ. Sites, holdings, caches and what was
  eaten are conserved across bury and dig (tested over 300 ticks).
- **Three caching rules** (`caching.rule`; `none` is the default):
  - `even` (Amodio et al.'s compartment-independent model) buries a share (`caching.share`, 0.5) of
    its surplus above R where it stands, every tick it has any.
  - `compensate` (their compensatory mechanism) keeps a weight for each site it has harvested,
    starting at 1 and multiplied by 1 − λ (`caching.lambda`, 0.5) each time it finds food there. It
    buries share × surplus × w ÷ w̄, where w̄ is the mean weight of the sites it knows, capped at the
    surplus.
  - `plan` (the Future Planning Hypothesis) keeps a what-where-when memory of its days (where it
    was, and whether it found food). A **cycle finder** takes the smallest period each sequence
    repeats with and extrapolates it; with no period, it predicts nothing, and the agent buries
    nothing. In the lab it provisions for the next `caching.lookahead` days (1 is FPH 1, 3 is
    FPH 2), splitting its food over the places it predicts will lack food. In the field (global
    seasons only) it knows the calendar and buries through the summer toward its forecast
    shortfall: a winter's need, less the intake from sites it recorded last winter, less what it
    has cached. It buries at sites it foraged last winter, and before its first winter it forecasts
    no winter intake.
  - `caching.mixed` deals the four rules round-robin by id. A child takes the rule of the parent
    whose turn it is.
- **Global winter** (`seasons.mode: global`): every site grows back at 1/β of its rate at once. The
  book's hemispheres stay the default, and global winter with β = 1 equals seasons off (tested).
- **The lab as a world mode** (`lab`, reset-only; the spec had planned a harness outside the world).
  The rig is 13 × 8: three compartments, K1–K3, whose doorways open onto a hall. The world applies
  the schedule itself (walls, each morning's food, which compartment the agents are shut in), so
  tests, survey and presets share one code path. A day is 4 ticks: a morning shut in one
  compartment, with or without food, then an evening in the hall. On the test evening the doorways
  open (only K1's and K3's for Raby, whose trays were in two compartments). Each agent in turn is
  given 30 sugar and walks out alone to cache all of it, in whole units, with remainders in K
  order. The schedule draws nothing. A population's λ is drawn per agent in [0.3, 0.7), through the
  world's existing draws.
- **Central-place provisioning** (`central.enabled`, under the marginal-value rule). Each agent has
  a home and a larder there. Its ρ is its delivery rate: ρ ← ρ + α(delivered this tick − ρ), with
  α = `mvt.alpha` (0.05), so walking and harvesting ticks count 0. At home it buries the trip's load
  in the larder and leaves for the best site it knows (ties go to the nearer). It keeps enough in
  hand for the walk out and back (metabolism × (2D + 2), dug from the larder if short). In a patch it
  harvests while the best site within a step yields at least ρ, its load has room under the limit,
  and it has food for the walk home. Then it walks home. The limit caps a trip's load, not load plus
  provisions.

**The labs** (the survey, 20 seeds; claims and thresholds set before running).

- **Oracles first.** Every rule's test-evening caches match a hand derivation in both protocols and
  both of Amodio's groups. These are tests, not claims.
  - **Raby:** `even` 15/15. `compensate` 27/3, or 26/4 in the other counterbalancing (the K-order
    remainder). `plan` looking one day ahead puts all 30 in the no-breakfast compartment, or caches
    nothing when breakfast came first.
  - **Amodio,** Food-First and Empty-First (K1/K2/K3): `even` 10/10/10 in both. `compensate`
    8/15/7 and 12/6/12. `plan` looking one day ahead 30/0/0, and nothing. `plan` looking three
    days ahead 15/0/15 and 0/30/0.
- **FPH 2 and the Figure 6 caption.** For Food-First planners looking three days ahead, the next
  days are K1 without food (day 10), K2 with food (11) and K3 without (12), so our planners cache in
  K1 and K3. The paper's Figure 6 caption says "K1 and K2". But its Methods constrain FPH 2 for the
  Food-First group as r_K2 ≤ r_K1 and r_K2 ≤ r_K3, which is K1 and K3. So the caption slips, the
  comparison the paper ran tested the right pattern, and ours matches it.
- **Provisioning for tomorrow caches nothing when tomorrow has food.** A planner looking one day
  ahead caches nothing in Amodio's Empty-First group or in Raby's breakfast-first order. The paper's
  FPH 1 means "the next day without food", while ours is literally tomorrow. We state this rather
  than patch it.
- **Raby: `compensate` Holds, `plan` is Weak, and `even` Fails, as it should.**
  - Compensating agents cache 26.4 ± 0.1 in the compartment without breakfast against 3.6 ± 0.1
    in the other (mean ± s.e.m., both orders, 320 agents), every one of them more where breakfast
    was missing. Raby et al.'s birds cached 16.3 ± 1.8 against 5.4 ± 1.8.
  - Planners cache all 30 there in one order and nothing in the other. So only 160 of 320 cache
    more where breakfast was missing, and the claim is Weak: breakfast-first planners cache
    nothing.
  - Even splitters cache 15 in each. That's the rule's predicted tie, the expected control, not a
    failure of the rule.
  - Individual differences come only from `compensate`'s λ. Even and plan populations have none.
- **Amodio: all four Hold.** Each rule leaves its predicted pattern in 20 of 20 seeds, in both
  groups. Compensating agents cache a mean 7.9/15.2/6.9 (Food-First) and 12.5/6.1/11.5
  (Empty-First).
- **The paper's Bayesian comparison, reproduced.** We wrote it from the Experiment 2 Methods: nine
  models, multinomial counts, flat Dirichlet priors truncated to each hypothesis's constraint
  region, rates integrated out, and a uniform prior over models. The text leaves one reading open,
  and we share a bird-independent model's rates within each group. With that reading, the paper's
  own Table 2 gives the compartment-independent model 0.719, CCH 0.160, FPH 1 0.0023 and FPH 2
  0.0018. Those are the published 0.72, 0.16 and 0.002, and a unit test checks them. On our agents
  it picks each rule's own hypothesis:
  - the compartment-independent model at 0.999 for even splitters;
  - CCH at 0.90 for compensating agents (at the paper's size, three agents a group, CCH wins in 15
    of 20 seeds and the compartment-independent model in the other 5);
  - FPH 1 at 0.55 and FPH 2 at 0.62 for planners.
- **The planners' 0.55 and 0.62 are ceilings, not weak evidence.** Every planner caches alike, so
  the comparison treats them as one bird, and more agents don't raise the figures. The hypotheses
  also share the evidence. FPH 2's constraint also fits caching only in K1, FPH 1's fits 15/0/15 at
  half weight, and the two agree on the Empty-First group.
- **The jays' pattern looks like the even splitters'.** Amodio et al.'s 0.72 for the
  compartment-independent model is the signature our even-share rule leaves, not the planners' or
  the compensators'. That's a resemblance, not a test of what the jays do.

**The winter world, balanced as measured.** The spec asked for a world where (a) an agent's summer
surplus is at least 1.5 times its winter need, and (b) winter regrowth is under half the
population's winter need, with the settings adjusted mechanically. walk-capacity's own agents
(metabolism 1–4) can't meet (a) at any population, β or γ tried (0.38–0.98), because a walker
harvests at most about 3.5 a tick. The ruled world is walk-capacity's landscape with 175 agents of
metabolism 1 (half remembering for 100 ticks), rule M walking, γ = 100 and β = 32. There (a) is
151.0 against 100 (1.51) and (b) is 6 466 against 16 580 (0.39), both measured with no caching and
no carrying limit. Three more rulings came from measuring:

- **A carrying limit of 50,** half a winter's need. Without a limit, holdings are an unlimited cache
  and caching is moot.
- **An empirical balance test.** (b) counts only regrowth, but every site starts winter full, and
  that standing sugar feeds many: without a limit, 88 % of agents who don't cache survived the first
  winter. So the test became: over 5 seeds, `none`'s first-winter survival must be at least 20
  points below `even`'s.
- **A horizon of 20** (R = 20, digging below 10) for every rule. At the default 10, `even` did worse
  than `none` (45.2 % against 48.5 %): a hungry agent under rule M heads for its biggest cache,
  goes hungry only below 5, and starves on the way. At 20 the 5-seed probe gave `none` 48.5 %,
  `even` 74.9 %, `compensate` 73.8 % and `plan` 88.2 %. `none` doesn't use the horizon, and neither
  (a) nor (b) depends on it.

**The winter** (the survey, 20 seeds; first-winter survival is alive at tick 200 ÷ alive at 100):

- **Caching gets agents through winter (all Hold, 20 of 20 seeds each).** First-winter survival is
  49 % with no caching, 74 % with an even share, 74 % compensating and 88 % planning. After five
  winters (alive at 1000 ÷ alive at 100) it's 49 %, 57 %, 55 % and 70 %. Planning beats both other
  rules after one winter and after five, in 20 of 20 seeds. Per founding agent (the dead counted),
  alive at ticks 200 and 1000:
  - none: 47 % and 46 %;
  - even: 70 % and 54 %;
  - compensate: 70 % and 53 %;
  - plan: 84 % and 67 %.
- **Much of what's buried is not dug by tick 1000** (the end of the fifth winter, a snapshot at the
  run's end):

  | World | Not dug by tick 1000 | Lost with the dead | Still buried |
  |---|---|---|---|
  | even | 74.7 % | 15.8 % | 58.9 % |
  | compensate | 75.0 % | 18.3 % | 56.7 % |
  | plan | 55.6 % | 11.5 % | 44.1 % |
  | mixed | 75.6 % | 13.9 % | 61.6 % |

  Planners bury a sixth as much as even splitters. The mean age of a cache when it's dug is 212–312
  ticks.
- **Rule M heads for the biggest cache.** Take the choices where no biggest cache is also a nearest
  one. There, a hungry agent choosing a cache takes its biggest 70 % of the time (even), 70 %
  (compensate) and 76 % (plan). That cache is a mean 6.5, 6.6 and 5.0 steps away, when the nearest
  is 1.4, 1.4 and 2.1. Of the agents who die holding caches, 89 %, 91 % and 94 % die heading for
  one. At their last choice of a cache, 41.5 %, 44.3 % and 40.7 % chose one beyond the reach of
  their food while a nearer one was within it. This is likely one cause of the deaths, not isolated.
- **The expected planner mechanism isn't seen.** We expected planners who lived through a winter on
  their caches to forecast little winter intake, and to re-bury a full winter's need each summer.
  They don't. Survivors forecast a mean 54 of winter intake from sites (0 of 2 932 forecast under
  10). They bury a median 5.4 each in the second summer against 82 in the first. Likely cause: the
  sugar standing when winter starts counts as intake from sites. The forecast is measured on
  survivors only.
- **The mixed world** (a quarter on each rule). Planners are 25 % of the founders but 32 % of the
  survivors at tick 200 (31 % at 1000); agents who don't cache are 18 % (19 %). Of each rule's
  founders, alive at ticks 200 and 1000:
  - plan: 84 % and 70 %;
  - compensate: 68 % and 52 %;
  - even: 65 % and 56 %;
  - none: 47 % and 41 %.

  Each rule fares about as it does in a world of its own, likely because nobody sees or takes
  another's caches.

**Central place** (the survey, 20 seeds). The world is a 60 × 30 torus with a column of five patches
(peaks of radius 3 and height 4) growing back 0.25 a tick. Five agents of metabolism 1, endowment
60 and vision 1–6 know the map and carry loads of at most 320. Their homes are 8 columns west
(`central-near`) or 20 (`central-far`), and `central-linear` is near with instant growback. The
limit was raised from 80 to 320 until ρ or an empty site, not a full load, ended at least half the
trips at both distances. Nothing else was tuned.

- **Loads rise with distance (Holds, 20 of 20 seeds).** The mean load per trip is 68, 84, 100, 107,
  108 and 110 with the patches 4, 8, 12, 16, 20 and 24 columns from home. Near, a trip gathers a
  mean 128 and brings 84 home; far, it gathers 176 and brings 108. Every agent lives.
- **The analytic result, and a cancellation.** The test patch is a staircase of ten sites around
  home, yielding 15, 12, 10, 8, 6, 5, 4, 3, 2 and 1 in harvest order. There the tangent
  construction gives optimal loads of 51, 60 and 65 at distances 2, 5 and 10, and the learned rule
  takes 49, 60 and 65. But that match is a cancellation, not the theorem at work.
  - A staircase site's cost depends on its parity: a step out costs 2 ticks, a step back 0 net. The
    rule's per-tick comparison with ρ can't see that.
  - With ρ fixed at the true optimal rate, the rule takes 45, 60 and 63.
  - The learned ρ is a pulse average. By the decision to leave it has decayed below the optimal
    rate (5.73, 3.21 and 1.78 against 6.375, 3.75 and 2.32), and at distances 5 and 10 that
    underestimate cancels the parity error.
  - The distance effect exists only at small α. At α 0.2 the load is 65–66 at every distance.
- **Lima's prediction fails as pre-registered.** Lima's point (Stephens and Krebs): the leaving rule
  depends on the habitat's long-run rate, so a forager alternating between a near and a far patch
  with the same loading curve should take the same load from both. The judge, set before running:
  |near − far| within 10 % of the seed's mean load in at least 80 % of seeds, with each trip
  assigned to the patch that gave over half its load. Far trips bring home 120 and near ones 54,
  and 0 of 20 seeds are within. The verdict stands.
- **But the habitat confounded it.** The near patch lay on the way to the far one, and agents
  harvest where they step: 202 trips headed for the far patch took over half their load from the
  near one. Follow-up rows (reported, not judged; delivered per trip, pooled over seeds, trips sorted
  by where the agent was headed):

  | Row | Habitat | Near | Far | Seeds within 10 % | Alive at 1000 |
  |---|---|---|---|---|---|
  | 1 (pre-registered) | far patch beyond the near one | 71 | 75 | 5 of 20 | 97 of 100 |
  | 2 | far patch on the other side of home; no trip crosses the other | 73 | 83 | 8 of 20 | 99 of 100 |
  | 3 | row 1, ρ held at each seed's long-run rate (median 0.61) | 85 | 82 | 4 of 20 | 88 of 100 |
  | 4 | row 2, ρ held the same way | 74 | 100 | 1 of 20 | 80 of 100 |

  - Without the crossing, far loads are 14 % larger, and with ρ fixed as well, 35 % larger. So ρ's
    decay over the walk isn't needed for the gap.
  - In rows 2 and 4 the far patch holds more sugar when an agent arrives (37.7 and 37.5 against
    34.5 and 33.6). An agent heads far only when a far site is strictly richer than every near one
    (ties go to the nearer). That selection is the likely cause, not isolated. In row 4 a 12 %
    difference in standing sugar explains a 35 % gap in loads only in part.
  - Holding ρ fixed also costs lives (88 and 80 of 100).
  - In no arrangement are near and far within 10 % of each other in most seeds.
- **So the distance effect likely Holds for reasons the theory doesn't use.** ρ on arrival is lower
  after a longer walk (a median 2.4 near and 0.73 far). Far out, the carrying limit sets the loads:
  53 % of far trips end full, and the limit ends 75 % of the pre-registered habitat's majority-far
  trips. Both are likely contributors, neither isolated.
- **Linear loading and the pulse.** With instant growback a load grows with the time spent
  gathering it. A delivery lifts ρ by about α × the load (5.9 here), above the most any site yields
  (4), so the next trip turns back almost at once. 54 % of trips end almost at once: 36 % on the
  first tick deciding in the patch, and 17 % before deciding there, likely because the target was
  taken on the way. The other 46 % fill to the limit. The theorem predicts no effect of distance
  under linear loading, yet loads grow with it (121 near, 151 at 20 columns), as with Kacelnik and
  Cuthill's starlings. The likely cause is ρ decaying over the longer walk.
- **GOAP's "deliver G"** (G = 10; reported, not judged). It delivers about G at any distance
  (17.8 near, 10.3 far, 19.1 linear), and 28, 0 and 52 of 100 agents are alive at tick 1000. It has
  no reason to carry more, and it starves far from home.

**The sweeps** (built in; observations, not judged). `central-distance` is the load-on-distance
series above. `cache-capacity` (population at the end of the first winter against the carrying
limit, per rule) and `cache-winter` (population against the season length under `plan`) had only a 3-seed
check. In it, every rule came out about the same at a limit of 20, the reserve, where the surplus
above R is near zero. Population fell as γ grew, in a fixed window that reads several winters at
small γ. Neither has been run at 20 seeds.

**Cost** (µs per agent-tick, CPU time, measured as in Minds 2–4). The full table, with Minds 4's
presets re-timed and the machine's load, is in the program document. No caching costs 2.94. An
even share costs 10.5 and compensating 10.4, likely because those agents hold many caches and each
hungry tick adds all of them to the candidates. Planning costs 5.90 and the mixed world 6.88. The
central-place worlds cost 15.8–19.2 (5 agents on 1 800 sites, which inflates the figure). The labs
cost under 1, mostly process start and idle ticks.

Switches: the Rules panel's **Caching (Minds 5)** group has **Caching rule**, **Mix the rules**,
**Carrying limit**, **Seasons** (north and south take turns, or winter everywhere at once) and
**Central-place foraging**, all on reset, and live **Share**, **λ** and **Days looked ahead**. The
Seasons group's β now reaches 32. Inspect shows an agent's caching rule, what it carries against the
limit, its caches (count and total) and a planner's forecast shortfall. In a central-place world it
also shows the larder at home on its own row, the home, and the last load. The map draws the
inspected agent's caches as diamonds and its home as an outlined square. Charts: **Caching**
(cached, buried, dug) and **Recovery** when caching is on; **Loads** and **Trips** in central-place
worlds. Presets: `cache-winter-none`, `cache-winter-even`, `cache-winter-compensate`,
`cache-winter-plan`, `cache-winter-mixed`, `central-near`, `central-far`, `central-linear`,
`cache-raby` and `cache-amodio` (the labs, whose roster is fixed, with the four rules
dealt round-robin). Built-in sweeps: `cache-capacity`, `cache-winter`, `central-distance`.

Credit: N. J. Raby, D. M. Alexis, A. Dickinson and N. S. Clayton, "Planning for the future by
western scrub-jays," *Nature* 445 (2007), 919–921; P. Amodio, J. Brea, B. G. Farrar, L. Ostojić and
N. S. Clayton, "Testing two competing hypotheses for Eurasian jays' caching for the future,"
*Scientific Reports* 11 (2021), 835; D. W. Stephens and J. R. Krebs, *Foraging Theory* (1986),
§3.5 (central-place foraging, after Orians and Pearson, 1979; Lima's point on individual trips)
and §9.4–9.5 (Kacelnik and Houston, 1984; Kacelnik and Cuthill, 1986). See
`docs/superpowers/specs/2026-09-29-minds-5-caching-design.md`.

### Minds 6: theft

This is our own experiment, not a reproduction: the sixth step of the Minds program
(`docs/studies/2026-09-27-minds.md`) and the first of its campaign on pilfering. Andersson and Krebs
(1978) argued that hoarding pays in a group only if a hoarder is more likely to recover its own
food than anyone else is to find it first. They added that where recovery is equal, cheaters who
never hoard win. Vander Wall and Jenkins (2003) surveyed field studies reporting pilferage of 2–30 %
a day, so high that a recovery advantage seems not to be enough. They argued that hoarding persists because pilfering is
reciprocal: the thief caches what it takes, and gets robbed in turn. Minds 6 lets Minds 5's agents
steal each other's caches. It tests Andersson and Krebs's condition, their claims about frequency
and equal recovery, Vander Wall and Jenkins's reciprocity, and their untested suggestion that a
cheater could flourish in mild winters. Strategies are fixed mixes; evolution is the next step.

**What was built.**

- **Theft by stumbling** (`theft.find`, 0 to 1, default 0). An agent that arrives on a site
  holding other agents' caches makes one draw for each of them, in owner-id order, and finds each
  with chance `find`. Staying on a site counts as arriving.
  - It takes the first cache it finds, so there is **one take per arrival**, and the take replaces
    that tick's harvest.
  - Nobody plans a theft: caches an agent hasn't found aren't among its candidate sites.
  - **The dig wins.** An owner digging its own cache makes no draw.
  - With `find` 0 nothing is drawn, and every world is its Minds 5 self (the goldens are
    unchanged).
- **The owner's memory** (`theft.owner_memory`, on by default). With it off, owners don't know their
  own caches. An owner finds its own cache only by stumbling on it, at rate `find`, like anyone
  else. This is Andersson and Krebs's equal-recovery case as an equal chance per draw. Owners bury
  where they stand, so they still cross their own caches far more often than others do, and
  without memory their recovery rises (p_s 0.015 → 0.27 in `theft-winter-half`).
- **Loot** (`theft.loot`).
  - `keep` (the default) puts the take into holdings, up to the carrying limit, where a hoarding
    thief's rule may bury it again. Reciprocal pilfering then arises; it isn't scripted.
  - `eat` puts the whole cache into the thief's **stomach**, which metabolism draws on before
    holdings. Stomach sugar can't be buried again.
  - The spec had said eaten loot adds nothing, which starved thieves. The stomach is a spec
    amendment.
- **Cheaters** (`theft.cheaters`, a share of the founders). A cheater never buries. Agent i is a
  cheater iff ⌊i·s⌋ > ⌊(i − 1)·s⌋, an exact share with no draw. A child takes its acting parent's
  way. Everyone pilfers, hoarders included.
- **A bury cost** (`caching.bury_cost`, field worlds only). Burying q costs q × cost more, counted
  as eaten. This is Andersson and Krebs's C per unit G.
- **Every cache's fate** can be logged while theft is on (recorded only when a caller asks for it,
  as the survey does; the app doesn't), with the tick it was buried and the tick it ended: dug by its owner, pilfered (and by whom), lost with a dead owner, or still buried.
  - The log closes records oldest first.
  - Sugar cached before theft came on is backfilled the first time its cache is touched.
  - Sites + holdings + caches + stomachs + eaten + what left with the dead is conserved through
    theft under either loot rule (tested over 300 ticks).
- **A survey-only probe.** Owners dig below their whole reserve R instead of R/2. It is not a
  setting, and it serves to test whether the owner's advantage goes unused.

**Where `find` comes from.** The spec asked for a `find` that puts daily pilferage inside the
field's 2–30 %. In the winter world the rate came out at about 0.09 × `find`, and none of 0.01–0.2
reached 2 % (0.2 gave 1.83 % over the winter). The list was extended mechanically, and the smallest
value to reach 2 % was 0.25 (2.21 %). **`find` is a free parameter, and 0.25 is an anchor, not a fit
to the field.** The survey judges every claim at 0.25 and reports each one across `find` 0.02, 0.05,
0.1, 0.25, 0.5 and 1. The worlds are:

- **The winter field:** Minds 5's `cache-winter-even` unchanged, plus theft.
- **The arena:** 2, 4 or 8 agents shut in a room walled on all four sides, with each agent's sugar
  and regrowth equal at every n and vision scaled to the room. Half the agents are cheaters.

In the arena without theft, agents who don't cache survive the first winter 0–2.0 % of the time,
against 92.8–99.5 % for hoarders (100 seeds per n).

*2026-09-30: the arenas are now walled on all four sides.* Each room had been a (k + 1)-torus with a
wall along row 0 and column 0 only. That closed the room on the torus, but on the page the room sat
against the bottom and right edges and looked offset. Each is now a (k + 2)-torus with walls along
every edge and the room centered. Every arena number below was re-measured; no verdict changed
(claim 7's arena wins at the anchor went from two configurations to three).

**The survey** (20 seeds, ticks 1–200, a summer and the first winter; claims and thresholds set
before running; causes "likely" unless isolated). Survival is alive at 200 ÷ alive at 100, and every
figure is also reported per founding agent.

- **1. Pilferage is visits × find, and it Fails the pre-set tolerance narrowly.** At the anchor 2.3 %
  of caches are pilfered a tick. v, the non-owner draws per cache per tick, is 0.118, so v × `find`
  is 2.94 %. The ratio of the two has a median of 0.789, against a tolerance of 0.8–1.2 (0 of 20 seeds
  inside). The shortfall matches the one-take rule's prediction (within 3 %). An arrival on k foreign caches takes one with chance
  1 − (1 − `find`)^k, not k × `find`. Summed over arrivals, that predicts the takes within 3 % at
  every `find` (0.997 at the anchor). 85 % of draws at the anchor are on sites with two or more
  foreign caches.
- **Stumbling can't reach the field's median.** The rate is at most v, reached at `find` 1, and v is
  mostly set by crowding (0.08 agents per open site). v also falls as `find` rises (0.146 to
  0.078), likely because the caches left are on sites fewer agents cross. Even finding every cache
  stood on loses 7.0 % of the caches a tick (10.9 % of the sugar), against Vander Wall and Jenkins's
  median of 9 %. The band is context, not a target. Animals that reach those rates search for
  caches, and watching others cache is the campaign's next step but one.
- **Theft pools the stores.** First-winter survival in the winter field is 74 % without theft, 90 %
  at the anchor and 98.5 % at `find` 1, and winter deaths fall from 42 to 2.5. The likely reason is
  that a thief is often a hungry agent walking onto someone else's surplus. It isn't monotone
  everywhere: in `cache-winter-mixed`, winter deaths fall from 51.5 to 10.5 at `find` 0.5 but rise
  to 20.5 at 1.
- **2. Andersson and Krebs's threshold is Weak** (186 of 300 arena runs agree, 62 %). Their condition
  is p_s/p_o > (C/G)(n − 1) + 1, with p_s and p_o measured from cache fates. It holds in only 13
  runs, all at n = 2. Hoarders end richer in 127 runs, 114 of them where it fails. **This depends on
  valuing still-buried caches.** Counting them as nothing, agreement is 90.7 % and hoarders are
  richer in 17 runs, not 127. As a fitness measure, wealth per founder (the dead as 0) is
  pre-registered, because nearly every agent survives the arenas.
- **3. Frequency independence Fails.** The hoarder's survival advantage falls as cheaters grow more
  common. Its slope is −0.17 per unit share (95 % CI −0.22 to −0.12), and 0 of 20 seed slopes are
  above 0. Cheaters survive better than hoarders at all nine shares. The slope's sign flips with
  `find`: +0.30 at 0.02, +0.22 at 0.05, a CI that includes 0 at 0.1, −0.53 at 0.5 and −0.62 at 1.
- **Cheaters win by transfer.** With nobody finding caches (`find` 0), cheaters survive 48–56 %,
  which is the 49 % of `cache-winter-none`, where nobody caches. Their lead at the anchor is sugar
  taken from hoarders. It isn't zero-sum, though: with a tenth cheaters, hoarders survive 89 %,
  against 73 % at `find` 0. By amount, hoarders pilfer 5.5–6.9 times as much per founder as cheaters
  in the winter field (the arenas show 1.2–1.8 times at n = 4 and 8 with no bury cost, and none at
  n = 2, where the cheater has no caches), consistent with Vander Wall and Jenkins (p.661): "food
  hoarders are expected to pilfer far more than they can consume, recaching the excess".
- **The owner's advantage goes nearly unused.** In `theft-winter-half`, p_s (owners' share of the
  sugar dug or pilfered) is 0.015, and p_o (thieves' share of all that ended) is 0.977. Owners dig
  only below half their reserve, and 99.7 % of their digging falls in winter. Thieves take caches
  all year, 47 % in summer. The probe that digs below R raises p_s to 0.28, and the gap between
  cheater and hoarder survival shrinks from 13 points to 3 (hoarders 82 % → 93 %, cheaters 95 % →
  96 %). This is likely much of the cheaters' lead, but not all of it, since cheaters still lead.
- **4. Equal recovery Holds.** With owner memory off, cheaters survive better at every share, in
  19–20 of 20 seeds, and at every `find`. The median lead grows from 0.11 at a tenth cheaters to
  0.57 at nine tenths. What is equal is the chance per draw, not recovery: owners bury where they
  stand, so they still cross their own caches far more often, and without memory their recovery
  rises. In `theft-winter-half` p_s goes from 0.015 with memory to 0.27 without (p_o 0.98 → 0.71),
  and owners find their own caches 25 800 times over the 20 seeds, 19 200 of them in summer
  (reported, not judged).
- **5. No mixed equilibrium at the anchor (Fails).** The hoarder − cheater advantage never crosses
  zero; its median is −0.09 to −0.18. Andersson and Krebs assume hoarders are poorer thieves. Here
  every agent finds a cache it stands on at the same rate per draw. At `find` 0.02 and 0.05 the
  median curve does cross, at shares 0.37 and 0.79, in Andersson and Krebs's orientation. Its
  stability is the next step's question.
- **6. Reciprocity splits (Fails as judged).** `keep` stores far more than `eat`: 86 against 47
  caches per founding hoarder at tick 100, in 20 of 20 seeds. But hoarders survive better under
  `eat` (93.3 % against 90.4 %; keep ahead in 2 of 20 seeds). `eat` leads at `find` ≤ 0.25 and `keep`
  at 0.5 and 1. The likely reason is that eaten loot feeds the thief and can't be stolen again,
  while kept loot is buried again and pilfered again.
- **7. Vander Wall and Jenkins's 18 % is Untestable.** Nothing at the anchor reaches it (2.6 % at
  most), and nothing in the sweep does either (8.9 % at `find` 1). Hoarders win only at low rates.
  At the anchor that is arena-2 with a bury cost of at most 0.1 (1.45 % a day) and arena-8 with none
  (2.18 %). Counting still-buried caches as nothing, hoarders win in none of the 15 arena
  configurations at the anchor.
- **8. The mild-winter cheater Fails on wealth; a reported holdings reading (not judged) shows
  cheaters ahead at every β, gaining slightly as winter softens.** On pre-registered wealth, cheaters fall further behind as winter softens: −98 per
  founder at β 2, against −10 at β 32. The slope on log₂β has a median of +22.3, and 0 of 20 seeds
  are below 0. But wealth counts the hoarders' still-buried caches (123 per founding hoarder at β 2).
  On holdings plus stomach, cheaters lead at every β (25.1, 26.4, 27.1, 25.0 and 19.7 from β 2 to
  32), and they gain slightly as winter softens (a median slope of −1.20, below 0 in 20 of 20
  seeds). Survival ties at β ≤ 8, where everyone survives.
- **9. Theft acts (Holds):** pilfering happens in 20 of 20 seeds of every preset. In `theft-winter`
  a seed buries 74 000 and thieves take 66 000, likely much of it buried again. By tick 200, 88.8 % of the
  sugar ever buried has been pilfered, 0.3 % dug by its owner, 0.2 % lost with the dead and 10.8 %
  is still buried.
- **The arena: pilferage rises with n** (1.45, 1.77 and 2.18 % a tick at n = 2, 4 and 8). v rises
  from 0.059 to 0.075 to 0.094, while agents per open site stay at 0.11–0.125. More strangers cross
  each cache, as in Andersson and Krebs's reason for their (n − 1). Thieves take 51 %, 85 % and 91 %
  of the ended caches' sugar. In the 2-agent room the hoarder steals nothing, because the cheater
  has no caches.

The valuation rows are reported beside the verdicts; they are not judged and don't change them.

| Claim | Verdict | Depends on valuing still-buried caches? |
|---|---|---|
| 1. Pilferage = visits × find | Fails (0.789 against 0.8) | no |
| 2. Andersson and Krebs's threshold | Weak (62 %) | yes: 90.7 % with caches at 0 |
| 3. Frequency independence | Fails (slope −0.17) | no (survival) |
| 4. Equal recovery | Holds | no (survival) |
| 5. A mixed equilibrium | Fails at the anchor (crossings at `find` 0.02 and 0.05) | no (survival) |
| 6. Reciprocity | Fails (stores Hold, survival Fails) | no |
| 7. The 18 % hoarding withstands | Untestable (max 8.9 %) | yes: arena wins 3 → 0 at the anchor |
| 8. The mild-winter cheater | Fails on wealth | yes: on holdings cheaters gain as winter softens, 20 of 20 |
| 9. Usage | Holds | no |

**The sweeps** (built in; observations, not judged). `theft-find` is the pilferage rate over the
first winter against `find`. `theft-cheaters` and `theft-winter` are the hoarders alive at the end of
the first winter against the share of cheaters and against β. The sweep format takes one metric
with no ratio, so per-capita comparisons come from the survey.

**Cost** (µs per agent-tick, CPU time, measured as in Minds 2–5). The full table is in the program
document. `theft-winter` costs 17.9, 1.8 times `cache-winter-even`. That is likely partly the larger
population theft keeps alive, and partly the find draws and the fate log (not isolated; the log was
recorded then, and is now kept only when the survey asks for it). With a
quarter or half cheaters it costs 15.2 and 11.6. The arenas cost 2.50–3.95, mostly per-tick work
that doesn't scale with 2–8 agents (timed with the two-walled rooms; the four-walled ones timed
alongside, under load, within noise of them).

Switches: the Rules panel's **Theft (Minds 6)** group has **Chance to find a cache**, **Loot**
(keep it, or eat it on the spot) and **Bury cost**, all live. **Owners remember their caches** and
**Share of cheaters** rebuild the world. Inspect shows whether an agent is a cheater, what it **Stole**
and **Lost to thieves**, and its **Stomach** when not empty. Charts: **Pilferage** and **Cache
fates** when theft is on; **Hoarders vs cheaters** (alive) and **Hoarder and cheater holdings** when
there are cheaters. Presets: `theft-winter`, `theft-winter-quarter`, `theft-winter-half`,
`theft-arena-2`, `theft-arena-4` and `theft-arena-8`. Built-in sweeps: `theft-find`,
`theft-cheaters`, `theft-winter`.

Credit: M. Andersson and J. Krebs, "On the evolution of hoarding behaviour," *Animal Behaviour* 26
(1978), 707–711; S. B. Vander Wall and S. H. Jenkins, "Reciprocal pilferage and the evolution of
food-hoarding behavior," *Behavioral Ecology* 14 (2003), 656–667. See
`docs/superpowers/specs/2026-09-30-minds-6-theft-design.md`.

### Minds 7: the evolution of hoarding

The seventh step of the Minds program (`docs/studies/2026-09-27-minds.md`) and P1b of its pilfering
campaign. Unlike Minds 1–6 it starts as a reproduction. Vander Wall and Jenkins (2003) argued that
scatter hoarding persists despite heavy pilferage because pilfering is reciprocal, and that the
cheater most likely to damage scatter hoarders is one who pilfers and keeps the loot "in a
defensible larder" (p.661). They tested this with a genetic algorithm: 20 animals, a 100-day season,
60 generations, and heritable probabilities of larder hoarding (L) and of defending the larder (D).
Their result is a threshold in how visible scattered caches are relative to larders: "For ratios
less than 0.2, high probabilities of larder hoarding never evolved; for ratios greater than 0.3,
high probabilities of larder hoarding almost invariably evolved" (p.663). Minds 7 builds that model
as written, tests what they report, and then varies what they held fixed.

**What was built.**

- **A new model kind, `hoard`, built from the paper's Appendix** (pp. 664–666). It is non-spatial,
  and nothing in the Sugarscape world changed. One tick is one foraging bout: 20 bouts a day, 100
  days a season, one season a generation. Public food is 82 items on day 1 falling to 2 on day 50,
  2 100 in all (p.665). On days 1–5 everyone is fed by food that can't be stored; from day 6 an agent
  that eats nothing all day starves.
- **A bout:** owners eat from their stores in the day's first bout (larder first; a larder eater
  stays home and defends), satiated agents draw whether to defend, predation strikes at 0.0001 a
  bout, and everyone else forages in a fresh random order. A search finds an item with
  P = 1 − e^(−λ), λ growing with the food available to that agent: public food, plus others'
  scattered items × app_scat, plus others' larders × app_lard. A found larder is raided one item a
  bout until the day ends or its owner stays home. What an agent doesn't eat it stores, in its
  larder with probability L, otherwise scattered.
- **Breeding:** parents drawn in proportion to the survivors' leftover stores; L and D inherited on
  the logit scale with heritability 0.8 and segregation variance V_seg.
- **Gaps filled as stated choices.** The paper doesn't print the defense logistic's slope (we use
  10), V_seg ("a moderate amount of genetic variation", p.662; we use 0.5), what a search that lands
  on a defended larder gets (nothing), the stores of the dead (they stay, open to raids), the food
  line's exact form, or how "larders take over" is measured (the hoarders' mean L above 0.95 over
  generations 51–60). Each choice is recorded in the spec's amendments, with its reason, before any
  survey run. The paper's 186 % daily larder loss can't be a proportion; we measure loss as items
  lost per item-day held, a hazard that can exceed 1.
- **Named switches**, defaults reproducing the above bit for bit: `larder_weight` (per burrow or per
  item, below), `defense_slope`, `v_seg`, `defended_in_pool`, `dead_stores` and `early_bout1_eats`
  (the gap choices); and as new ground, `owner_recovery` (an owner finds its own scattered cache only
  with this chance; Vander Wall and Jenkins give free recovery), `cheaters` (a share of founders that
  never store, Minds 6's cheater, dealt by id) and `cheater_fitness` (leftover stores, as for
  everyone, or survival weighed as an average surviving hoarder's stores).

**Per burrow, and why the default changed.** The paper can be read two ways on how a larder is
found. The Appendix sums "all larder hoards of other individuals weighted by apparency of larder
hoards" (p.665), which can weigh every item in a larder. But p.662 defines app_lard as "the relative
probability of detecting a burrow of another individual while searching for food", and p.664 has
foragers encounter "public items, as well as scatter hoards and burrow entrances of other
individuals, with probabilities determined by the apparency values". We first chose per item, on the
grounds that only the Appendix gives a formula. Under per item, larders never took over in 0 of 15
diagnostic runs at app_scat 0.1, 0.44 and 0.8, and those runs prompted the change (the survey, run
after it, kept per item as a full row: 0 of 1 350 runs at ratios up to 0.45). A diagnosis found why: a larder's chance of being found grows with its size, raids start from
those finds and cause 84 % of larder loss, and so larders never grow large enough to be defended.
Re-reading the text with that in view, per burrow (each other agent's non-empty larder counts
app_lard once) is the better-supported reading, and the Appendix sentence doesn't settle it, since
each animal has one "larder hoard". **Per burrow became the default, and the change was prompted by
the result.** Per item stays as the named switch, and the survey runs its claims under both readings
as full rows.

**The survey** (50 runs a condition, seeds 1–50, every run the full 60 generations; Vander Wall and
Jenkins report 35 runs. The grid is app_lard 1, 2 and 3 by app_scat/app_lard 0.05–0.45 with app_scat
≤ 0.9, their range: 27 cells, 1 350 runs per reading. Judges and thresholds were committed before
any run; causes are "likely" unless isolated; per burrow unless stated.)

- **1. All-or-nothing, but slower: Fails on speed.** "In 35 runs of the model, average probability
  of larder hoarding always remained less than 0.2 or increased rapidly to more than 0.95, usually
  within 10 generations (Figure 2A)" (p.663). The ending reproduces: 1 331 of 1 350 runs (98.6 %)
  end below 0.2 or above 0.95, and the other 19 are intermediate (11 of them average below 0.2 but
  cross it in some generation). The speed doesn't. Only 68 of 634
  takeovers pass 0.95 by generation 10 (10.7 %); the median is generation 15 (IQR 12–20). Read the
  other way, within 10 generations of lifting above 0.2, it is 239 of 634 (37.7 %), still not
  "usually". Their own Fig. 2A example passes 0.95 near generation 16, and in that cell (app_lard 1,
  ratio 0.30) ours pass at a median 16.5. V_seg likely sets the speed: at ratio 0.4, the takeovers
  passing 0.95 by generation 10 are 0 of 31 at V_seg 0.25, 6 of 49 at 0.5 and 35 of 50 at 1.
- **2. The threshold is Weak.** A logistic fit of takeover on the ratio gives logit = −5.98 +
  25.45 × ratio (McFadden's ρ² 0.50), with its 50 % point at **0.235** (SE 0.003) against their
  **0.219** (Fig. 2B: "logit = −8.00 + 36.59 × ratio; McFadden's ρ2 = 0.61", p.662). That is within
  the pre-set ±0.03, so the 50 % point Holds, though our curve is shallower. Above 0.3, 285 of 300
  runs took over (95 %, Holds). Below 0.2, 7 of 450 did, all at 0.15, where they report none
  (Weak). Fitted per app_lard, the 50 % point is 0.232, 0.237 and 0.236, so the ratio alone nearly
  sets it.
- **3. Larder loss above scatter loss: Weak** (1 266 of 1 350 runs, 93.8 %; they report "all 35
  simulations"). All 84 misses are takeover runs at ratios 0.35–0.45. Over generations 1–10 the
  medians are 90 % a day for larders and 27 % for scatter. Their 186 % and 24 % are context only,
  since their rate is undefined.
- **The CV judge was omitted, and is disclosed.** The spec's amendments called for judging the
  spread of loss rates ("the coefficient of variation in rate of loss of larder-hoarded items (57%)
  was almost twice that of scattered caches (33%)", p.663). The judge commit left it reported, an
  omission made before any run, and no judge was added after. The larder/scatter CV ratio is 2.85
  within runs (1.49 across runs) against their 1.73: not matched under our primary definition.
- **4. The predictor Holds, against a weak baseline.** "If one or more individuals in the first 10
  generations lost items from larders at a lower rate than the average rate of loss of scattered
  caches in the population, then larder hoarding usually became established" (p.663). With a
  pre-registered floor (an agent counts only if its larder averaged at least one item), that rule
  is 84.5 % accurate against takeover, against 59.3 % for comparing the means, and takeover follows
  in 76 % of the 811 runs it predicts. But the ratio alone (takeover at ratio ≥ 0.25) is 83.7 %
  accurate, and generation 1 alone gives 76.5 %. The predictor is also partly circular, since early
  takeovers make larders large and defended. So it is not claimed to explain the outcome.
- **5. Scatter withstands about the loss they report.** In runs without takeover, scattered caches
  lose 20.5 % a day over generations 1–10 (20.8 % over 1–60), against "18% in cases in which larder
  hoarding did not become established" (p.663). Reported, not judged.
- **Neither reading of the larder reproduces both.** Per item reproduces the loss statistics: larder
  loss above scatter in 1 350 of 1 350 runs, a CV ratio of 1.75 against their 1.73, and a 210 %
  median larder loss against their 186 %. But larders never take over (threshold and predictor
  Fail). Per burrow gets the outcome and misses those. Likely their detection or defense differs
  from both readings in some unstated way, and per burrow may stand in for that difference rather
  than being their rule.
- **The reproduction is contingent on three choices.** Larders weighted per burrow (chosen after per
  item failed), a defense slope of 10 and V_seg 0.5 (both fixed before any run, and neither printed).
  At app_lard 2 the fitted 50 % point in the sensitivity runs is 0.236 at those values, 0.351 at
  V_seg 0.25, 0.174 at V_seg 1, and about 0.09–0.14 at slope 4 (two of the three fits
  near-separated), where larder loss also falls below scatter loss in most
  runs. Every other pair tried misses 0.219 by more than ±0.03. And the V_seg that speeds the rise
  (1) moves the 50 % point to 0.174, so no single value fits both. Vander Wall and Jenkins explored
  this: "Results of a full sensitivity analysis will be reported elsewhere" (p.665). Three searches
  did not find it, so it is unfound.

| Claim | Verdict | Per item |
|---|---|---|
| 1. All-or-nothing | Fails (98.6 % end low or high; 10.7 % rise by generation 10) | Weak, vacuously (nothing takes over) |
| 2. Threshold | Weak (50 % point 0.235 against 0.219; 7 of 450 below 0.2) | Fails (no takeover) |
| 3. Larder loss above scatter | Weak (93.8 %) | reported: 1 350 of 1 350 |
| CV ratio about 1.7 | Not judged (omission disclosed): 2.85 | 1.75 |
| 4. The best early larder predicts | Holds (84.5 % against 59.3 %; the ratio alone 83.7 %) | Fails |
| 5. Scatter withstands 18 % | Reported: 20.5 % | |

**New ground** (reported, not judged).

- **Visibility.** Vander Wall and Jenkins predicted, untested, that "the prevalence of scatter
  hoarding in communities should depend on environmental conditions that affect the apparency of
  scattered caches and larders" (p.663). Takeover rises with app_scat at every app_lard, apart from
  small dips, from none at the lowest values (app_scat up to 0.15, 0.2 and 0.3 for app_lard 1, 2
  and 3) to 78–100 % at the top of each row. At a fixed app_lard this is the ratio axis again; at a
  given ratio the three app_lard values give similar shares (at 0.25: 70, 66 and 58 %).
- **Owner recovery.** Making owners find their own scattered caches doesn't clearly change the
  outcome. At ratio 0.22 takeovers are 23, 28, 21 and 21 of 50 for recovery 0.25, 0.5, 0.75 and 1,
  with no trend. At ratio 0.1 nothing takes over at any recovery, and survival falls from 82 % a
  season with free recovery to 74 % at 0.25, likely because a missed meal late in the season is a
  starved agent.
- **The cheater.** Vander Wall and Jenkins suggested, untested, that "under ideal conditions (e.g.,
  mild winters), a nonhoarding cheater could survive and even flourish" (p.661). Under their fitness
  rule (leftover stores) a cheater holds nothing and is never a parent: cheaters are gone by
  generation 2 in all 50 runs, though in generation 1 they survive about as well as hoarders (a
  median 80 % against 87 %). If a surviving cheater instead weighs as much as an average surviving
  hoarder's stores (`cheater_fitness = survival`), cheaters persist at about 6 % of births by
  generation 60 (never gone in 3 of 50 runs), survive somewhat less well than hoarders (0.71 against
  0.80), and larders take over in 33 of 50 runs against 21 without cheaters and 22 under `stores`.
- **Where larders take over, about half the agents die each season, mostly starving.** In
  `hoard-larder` survival is 0.50 a season, against about 0.82 where scatter holds; predation alone
  takes about 18 %. A review probe found 92 % of the starvations after day 50, the starved having lost
  a median 478 larder items to raiders. Likely, larders raided empty after public food ends on day 50
  leave their owners nothing, as the paper notes: "Starvation also occurred in the second half of the
  hoarding season if the food stores of an individual were depleted" (p.664). The cause is not
  isolated.

**The presets** (50 runs each, 60 generations). `hoard-threshold` (ratio 0.22): 21 takeovers.
`hoard-scatter` (0.1): none; scattered caches lose 18 % a day and larders 180 %. `hoard-larder`
(0.4): 49 takeovers, rising a median 13 generations in. `hoard-no-free-recovery` (recovery 0.5): 28.
`hoard-cheaters` (a quarter cheaters): 22.

**The sweeps** (built in; observations, not judged). Built-in sweeps cap at 100 000 ticks, so they
read the hoarders' mean L in generation 50, not the generation-60 takeover. `hoard-ratio` sweeps
app_scat at app_lard 2; `hoard-recovery` sweeps owner recovery; `hoard-cheaters` reads survivors in
generation 1 against the share of cheaters, since under Vander Wall and Jenkins's fitness rule the cheaters are gone after it.

**Cost** (µs per agent-bout, CPU time, the full 60 generations; the table is in the program
document). 0.30–0.47, a fifth or less of the cheapest Sugarscape figure in the Minds tables, since
there is no grid: a bout is a few draws and one weighted pick. A full run of about 2 million
agent-bouts takes under a second. `hoard-larder` is the cheapest, likely because agents who stay
home to defend don't search (not isolated).

The page: Minds 7 sits in the Minds menu as "Minds 7: evolution of hoarding". The frame is one column
per agent: its state on top (hungry, fed, defending, raiding, cheater, dead), its L and D, and its
larder as a bar up and its scattered caches down. The toolbar shows the generation, day, bout and
public food. Charts: **Larder probability (L) by generation** (all agents and hoarders only),
**Defense propensity (D)**, **Survivors**, **Larder share** and **Loss rates** by generation, and
**Larder share this season** by bout. Inspect shows an agent's traits, stores, losses and loss rates,
and how it died. The Rules panel groups the parameters (Season, Run, Food, Search, Apparency, Risk,
Inheritance, Founders, Defense, Switches) and says which apply live and which rebuild the world.
Presets: `hoard-threshold`, `hoard-scatter`, `hoard-larder`, `hoard-no-free-recovery` and
`hoard-cheaters`. **Compare** entry: "Scattered caches hard vs easy to find — Minds 7 (Compare)".
Built-in sweeps: `hoard-ratio`, `hoard-recovery`, `hoard-cheaters`.

Credit: S. B. Vander Wall and S. H. Jenkins, "Reciprocal pilferage and the evolution of
food-hoarding behavior," *Behavioral Ecology* 14 (2003), 656–667. See
`docs/superpowers/specs/2026-09-30-minds-7-hoarding-evolution-design.md`.

### Minds 8: watching

This is our own experiment, not a reproduction: the eighth step of the Minds program
(`docs/studies/2026-09-27-minds.md`) and P2 of its pilfering campaign. Minds 6's thieves only
stumble on caches. Ravens do better by watching. Bugnyar and Kotrschal (2002) found that of 29
caches made in front of observers and not taken back at once, "all were found during retrieval
sessions by conspecifics that had been in the pathway during caching", against one of 13 made
unobserved. Heinrich and Pepper (1998) found that ravens "recovered none of 40 artificial caches",
and that observers raided seen caches the next day but none at 14 days. Minds 8 gives agents that
observational memory: an agent who sees another bury remembers the cache, and goes to take it on
purpose. It asks whether watching takes fresh caches faster than stumbling, whether it breaks
Andersson and Krebs's condition where that condition held, and whether scrounging on what others
bury is worth less as it becomes common (Barnard and Sibly 1981).

Minds 8 ran in two rounds before merging. Four audits found that three of the first design's five
judged results were foreseeable from Minds 6's numbers, so they weren't tests. The second round
(`docs/superpowers/specs/2026-10-01-minds-8-second-round-design.md`) replaced them, after a
pre-mortem of what each hypothesis predicts. Its results are the milestone's.

**What was built.**

- **Watching** (`watching.on`, off by default). With it off nothing is observed, allocated or drawn,
  and every world is its Minds 7 self (the goldens are unchanged).
  - **Who watches** (`watching.watchers`, a share of the founders, default 1). Agent i watches iff
    ⌊i·s⌋ > ⌊(i − 1)·s⌋, Minds 6's cheater rule, so at equal shares the watchers and the cheaters
    are the same agents. A child born to parents doesn't watch. `watching.who` (reset-only) can
    instead make every hoarder (`hoarders`) or every cheater (`cheaters`) a watcher.
  - **Seeing a burial.** When an agent buries, every living watcher other than the owner whose sight
    covers the site remembers the cache: the site is on one of the four lattice lines from it,
    within its vision, and not behind an opaque wall. The owner never knows it was seen.
  - **Memory** (`watching.span`, default 7 ticks). Heinrich and Pepper give recovery the next day
    and none at 14 days. Bednekoff and Balda's pinyon jays relocated exact sites at 2 days but only
    general areas at 7 (1996a, p. 823), and Bugnyar and Kotrschal summarize observational memory as
    "a few days" (p. 189). So 7 is at the generous end of what's known. Every claim is reported at
    span 1, 2, 3, 7 and 13. An entry is forgotten when it is older than the span, when its watcher
    arrives on the site, or when its watcher dies.
  - **Going to a seen cache.** Remembered caches join the watcher's candidates like its own caches,
    valued at the amount it saw buried (`value: amount`, the default), or at that capped by its room
    under the carrying limit (`value: room`). A cache emptied since still looks full. A walk that
    finds no path to a seen cache gives it up.
  - **A raid.** On arrival, after an owner's dig and before Minds 6's stumbling, a watcher takes
    from the first remembered cache still there, in owner-id order, with no draw, by Minds 6's
    `loot` and carrying-limit rules. A raid is a pilfer, and it replaces that tick's harvest. Under
    `raid_if: better` (the default) it raids only when the remembered amount there is at least what
    the site would give it, and otherwise harvests and keeps the memory. `raid_if: always` was the
    first design's rule. If every remembered cache there is gone, the raid is wasted.
  - **When to raid** (`watching.raid_when`). `always` (the default), or `hungry`: only while the
    watcher holds less than R / 2, on the owners' terms.
  - **Scroungers who forgo** (`watching.scrounge: forgo`; default `harvest`). A scrounger (a watcher
    who is a cheater, so never caches) holding a fresh memory chooses only between its seen caches
    and staying put, and harvests nothing on any tick it doesn't raid. This is Barnard and Sibly's
    assumption that scrounging excludes producing.
  - **Digging earlier** (`caching.dig_below: reserve`; default `half`). Owners dig when they hold
    less than their whole reserve R, not R / 2. Claim 2's calibrated world needed it.
- **The counts:** burials seen, sightings, raids, sugar raided and wasted raids, pilfers split into
  seen and stumbled, and watcher against non-watcher survival and wealth per founder. Sugar is
  conserved through raids under every switch.
- **A survey-only probe.** A raid that took something also harvests the site that tick. It is not a
  setting; it measures what replacing the harvest costs.

**The survey** (`survey/out/minds8b-results.md`; claims, thresholds and a calibration rule committed
before any run; causes "likely" unless isolated). It covers ticks 1–200, a summer and the first
winter. Seeds 1–20 for claims 1 and 2, and 1–60 for claim 3. Span 7 is judged. Fitness is
agent-ticks alive per founder over ticks 1–200 ÷ 200.

- **1. Watching takes fresh caches faster than stumbling (Holds).** Of the caches buried in ticks
  1–90 of `watch-winter` (everyone watching, no stumbling), 6.6 % are taken within a day and 36 %
  within a week: a hazard of 6.1 % a day, against 3.0 % for Minds 6's stumbling, higher in 20 of 20
  seeds. Watchers see 88 % of burials and raid 54 % of the caches they see, so the limit is acting
  on what they see, not seeing it: action-bound by the rule fixed before the run, in every world and
  setting. Watching costs lives. 59 % of the founders survive the first winter, against 70 % without
  watching. About two thirds of that cost is the harvest a raid replaces (the probe lifts it to
  66 %).
- **2. Andersson and Krebs.** A calibration, its rule committed first, found the first world on a
  fixed list where p_s > p_o held without watching: `theft-winter-half` at `find` 0.02 with owners
  digging below their whole reserve (20 of 20 seeds). With everyone watching (`watch-ak`), the
  condition **flips (Holds)**: p_s ÷ p_o falls from 3.57 to 0.67, below 1 in 20 of 20 seeds. But
  **fitness doesn't follow (Fails)**: across watching off and spans 1, 3, 7 and 13, the sign of the
  hoarders' advantage matches the condition's in 62 of 100 runs. Neither **being watched (Fails)**
  nor **the hoarders' own raiding (Fails)** costs them 0.05 in any seed. Watching only by the
  cheaters costs the hoarders 0.012 of advantage (95 % interval 0.008 to 0.017), and watching only
  by the hoarders changes their fitness by no detectable amount.
- **3. Producers and scroungers.** Where scrounging excludes producing (`watch-scroungers-forgo`),
  the scroungers trail, and **trail further as they become common (Holds)**: the mean per-seed slope
  on their share is −0.20 (95 % CI −0.23 to −0.17). But **no stable mix forms (Fails)**: they are
  ahead at a share of 0.1 in only 13 of 60 seeds. Never caching costs a little on its own: with
  watching off (a baseline added after the run, reported only) the same agents trail by about 0.02
  at every share, a median 0.015 at 0.1. Per seed, watching adds a median 0.034 to the shortfall at
  0.1 (in 47 of 60 seeds) and 0.204 at 0.8, so the shortfall when rare is mostly scrounging's, and
  its growth with share entirely so. Within scrounging, forgoing is the cost, since letting them
  harvest too raises their fitness by 0.13 (at half, seeds 1–20). Among watchers who also bury,
  **no fall in their lead
  was detected (Fails, flat)**: slope 0.011, 90 % CI −0.004 to 0.027. Instead it is **a mild social
  dilemma (Holds)**: watchers are ahead by 0.009 (95 % CI 0.003 to 0.016) while everyone's fitness
  falls as more watch.
- **The switches.** `raid_if: always` costs 6 points of survival in `watch-winter` and changes no
  verdict. `value: room` pays where agents carry close to their limit (+6 to +7 points in
  `watch-half` and `watch-ak`) and changes no verdict.
- **An upper bound, sensitive to the span.** Watchers here see every burial in sight and remember it
  exactly, cachers never hide and owners never defend (both P3), and a 7-tick memory is at the
  generous end of what's known. With a memory of 1 or 2 ticks, three results reverse and the dilemma
  goes undetected (through span 3). Watching's hazard falls below stumbling's (0.85 % and 2.49 % a
  day), the condition doesn't flip (1.72 and 1.10), the forgoing scroungers' shortfall shrinks as
  they become common, and the burying watchers' lead isn't detected (pooled 0.0007 and −0.0015; also
  −0.0027 at span 3), so 3c holds only at spans 7 and 13. Bednekoff and Balda's pinyon jays
  relocated exact sites after two days and only the general areas after seven (1996a, p. 823).

Judged at span 7; the span notes are from the reported spans 1, 2, 3, 7 and 13.

| Claim | Verdict |
|---|---|
| 1a. Watching takes fresh caches faster than stumbling | Holds (6.1 % against 3.0 % a day; 20 of 20 seeds). Holds at 3–13; reverses at 1–2 |
| 2a. Watching flips Andersson and Krebs's condition (`watch-ak`) | Holds (p_s ÷ p_o 0.67 against 3.57; 20 of 20). Reverses at 1–2 |
| 2b. Fitness follows the condition | Fails (62 of 100 runs) |
| 2c. Being watched costs hoarders 0.05 | Fails (0 of 20; mean 0.012) |
| 2d. Their own raiding costs hoarders 0.05 | Fails (0 of 20; mean 0.003, not detected) |
| 3a. Scroungers who forgo: shortfall grows with share | Holds (slope −0.20). Reverses at 1–2 |
| 3a. Watchers who bury: lead falls with share | Fails, flat (0.011; no fall detected) |
| 3b. A stable mix | Fails (ahead when rare in 13 of 60 seeds; at 0.1 the mean shortfall, 0.056, is 0.021 without watching plus 0.035 that watching adds) |
| 3c. A social dilemma | Holds (lead 0.009; world slope −0.008). Holds at 7 and 13; not detected at 1–3 |

**The first design, superseded.** Its verdicts (the field band, Andersson and Krebs in `watch-half`,
and producers and scroungers in two variants, all Fails; usage Holds) are kept in the program
document with the audits' explanation. The band was a strawman, the ratio sat at a floor, and its
"pure scroungers" still produced their own food. Its write-up over-stated four things, now
withdrawn. Watching wasn't shown unable to reach field rates, and it was action-bound, not
knowledge-bound. Frequency dependence among watchers who bury was not detected, which doesn't make
it absent. And the never-caching agents' shortfall predates watching (48 % against the hoarders'
70 % with watching off), so it isn't a fact about scrounging.

**The sweeps** (built in; observations, not judged). `watch-span` is the first winter's pilferage
rate against the span in `watch-winter`'s world. `watch-scroungers` is the watcher advantage at the
end of the first winter against the share of watchers.

**Cost** (µs per agent-tick; not re-timed in the second round; the table, measured under the first
design's rule, is in the program document). `watch-winter` cost 37.0, about 2.2 times `theft-winter`
(17.1 re-timed), likely from walking the lines out from each burial, ranking seen caches and the
walks to them (not isolated). `watch-scroungers-forgo` and `watch-ak` weren't timed.

Switches: the Rules panel's **Watching (Minds 8)** group has **Watching**, **Span**, **Raid when**,
**Raid if**, **Seen cache value** and **Scroungers**, all live, and **Share of watchers** and **Who
watches**, which rebuild the world. The Caching group has **Dig below**, live. Watchers who bury and
scroungers who never bury are told apart by the Theft group's share of cheaters. The **Watching**
color mode (the default when watching is on) shows watchers who bury, scroungers and agents who
don't watch. Inspect shows whether an agent watches (and is a scrounger) and each cache it remembers
seeing buried: site, owner, amount and how long ago. Charts: **Pilfers by source** (seen, then
raided, against stumbled on) and **Wasted raids** while watching is on; **Watcher survival
advantage** and **Watcher and other wealth per founder** when some founders watch and some don't.
Presets: `watch-winter`, `watch-winter-stumble`, `watch-half`, `watch-scroungers`,
`watch-scroungers-only`, `watch-scroungers-forgo`, `watch-ak` and `watch-arena`. Built-in sweeps:
`watch-span`, `watch-scroungers`.

Credit: T. Bugnyar and K. Kotrschal, "Observational learning and the raiding of food caches in
ravens, *Corvus corax*: is it 'tactical' deception?", *Animal Behaviour* 64 (2002), 185–195; B.
Heinrich and J. W. Pepper, "Influence of competitors on caching behaviour in the common raven,
*Corvus corax*", *Animal Behaviour* 56 (1998), 1083–1090; M. Andersson and J. Krebs, "On the
evolution of hoarding behaviour," *Animal Behaviour* 26 (1978), 707–711; C. J. Barnard and R. M.
Sibly, "Producers and scroungers: a general model and its application to captive flocks of house
sparrows", *Animal Behaviour* 29 (1981), 543–550; W. L. Vickery, L.-A. Giraldeau, J. J. Templeton,
D. L. Kramer and C. A. Chapman, "Producers, scroungers, and group foraging", *American Naturalist*
137 (1991), 847–863; P. A. Bednekoff and R. P. Balda (1996a, b), *Behaviour* 133, 807–826 and
*Animal Behaviour* 52, 833–839. See `docs/superpowers/specs/2026-10-01-minds-8-watching-design.md`
and `docs/superpowers/specs/2026-10-01-minds-8-second-round-design.md`.

### Threshold Models (Granovetter 1978; Watts 2002)

**The crowd.** Each person has a threshold: the share of the crowd he must see join before he joins
(Granovetter). An instigator (threshold 0) acts; whoever's threshold that reaches acts next; and so
on until nobody new is tipped. Everything turns on the exact distribution of thresholds, not its
average. **Watts** put the same rule on a sparse random network — each person watches only his
neighbors — and asked when a single spark becomes a cascade that sweeps the network.

Measured (the survey and the presets' descriptions):

- **Granovetter's crowds reproduce.** Thresholds 0 to 99 give a riot of 100; move the person at 1
  up to 2 and only the instigator riots. His Figure 2's continuous calculation jumps between σ 12.2
  and 12.3 — "about six" rioters below (5.5), "nearly 100" above, 50 in the limit.
- **A crowd of real people has no single tipping point.** A crowd of 100 whose thresholds are the
  normal's quantiles tips at σ 12.23 when thresholds are rounded to whole people (his 12.2), 11.89
  when rounded down, 12.55 when kept as fractions. And crowds drawn at random from the normal
  distribution show no jump at all: they riot past half 15 % of the time at σ 12, 25 % at 12.5.
- **The "equilibrium of 100" is rare.** Of crowds drawn from his uniform city, 36.9 % + 13.7 % =
  50.5 % end with no rioters or one ("over half … .51"), as he says — but everyone riots in only
  2.3 %, and the mean is 12 rioters.
- **The friends claims hold under our reading** (friends at random, counted w times, the actor
  dividing by the whole crowd with himself included, as in his 63/120 example): the uniform crowd's
  most common outcome becomes one rioter; the perturbed crowd spreads more often as friends weigh
  more (0, 0, 43 %, 52 % at weights 1, 2, 5, 10), most at an acquaintance of a quarter, rarely past
  seven rioters; one-way friendships change little.
- **A middling movement between crowds is the most incendiary** (12 % rioting with no movement, 41 %
  at 0.05, 29 % with everyone moving every step), as he suggests.
- **Ceilings make riots pulse.** With some people leaving once more than 90 % riot (his Figure 3),
  most crowds never settle — the riot climbs, the cautious leave, it climbs again — but whether a
  given crowd pulses depends on who holds the ceilings; decided one at a time, it hovers near 90 %.
- **Watts's window reproduces**: cascades between z ≈ 1 and 6 at threshold 18 % (the analytic window
  1.02–5.76), global cascades filling the connected network (0.941 against S = 0.940), and a
  power law of slope ½ at the lower edge (−0.48).
- **His upper edge depends on network size.** At his n 1 000 and z 6.14, 20 % of sparks go global,
  not "a single cascade in 1,000 trials" (3 % at n 10 000).
- **Varied thresholds widen only the dense side of the window**; at the sparse side they narrow it
  (9 % against 28 % at z 1.2). **His Figure 4b cannot be built as stated**: with τ 2.5 and k ≥ 1 a
  power law's mean degree cannot exceed 1.95, and at threshold 18 % no such network cascades.
- **Hubs help in both regimes**: the best-connected spark goes global far more often at z 1.3 (95 %
  against 39 %) and still twice as often at z 5.5 (89 % against 44 %), where he says it does not.

Switches: **Actors**, **Each crowd** (as drawn, or sampled from the city), **Started by**
(instigators, one random actor, the hub), **Actors decide** (together, or one at a time), **Count
oneself in the group**, **Thresholds** (uniform, perturbed, normal, everyone the same), **Mean**,
**Spread**, **A normal crowd is** (quantiles, or drawn), **Thresholds are** (fractions, or whole
people rounded down or to the nearest), **A threshold of 0** (acts at once, or once a neighbor does),
**Friends count more** with **Acquaintance**, **A friend counts as** and **Friendship is mutual**,
**Who sees whom** (the whole crowd, a random network, a network with hubs) with **Mean degree**,
**Ceilings** (the share who leave, and above what), **Several crowds** with **Crowds** and **Movement
per step**, and **Episodes** (start again at each equilibrium; what counts as global; the longest
episode). The view: the share acting over the last 400 steps (one line per crowd), Granovetter's
Figure 1 (the thresholds' c.d.f. against the 45° line, with the riot's staircase) for a single crowd
seen whole or the histogram of episode sizes otherwise, and the actors as a grid. Color modes:
**State**, **Threshold**, **Degree**, **Crowd**. Charts: Participation (with Granovetter's
continuous equilibrium); Episodes; Last cascade; Swing. Presets: `gr-uniform`, `gr-perturbed`,
`gr-normal-12`, `gr-normal-13`, `gr-normal-sampled`, `gr-city`, `gr-friends`,
`gr-friends-perturbed`, `gr-ceilings`, `gr-clusters`, `watts-lower`, `watts-middle`, `watts-upper`,
`watts-hetero`, `watts-hub`. **Compare** entry: "Uniform vs perturbed crowd — Threshold Models
(Compare)". Built-in sweeps: `gr-sd`, `gr-friends`, `gr-movement`, `gr-ceilings`, `watts-window`,
`watts-hetero`, `watts-targeting`.

Credit: Mark Granovetter, "Threshold Models of Collective Behavior," *American Journal of Sociology*
83(6) (1978), 1420–1443; Duncan J. Watts, "A Simple Model of Global Cascades on Random Networks,"
*PNAS* 99(9) (2002), 5766–5771. See `docs/superpowers/specs/2026-09-27-thresholds-design.md`.

### The Timing of Retirement (Axtell & Epstein 1999)

**The model.** In 1961 Congress let workers claim Social Security at 62 instead of 65, yet it took
nearly three decades for the most common retirement age to follow. Axtell and Epstein's agents live
in 81 one-year cohorts, die at random between 60 and 100, and are replaced by 20-year-olds. A few are
rational and retire as soon as they may; a few retire at random; most imitate, retiring once half the
eligible members of their own small network — people within a few years of their age — have.

Measured (the survey and the presets' descriptions):

- **The realizations reproduce in shape, a little slower.** With 15 % rational, 95 % of those eligible
  have retired by period 8 on average, rising steadily (the text says "within the first 6 periods"; 4
  runs of 20 make it by then); with 5 %, retirement
  stalls, wavers and "percolates up" from the old, finishing near period 61. Larger networks slow the
  transition, a spread of network sizes speeds it, the cohort size does not matter, and retirement
  mandatory at 70 speeds it — all as stated. Wider networks speed it at 10 % rational, as stated, but
  not at 5 % (Figure 6-9), where the narrowest are as fast as the widest.
- **Footnote 5 is false.** Counting every friend instead of the eligible ones is said to leave the
  results' "qualitative character" unchanged; counting every friend, no norm ever forms — the young
  friends hold the share retired below one half.
- **Figure 6-6 needs an unstated rule.** Under the pseudo-code's reading — a dead friend's place
  passes to the newborn in its slot — no minimum of rationality is needed (72 periods with no
  rationals at all) and nothing takes the paper's hundreds of periods. Only if friends who die are
  replaced by someone of about the same age do the paper's "minimum proportions" and long, erratic
  transitions appear (no norm at 0 or 5 % rational; at 10 %, 8 runs of 10 reach it after 22 to 279 periods and 2 never
  do within 600).
- **The policy switch does not reproduce.** Lowering eligibility to 62 once the norm is established,
  the paper's new norm "emerges after twenty to thirty periods"; here it comes in 2, at every share of
  rationals, under either rule: an imitator just turned 62 counts its retired 65-to-67-year-old friends
  and retires at once. The decades the model was built to explain do not follow from its rules.
- **Coupling pulls both ways.** A little coupling between a community without rationals and one with
  them pulls the first into line (75 → 46 periods at 0.1), as the paper says, but slows the second just
  as much (19 → 34), until both take about 58; the paper's figure keeps the rational group fast. A
  little spread in the thresholds first doubles the transition time before more spread shortens it.

Switches: **Agents per cohort**, **The first agents' death ages**, **Each period, agents act** (cohort by
cohort, oldest first, or in one random order), **Rational share**, **Random share**, **Random agents'
chance**, **Imitation threshold**, **Threshold spread**, **Imitators count** (eligible members, or every
member), **Network size**, **Extent**, **When a member dies** (the newborn in its slot takes its place,
or it is replaced within the holder's age range), **Eligibility age**, **Mandatory age**, **Lower the
age once the norm is reached** with **To**, **The norm is reached at** (the paper never defines its
transition time; here, the first period with that share of the eligible retired), **Two
sub-populations** with **Coupling**, and **Stop at the norm**. The view is Axtell and Epstein's: one row
per age from 20 at the top, agents colored by type while working and red once retired; beside it, the
share retiring at each age over the last 10 periods, and the share of the eligible retired over time.
Color modes: **Status**, **Type**, **Threshold**, **Group**. Charts: Retired share (by group with two
sub-populations); Retirement age; Transition; Group transitions. Presets: `ae-rapid`, `ae-base`,
`ae-slow`, `ae-policy`, `ae-groups`, `ae-all-members`, `ae-replace`. **Compare** entry: "15 % vs 5 %
rational — Retirement (Compare)". Built-in sweeps: `ae-rational`, `ae-rational-replace`,
`ae-threshold`, `ae-size`, `ae-extent`, `ae-policy`, `ae-coupling`, `ae-coupling-rational`.

Credit: Robert L. Axtell and Joshua M. Epstein, "Coordination in Transient Social Networks: An
Agent-Based Computational Model of the Timing of Retirement," Brookings CSED Working Paper No. 1 (1999),
in H. Aaron, ed., *Behavioral Dimensions of Retirement Economics* (1999); Joshua M. Epstein, *Generative
Social Science* (Princeton, 2006), chapter 7. See `docs/superpowers/specs/2026-09-27-retirement-design.md`.

### Altruistic Punishment (Boyd, Gintis, Bowles & Richerson 2003)

**The model.** People punish free riders even when it costs them and brings them nothing, and group
selection was thought to sustain costly cooperation only in small groups. Boyd, Gintis, Bowles and
Richerson's answer: punishment is cheap once defectors are rare. 128 groups of contributors, defectors
and punishers play a one-shot game (cooperating costs c = 0.2; punishers fine each defector p/n = 0.8/n at
a cost of 0.2/n); everyone copies someone who earns more, sometimes from another group; groups fight,
the one with fewer defectors more likely to win and replace the loser; a few agents mutate. Their
figures plot cooperation, averaged over the last 1 000 of 2 000 periods, against group size.

**How this reproduction handles the paper's gaps and contradictions.** Every reading is a named
switch, every figure was read from the PDF at 300 dpi by marker, and every "reproduces the figure"
claim uses one rule, fixed in advance: a mean gap of at most 0.05 over group sizes 4–256, reported with
each curve's worst point (a mean over curves that sit mostly at the 0.09 floor is lenient).
- **The payoff baseline is never stated.** Imitation needs payoffs above 0; a baseline of 1 fits the
  paper's own calibration (a trait with advantage c spreads from 10 % to 90 % in about 40 periods;
  "50" stated).
- **The conflict rate contradicts itself.** Fig. 1's caption gives 0.075, 0.015, 0.003; its legend 0.0075,
  0.015, 0.03. Under the text's rules neither reproduces the figure: cooperation collapses a group size
  or two too soon (0.17 at n 128 where the figure has 0.64). No baseline fixes it — a higher one lets
  punishment reach larger groups but lifts cooperation without punishment far above the figure.
- **The figures fit about twice the stated conflict rate.** Of Figs. 1–4's 14 curves, the stated model
  reproduces 2 (worst points up to 0.47 off). With pairs fighting at 2ε, all 14 reproduce (mean gaps
  0.006–0.049, worst points at most 0.16) — including the eight curves of Figs. 2–4, which were not used
  to find the factor. Their Methods derive ε = 0.015 from an extinction rate of 0.0075 because "only one
  of the two groups entering into a conflict becomes extinct", so the text says pairs fight at ε; the
  figures look like groups fighting at about ε each — as if their code let either group of a pair start
  the conflict. That is a switch here, **Groups meet: in random pairs; either can start it (the
  figures)** (preset `bg-either`: a pair fights at 2ε − ε²), and it reproduces 13 of 14 (Fig. 4's fixed
  cost at 0.051, where 2ε has 0.049 — noise at the threshold). Janssen's "each group challenges one" is a
  third way to double it; the data cannot tell these apart, and doubling the victory slope, another way
  to strengthen group selection, is untested.
- **Janssen's NetLogo replication** (CoMSES 2223) fills the gaps differently — a benefit, every group
  challenging one, conflict over this period's acts, imitation in turn; his readings are switches here
  (not his code). Together they come close too (84 % at n 32) because his pairing also doubles conflict,
  but his benefit lifts cooperation without punishment above the figure (0.42 at n 16 against 0.20).

Measured (the survey and the presets' descriptions; the text's readings unless stated):

- **The figures' shapes hold.** Without punishment, cooperation survives only in groups of 4 or 8
  (Fig. 1a); punishment sustains more at every size (Fig. 1b); more conflict, more cooperation; more
  mixing, less (Fig. 2); a fine only twice the cost gives much less (Fig. 3); a fixed punishing cost gives
  nothing from n 32 (Fig. 4). Lower mutation raises cooperation substantially, more errors lower it, and
  where the population starts does not matter — all as stated.
- **The reach does not, under the text's reading:** "cooperation is sustained in groups on the order of
  100 individuals" — 17 % at n 128. Under the "either" reading, 59 %.
- **The mixing calibration is off.** m = 0.01 is said to equalize two groups in about 50 periods; after
  50, 58 % of the difference remains. A member meets the other group with probability m and copies it
  half the time, so the gap shrinks by about m a period.
- **Fewer groups add more than noise:** 0.56 at 8 groups against 0.69 at 128.
- **Continuous traits are not similar.** Cooperation rises with group size (94 % at n 32, 90 % at 256,
  against the base model's 69 % and 12 %): uniform mutants keep the mean punishment near ½, and a
  defector then pays about p/2 = 0.4, more than c.
- **The ring is not cooperation-free**: about half cooperate in groups of 4 or 8, though little from 32.
- **The per-capita benefit with payoff conflict** is qualitatively similar, as stated.
- **Cooney's PDE claims (2024):** a shallow dip in payoff at weak punishment appears — but under every
  victory rule, not only the normalized one his Remark 6.1 blames; a higher cost of punishing never
  raises the share of punishers here.

Switches: **Groups (N)**, **Group size (n)**, **At the start**, **Cost of cooperating (c)**, **Cost of being
punished (p)**, **Punishers pay** (k/n per defector, or a fixed cost), **Cost of punishing (k)**, **Fixed
cost**, **Errors (e)**, **A punisher who errs** (punishes the others, nobody, or itself too), **Benefit to
others (b)**, **Baseline payoff**, **Mixing (m)**, **Imitation happens** (all at once, or in turn),
**Mutation (μ)**, **Conflict (ε)**, **Groups meet** (in random pairs; either can start it; each challenges
one), **Groups fight over** (defectors, payoffs normalized, payoffs through tanh) with **Sensitivity**,
**Defectors are counted by** (type, or this period's acts), **A defeated group** (becomes a copy of the
winners, or is refilled with them from the winners), **Traits** (discrete or continuous), **Groups are**
(anywhere with conflict, or on a ring without), **Long-run window** and **Stop at period**. The view: every
group a block of its agents, contributors blue, punishers green, defectors red, a group that just lost
framed; below, cooperation and punishment over time. Color modes: **Type**, **Acts**, **Payoff**,
**Group**. Charts: Types; Cooperation (with the long-run average); Payoff; Conflict. Presets: `bg-base`,
`bg-either`, `bg-none`, `bg-large`, `bg-weak`, `bg-fixed`, `bg-mixing`, `bg-benefit`, `bg-continuous`,
`bg-ring`, `bg-janssen`. **Compare** entry: "With vs without punishment — Altruistic Punishment
(Compare)". Built-in sweeps: `bg-fig1a`, `bg-fig1b`, `bg-fig1-caption`, `bg-fig1-either`, `bg-fig2a`,
`bg-fig2b`, `bg-fig3`, `bg-fig4`, `bg-baseline`, `bg-readings`, `bg-mutation`, `bg-error`, `bg-groups`,
`bg-benefit`, `bg-continuous`, `bg-ring`, `bg-cooney-fine`, `bg-cooney-cost`.

Credit: Robert Boyd, Herbert Gintis, Samuel Bowles and Peter J. Richerson, "The evolution of altruistic
punishment," *PNAS* 100(6): 3531–3535 (2003); Daniel B. Cooney, "Exploring the Evolution of Altruistic
Punishment with a PDE Model of Cultural Multilevel Selection," arXiv:2405.18419 (2024; *Bull. Math.
Biol.* 2025); Marco Janssen's replication, CoMSES Net 2223 (GPL-3.0, read for its readings only). See
`docs/superpowers/specs/2026-09-28-punishment-design.md`.

### Zero-Intelligence Traders (Gode & Sunder 1993; Cliff 1997)

**The model.** Are markets efficient because traders are smart? Gode and Sunder replaced human traders
in a double auction with programs that shout random prices. Unconstrained (ZI-U), they trade at a loss
and waste surplus; merely forbidden to trade at a loss (ZI-C), they capture almost all of it — "the
market as a partial substitute for individual rationality." Six buyers and six sellers trade units
one at a time; a shout that crosses the standing bid or ask trades at the earlier order's price, and
each trade clears the book. Cliff (1997) argued that ZI-C prices converge on equilibrium only when
supply and demand are symmetric, and built traders that learn a profit margin (ZIP).

**How the sources were read.** Gode and Sunder's paper is a scan (read by OCR); their five markets
exist only as step curves in the figures. Because ZI-U traders trade every unit, a market's ZI-U
efficiency follows arithmetically from its schedules — so Table 2's ZI-U numbers pin markets 1–4 down
exactly (90.0, 90.0, 76.7, 48.8; market 4's second cost reads 141 by pixel, and 142 gives 48.8),
alongside the text's P₀ of 69 and 170 and volumes of 24 and 6 — a calibration, not a finding;
market 5, a fine staircase, is read as well as the scan allows (86.7 against 86.0). Cliff's results
come from the C code in his appendices, which differs from his text in places; the code is the
default, the text a switch.

Measured (the survey and the presets' descriptions):

- **Gode and Sunder reproduce, given enough time.** ZI-C efficiency 99.9, 99.8, 99.7, 99.5, 97.1 in
  markets 1–5 (their 99.9, 99.2, 99.0, 98.2, 97.1); profit dispersion close to Table
  3; prices tightening within each period (Table 1's negative slopes). The rank correlation of the
  trading order with the efficient one is higher for ZI-C than ZI-U, as their footnote says, though
  higher than theirs (0.91 and 0.85 against 0.74 and 0.42).
- **But "30 seconds" decides it.** Their periods lasted 30 seconds, never translated into shouts. At 100
  shouts a period ZI-C efficiency is 44–86 %; it needs about 500–1 000 to reach their numbers. The
  random traders capture the surplus because they get enough chances.
- **Cliff's critique holds in direction, not in number.** In his simulator (a random willing trader at
  the shout's price, days of up to 11 sessions as his code runs them) ZI-C mean prices are 199.1,
  233.7, 137.0, 249.7 in his four markets (P₀ 200): his predictions hold for the symmetric and flat
  markets but miss the box markets by 12 and 10, and his printed 233⅓ matches his simulation, not his
  own formula (241⅔). In Gode and Sunder's own mechanism the prices sit
  about half as far from P₀ (216.6, 161.8, 232.9) — still off, so the critique's direction survives.
- **ZIP learns its way to equilibrium.** Daily mean prices converge on P₀ in all four markets — the
  flat one within 4 days, the excess-demand box from below and steadily — and after a demand or supply
  shift; profit dispersion falls to a tenth of ZI-C's or less; efficiency averages 99.9–100 %. In
  Smith's retail market, where only sellers
  post prices, trades stay below P₀ as Cliff says, but rise past $2.00 by day 9.
- **Cliff's text is not his code.** His text draws ZIP's momentum from U[0.2, 0.8]; his code overwrites
  it with U[0, 0.1] — the text's reading converges a day or two sooner in the box markets. His text
  ends a day after 100 failed shouts; his code ends only a *session* there, a day running up to a set
  number of sessions (11 in his ZI-C runs, 9 in his ZIP control file). The code is the default
  (**A period ends: after its sessions**); the text's rule cuts ZIP's efficiency to 98.9 % in the
  symmetric market.

Switches: **Market** (Gode and Sunder's 1–5, Cliff's symmetric, flat supply, excess demand, excess supply
and retail, or custom), **Highest price**, **Traders** (ZI-C, ZI-U, ZIP) with **ZIP momentum** (his code or
his text), **Trades happen** (against the standing quote, or with a random willing trader) with **NYSE
rules**, **Who shouts** (a random trader, or a side then a trader), **Only sellers shout**, **A period
ends** (after a number of shouts; after its sessions, as Cliff's code; or after 100 failures in a row,
as his text) with **Shouts a period** and **Sessions a day**, **Shift**
(demand up or supply down 50) with **Shift from period**, and **Stop after period**. The view follows Gode
and Sunder's figures: the schedules, the trade prices across periods, and a strip of traders with
their profits against their equilibrium profits. Color modes: **Side**, **Profit**, **Margin**. Charts:
Prices; Efficiency; Convergence (Smith's α); Profit dispersion; Volume. Presets: `gs-1`–`gs-5`, `gs-1-u`,
`gs-4-u`, `cliff-symmetric`, `cliff-flat`, `cliff-excess-demand`, `cliff-excess-supply`, `zip-symmetric`,
`zip-flat`, `zip-excess-demand`, `zip-excess-supply`, `zip-demand-shift`, `zip-supply-shift`, `zip-retail`.
**Compare** entries: "With vs without the budget constraint — Zero-Intelligence Traders (Compare)" and
"ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)". Built-in sweeps: `gs-efficiency`,
`gs-dispersion`, `gs-shouts`, `gs-mechanism`, `cliff-prices`, `zip-days`, `zip-momentum`, `zip-shift`.

Credit: Dhananjay K. Gode and Shyam Sunder, "Allocative Efficiency of Markets with Zero-Intelligence
Traders: Market as a Partial Substitute for Individual Rationality," *Journal of Political Economy*
101(1): 119–137 (1993); Dave Cliff, "Minimal-Intelligence Agents for Bargaining Behaviours in
Market-Based Environments," HP Laboratories HPL-97-91 (1997). See
`docs/superpowers/specs/2026-09-28-zi-traders-design.md`.

### Balinese Water Temples (Lansing & Kremer 1993; Janssen 2007)

**The model.** On the Oos and Petanu rivers of Bali, 172 subaks (farmers' associations) take water from
12 weirs and share pests with their neighbors. Planting at the same time as your neighbors lets a
shared fallow starve the pests; planting at the same time as everyone upstream leaves too little water.
Lansing and Kremer's simulation let each subak copy the planting plan of its best-harvesting neighbor
once a year, and found that within 8 to 35 years yields rose and the subaks fell into patches that
"bore a remarkable similarity" to the congregations of the water temples — the temples, they argued,
solve the trade-off. Janssen (2007) reimplemented the model and asked how much coordination is worth,
at what scale, and whether other decision rules do as well. One tick is a month.

**How the sources were read.** Lansing and Kremer's paper is a scan with the rules in prose; Janssen's
paper gives the equations and his later NetLogo release (CoMSES 2221) the watershed: the subaks' areas,
temples, dams and pest links, the dams' flows, catchments and rain zones, the 21 plans, the rain tables
and the crops' constants. Those data files are GPL-2.0 and ship beside the MIT code in `data/bali/`
(see its `NOTICE`); the code was written from the published descriptions. Janssen's code departs from
the texts in two places, each a switch: each month it balances the water of one random dam, with no
inflow from upstream (**Water flows**), and it reads the subak–dam file's columns as (return, source)
although the first is the upstream dam in 93 of 95 cases (**Dam columns**). His code also resets pests
each year ("If we don't … the system gets locked into low harvest rates"): the default, and a switch.
Our readings where the texts are silent or cannot be followed: level 2 is the two rivers (Janssen's
highlands and lowlands are not identifiable in the data); level 7 is adjacent pairs of mascetis; level 28
is the mascetis split by the data's second temple column, which gives only 22 groups; the high-yielding
runs use two rice crops without Lansing and Kremer's vegetable crop, which Janssen's 21 plans drop;
adaptive subaks' water threshold is m/day per hectare the source dam serves (his Fig. 12's axis, 0–500
m³/day per hectare, agrees), their neighborhood is the subak and the subaks whose pests reach it, and they
plant three-month rice; under eq. 4 a subak innovates only when no one qualifies to be copied; the plan
search scores year 2 of a two-year run, on the same random draws for every option, climbing from the
better of random plans and the best single plan for everyone (Janssen used several starting points), and
under random rain it plans for middle rain; the perturbation's magnitudes are ours.

Measured (the survey — 9 claims hold, 3 are weak, 11 fail — and the presets' descriptions):

- **Imitation works, as Lansing and Kremer say.** From random plans the harvest rises from 10.9 to 20.4
  t/ha/yr, nearly all of it in eight years; Table 1's three rises reproduce within 10 % (traditional
  rice 5.0 → 8.1, their 4.9 → 8.57; high-yielding 16.9 → 18.2, their 15.91 → 18.08; low rain and high
  pests 12.9 → 16.5, their 13.67 → 17.66). It holds "every time": at every pest growth and dispersal,
  rain and start we tried, imitating subaks reap 1.6 to 2.1 times what the same plans fixed reap.
- **But the resemblance to the temples is mostly the pest network's.** The mapped pest links fall into 46
  groups (27 of them single subaks); those groups alone match the 14 masceti congregations with an
  adjusted Rand index of 0.33. Imitation ends with mostly one plan per group, and its patches match the
  temples at 0.37 on average — slightly higher, but not consistently (above the network on 6 of 10
  seeds), and short of the 0.05 margin we set after planning had measured it. The subaks also settle
  harder than the paper says: 7 are still changing in year 8 and 2 by year 30, not 20.
- **The perturbation does not recover — from the paper's start.** Pests and drought from year 21 (Fig.
  11) cut the harvest of the high-yielding run from 18.1 to 16.5, and it stays there: every subak began
  with the same plan, so imitation has nothing different to copy. From random plans the same
  perturbation recovers within seven years on 3 of 10 seeds. The paper's "twice as long" from the start
  does not appear either.
- **The scale of coordination hardly matters — how little depends on a reading of the data.** Janssen's
  search finds finer coordination always helps, but at middle rain by only 0.7 % from one group to 172
  under his code's reading of the subak–dam columns and 3.7 % under the physical reading; the temple
  scale is never best. Under the code's reading water rarely binds — summed over the watershed the dams'
  base flow roughly meets the demand of every subak planting at once, the network routing passes surplus
  down to the intakes short of it, and growing months lose at most 2 % of their water at any rain.
  Under the physical reading water binds at low rain (8 % lost) and the rise from one group to 172
  reaches 7 %. Janssen's Fig. 1 rise (≈ 17.5 to 22.8, +30 %) appears under neither reading, and neither
  do his rising inequality, his benefit of coordination at g 2.2 alone or his losses at high dispersal.
- **Janssen's two nodes and his other rules reproduce in part.** The two-node threshold at ∛10 ≈ 2.14
  holds exactly (the best harvest falls between g 2.1 and 2.2); imitation discounted by distance (eq. 4)
  does best with γp below 0.5 and beats neighbor imitation (25.4 against 20.4). Adaptive subaks never
  plant at very low pest tolerance and plant too early at high tolerance, as his Fig. 12 says; within
  his plotted water thresholds the harvest is nearly flat (26.0, 26.2, 25.3, 26.0), and his best pair
  (0.05, 0.02) is 7 % below (0.05, 0.05). Adaptive subaks lose harvest when pest links are added and not
  when removed, as he found; but his eq.-4 imitators, which should lose when links are removed and not
  mind added ones, lose about 4 % when half are removed (Mann–Whitney p = 0.08) and 6 % when links are
  added (p = 0.007).
- **His code's departures barely matter for imitation.** The one-random-dam routing, the swapped columns
  and the diffusion form of the pest equation each move the imitation endpoint by 1–3 %; the columns
  matter more for the searched levels (above). The pest reset is essential (without it, 4.1 against
  20.1).

Switches: **Watershed** (the Oos and Petanu, or Janssen's two nodes with **Rain units a month** and
**Periods a year**), **Starting plans** (random, traditional, high-yielding, one per temple, or Janssen's
search) with **Groups sharing a plan**, **Each year, subaks** (copy their best neighbor; copy by eq. 4
with **γp**, **γw** and **Innovation (ρ)**; plant adaptively with **Water to plant** and **Pests to plant
under**; or keep their plans), **Pest growth (g)**, **Pest dispersal (d)**, **Pest equation** (Lansing
and Kremer's shortcut or the diffusion form), **Pests reset each year**, **Pests and drought strike
(Fig. 11)** with **From year**, **Rain**, **Rain ×**, **Water flows**, **Dam columns**, **Remove pest
links (pₑ)**, **Add pest links (pₙ)**, **Score from year** and **Stop after year**. The view is the
watershed: subaks as discs sized by area, dams as squares, rivers and pest links as lines, and below it a
strip of each dam's water over the last twelve months. Color modes: **Plan**, **Temple**, **Harvest**,
**Pests**, **Water**, **Crop**. Charts: Harvest; Changing plans; Water and pests; Patches; Temple match.
Presets: `lk-random`, `lk-random-fixed`, `lk-traditional`, `lk-hyv`, `lk-perturbed`, `lk-stressed`,
`lk-temples`, `janssen-code`, `janssen-levels-14`, `janssen-two-node`, `janssen-generalized`,
`janssen-adaptive`, `janssen-fewer-links`. **Compare** entry: "Imitating neighbors vs fixed random
plans — Balinese Water Temples (Compare)". Built-in sweeps: `bali-levels`, `bali-growth`,
`bali-dispersal`, `bali-rain`, `bali-imitation-growth`, `bali-two-node`, `bali-gamma`, `bali-adaptive`,
`bali-links`.

Credit: J. Stephen Lansing and James N. Kremer, "Emergent Properties of Balinese Water Temples,"
*American Anthropologist* 95(1): 97–114 (1993); Marco A. Janssen, "Coordination in Irrigation Systems:
An Analysis of the Lansing–Kremer Model of Bali," *Agricultural Systems* 93: 170–190 (2007); the
watershed data from Janssen's "Lansing–Kremer model" (CoMSES Net 2221, v1.2.0, GPL-2.0). See
`docs/superpowers/specs/2026-09-28-bali-water-temples-design.md`.

### The Emergence of Firms (Axtell 1999; Axtell 2013)

**The model.** Why do firms exist, and why are their sizes spread like a power law? Axtell's agents
each prefer income and leisure in their own proportion (θ, uniform on [0, 1]). A firm's output grows
faster than its members' total effort (a·E + b·E², increasing returns), and is shared equally — so
working together pays, but each member's share is only weakly tied to its own effort, and free
riders creep in. Each period every agent, activated at random, weighs re-choosing its effort at
home, starting a firm alone, and joining one of its two friends' firms, and takes the best. Firms
form, grow, fill with free riders and collapse. One tick is a period.

**How the paper was read.** It is a 108-page working paper with the rules in prose, a text layer, and
no code. Its §2 analytics — each agent's best effort (a closed form), the Nash equilibria of a group
of like agents, the size beyond which that equilibrium turns unstable (Table 1) — are exact, and
Table 1 reproduces to three decimals; his other §2 claim, optimal group sizes "less than 10" for
θ < 0.85, misses by a hair (θ = 0.84 already optimizes at 11). Its simulation leaves much unstated,
and each gap is a switch: what an agent sees of others' effort (last period's, inferred from output,
as the text says; or live), how agents are activated (at random with replacement, or once each), how
finely effort is searched, and for §4: whether sticky effort and groping apply in a firm an agent
joins or founds, which way seniority pay runs, and who covers a shortfall when base pay exceeds
output. Its own figures, tables and text disagree in places — firm counts, mean sizes and lifetimes
that can't all be true of 1 000 agents; output levels half of what the all-alone start already gives;
four rows of Table 11 that are one model (so their spread, 0.89–1.07, is the paper's noise); text
that says a table rises where it falls; and a table captioned for a rule ("target output") that
appears nowhere. Axtell's 2013 paper recasts the model (per-firm a, b and β; 2–6 friends; 4 % of
agents activated a period, a month) and is a preset.

Measured (the survey — 25 claims, 15 hold and 10 fail — and the presets' and sweeps' descriptions):

- **The base case does not reproduce the paper's numbers, under any reading.** Firm sizes fall off
  with µ ≈ 2.5 by Axtell's own OLS (1.4 by maximum likelihood), not 1.28; firms live about 4
  periods, not 23.4; the largest firm reaches 62–150 over the survey's 10 seeds (his typical run: about 205); output runs
  about 750 — below the all-alone start's 934, but still above his 450–600; productivity shows
  decreasing returns (output ∝ size^0.67), not his near-constant ones (s^1.15). Reading others'
  effort live and activating each agent once comes closest (µ 1.79). Population size and the start
  don't matter, as he says.
- **The growth of firms passes, but on thin grounds.** Growth rates fit a Laplace better than a
  Gaussian in 10 of 10 seeds, as his figure 20 says — but 38–41 % of them are exactly zero (firms
  that kept their size, singletons included), and that spike is what the Laplace fits: on the
  nonzero rates alone the Gaussian wins in 10 of 10 (mean log-likelihood −1.16 against −1.32).
  Their spread falls with size as s^−γ with median γ 0.171 over 10 seeds — his is 0.174 ± 0.004 —
  but the seeds run from 0.12 to 0.24.
- **Most of §4's directions hold, two reverse.** µ falls as increasing returns (β, b) strengthen,
  as random-firm networks widen, as loyalty grows and as base pay rises, and as hiring standards
  first steady large firms and then undo them, all as his tables say — roughly 1 to 2 above his
  values (random-firm networks widest, about 1.8–2.0 above: 3.29 against 1.28, 2.81 against 1.03;
  the rest nearer 1, though b = 1.5 and β = 1.7 run further out still, 2.09 against 0.53 and 3.63
  against 2.06). His "breaks down" at full hiring standards does not: the largest firm stays at
  20–52, not under 20.
  More fixed friends *raise* µ, where his Table 6 lowers it, and seniority pay raises µ far more
  sharply (2.49 to 10.03) where his Table 11 only slightly lowers it (1.28 to 1.11). Loyalty is the
  one change that gives his long lifetimes (about 35 periods at λ = 10).
- **Three unstated details decide whole tables.** Sticky effort (±0.05 a step) and groping, applied
  to effort in any firm, pull the whole population into one firm at times, in every seed; the
  survey counts that a Hold only because µ turns negative there — a one-giant-firm regime, not a
  power law — while applied only at home, they barely matter. The text calls groping's effect "more
  pronounced," but the paper's own tables say sticky effort's is. Seniority pay with the founder
  paid most (the text's order) keeps every firm to four or fewer (a joiner's share of a pair's
  output under 5^−rank is 1/6); with the newest paid most, near-giant firms form. His modest Table 11 effect
  comes from neither. Base pay, paid even when a firm's output falls short (eq. 21,
  our literal default), has firms-base-pay-80 paying out about 2.85× what it makes; scaling pay down
  to output instead, so agents expect it, lifts effort from 0.10 to about 0.24 and output from 290
  to about 674, in smaller firms.
- **Random choices of firm hold; random effort's "fail" is a borrowed threshold, not a
  contradiction.** Random choices keep every firm small, as he reports. Random effort instead
  pulls all 1 000 agents into one firm in every seed; his own words, "nothing like power law size
  distributions," are satisfied by that too, so the survey's Fail there comes only from reusing
  random choices' "rarely above 9 or 10" threshold, which he states only for random choices.
- **Myopic agents join even without increasing returns.** His footnote 19 says working together
  never pays under constant returns — true at equilibrium; an agent taking others' effort as given
  still gains by joining an identical one, and firms form.
- **His 2013 parameterization gives Zipf's law, and one giant firm.** µ ≈ 1.0 (his 2013 α ≈ 1.06),
  but the largest firm peaks at 3 000–5 800 of the 10 000 agents after the burn-in (5 seeds), though most of the time it stays under about 1 000 (median 780–955). Firms live about 77 periods, but a 2013 period activates only
  4 % of agents: that is about 3.1 activations per agent, against the 1999 base case's 3.9 periods
  at one activation per agent a period on average (about 3.9) — so 2013's firms are not
  longer-lived per decision. The famous size distribution belongs to the later model, not the 1999
  one.

Switches: **Agents** and **Start**; **Constant returns a**, **Increasing returns b**, **Exponent β**
(each fixed, or drawn per firm up to a maximum); **Preferences** (nine distributions, including CES
with **CES exponent** convention); **Agents look at** (friends' firms or random firms) and **Friends
or firms ν**; **Activation**, **Activations a period ÷ agents**, **Others' effort**, **Best effort**,
**Effort changes by at most**, **Grope for effort**, **Sticky effort and groping apply**, **Loyalty
λ**, **Random behavior**; **Output is shared** (equally, by seniority with **The largest share goes
to**, or base pay plus a bonus with **When output falls short of base pay**), **Hiring standard φ**;
and the measurement's burn-in, sampling and stop. The view follows Axtell's Animation 1 — each firm a row, its longest-serving member first, the
largest firms first — beside the firm-size distribution on log-log axes with its OLS line. Color modes: **Founder**, **θ
(income)**, **Effort**, **Income**. Charts: Firms; Sizes; Effort and pay; Output; Scaling. Presets:
`firms-base`, `firms-live`, `firms-uniform`, `firms-beta-17`, `firms-beta-21`, `firms-b-15`,
`firms-b-random`, `firms-theta-075`, `firms-friends-10`, `firms-random-firms-10`, `firms-loyal-10`,
`firms-sticky`, `firms-groping`, `firms-seniority-5`, `firms-base-pay-80`, `firms-hiring-100`,
`firms-random-choices`, `firms-2013`. **Compare** entry: "Last period's effort vs live effort — The
Emergence of Firms (Compare)". Built-in sweeps: `firms-beta`, `firms-b`, `firms-preferences`,
`firms-friends`, `firms-random-firms`, `firms-loyalty`, `firms-sticky`, `firms-groping`,
`firms-seniority`, `firms-base-pay`, `firms-hiring`, `firms-readings`, `firms-population`.

Credit: Robert L. Axtell, "The Emergence of Firms in a Population of Agents: Local Increasing
Returns, Unstable Nash Equilibria, and Power Law Size Distributions," Brookings Institution CSED
Working Paper 3 (1999); Robert L. Axtell, "Endogenous Dynamics of Firms and Labor with Large Numbers
of Simple Agents" (working paper, 2013). See
`docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md`.

### Algorithmic Collusion (Calvano, Calzolari, Denicolò & Pastorello 2020; and its critics)

**The model.** Can pricing algorithms learn to collude without being told to and without talking?
Two firms sell differentiated goods (logit demand) and each picks one of 15 prices a period, a grid
from a little below the competitive (Bertrand–Nash) price to a little above the joint-monopoly
price. Each firm learns by Q-learning on the state "both firms' prices last period": it keeps a
value for every price in every state, updates the value of the price it charged toward the profit
it earned plus the discounted value of the best price next period, and picks the best price —
except, with a probability that falls over time (ε = e^(−βt)), a price at random. A session ends
when neither firm's strategy has changed for 100 000 periods. The paper reports that the firms
settle far above the competitive price (profit gain Δ = 0.849 of the way from the Nash profit to
the monopoly profit) and punish a rival's price cut, then return: a reward–punishment scheme
learned from scratch. A tick is 1 000 periods; a session takes about 1 800 ticks.

**How the paper was read, and its code.** The paper comes with its authors' Fortran (MIT). Built
here with gfortran, it reproduces Table I; and under the code's own readings — ties broken at
random, its RAN2 generator seeded by session number (the seed), its cap, its tests — this model
reproduces the code's sessions exactly: the same strategies in the same period, 100 of 100. Where
the paper's text and its code differ, the text is the default and the code's choice a switch
(`collusion-code` turns them all on): ties to the lowest price; a cap of 10⁹ periods, not
1.25 × 10⁹; an equilibrium is a best response against the rival's strategy (the paper's
description) rather than "no one-period deviation pays" (the code's test); and the deviation in
Fig. 4 answers the rival's price at the state itself — the code's impulse-response routine passes
the cycle position where it means the state. The critics' tests are switches too: no memory
(Lambin), no future (δ = 0), synchronous updates of every price (Asker, Fershtman & Pakes),
exploration that decays more slowly, stays constant, or stops after a random phase (Lambin), a grid
below the Nash price and deviations up as well as down (Epivent & Lambin), RP-completeness and the
first periods' discounted gain (den Boer, Meylahn & Schinkel), and firms re-paired across sessions
(Eschenbaum, Mellgren & Zahn).

Measured (the survey — 20 claims, 15 hold and 5 fail — and the presets' and sweeps' descriptions):

- **The paper reproduces, exactly.** Under the code's readings, sessions 1–1 000 are the authors'
  own: Table I to the digit (Δ 0.849, 50.5 % in equilibrium on path; one-price cycles 64.3 %,
  two 23.8 %, longer 11.9 %) and Table A5's responses to a deviation (the rival's price −12.7 %,
  deviations unprofitable 93.6 %, punishment 5.7 periods). The paper's own readings give the same
  Δ (0.851). The α × β heat map (mean gap 0.008) and Δ against δ (minimum near δ = 0.34) match.
- **The text overstates its tables.** "In more than 95 % of the cases the punishment makes the
  deviation unprofitable" is 93.6 % in its own Table A5. And the equilibrium the paper describes —
  each firm's strategy a best response to the other's — holds on the path in 0.2 % of sessions;
  the code's test, which checks only one-period deviations, passes 49.7 %. A firm that re-optimizes
  against its rival's learned strategy gains 5–41 % of its value: the strategies punish a single
  deviation but can be exploited by longer ones.
- **High prices without strategies.** Firms with no memory, which cannot punish anything, price
  higher still (Δ 0.958 against 0.851; Lambin's test holds — the authors' code cannot run it, as
  it sets δ = 0 whenever memory is 0). Firms that value only the present (δ = 0) reach Δ 0.212, as
  the paper's own Fig. 3 shows: a quarter of the baseline needs no future at all.
- **The "punishment" is not specific to cuts.** A rival's price *increase* is answered by a price
  cut too, in all 25 cells Epivent & Lambin's Table 1 covers (9.9 % against 13.0 % after a cut); an
  invitation to raise prices is met by a cut in 84 % of sessions; in 91 % of the sessions that
  punish a cut, some other deviation goes unpunished (den Boer et al.'s RP-completeness).
- **It does not transfer, and it does not start early.** A firm paired with a rival trained in
  another session earns Δ 0.125, not 0.851. Over the first 165 periods — the horizon that matters
  at δ = 0.95 — the algorithms earn exactly what uniform random pricing earns (Δ̃ 0.498 against
  0.497).
- **How firms learn matters most.** Updating every price toward what it would have earned
  (synchronous learning) halves Δ (0.345). Exploration decaying ten times more slowly barely lowers
  it (0.727); a constant ε = 0.05 never settles.
- **Lambin's Theorem 1 does not predict the simulations.** After a random phase and none after,
  the theorem says the firms settle at 1.6990 (δ = 0) or 1.7377 (δ = 0.95); at the paper's α they
  do in 27 %, 13 %, 0.4 % and 3.6 % of sessions across memory and δ — the theorem is a mean-field
  limit and the Q-values stay noisy.

Presets: `collusion-calvano`, `collusion-code`, `collusion-no-memory`, `collusion-myopic`,
`collusion-two-phase`, `collusion-synchronous`, `collusion-explore-more`, `collusion-every-price`,
`collusion-invitation`, `collusion-below-nash`. Sweeps: `collusion-table-i`,
`collusion-alpha-beta`, `collusion-delta`, `collusion-memory`, `collusion-myopic`,
`collusion-two-phase`, `collusion-every-price`, `collusion-below-nash`, `collusion-invitation`,
`collusion-synchronous`, `collusion-exploration`, `collusion-timescale`, `collusion-rp-complete`.
Compare: **Learning from the price charged vs every price**. The view: each firm's strategy as a
map (its own last price across, the rival's down, the price it would charge as the color — or how
often each state was visited), the last 240 periods' prices between the Nash (green) and monopoly
(red) lines, and, once the session has finished, the response to a deviation. Inspect a map cell
for both firms' Q-values there, and any cell for the session's results.
See `docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md`.

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
    sugarscape shot beat.json --out beat.frames.json    # a frame dump for the Flump studio

`run` runs a preset or config of any model; it defaults to seed 1 and 1000 ticks (an anasazi run
stops at its end year and says so on stderr); `--config-out` writes the config it ran and
`--fingerprint` prints the final world's fingerprint. `sweep` uses every core unless `--jobs`
says otherwise, prints the result JSON unless `--out` is given, and reports progress on
stderr unless `--quiet`. Exit codes: 0 success, 1 I/O error, 2 usage or validation error
(printed as `field: message`, one per line).

`shot` runs a shot file — `preset` or `config`, `seed` (default 1), `ticks`, `set` (config
paths to override, e.g. `"goods.0.map"`), `empty` (start every site with no sugar) and `place`
(agents placed by hand at a tick with a given vision, metabolism and sugar) — and writes every
tick of the run as JSON: each agent (`[id, x, y, sugar, age, vision, metabolism]`), the sugar at
every site, deaths with their cause, births, the placed agents' ids and the statistics series.
Frame t is the world after t ticks and after that tick's placements. Sugarscape only; see
`studio/README.md`.

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

The A* test fixtures (`crates/sugarscape-core/tests/fixtures/movingai/`) are a subset of Nathan
Sturtevant's Moving AI grid benchmarks (movingai.com/benchmarks), under the Open Data Commons
Attribution License; the README there gives the source and what was kept. They are used only by
the tests.


### Q-learning Auctions (Banchio & Skrzypacz 2022)

Two or three bidders learn bids on a discrete grid. Each bidder values the item at 1;
first price charges the winning bid, second price charges the highest competing bid,
and a mixture interpolates between them. The source claims low first-price bids,
high second-price bids, and higher seller revenue when rival bids support updating
every action. This reconstruction exposes those claims as experiments with fixed
rules; it does not assume their conclusions.

`auctions-first-price` uses the paper's 19 bids (.05–.95), learning rate .05,
discount .99, and exploration `.025 * exp(-.0002 * t)` for one million auctions.
One tick runs 1,000 auctions. A session always completes its horizon; its final
1,000 greedy policies determine stability. Terminal survey comparisons select
stable sessions and also report all sessions and discarded seeds. The default
numerical optimistic initialization (100), sampled auction ties, and lowest-bid greedy-tie
choice are explicit reconstruction choices: the paper does not specify them.
No original simulator was located in the checked public author sources.

The Bids canvas shows one session's **played-pair occupancy**, with a final-20%
view. It is a different statistic from Figure 1's ensemble of **terminal greedy
policies**. Three-bidder occupancy projects onto the first two bidders. Values
and Inspect show each Q vector, chosen/update counts, competing-bid hypothetical
rewards, payment, stability, and completed periods. Tick-mean played bids and
rewards are distinguished from end-of-tick greedy bids in the charts.

Feedback and learning are separate switches: `auctions-unused-feedback` receives
rival bids but keeps chosen-action updates; `auctions-feedback` uses that information
to update every action against the same old continuation value. Other presets
cover local exploration, biased initial values, a downward nudge, nonparticipation,
a .20 reserve, three bidders, a uniform fringe bidder, and persistent exploration.
The biased values and downward trigger/clock are named readings of unspecified
source details. Twelve `auctions-*` sweeps expose initialization, ties, information,
market variants, and duration. The persistent preset runs 100 million auctions;
long surveys are separate from CI.
Native chart/CSV history beyond one million ticks retains the initial prefix and
latest snapshot; actual tick stamps show the gap. Full economic outcomes and both
occupancy histograms still count every auction.

The complete registered native studies give 14 Holds, four Fails, one Inconclusive
and five descriptive Untestable controls. Baseline terminal revenue is .232500 FPA
(1,000/1,000 stable) and .945295 SPA (999/1,000 stable); all-action feedback raises
FPA revenue to .906800 but misses Figure 5's .90-profile concentration rule.
The primary downward, fringe-concentration and persistent-occupancy checks fail;
three-bidder discount .999 has insufficient stable coverage. Initialization can
remove the baseline format distinction. All workloads and primary choices remain
fixed; [the full findings](docs/superpowers/specs/2026-10-02-q-learning-auctions-findings.md)
include conditional/unconditional outcomes, every sensitivity arm and provenance.

Source comparisons retain documentary differences: Figure 1's FPA counts imply
revenue .2265 whereas the text says .24; SPA includes six exceptions to the
(.95,.95) profile, including an off-diagonal pair. Full feedback's Figure 5 targets
(.90,.90), which is a weak first-price Nash equilibrium on this grid. Below-top
bids alone do not imply a profitable deviation. The survey separately judges the
figure, prose, source readings and protocol controls, using the preregistered
coverage and tolerance rules.

Low bids measure seller revenue loss and bidder benefit. They do not establish
communication, intent or punishment. Follow-ups show that information, updates,
participation, memory and pricing constraints can change coordination; the scalar
second-price result is therefore a statement about this model and protocol.

See [the design](docs/superpowers/specs/2026-10-02-q-learning-auctions-design.md)
and [reading notes](docs/superpowers/specs/2026-10-02-q-learning-auctions-reading-notes.md).


### Emergent polarity (Cederman 1994; 1997 chapters 4–5)

`polarity` reconstructs sovereign capitals and provinces on a bounded cardinal
lattice, with predator/status-quo types, signed normal resource stocks and
harvests, strict superiority/victory thresholds, conquest, collapse and behavioral
alliances. Inspect shows the government, members, fronts, commitments, paths,
trust, coalition and retained events. Territory, resources, strategy and coalition
views work through the shared native/WASM engine, Compare and Experiments.

Six presets expose the reconstruction: `polarity-original`, `polarity-defense`,
`polarity-alliances`, `polarity-pra`, `polarity-two-level` and
`polarity-overextension`. The original/defense/alliance presets use chapter 4;
PRA and the provincial variants use chapter 5's damage convention. Predator share
.2 is a playground choice; provincial presets use all predators. The predator-share
sweep is exploratory and does not execute the complete source studies. One economic
period is one full model iteration; batched ticks retain actual completed periods
and stop at the horizon or the configured hegemonic endpoint.

The complete frozen native workload retains **28,520 sessions across 572 arms**,
with the original 640 and separate 6,400 precision sessions, all named alternative
readings, allocation/support controls and provincial extensions. Registered judges
use 100,000 draws and seed 2026100202. Source mean/category judgments give
45 Compatible, 24 Incompatible and 3 Unresolved rows. The positive defense P2
and alliance P3 aggregate hypotheses fail.
28,281 sessions are valid; 239 invalid outcomes remain in raw and registered
denominators. No replacement seeds or fitted defaults were used.

[The complete findings](docs/superpowers/specs/2026-10-02-emergent-polarity-findings.md)
retain source digitization uncertainty, incompatible/unresolved strata, positive
hypotheses and negative source counterexamples separately. The uniform nonzero-share
defense/alliance contrasts in the probability of 2–10 sovereigns reproduce negative
source directions while the positive P2/P3 stabilization hypotheses fail; mean
polarity and hegemony are different measures. Compatibility is not
equivalence. No original EPM executable was recovered. Störmer's attributed
adaptation uses 110 configuration means over ten repeats, with a committed scatter;
no exact 75% inertness cutoff or unknown-seed overextension trajectory fit is claimed.

Direct initial/terminal nonpositive-stock frequencies have explicit stock and
session denominators; period-exposure frequency was not measured. Signed resource
creation, positive destruction and episode durations/end causes/censoring are
separate. Abstract resources do not measure casualties. Dated telemetry, strict
JSON export and sequential stale-front corrections retain original data/verdicts,
exact hashes, every affected before/after result and full same-seed rerun evidence.
See the [design](docs/superpowers/specs/2026-10-02-emergent-polarity-design.md),
[reading notes](docs/superpowers/specs/2026-10-02-emergent-polarity-reading-notes.md)
and [native protocol](survey/polarity/README.md).
