# Minds 2: A* and walking (design)

**Date:** 2026-09-27
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 2. Our own experiment, numbered apart
from the reproductions' milestones.
**Builds on:** the milestone 1–24 specs and Minds 1
(`docs/superpowers/specs/2026-09-27-minds-1-utility-design.md`); all remain binding where not
changed here. In particular: the decision seam (`decision.rule`), the literal-default-plus-named-switch
pattern, the preset titles of `crates/sugarscape-core/src/titles.rs`, and the Minds rule that every
engine must reduce to rule M where it sees and does only what rule M does.

**Sources:**
- P. E. Hart, N. J. Nilsson and B. Raphael, "A Formal Basis for the Heuristic Determination of
  Minimum Cost Paths", *IEEE Trans. Systems Science and Cybernetics* 4(2) (1968), 100–107
  (*not in `papers/`*).
- N. R. Sturtevant, "Benchmarks for Grid-Based Pathfinding", *IEEE Trans. Computational
  Intelligence and AI in Games* 4(2) (2012), 144–148. The paper is open
  (webdocs.cs.ualberta.ca/~nathanst/papers/benchmarks.pdf; copy in `papers/pathfinding/`). The
  benchmark maps and scenarios come from movingai.com/benchmarks under the Open Data Commons
  Attribution License.
- J. M. Epstein and R. Axtell, *Growing Artificial Societies* (1996): Animations II-2, II-5, II-6,
  II-7 and III-12, and rule M.
- W. M. Baum and J. R. Kraft, "Group choice: competition, travel, and the ideal free distribution",
  *JEAB* 69 (1998), 227–245 (*not in `papers/`*). They varied travel between patches and a visual
  barrier separately.

## Goal

Add the atom every later planner needs: an A* engine, verified against a known-optimal oracle and
published benchmarks. Give the Sugarscape a way to walk instead of jump, and walls that block
movement, sight or both. Then measure:

- which of the book's results depend on rule M's jump;
- whether walking brings back the book's waves;
- the ideal free distribution when switching patches truly costs travel. This retests Minds 1's
  failed travel claim under Baum and Kraft's own condition.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited.
  Every existing config, link, session and sweep reads and runs as before. `movement.mode: jump`
  and no walls are the defaults, and they draw nothing new from `World.rng`.
- **The reduction holds exactly:** at vision 1, walking is jumping. Every golden Sugarscape preset
  without combat, with vision forced to 1, has the same fingerprint under `walk` as under `jump`.
- **With no walls, sight and every "is this site free" test are exactly today's,** in the same
  order.
- **A* draws no random numbers** and breaks ties by a stated deterministic rule.
- **One engine path; deterministic; portable.** Native and WASM agree. Path costs are sums of 1
  (4-way) or of 1 and √2 (8-way, benchmarks only); `sqrt` is IEEE-exact.
- **Truthful titles and descriptions.**

## Source summary

- **A\*** (Hart, Nilsson and Raphael): with an admissible heuristic (never overestimating the
  remaining cost), A* finds a minimum-cost path. With a consistent heuristic (h(n) ≤ c(n, n′) +
  h(n′)), no node is expanded twice.
- **The benchmarks** (Sturtevant, 2012): octile grid maps and scenario files. Each scenario gives
  a start, a goal and the optimal path length under 8-way moves, where diagonals cost √2 and may
  not cut corners.
- **The book's rule M:** "Look out as far as vision permits in the four principal lattice
  directions … Move to the nearest unoccupied site of maximum sugar … Collect all the sugar at this
  new position." The move is instantaneous, whatever the distance.
- **The book's waves:**
  - II-6: a block of agents in one corner produces waves that travel diagonally across the
    landscape.
  - III-12: two tribes set out from opposite corners in waves that collide.

  Neither reproduces under rule M's jump (see the `ii-6-waves` and `iii-12-collision`
  descriptions).
- **Baum and Kraft (1998):** "When travel was required to switch patches, undermatching decreased
  slightly … A visual barrier … had no effect."

## Measured in planning

A throwaway hack on rule M took one lattice step toward rule M's target each tick, and waited when
the next site was occupied. There was no A* and there were no walls; 10 seeds; the code was
deleted.

- **Carrying capacity depends on the jump.** On `ii-2-unit`, the mean population over ticks 300–500
  was 230.1 under jump (the book: about 224) and 179.0 under walking. The Gini at tick 500 was 0.392
  against 0.277.
