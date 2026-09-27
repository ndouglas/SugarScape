# Minds 1: the utility mind and the ideal free distribution — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a decision seam to the Sugarscape's rule M. Put a utility mind behind it that reduces exactly to rule M. Then measure the book's rule and the utility mind against the ideal free distribution.

**Architecture:**
- A `decision` object on the Sugarscape `Config` picks `book` (rule M, the default) or `utility`.
- `rules::agent_turn` calls `minds::decide`, which dispatches to `movement::act` or `minds::utility::act`.
- Rule M is split into shared pieces (`candidates`, `go_and_gather`) so the utility mind reuses its candidate list, tie rule, random draw and harvest exactly.
- Patch series appear only on `peaks` maps with two or more peaks.
- Presets, sweeps, a page Decision group and Patches chart, and a survey module follow the milestone pattern.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), the standalone `survey/` crate, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-09-27-minds-1-utility-design.md`

## Global Constraints

- Every existing golden entry, legacy fixture and pinned fingerprint stays green and **unedited**.
- `decision.rule: book` is the default. Under `book` and under `idle: stay`, nothing draws from `World.rng` beyond what rule M draws today.
- Under `utility` with `travel` 0, `crowding` 0 and `idle: stay`, every golden Sugarscape preset without combat has its golden fingerprint.
- `utility` with combat on is a validation error on `decision.rule`: "rule C decides moves under combat".
- `travel` and `crowding` are 0–10. Under `book` they are kept but ignored.
- `decision.rule` changes only on reset. `travel`, `crowding` and `idle` are live.
- `ln` and `exp` come from `crate::portable` (`ln`, `exp_neg`). Never `f64::ln` or `f64::exp` in a rule.
- American spelling (neighbor, center, color) in code, captions and docs.
- Titles follow `titles.rs`: a plain headline of what happens, not starting with "Animation", "Fig.", "Figure" or "Table ", with no "({" notation.
- Every commit message ends with the line `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- Work in the worktree `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds` (branch `minds`).

## Review Focus

1. **Configs that omit `decision` or give it partly** (old share links, sessions, saved configs) must load as the book, with missing fields defaulted. Owned by Task 1.
2. **A schedule that sets `decision.travel` mid-run** must be accepted and take effect; one that sets `decision.rule` or the whole `decision` must be refused as reset-only. Owned by Task 1.
3. **A wandering Flump boxed in with every neighboring site in sight occupied** must stay put, and draw nothing, rather than panic or pick its own site through a draw. Owned by Task 3.
4. **Crowding must not count the mover itself** when it scores its own current site. Owned by Task 3.
5. **Patch series on a one-peak map or a non-peaks map must be absent, not zero.** An empty world must give `first_patch_share` 0, not NaN. Owned by Task 4.

---

## File structure

- Modify `crates/sugarscape-core/src/config.rs`: the `Decision`, `DecisionRule` and `Idle` types, the config field, validation and reset-only rules.
- Modify `crates/sugarscape-core/src/legacy.rs`: the default `decision` in `convert`.
- Modify `crates/sugarscape-core/src/rules/movement.rs`: split `act` into `candidates` and `go_and_gather`.
- Create `crates/sugarscape-core/src/minds/mod.rs`: the seam (`decide`).
- Create `crates/sugarscape-core/src/minds/utility.rs`: the considerations, `score`, `crowd`, `act`.
- Modify `crates/sugarscape-core/src/lib.rs`: `pub mod minds;`.
- Modify `crates/sugarscape-core/src/rules/mod.rs`: call the seam.
- Create `crates/sugarscape-core/tests/minds.rs`: the reduction test.
- Modify `crates/sugarscape-core/src/landscape.rs`: `peak_distance`, `patch_of`.
- Modify `crates/sugarscape-core/src/stats.rs`: `PatchStats`, the optional series.
- Modify `crates/sugarscape-core/src/presets.rs`, `titles.rs` and `tests/golden.rs`: eight presets.
- Create `sweeps/ifd-matching.json`, `sweeps/ifd-idle.json`, `sweeps/ifd-crowding.json` and `sweeps/ifd-travel.json`; modify `crates/sugarscape-core/src/sweep.rs`.
- Modify `crates/sugarscape-wasm/tests/web.rs`: pin `ifd-crowding`.
- Modify `web/src/types.ts`, `web/src/schema.ts` and `web/src/ui/charts-panel.ts`; create `web/src/patches.ts`; tests in `web/src/schema.test.ts` and `web/src/patches.test.ts`.
- Create `survey/src/claims/minds1.rs`; modify `survey/src/claims/mod.rs`.
- Docs: `README.md`, `docs/studies/2026-09-27-minds.md` and `docs/roadmap.md`.

---

### Task 1: The `decision` config

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs` (the types near `PriceRule` at ~`:382`; `Config` `:585-611`; `Default` `:613-684`; `RESET_ONLY_PATHS` `:515`; `validate_fields` near the combat check at ~`:1143`; `structural_changes` `:1284-1332`; tests module)
- Modify: `crates/sugarscape-core/src/legacy.rs:174-210`

**Interfaces:**
- Produces: `config::{Decision, DecisionRule, Idle}`; `Config.decision: Decision`. `Decision { rule: DecisionRule, travel: f64, crowding: f64, idle: Idle }`, `Copy`, `Default` (Book, 0.0, 0.0, Stay). Serde names: `"book" | "utility"`, `"stay" | "wander"`.

- [ ] **Step 1: Write the failing tests** (append to `config.rs`'s `mod tests`)

```rust
    #[test]
    fn decision_defaults_to_the_book_and_older_configs_load_as_the_book() {
        assert_eq!(Config::default().decision, Decision::default());
        assert_eq!(Decision::default().rule, DecisionRule::Book);
        assert_eq!(Decision::default().idle, Idle::Stay);
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v.as_object_mut().unwrap().remove("decision");
        assert_eq!(Config::from_value(v).unwrap().decision, Decision::default());
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v["decision"] = serde_json::json!({ "rule": "utility" });
        let d = Config::from_value(v).unwrap().decision;
        assert_eq!((d.rule, d.travel, d.crowding, d.idle), (DecisionRule::Utility, 0.0, 0.0, Idle::Stay));
    }

    #[test]
    fn decision_is_validated() {
        let with = |f: fn(&mut Config)| {
            let mut c = Config::default();
            f(&mut c);
            fields(c.validate())
        };
        assert!(with(|c| c.decision.travel = 10.0).is_empty());
        assert_eq!(with(|c| c.decision.travel = 10.5), ["decision.travel"]);
        assert_eq!(with(|c| c.decision.travel = -0.1), ["decision.travel"]);
        assert_eq!(with(|c| c.decision.crowding = f64::NAN), ["decision.crowding"]);
        assert_eq!(
            with(|c| {
                c.decision.rule = DecisionRule::Utility;
                c.combat.enabled = true;
            }),
            ["decision.rule"]
        );
        // Under the book the utility fields are kept but ignored: no error.
        assert!(with(|c| {
            c.decision.travel = 2.0;
            c.decision.idle = Idle::Wander;
        })
        .is_empty());
    }

    #[test]
    fn the_decision_rule_changes_only_on_reset_and_its_knobs_live() {
        let a = Config::default();
        let mut b = a.clone();
        b.decision.rule = DecisionRule::Utility;
        let f: Vec<String> = a.structural_changes(&b).into_iter().map(|e| e.field).collect();
        assert_eq!(f, ["decision.rule"]);
        let mut b = a.clone();
        b.decision.travel = 1.0;
        b.decision.crowding = 1.0;
        b.decision.idle = Idle::Wander;
        assert!(a.structural_changes(&b).is_empty());
        for (path, value) in [
            ("decision.rule", serde_json::json!("utility")),
            ("decision", serde_json::json!({ "rule": "utility" })),
        ] {
            let c = Config { schedule: vec![change(5, path, value)], ..Default::default() };
            let errs = c.validate().unwrap_err();
            assert!(errs[0].message.contains("only on reset"), "{path}: {errs:?}");
        }
        let c = Config {
            schedule: vec![
                change(5, "decision.travel", serde_json::json!(0.5)),
                change(6, "decision.idle", serde_json::json!("wander")),
            ],
            ..Default::default()
        };
        c.validate().unwrap();
    }
```

- [ ] **Step 2: Run them and check they fail**

Run: `cargo test -p sugarscape-core --lib config::tests::decision -- --nocapture`
Expected: compile errors (`Decision` not found).

- [ ] **Step 3: Implement**

In `config.rs`, after `TradeRule` (~`:399`):

```rust
/// Minds 1: which rule makes rule M's decision (where to move). `Book` is rule
/// M as stated; `Utility` is `crate::minds::utility`. Rule C still decides
/// moves under combat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionRule {
    #[default]
    Book,
    Utility,
}

/// What the utility mind does when every site it sees scores 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Idle {
    /// Stay put, as rule M does (the current site wins ties at distance 0).
    #[default]
    Stay,
    /// Move to a uniformly random unoccupied site in sight.
    Wander,
}

/// The decision seam (Minds 1). `travel`, `crowding` and `idle` apply under
/// the utility rule; under the book they are kept but ignored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Decision {
    pub rule: DecisionRule,
    /// k in the travel consideration 1 / (1 + k·d).
    pub travel: f64,
    /// m in the crowding consideration (1 + n)^(−m).
    pub crowding: f64,
    pub idle: Idle,
}
```

- In `Config`, add `pub decision: Decision,` just before `pub schedule`.
- In `Default`, add `decision: Decision::default(),` just before `schedule`.
- In `legacy.rs`'s `Config { … }` literal, add `decision: crate::config::Decision::default(),` before `schedule,`.

`RESET_ONLY_PATHS` becomes `[&str; 12]`; append `"decision", "decision.rule",`. Update its doc comment to "Paths a schedule may not set: disease structure and the decision rule."

In `validate_fields`, after the `combat.enabled` check:

```rust
        let dc = &self.decision;
        for (v, field) in [(dc.travel, "decision.travel"), (dc.crowding, "decision.crowding")] {
            e.check(
                v.is_finite() && (0.0..=10.0).contains(&v),
                field,
                "must be between 0 and 10",
            );
        }
        e.check(
            !(dc.rule == DecisionRule::Utility && self.combat.enabled),
            "decision.rule",
            "rule C decides moves under combat",
        );
