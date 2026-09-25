//! The Long House Valley's data (milestone 10): zones, adjusted PDSI,
//! per-zone hydrology, water points and the archaeological household
//! estimate, parsed from the five files of *Artificial Anasazi* v1.1.0
//! (Janssen, CoMSES doi:10.25937/krp4-g724, GPL-2.0) bundled unmodified in
//! `data/anasazi/` (see `data/anasazi/NOTICE`). Layouts and rules are those
//! of docs/superpowers/specs/2026-09-25-anasazi-extraction.md (§5, A-4 to
//! A-8); this module only reads the files.
//!
//! Cells are numbered in frame order: `i = y * WIDTH + x` with `x` west to
//! east and `y` the row counted from the **north** edge (the data's
//! northward coordinate is `HEIGHT - 1 - y`).

use std::sync::OnceLock;

use serde::Serialize;

/// The grid: 80 columns west to east, 120 rows (JASSS ¶2.5).
pub const WIDTH: u32 = 80;
pub const HEIGHT: u32 = 120;
pub const CELLS: usize = (WIDTH * HEIGHT) as usize;
/// The years every data file covers (`environment.txt` starts in 382,
/// `adjustedPDSI.txt` in 200; both end in 1499; D-6).
pub const FIRST_YEAR: u32 = 382;
pub const LAST_YEAR: u32 = 1499;
const PDSI_FIRST_YEAR: u32 = 200;
const PDSI_YEARS: usize = 1300;
const ENV_RECORD: usize = 15;
const WATER_RECORD: usize = 6;
const SITE_RECORD: usize = 12;

const MAP: &str = include_str!("../../../../data/anasazi/Map.txt");
const PDSI: &str = include_str!("../../../../data/anasazi/adjustedPDSI.txt");
const ENVIRONMENT: &str = include_str!("../../../../data/anasazi/environment.txt");
const WATER: &str = include_str!("../../../../data/anasazi/water.txt");
const SETTLEMENTS: &str = include_str!("../../../../data/anasazi/settlements.txt");

/// A cell's land-cover zone (§5.2; ODD p.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    General,
    North,
    NorthDunes,
    Mid,
    MidDunes,
    /// Uplands Nonarable ("Natural").
    Natural,
    /// Arable Uplands.
    Uplands,
    Kinbiko,
    /// Outside the valley.
    Empty,
}

/// A zone's column of JASSS Table 3 (§3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YieldColumn {
    /// North Valley, Mid Valley and Kinbiko Canyon.
    NorthMidKinbiko,
    General,
    Uplands,
    Dunes,
}

/// The four adjusted-PDSI series, in file block order (§5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdsiSeries {
    General = 0,
    North = 1,
    Mid = 2,
    Natural = 3,
}

impl Zone {
    pub const ALL: [Zone; 9] = [
        Zone::General,
        Zone::North,
        Zone::NorthDunes,
        Zone::Mid,
        Zone::MidDunes,
        Zone::Natural,
        Zone::Uplands,
        Zone::Kinbiko,
        Zone::Empty,
    ];

    /// The zone of a `Map.txt` code.
    pub fn from_code(code: u32) -> Option<Zone> {
        Some(match code {
            0 => Zone::General,
            10 => Zone::North,
            15 => Zone::NorthDunes,
            20 => Zone::Mid,
            25 => Zone::MidDunes,
            30 => Zone::Natural,
            40 => Zone::Uplands,
            50 => Zone::Kinbiko,
            60 => Zone::Empty,
            _ => return None,
        })
    }

