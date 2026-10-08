# Roadmap: future campaigns

Ideas beyond the current milestone, roughly in order of how much they add. Each item gets its own brainstorm → spec → plan cycle when picked up.

## Milestone 3: Chapter V — disease (done)

Immune and disease bit strings, immune response and transmission (E), metabolic symptoms,
immune-genome inheritance, disease mutation, outbreaks, Infect/Vaccinate tools, a disease
network overlay and the book's presets (V-1 eradication, V-2, which clears rather than
staying endemic under footnote 16's reading, and the McNeill outbreak), plus `vi-1-everything`. See
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
runs as presets. The paper's tables reproduce with the tie rule of its p. 442 (the current agent
wins); read as a coin flip (its p. 441 wording), two pairings give 42 % donation, not 4.3 %. Under any tie rule cooperation
rests on forced donation between identical tags: the strict test, a floor below zero or tag noise
collapse it, and tolerance fixed at zero raises it. The paper's cycle of clusters rising and being
invaded reproduces (a takeover every ~340 generations), each new cluster right next to the old. See `docs/superpowers/specs/2026-09-25-tags-design.md`.

## Milestone 13: Spatial games (done)

Nowak and May's spatial Prisoner's Dilemma (1992) as a seventh model kind, with Huberman and Glance's
asynchronous updating (1993) and Nowak, Bonhoeffer and May's probabilistic winning, continuous time, random
arrays and cubes (1994). The kaleidoscope reproduces exactly and the chaotic regime settles at 12 ln 2 − 8 to
three decimals; Huberman and Glance's one-defector takeover reproduces in the regime they ran (1.8 < b < 2),
and, as Nowak, Bonhoeffer and May showed, not below it; and
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
doubled-cost figures don't reproduce under any reading tried (seeing 56 %, here 65–67 %; color-blind 14 %,
here 42 %). Ethnocentrics take over
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
the two starts are equivalent, and of the two noise rules the Appendix's matches Table 2 more closely (the
other overshoots every fixed structure and erases FRNE's edge over 2DK). See
`docs/superpowers/specs/2026-09-26-social-structure-design.md`.

## Milestone 19: The demographic Prisoner's Dilemma (done)

Epstein's demographic Prisoner's Dilemma (1998) as a thirteenth model kind, with the working paper's rule,
Radax and Rengs' six unstated timing choices, soup, metabolism and the coordination game as switches and
presets. Cooperation dominates and soup runs to pure defection, as Epstein says, but Tables 1 and 2 are
rejected (729 / 171 cooperators and defectors against 779 / 121) under every timing setting but one of 64
for Run 2; they reproduce together only with the working paper's rule, founders with no wealth and newborns
acting at once, none of which the published text says. Run 4 dies out instead of cycling, the shifted
payoffs never converge to pure defection, footnote 27's monopoly comes in 1 run of 30, and the metabolism
"equivalence" is exact only when metabolism is charged per game. See
`docs/superpowers/specs/2026-09-26-demographic-pd-design.md`.

## Milestone 20: Norms and Metanorms (done)

Axelrod's norms and metanorms games (1986) as a model kind, with his dominance variant, Galán and
Izquierdo's departures (run length, mutation, meta-payoffs, temptation, three other selection rules) and
their readings of what Axelrod left unstated as switches. Axelrod's 100-generation results and dominance
claims reproduce; so do Galán and Izquierdo's reversals — metanorms usually collapse by 10⁶ generations,
and sooner under milder meta-payoffs, lower mutation or any other selection rule. The unstated details
decide the result: ties kept, the metanorm holds; a ranked refill, it collapses in every run. See
`docs/superpowers/specs/2026-09-26-norms-design.md`.

## Milestone 21: Image scoring (done)

Nowak and Sigmund's image scoring (1998) as a model kind, with Leimar and Hammerstein's island
model, errors, own-score, standing and q strategies, the payoff offset and how an observer records as
switches. (Numbered 21: the norms milestone takes 20.) NS98's universal constant reproduces to every
printed digit, and its unstated start is found (the negatives at −1, the rest out of reach); Fig. 2's
endless cycles and Fig. 3's group-size effect reproduce, the latter only when each observer keeps its own
tally. But Fig. 1's victory of k = 0 comes in 20 runs of 100 (defection wins 60), two interactions per
lifetime give cooperation 18 % of the time, not most, and Leimar and Hammerstein's island model does not
undo image scoring (44 % help against their 9 %), though standing invades as they say. See
`docs/superpowers/specs/2026-09-26-image-scoring-design.md`.

