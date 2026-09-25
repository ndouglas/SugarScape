# Civil Violence (Epstein 2002) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Epstein's civil violence model — Model I (rebellion against a central authority) and Model II (violence between two ethnic groups) — as a fifth model kind, with the paper's runs as presets, NetLogo *Rebellion*'s five departures as named switches, the planning dry run's Finding (the paper's stated arrest rule gives Model I no rebellion) shown by a sweep, and every claim measured over 20 seeds — in the worker engine at every speed, links, Compare, recording, Experiments, the CLI and the survey, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/civil/` — `math.rs` (a portable `exp`), `stats.rs` (the series and the paper's outburst bookkeeping), `config.rs` (parameters, quirks, schedule and ramps, validation, schema), `world.rs` (`CivilWorld`: agents and cops acting once a tick in one random order, jail, Model II's killing, cloning and death, frames, Inspect), `presets.rs` and `mod.rs` — plugged into milestone 9's `ModelConfig`/`Model`/`ModelWorld` as `civil`, reusing milestone 10's `Model::finished()` for "a group is gone". `Param` gains an optional `show_if`. The WASM `Sim` needs no new calls. On the page the model joins `models.ts`, `MODEL_CHARTS` (with a per-config `shown`), the schema panel (which learns `show_if` and lists the civil schedule), Inspect and Experiments.

**Tech Stack:** Rust core (`sugarscape-core`), `wasm-bindgen` (`sugarscape-wasm`), the `sugarscape` CLI (clap), TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-25-civil-violence-design.md` (binding, including its Finding). The milestone 1–10 specs stay binding where not changed, in particular `2026-09-25-other-artificial-societies-design.md` and `2026-09-25-anasazi-design.md`, whose plans' conventions this plan follows. Source: Epstein, "Modeling civil violence: An agent-based computational approach", *PNAS* 99 suppl. 3 (2002) 7243–7250; NetLogo *Rebellion* (Wilensky 2004), `Sample Models/Social Science/Rebellion.nlogox`.

## Global Constraints

- **Existing models unchanged.** Every existing `GOLDEN` and `MODEL_GOLDEN` entry in `crates/sugarscape-core/tests/golden.rs` and every legacy fixture stays green and **unedited** (`MODEL_GOLDEN` gains ten entries; nothing else in golden.rs changes). `tests/legacy.rs`, `tests/invariants.rs` and `tests/book.rs` are not edited. Every existing config, share link, session file and sweep reads and runs as before.
- **One engine path.** The civil model plugs into the `Model` trait, `ModelConfig`, `ModelWorld`, the WASM `Sim`, host, engine, schema panel, chart table, Compare, sweeps and CLI — no per-model copies of that machinery.
- **Faithful, and honest where the paper fails.** The config default is the paper's rule. Model I presets set `quirks.floor_ratio` and say why. Every preset and sweep description states what was measured, including what does not reproduce (Decision 14). Showing where a paper's stated rules fail to produce its results is part of this project's purpose.
- **Deterministic and portable.** A world is a function of (config, seed). Sample `u32` ranges, never `usize` (`gen_range(0..n as u32)`); iterate vectors, never a `HashMap`; never call the platform's `f64::exp`/`ln` in a rule — `civil::exp_neg` instead (Decision 3). Golden fingerprints must come out bit-identical from `wasm-pack test`.
- **Copy (verbatim):** model label **Civil Violence** (presets-menu optgroup); preset ids `cv-run-1-no-movement`, `cv-run-2-punctuated`, `cv-run-3-salami`, `cv-run-4-one-jump`, `cv-run-5-cop-reductions`, `cv-run-6-coexistence`, `cv-run-7-cleansing`, `cv-run-8-nasty-regime`, `cv-safe-havens`, `cv-netlogo`; Compare entries **Salami tactics vs one jump — Civil Violence (Compare)** (id `cv-salami-vs-jump`) and **Ethnic cleansing vs peacekeepers — Civil Violence (Compare)** (id `cv-cleansing-vs-peacekeepers`); color modes **Action**, **Grievance**, **Group**; schema groups **World**, **Agents**, **Cops**, **Jail**, **Population**, **Outbursts**, **NetLogo quirks**; chart titles **Actives, quiet and jailed**, **Legitimacy**, **Cops**, **Tension**, **Outbursts**, **Wait between outbursts**, **Activation per outburst**, **Groups**, **Killed**; built-in sweep ids `cv-ratio-rules`, `cv-peacekeeping`, `cv-jail-waits`; series `population, active, quiet, jailed, cops, legitimacy, mean_grievance, tension, outbursts, mean_wait, mean_activation, blue, green, killed, extinction`, in that order; the finished notice `A group has died out at t = 94 — Reset to run it again`.
- **Names are binding across tasks** (each task's Interfaces block repeats the ones it uses): core `civil::{exp_neg, CivilSnapshot, Outbursts, SERIES, CivilConfig, Variant, Vision, Jail, Quirks, Ramp, LIVE, RAMPABLE, is_live, schema, presets, CivilWorld, Citizen, Cop, Term, CivilMode, CivilInspection, CitizenView, CopView, SiteXy, sight, COP, GREEN}`, `Quirks::ALL`, `CivilConfig::{sites, count, agents, cops, validate, changes, number, set_number}`, `CivilWorld::{new, step, run, is_finished, inspect, agents, cops, grievance}`, `ModelKind::Civil`, `ModelConfig::Civil`, `ModelWorld::Civil`, `schema::{ShowIf, Param::show_if, Param::shown_if}`; survey `runner::{model_preset, model_after}`; web `CivilConfig`, `CivilQuirks`, `CivilRamp`, `CivilStats`, `CitizenView`, `CivilInspection`, `Param.show_if` (types.ts), `isCivilView` (models.ts), `ModelChart.shown`, `isEthnic` (series-data.ts), `paramShown` (schema-form.ts), `scheduleLines`, `citizenRows`, `shownCitizen` (civil.ts).
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn`; the commit commands below pass it as a second `-m`. Stage **only** the files named in the task (`git add <paths>`, never `-A`/`.`; never `.claude/`).
- Rust tasks finish with `cargo fmt --all && cargo clippy --all-targets -- -D warnings` before committing. Code below is shown `rustfmt`-formatted; run `cargo fmt` anyway. CI runs the newest stable clippy: after pushing, check `gh run list`.
- Web tasks run `(cd web && npm run build && npm test)`. The build regenerates `web/src/wasm-pkg` (gitignored) with `wasm-pack` and runs `tsc --noEmit` then `vite build`; Vitest imports that package (determinism.test.ts), so always build before testing. Single test files run with `(cd web && npx vitest run src/<file>.test.ts)`.
- **TypeScript:** `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`. Unused imports fail the build — each task lists its import changes. Never pass a possibly-null child to `replaceChildren`/`append`: use `h()`, which skips `null`/`false`. Test names use the typographic apostrophe (’) inside single-quoted strings.
- **Browser checks are the controller's**, not the implementer's. Each web task's "Browser (controller):" line lists the scenarios it affects; the controller runs its puppeteer pass on them (and the full list in Task 9). `?debug` exposes `window.sugarscape = { engine, compare }`.

## Review Focus

The five inputs or conditions the spec implies that most likely bite a person using this, and the test that pins each (added to the owning task):

1. **A crowded lattice** — agent and cop densities summing to 1, or Model II's clones filling every site: moving, releasing and adding cops must never panic, and a cop that has nowhere to go waits. Pinned in Task 2 by `a_full_lattice_takes_no_more_cops_until_a_site_frees` and `released_agents_go_anywhere_when_their_arrest_site_is_crowded_and_wait_when_full`.
2. **Live edits mid-run** — a vision change must rebuild what agents see; turning "Terms never end" off must not free agents already jailed for life; size, density and variant must rebuild the world. Pinned in Task 2 by `a_live_vision_change_rebuilds_what_agents_see`, `life_terms_outlast_a_change_of_the_jail_rule` and `schema_paths_exist_and_match_what_set_config_allows`.
3. **Following an agent that is jailed, released or killed** — Inspect must keep showing it while it is jailed and say plainly when it is gone. Pinned in Task 2 by `frames_and_inspect_show_the_papers_left_screen` (a jailed agent is listed at its arrest site and still located) and in Task 6 by the `shownCitizen` tests.
4. **A Model II world that is finished from the start or finishes mid-Compare** — one group absent at t = 0, or world A dying out while B runs: the engine must pause, show the notice once, and not step a finished world. Pinned in Task 2 by `extinction_is_recorded_and_can_stop_the_run` and in Task 4 by the determinism test that runs run 7 to its end through the engine.
5. **Schedules and ramps from files and links** — a ramp on a reset field, overlapping ramps, a value out of range: rejected with the field named, never applied halfway. Pinned in Task 2 by `schedules_take_only_live_fields_with_valid_values` and `ramps_take_live_numbers_forward_in_time_without_overlapping`.

## Why this task order

- **The core first, bottom up (Tasks 1–3):** the pure pieces (a portable `exp`, the outburst bookkeeping), then the model itself with its plumbing into `ModelConfig`/`ModelWorld`, presets and golden entries, then the measurements that judge it (book-style tests over 20 seeds, the three sweeps, the CLI and the WASM portability check). Existing golden entries pin everything that must not move.
- **The page (Tasks 4–7):** types, models, charts and the engine's notice first (with the determinism checks), then the Rules panel and Compare entries, Inspect, and Experiments.
- **The survey (Task 8)** records the verdicts in the project's audit.
- **Docs and the full verification last (Task 9).**

## Decisions (where the spec leaves room)

These are binding; each is repeated in the task that implements it. Every rule below was implemented in a scratch copy during planning and measured; the Rust code in Tasks 1–3 and 8 is that code, and passes. The web code (Tasks 4–7) was written against the files as they stand and was not run during planning.

1. **Module layout.** `crates/sugarscape-core/src/civil/` with `math.rs`, `stats.rs`, `config.rs`, `world.rs`, `presets.rs` and a `mod.rs` that re-exports the public names. `pub mod civil;` in `lib.rs` after `bits`.
2. **Vision is a Euclidean radius** (the paper's "1.7" only makes sense that way; NetLogo's `in-radius`): the offsets `(dx, dy)` with `dx² + dy² ≤ v²`, excluding `(0, 0)`, row by row. 1.7 → the 8 Moore neighbors, 7 → 148 sites. Each site's view (itself, then the offsets) is precomputed per radius (`View`), rebuilt only when a radius changes. Validation: `v ≥ 1` and `2·⌊v⌋ < min(width, height)`, so no view wraps onto itself.
3. **Portable exp.** P = 1 − exp(−k·C/A) decides who rebels, so it uses `exp_neg(x)` for `x ≤ 0`: `n = round(x / ln 2)`, `r = (x − n·LN2_HI) − n·LN2_LO` (fdlibm's split, exact bit patterns `0x3fe6_2e42_fee0_0000` and `0x3dea_39ef_3579_3c76`), `e^r` by Horner's rule over 20 Taylor terms, scaled by `2^n` through the exponent bits; 0 below −708. Within 8 ulps of `f64::exp` (tested over 20 000 draws).
4. **A tick, in order.** (1) schedule entries due at this tick, then ramps; the vision tables refresh if a radius changed. (2) Cops synced to `round(cop_density × sites)`: surplus cops removed uniformly at random, missing ones placed on uniformly random empty sites (as many as fit). (3) One list — every free agent (index order), then every cop — shuffled once; each in turn, skipping agents killed or jailed earlier this tick: an agent moves (if `movement`), decides (rule A) and, in Model II if active, kills; a cop moves and arrests (rule C). (4) Jail terms count down; those reaching 0 are released in index order. (5) Model II: cloning (shuffled free agents), then aging and death. (6) Dead agents dropped, sites refiled, statistics recorded.
5. **Rule M:** a uniformly random empty site among the sites the actor sees (not its own); none → stay. "Empty" means no free agent and no cop; jailed agents are on no site's list (with `jailed_stay` a jailed agent's site therefore counts as empty, as in NetLogo).
6. **Rule A:** G = H(1 − L); C = cops seen (own site included); A = 1 + other active agents seen (own site included; `active_counts_twice` adds 1 for an already-active agent); ratio C/A (`floor_ratio`: ⌊C/A⌋); P = 1 − exp_neg(−k·ratio); active iff G − R·P > T (strictly).
7. **Rule C and jail:** a cop arrests one uniformly random active agent among those it sees (own site included); the arrested agent is quiet, jailed and off its site's list; with `cop_moves_to_arrest` the cop steps onto the arrest site. Terms: `gen_range(1..=J_max)` (the spec's ⌈U(0, J_max)⌉), `gen_range(0..J_max)` with `netlogo_jail_term`, `Term::Life` while `jail.infinite` (a life term stays a life term if the switch later goes off). Release: with `jailed_stay` free where it stands (possibly sharing); otherwise a uniformly random empty site among the arrest site and the sites the agent sees from there, else a uniformly random empty site anywhere, else it waits with 0 ticks left.
8. **Model II.** Groups Blue/Green with probability ½ (drawn after H and R), death ages `gen_range(1..=max_age)`, age 0 at setup. An active agent kills one uniformly random free agent of the other group among those it sees (own site included), removed at once. Cloning: each free agent in a shuffled order with `gen_bool(clone_probability)` onto a uniformly random empty Moore neighbor (fixed order NW, N, NE, W, E, SW, S, SE); the child keeps group and H, draws R then a death age, starts quiet at age 0 and does not age this tick. Every other agent, free or jailed, ages by 1 and dies when its age reaches its death age.
9. **Setup.** One shuffle of all sites; the first `cops()` get cops, the next `agents()` agents. Agents are created first (ids 1…N, each drawing H, R and in Model II group then death age), then cops (ids N+1…). Ids come from one counter shared with later cops and clones.
10. **Schedule and ramps.** `schedule: Vec<ScheduledChange>` (the sugarscape's type) applies an entry when the tick about to run equals its `tick`, through `ModelConfig::with_path`. A ramp `{path, start, end, to}` records the field's value when tick `start` begins (after that tick's schedule) and sets `base + (to − base)·(t − start)/(end − start)` for ticks `start < t ≤ end`. Only live fields may be scheduled; only `RAMPABLE` numbers ramped; `start < end`; two ramps of one path may not overlap (touching is fine); the target and each scheduled value must validate. `schedule` and `ramps` themselves are reset-only.
11. **Live and reset fields.** Live: `cop_density`, `legitimacy`, `threshold`, `k`, `vision`, `movement`, `jail`, `clone_probability`, `max_age`, `stop_at_extinction`, `outburst_threshold`, `quirks` (and any path inside them). Reset: `variant`, `width`, `height`, `agent_density`, `schedule`, `ramps`. `set_config` refreshes the views and syncs the cops at once.
12. **Statistics, frames, Inspect, fingerprint.** Series in the Copy order. `mean_wait`/`mean_activation` are NaN before their first value (JSON null; charts draw a gap; sweeps skip it). `extinction` is the first tick a group was gone, else the current tick (Model I: always the current tick); `finished()` is Model II with `stop_at_extinction` and a group gone. Frames: empty `BACKGROUND`, any cop on a site `COP` (`#e6e6e6`, the paper's black lightened), else the site's first free agent — Action: active `RED`, quiet `BLUE` (Model II: the group's color, Green `LENDER`); Grievance: `lerp(BACKGROUND, RED, 0.15 + 0.85·G)`; Group: its group's color. Inspect JSON `{ site: {x, y}, agent: CitizenView | null, cop: {id} | null, jailed: CitizenView[] }` where `jailed` lists jailed agents whose arrest site this is; `CitizenView { id, state: quiet|active|jailed, hardship, risk_aversion, grievance, arrest_probability, net_risk, jail_left, jail_life, group, age, death_age }` (the last three null in Model I; P and N 0 while jailed). `locate(id)` is a living agent's site (its arrest site while jailed) or a cop's. Agents CSV `id,x,y,state,jail_left,hardship,risk_aversion,grievance,group,age,death_age`. Fingerprint: FNV-1a over the tick; per living agent id, site, H and R bits, jail (0 free, 1 + ticks, or `u64::MAX` for life), active | green << 1, age << 32 | death age; per cop id and site; each ramp's recorded base (`u64::MAX` before it starts).
13. **Presets** (`civil::presets()`, after the anasazi's in the catalog): Model I runs through `model_one` (sets `floor_ratio`), Model II through `model_two` (vision 1.7, J_max 15, the paper's rule). Salami: ramp `legitimacy` 77 → 147 to 0.2. One jump: schedule `legitimacy` 0.7 at 77. Cop reductions: ramp `cop_density` 50 → 550 to 0. Safe havens: run 7 with `cop_density` 0.04 at 50. NetLogo: defaults with `Quirks::ALL`. `cv-peacekeepers-withdrawn` from the spec is **dropped**: measured, Run 8 never stabilizes (a group is gone by t ≤ 882 in 20 of 20 seeds), so there is nothing to withdraw; the spec is amended in Task 9. The spec's "legitimacy × 1000 and cops on a second axis" is drawn as separate **Legitimacy** and **Cops** charts (the chart table has one axis).
14. **Measured results** (release, seeds 1–20, recorded 2026-09-25):
    - Run 2, floored, 3000 ticks: 101–122 outbursts; peaks 290–368; 61–68 % of ticks under 10 actives; mean activation 779 (679–929; paper 708 ± 230); mean wait 21.8 (18.7–24.2; paper 60 ± 55). **Literal rule: 0 outbursts in every seed, peaks 13–34.**
    - Literal-rule rescue checks (Run 2, seeds 1–5, 3000 ticks): as specified, peaks 13–34; agents deciding before moving, 11–15; cops acting after all agents, 14–21; the paper's "north, south, east, and west" read as a Sugarscape cross (4·⌊v⌋ sites), mean actives ~40 at every tick — constant unrest, never calm, not punctuated. None produces the paper's pattern.
    - Runs 3/4, floored, paired by seed (300 ticks): the jump's peak after t = 77 is higher in 17; peaks above 50: jump 9, salami 3; the jump's jail ends larger in only 7 (salami jails 322–910, mean 425; jump 58–692, mean 314). Literal: jump higher in 19, no spikes at all, jail larger in 0.
    - Run 5, floored, 700 ticks: every seed tips; peak 64–345 (mean 194) at mean t = 177 with 66–118 cops left (mean 88). Literal: peaks 14–33, never tips.
    - Run 1, floored, 700 ticks: 33–62 actives (mean 48) over t = 100–700, never calm.
    - Run 6: no kills in 20 of 20. Run 7: genocide in 20 of 20 at t = 30–190 (mean 94), Blue survives 12, Green 8. Run 8: a group gone in 20 of 20 by t = 75–882 (mean 180). Safe havens: 20 of 20 by t = 30–377 (mean 131); at t = 50 with 0.1, 0.2 and 0.3 (seeds 1–5) still gone by t = 76–1475.
    - NetLogo: 93–109 outbursts, mean activation 1107, mean wait 23.1.
    - `cv-ratio-rules` (seeds 1–5, 1000 ticks) at L = 0.6…0.95: literal 6.2, 2.0, 0.4, 0.2, 0, 0, 0, 0; floored 48.2, 50, 47.8, 47.4, 43.2, 26.8, 0, 0; floored + twice 51.4, 48.4, 48, 47, 39.8, 25.4, 0, 0.
    - `cv-peacekeeping` (seeds 1–20, 3000 ticks) at 0…0.1: means 93.9, 106.5, 119.1, 115.7, 180.4, 155.9, 157.3, 196.1, 262.6, 258.1, 222.8; s.d. 42.0 … 325.7; max 1596 (0.08); all 220 runs end in genocide.
    - `cv-jail-waits` (seeds 1–5, 2000 ticks) at J_max 5…60: none (one endless outburst of ~320 actives), 3.5, 11.0, 14.8, 21.2, 28.1, 33.2, 40.4.
    - The paper's own arithmetic: its example outburst "60, 100, 120, 95, and 80" sums to 455, not the 500 it states (average 91, not 100); `stats.rs`'s test uses 455.
    - **Re-measuring:** `cargo test -p sugarscape-core --release --test civil -- --ignored` (the thresholds) and the sweeps with `cargo run --release -p sugarscape-cli -- sweep --builtin <id> --quiet --summary-csv /dev/stdout --out /dev/null`. An implementation that follows this plan exactly reproduces these numbers bit for bit (the golden entries pin the RNG order). **If the numbers differ, stop and report** rather than retune.
15. **Golden entries** (200 ticks from seed 1, `MODEL_GOLDEN`): `cv-run-1-no-movement` 0x49637b8b34864721, `cv-run-2-punctuated` 0x7888f03e0d511f6d, `cv-run-3-salami` 0x51664e9ecc568140, `cv-run-4-one-jump` 0xc8ad285446786559, `cv-run-5-cop-reductions` 0x5713c0cafe4898dc, `cv-run-6-coexistence` 0x1ce4fc6300e993ee, `cv-run-7-cleansing` 0x11e4a982d405341, `cv-run-8-nasty-regime` 0x5ce734d905ba0c5d, `cv-safe-havens` 0x4259235e1ab53504, `cv-netlogo` 0x87a92345c017b0ae. `wasm-pack test` reproduces cv-run-2, cv-run-8 and cv-netlogo (checked during planning). If an implementation that follows this plan gives different values, stop and report rather than re-recording.
16. **Performance** (native release): Run 2 (vision 7, 1120 agents, 64 cops) ~1.3 ms a tick; Model II at vision 1.7 ~0.2 ms. The built-in sweeps take 18–44 s natively on 10 threads.
17. **The page.** `modelOf` reads `civil`; `isCivilView(v)` is `'jailed' in v` (its site is `{x, y}` like Schelling's). `COLOR_MODES.civil` = Action, Grievance, Group (Group in Model I is all Blue); `MODEL_OVERLAYS.civil = []`. `finishedNotice` for a civil config: `A group has died out at t = ${tick} — Reset to run it again`. `ModelChart.shown?: (c: ModelConfig) => boolean` — the charts panel shows such a chart while some world of its model passes it (**Groups** and **Killed** use `isEthnic`). The schema panel hides a control whose `show_if` fails and a section whose every control is hidden, re-evaluated on every sync; for the civil model it adds a read-only **Schedule** section listing `scheduleLines(config)` (hidden when empty). Inspect shows the followed agent (free, or jailed at that site) through `shownCitizen`, the site's cop, and how many others were jailed there; a followed agent that is gone reads "Agent #id is gone: killed, or dead of old age." `defaultForm('civil')`: x `legitimacy` `0.6:0.95:0.05`, final `outbursts`, 1000 ticks, 3 seeds.
18. **The CLI.** `sugarscape run` of a world that finishes early prints `finished at tick N (a group has died out)` for a civil world, `(its end year)` for the anasazi's.
19. **The survey** gains `runner::model_preset`, `runner::model_after` (worlds of any model, on the existing thread pool through a private `on_threads`) and `claims/civil.rs` with 13 claims. Measured verdicts (20 seeds): holds — no outbursts as stated, punctuated, activation, no spike, explosion, tips, coexistence, genocide, delay; **fails** — the wait (21.6 vs 60), the one jump's larger jail, Run 8's stable regime, safe havens.

## File Structure

```
crates/sugarscape-core/src/civil/mod.rs            NEW  module doc and re-exports (1, 2)
crates/sugarscape-core/src/civil/math.rs           NEW  exp_neg (1)
crates/sugarscape-core/src/civil/stats.rs          NEW  SERIES, CivilSnapshot, Outbursts (1)
crates/sugarscape-core/src/civil/config.rs         NEW  CivilConfig, Quirks, Ramp, validation, schema (2)
crates/sugarscape-core/src/civil/world.rs          NEW  CivilWorld (2)
crates/sugarscape-core/src/civil/presets.rs        NEW  the ten presets (2)
crates/sugarscape-core/src/lib.rs                  MOD  pub mod civil (1)
crates/sugarscape-core/src/schema.rs               MOD  ShowIf, Param::show_if, shown_if (2)
crates/sugarscape-core/src/model.rs                MOD  Civil kind/config/world (2)
crates/sugarscape-core/src/presets.rs              MOD  catalog lists civil presets (2)
crates/sugarscape-core/tests/golden.rs             MOD  ten MODEL_GOLDEN entries (2)
crates/sugarscape-core/tests/civil.rs              NEW  the paper's claims over 20 seeds (3)
crates/sugarscape-core/src/sweep.rs                MOD  three built-ins and their list test (3)
sweeps/cv-ratio-rules.json, cv-peacekeeping.json, cv-jail-waits.json   NEW (3)
crates/sugarscape-cli/src/main.rs, tests/cli.rs    MOD  finish reason; built-in list; civil run (3)
crates/sugarscape-wasm/tests/web.rs                MOD  built-in list; civil fingerprints (3)
web/src/types.ts, models.ts, engine.ts             MOD  the model, its views, the notice (4)
web/src/ui/series-data.ts, ui/charts-panel.ts      MOD  MODEL_CHARTS.civil, shown (4)
web/src/schema-form.ts, ui/schema-panel.ts         MOD  show_if; the Schedule section (5)
web/src/civil.ts, civil.test.ts                    NEW  scheduleLines (5); citizenRows, shownCitizen (6)
web/src/compare-presets.ts                         MOD  two entries (5)
web/src/ui/inspect-panel.ts                        MOD  civil rows (6)
web/src/experiments/form.ts                        MOD  defaultForm('civil') (7)
web/src/*.test.ts                                  MOD  per task, as listed
survey/src/runner.rs, claims/mod.rs, claims/civil.rs   MOD/NEW (8)
README.md, docs/roadmap.md, the spec               MOD  (9)
```

---

### Task 1: A portable exponential and the outburst bookkeeping

**Files:**
- Create: `crates/sugarscape-core/src/civil/mod.rs`, `crates/sugarscape-core/src/civil/math.rs`, `crates/sugarscape-core/src/civil/stats.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Consumes: `crate::stats::Series` (a snapshot's named values).
- Produces: `civil::exp_neg(x: f64) -> f64`; `civil::SERIES: [&str; 15]`; `civil::CivilSnapshot` (fields named as the series, plus `tick`); `civil::Outbursts { pub ended: u32, .. }` with `record(&mut self, tick: u64, active: u32, threshold: u32)`, `mean_wait(&self) -> f64`, `mean_activation(&self) -> f64`.

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/civil/math.rs` with only its tests for now:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn exp_neg_matches_the_library_to_a_few_ulps() {
        let mut r = crate::rng::seeded(5);
        let mut xs: Vec<f64> = (0..20_000).map(|_| -r.gen::<f64>() * 50.0).collect();
        xs.extend([0.0, -1e-12, -0.5, -1.0, -2.3, -2.3 / 3.0, -100.0, -700.0]);
        for x in xs {
            let (ours, std) = (exp_neg(x), x.exp());
            assert!(
                (ours - std).abs() <= 8.0 * f64::EPSILON * std,
                "{x}: {ours} vs {std}"
            );
        }
        assert_eq!(exp_neg(0.0), 1.0);
        assert_eq!(exp_neg(-800.0), 0.0);
    }
}
```

and `crates/sugarscape-core/src/civil/stats.rs` with only its tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbursts_follow_the_papers_example() {
        // The paper's example flare-up of 60, 100, 120, 95, 80 (which it
        // says sums to 500 and averages 100; the sum is 455), then a wait of
        // 3 ticks and a second outburst.
        let mut o = Outbursts::default();
        let actives = [10, 60, 100, 120, 95, 80, 20, 30, 40, 55, 70, 50, 49];
        for (t, &a) in actives.iter().enumerate() {
            o.record(t as u64, a, 50);
        }
        assert_eq!(o.ended, 2);
        // The first ends at t = 6, the second starts at t = 9. At exactly
        // 50 the second is still under way (not "below 50"): 55 + 70 + 50.
        assert_eq!(o.mean_wait(), 3.0);
        assert_eq!(o.mean_activation(), (455.0 + 175.0) / 2.0);
    }

    #[test]
    fn nothing_is_measured_before_the_first_outburst() {
        let mut o = Outbursts::default();
        o.record(0, 50, 50);
        o.record(1, 51, 50);
        assert_eq!(o.ended, 0);
        assert!(o.mean_wait().is_nan() && o.mean_activation().is_nan());
        o.record(2, 3, 50);
        assert_eq!(o.mean_activation(), 51.0);
        assert!(o.mean_wait().is_nan(), "no second outburst yet");
    }
}
```

Create `crates/sugarscape-core/src/civil/mod.rs` (Task 2 replaces it):

```rust
//! Epstein's civil violence model (milestone 11). See
//! docs/superpowers/specs/2026-09-25-civil-violence-design.md.

mod math;
mod stats;

pub use math::exp_neg;
pub use stats::{CivilSnapshot, Outbursts, SERIES};
```

and in `crates/sugarscape-core/src/lib.rs` add `pub mod civil;` after `pub mod bits;`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p sugarscape-core --lib civil`
Expected: compile errors (`exp_neg`, `Outbursts` not found).

- [ ] **Step 3: Implement**

Put the implementation above each tests module. `math.rs` (Decision 3):

```rust
//! An exponential that is bit-for-bit the same on every platform. `f64::exp`
//! calls the platform's math library, which may differ in the last bit
//! between a native build and wasm32; the arrest probability decides who
//! rebels, so it uses only IEEE-exact arithmetic (as the anasazi's `ln`).

use std::f64::consts::LN_2;

/// ln 2 split in two (fdlibm's): the high part's low 21 bits are zero, so
/// `n · LN2_HI` is exact for every `n` used here.
const LN2_HI: f64 = f64::from_bits(0x3fe6_2e42_fee0_0000);
const LN2_LO: f64 = f64::from_bits(0x3dea_39ef_3579_3c76);

/// `e^x` for `x ≤ 0` from `+ − × ÷` only: `x = n·ln 2 + r` with `n` the
/// nearest whole number to `x / ln 2` (so `|r| ≤ ½ ln 2`, reduced with the
/// split ln 2 so large `n` lose nothing), `e^r` summed by Horner's rule over
/// 20 Taylor terms (far below one ulp for `|r| ≤ 0.35`), then scaled by
/// `2^n` through the exponent bits. Below −708 the result is 0 (the smallest
/// normal double is about e^−708).
pub fn exp_neg(x: f64) -> f64 {
    debug_assert!(x <= 0.0 && !x.is_nan(), "exp_neg of {x}");
    if x < -708.0 {
        return 0.0;
    }
    let n = (x / LN_2).round();
    let r = (x - n * LN2_HI) - n * LN2_LO;
    let mut series = 1.0;
    for k in (1..=20).rev() {
        series = 1.0 + series * r / f64::from(k);
    }
    // 2^n for n in −1022..=0, built from its exponent bits.
    let scale = f64::from_bits(((n as i64 + 1023) as u64) << 52);
    series * scale
}
```

`stats.rs` (Decision 12; the paper's "exceeds 50" / "falls below 50"):

```rust
//! The civil violence model's statistics and the paper's outburst
//! bookkeeping (Figs. 5 and 7).

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 15] = [
    "population",
    "active",
    "quiet",
    "jailed",
    "cops",
    "legitimacy",
    "mean_grievance",
    "tension",
    "outbursts",
    "mean_wait",
    "mean_activation",
    "blue",
    "green",
    "killed",
    "extinction",
];

/// One tick's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct CivilSnapshot {
    pub tick: u64,
    /// Agents alive, free or jailed.
    pub population: u32,
    /// Free agents that are active.
    pub active: u32,
    /// Free agents that are quiet.
    pub quiet: u32,
    pub jailed: u32,
    pub cops: u32,
    pub legitimacy: f64,
    /// Mean G over free agents (0 with none).
    pub mean_grievance: f64,
    /// The paper's ripeness index Ḡ·B̄/R̄ over free agents, B̄ the quiet
    /// share (0 with none).
    pub tension: f64,
    /// Outbursts that have ended.
    pub outbursts: u32,
    /// Mean ticks from one outburst's end to the next one's start (NaN
    /// before the first such wait; JSON null).
    pub mean_wait: f64,
    /// Mean total activation (the sum of actives over its ticks) of the
    /// outbursts that have ended (NaN before the first; JSON null).
    pub mean_activation: f64,
    /// Agents of each group, free or jailed (0 in Model I).
    pub blue: u32,
    pub green: u32,
    /// Agents killed this tick (Model II).
    pub killed: u32,
    /// The first tick at which a group was gone (Model II), else this tick.
    pub extinction: u64,
}

impl Series for CivilSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "active" => f64::from(self.active),
            "quiet" => f64::from(self.quiet),
            "jailed" => f64::from(self.jailed),
            "cops" => f64::from(self.cops),
            "legitimacy" => self.legitimacy,
            "mean_grievance" => self.mean_grievance,
            "tension" => self.tension,
            "outbursts" => f64::from(self.outbursts),
            "mean_wait" => self.mean_wait,
            "mean_activation" => self.mean_activation,
            "blue" => f64::from(self.blue),
            "green" => f64::from(self.green),
            "killed" => f64::from(self.killed),
            "extinction" => self.extinction as f64,
            _ => return None,
        })
    }
}

