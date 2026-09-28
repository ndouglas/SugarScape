# Minds 3: memory, belief and truffles (design)

**Date:** 2026-09-28
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 3. Our own experiment, numbered apart
from the reproductions' milestones.
**Builds on:** the milestone specs and Minds 1–2 (`2026-09-27-minds-1-utility-design.md`,
`2026-09-27-minds-2-walking-design.md`). All remain binding where not changed here. In particular:
the decision seam, walking (`movement.mode: walk`, A*), walls and fences, the
literal-default-plus-named-switch pattern, the titles rules, and the Minds rule that every engine
reduces to what came before when its switches are off.

**Sources:**
- E. L. Charnov, "Optimal foraging, the marginal value theorem", *Theoretical Population Biology*
  9(2) (1976), 129–136 (open scan; copy in `papers/foraging/`).
- D. W. Stephens and J. R. Krebs, *Foraging Theory* (1986) (copy in `papers/foraging/`).
- B. Y. Hayden, J. M. Pearson and M. L. Platt, "Neuronal basis of sequential foraging decisions in
  a patchy environment", *Nature Neuroscience* 14(7) (2011), 933–939 (PMC3553855).
- P. Nonacs, "State dependent behavior and the Marginal Value Theorem", *Behavioral Ecology* 12(1)
  (2001), 71–83 (*not in `papers/`*).
- C. Bracis, E. Gurarie, B. Van Moorter and R. A. Goodwin, "Memory effects on movement behavior in
  animal foraging", *PLoS ONE* 10(8) (2015), e0136057.
- D. Boyer and P. D. Walsh, "Modelling the mobility of living organisms in heterogeneous
  landscapes: does memory improve foraging success?", *Phil. Trans. R. Soc. A* 368 (2010),
  5645–5659 (arXiv 1006.0079).
- J. D. Thomson, M. Slatkin and B. A. Thomson, "Trapline foraging by bumble bees: II. Definition
  and detection from sequence data", *Behavioral Ecology* 8(2) (1997), 199–210 (open at Thomson's
  lab site; copy in `papers/traplining/`).
- J. D. Thomson, "Trapline foraging by bumblebees: I. Persistence of flight-path geometry",
  *Behavioral Ecology* 7(2) (1996), 158–164 (open; copy in `papers/traplining/`).
- K. Ohashi and J. D. Thomson, "Efficient harvesting of renewing resources", *Behavioral Ecology*
  16(3) (2005), 592–605 (open; copy in `papers/traplining/`).
- K. Ohashi, A. Leslie and J. D. Thomson, "Trapline foraging by bumble bees: V. Effects of
  experience and priority on competitive performance", *Behavioral Ecology* 19(5) (2008), 936–948
  (open; copy in `papers/traplining/`).
- F. B. Gill, "Trapline foraging by hermit hummingbirds: competition for an undefended, renewable
  resource", *Ecology* 69(6) (1988), 1933–1942 (*not in `papers/`*).
- M. Lihoreau et al., "Radar tracking and motion-sensitive cameras on flowers reveal the development
  of pollinator multi-destination routes over large spatial scales", *PLoS Biology* 10(9) (2012),
  e1001392.
- Hornvale's kernel M2c1 design (`../hornvale/docs/superpowers/specs/2026-09-26-kernel-m2c-minds-things-contests-design.md`):
  beliefs as dated observations with persistence models. It's the docking reference for `belief`.

## Goal

Give Flumps memory of the sites they've seen, and a belief about what a remembered site holds now.
Then measure memory's value as an information asymmetry, rememberers against non-rememberers in
the same world, across environments chosen so that information beyond sight does or doesn't
matter.

Add **truffles**: hidden spots, revealed only by standing on them, that regrow a fixed time after
harvest. They give memory something only it can exploit.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry, legacy fixture and pinned fingerprint stays
  green and unedited. `memory.span: 0` and `truffles.share: 0` are the defaults, and they draw
  nothing new from `World.rng`.