```

In `structural_changes`, before `out`:

```rust
        if self.decision.rule != next.decision.rule {
            out.push(FieldError::new("decision.rule", msg));
        }
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --lib config && cargo test -p sugarscape-core --test legacy --test golden`
Expected: all pass. The golden and legacy tests pass unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/config.rs crates/sugarscape-core/src/legacy.rs
git commit -m "Minds 1: the decision config (book or utility, travel, crowding, idle)

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 2: Split rule M into shared pieces

**Files:**
- Modify: `crates/sugarscape-core/src/rules/movement.rs:52-137`

**Interfaces:**
- Produces:
  - `movement::candidates(world: &World, id: AgentId) -> Vec<(Pos, u32, f64)>`: the current site first at distance 0, then `torus.sight` order, skipping occupied sites, each with rule M's welfare (single-good with the pollution discount, or foresight welfare over n goods).
  - `movement::go_and_gather(world: &mut World, id: AgentId, target: Pos) -> Harvest`: moves, records neighbors, and gathers every good at `target`.
  - `movement::act` is unchanged in behavior and becomes `choose` plus these two.
  - `choose` stays `pub(crate)`.

- [ ] **Step 1: Write the failing test** (in `movement.rs`'s tests)

```rust
    #[test]
    fn candidates_list_the_current_site_first_then_sight_order_skipping_occupied() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 2);
        spawn(&mut w, 5, 6); // occupied: skipped
        set_sugar(&mut w, 5, 5, 1.0);
        set_sugar(&mut w, 7, 5, 3.0);
        let c = candidates(&w, id);
        assert_eq!(c[0], (Pos::new(5, 5), 0, 1.0));
        assert_eq!(c.len(), 1 + 8 - 1);
        assert!(c.iter().all(|x| x.0 != Pos::new(5, 6)));
        assert!(c.windows(2).all(|p| p[0].1 <= p[1].1), "distance order");
        assert!(c.contains(&(Pos::new(7, 5), 2, 3.0)));
    }
```

- [ ] **Step 2: Run it and check it fails**

Run: `cargo test -p sugarscape-core --lib movement::tests::candidates`
Expected: FAIL (`candidates` not found).

- [ ] **Step 3: Implement.** Replace `act` and `act_goods` (`:52-137`) with:

```rust
/// Rule M: look along the four lattice directions as far as vision permits,
/// go to the nearest unoccupied site of maximum welfare and collect its sugar
/// (every good, with n ≥ 2). The agent's current site competes at distance 0,
/// so it stays put when nothing visible is better. Returns the harvest.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let candidates = candidates(world, id);
    let target = choose(&candidates, &mut world.rng);
    go_and_gather(world, id, target)
}

/// Rule M's candidates for `id`: its current site at distance 0, then the
/// unoccupied sites in sight in `torus.sight` order, each with rule M's
/// welfare. One good: sugar, discounted by 1/(1 + Σ pₖ) under pollution.
/// n ≥ 2 goods: foresight welfare after gathering, each good discounted by
/// the pollutants that devalue it.
pub(crate) fn candidates(world: &World, id: AgentId) -> Vec<(Pos, u32, f64)> {
    let a = world.agent(id).expect("live agent");
    let (pos, vision) = (a.pos, a.vision);
    let n = world.config.goods.len();
    let value: Box<dyn Fn(Pos) -> f64 + '_> = if n >= 2 {
        let fee = world.config.disease.active_fee();
        let (phi, held) = (a.foresight, a.holdings);
        let mets = a.effective_metabolisms(n, fee);
        Box::new(move |p: Pos| {
            let s = world.site(p);
            let after: [f64; MAX_GOODS] = std::array::from_fn(|i| {
                if i >= n {
                    return 0.0;
                }
                let counted = match devaluation(&world.config, s, i) {
                    Some(d) => s.resource[i] * (1.0 / (1.0 + d)),
                    None => s.resource[i],
                };
                held[i] + counted
            });
            crate::econ::foresight_welfare_n(&after[..n], &mets[..n], phi)
        })
    } else {
        Box::new(|p: Pos| {
            let s = world.site(p);
            match devaluation(&world.config, s, 0) {
                Some(d) => s.resource[0] / (1.0 + d),
                None => s.resource[0],
            }
        })
    };
    let mut out = vec![(pos, 0, value(pos))];
    for (q, d) in world.torus.sight(pos, vision) {
        if !world.is_occupied(q) {
            out.push((q, d, value(q)));
        }
    }
    out
}

/// Moves `id` to `target` (its own site to stay), records its neighbors, and
/// gathers every good there.
pub(crate) fn go_and_gather(world: &mut World, id: AgentId, target: Pos) -> Harvest {
    let n = world.config.goods.len();
    let a = world.agent(id).expect("live agent");
    let (tags, mut social) = (a.tags, a.social);
    world.move_agent(id, target);
    social.moved(world, Seen::at(world, target), tags);
    let site = world.site_mut(target);
    let mut harvest = Harvest::default();
    for (got, level) in harvest
        .gathered
        .iter_mut()
        .zip(site.resource.iter_mut())
        .take(n)
    {
        *got = *level;
        *level = 0.0;
    }
    let a = world.agent_mut(id).expect("live agent");
    for (have, got) in a.holdings.iter_mut().zip(&harvest.gathered).take(n) {
        *have += got;
    }
    a.social = social;
    harvest
}
```

Keep every expression exactly as it was: `s.resource[i] * (1.0 / (1.0 + d))` for n ≥ 2, and `s.resource[0] / (1.0 + d)` for one good. Floating-point results must not change. If `effective_metabolisms` returns an owned array, the `move` closure takes it by value.

- [ ] **Step 4: Run every test that could see a changed stream**

Run: `cargo test -p sugarscape-core`
Expected: all pass, including `tests/golden.rs` (every entry unchanged) and `tests/legacy.rs`. If any golden entry changes, the split changed an expression or its order. Fix the code, never the entry.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/rules/movement.rs
git commit -m "Split rule M into its candidates, its choice and the move and harvest, for the decision seam

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 3: The seam and the utility mind, with the reduction test

**Files:**
- Create: `crates/sugarscape-core/src/minds/mod.rs`
- Create: `crates/sugarscape-core/src/minds/utility.rs`
- Modify: `crates/sugarscape-core/src/lib.rs` (add `pub mod minds;` after `pub mod landscape;`)
- Modify: `crates/sugarscape-core/src/rules/mod.rs` (`agent_turn`)
- Create: `crates/sugarscape-core/tests/minds.rs`

**Interfaces:**
- Consumes: `movement::{candidates, choose, go_and_gather}` (Task 2); `config::{Decision, DecisionRule, Idle}` (Task 1); `portable::{ln, exp_neg}`.
- Produces:
  - `minds::decide(world: &mut World, id: AgentId) -> Harvest`;
  - `minds::utility::score(welfare: f64, distance: u32, crowd: u32, d: &Decision) -> f64` (pub, for the survey and tests);
  - `minds::utility::crowd(world: &World, site: Pos, mover: AgentId) -> u32`.

- [ ] **Step 1: Write the failing unit tests.** Create `minds/utility.rs` with only the tests module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Decision, DecisionRule, Idle};
    use crate::geometry::Pos;
    use crate::testkit::*;

    fn utility(travel: f64, crowding: f64, idle: Idle) -> Decision {
        Decision { rule: DecisionRule::Utility, travel, crowding, idle }
    }

    #[test]
    fn neutral_considerations_leave_welfare_exactly() {
        let d = utility(0.0, 0.0, Idle::Stay);
        for w in [0.0, 0.25, 3.0, 1e-300, 12345.678] {
            for (dist, n) in [(0, 0), (6, 4), (20, 3)] {
                assert_eq!(score(w, dist, n, &d).to_bits(), w.to_bits());
            }
        }
    }

    #[test]
    fn travel_discounts_hyperbolically_and_crowding_as_a_power() {
        let t = utility(0.5, 0.0, Idle::Stay);
        assert_eq!(score(4.0, 2, 0, &t), 4.0 / 2.0);
        assert_eq!(score(4.0, 0, 0, &t), 4.0);
        let c = utility(0.0, 1.0, Idle::Stay);
        assert!((score(4.0, 0, 1, &c) - 2.0).abs() < 1e-12); // 4 · 2^−1
        assert!((score(4.0, 0, 3, &c) - 1.0).abs() < 1e-12); // 4 · 4^−1
        assert_eq!(score(4.0, 0, 0, &c), 4.0);
        let c2 = utility(0.0, 2.0, Idle::Stay);
        assert!((score(9.0, 0, 2, &c2) - 1.0).abs() < 1e-12); // 9 · 3^−2
    }

    #[test]
    fn crowd_counts_neighbors_but_never_the_mover() {
        let mut w = blank_world(11, 11);
        let me = spawn(&mut w, 5, 5);
        spawn(&mut w, 5, 4);
        spawn(&mut w, 4, 5);
        assert_eq!(crowd(&w, Pos::new(5, 5), me), 2);
        // (6, 5)'s neighbors are (6, 4), (6, 6), (7, 5) and (5, 5): only the mover.
        assert_eq!(crowd(&w, Pos::new(6, 5), me), 0);
        // (4, 4)'s neighbors hold (5, 4) and (4, 5).
        assert_eq!(crowd(&w, Pos::new(4, 4), me), 2);
    }

    #[test]
    fn crowding_turns_a_flump_away_from_a_crowded_site() {
        let mut w = blank_world(11, 11);
        w.config.decision = utility(0.0, 1.0, Idle::Stay);
        let me = spawn(&mut w, 5, 5);
        w.agent_mut(me).unwrap().vision = 3;
        set_sugar(&mut w, 5, 8, 3.0); // neighbors (4, 8), (6, 8): crowd 2, score 1
        spawn(&mut w, 4, 8);
        spawn(&mut w, 6, 8);
        set_sugar(&mut w, 8, 5, 2.0); // no crowd: score 2
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(8, 5));
    }

    #[test]
    fn wander_moves_only_when_nothing_in_sight_scores_and_stay_does_not() {
        let run = |idle: Idle| {
            let mut w = blank_world(11, 11);
            w.config.decision = utility(0.0, 0.0, idle);
            let me = spawn(&mut w, 5, 5);
            w.agent_mut(me).unwrap().vision = 2;
            act(&mut w, me);
            w.agent(me).unwrap().pos
        };
        assert_eq!(run(Idle::Stay), Pos::new(5, 5));
        let p = run(Idle::Wander);
        assert_ne!(p, Pos::new(5, 5));
        assert!((p.x == 5) != (p.y == 5), "a site in sight along one axis");
    }

    #[test]
    fn a_wanderer_still_takes_sugar_in_sight() {
        let mut w = blank_world(11, 11);
        w.config.decision = utility(0.0, 0.0, Idle::Wander);
        let me = spawn(&mut w, 5, 5);
        w.agent_mut(me).unwrap().vision = 2;
        set_sugar(&mut w, 7, 5, 1.0);
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(7, 5));
    }

    #[test]
    fn a_boxed_in_wanderer_stays_and_draws_nothing() {
        let mut w = blank_world(11, 11);
        w.config.decision = utility(0.0, 0.0, Idle::Wander);
        let me = spawn(&mut w, 5, 5);
        for (x, y) in [(5, 4), (5, 6), (4, 5), (6, 5)] {
            spawn(&mut w, x, y);
        }
        let before = w.rng.clone();
        act(&mut w, me);
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(5, 5));
        // Staying draws exactly as rule M's choose over one candidate draws.
        let mut expected = before;
        crate::rules::movement::choose(&[(Pos::new(5, 5), 0, 0.0)], &mut expected);
        assert_eq!(w.rng, expected);
    }
}
```