- **Walking doesn't bring back the waves.** In `ii-6-waves` (seed 1), the block spreads onto the
  nearby mountain and settles under both jump and walk, and nothing travels to the far corner. By
  tick 80, 209 Flumps are alive under jump and 182 under walking.
- **The cost baseline** (CLI wall-clock over a whole tick, all rules, divided by Flump-ticks):
  - `ii-2-unit`: 1.09 µs (book) against 1.13 µs (utility mind);
  - `ifd-two-to-one`: 1.47 against 1.45 µs;
  - `ifd-crowding` (vision 10–20): 3.06 µs (book) against 7.44 µs (utility with crowding, whose
    portable `exp` and `ln` run over about 80 candidates).

## Architecture

- **`crates/sugarscape-core/src/minds/astar.rs`:** a generic A* over a small `Graph` trait
  (`neighbors(n) -> impl Iterator<Item = (n, cost)>`, `heuristic(n, goal)`), with an expansion
  limit. It returns the path and its cost, plus the number of expansions for the tests. Nodes are
  `usize` indices.
- **`crates/sugarscape-core/src/minds/grid.rs`:** the grids A* searches.
  - The Sugarscape torus is 4-way. Walls and occupied sites are impassable, except the goal. The
    heuristic is torus Manhattan distance.
  - An octile map is 8-way and flat (not a torus), with no corner cutting. It's used only by the
    benchmark tests. The heuristic is octile distance.
- **`crates/sugarscape-core/src/rules/movement.rs`:**
  - `candidates` takes its sites from a wall-aware sight (see Walls).
  - `act` becomes `choose`, then `arrive`: `arrive` either jumps to the target (the book) or walks
    toward it.
  - The utility mind calls the same `arrive`.
- **`crates/sugarscape-core/src/world.rs`:**
  - a wall bitmap built from `walls`;
  - `World::is_occupied` is true for wall sites (so every rule that asks whether a site is free
    treats walls as taken);
  - `World::sight(pos, vision)` stops each of the four rays at the first opaque wall.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `movement.mode` | `jump` | live | `jump` (rule M: go to the target in one tick) or `walk` (take `speed` steps along an A* path toward it) |
| `movement.speed` | 1 | live | steps per tick under `walk` (1–50) |
| `walls` | `[]` | reset | rectangles `{x, y, width, height, opaque}`; every site in one is a wall. `opaque: true` blocks sight as well as movement (a wall); `false` blocks only movement (a fence) |

**Validation:**
- `speed` is 1–50.
- Each wall lies inside the grid, with width and height ≥ 1.
- `walk` with combat on is an error on `movement.mode`: "rule C jumps; walking combat isn't
  defined".
- Walls may not cover every site.
- `placement` must leave room for the population: the free sites number at least `population`.
- `walls` changes only on reset: it's added to the reset-only paths and to `structural_changes`, as
  maps are.
- `#[serde(default)]` for both, so older configs load as today; `legacy::convert` gets the
  defaults.

## Walls

- A wall site has capacity 0 for every good and never grows back.
- It never holds a Flump: `is_occupied` is true, so placement, rule M, children, replacement and
  every "empty neighbor" test skip it.
- **Sight:** each of rule M's four rays stops at the first opaque wall. A fence doesn't stop sight,
  but a Flump can't stand on it. With no walls, `World::sight` returns exactly `torus.sight`'s list.
- **Pollution** neither lands on walls nor diffuses into them. Diffusion averages over a site's
  non-wall neighbors, and with no walls it's today's arithmetic.
- **Rule C** (combat) uses the same wall-aware sight.
- **Placement** draws from non-wall sites. With no walls the draw is today's.

## Walking

Each tick, under `movement.mode: walk`:

1. The decision rule (rule M or the utility mind) picks a target from the candidates, unchanged.
2. If the target is the Flump's own site, it stays and gathers there, as today.
3. Otherwise A* searches the 4-way torus from the Flump to the target. Walls and occupied sites
   are impassable, except the target. Each step costs 1, and the heuristic is torus Manhattan
   distance, which is admissible and consistent.
4. The Flump moves `speed` steps along the path (stopping at the target) and gathers every good at
   the site where it stops. It gathers nothing on the sites it passes.
