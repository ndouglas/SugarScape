# Artificial Anasazi (Long House Valley): model extraction

Sources and how they are cited:
- **ODD**: `aa/docs/ODD_LHV.pdf`, "ODD of Netlogo implementation of Artificial Anasazi", Janssen, 1/17/2013. Cited as *ODD p.N, section*.
- **JASSS**: Janssen, M.A. (2009), "Understanding Artificial Anasazi", *JASSS* 12(4) 13, https://jasss.soc.surrey.ac.uk/12/4/13.html. Cited as *JASSS ¶x.y*, *Table n* or *Fig n*.
- **DATA**: my own analysis of the files in `aa/code/*.txt`, with the counts computed by me.
- **LAYOUT**: facts about file layout, units and indexing taken from `lhv.nlogo` (NetLogo 5.0.3 save, "NetLogo 4.0.2" per the papers). Only data formats are described. Where the ODD/paper is ambiguous or silent, a single line says "ODD ambiguous; the replication does X". Nothing below translates the procedures.
- Licensing: the archive is GPL-2.0 (`CITATION.cff`, `LICENSE`). The `.nlogo` Info tab also includes a BSD license (Brookings/NuTech/Metascape) for the original Ascape model. The **data files** carry the archive's GPL-2.0, which is something to think about before bundling them in an MIT repo (see §7, D-1).

---

## 1. Entities and state variables

### 1.1 Household (the only agent)
- One household = **5 persons**. Composition is fixed and not tracked individually (ODD p.1 "State variables"; JASSS ¶2.4, note 1).
- Each household has **one farm cell of its own** and a **residence (settlement) cell** that is a different cell. Several households may live on one cell, but a farm cell belongs to exactly one household (JASSS ¶2.4, ¶2.14; ODD p.2).
- No food sharing between households. Storage belongs to the household (JASSS ¶2.4).

| Variable | Meaning | Unit | Source |
|---|---|---|---|
| age | household age | years (integer) | ODD p.1 |
| lastHarvest `H0` | harvest in the current or last year | kg maize | ODD p.1, p.4 |
| estimate `E[H]` | expected food available next year | kg | ODD p.1, p.5 |
| fertilityAge | age at which fission becomes possible (min fission age) | years | ODD p.1, p.3 |
| fertilityEndsAge | max fission age | years | ODD p.3 Table 2 (JASSS: "End of Fertility Age") |
| deathAge | max household age | years | ODD p.3 Table 2 |
| corn stocks `S0, S-1, S-2` | stock by harvest age: current year, 1 year old, 2 years old | kg | ODD p.4 |
| nutritionNeed | yearly need = 800 (= 160 kg/person × 5) | kg/yr | ODD p.1, p.3 Table 2, p.2 ("160 kg corn per year" per person) |
| farm cell, residence cell | grid coordinates | cell | JASSS ¶2.14 |

- LAYOUT: the stock is an array of 3 slots, with index 0 newest (`S0`), 1 = `S-1` and 2 = `S-2`. `yearsOfStock = 2` in Table 2 means that "2 years" = the two older slots beyond the current year.
- Heterogeneity: Axtell et al. (2002) drew deathAge and fertilityEndsAge per household from uniform ranges (JASSS ¶3.4, Table 4). Janssen's calibration uses homogeneous values and finds "no significant difference" (JASSS ¶4.1). ODD ambiguous; the replication gives every household the same deathAge, fertilityEndsAge, fertilityAge = 16 and pf, because min = max is hard-wired.

