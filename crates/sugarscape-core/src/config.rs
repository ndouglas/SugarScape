//! Every rule parameter and toggle. Rule names follow the book's notation:
//! G_α growback, S_{α,β,γ} seasons, P/D pollution, R_[a,b] replacement,
//! S sex, I inheritance, K culture, C_α combat.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::agent::Sex;

/// Most goods a world can hold (Appendix A's n-vectors). Site and agent
/// arrays have this many slots; only goods 0..n are used.
pub const MAX_GOODS: usize = 8;
/// Most pollutants (Appendix A's m-vectors); only 0..m are used.
pub const MAX_POLLUTANTS: usize = 4;

/// Inclusive integer range sampled uniformly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct URange {
    pub min: u32,
    pub max: u32,
}

impl URange {
    pub const fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> u32 {
        rng.gen_range(self.min..=self.max)
    }
}

/// How a good's copy of the book's two-peak map is turned (Decision 7): the
/// value shown at (x, y) is the base map's value at `source(x, y)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transform {
    #[serde(rename = "identity")]
    Identity,
    /// 90° clockwise: the north-east peak moves south-east.
    #[serde(rename = "rotate_90")]
    Rotate90,
    #[serde(rename = "rotate_180")]
    Rotate180,
    #[serde(rename = "rotate_270")]
    Rotate270,
    /// Left↔right: Chapter IV's spice map.
    #[serde(rename = "mirror_x")]
    MirrorX,
    /// North↔south.
    #[serde(rename = "mirror_y")]
    MirrorY,
    #[serde(rename = "transpose")]
    Transpose,
    #[serde(rename = "anti_transpose")]
    AntiTranspose,
}

impl Transform {
    pub const ALL: [Transform; 8] = [
        Transform::Identity,
        Transform::Rotate90,
        Transform::Rotate180,
        Transform::Rotate270,
        Transform::MirrorX,
        Transform::MirrorY,
        Transform::Transpose,
        Transform::AntiTranspose,
    ];

    /// The base-map cell shown at (x, y) on a w×h map (w = h for the
    /// two-peak map).
    pub fn source(self, x: usize, y: usize, w: usize, h: usize) -> (usize, usize) {
        match self {
            Transform::Identity => (x, y),
            Transform::Rotate90 => (y, h - 1 - x),
            Transform::Rotate180 => (w - 1 - x, h - 1 - y),
            Transform::Rotate270 => (w - 1 - y, x),
            Transform::MirrorX => (w - 1 - x, y),
            Transform::MirrorY => (x, h - 1 - y),
            Transform::Transpose => (y, x),
            Transform::AntiTranspose => (h - 1 - y, w - 1 - x),
        }
    }
}

/// One mountain of a `peaks` map: capacity ⌈height·(1 − d/radius)⌉ at torus
/// distance d from (x, y), never below 0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Peak {
    pub x: u32,
    pub y: u32,
    pub radius: f64,
    pub height: f64,
}

/// A good's capacity map.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Map {
    /// The book's 50×50 two-peak map, turned by `transform`.
    TwoPeaks {
        transform: Transform,
    },
    /// The highest of 1–16 linear peaks at each site.
    Peaks {
        peaks: Vec<Peak>,
    },
    Flat {
        capacity: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Placement {
    /// Uniformly random distinct sites.
    Random,
    /// Random sites inside a rectangle (Animation II-6's block of agents).
    Block {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// Blues in a `size`×`size` southwest block, Reds in the northeast block.
    Tribes { size: u32 },
}

/// G_α: grow back `rate` per tick up to capacity; `instant` is G_∞.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Growback {
    pub rate: f64,
    pub instant: bool,
}

/// S_{α,β,γ}: summer in the north first; flip every `period` (γ) ticks;
/// winter grows at α / `winter_divisor` (β) per tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seasons {
    pub enabled: bool,
    pub winter_divisor: u32,
    pub period: u32,
}

/// Colors of Chapter IV's goods (the renderer's `SUGAR` and `SPICE`).
pub const SUGAR_COLOR: &str = "#f2c14e";
pub const SPICE_COLOR: &str = "#e07a3f";

/// `#rrggbb` (hex digits in either case) as RGB.
pub fn parse_color(s: &str) -> Option<[u8; 3]> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// One commodity (Appendix A's n-vectors): its map, and the traits new agents
/// draw for it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Good {
    /// Display name (1–16 characters, unique).
    pub name: String,
    /// `#rrggbb`, used by layers, charts and the inspector.
    pub color: String,
    /// Capacity map (fixed at reset).
    pub map: Map,
    /// Per-tick burn drawn for new agents.
    pub metabolism: URange,
    /// Initial holding drawn for new agents.
    pub endowment: URange,
}

impl Good {
    /// Chapter II's sugar on the book's two-peak map.
    pub fn sugar() -> Self {
        Self {
            name: "sugar".into(),
            color: SUGAR_COLOR.into(),
            map: Map::TwoPeaks {
                transform: Transform::Identity,
            },
            metabolism: URange::new(1, 4),
            endowment: URange::new(5, 25),
        }
    }

    /// Chapter IV's spice on the mirrored two-peak map.
    pub fn spice() -> Self {
        Self {
            name: "spice".into(),
            color: SPICE_COLOR.into(),
            map: Map::TwoPeaks {
                transform: Transform::MirrorX,
            },
            ..Self::sugar()
        }
    }
}

/// One pollutant (Appendix B's rule P with n goods and m pollutants): it
/// forms from each good gathered (Πₖᵢ, `production`) and burned (Χₖᵢ,
/// `consumption`), and devalues the goods marked in `devalues` when agents
/// choose sites.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pollutant {
    pub name: String,
    pub production: Vec<f64>,
    pub consumption: Vec<f64>,
    pub devalues: Vec<bool>,
}

impl Pollutant {
    /// The book's pollutant on `n` goods: α = β = 1 for good 0 only, which
    /// is also the only good it devalues.
    pub fn book(n: usize) -> Self {
        let first = |v: f64| (0..n).map(|i| if i == 0 { v } else { 0.0 }).collect();
        Self {
            name: "pollution".into(),
            production: first(1.0),
            consumption: first(1.0),
            devalues: (0..n).map(|i| i == 0).collect(),
        }
    }
}

/// Rule P with m pollutants; D_α diffuses each separately.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pollution {
    pub enabled: bool,
    pub pollutants: Vec<Pollutant>,
}

/// D_α: every `every` ticks each site's pollution becomes its neighbors' mean.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diffusion {
    pub enabled: bool,
    pub every: u32,
}

/// Death from old age; `max_age` is also R_[a,b]'s [a, b].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lifespan {
    pub enabled: bool,
    pub max_age: URange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Toggle {
    pub enabled: bool,
}

/// Most tag groups a config may list.
pub const MAX_GROUPS: usize = 8;
/// The book's tribe colors (the renderer's `BLUE` and `RED`) and the third
/// group's green (Chapter III, note 20).
pub const BLUE_COLOR: &str = "#3d7eff";
pub const RED_COLOR: &str = "#ff4d4d";
pub const GREEN_COLOR: &str = "#3dd66b";

/// A tag group (tribe): the agents whose tag strings hold a number of zeros
/// in `zeros`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Display name (1–16 characters, unique).
    pub name: String,
    /// `#rrggbb`, used by the Tribe color mode and the group-share chart.
    pub color: String,
    pub zeros: URange,
}

impl Group {
    pub fn new(name: &str, color: &str, min: u32, max: u32) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
            zeros: URange::new(min, max),
        }
    }
}

/// Chapter III's two tribes on `tag_length`-bit tags: Blue when zeros
/// outnumber ones (⌈(L+1)/2⌉..=L zeros), otherwise Red. Group 0 is Blue.
pub fn default_groups(tag_length: u32) -> Vec<Group> {
    // ⌊L/2⌋ + 1 = ⌈(L+1)/2⌉: the fewest zeros that outnumber the ones.
    let blue_from = tag_length / 2 + 1;
    vec![
        Group::new("Blue", BLUE_COLOR, blue_from, tag_length),
        Group::new("Red", RED_COLOR, 0, blue_from - 1),
    ]
}

/// Chapter III note 20's three groups — Blue 0–3, Green 4–7, Red 8–11 zeros
/// on 11-bit tags — generalized to `tag_length` ≥ 2 by cutting 0..=L into
/// thirds at ⌊k(L+1)/3⌋.
pub fn three_tribes(tag_length: u32) -> Vec<Group> {
    assert!(tag_length >= 2, "three tribes need tags of at least 2 bits");
    let cut = |k: u32| k * (tag_length + 1) / 3;
    vec![
        Group::new("Blue", BLUE_COLOR, 0, cut(1) - 1),
        Group::new("Green", GREEN_COLOR, cut(1), cut(2) - 1),
        Group::new("Red", RED_COLOR, cut(2), tag_length),
    ]
}

/// The first group whose range holds `zeros`; 0 if none does (a validated
/// config's groups cover every count).
pub fn group_of(groups: &[Group], zeros: u32) -> usize {
    groups
        .iter()
        .position(|g| (g.zeros.min..=g.zeros.max).contains(&zeros))
        .unwrap_or(0)
}