/// Outbursts as the paper counts them: one starts on the first tick with
/// more than `threshold` actives and ends on the first later tick with
/// fewer; its total activation sums the actives of every tick from its
/// start to the tick before its end. A wait runs from one outburst's end to
/// the next one's start.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outbursts {
    /// The running outburst's activation so far, if one is under way.
    current: Option<u64>,
    last_end: Option<u64>,
    pub ended: u32,
    activation_sum: u64,
    waits: u32,
    wait_sum: u64,
}

impl Outbursts {
    /// Records tick `tick`'s actives.
    pub fn record(&mut self, tick: u64, active: u32, threshold: u32) {
        match self.current {
            None if active > threshold => {
                if let Some(end) = self.last_end {
                    self.waits += 1;
                    self.wait_sum += tick - end;
                }
                self.current = Some(u64::from(active));
            }
            Some(sum) if active < threshold => {
                self.ended += 1;
                self.activation_sum += sum;
                self.last_end = Some(tick);
                self.current = None;
            }
            Some(sum) => self.current = Some(sum + u64::from(active)),
            None => {}
        }
    }

    pub fn mean_wait(&self) -> f64 {
        if self.waits == 0 {
            f64::NAN
        } else {
            self.wait_sum as f64 / f64::from(self.waits)
        }
    }

    pub fn mean_activation(&self) -> f64 {
        if self.ended == 0 {
            f64::NAN
        } else {
            self.activation_sum as f64 / f64::from(self.ended)
        }
    }
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p sugarscape-core --lib civil`
Expected: PASS (3 tests). Note the stats test's comment: the paper's worked example sums to 455, not the 500 it states.

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/civil/mod.rs crates/sugarscape-core/src/civil/math.rs crates/sugarscape-core/src/civil/stats.rs crates/sugarscape-core/src/lib.rs
git commit -m "Add a portable exponential and the civil violence outburst bookkeeping" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 2: The civil violence model in the core

**Files:**
- Create: `crates/sugarscape-core/src/civil/config.rs`, `crates/sugarscape-core/src/civil/world.rs`, `crates/sugarscape-core/src/civil/presets.rs`
- Modify: `crates/sugarscape-core/src/civil/mod.rs`, `crates/sugarscape-core/src/schema.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: Task 1's `exp_neg`, `CivilSnapshot`, `Outbursts`, `SERIES`; `crate::config::{FieldError, ScheduledChange}`; `crate::geometry::{Pos, Torus}`; `crate::model::{wrong_model, Model, ModelConfig, ModelKind}`; `crate::presets::ModelPreset`; `crate::render::{lerp, Rgb, BACKGROUND, BLUE, LENDER, RED}`; `crate::rng::{self, SimRng}`; `crate::schema::{Apply, Param}`; `crate::stats::Stats`; `crate::schema::check_schema` (tests).
- Produces: everything in Global Constraints' core list; `ModelKind::Civil` (`as_str` `"civil"`, `schema()` → `civil::schema()`), `ModelConfig::Civil(CivilConfig)` tagged `"model": "civil"`, `ModelWorld::Civil(Box<CivilWorld>)`; `Param::show_if: Option<ShowIf>`, `ShowIf { path: &'static str, equals: &'static str }`, `Param::shown_if(self, path, equals) -> Self`.

- [ ] **Step 1: `Param.show_if`**

In `crates/sugarscape-core/src/schema.rs`, add the field after `help` and the type after the struct:

```rust
    /// A one-line explanation shown under the control (milestone 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<&'static str>,
    /// Shown only while another field has a value (milestone 11: Model II's
    /// population fields).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_if: Option<ShowIf>,
}

/// A condition on the config: the field at `path` (a string) equals `equals`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ShowIf {
    pub path: &'static str,
    pub equals: &'static str,
}
```

set `show_if: None,` in `Param::new` after `help: None,`, and add after `with_help`:

```rust
    /// The same field, shown only while the string field `path` is `equals`.
    pub fn shown_if(mut self, path: &'static str, equals: &'static str) -> Self {
        self.show_if = Some(ShowIf { path, equals });
        self
    }
```

- [ ] **Step 2: Write the failing tests**

Create `crates/sugarscape-core/src/civil/config.rs` with its tests module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn change(tick: u64, path: &str, value: serde_json::Value) -> ScheduledChange {
        ScheduledChange {
            tick,
            set: [(path.to_string(), value)].into_iter().collect(),
        }
    }

    fn ramp(path: &str, start: u64, end: u64, to: f64) -> Ramp {
        Ramp {
            path: path.into(),
            start,
            end,
            to,
        }
    }

