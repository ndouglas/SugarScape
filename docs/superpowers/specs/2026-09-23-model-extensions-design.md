# SugarScape Milestone 6 — Model extensions — Design

**Date:** 2026-09-23
**Builds on:** the milestone 1–5 specs in `docs/superpowers/specs/`; all remain binding where not changed here.
**Source text:** Epstein & Axtell, *Growing Artificial Societies*: Chapter III note 20 (a three-group tag scheme: Blue 0–3 zeros, Green 4–7, Red 8–11 on 11-bit strings), Chapter IV note 15 (a bargaining rule that "picks a random number from the interval [MRS_A, MRS_B]"; "the qualitative character of the results … is insensitive to this change"), Animation IV-1 (an agent followed by a black tail), Animation IV-5 (the lender → borrower hierarchy; "as many as five levels of lenders-borrowers emerge").

## Goal

Four extensions: user-defined tag groups (tribes), a pluggable bargaining rule, seeded noise maps plus image import for landscapes, and two views — an agent trail and a layered credit hierarchy.

## Non-negotiable constraints

- **Earlier runs are unchanged.** With default settings (the default groups, `geometric_mean` pricing, no noise maps) every golden entry and legacy fixture stays green and unedited. New RNG draws happen only under `random` pricing.
- **Trails are not simulation state:** never hashed, never exported in configs.
- **One implementation:** model rules in `sugarscape-core`; the web calls WASM.

## Groups (tribes)