### 1.2 Cell (patch)
- 100 m × 100 m (ODD p.1; JASSS ¶2.5). The grid is 80 × 120 cells (JASSS ¶2.5).
- Per-cell state: zone (land-cover class), soil quality `q` (fixed at init), yield `y` (from zone + that year's PDSI class), base yield `BY = y·q·Ha`, water-source flag, farm-occupied flag, and the number of resident households (ODD p.1, p.4; JASSS ¶2.6, ¶2.11).
- LAYOUT: each cell also carries a per-zone, per-year **hydro** value from `environment.txt` (§5.4).

### 1.3 Zone
There are 7 named zones (ODD p.1; JASSS ¶2.5) plus an "Empty" code outside the valley (LAYOUT). The codes are in §5.2.

### 1.4 Water source
- A cell is flagged as a water source for particular periods: "rivers, wells, etc." (ODD p.2 "Input"; JASSS ¶2.9). The flags come from `water.txt` point records and zone-wide stream/alluvium periods (§5.5).

### 1.5 Settlement
- A settlement is a residence cell that may hold several households (ODD p.2 top). The simulated settlement is just a count per cell.
- Historical settlements (`settlements.txt`) are **observational data only**. They are used to build the target household curve (§5.6), not as agents with behavior.

---

## 2. Parameters

### 2.1 Defaults (the "Dean et al. 2000 / original" set)
From ODD p.3 Table 2. JASSS Table 2 (¶2.10) matches except where noted.

| Parameter | Symbol | Default | Notes / source |
|---|---|---|---|
| Simulation period | — | 800 to 1350 AD, annual steps | ODD Table 2; JASSS ¶2.3 |
| Nutritional need per household | NN | 800 kg/yr | ODD Table 2 |
| Persons per household | — | 5 | ODD Table 2 |
| Maximum corn storage length | — | 2 years | ODD Table 2 |
| Harvest adjustment | Ha | 1.00 | ODD Table 2; JASSS ¶2.11 "used for calibration" |
| Annual variance in harvest | σahv | 0.1 | ODD Table 2 (JASSS Table 2 lists a single "Harvest Variance 0.1") |
| Spatial variance in harvest | σshv | 0.1 | ODD Table 2 only |
| Min household age for fission | — | 16 | ODD Table 2 ("Start of Fertility Age" in JASSS) |
| Max household age for fission | — | 30 | ODD Table 2 ("End of Fertility Age") |
| Household death age | — | 30 | ODD Table 2 |
| Annual fission probability | pf | 0.125 | ODD Table 2 |
| Corn stock fraction given to child | fcs | 0.33 | ODD Table 2 |
| Max residence to farm distance | — | 1600 m | ODD Table 2 (the text says "1 mile", ODD p.5) |
| Initial households | — | 14 | ODD p.2 "Initialization"; JASSS ¶2.8 |
| Initial age | — | U[0, 29] | ODD p.2 |
| Initial corn | — | U[2000, 2400] kg | ODD p.2 |

- ODD ambiguous (1 vs 2 variance parameters); the replication uses **one** parameter, `harvestVariance`, as both σshv and σahv. JASSS ¶4.4 and Table 4 also treat "Harvest Variance" as a single parameter.
- "Variance" is used as a **standard deviation** throughout: n(0, σ) (ODD p.2, p.4; JASSS ¶2.12).
- ODD ambiguous (1 mile vs 1600 m); the replication uses 16 cells (= 1600 m at 100 m/cell).

### 2.2 Calibrated / best-fit values (JASSS Table 4, ¶3.4, ¶4.7)
| Parameter | Replication best (L1 and L2 population, L2 carrying capacity) | Carrying-capacity model best (L1) | Axtell 2002, L1 population | Axtell 2002, L2 population |
|---|---|---|---|---|
| Death age | 38 | — | U(30–36) | U(25–38) |
| End of fertility age | 34 | — | U(30–32) | U(30–38) |
| Fission probability | 0.155 | — | 0.125 | 0.125 |
| Harvest adjustment | 0.56 | 0.54 | 0.6 | 0.6 |
| Harvest variance | 0.4 | 0.4 | 0.4 | 0.4 |

- JASSS ¶3.4: Axtell's calibrated set is HV 0.4, Ha 0.6, pf 0.125, death age U(30–36), end of fertility U(30–32). Janssen runs it 100 times (Fig 3) and shows the best of those by L2 (Fig 4).
- LAYOUT: the `.nlogo` slider defaults are Ha **0.54**, harvestVariance 0.4, DeathAge 38, FertilityEndsAge 34, Fertility 0.155. Ha = 0.54 is the carrying-capacity L1 optimum, not the population optimum of 0.56 (see §7, A-15). Slider ranges: Ha 0–1 (step 0.01), HV 0–1 (0.01), DeathAge 26–40, FertilityEndsAge 26–36, Fertility 0–0.2 (0.001).

### 2.3 Sweep ranges used in JASSS §4
- Death age and End of Fertility Age ∈ {26, 28, …, 40}, with EFA ≤ DA, giving 36 combinations (¶4.2).
- pf ∈ 0.095…0.185, step 0.015 (¶4.2).
- Ha ∈ 0.54…0.70, step 0.02; HV ∈ 0…0.7, step 0.1 (¶4.4).
- 18,144 combinations × 15 runs each (¶4.6, ¶4.1).
- Findings: best fits need DA ≥ 34–36, EFA > 30 and pf ≥ 0.125, which are plateaus (¶4.9–4.10). Fit is steeply sensitive to Ha (best 0.54–0.56) and to HV (best 0.4) (¶4.11).

---

## 3. Process overview and schedule

### 3.1 Annual sequence (ODD p.2 "Process overview and scheduling"; JASSS ¶2.7)
1. Calculate the harvest for each household.
2. Remove households that lack enough food (harvest + storage), or whose age is beyond the max age.
3. Calculate the estimated harvest for next year from the corn in stock and this year's harvest.
4. Households that expect not to meet their need next year move to a new farm location:
5. find a farm plot, then
6. find a plot to settle nearby.
7. Fission: a household older than the min fission age (and, per Table 2, not older than the max fission age) creates a new household with probability pf. The child gets a fraction fcs of the parent's corn stock.
8. Update water sources from the input data.
9. Each household ages by one year.

Agents are updated in random order (ODD p.2 "Stochasticity").

- ODD ambiguous on the ordering of aging; the replication increments age **during step 1** (before the death check), not at step 9. The death test is `age > deathAge` after that increment.
- ODD ambiguous on which year's PDSI and water apply. In the replication, the yields for step Y use the PDSI of year Y. The water flags used for relocation in step Y are the ones computed at the end of step Y−1 (year 800 uses year 800's flags, computed at setup). In effect, water lags one year.
- The replication runs the steps for years 800, 801, …, 1350 inclusive (551 steps) and stops after 1350.