    fn fields(c: &CivilConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_run_2_and_validates() {
        let c = CivilConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!((c.agents(), c.cops()), (1120, 64));
        assert_eq!((c.legitimacy, c.jail.max, c.vision.agent), (0.82, 30, 7.0));
        assert_eq!(c.quirks, Quirks::default(), "the paper's rules");
    }

    #[test]
    fn agents_and_cops_must_fit_and_vision_must_stay_under_half_the_side() {
        let mut c = CivilConfig {
            agent_density: 0.97,
            ..Default::default()
        };
        assert_eq!(fields(&c), ["cop_density"]);
        c.agent_density = 0.7;
        c.vision.agent = 20.0;
        assert_eq!(fields(&c), ["vision.agent"]);
        c.vision.agent = 19.9;
        assert!(c.validate().is_ok(), "2·⌊19.9⌋ = 38 < 40");
        c.vision.cop = 0.5;
        assert_eq!(fields(&c), ["vision.cop"]);
    }

    #[test]
    fn schedules_take_only_live_fields_with_valid_values() {
        let mut c = CivilConfig {
            schedule: vec![change(10, "legitimacy", json!(0.5))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![change(10, "width", json!(50))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![change(10, "legitimacy", json!(1.5))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![change(10, "quirks.floor_ratio", json!(true))];
        assert!(c.validate().is_ok(), "leaves of a live object are live");
    }

    #[test]
    fn ramps_take_live_numbers_forward_in_time_without_overlapping() {
        let mut c = CivilConfig {
            ramps: vec![ramp("legitimacy", 77, 147, 0.2)],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.ramps = vec![ramp("movement", 1, 2, 1.0)];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![ramp("legitimacy", 5, 5, 0.2)];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![ramp("legitimacy", 0, 10, 2.0)];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![
            ramp("legitimacy", 0, 10, 0.5),
            ramp("legitimacy", 5, 20, 0.1),
        ];
        assert_eq!(fields(&c), ["ramps"]);
        c.ramps = vec![
            ramp("legitimacy", 0, 10, 0.5),
            ramp("legitimacy", 10, 20, 0.1),
        ];
        assert!(c.validate().is_ok(), "one may start where another ends");
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = CivilConfig::default();
        let mut b = a.clone();
        b.legitimacy = 0.5;
        b.cop_density = 0.1;
        b.quirks.jailed_stay = true;
        b.vision.cop = 3.0;
        assert!(a.changes(&b).is_empty());
        b.width = 50;
        b.variant = Variant::Ethnic;
        b.ramps = vec![ramp("k", 1, 2, 3.0)];
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["ramps", "variant", "width"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: CivilConfig =
            serde_json::from_str(r#"{"variant": "ethnic", "quirks": {"floor_ratio": true}}"#)
                .unwrap();
        assert_eq!(c.variant, Variant::Ethnic);
        assert!(c.quirks.floor_ratio && !c.quirks.jailed_stay);
        assert_eq!(c.width, 40);
        assert!(serde_json::from_str::<CivilConfig>(r#"{"cops": 3}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Civil(CivilConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

Create `crates/sugarscape-core/src/civil/world.rs` with its tests module (hand-built scenes on a 10 × 10 torus with vision 1.7; `put_cop` keeps `cop_density` in step so a tick neither adds nor removes cops):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::civil::config::{Ramp, Vision};
    use crate::config::ScheduledChange;
    use serde_json::json;

    /// A 10 × 10 world with nobody on it, vision 1.7 (the Moore
    /// neighborhood), after `edit`.
    fn blank(edit: impl FnOnce(&mut CivilConfig)) -> CivilWorld {
        let mut c = CivilConfig {
            width: 10,
            height: 10,
            agent_density: 0.0,
            cop_density: 0.0,
            vision: Vision {
                agent: 1.7,
                cop: 1.7,
            },
            ..Default::default()
        };
        edit(&mut c);
        CivilWorld::new(c, 1).unwrap()
    }

    /// Puts a quiet, free agent with hardship `h` and risk aversion `r` at
    /// (x, y); returns its index.
    fn put_agent(w: &mut CivilWorld, x: u32, y: u32, h: f64, r: f64) -> usize {
        let id = w.take_id();
        w.agents.push(Citizen {
            id,
            pos: Pos::new(x, y),
            hardship: h,
            risk_aversion: r,
            active: false,
            jail: None,
            green: false,
            age: 0,
            death_age: 1000,
            alive: true,
        });
        w.rebuild_sites();
        w.agents.len() - 1
    }

    /// Puts a cop at (x, y), keeping the cop density in step so a tick
    /// neither adds nor removes cops; returns its index.
    fn put_cop(w: &mut CivilWorld, x: u32, y: u32) -> usize {
        let id = w.take_id();
        w.cops.push(Cop {
            id,
            pos: Pos::new(x, y),
        });
        w.config.cop_density = w.cops.len() as f64 / w.torus.len() as f64;
        w.rebuild_sites();
        w.cops.len() - 1
    }

    fn at(w: &CivilWorld, x: u32, y: u32) -> usize {
        w.torus.index(Pos::new(x, y))
    }

    #[test]
    fn vision_is_a_euclidean_radius() {
        assert_eq!(sight(1.0).len(), 4);
        assert_eq!(sight(1.5).len(), 8, "√2 ≤ 1.5");
        assert_eq!(sight(1.7).len(), 8, "the paper's 1.7: the eight neighbors");
        assert_eq!(sight(7.0).len(), 148);
        assert!(!sight(7.0).contains(&(0, 0)));
    }

    #[test]
    fn views_wrap_around_the_torus() {
        let w = blank(|_| {});
        let seen = w.agent_view.of(at(&w, 0, 0));
        assert_eq!(seen[0] as usize, at(&w, 0, 0), "its own site first");
        assert!(seen.contains(&(at(&w, 9, 9) as u32)));
        assert!(seen.contains(&(at(&w, 1, 9) as u32)));
        assert_eq!(seen.len(), 9);
    }

    #[test]
    fn the_arrest_probability_counts_cops_and_other_actives_in_view() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        assert_eq!(w.arrest_probability(me), 0.0, "no cops, no risk");
        put_cop(&mut w, 5, 6);
        put_cop(&mut w, 5, 8); // out of view
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64).exp())).abs() < 1e-12,
            "C = A = 1: {p}"
        );
        assert!((p - 0.9).abs() < 0.001, "the paper's calibration");
        for x in [4, 6] {
            let a = put_agent(&mut w, x, 5, 0.5, 0.5);
            w.agents[a].active = true;
        }
        let quiet = put_agent(&mut w, 4, 4, 0.5, 0.5);
        assert!(!w.agents[quiet].active, "quiet agents are not counted");
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64 / 3.0).exp())).abs() < 1e-12,
            "C/A = 1/3: {p}"
        );
        w.config.quirks.floor_ratio = true;
        assert_eq!(w.arrest_probability(me), 0.0, "⌊1/3⌋ = 0");
        w.config.quirks.floor_ratio = false;
        w.config.quirks.active_counts_twice = true;
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64 / 3.0).exp())).abs() < 1e-12,
            "a quiet agent once"
        );
        w.agents[me].active = true;
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64 / 4.0).exp())).abs() < 1e-12,
            "an active one twice"
        );
    }

    #[test]
    fn an_agent_is_active_only_when_g_minus_n_exceeds_t() {
        let mut w = blank(|c| {
            c.legitimacy = 0.0;
            c.threshold = 0.25;
        });
        let at_t = put_agent(&mut w, 1, 1, 0.25, 1.0);
        let above = put_agent(&mut w, 5, 5, 0.250_001, 1.0);
        w.decide(at_t);
        w.decide(above);
        assert!(!w.agents[at_t].active, "G − N = T is quiet");
        assert!(w.agents[above].active);
        put_cop(&mut w, 5, 6);
        w.decide(above);
        assert!(!w.agents[above].active, "R·P now outweighs G");
    }

    #[test]
    fn agents_move_only_to_empty_sites_in_view() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        for (x, y) in [(4, 4), (5, 4), (6, 4), (4, 5), (6, 5), (4, 6), (5, 6)] {
            put_cop(&mut w, x, y);
        }
        w.move_agent(me);
        assert_eq!(w.agents[me].pos, Pos::new(6, 6), "the one empty site");
        for (x, y) in [(5, 5), (7, 5), (7, 6), (5, 7), (6, 7), (7, 7)] {
            put_cop(&mut w, x, y);
        }
        w.move_agent(me);
        assert_eq!(w.agents[me].pos, Pos::new(6, 6), "boxed in: it stays");
        let here = &w.sites[at(&w, 6, 6)];
        assert_eq!(here, &[Occupant::Agent(me as u32)]);
    }

    #[test]
    fn without_movement_agents_stay_but_cops_still_move() {
        let mut w = blank(|c| c.movement = false);
        let me = put_agent(&mut w, 2, 2, 0.1, 0.9);
        let cop = put_cop(&mut w, 7, 7);
        w.step();
        assert_eq!(w.agents[me].pos, Pos::new(2, 2));
        assert_ne!(w.cops[cop].pos, Pos::new(7, 7));
    }

    #[test]
    fn cops_arrest_one_active_agent_they_see() {
        let mut w = blank(|c| c.jail.max = 5);
        let cop = put_cop(&mut w, 0, 0);
        let near = put_agent(&mut w, 1, 1, 0.5, 0.5);
        let far = put_agent(&mut w, 3, 3, 0.5, 0.5);
        let quiet = put_agent(&mut w, 9, 9, 0.5, 0.5);
        w.agents[near].active = true;
        w.agents[far].active = true;
        w.arrest(cop);
        assert!(matches!(w.agents[near].jail, Some(Term::Ticks(1..=5))));
        assert!(!w.agents[near].active, "jailed agents are quiet");
        assert!(w.sites[at(&w, 1, 1)].is_empty(), "off the lattice");
        assert!(w.agents[far].jail.is_none() && w.agents[quiet].jail.is_none());
        assert_eq!(w.cops[cop].pos, Pos::new(0, 0), "the paper's cop stays put");
    }

    #[test]
    fn jail_terms_follow_the_rule_in_force() {
        let mut w = blank(|c| c.jail.max = 3);
        let draws = |w: &mut CivilWorld| {
            let mut seen: Vec<Term> = (0..300).map(|_| w.jail_term()).collect();
            seen.sort_by_key(|t| match t {
                Term::Ticks(n) => *n,
                Term::Life => u32::MAX,
            });
            seen.dedup();
            seen
        };
        assert_eq!(
            draws(&mut w),
            [Term::Ticks(1), Term::Ticks(2), Term::Ticks(3)]
        );
        w.config.quirks.netlogo_jail_term = true;
        assert_eq!(
            draws(&mut w),
            [Term::Ticks(0), Term::Ticks(1), Term::Ticks(2)]
        );
        w.config.jail.infinite = true;
        assert_eq!(draws(&mut w), [Term::Life]);
    }

    #[test]
    fn terms_count_down_and_agents_are_released_near_their_arrest() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        let who = Occupant::Agent(me as u32);
        w.leave(at(&w, 5, 5), who);
        w.agents[me].jail = Some(Term::Ticks(2));
        w.serve_terms();
        assert_eq!(w.agents[me].jail, Some(Term::Ticks(1)));
        w.serve_terms();
        assert_eq!(w.agents[me].jail, None);
        let p = w.agents[me].pos;
        assert!(
            p.x.abs_diff(5) <= 1 && p.y.abs_diff(5) <= 1,
            "near (5, 5): {p:?}"
        );
        assert!(w.sites[w.torus.index(p)].contains(&who));
    }

    #[test]
    fn released_agents_go_anywhere_when_their_arrest_site_is_crowded_and_wait_when_full() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        w.leave(at(&w, 5, 5), Occupant::Agent(me as u32));
        w.agents[me].jail = Some(Term::Ticks(1));
        for y in 4..=6 {
            for x in 4..=6 {
                put_cop(&mut w, x, y);
            }
        }
        w.serve_terms();
        let p = w.agents[me].pos;
        assert!(w.agents[me].jail.is_none());
        assert!(p.x.abs_diff(5) > 1 || p.y.abs_diff(5) > 1, "{p:?}");
        let mut full = blank(|_| {});
        let me = put_agent(&mut full, 0, 0, 0.5, 0.5);
        full.leave(0, Occupant::Agent(me as u32));
        full.agents[me].jail = Some(Term::Ticks(1));
        for y in 0..10 {
            for x in 0..10 {
                put_cop(&mut full, x, y);
            }
        }
        full.serve_terms();
        assert_eq!(full.agents[me].jail, Some(Term::Ticks(0)), "it waits");
    }

    #[test]
    fn netlogo_jails_keep_their_site_and_the_cop_steps_onto_it() {
        let mut w = blank(|c| {
            c.quirks.jailed_stay = true;
            c.quirks.cop_moves_to_arrest = true;
            c.jail.max = 1;
        });
        let cop = put_cop(&mut w, 5, 6);
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        w.agents[me].active = true;
        w.arrest(cop);
        assert_eq!(w.cops[cop].pos, Pos::new(5, 5));
        assert_eq!(w.agents[me].pos, Pos::new(5, 5), "jailed where it stood");
        assert!(w.sites[at(&w, 5, 6)].is_empty());
        w.serve_terms();
        assert!(w.agents[me].jail.is_none());
        let here = &w.sites[at(&w, 5, 5)];
        assert!(here.contains(&Occupant::Cop(cop as u32)));
        assert!(
            here.contains(&Occupant::Agent(me as u32)),
            "sharing with the cop"
        );
    }

    #[test]
    fn a_live_cop_density_adds_and_removes_cops() {
        let mut w = blank(|_| {});
        put_agent(&mut w, 0, 0, 0.5, 0.5);
        let mut next = w.config.clone();
        next.cop_density = 0.05;
        w.set_config(ModelConfig::Civil(next.clone())).unwrap();
        assert_eq!(w.cops.len(), 5);
        let mut sites: Vec<Pos> = w.cops.iter().map(|c| c.pos).collect();
        sites.sort();
        sites.dedup();
        assert_eq!(sites.len(), 5, "on distinct sites");
        assert!(!sites.contains(&Pos::new(0, 0)), "on empty sites");
        next.cop_density = 0.02;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        assert_eq!(w.cops.len(), 2);
    }

    #[test]
    fn a_full_lattice_takes_no_more_cops_until_a_site_frees() {
        let mut w = blank(|_| {});
        for y in 0..10 {
            for x in 0..10 {
                put_agent(&mut w, x, y, 0.1, 0.9);
            }
        }
        let mut next = w.config.clone();
        next.cop_density = 0.05;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        assert!(w.cops.is_empty(), "no empty site to put one on");
        w.agents.pop();
        w.rebuild_sites();
        w.step();
        assert_eq!(w.cops.len(), 1, "the one freed site");
    }

    #[test]
    fn a_live_vision_change_rebuilds_what_agents_see() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        put_cop(&mut w, 5, 7);
        assert_eq!(
            w.arrest_probability(me),
            0.0,
            "two sites away: out of sight"
        );
        let mut next = w.config.clone();
        next.vision.agent = 2.0;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        assert_eq!(w.agent_view.of(0).len(), 13, "itself and 12 sites");
        assert!(w.arrest_probability(me) > 0.8);
    }

    #[test]
    fn life_terms_outlast_a_change_of_the_jail_rule() {
        let mut w = blank(|c| c.jail.infinite = true);
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        w.leave(at(&w, 5, 5), Occupant::Agent(me as u32));
        w.agents[me].jail = Some(Term::Life);
        let mut next = w.config.clone();
        next.jail.infinite = false;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        for _ in 0..40 {
            w.serve_terms();
        }
        assert_eq!(w.agents[me].jail, Some(Term::Life));
    }

    #[test]
    fn schedules_apply_before_ramps_and_ramps_move_linearly() {
        let mut w = blank(|c| {
            c.legitimacy = 0.9;
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("legitimacy".to_string(), json!(0.5))]
                    .into_iter()
                    .collect(),
            }];
            c.ramps = vec![Ramp {
                path: "legitimacy".into(),
                start: 2,
                end: 6,
                to: 0.1,
            }];
        });
        w.run(8);
        let l = w.stats.series("legitimacy").unwrap();
        let want = [0.9, 0.9, 0.9, 0.5, 0.4, 0.3, 0.2, 0.1, 0.1];
        for (t, (got, want)) in l.iter().zip(want).enumerate() {
            assert!((got - want).abs() < 1e-12, "tick {t}: {got} vs {want}");
        }
    }

    #[test]
    fn model_two_agents_kill_only_the_other_group_in_view() {
        let mut w = blank(|c| {
            c.variant = Variant::Ethnic;
            c.legitimacy = 0.0;
        });
        let killer = put_agent(&mut w, 5, 5, 1.0, 0.5);
        let friend = put_agent(&mut w, 4, 5, 0.0, 0.5);
        let victim = put_agent(&mut w, 5, 6, 0.0, 0.5);
        let far = put_agent(&mut w, 8, 8, 0.0, 0.5);
        for i in [victim, far] {
            w.agents[i].green = true;
        }
        w.decide(killer);
        assert!(w.agents[killer].active, "G = 1 > T with no cops");
        w.kill(killer);
        assert!(!w.agents[victim].alive);
        assert!(w.sites[at(&w, 5, 6)].is_empty());
        assert!(w.agents[friend].alive && w.agents[far].alive);
        assert_eq!(w.killed, 1);
    }

    #[test]
    fn model_two_agents_clone_onto_an_empty_neighbor_and_die_of_age() {
        let mut w = blank(|c| {
            c.variant = Variant::Ethnic;
            c.clone_probability = 1.0;
        });
        let parent = put_agent(&mut w, 5, 5, 0.3, 0.5);
        w.agents[parent].green = true;
        for (x, y) in [(5, 4), (6, 4), (4, 5), (6, 5), (4, 6), (5, 6), (6, 6)] {
            put_cop(&mut w, x, y);
        }
        let jailed = put_agent(&mut w, 0, 0, 0.5, 0.5);
        w.leave(0, Occupant::Agent(jailed as u32));
        w.agents[jailed].jail = Some(Term::Life);
        w.agents[jailed].death_age = 1;
        w.demography();
        let child = w.agents.last().unwrap();
        assert_eq!(child.pos, Pos::new(4, 4), "the one empty neighbor");
        assert!(child.green && child.hardship == 0.3 && child.age == 0);
        assert!(child.death_age >= 1 && child.death_age <= w.config.max_age);
        assert_eq!(w.agents[parent].age, 1);
        assert!(!w.agents[jailed].alive, "jailed agents age and die too");
    }

    #[test]
    fn extinction_is_recorded_and_can_stop_the_run() {
        let mut w = blank(|c| {
            c.variant = Variant::Ethnic;
            c.stop_at_extinction = true;
        });
        put_agent(&mut w, 1, 1, 0.1, 0.9);
        w.stats = Stats::default();
        w.record();
        assert_eq!(
            w.stats.latest().unwrap().extinction,
            0,
            "Green was never there"
        );
        assert!(w.is_finished());
        w.run(5);
        assert_eq!(w.tick, 0, "a finished world does not run");
        let mut one = blank(|_| {});
        put_agent(&mut one, 1, 1, 0.1, 0.9);
        one.run(3);
        assert!(!one.is_finished(), "Model I never finishes");
        assert_eq!(one.stats.latest().unwrap().extinction, 3);
    }

    #[test]
    fn model_one_keeps_its_population_and_worlds_follow_their_seed() {
        let c = CivilConfig::default();
        let mut a = CivilWorld::new(c.clone(), 7).unwrap();
        let mut b = CivilWorld::new(c.clone(), 7).unwrap();
        let mut other = CivilWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(40);
        }
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
        assert!(a
            .stats
            .series("population")
            .unwrap()
            .iter()
            .all(|&p| p == 1120.0));
        let s = a.stats.latest().unwrap();
        assert_eq!(s.active + s.quiet + s.jailed, 1120);
        assert_eq!(s.cops, 64);
    }

    #[test]
    fn frames_and_inspect_show_the_papers_left_screen() {
        let mut w = blank(|_| {});
        put_cop(&mut w, 1, 0);
        let me = put_agent(&mut w, 0, 0, 0.5, 0.25);
        w.agents[me].active = true;
        let mut buf = Vec::new();
        w.render("action", "", &mut buf).unwrap();
        assert_eq!(&buf[0..3], &RED);
        assert_eq!(&buf[4..7], &COP);
        assert_eq!(&buf[8..11], &BACKGROUND);
        w.render("grievance", "", &mut buf).unwrap();
        assert_eq!(&buf[0..3], &lerp(BACKGROUND, RED, 0.15 + 0.85 * 0.5 * 0.18));
        assert!(w.render("nope", "", &mut buf).is_err());
        let i = w.inspect(0, 0).unwrap();
        let a = i.agent.unwrap();
        assert_eq!((a.state, a.group, a.age), ("active", None, None));
        assert!((a.grievance - 0.09).abs() < 1e-12);
        assert!((a.net_risk - 0.25 * a.arrest_probability).abs() < 1e-12);
        assert!(i.cop.is_none());
        assert!(w.inspect(1, 0).unwrap().cop.is_some());
        assert!(w.inspect(10, 0).is_err());
        w.arrest(0);
        let i = w.inspect(0, 0).unwrap();
        assert!(i.agent.is_none(), "off the lattice");
        assert_eq!(i.jailed.len(), 1, "but listed where it was arrested");
        assert_eq!(i.jailed[0].state, "jailed");
        assert_eq!(
            w.locate(w.agents[me].id),
            Some((0, 0)),
            "Inspect keeps following it"
        );
    }
}
```

In `crates/sugarscape-core/src/model.rs`'s tests, add before `only_the_anasazi_finishes`:

```rust
    #[test]
    fn civil_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "civil", "variant": "ethnic", "quirks": {"floor_ratio": true}}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Civil);
        let ModelConfig::Civil(v) = &c else {
            unreachable!()
        };
        assert!(v.quirks.floor_ratio && !v.quirks.jailed_stay);
        assert_eq!(
            (v.width, v.legitimacy),
            (40, 0.82),
            "missing fields take the defaults"
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "civil");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..3], ["population", "active", "quiet"]);
        assert_eq!(c.max_ticks(), None);
        let next = c.with_path("vision.cop", &json!(3.0)).unwrap();
        let ModelConfig::Civil(n) = &next else {
            unreachable!()
        };
        assert_eq!(n.vision.cop, 3.0);
        let e = ModelConfig::from_json(r#"{"model": "civil", "legitimacy": 2}"#).unwrap_err();
        assert_eq!(e[0].field, "legitimacy");
        let w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Civil);
        assert_eq!(w.model().size(), (40, 40));
    }
```

and in `every_kind_names_itself_and_only_other_models_have_schemas` change the expected names to `["sugarscape", "schelling", "ring", "anasazi", "civil"]`.

In `crates/sugarscape-core/tests/golden.rs`, append to `MODEL_GOLDEN` after `("lhv-documented", 0x33ffb6476e824b0),` (Decision 15):

```rust
    ("cv-run-1-no-movement", 0x49637b8b34864721),
    ("cv-run-2-punctuated", 0x7888f03e0d511f6d),
    ("cv-run-3-salami", 0x51664e9ecc568140),
    ("cv-run-4-one-jump", 0xc8ad285446786559),
    ("cv-run-5-cop-reductions", 0x5713c0cafe4898dc),
    ("cv-run-6-coexistence", 0x1ce4fc6300e993ee),
    ("cv-run-7-cleansing", 0x11e4a982d405341),
    ("cv-run-8-nasty-regime", 0x5ce734d905ba0c5d),
    ("cv-safe-havens", 0x4259235e1ab53504),
    ("cv-netlogo", 0x87a92345c017b0ae),
```

- [ ] **Step 3: Run the tests to see them fail**

Run: `cargo test -p sugarscape-core --lib civil`
Expected: compile errors (no `CivilConfig`, `CivilWorld`, `ModelConfig::Civil`).

- [ ] **Step 4: Wire the model into `model.rs`**

In `crates/sugarscape-core/src/model.rs` (each edit beside the anasazi's):

```rust
use crate::anasazi::{AnasaziConfig, AnasaziWorld};
use crate::civil::{CivilConfig, CivilWorld};
// …
use crate::{anasazi, civil, export, ring, schelling, stats};
```

- `ModelKind`: add the variant `Civil` after `Anasazi`; `ALL: [ModelKind; 5]` ending with `ModelKind::Civil`; `as_str`: `ModelKind::Civil => "civil"`; `schema`: `ModelKind::Civil => civil::schema()`.
- `ModelConfig`: variant `Civil(CivilConfig)`; `Tagged`: `Civil(&'a CivilConfig)`; `serialize`: `ModelConfig::Civil(c) => Tagged::Civil(c).serialize(s)`; `kind`: `ModelConfig::Civil(_) => ModelKind::Civil`.
- `from_value`, after the `"anasazi"` arm:

```rust
            "civil" => serde_json::from_value(value)
                .map(ModelConfig::Civil)
                .map_err(|e| FieldError::new("config", e.to_string())),
```

  and the unknown-model message becomes `"unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi or civil)"`.
- `validate`: `ModelConfig::Civil(c) => c.validate()`; `with_path`: `ModelConfig::Civil(c) => set_path(c, path, value).map(ModelConfig::Civil)`; `max_ticks`: add `| ModelConfig::Civil(_)` to the `None` arm; `series_names`: `ModelConfig::Civil(_) => civil::SERIES.iter().map(|s| s.to_string()).collect()`.
- `ModelWorld`: variant `Civil(Box<CivilWorld>)`; `with_landscapes`: `ModelConfig::Civil(c) => ModelWorld::Civil(Box::new(CivilWorld::new(c, seed)?))`; `kind`, `model`, `model_mut`: the `Civil` arms like the anasazi's.

In `crates/sugarscape-core/src/presets.rs`, `catalog()` gains `out.extend(crate::civil::presets());` after the anasazi's, and its doc comment reads "the sugarscape's (`all`), then Schelling's, Ring World's, the anasazi's and civil violence's."

- [ ] **Step 5: Implement the config**

Above `config.rs`'s tests (Decisions 10, 11; the schema's groups and labels are Copy):

```rust
//! The civil violence model's parameters (Epstein 2002, Table 2), NetLogo
//! *Rebellion*'s departures as named switches, and the model's own schedule:
//! steps (`schedule`) and linear ramps (`ramps`) of its live fields.

use serde::{Deserialize, Serialize};

use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// Model I (rebellion against a central authority) or Model II (violence
/// between two ethnic groups).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    #[default]
    Rebellion,
    Ethnic,
}

/// Agent and cop vision: Euclidean radii in cells.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Vision {
    pub agent: f64,
    pub cop: f64,
}

impl Default for Vision {
    fn default() -> Self {
        Vision {
            agent: 7.0,
            cop: 7.0,
        }
    }
}

/// The maximum jail term J_max, in ticks, or terms that never end.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Jail {
    pub max: u32,
    pub infinite: bool,
}

impl Default for Jail {
    fn default() -> Self {
        Jail {
            max: 30,
            infinite: false,
        }
    }
}

/// NetLogo *Rebellion*'s departures from the paper (all off: the paper).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Quirks {
    /// P = 1 − exp(−k·⌊C/A⌋): the ratio rounded down.
    pub floor_ratio: bool,
    /// A counts an already-active agent twice (NetLogo's `1 + count …
    /// with [active?]` includes the agent itself).
    pub active_counts_twice: bool,
    /// The arresting cop steps onto the arrested agent's site.
    pub cop_moves_to_arrest: bool,
    /// Jailed agents keep their site, which counts as empty; a released
    /// agent is free where it stands, perhaps sharing the site.
    pub jailed_stay: bool,
    /// Jail terms are uniform whole numbers in 0..J_max − 1.
    pub netlogo_jail_term: bool,
}

impl Quirks {
    pub const ALL: Quirks = Quirks {
        floor_ratio: true,
        active_counts_twice: true,
        cop_moves_to_arrest: true,
        jailed_stay: true,
        netlogo_jail_term: true,
    };
}

/// A numeric live field moving linearly from its value when tick `start`
/// begins to `to` when tick `end` begins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ramp {
    pub path: String,
    pub start: u64,
    pub end: u64,
    pub to: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CivilConfig {
    pub variant: Variant,
    pub width: u32,
    pub height: u32,
    pub agent_density: f64,
    pub cop_density: f64,
    pub legitimacy: f64,
    pub threshold: f64,
    pub k: f64,
    pub vision: Vision,
    pub movement: bool,
    pub jail: Jail,
    pub clone_probability: f64,
    pub max_age: u32,
    pub stop_at_extinction: bool,
    pub outburst_threshold: u32,
    pub quirks: Quirks,
    pub schedule: Vec<ScheduledChange>,
    pub ramps: Vec<Ramp>,
}

