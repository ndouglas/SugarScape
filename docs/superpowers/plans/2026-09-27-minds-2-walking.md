# Minds 2: A* and walking — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- A verified A* engine;
- walking and walls in the Sugarscape;
- measurements of which of the book's results need rule M's jump, and of the ideal free distribution when switching patches truly costs travel.

**Architecture:**
- `minds/astar.rs` is a generic A* over a small `Graph` trait.
- `minds/grid.rs` supplies the grids it searches: a 4-way torus for the Sugarscape, and a flat 8-way octile map for Sturtevant's benchmarks.
- Walls are a per-site mask in `World`. `is_occupied` is true on them, sight rays stop at opaque ones, they hold no sugar, and diffusion skips them.
- `movement::arrive` either jumps (the book) or walks `speed` steps along an A* path. Rule M and the utility mind both end in it.
- Each Flump keeps an observational `Plan` (target and path) for Inspect and the page's path line.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), the standalone `survey/` crate, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-09-27-minds-2-walking-design.md`

## Global Constraints

- Every existing golden entry, legacy fixture and pinned fingerprint stays green and **unedited**.
- `movement.mode: jump` and `walls: []` are the defaults, and they draw nothing new from `World.rng`.
- With no walls, sight, placement, "is this site free" and diffusion are exactly today's: the same lists, in the same order, with the same arithmetic.
- At vision 1, `walk` gives the same fingerprint as `jump` on every golden Sugarscape preset without combat.
- A* draws no random numbers. Ties go to the lowest f, then the lowest h, then the earliest pushed. The torus is pushed in N, S, E, W order (`geometry::DIRECTIONS`).
- A* expands at most 4 096 sites for a walking Flump. Past that the target counts as unreachable, and the Flump stays and gathers where it is.
- `walk` with combat on is a validation error on `movement.mode`: "rule C jumps; walking combat isn't defined".
- `walls` changes only on reset. `movement.mode` and `movement.speed` are live. `speed` is 1–50.
- A Flump's `Plan` is observational, like `social` and trails: never hashed, exported or shared.
- `sqrt` is the only irrational operation, and it appears only in the 8-way benchmark grid. No `f64::ln`/`exp` anywhere new.
- American spelling. Titles follow `titles.rs`'s rules.
- Every commit message ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- Work in `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-2` (branch `minds-2`).

## Review Focus

1. **Configs without `movement` or `walls`** (old links, sessions, saved configs) load as the book. A schedule may set `movement.mode` and `movement.speed` mid-run but not `walls`. Owned by Task 1.
2. **A Flump boxed in by walls, or whose target is unreachable within the expansion limit,** stays and gathers where it is. It must not panic or wander through a wall. Owned by Task 4.
3. **A placement rectangle partly covered by walls** places Flumps only on free sites. A population larger than the free sites is a validation error, not a panic. Owned by Task 3 (placement) and Task 1 (validation).
4. **Placing a Flump on a wall by edit or shot** is refused with a clear error. Painting capacity onto a wall leaves it at 0. Owned by Task 3.
5. **A planned path that crosses the torus seam** draws as separate segments, not a line across the whole grid. Owned by Task 8.

---

## File structure

- Modify `crates/sugarscape-core/src/config.rs`: `Movement`, `MoveMode`, `Wall`, the fields, validation and reset-only rules. Modify `legacy.rs` for the defaults.
- Create `crates/sugarscape-core/src/minds/astar.rs`: `Graph`, `Search` and `astar`.
- Create `crates/sugarscape-core/src/minds/grid.rs`: `TorusGrid` (4-way, with a passability closure) and `OctileMap` (the benchmarks).
- Modify `crates/sugarscape-core/src/minds/mod.rs`: `pub mod astar; pub mod grid;`.
- Create `crates/sugarscape-core/tests/astar.rs`: property tests against Dijkstra, and the benchmark scenarios. The fixtures are already committed at `crates/sugarscape-core/tests/fixtures/movingai/`.
- Modify `crates/sugarscape-core/src/geometry.rs`: `Torus::sight_until`.
- Modify `crates/sugarscape-core/src/world.rs`: the wall mask, `is_wall`/`is_opaque`/`has_walls`, `is_occupied`, `empty_sites`, `insert_agent`, capacities, placement, `sight`, `landscape_edited`.
- Modify `crates/sugarscape-core/src/rules/pollution.rs` (diffusion), `rules/combat.rs` (sight), `rules/movement.rs` (sight, `arrive`), `minds/utility.rs` (`arrive`), `edit.rs` (capacity painting, `AgentView.plan`), `render.rs` (wall and fence colors), and `agent.rs` (`Plan`).
- Modify `crates/sugarscape-core/tests/minds.rs`: the vision-1 walk reduction.
- Modify `presets.rs`, `titles.rs`, `tests/golden.rs`: eight presets.
- Create `sweeps/walk-speed.json`, `sweeps/walk-vision.json`, `sweeps/ifd-detour.json`; modify `sweep.rs`.
- Modify `crates/sugarscape-wasm/tests/web.rs`: the pin and the catalog lists.
- Modify `web/src/types.ts`, `web/src/schema.ts`, `web/src/ui/inspect-panel.ts`, `web/src/ui/grid-view.ts`; tests in `web/src/schema.test.ts` and `web/src/ui/plan-path.test.ts` (with `web/src/ui/plan-path.ts`).
- Create `survey/src/claims/minds2.rs`; modify `survey/src/claims/mod.rs`.
- Docs: `README.md`, `docs/studies/2026-09-27-minds.md`, `docs/roadmap.md`, and the spec's amendments.

---

### Task 1: The `movement` and `walls` config

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs` (types after `Decision`; `Config`, `Default`; `RESET_ONLY_PATHS`; `reset_only`; `validate_fields`; `structural_changes`; tests)
- Modify: `crates/sugarscape-core/src/legacy.rs` (the `Config { … }` literal)

**Interfaces:**
- Produces:
  - `config::{Movement, MoveMode, Wall}`;
  - `Config.movement: Movement` (`Copy`; default `{ mode: Jump, speed: 1 }`);
  - `Config.walls: Vec<Wall>` (`Wall { x: u32, y: u32, width: u32, height: u32, opaque: bool }`, `Copy`);
  - serde names `"jump" | "walk"`;
  - `Config::free_sites(&self) -> usize`, which counts sites not covered by any wall.

- [ ] **Step 1: Write the failing tests** (append to `config.rs`'s tests)

```rust
    #[test]
    fn movement_and_walls_default_to_the_book_and_older_configs_load() {
        let d = Config::default();
        assert_eq!(d.movement, Movement { mode: MoveMode::Jump, speed: 1 });
        assert!(d.walls.is_empty());
        let mut v = serde_json::to_value(Config::default()).unwrap();
        let o = v.as_object_mut().unwrap();
        o.remove("movement");
        o.remove("walls");
        let c = Config::from_value(v).unwrap();
        assert_eq!((c.movement, c.walls.len()), (Movement::default(), 0));
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v["movement"] = serde_json::json!({ "mode": "walk" });
        assert_eq!(Config::from_value(v).unwrap().movement, Movement { mode: MoveMode::Walk, speed: 1 });
    }

    #[test]
    fn movement_and_walls_are_validated() {
        let with = |f: &dyn Fn(&mut Config)| {
            let mut c = Config::default();
            f(&mut c);
            fields(c.validate())
        };
        assert!(with(&|c| c.movement.speed = 50).is_empty());
        assert_eq!(with(&|c| c.movement.speed = 0), ["movement.speed"]);
        assert_eq!(with(&|c| c.movement.speed = 51), ["movement.speed"]);
        assert_eq!(
            with(&|c| {
                c.movement.mode = MoveMode::Walk;
                c.combat.enabled = true;
            }),
            ["movement.mode"]
        );
        let wall = |x, y, width, height| Wall { x, y, width, height, opaque: true };
        assert!(with(&|c| c.walls = vec![wall(0, 0, 50, 1)]).is_empty());
        assert_eq!(with(&|c| c.walls = vec![wall(45, 0, 6, 1)]), ["walls.0"]); // past the edge
        assert_eq!(with(&|c| c.walls = vec![wall(0, 0, 0, 1)]), ["walls.0"]); // empty
        assert_eq!(with(&|c| c.walls = vec![wall(0, 0, 50, 50)]), ["walls"]); // every site
        // 400 Flumps need 400 free sites: 50×50 minus a 50×43 block leaves 350.
        assert_eq!(with(&|c| c.walls = vec![wall(0, 0, 50, 43)]), ["population"]);
    }

    #[test]
    fn free_sites_count_each_walled_site_once() {
        let mut c = Config::default();
        let wall = |x, y, width, height| Wall { x, y, width, height, opaque: false };
        assert_eq!(c.free_sites(), 2500);
        c.walls = vec![wall(0, 0, 10, 1), wall(5, 0, 10, 2)]; // overlapping
        assert_eq!(c.free_sites(), 2500 - (15 + 10));
    }

    #[test]
    fn walls_change_only_on_reset_and_movement_lives() {
        let a = Config::default();
        let mut b = a.clone();
        b.walls = vec![Wall { x: 1, y: 1, width: 1, height: 1, opaque: true }];
        let f: Vec<String> = a.structural_changes(&b).into_iter().map(|e| e.field).collect();
        assert_eq!(f, ["walls"]);
        let mut b = a.clone();
        b.movement = Movement { mode: MoveMode::Walk, speed: 3 };
        assert!(a.structural_changes(&b).is_empty());
        for (path, value) in [
            ("walls", serde_json::json!([])),
            ("walls.0", serde_json::json!({ "x": 0, "y": 0, "width": 1, "height": 1, "opaque": true })),
        ] {
            let c = Config { schedule: vec![change(5, path, value)], ..Default::default() };
            let errs = c.validate().unwrap_err();
            assert!(errs[0].message.contains("only on reset"), "{path}: {errs:?}");
        }
        let c = Config {
            schedule: vec![
                change(5, "movement.mode", serde_json::json!("walk")),
                change(6, "movement.speed", serde_json::json!(4)),
            ],
            ..Default::default()
        };
        c.validate().unwrap();
    }