## Milestone 22: Relative Agreement (done)

Deffuant et al.'s relative agreement model with extremists (2002) as a model kind, with their pairwise
bounded confidence (2000) and §6 variants, Amblard and Deffuant's lattices and small worlds (2004),
Weisbuch's scale-free networks (2004), and Meadows and Cliff's (2012) and the authors' (2013) readings
of what the paper left unstated as switches. Fig. 9's layout reproduces as the paper states the model;
Meadows and Cliff's failure reproduces under their reading, and both of the reply's fixes are needed.
Balanced extremists' single extreme is a finite-size effect; Figs. 5 and 7 do not reproduce at their
stated parameters; eq. 11 as printed reproduces none of §6; the unstated cutoff for counting extremists
decides the network results. See `docs/superpowers/specs/2026-09-26-relative-agreement-design.md`.

## Milestone 23: El Farol and the Minority Game (done)

Arthur's El Farol bar (1994) and Challet and Zhang's minority game (1997) as one model kind,
with an explicit library, scoring and random ties, CMO-inspired payoff/binary variants, random
baseline, memory transition, mixed memories and evolution. The Arthur-inspired variant averages
near 60 with elevated variance and negative lag-1 correlation; this does not establish persistent
cycles. The memory transition has qualitative support. Fig. 4 central mass and Fig. 10 evolution
comparisons do not test histogram shape or population purity; a separate homogeneous-population
probe confirms Fig. 10's extreme waste. See `docs/superpowers/specs/2026-09-27-el-farol-design.md`.

## Milestone 24: Ants and Recruitment (done)

Kirman's recruitment chain (1993) as a model kind, with the three extensions he names but does not
run — Becker's majority pull, more food sources, meetings over a network — and Alfarano and
Milaković's agent rule and network critique (2007) as switches. The chain's long-run distribution
reproduces exactly (the beta-binomial), but never at the ants' 80–20; Becker's pull gives it. Figure
IIb's "average about one-half" needs a hundred times the figure's run; the herding fades as the
colony grows; a random network cures that under Alfarano and Milaković's rule but not Kirman's, and
their mean field fails on rings. See `docs/superpowers/specs/2026-09-27-ants-design.md`.

## Milestone 25: Threshold Models (done)

Granovetter's threshold model (1978) as a model kind, with the four extensions he sketches — friends,
crowds sampled from a city, clusters with movement, ceilings — and Watts's cascades on random
networks (2002). His crowds and Figure 2's continuous jump reproduce, but a crowd of real people has
no single tipping point, and the city's "equilibrium of 100" happens in 2 % of crowds; the friends
claims hold under a stated reading; middling movement is most incendiary; ceilings make riots pulse.
Watts's window and power law reproduce; his upper edge depends on n, his Fig. 4b cannot be built as
stated, and hubs help in both regimes. See `docs/superpowers/specs/2026-09-27-thresholds-design.md`.

## Milestone 26: The Timing of Retirement (done)

Axtell and Epstein's retirement model (1999; Epstein 2006, ch. 7) as a model kind: cohorts, deaths and
newborns, rational, random and imitating agents in transient networks, the policy switch from 65 to 62
and two coupled sub-populations, with the unstated rules — whom an imitator counts, what becomes of a
dead friend's place, activation order, transition time — as switches. The realizations reproduce in
shape (a little slower than stated) and the network-size effects as stated, but network extent has no
effect at 5 % rational; footnote 5 is false (counting every friend, no norm forms); Figure
6-6's minimum of rationality needs an unstated renewal rule; the policy switch's slow response does
not reproduce (the new norm comes in 2 periods); coupling slows the rational group as much as it
speeds the other. See `docs/superpowers/specs/2026-09-27-retirement-design.md`.

## Milestone 27: Altruistic Punishment (done)