Create `minds/mod.rs`:

```rust
//! The decision seam (Minds 1): which engine makes rule M's decision of
//! where an agent moves. `book` is rule M itself; each Minds step adds an
//! engine beside it, and every engine reduces to rule M when it sees and
//! values only what rule M does.

pub mod utility;

use crate::agent::AgentId;
use crate::config::DecisionRule;
use crate::rules::{movement, Harvest};
use crate::world::World;

/// Rule M's step under the configured decision rule: moves `id` and returns
/// its harvest.
pub(crate) fn decide(world: &mut World, id: AgentId) -> Harvest {
    match world.config.decision.rule {
        DecisionRule::Book => movement::act(world, id),
        DecisionRule::Utility => utility::act(world, id),
    }
}
```

Check that `rules::movement` is visible from `minds`. If `movement` is `pub mod` in `rules/mod.rs` (it is) and `choose`, `candidates` and `go_and_gather` are `pub(crate)`, this compiles. If `World.rng` doesn't implement `PartialEq` or `Clone` for the boxed-in test, compare two `gen::<u64>()` draws from each instead.

- [ ] **Step 2: Run the tests and check they fail**

Run: `cargo test -p sugarscape-core --lib minds`
Expected: compile errors (`score`, `crowd`, `act` not found).

- [ ] **Step 3: Implement.** At the top of `minds/utility.rs`:

```rust
//! The utility mind (Minds 1; Mark 2009, Dill and Mark 2010, Lewis 2017):
//! each candidate site's score is the product of its considerations —
//! rule M's welfare W, travel T(d) = 1 / (1 + k·d) and crowding
//! C = (1 + n)^(−m), n the Flumps on the site's von Neumann neighbors other
//! than the mover. With k = m = 0 the score is W exactly, and the choice is
//! rule M's (same candidates, tie rule and draw).
//!
//! Stated choices (the spec): W is not normalized (dividing by a constant
//! shared by every candidate doesn't change the order); no compensation
//! factor (every candidate has the same considerations); crowding is local.

use rand::seq::SliceRandom;

use crate::agent::AgentId;
use crate::config::{Decision, Idle};
use crate::geometry::Pos;
use crate::portable::{exp_neg, ln};
use crate::rules::movement::{candidates, choose, go_and_gather};
use crate::rules::Harvest;
use crate::world::World;

/// W · T(d) · C(n). A consideration at its neutral value (k = 0, m = 0) is
/// not computed, so the score is then `welfare` bit for bit.
pub fn score(welfare: f64, distance: u32, crowd: u32, d: &Decision) -> f64 {
    let mut s = welfare;
    if d.travel > 0.0 {
        s /= 1.0 + d.travel * f64::from(distance);
    }
    if d.crowding > 0.0 {
        s *= exp_neg(-d.crowding * ln(1.0 + f64::from(crowd)));
    }
    s
}

/// Flumps on `site`'s four von Neumann neighbors, not counting `mover`.
pub fn crowd(world: &World, site: Pos, mover: AgentId) -> u32 {
    world
        .torus
        .neighbors(site)
        .into_iter()
        .filter(|&q| world.occupant(q).is_some_and(|o| o != mover))
        .count() as u32
}

/// Rule M's step under the utility mind.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let d = world.config.decision;
    let mut scored = candidates(world, id);
    if d.crowding > 0.0 || d.travel > 0.0 {
        for c in &mut scored {
            let n = if d.crowding > 0.0 { crowd(world, c.0, id) } else { 0 };
            c.2 = score(c.2, c.1, n, &d);
        }
    }
    let target = if d.idle == Idle::Wander && scored.iter().all(|c| c.2 == 0.0) {
        // The current site is scored[0]; wander among the others, if any.
        match scored[1..].choose(&mut world.rng) {
            Some(c) => c.0,
            None => {
                return go_and_gather(world, id, choose(&scored, &mut world.rng));
            }
        }
    } else {
        choose(&scored, &mut world.rng)
    };
    go_and_gather(world, id, target)
}
```

The boxed-in case uses `choose` over the single candidate, drawing exactly as rule M would. That keeps "stays" identical to `stay`, and the test pins it. (`scored[1..].choose` on an empty slice draws nothing.)

In `rules/mod.rs`, replace `movement::act(world, id)` in `agent_turn` with `crate::minds::decide(world, id)`.

In `lib.rs`, add `pub mod minds;`.

- [ ] **Step 4: Write the reduction test.** Create `crates/sugarscape-core/tests/minds.rs`:

```rust
//! Minds 1's reduction: the utility mind with rule M's single consideration
//! (travel 0, crowding 0, idle stay) is rule M. Every golden Sugarscape
//! preset without combat gives the same fingerprint under either rule.

use sugarscape_core::config::{Config, DecisionRule, Idle};
use sugarscape_core::presets;
use sugarscape_core::world::World;

fn fingerprint(config: Config) -> u64 {
    let mut w = World::new(config, 1).unwrap();
    w.run(200);
    w.fingerprint()
}

#[test]
fn the_utility_mind_with_rule_ms_consideration_is_rule_m() {
    let mut checked = 0;
    for p in presets::all() {
        if p.config.combat.enabled {
            continue;
        }
        let mut c = p.config.clone();
        c.decision.rule = DecisionRule::Utility;
        c.decision.travel = 0.0;
        c.decision.crowding = 0.0;
        c.decision.idle = Idle::Stay;
        assert_eq!(fingerprint(c), fingerprint(p.config.clone()), "{}", p.id);
        checked += 1;
    }
    assert!(checked >= 20, "checked {checked} presets");
}
```

- [ ] **Step 5: Run everything**

Run: `cargo test -p sugarscape-core`
Expected: all pass: the new unit tests, `tests/minds.rs`, and every golden and legacy test unchanged.

- [ ] **Step 6: Commit**

```bash
git add crates/sugarscape-core/src/minds crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/rules/mod.rs crates/sugarscape-core/tests/minds.rs
git commit -m "Minds 1: the decision seam and the utility mind, which reduces to rule M on every golden preset

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 4: Patch statistics

**Files:**
- Modify: `crates/sugarscape-core/src/landscape.rs` (after `peak_capacity`, ~`:92`)
- Modify: `crates/sugarscape-core/src/stats.rs` (`series_names` `:53-71`, `Snapshot` `:73-118`, `Snapshot::of`, `Snapshot::value` `:284-289`)

**Interfaces:**
- Produces:
  - `landscape::peak_distance(p: &Peak, x: u32, y: u32, w: u32, h: u32) -> f64`;
  - `landscape::patch_of(peaks: &[Peak], x: u32, y: u32, w: u32, h: u32) -> Option<usize>`;
  - `stats::PatchStats { on_first: u32, on_other: u32, off: u32 }`;
  - `Snapshot.patches: Option<PatchStats>`;
  - series `on_first_patch`, `on_other_patches`, `off_patch` and `first_patch_share`, present iff goods[0]'s map is `peaks` with ≥ 2 peaks.

- [ ] **Step 1: Write the failing tests**

In `landscape.rs` tests (create `#[cfg(test)] mod tests` if none exists; `use super::*;`):