- **The reduction holds exactly:** with `span: 0` (or `share: 0`) and no truffles, every walking
  preset has Minds 2's fingerprint.
- **Truffle spots are placed by a hash of (site, `truffles.seed`), not by `World.rng`.** So the
  layout is the same with memory on and off, and paired comparisons compare like with like.
- **Deterministic and portable.** No new irrational arithmetic.
- **Truthful titles and descriptions.** Results are ours. The expectation that memory is worth
  little on the open scape is stated, and reported whatever happens.

## Source summary

- **The marginal value theorem** (Charnov, p. 132): "The predator should leave the patch it is
  presently in when the marginal capture rate in the patch (∂g/∂T) drops to the average capture
  rate for the habitat."
  - It assumes patches are visited "with little or no revisitation" (p. 130). Memory exists to
    revisit, so memory strains the theorem's assumptions.
  - Its robust qualitative prediction (Stephens and Krebs, Fig. 2.2): "When the travel time is
    long … the rate-maximizing residence time … is long."
- **Hayden, Pearson and Platt (2011):** "monkeys' patch-residence times rose with increasing travel
  time and were nearly rate maximizing … Both monkeys remained in patches slightly longer than
  predicted by the MVT."
- **Nonacs (2001):** "across a survey of 26 studies, foragers rather consistently 'erred' in
  staying too long in patches."
- **Bracis et al. (2015):**
  - "memory almost always leads to improved foraging success, but … this effect is most marked in
    landscapes containing sparse, contiguous patches of high-value resources that regenerate
    relatively fast";
  - "the rate of forgetting (the short-term memory decay rate) tracked the rate of environmental
    change (the regeneration rate)."
- **Boyer and Walsh (2010):** "excessive memory use over stochastic decisions prevents the forager
  from updating its knowledge in rapidly changing environments."
- **Traplining** (Thomson, Slatkin and Thomson, 1997): "repeated sequential visits to a series of
  feeding locations".
  - Their index of return variability divides the variance of return lengths (steps before
    returning to the same site) by its mean under 999 shuffled, non-traplining sequences:
    "zero for a perfectly traplining bee and 1.0 for a non-trapliner that exactly matches the null
    model".
  - The authors call it the most problematic of their tests for significance.
- **Ohashi and Thomson (2005):**
  - "Complete traplining always produces less variation in elapsed time between visits than random
    searching … Moreover, the systematic revisitation schedule produced by complete traplining makes
    it more competitive, regardless of resource renewal schedule or competitor frequency."
  - A spot that ripens fully a fixed time after harvest is their capped (nonlinear) renewal case,
    where traplining pays most.
- **Gill (1988):**
  - "Competition caused hummingbirds to visit a feeder frequently, often before a scheduled
    refill";
  - "Under conditions of nearly exclusive use … the hummingbirds adjusted their visits to operant
    (fixed-interval) schedules."
- **Ohashi, Leslie and Thomson (2008):** experienced bees won more nectar "because they traveled
  faster between flowers and returned to flowers at more regular intervals".
- **Hornvale M2c1:**
  - In its census, 9 of 10 goblin deaths came from giving up on a fished-down lake the goblin
    believed would stay empty.
  - Its fix is the "dynamics" persistence model: "A lake seen at 6 fish, with a known regrowth
    rate, is believed to be recovering. The projection is the belief."
- **The book:** Sugarscape's agents have no spatial memory. Its only memories are immunological,
  plus the friends list. The book's remedy for myopia is foresight (p. 129).

## Architecture

- **Memory** (`crates/sugarscape-core/src/minds/memory.rs`):
  - a Flump's remembered sites, as `BTreeMap<u32 site index, Seen>` so iteration order is
    deterministic;
  - `Seen { levels: [f64; MAX_GOODS] (as seen), most: [f64; MAX_GOODS] (the most ever seen there), tick: u64, truffle: Option<TruffleSeen> }`;
  - `TruffleSeen { ripe: bool, tick: u64 }`.
