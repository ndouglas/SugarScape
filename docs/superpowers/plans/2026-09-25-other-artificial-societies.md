# Other Artificial Societies: Schelling and Ring World Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the book's Schelling segregation variant (animations VI-4 to VI-7) and Ring World (VI-8, VI-9) as model kinds beside the sugarscape — in the worker engine at every speed including Max, in replay and share links, Compare, recording, Experiments (with the measured built-in sweep `schelling-tipping`) and the CLI — without changing a single sugarscape run, config, link, session or sweep.

**Architecture:** A config becomes a `ModelConfig` (untagged JSON = sugarscape, `"model": "schelling" | "ring"` otherwise) and a world a `ModelWorld` behind a `Model` trait (crates/sugarscape-core/src/model.rs); `World` implements the trait by delegating, so the sugarscape's code paths are untouched. `schelling.rs` and `ring.rs` implement the two new models with their own statistics, frames, inspections, schemas and presets; sweeps and the CLI run any model through the trait. The WASM `Sim` holds a `ModelWorld` (sugarscape-only calls answer empty for the others). On the page the sugarscape's `Config` type stays as it is: `ModelConfig` appears only at the boundaries (engine, host, protocol, share links, sweeps), and the sugarscape panels read `Engine.sugar`; the new models get a schema-driven Rules panel, a model-aware display, a ring view over a host-rendered space–time diagram, their own charts and Inspect rows.

**Tech Stack:** Rust core (`sugarscape-core`), `wasm-bindgen` (`sugarscape-wasm`), the `sugarscape` CLI (clap), TypeScript + Vite + uPlot + Vitest. No new dependencies.

**Spec:** docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md (the milestone 1–8 specs in docs/superpowers/specs/ stay binding where not changed, in particular 2026-09-24-worker-simulation-design.md, 2026-09-24-sessions-compare-recording-design.md and 2026-09-23-experiments-design.md).

## Global Constraints