```rust
    #[test]
    fn a_site_belongs_to_its_nearest_peaks_patch_inside_the_radius() {
        let peaks = [
            Peak { x: 15, y: 20, radius: 10.0, height: 4.0 },
            Peak { x: 42, y: 20, radius: 7.0, height: 4.0 },
        ];
        assert_eq!(patch_of(&peaks, 15, 20, 60, 40), Some(0));
        assert_eq!(patch_of(&peaks, 24, 20, 60, 40), Some(0)); // d 9 < 10
        assert_eq!(patch_of(&peaks, 25, 20, 60, 40), None); // d 10: not inside
        assert_eq!(patch_of(&peaks, 36, 20, 60, 40), Some(1)); // d 6 < 7
        assert_eq!(patch_of(&peaks, 30, 20, 60, 40), None);
        // Across the seam: x 58 is 17 from 15 one way, 43 the other.
        assert_eq!(peak_distance(&peaks[0], 58, 20, 60, 40), 17.0);
        assert_eq!(patch_of(&peaks, 7, 39, 60, 40), None); // d √(64+361)
        // The patch sites match capacity ≥ 1 exactly (the spec's nominal input).
        let caps = capacities_for_test(&peaks, 60, 40);
        for y in 0..40 {
            for x in 0..60 {
                let inside = patch_of(&peaks, x, y, 60, 40).is_some();
                assert_eq!(inside, caps[(y * 60 + x) as usize] >= 1.0, "({x}, {y})");
            }
        }
    }
```

Here `capacities_for_test` is a test-only helper calling `peak_capacity(&peaks, x as usize, y as usize, 60, 40)` for every site; write it inside the tests module.

In `stats.rs` tests:

```rust
    fn two_patch_world() -> World {
        let mut c = Config::default();
        c.width = 60;
        c.height = 40;
        c.population = 0;
        c.goods[0].map = crate::config::Map::Peaks {
            peaks: vec![
                crate::config::Peak { x: 15, y: 20, radius: 10.0, height: 4.0 },
                crate::config::Peak { x: 42, y: 20, radius: 7.0, height: 4.0 },
            ],
        };
        World::new(c, 1).unwrap()
    }

    #[test]
    fn patch_series_exist_only_on_maps_of_two_or_more_peaks() {
        let names = |c: &Config| series_names(c);
        let two = two_patch_world().config.clone();
        for s in ["on_first_patch", "on_other_patches", "off_patch", "first_patch_share"] {
            assert!(names(&two).contains(&s.to_string()), "{s}");
            assert!(!names(&Config::default()).contains(&s.to_string()), "{s} on two_peaks");
        }
        let mut one = two.clone();
        if let crate::config::Map::Peaks { peaks } = &mut one.goods[0].map {
            peaks.truncate(1);
        }
        assert!(!names(&one).contains(&"off_patch".to_string()));
        let snap = Snapshot::of(&World::new(Config::default(), 1).unwrap());
        assert!(snap.patches.is_none());
        assert_eq!(snap.value("off_patch"), None);
    }

    #[test]
    fn patch_series_count_flumps_by_patch() {
        let mut w = two_patch_world();
        assert_eq!(Snapshot::of(&w).value("first_patch_share"), Some(0.0)); // nobody: 0, not NaN
        for (x, y) in [(15, 20), (16, 20), (42, 20), (30, 5)] {
            crate::testkit::spawn(&mut w, x, y);
        }
        let s = Snapshot::of(&w);
        assert_eq!(s.value("on_first_patch"), Some(2.0));
        assert_eq!(s.value("on_other_patches"), Some(1.0));
        assert_eq!(s.value("off_patch"), Some(1.0));
        assert_eq!(s.value("first_patch_share"), Some(2.0 / 3.0));
    }
```

If `testkit` is `#[cfg(test)]`-only, it's reachable from `stats`'s tests as `crate::testkit`.

- [ ] **Step 2: Run them and check they fail**

Run: `cargo test -p sugarscape-core --lib patch`
Expected: compile errors.

- [ ] **Step 3: Implement.** In `landscape.rs`:

```rust
/// Torus (Euclidean) distance from (x, y) to `p`'s center on a w × h grid,
/// as `peak_capacity` measures it.
pub fn peak_distance(p: &Peak, x: u32, y: u32, w: u32, h: u32) -> f64 {
    let torus = |a: u32, b: u32, n: u32| {
        let d = (f64::from(a) - f64::from(b)).abs();
        d.min(f64::from(n) - d)
    };
    let (dx, dy) = (torus(x, p.x, w), torus(y, p.y, h));
    (dx * dx + dy * dy).sqrt()
}

/// The patch holding (x, y) (Minds 1): its nearest peak (the lower index on
/// ties), when (x, y) is closer than that peak's radius — exactly the sites
/// where that peak gives capacity ≥ 1.
pub fn patch_of(peaks: &[Peak], x: u32, y: u32, w: u32, h: u32) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (k, p) in peaks.iter().enumerate() {
        let d = peak_distance(p, x, y, w, h);
        if best.is_none_or(|(_, b)| d < b) {
            best = Some((k, d));
        }
    }
    best.filter(|&(k, d)| d < peaks[k].radius).map(|(k, _)| k)
}
```

Use `map_or(true, …)` instead of `is_none_or` if the toolchain predates Rust 1.82.

In `stats.rs`:

```rust
/// Minds 1's patch counts: Flumps on the first peak's patch, on any other's,
/// and on none (`landscape::patch_of`). Present when goods[0]'s map is
/// `peaks` with at least two peaks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct PatchStats {
    pub on_first: u32,
    pub on_other: u32,
    pub off: u32,
}

/// goods[0]'s peaks when there are at least two (the patch series' condition).
fn patches(config: &Config) -> Option<&[crate::config::Peak]> {
    match &config.goods[0].map {
        crate::config::Map::Peaks { peaks } if peaks.len() >= 2 => Some(peaks),
        _ => None,
    }
}
```

- In `series_names`, after the Axelrod push: `if patches(config).is_some() { for s in ["on_first_patch", "on_other_patches", "off_patch", "first_patch_share"] { names.push(s.into()); } }`.
- Update its doc comment to mention the patch series last.
- In `Snapshot`, add after `axelrod`: `#[serde(skip_serializing_if = "Option::is_none")] pub patches: Option<PatchStats>,` with a doc comment.
- In `Snapshot::of`, add:

```rust
            patches: patches(&world.config).map(|peaks| {
                let (w, h) = (world.config.width, world.config.height);
                let mut p = PatchStats::default();
                for a in world.agents() {
                    match crate::landscape::patch_of(peaks, a.pos.x, a.pos.y, w, h) {
                        Some(0) => p.on_first += 1,
                        Some(_) => p.on_other += 1,
                        None => p.off += 1,
                    }
                }
                p
            }),
```

- In `value`'s fallback, before `return None;`:

```rust
                if let Some(p) = self.patches {
                    match name {
                        "on_first_patch" => return Some(f64::from(p.on_first)),
                        "on_other_patches" => return Some(f64::from(p.on_other)),
                        "off_patch" => return Some(f64::from(p.off)),
                        "first_patch_share" => {
                            let on = p.on_first + p.on_other;
                            return Some(if on == 0 { 0.0 } else { f64::from(p.on_first) / f64::from(on) });
                        }
                        _ => {}
                    }
                }
```

Check `Pos`'s field types (`x`, `y` as `u32`). If `Snapshot` is built elsewhere with a struct literal (grep `Snapshot {`), add `patches: None` there.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core && cargo test -p sugarscape-cli`
Expected: all pass. The pinned header tests (`export.rs:96`, `stats.rs:691`, `stats.rs:1081`) are unchanged, because default configs have no patch series.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/landscape.rs crates/sugarscape-core/src/stats.rs
git commit -m "Minds 1: count Flumps on each patch of a two-peak map

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 5: The eight presets, their titles and golden entries

**Files:**
- Modify: `crates/sugarscape-core/src/presets.rs` (append to `all()` after `dock-mobility-30`; a helper after `docking`; the count in `every_preset_is_valid_and_runs` `:794`, 31 → 39)
- Modify: `crates/sugarscape-core/src/titles.rs` (`[(&str, &str); 202]` → `210`; insert after the `dock-mobility-30` entry at `:125`)
- Modify: `crates/sugarscape-core/tests/golden.rs` (`GOLDEN`, after `dock-mobility-30`)

**Interfaces:**
- Consumes: `Decision`, `DecisionRule` and `Idle` (Task 1).
- Produces: presets `ifd-even`, `ifd-two-to-one`, `ifd-four-to-one`, `ifd-far-sighted`, `ifd-no-starving`, `ifd-wander`, `ifd-crowding` and `ifd-travel`, and the helper `presets::two_patches(c: &mut Config, second_radius: f64)` (`pub(crate)`).

- [ ] **Step 1: Write the failing test** (in `presets.rs` tests)

```rust
    #[test]
    fn the_ifd_presets_share_the_two_patch_world() {
        let ids = [
            "ifd-even", "ifd-two-to-one", "ifd-four-to-one", "ifd-far-sighted",
            "ifd-no-starving", "ifd-wander", "ifd-crowding", "ifd-travel",
        ];
        for id in ids {
            let c = by_id(id).unwrap_or_else(|| panic!("{id}")).config;
            assert_eq!((c.width, c.height, c.population), (60, 40, 100), "{id}");
            assert_eq!(c.growback.rate, 0.25, "{id}");
            let Map::Peaks { peaks } = &c.goods[0].map else { panic!("{id}: peaks") };
            assert_eq!((peaks[0].x, peaks[0].y, peaks[0].radius), (15, 20, 10.0), "{id}");
            assert_eq!((peaks[1].x, peaks[1].y), (42, 20), "{id}");
        }
        let radius = |id: &str| match &by_id(id).unwrap().config.goods[0].map {
            Map::Peaks { peaks } => peaks[1].radius,
            _ => unreachable!(),
        };
        assert_eq!([radius("ifd-even"), radius("ifd-two-to-one"), radius("ifd-four-to-one")], [10.0, 7.0, 5.0]);
        let d = |id: &str| by_id(id).unwrap().config.decision;
        assert_eq!(d("ifd-two-to-one").rule, DecisionRule::Book);
        assert_eq!((d("ifd-wander").rule, d("ifd-wander").idle), (DecisionRule::Utility, Idle::Wander));
        assert_eq!(d("ifd-crowding").crowding, 1.0);
        assert_eq!(d("ifd-travel").travel, 0.5);
        assert_eq!(by_id("ifd-far-sighted").unwrap().config.vision, URange::new(10, 20));
        assert_eq!(by_id("ifd-no-starving").unwrap().config.goods[0].endowment, URange::new(100_000, 100_000));
    }
```

