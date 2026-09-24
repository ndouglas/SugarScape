# Model Extensions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Four model extensions and two views for the SugarScape playground — user-defined tag groups (tribes, with the book's three-tribe preset), a pluggable bargaining rule (the book's geometric mean or a random price in [MRS_A, MRS_B], with a measured built-in sweep), seeded fractal-noise maps and image import for landscapes, an agent trail, and a layered credit-hierarchy tab — without changing any earlier run.

**Architecture:** Every model rule lives in `sugarscape-core`: `config.rs` gains `CultureRule { enabled, groups }` (with `Group`, `default_groups`, `three_tribes`, `group_of`), `TradeRule { enabled, price }` (`PriceRule`) and `Map::Noise`; `Agent::group` feeds combat, statistics (`group_share_K`), the Tribe color mode and the inspector; `rules/trade.rs` draws the random price from `World.rng` only under `random`; `landscape.rs` generates noise from an integer hash (no RNG); `World` gains `set_capacities` and a trail that is never hashed or exported; `network::credit_graph` lists outstanding loans. `sugarscape-wasm` exposes `set_landscape`, `follow`/`unfollow`/`trail`/`followed` and `credit_graph`. The web app gets pure, Vitest-tested modules (`groups.ts`, `image.ts`, `ui/trail.ts`, `credit.ts`) and thin UI on top: a groups table in the Culture section, a Price rule select, a Noise map kind, image import in the paint tool, a Follow button with a trail overlay and toolbar chip, and a Credit tab with an SVG layered graph.

**Tech Stack:** Rust (`sugarscape-core`, `sugarscape-wasm`, `sugarscape-cli`; rand 0.8, rand_pcg, serde, serde_json, proptest), wasm-bindgen, Vite + TypeScript + uPlot + Vitest. No new dependencies.

**Spec:** docs/superpowers/specs/2026-09-23-model-extensions-design.md

## Global Constraints

- **Earlier runs are unchanged.** `crates/sugarscape-core/tests/legacy.rs` and `tests/fixtures/*` are never edited. `tests/golden.rs` is edited exactly once, in Task 3, and only by **appending** one `GOLDEN` entry (for the new preset `iii-6-three-tribes`); no existing entry changes. Both files pass at the end of every task. With the default groups, `geometric_mean` pricing and no noise map, no rule behaves differently and no new RNG draw happens anywhere; `random` pricing is the only new consumer of `World.rng`.
- **Default groups reproduce the old tribe rule bit for bit** (Blue when zeros outnumber ones, else Red), checked exhaustively over all 2048 11-bit tag strings and over every zero count for tag lengths 1–64 (Task 1). **`geometric_mean` is today's code path** (`(ma * mb).sqrt()`, no draw; Task 5 checks the RNG state is untouched). **Trails** are not simulation state: never in `fingerprint`, configs, exports or share links (Task 12 checks the fingerprint).
- **One implementation.** Model rules are in `sugarscape-core`; the web calls WASM. The only TypeScript that mirrors Rust is `groups.ts`' `defaultGroups`/`threeTribes` (two one-line formulas the UI needs without a WASM round trip, as `goods.ts` already mirrors `Config::retarget_schedule`); Vitest pins them to the values Task 1 tests in Rust (Decision 7). Image → capacity conversion, trail segmenting and credit levels are presentation logic and live in TypeScript as the spec says.
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3` — the commit commands below pass it as a second `-m`. Stage **only** the files named in the task (`git add <paths>`, never `-A`/`.`). No dependency changes, so `Cargo.lock` is not staged.
- Rust tasks run `cargo fmt --all` **before** `cargo fmt --all --check`, then `cargo clippy --all-targets -- -D warnings` and `cargo test` (the whole workspace, including `golden.rs`, `legacy.rs` and the CLI tests). Tasks touching `sugarscape-wasm` also run `wasm-pack test --node crates/sugarscape-wasm`. Web tasks run `(cd web && npm run build && npm test)` (the build runs `wasm-pack` and `tsc --noEmit` first). If clippy flags a loop as `needless_range_loop`, rewrite it with `iter().enumerate()` without changing what is computed.
- **TypeScript:** `strict`, `noUnusedLocals`, `noUnusedParameters`. Never pass a possibly-null child to `replaceChildren`/`append` — build lists with `.filter(...)` or use `h()`, which skips `null`/`false`. New key orders of config objects built in TypeScript must match the core's serde order (`{ name, color, zeros: { min, max } }`, `{ kind, seed, scale, octaves, height }`) so preset matching (`JSON.stringify` comparison) keeps working.
- **Nobody browser-checks until the controller's puppeteer pass after Task 17** (the spec's "Browser" list). Web tasks are verified by `tsc`, `vite build` and Vitest.

## Why this task order

The suggested order is kept, with each feature split into core → WASM → browser so every layer is tested before anything consumes it:

- **Groups** come first (Tasks 1–4) because `CultureRule` changes the shape of `Config`, which every later config test and the legacy loader touch. Task 1 is config only (types, defaults, validation, reset-only), so the exhaustive old-rule test lands before any rule reads groups; Task 2 switches combat, statistics, rendering and inspection over; Task 3 isolates the one golden-file edit (the new preset) in its own small commit, where the expected fingerprint can be predicted (it must equal `iii-6-culture`'s, since no rule reads groups while combat is off); Task 4 is the browser.
- **Bargaining** (Tasks 5–7): core rule, then the one-select UI, then the built-in sweep, whose settings and the book test's tolerance are *measured* in Task 7 so Task 16 only encodes them.
- **Noise maps** (Tasks 8–9) before **image import** (Tasks 10–11): both feed landscapes; `set_capacities` (core + WASM) precedes the paint-tool UI that calls it.
- **Trails** (Tasks 12–13) and **the credit graph** (Tasks 14–15): core + WASM first, then UI. `Engine.follow` (select-by-id) is renamed `selectAgent` in Task 13 so the Credit tab (Task 15) and the trail API do not collide.
- **Book-style tests** (Task 16) read the preset and the measured sweep; **docs and full verification** (Task 17) close.

## Decisions (where the spec leaves room)

These are binding for this plan; each is repeated in the task that implements it.

1. **Groups.** `pub struct Group { name: String, color: String, zeros: URange }` in `config.rs`; `pub fn default_groups(tag_length) -> Vec<Group>` = `[Blue (⌊L/2⌋+1)..=L, Red 0..=⌊L/2⌋]` (⌊L/2⌋+1 = ⌈(L+1)/2⌉ for every L); `pub fn three_tribes(tag_length) -> Vec<Group>` = thirds cut at ⌊k(L+1)/3⌋ (Blue, Green, Red; exactly the book's 0–3/4–7/8–11 at L = 11; requires L ≥ 2); `pub fn group_of(groups, zeros) -> usize` = the first group containing `zeros`, 0 if none (only reachable on an unvalidated config). Colors: Blue `#3d7eff`, Red `#ff4d4d` (the renderer's `BLUE`/`RED`, so default renders are unchanged), Green `#3dd66b` (the renderer's lender green). `MAX_GROUPS = 8`.
2. **"A config without `groups`"** is detected by key presence in `Config::from_value`: a new-shape object whose `culture` is missing or has no `groups` key gets `default_groups(tag_length)`; pre-N-goods (legacy) configs always get it. An explicit `"groups": []` is kept and fails validation. `CultureRule.groups` is `#[serde(default)]` only so that deserialization succeeds before this fill-in.
3. **Validation fields.** `culture.groups` ("must list 1 to 8 groups", "no group holds tags with Z zeros", "tags with Z zeros fall in more than one group"); `culture.groups.K.name` (1–16 chars, unique), `culture.groups.K.color` (`#rrggbb`), `culture.groups.K.zeros` ("min must be ≤ max", "must lie within 0–L (the tag length)"). Groups are validated whether or not culture is on (combat and statistics read them).
4. **Reset-only.** Scheduled paths `culture`, `culture.groups`, `culture.groups.K`, `culture.groups.K.zeros[.min|.max]` are rejected ("changes only on reset"); `culture.groups.K.name`/`.color` and `trade.price` may be scheduled. `structural_changes` reports field `culture.groups` when the number of groups or any range differs.
5. **`Tribe` stays** (Blue/Red by majority) for what is inherently two-tribe: the `tribes` corner placement, replacement's same-tribe newcomer (`Death.tribe`, `forced_to`), the Place tool's Tribe override, `AgentView.tribe` and the agents CSV `tribe` column. Groups drive combat, the statistics, the Tribe color mode and the inspector (new `AgentView.group`). With the default groups the two coincide (group 0 ⟺ Blue).
6. **Statistics.** `Snapshot.groups: Vec<f64>` (share of living agents in each group; 0 with no agents); series `group_share_K` appended after `mean_pollution_K`; `blue_fraction` is computed by the same expression as before with "group 0" in place of "Blue".
7. **Browser groups.** `web/src/groups.ts` mirrors `default_groups`/`three_tribes`. Changing the Setup tag length rebuilds the groups **only if they were the default for the old length** (custom groups are kept and validation explains any mismatch; the book buttons fix it). "Three tribes (book)" is disabled below 2 bits. Editing a range end moves the adjoining group's end with it (`moveBoundary`), so single edits keep the ranges tiling; "Add group" splits the widest group (upper half to the new one); remove merges the range into the group below it (else above). The "Blue share" chart becomes "Group shares" (one line per group, in its color).
8. **Price rule.** `pub enum PriceRule { GeometricMean (default), Random }` (`"geometric_mean"`/`"random"`), in `pub struct TradeRule { enabled, #[serde(default)] price }`. Under `Random`, each attempted exchange draws `p = rng.gen_range(min(MRS_A, MRS_B)..=max(MRS_A, MRS_B))` after the pair ranking and before the checks (an attempt that fails the checks has still drawn). Legacy configs get `geometric_mean`.
9. **`bargaining-rules` settings and the tolerance are measured** in Task 7 with the experiments plan's settling rule (T ∈ {300, 500, 1000}, window = last 100 ticks), the same 60 s one-thread seed budget, and this tolerance rule: τ is the smallest of {0.10, 0.15, 0.20, 0.25} such that at every mean vision |mean_random − mean_geometric| + 2·se < τ·mean_geometric (se = √(sd_g²/n_g + sd_r²/n_r)); if none qualifies, stop (BLOCKED). The book test asserts |d| < τ·mean_geometric.
10. **Noise octaves (spec conflict, see "Spec gaps" 1).** Octave o's lattice has period `max(1, round(N · 2^o / scale))` cells along a side of N cells — `scale` is octave 0's feature size and each octave is twice as fine at half the amplitude (standard fractal noise). The spec's `round(W / (scale · 2^o))` would make each octave twice as *coarse* at half the amplitude, which is not fractal noise. Everything else is as specified: corner values `mix(mix(mix(seed << 32 | o) ^ i) ^ j)` (SplitMix64 finalizer), value in [0, 1) from the top 53 bits, smoothstep, lattice indices modulo the period, amplitudes 1, ½, ¼…, normalized by the total, capacity `round(height · v)` clamped to `0..=height`. Cell (x, y) samples lattice coordinate (x · pₓ/W, y · p_y/H), so x = W is exactly x = 0. No transcendental functions: the map is identical on every platform.
11. **`set_capacities`** errors (as `String`, like `paint_capacity`): "there is no good G", "expected N capacities, got M", "capacities must be between 0 and 10 (got V)". WASM `set_landscape(good: u32, capacities: &[u8])` (a `Uint8Array` in JS) reports them as field `edit`.
12. **Image import** applies to the paint tool's selected good; max capacity is an integer 0–10 (default 4); errors and the imported file name show next to the controls. Taken literally, "alpha < 128 counts as 0" makes transparent pixels luminance 0, so with Invert they become the maximum.
13. **Trails.** `World` keeps `followed: Option<AgentId>` and `trail: Vec<Pos>` (at most `TRAIL_LEN = 500`, oldest dropped first); `follow(None)` also clears the trail; following an id that is not alive records nothing; the position is recorded at the very end of `step` (after the snapshot). In the browser the existing `Engine.follow(id)` (select an agent by id) is renamed `selectAgent`; the trail API is `followAgent`, `unfollow`, `followed`, `trail`, with a new engine event `'follow'`.
14. **Trail drawing.** Segment i (between points i and i + 1 of n) has alpha (i + 1)/(n − 1), color `--text`, width 2; a step whose |dx| > W/2 or |dy| > H/2 (strictly) is a wrap and is not drawn (movement never exceeds vision ≤ half the grid). The toolbar chip reads "Following #id" plus "†" once the agent has died.
15. **Credit graph.** Core `network::credit_graph(&World) -> CreditGraph { agents: [{ id, role }], loans: [{ lender, borrower, good, due }] }`: agents in id order, loans in loan-id order, roles `"lender" | "borrower" | "both"`.
16. **Levels** are computed on the graph's strongly connected components (Tarjan): a component's level is 0 when no loan enters it from another component, else 1 + the highest level among components lending into it; every member gets its component's level. On acyclic graphs this is exactly the spec's formula; in a cycle it cuts every edge that closes the cycle regardless of visit order, which is what "agents in a cycle with no pure lender above get level 0" requires (a plain DFS would give order-dependent levels).
17. **Large graphs.** With more than 400 loans, the 400 with the largest `due` are drawn (ties: earlier loan first); rows and levels are those of the drawn loans; the header gives the full graph's agent and loan counts, the drawn levels, and "the 400 largest loans drawn (K omitted)".
18. **Credit tab.** `Tabs.setHidden(label, hidden)` hides the tab button while credit is off (a hidden selected tab falls back to the first visible one). The graph redraws at most every 500 ms while visible and on reset; clicking a node selects that agent and shows the Inspect tab. Node colors are CSS tokens `--lender` `#3dd66b`, `--borrower` `#ff4d4d`, `--both` `#ffe04d` (the renderer's credit colors).

## File Structure

```
crates/sugarscape-core/
  src/config.rs        MOD  Group, CultureRule, default_groups, three_tribes, group_of, validation,
                            reset-only, structural (1); PriceRule, TradeRule (5); Map::Noise (8)
  src/legacy.rs        MOD  groups and price for legacy configs (1, 5)
  src/agent.rs         MOD  Agent::group (2)
  src/rules/combat.rs  MOD  own group instead of own tribe (2)
  src/rules/culture.rs MOD  doc comment (2)
  src/rules/trade.rs   MOD  random price (5)
  src/stats.rs         MOD  Snapshot.groups, group_share_K (2)
  src/export.rs        MOD  header test (2)
  src/render.rs        MOD  Tribe mode colors by group (2)
  src/edit.rs          MOD  AgentView.group (2); set_capacities (10)
  src/presets.rs       MOD  iii-6-three-tribes (3)
  src/landscape.rs     MOD  noise generation (8)
  src/world.rs         MOD  trails (12)
  src/network.rs       MOD  credit_graph (14)
  src/sweep.rs         MOD  bargaining-rules built-in (7)
  tests/golden.rs      MOD  one appended entry (3)
  tests/book.rs        MOD  temporary measurement (7, removed before commit); two book tests (16)
sweeps/bargaining-rules.json               NEW (7)
crates/sugarscape-cli/tests/cli.rs         MOD  lists bargaining-rules (7)
crates/sugarscape-wasm/src/lib.rs          MOD  set_landscape (10); follow/unfollow/trail/followed (12); credit_graph (14)
crates/sugarscape-wasm/tests/web.rs        MOD  (7, 10, 12, 14)
web/src/types.ts        MOD  TagGroup, groups, AgentView.group, Snapshot.groups (4); PriceRule (6); noise map (9)
web/src/groups.ts, groups.test.ts          NEW (4)
web/src/schema.ts       MOD  tag_length adjust, Culture custom editor (4); Price rule select (6)
web/src/schema.test.ts  NEW (4), MOD (6)
web/src/ui/groups-editor.ts                NEW (4)
web/src/ui/rules-panel.ts                  MOD  groups editor, control adjust (4)
web/src/ui/charts-panel.ts                 MOD  Group shares chart (4)
web/src/ui/inspect-panel.ts                MOD  group name (4); Follow button, selectAgent (13)
web/src/goods.ts, goods.test.ts            MOD  noise defaultMap (9)
web/src/ui/goods-editor.ts                 MOD  Noise kind (9)
web/src/image.ts, image.test.ts            NEW (11)
web/src/ui/image-import.ts                 NEW (11)
web/src/engine.ts       MOD  importLandscape (11); follow API, selectAgent (13); creditGraph (15)
web/src/ui/tools.ts                        MOD  image import (11)
web/src/ui/trail.ts, trail.test.ts         NEW (13)
web/src/ui/toolbar.ts, ui/grid-view.ts     MOD  (13)
web/src/credit.ts, credit.test.ts          NEW (15)
web/src/ui/credit-panel.ts                 NEW (15)
web/src/ui/tabs.ts                         MOD  setHidden (15)
web/src/main.ts                            MOD  (13, 15)
web/src/style.css                          MOD  (4, 13, 15)
README.md, docs/roadmap.md                 MOD (17)
```

---

### Task 1: Tag groups in the config

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs`
- Modify: `crates/sugarscape-core/src/legacy.rs`

**Interfaces:**
- Consumes: `URange`, `parse_color`, `Errors`, `Config::{from_value, validate_fields, structural_changes}`, `reset_only`, `crate::agent::{Tags, Tribe}` (tests only).
- Produces (module `sugarscape_core::config`):
  - `pub const MAX_GROUPS: usize = 8; pub const BLUE_COLOR: &str = "#3d7eff"; pub const RED_COLOR: &str = "#ff4d4d"; pub const GREEN_COLOR: &str = "#3dd66b";`
  - `pub struct Group { pub name: String, pub color: String, pub zeros: URange }` (`Clone, Debug, PartialEq, Eq, Serialize, Deserialize`) with `pub fn new(name: &str, color: &str, min: u32, max: u32) -> Group`
  - `pub fn default_groups(tag_length: u32) -> Vec<Group>`, `pub fn three_tribes(tag_length: u32) -> Vec<Group>`, `pub fn group_of(groups: &[Group], zeros: u32) -> usize`
  - `pub struct CultureRule { pub enabled: bool, #[serde(default)] pub groups: Vec<Group> }` (`Clone, Debug, PartialEq, Eq, Serialize, Deserialize`); `Config.culture: CultureRule` (was `Toggle`)
  - private `Config::check_groups(&self, e: &mut Errors)`

- [ ] **Step 1: Confirm the baseline**

Run: `cargo test -p sugarscape-core --test golden --test legacy`
Expected: golden `2 passed; 0 failed; 1 ignored`, legacy `2 passed`. If anything fails, stop: the branch is not at the expected state.

- [ ] **Step 2: Write the failing tests**

Append inside `mod tests` in `crates/sugarscape-core/src/config.rs`:
```rust
    /// `n` one-bits (the low `n` tag positions).
    fn ones(n: u32) -> u64 {
        if n == 64 {
            u64::MAX
        } else {
            (1u64 << n) - 1
        }
    }

    #[test]
    fn default_groups_reproduce_the_two_tribe_rule() {
        use crate::agent::{Tags, Tribe};
        let tribe_index = |t: Tribe| match t {
            Tribe::Blue => 0,
            Tribe::Red => 1,
        };
        let groups = default_groups(11);
        assert_eq!(
            groups,
            vec![
                Group::new("Blue", BLUE_COLOR, 6, 11),
                Group::new("Red", RED_COLOR, 0, 5)
            ]
        );
        for bits in 0..1u64 << 11 {
            let tags = Tags::new(bits, 11);
            assert_eq!(
                group_of(&groups, tags.zeros()),
                tribe_index(tags.tribe()),
                "{}",
                tags.to_bit_string()
            );
        }
        for len in 1..=64 {
            let groups = default_groups(len);
            for zeros in 0..=len {
                let tags = Tags::new(ones(len - zeros), len);
                assert_eq!(tags.zeros(), zeros);
                assert_eq!(
                    group_of(&groups, zeros),
                    tribe_index(tags.tribe()),
                    "L = {len}, {zeros} zeros"
                );
            }
        }
    }

    #[test]
    fn three_tribes_are_the_books_thirds() {
        let spans = |g: Vec<Group>| -> Vec<(String, u32, u32)> {
            g.into_iter()
                .map(|g| (g.name, g.zeros.min, g.zeros.max))
                .collect()
        };
        let s = |name: &str, min, max| (name.to_string(), min, max);
        assert_eq!(
            spans(three_tribes(11)),
            vec![s("Blue", 0, 3), s("Green", 4, 7), s("Red", 8, 11)]
        );
        assert_eq!(
            spans(three_tribes(2)),
            vec![s("Blue", 0, 0), s("Green", 1, 1), s("Red", 2, 2)]
        );
        let groups = three_tribes(11);
        assert_eq!(
            [0, 3, 4, 7, 8, 11].map(|z| group_of(&groups, z)),
            [0, 0, 1, 1, 2, 2]
        );
        let mut c = Config::default();
        c.culture.groups = groups;
        c.validate().unwrap();
    }

    #[test]
    fn groups_are_validated() {
        let with = |f: &dyn Fn(&mut Vec<Group>)| {
            let mut c = Config::default();
            f(&mut c.culture.groups);
            c.validate().err().unwrap_or_default()
        };
        let has = |errs: &[FieldError], field: &str, text: &str| {
            errs.iter()
                .any(|e| e.field == field && e.message.contains(text))
        };
        assert!(with(&|_| {}).is_empty());
        let gap = with(&|g| g[0].zeros.min = 7);
        assert!(has(&gap, "culture.groups", "with 6 zeros"), "{gap:?}");
        let overlap = with(&|g| g[1].zeros.max = 6);
        assert!(
            has(&overlap, "culture.groups", "more than one group"),
            "{overlap:?}"
        );
        let outside = with(&|g| g[0].zeros.max = 12);
        assert!(
            has(&outside, "culture.groups.0.zeros", "0–11"),
            "{outside:?}"
        );
        let inverted = with(&|g| g[0].zeros = URange::new(11, 6));
        assert!(has(&inverted, "culture.groups.0.zeros", "min must be ≤ max"));
        assert!(has(&with(&|g| g.clear()), "culture.groups", "1 to 8"));
        let nine = with(&|g| {
            *g = (0..9)
                .map(|k| Group::new(&format!("g{k}"), BLUE_COLOR, k, if k == 8 { 11 } else { k }))
                .collect();
        });
        assert!(has(&nine, "culture.groups", "1 to 8"), "{nine:?}");
        assert_eq!(nine.len(), 1, "nine groups that tile 0–11: only the count is wrong");
        assert!(has(
            &with(&|g| g[1].name = "Blue".into()),
            "culture.groups.1.name",
            "another group"
        ));
        assert!(has(
            &with(&|g| g[0].name = String::new()),
            "culture.groups.0.name",
            "1–16"
        ));
        assert!(has(
            &with(&|g| g[0].color = "blue".into()),
            "culture.groups.0.color",
            "#rrggbb"
        ));
        // A shorter tag length strands the default groups for 11 bits.
        let short = Config {
            tag_length: 5,
            ..Config::default()
        };
        assert!(fields(short.validate()).contains(&"culture.groups.0.zeros".to_string()));
        let fixed = Config {
            tag_length: 5,
            culture: CultureRule {
                enabled: false,
                groups: default_groups(5),
            },
            ..Config::default()
        };
        fixed.validate().unwrap();
    }

    #[test]
    fn configs_without_groups_get_the_two_tribes_for_their_tag_length() {
        assert_eq!(Config::default().culture.groups, default_groups(11));
        // Pre-N-goods shape (no `goods` key): always the default.
        let legacy =
            Config::from_json(r#"{"tag_length": 5, "culture": {"enabled": true}}"#).unwrap();
        assert_eq!(legacy.culture.groups, default_groups(5));
        assert!(legacy.culture.enabled);
        // New shape without `culture.groups`, and without `culture`.
        let seven = Config {
            tag_length: 7,
            culture: CultureRule {
                enabled: true,
                groups: default_groups(7),
            },
            ..Config::default()
        };
        let mut value = serde_json::to_value(&seven).unwrap();
        value["culture"].as_object_mut().unwrap().remove("groups");
        assert_eq!(Config::from_json(&value.to_string()).unwrap(), seven);
        value.as_object_mut().unwrap().remove("culture");
        let c = Config::from_json(&value.to_string()).unwrap();
        assert_eq!(
            (c.culture.enabled, c.culture.groups),
            (false, default_groups(7))
        );
        // Given groups are kept, in the core's key order.
        let mut three = Config::default();
        three.culture.groups = three_tribes(11);
        let json = serde_json::to_string(&three).unwrap();
        assert_eq!(Config::from_json(&json).unwrap(), three);
        assert!(json.contains(
            r##""groups":[{"name":"Blue","color":"#3d7eff","zeros":{"min":0,"max":3}}"##
        ));
        // An explicit empty list is not "missing".
        let mut empty = serde_json::to_value(Config::default()).unwrap();
        empty["culture"]["groups"] = serde_json::json!([]);
        assert_eq!(
            Config::from_json(&empty.to_string()).unwrap_err()[0].field,
            "culture.groups"
        );
    }

    #[test]
    fn group_names_and_colors_are_live_but_ranges_change_only_on_reset() {
        let live = Config {
            schedule: vec![
                change(5, "culture.groups.0.name", serde_json::json!("Azure")),
                change(6, "culture.groups.1.color", serde_json::json!("#aa0000")),
                change(7, "culture.enabled", serde_json::json!(true)),
            ],
            ..Default::default()
        };
        live.validate().unwrap();
        for (path, value) in [
            (
                "culture",
                serde_json::to_value(&Config::default().culture).unwrap(),
            ),
            ("culture.groups", serde_json::json!([])),
            (
                "culture.groups.0",
                serde_json::to_value(&default_groups(11)[0]).unwrap(),
            ),
            (
                "culture.groups.0.zeros",
                serde_json::json!({"min": 6, "max": 11}),
            ),
            ("culture.groups.1.zeros.max", serde_json::json!(5)),
        ] {
            let c = Config {
                schedule: vec![change(5, path, value)],
                ..Default::default()
            };
            let errs = c.validate().unwrap_err();
            assert!(errs[0].message.contains("only on reset"), "{path}: {errs:?}");
        }
        let a = Config::default();
        let mut b = a.clone();
        b.culture.enabled = true;
        b.culture.groups[0].name = "Azure".into();
        b.culture.groups[1].color = "#aa0000".into();
        assert!(a.structural_changes(&b).is_empty());
        let mut moved = a.clone();
        moved.culture.groups[0].zeros.min = 7;
        moved.culture.groups[1].zeros.max = 6;
        assert_eq!(a.structural_changes(&moved)[0].field, "culture.groups");
        let mut one = a.clone();
        one.culture.groups = vec![Group::new("All", BLUE_COLOR, 0, 11)];
        assert_eq!(a.structural_changes(&one)[0].field, "culture.groups");
    }
```

Run: `cargo test -p sugarscape-core config::`
Expected: compile errors (`default_groups`, `Group`, `CultureRule` … not found).

- [ ] **Step 3: Add the group types and functions**

In `crates/sugarscape-core/src/config.rs`, insert directly after the `Toggle` struct:
```rust
/// Most tag groups a config may list.
pub const MAX_GROUPS: usize = 8;
/// The book's tribe colors (the renderer's `BLUE` and `RED`) and the third
/// group's green (Chapter III, note 20).
pub const BLUE_COLOR: &str = "#3d7eff";
pub const RED_COLOR: &str = "#ff4d4d";
pub const GREEN_COLOR: &str = "#3dd66b";

/// A tag group (tribe): the agents whose tag strings hold a number of zeros
/// in `zeros`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Display name (1–16 characters, unique).
    pub name: String,
    /// `#rrggbb`, used by the Tribe color mode and the group-share chart.
    pub color: String,
    pub zeros: URange,
}

impl Group {
    pub fn new(name: &str, color: &str, min: u32, max: u32) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
            zeros: URange::new(min, max),
        }
    }
}