    /// The ODD's name for the zone.
    pub fn name(self) -> &'static str {
        match self {
            Zone::General => "General Valley Floor",
            Zone::North => "North Valley Floor",
            Zone::NorthDunes => "North Valley Dunes",
            Zone::Mid => "Midvalley Floor",
            Zone::MidDunes => "Mid Valley Dunes",
            Zone::Natural => "Uplands Nonarable",
            Zone::Uplands => "Arable Uplands",
            Zone::Kinbiko => "Kinbiko Canyon",
            Zone::Empty => "Outside the valley",
        }
    }

    /// Its yield column; `None` for the zones that yield nothing
    /// (Nonarable Uplands and outside the valley).
    pub fn column(self) -> Option<YieldColumn> {
        Some(match self {
            Zone::North | Zone::Mid | Zone::Kinbiko => YieldColumn::NorthMidKinbiko,
            Zone::General => YieldColumn::General,
            Zone::Uplands => YieldColumn::Uplands,
            Zone::NorthDunes | Zone::MidDunes => YieldColumn::Dunes,
            Zone::Natural | Zone::Empty => return None,
        })
    }

    /// The PDSI series its yield follows (A-6, adopted from the replication):
    /// Kinbiko shares North's; the Dunes have none (their PDSI is 0, so they
    /// always yield the (−1, 1) class: JASSS ¶4.3's 855 kg); Arable Uplands
    /// follow the Natural series, or none when `uplands_single_class` (the
    /// replication's name mismatch) is on. `None` means a PDSI of 0.
    pub fn pdsi_series(self, uplands_single_class: bool) -> Option<PdsiSeries> {
        match self {
            Zone::General => Some(PdsiSeries::General),
            Zone::North | Zone::Kinbiko => Some(PdsiSeries::North),
            Zone::Mid => Some(PdsiSeries::Mid),
            Zone::Natural => Some(PdsiSeries::Natural),
            Zone::Uplands if !uplands_single_class => Some(PdsiSeries::Natural),
            _ => None,
        }
    }

    /// Its group in `environment.txt` (§5.4: General, North, Mid, Natural —
    /// also used for Arable Uplands — and Kinbiko); `None` (hydro 0) for the
    /// Dunes and outside the valley.
    fn hydro_group(self) -> Option<usize> {
        match self {
            Zone::General => Some(0),
            Zone::North => Some(1),
            Zone::Mid => Some(2),
            Zone::Natural | Zone::Uplands => Some(3),
            Zone::Kinbiko => Some(4),
            _ => None,
        }
    }
}

/// The PDSI class of JASSS ¶2.11 / ODD p.4, 0 (driest) to 4: (−∞, −3],
/// (−3, −1], (−1, 1), [1, 3), [3, ∞).
pub fn pdsi_class(pdsi: f64) -> usize {
    if pdsi <= -3.0 {
        0
    } else if pdsi <= -1.0 {
        1
    } else if pdsi < 1.0 {
        2
    } else if pdsi < 3.0 {
        3
    } else {
        4
    }
}

/// Yield in kg per cell per year by column and PDSI class (JASSS Table 3).
pub fn column_yield(column: YieldColumn, class: usize) -> f64 {
    const NORTH_MID_KINBIKO: [f64; 5] = [617.0, 719.0, 821.0, 988.0, 1153.0];
    const GENERAL: [f64; 5] = [514.0, 599.0, 684.0, 824.0, 961.0];
    const UPLANDS: [f64; 5] = [411.0, 479.0, 547.0, 659.0, 769.0];
    const DUNES: [f64; 5] = [642.0, 749.0, 855.0, 1030.0, 1201.0];
    let row = match column {
        YieldColumn::NorthMidKinbiko => &NORTH_MID_KINBIKO,
        YieldColumn::General => &GENERAL,
        YieldColumn::Uplands => &UPLANDS,
        YieldColumn::Dunes => &DUNES,
    };
    row[class]
}

/// A water point of `water.txt` (§5.5), on the grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterPoint {
    pub x: u32,
    pub y: u32,
    /// 1–4; only 2 (permanent) and 3 (in `start..=end`) are ever water (A-5).
    pub kind: u32,
    pub start: i64,
    pub end: i64,
}

/// Water periods not in any data file (A-5, adopted from the replication),
/// as half-open year ranges: every General, North, Mid and Kinbiko cell is
/// water in the alluvium periods, every Kinbiko cell in the stream periods.
const ALLUVIUM: [(u32, u32); 4] = [(420, 560), (630, 680), (980, 1120), (1180, 1230)];
const STREAM: [(u32, u32); 3] = [(280, 360), (800, 930), (1300, 1450)];
/// Eight cells that are always water (A-5), in the data's (x, northward y):
/// a stream line into the North Valley.
const STREAM_CELLS: [(u32, u32); 8] = [
    (72, 114),
    (70, 113),
    (69, 112),
    (68, 111),
    (67, 110),
    (66, 109),
    (65, 108),
    (65, 107),
];

