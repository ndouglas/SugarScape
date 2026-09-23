# SugarScape Milestone 2 — Chapter IV: Sugar and Spice — Design

**Date:** 2026-09-22
**Builds on:** `docs/superpowers/specs/2026-09-22-sugarscape-wasm-playground-design.md` (milestone 1; still binding where not changed here).
**Source text:** Epstein & Axtell, *Growing Artificial Societies*, Chapter IV and Appendix B.
**Future work:** `docs/roadmap.md` (N-commodity generalization, Chapter V, and more).

## Goal

Add the book's second commodity and its economy — spice, multicommodity movement, bilateral trade (T), credit (L), foresight — plus trade/credit network overlays, a supply-and-demand view, sugar-as-dirty-good pollution, and scheduled rule changes so presets can replay the book's timed experiments.

## Non-negotiable constraint: milestone-1 runs are unchanged

With every new rule off (`spice`, `trade`, `credit`, `foresight` disabled, empty `schedule`), a world must evolve **byte-identically** to milestone 1 — same RNG draw order, same results. Concretely:

- New agent traits (spice metabolism, spice endowment, foresight) are drawn from the RNG **only when their rule is enabled**.
- New per-turn steps are skipped entirely when their rule is off.
- Before any core change, the fingerprint (`World::fingerprint`) of every existing preset after 200 ticks (seed 1) is recorded as a golden value; a test asserts they are unchanged. The existing book tests must keep passing with their current bounds.

## Approach

Sugar and spice are **explicit fields** (not an N-goods vector); spice is a rule toggle. The N-goods generalization is deferred to a future campaign (roadmap).

## Configuration additions

All new blocks have `Default`s so existing configs and share links deserialize unchanged.

| Block | Fields | Defaults |
|---|---|---|
| `spice` | `enabled`, `metabolism: URange`, `endowment: URange` | off, 1–4, 5–25 |
| `trade` | `enabled` | off |
| `credit` | `enabled`, `duration: u32` (d), `rate: f64` (r, percent) | off, 10, 10.0 |
| `foresight` | `enabled`, `range: URange` | off, 0–10 |
| `pollution` | + `spice_pollutes: bool` | false |
| `schedule` | `Vec<ScheduledChange { tick: u64, set: BTreeMap<String, serde_json::Value> }>` | empty |

**Validation (new rules):** trade requires spice; foresight requires spice; credit requires sex; combat and spice are mutually exclusive (the book never combines them); `credit.duration ≥ 1`; `credit.rate` finite and ≥ 0; ranges min ≤ max. Each schedule entry: `tick ≥ 1`; every key must be a dot path to an existing config field; structural fields (`width`, `height`, `tag_length`, `landscape`, `population`, `placement`) are rejected, as are `spice` / `spice.enabled` (reset-only, below) and any path under `schedule` (a schedule may not edit itself); applying all of an entry's `set` values to a copy of the config must yield a valid config. A new world validates the whole schedule; a mid-run config change (`set_config`) validates only entries with `tick ≥` the world's current tick, since earlier entries already fired and their effects are part of the live config.

**Reset-only:** `spice.enabled` joins the fields a running world rejects ("changes only on reset"): switching spice on mid-run would starve agents born without spice, and switching it off would leave spice endowments nobody can meet.