/// Chapter III's two tribes on `tag_length`-bit tags: Blue when zeros
/// outnumber ones (⌈(L+1)/2⌉..=L zeros), otherwise Red. Group 0 is Blue.
pub fn default_groups(tag_length: u32) -> Vec<Group> {
    // ⌊L/2⌋ + 1 = ⌈(L+1)/2⌉: the fewest zeros that outnumber the ones.
    let blue_from = tag_length / 2 + 1;
    vec![
        Group::new("Blue", BLUE_COLOR, blue_from, tag_length),
        Group::new("Red", RED_COLOR, 0, blue_from - 1),
    ]
}

/// Chapter III note 20's three groups — Blue 0–3, Green 4–7, Red 8–11 zeros
/// on 11-bit tags — generalized to `tag_length` ≥ 2 by cutting 0..=L into
/// thirds at ⌊k(L+1)/3⌋.
pub fn three_tribes(tag_length: u32) -> Vec<Group> {
    assert!(tag_length >= 2, "three tribes need tags of at least 2 bits");
    let cut = |k: u32| k * (tag_length + 1) / 3;
    vec![
        Group::new("Blue", BLUE_COLOR, 0, cut(1) - 1),
        Group::new("Green", GREEN_COLOR, cut(1), cut(2) - 1),
        Group::new("Red", RED_COLOR, cut(2), tag_length),
    ]
}

/// The first group whose range holds `zeros`; 0 if none does (a validated
/// config's groups cover every count).
pub fn group_of(groups: &[Group], zeros: u32) -> usize {
    groups
        .iter()
        .position(|g| (g.zeros.min..=g.zeros.max).contains(&zeros))
        .unwrap_or(0)
}

/// K: cultural transmission, plus the tag groups that combat, the Tribe
/// color mode and the group-share statistics read (whether or not K is on).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CultureRule {
    pub enabled: bool,
    /// An agent belongs to the first group whose `zeros` holds its tags'
    /// number of zeros. When JSON omits it, `Config::from_value` fills in
    /// `default_groups(tag_length)` (Decision 2).
    #[serde(default)]
    pub groups: Vec<Group>,
}
```

In `struct Config`, change `pub culture: Toggle,` to:
```rust
    pub culture: CultureRule,
```
In `impl Default for Config`, change `culture: Toggle { enabled: false },` to:
```rust
            culture: CultureRule {
                enabled: false,
                groups: default_groups(11),
            },
```

- [ ] **Step 4: Fill in missing groups when reading JSON**

In `Config::from_value`, replace
```rust
        let defaulted_pollution = !object.contains_key("pollution");
        let mut config: Config =
            serde_json::from_value(value).map_err(|e| FieldError::new("config", e.to_string()))?;
        if defaulted_pollution {
            config.pollution.pollutants = vec![Pollutant::book(config.goods.len())];
        }
        Ok(config)
```
with
```rust
        let defaulted_pollution = !object.contains_key("pollution");
        let defaulted_groups = object
            .get("culture")
            .is_none_or(|c| c.get("groups").is_none());
        let mut config: Config =
            serde_json::from_value(value).map_err(|e| FieldError::new("config", e.to_string()))?;
        if defaulted_pollution {
            config.pollution.pollutants = vec![Pollutant::book(config.goods.len())];
        }
        if defaulted_groups {
            config.culture.groups = default_groups(config.tag_length);
        }
        Ok(config)
```
and extend its doc comment with: `A new-shape config without \`culture.groups\` gets the two book tribes for its \`tag_length\`.`

- [ ] **Step 5: Validate groups**

Add this method inside `impl Config` (next to `check_map`):
```rust
    /// `culture.groups` (Decision 3): 1–8 groups with unique names and
    /// `#rrggbb` colors whose zero ranges tile 0..=tag_length exactly once.
    fn check_groups(&self, e: &mut Errors) {
        let groups = &self.culture.groups;
        let l = self.tag_length;
        e.check(
            (1..=MAX_GROUPS).contains(&groups.len()),
            "culture.groups",
            format!("must list 1 to {MAX_GROUPS} groups"),
        );
        let mut names = BTreeSet::new();
        for (k, g) in groups.iter().enumerate() {
            let field = |f: &str| format!("culture.groups.{k}.{f}");
            e.name(&g.name, &field("name"));
            e.check(
                names.insert(g.name.as_str()),
                &field("name"),
                "another group has this name",
            );
            e.check(
                parse_color(&g.color).is_some(),
                &field("color"),
                "must be a #rrggbb color",
            );
            e.range(g.zeros, &field("zeros"));
            e.check(
                g.zeros.max <= l,
                &field("zeros"),
                format!("must lie within 0–{l} (the tag length)"),
            );
        }
        if !(1..=64).contains(&l) {
            return; // `tag_length` reports itself
        }
        let mut hits = vec![0usize; l as usize + 1];
        for g in groups {
            for z in g.zeros.min..=g.zeros.max.min(l) {
                hits[z as usize] += 1;
            }
        }
        let counts = |want: fn(usize) -> bool| {
            hits.iter()
                .enumerate()
                .filter(|&(_, &n)| want(n))
                .map(|(z, _)| z.to_string())
                .collect::<Vec<_>>()
        };
        let (gaps, overlaps) = (counts(|n| n == 0), counts(|n| n > 1));
        e.check(
            gaps.is_empty(),
            "culture.groups",
            format!("no group holds tags with {} zeros", gaps.join(", ")),
        );
        e.check(
            overlaps.is_empty(),
            "culture.groups",
            format!(
                "tags with {} zeros fall in more than one group",
                overlaps.join(", ")
            ),
        );
    }
```
In `validate_fields`, directly after the `tag_length` check (`"must be between 1 and 64"`), add:
```rust
        self.check_groups(&mut e);
```

- [ ] **Step 6: Make group ranges reset-only**

Replace `reset_only` with:
```rust
/// Whether a schedule may not set `path` (Decision 5): the goods list, whole
/// goods and their maps, the pollutant list, the groups list, whole groups
/// and their ranges, and `RESET_ONLY_PATHS`. A good's name, color and trait
/// ranges, a pollutant's name and coefficients, and a group's name and color
/// may be scheduled.
fn reset_only(path: &str) -> bool {
    let parts: Vec<&str> = path.split('.').collect();
    matches!(
        parts.as_slice(),
        ["goods"]
            | ["goods", _]
            | ["goods", _, "map", ..]
            | ["pollution"]
            | ["pollution", "pollutants"]
            | ["culture"]
            | ["culture", "groups"]
            | ["culture", "groups", _]
            | ["culture", "groups", _, "zeros", ..]
    ) || RESET_ONLY_PATHS.contains(&path)
}
```
In `structural_changes`, directly after the `tag_length` check, add:
```rust
        let ranges = |c: &Config| c.culture.groups.iter().map(|g| g.zeros).collect::<Vec<_>>();
        if ranges(self) != ranges(next) {
            out.push(FieldError::new("culture.groups", msg));
        }
```

- [ ] **Step 7: Legacy configs get the default groups**

In `crates/sugarscape-core/src/legacy.rs`, add `default_groups` and `CultureRule` to the `use crate::config::{…}` list (keep `Toggle`: the legacy struct still reads `culture` as a toggle). In `impl Default for LegacyConfig`, change `culture: c.culture,` to:
```rust
            culture: Toggle {
                enabled: c.culture.enabled,
            },
```
In `convert`, change `culture: old.culture,` to:
```rust
        culture: CultureRule {
            enabled: old.culture.enabled,
            groups: default_groups(old.tag_length),
        },