### 3.2 Yield (ODD p.4; JASSS ¶2.11, Table 3)
Base yield: **BY = y · q · Ha**, where `y` depends on the cell's zone and on that year's PDSI class for the zone.

PDSI classes (the JASSS ¶2.11 interval notation matches ODD p.4's IF rules exactly):

| PDSI class | North Valley, Mid Valley, Kinbiko Canyon | General Valley | Arable Uplands | Dunes |
|---|---|---|---|---|
| (−∞, −3] | 617 | 514 | 411 | 642 |
| (−3, −1] | 719 | 599 | 479 | 749 |
| (−1, 1) | 821 | 684 | 547 | 855 |
| [1, 3) | 988 | 824 | 659 | 1030 |
| [3, ∞) | 1153 | 961 | 769 | 1201 |

- Units: kg per cell (per household plot) per year. Nonarable Uplands and Empty yield **0** (implicit in ODD/JASSS, which give no column; LAYOUT confirms).
- Note: the class edges are asymmetric. Exactly −1 falls in (−3,−1], exactly 1 in [1,3), exactly −3 in (−∞,−3], and exactly 3 in [3,∞).
- The ODD's Table 1 (p.3) is the conventional 11-class Palmer classification. It is **informational only**; the model uses the 5 classes above.

### 3.3 Soil quality (ODD p.2 "Initialization"; JASSS ¶2.6)
- Fixed at initialization: "adding a number drawn from n(0, σshv)". This is read as q = 1 + n(0, σshv), independently per cell.
- ODD ambiguous (base value, negative q); the replication uses q = 1 + σ·N(0,1), clamped to q ≥ 0, with σ = harvestVariance.

### 3.4 Harvest (ODD p.4; JASSS ¶2.12)
- **H0 = BY · (1 + n(0, σahv))**, drawn independently per household per year, using the BY of the household's farm cell.
- ODD silent on negative harvest (possible when n < −1; about 0.6% of draws at σ = 0.4). The replication does not clamp it. A negative H0 is stored in S0 and ends up *increasing* the unmet need. A spec should probably clamp H0 ≥ 0 and record that as a deviation.

### 3.5 Storage aging and consumption (ODD p.4–5; JASSS ¶2.13)
Every year:
1. Discard the unused `S-2`. Shift the slots: `S-2 ← S-1`, `S-1 ← S0`, `S0 ← H0`.
2. Set NNR = 800. Consume **oldest first**: take from S-2, then S-1, then S0. Each slot is drawn down by min(slot, NNR), and NNR is reduced by the same amount.
3. **If NNR > 0 the household is removed** (starvation) (ODD p.5 top).
- So corn is at most 2 years old when eaten: a harvest from year t can be eaten in t, t+1 and t+2, and is discarded at the start of t+3.

### 3.6 Death / removal (ODD p.2 step 2; p.5)
- Removed if NNR > 0 after consumption, **or** age > death age.
- Removal frees the farm cell and removes the household from its residence.

### 3.7 Expected harvest (ODD p.5; JASSS ¶2.13; "Prediction" ODD p.2)
- **E[H] = S0 + S-1 + H0**, computed after consumption. The leftover S-2 is excluded because it will spoil.
- Note: S0 already holds this year's harvest minus whatever was eaten from it. The formula therefore counts the current harvest twice: once as stock and once as the forecast for next year. That is intended ("stored corn left plus the expected harvest").
- **Move trigger:** E[H] < 800 (nutrition need) (ODD p.2 step 4; JASSS ¶2.14).

### 3.8 Farm plot choice (ODD p.5; JASSS ¶2.14)
- Candidates: **unoccupied** cells that "produce more than the minimum nutrition requirement (800 kg)" **and** are "within 1 mile from a water source".
- If several qualify, choose the one **closest to the current location**.
- If there are no candidates, the household leaves the system (implied by ODD p.5 and by the fission step; the replication does this).
- ODD ambiguous, resolved by the replication as follows:
  - "Produce more than 800" is tested as BY ≥ 800 (≥, not >), using this year's BY with no harvest noise.
  - "Unoccupied" means not farmed **and** no resident households on the cell, and the cell is not Empty.
  - **No** water-distance test is applied to the farm candidate itself. Water enters only in the settlement step.
  - "Current location" is the household's current **farm** cell.
  - Distance is Euclidean in cell units on a **wrapping (torus) 80×120 world**. The torus is a NetLogo world setting; use a non-wrapping plane (see §7, A-9).
  - Ties are broken randomly.
  - The mover's own current farm is released before the search, so it is a candidate again.
  - Movers are processed one at a time, in random order, against the occupancy state as it stands. A plot taken by an earlier mover is unavailable to later ones.

### 3.9 Settlement (residence) choice (ODD p.5; JASSS ¶2.15)
Conditions:
- (i) The site is unfarmed. It may already be inhabited, so multi-household sites are allowed.
- (ii) The site is within 1 mile of the newly selected farm plot.
- (iii) The site is in a **less productive zone** than the new farm plot.

Choice rules:
- If several sites satisfy (i)–(iii), pick the one **closest to water resources**.
- Fallback 1: if none satisfy (i)–(iii), use sites meeting (i) and (ii).
- Fallback 2: if still none, use sites meeting (i) only.
- If still none, the household leaves the system. The ODD notes this "should not happen".

- ODD ambiguous, resolved by the replication as follows:
  - "Less productive zone" is a comparison of the zone-level yield `y` (this year's PDSI-class value, without q or Ha) between the candidate and the farm.
  - "Closest to water" is implemented in two stages. First, take the water-source cell that is unfarmed, has lower `y` than the farm, and is nearest to the farm. If it is within 16 cells of the farm, reside at the unfarmed cell with zone hydro ≤ 0 that is nearest to that water cell (ties random).
  - Fallback 1 takes the unfarmed cell nearest the farm (if within 16 cells), then the nearest unfarmed hydro ≤ 0 cell to it. Water is not used in this fallback.
  - The replication **adds a hydro ≤ 0 condition** that is not in the ODD (§5.4).
  - Because of the hydro filter, North and Mid valley cells can never be residences.

### 3.10 Fission (ODD p.2 step 7, p.3 Table 2; JASSS ¶2.7)
- Eligibility: min fission age < age ≤ max fission age. ODD says "older than" the minimum. Table 2 gives the max; the replication uses `age > 16 and age ≤ fertilityEndsAge`, with age already incremented this year.
- With probability pf per eligible household per year, a new household is created.
- Endowment: the child gets fraction **fcs = 0.33** of the parent's corn stock. The parent keeps 1 − fcs. ODD ambiguous per slot; the replication applies the fraction to each of the three age slots.
- The child starts at **age 0**, "immediately with 3 children that require 160 kg corn per year" each. In practice it is an ordinary 5-person, 800 kg household (ODD p.2 "Interaction").
- The child finds a farm with the farm rule, searching from the **parent's farm**, and a residence with the settlement rule. If no farm is available it does not establish.
- ODD ambiguous; the replication only attempts fission when at least one candidate farm exists.
- Replication artifact, not to copy: the child's actual endowment in the replication is fcs/(1−fcs) × three fresh U[2000,2400] draws, not fcs × the parent's stock. So corn is not conserved. Follow the ODD (see §7, A-12).

### 3.11 Water source update (ODD p.2 step 8; JASSS ¶2.9)
- "Update water sources based on input data." The data and rules are in §5.5.

---

## 4. Initialization (ODD p.2 "Initialization"; JASSS ¶2.8, ¶2.6)
- Year 800 with 14 households. This matches the historical count of 14 in year 800 computed from `settlements.txt` (DATA).
- Age ~ U[0, 29]. ODD ambiguous (int or real, inclusive or not); the replication draws integers 0..28 (uniform integer in [0, 29)).
- Corn ~ U[2000, 2400] kg. ODD ambiguous (total or per slot); the replication fills **each of the three stock slots** with an independent U[2000,2400] draw, so the total is about 6.6 t. Two of those slots survive the first shift.
- Soil quality per cell is drawn once (§3.3).
- Positions: ODD silent. The replication gives each initial household a uniform-random reference cell, then applies the farm rule (nearest candidate farm to that cell) and the settlement rule.
- ODD silent; the replication applies the initial farm eligibility with BY = y·q, **without Ha** (see §7, A-13). That artifact is not worth copying.
- Randomness: cell quality, initial ages, corn and positions, the yearly harvest noise, fission draws, update order and distance tie-breaks (ODD p.2 "Stochasticity").

---

## 5. Landscape and data files
All files are whitespace-separated numbers with no line structure. Read them as one token stream.

### 5.1 Grid
- 80 columns (x, west to east) × 120 rows (y, south to north), with 100 m cells (JASSS ¶2.5; ODD p.1).
- LAYOUT: x ∈ 0..79 and y ∈ 0..119, with y = 119 the **northern** edge. The orientation was checked against JASSS Fig 1: North Valley is at the top right, the General Valley runs south, and Kinbiko Canyon is at the upper left.

### 5.2 `Map.txt`: zones (9,600 tokens = 80 × 120)
- LAYOUT, column-major order: token k gives x = k div 120 and y = 119 − (k mod 120). Each block of 120 tokens is one column, read from north (y = 119) down to south (y = 0), and the columns run west to east.

| Code | Zone (ODD/JASSS name) | Yield column (Table 3) | Cells (DATA) |
|---|---|---|---|
| 0 | General Valley Floor | General Valley | 637 |
| 10 | North Valley Floor | North/Mid/Kinbiko | 328 |
| 15 | North Valley Dunes | Dunes | 35 |
| 20 | Midvalley Floor | North/Mid/Kinbiko | 51 |
| 25 | Mid Valley Dunes | Dunes | 15 |
| 30 | Uplands Nonarable ("Natural") | none, yield 0 | 3396 |
| 40 | Arable Uplands | Arable Uplands | 145 |
| 50 | Kinbiko Canyon | North/Mid/Kinbiko | 27 |
| 60 | Empty (outside the valley) | none, yield 0 | 4966 |

- DATA: 328 + 51 + 27 = **406**, 637 and 35 + 15 = **50**. These match JASSS ¶4.3 exactly ("406 cells in the North and Mid Valley and the Kinbiko Canyon … 637 cells in the General Valley … 50 cells in the Dunes").
- LAYOUT: the replication contains a dormant rule that would give Mid cells with x > 74 the General yield column. No Mid cell has x > 74 (DATA: Mid spans x 27–38), so all Mid cells use the North/Mid/Kinbiko column.
- Farmable in practice: any cell with BY ≥ 800 (§3.8), i.e. North, Mid, Kinbiko, General and Dunes. Arable Uplands at most 769·q·Ha, so only exceptional-q cells qualify. Nonarable Uplands and Empty are never farmable.

### 5.3 `adjustedPDSI.txt`: adjusted PDSI (5,200 tokens = 4 series × 1,300 years)
- ODD p.3 says "for each year and each category of landcover". JASSS ¶2.9 calls it "the main input".
- LAYOUT: 4 consecutive blocks of 1,300 values each, covering years **200 to 1499**. The value for year Y in block b is token `b·1300 + (Y − 200)`.

| Block | Series | Used by zones |
|---|---|---|
| 0 | General | General Valley |
| 1 | North | North Valley **and Kinbiko Canyon** |
| 2 | Mid | Midvalley |
| 3 | Natural/Uplands | Nonarable Uplands (yield 0 regardless); intended for Arable Uplands |

- There is **no series for the Dunes.** ODD ambiguous; the replication leaves the Dunes' PDSI at 0, so the Dunes always get the (−1,1) class = 855 kg. JASSS ¶4.3 states "50 cells in the Dunes … receive 855 kg", which confirms this for the published results.
- Replication bug, decide explicitly (see §7, A-6): the Arable Uplands lookup uses a mismatched zone name. Uplands PDSI therefore also stays 0, so Uplands always yield 547·q·Ha. The evident intent is block 3.
- DATA, the character of the series:
  - Blocks 1 and 2 (North, Mid) contain **only the values 0.0, 2.0 and 4.0**, i.e. the classes (−1,1), [1,3) and [3,∞). They are categorical and hydrology-driven.
  - Blocks 0 and 3 are real-valued (about −5.9 to 8.0 over 800–1350), but blocks of years are replaced by the constants 0.010101, 2.0 and 4.0 ("adjusted").
  - Block 0 is heavily 4.0 around 1000–1260.
  - In 800–1350, the class counts for General are [3,∞) 253, [1,3) 105, (−1,1) 85, (−3,−1] 77, ≤−3 31.
- Carrying-capacity sanity check (DATA + JASSS ¶4.3): with Ha = 1, around 1260 blocks 0–2 are all 4.0, giving 406×1153, 637×961 and 50×855 kg. That is about 1,093 cells ≥ 800 before the q spread, which matches "around 1050" (¶4.3) and JASSS Fig 2's plateau of about 1,050.

### 5.4 `environment.txt`: per-year zone environment (16,770 tokens = 1,118 records × 15)
- ODD p.3 says it holds "for each cell which zone it relates to and when the cell is a water source". **The file does not match that description** (see §7, A-4).
- LAYOUT: record r is year **382 + r** (years 382–1499). Each record is 5 groups of 3 numbers. In group order the zones are General, North, Mid, Natural (also used for Uplands), Kinbiko. In each group:
  - field 0 is a PDSI value. DATA: it matches `adjustedPDSI` block 0 except in the adjusted years, so it is the unadjusted PDSI, and it is almost identical across the 5 groups.
  - field 1 is the **hydrologic value** ("hydro").
  - field 2 is undocumented and unused. DATA: range 0–10.
- ODD silent; the replication uses only field 1, as a residence filter: a cell may be a residence only if its zone's hydro ≤ 0.
  - Zones without a hydro series (Arable Uplands, the Dunes and Empty) count as hydro = 0, i.e. always allowed.
  - DATA over 800–1350: North hydro is 8–10 and Mid 6–10, so never allowed. General and Kinbiko are ≤ 0 only in **877–919**. Natural is always 0.
  - In practice, residences end up on Nonarable Uplands, Arable Uplands and Dunes cells (and Empty cells, if they are nearest), or on General/Kinbiko cells in 877–919.

### 5.5 `water.txt`: water points (648 tokens = 108 records × 6)
- ODD p.3: "locations of water points and period in which they contain water".
- LAYOUT record: `id, metersNorth, metersEast, type, startYear, endYear`. **North comes before East.**
- LAYOUT conversion to cells:
  - x = 24.5 + trunc((E − 2392)/93.5), where trunc rounds toward 0. The NetLogo patch is then round-half-up, so in effect x_cell = 25 + trunc((E − 2392)/93.5).
  - y_cell = 45 + trunc(37.6 + (N − 7954)/93.5).
  - Implied cell pitch: **93.5 m**, not 100 m (see §7, A-8).
- Types (DATA counts): 1 = 56, 2 = 38, 3 = 12, 4 = 2. ODD silent on type semantics. The replication treats:
  - **type 2** as a permanent water source;
  - **type 3** as a source when startYear ≤ year ≤ endYear (inclusive);
  - **types 1 and 4 as never water.** Type 4 is 2 records at (0,0), which map off-grid to y = −2.
  - Examples of type 3 intervals: 1150–1300, 1200–1270, 1050–1200, 1100–1300, 950–1100.
- ODD silent; the replication also flags these, which come from **no data file**:
  - (a) **Alluvium periods** 420–559, 630–679, 980–1119 and 1180–1229: every General, North, Mid and Kinbiko cell is a water source.
  - (b) **Stream periods** 280–359, 800–929 and 1300–1449: every Kinbiko cell is a water source.
  - (c) 8 fixed cells that are always water: (72,114), (70,113), (69,112), (68,111), (67,110), (66,109), (65,108), (65,107). These form a stream line into the North Valley; the first is in Nonarable Uplands and the rest are North Valley cells.
  - Periods are half-open [start, end).
- DATA: nearly all type 1–3 points land on Nonarable Uplands (code 30) cells bordering the valley floors. That makes sense for springs, and it confirms the conversion and orientation.

### 5.6 `settlements.txt`: archaeological sites (5,856 tokens = 488 records × 12)
- ODD p.3: "estimates for each excavated settlement the time period of occupation and population numbers".
- LAYOUT record, all values integer:

| # | Field | Notes |
|---|---|---|
| 0 | SARG site number | |
| 1 | meters north | same coordinate system as `water.txt` |
| 2 | meters east | |
| 3 | start date | AD |
| 4 | end date | AD |
| 5 | median date | **years BP**: median AD = 1950 − value |
| 6 | type | 1 = habitation, the only type counted. DATA: 1 = 235, 2 = 244, −1 = 9 |
| 7 | size | DATA: −1 = unknown |
| 8 | description code | 1–8 |
| 9 | room count | |
| 10 | elevation | m |
| 11 | baseline households | households at the median date |

- LAYOUT position: x = trunc(24.5 + (E − 2392)/93.5) and y = trunc(45 + 37.6 + (N − 7954)/93.5). This is used for display only, and is slightly different from the water formula (see §7, A-8).
- **Historical households in year Y**, as a LAYOUT/data derivation. For each type-1 site with start ≤ Y < end, with m = median AD and b = baseline:
  - if Y > m: n = ⌈b·(end − Y)/(end − m)⌉;
  - if Y ≤ m and m ≠ start: n = ⌈b·(Y − start)/(m − start)⌉;
  - then n = max(n, 1);
  - if Y ≤ m = start: n = 0, the degenerate case. DATA: 1 site has m = start and none has m = end.
  - Sites outside [start, end) count 0. Total = Σ n.
  - This is the "linearly extrapolated" room-count estimate of JASSS ¶3.1: a triangle rising from start to the median, then falling to the end, rounded up, and at least 1 while occupied.
- DATA, the resulting target curve (file `hist.txt` beside this document, years 800–1350):
  - 14 in 800 (dips to 7 around 840); 28 in 850; about 60 in 900–990; 87 in 1000.
  - A jump to 134 in 1050, then a **first peak of 167 in 1108**.
  - Drops to 116 in 1150, rises to a **max of 216 in 1269**; 159 in 1280, 95 in 1290.
  - **0 from 1300 on** (last nonzero year 1299).
  - This is consistent with JASSS ¶2.3 ("increased to about 250 households", no households after 1300), ¶4.3 ("just above 200" in 1260), and Figs 2 and 4 (red "data" line).

---

## 6. Outputs, observables and fit
- Main output: the number of households each year, compared with the historical households (ODD p.5–6, Fig 2a–c; JASSS ¶3.2–3.5).
- **Carrying capacity** each year: the number of cells with BY ≥ nutrition need, i.e. potential farm plots (JASSS ¶3.3, Fig 2 caption). It depends only on Ha, HV (via q) and the PDSI/zone data, not on agent behavior (¶4.1, ¶4.5).
  - ODD silent; the replication counts it with the previous step's BY (a one-year lag) and without the water/occupancy filters.
- Fit metrics (JASSS ¶4.6), summed over years:
  - L1 = Σ|hist − sim|;
  - **L2 = Σ(hist − sim)²** (sum of squares; no square root);
  - the same two metrics for carrying capacity vs hist.
  - "Best run" = the smallest sum of squared differences (¶3.5).
  - Calibration uses the **mean over 15 runs** per parameter set (¶4.1, Table 4 caption).
- ODD ambiguous (year range, sample point); the replication pairs years 800..1350 inclusive (551 points). The simulated count is taken after fission in the step for year Y, and hist(Y) uses the same Y.
- Other plots in the replication: histograms of household age, last harvest and estimate (informational).
- Target shape to reproduce:
  - Defaults (Ha 1, HV 0.1): the simulation climbs to about 1,050 households by about 1030, tracks carrying capacity, dips around 1130–1170, and stays high until about 1270 before falling to about 400–500 by 1350. That is far above the data (JASSS Fig 2, ¶4.3).
  - Calibrated (Axtell set, JASSS Fig 4): carrying capacity is flat at about 250 in roughly 1035–1130 and 1175–1265, drops to about 125 around 1130–1170, and fluctuates 25–180 before 1000 and after 1270.
    - The best simulated run rises to about 250 around 1125 and again about 250 around 1250.
    - It then collapses to about 50–100 by 1300 but **does not reach 0**, and persists to 1350.
  - JASSS ¶4.13: calibration overshoots the first data peak (1030–1130) and undershoots the second (1180–1260). "In all simulations the population and carrying capacity remain positive after the archaeological data suggest the abandonment."
  - Conclusion (¶5.1): the population follows carrying capacity, smoothed by storage and reproduction.

---

## 7. Ambiguities, discrepancies and data issues

### A. Model-rule ambiguities (the ODD/JASSS text vs the replication)
- **A-1 Variance.** Harvest "variance" parameters are standard deviations. The ODD lists separate spatial and annual values; JASSS and the replication use one parameter (§2.1).
- **A-2 Aging position.** The schedule puts aging at step 9, but the replication ages at step 1, so the death check sees the incremented age (§3.1). With deathAge 38 and `age > deathAge`, a household's last year is age 38.
- **A-3 Farm water criterion.** The ODD and JASSS ¶2.14 require farm candidates within 1 mile of water; the replication does not check this (§3.8). The spec must choose. The ODD text is the documented rule, but the published calibration was produced without it.
- **A-4 `environment.txt`.** The ODD describes per-cell zone and water data. The actual file is per-year × 5-zone (PDSI, hydro, unknown) records for 382–1499 (§5.4). The hydro ≤ 0 residence filter appears in neither the ODD nor JASSS.
- **A-5 Water rules not in data.** The alluvium/stream periods, the 8 fixed stream cells and the type semantics (1 and 4 ignored) come from the replication, not from any input file or the ODD (§5.5). "Water source" and "within 1 mile of water" are otherwise undefined.
- **A-6 PDSI series coverage.** There are 4 series for 7 zones. Kinbiko reuses North. The Dunes have no series (constant 855, confirmed by JASSS ¶4.3). Arable Uplands are intended to use series 3 but get a constant class (−1,1) through the replication's name mismatch. Choose one and document it: series 3 for Uplands is the evident intent, and the numerical effect is small because at most a handful of cells reach 800.
- **A-7 "Less productive zone"** is compared by the current PDSI-class yield `y`, not by a static zone rank (§3.9). With a static rank (e.g. Dunes > North/Mid/Kinbiko > General > Arable > Nonarable), the result differs only in the years when classes differ between zones.
- **A-8 Cell size.** The text says 100 m cells and a 1600 m radius (16 cells). The coordinate conversion in the data uses 93.5 m per cell. The water and settlement formulas also differ slightly: water truncates the scaled offset and then adds 24.5; settlements truncate the whole sum.
- **A-9 Distance metric and topology.** Not specified. The replication's world wraps (torus), so distances near the edges wrap. The valley is interior (the edges are Empty or Nonarable), but "nearest unfarmed cell" searches can still wrap. A non-wrapping Euclidean metric is recommended as a documented deviation.
- **A-10 Tie-breaks.** "Closest" ties are random in the replication. The ODD is silent.
- **A-11 Initial stock.** Per slot vs total (§4). The replication fills 3 slots, which gives ~2× the corn the ODD implies over the first two years.
- **A-12 Fission endowment.** The ODD says fcs of the parent's stock. The replication's child receives a fraction of fresh random stock instead, so corn is not conserved. The replication also requires ≥ 1 free farm before attempting fission (§3.10).
- **A-13 Initial farm eligibility** ignores Ha in the replication. The carrying-capacity count lags one year (§4, §6).
- **A-14 Negative harvest** is possible and not clamped (§3.4). Quality is clamped at 0.
- **A-15 Best-fit Ha.** Table 4 gives the population optimum Ha = 0.56, while the `.nlogo` slider default is 0.54, the carrying-capacity L1 optimum. JASSS ¶4.11 says the "best fit is derived for values 0.54 and 0.56".
- **A-16 Fission age bounds.** The ODD Table 2 "Max household age for fission 30" matches JASSS's "End of Fertility Age". The ODD text says only "older than the minimum". Axtell's paper says fission happens when "a daughter reaches the age of 15" (JASSS note 1), while the model uses 16 and strict >.
- **A-17 Monthly vs annual.** Axtell 2002 describes monthly consumption (13.33 kg/person/month). The ODD and the replication are annual (JASSS note 1). 160 kg × 5 = 800 kg is equivalent over a year.
- **A-18 Move vs stay when there is no farm.** A mover with no candidate farm leaves the valley even if it still has stock (ODD p.5 implied; replication).
- **A-19 Occupancy bookkeeping (replication artifact).** The replication never releases a vacated residence cell when a household moves, and double-counts a new one. Once a cell has been settled, it is effectively excluded from farm candidacy for good (only a death decrements the count). The spec should implement correct counts and expect a slightly higher carrying capacity than the replication.
- **A-20 Ascape ambiguity.** JASSS ¶2.1 says it was "not clear which version of the original code was used", and that the replication was verified by eyeballing it against Ascape. The Axtell 2002 figures are therefore not an exact reproduction target. The ODD's Fig 2c and JASSS Fig 4 are the realistic targets.
- **A-21 File name.** The ODD calls it `settlement.txt`; the actual file is `settlements.txt`. The ODD also says "800 and 1400" in its Purpose section, while the period is 800–1350 (Table 2).

### D. Data-level observations
- **D-1 Licensing.** The data files ship in a GPL-2.0 archive. Check whether bundling them in an MIT repo is acceptable (e.g. keep them as a separately licensed data directory or download them at build time) before committing.
- **D-2** All numeric tokens are integers except in the PDSI and environment files. Parse everything as f64.
- **D-3** `adjustedPDSI` blocks 1–2 are purely categorical (0/2/4). Blocks 0/3 are partly overwritten with constants (0.010101, 2, 4) that encode hydrologic adjustment (§5.3).
- **D-4** Two water records (ids 61 and 98, type 4) have coordinates (0,0) and map off-grid. They are harmless if type 4 is ignored.
- **D-5** Eight type-1 settlement sites fall on Empty cells and one maps off-grid (DATA). This does not matter, because positions are display-only.
- **D-6** The environment and PDSI files start in 382 and 200 respectively, and run to 1499. The simulation needs only 800–1350.
- **D-7** The historical curve derivation depends on the median-BP conversion and the ⌈⌉/min-1 rounding. Verify an implementation against `hist.txt`: 14 @800, 167 @1108, 216 @1269, 0 @1300.

### Blocking gaps
None are strictly blocking. The ODD alone, however, is **insufficient to reproduce the published curves**. It does not define:
- the water-source rules (A-5);
- the hydro residence filter (A-4);
- the PDSI zone→series mapping and the Dunes/Uplands handling (A-6);
- the grid↔meter conversion (A-8).

Each of these is stated above as the replication's behavior. The spec must adopt them as documented facts (or deliberate deviations) and validate against JASSS Fig 2 (defaults) and Fig 4 (calibrated), comparing shapes and levels.