- **Sugarscape unchanged.** Every existing entry of `GOLDEN` in `crates/sugarscape-core/tests/golden.rs` and every legacy fixture (`crates/sugarscape-core/tests/fixtures/`) stays green and **unedited**; `tests/legacy.rs`, `tests/book.rs`'s existing tests and `tests/invariants.rs` are not edited at all (golden.rs gains a second table, `MODEL_GOLDEN`, which `print_golden` also prints; book.rs only gains tests). `presets::all()`, `presets::by_id()`, `Preset`, `Config` and `World` keep their signatures; every config, share link, session file and sweep written before this milestone reads as before (a config without a `model` key is a sugarscape config), and a sugarscape config serializes byte for byte as before (no `model` key).
- **Faithful to the book** where it is specific (quoted in Tasks 2 and 3); where it is silent, the choice is the spec's or a Decision below.
- **One engine path.** Host, engine, replay, Compare, sweeps and the CLI stay single-path over "a model with a config and named series"; no per-model copies of that machinery.
- **Deterministic and portable.** Each model is a function of (config, seed); golden entries pin every new preset. The WASM build must give the same fingerprints as the native build: never sample a `usize` range from the RNG (wasm32's `usize` is 32 bits, so `gen_range(0..n_usize)` draws differently there) — sample `u32` (Decision 9).
- **Performance:** Schelling's "every acceptable site" must stay cheap: a 50 × 50 / 2 000-agent tick well under 1 ms and a 200 × 200 / 32 000-agent tick under 10 ms in release (Decision 9 records the measurement that ruled out the naive scan).
- **Copy (verbatim):** presets-menu optgroups **Sugarscape**, **Schelling**, **Ring World** (then the existing **Compare**); Schelling color modes **Colour**, **Satisfaction**, **Preference**; chart titles **Segregation**, **Unsatisfied**, **Moves**, **Red share**, **Flocks**, **Flock size**, **Distance moved**; preset ids `vi-4-schelling-25`, `vi-5-schelling-25-residence`, `vi-6-schelling-50-residence`, `vi-7-schelling-mixed`, `vi-8-ring-world`, `vi-9-ring-megagroup`; built-in sweep id `schelling-tipping`; series `unsatisfied`, `segregation`, `moves`, `red_share`, `quiet`, `population` (Schelling) and `flocks`, `mean_flock`, `largest_flock`, `mean_distance`, `population` (Ring World), in those orders.
- **Names are binding across tasks** (each task's Interfaces block repeats the ones it uses): core `model::{ModelKind, ModelConfig, Model, ModelWorld, wrong_model}`, `ModelKind::{ALL, as_str, schema}`, `ModelConfig::{kind, sugarscape, from_json, from_value, validate, with_path, series_names}`, `ModelWorld::{new, with_landscapes, kind, model, model_mut, sugarscape, sugarscape_mut, ring}`, `stats::{Series, Stats<S>}`, `export::history_csv`, `presets::{ModelPreset, catalog, find}`, `schema::{Param, ParamKind, Apply, Choice}`, `schelling::{SchellingConfig, FRange, Residence, SchellingWorld, SchellingSnapshot, SchellingMode, Resident, SERIES, satisfied, schema, presets}`, `ring::{RingConfig, Start, RingWorld, RingSnapshot, Walker, HISTORY, AGENT, SERIES, flocks, schema, presets}`; WASM `presets_json` (catalog), `model_schemas_json`, `Sim::{model_kind, ring_sugar, ring_agents}`; web `ModelKind`, `ModelConfig`, `SchellingConfig`, `RingConfig`, `FRange`, `Param`, `ModelStats`, `SchellingStats`, `RingStats`, `AnyInspection`, `SchellingInspection`, `RingInspection` (types.ts), `MODELS`, `MODEL_LABELS`, `modelOf`, `isSugar`, `isSugarView`, `isRingView`, `presetModel`, `presetGroups`, `COLOR_MODES` (models.ts), `Wants.ring`, `RingState`, `WorldSnapshot.ring` (protocol.ts), `SimLike.{ring_sugar, ring_agents}`, `Engine.{model, sugar, ring, schemas, applyModelConfig, resetModelWith}`, `EngineDeps.schemas`, `groupParams`, `paramEdit`, `paramInput`, `ParamInput` (schema-form.ts), `SchemaPanel`, `RulesOptions`, `RING_HISTORY`, `siteAngle`, `siteAt`, `sugarShade` (ring.ts), `RingView`, `MODEL_CHARTS`, `ModelChart`, `ChartLine`, `showsForModel` (series-data.ts), `percent` (ui/format.ts), `defaultForm(model)`, `worldViews`, `WorldView`.
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3`; the commit commands below pass it as a second `-m`. Stage **only** the files named in the task (`git add <paths>`, never `-A`/`.`).
- Rust tasks finish with `cargo fmt --all && cargo clippy --all-targets -- -D warnings` before committing. Code in this plan is shown `rustfmt`-formatted where it matters; run `cargo fmt` anyway.
- Web tasks run `(cd web && npm run build && npm test)`. The build regenerates `web/src/wasm-pkg` (gitignored) with `wasm-pack` and runs `tsc --noEmit` then `vite build`; Vitest imports that package (determinism.test.ts), so always build before testing. Single test files run with `(cd web && npx vitest run src/<file>.test.ts)`.
- **TypeScript:** `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`. Unused imports fail the build — each task lists its import changes. Never pass a possibly-null child to `replaceChildren`/`append`: use `h()`, which skips `null`/`false`. Test names use the typographic apostrophe (’) inside single-quoted strings.
- **Browser checks are the controller's**, not the implementer's. Each task's "Browser (controller):" line lists the scenarios it affects; the controller runs its puppeteer pass on them (and the full list in Task 12). `?debug` exposes `window.sugarscape = { engine, compare }`.

## Why this task order

- **The refactor first (Task 1):** the model tag, `ModelConfig`, the `Model` trait, `ModelWorld`, the preset catalog, sweeps and the CLI over the trait — with only the sugarscape behind it. It changes no behavior, so golden, legacy and book tests pin it before anything new exists.
- **The two models in the core (Tasks 2–3)**, each with its unit tests, schema, presets, golden entries and measured book-style tests; **WASM (Task 4)** exposes them.
- **The page's plumbing (Task 5)** — the `ModelConfig` union at the boundaries, `Engine.sugar`, model-aware wants and display clamping — is again a no-change refactor for the sugarscape, with the new models reachable only through the engine (tests); the determinism tests run them through the real WASM.
- **Then the page, one reviewable piece at a time:** presets by model, the schema-driven Rules panel and one model per Compare (Task 6); display and views (Task 7); charts (Task 8); Inspect and the sugarscape-only tools (Task 9); Experiments, the CLI and the tipping sweep (Task 10); recording (Task 11); docs and the full verification (Task 12).

## Decisions (where the spec leaves room)

These are binding; each is repeated in the task that implements it.

1. **The model tag on the wire.** A config is a JSON object; its optional `model` key names the model. Absent or `"sugarscape"` → the sugarscape (`ModelConfig::from_value` removes the key and hands the rest to `Config::from_value`, legacy conversion included); `"schelling"` / `"ring"` → that model's config (`#[serde(default, deny_unknown_fields)]`: missing fields take the defaults, unknown fields are a `config` field error); anything else → field `model`, message `unknown model "<tag>" (expected sugarscape, schelling or ring)` (a non-string tag: `must be a model name, not <value>`). Serializing writes a sugarscape config **without** a `model` key (byte-identical to today) and the others with `"model"` first (serde's internally tagged enum `Tagged`). `with_path` can never change the model: `model` is not a field of any model's config (`unknown field model`).
2. **Presets.** `Preset`, `presets::all()` and `presets::by_id()` stay exactly as they are (sugarscape only), so the 48 call sites in the tests, the CLI tests and the WASM tests are untouched. `presets::catalog() -> Vec<ModelPreset>` lists every model's presets — the sugarscape's (in `all()` order, each serializing exactly as its `Preset`), then Schelling's, then Ring World's — and `presets::find(id)` searches it. The page (`presets_json`), sweeps (`base.preset`) and the CLI (`presets`, `run --preset`) use the catalog. The spec's "`presets()` lists all" is `presets_json()`/`catalog()`.
3. **The `Model` trait and `ModelWorld`.** `trait Model { config, run(ticks), tick, population, fingerprint, size, render(mode, layer, buf), latest_json, series_names, series(name), series_csv, agents_csv, inspect_json(x, y), locate(id), set_config(next) }`. `run` is the spec's `step(n)` (named after `World::run` so it cannot be confused with `World::step`, one tick); the spec's `stats()` is `latest_json()`; `schema()` lives on `ModelKind` because it depends only on the kind. `impl Model for World` delegates to the existing inherent methods (`World::run`, `World::fingerprint`, `render::render`, `export::series_csv`, …), so no sugarscape code path changes. `ModelWorld` is an enum of boxed worlds with `model()`/`model_mut()` (`&dyn Model`) and `sugarscape()`/`sugarscape_mut()`/`ring()` for model-specific calls. `Stats` becomes `Stats<S = Snapshot>` over a `Series` trait (`tick()`, `value(name)`), and `export::history_csv(names, history)` writes any model's statistics CSV; `series_csv(world)` calls it with exactly today's output. `ModelWorld::with_landscapes` passes the sugarscape's painted maps on and ignores them for the other models.
4. **Sweeps over any model.** `Sweep::config_for` and `run_config` take and return `ModelConfig`; `base.preset` looks up the catalog; `base.config` is read by `ModelConfig::from_value` (so a sweep over a Schelling config carries `"model": "schelling"`); `set` paths go through `ModelConfig::with_path` (the other models set a path through their JSON, with the same `schedule`-field error messages as `Config::with_path`); the metric's series is checked against `ModelConfig::series_names()`. Nothing else in the sweep format changes.
5. **WASM.** `Sim` holds a `ModelWorld`. `width`/`height` are the frame's size (`Model::size`: Ring World is `sites` × 150). Sugarscape-only calls answer empty for the other models — `lorenz`, `wealth_hist`, `age_hist`, `tag_hist`, `lorenz_total`, `supply_demand`, `trail`, `networks` → empty arrays; `credit_graph` → `{"agents":[],"loans":[]}`; `disease_list` → `[]`; `followed` → −1; `landscape_edited` → false; `follow`/`unfollow` do nothing — while edits and good-indexed calls are field errors (`edit`: `this world is not a sugarscape`, or `there is no good N` for `good_wealth_hist`/`export_landscape`/`set_landscape`). New: `model_kind()`, `ring_sugar()` / `ring_agents()` (empty unless Ring World), top-level `model_schemas_json()` (`{ "schelling": [...], "ring": [...] }`). `presets_json()` serializes the catalog; `config_series_names` accepts any model.
6. **The page's config types.** `Config` (the sugarscape's) is unchanged. `ModelConfig = Config | SchellingConfig | RingConfig` appears where configs cross boundaries: `Preset.config`, `InitialState`, `Session`, `ShareState`, `Command` (`init`/`reset`/`setConfig`), `WorldSnapshot.config`, `SweepBase`, `SessionSource.baseConfig`, `Engine.config`/`baseConfig`. `modelOf(c)` reads the tag (anything but `'schelling'`/`'ring'` is `'sugarscape'`) and `isSugar(c)` narrows. The sugarscape panels (Rules sections, Display's layers and overlays, tools, Inspect's agent rows, Credit, Charts' sugarscape charts) read **`Engine.sugar: Config`** — the live config while the world is a sugarscape, otherwise the last sugarscape config the engine ran (initially the first sugarscape preset's) — so they keep their code and never see another model's config; they are hidden while another model runs. `Engine.applyConfig`/`resetWith` keep their `(c: Config) => void` signatures and refuse another model's world with `[{ field: 'config', message: 'this world is a <model> world, not a sugarscape' }]`; `applyModelConfig`/`resetModelWith` take `(c: ModelConfig) => void` for the schema panel. `Engine.model` is `modelOf(config)`; `Engine.latest` becomes `ModelStats`. A thrown mutation (an unknown path) becomes a `config` field error instead of a rejected promise.
7. **Wants and display per model.** The host answers the sugarscape-only wants (`trail`, `networks`, `lorenz`, `wealthHist`, `ageHist`, `tagHist`, `lorenzTotal`, `goodWealthHists`, `supplyDemand`, `creditGraph`, `diseaseList`) only while the world is a sugarscape, and sends `editedLandscapes: []` for the others; `wants.ring` is answered only by Ring World (`WorldSnapshot.ring = { sugar: Float64Array, agents: Uint32Array }`). The engine's own wants ask for networks only in a sugarscape and for `ring` on every request in Ring World. `clampDisplay(d, config: ModelConfig)`: in a sugarscape a color mode that is not a sugarscape mode falls back to `tribe` (as does `disease` with disease off); Schelling keeps its modes (`color`, `satisfaction`, `preference`, default `color`); Ring World keeps whatever mode (it has none); every overlay is off in the other models; their layer is kept, unused. `ColorMode` gains `'color' | 'satisfaction' | 'preference'`; `COLOR_MODES` (models.ts) lists each model's modes with their labels.
8. **The schema and the schema-driven Rules panel.** `Param = { path, label, kind: integer | number | range | bool | choice, min?, max?, step?, choices?: {value, label}[], apply: live | reset, group }` (min/max/step only for the numeric kinds, choices only for `choice`). A `range` param sets `<path>.min` and `<path>.max` (so it works for `vision`, `preference` and `residence`, which also holds `enabled`); integer params and whole-step ranges round their inputs. Sections follow the groups in first-appearance order, each with a note: all reset → "Changing these rebuilds the world."; all live → "These apply to the running world."; mixed → "Fields marked ↺ rebuild the world; the rest apply to it as it runs." (reset fields marked ↺). Live fields call `Engine.applyModelConfig` (logged as `setConfig`, replayed by links); reset fields call `Engine.resetModelWith`. Schelling: every field is reset-only (agents draw their preference and residence when created; width, height and population shape the world) — Setup: Width, Height (5–200), Agents (0–39 999); Preference: "Like neighbors wanted (fraction)" (0–1, step 0.05); Residence: "Maximum residence" (bool) and "Residence (ticks)" (1–1000). Ring World: Setup (reset): Sites (10–1000), Agents (1–999), Start (choice "Scattered at random" / "One megagroup"); Agents (reset): "Vision (sites)" (1–100); Sugar (live): Capacity (1–20), "Growback per tick" (0.1–10, step 0.1). Each model's unit test checks every schema path exists and that `set_config` accepts exactly the live fields (`schema::check_schema`).
9. **Schelling, precisely.** Config `{ width, height, population, preference: {min, max}, residence: {enabled, min, max} }`, default = VI-4 (50 × 50, 2 000, 0.25–0.25, residence off 80–100). Validation: width and height 5–500; population < width × height; preference min and max in [0, 1] and min ≤ max (field `preference`); residence.min ≥ 1 and min ≤ max. **Setup**, in this RNG order: shuffle all site indices; for each of the first `population` sites, one agent: `red = gen_bool(0.5)`, `preference = gen_range(min..=max)` (drawn even when min = max), `residence = gen_range(min..=max)` only with residence on (else 0); ids 1, 2, … in that order; age 0. **Satisfaction**: over the occupied von Neumann neighbors, `like / occupied >= preference`; no neighbors → satisfied. **Step**: ids in a shuffled order; an unsatisfied agent (judged where it stands) **vacates its site**, then picks uniformly among the empty sites (not its own) at which it would be satisfied — its own site no longer counts as a neighbor — and moves there, or goes back if there is none; then every agent ages 1; with residence on, every agent with `age >= residence` (in id order) leaves: its site is vacated, a newcomer is drawn (color, preference, residence, as at setup, next id, age 0) and placed on a uniformly chosen empty site that satisfies it, else on a uniformly chosen empty site (the departed agent's site is a candidate). **Data structure** (the measurement that forced it: the naive "scan every site for every unsatisfied agent" took 14 ms for the first 50 × 50 tick and **11.9 s** (25 %) / **22 s** (50 %) for the first 200 × 200 / 32 000-agent tick, 0.9 s per tick afterwards with residence): per-site counts of Red and Blue occupied neighbors, and every empty site filed, for each color, under its (like, occupied) class — 15 classes, `occupied·(occupied+1)/2 + like` — so the acceptable sites for (color, preference) are the union of the classes whose `like/occupied` meets the preference (or `occupied = 0`). A pick draws `r = gen_range(0..total as u32)` and walks the classes in index order; a moving agent's own site is vacated but not filed while it chooses, so the pools then hold exactly "every unoccupied site at which it would be satisfied, not counting its current site". Same distribution as collecting and choosing; measured after: 0.11 ms per 50 × 50 tick (0.15 ms the first), 2.6–2.9 ms per 200 × 200 tick (3.3–4.4 ms the first). A unit test checks the pools against a direct count through 25 ticks of moves and departures. **Portability:** `gen_range` over a `usize` range differs between wasm32 and 64-bit targets (planning's first WASM fingerprint of `vi-4-schelling-25` differed from the native one); sample `u32`. **Statistics** (after the tick): `unsatisfied` = share unsatisfied (0 with no agents); `segregation` = mean like-share over agents with ≥ 1 neighbor (0 if none); `moves` = agents that moved this tick; `red_share`; `quiet` = 1 when a tick ran and nobody moved (0 at t = 0); `population`. **Render**: empty sites `BACKGROUND`; `color` Red `RED` / Blue `BLUE`; `satisfaction` satisfied agents their color at 30 % over the background, unsatisfied `BOTH` (yellow); `preference` `lerp(COOL, HOT, preference)`. **Fingerprint**: FNV-1a over the tick and, per agent in id order, id, site, color, preference bits, (age << 32 | residence). **Inspect**: `{ site: {x, y}, agent: {id, color, preference, satisfied, like, neighbors, age, residence|null} | null }`; agents CSV `id,x,y,color,preference,satisfied,age,residence`.
10. **Ring World, precisely.** Config `{ sites, agents, vision: {min, max}, capacity, growback, start: random | megagroup }`, default = VI-8 (150, 40, 15–30, 4, 1.0, random). Validation: sites 10–1000; agents < sites; vision.min ≥ 1, min ≤ max, max < sites; capacity 1–100; growback finite > 0. **Setup**, in this RNG order: each site's sugar `gen_range(0..=capacity)` in site order; then places — `random`: shuffle `0..sites` and take `agents`; `megagroup`: `from = gen_range(0..sites)`, sites `from, from+1, …` (mod sites); then each agent's vision `vision.sample(rng)` in place order; ids 1, 2, …. **Step**: ids shuffled; each agent looks at distances 1…vision counterclockwise (increasing index mod sites), skips occupied sites, and takes the nearest site with the most sugar (strictly more replaces, so the nearest wins ties; a maximum of 0 still counts); it moves there and eats all its sugar; with every site occupied it stays (and eats nothing). After everyone: every site grows back `growback`, capped at `capacity`. **Flocks** (the spec's definition): agents sorted by site; a flock is a maximal run whose consecutive gaps (around the ring) are ≤ 2; a lone agent is a flock of 1; with no gap > 2 everyone is one flock. **Statistics**: `flocks`, `mean_flock` (agents / flocks), `largest_flock`, `mean_distance` (sites moved per agent this tick, 0 at t = 0), `population`. **Space–time diagram** (the host frame, `size() = (sites, 150)`): the world keeps the last `HISTORY = 150` rows (each site's sugar / capacity, or an agent), recorded at setup and after every tick, so every tick appears even at Max; the current tick is the bottom row, rows before t = 0 are dark; sugar is `lerp(BACKGROUND, SUGAR, level)`, agents `AGENT` = the ramp's cool blue `#4f9dff` (planning tried near-white first: on full sugar it barely showed). History is observational: not hashed, not exported. **Inspect**: `inspect_json(x, _)` = site x (`{ site: {x, sugar, capacity}, agent: {id, vision} | null }`), the row is ignored; `locate(id)` = (site, 149), so a tracked agent's selection box sits on the current row. Fingerprint: tick, every site's sugar bits, per agent id and (site << 32 | vision). Agents CSV `id,site,vision`.
11. **One model per Compare.** Both worlds always run the same model. A's presets menu, choosing another model's preset while comparing, first leaves Compare keeping A (`RulesOptions.beforeModelChange`, the same path as opening a session file); if Compare cannot be left the menu snaps back. B's panel ("Rules for: B") lists only its own model's presets (`sameModelOnly`). A `#c=` link or a comparison session file whose two worlds are of different models is rejected as malformed (`not a SugarScape compare link` / `not a SugarScape session file`); `comparePresetStates` returns null for an entry whose presets differ in model. The Compare button's copy of A is the same model by construction.
12. **Views (spec "Page — Views").** The Agents menu offers the model's `COLOR_MODES` (hidden for Ring World); the Landscape menu and every overlay checkbox show only for a sugarscape. Schelling is the host-rendered grid (`GridView`, unchanged). Ring World: `RingView` (ui/ring-view.ts) draws a 600 px canvas above the grid (`#ring`, and `.ring-view` in B's figure): a dark square, a band of `sites` cells between radii 0.72 and 0.90 of the half-size shaded by sugar (`sugarShade`), each agent a blue dot at radius 0.64, the selected site outlined in `--accent`; site 0 is at the top and index increases counterclockwise on screen (`siteAngle(i, n) = −π/2 − 2πi/n`), the way the agents look and move. Its state comes from `Engine.ring` (the snapshot's `ring`). Below it `#grid` shows the space–time diagram. A click on the ring selects that site at the bottom row (`engine.select(site, height − 1)`) and shows Inspect; clicks on the diagram go through the Inspect tool as on any grid. `body[data-model]` follows A's model (CSS shrinks both canvases for Ring World).
13. **Charts, Inspect and hidden sugarscape UI.** Charts: `MODEL_CHARTS` (series-data.ts) — Schelling: **Segregation** (`segregation`, y 0–1), **Unsatisfied** (`unsatisfied`, 0–1), **Moves** (`moves`), **Red share** (`red_share`, 0–1); Ring World: **Flocks** (`flocks`), **Flock size** (`mean_flock` "Mean", `largest_flock` "Largest"), **Distance moved** (`mean_distance` "Sites per agent") — time charts on the existing machinery (downsampled groups, quiet when paused, Compare A solid / B dashed); a chart shows while a world on screen runs its model (`showsForModel`); the sugarscape's charts, sections and distributions only for a sugarscape. Inspect rows — Schelling: Site `(x, y)`, Agent `#id · Red|Blue`, Preference `at least 25% alike` (`percent`), Satisfied `yes|no (L of N neighbors alike | no neighbors)`, Residence `age A of R` or `age A (no maximum)`; a tracked agent that left shows `Agent #id has left.`; Ring World: Site `#x`, Sugar `s / capacity`, Agent `#id`, Vision `v sites`. Tools: only **Inspect** while no world on screen is a sugarscape (a chosen editing tool falls back to Inspect). The Credit tab needs a sugarscape with credit on; Follow exists only in sugarscape agent rows.
14. **Experiments and the CLI.** A sweep base may be any model's preset or config; the form's path suggestions (`numericPaths`, any object) and series list (`config_series_names`) follow the base's model; a new sweep's defaults follow the current world's model (`defaultForm(model)`: Schelling x `population` 1000:2400:200, final `segregation`, 200 ticks; Ring World x `agents` 10:70:10, window mean `flocks` from 400; the sugarscape's unchanged). `sugarscape presets` lists the catalog; `run --preset`/`--config` run any model (series and agents CSVs in the model's columns, `--config-out` tagged); `sweep` runs any model's sweeps.
15. **The `schelling-tipping` sweep** (sweeps/schelling-tipping.json, sixth built-in). Base `vi-4-schelling-25`; x "Like neighbors wanted (fixed preference)" = 0, 0.05, …, 0.6 (13 values, each setting `preference.min` and `preference.max`); final `segregation`; seeds 1–5; 50 ticks. Measured (release, seeds 1–5): mean final segregation 0.498 color-blind, 0.629 for 0.05–0.25, 0.731 at 0.30, 0.830 for 0.35–0.50, 0.934 for 0.55–0.60 — a staircase, because with four neighbors only the shares 1/4, 1/3, 1/2 and 2/3 can matter (0.05–0.25 give identical runs); every run was quiet by t = 7, and 50 is the smallest multiple of 50 at least five times that; per-cell sd ≤ 0.009, so 5 seeds suffice. The book test requires a rise of at least 0.40 (the measured 0.436 rounded down to a multiple of 0.05) and no step down.
16. **Recording.** `worldViews` (recording/frames.ts): a world records its grid; Ring World records its ring (as a square as tall as the diagram, 150 × 150 cells) beside its space–time diagram; in Compare A's views then B's, each world's first view tagged. Two 150-cell squares side by side record at 3 px a cell (904 × 450).
17. **Measured thresholds** (book-style, `#[ignore]`, release, seeds 1–5, rounded toward failing less by fixed rules; recorded 2026-09-25 in comments):
    - Schelling (`measure_schelling`): VI-4 first quiet at t = 2–3, segregation 0.489–0.509 → 0.618–0.639 (gains 0.122–0.150); late segregation (mean over t = 500–1000) VI-5 0.756–0.763, VI-6 0.943–0.950, VI-7 0.924–0.944 (VI-6 − VI-5 per seed 0.184–0.193). Thresholds: `VI4_QUIET_BY = 10` (twice the latest first quiet tick, up to a multiple of 10), `VI4_GAIN = 0.10`, `VI6_OVER_VI5 = 0.15`, `VI7_AT_LEAST = 0.90` (down to a multiple of 0.05), and VI-7 closer to VI-6 than to VI-5 on every seed. VI-5's 0.76 is above VI-4's 0.63 (the book calls the two comparable): recorded in VI-5's description.
    - Ring World (`measure_ring_world`): VI-8 starts as 20–25 flocks of 1.60–2.00 and settles (t = 500–1000) to 6.75–8.16 flocks of 5.06–6.09 (growth 2.66–3.30 ×); VI-9 starts as 1 flock of 40 and settles to 6.94–7.78 flocks, the largest never above 14 after t = 500. Thresholds: `RING_LATE_FLOCKS_AT_LEAST = 6.0`, `RING_FLOCK_GROWTH = 2.5`, `RING_MEGAGROUP_LARGEST_AT_MOST = 15.0`.
    - **Re-measuring:** run `cargo test -p sugarscape-core --release --test book measure_ -- --ignored --nocapture`, apply the rules above to the printed figures, and update the constants, their comments and the preset descriptions together. If a golden fingerprint changes, re-record it with `print_golden` (never edit a `GOLDEN` entry).
18. **Golden entries** (200 ticks from seed 1, `MODEL_GOLDEN` in golden.rs): `vi-4-schelling-25` 0x7a7072c3433f5f6f, `vi-5-schelling-25-residence` 0x9abe1c25e873debd, `vi-6-schelling-50-residence` 0x637412f7af91f684, `vi-7-schelling-mixed` 0x79346d2a338108cf, `vi-8-ring-world` 0x1c341361c466db90, `vi-9-ring-megagroup` 0x430d0c3b19b6e58e. The same values come out of the WASM build (wasm-pack test and determinism.test.ts check two of them), which is the portability check of Decision 9. If an implementation that follows this plan exactly gives different values, stop and report rather than re-recording: the RNG order is part of the specification.

## File Structure

```
crates/sugarscape-core/src/model.rs          NEW  ModelKind, ModelConfig, Model, ModelWorld (1); Schelling (2) and Ring (3) variants
crates/sugarscape-core/src/stats.rs          MOD  Series, Stats<S> (1)
crates/sugarscape-core/src/export.rs         MOD  history_csv (1)
crates/sugarscape-core/src/presets.rs        MOD  ModelPreset, catalog, find (1); + Schelling (2), Ring (3)
crates/sugarscape-core/src/sweep.rs          MOD  sweeps over ModelConfig (1); schelling-tipping built-in (10)
crates/sugarscape-core/src/lib.rs            MOD  pub mod model (1), schelling + schema (2), ring (3)
crates/sugarscape-core/src/schema.rs         NEW  Param, ParamKind, Apply, Choice, check_schema (2)
crates/sugarscape-core/src/schelling.rs      NEW  the Schelling model (2)
crates/sugarscape-core/src/ring.rs           NEW  Ring World (3)
crates/sugarscape-core/tests/golden.rs       MOD  MODEL_GOLDEN machinery (1), entries (2, 3)
crates/sugarscape-core/tests/book.rs         MOD  Schelling (2), Ring (3), tipping (10) book-style tests
crates/sugarscape-cli/src/main.rs            MOD  run any model (1)
crates/sugarscape-cli/tests/cli.rs           MOD  other models and the tipping sweep (10)
crates/sugarscape-wasm/src/lib.rs            MOD  Sim over ModelWorld, schemas, ring state (4)
crates/sugarscape-wasm/tests/web.rs          MOD  (4, 10)
sweeps/schelling-tipping.json                NEW  (10)
web/src/types.ts                             MOD  ModelConfig union, new stats/inspections, Param, ColorMode (5)
web/src/models.ts, models.test.ts            NEW  modelOf, isSugar, COLOR_MODES, presetGroups, … (5)
web/src/protocol.ts                          MOD  ModelConfig on the wire, Wants.ring, RingState (5)
web/src/layers.ts, layers.test.ts            MOD  clampDisplay per model (5)
web/src/sim-host.ts, sim-host.test.ts        MOD  sugarscape-only wants, ring state (5)
web/src/fake-sim.fixture.ts                  MOD  ring_sugar, ring_agents, model (5)
web/src/engine.ts, engine.test.ts            MOD  model, sugar, ring, schemas, applyModelConfig, resetModelWith (5)
web/src/share.ts, share.test.ts              MOD  ModelConfig (5); one model per comparison (6)
web/src/sessions.ts, sessions.test.ts        MOD  ModelConfig (5)
web/src/experiments/{types,form,view}.ts     MOD  ModelConfig (5); defaultForm(model) (10)
web/src/ui/{rules-panel,charts-panel,inspect-panel,display,tools}.ts, main.ts, compare/compare-view.ts  MOD  Engine.sugar (5)
web/src/determinism.test.ts, compare-presets.test.ts  MOD  (5; 8, 10)
web/src/schema-form.ts, schema-form.test.ts  NEW  (6)
web/src/ui/schema-panel.ts                   NEW  (6)
web/src/ui/rules-panel.ts                    MOD  presets by model, schema panel, RulesOptions (6)
web/src/compare-presets.ts                   MOD  one model per entry (6)
web/src/ring.ts, ring.test.ts                NEW  ring geometry and colors (7)
web/src/ui/ring-view.ts                      NEW  (7)
web/src/ui/display.ts                        MOD  per model (7)
web/index.html, web/src/style.css            MOD  ring canvas (7)
web/src/ui/series-data.ts, series-data.test.ts, ui/charts-panel.ts  MOD  model charts (8)
web/src/ui/format.ts, format.test.ts         MOD  percent (9)
web/src/ui/inspect-panel.ts, ui/tools.ts     MOD  Inspect per model, only Inspect for other models (9)
web/src/experiments/form.test.ts             MOD  (10)
web/src/recording/frames.ts, frames.test.ts  MOD  worldViews (11)
README.md, docs/roadmap.md                   MOD  (12)
```

---

### Task 1: The model tag, the `Model` trait and `ModelWorld` (sugarscape only)

*Mechanical (full code); a pure refactor — every existing test must stay green unedited, except the six `config_for` call sites in sweep.rs's own unit tests.* Browser (controller): nothing to check (no page change; the WASM crate compiles unchanged against the new sweep types).

**Files:**
- Create: `crates/sugarscape-core/src/model.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/stats.rs`, `crates/sugarscape-core/src/export.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`
- Test: `crates/sugarscape-core/src/model.rs` (unit tests), `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `Config::{from_value, validate, with_path}`, `World::{with_landscapes, run, fingerprint, population, set_config, inspect, locate, stats, torus}`, `render::{render, ColorMode, Layer}`, `export::{series_csv, agents_csv}`, `stats::series_names`.
- Produces:
  - `stats.rs`: `pub trait Series { fn tick(&self) -> u64; fn value(&self, name: &str) -> Option<f64>; }` (implemented by `Snapshot`); `pub struct Stats<S = Snapshot>` with `push`, `latest`, `history`, `series` for `S: Series + Default` and `Default` for every `S`.
  - `export.rs`: `pub fn history_csv<S: Series>(names: &[String], history: &[S]) -> String`.
  - `model.rs`: `pub enum ModelKind { Sugarscape }` with `as_str()`; `pub enum ModelConfig { Sugarscape(Config) }` with `kind()`, `sugarscape() -> Option<&Config>`, `from_json(&str) -> Result<Self, Vec<FieldError>>`, `from_value(Value) -> Result<Self, FieldError>`, `validate()`, `with_path(&str, &Value) -> Result<Self, FieldError>`, `series_names() -> Vec<String>`, `impl Serialize`, `impl From<Config>`; `pub trait Model` (Decision 3's methods); `impl Model for World`; `pub enum ModelWorld { Sugarscape(Box<World>) }` with `new(ModelConfig, u64)`, `with_landscapes(ModelConfig, u64, &[Option<Vec<f64>>])`, `kind()`, `model() -> &dyn Model`, `model_mut() -> &mut dyn Model`, `sugarscape() -> Option<&World>`, `sugarscape_mut() -> Option<&mut World>`.
  - `presets.rs`: `#[derive(Clone, Debug, Serialize)] pub struct ModelPreset { id, name, source, description: &'static str, config: ModelConfig }`, `impl From<Preset>`, `pub fn catalog() -> Vec<ModelPreset>`, `pub fn find(id: &str) -> Option<ModelPreset>`.
  - `sweep.rs`: `Sweep::config_for(&Point) -> Result<ModelConfig, Vec<FieldError>>`; `run_config(&Sweep, &Point, ModelConfig) -> RunResult`.
  - `golden.rs`: `MODEL_GOLDEN` (empty until Task 2) and `model_fingerprint(id)`.

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/model.rs` with its module doc and the tests only (the implementation comes in Step 3):
```rust
//! Model kinds (milestone 9): the sugarscape and the other artificial
//! societies of Chapter VI behind one config type and one trait, so the
//! worker host, sweeps and the CLI run any of them the same way. See
//! docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets;
    use serde_json::json;

    #[test]
    fn untagged_and_sugarscape_tagged_configs_are_sugarscape() {
        let plain = ModelConfig::from_json(r#"{"population": 100}"#).unwrap();
        let tagged =
            ModelConfig::from_json(r#"{"model": "sugarscape", "population": 100}"#).unwrap();
        assert_eq!(plain, tagged);
        assert_eq!(plain.kind(), ModelKind::Sugarscape);
        assert_eq!(plain.sugarscape().unwrap().population, 100);
    }

    #[test]
    fn sugarscape_configs_serialize_without_a_tag() {
        let c = presets::by_id("iv-3-trade").unwrap().config;
        let model = ModelConfig::from(c.clone());
        assert_eq!(
            serde_json::to_string(&model).unwrap(),
            serde_json::to_string(&c).unwrap()
        );
        assert!(!serde_json::to_string(&model).unwrap().contains("\"model\""));
    }

    #[test]
    fn unknown_models_are_field_errors() {
        let e = ModelConfig::from_json(r#"{"model": "boids"}"#).unwrap_err();
        assert_eq!(e[0].field, "model");
        assert!(e[0].message.contains("\"boids\""), "{e:?}");
        let e = ModelConfig::from_json(r#"{"model": 3}"#).unwrap_err();
        assert_eq!(e[0].field, "model");
    }

    #[test]
    fn with_path_cannot_change_the_model() {
        let c = ModelConfig::from(Config::default());
        assert!(c.with_path("model", &json!("schelling")).is_err());
        let next = c.with_path("population", &json!(10)).unwrap();
        assert_eq!(next.sugarscape().unwrap().population, 10);
    }

    #[test]
    fn a_sugarscape_model_world_is_the_world() {
        let config = presets::by_id("ii-2-unit").unwrap().config;
        let mut direct = World::new(config.clone(), 1).unwrap();
        let mut any = ModelWorld::new(config.into(), 1).unwrap();
        direct.run(50);
        any.model_mut().run(50);
        let m = any.model();
        assert_eq!(m.fingerprint(), direct.fingerprint());
        assert_eq!((m.tick(), m.population()), (50, direct.population()));
        assert_eq!(m.series("population"), direct.stats.series("population"));
        assert_eq!(m.series_csv(), export::series_csv(&direct));
        assert_eq!(m.size(), (50, 50));
        assert!(m.render("nope", "resource:0", &mut Vec::new()).is_err());
        assert!(any.sugarscape().is_some());
    }

    #[test]
    fn a_sugarscape_world_refuses_structural_changes() {
        let mut c = Config::default();
        c.goods[0].map = crate::config::Map::Flat { capacity: 4.0 };
        let mut any = ModelWorld::new(c.clone().into(), 1).unwrap();
        c.width = 60;
        let e = any.model_mut().set_config(c.into()).unwrap_err();
        assert_eq!(e[0].field, "width");
    }
}
```
In `crates/sugarscape-core/src/lib.rs`, after `mod legacy;` add `pub mod model;`.

In `crates/sugarscape-core/src/presets.rs`'s tests module, before `fn ids_are_unique_and_findable`, add:
```rust
    #[test]
    fn the_catalog_lists_every_sugarscape_preset_first_unchanged() {
        let catalog = catalog();
        let sugarscape = all();
        for (p, q) in sugarscape.iter().zip(&catalog) {
            assert_eq!(p.id, q.id);
            assert_eq!(q.config.sugarscape(), Some(&p.config));
            assert_eq!(
                serde_json::to_string(p).unwrap(),
                serde_json::to_string(q).unwrap()
            );
        }
        assert_eq!(
            find("iv-3-trade").unwrap().config,
            by_id("iv-3-trade").unwrap().config.into()
        );
        assert!(find("nope").is_none());
    }

```
In `crates/sugarscape-core/tests/golden.rs` (its `GOLDEN` table and existing tests stay exactly as they are): add `use sugarscape_core::model::ModelWorld;` above `use sugarscape_core::presets;`; after the `GOLDEN` table add
```rust
/// Other models (milestone 9): (preset id, fingerprint after 200 ticks from seed 1).
const MODEL_GOLDEN: &[(&str, u64)] = &[];
```
and replace the `print_golden` test (its doc comment included) with
```rust
/// Any model's preset `id` after 200 ticks from seed 1.
fn model_fingerprint(id: &str) -> u64 {
    let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
    let mut world = ModelWorld::new(preset.config, 1).unwrap();
    world.model_mut().run(200);
    world.model().fingerprint()
}

#[test]
fn other_models_are_unchanged() {
    for &(id, expected) in MODEL_GOLDEN {
        assert_eq!(model_fingerprint(id), expected, "preset {id} changed");
    }
}

#[test]
fn every_model_preset_has_a_golden_entry() {
    for p in presets::catalog() {
        assert!(
            GOLDEN.iter().chain(MODEL_GOLDEN).any(|&(id, _)| id == p.id),
            "record a golden fingerprint for {} (run print_golden)",
            p.id
        );
    }
}

/// Prints `GOLDEN` entries, then `MODEL_GOLDEN`'s:
/// `cargo test -p sugarscape-core --test golden -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_golden() {
    for p in presets::all() {
        println!("    (\"{}\", {:#x}),", p.id, fingerprint(p.id));
    }
    println!("MODEL_GOLDEN:");
    for p in presets::catalog()
        .iter()
        .filter(|p| p.config.sugarscape().is_none())
    {
        println!("    (\"{}\", {:#x}),", p.id, model_fingerprint(p.id));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core --lib model`
Expected: FAIL to compile — `ModelConfig`, `ModelKind`, `ModelWorld` not found (and `catalog`, `find` in presets.rs).

- [ ] **Step 3: Implement**

`crates/sugarscape-core/src/stats.rs` — replace the `Stats` struct and its `impl` (from `#[derive(Clone, Debug, Default)]\npub struct Stats {` through the end of `impl Stats { … }`) with (Decision 3):
```rust
/// One tick's statistics of any model: its tick and its series by name.
pub trait Series {
    fn tick(&self) -> u64;
    /// Series `name` (or `"tick"`), or `None` if the model has no such series.
    fn value(&self, name: &str) -> Option<f64>;
}

impl Series for Snapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Snapshot::value(self, name)
    }
}

/// A model's statistics history, one snapshot per tick from 0.
#[derive(Clone, Debug)]
pub struct Stats<S = Snapshot> {
    history: Vec<S>,
}

impl<S> Default for Stats<S> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
        }
    }
}

impl<S: Series + Default> Stats<S> {
    pub fn push(&mut self, s: S) {
        self.history.push(s);
    }

    pub fn latest(&self) -> Option<&S> {
        self.history.last()
    }

    pub fn history(&self) -> &[S] {
        &self.history
    }

    /// The full history of one series (or `"tick"`), or `None` if unknown.
    pub fn series(&self, name: &str) -> Option<Vec<f64>> {
        if self.history.is_empty() {
            return S::default().value(name).map(|_| Vec::new());
        }
        self.history.iter().map(|s| s.value(name)).collect()
    }
}
```
(`World`'s field `pub stats: Stats` is now `Stats<Snapshot>`; `Stats::default()` there needs no change.)

`crates/sugarscape-core/src/export.rs` — replace `use crate::world::World;` and `series_csv` with:
```rust
use crate::stats::Series;
use crate::world::World;

pub fn series_csv(world: &World) -> String {
    history_csv(
        &crate::stats::series_names(&world.config),
        world.stats.history(),
    )
}

/// Any model's statistics CSV: `tick`, then `names`, one row per snapshot.
pub fn history_csv<S: Series>(names: &[String], history: &[S]) -> String {
    let mut out = String::from("tick");
    for name in names {
        out.push(',');
        out.push_str(name);
    }
    out.push('\n');
    for s in history {
        out.push_str(&s.tick().to_string());
        for name in names {
            write!(out, ",{}", s.value(name).expect("known series")).unwrap();
        }
        out.push('\n');
    }
    out
}
```

`crates/sugarscape-core/src/model.rs` — between the module doc and the tests, add (Decisions 1 and 3):
```rust
use serde::{Serialize, Serializer};

use crate::config::{Config, FieldError};
use crate::render::{self, ColorMode, Layer};
use crate::world::World;
use crate::{export, stats};

/// Which model a config or world is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Sugarscape,
}

impl ModelKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ModelKind::Sugarscape => "sugarscape",
        }
    }
}

/// A config of any model. On the wire it is the model's own config object;
/// every model but the sugarscape carries `"model": "<kind>"`, and an object
/// without a `model` key (every config, link, session and sweep written
/// before milestone 9) is a sugarscape config.
#[derive(Clone, Debug, PartialEq)]
pub enum ModelConfig {
    Sugarscape(Config),
}

impl From<Config> for ModelConfig {
    fn from(c: Config) -> Self {
        ModelConfig::Sugarscape(c)
    }
}

impl Serialize for ModelConfig {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            // Untagged, so sugarscape configs serialize exactly as before.
            ModelConfig::Sugarscape(c) => c.serialize(s),
        }
    }
}

impl ModelConfig {
    pub fn kind(&self) -> ModelKind {
        match self {
            ModelConfig::Sugarscape(_) => ModelKind::Sugarscape,
        }
    }

    /// The sugarscape config, if this is one.
    pub fn sugarscape(&self) -> Option<&Config> {
        match self {
            ModelConfig::Sugarscape(c) => Some(c),
        }
    }

    /// Parses and validates a config of any model.
    pub fn from_json(json: &str) -> Result<Self, Vec<FieldError>> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| vec![FieldError::new("config", e.to_string())])?;
        let config = Self::from_value(value).map_err(|e| vec![e])?;
        config.validate()?;
        Ok(config)
    }

    /// Reads a config of any model by its `model` key: absent or
    /// `"sugarscape"` is a sugarscape config in either shape
    /// (`Config::from_value`).
    pub fn from_value(mut value: serde_json::Value) -> Result<Self, FieldError> {
        let tag = match value.as_object_mut().and_then(|o| o.remove("model")) {
            None => "sugarscape".to_string(),
            Some(serde_json::Value::String(tag)) => tag,
            Some(other) => {
                return Err(FieldError::new(
                    "model",
                    format!("must be a model name, not {other}"),
                ))
            }
        };
        match tag.as_str() {
            "sugarscape" => Config::from_value(value).map(ModelConfig::Sugarscape),
            _ => Err(FieldError::new(
                "model",
                format!("unknown model {tag:?} (expected sugarscape)"),
            )),
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        match self {
            ModelConfig::Sugarscape(c) => c.validate(),
        }
    }

    /// A copy with one dotted `path` set to `value` (the model cannot be
    /// changed this way: `model` is not a field of any model's config).
    pub fn with_path(&self, path: &str, value: &serde_json::Value) -> Result<Self, FieldError> {
        match self {
            ModelConfig::Sugarscape(c) => c.with_path(path, value).map(ModelConfig::Sugarscape),
        }
    }

    /// The statistics series a world with this config records.
    pub fn series_names(&self) -> Vec<String> {
        match self {
            ModelConfig::Sugarscape(c) => stats::series_names(c),
        }
    }
}

/// What the host, sweeps and the CLI need from a running model. `run` is
/// the spec's `step(n)`, named after `World::run` so it cannot be mistaken
/// for `World::step` (one tick).
pub trait Model {
    /// The live config (after scheduled changes and `set_config`).
    fn config(&self) -> ModelConfig;
    /// Runs `ticks` ticks.
    fn run(&mut self, ticks: u32);
    /// Completed ticks.
    fn tick(&self) -> u64;
    fn population(&self) -> usize;
    /// A hash of the full dynamic state (the golden tests' fingerprint).
    fn fingerprint(&self) -> u64;
    /// The rendered frame's width and height in cells.
    fn size(&self) -> (u32, u32);
    /// Renders the frame as RGBA into `buf` (`size()` cells). `mode` and
    /// `layer` name the model's color mode and landscape layer; models
    /// without them ignore them.
    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String>;
    /// The latest statistics snapshot as JSON.
    fn latest_json(&self) -> String;
    fn series_names(&self) -> Vec<String>;
    /// The full history of series `name` (or `"tick"`), or `None` if unknown.
    fn series(&self, name: &str) -> Option<Vec<f64>>;
    /// The statistics history as CSV (`tick`, then `series_names`).
    fn series_csv(&self) -> String;
    /// The agents alive now as CSV.
    fn agents_csv(&self) -> String;
    /// The site (x, y) and its agent as JSON.
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String>;
    /// Where agent `id` is, in frame cells, while it lives.
    fn locate(&self, id: u64) -> Option<(u32, u32)>;
    /// Applies a changed config to the running world; fields that change
    /// only on reset are refused.
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>>;
}

impl Model for World {
    fn config(&self) -> ModelConfig {
        ModelConfig::Sugarscape(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        World::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        World::population(self)
    }

    fn fingerprint(&self) -> u64 {
        World::fingerprint(self)
    }

    fn size(&self) -> (u32, u32) {
        (self.torus.width, self.torus.height)
    }

    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ColorMode = mode.parse()?;
        let layer: Layer = layer.parse()?;
        render::render(self, mode, layer, buf)
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        stats::series_names(&self.config)
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::series_csv(self)
    }

    fn agents_csv(&self) -> String {
        export::agents_csv(self)
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        World::locate(self, id).map(|p| (p.x, p.y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        match next {
            ModelConfig::Sugarscape(c) => World::set_config(self, c),
        }
    }
}

/// A world of any model. Sugarscape-only calls (painting, networks, the
/// credit graph …) go through `sugarscape()`/`sugarscape_mut()`.
pub enum ModelWorld {
    Sugarscape(Box<World>),
}

impl ModelWorld {
    pub fn new(config: ModelConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        Self::with_landscapes(config, seed, &[])
    }

    /// Like `new`; `landscapes` are the sugarscape's painted maps
    /// (`World::with_landscapes`). Other models have no landscapes and ignore them.
    pub fn with_landscapes(
        config: ModelConfig,
        seed: u64,
        landscapes: &[Option<Vec<f64>>],
    ) -> Result<Self, Vec<FieldError>> {
        Ok(match config {
            ModelConfig::Sugarscape(c) => {
                ModelWorld::Sugarscape(Box::new(World::with_landscapes(c, seed, landscapes)?))
            }
        })
    }

    pub fn kind(&self) -> ModelKind {
        match self {
            ModelWorld::Sugarscape(_) => ModelKind::Sugarscape,
        }
    }

    pub fn model(&self) -> &dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_ref(),
        }
    }

    pub fn model_mut(&mut self) -> &mut dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_mut(),
        }
    }

    pub fn sugarscape(&self) -> Option<&World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
        }
    }

    pub fn sugarscape_mut(&mut self) -> Option<&mut World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
        }
    }
}
```

`crates/sugarscape-core/src/presets.rs` — add `use crate::model::ModelConfig;` after the `use crate::config::{…};` block, and after `by_id` add (Decision 2):
```rust
/// A preset of any model: what the page's presets menu, sweeps and the CLI
/// list. A sugarscape preset's config serializes exactly as its `Preset`'s.
#[derive(Clone, Debug, Serialize)]
pub struct ModelPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    pub description: &'static str,
    pub config: ModelConfig,
}

impl From<Preset> for ModelPreset {
    fn from(p: Preset) -> Self {
        ModelPreset {
            id: p.id,
            name: p.name,
            source: p.source,
            description: p.description,
            config: p.config.into(),
        }
    }
}

/// Every model's presets: the sugarscape's (`all`) first.
pub fn catalog() -> Vec<ModelPreset> {
    all().into_iter().map(ModelPreset::from).collect()
}

/// The preset `id` of any model.
pub fn find(id: &str) -> Option<ModelPreset> {
    catalog().into_iter().find(|p| p.id == id)
}
```

`crates/sugarscape-core/src/sweep.rs` (Decision 4):
- Replace the imports `use crate::config::{Config, FieldError};`, `use crate::world::World;`, `use crate::{presets, stats};` with
  ```rust
  use crate::config::FieldError;
  use crate::model::{ModelConfig, ModelWorld};
  use crate::presets;
  ```
- `Base`'s doc becomes `/// Where every run's config starts: a preset of any model, or a config of any\n/// model (a sugarscape config in either shape).`
- In `prepare`, `config_for`, `base_config` and `config_at`, every `Config` in a return type becomes `ModelConfig` (`Result<(Vec<Point>, Vec<ModelConfig>), Vec<FieldError>>`, `Result<ModelConfig, Vec<FieldError>>`).
- In `base_config`: `presets::by_id(id)` → `presets::find(id)`; `Config::from_value(value.clone())` → `ModelConfig::from_value(value.clone())`.
- In `config_at`: `if !stats::series_names(&config).iter().any(|n| n == name) {` → `if !config.series_names().iter().any(|n| n == name) {`.
- Replace the start of `run_config` with
  ```rust
  pub fn run_config(sweep: &Sweep, point: &Point, config: ModelConfig) -> RunResult {
      let mut world = ModelWorld::new(config, point.seed).expect("sweep configs are validated");
      world.model_mut().run(sweep.ticks);
      let history = world
          .model()
          .series(sweep.metric.series())
          .expect("the metric's series is checked against every config");
  ```
- Tests module: after `use super::*;` add `use crate::config::Config;` and `use crate::world::World;`; after `pub(crate) fn sweep(value: Value) -> Sweep { … }` add
  ```rust
      /// Point `point`'s config, which must be a sugarscape config.
      fn sugarscape(s: &Sweep, point: &Point) -> Config {
          s.config_for(point)
              .unwrap()
              .sugarscape()
              .cloned()
              .expect("a sugarscape config")
      }
  ```
  and replace the six uses: `let config = s.config_for(point).unwrap();` → `let config = sugarscape(&s, point);` (tag_length test); `s.config_for(&s.point(0).unwrap()).unwrap()` → `sugarscape(&s, &s.point(0).unwrap())` (twice: `first` and the either-shape test's `config`); `s.config_for(&s.point(11).unwrap()).unwrap()` → `sugarscape(&s, &s.point(11).unwrap())`; `World::new(s.config_for(&point).unwrap(), 6)` → `World::new(sugarscape(&s, &point), 6)`; and in `four_goods_with_trade_is_the_n_4_peaks_preset` `s.config_for(&point).unwrap(),` → `sugarscape(&s, &point),`.

`crates/sugarscape-cli/src/main.rs` (Decision 14):
- Replace the imports `use sugarscape_core::config::{Config, FieldError};`, `use sugarscape_core::sweep::{self, Sweep};`, `use sugarscape_core::world::World;`, `use sugarscape_core::{export, presets};` with
  ```rust
  use sugarscape_core::config::FieldError;
  use sugarscape_core::model::{ModelConfig, ModelWorld};
  use sugarscape_core::presets;
  use sugarscape_core::sweep::{self, Sweep};
  ```
- The `Presets` doc becomes `/// List the presets of every model (id, source, name).`; `ConfigSource.config`'s doc becomes `/// A config JSON file of any model (a sugarscape config in the current or pre-N-goods shape).`
- In `run`: `for p in presets::all() {` → `for p in presets::catalog() {`.
- In `run_world`: `presets::by_id(id)` → `presets::find(id)`; `Config::from_json(&read(path)?)?` → `ModelConfig::from_json(&read(path)?)?`; and replace
  ```rust
      let mut world = World::new(config.clone(), args.seed)?;
      world.run(args.ticks);
      if let Some(path) = &args.series_csv {
          write(path, &export::series_csv(&world))?;
      }
      if let Some(path) = &args.agents_csv {
          write(path, &export::agents_csv(&world))?;
      }
  ```
  with
  ```rust
      let mut world = ModelWorld::new(config.clone(), args.seed)?;
      world.model_mut().run(args.ticks);
      let world = world.model();
      if let Some(path) = &args.series_csv {
          write(path, &world.series_csv())?;
      }
      if let Some(path) = &args.agents_csv {
          write(path, &world.agents_csv())?;
      }
  ```
  (`--config-out` serializes the `ModelConfig`: identical JSON for a sugarscape config; `world.fingerprint()` is now the trait's.)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --workspace`
Expected: PASS — the new model and catalog tests, every sweep test, the CLI tests (`run_prints_the_golden_fingerprint_and_writes_its_files` still prints `0x75b93943813545e4` and writes the same config), golden (`earlier_presets_are_unchanged`, `every_preset_has_a_golden_entry`, `other_models_are_unchanged`, `every_model_preset_has_a_golden_entry`) and legacy — none of them edited beyond this task's lines.
Run: `cargo test -p sugarscape-core --release --test book -- --ignored`
Expected: PASS (every book test unchanged).
Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS (the WASM crate compiles against `ModelConfig` through `config_for`/`run_config` and is otherwise unchanged).

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/stats.rs crates/sugarscape-core/src/export.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/sweep.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-cli/src/main.rs
git commit -m "Add the model tag, the Model trait and ModelWorld over the sugarscape" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---


### Task 2: The Schelling model in the core

*Needs judgement: the implementation is given in full, but Step 7's golden values and Step 8's thresholds come from runs (Decisions 9, 17, 18); if a measurement differs, apply the fixed rules and record what you measured.* Browser (controller): nothing to check (not reachable from the page yet).

The book (Chapter VI, "A Variant of Schelling's Segregation Model"): "every agent is a member of one or another group (here either Red or Blue) and has a fixed preference for like-colored neighbors. Here, a preference is simply a minimum percentage … The agent computes the fraction of neighbors who are its own color; If this number is greater than or equal to its preference the agent is considered satisfied … we use the von Neumann neighborhood; he moves agents to the nearest satisfactory site, whereas our agents simply select an acceptable site at random; his landscape has a finite boundary, whereas ours is a torus. As a first example of this model we randomly populate a 50 × 50 lattice (2500 sites) with 2000 Red and Blue agents in approximately equal numbers … Each agent wishes at least 25 percent of its neighbors to be of its own color" (VI-4); "all agents are given a randomly assigned maximum lifetime between 80 and 100 time periods. Once an agent reaches its maximum age it is removed and the population is kept constant by replacing it with a new agent of random color. This agent is placed at a randomly selected position satisfying its preference for neighbors" (VI-5; note 11: "maximum lifetime" is "the point at which an agent decides to move to another landscape altogether; maximum residence duration is an equivalent notion"); "giving all agents the preference that at least 50 percent of their neighbors be of their own color" (VI-6); "we distribute agent preferences for like neighbors uniformly between 25 percent and 50 percent" (VI-7).

**Files:**
- Create: `crates/sugarscape-core/src/schema.rs`, `crates/sugarscape-core/src/schelling.rs`
- Modify: `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: `model::{Model, ModelConfig, ModelKind, ModelWorld}` (Task 1), `stats::{Series, Stats}`, `export::history_csv`, `presets::ModelPreset`, `geometry::{Pos, Torus}` (`Torus::neighbors` is N, S, E, W), `render::{lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, RED}`, `rng::{seeded, SimRng}`.
- Produces:
  - `schema.rs`: `ParamKind { Integer, Number, Range, Bool, Choice }`, `Apply { Live, Reset }` (both serialize snake_case), `Choice { value, label }`, `Param { path, label, kind, min, max, step: Option<f64>, choices: Vec<Choice>, apply, group }` with constructors `Param::integer(group, path, label, (min, max): (u32, u32), apply)`, `Param::number(group, path, label, (min, max, step), apply)`, `Param::range(group, path, label, (min, max, step), apply)`, `Param::bool(group, path, label, apply)`, `Param::choice(group, path, label, &[(value, label)], apply)`; `#[cfg(test)] pub(crate) fn check_schema(schema, config, world: impl Fn() -> ModelWorld)`.
  - `schelling.rs`: `FRange { min: f64, max: f64 }`, `Residence { enabled, min: u32, max: u32 }`, `SchellingConfig { width, height, population, preference: FRange, residence: Residence }` (`Default` = VI-4, `validate()`), `SERIES: [&str; 6]`, `SchellingSnapshot`, `Resident { id, pos, red, preference, age, residence }`, `SchellingInspection`, `SchellingMode` (`color` | `satisfaction` | `preference`), `pub fn satisfied(like: u32, occupied: u32, preference: f64) -> bool`, `SchellingWorld::{new, agents, agent_at, is_satisfied, step, run, inspect}` and `pub stats: Stats<SchellingSnapshot>`, `impl Model for SchellingWorld`, `pub fn schema() -> Vec<Param>`, `pub fn presets() -> Vec<ModelPreset>`.
  - `model.rs`: `ModelKind::Schelling`, `ModelKind::ALL`, `ModelKind::schema()`, `ModelConfig::Schelling(SchellingConfig)`, `ModelWorld::Schelling(Box<SchellingWorld>)`, `pub(crate) fn wrong_model(want: ModelKind, got: &ModelConfig) -> Vec<FieldError>`.

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/schelling.rs` with its module doc and the tests only (the implementation comes in Step 3):
```rust
//! The book's variant of Schelling's segregation model (Chapter VI,
//! animations VI-4 to VI-7): Red and Blue agents on a torus, each wanting at
//! least a fraction of its von Neumann neighbors to share its color, moving
//! to a random acceptable site when unsatisfied.

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w` × `h` world with nobody on it (agents are placed by `put`).
    fn empty(w: u32, h: u32) -> SchellingWorld {
        SchellingWorld::new(
            SchellingConfig {
                width: w,
                height: h,
                population: 0,
                ..SchellingConfig::default()
            },
            1,
        )
        .unwrap()
    }

    fn put(w: &mut SchellingWorld, x: u32, y: u32, red: bool, preference: f64) -> u64 {
        let mut a = w.newcomer(Pos::new(x, y));
        a.red = red;
        a.preference = preference;
        w.insert(a);
        a.id
    }

    fn agent(w: &SchellingWorld, id: u64) -> Resident {
        w.agents[&id]
    }

    /// The reference the pools must match: every empty site (in site order)
    /// where an agent of color `red` with `preference` would be satisfied,
    /// counted from the grid; `skip` (its own site) is neither a neighbor
    /// nor an option.
    fn brute(w: &SchellingWorld, red: bool, preference: f64, skip: Option<Pos>) -> Vec<Pos> {
        (0..w.torus.len())
            .map(|i| w.torus.pos(i))
            .filter(|&p| w.agent_at(p).is_none() && Some(p) != skip)
            .filter(|&p| {
                let (mut like, mut occupied) = (0, 0);
                for n in w.torus.neighbors(p) {
                    if Some(n) == skip {
                        continue;
                    }
                    if let Some(other) = w.agent_at(n) {
                        occupied += 1;
                        like += u32::from(other.red == red);
                    }
                }
                satisfied(like, occupied, preference)
            })
            .collect()
    }

    /// The filed sites an agent of color `red` with `preference` may pick, in site order.
    fn pooled(w: &SchellingWorld, red: bool, preference: f64) -> Vec<Pos> {
        let ok = acceptable_classes(preference);
        let mut out: Vec<Pos> = (0..CLASSES)
            .filter(|&k| ok[k])
            .flat_map(|k| {
                w.pools[usize::from(red)][k]
                    .iter()
                    .map(|&i| w.torus.pos(i as usize))
            })
            .collect();
        out.sort_by_key(|p| w.torus.index(*p));
        out
    }

    #[test]
    fn the_pools_match_a_direct_count_as_agents_move_and_leave() {
        let mut c = SchellingConfig {
            width: 12,
            height: 9,
            population: 80,
            preference: FRange { min: 0.2, max: 0.8 },
            ..SchellingConfig::default()
        };
        c.residence = Residence {
            enabled: true,
            min: 2,
            max: 6,
        };
        let mut w = SchellingWorld::new(c, 11).unwrap();
        for _ in 0..25 {
            for red in [false, true] {
                for p in [0.0, 0.25, 1.0 / 3.0, 0.5, 0.75, 1.0] {
                    assert_eq!(
                        pooled(&w, red, p),
                        brute(&w, red, p, None),
                        "t={} red={red} p={p}",
                        w.tick
                    );
                }
            }
            for a in w.agents() {
                let (like, occupied) = w.neighbors(a);
                let direct = w
                    .torus
                    .neighbors(a.pos)
                    .into_iter()
                    .filter_map(|n| w.agent_at(n))
                    .fold((0, 0), |(l, o), b| (l + u32::from(b.red == a.red), o + 1));
                assert_eq!((like, occupied), direct);
            }
            w.step();
        }
        // While an agent considers moving its site is vacated and unfiled:
        // the pools then hold exactly the sites acceptable without counting it.
        let a = *w.agents().next().unwrap();
        let from = w.torus.index(a.pos);
        w.vacate(from, a.red);
        for p in [0.25, 0.5, 1.0] {
            assert_eq!(pooled(&w, a.red, p), brute(&w, a.red, p, Some(a.pos)));
        }
    }

    #[test]
    fn satisfaction_counts_occupied_neighbors_and_ties_satisfy() {
        assert!(satisfied(0, 0, 1.0), "no neighbors: satisfied");
        assert!(satisfied(1, 4, 0.25), "exactly at the threshold");
        assert!(!satisfied(0, 4, 0.25));
        assert!(satisfied(2, 4, 0.5) && !satisfied(1, 3, 0.5));
        let mut w = empty(10, 10);
        let me = put(&mut w, 5, 5, true, 0.5);
        put(&mut w, 5, 4, false, 0.5);
        put(&mut w, 6, 6, false, 0.5); // diagonal: not a neighbor
        assert!(!w.is_satisfied(&agent(&w, me)), "0 of 1 alike");
        put(&mut w, 4, 5, true, 0.5);
        assert!(w.is_satisfied(&agent(&w, me)), "1 of 2 alike meets 50%");
    }

    #[test]
    fn acceptable_sites_do_not_count_the_movers_own_site() {
        // A Red wanting 50% at (2, 2) and a Blue at (4, 2). At (3, 2) its own
        // site would be a like neighbor (1 of 2); not counted, it has 0 of 1.
        let mut w = empty(7, 7);
        let me = put(&mut w, 2, 2, true, 0.5);
        put(&mut w, 4, 2, false, 0.0);
        let a = agent(&w, me);
        assert!(!brute(&w, a.red, a.preference, Some(a.pos)).contains(&Pos::new(3, 2)));
        assert!(brute(&w, a.red, a.preference, None).contains(&Pos::new(3, 2)));
        let options = brute(&w, a.red, a.preference, Some(a.pos));
        assert!(
            !options.contains(&Pos::new(2, 2)),
            "occupied sites are never options"
        );
        assert!(
            options.contains(&Pos::new(0, 0)),
            "a site with no neighbors is acceptable"
        );
    }

    #[test]
    fn an_unsatisfied_agent_with_no_acceptable_site_stays() {
        // 5 × 5 of Blues but a Red wanting 100% at (2, 2) and three empty
        // sites, each next to Blues only.
        let mut w = empty(5, 5);
        let mut me = 0;
        for y in 0..5 {
            for x in 0..5 {
                match (x, y) {
                    (2, 2) => me = put(&mut w, x, y, true, 1.0),
                    (0, 0) | (4, 4) | (0, 4) => {}
                    _ => {
                        put(&mut w, x, y, false, 0.0);
                    }
                }
            }
        }
        assert!(!w.is_satisfied(&agent(&w, me)));
        assert!(brute(&w, true, 1.0, Some(Pos::new(2, 2))).is_empty());
        w.step();
        assert_eq!(agent(&w, me).pos, Pos::new(2, 2));
        let s = w.stats.latest().unwrap();
        assert_eq!((s.moves, s.quiet), (0, 1));
    }

    #[test]
    fn a_mover_picks_among_every_acceptable_site_at_random() {
        let mut seen = std::collections::BTreeSet::new();
        for seed in 1..=40 {
            let mut w = empty(6, 6);
            w.rng = rng::seeded(seed);
            let me = put(&mut w, 0, 0, true, 1.0);
            put(&mut w, 1, 0, false, 0.0);
            w.step();
            seen.insert(agent(&w, me).pos);
        }
        // Any empty site not next to the Blue at (1, 0) is acceptable: many are chosen.
        assert!(seen.len() > 10, "{seen:?}");
        for p in seen {
            assert!(
                ![
                    Pos::new(0, 0),
                    Pos::new(2, 0),
                    Pos::new(1, 1),
                    Pos::new(1, 5)
                ]
                .contains(&p),
                "{p:?}"
            );
        }
    }

    #[test]
    fn residence_replaces_agents_and_keeps_the_population() {
        let mut c = SchellingConfig {
            width: 20,
            height: 20,
            population: 300,
            ..SchellingConfig::default()
        };
        c.residence = Residence {
            enabled: true,
            min: 3,
            max: 5,
        };
        let mut w = SchellingWorld::new(c, 7).unwrap();
        let first: Vec<u64> = w.agents().map(|a| a.id).collect();
        assert!(w
            .agents()
            .all(|a| (3..=5).contains(&a.residence) && a.age == 0));
        w.run(5);
        assert_eq!(w.agents().count(), 300);
        assert!(
            w.agents().all(|a| !first.contains(&a.id)),
            "everyone left by t = 5"
        );
        assert!(w.agents().all(|a| a.age < a.residence));
        let mut sites: Vec<Pos> = w.agents().map(|a| a.pos).collect();
        sites.sort();
        sites.dedup();
        assert_eq!(sites.len(), 300, "one agent per site");
        for (i, slot) in w.grid.iter().enumerate() {
            if let Some(id) = slot {
                assert_eq!(w.agents[id].pos, w.torus.pos(i));
            }
        }
    }

    #[test]
    fn a_newcomer_is_placed_where_it_is_satisfied_when_possible() {
        let mut w = empty(5, 5);
        let old = put(&mut w, 2, 2, true, 1.0);
        for (x, y) in [(0, 0), (4, 4)] {
            put(&mut w, x, y, false, 0.0);
        }
        w.agents.get_mut(&old).unwrap().residence = 1;
        w.config.residence = Residence {
            enabled: true,
            min: 5,
            max: 5,
        };
        w.config.preference = FRange { min: 1.0, max: 1.0 };
        w.step();
        assert!(!w.agents.contains_key(&old), "it left at age 1");
        let new = w.agents().find(|a| a.preference == 1.0).unwrap();
        assert!(w.is_satisfied(new), "placed where it is satisfied");
        assert_eq!((new.age, new.residence), (0, 5));
        assert_eq!(w.agents().count(), 3);
    }

    #[test]
    fn a_newcomer_always_finds_a_site() {
        // 5 × 5 of Blues but one empty site; the leaver's site frees another.
        let mut w = empty(5, 5);
        let mut leaver = 0;
        for y in 0..5 {
            for x in 0..5 {
                if (x, y) == (0, 0) {
                    continue;
                }
                let id = put(&mut w, x, y, false, 0.0);
                if (x, y) == (3, 3) {
                    leaver = id;
                }
            }
        }
        w.agents.get_mut(&leaver).unwrap().residence = 1;
        w.config.residence = Residence {
            enabled: true,
            min: 9,
            max: 9,
        };
        w.config.preference = FRange { min: 1.0, max: 1.0 };
        w.step();
        assert_eq!(w.agents().count(), 24);
        assert!(!w.agents.contains_key(&leaver));
        let empty: Vec<usize> = (0..25).filter(|&i| w.grid[i].is_none()).collect();
        assert_eq!(empty.len(), 1);
    }

    #[test]
    fn statistics_follow_the_definitions() {
        let mut w = empty(10, 10);
        let a = put(&mut w, 1, 1, true, 0.5);
        put(&mut w, 1, 2, true, 0.5);
        put(&mut w, 2, 1, false, 0.5);
        put(&mut w, 8, 8, false, 0.9); // no neighbors: satisfied, not in segregation
        let s = w.snapshot();
        // a: 1 of 2 alike; (1,2): 1 of 1; (2,1): 0 of 1 (unsatisfied).
        assert_eq!(s.population, 4);
        assert_eq!(s.unsatisfied, 0.25);
        assert!((s.segregation - (0.5 + 1.0 + 0.0) / 3.0).abs() < 1e-12);
        assert_eq!(s.red_share, 0.5);
        assert_eq!((s.moves, s.quiet), (0, 0), "t = 0 is not quiet");
        assert!(w.is_satisfied(&agent(&w, a)));
    }

    #[test]
    fn setup_places_the_population_on_distinct_sites_with_both_colors() {
        let w = SchellingWorld::new(SchellingConfig::default(), 3).unwrap();
        assert_eq!(w.agents().count(), 2000);
        let reds = w.agents().filter(|a| a.red).count();
        assert!((900..=1100).contains(&reds), "{reds}");
        assert!(w.agents().all(|a| a.preference == 0.25 && a.residence == 0));
        let mixed = SchellingWorld::new(
            SchellingConfig {
                preference: FRange {
                    min: 0.25,
                    max: 0.5,
                },
                ..SchellingConfig::default()
            },
            3,
        )
        .unwrap();
        assert!(mixed.agents().all(|a| (0.25..=0.5).contains(&a.preference)));
        assert!(
            mixed.agents().any(|a| a.preference > 0.4)
                && mixed.agents().any(|a| a.preference < 0.3)
        );
    }

    #[test]
    fn validation_names_fields() {
        let bad = SchellingConfig {
            width: 4,
            population: 2500,
            preference: FRange { min: 0.6, max: 0.5 },
            residence: Residence {
                enabled: true,
                min: 0,
                max: 5,
            },
            ..SchellingConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["width", "population", "preference", "residence.min"]
        );
    }

    #[test]
    fn renders_colors_satisfaction_and_preference() {
        let mut w = empty(5, 5);
        put(&mut w, 0, 0, true, 0.25);
        put(&mut w, 1, 0, false, 1.0);
        let mut buf = Vec::new();
        let pixel = |buf: &[u8], i: usize| [buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2]];
        w.render("color", "", &mut buf).unwrap();
        assert_eq!(
            (pixel(&buf, 0), pixel(&buf, 1), pixel(&buf, 2)),
            (RED, BLUE, BACKGROUND)
        );
        w.render("satisfaction", "", &mut buf).unwrap();
        assert_eq!(pixel(&buf, 0), BOTH, "0 of 1 alike < 25%: unsatisfied");
        assert_eq!(pixel(&buf, 1), BOTH);
        w.render("preference", "", &mut buf).unwrap();
        assert_eq!(pixel(&buf, 1), HOT);
        assert!(w.render("tribe", "", &mut buf).is_err());
    }

    #[test]
    fn set_config_refuses_every_change() {
        let mut w = SchellingWorld::new(SchellingConfig::default(), 1).unwrap();
        let same = ModelConfig::Schelling(SchellingConfig::default());
        assert!(Model::set_config(&mut w, same).is_ok());
        let next = SchellingConfig {
            population: 10,
            ..SchellingConfig::default()
        };
        let e = Model::set_config(&mut w, ModelConfig::Schelling(next)).unwrap_err();
        assert_eq!(e[0].field, "population");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Schelling(SchellingConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }

    #[test]
    fn presets_follow_the_book() {
        let ps = presets();
        let get = |id: &str| match &ps.iter().find(|p| p.id == id).unwrap().config {
            ModelConfig::Schelling(c) => c.clone(),
            _ => unreachable!(),
        };
        let vi4 = get("vi-4-schelling-25");
        assert_eq!((vi4.width, vi4.height, vi4.population), (50, 50, 2000));
        assert_eq!(
            vi4.preference,
            FRange {
                min: 0.25,
                max: 0.25
            }
        );
        assert!(!vi4.residence.enabled);
        let vi5 = get("vi-5-schelling-25-residence");
        assert_eq!((vi5.residence.min, vi5.residence.max), (80, 100));
        assert!(vi5.residence.enabled);
        assert_eq!(
            get("vi-6-schelling-50-residence").preference,
            FRange { min: 0.5, max: 0.5 }
        );
        assert_eq!(
            get("vi-7-schelling-mixed").preference,
            FRange {
                min: 0.25,
                max: 0.5
            }
        );
        for p in &ps {
            p.config.validate().unwrap();
        }
    }
}
```
In `crates/sugarscape-core/src/lib.rs`, after `pub mod rules;` add `pub mod schelling;` and `pub mod schema;`.

In `crates/sugarscape-core/src/model.rs`'s tests module, before `fn unknown_models_are_field_errors`, add:
```rust
    #[test]
    fn schelling_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "schelling", "population": 100}"#).unwrap();
        assert_eq!(c.kind(), ModelKind::Schelling);
        let ModelConfig::Schelling(s) = &c else {
            unreachable!()
        };
        assert_eq!(
            (s.population, s.width),
            (100, 50),
            "missing fields take the defaults"
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "schelling");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        let e = ModelConfig::from_json(r#"{"model": "schelling", "vision": 3}"#).unwrap_err();
        assert!(e[0].message.contains("vision"), "{e:?}");
        let e =
            ModelConfig::from_json(r#"{"model": "schelling", "population": 2500}"#).unwrap_err();
        assert_eq!(e[0].field, "population");
    }

    #[test]
    fn with_path_sets_another_models_fields() {
        let c = ModelConfig::Schelling(SchellingConfig::default());
        let next = c.with_path("preference.max", &json!(0.5)).unwrap();
        let ModelConfig::Schelling(s) = &next else {
            unreachable!()
        };
        assert_eq!((s.preference.min, s.preference.max), (0.25, 0.5));
        assert_eq!(
            c.with_path("vision.max", &json!(3)).unwrap_err().message,
            "unknown field vision.max"
        );
        assert!(c.with_path("model", &json!("sugarscape")).is_err());
        assert!(c.with_path("population", &json!("many")).is_err());
    }

    #[test]
    fn a_world_refuses_another_models_config() {
        let mut any = ModelWorld::new(ModelConfig::from(Config::default()), 1).unwrap();
        let e = any
            .model_mut()
            .set_config(ModelConfig::Schelling(SchellingConfig::default()))
            .unwrap_err();
        assert_eq!(e[0].field, "model");
        let mut s = ModelWorld::new(ModelConfig::Schelling(SchellingConfig::default()), 1).unwrap();
        assert!(s.model_mut().set_config(Config::default().into()).is_err());
        assert!(s.sugarscape().is_none());
    }

```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core --lib schelling`
Expected: FAIL to compile — `SchellingWorld`, `SchellingConfig`, `crate::schema` not found.

- [ ] **Step 3: Implement**

Create `crates/sugarscape-core/src/schema.rs` (Decision 8):
```rust
//! Parameter schemas (milestone 9): each model other than the sugarscape
//! describes its config's fields, and the page builds its Rules panel from
//! the description (the sugarscape keeps its bespoke panel).

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamKind {
    /// A whole number.
    Integer,
    /// Any number in `min..=max`, in steps of `step`.
    Number,
    /// A `{ min, max }` pair: the panel sets `<path>.min` and `<path>.max`.
    Range,
    Bool,
    /// One of `choices` (a string).
    Choice,
}

/// Whether a change applies to the running world or rebuilds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Apply {
    Live,
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub value: &'static str,
    pub label: &'static str,
}

/// One field of a model's config, as the page shows it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Param {
    pub path: &'static str,
    pub label: &'static str,
    pub kind: ParamKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<Choice>,
    pub apply: Apply,
    /// The panel section it is shown in.
    pub group: &'static str,
}

impl Param {
    fn new(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        kind: ParamKind,
        apply: Apply,
    ) -> Self {
        Param {
            path,
            label,
            kind,
            min: None,
            max: None,
            step: None,
            choices: Vec::new(),
            apply,
            group,
        }
    }

    fn bounded(mut self, min: f64, max: f64, step: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self.step = Some(step);
        self
    }

    pub fn integer(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        (min, max): (u32, u32),
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Integer, apply).bounded(
            f64::from(min),
            f64::from(max),
            1.0,
        )
    }

    pub fn number(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        (min, max, step): (f64, f64, f64),
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Number, apply).bounded(min, max, step)
    }

    /// A `{ min, max }` pair whose ends lie in `min..=max`, in steps of `step`.
    pub fn range(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        (min, max, step): (f64, f64, f64),
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Range, apply).bounded(min, max, step)
    }

    pub fn bool(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        apply: Apply,
    ) -> Self {
        Self::new(group, path, label, ParamKind::Bool, apply)
    }

    pub fn choice(
        group: &'static str,
        path: &'static str,
        label: &'static str,
        choices: &[(&'static str, &'static str)],
        apply: Apply,
    ) -> Self {
        let mut p = Self::new(group, path, label, ParamKind::Choice, apply);
        p.choices = choices
            .iter()
            .map(|&(value, label)| Choice { value, label })
            .collect();
        p
    }
}

/// Checks a model's schema against its config type (used by each model's
/// tests): every path exists in `config`'s JSON with the kind's shape, and
/// changing it on a running world is accepted exactly when it is `Live`.
#[cfg(test)]
pub(crate) fn check_schema(
    schema: &[Param],
    config: &crate::model::ModelConfig,
    world: impl Fn() -> crate::model::ModelWorld,
) {
    use serde_json::{json, Value};
    let value = serde_json::to_value(config).unwrap();
    for p in schema {
        let at = p
            .path
            .split('.')
            .try_fold(&value, |v, k| v.get(k))
            .unwrap_or_else(|| panic!("{} is not in the config", p.path));
        let changed: Value = match p.kind {
            ParamKind::Integer => json!(at.as_u64().unwrap() + 1),
            ParamKind::Number => json!(at.as_f64().unwrap() + p.step.unwrap()),
            ParamKind::Range => {
                let (lo, hi) = (at["min"].as_f64().unwrap(), at["max"].as_f64().unwrap());
                assert!(lo <= hi, "{}", p.path);
                if at["min"].is_u64() {
                    json!({ "min": lo as u64, "max": hi as u64 + 1 })
                } else {
                    json!({ "min": lo, "max": (hi + p.step.unwrap()).min(p.max.unwrap()) })
                }
            }
            ParamKind::Bool => json!(!at.as_bool().unwrap()),
            ParamKind::Choice => {
                let now = at.as_str().unwrap();
                json!(p.choices.iter().find(|c| c.value != now).unwrap().value)
            }
        };
        let mut next = config.clone();
        for (k, v) in match p.kind {
            ParamKind::Range => vec![
                (format!("{}.min", p.path), changed["min"].clone()),
                (format!("{}.max", p.path), changed["max"].clone()),
            ],
            _ => vec![(p.path.to_string(), changed)],
        } {
            next = next
                .with_path(&k, &v)
                .unwrap_or_else(|e| panic!("{k}: {e:?}"));
        }
        assert_ne!(&next, config, "{} did not change", p.path);
        let mut w = world();
        let result = w.model_mut().set_config(next);
        assert_eq!(
            result.is_ok(),
            p.apply == Apply::Live,
            "{}: {result:?}",
            p.path
        );
    }
}
```

`crates/sugarscape-core/src/schelling.rs` — between the module doc and the tests, add (Decision 9; the class pools are the performance-critical part: keep `counts`, `pools`, `filed` and `slot` consistent in `file`/`unfile`/`tell_neighbors`, and sample `u32` in `pick`):
```rust
use std::collections::BTreeMap;
use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::export;
use crate::geometry::{Pos, Torus};
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::presets::ModelPreset;
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, RED};
use crate::rng::{self, SimRng};
use crate::schema::{Apply, Param};
use crate::stats::{Series, Stats};

/// A fraction range `[min, max]` (preferences).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FRange {
    pub min: f64,
    pub max: f64,
}

/// Note 11's "maximum lifetime", read as a maximum residence: each agent
/// leaves after a whole number of ticks drawn from `[min, max]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Residence {
    pub enabled: bool,
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SchellingConfig {
    pub width: u32,
    pub height: u32,
    pub population: u32,
    /// Each agent's minimum share of like-colored neighbors, uniform in
    /// `[min, max]` (min = max for a fixed preference).
    pub preference: FRange,
    pub residence: Residence,
}

impl Default for SchellingConfig {
    /// Animation VI-4: "a 50 × 50 lattice … with 2000 Red and Blue agents …
    /// at least 25 percent of its neighbors", no maximum residence.
    fn default() -> Self {
        SchellingConfig {
            width: 50,
            height: 50,
            population: 2000,
            preference: FRange {
                min: 0.25,
                max: 0.25,
            },
            residence: Residence {
                enabled: false,
                min: 80,
                max: 100,
            },
        }
    }
}

impl SchellingConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (5..=500).contains(&self.width),
            "width",
            "must be between 5 and 500",
        );
        check(
            (5..=500).contains(&self.height),
            "height",
            "must be between 5 and 500",
        );
        check(
            u64::from(self.population) < u64::from(self.width) * u64::from(self.height),
            "population",
            "must be less than the number of sites (width × height)",
        );
        let p = self.preference;
        let fraction = |v: f64| (0.0..=1.0).contains(&v);
        check(
            fraction(p.min) && fraction(p.max),
            "preference",
            "must be fractions between 0 and 1",
        );
        check(p.min <= p.max, "preference", "min must be ≤ max");
        let r = self.residence;
        check(r.min >= 1, "residence.min", "must be ≥ 1");
        check(r.min <= r.max, "residence", "min must be ≤ max");
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that differ from `next` (every field changes only on reset).
    fn changes(&self, next: &SchellingConfig) -> Vec<FieldError> {
        let msg = "changes only on reset";
        let mut out = Vec::new();
        for (field, same) in [
            ("width", self.width == next.width),
            ("height", self.height == next.height),
            ("population", self.population == next.population),
            ("preference", self.preference == next.preference),
            ("residence", self.residence == next.residence),
        ] {
            if !same {
                out.push(FieldError::new(field, msg));
            }
        }
        out
    }
}

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 6] = [
    "unsatisfied",
    "segregation",
    "moves",
    "red_share",
    "quiet",
    "population",
];

