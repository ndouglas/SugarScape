# SugarScape WASM Playground — Design

**Date:** 2026-09-22
**Source text:** Epstein & Axtell, *Growing Artificial Societies: Social Science from the Bottom Up* (1996). Rule definitions follow Appendix B ("Summary of Rule Notation") and the rule statements in Chapters II–III.

## Goal

A browser playground that faithfully implements the Sugarscape model through Chapters II–III, built as a pure Rust simulation core compiled to WASM with a TypeScript front end. Users can run the book's rule systems, tune parameters, inspect agents, edit the world, share configurations by URL, and export data.

## Scope

**In scope (milestone 1):**

| Rule | Name | Notes |
|------|------|-------|
| G_α | Sugarscape growback | α = ∞ supported (instant regrowth) |
| S_{α,β,γ} | Seasonal growback | summer top half / winter bottom half, flip every γ ticks; winter rate α per β ticks |
| M | Agent movement | with pollution-modified welfare when pollution is on |
| P_{α,β} / D_α | Pollution formation / diffusion | single resource, single pollutant (Chapter II form) |
| R_[a,b] | Agent replacement | only meaningful when S (sex) is off |
| S | Agent sex (mating) | Mendelian crossover of genetics and culture |
| I | Agent inheritance | wealth split equally among living children |
| K | Agent culture | cultural transmission + group (tribe) membership |
| C_α | Agent combat | replaces M when enabled; α = ∞ supported |

**Out of scope (future milestones):** spice and trade (T), credit (L), disease (E), Chapter VI. The engine must accommodate them as additional rule functions and toggles without restructuring.

## Architecture

```
SugarScape/
├── Cargo.toml                  # workspace
├── crates/
│   ├── sugarscape-core/        # pure Rust, no wasm deps
│   │   └── src/
│   │       ├── config.rs       # all parameters + rule toggles; serde; validate() -> Vec<FieldError>
│   │       ├── rng.rs          # seeded PCG (rand_pcg); single RNG stream owned by World
│   │       ├── landscape.rs    # Site { sugar, capacity, pollution }; torus indexing
│   │       ├── agent.rs        # Agent { id, pos, vision, metabolism, sugar, initial_sugar, age,
│   │       │                   #         max_age, sex, fertility window, culture tags, parents, children }
│   │       ├── rules/          # growback.rs seasons.rs pollution.rs movement.rs replacement.rs
│   │       │                   # sex.rs inheritance.rs culture.rs combat.rs
│   │       ├── world.rs        # World::step — fixed rule order
│   │       ├── stats.rs        # per-tick series, Gini, Lorenz, histograms
│   │       ├── render.rs       # RGBA frame buffer for (color mode, layer)
│   │       ├── export.rs       # CSV of series and agents
│   │       └── presets.rs      # book rule systems + two-peak sugar capacity map
│   └── sugarscape-wasm/        # wasm-bindgen wrapper exposing `Sim`
└── web/                        # Vite + TypeScript front end
```

### Boundary strategy

Rust owns all state. Each frame Rust renders the grid into an RGBA buffer; JS views it zero-copy as a `Uint8ClampedArray` over WASM memory and blits it with `putImageData`. JS draws only overlays (selection ring, brush cursor). Chart data crosses as `Float64Array`s. Small structured data (latest stats, inspection results, config) crosses as JSON strings.

Note: any WASM memory growth invalidates existing typed-array views, so JS must re-create views from the pointer after every call that can allocate (i.e. re-acquire each frame).

## Simulation semantics

### Tick order

Agents act **asynchronously in a freshly shuffled random order** each tick (the book: an agent mates with every neighbor "before it is the next agent's turn to move").

1. Shuffle living agents.
2. For each agent in order (skipping any killed earlier this tick):
   1. **Move** — M, or C_α instead if combat is enabled. Collect the site's sugar.
   2. **Metabolize** — sugar -= metabolism. If pollution is on, gathering adds α·gathered and metabolizing adds β·metabolism to the site's pollution.
   3. **Death check** — die if sugar ≤ 0 or age > max_age. On death, if I is on, split wealth equally among living children.
   4. **Sex (S)** — if fertile, iterate neighbors in random order; mate with each fertile opposite-sex neighbor where either party has an empty von Neumann neighbor site.
   5. **Culture (K)** — for each neighbor: pick a random tag index; set neighbor's tag to the agent's.
3. **Environment** — growback (G_α or seasonal; G∞ ignores seasons), pollution diffusion every α_D ticks.
4. **Replacement** — if R is on, replace each agent that died this tick with a fresh random agent.
5. **Ageing** — age all agents by 1. Replacement runs before ageing, so replacements, like newborns, end their first tick at age 1.
6. **Stats** — append this tick's statistics.

### Movement (M)

Look up to `vision` sites in each of the four lattice directions (torus wraparound). Consider only unoccupied sites plus the current site. Choose the maximum-welfare site; break ties by nearest distance, then uniformly at random. Welfare is `sugar` or, with pollution, `sugar / (1 + pollution)`. Move there and gather all sugar.

### Combat (C_α)

Per Appendix B: candidate sites in the four directions within vision; discard sites occupied by own tribe; discard sites occupied by other-tribe agents at least as wealthy as the attacker; reward = site sugar + (if occupied) min(α, occupant's sugar); discard sites vulnerable to retaliation; move to the nearest max-reward site; collect reward; the former occupant is killed (removed permanently). The victim's sugar beyond the reward is passed to its children if I is on, otherwise lost.

**Retaliation (interpretation):** a target site is vulnerable if any other-tribe agent within the attacker's vision *of the target site* has sugar greater than the attacker's post-attack wealth (attacker sugar + reward).
Further interpretations: staying put (the current site, distance 0) is not subject to the retaliation filter; equal-wealth targets are excluded, since the book's text says the predator "must be bigger than" its prey; loot taken from a victim does not count toward production pollution (only the site's sugar does).

