//! Schelling's line (1971, "Linear Distribution", pp. 149–154): stars and
//! zeros in a row with no gaps; each counts "the four nearest neighbors on
//! either side of him" and wants at least half of them like himself; the
//! discontented, a round at a time "counting from left to right", move to
//! "the nearest point that meets his minimum demand … 'Nearest' means the
//! point reached by passing the smallest number of neighbors on the way",
//! intruding between two others. Stars are Red, zeros Blue.

use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::presets::ModelPreset;
use crate::render::{Rgb, BACKGROUND};
use crate::rng::{self, SimRng};
use crate::schema::{Apply, Param};
use crate::stats::{Series, Stats};

const RED: Rgb = [0xd9, 0x4a, 0x3f];
const BLUE: Rgb = [0x3f, 0x7f, 0xd9];
const DISCONTENT: Rgb = [0xf2, 0xc9, 0x4c];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LineConfig {
    /// People in the row (Schelling: 70, "the 70 people who fit within the
    /// margins of a typewriter").
    pub length: u32,
    /// The share who are Red (stars).
    pub red_share: f64,
    /// Exactly `round(length × red_share)` Red; off (Schelling's random
    /// digits: "It turns out that there are 35 stars and 35 zeros"), a coin each.
    pub exact: bool,
    /// Neighbors counted on either side (Schelling: four; three in a variation).
    pub radius: u32,
    /// The least share of neighbors alike wanted ("at least half").
    pub preference: f64,
    /// The most people a mover may pass (0: any number); Schelling's
    /// "restricted movement", whose radius he does not give.
    pub reach: u32,
    /// Within a limited reach and failing `preference`, the share accepted
    /// instead ("the nearest place where three out of eight occur").
    pub fallback: f64,
    /// People per row when drawn.
    pub wrap: u32,
}

impl Default for LineConfig {
    fn default() -> Self {
        LineConfig {
            length: 70,
            red_share: 0.5,
            exact: false,
            radius: 4,
            preference: 0.5,
            reach: 0,
            fallback: 0.375,
            wrap: 70,
        }
    }
}

