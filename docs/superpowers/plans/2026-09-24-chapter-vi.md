# Chapter VI: Indecomposability and the Emergent-Society Views Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reproduce the book's indecomposability demonstration (the same society without and with trade, `vi-2-no-trade` / `vi-3-trade`, plus a presets-menu entry that opens them in Compare) and complete `vi-1-everything`'s views with the neighbor, friends and family networks, Lineage colors, and the age and cultural-tag histograms.

**Architecture:** Each agent carries an observational, fixed-size `Social` record (its neighbor list and up to five friends) that the movement rules fill in right after they move the agent, reading only the occupancy grid and tags, never `World.rng` (crates/sugarscape-core/src/social.rs). `World::neighbor_edges/friend_edges/family_edges/lineage` and `stats::age_histogram/tag_histogram` read it; the renderer gains a `lineage` color mode. The WASM `Sim` exposes the new networks through `networks(kind)` and the histograms as `age_hist`/`tag_hist`; the host adds them to snapshots through `wants` exactly like `wealthHist`; the page adds three overlay checkboxes (drawn by the existing edge renderer, neighbors with a direction marker), a Lineage color mode, two charts (bars, or step outlines in Compare) and a Compare entry in the presets menu.

**Tech Stack:** Rust core (`sugarscape-core`), `wasm-bindgen` (`sugarscape-wasm`), TypeScript + Vite + uPlot + Vitest. No new dependencies.

**Spec:** docs/superpowers/specs/2026-09-24-chapter-vi-design.md (earlier specs in docs/superpowers/specs/ stay binding where not changed, in particular 2026-09-24-worker-simulation-design.md and 2026-09-24-sessions-compare-recording-design.md).

## Global Constraints

- **No simulation change.** Every existing entry in `crates/sugarscape-core/tests/golden.rs` and every legacy fixture stays green and unedited (golden.rs only gains the two new entries in Task 8). The new bookkeeping uses no RNG, never affects behavior, and is excluded from `fingerprint`, configs, exports and share links (like trails).
- **Faithful definitions.** Networks and histograms follow the book's definitions quoted in the tasks, including asymmetric neighbor lists and never-rechecked friends.
- **Quiet when paused.** New data is fetched only while its view is shown and only when stale (7a's rules, ruling PF6): overlays through the engine's own wants (never a reason to refresh), histograms through the Charts panel's distributions provider.
- **Determinism:** the bookkeeping never reads or advances `World.rng` and never changes the agent iteration order. Task 2's invariance test and the golden test pin it.
- **Performance:** the bookkeeping runs on every move, so it allocates nothing (fixed-size arrays) and, with culture off, adds no agent lookup. Max-speed throughput on the 200 × 200 / 2 000-agent perf world may drop by at most ~10 % (Task 1 measures it with the CLI).
- **Copy (verbatim):** checkboxes **Neighbor network**, **Friends network**, **Family network**; color mode **Lineage**; presets-menu entry **Indecomposability — VI-2 vs VI-3 (Compare)** under an optgroup **Compare**; preset ids `vi-2-no-trade`, `vi-3-trade`; chart titles **Age histogram** and **Cultural tags (% zeros by position)**.
- **Names are binding across tasks** (each task's Interfaces block repeats the ones it uses): core `social::{Social, Seen, Lineage, MAX_FRIENDS}`, `Social::{neighbors, friends, moved}`, `Seen::at`, `Agent.social`, `World::{neighbor_edges, friend_edges, family_edges, lineage}`, `render::{ColorMode::Lineage, FOUNDER, FOUNDER_PARENT, BORN, BORN_PARENT}`, `stats::{age_histogram, tag_histogram}`; WASM `Sim::{age_hist, tag_hist}`, `networks("neighbors" | "friends" | "family")`; web `Overlay`, `OVERLAYS`, `noOverlays`, `overlayAvailable`, `AGE_BIN`, `Wants.ageHist/tagHist`, `WorldSnapshot.ageHist/tagHist`, `SimLike.age_hist/tag_hist`, `arrowHead`, `positionBars`, `positionSteps`, `showsAgeHist`, `showsTagHist`, `DistState`, `distributionsDue`, `distributionWants`, `COMPARE_PRESETS`, `ComparePreset`, `comparePresetStates`, `Toolbar.typedSeed`, `Engine.loadPreset(id, seed?)`.
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3`; the commit commands below pass it as a second `-m`. Stage **only** the files named in the task (`git add <paths>`, never `-A`/`.`).
- Rust tasks finish with `cargo fmt --all && cargo clippy --all-targets -- -D warnings` before committing.
- Web tasks run `(cd web && npm run build && npm test)`. The build regenerates `web/src/wasm-pkg` (gitignored) with `wasm-pack` and runs `tsc --noEmit` then `vite build`; Vitest imports that package (determinism.test.ts), so always build before testing. Single test files run with `(cd web && npx vitest run src/<file>.test.ts)`.
- **TypeScript:** `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`. Unused imports fail the build — each task lists its import changes. Never pass a possibly-null child to `replaceChildren`/`append`: use `h()`, which skips `null`/`false`.
- **Browser checks are the controller's**, not the implementer's. Each task's "Browser (controller):" line lists the scenarios it affects; the controller runs its puppeteer pass on them (and the full list in Task 10). `?debug` exposes `window.sugarscape = { engine, compare }`.

## Why this task order

- **Core first (Tasks 1–3):** the bookkeeping inside the movement rules with its unit tests and the performance measurement (Task 1), the edge and lineage queries, the Lineage render mode and the fingerprint-invariance test (Task 2), then the two histograms (Task 3). Each is testable in Rust alone.
- **WASM (Task 4)** exposes them; **host and protocol (Task 5)** carry them in snapshots; **the page (Task 6)** draws the overlays and Lineage colors; **charts (Task 7)** draw the histograms, in Compare too.
- **Presets (Task 8)** come after the views so the measured calibration, golden entries and book-style test land together, and before **the Compare entry (Task 9)**, which needs them in `presets_json`.
- **Docs and full verification (Task 10).**

## Decisions (where the spec leaves room)

These are binding; each is repeated in the task that implements it.

1. **Where the bookkeeping lives.** A `Social` value on every `Agent` (`pub social: Social`, `Copy`, about 130 bytes): `neighbors: [AgentId; 4]` + count, `friends: [(AgentId, u32); 5]` + count. New agents (`Agent::random`, births, the test kit) start with `Social::default()` (no neighbors, no friends: "When an agent is born it has no friends"). `fingerprint` hashes fields one by one and `export.rs` writes columns one by one, so neither sees it; configs and share links never contained agents.
2. **When a list is recorded.** Right after the agent's movement rule moves it — M (both the one-good and the n-good variant) or C when combat replaces M — whether or not its site changed, before metabolism (Chapter II: "The first agent now executes M, moves to a new site, and then builds a list of its von Neumann neighbors, which it maintains until its next move"). The movers already hold a copy of the agent's traits from their first lookup and write the agent back at the end, so they carry `social` through (`Social::moved`) and the neighbor ids come from the occupancy grid (`Seen::at`): with culture off there is no extra agent lookup at all. An agent that then starves keeps nothing (it is gone). At t = 0 nobody has moved, so the neighbor network is empty until the first tick.
3. **Neighbor order and uniqueness.** North, south, east, west (`Torus::neighbors`). Grids are at least 5 × 5 (config validation), so the four sites are distinct and never the agent's own: no de-duplication.
4. **The friend rule** (Chapter III and notes 25–28). Each neighbor on the new list is met in that order. Already a friend: nothing happens (the stored distance is never rechecked, note 28). Fewer than five friends: it is added. Otherwise it replaces the friend with the largest stored distance if strictly closer; among equally far friends the earliest-added is replaced; ties keep existing friends. The newcomer joins the end, so the list stays in order of addition. The distance is the Hamming distance between the mover's tags (after its move, before K runs this turn) and the neighbor's tags at that moment. A friend who dies is dropped: lazily, when the list is full and a new neighbor is met (the only time a free slot matters), and `friend_edges` skips the dead — the same lists as dropping friends the moment they die, without scanning every agent on each death.
5. **Culture toggled live.** Friends are maintained only while `culture.enabled` is true at the moment of the move. With culture off, a moving agent's friends are cleared and nothing is compared; `World::friend_edges()` is empty whenever culture is off. Turning culture on again (live, or by a schedule entry) starts from whatever lists survive: empty for every agent that moved while culture was off; a toggle off and on again while paused keeps them. Neighbor lists are recorded whatever the rules.
6. **Edges.** `(from, to)` position pairs over living agents only, `from` in agent-id order: neighbors agent → each agent on its list (an edge follows the neighbor to its current site until the lister moves again; asymmetric by design, note 29); friends agent → friend; family parent → child (both living). Nothing is de-duplicated (the network is directed).
7. **Lineage.** Founder = no parents (the initial population and any agent placed by the Place tool or by replacement); parent = has had a child (`children` non-empty; a dead child still counts, as in the book's coloring). Colors: founder non-parent `#5a5a5a` (the book's black; the grid background is always the dark `BACKGROUND`, so it is always drawn dark grey), founder parent `#ff4d4d`, born non-parent `#3dd66b`, born parent `#ffe04d` (the red/green/yellow of the credit view). The Lineage mode is available in every world (without sex everyone is a grey founder) and is never clamped.
8. **Age histogram.** `stats::age_histogram(world, bin)`: `(max_age.max + 1) / bin + 1` bins of width `bin` from 0, where `max_age.max` is the config's largest maximum lifetime. An agent dies at its first turn with age > its maximum, so it lives through the end-of-tick aging that makes it one older: the last bin reaches `max_age.max + 1`, and any older agent (a lowered maximum) is counted there too. The host asks for 5-tick bins (`AGE_BIN = 5`); WASM returns `[bin, count₀, …]`, the wealth histogram's shape, so the existing `barsData`/`histTable` draw it. Nobody alive: all zeros.
9. **Tag histogram.** `stats::tag_histogram(world)`: for tag positions 0…L−1 (bit i is position i, as `Tags::to_bit_string` prints) the percentage of living agents with a 0 there; all zeros with nobody alive. Drawn at positions 1…L (the book numbers them from 1), y from 0 to 100.
10. **Overlays on the wire.** `Overlay = 'trade' | 'credit' | 'disease' | 'neighbors' | 'friends' | 'family'`, `OVERLAYS` in that order, and `noOverlays()` builds the all-off record (the three `Record<Overlay, boolean>` literals in tests and the two in the host and engine use it). `overlayAvailable(kind, config)`: disease needs disease, friends needs culture, family needs sex. `clampDisplay` turns an unavailable overlay off (as it already does for disease) and the display hides its checkbox. Networks travel as today: `wants.networks` from the engine's own wants (never a refresh trigger while paused), `Uint32Array` quadruples.
11. **Drawing.** Neighbors: `--muted`, 1 px, alpha 0.7, with a filled arrowhead (4 px long, 4 px wide) whose tip stops half a cell short of the target's centre (outside its cell), drawn on the last segment `wrappedSegments` returns — the one that ends at the target — so a wrapped edge's marker is on the right side. Friends `--c1`, family `--accent`, 1.5 px, alpha 0.8, no marker. Edges crossing the torus edge are split exactly as the existing networks are. In Compare each grid draws its own world's networks (the display is already mirrored to B and B's host clamps it for B's rules).
12. **Charts.** Two charts join the top section after **Wealth distribution**: **Age histogram** (`kind: 'age'`, shown while `lifespan.enabled`, x "Age") and **Cultural tags (% zeros by position)** (`kind: 'tags'`, shown while `culture.enabled`, x "Tag position", y 0–100). One world: bars; Compare: one step outline per world (A solid, B dashed), exactly like the wealth histogram. They are distributions: fetched with the Lorenz curve through the same provider rule, now the pure `distributionsDue(dist, tick, now, 250)` + `distributionWants(config)` in `series-data.ts` (testable without uPlot); a histogram the world stopped sending (lifetimes or culture turned off) is cleared, not kept.
13. **VI-2 / VI-3 setup.** Both presets are built by one helper, `indecomposability(c, trade)`: 500 agents, Chapter III demography (`demography`: sex and lifespan on, lifetimes 60–100, fertility at its defaults 12–15 / 40–50 / 50–60), spice on Chapter IV's mirrored map, the calibrated vision, metabolism and endowment (equal for both goods), and `trade.enabled = trade` — nothing else differs (a unit test pins it). Vision and metabolism ranges are "not fixed by the text" like endowments and fertility ages (the book names only the landscape, the population, M and S), so the calibration may choose them; any choice applies to both presets.
14. **Calibration** (Task 8). `measure_indecomposability` runs eight candidates (vision, metabolism, endowment), in a fixed order that starts from Chapter IV's own traits, on seeds 1–5 × up to 1 000 ticks, printing per run: the tick VI-2/VI-3 reached population 0 (if any), the population at t = 1000, the peak population ÷ 500, the number of population peaks and the widest spacing between consecutive peaks (`population_peaks`: the 21-tick centered moving average's local maxima that are the maximum within ±40 ticks, t ≥ 50). **Choose the first candidate in the list for which all five VI-2 runs reach 0 by t = 1000 and all five VI-3 runs are alive at t = 1000 with at least two peaks.** Planning measured the whole list (numbers in Task 8): only candidate 7 (vision 1–5, metabolism 1–2, endowment 65 for both goods) qualifies, and its neighbours at endowment 64 and 66 do not, so the separation is a knife edge; the preset comment and description say so. If a re-run prints different numbers, apply the same rule to them; if no candidate qualifies, stop and report BLOCKED with the table (the controller decides; do not try other candidates).
15. **Book-style thresholds** come from the chosen candidate's measurements by fixed rules: `VI2_EXTINCT_BY` = the latest VI-2 extinction tick rounded up to a multiple of 50; `VI3_PEAK_FACTOR` = the smallest VI-3 peak ÷ 500 rounded down to a multiple of 0.05; `VI3_WIDEST_SPACING` = the smallest VI-3 widest spacing rounded down to a multiple of 10. With the planning numbers: 900, 1.25, 290. The book's "more than twice the initial population", "minima … at 700 agents" and "roughly 115 years" are not reproduced by these rules; per the spec the recorded measurements stand, and the descriptions say so.
16. **The Compare entry.** A web-side table `COMPARE_PRESETS` (`id`, `label`, preset ids `a` and `b`) feeds an optgroup **Compare** at the end of the playground's (A's) preset select, option values `compare:<id>`; B's Rules panel (Compare's "Rules for: B") has no Compare entries. Choosing it: the select snaps back to the current preset; the seed is the seed box's typed value (`Toolbar.typedSeed()`, as Reset reads it); if Compare is on it is left keeping A first (as opening a session file does); then, `busy` and held, A rebuilds as `vi-2-no-trade` at that seed (`engine.loadPreset(a, seed)`, so A's preset badge shows it), the address-bar hash is cleared, and B is built from `vi-3-trade` at the same seed through the existing `buildCompare(state)` path — both at t = 0, settled by the lockstep coordinator, exactly as a `#c=` link opens. Failures show a notice.
17. **VI-1's eighteen views.** `vi-1-everything`'s description gains where each view lives. Two of the book's views have close stand-ins rather than exact matches, and the description is worded to cover them: view 4 (spice wealth histogram) is the Mean holdings chart's spice line, and view 5 (Lorenz curve and Gini "for total wealth") uses sugar wealth, as the Lorenz chart always has. Its config is unchanged (golden entry unchanged).
18. **Performance measurement.** Before any change, build the CLI and keep the binary; afterwards run both on two JSON configs of the perf world (200 × 200, 2 000 agents, a noise sugar map; culture on and off), seven times each, compare the median `user` seconds: each must be ≤ 1.10 × the baseline, and the fingerprints must match (a behavior check). Planning measured +7 % (culture on, 1.27 → 1.36 s) and +5 % (off, 1.17 → 1.23 s); a first version that looked agents up again after the move cost +11 %, which is why the movers carry `social`.

## File Structure