/// The valley's data, parsed once (`valley()`).
#[derive(Debug)]
pub struct Valley {
    zones: Vec<Zone>,
    /// The four series, each indexed by `year - 200`.
    pdsi: [Vec<f64>; 4],
    /// Per year from 382, the five groups' hydro values (field 1 of each).
    hydro: Vec<[f64; 5]>,
    /// Every on-grid water point, of every type.
    water: Vec<WaterPoint>,
    /// Historical households per year from `FIRST_YEAR` to `LAST_YEAR`.
    historical: Vec<u32>,
}

fn tokens(name: &str, text: &str, record: usize, expected: usize) -> Result<Vec<f64>, String> {
    let values = text
        .split_whitespace()
        .map(|t| t.parse::<f64>().map_err(|e| format!("{name}: {t:?}: {e}")))
        .collect::<Result<Vec<f64>, String>>()?;
    if values.len() != expected || values.len() % record != 0 {
        return Err(format!(
            "{name}: {} numbers, expected {expected}",
            values.len()
        ));
    }
    Ok(values)
}

/// The cell of a point `n` metres north and `e` metres east in the data's
/// coordinates (§5.5's water conversion, A-8: 93.5 m per cell), or `None`
/// off the grid.
fn water_cell(n: f64, e: f64) -> Option<(u32, u32)> {
    let x = 25.0 + ((e - 2392.0) / 93.5).trunc();
    let north = 45.0 + (37.6 + (n - 7954.0) / 93.5).trunc();
    let on = |v: f64, n: u32| v >= 0.0 && v < f64::from(n);
    (on(x, WIDTH) && on(north, HEIGHT)).then(|| (x as u32, HEIGHT - 1 - north as u32))
}

/// `ceil(p / q)` for `p ≥ 0`, `q > 0`.
fn ceil_div(p: i64, q: i64) -> i64 {
    (p + q - 1) / q
}

/// One site's households in `year` (§5.6): a triangle rising from `start`
/// to the median and falling to `end`, rounded up and at least 1 while
/// occupied; a site whose median is its start counts 0 up to the median.
fn site_households(start: i64, end: i64, median: i64, baseline: i64, year: i64) -> i64 {
    if year < start || year >= end {
        return 0;
    }
    if year > median {
        return ceil_div(baseline * (end - year), end - median).max(1);
    }
    if median == start {
        return 0;
    }
    ceil_div(baseline * (year - start), median - start).max(1)
}

