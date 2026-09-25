//! The Long House Valley world: one tick is one year, run in the ODD's
//! order (§3.1). Rules marked *adopted* come from the published replication
//! because the ODD and JASSS do not define them (the spec's "Adopted from
//! the replication in both modes"); rules marked with a quirk follow the
//! replication only while that switch is on.

use std::collections::BTreeMap;
use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use crate::anasazi::config::AnasaziConfig;
use crate::anasazi::household::{
    consume, expected_harvest, split_endowment, store_harvest, Household,
};
use crate::anasazi::random::normal;
use crate::anasazi::valley::{self, column_yield, pdsi_class, Valley, Zone, CELLS, HEIGHT, WIDTH};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, BOTH, LENDER, SUGAR};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// One mile — the ODD's 1600 m at 100 m a cell (§2.1) — squared, in cells:
/// the farthest a farm may be from water and a residence from its farm.
pub const MILE2: u32 = 16 * 16;
/// Initial ages are whole years uniform in `[0, 29)` (§4, the replication's
/// reading of the ODD's "[0, 29]").
const INITIAL_AGES: std::ops::Range<u32> = 0..29;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 9] = [
    "households",
    "historical",
    "fit",
    "capacity",
    "mean_corn",
    "births",
    "moves",
    "departures",
    "year",
];

/// One year's statistics, taken after the year's fission (§6).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct AnasaziSnapshot {
    pub tick: u64,
    pub year: u32,
    pub households: u32,
    /// The archaeological estimate for this year (§5.6).
    pub historical: u32,
    /// The sum of `(households − historical)²` over every year so far,
    /// this one included (JASSS ¶4.6's L2).
    pub fit: f64,
    /// Cells whose base yield this year meets the food need (§6).
    pub capacity: u32,
    /// Mean corn per household (kg), 0 with none.
    pub mean_corn: f64,
    pub births: u32,
    /// Households that moved to a new farm this year.
    pub moves: u32,
    /// Households that left the valley for want of a farm or home.
    pub departures: u32,
}

impl Series for AnasaziSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "year" => f64::from(self.year),
            "households" => f64::from(self.households),
            "historical" => f64::from(self.historical),
            "fit" => self.fit,
            "capacity" => f64::from(self.capacity),
            "mean_corn" => self.mean_corn,
            "births" => f64::from(self.births),
            "moves" => f64::from(self.moves),
            "departures" => f64::from(self.departures),
            _ => return None,
        })
    }
}

/// The color modes the valley can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnasaziMode {
    /// Farms and homes over dimmed zones.
    Occupation,
    /// The land-cover zones.
    Zones,
    /// This year's base yield.
    Yield,
}

impl std::str::FromStr for AnasaziMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "occupation" => Self::Occupation,
            "zones" => Self::Zones,
            "yield" => Self::Yield,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// Each zone's color, after the ODD's Figure 1 on a dark ground.