```
crates/sugarscape-core/src/social.rs         NEW  Social, Seen, the friend rule (1); Lineage, World::{neighbor,friend,family}_edges, lineage (2)
crates/sugarscape-core/src/agent.rs          MOD  Agent.social (1)
crates/sugarscape-core/src/lib.rs            MOD  pub mod social (1)
crates/sugarscape-core/src/rules/movement.rs MOD  record after M (1)
crates/sugarscape-core/src/rules/combat.rs   MOD  record after C (1)
crates/sugarscape-core/src/rules/sex.rs      MOD  newborns' Social::default() (1)
crates/sugarscape-core/src/testkit.rs        MOD  spawn's Social::default() (1)
crates/sugarscape-core/src/render.rs         MOD  ColorMode::Lineage and its colors (2)
crates/sugarscape-core/tests/invariants.rs   MOD  observing does not change a run (2)
crates/sugarscape-core/src/stats.rs          MOD  age_histogram, tag_histogram (3)
crates/sugarscape-wasm/src/lib.rs            MOD  networks kinds, age_hist, tag_hist (4)
crates/sugarscape-wasm/tests/web.rs          MOD  (4)
web/src/protocol.ts, protocol.test.ts        MOD  overlays, noOverlays, ageHist/tagHist (5)
web/src/layers.ts, layers.test.ts            MOD  overlayAvailable, clampDisplay (5)
web/src/types.ts                             MOD  ColorMode 'lineage' (5)
web/src/sim-host.ts, sim-host.test.ts        MOD  AGE_BIN, histograms in snapshots (5)
web/src/fake-sim.fixture.ts                  MOD  age_hist, tag_hist, sex/lifespan/culture (5)
web/src/transport.test.ts                    MOD  noOverlays (5)
web/src/engine.ts                            MOD  noOverlays (5); loadPreset seed (9)
web/src/ui/display.ts                        MOD  Lineage mode, three checkboxes (6)
web/src/ui/overlay.ts, overlay.test.ts       MOD  arrowHead (6)
web/src/ui/grid-view.ts                      MOD  overlay styles, direction markers (6)
web/src/determinism.test.ts                  MOD  Chapter VI views change nothing (6)
web/src/ui/series-data.ts, series-data.test.ts MOD positionBars/Steps, distributions (7)
web/src/ui/charts-panel.ts                   MOD  age and tag charts (7)
web/src/engine.test.ts                       MOD  quiet when paused (7); loadPreset seed (9)
crates/sugarscape-core/src/presets.rs        MOD  vi-2-no-trade, vi-3-trade, vi-1 description (8)
crates/sugarscape-core/tests/golden.rs       MOD  two entries (8)
crates/sugarscape-core/tests/book.rs         MOD  measurement and book-style test (8)
web/src/compare-presets.ts, compare-presets.test.ts NEW (9)
web/src/ui/rules-panel.ts                    MOD  Compare optgroup (9)
web/src/ui/toolbar.ts                        MOD  typedSeed (9)
web/src/main.ts                              MOD  openComparePreset (9)
README.md, docs/roadmap.md                   MOD  (10)
```

---

### Task 1: Neighbor lists and friends in the core

*Mechanical (full code); Step 1 and Step 7 are a measurement with a fixed threshold.* Browser (controller): nothing to check (no page change; every world runs exactly as before).

**Files:**
- Create: `crates/sugarscape-core/src/social.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/agent.rs`, `crates/sugarscape-core/src/rules/movement.rs`, `crates/sugarscape-core/src/rules/combat.rs`, `crates/sugarscape-core/src/rules/sex.rs`, `crates/sugarscape-core/src/testkit.rs`

**Interfaces:**
- Consumes: `World::{agent, agent_mut, occupant, move_agent, torus}`, `Torus::neighbors` (N, S, E, W), `Tags::bits`, `testkit::{blank_world, spawn}`, `World::remove_agent`.
- Produces:
  - `social.rs`: `pub const MAX_FRIENDS: usize = 5`; `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] pub struct Social` with `pub fn neighbors(&self) -> &[AgentId]` (N, S, E, W order) and `pub fn friends(&self) -> &[(AgentId, u32)]` (id, Hamming distance at meeting; earliest-added first; may include dead friends); `pub(crate) fn moved(&mut self, world: &World, seen: Seen, tags: Tags)`; `pub(crate) struct Seen` with `pub(crate) fn at(world: &World, pos: Pos) -> Seen`.
  - `agent.rs`: `pub social: Social` on `Agent`.

- [ ] **Step 1: Measure the baseline (before any change)**

Write the two perf configs into `target/` (gitignored) and keep today's CLI binary:
```bash
cat > target/perf-culture.json <<'EOF'
{"width":200,"height":200,"population":2000,"culture":{"enabled":true},"goods":[{"name":"sugar","color":"#f2c14e","map":{"kind":"noise","seed":1,"scale":40,"octaves":3,"height":4},"metabolism":{"min":1,"max":4},"endowment":{"min":5,"max":25}}]}
EOF
cat > target/perf-plain.json <<'EOF'
{"width":200,"height":200,"population":2000,"goods":[{"name":"sugar","color":"#f2c14e","map":{"kind":"noise","seed":1,"scale":40,"octaves":3,"height":4},"metabolism":{"min":1,"max":4},"endowment":{"min":5,"max":25}}]}
EOF
cargo build --release -p sugarscape-cli && cp target/release/sugarscape target/sugarscape-before
for cfg in culture plain; do echo $cfg; for i in 1 2 3 4 5 6 7; do /usr/bin/time -p target/sugarscape-before run --config target/perf-$cfg.json --ticks 1000 --fingerprint 2>&1 | grep -v real | tr '\n' ' '; echo; done; done
```
Expected: `0x62cbcc61337ad569` seven times for culture and `0xb5cf720027a99f64` for plain, each with a `user` time (planning: about 1.27 s and 1.17 s on an M-series Mac). Note the two medians.

- [ ] **Step 2: Write the failing tests**

Create `crates/sugarscape-core/src/social.rs` with only the module doc and the tests (the implementation comes in Step 4):
```rust
//! Observational social bookkeeping: Chapter II's neighbor lists (the
//! neighbor connection network, Animation II-5) and Chapter III's friends
//! (Animation III-8). Nothing here reads or advances `World.rng`, changes a
//! rule's behavior, or is hashed, exported or shared (like trails).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    fn tags(w: &mut World, id: AgentId, bits: u64) {
        let len = w.config.tag_length;
        w.agent_mut(id).unwrap().tags = Tags::new(bits, len);
    }

    fn social(w: &World, id: AgentId) -> Social {
        w.agent(id).unwrap().social
    }

    /// What a move of `id` onto its current site records (as M does).
    fn record(w: &mut World, id: AgentId) {
        let a = w.agent(id).unwrap();
        let (mut social, seen) = (a.social, Seen::at(w, a.pos));
        social.moved(w, seen, a.tags);
        w.agent_mut(id).unwrap().social = social;
    }

    #[test]
    fn a_move_records_the_neighbors_in_north_south_east_west_order() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let east = spawn(&mut w, 6, 5);
        let north = spawn(&mut w, 5, 4);
        spawn(&mut w, 6, 6); // diagonal: not a von Neumann neighbor
        record(&mut w, me);
        assert_eq!(social(&w, me).neighbors(), &[north, east]);
        assert!(
            social(&w, north).neighbors().is_empty(),
            "only the mover records"
        );
    }

    #[test]
    fn a_stationary_move_still_replaces_the_list() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let a = spawn(&mut w, 5, 6);
        record(&mut w, me);
        assert_eq!(social(&w, me).neighbors(), &[a]);
        let pos = w.agent(a).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        let b = spawn(&mut w, 4, 5);
        record(&mut w, me);
        assert_eq!(social(&w, me).neighbors(), &[b]);
    }

    #[test]
    fn lists_can_be_asymmetric_and_outlast_the_neighborhood() {
        let mut w = blank_world(10, 10);
        let i = spawn(&mut w, 2, 2);
        let k = spawn(&mut w, 2, 3);
        record(&mut w, i); // i moved next to k: k is on i's list
        assert_eq!(social(&w, i).neighbors(), &[k]);
        assert!(social(&w, k).neighbors().is_empty(), "i is not on k's list");
        // k moves away; i keeps k on its list until i's own next move.
        w.move_agent(k, crate::geometry::Pos::new(7, 7));
        record(&mut w, k);
        assert_eq!(social(&w, i).neighbors(), &[k]);
        assert!(social(&w, k).neighbors().is_empty());
    }

    #[test]
    fn friends_fill_to_five_in_meeting_order_with_their_distances() {
        let mut w = blank_world(10, 10);
        w.config.culture.enabled = true;
        let me = spawn(&mut w, 5, 5);
        let mut met = Vec::new();
        for (x, y, bits) in [(5, 4, 0b1), (5, 6, 0b11), (6, 5, 0b111), (4, 5, 0b1111)] {
            let id = spawn(&mut w, x, y);
            tags(&mut w, id, bits);
            met.push(id);
        }
        record(&mut w, me);
        let expected: Vec<(AgentId, u32)> = met.iter().copied().zip([1, 2, 3, 4]).collect();
        assert_eq!(social(&w, me).friends(), &expected[..]);
        // Meeting the same neighbors again adds nobody.
        record(&mut w, me);
        assert_eq!(social(&w, me).friends().len(), 4);
        // A fifth friend fills the list.
        let pos = w.agent(met[0]).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        let fifth = spawn(&mut w, 5, 4);
        tags(&mut w, fifth, 0b1_1111);
        record(&mut w, me);
        let f = social(&w, me);
        assert_eq!(f.friends().len(), MAX_FRIENDS);
        assert_eq!(f.friends()[4], (fifth, 5));
    }

    /// `me` at (5, 5) with five friends at distances `ds` (earliest first);
    /// every friend is taken off to column 0 again, so the next move meets
    /// only the agents a test places.
    fn five_friends(ds: [u32; 5]) -> (World, AgentId, Vec<AgentId>) {
        let mut w = blank_world(10, 10);
        w.config.culture.enabled = true;
        let me = spawn(&mut w, 5, 5);
        let mut ids = Vec::new();
        for d in ds {
            let id = spawn(&mut w, 5, 4);
            tags(&mut w, id, (1u64 << d) - 1);
            record(&mut w, me);
            w.move_agent(id, crate::geometry::Pos::new(0, ids.len() as u32));
            ids.push(id);
        }
        let got: Vec<u32> = social(&w, me).friends().iter().map(|f| f.1).collect();
        assert_eq!(got, ds);
        (w, me, ids)
    }

    #[test]
    fn a_strictly_closer_neighbor_replaces_the_farthest_friend() {
        let (mut w, me, ids) = five_friends([3, 7, 2, 7, 5]);
        let meet = |w: &mut World, ones: u32| {
            let id = spawn(w, 6, 5);
            tags(w, id, (1u64 << ones) - 1);
            record(w, me);
            w.remove_agent(6, 5).unwrap();
            id
        };
        let far = meet(&mut w, 8);
        assert!(
            !social(&w, me).friends().iter().any(|f| f.0 == far),
            "farther than every friend"
        );
        let tie = meet(&mut w, 7);
        assert!(
            !social(&w, me).friends().iter().any(|f| f.0 == tie),
            "ties keep existing friends"
        );
        let closer = meet(&mut w, 1);
        assert_eq!(
            social(&w, me).friends(),
            &[
                (ids[0], 3),
                (ids[2], 2),
                (ids[3], 7),
                (ids[4], 5),
                (closer, 1)
            ],
            "the earlier of the two farthest (7) is replaced; the newcomer joins the end"
        );
    }

    #[test]
    fn friends_are_never_rechecked_after_tags_change() {
        let (mut w, me, ids) = five_friends([1, 1, 1, 1, 1]);
        tags(&mut w, ids[0], u64::MAX); // now far away culturally
        let newcomer = spawn(&mut w, 6, 5);
        tags(&mut w, newcomer, 0b1);
        record(&mut w, me);
        assert_eq!(
            social(&w, me).friends()[0],
            (ids[0], 1),
            "the stored distance stands"
        );
        assert!(!social(&w, me).friends().iter().any(|f| f.0 == newcomer));
    }

    #[test]
    fn a_dead_friend_frees_its_slot() {
        let (mut w, me, ids) = five_friends([1, 2, 3, 4, 5]);
        let pos = w.agent(ids[1]).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        let newcomer = spawn(&mut w, 6, 5);
        tags(&mut w, newcomer, 0b111_1111); // 7: farther than every friend
        record(&mut w, me);
        let got: Vec<AgentId> = social(&w, me).friends().iter().map(|f| f.0).collect();
        assert_eq!(got, vec![ids[0], ids[2], ids[3], ids[4], newcomer]);
    }

    #[test]
    fn with_culture_off_friends_are_empty_and_not_kept() {
        let mut w = blank_world(10, 10);
        let me = spawn(&mut w, 5, 5);
        let n = spawn(&mut w, 5, 6);
        record(&mut w, me);
        assert_eq!(
            social(&w, me).neighbors(),
            &[n],
            "neighbors are always recorded"
        );
        assert!(social(&w, me).friends().is_empty());
        w.config.culture.enabled = true;
        record(&mut w, me);
        assert_eq!(social(&w, me).friends().len(), 1);
        w.config.culture.enabled = false;
        record(&mut w, me);
        assert!(
            social(&w, me).friends().is_empty(),
            "turning culture off clears at the next move"
        );
    }

    #[test]
    fn every_move_records_during_a_tick() {
        let mut w = blank_world(10, 10);
        let a = spawn(&mut w, 5, 5);
        let b = spawn(&mut w, 5, 6);
        w.step();
        // Vision 1 on a zero landscape: both stay (distance 0 wins), and each
        // records the other after its own move.
        assert_eq!(social(&w, a).neighbors(), &[b]);
        assert_eq!(social(&w, b).neighbors(), &[a]);
    }
}
```
In `crates/sugarscape-core/src/lib.rs`, after `pub mod rules;` add:
```rust
pub mod social;
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core --lib social`
Expected: FAIL to compile — `cannot find type Social`, `Seen`, `World`, `Tags`, `AgentId` … in `social.rs`.

- [ ] **Step 4: Implement**