```

- [ ] **Step 8: Run the tests**

Run: `cargo test -p sugarscape-core config::`
Expected: all pass, including the five new tests.

- [ ] **Step 9: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
```
Expected: all pass (golden and legacy unchanged: the legacy fixtures load with `default_groups(11)`, equal to the presets').

- [ ] **Step 10: Commit**

```bash
git add crates/sugarscape-core/src/config.rs crates/sugarscape-core/src/legacy.rs
git commit -m "Add tag groups to the culture config" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 2: Groups in combat, statistics, rendering and inspection

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/agent.rs`, `rules/combat.rs`, `rules/culture.rs` (doc only), `stats.rs`, `export.rs` (test only), `render.rs`, `edit.rs`

**Interfaces:**
- Consumes: `config::{Group, group_of, three_tribes, default_groups}` (Task 1).
- Produces: `Agent::group(&self, groups: &[Group]) -> usize`; `Snapshot.groups: Vec<f64>`; series `group_share_K` (appended after `mean_pollution_K` by `series_names`, readable through `Snapshot::value`); `AgentView.group: usize`; combat's private `vulnerable(world, attacker, target, vision, group: usize, after)`.

- [ ] **Step 1: Write the failing tests**

In `crates/sugarscape-core/src/rules/combat.rs`, append inside `mod tests`:
```rust
    fn tagged(w: &mut World, x: u32, y: u32, sugar: f64, bits: u64) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.tags = Tags::new(bits, 11);
        a.holdings[0] = sugar;
        id
    }

    #[test]
    fn every_other_group_is_an_enemy() {
        let mut w = fighting_world();
        w.config.culture.groups = crate::config::three_tribes(11);
        let me = blue(&mut w, 5, 5, 10.0, 2); // 11 zeros: Red (8–11)
        let friend = tagged(&mut w, 5, 7, 3.0, 0b111); // 8 zeros: Red
        let green = tagged(&mut w, 7, 5, 3.0, 0b111_1111); // 4 zeros: Green
        act(&mut w, me);
        assert!(w.agent(friend).is_some(), "same group");
        assert!(w.agent(green).is_none(), "another group");
        assert_eq!(w.agent(me).unwrap().pos, Pos::new(7, 5));
    }
```
In `crates/sugarscape-core/src/stats.rs`, append inside `mod tests`:
```rust
    #[test]
    fn group_shares_follow_the_configured_groups() {
        use crate::agent::Tags;
        use crate::testkit::*;
        let mut w = blank_world(5, 5);
        w.config.culture.groups = crate::config::three_tribes(11);
        for (x, bits) in [(0, 0u64), (1, 0), (2, 0b111_1111), (3, u64::MAX)] {
            let id = spawn(&mut w, x, 0);
            w.agent_mut(id).unwrap().tags = Tags::new(bits, 11);
        }
        // Zeros 11, 11 (Red 8–11), 4 (Green 4–7), 0 (Blue 0–3).
        let s = Snapshot::of(&w);
        assert_eq!(s.groups, vec![0.25, 0.25, 0.5]);
        assert_eq!(s.blue_fraction, 0.25, "the share of group 0");
        assert_eq!(s.value("group_share_2"), Some(0.5));
        assert_eq!(s.value("group_share_3"), None);
        let names = series_names(&w.config);
        assert_eq!(
            &names[names.len() - 3..],
            ["group_share_0", "group_share_1", "group_share_2"]
        );
        w.config.culture.groups = crate::config::default_groups(11);
        let s = Snapshot::of(&w);
        assert_eq!(s.groups, vec![0.5, 0.5]);
        assert_eq!(s.blue_fraction, 0.5, "Blue: zeros outnumber ones");
        let empty = Snapshot::of(&blank_world(5, 5));
        assert_eq!(empty.groups, vec![0.0, 0.0]);
    }
```
In the same file, in `per_good_and_per_pollutant_series`, replace
```rust
        assert_eq!(names.len(), SERIES.len() + 3 * 3 + 2);
```
with
```rust
        assert_eq!(names.len(), SERIES.len() + 3 * 3 + 2 + 2);
```
and replace
```rust
        assert_eq!(names.last().unwrap(), "mean_pollution_1");
```
with
```rust
        assert_eq!(names[names.len() - 3], "mean_pollution_1");
        assert_eq!(names.last().unwrap(), "group_share_1");
```
In `crates/sugarscape-core/src/export.rs`, in `series_csv_has_a_header_and_a_row_per_tick`, change the expected header's ending `…,traded_0,mean_pollution_0"` to `…,traded_0,mean_pollution_0,group_share_0,group_share_1"`.

In `crates/sugarscape-core/src/render.rs`, append inside `mod tests`:
```rust
    #[test]
    fn tribe_mode_uses_each_groups_color() {
        let mut w = blank_world(10, 10);
        let red = spawn(&mut w, 4, 4);
        w.agent_mut(red).unwrap().tags = crate::agent::Tags::new(u64::MAX, 11);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(
            pixel(&buf, &w, 4, 4)[..3],
            RED,
            "the default groups keep the book's colors"
        );
        w.config.culture.groups = crate::config::three_tribes(11);
        w.config.culture.groups[0].color = "#102030".into();
        render(&w, ColorMode::Tribe, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(
            pixel(&buf, &w, 4, 4)[..3],
            [0x10, 0x20, 0x30],
            "no zeros: group 0"
        );
    }
```
In `crates/sugarscape-core/src/edit.rs`, append inside `mod tests`:
```rust
    #[test]
    fn inspection_gives_the_agents_group() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 1, 1); // all zeros: Blue (6–11 zeros)
        let group = |w: &World| w.inspect(1, 1).unwrap().agent.unwrap().group;
        assert_eq!(group(&w), 0);
        w.agent_mut(id).unwrap().tags = crate::agent::Tags::new(u64::MAX, 11);
        assert_eq!(group(&w), 1, "no zeros: Red");
        w.config.culture.groups = crate::config::three_tribes(11);
        assert_eq!(group(&w), 0, "no zeros: Blue (0–3)");
    }
```

Run: `cargo test -p sugarscape-core`
Expected: compile errors (`groups` field, `group` field not found).

- [ ] **Step 2: `Agent::group`**

In `crates/sugarscape-core/src/agent.rs`, change `use crate::config::{Config, MAX_GOODS};` to `use crate::config::{group_of, Config, Group, MAX_GOODS};`, change the doc comment of `enum Tribe` to
```rust
/// The book's two tribes (Chapter III): Blue when zeros outnumber ones on the
/// tag string. Used where the rule is inherently two-tribe (corner placement,
/// replacement, the Place tool); `culture.groups` generalizes it and, with
/// its defaults, agrees with it (group 0 is Blue).
```
and add inside `impl Agent`, after `tribe`:
```rust
    /// Index of the agent's tag group in `groups` (`config::group_of`).
    pub fn group(&self, groups: &[Group]) -> usize {
        group_of(groups, self.tags.zeros())
    }
```

- [ ] **Step 3: Combat reads groups**

In `crates/sugarscape-core/src/rules/combat.rs`, replace the module doc's first sentences
```rust
//! Sites held by the agent's own tribe, or by other-tribe agents at least as
//! wealthy as the agent, are discarded. A site's reward is its sugar plus,
//! if occupied, min(α, occupant's sugar). Sites vulnerable to retaliation are
//! discarded: after taking the site, would some other-tribe agent visible
```
with
```rust
//! Sites held by the agent's own group (`culture.groups`; with the defaults,
//! the book's tribe), or by other-group agents at least as wealthy as the
//! agent, are discarded. Every other group is an enemy. A site's reward is
//! its sugar plus, if occupied, min(α, occupant's sugar). Sites vulnerable to
//! retaliation are discarded: after taking the site, would some other-group
//! agent visible
```
Change `use crate::agent::{AgentId, Tribe};` to `use crate::agent::AgentId;`, and replace `act`'s first lines and loop plus `vulnerable` with:
```rust
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let me = world.agent(id).expect("live agent");
    let group = me.group(&world.config.culture.groups);
    let (pos, vision, wealth) = (me.pos, me.vision, me.holdings[0]);
    let cap = if world.config.combat.unlimited {
        f64::INFINITY
    } else {
        world.config.combat.reward
    };

    let mut candidates = vec![(pos, 0, world.site(pos).resource[0])];
    for (q, d) in world.torus.sight(pos, vision) {
        let site_sugar = world.site(q).resource[0];
        let reward = match world.agent_at(q) {
            Some(o) if o.group(&world.config.culture.groups) == group || o.holdings[0] >= wealth => {
                continue
            }
            Some(o) => site_sugar + cap.min(o.holdings[0]),
            None => site_sugar,
        };
        if vulnerable(world, id, q, vision, group, wealth + reward) {
            continue;
        }
        candidates.push((q, d, reward));
    }
```
(the rest of `act`, from `let target = choose(…)`, is unchanged) and
```rust
/// Whether some other-group agent visible from `target` would be wealthier
/// than the attacker after the attack.
fn vulnerable(
    world: &World,
    attacker: AgentId,
    target: Pos,
    vision: u32,
    group: usize,
    after: f64,
) -> bool {
    let groups = &world.config.culture.groups;
    world.torus.sight(target, vision).into_iter().any(|(q, _)| {
        world.agent_at(q).is_some_and(|o| {
            o.id != attacker && o.group(groups) != group && o.holdings[0] > after
        })
    })
}
```
In `crates/sugarscape-core/src/rules/culture.rs`, change the module doc's last line to `//! Group membership is derived from tags by \`culture.groups\` (\`Agent::group\`).`

- [ ] **Step 4: Statistics**

In `crates/sugarscape-core/src/stats.rs`: delete `use crate::agent::Tribe;`. Replace `series_names` with:
```rust
/// `SERIES`, then `mean_holding_I`, `mean_metabolism_I`, `traded_I` for
/// each good, then `mean_pollution_K` for each pollutant, then
/// `group_share_K` for each group.
pub fn series_names(config: &Config) -> Vec<String> {
    let mut names: Vec<String> = SERIES.iter().map(|s| s.to_string()).collect();
    for i in 0..config.goods.len() {
        names.push(format!("mean_holding_{i}"));
        names.push(format!("mean_metabolism_{i}"));
        names.push(format!("traded_{i}"));
    }
    for k in 0..config.pollution.pollutants.len() {
        names.push(format!("mean_pollution_{k}"));
    }
    for k in 0..config.culture.groups.len() {
        names.push(format!("group_share_{k}"));
    }
    names
}
```
Add the field at the end of `struct Snapshot`:
```rust
    /// Share of living agents in each group (`culture.groups`); 0 with no
    /// agents.
    pub groups: Vec<f64>,
```
In `Snapshot::of`, directly after `let pollution = …;`, add:
```rust
        let groups = &world.config.culture.groups;
        let mut members = vec![0usize; groups.len()];
        for a in world.agents() {
            if let Some(m) = members.get_mut(a.group(groups)) {
                *m += 1;
            }
        }
        let group_shares = members
            .iter()
            .map(|&m| if n == 0 { 0.0 } else { m as f64 / n as f64 })
            .collect();
```
replace the `blue_fraction` line with
```rust
            // Group 0 is Blue under the default groups (Decision 6).
            blue_fraction: mean(&|a| if a.group(groups) == 0 { 1.0 } else { 0.0 }),
```
and add `groups: group_shares,` after `pollution,` in the `Self { … }` literal. In `Snapshot::value`, directly before the final `return None;`, add:
```rust
                if let Some(k) = index("group_share_") {
                    return self.groups.get(k).copied();
                }
```

- [ ] **Step 5: Rendering**

In `crates/sugarscape-core/src/render.rs`, change `use crate::agent::{Agent, Sex, Tribe};` to
```rust
use crate::agent::{Agent, Sex};
use crate::config::Group;
```
replace `struct Scales` with
```rust
struct Scales<'a> {
    log_max_wealth: f64,
    vision_min: f64,
    vision_span: f64,
    /// The config's groups and their parsed colors (Tribe mode).
    groups: &'a [Group],
    group_colors: Vec<Rgb>,
}
```
replace the `ColorMode::Tribe` arm of `agent_color` with
```rust
        ColorMode::Tribe => s
            .group_colors
            .get(a.group(s.groups))
            .copied()
            .unwrap_or(NEUTRAL),
```
and in `render`, replace the `let scales = Scales { … };` statement with
```rust
    let groups = &world.config.culture.groups;
    let scales = Scales {
        log_max_wealth: world
            .agents()
            .map(|a| a.holdings[0])
            .fold(0.0, f64::max)
            .ln_1p()
            .max(1e-9),
        vision_min: f64::from(v.min),
        vision_span: f64::from(v.max.saturating_sub(v.min).max(1)),
        groups,
        group_colors: groups
            .iter()
            .map(|g| parse_color(&g.color).unwrap_or(NEUTRAL))
            .collect(),
    };
```

- [ ] **Step 6: Inspection**

In `crates/sugarscape-core/src/edit.rs`, add to `struct AgentView` after `pub tribe: Tribe,`:
```rust
    /// Index of the agent's group in `culture.groups`.
    pub group: usize,
```
and in `inspect`, after `tribe: a.tribe(),`:
```rust
            group: a.group(&self.config.culture.groups),
```

- [ ] **Step 7: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
```
Expected: all pass. Golden unchanged: with the default groups, combat's "same group" is exactly the old "same tribe", and statistics are not hashed.

- [ ] **Step 8: Commit**

```bash
git add crates/sugarscape-core/src/agent.rs crates/sugarscape-core/src/rules/combat.rs crates/sugarscape-core/src/rules/culture.rs crates/sugarscape-core/src/stats.rs crates/sugarscape-core/src/export.rs crates/sugarscape-core/src/render.rs crates/sugarscape-core/src/edit.rs
git commit -m "Use tag groups for combat, group shares, Tribe colors and inspection" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 3: The `iii-6-three-tribes` preset

*Mechanical (full code); the golden value is printed, and predicted.*

**Files:**
- Modify: `crates/sugarscape-core/src/presets.rs`
- Modify (append one entry): `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `config::three_tribes` (Task 1).
- Produces: preset `iii-6-three-tribes` (listed right after `iii-6-culture`); `GOLDEN` gains `("iii-6-three-tribes", …)` as its last entry.

- [ ] **Step 1: Write the failing test**

In `crates/sugarscape-core/src/presets.rs`, in `every_preset_is_valid_and_runs`, change `assert_eq!(presets.len(), 26);` to `assert_eq!(presets.len(), 27);`, and append inside `mod tests`:
```rust
    #[test]
    fn three_tribes_is_the_culture_preset_with_the_books_groups() {
        let three = by_id("iii-6-three-tribes").unwrap().config;
        let mut culture = by_id("iii-6-culture").unwrap().config;
        culture.culture.groups = three_tribes(11);
        assert_eq!(three, culture);
        let spans: Vec<(&str, u32, u32)> = three
            .culture
            .groups
            .iter()
            .map(|g| (g.name.as_str(), g.zeros.min, g.zeros.max))
            .collect();
        assert_eq!(spans, [("Blue", 0, 3), ("Green", 4, 7), ("Red", 8, 11)]);
    }
```
Run: `cargo test -p sugarscape-core presets::`
Expected: compile error (`three_tribes` not imported) — then, after importing only, the two tests fail (26 presets; `by_id` returns `None`).

- [ ] **Step 2: Add the preset**

Add `three_tribes` to the `use crate::config::{…}` list at the top of `presets.rs`, and insert directly after the `iii-6-culture` entry in `all()`:
```rust
        preset(
            "iii-6-three-tribes",
            "({G₁}, {M, K}) with three tribes",
            "Chapter III, note 20",
            "Cultural transmission with the book's three-group tag scheme: Blue 0–3 zeros, Green 4–7, Red 8–11. Watch the Group shares chart.",
            |c| {
                c.culture.enabled = true;
                c.culture.groups = three_tribes(c.tag_length);
            },
        ),
```
Run: `cargo test -p sugarscape-core presets::`
Expected: pass. `cargo test -p sugarscape-core --test golden` now fails `every_preset_has_a_golden_entry` ("record a golden fingerprint for iii-6-three-tribes").

- [ ] **Step 3: Record the golden fingerprint**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`
Expected: the line `("iii-6-three-tribes", 0xf8973190b0435a81),` — the same value as `iii-6-culture`, because no rule reads groups while combat is off and the fingerprint does not hash the config. **If the printed value differs, stop and report BLOCKED** with both values: something other than combat, statistics or rendering reads groups. Also check that every other printed value equals its existing `GOLDEN` entry.

Append the entry as the **last** element of `GOLDEN` in `crates/sugarscape-core/tests/golden.rs` (change nothing else):
```rust
    // Model extensions: the culture preset with the book's three groups.
    // Groups change no rule while combat is off, so it equals iii-6-culture.
    ("iii-6-three-tribes", 0xf8973190b0435a81),
```

- [ ] **Step 4: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
git diff crates/sugarscape-core/tests/golden.rs
```
Expected: all pass; the diff shows only the three added lines.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs
git commit -m "Add the three-tribe culture preset" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 4: Groups in the browser

*Mechanical (full code).*

**Files:**
- Create: `web/src/groups.ts`, `web/src/groups.test.ts`, `web/src/schema.test.ts`, `web/src/ui/groups-editor.ts`
- Modify: `web/src/types.ts`, `web/src/schema.ts`, `web/src/ui/rules-panel.ts`, `web/src/ui/charts-panel.ts`, `web/src/ui/inspect-panel.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: the core's config JSON (`culture.groups`), `AgentView.group`, `Snapshot.groups`, series `group_share_K` (Tasks 1–2); `bind`, `editor`, `num`, `Commit`, `Editor`, `Sync` from `ui/goods-editor.ts`.
- Produces:
  - `types.ts`: `export interface TagGroup { name: string; color: string; zeros: URange }`; `Config.culture: { enabled: boolean; groups: TagGroup[] }`; `AgentView.group: number`; `Snapshot.groups: number[]`.
  - `groups.ts`: `MAX_GROUPS`, `BLUE`, `GREEN`, `RED`, `GROUP_PALETTE`, `defaultGroups(tagLength: number): TagGroup[]`, `threeTribes(tagLength: number): TagGroup[]`, `sameGroups(a: TagGroup[], b: TagGroup[]): boolean`, `canAddGroup(config: Config): boolean`, `addGroup(config: Config): void`, `removeGroup(config: Config, k: number): void`, `moveBoundary(groups: TagGroup[], k: number, end: 'min' | 'max', value: number): void`, `groupsEditorSignature(config: Config): string`, `groupSharesSignature(config: Config): string`.
  - `schema.ts`: `Control` base gains `adjust?: (next: Config, before: Config) => void`; `Group.custom` gains `'groups'`.
  - `ui/groups-editor.ts`: `groupsEditor(config: Config, commit: Commit): Editor`.

- [ ] **Step 1: Types**

In `web/src/types.ts`, add after `export interface Pollutant …`:
```ts
/** A tag group (tribe): agents whose tags hold a number of zeros in `zeros`. */
export interface TagGroup { name: string; color: string; zeros: URange }
```
change `culture: { enabled: boolean };` to `culture: { enabled: boolean; groups: TagGroup[] };`, add `groups: number[];` as the last field of `Snapshot` (after `pollution: number[];`), and add `group: number;` to `AgentView` after `tribe: 'blue' | 'red';`.

- [ ] **Step 2: Write the failing tests**

Create `web/src/groups.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import type { Config, TagGroup } from './types';
import {
  MAX_GROUPS,
  addGroup,
  canAddGroup,
  defaultGroups,
  groupSharesSignature,
  groupsEditorSignature,
  moveBoundary,
  removeGroup,
  sameGroups,
  threeTribes,
} from './groups';

const config = (groups: TagGroup[], tagLength = 11): Config =>
  ({ tag_length: tagLength, culture: { enabled: false, groups } }) as unknown as Config;
const ranges = (c: Config) => c.culture.groups.map((g) => [g.zeros.min, g.zeros.max]);
/** Every zero count 0..=L lies in exactly one group. */
const tiles = (c: Config) =>
  Array.from({ length: c.tag_length + 1 }, (_, z) => c.culture.groups.filter((g) => g.zeros.min <= z && z <= g.zeros.max).length).every(
    (n) => n === 1,
  );

describe('book groups (mirroring config::default_groups and config::three_tribes)', () => {
  it('makes Blue the tags whose zeros outnumber their ones', () => {
    expect(defaultGroups(11)).toEqual([
      { name: 'Blue', color: '#3d7eff', zeros: { min: 6, max: 11 } },
      { name: 'Red', color: '#ff4d4d', zeros: { min: 0, max: 5 } },
    ]);
    expect(defaultGroups(4).map((g) => g.zeros)).toEqual([{ min: 3, max: 4 }, { min: 0, max: 2 }]);
    expect(defaultGroups(1).map((g) => g.zeros)).toEqual([{ min: 1, max: 1 }, { min: 0, max: 0 }]);
  });

  it("splits 11-bit tags into the book's three tribes", () => {
    expect(threeTribes(11)).toEqual([
      { name: 'Blue', color: '#3d7eff', zeros: { min: 0, max: 3 } },
      { name: 'Green', color: '#3dd66b', zeros: { min: 4, max: 7 } },
      { name: 'Red', color: '#ff4d4d', zeros: { min: 8, max: 11 } },
    ]);
    expect(threeTribes(2).map((g) => g.zeros)).toEqual([{ min: 0, max: 0 }, { min: 1, max: 1 }, { min: 2, max: 2 }]);
  });

  it("writes keys in the core's order, so preset matching works", () => {
    expect(JSON.stringify(defaultGroups(11)[0])).toBe('{"name":"Blue","color":"#3d7eff","zeros":{"min":6,"max":11}}');
    expect(sameGroups(defaultGroups(11), defaultGroups(11))).toBe(true);
    expect(sameGroups(defaultGroups(11), threeTribes(11))).toBe(false);
  });
});

describe('groups table', () => {
  it('moves the neighbouring end with a range end, so the ranges still tile', () => {
    const c = config(threeTribes(11));
    moveBoundary(c.culture.groups, 0, 'max', 5);
    expect(ranges(c)).toEqual([[0, 5], [6, 7], [8, 11]]);
    moveBoundary(c.culture.groups, 2, 'min', 7);
    expect(ranges(c)).toEqual([[0, 5], [6, 6], [7, 11]]);
    expect(tiles(c)).toBe(true);
  });

  it('adds a group by splitting the widest and removes one into its neighbour', () => {
    const c = config(defaultGroups(11));
    addGroup(c);
    expect(ranges(c)).toEqual([[6, 8], [0, 5], [9, 11]]);
    expect(c.culture.groups[2]).toEqual({ name: 'group 3', color: '#3dd66b', zeros: { min: 9, max: 11 } });
    expect(tiles(c)).toBe(true);
    removeGroup(c, 0);
    expect(ranges(c)).toEqual([[0, 8], [9, 11]]);
    removeGroup(c, 0);
    expect(ranges(c)).toEqual([[0, 11]]);
    removeGroup(c, 0);
    expect(c.culture.groups).toHaveLength(1);
    expect(tiles(c)).toBe(true);
  });

  it('adds only while a group can be split and fewer than eight exist', () => {
    expect(canAddGroup(config(defaultGroups(1), 1))).toBe(false);
    const eight = config(Array.from({ length: MAX_GROUPS }, (_, k) => ({ name: `g${k}`, color: '#000000', zeros: { min: k, max: k === 7 ? 11 : k } })));
    expect(canAddGroup(eight)).toBe(false);
    addGroup(eight);
    expect(eight.culture.groups).toHaveLength(MAX_GROUPS);
    expect(canAddGroup(config(defaultGroups(11)))).toBe(true);
  });

  it('rebuilds the table only when the tag length or a range changes', () => {
    const c = config(defaultGroups(11));
    const before = groupsEditorSignature(c);
    const charts = groupSharesSignature(c);
    c.culture.groups[0].name = 'Azure';
    expect(groupsEditorSignature(c)).toBe(before);
    expect(groupSharesSignature(c)).not.toBe(charts);
    moveBoundary(c.culture.groups, 0, 'min', 7);
    expect(groupsEditorSignature(c)).not.toBe(before);
    expect(groupsEditorSignature(config(defaultGroups(11), 12))).not.toBe(groupsEditorSignature(config(defaultGroups(11))));
  });
});
```
Create `web/src/schema.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { defaultGroups, threeTribes } from './groups';
import { GROUPS } from './schema';
import type { Config, TagGroup } from './types';

const config = (groups: TagGroup[], tagLength: number): Config =>
  ({ tag_length: tagLength, culture: { enabled: false, groups } }) as unknown as Config;
const control = (path: string) => GROUPS.flatMap((g) => g.controls).find((c) => c.path === path)!;

describe('tag length', () => {
  it('rebuilds the default groups for the new length', () => {
    const next = config(defaultGroups(11), 5);
    control('tag_length').adjust!(next, config(defaultGroups(11), 11));
    expect(next.culture.groups).toEqual(defaultGroups(5));
  });

  it('keeps custom groups (validation reports a mismatch)', () => {
    const next = config(threeTribes(11), 5);
    control('tag_length').adjust!(next, config(threeTribes(11), 11));
    expect(next.culture.groups).toEqual(threeTribes(11));
  });
});

describe('culture', () => {
  it('shows the groups table', () => {
    expect(GROUPS.find((g) => g.enable === 'culture.enabled')?.custom).toBe('groups');
  });
});
```
Run: `cd web && npx vitest run src/groups.test.ts src/schema.test.ts`
Expected: FAIL (`./groups` does not exist).

- [ ] **Step 3: `groups.ts`**

Create `web/src/groups.ts`:
```ts
import type { Config, TagGroup } from './types';

/** Mirrors `config::MAX_GROUPS` and `config::{BLUE, GREEN, RED}_COLOR`. */
export const MAX_GROUPS = 8;
export const BLUE = '#3d7eff';
export const GREEN = '#3dd66b';
export const RED = '#ff4d4d';
/** Colors new groups take, first unused first. */
export const GROUP_PALETTE = [BLUE, RED, GREEN, '#ffe04d', '#b894f0', '#ff8a5c', '#6ad0c4', '#d67a8f'];

/** A group with keys in the core's serde order. */
function group(name: string, color: string, min: number, max: number): TagGroup {
  return { name, color, zeros: { min, max } };
}

/** The book's two tribes on `tagLength`-bit tags (mirrors `config::default_groups`): Blue when zeros outnumber ones. */
export function defaultGroups(tagLength: number): TagGroup[] {
  const blueFrom = Math.floor(tagLength / 2) + 1;
  return [group('Blue', BLUE, blueFrom, tagLength), group('Red', RED, 0, blueFrom - 1)];
}

/** Chapter III note 20's three groups, thirds of 0..=L (mirrors `config::three_tribes`); needs tags of at least 2 bits. */
export function threeTribes(tagLength: number): TagGroup[] {
  const cut = (k: number) => Math.floor((k * (tagLength + 1)) / 3);
  return [group('Blue', BLUE, 0, cut(1) - 1), group('Green', GREEN, cut(1), cut(2) - 1), group('Red', RED, cut(2), tagLength)];
}

export function sameGroups(a: TagGroup[], b: TagGroup[]): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** Fewer than MAX_GROUPS, and some group holds two or more zero counts to split. */
export function canAddGroup(config: Config): boolean {
  const groups = config.culture.groups;
  return groups.length < MAX_GROUPS && groups.some((g) => g.zeros.max > g.zeros.min);
}

/** Splits the widest group (the first of equals): the new group takes the upper half of its zero counts. */
export function addGroup(config: Config): void {
  if (!canAddGroup(config)) return;
  const groups = config.culture.groups;
  const width = (g: TagGroup) => g.zeros.max - g.zeros.min;
  const widest = groups.reduce((best, g) => (width(g) > width(best) ? g : best), groups[0]);
  const mid = Math.floor((widest.zeros.min + widest.zeros.max) / 2);
  const names = groups.map((g) => g.name);
  let k = groups.length + 1;
  while (names.includes(`group ${k}`)) k++;
  const colors = new Set(groups.map((g) => g.color.toLowerCase()));
  const color = GROUP_PALETTE.find((c) => !colors.has(c)) ?? GROUP_PALETTE[groups.length % GROUP_PALETTE.length];
  groups.push(group(`group ${k}`, color, mid + 1, widest.zeros.max));
  widest.zeros.max = mid;
}

/** Removes group `k` (never the last); its zero counts join the group just below them, else the one just above. */
export function removeGroup(config: Config, k: number): void {
  const groups = config.culture.groups;
  if (groups.length <= 1 || k < 0 || k >= groups.length) return;
  const g = groups[k];
  const below = groups.find((o) => o !== g && o.zeros.max === g.zeros.min - 1);
  const above = groups.find((o) => o !== g && o.zeros.min === g.zeros.max + 1);
  if (below) below.zeros.max = g.zeros.max;
  else if (above) above.zeros.min = g.zeros.min;
  groups.splice(k, 1);
}

/** Sets one end of group `k`'s range, moving the adjoining group's end with it so the ranges still tile. */
export function moveBoundary(groups: TagGroup[], k: number, end: 'min' | 'max', value: number): void {
  const g = groups[k];
  if (!g) return;
  if (end === 'max') {
    const next = groups.find((o) => o !== g && o.zeros.min === g.zeros.max + 1);
    g.zeros.max = value;
    if (next) next.zeros.min = value + 1;
  } else {
    const prev = groups.find((o) => o !== g && o.zeros.max === g.zeros.min - 1);
    g.zeros.min = value;
    if (prev) prev.zeros.max = value - 1;
  }
}

/** The groups table's structure: the tag length and each range (names and colors refresh in place). */
export function groupsEditorSignature(config: Config): string {
  return JSON.stringify([config.tag_length, config.culture.groups.map((g) => [g.zeros.min, g.zeros.max])]);
}

/** The Group shares chart's lines: each group's name and color. */
export function groupSharesSignature(config: Config): string {
  return JSON.stringify(config.culture.groups.map((g) => [g.name, g.color]));
}
```

- [ ] **Step 4: Schema**

In `web/src/schema.ts`, add `import { defaultGroups, sameGroups } from './groups';` after the existing import; change the `Base` interface to
```ts
interface Base {
  path: string;
  label: string;
  reset?: boolean;
  hint?: string;
  /** Called after the control sets its value on a copy of the config (`before` is the copy as it was). */
  adjust?: (next: Config, before: Config) => void;
}
```
change `custom?: 'goods' | 'pollution';` to `custom?: 'goods' | 'pollution' | 'groups';`, replace the `tag_length` control with
```ts
      {
        kind: 'number', path: 'tag_length', label: 'Tag length', min: 1, max: 64, step: 1, reset: true,
        // Default groups follow the tag length; custom groups are kept (Decision 7).
        adjust: (next, before) => {
          if (sameGroups(before.culture.groups, defaultGroups(before.tag_length))) next.culture.groups = defaultGroups(next.tag_length);
        },
      },
```
and replace `{ title: 'Culture (K)', enable: 'culture.enabled', controls: [] },` with
```ts
  {
    title: 'Culture (K)', enable: 'culture.enabled',
    custom: 'groups',
    note: 'An agent belongs to the first group whose range holds its number of zero tags. Combat, the Tribe colors and the Group shares chart use the groups even while culture is off. Adding or removing a group or changing a range rebuilds the world; names and colors apply live.',
    controls: [],
  },
```

- [ ] **Step 5: The groups table**

Create `web/src/ui/groups-editor.ts`:
```ts
import { addGroup, canAddGroup, defaultGroups, moveBoundary, removeGroup, threeTribes } from '../groups';
import type { Config } from '../types';
import { h } from './dom';
import { bind, editor, num, type Commit, type Editor, type Sync } from './goods-editor';

/** Group `k`: color and name apply live; its range ends rebuild the world. */
function groupRow(config: Config, k: number, commit: Commit, syncs: Sync[]): HTMLElement {
  const live = (f: (c: Config) => void) => commit(f, false);
  const name = h('input', { type: 'text', maxLength: 16, class: 'name', 'aria-label': `Group ${k + 1} name` });
  bind(syncs, name, (c) => (name.value = c.culture.groups[k].name));
  name.addEventListener('change', () => live((c) => (c.culture.groups[k].name = name.value)));
  const color = h('input', { type: 'color', 'aria-label': `Group ${k + 1} color` });
  bind(syncs, color, (c) => (color.value = c.culture.groups[k].color));
  color.addEventListener('change', () => live((c) => (c.culture.groups[k].color = color.value)));
  const end = (which: 'min' | 'max') =>
    num(syncs, (c) => c.culture.groups[k].zeros[which], 0, config.tag_length, 1, (v) =>
      commit((c) => moveBoundary(c.culture.groups, k, which, v), true),
    );
  const remove = h('button', {
    disabled: config.culture.groups.length <= 1,
    title: 'Remove this group (its zero counts join a neighbour; rebuilds the world)',
    'aria-label': `Remove group ${k + 1}`,
    onclick: () => commit((c) => removeGroup(c, k), true),
  }, '×');
  return h('div', { class: 'row' }, color, name, h('span', { class: 'hint' }, 'zeros'), end('min'), h('span', { class: 'hint' }, 'to'), end('max'), remove);
}

/** The Culture section's groups table, for `config`'s `groupsEditorSignature`. */
export function groupsEditor(config: Config, commit: Commit): Editor {
  const syncs: Sync[] = [];
  const el = h(
    'div',
    { class: 'tag-groups' },
    h(
      'div',
      { class: 'row' },
      h('button', { onclick: () => commit((c) => (c.culture.groups = defaultGroups(c.tag_length)), true) }, 'Two tribes (book)'),
      h('button', {
        disabled: config.tag_length < 2,
        title: 'Blue, Green and Red: thirds of the zero counts (0–3, 4–7, 8–11 on 11-bit tags)',
        onclick: () => commit((c) => (c.culture.groups = threeTribes(c.tag_length)), true),
      }, 'Three tribes (book)'),
    ),
    ...config.culture.groups.map((_, k) => groupRow(config, k, commit, syncs)),
    h('button', { disabled: !canAddGroup(config), onclick: () => commit((c) => addGroup(c), true) }, 'Add group'),
  );
  return editor(el, syncs, config);
}
```

- [ ] **Step 6: Wire it into the Rules panel**

In `web/src/ui/rules-panel.ts`: add `import { groupsEditorSignature } from '../groups';` and `import { groupsEditor } from './groups-editor';`, change `import { GROUPS, type Control } from '../schema';` to `import { GROUPS, type Control, type Group } from '../schema';`, and add after the imports:
```ts
type CustomEditor = NonNullable<Group['custom']>;

const EDITORS: Record<CustomEditor, { build: (c: Config, commit: Commit) => Editor; signature: (c: Config) => string; errors: string }> = {
  goods: { build: goodsEditor, signature: goodsEditorSignature, errors: 'goods' },
  pollution: { build: pollutionEditor, signature: pollutionEditorSignature, errors: 'pollution.pollutants' },
  groups: { build: groupsEditor, signature: groupsEditorSignature, errors: 'culture.groups' },
};
```
Replace `customEditor` with:
```ts
  /**
   * Rebuilds a custom editor only when its structure changes; otherwise refreshes its values in
   * place, so focus and half-typed input survive live and scheduled edits.
   */
  private customEditor(kind: CustomEditor): HTMLElement[] {
    const holder = h('div');
    const commit: Commit = (mutate, reset) => this.commit(mutate, reset);
    const { build, signature, errors } = EDITORS[kind];
    let built: string | null = null;
    let current: Editor | null = null;
    this.editorSyncers.push(() => {
      const config = this.engine.config;
      const next = signature(config);
      if (current && next === built) {
        current.sync(config);
        return;
      }
      built = next;
      current = build(config, commit);
      holder.replaceChildren(current.el);
    });
    return [holder, this.errorSlot(errors, true)];
  }
```
In `control`, `case 'number'`, replace `const apply = (v: string) => this.commit((cfg) => setPath(cfg, c.path, Number(v)), reset);` with:
```ts
        const apply = (v: string) =>
          this.commit((cfg) => {
            const before = structuredClone(cfg);
            setPath(cfg, c.path, Number(v));
            c.adjust?.(cfg, before);
          }, reset);
```

- [ ] **Step 7: Group shares chart and the inspector**

In `web/src/ui/charts-panel.ts`: add `import { groupSharesSignature } from '../groups';`; delete the `{ title: 'Blue share', … }` entry from `TIME_CHARTS`; add the fields
```ts
  /** Holds the Group shares chart, whose lines follow `culture.groups`. */
  private groupsSection = h('section', { class: 'group-shares' });
  private groupPlots = new Set<uPlot>();
```
add the method
```ts
  private rebuildGroupChart(): void {
    this.plots = this.plots.filter((p) => {
      if (!this.groupPlots.has(p.plot)) return true;
      p.plot.destroy();
      return false;
    });
    this.groupPlots.clear();
    this.groupsSection.replaceChildren();
    const before = this.plots.length;
    const lines = this.engine.config.culture.groups.map((g, k) => ({ key: `group_share_${k}`, label: g.name, color: g.color }));
    this.addTimeChart({ title: 'Group shares', lines, range: [0, 1] }, this.groupsSection);
    for (const p of this.plots.slice(before)) this.groupPlots.add(p.plot);
    this.resize();
  }
```
and in `build()` replace `for (const chart of TIME_CHARTS) this.addTimeChart(chart);` with
```ts
    for (const chart of TIME_CHARTS) {
      this.addTimeChart(chart);
      // Group shares sit where the Blue share chart was.
      if (chart.title === 'Mean traits') this.el.append(this.groupsSection);
    }
    let groupLines = '';
    const syncGroupChart = () => {
      const next = groupSharesSignature(this.engine.config);
      if (next === groupLines) return;
      groupLines = next;
      this.rebuildGroupChart();
    };
    this.engine.on('reset', syncGroupChart);
    this.engine.on('config', syncGroupChart);
    syncGroupChart();
```
In `web/src/ui/inspect-panel.ts`, in `agentRows`, replace `row('Agent', \`#${a.id} · ${a.sex} · ${a.tribe}\`),` with
```ts
      row('Agent', `#${a.id} · ${a.sex} · ${this.engine.config.culture.groups[a.group]?.name ?? a.tribe}`),
```

- [ ] **Step 8: Styles**

Append to `web/src/style.css`:
```css
.tag-groups { display: grid; gap: 4px; margin-top: 6px; }
.tag-groups .name { width: 7em; }
.tag-groups .num { width: 4em; }
.tag-groups input[type='color'] { padding: 0; width: 2.2em; height: 1.8em; }
```

- [ ] **Step 9: Verify**

```bash
(cd web && npm run build && npm test)
```
Expected: `tsc` clean, build succeeds, all Vitest suites pass (including `groups.test.ts` and `schema.test.ts`).

- [ ] **Step 10: Commit**

```bash
git add web/src/types.ts web/src/groups.ts web/src/groups.test.ts web/src/schema.ts web/src/schema.test.ts web/src/ui/groups-editor.ts web/src/ui/rules-panel.ts web/src/ui/charts-panel.ts web/src/ui/inspect-panel.ts web/src/style.css
git commit -m "Edit tag groups in the Culture section and chart group shares" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 5: The price rule in the core

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs`, `crates/sugarscape-core/src/legacy.rs`, `crates/sugarscape-core/src/rules/trade.rs`

**Interfaces:**
- Consumes: `World.rng` (`pub(crate)`), `econ::{mrs_n, welfare_n}`, testkit `blank_world`, `add_goods`, `spawn`.
- Produces: `pub enum PriceRule { GeometricMean, Random }` (`Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize`, snake_case, default `GeometricMean`); `pub struct TradeRule { pub enabled: bool, #[serde(default)] pub price: PriceRule }`; `Config.trade: TradeRule` (was `Toggle`).

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` in `crates/sugarscape-core/src/config.rs`:
```rust
    #[test]
    fn the_price_rule_defaults_to_the_geometric_mean_and_may_be_scheduled() {
        let c = Config::default();
        assert_eq!(c.trade.price, PriceRule::GeometricMean);
        let json = serde_json::to_string(&c).unwrap();
        assert!(
            json.contains(r#""trade":{"enabled":false,"price":"geometric_mean"}"#),
            "{json}"
        );
        let mut value = serde_json::to_value(&c).unwrap();
        value["trade"] = serde_json::json!({"enabled": false});
        assert_eq!(
            Config::from_json(&value.to_string()).unwrap().trade.price,
            PriceRule::GeometricMean
        );
        value["trade"] = serde_json::json!({"enabled": false, "price": "random"});
        assert_eq!(
            Config::from_json(&value.to_string()).unwrap().trade.price,
            PriceRule::Random
        );
        value["trade"] = serde_json::json!({"enabled": false, "price": "haggle"});
        assert_eq!(
            Config::from_json(&value.to_string()).unwrap_err()[0].field,
            "config"
        );
        let legacy = Config::from_json(r#"{"trade": {"enabled": false}}"#).unwrap();
        assert_eq!(legacy.trade.price, PriceRule::GeometricMean);
        let mut two = Config::default();
        two.add_good(Good::spice());
        two.trade.enabled = true;
        two.schedule = vec![change(5, "trade.price", serde_json::json!("random"))];
        two.validate().unwrap();
        let next = two.apply_change(&two.schedule[0]).unwrap();
        assert_eq!(next.trade.price, PriceRule::Random);
    }
```
Append inside `mod tests` in `crates/sugarscape-core/src/rules/trade.rs` (and change the tests' `use crate::econ::{mrs, welfare, welfare_n};` to `use crate::econ::{mrs, mrs_n, welfare, welfare_n};`):
```rust
    #[test]
    fn geometric_mean_pricing_draws_nothing() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        let a = trader(&mut w, 0, 5.0, 8.0);
        let b = trader(&mut w, 1, 15.0, 2.0);
        let rng = w.rng.clone();
        trade_pair(&mut w, a, b);
        assert!(!w.events().trades.is_empty());
        assert!(w.rng == rng, "no draws under geometric_mean");
    }

    #[test]
    fn random_prices_draw_from_the_rng_and_lie_between_the_two_mrss() {
        let mut traded = 0;
        for seed in 0..20 {
            let mut w = blank_world(5, 5);
            add_goods(&mut w.config, 2);
            w.config.trade.price = PriceRule::Random;
            w.rng = crate::rng::seeded(seed);
            let a = trader(&mut w, 0, 5.0, 8.0);
            let b = trader(&mut w, 1, 15.0, 2.0);
            let rng = w.rng.clone();
            trade_pair(&mut w, a, b);
            assert!(w.rng != rng, "seed {seed}: random pricing draws");
            if let Some(first) = w.events().trades.first() {
                // Before the first exchange MRS_A = 8/5 and MRS_B = 2/15.
                assert!(
                    (2.0 / 15.0..=1.6).contains(&first.price),
                    "seed {seed}: {}",
                    first.price
                );
                traded += 1;
            }
        }
        assert!(traded > 0, "some seed's first price passes the checks");
    }

    #[test]
    fn random_pricing_is_reproducible_per_seed_and_changes_the_run() {
        let geometric = crate::presets::by_id("iv-3-trade").unwrap().config;
        let mut random = geometric.clone();
        random.trade.price = PriceRule::Random;
        let run = |c: &crate::config::Config, seed| {
            let mut w = World::new(c.clone(), seed).unwrap();
            w.run(5);
            w.fingerprint()
        };
        assert_eq!(run(&random, 1), run(&random, 1));
        assert_ne!(run(&random, 1), run(&geometric, 1));
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]
        #[test]
        fn random_prices_stay_between_the_mrss_and_every_check_holds(
            n in 2usize..=4,
            a in proptest::collection::vec(0.5f64..60.0, 4),
            b in proptest::collection::vec(0.5f64..60.0, 4),
            ma in proptest::collection::vec(0u32..6, 4),
            mb in proptest::collection::vec(0u32..6, 4),
            seed in 0u64..1000,
        ) {
            let mut w = blank_world(5, 5);
            add_goods(&mut w.config, n);
            w.config.trade.price = PriceRule::Random;
            w.rng = crate::rng::seeded(seed);
            let x = spawn(&mut w, 0, 0);
            let y = spawn(&mut w, 1, 0);
            let mut held = std::collections::BTreeMap::new();
            let mut mets = std::collections::BTreeMap::new();
            for (id, h, m) in [(x, &a, &ma), (y, &b, &mb)] {
                let agent = w.agent_mut(id).unwrap();
                agent.holdings[..n].copy_from_slice(&h[..n]);
                agent.metabolism[..n].copy_from_slice(&m[..n]);
                held.insert(id, h[..n].to_vec());
                mets.insert(id, m[..n].iter().map(|&v| f64::from(v)).collect::<Vec<f64>>());
            }
            trade_pair(&mut w, x, y);
            // Replay each exchange with `exchange`'s own arithmetic.
            for t in w.events().trades.clone() {
                let (i, j) = t.goods;
                let mrs = |id: AgentId, h: &[f64]| mrs_n(h, &mets[&id], i, j);
                let (buy0, sell0) = (held[&t.buyer].clone(), held[&t.seller].clone());
                let (m_buy, m_sell) = (mrs(t.buyer, &buy0), mrs(t.seller, &sell0));
                proptest::prop_assert!(
                    m_buy.min(m_sell) <= t.price && t.price <= m_buy.max(m_sell),
                    "price {} outside [{}, {}]", t.price, m_sell, m_buy
                );
                let paid = if t.price >= 1.0 { t.price } else { 1.0 };
                let (mut buy, mut sell) = (buy0.clone(), sell0.clone());
                buy[i] = buy0[i] + t.amount;
                buy[j] = buy0[j] - paid;
                sell[i] = sell0[i] - t.amount;
                sell[j] = sell0[j] + paid;
                proptest::prop_assert!(buy.iter().chain(&sell).all(|&v| v > 0.0));
                proptest::prop_assert!(welfare_n(&buy, &mets[&t.buyer]) > welfare_n(&buy0, &mets[&t.buyer]));
                proptest::prop_assert!(welfare_n(&sell, &mets[&t.seller]) > welfare_n(&sell0, &mets[&t.seller]));
                proptest::prop_assert!(mrs(t.buyer, &buy) >= mrs(t.seller, &sell), "MRSs crossed");
                held.insert(t.buyer, buy);
                held.insert(t.seller, sell);
            }
            for id in [x, y] {
                proptest::prop_assert_eq!(&w.agent(id).unwrap().holdings[..n], &held[&id][..]);
            }
        }
    }
```
Run: `cargo test -p sugarscape-core`
Expected: compile errors (`PriceRule`, `trade.price` not found).

- [ ] **Step 2: `PriceRule` and `TradeRule`**

In `crates/sugarscape-core/src/config.rs`, insert after `CultureRule`:
```rust
/// How rule T prices an exchange (Decision 8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceRule {
    /// The book's p = √(MRS_A · MRS_B).
    #[default]
    GeometricMean,
    /// Chapter IV note 15: p drawn uniformly from [MRS_A, MRS_B].
    Random,
}

/// T: trade between neighbors, priced by `price`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeRule {
    pub enabled: bool,
    #[serde(default)]
    pub price: PriceRule,
}
```
In `struct Config`, change `pub trade: Toggle,` to `pub trade: TradeRule,`; in `impl Default for Config`, change `trade: Toggle { enabled: false },` to:
```rust
            trade: TradeRule {
                enabled: false,
                price: PriceRule::GeometricMean,
            },