5. If there's no path within the expansion limit (4 096 sites), it stays and gathers where it is.

**Stated choices:**
- **Plan every tick.** The Flump plans again every tick, as rule M decides again every tick.
  Committing to a target is planning, and it belongs with memory and GOAP (Minds 3–4).
- **Ties:** lowest f, then lowest h, then the earliest pushed. Neighbors are pushed in the order
  north, south, east, west (the geometry's direction order).
- **Travel distance:** the utility mind's `travel` consideration still uses the sight distance,
  not the path length.
- **Neighbors:** `social.moved` records neighbors at the stopping site, as today.

**The reduction:** at vision 1 the target is the Flump's own site or an adjacent one, which is
reached in one step. So walking is jumping, with no extra draws.

## Inspect

Under `walk`, each Flump keeps its last target and the remaining path. This is observational, like
`social`: it's never hashed, exported or shared. Inspect shows a row "Heading to (x, y), n steps
left" (or "Staying"). The grid draws the inspected Flump's planned path as a line in the trail
style (Milestone 6's trails). Under `jump`, Inspect shows "Moved to (x, y)".

## Statistics

No new series. The findings use `population`, `gini`, the patch series (Minds 1) and position
measures computed in the survey.

## Presets

Titles are drafts, to be replaced by measured wording.

| Preset | Title | Setup |
|---|---|---|
| `walk-capacity` | Flumps who walk instead of jump: fewer of them survive | `ii-2-unit` with `walk` |
| `walk-wealth` | Walking changes who gets rich | `ii-5-wealth` with `walk` |
| `walk-seasons` | Walking through the seasons | `ii-7-seasons` with `walk` |
| `walk-waves` | Walking doesn't bring back the book's waves | `ii-6-waves` with `walk` |
| `walk-fast` | Flumps who walk three steps a tick | `ii-2-unit` with `walk`, speed 3 |
| `ifd-fence` | A fence between the patches, with one gap | Minds 1's 2.10 : 1 world (vision 10–20), `walk`, a fence (not opaque) between the patches with a gap at the center row |
| `ifd-fence-far` | The gap moves to the far end: switching costs a long walk | the same, with the gap 15 rows from the center |
| `ifd-wall` | A wall between the patches: the other patch is out of sight too | the same as `ifd-fence`, opaque |

The fence runs from top to bottom at x = 28, one site wide, with a two-site gap. A second fence at
x = 2 closes the wrap-around route. Both use the same gap row.

## Sweeps

Twenty seeds.
- `walk-speed`: `ii-2-unit` population (window mean over ticks 300–500) against speed 1, 2, 3, 4,
  6, 10, with series `walk` and `jump`.
- `walk-vision`: `ii-2-unit` population against vision range 1–1, 1–3, 1–6, 1–10, with series
  `walk` and `jump`.
- `ifd-detour`: `first_patch_share` (ticks 500–1000) against the gap's offset from the center row
  (0, 5, 10, 15), with series `fence` and `wall`.

## Survey

A `minds2` claims module. The book's claims are judged under walking and reported whether they hold
or fail.

- **A\*:** covered by tests, not claims (see Testing).
- **The book's II-2 under walking:** the population stabilizes near 224 (Animation II-2).
  Expected to fail; the probe measured 179.
- **Jump and walk differ:** the carrying capacity under walk is lower than under jump.
- **The book's II-5 under walking:** the wealth distribution is skewed (skewness > 0), with the
  Gini reported.
- **The book's II-7 under walking:** Flumps migrate between hemispheres with the seasons, measured
  as in `ch2`'s seasons claim.
- **Waves (II-6):** a wave reaches the far (northeast) mountain. Measured as the share of Flumps
  within the northeast mountain's radius at tick 100, and expected to fail. It's recorded as a
  negative: walking isn't the missing mechanism.
- **Speed:** the capacity under walk rises toward the jump's as speed rises (`walk-speed`).
- **Baum and Kraft, travel:** requiring travel to switch patches reduces undermatching. s with the
  far gap exceeds s in the same world under `walk` with no fence (`ifd-fence-far` against
  `ifd-far-sighted` switched to `walk`). The `ifd-detour` sweep shows s against the gap's offset,
  and the direction is reported either way.
- **Baum and Kraft, visual barrier:** a visual barrier has no effect. s with the opaque wall is
  equivalent to s with the fence at the same gap (`ifd-wall` against `ifd-fence`, paired).

## Page

- **Rules panel:** a **Movement** group with `mode` (Jump (book) / Walk) and `speed`. Its note says
  walls come from presets and that walking plans an A* path every tick.
- **Grid:** walls drawn in a dark neutral color, fences as hatched sites; both are theme tokens in
  light and dark.
- **Inspect:** the heading row, and the path line for the inspected or followed Flump.
- **Types:** `web/src/types.ts` gains `movement?` and `walls?`, and `AgentView` gains the plan.

## Testing

- **Golden and legacy:** existing entries untouched; new entries for the eight presets; titles.
- **Reduction** (`tests/minds.rs`): every Sugarscape preset without combat, with vision forced to
  1, has the same fingerprint under `walk` as under `jump`.
- **A\* (unit and property):**
  - its cost equals Dijkstra's on 2 000 random grids (4-way torus and 8-way octile, wall densities
    0–40 %, seeded);
  - no node is expanded twice with the consistent heuristics;
  - ties follow the stated order;
  - the expansion limit returns "no path";
  - start = goal;
  - an unreachable goal.
- **Benchmarks** (`tests/fixtures/movingai/`): one random map and one maze map from the synthetic
  sets, with a subset of their scenarios (every 10th). A*'s octile cost equals each scenario's
  optimal length within 1e-6. A README gives the ODC-By attribution.
- **Walls:**
  - sight stops at opaque walls and passes fences;
  - with no walls, `World::sight` equals `torus.sight`;
  - walls hold no sugar and no Flump;
  - placement, children and replacement avoid walls;
  - diffusion skips walls;
  - rule C's sight stops at walls;
  - validation.
- **Walking:**
  - one step toward the target;
  - a detour around an occupant;
  - speed 3 stops at the target;
  - gathering only at the stop;
  - staying when there's no path;
  - no draws beyond `choose`;
  - the plan is recorded and not hashed.
- **WASM:** a pinned fingerprint for `ifd-fence`.
- **Web:** the Movement group, walls and fences drawn, the Inspect row and the path line.

## Docs

- **README:** a Minds 2 section.
- **`docs/studies/2026-09-27-minds.md`:** Minds 2's status and results, and the cost table (book,
  utility, walk; per Flump per tick).
