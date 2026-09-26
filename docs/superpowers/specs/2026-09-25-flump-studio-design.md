# Flump Studio — short explainer videos from the engine — Design

**Date:** 2026-09-25
**Builds on:** the milestone specs in `docs/superpowers/specs/`; the engine, presets and CLI are unchanged except for the additions below.

## Goal

A series of 8–10 short (30–75 s, "commercial length") explainer videos in the spirit of Primer: cute
characters following simple rules on screen, and a social-science phenomenon emerging from them. The
characters are **Flumps**; the first episode (the pilot) is the sugarscape itself. The videos are for
Bluesky (and LinkedIn), autoplaying muted, so the narration is on-screen captions.

This spec covers three parts, built in order: the **data bridge** (a CLI `shot` command that dumps a
run frame by frame), the **Blender studio** (`studio/`, which turns dumps into rendered beats and cuts
them into a video) and the **pilot episode**. Later episodes each get their own short spec.

## Non-negotiable constraints

- **Every Flump behavior on screen is a real engine run.** Close-ups use small configs of the same model
  and agents placed with the existing `place_agent`; nothing a Flump does is hand-animated. Diagrams
  that explain a rule (sight lines, meters) are drawn from the dump's data, not invented.
- **Every caption that states a result is measured.** A claim such as "the survivors see farther" is
  checked over 20 seeds (as the survey does) before it is rendered; if it does not hold, the caption
  changes, not the data.
- **The engine and playground are unchanged.** Every golden entry, legacy fixture and existing CLI
  invocation behaves as before. `studio/` is not part of the deployed site.
- **Reproducible.** A beat is a function of its shot file (config, seed, placements) and its beat
  entry; rerunning the build gives the same frames.
- **Original character design.** Flumps are our own design, not Primer's blobs.

## Aesthetic

Amigurumi meets Kirby: a round crocheted blob with four nubbin limbs, glossy black eyes with a bright
specular highlight, blush dots. Knit-yarn material (sheen, fine knit bump, a visible stitch seam). The
landscape is a felt board; sugar is small translucent glossy gumdrops that swell as sugar grows back.
Warm tabletop lighting: a handmade toy set. Captions in a rounded open-licence font (Baloo 2 or
Nunito). 16:9, 1920 × 1080, 30 fps.

## Part 1: the data bridge (Rust)

### Shot files

One JSON file per beat:

```json
{
  "preset": "ii-2-unit",
  "seed": 7,
  "ticks": 300,
  "set": { "width": 12, "height": 12, "population": 0 },
  "place": [ { "tick": 0, "x": 3, "y": 4, "vision": 6, "metabolism": 1, "sugar": 10 } ]
}
```

- Exactly one of `preset` or `config` (a full config, as `--config` takes).
- `set`: optional config-path overrides (e.g. `"goods.0.map"`), applied with `Config::with_path` in
  sorted key order and validated once at the end.
- `empty`: optional, default false: start every site with no sugar (sites otherwise start full);
  growback then runs by its rule.
- `place`: agents placed before the given tick runs, through `World::place_agent` with
  `AgentOverrides` (vision, metabolism, sugar; sex and tribe as the struct allows). A placement on an
  occupied site or outside the grid is an error.
- Sugarscape model only for now; any other model kind is rejected (exit code 2, `model: …`), and the
  format leaves room to add models later.
- Errors print as `field: message`, one per line, with exit code 2, like `run`.

### Command

`sugarscape shot beat.json --out beat.frames.json` (stdout without `--out`).

### Frame dump

One JSON document:

- **Header:** `format` (`1`), `model`, `seed`, `ticks`, `width`, `height`, `capacity` (flat, per site,
  row-major, good 0), `placed` (the placed agents' ids, in `place` order), and `config` (the
  resolved config, so the dump reproduces itself).