Boyd, Gintis, Bowles and Richerson's altruistic punishment (PNAS 2003) as a model kind: groups of
contributors, defectors and punishers, payoff-biased imitation with mixing, intergroup conflict and
mutation, with the paper's structural variants (a per-capita benefit with payoff conflict, continuous
traits, a ring without extinction), Cooney's PDE critique (2024) and Janssen's NetLogo replication's
readings as switches. The figures' shapes hold under the text's rules, but not their reach; the
payoff baseline is unstated and Fig. 1's caption and legend disagree on the conflict rates. The
figures fit about twice the stated conflict rate: pairs fighting at 2ε reproduce all 14 curves of
Figs. 1–4, the text's rate 2 (either group starting a conflict, a switch here, 13). Continuous traits are not "similar"; the mixing
calibration is off by five; Cooney's payoff dip appears under every victory rule. See
`docs/superpowers/specs/2026-09-28-punishment-design.md`.

## Milestone 28: Zero-Intelligence Traders (done)

Gode and Sunder's double auction with zero-intelligence traders (JPE 1993) as a model kind, with
Cliff's critique, simulator and ZIP traders (HP Labs 1997): their five markets read from the scan
(Table 2's ZI-U efficiencies pin four of them down exactly), Cliff's markets, shifts and retail
market, and every unstated rule a switch. Gode and Sunder's efficiencies and dispersions reproduce —
but only with enough shouts per period, which their "30 seconds" never gives; Cliff's price
predictions miss the box markets and his 233⅓ is not his formula's, though his critique's direction
holds even in Gode and Sunder's mechanism; ZIP converges; and his code's momentum and day are not his
text's.
See `docs/superpowers/specs/2026-09-28-zi-traders-design.md`.

## Milestone 29: Balinese Water Temples (done)

Lansing and Kremer's water temples (American Anthropologist 1993) as a model kind on Janssen's data for
the Oos and Petanu, with Janssen's reanalysis (Agricultural Systems 2007): the watershed's subaks, dams,
rain, water and pests month by month; neighbor imitation, Janssen's plan search at six scales of
coordination, his two-node model, imitation discounted by network distance and adaptive subaks; and
his code's departures from the texts as switches. Imitation's rise and Table 1 reproduce, and hold
"every time"; but the patches' resemblance to the temples is the pest network's own, the subaks settle
harder than the paper says, and the paper's perturbed run never recovers. Finer coordination helps by
only 1–7 % — how much depends on the reading of the subak–dam columns, since water binds only under the
physical one — never Janssen's +30 %, and the temple scale is never best; his two-node threshold holds
exactly.
See `docs/superpowers/specs/2026-09-28-bali-water-temples-design.md`.

## Milestone 33: The Emergence of Firms (done)

Axtell's emergence of firms (Brookings working paper, 1999) as a model kind, with his 2013
parameterization: agents with preferences between income and leisure choose their effort in teams
with increasing returns and equal shares, moving between their firm, a start-up and their friends'
firms; every variation of his §4 and every unstated rule a switch. Table 1's §2 analytics reproduce
exactly (his other §2 claim, optimal group sizes under 10 below θ = 0.85, misses by a hair), but
the base case does not: firm sizes fall off far faster than his µ = 1.28 and firms
live about 4 periods rather than 23.4, under every reading. Most of §4's tables hold in direction;
three unstated details — whether sticky effort applies in a new firm, which way seniority pay runs,
and who covers a base-pay shortfall — decide whole tables; and his 2013 parameterization gives
Zipf's law, though the largest firm peaks at 3 000–5 800 of the 10 000 agents after the burn-in (5 seeds), though most of the time it stays under about 1 000 (median 780–955).
See `docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md`.

## Milestone 34: Algorithmic Collusion (done)

Calvano, Calzolari, Denicolò and Pastorello's Q-learning pricing algorithms (AER 2020) as a model
kind, checked against the authors' own Fortran: under the code's readings the sessions are theirs,
period for period, and Table I and Table A5 reproduce to the digit. The paper's text overstates its
tables (deviations unprofitable 93.6 %, not "more than 95 %"), and its equilibrium — a best response
— holds in 0.2 % of sessions where the code's one-period test passes half. The critics' tests are
switches, and most hold: memoryless firms price higher, myopic firms reach a quarter of the profit
gain, price increases draw the same "punishments" as cuts, synchronous learning halves the gain,
collusion does not survive a new rival, and the first 165 periods look like random pricing. Slower
exploration and Lambin's Theorem 1 do not hold up.
See `docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md`.

## Milestone 35: Q-learning Auctions (done)

Banchio & Skrzypacz (2022) is a separate `auctions` kind: fixed-value bidders,
first/second-price and mixture payments, disclosure versus counterfactual learning,
and the paper's market and exploration extensions. Initialization, ties and
underspecified source protocols are named reconstruction switches. The design
fixes histogram comparisons, coverage gates and decision rules before measuring;
full ensembles and long duration controls stay outside CI. See
`docs/superpowers/specs/2026-10-02-q-learning-auctions-design.md`.

The full registered native studies are measured: 11 source/reading checks hold,
four fail and the patient three-bidder check is inconclusive. Protocol/documentary
controls hold; initialization, ties, hindsight and duration remain descriptive.
[Findings and provenance](superpowers/specs/2026-10-02-q-learning-auctions-findings.md)
retain failed criteria and every declared arm. Fresh review, CI and deployed
Inspect/Compare/Experiments checks passed.

## Milestone 36: Emergent Polarity (published)

Cederman's 1994 EPM and chapter 5 provincial variants are the `polarity` kind,
with six inspectable presets and an exploratory predator-share sweep on the shared
native/WASM engine. Numerical policy, action memory, scheduling, paths, ties,
capital capture, transfer and provincial tax defaults are explicit reconstruction
choices. No original executable was recovered.

All 572 registered arms / 28,520 sessions and 100,000-draw judges are measured;
28,281 valid and 239 invalid outcomes remain retained. Source mean/category
judgments: 45 Compatible, 24 Incompatible, 3 Unresolved. Defense P2 aggregate: Fails; Alliance P3 aggregate: Fails. Original and precision populations,
source uncertainty, direct endpoint stocks and signed accounting remain separate.
Störmer's scatter has 110 configuration means × ten repeats. The source
overextension illustration uses a 10×10 grid (100 primitive units) with unknown
seed and remains descriptive. Approved diagnostics/export/obsolete-front/domestic decision
repairs preserve original bytes/verdicts and every affected before/after result.
[Findings and provenance](superpowers/specs/2026-10-02-emergent-polarity-findings.md) and
[war-study handoff](studies/2026-09-26-war-and-society.md) document the limits.
Fresh local Rust/native/Python/release WASM/web/typecheck gates, independent
whole-branch review and rereview, integrated CI and deployed Pages smoke checks passed.
See the [publication receipt](superpowers/specs/2026-10-03-emergent-polarity-publication.md).

## Milestone 37: GeoSim war sizes (published)

`geosim` reconstructs technological shocks, territorial conquest, contingent front
commitments and connected war clusters in the shared native/WASM engine. Five
presets and Inspect/Compare/Experiments retain resolved readings, source clocks
and complete/censored/queued censuses. Severity is abstract resource damage,
not battle deaths. The later artifact preset is an uncertified reading bundle.

All 37 arms / 1,490 registered attempts are retained: 1,486 complete and 4 invalid.
Ten of 11 conditional source-arm comparisons remain Unresolved because the
predeclared joint source-fit population is incomplete; the 75×75 arm is
Incompatible. Eight of 88 source targets are available,80 unavailable; all six
fixed technology/context source contrasts are Unresolved. Printed severity,
range and count uncertainties keep exact source equivalence Unresolved throughout.

Modern fits are available in 1,478 histories, with 8 insufficient tails and 4 invalid
histories. All 22 pools have 1,000 successful refitted KS draws: 15 reject,5 do not
reject,2 are inconclusive under the declared iid diagnostic. Thirty-six of 37
whole-history parameter summaries are available. The two descriptive modern
technology/context contrasts are separate from the six source mechanism tests.
Nested cutoff-Pareto calibration remains declared but unvalidated; pooled iid
nonrejection does not validate dependent histories or historical war data.

[Findings](superpowers/specs/2026-10-03-geosim-findings.md),
[execution provenance](superpowers/specs/2026-10-03-geosim-provenance.json) and the
[war-study handoff](studies/2026-09-26-war-and-society.md) retain every unavailable
slot and source limit. The [postmeasurement recorder amendment](superpowers/specs/2026-10-03-geosim-postmeasurement-recorder-amendment.md)
preserves the original measured source/binary, raw data and verdicts separately
from corrected and integrated executables. Original and amended plan packets are
retained in history. Final-head CI/Pages and deployed smoke remain publication gates.

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
- **Epstein's demographic Prisoner's Dilemma** (and Radax & Rengs' replication): done (Milestone 19).
- **Axelrod's norms and metanorms** (and Galán & Izquierdo's re-implementation): done (Milestone 20).
- **Nowak & Sigmund's image scoring** (and Leimar & Hammerstein's critique): done (Milestone 21).
- **Deffuant et al.'s relative agreement and extremism** (and Meadows & Cliff's replication): done (Milestone 22).
- **Arthur's El Farol and Challet & Zhang's minority game** (and the memory transition, and Challet, Marsili & Ottino's critique): done (Milestone 23).
- **Kirman's ants and recruitment** (and Alfarano & Milaković's network critique): done (Milestone 24).
- **Granovetter's threshold models** (and Watts's global cascades): done (Milestone 25).
- **Axtell and Epstein's timing of retirement**: done (Milestone 26).
- **Boyd, Gintis, Bowles and Richerson's altruistic punishment** (and Cooney's PDE critique): done (Milestone 27).
- **Gode and Sunder's zero-intelligence traders** (and Cliff's critique and ZIP traders): done (Milestone 28).
- **Lansing and Kremer's Balinese water temples** (and Janssen's reanalysis): done (Milestone 29).
- **Axtell's emergence of firms** (and his 2013 parameterization): done (Milestone 33).
- **Calvano, Calzolari, Denicolò and Pastorello's algorithmic collusion** (and its critics: Asker, Fershtman & Pakes; Lambin; Epivent & Lambin; den Boer, Meylahn & Schinkel; Eschenbaum, Mellgren & Zahn): done (Milestone 34).
- **Banchio & Skrzypacz’s Q-learning auctions**: done (Milestone 35); baseline and feedback direction hold, four extension/endpoint criteria fail, and patient three-bidder coverage is inconclusive.
- **Cederman emergent polarity and provincial variants**: measured (Milestone 36; published). 572 arms / 28,520 sessions retained; 45 Compatible, 24 Incompatible, 3 Unresolved; Defense P2 aggregate: Fails; Alliance P3 aggregate: Fails. See [findings](superpowers/specs/2026-10-02-emergent-polarity-findings.md). GeoSim war-size findings are measured separately (Milestone 37).
- **Cederman GeoSim war sizes**: published (Milestone 37). 37 arms / 1,490 attempts: 1,486 complete, 4 invalid; 10 conditional Unresolved, 1 Incompatible, all source-equivalence and six mechanism slots Unresolved. [Findings](superpowers/specs/2026-10-03-geosim-findings.md).
- **Minds 1: the utility mind and the ideal free distribution** (our experiment; docs/studies/2026-09-27-minds.md): done.
- **Minds 2: A\* and walking; which of the book's results need the jump** (our experiment; docs/studies/2026-09-27-minds.md): done.
- **Minds 3: memory, belief and truffles; memory's value as an information asymmetry** (our experiment; docs/studies/2026-09-27-minds.md): done. Memory mostly hurts under rule M, which prices no travel; the marginal value theorem moves to Minds 4.
- **Minds 4: GOAP and the marginal value theorem; does memory pay a mind that prices travel** (our experiment; docs/studies/2026-09-27-minds.md): done. Planners stay longer when travel is longer; overstaying isn't shown; memory pays a planner among the living.
- **Minds 5: caching for the future; which hypothesis the jays' caches resemble, winter, and central-place foraging** (our experiment; docs/studies/2026-09-27-minds.md): done. Each caching rule caches as derived in Raby's and Amodio's protocols (a planner looking a day ahead caches nothing when tomorrow has food, so Raby's planner claim is Weak), and the paper's Bayesian comparison reproduces; the jays' pattern looks like the even splitters'. Caching gets agents through winter, planning best (88 % against 49 %). Loads rise with distance, but Lima's equal near and far loads fail, and the analytic optimum is a cancellation.
- **Minds 6: theft; the first step of the pilfering campaign** (our experiment; docs/studies/2026-09-27-minds.md): done. Agents stumble on each other's caches; the chance to find one is a free parameter (0.25 an anchor), and stumbling can't reach the field's median pilferage (7.0 % a day even finding every cache stood on). Theft pools stores (winter survival 74 % → 90 %); cheaters win by transfer, and the owner's advantage goes nearly unused (with half cheaters, owners dig back 1.5 % of the sugar dug or pilfered). Andersson and Krebs's threshold is Weak and turns on valuing still-buried caches; equal recovery holds (as an equal chance per draw: owners still cross their own caches far more often); reciprocity splits; Vander Wall and Jenkins's 18 % is untestable. Next: P1b, the evolution of larder hoarding (Minds 7); then watching, protection and deception; behavior trees after.
- **Minds 7: the evolution of hoarding; Vander Wall and Jenkins's genetic algorithm (P1b of the pilfering campaign)** (our experiment, a reproduction first; docs/studies/2026-09-27-minds.md): done. A non-spatial `hoard` model built from the paper's Appendix, gaps filled as stated choices. Larders were first weighted per item and never took over in 15 diagnostic runs; the text (pp. 662, 664) supports per burrow, which became the default, a change prompted by that result and disclosed (the survey later ran per item too: 0 of 1 350 runs took over). Per burrow, the threshold is Weak (50 % point 0.235 against 0.219), runs end low or high (98.6 %) but rise slower than "within 10 generations" (as does the paper's own Fig. 2A example), larder loss exceeds scatter loss in 93.8 % of runs, and the best-early-larder predictor holds but barely beats the ratio alone: a partial reproduction. Per item reproduces the loss statistics but not the outcome, so neither reading reproduces both, and the match is contingent on three unprinted choices (per burrow, defense slope, V_seg); the paper's sensitivity analysis is unfound. New ground: under the paper's fitness a non-hoarding cheater is gone in one generation; owner recovery shows no clear effect. Next: P2, watching, or a spatial P1b (not yet chosen); then protection and deception; behavior trees after.
- **Minds 8: watching; observational spatial memory and deliberate raids (P2 of the pilfering campaign)** (our experiment; docs/studies/2026-09-27-minds.md): done, in two rounds. Agents who see another bury remember the cache for a span (7 ticks, a gap choice between Heinrich and Pepper's next day and 14 days) and go to take it, with no draw; by default they raid only when the cache is worth at least the site (`raid_if: better`). Four audits found three of the first design's five results foreseeable (a strawman field band, a ratio at its floor, "pure scroungers" who still produced), so a second round, with a pre-mortem, replaced them. With a 7-tick memory, watching takes fresh caches twice as fast as stumbling (a hazard of 6.1 % a day against 3.0 %, 20 of 20 seeds), action-bound (watchers see 88 % of burials and raid 54 % of what they see), and costs lives (59 % survive the first winter against 70 %; about two thirds of that is the harvest a raid replaces). In a calibrated world where Andersson and Krebs's condition held (`watch-ak`), watching flips it (p_s ÷ p_o 3.57 → 0.67), but the hoarders' fitness hardly follows (signs agree in 62 of 100 runs; neither being watched nor their own raiding costs 0.05 in any seed). Scroungers who forgo producing fall further behind as they become common (slope −0.20) but trail even when rare (at a share of 0.1 the mean shortfall of 0.056 is 0.021 without watching, from never caching, plus 0.035 that watching adds), so no stable mix forms; among watchers who also bury, no fall in their lead was detected, and the world loses as more watch (a mild social dilemma). An upper bound (no hiding, no defense, generous memory): at spans 1–2, the fresh-cache, flip and scrounger results reverse, and the watchers' lead in 3c isn't detected at spans 1–3. Next: Minds 9, a spatial P1b (heritable larder hoarding in the Sugarscape world); then protection and deception; behavior trees after.
- **Minds 9: spatial hoarding, paid defense and seasonal inheritance (spatial P1b)** (our experiment; [study](studies/2026-09-27-minds.md), [complete results](../survey/out/minds9-results.md), [approved protocol](superpowers/specs/2026-10-02-minds-9-judge-amendment.md)): done. All 81×40 comparison runs and 21×5 serial timing runs completed; no terminal runs. Unguarded L=1 minus L=0 changes survival by −10.24/−4.50/+18.64 points at watching off/2/7; paid guarding lowers survival in every paired seed (−30.50/−45.26/−46.69 points), despite improved recovery. Neutral/inheritance controls separate drift from selection, and Stores selection lowers L without uniformly improving survival. Frozen rare/common scrounger and burying-watcher directions depend on span; seed heterogeneity and missing groups remain visible, with no stable-mixture or calibrated-apparency verdict. Exact overlap after reviewed scheduling optimization and byte-identical saved-data reanalysis preserve reproducibility. Next: [P3 costly re-caching design](superpowers/specs/2026-10-02-minds-protection-design.md), approved; its [implementation plan](superpowers/plans/2026-10-02-minds-protection.md) is complete; verification and independent task/whole-branch reviews passed. The separately approved native gate now precedes a complete 7,680-episode [P3 campaign and dated findings](superpowers/specs/2026-10-06-minds-protection-findings.md), independently reviewed; publication pending; all 208 descriptive estimates remain separate by registered stratum. Private re-caching benefit depends on renewed observation, cost and recovery geometry. The original [protocol](superpowers/specs/2026-10-02-minds-protection-protocol.md) retains its registration bytes/status. P4 [supplied caching gestures](superpowers/specs/2026-10-07-minds-deception-protocol.md) is measured and independently empirically Approved: [dated findings](superpowers/specs/2026-10-07-minds-deception-findings.md) retain all 96×40 episodes, 64 primary and 64 separate secondary estimates with byte-identical saved-data reanalysis. Off-route diversion can improve food/lifetime; on-route false inspection does not, and ordinary recovery/source occupancy matter. Final integration review, publication, remote verification and final archival remain pending. Behavior trees follow.
- **Credit hierarchy view**: done (Milestone 6).
- **Minds and collective agency, long-term campaigns:** [program design](superpowers/specs/2026-10-02-minds-collective-agency-program-design.md), proposed for review. Affordances and environmental controls; persistent messages and private networks; learning and information-seeking; social knowledge; shared minds; costly helping; distributed research; safety/oversight; culture and environmental inheritance. The first bounded P3 protection and P4 supplied-gesture campaigns are measured and independently reviewed; broader protection/deception and these later features remain prospective. The first new integration target is a gate, transferable key and message board, followed by collective research and incident-shaped intervention experiments with cheap, inspectable agents.

- **Institutional corruption, future campaign:** deferred October 6, 2026. A potential political-science collaborator studies how institutions become corrupted; the institution, research question and empirical targets remain unspecified. Possible replication starting points are [Hammond's agent-based corruption model (2000)](https://www.brookings.edu/articles/endogenous-transition-dynamics-in-corruption-an-agent-based-computer-model/) and [Lee et al.'s cooperation, monitoring and corruption cycles (2019)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6613283/). [Tirole's organizational collusion model (1986)](https://academic.oup.com/jleo/article-abstract/2/2/181/848407) and [the Thompson–Lessig institutional-corruption literature](https://hls.harvard.edu/bibliography/institutional-corruption-revisited-exploring-open-questions-within-the-institutional-corruption-literature/) offer additional foundations. Candidate extensions: corrupted oversight and reporting, capture through appointments/funding/enforcement priorities, and cycles of costly vigilance. Existing deduction atoms could support information manipulation; authority, resource allocation and repeated institutional feedback would need explicit mechanics. Define corruption relative to institutional purpose, including legal systemic diversion, and measure mission performance, selective enforcement, concentration of benefits, reporting accuracy and recovery after reform. Replicate first, then add one mechanism tied to the collaborator's question and observable evidence. No model, design or implementation is selected; this idea does not redirect the active deduction work.

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
