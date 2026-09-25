//! The tags world: agents donate to those whose tags lie within their
//! tolerance, then the next generation is chosen by score and mutated.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{DonationTest, InitialTolerance, Selection, TagsConfig, TieRule};
use super::stats::{cluster, TagsSnapshot, Takeovers};
use crate::anasazi::random::normal;
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, COOL, HOT, SUGAR};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

/// Rows of the diagram: the last this many generations.
pub const HISTORY: usize = 200;
/// Columns of the diagram: tag bins of width 1 / BINS.
pub const BINS: usize = 100;

/// One agent of the current generation, with this generation's donations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tagger {
    pub id: u64,
    /// The agent whose traits it carries (0 for the first generation).
    pub parent: u64,
    pub tag: f64,
    pub tolerance: f64,
    /// Donations made and received this generation.
    pub given: u32,
    pub received: u32,
}

/// One tag bin of one generation, as the diagram and Inspect show it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bin {
    pub count: u16,
    pub zero: u16,
    pub distinct: u16,
    pub tolerance_sum: f32,
    pub tolerance_min: f32,
    pub tolerance_max: f32,
    pub given: u32,
    pub received: u32,
}

pub type Row = [Bin; BINS];

/// The diagram's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagsMode {
    /// Agents in the bin.
    Count,
    /// The bin's mean tolerance.
    Tolerance,
    /// The bin's share of agents with tolerance ≤ 0.
    Clones,
}

impl std::str::FromStr for TagsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "count" => Self::Count,
            "tolerance" => Self::Tolerance,
            "clones" => Self::Clones,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// The bin of `tag` (tag 1.0 falls in the last).
pub fn bin(tag: f64) -> usize {
    ((tag * BINS as f64) as usize).min(BINS - 1)
}