impl Default for CivilConfig {
    /// Table 2's Run 2 on the "All models" line: a 40 × 40 torus, density
    /// 0.7, cops 0.04, vision 7, L = 0.82, J_max = 30, k = 2.3, T = 0.1.
    fn default() -> Self {
        CivilConfig {
            variant: Variant::Rebellion,
            width: 40,
            height: 40,
            agent_density: 0.7,
            cop_density: 0.04,
            legitimacy: 0.82,
            threshold: 0.1,
            k: 2.3,
            vision: Vision::default(),
            movement: true,
            jail: Jail::default(),
            clone_probability: 0.05,
            max_age: 200,
            stop_at_extinction: false,
            outburst_threshold: 50,
            quirks: Quirks::default(),
            schedule: Vec::new(),
            ramps: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 12] = [
    "cop_density",
    "legitimacy",
    "threshold",
    "k",
    "vision",
    "movement",
    "jail",
    "clone_probability",
    "max_age",
    "stop_at_extinction",
    "outburst_threshold",
    "quirks",
];

/// The numeric fields a ramp may move.
pub const RAMPABLE: [&str; 7] = [
    "legitimacy",
    "cop_density",
    "threshold",
    "k",
    "vision.agent",
    "vision.cop",
    "clone_probability",
];

/// Whether a dotted path names a live field or lies inside one.
pub fn is_live(path: &str) -> bool {
    LIVE.contains(&path.split('.').next().unwrap_or(path))
}

impl CivilConfig {
    pub fn sites(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    /// round(density × sites).
    pub fn count(&self, density: f64) -> u32 {
        (density * self.sites() as f64).round() as u32
    }

    pub fn agents(&self) -> u32 {
        self.count(self.agent_density)
    }

    pub fn cops(&self) -> u32 {
        self.count(self.cop_density)
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = self.validate_fields();
        e.extend(self.validate_changes());
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// The checks on the fields themselves (not the schedule or ramps).
    fn validate_fields(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (5..=400).contains(&self.width),
            "width",
            "must be between 5 and 400",
        );
        check(
            (5..=400).contains(&self.height),
            "height",
            "must be between 5 and 400",
        );
        check(unit(self.agent_density), "agent_density", "must be 0–1");
        check(unit(self.cop_density), "cop_density", "must be 0–1");
        check(
            u64::from(self.agents()) + u64::from(self.cops()) <= self.sites(),
            "cop_density",
            "agents and cops must fit on the lattice (densities sum to at most 1)",
        );
        check(unit(self.legitimacy), "legitimacy", "must be 0–1");
        check(
            (-1.0..=1.0).contains(&self.threshold),
            "threshold",
            "must be between −1 and 1",
        );
        check(
            self.k.is_finite() && self.k > 0.0 && self.k <= 100.0,
            "k",
            "must be above 0 and at most 100",
        );
        let side = f64::from(self.width.min(self.height));
        for (field, v) in [
            ("vision.agent", self.vision.agent),
            ("vision.cop", self.vision.cop),
        ] {
            check(
                v.is_finite() && v >= 1.0 && 2.0 * v.floor() < side,
                field,
                "must be at least 1 and less than half the lattice's smaller side",
            );
        }
        check(self.jail.max >= 1, "jail.max", "must be ≥ 1");
        check(
            unit(self.clone_probability),
            "clone_probability",
            "must be 0–1",
        );
        check(self.max_age >= 1, "max_age", "must be ≥ 1");
        check(
            self.outburst_threshold >= 1,
            "outburst_threshold",
            "must be ≥ 1",
        );
        e
    }

    /// Schedule entries and ramps: live paths only, values that validate,
    /// ramps on numeric fields with `start < end`, and no two ramps of one
    /// path overlapping.
    fn validate_changes(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        for change in &self.schedule {
            for (path, value) in &change.set {
                if !is_live(path) {
                    e.push(FieldError::new(
                        "schedule",
                        format!("{path} changes only on reset and cannot be scheduled"),
                    ));
                    continue;
                }
                match ModelConfig::Civil(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Civil(next)) => {
                        for f in next.validate_fields() {
                            e.push(FieldError::new(
                                "schedule",
                                format!("tick {}: {}: {}", change.tick, f.field, f.message),
                            ));
                        }
                    }
                    Ok(_) => unreachable!("with_path keeps the model"),
                    Err(f) => e.push(f),
                }
            }
        }
        for (i, r) in self.ramps.iter().enumerate() {
            if !RAMPABLE.contains(&r.path.as_str()) {
                e.push(FieldError::new(
                    "ramps",
                    format!("{} cannot be ramped (ramp a live number field)", r.path),
                ));
                continue;
            }
            if r.start >= r.end {
                e.push(FieldError::new(
                    "ramps",
                    format!("{}: start must be before end", r.path),
                ));
            }
            let mut probe = self.clone();
            probe.set_number(&r.path, r.to);
            for f in probe.validate_fields() {
                e.push(FieldError::new(
                    "ramps",
                    format!("{} to {}: {}", r.path, r.to, f.message),
                ));
            }
            for other in &self.ramps[i + 1..] {
                if other.path == r.path && other.start < r.end && r.start < other.end {
                    e.push(FieldError::new(
                        "ramps",
                        format!("two ramps of {} overlap", r.path),
                    ));
                }
            }
        }
        e
    }

    /// The value of a rampable field.
    pub fn number(&self, path: &str) -> f64 {
        match path {
            "legitimacy" => self.legitimacy,
            "cop_density" => self.cop_density,
            "threshold" => self.threshold,
            "k" => self.k,
            "vision.agent" => self.vision.agent,
            "vision.cop" => self.vision.cop,
            "clone_probability" => self.clone_probability,
            _ => unreachable!("{path} is not rampable"),
        }
    }

    /// Sets a rampable field.
    pub fn set_number(&mut self, path: &str, v: f64) {
        let slot = match path {
            "legitimacy" => &mut self.legitimacy,
            "cop_density" => &mut self.cop_density,
            "threshold" => &mut self.threshold,
            "k" => &mut self.k,
            "vision.agent" => &mut self.vision.agent,
            "vision.cop" => &mut self.vision.cop,
            "clone_probability" => &mut self.clone_probability,
            _ => unreachable!("{path} is not rampable"),
        };
        *slot = v;
    }

    /// The reset-only fields that differ from `next`.
    pub fn changes(&self, next: &CivilConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !is_live(k))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::choice(
            "World",
            "variant",
            "Model",
            &[
                ("rebellion", "I — rebellion against a central authority"),
                ("ethnic", "II — violence between two groups"),
            ],
            Reset,
        ),
        Param::integer("World", "width", "Width", (5, 200), Reset),
        Param::integer("World", "height", "Height", (5, 200), Reset),
        Param::number(
            "World",
            "legitimacy",
            "Legitimacy (L)",
            (0.0, 1.0, 0.01),
            Live,
        )
        .with_help("G = H(1 − L): the regime's legitimacy, or in Model II each group's view of the other's right to exist"),
        Param::number(
            "Agents",
            "agent_density",
            "Agent density",
            (0.0, 1.0, 0.01),
            Reset,
        ),
        Param::number(
            "Agents",
            "vision.agent",
            "Agent vision (cells)",
            (1.0, 20.0, 0.1),
            Live,
        )
        .with_help("A Euclidean radius: 1.7 sees the eight neighbors"),
        Param::number(
            "Agents",
            "threshold",
            "Threshold (T)",
            (-1.0, 1.0, 0.01),
            Live,
        )
        .with_help("Active when G − R·P > T"),
        Param::number("Agents", "k", "Arrest constant (k)", (0.1, 10.0, 0.1), Live)
            .with_help("P = 1 − exp(−k·C/A); 2.3 gives P ≈ 0.9 for one cop and one active"),
        Param::bool("Agents", "movement", "Agents move", Live),
        Param::number(
            "Cops",
            "cop_density",
            "Cop density",
            (0.0, 0.3, 0.001),
            Live,
        )
        .with_help("Cops are added on random empty sites or removed at random"),
        Param::number(
            "Cops",
            "vision.cop",
            "Cop vision (cells)",
            (1.0, 20.0, 0.1),
            Live,
        ),
        Param::integer("Jail", "jail.max", "Longest term (ticks)", (1, 1000), Live),
        Param::bool("Jail", "jail.infinite", "Terms never end", Live),
        Param::number(
            "Population",
            "clone_probability",
            "Cloning probability",
            (0.0, 1.0, 0.01),
            Live,
        )
        .with_help("Each tick a free agent clones onto an empty neighboring site")
        .shown_if("variant", "ethnic"),
        Param::integer("Population", "max_age", "Longest life (ticks)", (1, 1000), Live)
            .with_help("Each agent's death age is uniform in 1…this")
            .shown_if("variant", "ethnic"),
        Param::bool(
            "Population",
            "stop_at_extinction",
            "Stop when a group dies out",
            Live,
        )
        .shown_if("variant", "ethnic"),
        Param::integer(
            "Outbursts",
            "outburst_threshold",
            "Outburst above (actives)",
            (1, 1000),
            Live,
        )
        .with_help("An outburst starts above this many actives and ends below it (the paper's 50)"),
        Param::bool(
            "NetLogo quirks",
            "quirks.floor_ratio",
            "Round C/A down",
            Live,
        )
        .with_help("NetLogo's floor(C/A): no risk once actives outnumber the cops in view. The paper's rule gives Model I no outbursts without it"),
        Param::bool(
            "NetLogo quirks",
            "quirks.active_counts_twice",
            "Count an active agent twice",
            Live,
        )
        .with_help("NetLogo's A = 1 + actives in view includes the agent itself once it is active"),
        Param::bool(
            "NetLogo quirks",
            "quirks.cop_moves_to_arrest",
            "Cop moves to the arrest",
            Live,
        )
        .with_help("The arresting cop steps onto the arrested agent's site"),
        Param::bool(
            "NetLogo quirks",
            "quirks.jailed_stay",
            "Jailed agents stay put",
            Live,
        )
        .with_help("A jailed agent's site counts as empty; released agents may share a site"),
        Param::bool(
            "NetLogo quirks",
            "quirks.netlogo_jail_term",
            "Terms 0 to J − 1",
            Live,
        )
        .with_help("NetLogo's random J_max: a term may be 0 ticks"),
    ]
}
```

- [ ] **Step 6: Implement the world**

Above `world.rs`'s tests (Decisions 2, 4–9, 12):

```rust
//! The civil violence world: agents and cops on a torus, acting once a tick
//! in one random order (Epstein 2002's rules M, A and C), jail, and in
//! Model II killing, cloning and death by age.

use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{CivilConfig, Variant};
use super::math::exp_neg;
use super::stats::{CivilSnapshot, Outbursts, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::geometry::{Pos, Torus};
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, LENDER, RED};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

/// Cops: the paper's black, lightened for the dark grid.
pub const COP: Rgb = [0xe6, 0xe6, 0xe6];
/// Model II's second group.
pub const GREEN: Rgb = LENDER;

/// What is left of a jail term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Term {
    Ticks(u32),
    Life,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Citizen {
    pub id: u64,
    /// Where it stands; while jailed, where it was arrested.
    pub pos: Pos,
    /// Perceived hardship H and risk aversion R, each U(0,1).
    pub hardship: f64,
    pub risk_aversion: f64,
    pub active: bool,
    pub jail: Option<Term>,
    /// Model II: Green (else Blue). Always false in Model I.
    pub green: bool,
    /// Model II: ticks lived and the age at which it dies (0 in Model I).
    pub age: u32,
    pub death_age: u32,
    alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cop {
    pub id: u64,
    pub pos: Pos,
}

/// Who stands on a site: an index into the agents or the cops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Occupant {
    Agent(u32),
    Cop(u32),
}

/// The color modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CivilMode {
    /// The paper's left screen: quiet blue (Model II: the group's color),
    /// active red.
    Action,
    /// The right screen: red by grievance.
    Grievance,
    /// Model II's groups.
    Group,
}

impl std::str::FromStr for CivilMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "action" => Self::Action,
            "grievance" => Self::Grievance,
            "group" => Self::Group,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// The sites within a Euclidean radius, as offsets, excluding the center:
/// `dx² + dy² ≤ radius²`, row by row (`dy`, then `dx`, ascending).
pub fn sight(radius: f64) -> Vec<(i32, i32)> {
    let r = radius.floor() as i32;
    let r2 = radius * radius;
    let mut out = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            if (dx, dy) != (0, 0) && f64::from(dx * dx + dy * dy) <= r2 {
                out.push((dx, dy));
            }
        }
    }
    out
}

/// What each site sees at one vision radius: its own index, then the sites
/// at `sight(radius)`'s offsets, for every site (computed once per radius).
struct View {
    radius: f64,
    stride: usize,
    idx: Vec<u32>,
}

impl View {
    fn new(torus: Torus, radius: f64) -> Self {
        let offsets = sight(radius);
        let stride = offsets.len() + 1;
        let mut idx = Vec::with_capacity(torus.len() * stride);
        for i in 0..torus.len() {
            let p = torus.pos(i);
            idx.push(i as u32);
            for &(dx, dy) in &offsets {
                idx.push(torus.index(torus.offset(p, dx, dy)) as u32);
            }
        }
        View {
            radius,
            stride,
            idx,
        }
    }