```

(`"walls.0"` on a config with no walls may fail as an unknown path before the reset-only check. If so, assert only that it's refused, and keep the `"walls"` case asserting "only on reset".)

- [ ] **Step 2: Run the tests and check they fail**

Run: `cargo test -p sugarscape-core --lib config::tests -- movement walls free_sites`
Expected: compile errors.

- [ ] **Step 3: Implement.** After `Decision`:

```rust
/// Minds 2: how an agent reaches the site its decision picked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveMode {
    /// Rule M: arrive in one tick, whatever the distance.
    #[default]
    Jump,
    /// Take `speed` steps along an A* path toward it.
    Walk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Movement {
    pub mode: MoveMode,
    /// Steps per tick under `walk` (1–50).
    pub speed: u32,
}

impl Default for Movement {
    fn default() -> Self {
        Self { mode: MoveMode::Jump, speed: 1 }
    }
}

/// Minds 2: a rectangle of wall sites. Every site in it is impassable and
/// holds nothing; `opaque` walls also stop sight (a wall), others don't (a
/// fence).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wall {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub opaque: bool,
}
```

- Add `pub movement: Movement,` and `pub walls: Vec<Wall>,` to `Config` after `decision`.
- Add `movement: Movement::default(), walls: Vec::new(),` to `Default` and to `legacy.rs`'s literal.

`free_sites`:

```rust
    /// Sites not covered by any wall (Minds 2).
    pub fn free_sites(&self) -> usize {
        let (w, h) = (self.width as usize, self.height as usize);
        if self.walls.is_empty() {
            return w * h;
        }
        let mut walled = vec![false; w * h];
        for wall in &self.walls {
            for y in wall.y..wall.y.saturating_add(wall.height).min(self.height) {
                for x in wall.x..wall.x.saturating_add(wall.width).min(self.width) {
                    walled[y as usize * w + x as usize] = true;
                }
            }
        }
        walled.iter().filter(|&&b| !b).count()
    }
```

In `validate_fields`, after the decision checks:

```rust
        e.check(
            (1..=50).contains(&self.movement.speed),
            "movement.speed",
            "must be between 1 and 50",
        );
        e.check(
            !(self.movement.mode == MoveMode::Walk && self.combat.enabled),
            "movement.mode",
            "rule C jumps; walking combat isn't defined",
        );
        for (i, wall) in self.walls.iter().enumerate() {
            e.check(
                wall.width >= 1
                    && wall.height >= 1
                    && wall.x.saturating_add(wall.width) <= self.width
                    && wall.y.saturating_add(wall.height) <= self.height,
                &format!("walls.{i}"),
                "must be a nonempty rectangle inside the grid",
            );
        }
        if !self.walls.is_empty() {
            let free = self.free_sites();
            e.check(free > 0, "walls", "walls may not cover every site");
            e.check(
                free == 0 || self.population as usize <= free,
                "population",
                format!("must be at most the {free} sites not covered by walls"),
            );
        }
```

(If `Errors::check` takes `&str`, pass `&format!(…)` as shown. Keep the per-wall check before `free_sites` so an out-of-grid wall can't index out of bounds: `free_sites` clamps with `min`.)

Reset-only:
- `RESET_ONLY_PATHS` grows by `"walls"`.
- In `reset_only`'s pattern, add `| ["walls", ..]`.
- In `structural_changes`, add `if self.walls != next.walls { out.push(FieldError::new("walls", msg)); }`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --lib config && cargo test -p sugarscape-core --test legacy --test golden`
Expected: pass, with golden and legacy unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/config.rs crates/sugarscape-core/src/legacy.rs
git commit -m "Minds 2: the movement (jump or walk, speed) and walls config

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 2: The A* engine and its grids, verified

**Files:**
- Create: `crates/sugarscape-core/src/minds/astar.rs`
- Create: `crates/sugarscape-core/src/minds/grid.rs`
- Modify: `crates/sugarscape-core/src/minds/mod.rs` (`pub mod astar; pub mod grid;`)
- Create: `crates/sugarscape-core/tests/astar.rs`
- Uses: `crates/sugarscape-core/tests/fixtures/movingai/` (committed, with a README)

**Interfaces:**
- Produces:
  - `minds::astar::{Graph, Search, astar}`;
  - `minds::grid::{TorusGrid, OctileMap}`.
- Signatures:
  - `trait Graph { fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>); fn heuristic(&self, n: usize, goal: usize) -> f64; }`
  - `struct Search { pub path: Vec<usize> /* start ..= goal */, pub cost: f64, pub expanded: usize }`
  - `fn astar<G: Graph>(g: &G, start: usize, goal: usize, limit: usize) -> Option<Search>`
  - `TorusGrid::new(torus: Torus, passable: F) where F: Fn(Pos) -> bool`
  - `OctileMap::parse(text: &str) -> Result<OctileMap, String>`
  - `OctileMap::{width, height, index(x, y), passable(x, y)}`

- [ ] **Step 1: Write the failing tests.** Create `tests/astar.rs`:

```rust
//! Minds 2's A*: verified against Dijkstra on random grids and against
//! Sturtevant's published optimal lengths (Moving AI benchmarks, ODC-By;
//! see tests/fixtures/movingai/README.md).

use std::collections::BinaryHeap;

use rand::{Rng, SeedableRng};
use sugarscape_core::geometry::{Pos, Torus};
use sugarscape_core::minds::astar::{astar, Graph};
use sugarscape_core::minds::grid::{OctileMap, TorusGrid};

/// Dijkstra's optimal cost, the oracle (no heuristic, no ties to break).
fn dijkstra<G: Graph>(g: &G, start: usize, goal: usize, nodes: usize) -> Option<f64> {
    #[derive(PartialEq)]
    struct Item(f64, usize);
    impl Eq for Item {}
    impl PartialOrd for Item {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(o)) }
    }
    impl Ord for Item {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering { o.0.total_cmp(&self.0) }
    }
    let mut dist = vec![f64::INFINITY; nodes];
    let mut heap = BinaryHeap::new();
    let mut out = Vec::new();
    dist[start] = 0.0;
    heap.push(Item(0.0, start));
    while let Some(Item(d, n)) = heap.pop() {
        if n == goal {
            return Some(d);
        }
        if d > dist[n] {
            continue;
        }
        out.clear();
        g.neighbors(n, &mut out);
        for &(m, c) in &out {
            if d + c < dist[m] {
                dist[m] = d + c;
                heap.push(Item(d + c, m));
            }
        }
    }
    None
}

fn random_mask(rng: &mut impl Rng, len: usize, density: f64) -> Vec<bool> {
    (0..len).map(|_| rng.gen::<f64>() >= density).collect()
}

#[test]
fn astar_matches_dijkstra_on_random_walled_tori() {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(20260927);
    for case in 0..1000 {
        let (w, h) = (rng.gen_range(3..30u32), rng.gen_range(3..30u32));
        let torus = Torus::new(w, h);
        let density = f64::from(case % 5) * 0.1; // 0–40 %
        let open = random_mask(&mut rng, (w * h) as usize, density);
        let grid = TorusGrid::new(torus, |p: Pos| open[torus.index(p)]);
        let (s, t) = (rng.gen_range(0..w * h) as usize, rng.gen_range(0..w * h) as usize);
        if !open[s] || !open[t] {
            continue;
        }
        let want = dijkstra(&grid, s, t, (w * h) as usize);
        let got = astar(&grid, s, t, usize::MAX);
        match (want, got) {
            (None, None) => {}
            (Some(c), Some(found)) => {
                assert_eq!(found.cost, c, "case {case}");
                assert_eq!(found.path.len() as f64 - 1.0, c, "case {case}: unit steps");
                assert_eq!((found.path[0], *found.path.last().unwrap()), (s, t));
                for pair in found.path.windows(2) {
                    let (a, b) = (torus.pos(pair[0]), torus.pos(pair[1]));
                    assert!(torus.neighbors(a).contains(&b), "case {case}: a real step");
                    assert!(open[pair[1]], "case {case}: through open sites only");
                }
            }
            (w, g) => panic!("case {case}: dijkstra {w:?}, astar {:?}", g.map(|s| s.cost)),
        }
    }
}

#[test]
fn astar_matches_dijkstra_on_random_octile_maps() {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(7);
    for case in 0..1000 {
        let (w, h) = (rng.gen_range(2..40usize), rng.gen_range(2..40usize));
        let density = (case % 5) as f64 * 0.1;
        let rows: Vec<String> = (0..h)
            .map(|_| (0..w).map(|_| if rng.gen::<f64>() < density { '@' } else { '.' }).collect())
            .collect();
        let text = format!("type octile\nheight {h}\nwidth {w}\nmap\n{}\n", rows.join("\n"));
        let map = OctileMap::parse(&text).unwrap();
        let (sx, sy, tx, ty) = (rng.gen_range(0..w), rng.gen_range(0..h), rng.gen_range(0..w), rng.gen_range(0..h));
        if !map.passable(sx, sy) || !map.passable(tx, ty) {
            continue;
        }
        let (s, t) = (map.index(sx, sy), map.index(tx, ty));
        let want = dijkstra(&map, s, t, w * h);
        let got = astar(&map, s, t, usize::MAX).map(|f| f.cost);
        match (want, got) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!((a - b).abs() < 1e-9, "case {case}: {a} vs {b}"),
            other => panic!("case {case}: {other:?}"),
        }
    }
}

fn scenarios(map_file: &str, scen_file: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/movingai/");
    let map = OctileMap::parse(&std::fs::read_to_string(format!("{dir}{map_file}")).unwrap()).unwrap();
    let scen = std::fs::read_to_string(format!("{dir}{scen_file}")).unwrap();
    let mut n = 0;
    for line in scen.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        let num = |i: usize| f[i].parse::<usize>().unwrap();
        let optimal: f64 = f[8].parse().unwrap();
        let (s, t) = (map.index(num(4), num(5)), map.index(num(6), num(7)));
        let found = astar(&map, s, t, usize::MAX).unwrap_or_else(|| panic!("{line}: no path"));
        assert!((found.cost - optimal).abs() < 1e-6, "{line}: A* {} vs optimal {optimal}", found.cost);
        n += 1;
    }
    assert!(n >= 80, "{scen_file}: {n} scenarios");
}

#[test]
fn astar_meets_sturtevants_random_benchmark() {
    scenarios("random512-10-0.map", "random512-10-0.map.scen");
}

#[test]
fn astar_meets_sturtevants_maze_benchmark() {
    scenarios("maze512-4-0.map", "maze512-4-0.map.scen");
}
```

Adjust the crate names (`rand_pcg`, `rand`) to what `sugarscape-core` depends on (check `Cargo.toml`; `crate::rng` wraps `Pcg64Mcg`). If `Torus::index` or `pos` isn't `pub`, make them `pub`. If `geometry` isn't a public module, it is (`pub mod geometry`).

Unit tests inside `astar.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Pos, Torus};
    use crate::minds::grid::TorusGrid;

    #[test]
    fn start_is_goal() {
        let t = Torus::new(5, 5);
        let g = TorusGrid::new(t, |_| true);
        let s = astar(&g, 7, 7, 10).unwrap();
        assert_eq!((s.path, s.cost, s.expanded), (vec![7], 0.0, 0));
    }

    #[test]
    fn unreachable_and_over_the_limit_give_no_path() {
        let t = Torus::new(5, 5);
        // The goal (2, 2) is walled in on all four sides.
        let walls = [Pos::new(2, 1), Pos::new(2, 3), Pos::new(1, 2), Pos::new(3, 2)];
        let g = TorusGrid::new(t, |p| !walls.contains(&p));
        assert!(astar(&g, t.index(Pos::new(0, 0)), t.index(Pos::new(2, 2)), usize::MAX).is_none());
        let open = TorusGrid::new(t, |_| true);
        assert!(astar(&open, t.index(Pos::new(0, 0)), t.index(Pos::new(2, 2)), 1).is_none());
    }

    #[test]
    fn ties_follow_the_stated_order_and_nothing_is_expanded_twice() {
        // On an open 9×9 torus from (4, 4) to (6, 6), every shortest path
        // has 4 steps; the stated order (f, then h, then push order N, S, E,
        // W) picks the path that goes south first, then east.
        let t = Torus::new(9, 9);
        let g = TorusGrid::new(t, |_| true);
        let s = astar(&g, t.index(Pos::new(4, 4)), t.index(Pos::new(6, 6)), usize::MAX).unwrap();
        let path: Vec<Pos> = s.path.iter().map(|&i| t.pos(i)).collect();
        assert_eq!(path.len(), 5);
        assert_eq!(path[1], Pos::new(4, 5), "south first (pushed before east at equal f and h)");
        assert!(s.expanded <= 81, "each site at most once");
    }
}
```

The tie test's expected first step follows from the stated rule: south and east have equal f and h, and south is pushed first. If a correct implementation of the stated rule gives a different path, the rule is the authority. Check it by hand, and fix the test only if the expectation was derived wrongly, not the rule.

- [ ] **Step 2: Run the tests and check they fail**

Run: `cargo test -p sugarscape-core --test astar; cargo test -p sugarscape-core --lib minds::astar`
Expected: compile errors.

- [ ] **Step 3: Implement `astar.rs`**

```rust
//! A* (Hart, Nilsson & Raphael 1968) over any graph with an admissible,
//! consistent heuristic (Minds 2). Deterministic: ties go to the lowest f,
//! then the lowest h, then the earliest pushed; it draws no random numbers.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// A graph A* can search. Nodes are indices.
pub trait Graph {
    /// Appends `n`'s neighbors and step costs to `out`, in the graph's
    /// fixed order (the tie rule's push order).
    fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>);
    /// A lower bound on the cost from `n` to `goal` (admissible) that never
    /// drops by more than a step's cost (consistent).
    fn heuristic(&self, n: usize, goal: usize) -> f64;
}

/// A found path: `start ..= goal`, its cost, and how many nodes were
/// expanded to find it.
#[derive(Clone, Debug, PartialEq)]
pub struct Search {
    pub path: Vec<usize>,
    pub cost: f64,
    pub expanded: usize,
}

struct Open {
    f: f64,
    h: f64,
    seq: u64,
    node: usize,
    g: f64,
}

impl PartialEq for Open {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for Open {}
impl PartialOrd for Open {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Open {
    /// Reversed, so `BinaryHeap` pops the lowest f, then h, then seq.
    fn cmp(&self, o: &Self) -> Ordering {
        o.f.total_cmp(&self.f)
            .then(o.h.total_cmp(&self.h))
            .then(o.seq.cmp(&self.seq))
    }
}

/// The minimum-cost path from `start` to `goal`, or `None` when there is
/// none or finding it would expand more than `limit` nodes.
pub fn astar<G: Graph>(g: &G, start: usize, goal: usize, limit: usize) -> Option<Search> {
    if start == goal {
        return Some(Search { path: vec![start], cost: 0.0, expanded: 0 });
    }
    let mut best: HashMap<usize, (f64, usize)> = HashMap::new(); // g, parent
    let mut closed: HashMap<usize, ()> = HashMap::new();
    let mut open = BinaryHeap::new();
    let mut seq = 0;
    let h0 = g.heuristic(start, goal);
    best.insert(start, (0.0, start));
    open.push(Open { f: h0, h: h0, seq, node: start, g: 0.0 });
    let mut out = Vec::new();
    let mut expanded = 0;
    while let Some(Open { node, g: gn, .. }) = open.pop() {
        if closed.contains_key(&node) || gn > best[&node].0 {
            continue;
        }
        if node == goal {
            let mut path = vec![goal];
            let mut at = goal;
            while at != start {
                at = best[&at].1;
                path.push(at);
            }
            path.reverse();
            return Some(Search { path, cost: gn, expanded });
        }
        if expanded == limit {
            return None;
        }
        expanded += 1;
        closed.insert(node, ());
        out.clear();
        g.neighbors(node, &mut out);
        for &(m, c) in &out {
            let gm = gn + c;
            if closed.contains_key(&m) || best.get(&m).is_some_and(|&(b, _)| b <= gm) {
                continue;
            }
            best.insert(m, (gm, node));
            let h = g.heuristic(m, goal);
            seq += 1;
            open.push(Open { f: gm + h, h, seq, node: m, g: gm });
        }
    }
    None
}
```