impl Valley {
    /// Parses the five files (their text), checking each one's size.
    pub fn parse(
        map: &str,
        pdsi: &str,
        environment: &str,
        water: &str,
        settlements: &str,
    ) -> Result<Valley, String> {
        let codes = tokens("Map.txt", map, 1, CELLS)?;
        let mut zones = vec![Zone::Empty; CELLS];
        for (k, &code) in codes.iter().enumerate() {
            let zone = Zone::from_code(code as u32)
                .filter(|_| code.fract() == 0.0)
                .ok_or_else(|| format!("Map.txt: unknown zone code {code}"))?;
            // Column-major, each column north to south, columns west to east.
            let (x, y) = (k / HEIGHT as usize, k % HEIGHT as usize);
            zones[y * WIDTH as usize + x] = zone;
        }
        let series = tokens("adjustedPDSI.txt", pdsi, PDSI_YEARS, 4 * PDSI_YEARS)?;
        let pdsi = [0, 1, 2, 3].map(|b| series[b * PDSI_YEARS..(b + 1) * PDSI_YEARS].to_vec());
        let years = (LAST_YEAR - FIRST_YEAR + 1) as usize;
        let env = tokens(
            "environment.txt",
            environment,
            ENV_RECORD,
            years * ENV_RECORD,
        )?;
        let hydro = env
            .as_chunks::<ENV_RECORD>()
            .0
            .iter()
            .map(|r| [r[1], r[4], r[7], r[10], r[13]])
            .collect();
        let points = tokens("water.txt", water, WATER_RECORD, 108 * WATER_RECORD)?;
        let water = points
            .as_chunks::<WATER_RECORD>()
            .0
            .iter()
            .filter_map(|r| {
                // id, metres north, metres east, type, start, end
                water_cell(r[1], r[2]).map(|(x, y)| WaterPoint {
                    x,
                    y,
                    kind: r[3] as u32,
                    start: r[4] as i64,
                    end: r[5] as i64,
                })
            })
            .collect();
        let sites = tokens(
            "settlements.txt",
            settlements,
            SITE_RECORD,
            488 * SITE_RECORD,
        )?;
        let historical = (FIRST_YEAR..=LAST_YEAR)
            .map(|year| {
                sites
                    .as_chunks::<SITE_RECORD>()
                    .0
                    .iter()
                    // Field 6 is the type: only habitations (1) count.
                    .filter(|r| r[6] == 1.0)
                    .map(|r| {
                        // start, end, median (years BP) and baseline households.
                        let (start, end) = (r[3] as i64, r[4] as i64);
                        let median = 1950 - r[5] as i64;
                        site_households(start, end, median, r[11] as i64, i64::from(year))
                    })
                    .sum::<i64>() as u32
            })
            .collect();
        Ok(Valley {
            zones,
            pdsi,
            hydro,
            water,
            historical,
        })
    }

    pub fn zone(&self, cell: usize) -> Zone {
        self.zones[cell]
    }

    /// Series `s`'s adjusted PDSI in `year` (200–1499).
    pub fn pdsi(&self, s: PdsiSeries, year: u32) -> f64 {
        self.pdsi[s as usize][(year - PDSI_FIRST_YEAR) as usize]
    }

    /// `zone`'s PDSI in `year` (0 for a zone without a series; A-6).
    pub fn zone_pdsi(&self, zone: Zone, year: u32, uplands_single_class: bool) -> f64 {
        zone.pdsi_series(uplands_single_class)
            .map_or(0.0, |s| self.pdsi(s, year))
    }

    /// `zone`'s hydrologic value in `year` (§5.4; 0 for the Dunes and
    /// outside the valley).
    pub fn hydro(&self, zone: Zone, year: u32) -> f64 {
        zone.hydro_group()
            .map_or(0.0, |g| self.hydro[(year - FIRST_YEAR) as usize][g])
    }

    /// Every on-grid point of `water.txt`.
    pub fn water_points(&self) -> &[WaterPoint] {
        &self.water
    }

    /// Which cells are water sources in `year` (A-5): type-2 points always,
    /// type-3 points from their start to their end year inclusive, every
    /// General, North, Mid and Kinbiko cell in an alluvium period, every
    /// Kinbiko cell in a stream period, and the eight stream cells.
    pub fn water_sources(&self, year: u32) -> Vec<bool> {
        let within = |periods: &[(u32, u32)]| periods.iter().any(|&(a, b)| (a..b).contains(&year));
        let (alluvium, stream) = (within(&ALLUVIUM), within(&STREAM));
        let mut out: Vec<bool> = self
            .zones
            .iter()
            .map(|z| match z {
                Zone::General | Zone::North | Zone::Mid => alluvium,
                Zone::Kinbiko => alluvium || stream,
                _ => false,
            })
            .collect();
        let y = i64::from(year);
        for p in &self.water {
            if p.kind == 2 || (p.kind == 3 && (p.start..=p.end).contains(&y)) {
                out[(p.y * WIDTH + p.x) as usize] = true;
            }
        }
        for (x, north) in STREAM_CELLS {
            out[((HEIGHT - 1 - north) * WIDTH + x) as usize] = true;
        }
        out
    }

    /// The archaeological estimate of households in `year` (§5.6), 0
    /// outside `FIRST_YEAR..=LAST_YEAR`.
    pub fn historical(&self, year: u32) -> u32 {
        year.checked_sub(FIRST_YEAR)
            .and_then(|k| self.historical.get(k as usize))
            .copied()
            .unwrap_or(0)
    }
}