- **`frames`:** ticks + 1 entries, tick 0 (after placements at tick 0) first. Each has:
  - `tick`
  - `agents`: `[[id, x, y, sugar, age, vision, metabolism], …]`, sorted by id
  - `sugar`: the level at every site (flat, row-major, good 0)
  - `deaths`: `[[id, "starvation" | "old_age" | "combat"], …]` from `TickEvents::deaths` (`DeathCause`, snake case)
  - `born`: ids that first appear in this frame (replacements and placements)
- **`stats`:** the run's per-tick series by name (the same series `--series-csv` writes).

Trades, infections, loans and tags are left out until an episode needs them.

### Code

- `crates/sugarscape-core/src/frames.rs`: `Shot` (deserialize and validate), `run_shot(&Shot) ->
  Result<FrameDump, Vec<FieldError>>`, and `FrameDump`'s serialization.
- `crates/sugarscape-cli`: the `shot` subcommand.

### Tests

- The same shot file gives byte-identical output.
- `frames.len() == ticks + 1`.
- An id is present in every frame from its first appearance until the frame of its death event and
  never after; every death's id was present in the previous frame.
- A placed agent appears at its site with its given traits in the frame of its placement tick.
- Frame 0's `sugar` equals `capacity` for `ii-2-unit` (sites start full).
- Rejections: both or neither of `preset`/`config`, an unknown path in `set`, a placement off the grid
  or on an occupied site, a non-sugarscape model.

## Part 2: the Blender studio (Python, `studio/`)

Targets **Blender 5.2 LTS** (installed: 5.2.2). Only Blender's bundled Python and modules inside
Blender; stdlib `unittest` for the pure-Python tests outside it (pytest is not installed).

Animation is not keyframed: one frame-change handler sets every object from pure functions of the
frame (Blender 5.2 has no `Action.fcurves`, and the handler also runs at motion blur's subframes).

### Modules

- `flump.py`: builds a Flump from primitives (no armature): a subdivided round body, four nubbin
  limbs, eyes, blush. Squash, stretch, droop and blinks are scales on the Flump's parts, computed per
  frame. Canned motions:
  **spawn** (pop and land), **hop** (arc, stretch in flight, squash on landing), **eat**, **look** (eyes
  turn toward a target), **poof** (collapse into a burst of felt-fluff particles).
- `materials.py`: knit yarn, felt, gumdrop, and the lighting rig.
- `board.py`: the felt board from a dump's size and capacity; sugar as one geometry-nodes object
  instancing a gumdrop per site, scaled by the site's level for the current frame (levels set from the
  dump on frame change), so a 50 × 50 board stays one object.
- `dump.py`: loads a frame dump; pure Python, tested outside Blender.
- `animate.py`: maps ticks to frames (each beat sets ticks per second); plans keyframes from each
  agent's track (a hop per move, eased between ticks), a poof per death, a spawn per birth; sight
  lines along the four lattice directions for `vision` sites. The planning is pure Python and tested;
  applying the plan to Blender objects is a thin layer.
- `camera.py`: named moves (push-in, pull-back, orbit, hold) with ease-in/ease-out.
- `overlays.py`: felt-block stat displays fed by the dump's `stats` (histogram bars, a dial or bar for
  a mean).
- `cut.py`: lays each beat's caption over it (ffmpeg `overlay`, faded in and out), joins the beats
  with `xfade` cross-dissolves and writes H.264 MP4 (`yuv420p`). The installed ffmpeg has no
  `drawtext`, so captions are rendered by Blender, once per beat, as transparent PNGs; kept out of
  the beat's scene, they are never hidden by scenery or blurred by depth of field, and changing one
  needs only a re-cut.
- `render.py`: the entry point run inside Blender: `blender -b -P studio/render.py -- <episode> <beat>
  [--preview] [--still F | --caption]`. Builds the beat's scene, saves `<beat>.blend` beside the
  render so it can be opened and tweaked by hand, and renders it (or one frame, or the caption).