A `HashMap` isn't iterated here (only looked up), so its randomized order can't leak into results. Use `HashSet` for `closed` if you prefer; the behavior is the same.

`grid.rs`:

```rust
//! The grids Minds 2's A* searches: the Sugarscape's 4-way torus, and the
//! flat 8-way octile maps of Sturtevant's benchmarks.

use crate::geometry::{Pos, Torus};
use crate::minds::astar::Graph;

/// The 4-way torus; `passable` decides which sites a path may enter.
/// Neighbors come in `geometry::DIRECTIONS` order (N, S, E, W); each step
/// costs 1; the heuristic is torus Manhattan distance.
pub struct TorusGrid<F: Fn(Pos) -> bool> {
    torus: Torus,
    passable: F,
}

impl<F: Fn(Pos) -> bool> TorusGrid<F> {
    pub fn new(torus: Torus, passable: F) -> Self {
        Self { torus, passable }
    }
}

impl<F: Fn(Pos) -> bool> Graph for TorusGrid<F> {
    fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>) {
        for q in self.torus.neighbors(self.torus.pos(n)) {
            if (self.passable)(q) {
                out.push((self.torus.index(q), 1.0));
            }
        }
    }

    fn heuristic(&self, n: usize, goal: usize) -> f64 {
        let (a, b) = (self.torus.pos(n), self.torus.pos(goal));
        let d = |p: u32, q: u32, len: u32| {
            let d = p.abs_diff(q);
            d.min(len - d)
        };
        f64::from(d(a.x, b.x, self.torus.width) + d(a.y, b.y, self.torus.height))
    }
}

/// A Moving AI octile map: 8-way, flat (not a torus), diagonals cost √2 and
/// may not cut a corner (both orthogonal neighbors must be passable). `.`,
/// `G` and `S` are passable.
pub struct OctileMap {
    width: usize,
    height: usize,
    open: Vec<bool>,
}

impl OctileMap {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        let mut header = |key: &str| -> Result<String, String> {
            let line = lines.next().ok_or("truncated header")?;
            line.strip_prefix(key).map(|v| v.trim().to_string()).ok_or(format!("expected {key}"))
        };
        header("type")?;
        let height: usize = header("height")?.parse().map_err(|_| "bad height")?;
        let width: usize = header("width")?.parse().map_err(|_| "bad width")?;
        header("map")?;
        let mut open = Vec::with_capacity(width * height);
        for row in lines.by_ref().take(height) {
            let row: Vec<char> = row.chars().collect();
            if row.len() < width {
                return Err("short row".into());
            }
            open.extend(row[..width].iter().map(|c| matches!(c, '.' | 'G' | 'S')));
        }
        if open.len() != width * height {
            return Err("too few rows".into());
        }
        Ok(Self { width, height, open })
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn index(&self, x: usize, y: usize) -> usize { y * self.width + x }
    pub fn passable(&self, x: usize, y: usize) -> bool { self.open[self.index(x, y)] }
}

impl Graph for OctileMap {
    fn neighbors(&self, n: usize, out: &mut Vec<(usize, f64)>) {
        let (x, y) = ((n % self.width) as i64, (n / self.width) as i64);
        let ok = |x: i64, y: i64| {
            x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
                && self.passable(x as usize, y as usize)
        };
        for (dx, dy) in [(0, -1), (0, 1), (1, 0), (-1, 0), (1, -1), (1, 1), (-1, 1), (-1, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !ok(nx, ny) {
                continue;
            }
            let diagonal = dx != 0 && dy != 0;
            if diagonal && !(ok(x + dx, y) && ok(x, y + dy)) {
                continue; // no corner cutting
            }
            let cost = if diagonal { std::f64::consts::SQRT_2 } else { 1.0 };
            out.push((self.index(nx as usize, ny as usize), cost));
        }
    }

    fn heuristic(&self, n: usize, goal: usize) -> f64 {
        let (dx, dy) = ((n % self.width).abs_diff(goal % self.width), (n / self.width).abs_diff(goal / self.width));
        let (lo, hi) = (dx.min(dy) as f64, dx.max(dy) as f64);
        hi + (std::f64::consts::SQRT_2 - 1.0) * lo
    }
}
```

Adapt `Torus`'s field names (`width`/`height`) and the visibility of `index`/`pos` to the real ones.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --test astar --release && cargo test -p sugarscape-core --lib minds`
Expected: all pass. The benchmark tests search 512×512 maps, so run them in release if debug takes more than about a minute.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/minds crates/sugarscape-core/tests/astar.rs crates/sugarscape-core/src/geometry.rs
git commit -m "Minds 2: A* over any graph, verified against Dijkstra and Sturtevant's benchmarks

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 3: Walls in the world

**Files:**
- Modify: `crates/sugarscape-core/src/geometry.rs` (`Torus::sight_until`)
- Modify: `crates/sugarscape-core/src/world.rs` (a `walls: Vec<u8>` mask where 0 is free, 1 a fence and 2 opaque; `has_walls`, `is_wall`, `is_opaque`, `sight`; `is_occupied`, `empty_sites`, `insert_agent`, `with_landscapes`, `populate`, `landscape_edited`)
- Modify: `crates/sugarscape-core/src/rules/movement.rs` (`candidates` uses `world.sight`), `rules/combat.rs` (both `sight` calls use `world.sight`), `rules/pollution.rs` (`diffuse`), `edit.rs` (painting capacity), `render.rs` (colors)

**Interfaces:**
- Consumes: `Config.walls`, `Wall` (Task 1).
- Produces:
  - `World::has_walls(&self) -> bool`, `World::is_wall(&self, p: Pos) -> bool`, `World::is_opaque(&self, p: Pos) -> bool`;
  - `World::sight(&self, p: Pos, vision: u32) -> Vec<(Pos, u32)>`;
  - `Torus::sight_until(&self, p, vision, blocked: impl Fn(Pos) -> bool) -> Vec<(Pos, u32)>`.

- [ ] **Step 1: Write the failing tests** (in `world.rs` tests; use `testkit` helpers where they exist)

