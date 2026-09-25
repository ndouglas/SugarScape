# SugarScape Milestone 9 — Other artificial societies: Schelling and Ring World — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–8 specs in `docs/superpowers/specs/`; all remain binding where not changed here.
**Source text:** Epstein & Axtell, *Growing Artificial Societies*, Chapter VI, "Other Artificial Societies": "A Variant of Schelling's Segregation Model" (animations VI-4 to VI-7, notes 9–11) and "Ring World" (animations VI-8, VI-9, note 12).

## Goal

Add two new model kinds beside the sugarscape — the book's Schelling segregation variant and Ring World — as full citizens of the playground (worker engine, Max speed, replay and share links, Compare, recording, Experiments and the CLI), on an architecture that later models can reuse.

## Non-negotiable constraints

- **Sugarscape unchanged.** Every sugarscape golden entry and legacy fixture stays green and unedited; every existing config, share link, session file and sweep reads as before (a config without a model tag is a sugarscape config).
- **Faithful to the book** where it is specific (quoted below); where it is silent, the choice is stated in this spec.
- **One engine path.** The worker host, engine, replay, Compare, sweeps and CLI stay single-path over "a model with a config and named series"; no per-model forks of that machinery.
- **Deterministic.** Each model is a function of (config, seed); fingerprints per preset are pinned by golden entries.

## Architecture