```
In `crates/sugarscape-core/src/legacy.rs`, add `PriceRule, TradeRule` to the config imports; in `impl Default for LegacyConfig`, change `trade: c.trade,` to
```rust
            trade: Toggle {
                enabled: c.trade.enabled,
            },
```
and in `convert`, change `trade: old.trade,` to
```rust
        trade: TradeRule {
            enabled: old.trade.enabled,
            price: PriceRule::GeometricMean,
        },
```

- [ ] **Step 3: The random price**

In `crates/sugarscape-core/src/rules/trade.rs`, replace the module doc's `…buys it with\n//! good j at p = √(MRS_A·MRS_B), …` sentence so the doc reads:
```rust
//! Agent trade rule T (Chapter IV) over pairs of goods. With each neighbor,
//! in random order: rank the pairs i < j by |ln MRS_A − ln MRS_B| (widest
//! first; ties to the lowest (i, j)) and make one exchange on the first pair
//! that passes the book's checks — the agent valuing good i more buys it with
//! good j at price p, 1 of i for p of j if p ≥ 1, else 1/p of i for 1 of j —
//! as long as both agents' welfare over every good strictly rises, their MRSs
//! for the pair don't cross, and every holding stays positive; then re-rank.
//! Pairs with a non-finite, non-positive or equal MRS are skipped. With two
//! goods this is the book's loop. `trade.price` sets p: the book's
//! √(MRS_A·MRS_B), or (note 15) a draw from [MRS_A, MRS_B] made for every
//! attempted exchange — the only use of the RNG here.
```
Change the imports to
```rust
use rand::seq::SliceRandom;
use rand::Rng;

use crate::agent::AgentId;
use crate::config::{PriceRule, MAX_GOODS};
```
and in `exchange`, replace `let p = (ma * mb).sqrt();` with:
```rust
    let p = match world.config.trade.price {
        PriceRule::GeometricMean => (ma * mb).sqrt(),
        // `ranked_pairs` guarantees finite, positive, unequal MRSs.
        PriceRule::Random => world.rng.gen_range(ma.min(mb)..=ma.max(mb)),
    };
```