/// K: cultural transmission, plus the tag groups that combat, the Tribe
/// color mode and the group-share statistics read (whether or not K is on).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CultureRule {
    pub enabled: bool,
    /// An agent belongs to the first group whose `zeros` holds its tags'
    /// number of zeros. When JSON omits it, `Config::from_value` fills in
    /// `default_groups(tag_length)` (Decision 2).
    #[serde(default)]
    pub groups: Vec<Group>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SexRule {
    pub enabled: bool,
    pub fertility_onset: URange,
    pub female_end: URange,
    pub male_end: URange,
}

impl SexRule {
    pub fn end_for(&self, sex: Sex) -> URange {
        match sex {
            Sex::Female => self.female_end,
            Sex::Male => self.male_end,
        }
    }
}

/// C_α: reward is min(α, victim's sugar); `unlimited` is C_∞.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatRule {
    pub enabled: bool,
    pub unlimited: bool,
    pub reward: f64,
}

/// L_{d,r}: sugar loans of `duration` (d) ticks at `rate` (r) percent simple
/// interest per tick.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreditRule {
    pub enabled: bool,
    pub duration: u32,
    pub rate: f64,
}

/// Book eq. 6: agents value holdings as if `φ` periods of metabolism were
/// already spent; `range` is φ's initial distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Foresight {
    pub enabled: bool,
    pub range: URange,
}

/// A novel disease appearing mid-run (Chapter V's McNeill scenario): at the
/// start of the tick when `World::tick == tick`, a brand-new random disease
/// infects `agents` random living agents (all of them if there are fewer).
/// The new disease's length is drawn from `length` if given, else from
/// `disease.length` — a genuinely novel outbreak can be given a length the
/// existing immune population is unlikely to already contain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outbreak {
    pub tick: u64,
    pub agents: u32,
    #[serde(default)]
    pub length: Option<URange>,
}

/// Rule E (Chapter V, Appendix B): immune response and disease transmission.
/// The defaults are Animation V-1's.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DiseaseRule {
    pub enabled: bool,
    /// Size of the initial master list of diseases.
    pub count: u32,
    /// Disease string lengths.
    pub length: URange,
    /// Distinct random diseases given to each new (not newborn) agent.
    pub initial: u32,
    /// Immune string length (1–64).
    pub immune_length: u32,
    /// Extra metabolism of each good per carried disease.
    pub fee: f64,
    /// Immune bits flipped per carried disease per tick ("medicine").
    pub flips_per_tick: u32,
    /// Per-bit mutation probability of a child's immune genome.
    pub genome_mutation: f64,
    /// Probability that a transmitted disease mutates one random bit.
    pub disease_mutation: f64,
    pub outbreaks: Vec<Outbreak>,
}

impl DiseaseRule {
    /// The per-disease metabolic fee in force: `fee` while disease is on, else 0.
    pub fn active_fee(&self) -> f64 {
        if self.enabled {
            self.fee
        } else {
            0.0
        }
    }
}

impl Default for DiseaseRule {
    fn default() -> Self {
        Self {
            enabled: false,
            count: 10,
            length: URange::new(1, 10),
            initial: 4,
            immune_length: 50,
            fee: 1.0,
            flips_per_tick: 1,
            genome_mutation: 0.0,
            disease_mutation: 0.0,
            outbreaks: Vec::new(),
        }
    }
}

/// Fields a schedule may not change (they shape the world's storage or setup).
pub const STRUCTURAL_FIELDS: [&str; 5] =
    ["width", "height", "tag_length", "population", "placement"];

/// Disease paths a schedule may not set (they fix the disease list and
/// immune strings).
pub const RESET_ONLY_PATHS: [&str; 7] = [
    "disease",
    "disease.enabled",
    "disease.count",
    "disease.length",
    "disease.length.min",
    "disease.length.max",
    "disease.immune_length",
];

/// Whether a schedule may not set `path` (Decision 5): the goods list, whole
/// goods and their maps, the pollutant list, the groups list, whole groups
/// and their ranges, and `RESET_ONLY_PATHS`. A good's name, color and trait
/// ranges, a pollutant's name and coefficients, and a group's name and color
/// may be scheduled.
fn reset_only(path: &str) -> bool {
    let parts: Vec<&str> = path.split('.').collect();
    matches!(
        parts.as_slice(),
        ["goods"]
            | ["goods", _]
            | ["goods", _, "map", ..]
            | ["pollution"]
            | ["pollution", "pollutants"]
            | ["culture"]
            | ["culture", "groups"]
            | ["culture", "groups", _]
            | ["culture", "groups", _, "zeros", ..]
    ) || RESET_ONLY_PATHS.contains(&path)
}

/// Whether `parts` is `pollution.pollutants.K.{production,consumption,devalues}…`.
fn is_coefficients(parts: &[String]) -> bool {
    parts.len() >= 4
        && parts[0] == "pollution"
        && parts[1] == "pollutants"
        && matches!(parts[3].as_str(), "production" | "consumption" | "devalues")
}

/// Whether `parts` sets a whole pollutant or a whole coefficient array, whose
/// length is the number of goods.
fn sets_good_columns(parts: &[String]) -> bool {
    (parts.len() == 3 && parts[0] == "pollution" && parts[1] == "pollutants")
        || (parts.len() == 4 && is_coefficients(parts))
}

/// For a removed index `removed`: false if `segment` names it, otherwise
/// true, moving higher indices down one.
fn shift_index(segment: &mut String, removed: usize) -> bool {
    match segment.parse::<usize>() {
        Ok(j) if j == removed => false,
        Ok(j) if j > removed => {
            *segment = (j - 1).to_string();
            true
        }
        _ => true,
    }
}

/// At the start of the tick when `World::tick == tick`, set each dotted config
/// path in `set` to its value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledChange {
    pub tick: u64,
    pub set: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub population: u32,
    pub placement: Placement,
    pub vision: URange,
    pub tag_length: u32,
    /// Goods 0..n (1–8); good 0 always exists.
    pub goods: Vec<Good>,
    pub growback: Growback,
    pub seasons: Seasons,
    pub pollution: Pollution,
    pub diffusion: Diffusion,
    pub lifespan: Lifespan,
    pub replacement: Toggle,
    pub sex: SexRule,
    pub inheritance: Toggle,
    pub culture: CultureRule,
    pub combat: CombatRule,
    pub trade: Toggle,
    pub credit: CreditRule,
    pub foresight: Foresight,
    pub disease: DiseaseRule,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for Config {
    /// ({G₁}, {M}) on the two-peak map with Chapter II's agent distributions;
    /// every other rule's parameters preset to the book's values but off.
    fn default() -> Self {
        Self {
            width: 50,
            height: 50,
            population: 400,
            placement: Placement::Random,
            vision: URange::new(1, 6),
            tag_length: 11,
            goods: vec![Good::sugar()],
            growback: Growback {
                rate: 1.0,
                instant: false,
            },
            seasons: Seasons {
                enabled: false,
                winter_divisor: 8,
                period: 50,
            },
            pollution: Pollution {
                enabled: false,
                pollutants: vec![Pollutant::book(1)],
            },
            diffusion: Diffusion {
                enabled: false,
                every: 1,
            },
            lifespan: Lifespan {
                enabled: false,
                max_age: URange::new(60, 100),
            },
            replacement: Toggle { enabled: false },
            sex: SexRule {
                enabled: false,
                fertility_onset: URange::new(12, 15),
                female_end: URange::new(40, 50),
                male_end: URange::new(50, 60),
            },
            inheritance: Toggle { enabled: false },
            culture: CultureRule {
                enabled: false,
                groups: default_groups(11),
            },
            combat: CombatRule {
                enabled: false,
                unlimited: true,
                reward: 2.0,
            },
            trade: Toggle { enabled: false },
            credit: CreditRule {
                enabled: false,
                duration: 10,
                rate: 10.0,
            },
            foresight: Foresight {
                enabled: false,
                range: URange::new(0, 10),
            },
            disease: DiseaseRule::default(),
            schedule: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Default)]
struct Errors(Vec<FieldError>);

impl Errors {
    fn check(&mut self, ok: bool, field: &str, message: impl Into<String>) {
        if !ok {
            self.0.push(FieldError::new(field, message));
        }
    }

    fn range(&mut self, r: URange, field: &str) {
        self.check(r.min <= r.max, field, "min must be ≤ max");
    }

    fn non_negative(&mut self, v: f64, field: &str) {
        self.check(v.is_finite() && v >= 0.0, field, "must be a number ≥ 0");
    }

    fn probability(&mut self, v: f64, field: &str) {
        self.check(
            (0.0..=1.0).contains(&v),
            field,
            "must be a probability between 0 and 1",
        );
    }

    fn name(&mut self, name: &str, field: &str) {
        let len = name.chars().count();
        self.check((1..=16).contains(&len), field, "must be 1–16 characters");
    }

    fn finish(self) -> Result<(), Vec<FieldError>> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.0)
        }
    }
}