    /// Site `i` itself, then the sites it sees.
    fn of(&self, i: usize) -> &[u32] {
        &self.idx[i * self.stride..(i + 1) * self.stride]
    }
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CivilInspection {
    pub site: SiteXy,
    pub agent: Option<CitizenView>,
    pub cop: Option<CopView>,
    /// Jailed agents arrested on this site (where they wait with
    /// `jailed_stay`, and where Inspect finds an agent it follows).
    pub jailed: Vec<CitizenView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SiteXy {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CopView {
    pub id: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CitizenView {
    pub id: u64,
    /// "quiet", "active" or "jailed".
    pub state: &'static str,
    pub hardship: f64,
    pub risk_aversion: f64,
    pub grievance: f64,
    /// The arrest probability P it would estimate here now, and N = R·P.
    pub arrest_probability: f64,
    pub net_risk: f64,
    /// Ticks of jail left, or null (free, or a term that never ends).
    pub jail_left: Option<u32>,
    pub jail_life: bool,
    /// Model II: "blue" or "green", its age and death age; null in Model I.
    pub group: Option<&'static str>,
    pub age: Option<u32>,
    pub death_age: Option<u32>,
}

pub struct CivilWorld {
    pub config: CivilConfig,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    agents: Vec<Citizen>,
    cops: Vec<Cop>,
    /// Free agents and cops on each site (row-major); jailed agents are on
    /// no site.
    sites: Vec<Vec<Occupant>>,
    agent_view: View,
    cop_view: View,
    /// A reusable buffer for candidate sites.
    scratch: Vec<usize>,
    /// Each ramp's value when its start tick began (config order).
    ramp_base: Vec<Option<f64>>,
    rng: SimRng,
    next_id: u64,
    killed: u32,
    outbursts: Outbursts,
    extinct_at: Option<u64>,
    pub stats: Stats<CivilSnapshot>,
}

impl CivilWorld {
    pub fn new(config: CivilConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let torus = Torus::new(config.width, config.height);
        let mut w = CivilWorld {
            torus,
            tick: 0,
            agents: Vec::new(),
            cops: Vec::new(),
            sites: vec![Vec::new(); torus.len()],
            agent_view: View::new(torus, config.vision.agent),
            cop_view: View::new(torus, config.vision.cop),
            scratch: Vec::new(),
            ramp_base: vec![None; config.ramps.len()],
            rng: rng::seeded(seed),
            next_id: 1,
            killed: 0,
            outbursts: Outbursts::default(),
            extinct_at: None,
            stats: Stats::default(),
            config,
        };
        // Cops first, then agents, each on a random empty site: the first
        // sites of one shuffle.
        let mut cells: Vec<u32> = (0..torus.len() as u32).collect();
        cells.shuffle(&mut w.rng);
        let (nc, na) = (w.config.cops() as usize, w.config.agents() as usize);
        let cop_cells = cells[..nc].to_vec();
        for &i in &cells[nc..nc + na] {
            let a = w.newcomer(torus.pos(i as usize));
            w.agents.push(a);
        }
        for i in cop_cells {
            let id = w.take_id();
            w.cops.push(Cop {
                id,
                pos: torus.pos(i as usize),
            });
        }
        w.rebuild_sites();
        w.record();
        Ok(w)
    }

    fn take_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// A quiet, free agent at `pos` with H and R from U(0,1); in Model II
    /// Blue or Green with probability ½, age 0 and a death age uniform in
    /// 1..=max_age.
    fn newcomer(&mut self, pos: Pos) -> Citizen {
        let id = self.take_id();
        let hardship = self.rng.gen::<f64>();
        let risk_aversion = self.rng.gen::<f64>();
        let (green, death_age) = match self.config.variant {
            Variant::Rebellion => (false, 0),
            Variant::Ethnic => (
                self.rng.gen_bool(0.5),
                self.rng.gen_range(1..=self.config.max_age),
            ),
        };
        Citizen {
            id,
            pos,
            hardship,
            risk_aversion,
            active: false,
            jail: None,
            green,
            age: 0,
            death_age,
            alive: true,
        }
    }

    /// Model II: agent `i`'s clone at `pos`, keeping its group and H,
    /// drawing R and a death age.
    fn child(&mut self, i: usize, pos: Pos) -> Citizen {
        let id = self.take_id();
        let risk_aversion = self.rng.gen::<f64>();
        let death_age = self.rng.gen_range(1..=self.config.max_age);
        let parent = &self.agents[i];
        Citizen {
            id,
            pos,
            hardship: parent.hardship,
            risk_aversion,
            active: false,
            jail: None,
            green: parent.green,
            age: 0,
            death_age,
            alive: true,
        }
    }

    /// Files every free, living agent and every cop on its site, agents
    /// first, each in index order.
    fn rebuild_sites(&mut self) {
        for s in &mut self.sites {
            s.clear();
        }
        for (i, a) in self.agents.iter().enumerate() {
            if a.alive && a.jail.is_none() {
                self.sites[self.torus.index(a.pos)].push(Occupant::Agent(i as u32));
            }
        }
        for (j, c) in self.cops.iter().enumerate() {
            self.sites[self.torus.index(c.pos)].push(Occupant::Cop(j as u32));
        }
    }

    fn site(&self, pos: Pos) -> usize {
        self.torus.index(pos)
    }

    /// No free agent and no cop on it (jailed agents do not count).
    fn is_empty(&self, i: usize) -> bool {
        self.sites[i].is_empty()
    }

    fn leave(&mut self, i: usize, who: Occupant) {
        let list = &mut self.sites[i];
        let at = list.iter().position(|&o| o == who).expect("on its site");
        list.remove(at);
    }

    /// A uniformly random empty site among those site `i` sees with cop
    /// (`cop`) or agent vision, counting site `i` itself when `own`.
    fn empty_in_view(&mut self, i: usize, cop: bool, own: bool) -> Option<usize> {
        let mut buf = std::mem::take(&mut self.scratch);
        buf.clear();
        let view = if cop {
            &self.cop_view
        } else {
            &self.agent_view
        };
        let seen = view.of(i);
        let seen = if own { seen } else { &seen[1..] };
        buf.extend(
            seen.iter()
                .map(|&s| s as usize)
                .filter(|&s| self.sites[s].is_empty()),
        );
        let out = (!buf.is_empty()).then(|| buf[self.rng.gen_range(0..buf.len() as u32) as usize]);
        self.scratch = buf;
        out
    }

    /// A uniformly random empty site among `pos + offsets` (not `pos`).
    fn random_empty(&mut self, pos: Pos, offsets: &[(i32, i32)]) -> Option<usize> {
        let torus = self.torus;
        let empty: Vec<usize> = offsets
            .iter()
            .map(|&(dx, dy)| torus.index(torus.offset(pos, dx, dy)))
            .filter(|&i| self.is_empty(i))
            .collect();
        (!empty.is_empty()).then(|| empty[self.rng.gen_range(0..empty.len() as u32) as usize])
    }

    /// Rebuilds the vision tables whose radius changed.
    fn refresh_views(&mut self) {
        let v = self.config.vision;
        if v.agent != self.agent_view.radius {
            self.agent_view = View::new(self.torus, v.agent);
        }
        if v.cop != self.cop_view.radius {
            self.cop_view = View::new(self.torus, v.cop);
        }
    }

    /// A uniformly random empty site anywhere.
    fn random_empty_anywhere(&mut self) -> Option<usize> {
        let empty: Vec<usize> = (0..self.sites.len())
            .filter(|&i| self.is_empty(i))
            .collect();
        (!empty.is_empty()).then(|| empty[self.rng.gen_range(0..empty.len() as u32) as usize])
    }

    pub fn agents(&self) -> impl Iterator<Item = &Citizen> {
        self.agents.iter().filter(|a| a.alive)
    }

    pub fn cops(&self) -> &[Cop] {
        &self.cops
    }

    /// G = H(1 − L).
    pub fn grievance(&self, a: &Citizen) -> f64 {
        a.hardship * (1.0 - self.config.legitimacy)
    }

    /// The arrest probability agent `i` estimates where it stands:
    /// P = 1 − exp(−k·C/A), C the cops and A one plus the other active
    /// agents it sees (with `floor_ratio`, NetLogo's ⌊C/A⌋ with an active
    /// agent counted twice).
    fn arrest_probability(&self, i: usize) -> f64 {
        let a = &self.agents[i];
        let (mut cops, mut actives) = (0u32, 0u32);
        for &s in self.agent_view.of(self.site(a.pos)) {
            for &o in &self.sites[s as usize] {
                match o {
                    Occupant::Cop(_) => cops += 1,
                    Occupant::Agent(j) if j as usize != i && self.agents[j as usize].active => {
                        actives += 1
                    }
                    Occupant::Agent(_) => {}
                }
            }
        }
        let q = self.config.quirks;
        let mut a_count = 1 + actives;
        if q.active_counts_twice && a.active {
            a_count += 1;
        }
        let mut ratio = f64::from(cops) / f64::from(a_count);
        if q.floor_ratio {
            ratio = ratio.floor();
        }
        1.0 - exp_neg(-self.config.k * ratio)
    }

    /// Rule M for agent `i`.
    fn move_agent(&mut self, i: usize) {
        let from = self.agents[i].pos;
        if let Some(to) = self.empty_in_view(self.site(from), false, false) {
            let who = Occupant::Agent(i as u32);
            self.leave(self.site(from), who);
            self.sites[to].push(who);
            self.agents[i].pos = self.torus.pos(to);
        }
    }

    /// Rule M for cop `j`.
    fn move_cop(&mut self, j: usize) {
        let from = self.cops[j].pos;
        if let Some(to) = self.empty_in_view(self.site(from), true, false) {
            let who = Occupant::Cop(j as u32);
            self.leave(self.site(from), who);
            self.sites[to].push(who);
            self.cops[j].pos = self.torus.pos(to);
        }
    }

    /// Rule A for agent `i`: active iff G − R·P > T.
    fn decide(&mut self, i: usize) {
        let p = self.arrest_probability(i);
        let a = &self.agents[i];
        let active = self.grievance(a) - a.risk_aversion * p > self.config.threshold;
        self.agents[i].active = active;
    }

    /// Model II: an active agent kills one uniformly random free agent of
    /// the other group that it sees, if any.
    fn kill(&mut self, i: usize) {
        let a = &self.agents[i];
        let green = a.green;
        let targets: Vec<u32> = self
            .agent_view
            .of(self.site(a.pos))
            .iter()
            .flat_map(|&s| self.sites[s as usize].iter())
            .filter_map(|&o| match o {
                Occupant::Agent(j) if self.agents[j as usize].green != green => Some(j),
                _ => None,
            })
            .collect();
        if targets.is_empty() {
            return;
        }
        let victim = targets[self.rng.gen_range(0..targets.len() as u32) as usize] as usize;
        let at = self.site(self.agents[victim].pos);
        self.leave(at, Occupant::Agent(victim as u32));
        self.agents[victim].alive = false;
        self.killed += 1;
    }

    /// Rule C for cop `j`: arrest one uniformly random active agent it
    /// sees, if any.
    fn arrest(&mut self, j: usize) {
        let pos = self.cops[j].pos;
        let suspects: Vec<u32> = self
            .cop_view
            .of(self.site(pos))
            .iter()
            .flat_map(|&s| self.sites[s as usize].iter())
            .filter_map(|&o| match o {
                Occupant::Agent(i) if self.agents[i as usize].active => Some(i),
                _ => None,
            })
            .collect();
        if suspects.is_empty() {
            return;
        }
        let i = suspects[self.rng.gen_range(0..suspects.len() as u32) as usize] as usize;
        let term = self.jail_term();
        let at = self.agents[i].pos;
        self.leave(self.site(at), Occupant::Agent(i as u32));
        let a = &mut self.agents[i];
        a.active = false;
        a.jail = Some(term);
        if self.config.quirks.cop_moves_to_arrest {
            let who = Occupant::Cop(j as u32);
            self.leave(self.site(pos), who);
            let to = self.site(at);
            self.sites[to].push(who);
            self.cops[j].pos = at;
        }
    }

    /// ⌈U(0, J_max)⌉ ticks, i.e. uniform in 1..=J_max (NetLogo: 0..J_max),
    /// or a term that never ends.
    fn jail_term(&mut self) -> Term {
        let j = self.config.jail;
        if j.infinite {
            Term::Life
        } else if self.config.quirks.netlogo_jail_term {
            Term::Ticks(self.rng.gen_range(0..j.max))
        } else {
            Term::Ticks(self.rng.gen_range(1..=j.max))
        }
    }

    /// Every jailed agent's term counts down by one; those at 0 are
    /// released in index order — near where they were arrested, or where
    /// they stand with `jailed_stay`.
    fn serve_terms(&mut self) {
        for i in 0..self.agents.len() {
            let a = &mut self.agents[i];
            if !a.alive {
                continue;
            }
            let Some(Term::Ticks(left)) = a.jail else {
                continue;
            };
            let left = left.saturating_sub(1);
            a.jail = Some(Term::Ticks(left));
            if left == 0 {
                self.release(i);
            }
        }
    }

    /// Frees agent `i` (its term is served): with `jailed_stay` where it
    /// stands; otherwise on a random empty site among its arrest site and
    /// the sites it sees from there, else a random empty site anywhere, else
    /// it waits (still jailed, with 0 ticks left).
    fn release(&mut self, i: usize) {
        let pos = self.agents[i].pos;
        let to = if self.config.quirks.jailed_stay {
            Some(self.site(pos))
        } else {
            self.empty_in_view(self.site(pos), false, true)
                .or_else(|| self.random_empty_anywhere())
        };
        if let Some(to) = to {
            self.sites[to].push(Occupant::Agent(i as u32));
            let a = &mut self.agents[i];
            a.jail = None;
            a.pos = self.torus.pos(to);
        }
    }

    /// Model II: in a random order each free agent clones with probability
    /// p onto a random empty Moore neighbor; the child keeps its group and
    /// H, draws R and a death age. Then every agent but the newborn ages,
    /// and dies on reaching its death age.
    fn demography(&mut self) {
        let mut order: Vec<usize> = (0..self.agents.len())
            .filter(|&i| self.agents[i].alive && self.agents[i].jail.is_none())
            .collect();
        order.shuffle(&mut self.rng);
        let born_from = self.agents.len();
        const MOORE: [(i32, i32); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        let p = self.config.clone_probability;
        for i in order {
            if !self.rng.gen_bool(p) {
                continue;
            }
            let Some(to) = self.random_empty(self.agents[i].pos, &MOORE) else {
                continue;
            };
            let child = self.child(i, self.torus.pos(to));
            self.sites[to].push(Occupant::Agent(self.agents.len() as u32));
            self.agents.push(child);
        }
        for a in &mut self.agents[..born_from] {
            if !a.alive {
                continue;
            }
            a.age += 1;
            if a.age >= a.death_age {
                a.alive = false;
            }
        }
    }

    /// Applies schedule entries due at the tick about to run, then ramps.
    fn apply_changes(&mut self) {
        let t = self.tick;
        let due: Vec<_> = self
            .config
            .schedule
            .iter()
            .filter(|c| c.tick == t)
            .flat_map(|c| c.set.clone())
            .collect();
        for (path, value) in due {
            if let Ok(ModelConfig::Civil(next)) =
                ModelConfig::Civil(self.config.clone()).with_path(&path, &value)
            {
                self.config = next;
            }
        }
        for k in 0..self.config.ramps.len() {
            let r = self.config.ramps[k].clone();
            if t == r.start {
                self.ramp_base[k] = Some(self.config.number(&r.path));
            }
            if let Some(base) = self.ramp_base[k] {
                if t > r.start && t <= r.end {
                    let f = (t - r.start) as f64 / (r.end - r.start) as f64;
                    self.config.set_number(&r.path, base + (r.to - base) * f);
                }
            }
        }
        self.refresh_views();
    }

    /// Adds cops on random empty sites or removes random cops until there
    /// are round(cop density × sites) (as many as fit).
    fn sync_cops(&mut self) {
        let target = self.config.cops() as usize;
        if self.cops.len() == target {
            return;
        }
        while self.cops.len() > target {
            let j = self.rng.gen_range(0..self.cops.len() as u32) as usize;
            self.cops.remove(j);
        }
        let mut empty: Vec<usize> = (0..self.sites.len())
            .filter(|&i| self.is_empty(i))
            .collect();
        while self.cops.len() < target && !empty.is_empty() {
            let k = self.rng.gen_range(0..empty.len() as u32) as usize;
            let i = empty.swap_remove(k);
            let id = self.take_id();
            self.cops.push(Cop {
                id,
                pos: self.torus.pos(i),
            });
            // Filed now so the next pick sees it taken.
            self.sites[i].push(Occupant::Cop(u32::MAX));
        }
        self.rebuild_sites();
    }

    pub fn is_finished(&self) -> bool {
        self.config.variant == Variant::Ethnic
            && self.config.stop_at_extinction
            && self.extinct_at.is_some()
    }

    /// One tick.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        self.apply_changes();
        self.sync_cops();
        self.killed = 0;
        #[derive(Clone, Copy)]
        enum Actor {
            Agent(usize),
            Cop(usize),
        }
        let mut order: Vec<Actor> = (0..self.agents.len())
            .filter(|&i| self.agents[i].jail.is_none())
            .map(Actor::Agent)
            .chain((0..self.cops.len()).map(Actor::Cop))
            .collect();
        order.shuffle(&mut self.rng);
        let ethnic = self.config.variant == Variant::Ethnic;
        for actor in order {
            match actor {
                Actor::Agent(i) => {
                    let a = &self.agents[i];
                    if !a.alive || a.jail.is_some() {
                        continue;
                    }
                    if self.config.movement {
                        self.move_agent(i);
                    }
                    self.decide(i);
                    if ethnic && self.agents[i].active {
                        self.kill(i);
                    }
                }
                Actor::Cop(j) => {
                    self.move_cop(j);
                    self.arrest(j);
                }
            }
        }
        self.serve_terms();
        if ethnic {
            self.demography();
        }
        self.agents.retain(|a| a.alive);
        self.rebuild_sites();
        self.tick += 1;
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    fn record(&mut self) {
        let mut s = CivilSnapshot {
            tick: self.tick,
            cops: self.cops.len() as u32,
            legitimacy: self.config.legitimacy,
            killed: self.killed,
            ..Default::default()
        };
        let (mut g, mut r, mut free) = (0.0, 0.0, 0u32);
        for a in self.agents() {
            s.population += 1;
            if a.green {
                s.green += 1;
            } else {
                s.blue += 1;
            }
            if a.jail.is_some() {
                s.jailed += 1;
                continue;
            }
            free += 1;
            g += self.grievance(a);
            r += a.risk_aversion;
            if a.active {
                s.active += 1;
            } else {
                s.quiet += 1;
            }
        }
        if self.config.variant == Variant::Rebellion {
            (s.blue, s.green) = (0, 0);
        }
        if free > 0 {
            let n = f64::from(free);
            s.mean_grievance = g / n;
            let quiet_share = f64::from(s.quiet) / n;
            s.tension = if r > 0.0 {
                (g / n) * quiet_share / (r / n)
            } else {
                0.0
            };
        }
        self.outbursts
            .record(self.tick, s.active, self.config.outburst_threshold);
        s.outbursts = self.outbursts.ended;
        s.mean_wait = self.outbursts.mean_wait();
        s.mean_activation = self.outbursts.mean_activation();
        if self.config.variant == Variant::Ethnic
            && self.extinct_at.is_none()
            && (s.blue == 0 || s.green == 0)
        {
            self.extinct_at = Some(self.tick);
        }
        s.extinction = self.extinct_at.unwrap_or(self.tick);
        self.stats.push(s);
    }

    fn color(&self, a: &Citizen, mode: CivilMode) -> Rgb {
        let group = if a.green { GREEN } else { BLUE };
        match mode {
            CivilMode::Action if a.active => RED,
            CivilMode::Action | CivilMode::Group => group,
            CivilMode::Grievance => lerp(BACKGROUND, RED, 0.15 + 0.85 * self.grievance(a)),
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<CivilInspection, String> {
        if x >= self.torus.width || y >= self.torus.height {
            return Err(format!("({x}, {y}) is outside the grid"));
        }
        let pos = Pos::new(x, y);
        let here = &self.sites[self.site(pos)];
        let agent = here.iter().find_map(|&o| match o {
            Occupant::Agent(i) => Some(i as usize),
            Occupant::Cop(_) => None,
        });
        let cop = here.iter().find_map(|&o| match o {
            Occupant::Cop(j) => Some(CopView {
                id: self.cops[j as usize].id,
            }),
            Occupant::Agent(_) => None,
        });
        let jailed = (0..self.agents.len())
            .filter(|&i| {
                let a = &self.agents[i];
                a.alive && a.jail.is_some() && a.pos == pos
            })
            .map(|i| self.view(i))
            .collect();
        Ok(CivilInspection {
            site: SiteXy { x, y },
            agent: agent.map(|i| self.view(i)),
            cop,
            jailed,
        })
    }

    fn view(&self, i: usize) -> CitizenView {
        let a = &self.agents[i];
        let p = if a.jail.is_none() {
            self.arrest_probability(i)
        } else {
            0.0
        };
        let ethnic = self.config.variant == Variant::Ethnic;
        CitizenView {
            id: a.id,
            state: match (a.jail, a.active) {
                (Some(_), _) => "jailed",
                (None, true) => "active",
                (None, false) => "quiet",
            },
            hardship: a.hardship,
            risk_aversion: a.risk_aversion,
            grievance: self.grievance(a),
            arrest_probability: p,
            net_risk: a.risk_aversion * p,
            jail_left: match a.jail {
                Some(Term::Ticks(t)) => Some(t),
                _ => None,
            },
            jail_life: a.jail == Some(Term::Life),
            group: ethnic.then_some(if a.green { "green" } else { "blue" }),
            age: ethnic.then_some(a.age),
            death_age: ethnic.then_some(a.death_age),
        }
    }
}

impl Model for CivilWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Civil(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        CivilWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents().count()
    }

    /// FNV-1a over the tick, every agent (id, site, H, R, state, group, age
    /// and death age), every cop (id, site) and the ramps' starting values.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for a in self.agents() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(a.hardship.to_bits());
            eat(a.risk_aversion.to_bits());
            let jail = match a.jail {
                None => 0,
                Some(Term::Ticks(t)) => 1 + u64::from(t),
                Some(Term::Life) => u64::MAX,
            };
            eat(jail);
            eat(u64::from(a.active) | u64::from(a.green) << 1);
            eat((u64::from(a.age) << 32) | u64::from(a.death_age));
        }
        for c in &self.cops {
            eat(c.id);
            eat((u64::from(c.pos.x) << 32) | u64::from(c.pos.y));
        }
        for b in &self.ramp_base {
            eat(b.map_or(u64::MAX, f64::to_bits));
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.torus.width, self.torus.height)
    }

    /// Empty sites dark; cops light gray; agents by `mode`. `layer` is
    /// ignored.
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: CivilMode = mode.parse()?;
        buf.resize(self.sites.len() * 4, 0);
        for (i, here) in self.sites.iter().enumerate() {
            let rgb = if here.iter().any(|o| matches!(o, Occupant::Cop(_))) {
                COP
            } else {
                match here.first() {
                    Some(&Occupant::Agent(a)) => self.color(&self.agents[a as usize], mode),
                    _ => BACKGROUND,
                }
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
        let mut out = String::from(
            "id,x,y,state,jail_left,hardship,risk_aversion,grievance,group,age,death_age\n",
        );
        let ethnic = self.config.variant == Variant::Ethnic;
        for (i, a) in self.agents.iter().enumerate() {
            if !a.alive {
                continue;
            }
            let v = self.view(i);
            let jail = match a.jail {
                None => String::new(),
                Some(Term::Ticks(t)) => t.to_string(),
                Some(Term::Life) => "life".to_string(),
            };
            let (group, age, death) = if ethnic {
                (
                    v.group.unwrap_or_default().to_string(),
                    a.age.to_string(),
                    a.death_age.to_string(),
                )
            } else {
                Default::default()
            };
            writeln!(
                out,
                "{},{},{},{},{},{},{},{},{},{},{}",
                a.id,
                a.pos.x,
                a.pos.y,
                v.state,
                jail,
                a.hardship,
                a.risk_aversion,
                v.grievance,
                group,
                age,
                death
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// An agent's site (while jailed, its arrest site) or a cop's.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        self.agents()
            .find(|a| a.id == id)
            .map(|a| a.pos)
            .or_else(|| self.cops.iter().find(|c| c.id == id).map(|c| c.pos))
            .map(|p| (p.x, p.y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Civil(next) = next else {
            return Err(wrong_model(ModelKind::Civil, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        self.refresh_views();
        self.sync_cops();
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }
}
```

- [ ] **Step 7: The presets and the module**

`crates/sugarscape-core/src/civil/presets.rs` (Decisions 13, 14 — the descriptions carry the measurements):

```rust
//! The paper's runs (Table 2) and scenarios, and NetLogo's defaults.

use super::config::{CivilConfig, Jail, Quirks, Ramp, Variant, Vision};
use crate::config::ScheduledChange;
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut CivilConfig),
) -> ModelPreset {
    let mut c = CivilConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Civil(c),
    }
}

/// A Model I column of Table 2 — cop and agent vision, legitimacy, J_max
/// (None: terms never end), movement, initial cop density — with the arrest
/// ratio rounded down, without which the paper's rule never rebels (the
/// spec's Finding).
fn model_one(
    c: &mut CivilConfig,
    vision: f64,
    legitimacy: f64,
    jail: Option<u32>,
    movement: bool,
    cops: f64,
) {
    c.variant = Variant::Rebellion;
    c.vision = Vision {
        agent: vision,
        cop: vision,
    };
    c.legitimacy = legitimacy;
    c.jail = match jail {
        Some(max) => Jail {
            max,
            infinite: false,
        },
        None => Jail {
            max: 30,
            infinite: true,
        },
    };
    c.movement = movement;
    c.cop_density = cops;
    c.quirks.floor_ratio = true;
}

/// A Model II column of Table 2: vision 1.7, J_max 15, random movement,
/// the given legitimacy and initial cop density, the paper's arrest rule.
fn model_two(c: &mut CivilConfig, legitimacy: f64, cops: f64) {
    c.variant = Variant::Ethnic;
    c.vision = Vision {
        agent: 1.7,
        cop: 1.7,
    };
    c.legitimacy = legitimacy;
    c.jail = Jail {
        max: 15,
        infinite: false,
    };
    c.cop_density = cops;
}

fn at(tick: u64, path: &str, value: serde_json::Value) -> ScheduledChange {
    ScheduledChange {
        tick,
        set: [(path.to_string(), value)].into_iter().collect(),
    }
}

const SOURCE: &str = "Epstein 2002, PNAS 99 suppl. 3";

pub fn presets() -> Vec<ModelPreset> {
    use serde_json::json;
    vec![
        preset(
            "cv-run-1-no-movement",
            "Run 1 — no movement",
            SOURCE,
            "Table 2's Run 1: vision 1.7 (the eight neighbors), L = 0.89, J_max = 15, cops 0.04, and agents that do not move (cops still do), with C/A rounded down — without it the paper's stated rule gives Model I no rebellion (the sweep cv-ratio-rules). The paper shows this society's deceptive agents and local outbursts (Figs. 1–2): agents turn red when the cops near them move on. Measured (seeds 1–20): a steady simmer of 33–62 actives (mean 48 over t = 100–700) with no calm stretches.",
            |c| model_one(c, 1.7, 0.89, Some(15), false, 0.04),
        ),
        preset(
            "cv-run-2-punctuated",
            "Run 2 — punctuated equilibrium",
            SOURCE,
            "Table 2's Run 2 (vision 7, L = 0.82, J_max = 30, cops 0.04) with C/A rounded down, as NetLogo's Rebellion does. With the paper's stated P = 1 − exp(−k·C/A) no seed of 20 has an outburst in 3000 ticks (at most 13–34 actives): a crowd that outnumbers the cops still faces P ≈ 0.5, which outweighs every grievance at L = 0.82. Rounded down, P is 0 once actives outnumber the cops in view, and the paper's punctuated equilibrium appears (Figs. 3–4). Measured (seeds 1–20, 3000 ticks): 101–122 outbursts above 50 actives, peaks of 290–368, fewer than 10 actives on 61–68% of ticks; mean total activation 779 per outburst (the paper's Fig. 7: 708 ± 230) and a mean wait of 22 ticks between outbursts (Fig. 5: 60 ± 55), a third of the paper's.",
            |c| model_one(c, 7.0, 0.82, Some(30), true, 0.04),
        ),
        preset(
            "cv-run-3-salami",
            "Run 3 — salami tactics",
            SOURCE,
            "Runs 3 and 4's inputs (vision 7, L = 0.9, terms that never end, cops 0.074) with C/A rounded down; from t = 77 legitimacy falls 0.01 a tick until it reaches 0.2 at t = 147. The paper (Fig. 9): no spike of actives, and a jail that fills smoothly, because each new rebel is picked off alone. Measured (seeds 1–20, paired with Run 4): 17 seeds stay under 50 actives while the jail fills (322–910 jailed by t = 300); 3 explode. Open Salami tactics vs one jump in the presets menu to run both.",
            |c| {
                model_one(c, 7.0, 0.9, None, true, 0.074);
                c.ramps = vec![Ramp {
                    path: "legitimacy".into(),
                    start: 77,
                    end: 147,
                    to: 0.2,
                }];
            },
        ),
        preset(
            "cv-run-4-one-jump",
            "Run 4 — one jump",
            SOURCE,
            "Runs 3 and 4's inputs with C/A rounded down; legitimacy drops from 0.9 to 0.7 in one step at t = 77. The paper (Fig. 10): an explosion of actives, and a jail that ends larger than Run 3's although legitimacy fell far less. Measured (seeds 1–20, paired with Run 3): the jump's peak beats salami tactics' in 17 seeds and passes 50 actives in 9 (salami: 3), but its jail ends larger in only 7 — the explosion reproduces about half the time, the larger jail does not.",
            |c| {
                model_one(c, 7.0, 0.9, None, true, 0.074);
                c.schedule = vec![at(77, "legitimacy", json!(0.7))];
            },
        ),
        preset(
            "cv-run-5-cop-reductions",
            "Run 5 — cop reductions",
            SOURCE,
            "Table 2's Run 5 (vision 7, L = 0.8, terms that never end, cops 0.074) with C/A rounded down; from t = 50 the cops are walked down in a straight line to none at t = 550. The paper (Fig. 11): unlike falling legitimacy, a marginal cut in cops tips the society into rebellion. Measured (seeds 1–20): every run tips — a peak of 64–345 actives (mean 194) at a mean t = 177, with about 88 of the 118 cops left, and most rebels jailed for good. With the paper's stated rule no run tips (peaks of 14–33).",
            |c| {
                model_one(c, 7.0, 0.8, None, true, 0.074);
                c.ramps = vec![Ramp {
                    path: "cop_density".into(),
                    start: 50,
                    end: 550,
                    to: 0.0,
                }];
            },
        ),
        preset(
            "cv-run-6-coexistence",
            "Run 6 — peaceful coexistence",
            SOURCE,
            "Table 2's Run 6: Model II (Blue and Green; going active means killing a member of the other group), vision 1.7, L = 0.9, no cops, cloning 0.05, death ages up to 200. The paper (Fig. 12): peaceful coexistence. At L = 0.9 grievance never exceeds the 0.1 threshold, so nobody ever goes active: no killing in 20 of 20 seeds.",
            |c| model_two(c, 0.9, 0.0),
        ),
        preset(
            "cv-run-7-cleansing",
            "Run 7 — ethnic cleansing",
            SOURCE,
            "Table 2's Run 7: Model II, L = 0.8, no cops, stopping once a group is gone. The paper (Fig. 13): local ethnic cleansing, and genocide in each of 30 runs with a random victor. Measured (seeds 1–20): genocide in every seed at t = 30–190 (mean 94); Blue survived 12 times, Green 8.",
            |c| {
                model_two(c, 0.8, 0.0);
                c.stop_at_extinction = true;
            },
        ),
        preset(
            "cv-run-8-nasty-regime",
            "Run 8 — cops from the start",
            SOURCE,
            "Table 2's Run 8: Model II, L = 0.8, cops 0.04 from the start. The paper's text says this gives 'a stable, but nasty, regime' in which the cops keep both groups alive; its own Fig. 15 shows rapid genocide at every cop density. Measured (seeds 1–20): one group is gone in every seed by t = 75–882 (mean 180) — the stable regime does not reproduce.",
            |c| model_two(c, 0.8, 0.04),
        ),
        preset(
            "cv-safe-havens",
            "Peacekeepers at t = 50",
            SOURCE,
            "Run 7 with peacekeepers deployed at t = 50 on random empty sites, stopping once a group is gone. The paper (Fig. 14) says this typically produces safe havens but gives no density; 0.04 is the density it calls high. Measured (seeds 1–20): genocide in every seed by t = 30–377 (mean 131). Denser forces only delay it: at 0.1, 0.2 and 0.3 (seeds 1–5) a group is still gone by t = 76–1475.",
            |c| {
                model_two(c, 0.8, 0.0);
                c.stop_at_extinction = true;
                c.schedule = vec![at(50, "cop_density", json!(0.04))];
            },
        ),
        preset(
            "cv-netlogo",
            "NetLogo Rebellion",
            "Wilensky 2004, NetLogo Rebellion",
            "NetLogo Rebellion's defaults (70% agents, 4% cops, vision 7, L = 0.82, J_max = 30) with all five of its departures from the paper: C/A rounded down, an active agent counted twice, the arresting cop stepping onto the arrest, jailed agents keeping their site, and terms of 0 to J_max − 1 ticks. Measured (seeds 1–20, 3000 ticks): 93–109 outbursts, mean total activation 1107, mean wait 23 ticks.",
            |c| c.quirks = Quirks::ALL,
        ),
    ]
}
```

Replace `crates/sugarscape-core/src/civil/mod.rs`:

```rust
//! Epstein's civil violence model (milestone 11): "Modeling civil
//! violence: An agent-based computational approach", PNAS 99 suppl. 3
//! (2002). Model I — agents rebel against a central authority whose cops
//! arrest them — and Model II — two ethnic groups whose active members
//! kill each other, with cloning and death by age — as one model kind,
//! with NetLogo *Rebellion*'s departures as named switches. See
//! docs/superpowers/specs/2026-09-25-civil-violence-design.md.

mod config;
mod math;
mod presets;
mod stats;
mod world;

pub use config::{
    is_live, schema, CivilConfig, Jail, Quirks, Ramp, Variant, Vision, LIVE, RAMPABLE,
};
pub use math::exp_neg;
pub use presets::presets;
pub use stats::{CivilSnapshot, Outbursts, SERIES};
pub use world::{
    sight, Citizen, CitizenView, CivilInspection, CivilMode, CivilWorld, Cop, CopView, SiteXy,
    Term, COP, GREEN,
};
```

- [ ] **Step 8: Run the tests to see them pass**

Run: `cargo test -p sugarscape-core --lib civil && cargo test -p sugarscape-core --lib model:: && cargo test -p sugarscape-core --test golden`
Expected: PASS — 32 civil tests (3 from Task 1), the model tests, and every golden entry including the ten new ones. If a fingerprint differs, stop and report (Decision 15).

Then the whole workspace: `cargo test --workspace`
Expected: PASS. (The WASM crate's `Sim` dispatches through `ModelWorld` and needs no change.)

- [ ] **Step 9: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/civil crates/sugarscape-core/src/schema.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs
git commit -m "Run Epstein's civil violence, Models I and II, as the civil model kind" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 3: Measured against the paper — book-style tests, three sweeps, the CLI and WASM

**Files:**
- Create: `crates/sugarscape-core/tests/civil.rs`, `sweeps/cv-ratio-rules.json`, `sweeps/cv-peacekeeping.json`, `sweeps/cv-jail-waits.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 2's presets, `CivilConfig`, `ModelWorld`, `Model::{series, finished, tick}`.
- Produces: built-in sweep ids `cv-ratio-rules`, `cv-peacekeeping`, `cv-jail-waits` (after `lhv-quirks`).

- [ ] **Step 1: The book-style tests**

Create `crates/sugarscape-core/tests/civil.rs` (Decision 14's thresholds; the paper's claims that fail are pinned as measured):

```rust
//! Epstein's civil violence (milestone 11) against the paper, over seeds
//! 1–20 in release: `cargo test -p sugarscape-core --release --test civil
//! -- --ignored`. Thresholds come from the measurements recorded
//! 2026-09-25 (docs/superpowers/plans/2026-09-25-civil-violence.md,
//! Decision 14), each with room around the measured range; the claims the
//! paper makes that do not reproduce are pinned as measured, so a change
//! that would reproduce them shows up here.

use std::thread;

use sugarscape_core::civil::CivilConfig;
use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::presets;

const SEEDS: u64 = 20;

fn config(id: &str) -> CivilConfig {
    match presets::find(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
    {
        ModelConfig::Civil(c) => c,
        _ => panic!("{id} is not a civil preset"),
    }
}

/// Runs `c` for `ticks` from seeds 1–20 in parallel and measures each run.
fn each_seed<T: Send>(c: &CivilConfig, ticks: u32, f: impl Fn(&ModelWorld) -> T + Sync) -> Vec<T> {
    thread::scope(|s| {
        let handles: Vec<_> = (1..=SEEDS)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = ModelWorld::new(ModelConfig::Civil(c.clone()), seed).unwrap();
                    w.model_mut().run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn series(w: &ModelWorld, name: &str) -> Vec<f64> {
    w.model().series(name).unwrap()
}

fn last(w: &ModelWorld, name: &str) -> f64 {
    *series(w, name).last().unwrap()
}

fn max(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn count(v: &[bool]) -> usize {
    v.iter().filter(|&&b| b).count()
}

#[test]
#[ignore]
fn run_2_is_punctuated_only_with_the_ratio_rounded_down() {
    // Measured: 101–122 outbursts in 3000 ticks, mean activation 779 (paper
    // 708 ± 230), mean wait 21.8 (paper 60 ± 55), 61–68% of ticks calm.
    let c = config("cv-run-2-punctuated");
    let runs = each_seed(&c, 3000, |w| {
        let a = series(w, "active");
        (
            last(w, "outbursts"),
            last(w, "mean_activation"),
            last(w, "mean_wait"),
            a.iter().filter(|&&x| x < 10.0).count() as f64 / a.len() as f64,
        )
    });
    for (seed, &(outbursts, activation, wait, calm)) in runs.iter().enumerate() {
        assert!(
            outbursts >= 80.0,
            "seed {}: {outbursts} outbursts",
            seed + 1
        );
        assert!(calm >= 0.5, "seed {}: calm {calm}", seed + 1);
        assert!(
            (500.0..=1100.0).contains(&activation),
            "seed {}: {activation}",
            seed + 1
        );
        assert!(
            (12.0..=35.0).contains(&wait),
            "seed {}: wait {wait}",
            seed + 1
        );
    }
    // The paper's rule: no outburst at all (at most 13–34 actives).
    let mut literal = c;
    literal.quirks.floor_ratio = false;
    let peaks = each_seed(&literal, 3000, |w| max(&series(w, "active")));
    assert!(peaks.iter().all(|&p| p < 50.0), "{peaks:?}");
}

#[test]
#[ignore]
fn one_jump_outpeaks_salami_tactics_but_does_not_outjail_them() {
    // Measured (paired by seed): the jump's peak after t = 77 is higher in
    // 17 of 20, passes 50 in 9 (salami 3); its jail ends larger in 7.
    let peak_and_jail = |w: &ModelWorld| (max(&series(w, "active")[77..]), last(w, "jailed"));
    let salami = each_seed(&config("cv-run-3-salami"), 300, peak_and_jail);
    let jump = each_seed(&config("cv-run-4-one-jump"), 300, peak_and_jail);
    let higher: Vec<bool> = jump.iter().zip(&salami).map(|(j, s)| j.0 > s.0).collect();
    let jailed: Vec<bool> = jump.iter().zip(&salami).map(|(j, s)| j.1 > s.1).collect();
    let spikes = |r: &[(f64, f64)]| r.iter().filter(|(p, _)| *p > 50.0).count();
    assert!(count(&higher) >= 14, "higher in {}", count(&higher));
    assert!(
        spikes(&jump) > spikes(&salami),
        "{} vs {}",
        spikes(&jump),
        spikes(&salami)
    );
    assert!(count(&jailed) <= 12, "jailed more in {}", count(&jailed));
}

#[test]
#[ignore]
fn walking_the_cops_down_tips_the_society() {
    // Measured: peaks of 64–345 actives (mean 194) with about 88 of 118
    // cops left; the paper's rule peaks at 14–33 and never tips.
    let c = config("cv-run-5-cop-reductions");
    let peaks = each_seed(&c, 700, |w| max(&series(w, "active")));
    assert!(peaks.iter().all(|&p| p >= 50.0), "{peaks:?}");
    assert!(mean(&peaks) >= 120.0, "{peaks:?}");
    let mut literal = c;
    literal.quirks.floor_ratio = false;
    let peaks = each_seed(&literal, 700, |w| max(&series(w, "active")));
    assert!(peaks.iter().all(|&p| p < 50.0), "{peaks:?}");
}

#[test]
#[ignore]
fn model_two_coexists_at_high_legitimacy_and_cleanses_at_low() {
    // Measured: run 6 no kills; run 7 genocide in every seed at t = 30–190,
    // Blue surviving 12 times and Green 8.
    let kills = each_seed(&config("cv-run-6-coexistence"), 1000, |w| {
        (
            series(w, "killed").iter().sum::<f64>(),
            last(w, "blue"),
            last(w, "green"),
        )
    });
    assert!(
        kills
            .iter()
            .all(|&(k, b, g)| k == 0.0 && b > 0.0 && g > 0.0),
        "{kills:?}"
    );
    let ends = each_seed(&config("cv-run-7-cleansing"), 3000, |w| {
        (
            w.model().finished(),
            w.model().tick(),
            last(w, "blue") > 0.0,
        )
    });
    assert!(
        ends.iter().all(|&(done, tick, _)| done && tick <= 400),
        "{ends:?}"
    );
    let blue = ends.iter().filter(|e| e.2).count();
    assert!(
        (4..=16).contains(&blue),
        "the victor is random: Blue {blue} of 20"
    );
}

#[test]
#[ignore]
fn cops_do_not_keep_both_groups_alive() {
    // The paper's "stable, but nasty, regime" (Run 8) and safe havens do
    // not reproduce: measured, one group gone in every seed by t = 882 and
    // t = 377. Its Fig. 15's rapid genocide at every density does.
    for id in ["cv-run-8-nasty-regime", "cv-safe-havens"] {
        let mut c = config(id);
        c.stop_at_extinction = false;
        let alive = each_seed(&c, 3000, |w| {
            last(w, "blue") > 0.0 && last(w, "green") > 0.0
        });
        assert_eq!(
            count(&alive),
            0,
            "{id}: both groups alive in {}",
            count(&alive)
        );
    }
}

#[test]
#[ignore]
fn netlogos_rebellion_is_punctuated() {
    // Measured: 93–109 outbursts in 3000 ticks, mean activation 1107.
    let runs = each_seed(&config("cv-netlogo"), 3000, |w| last(w, "outbursts"));
    assert!(runs.iter().all(|&o| o >= 70.0), "{runs:?}");
}
```

Run: `cargo test -p sugarscape-core --release --test civil -- --ignored`
Expected: PASS, 6 tests in about 45 s.

- [ ] **Step 2: The sweeps**

`sweeps/cv-ratio-rules.json`:

```json
{
  "name": "Civil violence: the arrest rule decides whether anyone rebels",
  "description": "Run 2 (vision 7, J_max = 30, cops 0.04) at legitimacy 0.6–0.95 under three arrest rules: the paper's P = 1 − exp(−k·C/A); C/A rounded down (NetLogo's Rebellion); and rounded down with an already-active agent counted twice (NetLogo's A). Metric: outbursts above 50 actives in 1000 ticks. Measured (release, seeds 1–5, recorded 2026-09-25): the paper's rule gives 6.2 at L = 0.6, 2.0 at 0.65, fewer than 0.5 at 0.7–0.75 and none from 0.8; rounded down, 47–50 from 0.6 to 0.75, 43 at 0.8, 27 at 0.85 and none from 0.9; counting an active agent twice as well changes little (25.4–51.4 where rounding alone gives 26.8–50). The paper's stated rule cannot produce its own punctuated equilibrium at its own inputs.",
  "base": { "preset": "cv-run-2-punctuated" },
  "x": { "label": "Legitimacy (L)", "path": "legitimacy", "values": [0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95] },
  "series": {
    "label": "Arrest rule",
    "values": [
      { "at": 0, "name": "The paper’s: P = 1 − exp(−k·C/A)", "set": { "quirks.floor_ratio": false } },
      { "at": 1, "name": "C/A rounded down (NetLogo)", "set": {} },
      { "at": 2, "name": "Rounded down, an active agent counted twice (NetLogo)", "set": { "quirks.active_counts_twice": true } }
    ]
  },
  "seeds": { "from": 1, "count": 5 },
  "ticks": 1000,
  "metric": { "kind": "final", "series": "outbursts" }
}
```

`sweeps/cv-peacekeeping.json` (sweeps read NaN past a world's end, so the series value runs past extinction; the `extinction` series holds the tick):

```json
{
  "name": "Civil violence: peacekeepers and the time to genocide (Figs. 15–16)",
  "description": "Run 7 (Model II, L = 0.8) with cops from the start at densities 0–0.1, run for 3000 ticks without stopping; metric: the tick at which a group was gone (3000 if neither was). The paper (Figs. 15–17, densities 0–0.1 by 0.002, 50 runs each, capped at 15 000): rapid genocide at every density, a mean time that rises with density, a spread that rises too, and some runs past 15 000 from 0.08 up. Measured (release, seeds 1–20, recorded 2026-09-25): genocide in all 220 runs; mean 94 ticks with no cops, 106–119 at 0.01–0.03, 156–196 at 0.04–0.07 and 223–263 at 0.08–0.1; s.d. from 42 to 142–326; the longest 1596 ticks (0.08). The rise and the widening reproduce; the long delays do not.",
  "base": { "preset": "cv-run-7-cleansing" },
  "x": { "label": "Initial cop density", "path": "cop_density", "values": [0, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.1] },
  "series": {
    "label": "Runs",
    "values": [
      { "at": 0, "name": "Ticks until a group is gone (the cap if none is)", "set": { "stop_at_extinction": false } }
    ]
  },
  "seeds": { "from": 1, "count": 20 },
  "ticks": 3000,
  "metric": { "kind": "final", "series": "extinction" }
}
```

`sweeps/cv-jail-waits.json`:

```json
{
  "name": "Civil violence: jail terms and the wait between outbursts",
  "description": "Run 2 (C/A rounded down) with the longest jail term from 5 to 60 ticks; metric: the mean wait between outbursts over 2000 ticks — the paper's open question whether longer terms raise it. Measured (release, seeds 1–5, recorded 2026-09-25): 3.5 ticks at J_max = 10, 11.0 at 15, 14.8 at 20, 21.2 at 30, 28.1 at 40, 33.2 at 50 and 40.4 at 60, 0.66–0.74 × J_max from 15 up. At 5 no outburst ever ends: about 320 agents stay active throughout, so no wait is measured.",
  "base": { "preset": "cv-run-2-punctuated" },
  "x": { "label": "Longest jail term (ticks)", "path": "jail.max", "values": [5, 10, 15, 20, 30, 40, 50, 60] },
  "seeds": { "from": 1, "count": 5 },
  "ticks": 2000,
  "metric": { "kind": "final", "series": "mean_wait" }
}
```

In `crates/sugarscape-core/src/sweep.rs`, `BUILTINS` becomes `[Builtin; 11]` with, after `lhv-quirks`:

```rust
    Builtin {
        id: "cv-ratio-rules",
        json: include_str!("../../../sweeps/cv-ratio-rules.json"),
    },
    Builtin {
        id: "cv-peacekeeping",
        json: include_str!("../../../sweeps/cv-peacekeeping.json"),
    },
    Builtin {
        id: "cv-jail-waits",
        json: include_str!("../../../sweeps/cv-jail-waits.json"),
    },
```

and the three lists of built-in ids gain `"cv-ratio-rules", "cv-peacekeeping", "cv-jail-waits"` after `"lhv-quirks"`: `builtin_sweeps_parse_and_validate` in `sweep.rs`, the `sweeps` listing test in `crates/sugarscape-cli/tests/cli.rs` (its `for id in [...]`), and the `builtin_sweeps` test in `crates/sugarscape-wasm/tests/web.rs` (its `assert_eq!(ids, [...])`).

Run each sweep and compare with Decision 14:

```bash
cargo run --release -p sugarscape-cli -- sweep --builtin cv-ratio-rules --quiet --summary-csv /dev/stdout --out /dev/null
cargo run --release -p sugarscape-cli -- sweep --builtin cv-peacekeeping --quiet --summary-csv /dev/stdout --out /dev/null
cargo run --release -p sugarscape-cli -- sweep --builtin cv-jail-waits --quiet --summary-csv /dev/stdout --out /dev/null
```

Expected: the means recorded in the descriptions, to every printed digit.

- [ ] **Step 3: The CLI names why a run stopped**

In `crates/sugarscape-cli/tests/cli.rs`, add:

```rust
#[test]
fn a_civil_run_stops_when_a_group_is_gone() {
    let out = sugarscape(&["run", "--preset", "cv-run-7-cleansing", "--ticks", "1000"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        err.starts_with("finished at tick ") && err.ends_with(" (a group has died out)\n"),
        "{err}"
    );
}
```

Run: `cargo test -p sugarscape-cli a_civil_run` — Expected: FAIL (the message says "its end year").

In `crates/sugarscape-cli/src/main.rs`, import `ModelKind` (`use sugarscape_core::model::{ModelConfig, ModelKind, ModelWorld};`) and in `run_world` replace

```rust
        // The anasazi stops at its end year.
        eprintln!("finished at tick {} (its end year)", world.tick());
```

with (`world` is the `&dyn Model` there; `config` is the run's `ModelConfig`)

```rust
        // The anasazi stops at its end year; civil violence when a group is gone.
        let why = match config.kind() {
            ModelKind::Civil => "a group has died out",
            _ => "its end year",
        };
        eprintln!("finished at tick {} ({why})", world.tick());
```

Run: `cargo test -p sugarscape-cli` — Expected: PASS.

- [ ] **Step 4: WASM portability**

In `crates/sugarscape-wasm/tests/web.rs`, before `anasazi_overlays_and_inspection`:

```rust
#[wasm_bindgen_test]
fn civil_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the arrest
    // probability's exponential is the same bits here as natively.
    for (id, fp) in [
        ("cv-run-2-punctuated", "0x7888f03e0d511f6d"),
        ("cv-run-8-nasty-regime", "0x5ce734d905ba0c5d"),
        ("cv-netlogo", "0x87a92345c017b0ae"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "civil");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}
```

Run: `cargo test -p sugarscape-core --lib sweep && cargo test -p sugarscape-cli && wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS.

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/tests/civil.rs sweeps/cv-ratio-rules.json sweeps/cv-peacekeeping.json sweeps/cv-jail-waits.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/src/main.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure civil violence against the paper and add its three sweeps" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 4: Civil violence on the page — types, views, charts and the notice

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/ui/series-data.ts`, `web/src/ui/charts-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/engine.test.ts`, `web/src/determinism.test.ts`, `web/src/ui/series-data.test.ts`

**Interfaces:**
- Consumes: the WASM package (Task 2's presets and schema, Task 3's sweeps); `ScheduledChange` (types.ts).
- Produces: `CivilQuirks`, `CivilRamp`, `CivilConfig`, `CivilStats`, `CitizenView`, `CivilInspection`, `Param.show_if?: { path: string; equals: string }` (types.ts); `ModelKind` gains `'civil'`, `ColorMode` gains `'action' | 'grievance' | 'group'`; `isCivilView(v: AnyInspection): v is CivilInspection` (models.ts); `ModelChart.shown?: (c: ModelConfig) => boolean`, `isEthnic(c: ModelConfig): boolean` (series-data.ts).

- [ ] **Step 1: Write the failing tests**

`web/src/models.test.ts` — add `isCivilView` to the import from `./models`, and:

```ts
describe('the civil model', () => {
  it('is read by its tag, and its inspections by their jailed list', () => {
    const civil = { model: 'civil', variant: 'rebellion' } as unknown as ModelConfig;
    expect(modelOf(civil)).toBe('civil');
    expect(isSugar(civil)).toBe(false);
    const site = { site: { x: 1, y: 2 }, agent: null, cop: null, jailed: [] } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([site, schelling].map(isCivilView)).toEqual([true, false]);
    expect(isRingView(site) || isSugarView(site) || isValleyView(site)).toBe(false);
  });

  it('offers the paper’s two screens and its groups, and no overlays', () => {
    expect(COLOR_MODES.civil).toEqual([
      ['action', 'Action'],
      ['grievance', 'Grievance'],
      ['group', 'Group'],
    ]);
    expect(MODEL_OVERLAYS.civil).toEqual([]);
    expect(ticksLeft({ model: 'civil' } as unknown as ModelConfig, 5)).toBe(Infinity);
    expect(calendarYear({ model: 'civil' } as unknown as ModelConfig, 5)).toBeNull();
  });
});
```

`web/src/engine.test.ts` — beside the existing `finishedNotice` expectations (near line 1077), add:

```ts
    expect(finishedNotice({ model: 'civil' } as unknown as ModelConfig, 94)).toBe('A group has died out at t = 94 — Reset to run it again');
```

`web/src/ui/series-data.test.ts` — add `isEthnic` and `MODEL_CHARTS` to its imports from `./series-data` (and `ModelConfig` from `../types` if missing), and:

```ts
describe('civil charts', () => {
  it('show Model II’s groups and kills only in Model II', () => {
    const ethnic = { model: 'civil', variant: 'ethnic' } as unknown as ModelConfig;
    const rebellion = { model: 'civil', variant: 'rebellion' } as unknown as ModelConfig;
    expect([isEthnic(ethnic), isEthnic(rebellion)]).toEqual([true, false]);
    const conditional = MODEL_CHARTS.civil.filter((c) => c.shown).map((c) => c.title);
    expect(conditional).toEqual(['Groups', 'Killed']);
    expect(MODEL_CHARTS.civil.map((c) => c.title)).toEqual([
      'Actives, quiet and jailed',
      'Legitimacy',
      'Cops',
      'Tension',
      'Outbursts',
      'Wait between outbursts',
      'Activation per outburst',
      'Groups',
      'Killed',
    ]);
  });
});
```

`web/src/determinism.test.ts` — add `CivilStats` to the `./types` import; append to `GOLDEN_MODELS` (the presets that run past tick 200):

```ts
    ['cv-run-1-no-movement', '0x49637b8b34864721'],
    ['cv-run-2-punctuated', '0x7888f03e0d511f6d'],
    ['cv-run-3-salami', '0x51664e9ecc568140'],
    ['cv-run-4-one-jump', '0xc8ad285446786559'],
    ['cv-run-5-cop-reductions', '0x5713c0cafe4898dc'],
    ['cv-run-6-coexistence', '0x1ce4fc6300e993ee'],
    ['cv-run-8-nasty-regime', '0x5ce734d905ba0c5d'],
    ['cv-netlogo', '0x87a92345c017b0ae'],
```

and at the end:

```ts
describe('civil violence through the engine', () => {
  it('stops run 7 when a group is gone, once', async () => {
    const run7 = presets.find((p) => p.id === 'cv-run-7-cleansing')!;
    const e = await Engine.create({ config: structuredClone(run7.config), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'grievance' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1000);
    const s = e.latest as CivilStats;
    expect([e.finished, ends, s.extinction]).toEqual([true, 1, e.tick]);
    expect(e.tick).toBeLessThan(1000);
    expect(Math.min(s.blue, s.green)).toBe(0);
    await e.advance(10);
    expect(s.extinction).toBe(e.tick);
  });
});
```

(The existing "model charts draw only series their model records" test checks every key of `MODEL_CHARTS.civil` against a civil preset's series.)

Run: `(cd web && npm run build && npm test)`
Expected: FAIL — `tsc` rejects `'civil'`, `isCivilView`, `isEthnic`, `CivilStats`.

- [ ] **Step 2: The types**

In `web/src/types.ts`:

- `ModelKind`: `'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil'` (doc: "milestones 9–11").
- after `AnasaziConfig`:

```ts
/** NetLogo Rebellion's departures from the paper (all off: the paper's rules). */
export interface CivilQuirks {
  floor_ratio: boolean;
  active_counts_twice: boolean;
  cop_moves_to_arrest: boolean;
  jailed_stay: boolean;
  netlogo_jail_term: boolean;
}

/** A numeric live field moving linearly from its value when tick `start` begins to `to` when tick `end` begins. */
export interface CivilRamp { path: string; start: number; end: number; to: number }

/** Epstein's civil violence (milestone 11): Model I (`rebellion`) or Model II (`ethnic`). */
export interface CivilConfig {
  model: 'civil';
  variant: 'rebellion' | 'ethnic';
  width: number;
  height: number;
  agent_density: number;
  cop_density: number;
  legitimacy: number;
  threshold: number;
  k: number;
  vision: { agent: number; cop: number };
  movement: boolean;
  jail: { max: number; infinite: boolean };
  clone_probability: number;
  max_age: number;
  stop_at_extinction: boolean;
  outburst_threshold: number;
  quirks: CivilQuirks;
  schedule: ScheduledChange[];
  ramps: CivilRamp[];
}
```

- `ModelConfig`: `Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig`.
- `Param`: after `help?: string;` add

```ts
  /** Shown only while the string field `path` equals `equals` (Model II's population fields). */
  show_if?: { path: string; equals: string };
```

- after `AnasaziStats`:

```ts
export interface CivilStats {
  tick: number;
  population: number;
  active: number;
  quiet: number;
  jailed: number;
  cops: number;
  legitimacy: number;
  mean_grievance: number;
  tension: number;
  outbursts: number;
  /** Null until an outburst has followed another. */
  mean_wait: number | null;
  /** Null until an outburst has ended. */
  mean_activation: number | null;
  blue: number;
  green: number;
  killed: number;
  /** The tick a group was first gone (Model II), else the current tick. */
  extinction: number;
}
```

  and `ModelStats` gains `| CivilStats`.
- after `AnasaziInspection`:

```ts
/** A civil agent: its state, H, R, G, the arrest probability it estimates here, N = R·P, and in Model II its group and age. */
export interface CitizenView {
  id: number;
  state: 'quiet' | 'active' | 'jailed';
  hardship: number;
  risk_aversion: number;
  grievance: number;
  arrest_probability: number;
  net_risk: number;
  /** Ticks of jail left; null while free or for a term that never ends. */
  jail_left: number | null;
  jail_life: boolean;
  group: 'blue' | 'green' | null;
  age: number | null;
  death_age: number | null;
}
/** A civil site: its free agent, its cop, and the agents jailed after arrest here. */
export interface CivilInspection { site: { x: number; y: number }; agent: CitizenView | null; cop: { id: number } | null; jailed: CitizenView[] }
```

  and `AnyInspection` gains `| CivilInspection`.
- `ColorMode` gains `| 'action' | 'grievance' | 'group'` (its doc adds "or (civil violence) `action`, `grievance`, `group`").

- [ ] **Step 3: The model's views and the notice**

In `web/src/models.ts` (header comment: "milestones 9–11"; import `CivilInspection` from `./types`):

```ts
export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil'];
```

`MODEL_LABELS` gains `civil: 'Civil Violence'`; `modelOf`'s check becomes `tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil'`; after `isValleyView`:

```ts
/** A civil violence site's inspection (it lists the agents jailed after arrest there). */
export function isCivilView(v: AnyInspection): v is CivilInspection {
  return 'jailed' in v;
}
```

`COLOR_MODES` gains

```ts
  // The paper's two screens (Fig. 1) and Model II's groups (all Blue in Model I).
  civil: [
    ['action', 'Action'],
    ['grievance', 'Grievance'],
    ['group', 'Group'],
  ],
```

and `MODEL_OVERLAYS` gains `civil: []`.

In `web/src/engine.ts`, import `modelOf` beside `calendarYear` from `./models` and make `finishedNotice`:

```ts
export function finishedNotice(config: ModelConfig, tick: number): string {
  if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
  const year = calendarYear(config, tick);
  return `This run has reached its end year${year === null ? '' : ` (AD ${year})`} — Reset to run it again`;
}
```

- [ ] **Step 4: The charts**

In `web/src/ui/series-data.ts` (import `ModelConfig` from `../types` if not yet):

```ts
/** A time chart of another model: its title, lines and y range, and (civil Model II's) when it shows. */
export interface ModelChart { title: string; lines: ChartLine[]; range?: [number, number]; shown?: (c: ModelConfig) => boolean }

/** A civil config of Model II (its groups and kills have charts). */
export const isEthnic = (c: ModelConfig): boolean => 'variant' in c && c.variant === 'ethnic';
```

and `MODEL_CHARTS` gains (its doc comment adds "civil violence's actives, quiet and jailed, legitimacy, cops, tension, outbursts, and in Model II its groups and kills"):

```ts
  civil: [
    {
      title: 'Actives, quiet and jailed',
      lines: [
        { key: 'active', label: 'Active', color: '--red' },
        { key: 'quiet', label: 'Quiet', color: '--blue' },
        { key: 'jailed', label: 'Jailed', color: '--muted' },
      ],
    },
    { title: 'Legitimacy', lines: [{ key: 'legitimacy', label: 'L', color: '--c2' }], range: [0, 1] },
    { title: 'Cops', lines: [{ key: 'cops', label: 'Cops', color: '--c3' }] },
    { title: 'Tension', lines: [{ key: 'tension', label: 'Mean G × quiet share ÷ mean R', color: '--c4' }] },
    { title: 'Outbursts', lines: [{ key: 'outbursts', label: 'Outbursts ended', color: '--c1' }] },
    { title: 'Wait between outbursts', lines: [{ key: 'mean_wait', label: 'Mean (ticks)', color: '--c2' }] },
    { title: 'Activation per outburst', lines: [{ key: 'mean_activation', label: 'Mean total actives', color: '--c3' }] },
    {
      title: 'Groups',
      lines: [
        { key: 'blue', label: 'Blue', color: '--blue' },
        { key: 'green', label: 'Green', color: '--lender' },
      ],
      shown: isEthnic,
    },
    { title: 'Killed', lines: [{ key: 'killed', label: 'Killed this tick', color: '--red' }], shown: isEthnic },
  ],
```

In `web/src/ui/charts-panel.ts`: `ChartDef` gains `modelShown?: (c: ModelConfig) => boolean;` (import `ModelConfig` from `../types`); the model-chart mapping passes it:

```ts
    charts.map((c): ChartDef => ({ title: c.title, kind: 'time', section: 'top', model, lines: fixed(c.lines), range: c.range, modelShown: c.shown })),
```

and `shown(def)` for another model becomes

```ts
    if (model !== 'sugarscape') return !def.modelShown || this.worlds.some((w) => w.model === model && def.modelShown!(w.config));
```

(The panel already re-syncs on `reset` and `config`, so switching the variant shows or hides these charts.)

- [ ] **Step 5: Run the tests to see them pass**

Run: `(cd web && npm run build && npm test)`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src/types.ts web/src/models.ts web/src/engine.ts web/src/ui/series-data.ts web/src/ui/charts-panel.ts web/src/models.test.ts web/src/engine.test.ts web/src/determinism.test.ts web/src/ui/series-data.test.ts
git commit -m "Carry civil violence through the page: its types, screens, charts and notice" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): choose each civil preset from the **Civil Violence** group — both screens (Action, Grievance) draw cops light gray and actives red; Group shows Blue and Green in Model II; the charts show, and Groups/Killed only in Model II; `cv-run-7-cleansing` at Max pauses with the notice once.

---

### Task 5: The Rules panel — hidden fields, the schedule and the Compare entries

**Files:**
- Create: `web/src/civil.ts`, `web/src/civil.test.ts`
- Modify: `web/src/schema-form.ts`, `web/src/ui/schema-panel.ts`, `web/src/compare-presets.ts`
- Test: `web/src/schema-form.test.ts`, `web/src/compare-presets.test.ts`

**Interfaces:**
- Consumes: Task 4's `CivilConfig`, `Param.show_if`; `getPath` (paths.ts).
- Produces: `paramShown(p: Param, config: ModelConfig): boolean` (schema-form.ts); `scheduleLines(c: CivilConfig): string[]` (civil.ts).

- [ ] **Step 1: Write the failing tests**

`web/src/schema-form.test.ts` — import `paramShown` from `./schema-form`, and:

```ts
  it('shows a field with show_if only while its condition holds', () => {
    const p = { path: 'max_age', label: 'Longest life', kind: 'integer', apply: 'live', group: 'Population', show_if: { path: 'variant', equals: 'ethnic' } } as Param;
    const always = { ...p, show_if: undefined } as Param;
    const ethnic = { model: 'civil', variant: 'ethnic' } as unknown as ModelConfig;
    const rebellion = { model: 'civil', variant: 'rebellion' } as unknown as ModelConfig;
    expect([paramShown(p, ethnic), paramShown(p, rebellion)]).toEqual([true, false]);
    expect(paramShown(always, rebellion)).toBe(true);
  });
```

(inside the file's existing `describe`; import `Param`/`ModelConfig` types if they are not already.)

Create `web/src/civil.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { scheduleLines } from './civil';
import type { CivilConfig } from './types';

const civil = (c: Partial<CivilConfig>): CivilConfig => ({ schedule: [], ramps: [], ...c }) as CivilConfig;

describe('scheduleLines', () => {
  it('lists steps and ramps by their first tick, steps first', () => {
    const c = civil({
      schedule: [
        { tick: 77, set: { legitimacy: 0.7 } },
        { tick: 10, set: { cop_density: 0.04, 'quirks.floor_ratio': true } },
      ],
      ramps: [{ path: 'legitimacy', start: 77, end: 147, to: 0.2 }],
    });
    expect(scheduleLines(c)).toEqual([
      't = 10: cop_density → 0.04, quirks.floor_ratio → true',
      't = 77: legitimacy → 0.7',
      't = 77–147: legitimacy moves steadily to 0.2',
    ]);
  });

  it('is empty without a schedule', () => {
    expect(scheduleLines(civil({}))).toEqual([]);
  });
});
```

`web/src/compare-presets.test.ts` — beside the anasazi entry's test:

```ts
  it('pairs the civil runs the paper compares', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['cv-salami-vs-jump', 'cv-run-3-salami', 'cv-run-4-one-jump', 'Salami tactics vs one jump — Civil Violence (Compare)']);
    expect(ids).toContainEqual([
      'cv-cleansing-vs-peacekeepers',
      'cv-run-7-cleansing',
      'cv-safe-havens',
      'Ethnic cleansing vs peacekeepers — Civil Violence (Compare)',
    ]);
  });
```

Run: `(cd web && npm run build && npm test)` — Expected: FAIL (`paramShown`, `./civil`, the entries).

- [ ] **Step 2: Implement**

In `web/src/schema-form.ts`:

```ts
/** Whether a field shows in `config`: always, or while its `show_if` field equals its value. */
export function paramShown(p: Param, config: ModelConfig): boolean {
  return !p.show_if || getPath(config, p.show_if.path) === p.show_if.equals;
}
```

Create `web/src/civil.ts`:

```ts
// Civil violence's pure page helpers (milestone 11): its schedule as the Rules panel lists it.
import type { CivilConfig } from './types';

const show = (v: unknown): string => (typeof v === 'number' ? String(Number(v.toFixed(4))) : String(v));

/** The schedule's steps and ramps, one line each, by their first tick (a tick's steps before its ramps). */
export function scheduleLines(c: CivilConfig): string[] {
  const lines: [number, number, string][] = [];
  for (const s of c.schedule) {
    const sets = Object.entries(s.set)
      .map(([path, v]) => `${path} → ${show(v)}`)
      .join(', ');
    lines.push([s.tick, 0, `t = ${s.tick}: ${sets}`]);
  }
  for (const r of c.ramps) lines.push([r.start, 1, `t = ${r.start}–${r.end}: ${r.path} moves steadily to ${show(r.to)}`]);
  return lines.sort((a, b) => a[0] - b[0] || a[1] - b[1]).map(([, , line]) => line);
}
```

In `web/src/ui/schema-panel.ts` (imports: `paramShown` from `../schema-form`, `scheduleLines` from `../civil`, `CivilConfig` from `../types`):

- In `build`, give each section a syncer when all its params are conditional, and add the civil schedule:

```ts
    const sections = groupParams(this.engine.schemas[model] ?? []).map(({ group, params }) => {
      const section = h('section', { class: 'group' }, h('h3', {}, group), h('p', { class: 'hint' }, note(params)), ...params.map((p) => this.control(p)));
      if (params.every((p) => p.show_if)) this.syncers.push(() => (section.hidden = !params.some((p) => paramShown(p, this.engine.config))));
      return section;
    });
    const extra = model === 'anasazi' ? [valleyCredit()] : [];
    const schedule = model === 'civil' ? [this.schedule()] : [];
    this.el.replaceChildren(...extra, this.general, ...sections, ...schedule);
```

- Add the Schedule section:

```ts
  /** Civil violence's schedule and ramps, read-only (they travel in links and sessions). */
  private schedule(): HTMLElement {
    const list = h('ul', { class: 'hint' });
    const section = h('section', { class: 'group' }, h('h3', {}, 'Schedule'), h('p', { class: 'hint' }, 'Set by the preset; these change the running world at their ticks.'), list);
    this.syncers.push(() => {
      const lines = scheduleLines(this.engine.config as CivilConfig);
      section.hidden = lines.length === 0;
      list.replaceChildren(...lines.map((l) => h('li', {}, l)));
    });
    return section;
  }
```

- In `control(p)`, wrap the element the `switch` returns: rename the current body to `private controlBody(p: Param): HTMLElement` and add

```ts
  private control(p: Param): HTMLElement {
    const el = this.controlBody(p);
    if (p.show_if) this.syncers.push(() => (el.hidden = !paramShown(p, this.engine.config)));
    return el;
  }
```

In `web/src/compare-presets.ts`, append to `COMPARE_PRESETS`:

```ts
  {
    id: 'cv-salami-vs-jump',
    label: 'Salami tactics vs one jump — Civil Violence (Compare)',
    a: 'cv-run-3-salami',
    b: 'cv-run-4-one-jump',
  },
  {
    id: 'cv-cleansing-vs-peacekeepers',
    label: 'Ethnic cleansing vs peacekeepers — Civil Violence (Compare)',
    a: 'cv-run-7-cleansing',
    b: 'cv-safe-havens',
  },
```

- [ ] **Step 3: Run the tests to see them pass**

Run: `(cd web && npm run build && npm test)` — Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add web/src/civil.ts web/src/civil.test.ts web/src/schema-form.ts web/src/schema-form.test.ts web/src/ui/schema-panel.ts web/src/compare-presets.ts web/src/compare-presets.test.ts
git commit -m "Show civil violence's rules by variant, list its schedule, and pair its runs in Compare" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): the Rules panel for `cv-run-2-punctuated` has no Population section; switching Model to II shows it; `cv-run-3-salami` lists `t = 77–147: legitimacy moves steadily to 0.2`; a live legitimacy change applies without a rebuild; both Compare entries open side by side, and the cleansing pair pauses when A's group is gone.

---

### Task 6: Inspect a civil site and follow its agent into jail

**Files:**
- Modify: `web/src/civil.ts`, `web/src/civil.test.ts`, `web/src/ui/inspect-panel.ts`

**Interfaces:**
- Consumes: Task 4's `CitizenView`, `CivilInspection`, `isCivilView`.
- Produces: `citizenRows(a: CitizenView): [string, string][]`, `shownCitizen(view: CivilInspection, followed: number | null): CitizenView | null` (civil.ts).

- [ ] **Step 1: Write the failing tests**

Append to `web/src/civil.test.ts` (import `citizenRows`, `shownCitizen`; types `CitizenView`, `CivilInspection`):

```ts
const agent = (c: Partial<CitizenView>): CitizenView => ({
  id: 1,
  state: 'quiet',
  hardship: 0.5,
  risk_aversion: 0.25,
  grievance: 0.09,
  arrest_probability: 0.9,
  net_risk: 0.225,
  jail_left: null,
  jail_life: false,
  group: null,
  age: null,
  death_age: null,
  ...c,
});

describe('citizenRows', () => {
  it('shows H, R, G and the risk it runs, and Model II’s group and age', () => {
    expect(citizenRows(agent({}))).toEqual([
      ['Agent', '#1 · quiet'],
      ['Hardship (H)', '0.50'],
      ['Risk aversion (R)', '0.25'],
      ['Grievance (G)', '0.09 = H × (1 − L)'],
      ['Arrest risk (P)', '0.90 · net risk R·P 0.23'],
    ]);
    const rows = citizenRows(agent({ group: 'green', age: 12, death_age: 150 }));
    expect(rows.slice(-2)).toEqual([
      ['Group', 'Green'],
      ['Age', '12 of 150'],
    ]);
  });

  it('says how long a jailed agent has left instead of its risk', () => {
    expect(citizenRows(agent({ state: 'jailed', jail_left: 4 }))[0]).toEqual(['Agent', '#1 · jailed (4 ticks left)']);
    const life = citizenRows(agent({ state: 'jailed', jail_life: true }));
    expect(life[0]).toEqual(['Agent', '#1 · jailed (for life)']);
    expect(life.some(([k]) => k === 'Arrest risk (P)')).toBe(false);
  });
});

describe('shownCitizen', () => {
  const free = agent({ id: 2 });
  const jailed = agent({ id: 3, state: 'jailed', jail_left: 2 });
  const view: CivilInspection = { site: { x: 0, y: 0 }, agent: free, cop: null, jailed: [jailed] };

  it('shows the followed agent while it is jailed at this site', () => {
    expect(shownCitizen(view, 3)).toBe(jailed);
    expect(shownCitizen(view, 2)).toBe(free);
  });

  it('shows the free agent otherwise', () => {
    expect(shownCitizen(view, null)).toBe(free);
    expect(shownCitizen(view, 99)).toBe(free);
    expect(shownCitizen({ ...view, agent: null }, null)).toBeNull();
  });
});
```

Run: `(cd web && npx vitest run src/civil.test.ts)` — Expected: FAIL (not exported).

- [ ] **Step 2: Implement the helpers**

Append to `web/src/civil.ts` (import `CitizenView`, `CivilInspection` types):

```ts
const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));

/** An agent's Inspect rows: its state, H, R, G and (while free) the risk it runs; in Model II its group and age. */
export function citizenRows(a: CitizenView): [string, string][] {
  const state = a.state === 'jailed' ? `jailed (${a.jail_life ? 'for life' : `${a.jail_left} ticks left`})` : a.state;
  const rows: [string, string][] = [
    ['Agent', `#${a.id} · ${state}`],
    ['Hardship (H)', fmt(a.hardship)],
    ['Risk aversion (R)', fmt(a.risk_aversion)],
    ['Grievance (G)', `${fmt(a.grievance)} = H × (1 − L)`],
  ];
  if (a.state !== 'jailed') rows.push(['Arrest risk (P)', `${fmt(a.arrest_probability)} · net risk R·P ${fmt(a.net_risk)}`]);
  if (a.group !== null) rows.push(['Group', a.group === 'blue' ? 'Blue' : 'Green'], ['Age', `${a.age} of ${a.death_age}`]);
  return rows;
}

/** The agent Inspect shows at a civil site: the one it follows (free there, or jailed after arrest there), else the free occupant. */
export function shownCitizen(view: CivilInspection, followed: number | null): CitizenView | null {
  if (followed !== null) {
    if (view.agent?.id === followed) return view.agent;
    const jailed = view.jailed.find((a) => a.id === followed);
    if (jailed) return jailed;
  }
  return view.agent;
}
```

- [ ] **Step 3: The panel**

In `web/src/ui/inspect-panel.ts` (imports: `isCivilView` from `../models`; `CivilInspection` from `../types`; `citizenRows`, `shownCitizen` from `../civil`), add:

```ts
  /** A civil site: its cop, the agent shown there (followed into jail), and others jailed after arrest here. */
  private civilRows(view: CivilInspection, followed: number | null, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
    if (view.cop) rows.push(row('Cop', `#${view.cop.id}`));
    const a = gone ? null : shownCitizen(view, followed);
    if (a) rows.push(...citizenRows(a).map(([k, v]) => row(k, v)));
    const others = view.jailed.filter((j) => j.id !== a?.id).length;
    if (others > 0) rows.push(row('Jailed here', `${others} arrested on this site`));
    return rows;
  }
```

and in `render()`'s non-sugarscape branch:

```ts
      const left = isValleyView(view)
        ? `Household #${shown.agentId} is gone: it died or left the valley.`
        : isCivilView(view)
          ? `Agent #${shown.agentId} is gone: killed, or dead of old age.`
          : `Agent #${shown.agentId} has left.`;
      const note = gone ? [h('p', { class: 'error' }, left)] : [];
      const rows = isRingView(view)
        ? this.ringRows(view, gone)
        : isValleyView(view)
          ? this.valleyRows(view, gone)
          : isCivilView(view)
            ? this.civilRows(view, shown.agentId, gone)
            : this.schellingRows(view, gone);
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `(cd web && npm run build && npm test)` — Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add web/src/civil.ts web/src/civil.test.ts web/src/ui/inspect-panel.ts
git commit -m "Inspect civil sites and follow an agent into jail" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): in `cv-run-2-punctuated`, Inspect an agent near an outburst, step until it is arrested: the panel keeps it, "jailed (n ticks left)", then free again elsewhere; in `cv-run-7-cleansing`, a followed agent that is killed reads "Agent #id is gone: killed, or dead of old age."

---

### Task 7: Experiments over a civil base

**Files:**
- Modify: `web/src/experiments/form.ts`
- Test: `web/src/experiments/form.test.ts`

**Interfaces:**
- Consumes: `defaultForm(model: ModelKind)`.
- Produces: `defaultForm('civil')`.

- [ ] **Step 1: Write the failing test**

In `web/src/experiments/form.test.ts`, beside the anasazi's:

```ts
    expect(defaultForm('civil')).toMatchObject({
      x: { path: 'legitimacy', values: '0.6:0.95:0.05' },
      ticks: 1000,
      seeds: 3,
      metric: { kind: 'final', series: 'outbursts' },
    });
```

Run: `(cd web && npx vitest run src/experiments/form.test.ts)` — Expected: FAIL.

- [ ] **Step 2: Implement**

In `web/src/experiments/form.ts`'s `defaultForm`, before `return form;`:

```ts
  if (model === 'civil') {
    // The Finding's axis (the built-in cv-ratio-rules): outbursts against legitimacy.
    return { ...form, x: { path: 'legitimacy', values: '0.6:0.95:0.05' }, ticks: 1000, metric: { ...form.metric, kind: 'final', series: 'outbursts' } };
  }
```

- [ ] **Step 3: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS.

```bash
git add web/src/experiments/form.ts web/src/experiments/form.test.ts
git commit -m "Start civil sweeps from outbursts against legitimacy" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): Experiments → built-in `cv-ratio-rules` runs on the worker pool and draws three lines (the paper's rule flat near 0); a new sweep on `cv-run-2-punctuated` opens with the legitimacy axis; `cv-peacekeeping` runs to completion.

---

### Task 8: The survey's civil claims

**Files:**
- Create: `survey/src/claims/civil.rs`
- Modify: `survey/src/runner.rs`, `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `sugarscape_core::{presets::find, model::{ModelConfig, ModelWorld}}`; `crate::claim::{greater, range, Claim, Source}`.
- Produces: `runner::model_preset(id: &str) -> ModelConfig`, `runner::model_after<T: Send>(config: &ModelConfig, seeds: &[u64], ticks: u32, f: impl Fn(&ModelWorld) -> T + Sync) -> Vec<T>`.

- [ ] **Step 1: Write the failing test**

In `survey/src/runner.rs`'s tests:

```rust
    #[test]
    fn worlds_of_any_model_run_per_seed_and_stop_when_finished() {
        let c = model_preset("cv-run-7-cleansing");
        let ends = model_after(&c, &[1, 2], 5000, |w| {
            (w.model().tick(), w.model().finished())
        });
        assert!(
            ends.iter().all(|&(tick, done)| done && tick < 5000),
            "{ends:?}"
        );
        assert_ne!(ends[0], ends[1], "each seed its own world");
    }
```

Run: `(cd survey && cargo test)` — Expected: FAIL (no `model_preset`).

- [ ] **Step 2: Implement the runner**

In `survey/src/runner.rs`: import `use sugarscape_core::model::{ModelConfig, ModelWorld};`; add after `preset`:

```rust
/// A preset of any model (`presets::find`).
pub fn model_preset(id: &str) -> ModelConfig {
    presets::find(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
}
```

make `each_seed`'s body `on_threads(seeds, |seed| f(World::new(config.clone(), seed).expect("a valid config")))`, add

```rust
/// Like `after`, for a world of any model: builds it from `config` per seed,
/// runs `ticks` ticks (fewer if it finishes) and measures it with `f`.
pub fn model_after<T: Send>(
    config: &ModelConfig,
    seeds: &[u64],
    ticks: u32,
    f: impl Fn(&ModelWorld) -> T + Sync,
) -> Vec<T> {
    on_threads(seeds, |seed| {
        let mut w = ModelWorld::new(config.clone(), seed).expect("a valid config");
        w.model_mut().run(ticks);
        f(&w)
    })
}
```

and move the thread pool that was `each_seed`'s body into

```rust
/// `f(seed)` for every seed on a thread pool, in seed order (see
/// `each_seed` for panics).
fn on_threads<T: Send>(seeds: &[u64], f: impl Fn(u64) -> T + Sync) -> Vec<T> {
```

with its per-seed call `std::panic::catch_unwind(AssertUnwindSafe(|| f(seeds[i])))` (the rest unchanged).

- [ ] **Step 3: The claims**

Create `survey/src/claims/civil.rs` (Decision 19):

```rust
//! Epstein 2002's civil violence (milestone 11): the paper's figures, the
//! presets' descriptions and the Finding that its stated arrest rule gives
//! Model I no rebellion. Runs that several claims share are memoized per
//! process, keyed by the seeds.

use std::sync::{Arc, Mutex};

use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{greater, range, Claim, Source};
use crate::runner::{model_after, model_preset};

const PAPER: &str = "Epstein 2002, PNAS 99 suppl. 3";

/// A per-process cache of one computation per seed list.
struct Memo<T>(Mutex<Vec<(Vec<u64>, Arc<T>)>>);

impl<T> Memo<T> {
    const fn new() -> Self {
        Memo(Mutex::new(Vec::new()))
    }

    fn get(&self, seeds: &[u64], f: impl FnOnce() -> T) -> Arc<T> {
        let mut m = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, v)) = m.iter().find(|(s, _)| s == seeds) {
            return v.clone();
        }
        let v = Arc::new(f());
        m.push((seeds.to_vec(), v.clone()));
        v
    }
}

fn series(w: &ModelWorld, name: &str) -> Vec<f64> {
    w.model()
        .series(name)
        .unwrap_or_else(|| panic!("no series {name}"))
}

fn last(w: &ModelWorld, name: &str) -> f64 {
    *series(w, name).last().expect("a recorded tick")
}

fn max(v: &[f64]) -> f64 {
    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

/// A preset with its arrest ratio rounded down or not.
fn with_floor(id: &str, floor: bool) -> ModelConfig {
    let mut c = model_preset(id);
    if let ModelConfig::Civil(c) = &mut c {
        c.quirks.floor_ratio = floor;
    }
    c
}

/// A Model II preset that runs on past extinction (so a seed is measured
/// for the whole run).
fn running_on(id: &str) -> ModelConfig {
    let mut c = model_preset(id);
    if let ModelConfig::Civil(c) = &mut c {
        c.stop_at_extinction = false;
    }
    c
}

/// Run 2 over 3000 ticks: outbursts, mean activation, mean wait, peak.
struct Run2 {
    outbursts: f64,
    activation: f64,
    wait: f64,
    peak: f64,
}

fn run_2(seeds: &[u64], floor: bool) -> Arc<Vec<Run2>> {
    static FLOOR: Memo<Vec<Run2>> = Memo::new();
    static LITERAL: Memo<Vec<Run2>> = Memo::new();
    let memo = if floor { &FLOOR } else { &LITERAL };
    memo.get(seeds, || {
        model_after(
            &with_floor("cv-run-2-punctuated", floor),
            seeds,
            3000,
            |w| Run2 {
                outbursts: last(w, "outbursts"),
                activation: last(w, "mean_activation"),
                wait: last(w, "mean_wait"),
                peak: max(&series(w, "active")),
            },
        )
    })
}

/// Runs 3 and 4 (salami tactics, one jump) over 300 ticks: the peak of
/// actives after t = 77 and the jailed at t = 300.
fn runs_3_4(seeds: &[u64]) -> Arc<[Vec<(f64, f64)>; 2]> {
    static M: Memo<[Vec<(f64, f64)>; 2]> = Memo::new();
    M.get(seeds, || {
        let one = |id| {
            model_after(&model_preset(id), seeds, 300, |w| {
                (max(&series(w, "active")[77..]), last(w, "jailed"))
            })
        };
        [one("cv-run-3-salami"), one("cv-run-4-one-jump")]
    })
}

/// Whether both groups are alive after `ticks`, per seed.
fn both_alive(id: &str, seeds: &[u64], ticks: u32) -> Vec<f64> {
    model_after(&running_on(id), seeds, ticks, |w| {
        f64::from(u8::from(last(w, "blue") > 0.0 && last(w, "green") > 0.0))
    })
}

/// The tick at which a group was gone (3000 if neither was), run 7 at a
/// cop density.
fn extinction(seeds: &[u64], cops: f64) -> Vec<f64> {
    let mut c = running_on("cv-run-7-cleansing");
    if let ModelConfig::Civil(c) = &mut c {
        c.cop_density = cops;
    }
    model_after(&c, seeds, 3000, |w| last(w, "extinction"))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "cv-run-2.no-outbursts-as-stated",
            item: "cv-run-2-punctuated",
            source: Source::App,
            citation: "spec 2026-09-25-civil-violence-design.md, Finding",
            text: "with the paper's P = 1 − exp(−k·C/A), Run 2 never has an outburst: at most 49 actives in 3000 ticks",
            check: |s| {
                let v: Vec<f64> = run_2(s, false).iter().map(|r| r.peak).collect();
                range(&v, 0.0, 49.0, false)
            },
        },
        Claim {
            id: "cv-run-2.punctuated",
            item: "cv-run-2-punctuated",
            source: Source::Book,
            citation: PAPER,
            text: "punctuated equilibrium (Figs. 3–4): with C/A rounded down, at least 80 outbursts above 50 actives in 3000 ticks",
            check: |s| {
                let v: Vec<f64> = run_2(s, true).iter().map(|r| r.outbursts).collect();
                range(&v, 80.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "cv-run-2.activation",
            item: "cv-run-2-punctuated",
            source: Source::Book,
            citation: PAPER,
            text: "total activation per outburst has mean 708 and s.d. 230 (Fig. 7): the mean within 708 ± 230",
            check: |s| {
                let v: Vec<f64> = run_2(s, true).iter().map(|r| r.activation).collect();
                range(&v, 478.0, 938.0, false)
            },
        },
        Claim {
            id: "cv-run-2.wait",
            item: "cv-run-2-punctuated",
            source: Source::Book,
            citation: PAPER,
            text: "the average duration between outbursts is 60 (Fig. 5): the mean wait about 60 (50–70, widened by 10%)",
            check: |s| {
                let v: Vec<f64> = run_2(s, true).iter().map(|r| r.wait).collect();
                range(&v, 50.0, 70.0, true)
            },
        },
        Claim {
            id: "cv-run-3.no-spike",
            item: "cv-run-3-salami",
            source: Source::Book,
            citation: PAPER,
            text: "a large legitimacy reduction in small increments gives no red spike (Fig. 9): at most 50 actives after t = 77",
            check: |s| {
                let v: Vec<f64> = runs_3_4(s)[0].iter().map(|r| r.0).collect();
                range(&v, 0.0, 50.0, false)
            },
        },
        Claim {
            id: "cv-run-4.explosion",
            item: "cv-run-4-one-jump",
            source: Source::Book,
            citation: PAPER,
            text: "a smaller reduction in one jump gives an explosion of actives (Fig. 10): its peak after t = 77 exceeds salami tactics'",
            check: |s| {
                let r = runs_3_4(s);
                let peak = |i: usize| r[i].iter().map(|x| x.0).collect::<Vec<_>>();
                greater(&peak(1), &peak(0), "one jump", "salami")
            },
        },
        Claim {
            id: "cv-run-4.more-jailed",
            item: "cv-run-4-one-jump",
            source: Source::Book,
            citation: PAPER,
            text: "the jailed population's absolute size exceeds that of the previous run (Fig. 10): more jailed at t = 300 than salami tactics",
            check: |s| {
                let r = runs_3_4(s);
                let jailed = |i: usize| r[i].iter().map(|x| x.1).collect::<Vec<_>>();
                greater(&jailed(1), &jailed(0), "one jump", "salami")
            },
        },
        Claim {
            id: "cv-run-5.tips",
            item: "cv-run-5-cop-reductions",
            source: Source::Book,
            citation: PAPER,
            text: "a marginal reduction in cops tips society into rebellion (Fig. 11): more than 50 actives at some tick of 700",
            check: |s| {
                let v = model_after(&model_preset("cv-run-5-cop-reductions"), s, 700, |w| {
                    max(&series(w, "active"))
                });
                range(&v, 50.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "cv-run-6.coexistence",
            item: "cv-run-6-coexistence",
            source: Source::Book,
            citation: PAPER,
            text: "with high legitimacy and no cops, peaceful coexistence prevails (Fig. 12): both groups alive after 1000 ticks",
            check: |s| range(&both_alive("cv-run-6-coexistence", s, 1000), 1.0, 1.0, false),
        },
        Claim {
            id: "cv-run-7.genocide",
            item: "cv-run-7-cleansing",
            source: Source::Book,
            citation: PAPER,
            text: "at L = 0.8 with no cops genocide is always observed (Fig. 13): one group gone within 3000 ticks",
            check: |s| range(&both_alive("cv-run-7-cleansing", s, 3000), 0.0, 0.0, false),
        },
        Claim {
            id: "cv-run-8.stable",
            item: "cv-run-8-nasty-regime",
            source: Source::Book,
            citation: PAPER,
            text: "with cop density 0.04 from the outset a stable, but nasty, regime emerges; the cops prevent either side wiping the other out: both groups alive after 3000 ticks",
            check: |s| range(&both_alive("cv-run-8-nasty-regime", s, 3000), 1.0, 1.0, false),
        },
        Claim {
            id: "cv-safe-havens.havens",
            item: "cv-safe-havens",
            source: Source::Book,
            citation: PAPER,
            text: "peacekeepers deployed at t = 50 typically produce safe havens (Fig. 14): both groups alive after 3000 ticks",
            check: |s| range(&both_alive("cv-safe-havens", s, 3000), 1.0, 1.0, false),
        },
        Claim {
            id: "cv-peacekeeping.delay",
            item: "cv-peacekeeping",
            source: Source::Book,
            citation: PAPER,
            text: "the larger the initial force of peacekeepers, the more time one buys (Fig. 16): extinction later at density 0.1 than at 0",
            check: |s| greater(&extinction(s, 0.1), &extinction(s, 0.0), "0.1", "none"),
        },
    ]
}
```

and in `survey/src/claims/mod.rs` add `mod civil;` after `mod ch6;` and `civil::claims(),` after `ch6::claims(),` in `all()`.

- [ ] **Step 4: Run it**

Run: `(cd survey && cargo test && cargo run --release -- --only cv)`
Expected: tests PASS; the table shows 9 Holds and 4 Fails — `cv-run-2.wait` (median 21.6), `cv-run-4.more-jailed`, `cv-run-8.stable`, `cv-safe-havens.havens` — as Decision 19. (`cargo clippy` in `survey/` also flags three older lints in `ch6.rs` with the newest clippy; they predate this milestone and the survey is not in CI — leave them.)

- [ ] **Step 5: Commit**

```bash
(cd survey && cargo fmt)
git add survey/src/runner.rs survey/src/claims/mod.rs survey/src/claims/civil.rs
git commit -m "Survey civil violence's claims: nine hold, four do not" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 9: README, roadmap, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/superpowers/specs/2026-09-25-civil-violence-design.md`

- [ ] **Step 1: The spec's amendments**

In the spec: drop `cv-peacekeepers-withdrawn` from the presets table and say why (Decision 13); replace "Actives and jailed (with legitimacy × 1000 and cops on a second axis)" with the separate charts of the Copy list; in Survey, say the literal-rule rescue checks were run during planning, with Decision 14's numbers.

- [ ] **Step 2: README**

Add a section **Civil violence (Epstein 2002)** after the anasazi's, covering: the two models in a paragraph each (rules A, C and M; Model II's killing, cloning and death); the choices the paper leaves open (Euclidean vision, jail terms 1…J_max, release near the arrest site, the Moore neighborhood for clones); **the Finding**, plainly: the paper's stated arrest rule gives Model I no outbursts at its own inputs (0 in 20 of 20 seeds of Run 2), only NetLogo's ⌊C/A⌋ produces the punctuated equilibrium, so Model I presets round down and say so, with `cv-ratio-rules` to see it; the other results that do and do not reproduce (Decision 14 in a short list, including the mean wait a third of the paper's, the one jump's jail, Run 8's regime and safe havens, and the paper's 455-not-500 sum); the five NetLogo quirks and `cv-netlogo`; the three sweeps; the two Compare entries; credit to the paper and to NetLogo *Rebellion*. Add `civil` to the list of models the presets menu groups and the Other artificial societies section's list.

- [ ] **Step 3: Roadmap**

Add after Milestone 10:

```markdown
## Milestone 11: Civil violence (done)

Epstein's civil violence (2002), Models I and II, as a fifth model kind, with NetLogo Rebellion's five
departures as switches and the paper's runs as presets. The paper's stated arrest rule gives Model I no
rebellion at its own inputs; only NetLogo's rounded-down C/A reproduces its punctuated equilibrium, so the
Model I presets round down and say so (the sweep `cv-ratio-rules` shows it). The salami-tactics, cop-reduction,
coexistence and cleansing results reproduce; the mean wait is a third of the paper's, and Run 8's stable
regime and the safe havens do not reproduce. See `docs/superpowers/specs/2026-09-25-civil-violence-design.md`.
```

and change the **Epstein's civil violence** line under Experiments and science to "done (Milestone 11)".

- [ ] **Step 4: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test civil -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && cargo test)
```

Expected: all PASS. Browser (controller): every civil preset (both screens, charts, Inspect, the Rules panel by variant), both Compare entries, recording a civil world (WebM and GIF), Experiments `cv-ratio-rules` and `cv-peacekeeping`, share links and session files of a civil world with a live legitimacy change replaying exactly, Max speed on `cv-run-2-punctuated` (the performance check), and every existing scenario.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/roadmap.md docs/superpowers/specs/2026-09-25-civil-violence-design.md
git commit -m "Document civil violence and mark milestone 11 done" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

## Self-review (planning)

- **Spec coverage:** architecture and config (Tasks 1–2), the step and every rule (2), the Finding and its sweep (3, 9), statistics and outbursts (1), views, Inspect, charts (4, 6), presets and Compare entries (2, 5), Experiments and CLI (3, 7), the survey (8), testing (every task), docs (9). The dropped preset and the separate charts are Decision 13, and Task 9 amends the spec.
- **Placeholders:** none; every Rust block is the dry run's code.
- **Type consistency:** names in Global Constraints match the code blocks; `CivilInspection.jailed` (Rust `Vec<CitizenView>`, TS `CitizenView[]`) and `show_if` (Rust `ShowIf`, TS `{ path, equals }`) agree.
- **Review Focus:** each of the five has its test in the owning task.