/// One tick's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SchellingSnapshot {
    pub tick: u64,
    pub population: u32,
    /// Share of agents not satisfied where they stand (0 with no agents).
    pub unsatisfied: f64,
    /// Mean share of like-colored neighbors over the agents with at least
    /// one neighbor (0 when none has one).
    pub segregation: f64,
    /// Agents that moved this tick.
    pub moves: u32,
    /// Share of agents that are Red (0 with no agents).
    pub red_share: f64,
    /// 1 when a tick ran and no agent moved, else 0 (0 at t = 0).
    pub quiet: u32,
}

impl Series for SchellingSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "unsatisfied" => self.unsatisfied,
            "segregation" => self.segregation,
            "moves" => f64::from(self.moves),
            "red_share" => self.red_share,
            "quiet" => f64::from(self.quiet),
            _ => return None,
        })
    }
}

/// One agent: its color, preference, age and (with residence on) the age at
/// which it leaves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resident {
    pub id: u64,
    pub pos: Pos,
    pub red: bool,
    pub preference: f64,
    pub age: u32,
    /// The age at which it leaves; 0 with residence off.
    pub residence: u32,
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SchellingInspection {
    pub site: SiteXy,
    pub agent: Option<ResidentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SiteXy {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResidentView {
    pub id: u64,
    pub color: &'static str,
    pub preference: f64,
    pub satisfied: bool,
    /// Like-colored and all occupied von Neumann neighbors.
    pub like: u32,
    pub neighbors: u32,
    pub age: u32,
    /// The age at which it leaves, or null with residence off.
    pub residence: Option<u32>,
}

/// The color modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchellingMode {
    Color,
    Satisfaction,
    Preference,
}

impl std::str::FromStr for SchellingMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "color" => Self::Color,
            "satisfaction" => Self::Satisfaction,
            "preference" => Self::Preference,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// Whether `like` of `occupied` neighbors meet `preference` ("if this number
/// is greater than or equal to its preference the agent is … satisfied");
/// an agent with no neighbors is satisfied.
pub fn satisfied(like: u32, occupied: u32, preference: f64) -> bool {
    occupied == 0 || f64::from(like) / f64::from(occupied) >= preference
}

/// A (like, occupied) neighbor class: `occupied` in 0..=4, `like` in
/// 0..=occupied, numbered `occupied·(occupied + 1)/2 + like` (15 classes).
const CLASSES: usize = 15;
const UNFILED: u8 = u8::MAX;

fn class(like: u8, occupied: u8) -> usize {
    usize::from(occupied) * (usize::from(occupied) + 1) / 2 + usize::from(like)
}

/// Whether an agent with `preference` is satisfied in each class.
fn acceptable_classes(preference: f64) -> [bool; CLASSES] {
    let mut out = [false; CLASSES];
    for occupied in 0..=4u8 {
        for like in 0..=occupied {
            out[class(like, occupied)] =
                satisfied(u32::from(like), u32::from(occupied), preference);
        }
    }
    out
}

pub struct SchellingWorld {
    pub config: SchellingConfig,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    agents: BTreeMap<u64, Resident>,
    /// The agent on each site (row-major).
    grid: Vec<Option<u64>>,
    /// Occupied von Neumann neighbors of each site: `[blue, red]` counts.
    counts: [Vec<u8>; 2],
    /// Every empty site filed, for each color (`[blue, red]`), under the
    /// class it would give an agent of that color standing there.
    pools: [[Vec<u32>; CLASSES]; 2],
    /// Each site's class in `pools` (`UNFILED` when occupied, or while its
    /// agent considers moving) and its index there.
    filed: [Vec<u8>; 2],
    slot: [Vec<u32>; 2],
    rng: SimRng,
    next_id: u64,
    moves: u32,
    pub stats: Stats<SchellingSnapshot>,
}

impl SchellingWorld {
    pub fn new(config: SchellingConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let torus = Torus::new(config.width, config.height);
        let n = torus.len();
        let mut world = SchellingWorld {
            torus,
            tick: 0,
            agents: BTreeMap::new(),
            grid: vec![None; n],
            counts: [vec![0; n], vec![0; n]],
            pools: Default::default(),
            filed: [vec![UNFILED; n], vec![UNFILED; n]],
            slot: [vec![0; n], vec![0; n]],
            rng: rng::seeded(seed),
            next_id: 1,
            moves: 0,
            stats: Stats::default(),
            config,
        };
        for i in 0..n {
            world.file(i);
        }
        let mut cells: Vec<usize> = (0..n).collect();
        cells.shuffle(&mut world.rng);
        for &i in cells.iter().take(world.config.population as usize) {
            let agent = world.newcomer(torus.pos(i));
            world.insert(agent);
        }
        let snapshot = world.snapshot();
        world.stats.push(snapshot);
        Ok(world)
    }

    /// A new agent at `pos`: Red or Blue with probability ½, a preference
    /// uniform in the configured range, and (residence on) a residence
    /// uniform in its range.
    fn newcomer(&mut self, pos: Pos) -> Resident {
        let c = &self.config;
        let (p, r) = (c.preference, c.residence);
        let red = self.rng.gen_bool(0.5);
        let preference = self.rng.gen_range(p.min..=p.max);
        let residence = if r.enabled {
            self.rng.gen_range(r.min..=r.max)
        } else {
            0
        };
        let id = self.next_id;
        self.next_id += 1;
        Resident {
            id,
            pos,
            red,
            preference,
            age: 0,
            residence,
        }
    }

    /// Files empty site `i` under its class for each color.
    fn file(&mut self, i: usize) {
        let (blues, reds) = (self.counts[0][i], self.counts[1][i]);
        for (c, like) in [(0, blues), (1, reds)] {
            let k = class(like, blues + reds);
            self.filed[c][i] = k as u8;
            self.slot[c][i] = self.pools[c][k].len() as u32;
            self.pools[c][k].push(i as u32);
        }
    }

    fn unfile(&mut self, i: usize) {
        for c in 0..2 {
            let k = self.filed[c][i];
            if k == UNFILED {
                continue;
            }
            let pool = &mut self.pools[c][usize::from(k)];
            let at = self.slot[c][i] as usize;
            pool.swap_remove(at);
            if let Some(&moved) = pool.get(at) {
                self.slot[c][moved as usize] = at as u32;
            }
            self.filed[c][i] = UNFILED;
        }
    }

    /// Adds `delta` to the `red` count of each neighbor of site `i`,
    /// refiling the empty ones.
    fn tell_neighbors(&mut self, i: usize, red: bool, add: bool) {
        for n in self.torus.neighbors(self.torus.pos(i)) {
            let j = self.torus.index(n);
            let count = &mut self.counts[usize::from(red)][j];
            if add {
                *count += 1;
            } else {
                *count -= 1;
            }
            if self.filed[0][j] != UNFILED {
                self.unfile(j);
                self.file(j);
            }
        }
    }

    /// Puts agent `id` of color `red` on site `i`.
    fn occupy(&mut self, i: usize, id: u64, red: bool) {
        self.unfile(i);
        debug_assert!(self.grid[i].is_none());
        self.grid[i] = Some(id);
        self.tell_neighbors(i, red, true);
    }

    /// Takes the agent of color `red` off site `i`, leaving it unfiled.
    fn vacate(&mut self, i: usize, red: bool) {
        self.grid[i] = None;
        self.tell_neighbors(i, red, false);
    }

    fn insert(&mut self, a: Resident) {
        self.occupy(self.torus.index(a.pos), a.id, a.red);
        self.agents.insert(a.id, a);
    }

    /// A uniformly random filed site that satisfies an agent of color `red`
    /// with `preference` (every filed site when `preference` is `None`).
    fn pick(&mut self, red: bool, preference: Option<f64>) -> Option<usize> {
        let ok = preference.map_or([true; CLASSES], acceptable_classes);
        let pools = &self.pools[usize::from(red)];
        let total: usize = (0..CLASSES)
            .filter(|&k| ok[k])
            .map(|k| pools[k].len())
            .sum();
        if total == 0 {
            return None;
        }
        // Sampled as u32 (sites ≤ 250 000): a usize range would draw
        // differently on wasm32 than on 64-bit targets.
        let mut r = self.rng.gen_range(0..total as u32) as usize;
        for k in (0..CLASSES).filter(|&k| ok[k]) {
            let pool = &pools[k];
            if r < pool.len() {
                return Some(pool[r] as usize);
            }
            r -= pool.len();
        }
        unreachable!("r < total")
    }

    pub fn agents(&self) -> impl Iterator<Item = &Resident> {
        self.agents.values()
    }

    pub fn agent_at(&self, pos: Pos) -> Option<&Resident> {
        self.grid[self.torus.index(pos)].and_then(|id| self.agents.get(&id))
    }

    /// Like-colored and occupied neighbors of `a` where it stands.
    fn neighbors(&self, a: &Resident) -> (u32, u32) {
        let i = self.torus.index(a.pos);
        let (blues, reds) = (self.counts[0][i], self.counts[1][i]);
        let like = if a.red { reds } else { blues };
        (u32::from(like), u32::from(blues + reds))
    }

    /// Whether `a` is satisfied where it stands.
    pub fn is_satisfied(&self, a: &Resident) -> bool {
        let (like, occupied) = self.neighbors(a);
        satisfied(like, occupied, a.preference)
    }

    /// One tick: agents act in a random order, and each unsatisfied one moves
    /// to a site chosen uniformly at random among the empty sites where it
    /// would be satisfied — judged with its own site vacated, so it does not
    /// count itself as a neighbor — or stays if there is none. Then every
    /// agent ages, and with residence on each that reaches its maximum is
    /// replaced.
    pub fn step(&mut self) {
        let mut order: Vec<u64> = self.agents.keys().copied().collect();
        order.shuffle(&mut self.rng);
        let mut moves = 0;
        for id in order {
            let a = self.agents[&id];
            if self.is_satisfied(&a) {
                continue;
            }
            let from = self.torus.index(a.pos);
            self.vacate(from, a.red);
            match self.pick(a.red, Some(a.preference)) {
                Some(to) => {
                    self.occupy(to, id, a.red);
                    self.file(from);
                    self.agents.get_mut(&id).expect("living agent").pos = self.torus.pos(to);
                    moves += 1;
                }
                None => self.occupy(from, id, a.red),
            }
        }
        for a in self.agents.values_mut() {
            a.age += 1;
        }
        if self.config.residence.enabled {
            self.replace_departed();
        }
        self.moves = moves;
        self.tick += 1;
        let snapshot = self.snapshot();
        self.stats.push(snapshot);
    }

    /// VI-5: each agent that has reached its maximum residence leaves (in id
    /// order) and "is replaced … with a new agent of random color … placed
    /// at a randomly selected position satisfying its preference" — any
    /// empty site if none satisfies it.
    fn replace_departed(&mut self) {
        let departed: Vec<u64> = self
            .agents
            .values()
            .filter(|a| a.residence > 0 && a.age >= a.residence)
            .map(|a| a.id)
            .collect();
        for id in departed {
            let old = self.agents.remove(&id).expect("living agent");
            let i = self.torus.index(old.pos);
            self.vacate(i, old.red);
            self.file(i);
            let mut new = self.newcomer(old.pos);
            let to = self
                .pick(new.red, Some(new.preference))
                .or_else(|| self.pick(new.red, None))
                .expect("the departed agent's site is empty");
            new.pos = self.torus.pos(to);
            self.insert(new);
        }
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    fn snapshot(&self) -> SchellingSnapshot {
        let n = self.agents.len();
        let (mut unsatisfied, mut reds, mut shares, mut counted) = (0usize, 0usize, 0.0, 0usize);
        for a in self.agents.values() {
            let (like, occupied) = self.neighbors(a);
            if !satisfied(like, occupied, a.preference) {
                unsatisfied += 1;
            }
            if a.red {
                reds += 1;
            }
            if occupied > 0 {
                shares += f64::from(like) / f64::from(occupied);
                counted += 1;
            }
        }
        let share = |k: usize, of: usize| if of == 0 { 0.0 } else { k as f64 / of as f64 };
        SchellingSnapshot {
            tick: self.tick,
            population: n as u32,
            unsatisfied: share(unsatisfied, n),
            segregation: if counted == 0 {
                0.0
            } else {
                shares / counted as f64
            },
            moves: self.moves,
            red_share: share(reds, n),
            quiet: u32::from(self.tick > 0 && self.moves == 0),
        }
    }

    fn color(&self, a: &Resident, mode: SchellingMode) -> Rgb {
        let own = if a.red { RED } else { BLUE };
        match mode {
            SchellingMode::Color => own,
            SchellingMode::Satisfaction => {
                if self.is_satisfied(a) {
                    lerp(BACKGROUND, own, 0.3)
                } else {
                    BOTH
                }
            }
            SchellingMode::Preference => lerp(COOL, HOT, a.preference),
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<SchellingInspection, String> {
        if x >= self.torus.width || y >= self.torus.height {
            return Err(format!("({x}, {y}) is outside the grid"));
        }
        let agent = self.agent_at(Pos::new(x, y)).map(|a| {
            let (like, neighbors) = self.neighbors(a);
            ResidentView {
                id: a.id,
                color: if a.red { "red" } else { "blue" },
                preference: a.preference,
                satisfied: satisfied(like, neighbors, a.preference),
                like,
                neighbors,
                age: a.age,
                residence: (a.residence > 0).then_some(a.residence),
            }
        });
        Ok(SchellingInspection {
            site: SiteXy { x, y },
            agent,
        })
    }
}
impl Model for SchellingWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Schelling(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        SchellingWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick and every agent's id, site, color, preference,
    /// age and residence.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(u64::from(a.red));
            eat(a.preference.to_bits());
            eat((u64::from(a.age) << 32) | u64::from(a.residence));
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.torus.width, self.torus.height)
    }

    /// Empty sites dark; agents Red/Blue (`color`), dimmed when satisfied and
    /// yellow when not (`satisfaction`), or cool-to-hot by preference
    /// (`preference`). `layer` is ignored.
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: SchellingMode = mode.parse()?;
        buf.resize(self.grid.len() * 4, 0);
        for (i, slot) in self.grid.iter().enumerate() {
            let rgb = match slot {
                Some(id) => self.color(&self.agents[id], mode),
                None => BACKGROUND,
            };
            buf[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,x,y,color,preference,satisfied,age,residence\n");
        for a in self.agents.values() {
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                a.id,
                a.pos.x,
                a.pos.y,
                if a.red { "red" } else { "blue" },
                a.preference,
                self.is_satisfied(a),
                a.age,
                a.residence
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        self.agents.get(&id).map(|a| (a.pos.x, a.pos.y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Schelling(next) = next else {
            return Err(wrong_model(ModelKind::Schelling, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }
}

/// The Rules panel's fields: every one rebuilds the world (agents draw their
/// preference and residence when they are created).
pub fn schema() -> Vec<Param> {
    use Apply::Reset;
    vec![
        Param::integer("Setup", "width", "Width", (5, 200), Reset),
        Param::integer("Setup", "height", "Height", (5, 200), Reset),
        Param::integer("Setup", "population", "Agents", (0, 39_999), Reset),
        Param::range(
            "Preference",
            "preference",
            "Like neighbors wanted (fraction)",
            (0.0, 1.0, 0.05),
            Reset,
        ),
        Param::bool("Residence", "residence.enabled", "Maximum residence", Reset),
        Param::range(
            "Residence",
            "residence",
            "Residence (ticks)",
            (1.0, 1000.0, 1.0),
            Reset,
        ),
    ]
}

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut SchellingConfig),
) -> ModelPreset {
    let mut c = SchellingConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Schelling(c),
    }
}

fn residence(c: &mut SchellingConfig) {
    c.residence = Residence {
        enabled: true,
        min: 80,
        max: 100,
    };
}

/// Animations VI-4 to VI-7.
pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "vi-4-schelling-25",
            "Schelling segregation, 25% like neighbors",
            "Animation VI-4",
            "2000 Red and Blue agents on a 50×50 torus, each wanting at least 25% of its von Neumann neighbors to share its color; an unsatisfied agent moves to a random site where it would be satisfied. Within a few ticks nobody moves, and the pattern is clearly more segregated than the start (segregation about 0.50 → 0.63).",
            |_| {},
        ),
        preset(
            "vi-5-schelling-25-residence",
            "Schelling segregation, 25%, residence 80–100",
            "Animation VI-5",
            "As VI-4, but each agent leaves after 80–100 ticks and is replaced by a new agent of random color placed where it is satisfied: the pattern never settles. Its segregation climbs to about 0.76, somewhat above VI-4's (the book calls the two comparable).",
            residence,
        ),
        preset(
            "vi-6-schelling-50-residence",
            "Schelling segregation, 50%, residence 80–100",
            "Animation VI-6",
            "As VI-5 with every agent wanting at least half of its neighbors to share its color: far more segregated (about 0.95).",
            |c| {
                residence(c);
                c.preference = FRange { min: 0.5, max: 0.5 };
            },
        ),
        preset(
            "vi-7-schelling-mixed",
            "Schelling segregation, 25–50% mixed, residence 80–100",
            "Animation VI-7",
            "As VI-5 with preferences spread uniformly between 25% and 50%: the tolerant agents are not enough to undo the segregation, which stays high (about 0.93), close to VI-6's.",
            |c| {
                residence(c);
                c.preference = FRange {
                    min: 0.25,
                    max: 0.5,
                };
            },
        ),
    ]
}
```

`crates/sugarscape-core/src/model.rs` (Decision 1):
- Imports: after `use crate::render::{self, ColorMode, Layer};` add `use crate::schelling::{SchellingConfig, SchellingWorld};` and `use crate::schema::Param;`; `use crate::{export, stats};` becomes `use crate::{export, schelling, stats};`.
- `ModelKind` gains `Schelling` (after `Sugarscape`), and its `impl` becomes
  ```rust
  impl ModelKind {
      pub const ALL: [ModelKind; 2] = [ModelKind::Sugarscape, ModelKind::Schelling];

      pub fn as_str(self) -> &'static str {
          match self {
              ModelKind::Sugarscape => "sugarscape",
              ModelKind::Schelling => "schelling",
          }
      }

      /// The Rules panel's fields; empty for the sugarscape, whose panel is
      /// hand-built.
      pub fn schema(self) -> Vec<Param> {
          match self {
              ModelKind::Sugarscape => Vec::new(),
              ModelKind::Schelling => schelling::schema(),
          }
      }
  }
  ```
- Replace `#[derive(Clone, Debug, PartialEq)]\npub enum ModelConfig {\n    Sugarscape(Config),\n}` with
  ```rust
  // Configs are cloned rarely (never per tick), so the sugarscape's larger
  // variant is not boxed.
  #[allow(clippy::large_enum_variant)]
  #[derive(Clone, Debug, PartialEq)]
  pub enum ModelConfig {
      Sugarscape(Config),
      Schelling(SchellingConfig),
  }

  /// Another model's config on the wire: its fields and `"model": "<kind>"`.
  #[derive(Serialize)]
  #[serde(tag = "model", rename_all = "snake_case")]
  enum Tagged<'a> {
      Schelling(&'a SchellingConfig),
  }
  ```
- `Serialize`: after `ModelConfig::Sugarscape(c) => c.serialize(s),` add `ModelConfig::Schelling(c) => Tagged::Schelling(c).serialize(s),`.
- `kind()`: add `ModelConfig::Schelling(_) => ModelKind::Schelling,`. `sugarscape()`: add `_ => None,` after the `Some(c)` arm.
- `from_value`: its doc's last line becomes `/// (`Config::from_value`); another model's missing fields take its\n/// defaults, and unknown fields are errors.`, and the `match tag.as_str()` becomes
  ```rust
          match tag.as_str() {
              "sugarscape" => Config::from_value(value).map(ModelConfig::Sugarscape),
              "schelling" => serde_json::from_value(value)
                  .map(ModelConfig::Schelling)
                  .map_err(|e| FieldError::new("config", e.to_string())),
              _ => Err(FieldError::new(
                  "model",
                  format!("unknown model {tag:?} (expected sugarscape or schelling)"),
              )),
          }
  ```
- `validate()`: add `ModelConfig::Schelling(c) => c.validate(),`. `with_path()`: add `ModelConfig::Schelling(c) => set_path(c, path, value).map(ModelConfig::Schelling),`. `series_names()`: add `ModelConfig::Schelling(_) => schelling::SERIES.iter().map(|s| s.to_string()).collect(),`.
- After `impl ModelConfig { … }` add
  ```rust
  /// `config` with the dotted `path` set to `value` (through its JSON, like
  /// `Config::with_path`; the error field is `schedule`, as there).
  fn set_path<T: Serialize + serde::de::DeserializeOwned>(
      config: &T,
      path: &str,
      value: &serde_json::Value,
  ) -> Result<T, FieldError> {
      let mut json = serde_json::to_value(config).expect("config serializes");
      let unknown = || FieldError::new("schedule", format!("unknown field {path}"));
      let mut slot = &mut json;
      for key in path.split('.') {
          slot = slot.get_mut(key).ok_or_else(unknown)?;
      }
      *slot = value.clone();
      serde_json::from_value(json).map_err(|e| FieldError::new("schedule", format!("{path}: {e}")))
  }
  ```
- Before `impl Model for World {` add
  ```rust
  /// The error for handing a world another model's config.
  pub(crate) fn wrong_model(want: ModelKind, got: &ModelConfig) -> Vec<FieldError> {
      vec![FieldError::new(
          "model",
          format!(
              "a {} world cannot take a {} config",
              want.as_str(),
              got.kind().as_str()
          ),
      )]
  }
  ```
  and in `World`'s `set_config` add the arm `other => Err(wrong_model(ModelKind::Sugarscape, &other)),`.
- `ModelWorld` gains `Schelling(Box<SchellingWorld>),`; `with_landscapes` gains the arm
  ```rust
              ModelConfig::Schelling(c) => {
                  ModelWorld::Schelling(Box::new(SchellingWorld::new(c, seed)?))
              }
  ```
  `kind()` gains `ModelWorld::Schelling(_) => ModelKind::Schelling,`, `model()` gains `ModelWorld::Schelling(w) => w.as_ref(),`, `model_mut()` gains `ModelWorld::Schelling(w) => w.as_mut(),`, and `sugarscape()` / `sugarscape_mut()` gain `_ => None,`.

`crates/sugarscape-core/src/presets.rs` — `catalog` becomes
```rust
/// Every model's presets: the sugarscape's (`all`), then Schelling's.
pub fn catalog() -> Vec<ModelPreset> {
    let mut out: Vec<ModelPreset> = all().into_iter().map(ModelPreset::from).collect();
    out.extend(crate::schelling::presets());
    out
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core --lib`
Expected: PASS — including `the_pools_match_a_direct_count_as_agents_move_and_leave`, `schema_paths_exist_and_match_what_set_config_allows` and the three new model tests.
Run: `cargo test -p sugarscape-core --test golden`
Expected: FAIL only `every_model_preset_has_a_golden_entry` ("record a golden fingerprint for vi-4-schelling-25").

- [ ] **Step 5: Measure the performance (fixed threshold)**

```bash
cargo build --release -p sugarscape-cli
echo '{"model":"schelling"}' > target/perf-schelling-small.json
echo '{"model":"schelling","width":200,"height":200,"population":32000,"preference":{"min":0.5,"max":0.5},"residence":{"enabled":true,"min":80,"max":100}}' > target/perf-schelling.json
for f in perf-schelling-small perf-schelling; do for i in 1 2 3; do /usr/bin/time -p target/release/sugarscape run --config target/$f.json --ticks 1000 --fingerprint 2>&1 | tr '\n' ' '; echo; done; done
```
Expected: `user` at most 1 s for the 50 × 50 world and at most 10 s for the 200 × 200 one (1000 ticks each; planning: 0.10 s and 2.8 s on an M-series Mac, fingerprints `0x18ec4e09164dfcf6` and `0xc810b48432c87cf0`). Slower means the pools are being scanned or rebuilt: fix it before going on.

- [ ] **Step 6: Record the golden fingerprints**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`
Expected (the lines after `MODEL_GOLDEN:`; Decision 18):
```
    ("vi-4-schelling-25", 0x7a7072c3433f5f6f),
    ("vi-5-schelling-25-residence", 0x9abe1c25e873debd),
    ("vi-6-schelling-50-residence", 0x637412f7af91f684),
    ("vi-7-schelling-mixed", 0x79346d2a338108cf),
```
Put exactly these four entries into `MODEL_GOLDEN` in `crates/sugarscape-core/tests/golden.rs` (the `GOLDEN` table stays unedited). If the printed values differ, stop and report (Decision 18).
Run: `cargo test -p sugarscape-core --test golden`
Expected: PASS.

- [ ] **Step 7: The book-style tests (measured)**

Add `use sugarscape_core::model::ModelWorld;` after `use sugarscape_core::econ;` in `crates/sugarscape-core/tests/book.rs` and append:
```rust
/// Any model's preset `id` run `ticks` ticks from `seed`: its series `name`.
fn model_series(id: &str, seed: u64, ticks: u32, name: &str) -> Vec<f64> {
    let mut w = ModelWorld::new(presets::find(id).unwrap().config, seed).unwrap();
    w.model_mut().run(ticks);
    w.model().series(name).unwrap()
}

/// Mean segregation over ticks 500–1000 of a 1000-tick run.
fn late_segregation(id: &str, seed: u64) -> f64 {
    let seg = model_series(id, seed, 1000, "segregation");
    seg[500..=1000].iter().sum::<f64>() / 501.0
}

/// Prints the figures the Schelling thresholds below come from.
#[test]
#[ignore]
fn measure_schelling() {
    for seed in 1..=5 {
        let seg = model_series("vi-4-schelling-25", seed, 200, "segregation");
        let quiet = model_series("vi-4-schelling-25", seed, 200, "quiet");
        let first = quiet.iter().position(|&q| q == 1.0);
        println!(
            "vi-4 seed {seed}: segregation at t=0 {:.3}, first quiet tick {first:?}, segregation then {:.3}",
            seg[0],
            first.map_or(f64::NAN, |t| seg[t])
        );
    }
    for id in [
        "vi-5-schelling-25-residence",
        "vi-6-schelling-50-residence",
        "vi-7-schelling-mixed",
    ] {
        let late: Vec<String> = (1..=5)
            .map(|s| format!("{:.4}", late_segregation(id, s)))
            .collect();
        println!("{id}: late segregation (t=500–1000) seeds 1–5 {late:?}");
    }
}

/// From `measure_schelling` (release, seeds 1–5, recorded 2026-09-25): VI-4
/// first went quiet at t = 2–3 with segregation rising from 0.489–0.509 to
/// 0.618–0.639 (gains 0.122–0.150); late segregation was VI-5 0.756–0.763,
/// VI-6 0.943–0.950, VI-7 0.924–0.944 (per-seed VI-6 − VI-5 0.184–0.193).
/// Each threshold is the measured extreme rounded toward failing less
/// (quiet-by: twice the latest first quiet tick, up to a multiple of 10;
/// the others down to a multiple of 0.05).
const VI4_QUIET_BY: usize = 10;
const VI4_GAIN: f64 = 0.10;
const VI6_OVER_VI5: f64 = 0.15;
const VI7_AT_LEAST: f64 = 0.90;

#[test]
#[ignore]
fn schelling_25_percent_settles_into_a_more_segregated_pattern() {
    // Animation VI-4: "This process repeats until all agents are satisfied …
    // Notice that the final configuration is significantly less random—more
    // segregated—than the initial one."
    for seed in 1..=5 {
        let seg = model_series("vi-4-schelling-25", seed, 200, "segregation");
        let quiet = model_series("vi-4-schelling-25", seed, 200, "quiet");
        let first = quiet.iter().position(|&q| q == 1.0);
        let t = first.unwrap_or_else(|| panic!("seed {seed} never went quiet"));
        assert!(t <= VI4_QUIET_BY, "seed {seed}: first quiet at t = {t}");
        assert!(
            seg[t] - seg[0] >= VI4_GAIN,
            "seed {seed}: {} → {}",
            seg[0],
            seg[t]
        );
        assert!(
            quiet[t..].iter().all(|&q| q == 1.0),
            "seed {seed}: moved again after t = {t}"
        );
    }
}

#[test]
#[ignore]
fn schelling_50_percent_is_far_more_segregated_and_mixed_stays_high() {
    // VI-6: "a high degree of segregation results, far higher than in the
    // previous runs"; VI-7: "Adding this degree of tolerance is not
    // sufficient to generate desegregation—indeed, a highly segregated
    // pattern endures."
    for seed in 1..=5 {
        let vi5 = late_segregation("vi-5-schelling-25-residence", seed);
        let vi6 = late_segregation("vi-6-schelling-50-residence", seed);
        let vi7 = late_segregation("vi-7-schelling-mixed", seed);
        assert!(
            vi6 - vi5 >= VI6_OVER_VI5,
            "seed {seed}: VI-5 {vi5}, VI-6 {vi6}"
        );
        assert!(vi7 >= VI7_AT_LEAST, "seed {seed}: VI-7 {vi7}");
        assert!(
            (vi7 - vi6).abs() < (vi7 - vi5).abs(),
            "seed {seed}: VI-5 {vi5}, VI-6 {vi6}, VI-7 {vi7}"
        );
    }
}
```
Run: `cargo test -p sugarscape-core --release --test book measure_schelling -- --ignored --nocapture`
Expected (planning's figures, Decision 17):
```
vi-4 seed 1: segregation at t=0 0.506, first quiet tick Some(2), segregation then 0.633
vi-4 seed 2: segregation at t=0 0.489, first quiet tick Some(3), segregation then 0.639
vi-4 seed 3: segregation at t=0 0.491, first quiet tick Some(3), segregation then 0.622
vi-4 seed 4: segregation at t=0 0.496, first quiet tick Some(2), segregation then 0.618
vi-4 seed 5: segregation at t=0 0.509, first quiet tick Some(2), segregation then 0.636
vi-5-schelling-25-residence: late segregation (t=500–1000) seeds 1–5 ["0.7568", "0.7602", "0.7627", "0.7556", "0.7632"]
vi-6-schelling-50-residence: late segregation (t=500–1000) seeds 1–5 ["0.9501", "0.9468", "0.9464", "0.9430", "0.9475"]
vi-7-schelling-mixed: late segregation (t=500–1000) seeds 1–5 ["0.9241", "0.9308", "0.9326", "0.9313", "0.9440"]
```
If they differ, recompute the four constants by Decision 17's rules and update their comment and the preset descriptions (the rounded figures in `schelling::presets`).
Run: `cargo test -p sugarscape-core --release --test book schelling -- --ignored`
Expected: PASS (`schelling_25_percent_settles_into_a_more_segregated_pattern`, `schelling_50_percent_is_far_more_segregated_and_mixed_stays_high`, `measure_schelling`).

- [ ] **Step 8: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
cargo test --workspace
git add crates/sugarscape-core/src/schema.rs crates/sugarscape-core/src/schelling.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/book.rs
git commit -m "Add the book's Schelling segregation variant as a model kind" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 3: Ring World in the core

*Needs judgement: the implementation is given in full; Step 6's golden values and Step 7's thresholds come from runs (Decisions 10, 17, 18).* Browser (controller): nothing to check (not reachable from the page yet).

The book (Chapter VI, "Ring World"): "the landscape is a circle of sugar sites … agents search only in the counterclockwise direction on the sugar ring. Each agent has vision randomly chosen from some range (15 to 30 in the animations here). Subject to these strictures, the agent rule is: Inspect all unoccupied sites within your vision, select the nearest site with maximum sugar, go there and eat the sugar. As for the ring, there are 150 sites. Initially, the sugar level is distributed randomly between values of 0 and 4, and 40 agents are distributed randomly around the ring. The rule for the sites is that sugar grows back at unit rate to a capacity value, which is 4 … There is no death, birth, combat, cultural transmission, disease, or trade; no attention is paid to sugar accumulation. Each agent moves once each time period; agents are called in random order" (VI-8); "What if we start with all agents in one megagroup (of 40)? Will they disaggregate into like-sized cliques? The answer is 'yes'" (VI-9).

**Files:**
- Create: `crates/sugarscape-core/src/ring.rs`
- Modify: `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: Task 1's `model` items and `wrong_model`, Task 2's `schema::{Apply, Param}` and `check_schema`, `config::URange` (`sample`), `render::{lerp, Rgb, BACKGROUND, SUGAR}`.
- Produces:
  - `ring.rs`: `HISTORY: usize = 150`, `AGENT: Rgb = [0x4f, 0x9d, 0xff]`, `Start { Random, Megagroup }` (snake_case), `RingConfig { sites, agents, vision: URange, capacity: u32, growback: f64, start: Start }` (`Default` = VI-8, `validate()`), `SERIES: [&str; 5]`, `RingSnapshot`, `Walker { id, site, vision }`, `RingInspection`, `pub fn flocks(sites: &[u32], n: u32) -> Vec<u32>`, `RingWorld::{new, agents, sugar() -> &[f64], agent_sites() -> Vec<u32>, step, run, inspect(x)}` and `pub stats: Stats<RingSnapshot>`, `impl Model for RingWorld`, `schema()`, `presets()`.
  - `model.rs`: `ModelKind::Ring`, `ModelConfig::Ring(RingConfig)`, `ModelWorld::Ring(Box<RingWorld>)`, `ModelWorld::ring() -> Option<&RingWorld>`.

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/ring.rs` with its module doc and the tests only:
```rust
//! Ring World (Chapter VI, animations VI-8 and VI-9): sugar harvesters on a
//! circle of sites who look only counterclockwise, move to the nearest best
//! site they see and eat its sugar, and fall into flocks.

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring of `n` sites with sugar `sugar` everywhere and agents
    /// `(site, vision)` (ids 1, 2, … in that order).
    fn ring(n: u32, sugar: f64, agents: &[(u32, u32)]) -> RingWorld {
        let mut w = RingWorld::new(
            RingConfig {
                sites: n,
                agents: 0,
                vision: URange::new(1, 1),
                ..RingConfig::default()
            },
            1,
        )
        .unwrap();
        w.sugar = vec![sugar; n as usize];
        for (k, &(site, vision)) in agents.iter().enumerate() {
            let id = k as u64 + 1;
            w.occupant[site as usize] = Some(id);
            w.agents.insert(id, Walker { id, site, vision });
        }
        w
    }

    fn site(w: &RingWorld, id: u64) -> u32 {
        w.agents[&id].site
    }

    #[test]
    fn agents_look_counterclockwise_and_wrap() {
        let mut w = ring(12, 0.0, &[(10, 3)]);
        w.sugar[1] = 4.0; // distance 3 across the wrap
        w.sugar[9] = 4.0; // behind it: never seen
        assert_eq!(w.target(&w.agents[&1]), Some((1, 3)));
    }

    #[test]
    fn the_nearest_site_with_the_most_sugar_wins() {
        let mut w = ring(20, 1.0, &[(0, 6)]);
        w.sugar[3] = 3.0;
        w.sugar[5] = 3.0;
        w.sugar[6] = 2.0;
        assert_eq!(
            w.target(&w.agents[&1]),
            Some((3, 3)),
            "nearest of the two 3s"
        );
        let flat = ring(20, 0.0, &[(0, 6)]);
        assert_eq!(
            flat.target(&flat.agents[&1]),
            Some((1, 1)),
            "all zero: the nearest"
        );
    }

    #[test]
    fn occupied_sites_are_skipped_and_a_blocked_agent_stays() {
        let mut w = ring(20, 1.0, &[(0, 3), (1, 5), (3, 5)]);
        w.sugar[1] = 4.0;
        assert_eq!(w.target(&w.agents[&1]), Some((2, 2)), "site 1 is occupied");
        let blocked = ring(20, 4.0, &[(0, 2), (1, 5), (2, 5)]);
        assert_eq!(blocked.target(&blocked.agents[&1]), None);
    }

    #[test]
    fn movers_eat_and_sites_grow_back_after_everyone_moved() {
        let mut w = ring(20, 0.0, &[(0, 3)]);
        w.sugar[2] = 4.0;
        w.config.growback = 1.0;
        w.step();
        assert_eq!(site(&w, 1), 2);
        assert_eq!(w.sugar[2], 1.0, "eaten to 0, then grew back 1");
        assert_eq!(w.sugar[5], 1.0);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.mean_distance, s.population), (2.0, 1));
        w.config.capacity = 1;
        w.run(3);
        assert!(w.sugar.iter().all(|&s| s <= 1.0), "capped at capacity");
    }

    #[test]
    fn a_follower_leapfrogs_its_leader() {
        // The book's two-agent analysis: with all sugar at 4 the follower
        // jumps to the site just in front of the leader.
        let mut w = ring(30, 4.0, &[(5, 10), (6, 10)]);
        w.config.growback = 4.0;
        for _ in 0..20 {
            w.step();
            let (a, b) = (site(&w, 1), site(&w, 2));
            assert!((a + 30 - b) % 30 == 1 || (b + 30 - a) % 30 == 1, "{a} {b}");
        }
    }

    #[test]
    fn flocks_split_at_gaps_of_more_than_two_sites_across_the_wrap() {
        assert_eq!(flocks(&[], 10), Vec::<u32>::new());
        assert_eq!(flocks(&[4], 10), vec![1]);
        assert_eq!(flocks(&[0, 1, 3], 10), vec![3], "gaps of 1 and 2");
        assert_eq!(flocks(&[0, 1, 4], 10), vec![1, 2], "a gap of 3 splits");
        // 9 → 0 wraps: 8, 9, 0, 1 is one flock; 5 is alone.
        let mut sizes = flocks(&[0, 1, 5, 8, 9], 10);
        sizes.sort_unstable();
        assert_eq!(sizes, vec![1, 4]);
        assert_eq!(flocks(&[0, 2, 4, 6, 8], 10), vec![5], "every gap is 2");
    }

    #[test]
    fn statistics_count_flocks() {
        let w = ring(
            30,
            0.0,
            &[(0, 1), (1, 1), (2, 1), (10, 1), (20, 1), (21, 1)],
        );
        let s = w.snapshot();
        assert_eq!((s.flocks, s.largest_flock, s.population), (3, 3, 6));
        assert_eq!(s.mean_flock, 2.0);
    }

    #[test]
    fn setup_scatters_or_masses_the_agents() {
        let random = RingWorld::new(RingConfig::default(), 2).unwrap();
        assert_eq!(random.agents().count(), 40);
        assert!(random.agents().all(|a| (15..=30).contains(&a.vision)));
        assert!(random
            .sugar
            .iter()
            .all(|&s| s.fract() == 0.0 && (0.0..=4.0).contains(&s)));
        assert!(random.sugar.contains(&0.0) && random.sugar.contains(&4.0));
        let mega = RingWorld::new(
            RingConfig {
                start: Start::Megagroup,
                ..RingConfig::default()
            },
            2,
        )
        .unwrap();
        assert_eq!(flocks(&mega.agent_sites(), 150), vec![40]);
        let mut sites = mega.agent_sites();
        sites.sort_unstable();
        let gaps = sites.windows(2).filter(|p| p[1] - p[0] != 1).count();
        assert!(gaps <= 1, "consecutive sites, possibly across the wrap");
        assert_eq!(mega.stats.latest().unwrap().flocks, 1);
    }

    #[test]
    fn the_space_time_diagram_keeps_the_last_150_ticks() {
        let mut w = RingWorld::new(RingConfig::default(), 3).unwrap();
        let mut buf = Vec::new();
        w.render("", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 150 * 150 * 4);
        assert_eq!(&buf[..3], &BACKGROUND, "rows before t = 0 are dark");
        let bottom = &buf[149 * 150 * 4..];
        let agents = bottom.chunks_exact(4).filter(|p| p[..3] == AGENT).count();
        assert_eq!(agents, 40, "the current tick is the bottom row");
        w.run(200);
        assert_eq!(w.history.len(), HISTORY);
        w.render("", "", &mut buf).unwrap();
        let top = &buf[..150 * 4];
        assert_eq!(
            top.chunks_exact(4).filter(|p| p[..3] == AGENT).count(),
            40,
            "full: no dark rows"
        );
        assert_eq!(Model::locate(&w, 1).unwrap().1, 149);
    }

    #[test]
    fn validation_names_fields() {
        let bad = RingConfig {
            sites: 5,
            agents: 5,
            vision: URange::new(0, 9),
            capacity: 0,
            growback: 0.0,
            start: Start::Random,
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            [
                "sites",
                "agents",
                "vision.min",
                "vision.max",
                "capacity",
                "growback"
            ]
        );
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Ring(RingConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }

    #[test]
    fn presets_follow_the_book() {
        let ps = presets();
        let configs: Vec<RingConfig> = ps
            .iter()
            .map(|p| match &p.config {
                ModelConfig::Ring(c) => c.clone(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(configs[0], RingConfig::default());
        assert_eq!(
            (
                configs[0].sites,
                configs[0].agents,
                configs[0].vision,
                configs[0].capacity
            ),
            (150, 40, URange::new(15, 30), 4)
        );
        assert_eq!(configs[1].start, Start::Megagroup);
        assert_eq!(
            RingConfig {
                start: Start::Random,
                ..configs[1].clone()
            },
            configs[0]
        );
    }
}
```
In `crates/sugarscape-core/src/lib.rs`, after `pub mod render;` add `pub mod ring;`.

In `crates/sugarscape-core/src/model.rs`'s tests module, before `fn unknown_models_are_field_errors`, add:
```rust
    #[test]
    fn ring_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "ring", "start": "megagroup"}"#).unwrap();
        assert_eq!(c.kind(), ModelKind::Ring);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["sites"].as_u64()),
            (Some("ring"), Some(150))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(
            c.series_names(),
            [
                "flocks",
                "mean_flock",
                "largest_flock",
                "mean_distance",
                "population"
            ]
        );
        let e = ModelConfig::from_json(r#"{"model": "ring", "start": "clumps"}"#).unwrap_err();
        assert_eq!(e[0].field, "config");
    }

    #[test]
    fn every_kind_names_itself_and_only_other_models_have_schemas() {
        let names: Vec<&str> = ModelKind::ALL.iter().map(|k| k.as_str()).collect();
        assert_eq!(names, ["sugarscape", "schelling", "ring"]);
        assert!(ModelKind::Sugarscape.schema().is_empty());
        assert!(!ModelKind::Schelling.schema().is_empty() && !ModelKind::Ring.schema().is_empty());
    }

```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core --lib ring`
Expected: FAIL to compile — `RingWorld`, `RingConfig`, `ModelKind::Ring` not found.

- [ ] **Step 3: Implement**

`crates/sugarscape-core/src/ring.rs` — between the module doc and the tests, add (Decision 10):
```rust
use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::{FieldError, URange};
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::presets::ModelPreset;
use crate::render::{lerp, Rgb, BACKGROUND, SUGAR};
use crate::rng::{self, SimRng};
use crate::schema::{Apply, Param};
use crate::stats::{Series, Stats};

/// Rows of the space–time diagram: the last this many ticks.
pub const HISTORY: usize = 150;
/// An agent in the space–time diagram: the ramp's cool blue, which stands
/// out against both dark and full sites.
pub const AGENT: Rgb = [0x4f, 0x9d, 0xff];

/// How the agents start: scattered, or VI-9's "one megagroup".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// On random distinct sites.
    Random,
    /// On consecutive sites from a random site.
    Megagroup,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RingConfig {
    pub sites: u32,
    pub agents: u32,
    /// Each agent's vision, uniform in `[min, max]`.
    pub vision: URange,
    /// Sugar a site holds at most; initial sugar is uniform in `0..=capacity`.
    pub capacity: u32,
    /// Sugar every site grows back each tick, up to `capacity`.
    pub growback: f64,
    pub start: Start,
}

impl Default for RingConfig {
    /// Animation VI-8: "there are 150 sites. Initially, the sugar level is
    /// distributed randomly between values of 0 and 4, and 40 agents are
    /// distributed randomly around the ring … vision randomly chosen from
    /// some range (15 to 30) … sugar grows back at unit rate to a capacity
    /// value, which is 4".
    fn default() -> Self {
        RingConfig {
            sites: 150,
            agents: 40,
            vision: URange::new(15, 30),
            capacity: 4,
            growback: 1.0,
            start: Start::Random,
        }
    }
}

impl RingConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (10..=1000).contains(&self.sites),
            "sites",
            "must be between 10 and 1000",
        );
        check(
            self.agents < self.sites,
            "agents",
            "must be less than the number of sites",
        );
        let v = self.vision;
        check(v.min >= 1, "vision.min", "must be ≥ 1");
        check(v.min <= v.max, "vision", "min must be ≤ max");
        check(
            v.max < self.sites,
            "vision.max",
            "must be less than the number of sites",
        );
        check(
            (1..=100).contains(&self.capacity),
            "capacity",
            "must be between 1 and 100",
        );
        check(
            self.growback.is_finite() && self.growback > 0.0,
            "growback",
            "must be a number > 0",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next` (capacity and
    /// growback apply to the running world).
    fn structural_changes(&self, next: &RingConfig) -> Vec<FieldError> {
        let msg = "changes only on reset";
        let mut out = Vec::new();
        for (field, same) in [
            ("sites", self.sites == next.sites),
            ("agents", self.agents == next.agents),
            ("vision", self.vision == next.vision),
            ("start", self.start == next.start),
        ] {
            if !same {
                out.push(FieldError::new(field, msg));
            }
        }
        out
    }
}

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 5] = [
    "flocks",
    "mean_flock",
    "largest_flock",
    "mean_distance",
    "population",
];