In `social.rs`, between the module doc and `#[cfg(test)]`, insert (Decisions 1–5):
```rust
use crate::agent::{AgentId, Tags};
use crate::geometry::Pos;
use crate::world::World;

/// Most friends an agent keeps (Chapter III: "the five agents it has
/// encountered who are nearest it culturally").
pub const MAX_FRIENDS: usize = 5;

/// An agent's neighbor list and friends. Fixed-size, so recording after
/// every move allocates nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Social {
    neighbors: [AgentId; 4],
    neighbor_count: u8,
    /// (friend, Hamming distance between the two tag strings when they met),
    /// earliest-added first.
    friends: [(AgentId, u32); MAX_FRIENDS],
    friend_count: u8,
}

impl Social {
    /// The agents that were von Neumann neighbors after this agent's last
    /// move, in north, south, east, west order (some may have died since).
    pub fn neighbors(&self) -> &[AgentId] {
        &self.neighbors[..usize::from(self.neighbor_count)]
    }

    /// Friends with the Hamming distance recorded when they met, earliest-added
    /// first (some may have died since the agent last met a new neighbor).
    pub fn friends(&self) -> &[(AgentId, u32)] {
        &self.friends[..usize::from(self.friend_count)]
    }

    /// After a move (rule M, or C when combat replaces it), whether or not
    /// the agent changed site: `seen` replaces the neighbor list. While
    /// culture is on each neighbor on it, in order, is then met under the
    /// friend rule at the Hamming distance between `tags` (the mover's) and
    /// its tags now; with culture off the friends are cleared (they are not
    /// kept while off). Reads `world`; changes nothing there.
    pub(crate) fn moved(&mut self, world: &World, seen: Seen, tags: Tags) {
        let n = usize::from(seen.len);
        self.neighbors[..n].copy_from_slice(&seen.ids[..n]);
        self.neighbor_count = seen.len;
        if !world.config.culture.enabled {
            self.friend_count = 0;
            return;
        }
        let alive = |f: AgentId| world.agent(f).is_some();
        for &other in &seen.ids[..n] {
            // Most moves meet friends again: skip them before looking them up.
            if self.is_friend(other) {
                continue;
            }
            let them = world.agent(other).expect("a neighbor that was just seen");
            self.meet(other, (tags.bits() ^ them.tags.bits()).count_ones(), alive);
        }
    }

    fn is_friend(&self, other: AgentId) -> bool {
        self.friends().iter().any(|&(f, _)| f == other)
    }

    /// Drops friends for whom `alive` is false, keeping the order.
    fn drop_dead(&mut self, alive: impl Fn(AgentId) -> bool) {
        let mut kept = 0;
        for i in 0..usize::from(self.friend_count) {
            if alive(self.friends[i].0) {
                self.friends[kept] = self.friends[i];
                kept += 1;
            }
        }
        self.friend_count = kept as u8;
    }

    /// The friend rule for one neighbor met at `distance`: an existing friend
    /// is left as it is (distances are never rechecked, note 28); with fewer
    /// than five friends the neighbor is added; otherwise it replaces the
    /// friend farthest away if strictly closer (among equally far friends
    /// the earliest-added goes), and joins the end of the list. `alive` is
    /// consulted only when the list is full, to free dead friends' slots
    /// first (the same lists as dropping them when they die).
    fn meet(&mut self, other: AgentId, distance: u32, alive: impl Fn(AgentId) -> bool) {
        if self.is_friend(other) {
            return;
        }
        if usize::from(self.friend_count) == MAX_FRIENDS {
            self.drop_dead(alive);
        }
        let n = usize::from(self.friend_count);
        if n < MAX_FRIENDS {
            self.friends[n] = (other, distance);
            self.friend_count += 1;
            return;
        }
        let mut far = 0;
        for i in 1..n {
            if self.friends[i].1 > self.friends[far].1 {
                far = i;
            }
        }
        if distance < self.friends[far].1 {
            self.friends.copy_within(far + 1..n, far);
            self.friends[n - 1] = (other, distance);
        }
    }
}

/// The agents in a site's von Neumann neighborhood, in north, south, east,
/// west order: what an agent records right after its move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Seen {
    ids: [AgentId; 4],
    len: u8,
}

impl Seen {
    /// Reads only the occupancy grid (no agent lookups). Grids are at least
    /// 5 × 5, so the four sites are distinct and never `pos` itself.
    pub(crate) fn at(world: &World, pos: Pos) -> Self {
        let mut seen = Self::default();
        for q in world.torus.neighbors(pos) {
            if let Some(other) = world.occupant(q) {
                seen.ids[usize::from(seen.len)] = other;
                seen.len += 1;
            }
        }
        seen
    }
}
```

In `crates/sugarscape-core/src/agent.rs`: after `use crate::geometry::Pos;` add `use crate::social::Social;`; in `struct Agent`, after the `infected_by` field add
```rust
    /// Neighbor list and friends (observation only: never hashed, exported or
    /// shared; see `social`).
    pub social: Social,
```
and in `Agent::random`'s struct literal, after `infected_by: None,` add `social: Social::default(),`.

In `crates/sugarscape-core/src/rules/sex.rs` (`birth`'s `child` literal) and `crates/sugarscape-core/src/testkit.rs` (`spawn`'s `agent` literal): after `infected_by: None,` add `social: Social::default(),`, and after `use crate::geometry::Pos;` add `use crate::social::Social;`.

In `crates/sugarscape-core/src/rules/movement.rs` (Decision 2): after `use crate::rules::Harvest;` add `use crate::social::Seen;`. In `act`, replace
```rust
    let (pos, vision) = (agent.pos, agent.vision);
```
with
```rust
    let (pos, vision) = (agent.pos, agent.vision);
    let (tags, mut social) = (agent.tags, agent.social);
```
and replace
```rust
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let gathered = site.resource[0];
    site.resource[0] = 0.0;
    world.agent_mut(id).expect("live agent").holdings[0] += gathered;
    Harvest::of(&[gathered])
```
with
```rust
    world.move_agent(id, target);
    social.moved(world, Seen::at(world, target), tags);
    let site = world.site_mut(target);
    let gathered = site.resource[0];
    site.resource[0] = 0.0;
    let a = world.agent_mut(id).expect("live agent");
    a.holdings[0] += gathered;
    a.social = social;
    Harvest::of(&[gathered])
```
In `act_goods`, after `let (pos, vision, phi, held) = (a.pos, a.vision, a.foresight, a.holdings);` add `let (tags, mut social) = (a.tags, a.social);`; after its `world.move_agent(id, target);` add `social.moved(world, Seen::at(world, target), tags);`; and just before its final `harvest` (after the `for (have, got) in a.holdings…` loop) add `a.social = social;`.