- **Agents:** `Agent.remembers: bool` and `Agent.memory: Memory`.
  - Memory is behavioral state. It isn't hashed: behavior shows in positions and holdings, which
    are hashed. Checkpoints and keyframes clone the world.
  - `remembers` is drawn at birth with probability `memory.share`, only when `span > 0`, and drawn
    last, so every other draw is unchanged (the Axelrod precedent).
- **Truffles** (`crates/sugarscape-core/src/rules/truffles.rs`):
  - a per-site `ripe_at: Option<u64>`, where `None` means no spot;
  - built at world creation from the hash;
  - harvested in the move step when a Flump stops on a ripe spot.
- **The decision:** `movement::candidates` adds remembered out-of-sight sites for a Flump that
  remembers, with believed values. `choose` and the utility mind's score are unchanged.
- **Series:** memory diagnostics and truffle counts, present when memory or truffles are on.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `memory.span` | 0 | reset | ticks a remembered site is kept after it was last seen; 0 means no memory (the book) |
| `memory.share` | 1.0 | reset | the share of Flumps born remembering, when `span > 0` |
| `memory.belief` | `project` | live | `recall` (a remembered site holds what it held when seen) or `project` (plus growback since, capped at the most ever seen there) |
| `truffles.share` | 0 | reset | the share of sites with a truffle spot; 0 means no truffles |
| `truffles.value` | 5 | live | sugar a ripe truffle yields |
| `truffles.regrow` | 30 | live | ticks after a harvest until the spot is ripe again |
| `truffles.seed` | 1 | reset | the layout hash's seed, independent of the world's seed |

**Validation:**
- `span` is 0–10 000, `share` 0–1, truffle `share` 0–1, `value` ≥ 0, and `regrow` 1–10 000.
- `span > 0` requires `movement.mode: walk`: "remembered sites out of sight can only be walked
  to".
- Truffles need no memory, since anyone can find a spot by stepping on it.

## Memory

- **Observing.** Each tick, after moving, a Flump that remembers records every site in its sight,
  and its own site, with the levels it sees and the tick. It keeps `most` as the running maximum.
  A truffle spot is recorded only on the site the Flump stands on, with whether it was ripe.
- **Forgetting.** An entry whose last sighting is more than `span` ticks old is dropped. That
  happens lazily when candidates are built, and on a periodic sweep.