### Sex (S)

- Fertile = age within the agent's fertility window **and** sugar ≥ initial endowment.
- Default fertility windows (book): onset uniform in [12, 15] for both sexes; end uniform in [40, 50] for women, [50, 60] for men. All configurable.
- Child endowment = ½ father's initial endowment + ½ mother's initial endowment, deducted from each parent.
- Child genetics: vision, metabolism, max age and fertility onset each from a uniformly chosen parent. Fertility end is drawn from the child's own sex-specific range (the ranges differ by sex, so it cannot be inherited across sexes).
- Child culture: for each tag, parents' common value if they agree, else a random parent's.
- Child sex: uniform.
- Child placement: a uniformly random empty von Neumann neighbor of either parent.
- Children do not act until the tick after birth.

### Culture (K)

Tag strings of configurable length (default 11). Tribe = Blue if zeros outnumber ones, Red otherwise.

### Numeric representation

Sugar (site and agent) is `f64` so seasonal fractional growback and inheritance splits need no rounding. Capacities are integers 0–4 in the default map but stored as `f64`.

### Presets

Named after the book's rule systems, e.g. `({G∞}, {M})`, `({G₁}, {M})`, `({G₁}, {M, R[60,100]})` (wealth distribution), seasonal migration, pollution, `({G₁}, {M, S})`, `({G₁}, {M, K})`, `({G₁}, {C∞})`, and a combined Chapter III system. Each preset's exact parameters (population size, vision/metabolism/endowment ranges, etc.) are transcribed from the book's text during implementation, with a comment citing the page.

The default landscape is the classic 50×50 two-peak capacity map (capacities 0–4).

## WASM API (`Sim`)

```
new(config_json, seed) -> Result<Sim>
step(n)
render(color_mode, layer) -> ptr;  frame_len(), width(), height()
stats_latest() -> JSON;  series(name) -> Float64Array;  lorenz() / wealth_hist() -> Float64Array
inspect(x, y) -> JSON            // site + agent + lineage ids
paint_capacity(x, y, radius, value);  place_agent(x, y, overrides_json) -> Result;  remove_agent(x, y)
set_config(json) -> Result       // mid-run rule/parameter changes; per-field validation errors
export_config() -> JSON;  export_landscape() -> bytes
export_series_csv() -> String;  export_agents_csv() -> String
```

Color modes: culture/tribe, wealth, sex, age, vision. Layers: sugar, capacity, pollution.

## Web UI

- **Grid canvas** (center): nearest-neighbor scaled; overlays for selection and brush.
- **Toolbar:** play/pause, step, steps-per-frame, reset, seed field + randomize.
- **Tools:** Inspect, Paint capacity (brush radius, value 0–4), Place agent, Erase agent.
- **Display:** color-mode and layer selectors.
- **Side panel tabs:**
  - *Rules* — preset dropdown; per-rule toggles; grouped parameter sliders. Deviating from a preset marks it "modified".
  - *Charts* (uPlot) — population, Gini, mean vision, mean metabolism, Blue fraction over time; live Lorenz curve; wealth histogram.
  - *Inspect* — selected agent/site details; parents/children are clickable.
- Runs on the main thread via `requestAnimationFrame`. No Web Worker for now.

## Sharing and export

- **Share URL:** `#s=` + base64url(deflate(JSON `{config, seed, landscape?}`)) via the native `CompressionStream`. Landscape is included only if it differs from the preset's. Hand-placed agents are not captured. Malformed links show an error and fall back to the default preset.
- **Exports:** series CSV (full per-tick history held in core), current-agents CSV, grid PNG, per-chart PNG.

## Error handling

- `Config::validate()` returns field-level errors; `Sim::new`/`set_config` surface them as `JsValue` errors; the UI shows them inline next to the offending control and keeps the previous valid config.
- Edits outside the grid or onto occupied sites return errors; the UI ignores them.
- A Rust panic hook (`console_error_panic_hook`) logs to console; the UI shows a "simulation crashed" banner with a Reload button (a panic leaves the WASM instance unusable; reloading keeps any `#s=` share state).

## Testing

- **Unit tests per rule** on small hand-built worlds (e.g. vision-2 agent picks nearer of two equal sites; combat skips a retaliation-vulnerable site; inheritance splits evenly; K flips exactly one tag per neighbor).
- **Property tests** (`proptest`): at most one agent per site; site sugar ≤ capacity; no living agent with sugar ≤ 0 after the tick; population never negative; R keeps population constant.
- **Determinism:** same seed + config ⇒ byte-identical world state hash after N ticks.
- **Book-reproduction tests** (`#[ignore]`, run explicitly in CI with `--ignored` in release mode, and manually): ({G₁},{M}) settles to a stable carrying capacity; R[60,100] Gini exceeds ~0.5; K converges toward single-tribe dominance.
- **TypeScript:** `vitest` for URL encode/decode and config/form mapping; `tsc --noEmit`.

## Build and deploy

- Local: `npm run dev` in `web/` runs `wasm-pack build --target web` then Vite dev server. `wasm-pack` must be installed.
- CI (GitHub Actions): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `tsc --noEmit`, `vitest`.
- Deploy: on `main`, build WASM + `vite build`, publish to GitHub Pages.
