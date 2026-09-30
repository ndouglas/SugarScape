# Minds 7: the evolution of larder hoarding — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vander Wall and Jenkins's (2003) genetic algorithm, built as written as a new non-spatial model
kind `hoard`, with its five reported results as survey claims and three named switches reported as new ground.

**Architecture:**
- `crates/sugarscape-core/src/hoard/` follows the model-kind layout of `zi/` and `punishment/`: `config.rs`,
  `world.rs`, `stats.rs`, `view.rs`, `presets.rs` and `mod.rs`.
- It is registered in `model.rs`, `lib.rs`, the presets catalog, titles, goldens, the WASM crate and the page.
- The ZI milestone is the wiring checklist: plan `docs/superpowers/plans/2026-09-28-zi-traders.md` and commits
  `7623e28` (core), `5d638ea` (sweeps and WASM) and `8d3eb55` (page). Mirror every place it registered a
  model kind.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), `survey/`, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-09-30-minds-7-hoarding-evolution-design.md`

## Global Constraints

- **Originator first.** Defaults are V&J's parameters, quoted in the spec.
- **Gaps in the paper.** Every gap is filled as a stated choice in the spec's amendments, never tuned to
  reproduce a result. Task 1 settles the gaps before any code.
- **Isolation.** No Sugarscape world changes. Existing golden entries, legacy fixtures and pinned WASM
  fingerprints stay green and unedited.
- **Determinism.** One seeded RNG per `HoardWorld`. Draws happen in a fixed, documented order, and the same
  seed gives the same run. No `HashMap` iteration. Nothing that is platform-dependent.
- **Menu.** Every preset's `source` contains "Minds 7" and "Vander Wall & Jenkins 2003", so the preset lands
  under the Minds entry. `web/src/models.ts` gets `MINDS_TITLES['7'] = 'Minds 7: evolution of hoarding'`.
- **Style.** Never write "Flump"; say "agent". American spelling. Titles follow `titles.rs`.
- **Survey judges** are committed before any survey run and never tuned.
- **Commits.** Every commit ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- **Where to work.** `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-7` (branch
  `minds-7`). For web tests, temporarily symlink the main checkout's `web/node_modules` and remove it afterward.

## Review Focus

1. **A generation where every agent starves.** Breeding must not panic or divide by zero. Choose what
   happens (the run ends, or the population restarts from the prior generation's traits), state it, and
   test it. Owned by Task 4.
2. **Selection when every surviving agent has 0 stores left.** Fitness-proportional selection with total
   weight 0 needs a stated rule, for example uniform among survivors. Owned by Task 4.
3. **Logit-scale inheritance at extreme values.** L at 0 or 1 must not produce NaN or infinities. Clamp to
   (ε, 1 − ε), state ε, and test it. Owned by Task 4.
4. **A raid that empties a larder,** or an owner returning mid-raid. Items are conserved and the raid ends.
   Owned by Task 3.
5. **Configs missing optional `hoard` fields** load with the defaults. Owned by Task 2.

---

### Task 1: Read the Methods and settle the gaps (no code)

**Files:** the spec's "## Amendments (implementation)" section (create it).

- Read V&J pp. 661–666 in full with `pdftotext -layout` on `papers/caching/vanderwall-jenkins-2003-behecol-reciprocal-pilferage.pdf`,
  including the figure captions and the Appendix, if any.
- For each item below, quote what the paper says. Where it's silent, record a stated choice with its
  reason. Choose the simplest reading consistent with the text, and never pick one to hit a result.
  1. **Order within a bout:** random order of agents each bout, or fixed.
  2. **"The first two options took priority":** the exact decision sequence for a fed and an unfed agent,
     including when an agent forages to store versus defends.
  3. **Defense:**
     - the logistic's slope;
     - how the target is computed from D;
     - "the minimum needed to survive": what that is at a given day;
     - "the maximum obtainable": what that is;
     - whether defense is decided per bout or per day.
  4. **Raids:** does a raider take one item per bout; does "the owner returns" mean the owner's next
     defend decision; and can several raiders raid one larder at once.
  5. **Food available:**
     - Are the pools counted in items or weighted items?
     - Is an agent's own stores excluded from its search?
     - Does public food include the nonstorable food on days 1–5, and how is nonstorable food handled?
  6. **Eating from stores:** whether larder or scatter is drawn first, and whether V&J's free recovery
     applies to both.
  7. **Starvation:** whether a day without eating kills the agent that night.
  8. **Reproduction:**
     - pairing (two parents drawn with replacement in proportion to stores, or one parent);
     - V_seg's value;
     - how the initial "logistic distribution" around 0.15 and 0.50 is parameterized.
  9. **Traits:** which are heritable. The spec says L and D, and forage_i is drawn fresh; confirm this.
  10. **Loss rates:**
      - How V&J compute "daily rate of loss" per agent for larder and scatter items. Try to reconstruct
        the 186 % figure (for example, items lost ÷ mean store), and state the reading we'll use.
      - The CV definition.
  11. **"Larders take over":** the precise criterion, mean L > 0.95, and the generation at which it's
      counted.
  12. **Default apparencies:** choose `app_scat` and `app_lard` for the default preset so that the ratio is
      near 0.22, V&J's 50 % point.
- Commit ("Minds 7: the Methods read, and the gaps filled as stated choices").

---

### Task 2: The model kind, config and skeleton

**Files:** `hoard/{mod,config}.rs`, `model.rs`, `lib.rs`, `schema` registration, `legacy.rs` if model
kinds are listed there. Mirror the core half of `7623e28`.

- `HoardConfig`:
  - V&J's fields: n 20, days 100, bouts 20, the food schedule parameters, app_scat, app_lard, predation
    0.0001, heritability 0.8, V_seg, initial L and D distribution parameters, generations 60;
  - the gap choices from Task 1 as fields where it makes sense;
  - the switches `owner_recovery` (1.0) and `cheaters` (0).
- Validation, with field names and messages following ZI's style.
- `ModelKind::Hoard` (serde `hoard`), `model_name`, schema and dispatch stubs.
- **Tests:** defaults, validation, and older configs loading with the defaults (Review Focus 5).

**Commit.**

---

### Task 3: A season

**Files:** `hoard/world.rs`.

- One tick is one bout. Days and seasons follow, with the food schedule, search, eating, storing, defense,
  raids and predation, exactly as the spec and Task 1's choices say.
- **Verification tests:**
  - the schedule sums to 2 100 (82 on day 1, 2 on day 50);
  - calibration gives k = 0.23 and P = 0.21 at the start;
  - a day-1 total-failure chance near 0.01, checked by Monte Carlo over 10 000 seeds within a stated
    tolerance;
  - conservation every bout;
  - raids (Review Focus 4);
  - determinism.

**Commit.**

---

### Task 4: Generations

**Files:** `hoard/world.rs`.

- **End of season:** fitness is the stores left. Selection and pairing follow Task 1. Inheritance is on the
  logit scale.
- **Measurements:**
  - per agent, the larder and scatter daily loss rates as defined in Task 1;
  - per generation, mean L, mean D, survivors, the larder share and the takeover flag.
- **Tests:**
  - the inheritance regression with V = 0;
  - selection in expectation;
  - Review Focus 1–3;
  - a 60-generation run is deterministic.

**Commit.**

---

### Task 5: Switches

- **`owner_recovery` < 1.** Each time an owner tries to eat from its own scattered caches, it finds one
  with this probability, or else forages. State whether larders are exempt; a larder is at home.
- **`cheaters` > 0.** Founders are assigned by id with no draw, as Minds 6 did. Cheaters never cache and eat
  what they find. Their children inherit the type.
- **Tests:** 1.0 and 0 reproduce the default run bit for bit. Each switch changes what it should.

**Commit.**

---

### Task 6: Series, presets, titles and golden

- **`hoard/stats.rs`:** series per bout and per generation, as the plan's page needs:
  - `mean_larder_prob`, `mean_defense`;
  - `larder_loss_rate`, `scatter_loss_rate`;
  - `survivors`, `larder_share`, `generation`.
- **`hoard/presets.rs`:**
  - `hoard-threshold`, at the default ratio of about 0.22;
  - `hoard-scatter`, a low ratio of about 0.1 (scatter should hold);
  - `hoard-larder`, a high ratio of about 0.4;
  - `hoard-no-free-recovery`, with `owner_recovery` 0.5;
  - `hoard-cheaters`, with `cheaters` 0.25.
  - Each `source` names "Vander Wall & Jenkins 2003; Minds 7". Descriptions cover the setup only.
- **Titles** are drafts, plain "what happens" lines. **Golden:** record only new entries.

**Commit.**

---

### Task 7: Sweeps

- `hoard-ratio`: final mean L, or the takeover share, against app_scat at a fixed app_lard. It covers
  0.05–0.9 and is Fig. 2B's axis.
- `hoard-recovery`: against `owner_recovery`.
- `hoard-cheaters`: against the cheater share.
- Register them where Minds 6's sweeps are registered. Run `hoard-ratio` with the CLI and record the table.

**Commit.**

---

### Task 8: WASM

- Pin `hoard-threshold`'s golden in `web.rs`.
- Inspect and frame views follow ZI's `view.rs`: the population panel's data and an agent's traits,
  stores and losses.
- Add a WASM test.

**Commit.**

---

### Task 9: The page

- `hoard` joins `MODELS`, `MODEL_LABELS` and the other lists, mirroring `8d3eb55`.
- The Minds menu gets `MINDS_TITLES['7']`, and its presets group under it.
- **Views:**
  - a population panel of 20 agents, with larder and scatter stores and L;
  - charts: mean L and D by generation, loss rates (larder against scatter), survivors;
  - Inspect.
- **Rules group:** the switches and apparencies, with live or reset set to match the core.
- Tests. Browser check. **Commit.**

---

### Task 10: The survey

`survey/src/claims/minds7.rs`, following the spec's Survey section. **Judges go in their own commit first.**
There are at least 35 runs per condition.

- **All-or-nothing:** judged across a ratio grid.
- **Threshold:** a logistic fit, with the 50 % point judged against 0.219 within a tolerance stated in the
  judge commit. Also: no takeover below 0.2, and takeover almost always above 0.3.
- **Larder loss:** larder above scatter in every run. The CV ratio is reported against 57/33.
- **Predictor:** the minimum larder loss against the mean scatter loss, compared with a means-only
  predictor.
- **Scatter withstands loss:** reported against 18 %.
- **New ground, reported only:** visibility, owner recovery, and the non-hoarding cheater.
- Descriptions and titles are rewritten from the measurements, with causes labeled "likely".
- **Cost:** the µs per agent-bout, added to the program doc's cost table.

**Commit.**

---

### Task 11: Docs

- README: a Minds 7 section.
- The program doc: status "Minds 1–7 done", results, and the next step (P2 watching, or spatial P1b).
- The roadmap line.
- The spec's amendments.
- Run the full suite. **Commit.**