- **Belief** about a remembered site's level of good i now, at age a = now − tick:
  - `recall`: the level seen;
  - `project`: `min(seen + rate · a, most)`, where `rate` is the growback rate in force (seasons
    included, at the site's row, as growback computes it). Under instant growback, it's `most`.
- **Truffle belief:**
  - `recall`: ripe if it was seen ripe, otherwise not.
  - `project`: ripe once `regrow` ticks have passed since it was seen unripe or harvested. A Flump
    that harvests a spot knows exactly when it ripens; one that finds it unripe assumes it was
    just harvested.
- **Candidates:**
  - Sites in sight carry their true levels, plus the believed truffle value if the Flump knows a
    spot there. Truffles are hidden from sight.
  - Remembered sites out of sight carry believed levels plus the believed truffle value, at their
    lattice (torus Manhattan) distance.
  - Welfare is rule M's own: sugar, or foresight welfare over n goods, with truffle sugar added to
    good 0.
- **Arriving.** A walker whose target is occupied when it reaches it stays where it stands. That
  was already true, since the target is impassable while occupied.

## Truffles

- **Layout:** a site has a spot when `hash(site index, truffles.seed) < share · 2⁶⁴`. The hash is
  SplitMix64's output function (already `landscape::mix`). Walls have no spots.
- **Ripeness:** every spot starts ripe. When a Flump stops on a ripe spot, it gathers `value` sugar
  (added to good 0 with the site's own gather), and the spot ripens again at `tick + regrow`.
  Nobody can see a spot from a distance.
- **Finding:** anyone stopping on a ripe spot gathers it. That includes non-rememberers, who find
  truffles only by chance.

## Statistics

Present when `span > 0`:
- `remembering`: the share of living Flumps that remember;
- `remembered_moves`: the share of this tick's moves whose target was a remembered out-of-sight
  site;
- `belief_error`: the mean |believed − actual| of good 0 plus truffle sugar at arrivals this tick
  at remembered targets, or 0 with none;
- `stale_arrivals`: the share of those arrivals that found less than believed;
- `wealth_rememberers` and `wealth_others`: mean sugar held by each group.

Present when truffles are on:
- `truffles_found`: truffles gathered this tick;
- `truffles_by_rememberers`: those gathered by Flumps that remember.

## Environments and presets

Titles are drafts, to be replaced by measured wording. Every memory preset walks.

| Preset | World | Why memory might matter |
|---|---|---|
| `mem-open` | `walk-capacity` with memory (span 100, share 0.5) | The control. Rich mountains always in sight, crowded, fast-depleting. Expected: little or no advantage. |
| `mem-catchment` | Minds 1's `ifd-no-starving` world, walking, utility mind with wander, memory (span 200, share 0.5) | Patches out of sight; wanderers that once saw a patch can return. |
| `mem-walled` | `ifd-wall` with memory (span 200, share 0.5) | The wall hides the other patch; only those who passed the gap know it. |
| `mem-seasons` | `walk-seasons` with memory (span 100, share 0.5) | The rich hemisphere alternates; remembering the other one pays. |
| `mem-truffles` | `walk-capacity` with truffles (share 0.05, value 5, regrow 30) and memory (span 200, share 0.5) | Hidden spots only memory exploits. |
| `mem-trapline` | a sparse world: 20 Flumps, flat low sugar (capacity 1, growback 0.1), truffles (share 0.02, value 10, regrow 40), memory (span 400, share 1), vision 1–6 | Spots are the main food; traplining should appear. |
| `mem-mvt` | 10 Flumps on a 60 × 60 torus with 9 equal patches (peaks, radius 4, height 4) on a square lattice of spacing 20, vision 1–20, growback 0.25, the utility mind with travel 0.5, memory (span 400, share 1, `project`) | Few foragers, patches in sight, and travel that takes time: the marginal value theorem's setting. |

## Sweeps

Twenty seeds.
- `mem-span`: the rememberers' wealth advantage (`wealth_rememberers` − `wealth_others`, as a mean
  over ticks 200–500; see the Survey for the per-seed fit) against `span` (0, 10, 25, 50, 100,
  200, 400). The series are growback 0.25, 0.5 and 1, on `mem-open`'s world under `recall` and
  under `project` (two sweeps: `mem-span-recall`, `mem-span-project`).
- `mem-share`: the advantage against `share` (0.1, 0.25, 0.5, 0.75, 0.9) in `mem-truffles`. Is
  memory worth more when rare?
- `mem-mvt-spacing`: mean patch residence (see Survey) against the lattice spacing (12, 16, 20,
  24), with series rule M and the utility mind with travel.

A sweep's metric must be a series. Residence and advantage are computed from series or
positions; where a sweep needs them, the Statistics above gain `wealth_advantage` (rememberers
minus others) and `mean_residence` (the mean length of patch visits completed this tick, or the
last value carried forward). Both are present only when memory is on.

## Survey

A `minds3` claims module. Same-world comparisons of rememberers and others are paired within each
seed. Memory-on against memory-off comparisons are paired across the same seeds.

1. **Memory's value by environment:** the rememberers' wealth advantage (ticks 200–500, per seed)
   in each preset.
   - Expected, and reported whatever happens: about 0 in `mem-open`; positive in `mem-walled`,
     `mem-seasons`, `mem-truffles` and `mem-trapline`.
   - `mem-catchment`'s time to first reach a patch is reported for rememberers and others.
2. **Truffles:** rememberers gather more truffles per head than others (paired, `mem-truffles`).
3. **Traplining** (Thomson, Slatkin and Thomson, 1997; Ohashi and Thomson, 2005; Gill, 1988):
   - **The index:** in `mem-trapline`, each rememberer's sequence of truffle-spot visits gives its
     index of return variability: the variance of return lengths, divided by its mean over 999
     shuffles of the same sequence (drawn from a survey-side seeded RNG, not `World.rng`). The
     index is below 1 for most rememberers (per seed, the median index < 1, and < 0.8 in ≥ 80 %
     of seeds). Non-rememberers' visits are the control.
   - **Timing and competition:** the median revisit interval is reported against `regrow`. Gill
     predicts revisits before the regrowth time under competition and near it when alone, so
     `mem-trapline` is run at 20 Flumps and at 5 (a paired comparison of median intervals: shorter
     with more competitors).
   - **Payoff:** rememberers' truffle intake beats non-rememberers' in a mixed run (`share` 0.5), as
     Ohashi and Thomson's "more competitive" predicts.
4. **The marginal value theorem, travel time:** in `mem-mvt`, residence rises with spacing under
   the utility mind with travel (the per-seed slope of residence on spacing > 0). Under rule M,
   with no travel cost, the slope is reported. Expected: flat.
5. **Overstaying** (Nonacs): at each departure, the Flump's intake over its last tick in the patch
   is compared with its long-run mean intake rate. Overstaying means the marginal rate at leaving
   is below the average; the share of departures that overstay is reported.
6. **Forgetting tracks regrowth** (Bracis et al.): the span that maximizes the rememberers'
   advantage shortens as growback speeds up (under `recall`). Under `project`, stale memories
   mislead less, so the best span is longer or the advantage keeps rising. The direction is
   reported.
7. **Hornvale's pathology:** `project` beats `recall` (a larger advantage and a smaller
   `belief_error`), paired in `mem-truffles` and `mem-open`.
