# SugarScape Milestone 4 — N goods — Design

**Date:** 2026-09-23
**Builds on:** the milestone-1, Chapter IV and Chapter V specs (`2026-09-22-sugarscape-wasm-playground-design.md`, `2026-09-22-chapter-iv-sugar-and-spice-design.md`, `2026-09-23-chapter-v-disease-design.md`); all remain binding where not changed here.
**Source text:** Epstein & Axtell, *Growing Artificial Societies*: Chapter IV footnote 7 ("In the Sugarscape software system the number of commodities, n, is a user adjustable parameter, and so M has actually been implemented as the n-dimensional analog of expression (2)"), footnote 55 (the software "implements the n commodity generalization of credit rule L"), Appendix A (sites hold "resource levels and capacities (both n-dimensional vectors) … pollution levels and fluxes (both m-vectors)"), Appendix B rule P (p = Πr, c = Χm for n resources and m pollutants).

## Goal

Replace the explicit sugar and spice fields with a list of 1–8 goods and a list of 1–4 pollutants, as in the book's own software: N-dimensional Cobb–Douglas welfare, trade over pairs of goods, per-good credit, per-good landscapes (transforms of the two-peaks map, a peaks generator, flat), full pollution matrices, per-good statistics, and a UI to edit goods and pollutants.

## Non-negotiable constraint: earlier runs are unchanged

Every existing preset must evolve byte-identically (its `tests/golden.rs` entry unchanged) **except `vi-1-everything`**, whose credit changes from sugar-only to per-good (re-recorded with a note). Before any core change, the golden table is confirmed green on the branch base. This requires:

- **RNG draw order:** per-agent draws for good 0 happen where sugar's are drawn today; draws for goods 1..n happen, in good order, where spice's are drawn today (only when n ≥ 2). No new draws for n ≤ 2 except where noted (none).
- **Floating-point order:** sums over goods run from 0 in good order starting at `0.0` (or the first term); products start at `1.0` and multiply in good order. `0.0 + x`, `1.0 * x` and `x + 0.0 * y` (for finite y ≥ 0) are exact, so today's expressions are reproduced.
- **Skipped work:** per-good work for goods ≥ 1 is skipped exactly where today's code skips spice.
- **Fingerprint:** `World::fingerprint` hashes goods 0..n and pollutants 0..m in the order and conditions that reproduce today's hashes (good 1 hashed where spice is hashed today, only when n ≥ 2).

## Configuration

### Goods

`goods: Vec<Good>` (1–8 entries; good 0 always exists). Replaces top-level `metabolism`, `endowment` and `landscape`, and the `spice` block.

| Field | Meaning |
|---|---|
| `name: String` | display name (1–16 chars, unique) |
| `color: String` | `#rrggbb`, used by layers, charts, inspector |
| `map: Map` | capacity map (below) |
| `metabolism: URange` | per-tick burn drawn for new agents |
| `endowment: URange` | initial holding drawn for new agents |

`Map` (`#[serde(tag = "kind")]`):
- `two_peaks { transform: Transform }` — the book's 50×50 two-peaks map (valid only at 50×50, as today) under one of `identity`, `rotate_90`, `rotate_180`, `rotate_270`, `mirror_x` (left↔right; today's spice map), `mirror_y`, `transpose`, `anti_transpose`.
- `peaks { peaks: Vec<Peak { x, y, radius: f64, height: f64 }> }` — 1–16 peaks; capacity at a site = max over peaks of `max(0, ceil(height × (1 − d / radius)))`, d = torus Euclidean distance from the site to (x, y); radius > 0, height 0–10.
- `flat { capacity: f64 }`.

Growback and seasons stay global and apply to every good (as today to sugar and spice).

Rule dependencies: trade and foresight need n ≥ 2 (replacing "needs spice"); combat needs n = 1 (replacing "combat excludes spice").

