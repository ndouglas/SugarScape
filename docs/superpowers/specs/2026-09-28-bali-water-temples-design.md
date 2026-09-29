# SugarScape Milestone 29 — Balinese Water Temples — Design

**Date:** 2026-09-28
**Builds on:** the milestone 1–28 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–28, the preset titles of `crates/sugarscape-core/src/titles.rs`, milestone 10's handling of the Anasazi valley data (GPL-2.0 data in the MIT build, a settled decision), and milestones 27–28's method (figures read from the PDF; one fit rule fixed before measuring; where a source's code and text differ, the difference a named switch).
**Source texts** (local copies):
- J. Stephen Lansing and James N. Kremer, "Emergent Properties of Balinese Water Temple Networks: Coadaptation on a Rugged Fitness Landscape," *American Anthropologist* 95(1): 97–114 (1993), `papers/bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (LK below; a scan, read by OCR). Its ecological model is only cited (Kremer's technical report in *Priests and Programmers*, 1991), not specified.
- The follow-up: Marco A. Janssen, "Coordination in irrigation systems: An analysis of the Lansing–Kremer model of Bali," *Agricultural Systems* 93: 170–190 (2007), `papers/bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf` (J below; a 42-page preprint; page numbers are the preprint's).
- Janssen's NetLogo replication, CoMSES Net model 2221 v1.2.0 (GPL-2.0), and its ODD, `papers/bali/` (to be saved there; read, not copied): the watershed data and the rainfall, crop and plan tables are taken as data, as the Anasazi valley's were; the rules are written from LK, J and the ODD. Marshall Abrams's fork (github.com/mars0i/bali, no license) is read for its bug notes only.

## Goal

Lansing and Kremer's water-temple model as one model kind, `bali` ("Balinese Water Temples"), a full citizen of the playground: 172 subaks and 12 dams on the Oos and Petanu rivers, monthly rain, water sharing, rice growth and pests; each year, subaks imitating their best neighbor (LK Figs. 7–11, Table 1); Janssen's analyses — coordination levels found by search (J Figs. 1–4), the two-node model (Figs. 5–8), generalized imitation with innovation (Figs. 9–11), adaptive subaks (Figs. 12–14) and perturbed networks (Figs. 15–16); and every unstated rule and every place Janssen's code departs from his text a named switch, with every claim measured — above all LK's central one, that temple-like networks self-organize.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited.
- **Faithful where the sources are specific**; where silent, the choice is stated here and in the module docs. Where Janssen's code departs from his text, the text is the default and the code a switch (his code is a replication whose departures are bugs, not results).
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only; `+ − × ÷` for state).
- **Truthful descriptions and titles.**

## Source summary

- **LK's watershed:** "approximately 6,136 hectares of irrigated rice terraces … 12 subsections … For each of the 172 subaks … the name, the area, the basin in which it resides, the weir from which it receives irrigation water, and the weir to which any excess is returned … the program simulates the rainfall, river flow, irrigation demand, rice growth stage, and pest levels … the harvest is adjusted for cumulative water stress and pest damage." Water sharing and pest control are "opposing constraints".
- **LK's coadaptation:** "each subak checks to see whether any of its neighbors got higher yields. If so, the target subak copies the cropping pattern of its (best) neighbor … compared its yield with those of its four closest neighbors … After eight years, average yields peaked, and all but 20 subaks stopped changing … The remaining 20 subaks keep swapping … The resemblance between the last run … and the temple system … is evident." "after 8–35 years, a complex structure of coordinated cropping patterns emerged, which bore a remarkable similarity to the actual pattern of water temple coordination"; "the same phenomenon occurs every time, regardless of the initial distribution of cropping patterns, or ecological parameters such as flow rates or pest biology." Fig. 7: the traditional pattern (del or mansur, fallow, cicih) with random start dates, 4.9 → 8.57 t/ha. Table 1: traditional 4.9 → 8.57; high-yielding 15.91 → 18.08; low rain and high pests 13.67 → 17.66. Fig. 11: two crops of high-yielding rice and a vegetable crop, 16 → 18.5 t/ha/yr in 20 years with 20 subaks still changing; in year 21 pest growth, dispersal and damage raised and rain cut to 80 %: 15.3, recovering to 15.8 within 7 years; "when the conditions of low rain and high pests occurred from the very beginning … it took twice as long." Fig. 6: of seven scales of coordination, "the highest peak is achieved by the scale of coordination that most closely approximates the temple scale."
- **Janssen's reimplementation:** 172 subaks, 12 dams; rainfall by elevation zone in low, middle and high scenarios; runoff and demand per dam; "When subaks ask for more water than there is supply, all subaks receive the same reduction"; 21 twelve-month plans (49 less the vegetable ones) with a start month; harvest = potential yield × accumulated water stress × pest damage; pests grow at g (2–2.4) with rice, 0.1 fallow, and spread by LK's "shortcut" equation at d (0.18–0.45); imitation of the best pest-neighbor. His findings: finer coordination (1 → 172 groups) raises harvest and inequality (Fig. 1: ≈ 17.5, 17.6, 18.7, 18.8, 19.2, 22.8 t/ha/yr); "The benefit of synchronization is only derived for the medium growth rate of pests" (Fig. 3: flat at g 2 and 2.4); the two-node model's threshold near g = ∛10 ≈ 2.14; generalized imitation (eq. 4: imitate j if Hᵢ < Hⱼ / (1 + min{γp χp², γw χw²}), the best such j; innovation with probability ρ below the mean) best for γp < 0.5; adaptive subaks (plant when expected water exceeds m_w and neighborhood pests per hectare are below m_p; best m_w 0.05, m_p 0.02); imitators lose harvest when pest links are removed, adaptive subaks when links are added; "There is a strong overlap between the 14 Masceti temples and subaks connected via pest relationships in the empirical dataset."
- **Janssen's code** (read for its departures): each month only one randomly chosen dam computes its water balance, with no upstream inflow ever added (the others keep a stale water stress, starting at 0); the subak–dam file's columns read as (return, source) though the first is the upstream dam in 93 of 95 cases; pests reset to 0.01 every year ("If we don't [reset] the pest level, the system gets locked into low harvest rates"); the harvest he plots is per harvest event, not per year (≈ 7.5, "somewhat lower than the original publication").

## Measured in planning

A throwaway prototype (Python) of the rules below, on Janssen's watershed data; 5 seeds, 30 years unless stated. The survey reproduces each with the implementation.

- **Imitation raises yields, as LK say:** random plans 9.1, 14.4, 16.5, 17.9, 18.5 … 20.9 t/ha/yr (year 30); subaks changing 98, 53, 34, 22, 15 … 2. The traditional pattern alone: 3.4 → 7.5 (LK: 4.9 → 8.57).
- **The annual pest reset decides it:** without it, yields collapse after the first year (9.1, 3.9, 4.4 … 3.9) and imitation stops — as Janssen warns.
- **Janssen's code's water routing** barely changes the endpoint (20.6 against 20.9); the swapped dam columns cost 0.6; the diffusion form of the pest equation changes little.
- **The temple resemblance is largely the network's.** The pest-link graph has 46 connected components (27 isolated subaks); those components alone match the 14 masceti groups with an adjusted Rand index of 0.33; imitation's final patches match them at 0.33–0.48, and mostly end as one strategy per component. Independent runs agree with each other at 0.62–0.81.
- **Imitation's gain is not a knife-edge:** it roughly doubles yields at every pest growth rate from 2.0 to 2.4 (11.0 → 21.5 at 2.0; 7.4 → 19.7 at 2.4). Janssen's knife-edge concerns the value of coordination *levels*, measured in implementation by the search.

## Architecture

Model kind `bali` ("Balinese Water Temples"): `ModelKind::Bali`, `ModelConfig::Bali(BaliConfig)` tagged `"model": "bali"`, a `BaliWorld` implementing `Model`. Code in `crates/sugarscape-core/src/bali/` (`config.rs`, `data.rs` — the watershed and the tables, parsed from `crates/sugarscape-core/data/bali/` —, `water.rs`, `world.rs`, `search.rs` — the plan search —, `two_node.rs`, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`). One tick is one month.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `watershed` | `bali` | reset | `bali` (the Oos and Petanu, 172 subaks) or `two_node` (J §4) |
| `plans` | `random` | reset | the starting plans: `random` (any of the 21, any month), `traditional` (LK's kerta masa: 6-month and 4-month traditional rice), `hyv` (two high-yielding crops a year: LK's high-yielding runs, without their vegetable crop, which the 21 plans drop), `temples` (one random plan per masceti), or `search` (Janssen's hill-climbing at `level`) |
| `level` | 14 | reset | groups sharing a plan under `search` or `decision: fixed`: 1 (the watershed), 2 (the two river systems), 7 (adjacent masceti pairs — a stated choice), 14 (the mascetis), 28 (masceti × the data's second temple column), 172 (every subak) |
| `decision` | `imitate` | live | `imitate` (LK: the best out-neighbor, if strictly better, all at once), `generalized` (J eq. 4), `adaptive` (J §5), or `fixed` |
| `growth`, `dispersal` | 2.2, 0.3 | live | g and d |
| `rain` | `middle` | live | `low`, `middle`, `high`, or `random` (25/50/25 % a year) |
| `rain_scale` | 1.0 | live | rain × this |
| `perturb` | off | live | from year `at`: `growth`, `dispersal`, pest `damage` (× sensitivity) and `rain` (× rain) — LK's Fig. 11 |
| `routing` | `network` | live | `network` (flow passes down the dam network: the texts) or `janssen_code` (one random dam a month, no inflow) |
| `dam_columns` | `code` | reset | the subak–dam file read as Janssen's code reads it, or `physical` (swapped) |
| `pest_form` | `shortcut` | live | LK's shortcut, or `diffusion` (the standard form Janssen expected) |
| `pest_reset` | true | live | pests back to 0.01 each year (Janssen's code) |
| `gamma_p`, `gamma_w`, `innovation` | 0.4, 0.4, 0.04 | live | eq. 4's γp, γw and ρ |
| `m_w`, `m_p` | 0.05, 0.02 | live | the adaptive thresholds: water expected per hectare (m/day at the source dam, a stated choice) and pests per hectare in the neighborhood |
| `remove_links`, `add_links` | 0, 0 | reset | J's pₑ and pₙ: each pest link removed, and each pair sharing a source or return dam linked, with these probabilities |
| `node_rain`, `node_periods` | 2.0, 12 | reset | `two_node`: rain units a month; 2 or 12 periods a year |
| `score_from` | 6 | live | the years scored (Janssen: the last five of ten) |
| `stop_at` | 30 | live | `finished()` after this many years (0: never) |

## Step (one month)

Rain (by dam zone and scenario) → runoff per dam → each subak's demand (its crop's use less the return dam's rain), drawn from its source dam → each dam's flow (base flow + runoff + upstream outflow − demand), a shortage cutting its subaks' water by the same fraction → each growing crop advances by its water fraction over its growing time → pests grow and spread → a crop ending is harvested (stage × potential yield × max(0, 1 − pests × sensitivity)). At the year's end: the decision rule; pests reset (if on); the year's statistics.

## Statistics

`SERIES`: `harvest` (mean t/ha/yr, area-weighted, last year), `spread` (standard deviation across subaks: Janssen's inequality), `scored` (the mean over the scored years), `changing` (subaks that changed plans), `water_stress`, `pest_loss`, `patches` (connected groups sharing a plan and start month), `strategies` (distinct plans and starts in use), `temple_match` (the adjusted Rand index of the patches against the mascetis), `network_match` (the same for the pest network's components — the baseline), `year`.

## Views

- **The map:** subaks at their data coordinates, discs sized by area; dams and their river links; pest links on request.
- **Below:** the year's months, each dam's water stress.
- **Color modes:** **Plan**, **Temple**, **Harvest**, **Pests**, **Water**, **Crop**.
- **Inspect:** a subak (area, temple, dams, plan and start, crop, harvest, pests, stress, neighbors) or a dam (flow, demand, shortage).
- **Charts:** Harvest (`harvest`, `scored`, `spread`); Changing plans; Water and pests; Patches (`patches`, `strategies`, `temple_match`, `network_match`). Time axis: Months.
- **Compare:** "Imitating neighbors vs fixed random plans — Balinese Water Temples (Compare)".
- **Experiments default:** `scored` against `growth` 2.0–2.4.

## Presets

Drafts, to be measured: `lk-traditional`, `lk-hyv`, `lk-perturbed` (from year 21), `lk-stressed` (from the start), `lk-temples` (plans fixed by temple), `janssen-code`, `janssen-levels-14`, `janssen-two-node`, `janssen-generalized`, `janssen-adaptive`, `janssen-fewer-links`.

## Experiments and CLI

Sweeps (seeds and horizons in each description): `bali-levels` (the six levels, J Fig. 1), `bali-growth` (levels × g 2.0, 2.2, 2.4, Fig. 3), `bali-rain`, `bali-dispersal`, `bali-two-node` (g × d, both period counts, Figs. 5–8), `bali-gamma` (γp × γw, Figs. 9–10), `bali-adaptive` (m_w × m_p, Figs. 12–13), `bali-links` (pₑ, pₙ for imitators and adaptive subaks, Figs. 15–16), `bali-imitation-growth`. The CLI names the stop `(its last year)`.

## Survey

A `bali` claims module, each claim with its decision rule written before measuring: LK — emergence within 8–35 years, Table 1's three rises, about 20 subaks still changing, the patches' resemblance to the temples *beyond the pest network's own* (the patches' match above the network components'), recovery after the perturbation, twice as long when stressed from the start, the temple scale best among the levels, "every time, regardless" of initial plans and ecology; Janssen — decentralized plans beat the temple scale, inequality rising, the benefit of coordination only at g = 2.2, the two-node threshold near 2.14, γp < 0.5 best, imitators and adaptive subaks diverging under perturbed networks, his code's harvest lower; ours — the effects of the routing, the dam columns, the pest reset and the pest equation's form.

## Page

The presets menu gains a **Balinese Water Temples** group and the Compare entry; the Rules panel is generated from the schema in groups Watershed, Plans, Decisions, Pests, Water, Network and Stopping.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `bali` preset.
- **Core unit:** the data (172 subaks, 12 dams, 323 links, 5861 ha, the masceti sizes); runoff, demand and a shortage by hand on a small dam network; both routings; both column readings; a crop's growth and harvest by hand; the pest equation (both forms) on two linked subaks; the reset; imitation (strictly better, synchronous, out-links); eq. 4 on hand-built distances; innovation; the adaptive rule; link removal and addition; the levels' groups (sizes 1, 2, 7, 14, 28, 172); the search improving a grouped plan; the two-node model against J's statements (no pest explosion below g ≈ 2.14 with fallow; water units); statistics (patches, the adjusted Rand index); the view and Inspect; keyframes; degenerate configs.
- **Web and browser:** as for the earlier models.

## Docs

README: a Balinese Water Temples section (the model, how the sources and Janssen's code were read, switches, presets, sweeps, findings). `docs/papers.md`: the milestone's row; the Queue's first entry removed; roadmap: Milestone 29 done.

## Amendments (implementation planning)

- **Files:** the water, growth, pest and harvest step is `engine.rs` (with the network and the yearly scoring the search uses), not `water.rs`; the data live in `data/bali/` beside `data/anasazi/`, not under the crate, with Janssen's `LICENSE` and `CITATION.cff` and our `NOTICE`; `tables.txt` transcribes his NetLogo constants (the 21 plans, the rain tables, the crops' constants, the dam network).
- **Presets:** `lk-random` (imitation from random plans, the defaults) heads the list, and `lk-random-fixed` (the same plans, never changed) is Compare's B. Twelve drafted presets become thirteen.
- **Page:** a fifth chart, **Temple match** (`temple_match`, `network_match`), apart from **Patches** (`patches`, `strategies`), whose scale is a count; the perturbation's switch and year sit in the Pests group; the Experiments default is `scored` against `growth` 2.0–2.4 step 0.1 over 360 months.
- **Sweeps:** `bali-links` compares eq.-4 imitators (γ 0.4, as Janssen's §6 uses), neighbor imitators and adaptive subaks; `bali-adaptive` adds m_p 0.5 (Janssen: large m_p plants too early).
- **Two nodes:** six crops yield 5.52, not 6 — each crop loses the pests grown from the floor in its three months (0.01 × 2³); the threshold is read from the largest fall of the best harvest.
- **Measured in implementation** (the survey, 22 claims, rules as written in `survey/src/claims/bali.rs`): 9 hold (emergence, Table 1, "every time", inequality rising with finer levels, the two-node threshold, γp < 0.5, the pest network's overlap with the temples, the reset, water hardly binding), 4 are weak (the temple scale best but by 0.1 %, not 1 %; the routing, the columns and the pest form within 5 % but not shown equivalent at 10 seeds), 9 fail (20 subaks still changing; patches beyond the network's own; recovery after the perturbation; twice as long from the start; Janssen's Fig. 1 rise; his benefit at g 2.2 alone; his dispersal loss; his best adaptive pair; his imitators' link sensitivity). The levels rule's 1 % margin and the claims' 5 % equivalence margins were written after the planning measurements showed the levels within 0.3 % of each other; the survey reports the Weak verdicts rather than moving them.
- **The final review's amendment:** the column reading decides how much water binds (before rain, the base flow of 5 of the 12 dams falls short of full planting under the code's reading and 6 under the physical one; only the watershed's sum, 907 200 against 879 150 m³/day, meets it), so every claim about the searched levels is judged under both readings — Holds under both, Weak under one, Fails under neither, a rule written after the review's measurements — and a new claim (`bali.ours.columns-levels`) records the dependence; the water claim is split by reading. The search now scores every option on the same draws (Janssen's routing is random) and climbs from the better of random plans and the best single plan (Janssen used several starts), so a finer level never ends below one group: the levels rise monotonically, level 172 is best, and the seeds agree. Level 2 is the two rivers (Janssen's highlands and lowlands are not in the data) and level 28 has 22 groups; both are disclosed. Two nodes take live edits of growth and dispersal (re-planning the pair). The temple-match claim's 0.05 margin was set after planning had measured 0.37 against 0.33; the adaptive water check is now within Janssen's plotted range (0–0.05). The survey: 23 claims; 9 hold, 3 are weak, 11 fail.