8. **Minds 2 follow-up:** walking's lost capacity. Does `walk-capacity` with memory for everyone
   (`share` 1) raise the population against walking without memory (paired)?

## Page

- **Rules panel:** a **Memory (Minds 3)** group (`span`, `share`, `belief`) with a note that memory
  needs walking, and a **Truffles** group (`share`, `value`, `regrow`, `seed`).
- **Grid:**
  - Truffle spots are hidden, since nobody can see them.
  - The inspected Flump's known spots are drawn as small marks.
  - Its remembered sites are drawn as a faint overlay, fading with age. This is the view of
    what one Flump knows.
- **Inspect:** "Remembers: n sites (m truffle spots)", or "Doesn't remember".
- **Charts:**
  - **Memory:** `remembered_moves`, `stale_arrivals` and `belief_error`;
  - **Rememberers vs others:** the two wealth lines;
  - **Truffles:** found, and found by rememberers.
- **Types:** `memory?`, `truffles?`, and the `AgentView` fields.

## Testing

- **Golden and legacy:** existing entries untouched; new entries for every preset; titles.
- **Reduction** (`tests/minds.rs`):
  - every walking preset with memory switched on at `span` 0, or `share` 0, has its fingerprint
    unchanged;
  - every preset with `truffles.share` 0 is unchanged.
- **Memory:**
  - recording in sight and on one's own site;
  - forgetting at `span`;
  - `recall` and `project` beliefs, including the cap at `most` and seasons' rate;
  - remembered candidates out of sight at lattice distance;
  - sight overriding memory;
  - rememberers only;
  - `remembers` drawn only under memory, and last.
- **Truffles:**
  - the layout comes from the hash and is independent of the world seed;
  - no spots on walls;
  - gathered on stopping when ripe; ripe again after `regrow`;
  - invisible to non-knowers;
  - the truffle beliefs.
- **Series:** present only when memory or truffles are on, with values on hand-built worlds.
- **WASM:** a pinned fingerprint for `mem-truffles`.
- **Web:** the groups, the overlay and the Inspect row.

## Docs

- **README:** a Minds 3 section.
- **The program document:** results, the cost table extended, and **Minds 4's target**: GOAP and
  caching. A carrying limit gives the reason to bury surplus, winter gives the reason to plan, and
  pilfering and re-caching when observed set up the deception program.