- [ ] **Step 4: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
```
Expected: all pass; golden unchanged (`geometric_mean` is the old expression and draws nothing).

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/config.rs crates/sugarscape-core/src/legacy.rs crates/sugarscape-core/src/rules/trade.rs
git commit -m "Add a random bargaining price as an alternative trade rule" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 6: The Price rule select

*Mechanical (full code).*

**Files:**
- Modify: `web/src/types.ts`, `web/src/schema.ts`, `web/src/schema.test.ts`

**Interfaces:**
- Consumes: `trade.price` in the core's config JSON (Task 5); the Rules panel's `select` control (applied live through `applyConfig`).
- Produces: `export type PriceRule = 'geometric_mean' | 'random'`; `Config.trade: { enabled: boolean; price: PriceRule }`; a `select` control at path `trade.price`.

- [ ] **Step 1: Write the failing test**

Append to `web/src/schema.test.ts`:
```ts
describe('trade', () => {
  it('offers the two price rules, applied live', () => {
    const price = control('trade.price');
    expect(price.kind).toBe('select');
    if (price.kind !== 'select') return;
    expect(price.options.map((o) => o.value)).toEqual(['geometric_mean', 'random']);
    expect(price.reset).toBeUndefined();
    const c = { trade: { enabled: true, price: 'geometric_mean' } } as unknown as Config;
    price.options[1].apply(c);
    expect(price.current(c)).toBe('random');
    price.options[0].apply(c);
    expect(c.trade.price).toBe('geometric_mean');
  });
});
```
Run: `cd web && npx vitest run src/schema.test.ts`
Expected: FAIL (no control at `trade.price`).

- [ ] **Step 2: Implement**

In `web/src/types.ts`, add `export type PriceRule = 'geometric_mean' | 'random';` above `export interface Config`, and change `trade: { enabled: boolean };` to `trade: { enabled: boolean; price: PriceRule };`.

In `web/src/schema.ts`, replace `{ title: 'Trade (T)', enable: 'trade.enabled', note: 'Needs at least two goods.', controls: [] },` with:
```ts
  {
    title: 'Trade (T)', enable: 'trade.enabled',
    note: 'Needs at least two goods. The price rule applies to the running world and can be scheduled.',
    controls: [
      {
        kind: 'select', path: 'trade.price', label: 'Price rule',
        current: (c) => c.trade.price,
        options: [
          { value: 'geometric_mean', label: 'Geometric mean √(MRS_A·MRS_B) (book)', apply: (c) => { c.trade.price = 'geometric_mean'; } },
          { value: 'random', label: 'Random in [MRS_A, MRS_B] (note 15)', apply: (c) => { c.trade.price = 'random'; } },
        ],
      },
    ],
  },
```

- [ ] **Step 3: Verify**

```bash
(cd web && npm run build && npm test)
```
Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add web/src/types.ts web/src/schema.ts web/src/schema.test.ts
git commit -m "Choose the trade price rule in the Rules panel" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 7: The `bargaining-rules` sweep, measured

*Needs judgement: ticks, seeds and the book test's tolerance come from measurement, by the rules below only.*

**Files:**
- Create: `sweeps/bargaining-rules.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-wasm/tests/web.rs`, `crates/sugarscape-cli/tests/cli.rs`
- Modify temporarily (not committed): `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: `sweep::{builtin, run_all, measure, Metric, Outcome, Summary}`, preset `iv-3-trade`, `trade.price` (Task 5).
- Produces: built-in `bargaining-rules` (fifth in `builtins()`); τ (the book test's tolerance) recorded in its description for Task 16.

- [ ] **Step 1: Write the sweep with its starting settings**

These start at `fig-iv-6`'s measured settings (whose Trade line is this sweep's geometric-mean line) and are replaced in Step 5. Create `sweeps/bargaining-rules.json`:
```json
{
  "name": "Bargaining rules: carrying capacity under two price rules",
  "description": "Starting settings; measured in the model extensions plan, Task 7.",
  "base": { "preset": "iv-3-trade" },
  "x": {
    "label": "Mean vision",
    "values": [
      { "at": 1, "set": { "vision": { "min": 1, "max": 1 } } },
      { "at": 2, "set": { "vision": { "min": 1, "max": 3 } } },
      { "at": 3, "set": { "vision": { "min": 1, "max": 5 } } },
      { "at": 4, "set": { "vision": { "min": 1, "max": 7 } } },
      { "at": 5, "set": { "vision": { "min": 1, "max": 9 } } },
      { "at": 6, "set": { "vision": { "min": 1, "max": 11 } } }
    ]
  },
  "series": {
    "label": "Price rule",
    "values": [
      { "at": 0, "name": "Geometric mean", "set": { "trade.price": "geometric_mean" } },
      { "at": 1, "name": "Random in [MRS_A, MRS_B]", "set": { "trade.price": "random" } }
    ]
  },
  "seeds": { "from": 1, "count": 10 },
  "ticks": 300,
  "metric": { "kind": "window_mean", "series": "population", "from": 200 }
}
```

- [ ] **Step 2: Register it (failing tests first)**

In `crates/sugarscape-core/src/sweep.rs`, in `builtin_sweeps_parse_and_validate`, change the expected id list to
```rust
            [
                "fig-ii-5",
                "fig-iv-6",
                "fig-iv-10-11",
                "n-goods-carrying-capacity",
                "bargaining-rules"
            ]
```
In `crates/sugarscape-wasm/tests/web.rs`, in `builtins_and_series_names_are_listed`, make the same change to its id list. In `crates/sugarscape-cli/tests/cli.rs`, in `presets_and_sweeps_are_listed`, add `"bargaining-rules",` after `"n-goods-carrying-capacity",`.

Run: `cargo test -p sugarscape-core sweep::`
Expected: FAIL (`builtin_sweeps_parse_and_validate`: four ids).

Then change `const BUILTINS: [Builtin; 4] = [` to `const BUILTINS: [Builtin; 5] = [` and append after the `n-goods-carrying-capacity` entry:
```rust
    Builtin {
        id: "bargaining-rules",
        json: include_str!("../../../sweeps/bargaining-rules.json"),
    },
```
Run: `cargo test -p sugarscape-core sweep::`
Expected: pass.

- [ ] **Step 3: Add the temporary measurement**

Append to `crates/sugarscape-core/tests/book.rs` (it already imports `sweep::{self, Summary, SweepResult}`, `World` and `Config`):
```rust
#[test]
#[ignore]
fn measure_bargaining_rules() {
    use sugarscape_core::sweep::{Metric, Outcome};
    // 1. Settling: mean population per 50-tick block ending t = 50, 100, …,
    //    1000, over seeds 1–5, for every (price rule, vision) cell.
    let mut probe = sweep::builtin("bargaining-rules").unwrap();
    probe.ticks = 1000;
    probe.seeds.count = 5;
    let blocks = Metric::Timeseries {
        series: "population".into(),
        every: 50,
    };
    let points = probe.points().unwrap();
    for series in 0..probe.series_count() {
        for (x, value) in probe.x.values.iter().enumerate() {
            let mut means = [0.0; 20];
            for point in points.iter().filter(|p| p.series == series && p.x == x) {
                let mut w = World::new(probe.config_for(point).unwrap(), point.seed).unwrap();
                w.run(1000);
                let history = w.stats.series("population").unwrap();
                let Outcome::Series { values } = sweep::measure(&blocks, 1000, &history) else {
                    unreachable!()
                };
                for (m, v) in means.iter_mut().zip(values) {
                    *m += v / 5.0;
                }
            }
            let shown: Vec<f64> = means.iter().map(|m| (m * 10.0).round() / 10.0).collect();
            println!("{} | vision {}: {shown:?}", probe.series_name(series), value.at);
        }
    }
    // 2. Runtime and the two lines at the file's settings, on one thread.
    let s = sweep::builtin("bargaining-rules").unwrap();
    let start = std::time::Instant::now();
    let result = sweep::run_all(&s, 1, |_, _| {}).unwrap();
    println!(
        "{} runs in {:.1} s on one thread",
        result.runs.len(),
        start.elapsed().as_secs_f64()
    );
    let Summary::Scalar(rows) = &result.summary else {
        panic!("a scalar metric was expected")
    };
    for x in 0..s.x.values.len() {
        let cell = |series: usize| rows.iter().find(|r| r.series == series && r.x == x).unwrap();
        let (g, r) = (cell(0), cell(1));
        let d = (r.mean - g.mean).abs();
        let se = (g.sd.powi(2) / g.n as f64 + r.sd.powi(2) / r.n as f64).sqrt();
        println!(
            "vision {}: geometric {:.1} (sd {:.1}), random {:.1} (sd {:.1}); |d|/g = {:.3}; (|d| + 2 se)/g = {:.3}",
            s.x.values[x].at,
            g.mean,
            g.sd,
            r.mean,
            r.sd,
            d / g.mean,
            (d + 2.0 * se) / g.mean
        );
    }
}
```
Run: `cargo test -p sugarscape-core --release --test book -- --ignored --nocapture measure_bargaining_rules`

- [ ] **Step 4: Choose, with these rules only** (do not invent other candidates)

- **Ticks and window.** Let B(t) be a cell's printed block mean ending at tick t. A candidate T ∈ {300, 500, 1000} is *settled* when, for **every** cell of **both** lines, |B(T − 100) − B(T)| and |B(T − 50) − B(T)| are both ≤ max(3, 0.05 · B(T)). Use the **smallest** settled T (1000 if none is, and say so in the description): set `"ticks": T` and `"metric": { "kind": "window_mean", "series": "population", "from": T − 100 }`.
- **Seeds.** Re-run the command after editing ticks and window; let t be the one-thread time printed at 10 seeds. Keep `"count": 10` if t ≤ 60 s; otherwise 5 if t/2 ≤ 60 s; otherwise 3 (time is proportional to the seed count). Re-run after changing the count.
- **Consistency check.** If the final settings are ticks 300, window 200–300 and 10 seeds (`fig-iv-6`'s), the printed geometric-mean means must equal `fig-iv-6`'s Trade line (41.8 at vision 1 … 76.6 at vision 6, to one decimal). If they differ, stop and report **BLOCKED**: the geometric-mean path changed.
- **Tolerance τ** (Decision 9), from the final run's last printout: τ is the **smallest** of {0.10, 0.15, 0.20, 0.25} such that at **every** vision (|d| + 2 se)/g < τ. If even 0.25 fails, stop and report **BLOCKED** with the printout: the book's "insensitive to this change" is not reproduced here; do not widen τ, change seeds or axes.

- [ ] **Step 5: Record the settings in the description**

Set the file's `ticks`, `metric.from` and `seeds.count` from Step 4, and replace `description` with (filling T, F = T − 100, N, the printed means rounded to one decimal, q = the largest printed (|d| + 2 se)/g, and τ):

`"Chapter IV note 15: carrying capacity (mean population over ticks F–T) against mean vision in iv-3-trade's ({G₁}, {M, T}) market, priced by the book's geometric mean √(MRS_A·MRS_B) and by a price drawn uniformly from [MRS_A, MRS_B]. A mean of m is the uniform range 1–(2m−1). Seeds 1–N. Measured (release, seeds 1–N): geometric mean <g₁> at vision 1 and <g₆> at vision 6; random <r₁> and <r₆>; the largest (|d| + 2 se) relative to the geometric-mean line is <q>, so the book test's tolerance is τ = <τ> of the geometric-mean line."`

Then delete `measure_bargaining_rules` from `tests/book.rs`; `git diff crates/sugarscape-core/tests/book.rs` must be empty.

- [ ] **Step 6: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
```
Expected: all pass, including `builtin_sweeps_parse_and_validate` with the final file.

- [ ] **Step 7: Commit**

```bash
git add sweeps/bargaining-rules.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-wasm/tests/web.rs crates/sugarscape-cli/tests/cli.rs
git commit -m "Ship a measured sweep comparing the two bargaining rules" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 8: Noise maps in the core

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/config.rs`, `crates/sugarscape-core/src/landscape.rs`

**Interfaces:**
- Consumes: `Map`, `Config::check_map`, `landscape::generate`.
- Produces: `Map::Noise { seed: u32, scale: f64, octaves: u32, height: f64 }` (JSON `{"kind":"noise","seed":…,"scale":…,"octaves":…,"height":…}`); `landscape` private `mix`, `corner`, `period(cells: usize, scale: f64, octave: u32) -> u64`, `smoothstep`, `noise_at(seed: u32, scale: f64, octaves: u32, (w, h): (usize, usize), (x, y): (f64, f64)) -> f64`.

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` in `crates/sugarscape-core/src/config.rs`:
```rust
    #[test]
    fn noise_maps_are_validated_on_any_grid() {
        let with = |map: Map| {
            let c = Config {
                width: 37,
                height: 11,
                population: 50,
                vision: URange::new(1, 5),
                goods: vec![Good {
                    map,
                    ..Good::sugar()
                }],
                ..Default::default()
            };
            fields(c.validate())
        };
        let noise = |scale, octaves, height| Map::Noise {
            seed: 1,
            scale,
            octaves,
            height,
        };
        assert!(with(noise(8.0, 3, 4.0)).is_empty());
        assert!(with(noise(1.0, 1, 0.0)).is_empty());
        assert!(with(noise(100.0, 6, 10.0)).is_empty());
        for bad in [0.5, 101.0, f64::NAN, f64::INFINITY] {
            assert_eq!(with(noise(bad, 3, 4.0)), vec!["goods.0.map.scale"], "{bad}");
        }
        for bad in [0, 7] {
            assert_eq!(with(noise(8.0, bad, 4.0)), vec!["goods.0.map.octaves"]);
        }
        for bad in [-1.0, 10.5, f64::NAN] {
            assert_eq!(with(noise(8.0, 3, bad)), vec!["goods.0.map.height"], "{bad}");
        }
    }
```
Append inside `mod tests` in `crates/sugarscape-core/src/landscape.rs`:
```rust
    fn noise(seed: u32, scale: f64, octaves: u32, height: f64) -> Map {
        Map::Noise {
            seed,
            scale,
            octaves,
            height,
        }
    }

    #[test]
    fn noise_maps_are_in_range_deterministic_and_seeded() {
        let caps = generate(&noise(7, 8.0, 3, 4.0), 50, 40);
        assert_eq!(caps.len(), 2000);
        assert!(caps
            .iter()
            .all(|&c| (0.0..=4.0).contains(&c) && c.fract() == 0.0));
        assert!(caps.iter().any(|&c| c != caps[0]), "not flat");
        assert_eq!(caps, generate(&noise(7, 8.0, 3, 4.0), 50, 40), "no RNG state");
        assert_ne!(caps, generate(&noise(8, 8.0, 3, 4.0), 50, 40), "the seed matters");
        assert!(generate(&noise(7, 8.0, 3, 0.0), 50, 40)
            .iter()
            .all(|&c| c == 0.0));
        assert!(generate(&noise(7, 8.0, 3, 2.5), 50, 40)
            .iter()
            .all(|&c| (0.0..=2.5).contains(&c)));
        for (w, h) in [(5, 5), (50, 50), (37, 11)] {
            for octaves in 1..=6 {
                for scale in [1.0, 3.5, 100.0] {
                    for i in 0..w * h {
                        let at = ((i % w) as f64, (i / w) as f64);
                        let v = noise_at(3, scale, octaves, (w, h), at);
                        assert!((0.0..=1.0).contains(&v), "{v} at {at:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn noise_tiles_the_torus_seamlessly() {
        let (w, h) = (50usize, 40usize);
        for (seed, scale, octaves) in [(1, 10.0, 1), (2, 8.0, 2), (3, 20.0, 3)] {
            let at = |x: usize, y: usize| noise_at(seed, scale, octaves, (w, h), (x as f64, y as f64));
            // The largest change between neighbouring cells: smoothstep's slope
            // is at most 1.5 and corner values differ by less than 1, so octave
            // o changes by at most 1.5 · period / cells per cell.
            let bound = |cells: usize| {
                let (mut sum, mut total, mut amplitude) = (0.0, 0.0, 1.0);
                for o in 0..octaves {
                    sum += amplitude * 1.5 * period(cells, scale, o) as f64 / cells as f64;
                    total += amplitude;
                    amplitude *= 0.5;
                }
                sum / total + 1e-12
            };
            let (bx, by) = (bound(w), bound(h));
            for y in 0..h {
                assert_eq!(at(w, y), at(0, y), "x = W is x = 0");
                for x in 0..w {
                    let step = (at((x + 1) % w, y) - at(x, y)).abs();
                    assert!(step <= bx, "x {x} → {} at y {y}: {step} > {bx}", (x + 1) % w);
                }
            }
            for x in 0..w {
                assert_eq!(at(x, h), at(x, 0), "y = H is y = 0");
                for y in 0..h {
                    let step = (at(x, (y + 1) % h) - at(x, y)).abs();
                    assert!(step <= by, "y {y} → {} at x {x}: {step} > {by}", (y + 1) % h);
                }
            }
        }
    }

    #[test]
    fn noise_maps_ignore_the_world_seed_and_round_trip_as_json() {
        let mut c = crate::config::Config::default();
        c.goods[0].map = noise(5, 6.0, 2, 4.0);
        let a = crate::world::World::new(c.clone(), 1).unwrap();
        let b = crate::world::World::new(c, 2).unwrap();
        assert_eq!(a.capacities(0), b.capacities(0));
        assert!(!a.landscape_edited(0));
        let json = r#"{"kind":"noise","seed":7,"scale":8.0,"octaves":3,"height":4.0}"#;
        let map: Map = serde_json::from_str(json).unwrap();
        assert_eq!(map, noise(7, 8.0, 3, 4.0));
        assert_eq!(serde_json::to_string(&map).unwrap(), json);
    }
```
Run: `cargo test -p sugarscape-core`
Expected: compile errors (`Map::Noise`, `noise_at`, `period` not found).

- [ ] **Step 2: The map kind and its validation**

In `crates/sugarscape-core/src/config.rs`, add to `enum Map` after `Flat { capacity: f64 },`:
```rust
    /// Seeded fractal value noise on the torus (Decision 10): octave 0's
    /// features are about `scale` cells across, each further octave is twice
    /// as fine at half the amplitude, and capacity is round(height · v) for
    /// v in [0, 1]. Independent of the world's seed.
    Noise {
        seed: u32,
        scale: f64,
        octaves: u32,
        height: f64,
    },
```
and add an arm to `check_map`'s `match` after the `Map::Flat` arm:
```rust
            Map::Noise {
                scale,
                octaves,
                height,
                ..
            } => {
                e.check(
                    (1.0..=100.0).contains(scale),
                    &format!("{field}.scale"),
                    "must be between 1 and 100 cells",
                );
                e.check(
                    (1..=6).contains(octaves),
                    &format!("{field}.octaves"),
                    "must be between 1 and 6",
                );
                e.check(
                    (0.0..=10.0).contains(height),
                    &format!("{field}.height"),
                    "must be between 0 and 10",
                );
            }
```

- [ ] **Step 3: Generate noise**

In `crates/sugarscape-core/src/landscape.rs`, add an arm to `generate`'s `match` after `Map::Flat`:
```rust
        Map::Noise {
            seed,
            scale,
            octaves,
            height,
        } => (0..w * h)
            .map(|i| {
                let at = ((i % w) as f64, (i / w) as f64);
                let v = noise_at(*seed, *scale, *octaves, (w, h), at);
                (*height * v).round().clamp(0.0, *height)
            })
            .collect(),
```
and add after `peak_capacity`:
```rust
/// SplitMix64's output function: a fixed integer hash (no RNG state).
fn mix(z: u64) -> u64 {
    let mut z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The value in [0, 1) at lattice corner (i, j) of octave `octave`.
fn corner(seed: u32, octave: u32, i: u64, j: u64) -> f64 {
    let h = mix(mix(mix((u64::from(seed) << 32) | u64::from(octave)) ^ i) ^ j);
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Lattice cells of octave `octave` along a side of `cells` grid cells:
/// max(1, round(cells · 2^octave / scale)) (Decision 10).
fn period(cells: usize, scale: f64, octave: u32) -> u64 {
    ((cells as f64 * f64::from(1u32 << octave) / scale).round() as u64).max(1)
}

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Fractal value noise in [0, 1] at (x, y) on a w×h torus. Cell x samples
/// lattice coordinate x · period / w, and lattice indices wrap modulo the
/// period, so x = w is exactly x = 0 and the map tiles seamlessly. Only
/// integer hashing and + − × ÷ are used: identical on every platform.
fn noise_at(seed: u32, scale: f64, octaves: u32, (w, h): (usize, usize), (x, y): (f64, f64)) -> f64 {
    let (mut sum, mut total, mut amplitude) = (0.0, 0.0, 1.0);
    for o in 0..octaves {
        let (px, py) = (period(w, scale, o), period(h, scale, o));
        let (u, v) = (x * px as f64 / w as f64, y * py as f64 / h as f64);
        let (fu, fv) = (u.floor(), v.floor());
        let (tx, ty) = (smoothstep(u - fu), smoothstep(v - fv));
        let (i0, j0) = (fu as u64 % px, fv as u64 % py);
        let (i1, j1) = ((i0 + 1) % px, (j0 + 1) % py);
        let c = |i, j| corner(seed, o, i, j);
        let top = c(i0, j0) + (c(i1, j0) - c(i0, j0)) * tx;
        let bottom = c(i0, j1) + (c(i1, j1) - c(i0, j1)) * tx;
        sum += amplitude * (top + (bottom - top) * ty);
        total += amplitude;
        amplitude *= 0.5;
    }
    sum / total
}
```
Update `generate`'s doc comment to mention noise: `/// Row-major capacities of \`map\` on a width×height torus (row 0 = north). \`TwoPeaks\` is only valid at 50×50 (enforced by \`Config::validate\`); \`Noise\` depends only on its own parameters.`

- [ ] **Step 4: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/config.rs crates/sugarscape-core/src/landscape.rs
git commit -m "Add seeded fractal-noise capacity maps" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 9: Noise in the Goods editor

*Mechanical (full code).*

**Files:**
- Modify: `web/src/types.ts`, `web/src/goods.ts`, `web/src/goods.test.ts`, `web/src/ui/goods-editor.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `Map::Noise` JSON (Task 8); `randomSeed()` from `engine.ts`.
- Produces: `GoodMap` gains `{ kind: 'noise'; seed: number; scale: number; octaves: number; height: number }`; `defaultMap(config: Config, kind: GoodMap['kind'] = 'two_peaks', seed = 1): GoodMap`.

- [ ] **Step 1: Write the failing test**

Append inside the top-level `describe` of `web/src/goods.test.ts` that holds the `defaultMap` expectations (next to `expect(defaultMap(c, 'peaks'))…`), as its own `it`:
```ts
  it('builds a noise map with the given seed, in the core key order', () => {
    const c = config();
    expect(defaultMap(c, 'noise', 42)).toEqual({ kind: 'noise', seed: 42, scale: 8, octaves: 3, height: 4 });
    expect(JSON.stringify(defaultMap(c, 'noise'))).toBe('{"kind":"noise","seed":1,"scale":8,"octaves":3,"height":4}');
  });
```
Run: `cd web && npx vitest run src/goods.test.ts`
Expected: FAIL (tsc/vitest: `'noise'` is not a map kind, or the returned map is flat).

- [ ] **Step 2: Types and the default map**

In `web/src/types.ts`, change `GoodMap` to:
```ts
export type GoodMap =
  | { kind: 'two_peaks'; transform: Transform }
  | { kind: 'peaks'; peaks: Peak[] }
  | { kind: 'flat'; capacity: number }
  | { kind: 'noise'; seed: number; scale: number; octaves: number; height: number };
```
In `web/src/goods.ts`, replace `defaultMap` with:
```ts
/**
 * A map of `kind`: the next unused two-peak transform (flat off the 50×50 grid), one central peak,
 * flat 2, or noise with `seed` (features about 8 cells across, 3 octaves, height 4).
 */
export function defaultMap(config: Config, kind: GoodMap['kind'] = 'two_peaks', seed = 1): GoodMap {
  if (kind === 'peaks') return { kind: 'peaks', peaks: [newPeak(config)] };
  if (kind === 'noise') return { kind: 'noise', seed, scale: 8, octaves: 3, height: 4 };
  if (kind === 'flat' || config.width !== 50 || config.height !== 50) return { kind: 'flat', capacity: 2 };
  const used = new Set(config.goods.flatMap((g) => (g.map.kind === 'two_peaks' ? [g.map.transform] : [])));
  const transform = TRANSFORMS.map(([t]) => t).find((t) => !used.has(t)) ?? 'identity';
  return { kind: 'two_peaks', transform };
}
```

- [ ] **Step 3: The editor**

In `web/src/ui/goods-editor.ts`: add `import { randomSeed } from '../engine';`; add `['noise', 'Noise'],` as the last entry of `KINDS`; change the kind select's change handler to
```ts
  kind.addEventListener('change', () =>
    commit((c) => (c.goods[i].map = defaultMap(c, kind.value as GoodMap['kind'], randomSeed())), true),
  );
```
and add this case to `mapDetails`' `switch` (after `case 'peaks'`):
```ts
    case 'noise': {
      const field = (key: 'scale' | 'octaves' | 'height', min: number, max: number, step: number) =>
        h('label', {}, `${key} `, num(
          syncs,
          (c) => {
            const m = c.goods[i].map;
            return m.kind === 'noise' ? m[key] : 0;
          },
          min,
          max,
          step,
          (v) =>
            setMap((m) => {
              if (m.kind === 'noise') m[key] = v;
            }),
        ));
      const seed = num(
        syncs,
        (c) => {
          const m = c.goods[i].map;
          return m.kind === 'noise' ? m.seed : 0;
        },
        0,
        4294967295,
        1,
        (v) =>
          setMap((m) => {
            if (m.kind === 'noise') m.seed = v >>> 0;
          }),
      );
      seed.classList.add('seed');
      const reroll = h('button', {
        title: 'New random seed (rebuilds the world)',
        'aria-label': 'New random noise seed',
        onclick: () =>
          setMap((m) => {
            if (m.kind === 'noise') m.seed = randomSeed();
          }),
      }, '🎲');
      return h('div', { class: 'row noise' }, h('label', {}, 'seed ', seed), reroll, field('scale', 1, 100, 0.5), field('octaves', 1, 6, 1), field('height', 0, 10, 0.5));
    }
```
Append to `web/src/style.css`:
```css
.goods .noise { flex-wrap: wrap; }
.goods .noise .num { width: 4.5em; }
.goods .noise .seed { width: 8.5em; }
```

- [ ] **Step 4: Verify**

```bash
(cd web && npm run build && npm test)
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/types.ts web/src/goods.ts web/src/goods.test.ts web/src/ui/goods-editor.ts web/src/style.css
git commit -m "Offer noise maps in the Goods editor" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 10: `set_capacities` and `set_landscape`

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/edit.rs`, `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `World::{sites, config, capacities, landscape_edited}`; WASM `Sim::good`, `edit_error`.
- Produces: `World::set_capacities(&mut self, good: usize, capacities: &[f64]) -> Result<(), String>`; WASM `Sim::set_landscape(&mut self, good: u32, capacities: &[u8]) -> Result<(), JsValue>` (JS: `set_landscape(good: number, capacities: Uint8Array): void`).

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` in `crates/sugarscape-core/src/edit.rs`:
```rust
    #[test]
    fn set_capacities_replaces_a_goods_map_and_clamps_levels() {
        let mut w = blank_world(4, 4);
        add_goods(&mut w.config, 2);
        for s in &mut w.sites {
            s.capacity[1] = 5.0;
            s.resource[1] = 5.0;
        }
        w.sites[0].resource[1] = 0.0;
        let caps: Vec<f64> = (0..16).map(|i| f64::from(i % 3)).collect();
        w.set_capacities(1, &caps).unwrap();
        assert_eq!(w.capacities(1), caps);
        let levels: Vec<f64> = w.sites.iter().map(|s| s.resource[1]).collect();
        assert_eq!(levels, caps, "levels above the new capacity are clamped");
        assert!(w.landscape_edited(1) && !w.landscape_edited(0));
        let err = |r: Result<(), String>| r.unwrap_err();
        assert_eq!(err(w.set_capacities(2, &caps)), "there is no good 2");
        assert_eq!(
            err(w.set_capacities(1, &caps[..15])),
            "expected 16 capacities, got 15"
        );
        for bad in [10.5, -1.0, f64::NAN, f64::INFINITY] {
            let e = err(w.set_capacities(1, &[bad; 16]));
            assert!(e.starts_with("capacities must be between 0 and 10"), "{e}");
        }
        assert_eq!(w.capacities(1), caps, "a rejected call changes nothing");
    }
```
Append to `crates/sugarscape-wasm/tests/web.rs`:
```rust
#[wasm_bindgen_test]
fn set_landscape_replaces_a_goods_capacities() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    let caps: Vec<u8> = (0..2500).map(|i| (i % 11) as u8).collect();
    sim.set_landscape(0, &caps).unwrap();
    assert!(sim.landscape_edited(0));
    assert_eq!(sim.export_landscape(0).unwrap(), caps);
    assert!(sim.set_landscape(0, &caps[..10]).is_err());
    assert!(sim.set_landscape(0, &[11; 2500]).is_err());
    assert!(sim.set_landscape(1, &caps).is_err());
}
```
Run: `cargo test -p sugarscape-core edit::`
Expected: compile error (`set_capacities` not found).

- [ ] **Step 2: Implement**

In `crates/sugarscape-core/src/edit.rs`, add after `paint_capacity`:
```rust
    /// Replaces good `good`'s capacities with `capacities` (row-major, one per
    /// site, each 0–10 — an imported image), clamping each site's level to its
    /// new capacity as painting does. The good then counts as edited
    /// (`landscape_edited`), so share links, export and reset keep it.
    pub fn set_capacities(&mut self, good: usize, capacities: &[f64]) -> Result<(), String> {
        if good >= self.config.goods.len() {
            return Err(format!("there is no good {good}"));
        }
        if capacities.len() != self.sites.len() {
            return Err(format!(
                "expected {} capacities, got {}",
                self.sites.len(),
                capacities.len()
            ));
        }
        if let Some(bad) = capacities.iter().find(|c| !(0.0..=10.0).contains(*c)) {
            return Err(format!("capacities must be between 0 and 10 (got {bad})"));
        }
        for (site, &c) in self.sites.iter_mut().zip(capacities) {
            site.capacity[good] = c;
            site.resource[good] = site.resource[good].min(c);
        }
        Ok(())
    }