In `crates/sugarscape-core/src/rules/combat.rs`: after `use crate::rules::{movement::choose, Harvest};` add `use crate::social::Seen;`; after `let (pos, vision, wealth) = (me.pos, me.vision, me.holdings[0]);` add `let (tags, mut social) = (me.tags, me.social);`; replace
```rust
    world.move_agent(id, target);
    let site = world.site_mut(target);
    let gathered = site.resource[0];
    site.resource[0] = 0.0;
    world.agent_mut(id).expect("live agent").holdings[0] += gathered + loot;
```
with
```rust
    world.move_agent(id, target);
    social.moved(world, Seen::at(world, target), tags);
    let site = world.site_mut(target);
    let gathered = site.resource[0];
    site.resource[0] = 0.0;
    let a = world.agent_mut(id).expect("live agent");
    a.holdings[0] += gathered + loot;
    a.social = social;
```
(`me`'s borrow ends before `world.kill`/`move_agent`: `tags` and `social` are copies.)

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core --lib social`
Expected: PASS, 9 tests.

- [ ] **Step 6: Nothing else moved**

Run: `cargo test -p sugarscape-core`
Expected: PASS, including `golden` (`earlier_presets_are_unchanged`: every fingerprint unchanged — the bookkeeping neither draws from the RNG nor changes behavior) and `legacy`.

- [ ] **Step 7: Measure again** (Decision 18)

```bash
cargo build --release -p sugarscape-cli
for cfg in culture plain; do for bin in target/sugarscape-before target/release/sugarscape; do echo "$cfg $bin"; for i in 1 2 3 4 5 6 7; do /usr/bin/time -p $bin run --config target/perf-$cfg.json --ticks 1000 --fingerprint 2>&1 | grep -v real | tr '\n' ' '; echo; done; done; done
```
Expected: the same fingerprints as Step 1 for both binaries (`0x62cbcc61337ad569`, `0xb5cf720027a99f64`), and each new median `user` time ≤ 1.10 × its baseline median (planning: culture 1.27 → 1.36 s, plain 1.17 → 1.23 s). If a ratio is above 1.10, re-run both binaries back to back once more (machine noise is a few percent); if it is still above, finish the task and report DONE_WITH_CONCERNS with both tables — do not restructure without the controller. Put the four medians in the commit message body.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/social.rs crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/agent.rs crates/sugarscape-core/src/rules/movement.rs crates/sugarscape-core/src/rules/combat.rs crates/sugarscape-core/src/rules/sex.rs crates/sugarscape-core/src/testkit.rs
git commit -m "Record neighbor lists and friends after each move" -m "Perf world, median user seconds over 7 runs (before -> after): culture on <a> -> <b>, culture off <c> -> <d>; fingerprints unchanged." -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```
(Replace `<a>`…`<d>` with the Step 7 medians.)

---

### Task 2: Networks, lineage and the Lineage color mode in the core

*Mechanical (full code).* Browser (controller): nothing to check (no page change).

**Files:**
- Modify: `crates/sugarscape-core/src/social.rs`, `crates/sugarscape-core/src/render.rs`
- Test: `crates/sugarscape-core/tests/invariants.rs`

**Interfaces:**
- Consumes: `Social::{neighbors, friends}`, `Agent.social`, `Agent::{parents, children}`, `World::agents` (id order), `social` test helper `record` (Task 1).
- Produces:
  - `social.rs`: `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Lineage { Founder, FounderParent, Born, BornParent }` with `pub fn of(agent: &Agent) -> Lineage`; on `World`: `pub fn neighbor_edges(&self) -> Vec<(Pos, Pos)>`, `pub fn friend_edges(&self) -> Vec<(Pos, Pos)>` (empty while culture is off), `pub fn family_edges(&self) -> Vec<(Pos, Pos)>`, `pub fn lineage(&self, id: AgentId) -> Option<Lineage>`.
  - `render.rs`: `ColorMode::Lineage` (parsed from `"lineage"`), `pub const FOUNDER: Rgb = [0x5a, 0x5a, 0x5a]`, `FOUNDER_PARENT = [0xff, 0x4d, 0x4d]`, `BORN = [0x3d, 0xd6, 0x6b]`, `BORN_PARENT = [0xff, 0xe0, 0x4d]`.

- [ ] **Step 1: Write the failing tests**

Append inside `social.rs`'s `mod tests` (before its closing brace):
```rust
    #[test]
    fn neighbor_edges_run_from_each_agent_to_the_living_agents_on_its_list() {
        use crate::geometry::Pos;
        let mut w = blank_world(10, 10);
        let i = spawn(&mut w, 2, 2);
        let k = spawn(&mut w, 2, 3);
        spawn(&mut w, 3, 2);
        record(&mut w, i);
        assert_eq!(
            w.neighbor_edges(),
            vec![
                (Pos::new(2, 2), Pos::new(2, 3)),
                (Pos::new(2, 2), Pos::new(3, 2))
            ],
            "directed: the other two have not moved, so they list nobody"
        );
        w.remove_agent(3, 2).unwrap();
        w.move_agent(k, Pos::new(7, 7));
        assert_eq!(
            w.neighbor_edges(),
            vec![(Pos::new(2, 2), Pos::new(7, 7))],
            "the edge follows k to its new site until i moves; the dead are dropped"
        );
    }

    #[test]
    fn friend_edges_need_culture_and_living_friends() {
        use crate::geometry::Pos;
        let mut w = blank_world(10, 10);
        w.config.culture.enabled = true;
        let me = spawn(&mut w, 5, 5);
        spawn(&mut w, 5, 4);
        spawn(&mut w, 5, 6);
        record(&mut w, me);
        assert_eq!(w.friend_edges().len(), 2);
        w.remove_agent(5, 6).unwrap();
        assert_eq!(w.friend_edges(), vec![(Pos::new(5, 5), Pos::new(5, 4))]);
        w.config.culture.enabled = false;
        assert!(w.friend_edges().is_empty(), "none while culture is off");
    }

    #[test]
    fn family_edges_and_lineage_follow_parents_and_children() {
        use crate::geometry::Pos;
        let mut w = blank_world(10, 10);
        let mum = spawn(&mut w, 1, 1);
        let dad = spawn(&mut w, 3, 1);
        let kid = spawn(&mut w, 5, 5);
        let grandkid = spawn(&mut w, 7, 7);
        w.agent_mut(kid).unwrap().parents = Some([mum, dad]);
        w.agent_mut(grandkid).unwrap().parents = Some([kid, dad]);
        w.agent_mut(mum).unwrap().children = vec![kid];
        w.agent_mut(dad).unwrap().children = vec![kid, grandkid];
        w.agent_mut(kid).unwrap().children = vec![grandkid];
        let loner = spawn(&mut w, 9, 9);
        assert_eq!(
            w.family_edges(),
            vec![
                (Pos::new(1, 1), Pos::new(5, 5)),
                (Pos::new(3, 1), Pos::new(5, 5)),
                (Pos::new(3, 1), Pos::new(7, 7)),
                (Pos::new(5, 5), Pos::new(7, 7)),
            ]
        );
        assert_eq!(w.lineage(loner), Some(Lineage::Founder));
        assert_eq!(w.lineage(mum), Some(Lineage::FounderParent));
        assert_eq!(w.lineage(kid), Some(Lineage::BornParent));
        assert_eq!(w.lineage(grandkid), Some(Lineage::Born));
        w.remove_agent(7, 7).unwrap();
        assert_eq!(w.family_edges().len(), 2, "edges to a dead child go");
        assert_eq!(
            w.lineage(kid),
            Some(Lineage::BornParent),
            "a dead child still makes a parent"
        );
        assert_eq!(w.lineage(grandkid), None);
    }
```
Append inside `render.rs`'s `mod tests`:
```rust
    #[test]
    fn lineage_mode_colors_founders_parents_and_children() {
        let mut w = blank_world(10, 10);
        let founder = spawn(&mut w, 1, 1);
        let parent = spawn(&mut w, 2, 2);
        let child = spawn(&mut w, 3, 3);
        let both = spawn(&mut w, 4, 4);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.agent_mut(child).unwrap().parents = Some([parent, founder]);
        w.agent_mut(both).unwrap().parents = Some([parent, founder]);
        w.agent_mut(both).unwrap().children = vec![999];
        let mut buf = Vec::new();
        render(&w, ColorMode::Lineage, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], FOUNDER);
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], FOUNDER_PARENT);
        assert_eq!(pixel(&buf, &w, 3, 3)[..3], BORN);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], BORN_PARENT);
        assert_eq!("lineage".parse::<ColorMode>().unwrap(), ColorMode::Lineage);
    }
```
Append to `crates/sugarscape-core/tests/invariants.rs` (the spec's "a run's fingerprint is identical whether or not the new edge/lineage queries are called"; the histograms join in Task 3):
```rust
#[test]
fn observing_networks_and_lineage_does_not_change_a_run() {
    use sugarscape_core::presets;
    for id in ["vi-1-everything", "iii-6-culture", "iii-14-combat-culture"] {
        let config = presets::by_id(id).unwrap().config;
        let mut watched = World::new(config.clone(), 3).unwrap();
        let mut plain = World::new(config, 3).unwrap();
        for _ in 0..60 {
            watched.step();
            plain.step();
            let edges = watched.neighbor_edges().len()
                + watched.friend_edges().len()
                + watched.family_edges().len();
            let ids: Vec<u64> = watched.agents().map(|a| a.id).collect();
            let classes = ids.iter().filter(|&&a| watched.lineage(a).is_some()).count();
            assert!(edges + classes > 0);
        }
        assert_eq!(watched.fingerprint(), plain.fingerprint(), "{id}");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core --lib social render && cargo test -p sugarscape-core --test invariants`
Expected: FAIL to compile — no method `neighbor_edges`/`friend_edges`/`family_edges`/`lineage` on `World`, no `Lineage`, no `ColorMode::Lineage`/`FOUNDER`….

- [ ] **Step 3: Implement**

In `social.rs` change `use crate::agent::{AgentId, Tags};` to `use crate::agent::{Agent, AgentId, Tags};` and insert before `#[cfg(test)]` (Decisions 6 and 7):
```rust
/// An agent's class in Animation III-5's genealogical view: "The initial
/// population is colored black. When a member of this population has a
/// child, the new parent is colored red, the child green. Agents who are both
/// parents and children are colored yellow." Founders are agents without
/// parents (the initial population, and agents placed or replaced later);
/// a parent has had a child, living or dead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lineage {
    Founder,
    FounderParent,
    Born,
    BornParent,
}

impl Lineage {
    pub fn of(agent: &Agent) -> Self {
        match (agent.parents.is_some(), !agent.children.is_empty()) {
            (false, false) => Self::Founder,
            (false, true) => Self::FounderParent,
            (true, false) => Self::Born,
            (true, true) => Self::BornParent,
        }
    }
}

/// (from, to) positions: an edge from each living agent (in id order) to
/// each living agent `targets` names for it, in that order.
fn directed<'a, I>(world: &'a World, targets: impl Fn(&'a Agent) -> I) -> Vec<(Pos, Pos)>
where
    I: Iterator<Item = AgentId>,
{
    let mut out = Vec::new();
    for a in world.agents() {
        for t in targets(a) {
            if let Some(b) = world.agent(t) {
                out.push((a.pos, b.pos));
            }
        }
    }
    out
}

impl World {
    /// Chapter II's neighbor connection network (Animation II-5): "lines
    /// are drawn from each agent to all agents on its list". Directed, so it
    /// may be asymmetric (note 29); living agents only.
    pub fn neighbor_edges(&self) -> Vec<(Pos, Pos)> {
        directed(self, |a| a.social.neighbors().iter().copied())
    }

    /// Chapter III's network of friends (Animation III-8): an edge from each
    /// agent to each of its living friends; none while culture is off.
    pub fn friend_edges(&self) -> Vec<(Pos, Pos)> {
        if !self.config.culture.enabled {
            return Vec::new();
        }
        directed(self, |a| a.social.friends().iter().map(|&(f, _)| f))
    }

    /// Animation III-5's genealogical network: "a line from every parent to
    /// each of its children", both living.
    pub fn family_edges(&self) -> Vec<(Pos, Pos)> {
        directed(self, |a| a.children.iter().copied())
    }

    /// Agent `id`'s genealogical class, if it is alive.
    pub fn lineage(&self, id: AgentId) -> Option<Lineage> {
        self.agent(id).map(Lineage::of)
    }
}
```
In `render.rs`: after `use crate::network::CreditRole;` add `use crate::social::Lineage;`; after `pub const HEALTHY: …;` add
```rust
/// Animation III-5's lineage colors. The book's founders are black; the grid's
/// background is always dark, so they are drawn dark grey.
pub const FOUNDER: Rgb = [0x5a, 0x5a, 0x5a];
pub const FOUNDER_PARENT: Rgb = [0xff, 0x4d, 0x4d];
pub const BORN: Rgb = [0x3d, 0xd6, 0x6b];
pub const BORN_PARENT: Rgb = [0xff, 0xe0, 0x4d];
```
add `Lineage,` after `Disease,` in `enum ColorMode`; add `"lineage" => Self::Lineage,` after `"disease" => Self::Disease,` in `from_str`; and in `agent_color`'s match, after the `ColorMode::Disease => { … }` arm, add
```rust
        ColorMode::Lineage => match Lineage::of(a) {
            Lineage::Founder => FOUNDER,
            Lineage::FounderParent => FOUNDER_PARENT,
            Lineage::Born => BORN,
            Lineage::BornParent => BORN_PARENT,
        },
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: PASS (the three new `social` tests, the render test, `observing_networks_and_lineage_does_not_change_a_run`, and golden unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/social.rs crates/sugarscape-core/src/render.rs crates/sugarscape-core/tests/invariants.rs
git commit -m "Add neighbor, friend and family networks and the Lineage color mode" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 3: Age and cultural-tag histograms in the core

*Mechanical (full code).* Browser (controller): nothing to check.

**Files:**
- Modify: `crates/sugarscape-core/src/stats.rs`, `crates/sugarscape-core/tests/invariants.rs`

**Interfaces:**
- Consumes: `World::{agents, population, config}`, `Tags::get`.
- Produces: `pub fn age_histogram(world: &World, bin: u32) -> Vec<f64>` (`(max_age.max + 1) / bin + 1` counts; panics on `bin == 0`), `pub fn tag_histogram(world: &World) -> Vec<f64>` (`tag_length` percentages, position 0 first).

- [ ] **Step 1: Write the failing tests**

Append inside `stats.rs`'s `mod tests`:
```rust
    #[test]
    fn age_histogram_bins_ages_up_to_the_largest_maximum_lifetime() {
        use crate::testkit::*;
        let mut w = blank_world(10, 10);
        assert_eq!(
            age_histogram(&w, 5),
            vec![0.0; 21],
            "(100 + 1) / 5 + 1 bins, all empty"
        );
        for (x, age) in [(0, 0), (1, 4), (2, 5), (3, 99), (4, 100), (5, 101), (6, 250)] {
            let id = spawn(&mut w, x, 0);
            w.agent_mut(id).unwrap().age = age;
        }
        let h = age_histogram(&w, 5);
        assert_eq!(h.len(), 21);
        assert_eq!((h[0], h[1], h[19], h[20]), (2.0, 1.0, 1.0, 3.0));
        assert_eq!(h.iter().sum::<f64>(), 7.0);
        w.config.lifespan.max_age = crate::config::URange::new(60, 64);
        assert_eq!(age_histogram(&w, 5).len(), 14, "(64 + 1) / 5 + 1");
    }

    #[test]
    fn tag_histogram_is_the_percentage_of_zeros_at_each_position() {
        use crate::agent::Tags;
        use crate::testkit::*;
        let mut w = blank_world(10, 10);
        assert_eq!(tag_histogram(&w), vec![0.0; 11], "nobody alive");
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        w.agent_mut(a).unwrap().tags = Tags::new(0b000_0000_0011, 11);
        w.agent_mut(b).unwrap().tags = Tags::new(0b100_0000_0001, 11);
        let h = tag_histogram(&w);
        assert_eq!(h.len(), 11);
        assert_eq!((h[0], h[1], h[2], h[10]), (0.0, 50.0, 100.0, 50.0));
    }
```
In `tests/invariants.rs`'s `observing_networks_and_lineage_does_not_change_a_run`, rename it `observing_networks_lineage_and_histograms_does_not_change_a_run`, change `use sugarscape_core::presets;` to `use sugarscape_core::{presets, stats};`, and replace `assert!(edges + classes > 0);` with
```rust
            let hists =
                stats::age_histogram(&watched, 5).len() + stats::tag_histogram(&watched).len();
            assert!(edges + classes + hists > 0);
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core --lib stats`
Expected: FAIL to compile — `cannot find function age_histogram` / `tag_histogram`.

- [ ] **Step 3: Implement**

In `stats.rs`, before the `/// Largest-Triangle-Three-Buckets` doc of `downsample`, add (Decisions 8 and 9):
```rust
/// Animation III-1's age histogram: living agents' ages in `bin`-tick bins
/// from 0. The last bin reaches the largest configured maximum lifetime plus
/// one (an agent dies at its first turn with age > its maximum, so it lives
/// through the end-of-tick aging that makes it one older) and also holds any
/// older agent. With nobody alive every count is 0.
pub fn age_histogram(world: &World, bin: u32) -> Vec<f64> {
    assert!(bin >= 1);
    let bins = ((world.config.lifespan.max_age.max + 1) / bin + 1) as usize;
    let mut counts = vec![0.0; bins];
    for a in world.agents() {
        counts[((a.age / bin) as usize).min(bins - 1)] += 1.0;
    }
    counts
}

/// Animation III-7's cultural tag histogram: "one bin for each tag position.
/// The height of the bin gives the percentage of agents having a 0 at that
/// position" (position 0 first). With nobody alive every bin is 0.
pub fn tag_histogram(world: &World) -> Vec<f64> {
    let mut zeros = vec![0u32; world.config.tag_length as usize];
    for a in world.agents() {
        for (i, z) in zeros.iter_mut().enumerate() {
            if !a.tags.get(i as u32) {
                *z += 1;
            }
        }
    }
    let n = world.population();
    zeros
        .into_iter()
        .map(|z| if n == 0 { 0.0 } else { 100.0 * f64::from(z) / n as f64 })
        .collect()
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/stats.rs crates/sugarscape-core/tests/invariants.rs
git commit -m "Add the age and cultural-tag histograms" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 4: The new networks and histograms in WASM

*Mechanical (full code).* Browser (controller): nothing to check (the page does not ask for them yet).

**Files:**
- Modify: `crates/sugarscape-wasm/src/lib.rs`
- Test: `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `World::{neighbor_edges, friend_edges, family_edges}` (Task 2), `stats::{age_histogram, tag_histogram}` (Task 3), `render` with `"lineage"` (Task 2).
- Produces (`#[wasm_bindgen] impl Sim`): `networks(kind)` also accepts `"neighbors"`, `"friends"`, `"family"`; `pub fn age_hist(&self, bin: u32) -> Vec<f64>` → JS `Float64Array` `[bin, count₀, …]` (`bin` 0 is treated as 1); `pub fn tag_hist(&self) -> Vec<f64>` → percentages, position 0 first.

- [ ] **Step 1: Write the failing test**

Append to `crates/sugarscape-wasm/tests/web.rs`:
```rust
#[wasm_bindgen_test]
fn chapter_vi_networks_histograms_and_lineage_colors() {
    let preset = sugarscape_core::presets::by_id("vi-1-everything").unwrap();
    let json = serde_json::to_string(&preset.config).unwrap();
    let mut sim = Sim::new(&json, 1, JsValue::NULL).unwrap();
    assert!(
        sim.networks("neighbors").unwrap().is_empty(),
        "nobody has moved yet"
    );
    sim.step(30);
    for kind in ["neighbors", "friends", "family"] {
        let edges = sim.networks(kind).unwrap();
        assert!(!edges.is_empty(), "{kind}");
        assert_eq!(edges.len() % 4, 0, "{kind}");
    }
    let ages = sim.age_hist(5);
    assert_eq!(ages[0], 5.0);
    assert_eq!(ages.len(), 1 + 21);
    assert_eq!(ages[1..].iter().sum::<f64>(), f64::from(sim.population()));
    let tags = sim.tag_hist();
    assert_eq!(tags.len(), 11);
    assert!(tags.iter().all(|p| (0.0..=100.0).contains(p)));
    sim.render("lineage", "sugar").unwrap();
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: FAIL to compile — no method `age_hist`/`tag_hist` on `Sim`.

- [ ] **Step 3: Implement**

In `crates/sugarscape-wasm/src/lib.rs`, replace `networks`' doc and match head:
```rust
    /// Edges as `[x1, y1, x2, y2, …]` for `"trade"` (this tick), `"credit"` (outstanding) or `"disease"` (infector → infected, this tick).
    pub fn networks(&self, kind: &str) -> Result<Vec<u32>, JsValue> {
        let edges = match kind {
            "trade" => network::trade_edges(&self.world),
            "credit" => network::credit_edges(&self.world),
            "disease" => network::disease_edges(&self.world),
```
with
```rust
    /// Edges as `[x1, y1, x2, y2, …]` for `"trade"` (this tick), `"credit"` (outstanding),
    /// `"disease"` (infector → infected, this tick), `"neighbors"` (agent → each agent on its
    /// neighbor list), `"friends"` (agent → friend) or `"family"` (parent → child).
    pub fn networks(&self, kind: &str) -> Result<Vec<u32>, JsValue> {
        let edges = match kind {
            "trade" => network::trade_edges(&self.world),
            "credit" => network::credit_edges(&self.world),
            "disease" => network::disease_edges(&self.world),
            "neighbors" => self.world.neighbor_edges(),
            "friends" => self.world.friend_edges(),
            "family" => self.world.family_edges(),
```
and before `pub fn inspect(&self, x: u32, y: u32)` add:
```rust
    /// `[bin, count_0, …]`: living agents' ages in `bin`-tick bins (`stats::age_histogram`).
    pub fn age_hist(&self, bin: u32) -> Vec<f64> {
        let bin = bin.max(1);
        std::iter::once(f64::from(bin))
            .chain(stats::age_histogram(&self.world, bin))
            .collect()
    }

    /// The percentage of living agents with a 0 at each tag position, position 0 first
    /// (`stats::tag_histogram`).
    pub fn tag_hist(&self) -> Vec<f64> {
        stats::tag_histogram(&self.world)
    }

```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS (21 tests).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-wasm/src/lib.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Expose the Chapter VI networks and histograms to JavaScript" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 5: Overlays, Lineage and histograms on the wire

*Mechanical (full code).* Browser (controller): every existing overlay (trade on `iv-3-trade`, credit on `iv-5-credit`, disease on `v-2-endemic`) and color mode still works; turning disease off still unticks and hides the disease overlay.

**Files:**
- Modify: `web/src/protocol.ts`, `web/src/layers.ts`, `web/src/types.ts`, `web/src/sim-host.ts`, `web/src/fake-sim.fixture.ts`, `web/src/engine.ts`
- Test: `web/src/protocol.test.ts`, `web/src/layers.test.ts`, `web/src/sim-host.test.ts`, `web/src/transport.test.ts`

**Interfaces:**
- Consumes: WASM `Sim.age_hist(bin)`, `Sim.tag_hist()`, `networks("neighbors" | "friends" | "family")`, `render("lineage", …)` (Task 4).
- Produces:
  - `protocol.ts`: `type Overlay = 'trade' | 'credit' | 'disease' | 'neighbors' | 'friends' | 'family'`; `OVERLAYS` in that order; `function noOverlays(): Record<Overlay, boolean>`; `Wants.ageHist?: boolean`, `Wants.tagHist?: boolean` (flags merged by `mergeWants`); `WorldSnapshot.ageHist?: Float64Array` (`[bin, counts…]`), `WorldSnapshot.tagHist?: Float64Array` (percent per position).
  - `layers.ts`: `function overlayAvailable(kind: Overlay, config: Config): boolean`; `clampDisplay` turns unavailable overlays off.
  - `types.ts`: `ColorMode` gains `'lineage'`.
  - `sim-host.ts`: `export const AGE_BIN = 5`; `SimLike.age_hist(bin: number): Float64Array`, `SimLike.tag_hist(): Float64Array`.
  - `fake-sim.fixture.ts`: `FakeConfig` gains `sex`, `lifespan`, `culture` (`{ enabled: boolean }`, default off); `age_hist(bin)` → `[bin, population, 0]`; `tag_hist()` → `[100, 0]`.

- [ ] **Step 1: Write the failing tests**

Replace `web/src/layers.test.ts` with:
```ts
import { describe, expect, it } from 'vitest';
import { noOverlays, type DisplayState } from './protocol';
import type { Config } from './types';
import { clampDisplay, layerOptions, overlayAvailable, validLayer } from './layers';

const config = {
  goods: [{ name: 'sugar' }, { name: 'salt' }],
  pollution: { enabled: true, pollutants: [{ name: 'smoke' }] },
} as unknown as Config;

describe('layers', () => {
  it('lists each good and its capacity by name, then each pollutant', () => {
    expect(layerOptions(config)).toEqual([
      ['resource:0', 'sugar'],
      ['capacity:0', 'sugar capacity'],
      ['resource:1', 'salt'],
      ['capacity:1', 'salt capacity'],
      ['pollution:0', 'smoke'],
    ]);
  });

  it('falls back to good 0 for a layer the world lacks', () => {
    expect(validLayer('capacity:1', config)).toBe('capacity:1');
    expect(validLayer('resource:2', config)).toBe('resource:0');
    expect(validLayer('pollution:1', config)).toBe('resource:0');
  });
});

describe('clampDisplay', () => {
  const rules = (on: boolean) =>
    ({ ...config, disease: { enabled: on }, culture: { enabled: on }, sex: { enabled: on } }) as unknown as Config;
  const display: DisplayState = {
    colorMode: 'disease',
    layer: 'pollution:0',
    overlays: { ...noOverlays(), trade: true, disease: true, neighbors: true, friends: true, family: true },
  };

  it('keeps a valid display as the same object', () => {
    expect(clampDisplay(display, rules(true))).toBe(display);
  });

  it('drops the disease mode and every overlay the world cannot show, and a layer the world lacks', () => {
    expect(clampDisplay(display, rules(false))).toEqual({
      colorMode: 'tribe',
      layer: 'pollution:0',
      overlays: { ...noOverlays(), trade: true, neighbors: true },
    });
    expect(clampDisplay({ ...display, layer: 'capacity:2' }, rules(true)).layer).toBe('resource:0');
  });

  it('keeps the Lineage color mode in any world', () => {
    expect(clampDisplay({ ...display, colorMode: 'lineage' }, rules(false)).colorMode).toBe('lineage');
  });

  it('offers the neighbor network always, friends with culture and family with sex', () => {
    const only = (culture: boolean, sex: boolean) =>
      ({ ...config, disease: { enabled: false }, culture: { enabled: culture }, sex: { enabled: sex } }) as unknown as Config;
    expect(overlayAvailable('neighbors', only(false, false))).toBe(true);
    expect(overlayAvailable('friends', only(false, true))).toBe(false);
    expect(overlayAvailable('friends', only(true, false))).toBe(true);
    expect(overlayAvailable('family', only(true, false))).toBe(false);
    expect(overlayAvailable('family', only(false, true))).toBe(true);
    expect(overlayAvailable('disease', only(true, true))).toBe(false);
  });
});
```
In `web/src/protocol.test.ts`, in the first `mergeWants` test, make the second part
```ts
      {
        select: { x: 9, y: 9, agentId: 3 },
        lorenz: true,
        ageHist: true,
        networks: ['family', 'trade', 'disease', 'neighbors'],
        charts: { groups: [['population'], ['gini', 'births']], max: 2000 },
      },
```
and the expected result
```ts
    expect(merged).toEqual({
      select: { x: 1, y: 2, agentId: null },
      lorenz: true,
      ageHist: true,
      networks: ['trade', 'disease', 'neighbors', 'family'],
      charts: { groups: [['population'], ['gini', 'births']], max: 2000 },
    });
```
In `web/src/sim-host.test.ts`: replace the `import type { Command, … } from './protocol';` line with
```ts
import {
  noOverlays,
  type Command,
  type DisplayState,
  type EditCommand,
  type HostMessage,
  type HostReply,
  type LogEntry,
  type SessionLog,
  type Wants,
  type WorldSnapshot,
} from './protocol';
```
change the sim-host import to `import { AGE_BIN, BATCH_MS, channelDefer, LOG_CAP, serve, SimHost } from './sim-host';`, change the `display` constant's overlays to `overlays: noOverlays()`, and in `'adds extras only when wanted'`: add `'ageHist', 'tagHist',` after `'wealthHist',` in `keys`; in `all` set `networks: ['trade', 'neighbors', 'friends', 'family'],` and add `ageHist: true,` and `tagHist: true,` after `wealthHist: true,`; after `expect(full.wealthHist).toHaveLength(21);` add
```ts
    expect(Object.keys(full.networks ?? {})).toEqual(['trade', 'neighbors', 'friends', 'family']);
    expect(full.ageHist).toEqual(Float64Array.of(AGE_BIN, 1, 0));
    expect(full.tagHist).toEqual(Float64Array.of(100, 0));
```
In `web/src/transport.test.ts`: replace `import type { Command, DisplayState, HostRequest, WorldSnapshot } from './protocol';` with `import { noOverlays, type Command, type DisplayState, type HostRequest, type WorldSnapshot } from './protocol';` and the `display` constant's overlays with `overlays: noOverlays()`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `(cd web && npm run build)`
Expected: FAIL in `tsc` — `noOverlays`, `overlayAvailable`, `AGE_BIN` are not exported; `'lineage'` is not a `ColorMode`; `ageHist` is not in `Wants`; `neighbors` is not an `Overlay`.

- [ ] **Step 3: Implement**

`web/src/protocol.ts` — replace
```ts
export type Overlay = 'trade' | 'credit' | 'disease';
export const OVERLAYS: Overlay[] = ['trade', 'credit', 'disease'];
```
with (Decision 10)
```ts
export type Overlay = 'trade' | 'credit' | 'disease' | 'neighbors' | 'friends' | 'family';
export const OVERLAYS: Overlay[] = ['trade', 'credit', 'disease', 'neighbors', 'friends', 'family'];

/** Every overlay off (a fresh object each call). */
export function noOverlays(): Record<Overlay, boolean> {
  return Object.fromEntries(OVERLAYS.map((k) => [k, false])) as Record<Overlay, boolean>;
}
```
in `Wants`, after `wealthHist?: boolean;` add `ageHist?: boolean;` and `tagHist?: boolean;`; in `WorldSnapshot`, after `wealthHist?: Float64Array;` add
```ts
  /** `[bin, count_0, …]`: living agents' ages in 5-tick bins (Animation III-1). */
  ageHist?: Float64Array;
  /** The percentage of agents with a 0 at each tag position, position 0 first (Animation III-7). */
  tagHist?: Float64Array;
```
and change `FLAGS` to
```ts
const FLAGS = ['trail', 'lorenz', 'wealthHist', 'ageHist', 'tagHist', 'supplyDemand', 'creditGraph', 'diseaseList'] as const;
```
`web/src/layers.ts` — change the first import to `import { OVERLAYS, type DisplayState, type Overlay } from './protocol';` and replace `clampDisplay` (with its doc comment) by
```ts
/** Whether an overlay can show anything in a world with `config`: disease, friends and family need their rules. */
export function overlayAvailable(kind: Overlay, config: Config): boolean {
  switch (kind) {
    case 'disease':
      return config.disease.enabled;
    case 'friends':
      return config.culture.enabled;
    case 'family':
      return config.sex.enabled;
    default:
      return true;
  }
}

/**
 * `d` kept valid for `config` (the host applies it to every snapshot): a layer the world lacks falls
 * back to good 0's level; with disease off the Disease color mode falls back to Tribe; an overlay
 * the world cannot show (`overlayAvailable`) is turned off. Returns `d` itself when nothing changes.
 */
export function clampDisplay(d: DisplayState, config: Config): DisplayState {
  const layer = validLayer(d.layer, config);
  const colorMode = !config.disease.enabled && d.colorMode === 'disease' ? 'tribe' : d.colorMode;
  const off = OVERLAYS.filter((k) => d.overlays[k] && !overlayAvailable(k, config));
  if (layer === d.layer && colorMode === d.colorMode && off.length === 0) return d;
  const overlays = { ...d.overlays };
  for (const k of off) overlays[k] = false;
  return { colorMode, layer, overlays };
}
```
`web/src/types.ts` — `export type ColorMode = 'tribe' | 'wealth' | 'sex' | 'age' | 'vision' | 'credit' | 'disease' | 'lineage';`

`web/src/sim-host.ts` — add `noOverlays,` to the `./protocol` value imports (after `chartKey,`); in `SimLike`, after `wealth_hist(bins: number): Float64Array;` add
```ts
  age_hist(bin: number): Float64Array;
  tag_hist(): Float64Array;
```
before the `LOG_CAP` doc comment add
```ts
/** Animation III-1's age histogram bins are this many ticks wide. */
export const AGE_BIN = 5;

```
replace the `display` field's initializer with `{ colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() }`; and in `snapshot`, after `if (wants.wealthHist) s.wealthHist = sim.wealth_hist(20);` add
```ts
    if (wants.ageHist) s.ageHist = sim.age_hist(AGE_BIN);
    if (wants.tagHist) s.tagHist = sim.tag_hist();
```
`web/src/fake-sim.fixture.ts` — in `FakeConfig` after `disease: { enabled: boolean };` add `sex: { enabled: boolean }; lifespan: { enabled: boolean }; culture: { enabled: boolean };` (one per line); in `normalize` after `disease: { enabled: false },` add `sex: { enabled: false }, lifespan: { enabled: false }, culture: { enabled: false },` (one per line); before `supply_demand()` add
```ts
  age_hist(bin: number): Float64Array {
    return Float64Array.of(bin, this.agents.size, 0);
  }
  tag_hist(): Float64Array {
    return Float64Array.of(100, 0);
  }
```
`web/src/engine.ts` — add `noOverlays,` to the `./protocol` imports (after `mergeWants,`) and replace `overlays: Record<Overlay, boolean> = { trade: false, credit: false, disease: false };` with `overlays: Record<Overlay, boolean> = noOverlays();`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/protocol.ts web/src/protocol.test.ts web/src/layers.ts web/src/layers.test.ts web/src/types.ts web/src/sim-host.ts web/src/sim-host.test.ts web/src/fake-sim.fixture.ts web/src/transport.test.ts web/src/engine.ts
git commit -m "Carry the Chapter VI networks, Lineage mode and histograms between host and page" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 6: The three network overlays and Lineage colors on the page

*Mechanical (full code).* Browser (controller): on `vi-1-everything` run 30 ticks, tick **Neighbor network** (thin lines, one arrowhead per edge just outside the target's cell; edges crossing the torus edge are drawn as two halves, the arrowhead on the target's half), **Friends network** and **Family network**; on `ii-2-unit` only Neighbor network is offered (Friends and Family hidden), on `iii-6-culture` Friends appears, on `iii-2-sex` Family appears; turning culture off live on `iii-6-culture` unticks and hides Friends; Agents → **Lineage** on `iii-2-sex`: grey founders at t = 0, then red new parents, green children and later yellow born parents; in Compare (`iii-2-sex`, Compare, then culture on in B) each grid draws its own world's networks; paused with the overlays on, DevTools' network/worker messages stay quiet.

**Files:**
- Modify: `web/src/ui/display.ts`, `web/src/ui/overlay.ts`, `web/src/ui/grid-view.ts`
- Test: `web/src/ui/overlay.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: `Overlay`, `OVERLAYS` (via `Engine.overlays`), `overlayAvailable` (Task 5), `Engine.networks(kind)`, `Engine.setDisplay`, `Engine.want`, `Engine.last`, `Engine.frame()`, `wrappedSegments`.
- Produces: `overlay.ts`: `export function arrowHead(ax, ay, bx, by, size, back): [number, number, number, number, number, number] | null` (tip, left, right in pixels; tip `back` short of b).

- [ ] **Step 1: Write the failing tests**

Replace `web/src/ui/overlay.test.ts` with:
```ts
import { describe, expect, it } from 'vitest';
import { arrowHead, wrappedSegments } from './overlay';

describe('wrappedSegments', () => {
  it('keeps nearby edges as one segment', () => {
    expect(wrappedSegments(1, 1, 2, 1, 50, 50)).toEqual([[1, 1, 2, 1]]);
  });
  it('splits an edge that wraps horizontally', () => {
    expect(wrappedSegments(0, 5, 49, 5, 50, 50)).toEqual([[0, 5, -1, 5], [50, 5, 49, 5]]);
  });
  it('splits an edge that wraps vertically', () => {
    expect(wrappedSegments(3, 49, 3, 0, 50, 50)).toEqual([[3, 49, 3, 50], [3, -1, 3, 0]]);
  });
  it('ends a split directed edge at its target, so the marker is drawn at the right end', () => {
    // A neighbor edge across the east edge: from (49, 7) to its neighbor (0, 7).
    const segments = wrappedSegments(49, 7, 0, 7, 50, 50);
    expect(segments).toHaveLength(2);
    expect(segments.at(-1)!.slice(2)).toEqual([0, 7]);
    expect(segments.at(-1)!.slice(0, 2)).toEqual([-1, 7]);
  });
});

describe('arrowHead', () => {
  it('points along the edge, its tip short of the target', () => {
    const [tx, ty, lx, ly, rx, ry] = arrowHead(0, 0, 10, 0, 4, 2)!;
    expect([tx, ty]).toEqual([8, 0]);
    expect([lx, ly]).toEqual([4, 2]);
    expect([rx, ry]).toEqual([4, -2]);
  });
  it('works in any direction', () => {
    const [tx, ty, lx, ly, rx, ry] = arrowHead(5, 5, 5, 25, 4, 0)!;
    expect([tx, ty]).toEqual([5, 25]);
    expect([lx, ly]).toEqual([3, 21]);
    expect([rx, ry]).toEqual([7, 21]);
  });
  it('has none for a zero-length edge', () => {
    expect(arrowHead(3, 3, 3, 3, 4, 2)).toBeNull();
  });
});
```
In `web/src/determinism.test.ts`, insert before `describe('sessions replay exactly', () => {`:
```ts
describe('Chapter VI views', () => {
  const everything = presets.find((p) => p.id === 'vi-1-everything')!;
  const create = () => Engine.create({ config: structuredClone(everything.config), seed: 1 }, { presets, transport: inline() });
  /** crates/sugarscape-core/src/render.rs: FOUNDER (dark grey) and BORN (green). */
  const has = (frame: Uint8ClampedArray, rgb: [number, number, number]) => {
    for (let i = 0; i < frame.length; i += 4) if (frame[i] === rgb[0] && frame[i + 1] === rgb[1] && frame[i + 2] === rgb[2]) return true;
    return false;
  };

  it('draw lineage colors and fetch networks and histograms without changing the run', async () => {
    const plain = await create();
    await plain.advance(100);
    const watched = await create();
    watched.setDisplay({ colorMode: 'lineage', overlays: { neighbors: true, friends: true, family: true } });
    watched.want(() => ({ ageHist: true, tagHist: true }));
    for (let i = 0; i < 10; i++) await watched.advance(10);
    expect(watched.networks('neighbors').length).toBeGreaterThan(0);
    expect(watched.networks('friends').length).toBeGreaterThan(0);
    expect(watched.networks('family').length).toBeGreaterThan(0);
    expect(watched.last?.ageHist?.[0]).toBe(5);
    expect(watched.last?.tagHist).toHaveLength(11);
    const frame = watched.frame()!;
    expect(has(frame, [0x5a, 0x5a, 0x5a])).toBe(true);
    expect(has(frame, [0x3d, 0xd6, 0x6b])).toBe(true);
    expect(await watched.fingerprint()).toBe(await plain.fingerprint());
  });
});

```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `(cd web && npm run build && npx vitest run src/ui/overlay.test.ts src/determinism.test.ts)`
Expected: the build fails in `tsc` — `arrowHead` is not exported from `./overlay` (determinism's new test needs no new code: it passes once the build succeeds).

- [ ] **Step 3: Implement**

`web/src/ui/overlay.ts` — replace the file with:
```ts
/** Line segments (in cell units) for an edge on a torus: one if the endpoints
 *  are within half the grid, otherwise two segments running off each side.
 *  The last segment always ends at (x2, y2), where a directed edge's marker goes. */
export function wrappedSegments(
  x1: number, y1: number, x2: number, y2: number, width: number, height: number,
): [number, number, number, number][] {
  let dx = x2 - x1;
  let dy = y2 - y1;
  if (dx > width / 2) dx -= width;
  if (dx < -width / 2) dx += width;
  if (dy > height / 2) dy -= height;
  if (dy < -height / 2) dy += height;
  if (x1 + dx === x2 && y1 + dy === y2) return [[x1, y1, x2, y2]];
  return [[x1, y1, x1 + dx, y1 + dy], [x2 - dx, y2 - dy, x2, y2]];
}

/**
 * A direction marker for the segment (ax, ay) → (bx, by), in pixels: a triangle whose tip is `back`
 * short of b (outside the target agent's cell) and whose base is `size` behind the tip and `size`
 * wide. Returns [tipX, tipY, leftX, leftY, rightX, rightY], or null for a zero-length segment.
 */
export function arrowHead(
  ax: number, ay: number, bx: number, by: number, size: number, back: number,
): [number, number, number, number, number, number] | null {
  const len = Math.hypot(bx - ax, by - ay);
  if (len === 0) return null;
  const ux = (bx - ax) / len;
  const uy = (by - ay) / len;
  const tx = bx - ux * back;
  const ty = by - uy * back;
  const cx = tx - ux * size;
  const cy = ty - uy * size;
  const half = size / 2;
  return [tx, ty, cx - uy * half, cy + ux * half, cx + uy * half, cy - ux * half];
}
```
`web/src/ui/grid-view.ts` (Decision 11) — change the imports to
```ts
import type { Engine, Overlay } from '../engine';
import { arrowHead, wrappedSegments } from './overlay';
```
after `const CELL = 12;` add
```ts

/** How each overlay's edges are drawn: a color token, and a direction marker for directed networks. */
const OVERLAY_STYLE: Record<Overlay, { color: string; directed?: true; width: number; alpha: number }> = {
  trade: { color: '--c3', width: 1.5, alpha: 0.8 },
  credit: { color: '--c2', width: 1.5, alpha: 0.8 },
  disease: { color: '--c4', width: 1.5, alpha: 0.8 },
  neighbors: { color: '--muted', directed: true, width: 1, alpha: 0.7 },
  friends: { color: '--c1', width: 1.5, alpha: 0.8 },
  family: { color: '--accent', width: 1.5, alpha: 0.8 },
};
```
and in `draw()` replace the whole `for (const [kind, color] of [['trade', '--c3'], …] as const) { … }` loop with
```ts
    for (const [kind, style] of Object.entries(OVERLAY_STYLE) as [Overlay, (typeof OVERLAY_STYLE)[Overlay]][]) {
      if (!this.engine.overlays[kind]) continue;
      const e = this.engine.networks(kind);
      const color = getComputedStyle(this.canvas).getPropertyValue(style.color).trim() || '#fff';
      ctx.save();
      ctx.strokeStyle = color;
      ctx.fillStyle = color;
      ctx.globalAlpha = style.alpha;
      ctx.lineWidth = style.width;
      ctx.beginPath();
      const heads = new Path2D();
      for (let i = 0; i < e.length; i += 4) {
        const segments = wrappedSegments(e[i], e[i + 1], e[i + 2], e[i + 3], width, height);
        for (const [ax, ay, bx, by] of segments) {
          ctx.moveTo((ax + 0.5) * CELL, (ay + 0.5) * CELL);
          ctx.lineTo((bx + 0.5) * CELL, (by + 0.5) * CELL);
        }
        if (!style.directed) continue;
        // The last segment ends at the target: the marker sits just outside its cell.
        const [ax, ay, bx, by] = segments[segments.length - 1];
        const head = arrowHead((ax + 0.5) * CELL, (ay + 0.5) * CELL, (bx + 0.5) * CELL, (by + 0.5) * CELL, 4, CELL / 2);
        if (!head) continue;
        heads.moveTo(head[0], head[1]);
        heads.lineTo(head[2], head[3]);
        heads.lineTo(head[4], head[5]);
        heads.closePath();
      }
      ctx.stroke();
      if (style.directed) ctx.fill(heads);
      ctx.restore();
    }
```
`web/src/ui/display.ts` — change `import { layerOptions } from '../layers';` to `import { layerOptions, overlayAvailable } from '../layers';`; add `['lineage', 'Lineage'],` after `['disease', 'Disease'],` in `MODES`, and after `MODES` add
```ts

/** The overlay checkboxes, in order (Chapter VI's three networks after the others). */
const OVERLAY_LABELS: [Overlay, string][] = [
  ['trade', 'Trade network'],
  ['credit', 'Credit network'],
  ['disease', 'Disease network'],
  ['neighbors', 'Neighbor network'],
  ['friends', 'Friends network'],
  ['family', 'Family network'],
];
```
and replace the `overlay` helper and the returned element with (Decision 10):
```ts
  /** A checkbox per overlay; Friends and Family show only while culture and sex are on (and Disease while disease is). */
  const overlay = (kind: Overlay, label: string) => {
    const box = h('input', { type: 'checkbox', onchange: () => engine.setDisplay({ overlays: { [kind]: box.checked } }) });
    const el = h('label', {}, box, ` ${label}`);
    const show = () => (el.hidden = !overlayAvailable(kind, engine.config));
    engine.on('display', () => (box.checked = engine.overlays[kind]));
    engine.on('reset', show);
    engine.on('config', show);
    show();
    return el;
  };
  return h(
    'div',
    { class: 'display-controls' },
    h('label', {}, 'Agents ', mode),
    h('label', {}, 'Landscape ', layer),
    ...OVERLAY_LABELS.map(([kind, label]) => overlay(kind, label)),
  );
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all tests PASS (the determinism test shows the lineage frame has grey founders and green children, and the watched run's fingerprint equals the plain one's).

- [ ] **Step 5: Commit**

```bash
git add web/src/ui/display.ts web/src/ui/overlay.ts web/src/ui/overlay.test.ts web/src/ui/grid-view.ts web/src/determinism.test.ts
git commit -m "Draw the neighbor, friends and family networks and Lineage colors" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 7: The age and cultural-tag histograms in Charts

*Mechanical (full code).* Browser (controller): `iii-2-sex` — **Age histogram** appears after Wealth distribution (bars in 5-tick bins up to 105, shifting as the population ages) and **Cultural tags** does not; `iii-6-culture` — **Cultural tags (% zeros by position)** shows 11 bars at positions 1–11 (0–100 %) moving toward 0 or 100 while Age histogram is hidden; `vi-1-everything` shows both; turning lifespan off live hides the age chart; paused with both shown, the charts stop fetching after one catch-up (no worker messages) and a paint/erase while paused refreshes them once; Compare on `vi-1-everything` — both charts are two step outlines (A solid, B dashed) with "A · …"/"B · …" legends.

**Files:**
- Modify: `web/src/ui/series-data.ts`, `web/src/ui/charts-panel.ts`
- Test: `web/src/ui/series-data.test.ts`, `web/src/engine.test.ts`

**Interfaces:**
- Consumes: `Wants.ageHist/tagHist`, `WorldSnapshot.ageHist/tagHist` (Task 5), `barsData`, `histTable`, `overlayData`.
- Produces (`series-data.ts`): `positionBars(pct: Float64Array | null | undefined): LineData` (x = 1…L); `positionSteps(pct): LineData` (edges 0.5…L + 0.5, values then 0); `showsAgeHist(c: Config): boolean`; `showsTagHist(c: Config): boolean`; `interface DistState { at: number; tick: number; stale: boolean }`; `distributionsDue(d: DistState, tick: number, now: number, every: number): boolean`; `distributionWants(c: Config): Wants`.

- [ ] **Step 1: Write the failing tests**

In `web/src/ui/series-data.test.ts`, replace the `./series-data` import with
```ts
import type { Config } from '../types';
import {
  bandData,
  barsData,
  chartsBehind,
  distributionsDue,
  distributionWants,
  emptyTable,
  histTable,
  lineData,
  overlayData,
  positionBars,
  positionSteps,
  showsAgeHist,
  showsTagHist,
  supplyDemandTable,
  type LineData,
} from './series-data';
```
and append:
```ts
describe('Chapter VI histograms', () => {
  const config = (goods: number, lifespan: boolean, culture: boolean) =>
    ({ goods: Array.from({ length: goods }, () => ({})), lifespan: { enabled: lifespan }, culture: { enabled: culture } }) as unknown as Config;

  it('shows the age histogram while lifetimes are finite and the tag histogram while culture is on', () => {
    expect([showsAgeHist(config(1, true, false)), showsTagHist(config(1, true, false))]).toEqual([true, false]);
    expect([showsAgeHist(config(1, false, true)), showsTagHist(config(1, false, true))]).toEqual([false, true]);
  });

  it('asks for the distributions each world draws', () => {
    expect(distributionWants(config(1, false, false))).toEqual({ lorenz: true, wealthHist: true });
    expect(distributionWants(config(2, true, true))).toEqual({
      lorenz: true,
      wealthHist: true,
      supplyDemand: true,
      ageHist: true,
      tagHist: true,
    });
  });

  it('fetches distributions again only once the tick moved or they went stale, at most every 250 ms', () => {
    const d = { at: 1000, tick: 7, stale: false };
    expect(distributionsDue(d, 7, 5000, 250)).toBe(false); // paused and caught up
    expect(distributionsDue(d, 8, 1100, 250)).toBe(false); // too soon
    expect(distributionsDue(d, 8, 1250, 250)).toBe(true);
    expect(distributionsDue({ ...d, stale: true }, 7, 1250, 250)).toBe(true);
  });

  it('draws the tag histogram as bars at positions 1…L or as a step outline around them', () => {
    const pct = Float64Array.of(49, 100, 0);
    expect(positionBars(pct)).toEqual([[1, 2, 3], [49, 100, 0]]);
    expect(positionSteps(pct)).toEqual([[0.5, 1.5, 2.5, 3.5], [49, 100, 0, 0]]);
    expect(positionBars(null)).toEqual([[], []]);
    expect(positionSteps(undefined)).toEqual([[], []]);
  });

  it('draws the age histogram like the wealth histogram ([bin, counts…])', () => {
    const ages = Float64Array.of(5, 3, 0, 2);
    expect(barsData(ages)).toEqual([[2.5, 7.5, 12.5], [3, 0, 2]]);
    expect(histTable(ages)).toEqual([[0, 5, 10, 15], [3, 0, 2, 0]]);
  });
});
```
In `web/src/engine.test.ts`, change `import { chartsBehind } from './ui/series-data';` to `import { chartsBehind, distributionsDue, distributionWants, type DistState } from './ui/series-data';` and append:
```ts
describe('Chapter VI views while paused', () => {
  it('fetch the networks and histograms once, then send nothing while caught up', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const chapterVi = { width: 4, height: 3, sex: { enabled: true }, lifespan: { enabled: true }, culture: { enabled: true } } as unknown as Config;
    const engine = await Engine.create({ config: chapterVi, seed: 7 }, { presets, transport });
    // The Charts panel's distributions provider (ui/charts-panel.ts), with its receive step.
    const dist: DistState = { at: -Infinity, tick: -1, stale: true };
    engine.want((now) => (distributionsDue(dist, engine.tick, now, 250) ? distributionWants(engine.config) : {}));
    engine.on('snapshot', () => {
      const s = engine.last;
      if (s?.lorenz) Object.assign(dist, { at: performance.now(), tick: s.tick, stale: false });
    });
    const sent: string[] = [];
    transport.after = (cmd) => sent.push(cmd.type);
    let now = performance.now();
    const pumps = async (n: number) => {
      for (let i = 0; i < n; i++) {
        engine.pump((now += 1000));
        await settle();
      }
    };

    await pumps(1);
    expect(sent).toEqual(['refresh']);
    expect(engine.last?.ageHist).toEqual(Float64Array.of(5, 1, 0));
    expect(engine.last?.tagHist).toEqual(Float64Array.of(100, 0));

    engine.setDisplay({ colorMode: 'lineage', overlays: { neighbors: true, friends: true, family: true } });
    await settle();
    expect(Object.keys(engine.last?.networks ?? {})).toEqual(['neighbors', 'friends', 'family']);
    expect(engine.colorMode).toBe('lineage');

    // Paused and caught up, with three overlays and both histograms shown: nothing more is sent.
    await pumps(5);
    expect(sent).toEqual(['refresh', 'setDisplay']);
  });
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `(cd web && npm run build)`
Expected: FAIL in `tsc` — `distributionsDue`, `distributionWants`, `DistState`, `positionBars`, `positionSteps`, `showsAgeHist`, `showsTagHist` are not exported from `./series-data`.

- [ ] **Step 3: Implement**

`web/src/ui/series-data.ts` — change the first import to
```ts
import type { ChartGroup, Wants } from '../protocol';
import type { Config } from '../types';
```
and insert before the `/**\n * Supply & demand` doc of `supplyDemandTable` (Decisions 9 and 12):
```ts
/** The tag histogram (percent with a 0 at each position, position 0 first) as bars at positions 1…L. */
export function positionBars(pct: Float64Array | null | undefined): LineData {
  if (!pct) return [[], []];
  const values = Array.from(pct);
  return [values.map((_, i) => i + 1), values];
}

/** The tag histogram as a step outline: each position's bar from p − ½ to p + ½, closed at the right. */
export function positionSteps(pct: Float64Array | null | undefined): LineData {
  if (!pct) return [[], []];
  const values = Array.from(pct);
  return [Array.from({ length: values.length + 1 }, (_, k) => k + 0.5), [...values, 0]];
}

/** The age histogram shows while lifetimes are finite (Animation III-1). */
export const showsAgeHist = (c: Config): boolean => c.lifespan.enabled;
/** The cultural tag histogram shows while culture is on (Animation III-7). */
export const showsTagHist = (c: Config): boolean => c.culture.enabled;

/** A world's distributions as last received: when, at which tick, and whether an edit, reset or config change has made them stale. */
export interface DistState { at: number; tick: number; stale: boolean }

/**
 * Whether a world's distributions are fetched again: only once the tick moved or they went stale,
 * and at most every `every` ms — so a paused, caught-up Charts tab asks for nothing (7a's rule).
 */
export function distributionsDue(d: DistState, tick: number, now: number, every: number): boolean {
  return (d.stale || tick !== d.tick) && now - d.at >= every;
}

/**
 * The distributions a world's charts draw: the Lorenz curve and wealth histogram always, supply &
 * demand with two goods, the age histogram while lifetimes are finite, the tag histogram while
 * culture is on.
 */
export function distributionWants(c: Config): Wants {
  const out: Wants = { lorenz: true, wealthHist: true };
  if (c.goods.length >= 2) out.supplyDemand = true;
  if (showsAgeHist(c)) out.ageHist = true;
  if (showsTagHist(c)) out.tagHist = true;
  return out;
}

```
`web/src/ui/charts-panel.ts`:
1. In the `./series-data` import add `distributionsDue,` and `distributionWants,` after `chartsBehind,`; `positionBars,`, `positionSteps,`, `showsAgeHist,`, `showsTagHist,` after `overlayData,`; and `type DistState,` before `type LineData,`.
2. `type Kind = 'time' | 'band' | 'lorenz' | 'wealth' | 'age' | 'tags' | 'supplyDemand';`
3. The `REFRESH_MS` doc becomes `/** The distributions (Lorenz curve, wealth, age and tag histograms, supply & demand) are fetched at most this often (per world). */`.
4. Replace `X_LABEL` with
```ts
const X_LABEL: Record<Kind, string> = {
  time: 'Tick',
  band: 'Tick',
  lorenz: 'Population share',
  wealth: 'Sugar',
  age: 'Age',
  tags: 'Tag position',
  supplyDemand: 'Price',
};
```
5. In `CHARTS`, after `{ title: 'Wealth distribution', kind: 'wealth', section: 'top' },` add
```ts
  { title: 'Age histogram', kind: 'age', section: 'top', shown: showsAgeHist },
  { title: 'Cultural tags (% zeros by position)', kind: 'tags', section: 'top', shown: showsTagHist, range: [0, 100] },
```
6. Replace `interface Dist { … }` and `freshDist` with
```ts
/** A world's latest distributions, and when (and at which tick) they arrived. */
interface Dist extends DistState {
  lorenz: Float64Array | null;
  wealthHist: Float64Array | null;
  ageHist: Float64Array | null;
  tagHist: Float64Array | null;
  supplyDemand: Float64Array | null;
  version: number;
}

const freshDist = (): Dist => ({
  lorenz: null,
  wealthHist: null,
  ageHist: null,
  tagHist: null,
  supplyDemand: null,
  version: 0,
  at: -Infinity,
  tick: -1,
  stale: true,
});
```
7. In `wants()`, replace
```ts
    const d = this.dist[i];
    if ((d.stale || w.tick !== d.tick) && now - d.at >= REFRESH_MS) {
      out.lorenz = true;
      out.wealthHist = true;
      if (twoGoods(w.config)) out.supplyDemand = true;
    }
    return out;
```
with
```ts
    if (distributionsDue(this.dist[i], w.tick, now, REFRESH_MS)) Object.assign(out, distributionWants(w.config));
    return out;
```
8. In `receive()`, replace
```ts
      // A world that drops to one good stops sending this: clear it, not keep the last curve.
      d.supplyDemand = s.supplyDemand ?? null;
```
with
```ts
      // A world that drops to one good (or turns lifetimes or culture off) stops sending these:
      // clear them, not keep the last one.
      d.supplyDemand = s.supplyDemand ?? null;
      d.ageHist = s.ageHist ?? null;
      d.tagHist = s.tagHist ?? null;
```
9. In `seriesFor()`, replace the `case 'wealth': return this.worlds.length > 1 ? … : …;` arm with
```ts
      case 'wealth':
      case 'age':
        return this.histSeries(`${tag}Agents`, '--c1', dash);
      case 'tags':
        return this.histSeries(`${tag}% zeros`, '--c4', dash);
```
and add this method just before `/** One table as is; several on the union of their x values (Decision 11). */`:
```ts
  /** A histogram's series: bars for one world; in Compare a step outline per world (B dashed). */
  private histSeries(label: string, color: string, dash: number[] | undefined): uPlot.Series[] {
    const c = this.color(color);
    return this.worlds.length > 1
      ? [{ label, stroke: c, width: 1.5, dash, paths: uPlot.paths.stepped!({ align: 1 }), points: { show: false } }]
      : [{ label, fill: c, stroke: c, paths: uPlot.paths.bars!({ size: [0.9, 64] }), points: { show: false } }];
  }

```
10. In `distData()`, after the `case 'wealth':` arm add
```ts
      case 'age':
        return this.worlds.length > 1 ? overlayData(this.dist.map((d) => histTable(d.ageHist))) : barsData(this.dist[0].ageHist);
      case 'tags':
        return this.worlds.length > 1 ? overlayData(this.dist.map((d) => positionSteps(d.tagHist))) : positionBars(this.dist[0].tagHist);
```
(`twoGoods` stays: `SECTIONS` and `CHARTS` still use it.)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/ui/charts-panel.ts web/src/engine.test.ts
git commit -m "Chart the age and cultural-tag histograms, overlaid in Compare" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 8: The VI-2 and VI-3 presets, measured

*Needs judgement: the settings and thresholds come from measurement (Decisions 13–15).* Browser (controller): the presets menu lists `({G₁}, {M, S}) with spice, no trade — Animation VI-2` and `({G₁}, {M, S, T}) with spice — Animation VI-3` after `vi-1-everything`, with their descriptions; at Max, VI-2 seed 1 dies out (population 0 by t ≈ 400) and VI-3 seed 1 survives past t = 1000; `vi-1-everything`'s description names where each view lives.

**Files:**
- Modify: `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: `presets::{demography, spice}` helpers, `World::{step, population, stats}`.
- Produces: presets `vi-2-no-trade` and `vi-3-trade` (listed after `vi-1-everything`), identical except `trade.enabled`; `book.rs`: `population_peaks(pop: &[f64]) -> Vec<usize>`, `widest_peak_spacing(peaks: &[usize]) -> usize`, `indecomposability_run(config: &Config, seed: u64) -> (Option<u64>, usize, f64, Vec<usize>)`, `#[ignore] measure_indecomposability`, `#[ignore] trade_decides_whether_the_society_survives`.

- [ ] **Step 1: Write the failing preset test**

Append inside `presets.rs`'s `mod tests`:
```rust
    #[test]
    fn indecomposability_presets_differ_only_in_trade() {
        let no_trade = by_id("vi-2-no-trade").unwrap().config;
        let mut trade = by_id("vi-3-trade").unwrap().config;
        assert!(!no_trade.trade.enabled && trade.trade.enabled);
        assert_eq!(no_trade.population, 500);
        assert_eq!(no_trade.goods.len(), 2, "sugar and spice");
        assert!(no_trade.sex.enabled && no_trade.lifespan.enabled);
        assert_eq!(no_trade.lifespan.max_age, URange::new(60, 100));
        assert!(!no_trade.culture.enabled && !no_trade.credit.enabled && !no_trade.disease.enabled);
        trade.trade.enabled = false;
        assert_eq!(trade, no_trade);
    }
```
and in `every_preset_is_valid_and_runs` change `assert_eq!(presets.len(), 27);` to `assert_eq!(presets.len(), 29);`.

Run: `cargo test -p sugarscape-core --lib presets`
Expected: FAIL — `by_id("vi-2-no-trade")` is `None` (unwrap on None), and 27 ≠ 29.

- [ ] **Step 2: Add the presets with the planning measurement's values**

In `presets.rs`, before `/// A further good with good 0's trait ranges…`, add (Decision 13; the comment is the calibration record — Step 4 checks every number in it):
```rust
/// Chapter VI's indecomposability society (animations VI-2 and VI-3): 500
/// agents on Chapter IV's sugar and spice landscape under M and S with
/// Chapter III's demography; `trade` switches rule T, the only difference.
fn indecomposability(c: &mut Config, trade: bool) {
    c.population = 500;
    demography(c);
    // The book fixes the population, the landscape and the rules; vision,
    // metabolism and endowment (equal for both goods) were chosen by
    // `measure_indecomposability` (release, seeds 1-5, up to t = 1000), in
    // its order, as the first candidate whose five VI-2 runs all die out and
    // whose five VI-3 runs all survive with at least two population peaks.
    // Runs per candidate (VI-2 extinct / VI-3 alive at t = 1000):
    //   vision 1-10, metabolism 1-5, endowment 25-50: 0/5, 5/5 (no crash)
    //   vision 1-5,  metabolism 1-5, endowment 25-50: 0/5, 5/5 (no crash)
    //   vision 1-5,  metabolism 1-5, endowment 50-100: 5/5, 0/5 (trade dies faster)
    //   vision 1-5,  metabolism 1-6, endowment 30-60: 4/5, 4/5
    //   vision 1-5,  metabolism 1-3, endowment 60:    4/5, 5/5
    //   vision 1-5,  metabolism 1-2, endowment 64:    2/5, 4/5
    //   vision 1-5,  metabolism 1-2, endowment 65:    5/5, 5/5  <- chosen
    //   vision 1-5,  metabolism 1-2, endowment 66:    3/5, 2/5
    // With a high endowment, fertility (holdings at least the endowment of
    // both goods) is rare, and trade, which moves holdings toward the
    // metabolism ratio rather than the endowments, makes it rarer still, so
    // trade rescues the population only on a knife edge: at endowment 65,
    // VI-2 dies out at t = 385, 900, 530, 445, 651 and VI-3 ends at 173,
    // 181, 75, 106, 158 agents (seeds 1-5); 64 and 66 do not separate. Both
    // boom to about 1.3-1.4 times 500 by t = 53 and then crash. The book's
    // VI-3 recovery to twice the initial population, its minima at 700 and
    // its 115-year waves are not reproduced by these rules.
    c.vision = URange::new(1, 5);
    c.goods[0].metabolism = URange::new(1, 2);
    c.goods[0].endowment = URange::new(65, 65);
    spice(c, URange::new(1, 2), URange::new(65, 65));
    c.trade.enabled = trade;
}

```
and in `all()`, after the `vi-1-everything` preset and before `"n-3-trade"`, add:
```rust
        preset(
            "vi-2-no-trade",
            "({G₁}, {M, S}) with spice, no trade",
            "Animation VI-2",
            "500 agents on the sugar and spice landscape move and reproduce but never trade: after an early baby boom the population collapses and dies out (measured, seeds 1–5: extinct at t = 385–900). Compare it with VI-3 from the presets menu.",
            |c| indecomposability(c, false),
        ),
        preset(
            "vi-3-trade",
            "({G₁}, {M, S, T}) with spice",
            "Animation VI-3",
            "Everything as in VI-2, with trade on: the population collapses as fast but survives, recovering slowly (seeds 1–5: 75–181 agents at t = 1000). The book's recovery to twice the initial population and its 115-year waves are not reproduced by these rules.",
            |c| indecomposability(c, true),
        ),
```
Run: `cargo test -p sugarscape-core --lib presets`
Expected: PASS.

- [ ] **Step 3: Add the measurement** (Decision 14)

Append to `crates/sugarscape-core/tests/book.rs`:
```rust
/// Peaks of a population series: ticks t ≥ 50 (and at least 10 before the
/// end) where the 21-tick centered moving average is at least every value
/// within 40 ticks and above every value in the 40 ticks before (so a
/// plateau counts once).
fn population_peaks(pop: &[f64]) -> Vec<usize> {
    let smooth: Vec<f64> = (0..pop.len())
        .map(|i| {
            let (lo, hi) = (i.saturating_sub(10), (i + 10).min(pop.len() - 1));
            pop[lo..=hi].iter().sum::<f64>() / (hi - lo + 1) as f64
        })
        .collect();
    (50..pop.len().saturating_sub(10))
        .filter(|&t| {
            let (lo, hi) = (t - 40, (t + 40).min(pop.len() - 1));
            smooth[lo..=hi].iter().all(|&v| v <= smooth[t])
                && smooth[lo..t].iter().all(|&v| v < smooth[t])
        })
        .collect()
}

/// The longest gap between consecutive peaks (0 with fewer than two).
fn widest_peak_spacing(peaks: &[usize]) -> usize {
    peaks.windows(2).map(|w| w[1] - w[0]).max().unwrap_or(0)
}

/// One VI-2 / VI-3 run: the tick the population reached 0 (if it did by
/// t = 1000), the population at the end, its peak as a multiple of the
/// initial 500, and its peaks.
fn indecomposability_run(config: &Config, seed: u64) -> (Option<u64>, usize, f64, Vec<usize>) {
    let mut w = World::new(config.clone(), seed).unwrap();
    let mut extinct = None;
    while w.tick < 1000 {
        w.step();
        if w.population() == 0 {
            extinct = Some(w.tick);
            break;
        }
    }
    let pop = w.stats.series("population").unwrap();
    let max = pop.iter().copied().fold(0.0, f64::max);
    (extinct, w.population(), max / 500.0, population_peaks(&pop))
}

/// Calibration for `vi-2-no-trade` / `vi-3-trade`: each candidate (vision,
/// metabolism of both goods, endowment of both goods) on seeds 1–5.
#[test]
#[ignore]
fn measure_indecomposability() {
    use sugarscape_core::config::URange;
    let candidates = [
        ((1, 10), (1, 5), (25, 50)),
        ((1, 5), (1, 5), (25, 50)),
        ((1, 5), (1, 5), (50, 100)),
        ((1, 5), (1, 6), (30, 60)),
        ((1, 5), (1, 3), (60, 60)),
        ((1, 5), (1, 2), (64, 64)),
        ((1, 5), (1, 2), (65, 65)),
        ((1, 5), (1, 2), (66, 66)),
    ];
    for (vision, metabolism, endowment) in candidates {
        println!("vision {vision:?}, metabolism {metabolism:?}, endowment {endowment:?}:");
        for id in ["vi-2-no-trade", "vi-3-trade"] {
            let mut c = presets::by_id(id).unwrap().config;
            c.vision = URange::new(vision.0, vision.1);
            for g in &mut c.goods {
                g.metabolism = URange::new(metabolism.0, metabolism.1);
                g.endowment = URange::new(endowment.0, endowment.1);
            }
            for seed in 1..=5 {
                let (extinct, end, peak, peaks) = indecomposability_run(&c, seed);
                println!(
                    "  {id} seed {seed}: extinct at {extinct:?}, t=1000 population {end}, peak {peak:.2}x, {} peaks, widest spacing {}",
                    peaks.len(),
                    widest_peak_spacing(&peaks),
                );
            }
        }
    }
}
```

- [ ] **Step 4: Measure and choose**

Run: `cargo test -p sugarscape-core --release --test book -- --ignored --nocapture measure_indecomposability` (about a minute).
Expected (planning run; the runs are deterministic, so these should reproduce exactly):

| # | vision, metabolism, endowment | VI-2 extinct at (seeds 1–5) | VI-3 at t = 1000 | VI-3 peak × 500 | VI-3 peaks / widest spacing |
|---|---|---|---|---|---|
| 1 | 1–10, 1–5, 25–50 | none | 857, 793, 747, 860, 898 | 1.69–1.99 | 4–6 / 184–263 |
| 2 | 1–5, 1–5, 25–50 | none | 692, 705, 680, 731, 680 | 1.42–1.65 | 3–5 / 178–305 |
| 3 | 1–5, 1–5, 50–100 | 196, 181, 286, 196, 182 | all extinct (139–187) | — | — |
| 4 | 1–5, 1–6, 30–60 | 299, 224, 420, 306, alive (375) | 566, **0** (202), 622, 527, 519 | — | — |
| 5 | 1–5, 1–3, 60 | alive (220), 386, 794, 535, 509 | 235, 157, 192, 225, 258 | — | — |
| 6 | 1–5, 1–2, 64 | 541, 356, alive ×3 | 156, 187, 189, 203, **0** (570) | — | — |
| 7 | 1–5, 1–2, 65 | 385, 900, 530, 445, 651 | 173, 181, 75, 106, 158 | 1.31, 1.33, 1.31, 1.28, 1.34 | 3, 3, 3, 4, 4 / 709, 702, 571, 294, 521 |
| 8 | 1–5, 1–2, 66 | 498, alive, 409, 347, alive | **0**, 18, **0**, **0**, 43 | — | — |

Apply Decision 14's rule to the printed table: the first candidate with all five VI-2 runs extinct and all five VI-3 runs alive with at least two peaks. With the numbers above that is candidate 7, which Step 2 already uses — nothing to change. If the printed numbers differ, apply the same rule to them, put the chosen candidate in `indecomposability`, and rewrite its comment table and both descriptions with the printed numbers. If no candidate qualifies, stop and report BLOCKED with the printed table.

- [ ] **Step 5: Golden entries**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`. Check that the 27 earlier values equal their entries, then append to `GOLDEN` in `tests/golden.rs`, after the `iii-6-three-tribes` entry:
```rust
    // Chapter VI: indecomposability.
    ("vi-2-no-trade", 0xadfed5e8c7663a3e),
    ("vi-3-trade", 0x8cffd7cb9a31bf83),
```
(These are the planning values for candidate 7; if Step 4 chose another candidate, use the printed values.)

- [ ] **Step 6: The book-style test** (Decision 15)

Thresholds from the chosen candidate's VI-2/VI-3 rows: `VI2_EXTINCT_BY` = the latest VI-2 extinction tick rounded up to a multiple of 50; `VI3_PEAK_FACTOR` = the smallest VI-3 peak rounded down to a multiple of 0.05; `VI3_WIDEST_SPACING` = the smallest VI-3 widest spacing rounded down to a multiple of 10. For candidate 7: 900, 1.25, 290. Append to `book.rs`:
```rust
/// Chapter VI's thresholds, from `measure_indecomposability` (release, seeds
/// 1–5; presets.rs records the measurements).
const VI2_EXTINCT_BY: u32 = 900;
const VI3_PEAK_FACTOR: f64 = 1.25;
const VI3_WIDEST_SPACING: usize = 290;

#[test]
#[ignore]
fn trade_decides_whether_the_society_survives() {
    // Animations VI-2 and VI-3: "In the first run there is no trading. This
    // population crashes"; with trade on, "society pulls out of its
    // demographic nose dive". The book's recovery to twice the initial
    // population and 115-year waves are not reproduced (see presets.rs).
    let no_trade = presets::by_id("vi-2-no-trade").unwrap().config;
    let trade = presets::by_id("vi-3-trade").unwrap().config;
    for seed in 1..=5 {
        let w = run(no_trade.clone(), seed, VI2_EXTINCT_BY);
        assert_eq!(
            w.population(),
            0,
            "VI-2 seed {seed}: still alive at t = {VI2_EXTINCT_BY}"
        );
        let (extinct, _, peak, peaks) = indecomposability_run(&trade, seed);
        assert_eq!(extinct, None, "VI-3 seed {seed} died out");
        assert!(peak >= VI3_PEAK_FACTOR, "VI-3 seed {seed}: peak {peak:.2}x");
        assert!(
            peaks.len() >= 2 && widest_peak_spacing(&peaks) >= VI3_WIDEST_SPACING,
            "VI-3 seed {seed}: peaks {peaks:?}"
        );
    }
}
```
Run: `cargo test -p sugarscape-core --release --test book -- --ignored trade_decides_whether_the_society_survives`
Expected: PASS. (`measure_indecomposability` stays, `#[ignore]`d, as the calibration's record.)

- [ ] **Step 7: VI-1's eighteen views** (Decision 17)

Replace `vi-1-everything`'s description string with:
```rust
            "Every rule at once: spice, sex, finite lives, inheritance, culture, trade, credit and disease, with new diseases arriving by outbreak at t = 150, 400 and 650; disease flares after each outbreak and tends to die out again before the next one. The book's eighteen views: Agents → Disease colors; the neighbor, family, friends, trade, credit and disease network overlays; Charts for population, wealth distribution, Lorenz curve and Gini (sugar wealth), mean holdings of sugar and spice, age histogram, cultural tags, trade price with its spread, trade volume and disease; and the Credit tab's lender–borrower hierarchy.",
```
(The config is unchanged; its golden entry stays.)

- [ ] **Step 8: Verify**

Run: `cargo test -p sugarscape-core && wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS (`every_preset_has_a_golden_entry` included).

- [ ] **Step 9: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/book.rs
git commit -m "Add the measured VI-2 and VI-3 indecomposability presets" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 9: The Compare entry in the presets menu

*Mechanical (full code).* Browser (controller): type seed 3 in the seed box (don't press Reset), choose **Indecomposability — VI-2 vs VI-3 (Compare)** from the Rules tab's menu (under **Compare**): Compare opens with A `vi-2-no-trade` and B `vi-3-trade`, both headers `seed 3`, t = 0, the address bar has no hash; Play at Max — A dies out, B survives (population chart: A solid falling to 0, B dashed alive; the age histograms overlay as outlines); the menu shows A's preset again afterwards; choosing the entry while Compare is on replaces the pair (Compare is left keeping A first); B's "Rules for: B" menu has no Compare entries; every existing Compare scenario (the Compare button, `#c=` links, Keep A/B) still works.

**Files:**
- Create: `web/src/compare-presets.ts`, `web/src/compare-presets.test.ts`
- Modify: `web/src/ui/rules-panel.ts`, `web/src/ui/toolbar.ts`, `web/src/main.ts`, `web/src/engine.ts`
- Test: `web/src/engine.test.ts`

**Interfaces:**
- Consumes: presets `vi-2-no-trade`, `vi-3-trade` (Task 8); `InitialState`; main's `busy`, `compare`, `leaveCompare`, `buildCompare`, `hold`, `syncCompareButton`, `showNotice`, `fieldErrorsMessage`, `errorMessage`.
- Produces: `compare-presets.ts`: `interface ComparePreset { id: string; label: string; a: string; b: string }`, `COMPARE_PRESETS: ComparePreset[]`, `comparePresetStates(presets: Preset[], entry: ComparePreset, seed: number): { a: InitialState; b: InitialState } | null`; `RulesPanel` constructor `(engine: Engine, onCompare?: (id: string) => void)`; `Toolbar.typedSeed(): number`; `Engine.loadPreset(id: string, seed?: number)`.

- [ ] **Step 1: Write the failing tests**

Create `web/src/compare-presets.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import type { Config, Preset } from './types';

const preset = (id: string, trade: boolean): Preset => ({
  id,
  name: id,
  source: '',
  description: '',
  config: { trade: { enabled: trade } } as unknown as Config,
});

describe('compare presets', () => {
  const entry = COMPARE_PRESETS[0];

  it('opens VI-2 as A and VI-3 as B', () => {
    expect(entry).toMatchObject({ a: 'vi-2-no-trade', b: 'vi-3-trade', label: 'Indecomposability — VI-2 vs VI-3 (Compare)' });
  });

  it('builds both worlds from copies of the two presets at one seed', () => {
    const presets = [preset('vi-2-no-trade', false), preset('vi-3-trade', true)];
    const states = comparePresetStates(presets, entry, 42)!;
    expect(states.a.seed).toBe(42);
    expect(states.b.seed).toBe(42);
    expect(states.a.config.trade.enabled).toBe(false);
    expect(states.b.config.trade.enabled).toBe(true);
    states.a.config.trade.enabled = true;
    expect(presets[0].config.trade.enabled).toBe(false);
  });

  it('is null when a preset is missing', () => {
    expect(comparePresetStates([preset('vi-2-no-trade', false)], entry, 1)).toBeNull();
  });
});
```
Append to `web/src/engine.test.ts`:
```ts
describe('Engine.loadPreset', () => {
  it('loads a preset with a given seed', async () => {
    const { engine, module } = await setup();
    await engine.loadPreset('ii-2-unit', 42);
    expect(engine.seed).toBe(42);
    expect(engine.presetId).toBe('ii-2-unit');
    expect(module.sims.at(-1)!.seed).toBe(42);
  });
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `(cd web && npm run build)`
Expected: FAIL in `tsc` — cannot find module `./compare-presets`; `loadPreset` expects 1 argument.

- [ ] **Step 3: Implement**

Create `web/src/compare-presets.ts` (Decision 16):
```ts
import type { InitialState } from './engine';
import type { Preset } from './types';

/** A presets-menu entry that opens Compare with two presets side by side at one seed. */
export interface ComparePreset {
  id: string;
  label: string;
  /** World A's preset id. */
  a: string;
  /** World B's preset id. */
  b: string;
}

export const COMPARE_PRESETS: ComparePreset[] = [
  { id: 'vi-2-vs-vi-3', label: 'Indecomposability — VI-2 vs VI-3 (Compare)', a: 'vi-2-no-trade', b: 'vi-3-trade' },
];

/** A and B's setups: the entry's two presets (copied) at `seed`, or null if either preset is missing. */
export function comparePresetStates(
  presets: Preset[],
  entry: ComparePreset,
  seed: number,
): { a: InitialState; b: InitialState } | null {
  const a = presets.find((p) => p.id === entry.a);
  const b = presets.find((p) => p.id === entry.b);
  if (!a || !b) return null;
  return { a: { config: structuredClone(a.config), seed }, b: { config: structuredClone(b.config), seed } };
}
```
`web/src/engine.ts` — replace
```ts
  async loadPreset(id: string): Promise<FieldError[] | null> {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    return this.quiet(() => this.rebuild(structuredClone(preset.config), this.seed, [], { presetId: id }));
```
with
```ts
  /** Rebuilds the world as preset `id`, at `seed` (default: the world's seed). */
  async loadPreset(id: string, seed?: number): Promise<FieldError[] | null> {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    return this.quiet(() => this.rebuild(structuredClone(preset.config), seed ?? this.seed, [], { presetId: id }));
```
`web/src/ui/toolbar.ts` — in `reset()` replace `const s = Number(this.seed.value) >>> 0;` with `const s = this.typedSeed();`, and add before `private sync(): void {`:
```ts
  /** The seed in the seed box (typed, not yet applied), as an unsigned 32-bit integer. */
  typedSeed(): number {
    return Number(this.seed.value) >>> 0;
  }

```
`web/src/ui/rules-panel.ts` — add `import { COMPARE_PRESETS } from '../compare-presets';` before `import type { Engine } from '../engine';`; replace `constructor(private engine: Engine) {` with
```ts
  /** `onCompare` (A's panel only) makes the Compare entries of the presets menu work: it gets the entry's id. */
  constructor(
    private engine: Engine,
    private onCompare?: (id: string) => void,
  ) {
```
and in `presetSection()` replace the select's props and children
```ts
      {
        onchange: async () => {
          this.errors = (await this.engine.loadPreset(select.value)) ?? [];
          if (this.errors.length > 0) this.sync();
          this.renderErrors();
        },
      },
      h('option', { value: '', disabled: true }, 'Custom'),
      ...this.engine.presets.map((p) => h('option', { value: p.id }, `${p.name} — ${p.source}`)),
    );
```
with
```ts
      {
        onchange: async () => {
          const compare = COMPARE_PRESETS.find((c) => `compare:${c.id}` === select.value);
          if (compare) {
            // Not a rule system of this world: the menu goes back to showing the current one.
            this.sync();
            this.onCompare?.(compare.id);
            return;
          }
          this.errors = (await this.engine.loadPreset(select.value)) ?? [];
          if (this.errors.length > 0) this.sync();
          this.renderErrors();
        },
      },
      h('option', { value: '', disabled: true }, 'Custom'),
      ...this.engine.presets.map((p) => h('option', { value: p.id }, `${p.name} — ${p.source}`)),
      this.onCompare
        ? h('optgroup', { label: 'Compare' }, ...COMPARE_PRESETS.map((c) => h('option', { value: `compare:${c.id}` }, c.label)))
        : null,
    );
```
`web/src/main.ts` — add `import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';` after the `./compare/lockstep` import; replace `const rules = new WorldSlot(new RulesPanel(engine), 'switch', 'Rules for');` with
```ts
  const rules = new WorldSlot(
    new RulesPanel(engine, (id) => void openComparePreset(id)),
    'switch',
    'Rules for',
  );
```
and add, just before `async function toggleCompare(): Promise<void> {`:
```ts
  /**
   * A Compare entry of the presets menu: A rebuilds as the entry's first preset and B as its second,
   * both with the seed box's seed, and Compare starts at t = 0 in lockstep (as a `#c=` link does).
   * If Compare is on it is left first, keeping A.
   */
  async function openComparePreset(id: string): Promise<void> {
    const entry = COMPARE_PRESETS.find((c) => c.id === id);
    const states = entry && comparePresetStates(engine.presets, entry, toolbar.typedSeed());
    if (!entry || !states) {
      showNotice(`The comparison ${id} is not available`, 10_000);
      return;
    }
    if (busy) {
      showNotice('Compare is starting or ending; try again in a moment');
      return;
    }
    if (compare) await leaveCompare('A');
    if (busy || compare) {
      showNotice('Compare could not be left; try again in a moment');
      return;
    }
    busy = true;
    syncCompareButton();
    hold(true);
    try {
      engine.setRunning(false);
      const errors = await engine.loadPreset(entry.a, states.a.seed);
      if (errors) throw new Error(fieldErrorsMessage(errors));
      // The address bar no longer describes this world.
      history.replaceState(null, '', location.pathname + location.search);
      await buildCompare(states.b);
    } catch (e) {
      showNotice(`Compare could not start (${errorMessage(e)})`, 10_000);
    } finally {
      busy = false;
      hold(false);
      syncCompareButton();
    }
  }
```
(`openComparePreset` only runs from a menu change, after `main` has defined `hold` and `syncCompareButton` synchronously: no temporal-dead-zone risk. `buildCompare` rethrows after undoing its own work, and the notice reports it.)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: build succeeds; all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/compare-presets.ts web/src/compare-presets.test.ts web/src/ui/rules-panel.ts web/src/ui/toolbar.ts web/src/main.ts web/src/engine.ts web/src/engine.test.ts
git commit -m "Open VI-2 vs VI-3 in Compare from the presets menu" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 10: README, roadmap and full verification

*Needs judgement (prose).* Browser (controller): the full pass below.

**Files:**
- Modify: `README.md`, `docs/roadmap.md`

**Interfaces:**
- Consumes: everything above (names and measured numbers from Task 8's commit).
- Produces: documentation only.

- [ ] **Step 1: README**

After the paragraph that begins "Chapter V: immune and disease bit strings", add:
```markdown
Chapter VI: the indecomposability demonstration and the emergent society's views. Presets
`vi-2-no-trade` and `vi-3-trade` are one society — 500 agents on the sugar and spice landscape that
move and reproduce — without and with trade, and the presets menu's **Indecomposability — VI-2 vs
VI-3 (Compare)** opens them side by side in Compare at the seed box's seed. `vi-1-everything` offers
the book's eighteen views (its description says where each lives). Three more overlays: **Neighbor
network** (Chapter II: each agent → the agents that were its von Neumann neighbors after its last
move, with a direction marker; lists may be one-sided), **Friends network** (Chapter III: each agent →
the up to five culturally closest neighbors it has met, never rechecked; with culture on) and
**Family network** (parent → child; with sex on). The **Lineage** color mode shows Animation III-5's
genealogy: founders grey (the book's black, lightened for the dark grid), founders with children red,
the born green, born parents yellow. Charts gain the **Age histogram** (5-tick bins, while lifetimes
are finite) and **Cultural tags** (the percentage of agents with a 0 at each tag position, while
culture is on); in Compare both are drawn as outlines. Neighbor lists, friends and lineage are views
only: they never change a run and are not exported or shared.
```
and in "### Notes" add (with Task 8's measured numbers if they differ):
```markdown
- `vi-2-no-trade` and `vi-3-trade` were calibrated by measurement (vision 1–5, metabolism 1–2 and
  endowment 65 for both goods; see `presets.rs`): without trade the population dies out on seeds
  1–5 (t = 385–900), with trade it survives (75–181 agents at t = 1000). The separation is narrow
  (endowments 64 and 66 do not separate), and the book's recovery to twice the initial population
  and its 115-year waves are not reproduced by these rules.
- In `vi-1-everything`, the book's spice wealth histogram is shown as the Mean holdings chart's
  spice line, and the Lorenz curve and Gini coefficient use sugar wealth.
```

- [ ] **Step 2: Roadmap**

After the "## Milestone 7b" section add:
```markdown
## Milestone 8: Chapter VI — indecomposability and the emergent society (done)

The indecomposability presets `vi-2-no-trade` / `vi-3-trade` (measured; the separation holds on
seeds 1–5 but the book's recovery and long waves are not reproduced) with a presets-menu entry that
opens them in Compare; the neighbor, friends and family network overlays, the Lineage color mode,
and the age and cultural-tag histograms, so `vi-1-everything` offers the book's eighteen views. Runs
are unchanged. See `docs/superpowers/specs/2026-09-24-chapter-vi-design.md`.

## Next: Other artificial societies

Chapter VI's other model kinds, each its own world type beside the sugarscape:

- **Schelling segregation variant** (animations VI-4 to VI-7): a 50 × 50 torus, 2 000 Red/Blue
  agents, von Neumann neighbors, preferences 25 % / 50 % / uniform 25–50 %, random acceptable
  relocation, residence 80–100 with random-color replacement.
- **Ring World** (VI-8, VI-9): a 150-site ring, capacity 4 growing back at 1, initial sugar uniform
  0–4, 40 agents, vision 15–30 looking counterclockwise, moving to the nearest maximum-sugar
  unoccupied site, in random order, and a megagroup start.
```
and replace the Experiments bullet
`- **Chapter VI "artificial history" presets**: the book's culminating combined rule systems, and its flocking / group-formation asides (Animation VI-8).`
with
`- **Chapter VI "artificial history" presets**: done (Milestone 8); the flocking / group-formation aside (Animation VI-8) moves to *Other artificial societies*.`

- [ ] **Step 3: Full verification**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test book -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
git diff main -- crates/sugarscape-core/tests/fixtures crates/sugarscape-core/tests/legacy.rs
```
Expected: everything PASSES (the book suite includes `trade_decides_whether_the_society_survives`; `measure_indecomposability` just prints), and the last command prints nothing (legacy fixtures untouched). `git diff main -- crates/sugarscape-core/tests/golden.rs` shows only the two added entries and their comment.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md
git commit -m "Document Chapter VI and plan other artificial societies" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

The controller then runs the full puppeteer pass (implementers don't): every existing scenario (rules, tools, painting, image import, inspection, charts, share links, sessions, exports, disease, credit tab, groups, trails, Experiments, Compare with Keep A/B, recording, the worker determinism check `0x75b93943813545e4`); every scenario in Tasks 6, 7 and 9; the VI-2 vs VI-3 Compare entry at Max to t = 1000 (A extinct, B alive); each new overlay on `vi-1-everything` and on a Chapter III preset; Lineage colors; both histograms, also in Compare; and the performance check — a 200 × 200 world with 2 000 agents at **Max** for 20 s, once as set up and once with culture on and all three new overlays ticked, with a `PerformanceObserver({ type: 'longtask', buffered: true })`: no main-thread task over 50 ms and a grid redraw rate near 30 per second.

---

## Coverage check (spec → task)

| Spec requirement | Task |
|---|---|
| Neighbor lists after M, kept until next move, asymmetric | 1 (record), 2 (edges) |
| Friends: fill to 5, strictly closer replaces farthest, ties, earliest-added, never rechecked, dropped on death, empty with culture off | 1 |
| Family network, lineage classes, Lineage color mode | 2 (core), 6 (page) |
| Core API `neighbor_edges`/`friend_edges`/`family_edges`/`lineage`, no RNG | 2 |
| Age histogram (5-tick bins to the largest maximum lifetime), tag histogram (% zeros per position) | 3, 4, 5, 7 |
| WASM edge lists as quadruples; overlay list `trade … family` | 4, 5 |
| Checkboxes with availability, direction marker, torus splitting, Compare per grid | 5, 6 |
| `ageHist`/`tagHist` through `wants`, only when visible and stale; Compare step outlines | 5, 7 |
| `vi-2-no-trade` / `vi-3-trade`, differ only in trade, measured calibration recorded | 8 |
| Compare entry, seed box's seed, t = 0 lockstep | 9 |
| VI-1 eighteen views in its description | 8 |
| Golden unchanged + two new entries; fingerprint invariance test | 1, 2, 3, 8 |
| Core unit tests listed in the spec | 1, 2, 3 |
| Book-style test | 8 |
| Web tests: overlay wants, torus splitting, lineage colors, histogram visibility, paused quiet | 5, 6, 7 |
| README, roadmap with "Other artificial societies" | 10 |
| Max-speed performance | 1 (CLI), 10 (browser) |