- **Roadmap:** the Minds line.
- **The spec's amendments.**

## Amendments (implementation)

These change or extend the sections above. The measured values are the survey's (20 seeds) and the
sweeps' (20 seeds), with the source named. The cost figures are the release CLI's.

- **Ruling: an occupied remembered target.** A walker's A\* goal may be occupied, because a
  remembered target can be, and the Flump can't see who stands there. If the site it would stop on
  is occupied (possible only at the target), it stops at the site before it on the path, or stays.
  This supersedes the Memory section's "stays where it stands".
- **Ruling: diagnostics measured at choosing.** `belief_error` and the share of stale choices are
  measured when a Flump chooses a remembered target out of sight, as believed against true value
  then, not on arrival. The series `stale_choices` (the share of those choices whose target was
  truly worth less than believed) replaces `stale_arrivals`. The page's **Memory** chart shows
  `stale_choices`.
- **Remembered sites carry no pollution discount.** Memory keeps no pollution, and the Flump can't
  see it out of sight. The true value in `belief_error` does include the discount, so under
  pollution the belief error includes pollution the Flump couldn't see. No memory preset uses
  pollution.
- **Wander and memory.** Under the utility mind's idle `wander`, a Flump wanders only when nothing,
  in sight or remembered, scores above 0, and then draws only among sites in sight. A remembered
  site worth more than 0 is chosen as any other candidate. A non-rememberer's draws are unchanged.
- **The truffle hash, as built.** A site has a spot when
  `mix(index ⊕ ((seed << 32) | 0x5eed)) < share · 2⁶⁴` (`rules::truffles::has_spot`), with
  `share ≥ 1` meaning every non-wall site exactly. Every spot starts ripe. A live change to
  `truffles.regrow` applies only to future harvests: a spot already waiting keeps its ripe tick.
- **Inspect and the overlay, as built.**
  - `AgentView.memory` is `{ remembers, sites, spots }`, or `None` when `span` is 0.
  - `World::memory_view` and the WASM `inspect_memory(x, y)` give the inspected Flump's remembered
    sites as flat `[x, y, age, spot]` records in site order, `spot` 0 (none known), 1 (believed
    unripe) or 2 (believed ripe). They're empty for a non-rememberer, an empty site or memory off.
  - The page's worker plumbing follows `trail()`: `Wants.memory`, `WorldSnapshot.memory` and
    `Engine.inspectMemory()` keep the records in step with the selection, cleared on reset.
  - The grid draws remembered sites as squares fading with age (alpha 0.4 at age 0, 0 at `span`),
    and known spots as circles, filled when believed ripe. Inspect's row reads "Remembers: n sites
    (m truffle spots)", singular at 1, or "Doesn't remember".
- **The reduction, as built.** `memory.span` 0 with `truffles.share` 0, written in explicitly, gives
  every non-Minds-3 preset its own fingerprint (`tests/minds.rs`). The walking reduction sets
  `span` 0 on both sides, since a memory preset can't be forced to jump. `share` 0 with `span` > 0
  is checked by behavior (nobody remembers, no remembered moves), not by fingerprint, since
  `remembers` is still drawn.
- **Sweeps, as built.** `mem-span-recall` and `mem-span-project` use spans 10–400; span 0 is left
  out, since the advantage is undefined with nobody remembering. `mem-share` is as specified.
