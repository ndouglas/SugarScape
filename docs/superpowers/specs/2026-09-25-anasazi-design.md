# SugarScape Milestone 10 — Artificial Anasazi (Long House Valley) — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–9 specs in `docs/superpowers/specs/` (especially milestone 9's multi-model architecture); all remain binding where not changed here.
**Sources:** Janssen, M.A. (2009), "Understanding Artificial Anasazi", *JASSS* 12(4) 13, https://jasss.soc.surrey.ac.uk/12/4/13.html; Janssen, "ODD of Netlogo implementation of Artificial Anasazi" (2013) and the data files of *Artificial Anasazi* v1.1.0, CoMSES Computational Model Library, doi:10.25937/krp4-g724 (GPL-2.0). The book's Chapter VI names the project ("Computational Archaeology").
**Extraction:** `docs/superpowers/specs/2026-09-25-anasazi-extraction.md` — every rule, parameter, data layout and ambiguity, cited to ODD page / JASSS paragraph / data analysis. It is binding where this spec refers to it; section numbers below (§n, A-n, D-n) are the extraction's.

## Goal

Add the Artificial Anasazi model of the Long House Valley (AD 800–1350) as a fourth model kind, implemented from its published description, with the published replication's departures available as named switches, so the playground can reproduce the published results and show what those departures change.

## Non-negotiable constraints

- **Licensing:** the five data files are bundled unmodified under `data/anasazi/` with their GPL-2.0 license text, the package's `CITATION.cff`, and a `NOTICE` stating they are GPL-2.0, from doi:10.25937/krp4-g724, and separate from the MIT-licensed code. The model code is written from the ODD and the JASSS paper, not translated from `lhv.nlogo` (which was read only for data layouts, as the extraction records). The historical curve derived from `settlements.txt` is data and lives under `data/anasazi/` too.
- **Existing models unchanged:** sugarscape, Schelling and Ring World golden entries and legacy fixtures stay green and unedited.
- **One engine path:** the anasazi model plugs into the milestone 9 `Model` trait, `ModelWorld`, WASM `Sim`, host, engine, schema panel, charts table, Compare, sweeps and CLI — no forks.
- **Honesty:** presets, README and descriptions say this reproduces Janssen's 2009 replication (itself an approximation of Axtell et al. 2002, A-20), which behaviors come from the replication rather than the written description, and how close each preset gets to the historical record as measured.

## Data

- `data/anasazi/Map.txt`, `adjustedPDSI.txt`, `environment.txt`, `water.txt`, `settlements.txt` (unmodified), `LICENSE` (GPL-2.0), `CITATION.cff`, `NOTICE`, and `historical.txt` (the per-year historical household count derived per §5.6, checked: 14 in 800, 167 in 1108, 216 at the 1269 maximum, 0 from 1300).
- The core parses them (§5) into a typed valley: zones per cell (column-major, north→south within a column, columns west→east, §5.2), four PDSI series indexed by year (§5.3), per-zone yearly environment records (hydro used, §5.4), water points (§5.5), and the historical curve. Native builds embed the files (`include_str!`); the WASM build embeds them the same way (≈ 440 KB).
- Parser tests: token counts (§5.2–5.6), zone cell counts (406 / 637 / 50 as in the extraction), and the derived historical curve equal to `historical.txt`.

## Model kind `anasazi`