- **Roadmap:** the Minds line.
- **The spec's amendments.**

## Amendments (implementation planning)

These change or extend the sections above. The measured values are the survey's (20 seeds) and
the sweeps' (20 seeds), with the source named. The cost figures are the release CLI's.

- **Wall and fence colors are fixed, not theme tokens.** The core render is always dark, so walls
  are stone `[0x5a, 0x55, 0x4c]` and fences wood `[0x8a, 0x6d, 0x3b]` (`render::WALL`,
  `render::FENCE`). This replaces the Page section's "theme tokens in light and dark".
- **Validation checks free sites per placement rectangle,** not only across the grid: a Block, and
  each of the two Tribes corners, must hold its own population on its non-wall sites. Combat skips
  wall sites explicitly (a fence doesn't stop rule C's sight, so its sites are excluded as
  targets), and `move_agent` debug-asserts that its destination isn't a wall.
- **The plan and Inspect, as built.**
  - `Agent.plan` is observational: never hashed, exported or shared.
  - `AgentView.plan` is `None` until the agent's first move. Its path is empty under jump, or once
    the agent has arrived.
  - Inspect shows "Heading: (x, y), n step(s) left" (singular at 1), "Staying", or under jump
    "Moved to (x, y)".
  - The inspected agent's planned path is drawn as a dashed accent line, split into separate
    segments where it crosses the torus seam.
- **The walking reduction test sets jump explicitly on both sides.** The `walk-*` presets start in
  walk mode, so without that the test would have compared walk against walk for them. The utility
  reduction skips presets whose rule is already the utility mind, as in Minds 1.
- **The benchmark subset** keeps every 20th scenario of `random512-10-0` (89 checked) and every
  100th of `maze512-4-0` (106 checked), not every 10th of each.
- **A `walk-vision` sweep** was added beside `walk-speed` and `ifd-detour` (the capacity against
  vision 1, 1–3, 1–6, 1–10, walking and jumping).