/// One tick's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct RingSnapshot {
    pub tick: u64,
    pub population: u32,
    /// Flocks (see `flocks`); 0 with no agents.
    pub flocks: u32,
    /// Agents per flock (0 with no agents).
    pub mean_flock: f64,
    pub largest_flock: u32,
    /// Sites moved per agent this tick (0 at t = 0).
    pub mean_distance: f64,
}

impl Series for RingSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "flocks" => f64::from(self.flocks),
            "mean_flock" => self.mean_flock,
            "largest_flock" => f64::from(self.largest_flock),
            "mean_distance" => self.mean_distance,
            _ => return None,
        })
    }
}

/// One harvester: its site and vision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Walker {
    pub id: u64,
    pub site: u32,
    pub vision: u32,
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RingInspection {
    pub site: RingSiteView,
    pub agent: Option<WalkerView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct RingSiteView {
    /// The site's index (counterclockwise from 0).
    pub x: u32,
    pub sugar: f64,
    pub capacity: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct WalkerView {
    pub id: u64,
    pub vision: u32,
}

/// The sizes of the flocks formed by agents at `sites` on a ring of `n`
/// sites, starting after the widest-apart pair: a flock is a maximal run of
/// agents (in ring order) whose consecutive gaps are at most 2 sites — at
/// most one empty site between neighbors. (The book does not define a
/// flock; the spec states this one.)
pub fn flocks(sites: &[u32], n: u32) -> Vec<u32> {
    let mut sorted = sites.to_vec();
    sorted.sort_unstable();
    let k = sorted.len();
    if k == 0 {
        return Vec::new();
    }
    // The gap after agent i, to the next around the ring.
    let gap = |i: usize| (sorted[(i + 1) % k] + n - sorted[i]) % n;
    let gaps: Vec<u32> = (0..k).map(|i| if k == 1 { n } else { gap(i) }).collect();
    let Some(first_break) = gaps.iter().position(|&g| g > 2) else {
        return vec![k as u32];
    };
    let mut out = Vec::new();
    let mut size = 0;
    for step in 1..=k {
        let i = (first_break + step) % k;
        size += 1;
        if gaps[i] > 2 {
            out.push(size);
            size = 0;
        }
    }
    out
}

pub struct RingWorld {
    pub config: RingConfig,
    /// Completed ticks.
    pub tick: u64,
    sugar: Vec<f64>,
    occupant: Vec<Option<u64>>,
    agents: BTreeMap<u64, Walker>,
    rng: SimRng,
    /// Sites moved by all agents this tick.
    distance: u64,
    /// The space–time diagram's rows (oldest first, at most `HISTORY`): each
    /// site's sugar as a fraction of capacity, or `None` where an agent stood.
    history: VecDeque<Vec<Option<f32>>>,
    pub stats: Stats<RingSnapshot>,
}

impl RingWorld {
    pub fn new(config: RingConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let n = config.sites as usize;
        let mut rng = rng::seeded(seed);
        let sugar = (0..n)
            .map(|_| f64::from(rng.gen_range(0..=config.capacity)))
            .collect();
        let places: Vec<u32> = match config.start {
            Start::Random => {
                let mut all: Vec<u32> = (0..config.sites).collect();
                all.shuffle(&mut rng);
                all.truncate(config.agents as usize);
                all
            }
            Start::Megagroup => {
                let from = rng.gen_range(0..config.sites);
                (0..config.agents)
                    .map(|k| (from + k) % config.sites)
                    .collect()
            }
        };
        let mut world = RingWorld {
            tick: 0,
            sugar,
            occupant: vec![None; n],
            agents: BTreeMap::new(),
            rng,
            distance: 0,
            history: VecDeque::with_capacity(HISTORY),
            stats: Stats::default(),
            config,
        };
        for (k, site) in places.into_iter().enumerate() {
            let id = k as u64 + 1;
            let vision = world.config.vision.sample(&mut world.rng);
            world.occupant[site as usize] = Some(id);
            world.agents.insert(id, Walker { id, site, vision });
        }
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> impl Iterator<Item = &Walker> {
        self.agents.values()
    }

    /// Each site's sugar, site 0 first.
    pub fn sugar(&self) -> &[f64] {
        &self.sugar
    }

    /// Each agent's site, in id order.
    pub fn agent_sites(&self) -> Vec<u32> {
        self.agents.values().map(|a| a.site).collect()
    }

    /// Where agent `a` goes: among the unoccupied sites at distances
    /// 1…vision counterclockwise (increasing index), the nearest with the
    /// most sugar (even if that is 0), as (site, distance); `None` if every
    /// one is occupied.
    fn target(&self, a: &Walker) -> Option<(u32, u32)> {
        let n = self.config.sites;
        let mut best: Option<(u32, u32, f64)> = None;
        for d in 1..=a.vision {
            let s = (a.site + d) % n;
            if self.occupant[s as usize].is_some() {
                continue;
            }
            let sugar = self.sugar[s as usize];
            if best.is_none_or(|(_, _, b)| sugar > b) {
                best = Some((s, d, sugar));
            }
        }
        best.map(|(s, d, _)| (s, d))
    }

    /// One tick: agents act in a random order — "Inspect all unoccupied sites
    /// within your vision, select the nearest site with maximum sugar, go
    /// there and eat the sugar" — then every site grows back.
    pub fn step(&mut self) {
        let mut order: Vec<u64> = self.agents.keys().copied().collect();
        order.shuffle(&mut self.rng);
        let mut distance = 0;
        for id in order {
            let a = self.agents[&id];
            let Some((to, d)) = self.target(&a) else {
                continue;
            };
            self.occupant[a.site as usize] = None;
            self.occupant[to as usize] = Some(id);
            self.agents.get_mut(&id).expect("living agent").site = to;
            self.sugar[to as usize] = 0.0;
            distance += u64::from(d);
        }
        let (rate, cap) = (self.config.growback, f64::from(self.config.capacity));
        for s in &mut self.sugar {
            *s = (*s + rate).min(cap);
        }
        self.distance = distance;
        self.tick += 1;
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    /// Pushes this tick's statistics and space–time row.
    fn record(&mut self) {
        let cap = f64::from(self.config.capacity);
        let row = self
            .sugar
            .iter()
            .zip(&self.occupant)
            .map(|(&s, o)| o.is_none().then_some((s / cap) as f32))
            .collect();
        if self.history.len() == HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(row);
        let snapshot = self.snapshot();
        self.stats.push(snapshot);
    }

    fn snapshot(&self) -> RingSnapshot {
        let n = self.agents.len();
        let sizes = flocks(&self.agent_sites(), self.config.sites);
        RingSnapshot {
            tick: self.tick,
            population: n as u32,
            flocks: sizes.len() as u32,
            mean_flock: if sizes.is_empty() {
                0.0
            } else {
                n as f64 / sizes.len() as f64
            },
            largest_flock: sizes.iter().copied().max().unwrap_or(0),
            mean_distance: if n == 0 {
                0.0
            } else {
                self.distance as f64 / n as f64
            },
        }
    }

    pub fn inspect(&self, x: u32) -> Result<RingInspection, String> {
        if x >= self.config.sites {
            return Err(format!("site {x} is not on the ring"));
        }
        Ok(RingInspection {
            site: RingSiteView {
                x,
                sugar: self.sugar[x as usize],
                capacity: self.config.capacity,
            },
            agent: self.occupant[x as usize].map(|id| {
                let a = self.agents[&id];
                WalkerView {
                    id,
                    vision: a.vision,
                }
            }),
        })
    }
}

impl Model for RingWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Ring(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        RingWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick, every site's sugar, and every agent's id, site
    /// and vision.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for s in &self.sugar {
            eat(s.to_bits());
        }
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.site) << 32) | u64::from(a.vision));
        }
        h
    }

    /// The space–time diagram: one column per site, one row per tick, the
    /// current tick at the bottom.
    fn size(&self) -> (u32, u32) {
        (self.config.sites, HISTORY as u32)
    }

    /// Sugar shaded from dark to the sugar color, agents light; rows before
    /// t = 0 dark. `mode` and `layer` are ignored.
    fn render(&self, _mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let n = self.config.sites as usize;
        buf.clear();
        buf.resize(n * HISTORY * 4, 0);
        let blank = HISTORY - self.history.len();
        for (y, px) in buf.chunks_exact_mut(n * 4).enumerate() {
            let row = y.checked_sub(blank).map(|r| &self.history[r]);
            for (x, p) in px.chunks_exact_mut(4).enumerate() {
                let rgb = match row.map(|r| r[x]) {
                    None => BACKGROUND,
                    Some(None) => AGENT,
                    Some(Some(level)) => lerp(BACKGROUND, SUGAR, f64::from(level)),
                };
                p.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,site,vision\n");
        for a in self.agents.values() {
            writeln!(out, "{},{},{}", a.id, a.site, a.vision).unwrap();
        }
        out
    }

    /// Site `x` (`y`, a row of the diagram, is ignored).
    fn inspect_json(&self, x: u32, _y: u32) -> Result<String, String> {
        let inspection = self.inspect(x)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// The agent's site, in the diagram's current (bottom) row.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        self.agents.get(&id).map(|a| (a.site, HISTORY as u32 - 1))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Ring(next) = next else {
            return Err(wrong_model(ModelKind::Ring, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }
}

/// The Rules panel's fields: capacity and growback apply to the running
/// world; the rest rebuild it.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Setup", "sites", "Sites", (10, 1000), Reset),
        Param::integer("Setup", "agents", "Agents", (1, 999), Reset),
        Param::choice(
            "Setup",
            "start",
            "Start",
            &[
                ("random", "Scattered at random"),
                ("megagroup", "One megagroup"),
            ],
            Reset,
        ),
        Param::range(
            "Agents",
            "vision",
            "Vision (sites)",
            (1.0, 100.0, 1.0),
            Reset,
        ),
        Param::integer("Sugar", "capacity", "Capacity", (1, 20), Live),
        Param::number(
            "Sugar",
            "growback",
            "Growback per tick",
            (0.1, 10.0, 0.1),
            Live,
        ),
    ]
}

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    start: Start,
) -> ModelPreset {
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Ring(RingConfig {
            start,
            ..RingConfig::default()
        }),
    }
}