Import `DecisionRule` and `Idle` in the tests module (`use crate::config::{DecisionRule, Idle};`).

- [ ] **Step 2: Run it and check it fails**

Run: `cargo test -p sugarscape-core --lib presets::tests::the_ifd`
Expected: FAIL (unknown preset).

- [ ] **Step 3: Implement.** The helper, after `docking`:

```rust
/// Minds 1's world (the spec's probe): a 60 × 40 torus with two cone patches
/// of height 4 at (15, 20), radius 10, and (42, 20), radius `second_radius`;
/// sugar grows back 0.25 a tick; 100 Flumps with metabolism 1, endowment 50
/// and vision 1–6. Nominal inputs (sites with capacity ≥ 1, × 0.25): 305
/// against 305, 225, 145, 109 and 69 sites at radius 10, 8.5, 7, 6 and 5
/// (R 1.00, 1.36, 2.10, 2.80, 4.42); about 112 Flumps can be fed at R 2.10.
pub(crate) fn two_patches(c: &mut Config, second_radius: f64) {
    c.width = 60;
    c.height = 40;
    c.population = 100;
    c.goods[0].map = Map::Peaks {
        peaks: vec![
            Peak { x: 15, y: 20, radius: 10.0, height: 4.0 },
            Peak { x: 42, y: 20, radius: second_radius, height: 4.0 },
        ],
    };
    c.goods[0].metabolism = URange::new(1, 1);
    c.goods[0].endowment = URange::new(50, 50);
    c.growback.rate = 0.25;
}
```

Presets, appended to `all()`. The descriptions carry the probe's numbers now. Task 9 replaces the "Measured" sentences with the survey's numbers.

```rust
        preset(
            "ifd-even",
            "Ideal free distribution: equal patches",
            "Fretwell & Lucas 1969; Minds 1",
            "Two cone-shaped sugar patches of the same size (305 sites each, input 0.25 a tick per site) on a 60 × 40 torus, and 100 Flumps of metabolism 1 and vision 1–6. The ideal free distribution predicts an even split. Measured in planning (20 seeds, tick 1000): 0.98 as many Flumps on the first patch as the second.",
            |c| two_patches(c, 10.0),
        ),
        preset(
            "ifd-two-to-one",
            "Ideal free distribution: 2.1 : 1",
            "Parker 1978; Milinski 1979; Minds 1",
            "The second patch has radius 7 (145 sites against 305: input ratio 2.10, near Milinski's 2 : 1). Input matching predicts 2.10 times as many Flumps on the richer patch. Measured in planning (20 seeds, tick 1000): 1.71 times; with ratios 1, 2.10 and 4.42 the matching exponent s is 0.745 (s = 1 is matching; below 1, undermatching, as Kennedy & Gray 1993 report for most animal experiments).",
            |c| two_patches(c, 7.0),
        ),
        preset(
            "ifd-four-to-one",
            "Ideal free distribution: 4.4 : 1",
            "Parker 1978; Minds 1",
            "The second patch has radius 5 (69 sites: input ratio 4.42). Input matching predicts 4.42 times as many Flumps on the richer patch. Measured in planning (20 seeds, tick 1000): 2.96 times.",
            |c| two_patches(c, 5.0),
        ),
        preset(
            "ifd-far-sighted",
            "Ideal free distribution: vision 10–20",
            "Kennedy & Gray 1993; Minds 1",
            "The 2.10 : 1 patches with vision 10–20, far enough to see across the 7-site gap between the patches. Measured in planning (20 seeds, tick 1000): 1.86 times as many Flumps on the richer patch; s 0.92 across ratios 1, 2.10 and 4.42 — close to matching.",
            |c| {
                two_patches(c, 7.0);
                c.vision = URange::new(10, 20);
            },
        ),
        preset(
            "ifd-no-starving",
            "Ideal free distribution: nobody starves",
            "Fretwell & Lucas 1969; Minds 1",
            "The 2.10 : 1 patches with an endowment of 100 000, so nobody starves within the run. The on-patch counts are the same as with starvation: the Flumps who die are the ones who never find sugar. Rule M keeps a Flump in place when nothing it sees is better, so one that starts out of sight of sugar never moves. Measured in planning (20 seeds, tick 1000): 65 of 100 off both patches.",
            |c| {
                two_patches(c, 7.0);
                c.goods[0].endowment = URange::new(100_000, 100_000);
            },
        ),
        preset(
            "ifd-wander",
            "Utility mind: wander when nothing scores",
            "Minds 1",
            "The no-starving world under the utility mind with idle wander: a Flump that sees no sugar moves to a random free site in sight instead of staying put. It tests the 'free' of the ideal free distribution apart from the 'ideal'.",
            |c| {
                two_patches(c, 7.0);
                c.goods[0].endowment = URange::new(100_000, 100_000);
                c.decision.rule = DecisionRule::Utility;
                c.decision.idle = Idle::Wander;
            },
        ),
        preset(
            "ifd-crowding",
            "Utility mind: crowding m = 1",
            "Sutherland 1983; Minds 1",
            "The 2.10 : 1 patches under the utility mind with crowding m = 1: a site's welfare is divided by (1 + n), n the Flumps next to it. Sutherland's interference model predicts input matching at m = 1; here the interference is local.",
            |c| {
                two_patches(c, 7.0);
                c.decision.rule = DecisionRule::Utility;
                c.decision.crowding = 1.0;
            },
        ),
        preset(
            "ifd-travel",
            "Utility mind: travel k = 0.5",
            "Baum & Kraft 1998; Minds 1",
            "The 2.10 : 1 patches under the utility mind with travel k = 0.5: a site's welfare is divided by (1 + 0.5·d), d its distance. Baum & Kraft found that requiring travel to switch patches slightly reduced undermatching; here travel is a preference for nearby sugar under rule M's one-tick jump, not a cost of switching.",
            |c| {
                two_patches(c, 7.0);
                c.decision.rule = DecisionRule::Utility;
                c.decision.travel = 0.5;
            },
        ),
```

Import `DecisionRule` and `Idle` at the top of `presets.rs`.

In `titles.rs`, bump to `210` and insert after the `dock-mobility-30` tuple:

```rust
    ("ifd-even", "Two equal sugar patches: the Flumps split evenly"),
    ("ifd-two-to-one", "One patch yields twice as much, but draws fewer than twice the Flumps"),
    ("ifd-four-to-one", "One patch yields four times as much, but draws under three times the Flumps"),
    ("ifd-far-sighted", "Flumps who can see across the gap come close to matching the yields"),
    ("ifd-no-starving", "Nobody starves, and most Flumps never find sugar"),
    ("ifd-wander", "Flumps who see no sugar wander instead of waiting"),
    ("ifd-crowding", "Flumps who avoid crowded sugar"),
    ("ifd-travel", "Flumps who prefer nearby sugar"),
```

Bump `every_preset_is_valid_and_runs`'s count from 31 to 39.

- [ ] **Step 4: Record the golden entries**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden 2>&1 | grep ifd-`
Paste the eight lines into `GOLDEN` after `dock-mobility-30`, under a comment `// Minds 1: the ideal free distribution (two patches; the utility mind).`

- [ ] **Step 5: Run the tests**

Run: `cargo test -p sugarscape-core`
Expected: all pass, including the titles test, the reduction test (now covering the book `ifd-*` presets too) and golden.

- [ ] **Step 6: Commit**

```bash
git add crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs crates/sugarscape-core/tests/golden.rs
git commit -m "Minds 1: eight two-patch presets for the ideal free distribution

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 6: The four sweeps

**Files:**
- Create: `sweeps/ifd-matching.json`, `sweeps/ifd-idle.json`, `sweeps/ifd-crowding.json`, `sweeps/ifd-travel.json`
- Modify: `crates/sugarscape-core/src/sweep.rs` (`BUILTINS: [Builtin; 80]` → `84`, appended at the end; the id list in `builtin_sweeps_parse_and_validate`, appended after `"cmo-bias"`)

**Interfaces:**
- Consumes: the presets from Task 5 and the `first_patch_share` series from Task 4.

- [ ] **Step 1: Write the failing test.** Append the four ids to the expected list in `builtin_sweeps_parse_and_validate` (after `"cmo-bias"`): `"ifd-matching", "ifd-idle", "ifd-crowding", "ifd-travel"`.

- [ ] **Step 2: Run it and check it fails**

Run: `cargo test -p sugarscape-core --lib sweep::tests::builtin_sweeps_parse_and_validate`
Expected: FAIL (the lists differ).

- [ ] **Step 3: Write the files.** The input-ratio axis is shared by `ifd-matching` and `ifd-idle`:

```json
    "label": "Input ratio (first patch : second)",
    "values": [
      { "at": 1.0, "set": { "goods.0.map.peaks.1.radius": 10.0 } },
      { "at": 1.36, "set": { "goods.0.map.peaks.1.radius": 8.5 } },
      { "at": 2.1, "set": { "goods.0.map.peaks.1.radius": 7.0 } },
      { "at": 2.8, "set": { "goods.0.map.peaks.1.radius": 6.0 } },
      { "at": 4.42, "set": { "goods.0.map.peaks.1.radius": 5.0 } }
    ]