- `build` (a script): for an episode, runs `sugarscape shot` for each beat, renders each beat and runs
  the cut. `--preview` renders 960 × 540 with low sampling; `--beat N` renders one beat.

### Episode layout

`studio/episodes/<episode>/`:

- `shots/*.json`: the bridge inputs.
- `beats.py`: an ordered list of beats, each with its shot, caption, camera move, duration (seconds)
  and ticks per second, plus any beat-specific staging (which agent to follow, which overlay).

Build outputs go to `studio/out/<episode>/` (ignored by git): dumps, `.blend` files, beat renders and
the final MP4.

### Rendering

Eevee, 1920 × 1080, 30 fps, motion blur on (shutter 0.3); depth of field in close-ups only. Preview:
960 × 540, low samples. The handler edits scene data while rendering, so the interface is locked
during renders, and objects are hidden by shrinking them away under the board, never by toggling
`hide_render`: both of those hung or crashed Eevee mid-animation.

### Tests

- `unittest` for `dump.py` and the planning in `animate.py` and `camera.py` (tick-to-frame mapping,
  a hop per move, a poof at a death's frame, easing endpoints).
- Smoke test: each beat of the pilot renders one preview frame headless without error.
- The final MP4's frame count equals the beats' total duration × 30 (less the dissolve overlaps).

## Part 3: the pilot episode, "Sugarscape" (~60–75 s)

| # | Caption | On screen | Source |
|---|---|---|---|
| 1 | *This is sugar.* | Close-up: one gumdrop on felt grows back a notch a tick to its capacity (rule G₁). | tiny config, one site's view |
| 2 | *Sugar grows in some places… but not others.* | Pull back across the 50 × 50 felt: two sugar hills, bare plains. | `ii-2-unit`'s landscape |
| 3 | *This is a Flump.* | A Flump pops in, squashes on landing, blinks. | small config, one placed agent |
| 4 | *Flumps eat sugar. Without it, they die.* | It eats (gumdrop gone, a belly meter fills), the meter drains each tick (metabolism); a second Flump on bare felt droops and poofs. | small config, two placed agents |
| 5 | *A Flump can see a little way…* | Sight lines reach along the four lattice directions; it looks along each and hops to the nearest best site (rule M). | small config |
| 6 | *…and every Flump is different.* | Two Flumps side by side: far-sighted and frugal vs short-sighted and hungry, labelled. | small config, two placed agents |
| 7 | *Now: 400 Flumps.* | Zoom out; 400 random Flumps drop in; many poof early; survivors crowd the hills. | `ii-2-unit` |
| 8 | *Nobody told them to gather there.* | Hold on the settled crowd; the survivors' mean vision rises and mean metabolism falls (selection), as felt dials. | same run, `stats` |
| 9 | *Same rules for everyone. So why are some rich?* | Replacement on; a sugar-cube stack over each Flump grows; a few tower over the rest; a histogram builds beside the board. | `ii-5-wealth` |
| 10 | End card | "Sugarscape — Epstein & Axtell, 1996" and the playground link | — |

Before its caption is final, each claim is measured over 20 seeds: beat 7's early die-off and hill
crowding, beat 8's rise in mean vision and fall in mean metabolism, beat 9's skewed wealth (Gini and
histogram shape). The seed rendered is one whose run is typical of those 20, not the most dramatic.

Beats 1–6 use small configs (about 12 × 12, sugar from a map or painted capacities via `config`) with
agents placed by `place`; exact configs are chosen while building, subject to the constraints above.

## Out of scope

- Later episodes (each gets its own spec; candidates: Schelling, seasonal migration, pollution, tribes,
  trade, disease, Ring World, civil violence, tag cooperation, spatial games, Anasazi).
- Frame dumps for other model kinds, trades, infections or tags.
- Audio.
- The three.js renderer considered earlier (superseded by Blender).