**Reset-only:** `goods` (the list, its length, and each good's `map`); `goods.N.metabolism` / `goods.N.endowment` / `name` / `color` apply live (affect new agents / display only). Adding or removing a good or pollutant rebuilds the world.

### Pollutants

`pollution: { enabled: bool, pollutants: Vec<Pollutant> }` (1–4 entries), replacing `production`, `consumption`, `spice_pollutes`.

| Field | Meaning |
|---|---|
| `name: String` | display name |
| `production: Vec<f64>` | Πₖᵢ per good (length n, ≥ 0) |
| `consumption: Vec<f64>` | Χₖᵢ per good (length n, ≥ 0) |
| `devalues: Vec<bool>` | goods whose site value this pollutant reduces (length n) |

**Formation** (on the agent's site, after metabolizing): `pₖ += (Σᵢ Πₖᵢ·gatheredᵢ) + (Σᵢ Χₖᵢ·burnedᵢ)`, sums in good order from 0. **Diffusion** D_α applies to each pollutant separately. **Movement:** good i on a site counts as `xᵢ / (1 + Σ_{k: devaluesₖᵢ} pₖ)` (sum in pollutant order).

Adding/removing a pollutant is reset-only; coefficients and `devalues` apply live and are schedulable (`pollution.pollutants.K.production.I`).

### Legacy configs

Deserialization goes through a legacy-aware conversion so existing share links, presets and saved configs load unchanged. A config is legacy if it has no `goods` key:
- good 0 = `{ name: "sugar", color: SUGAR, map: from landscape (TwoPeaks → two_peaks/identity, Flat → flat), metabolism, endowment }`;
- if `spice.enabled`: good 1 = `{ name: "spice", color: SPICE, map: sugar's map with transform mirror_x (flat stays flat), metabolism: spice.metabolism, endowment: spice.endowment }`;
- one pollutant `{ name: "pollution", production: [α, α if spice_pollutes else 0], consumption: [β, β if spice_pollutes else 0], devalues: [true, spice_pollutes] }` (length n);
- schedule paths rewritten: `pollution.production` / `pollution.consumption` → one entry per polluting good (`pollution.pollutants.0.production.I`); `spice.metabolism*` / `spice.endowment*` → `goods.1.…`; `metabolism*` / `endowment*` → `goods.0.…`.

Export always writes the new shape.

## Model

- **Site:** `resource: [f64; 8]`, `capacity: [f64; 8]`, `pollution: [f64; 4]` (only 0..n / 0..m meaningful).
- **Agent:** `holdings: [f64; 8]`, `initial: [f64; 8]`, `metabolism: [u32; 8]` (only 0..n meaningful). Good 1's draws happen where spice's are today.
- **Welfare:** `W = Πᵢ wᵢ^(mᵢ/m_T)`, `m_T = Σᵢ mᵢ`; all-zero metabolisms weigh goods equally (1/n); negative holdings count as 0. Foresight subtracts φ·mᵢ per good. Single-good movement (n = 1) keeps today's rule (site value = discounted resource).
- **MRS** for goods i < j: `MRSᵢⱼ = (wⱼ·mᵢ)/(mⱼ·wᵢ)` (units of j worth one unit of i; at (0,1) today's MRS; with all-zero metabolisms `wⱼ/wᵢ`).
- **Trade (widest gap first):** when two neighbors meet, rank pairs i < j by `|ln MRS_A − ln MRS_B|` descending (ties: lowest (i, j)); the first pair passing the book's checks gets one unit exchange at `p = √(MRS_A·MRS_B)` (1 of i for p of j if p ≥ 1, else 1/p of i for 1 of j), then re-rank. Checks: both agents' full N-good welfare strictly rises, their pair MRSs don't cross, all holdings stay positive. Stop when no pair can exchange, or after the existing 10,000-exchange cap. Pairs with non-finite or non-positive MRS are skipped. At n = 2 this is today's loop.
- **Trade event:** `Trade { buyer, seller, goods: (i, j), price, amount }` (amount = units of i).
- **Credit (per good):** lendableᵢ and a would-be parent's shortfallᵢ use good i's endowment; incomeᵢ = gatheredᵢ − effective metabolismᵢ − obligationsᵢ; creditworthiness per good. Each `Loan` carries its `good`. Settlement pays in full if the borrower holds more than is due in that good, otherwise half (re-lent), as today. The borrower shuffles neighbors once per turn and borrows goods in order; lender inheritance unchanged per loan. At n = 1 this is today's rule.
- **Life cycle:** death when any good ≤ 0; fertile when every good ≥ its initial amount (goods with initial ≤ 0 excepted, as today); child endowment of good i = ½ each parent's initialᵢ; metabolism inherited per good; inheritance splits every good; the disease fee is added to every good's metabolism.
- **Supply and demand** (Chapter IV's chart): computed for the pair (0, 1) only, with weights restricted to that pair.

## Statistics

Existing series keep their names and meanings: `mean_wealth`, `gini`, `mean_metabolism` (good 0); `mean_spice`, `mean_spice_metabolism` (good 1, zero when n = 1); `mean_log_price`, `sd_log_price`, `sugar_traded` (pair (0, 1) only); `trade_volume` (all exchanges). Appended per good i: `mean_holding_i`, `mean_metabolism_i`, `traded_i` (units of good i exchanged this tick); per pollutant k: `mean_pollution_k`. `trade_pairs` = distinct pairs that traded this tick. The `SERIES` list is built from the config's n and m.

## WASM API

- `render(colorMode, layer)`: layers `resource:I`, `capacity:I`, `pollution:K`; legacy aliases `sugar` = `resource:0`, `spice` = `resource:1`, `capacity` = `capacity:0`, `spice_capacity` = `capacity:1`, `pollution` = `pollution:0`. Resource and capacity layers blend from the background to the good's color.
- `paint_capacity(x, y, radius, value, good)`; `export_landscape(good)`; `Sim.new(config, seed, landscapes?)` where `landscapes` is a list of per-good capacity arrays (null entries use the generated map). A legacy single sugar array is accepted as good 0.
- Inspection: `SiteView { resources, capacities, pollution }`, `AgentView { holdings, metabolism, initial, … }` (arrays of length n / m); `LoanView.good`. CSV exports append one column per good and per pollutant.

## UI

- **Goods group** (replaces Spice): one row per good — name, color, map (kind; transform, peaks list with add/remove, or flat capacity), metabolism range, endowment range; add (≤ 8) and remove (≥ 1) buttons, which rebuild the world. Controls needing n ≥ 2 say so.
- **Pollution group:** enable toggle plus a table — one row per pollutant, columns per good for production, consumption and a "devalues" checkbox; add/remove pollutant (1–4).
- **Display:** the Landscape selector lists resource and capacity per good (by name) and each pollutant.
- **Paint tool:** a good picker.
- **Charts:** a Goods section (holdings, metabolism, traded — one line per good in its color) and a Pollution chart (one line per pollutant). Price and supply-and-demand charts are labelled with the pair ("sugar/spice").
- **Inspector:** per-good site and agent rows; loans show their good.
- **Share links:** carry per-good painted maps only for goods whose map differs from the generated one; legacy links decode through the conversion.

## Presets

| id | Setup |
|---|---|
| `n-3-trade` | sugar, spice, salt (two-peaks `rotate_90`); Chapter IV market parameters with demography; three-way price formation |
| `n-4-peaks` | four goods on `peaks` maps, one peak near each corner; trade on |
| `n-2-pollutants` | sugar produces "smoke" (devalues sugar), spice produces "runoff" (devalues spice); pollution and diffusion on |

Exact parameters are measured during implementation so the population survives, and recorded in comments (as with `iv-18-foresight`).

## Testing

- **Golden:** every existing entry unchanged except `vi-1-everything` (re-recorded, with a comment); entries added for the new presets.
- **Unit:** N-good welfare/MRS equal today's two-good functions at n = 2 (bit-exact) and at n = 1; widest-gap pair choice and tie-break; trade conserves each good; the 8 transforms; peaks capacities; pollution formation and the devalues discount; per-good credit (borrow, settle, default); legacy conversion of config and schedule paths.
- **Round trip:** every existing preset exported in the legacy shape and loaded through the conversion equals the preset's new config; a saved legacy share link from before this milestone loads and runs (fixture).
- **Property:** holdings positive after trade; each good's total unchanged by trade; pollution ≥ 0; resource ≤ capacity.
- **Book-style (`#[ignore]`, release, measured values in comments):** in `n-3-trade` the cross-agent spread of ln MRS shrinks over time for every pair; with three goods, carrying capacity with trade exceeds without.
- **Browser:** the controller checks the Goods and Pollution editors, adding/removing goods, per-good painting, layers, charts, inspector and a legacy share link with the puppeteer harness.