```
In `crates/sugarscape-wasm/src/lib.rs`, add after `paint_capacity`:
```rust
    /// Replaces good `good`'s capacities with `capacities` (row-major bytes,
    /// each 0–10): an imported image. The good then counts as painted.
    pub fn set_landscape(&mut self, good: u32, capacities: &[u8]) -> Result<(), JsValue> {
        let g = self.good(good)?;
        let caps: Vec<f64> = capacities.iter().map(|&c| f64::from(c)).collect();
        self.world.set_capacities(g, &caps).map_err(edit_error)
    }
```

- [ ] **Step 3: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
```
Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add crates/sugarscape-core/src/edit.rs crates/sugarscape-wasm/src/lib.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Set a good's whole capacity map from the core and WASM" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 11: Image import in the paint tool

*Mechanical (full code).*

**Files:**
- Create: `web/src/image.ts`, `web/src/image.test.ts`, `web/src/ui/image-import.ts`
- Modify: `web/src/engine.ts`, `web/src/ui/tools.ts`

**Interfaces:**
- Consumes: `Sim.set_landscape`, `Sim.export_landscape` (Task 10); the paint tool's selected `good`.
- Produces: `capacitiesFromPixels(rgba: ArrayLike<number>, max: number, invert: boolean): Uint8Array`; `readImagePixels(file: Blob, width: number, height: number): Promise<Uint8ClampedArray>`; `Engine.importLandscape(good: number, capacities: Uint8Array): FieldError[] | null`; private `Engine.keepLandscape(good: number): void`.

- [ ] **Step 1: Write the failing test**

Create `web/src/image.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { capacitiesFromPixels } from './image';

const px = (...rgba: number[][]) => new Uint8ClampedArray(rgba.flat());
const caps = (pixels: Uint8ClampedArray, max: number, invert = false) => Array.from(capacitiesFromPixels(pixels, max, invert));

describe('capacitiesFromPixels', () => {
  it('maps luminance onto 0..max', () => {
    expect(caps(px([255, 255, 255, 255], [0, 0, 0, 255], [128, 128, 128, 255]), 4)).toEqual([4, 0, 2]);
  });

  it('weights the channels by Rec. 709 luma', () => {
    // Y = 182.4, 54.2 and 18.4: 7.15, 2.13 and 0.72 of 10.
    expect(caps(px([0, 255, 0, 255], [255, 0, 0, 255], [0, 0, 255, 255]), 10)).toEqual([7, 2, 1]);
  });

  it('inverts', () => {
    expect(caps(px([255, 255, 255, 255], [0, 0, 0, 255]), 4, true)).toEqual([0, 4]);
  });

  it('counts mostly transparent pixels as black', () => {
    expect(caps(px([255, 255, 255, 127], [255, 255, 255, 128]), 4)).toEqual([0, 4]);
    expect(caps(px([255, 255, 255, 127]), 4, true)).toEqual([4]);
  });

  it('rounds to the nearest unit', () => {
    // 96/255 · 4 = 1.506 and 95/255 · 4 = 1.490.
    expect(caps(px([96, 96, 96, 255], [95, 95, 95, 255]), 4)).toEqual([2, 1]);
    expect(caps(px([255, 255, 255, 255]), 0)).toEqual([0]);
  });
});
```
Run: `cd web && npx vitest run src/image.test.ts`
Expected: FAIL (`./image` does not exist).

- [ ] **Step 2: The conversion**

Create `web/src/image.ts`:
```ts
/**
 * Capacities from RGBA pixels (row-major, 4 bytes each): luminance
 * Y = 0.2126 R + 0.7152 G + 0.0722 B (a pixel with alpha < 128 counts as 0),
 * v = Y / 255 (or 1 − v inverted), capacity round(v · max).
 */
export function capacitiesFromPixels(rgba: ArrayLike<number>, max: number, invert: boolean): Uint8Array {
  const n = Math.floor(rgba.length / 4);
  const out = new Uint8Array(n);
  for (let i = 0; i < n; i++) {
    const [r, g, b, a] = [rgba[4 * i], rgba[4 * i + 1], rgba[4 * i + 2], rgba[4 * i + 3]];
    const y = a < 128 ? 0 : 0.2126 * r + 0.7152 * g + 0.0722 * b;
    const v = invert ? 1 - y / 255 : y / 255;
    out[i] = Math.round(v * max);
  }
  return out;
}
```
Create `web/src/ui/image-import.ts`:
```ts
/** An image file's pixels, drawn onto a width × height canvas with high-quality smoothing. */
export async function readImagePixels(file: Blob, width: number, height: number): Promise<Uint8ClampedArray> {
  const bitmap = await createImageBitmap(file);
  try {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d')!;
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(bitmap, 0, 0, width, height);
    return ctx.getImageData(0, 0, width, height).data;
  } finally {
    bitmap.close();
  }
}
```

- [ ] **Step 3: The engine**

In `web/src/engine.ts`, replace `paint` with:
```ts
  /** Keeps good `good`'s current capacities as its custom map (share links, export, reset). */
  private keepLandscape(good: number): void {
    const next = [...this.customLandscapes];
    while (next.length <= good) next.push(null);
    next[good] = this.sim.export_landscape(good);
    this.customLandscapes = next;
  }

  paint(x: number, y: number, radius: number, value: number, good = 0): FieldError[] | null {
    return this.edit(() => {
      this.sim.paint_capacity(x, y, radius, value, good);
      this.keepLandscape(good);
    });
  }

  /** Replaces good `good`'s capacity map (one byte per site, 0–10). */
  importLandscape(good: number, capacities: Uint8Array): FieldError[] | null {
    return this.edit(() => {
      this.sim.set_landscape(good, capacities);
      this.keepLandscape(good);
    });
  }
```

- [ ] **Step 4: The paint tool**

In `web/src/ui/tools.ts`, add `import { capacitiesFromPixels } from '../image';` and `import { readImagePixels } from './image-import';`. After the `goodLabel`/`refreshGoods` block, add:
```ts
  /** Image import (Decision 12): for the paint tool's good, max capacity 0–10, optionally inverted. */
  let importMax = 4;
  let invert = false;
  const importStatus = h('span', { class: 'hint', 'aria-live': 'polite' });
  const fileInput = h('input', { type: 'file', accept: 'image/*', hidden: true });
  fileInput.addEventListener('change', async () => {
    const file = fileInput.files?.[0];
    fileInput.value = '';
    if (!file) return;
    try {
      const { width, height } = engine.size();
      const capacities = capacitiesFromPixels(await readImagePixels(file, width, height), importMax, invert);
      const errors = engine.importLandscape(good, capacities);
      importStatus.textContent = errors ? errors.map((e) => e.message).join('; ') : `Imported ${file.name}`;
    } catch (e) {
      importStatus.textContent = `Could not read ${file.name}: ${e instanceof Error ? e.message : String(e)}`;
    }
  });
  const invertBox = h('input', { type: 'checkbox', onchange: () => (invert = invertBox.checked) });
  const importControls = h(
    'span',
    { class: 'tool-options' },
    h('button', { onclick: () => fileInput.click(), title: 'Set the good’s capacities from an image’s brightness' }, 'Import image…'),
    fileInput,
    number('Max', 0, 10, () => importMax, (v) => (importMax = Math.round(v))),
    h('label', {}, invertBox, ' Invert'),
    importStatus,
  );
```
and in `choose`, change the paint branch to
```ts
        ? [goodLabel, number('Radius', 0, 10, () => radius, (v) => (radius = v)), number('Capacity', 0, 10, () => value, (v) => (value = v)), importControls]
```
(`number` is declared above `goodLabel`, so it is in scope.)

- [ ] **Step 5: Verify**

```bash
(cd web && npm run build && npm test)
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/image.ts web/src/image.test.ts web/src/ui/image-import.ts web/src/engine.ts web/src/ui/tools.ts
git commit -m "Import an image as a good's capacity map" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 12: Agent trails in the core and WASM

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/world.rs`, `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `World::step`, `Pos`, `World::remove_agent` (tests).
- Produces: `pub const TRAIL_LEN: usize = 500;` in `world`; `World::follow(&mut self, id: Option<AgentId>)`, `World::followed(&self) -> Option<AgentId>`, `World::trail(&self) -> &[Pos]`; private `record_trail`; WASM `Sim::follow(&mut self, id: f64)`, `Sim::unfollow(&mut self)`, `Sim::trail(&self) -> Vec<u32>` (JS `Uint32Array`), `Sim::followed(&self) -> f64`.

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` in `crates/sugarscape-core/src/world.rs`:
```rust
    #[test]
    fn a_followed_agent_leaves_a_capped_trail_that_is_not_hashed() {
        let mut w = crate::testkit::blank_world(10, 10);
        let id = crate::testkit::spawn(&mut w, 2, 3);
        let mut twin = crate::testkit::blank_world(10, 10);
        crate::testkit::spawn(&mut twin, 2, 3);
        assert_eq!((w.followed(), w.trail().len()), (None, 0));
        w.follow(Some(id));
        assert_eq!(w.followed(), Some(id));
        assert_eq!(w.trail(), &[Pos::new(2, 3)], "the current position at once");
        w.run(3);
        twin.run(3);
        assert_eq!(w.trail().len(), 4);
        assert_eq!(*w.trail().last().unwrap(), w.agent(id).unwrap().pos);
        assert_eq!(w.fingerprint(), twin.fingerprint(), "trails are not simulation state");
        w.run(TRAIL_LEN as u32);
        assert_eq!(w.trail().len(), TRAIL_LEN, "the oldest positions are dropped");
        assert_eq!(*w.trail().last().unwrap(), w.agent(id).unwrap().pos);
        // Death: the trail stops growing and stays until `follow` is called.
        let before = w.trail().to_vec();
        let pos = w.agent(id).unwrap().pos;
        w.remove_agent(pos.x, pos.y).unwrap();
        w.run(2);
        assert_eq!(w.trail(), &before[..]);
        assert_eq!(w.followed(), Some(id));
        w.follow(None);
        assert_eq!((w.followed(), w.trail().len()), (None, 0));
        w.follow(Some(999));
        assert!(w.trail().is_empty(), "nobody alive to record");
    }
```
Append to `crates/sugarscape-wasm/tests/web.rs`:
```rust
#[wasm_bindgen_test]
fn following_an_agent_records_its_trail() {
    let mut sim = Sim::new("{}", 1, JsValue::NULL).unwrap();
    assert_eq!(sim.followed(), -1.0);
    assert!(sim.trail().is_empty());
    let config = sim.export_config();
    sim.follow(1.0);
    assert_eq!(sim.followed(), 1.0);
    assert_eq!(sim.trail(), sim.locate(1.0).unwrap());
    for _ in 0..3 {
        sim.step(1);
        if let Some(at) = sim.locate(1.0) {
            let trail = sim.trail();
            assert_eq!(&trail[trail.len() - 2..], &at[..], "newest last");
        }
    }
    assert!(sim.trail().len() >= 2 && sim.trail().len() % 2 == 0);
    assert_eq!(sim.export_config(), config, "trails are not config");
    sim.unfollow();
    assert_eq!(sim.followed(), -1.0);
    assert!(sim.trail().is_empty());
}
```
Run: `cargo test -p sugarscape-core world::`
Expected: compile errors (`follow`, `TRAIL_LEN` not found).

- [ ] **Step 2: Implement in the core**