```

`sweeps/ifd-matching.json`:

```json
{
  "name": "Minds 1: the ideal free distribution under rule M",
  "description": "The share of on-patch Flumps on the richer patch (mean over ticks 500–1000) against the input ratio of the two patches, at three visions, under the book's rule M. Input matching (Parker 1978) predicts a share of R / (1 + R): 0.50, 0.58, 0.68, 0.74 and 0.82. Seeds 1–20.",
  "base": { "preset": "ifd-even" },
  "x": { …the input-ratio axis above… },
  "series": {
    "label": "Vision",
    "values": [
      { "at": 1, "name": "Vision 1–6", "set": { "vision": { "min": 1, "max": 6 } } },
      { "at": 2, "name": "Vision 5–10", "set": { "vision": { "min": 5, "max": 10 } } },
      { "at": 3, "name": "Vision 10–20", "set": { "vision": { "min": 10, "max": 20 } } }
    ]
  },
  "seeds": { "from": 1, "count": 20 },
  "ticks": 1000,
  "metric": { "kind": "window_mean", "series": "first_patch_share", "from": 500 }
}
```

`sweeps/ifd-idle.json` has the same `x`, `base` `ifd-no-starving`, and:

```json
  "name": "Minds 1: waiting or wandering when nothing is in sight",
  "description": "The richer patch's share of on-patch Flumps against the input ratio, with nobody starving, when a Flump that sees no sugar stays put (rule M) or wanders (the utility mind's idle wander). Seeds 1–20, ticks 500–1000.",
  "series": {
    "label": "Idle",
    "values": [
      { "at": 0, "name": "Stay (rule M)", "set": { "decision.rule": "book" } },
      { "at": 1, "name": "Wander", "set": { "decision.rule": "utility", "decision.idle": "wander" } }
    ]
  },
```

`sweeps/ifd-crowding.json`:

```json
{
  "name": "Minds 1: crowding and the ideal free distribution",
  "description": "The richer patch's share of on-patch Flumps against the utility mind's crowding m (a site's welfare times (1 + n)^−m, n the Flumps next to it), at input ratios 2.10 and 4.42 (matching: 0.68 and 0.82). Sutherland 1983 predicts matching at m = 1 for patch-wide interference; this crowding is local. Seeds 1–20, ticks 500–1000.",
  "base": { "preset": "ifd-crowding" },
  "x": {
    "label": "Crowding m",
    "path": "decision.crowding",
    "values": [0, 0.5, 1, 2, 4]
  },
  "series": {
    "label": "Input ratio",
    "values": [
      { "at": 2.1, "name": "2.10 : 1", "set": { "goods.0.map.peaks.1.radius": 7.0 } },
      { "at": 4.42, "name": "4.42 : 1", "set": { "goods.0.map.peaks.1.radius": 5.0 } }
    ]
  },
  "seeds": { "from": 1, "count": 20 },
  "ticks": 1000,
  "metric": { "kind": "window_mean", "series": "first_patch_share", "from": 500 }
}
```

`sweeps/ifd-travel.json` is the same with base `ifd-travel` and:

```json
  "name": "Minds 1: travel and the ideal free distribution",
  "description": "The richer patch's share of on-patch Flumps against the utility mind's travel k (a site's welfare divided by 1 + k·d, d its distance), at input ratios 2.10 and 4.42 (matching: 0.68 and 0.82). Baum & Kraft 1998 found travel between patches slightly reduced undermatching; here travel is a preference for nearby sugar under rule M's jump. Seeds 1–20, ticks 500–1000.",
  "x": { "label": "Travel k", "path": "decision.travel", "values": [0, 0.1, 0.5, 1, 2] },
```

In `sweep.rs`, append four `Builtin { id: "ifd-…", json: include_str!("../../../sweeps/ifd-….json") }` entries and bump 80 → 84.

- [ ] **Step 4: Run the tests and one sweep**

Run: `cargo test -p sugarscape-core --lib sweep && cargo run --release -p sugarscape-cli -- sweep --help`
Then run `ifd-matching` with the CLI's sweep command (use the flag `--help` shows for a built-in id), and check the share at x = 2.1 for vision 1–6 is near the probe's (ratio 1.71, share ≈ 0.63).
Expected: tests pass; the sweep runs in well under a minute.

- [ ] **Step 5: Commit**

```bash
git add sweeps/ifd-*.json crates/sugarscape-core/src/sweep.rs
git commit -m "Minds 1: sweeps of the ideal free distribution against input ratio, vision, idling, crowding and travel

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 7: WASM agreement

**Files:**
- Modify: `crates/sugarscape-wasm/tests/web.rs` (after `fingerprint_matches_the_golden_entry`, ~`:482`)

- [ ] **Step 1: Write the test.** Take `ifd-crowding`'s value from `GOLDEN`:

```rust
#[wasm_bindgen_test]
fn the_utility_minds_crowding_matches_its_golden_entry() {
    // Crowding uses portable ln and exp: native and wasm must agree.
    let preset = sugarscape_core::presets::by_id("ifd-crowding").unwrap();
    let json = serde_json::to_string(&preset.config).unwrap();
    let mut sim = Sim::new(&json, 1, JsValue::NULL).unwrap();
    sim.step(200);
    // crates/sugarscape-core/tests/golden.rs
    assert_eq!(sim.fingerprint(), "0x<ifd-crowding's golden value, 16 hex digits>");
}
```

- [ ] **Step 2: Run it**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS. A mismatch means a platform `ln` or `exp` crept into the score. Fix the code.

- [ ] **Step 3: Commit**

```bash
git add crates/sugarscape-wasm/tests/web.rs
git commit -m "Minds 1: pin the crowding preset's fingerprint in WASM

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 8: The page (Decision group, Patches chart)

**Files:**
- Modify: `web/src/types.ts:54-81` (`Config`)
- Modify: `web/src/schema.ts` (a group after `Foresight`, ~`:156`)
- Create: `web/src/patches.ts`, `web/src/patches.test.ts`
- Modify: `web/src/ui/charts-panel.ts` (a chart after `Distinct cultures`, ~`:137`)
- Modify: `web/src/schema.test.ts`

**Interfaces:**
- Produces: `hasPatches(c: Config): boolean` in `web/src/patches.ts`; `Config.decision?: Decision` in `types.ts`.

- [ ] **Step 1: Write the failing tests**

`web/src/patches.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { hasPatches } from './patches';
import type { Config } from './types';

const withMap = (map: unknown) => ({ goods: [{ map }] }) as unknown as Config;
const peak = { x: 1, y: 1, radius: 2, height: 4 };

describe('hasPatches', () => {
  it('is true only for a peaks map of two or more peaks', () => {
    expect(hasPatches(withMap({ kind: 'peaks', peaks: [peak, peak] }))).toBe(true);
    expect(hasPatches(withMap({ kind: 'peaks', peaks: [peak] }))).toBe(false);
    expect(hasPatches(withMap({ kind: 'two_peaks', transform: 'identity' }))).toBe(false);
  });
});
```

Append to `web/src/schema.test.ts`:

```ts
describe('decision', () => {
  it('offers the book and the utility mind, rebuilding the world, with live knobs', () => {
    const rule = control('decision.rule');
    expect(rule.kind).toBe('select');
    if (rule.kind !== 'select') return;
    expect(rule.reset).toBe(true);
    expect(rule.options.map((o) => o.value)).toEqual(['book', 'utility']);
    const c = {} as unknown as Config;
    expect(rule.current(c)).toBe('book'); // older configs have no decision
    rule.options[1].apply(c);
    expect(rule.current(c)).toBe('utility');
    expect(c.decision).toEqual({ rule: 'utility', travel: 0, crowding: 0, idle: 'stay' });
    for (const path of ['decision.travel', 'decision.crowding']) {
      const k = control(path);
      expect(k.kind).toBe('number');
      expect(k.reset).toBeUndefined();
    }
    const idle = control('decision.idle');
    if (idle.kind !== 'select') throw new Error('idle is a select');
    expect(idle.options.map((o) => o.value)).toEqual(['stay', 'wander']);
    const d = {} as unknown as Config;
    idle.options[1].apply(d);
    expect(d.decision?.idle).toBe('wander');
    expect(d.decision?.rule).toBe('book');
  });
});
```

- [ ] **Step 2: Run them and check they fail**

Run: `cd web && npx vitest run src/patches.test.ts src/schema.test.ts`
Expected: FAIL (no module; no control).

- [ ] **Step 3: Implement.** `types.ts`, before `export interface Config`:

```ts
/** Minds 1's decision seam (absent from older configs: the book's rule M). */
export interface Decision {
  rule: 'book' | 'utility';
  travel: number;
  crowding: number;
  idle: 'stay' | 'wander';
}
```

Add `decision?: Decision;` to `Config` before `schedule`.

`web/src/patches.ts`:

```ts
import type { Config } from './types';