impl Config {
    pub fn from_json(json: &str) -> Result<Self, Vec<FieldError>> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| vec![FieldError::new("config", e.to_string())])?;
        let config = Self::from_value(value).map_err(|e| vec![e])?;
        config.validate()?;
        Ok(config)
    }

    /// Reads either config shape (Decision 2): a JSON object without a `goods`
    /// key is a pre-N-goods config and is converted. A new-shape config
    /// without a `pollution` block gets the book's pollutant on all its
    /// goods. A new-shape config without `culture.groups` gets the two book
    /// tribes for its `tag_length`.
    pub fn from_value(value: serde_json::Value) -> Result<Self, FieldError> {
        let Some(object) = value.as_object() else {
            return serde_json::from_value(value)
                .map_err(|e| FieldError::new("config", e.to_string()));
        };
        if !object.contains_key("goods") {
            return crate::legacy::convert(value);
        }
        let defaulted_pollution = !object.contains_key("pollution");
        let defaulted_groups = object
            .get("culture")
            .is_none_or(|c| c.get("groups").is_none());
        let mut config: Config =
            serde_json::from_value(value).map_err(|e| FieldError::new("config", e.to_string()))?;
        if defaulted_pollution {
            config.pollution.pollutants = vec![Pollutant::book(config.goods.len())];
        }
        if defaulted_groups {
            config.culture.groups = default_groups(config.tag_length);
        }
        Ok(config)
    }

    /// Appends `good`; every pollutant gets zero coefficients for it and does
    /// not devalue it. Scheduled changes that set a whole pollutant or a
    /// whole coefficient array (now one short) are dropped.
    pub fn add_good(&mut self, good: Good) {
        self.goods.push(good);
        for p in &mut self.pollution.pollutants {
            p.production.push(0.0);
            p.consumption.push(0.0);
            p.devalues.push(false);
        }
        self.retarget_schedule(|parts| !sets_good_columns(parts));
    }

    /// Removes good `i` and its pollutant coefficients. Scheduled changes to
    /// good `i`, its coefficients, or a whole pollutant or coefficient array
    /// are dropped; later goods' paths move down one index.
    pub fn remove_good(&mut self, i: usize) {
        self.goods.remove(i);
        for p in &mut self.pollution.pollutants {
            p.production.remove(i);
            p.consumption.remove(i);
            p.devalues.remove(i);
        }
        self.retarget_schedule(|parts| match parts.len() {
            n if n >= 2 && parts[0] == "goods" => shift_index(&mut parts[1], i),
            _ if sets_good_columns(parts) => false,
            n if n >= 5 && is_coefficients(parts) => shift_index(&mut parts[4], i),
            _ => true,
        });
    }

    /// Removes pollutant `k`. Scheduled changes to it are dropped; later
    /// pollutants' paths move down one index.
    pub fn remove_pollutant(&mut self, k: usize) {
        self.pollution.pollutants.remove(k);
        self.retarget_schedule(|parts| {
            if parts.len() >= 3 && parts[0] == "pollution" && parts[1] == "pollutants" {
                shift_index(&mut parts[2], k)
            } else {
                true
            }
        });
    }

    /// Rewrites every scheduled path's segments with `keep`, dropping the
    /// paths it rejects and then the changes left with none.
    fn retarget_schedule(&mut self, keep: impl Fn(&mut [String]) -> bool) {
        for change in &mut self.schedule {
            change.set = std::mem::take(&mut change.set)
                .into_iter()
                .filter_map(|(path, value)| {
                    let mut parts: Vec<String> = path.split('.').map(String::from).collect();
                    keep(&mut parts).then(|| (parts.join("."), value))
                })
                .collect();
        }
        self.schedule.retain(|c| !c.set.is_empty());
    }

    fn check_map(&self, map: &Map, field: &str, e: &mut Errors) {
        match map {
            Map::TwoPeaks { .. } => e.check(
                self.width == 50 && self.height == 50,
                field,
                "the two-peak map is 50×50; set width and height to 50",
            ),
            Map::Peaks { peaks } => {
                e.check(
                    (1..=16).contains(&peaks.len()),
                    field,
                    "needs 1 to 16 peaks",
                );
                e.check(
                    peaks.iter().all(|p| p.x < self.width && p.y < self.height),
                    field,
                    "peak centers must lie on the grid",
                );
                e.check(
                    peaks.iter().all(|p| p.radius.is_finite() && p.radius > 0.0),
                    field,
                    "peak radius must be > 0",
                );
                e.check(
                    peaks
                        .iter()
                        .all(|p| p.height.is_finite() && (0.0..=10.0).contains(&p.height)),
                    field,
                    "peak height must be between 0 and 10",
                );
            }
            Map::Flat { capacity } => e.non_negative(*capacity, &format!("{field}.capacity")),
        }
    }

    /// `culture.groups` (Decision 3): 1–8 groups with unique names and
    /// `#rrggbb` colors whose zero ranges tile 0..=tag_length exactly once.
    fn check_groups(&self, e: &mut Errors) {
        let groups = &self.culture.groups;
        let l = self.tag_length;
        e.check(
            (1..=MAX_GROUPS).contains(&groups.len()),
            "culture.groups",
            format!("must list 1 to {MAX_GROUPS} groups"),
        );
        let mut names = BTreeSet::new();
        for (k, g) in groups.iter().enumerate() {
            let field = |f: &str| format!("culture.groups.{k}.{f}");
            e.name(&g.name, &field("name"));
            e.check(
                names.insert(g.name.as_str()),
                &field("name"),
                "another group has this name",
            );
            e.check(
                parse_color(&g.color).is_some(),
                &field("color"),
                "must be a #rrggbb color",
            );
            e.range(g.zeros, &field("zeros"));
            e.check(
                g.zeros.max <= l,
                &field("zeros"),
                format!("must lie within 0–{l} (the tag length)"),
            );
        }
        if !(1..=64).contains(&l) {
            return; // `tag_length` reports itself
        }
        let mut hits = vec![0usize; l as usize + 1];
        for g in groups {
            for z in g.zeros.min..=g.zeros.max.min(l) {
                hits[z as usize] += 1;
            }
        }
        let counts = |want: fn(usize) -> bool| {
            hits.iter()
                .enumerate()
                .filter(|&(_, &n)| want(n))
                .map(|(z, _)| z.to_string())
                .collect::<Vec<_>>()
        };
        let (gaps, overlaps) = (counts(|n| n == 0), counts(|n| n > 1));
        e.check(
            gaps.is_empty(),
            "culture.groups",
            format!("no group holds tags with {} zeros", gaps.join(", ")),
        );
        e.check(
            overlaps.is_empty(),
            "culture.groups",
            format!(
                "tags with {} zeros fall in more than one group",
                overlaps.join(", ")
            ),
        );
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        self.validate_with_schedule_from(0)
    }

    /// Like `validate`, but checks only the schedule entries with
    /// `tick >= from` — on a running world, earlier entries already fired and
    /// their effects are part of this config.
    pub fn validate_with_schedule_from(&self, from: u64) -> Result<(), Vec<FieldError>> {
        self.validate_fields()?;
        self.validate_schedule(from)
    }

    /// Everything except the schedule (renamed from the old `validate` body).
    fn validate_fields(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Errors::default();
        e.check(
            (5..=500).contains(&self.width),
            "width",
            "must be between 5 and 500",
        );
        e.check(
            (5..=500).contains(&self.height),
            "height",
            "must be between 5 and 500",
        );
        let pop = u64::from(self.population);
        match self.placement {
            Placement::Random => e.check(
                pop <= u64::from(self.width) * u64::from(self.height),
                "population",
                "cannot exceed the number of sites",
            ),
            Placement::Block {
                x,
                y,
                width,
                height,
            } => {
                e.check(
                    width > 0
                        && height > 0
                        && x.saturating_add(width) <= self.width
                        && y.saturating_add(height) <= self.height,
                    "placement",
                    "block must fit inside the grid",
                );
                e.check(
                    pop <= u64::from(width) * u64::from(height),
                    "population",
                    "cannot exceed the block's area",
                );
            }
            Placement::Tribes { size } => {
                e.check(
                    size > 0 && size.saturating_mul(2) <= self.width.min(self.height),
                    "placement.size",
                    "the two corner blocks must fit without overlapping",
                );
                e.check(
                    pop <= 2 * u64::from(size) * u64::from(size),
                    "population",
                    "cannot exceed the two blocks' area",
                );
            }
        }
        let max_vision = self.width.min(self.height) / 2;
        e.range(self.vision, "vision");
        e.check(self.vision.min >= 1, "vision.min", "must be ≥ 1");
        e.check(
            self.vision.max <= max_vision,
            "vision.max",
            format!("must be ≤ {max_vision} (half the grid)"),
        );
        let n = self.goods.len();
        e.check(
            (1..=MAX_GOODS).contains(&n),
            "goods",
            format!("must list 1 to {MAX_GOODS} goods"),
        );
        let mut names = BTreeSet::new();
        for (i, g) in self.goods.iter().enumerate() {
            let field = |f: &str| format!("goods.{i}.{f}");
            e.name(&g.name, &field("name"));
            e.check(
                names.insert(g.name.as_str()),
                &field("name"),
                "another good has this name",
            );
            e.check(
                parse_color(&g.color).is_some(),
                &field("color"),
                "must be a #rrggbb color",
            );
            e.range(g.metabolism, &field("metabolism"));
            e.range(g.endowment, &field("endowment"));
            self.check_map(&g.map, &field("map"), &mut e);
        }
        let m = self.pollution.pollutants.len();
        e.check(
            (1..=MAX_POLLUTANTS).contains(&m),
            "pollution.pollutants",
            format!("must list 1 to {MAX_POLLUTANTS} pollutants"),
        );
        let mut names = BTreeSet::new();
        for (k, p) in self.pollution.pollutants.iter().enumerate() {
            let field = |f: &str| format!("pollution.pollutants.{k}.{f}");
            e.name(&p.name, &field("name"));
            e.check(
                names.insert(p.name.as_str()),
                &field("name"),
                "another pollutant has this name",
            );
            for (key, v) in [
                ("production", &p.production),
                ("consumption", &p.consumption),
            ] {
                e.check(v.len() == n, &field(key), "needs one coefficient per good");
                e.check(
                    v.iter().all(|x| x.is_finite() && *x >= 0.0),
                    &field(key),
                    "coefficients must be numbers ≥ 0",
                );
            }
            e.check(
                p.devalues.len() == n,
                &field("devalues"),
                "needs one entry per good",
            );
        }
        e.check(
            (1..=64).contains(&self.tag_length),
            "tag_length",
            "must be between 1 and 64",
        );
        self.check_groups(&mut e);
        e.check(
            self.growback.rate.is_finite() && self.growback.rate > 0.0,
            "growback.rate",
            "must be a number > 0",
        );
        e.check(
            self.seasons.winter_divisor >= 1,
            "seasons.winter_divisor",
            "must be ≥ 1",
        );
        e.check(self.seasons.period >= 1, "seasons.period", "must be ≥ 1");
        e.check(self.diffusion.every >= 1, "diffusion.every", "must be ≥ 1");
        e.range(self.lifespan.max_age, "lifespan.max_age");
        e.range(self.sex.fertility_onset, "sex.fertility_onset");
        e.range(self.sex.female_end, "sex.female_end");
        e.range(self.sex.male_end, "sex.male_end");
        e.check(
            !(self.replacement.enabled && self.sex.enabled),
            "replacement.enabled",
            "replacement (R) and sex (S) are mutually exclusive",
        );
        e.check(
            !self.replacement.enabled || self.lifespan.enabled,
            "replacement.enabled",
            "replacement R[a,b] needs lifespan on (it supplies [a,b])",
        );
        e.non_negative(self.combat.reward, "combat.reward");
        e.range(self.foresight.range, "foresight.range");
        e.check(
            !self.credit.enabled || self.sex.enabled,
            "credit.enabled",
            "credit (L) needs sex (S) on",
        );
        e.check(
            !self.trade.enabled || n >= 2,
            "trade.enabled",
            "trade (T) needs at least two goods",
        );
        e.check(
            !self.foresight.enabled || n >= 2,
            "foresight.enabled",
            "foresight needs at least two goods",
        );
        e.check(
            !self.combat.enabled || n == 1,
            "combat.enabled",
            "combat (C) needs exactly one good",
        );
        e.check(self.credit.duration >= 1, "credit.duration", "must be ≥ 1");
        e.non_negative(self.credit.rate, "credit.rate");
        let d = &self.disease;
        e.check(
            (1..=64).contains(&d.immune_length),
            "disease.immune_length",
            "must be between 1 and 64",
        );
        e.range(d.length, "disease.length");
        e.check(
            d.length.min >= 1,
            "disease.length",
            "diseases are at least 1 bit long",
        );
        e.check(
            d.length.max < d.immune_length,
            "disease.length",
            "diseases must be shorter than the immune string",
        );
        e.check(
            (1..=1000).contains(&d.count),
            "disease.count",
            "must be between 1 and 1000",
        );
        e.check(
            d.initial <= d.count,
            "disease.initial",
            "cannot exceed the number of diseases",
        );
        e.non_negative(d.fee, "disease.fee");
        e.check(
            d.flips_per_tick >= 1,
            "disease.flips_per_tick",
            "must be ≥ 1",
        );
        e.probability(d.genome_mutation, "disease.genome_mutation");
        e.probability(d.disease_mutation, "disease.disease_mutation");
        e.check(
            d.outbreaks.iter().all(|o| o.tick >= 1 && o.agents >= 1),
            "disease.outbreaks",
            "each outbreak needs tick ≥ 1 and at least 1 agent",
        );
        e.check(
            d.outbreaks.iter().all(|o| match o.length {
                Some(l) => l.min >= 1 && l.max < d.immune_length && l.min <= l.max,
                None => true,
            }),
            "disease.outbreaks",
            "an outbreak's length override must be 1 ≤ min ≤ max < immune_length",
        );
        e.finish()
    }

    fn validate_schedule(&self, from: u64) -> Result<(), Vec<FieldError>> {
        let mut entries: Vec<&ScheduledChange> =
            self.schedule.iter().filter(|c| c.tick >= from).collect();
        entries.sort_by_key(|c| c.tick);
        let mut patched = self.clone();
        for change in entries {
            if change.tick == 0 {
                return Err(vec![FieldError::new(
                    "schedule",
                    "scheduled ticks start at 1",
                )]);
            }
            patched = patched.apply_change(change).map_err(|e| vec![e])?;
        }
        Ok(())
    }

    /// A copy with one dotted `path` set to `value`.
    pub fn with_path(&self, path: &str, value: &serde_json::Value) -> Result<Config, FieldError> {
        let mut json = serde_json::to_value(self).expect("config serializes");
        let unknown = || FieldError::new("schedule", format!("unknown field {path}"));
        let mut slot = &mut json;
        for key in path.split('.') {
            slot = match slot {
                serde_json::Value::Array(items) => {
                    key.parse::<usize>().ok().and_then(|i| items.get_mut(i))
                }
                other => other.get_mut(key),
            }
            .ok_or_else(unknown)?;
        }
        *slot = value.clone();
        serde_json::from_value(json)
            .map_err(|e| FieldError::new("schedule", format!("{path}: {e}")))
    }

    /// This config with every path in `change` set, checked for validity
    /// (excluding the schedule itself).
    pub fn apply_change(&self, change: &ScheduledChange) -> Result<Config, FieldError> {
        let mut next = self.clone();
        for (path, value) in &change.set {
            let root = path.split('.').next().unwrap_or_default();
            if STRUCTURAL_FIELDS.contains(&root) || reset_only(path) {
                return Err(FieldError::new(
                    "schedule",
                    format!("{path} changes only on reset"),
                ));
            }
            if root == "schedule" {
                return Err(FieldError::new(
                    "schedule",
                    format!("{path}: the schedule cannot change itself"),
                ));
            }
            if path == "disease.outbreaks" || path.starts_with("disease.outbreaks.") {
                return Err(FieldError::new(
                    "schedule",
                    format!("{path}: outbreaks are their own schedule"),
                ));
            }
            next = next.with_path(path, value)?;
        }
        next.validate_fields().map_err(|errs| {
            let e = &errs[0];
            FieldError::new(
                "schedule",
                format!("at t={}: {}: {}", change.tick, e.field, e.message),
            )
        })?;
        Ok(next)
    }

    /// Fields that cannot change on a running world (they shape its storage,
    /// or — for spice and disease — every agent's traits and the disease list).
    pub fn structural_changes(&self, next: &Config) -> Vec<FieldError> {
        let mut out = Vec::new();
        let msg = "changes only on reset";
        if self.width != next.width {
            out.push(FieldError::new("width", msg));
        }
        if self.height != next.height {
            out.push(FieldError::new("height", msg));
        }
        if self.tag_length != next.tag_length {
            out.push(FieldError::new("tag_length", msg));
        }
        let ranges = |c: &Config| c.culture.groups.iter().map(|g| g.zeros).collect::<Vec<_>>();
        if ranges(self) != ranges(next) {
            out.push(FieldError::new("culture.groups", msg));
        }
        if self.goods.len() != next.goods.len() {
            out.push(FieldError::new("goods", msg));
        } else {
            for (i, (a, b)) in self.goods.iter().zip(&next.goods).enumerate() {
                if a.map != b.map {
                    out.push(FieldError::new(format!("goods.{i}.map"), msg));
                }
            }
        }
        if self.pollution.pollutants.len() != next.pollution.pollutants.len() {
            out.push(FieldError::new("pollution.pollutants", msg));
        }
        let (a, b) = (&self.disease, &next.disease);
        if a.enabled != b.enabled {
            out.push(FieldError::new("disease.enabled", msg));
        }
        if a.count != b.count {
            out.push(FieldError::new("disease.count", msg));
        }
        if a.length != b.length {
            out.push(FieldError::new("disease.length", msg));
        }
        if a.immune_length != b.immune_length {
            out.push(FieldError::new("disease.immune_length", msg));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(r: Result<(), Vec<FieldError>>) -> Vec<String> {
        r.err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn default_is_the_books_chapter_two_setup_and_valid() {
        let c = Config::default();
        assert_eq!((c.width, c.height, c.population), (50, 50, 400));
        assert_eq!(c.vision, URange::new(1, 6));
        assert_eq!(c.goods[0].metabolism, URange::new(1, 4));
        assert_eq!(c.goods[0].endowment, URange::new(5, 25));
        assert_eq!(c.tag_length, 11);
        assert_eq!(c.growback.rate, 1.0);
        c.validate().unwrap();
    }

    #[test]
    fn rejects_inverted_ranges_with_field_names() {
        let mut c = Config::default();
        c.goods[0].metabolism = URange::new(4, 1);
        assert_eq!(fields(c.validate()), vec!["goods.0.metabolism"]);
    }

    #[test]
    fn two_peak_map_requires_50_by_50() {
        let c = Config {
            width: 40,
            ..Default::default()
        };
        assert!(fields(c.validate()).contains(&"goods.0.map".to_string()));
    }

    #[test]
    fn width_height_bounds_are_5_to_500() {
        let c_too_small = Config {
            goods: flat_goods(1.0),
            width: 4,
            ..Default::default()
        };
        assert!(fields(c_too_small.validate()).contains(&"width".to_string()));
        let c_valid = Config {
            goods: flat_goods(1.0),
            width: 5,
            height: 5,
            population: 10,
            vision: URange::new(1, 2),
            ..Default::default()
        };
        c_valid.validate().unwrap();
    }

    #[test]
    fn vision_cannot_exceed_half_the_grid() {
        let mut c = Config::default();
        c.vision.max = 26;
        assert!(fields(c.validate()).contains(&"vision.max".to_string()));
    }

    #[test]
    fn replacement_excludes_sex_and_needs_lifespan() {
        let mut c = Config::default();
        c.replacement.enabled = true;
        assert!(fields(c.validate()).contains(&"replacement.enabled".to_string()));
        c.lifespan.enabled = true;
        c.validate().unwrap();
        c.sex.enabled = true;
        assert!(fields(c.validate()).contains(&"replacement.enabled".to_string()));
    }

    #[test]
    fn tribes_placement_must_fit() {
        let c = Config {
            placement: Placement::Tribes { size: 30 },
            ..Default::default()
        };
        assert!(fields(c.validate()).contains(&"placement.size".to_string()));
    }

    #[test]
    fn json_round_trips_and_missing_fields_default() {
        let c = Config::default();
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(Config::from_json(&json).unwrap(), c);
        let partial = Config::from_json(r#"{"population": 100}"#).unwrap();
        assert_eq!(partial.population, 100);
        assert_eq!(partial.width, 50);
        assert!(json.contains(r#""map":{"kind":"two_peaks","transform":"identity"}"#));
        assert!(!json.contains("landscape") && !json.contains("spice"));
    }

    #[test]
    fn bad_json_is_a_config_field_error() {
        let errs = Config::from_json("{not json").unwrap_err();
        assert_eq!(errs[0].field, "config");
    }

    #[test]
    fn structural_changes_are_reported() {
        let a = Config::default();
        let mut b = a.clone();
        b.sex.enabled = true;
        assert!(a.structural_changes(&b).is_empty());
        b.tag_length = 5;
        assert_eq!(a.structural_changes(&b)[0].field, "tag_length");
    }

    #[test]
    fn chapter_four_defaults_are_off_and_old_json_still_loads() {
        let c = Config::default();
        assert!(!c.trade.enabled && !c.credit.enabled && !c.foresight.enabled);
        assert_eq!(c.goods.len(), 1);
        assert_eq!((c.credit.duration, c.credit.rate), (10, 10.0));
        assert_eq!(c.foresight.range, URange::new(0, 10));
        let old = r#"{"pollution":{"enabled":true,"production":1.0,"consumption":1.0}}"#;
        let loaded = Config::from_json(old).unwrap();
        assert!(loaded.pollution.enabled);
        assert_eq!(loaded.pollution.pollutants[0].devalues, vec![true]);
    }

    #[test]
    fn chapter_four_rule_dependencies_are_validated() {
        let with = |f: fn(&mut Config)| {
            let mut c = Config::default();
            f(&mut c);
            fields(c.validate())
        };
        assert!(with(|c| c.credit.enabled = true).contains(&"credit.enabled".to_string()));
        assert!(with(|c| c.credit.duration = 0).contains(&"credit.duration".to_string()));
        assert!(with(|c| c.credit.rate = -1.0).contains(&"credit.rate".to_string()));
    }

    fn change(tick: u64, path: &str, value: serde_json::Value) -> ScheduledChange {
        ScheduledChange {
            tick,
            set: [(path.to_string(), value)].into_iter().collect(),
        }
    }

    #[test]
    fn with_path_sets_nested_fields_and_rejects_unknown_ones() {
        let c = Config::default();
        let on = c
            .with_path("pollution.enabled", &serde_json::json!(true))
            .unwrap();
        assert!(on.pollution.enabled);
        assert_eq!(
            c.with_path("pollution.nope", &serde_json::json!(1))
                .unwrap_err()
                .field,
            "schedule"
        );
        assert_eq!(
            c.with_path("pollution.enabled", &serde_json::json!("yes"))
                .unwrap_err()
                .field,
            "schedule"
        );
    }

    #[test]
    fn schedule_entries_are_validated() {
        let mut c = Config {
            schedule: vec![change(50, "pollution.enabled", serde_json::json!(true))],
            ..Default::default()
        };
        c.validate().unwrap();
        c.schedule = vec![change(0, "pollution.enabled", serde_json::json!(true))];
        assert_eq!(fields(c.validate()), vec!["schedule"]);
        c.schedule = vec![change(5, "width", serde_json::json!(60))];
        assert_eq!(fields(c.validate()), vec!["schedule"]);
        c.schedule = vec![change(5, "trade.enabled", serde_json::json!(true))];
        assert_eq!(
            fields(c.validate()),
            vec!["schedule"],
            "trade with one good is invalid"
        );
    }

    #[test]
    fn disease_defaults_are_animation_v1_and_off() {
        let d = Config::default().disease;
        assert!(!d.enabled);
        assert_eq!(
            (d.count, d.length, d.initial, d.immune_length),
            (10, URange::new(1, 10), 4, 50)
        );
        assert_eq!(
            (
                d.fee,
                d.flips_per_tick,
                d.genome_mutation,
                d.disease_mutation
            ),
            (1.0, 1, 0.0, 0.0)
        );
        assert!(d.outbreaks.is_empty());
        let partial = Config::from_json(r#"{"disease":{"enabled":true}}"#).unwrap();
        assert!(partial.disease.enabled);
        assert_eq!(partial.disease.count, 10, "missing disease fields default");
    }

    #[test]
    fn disease_parameters_are_validated() {
        let with = |f: fn(&mut DiseaseRule)| {
            let mut c = Config::default();
            f(&mut c.disease);
            fields(c.validate())
        };
        let has = |errs: Vec<String>, field: &str| errs.contains(&field.to_string());
        assert!(has(with(|d| d.immune_length = 0), "disease.immune_length"));
        assert!(has(with(|d| d.immune_length = 65), "disease.immune_length"));
        assert!(has(
            with(|d| d.length = URange::new(0, 5)),
            "disease.length"
        ));
        assert!(has(
            with(|d| d.length = URange::new(6, 5)),
            "disease.length"
        ));
        assert!(
            has(with(|d| d.length = URange::new(1, 50)), "disease.length"),
            "diseases must be shorter than the 50-bit immune string"
        );
        assert!(has(with(|d| d.count = 0), "disease.count"));
        assert!(has(with(|d| d.count = 1001), "disease.count"));
        assert!(has(with(|d| d.initial = 11), "disease.initial"));
        assert!(has(with(|d| d.fee = -1.0), "disease.fee"));
        assert!(has(
            with(|d| d.flips_per_tick = 0),
            "disease.flips_per_tick"
        ));
        assert!(has(
            with(|d| d.genome_mutation = 1.5),
            "disease.genome_mutation"
        ));
        assert!(has(
            with(|d| d.disease_mutation = f64::NAN),
            "disease.disease_mutation"
        ));
        assert!(has(
            with(|d| d.outbreaks = vec![Outbreak {
                tick: 0,
                agents: 5,
                length: None
            }]),
            "disease.outbreaks"
        ));
        assert!(has(
            with(|d| d.outbreaks = vec![Outbreak {
                tick: 3,
                agents: 0,
                length: None
            }]),
            "disease.outbreaks"
        ));
        assert!(has(
            with(|d| d.outbreaks = vec![Outbreak {
                tick: 3,
                agents: 5,
                length: Some(URange::new(5, 3))
            }]),
            "disease.outbreaks"
        ));
        assert!(has(
            with(|d| d.outbreaks = vec![Outbreak {
                tick: 3,
                agents: 5,
                length: Some(URange::new(1, 50))
            }]),
            "disease.outbreaks"
        ));
        assert!(with(|d| {
            d.enabled = true;
            d.genome_mutation = 0.01;
            d.disease_mutation = 1.0;
            d.outbreaks = vec![Outbreak {
                tick: 300,
                agents: 5,
                length: Some(URange::new(10, 10)),
            }];
        })
        .is_empty());
    }

    #[test]
    fn schedule_may_not_restructure_disease_or_set_outbreaks() {
        let rejected = |path: &str, value: serde_json::Value| {
            let c = Config {
                schedule: vec![change(5, path, value)],
                ..Default::default()
            };
            let errs = c.validate().unwrap_err();
            assert_eq!(errs[0].field, "schedule", "{path}");
            errs[0].message.clone()
        };
        for (path, value) in [
            ("disease.enabled", serde_json::json!(true)),
            ("disease.count", serde_json::json!(20)),
            ("disease.immune_length", serde_json::json!(40)),
            ("disease.length", serde_json::json!({"min": 1, "max": 5})),
            ("disease.length.max", serde_json::json!(5)),
        ] {
            let msg = rejected(path, value);
            assert!(msg.contains("only on reset"), "{path}: {msg}");
        }
        let disease = serde_json::to_value(Config::default().disease).unwrap();
        rejected("disease", disease);
        let msg = rejected("disease.outbreaks", serde_json::json!([]));
        assert!(msg.contains("their own schedule"), "{msg}");
        // Live knobs may be scheduled.
        let c = Config {
            schedule: vec![
                change(5, "disease.fee", serde_json::json!(2.0)),
                change(6, "disease.flips_per_tick", serde_json::json!(3)),
            ],
            ..Default::default()
        };
        c.validate().unwrap();
    }

    #[test]
    fn disease_structure_changes_only_on_reset() {
        let a = Config::default();
        let changed = |f: fn(&mut DiseaseRule)| {
            let mut b = a.clone();
            f(&mut b.disease);
            a.structural_changes(&b)
                .into_iter()
                .map(|e| e.field)
                .collect::<Vec<_>>()
        };
        assert_eq!(changed(|d| d.enabled = true), vec!["disease.enabled"]);
        assert_eq!(changed(|d| d.count = 20), vec!["disease.count"]);
        assert_eq!(
            changed(|d| d.length = URange::new(2, 8)),
            vec!["disease.length"]
        );
        assert_eq!(
            changed(|d| d.immune_length = 40),
            vec!["disease.immune_length"]
        );
        assert!(changed(|d| {
            d.initial = 2;
            d.fee = 2.0;
            d.flips_per_tick = 3;
            d.genome_mutation = 0.1;
            d.disease_mutation = 0.1;
            d.outbreaks = vec![Outbreak {
                tick: 9,
                agents: 1,
                length: None,
            }];
        })
        .is_empty());
    }

    #[test]
    fn with_path_indexes_into_arrays() {
        let mut c = Config::default();
        c.disease.outbreaks = vec![
            Outbreak {
                tick: 5,
                agents: 1,
                length: None,
            },
            Outbreak {
                tick: 9,
                agents: 2,
                length: None,
            },
        ];
        let next = c
            .with_path("disease.outbreaks.1.agents", &serde_json::json!(7))
            .unwrap();
        assert_eq!(
            (
                next.disease.outbreaks[0].agents,
                next.disease.outbreaks[1].agents
            ),
            (1, 7)
        );
        for bad in [
            "disease.outbreaks.2.agents",
            "disease.outbreaks.x.agents",
            "disease.outbreaks.-1",
        ] {
            let err = c.with_path(bad, &serde_json::json!(1)).unwrap_err();
            assert_eq!(err.message, format!("unknown field {bad}"));
        }
    }

    fn flat_goods(capacity: f64) -> Vec<Good> {
        vec![Good {
            map: Map::Flat { capacity },
            ..Good::sugar()
        }]
    }

    #[test]
    fn default_has_one_good_and_the_books_pollutant() {
        let c = Config::default();
        assert_eq!(c.goods, vec![Good::sugar()]);
        let g = &c.goods[0];
        assert_eq!((g.name.as_str(), g.color.as_str()), ("sugar", SUGAR_COLOR));
        assert_eq!(
            g.map,
            Map::TwoPeaks {
                transform: Transform::Identity
            }
        );
        assert_eq!(
            (g.metabolism, g.endowment),
            (URange::new(1, 4), URange::new(5, 25))
        );
        assert!(!c.pollution.enabled);
        assert_eq!(
            c.pollution.pollutants,
            vec![Pollutant {
                name: "pollution".into(),
                production: vec![1.0],
                consumption: vec![1.0],
                devalues: vec![true],
            }]
        );
        assert_eq!(
            Good::spice().map,
            Map::TwoPeaks {
                transform: Transform::MirrorX
            }
        );
    }

    #[test]
    fn legacy_json_converts_to_goods_pollutants_and_new_schedule_paths() {
        let old = r#"{
            "width": 20, "height": 20, "population": 50, "vision": {"min": 1, "max": 5},
            "landscape": {"kind": "flat", "capacity": 3.0},
            "metabolism": {"min": 2, "max": 3}, "endowment": {"min": 10, "max": 20},
            "spice": {"enabled": true, "metabolism": {"min": 1, "max": 2}, "endowment": {"min": 5, "max": 6}},
            "pollution": {"enabled": true, "production": 0.5, "consumption": 2.0, "spice_pollutes": true},
            "schedule": [{"tick": 5, "set": {
                "pollution.production": 0.0, "spice.metabolism.max": 4,
                "endowment": {"min": 1, "max": 2}}}]
        }"#;
        let c = Config::from_json(old).unwrap();
        assert_eq!(
            c.goods,
            vec![
                Good {
                    name: "sugar".into(),
                    color: SUGAR_COLOR.into(),
                    map: Map::Flat { capacity: 3.0 },
                    metabolism: URange::new(2, 3),
                    endowment: URange::new(10, 20),
                },
                Good {
                    name: "spice".into(),
                    color: SPICE_COLOR.into(),
                    map: Map::Flat { capacity: 3.0 },
                    metabolism: URange::new(1, 2),
                    endowment: URange::new(5, 6),
                },
            ]
        );
        assert_eq!(
            c.pollution,
            Pollution {
                enabled: true,
                pollutants: vec![Pollutant {
                    name: "pollution".into(),
                    production: vec![0.5, 0.5],
                    consumption: vec![2.0, 2.0],
                    devalues: vec![true, true],
                }],
            }
        );
        let paths: Vec<&str> = c.schedule[0].set.keys().map(String::as_str).collect();
        assert_eq!(
            paths,
            vec![
                "goods.0.endowment",
                "goods.1.metabolism.max",
                "pollution.pollutants.0.production.0",
                "pollution.pollutants.0.production.1",
            ]
        );

        let spicy = Config::from_json(
            r#"{"spice": {"enabled": true, "metabolism": {"min": 1, "max": 4}, "endowment": {"min": 5, "max": 25}},
                "pollution": {"enabled": false, "production": 1.0, "consumption": 1.0},
                "schedule": [{"tick": 3, "set": {"pollution.consumption": 0.0}}]}"#,
        )
        .unwrap();
        assert_eq!(
            spicy.goods[1].map,
            Map::TwoPeaks {
                transform: Transform::MirrorX
            }
        );
        let p = &spicy.pollution.pollutants[0];
        assert_eq!(
            (p.production.clone(), p.devalues.clone()),
            (vec![1.0, 0.0], vec![true, false])
        );
        assert_eq!(
            spicy.schedule[0].set.keys().collect::<Vec<_>>(),
            vec!["pollution.pollutants.0.consumption.0"],
            "only the polluting good"
        );

        assert_eq!(Config::from_json("{}").unwrap(), Config::default());
        let dropped = Config::from_json(
            r#"{"schedule": [{"tick": 3, "set": {"spice.metabolism": {"min": 1, "max": 2}}}]}"#,
        )
        .unwrap();
        assert!(
            dropped.schedule.is_empty(),
            "spice traits can't matter without spice"
        );
        for path in ["pollution.spice_pollutes", "pollution"] {
            let json = format!(r#"{{"schedule": [{{"tick": 3, "set": {{"{path}": true}}}}]}}"#);
            assert_eq!(
                Config::from_json(&json).unwrap_err()[0].field,
                "schedule",
                "{path}"
            );
        }
        let bad = r#"{"schedule": [{"tick": 3, "set": {"spice.enabled": true}}]}"#;
        assert_eq!(Config::from_json(bad).unwrap_err()[0].field, "schedule");
    }

    #[test]
    fn goods_and_pollutants_are_validated() {
        let with = |f: &dyn Fn(&mut Config)| {
            let mut c = Config::default();
            f(&mut c);
            fields(c.validate())
        };
        let has = |errs: Vec<String>, field: &str| errs.contains(&field.to_string());
        assert!(has(with(&|c| c.goods.clear()), "goods"));
        assert!(has(
            with(&|c| c.goods[0].name = String::new()),
            "goods.0.name"
        ));
        assert!(has(
            with(&|c| c.goods[0].name = "x".repeat(17)),
            "goods.0.name"
        ));
        assert!(has(
            with(&|c| c.add_good(Good {
                name: "sugar".into(),
                ..Good::spice()
            })),
            "goods.1.name"
        ));
        assert!(has(
            with(&|c| c.goods[0].color = "red".into()),
            "goods.0.color"
        ));
        assert!(has(
            with(&|c| c.goods[0].color = "#12345g".into()),
            "goods.0.color"
        ));
        assert!(with(&|c| c.goods[0].color = "#A0b1C2".into()).is_empty());
        assert!(has(
            with(&|c| c.goods[0].metabolism = URange::new(3, 1)),
            "goods.0.metabolism"
        ));
        assert!(has(
            with(&|c| c.goods[0].endowment = URange::new(3, 1)),
            "goods.0.endowment"
        ));
        let peaks = |p: Vec<Peak>| Map::Peaks { peaks: p };
        let peak = |x, y, radius, height| Peak {
            x,
            y,
            radius,
            height,
        };
        assert!(has(
            with(&|c| c.goods[0].map = peaks(vec![])),
            "goods.0.map"
        ));
        assert!(has(
            with(&|c| c.goods[0].map = peaks(vec![peak(50, 0, 5.0, 4.0)])),
            "goods.0.map"
        ));
        assert!(has(
            with(&|c| c.goods[0].map = peaks(vec![peak(0, 0, 0.0, 4.0)])),
            "goods.0.map"
        ));
        assert!(has(
            with(&|c| c.goods[0].map = peaks(vec![peak(0, 0, 5.0, 11.0)])),
            "goods.0.map"
        ));
        assert!(with(&|c| c.goods[0].map = peaks(vec![peak(49, 49, 5.0, 10.0)])).is_empty());
        assert!(has(
            with(&|c| c.goods[0].map = Map::Flat { capacity: -1.0 }),
            "goods.0.map.capacity"
        ));
        assert!(has(
            with(&|c| c.pollution.pollutants.clear()),
            "pollution.pollutants"
        ));
        assert!(has(
            with(&|c| c.pollution.pollutants[0].production = vec![1.0, 1.0]),
            "pollution.pollutants.0.production"
        ));
        assert!(has(
            with(&|c| c.pollution.pollutants[0].consumption = vec![-1.0]),
            "pollution.pollutants.0.consumption"
        ));
        assert!(has(
            with(&|c| c.pollution.pollutants[0].devalues = vec![]),
            "pollution.pollutants.0.devalues"
        ));
        assert!(has(
            with(&|c| c.pollution.pollutants.push(Pollutant::book(1))),
            "pollution.pollutants.1.name"
        ));
    }

    #[test]
    fn rule_dependencies_count_goods() {
        let with = |f: &dyn Fn(&mut Config)| {
            let mut c = Config::default();
            f(&mut c);
            fields(c.validate())
        };
        assert!(with(&|c| c.trade.enabled = true).contains(&"trade.enabled".to_string()));
        assert!(with(&|c| c.foresight.enabled = true).contains(&"foresight.enabled".to_string()));
        assert!(with(&|c| {
            c.add_good(Good::spice());
            c.combat.enabled = true;
        })
        .contains(&"combat.enabled".to_string()));
        assert!(with(&|c| {
            c.add_good(Good::spice());
            c.trade.enabled = true;
            c.foresight.enabled = true;
        })
        .is_empty());
    }

    #[test]
    fn add_and_remove_good_keep_pollutant_columns_in_step() {
        let mut c = Config::default();
        c.add_good(Good::spice());
        let p = &c.pollution.pollutants[0];
        assert_eq!(
            (p.production.clone(), p.consumption.clone()),
            (vec![1.0, 0.0], vec![1.0, 0.0])
        );
        assert_eq!(p.devalues, vec![true, false]);
        c.remove_good(0);
        assert_eq!(c.goods, vec![Good::spice()]);
        assert_eq!(c.pollution.pollutants[0].production, vec![0.0]);
        assert_eq!(c.pollution.pollutants[0].devalues, vec![false]);
    }

    #[test]
    fn new_shape_json_without_pollution_gets_the_books_pollutant_on_every_good() {
        let mut two = Config::default();
        two.add_good(Good::spice());
        let mut value = serde_json::to_value(&two).unwrap();
        value.as_object_mut().unwrap().remove("pollution");
        let c = Config::from_json(&value.to_string()).unwrap();
        assert_eq!(c.pollution.pollutants, vec![Pollutant::book(2)]);
        assert!(!c.pollution.enabled);
    }

    #[test]
    fn adding_and_removing_goods_retargets_the_schedule() {
        let paths = |c: &Config| -> Vec<Vec<String>> {
            c.schedule
                .iter()
                .map(|s| s.set.keys().cloned().collect())
                .collect()
        };
        let v = serde_json::json!(0);
        let mut c = Config::default();
        c.add_good(Good::spice());
        c.add_good(Good::spice());
        c.schedule = vec![
            change(5, "goods.0.name", v.clone()),
            change(6, "goods.1.metabolism.max", v.clone()),
            change(7, "goods.2.endowment", v.clone()),
            change(8, "pollution.pollutants.0.production.1", v.clone()),
            change(9, "pollution.pollutants.0.devalues.2", v.clone()),
            change(10, "pollution.pollutants.0.consumption", v.clone()),
            change(11, "pollution.pollutants.0.name", v.clone()),
            change(12, "pollution.pollutants.0", v.clone()),
            change(13, "trade.enabled", v.clone()),
        ];
        c.schedule[0].set.insert("goods.1.name".into(), v.clone());
        c.remove_good(1);
        assert_eq!(
            paths(&c),
            vec![
                vec!["goods.0.name"],
                vec!["goods.1.endowment"],
                vec!["pollution.pollutants.0.devalues.1"],
                vec!["pollution.pollutants.0.name"],
                vec!["trade.enabled"],
            ]
        );
        assert_eq!(
            c.schedule.iter().map(|s| s.tick).collect::<Vec<_>>(),
            vec![5, 7, 9, 11, 13]
        );
        c.schedule
            .push(change(14, "pollution.pollutants.0.production", v.clone()));
        c.schedule
            .push(change(15, "pollution.pollutants.0.production.1", v.clone()));
        c.add_good(Good::spice());
        assert_eq!(
            c.schedule.iter().map(|s| s.tick).collect::<Vec<_>>(),
            vec![5, 7, 9, 11, 13, 15]
        );
    }

    #[test]
    fn removing_a_pollutant_retargets_the_schedule() {
        let v = serde_json::json!(0);
        let mut c = Config::default();
        c.pollution.pollutants.push(Pollutant::book(1));
        c.pollution.pollutants.push(Pollutant::book(1));
        c.schedule = vec![
            change(5, "pollution.pollutants.0.name", v.clone()),
            change(6, "pollution.pollutants.1.production.0", v.clone()),
            change(7, "pollution.pollutants.2.devalues", v.clone()),
            change(8, "pollution.enabled", v.clone()),
        ];
        c.remove_pollutant(1);
        assert_eq!(c.pollution.pollutants.len(), 2);
        let paths: Vec<&str> = c
            .schedule
            .iter()
            .flat_map(|s| s.set.keys().map(String::as_str))
            .collect();
        assert_eq!(
            paths,
            vec![
                "pollution.pollutants.0.name",
                "pollution.pollutants.1.devalues",
                "pollution.enabled"
            ]
        );
    }

    #[test]
    fn schedule_may_change_traits_and_coefficients_but_not_structure() {
        let rejected = |path: &str, value: serde_json::Value| {
            let c = Config {
                schedule: vec![change(5, path, value)],
                ..Default::default()
            };
            let errs = c.validate().unwrap_err();
            assert_eq!(errs[0].field, "schedule", "{path}");
            errs[0].message.clone()
        };
        let goods = serde_json::to_value(Config::default().goods).unwrap();
        let good = serde_json::to_value(Good::sugar()).unwrap();
        let pollution = serde_json::to_value(Config::default().pollution).unwrap();
        for (path, value) in [
            ("goods", goods),
            ("goods.0", good),
            (
                "goods.0.map",
                serde_json::json!({"kind": "flat", "capacity": 1.0}),
            ),
            ("goods.0.map.transform", serde_json::json!("rotate_90")),
            ("pollution", pollution.clone()),
            ("pollution.pollutants", pollution["pollutants"].clone()),
        ] {
            let msg = rejected(path, value);
            assert!(msg.contains("only on reset"), "{path}: {msg}");
        }
        let msg = rejected("schedule", serde_json::json!([]));
        assert!(msg.contains("schedule cannot change itself"), "{msg}");
        let c = Config {
            schedule: vec![
                change(
                    5,
                    "goods.0.metabolism",
                    serde_json::json!({"min": 1, "max": 2}),
                ),
                change(6, "goods.0.endowment.max", serde_json::json!(30)),
                change(7, "goods.0.name", serde_json::json!("honey")),
                change(8, "goods.0.color", serde_json::json!("#123456")),
                change(
                    9,
                    "pollution.pollutants.0.production.0",
                    serde_json::json!(0.0),
                ),
                change(
                    9,
                    "pollution.pollutants.0.devalues.0",
                    serde_json::json!(false),
                ),
            ],
            ..Default::default()
        };
        c.validate().unwrap();
    }

    #[test]
    fn structural_changes_cover_goods_maps_and_pollutants() {
        let a = Config::default();
        let changed = |f: &dyn Fn(&mut Config)| {
            let mut b = a.clone();
            f(&mut b);
            a.structural_changes(&b)
                .into_iter()
                .map(|e| e.field)
                .collect::<Vec<_>>()
        };
        assert_eq!(changed(&|c| c.add_good(Good::spice())), vec!["goods"]);
        assert_eq!(
            changed(&|c| c.goods[0].map = Map::Flat { capacity: 1.0 }),
            vec!["goods.0.map"]
        );
        assert_eq!(
            changed(&|c| c.pollution.pollutants.push(Pollutant::book(1))),
            vec!["pollution.pollutants"]
        );
        assert!(changed(&|c| {
            c.goods[0].metabolism = URange::new(2, 3);
            c.goods[0].name = "honey".into();
            c.pollution.pollutants[0].production[0] = 0.0;
        })
        .is_empty());
    }

    /// `n` one-bits (the low `n` tag positions).
    fn ones(n: u32) -> u64 {
        if n == 64 {
            u64::MAX
        } else {
            (1u64 << n) - 1
        }
    }

    #[test]
    fn default_groups_reproduce_the_two_tribe_rule() {
        use crate::agent::{Tags, Tribe};
        let tribe_index = |t: Tribe| match t {
            Tribe::Blue => 0,
            Tribe::Red => 1,
        };
        let groups = default_groups(11);
        assert_eq!(
            groups,
            vec![
                Group::new("Blue", BLUE_COLOR, 6, 11),
                Group::new("Red", RED_COLOR, 0, 5)
            ]
        );
        for bits in 0..1u64 << 11 {
            let tags = Tags::new(bits, 11);
            assert_eq!(
                group_of(&groups, tags.zeros()),
                tribe_index(tags.tribe()),
                "{}",
                tags.to_bit_string()
            );
        }
        for len in 1..=64 {
            let groups = default_groups(len);
            for zeros in 0..=len {
                let tags = Tags::new(ones(len - zeros), len);
                assert_eq!(tags.zeros(), zeros);
                assert_eq!(
                    group_of(&groups, zeros),
                    tribe_index(tags.tribe()),
                    "L = {len}, {zeros} zeros"
                );
            }
        }
    }

    #[test]
    fn three_tribes_are_the_books_thirds() {
        let spans = |g: Vec<Group>| -> Vec<(String, u32, u32)> {
            g.into_iter()
                .map(|g| (g.name, g.zeros.min, g.zeros.max))
                .collect()
        };
        let s = |name: &str, min, max| (name.to_string(), min, max);
        assert_eq!(
            spans(three_tribes(11)),
            vec![s("Blue", 0, 3), s("Green", 4, 7), s("Red", 8, 11)]
        );
        assert_eq!(
            spans(three_tribes(2)),
            vec![s("Blue", 0, 0), s("Green", 1, 1), s("Red", 2, 2)]
        );
        let groups = three_tribes(11);
        assert_eq!(
            [0, 3, 4, 7, 8, 11].map(|z| group_of(&groups, z)),
            [0, 0, 1, 1, 2, 2]
        );
        let mut c = Config::default();
        c.culture.groups = groups;
        c.validate().unwrap();
    }

    #[test]
    fn groups_are_validated() {
        let with = |f: &dyn Fn(&mut Vec<Group>)| {
            let mut c = Config::default();
            f(&mut c.culture.groups);
            c.validate().err().unwrap_or_default()
        };
        let has = |errs: &[FieldError], field: &str, text: &str| {
            errs.iter()
                .any(|e| e.field == field && e.message.contains(text))
        };
        assert!(with(&|_| {}).is_empty());
        let gap = with(&|g| g[0].zeros.min = 7);
        assert!(has(&gap, "culture.groups", "with 6 zeros"), "{gap:?}");
        let overlap = with(&|g| g[1].zeros.max = 6);
        assert!(
            has(&overlap, "culture.groups", "more than one group"),
            "{overlap:?}"
        );
        let outside = with(&|g| g[0].zeros.max = 12);
        assert!(
            has(&outside, "culture.groups.0.zeros", "0–11"),
            "{outside:?}"
        );
        let inverted = with(&|g| g[0].zeros = URange::new(11, 6));
        assert!(has(
            &inverted,
            "culture.groups.0.zeros",
            "min must be ≤ max"
        ));
        assert!(has(&with(&|g| g.clear()), "culture.groups", "1 to 8"));
        let nine = with(&|g| {
            *g = (0..9)
                .map(|k| Group::new(&format!("g{k}"), BLUE_COLOR, k, if k == 8 { 11 } else { k }))
                .collect();
        });
        assert!(has(&nine, "culture.groups", "1 to 8"), "{nine:?}");
        assert_eq!(
            nine.len(),
            1,
            "nine groups that tile 0–11: only the count is wrong"
        );
        assert!(has(
            &with(&|g| g[1].name = "Blue".into()),
            "culture.groups.1.name",
            "another group"
        ));
        assert!(has(
            &with(&|g| g[0].name = String::new()),
            "culture.groups.0.name",
            "1–16"
        ));
        assert!(has(
            &with(&|g| g[0].color = "blue".into()),
            "culture.groups.0.color",
            "#rrggbb"
        ));
        // A shorter tag length strands the default groups for 11 bits.
        let short = Config {
            tag_length: 5,
            ..Config::default()
        };
        assert!(fields(short.validate()).contains(&"culture.groups.0.zeros".to_string()));
        let fixed = Config {
            tag_length: 5,
            culture: CultureRule {
                enabled: false,
                groups: default_groups(5),
            },
            ..Config::default()
        };
        fixed.validate().unwrap();
    }

    #[test]
    fn configs_without_groups_get_the_two_tribes_for_their_tag_length() {
        assert_eq!(Config::default().culture.groups, default_groups(11));
        // Pre-N-goods shape (no `goods` key): always the default.
        let legacy =
            Config::from_json(r#"{"tag_length": 5, "culture": {"enabled": true}}"#).unwrap();
        assert_eq!(legacy.culture.groups, default_groups(5));
        assert!(legacy.culture.enabled);
        // New shape without `culture.groups`, and without `culture`.
        let seven = Config {
            tag_length: 7,
            culture: CultureRule {
                enabled: true,
                groups: default_groups(7),
            },
            ..Config::default()
        };
        let mut value = serde_json::to_value(&seven).unwrap();
        value["culture"].as_object_mut().unwrap().remove("groups");
        assert_eq!(Config::from_json(&value.to_string()).unwrap(), seven);
        value.as_object_mut().unwrap().remove("culture");
        let c = Config::from_json(&value.to_string()).unwrap();
        assert_eq!(
            (c.culture.enabled, c.culture.groups),
            (false, default_groups(7))
        );
        // Given groups are kept, in the core's key order.
        let mut three = Config::default();
        three.culture.groups = three_tribes(11);
        let json = serde_json::to_string(&three).unwrap();
        assert_eq!(Config::from_json(&json).unwrap(), three);
        assert!(json.contains(
            r##""groups":[{"name":"Blue","color":"#3d7eff","zeros":{"min":0,"max":3}}"##
        ));
        // An explicit empty list is not "missing".
        let mut empty = serde_json::to_value(Config::default()).unwrap();
        empty["culture"]["groups"] = serde_json::json!([]);
        assert_eq!(
            Config::from_json(&empty.to_string()).unwrap_err()[0].field,
            "culture.groups"
        );
    }

    #[test]
    fn group_names_and_colors_are_live_but_ranges_change_only_on_reset() {
        let live = Config {
            schedule: vec![
                change(5, "culture.groups.0.name", serde_json::json!("Azure")),
                change(6, "culture.groups.1.color", serde_json::json!("#aa0000")),
                change(7, "culture.enabled", serde_json::json!(true)),
            ],
            ..Default::default()
        };
        live.validate().unwrap();
        for (path, value) in [
            (
                "culture",
                serde_json::to_value(&Config::default().culture).unwrap(),
            ),
            ("culture.groups", serde_json::json!([])),
            (
                "culture.groups.0",
                serde_json::to_value(&default_groups(11)[0]).unwrap(),
            ),
            (
                "culture.groups.0.zeros",
                serde_json::json!({"min": 6, "max": 11}),
            ),
            ("culture.groups.1.zeros.max", serde_json::json!(5)),
        ] {
            let c = Config {
                schedule: vec![change(5, path, value)],
                ..Default::default()
            };
            let errs = c.validate().unwrap_err();
            assert!(
                errs[0].message.contains("only on reset"),
                "{path}: {errs:?}"
            );
        }
        let a = Config::default();
        let mut b = a.clone();
        b.culture.enabled = true;
        b.culture.groups[0].name = "Azure".into();
        b.culture.groups[1].color = "#aa0000".into();
        assert!(a.structural_changes(&b).is_empty());
        let mut moved = a.clone();
        moved.culture.groups[0].zeros.min = 7;
        moved.culture.groups[1].zeros.max = 6;
        assert_eq!(a.structural_changes(&moved)[0].field, "culture.groups");
        let mut one = a.clone();
        one.culture.groups = vec![Group::new("All", BLUE_COLOR, 0, 11)];
        assert_eq!(a.structural_changes(&one)[0].field, "culture.groups");
    }
}