- **MVT residence and overstaying are survey measures,** with no series or sweep: `mean_residence`
  and the `mem-mvt-spacing` sweep were not built. The survey steps the world and times completed
  patch visits itself, on nine peaks on a 3s × 3s torus at s = 12, 16, 20 and 24, with vision 1–18
  throughout (a 36 × 36 torus at s = 12 can't have vision 20).
- **The MVT world.** Two redesigns were tried and withdrawn, and the rich-patch preset was restored
  (golden `0xa00361ad0017c367` again).
  - Depleting patches (radius 2, growback 0.05, 0.45 sugar a tick per patch) with 10 Flumps: nobody
    is alive after tick 200.
  - The same with 3 Flumps (4.05 sugar a tick in all, against a need of 3): nobody is alive at tick
    1000, and only 3 departures happen across 20 seeds.
  - Rich patches: foragers who find a patch never leave (0 departures over 20 seeds; a median 5 of
    10 alive at tick 1000, all on a patch).
  - Both MVT claims are Untestable. Likely reason: rule M and the utility mind compare the values of
    sites, not rates of intake, and hold no estimate of the habitat's average; and with sight only
    along rows and columns, a forager that has emptied a patch often sees no other. The theorem
    moves to Minds 4.
- **An added claim, `mem-open.travel`,** set after the first survey run showed rule M's missing
  travel price. It asks two paired things of `mem-open` under the utility mind with travel 0.5
  (memory as in the preset): the advantage beats rule M's, and rememberers beat the others. The
  detail reports the same switch for `mem-walled` and `mem-truffles`.
- **Measured (the survey, 20 seeds, ticks 200–500 unless named):**
  - `mem-open`: advantage −113 (rememberers 324, others 438); within 10 % in 2 of 20 seeds
    (**Fails**; we expected about 0).
  - `mem-walled` −114 (7 % of rememberers alive at tick 500 against 74 %; under `recall` −40),
    `mem-seasons` −48, `mem-truffles` −82, `mem-catchment` −90: all **Fail**. Rememberers reach a
    patch sooner in `mem-catchment` (median tick 27.5 against 33).
  - `mem-trapline`: +69, in every seed (**Holds**). Memory pays only here.
  - `mem-open.travel` is **Weak**: travel 0.5 raises the advantage in every seed, by a median 119,
    to +7.4, but rememberers are richer in only 10 of 20 seeds. `mem-walled` goes from −114 to −55
    (every seed), `mem-truffles` from −82 to +1.5 (18 of 20).
  - Hornvale's pathology **fails**: `project` is worse than `recall` in every seed, −113 against −34
    on `mem-open` (belief error 2.67 against 2.46) and −82 against −25 on `mem-truffles` (4.99
    against 2.46). `stale_choices` is 0.93 and 0.95 on `mem-open`.
  - Truffles **hold**: rememberers gather 0.0147 a Flump-tick against 0.0065 (ticks 1–500), about
    2.3 times.
  - Traplining **holds**: index median 0.15, below 0.8 in every seed; non-rememberers 0.36 with share
    0.5. The payoff **holds**: 0.050 against 0.011. Gill **fails**: the median revisit interval is 50
    ticks with 5 Flumps and with 20.
  - Forgetting: under `recall` the best span is 25 at growback 0.25 and 10 at 1, shorter in 12 of 20
    seeds (**Weak**). Under `project` the median advantage is highest at span 10 at every rate, so a
    longer best span **fails**.
  - Capacity **fails**: memory for everyone gives 154 against 181 without memory (175 under
    `recall`).
  - The `mem-span-project` sweep: the mean advantage is negative everywhere, from −94 to −174.
  - The `mem-share` sweep (untested): −88, −73, −76, −86 and −98 at shares 0.1, 0.25, 0.5, 0.75
    and 0.9 (sd 46–74).
- **Cost** (µs per Flump-tick, release CLI, 2 000 ticks, seeds 1–5). The machine was loaded, so
  Minds 2's presets were re-timed alongside: `mem-open` 19.7, `mem-truffles` 28.9, `mem-mvt` 14.5,
  `mem-walled` 8.7; `walk-capacity` 4.91 (4.66 in Minds 2), `ifd-fence` 14.74 (7.77), `ii-2-unit`
  1.62 (1.05).
- **Titles** follow the measurements: `mem-open` "Remembering on the open sugarscape: rememberers
  end up poorer"; `mem-walled` "Remembering beyond the wall: rememberers starve"; `mem-mvt` "When to
  leave a patch: foragers who find a rich one never leave".