/** Whether good 0's map has patches (Minds 1): a peaks map of two or more peaks. */
export function hasPatches(c: Config): boolean {
  const map = c.goods[0]?.map as { kind: string; peaks?: unknown[] } | undefined;
  return map?.kind === 'peaks' && (map.peaks?.length ?? 0) >= 2;
}
```

`schema.ts`, a group after `Foresight`:

```ts
  {
    title: 'Decision (Minds 1)',
    note: 'Which rule decides where a Flump moves. The book’s rule M goes to the best site in sight. The utility mind multiplies that welfare by travel and crowding considerations; with both at 0 and Idle at Stay it is rule M exactly. Travel, crowding and idle apply only under the utility mind; rule C decides moves under combat.',
    controls: [
      {
        kind: 'select', path: 'decision.rule', label: 'Rule', reset: true,
        current: (c) => c.decision?.rule ?? 'book',
        options: [
          { value: 'book', label: 'Rule M (book)', apply: (c) => { c.decision = { ...decision(c), rule: 'book' }; } },
          { value: 'utility', label: 'Utility mind', apply: (c) => { c.decision = { ...decision(c), rule: 'utility' }; } },
        ],
      },
      { kind: 'number', path: 'decision.travel', label: 'Travel k (welfare ÷ (1 + k·distance))', min: 0, max: 10, step: 0.1 },
      { kind: 'number', path: 'decision.crowding', label: 'Crowding m (welfare × (1 + neighbors)^−m)', min: 0, max: 10, step: 0.1 },
      {
        kind: 'select', path: 'decision.idle', label: 'When nothing in sight scores',
        current: (c) => c.decision?.idle ?? 'stay',
        options: [
          { value: 'stay', label: 'Stay (book)', apply: (c) => { c.decision = { ...decision(c), idle: 'stay' }; } },
          { value: 'wander', label: 'Wander to a random free site in sight', apply: (c) => { c.decision = { ...decision(c), idle: 'wander' }; } },
        ],
      },
    ],
  },
```

The helper at the top of `schema.ts`, with `import type { Config, Decision } from './types';`:

```ts
/** A config's decision, or the book's for older configs. */
const decision = (c: Config): Decision => c.decision ?? { rule: 'book', travel: 0, crowding: 0, idle: 'stay' };
```

Check how `rules-panel.ts` reads a `number` control's value at a path that may be missing (`decision.travel` on an older config). If it reads `undefined`, default the display to 0. When the path's parent is missing, the setter must create it from `decision(c)`, e.g. via `adjust: (next) => { next.decision = { ...decision(next), ...next.decision } }`. Add a schema test for setting `decision.travel` on a config without `decision`, if the setter is generic.

`charts-panel.ts`, after the `Distinct cultures` chart, with `import { hasPatches } from '../patches';`:

```ts
  {
    title: 'Patches',
    kind: 'time',
    section: 'top',
    lines: fixed([
      { key: 'on_first_patch', label: 'First patch', color: '--c1' },
      { key: 'on_other_patches', label: 'Other patches', color: '--c2' },
      { key: 'off_patch', label: 'Off patch', color: '--c3' },
    ]),
    shown: hasPatches,
  },
  {
    title: 'First patch share',
    kind: 'time',
    section: 'top',
    lines: fixed([{ key: 'first_patch_share', label: 'Share', color: '--c1' }]),
    range: [0, 1],
    shown: hasPatches,
  },
```

- [ ] **Step 4: Run the build and tests**

Run: `cd web && npm run build && npm test`
Expected: all pass (the build first, since the engine tests load the real WASM package).

- [ ] **Step 5: Check it in the browser.** Use the `run` skill (or `npm run dev`):
  - load `ifd-two-to-one`: the Patches charts appear, and the Decision group shows Rule M;
  - switch the rule to Utility mind: the world rebuilds;
  - set Crowding 1: it applies live;
  - load `ii-2-unit`: no Patches charts.

- [ ] **Step 6: Commit**

```bash
git add web/src/types.ts web/src/schema.ts web/src/schema.test.ts web/src/patches.ts web/src/patches.test.ts web/src/ui/charts-panel.ts
git commit -m "Minds 1 on the page: the Decision group and the Patches charts

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 9: The survey, and measured descriptions and titles

**Files:**
- Create: `survey/src/claims/minds1.rs`
- Modify: `survey/src/claims/mod.rs` (`mod minds1;` and `minds1::claims(),`, alphabetical after `image`)
- Modify: `crates/sugarscape-core/src/presets.rs` and `titles.rs` (measured wording)
- Copy the open sources into `papers/ideal-free/` and `papers/utility/`

**Interfaces:**
- Consumes: the presets (Task 5), `landscape::patch_of` (Task 4), `geometry::Torus::{new, sight}`, and `runner::{each_seed, preset, series}`.

- [ ] **Step 1: Write the module**

```rust
//! Minds 1: the ideal free distribution under rule M and the utility mind
//! (docs/superpowers/specs/2026-09-27-minds-1-utility-design.md). The
//! matching exponent s is fitted per seed across the five input ratios
//! from means of per-sample log ratios (Earn & Johnstone 1997), never from
//! ratios of mean counts.

use sugarscape_core::config::{Config, DecisionRule, Idle, Map, URange};
use sugarscape_core::geometry::{Pos, Torus};
use sugarscape_core::landscape::patch_of;
use sugarscape_core::world::World;

use crate::claim::{all_of, equivalent, greater, range, Claim, Outcome, Source};
use crate::runner::{each_seed, preset, series};

const SPEC: &str = "docs/superpowers/specs/2026-09-27-minds-1-utility-design.md";
const RADII: [f64; 5] = [10.0, 8.5, 7.0, 6.0, 5.0];

fn with_radius(mut c: Config, r: f64) -> Config {
    if let Map::Peaks { peaks } = &mut c.goods[0].map {
        peaks[1].radius = r;
    }
    c
}

/// Sites of patch `k` (capacity ≥ 1): the nominal input divided by the rate.
fn sites(c: &Config, k: usize) -> f64 {
    let Map::Peaks { peaks } = &c.goods[0].map else { unreachable!() };
    let mut n = 0;
    for y in 0..c.height {
        for x in 0..c.width {
            if patch_of(peaks, x, y, c.width, c.height) == Some(k) {
                n += 1;
            }
        }
    }
    f64::from(n)
}

/// Mean over ticks 500, 510, …, 1000 of ln(first / other), skipping samples
/// with an empty patch; NaN when every sample has one.
fn log_ratio(w: &World) -> f64 {
    let (a, b) = (series(w, "on_first_patch"), series(w, "on_other_patches"));
    let v: Vec<f64> = (500..=1000)
        .step_by(10)
        .filter(|&t| a[t] > 0.0 && b[t] > 0.0)
        .map(|t| (a[t] / b[t]).ln())
        .collect();
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

/// Least-squares slope of y on x over the finite pairs; NaN with fewer than 3.
fn slope(x: &[f64], y: &[f64]) -> f64 {
    let pts: Vec<(f64, f64)> = x.iter().zip(y).filter(|(_, b)| b.is_finite()).map(|(a, b)| (*a, *b)).collect();
    if pts.len() < 3 {
        return f64::NAN;
    }
    let n = pts.len() as f64;
    let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    sxy / sxx
}

/// Per seed: s across the five input ratios, starting from `base` edited by `edit`.
fn s_per_seed(base: &str, seeds: &[u64], edit: impl Fn(&mut Config) + Copy) -> Vec<f64> {
    let per_radius: Vec<Vec<f64>> = RADII
        .iter()
        .map(|&r| {
            let mut c = with_radius(preset(base), r);
            edit(&mut c);
            each_seed(&c, seeds, |mut w| {
                w.run(1000);
                log_ratio(&w)
            })
        })
        .collect();
    let ln_r: Vec<f64> = RADII
        .iter()
        .map(|&r| {
            let c = with_radius(preset(base), r);
            (sites(&c, 0) / sites(&c, 1)).ln()
        })
        .collect();
    (0..seeds.len())
        .map(|i| slope(&ln_r, &per_radius.iter().map(|v| v[i]).collect::<Vec<_>>()))
        .collect()
}

/// Share of Flumps off both patches at tick 1000, per seed.
fn off_share(c: &Config, seeds: &[u64]) -> Vec<f64> {
    each_seed(c, seeds, |mut w| {
        w.run(1000);
        let off = *series(&w, "off_patch").last().unwrap();
        let pop = f64::from(w.population());
        if pop == 0.0 { f64::NAN } else { off / pop }
    })
}

/// The catchment prediction of s at `vision`: the slope of ln(C₁/C₂) on
/// ln(R₁/R₂), C_k the expected number of sites from which a site of patch k
/// is in sight (or which is one), vision uniform on its range. A site that
/// sees both patches counts for both.
fn catchment_s(vision: URange) -> f64 {
    let (mut x, mut y) = (Vec::new(), Vec::new());
    for &r in &RADII {
        let c = with_radius(preset("ifd-even"), r);
        let Map::Peaks { peaks } = &c.goods[0].map else { unreachable!() };
        let torus = Torus::new(c.width, c.height);
        let catch = |k: usize| {
            let mut total = 0.0;
            for v in vision.min..=vision.max {
                let mut n = 0u32;
                for yy in 0..c.height {
                    for xx in 0..c.width {
                        let p = Pos::new(xx, yy);
                        let sees = patch_of(peaks, xx, yy, c.width, c.height) == Some(k)
                            || torus.sight(p, v).iter().any(|(q, _)| patch_of(peaks, q.x, q.y, c.width, c.height) == Some(k));
                        n += u32::from(sees);
                    }
                }
                total += f64::from(n);
            }
            total / f64::from(vision.max - vision.min + 1)
        };
        x.push((sites(&c, 0) / sites(&c, 1)).ln());
        y.push((catch(0) / catch(1)).ln());
    }
    slope(&x, &y)
}

fn vision(v: (u32, u32)) -> impl Fn(&mut Config) + Copy {
    move |c: &mut Config| c.vision = URange::new(v.0, v.1)
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ifd-matching.parker",
            item: "ifd-matching",
            source: Source::Book,
            citation: "Parker 1978 (via Collins, Houston & Lang 2002)",
            text: "Input matching: the ratio of animals at two sites matches the ratio of their input rates (s = 1), under rule M at vision 1–6 and 10–20",
            check: |seeds| all_of(
                [(1, 6), (10, 20)]
                    .into_iter()
                    .map(|v| (format!("vision {}–{}", v.0, v.1), range(&s_per_seed("ifd-even", seeds, vision(v)), 0.9, 1.1, false)))
                    .collect(),
            ),
        },
        Claim {
            id: "ifd-matching.undermatching",
            item: "ifd-matching",
            source: Source::Book,
            citation: "Kennedy & Gray 1993",
            text: "Animals undermatch: fewer than input matching predicts on the richer patch (s < 1), under rule M at each vision",
            check: |seeds| all_of(
                [(1, 6), (5, 10), (10, 20)]
                    .into_iter()
                    .map(|v| (format!("vision {}–{}", v.0, v.1), range(&s_per_seed("ifd-even", seeds, vision(v)), 0.0, 0.999, false)))
                    .collect(),
            ),
        },
        Claim {
            id: "ifd-no-starving.survival",
            item: "ifd-no-starving",
            source: Source::Comment,
            citation: SPEC,
            text: "Survival plays no part: s is the same with and without starvation (vision 1–6)",
            check: |seeds| {
                let starving = s_per_seed("ifd-even", seeds, |_| {});
                let fed = s_per_seed("ifd-even", seeds, |c| c.goods[0].endowment = URange::new(100_000, 100_000));
                equivalent(&fed, &starving, Some(0.05), "no starving", "starving")
            },
        },
        Claim {
            id: "ifd-no-starving.stuck",
            item: "ifd-no-starving",
            source: Source::Comment,
            citation: SPEC,
            text: "'Free' fails: under rule M most Flumps who start out of sight of sugar never move; over half of 100 are off both patches at tick 1000",
            check: |seeds| range(&off_share(&preset("ifd-no-starving"), seeds), 0.5, 1.0, false),
        },
        Claim {
            id: "ifd-wander.free",
            item: "ifd-wander",
            source: Source::Comment,
            citation: SPEC,
            text: "Wandering when nothing scores makes the Flumps free: under 5 % are off both patches at tick 1000",
            check: |seeds| range(&off_share(&preset("ifd-wander"), seeds), 0.0, 0.05, false),
        },
        Claim {
            id: "ifd-wander.toward-matching",
            item: "ifd-idle",
            source: Source::Comment,
            citation: SPEC,
            text: "Wandering moves s toward 1: s under wander exceeds s under stay (nobody starving)",
            check: |seeds| {
                let fed = |c: &mut Config| c.goods[0].endowment = URange::new(100_000, 100_000);
                let stay = s_per_seed("ifd-even", seeds, fed);
                let wander = s_per_seed("ifd-even", seeds, move |c| {
                    fed(c);
                    c.decision.rule = DecisionRule::Utility;
                    c.decision.idle = Idle::Wander;
                });
                greater(&wander, &stay, "wander", "stay")
            },
        },
        Claim {
            id: "ifd-matching.catchment",
            item: "ifd-matching",
            source: Source::Comment,
            citation: SPEC,
            text: "Catchment, not choice: at vision 1–6, s is within 0.1 of the catchment prediction",
            check: |seeds| {
                let pred = catchment_s(URange::new(1, 6));
                range(&s_per_seed("ifd-even", seeds, |_| {}), pred - 0.1, pred + 0.1, false)
                    .with(&format!("catchment prediction s = {pred:.3}"))
            },
        },
        Claim {
            id: "ifd-crowding.sutherland",
            item: "ifd-crowding",
            source: Source::Book,
            citation: "Sutherland 1983 (via Doncaster 1999)",
            text: "With interference m = 1, the distribution matches the inputs (s within 0.9–1.1); here the interference is local",
            check: |seeds| {
                let m = |m: f64| move |c: &mut Config| {
                    c.decision.rule = DecisionRule::Utility;
                    c.decision.crowding = m;
                };
                let (s0, s1) = (s_per_seed("ifd-even", seeds, m(0.0)), s_per_seed("ifd-even", seeds, m(1.0)));
                let direction = if crate::stats::median(&s1) < crate::stats::median(&s0) { "s falls" } else { "s does not fall" };
                range(&s1, 0.9, 1.1, false).with(&format!("{direction} from m 0 (median {:.3}) to m 1 (median {:.3})", crate::stats::median(&s0), crate::stats::median(&s1)))
            },
        },
        Claim {
            id: "ifd-travel.baum-kraft",
            item: "ifd-travel",
            source: Source::Book,
            citation: "Baum & Kraft 1998",
            text: "'When travel was required to switch patches, undermatching decreased slightly': s with travel k = 0.5 exceeds s at k = 0 (here travel is a preference for nearby sugar, not a cost of switching)",
            check: |seeds| {
                let k = |k: f64| move |c: &mut Config| {
                    c.decision.rule = DecisionRule::Utility;
                    c.decision.travel = k;
                };
                greater(&s_per_seed("ifd-even", seeds, k(0.5)), &s_per_seed("ifd-even", seeds, k(0.0)), "travel 0.5", "travel 0")
            },
        },
    ]
}
```

