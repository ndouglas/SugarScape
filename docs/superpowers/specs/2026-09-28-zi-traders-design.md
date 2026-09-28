# SugarScape Milestone 28 — Zero-Intelligence Traders — Design

**Date:** 2026-09-28
**Builds on:** the milestone 1–27 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–27, the preset titles of `crates/sugarscape-core/src/titles.rs`, and milestone 27's method for figures (read from the PDF by pixel; one fit rule fixed before measuring, with each curve's worst point).
**Source texts** (local copies):
- Dhananjay K. Gode and Shyam Sunder, "Allocative Efficiency of Markets with Zero-Intelligence Traders: Market as a Partial Substitute for Individual Rationality," *Journal of Political Economy* 101(1): 119–137 (1993), `papers/zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (GS below; a scan, read by OCR at 300 dpi).
- The critique and follow-up: Dave Cliff, "Minimal-Intelligence Agents for Bargaining Behaviours in Market-Based Environments," HP Laboratories Technical Report HPL-97-91 (1997), `papers/zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff below; page numbers are the report's printed ones). Its appendices hold the C source that produced its results.

## Goal

Gode and Sunder's double auction with zero-intelligence traders and Cliff's critique and ZIP traders as one model kind, `zi` ("Zero-Intelligence Traders"), a full citizen of the playground: GS's five markets with and without the budget constraint (their Figs. 1–8, Tables 1–3), Cliff's four markets and closed-form E(P) predictions for ZI-C, and his ZIP traders (the four markets, the demand and supply shifts, the sellers-only retail market); every detail the texts leave open — above all how many shouts GS's "30 seconds" allowed, which trader holds which units, and whether Cliff's mechanism is GS's — a named switch or a stated choice, and every claim measured.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs. Where Cliff's text and code differ, the code — which produced his results — is the default, and the text a switch.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only). Prices are integers (GS: 1–200; Cliff: cents, 1–400); ZIP margins are `f64` updated with `+ − × ÷` only.
- **Truthful descriptions and titles:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **GS traders:** "Each ZI trader generated random bids or offers … distributed independently, identically, and uniformly over the entire feasible range of trading prices from 1 to 200." ZI-C: "if they generated a bid (to buy) above their redemption value or an offer (to sell) below their cost, such actions were considered invalid and were ignored … the support of the distribution … was restricted between one and the redemption value … [asks] between the seller's cost and 200." ZI-U: "random bids and offers were permitted over the entire range (1–200)."
- **GS mechanism:** "any buyer can enter a bid … The same buyer or other buyers can subsequently raise the bid … If bids and asks match or cross, a binding transaction occurs … Each bid, ask, and transaction was valid for a single unit. A transaction canceled any unaccepted bids and offers. Finally, when a bid and ask crossed, the transaction price was equal to the earlier of the two." "Each double auction consisted of 12 traders, equally divided into two groups … six periods of specified duration (4 minutes for human traders and 30 seconds for machine traders) … Every trader had to trade the ith unit before trading the (i + 1)th unit."
- **GS markets:** "five sets of demand and supply schedules … a broad range of equilibrium prices (from 69 in market 2 to 170 in market 4) and volumes (from 6 in market 3 to 24 in market 1) … In market 5, costs and redemption values of all the units of several buyers and sellers were placed just beyond the equilibrium point." The schedules exist only as step curves beside Figs. 1–5.
- **GS results:** ZI-C prices "converge slowly toward equilibrium within each period" (Fig. 6; Table 1: regression slopes of RMS deviation on transaction number −0.64, −0.61, −1.23, −3.59, −0.83); efficiency (Table 2) ZI-U 90.0, 90.0, 76.7, 48.8, 86.0; ZI-C 99.9, 99.2, 99.0, 98.2, 97.1; human 99.7, 99.1, 100.0, 99.1, 90.2 ("the maximum possible number of units … is always traded" by ZI-U); profit dispersion (Table 3, RMS of actual − equilibrium profits) ZI-U 225.48, 253.12, 90.54, 363.80, 156.28; ZI-C 28.53, 49.81, 15.90, 60.47, 19.07; footnote 5: Spearman correlation of the actual and efficient order of surplus extracted, ZI-C .74, ZI-U .42, human .52.
- **Cliff's critique of ZI-C** (pp. 27–38): GS's convergence claim "is proven below to be incorrect"; the expected ZI-C price is the mean of the pdf formed by the intersection of the bid and offer pdfs, equal to P₀ only for symmetric schedules. Four markets (P₀ = 200 cents, Q₀ = 6, Pmax 400, 10 days, 50 runs): symmetric (11 buyers and 11 sellers, limits 75–325 by 25), E(P) = 200; flat supply (11 sellers at 200, buyers 75–325), E(P) "233⅓" (his Eq. 5 gives 241⅔; summing his discrete pdf gives 245⅓); box, excess demand (11 buyers at 200, 6 sellers at 50), E(P) = 125; box, excess supply (6 buyers at 320, 11 sellers at 200), E(P) = 260. Within-day RMS falls in the symmetric and flat markets and stays flat in the box markets.
- **Cliff's mechanism, as coded** (`smith.c`, pp. 100–111): a side chosen with probability proportional to its active traders, a random able trader on it shouts; every active trader on the other side decides whether it is willing (a ZI-C trader draws a fresh random price and is willing if it strictly beats the shout; a ZIP trader if its current price crosses it); a random willing trader trades at the shout's price. An NYSE flag bars shouts that cannot beat the day's best quote (ZI-C: by limit) and resets the book on every trade. A day ends after 100 consecutive failed shouts or when the chosen side has no able trader.
- **ZIP** (pp. 42–45 and `agent.c`): price = limit × (1 + μ); sellers μ ≥ 0, buyers μ ∈ [−1, 0]; after each shout at q, traders raise or lower their margins toward a target τ = R·q + A (raising: R ~ U[1, 1.05], A ~ U[0, 0.05] dollars; lowering: R ~ U[0.95, 1], A ~ U[−0.05, 0]); Δ = β(τ − p), Γ = γΓ + (1 − γ)Δ, μ = (p + Γ)/limit − 1, an update that would cross zero rejected; β ~ U[0.1, 0.5]; μ₀ ~ U[0.05, 0.35] (sellers; buyers the negative); **γ ~ U[0.2, 0.8] in the text, but the code overwrites it with U[0, 0.1]**. Results: the symmetric and flat markets converge to $2.00 "typically within the first four trading days"; the box markets approach "from below", slowly; efficiency "often averaging 100%"; profit dispersion "in some cases approximately a factor of ten less" than ZI-C's; after a +$0.50 demand shift (P₀ $2.25) or −$0.50 supply shift ($1.75) after day 10, prices re-converge; a sellers-only retail market (12 buyers, 11 sellers, P₀ $2.25) settles below P₀.