pub fn zone_color(zone: Zone) -> Rgb {
    match zone {
        Zone::General => [0x8a, 0x7a, 0x52],
        Zone::North => [0xc0, 0x50, 0x4d],
        Zone::NorthDunes | Zone::MidDunes => [0xe8, 0xe2, 0xcf],
        Zone::Mid => [0x9a, 0x9a, 0x9a],
        Zone::Natural => [0x5c, 0x52, 0x30],
        Zone::Uplands => [0x4f, 0x7f, 0xbf],
        Zone::Kinbiko => [0xd9, 0x77, 0xb0],
        Zone::Empty => BACKGROUND,
    }
}
/// A farm plot and a home in the Occupation mode.
pub const FARM: Rgb = LENDER;
pub const HOME: Rgb = BOTH;

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AnasaziInspection {
    pub site: CellView,
    /// The cell's farmer, else its first resident.
    pub agent: Option<HouseholdView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CellView {
    pub x: u32,
    pub y: u32,
    pub zone: Zone,
    pub zone_name: &'static str,
    pub pdsi: f64,
    /// 0 (≤ −3) to 4 (≥ 3).
    pub pdsi_class: usize,
    /// The zone's yield this year, `y`.
    pub zone_yield: f64,
    /// Soil quality `q`.
    pub quality: f64,
    /// This year's `BY = y · q · Ha`.
    pub base_yield: f64,
    /// A water source now.
    pub water: bool,
    /// Within a mile of a water source.
    pub water_near: bool,
    /// Whether the zone's hydrology allows a home here this year (A-4).
    pub habitable: bool,
    pub farmed_by: Option<u64>,
    /// The households living here, by id.
    pub residents: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HouseholdView {
    pub id: u64,
    pub age: u32,
    /// Newest first.
    pub corn: Vec<f64>,
    pub stock: f64,
    pub harvest: f64,
    pub expected: f64,
    pub farm: [u32; 2],
    pub home: [u32; 2],
}

fn xy(cell: u32) -> [u32; 2] {
    [cell % WIDTH, cell / WIDTH]
}

/// Squared distance in cells between two cells, across the edges when
/// `wrap` (A-9) and within the map otherwise.
pub fn distance2(a: u32, b: u32, wrap: bool) -> u32 {
    let (ax, ay, bx, by) = (a % WIDTH, a / WIDTH, b % WIDTH, b / WIDTH);
    let (mut dx, mut dy) = (ax.abs_diff(bx), ay.abs_diff(by));
    if wrap {
        dx = dx.min(WIDTH - dx);
        dy = dy.min(HEIGHT - dy);
    }
    dx * dx + dy * dy
}

/// Each cell's squared distance to the nearest water cell (`u32::MAX` with
/// none): the nearest in its column, then the nearest over columns.
fn water_distances(water: &[bool], wrap: bool) -> Vec<u32> {
    let (w, h) = (WIDTH as usize, HEIGHT as usize);
    let gap = |a: usize, b: usize, n: usize| {
        let d = a.abs_diff(b);
        (if wrap { d.min(n - d) } else { d }) as u32
    };
    let mut column = vec![u32::MAX; CELLS];
    for x in 0..w {
        let rows: Vec<usize> = (0..h).filter(|&y| water[y * w + x]).collect();
        for y in 0..h {
            column[y * w + x] = rows
                .iter()
                .map(|&r| gap(y, r, h).pow(2))
                .min()
                .unwrap_or(u32::MAX);
        }
    }
    let mut out = vec![u32::MAX; CELLS];
    for y in 0..h {
        for x in 0..w {
            out[y * w + x] = (0..w)
                .filter_map(|x2| {
                    let v = column[y * w + x2];
                    (v != u32::MAX).then(|| v + gap(x, x2, w).pow(2))
                })
                .min()
                .unwrap_or(u32::MAX);
        }
    }
    out
}

pub struct AnasaziWorld {
    pub config: AnasaziConfig,
    /// Completed ticks (years since the start year).
    pub tick: u64,
    year: u32,
    valley: &'static Valley,
    /// Soil quality `q`, drawn once (§3.3).
    quality: Vec<f64>,
    /// This year's yield `y` by zone (`Zone::ALL` order), and whether its
    /// hydrology allows homes (A-4).
    zone_yield: [f64; 9],
    habitable: [bool; 9],
    /// This year's `BY = y · q · Ha` per cell.
    base_yield: Vec<f64>,
    /// The cells whose base yield meets the need, in cell order.
    productive: Vec<u32>,
    /// Water now (updated at the end of each year, §3.11) and each cell's
    /// squared distance to it.
    water: Vec<bool>,
    water_d2: Vec<u32>,
    /// Each cell's farmer.
    farmer: Vec<Option<u64>>,
    /// Each cell's count of residents as the rules see it (with the
    /// occupancy leak, more than live there).
    settled: Vec<u32>,
    households: BTreeMap<u64, Household>,
    next_id: u64,
    rng: SimRng,
    fit: f64,
    births: u32,
    moves: u32,
    departures: u32,
    pub stats: Stats<AnasaziSnapshot>,
}

fn zone_index(zone: Zone) -> usize {
    Zone::ALL
        .iter()
        .position(|&z| z == zone)
        .expect("every zone is listed")
}

impl AnasaziWorld {
    /// The valley in the start year: soil quality, then the initial
    /// households (§4), in that RNG order.
    pub fn new(config: AnasaziConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let sd = config.soil_sd();
        // q = 1 + n(0, σ), at least 0 (§3.3; A-1 picks σ).
        let quality = (0..CELLS)
            .map(|_| (1.0 + sd * normal(&mut rng)).max(0.0))
            .collect();
        let year = config.start_year;
        let valley = valley::valley();
        let water = valley.water_sources(year);
        let water_d2 = water_distances(&water, config.quirks.wrap_edges);
        let mut world = AnasaziWorld {
            tick: 0,
            year,
            valley,
            quality,
            zone_yield: [0.0; 9],
            habitable: [false; 9],
            base_yield: vec![0.0; CELLS],
            productive: Vec::new(),
            water,
            water_d2,
            farmer: vec![None; CELLS],
            settled: vec![0; CELLS],
            households: BTreeMap::new(),
            next_id: 1,
            rng,
            fit: 0.0,
            births: 0,
            moves: 0,
            departures: 0,
            stats: Stats::default(),
            config,
        };
        world.prepare_year();
        for _ in 0..world.config.initial_households {
            world.add_initial_household();
        }
        world.record();
        Ok(world)
    }

    /// One initial household (§4): its age, its corn, a random cell from
    /// which it takes the nearest farm, then a home by the settlement rule.
    /// One that finds neither is not added.
    fn add_initial_household(&mut self) {
        let c = &self.config;
        let (range, slots, per_slot) = (
            c.initial_corn,
            c.storage_years as usize + 1,
            c.quirks.initial_corn_per_slot,
        );
        let age = self.rng.gen_range(INITIAL_AGES);
        let mut corn = vec![0.0; slots];
        if per_slot {
            // A-11: a draw in every slot.
            for slot in corn.iter_mut() {
                *slot = self.rng.gen_range(range.min..=range.max);
            }
        } else {
            corn[0] = self.rng.gen_range(range.min..=range.max);
        }
        let from = self.rng.gen_range(0..CELLS as u32);
        let id = self.next_id;
        let Some(farm) = self.find_farm(from, true) else {
            return;
        };
        self.farmer[farm as usize] = Some(id);
        let Some(home) = self.find_home(farm) else {
            self.farmer[farm as usize] = None;
            return;
        };
        self.settle(home);
        self.next_id += 1;
        let expected = expected_harvest(&corn, 0.0);
        self.households.insert(
            id,
            Household {
                id,
                farm,
                home,
                age,
                corn,
                harvest: 0.0,
                expected,
            },
        );
    }

    /// The current year.
    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn households(&self) -> impl Iterator<Item = &Household> {
        self.households.values()
    }

    /// Whether the end year has been simulated.
    pub fn is_finished(&self) -> bool {
        self.year >= self.config.end_year
    }

    /// This year's yields (§3.2) for the current `year`, the cells that can
    /// feed a household, and where homes are allowed (A-4).
    fn prepare_year(&mut self) {
        let single = self.config.quirks.uplands_single_class;
        for (k, &zone) in Zone::ALL.iter().enumerate() {
            let class = pdsi_class(self.valley.zone_pdsi(zone, self.year, single));
            self.zone_yield[k] = zone.column().map_or(0.0, |c| column_yield(c, class));
            self.habitable[k] = self.valley.hydro(zone, self.year) <= 0.0;
        }
        let ha = self.config.harvest_adjustment;
        self.productive.clear();
        for i in 0..CELLS {
            let y = self.zone_yield[zone_index(self.valley.zone(i))];
            self.base_yield[i] = y * self.quality[i] * ha;
            if self.base_yield[i] >= self.config.need {
                self.productive.push(i as u32);
            }
        }
    }

    fn cell_yield(&self, cell: u32) -> f64 {
        self.zone_yield[zone_index(self.valley.zone(cell as usize))]
    }

    fn cell_habitable(&self, cell: u32) -> bool {
        self.habitable[zone_index(self.valley.zone(cell as usize))]
    }

    /// Whether a household could farm `cell` now (§3.8): not farmed, nobody
    /// living there, inside the valley and — unless `no_farm_water_check`
    /// (A-3) — within a mile of water.
    fn farmable(&self, cell: u32) -> bool {
        let i = cell as usize;
        self.farmer[i].is_none()
            && self.settled[i] == 0
            && self.valley.zone(i) != Zone::Empty
            && (self.config.quirks.no_farm_water_check || self.water_d2[i] <= MILE2)
    }

    /// Uniformly one of `cells` (none: `None`); draws only for a real choice
    /// (ties are broken at random, A-10).
    fn pick(&mut self, cells: &[u32]) -> Option<u32> {
        match cells.len() {
            0 => None,
            1 => Some(cells[0]),
            n => Some(cells[self.rng.gen_range(0..n as u32) as usize]),
        }
    }

    /// The farm plot for a household searching from `from` (§3.8): of the
    /// farmable cells whose base yield meets the need, the nearest. The
    /// first households (`initial`) judge the yield without the harvest
    /// adjustment when `initial_eligibility_ignores_adjustment` (A-13).
    fn find_farm(&mut self, from: u32, initial: bool) -> Option<u32> {
        let wrap = self.config.quirks.wrap_edges;
        let unadjusted = initial && self.config.quirks.initial_eligibility_ignores_adjustment;
        let candidates: Vec<u32> = if unadjusted {
            (0..CELLS as u32)
                .filter(|&c| self.cell_yield(c) * self.quality[c as usize] >= self.config.need)
                .collect()
        } else {
            self.productive.clone()
        };
        let mut best = u32::MAX;
        let mut ties = Vec::new();
        for c in candidates {
            if !self.farmable(c) {
                continue;
            }
            let d = distance2(from, c, wrap);
            if d < best {
                best = d;
                ties.clear();
            }
            if d == best {
                ties.push(c);
            }
        }
        self.pick(&ties)
    }

    /// Whether any cell is free to farm now (`fission_needs_free_farm`).
    fn any_free_farm(&self) -> bool {
        self.productive.iter().any(|&c| self.farmable(c))
    }

    /// The cells within a mile of `farm`, row by row.
    fn mile_around(&self, farm: u32) -> Vec<u32> {
        let wrap = self.config.quirks.wrap_edges;
        let (fx, fy) = ((farm % WIDTH) as i64, (farm / WIDTH) as i64);
        let (w, h) = (i64::from(WIDTH), i64::from(HEIGHT));
        let mut out = Vec::new();
        for dy in -16i64..=16 {
            for dx in -16i64..=16 {
                if dx * dx + dy * dy > i64::from(MILE2) {
                    continue;
                }
                let (mut x, mut y) = (fx + dx, fy + dy);
                if wrap {
                    x = x.rem_euclid(w);
                    y = y.rem_euclid(h);
                } else if !(0..w).contains(&x) || !(0..h).contains(&y) {
                    continue;
                }
                out.push((y * w + x) as u32);
            }
        }
        out
    }

    /// The home for a household farming `farm` (ODD p.5, §3.9): among
    /// unfarmed cells whose hydrology allows homes this year (A-4, adopted),
    /// those (i) within a mile of the farm (ii) in a less productive zone —
    /// this year's zone yield below the farm's (A-7, adopted); failing that,
    /// within a mile; failing that, anywhere. Of the first tier that has
    /// any, the one closest to water, ties at random.
    fn find_home(&mut self, farm: u32) -> Option<u32> {
        let near = self.mile_around(farm);
        let farm_yield = self.cell_yield(farm);
        let ok = |w: &Self, c: u32| w.farmer[c as usize].is_none() && w.cell_habitable(c);
        let tiers: [Vec<u32>; 3] = [
            near.iter()
                .copied()
                .filter(|&c| ok(self, c) && self.cell_yield(c) < farm_yield)
                .collect(),
            near.iter().copied().filter(|&c| ok(self, c)).collect(),
            (0..CELLS as u32).filter(|&c| ok(self, c)).collect(),
        ];
        let tier = tiers.into_iter().find(|t| !t.is_empty())?;
        let best = tier.iter().map(|&c| self.water_d2[c as usize]).min()?;
        let ties: Vec<u32> = tier
            .into_iter()
            .filter(|&c| self.water_d2[c as usize] == best)
            .collect();
        self.pick(&ties)
    }

    /// A household starts living on `home`: counted twice with the
    /// occupancy leak (A-19).
    fn settle(&mut self, home: u32) {
        self.settled[home as usize] += if self.config.quirks.occupancy_leak {
            2
        } else {
            1
        };
    }

    fn vacate(&mut self, home: u32) {
        let n = &mut self.settled[home as usize];
        *n = n.saturating_sub(1);
    }

    /// Removes household `id`: its farm is free and its home loses it.
    fn remove(&mut self, id: u64) {
        if let Some(h) = self.households.remove(&id) {
            if self.farmer[h.farm as usize] == Some(id) {
                self.farmer[h.farm as usize] = None;
            }
            self.vacate(h.home);
        }
    }

    /// One year (§3.1), in the ODD's order; households act in one random
    /// order for the whole year.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        self.year += 1;
        self.tick += 1;
        (self.births, self.moves, self.departures) = (0, 0, 0);
        self.prepare_year();
        let mut order: Vec<u64> = self.households.keys().copied().collect();
        order.shuffle(&mut self.rng);
        self.harvest_and_eat(&order);
        let alive: Vec<u64> = order
            .into_iter()
            .filter(|id| self.households.contains_key(id))
            .collect();
        for &id in &alive {
            self.expect_and_move(id);
        }
        for &id in &alive {
            if self.households.contains_key(&id) {
                self.fission(id);
            }
        }
        // 8: water for the year just simulated, used from next year on.
        self.update_water();
        // 9: aging, unless it came first (A-2).
        if !self.config.quirks.age_before_death_check {
            for h in self.households.values_mut() {
                h.age += 1;
            }
        }
        self.record();
    }

    /// Steps 1–2: every household harvests `H0 = BY · (1 + n(0, σahv))`
    /// (§3.4; negative harvests are kept, A-14), ages its store, eats, and
    /// is removed if it could not eat its need or is older than the death
    /// age (§3.5–3.6).
    fn harvest_and_eat(&mut self, order: &[u64]) {
        let (sd, need, death) = (
            self.config.annual_sd,
            self.config.need,
            self.config.death_age,
        );
        let early_age = self.config.quirks.age_before_death_check;
        for &id in order {
            let noise = normal(&mut self.rng);
            let h = self
                .households
                .get_mut(&id)
                .expect("listed households live");
            if early_age {
                h.age += 1;
            }
            let harvest = self.base_yield[h.farm as usize] * (1.0 + sd * noise);
            h.harvest = harvest;
            store_harvest(&mut h.corn, harvest);
            let unmet = consume(&mut h.corn, need);
            if unmet > 0.0 || h.age > death {
                self.remove(id);
            }
        }
    }

    /// Steps 3–6: the expected harvest (§3.7); a household expecting less
    /// than its need moves — its farm is freed and it takes the nearest free
    /// farm from there (§3.8), then a home (§3.9) — or, with no farm or home
    /// to be had, leaves the valley.
    fn expect_and_move(&mut self, id: u64) {
        let need = self.config.need;
        let h = self
            .households
            .get_mut(&id)
            .expect("listed households live");
        h.expected = expected_harvest(&h.corn, h.harvest);
        if h.expected >= need {
            return;
        }
        let (from, old_home) = (h.farm, h.home);
        self.farmer[from as usize] = None;
        let Some(farm) = self.find_farm(from, false) else {
            self.remove(id);
            self.departures += 1;
            return;
        };
        self.farmer[farm as usize] = Some(id);
        let leak = self.config.quirks.occupancy_leak;
        if !leak {
            self.vacate(old_home);
        }
        let Some(home) = self.find_home(farm) else {
            self.farmer[farm as usize] = None;
            self.households.remove(&id);
            if leak {
                self.vacate(old_home);
            }
            self.departures += 1;
            return;
        };
        self.settle(home);
        let h = self
            .households
            .get_mut(&id)
            .expect("listed households live");
        (h.farm, h.home) = (farm, home);
        self.moves += 1;
    }

    /// Step 7 (§3.10): a household older than the fertility start and at
    /// most the fertility end splits with probability pf; the new household
    /// (age 0) takes the nearest free farm from its parent's farm and a
    /// home, and gets its corn — or does not establish.
    fn fission(&mut self, id: u64) {
        let c = &self.config;
        let h = &self.households[&id];
        if h.age <= c.fertility_start || h.age > c.fertility_end {
            return;
        }
        if c.quirks.fission_needs_free_farm && !self.any_free_farm() {
            return;
        }
        let p = c.fission_probability;
        if !self.rng.gen_bool(p) {
            return;
        }
        let parent_farm = h.farm;
        let child_id = self.next_id;
        let Some(farm) = self.find_farm(parent_farm, false) else {
            return;
        };
        self.farmer[farm as usize] = Some(child_id);
        let Some(home) = self.find_home(farm) else {
            self.farmer[farm as usize] = None;
            return;
        };
        self.settle(home);
        self.next_id += 1;
        let (fraction, range) = (self.config.child_endowment, self.config.initial_corn);
        let fresh = self.config.quirks.fission_fresh_endowment;
        let parent = self.households.get_mut(&id).expect("the parent lives");
        let corn = if fresh {
            // A-12: the parent loses its share, the child's corn is drawn
            // fresh — fcs / (1 − fcs) × an initial-corn draw per slot.
            for slot in parent.corn.iter_mut() {
                *slot *= 1.0 - fraction;
            }
            let slots = parent.corn.len();
            (0..slots)
                .map(|_| fraction / (1.0 - fraction) * self.rng.gen_range(range.min..=range.max))
                .collect()
        } else {
            split_endowment(&mut parent.corn, fraction)
        };
        let expected = expected_harvest(&corn, 0.0);
        self.households.insert(
            child_id,
            Household {
                id: child_id,
                farm,
                home,
                age: 0,
                corn,
                harvest: 0.0,
                expected,
            },
        );
        self.births += 1;
    }

    /// Step 8: this year's water sources (A-5) and the distances to them.
    fn update_water(&mut self) {
        let water = self.valley.water_sources(self.year);
        if water != self.water {
            self.water_d2 = water_distances(&water, self.config.quirks.wrap_edges);
            self.water = water;
        }
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
        let n = self.households.len() as u32;
        let historical = self.valley.historical(self.year);
        self.fit += (f64::from(n) - f64::from(historical)).powi(2);
        let stock: f64 = self.households.values().map(Household::stock).sum();
        let snapshot = AnasaziSnapshot {
            tick: self.tick,
            year: self.year,
            households: n,
            historical,
            fit: self.fit,
            capacity: self.productive.len() as u32,
            mean_corn: if n == 0 { 0.0 } else { stock / f64::from(n) },
            births: self.births,
            moves: self.moves,
            departures: self.departures,
        };
        self.stats.push(snapshot);
    }

    /// Households living on each cell (by id), for cells with any.
    fn residents(&self) -> BTreeMap<u32, Vec<u64>> {
        let mut out: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
        for h in self.households.values() {
            out.entry(h.home).or_default().push(h.id);
        }
        out
    }

    /// Water sources now, as `[x, y, …]`.
    pub fn water_xy(&self) -> Vec<u32> {
        (0..CELLS as u32)
            .filter(|&c| self.water[c as usize])
            .flat_map(xy)
            .collect()
    }

    /// Every inhabited cell and its households, as `[x, y, n, …]`.
    pub fn settlements_xy(&self) -> Vec<u32> {
        self.residents()
            .into_iter()
            .flat_map(|(c, ids)| {
                let [x, y] = xy(c);
                [x, y, ids.len() as u32]
            })
            .collect()
    }

    /// Each household's farm and home, as `[farm x, farm y, home x, home y, …]`.
    pub fn links_xy(&self) -> Vec<u32> {
        self.households
            .values()
            .flat_map(|h| [xy(h.farm), xy(h.home)].concat())
            .collect()
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<AnasaziInspection, String> {
        if x >= WIDTH || y >= HEIGHT {
            return Err(format!("({x}, {y}) is not in the valley"));
        }
        let cell = y * WIDTH + x;
        let i = cell as usize;
        let zone = self.valley.zone(i);
        let pdsi = self
            .valley
            .zone_pdsi(zone, self.year, self.config.quirks.uplands_single_class);
        let residents = self.residents().remove(&cell).unwrap_or_default();
        let shown = self.farmer[i].or_else(|| residents.first().copied());
        Ok(AnasaziInspection {
            site: CellView {
                x,
                y,
                zone,
                zone_name: zone.name(),
                pdsi,
                pdsi_class: pdsi_class(pdsi),
                zone_yield: self.cell_yield(cell),
                quality: self.quality[i],
                base_yield: self.base_yield[i],
                water: self.water[i],
                water_near: self.water_d2[i] <= MILE2,
                habitable: self.cell_habitable(cell),
                farmed_by: self.farmer[i],
                residents,
            },
            agent: shown.map(|id| {
                let h = &self.households[&id];
                HouseholdView {
                    id,
                    age: h.age,
                    corn: h.corn.clone(),
                    stock: h.stock(),
                    harvest: h.harvest,
                    expected: h.expected,
                    farm: xy(h.farm),
                    home: xy(h.home),
                }
            }),
        })
    }

    fn color(&self, i: usize, mode: AnasaziMode, homes: &[bool]) -> Rgb {
        let zone = self.valley.zone(i);
        match mode {
            AnasaziMode::Zones => zone_color(zone),
            AnasaziMode::Yield if zone == Zone::Empty => BACKGROUND,
            AnasaziMode::Yield => lerp(
                BACKGROUND,
                SUGAR,
                (self.base_yield[i] / (1.5 * self.config.need)).clamp(0.0, 1.0),
            ),
            AnasaziMode::Occupation if self.farmer[i].is_some() => FARM,
            AnasaziMode::Occupation if homes[i] => HOME,
            AnasaziMode::Occupation => lerp(BACKGROUND, zone_color(zone), 0.3),
        }
    }
}

impl Model for AnasaziWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Anasazi(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        AnasaziWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.households.len()
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// FNV-1a over the tick, every cell's soil quality, and every
    /// household's id, farm and home, age and corn.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for q in &self.quality {
            eat(q.to_bits());
        }
        for hh in self.households.values() {
            eat(hh.id);
            eat((u64::from(hh.farm) << 32) | u64::from(hh.home));
            eat(u64::from(hh.age));
            for c in &hh.corn {
                eat(c.to_bits());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (WIDTH, HEIGHT)
    }

    /// The valley, north at the top, in `occupation`, `zones` or `yield`
    /// mode; `layer` is ignored.
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: AnasaziMode = mode.parse()?;
        let mut homes = vec![false; CELLS];
        for h in self.households.values() {
            homes[h.home as usize] = true;
        }
        buf.clear();
        buf.resize(CELLS * 4, 0);
        for (i, px) in buf.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let [r, g, b] = self.color(i, mode, &homes);
            *px = [r, g, b, 255];
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
        let mut out = String::from("id,farm_x,farm_y,home_x,home_y,age,corn,harvest,expected\n");
        for h in self.households.values() {
            let ([fx, fy], [hx, hy]) = (xy(h.farm), xy(h.home));
            writeln!(
                out,
                "{},{fx},{fy},{hx},{hy},{},{},{},{}",
                h.id,
                h.age,
                h.stock(),
                h.harvest,
                h.expected
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// A household's farm.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        self.households
            .get(&id)
            .map(|h| (h.farm % WIDTH, h.farm / WIDTH))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Anasazi(next) = next else {
            return Err(wrong_model(ModelKind::Anasazi, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        // A new harvest adjustment changes this year's base yields now.
        self.prepare_year();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anasazi::config::Quirks;
    use crate::anasazi::valley::PdsiSeries;

    /// A calibrated world in AD 800 with no households, every soil quality 1
    /// and no harvest noise.
    fn world(quirks: Quirks) -> AnasaziWorld {
        world_seeded(quirks, 1)
    }

    fn world_seeded(quirks: Quirks, seed: u64) -> AnasaziWorld {
        let config = AnasaziConfig {
            initial_households: 0,
            spatial_sd: 0.0,
            annual_sd: 0.0,
            ..AnasaziConfig::calibrated(quirks)
        };
        AnasaziWorld::new(config, seed).unwrap()
    }

    fn at(x: u32, y: u32) -> u32 {
        y * WIDTH + x
    }

    /// Adds household `id` farming `farm` and living on `home`.
    fn put(w: &mut AnasaziWorld, id: u64, farm: u32, home: u32, age: u32, corn: &[f64]) {
        w.farmer[farm as usize] = Some(id);
        w.settle(home);
        w.households.insert(
            id,
            Household {
                id,
                farm,
                home,
                age,
                corn: corn.to_vec(),
                harvest: 0.0,
                expected: 0.0,
            },
        );
        w.next_id = w.next_id.max(id + 1);
    }

    /// A run of North Valley Floor cells, (45, 28) to (51, 28), and the
    /// Uplands Nonarable cell (48, 27) north of it.
    const NORTH: [(u32, u32); 7] = [
        (45, 28),
        (46, 28),
        (47, 28),
        (48, 28),
        (49, 28),
        (50, 28),
        (51, 28),
    ];

    #[test]
    fn distances_wrap_only_with_the_quirk() {
        assert_eq!(distance2(at(0, 10), at(79, 10), false), 79 * 79);
        assert_eq!(distance2(at(0, 10), at(79, 10), true), 1);
        assert_eq!(distance2(at(5, 0), at(5, 119), true), 1);
        assert_eq!(distance2(at(3, 4), at(0, 0), false), 25);
    }

    #[test]
    fn water_distances_are_the_nearest_squared_distance() {
        let mut water = vec![false; CELLS];
        for c in [at(10, 10), at(70, 100), at(0, 60)] {
            water[c as usize] = true;
        }
        for wrap in [false, true] {
            let d = water_distances(&water, wrap);
            for cell in (0..CELLS as u32).step_by(37) {
                let brute = [at(10, 10), at(70, 100), at(0, 60)]
                    .iter()
                    .map(|&w| distance2(cell, w, wrap))
                    .min()
                    .unwrap();
                assert_eq!(d[cell as usize], brute, "cell {cell} wrap {wrap}");
            }
        }
        assert!(water_distances(&vec![false; CELLS], false)
            .iter()
            .all(|&d| d == u32::MAX));
    }

    #[test]
    fn base_yield_is_zone_yield_times_quality_times_adjustment() {
        let mut w = world(Quirks::NONE);
        let cell = at(48, 28);
        let class = pdsi_class(w.valley.pdsi(PdsiSeries::North, 800));
        let y = column_yield(valley::YieldColumn::NorthMidKinbiko, class);
        w.quality[cell as usize] = 1.5;
        w.prepare_year();
        assert_eq!(w.base_yield[cell as usize], y * 1.5 * 0.56);
        assert_eq!(w.productive.contains(&cell), y * 1.5 * 0.56 >= 800.0);
        assert_eq!(
            w.cell_yield(at(48, 27)),
            0.0,
            "Uplands Nonarable yield nothing"
        );
        let cap = w.productive.len();
        assert_eq!(
            cap,
            (0..CELLS).filter(|&i| w.base_yield[i] >= 800.0).count()
        );
    }

    #[test]
    fn a_farm_must_feed_a_household_be_free_and_lie_near_water() {
        let mut w = world(Quirks::NONE);
        let [a, b, c, d] = [0, 1, 2, 3].map(|k| at(NORTH[k].0, NORTH[k].1));
        w.productive = vec![a, b, c, d];
        w.water_d2 = vec![0; CELLS];
        w.farmer[a as usize] = Some(99);
        w.settled[b as usize] = 1;
        w.water_d2[c as usize] = MILE2 + 1;
        let from = at(40, 28);
        assert_eq!(
            w.find_farm(from, false),
            Some(d),
            "farmed, lived on and dry cells are skipped"
        );
        w.config.quirks.no_farm_water_check = true;
        assert_eq!(w.find_farm(from, false), Some(c), "A-3: no water check");
        w.productive.clear();
        assert_eq!(w.find_farm(from, false), None, "no cell feeds a household");
    }

    #[test]
    fn the_nearest_farm_wins_and_equally_near_farms_are_drawn_at_random() {
        let mut w = world(Quirks::NONE);
        w.water_d2 = vec![0; CELLS];
        w.productive = vec![at(45, 28), at(47, 28)];
        assert_eq!(w.find_farm(at(48, 28), false), Some(at(47, 28)));
        let mut seen = std::collections::BTreeSet::new();
        for seed in 1..=20 {
            let mut w = world_seeded(Quirks::NONE, seed);
            w.water_d2 = vec![0; CELLS];
            w.productive = vec![at(45, 28), at(51, 28)];
            seen.insert(w.find_farm(at(48, 28), false).unwrap());
        }
        assert_eq!(seen.len(), 2, "both of the tied cells are chosen");
    }

    #[test]
    fn farm_searches_wrap_only_with_the_quirk() {
        // (70, 0) and (70, 2) are North Valley Floor on the top edge.
        let mut w = world(Quirks::NONE);
        w.water_d2 = vec![0; CELLS];
        w.productive = vec![at(70, 0), at(70, 2)];
        let from = at(70, 115);
        assert_eq!(w.find_farm(from, false), Some(at(70, 2)));
        w.config.quirks.wrap_edges = true;
        assert_eq!(
            w.find_farm(from, false),
            Some(at(70, 0)),
            "5 rows away across the edge"
        );
    }

    #[test]
    fn homes_fall_back_from_poorer_nearby_cells_to_any_nearby_cell_to_anywhere() {
        let mut w = world(Quirks::NONE);
        let farm = at(48, 28);
        w.farmer[farm as usize] = Some(1);
        w.water_d2 = vec![u32::MAX; CELLS];
        // Tier 1: less productive (Uplands Nonarable yield 0), the one
        // closest to water.
        w.water_d2[at(48, 27) as usize] = 100;
        w.water_d2[at(46, 27) as usize] = 50;
        assert_eq!(w.find_home(farm), Some(at(46, 27)));
        // Tier 2: nothing poorer than the farm, so any nearby cell.
        w.zone_yield[zone_index(Zone::North)] = 0.0;
        w.water_d2[at(48, 26) as usize] = 10;
        assert_eq!(w.find_home(farm), Some(at(48, 26)));
        // Tier 3: only Arable Uplands allowed, none within a mile.
        w.habitable = [false; 9];
        w.habitable[zone_index(Zone::Uplands)] = true;
        w.water_d2[at(17, 29) as usize] = 0;
        assert_eq!(w.find_home(farm), Some(at(17, 29)));
        // None at all: the household would leave.
        w.habitable = [false; 9];
        assert_eq!(w.find_home(farm), None);
    }

    #[test]
    fn farmed_and_uninhabitable_cells_are_never_homes() {
        let mut w = world(Quirks::NONE);
        let farm = at(48, 28);
        w.water_d2 = vec![u32::MAX; CELLS];
        w.water_d2[at(48, 27) as usize] = 0;
        w.water_d2[at(49, 28) as usize] = 0;
        w.farmer[at(48, 27) as usize] = Some(7);
        // (49, 28) is North Valley Floor: its hydro forbids homes (A-4).
        assert!(!w.cell_habitable(at(49, 28)));
        let home = w.find_home(farm).unwrap();
        assert!(home != at(48, 27) && home != at(49, 28));
    }

    #[test]
    fn a_harvest_is_the_farms_base_yield_times_one_plus_its_noise() {
        // Without noise the harvest is exactly the base yield (§3.4).
        let mut w = world(Quirks::NONE);
        let farm = at(48, 28);
        w.quality[farm as usize] = 2.0;
        put(&mut w, 1, farm, at(46, 27), 5, &[5000.0, 5000.0, 5000.0]);
        w.step();
        let by = w.base_yield[farm as usize];
        assert!(by > 0.0);
        assert_eq!(w.households[&1].harvest, by);
        assert_eq!(w.households[&1].corn[0], by, "S0 holds this year's harvest");
        // With noise, harvests scatter around it: n(0, σ) per household.
        let mut w = world(Quirks::NONE);
        w.config.annual_sd = 0.4;
        for k in 0..7 {
            let (x, y) = NORTH[k as usize];
            w.quality[at(x, y) as usize] = 2.0;
            put(
                &mut w,
                k + 1,
                at(x, y),
                at(46, 27),
                5,
                &[5000.0, 5000.0, 5000.0],
            );
        }
        w.step();
        let ratios: Vec<f64> = w
            .households
            .values()
            .map(|h| h.harvest / w.base_yield[h.farm as usize])
            .collect();
        assert!(ratios.iter().any(|r| (r - 1.0).abs() > 0.01), "{ratios:?}");
        assert!(ratios.iter().all(|r| (r - 1.0).abs() < 2.0), "{ratios:?}");
    }

    #[test]
    fn households_that_cannot_eat_or_are_too_old_are_removed() {
        let mut w = world(Quirks::NONE);
        w.config.end_year = 1350;
        let farm = at(48, 28);
        let (barren, home) = (at(48, 27), at(46, 27));
        put(&mut w, 1, barren, home, 5, &[0.0, 0.0, 700.0]);
        put(&mut w, 2, farm, home, 38, &[5000.0, 5000.0, 5000.0]);
        put(&mut w, 3, at(50, 28), home, 37, &[5000.0, 5000.0, 5000.0]);
        w.step();
        assert!(
            !w.households.contains_key(&1),
            "700 kg and a harvest of 0 cannot feed 800"
        );
        assert!(
            w.households.contains_key(&2),
            "38 is not older than 38 until it ages"
        );
        assert_eq!(w.households[&2].age, 39, "aging comes last");
        w.step();
        assert!(!w.households.contains_key(&2), "39 > 38");
        assert!(w.households.contains_key(&3));
        assert!(w.farmer[farm as usize].is_none(), "its farm is free again");
    }

    #[test]
    fn with_age_before_death_check_households_die_a_year_sooner() {
        let mut q = Quirks::NONE;
        q.age_before_death_check = true;
        let mut w = world(q);
        put(
            &mut w,
            1,
            at(48, 28),
            at(46, 27),
            38,
            &[5000.0, 5000.0, 5000.0],
        );
        w.step();
        assert!(w.households.is_empty(), "aged to 39 before the check");
    }

    #[test]
    fn a_household_expecting_too_little_moves_to_the_nearest_farm_or_leaves() {
        let mut w = world(Quirks::NONE);
        let home = at(46, 27);
        // Its farm yields nothing: it expects 700 kg next year.
        put(&mut w, 1, at(48, 27), home, 5, &[700.0, 0.0, 0.0]);
        w.water_d2 = vec![0; CELLS];
        w.productive = vec![at(51, 28)];
        w.expect_and_move(1);
        let h = &w.households[&1];
        assert_eq!(h.farm, at(51, 28));
        assert!(w.farmer[at(48, 27) as usize].is_none());
        assert_eq!(w.moves, 1);
        assert_eq!(w.settled[h.home as usize], 1, "its new home counts it once");
        // With no farm to be had it leaves the valley.
        put(&mut w, 2, at(47, 28), home, 5, &[0.0, 0.0, 0.0]);
        w.productive.clear();
        w.expect_and_move(2);
        assert!(!w.households.contains_key(&2));
        assert!(w.farmer[at(47, 28) as usize].is_none());
        assert_eq!(w.departures, 1);
    }

    #[test]
    fn a_household_expecting_enough_stays() {
        let mut w = world(Quirks::NONE);
        put(&mut w, 1, at(48, 28), at(46, 27), 5, &[500.0, 400.0, 900.0]);
        w.households.get_mut(&1).unwrap().harvest = 0.0;
        w.expect_and_move(1);
        assert_eq!(w.households[&1].expected, 900.0);
        assert_eq!((w.households[&1].farm, w.moves), (at(48, 28), 0));
    }

    #[test]
    fn with_the_occupancy_leak_a_vacated_home_stays_closed_to_farming() {
        for leak in [false, true] {
            let mut q = Quirks::NONE;
            q.occupancy_leak = leak;
            let mut w = world(q);
            let old_home = at(49, 28);
            put(&mut w, 1, at(48, 27), old_home, 5, &[700.0, 0.0, 0.0]);
            w.water_d2 = vec![0; CELLS];
            w.productive = vec![at(51, 28)];
            w.expect_and_move(1);
            let new_home = w.households[&1].home;
            assert_ne!(new_home, old_home);
            assert_eq!(w.settled[old_home as usize], if leak { 2 } else { 0 });
            assert_eq!(w.settled[new_home as usize], if leak { 2 } else { 1 });
            w.productive = vec![old_home];
            assert_eq!(w.find_farm(at(40, 28), false).is_some(), !leak);
            w.remove(1);
            assert_eq!(w.settled[new_home as usize], u32::from(leak));
        }
    }

    /// A world where household 1 (age 20, farming (48, 28)) splits for sure
    /// and (51, 28) is free to farm.
    fn fertile(quirks: Quirks) -> AnasaziWorld {
        let mut w = world(quirks);
        w.config.fission_probability = 1.0;
        put(&mut w, 1, at(48, 28), at(46, 27), 20, &[900.0, 300.0, 30.0]);
        w.water_d2 = vec![0; CELLS];
        w.productive = vec![at(51, 28)];
        w
    }

    #[test]
    fn fission_gives_the_child_a_share_of_its_parents_corn() {
        let mut w = fertile(Quirks::NONE);
        w.fission(1);
        let child = &w.households[&2];
        assert_eq!((child.age, child.farm, w.births), (0, at(51, 28), 1));
        let total: f64 = w.households.values().map(Household::stock).sum();
        assert!((total - 1230.0).abs() < 1e-9, "corn is conserved: {total}");
        assert!((child.stock() - 0.33 * 1230.0).abs() < 1e-9);
        assert_eq!(
            child.expected,
            expected_harvest(&child.corn, 0.0),
            "a new household expects what anyone would: no harvest yet"
        );
    }

    #[test]
    fn with_fresh_endowments_the_child_gets_new_corn() {
        let mut q = Quirks::NONE;
        q.fission_fresh_endowment = true;
        let mut w = fertile(q);
        w.fission(1);
        assert!(
            (w.households[&1].stock() - 0.67 * 1230.0).abs() < 1e-9,
            "the parent loses its share"
        );
        let fresh = 0.33 / 0.67;
        for &kg in &w.households[&2].corn {
            assert!((fresh * 2000.0..=fresh * 2400.0).contains(&kg), "{kg}");
        }
    }

    #[test]
    fn fission_needs_an_eligible_age_and_a_farm() {
        for age in [16, 35] {
            let mut w = fertile(Quirks::NONE);
            w.households.get_mut(&1).unwrap().age = age;
            w.fission(1);
            assert_eq!(w.births, 0, "age {age}");
        }
        let mut w = fertile(Quirks::NONE);
        w.productive.clear();
        w.fission(1);
        assert_eq!(
            (w.births, w.households.len()),
            (0, 1),
            "no farm: the child does not establish"
        );
        assert_eq!(
            w.households[&1].stock(),
            1230.0,
            "and the parent keeps its corn"
        );
    }

    #[test]
    fn with_fission_needs_free_farm_no_split_is_even_tried_without_one() {
        // The quirk skips the draw: the RNG is left untouched. (A certain
        // split draws nothing, so the chance here is ½.)
        let next = |q: Quirks| {
            let mut w = fertile(q);
            w.config.fission_probability = 0.5;
            w.productive.clear();
            w.fission(1);
            w.rng.gen::<u64>()
        };
        let mut q = Quirks::NONE;
        q.fission_needs_free_farm = true;
        let untouched = {
            let mut w = fertile(q);
            w.rng.gen::<u64>()
        };
        assert_eq!(next(q), untouched);
        assert_ne!(next(Quirks::NONE), untouched);
    }

    #[test]
    fn initial_households_hold_one_draw_or_one_per_slot() {
        let make = |per_slot: bool| {
            let mut q = Quirks::NONE;
            q.initial_corn_per_slot = per_slot;
            q.no_farm_water_check = true;
            AnasaziWorld::new(AnasaziConfig::calibrated(q), 4).unwrap()
        };
        let total = make(false);
        assert!(!total.households.is_empty());
        for h in total.households.values() {
            assert_eq!(h.corn[1..], [0.0, 0.0]);
            assert!((2000.0..=2400.0).contains(&h.corn[0]));
            assert!(h.age < 29);
        }
        for h in make(true).households.values() {
            assert!(
                h.corn.iter().all(|kg| (2000.0..=2400.0).contains(kg)),
                "{:?}",
                h.corn
            );
            assert_eq!(h.expected, h.corn[0] + h.corn[1], "the oldest slot spoils");
        }
    }

    #[test]
    fn initial_farms_ignore_the_harvest_adjustment_only_with_the_quirk() {
        let make = |quirk: bool| {
            let mut q = Quirks::NONE;
            q.initial_eligibility_ignores_adjustment = quirk;
            let config = AnasaziConfig {
                harvest_adjustment: 0.1,
                ..AnasaziConfig::calibrated(q)
            };
            AnasaziWorld::new(config, 2).unwrap().households.len()
        };
        assert_eq!(make(false), 0, "nothing yields 800 kg at Ha 0.1");
        assert_eq!(make(true), 14);
    }

    #[test]
    fn soil_quality_uses_the_spatial_sd_unless_one_harvest_sd_is_used() {
        let make = |single: bool| {
            let mut q = Quirks::NONE;
            q.single_harvest_variance = single;
            let config = AnasaziConfig {
                spatial_sd: 0.0,
                annual_sd: 0.4,
                initial_households: 0,
                ..AnasaziConfig::calibrated(q)
            };
            AnasaziWorld::new(config, 3).unwrap()
        };
        assert!(make(false).quality.iter().all(|&q| q == 1.0));
        let spread = make(true);
        assert!(
            spread.quality.iter().any(|&q| q > 1.5) && spread.quality.iter().all(|&q| q >= 0.0)
        );
    }

    #[test]
    fn arable_uplands_follow_their_pdsi_unless_held_in_one_class() {
        // A year whose uplands series is not in the (−1, 1) class.
        let v = valley::valley();
        let year = (800..=1350)
            .find(|&y| pdsi_class(v.pdsi(PdsiSeries::Natural, y)) != 2)
            .unwrap();
        let config = |single: bool| {
            let mut q = Quirks::NONE;
            q.uplands_single_class = single;
            AnasaziConfig {
                start_year: year,
                initial_households: 0,
                ..AnasaziConfig::calibrated(q)
            }
        };
        let uplands = |w: &AnasaziWorld| w.zone_yield[zone_index(Zone::Uplands)];
        assert_eq!(uplands(&AnasaziWorld::new(config(true), 1).unwrap()), 547.0);
        assert_ne!(
            uplands(&AnasaziWorld::new(config(false), 1).unwrap()),
            547.0
        );
    }

    #[test]
    fn water_is_updated_at_the_end_of_each_year() {
        // 980 starts an alluvium period: the valley floor becomes water at
        // the end of the year 980, for the moves of 981.
        let config = AnasaziConfig {
            start_year: 979,
            initial_households: 0,
            ..AnasaziConfig::default()
        };
        let mut w = AnasaziWorld::new(config, 1).unwrap();
        let floor = at(48, 28);
        assert!(!w.water[floor as usize]);
        w.step();
        assert_eq!(w.year(), 980);
        assert!(w.water[floor as usize]);
        assert_eq!(w.water_d2[floor as usize], 0);
    }

    #[test]
    fn statistics_follow_the_historical_record() {
        let mut w = AnasaziWorld::new(AnasaziConfig::default(), 1).unwrap();
        let first = w.stats.latest().unwrap().clone();
        assert_eq!((first.tick, first.year, first.historical), (0, 800, 14));
        assert_eq!(first.fit, (f64::from(first.households) - 14.0).powi(2));
        assert_eq!(first.capacity as usize, w.productive.len());
        w.run(10);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.tick, s.year), (10, 810));
        let fits = w.stats.series("fit").unwrap();
        let (hh, hist) = (
            w.stats.series("households").unwrap(),
            w.stats.series("historical").unwrap(),
        );
        let sum: f64 = hh.iter().zip(&hist).map(|(a, b)| (a - b).powi(2)).sum();
        assert_eq!(fits.last(), Some(&sum));
        assert_eq!(s.households as usize, w.households.len());
    }

    #[test]
    fn the_run_stops_at_the_end_year() {
        let config = AnasaziConfig {
            start_year: 1340,
            ..AnasaziConfig::default()
        };
        let mut w = AnasaziWorld::new(config, 1).unwrap();
        w.run(100);
        assert_eq!((w.year(), w.tick), (1350, 10));
        assert!(Model::finished(&w));
        let print = Model::fingerprint(&w);
        w.step();
        assert_eq!((w.tick, Model::fingerprint(&w)), (10, print));
    }

    #[test]
    fn the_valley_renders_in_three_modes_and_inspects_cells() {
        let mut w = AnasaziWorld::new(AnasaziConfig::default(), 5).unwrap();
        w.run(3);
        let mut buf = Vec::new();
        for mode in ["occupation", "zones", "yield"] {
            Model::render(&w, mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), CELLS * 4);
        }
        assert!(Model::render(&w, "tribe", "", &mut buf).is_err());
        Model::render(&w, "zones", "", &mut buf).unwrap();
        let px = |buf: &[u8], c: u32| {
            let i = c as usize * 4;
            [buf[i], buf[i + 1], buf[i + 2]]
        };
        assert_eq!(px(&buf, at(48, 28)), zone_color(Zone::North));
        let h = w.households.values().next().unwrap().clone();
        Model::render(&w, "occupation", "", &mut buf).unwrap();
        assert_eq!((px(&buf, h.farm), px(&buf, h.home)), (FARM, HOME));
        let [x, y] = xy(h.farm);
        let seen = w.inspect(x, y).unwrap();
        assert_eq!(seen.site.farmed_by, Some(h.id));
        assert_eq!(seen.agent.as_ref().map(|a| a.id), Some(h.id));
        assert_eq!(
            seen.site.zone_yield * seen.site.quality * 0.56,
            seen.site.base_yield
        );
        assert_eq!(Model::locate(&w, h.id), Some((x, y)));
        let [hx, hy] = xy(h.home);
        assert!(w.inspect(hx, hy).unwrap().site.residents.contains(&h.id));
        assert!(w.inspect(80, 0).is_err());
    }

    #[test]
    fn overlays_list_water_settlements_and_links() {
        let w = AnasaziWorld::new(AnasaziConfig::default(), 5).unwrap();
        let water = w.water_xy();
        assert_eq!(water.len() / 2, w.water.iter().filter(|&&b| b).count());
        let settlements = w.settlements_xy();
        let people: u32 = settlements.chunks(3).map(|s| s[2]).sum();
        assert_eq!(people as usize, w.households.len());
        assert_eq!(w.links_xy().len(), 4 * w.households.len());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Anasazi(AnasaziConfig::default());
        crate::schema::check_schema(&crate::anasazi::schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }

    #[test]
    fn a_live_harvest_adjustment_changes_this_years_yields() {
        let mut w = AnasaziWorld::new(AnasaziConfig::default(), 1).unwrap();
        let before = w.productive.len();
        let next = AnasaziConfig {
            harvest_adjustment: 1.0,
            ..w.config.clone()
        };
        Model::set_config(&mut w, ModelConfig::Anasazi(next)).unwrap();
        assert!(w.productive.len() > before);
        let reset = AnasaziConfig {
            death_age: 30,
            ..w.config.clone()
        };
        let e = Model::set_config(&mut w, ModelConfig::Anasazi(reset)).unwrap_err();
        assert_eq!(e[0].field, "death_age");
    }
}