In `crates/sugarscape-core/src/world.rs`, add above `pub struct World`:
```rust
/// Most positions a trail keeps; the oldest are dropped first.
pub const TRAIL_LEN: usize = 500;
```
add two fields at the end of `struct World`:
```rust
    /// The agent whose trail is recorded (observation only: never hashed,
    /// exported or put in configs).
    followed: Option<AgentId>,
    /// Its positions after each tick, oldest first.
    trail: Vec<Pos>,
```
initialize them in `with_landscapes`' `World { … }` literal with `followed: None, trail: Vec::new(),`; add to `impl World` (after `run`):
```rust
    /// Follows agent `id` from now on (a fresh trail that records its current
    /// position at once, then its position at the end of every tick), or
    /// stops following with `None` (the trail is cleared). A followed agent
    /// that dies leaves its trail as it was.
    pub fn follow(&mut self, id: Option<AgentId>) {
        self.followed = id;
        self.trail.clear();
        self.record_trail();
    }

    pub fn followed(&self) -> Option<AgentId> {
        self.followed
    }

    /// The followed agent's positions, oldest first (at most `TRAIL_LEN`).
    pub fn trail(&self) -> &[Pos] {
        &self.trail
    }

    fn record_trail(&mut self) {
        let Some(pos) = self
            .followed
            .and_then(|id| self.agents.get(&id))
            .map(|a| a.pos)
        else {
            return;
        };
        if self.trail.len() == TRAIL_LEN {
            self.trail.remove(0);
        }
        self.trail.push(pos);
    }
```
and at the end of `step`, after `self.stats.push(snapshot);`, add:
```rust
        self.record_trail();
```

- [ ] **Step 3: Implement in WASM**

In `crates/sugarscape-wasm/src/lib.rs`, add to `impl Sim` (after `locate`):
```rust
    /// Records agent `id`'s trail from now on (`World::follow`).
    pub fn follow(&mut self, id: f64) {
        self.world.follow(Some(id as u64));
    }

    pub fn unfollow(&mut self) {
        self.world.follow(None);
    }

    /// The followed agent's trail as `[x0, y0, x1, y1, …]`, oldest first.
    pub fn trail(&self) -> Vec<u32> {
        self.world.trail().iter().flat_map(|p| [p.x, p.y]).collect()
    }

    /// The followed agent's id, or −1 when none is followed.
    pub fn followed(&self) -> f64 {
        self.world.followed().map_or(-1.0, |id| id as f64)
    }
```

- [ ] **Step 4: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
```
Expected: all pass (golden unchanged: the trail is outside `fingerprint`).

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/world.rs crates/sugarscape-wasm/src/lib.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Record a followed agent's trail outside the simulation state" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 13: Following an agent in the browser

*Mechanical (full code).*

**Files:**
- Create: `web/src/ui/trail.ts`, `web/src/ui/trail.test.ts`
- Modify: `web/src/engine.ts`, `web/src/ui/inspect-panel.ts`, `web/src/ui/toolbar.ts`, `web/src/ui/grid-view.ts`, `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `Sim.{follow, unfollow, trail, followed, locate}` (Task 12).
- Produces: `interface TrailSegment { x1: number; y1: number; x2: number; y2: number; alpha: number }`, `trailSegments(trail: ArrayLike<number>, width: number, height: number): TrailSegment[]`; `EngineEvent` gains `'follow'`; `Engine.selectAgent(id: number): void` (renamed from `follow`), `Engine.followAgent(id: number): void`, `Engine.unfollow(): void`, `Engine.followed(): number | null`, `Engine.trail(): Uint32Array`.

- [ ] **Step 1: Write the failing test**

Create `web/src/ui/trail.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { trailSegments } from './trail';

describe('trailSegments', () => {
  it('joins consecutive cells, fading in from the oldest', () => {
    expect(trailSegments(new Uint32Array([1, 1, 2, 1, 2, 2]), 10, 10)).toEqual([
      { x1: 1, y1: 1, x2: 2, y2: 1, alpha: 0.5 },
      { x1: 2, y1: 1, x2: 2, y2: 2, alpha: 1 },
    ]);
  });

  it('breaks the line where the agent wraps around the torus', () => {
    const across = trailSegments(new Uint32Array([8, 5, 9, 5, 0, 5, 1, 5]), 10, 10);
    expect(across.map((s) => [s.x1, s.x2])).toEqual([[8, 9], [0, 1]]);
    expect(across.map((s) => s.alpha)).toEqual([1 / 3, 1]);
    expect(trailSegments(new Uint32Array([3, 0, 3, 9]), 10, 10)).toEqual([]);
  });

  it('keeps a step of exactly half the grid', () => {
    expect(trailSegments(new Uint32Array([0, 0, 5, 0]), 10, 10)).toHaveLength(1);
  });

  it('needs two positions', () => {
    expect(trailSegments(new Uint32Array([4, 4]), 10, 10)).toEqual([]);
    expect(trailSegments(new Uint32Array([]), 10, 10)).toEqual([]);
  });
});
```
Run: `cd web && npx vitest run src/ui/trail.test.ts`
Expected: FAIL (`./trail` does not exist).

- [ ] **Step 2: Trail segments**

Create `web/src/ui/trail.ts`:
```ts
export interface TrailSegment { x1: number; y1: number; x2: number; y2: number; alpha: number }

/**
 * Segments between consecutive trail cells (`[x0, y0, x1, y1, …]`, oldest first). A step of more than
 * half the grid in x or y is a wrap around the torus and is left out. Segment i has alpha (i + 1)/(n − 1),
 * so the line fades in from the oldest position to the newest.
 */
export function trailSegments(trail: ArrayLike<number>, width: number, height: number): TrailSegment[] {
  const n = Math.floor(trail.length / 2);
  const out: TrailSegment[] = [];
  for (let i = 0; i + 1 < n; i++) {
    const [x1, y1, x2, y2] = [trail[2 * i], trail[2 * i + 1], trail[2 * i + 2], trail[2 * i + 3]];
    if (Math.abs(x2 - x1) > width / 2 || Math.abs(y2 - y1) > height / 2) continue;
    out.push({ x1, y1, x2, y2, alpha: (i + 1) / (n - 1) });
  }
  return out;
}
```

- [ ] **Step 3: The engine**

In `web/src/engine.ts`, change the event type to
```ts
export type EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit' | 'follow';
```
rename the method `follow(id: number)` to `selectAgent(id: number)` (body unchanged; doc comment `/** Selects agent \`id\` if it is alive. */`), and add after it:
```ts
  /** Records and draws agent `id`'s trail from now on (a new trail replaces any other; reset clears it). */
  followAgent(id: number): void {
    this.sim.follow(id);
    this.emit('follow');
  }

  unfollow(): void {
    this.sim.unfollow();
    this.emit('follow');
  }

  /** The followed agent's id (alive or not), or null. */
  followed(): number | null {
    const id = this.sim.followed();
    return id < 0 ? null : id;
  }

  /** The followed agent's trail as `[x0, y0, x1, y1, …]`, oldest first. */
  trail(): Uint32Array {
    return this.sim.trail();
  }
```

- [ ] **Step 4: Inspect, toolbar and grid**

In `web/src/ui/inspect-panel.ts`: change both `this.engine.follow(` calls to `this.engine.selectAgent(`; change the constructor's event list to `['select', 'tick', 'reset', 'config', 'edit', 'follow'] as const`; add the method
```ts
  private followButton(id: number): HTMLElement {
    const following = this.engine.followed() === id;
    return h('button', {
      class: 'link',
      disabled: following,
      title: 'Draw this agent’s trail on the grid',
      onclick: () => this.engine.followAgent(id),
    }, following ? 'Following' : 'Follow');
  }
```
and replace the `row('Agent', …)` line in `agentRows` with
```ts
      row('Agent', h('span', {}, `#${a.id} · ${a.sex} · ${this.engine.config.culture.groups[a.group]?.name ?? a.tribe} `, this.followButton(a.id))),
```
In `web/src/ui/toolbar.ts`, after `const readout = …;` add:
```ts
  /** "Following #id ✕" while an agent's trail is drawn; † once it has died. */
  const chip = h('span', { class: 'chip' });
  const syncFollow = () => {
    const id = engine.followed();
    chip.hidden = id === null;
    if (id === null) return;
    const alive = engine.sim.locate(id) !== undefined;
    chip.replaceChildren(
      `Following #${id}${alive ? '' : ' †'}`,
      h('button', { class: 'link', title: 'Stop following', 'aria-label': 'Stop following', onclick: () => engine.unfollow() }, '✕'),
    );
  };
  for (const event of ['follow', 'reset', 'tick', 'edit'] as const) engine.on(event, syncFollow);
  syncFollow();
```
and in the returned toolbar put `chip` right after `readout`:
```ts
    readout,
    chip,
    h('div', { class: 'toolbar-end' }),
```
In `web/src/ui/grid-view.ts`, add `import { trailSegments } from './trail';` and insert after the overlays `for` loop (before `const accent = …`):
```ts
    if (this.engine.followed() !== null) {
      ctx.save();
      ctx.strokeStyle = getComputedStyle(this.canvas).getPropertyValue('--text').trim() || '#000';
      ctx.lineWidth = 2;
      ctx.lineCap = 'round';
      for (const s of trailSegments(this.engine.trail(), width, height)) {
        ctx.globalAlpha = s.alpha;
        ctx.beginPath();
        ctx.moveTo((s.x1 + 0.5) * CELL, (s.y1 + 0.5) * CELL);
        ctx.lineTo((s.x2 + 0.5) * CELL, (s.y2 + 0.5) * CELL);
        ctx.stroke();
      }
      ctx.restore();
    }
```
In `web/src/main.ts`, change the redraw event list to `['reset', 'tick', 'config', 'display', 'select', 'edit', 'follow'] as const`.

Append to `web/src/style.css`:
```css
.chip { display: inline-flex; gap: 4px; align-items: center; border: 1px solid var(--border); border-radius: 99px; padding: 1px 4px 1px 10px; font-size: 12px; color: var(--muted); }
body[data-view='experiments'] .toolbar .chip { display: none; }
```

- [ ] **Step 5: Verify**

```bash
(cd web && npm run build && npm test)
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/ui/trail.ts web/src/ui/trail.test.ts web/src/engine.ts web/src/ui/inspect-panel.ts web/src/ui/toolbar.ts web/src/ui/grid-view.ts web/src/main.ts web/src/style.css
git commit -m "Follow an agent and draw its trail on the grid" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 14: The credit graph in the core and WASM

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/network.rs`, `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `network::credit_roles`, `World::loans`, testkit `spawn`, `World::originate_loan`.
- Produces: `pub struct CreditNode { pub id: AgentId, pub role: &'static str }`, `pub struct CreditLink { pub lender: AgentId, pub borrower: AgentId, pub good: usize, pub due: f64 }`, `pub struct CreditGraph { pub agents: Vec<CreditNode>, pub loans: Vec<CreditLink> }` (all `Clone, Debug, PartialEq, Serialize`), `pub fn credit_graph(world: &World) -> CreditGraph`; WASM `Sim::credit_graph(&self) -> String`.

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` in `crates/sugarscape-core/src/network.rs`:
```rust
    #[test]
    fn the_credit_graph_lists_loans_and_the_agents_in_them() {
        let mut w = blank_world(5, 5);
        let a = spawn(&mut w, 0, 0);
        let b = spawn(&mut w, 1, 0);
        let c = spawn(&mut w, 2, 0);
        spawn(&mut w, 3, 0); // no loans: left out
        assert_eq!(credit_graph(&w), CreditGraph::default());
        w.originate_loan(b, c, 0, 2.0);
        w.originate_loan(a, b, 0, 1.0);
        let g = credit_graph(&w);
        let roles: Vec<(AgentId, &str)> = g.agents.iter().map(|n| (n.id, n.role)).collect();
        assert_eq!(roles, [(a, "lender"), (b, "both"), (c, "borrower")]);
        let pairs: Vec<(AgentId, AgentId)> = g.loans.iter().map(|l| (l.lender, l.borrower)).collect();
        assert_eq!(pairs, [(b, c), (a, b)], "loan order");
        assert_eq!(g.loans[0].good, 0);
        assert!(g.loans[0].due > 2.0, "due includes interest");
        let json = serde_json::to_value(&g).unwrap();
        assert_eq!(json["agents"][1]["role"], "both");
        assert!(json["loans"][0]["due"].is_number());
    }
```
Append to `crates/sugarscape-wasm/tests/web.rs`:
```rust
#[wasm_bindgen_test]
fn credit_graph_lists_the_outstanding_loans() {
    let config = serde_json::to_string(
        &sugarscape_core::presets::by_id("iv-5-credit")
            .unwrap()
            .config,
    )
    .unwrap();
    let mut sim = Sim::new(&config, 1, JsValue::NULL).unwrap();
    assert_eq!(sim.credit_graph(), r#"{"agents":[],"loans":[]}"#);
    // Measured natively: seed 1 has 9 loans outstanding at t = 20 and 15 at t = 50.
    sim.step(50);
    let graph: serde_json::Value = serde_json::from_str(&sim.credit_graph()).unwrap();
    let loans = graph["loans"].as_array().unwrap();
    assert!(!loans.is_empty());
    let agents = graph["agents"].as_array().unwrap();
    let ids: Vec<u64> = agents.iter().map(|a| a["id"].as_u64().unwrap()).collect();
    for l in loans {
        assert!(ids.contains(&l["lender"].as_u64().unwrap()));
        assert!(ids.contains(&l["borrower"].as_u64().unwrap()));
    }
    assert!(agents
        .iter()
        .all(|a| ["lender", "borrower", "both"].contains(&a["role"].as_str().unwrap())));
}
```
Run: `cargo test -p sugarscape-core network::`
Expected: compile error (`credit_graph`, `CreditGraph` not found).

- [ ] **Step 2: Implement**

In `crates/sugarscape-core/src/network.rs`, add `use serde::Serialize;` to the imports and add after `credit_roles`:
```rust
/// An agent taking part in an outstanding loan, with its role.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CreditNode {
    pub id: AgentId,
    /// `"lender"`, `"borrower"` or `"both"`.
    pub role: &'static str,
}

/// An outstanding loan: `due` of good `good` owed by `borrower` to `lender`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CreditLink {
    pub lender: AgentId,
    pub borrower: AgentId,
    pub good: usize,
    pub due: f64,
}

/// The lender → borrower graph of Animation IV-5 (Decision 15).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct CreditGraph {
    /// Agents in some outstanding loan, by id.
    pub agents: Vec<CreditNode>,
    /// Outstanding loans, in loan order.
    pub loans: Vec<CreditLink>,
}

/// The outstanding loans and the agents taking part in them.
pub fn credit_graph(world: &World) -> CreditGraph {
    let agents = credit_roles(world)
        .into_iter()
        .filter_map(|(id, role)| {
            let role = match role {
                CreditRole::Lender => "lender",
                CreditRole::Borrower => "borrower",
                CreditRole::Both => "both",
                CreditRole::None => return None,
            };
            Some(CreditNode { id, role })
        })
        .collect();
    let loans = world
        .loans()
        .map(|l| CreditLink {
            lender: l.lender,
            borrower: l.borrower,
            good: l.good,
            due: l.due,
        })
        .collect();
    CreditGraph { agents, loans }
}
```
In `crates/sugarscape-wasm/src/lib.rs`, add to `impl Sim` (after `networks`):
```rust
    /// JSON `{ agents: [{ id, role }], loans: [{ lender, borrower, good, due }] }`
    /// over the outstanding loans.
    pub fn credit_graph(&self) -> String {
        serde_json::to_string(&network::credit_graph(&self.world)).expect("graph serializes")
    }
```

- [ ] **Step 3: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
wasm-pack test --node crates/sugarscape-wasm
```
Expected: all pass. If the WASM test's `loans` is empty at t = 50, report the tick at which `sim.credit_graph()` first lists a loan (do not delete the assertion).

- [ ] **Step 4: Commit**

```bash
git add crates/sugarscape-core/src/network.rs crates/sugarscape-wasm/src/lib.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Expose the lender-borrower graph of outstanding loans" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 15: The Credit tab

*Mechanical (full code).*

**Files:**
- Create: `web/src/credit.ts`, `web/src/credit.test.ts`, `web/src/ui/credit-panel.ts`
- Modify: `web/src/ui/tabs.ts`, `web/src/engine.ts`, `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: `Sim.credit_graph()` (Task 14); `Engine.selectAgent` (Task 13).
- Produces: `credit.ts`: `type CreditRole`, `interface CreditLoan`, `interface CreditGraph`, `MAX_DRAWN_LOANS = 400`, `drawnLoans(loans: CreditLoan[], max?: number): CreditLoan[]`, `creditLevels(ids: number[], loans: { lender: number; borrower: number }[]): Map<number, number>`, `interface CreditLayout`, `creditLayout(graph: CreditGraph, max?: number): CreditLayout`, `creditHeader(layout: CreditLayout): string`; `Engine.creditGraph(): CreditGraph`; `Tabs.setHidden(label: string, hidden: boolean): void`; `class CreditPanel { el; constructor(engine: Engine, onSelect: () => void); setVisible(visible: boolean); maybeRefresh(now: number) }`.

- [ ] **Step 1: Write the failing test**

Create `web/src/credit.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { creditHeader, creditLayout, creditLevels, drawnLoans, type CreditGraph, type CreditLoan } from './credit';

const loan = (lender: number, borrower: number, due = 1): CreditLoan => ({ lender, borrower, good: 0, due });
const idsOf = (loans: CreditLoan[]) => [...new Set(loans.flatMap((l) => [l.lender, l.borrower]))].sort((a, b) => a - b);
const levels = (loans: CreditLoan[]) => Object.fromEntries(creditLevels(idsOf(loans), loans));
const graph = (loans: CreditLoan[]): CreditGraph => {
  const lenders = new Set(loans.map((l) => l.lender));
  const borrowers = new Set(loans.map((l) => l.borrower));
  return {
    agents: idsOf(loans).map((id): CreditGraph['agents'][number] => ({
      id,
      role: lenders.has(id) ? (borrowers.has(id) ? 'both' : 'lender') : 'borrower',
    })),
    loans,
  };
};

describe('creditLevels', () => {
  it('puts a chain one level per link', () => {
    expect(levels([loan(1, 2), loan(2, 3)])).toEqual({ 1: 0, 2: 1, 3: 2 });
  });

  it('takes the longest path through a diamond', () => {
    expect(levels([loan(1, 2), loan(1, 3), loan(2, 4), loan(3, 4), loan(1, 4)])).toEqual({ 1: 0, 2: 1, 3: 1, 4: 2 });
  });

  it('cuts the loans that close a cycle', () => {
    expect(levels([loan(1, 2), loan(2, 1)])).toEqual({ 1: 0, 2: 0 });
    expect(levels([loan(5, 1), loan(1, 2), loan(2, 1)])).toEqual({ 1: 1, 2: 1, 5: 0 });
    expect(levels([loan(1, 2), loan(2, 3), loan(3, 1), loan(3, 4)])).toEqual({ 1: 0, 2: 0, 3: 0, 4: 1 });
  });
});

describe('creditLayout', () => {
  it('lays out one row per level, in id order', () => {
    const layout = creditLayout(graph([loan(9, 4), loan(9, 2), loan(4, 7)]));
    expect(layout.rows).toEqual([[9], [2, 4], [7]]);
    expect(layout.roles.get(4)).toBe('both');
    expect(creditHeader(layout)).toBe('4 agents · 3 loans · 3 levels');
  });

  it('draws only the largest loans of a big graph', () => {
    const layout = creditLayout(graph(Array.from({ length: 401 }, (_, i) => loan(1000 + i, 2000 + i, i))));
    expect(layout.loans).toHaveLength(400);
    expect(layout.loans.some((l) => l.due === 0)).toBe(false);
    expect(layout.omitted).toBe(1);
    expect(creditHeader(layout)).toBe('802 agents · 401 loans · 2 levels · the 400 largest loans drawn (1 omitted)');
    expect(drawnLoans([loan(1, 2, 5), loan(3, 4, 5), loan(5, 6, 1)], 2)).toEqual([loan(1, 2, 5), loan(3, 4, 5)]);
  });

  it('says when there is nothing to draw', () => {
    const empty = creditLayout({ agents: [], loans: [] });
    expect(empty.rows).toEqual([]);
    expect(creditHeader(empty)).toBe('No outstanding loans.');
  });
});
```
Run: `cd web && npx vitest run src/credit.test.ts`
Expected: FAIL (`./credit` does not exist).

- [ ] **Step 2: Levels and layout**