/// The bundled valley, parsed on first use.
pub fn valley() -> &'static Valley {
    static VALLEY: OnceLock<Valley> = OnceLock::new();
    VALLEY.get_or_init(|| {
        Valley::parse(MAP, PDSI, ENVIRONMENT, WATER, SETTLEMENTS).expect("the bundled data parse")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HISTORICAL: &str = include_str!("../../../../data/anasazi/historical.txt");

    fn count(zone: Zone) -> usize {
        (0..CELLS).filter(|&i| valley().zone(i) == zone).count()
    }

    #[test]
    fn zones_match_the_extraction_and_jasss() {
        let v = valley();
        let counts: Vec<usize> = Zone::ALL.iter().map(|&z| count(z)).collect();
        assert_eq!(counts, [637, 328, 35, 51, 15, 3396, 145, 27, 4966]);
        // JASSS ¶4.3: 406 cells in the North and Mid Valley and Kinbiko, 637
        // in the General Valley, 50 in the Dunes.
        assert_eq!(
            count(Zone::North) + count(Zone::Mid) + count(Zone::Kinbiko),
            406
        );
        assert_eq!(count(Zone::NorthDunes) + count(Zone::MidDunes), 50);
        // Orientation (JASSS Fig 1): North Valley in the upper right, Kinbiko
        // in the upper left, the General Valley running south.
        let mean = |zone: Zone| {
            let cells: Vec<usize> = (0..CELLS).filter(|&i| v.zone(i) == zone).collect();
            let n = cells.len() as f64;
            let x = cells.iter().map(|&i| (i % 80) as f64).sum::<f64>() / n;
            let y = cells.iter().map(|&i| (i / 80) as f64).sum::<f64>() / n;
            (x, y)
        };
        let (nx, ny) = mean(Zone::North);
        let (kx, ky) = mean(Zone::Kinbiko);
        let (_, gy) = mean(Zone::General);
        assert!(
            nx > 40.0 && ny < 60.0,
            "North Valley upper right: {nx} {ny}"
        );
        assert!(kx < 40.0 && ky < 60.0, "Kinbiko upper left: {kx} {ky}");
        assert!(gy > ny, "the General Valley lies south of the North Valley");
    }

    #[test]
    fn map_codes_are_read_column_major_from_the_north() {
        // Token k is column k / 120, row k % 120 from the north; the cell it
        // fills is row * 80 + column. Markers at 0 and CELLS - 1 land on
        // themselves under either a column-major or a row-major reading (the
        // grid's corners are fixed points of the transpose), so they alone
        // don't distinguish the two. Token 81 (column 0, row 81) does not:
        // it belongs at cell 81 * 80 = 6480, not at cell 81, so an
        // axis-swapped (row-major) reading — which would instead drop each
        // token k straight into cell k — puts its zone on the wrong cell and
        // fails the assertions below.
        let mut codes = vec![60u32; CELLS];
        codes[0] = 10;
        codes[CELLS - 1] = 0;
        codes[81] = 20;
        let map = codes
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        let v = Valley::parse(&map, PDSI, ENVIRONMENT, WATER, SETTLEMENTS).unwrap();
        assert_eq!(v.zone(0), Zone::North, "x 0, the northern row");
        assert_eq!(v.zone(CELLS - 1), Zone::General, "x 79, the southern row");
        assert_eq!(v.zone(1), Zone::Empty, "token 1 is (0, 1), not (1, 0)");
        assert_eq!(
            v.zone(6480),
            Zone::Mid,
            "token 81 is column 0, row 81, so it fills cell 81 * 80 = 6480"
        );
        assert_eq!(
            v.zone(81),
            Zone::Empty,
            "cell 81 is filled by token 121 (column 1, row 1), not token 81"
        );
    }

    #[test]
    fn files_of_the_wrong_size_or_content_are_refused() {
        let e = Valley::parse("0 0", PDSI, ENVIRONMENT, WATER, SETTLEMENTS).unwrap_err();
        assert!(e.starts_with("Map.txt: 2 numbers"), "{e}");
        let bad = MAP.replacen("60", "61", 1);
        let e = Valley::parse(&bad, PDSI, ENVIRONMENT, WATER, SETTLEMENTS).unwrap_err();
        assert!(e.contains("unknown zone code 61"), "{e}");
        let e = Valley::parse(MAP, "x", ENVIRONMENT, WATER, SETTLEMENTS).unwrap_err();
        assert!(e.starts_with("adjustedPDSI.txt"), "{e}");
    }

    #[test]
    fn pdsi_classes_follow_the_interval_notation() {
        let classes: Vec<usize> = [
            -5.0, -3.0, -2.9, -1.0, -0.99, 0.0, 0.99, 1.0, 2.99, 3.0, 8.0,
        ]
        .iter()
        .map(|&p| pdsi_class(p))
        .collect();
        assert_eq!(classes, [0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 4]);
        assert_eq!(column_yield(YieldColumn::NorthMidKinbiko, 4), 1153.0);
        assert_eq!(column_yield(YieldColumn::General, 4), 961.0);
        assert_eq!(column_yield(YieldColumn::Dunes, 2), 855.0);
        assert_eq!(column_yield(YieldColumn::Uplands, 2), 547.0);
        assert_eq!(column_yield(YieldColumn::Uplands, 0), 411.0);
    }

    #[test]
    fn pdsi_series_are_read_by_year_and_the_dunes_have_none() {
        let v = valley();
        // Blocks 1 and 2 are categorical (§5.3, D-3).
        for year in 800..=1350 {
            for s in [PdsiSeries::North, PdsiSeries::Mid] {
                assert!([0.0, 2.0, 4.0].contains(&v.pdsi(s, year)), "{s:?} {year}");
            }
        }
        // General's classes over 800–1350 (§5.3's counts).
        let mut classes = [0; 5];
        for year in 800..=1350 {
            classes[pdsi_class(v.pdsi(PdsiSeries::General, year))] += 1;
        }
        assert_eq!(classes, [31, 77, 85, 105, 253]);
        assert_eq!(v.zone_pdsi(Zone::NorthDunes, 1000, false), 0.0);
        assert_eq!(
            v.zone_pdsi(Zone::Kinbiko, 1000, false),
            v.pdsi(PdsiSeries::North, 1000)
        );
        assert_eq!(v.zone_pdsi(Zone::Uplands, 1000, true), 0.0);
        assert_eq!(
            v.zone_pdsi(Zone::Uplands, 1000, false),
            v.pdsi(PdsiSeries::Natural, 1000)
        );
    }

    #[test]
    fn around_1260_about_1050_cells_can_feed_a_household_at_full_harvest() {
        // §5.3's sanity check (JASSS ¶4.3): with Ha = 1 and every q = 1.
        let v = valley();
        let fed = (0..CELLS)
            .filter(|&i| {
                let zone = v.zone(i);
                zone.column().is_some_and(|c| {
                    column_yield(c, pdsi_class(v.zone_pdsi(zone, 1260, true))) >= 800.0
                })
            })
            .count();
        assert_eq!(fed, 406 + 637 + 50);
    }

    #[test]
    fn hydro_only_allows_residences_where_the_extraction_says() {
        let v = valley();
        for year in 800..=1350 {
            assert!(v.hydro(Zone::North, year) >= 8.0, "North {year}");
            assert!(v.hydro(Zone::Mid, year) >= 6.0, "Mid {year}");
            assert_eq!(v.hydro(Zone::Natural, year), 0.0, "Natural {year}");
            assert_eq!(v.hydro(Zone::Uplands, year), 0.0, "Uplands {year}");
            assert_eq!(v.hydro(Zone::NorthDunes, year), 0.0);
            let dry = v.hydro(Zone::General, year) <= 0.0;
            assert_eq!(dry, (877..=919).contains(&year), "General {year}");
            assert_eq!(v.hydro(Zone::Kinbiko, year) <= 0.0, dry, "Kinbiko {year}");
        }
    }

    #[test]
    fn water_points_convert_onto_the_uplands_by_the_valley_floor() {
        let v = valley();
        let kinds = |k: u32| v.water_points().iter().filter(|p| p.kind == k).count();
        // Types 1 = 56, 2 = 38, 3 = 12; the two type-4 points map off the grid.
        assert_eq!((kinds(1), kinds(2), kinds(3), kinds(4)), (56, 38, 12, 0));
        let on_uplands = v
            .water_points()
            .iter()
            .filter(|p| v.zone((p.y * WIDTH + p.x) as usize) == Zone::Natural)
            .count();
        // "Nearly all" (§5.5): 95 of the 106 on-grid points.
        assert_eq!(on_uplands, 95);
    }

    #[test]
    fn water_sources_follow_the_adopted_periods() {
        let v = valley();
        let at = |year: u32| v.water_sources(year);
        let floor =
            |w: &[bool], zone: Zone| (0..CELLS).filter(|&i| v.zone(i) == zone && w[i]).count();
        // 1000 is an alluvium year: the whole valley floor is water.
        let w = at(1000);
        assert_eq!(floor(&w, Zone::General), 637);
        assert_eq!(floor(&w, Zone::North), 328);
        // 1150 is not: only points, Kinbiko outside its stream periods is dry.
        let w = at(1150);
        assert!(floor(&w, Zone::General) < 10);
        assert_eq!(floor(&w, Zone::Kinbiko), 0);
        // 850 is a stream year for Kinbiko; the periods are half-open.
        assert_eq!(floor(&at(850), Zone::Kinbiko), 27);
        assert_eq!(floor(&at(930), Zone::Kinbiko), 0);
        assert_eq!(floor(&at(1119), Zone::General), 637);
        assert!(floor(&at(1120), Zone::General) < 637);
        // The eight stream cells are always water: (72, 114) is row 5.
        for year in [800, 1150, 1350] {
            assert!(at(year)[5 * 80 + 72], "{year}");
        }
        // Type-3 points hold water only within their years (inclusive).
        let p = v.water_points().iter().find(|p| p.kind == 3).unwrap();
        let cell = (p.y * WIDTH + p.x) as usize;
        assert!(at(p.start as u32)[cell] && at(p.end as u32)[cell]);
    }

    #[test]
    fn the_historical_curve_is_the_bundled_one() {
        let v = valley();
        let mut years = 0;
        for line in HISTORICAL.lines() {
            let mut parts = line.split_whitespace().map(|t| t.parse::<u32>().unwrap());
            let (year, households) = (parts.next().unwrap(), parts.next().unwrap());
            assert_eq!(v.historical(year), households, "year {year}");
            years += 1;
        }
        assert_eq!(years, 551, "800 to 1350");
        // The extraction's checks (§5.6, D-7).
        let h = |y: u32| v.historical(y);
        assert_eq!(
            (h(800), h(840), h(850), h(1000), h(1050)),
            (14, 7, 28, 87, 134)
        );
        assert_eq!(
            (h(1108), h(1150), h(1269), h(1280), h(1290)),
            (167, 116, 216, 159, 95)
        );
        assert_eq!((800..=1350).map(h).max(), Some(216));
        assert!((1300..=1350).all(|y| h(y) == 0) && h(1299) > 0);
        assert_eq!((800..=1350).map(h).sum::<u32>(), 52_108);
        assert_eq!(v.historical(100), 0, "outside the data");
    }

    #[test]
    fn a_site_rises_to_its_median_and_falls_to_its_end() {
        // start 1000, end 1100, median 1050, 10 households at the median.
        let n = |y| site_households(1000, 1100, 1050, 10, y);
        assert_eq!(
            (n(999), n(1000), n(1001), n(1050), n(1051), n(1099), n(1100)),
            (0, 1, 1, 10, 10, 1, 0)
        );
        assert_eq!(n(1025), 5);
        // Median at the start: 0 up to it, then falling.
        assert_eq!(site_households(1000, 1100, 1000, 10, 1000), 0);
        assert_eq!(site_households(1000, 1100, 1000, 10, 1001), 10);
    }
}