**Landscape:** spice capacities for `TwoPeaks` are the sugar map mirrored left↔right (spice mountains in the northwest and southeast, as in the book's Figure IV-1). `Flat { capacity }` uses the same capacity for spice. Sites gain `spice` and `spice_capacity`. Growback (rate, instant, seasons) applies to both goods identically.

## Agents

New fields: `spice`, `initial_spice`, `spice_metabolism`, `foresight: u32`, and per-turn bookkeeping `income: f64` (sugar gathered minus sugar metabolism minus per-tick loan obligations, for credit). When spice is disabled, spice fields are 0 and never drawn; when foresight is disabled, foresight is 0 and never drawn.

## Welfare and valuation (`econ.rs`)

- Welfare (book eq. 1): `W(w₁, w₂) = w₁^(m₁/m_T) · w₂^(m₂/m_T)`, `m_T = m₁ + m₂`.
- Foresight (book eq. 6): movement evaluates `W(max(0, w₁ − φm₁), max(0, w₂ − φm₂))` on post-gathering holdings.
- MRS (book eq. 3): `MRS = (w₂/m₂) / (w₁/m₁)` — units of spice per unit of sugar. Trade uses actual holdings (no foresight).
- Cobb–Douglas sugar demand at price p (spice per sugar): `x₁*(p) = (m₁/m_T) · (p·w₁ + w₂) / p`; excess demand `e(p) = x₁*(p) − w₁`.

## Tick order (changes in **bold**)

1. **Apply scheduled changes whose `tick` equals the tick about to run** (each entry once), then validate; the resulting config becomes the world's config.
2. Shuffle living agents; for each (skipping any killed earlier this tick):
   1. Move — M (single-good rule unchanged when spice is off; **multicommodity M when spice is on**) or C.
   2. Metabolize — sugar, **and spice when on**; pollution formation (**spice counts only if `spice_pollutes`**).
   3. Death check — sugar ≤ 0, **or spice ≤ 0 when spice is on**, or age > max age (lifespan on).
   4. Sex (S).
   5. Culture (K).
   6. **Trade (T)** — when on.
   7. **Credit (L) — borrowing** — when on.
3. **Settle loans due this tick** (credit on), in loan-id order.
4. Growback, pollution diffusion, replacement, ageing (as in milestone 1, replacement before ageing).
5. Stats.

## Multicommodity movement (spice on)

Candidates are the current site (distance 0) plus unoccupied sites in sight (unchanged geometry). Value of a site = foresight-adjusted `W(w₁ + x₁′, w₂ + x₂)` where `x₁′ = x₁ / (1 + pollution)` when pollution is on (sugar is the "dirty" good), and likewise `x₂′ = x₂ / (1 + pollution)` if `spice_pollutes`. Ties: max value → nearest → uniformly random (existing `choose`). Gather all sugar and spice at the destination.

## Trade (T)

For each von Neumann neighbor, in random order (once per neighbor per agent turn):

1. Compute both MRSs. If equal, stop.
2. The agent with the higher MRS buys sugar and sells spice; the other the reverse.
3. Price `p = √(MRS_A · MRS_B)`.
4. Quantities: if `p ≥ 1`, 1 sugar ↔ `p` spice; else `1/p` sugar ↔ 1 spice.
5. Execute only if both agents' welfare strictly increases **and** their MRSs do not cross (the ordering of MRS_A vs MRS_B is preserved) **and** both keep positive holdings of both goods. If executed, record the trade (price, sugar quantity) and go to 1; otherwise stop with this neighbor.

Each executed exchange is one trade. The pair is recorded in this tick's trade network.

## Credit (L) — sugar loans, book's single-commodity form

- **Potential lender:** too old to have children (`age > fertility_end`) → may lend up to half its sugar; or of childbearing age with sugar above its birth endowment → may lend the excess.
- **Potential borrower:** of childbearing age, sugar below its birth endowment, and positive `income` this turn. Need = endowment − sugar.
- **Creditworthiness (interpretation):** a loan of principal P is acceptable if `income × d ≥ P × (1 + r/100 × d)` (simple interest over the loan's life).
- **Origination (during the borrower's turn):** a potential borrower asks each neighbor in random order; each potential lender lends `min(its available amount, remaining need, largest creditworthy P)`. A loan records lender, borrower, principal, amount due `P × (1 + r/100 × d)`, due tick `now + d`, and its terms `d` and `r` (which travel with the loan; later changes to the credit settings apply only to new loans).
- **Settlement (step 3):** at the due tick, if the borrower's sugar > amount due (paying in full must leave the borrower alive), it pays in full. Otherwise it pays half its sugar and a **new loan** (the original loan's `d` and `r`, fresh due date) is originated for the remainder — counted as a default.
- **Deaths:** borrower dies → loan cancelled (lender's loss). Lender dies → loan cancelled, unless inheritance (I) is on, in which case the claim is split equally among the lender's living children (one new loan per child, same terms and due tick; a child who is itself the borrower has its share forgiven, so no one owes itself).
- `income` = sugar gathered this turn − sugar metabolism − Σ(amount due / the loan's own d) over the agent's outstanding loans as borrower.

## Extras

- **Sex, inheritance, replacement with spice:** fertility requires sugar ≥ initial sugar **and**, for an agent born with spice traits (initial spice > 0), spice ≥ initial spice; each parent gives half of each birth endowment; child foresight is inherited from a random parent; inheritance splits both goods; replacements draw spice traits (and foresight) when enabled.
- **Supply and demand (on demand):** over 41 log-spaced prices in [0.1, 10], aggregate demand `D(p) = Σ max(e, 0)` and supply `S(p) = Σ max(−e, 0)`; the equilibrium price is where `D − S` changes sign (log-linear interpolation) and the equilibrium quantity is `D` there. Reported with the tick's actual geometric-mean trade price and total sugar traded.
- **Networks:** trade edges = pairs that traded this tick; credit edges = outstanding loans between living agents. Returned as agent positions.
- **Credit color mode:** lender only (green), borrower only (red), both (yellow), neither (neutral gray).

## Statistics (appended to `SERIES`)

`mean_log_price` (mean of ln p over this tick's trades; 0 if none), `sd_log_price`, `trade_volume` (trades this tick), `sugar_traded`, `loans_made`, `amount_lent`, `defaults`, `debt_outstanding`, `mean_foresight`, `mean_spice`, `mean_spice_metabolism`.

## WASM API additions

- `render(mode, layer)`: mode adds `credit`; layer adds `spice`, `spice_capacity`.
- `networks(kind: "trade" | "credit") → Uint32Array` of `[x1, y1, x2, y2, …]`.
- `supply_demand() → Float64Array`: `[n, prices(n), demand(n), supply(n), ge_price, ge_quantity, actual_price, actual_quantity]` (NaN where undefined).
- `inspect` adds spice, initial spice, spice metabolism, foresight, and the agent's loans (as lender and as borrower: counterparty id, amount due, due tick).
- Config JSON carries the new blocks and `schedule`.

## Presets (added)

| id | Rules | Parameters |
|---|---|---|
| `iv-1-spice` | ({G₁}, {M}) with spice | vision 1–10; sugar & spice metabolism 1–5; endowments 25–50 |
| `iv-3-trade` | ({G₁}, {M, T}) | 200 immortal agents; vision 1–5; metabolisms 1–5; endowments 25–50 |
| `iv-15-trade-sex` | ({G₁}, {M, S, T}) | as `iv-3-trade` + sex and lifespan 60–100 (finite lives, evolving preferences) |
| `iv-3-pollution` | ({G₁, D₁}, {M, T, P}) scheduled | `iv-3-trade`; t=100: pollution on (sugar only); t=150: production and consumption set to 0 (pollution stays on, so what remains still repels agents), diffusion on |
| `iv-18-foresight` | ({G₁}, {M, S}) with spice + foresight | foresight 0–10; demography as `iii-2-sex`; sugar & spice endowments 25–50 (the book-scale 50–100 made the population go extinct under the two-good fertility requirement — sugar *and* spice each ≥ their own initial endowment, on largely anti-correlated terrain) |
| `iv-5-credit` | ({G₁}, {M, S, L₁₀,₁₀}) sugar only | as `iii-2-sex` + credit (d = 10, r = 10) |

Milestone-1 presets gain the schedules the book describes: `ii-8-pollution` → pollution on at t=50, diffusion on at t=100 (starting with both off). This changes that preset's golden fingerprint intentionally; the invariance test records it after the change.

## UI

- **Rules panel:** groups Spice, Trade, Credit, Foresight; "Spice pollutes" under Pollution; a read-only **Schedule** list ("t=100 · pollution.enabled = true") with a Clear button. New validation errors route to their controls. The Spice switch rebuilds the world (reset), like the Setup controls. The panel shows the **live** config (including scheduled changes that have fired); the engine also keeps the **base** config (the setup as chosen by preset, share link or reset), which reset, share links, preset matching and the "modified" badge use. A mid-run edit is applied to both, so it never undoes a fired scheduled change; the live config is re-read after any tick range that crosses a scheduled entry.
- **Display bar:** new layers and the credit color mode; "Trade network" and "Credit network" overlay checkboxes. Overlays draw lines between cell centers; an edge whose endpoints are more than half the grid apart on an axis is drawn as two segments across the torus edge.
- **Charts:** an **Economy** group, shown when spice or credit is on. With spice on: mean log price with ±1 SD band, trade volume, supply & demand (curves + equilibrium point + actual point), and a separate **Spice & foresight** chart (mean spice metabolism and mean foresight). With credit on: loans (made, defaults) and outstanding debt. Each chart is hidden while its rule is off.
- **Inspector:** spice holdings/metabolism, foresight, loans with clickable counterparties.
- **Exports:** series and agent CSVs include the new columns.

## Testing

- **Golden invariance test** (see constraint above), recorded before the first core change.
- **Unit tests:** welfare, MRS, foresight floor; trade direction, price, no-crossover termination, positivity; the book's Edgeworth-box example (A = (5, 8), B = (15, 2) with equal metabolisms) converges without crossing; credit origination, creditworthiness, full repayment, default rollover, borrower death, lender death with and without inheritance; supply/demand equilibrium ≈ 1 for a symmetric population; schedule application at the right tick and rejection of invalid entries; spice mirror map.
- **Property tests:** extended to spice and credit: spice ≤ capacity; living agents have positive sugar (and spice when on); every outstanding loan references living agents; sugar is conserved by trade (sum of both parties' sugar unchanged by a trade).
- **Book reproductions (`#[ignore]`, release):** trade price clusters near 1 (mean `mean_log_price` over ticks 500–1000 of `iv-3-trade`, averaged over seeds, within ±0.25); carrying capacity with trade exceeds without (same preset with trade off; mean population at t=500 over ≥ 3 seeds); mean foresight in `iv-18-foresight` stays above 0 and falls below its initial mean. Bounds are set from measured values, recorded in comments.
- **Browser:** the controller runs a puppeteer checklist for the new controls, overlays, charts and scheduled preset.