- **Config:** households' food need (800), fertility age start (16) and end, death age, fission probability, child endowment fraction (0.33), storage years (2), harvest adjustment, spatial harvest s.d., annual harvest s.d., start year (800), end year (1350), initial households (14), initial corn range, and `quirks: { … }` (below). A tick is a year; `tick 0` = AD 800.
- **Documented rules** (ODD / JASSS, §3–§4), one year per tick, in the ODD's order: harvest (base yield = zone yield for the zone's PDSI class × cell soil quality × harvest adjustment, then × (1 + N(0, annual s.d.)), §3.2–3.4); storage aging and consumption (§3.5); removal when need can't be met or age exceeds death age (§3.6); expected harvest (§3.7); relocation for those expecting shortfall — farm plot: nearest unfarmed, unsettled cell yielding ≥ need and within 1 mile of water (§3.8); settlement: the fallback chain of §3.9; no farm → the household leaves; fission (§3.10: between fertility start and end, probability pf, child gets the endowment fraction of the parent's stock, which the parent loses); water update (§3.11); aging last (§3.1).
- **Adopted from the replication in both modes** (the documented description does not define them; each recorded in code comments with its extraction reference): water-source rules — which types count, alluvium/stream periods, the 8 fixed stream cells (A-5); the hydro ≤ 0 residence filter (A-4); PDSI series per zone, Kinbiko sharing North, Dunes constant 855 kg (A-6); 93.5 m per cell and the replication's water/settlement coordinate conversions (A-8); random tie-breaks among equally close candidates (A-10); negative harvests not clamped (A-14); the "less productive" comparison by current yield (A-7); non-wrapping Euclidean distance in cells unless the wrap quirk is on.
- **Replication quirks** (`quirks`, each a boolean, named for what it does; all true in the "published" presets, all false in "documented"):
  - `age_before_death_check` (A-2), `no_farm_water_check` (A-3), `uplands_single_class` (A-6), `wrap_edges` (A-9), `initial_corn_per_slot` (A-11), `fission_fresh_endowment` (A-12, child's corn drawn fresh, not conserved), `fission_needs_free_farm` (A-12), `initial_eligibility_ignores_adjustment` (A-13), `occupancy_leak` (A-19), `single_harvest_variance` (A-1: one s.d. for spatial and annual).
- **Statistics:** `households`, `historical` (this year's archaeological estimate), `fit` (running sum of squared differences between households and historical over the years so far, §6), `capacity` (cells yielding ≥ need this year, §6), `mean_corn`, `births`, `moves`, `departures`, `year`.
- **Determinism:** own seeded RNG with the milestone 9 portability rules (u32 sampling); fingerprint over households (id, cell, settlement, age, corn slots), soil quality, tick.
- **Presets:** `lhv-published` (quirks on; calibrated: death age 38, fertility end 34, fission 0.155, harvest adjustment 0.56, harvest s.d. 0.4 — JASSS Table 4; target JASSS Fig 4), `lhv-published-defaults` (quirks on; the replication's defaults, §2.1; target JASSS Fig 2), `lhv-documented` (quirks off; calibrated values). Where the extraction notes a conflict (A-15: 0.56 vs 0.54), the preset uses JASSS Table 4 and says so.

## Page

- **View:** the valley on its 80 × 120 grid. Color modes: Zones (map colors), Yield (this year's potential harvest), Occupation (farms and settlements). Overlays: active water sources (incl. the fixed stream cells), settlements as dots sized by households, farm–settlement links. The readout shows the calendar year ("AD 1142"); the run stops at the end year (the world reports finished; the page pauses with a notice).
- **Inspect:** a cell — zone, PDSI class, potential yield, soil quality, farm/settlement occupancy, water within 1 mile; a household — age, corn stock, farm, settlement, expected harvest.
- **Rules panel:** from the schema, groups Households (reset), Harvest (live), Replication quirks (reset; each with a one-line explanation and its extraction reference).
- **Charts:** Households vs historical (the historical estimate as a reference line on the same chart), Carrying capacity, Fit (sum of squared differences), Mean stored corn, Births / moves / departures; the usual Compare overlay. The presets menu gains "Replication vs documented — Anasazi (Compare)" (A = `lhv-published`, B = `lhv-documented`, same seed).
- **Other machinery** as for every model: share links and replay (harvest changes logged), recording, Max, Experiments, CLI.

## Experiments

- **`lhv-calibration`:** base `lhv-published`, x = harvest adjustment 0.50–0.62 (step 0.02), metric = final `fit`, 15 seeds (as JASSS averages 15 runs); expected minimum near 0.54–0.56; measured values recorded in its description.
- **`lhv-quirks`:** base `lhv-published`, series = each quirk turned off alone (and none), metric = final `fit`, 15 seeds; measured and recorded.

## Testing

- **Golden/legacy:** unchanged; new golden entries for the three presets.
- **Core unit:** parsers (above); yield table lookup incl. class boundaries; harvest, storage aging and consumption; removal; expected harvest; farm choice (need, water radius, unfarmed/unsettled, nearest, ties); settlement fallback chain; fission and endowment conservation (documented) vs fresh (quirk); water update by year; each quirk's effect in isolation; wrap vs non-wrap distance.
- **Book-style (`#[ignore]`, release, 15 seeds):** `lhv-published` follows JASSS Fig 4's shape and level (rise to a peak in the 1100s–1200s, decline after ~1270; mean fit below a measured bound); `lhv-published-defaults` likewise vs Fig 2; `lhv-documented` measured and recorded (no pass/fail on its fit beyond sanity). Thresholds measured and recorded.
- **Web:** schema panel for the quirks; the historical reference series; Inspect fields; year readout; end-year stop.
- **Browser (controller):** each preset to AD 1350 with the historical line; Compare published vs documented; the calibration sweep; recording; every existing scenario; Max performance.

## Docs

README: an Anasazi section (sources, license notes, what is documented vs from the replication, the quirks, measured results). Roadmap: mark Artificial Anasazi done; keep Nowak–May spatial games and civil violence as candidates.
