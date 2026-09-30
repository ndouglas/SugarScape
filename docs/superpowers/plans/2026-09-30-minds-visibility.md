# Minds visibility pass and arena fix: plan

**Why.** The user found the Minds 5–6 visuals confusing. An audit (2026-09-30) found:

- No drawing bug. Every overlay lands on the cell the engine reports.
- The theft arenas look shifted one cell down and to the right. Each is built as a (k+1)-torus with a wall
  only along row 0 and column 0. The wall closes all four sides on the torus, but on screen it shows only at
  the top and left, and the room sits against the bottom and right edges.
- Much of what drives behavior isn't drawn: other agents' caches, cheater or hoarder, the winter season,
  each agent's caching rule under `mixed`, the lab's schedule, homes and larders, and legends.

**Constraints**
- Existing golden fingerprints stay unedited, except the three theft arenas, whose world changes by design.
  Re-record only those three, native and WASM.
- Published numbers that depend on the arenas are re-measured and updated. That covers the Minds 6 survey
  arena rows, the preset descriptions, the README and the program doc.
- Never write "Flump". American spelling. Titles follow `titles.rs`.
- Every commit ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- Work in `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-visibility` (branch `minds-visibility`).

## Task A: Arenas walled on all four sides

- `presets.rs::theft_arena`: a (k+2) × (k+2) torus with opaque walls along row 0, row k+1, column 0 and
  column k+1. The room is the centered k × k block. Capacity, growback and scaling are unchanged.
- Vision stays 1–k/2, which is within the new cap.
- Update the arena test: the room is centered, walls sit on all four sides, and every agent starts inside the room.
- Re-measure:
  - the arena balance (none against even without theft, 100 seeds per n);
  - the pilferage rate and survival at `find` 0.25 with half cheaters.
- Re-record the three arena goldens, native and in `crates/sugarscape-wasm/tests/web.rs` if pinned.
- Rerun the Minds 6 survey (`survey`, `--only` the minds6 prefix). Judges are unchanged.
- Update every arena number in the preset descriptions, titles, the README Minds 6 section,
  docs/studies/2026-09-27-minds.md and the Minds 6 spec amendments. Note the change and its reason.
  If a verdict changes, report it plainly.

## Task B: Visibility

1. **Color modes** (core `render.rs` and the page's color menu), each with a legend line:
   - `Strategy`: hoarder or cheater. It is the default color mode when theft is on.
   - `Caching rule`: none, even, compensate or plan, one color each. It is the default under `caching.mixed`.
2. **Winter.** A "Winter" badge beside the tick counter when `seasons.mode` is global and it's winter, and a
   shaded band on the time charts over winter ticks.
3. **All caches.**
   - An overlay showing every cache in the world, tinted by owner type (hoarder or cheater) when there are
     cheaters, otherwise one color.
   - A site's Inspect lists the caches buried there, with owner and amount. That needs `SiteView.caches`.
   - The selected agent's caches stay highlighted.
4. **Homes and larders.** In central worlds, draw every agent's home and larder faintly. Hide the memory
   overlay when `memory.prior` is `map`, since it would fill every site.
5. **Lab.**
   - Label compartments K1–K3 and mark the tray sites.
   - A status line: "Day d · morning in K# · food/none", or "Test evening".
   - Highlight the agent whose turn it is on the test evening.
   - Inspect shows the agent's frozen allocation (`lab_allocation`).
6. **Legends.** A legend under the map for the active color mode and overlays: cache diamonds, homes,
   larders, the memory squares and the plan route.
7. **Age.** Where death by age is off, show "Age a", not "a / max".

Tests: the core views (`SiteView.caches`, the new color modes) and the web text and legend builders.
Browser check with screenshots of each Minds 5 and 6 preset. One commit per numbered item, or grouped
sensibly.