Check the names this relies on and adjust to the real API:
- `Outcome::with` (exists), `crate::stats::median` (exists) and `Torus::new(w, h)`.
- Whether `sight` is `pub` on `Torus` and `Pos` has public `x`/`y` fields.
- Whether `World::population()` returns `u32`; wrap with `f64::from` or `as f64` as needed.
- Whether `Map` and `URange` are re-exported from `sugarscape_core::config`.

If a claim's closure captures by `move`, keep the helper closures `Copy`.

- [ ] **Step 2: Build and run the module**

Run: `cd survey && cargo run --release -- --only ifd`
Expected: every claim yields a verdict (Holds, Weak or Fails), none Error. Record `survey/out/results-ifd.json`'s measured values. The spec predicts that Parker and Sutherland fail, and that undermatching, survival and the stuck share hold. Whatever comes out is the finding; don't tune the claims to pass.

- [ ] **Step 3: Put the measured values in the presets.** In each `ifd-*` description, replace the probe's numbers with the survey's ("Measured (20 seeds): …"). Name the claims that hold, the ones that fail and why, following `dock-mobility-15`'s description. Adjust any title the measurements contradict. For example, if s under wander doesn't rise, `ifd-wander`'s title must not imply it does.

- [ ] **Step 4: Fetch the open sources**
  - Earn & Johnstone 1997, PMC1688719 → `papers/ideal-free/earn-johnstone-1997-prsb-systematic-error-in-tests-of-ideal-free-theory.pdf`
  - Collins, Houston & Lang 2002 (people.maths.bris.ac.uk/~maejc/reports/ifd.pdf) → `papers/ideal-free/collins-houston-lang-2002-eer-ideal-free-distribution-perceptual-limit.pdf`
  - Lewis 2017 (gameaipro.com, *Game AI Pro 3* ch. 13) → `papers/utility/lewis-2017-game-ai-pro-3-choosing-effective-utility-based-considerations.pdf`
  - Baum & Kraft 1998 (PMC1284661) is a captcha-blocked scan. Try once, and if it's blocked, note "*not in `papers/`*" in the spec's sources.

  Don't send any email address or personal identifier to any service while fetching.

- [ ] **Step 5: Run the core tests again** (the descriptions changed, the configs didn't)

Run: `cargo test -p sugarscape-core`
Expected: pass. Golden entries are unchanged, since descriptions aren't fingerprinted.

- [ ] **Step 6: Commit**

```bash
git add survey/src/claims/minds1.rs survey/src/claims/mod.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs papers/ideal-free papers/utility docs/superpowers/specs/2026-09-27-minds-1-utility-design.md
git commit -m "Survey Minds 1: the ideal free distribution under rule M and the utility mind

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```

---

### Task 10: Docs

**Files:**
- Modify: `README.md` (a section after "El Farol and the Minority Game", ~`:1260-1320`, before `## Experiments`)
- Modify: `docs/studies/2026-09-27-minds.md` (Status; "What's already known")
- Modify: `docs/roadmap.md` (a Minds line under "Experiments and science")
- Modify: `docs/superpowers/specs/2026-09-27-minds-1-utility-design.md` (an "Amendments" section, if implementation changed anything)

- [ ] **Step 1: Write the README section.** Use the heading `### Minds 1: the utility mind and the ideal free distribution`. Cover:
  - the seam (`decision.rule`) and the reduction to rule M;
  - the three switches and their formulas;
  - the eight presets and four sweeps;
  - the findings as the survey measured them: s per vision, the stuck share, wander, catchment, Sutherland and Baum & Kraft, each with its verdict;
  - Earn and Johnstone's statistic.

  Label it as our experiment, not a reproduction of the book. Match the neighboring sections' tone and density.

- [ ] **Step 2: Update the program document.** Set Status to "Minds 1 done (branch `minds`); Minds 2 (A\* and walking) next". Replace the probe numbers in "What's already known" with the survey's.

- [ ] **Step 3: Update the roadmap.** Add under "Experiments and science": `- **Minds 1: the utility mind and the ideal free distribution** (our experiment; docs/studies/2026-09-27-minds.md): done.`

- [ ] **Step 4: Add amendments to the spec.** Following El Farol's "Amendments (implementation planning)" section, record any change made while implementing and the measured values.

- [ ] **Step 5: Run the whole suite once more**

Run: `cargo test && (cd web && npm run build && npm test) && wasm-pack test --node crates/sugarscape-wasm && cargo clippy --all-targets -- -D warnings`
Expected: all pass, with no clippy warnings. CI runs the latest stable clippy, so prefer `as_chunks`-style idioms if a newer lint fires.

- [ ] **Step 6: Commit**

```bash
git add README.md docs/studies/2026-09-27-minds.md docs/roadmap.md docs/superpowers/specs/2026-09-27-minds-1-utility-design.md
git commit -m "Document Minds 1 and what the ideal free distribution shows under rule M; mark it done

Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ"
```