- **Config:** `culture.groups: Vec<Group { name: String, color: String, zeros: URange }>` (`#[serde(default)]`). An agent's group is the first whose `zeros` range contains the number of zeros in its tags.
- **Default:** two groups reproducing today's rule (Blue when zeros outnumber ones, else Red): Blue `zeros ⌈(L+1)/2⌉..=L`, Red `0..=⌈(L+1)/2⌉−1` for tag length L (L = 11: Blue 6–11, Red 0–5). A config without `groups` gets the default for its `tag_length`; `Config::default()` includes it. (If `tag_length` changes, the UI rebuilds the default groups; a config whose groups no longer cover `0..=tag_length` fails validation.)
- **Validation:** 1–8 groups; names 1–16 chars, unique; colors `#rrggbb`; ranges non-empty, together covering `0..=tag_length` exactly once (no gaps, no overlaps).
- **Rules:** combat's "own tribe" means the same group (every other group is an enemy); nothing else reads the tribe today.
- **Stats:** appended per group `group_share_K` (share of living agents in group K). `blue_fraction` keeps its name and meaning when the groups are the default (share of group 0, which is Blue); with custom groups it is the share of group 0.
- **Render:** the Tribe color mode colors each agent by its group's color.
- **Reset-only:** changing the number of groups, their ranges or `tag_length` is reset-only; names and colors apply live.
- **Preset:** `iii-6-three-tribes` — the culture preset (`iii-6-culture`) with groups Blue 0–3, Green 4–7, Red 8–11 (the book's scheme) and colors matching the default Blue/Red plus a green.
- **UI:** the Culture rules group gets a groups table (name, color, zeros from–to; add/remove), with a "two tribes (book)" / "three tribes (book)" preset button pair.

## Bargaining

- **Config:** `trade.price: "geometric_mean" | "random"` (default `geometric_mean`, `#[serde(default)]`).
- **`random`:** for each exchange, p is drawn uniformly from `[min(MRS_A, MRS_B), max(MRS_A, MRS_B)]` (the pair's MRSs for the goods pair being traded) with `World.rng`; everything else about the exchange (quantities from p, welfare, no-crossing and positivity checks, widest-gap pair choice) is unchanged. Draws happen only under `random`. A draw of exactly the lower bound is allowed; the checks still apply.
- **Stats:** unchanged (`mean_log_price`, `sd_log_price` use the prices actually traded).
- **UI:** a "Price rule" select in the Trade group, applied live (schedulable).
- **Built-in sweep:** `bargaining-rules` — base `iv-3-trade`, series = price rule (geometric mean / random), x = mean vision (as in `fig-iv-6`), metric = carrying capacity (window mean of population). Settings measured and recorded in its description.

## Noise maps

- **Map kind:** `noise { seed: u32, scale: f64, octaves: u32, height: f64 }` in each good's `map` (alongside `two_peaks`, `peaks`, `flat`).
- **Generation:** fractal value noise over the torus: octave o (0-based) samples a lattice of period `max(1, round(W / (scale · 2^o)))` × `max(1, round(H / (scale · 2^o)))` cells whose corner values are hashed from `(seed, o, lattice x, lattice y)` (a fixed integer hash, e.g. SplitMix64-style, no RNG state), interpolated with smoothstep, wrapping lattice indices modulo the period so the map tiles seamlessly; amplitudes 1, ½, ¼ …; the sum is normalized to [0, 1] by the total amplitude and mapped to capacity `round(height · v)`, clamped to `0..=height`.
- **Validation:** `scale` finite, 1–100 (cells); `octaves` 1–6; `height` 0–10.
- **Determinism:** independent of `World.rng` and of platform integer/float differences beyond the documented last-bit caveat (the hash is integer; the interpolation is plain arithmetic).
- **UI:** the Goods editor's map kind select gains "Noise" with seed (with a 🎲 re-roll), scale, octaves and height.

## Agent trails

- **Core:** `World::follow(&mut self, id: Option<AgentId>)`, `World::trail(&self) -> &[Pos]`, `World::followed(&self) -> Option<AgentId>`. After each tick's movement phase completes (end of `step`), if the followed agent is alive its position is pushed; the buffer keeps the last 500 positions. When the agent dies the trail stops growing and remains until `follow` is called again. Following starts the trail empty, then records the current position immediately. Not part of `fingerprint`, configs, exports or share links.
- **WASM:** `follow(id: f64)`, `unfollow()`, `trail(): Uint32Array` (x, y pairs, oldest first), `followed(): f64` (−1 for none).
- **UI:** the Inspect panel's agent section gets a **Follow** button; a toolbar chip "Following #id ✕" (✕ unfollows). The grid overlay draws the trail as a polyline through cell centers, fading from transparent (oldest) to the foreground color (newest), breaking the line where consecutive positions are more than half the grid apart in x or y (torus wrap). Resetting the world clears the follow.

## Credit hierarchy

- **WASM:** `credit_graph(): string` → `{ agents: [{ id, role: "lender" | "borrower" | "both" }], loans: [{ lender, borrower, good, due }] }` over outstanding loans (agents appear only if they take part in a loan).
- **Levels (TS, pure function):** level(a) = 0 for pure lenders; otherwise 1 + max level of its lenders, computed by DFS with loop cutting (an edge that closes a cycle is ignored for levels). Agents in a cycle with no pure lender above get level 0.
- **View:** a **Credit** tab in the side panel, visible when credit is on. An SVG layered graph: one row per level (top = level 0), nodes ordered by id within a row, edges lender → borrower; node colors lender green, borrower red, both yellow (the book's colors, via CSS tokens); clicking a node selects that agent (Inspect). A header line: agents, loans, levels. Redraw throttled to ≤ 2/s while the tab is visible. With more than 400 loans, the 400 largest by `due` are drawn and the header says how many are omitted.

## Image import

- **Paint tool:** "Import image…" (file input, `image/*`), a max capacity (0–10, default 4) and an Invert checkbox, for the paint tool's selected good.
- **Conversion (TS, pure function on RGBA pixels):** the image is drawn to an offscreen canvas at the grid's width × height with `imageSmoothingQuality = 'high'`; per pixel, luminance `Y = 0.2126 R + 0.7152 G + 0.0722 B` (alpha < 128 counts as 0), `v = Y / 255` (or `1 − v` if inverted), capacity `round(v · max)`.
- **Core/WASM:** `World::set_capacities(&mut self, good: usize, capacities: &[f64]) -> Result<(), String>` (length must equal the grid; values 0–10; resources above the new capacity are clamped, as painting does); WASM `set_landscape(good: u32, capacities: Uint8Array)`. The good then counts as painted (`landscape_edited`), so share links, export and reset keep it.

## Testing

- **Golden/legacy:** unchanged; new golden entries for new presets only.
- **Core unit:** default groups equal the old tribe rule for every tag string of length 11 (exhaustive); three-group assignment; group validation (gap, overlap, out of range); combat treats every other group as enemy; `group_share_K` stats; `geometric_mean` identical to before (golden); `random` prices lie in the MRS interval, are reproducible per seed, and all exchange checks hold (property test); noise range, torus seamlessness (value at x = 0 vs x = W−1 continuity), determinism, seed sensitivity; trail recording, cap, death, not in fingerprint; `set_capacities` clamping and errors; `credit_graph` roles.
- **Web (Vitest):** credit levels (chain, diamond, cycle); image → capacities (luminance, invert, alpha, rounding); trail segment splitting at wrap; groups table ↔ config.
- **Book-style (`#[ignore]`, release):** `bargaining-rules` — at every x the two rules' mean carrying capacities differ by less than a tolerance measured and recorded; `iii-6-three-tribes` runs and all three groups are present at t = 0.
- **Browser (controller):** three-tribe preset colors and chart, random pricing, a noise map in the Goods editor, image import, following an agent (trail across a wrap), and the Credit tab on `iv-5-credit`.

## Docs

README: groups, price rule, noise maps, image import, trails, credit tab; roadmap: mark the model-extension items done.