- **Three survey claims are judged on paired per-seed differences,** because the same seeds run in
  both arms: `walk-capacity.lower` (jump − walk), `walk-speed.recovers` (speed 10 − speed 1) and
  `ifd-fence-far.baum-kraft` (s far gap − s no fence). Each holds when the difference is positive
  in at least 80 % of seeds. The unpaired Mann–Whitney result is in each detail and gives the same
  verdict in all three. The judges were chosen before any run. `walk-speed.recovers`' text follows
  this spec's "rises toward the jump's"; the judge is speed 10 above speed 1.
- **The waves measure** is "the share of Flumps farther than 25 (torus distance) from the starting
  block's center at tick 100", not "within the northeast mountain's radius". The block is
  `ii-6-waves`' 20 × 20 placement at (0, 30), center (9.5, 39.5).
- **Measured (the survey, 20 seeds):**
  - II-2 under walking **fails**, as expected: mean population over ticks 300–500, median 181
    (IQR 176–190) against 228 jumping; no seed within 214–234. Walking is lower in 20 of 20 seeds,
    by a median 47. The planning probe's 179 and 230 are replaced.
  - II-5 under walking **holds**: skewness median 1.26 (1.27 jumping), Gini 0.46 (0.48).
  - II-7 under walking **holds**: a median 55 % of Flumps alive over ticks 100–300 change
    hemisphere at least twice, against 83 % jumping.
  - The waves **fail**, as expected: median 0.6 % beyond 25 sites at tick 100 (0.8 % jumping);
    0 of 20 seeds reach a quarter.
  - Speed **holds**: speed 10 (median 227.4) beats speed 1 in 20 of 20 seeds; speed 3 gives 214.0;
    jumping 228.3.
  - Baum and Kraft's travel claim **fails in the opposite direction**: s median 0.85 with the far
    gap against 0.90 walking with no fence (0.90 jumping), lower in 18 of 20 seeds.
  - The visual barrier is **Weak**: s median 0.91 with the wall against 0.88 with the fence at the
    same gap; 10 of 20 seeds within 0.05.
- **The ratio and s point different ways.** At the presets' own 2.10 : 1 ratio, the median mean
  N₁/N₂ over ticks 500–1000 is 1.96 with the far fence, 1.99 with the near fence, 1.99 with the
  wall, 1.97 walking with no fence and 1.88 jumping. So the fenced worlds put slightly more Flumps
  on the richer patch at that ratio, yet s fitted across the five patch sizes is lower with the far
  gap (0.846 against 0.899). s measures how the split tracks the input across sizes; the ratio is
  a single point on it. The cause of the difference isn't measured.
- **Sweeps (20 seeds, means):**
  - `walk-speed`: walking 181.5, 201.4, 212.8, 218.4, 225.2, 227.5 at speeds 1, 2, 3, 4, 6, 10;
    jumping 228.5 throughout.
  - `walk-vision`: jumping 182.7, 205.5, 228.5, 241.5 at vision 1, 1–3, 1–6, 1–10; walking 182.7,
    188.8, 181.5, 182.2. Walking gains nothing from vision.
  - `ifd-detour`: the richer patch's share at gap offsets 0, 5, 10, 15 is 0.664, 0.665, 0.657,
    0.658 behind a fence and 0.668, 0.657, 0.652, 0.652 behind a wall.
- **Cost** (µs per Flump-tick, whole tick, release CLI, 2 000 ticks, seeds 1–5): `ii-2-unit` jump
  1.05, `walk-capacity` 4.66, `walk-fast` 5.10, `ifd-far-sighted` jump 2.65, `ifd-fence` walk
  7.77. Minds 1's baseline: 1.09 (book) and 1.13 (utility).
- **Titles** follow the measurements: `walk-wealth` "Flumps who walk are born and die, and wealth
  grows as lopsided as when they jump"; `walk-seasons` "Walking through the seasons: Flumps still
  migrate, but fewer of them"; `walk-fast` "Flumps who walk three steps a tick: most of the lost
  population comes back"; `ifd-fence-far` "The gap moves to the far end: a long walk to switch,
  and across patch sizes Flumps stray further from matching the yields".
- **Sources.** The Moving AI fixture subset is committed under ODC-By, with its README
  (`crates/sugarscape-core/tests/fixtures/movingai/README.md`). Sturtevant's paper is saved in
  the gitignored `papers/pathfinding/`. Hart, Nilsson and Raphael (1968) and Baum and Kraft (1998)
  are not in `papers/`.
