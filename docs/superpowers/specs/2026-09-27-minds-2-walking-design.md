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