/// Animations VI-8 and VI-9.
pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "vi-8-ring-world",
            "Ring World: flocking on the ring",
            "Animation VI-8",
            "40 agents with vision 15–30 on a ring of 150 sugar sites (0–4 sugar, growing back 1 per tick) look only counterclockwise, jump to the nearest richest empty site they see and eat it. With no social rule at all they fall into flocks that tread around the ring.",
            Start::Random,
        ),
        preset(
            "vi-9-ring-megagroup",
            "Ring World: a megagroup breaks up",
            "Animation VI-9",
            "The same ring with all 40 agents starting as one megagroup on consecutive sites: nobody can see past the front of a group of 40, so it breaks up into smaller flocks.",
            Start::Megagroup,
        ),
    ]
}
```

`crates/sugarscape-core/src/model.rs`:
- Imports: add `use crate::ring::{RingConfig, RingWorld};` before the `schelling` import; `use crate::{export, schelling, stats};` becomes `use crate::{export, ring, schelling, stats};`.
- `ModelKind` gains `Ring`; `ALL` becomes `pub const ALL: [ModelKind; 3] = [ModelKind::Sugarscape, ModelKind::Schelling, ModelKind::Ring];`; `as_str` gains `ModelKind::Ring => "ring",`; `schema` gains `ModelKind::Ring => ring::schema(),`.
- `ModelConfig` gains `Ring(RingConfig),`; `Tagged` gains `Ring(&'a RingConfig),`; `Serialize` gains `ModelConfig::Ring(c) => Tagged::Ring(c).serialize(s),`; `kind()` gains `ModelConfig::Ring(_) => ModelKind::Ring,`.
- In `from_value`'s match, before the `_ =>` arm add
  ```rust
              "ring" => serde_json::from_value(value)
                  .map(ModelConfig::Ring)
                  .map_err(|e| FieldError::new("config", e.to_string())),
  ```
  and the error message becomes `format!("unknown model {tag:?} (expected sugarscape, schelling or ring)")`.
- `validate()` gains `ModelConfig::Ring(c) => c.validate(),`; `with_path()` gains `ModelConfig::Ring(c) => set_path(c, path, value).map(ModelConfig::Ring),`; `series_names()` gains `ModelConfig::Ring(_) => ring::SERIES.iter().map(|s| s.to_string()).collect(),`.
- `ModelWorld` gains `Ring(Box<RingWorld>),`; `with_landscapes` gains `ModelConfig::Ring(c) => ModelWorld::Ring(Box::new(RingWorld::new(c, seed)?)),`; `kind()`, `model()`, `model_mut()` gain `ModelWorld::Ring(_) => ModelKind::Ring,`, `ModelWorld::Ring(w) => w.as_ref(),`, `ModelWorld::Ring(w) => w.as_mut(),`; and after `sugarscape_mut` add
  ```rust
      pub fn ring(&self) -> Option<&RingWorld> {
          match self {
              ModelWorld::Ring(w) => Some(w),
              _ => None,
          }
      }
  ```

`crates/sugarscape-core/src/presets.rs` — `catalog` becomes
```rust
/// Every model's presets: the sugarscape's (`all`), then Schelling's, then
/// Ring World's.
pub fn catalog() -> Vec<ModelPreset> {
    let mut out: Vec<ModelPreset> = all().into_iter().map(ModelPreset::from).collect();
    out.extend(crate::schelling::presets());
    out.extend(crate::ring::presets());
    out
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core --lib`
Expected: PASS — including `a_follower_leapfrogs_its_leader` (the book's two-agent analysis), `flocks_split_at_gaps_of_more_than_two_sites_across_the_wrap` and `the_space_time_diagram_keeps_the_last_150_ticks`.

- [ ] **Step 5: Record the golden fingerprints**

Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden`
Expected, after the four Schelling lines:
```
    ("vi-8-ring-world", 0x1c341361c466db90),
    ("vi-9-ring-megagroup", 0x430d0c3b19b6e58e),
```
Append exactly these two entries to `MODEL_GOLDEN` (stop and report if they differ, Decision 18).
Run: `cargo test -p sugarscape-core --test golden`
Expected: PASS.

- [ ] **Step 6: The book-style tests (measured)**

Append to `crates/sugarscape-core/tests/book.rs`:
```rust
/// The mean of `values` over ticks 500–1000.
fn late_mean(values: &[f64]) -> f64 {
    values[500..=1000].iter().sum::<f64>() / 501.0
}

/// Prints the figures the Ring World thresholds below come from.
#[test]
#[ignore]
fn measure_ring_world() {
    for id in ["vi-8-ring-world", "vi-9-ring-megagroup"] {
        for seed in 1..=5 {
            let flocks = model_series(id, seed, 1000, "flocks");
            let size = model_series(id, seed, 1000, "mean_flock");
            let largest = model_series(id, seed, 1000, "largest_flock");
            println!(
                "{id} seed {seed}: t=0 {} flocks of {:.2}; t=500–1000 mean {:.2} flocks of {:.2}, largest ever {}",
                flocks[0],
                size[0],
                late_mean(&flocks),
                late_mean(&size),
                largest[500..].iter().copied().fold(0.0, f64::max)
            );
        }
    }
}

/// From `measure_ring_world` (release, seeds 1–5, recorded 2026-09-25):
/// VI-8 starts as 20–25 flocks of 1.60–2.00 agents and settles to 6.75–8.16
/// flocks of 5.06–6.09 (growth 2.66–3.30×); VI-9 starts as 1 flock of 40
/// and settles to 6.94–7.78 flocks, the largest never above 14 after
/// t = 500. Counts are rounded down to a whole number, factors down to a
/// multiple of 0.5 and the cap up to a multiple of 5.
const RING_LATE_FLOCKS_AT_LEAST: f64 = 6.0;
const RING_FLOCK_GROWTH: f64 = 2.5;
const RING_MEGAGROUP_LARGEST_AT_MOST: f64 = 15.0;

#[test]
#[ignore]
fn ring_world_agents_cluster_into_several_flocks() {
    // Animation VI-8: "Remarkably, the agents cluster into groups. Often they
    // separate into groups of comparable size!"
    for seed in 1..=5 {
        let flocks = model_series("vi-8-ring-world", seed, 1000, "flocks");
        let size = model_series("vi-8-ring-world", seed, 1000, "mean_flock");
        assert!(
            late_mean(&flocks) >= RING_LATE_FLOCKS_AT_LEAST,
            "seed {seed}: {}",
            late_mean(&flocks)
        );
        assert!(
            late_mean(&size) >= RING_FLOCK_GROWTH * size[0],
            "seed {seed}: flocks of {} at t = 0, {} late",
            size[0],
            late_mean(&size)
        );
    }
}

#[test]
#[ignore]
fn ring_world_megagroup_breaks_up() {
    // Animation VI-9: "What if we start with all agents in one megagroup (of
    // 40)? Will they disaggregate into like-sized cliques? The answer is
    // 'yes'."
    for seed in 1..=5 {
        let flocks = model_series("vi-9-ring-megagroup", seed, 1000, "flocks");
        let largest = model_series("vi-9-ring-megagroup", seed, 1000, "largest_flock");
        assert_eq!(flocks[0], 1.0, "seed {seed} starts as one flock");
        assert!(
            late_mean(&flocks) >= RING_LATE_FLOCKS_AT_LEAST,
            "seed {seed}: {}",
            late_mean(&flocks)
        );
        let most = largest[500..].iter().copied().fold(0.0, f64::max);
        assert!(
            most <= RING_MEGAGROUP_LARGEST_AT_MOST,
            "seed {seed}: a flock of {most}"
        );
    }
}
```
Run: `cargo test -p sugarscape-core --release --test book measure_ring_world -- --ignored --nocapture`
Expected (planning's figures, Decision 17; 1000 ticks take about 6 ms each):
```
vi-8-ring-world seed 1: t=0 22 flocks of 1.82; t=500–1000 mean 7.84 flocks of 5.21, largest ever 14
vi-8-ring-world seed 2: t=0 21 flocks of 1.90; t=500–1000 mean 8.16 flocks of 5.06, largest ever 15
vi-8-ring-world seed 3: t=0 20 flocks of 2.00; t=500–1000 mean 6.75 flocks of 6.09, largest ever 13
vi-8-ring-world seed 4: t=0 22 flocks of 1.82; t=500–1000 mean 7.81 flocks of 5.25, largest ever 14
vi-8-ring-world seed 5: t=0 25 flocks of 1.60; t=500–1000 mean 7.72 flocks of 5.31, largest ever 13
vi-9-ring-megagroup seed 1: t=0 1 flocks of 40.00; t=500–1000 mean 7.51 flocks of 5.53, largest ever 13
vi-9-ring-megagroup seed 2: t=0 1 flocks of 40.00; t=500–1000 mean 6.94 flocks of 6.03, largest ever 12
vi-9-ring-megagroup seed 3: t=0 1 flocks of 40.00; t=500–1000 mean 7.51 flocks of 5.59, largest ever 13
vi-9-ring-megagroup seed 4: t=0 1 flocks of 40.00; t=500–1000 mean 7.78 flocks of 5.30, largest ever 13
vi-9-ring-megagroup seed 5: t=0 1 flocks of 40.00; t=500–1000 mean 7.25 flocks of 5.69, largest ever 14
```
If they differ, recompute the three constants by Decision 17's rules and update their comment.
Run: `cargo test -p sugarscape-core --release --test book ring -- --ignored`
Expected: PASS (`ring_world_agents_cluster_into_several_flocks`, `ring_world_megagroup_breaks_up`, `measure_ring_world`).

- [ ] **Step 7: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
cargo test --workspace
git add crates/sugarscape-core/src/ring.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/book.rs
git commit -m "Add Ring World as a model kind" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---


### Task 4: Every model in WASM

*Mechanical (full code).* Browser (controller): the page still starts on `ii-2-unit` and every existing scenario works (the presets menu now also lists the six new presets at the end, ungrouped; do not choose them until Task 6).

**Files:**
- Modify: `crates/sugarscape-wasm/src/lib.rs`
- Test: `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `model::{Model, ModelConfig, ModelKind, ModelWorld}`, `ModelWorld::ring`, `RingWorld::{sugar, agent_sites}`, `presets::catalog`, `ModelKind::{ALL, schema}` (Tasks 1–3).
- Produces (Decision 5): `presets_json()` (the catalog), `model_schemas_json() -> String`, `config_series_names` for any model; `Sim` over a `ModelWorld` with `model_kind() -> String`, `ring_sugar() -> Vec<f64>`, `ring_agents() -> Vec<u32>`; every other `Sim` method keeps its name and signature.

- [ ] **Step 1: Write the failing tests**

In `crates/sugarscape-wasm/tests/web.rs`, replace the `use sugarscape_wasm::{…};` block with
```rust
use sugarscape_wasm::{
    aggregate, builtin_sweeps, config_series_names, fingerprint_hex, model_schemas_json,
    parse_sweep, presets_json, run_point, sweep_csv, sweep_points, sweep_result, Sim,
};
```
and append:
```rust
/// Preset `id`'s config JSON (any model).
fn preset_json(id: &str) -> String {
    serde_json::to_string(&sugarscape_core::presets::find(id).unwrap().config).unwrap()
}

#[wasm_bindgen_test]
fn presets_list_every_model_with_sugarscape_configs_untagged() {
    let list: Vec<serde_json::Value> = serde_json::from_str(&presets_json()).unwrap();
    let model = |id: &str| {
        let p = list.iter().find(|p| p["id"] == id).unwrap();
        p["config"]
            .get("model")
            .and_then(|m| m.as_str())
            .map(String::from)
    };
    assert_eq!(model("ii-2-unit"), None);
    assert_eq!(model("vi-4-schelling-25").as_deref(), Some("schelling"));
    assert_eq!(model("vi-9-ring-megagroup").as_deref(), Some("ring"));
    let first = &list[0];
    let direct = serde_json::to_value(&sugarscape_core::presets::all()[0]).unwrap();
    assert_eq!(first, &direct, "sugarscape presets serialize as before");
}

#[wasm_bindgen_test]
fn schemas_are_listed_for_the_other_models() {
    let schemas: serde_json::Value = serde_json::from_str(&model_schemas_json()).unwrap();
    assert!(schemas.get("sugarscape").is_none());
    let schelling = schemas["schelling"].as_array().unwrap();
    assert!(schelling
        .iter()
        .any(|p| p["path"] == "preference" && p["kind"] == "range" && p["apply"] == "reset"));
    let ring = schemas["ring"].as_array().unwrap();
    let growback = ring.iter().find(|p| p["path"] == "growback").unwrap();
    assert_eq!(
        (growback["kind"].as_str(), growback["apply"].as_str()),
        (Some("number"), Some("live"))
    );
    let start = ring.iter().find(|p| p["path"] == "start").unwrap();
    assert_eq!(start["choices"][1]["value"], "megagroup");
    assert!(start.get("min").is_none());
}

#[wasm_bindgen_test]
fn a_schelling_sim_runs_renders_and_inspects() {
    let mut sim = Sim::new(&preset_json("vi-4-schelling-25"), 1, JsValue::NULL).unwrap();
    assert_eq!(sim.model_kind(), "schelling");
    assert_eq!(
        (sim.width(), sim.height(), sim.population()),
        (50, 50, 2000)
    );
    sim.render("satisfaction", "resource:0").unwrap();
    assert_eq!(sim.frame_len(), 50 * 50 * 4);
    assert!(sim.render("tribe", "resource:0").is_err());
    sim.step(200);
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
    assert_eq!(sim.fingerprint(), "0x7a7072c3433f5f6f");
    let latest: serde_json::Value = serde_json::from_str(&sim.stats_latest()).unwrap();
    assert_eq!(
        (latest["tick"].as_u64(), latest["quiet"].as_u64()),
        (Some(200), Some(1))
    );
    let names: Vec<String> = serde_json::from_str(&sim.series_names()).unwrap();
    assert_eq!(names[..2], ["unsatisfied", "segregation"]);
    assert_eq!(sim.series("segregation").unwrap().len(), 201);
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(0, 0).unwrap()).unwrap();
    assert_eq!(view["site"]["x"], 0);
    let config: serde_json::Value = serde_json::from_str(&sim.export_config()).unwrap();
    assert_eq!(config["model"], "schelling");
    assert!(sim
        .export_series_csv()
        .starts_with("tick,unsatisfied,segregation,"));
    assert!(sim.export_agents_csv().starts_with("id,x,y,color,"));
}

#[wasm_bindgen_test]
fn sugarscape_only_calls_are_empty_or_refused_in_other_models() {
    let mut sim = Sim::new(&preset_json("vi-8-ring-world"), 1, JsValue::NULL).unwrap();
    assert!(sim.lorenz(101).is_empty() && sim.wealth_hist(20).is_empty());
    assert!(sim.age_hist(5).is_empty() && sim.tag_hist().is_empty());
    assert!(sim.lorenz_total(101).is_empty() && sim.supply_demand().is_empty());
    assert!(sim.good_wealth_hist(0, 20).is_err());
    assert!(sim.networks("trade").unwrap().is_empty());
    assert_eq!(sim.credit_graph(), r#"{"agents":[],"loans":[]}"#);
    assert_eq!(sim.disease_list(), "[]");
    assert!(!sim.landscape_edited(0) && sim.export_landscape(0).is_err());
    assert!(sim.paint_capacity(0, 0, 1, 1.0, 0).is_err());
    assert!(sim.place_agent(0, 0, "{}").is_err());
    assert!(sim.remove_agent(0, 0).is_err());
    sim.follow(1.0);
    assert_eq!((sim.followed(), sim.trail().len()), (-1.0, 0));
}

#[wasm_bindgen_test]
fn a_ring_sim_draws_its_space_time_diagram_and_reports_its_ring() {
    let mut sim = Sim::new(&preset_json("vi-8-ring-world"), 1, JsValue::NULL).unwrap();
    assert_eq!(sim.model_kind(), "ring");
    assert_eq!((sim.width(), sim.height()), (150, 150));
    assert_eq!((sim.ring_sugar().len(), sim.ring_agents().len()), (150, 40));
    sim.render("", "").unwrap();
    assert_eq!(sim.frame_len(), 150 * 150 * 4);
    sim.step(200);
    assert_eq!(sim.fingerprint(), "0x1c341361c466db90");
    let site = sim.ring_agents()[0];
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(site, 7).unwrap()).unwrap();
    assert_eq!(view["agent"]["id"], 1);
    assert_eq!(sim.locate(1.0), Some(vec![site, 149]));
    assert!(sim.inspect(150, 0).is_err());
    // Capacity and growback apply live; the ring's size does not.
    let mut config: serde_json::Value = serde_json::from_str(&sim.export_config()).unwrap();
    config["growback"] = serde_json::json!(2.0);
    sim.set_config(&config.to_string()).unwrap();
    config["sites"] = serde_json::json!(200);
    let err = sim
        .set_config(&config.to_string())
        .unwrap_err()
        .as_string()
        .unwrap();
    assert!(err.contains(r#""field":"sites""#), "{err}");
    assert!(Sim::new(&preset_json("ii-2-unit"), 1, JsValue::NULL)
        .unwrap()
        .ring_sugar()
        .is_empty());
    let names: Vec<String> =
        serde_json::from_str(&config_series_names(r#"{"model":"ring"}"#).unwrap()).unwrap();
    assert_eq!(names[0], "flocks");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: FAIL to compile — `model_schemas_json`, `Sim::model_kind`, `Sim::ring_sugar` not found.

- [ ] **Step 3: Implement**

`crates/sugarscape-wasm/src/lib.rs`:
- Replace the core imports with
  ```rust
  use sugarscape_core::config::{Config, FieldError};
  use sugarscape_core::edit::AgentOverrides;
  use sugarscape_core::model::{Model, ModelConfig, ModelKind, ModelWorld};
  use sugarscape_core::sweep::{RunResult, Sweep, SweepResult};
  use sugarscape_core::world::World;
  use sugarscape_core::{network, presets, stats, sweep};
  ```
- Replace `presets_json` with
  ```rust
  /// JSON list of every model's presets (`presets::catalog`): the
  /// sugarscape's first, each exactly as before milestone 9.
  #[wasm_bindgen]
  pub fn presets_json() -> String {
      serde_json::to_string(&presets::catalog()).expect("presets serialize")
  }

  /// JSON `{ <model>: [param, …] }`: the Rules panel's fields of every model
  /// that has a schema (every model but the sugarscape).
  #[wasm_bindgen]
  pub fn model_schemas_json() -> String {
      let schemas: serde_json::Map<String, serde_json::Value> = ModelKind::ALL
          .iter()
          .filter(|k| !k.schema().is_empty())
          .map(|k| {
              let schema = serde_json::to_value(k.schema()).expect("schemas serialize");
              (k.as_str().to_string(), schema)
          })
          .collect();
      serde_json::to_string(&schemas).expect("schemas serialize")
  }
  ```
- Replace `config_series_names` with
  ```rust
  /// JSON list of the statistics series a config of any model records.
  #[wasm_bindgen]
  pub fn config_series_names(config: &str) -> Result<String, JsValue> {
      let config = ModelConfig::from_json(config).map_err(field_errors)?;
      Ok(serde_json::to_string(&config.series_names()).expect("names serialize"))
  }
  ```
  (`default_config_json` still serializes `Config::default()`.)
- Replace everything from `#[wasm_bindgen]\npub struct Sim {` to the end of the file with (Decision 5):
  ```rust
  /// One world of any model (Decision 5): the sugarscape-only calls (maps,
  /// editing, trails, networks, the credit graph, the disease list and the
  /// wealth views) answer empty (or, for edits, a field error) for the other
  /// models, and `ring_sugar`/`ring_agents` answer empty for every model but
  /// Ring World.
  #[wasm_bindgen]
  pub struct Sim {
      world: ModelWorld,
      frame: Vec<u8>,
  }

  const NO_CREDIT_GRAPH: &str = r#"{"agents":[],"loans":[]}"#;

  impl Sim {
      fn model(&self) -> &dyn Model {
          self.world.model()
      }

      fn sugar(&self) -> Option<&World> {
          self.world.sugarscape()
      }

      /// The sugarscape world, for an edit; a field error for other models.
      fn sugar_mut(&mut self) -> Result<&mut World, JsValue> {
          self.world
              .sugarscape_mut()
              .ok_or_else(|| edit_error("this world is not a sugarscape".into()))
      }

      fn good(&self, good: u32) -> Result<usize, JsValue> {
          let g = good as usize;
          match self.sugar() {
              Some(w) if g < w.config.goods.len() => Ok(g),
              _ => Err(edit_error(format!("there is no good {good}"))),
          }
      }

      /// `f` of the sugarscape world, or `empty` for other models.
      fn sugar_or<T>(&self, empty: T, f: impl FnOnce(&World) -> T) -> T {
          self.sugar().map_or(empty, f)
      }
  }

  #[wasm_bindgen]
  impl Sim {
      /// A world of the config's model (a sugarscape config without a `model`
      /// key, in either shape). `landscapes` are the sugarscape's painted maps;
      /// other models ignore them.
      #[wasm_bindgen(constructor)]
      pub fn new(config_json: &str, seed: u32, landscapes: JsValue) -> Result<Sim, JsValue> {
          let config = ModelConfig::from_json(config_json).map_err(field_errors)?;
          let landscapes = landscapes_from_js(&landscapes)?;
          let world = ModelWorld::with_landscapes(config, u64::from(seed), &landscapes)
              .map_err(field_errors)?;
          Ok(Sim {
              world,
              frame: Vec::new(),
          })
      }

      /// `"sugarscape"`, `"schelling"` or `"ring"`.
      pub fn model_kind(&self) -> String {
          self.world.kind().as_str().to_string()
      }

      pub fn step(&mut self, n: u32) {
          self.world.model_mut().run(n);
      }

      pub fn tick(&self) -> f64 {
          self.model().tick() as f64
      }

      /// The frame's width in cells (Ring World: its sites).
      pub fn width(&self) -> u32 {
          self.model().size().0
      }

      /// The frame's height in cells (Ring World: the space–time diagram's rows).
      pub fn height(&self) -> u32 {
          self.model().size().1
      }

      pub fn population(&self) -> u32 {
          self.model().population() as u32
      }

      /// Renders into the internal frame and returns a pointer into WASM memory.
      /// Re-create any JS view after each call: memory may have grown.
      pub fn render(&mut self, color_mode: &str, layer: &str) -> Result<usize, JsValue> {
          let Sim { world, frame } = self;
          world
              .model()
              .render(color_mode, layer, frame)
              .map_err(edit_error)?;
          Ok(frame.as_ptr() as usize)
      }

      pub fn frame_len(&self) -> usize {
          self.frame.len()
      }

      pub fn stats_latest(&self) -> String {
          self.model().latest_json()
      }

      pub fn series(&self, name: &str) -> Result<Vec<f64>, JsValue> {
          self.model()
              .series(name)
              .ok_or_else(|| edit_error(format!("unknown series {name:?}")))
      }

      /// Series `name` cut to at most `max` points (`stats::downsample`) as
      /// `[tick, value, tick, value, …]`.
      pub fn series_downsampled(&self, name: &str, max: u32) -> Result<Vec<f64>, JsValue> {
          let values = self.series(name)?;
          let ticks = self.model().series("tick").unwrap_or_default();
          Ok(stats::downsample(&values, max as usize)
              .into_iter()
              .flat_map(|(i, v)| [ticks[i as usize], v])
              .collect())
      }

      /// Several series on one x axis (`stats::downsample_union`: each keeps
      /// at most `max` points of its own shape) as `[n, ticks (n), then n
      /// values per name]`. `names_json` is a JSON array of series names.
      pub fn series_group(&self, names_json: &str, max: u32) -> Result<Vec<f64>, JsValue> {
          let names: Vec<String> =
              serde_json::from_str(names_json).map_err(|e| edit_error(e.to_string()))?;
          let columns = names
              .iter()
              .map(|name| self.series(name))
              .collect::<Result<Vec<_>, _>>()?;
          let ticks = self.model().series("tick").unwrap_or_default();
          let keep = stats::downsample_union(&columns, max as usize);
          let mut out = Vec::with_capacity(1 + keep.len() * (1 + columns.len()));
          out.push(keep.len() as f64);
          out.extend(keep.iter().map(|&i| ticks[i]));
          for column in &columns {
              out.extend(keep.iter().map(|&i| column[i]));
          }
          Ok(out)
      }

      /// The model's fingerprint as `0x…` hex, the golden tests' format (see [`fingerprint_hex`]).
      pub fn fingerprint(&self) -> String {
          fingerprint_hex(self.model().fingerprint())
      }

      pub fn lorenz(&self, points: usize) -> Vec<f64> {
          self.sugar_or(Vec::new(), |w| {
              stats::lorenz(&stats::wealths(w), points.max(2))
          })
      }

      /// `[bin_width, count_0, …, count_{bins-1}]`.
      pub fn wealth_hist(&self, bins: usize) -> Vec<f64> {
          self.sugar_or(Vec::new(), |w| {
              let (width, counts) = stats::histogram(&stats::wealths(w), bins.max(1));
              std::iter::once(width).chain(counts).collect()
          })
      }

      /// `[bin_width, count_0, …, count_{bins-1}]` of good `good`'s holdings (the wealth
      /// histogram's bins, for any good).
      pub fn good_wealth_hist(&self, good: u32, bins: usize) -> Result<Vec<f64>, JsValue> {
          let g = self.good(good)?;
          let w = self.sugar().expect("goods exist only in a sugarscape");
          let (width, counts) = stats::histogram(&stats::good_wealths(w, g), bins.max(1));
          Ok(std::iter::once(width).chain(counts).collect())
      }

      /// The Lorenz curve of total wealth (every good's holdings summed).
      pub fn lorenz_total(&self, points: usize) -> Vec<f64> {
          self.sugar_or(Vec::new(), |w| {
              stats::lorenz(&stats::total_wealths(w), points.max(2))
          })
      }

      /// `[bin, count_0, …]`: living agents' ages in `bin`-tick bins (`stats::age_histogram`).
      pub fn age_hist(&self, bin: u32) -> Vec<f64> {
          let bin = bin.max(1);
          self.sugar_or(Vec::new(), |w| {
              std::iter::once(f64::from(bin))
                  .chain(stats::age_histogram(w, bin))
                  .collect()
          })
      }

      /// The percentage of living agents with a 0 at each tag position, position 0 first
      /// (`stats::tag_histogram`).
      pub fn tag_hist(&self) -> Vec<f64> {
          self.sugar_or(Vec::new(), stats::tag_histogram)
      }

      /// The site (x, y) and its agent, in the model's own JSON shape (Ring
      /// World: site `x`, whatever `y`).
      pub fn inspect(&self, x: u32, y: u32) -> Result<String, JsValue> {
          self.model().inspect_json(x, y).map_err(edit_error)
      }

      pub fn locate(&self, id: f64) -> Option<Vec<u32>> {
          self.model().locate(id as u64).map(|(x, y)| vec![x, y])
      }

      /// Records agent `id`'s trail from now on (`World::follow`; sugarscape only).
      pub fn follow(&mut self, id: f64) {
          if let Some(w) = self.world.sugarscape_mut() {
              w.follow(Some(id as u64));
          }
      }

      pub fn unfollow(&mut self) {
          if let Some(w) = self.world.sugarscape_mut() {
              w.follow(None);
          }
      }

      /// The followed agent's trail as `[x0, y0, x1, y1, …]`, oldest first.
      pub fn trail(&self) -> Vec<u32> {
          self.sugar_or(Vec::new(), |w| {
              w.trail().iter().flat_map(|p| [p.x, p.y]).collect()
          })
      }

      /// The followed agent's id, or −1 when none is followed.
      pub fn followed(&self) -> f64 {
          self.sugar()
              .and_then(World::followed)
              .map_or(-1.0, |id| id as f64)
      }

      pub fn paint_capacity(
          &mut self,
          x: u32,
          y: u32,
          radius: u32,
          value: f64,
          good: u32,
      ) -> Result<(), JsValue> {
          self.sugar_mut()?
              .paint_capacity(x, y, radius, value, good as usize)
              .map_err(edit_error)
      }

      /// Replaces good `good`'s capacities with `capacities` (row-major bytes,
      /// each 0–10): an imported image. The good then counts as painted.
      pub fn set_landscape(&mut self, good: u32, capacities: &[u8]) -> Result<(), JsValue> {
          let g = self.good(good)?;
          let caps: Vec<f64> = capacities.iter().map(|&c| f64::from(c)).collect();
          self.sugar_mut()?
              .set_capacities(g, &caps)
              .map_err(edit_error)
      }

      pub fn place_agent(&mut self, x: u32, y: u32, overrides_json: &str) -> Result<f64, JsValue> {
          let overrides: AgentOverrides =
              serde_json::from_str(overrides_json).map_err(|e| edit_error(e.to_string()))?;
          self.sugar_mut()?
              .place_agent(x, y, &overrides)
              .map(|id| id as f64)
              .map_err(edit_error)
      }

      pub fn remove_agent(&mut self, x: u32, y: u32) -> Result<(), JsValue> {
          self.sugar_mut()?.remove_agent(x, y).map_err(edit_error)
      }

      /// Applies a changed config of the world's model to the running world.
      pub fn set_config(&mut self, json: &str) -> Result<(), JsValue> {
          let config = ModelConfig::from_json(json).map_err(field_errors)?;
          self.world
              .model_mut()
              .set_config(config)
              .map_err(field_errors)
      }

      /// The live config, tagged with its model unless it is a sugarscape's.
      pub fn export_config(&self) -> String {
          serde_json::to_string(&self.model().config()).expect("config serializes")
      }

      /// Good `good`'s capacities rounded to bytes, row-major.
      pub fn export_landscape(&self, good: u32) -> Result<Vec<u8>, JsValue> {
          let g = self.good(good)?;
          let w = self.sugar().expect("goods exist only in a sugarscape");
          Ok(w.capacities(g)
              .into_iter()
              .map(|c| c.round().clamp(0.0, 255.0) as u8)
              .collect())
      }

      /// Whether good `good`'s capacities differ from its generated map.
      pub fn landscape_edited(&self, good: u32) -> bool {
          self.sugar_or(false, |w| w.landscape_edited(good as usize))
      }

      /// JSON list of this world's series names (a sugarscape's depend on its
      /// goods, pollutants and groups).
      pub fn series_names(&self) -> String {
          serde_json::to_string(&self.model().series_names()).expect("names serialize")
      }

      pub fn export_series_csv(&self) -> String {
          self.model().series_csv()
      }

      pub fn export_agents_csv(&self) -> String {
          self.model().agents_csv()
      }

      /// Edges as `[x1, y1, x2, y2, …]` for `"trade"` (this tick), `"credit"` (outstanding),
      /// `"disease"` (infector → infected, this tick), `"neighbors"` (agent → each agent on its
      /// neighbor list), `"friends"` (agent → friend) or `"family"` (parent → child); none in
      /// other models.
      pub fn networks(&self, kind: &str) -> Result<Vec<u32>, JsValue> {
          let Some(w) = self.sugar() else {
              return Ok(Vec::new());
          };
          let edges = match kind {
              "trade" => network::trade_edges(w),
              "credit" => network::credit_edges(w),
              "disease" => network::disease_edges(w),
              "neighbors" => w.neighbor_edges(),
              "friends" => w.friend_edges(),
              "family" => w.family_edges(),
              _ => return Err(edit_error(format!("unknown network {kind:?}"))),
          };
          Ok(edges
              .into_iter()
              .flat_map(|(a, b)| [a.x, a.y, b.x, b.y])
              .collect())
      }

      /// JSON `{ agents: [{ id, role }], loans: [{ lender, borrower, good, due }] }`
      /// over the outstanding loans.
      pub fn credit_graph(&self) -> String {
          self.sugar_or(NO_CREDIT_GRAPH.into(), |w| {
              serde_json::to_string(&network::credit_graph(w)).expect("graph serializes")
          })
      }

      /// JSON `[{ id, bits, carriers }]`.
      pub fn disease_list(&self) -> String {
          self.sugar_or("[]".into(), |w| {
              serde_json::to_string(&w.disease_list()).expect("list serializes")
          })
      }

      /// Infects the agent at (x, y) with `disease` (−1 = a brand-new random
      /// disease). Returns whether it was infected.
      pub fn infect(&mut self, x: u32, y: u32, disease: i32) -> Result<bool, JsValue> {
          self.sugar_mut()?
              .infect(x, y, i64::from(disease))
              .map_err(edit_error)
      }

      /// Vaccinates every agent within `radius` of (x, y) against `disease`.
      /// Returns how many agents were vaccinated.
      pub fn vaccinate(&mut self, x: u32, y: u32, radius: u32, disease: u32) -> Result<u32, JsValue> {
          self.sugar_mut()?
              .vaccinate(x, y, radius, disease)
              .map_err(edit_error)
      }

      /// `[n, prices(n), demand(n), supply(n), eq_price, eq_quantity, actual_price, actual_quantity]`.
      pub fn supply_demand(&self) -> Vec<f64> {
          self.sugar_or(Vec::new(), |w| {
              let sd = stats::supply_demand(w);
              let mut out = vec![sd.prices.len() as f64];
              out.extend(sd.prices);
              out.extend(sd.demand);
              out.extend(sd.supply);
              out.extend([
                  sd.equilibrium_price,
                  sd.equilibrium_quantity,
                  sd.actual_price,
                  sd.actual_quantity,
              ]);
              out
          })
      }

      /// Ring World's sugar per site, site 0 first (empty for other models).
      pub fn ring_sugar(&self) -> Vec<f64> {
          self.world.ring().map_or(Vec::new(), |r| r.sugar().to_vec())
      }

      /// Ring World's agents' sites, in id order (empty for other models).
      pub fn ring_agents(&self) -> Vec<u32> {
          self.world.ring().map_or(Vec::new(), |r| r.agent_sites())
      }
  }
  ```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS, including `a_schelling_sim_runs_renders_and_inspects` (its fingerprint `0x7a7072c3433f5f6f` is golden.rs's `vi-4-schelling-25` entry: the WASM build agrees with the native one, Decision 9) and `a_ring_sim_draws_its_space_time_diagram_and_reports_its_ring` (`0x1c341361c466db90`). A different Schelling fingerprint here means a `usize` range is being sampled somewhere — fix that, never the expected value.
Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-wasm/src/lib.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Run any model in the WASM Sim; expose schemas and Ring World's state" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 5: `ModelConfig` on the page — types, protocol, host, engine and links

*Needs judgement (full code given; the point is that every sugarscape path is unchanged — Decisions 6 and 7 — and that the sugarscape panels only ever read `Engine.sugar`).* Browser (controller): regression only — every existing scenario (presets, Reset, 🎲, rule edits, painting, overlays, share links incl. the legacy link, session files, Compare, Experiments, recording) works as before; the presets menu lists the six new presets (ungrouped until Task 6) — do not choose them yet.

**Files:**
- Create: `web/src/models.ts`, `web/src/models.test.ts`
- Modify: `web/src/types.ts`, `web/src/protocol.ts`, `web/src/layers.ts`, `web/src/sim-host.ts`, `web/src/fake-sim.fixture.ts`, `web/src/engine.ts`, `web/src/share.ts`, `web/src/sessions.ts`, `web/src/experiments/types.ts`, `web/src/experiments/form.ts`, `web/src/experiments/view.ts`, `web/src/main.ts`, `web/src/compare/compare-view.ts`, `web/src/ui/rules-panel.ts`, `web/src/ui/charts-panel.ts`, `web/src/ui/inspect-panel.ts`, `web/src/ui/display.ts`, `web/src/ui/tools.ts`
- Test: `web/src/layers.test.ts`, `web/src/sim-host.test.ts`, `web/src/engine.test.ts`, `web/src/share.test.ts`, `web/src/sessions.test.ts`, `web/src/compare-presets.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: WASM `presets_json` (every model), `model_schemas_json`, `Sim.{ring_sugar, ring_agents}`, `Sim.export_config()` (tagged for the other models), `Sim.stats_latest()` / `inspect()` in each model's shape (Task 4).
- Produces:
  - `types.ts`: `ModelKind`, `FRange`, `SchellingConfig`, `RingConfig`, `ModelConfig`, `Param`, `SchellingStats`, `RingStats`, `ModelStats`, `SchellingAgentView`, `SchellingInspection`, `RingInspection`, `AnyInspection`; `Preset.config: ModelConfig`; `ColorMode` gains `'color' | 'satisfaction' | 'preference'`.
  - `models.ts`: `MODELS`, `MODEL_LABELS`, `modelOf(c)`, `isSugar(c): c is Config`, `isSugarView(v): v is Inspection`, `isRingView(v): v is RingInspection`, `presetModel(p)`, `presetGroups(presets)`, `COLOR_MODES: Record<ModelKind, [ColorMode, string][]>`.
  - `protocol.ts`: `Wants.ring?`, `RingState { sugar: Float64Array; agents: Uint32Array }`, `WorldSnapshot.ring?`, `WorldSnapshot.latest: ModelStats`, `WorldSnapshot.config?: ModelConfig`, `Selected.view: AnyInspection`, `ModelConfig` in `init`/`reset`/`setConfig` and `Session`.
  - `layers.ts`: `clampDisplay(d, config: ModelConfig)`.
  - `sim-host.ts`: `SimLike.ring_sugar(): Float64Array`, `SimLike.ring_agents(): Uint32Array`.
  - `engine.ts`: `InitialState.config: ModelConfig`; `EngineDeps.schemas?: Partial<Record<ModelKind, Param[]>>`; `Engine.{config, baseConfig}: ModelConfig`, `Engine.latest: ModelStats | null`, `Engine.ring: RingState | null`, `readonly Engine.schemas`, getters `Engine.model: ModelKind` and `Engine.sugar: Config`, `applyModelConfig(mutate: (c: ModelConfig) => void)`, `resetModelWith(mutate: (c: ModelConfig) => void, seed?)`; `reset(config?: ModelConfig, seed?)`.
  - `share.ts`: `ShareState.config: ModelConfig`; `sessions.ts`: `SessionSource.baseConfig: ModelConfig`; `experiments/types.ts`: `SweepBase = { preset: string } | { config: ModelConfig }`; `numericPaths(config: ModelConfig)`.

- [ ] **Step 1: Write the failing tests**

Create `web/src/models.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { COLOR_MODES, isRingView, isSugar, isSugarView, modelOf, presetGroups } from './models';
import type { AnyInspection, Config, ModelConfig, Preset } from './types';

describe('modelOf', () => {
  it('reads a config without a model key (every config before milestone 9) as a sugarscape', () => {
    const old = { width: 50, goods: [] } as unknown as Config;
    expect(modelOf(old)).toBe('sugarscape');
    expect(isSugar(old)).toBe(true);
    expect(modelOf({ ...old, model: 'sugarscape' } as unknown as ModelConfig)).toBe('sugarscape');
  });

  it('reads the other models by their tag', () => {
    expect(modelOf({ model: 'schelling' } as ModelConfig)).toBe('schelling');
    expect(modelOf({ model: 'ring' } as ModelConfig)).toBe('ring');
    expect(isSugar({ model: 'ring' } as ModelConfig)).toBe(false);
  });

  it('tells a sugarscape inspection by its resources and Ring World’s by its sugar', () => {
    const sugar = { site: { x: 0, y: 0, resources: [1], capacities: [4], pollution: [] }, agent: null } as AnyInspection;
    const ring = { site: { x: 3, sugar: 2, capacity: 4 }, agent: null } as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect(isSugarView(sugar)).toBe(true);
    expect(isSugarView(ring)).toBe(false);
    expect([sugar, ring, schelling].map(isRingView)).toEqual([false, true, false]);
  });

  it('offers each model its color modes, Schelling first Colour, Ring World none', () => {
    expect(COLOR_MODES.sugarscape[0][0]).toBe('tribe');
    expect(COLOR_MODES.schelling.map(([m]) => m)).toEqual(['color', 'satisfaction', 'preference']);
    expect(COLOR_MODES.ring).toEqual([]);
  });
});

describe('presetGroups', () => {
  it('groups the presets menu by model in a fixed order, leaving out models without presets', () => {
    const p = (id: string, config: unknown): Preset => ({ id, name: id, source: '', description: '', config: config as ModelConfig });
    const presets = [p('ring-1', { model: 'ring' }), p('ii-2', {}), p('ii-3', {}), p('ring-2', { model: 'ring' })];
    expect(presetGroups(presets).map((g) => [g.label, g.presets.map((x) => x.id)])).toEqual([
      ['Sugarscape', ['ii-2', 'ii-3']],
      ['Ring World', ['ring-1', 'ring-2']],
    ]);
  });
});
```
`web/src/layers.test.ts`: `import type { Config } from './types';` becomes `import type { Config, RingConfig, SchellingConfig } from './types';` and, before `it('keeps the Lineage color mode in any world'`, add
```ts
  it('clamps to the model: Schelling offers its own modes and no overlays; a sugarscape falls back to Tribe', () => {
    const schelling = { model: 'schelling' } as SchellingConfig;
    const clamped = clampDisplay(display, schelling);
    expect(clamped).toEqual({ colorMode: 'color', layer: 'pollution:0', overlays: noOverlays() });
    const satisfaction: DisplayState = { ...clamped, colorMode: 'satisfaction' };
    expect(clampDisplay(satisfaction, schelling)).toBe(satisfaction);
    expect(clampDisplay(satisfaction, rules(true)).colorMode).toBe('tribe');
    // Ring World draws no color modes: the mode is kept for the next sugarscape (and clamped there).
    const ring = { model: 'ring' } as RingConfig;
    expect(clampDisplay(display, ring)).toEqual({ ...display, overlays: noOverlays() });
  });

```
`web/src/sim-host.test.ts`: `import type { Config } from './types';` becomes `import type { Config, ModelConfig } from './types';`; in `'answers init with the world, …'` `expect(s.config?.goods).toHaveLength(1);` becomes `expect((s.config as Config).goods).toHaveLength(1);`; in `'reports a scheduled change as a config change, …'` `.config?.schedule).toHaveLength(1); // the step from 5 started` becomes `expect((t.snap(t.send({ type: 'step', n: 2 })).config as Config).schedule).toHaveLength(1); // the step from 5 started`; in `'reports a scheduled change in the next post'` `expect(post.config?.schedule).toHaveLength(1);` becomes `expect((post.config as Config).schedule).toHaveLength(1);`; and before `describe('SimHost at Max speed'` add
```ts
describe('SimHost with another model', () => {
  /** A host whose world is a fake of `model`. */
  function other(model: 'schelling' | 'ring') {
    const host = new SimHost(fakeModule());
    const config = { model, width: 6, height: 4 } as unknown as ModelConfig;
    let id = 0;
    const send = (cmd: Command, wants?: Wants): WorldSnapshot => {
      const reply = host.handle({ id: ++id, cmd, wants });
      if (!reply.result.ok || !reply.result.snapshot) throw new Error(JSON.stringify(reply.result));
      return reply.result.snapshot;
    };
    const init = send({ type: 'init', config, seed: 1, landscapes: [], display });
    return { init, send };
  }

  it('clamps the display to the model and sends no maps', () => {
    const { init } = other('schelling');
    expect(init.display).toEqual({ colorMode: 'color', layer: 'resource:0', overlays: noOverlays() });
    expect(init.editedLandscapes).toEqual([]);
  });

  it('answers only the wishes the model can: charts and the ring, never sugarscape extras', () => {
    const { send } = other('ring');
    const s = send({ type: 'step', n: 2 }, {
      ring: true,
      trail: true,
      networks: ['trade'],
      lorenz: true,
      diseaseList: true,
      charts: { groups: [['population']], max: 10 },
    });
    expect(s.ring?.sugar).toHaveLength(6);
    expect(Array.from(s.ring!.agents)).toEqual([3]);
    expect(s.charts).toBeDefined();
    for (const key of ['trail', 'networks', 'lorenz', 'diseaseList'] as const) expect(s[key]).toBeUndefined();
  });

  it('sends the ring only for Ring World', () => {
    const { send } = other('schelling');
    expect(send({ type: 'refresh' }, { ring: true }).ring).toBeUndefined();
  });
});

```
`web/src/engine.test.ts`: the types import becomes `import type { Config, ModelConfig, Preset, RingConfig } from './types';` with `import { isSugar } from './models';` above it; after `const settle = …;` add
```ts
/** A config the test knows is a sugarscape's. */
const sugar = (c: ModelConfig): Config => {
  if (!isSugar(c)) throw new Error('not a sugarscape config');
  return c;
};
```
then wrap every read of a sugarscape field through the engine's configs: each `engine.config.<width|height|population>` becomes `sugar(engine.config).<…>` and each `engine.baseConfig.<population|height>` becomes `sugar(engine.baseConfig).<…>` (in `'starts from the init snapshot'`, `"resolves writes with the core's errors"`, `'builds each queued config write on the one before it'`, `'builds a reset-requiring change on a config write still in flight'` and `'Reset (replay) rebuilds the session and keeps the setup; …'`); in the clamp test replace `expect(await engine.reset({ ...engine.baseConfig, disease: { ...engine.baseConfig.disease, enabled: false } })).toBeNull();` with
```ts
    const base = sugar(engine.baseConfig);
    expect(await engine.reset({ ...base, disease: { ...base.disease, enabled: false } })).toBeNull();
```
in `'Chapter VI views while paused'` `distributionWants(engine.config)` becomes `distributionWants(engine.sugar)`; and append
```ts
describe('Engine with other models', () => {
  const schelling = { model: 'schelling', width: 6, height: 4 } as unknown as ModelConfig;
  const ring = { model: 'ring', width: 5, height: 3 } as unknown as ModelConfig;
  const models: Preset[] = [
    ...presets,
    { id: 'vi-4', name: 'Schelling', source: 'VI-4', description: '', config: schelling },
    { id: 'vi-8', name: 'Ring', source: 'VI-8', description: '', config: ring },
  ];
  const make = (config: ModelConfig, transport = new HookedTransport(new SimHost(fakeModule()))) =>
    Engine.create({ config, seed: 1 }, { presets: models, transport });

  it('knows its model and keeps the last sugarscape config for the sugarscape panels', async () => {
    const e = await make(config);
    expect(e.model).toBe('sugarscape');
    expect(e.sugar).toBe(e.config);
    const before = e.sugar;
    expect(await e.loadPreset('vi-4')).toBeNull();
    expect(e.presetId).toBe('vi-4');
    expect(e.model).toBe('schelling');
    expect(e.sugar).toBe(before);
  });

  it('refuses sugarscape rule edits in another model and applies its own', async () => {
    const e = await make(ring);
    const refused = [{ field: 'config', message: 'this world is a ring world, not a sugarscape' }];
    expect(await e.applyConfig((c) => void (c.population = 5))).toEqual(refused);
    expect(await e.resetWith((c) => void (c.population = 5))).toEqual(refused);
    expect(await e.applyModelConfig((c) => void ((c as RingConfig).growback = 2))).toBeNull();
    expect((e.config as RingConfig).growback).toBe(2);
    expect((e.baseConfig as RingConfig).growback).toBe(2);
    expect(await e.resetModelWith((c) => void ((c as RingConfig).sites = 7))).toBeNull();
    expect(e.size().width).toBe(5);
  });

  it('asks for the ring with every request in Ring World and never for overlays; other models drop it', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const sent: Wants[] = [];
    transport.after = (_cmd, wants) => sent.push(wants ?? {});
    const e = await make(ring, transport);
    e.setDisplay({ overlays: { trade: true } });
    await e.advance(1);
    expect(sent.at(-1)).toMatchObject({ ring: true });
    expect(sent.at(-1)?.networks).toBeUndefined();
    expect(e.ring?.agents).toHaveLength(1);
    expect(await e.loadPreset('ii-2-unit')).toBeNull();
    expect(e.ring).toBeNull();
  });

  it('drops painted maps when a reset changes the model', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const resets: Command[] = [];
    transport.after = (cmd) => void (cmd.type === 'reset' && resets.push(cmd));
    const e = await make(config, transport);
    await e.paint(0, 0, 1, 3);
    expect(e.editedLandscapes()).toBeDefined();
    expect(await e.reset(ring)).toBeNull();
    expect(resets.at(-1)).toMatchObject({ type: 'reset', landscapes: [] });
  });
});
```
`web/src/share.test.ts`: `import type { Config } from './types';` becomes `import { modelOf } from './models';` + `import type { Config, SchellingConfig } from './types';`, and append
```ts
describe('models in links', () => {
  it('opens a link made before milestone 9 as a sugarscape', async () => {
    const opened = await decodeShare(LEGACY_SHARE_TOKEN);
    expect(modelOf(opened.config)).toBe('sugarscape');
  });

  it('carries another model’s tagged config through a link, a compare link and a session file', async () => {
    const schelling = { model: 'schelling', width: 50, height: 50, population: 2000 } as SchellingConfig;
    const back = await decodeShare(await encodeShare({ config: schelling, seed: 3 }));
    expect(back.config).toEqual(schelling);
    expect(modelOf(back.config)).toBe('schelling');
    const pair = await decodeCompare(await encodeCompare({ a: { config: schelling, seed: 1 }, b: { config: schelling, seed: 2 } }));
    expect(modelOf(pair.b.config)).toBe('schelling');
    const file = parseSessionFile(sessionFileText({ kind: 'session', state: { config: schelling, seed: 4 } }));
    expect(file.kind === 'session' && modelOf(file.state.config)).toBe('schelling');
  });
});
```
`web/src/sessions.test.ts`: `expect(back.config.population).toBe(99);` becomes `expect((back.config as Config).population).toBe(99);`.
`web/src/compare-presets.test.ts`: replace
```ts
    expect(states.b.config.trade.enabled).toBe(true);
    states.b.config.trade.enabled = false;
    expect(presets[1].config.trade.enabled).toBe(true);
```
with
```ts
    const b = states.b.config as Config;
    expect(b.trade.enabled).toBe(true);
    b.trade.enabled = false;
    expect((presets[1].config as Config).trade.enabled).toBe(true);
```
`web/src/determinism.test.ts`: `import type { Preset } from './types';` becomes `import type { Preset, Snapshot } from './types';`; `expect(watched.latest!.gini_total)` becomes `expect((watched.latest as Snapshot).gini_total)`; append (the fingerprints are golden.rs's `MODEL_GOLDEN` entries, in `fingerprint_hex`'s 16-digit form):
```ts
describe('other models through the engine', () => {
  /** crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: each preset after 200 ticks from seed 1. */
  const GOLDEN_MODELS: [string, string][] = [
    ['vi-4-schelling-25', '0x7a7072c3433f5f6f'],
    ['vi-8-ring-world', '0x1c341361c466db90'],
  ];

  it.each(GOLDEN_MODELS)('%s reproduces its golden fingerprint, whatever is watched', async (id, golden) => {
    const preset = presets.find((p) => p.id === id)!;
    const e = await Engine.create({ config: structuredClone(preset.config), seed: 1 }, { presets, transport: inline() });
    // Sugarscape-only wishes are ignored; charts, the ring and inspection change nothing.
    e.want(() => ({ charts: { groups: [['population']], max: 50 }, lorenz: true, networks: ['trade'] }));
    await e.select(3, 3);
    for (const n of [1, 9, 40, 150]) await e.advance(n);
    expect(e.tick).toBe(200);
    expect(await e.fingerprint()).toBe(golden);
  });
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `(cd web && npm run build)`
Expected: FAIL in `tsc` — `./models` does not exist; `SchellingConfig`, `RingConfig`, `ModelConfig`, `Snapshot`-less `latest`, `applyModelConfig`, `resetModelWith`, `Engine.sugar`, `Engine.ring`, `ring_sugar` are unknown.

- [ ] **Step 3: Implement the types and `models.ts`**

`web/src/types.ts` — replace `export interface Preset { id: string; name: string; source: string; description: string; config: Config }` with
```ts
/** The models the playground runs (milestone 9). */
export type ModelKind = 'sugarscape' | 'schelling' | 'ring';

/** A fraction range (Schelling's preferences). */
export interface FRange { min: number; max: number }

/** The book's Schelling variant (animations VI-4 to VI-7). */
export interface SchellingConfig {
  model: 'schelling';
  width: number;
  height: number;
  population: number;
  preference: FRange;
  residence: { enabled: boolean; min: number; max: number };
}

/** Ring World (animations VI-8 and VI-9). */
export interface RingConfig {
  model: 'ring';
  sites: number;
  agents: number;
  vision: URange;
  capacity: number;
  growback: number;
  start: 'random' | 'megagroup';
}

/**
 * A config of any model. A sugarscape `Config` carries no `model` key (every config, link, session
 * and sweep written before milestone 9 is one); the others carry theirs. Narrow with `isSugar` /
 * `modelOf` (models.ts).
 */
export type ModelConfig = Config | SchellingConfig | RingConfig;

export interface Preset { id: string; name: string; source: string; description: string; config: ModelConfig }

/** One field of a model's Rules panel (the core's `schema::Param`). */
export interface Param {
  path: string;
  label: string;
  kind: 'integer' | 'number' | 'range' | 'bool' | 'choice';
  min?: number;
  max?: number;
  step?: number;
  choices?: { value: string; label: string }[];
  apply: 'live' | 'reset';
  group: string;
}
```
before `export interface SiteView {` add
```ts
export interface SchellingStats {
  tick: number;
  population: number;
  unsatisfied: number;
  segregation: number;
  moves: number;
  red_share: number;
  quiet: number;
}

export interface RingStats {
  tick: number;
  population: number;
  flocks: number;
  mean_flock: number;
  largest_flock: number;
  mean_distance: number;
}

/** The latest statistics of a world of any model. */
export type ModelStats = Snapshot | SchellingStats | RingStats;

```
and replace the `ColorMode` line with (after `export interface Inspection { … }`)
```ts
export interface SchellingAgentView {
  id: number;
  color: 'red' | 'blue';
  preference: number;
  satisfied: boolean;
  /** Like-colored and all occupied von Neumann neighbors. */
  like: number;
  neighbors: number;
  age: number;
  /** The age at which it leaves; null with residence off. */
  residence: number | null;
}
export interface SchellingInspection { site: { x: number; y: number }; agent: SchellingAgentView | null }
export interface RingInspection { site: { x: number; sugar: number; capacity: number }; agent: { id: number; vision: number } | null }
/** What a world of any model says about a site. */
export type AnyInspection = Inspection | SchellingInspection | RingInspection;

/** A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`. */
export type ColorMode =
  | 'tribe'
  | 'wealth'
  | 'sex'
  | 'age'
  | 'vision'
  | 'credit'
  | 'disease'
  | 'lineage'
  | 'color'
  | 'satisfaction'
  | 'preference';
```
Create `web/src/models.ts` (Decisions 6, 7, 12):
```ts
// Which model a config is (milestone 9), and what each model offers the page.
import type { AnyInspection, ColorMode, Config, Inspection, ModelConfig, ModelKind, Preset, RingInspection } from './types';

export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring'];

/** The presets menu's group labels. */
export const MODEL_LABELS: Record<ModelKind, string> = { sugarscape: 'Sugarscape', schelling: 'Schelling', ring: 'Ring World' };

/** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
export function modelOf(c: ModelConfig): ModelKind {
  const tag = (c as { model?: unknown }).model;
  return tag === 'schelling' || tag === 'ring' ? tag : 'sugarscape';
}

export function isSugar(c: ModelConfig): c is Config {
  return modelOf(c) === 'sugarscape';
}

/** A sugarscape site's inspection (it lists the site's resources). */
export function isSugarView(v: AnyInspection): v is Inspection {
  return 'resources' in v.site;
}

/** A Ring World site's inspection (it has sugar but no resources list). */
export function isRingView(v: AnyInspection): v is RingInspection {
  return 'sugar' in v.site;
}

export function presetModel(p: Preset): ModelKind {
  return modelOf(p.config);
}

/** The presets menu's groups: each model with presets, in `MODELS` order, its presets in list order. */
export function presetGroups(presets: Preset[]): { model: ModelKind; label: string; presets: Preset[] }[] {
  return MODELS.map((model) => ({ model, label: MODEL_LABELS[model], presets: presets.filter((p) => presetModel(p) === model) })).filter(
    (g) => g.presets.length > 0,
  );
}

/** The color modes (value, label) the Agents menu offers for each model, in order; the first is the default. */
export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
  sugarscape: [
    ['tribe', 'Tribe'],
    ['wealth', 'Wealth'],
    ['sex', 'Sex'],
    ['age', 'Age'],
    ['vision', 'Vision'],
    ['credit', 'Credit'],
    ['disease', 'Disease'],
    ['lineage', 'Lineage'],
  ],
  schelling: [
    ['color', 'Colour'],
    ['satisfaction', 'Satisfaction'],
    ['preference', 'Preference'],
  ],
  // The ring view and space–time diagram have no color modes.
  ring: [],
};
```

- [ ] **Step 4: Implement the protocol, display clamping and the host**

`web/src/protocol.ts` (Decision 7):
- The types import becomes `import type { AnyInspection, ColorMode, DiseaseEntry, FieldError, Layer, ModelConfig, ModelStats } from './types';`.
- `Selected`'s `view: Inspection` becomes `view: AnyInspection`.
- In `Wants`, after `diseaseList?: boolean;` add
  ```ts
    /** Ring World's sugar per site and agents' sites (the ring view). */
    ring?: boolean;
  ```
  and after `Wants` add
  ```ts
  /** Ring World's state for the ring view: sugar per site (site 0 first) and each agent's site. */
  export interface RingState { sugar: Float64Array; agents: Uint32Array }
  ```
- In `WorldSnapshot`: `latest: Snapshot;` → `latest: ModelStats;`; `config?: Config;` → `config?: ModelConfig;`; after `diseaseList?: DiseaseEntry[];` add `ring?: RingState;`.
- In `Command`, the `config: Config` of `init`, `reset` and `setConfig` becomes `config: ModelConfig`; `Session`'s `config: Config` becomes `config: ModelConfig`.
- In `FLAGS`, after `'diseaseList',` add `'ring',`.

`web/src/layers.ts` — the imports become
```ts
import { COLOR_MODES, isSugar, modelOf } from './models';
import { noOverlays, OVERLAYS, type DisplayState, type Overlay } from './protocol';
import type { Config, Layer, ModelConfig } from './types';
```
and `clampDisplay` (doc included) becomes
```ts
/**
 * `d` kept valid for `config` (the host applies it to every snapshot). In a sugarscape: a layer the
 * world lacks falls back to good 0's level; another model's color mode, or Disease with disease off,
 * falls back to Tribe; an overlay the world cannot show (`overlayAvailable`) is turned off. In
 * another model: a color mode it lacks falls back to its first, every overlay is off and the layer
 * is kept (unused). Returns `d` itself when nothing changes.
 */
export function clampDisplay(d: DisplayState, config: ModelConfig): DisplayState {
  if (!isSugar(config)) {
    const modes = COLOR_MODES[modelOf(config)].map(([m]) => m);
    const colorMode = modes.length === 0 || modes.includes(d.colorMode) ? d.colorMode : modes[0];
    if (colorMode === d.colorMode && !OVERLAYS.some((k) => d.overlays[k])) return d;
    return { colorMode, layer: d.layer, overlays: noOverlays() };
  }
  const layer = validLayer(d.layer, config);
  const sugarMode = COLOR_MODES.sugarscape.some(([m]) => m === d.colorMode);
  const colorMode = !sugarMode || (!config.disease.enabled && d.colorMode === 'disease') ? 'tribe' : d.colorMode;
  const off = OVERLAYS.filter((k) => d.overlays[k] && !overlayAvailable(k, config));
  if (layer === d.layer && colorMode === d.colorMode && off.length === 0) return d;
  const overlays = { ...d.overlays };
  for (const k of off) overlays[k] = false;
  return { colorMode, layer, overlays };
}
```

`web/src/sim-host.ts`:
- Imports: after `import { clampDisplay } from './layers';` add `import { isSugar, modelOf } from './models';`; the types import becomes `import { parseErrors, type AnyInspection, type DiseaseEntry, type ModelConfig, type ModelStats } from './types';`.
- `SimLike`: after `disease_list(): string;` add `ring_sugar(): Float64Array;` and `ring_agents(): Uint32Array;`.
- `private config: Config | null = null;` → `private config: ModelConfig | null = null;`.
- `fired` becomes
  ```ts
    private fired(from: number, to: number): void {
      const config = this.config;
      if (config && isSugar(config) && config.schedule.some((c) => c.tick >= from && c.tick < to)) this.configDue = true;
    }
  ```
- In `snapshot`: `latest: JSON.parse(sim.stats_latest()) as Snapshot,` → `as ModelStats,`; `config = JSON.parse(sim.export_config()) as Config;` → `as ModelConfig;`; replace
  ```ts
      if (this.landscapesDue) {
        s.editedLandscapes = config.goods.map((_, i) => (sim.landscape_edited(i) ? sim.export_landscape(i) : null));
        this.landscapesDue = false;
      }
  ```
  with
  ```ts
      const sugar = isSugar(config) ? config : null;
      if (this.landscapesDue) {
        // Only a sugarscape has maps.
        s.editedLandscapes = sugar ? sugar.goods.map((_, i) => (sim.landscape_edited(i) ? sim.export_landscape(i) : null)) : [];
        this.landscapesDue = false;
      }
  ```
  and replace everything from `if (wants.trail) s.trail = sim.trail();` to `return s;` with (Decision 7)
  ```ts
      if (wants.charts) {
        const charts = this.charts(sim, wants.charts.groups, wants.charts.max, throttleCharts);
        if (charts) s.charts = charts;
      }
      if (wants.ring && modelOf(config) === 'ring') s.ring = { sugar: sim.ring_sugar(), agents: sim.ring_agents() };
      // The rest exist only in a sugarscape (Decision 7): a wish for them in another model is ignored.
      if (!sugar) return s;
      if (wants.trail) s.trail = sim.trail();
      if (wants.networks) {
        const networks: Partial<Record<Overlay, Uint32Array>> = {};
        for (const kind of wants.networks) networks[kind] = sim.networks(kind);
        s.networks = networks;
      }
      if (wants.lorenz) s.lorenz = sim.lorenz(101);
      if (wants.wealthHist) s.wealthHist = sim.wealth_hist(20);
      if (wants.ageHist) s.ageHist = sim.age_hist(AGE_BIN);
      if (wants.tagHist) s.tagHist = sim.tag_hist();
      if (wants.lorenzTotal) s.lorenzTotal = sim.lorenz_total(101);
      if (wants.goodWealthHists) s.goodWealthHists = sugar.goods.map((_, i) => sim.good_wealth_hist(i, 20));
      if (wants.supplyDemand) s.supplyDemand = sim.supply_demand();
      if (wants.creditGraph) s.creditGraph = JSON.parse(sim.credit_graph()) as CreditGraph;
      if (wants.diseaseList) s.diseaseList = JSON.parse(sim.disease_list()) as DiseaseEntry[];
      return s;
  ```
- In `selectAt` and `track`: `as Inspection` → `as AnyInspection` (twice).

`web/src/fake-sim.fixture.ts`: in `FakeConfig`, before `width: number;` add
```ts
  /** Absent for a sugarscape; `'ring'` also answers `ring_sugar`/`ring_agents`. */
  model?: 'schelling' | 'ring';
```
and after `disease_list()` add
```ts
  ring_sugar(): Float64Array {
    return this.config.model === 'ring' ? new Float64Array(this.config.width).fill(2) : new Float64Array(0);
  }
  ring_agents(): Uint32Array {
    return this.config.model === 'ring' ? Uint32Array.from(this.agents.values(), (p) => p[0]) : new Uint32Array(0);
  }
```
(`normalize` spreads the given config last, so `model` survives.)

- [ ] **Step 5: Implement the engine (Decision 6)**

`web/src/engine.ts`:
- In the protocol import add `type RingState,` after `type Result,`; after it add `import { isSugar, modelOf } from './models';`; the types import becomes `import type { ColorMode, Config, FieldError, Layer, ModelConfig, ModelKind, ModelStats, Param, Preset } from './types';` and the WASM import `import init, { model_schemas_json, presets_json } from './wasm-pkg/sugarscape.js';`.
- Replace `InitialState` and `EngineDeps` with
  ```ts
  export interface InitialState { config: ModelConfig; seed: number; landscapes?: (Uint8Array | null)[]; log?: LogEntry[] }

  /**
   * What an engine runs on: the presets (every model's), each non-sugarscape model's Rules-panel
   * schema, and a transport to a SimHost (tests pass fakes).
   */
  export interface EngineDeps { presets: Preset[]; schemas?: Partial<Record<ModelKind, Param[]>>; transport: Transport }
  ```
- Before `export function randomSeed()` add
  ```ts
  /** A sugarscape Rules-panel edit, refused (thrown) on another model's config. */
  function sugarOnly(mutate: (c: Config) => void): (c: ModelConfig) => void {
    return (c) => {
      if (!isSugar(c)) throw new Error(`this world is a ${modelOf(c)} world, not a sugarscape`);
      mutate(c);
    };
  }

  /** Runs `mutate` on `c`; a throw (an unknown path, the wrong model) becomes a field error. */
  function tryMutate(mutate: (c: ModelConfig) => void, c: ModelConfig): FieldError[] | null {
    try {
      mutate(c);
      return null;
    } catch (e) {
      return [{ field: 'config', message: e instanceof Error ? e.message : String(e) }];
    }
  }
  ```
- In `defaultDeps`: after `const presets = …;` add `const schemas = JSON.parse(model_schemas_json()) as Partial<Record<ModelKind, Param[]>>;` and return `{ presets, schemas, transport }`.
- Fields: `baseConfig!: Config;` → `baseConfig!: ModelConfig;`; `config!: Config;` → `config!: ModelConfig;`; `latest: Snapshot | null = null;` → `latest: ModelStats | null = null;` followed by
  ```ts
    /** Ring World's sugar and agents as of the latest snapshot (the ring view); null in other models. */
    ring: RingState | null = null;
  ```
  `origin`'s `config: Config` → `config: ModelConfig`; after `private lastRefresh = -Infinity;` add
  ```ts
    /** The config the sugarscape panels show while another model runs: the last sugarscape config seen. */
    private lastSugar!: Config;
  ```
- The constructor gains `readonly schemas: Partial<Record<ModelKind, Param[]>>,` after `readonly presets: Preset[],`.
- In `create`: `const { presets, transport } = …` → `const { presets, schemas = {}, transport } = deps ?? (await defaultDeps());`; `new Engine(transport, presets, initial?.seed ?? randomSeed())` → `new Engine(transport, presets, schemas, initial?.seed ?? randomSeed())`; after that line add
  ```ts
      const firstSugar = presets.map((p) => p.config).find(isSugar);
      if (!firstSugar) throw new Error('no sugarscape preset');
      engine.lastSugar = structuredClone(firstSugar);
  ```
  and after `const config = initial?.config ?? structuredClone(fallback.config);` add
  ```ts
      // Until the init snapshot brings the normalized config, `wants()` reads the model from this one.
      engine.config = config;
  ```
- Before `on(event: EngineEvent, …)` add
  ```ts
    /** The model of the world running now. */
    get model(): ModelKind {
      return modelOf(this.config);
    }

    /**
     * The config the sugarscape panels (Rules, Display, tools, Inspect, Credit, Charts) read: the
     * live config while the world is a sugarscape, otherwise the last sugarscape config this engine
     * ran (those panels are hidden then, Decision 6). Never write through it.
     */
    get sugar(): Config {
      return isSugar(this.config) ? this.config : this.lastSugar;
    }
  ```
- `reset(config?: Config, seed?: number)` → `reset(config?: ModelConfig, seed?: number)`.
- Replace `resetWith` and `applyConfig` (docs included) with
  ```ts
    /**
     * A reset to the base config as changed by `mutate` (a reset-requiring rule edit), built when it
     * runs so that an earlier change still in flight is kept. The sugarscape Rules panel's; it fails
     * when the world is another model.
     */
    resetWith(mutate: (c: Config) => void, seed?: number): Promise<FieldError[] | null> {
      return this.resetModelWith(sugarOnly(mutate), seed);
    }

    /** `resetWith` for a config of any model (the schema-driven Rules panel's). */
    resetModelWith(mutate: (c: ModelConfig) => void, seed?: number): Promise<FieldError[] | null> {
      return this.quiet(() => {
        const next = structuredClone(this.baseConfig);
        const refused = tryMutate(mutate, next);
        if (refused) return Promise.resolve(refused);
        return this.rebuild(next, seed ?? this.seed, this.keptLandscapes(next));
      });
    }

    /**
     * Applies a rule/parameter change to the running world: `mutate` edits a copy of the live
     * config for the world and, on success, a copy of the base config too, so scheduled changes
     * that already fired are not undone. The sugarscape Rules panel's; it fails when the world is
     * another model.
     */
    applyConfig(mutate: (c: Config) => void): Promise<FieldError[] | null> {
      return this.applyModelConfig(sugarOnly(mutate));
    }

    /** `applyConfig` for a config of any model (the schema-driven Rules panel's live fields). */
    applyModelConfig(mutate: (c: ModelConfig) => void): Promise<FieldError[] | null> {
      return this.quiet(async () => {
        const next = structuredClone(this.config);
        const refused = tryMutate(mutate, next);
        if (refused) return refused;
        const result = await this.send({ type: 'setConfig', config: next }, true);
        if (!result.ok || !result.snapshot) return writeFailure(result);
        const base = structuredClone(this.baseConfig);
        mutate(base);
        this.baseConfig = base;
        // The reply carries the new config, so accepting it fires 'config'.
        this.accept(result.snapshot);
        void this.refresh();
        return null;
      });
    }
  ```
- In `takeWorld`, after `this.config = other.config;` add `this.lastSugar = other.lastSugar;` and `this.ring = other.ring;`.
- In `wants`, replace the two `networks` lines with
  ```ts
      // Overlays exist only in a sugarscape; the ring view needs Ring World's state with every snapshot.
      const networks = this.model === 'sugarscape' ? OVERLAYS.filter((k) => this.overlays[k]) : [];
      if (networks.length > 0) own.networks = networks;
      if (this.model === 'ring') own.ring = true;
  ```
- In `adopt`, replace `if (s.config) this.config = s.config;` with
  ```ts
      if (s.config) {
        this.config = s.config;
        if (isSugar(s.config)) this.lastSugar = s.config;
      }
      // A world of another model has no ring; Ring World sends its state with every snapshot.
      this.ring = s.ring ?? (this.model === 'ring' ? this.ring : null);
  ```
- `keptLandscapes(config: Config)` → `keptLandscapes(config: ModelConfig)`, and after `const base = this.baseConfig;` add
  ```ts
      // Only a sugarscape has maps, and only a sugarscape's maps carry over.
      if (!isSugar(config) || !isSugar(base)) return [];
  ```
- `rebuild(config: Config, …)` → `rebuild(config: ModelConfig, …)`.

- [ ] **Step 6: The boundaries and the sugarscape panels**

- `web/src/share.ts`: `import type { Config } from './types';` → `import type { ModelConfig } from './types';`; `ShareState`'s and `Wire`'s `Config` → `ModelConfig`; in `decodeCommand` `a[0] as unknown as Config` → `a[0] as unknown as ModelConfig`; in `fromWire` the comment and line become
  ```ts
    // A v1 config is in the pre-N-goods shape; the WASM side converts it. A config without a
    // `model` key is a sugarscape's (every link made before milestone 9).
    const state: ShareState = { config: c as unknown as ModelConfig, seed: s >>> 0, log: [] };
  ```
- `web/src/sessions.ts`: `Config` → `ModelConfig` (the import and `baseConfig: ModelConfig;`).
- `web/src/experiments/types.ts`: import `ModelConfig` instead of `Config`; `export type SweepBase = …` becomes `/** A preset of any model, or a config of any model. */\nexport type SweepBase = { preset: string } | { config: ModelConfig };`.
- `web/src/experiments/form.ts`: import `type FieldError, ModelConfig` instead of `Config, FieldError`; `numericPaths(config: Config)` → `numericPaths(config: ModelConfig)`.
- `web/src/experiments/view.ts`: the types import becomes `import { parseErrors, type FieldError, type ModelConfig } from '../types';`; `private configOf(base: SweepBase): Config | null {` → `…: ModelConfig | null {`.
- `web/src/ui/rules-panel.ts`: every `this.engine.config` becomes `this.engine.sugar` (the schedule section, the switch and control syncers, the custom editors).
- `web/src/ui/charts-panel.ts`: every `w.config` becomes `w.sugar` and `this.engine.config` becomes `this.engine.sugar`.
- `web/src/ui/display.ts`: `layerOptions(engine.config)` → `layerOptions(engine.sugar)`; `const configs = () => (b ? [engine.config, b.config] : [engine.config]);` → `const configs = () => (b ? [engine.sugar, b.sugar] : [engine.sugar]);`.
- `web/src/ui/inspect-panel.ts`: add `import { isSugarView } from '../models';` after the `Engine` import; every `this.engine.config` becomes `this.engine.sugar`; in `render`, replace `const { site, agent } = shown.view;` with
  ```ts
      // Only a sugarscape's inspection has these rows; the other models' are added with their Inspect rows.
      if (!isSugarView(shown.view)) return;
      const { site, agent } = shown.view;
  ```
- `web/src/ui/tools.ts`: `const diseaseOn = () => targets.some((t) => t.engine.config.disease.enabled);` → `const diseaseOn = () => targets.some((t) => t.engine.model === 'sugarscape' && t.engine.sugar.disease.enabled);`; in `refreshGoods` `primary.engine.config.goods` → `primary.engine.sugar.goods`; the disease-list provider becomes
  ```ts
        engine.want((now) =>
          DISEASE_TOOLS.includes(tool) && engine.model === 'sugarscape' && engine.sugar.disease.enabled && entry.poll.due(now, engine.tick)
            ? { diseaseList: true }
            : {},
        ),
  ```
- `web/src/main.ts`: `syncCreditTab` becomes
  ```ts
    const syncCreditTab = (b: Engine | null) =>
      tabs.setHidden('Credit', ![engine, b].some((e) => e && e.model === 'sugarscape' && e.sugar.credit.enabled));
  ```
- `web/src/compare/compare-view.ts`: in `syncCredit`, `const on = …` becomes
  ```ts
      const on = (w: WorldName) => {
        const e = w === 'A' ? this.p.engine : this.b;
        return e.model === 'sugarscape' && e.sugar.credit.enabled;
      };
  ```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: PASS — every existing test, plus models, the model clamp, `SimHost with another model`, `Engine with other models` and `other models through the engine` (both fingerprints through the real WASM host path, with sugarscape-only wishes ignored).

- [ ] **Step 8: Commit**

```bash
git add web/src/types.ts web/src/models.ts web/src/models.test.ts web/src/protocol.ts web/src/layers.ts web/src/layers.test.ts web/src/sim-host.ts web/src/sim-host.test.ts web/src/fake-sim.fixture.ts web/src/engine.ts web/src/engine.test.ts web/src/share.ts web/src/share.test.ts web/src/sessions.ts web/src/sessions.test.ts web/src/experiments/types.ts web/src/experiments/form.ts web/src/experiments/view.ts web/src/main.ts web/src/compare/compare-view.ts web/src/ui/rules-panel.ts web/src/ui/charts-panel.ts web/src/ui/inspect-panel.ts web/src/ui/display.ts web/src/ui/tools.ts web/src/compare-presets.test.ts web/src/determinism.test.ts
git commit -m "Carry model-tagged configs through the page's engine, host and links" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---


### Task 6: Presets by model, the schema-driven Rules panel and one model per Compare

*Needs judgement (full code given; check the panel reads well for both models and that choosing another model while comparing leaves Compare cleanly).* Browser (controller), `/?debug`: the Rules tab's menu shows optgroups **Sugarscape** (the 29 presets), **Schelling** (4), **Ring World** (2), **Compare** (1); choosing `vi-4-schelling-25` rebuilds the world as Schelling (the grid shows Red/Blue agents — the Agents menu is fixed in Task 7, so the frame may use the clamped mode), the sugarscape sections disappear and **Setup** (Width, Height, Agents), **Preference** and **Residence** appear with "Changing these rebuilds the world."; changing Agents to 1000 rebuilds with 1000 agents and marks the preset **modified**; `vi-8-ring-world` shows Setup, Agents and Sugar ("These apply to the running world."); changing Growback to 2 while running does not reset t and a share link made afterwards replays it; `ii-2-unit` brings the sugarscape panel back unchanged; in Compare on `vi-4-schelling-25` (Compare button), B's "Rules for: B" menu lists only Schelling presets, and choosing `vi-8-ring-world` in A's menu leaves Compare keeping A and loads Ring World; a `#c=` link whose worlds are of different models shows the "could not be loaded" banner.

**Files:**
- Create: `web/src/schema-form.ts`, `web/src/schema-form.test.ts`, `web/src/ui/schema-panel.ts`
- Modify: `web/src/ui/rules-panel.ts`, `web/src/main.ts`, `web/src/compare/compare-view.ts`, `web/src/share.ts`, `web/src/compare-presets.ts`
- Test: `web/src/share.test.ts`, `web/src/compare-presets.test.ts`

**Interfaces:**
- Consumes: `Engine.{schemas, model, config, presets, applyModelConfig, resetModelWith, loadPreset}`, `presetGroups`, `presetModel`, `modelOf` (Task 5); `paths.{getPath, setPath, errorsFor}`.
- Produces:
  - `schema-form.ts`: `type ParamInput = string | boolean | { min: string; max: string }`; `groupParams(params: Param[]): { group: string; params: Param[] }[]`; `paramEdit(p: Param, input: ParamInput): (c: ModelConfig) => void`; `paramInput(p: Param, config: ModelConfig): ParamInput`.
  - `ui/schema-panel.ts`: `class SchemaPanel { readonly el; constructor(engine: Engine); sync(): void }`.
  - `ui/rules-panel.ts`: `interface RulesOptions { onCompare?: (id: string) => void; beforeModelChange?: () => Promise<boolean>; sameModelOnly?: boolean }`; `new RulesPanel(engine, opts?: RulesOptions)`.

- [ ] **Step 1: Write the failing tests**

Create `web/src/schema-form.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { groupParams, paramEdit, paramInput } from './schema-form';
import type { ModelConfig, Param, RingConfig, SchellingConfig } from './types';

const ring = (): RingConfig => ({
  model: 'ring',
  sites: 150,
  agents: 40,
  vision: { min: 15, max: 30 },
  capacity: 4,
  growback: 1,
  start: 'random',
});
const param = (p: Partial<Param> & Pick<Param, 'path' | 'kind'>): Param => ({ label: p.path, apply: 'reset', group: 'Setup', ...p });

describe('the schema form', () => {
  it('groups params in the order their groups first appear', () => {
    const params = [param({ path: 'a', kind: 'integer' }), param({ path: 'b', kind: 'bool', group: 'Other' }), param({ path: 'c', kind: 'integer' })];
    expect(groupParams(params).map((s) => [s.group, s.params.map((p) => p.path)])).toEqual([
      ['Setup', ['a', 'c']],
      ['Other', ['b']],
    ]);
  });

  it('writes each kind of field: whole numbers, numbers, ranges, switches and choices', () => {
    const c: ModelConfig = ring();
    paramEdit(param({ path: 'sites', kind: 'integer' }), '99.6')(c);
    paramEdit(param({ path: 'growback', kind: 'number', step: 0.1 }), '2.5')(c);
    paramEdit(param({ path: 'vision', kind: 'range', step: 1 }), { min: '3', max: '7.2' })(c);
    paramEdit(param({ path: 'start', kind: 'choice' }), 'megagroup')(c);
    expect(c).toMatchObject({ sites: 100, growback: 2.5, vision: { min: 3, max: 7 }, start: 'megagroup' });
    const s = { model: 'schelling', preference: { min: 0.25, max: 0.25 }, residence: { enabled: false, min: 80, max: 100 } } as SchellingConfig;
    paramEdit(param({ path: 'preference', kind: 'range', step: 0.05 }), { min: '0.25', max: '0.5' })(s);
    paramEdit(param({ path: 'residence.enabled', kind: 'bool' }), true)(s);
    paramEdit(param({ path: 'residence', kind: 'range', step: 1 }), { min: '10', max: '20' })(s);
    expect(s.preference).toEqual({ min: 0.25, max: 0.5 });
    expect(s.residence).toEqual({ enabled: true, min: 10, max: 20 });
  });

  it('refuses a path the config does not have', () => {
    expect(() => paramEdit(param({ path: 'vision.maxx', kind: 'integer' }), '3')(ring())).toThrow('unknown field vision.maxx');
  });

  it('reads each field back as its control shows it', () => {
    const c = ring();
    expect(paramInput(param({ path: 'vision', kind: 'range' }), c)).toEqual({ min: '15', max: '30' });
    expect(paramInput(param({ path: 'start', kind: 'choice' }), c)).toBe('random');
    expect(paramInput(param({ path: 'growback', kind: 'number' }), c)).toBe('1');
  });
});
```
In `web/src/share.test.ts`'s `describe('models in links'`, after the `'carries another model’s tagged config …'` test, add
```ts

  it('refuses a comparison of two models (a link or a file)', async () => {
    const schelling = { model: 'schelling' } as SchellingConfig;
    const mixed = { a: { config, seed: 1 }, b: { config: schelling, seed: 1 } };
    await expect(decodeCompare(await encodeCompare(mixed))).rejects.toThrow('not a SugarScape compare link');
    expect(() => parseSessionFile(sessionFileText({ kind: 'compare', state: mixed }))).toThrow('not a SugarScape session file');
  });
```
In `web/src/compare-presets.test.ts`, replace the `'is null when a preset is missing'` test with
```ts
  it('is null when a preset is missing or the two are of different models', () => {
    expect(comparePresetStates([preset('vi-2-no-trade', false)], entry, 1)).toBeNull();
    const ring: Preset = { ...preset('vi-3-trade', true), config: { model: 'ring' } as unknown as Config };
    expect(comparePresetStates([preset('vi-2-no-trade', false), ring], entry, 1)).toBeNull();
  });
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `(cd web && npx vitest run src/schema-form.test.ts src/share.test.ts src/compare-presets.test.ts)`
Expected: FAIL — `./schema-form` does not exist; the mixed comparison decodes; `comparePresetStates` returns B's state.

- [ ] **Step 3: Implement the form helpers and the panel**

Create `web/src/schema-form.ts` (Decision 8):
```ts
// The schema-driven Rules panel's pure parts (Decision 8): grouping, reading and writing fields.
import { getPath, setPath } from './paths';
import type { ModelConfig, Param } from './types';

/** What a control's input holds: a number box's text, a checkbox, a choice, or a range's two boxes. */
export type ParamInput = string | boolean | { min: string; max: string };

/** The params in panel sections, in the order each group first appears. */
export function groupParams(params: Param[]): { group: string; params: Param[] }[] {
  const out: { group: string; params: Param[] }[] = [];
  for (const p of params) {
    let section = out.find((s) => s.group === p.group);
    if (!section) out.push((section = { group: p.group, params: [] }));
    section.params.push(p);
  }
  return out;
}

/** A number from a box: whole for `integer` params and ranges stepping by whole numbers. */
function number(p: Param, raw: string): number {
  const v = Number(raw);
  return p.kind === 'integer' || (p.kind === 'range' && Number.isInteger(p.step ?? 1)) ? Math.round(v) : v;
}

/**
 * The edit a control makes: sets `path` (a range sets `path.min` and `path.max`). The values are
 * sent as typed; the core validates them and names the field in its errors.
 */
export function paramEdit(p: Param, input: ParamInput): (c: ModelConfig) => void {
  return (c) => {
    if (p.kind === 'range') {
      const r = input as { min: string; max: string };
      setPath(c, `${p.path}.min`, number(p, r.min));
      setPath(c, `${p.path}.max`, number(p, r.max));
    } else if (p.kind === 'bool') {
      setPath(c, p.path, input === true);
    } else if (p.kind === 'choice') {
      setPath(c, p.path, String(input));
    } else {
      setPath(c, p.path, number(p, String(input)));
    }
  };
}

/** The control's current value in `config`, as its input shows it. */
export function paramInput(p: Param, config: ModelConfig): ParamInput {
  const v = getPath(config, p.path);
  if (p.kind === 'range') {
    const r = v as { min: number; max: number };
    return { min: String(r.min), max: String(r.max) };
  }
  if (p.kind === 'bool') return v === true;
  return String(v);
}
```
Create `web/src/ui/schema-panel.ts`:
```ts
import type { Engine } from '../engine';
import { errorsFor } from '../paths';
import { groupParams, paramEdit, paramInput, type ParamInput } from '../schema-form';
import type { FieldError, ModelKind, Param } from '../types';
import { h } from './dom';

/** A section's note: whether its fields rebuild the world or apply to it as it runs. */
function note(params: Param[]): string {
  if (params.every((p) => p.apply === 'reset')) return 'Changing these rebuilds the world.';
  if (params.every((p) => p.apply === 'live')) return 'These apply to the running world.';
  return 'Fields marked ↺ rebuild the world; the rest apply to it as it runs.';
}

/**
 * The Rules panel of a model other than the sugarscape, built from its schema (Decision 8): one
 * section per group; live fields edit the running world (logged as `setConfig`), reset fields
 * rebuild it from the base config.
 */
export class SchemaPanel {
  readonly el = h('div', { class: 'schema' });
  private model: ModelKind | null = null;
  private syncers: (() => void)[] = [];
  private slots: { path: string; el: HTMLElement }[] = [];
  private general = h('div', { class: 'error' });
  private errors: FieldError[] = [];

  constructor(private readonly engine: Engine) {}

  /** Shows the engine's model's fields (rebuilt when the model changes) with the live config's values. */
  sync(): void {
    const model = this.engine.model;
    if (model !== this.model) this.build(model);
    this.syncers.forEach((s) => s());
  }

  private build(model: ModelKind): void {
    this.model = model;
    this.syncers = [];
    this.slots = [];
    this.errors = [];
    const sections = groupParams(this.engine.schemas[model] ?? []).map(({ group, params }) =>
      h('section', { class: 'group' }, h('h3', {}, group), h('p', { class: 'hint' }, note(params)), ...params.map((p) => this.control(p, params))),
    );
    this.el.replaceChildren(this.general, ...sections);
    this.renderErrors();
  }

  private async commit(p: Param, input: ParamInput): Promise<void> {
    const edit = paramEdit(p, input);
    const errors = p.apply === 'live' ? await this.engine.applyModelConfig(edit) : await this.engine.resetModelWith(edit);
    this.errors = errors ?? [];
    if (this.errors.length > 0) this.sync();
    this.renderErrors();
  }

  private renderErrors(): void {
    const claimed = new Set<FieldError>();
    for (const slot of this.slots) {
      const mine = errorsFor(this.errors, slot.path);
      mine.forEach((e) => claimed.add(e));
      slot.el.replaceChildren(...mine.map((e) => h('p', {}, e.message)));
    }
    const rest = this.errors.filter((e) => !claimed.has(e));
    this.general.replaceChildren(...rest.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }

  private control(p: Param, section: Param[]): HTMLElement {
    const slot = h('div', { class: 'error' });
    this.slots.push({ path: p.path, el: slot });
    const mixed = section.some((q) => q.apply !== p.apply);
    const label = `${p.label}${mixed && p.apply === 'reset' ? ' ↺' : ''}`;
    const current = () => paramInput(p, this.engine.config);
    const bounds = { min: p.min, max: p.max, step: p.step };
    switch (p.kind) {
      case 'bool': {
        const box = h('input', { type: 'checkbox', onchange: () => void this.commit(p, box.checked) });
        this.syncers.push(() => (box.checked = current() === true));
        return h('div', { class: 'control' }, h('label', { class: 'switch' }, box, ` ${label}`), slot);
      }
      case 'choice': {
        const select = h(
          'select',
          { onchange: () => void this.commit(p, select.value) },
          ...(p.choices ?? []).map((c) => h('option', { value: c.value }, c.label)),
        );
        this.syncers.push(() => (select.value = String(current())));
        return h('div', { class: 'control' }, h('label', {}, label), select, slot);
      }
      case 'range': {
        const lo = h('input', { type: 'number', class: 'num', ...bounds });
        const hi = h('input', { type: 'number', class: 'num', ...bounds });
        const apply = () => void this.commit(p, { min: lo.value, max: hi.value });
        lo.addEventListener('change', apply);
        hi.addEventListener('change', apply);
        this.syncers.push(() => {
          const r = current() as { min: string; max: string };
          lo.value = r.min;
          hi.value = r.max;
        });
        return h('div', { class: 'control' }, h('label', {}, label), h('div', { class: 'row' }, lo, h('span', { class: 'hint' }, 'to'), hi), slot);
      }
      default: {
        const slider = h('input', { type: 'range', ...bounds });
        const num = h('input', { type: 'number', class: 'num', ...bounds });
        slider.addEventListener('input', () => (num.value = slider.value));
        slider.addEventListener('change', () => void this.commit(p, slider.value));
        num.addEventListener('change', () => void this.commit(p, num.value));
        this.syncers.push(() => {
          const v = String(current());
          slider.value = v;
          num.value = v;
        });
        return h('div', { class: 'control' }, h('label', {}, label), h('div', { class: 'row' }, slider, num), slot);
      }
    }
  }
}
```

`web/src/ui/rules-panel.ts`:
- Imports: after `import { groupsEditorSignature } from '../groups';` add `import { presetGroups, presetModel } from '../models';`; after `import { pollutionEditor } from './pollution-editor';` add `import { SchemaPanel } from './schema-panel';`.
- Replace the class doc and the head of the class, down to the first line of the constructor body, with
  ```ts
  export interface RulesOptions {
    /** A's panel: the presets menu's Compare entries work, getting the entry's id. */
    onCompare?: (id: string) => void;
    /**
     * A's panel: runs before another model's preset loads; resolving false cancels it (Compare pairs
     * one model, so it is left first — Decision 11).
     */
    beforeModelChange?: () => Promise<boolean>;
    /** B's panel in Compare: the menu offers only its world's model's presets. */
    sameModelOnly?: boolean;
  }

  /**
   * The preset picker (grouped by model), then the sugarscape's sections (one per rule, from GROUPS)
   * or, for another model, its schema-driven panel.
   */
  export class RulesPanel {
    readonly el = h('div', { class: 'rules' });
    private errors: FieldError[] = [];
    /** The preset picker's syncer (every model). */
    private presetSync: () => void = () => {};
    /** The sugarscape sections' syncers. */
    private syncers: (() => void)[] = [];
    /** The Goods editor's and Pollution table's syncers, which painting cannot affect. */
    private editorSyncers: (() => void)[] = [];
    private errorSlots: { path: string; el: HTMLElement; withField: boolean }[] = [];
    private general = h('div', { class: 'error' });
    private readonly sugarBody: HTMLElement;
    private readonly schema: SchemaPanel;

    constructor(
      private engine: Engine,
      private opts: RulesOptions = {},
    ) {
      this.sugarBody = h('div', {}, this.scheduleSection(), this.general, ...GROUPS.map((g) => this.groupSection(g)));
      this.schema = new SchemaPanel(engine);
      this.el.append(this.presetSection(), this.sugarBody, this.schema.el);
  ```
  (the rest of the constructor — the three `engine.on` lines and `this.sync()` — stays).
- `sync` becomes
  ```ts
    private sync(editors = true): void {
      this.presetSync();
      const sugar = this.engine.model === 'sugarscape';
      this.sugarBody.hidden = !sugar;
      this.schema.el.hidden = sugar;
      if (!sugar) {
        this.schema.sync();
        return;
      }
      this.syncers.forEach((s) => s());
      if (editors) this.editorSyncers.forEach((s) => s());
    }
  ```
- In `presetSection`'s `onchange`: `this.onCompare?.(compare.id);` → `this.opts.onCompare?.(compare.id);`, and before `this.errors = (await this.engine.loadPreset(select.value)) ?? [];` add
  ```ts
            const preset = this.engine.presets.find((p) => p.id === select.value);
            const leaving = preset !== undefined && presetModel(preset) !== this.engine.model;
            if (leaving && this.opts.beforeModelChange && !(await this.opts.beforeModelChange())) {
              this.sync();
              return;
            }
  ```
  replace the options
  ```ts
        ...this.engine.presets.map((p) => h('option', { value: p.id }, `${p.name} — ${p.source}`)),
        this.onCompare
  ```
  with
  ```ts
        ...presetGroups(this.engine.presets)
          .filter((g) => !this.opts.sameModelOnly || g.model === this.engine.model)
          .map((g) => h('optgroup', { label: g.label }, ...g.presets.map((p) => h('option', { value: p.id }, `${p.name} — ${p.source}`)))),
        this.opts.onCompare
  ```
  and replace `this.syncers.push(() => {` … `});` around the badge/description syncer with `this.presetSync = () => {` … `};` (same body).

- [ ] **Step 4: One model per Compare (Decision 11)**

`web/src/share.ts`: add `import { modelOf } from './models';` after the `classifyFile` import; before `decodeCompare` add
```ts
/** Both worlds of a comparison run one model (Decision 11). */
function samePair(a: ShareState, b: ShareState): CompareState {
  return modelOf(a.config) === modelOf(b.config) ? { a, b } : bad();
}
```
and use it: in `decodeCompare` `return { a: fromWire(json.a), b: fromWire(json.b) };` → `return samePair(fromWire(json.a), fromWire(json.b));`; in `parseSessionFile` the compare line becomes `if (isObject(json) && json.sugarscape === 'compare') return { kind: 'compare', state: samePair(fromWire(json.a), fromWire(json.b)) };`.

`web/src/compare-presets.ts`: add `import { presetModel } from './models';`; the doc's first line becomes `* A's seed and B's setup: the entry's two presets at \`seed\`, or null if either preset is missing or\n * they are of different models (Compare pairs one model).`; `if (!a || !b) return null;` → `if (!a || !b || presetModel(a) !== presetModel(b)) return null;`.

`web/src/compare/compare-view.ts`: `p.rules.setB(new RulesPanel(b));` → `p.rules.setB(new RulesPanel(b, { sameModelOnly: true }));`.

`web/src/main.ts`: A's panel becomes
```ts
  const rules = new WorldSlot(
    new RulesPanel(engine, { onCompare: (id) => void openComparePreset(id), beforeModelChange: leaveCompareForModel }),
    'switch',
    'Rules for',
  );
```
and before `async function toggleCompare()` add
```ts
  /**
   * Before A loads another model's preset: Compare pairs one model, so it is left keeping A first
   * (Decision 11). Resolves whether the preset may load.
   */
  async function leaveCompareForModel(): Promise<boolean> {
    if (!compare) return true;
    if (busy) {
      showNotice('Compare is starting or ending; try again in a moment');
      return false;
    }
    await leaveCompare('A');
    return compare === null;
  }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src/schema-form.ts web/src/schema-form.test.ts web/src/ui/schema-panel.ts web/src/ui/rules-panel.ts web/src/main.ts web/src/compare/compare-view.ts web/src/share.ts web/src/share.test.ts web/src/compare-presets.ts web/src/compare-presets.test.ts
git commit -m "Group presets by model, build other models' Rules panel from their schema, keep Compare to one model" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 7: Views — the display per model, the ring and its space–time diagram

*Needs judgement (full code given; the ring view is DOM-only, so the controller's pass is its test beyond the geometry).* Browser (controller), `/?debug`: on `vi-4-schelling-25` the display shows only **Agents** with **Colour**, **Satisfaction** (satisfied agents dimmed, the unsatisfied yellow — none once quiet at t ≈ 3) and **Preference** (all one shade at 25 %; a spread on `vi-7-schelling-mixed`), no Landscape menu and no overlay checkboxes; on `vi-8-ring-world` the display row is empty, the ring (dark square, sugar band, blue agent dots, site 0 at the top) sits above the space–time diagram (rows filling from the bottom, blue diagonal streaks of moving flocks after a few hundred ticks at Max); clicking the ring selects a site (outlined in accent on the ring, boxed on the diagram's bottom row) and shows Inspect; `vi-9-ring-megagroup` starts as one blue bar that breaks up; back on `ii-2-unit` the display is exactly as before (Tribe mode, Landscape menu, overlays); in Compare on `vi-8-ring-world` B's figure has its own ring and diagram.

**Files:**
- Create: `web/src/ring.ts`, `web/src/ring.test.ts`, `web/src/ui/ring-view.ts`
- Modify: `web/src/ui/display.ts`, `web/index.html`, `web/src/style.css`, `web/src/main.ts`, `web/src/compare/compare-view.ts`

**Interfaces:**
- Consumes: `Engine.{model, ring, config, selection, size, select, colorMode, layer, overlays, setDisplay, sugar}`, `COLOR_MODES` (Task 5); `h()`.
- Produces: `ring.ts`: `RING_HISTORY = 150`, `RING_BACKGROUND`, `RING_SUGAR`, `RING_AGENT`, `siteAngle(i, sites)`, `siteAt(dx, dy, sites)`, `sugarShade(sugar, capacity)`; `ui/ring-view.ts`: `class RingView { onSite: ((site: number) => void) | null; readonly canvas; constructor(canvas, engine); draw(): void }`; `Playground.ringView: RingView`; `CompareShell.ring: HTMLCanvasElement`; `CompareView.ringB: RingView`.

- [ ] **Step 1: Write the failing test**

Create `web/src/ring.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { siteAngle, siteAt, sugarShade } from './ring';

describe('the ring view geometry', () => {
  it('puts site 0 at the top and runs counterclockwise', () => {
    expect(siteAngle(0, 4)).toBeCloseTo(-Math.PI / 2);
    // A quarter of the way round counterclockwise from the top is the left (canvas y down).
    const a = siteAngle(1, 4);
    expect(Math.cos(a)).toBeCloseTo(-1);
    expect(Math.sin(a)).toBeCloseTo(0);
  });

  it('finds the site under a point, whatever its angle', () => {
    for (const sites of [10, 150, 1000]) {
      for (let i = 0; i < sites; i += 7) {
        const a = siteAngle(i, sites);
        expect(siteAt(Math.cos(a) * 50, Math.sin(a) * 50, sites)).toBe(i);
      }
    }
    expect(siteAt(0, -1, 150)).toBe(0);
  });

  it('shades sugar from the background to sugar yellow', () => {
    expect(sugarShade(0, 4)).toBe('rgb(22, 21, 18)');
    expect(sugarShade(4, 4)).toBe('rgb(242, 193, 78)');
    expect(sugarShade(9, 4)).toBe('rgb(242, 193, 78)');
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `(cd web && npx vitest run src/ring.test.ts)`
Expected: FAIL — `./ring` does not exist.

- [ ] **Step 3: Implement**

Create `web/src/ring.ts` (Decision 12):
```ts
// Ring World's page-side geometry and colors (the ring view, Decision 12).

/** Rows of the host's space–time diagram (the core's `ring::HISTORY`). */
export const RING_HISTORY = 150;

/** The frame's colors (crates/sugarscape-core/src/render.rs `BACKGROUND`, `SUGAR`; ring.rs `AGENT`). */
export const RING_BACKGROUND: [number, number, number] = [0x16, 0x15, 0x12];
export const RING_SUGAR: [number, number, number] = [0xf2, 0xc1, 0x4e];
export const RING_AGENT = 'rgb(79, 157, 255)';

/**
 * Site `i`'s angle on the canvas (radians; canvas y points down): site 0 at the top and increasing
 * index counterclockwise, the direction agents look and move.
 */
export function siteAngle(i: number, sites: number): number {
  return -Math.PI / 2 - (2 * Math.PI * i) / sites;
}

/** The site whose angle is nearest the direction (dx, dy) from the centre (canvas coordinates). */
export function siteAt(dx: number, dy: number, sites: number): number {
  const turns = (-Math.PI / 2 - Math.atan2(dy, dx)) / (2 * Math.PI);
  const i = Math.round((((turns % 1) + 1) % 1) * sites);
  return i % sites;
}

/** A site's color: dark to sugar yellow by `sugar / capacity` (as the diagram shades it). */
export function sugarShade(sugar: number, capacity: number): string {
  const t = capacity > 0 ? Math.min(1, Math.max(0, sugar / capacity)) : 0;
  const [r, g, b] = RING_BACKGROUND.map((c, k) => Math.round(c + (RING_SUGAR[k] - c) * t));
  return `rgb(${r}, ${g}, ${b})`;
}
```
Create `web/src/ui/ring-view.ts`:
```ts
import type { Engine } from '../engine';
import { RING_AGENT, sugarShade, siteAngle, siteAt } from '../ring';

/** The canvas behind the ring: the frame's dark background (as the grid's), so light agents show. */
const BACKGROUND = sugarShade(0, 1);
import type { RingConfig } from '../types';

/** The ring view's canvas size in pixels (CSS scales it). */
const SIZE = 600;
/** The sugar band's inner and outer radius, and the agents' dots' radius, as fractions of half the size. */
const INNER = 0.72;
const OUTER = 0.9;
const DOTS = 0.64;

/**
 * Ring World's ring (Decision 12): `sites` cells round a circle, shaded by sugar, site 0 at the top
 * and counterclockwise onwards, with a dot inside the band for each agent and the selected site
 * outlined. It draws the engine's `ring` state (sent with every snapshot of a Ring World).
 */
export class RingView {
  /** A pointerdown on the ring picks the site in that direction. */
  onSite: ((site: number) => void) | null = null;
  private readonly ctx: CanvasRenderingContext2D;

  constructor(
    readonly canvas: HTMLCanvasElement,
    private readonly engine: Engine,
  ) {
    canvas.width = SIZE;
    canvas.height = SIZE;
    this.ctx = canvas.getContext('2d')!;
    canvas.addEventListener('pointerdown', (e) => {
      const sites = this.engine.ring?.sugar.length;
      if (e.button !== 0 || !sites) return;
      const r = canvas.getBoundingClientRect();
      this.onSite?.(siteAt(e.clientX - r.left - r.width / 2, e.clientY - r.top - r.height / 2, sites));
    });
  }

  draw(): void {
    const ring = this.engine.ring;
    const config = this.engine.config as RingConfig;
    const ctx = this.ctx;
    ctx.fillStyle = BACKGROUND;
    ctx.fillRect(0, 0, SIZE, SIZE);
    if (this.engine.model !== 'ring' || !ring) return;
    const n = ring.sugar.length;
    const c = SIZE / 2;
    const half = Math.PI / n;
    for (let i = 0; i < n; i++) {
      const a = siteAngle(i, n);
      ctx.beginPath();
      ctx.arc(c, c, OUTER * c, a - half, a + half);
      ctx.arc(c, c, INNER * c, a + half, a - half, true);
      ctx.closePath();
      ctx.fillStyle = sugarShade(ring.sugar[i], config.capacity);
      ctx.fill();
    }
    ctx.fillStyle = RING_AGENT;
    const dot = Math.max(2, Math.min(6, (Math.PI * DOTS * c) / n));
    for (const site of ring.agents) {
      const a = siteAngle(site, n);
      ctx.beginPath();
      ctx.arc(c + Math.cos(a) * DOTS * c, c + Math.sin(a) * DOTS * c, dot, 0, 2 * Math.PI);
      ctx.fill();
    }
    const sel = this.engine.selection;
    if (sel && sel.x < n) {
      const a = siteAngle(sel.x, n);
      ctx.beginPath();
      ctx.arc(c, c, (OUTER + 0.02) * c, a - half, a + half);
      ctx.arc(c, c, (INNER - 0.02) * c, a + half, a - half, true);
      ctx.closePath();
      ctx.lineWidth = 3;
      ctx.strokeStyle = getComputedStyle(this.canvas).getPropertyValue('--accent').trim() || '#fff';
      ctx.stroke();
    }
  }
}
```
Replace `web/src/ui/display.ts` with:
```ts
import type { Engine, Overlay } from '../engine';
import { layerOptions, overlayAvailableAny } from '../layers';
import { COLOR_MODES } from '../models';
import type { ColorMode, Layer, ModelKind } from '../types';
import { h } from './dom';

/** The overlay checkboxes, in order (Chapter VI's three networks after the others). */
const OVERLAY_LABELS: [Overlay, string][] = [
  ['trade', 'Trade network'],
  ['credit', 'Credit network'],
  ['disease', 'Disease network'],
  ['neighbors', 'Neighbor network'],
  ['friends', 'Friends network'],
  ['family', 'Family network'],
];
/** The display controls' element, and (Decision 10) which worlds' configs its checkboxes offer for. */
export interface Display {
  el: HTMLElement;
  /** Compare on (a checkbox shows while either A's or B's config allows it) or off (A's alone). */
  setCompare(b: Engine | null): void;
}

/**
 * The Agents and Landscape menus and the overlay checkboxes, for the model on screen (Decision 12):
 * a sugarscape offers all of them; Schelling only its three color modes; Ring World none.
 */
export function buildDisplay(engine: Engine): Display {
  const mode = h('select', { onchange: () => engine.setDisplay({ colorMode: mode.value as ColorMode }) });
  const layer = h('select', { onchange: () => engine.setDisplay({ layer: layer.value as Layer }) });
  const modeLabel = h('label', {}, 'Agents ', mode);
  const layerLabel = h('label', {}, 'Landscape ', layer);
  const sync = () => {
    mode.value = engine.colorMode;
    layer.value = engine.layer;
  };
  let shownModel: ModelKind | null = null;
  /** The model's color modes (when the model changed), and the goods and pollutants (and their names), which change on reset and config. */
  const refill = () => {
    const model = engine.model;
    if (model !== shownModel) {
      shownModel = model;
      const modes = COLOR_MODES[model];
      mode.replaceChildren(...modes.map(([v, l]) => h('option', { value: v }, l)));
      modeLabel.hidden = modes.length === 0;
      layerLabel.hidden = model !== 'sugarscape';
    }
    layer.replaceChildren(...layerOptions(engine.sugar).map(([v, l]) => h('option', { value: v }, l)));
    sync();
  };
  engine.on('display', sync);
  engine.on('reset', refill);
  engine.on('config', refill);
  refill();
  let b: Engine | null = null;
  const configs = () => (b ? [engine.sugar, b.sugar] : [engine.sugar]);
  /** A checkbox per overlay (sugarscape only); Friends, Family and Disease show while either world on screen allows them. */
  const overlay = (kind: Overlay, label: string) => {
    const box = h('input', { type: 'checkbox', onchange: () => engine.setDisplay({ overlays: { [kind]: box.checked } }) });
    const el = h('label', {}, box, ` ${label}`);
    const show = () => (el.hidden = engine.model !== 'sugarscape' || !overlayAvailableAny(kind, configs()));
    engine.on('display', () => (box.checked = engine.overlays[kind]));
    engine.on('reset', show);
    engine.on('config', show);
    show();
    return { el, show };
  };
  const overlays = OVERLAY_LABELS.map(([kind, label]) => overlay(kind, label));
  const el = h('div', { class: 'display-controls' }, modeLabel, layerLabel, ...overlays.map((o) => o.el));
  let offB: (() => void)[] = [];
  const setCompare = (newB: Engine | null): void => {
    for (const off of offB) off();
    offB = [];
    b = newB;
    if (b) {
      const other = b;
      const show = () => overlays.forEach((o) => o.show());
      offB = [other.on('reset', show), other.on('config', show)];
    }
    overlays.forEach((o) => o.show());
  };
  return { el, setCompare };
}
```
`web/index.html`: before `<canvas id="grid" aria-label="Sugarscape grid"></canvas>` add
```html
            <canvas id="ring" class="ring-view" aria-label="Ring World's ring" hidden></canvas>
```
`web/src/style.css`: after the `.grid-b { … }` rule add
```css
/* Ring World: the ring above its space–time diagram, both smaller so the pair fits. */
.ring-view { width: 100%; max-width: min(100%, 46vh); aspect-ratio: 1; align-self: center; border-radius: 6px; touch-action: none; cursor: crosshair; }
body[data-model='ring'] #grid, body[data-model='ring'] .grid-b { max-width: min(100%, 36vh); }
```
`web/src/main.ts`:
- Import `RingView` (`import { RingView } from './ui/ring-view';` before the `RulesPanel` import).
- After `const grid = new GridView(…);` add
  ```ts
    // Ring World's ring, above its space–time diagram (the grid) — Decision 12.
    const ringView = new RingView(document.querySelector<HTMLCanvasElement>('#ring')!, engine);
    /** The page follows the model on screen: `data-model` for the layout, the ring view for Ring World. */
    const syncModel = () => {
      document.body.dataset.model = engine.model;
      ringView.canvas.hidden = engine.model !== 'ring';
    };
    engine.on('reset', syncModel);
    syncModel();
  ```
- After the `buildTools(…)` call add
  ```ts
    // A click on the ring inspects that site now (the diagram's bottom row is the current tick).
    ringView.onSite = (site) => {
      void engine.select(site, engine.size().height - 1);
      compare?.focus('A');
      tabs.show('Inspect');
    };
  ```
- In the `playground` object add `ringView,` after `display,`; in the frame loop, after `grid.draw();` add `ringView.draw();`.

`web/src/compare/compare-view.ts`:
- Import `RingView` (`import { RingView } from '../ui/ring-view';` before the `RulesPanel` import); in `Playground`, after `display: Display;` add
  ```ts
    /** A's ring view (Ring World). */
    ringView: RingView;
  ```
- `CompareShell` and `compareShell` become
  ```ts
  /** B's figure: its header, and the copy's progress until its grid (and, for Ring World, its ring) can be shown. */
  export interface CompareShell { figure: HTMLElement; header: HTMLElement; progress: HTMLElement; canvas: HTMLCanvasElement; ring: HTMLCanvasElement }

  /** Puts B's figure beside A's (the grid area splits at once) with "Copying A…" in it. */
  export function compareShell(): CompareShell {
    const header = h('header', { class: 'world-header' });
    const progress = h('p', { class: 'hint copy-progress', role: 'status' }, 'Copying A…');
    const canvas = h('canvas', { class: 'grid-b', 'aria-label': 'Sugarscape grid B', hidden: true });
    const ring = h('canvas', { class: 'ring-view', 'aria-label': 'Ring World’s ring B', hidden: true });
    const figure = h('figure', { class: 'world', id: 'world-b' }, header, progress, ring, canvas);
    document.querySelector('#grids')!.append(figure);
    document.body.dataset.compare = 'on';
    return { figure, header, progress, canvas, ring };
  }
  ```
- In `CompareView`: after `readonly gridB: GridView;` add `readonly ringB: RingView;`; after `this.gridB = new GridView(shell.canvas, b);` add
  ```ts
      this.ringB = new RingView(shell.ring, b);
      // Both worlds run one model (Decision 11): B shows its ring exactly when A does.
      shell.ring.hidden = b.model !== 'ring';
      this.ringB.onSite = (site) => {
        void b.select(site, b.size().height - 1);
        this.focus('B');
        p.tabs.show('Inspect');
      };
  ```
  and in `draw()` after `this.gridB.draw();` add `this.ringB.draw();`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/ring.ts web/src/ring.test.ts web/src/ui/ring-view.ts web/src/ui/display.ts web/index.html web/src/style.css web/src/main.ts web/src/compare/compare-view.ts
git commit -m "Show each model's display, and Ring World's ring over its space–time diagram" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 8: Charts per model

*Mechanical (full code).* Browser (controller): Charts on `vi-4-schelling-25` shows exactly **Segregation** (rising from ≈ 0.5 to ≈ 0.63 by t ≈ 3, then flat), **Unsatisfied** (≈ 0.13 → 0), **Moves**, **Red share** (≈ 0.5) and no sugarscape chart or section; `vi-6-schelling-50-residence` at Max: Segregation climbs to ≈ 0.95 with Moves never settling to 0; `vi-8-ring-world`: **Flocks** (≈ 22 falling to ≈ 8), **Flock size** (Mean and Largest), **Distance moved**; paused, the tab asks for nothing once caught up; Compare on Schelling draws A solid and B dashed; `ii-2-unit`'s charts are exactly as before.

**Files:**
- Modify: `web/src/ui/series-data.ts`, `web/src/ui/charts-panel.ts`
- Test: `web/src/ui/series-data.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: `Engine.{model, sugar}` (Task 5); `config_series_names` (WASM).
- Produces: `series-data.ts`: `interface ChartLine { key; label; color }`, `interface ModelChart { title; lines: ChartLine[]; range?: [number, number] }`, `MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]>`, `showsForModel(chartModel: ModelKind, worlds: ModelKind[]): boolean`; `ChartDef.model?: ModelKind` (charts-panel.ts).

- [ ] **Step 1: Write the failing tests**

`web/src/ui/series-data.test.ts`: add `showsForModel,` to the `./series-data` import, before `twoGoods,`, and append
```ts

describe('model charts', () => {
  it('shows a model’s charts while some world on screen runs it', () => {
    expect(showsForModel('schelling', ['schelling'])).toBe(true);
    expect(showsForModel('sugarscape', ['schelling'])).toBe(false);
    expect(showsForModel('ring', ['sugarscape', 'ring'])).toBe(true);
  });
});
```
`web/src/determinism.test.ts`: add `import { modelOf } from './models';` and `import { MODEL_CHARTS } from './ui/series-data';`, change the WASM import to `import { config_series_names, initSync, presets_json } from './wasm-pkg/sugarscape.js';`, and append
```ts
describe('model charts', () => {
  it('draw only series their model records', () => {
    for (const [model, charts] of Object.entries(MODEL_CHARTS)) {
      const preset = presets.find((p) => modelOf(p.config) === model)!;
      const names = JSON.parse(config_series_names(JSON.stringify(preset.config))) as string[];
      for (const c of charts) for (const line of c.lines) expect(names, `${model}: ${c.title}`).toContain(line.key);
    }
  });
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `(cd web && npm run build)`
Expected: FAIL in `tsc` — `showsForModel` and `MODEL_CHARTS` are not exported.

- [ ] **Step 3: Implement**

`web/src/ui/series-data.ts`: the types import becomes `import type { Config, ModelKind } from '../types';`, and before `/** A world's distributions as last received:` add (Decision 13)
```ts
/** A line of a time chart: its series, legend label and color (a CSS variable or `#rrggbb`). */
export interface ChartLine { key: string; label: string; color: string }

/** A time chart of another model: its title, lines and y range. */
export interface ModelChart { title: string; lines: ChartLine[]; range?: [number, number] }

/**
 * The other models' charts (Decision 13), each a time chart of the model's own series:
 * Schelling's segregation, share unsatisfied, moves and Red share; Ring World's flocks, flock
 * size (mean and largest) and distance moved.
 */
export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]> = {
  schelling: [
    { title: 'Segregation', lines: [{ key: 'segregation', label: 'Like neighbors (mean share)', color: '--c2' }], range: [0, 1] },
    { title: 'Unsatisfied', lines: [{ key: 'unsatisfied', label: 'Unsatisfied share', color: '--red' }], range: [0, 1] },
    { title: 'Moves', lines: [{ key: 'moves', label: 'Agents moved', color: '--c1' }] },
    { title: 'Red share', lines: [{ key: 'red_share', label: 'Red', color: '--red' }], range: [0, 1] },
  ],
  ring: [
    { title: 'Flocks', lines: [{ key: 'flocks', label: 'Flocks', color: '--c1' }] },
    {
      title: 'Flock size',
      lines: [
        { key: 'mean_flock', label: 'Mean', color: '--c2' },
        { key: 'largest_flock', label: 'Largest', color: '--c3' },
      ],
    },
    { title: 'Distance moved', lines: [{ key: 'mean_distance', label: 'Sites per agent', color: '--c4' }] },
  ],
};

/** A chart of `chartModel` shows while some world on screen runs that model (Compare pairs one model). */
export function showsForModel(chartModel: ModelKind, worlds: ModelKind[]): boolean {
  return worlds.includes(chartModel);
}

```
`web/src/ui/charts-panel.ts`:
- The types import becomes `import type { Config, ModelKind } from '../types';`; in the `./series-data` import add `MODEL_CHARTS,` (after `lineData,`), `showsForModel,` (after `showsAgeHist,`) and `type ChartLine,` (after `twoGoods,`); replace `interface Line { key: string; label: string; color: string }` with `type Line = ChartLine;`.
- `ChartDef`'s doc becomes `/**\n * One chart: its lines follow a world's config; it shows when a world on screen runs its model\n * (default the sugarscape) and, for a sugarscape chart, its section and \`shown\` hold for that world.\n */` and it gains `model?: ModelKind;` after `section: Section;`.
- At the end of `CHARTS`, after the `'New infections'` entry, add
  ```ts
    // The other models' time charts (Decision 13), in the top section.
    ...(Object.entries(MODEL_CHARTS) as [ModelKind, (typeof MODEL_CHARTS)['ring']][]).flatMap(([model, charts]) =>
      charts.map((c): ChartDef => ({ title: c.title, kind: 'time', section: 'top', model, lines: fixed(c.lines), range: c.range })),
    ),
  ```
- `shown(def)` becomes
  ```ts
    private shown(def: ChartDef): boolean {
      const model = def.model ?? 'sugarscape';
      if (!showsForModel(model, this.worlds.map((w) => w.model))) return false;
      if (model !== 'sugarscape') return true;
      const section = SECTIONS.find((s) => s.id === def.section)!;
      return this.worlds.some((w) => w.model === 'sugarscape' && section.shown(w.sugar) && (def.shown?.(w.sugar) ?? true));
    }
  ```
- In `wants`, the distributions line becomes
  ```ts
      // Distributions exist only in a sugarscape.
      if (w.model === 'sugarscape' && distributionsDue(this.dist[i], w.tick, now, REFRESH_MS)) Object.assign(out, distributionWants(w.sugar));
  ```
- In `sync`: the signature becomes `JSON.stringify(this.worlds.map((w) => [w.model, CHARTS.map((d) => d.lines?.(w.sugar) ?? null)]))` (a model change rebuilds the plots), and `const configs = this.worlds.map((w) => w.sugar);` becomes `const configs = this.worlds.filter((w) => w.model === 'sugarscape').map((w) => w.sugar);` (sections only for a sugarscape).

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: PASS, including `model charts > draw only series their model records` (every `MODEL_CHARTS` line is in the model's `config_series_names`).

- [ ] **Step 5: Commit**

```bash
git add web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/ui/charts-panel.ts web/src/determinism.test.ts
git commit -m "Chart Schelling's and Ring World's own series" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---


### Task 9: Inspect per model, and only Inspect for the other models

*Mechanical (full code).* Browser (controller): on `vi-4-schelling-25` the tool row shows only **Inspect**; clicking an agent shows Site `(x, y)`, Agent `#id · Red|Blue`, Preference `at least 25% alike`, Satisfied `yes (2 of 3 neighbors alike)` (or `no neighbors`), Residence `age 5 (no maximum)`; on `vi-5-schelling-25-residence` Residence reads `age A of R`, and a selected agent tracked past its residence shows "Agent #id has left."; on `vi-8-ring-world` clicking the diagram or the ring shows Site `#x`, Sugar `s / 4`, and for an occupied site Agent `#id`, Vision `v sites`; no Follow button in either; choosing Schelling while the Paint tool is active switches to Inspect; back on `ii-2-unit` every tool and the sugarscape Inspect rows (with Follow) are as before; the Credit tab never appears for the new models.

**Files:**
- Modify: `web/src/ui/format.ts`, `web/src/ui/inspect-panel.ts`, `web/src/ui/tools.ts`
- Test: `web/src/ui/format.test.ts`

**Interfaces:**
- Consumes: `isSugarView`, `isRingView` (Task 5); `SchellingInspection`, `RingInspection` (types.ts); `Engine.model`.
- Produces: `ui/format.ts`: `percent(fraction: number): string`.

- [ ] **Step 1: Write the failing test**

`web/src/ui/format.test.ts`: `import { compactNumber } from './format';` becomes `import { compactNumber, percent } from './format';`; append
```ts

describe('percent', () => {
  it('writes whole percentages plainly and others to one decimal', () => {
    expect(percent(0.25)).toBe('25%');
    expect(percent(0.3)).toBe('30%');
    expect(percent(0.4137)).toBe('41.4%');
    expect(percent(1)).toBe('100%');
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `(cd web && npx vitest run src/ui/format.test.ts)`
Expected: FAIL — `percent` is not exported.

- [ ] **Step 3: Implement**

`web/src/ui/format.ts` — append
```ts

/** A fraction as a percentage to one decimal, without a trailing ".0": 0.25 -> "25%", 0.4137 -> "41.4%". */
export function percent(fraction: number): string {
  const p = Math.round(fraction * 1000) / 10;
  return `${Number.isInteger(p) ? p : p.toFixed(1)}%`;
}
```
`web/src/ui/inspect-panel.ts` (Decision 13):
- Imports: `import { isSugarView } from '../models';` → `import { isRingView, isSugarView } from '../models';`; `import type { AgentView, LinkView } from '../types';` → `import type { AgentView, LinkView, RingInspection, SchellingInspection } from '../types';`; after `import { h } from './dom';` add `import { percent } from './format';`.
- Before `private render(): void {` add
  ```ts
    /** Schelling's site and agent: color, preference, satisfaction and residence (Decision 13). */
    private schellingRows(view: SchellingInspection, gone: boolean): HTMLElement[] {
      const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
      const a = view.agent;
      const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
      if (!a || gone) return rows;
      const alike = a.neighbors === 0 ? 'no neighbors' : `${a.like} of ${a.neighbors} neighbors alike`;
      return [
        ...rows,
        row('Agent', `#${a.id} · ${a.color === 'red' ? 'Red' : 'Blue'}`),
        row('Preference', `at least ${percent(a.preference)} alike`),
        row('Satisfied', `${a.satisfied ? 'yes' : 'no'} (${alike})`),
        row('Residence', a.residence === null ? `age ${a.age} (no maximum)` : `age ${a.age} of ${a.residence}`),
      ];
    }

    /** Ring World's site and its agent (Decision 13). */
    private ringRows(view: RingInspection, gone: boolean): HTMLElement[] {
      const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
      const a = view.agent;
      return [
        row('Site', `#${view.site.x}`),
        row('Sugar', `${fmt(view.site.sugar)} / ${view.site.capacity}`),
        ...(a && !gone ? [row('Agent', `#${a.id}`), row('Vision', `${a.vision} sites`)] : []),
      ];
    }

  ```
- In `render`, replace Task 5's guard (the comment, `if (!isSugarView(shown.view)) return;` and `const { site, agent } = shown.view;`) with
  ```ts
      const view = shown.view;
      if (!isSugarView(view)) {
        // A Schelling agent that reached its maximum residence has left the landscape.
        const note = gone ? [h('p', { class: 'error' }, `Agent #${shown.agentId} has left.`)] : [];
        const rows = isRingView(view) ? this.ringRows(view, gone) : this.schellingRows(view, gone);
        this.el.replaceChildren(...note, h('table', {}, ...rows));
        return;
      }
      const { site, agent } = view;
  ```
`web/src/ui/tools.ts` — `syncAvailability` (doc included) becomes (the rest of the function, from `// Keeps the paint tool's layer …`, stays):
```ts
  /**
   * Shows only Inspect while no world on screen is a sugarscape (the other models have no editing
   * tools, Decision 13), and hides the disease tools while no world on screen has disease (leaving
   * a tool that went away for Inspect).
   */
  function syncAvailability(): void {
    const sugar = targets.some((t) => t.engine.model === 'sugarscape');
    const on = diseaseOn();
    buttons.forEach((b, i) => {
      const t = TOOLS[i][0];
      b.hidden = t !== 'inspect' && (!sugar || (DISEASE_TOOLS.includes(t) && !on));
    });
    if (tool !== 'inspect' && (!sugar || (!on && DISEASE_TOOLS.includes(tool)))) choose('inspect');
    else refreshPicker(true);
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/ui/format.ts web/src/ui/format.test.ts web/src/ui/inspect-panel.ts web/src/ui/tools.ts
git commit -m "Inspect Schelling and Ring World sites; offer only Inspect outside the sugarscape" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 10: Experiments, the CLI and the `schelling-tipping` sweep

*Needs judgement: the sweep's settings come from Decision 15's measurement; re-run it if the model changed.* Browser (controller): Experiments lists **Schelling tipping: segregation against preference** last among the built-ins; running it (65 runs, a second or two) draws a staircase from ≈ 0.50 at 0 to ≈ 0.93 at 0.55–0.6 matching the description's figures, and its exports work; with `vi-4-schelling-25` in the playground, **From current world** defaults to x `population` `1000:2400:200`, statistic `segregation` (final), 200 ticks, the path box suggests `width`, `height`, `population`, `preference.min`, … and never `model`, and the statistic menu lists Schelling's series; on `vi-8-ring-world` it defaults to x `agents`, statistic `flocks`; a sweep over a Schelling base shares as a `#x=` link and reopens; `ii-2-unit`'s defaults are unchanged.

**Files:**
- Create: `sweeps/schelling-tipping.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-core/tests/book.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`, `web/src/experiments/form.ts`, `web/src/experiments/view.ts`
- Test: `web/src/experiments/form.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: sweeps over `ModelConfig` (Task 1), `schelling`'s `preference.min`/`.max` paths and `segregation` series (Task 2), WASM `sweep_points`/`run_point`/`config_series_names` (Task 4), `Engine.model` (Task 5).
- Produces: built-in sweep id `schelling-tipping` (the sixth); `defaultForm(model: ModelKind = 'sugarscape'): SweepForm`.

- [ ] **Step 1: Write the failing tests**

`crates/sugarscape-core/src/sweep.rs`'s `builtin_sweeps_parse_and_validate`: the id list ends `"bargaining-rules",\n                "schelling-tipping"`. `crates/sugarscape-cli/tests/cli.rs`'s `presets_and_sweeps_are_listed`: add `"schelling-tipping",` after `"bargaining-rules",`; and append
```rust
#[test]
fn every_model_runs_from_a_preset_or_its_written_config() {
    let out = sugarscape(&["presets"]);
    let list = stdout(&out);
    for line in [
        "vi-4-schelling-25\tAnimation VI-4\t",
        "vi-9-ring-megagroup\tAnimation VI-9\t",
    ] {
        assert!(list.lines().any(|l| l.starts_with(line)), "{list}");
    }
    let dir = scratch("models");
    // tests/golden.rs, MODEL_GOLDEN: each preset after 200 ticks from seed 1.
    for (id, golden, header) in [
        (
            "vi-4-schelling-25",
            "0x7a7072c3433f5f6f",
            "tick,unsatisfied,segregation,",
        ),
        (
            "vi-8-ring-world",
            "0x1c341361c466db90",
            "tick,flocks,mean_flock,",
        ),
    ] {
        let (series, agents, config) = (
            dir.join(format!("{id}-series.csv")),
            dir.join(format!("{id}-agents.csv")),
            dir.join(format!("{id}-config.json")),
        );
        let out = sugarscape(&[
            "run",
            "--preset",
            id,
            "--ticks",
            "200",
            "--fingerprint",
            "--series-csv",
            path(&series),
            "--agents-csv",
            path(&agents),
            "--config-out",
            path(&config),
        ]);
        assert!(out.status.success(), "{}", stderr(&out));
        assert_eq!(stdout(&out), format!("{golden}\n"));
        assert!(read(&series).starts_with(header), "{id}");
        assert_eq!(read(&series).lines().count(), 202);
        assert!(read(&agents).starts_with("id,"));
        let again = sugarscape(&[
            "run",
            "--config",
            path(&config),
            "--ticks",
            "200",
            "--fingerprint",
        ]);
        assert_eq!(
            stdout(&again),
            format!("{golden}\n"),
            "{id}: the written config reproduces the run"
        );
    }
}

#[test]
fn the_tipping_sweep_runs_on_the_command_line() {
    let out = sugarscape(&[
        "sweep",
        "--builtin",
        "schelling-tipping",
        "--quiet",
        "--seeds",
        "1",
        "--jobs",
        "2",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let result: sugarscape_core::sweep::SweepResult = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(result.runs.len(), 13);
}
```
`crates/sugarscape-wasm/tests/web.rs`'s `builtins_and_series_names_are_listed`: the id list ends `"bargaining-rules",\n            "schelling-tipping"`.
`crates/sugarscape-core/tests/book.rs` — append:
```rust
#[test]
#[ignore]
fn schelling_tipping_rises_in_steps() {
    // At `sweeps/schelling-tipping.json`'s settings (seeds 1–5): mean final
    // segregation 0.498 color-blind rising in steps to 0.934 at 60%. The
    // test needs the rise (the measured 0.436, rounded down to a multiple of
    // 0.05) and no step down between neighboring preferences.
    const RISE: f64 = 0.40;
    let means = cell_means(&run_builtin("schelling-tipping"));
    let line = &means[0];
    assert!(line[line.len() - 1] - line[0] >= RISE, "{line:?}");
    for pair in line.windows(2) {
        assert!(pair[1] >= pair[0] - 1e-9, "segregation fell: {line:?}");
    }
}
```
`web/src/experiments/form.test.ts`: add `defaultForm,` to the `./form` import (first) and `SchellingConfig` to the types import (`import type { Config, SchellingConfig } from '../types';`), and append
```ts

describe('sweeps over other models', () => {
  it('start from an axis and a statistic the base’s model has', () => {
    expect(defaultForm().x.path).toBe('vision.max');
    expect(defaultForm('schelling')).toMatchObject({ x: { path: 'population' }, metric: { kind: 'final', series: 'segregation' } });
    expect(defaultForm('ring')).toMatchObject({ x: { path: 'agents' }, metric: { series: 'flocks' } });
  });

  it('suggest a Schelling config’s own paths, never its model tag', () => {
    const schelling: SchellingConfig = {
      model: 'schelling',
      width: 50,
      height: 50,
      population: 2000,
      preference: { min: 0.25, max: 0.25 },
      residence: { enabled: false, min: 80, max: 100 },
    };
    expect(numericPaths(schelling)).toEqual([
      'width',
      'height',
      'population',
      'preference.min',
      'preference.max',
      'residence.enabled',
      'residence.min',
      'residence.max',
    ]);
    const { sweep } = formToSweep(defaultForm('schelling'), { config: schelling });
    expect(sweep!.base).toEqual({ config: schelling });
  });
});
```
`web/src/determinism.test.ts`: the WASM import becomes `import { config_series_names, initSync, presets_json, run_point, sweep_points } from './wasm-pkg/sugarscape.js';`; append
```ts
describe('sweeps over other models', () => {
  const spec = {
    name: 'Schelling population',
    base: { preset: 'vi-4-schelling-25' },
    x: { path: 'population', values: [500, 1500] },
    seeds: { from: 1, count: 1 },
    ticks: 10,
    metric: { kind: 'final', series: 'segregation' },
  };

  it('run over a Schelling base, its paths and statistics checked against Schelling', () => {
    const json = JSON.stringify(spec);
    expect(JSON.parse(sweep_points(json))).toHaveLength(2);
    const run = JSON.parse(run_point(json, 1)) as { value: number };
    expect(run.value).toBeGreaterThan(0.5);
    const wrongPath = JSON.stringify({ ...spec, x: { path: 'vision.max', values: [1] } });
    expect(() => sweep_points(wrongPath)).toThrow('unknown field vision.max');
    const wrongSeries = JSON.stringify({ ...spec, metric: { kind: 'final', series: 'gini' } });
    expect(() => sweep_points(wrongSeries)).toThrow('no statistics series');
  });
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core --lib builtin && cargo test -p sugarscape-cli`
Expected: FAIL — no built-in `schelling-tipping` (the Schelling CLI test passes already: Task 1's CLI runs any model).
Run: `(cd web && npm run build)`
Expected: FAIL in `tsc` — `defaultForm` takes no argument.

- [ ] **Step 3: Implement**

Create `sweeps/schelling-tipping.json` (Decision 15):
```json
{
  "name": "Schelling tipping: segregation against preference",
  "description": "Animation VI-4's society (2000 agents on 50×50, no maximum residence) with every agent wanting the same share of like-colored neighbors, from 0 (color-blind) to 60%: the book's question \"How little racism is enough to 'tip' a society into this segregated pattern?\" With four von Neumann neighbors only the shares 1/4, 1/3, 1/2 and 2/3 can matter, so segregation rises in steps. Measured (release, seeds 1–5, recorded 2026-09-25): mean final segregation 0.498 color-blind, 0.629 from 5% to 25%, 0.731 at 30%, 0.830 from 35% to 50% and 0.934 at 55–60%; every run was quiet (nobody moved) by t = 7, and 50 ticks is the smallest multiple of 50 at least five times that.",
  "base": { "preset": "vi-4-schelling-25" },
  "x": {
    "label": "Like neighbors wanted (fixed preference)",
    "values": [
      { "at": 0, "set": { "preference.min": 0, "preference.max": 0 } },
      { "at": 0.05, "set": { "preference.min": 0.05, "preference.max": 0.05 } },
      { "at": 0.1, "set": { "preference.min": 0.1, "preference.max": 0.1 } },
      { "at": 0.15, "set": { "preference.min": 0.15, "preference.max": 0.15 } },
      { "at": 0.2, "set": { "preference.min": 0.2, "preference.max": 0.2 } },
      { "at": 0.25, "set": { "preference.min": 0.25, "preference.max": 0.25 } },
      { "at": 0.3, "set": { "preference.min": 0.3, "preference.max": 0.3 } },
      { "at": 0.35, "set": { "preference.min": 0.35, "preference.max": 0.35 } },
      { "at": 0.4, "set": { "preference.min": 0.4, "preference.max": 0.4 } },
      { "at": 0.45, "set": { "preference.min": 0.45, "preference.max": 0.45 } },
      { "at": 0.5, "set": { "preference.min": 0.5, "preference.max": 0.5 } },
      { "at": 0.55, "set": { "preference.min": 0.55, "preference.max": 0.55 } },
      { "at": 0.6, "set": { "preference.min": 0.6, "preference.max": 0.6 } }
    ]
  },
  "seeds": { "from": 1, "count": 5 },
  "ticks": 50,
  "metric": { "kind": "final", "series": "segregation" }
}
```
`crates/sugarscape-core/src/sweep.rs`: `const BUILTINS: [Builtin; 5] = [` → `[Builtin; 6]`, and after the `bargaining-rules` entry add
```rust
    Builtin {
        id: "schelling-tipping",
        json: include_str!("../../../sweeps/schelling-tipping.json"),
    },
```
`web/src/experiments/form.ts`: the types import becomes `import type { FieldError, ModelConfig, ModelKind } from '../types';` and `defaultForm` becomes (Decision 14)
```ts
/** A new sweep's form for a base of `model`: an axis and a statistic that model has (Decision 14). */
export function defaultForm(model: ModelKind = 'sugarscape'): SweepForm {
  const form: SweepForm = {
    name: 'Untitled sweep',
    x: { path: 'vision.max', values: '1:6:1' },
    series: null,
    seeds: 3,
    ticks: 500,
    metric: { kind: 'window_mean', series: 'population', from: 400, to: null, every: 25 },
  };
  if (model === 'schelling') {
    return { ...form, x: { path: 'population', values: '1000:2400:200' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'segregation' } };
  }
  if (model === 'ring') {
    return { ...form, x: { path: 'agents', values: '10:70:10' }, metric: { ...form.metric, series: 'flocks' } };
  }
  return form;
}
```
`web/src/experiments/view.ts`: in `pick('current')`, `this.formView(defaultForm(), base, note)` → `this.formView(defaultForm(this.engine.model), base, note)`. (The path suggestions and statistic list already follow the base's config: `numericPaths` walks any object and skips the `model` string; `config_series_names` reads any model.)

- [ ] **Step 4: Measure the sweep and run the tests**

Run: `cargo run --release -p sugarscape-cli -- sweep --builtin schelling-tipping --quiet --summary-csv target/tipping.csv > /dev/null && cat target/tipping.csv`
Expected: 13 rows; mean 0.4983 at 0, 0.6295 from 0.05 to 0.25, 0.7309 at 0.3, 0.8301 from 0.35 to 0.5, 0.9337 at 0.55 and 0.6; every sd ≤ 0.009. (If the Schelling model changed, re-measure the first quiet tick per preference with `vi-4-schelling-25` and Decision 15's rule, and update the file's description and ticks.)
Run: `cargo test --workspace && cargo test -p sugarscape-core --release --test book schelling_tipping -- --ignored && wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS.
Run: `(cd web && npm run build && npm test)`
Expected: PASS.

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add sweeps/schelling-tipping.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-core/tests/book.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs web/src/experiments/form.ts web/src/experiments/form.test.ts web/src/experiments/view.ts web/src/determinism.test.ts
git commit -m "Add the schelling-tipping sweep; sweep and run every model from Experiments and the CLI" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 11: Recording the ring view

*Mechanical (full code).* Browser (controller): on `vi-8-ring-world` at 5×, ● Record → WebM (and GIF) for a few seconds: each frame shows the ring (left) beside the space–time diagram (right), 904 × 450 px, the tick stamp bottom left; in Compare on Ring World the frame shows A's ring and diagram then B's, tagged "A" and "B" on the rings; `vi-4-schelling-25` records its grid alone; `ii-2-unit` and sugarscape Compare recordings are unchanged.

**Files:**
- Modify: `web/src/recording/frames.ts`, `web/src/main.ts`
- Test: `web/src/recording/frames.test.ts`

**Interfaces:**
- Consumes: `Playground.ringView`, `CompareView.{gridB, ringB, b}`, `Engine.{model, size}` (Tasks 5, 7).
- Produces: `frames.ts`: `interface WorldView<C> { canvas: C; cells: () => { width: number; height: number }; label?: string }`, `worldViews<C>(world: { model: ModelKind; grid: C; ring: C; size: () => { width: number; height: number } }, label?: string): WorldView<C>[]`.

- [ ] **Step 1: Write the failing test**

`web/src/recording/frames.test.ts`: add `worldViews,` to the `./frames` import (after `Stopwatch,`) and append
```ts

describe('worldViews', () => {
  const size = () => ({ width: 150, height: 150 });
  it('records a grid alone, or Ring World’s ring beside its space–time diagram', () => {
    const sugar = worldViews({ model: 'sugarscape', grid: 'grid', ring: 'ring', size: () => ({ width: 50, height: 50 }) }, 'A');
    expect(sugar.map((v) => [v.canvas, v.cells(), v.label])).toEqual([['grid', { width: 50, height: 50 }, 'A']]);
    const ring = worldViews({ model: 'ring', grid: 'grid', ring: 'ring', size }, 'B');
    expect(ring.map((v) => [v.canvas, v.cells(), v.label])).toEqual([
      ['ring', { width: 150, height: 150 }, 'B'],
      ['grid', { width: 150, height: 150 }, undefined],
    ]);
    // Side by side at 3 px a cell, so the frame stays within 1080 px.
    expect(frameLayout(ring.map((v) => v.cells()))).toMatchObject({ width: 904, height: 450, scale: 3 });
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `(cd web && npx vitest run src/recording/frames.test.ts)`
Expected: FAIL — `worldViews` is not exported.

- [ ] **Step 3: Implement**

`web/src/recording/frames.ts`: after the file's first comment line add `import type { ModelKind } from '../types';`, and before `/** The first video type the browser can record, and its file extension. */` add (Decision 16)
```ts
/** A view a recording draws: its canvas, its size in cells, and the world's tag in Compare. */
export interface WorldView<C> { canvas: C; cells: () => { width: number; height: number }; label?: string }

/**
 * The views a recording shows for one world (milestone 9's Decision 16): its grid; for Ring
 * World its ring first (a square as tall as the space–time diagram) and the diagram beside it.
 * `label` tags the world's first view.
 */
export function worldViews<C>(
  world: { model: ModelKind; grid: C; ring: C; size: () => { width: number; height: number } },
  label?: string,
): WorldView<C>[] {
  const grid: WorldView<C> = { canvas: world.grid, cells: world.size };
  if (world.model !== 'ring') return [{ ...grid, label }];
  const side = () => world.size().height;
  return [{ canvas: world.ring, cells: () => ({ width: side(), height: side() }), label }, grid];
}

```
`web/src/main.ts`: add `import { worldViews } from './recording/frames';` after the `compare-presets` import, and replace the record control's `grids` (with its comment) by
```ts
    // In Compare each frame shows both worlds side by side, tagged "A" and "B"; Ring World records
    // its ring beside its space–time diagram.
    grids: () => {
      const c = compare;
      const a = { model: engine.model, grid: grid.canvas, ring: ringView.canvas, size: () => engine.size() };
      if (!c) return worldViews(a);
      const b = { model: c.b.model, grid: c.gridB.canvas, ring: c.ringB.canvas, size: () => c.b.size() };
      return [...worldViews(a, 'A'), ...worldViews(b, 'B')];
    },
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `(cd web && npm run build && npm test)`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/recording/frames.ts web/src/recording/frames.test.ts web/src/main.ts
git commit -m "Record Ring World's ring beside its space–time diagram" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 12: README, roadmap and full verification

*Needs judgement (prose).* Browser (controller): the full pass below.

**Files:**
- Modify: `README.md`, `docs/roadmap.md`

- [ ] **Step 1: README**

In `README.md`:
- After the Chapter VI paragraph (the one ending "…are views only: they never change a run and are not exported or shared."), add
  ```markdown
  Chapter VI's other artificial societies — the book's Schelling segregation variant and Ring World —
  run as their own model kinds beside the sugarscape (see [Other artificial societies](#other-artificial-societies)).
  ```
- Before `## Experiments`, add
  ```markdown
  ## Other artificial societies

  The presets menu groups its presets by model: **Sugarscape**, **Schelling** and **Ring World**.
  Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
  (Max included), Share, Export, Record, Compare, Experiments and the CLI work the same for every
  model. A config without a `model` key is a sugarscape config, so every older config, link, session
  file and sweep reads as before. The other models' Rules panels are built from their parameter
  schemas (each section says whether its fields rebuild the world or apply as it runs, and live
  changes replay from links); they have no editing tools, overlays, trails or Credit tab. Compare
  pairs two worlds of one model: choosing another model's preset while comparing leaves Compare
  keeping A. See `docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md`.

  ### Schelling segregation (animations VI-4 to VI-7)

  The book's variant of Schelling's model: 2 000 Red and Blue agents on a 50 × 50 torus, each
  wanting at least a share of its von Neumann neighbors to be its own color (an agent with no
  neighbors is satisfied). Agents act in random order; an unsatisfied one moves to a site chosen at
  random among every empty site where it would be satisfied (its own site not counted as a
  neighbor), or stays. With a maximum residence an agent leaves when it reaches it, and a newcomer of
  random color takes a random site where it is satisfied. Segregation is the mean share of like
  neighbors (over agents with neighbors).

  - `vi-4-schelling-25`: 25 %. Nobody moves after 2–3 ticks; segregation rises from about 0.50 to 0.63.
  - `vi-5-schelling-25-residence`: 25 %, residence 80–100 ticks. It never settles; segregation
    climbs to about 0.76, above VI-4's (the book calls the two comparable).
  - `vi-6-schelling-50-residence`: 50 %: about 0.95.
  - `vi-7-schelling-mixed`: preferences 25–50 %: about 0.93, close to VI-6.

  Agents are drawn by **Colour**, **Satisfaction** (the unsatisfied in yellow) or **Preference**;
  the charts are Segregation, Unsatisfied, Moves and Red share; Inspect shows an agent's color,
  preference, alike neighbors and residence. The built-in sweep `schelling-tipping` asks the book's
  "how little racism is enough to tip a society": with every agent wanting the same share, from 0 to
  60 %, segregation rises in steps (about 0.50, 0.63, 0.73, 0.83 and 0.93), because with four
  neighbors only the shares 1/4, 1/3, 1/2 and 2/3 can matter.

  ### Ring World (animations VI-8 and VI-9)

  40 sugar harvesters with vision 15–30 on a ring of 150 sites (sugar 0–4, growing back 1 a tick)
  look only counterclockwise, move to the nearest richest empty site they see and eat it. Started
  scattered (`vi-8-ring-world`) they fall from 20–25 flocks of about 2 into 7–8 flocks of 5–6 that
  tread around the ring; started as one group of 40 (`vi-9-ring-megagroup`) they break up into as
  many. A flock is a run of agents at most one empty site apart (the book does not define one). The
  page draws the ring — sugar shaded, agents as blue dots, site 0 at the top — above a space–time
  diagram of the last 150 ticks (the current tick at the bottom), which the simulation keeps, so every
  tick shows even at Max. Capacity and growback apply to the running world. Charts: Flocks, Flock
  size (mean and largest) and Distance moved; recordings show the ring beside the diagram.
  ```
- In `## Experiments`'s built-in sweeps bullet, "`n-goods-carrying-capacity` and `bargaining-rules`" becomes "`n-goods-carrying-capacity`, `bargaining-rules`", and "(carrying capacity vs vision under the two price rules)." becomes "(carrying capacity vs vision under the two price rules) and `schelling-tipping` (segregation vs a fixed Schelling preference)."; at the end of the **From current world** bullet add "The path suggestions, statistics and starting axis follow the current world's model."
- In `## Recording`, after "in Compare both grids are recorded side by side." add " Ring World records its ring beside its space–time diagram."
- In `## Command line`, add the example line `    sugarscape run --preset vi-8-ring-world --ticks 500 --series-csv ring.csv       # any model` after the `--config my-config.json` line, and make the first sentence after the examples "`run` runs a preset or config of any model; it defaults to seed 1 and 1000 ticks; …".

- [ ] **Step 2: Roadmap**

In `docs/roadmap.md`, replace the whole `## Next: Other artificial societies` section with
```markdown
## Milestone 9: Other artificial societies (done)

Chapter VI's Schelling segregation variant (VI-4 to VI-7, with random acceptable relocation and a
maximum residence) and Ring World (VI-8, VI-9, with its ring view and space–time diagram) as model
kinds beside the sugarscape, on a model-tagged config and a `Model` trait that later models can
reuse: every speed, links, Compare, recording, Experiments (with the measured `schelling-tipping`
sweep) and the CLI. Sugarscape runs, configs and links are unchanged. See
`docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md`.
```
and in `## Experiments and science` replace "the flocking / group-formation aside (Animation VI-8) moves to *Other artificial societies*." with "the flocking / group-formation aside (Animation VI-8) is Ring World (Milestone 9)."

- [ ] **Step 3: Full verification**

Run each; every one must pass:
```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test book -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
git diff main -- crates/sugarscape-core/tests/fixtures crates/sugarscape-core/tests/legacy.rs crates/sugarscape-core/tests/invariants.rs
git diff main -- crates/sugarscape-core/tests/golden.rs | grep '^-' | grep -v '^---'
```
Expected: the last two print nothing except, for golden.rs, the replaced `print_golden` doc line `-/// Prints \`GOLDEN\` entries: …` (no `GOLDEN` entry removed or changed).

Browser (controller), the full pass on `/?debug`: every scenario of Tasks 6–11; then the Max-speed performance check — `vi-4-schelling-25` and `vi-6-schelling-50-residence` at Max for 10 s (planning: thousands of ticks per second; the tab stays responsive), a 200 × 200 / 32 000-agent Schelling world (Setup: Width 200, Height 200, Agents 32000, Preference 0.5–0.5, residence on) at Max (planning's core measurement: about 3 ms a tick, so well over 100 ticks per second), and `vi-8-ring-world` at Max (planning: over 100 000 ticks a second; the diagram shows diagonal bands of flocks); Compare on Schelling (`vi-5` vs `vi-6` via "Rules for: B") shows B's segregation line dashed above A's; share links of a Schelling world after a reset-requiring change and of a Ring World with a live Growback change reproduce their fingerprints (`window.sugarscape.engine.fingerprint()`); and every existing scenario from the milestone 8 plan's full pass (presets, painting, overlays, Lineage, histograms, disease tools, credit tab, VI-2 vs VI-3 Compare, session files, the legacy share link, recording, Experiments' five older built-ins).

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md
git commit -m "Document the Schelling and Ring World models and mark milestone 9 done" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

## Self-review (planning)

- **Spec coverage:** config union and untagged = sugarscape (Tasks 1–3, 5; Decision 1); `Model` trait, `ModelWorld`, presets carrying their model (1–3; Decisions 2, 3); WASM enum dispatch with empty sugarscape-only calls (4; Decision 5); schema and schema-driven Rules panel (2, 3, 6; Decision 8); Schelling setup, satisfaction, step, residence, statistics, render modes, presets (2; Decision 9); Ring World setup, step, flocks, statistics, presets (3; Decision 10); page presets grouped by model and switching models (6); views — Schelling grid, ring circle from snapshot data sent only for Ring World, host-rendered space–time frame (5, 7; Decisions 7, 12); Inspect per model and hidden sugarscape UI (9; Decision 13); charts per model (8; Decision 13); replay and links with model-tagged configs, `setConfig` for live fields only (5, 6); Compare same-model rule (6; Decision 11); Experiments/CLI and the measured `schelling-tipping` (1, 10; Decisions 14, 15); recording both models (11; Decision 16); golden entries for the six presets (2, 3; Decision 18); every listed core unit test (2, 3), book-style tests with measured thresholds (2, 3, 10; Decision 17), config tests (1–3), web tests (schema panel 6, chart visibility 8, ring rendering helpers 7, Compare same-model 6, old links 5, a sweep over a Schelling base 10); docs (12).
- **Placeholders:** none; every step names its files and gives the code or the exact edit.
- **Type consistency:** checked against the Global Constraints' name list — `Engine.sugar`/`model`/`ring`/`schemas`, `applyModelConfig`/`resetModelWith`, `RingState`, `COLOR_MODES`, `presetGroups`, `MODEL_CHARTS`, `worldViews`, `ModelWorld::ring`, `wrong_model`, `check_schema` appear with the same signatures in every task that uses them.