```rust
    fn walled(walls: Vec<crate::config::Wall>) -> World {
        let mut c = crate::testkit::blank_config(11, 11);
        c.walls = walls;
        World::new(c, 7).unwrap()
    }
    fn wall(x: u32, y: u32, width: u32, height: u32, opaque: bool) -> crate::config::Wall {
        crate::config::Wall { x, y, width, height, opaque }
    }

    #[test]
    fn with_no_walls_sight_is_the_toruss() {
        let w = walled(vec![]);
        for v in [1, 3, 5, 6, 10] {
            assert_eq!(w.sight(Pos::new(5, 5), v), w.torus.sight(Pos::new(5, 5), v));
            assert_eq!(w.sight(Pos::new(0, 10), v), w.torus.sight(Pos::new(0, 10), v));
        }
        assert!(!w.has_walls());
    }

    #[test]
    fn opaque_walls_stop_sight_and_fences_do_not() {
        let w = walled(vec![wall(5, 2, 1, 1, true), wall(7, 5, 1, 1, false)]);
        let seen: Vec<Pos> = w.sight(Pos::new(5, 5), 4).into_iter().map(|s| s.0).collect();
        assert!(seen.contains(&Pos::new(5, 3)));
        assert!(!seen.contains(&Pos::new(5, 2)), "the wall itself isn't a sight line site");
        assert!(!seen.contains(&Pos::new(5, 1)), "behind the wall");
        assert!(seen.contains(&Pos::new(7, 5)) && seen.contains(&Pos::new(8, 5)), "a fence doesn't block sight");
    }

    #[test]
    fn walls_hold_nothing_and_nobody() {
        let mut w = walled(vec![wall(3, 3, 2, 2, false)]);
        let p = Pos::new(3, 3);
        assert!(w.is_wall(p) && w.is_occupied(p) && w.occupant(p).is_none());
        assert_eq!((w.site(p).capacity[0], w.site(p).resource[0]), (0.0, 0.0));
        assert!(!w.empty_sites().contains(&p));
        let mut a = crate::testkit::agent_at(&w, 3, 3); // or build via testkit::spawn's Agent literal
        a.pos = p;
        assert!(w.insert_agent(a).unwrap_err().contains("wall"));
        crate::rules::growback::grow(&mut w); // or the growback entry point
        assert_eq!(w.site(p).resource[0], 0.0);
    }

    #[test]
    fn placement_skips_walls_and_is_unchanged_without_them() {
        let mut c = crate::config::Config::default();
        c.walls = vec![wall(0, 0, 50, 40, true)];
        c.population = 400;
        let w = World::new(c, 3).unwrap();
        assert_eq!(w.population(), 400);
        assert!(w.agents().all(|a| a.pos.y >= 40));
    }
```

- Add a test that **diffusion** averages over non-wall neighbors: a site next to one wall gets the sum of its three open neighbors over 3, and wall sites stay 0. Also check the no-wall arithmetic is unchanged (the existing conservation test still passes).
- Add a test that **combat's** candidates stop at an opaque wall.
- Add a test that **painting capacity** (the `edit.rs` capacity-painting entry point) leaves a wall site at 0.
- Add a test that **`place_agent`** on a wall returns an error mentioning "wall".

Adapt the helper names (`agent_at`, `grow`) to the real `testkit` and growback functions. The assertions are the requirement.

- [ ] **Step 2: Run the tests and check they fail**

Run: `cargo test -p sugarscape-core --lib world::tests -- walls sight placement`
Expected: compile errors.

- [ ] **Step 3: Implement**
  - **`geometry.rs`:**
    - `sight_until(p, vision, blocked)` walks each direction in `DIRECTIONS` order, `d = 1..=vision`, and `break`s at the first `q` with `blocked(q)` without pushing it.
    - Then it does the same stable sort by distance and duplicate/self removal as `sight`.
    - `sight` becomes `self.sight_until(p, vision, |_| false)`, and the existing geometry tests must still pass.
  - **`world.rs`:**
    - Build `walls: Vec<u8>` from `config.walls` in `with_landscapes` (0 free, 1 fence, 2 opaque; opaque wins where rectangles overlap). Before `Site::full`, set every good's capacity to 0 on wall sites.
    - `has_walls` is `!config.walls.is_empty()`. `is_wall(p)` is `walls[idx] != 0`, and `is_opaque(p)` is `walls[idx] == 2`.
    - `is_occupied(p)` is `self.occupancy[i].is_some() || self.walls[i] != 0`. `occupant` is unchanged.
    - `empty_sites` also excludes walls.
    - `insert_agent` returns `Err(format!("site ({x}, {y}) is a wall"))` on a wall.
    - `sight(p, v)`: `if !self.has_walls() { self.torus.sight(p, v) } else { self.torus.sight_until(p, v, |q| self.is_opaque(q)) }`.
    - In `populate`, filter wall cells out of each placement's cell list **only when `has_walls()`**, so the list and the shuffle's draws are unchanged without walls.
    - `landscape_edited` compares against the generated map with wall sites zeroed.
  - **`movement.rs` `candidates` and both `combat.rs` sight calls:** use `world.sight(..)`.
  - **`pollution.rs` `diffuse`:** for a non-wall site, the average is the sum over non-wall neighbors, in the same order, divided by `f64::from(count)`; a wall site stays 0. With no walls, `count` is 4 and `f64::from(4u32) == 4.0`, so the arithmetic is bit-identical. Keep a fast path `if !world.has_walls()` that runs today's exact expression if that's simpler to prove unchanged.
  - **`edit.rs`:** painting or setting capacities leaves wall sites at 0.
  - **`render.rs`:** a wall site draws `WALL: Rgb = [0x5a, 0x55, 0x4c]` (stone) and a fence `FENCE: Rgb = [0x8a, 0x6d, 0x3b]` (wood), in place of the background lerp. Agents never stand on them. Add a render test that a wall pixel has the wall color.

- [ ] **Step 4: Run everything**

Run: `cargo test -p sugarscape-core && cargo clippy --workspace --all-targets -- -D warnings`
Expected: all pass. Golden and legacy are unchanged, because no existing preset has walls.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src
git commit -m "Minds 2: walls and fences — no sugar, no Flumps, sight stops at walls, diffusion skips them

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 4: Walking

**Files:**
- Modify: `crates/sugarscape-core/src/agent.rs` (`Plan`, the `Agent.plan` field; the three `Agent` literals: `Agent::random`, the child in `rules/sex.rs`, `testkit::spawn`)
- Modify: `crates/sugarscape-core/src/rules/movement.rs` (`arrive`; `act` ends in it)
- Modify: `crates/sugarscape-core/src/minds/utility.rs` (`act` ends in `arrive`)
- Modify: `crates/sugarscape-core/src/edit.rs` (`AgentView.plan`)
- Modify: `crates/sugarscape-core/tests/minds.rs` (the vision-1 reduction)

**Interfaces:**
- Consumes:
  - `astar`, `TorusGrid` (Task 2);
  - `World::is_occupied` with walls (Task 3);
  - `Movement`, `MoveMode` (Task 1);
  - `movement::go_and_gather`.
- Produces:
  - `agent::Plan { target: Option<Pos>, path: Vec<Pos> }` (`Clone, Debug, Default, PartialEq`);
  - `Agent.plan: Plan`;
  - `movement::arrive(world: &mut World, id: AgentId, target: Pos) -> Harvest`;
  - `edit::PlanView { target_x: u32, target_y: u32, path: Vec<[u32; 2]> }`, with `AgentView.plan: Option<PlanView>`.
- `pub const WALK_LIMIT: usize = 4096;` in `movement.rs`.

- [ ] **Step 1: Write the failing tests** (in `movement.rs` tests)

```rust
    fn walker(w: &mut World, vision: u32, speed: u32) -> AgentId {
        w.config.movement = crate::config::Movement { mode: crate::config::MoveMode::Walk, speed };
        mover(w, vision)
    }

    #[test]
    fn a_walker_takes_one_step_toward_the_target_and_gathers_only_there() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        set_sugar(&mut w, 5, 8, 3.0);
        set_sugar(&mut w, 5, 6, 1.0); // on the way: stepped onto, so gathered
        let h = act(&mut w, id);
        // Rule M picks (5, 8) (the most sugar); the walker steps to (5, 6).
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6));
        assert_eq!(h.gathered[0], 1.0);
        assert_eq!(w.site(Pos::new(5, 8)).resource[0], 3.0, "not reached yet");
        let plan = &w.agent(id).unwrap().plan;
        assert_eq!(plan.target, Some(Pos::new(5, 8)));
        assert_eq!(plan.path, vec![Pos::new(5, 7), Pos::new(5, 8)]);
    }

    #[test]
    fn a_walker_goes_around_an_occupant() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        spawn(&mut w, 5, 6);
        set_sugar(&mut w, 5, 7, 3.0);
        act(&mut w, id);
        let p = w.agent(id).unwrap().pos;
        assert!(p == Pos::new(4, 5) || p == Pos::new(6, 5), "a sidestep, got {p:?}");
    }

    #[test]
    fn speed_three_stops_at_the_target() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 3);
        set_sugar(&mut w, 5, 7, 3.0);
        let h = act(&mut w, id);
        assert_eq!((w.agent(id).unwrap().pos, h.gathered[0]), (Pos::new(5, 7), 3.0));
        assert!(w.agent(id).unwrap().plan.path.is_empty());
    }

    #[test]
    fn a_walker_with_no_path_stays_and_gathers_where_it_is() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        set_sugar(&mut w, 5, 5, 0.5);
        set_sugar(&mut w, 5, 8, 3.0);
        for (x, y) in [(5, 4), (5, 6), (4, 5), (6, 5)] {
            spawn(&mut w, x, y); // boxed in
        }
        let before = w.rng.clone();
        let h = act(&mut w, id);
        assert_eq!((w.agent(id).unwrap().pos, h.gathered[0]), (Pos::new(5, 5), 0.5));
        let mut expected = before; // only choose's draw
        let _ = choose(&candidates(&w, id), &mut expected);
        let _ = expected; // draws were compared through the reduction test; see below
    }
```