Create `web/src/credit.ts`:
```ts
export type CreditRole = 'lender' | 'borrower' | 'both';
export interface CreditLoan { lender: number; borrower: number; good: number; due: number }
/** `Sim.credit_graph()`: agents in some outstanding loan (by id) and the loans (in loan order). */
export interface CreditGraph { agents: { id: number; role: CreditRole }[]; loans: CreditLoan[] }

export const MAX_DRAWN_LOANS = 400;

/** The `max` loans with the largest `due` (ties: the earlier loan), in their original order (Decision 17). */
export function drawnLoans(loans: CreditLoan[], max = MAX_DRAWN_LOANS): CreditLoan[] {
  if (loans.length <= max) return loans;
  const keep = new Set(
    loans
      .map((l, i) => ({ due: l.due, i }))
      .sort((a, b) => b.due - a.due || a.i - b.i)
      .slice(0, max)
      .map(({ i }) => i),
  );
  return loans.filter((_, i) => keep.has(i));
}

/**
 * Each agent's level (Decision 16): 0 for pure lenders, otherwise 1 + the highest level among its lenders,
 * computed on strongly connected components so every loan that closes a cycle is ignored — members of a
 * cycle with no lender above it get level 0.
 */
export function creditLevels(ids: number[], loans: { lender: number; borrower: number }[]): Map<number, number> {
  const borrowersOf = new Map<number, number[]>(ids.map((id) => [id, []]));
  for (const l of loans) borrowersOf.get(l.lender)?.push(l.borrower);
  // Tarjan's strongly connected components.
  const index = new Map<number, number>();
  const low = new Map<number, number>();
  const component = new Map<number, number>();
  const stack: number[] = [];
  const onStack = new Set<number>();
  let next = 0;
  let components = 0;
  const strong = (v: number): void => {
    index.set(v, next);
    low.set(v, next);
    next++;
    stack.push(v);
    onStack.add(v);
    for (const w of borrowersOf.get(v) ?? []) {
      if (!index.has(w)) {
        strong(w);
        low.set(v, Math.min(low.get(v)!, low.get(w)!));
      } else if (onStack.has(w)) {
        low.set(v, Math.min(low.get(v)!, index.get(w)!));
      }
    }
    if (low.get(v) === index.get(v)) {
      let w: number;
      do {
        w = stack.pop()!;
        onStack.delete(w);
        component.set(w, components);
      } while (w !== v);
      components++;
    }
  };
  for (const id of ids) if (!index.has(id)) strong(id);
  // Components lending into each component (loans inside a component are cut).
  const lendersOf = new Map<number, Set<number>>();
  for (const l of loans) {
    const from = component.get(l.lender);
    const to = component.get(l.borrower);
    if (from === undefined || to === undefined || from === to) continue;
    let set = lendersOf.get(to);
    if (!set) lendersOf.set(to, (set = new Set()));
    set.add(from);
  }
  const level = new Map<number, number>();
  const levelOf = (c: number): number => {
    const known = level.get(c);
    if (known !== undefined) return known;
    let v = 0;
    for (const from of lendersOf.get(c) ?? []) v = Math.max(v, levelOf(from) + 1);
    level.set(c, v);
    return v;
  };
  return new Map(ids.map((id) => [id, levelOf(component.get(id)!)]));
}

export interface CreditLayout {
  /** Agent ids per level (row 0 = pure lenders), each row in id order. */
  rows: number[][];
  /** The loans drawn. */
  loans: CreditLoan[];
  roles: Map<number, CreditRole>;
  /** The whole graph's agents and loans, and the loans left out. */
  agents: number;
  totalLoans: number;
  omitted: number;
}

export function creditLayout(graph: CreditGraph, max = MAX_DRAWN_LOANS): CreditLayout {
  const loans = drawnLoans(graph.loans, max);
  const ids = [...new Set(loans.flatMap((l) => [l.lender, l.borrower]))].sort((a, b) => a - b);
  const levels = creditLevels(ids, loans);
  const rows: number[][] = [];
  for (const id of ids) {
    const level = levels.get(id) ?? 0;
    while (rows.length <= level) rows.push([]);
    rows[level].push(id);
  }
  return {
    rows,
    loans,
    roles: new Map(graph.agents.map((a) => [a.id, a.role])),
    agents: graph.agents.length,
    totalLoans: graph.loans.length,
    omitted: graph.loans.length - loans.length,
  };
}

/** "N agents · M loans · L levels", noting any loans left out. */
export function creditHeader(layout: CreditLayout): string {
  if (layout.totalLoans === 0) return 'No outstanding loans.';
  const count = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`;
  const text = `${count(layout.agents, 'agent')} · ${count(layout.totalLoans, 'loan')} · ${count(layout.rows.length, 'level')}`;
  return layout.omitted > 0 ? `${text} · the ${layout.loans.length} largest loans drawn (${layout.omitted} omitted)` : text;
}
```

- [ ] **Step 3: The panel, the tab and the engine**

Create `web/src/ui/credit-panel.ts`:
```ts
import { creditHeader, creditLayout } from '../credit';
import type { Engine } from '../engine';
import { h } from './dom';

const SVG_NS = 'http://www.w3.org/2000/svg';
const ROW = 56;
const PAD = 14;
const RADIUS = 5;
const REFRESH_MS = 500;

function svg<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string | number> = {}): SVGElementTagNameMap[K] {
  const el = document.createElementNS(SVG_NS, tag);
  for (const [key, value] of Object.entries(attrs)) el.setAttribute(key, String(value));
  return el;
}

/** Animation IV-5's lender → borrower hierarchy: one row per level, lenders green, borrowers red, both yellow. */
export class CreditPanel {
  readonly el = h('div', { class: 'credit' });
  private header = h('p', { class: 'hint' });
  private graph = h('div', { class: 'credit-graph' });
  private visible = false;
  private last = 0;

  constructor(private engine: Engine, private onSelect: () => void) {
    this.el.append(this.header, this.graph);
    engine.on('reset', () => this.refresh());
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) this.refresh();
  }

  /** Called every animation frame; redraws at most every REFRESH_MS while visible. */
  maybeRefresh(now: number): void {
    if (this.visible && now - this.last >= REFRESH_MS) this.refresh();
  }

  private refresh(): void {
    this.last = performance.now();
    if (!this.visible) return;
    const layout = creditLayout(this.engine.creditGraph());
    const summary = creditHeader(layout);
    this.header.textContent = summary;
    if (layout.rows.length === 0) {
      this.graph.replaceChildren();
      return;
    }
    const width = Math.max(240, this.graph.clientWidth || 360);
    const height = 2 * PAD + layout.rows.length * ROW;
    const at = new Map<number, [number, number]>();
    layout.rows.forEach((ids, level) =>
      ids.forEach((id, i) => at.set(id, [((i + 1) * width) / (ids.length + 1), PAD + level * ROW + ROW / 2])),
    );
    const edges = svg('g', { class: 'credit-edges' });
    for (const l of layout.loans) {
      const a = at.get(l.lender);
      const b = at.get(l.borrower);
      if (a && b) edges.append(svg('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1] }));
    }
    const nodes = svg('g', { class: 'credit-nodes' });
    for (const [id, [x, y]] of at) {
      const role = layout.roles.get(id) ?? 'both';
      const node = svg('circle', { cx: x, cy: y, r: RADIUS, class: `credit-node ${role}` });
      const title = svg('title');
      title.textContent = `#${id} · ${role}`;
      node.append(title);
      node.addEventListener('click', () => {
        this.engine.selectAgent(id);
        this.onSelect();
      });
      nodes.append(node);
    }
    const root = svg('svg', { viewBox: `0 0 ${width} ${height}`, width, height, role: 'img', 'aria-label': `Credit hierarchy: ${summary}` });
    root.append(edges, nodes);
    this.graph.replaceChildren(root);
  }
}
```
In `web/src/ui/tabs.ts`, add to `class Tabs`:
```ts
  /** Hides or shows a tab's button; hiding the selected tab selects the first visible one. */
  setHidden(label: string, hidden: boolean): void {
    const entry = this.entries.find((e) => e.label === label);
    if (!entry) return;
    entry.button.hidden = hidden;
    if (hidden && entry.button.getAttribute('aria-selected') === 'true') {
      const first = this.entries.find((e) => !e.button.hidden);
      if (first) this.show(first.label);
    }
  }
```
In `web/src/engine.ts`, add `import type { CreditGraph } from './credit';` and the method (after `diseaseList`):
```ts
  /** The outstanding loans and the agents in them. */
  creditGraph(): CreditGraph {
    return JSON.parse(this.sim.credit_graph()) as CreditGraph;
  }
```
In `web/src/main.ts`, add `import { CreditPanel } from './ui/credit-panel';`; after the Inspect tab lines (before `document.querySelector('#tools')!.append(…)`) add:
```ts
  const credit = new CreditPanel(engine, () => tabs.show('Inspect'));
  tabs.add('Credit', credit.el, (visible) => credit.setVisible(visible));
  // The Credit tab exists only while credit (L) is on.
  const syncCreditTab = () => tabs.setHidden('Credit', !engine.config.credit.enabled);
  engine.on('reset', syncCreditTab);
  engine.on('config', syncCreditTab);
  syncCreditTab();
```
and in `loop`, replace `charts.maybeRefresh(performance.now());` with
```ts
      const now = performance.now();
      charts.maybeRefresh(now);
      credit.maybeRefresh(now);
```
In `web/src/style.css`, add inside the first `:root { … }` block, after `--red: #ff4d4d;`:
```css
  --lender: #3dd66b;
  --borrower: #ff4d4d;
  --both: #ffe04d;
```
and append:
```css
.credit { display: grid; gap: 8px; }
.credit .hint { margin: 0; }
.credit-graph svg { display: block; width: 100%; height: auto; }
.credit-edges line { stroke: var(--muted); stroke-opacity: 0.6; stroke-width: 1; }
.credit-node { stroke: var(--text); stroke-width: 0.5; cursor: pointer; }
.credit-node.lender { fill: var(--lender); }
.credit-node.borrower { fill: var(--borrower); }
.credit-node.both { fill: var(--both); }
```

- [ ] **Step 4: Verify**

```bash
(cd web && npm run build && npm test)
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/credit.ts web/src/credit.test.ts web/src/ui/credit-panel.ts web/src/ui/tabs.ts web/src/engine.ts web/src/main.ts web/src/style.css
git commit -m "Show the credit hierarchy in a Credit tab" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 16: Book-style tests

*Needs judgement: the tolerance and the recorded numbers come from Task 7's measurement.*

**Files:**
- Modify: `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: `run_builtin`, `cell_means` (already in `book.rs`), `sweeps/bargaining-rules.json` at its recorded settings and τ (Task 7), preset `iii-6-three-tribes` (Task 3), `Agent::group`, `Snapshot.groups`.
- Produces: `bargaining_rules_give_similar_carrying_capacities`, `three_tribes_start_with_every_group_present` (both `#[ignore]`, run in release).

- [ ] **Step 1: Write the tests**

Append to `crates/sugarscape-core/tests/book.rs`:
```rust
#[test]
#[ignore]
fn bargaining_rules_give_similar_carrying_capacities() {
    // Chapter IV note 15: with a price drawn from [MRS_A, MRS_B], "the
    // qualitative character of the results … is insensitive to this change".
    // At `sweeps/bargaining-rules.json`'s settings (series 0 = geometric
    // mean, 1 = random), the two lines' mean carrying capacities differ by
    // less than TOLERANCE × the geometric-mean line at every mean vision.
    // TOLERANCE is the τ measured and recorded in the sweep's description
    // (model extensions plan, Task 7). Measured means: recorded in Step 2.
    const TOLERANCE: f64 = τ;
    let means = cell_means(&run_builtin("bargaining-rules"));
    for (x, (&geometric, &random)) in means[0].iter().zip(&means[1]).enumerate() {
        assert!(
            (random - geometric).abs() < TOLERANCE * geometric,
            "vision {x}: geometric mean {geometric}, random {random}"
        );
    }
}

#[test]
#[ignore]
fn three_tribes_start_with_every_group_present() {
    // Chapter III note 20's three groups on 11-bit tags: with uniformly random
    // tags about 11% of agents are Blue (0–3 zeros), 77% Green (4–7) and 11%
    // Red (8–11). Measured at t = 0, seed 1: recorded in Step 2.
    let config = presets::by_id("iii-6-three-tribes").unwrap().config;
    let groups = config.culture.groups.clone();
    let names: Vec<&str> = groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["Blue", "Green", "Red"]);
    let mut w = World::new(config, 1).unwrap();
    let mut members = vec![0; groups.len()];
    for a in w.agents() {
        members[a.group(&groups)] += 1;
    }
    assert!(members.iter().all(|&m| m > 0), "members per group at t = 0: {members:?}");
    let shares = &w.stats.latest().unwrap().groups;
    assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-12, "{shares:?}");
    w.run(500);
    assert!(w.population() > 0);
}
```

- [ ] **Step 2: Fill in the measured values**

Replace `τ` in `const TOLERANCE: f64 = τ;` with the τ written in `sweeps/bargaining-rules.json`'s description (one of 0.10, 0.15, 0.20, 0.25). Temporarily add `println!("{means:?}");` after `let means = …` and `println!("{members:?}");` after the members loop, then run:

`cargo test -p sugarscape-core --release --test book -- --ignored --nocapture bargaining_rules three_tribes`

Replace each "recorded in Step 2" with the printed values (means rounded to one decimal, as `[[geometric …], [random …]]`; the member counts as printed), then remove the `println!`s. If an assertion fails at the recorded settings, stop and report **BLOCKED** with the numbers: do not change τ, seeds or the sweep file here (Task 7 already chose τ from these same runs; a failure means something changed since).

- [ ] **Step 3: Verify**

```bash
cargo fmt --all && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
cargo test -p sugarscape-core --release --test book -- --ignored
```
Expected: every book test, old and new, passes.

- [ ] **Step 4: Commit**

```bash
git add crates/sugarscape-core/tests/book.rs
git commit -m "Check bargaining insensitivity and the three-tribe start against the book" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 17: README, roadmap and full verification

*Mechanical.*

**Files:**
- Modify: `README.md`, `docs/roadmap.md`

- [ ] **Step 1: README**

In `README.md`, insert after the N-goods paragraph of `## Rules implemented` (the one ending "Presets `n-3-trade`, `n-4-peaks` and `n-2-pollutants` show them.") and before `### Notes`:
```markdown
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
  brightness (max capacity 0–10, optionally inverted); like painted maps, imported maps travel
  with share links and survive resets.
- **Agent trails.** Inspect an agent and press **Follow** to draw its last 500 positions on the
  grid (Animation IV-1's tail), fading with age and broken where it wraps around the torus. The
  toolbar chip stops following. Trails are views only: they never change a run and are not
  exported or shared.
- **Credit hierarchy.** With credit on, the **Credit** tab draws Animation IV-5's lender →
  borrower hierarchy: one row per level (pure lenders on top; loans that close a cycle are
  ignored), lenders green, borrowers red, both yellow. Clicking an agent inspects it. Above 400
  loans only the 400 largest are drawn.
```
In `### Notes`, add as the last bullets:
```markdown
- With custom tag groups, `blue_fraction` is the share of group 0. The corner "Two tribes"
  placement, replacement's same-tribe newcomers, the Place tool's Tribe choice and the agents
  CSV `tribe` column still use the book's two-tribe rule (Blue when zeros outnumber ones).
- Changing the tag length rebuilds the default groups; custom groups are kept and must still
  cover every zero count, or the world is not rebuilt and the Culture section explains why.
```

- [ ] **Step 2: Roadmap**

In `docs/roadmap.md`, insert after the Milestone 5 section:
```markdown
## Milestone 6: Model extensions (done)

User-defined tag groups with the book's three-tribe scheme (`iii-6-three-tribes`), a pluggable
bargaining rule (geometric mean or a random price in [MRS_A, MRS_B]) with the measured
`bargaining-rules` sweep, seeded fractal-noise maps and image import for landscapes, agent
trails, and a layered credit-hierarchy tab. Earlier presets run unchanged. See
`docs/superpowers/specs/2026-09-23-model-extensions-design.md`.
```
In "Experiments and science", replace the "Credit hierarchy view" bullet with
```markdown
- **Credit hierarchy view**: done (Milestone 6).
```
and replace the four "Model extensions" bullets with
```markdown
- **More than two tribes**: done (Milestone 6).
- **Alternative bargaining rules**: done (Milestone 6).
- **Custom landscape generators**: done (Milestone 6: noise maps and image import).
- **Observational agent trails**: done (Milestone 6).
```

- [ ] **Step 3: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
cargo test -p sugarscape-core --release --test book -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
git diff main -- crates/sugarscape-core/tests/legacy.rs crates/sugarscape-core/tests/fixtures
git diff main -- crates/sugarscape-core/tests/golden.rs
```
All must pass; the first diff must print nothing and the second only the three lines Task 3 added. The controller then runs the puppeteer pass (implementers don't): load `iii-6-three-tribes` (Blue/Green/Red agents in the Tribe mode, a three-line Group shares chart, the groups table with "Three tribes (book)"; rename a group live; move a boundary → the world rebuilds); `iv-3-trade` with Price rule "Random" (runs, trade price chart moves); a good switched to a Noise map (🎲 re-roll changes it; share link reproduces it); "Import image…" with a PNG (capacity layer shows it; Share → reload keeps it); Follow an agent until its trail crosses an edge (no line across the grid; the chip's ✕ clears it; Reset clears it); `iv-5-credit` for ~100 ticks (Credit tab appears, rows by level, clicking a node opens Inspect; the tab disappears when credit is switched off).

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md
git commit -m "Document the model extensions" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

## Spec coverage

| Spec requirement | Task |
|---|---|
| Earlier runs unchanged; golden/legacy unedited except new-preset entries | Global Constraints; 3 (one appended entry, predicted value); every task's verification; 17 (diffs) |
| New RNG draws only under `random` | 5 (`geometric_mean_pricing_draws_nothing`), 8 (noise uses no RNG) |
| Trails never hashed, never in configs | 12 (fingerprint and export tests) |
| One implementation in the core; web calls WASM | Global Constraints, Decision 7; 10, 12, 14 (WASM) |
| `culture.groups` config, first matching group, `#[serde(default)]` | 1 |
| Default groups = today's rule (L = 11: Blue 6–11, Red 0–5); default for a config's `tag_length`; `Config::default()` includes it | 1 (exhaustive over 11-bit strings and all lengths 1–64) |
| Validation: 1–8 groups, names, colors, ranges tile 0..=L | 1 |
| Combat: every other group is an enemy | 2 |
| `group_share_K`; `blue_fraction` = share of group 0 | 2 |
| Tribe color mode by group color | 2 (core), 4 (chart colors) |
| Reset-only: count/ranges/`tag_length`; names/colors live | 1 (schedule + structural), 4 (UI commits) |
| Preset `iii-6-three-tribes` | 3 |
| UI groups table with add/remove and the two book buttons; tag length rebuilds default groups | 4 |
| `trade.price` config, `random` draws in [min, max] with `World.rng`, checks unchanged | 5 |
| Price rule select, live and schedulable | 5 (schedulable), 6 (UI) |
| `bargaining-rules` sweep, measured settings in its description | 7 |
| `Map::Noise`, generation (hash, smoothstep, torus wrap, octaves, normalization, rounding), validation, determinism | 8 (period convention: Decision 10) |
| Goods editor Noise kind with 🎲 | 9 |
| `World::set_capacities`, WASM `set_landscape`, counts as painted | 10, 11 (`keepLandscape`) |
| Image import UI and luminance conversion | 11 |
| `World::follow`/`trail`/`followed`, 500 cap, death, reset | 12 (core, WASM), 13 (reset clears via a new `Sim`) |
| Follow button, toolbar chip, fading polyline broken at wraps | 13 |
| WASM `credit_graph` | 14 |
| Levels (DFS loop cutting; cycles level 0), Credit tab (visible with credit, SVG rows, colors, click selects, header, ≤ 2 redraws/s, 400-loan cap) | 15 (levels: Decision 16) |
| Tests: core unit list | 1, 2, 5, 8, 10, 12, 14 |
| Tests: web (credit levels, image → capacities, trail wrap split, groups table ↔ config) | 15, 11, 13, 4 |
| Tests: book-style (bargaining tolerance, three tribes at t = 0) | 7 (measure), 16 |
| Tests: browser | 17 (controller pass) |
| Docs: README and roadmap | 17 |

## Spec gaps and conflicts found

1. **Noise octave periods.** The spec's `round(W / (scale · 2^o))` makes each octave twice as coarse at half the amplitude — the reverse of fractal noise (and of "scale" as the base feature size). This plan uses `round(W · 2^o / scale)` (Decision 10); everything else about the generator is as specified.
2. **"Nothing else reads the tribe today"** is not quite true: the `tribes` corner placement, replacement's same-tribe newcomer, the Place tool's Tribe override, the inspector and the agents CSV `tribe` column all use Blue/Red. They keep the two-tribe rule (they coincide with the default groups); only combat, statistics, the Tribe color mode and the inspector's group name follow custom groups (Decision 5).
3. **"If `tag_length` changes, the UI rebuilds the default groups"** does not say what happens to custom groups. This plan rebuilds only default groups and keeps custom ones, which then fail validation until fixed (Decision 7). The "three tribes (book)" button is defined only for 11 bits; it is generalized to thirds and disabled below 2 bits (Decision 1).
4. **Credit levels.** "DFS with loop cutting" gives order-dependent levels inside cycles, while "agents in a cycle with no pure lender above get level 0" requires all members to be 0. Levels are computed on strongly connected components (Decision 16), which satisfies both on acyclic graphs and the second sentence on cyclic ones.
5. **Random price draws.** "For each exchange" does not say whether exchanges that fail the checks draw. Every attempted exchange draws (Decision 8), which keeps the draw in one place and the run reproducible.
6. **Transparent pixels with Invert.** Taken literally ("alpha < 128 counts as 0", then `1 − v`), transparent areas become the maximum capacity when inverted; the plan does exactly that (Decision 12).
7. **Header with more than 400 loans.** The spec does not say whether "agents, loans, levels" count the whole graph or the drawn part: totals for agents and loans, drawn levels, plus the omitted count (Decision 17).
8. **`set_capacities` range.** It accepts 0–10 as specified, while `paint_capacity` accepts any capacity ≥ 0; imported maps are bytes, so this only matters to core callers.
9. **The three-tribe preset's golden fingerprint** equals `iii-6-culture`'s (groups change nothing while combat is off), so its golden entry duplicates a value — expected, and used in Task 3 as a check.