impl LineConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let fraction = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (2..=10_000).contains(&self.length),
            "length",
            "must be between 2 and 10000",
        );
        check(
            fraction(self.red_share),
            "red_share",
            "must be a fraction between 0 and 1",
        );
        check(
            (1..=20).contains(&self.radius),
            "radius",
            "must be between 1 and 20",
        );
        check(
            fraction(self.preference),
            "preference",
            "must be a fraction between 0 and 1",
        );
        check(
            fraction(self.fallback),
            "fallback",
            "must be a fraction between 0 and 1",
        );
        check(self.wrap >= 1, "wrap", "must be at least 1");
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    fn changes(&self, next: &LineConfig) -> Vec<FieldError> {
        if self == next {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (field, same) in [
            ("length", self.length == next.length),
            ("red_share", self.red_share == next.red_share),
            ("exact", self.exact == next.exact),
            ("radius", self.radius == next.radius),
            ("preference", self.preference == next.preference),
            ("reach", self.reach == next.reach),
            ("fallback", self.fallback == next.fallback),
            ("wrap", self.wrap == next.wrap),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The statistics series.
pub const SERIES: [&str; 8] = [
    "groups",
    "mean_group",
    "like_share",
    "no_unlike",
    "unsatisfied",
    "moves",
    "red_share",
    "quiet",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct LineSnapshot {
    pub tick: u64,
    /// Runs of one color ("clusters").
    pub groups: u32,
    pub mean_group: f64,
    /// Mean share of neighbors alike.
    pub like_share: f64,
    /// Share with no neighbor of the other color.
    pub no_unlike: f64,
    pub unsatisfied: f64,
    pub moves: u32,
    pub red_share: f64,
    /// 1 when a round ran and nobody moved.
    pub quiet: u32,
}

impl Series for LineSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "groups" => f64::from(self.groups),
            "mean_group" => self.mean_group,
            "like_share" => self.like_share,
            "no_unlike" => self.no_unlike,
            "unsatisfied" => self.unsatisfied,
            "moves" => f64::from(self.moves),
            "red_share" => self.red_share,
            "quiet" => f64::from(self.quiet),
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Person {
    pub id: u64,
    pub red: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PersonView {
    pub id: u64,
    pub place: u32,
    pub color: &'static str,
    pub like: u32,
    pub neighbors: u32,
    pub satisfied: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LineInspection {
    pub person: Option<PersonView>,
}

#[derive(Clone)]
pub struct LineWorld {
    pub config: LineConfig,
    pub tick: u64,
    people: Vec<Person>,
    rng: SimRng,
    moves: u32,
    pub stats: Stats<LineSnapshot>,
}

impl LineWorld {
    pub fn new(config: LineConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.length as usize;
        let mut colors: Vec<bool> = if config.exact {
            let reds = (f64::from(config.length) * config.red_share).round() as usize;
            let mut v: Vec<bool> = (0..n).map(|k| k < reds).collect();
            v.shuffle(&mut rng);
            v
        } else {
            (0..n).map(|_| rng.gen_bool(config.red_share)).collect()
        };
        let people = colors
            .drain(..)
            .enumerate()
            .map(|(k, red)| Person {
                id: k as u64 + 1,
                red,
            })
            .collect();
        let mut world = LineWorld {
            config,
            tick: 0,
            people,
            rng,
            moves: 0,
            stats: Stats::default(),
        };
        let s = world.snapshot();
        world.stats.push(s);
        Ok(world)
    }

    pub fn people(&self) -> &[Person] {
        &self.people
    }

    /// Like-colored and all neighbors of the person at `i`: up to `radius`
    /// on each side, fewer near the ends.
    pub fn counts(&self, i: usize) -> (u32, u32) {
        let red = self.people[i].red;
        self.around(&self.people, i, i + 1, red)
    }

    /// Like and all neighbors of a `red` person standing between `row[..left]`
    /// and `row[right..]`.
    fn around(&self, row: &[Person], left: usize, right: usize, red: bool) -> (u32, u32) {
        let r = self.config.radius as usize;
        let lo = left.saturating_sub(r);
        let hi = (right + r).min(row.len());
        let (mut like, mut n) = (0, 0);
        for p in row[lo..left].iter().chain(&row[right..hi]) {
            n += 1;
            like += u32::from(p.red == red);
        }
        (like, n)
    }

    fn content(&self, like: u32, n: u32, share: f64) -> bool {
        n == 0 || f64::from(like) / f64::from(n) >= share
    }

    pub fn is_satisfied(&self, i: usize) -> bool {
        let (like, n) = self.counts(i);
        self.content(like, n, self.config.preference)
    }

    /// The person at `i` takes its turn: if discontent, it leaves its place
    /// and intrudes at the gap passing fewest people where it would be
    /// content (ties at random); with a limited reach, failing that, the
    /// nearest gap within reach meeting `fallback`; failing both, it stays.
    /// Whether it moved.
    pub fn turn(&mut self, i: usize) -> bool {
        if self.is_satisfied(i) {
            return false;
        }
        let me = self.people.remove(i);
        let len = self.people.len();
        let reach = if self.config.reach == 0 {
            len
        } else {
            self.config.reach as usize
        };
        let demands = if self.config.reach == 0 {
            vec![self.config.preference]
        } else {
            vec![self.config.preference, self.config.fallback]
        };
        for share in demands {
            for d in 1..=reach {
                let found: Vec<usize> = [i.checked_sub(d), Some(i + d)]
                    .into_iter()
                    .flatten()
                    .filter(|&j| j <= len)
                    .filter(|&j| {
                        let (like, n) = self.around(&self.people, j, j, me.red);
                        self.content(like, n, share)
                    })
                    .collect();
                if !found.is_empty() {
                    let j = found[self.rng.gen_range(0..found.len() as u32) as usize];
                    self.people.insert(j, me);
                    return true;
                }
            }
        }
        self.people.insert(i, me);
        false
    }

    /// One round: the discontented at its start, left to right; each still
    /// discontent when its turn comes moves; one made discontent waits.
    pub fn step(&mut self) {
        let movers: Vec<u64> = (0..self.people.len())
            .filter(|&i| !self.is_satisfied(i))
            .map(|i| self.people[i].id)
            .collect();
        let mut moves = 0;
        for id in movers {
            let i = self
                .people
                .iter()
                .position(|p| p.id == id)
                .expect("a person");
            if self.turn(i) {
                moves += 1;
            }
        }
        self.moves = moves;
        self.tick += 1;
        let s = self.snapshot();
        self.stats.push(s);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    fn snapshot(&self) -> LineSnapshot {
        let n = self.people.len();
        let mut groups = 0u32;
        for (k, p) in self.people.iter().enumerate() {
            if k == 0 || self.people[k - 1].red != p.red {
                groups += 1;
            }
        }
        let (mut share, mut unmixed, mut unsatisfied, mut reds) = (0.0, 0usize, 0usize, 0usize);
        for i in 0..n {
            let (like, all) = self.counts(i);
            if all > 0 {
                share += f64::from(like) / f64::from(all);
            }
            unmixed += usize::from(like == all);
            unsatisfied += usize::from(!self.content(like, all, self.config.preference));
            reds += usize::from(self.people[i].red);
        }
        let of = |k: usize| if n == 0 { 0.0 } else { k as f64 / n as f64 };
        LineSnapshot {
            tick: self.tick,
            groups,
            mean_group: if groups == 0 {
                0.0
            } else {
                n as f64 / f64::from(groups)
            },
            like_share: if n == 0 { 0.0 } else { share / n as f64 },
            no_unlike: of(unmixed),
            unsatisfied: of(unsatisfied),
            moves: self.moves,
            red_share: of(reds),
            quiet: u32::from(self.tick > 0 && self.moves == 0),
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<LineInspection, String> {
        let (w, h) = self.dims();
        if x >= w || y >= h {
            return Err(format!("({x}, {y}) is outside the line"));
        }
        let k = (y * w + x) as usize;
        Ok(LineInspection {
            person: self.people.get(k).map(|p| {
                let (like, neighbors) = self.counts(k);
                PersonView {
                    id: p.id,
                    place: k as u32,
                    color: if p.red { "red" } else { "blue" },
                    like,
                    neighbors,
                    satisfied: self.content(like, neighbors, self.config.preference),
                }
            }),
        })
    }

    fn dims(&self) -> (u32, u32) {
        let w = self.config.wrap.min(self.config.length).max(1);
        (w, self.config.length.div_ceil(w))
    }

    #[cfg(test)]
    fn colors(&self) -> String {
        self.people
            .iter()
            .map(|p| if p.red { 'R' } else { 'B' })
            .collect()
    }

    #[cfg(test)]
    fn set_colors(&mut self, s: &str) {
        for (p, c) in self.people.iter_mut().zip(s.chars()) {
            p.red = c == 'R';
        }
        let snap = self.snapshot();
        self.stats = Stats::default();
        self.stats.push(snap);
    }
}

impl Model for LineWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Line(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        LineWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.people.len()
    }

    /// FNV-1a over the tick and each person's id and color, in order.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for p in &self.people {
            eat(p.id);
            eat(u64::from(p.red));
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        self.dims()
    }

    /// The row wrapped every `wrap` people: Red and Blue (`color`), or the
    /// discontented yellow (`satisfaction`); past the end, dark.
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let satisfaction = match mode {
            "color" => false,
            "satisfaction" => true,
            _ => return Err(format!("unknown color mode {mode:?}")),
        };
        let (w, h) = self.dims();
        buf.clear();
        buf.resize((w * h) as usize * 4, 0);
        for k in 0..(w * h) as usize {
            let rgb = match self.people.get(k) {
                None => BACKGROUND,
                Some(_) if satisfaction && !self.is_satisfied(k) => DISCONTENT,
                Some(p) => {
                    if p.red {
                        RED
                    } else {
                        BLUE
                    }
                }
            };
            buf[k * 4..k * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,place,color,like,neighbors,satisfied\n");
        for (k, p) in self.people.iter().enumerate() {
            let (like, n) = self.counts(k);
            writeln!(
                out,
                "{},{},{},{},{},{}",
                p.id,
                k,
                if p.red { "red" } else { "blue" },
                like,
                n,
                self.content(like, n, self.config.preference)
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let (w, _) = self.dims();
        self.people
            .iter()
            .position(|p| p.id == id)
            .map(|k| (k as u32 % w, k as u32 / w))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Line(next) = next else {
            return Err(wrong_model(ModelKind::Line, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }
}

/// The Rules panel's fields: every one rebuilds the line.
pub fn schema() -> Vec<Param> {
    use Apply::Reset;
    vec![
        Param::integer("Setup", "length", "People", (2, 2000), Reset).with_help(
            "Schelling: 70, \"the 70 people who fit within the margins of a typewriter\".",
        ),
        Param::number("Setup", "red_share", "Red share", (0.0, 1.0, 0.01), Reset)
            .with_help("Schelling's stars are Red, his zeros Blue."),
        Param::bool("Setup", "exact", "Exact numbers", Reset)
            .with_help("Off: a coin for each person, as Schelling's random digits were."),
        Param::integer(
            "Setup",
            "wrap",
            "People per row (display)",
            (10, 200),
            Reset,
        ),
        Param::integer(
            "Neighborhood",
            "radius",
            "Neighbors on each side",
            (1, 20),
            Reset,
        )
        .with_help("Schelling: four; three in a variation."),
        Param::number(
            "Preference",
            "preference",
            "Like neighbors wanted (fraction)",
            (0.0, 1.0, 0.05),
            Reset,
        ),
        Param::integer(
            "Movement",
            "reach",
            "Most people passed (0: any)",
            (0, 2000),
            Reset,
        )
        .with_help("Schelling's restricted movement; he does not give the radius."),
        Param::number(
            "Movement",
            "fallback",
            "Accepted within reach instead",
            (0.0, 1.0, 0.025),
            Reset,
        )
        .with_help("\"the nearest place where three out of eight occur\""),
    ]
}

fn preset(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut LineConfig),
) -> ModelPreset {
    let mut c = LineConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source: "Schelling 1971",
        description,
        config: ModelConfig::Line(c),
    }
}

/// Schelling's line and its variations (1971, pp. 149–154).
pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "s71-line",
            "Schelling's line",
            "Fig. 1: 70 stars (Red) and zeros (Blue) at random; each wants at least half of its four neighbors on either side like itself; the discontented, left to right a round at a time, move to the nearest point that suits them. Schelling: \"six clusters … averaging 12 members\", and from tabletop runs \"from about five groupings with an average of 14 members to seven or eight groupings with an average of 9 or 10\".",
            |_| {},
        ),
        preset(
            "s71-line-3",
            "Schelling's line, three neighbors each side",
            "The neighborhood cut to three on either side (p. 152). Schelling: \"a mean of 7 or 8 per cluster … with the average person's neighborhood 75% to 80% his own color\".",
            |c| c.radius = 3,
        ),
        preset(
            "s71-line-minority",
            "Schelling's line with a minority",
            "Half the zeros (Blue) removed, as Schelling did by die roll (p. 152): 35 stars to 18 zeros. Schelling: \"the minority itself tends to become more segregated from the majority, as its relative size diminishes\".",
            |c| {
                c.length = 53;
                c.red_share = 35.0 / 53.0;
            },
        ),
        preset(
            "s71-line-reach",
            "Schelling's line, movement restricted",
            "A 20% minority (Blue) that may pass at most 10 people; failing half alike within reach, \"the nearest place where three out of eight occur\" (pp. 153–154; the radius is not given). Schelling: \"everybody achieves his desired neighborhood … without traveling as far\".",
            |c| {
                c.red_share = 0.8;
                c.reach = 10;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line from a string of R and B, Schelling's defaults otherwise.
    fn line_of(s: &str, radius: u32) -> LineWorld {
        let mut w = LineWorld::new(
            LineConfig {
                length: s.len() as u32,
                radius,
                ..LineConfig::default()
            },
            1,
        )
        .unwrap();
        w.set_colors(s);
        w
    }

    #[test]
    fn a_line_mover_inserts_at_the_nearest_point_that_satisfies_it() {
        // Only the red at the left is discontent (0 of its 4 neighbors red).
        // Passing nine blues brings it to the red run, where 4 of its 8 are red.
        let mut w = line_of("RBBBBBBBBBRRRRRBBBBB", 4);
        w.step();
        assert_eq!(w.colors(), "BBBBBBBBBRRRRRRBBBBB");
        assert_eq!(w.stats.latest().unwrap().moves, 1);
        w.step();
        assert_eq!(w.stats.latest().unwrap().moves, 0, "everyone is content");
    }

    #[test]
    fn the_ends_of_the_line_count_what_is_there() {
        let w = line_of("RRBBBBBB", 4);
        assert_eq!(
            w.counts(0),
            (1, 4),
            "the first person has only its four inward neighbors"
        );
        assert_eq!(
            w.counts(2),
            (4, 6),
            "a blue with two outboard (red) and four inward (blue)"
        );
        assert_eq!(w.counts(4), (5, 7), "four on the left, three on the right");
    }

    #[test]
    fn a_limited_reach_falls_back_to_the_lesser_demand() {
        // Within three people passed, no point is half red for the red at the
        // left, but one point (after the first blue) is 2 of 5: over 3/8.
        let s = "RBBRBRBBBBBBBBRRRRRRBBBB";
        let mut near = line_of(s, 4);
        near.config.reach = 3;
        assert!(near.turn(0));
        assert_eq!(
            &near.colors()[..3],
            "BRB",
            "the fallback point, one person along"
        );
        let mut free = line_of(s, 4);
        assert!(free.turn(0));
        let at = free.colors().find("RRRRRRR").expect("it joins the red run");
        assert!(at > 5, "without a limit it travels to where half are red");
        let mut stuck = line_of(s, 4);
        stuck.config.reach = 3;
        stuck.config.fallback = 0.5;
        assert!(
            !stuck.turn(0),
            "nothing within reach meets either demand: it stays"
        );
        assert_eq!(stuck.colors(), s);
    }

    #[test]
    fn rounds_move_the_discontented_left_to_right_and_skip_the_content() {
        // One neighbor each side: both reds are discontent at the start. The
        // first moves in beside the second (two people along), which is then
        // content when its turn comes, and stays.
        let mut w = line_of("RBBRBBB", 1);
        assert_eq!(w.stats.latest().unwrap().unsatisfied, 2.0 / 7.0);
        w.step();
        assert_eq!(w.colors(), "BBRRBBB");
        assert_eq!(w.stats.latest().unwrap().moves, 1);
    }

    #[test]
    fn exact_counts_and_coins() {
        let exact = LineWorld::new(
            LineConfig {
                exact: true,
                ..LineConfig::default()
            },
            3,
        )
        .unwrap();
        assert_eq!(exact.colors().matches('R').count(), 35);
        let coin = LineWorld::new(LineConfig::default(), 3).unwrap();
        assert_eq!(coin.colors().len(), 70);
    }

    #[test]
    fn groups_and_like_shares() {
        let w = line_of("RRRBBRRRRB", 4);
        let s = w.stats.latest().unwrap();
        assert_eq!(s.groups, 4);
        assert_eq!(s.mean_group, 2.5);
        assert_eq!(
            s.no_unlike, 0.0,
            "every one of these ten sees the other color within four"
        );
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Line(LineConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }

    #[test]
    fn renders_wrapped_rows() {
        let mut w = LineWorld::new(
            LineConfig {
                length: 100,
                wrap: 70,
                ..LineConfig::default()
            },
            1,
        )
        .unwrap();
        assert_eq!(w.size(), (70, 2));
        let mut buf = Vec::new();
        w.render("color", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 140 * 4);
        assert_eq!(
            &buf[139 * 4..139 * 4 + 3],
            &BACKGROUND,
            "past the end of the line"
        );
        w.run(3);
        assert!(
            w.inspect(0, 1).unwrap().person.is_some() && w.inspect(40, 1).unwrap().person.is_none()
        );
    }
}
