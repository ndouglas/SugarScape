//! Interactive edits and inspection used by the playground UI.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::agent::{Agent, AgentId, DiseaseId, Sex, Tribe};
use crate::config::{Config, FieldError};
use crate::geometry::Pos;
use crate::minds::memory::believed_ripe;
use crate::rules;
use crate::world::{LoanId, World};

/// Why `place_agent` and `remove_agent` refuse a lab world (Minds 5).
const LAB_ROSTER: &str = "the lab's roster is fixed";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct AgentOverrides {
    pub vision: Option<u32>,
    pub metabolism: Option<u32>,
    pub sugar: Option<f64>,
    pub sex: Option<Sex>,
    pub tribe: Option<Tribe>,
    /// Good 1 (spice) held and needed; two-good worlds only.
    pub spice: Option<f64>,
    pub spice_metabolism: Option<u32>,
    /// Age at placement, and a birth endowment of sugar other than what it
    /// holds (credit's borrowers are those short of theirs).
    pub age: Option<u32>,
    pub endowment: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteView {
    pub x: u32,
    pub y: u32,
    /// Level of each good.
    pub resources: Vec<f64>,
    /// Capacity of each good.
    pub capacities: Vec<f64>,
    /// Level of each pollutant.
    pub pollution: Vec<f64>,
    /// Minds 5: the world's own wall state at this site right now (`0` free,
    /// `1` a fence, `2` opaque), not `config.walls`. In a lab world a
    /// doorway is `2` until the test evening opens it (`World::open_wall`),
    /// so this is how Inspect and drawing see it change; everywhere else
    /// walls change only on reset and this matches `config.walls`.
    pub wall: u8,
    /// Minds 5–6: every cache buried here, in owner-id order.
    pub caches: Vec<SiteCacheView>,
}

/// Minds 5–6: one cache at a site, for a site's Inspect.
#[derive(Clone, Debug, Serialize)]
pub struct SiteCacheView {
    pub owner: AgentId,
    pub amount: f64,
    /// Whether its owner is a Minds 6 cheater (`Agent.cheater`).
    pub cheater_owner: bool,
}

/// Minds 5–6: what the page draws of a Minds world beyond the frame, for
/// every agent at once (`World::minds_view`). Cheap: one pass over the
/// agents' caches, sized by the sites that hold any.
#[derive(Clone, Debug, Default, Serialize)]
pub struct MindsView {
    /// `Some` while `seasons.mode` is global: whether the tick just computed
    /// (`tick − 1`, the one the frame shows) was a winter tick
    /// (`growback::is_winter`). `None` with seasons off or by hemisphere.
    pub winter: Option<bool>,
    /// Every site holding a cache, in site order: `[x, y, total, flags]`,
    /// the flags' [`CACHE_HOARDER`] bit set when a hoarder (not a cheater)
    /// owns a cache there, [`CACHE_CHEATER`] when a cheater does, and
    /// [`CACHE_LARDER`] when one is its owner's larder (the cache at its
    /// home, in a central world).
    pub caches: Vec<[f64; 4]>,
    /// Central worlds: every agent's home and larder, in id order.
    pub homes: Vec<HomeView>,
    /// Lab worlds: the schedule as of the tick just computed.
    pub lab: Option<LabView>,
}

/// Cache flag bits in `MindsView::caches`.
pub const CACHE_HOARDER: u32 = 1;
pub const CACHE_CHEATER: u32 = 2;
pub const CACHE_LARDER: u32 = 4;

/// Minds 5: a central-place forager's home and what its larder holds.
#[derive(Clone, Debug, Serialize)]
pub struct HomeView {
    pub id: AgentId,
    pub x: u32,
    pub y: u32,
    pub larder: f64,
}

/// Minds 5: where a lab world is in its schedule (`minds::caching::lab`),
/// as of the tick just computed (`tick − 1`: the frame shows its end).
#[derive(Clone, Debug, Serialize)]
pub struct LabView {
    pub protocol: crate::config::LabProtocol,
    /// Training days before the test evening.
    pub days: u64,
    /// `start` (nothing run yet), `morning`, `evening`, `test` (the test
    /// evening, agents taking turns) or `done` (every agent finished).
    pub phase: &'static str,
    /// The day, from 1; `days + 1` on the test evening.
    pub day: u64,
    /// The day's compartment (0–2 for K1–K3), during training.
    pub place: Option<u32>,
    /// Whether the day's compartment had food, during training.
    pub food: Option<bool>,
    /// On the test evening, the agent whose turn it is (the next to move),
    /// and where it stands.
    pub turn: Option<AgentId>,
    pub turn_at: Option<[u32; 2]>,
    /// K1–K3: each compartment's `[x, y, width, height]`.
    pub compartments: Vec<[u32; 4]>,
    /// The trays of the protocol's caching compartments: `[k, x, y]`.
    pub trays: Vec<[u32; 3]>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinkView {
    pub id: AgentId,
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiseaseView {
    pub id: DiseaseId,
    pub bits: String,
    /// Smallest Hamming distance between the disease and a window of the
    /// agent's immune string.
    pub distance: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoanView {
    pub id: LoanId,
    pub role: &'static str,
    pub good: usize,
    pub counterparty: LinkView,
    pub due: f64,
    pub due_tick: u64,
}

/// Minds 2: an agent's plan, for display.
#[derive(Clone, Debug, Serialize)]
pub struct PlanView {
    pub target_x: u32,
    pub target_y: u32,
    pub path: Vec<[u32; 2]>,
    /// The agent walked (or tried to) rather than jumped.
    pub walked: bool,
}

/// Minds 4: a GOAP agent's plan, for Inspect.
#[derive(Clone, Debug, Serialize)]
pub struct GoapView {
    /// The targets left, in order.
    pub steps: Vec<[u32; 2]>,
    /// What the plan was to gather in all.
    pub gathers: f64,
    /// The goal G it planned for.
    pub goal: f64,
}

/// Minds 5: one of an agent's caches, for display.
#[derive(Clone, Debug, Serialize)]
pub struct CacheView {
    pub x: u32,
    pub y: u32,
    pub amount: f64,
}

/// Minds 5: an agent's caching state, for Inspect.
#[derive(Clone, Debug, Serialize)]
pub struct CachingView {
    /// The caching rule this agent follows (`caching.rule`, or its own
    /// under `caching.mixed`; `rules::rule_of`).
    pub rule: crate::config::CachingRule,
    /// The carrying limit (`caching.capacity`); 0 for none.
    pub holdings_cap: f64,
    /// This agent's own caches, in site order.
    pub caches: Vec<CacheView>,
    /// Σ of `caches`' amounts.
    pub total: f64,
    /// Rule `plan`'s current forecast shortfall (burn × γ − forecast −
    /// cached; `rules::shortfall`): `Some` only under rule `plan`, with
    /// `seasons.mode: global` on, and outside winter (when the rule isn't
    /// computing one); `None` otherwise, including for every other rule.
    pub forecast: Option<f64>,
    /// A lab world's test evening: the allocation of F frozen at its start
    /// (`Agent.lab_allocation`), `[compartment, amount]` in K order; `None`
    /// before then and outside a lab.
    pub lab_allocation: Option<Vec<(u32, f64)>>,
}

/// Minds 6: an agent's theft state, for Inspect.
#[derive(Clone, Debug, Serialize)]
pub struct TheftView {
    /// Whether this agent is a cheater (never buries, pilfers what it finds).
    pub cheater: bool,
    /// Running total of sugar it has pilfered from others' caches.
    pub stolen_by_me: f64,
    /// Running total of sugar thieves have taken from its caches.
    pub stolen_from_me: f64,
    /// Its stomach: loot eaten under `theft.loot: eat` and not yet burned.
    pub fed: f64,
}

/// Minds 5: a central-place forager's state, for Inspect.
#[derive(Clone, Debug, Serialize)]
pub struct CentralView {
    /// Where the agent forages from and delivers to.
    pub home: [u32; 2],
    /// The load delivered on its last trip that delivered anything
    /// (`Agent.last_load`); 0 before a first delivery.
    pub last_load: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiseaseEntry {
    pub id: DiseaseId,
    pub bits: String,
    /// Living agents carrying it.
    pub carriers: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentView {
    pub id: AgentId,
    pub x: u32,
    pub y: u32,
    pub sex: Sex,
    pub tribe: Tribe,
    /// Index of the agent's group in `culture.groups`.
    pub group: usize,
    pub tags: String,
    pub vision: u32,
    /// Holdings, birth endowment and metabolism of each good.
    pub holdings: Vec<f64>,
    pub initial: Vec<f64>,
    pub metabolism: Vec<u32>,
    pub age: u32,
    pub max_age: u32,
    pub fertile: bool,
    pub fertility_onset: u32,
    pub fertility_end: u32,
    pub born: u64,
    pub parents: Vec<LinkView>,
    pub children: Vec<LinkView>,
    pub foresight: u32,
    pub loans: Vec<LoanView>,
    pub immune: String,
    pub immune_genome: String,
    pub diseases: Vec<DiseaseView>,
    pub infected_by: Option<LinkView>,
    /// Minds 2: where the agent is walking and the path left to it. `None`
    /// until the agent first moves; its path is empty under `jump` or once
    /// the agent has arrived.
    pub plan: Option<PlanView>,
    /// Minds 3: whether the agent remembers, and how many sites and truffle
    /// spots it holds in memory. `None` while memory is off (`span` 0).
    pub memory: Option<MemoryView>,
    /// Minds 4: the GOAP plan. `Some` only under `decision.rule: goap` while
    /// the agent holds a plan: its steps are empty once the plan is done
    /// (or was empty, G = 0). `None` after a dropped plan or a fallback, so
    /// no stale figures show.
    pub goap: Option<GoapView>,
    /// Minds 4: the agent's running intake-rate estimate ρ. `Some` only
    /// under `decision.rule: mvt`.
    pub rate: Option<f64>,
    /// Minds 5: caching state. `Some` only while `caching.is_on()`.
    pub caching: Option<CachingView>,
    /// Minds 5: central-place foraging state. `Some` only while
    /// `central.enabled`.
    pub central: Option<CentralView>,
    /// Minds 6: theft state. `Some` only while `theft.is_on()`.
    pub theft: Option<TheftView>,
}

/// Minds 3: what an agent remembers, for display.
#[derive(Clone, Debug, Serialize)]
pub struct MemoryView {
    pub remembers: bool,
    /// Sites in its memory (forgotten ones may linger until the next sweep).
    pub sites: u32,
    /// Of those, the ones where it knows a truffle spot.
    pub spots: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Inspection {
    pub site: SiteView,
    pub agent: Option<AgentView>,
}

impl World {
    fn checked_pos(&self, x: u32, y: u32) -> Result<Pos, String> {
        if x < self.torus.width && y < self.torus.height {
            Ok(Pos::new(x, y))
        } else {
            Err(format!("({x}, {y}) is outside the grid"))
        }
    }

    /// Sets good `good`'s capacity to `value` on every site within Euclidean
    /// `radius` of (x, y) (wrapping), clamping its level to the new capacity.
    pub fn paint_capacity(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        value: f64,
        good: usize,
    ) -> Result<(), String> {
        let center = self.checked_pos(x, y)?;
        if good >= self.config.goods.len() {
            return Err(format!("there is no good {good}"));
        }
        if !(value.is_finite() && value >= 0.0) {
            return Err("capacity must be ≥ 0".into());
        }
        let r = radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let pos = self.torus.offset(center, dx, dy);
                if self.is_wall(pos) {
                    continue;
                }
                let site = self.site_mut(pos);
                site.capacity[good] = value;
                site.resource[good] = site.resource[good].min(value);
            }
        }
        Ok(())
    }

    /// Replaces good `good`'s capacities with `capacities` (row-major, one per
    /// site, each 0–10 — an imported image), clamping each site's level to its
    /// new capacity as painting does. The good then counts as edited
    /// (`landscape_edited`), so share links, export and reset keep it.
    pub fn set_capacities(&mut self, good: usize, capacities: &[f64]) -> Result<(), String> {
        if good >= self.config.goods.len() {
            return Err(format!("there is no good {good}"));
        }
        if capacities.len() != self.sites.len() {
            return Err(format!(
                "expected {} capacities, got {}",
                self.sites.len(),
                capacities.len()
            ));
        }
        if let Some(bad) = capacities.iter().find(|c| !(0.0..=10.0).contains(*c)) {
            return Err(format!("capacities must be between 0 and 10 (got {bad})"));
        }
        let walls = &self.walls;
        for ((i, site), &c) in self.sites.iter_mut().enumerate().zip(capacities) {
            if walls[i] != 0 {
                continue;
            }
            site.capacity[good] = c;
            site.resource[good] = site.resource[good].min(c);
        }
        Ok(())
    }

    /// Refused in a lab world ("the lab's roster is fixed"): the lab's
    /// schedule places its agents by roster.
    pub fn place_agent(&mut self, x: u32, y: u32, o: &AgentOverrides) -> Result<AgentId, String> {
        if self.config.lab.is_some() {
            return Err(LAB_ROSTER.into());
        }
        let pos = self.checked_pos(x, y)?;
        if (o.spice.is_some() || o.spice_metabolism.is_some()) && self.config.goods.len() < 2 {
            return Err("spice needs a second good".into());
        }
        let mut agent = Agent::random(&self.config, pos, self.tick, &mut self.rng);
        if let Some(v) = o.vision {
            agent.vision = v;
        }
        if let Some(m) = o.metabolism {
            agent.metabolism[0] = m;
        }
        if let Some(s) = o.sugar {
            agent.holdings[0] = s;
            agent.initial[0] = s;
        }
        if let Some(s) = o.spice {
            agent.holdings[1] = s;
            agent.initial[1] = s;
        }
        if let Some(m) = o.spice_metabolism {
            agent.metabolism[1] = m;
        }
        if let Some(age) = o.age {
            agent.age = age;
        }
        if let Some(e) = o.endowment {
            agent.initial[0] = e;
        }
        if let Some(sex) = o.sex {
            agent.sex = sex;
            agent.fertility_end = self.config.sex.end_for(sex).sample(&mut self.rng);
        }
        if let Some(t) = o.tribe {
            agent.tags = agent.tags.forced_to(t);
        }
        crate::rules::disease::endow(self, &mut agent);
        self.insert_agent(agent)
    }

    /// Removes the agent at (x, y) without counting a death or bequeathing
    /// its sugar. Refused in a lab world ("the lab's roster is fixed").
    pub fn remove_agent(&mut self, x: u32, y: u32) -> Result<(), String> {
        if self.config.lab.is_some() {
            return Err(LAB_ROSTER.into());
        }
        let pos = self.checked_pos(x, y)?;
        let id = self
            .occupant(pos)
            .ok_or_else(|| format!("no agent at ({x}, {y})"))?;
        self.remove(id);
        Ok(())
    }

    pub fn locate(&self, id: AgentId) -> Option<Pos> {
        self.agent(id).map(|a| a.pos)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<Inspection, String> {
        let pos = self.checked_pos(x, y)?;
        let s = self.site(pos);
        let (n, m) = (
            self.config.goods.len(),
            self.config.pollution.pollutants.len(),
        );
        let link = |id: AgentId| LinkView {
            id,
            alive: self.agent(id).is_some(),
        };
        let agent = self.agent_at(pos).map(|a| AgentView {
            id: a.id,
            x: a.pos.x,
            y: a.pos.y,
            sex: a.sex,
            tribe: a.tribe(),
            group: a.group(&self.config.culture.groups),
            tags: a.tags.to_bit_string(),
            vision: a.vision,
            holdings: a.holdings[..n].to_vec(),
            initial: a.initial[..n].to_vec(),
            metabolism: a.metabolism[..n].to_vec(),
            age: a.age,
            max_age: a.max_age,
            fertile: a.is_fertile(self.config.sex.fertile_wealth, self.config.goods.len()),
            fertility_onset: a.fertility_onset,
            fertility_end: a.fertility_end,
            born: a.born,
            parents: a
                .parents
                .map(|p| p.iter().map(|&id| link(id)).collect())
                .unwrap_or_default(),
            children: a.children.iter().map(|&id| link(id)).collect(),
            foresight: a.foresight,
            loans: self
                .loans()
                .filter(|l| l.lender == a.id || l.borrower == a.id)
                .map(|l| {
                    let lender = l.lender == a.id;
                    LoanView {
                        id: l.id,
                        role: if lender { "lender" } else { "borrower" },
                        good: l.good,
                        counterparty: link(if lender { l.borrower } else { l.lender }),
                        due: l.due,
                        due_tick: l.due_tick,
                    }
                })
                .collect(),
            immune: a.immune.to_bit_string(),
            immune_genome: a.immune_genome.to_bit_string(),
            diseases: a
                .diseases
                .iter()
                .map(|&id| {
                    let d = self.diseases[id as usize];
                    DiseaseView {
                        id,
                        bits: d.to_bit_string(),
                        distance: a
                            .immune
                            .closest_window(&d)
                            .map_or(d.len(), |(_, distance)| distance),
                    }
                })
                .collect(),
            infected_by: a.infected_by.map(link),
            plan: a.plan.target.map(|t| PlanView {
                target_x: t.x,
                target_y: t.y,
                path: a.plan.path.iter().map(|p| [p.x, p.y]).collect(),
                walked: a.plan.walked,
            }),
            memory: (self.config.memory.span > 0).then(|| MemoryView {
                remembers: a.remembers,
                sites: a.memory.sites.len() as u32,
                spots: a
                    .memory
                    .sites
                    .values()
                    .filter(|s| s.truffle.is_some())
                    .count() as u32,
            }),
            goap: a
                .goap_plan
                .as_ref()
                .filter(|_| self.config.decision.rule == crate::config::DecisionRule::Goap)
                .map(|g| GoapView {
                    steps: g.steps.iter().map(|(p, _)| [p.x, p.y]).collect(),
                    gathers: g.gathers,
                    goal: g.goal,
                }),
            rate: (self.config.decision.rule == crate::config::DecisionRule::Mvt).then_some(a.rate),
            caching: self.config.caching.is_on().then(|| {
                let rule = crate::minds::caching::rules::rule_of(self, a.id);
                let caches: Vec<CacheView> = a
                    .caches
                    .iter()
                    .map(|(&site, &amount)| {
                        let p = self.torus.pos(site as usize);
                        CacheView {
                            x: p.x,
                            y: p.y,
                            amount,
                        }
                    })
                    .collect();
                let total = a.caches.values().sum();
                let seasons = self.config.seasons;
                let forecast = (rule == crate::config::CachingRule::Plan
                    && seasons.enabled
                    && seasons.mode == crate::config::SeasonMode::Global
                    && !rules::growback::is_winter(&self.config, self.tick))
                .then(|| {
                    let fee = self.config.disease.active_fee();
                    let burn = a.effective_metabolism(0, fee);
                    crate::minds::caching::rules::shortfall(
                        burn,
                        seasons.period,
                        a.last_winter.as_ref(),
                        total,
                    )
                });
                CachingView {
                    rule,
                    holdings_cap: f64::from(self.config.caching.capacity),
                    caches,
                    total,
                    forecast,
                    lab_allocation: a.lab_allocation.clone(),
                }
            }),
            central: self.config.central.enabled.then(|| CentralView {
                home: a.home.map_or([a.pos.x, a.pos.y], |p| [p.x, p.y]),
                last_load: a.last_load,
            }),
            theft: self.config.theft.is_on().then_some(TheftView {
                cheater: a.cheater,
                stolen_by_me: a.stolen_by_me,
                stolen_from_me: a.stolen_from_me,
                fed: a.fed,
            }),
        });
        Ok(Inspection {
            site: SiteView {
                x,
                y,
                resources: s.resource[..n].to_vec(),
                capacities: s.capacity[..n].to_vec(),
                pollution: s.pollution[..m].to_vec(),
                wall: self.walls[self.torus.index(pos)],
                caches: {
                    let site = self.torus.index(pos) as u32;
                    self.agents()
                        .filter_map(|a| {
                            a.caches.get(&site).map(|&amount| SiteCacheView {
                                owner: a.id,
                                amount,
                                cheater_owner: a.cheater,
                            })
                        })
                        .collect()
                },
            },
            agent,
        })
    }

    /// Minds 5–6: the page's view of every cache, home and larder, the
    /// season and the lab's schedule (see [`MindsView`]).
    pub fn minds_view(&self) -> MindsView {
        use crate::minds::caching::lab;
        let seasons = self.config.seasons;
        let winter = (seasons.enabled && seasons.mode == crate::config::SeasonMode::Global)
            .then(|| rules::growback::is_winter(&self.config, self.tick.saturating_sub(1)));
        let central = self.config.central.enabled;
        let mut sites: std::collections::BTreeMap<u32, (f64, u32)> =
            std::collections::BTreeMap::new();
        let mut homes = Vec::new();
        for a in self.agents() {
            let home = a
                .home
                .filter(|_| central)
                .map(|p| self.torus.index(p) as u32);
            for (&site, &amount) in &a.caches {
                let e = sites.entry(site).or_insert((0.0, 0));
                e.0 += amount;
                e.1 |= if a.cheater {
                    CACHE_CHEATER
                } else {
                    CACHE_HOARDER
                };
                if home == Some(site) {
                    e.1 |= CACHE_LARDER;
                }
            }
            if let (Some(p), Some(site)) = (a.home, home) {
                homes.push(HomeView {
                    id: a.id,
                    x: p.x,
                    y: p.y,
                    larder: a.caches.get(&site).copied().unwrap_or(0.0),
                });
            }
        }
        let caches = sites
            .into_iter()
            .map(|(site, (total, flags))| {
                let p = self.torus.pos(site as usize);
                [f64::from(p.x), f64::from(p.y), total, f64::from(flags)]
            })
            .collect();
        let lab = self.config.lab.map(|l| {
            let days = lab::training_days(l.protocol);
            let (mut phase, mut day, mut place, mut food) = ("start", 1, None, None);
            if self.tick > 0 {
                let t = self.tick - 1;
                let d = t / lab::DAY;
                day = d + 1;
                if d < days {
                    phase = if t % lab::DAY < lab::MORNING {
                        "morning"
                    } else {
                        "evening"
                    };
                    place = Some(lab::place(l.protocol, d));
                    food = Some(lab::has_food(l, d));
                } else {
                    day = days + 1;
                    phase = if lab::finished(self) { "done" } else { "test" };
                }
            }
            let turn = (phase == "test").then(|| lab::turn(self)).flatten();
            LabView {
                protocol: l.protocol,
                days,
                phase,
                day,
                place,
                food,
                turn,
                turn_at: turn
                    .and_then(|id| self.agent(id))
                    .map(|a| [a.pos.x, a.pos.y]),
                compartments: (0..3).map(lab::compartment).collect(),
                trays: lab::caching_places(l.protocol)
                    .iter()
                    .map(|&k| {
                        let p = lab::tray(k);
                        [k, p.x, p.y]
                    })
                    .collect(),
            }
        });
        MindsView {
            winter,
            caches,
            homes,
            lab,
        }
    }

    /// Minds 3: the agent at `pos`'s remembered sites, in the memory's own
    /// (site-index) order: each site's `(x, y)`, its age in ticks since last
    /// seen (capped at `u32::MAX`), and its `spot` — 0 for no known truffle
    /// spot, 1 for one believed unripe, 2 for one believed ripe
    /// (`minds::memory::believed_ripe`). Empty when there's no agent there,
    /// it doesn't remember, or memory is off (`span` 0).
    pub fn memory_view(&self, pos: Pos) -> Vec<[u32; 4]> {
        if self.config.memory.span == 0 {
            return Vec::new();
        }
        let Some(agent) = self.agent_at(pos) else {
            return Vec::new();
        };
        if !agent.remembers {
            return Vec::new();
        }
        let now = self.tick;
        agent
            .memory
            .sites
            .iter()
            .map(|(&idx, seen)| {
                let p = self.torus.pos(idx as usize);
                let age = now.saturating_sub(seen.tick).min(u64::from(u32::MAX)) as u32;
                let spot = match &seen.truffle {
                    None => 0,
                    Some(t) => {
                        if believed_ripe(
                            t,
                            now,
                            self.config.truffles.regrow,
                            self.config.memory.belief,
                        ) {
                            2
                        } else {
                            1
                        }
                    }
                };
                [p.x, p.y, age, spot]
            })
            .collect()
    }

    /// Swaps in a new config mid-run. Rule toggles and parameters take effect
    /// on the next tick; grid size, tag length and landscape need a reset.
    /// Only schedule entries that have not fired yet are validated.
    pub fn set_config(&mut self, next: Config) -> Result<(), Vec<FieldError>> {
        next.validate_with_schedule_from(self.tick)?;
        let structural = self.config.structural_changes(&next);
        if !structural.is_empty() {
            return Err(structural);
        }
        self.config = next;
        Ok(())
    }

    fn disease_on(&self) -> Result<(), String> {
        if self.config.disease.enabled {
            Ok(())
        } else {
            Err("disease is off".into())
        }
    }

    /// The master disease list with each disease's carriers.
    pub fn disease_list(&self) -> Vec<DiseaseEntry> {
        let mut carriers = vec![0u32; self.diseases.len()];
        for a in self.agents() {
            for &d in &a.diseases {
                carriers[d as usize] += 1;
            }
        }
        self.diseases
            .iter()
            .zip(carriers)
            .enumerate()
            .map(|(i, (d, carriers))| DiseaseEntry {
                id: i as DiseaseId,
                bits: d.to_bit_string(),
                carriers,
            })
            .collect()
    }

    /// Infects the agent at (x, y) with listed `disease`, or — when `disease`
    /// is negative — with a brand-new random disease (appended to the list even
    /// if the agent resists it). No effect if the agent is immune or already
    /// carries it. Returns whether it was infected.
    pub fn infect(&mut self, x: u32, y: u32, disease: i64) -> Result<bool, String> {
        self.disease_on()?;
        let pos = self.checked_pos(x, y)?;
        let id = self
            .occupant(pos)
            .ok_or_else(|| format!("no agent at ({x}, {y})"))?;
        let d = if disease < 0 {
            rules::disease::new_random(self)
        } else {
            DiseaseId::try_from(disease)
                .ok()
                .filter(|&d| (d as usize) < self.diseases.len())
                .ok_or_else(|| format!("unknown disease {disease}"))?
        };
        Ok(rules::disease::infect(self, id, d))
    }

    /// Writes `disease` into the immune string of every agent within Euclidean
    /// `radius` of (x, y) (wrapping), over its closest window, then cures any
    /// carried disease the string now contains. The genome is untouched.
    /// Returns how many agents were vaccinated.
    pub fn vaccinate(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        disease: DiseaseId,
    ) -> Result<u32, String> {
        self.disease_on()?;
        let center = self.checked_pos(x, y)?;
        let d = *self
            .diseases
            .get(disease as usize)
            .ok_or_else(|| format!("unknown disease {disease}"))?;
        let r = radius as i32;
        let mut ids = BTreeSet::new();
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    if let Some(id) = self.occupant(self.torus.offset(center, dx, dy)) {
                        ids.insert(id);
                    }
                }
            }
        }
        let mut vaccinated = 0;
        for id in ids {
            if self.agent_mut(id).expect("occupant").immune.imprint(&d) {
                rules::disease::cure_immune(self, id);
                vaccinated += 1;
            }
        }
        Ok(vaccinated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Good, Map, URange};
    use crate::testkit::*;

    #[test]
    fn painting_a_good_changes_only_that_good() {
        let mut w = blank_world(20, 20);
        add_goods(&mut w.config, 2);
        w.paint_capacity(3, 3, 0, 2.0, 1).unwrap();
        assert_eq!(w.site(Pos::new(3, 3)).capacity[..2], [0.0, 2.0]);
        assert!(!w.landscape_edited(0) && w.landscape_edited(1));
        assert!(w.paint_capacity(3, 3, 0, 2.0, 2).is_err(), "no good 2");
    }

    #[test]
    fn painting_sets_capacity_in_a_disc_and_clamps_sugar() {
        let mut w = blank_world(20, 20);
        for i in 0..w.sites.len() {
            w.sites[i].capacity[0] = 4.0;
            w.sites[i].resource[0] = 4.0;
        }
        w.paint_capacity(10, 10, 1, 1.0, 0).unwrap();
        assert!(w.landscape_edited(0));
        for p in [(10, 10), (10, 9), (11, 10), (9, 10), (10, 11)] {
            let s = w.site(Pos::new(p.0, p.1));
            assert_eq!((s.capacity[0], s.resource[0]), (1.0, 1.0), "{p:?}");
        }
        assert_eq!(
            w.site(Pos::new(11, 11)).capacity[0],
            4.0,
            "radius 1 disc excludes diagonals"
        );
        assert!(w.paint_capacity(20, 0, 1, 1.0, 0).is_err());
    }

    #[test]
    fn painting_and_setting_capacity_leaves_a_wall_at_zero() {
        let mut c = crate::testkit::blank_config(20, 20);
        c.walls = vec![crate::config::Wall {
            x: 10,
            y: 10,
            width: 1,
            height: 1,
            opaque: false,
        }];
        let mut w = World::new(c, 1).unwrap();
        let wall = Pos::new(10, 10);
        w.paint_capacity(10, 10, 2, 4.0, 0).unwrap();
        assert_eq!(w.site(wall).capacity[0], 0.0, "painting skips the wall");
        assert_eq!(
            w.site(Pos::new(11, 10)).capacity[0],
            4.0,
            "an open site in the disc is still painted"
        );
        let caps = vec![4.0; 400];
        w.set_capacities(0, &caps).unwrap();
        assert_eq!(
            w.site(wall).capacity[0],
            0.0,
            "set_capacities skips the wall"
        );
        assert_eq!(w.site(Pos::new(0, 0)).capacity[0], 4.0);
    }

    #[test]
    fn placing_an_agent_on_a_wall_is_refused() {
        let mut c = crate::testkit::blank_config(10, 10);
        c.walls = vec![crate::config::Wall {
            x: 3,
            y: 3,
            width: 1,
            height: 1,
            opaque: true,
        }];
        let mut w = World::new(c, 1).unwrap();
        let err = w.place_agent(3, 3, &AgentOverrides::default()).unwrap_err();
        assert!(err.contains("wall"), "{err}");
    }

    #[test]
    fn a_placement_can_set_spice_in_a_two_good_world() {
        let config = match crate::presets::find("iv-1-spice").unwrap().config {
            crate::model::ModelConfig::Sugarscape(c) => c,
            _ => unreachable!(),
        };
        let mut w = World::new(
            Config {
                population: 0,
                ..config
            },
            1,
        )
        .unwrap();
        let overrides = AgentOverrides {
            spice: Some(7.0),
            spice_metabolism: Some(4),
            ..Default::default()
        };
        let id = w.place_agent(2, 3, &overrides).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!(
            (a.holdings[1], a.initial[1], a.metabolism[1]),
            (7.0, 7.0, 4)
        );
        let mut one = blank_world(10, 10);
        assert!(
            one.place_agent(2, 3, &overrides).is_err(),
            "no spice in a one-good world"
        );
    }

    #[test]
    fn a_placement_can_set_its_age_and_birth_endowment() {
        let mut w = blank_world(10, 10);
        let overrides = AgentOverrides {
            sugar: Some(5.0),
            age: Some(70),
            endowment: Some(40.0),
            ..Default::default()
        };
        let id = w.place_agent(2, 3, &overrides).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!((a.age, a.holdings[0], a.initial[0]), (70, 5.0, 40.0));
    }

    #[test]
    fn placing_and_removing_agents() {
        let mut w = blank_world(10, 10);
        let overrides = AgentOverrides {
            vision: Some(3),
            sex: Some(Sex::Male),
            tribe: Some(Tribe::Red),
            ..Default::default()
        };
        let id = w.place_agent(2, 3, &overrides).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!((a.vision, a.sex, a.tribe()), (3, Sex::Male, Tribe::Red));
        assert!(
            w.place_agent(2, 3, &AgentOverrides::default()).is_err(),
            "occupied"
        );
        w.remove_agent(2, 3).unwrap();
        assert_eq!(w.population(), 0);
        assert!(w.events().deaths.is_empty(), "removal is not a death");
        assert!(w.remove_agent(2, 3).is_err());
    }

    #[test]
    fn erasing_a_parent_does_not_bequeath() {
        let mut w = blank_world(10, 10);
        w.config.inheritance.enabled = true;
        let parent = spawn(&mut w, 1, 1);
        let child = spawn(&mut w, 2, 1);
        w.agent_mut(parent).unwrap().holdings[0] = 30.0;
        w.agent_mut(parent).unwrap().children = vec![child];
        let before = w.agent(child).unwrap().holdings[0];
        w.remove_agent(1, 1).unwrap();
        assert_eq!(w.agent(child).unwrap().holdings[0], before);
        assert_eq!(w.occupant(Pos::new(1, 1)), None);
        assert!(w.events().deaths.is_empty());
    }

    /// `seen`, with a truffle spot there seen `ripe` (or not) at `tick`.
    fn with_spot(
        mut seen: crate::minds::memory::Seen,
        ripe: bool,
        tick: u64,
    ) -> crate::minds::memory::Seen {
        seen.truffle = Some(crate::minds::memory::TruffleSeen { ripe, tick });
        seen
    }

    #[test]
    fn inspect_shows_memory_only_when_memory_is_on() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        assert!(w.inspect(2, 2).unwrap().agent.unwrap().memory.is_none());
        w.config.movement.mode = crate::config::MoveMode::Walk;
        w.config.memory.span = 20;
        let seen = crate::minds::memory::Seen::new(&[0.0], &[0.0], 0);
        {
            let a = w.agent_mut(id).unwrap();
            a.remembers = true;
            a.memory.sites.insert(1, seen.clone());
            a.memory.sites.insert(2, with_spot(seen.clone(), true, 0));
        }
        let m = w.inspect(2, 2).unwrap().agent.unwrap().memory.unwrap();
        assert_eq!((m.remembers, m.sites, m.spots), (true, 2, 1));
    }

    #[test]
    fn inspect_shows_the_rate_only_under_mvt() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        w.agent_mut(id).unwrap().rate = 1.25;
        assert!(w.inspect(2, 2).unwrap().agent.unwrap().rate.is_none());
        w.config.movement.mode = crate::config::MoveMode::Walk;
        w.config.decision.rule = crate::config::DecisionRule::Mvt;
        assert_eq!(w.inspect(2, 2).unwrap().agent.unwrap().rate, Some(1.25));
    }

    #[test]
    fn inspect_shows_caching_only_when_on() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        assert!(w.inspect(2, 2).unwrap().agent.unwrap().caching.is_none());
        w.config.caching.rule = crate::config::CachingRule::Even;
        w.config.caching.capacity = 50;
        let site = w.torus.index(Pos::new(3, 3)) as u32;
        w.agent_mut(id).unwrap().caches.insert(site, 12.0);
        let caching = w.inspect(2, 2).unwrap().agent.unwrap().caching.unwrap();
        assert_eq!(caching.rule, crate::config::CachingRule::Even);
        assert_eq!(caching.holdings_cap, 50.0);
        assert_eq!(caching.total, 12.0);
        assert_eq!(caching.caches.len(), 1);
        assert_eq!((caching.caches[0].x, caching.caches[0].y), (3, 3));
        assert_eq!(caching.caches[0].amount, 12.0);
        assert!(caching.forecast.is_none(), "forecast is rule plan's alone");
    }

    #[test]
    fn inspect_reports_plans_forecast_shortfall_outside_winter_only() {
        let mut w = blank_world(10, 10);
        spawn(&mut w, 2, 2);
        w.config.caching.rule = crate::config::CachingRule::Plan;
        w.config.seasons.enabled = true;
        w.config.seasons.mode = crate::config::SeasonMode::Global;
        w.config.seasons.period = 100;
        assert!(
            w.inspect(2, 2)
                .unwrap()
                .agent
                .unwrap()
                .caching
                .unwrap()
                .forecast
                .is_some(),
            "tick 0 is summer"
        );
        w.tick = 150;
        assert!(
            w.inspect(2, 2)
                .unwrap()
                .agent
                .unwrap()
                .caching
                .unwrap()
                .forecast
                .is_none(),
            "no forecast is computed in winter"
        );
    }

    #[test]
    fn a_sites_inspect_lists_every_cache_buried_there() {
        let mut w = blank_world(10, 10);
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 2);
        spawn(&mut w, 3, 3);
        let site = w.torus.index(Pos::new(5, 5)) as u32;
        w.agent_mut(a).unwrap().caches.insert(site, 4.0);
        w.agent_mut(b).unwrap().caches.insert(site, 2.5);
        w.agent_mut(b).unwrap().cheater = true;
        let caches = w.inspect(5, 5).unwrap().site.caches;
        assert_eq!(caches.len(), 2);
        assert_eq!(
            (caches[0].owner, caches[0].amount, caches[0].cheater_owner),
            (a, 4.0, false)
        );
        assert_eq!(
            (caches[1].owner, caches[1].amount, caches[1].cheater_owner),
            (b, 2.5, true)
        );
        assert!(w.inspect(6, 6).unwrap().site.caches.is_empty());
    }

    #[test]
    fn minds_view_sums_caches_per_site_with_owner_flags() {
        let mut w = blank_world(10, 10);
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 2);
        let here = w.torus.index(Pos::new(5, 5)) as u32;
        let there = w.torus.index(Pos::new(7, 3)) as u32;
        w.agent_mut(a).unwrap().caches.insert(here, 4.0);
        w.agent_mut(b).unwrap().caches.insert(here, 2.0);
        w.agent_mut(b).unwrap().caches.insert(there, 1.0);
        w.agent_mut(b).unwrap().cheater = true;
        let v = w.minds_view();
        assert_eq!(
            v.caches,
            vec![
                [7.0, 3.0, 1.0, f64::from(CACHE_CHEATER)],
                [5.0, 5.0, 6.0, f64::from(CACHE_HOARDER | CACHE_CHEATER)],
            ],
            "site order: (7, 3) is row 3"
        );
        assert!(v.homes.is_empty() && v.lab.is_none() && v.winter.is_none());
        // A central world's homes, and the larder among the caches.
        w.config.central.enabled = true;
        w.agent_mut(a).unwrap().home = Some(Pos::new(5, 5));
        let v = w.minds_view();
        assert_eq!(v.homes.len(), 1, "b has no home");
        assert_eq!((v.homes[0].id, v.homes[0].x, v.homes[0].y), (a, 5, 5));
        assert_eq!(v.homes[0].larder, 4.0);
        assert_eq!(v.caches[1][3] as u32 & CACHE_LARDER, CACHE_LARDER);
    }

    #[test]
    fn minds_view_reports_the_winter_of_the_tick_just_computed() {
        let mut w = blank_world(10, 10);
        w.config.seasons.enabled = true;
        w.config.seasons.period = 100;
        assert_eq!(
            w.minds_view().winter,
            None,
            "by hemisphere: no global winter"
        );
        w.config.seasons.mode = crate::config::SeasonMode::Global;
        for (tick, winter) in [
            (0, false),
            (100, false),
            (101, true),
            (200, true),
            (201, false),
        ] {
            w.tick = tick;
            assert_eq!(w.minds_view().winter, Some(winter), "tick {tick}");
        }
    }

    #[test]
    fn minds_view_follows_the_labs_schedule_and_turns() {
        use crate::config::{CachingRule, Lab, LabProtocol};
        use crate::minds::caching::lab;
        let raby = Lab {
            protocol: LabProtocol::Raby,
            food_first: true,
        };
        let c = lab::rig_config(raby, CachingRule::Even, lab::LabParams::default(), 3);
        let mut w = World::new(c, 1).unwrap();
        let v = w.minds_view().lab.unwrap();
        assert_eq!((v.phase, v.day, v.days), ("start", 1, 6));
        assert_eq!(
            v.compartments,
            vec![[1, 1, 3, 3], [5, 1, 3, 3], [9, 1, 3, 3]]
        );
        assert_eq!(
            v.trays,
            vec![[0, 2, 3], [2, 10, 3]],
            "Raby caches in K1 and K3"
        );
        w.step();
        let v = w.minds_view().lab.unwrap();
        assert_eq!(
            (v.phase, v.day, v.place, v.food),
            ("morning", 1, Some(0), Some(true))
        );
        w.run(2);
        let v = w.minds_view().lab.unwrap();
        assert_eq!((v.phase, v.day), ("evening", 1));
        w.run(2);
        let v = w.minds_view().lab.unwrap();
        assert_eq!(
            (v.phase, v.day, v.place, v.food),
            ("morning", 2, Some(2), Some(false))
        );
        w.run(lab::DAY as u32 * 5 - 1);
        assert!(w.agents().all(|a| a.lab_allocation.is_none()));
        w.step();
        let v = w.minds_view().lab.unwrap();
        let first = w.agents().next().unwrap();
        assert_eq!((v.phase, v.day, v.turn), ("test", 7, Some(first.id)));
        assert_eq!(v.turn_at, Some([first.pos.x, first.pos.y]));
        let (x, y) = (first.pos.x, first.pos.y);
        let alloc = w.inspect(x, y).unwrap().agent.unwrap().caching.unwrap();
        assert_eq!(alloc.lab_allocation, Some(vec![(0, 15.0), (2, 15.0)]));
        for _ in 0..500 {
            if lab::finished(&w) {
                break;
            }
            w.step();
        }
        let v = w.minds_view().lab.unwrap();
        assert_eq!((v.phase, v.turn), ("done", None));
    }

    #[test]
    fn inspect_shows_central_only_when_enabled() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        assert!(w.inspect(2, 2).unwrap().agent.unwrap().central.is_none());
        w.config.central.enabled = true;
        w.agent_mut(id).unwrap().home = Some(Pos::new(4, 4));
        w.agent_mut(id).unwrap().last_load = 7.5;
        let central = w.inspect(2, 2).unwrap().agent.unwrap().central.unwrap();
        assert_eq!(central.home, [4, 4]);
        assert_eq!(central.last_load, 7.5);
    }

    #[test]
    fn inspect_reports_the_worlds_current_wall_state_not_configs() {
        let mut c = crate::testkit::blank_config(10, 10);
        c.walls = vec![crate::config::Wall {
            x: 3,
            y: 3,
            width: 1,
            height: 1,
            opaque: true,
        }];
        let mut w = World::new(c, 1).unwrap();
        assert_eq!(w.inspect(3, 3).unwrap().site.wall, 2, "opaque");
        assert_eq!(w.inspect(0, 0).unwrap().site.wall, 0, "free");
        w.open_wall(Pos::new(3, 3));
        assert_eq!(
            w.inspect(3, 3).unwrap().site.wall,
            0,
            "opened at runtime, like a lab doorway"
        );
    }

    #[test]
    fn memory_view_is_empty_with_memory_off_no_agent_or_a_non_rememberer() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        assert!(
            w.memory_view(Pos::new(2, 2)).is_empty(),
            "memory off (span 0)"
        );
        w.config.movement.mode = crate::config::MoveMode::Walk;
        w.config.memory.span = 20;
        assert!(
            w.memory_view(Pos::new(5, 5)).is_empty(),
            "no agent at that site"
        );
        assert!(
            w.memory_view(Pos::new(2, 2)).is_empty(),
            "the agent there doesn't remember"
        );
        w.agent_mut(id).unwrap().remembers = true;
        assert!(
            w.memory_view(Pos::new(2, 2)).is_empty(),
            "remembers, but nothing in memory yet"
        );
    }

    #[test]
    fn memory_view_lists_sites_in_memory_order_with_age_and_spot() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 2, 2);
        w.config.movement.mode = crate::config::MoveMode::Walk;
        w.config.memory.span = 20;
        w.config.memory.belief = crate::config::Belief::Project;
        w.config.truffles.regrow = 10;
        w.tick = 20;
        let blank_seen = crate::minds::memory::Seen::new(&[0.0], &[0.0], 18);
        {
            let a = w.agent_mut(id).unwrap();
            a.remembers = true;
            // Site 1: no known spot, last seen 2 ticks ago.
            a.memory.sites.insert(1, blank_seen.clone());
            // Site 2: a spot seen unripe 5 ticks ago; regrow is 10, so it's
            // still believed unripe.
            a.memory
                .sites
                .insert(2, with_spot(blank_seen.clone(), false, 15));
            // Site 5: a spot seen unripe 15 ticks ago; past regrow, so it's
            // now believed ripe.
            a.memory
                .sites
                .insert(5, with_spot(blank_seen.clone(), false, 5));
        }
        let view = w.memory_view(Pos::new(2, 2));
        assert_eq!(
            view,
            vec![[1, 0, 2, 0], [2, 0, 2, 1], [5, 0, 2, 2],],
            "BTreeMap (site-index) order, each [x, y, age, spot]"
        );
    }

    #[test]
    fn inspect_reports_site_agent_and_lineage() {
        let mut w = blank_world(10, 10);
        let parent = spawn(&mut w, 1, 1);
        let child = spawn(&mut w, 2, 1);
        w.agent_mut(child).unwrap().parents = Some([parent, 999]);
        w.agent_mut(parent).unwrap().children = vec![child];
        let i = w.inspect(2, 1).unwrap();
        let a = i.agent.unwrap();
        assert_eq!(a.id, child);
        assert_eq!(a.tags, "00000000000");
        assert_eq!(a.parents.len(), 2);
        assert!(a.parents[0].alive && !a.parents[1].alive);
        assert!(w.inspect(5, 5).unwrap().agent.is_none());
        assert_eq!(w.locate(child), Some(Pos::new(2, 1)));
        assert_eq!(w.locate(999), None);
    }

    #[test]
    fn set_config_applies_rule_changes_but_rejects_structural_ones() {
        let mut w = blank_world(10, 10);
        let mut next = w.config.clone();
        next.culture.enabled = true;
        w.set_config(next.clone()).unwrap();
        assert!(w.config.culture.enabled);
        next.width = 12;
        let errs = w.set_config(next).unwrap_err();
        assert_eq!(errs[0].field, "width");
        let mut bad = w.config.clone();
        bad.goods[0].metabolism = URange::new(9, 1);
        assert_eq!(
            w.set_config(bad).unwrap_err()[0].field,
            "goods.0.metabolism"
        );
    }

    #[test]
    fn set_config_rejects_adding_a_good_mid_run() {
        let mut w = blank_world(10, 10);
        let mut next = w.config.clone();
        next.add_good(Good {
            map: Map::Flat { capacity: 0.0 },
            ..Good::spice()
        });
        let errs = w.set_config(next).unwrap_err();
        assert_eq!(errs[0].field, "goods");
        assert_eq!(w.config.goods.len(), 1);
    }

    #[test]
    fn set_config_ignores_schedule_entries_that_already_fired() {
        use crate::config::ScheduledChange;
        let mut c = blank_config(10, 10);
        c.lifespan.enabled = true;
        c.schedule = vec![ScheduledChange {
            tick: 1,
            set: [("replacement.enabled".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        }];
        let mut w = World::new(c, 1).unwrap();
        w.run(2);
        assert!(w.config.replacement.enabled, "t=1 entry fired");
        // Re-applying the t=1 entry to this config would enable replacement
        // without lifespan, but that entry is in the past.
        let mut next = w.config.clone();
        next.replacement.enabled = false;
        next.lifespan.enabled = false;
        w.set_config(next).unwrap();
        assert!(!w.config.lifespan.enabled);
        // Entries still to come are validated.
        let mut next = w.config.clone();
        next.schedule.push(ScheduledChange {
            tick: 5,
            set: [("replacement.enabled".to_string(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        });
        assert_eq!(w.set_config(next).unwrap_err()[0].field, "schedule");
    }

    #[test]
    fn inspection_shows_every_good_pollutant_and_loan() {
        let mut w = blank_world(10, 10);
        add_goods(&mut w.config, 2);
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 1);
        {
            let x = w.agent_mut(a).unwrap();
            x.foresight = 3;
            x.metabolism[1] = 2;
        }
        w.originate_loan(a, b, 1, 2.0);
        let view = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!(
            (view.holdings.clone(), view.initial.clone()),
            (vec![10.0, 10.0], vec![10.0, 10.0])
        );
        assert_eq!((view.metabolism.clone(), view.foresight), (vec![0, 2], 3));
        assert_eq!(view.loans.len(), 1);
        assert_eq!((view.loans[0].role, view.loans[0].good), ("lender", 1));
        assert_eq!(view.loans[0].counterparty.id, b);
        let other = w.inspect(2, 1).unwrap().agent.unwrap();
        assert_eq!(other.loans[0].role, "borrower");
        w.site_mut(Pos::new(1, 1)).capacity[1] = 3.0;
        let site = w.inspect(1, 1).unwrap().site;
        assert_eq!(
            (site.resources.clone(), site.capacities.clone()),
            (vec![0.0, 0.0], vec![0.0, 3.0])
        );
        assert_eq!(site.pollution, vec![0.0]);
    }

    #[test]
    fn placed_agents_get_immune_systems_when_disease_is_on() {
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![crate::bits::Bits::parse("1111111111").unwrap()];
        let id = w.place_agent(3, 3, &AgentOverrides::default()).unwrap();
        let a = w.agent(id).unwrap();
        assert_eq!(a.immune.len(), 50);
        assert_eq!(a.immune, a.immune_genome);
        assert!(
            a.diseases.len() <= 1,
            "initial 4 is capped by the list's length"
        );
    }

    #[test]
    fn inspection_shows_immune_strings_diseases_and_infector() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("111").unwrap()];
        let a = spawn(&mut w, 1, 1);
        let b = spawn(&mut w, 2, 1);
        {
            let x = w.agent_mut(a).unwrap();
            x.immune = Bits::parse(&format!("0110{}", "0".repeat(46))).unwrap();
            x.diseases = vec![0];
            x.infected_by = Some(b);
        }
        let v = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!(v.immune, format!("0110{}", "0".repeat(46)));
        assert_eq!(v.immune_genome, "0".repeat(50));
        assert_eq!(v.diseases.len(), 1);
        let d = &v.diseases[0];
        assert_eq!((d.id, d.bits.as_str(), d.distance), (0, "111", 1));
        assert_eq!(v.infected_by.unwrap().id, b);
        assert!(w
            .inspect(2, 1)
            .unwrap()
            .agent
            .unwrap()
            .infected_by
            .is_none());
    }

    #[test]
    fn inspection_gives_the_agents_group() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 1, 1); // all zeros: Blue (6–11 zeros)
        let group = |w: &World| w.inspect(1, 1).unwrap().agent.unwrap().group;
        assert_eq!(group(&w), 0);
        w.agent_mut(id).unwrap().tags = crate::agent::Tags::new(u64::MAX, 11);
        assert_eq!(group(&w), 1, "no zeros: Red");
        w.config.culture.groups = crate::config::three_tribes(11);
        assert_eq!(group(&w), 0, "no zeros: Blue (0–3)");
    }

    #[test]
    fn infect_tool_gives_a_listed_or_new_disease() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("101").unwrap()];
        let a = spawn(&mut w, 1, 1);
        assert!(w.infect(1, 1, 0).unwrap());
        assert!(!w.infect(1, 1, 0).unwrap(), "already carries it");
        assert_eq!(w.agent(a).unwrap().diseases, vec![0]);
        assert!(
            w.agent(a).unwrap().infected_by.is_none(),
            "a tool is not an infector"
        );
        w.infect(1, 1, -1).unwrap();
        assert_eq!(w.diseases.len(), 2, "a new disease is appended");
        assert_ne!(w.diseases[1], w.diseases[0]);
        assert!(w.infect(1, 1, 7).is_err(), "unknown disease");
        assert!(w.infect(5, 5, 0).is_err(), "no agent");
        assert!(
            w.events().infections.is_empty(),
            "edits are not counted as infections"
        );
        w.config.disease.enabled = false;
        assert_eq!(w.infect(1, 1, 0).unwrap_err(), "disease is off");
    }

    #[test]
    fn vaccination_writes_the_disease_into_immune_strings_in_the_brush() {
        use crate::bits::Bits;
        let mut w = blank_world(10, 10);
        w.config.disease.enabled = true;
        w.diseases = vec![Bits::parse("111").unwrap()];
        let near = spawn(&mut w, 5, 5);
        let edge = spawn(&mut w, 5, 6);
        let far = spawn(&mut w, 8, 8);
        for id in [near, edge, far] {
            w.agent_mut(id).unwrap().diseases = vec![0];
        }
        assert_eq!(w.vaccinate(5, 5, 1, 0).unwrap(), 2);
        for id in [near, edge] {
            let a = w.agent(id).unwrap();
            assert!(a.diseases.is_empty(), "cured");
            assert!(
                a.immune.to_bit_string().starts_with("111"),
                "leftmost closest window"
            );
            assert_eq!(
                a.immune_genome.to_bit_string(),
                "0".repeat(50),
                "genome untouched"
            );
        }
        assert_eq!(w.agent(far).unwrap().diseases, vec![0]);
        assert!(w.vaccinate(5, 5, 1, 3).is_err(), "unknown disease");
        let list = w.disease_list();
        assert_eq!(list.len(), 1);
        assert_eq!(
            (list[0].id, list[0].bits.as_str(), list[0].carriers),
            (0, "111", 1)
        );
    }

    #[test]
    fn set_capacities_replaces_a_goods_map_and_clamps_levels() {
        let mut w = blank_world(5, 5);
        add_goods(&mut w.config, 2);
        for s in &mut w.sites {
            s.capacity[1] = 5.0;
            s.resource[1] = 5.0;
        }
        w.sites[0].resource[1] = 0.0;
        let caps: Vec<f64> = (0..25).map(|i| f64::from(i % 3)).collect();
        w.set_capacities(1, &caps).unwrap();
        assert_eq!(w.capacities(1), caps);
        let levels: Vec<f64> = w.sites.iter().map(|s| s.resource[1]).collect();
        assert_eq!(levels, caps, "levels above the new capacity are clamped");
        assert!(w.landscape_edited(1) && !w.landscape_edited(0));
        let err = |r: Result<(), String>| r.unwrap_err();
        assert_eq!(err(w.set_capacities(2, &caps)), "there is no good 2");
        assert_eq!(
            err(w.set_capacities(1, &caps[..24])),
            "expected 25 capacities, got 24"
        );
        for bad in [10.5, -1.0, f64::NAN, f64::INFINITY] {
            let e = err(w.set_capacities(1, &[bad; 25]));
            assert!(e.starts_with("capacities must be between 0 and 10"), "{e}");
        }
        assert_eq!(w.capacities(1), caps, "a rejected call changes nothing");
    }
}