- **Config:** a top-level tagged union keyed by `model`: `sugarscape` (default when absent; today's `Config`), `schelling` (`SchellingConfig`) or `ring` (`RingConfig`). Validation, normalization and `with_path` work per model.
- **Core:** a `Model` trait — `step(n)`, `tick()`, `fingerprint()`, `render(mode)`, `stats()` (a named snapshot), `series_names()`, `population()`, `schema()` — implemented by today's `World` and by new `SchellingWorld` (`schelling.rs`) and `RingWorld` (`ring.rs`). Presets carry their model; `presets()` lists all.
- **WASM:** the `Sim` holds one model (enum dispatch). Sugarscape-only calls (networks, Lorenz, inspect details, editing, trails, credit graph …) return empty/`null` for other models; the host only requests them when the model supports them.
- **Schema:** each new model exposes a parameter schema — `{ path, label, kind: integer | number | range | bool | choice, min, max, step, choices?, apply: live | reset, group }` — from which the page builds its Rules panel. Sugarscape keeps its bespoke panel.

## Schelling (animations VI-4 to VI-7)

- **Book:** "every agent is a member of one or another group (here either Red or Blue) and has a fixed preference for like-colored neighbors … a minimum percentage"; rule: compute the fraction of neighbors of its own color; if ≥ its preference it is satisfied, else it moves; "we use the von Neumann neighborhood … our agents simply select an acceptable site at random … ours is a torus". Setup: "a 50 × 50 lattice … with 2000 Red and Blue agents in approximately equal numbers"; VI-4 25 %; VI-5 25 % with "a randomly assigned maximum lifetime between 80 and 100 … replacing it with a new agent of random color … placed at a randomly selected position satisfying its preference" (lifetime read as residence, note 11); VI-6 50 %; VI-7 preferences "uniformly between 25 percent and 50 percent".
- **Config:** `width`, `height` (≥ 5), `population` (< width × height), `preference: { min, max }` (fractions 0–1; min = max for a fixed preference), `residence: { enabled, min, max }` (ticks).
- **Setup:** agents on random distinct sites, color Red or Blue with probability ½ each, preference uniform in [min, max] (continuous), residence uniform integer in [min, max] when enabled.
- **Satisfaction:** over the four von Neumann neighbor sites that are occupied; with no neighbors an agent is satisfied.
- **Step:** agents act in a random order. An unsatisfied agent collects every unoccupied site at which it would be satisfied (evaluated as if it stood there, not counting its current site as a neighbor), picks one uniformly at random and moves there; with none it stays. Then ages increase by 1; with residence on, an agent whose age reaches its maximum is removed and replaced by a new agent (random color, preference, residence) placed on a random unoccupied site that satisfies it, or a random unoccupied site if none does. Population is constant.
- **Statistics:** `unsatisfied` (share of agents unsatisfied), `segregation` (mean share of like neighbors over agents with at least one neighbor), `moves` (agents that moved this tick), `red_share`, `quiet` (1 when no agent moved this tick, else 0), `population`.
- **Render modes:** Color (Red/Blue), Satisfaction (unsatisfied agents highlighted), Preference (shade by threshold); empty sites dark.
- **Presets:** `vi-4-schelling-25`, `vi-5-schelling-25-residence`, `vi-6-schelling-50-residence`, `vi-7-schelling-mixed` (the book's settings above).

## Ring World (animations VI-8, VI-9)

- **Book:** "the landscape is a circle of sugar sites … agents search only in the counterclockwise direction … vision randomly chosen from some range (15 to 30) … Inspect all unoccupied sites within your vision, select the nearest site with maximum sugar, go there and eat the sugar. … 150 sites. Initially, the sugar level is distributed randomly between values of 0 and 4, and 40 agents are distributed randomly around the ring. … sugar grows back at unit rate to a capacity value, which is 4 … There is no death, birth, combat, cultural transmission, disease, or trade … Each agent moves once each time period; agents are called in random order." VI-9 starts "with all agents in one megagroup (of 40)".
- **Config:** `sites` (≥ 10), `agents` (< sites), `vision: { min, max }`, `capacity`, `growback`, `start: random | megagroup`.
- **Setup:** site sugar uniform integer in 0..=capacity; agents on random distinct sites (`random`) or on consecutive sites starting at a random site (`megagroup`); vision uniform integer in [min, max].
- **Step:** agents act in a random order. Counterclockwise is increasing site index (mod sites). An agent inspects sites at distances 1…vision; among the unoccupied ones it selects the nearest with maximum sugar (even if that maximum is 0), moves there and eats all its sugar; if all are occupied it stays. After all agents have acted, every site grows back `growback` up to `capacity`.
- **Flocks (not defined by the book; stated here):** agents sorted by ring position; a flock is a maximal run whose consecutive gaps (in sites, around the ring) are at most 2 — at most one empty site between neighbors.
- **Statistics:** `flocks` (number of flocks), `mean_flock`, `largest_flock`, `mean_distance` (sites moved per agent this tick), `population`.
- **Presets:** `vi-8-ring-world` (random start) and `vi-9-ring-megagroup`.

## Page

- **Choosing a model:** the presets menu groups presets by model (Sugarscape, Schelling, Ring World); choosing a preset of another model rebuilds the world as that model. Toolbar, speeds (incl. Max), Share, Export, Record and Compare behave the same for every model.
- **Rules panel:** for Schelling and Ring World, generated from the schema (grouped fields; reset-only fields rebuild the world, live fields apply to the running world and are logged as `setConfig`).
- **Views:** Schelling — the host-rendered grid. Ring World — the ring drawn as a circle of `sites` cells shaded by sugar with agents as dots (from snapshot data: sugar per site and agent positions, sent only for the ring model), and below it a space–time diagram (rows = the last 150 ticks, columns = sites; sugar shaded, agents marked), kept and rendered by the host as the frame so every tick appears even at Max.
- **Inspect:** Schelling — color, preference, satisfied, residence (age / maximum); Ring World — agent vision, or a site's sugar. Sugarscape's editing tools, overlays, Credit tab and trails are hidden for other models.
- **Charts:** a chart table per model on the existing machinery (downsampled history, quiet when paused, Compare overlay A solid / B dashed): Schelling — segregation, share unsatisfied, moves, Red share; Ring World — flocks, mean and largest flock, mean distance.

## Shared machinery

- **Replay and links:** new models have no editing tools, so their edit log holds only live `setConfig` entries; share links, compare links and session files carry the model-tagged config; old links default to sugarscape.
- **Compare:** both worlds are the same model; choosing another model's preset while comparing first leaves Compare keeping A.
- **Experiments / CLI:** sweep bases may be any model's preset; `set` paths and metric series are validated against that model. New built-in sweep **`schelling-tipping`**: base `vi-4-schelling-25`, x = preference (fixed) from 0 to 0.6, metric = final `segregation`; ticks, steps and seeds chosen by measurement and recorded in its description. `sugarscape presets | run | sweep` accept all models.
- **Recording:** Schelling records its grid; Ring World records the ring view and the space–time diagram side by side.

## Testing

- **Golden/legacy:** sugarscape entries and fixtures unchanged; new golden entries for the six new presets.
- **Core unit (Schelling):** satisfaction (no neighbors, ties at the threshold), acceptable-site evaluation excluding the current site, random choice among acceptable sites only, staying when none, residence replacement keeping the population and placing satisfied when possible, statistics.
- **Core unit (Ring):** counterclockwise wrap, nearest-maximum selection (including all-zero sugar), occupied sites skipped, staying when all occupied, eating, growback after all moves, megagroup start, flock counting across the wrap.
- **Book-style (`#[ignore]`, release):** VI-4 reaches a quiet tick with segregation well above its start; VI-6's late segregation exceeds VI-5's; VI-7's stays high (closer to VI-6 than VI-5); Ring World's random start forms several flocks; the megagroup splits into several flocks — thresholds measured over seeds 1–5 and recorded.
- **Config:** model tag round trip; untagged configs are sugarscape; validation per model; `with_path` per model.
- **Web (Vitest):** schema-driven panel (field kinds, live vs reset); per-model chart visibility; ring and space–time rendering helpers; Compare same-model rule; old share links open as sugarscape; a sweep over a Schelling base.
- **Browser (controller):** each new preset (views, charts, Inspect); `schelling-tipping` in Experiments; Compare on Schelling (e.g. 25 % vs 50 %); Ring World at Max (space–time diagram shows bands); recording both models; every existing scenario; the Max-speed performance check.

## Docs

README: a section per model and the tipping sweep. Roadmap: mark "Other artificial societies" done.