(The last test's RNG check is awkward after the move. Keep the position and harvest assertions, and prove "no draws beyond `choose`" with the reduction test and one direct test: compare `w.rng` after `act` against a clone advanced by exactly one `choose` over the same candidates, computed before `act`.)

A test that `plan` isn't hashed: two worlds that differ only in an agent's `plan` have equal fingerprints.

In `tests/minds.rs`, add:

```rust
#[test]
fn walking_at_vision_one_is_jumping() {
    let mut checked = 0;
    for p in presets::all() {
        if p.config.combat.enabled {
            continue;
        }
        let mut jump = p.config.clone();
        jump.vision = sugarscape_core::config::URange::new(1, 1);
        let mut walk = jump.clone();
        walk.movement.mode = sugarscape_core::config::MoveMode::Walk;
        assert_eq!(fingerprint(walk), fingerprint(jump), "{}", p.id);
        checked += 1;
    }
    assert!(checked >= 20, "checked {checked} presets");
}
```

(If a preset's other settings make vision 1 invalid, skip it with a comment naming why. None should.)

- [ ] **Step 2: Run the tests and check they fail**

Run: `cargo test -p sugarscape-core --lib movement::tests::a_walker; cargo test -p sugarscape-core --test minds`
Expected: compile errors, or failures.

- [ ] **Step 3: Implement.** In `agent.rs`:

```rust
/// Minds 2: where the agent last decided to go and the path it planned
/// there that it hasn't walked yet (observation only: never hashed,
/// exported or shared, like `social`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    pub target: Option<Pos>,
    pub path: Vec<Pos>,
}
```

Add `pub plan: Plan,` to `Agent` (with that doc line), and `plan: Plan::default()` to the three literals.

In `movement.rs`:

```rust
/// Most sites A* may expand for a walking agent; past it the target counts
/// as unreachable (Minds 2).
pub const WALK_LIMIT: usize = 4096;

/// Reaches `target` by the configured movement, then gathers where the
/// agent stops. `jump` (rule M) goes there in one tick. `walk` takes `speed`
/// steps along an A* path on the 4-way torus (walls and occupied sites
/// impassable, except the target) and stays when there is none within
/// `WALK_LIMIT`. Records the agent's plan; draws nothing.
pub(crate) fn arrive(world: &mut World, id: AgentId, target: Pos) -> Harvest {
    let pos = world.agent(id).expect("live agent").pos;
    let m = world.config.movement;
    if m.mode == MoveMode::Jump || target == pos {
        world.agent_mut(id).expect("live agent").plan = Plan { target: Some(target), path: Vec::new() };
        return go_and_gather(world, id, target);
    }
    let torus = world.torus;
    let found = {
        let grid = TorusGrid::new(torus, |q| q == target || !world.is_occupied(q));
        astar(&grid, torus.index(pos), torus.index(target), WALK_LIMIT)
    };
    let (stop, rest) = match found {
        Some(s) => {
            let steps = (m.speed as usize).min(s.path.len() - 1);
            let rest = s.path[steps + 1..].iter().map(|&i| torus.pos(i)).collect();
            (torus.pos(s.path[steps]), rest)
        }
        None => (pos, Vec::new()),
    };
    world.agent_mut(id).expect("live agent").plan = Plan { target: Some(target), path: rest };
    go_and_gather(world, id, stop)
}
```

- `act` ends `arrive(world, id, target)`, and so does `utility::act` (both its normal path and its boxed-in wander fallback).
- In `edit.rs`, add `PlanView` and `plan: a.plan.target.map(|t| PlanView { target_x: t.x, target_y: t.y, path: a.plan.path.iter().map(|p| [p.x, p.y]).collect() })` to the `AgentView` fill, with serde.
- If there's a web `AgentView` test fixture in core, update it.

- [ ] **Step 4: Run everything**

Run: `cargo test -p sugarscape-core && cargo test -p sugarscape-wasm && cargo clippy --workspace --all-targets -- -D warnings`
Expected: all pass, including both reduction tests, and golden unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core
git commit -m "Minds 2: walking along an A* path, speed steps a tick; at vision 1 it is jumping

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 5: The eight presets, titles and golden entries

**Files:** `crates/sugarscape-core/src/presets.rs` (appended after the `ifd-*` presets; the count 39 → 47), `titles.rs` (+8, inserted after `ifd-travel`), `tests/golden.rs` (+8).

**Interfaces:**
- Consumes:
  - `Movement`, `MoveMode`, `Wall` (Task 1);
  - the `ii-2-unit`, `ii-5-wealth`, `ii-7-seasons` and `ii-6-waves` presets' edit closures (reuse by building from `by_id(..).config` inside the closure, or by extracting their setup into helpers; prefer helpers if the closures are inline);
  - Minds 1's `two_patches` helper.
- Produces:
  - presets `walk-capacity`, `walk-wealth`, `walk-seasons`, `walk-waves`, `walk-fast`, `ifd-fence`, `ifd-fence-far`, `ifd-wall`;
  - the helper `fn fence(c: &mut Config, offset: u32, opaque: bool)`.

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn the_walking_presets_walk_and_the_fenced_ones_have_a_gap() {
        for id in ["walk-capacity", "walk-wealth", "walk-seasons", "walk-waves", "walk-fast", "ifd-fence", "ifd-fence-far", "ifd-wall"] {
            let c = by_id(id).unwrap_or_else(|| panic!("{id}")).config;
            assert_eq!(c.movement.mode, MoveMode::Walk, "{id}");
        }
        assert_eq!(by_id("walk-fast").unwrap().config.movement.speed, 3);
        let base = |id: &str| by_id(id).unwrap().config;
        let mut jumped = base("walk-capacity");
        jumped.movement = Movement::default();
        assert_eq!(jumped, base("ii-2-unit"));
        let gap_rows = |id: &str| {
            let c = base(id);
            let covered = |y: u32| c.walls.iter().any(|w| w.x == 28 && (w.y..w.y + w.height).contains(&y));
            (0..40).filter(|&y| !covered(y)).collect::<Vec<_>>()
        };
        assert_eq!(gap_rows("ifd-fence"), [20, 21]);
        assert_eq!(gap_rows("ifd-fence-far"), [35, 36]);
        assert!(base("ifd-fence").walls.iter().all(|w| !w.opaque));
        assert!(base("ifd-wall").walls.iter().all(|w| w.opaque));
        assert_eq!(base("ifd-fence").walls.len(), 4, "x = 28 and x = 2, above and below the gap");
        assert_eq!(base("ifd-fence").vision, URange::new(10, 20));
    }
```

- [ ] **Step 2: Run it and check it fails.**

- [ ] **Step 3: Implement**

```rust
/// Minds 2's fences: one-site-wide fences at x = 28 (between the patches)
/// and x = 2 (closing the route around the torus), from top to bottom
/// except a two-site gap at rows 20 + `offset` and 21 + `offset`.
fn fence(c: &mut Config, offset: u32, opaque: bool) {
    let rect = |x, y, height| Wall { x, y, width: 1, height, opaque };
    c.walls = vec![
        rect(28, 0, 20 + offset),
        rect(28, 22 + offset, 18 - offset),
        rect(2, 0, 20 + offset),
        rect(2, 22 + offset, 18 - offset),
    ];
}
```

Presets (the descriptions state the setup and the planning probe's numbers where there are any; Task 9 replaces them with measured values):

- **`walk-capacity`** ("Walking: carrying capacity", "Epstein & Axtell II-2; Minds 2"): `ii-2-unit` with `movement.mode = Walk`. "Rule M's Flumps jump to the best site in sight; these walk there one step a tick along an A* path. Measured in planning (a throwaway hack, 10 seeds): the population settles near 179 instead of the book's 224 (230 under jump)."
- **`walk-wealth`** (II-5): `ii-5-wealth` walking.
- **`walk-seasons`** (II-7): `ii-7-seasons` walking.
- **`walk-waves`** (II-6): `ii-6-waves` walking. "The book's waves travel northeast from the southwest block; under rule M's jump they don't. In planning, walking didn't bring them back either: the block settles on the near mountain."
- **`walk-fast`**: `ii-2-unit` walking at speed 3.
- **`ifd-fence`** ("Travel between patches: a fence with a central gap", "Baum & Kraft 1998; Minds 2"): `two_patches(c, 7.0)`, vision 10–20, walk, `fence(c, 0, false)`.
- **`ifd-fence-far`**: the same with `fence(c, 15, false)`.
- **`ifd-wall`**: `ifd-fence` with `fence(c, 0, true)`. "An opaque wall: the other patch is visible only through the gap."

Titles (after `ifd-travel`'s):

```rust
    ("walk-capacity", "Flumps who walk instead of jump: fewer of them survive"),
    ("walk-wealth", "Walking changes who gets rich"),
    ("walk-seasons", "Walking through the seasons"),
    ("walk-waves", "Walking doesn't bring back the book's waves"),
    ("walk-fast", "Flumps who walk three steps a tick"),
    ("ifd-fence", "A fence between the patches, with one gap"),
    ("ifd-fence-far", "The gap moves to the far end: switching costs a long walk"),
    ("ifd-wall", "A wall between the patches: the other patch is out of sight too"),
```

`TITLES`' length grows by 8, and the preset count assertion becomes 47. Record the golden entries with `print_golden`; only the eight new lines.

- [ ] **Step 4: Run** `cargo test -p sugarscape-core` (reductions, titles and golden pass).

- [ ] **Step 5: Commit** ("Minds 2: eight presets — walking book worlds and fenced patches").

---

### Task 6: Three sweeps

**Files:** `sweeps/walk-speed.json`, `sweeps/walk-vision.json`, `sweeps/ifd-detour.json`; `crates/sugarscape-core/src/sweep.rs` (`BUILTINS` +3; the id list in `builtin_sweeps_parse_and_validate`).

- [ ] **Step 1:** Append `"walk-speed", "walk-vision", "ifd-detour"` to the expected id list, and run it to check it fails.
- [ ] **Step 2: Write the files.**

`walk-speed.json`: base `walk-capacity`; seeds 1–20; ticks 500; metric `window_mean` of `population` from 300.

```json
  "name": "Minds 2: carrying capacity against walking speed",
  "description": "The mean population over ticks 300–500 of Animation II-2's world when Flumps walk 1, 2, 3, 4, 6 or 10 steps a tick along an A* path, against rule M's jump at each point. The book reports about 224 under the jump. Seeds 1–20.",
  "x": { "label": "Steps per tick", "path": "movement.speed", "values": [1, 2, 3, 4, 6, 10] },
  "series": { "label": "Movement", "values": [
    { "at": 0, "name": "Jump (book)", "set": { "movement.mode": "jump" } },
    { "at": 1, "name": "Walk", "set": { "movement.mode": "walk" } } ] },
```

`walk-vision.json`: the same base and metric.

```json
  "name": "Minds 2: carrying capacity against vision, walking and jumping",
  "description": "The mean population over ticks 300–500 of Animation II-2's world against the vision range, when Flumps jump (rule M) or walk one step a tick. At vision 1 the two are the same rule. Seeds 1–20.",
  "x": { "label": "Vision range", "values": [
    { "at": 1, "name": "1", "set": { "vision": { "min": 1, "max": 1 } } },
    { "at": 3, "name": "1–3", "set": { "vision": { "min": 1, "max": 3 } } },
    { "at": 6, "name": "1–6", "set": { "vision": { "min": 1, "max": 6 } } },
    { "at": 10, "name": "1–10", "set": { "vision": { "min": 1, "max": 10 } } } ] },
  "series": { … the same Movement series … },
```

`ifd-detour.json`: base `ifd-fence`; seeds 1–20; ticks 1000; metric `window_mean` of `first_patch_share` from 500. The x axis moves the gap through the four fence rectangles' fields. The series sets each rectangle's `opaque`. Series settings apply before x settings, and they touch different fields.

```json
  "name": "Minds 2: travel between patches and the ideal free distribution",
  "description": "The richer patch's share of on-patch Flumps (2.10 : 1 inputs, vision 10–20, walking) against how far the only gap in the fences between the patches lies from the patches' center row, for a fence (sight passes) and an opaque wall (the other patch visible only through the gap). Baum & Kraft 1998: requiring travel to switch patches reduced undermatching slightly; a visual barrier had no effect. Seeds 1–20, ticks 500–1000.",
  "x": { "label": "Gap offset (rows)", "values": [
    { "at": 0, "set": { "walls.0.height": 20, "walls.1.y": 22, "walls.1.height": 18, "walls.2.height": 20, "walls.3.y": 22, "walls.3.height": 18 } },
    { "at": 5, "set": { "walls.0.height": 25, "walls.1.y": 27, "walls.1.height": 13, "walls.2.height": 25, "walls.3.y": 27, "walls.3.height": 13 } },
    { "at": 10, "set": { "walls.0.height": 30, "walls.1.y": 32, "walls.1.height": 8, "walls.2.height": 30, "walls.3.y": 32, "walls.3.height": 8 } },
    { "at": 15, "set": { "walls.0.height": 35, "walls.1.y": 37, "walls.1.height": 3, "walls.2.height": 35, "walls.3.y": 37, "walls.3.height": 3 } } ] },
  "series": { "label": "Barrier", "values": [
    { "at": 0, "name": "Fence", "set": { "walls.0.opaque": false, "walls.1.opaque": false, "walls.2.opaque": false, "walls.3.opaque": false } },
    { "at": 1, "name": "Wall", "set": { "walls.0.opaque": true, "walls.1.opaque": true, "walls.2.opaque": true, "walls.3.opaque": true } } ] },
```

Add the three `Builtin` entries (`BUILTINS` length +3).

- [ ] **Step 3: Run** `cargo test -p sugarscape-core --lib sweep`. Then run `walk-speed` through the CLI's sweep command and record the table in the report: walk at speed 1 should sit near the probe's 179, and jump near 230.
- [ ] **Step 4: Commit** ("Minds 2: sweeps of carrying capacity against speed and vision, and of detours between patches").

---

### Task 7: WASM agreement

**Files:** `crates/sugarscape-wasm/tests/web.rs`.

- [ ] **Step 1:** Add `walking_behind_a_fence_matches_its_golden_entry`, modeled on `the_utility_minds_crowding_matches_its_golden_entry`: preset `ifd-fence`, 200 ticks, seed 1, its GOLDEN value.
- [ ] **Step 2:** Update `builtins_and_series_names_are_listed`'s sweep list with the three new sweeps (after `ifd-travel`), and any other catalog list the new presets make stale.
- [ ] **Step 3:** Check that the WASM `inspect` output carries `plan` (serde) and add an assertion if there's an inspect test to extend.
- [ ] **Step 4:** Run `wasm-pack test --node crates/sugarscape-wasm` until everything passes.
- [ ] **Step 5: Commit** ("Minds 2: WASM agrees on walking behind a fence").

---

### Task 8: The page

**Files:** `web/src/types.ts`, `web/src/schema.ts`, `web/src/ui/inspect-panel.ts`, `web/src/ui/grid-view.ts`, `web/src/ui/plan-path.ts` (new), and tests `web/src/schema.test.ts` and `web/src/ui/plan-path.test.ts`.

**Interfaces:**
- Consumes: `AgentView.plan` (Task 4), `Config.movement`, `Config.walls` (Task 1). Walls are already drawn by the core render (Task 3).
- Produces: `planSegments(from: [number, number], path: [number, number][], width: number, height: number): [number, number, number, number][]`, the line segments between consecutive cells, skipping any that wrap across the seam, in the style of `trailSegments` (reuse it if its signature fits).

- [ ] **Step 1: Write the failing tests**
  - **`schema.test.ts`:**
    - a Movement group whose `movement.mode` select offers `jump` and `walk`, live (no `reset`);
    - `current` gives `'jump'` for a config without `movement`;
    - applying `walk` makes `movement` `{ mode: 'walk', speed: 1 }`;
    - a `movement.speed` number control (1–50, live) that on a config without `movement` sets a complete `{ mode: 'jump', speed: n }`, following Minds 1's `decision` pattern.
  - **`plan-path.test.ts`:**
    - a straight path gives consecutive segments;
    - a path across the x seam (from x = 49 to x = 0 on a 50-wide grid) gives no segment across the grid.

- [ ] **Step 2: Run them and check they fail.**

- [ ] **Step 3: Implement**
  - **`types.ts`:**
    - `Movement { mode: 'jump' | 'walk'; speed: number }`;
    - `Wall { x; y; width; height; opaque: boolean }`;
    - `Config.movement?: Movement`, `Config.walls?: Wall[]`;
    - `AgentView.plan?: { target_x: number; target_y: number; path: [number, number][] } | null`.
  - **`schema.ts`:** a group `Movement (Minds 2)` after `Decision (Minds 1)`. Its note reads: "How a Flump reaches the site it chose. The book's rule M jumps there in one tick. Walking takes that many steps a tick along an A* path around walls and other Flumps, and plans again every tick. Walls and fences come from presets (the Minds 2 fence presets); a wall also blocks sight."
    - Controls: `movement.mode` (a select, live) and `movement.speed` (a number, 1–50, live). Each `apply`/`adjust` seeds a complete `movement` from `c.movement ?? { mode: 'jump', speed: 1 }`.
  - **`inspect-panel.ts`:** in `agentRows`, when `a.plan` is present:
    - under walk, `row('Heading', a.plan.path.length ? \`(${a.plan.target_x}, ${a.plan.target_y}), ${a.plan.path.length} steps left\` : 'Staying')`;
    - under jump, `row('Moved to', \`(${a.plan.target_x}, ${a.plan.target_y})\`)`.
  - **`grid-view.ts`:** when an agent is inspected and its `plan` has a path, draw `planSegments` from the agent's cell through the path, after the trail, in a dashed style (`setLineDash([4, 4])`, stroke `--accent`, `lineWidth` 2, cell centers as the trail does).

- [ ] **Step 4: Run** `cd web && npm run build && npm test`.
- [ ] **Step 5: Check it in the browser** with the dev server:
  - load `ifd-fence`: the fences are drawn and the Movement group shows Walk;
  - inspect a Flump: the Heading row and the dashed path appear;
  - load `ii-2-unit`: no path line; Inspect shows "Moved to".

  Report what you saw, and stop the server.
- [ ] **Step 6: Commit** ("Minds 2 on the page: the Movement group, the heading and the planned path").

---

### Task 9: The survey, and measured descriptions and titles

**Files:** `survey/src/claims/minds2.rs`, `survey/src/claims/mod.rs`; `crates/sugarscape-core/src/presets.rs` and `titles.rs` (wording only).

**Interfaces:**
- Consumes:
  - the presets (Task 5);
  - `runner::{each_seed, preset, series}`;
  - Minds 1's module helpers (`survey/src/claims/minds1.rs`: `log_ratio`, `slope`, `sites`, per-seed s). Make the shared ones `pub(crate)` there and reuse them rather than copying.
  - `ch2.rs`'s seasonal-migration measure (reuse its helper by making it `pub(crate)`).

- [ ] **Step 1: Write the module.** Claim ids start with their item; `--only walk` and `--only ifd-` must select them. Twenty seeds unless stated.

| Claim | Item | Judge | Text |
|---|---|---|---|
| `walk-capacity.book` | `walk-capacity` | `range(pop, 214, 234)` (the book's 224 ± 10), mean over ticks 300–500 | "The population stabilizes at about 224 (Animation II-2), when Flumps walk" |
| `walk-capacity.lower` | `walk-capacity` | `greater(jump, walk)` on the same measure; `ii-2-unit` gives the jump | "Walking lowers the carrying capacity below the jump's" |
| `walk-wealth.skewed` | `walk-wealth` | `range(skewness, 0.0, ∞)` of wealth at tick 500 (`stats::skewness`) | "Wealth is right-skewed (Animation II-5), when Flumps walk", with the Gini in the detail |
| `walk-seasons.migrate` | `walk-seasons` | `range(migrant share, 0.05, 1.0)`, where the share changes hemisphere at least twice over ticks 100–300 (ch2's measure); jump's share in the detail | "Some Flumps migrate with the seasons (Animation II-7), when Flumps walk" |
| `walk-waves.reach` | `walk-waves` | `range(far share, 0.25, 1.0)`: the share of Flumps whose torus distance from the starting block's center is above 25 at tick 100; jump's share in the detail | "A wave reaches the far mountain (Animation II-6), when Flumps walk" |
| `walk-speed.recovers` | `walk-speed` | `greater(pop at speed 10, pop at speed 1)` under walk; jump's population in the detail | "Faster walking recovers the jump's carrying capacity" |
| `ifd-fence-far.baum-kraft` | `ifd-fence-far` | `greater(s far gap, s no fence)`; s per seed across Minds 1's five radii with the same fences and gap; "no fence" is `ifd-far-sighted` switched to walk | "Requiring travel to switch patches reduces undermatching (Baum & Kraft 1998)" |
| `ifd-wall.visual` | `ifd-wall` | paired `range(s_wall − s_fence, −0.05, 0.05)` at gap offset 0 | "A visual barrier has no effect (Baum & Kraft 1998), seed by seed" |

The fences use `fence`'s geometry with the second patch's radius varied. The gap stays at rows 20–21 for "near" and 35–36 for "far". Compute s exactly as Minds 1 does (Earn & Johnstone).

- [ ] **Step 2: Run** `cd survey && cargo run --release -- --only walk` and `--only ifd-` (make sure Minds 1's `ifd-*` claims still give the same verdicts). Record every verdict and measured string. **Never tune a claim to pass or fail.** Where a judge turns out to test something other than its text for a reason known before running, fix it, and say so in the report.

- [ ] **Step 3: Rewrite the descriptions and titles.**
  - Replace each Minds 2 preset's description numbers with the survey's ("Measured (20 seeds): …").
  - Adjust any title the measurements contradict.
  - Label causes as measured, or as "likely" when they aren't.
  - Golden entries must not change.

- [ ] **Step 4: Measure the cost table.** Build the release CLI and time 2 000 ticks of `ii-2-unit` under jump, `walk-capacity`, `walk-fast` and `ifd-fence` (walk), and `ifd-far-sighted` (jump), over seeds 1–5. Report the whole tick, all rules, per Flump-tick, alongside Minds 1's baseline. Put the table in the report for Task 10.

- [ ] **Step 5: Commit** ("Survey Minds 2: which book results need the jump, and travel between patches").

---

### Task 10: Docs

**Files:** `README.md` (a section after Minds 1's), `docs/studies/2026-09-27-minds.md`, `docs/roadmap.md`, and the spec's `## Amendments (implementation planning)`.

- [ ] **Step 1: README.** Add `### Minds 2: A* and walking`, covering:
  - A*'s verification: the Dijkstra property tests and Sturtevant's benchmarks with attribution;
  - walls and fences;
  - walking and its reduction at vision 1;
  - the presets and sweeps;
  - the findings with their verdicts;
  - the cost table.

  Label it as our experiment. Every number comes from the survey, the sweeps or the Task 9 cost table, with its source stated.
- [ ] **Step 2: The program document.**
  - Status: "Minds 1 and 2 done; Minds 3 (memory and belief) next".
  - Add a Results section for Minds 2 and the cost table.
  - Add one labeled suggestion line for Minds 3.
- [ ] **Step 3: Roadmap.** Add the Minds 2 line after Minds 1's.
- [ ] **Step 4: The spec's amendments.** Record:
  - the wall and fence render colors (fixed palette colors in the core render, not theme tokens, because the core render is always dark);
  - the plan-and-inspect details as built;
  - any rulings made during execution;
  - the measured values.
- [ ] **Step 5: The full suite.** Run `cargo test --release && (cd web && npm run build && npm test) && wasm-pack test --node crates/sugarscape-wasm && cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] **Step 6: Commit** ("Document Minds 2 and which of the book's results need the jump; mark it done").