## Measured in planning

A throwaway prototype (Python) of the rules below; GS markets 20 seeds × 6 periods of 2 000 shouts unless stated; Cliff's markets 50 seeds × 10 days. The survey reproduces each with the implementation.

- **The five GS markets, digitized** (the text's anchors: P₀ 69 in market 2, 170 in market 4; volumes 24 in market 1, 6 in market 3; and ZI-U's efficiency, which follows arithmetically from the schedules): six identical buyers and six identical sellers, one unit per step, in markets 1–4 — market 1 values 102, 97, 92, 87, 82, 77 and costs 34, 46, 58, 70, 82, 94; market 2 values 117, 105, 93, 81, 69, 57 and costs 49, 54, 59, 64, 69, 74; market 3 values 133, 95, 90 and costs 90, 95, 100; market 4 values 180, 175, 170, 165, 160 and costs 90, 142, 170, 190, 198 (142 reads 141 by pixel; 142 reproduces Table 2). ZI-U efficiency: 90.0, 90.0, 76.7, 48.8 — Table 2 exactly. Market 5 is a fine staircase: 18 intramarginal units a side (values 159 down to 131, costs 96 up to 124, evenly), held by three buyers and three sellers (six each, dealt in turn), and three buyers and three sellers holding seven extramarginal units each at 127 and 131; P₀ 129; ZI-U 86.7 against 86.0 — approximate, as the scan allows.
- **GS's results reproduce, given enough shouts:** ZI-C efficiency 99.9, 99.8, 99.7, 99.6, 97.0 (Table 2: 99.9, 99.2, 99.0, 98.2, 97.1); ZI-C dispersion 31.9, 50.8, 17.1, 62.4, 24.8 (Table 3: 28.5, 49.8, 15.9, 60.5, 19.1); ZI-U dispersion 173, 234, 98, 390, 227 (225, 253, 91, 364, 156); ZI-C prices tighten within a period (the second half's RMS about half the first's).
- **The unstated period length decides efficiency:** ZI-C efficiency in market 1 at 25, 50, 100, 200, 500, 2 000 shouts a period: 19, 36, 61, 82, 98, 99.8 (market 5: 10, 20, 40, 65, 93, 97). GS's figures need at least about 500–1 000 shouts a period; their "30 seconds" is never translated. Cliff's day end (100 failures in a row) costs 1–4 points; his side-first turns change nothing.
- **Cliff's critique holds in direction, but on his mechanism, not GS's.** ZI-C mean prices (P₀ 200): symmetric 199.7 (book), 200.9 (Cliff's mechanism); flat supply 216 (book), 234.5 (Cliff's) — his "233⅓" matches his simulation, not his formula (241⅔ or 245⅓); excess demand 162 (book), 136 (Cliff's; predicted 125); excess supply 232 (book), 250 (Cliff's; predicted 260). His simulations reproduce; his predictions miss the box markets by 10–11; and in GS's own mechanism the prices sit about half as far from P₀.
- **ZIP (Cliff's mechanism):** daily mean prices over 10 days — flat supply 226, 208, 203, 202, 201 … (converges by day 4, as stated); symmetric 181 … 193 (from below, as he notes, but still 7 below P₀ at day 10); excess demand 129 … 133 with the code's momentum, 131 … 149 with the text's; excess supply 246 … 232 (code), 245 … 222 (text). The momentum discrepancy triples the speed of convergence in the box markets.

## Architecture

Model kind `zi` ("Zero-Intelligence Traders"): `ModelKind::Zi`, `ModelConfig::Zi(ZiConfig)` tagged `"model": "zi"`, a `ZiWorld` implementing `Model`, schema, `SERIES`, presets, titles and golden entries. Code in `crates/sugarscape-core/src/zi/` (`config.rs`, `market.rs` — the built-in schedules, equilibrium and E(P) —, `world.rs`, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`). One tick is one shout.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `market` | `gs1` | reset | `gs1`–`gs5`, `symmetric`, `flat_supply`, `excess_demand`, `excess_supply`, `retail` (12 buyers, 11 sellers, P₀ 225, Q₀ 7; its limits read from Cliff's Fig. 50 in planning), or `custom` |
| `buyers`, `sellers` | [] | reset | `custom` only: each trader's unit limits, in trading order |
| `price_max` | 200 | reset | the top of the price range (GS 200; Cliff 400 cents); prices run from 1 |
| `strategy` | `zi_c` | reset | `zi_u`, `zi_c` or `zip` |
| `mechanism` | `book` | live | `book` (GS: a standing best bid and ask; a crossing shout trades at the earlier order's price; a trade clears the book) or `cliff` (the other side's willing traders, one chosen at random, at the shout's price) |
| `nyse` | true | live | `cliff` only: shouts that cannot beat the day's best quote are barred; the book resets on each trade |
| `turns` | `trader` | live | `trader` (a random trader with units left) or `side` (Cliff: a side by its active traders, then a trader) |
| `sellers_only` | false | live | only sellers shout (Cliff's retail market) |
| `period_end` | `shouts` | live | `shouts` (a fixed number) or `failures` (Cliff: 100 failed shouts in a row, or a side with no one able) |
| `shouts` | 2000 | live | shouts a period under `shouts` |
| `momentum` | `code` | reset | ZIP's γ: `code` U[0, 0.1] (what Cliff ran) or `text` U[0.2, 0.8] |
| `shift` | `none` | live | `demand` (+50 on every buyer's limit) or `supply` (−50 on every seller's) from period `shift_at` |
| `shift_at` | 11 | live | the first shifted period |
| `stop_at` | 6 | live | `finished()` after this many periods (0: never) |

## Step (one shout)

A trader acts (by `turns`; under `sellers_only`, sellers only) and shouts: ZI-U uniform on 1–`price_max`; ZI-C uniform between its limit and the edge of the range; ZIP its limit × (1 + μ), rounded. Under `book`, a bid at or above the standing ask (an ask at or below the standing bid) trades at the standing order's price; otherwise it replaces the standing bid (ask) if better. Under `cliff`, the other side's willing traders are found and one drawn. A trade moves both traders to their next unit and clears the book. ZIP traders then update their margins by Cliff's code. At the end of a period (by `period_end`), statistics are recorded and every trader's units are restored (ZIP margins persist).

## Statistics

`SERIES` (per shout; period values held until the period ends): `price` (this shout's trade, NaN if none), `mean_price`, `volume`, `efficiency` (profit earned over the maximum surplus, this period so far), `rmsd` (RMS deviation from P₀ of this period's trades), `alpha` (100 × `rmsd`/P₀), `dispersion` (RMS of each trader's profit minus its equilibrium profit), `period`, `p0`. Per period, for the survey: every trade's price and order, and the Spearman correlation of the actual and efficient order of surplus extracted.

## Views

- **Schedules** (left): the market's demand and supply steps, P₀ and Q₀ dashed; the units traded this period marked.
- **Prices** (right): every trade's price across the periods, P₀ a line, periods separated — GS's panels.
- **Traders** (below): one column per trader, buyers then sellers, its height its profit this period against a mark at its equilibrium profit.
- **Color modes:** **Side**, **Profit** (above or below the equilibrium profit), **Margin** (ZIP).
- **Inspect:** a trade (period, price, buyer, seller), a trader (limits, units traded, profit against its equilibrium profit, margin), a step of the schedules.
- **Charts:** Prices (`mean_price`, `p0`); Efficiency; Convergence (`alpha`, `rmsd`); Profit dispersion; Volume. Time axis: Shouts.

## Presets

Titles follow `titles.rs`'s style; drafts, to be measured.

| Preset | Setup |
|---|---|
| `gs-1`–`gs-5` | GS's markets with ZI-C |
| `gs-1-u`, `gs-4-u` | markets 1 and 4 with ZI-U (market 4's 48.8 % is the lowest baseline) |
| `cliff-symmetric`, `cliff-flat`, `cliff-excess-demand`, `cliff-excess-supply` | ZI-C in Cliff's markets, his mechanism, 10 days |
| `zip-symmetric`, `zip-flat`, `zip-excess-demand`, `zip-excess-supply` | ZIP in the four markets |
| `zip-demand-shift`, `zip-supply-shift` | ZIP, symmetric, shifted after day 10, 20 days |
| `zip-retail` | ZIP, the sellers-only market |

**Compare entries:** "With vs without the budget constraint — Zero-Intelligence Traders (Compare)" (`gs-1`, `gs-1-u`); "ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)" (`cliff-excess-demand`, `zip-excess-demand`).

## Experiments and CLI

Seeds and horizons recorded in each description; the metric per sweep stated there.
- `gs-efficiency`, `gs-dispersion`: against the market (1–5), series ZI-U and ZI-C (Tables 2, 3).
- `gs-shouts`: ZI-C efficiency against shouts a period, series markets 1–5.
- `gs-mechanism`: ZI-C efficiency and price under `book` and `cliff`.
- `cliff-prices`: ZI-C mean price in the four markets under both mechanisms, against P₀ and E(P).
- `zip-days`: ZIP's mean price by day in the four markets.
- `zip-momentum`: the box markets under both momenta.
- `zip-shift`: the shifts.
The CLI names the stop `(its last period)`.

## Survey

A `zi` claims module, each claim with its decision rule written before measuring: Table 2 (ZI-U exactly; ZI-C within 1.5 points), Table 3 (ZI-C within 25 %; ZI-U above ZI-C), Table 1 (a negative slope in every market), footnote 5's rank correlations (ZI-C above ZI-U), the period length (efficiency below 90 % at 100 shouts), the mechanism's effect on GS's claims; Cliff's four E(P) predictions (within 5) under his mechanism and under GS's, his 233⅓ against his formula; ZIP: convergence within 4 days (within 5 of P₀) in the symmetric and flat markets, the approach from below in excess demand, efficiency near 100 %, dispersion a tenth of ZI-C's, recovery after the shifts, the retail market below P₀, and what the momentum discrepancy changes. Claims that fail are reported, and the descriptions, titles and README say so.

## Page

The presets menu gains a **Zero-Intelligence Traders** group and the Compare entries; the Rules panel is generated from the schema in groups Market, Traders, Mechanism, Periods and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `zi` preset; titles for every preset.
- **Core unit:** each market's schedules, P₀, Q₀ and maximum surplus (markets 1–4 against the text's anchors; ZI-U's efficiency against Table 2); shouts in range for each strategy; the book (crossing, the earlier price, clearing, replacing only a better quote); Cliff's mechanism (willing sets, the shout's price, NYSE barring); turns and `sellers_only`; both period ends; units in order and restored each period; ZIP's update (a hand-worked step; a rejected crossing; both momenta's ranges); the shifts; statistics on hand-built trades; the view and Inspect; keyframes; live and reset fields; degenerate configs (one unit, no trade possible, equal limits).
- **Web:** schema groups, charts, the Compare entries, a sweep over a `zi` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, Compare, recording, Experiments.

## Docs

README: a Zero-Intelligence Traders section (the model, how the scans and the code were read, the stated choices, switches, presets, sweeps, and the findings). `docs/papers.md`: the milestone's row (with Cliff as the critique); the Queue's first entry removed; roadmap: Milestone 28 done.