/// What Inspect shows for a cell of the diagram.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TagsInspection {
    pub site: TagsCell,
    /// The generation of the row (`None` above the first).
    pub generation: Option<u64>,
    /// The bin's tag range.
    pub from: f64,
    pub to: f64,
    pub count: u32,
    pub distinct: u32,
    pub tolerance: Option<ToleranceView>,
    pub given: u32,
    pub received: u32,
    /// The agents in the bin, for the current generation only.
    pub agents: Vec<TaggerView>,
    /// Always null: agents live one generation, so there is nobody to follow.
    pub agent: Option<TaggerView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct TagsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ToleranceView {
    pub min: f64,
    pub mean: f64,
    pub max: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct TaggerView {
    pub id: u64,
    pub parent: u64,
    pub tag: f64,
    pub tolerance: f64,
    pub score: f64,
    pub given: u32,
    pub received: u32,
}

#[derive(Clone)]
pub struct TagsWorld {
    pub config: TagsConfig,
    /// Completed generations after the first (generation 0 is played at setup).
    pub tick: u64,
    agents: Vec<Tagger>,
    rng: SimRng,
    next_id: u64,
    donations: u64,
    pairings: u64,
    takeovers: Takeovers,
    /// The diagram's rows, oldest first (shared by keyframes).
    history: VecDeque<Arc<Row>>,
    pub stats: Stats<TagsSnapshot>,
}

impl TagsWorld {
    /// Generation 0: random tags and tolerances, then its donations.
    pub fn new(config: TagsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let agents = (1..=u64::from(config.agents))
            .map(|id| {
                let tag = rng.gen::<f64>();
                let tolerance = match config.initial_tolerance {
                    InitialTolerance::Uniform => rng.gen::<f64>(),
                    InitialTolerance::Fixed(x) => x,
                };
                Tagger {
                    id,
                    parent: 0,
                    tag,
                    tolerance,
                    given: 0,
                    received: 0,
                }
            })
            .collect();
        let mut world = TagsWorld {
            next_id: u64::from(config.agents) + 1,
            config,
            tick: 0,
            agents,
            rng,
            donations: 0,
            pairings: 0,
            takeovers: Takeovers::default(),
            history: VecDeque::with_capacity(HISTORY),
            stats: Stats::default(),
        };
        world.play();
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> &[Tagger] {
        &self.agents
    }

    /// An agent's score this generation: b per donation received, −c per donation made.
    pub fn score(&self, a: &Tagger) -> f64 {
        f64::from(a.received) * self.config.benefit - f64::from(a.given) * self.config.cost
    }

    /// Whether the run has reached its last generation.
    pub fn is_finished(&self) -> bool {
        self.config.end > 0 && self.tick >= u64::from(self.config.end)
    }

    /// One generation: the next population by selection and mutation, then its donations.
    pub fn step(&mut self) {
        self.reproduce();
        self.play();
        self.tick += 1;
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// A uniformly random agent other than `a`.
    fn other(&mut self, a: usize) -> usize {
        let b = self.rng.gen_range(0..self.agents.len() as u32 - 1) as usize;
        if b >= a {
            b + 1
        } else {
            b
        }
    }

    /// Whether `donor` gives to a recipient with tag `tag`.
    fn gives(&self, donor: &Tagger, tag: f64) -> bool {
        let d = (donor.tag - tag).abs();
        match self.config.donation_test {
            DonationTest::AtMost => d <= donor.tolerance,
            DonationTest::Below => d < donor.tolerance,
        }
    }

    /// Each agent, in order, meets P others drawn with replacement and donates when its test passes.
    fn play(&mut self) {
        for a in &mut self.agents {
            a.given = 0;
            a.received = 0;
        }
        let mut donations = 0;
        for a in 0..self.agents.len() {
            for _ in 0..self.config.pairings {
                let b = self.other(a);
                if self.gives(&self.agents[a], self.agents[b].tag) {
                    self.agents[a].given += 1;
                    self.agents[b].received += 1;
                    donations += 1;
                }
            }
        }
        self.donations = donations;
        self.pairings = self.agents.len() as u64 * u64::from(self.config.pairings);
    }

    /// The next generation: for each agent in order, a random opponent; the
    /// winner's (or, adopting, the chosen) traits pass on, then mutate.
    fn reproduce(&mut self) {
        let scores: Vec<f64> = self.agents.iter().map(|a| self.score(a)).collect();
        // Adopting: the most one donation can move a score apart.
        let span = self.config.benefit + self.config.cost;
        let mut next = Vec::with_capacity(self.agents.len());
        for a in 0..self.agents.len() {
            let b = self.other(a);
            let (sa, sb) = (scores[a], scores[b]);
            let from = match self.config.selection {
                Selection::Tournament if sa > sb => a,
                Selection::Tournament if sb > sa => b,
                Selection::Tournament => match self.config.tie_rule {
                    TieRule::Random => {
                        if self.rng.gen_bool(0.5) {
                            a
                        } else {
                            b
                        }
                    }
                    TieRule::Current => a,
                    TieRule::Other => b,
                },
                Selection::Adopt => {
                    if sb > sa && self.rng.gen::<f64>() * span < sb - sa {
                        b
                    } else {
                        a
                    }
                }
            };
            let parent = self.agents[from];
            let child = self.mutate(parent);
            next.push(child);
        }
        self.agents = next;
    }

    /// An offspring of `parent`: a fresh tag with `tag_mutation`, tag noise,
    /// then Gaussian tolerance noise with `tolerance_mutation`, floored.
    fn mutate(&mut self, parent: Tagger) -> Tagger {
        let c = &self.config;
        let (tag_mutation, tag_noise) = (c.tag_mutation, c.tag_noise);
        let (tolerance_mutation, sd, floor) =
            (c.tolerance_mutation, c.tolerance_sd, c.tolerance_floor);
        let mut tag = parent.tag;
        if self.rng.gen_bool(tag_mutation) {
            tag = self.rng.gen::<f64>();
        }
        if tag_noise > 0.0 {
            tag = (tag + tag_noise * normal(&mut self.rng)).clamp(0.0, 1.0);
        }
        let mut tolerance = parent.tolerance;
        if self.rng.gen_bool(tolerance_mutation) {
            tolerance = (tolerance + sd * normal(&mut self.rng)).max(floor);
        }
        let id = self.next_id;
        self.next_id += 1;
        Tagger {
            id,
            parent: parent.id,
            tag,
            tolerance,
            given: 0,
            received: 0,
        }
    }

    /// Pushes this generation's statistics and diagram row.
    fn record(&mut self) {
        let n = self.agents.len();
        let mut sorted: Vec<(f64, f64)> =
            self.agents.iter().map(|a| (a.tag, a.tolerance)).collect();
        sorted.sort_by(|p, q| p.0.total_cmp(&q.0));
        let c = cluster(&sorted).expect("at least two agents");
        self.takeovers.see(&c, n);
        let mut row: Row = [Bin::default(); BINS];
        for a in &self.agents {
            let b = &mut row[bin(a.tag)];
            let t = a.tolerance as f32;
            if b.count == 0 {
                (b.tolerance_min, b.tolerance_max) = (t, t);
            } else {
                b.tolerance_min = b.tolerance_min.min(t);
                b.tolerance_max = b.tolerance_max.max(t);
            }
            b.count += 1;
            b.zero += u16::from(a.tolerance <= 0.0);
            b.tolerance_sum += t;
            b.given += a.given;
            b.received += a.received;
        }
        let mut distinct = 0;
        for (i, p) in sorted.iter().enumerate() {
            if i == 0 || sorted[i - 1].0 != p.0 {
                row[bin(p.0)].distinct += 1;
                distinct += 1;
            }
        }
        if self.history.len() == HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(Arc::new(row));
        let nf = n as f64;
        self.stats.push(TagsSnapshot {
            tick: self.tick,
            population: n as u32,
            donation_rate: if self.pairings == 0 {
                0.0
            } else {
                self.donations as f64 / self.pairings as f64
            },
            mean_tolerance: sorted.iter().map(|p| p.1).sum::<f64>() / nf,
            cluster_share: f64::from(c.size) / nf,
            relatedness: f64::from(c.modal_count) / f64::from(c.size),
            cluster_tolerance: c.mean_tolerance,
            zero_tolerance_share: sorted.iter().filter(|p| p.1 <= 0.0).count() as f64 / nf,
            distinct_tags: distinct,
            takeovers: self.takeovers.count,
        });
    }

    /// The diagram cell (x, y): bin x of the generation in row y.
    pub fn inspect(&self, x: u32, y: u32) -> Result<TagsInspection, String> {
        if x as usize >= BINS || y as usize >= HISTORY {
            return Err(format!("({x}, {y}) is not in the diagram"));
        }
        let (x, y) = (x as usize, y as usize);
        let blank = HISTORY - self.history.len();
        let from = x as f64 / BINS as f64;
        let to = (x + 1) as f64 / BINS as f64;
        let cell = TagsCell {
            x: x as u32,
            y: y as u32,
        };
        let Some(r) = y.checked_sub(blank) else {
            return Ok(TagsInspection {
                site: cell,
                generation: None,
                from,
                to,
                count: 0,
                distinct: 0,
                tolerance: None,
                given: 0,
                received: 0,
                agents: Vec::new(),
                agent: None,
            });
        };
        let b = self.history[r][x];
        let newest = r + 1 == self.history.len();
        let agents = if newest {
            self.agents
                .iter()
                .filter(|a| bin(a.tag) == x)
                .map(|a| TaggerView {
                    id: a.id,
                    parent: a.parent,
                    tag: a.tag,
                    tolerance: a.tolerance,
                    score: self.score(a),
                    given: a.given,
                    received: a.received,
                })
                .collect()
        } else {
            Vec::new()
        };
        Ok(TagsInspection {
            site: cell,
            generation: Some(self.tick - (self.history.len() - 1 - r) as u64),
            from,
            to,
            count: u32::from(b.count),
            distinct: u32::from(b.distinct),
            tolerance: (b.count > 0).then(|| ToleranceView {
                min: f64::from(b.tolerance_min),
                mean: f64::from(b.tolerance_sum) / f64::from(b.count),
                max: f64::from(b.tolerance_max),
            }),
            given: b.given,
            received: b.received,
            agents,
            agent: None,
        })
    }

    fn color(&self, b: &Bin, mode: TagsMode) -> Rgb {
        if b.count == 0 {
            return BACKGROUND;
        }
        let count = f64::from(b.count);
        let (to, level) = match mode {
            TagsMode::Count => (SUGAR, (count / self.agents.len() as f64).sqrt()),
            TagsMode::Tolerance => (COOL, f64::from(b.tolerance_sum) / count / 0.05),
            TagsMode::Clones => (HOT, f64::from(b.zero) / count),
        };
        lerp(BACKGROUND, to, 0.15 + 0.85 * level.min(1.0))
    }
}

impl Model for TagsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Tags(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        TagsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick, the takeover count, and every agent's id,
    /// parent, tag, tolerance and donations.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        eat(u64::from(self.takeovers.count));
        for a in &self.agents {
            eat(a.id);
            eat(a.parent);
            eat(a.tag.to_bits());
            eat(a.tolerance.to_bits());
            eat((u64::from(a.given) << 32) | u64::from(a.received));
        }
        h
    }

    /// The tag × generation diagram: one column per tag bin, one row per
    /// generation, the current one at the bottom.
    fn size(&self) -> (u32, u32) {
        (BINS as u32, HISTORY as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: TagsMode = mode.parse()?;
        buf.clear();
        buf.resize(BINS * HISTORY * 4, 0);
        let blank = HISTORY - self.history.len();
        for (y, px) in buf.chunks_exact_mut(BINS * 4).enumerate() {
            let row = y.checked_sub(blank).map(|r| &self.history[r]);
            for (x, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let rgb = row.map_or(BACKGROUND, |r| self.color(&r[x], mode));
                p.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        super::SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn latest_value(&self, name: &str) -> Option<f64> {
        use crate::stats::Series;
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,parent,tag,tolerance,score,given,received\n");
        for a in &self.agents {
            writeln!(
                out,
                "{},{},{},{},{},{},{}",
                a.id,
                a.parent,
                a.tag,
                a.tolerance,
                self.score(a),
                a.given,
                a.received
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Agents live one generation; there is nothing to follow.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Tags(next) = next else {
            return Err(wrong_model(ModelKind::Tags, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world of the given `(tag, tolerance)` agents (ids 1, 2, …), no
    /// donations played yet, with `edit` applied to the default config.
    fn world(agents: &[(f64, f64)], edit: impl FnOnce(&mut TagsConfig)) -> TagsWorld {
        let mut c = TagsConfig {
            agents: agents.len() as u32,
            ..TagsConfig::default()
        };
        edit(&mut c);
        let mut w = TagsWorld::new(c, 1).unwrap();
        w.agents = agents
            .iter()
            .enumerate()
            .map(|(k, &(tag, tolerance))| Tagger {
                id: k as u64 + 1,
                parent: 0,
                tag,
                tolerance,
                given: 0,
                received: 0,
            })
            .collect();
        w
    }

    #[test]
    fn identical_tags_donate_under_at_most_and_not_under_below() {
        let w = world(&[(0.5, 0.0), (0.5, 0.0)], |_| {});
        assert!(w.gives(&w.agents[0], 0.5), "|0| ≤ 0");
        assert!(!w.gives(&w.agents[0], 0.5 + 1e-12));
        let strict = world(&[(0.5, 0.0), (0.5, 0.0)], |c| {
            c.donation_test = DonationTest::Below
        });
        assert!(!strict.gives(&strict.agents[0], 0.5), "|0| < 0 is false");
        let floor = world(&[(0.5, -1e-6), (0.5, 0.0)], |_| {});
        assert!(
            !floor.gives(&floor.agents[0], 0.5),
            "a negative tolerance refuses clones"
        );
    }

    #[test]
    fn the_donation_test_is_the_donors_tolerance_around_its_own_tag() {
        let w = world(&[(0.5, 0.1), (0.6, 0.0)], |_| {});
        assert!(w.gives(&w.agents[0], 0.4) && w.gives(&w.agents[0], 0.6));
        assert!(!w.gives(&w.agents[0], 0.61) && !w.gives(&w.agents[1], 0.5));
    }

    #[test]
    fn donations_move_scores_by_benefit_and_cost_and_never_to_oneself() {
        // Two clones with tolerance 0: every pairing is with the other, and gives.
        let mut w = world(&[(0.3, 0.0), (0.3, 0.0)], |c| c.pairings = 4);
        w.play();
        assert_eq!((w.donations, w.pairings), (8, 8));
        for a in &w.agents {
            assert_eq!((a.given, a.received), (4, 4));
            assert!((w.score(a) - (4.0 - 0.4)).abs() < 1e-12);
        }
        let mut none = world(&[(0.3, 0.0), (0.3, 0.0)], |c| c.pairings = 0);
        none.play();
        none.record();
        assert_eq!(none.stats.latest().unwrap().donation_rate, 0.0);
    }

    #[test]
    fn the_higher_score_always_reproduces() {
        for tie in [TieRule::Random, TieRule::Current, TieRule::Other] {
            let mut w = world(&[(0.1, 0.0), (0.9, 0.0)], |c| {
                c.tie_rule = tie;
                c.tag_mutation = 0.0;
                c.tolerance_mutation = 0.0;
            });
            w.agents[1].received = 1;
            w.reproduce();
            assert!(
                w.agents.iter().all(|a| a.tag == 0.9 && a.parent == 2),
                "{tie:?}"
            );
        }
    }

    #[test]
    fn ties_go_by_the_tie_rule() {
        let quiet = |c: &mut TagsConfig| {
            c.tag_mutation = 0.0;
            c.tolerance_mutation = 0.0;
        };
        let tags: Vec<(f64, f64)> = (0..50).map(|k| (f64::from(k) / 50.0, 0.0)).collect();
        // Current: with every score equal each agent passes on its own traits.
        let mut w = world(&tags, |c| {
            quiet(c);
            c.tie_rule = TieRule::Current;
        });
        w.reproduce();
        let parents: Vec<u64> = w.agents.iter().map(|a| a.parent).collect();
        assert_eq!(parents, (1..=50).collect::<Vec<_>>());
        // Other: each passes on its opponent's, never its own.
        let mut w = world(&tags, |c| {
            quiet(c);
            c.tie_rule = TieRule::Other;
        });
        w.reproduce();
        assert!(w
            .agents
            .iter()
            .enumerate()
            .all(|(k, a)| a.parent != k as u64 + 1));
        // Random: some of each.
        let mut w = world(&tags, |c| {
            quiet(c);
            c.tie_rule = TieRule::Random;
        });
        w.reproduce();
        let own = w
            .agents
            .iter()
            .enumerate()
            .filter(|(k, a)| a.parent == *k as u64 + 1)
            .count();
        assert!((10..=40).contains(&own), "{own}");
    }

    #[test]
    fn adopting_takes_better_traits_in_proportion_to_b_plus_c() {
        let quiet = |c: &mut TagsConfig| {
            c.selection = Selection::Adopt;
            c.tag_mutation = 0.0;
            c.tolerance_mutation = 0.0;
        };
        // Agent 2 is better by b + c or more: always adopted; agent 2 never adopts worse.
        let mut w = world(&[(0.1, 0.0), (0.9, 0.0)], quiet);
        w.agents[1].received = 2;
        w.reproduce();
        assert_eq!((w.agents[0].parent, w.agents[1].parent), (2, 2));
        // Equal scores: nobody adopts.
        let mut w = world(&[(0.1, 0.0), (0.9, 0.0)], quiet);
        w.reproduce();
        assert_eq!((w.agents[0].parent, w.agents[1].parent), (1, 2));
        // Better by b = 1 of b + c = 1.1: adopted about 91% of the time.
        let mut adopted = 0;
        for seed in 0..400 {
            let mut w = world(&[(0.1, 0.0), (0.9, 0.0)], quiet);
            w.rng = rng::seeded(seed);
            w.agents[1].received = 1;
            w.reproduce();
            adopted += usize::from(w.agents[0].parent == 2);
        }
        assert!((340..=390).contains(&adopted), "{adopted}");
    }

    #[test]
    fn mutation_redraws_tags_adds_noise_and_floors_tolerance() {
        let mut w = world(&[(0.5, 0.0), (0.5, 0.0)], |c| {
            c.tag_mutation = 1.0;
            c.tolerance_mutation = 1.0;
            c.tolerance_sd = 1.0;
            c.tolerance_floor = -1e-6;
        });
        let parent = w.agents[0];
        let kids: Vec<Tagger> = (0..200).map(|_| w.mutate(parent)).collect();
        assert!(kids.iter().all(|k| (0.0..1.0).contains(&k.tag)));
        assert!(kids.iter().any(|k| k.tag != 0.5));
        assert!(kids.iter().all(|k| k.tolerance >= -1e-6));
        assert!(
            kids.iter().any(|k| k.tolerance == -1e-6),
            "clamped to the floor"
        );
        assert!(kids.windows(2).all(|p| p[1].id == p[0].id + 1));
        let mut noisy = world(&[(1.0, 0.0), (0.0, 0.0)], |c| {
            c.tag_mutation = 0.0;
            c.tolerance_mutation = 0.0;
            c.tag_noise = 0.5;
        });
        let kids: Vec<Tagger> = (0..100).map(|_| noisy.mutate(noisy.agents[0])).collect();
        assert!(kids.iter().all(|k| (0.0..=1.0).contains(&k.tag)));
        assert!(kids.iter().any(|k| k.tag == 1.0) && kids.iter().any(|k| k.tag < 1.0));
        assert!(
            kids.iter().all(|k| k.tolerance == 0.0),
            "no tolerance mutation"
        );
    }

    #[test]
    fn a_step_replaces_every_agent_and_records_a_row() {
        let mut w = TagsWorld::new(TagsConfig::default(), 3).unwrap();
        let first: Vec<u64> = w.agents.iter().map(|a| a.id).collect();
        assert_eq!(first, (1..=100).collect::<Vec<_>>());
        assert_eq!((w.stats.history().len(), w.history.len()), (1, 1));
        let d0 = w.stats.latest().unwrap().donation_rate;
        assert!(
            (0.5..0.8).contains(&d0),
            "uniform tolerances give about 2/3: {d0}"
        );
        w.step();
        assert_eq!(w.tick, 1);
        assert!(w
            .agents
            .iter()
            .all(|a| (101..=200).contains(&a.id) && (1..=100).contains(&a.parent)));
        assert_eq!(w.stats.history().len(), 2);
    }

    #[test]
    fn the_run_stops_at_its_last_generation() {
        let mut w = TagsWorld::new(
            TagsConfig {
                end: 5,
                ..TagsConfig::default()
            },
            1,
        )
        .unwrap();
        w.run(10);
        assert_eq!(w.tick, 5);
        assert!(Model::finished(&w));
        let mut forever = TagsWorld::new(
            TagsConfig {
                end: 0,
                ..TagsConfig::default()
            },
            1,
        )
        .unwrap();
        forever.run(10);
        assert!(!Model::finished(&forever));
    }

    #[test]
    fn the_diagram_bins_tags_and_keeps_the_last_200_generations() {
        assert_eq!(
            (bin(0.0), bin(0.0099), bin(0.01), bin(0.999), bin(1.0)),
            (0, 0, 1, 99, 99)
        );
        let mut w = TagsWorld::new(TagsConfig::default(), 2).unwrap();
        let mut buf = Vec::new();
        w.render("count", "", &mut buf).unwrap();
        assert_eq!(buf.len(), BINS * HISTORY * 4);
        assert_eq!(&buf[..3], &BACKGROUND, "rows before generation 0 are dark");
        let bottom = &buf[(HISTORY - 1) * BINS * 4..];
        assert!(bottom
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[..3] != BACKGROUND));
        assert!(w.render("wealth", "", &mut buf).is_err());
        w.run(250);
        assert_eq!(w.history.len(), HISTORY);
        let top = w.inspect(0, 0).unwrap();
        assert_eq!(top.generation, Some(51));
        let row: u32 = w.history[HISTORY - 1]
            .iter()
            .map(|b| u32::from(b.count))
            .sum();
        assert_eq!(row, 100);
        assert_eq!(Model::locate(&w, 1), None);
    }

    #[test]
    fn inspect_shows_a_bin_and_its_agents_in_the_newest_row() {
        let mut w = world(
            &[(0.505, 0.0), (0.505, 0.02), (0.5, 0.01), (0.9, 0.0)],
            |_| {},
        );
        w.play();
        w.record();
        let y = HISTORY as u32 - 1;
        let v = w.inspect(50, y).unwrap();
        assert_eq!((v.count, v.distinct, v.agents.len()), (3, 2, 3));
        assert_eq!((v.from, v.to), (0.5, 0.51));
        let t = v.tolerance.unwrap();
        assert!((t.min, t.max) == (0.0, 0.02_f32 as f64) && (t.mean - 0.01).abs() < 1e-6);
        assert_eq!(v.given, v.agents.iter().map(|a| a.given).sum::<u32>());
        let empty = w.inspect(10, y).unwrap();
        assert_eq!((empty.count, empty.tolerance), (0, None));
        assert_eq!(w.inspect(50, 0).unwrap().generation, None);
        assert!(w.inspect(100, 0).is_err() && w.inspect(0, 200).is_err());
    }

    #[test]
    fn keyframes_keep_the_diagram() {
        let mut any =
            crate::model::ModelWorld::new(ModelConfig::Tags(TagsConfig::default()), 4).unwrap();
        any.model_mut().run(20);
        let cp = any.checkpoint().unwrap();
        let (mut at20, mut again) = (Vec::new(), Vec::new());
        any.model().render("count", "", &mut at20).unwrap();
        let print = any.model().fingerprint();
        any.model_mut().run(30);
        any.restore(&cp).unwrap();
        any.model().render("count", "", &mut again).unwrap();
        assert_eq!(at20, again);
        assert_eq!(any.model().fingerprint(), print);
        assert_eq!(any.model().series("donation_rate").unwrap().len(), 21);
    }

    #[test]
    fn statistics_follow_the_population() {
        let mut w = world(
            &[(0.2, 0.0), (0.2, 0.0), (0.205, 0.04), (0.8, 0.02)],
            |_| {},
        );
        w.play();
        w.record();
        let s = w.stats.latest().unwrap();
        assert_eq!((s.cluster_share, s.relatedness), (0.75, 2.0 / 3.0));
        assert_eq!((s.zero_tolerance_share, s.distinct_tags), (0.5, 3));
        assert!((s.mean_tolerance - 0.015).abs() < 1e-12);
        assert!((s.cluster_tolerance - 0.04 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            TagsConfig {
                agents: 2,
                ..TagsConfig::default()
            },
            TagsConfig {
                pairings: 0,
                ..TagsConfig::default()
            },
            TagsConfig {
                cost: 0.0,
                benefit: 0.0,
                selection: Selection::Adopt,
                ..TagsConfig::default()
            },
            TagsConfig {
                tag_mutation: 1.0,
                tolerance_mutation: 1.0,
                tolerance_sd: 1.0,
                tag_noise: 1.0,
                tolerance_floor: -1.0,
                ..TagsConfig::default()
            },
        ] {
            let mut w = TagsWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            let s = w.stats.latest().unwrap();
            assert_eq!(w.tick, 50, "{c:?}");
            assert!((0.0..=1.0).contains(&s.donation_rate), "{c:?}");
            assert!(s.cluster_share > 0.0 && s.relatedness > 0.0, "{c:?}");
        }
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = TagsWorld::new(TagsConfig::default(), 1).unwrap();
        let mut next = TagsConfig {
            pairings: 5,
            tie_rule: TieRule::Current,
            ..TagsConfig::default()
        };
        Model::set_config(&mut w, ModelConfig::Tags(next.clone())).unwrap();
        w.step();
        assert_eq!(w.pairings, 500);
        next.agents = 50;
        let e = Model::set_config(&mut w, ModelConfig::Tags(next)).unwrap_err();
        assert_eq!(e[0].field, "agents");
    }
}
